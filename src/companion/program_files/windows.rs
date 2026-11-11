//! Restart Manager is queried for diagnostics only: no shutdown or termination.
use crate::companion::{Result, error, manifest::PROGRAMS};
use std::{
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{ERROR_MORE_DATA, FILETIME, WAIT_OBJECT_0},
    System::{
        RestartManager::*,
        Threading::{
            GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
            WaitForSingleObject,
        },
    },
};

struct Session(u32);
impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            RmEndSession(self.0);
        }
    }
}

impl Session {
    fn new(paths: &[PathBuf]) -> Result<Self> {
        let mut handle = 0;
        let mut key = [0; CCH_RM_SESSION_KEY as usize + 1];
        check(unsafe { RmStartSession(&mut handle, 0, key.as_mut_ptr()) })?;
        let session = Self(handle);
        let names: Vec<Vec<u16>> = paths
            .iter()
            .map(|path| path.as_os_str().encode_wide().chain(Some(0)).collect())
            .collect();
        let names: Vec<_> = names.iter().map(|name| name.as_ptr()).collect();
        check(unsafe {
            RmRegisterResources(
                handle,
                names.len() as u32,
                names.as_ptr(),
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
            )
        })?;
        Ok(session)
    }

    fn blockers(&self) -> Result<Vec<RM_PROCESS_INFO>> {
        let mut needed = 0;
        let mut count = 0;
        let mut reasons = 0;
        let mut apps = Vec::new();
        for _ in 0..8 {
            let code = unsafe {
                RmGetList(
                    self.0,
                    &mut needed,
                    &mut count,
                    if apps.is_empty() {
                        std::ptr::null_mut()
                    } else {
                        apps.as_mut_ptr()
                    },
                    &mut reasons,
                )
            };
            if code == 0 {
                apps.truncate(count as usize);
                apps.retain(is_running);
                return Ok(apps);
            }
            if code != ERROR_MORE_DATA {
                check(code)?;
            }
            if needed > 4096 {
                return Err(error(
                    "Trop de processus utilisent les programmes Aetheris.",
                ));
            }
            count = needed;
            apps.resize(count as usize, RM_PROCESS_INFO::default());
        }
        Err(error(
            "Les processus utilisant Aetheris changent trop rapidement. Réessayez.",
        ))
    }
}

fn check(code: u32) -> Result<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(code as i32).into())
    }
}

fn is_running(app: &RM_PROCESS_INFO) -> bool {
    let raw = unsafe {
        OpenProcess(
            PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
            0,
            app.Process.dwProcessId,
        )
    };
    if raw.is_null() {
        // A disappeared PID is harmless; denied inspection must fail closed.
        return std::io::Error::last_os_error().raw_os_error() != Some(87);
    }
    let process = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut created = FILETIME::default();
    let mut exited = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    if unsafe {
        GetProcessTimes(
            process.as_raw_handle(),
            &mut created,
            &mut exited,
            &mut kernel,
            &mut user,
        )
    } != 0
        && (created.dwLowDateTime != app.Process.ProcessStartTime.dwLowDateTime
            || created.dwHighDateTime != app.Process.ProcessStartTime.dwHighDateTime)
    {
        return false;
    } // Restart Manager's PID was recycled.
    unsafe { WaitForSingleObject(process.as_raw_handle(), 0) != WAIT_OBJECT_0 }
}

fn describe(apps: &[RM_PROCESS_INFO]) -> String {
    apps.iter()
        .map(|app| {
            let length = app
                .strAppName
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(app.strAppName.len());
            format!(
                "{} (PID {})",
                String::from_utf16_lossy(&app.strAppName[..length]),
                app.Process.dwProcessId
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn wait(root: &Path, progress: impl Fn(&str), timeout: Duration) -> Result<()> {
    let paths: Vec<_> = PROGRAMS
        .iter()
        .map(|name| root.join(name))
        .filter(|path| path.is_file())
        .collect();
    wait_for_paths(&paths, progress, timeout)
}

pub(super) fn wait_for_installer(root: &Path) -> Result<()> {
    let paths: Vec<_> = PROGRAMS
        .iter()
        .map(|name| root.join(name))
        .chain(
            PROGRAMS
                .iter()
                .map(|name| root.join("service/installer-stage").join(name)),
        )
        .chain(Some(root.join("service/bootstrap.exe")))
        .filter(|path| path.is_file())
        .collect();
    wait_for_paths(&paths, |_| {}, Duration::from_secs(30))
}

fn wait_for_paths(paths: &[PathBuf], progress: impl Fn(&str), timeout: Duration) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let session = Session::new(paths)?;
    let started = Instant::now();
    loop {
        let apps = session.blockers()?;
        if apps.is_empty() {
            return Ok(());
        }
        let detail = describe(&apps);
        if apps
            .iter()
            .any(|app| app.Process.dwProcessId == std::process::id())
        {
            return Err(error(format!(
                "Le programme de remplacement utilise lui-même un fichier à remplacer : {detail}. Exécutez le helper depuis sa copie temporaire."
            )));
        }
        if started.elapsed() >= timeout {
            return Err(error(format!(
                "Des programmes Aetheris sont encore utilisés : {detail}. Fermez les processus indiqués puis réessayez ; le remplacement reste suspendu."
            )));
        }
        progress(&format!(
            "Attente de la fermeture complète des programmes : {detail}"
        ));
        thread::sleep(Duration::from_millis(100).min(timeout.saturating_sub(started.elapsed())));
    }
}

pub(super) fn replace(from: &Path, to: &Path) -> Result<()> {
    let started = Instant::now();
    loop {
        match crate::companion::durable_rename(from, to) {
            Ok(()) => return Ok(()),
            Err(cause) => {
                let transient = cause
                    .downcast_ref::<std::io::Error>()
                    .and_then(|e| e.raw_os_error())
                    .is_some_and(|code| matches!(code, 5 | 32 | 33));
                if !transient || started.elapsed() >= Duration::from_secs(5) {
                    let detail = Session::new(&[to.to_owned()])
                        .and_then(|s| s.blockers())
                        .ok()
                        .filter(|apps| !apps.is_empty())
                        .map(|apps| format!(" Utilisé par : {}.", describe(&apps)))
                        .unwrap_or_default();
                    return Err(error(format!(
                        "Remplacement de {} impossible : {cause}.{detail}",
                        to.display()
                    )));
                }
                // The process can release instances.lock just before Windows
                // unloads its executable; scanners can also hold short handles.
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::fs::OpenOptionsExt;

    #[test]
    fn short_sharing_lock_is_retried_without_an_unlink_gap() {
        let directory = crate::companion::tests::Directory::new();
        let old = directory.0.join("aetheris-companion.exe");
        let new = directory.0.join("replacement");
        std::fs::write(&old, b"old").unwrap();
        std::fs::write(&new, b"new").unwrap();
        // Readers may inspect the old image, but replacement is denied while
        // this non-delete-sharing handle exists (as with transient scanners).
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&old)
            .unwrap();
        let target = old.clone();
        let worker = thread::spawn(move || replace(&new, &target));
        thread::sleep(Duration::from_millis(200));
        assert!(!worker.is_finished());
        assert_eq!(std::fs::read(&old).unwrap(), b"old");
        drop(held);
        worker.join().unwrap().unwrap();
        assert_eq!(std::fs::read(old).unwrap(), b"new");
    }
}
