//! Disk boundary: clean reference, encoding preservation and atomic replacement.
use super::EngineIni;
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const BASELINE: &str = include_str!("../../resources/Engine.baseline.ini");
pub const BASELINE_NAME: &str = "Engine.baseline.ini";

// Some Windows network filesystems (including the WSL UNC share) do not
// implement FlushFileBuffers. Keep atomic replacement, but do not reject an
// otherwise valid save solely because that filesystem cannot flush explicitly.
fn sync_file(file: &fs::File) -> io::Result<()> {
    match file.sync_all() {
        Err(error)
            if error.kind() == io::ErrorKind::Unsupported
                || (cfg!(windows) && error.raw_os_error() == Some(50)) =>
        {
            Ok(())
        }
        result => result,
    }
}

/// Never derive defaults from the player's customized document.
pub fn load_baseline(directory: &Path) -> io::Result<EngineIni> {
    let path = directory.join(BASELINE_NAME);
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file) => {
            file.write_all(BASELINE.as_bytes())?;
            sync_file(&file)?;
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let baseline = read(&path)?;
    let section = "/Script/Engine.RendererSettings";
    let dlss = baseline.get(section, "r.NGX.DLSS.Enable");
    let fsr = baseline.get(section, "r.FidelityFX.FSR.Enabled");
    if !matches!(dlss, Some("0" | "1")) || fsr != Some("0") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "La référence d’origine doit contenir DLSS à 1 et FSR à 0.",
        ));
    }
    // Only clean renderer values are authoritative. Paths are retained in the
    // reference if provided, but are never restored into the player's file.
    let mut clean = EngineIni::parse(BASELINE);
    // Retain supplied paths only. Missing paths use the bundled game-content
    // reference; neither source is ever restored into the player's document.
    clean.copy_reference_paths_from(&baseline);
    // Canonicalize legacy reference files, including the old zero DLSS value,
    // missing paths and unrelated overrides, without capturing the player's file.
    let canonical = clean.to_string();
    let original = fs::read(&path)?;
    if original != canonical.as_bytes() {
        let mut access = WriteAccess::new(&path)?;
        let result = atomic_replace(&path, canonical.as_bytes(), Some(&original));
        access.finish(false)?;
        result?;
    }
    Ok(clean)
}

#[derive(Clone, Copy)]
enum Encoding {
    Utf8,
    Utf16Le,
    Utf16Be,
}
fn decode(bytes: &[u8]) -> io::Result<(String, Encoding)> {
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        if !(bytes.len() - 2).is_multiple_of(2) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Engine.ini UTF-16 invalide.",
            ));
        }
        let le = bytes[0] == 0xff;
        let units: Vec<_> = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| {
                if le {
                    u16::from_le_bytes([v[0], v[1]])
                } else {
                    u16::from_be_bytes([v[0], v[1]])
                }
            })
            .collect();
        let text = String::from_utf16(&units)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok((
            text,
            if le {
                Encoding::Utf16Le
            } else {
                Encoding::Utf16Be
            },
        ))
    } else {
        Ok((
            String::from_utf8(bytes.to_vec())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            Encoding::Utf8,
        ))
    }
}
fn encode(text: &str, encoding: Encoding) -> Vec<u8> {
    match encoding {
        Encoding::Utf8 => text.as_bytes().to_vec(),
        Encoding::Utf16Le | Encoding::Utf16Be => {
            let le = matches!(encoding, Encoding::Utf16Le);
            let mut bytes = if le {
                vec![0xff, 0xfe]
            } else {
                vec![0xfe, 0xff]
            };
            for unit in text.encode_utf16() {
                bytes.extend(if le {
                    unit.to_le_bytes()
                } else {
                    unit.to_be_bytes()
                });
            }
            bytes
        }
    }
}
pub fn read(path: &Path) -> io::Result<EngineIni> {
    Ok(EngineIni::parse(&decode(&fs::read(path)?)?.0))
}

/// Re-read before each operation, preserving edits made by the game or user.
pub fn edit(
    path: &Path,
    change: impl FnOnce(&mut EngineIni) -> Result<(), String>,
) -> Result<EngineIni, String> {
    let original = fs::read(path).map_err(|e| e.to_string())?;
    let (text, encoding) = decode(&original).map_err(|e| e.to_string())?;
    let mut document = EngineIni::parse(&text);
    change(&mut document)?;
    let output = encode(&document.to_string(), encoding);
    if output == original {
        protect(path).map_err(|e| format!("Protection en lecture seule : {e}"))?;
        return Ok(document);
    }
    let mut guard = WriteAccess::new(path).map_err(|e| format!("Accès en écriture : {e}"))?;
    let result = atomic_replace(path, &output, Some(&original));
    let restored = guard.finish(result.is_ok());
    match (result, restored) {
        (Ok(()), Ok(())) => Ok(document),
        (Err(error), Ok(())) => Err(error.to_string()),
        (result, Err(error)) => Err(format!(
            "{}Protection du fichier impossible : {error}",
            result.err().map(|e| format!("{e}. ")).unwrap_or_default()
        )),
    }
}

/// Sets the OS file attribute (Windows) or removes write permissions (Unix).
pub fn protect(path: &Path) -> io::Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)
}

#[cfg(unix)]
fn allow_writes(permissions: &mut fs::Permissions) {
    use std::os::unix::fs::PermissionsExt;
    permissions.set_mode(permissions.mode() | 0o200);
}

#[cfg(windows)]
#[allow(
    clippy::permissions_set_readonly_false,
    reason = "Windows clears only FILE_ATTRIBUTE_READONLY; Unix has its own owner-write implementation."
)]
fn allow_writes(permissions: &mut fs::Permissions) {
    permissions.set_readonly(false);
}

struct WriteAccess<'a> {
    path: &'a Path,
    original: fs::Permissions,
    restored: bool,
}
impl<'a> WriteAccess<'a> {
    fn new(path: &'a Path) -> io::Result<Self> {
        let original = fs::metadata(path)?.permissions();
        let mut writable = original.clone();
        allow_writes(&mut writable);
        fs::set_permissions(path, writable)?;
        Ok(Self {
            path,
            original,
            restored: false,
        })
    }
    fn finish(&mut self, success: bool) -> io::Result<()> {
        let result = if success {
            protect(self.path)
        } else {
            fs::set_permissions(self.path, self.original.clone())
        };
        self.restored = result.is_ok();
        result
    }
}
impl Drop for WriteAccess<'_> {
    fn drop(&mut self) {
        if !self.restored {
            // Also restores protection if writing unwinds unexpectedly.
            let _ = fs::set_permissions(self.path, self.original.clone());
        }
    }
}

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
pub fn atomic_replace(path: &Path, output: &[u8], expected: Option<&[u8]>) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Chemin sans dossier parent."))?;
    let temporary: PathBuf = parent.join(format!(
        ".aetheris-{}-{}.tmp",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(output)?;
        if path.exists() {
            fs::set_permissions(&temporary, fs::metadata(path)?.permissions())?;
        }
        sync_file(&file)?;
        drop(file);
        if let Some(expected) = expected
            && fs::read(path)? != expected
        {
            return Err(io::Error::other(
                "Engine.ini a changé pendant l’écriture. Réessayez.",
            ));
        }
        replace(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
#[cfg(not(windows))]
fn replace(from: &Path, to: &Path) -> io::Result<()> {
    fs::rename(from, to)
}
#[cfg(windows)]
fn replace(from: &Path, to: &Path) -> io::Result<()> {
    if !to.exists() {
        return fs::rename(from, to);
    }
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn ReplaceFileW(
            replaced: *const u16,
            replacement: *const u16,
            backup: *const u16,
            flags: u32,
            exclude: *mut std::ffi::c_void,
            reserved: *mut std::ffi::c_void,
        ) -> i32;
    }
    let (from_path, to_path) = (from, to);
    let from: Vec<_> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<_> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    // Both paths are owned, null-terminated UTF-16 and live for the call.
    if unsafe {
        ReplaceFileW(
            to.as_ptr(),
            from.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    } == 0
    {
        let error = io::Error::last_os_error();
        // ReplaceFileW is unsupported on some network filesystems, including
        // WSL UNC shares. A same-directory rename still replaces atomically;
        // never fall back for errors that could indicate a partial replacement.
        if error.raw_os_error() == Some(50) {
            fs::rename(from_path, to_path)
        } else {
            Err(error)
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn directory(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "aetheris-storage-{name}-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
    #[test]
    fn edits_keep_utf16_crlf_unknown_whitespace_and_comments() {
        let root = directory("encoding");
        let path = root.join("Engine.ini");
        let text = "; Joueur é\r\n[Core.System]\r\n  Paths = Custom/Path  \r\n[SystemSettings]\r\n  Unknown =  7  \r\nr.ShadowQuality=5\r\n";
        fs::write(&path, encode(text, Encoding::Utf16Le)).unwrap();
        edit(&path, |ini| {
            ini.set("SystemSettings", "r.ShadowQuality", "1");
            Ok(())
        })
        .unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.starts_with(&[0xff, 0xfe]));
        assert_eq!(
            decode(&bytes).unwrap().0,
            text.replace("r.ShadowQuality=5", "r.ShadowQuality=1")
        );
        #[cfg(windows)]
        for entry in fs::read_dir(&root).unwrap().flatten() {
            let mut permissions = entry.metadata().unwrap().permissions();
            allow_writes(&mut permissions);
            fs::set_permissions(entry.path(), permissions).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn baseline_never_uses_custom_overrides_as_origin() {
        let root = directory("baseline");
        let baseline = load_baseline(&root).unwrap();
        assert_eq!(baseline.get("SystemSettings", "r.ShadowQuality"), None);
        fs::write(root.join(BASELINE_NAME),"[Core.System]\nPaths=Clean\n[/Script/Engine.RendererSettings]\nr.NGX.DLSS.Enable=0\nr.FidelityFX.FSR.Enabled=0\n[SystemSettings]\nr.ShadowQuality=5\n").unwrap();
        let baseline = load_baseline(&root).unwrap();
        assert_eq!(baseline.get("Core.System", "Paths"), Some("Clean"));
        assert_eq!(baseline.get("SystemSettings", "r.ShadowQuality"), None);
        fs::write(
            root.join(BASELINE_NAME),
            BASELINE.replace("fsr.enabled=0", "fsr.enabled=1"),
        )
        .unwrap();
        assert!(load_baseline(&root).is_err());
        #[cfg(windows)]
        for entry in fs::read_dir(&root).unwrap().flatten() {
            let mut permissions = entry.metadata().unwrap().permissions();
            allow_writes(&mut permissions);
            fs::set_permissions(entry.path(), permissions).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn writes_reread_external_edits_and_validation_failure_keeps_file_identical() {
        let root = directory("reread");
        let path = root.join("Engine.ini");
        fs::write(&path, "[SystemSettings]\nr.Unknown=changed\n").unwrap();
        let before = fs::read(&path).unwrap();
        assert!(
            edit(&path, |ini| {
                ini.set("SystemSettings", "r.Unknown", "lost");
                Err("invalid".into())
            })
            .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), before);
        edit(&path, |ini| {
            ini.set("SystemSettings", "r.ShadowQuality", "1");
            Ok(())
        })
        .unwrap();
        assert_eq!(
            read(&path).unwrap().get("SystemSettings", "r.Unknown"),
            Some("changed")
        );
        assert!(atomic_replace(&path, b"lost", Some(&before)).is_err());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        #[cfg(windows)]
        for entry in fs::read_dir(&root).unwrap().flatten() {
            let mut permissions = entry.metadata().unwrap().permissions();
            allow_writes(&mut permissions);
            fs::set_permissions(entry.path(), permissions).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn successful_saves_protect_the_file_and_later_edits_can_still_write() {
        let root = directory("read-only");
        let path = root.join("Engine.ini");
        fs::write(&path, "[SystemSettings]\nr.Unknown=7\n").unwrap();
        for value in ["1", "2"] {
            edit(&path, |doc| {
                doc.set("SystemSettings", "r.ShadowQuality", value);
                Ok(())
            })
            .unwrap();
            assert!(fs::metadata(&path).unwrap().permissions().readonly());
            assert_eq!(
                read(&path)
                    .unwrap()
                    .get("SystemSettings", "r.ShadowQuality"),
                Some(value)
            );
        }
        let before = fs::read(&path).unwrap();
        assert!(edit(&path, |_| Err("invalide".into())).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert!(fs::metadata(&path).unwrap().permissions().readonly());
        // Windows cannot delete a file bearing the read-only attribute.
        let mut guard = WriteAccess::new(&path).unwrap();
        guard.finish(false).unwrap();
        #[cfg(windows)]
        {
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            allow_writes(&mut permissions);
            fs::set_permissions(&path, permissions).unwrap();
        }
        #[cfg(windows)]
        for entry in fs::read_dir(&root).unwrap().flatten() {
            let mut permissions = entry.metadata().unwrap().permissions();
            allow_writes(&mut permissions);
            fs::set_permissions(entry.path(), permissions).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn unchanged_save_still_sets_read_only_and_failed_write_restores_permissions() {
        let root = directory("protection-failure");
        let path = root.join("Engine.ini");
        fs::write(&path, "; unchanged").unwrap();
        edit(&path, |_| Ok(())).unwrap();
        assert!(fs::metadata(&path).unwrap().permissions().readonly());
        let original = fs::read(&path).unwrap();
        {
            let mut guard = WriteAccess::new(&path).unwrap();
            assert!(!fs::metadata(&path).unwrap().permissions().readonly());
            assert!(atomic_replace(&path, b"bad", Some(b"conflicting version")).is_err());
            guard.finish(false).unwrap();
        }
        assert_eq!(fs::read(&path).unwrap(), original);
        assert!(fs::metadata(&path).unwrap().permissions().readonly());
        #[cfg(windows)]
        {
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            allow_writes(&mut permissions);
            fs::set_permissions(&path, permissions).unwrap();
        }
        #[cfg(windows)]
        for entry in fs::read_dir(&root).unwrap().flatten() {
            let mut permissions = entry.metadata().unwrap().permissions();
            allow_writes(&mut permissions);
            fs::set_permissions(entry.path(), permissions).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn upgrades_the_old_zero_dlss_reference_on_disk_and_keeps_paths() {
        let root = directory("baseline-upgrade");
        let path = root.join(BASELINE_NAME);
        fs::write(&path,"[Core.System]\nPaths=Clean\n[/Script/Engine.RendererSettings]\nr.NGX.DLSS.Enable=0\nr.FidelityFX.FSR.Enabled=0\n").unwrap();
        for _ in 0..2 {
            let baseline = load_baseline(&root).unwrap();
            assert_eq!(
                baseline.get("/Script/Engine.RendererSettings", "r.ngx.dlss.enable"),
                Some("1")
            );
            assert_eq!(baseline.get("Core.System", "Paths"), Some("Clean"));
            assert!(
                fs::read_to_string(&path)
                    .unwrap()
                    .contains("r.ngx.dlss.enable=1\nr.fidelityfx.fsr.enabled=0")
            );
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn reference_upgrade_fills_missing_paths_and_discards_non_path_core_settings() {
        let root = directory("reference-paths");
        let path = root.join(BASELINE_NAME);
        fs::write(&path,"[Core.System]\nUnknown=do-not-copy\n[/Script/Engine.RendererSettings]\nr.ngx.dlss.enable=1\nr.fidelityfx.fsr.enabled=0\n").unwrap();
        let baseline = load_baseline(&root).unwrap();
        assert!(baseline.get("Core.System", "Paths").is_some());
        assert!(baseline.get("Core.System", "Unknown").is_none());
        assert!(read(&path).unwrap().get("Core.System", "Paths").is_some());
        fs::remove_dir_all(root).unwrap();
    }
}
