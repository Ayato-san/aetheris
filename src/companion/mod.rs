//! Supervision and distribution are deliberately independent of GPUI.
pub mod client;
mod diagnostics;
pub mod install;
pub mod managed;
#[cfg(windows)]
pub(crate) mod managed_security;
pub mod manifest;
pub mod migrations;
pub mod policy;
mod program_files;
pub mod releases;
pub mod runtime;
mod update_progress;

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub fn error(message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
    std::io::Error::other(message.into()).into()
}

pub fn data_directory() -> Result<PathBuf> {
    if let Some(root) = std::env::var_os("LOCALAPPDATA") {
        return Ok(PathBuf::from(root).join("Aetheris"));
    }
    if let Some(root) = std::env::var_os("XDG_DATA_HOME") {
        return Ok(PathBuf::from(root).join("Aetheris"));
    }
    Ok(PathBuf::from(
        std::env::var_os("HOME").ok_or_else(|| error("Dossier utilisateur introuvable."))?,
    )
    .join(".local/share/Aetheris"))
}

pub fn durable_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let temp = path.with_extension("tmp");
    let mut file = fs::File::create(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    durable_rename(&temp, path)?;
    #[cfg(unix)]
    if let Some(parent) = path.parent() {
        fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

/// Replace an existing file without an unlink/rename gap on Windows.
pub fn durable_rename(from: &Path, to: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        };
        let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
        let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
        if unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    #[cfg(not(windows))]
    fs::rename(from, to)?;
    Ok(())
}

/// OS-owned lock; process termination releases it without stale PID handling.
pub struct Lease(fs::File);
impl Lease {
    pub fn acquire(path: &Path) -> Result<Self> {
        let mut options = fs::OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(0);
        }
        let file = options.open(path)?;
        #[cfg(unix)]
        {
            use std::os::{fd::AsRawFd, raw::c_int};
            unsafe extern "C" {
                fn flock(fd: c_int, operation: c_int) -> c_int;
            }
            if unsafe { flock(file.as_raw_fd(), 6) } != 0 {
                return Err(error("Une autre instance utilise cette installation."));
            }
        }
        Ok(Self(file))
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        let _ = self.0.sync_all();
    }
}

#[cfg(test)]
mod tests;

/// Cross-process installation ownership is separate from the per-user instance
/// lock: two user sessions must never replace the same program directory.
#[cfg(windows)]
pub struct InstallationLease {
    handle: windows_sys::Win32::Foundation::HANDLE,
    _file: Option<fs::File>,
}
#[cfg(windows)]
impl InstallationLease {
    pub fn acquire(root: &Path) -> Result<Self> {
        Self::acquire_named(root, "installation")
    }

    pub(super) fn acquire_update(root: &Path) -> Result<Self> {
        Self::acquire_named(root, "update")
    }

    fn acquire_named(root: &Path, purpose: &str) -> Result<Self> {
        if managed::is_managed(root) {
            use std::os::windows::fs::OpenOptionsExt;
            use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
            let name = if purpose == "update" {
                "service/update.lock"
            } else {
                "service/instances.lock"
            };
            let file = fs::OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .open(root.join(name))?;
            return Ok(Self {
                handle: std::ptr::null_mut(),
                _file: Some(file),
            });
        }
        use sha2::{Digest, Sha256};
        use windows_sys::Win32::{
            Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0},
            System::Threading::{CreateMutexW, WaitForSingleObject},
        };
        let root = fs::canonicalize(root)?.to_string_lossy().to_lowercase();
        let name = format!(
            "Global\\Aetheris-{}{}\0",
            if purpose == "update" { "update-" } else { "" },
            hex::encode(Sha256::digest(root.as_bytes()))
        )
        .encode_utf16()
        .collect::<Vec<_>>();
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        let acquired = unsafe { WaitForSingleObject(handle, 0) };
        if acquired != WAIT_OBJECT_0 && acquired != WAIT_ABANDONED {
            unsafe {
                CloseHandle(handle);
            }
            return Err(error("Installation déjà utilisée par un autre processus."));
        }
        Ok(Self {
            handle,
            _file: None,
        })
    }
}
#[cfg(windows)]
impl Drop for InstallationLease {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                windows_sys::Win32::System::Threading::ReleaseMutex(self.handle);
                windows_sys::Win32::Foundation::CloseHandle(self.handle);
            }
        }
    }
}
#[cfg(not(windows))]
pub struct InstallationLease {
    _lease: Lease,
}
#[cfg(not(windows))]
impl InstallationLease {
    pub fn acquire(root: &Path) -> Result<Self> {
        Self::acquire_named(root, "installation")
    }

    pub(super) fn acquire_update(root: &Path) -> Result<Self> {
        Self::acquire_named(root, "update")
    }

    fn acquire_named(root: &Path, purpose: &str) -> Result<Self> {
        use sha2::{Digest, Sha256};
        let name = format!(
            "aetheris-{}-{}.lock",
            if purpose == "update" {
                "update"
            } else {
                "install"
            },
            hex::encode(Sha256::digest(
                fs::canonicalize(root)?.as_os_str().as_encoded_bytes()
            ))
        );
        Ok(Self {
            _lease: Lease::acquire(&std::env::temp_dir().join(name))?,
        })
    }
}
