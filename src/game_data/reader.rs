//! Shared archive/protection workflow for maintainers and the future UI worker.
//! This prepares decoded inputs; it neither validates schemas nor activates a DB.
use super::{archives, assets, binary::invalid};
use serde::Serialize;
use std::{
    fs, io,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Mount,
    Extract,
    KeySearch,
    Decrypt,
    Decode,
    Complete,
}
#[derive(Clone, Debug, Serialize)]
pub struct Progress {
    pub phase: Phase,
    pub completed: usize,
    pub total: usize,
    pub path: Option<String>,
}
/// One control object per invocation. Callbacks run on the caller's worker;
/// there is no global key, mutable session or dependency on an UI executor.
pub struct Control<'a> {
    cancel: &'a AtomicBool,
    progress: &'a mut dyn FnMut(Progress),
}
impl<'a> Control<'a> {
    pub fn new(cancel: &'a AtomicBool, progress: &'a mut dyn FnMut(Progress)) -> Self {
        Self { cancel, progress }
    }
    pub fn check(&self) -> io::Result<()> {
        if self.cancel.load(Ordering::Acquire) {
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "client reading cancelled",
            ))
        } else {
            Ok(())
        }
    }
    pub(super) fn emit(
        &mut self,
        phase: Phase,
        completed: usize,
        total: usize,
        path: Option<&str>,
    ) -> io::Result<()> {
        self.check()?;
        (self.progress)(Progress {
            phase,
            completed,
            total,
            path: path.map(str::to_owned),
        });
        self.check()
    }
}
#[derive(Serialize)]
pub struct Report {
    pub state: &'static str,
    pub extracted: usize,
    pub decoded: usize,
    pub app_version: Option<String>,
    pub errors: Vec<String>,
}
fn write(output: &Path, report: &Report) -> io::Result<()> {
    crate::engine_ini::storage::atomic_replace(
        &output.join("reader.json"),
        &serde_json::to_vec_pretty(report)?,
        None,
    )
}
/// A fresh work directory keeps failed or cancelled attempts separate from any
/// prior result. Archive AES and manifest material are deliberately independent.
pub fn read_client(
    paks: &Path,
    archive_key: [u8; 32],
    manifest_material: &[u8; 32],
    output: &Path,
    control: &mut Control<'_>,
) -> io::Result<Report> {
    control.check()?;
    let paks = paks.canonicalize()?;
    if output
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(invalid("output path must not contain parent traversal"));
    }
    if output.exists() {
        return Err(invalid("client reading requires a fresh work directory"));
    }
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = parent.canonicalize()?;
    if parent.starts_with(&paks) {
        return Err(invalid(
            "client reading output must be outside the archive directory",
        ));
    }
    fs::create_dir(output)?;
    let output = output.canonicalize()?;
    let mut report = Report {
        state: "incomplete",
        extracted: 0,
        decoded: 0,
        app_version: None,
        errors: vec![],
    };
    write(&output, &report)?;
    let result = (|| -> io::Result<()> {
        let extracted =
            archives::extract_data_with_control(&paks, &output.join("raw"), archive_key, control)?;
        report.extracted = extracted.files.len();
        if !extracted.errors.is_empty() {
            report.errors = extracted.errors;
            report.state = "partial";
            return Ok(());
        }
        let decoded = assets::decrypt_extraction_with_control(
            &output.join("raw"),
            manifest_material,
            &output.join("decoded"),
            control,
        )?;
        report.decoded = decoded.files.len();
        report.app_version = decoded.app_version;
        report.errors = decoded.errors;
        report.state = if report.errors.is_empty() {
            "complete"
        } else {
            "partial"
        };
        control.emit(Phase::Complete, 1, 1, None)
    })();
    if let Err(error) = result {
        report.state = if error.kind() == io::ErrorKind::Interrupted {
            "cancelled"
        } else {
            "failed"
        };
        report.errors.push(error.to_string());
        write(&output, &report)?;
        return Err(error);
    }
    write(&output, &report)?;
    Ok(report)
}
