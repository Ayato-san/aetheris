//! Installer-owned updates. SCM controls carry no paths, URLs or executable
//! arguments. Only the service downloads into its administrator-owned directory.
use super::{Result, error, policy::Channel};
use std::{fs, path::Path};

#[cfg(windows)]
pub(crate) fn registered_root() -> Option<std::path::PathBuf> {
    windows::registered_root()
}
#[cfg(windows)]
pub(crate) fn service_command(arguments: &[&str]) -> Result<()> {
    windows::sc(arguments)
}
#[cfg(windows)]
pub(crate) fn stop_named_service(name: &str) -> Result<()> {
    windows::stop_named_service(name)
}

pub fn current_is_managed() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(is_managed))
        .unwrap_or(false)
}
pub fn is_managed(root: &Path) -> bool {
    #[cfg(windows)]
    {
        windows::registered_root().is_some_and(|registered| {
            matches!((fs::canonicalize(registered), fs::canonicalize(root)), (Ok(a), Ok(b)) if a == b)
        })
    }
    #[cfg(not(windows))]
    {
        let _ = root;
        false
    }
}
pub fn channel(root: &Path) -> Channel {
    if fs::read(root.join("service/channel")).ok().as_deref() == Some(b"prerelease") {
        Channel::Prerelease
    } else {
        Channel::Stable
    }
}
pub fn set_channel(channel: Channel) -> Result<()> {
    #[cfg(windows)]
    {
        windows::control(if channel == Channel::Stable { 128 } else { 129 })
    }
    #[cfg(not(windows))]
    {
        let _ = channel;
        Err(error("Service disponible uniquement sous Windows."))
    }
}
pub fn handoff(root: &Path, operation: &str, source: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        windows::handoff(root, operation, source)
    }
    #[cfg(not(windows))]
    {
        let _ = (root, operation, source);
        Err(error("Installation gérée indisponible."))
    }
}
pub fn confirm_launch() -> Result<()> {
    #[cfg(windows)]
    {
        windows::control(134)
    }
    #[cfg(not(windows))]
    {
        Ok(())
    }
}
pub fn entry() -> Result<()> {
    #[cfg(windows)]
    if let Some(mode) = std::env::args().nth(1) {
        return match mode.as_str() {
            "--service" => windows::dispatch(),
            "--capture-service" => crate::capture::service::dispatch(),
            "--stop-capture-service" => crate::capture::service::stop(),
            "--remove-capture-service" => crate::capture::service::uninstall(),
            "--managed-worker" => windows::worker(),
            "--managed-wait" => windows::wait(),
            "--installer-check" => windows::installer_check(),
            "--installer-wait" => windows::installer_wait(),
            "--installer-recover" => windows::installer_recover(),
            "--installer-apply" => windows::installer_apply(),
            "--stop-service" => windows::stop_service(),
            "--configure-service" => windows::configure_service(),
            _ => super::runtime::run_helper(),
        };
    }
    super::runtime::run_helper()
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::{
        path::PathBuf,
        process::Command,
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicU32, Ordering},
        },
        time::Duration,
    };
    use windows_sys::Win32::{
        Foundation::ERROR_CALL_NOT_IMPLEMENTED,
        System::{Registry::*, Services::*},
    };
    const NAME: &str = "AetherisUpdate";
    static COMMAND: AtomicU32 = AtomicU32::new(0);
    static STOP: AtomicBool = AtomicBool::new(false);
    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }
    pub(super) fn sc(arguments: &[&str]) -> Result<()> {
        let mut buffer = [0u16; 32768];
        let count = unsafe {
            windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW(
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            )
        };
        if count == 0 || count as usize >= buffer.len() {
            return Err(error("Dossier système absent."));
        }
        let program = PathBuf::from(String::from_utf16(&buffer[..count as usize])?).join("sc.exe");
        if !Command::new(program).args(arguments).status()?.success() {
            return Err(error("Configuration du service impossible."));
        }
        Ok(())
    }
    pub fn configure_service() -> Result<()> {
        let root = std::env::current_exe()?
            .parent()
            .ok_or_else(|| error("Dossier absent."))?
            .to_owned();
        super::super::managed_security::validate(&root, true)?;
        super::super::manifest::verify(&root, &super::super::manifest::public_key()?)?;
        let executable = format!(
            "\"{}\" --service",
            root.join("service/bootstrap.exe").display()
        );
        let existing = unsafe {
            let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
            if scm.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            let service = OpenServiceW(scm, wide(NAME).as_ptr(), SERVICE_QUERY_STATUS);
            let exists = !service.is_null();
            if exists {
                CloseServiceHandle(service);
            }
            CloseServiceHandle(scm);
            exists
        };
        sc(&[
            if existing { "config" } else { "create" },
            NAME,
            "binPath=",
            &executable,
            "start=",
            "auto",
            "obj=",
            "LocalSystem",
            "DisplayName=",
            "Aetheris Update",
        ])?;
        // AU may query status and send fixed custom controls, never change the
        // executable, start/stop/delete the service or modify its security.
        sc(&["sdset", NAME, "D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;LCCR;;;AU)"])?;
        crate::capture::service::configure()
    }
    pub fn stop_service() -> Result<()> {
        stop_named_service(NAME)
    }
    pub(super) fn stop_named_service(name: &str) -> Result<()> {
        unsafe {
            let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
            if scm.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            let service = OpenServiceW(
                scm,
                wide(name).as_ptr(),
                SERVICE_STOP | SERVICE_QUERY_STATUS,
            );
            if service.is_null() {
                let cause = std::io::Error::last_os_error();
                CloseServiceHandle(scm);
                return if cause.raw_os_error() == Some(1060) {
                    Ok(())
                } else {
                    Err(cause.into())
                };
            }
            let result = (|| -> Result<()> {
                let mut state = std::mem::zeroed::<SERVICE_STATUS>();
                if QueryServiceStatus(service, &mut state) == 0 {
                    return Err(std::io::Error::last_os_error().into());
                }
                if state.dwCurrentState != SERVICE_STOPPED {
                    if ControlService(service, SERVICE_CONTROL_STOP, &mut state) == 0 {
                        return Err(std::io::Error::last_os_error().into());
                    }
                    let started = std::time::Instant::now();
                    loop {
                        if QueryServiceStatus(service, &mut state) == 0 {
                            return Err(std::io::Error::last_os_error().into());
                        }
                        if state.dwCurrentState == SERVICE_STOPPED {
                            break;
                        }
                        if started.elapsed() > Duration::from_secs(300) {
                            return Err(error(
                                "Service encore occupé. Réessayez après la mise à jour.",
                            ));
                        }
                        std::thread::sleep(Duration::from_millis(200));
                    }
                }
                Ok(())
            })();
            CloseServiceHandle(service);
            CloseServiceHandle(scm);
            result
        }
    }
    pub fn installer_check() -> Result<()> {
        let root = PathBuf::from(
            std::env::args_os()
                .nth(2)
                .ok_or_else(|| error("Destination absente."))?,
        );
        let parent = root
            .parent()
            .ok_or_else(|| error("Dossier parent absent."))?;
        super::super::managed_security::validate(parent, false)?;
        if !root.exists() {
            return Ok(());
        }
        if !is_managed(&root) {
            if fs::read_dir(&root)?.next().is_some() {
                return Err(error(
                    "Choisissez un dossier vide pour cette nouvelle installation.",
                ));
            }
            return Ok(());
        }
        let key = super::super::manifest::public_key()?;
        let installed = super::super::manifest::authenticate(&root, &key).or_else(|_| {
            super::super::manifest::authenticate(&root.join("service/installed"), &key)
        })?;
        let incoming = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
        if incoming.cmp_precedence(&installed.version).is_lt() {
            return Err(error(
                "Cet installeur est plus ancien que la version installée.",
            ));
        }
        Ok(())
    }
    pub fn installer_wait() -> Result<()> {
        let root = PathBuf::from(
            std::env::args_os()
                .nth(2)
                .ok_or_else(|| error("Destination absente."))?,
        );
        super::super::program_files::wait_for_installer(&root)
    }
    fn record_version(root: &Path) -> Result<()> {
        let record = root.join("service/installed");
        fs::create_dir_all(&record)?;
        for name in [
            super::super::manifest::MANIFEST,
            super::super::manifest::SIGNATURE,
        ] {
            super::super::durable_write(&record.join(name), &fs::read(root.join(name))?)?;
        }
        Ok(())
    }
    pub fn installer_apply() -> Result<()> {
        let root = registered_root().ok_or_else(|| error("Installation absente."))?;
        super::super::install::apply(&root, &root.join("service/installer-stage"), &[])?;
        record_version(&root)
    }
    pub fn installer_recover() -> Result<()> {
        let root = PathBuf::from(
            std::env::args_os()
                .nth(2)
                .ok_or_else(|| error("Destination absente."))?,
        );
        if is_managed(&root) {
            super::super::install::recover(&root)?;
        }
        Ok(())
    }
    pub fn registered_root() -> Option<PathBuf> {
        let mut bytes = 0;
        let key = wide("SOFTWARE\\Aetheris");
        let value = wide("InstallDir");
        unsafe {
            if RegGetValueW(
                HKEY_LOCAL_MACHINE,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ | RRF_SUBKEY_WOW6464KEY,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut bytes,
            ) != 0
            {
                return None;
            }
            let mut buffer = vec![0u16; bytes as usize / 2];
            if RegGetValueW(
                HKEY_LOCAL_MACHINE,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ | RRF_SUBKEY_WOW6464KEY,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &mut bytes,
            ) != 0
            {
                return None;
            }
            Some(PathBuf::from(
                String::from_utf16(buffer.split(|c| *c == 0).next()?).ok()?,
            ))
        }
    }
    fn root() -> Result<PathBuf> {
        let executable = std::env::current_exe()?;
        let root = executable
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| error("Service hors installation."))?;
        if !is_managed(root) {
            return Err(error("Service non enregistré pour cette installation."));
        }
        super::super::managed_security::validate(root, true)?;
        Ok(root.to_owned())
    }
    pub fn control(code: u32) -> Result<()> {
        unsafe {
            let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
            if scm.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            let service = OpenServiceW(scm, wide(NAME).as_ptr(), SERVICE_USER_DEFINED_CONTROL);
            let mut status = std::mem::zeroed();
            let result = if service.is_null() || ControlService(service, code, &mut status) == 0 {
                Err(error(format!(
                    "Le service de mise à jour ne peut pas répondre : {}",
                    std::io::Error::last_os_error()
                )))
            } else {
                Ok(())
            };
            if !service.is_null() {
                CloseServiceHandle(service);
            }
            CloseServiceHandle(scm);
            result
        }
    }
    fn status(root: &Path, message: &str) -> Result<()> {
        super::super::durable_write(&root.join("service/status"), message.as_bytes())
    }
    unsafe extern "system" fn handler(
        code: u32,
        _: u32,
        _: *mut std::ffi::c_void,
        _: *mut std::ffi::c_void,
    ) -> u32 {
        if code == SERVICE_CONTROL_STOP || code == SERVICE_CONTROL_SHUTDOWN {
            STOP.store(true, Ordering::SeqCst);
            return 0;
        }
        if code == SERVICE_CONTROL_INTERROGATE {
            return 0;
        }
        if !(128..=134).contains(&code) {
            return ERROR_CALL_NOT_IMPLEMENTED;
        }
        if COMMAND
            .compare_exchange(0, u32::MAX, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return 170;
        }
        let result = (|| -> Result<()> {
            let root = root()?;
            if code == 134 {
                let marker = root.join(".aetheris-previous/first-launch");
                if marker.exists() {
                    fs::remove_file(marker)?;
                }
                COMMAND.store(0, Ordering::SeqCst);
            } else if code < 130 {
                super::super::durable_write(
                    &root.join("service/channel"),
                    if code == 128 {
                        b"stable"
                    } else {
                        b"prerelease"
                    },
                )?;
                COMMAND.store(0, Ordering::SeqCst);
            } else {
                status(&root, "queued")?;
                COMMAND.store(code, Ordering::SeqCst);
            }
            Ok(())
        })();
        if result.is_err() {
            COMMAND.store(0, Ordering::SeqCst);
            return 1;
        }
        0
    }
    pub fn dispatch() -> Result<()> {
        let name = wide(NAME);
        let table = [
            SERVICE_TABLE_ENTRYW {
                lpServiceName: name.as_ptr().cast_mut(),
                lpServiceProc: Some(service_main),
            },
            SERVICE_TABLE_ENTRYW {
                lpServiceName: std::ptr::null_mut(),
                lpServiceProc: None,
            },
        ];
        if unsafe { StartServiceCtrlDispatcherW(table.as_ptr()) } == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(())
    }
    unsafe extern "system" fn service_main(_: u32, _: *mut *mut u16) {
        let handle = unsafe {
            RegisterServiceCtrlHandlerExW(wide(NAME).as_ptr(), Some(handler), std::ptr::null_mut())
        };
        if handle.is_null() {
            return;
        }
        let mut state = SERVICE_STATUS {
            dwServiceType: SERVICE_WIN32_OWN_PROCESS,
            dwCurrentState: SERVICE_RUNNING,
            dwControlsAccepted: SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN,
            dwWin32ExitCode: 0,
            dwServiceSpecificExitCode: 0,
            dwCheckPoint: 0,
            dwWaitHint: 0,
        };
        unsafe {
            SetServiceStatus(handle, &state);
        }
        if let Ok(root) = root() {
            // Recover before accepting a new update after service/power failure.
            if root.join(".aetheris-transaction/journal.json").exists() {
                let result = operation(&root, 132);
                let _ = status(
                    &root,
                    &result
                        .err()
                        .map(|e| format!("error: {e}"))
                        .unwrap_or_else(|| "done".into()),
                );
            }
            while !STOP.load(Ordering::SeqCst) {
                let code = COMMAND.load(Ordering::SeqCst);
                if (130..=133).contains(&code) {
                    let result = operation(&root, code);
                    let _ = status(
                        &root,
                        &result
                            .err()
                            .map(|e| format!("error: {e}"))
                            .unwrap_or_else(|| "done".into()),
                    );
                    COMMAND.store(0, Ordering::SeqCst);
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        }
        state.dwCurrentState = SERVICE_STOPPED;
        state.dwControlsAccepted = 0;
        unsafe {
            SetServiceStatus(handle, &state);
        }
    }
    fn operation(root: &Path, code: u32) -> Result<()> {
        use std::os::windows::fs::OpenOptionsExt;
        let _update = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(root.join("service/update.lock"))?;
        let _activity = Activity::new()?;
        status(
            root,
            "Attente de la fermeture de toutes les sessions Aetheris…",
        )?;
        let _exclusive = loop {
            match exclusive(root) {
                Ok(lease) => break lease,
                Err(_) if STOP.load(Ordering::SeqCst) => {
                    return Err(error("Service arrêté avant installation."));
                }
                Err(_) => std::thread::sleep(Duration::from_millis(250)),
            }
        };
        crate::capture::service::with_paused(|| operation_without_capture(root, code))
    }
    fn operation_without_capture(root: &Path, code: u32) -> Result<()> {
        if code == 132 {
            return super::super::install::recover(root).map(|_| ());
        }
        if code == 131 {
            return super::super::install::restore_previous(root);
        }
        super::super::install::recover(root)?;
        let key = super::super::manifest::public_key()?;
        let installed = super::super::manifest::authenticate(root, &key)?;
        let release = if code == 133 {
            super::super::releases::exact(&installed.version)?
        } else {
            let Some(release) = super::super::releases::latest(channel(root), &installed.version)?
            else {
                return Ok(());
            };
            release
        };
        let stage = root.join("service/stage");
        if stage.exists() {
            fs::remove_dir_all(&stage)?;
        }
        status(root, "Téléchargement et vérification de la version signée…")?;
        let downloaded =
            super::super::releases::download(&release, &stage, Arc::new(AtomicBool::new(false)))?;
        if downloaded
            .version
            .cmp_precedence(&installed.version)
            .is_lt()
        {
            return Err(error("Rétrogradation automatique interdite."));
        }
        // The newly authenticated helper owns the migration history for skipped releases.
        let result = Command::new(stage.join("aetheris-update-helper.exe"))
            .arg("--managed-worker")
            .arg(root)
            .status()?;
        if !result.success() {
            let detail = fs::read_to_string(root.join("service/status"))
                .ok()
                .and_then(|message| message.strip_prefix("error: ").map(str::to_owned));
            return Err(error(detail.unwrap_or_else(|| {
                format!("Échec du helper authentifié ({result}) ; récupération nécessaire.")
            })));
        }
        Ok(())
    }
    pub fn worker() -> Result<()> {
        let root = registered_root().ok_or_else(|| error("Installation absente."))?;
        if std::env::args_os().nth(2).as_deref() != Some(root.as_os_str()) {
            return Err(error("Destination étrangère à l’installation."));
        }
        let stage = root.join("service/stage");
        // This process never accepts a personal database path or user staging directory.
        let result = crate::capture::service::with_reconfigured(|| {
            super::super::install::apply(&root, &stage, &[])?;
            record_version(&root)
        });
        if let Err(cause) = &result {
            let _ = status(&root, &format!("error: {cause}"));
        }
        result
    }
    pub fn handoff(root: &Path, operation: &str, source: &Path) -> Result<()> {
        let code = match operation {
            "install" => 130,
            "restore" => 131,
            "recover" => 132,
            "repair" => 133,
            _ => return Err(error("Commande inconnue.")),
        };
        let authenticated =
            super::super::manifest::verify(source, &super::super::manifest::public_key()?)?;
        let mut random = [0; 16];
        getrandom::fill(&mut random).map_err(|e| error(e.to_string()))?;
        let directory =
            super::super::data_directory()?.join(format!("managed-helper-{}", hex::encode(random)));
        fs::create_dir_all(&directory)?;
        let executable = directory.join("aetheris-update-helper.exe");
        fs::copy(source.join("aetheris-update-helper.exe"), &executable)?;
        if super::super::manifest::hash(&executable)?
            != authenticated.files["aetheris-update-helper.exe"]
        {
            return Err(error("La copie du helper authentifié a été modifiée."));
        }
        control(code)?;
        Command::new(executable)
            .arg("--managed-wait")
            .arg(root)
            .spawn()?;
        Ok(())
    }
    pub fn wait() -> Result<()> {
        let root = PathBuf::from(
            std::env::args_os()
                .nth(2)
                .ok_or_else(|| error("Installation absente."))?,
        );
        if !is_managed(&root) {
            return Err(error("Installation étrangère au service."));
        }
        let progress = super::super::update_progress::Progress::start()?;
        loop {
            let message = fs::read_to_string(root.join("service/status"))?;
            if message == "done" {
                break;
            }
            if message.starts_with("error:") {
                return Err(error(message));
            }
            progress.set(&message);
            std::thread::sleep(Duration::from_millis(250));
        }
        Command::new(root.join("aetheris-companion.exe")).spawn()?;
        Ok(())
    }
    pub fn exclusive(root: &Path) -> Result<fs::File> {
        use std::os::windows::fs::OpenOptionsExt;
        Ok(fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(root.join("service/instances.lock"))?)
    }
    pub struct Activity(windows_sys::Win32::Foundation::HANDLE);
    impl Activity {
        pub fn new() -> Result<Self> {
            let handle = unsafe {
                windows_sys::Win32::System::Threading::CreateMutexW(
                    std::ptr::null(),
                    0,
                    wide("Global\\AetherisManagedActivity").as_ptr(),
                )
            };
            if handle.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            Ok(Self(handle))
        }
    }
    impl Drop for Activity {
        fn drop(&mut self) {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(self.0);
            }
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use std::os::windows::fs::OpenOptionsExt;
        #[test]
        fn managed_readers_coexist_and_updater_waits_without_terminating_them() {
            let directory = crate::companion::tests::Directory::new();
            fs::create_dir(directory.0.join("service")).unwrap();
            let path = directory.0.join("service/instances.lock");
            fs::write(&path, b"").unwrap();
            let reader = || {
                fs::OpenOptions::new()
                    .read(true)
                    .share_mode(1)
                    .open(&path)
                    .unwrap()
            };
            let first = reader();
            let second = reader();
            assert!(exclusive(&directory.0).is_err());
            drop(first);
            assert!(exclusive(&directory.0).is_err());
            drop(second);
            let update = exclusive(&directory.0).unwrap();
            assert!(
                fs::OpenOptions::new()
                    .read(true)
                    .share_mode(1)
                    .open(&path)
                    .is_err()
            );
            drop(update);
            drop(reader());
        }
    }
}
