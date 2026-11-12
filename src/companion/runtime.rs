use super::{
    Lease, Result,
    client::{Command as AppCommand, Event},
    error, install, manifest,
    policy::{self, Preferences},
    releases::{self, Release},
};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn activate_native() -> bool {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            FindWindowW, IsIconic, SW_RESTORE, SetForegroundWindow, ShowWindow,
        };
        let title: Vec<u16> = "Aetheris\0".encode_utf16().collect();
        let update_title: Vec<u16> = super::update_progress::TITLE
            .encode_utf16()
            .chain(Some(0))
            .collect();
        unsafe {
            let update = FindWindowW(std::ptr::null(), update_title.as_ptr());
            let window = if update.is_null() {
                FindWindowW(std::ptr::null(), title.as_ptr())
            } else {
                update
            };
            if !window.is_null() {
                if IsIconic(window) != 0 {
                    ShowWindow(window, SW_RESTORE);
                }
                SetForegroundWindow(window);
                return true;
            }
        }
    }
    false
}

struct ChildGuard(std::process::Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if matches!(self.0.try_wait(), Ok(None)) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

pub fn confirm(message: &str) -> bool {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            IDYES, MB_ICONWARNING, MB_YESNO, MessageBoxW,
        };
        let title: Vec<u16> = "Aetheris\0".encode_utf16().collect();
        let message: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
        unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                message.as_ptr(),
                title.as_ptr(),
                MB_YESNO | MB_ICONWARNING,
            ) == IDYES
        }
    }
    #[cfg(not(windows))]
    {
        eprintln!("{message}\nConfirmer avec « oui » :");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).is_ok() && input.trim() == "oui"
    }
}
pub fn report(message: &str) {
    notify(message, true);
}

fn notify(message: &str, failure: bool) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MessageBoxW,
        };
        let title: Vec<u16> = "Aetheris\0".encode_utf16().collect();
        let message: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
        unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                message.as_ptr(),
                title.as_ptr(),
                MB_OK
                    | if failure {
                        MB_ICONERROR
                    } else {
                        MB_ICONINFORMATION
                    },
            );
        }
    }
    #[cfg(not(windows))]
    {
        let _ = failure;
        eprintln!("{message}");
    }
}

pub fn installation_is_updating(root: &Path) -> bool {
    match super::InstallationLease::acquire_update(root) {
        Ok(probe) => {
            drop(probe);
            false
        }
        Err(_) => {
            // This very process maps aetheris-companion.exe. A modal dialog
            // here would keep that image locked without an installation lease,
            // preventing the updater from replacing it. The external helper or
            // installer already owns the progress UI; activate it and exit.
            activate_native();
            true
        }
    }
}
fn unique() -> Result<String> {
    let mut bytes = [0; 32];
    getrandom::fill(&mut bytes).map_err(|e| error(e.to_string()))?;
    Ok(hex::encode(bytes))
}
fn preferences(path: &Path) -> Preferences {
    fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}
fn save_preferences(path: &Path, prefs: &Preferences) -> Result<()> {
    super::durable_write(path, &serde_json::to_vec(prefs)?)
}

pub fn run() -> Result<()> {
    let root = std::env::current_exe()?
        .parent()
        .ok_or_else(|| error("Dossier d’installation absent."))?
        .to_owned();
    let data = super::data_directory()?;
    fs::create_dir_all(&data)?;
    // The helper takes this lock before acknowledging the handoff and keeps it
    // until replacement is complete, including while waiting for this process.
    if installation_is_updating(&root) {
        return Ok(());
    }
    let _lease = match Lease::acquire(&data.join("companion.lock")) {
        Ok(lease) => lease,
        Err(_) => {
            activate_native();
            super::durable_write(&data.join("activate"), b"activate")?;
            return Ok(());
        }
    };
    // Recheck under companion ownership: an updater may have begun its
    // handoff between the initial probe and acquisition of this lease.
    if installation_is_updating(&root) {
        return Ok(());
    }
    let _installation = match super::InstallationLease::acquire(&root) {
        Ok(lease) => lease,
        Err(cause) => {
            activate_native();
            report(&format!(
                "Installation déjà utilisée ou inaccessible : {cause}"
            ));
            return Ok(());
        }
    };
    match Lease::acquire(&data.join("application.lock")) {
        Ok(probe) => drop(probe),
        Err(_) => {
            activate_native();
            super::durable_write(&data.join("activate"), b"activate")?;
            return Ok(());
        }
    }
    let (initial_path, mut initial_file) = launch_log(&data)?;
    if root.join(".aetheris-transaction/journal.json").exists() {
        writeln!(initial_file, "Récupération d’une mise à jour interrompue")?;
        initial_file.sync_all()?;
        return helper(&root, &data, "recover", &root);
    }
    let preferences_path = data.join("updates.json");
    let mut prefs = preferences(&preferences_path);
    if super::managed::is_managed(&root) {
        prefs.channel = super::managed::channel(&root);
    }
    // Unsigned development builds remain usable without weakening distribution.
    let development = cfg!(debug_assertions) && !root.join(manifest::MANIFEST).exists();
    if !development {
        loop {
            match manifest::public_key().and_then(|key| manifest::verify(&root, &key)) {
                Ok(manifest) => {
                    writeln!(
                        initial_file,
                        "Intégrité vérifiée ; version installée {}",
                        manifest.version
                    )?;
                    break;
                }
                Err(cause) => {
                    writeln!(initial_file, "Lancement bloqué : {cause}")?;
                    initial_file.sync_all()?;
                    if !confirm(&format!(
                        "{cause}\nJournal local : {}\nLe lancement est bloqué. Réparer depuis une release signée en conservant les données personnelles ?",
                        initial_path.display()
                    )) {
                        return Ok(());
                    }
                    match repair(&root, &data, &prefs) {
                        Ok(stage) => return helper(&root, &data, "install", &stage),
                        Err(cause) => {
                            if !confirm(&format!(
                                "Réparation impossible : {cause}\nConnexion nécessaire. Le lancement reste bloqué. Réessayer ?"
                            )) {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }
    let mut initial_log = Some((initial_path, initial_file));
    loop {
        match supervise(
            &root,
            &data,
            &mut prefs,
            &preferences_path,
            development,
            initial_log.take(),
        )? {
            Outcome::Exit => return Ok(()),
            Outcome::Restart => {}
            Outcome::Install(stage) => return helper(&root, &data, "install", &stage),
            Outcome::Restore => return helper(&root, &data, "restore", &root),
        }
    }
}
fn repair(root: &Path, data: &Path, prefs: &Preferences) -> Result<PathBuf> {
    let authenticated = manifest::public_key()
        .and_then(|key| manifest::authenticate(root, &key))
        .ok();
    if let Some(manifest) = authenticated
        && let Ok(release) = releases::exact(&manifest.version)
    {
        let stage = data.join(format!("repair-{}", unique()?));
        if releases::download(&release, &stage, Arc::new(AtomicBool::new(false)))
            .is_ok_and(|downloaded| downloaded == manifest)
        {
            return Ok(stage);
        }
    }
    let release = releases::latest(prefs.channel, &semver::Version::new(0, 0, 0))?
        .ok_or_else(|| error("Aucune release admissible."))?;
    if !confirm(&format!(
        "La version installée ne peut pas être rétablie avec une identité authentifiée. Installer la version {} ? La version peut changer ; vos données sont conservées.",
        release.tag_name
    )) {
        return Err(error("Réparation annulée."));
    }
    let stage = data.join(format!("repair-{}", unique()?));
    releases::download(&release, &stage, Arc::new(AtomicBool::new(false)))?;
    Ok(stage)
}

enum Outcome {
    Exit,
    Restart,
    Install(PathBuf),
    Restore,
}
enum Worker {
    Checked(Result<Option<Release>>, bool, policy::Channel),
    Downloaded(Result<PathBuf>, SystemTime),
}
fn send(stream: &mut Option<TcpStream>, command: AppCommand) {
    if let Some(stream) = stream {
        let _ = serde_json::to_writer(&mut *stream, &command);
        let _ = stream.write_all(b"\n");
    }
}
fn status(stream: &mut Option<TcpStream>, message: impl Into<String>, available: bool) {
    send(
        stream,
        AppCommand::Status {
            message: message.into(),
            available,
        },
    );
}
fn check(
    tx: mpsc::Sender<Worker>,
    channel: policy::Channel,
    installed: semver::Version,
    startup: bool,
) {
    std::thread::spawn(move || {
        let _ = tx.send(Worker::Checked(
            releases::latest(channel, &installed),
            startup,
            channel,
        ));
    });
}
fn download(tx: mpsc::Sender<Worker>, release: Release, data: PathBuf, cancel: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let result = (|| {
            let stage = data.join(format!("update-{}", unique()?));
            releases::download(&release, &stage, cancel)?;
            Ok(stage)
        })();
        let _ = tx.send(Worker::Downloaded(result, SystemTime::now()));
    });
}
fn diagnostic_tail(path: &Path) -> String {
    use std::io::{Read, Seek, SeekFrom};
    let text = (|| -> std::io::Result<String> {
        let mut file = fs::File::open(path)?;
        let length = file.metadata()?.len();
        file.seek(SeekFrom::Start(length.saturating_sub(4000)))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    })();
    text.unwrap_or_else(|_| "Aucune information supplémentaire disponible.".into())
}

fn launch_log(data: &Path) -> Result<(PathBuf, fs::File)> {
    let logs = data.join("logs");
    fs::create_dir_all(&logs)?;
    let name = format!(
        "launch-{:020}-{}.log",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        unique()?
    );
    let path = logs.join(name);
    let mut files: Vec<_> = fs::read_dir(&logs)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name().to_string_lossy().starts_with("launch-")
                && e.path().extension().is_some_and(|e| e == "log")
        })
        .collect();
    files.sort_by_key(|e| e.file_name());
    let remove = files.len().saturating_sub(9);
    for entry in files.into_iter().take(remove) {
        fs::remove_file(entry.path())?;
    }
    let mut file = fs::File::create(&path)?;
    writeln!(
        file,
        "Aetheris {} / {} {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    )?;
    #[cfg(windows)]
    {
        // `ver` is local and reports the actual Windows build without requiring
        // a GPUI window or version-sensitive manifest compatibility flags.
        use std::os::windows::process::CommandExt;
        if let Ok(output) = Command::new("cmd.exe")
            .creation_flags(0x08000000)
            .args(["/c", "ver"])
            .output()
        {
            file.write_all(&output.stdout)?;
        }
    }
    writeln!(file, "Démarrage du compagnon")?;
    Ok((path, file))
}
fn supervise(
    root: &Path,
    data: &Path,
    prefs: &mut Preferences,
    prefs_path: &Path,
    development: bool,
    initial_log: Option<(PathBuf, fs::File)>,
) -> Result<Outcome> {
    let (log_path, mut log) = match initial_log {
        Some(log) => log,
        None => launch_log(data)?,
    };
    writeln!(log, "Démarrage du processus application")?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let token = unique()?;
    let app = root.join(if cfg!(windows) {
        "aetheris.exe"
    } else {
        "aetheris"
    });
    let child = match Command::new(app)
        .env(
            "AETHERIS_COMPANION_ADDRESS",
            listener.local_addr()?.to_string(),
        )
        .env("AETHERIS_COMPANION_TOKEN", &token)
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log.try_clone()?))
        .spawn()
    {
        Ok(child) => child,
        Err(cause) => {
            writeln!(log, "Échec du lancement : {cause}")?;
            return Ok(
                if super::diagnostics::show(
                    &format!(
                        "Impossible de démarrer Aetheris : {cause}\nJournal local : {}\n{}\nRéessayer ?",
                        log_path.display(),
                        diagnostic_tail(&log_path)
                    ),
                    &log_path,
                    "Relancer Aetheris",
                ) {
                    Outcome::Restart
                } else {
                    Outcome::Exit
                },
            );
        }
    };
    let mut child = ChildGuard(child);
    let started = Instant::now();
    let first = root.join(".aetheris-previous/first-launch").exists();
    let mut window_ready = None;
    let mut normal = false;
    let mut connection = None;
    let mut pending_status = None;
    let (events_tx, events) = mpsc::channel();
    let (worker_tx, workers) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let installed = if development {
        semver::Version::parse(env!("CARGO_PKG_VERSION"))?
    } else {
        manifest::authenticate(root, &manifest::public_key()?)?.version
    };
    let mut checking = !development;
    if checking {
        check(worker_tx.clone(), prefs.channel, installed.clone(), true);
    }
    let mut checked_at = Instant::now();
    let mut offered: Option<Release> = None;
    let mut downloading = false;
    let mut download_invalidated = false;
    let mut staged = None;
    let mut closed_at = None;
    let mut ready_at = None;
    let mut lifecycle = policy::UpdateLifecycle::default();
    let mut pending_close = false;
    loop {
        if let Ok((stream, _)) = listener.accept() {
            // Windows accepted sockets inherit the listener's nonblocking mode.
            // The dedicated reader must wait for later events, not stop after
            // the first WouldBlock (losing WindowReady and NormalShutdown).
            stream.set_nonblocking(false)?;
            stream.set_write_timeout(Some(Duration::from_secs(2)))?;
            let token = token.clone();
            let events_tx = events_tx.clone();
            let writer = stream.try_clone()?;
            std::thread::spawn(move || {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_ok()
                    && matches!(serde_json::from_str::<Event>(&line), Ok(Event::Hello(secret)) if secret == token)
                {
                    let _ = reader.get_ref().set_read_timeout(None);
                    let _ = events_tx.send((Some(writer), None));
                    for line in reader.lines() {
                        let Ok(line) = line else {
                            break;
                        };
                        if let Ok(event) = serde_json::from_str(&line) {
                            let _ = events_tx.send((None, Some(event)));
                        }
                    }
                }
            });
        }
        while let Ok((stream, event)) = events.try_recv() {
            if let Some(stream) = stream {
                connection = Some(stream);
                send(&mut connection, AppCommand::Channel(prefs.channel));
                if let Some((message, available)) = pending_status.take() {
                    status(&mut connection, message, available);
                }
            }
            if let Some(event) = event {
                writeln!(log, "{event:?}")?;
                match event {
                    Event::WindowReady => {
                        window_ready.get_or_insert_with(Instant::now);
                    }
                    Event::Interaction => {
                        lifecycle.interact();
                        if !lifecycle.should_close() {
                            pending_close = false;
                        }
                    }
                    Event::NormalShutdown => normal = true,
                    Event::Closing(at) => {
                        closed_at = Some(at);
                        if downloading {
                            lifecycle.closing(true);
                            cancel.store(true, Ordering::Relaxed);
                        }
                    }
                    Event::Update => {
                        lifecycle.request();
                        if staged.is_some() {
                            pending_close = true;
                        } else if !downloading && let Some(release) = offered.clone() {
                            downloading = true;
                            status(&mut connection, "Téléchargement et vérification…", false);
                            download(worker_tx.clone(), release, data.to_owned(), cancel.clone());
                        }
                    }
                    Event::CloseBlocked => {
                        lifecycle.blocked();
                        status(
                            &mut connection,
                            "Mise à jour prête. Enregistrez vos saisies puis réessayez.",
                            true,
                        );
                    }
                    Event::Channel(channel) => {
                        if super::managed::is_managed(root) {
                            super::managed::set_channel(channel)?;
                        }
                        prefs.channel = channel;
                        if offered
                            .as_ref()
                            .and_then(Release::version)
                            .is_some_and(|version| !channel.accepts(&version))
                        {
                            if downloading {
                                download_invalidated = true;
                                cancel.store(true, Ordering::Relaxed);
                            }
                            offered = None;
                            staged = None;
                            pending_close = false;
                            lifecycle.blocked();
                            status(
                                &mut connection,
                                "Changement de canal : recherche d’une version admissible…",
                                false,
                            );
                        }
                        send(&mut connection, AppCommand::Channel(channel));
                        save_preferences(prefs_path, prefs)?;
                        if !checking && !downloading && staged.is_none() {
                            checking = true;
                            check(worker_tx.clone(), channel, installed.clone(), false);
                        }
                    }
                    _ => {}
                }
            }
        }
        while let Ok(worker) = workers.try_recv() {
            match worker {
                Worker::Checked(result, startup, channel) => {
                    if channel != prefs.channel {
                        check(worker_tx.clone(), prefs.channel, installed.clone(), false);
                        continue;
                    }
                    checking = false;
                    checked_at = Instant::now();
                    match result {
                        Ok(Some(release)) => {
                            let excluded = prefs.excluded.as_ref() == release.version().as_ref();
                            let background = startup
                                && policy::offer(started.elapsed(), excluded)
                                    == policy::Offer::BackgroundDownload;
                            let message = format!("Mise à jour {} disponible.", release.tag_name);
                            offered = Some(release.clone());
                            if connection.is_none() {
                                pending_status = Some((message.clone(), true));
                            }
                            status(&mut connection, message, true);
                            if background {
                                lifecycle.background_download();
                                downloading = true;
                                download(
                                    worker_tx.clone(),
                                    release,
                                    data.to_owned(),
                                    cancel.clone(),
                                );
                            }
                        }
                        Ok(None) => status(&mut connection, "Aetheris est à jour.", false),
                        Err(cause) => {
                            writeln!(log, "Vérification des mises à jour : {cause}")?;
                            let message = format!(
                                "Mises à jour indisponibles : {cause}. L’intégrité locale reste vérifiée hors ligne."
                            );
                            if connection.is_none() {
                                pending_status = Some((message.clone(), false));
                            }
                            status(&mut connection, message, false);
                        }
                    }
                }
                Worker::Downloaded(result, finished_at) => {
                    downloading = false;
                    if lifecycle.cancelled() && !policy::install_after_close(finished_at, closed_at)
                    {
                        continue;
                    }
                    if download_invalidated {
                        download_invalidated = false;
                        cancel.store(false, Ordering::Relaxed);
                        if !checking {
                            checking = true;
                            check(worker_tx.clone(), prefs.channel, installed.clone(), false);
                        }
                        continue;
                    }
                    match result {
                        Ok(stage) => {
                            ready_at = Some(finished_at);
                            staged = Some(stage);
                            status(
                                &mut connection,
                                "Mise à jour vérifiée, prête à installer.",
                                true,
                            );
                            if lifecycle.should_close() {
                                pending_close = true;
                            }
                        }
                        Err(cause) => {
                            writeln!(log, "Téléchargement refusé : {cause}")?;
                            status(
                                &mut connection,
                                format!("Échec de mise à jour : {cause}. Vous pouvez réessayer."),
                                offered.is_some(),
                            );
                        }
                    }
                }
            }
        }
        if pending_close && connection.is_some() && window_ready.is_some() {
            send(
                &mut connection,
                AppCommand::CloseForUpdate {
                    unattended: lifecycle.unattended(),
                },
            );
            pending_close = false;
        }
        if super::managed::is_managed(root) {
            let shared = super::managed::channel(root);
            if prefs.channel != shared {
                prefs.channel = shared;
                send(&mut connection, AppCommand::Channel(shared));
                // Existing checked results carry their channel and are rejected below.
                if !checking && !downloading && staged.is_none() {
                    checking = true;
                    check(worker_tx.clone(), shared, installed.clone(), false);
                }
            }
        }
        let activate = data.join("activate");
        if activate.exists() && connection.is_some() {
            let _ = fs::remove_file(activate);
            send(&mut connection, AppCommand::Activate);
        }
        if !checking
            && !downloading
            && staged.is_none()
            && checked_at.elapsed() >= Duration::from_secs(3600)
        {
            checking = true;
            check(worker_tx.clone(), prefs.channel, installed.clone(), false);
        }
        if first
            && root.join(".aetheris-previous/first-launch").exists()
            && window_ready.is_some_and(|t: Instant| t.elapsed() >= Duration::from_secs(10))
        {
            if super::managed::is_managed(root) {
                let _ = super::managed::confirm_launch();
            } else {
                let _ = fs::remove_file(root.join(".aetheris-previous/first-launch"));
            }
        }
        if let Some(exit) = child.0.try_wait()? {
            // The IPC reader runs on another thread. Wait for its final event
            // rather than classifying a just-exited process before it is read.
            if !normal {
                let deadline = Instant::now() + Duration::from_millis(500);
                while Instant::now() < deadline {
                    match events.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                        Ok((_, Some(Event::Closing(at)))) => {
                            closed_at = Some(at);
                        }
                        Ok((_, Some(Event::NormalShutdown))) => {
                            normal = true;
                            break;
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            }
            // Drain the last IPC messages before classification; a normal signal
            // can still be queued when the process handle becomes signalled.
            while let Ok((_, event)) = events.try_recv() {
                match event {
                    Some(Event::NormalShutdown) => normal = true,
                    Some(Event::Closing(at)) => closed_at = Some(at),
                    _ => {}
                }
            }
            let normal = normal && exit.success();
            writeln!(
                log,
                "Sortie du processus : {exit}; fermeture normale explicite : {normal}"
            )?;
            log.sync_all()?;
            cancel.store(true, Ordering::Relaxed);
            if normal {
                if let Some(stage) = staged
                    && ready_at.is_some_and(|ready| policy::install_after_close(ready, closed_at))
                {
                    return Ok(Outcome::Install(stage));
                }
                return Ok(Outcome::Exit);
            }
            if first
                && policy::first_launch_crash(normal, window_ready.map(|t| t.elapsed()))
                && install::can_restore(root)
                && super::diagnostics::show(
                    &format!(
                        "La nouvelle version a quitté de façon inattendue ({exit}) avant dix secondes de fenêtre ouverte.\nJournal local : {}\nRestaurer la version précédente ? Les données après le crash seront copiées séparément ; l’ancienne sauvegarde redeviendra active.",
                        log_path.display()
                    ),
                    &log_path,
                    "Restaurer la version précédente",
                )
            {
                prefs.excluded = Some(installed.clone());
                save_preferences(prefs_path, prefs)?;
                return Ok(Outcome::Restore);
            }
            return Ok(
                if super::diagnostics::show(
                    &format!(
                        "Aetheris s’est arrêté de façon inattendue ({exit}). La cause ne peut pas être déduite de ce seul code.\nJournal local : {}\n{}\nRelancer Aetheris ?",
                        log_path.display(),
                        diagnostic_tail(&log_path)
                    ),
                    &log_path,
                    "Relancer Aetheris",
                ) {
                    Outcome::Restart
                } else {
                    Outcome::Exit
                },
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn helper(root: &Path, data: &Path, operation: &str, stage: &Path) -> Result<()> {
    let recovery_source;
    let source = if operation == "install" {
        stage
    } else if operation == "recover" {
        recovery_source = install::recovery_source(root)?;
        &recovery_source
    } else {
        root
    };
    let authenticated = manifest::verify(source, &manifest::public_key()?)?;
    if super::managed::is_managed(root) {
        let operation = if operation == "install"
            && manifest::authenticate(root, &manifest::public_key()?)
                .ok()
                .zip(manifest::authenticate(stage, &manifest::public_key()?).ok())
                .is_some_and(|(installed, incoming)| installed.version == incoming.version)
        {
            "repair"
        } else {
            operation
        };
        return super::managed::handoff(root, operation, source);
    }
    let directory = data.join(format!("helper-{}", unique()?));
    fs::create_dir(&directory)?;
    let executable = directory.join(if cfg!(windows) {
        "aetheris-update-helper.exe"
    } else {
        "aetheris-update-helper"
    });
    fs::copy(
        source.join(if cfg!(windows) {
            "aetheris-update-helper.exe"
        } else {
            "aetheris-update-helper"
        }),
        &executable,
    )?;
    let helper_name = if cfg!(windows) {
        "aetheris-update-helper.exe"
    } else {
        "aetheris-update-helper"
    };
    if cfg!(windows) && manifest::hash(&executable)? != authenticated.files[helper_name] {
        return Err(error("La copie temporaire du helper a été modifiée."));
    }
    let mut child = Command::new(executable)
        .arg(operation)
        .arg(root)
        .arg(stage)
        .arg(std::process::id().to_string())
        .spawn()?;
    let started = Instant::now();
    while !directory.join("ready").exists() {
        if child.try_wait()?.is_some() {
            return Err(error(
                "L’outil de mise à jour s’est arrêté avant de prendre le relais.",
            ));
        }
        if started.elapsed() >= Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error(
                "L’outil de mise à jour ne répond pas. L’installation n’a pas commencé.",
            ));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Ok(())
}

pub fn run_helper() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 4 {
        return Err(error("Arguments du helper invalides."));
    }
    let root = PathBuf::from(&args[1]);
    let stage = PathBuf::from(&args[2]);
    let update_lease = super::InstallationLease::acquire_update(&root)?;
    let progress = super::update_progress::Progress::start()?;
    // Acknowledge only once launches are blocked and the progress window exists.
    let executable = std::env::current_exe()?;
    super::durable_write(&executable.with_file_name("ready"), b"ready")?;
    #[cfg(windows)]
    {
        use windows_sys::Win32::{
            Foundation::{CloseHandle, WAIT_TIMEOUT},
            System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
        };
        let parent: u32 = args[3].to_string_lossy().parse()?;
        let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, parent) };
        if !handle.is_null() {
            let result = unsafe { WaitForSingleObject(handle, 20000) };
            unsafe {
                CloseHandle(handle);
            }
            if result == WAIT_TIMEOUT {
                return Err(error("Le compagnon précédent ne s’est pas arrêté."));
            }
        }
    }
    let operation = args[0].to_string_lossy();
    let data = super::data_directory()?;
    let started = Instant::now();
    let lease = loop {
        match Lease::acquire(&data.join("companion.lock")) {
            Ok(lease) => break lease,
            Err(cause) if started.elapsed() >= Duration::from_secs(20) => return Err(cause),
            Err(_) => std::thread::sleep(Duration::from_millis(100)),
        }
    };
    let installation_lease = super::InstallationLease::acquire(&root)?;
    let application_lease = Lease::acquire(&data.join("application.lock"))?;
    let mut update_log = fs::File::create(data.join("update.log"))?;
    writeln!(
        update_log,
        "Aetheris {} : {operation}",
        env!("CARGO_PKG_VERSION")
    )?;
    let result = match operation.as_ref() {
        "install" => install::apply_with_progress(
            &root,
            &stage,
            &[
                data.join("debug-captures.sqlite3"),
                data.join(crate::combat_history::storage::DATABASE_NAME),
            ],
            |message| progress.set(message),
        ),
        "restore" => {
            progress.set("Restauration de la version précédente…");
            install::restore_previous(&root)
        }
        "recover" => {
            progress.set("Récupération de l’installation interrompue…");
            install::recover(&root).map(|_| ())
        }
        _ => Err(error("Opération inconnue.")),
    };
    if let Err(cause) = result {
        progress.set("La mise à jour a échoué. Consultez le message de diagnostic.");
        writeln!(update_log, "Échec : {cause}")?;
        update_log.sync_all()?;
        if operation == "install" {
            let path = data.join("updates.json");
            let mut prefs = preferences(&path);
            prefs.excluded = manifest::public_key()
                .and_then(|key| manifest::authenticate(&stage, &key))
                .ok()
                .map(|manifest| manifest.version);
            if let Err(cause) = save_preferences(&path, &prefs) {
                writeln!(update_log, "Impossible de mémoriser l’exclusion : {cause}")?;
            }
        }
        report(&format!(
            "Mise à jour ou récupération impossible : {cause}\nJournal local : {}\nVérifiez les permissions du dossier. Aucun privilège administrateur n’est demandé.",
            data.join("update.log").display()
        ));
        // Never relaunch into an unresolved journal: that would automatically
        // start another helper and create a recovery loop.
        while root.join(".aetheris-transaction/journal.json").exists() {
            if !confirm(
                "La récupération de l’installation reste en attente. Le lancement est bloqué. Réessayer de restaurer la version précédente ?",
            ) {
                return Ok(());
            }
            if let Err(cause) = install::recover(&root) {
                report(&format!("Récupération impossible : {cause}"));
            }
        }
    }
    writeln!(update_log, "Installation cohérente ; relance du compagnon.")?;
    update_log.sync_all()?;
    progress.set("Mise à jour terminée. Redémarrage d’Aetheris…");
    drop(application_lease);
    drop(installation_lease);
    drop(lease);
    drop(update_lease);
    Command::new(root.join(if cfg!(windows) {
        "aetheris-companion.exe"
    } else {
        "aetheris-companion"
    }))
    .spawn()?;
    Ok(())
}
