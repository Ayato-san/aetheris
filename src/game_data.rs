//! Inspection of game archives before any extraction or catalog publication.
//!
//! An inventory only identifies files: it does not assert that they are readable
//! or that a particular game version or schema is supported.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self},
    path::{Path, PathBuf},
};

pub mod cache;
pub mod catalog;
pub mod discovery;
pub mod protection;
pub mod schema;

/// Version information is accepted only from explicit client metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClientIdentity {
    pub version: String,
    pub region: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Inventory {
    pub client: Option<ClientIdentity>,
    pub archives: Vec<Archive>,
    pub fingerprint: String,
    pub problems: Vec<String>,
    /// Evidence is retained separately from the optional operator declaration.
    #[serde(default)]
    pub metadata: Vec<SourceMetadata>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceMetadata {
    pub path: String,
    pub sha256: String,
    pub kind: String,
    pub version: Option<String>,
    pub region: Option<String>,
}

/// Builds a portable snapshot. Unknown identity remains unknown. The optional
/// `aetheris-client.json` is a user-supplied declaration, never auto-generated.
pub fn inventory(root: &Path) -> io::Result<Inventory> {
    let archives = inspect(root)?;
    let metadata = root.join("aetheris-client.json");
    let mut evidence = discovery::steam_metadata(root)?;
    let client: Option<ClientIdentity> = match fs::read(&metadata) {
        Ok(bytes) => {
            let identity: ClientIdentity = serde_json::from_slice(&bytes)?;
            if identity.version.trim().is_empty() || identity.region.trim().is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "empty client identity",
                ));
            }
            evidence.push(SourceMetadata {
                path: "aetheris-client.json".into(),
                sha256: hex::encode(Sha256::digest(&bytes)),
                kind: "operator_declaration".into(),
                version: Some(identity.version.clone()),
                region: Some(identity.region.clone()),
            });
            Some(identity)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let mut problems = Vec::new();
    let paths: std::collections::BTreeSet<_> = archives
        .iter()
        .map(|a| a.path.iter().collect::<PathBuf>())
        .collect();
    for path in &paths {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if extension.eq_ignore_ascii_case("utoc") || extension.eq_ignore_ascii_case("ucas") {
            let companion = if extension.eq_ignore_ascii_case("utoc") {
                "ucas"
            } else {
                "utoc"
            };
            // Compare inventory paths rather than assuming extension casing on disk.
            // IoStore partitions are named <container>_s1.ucas, _s2.ucas, ...
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let base = if extension.eq_ignore_ascii_case("ucas") {
                stem.rsplit_once("_s")
                    .filter(|(_, suffix)| {
                        !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit())
                    })
                    .map_or(stem, |(base, _)| base)
            } else {
                stem
            };
            let expected = path.with_file_name(base);
            if !paths.iter().any(|other| {
                other.with_extension("") == expected
                    && other
                        .extension()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| s.eq_ignore_ascii_case(companion))
            }) {
                problems.push(format!(
                    "Missing {companion} companion for {}",
                    path.display()
                ));
            }
        }
    }
    let fingerprint = hex::encode(Sha256::digest(serde_json::to_vec(&(
        &client, &archives, &evidence,
    ))?));
    Ok(Inventory {
        client,
        archives,
        fingerprint,
        problems,
        metadata: evidence,
    })
}

impl Inventory {
    /// Rehashes the source before activation, catching replacements, additions,
    /// removals and identity changes throughout a long extraction.
    pub fn verify_unchanged(&self, root: &Path) -> io::Result<()> {
        if *self != inventory(root)? {
            return Err(io::Error::other("client source changed during processing"));
        }
        Ok(())
    }
}

/// Discovery only suggests existing game roots. Explicit paths always work
/// independently of a launcher. Callers may pass Steam library/common folders,
/// copied-client parents, or installation roots.
pub fn discover(search_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut found = std::collections::BTreeSet::new();
    for root in search_roots {
        let candidates = std::iter::once(root.clone()).chain(
            fs::read_dir(root)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path()),
        );
        for candidate in candidates {
            if ["Aion2/Content/Paks", "AION2/Content/Paks", "Content/Paks"]
                .iter()
                .any(|suffix| candidate.join(suffix).is_dir())
                && let Ok(path) = candidate.canonicalize()
            {
                let installation = if path.join("Content/Paks").is_dir()
                    && path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| s.eq_ignore_ascii_case("Aion2"))
                {
                    path.parent().unwrap_or(&path).to_path_buf()
                } else {
                    path
                };
                found.insert(installation);
            }
        }
    }
    found.into_iter().collect()
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Archive {
    /// Relative path, with components kept separately for platform independence.
    pub path: Vec<String>,
    pub size: u64,
    pub sha256: String,
}

/// Inventories recognized container files without following symbolic links.
/// An unreadable directory, non-Unicode path, or changed file fails the entire
/// inspection rather than silently publishing an incomplete inventory.
pub fn inspect(root: &Path) -> io::Result<Vec<Archive>> {
    if !fs::symlink_metadata(root)?.file_type().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "client path is not a directory",
        ));
    }
    let mut paths = Vec::new();
    visit(root, root, &mut paths)?;
    paths.sort();
    if paths.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no pak, utoc, or ucas files found",
        ));
    }
    paths
        .into_iter()
        .map(|(path, components)| {
            let initial = fs::symlink_metadata(&path)?;
            if !initial.is_file() {
                return Err(io::Error::other("archive replaced before inspection"));
            }
            let mut file = File::open(&path)?;
            let before = file.metadata()?;
            let mut hash = Sha256::new();
            let size = io::copy(&mut file, &mut hash)?;
            let after = file.metadata()?;
            let current = fs::symlink_metadata(&path)?;
            if size != before.len()
                || before.len() != after.len()
                || before.modified()? != after.modified()?
                || !current.is_file()
                || current.len() != after.len()
                || current.modified()? != after.modified()?
                || initial.modified()? != before.modified()?
            {
                return Err(io::Error::other("archive changed during inspection"));
            }
            Ok(Archive {
                path: components,
                size,
                sha256: hex::encode(hash.finalize()),
            })
        })
        .collect()
}

fn visit(root: &Path, dir: &Path, paths: &mut Vec<(PathBuf, Vec<String>)>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_symlink()
            && path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| {
                    ["pak", "utoc", "ucas"]
                        .iter()
                        .any(|candidate| ext.eq_ignore_ascii_case(candidate))
                })
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "symbolic archive links are not supported",
            ));
        }
        if kind.is_dir() {
            visit(root, &path, paths)?;
        } else if kind.is_file()
            && path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| {
                    ["pak", "utoc", "ucas"]
                        .iter()
                        .any(|candidate| ext.eq_ignore_ascii_case(candidate))
                })
        {
            let relative = path.strip_prefix(root).map_err(io::Error::other)?;
            let components = relative
                .components()
                .map(|c| {
                    c.as_os_str().to_str().map(str::to_owned).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "archive path is not Unicode")
                    })
                })
                .collect::<io::Result<Vec<_>>>()?;
            paths.push((path, components));
        }
    }
    Ok(())
}
