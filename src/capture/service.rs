//! Privileged acquisition over authenticated, local-only IPC. Personal storage
//! and protocol interpretation stay in the unprivileged application.
use super::{
    CaptureBatch, CaptureSource, Interface, Result,
    protocol::{self, Request, Response},
    windows,
};
use std::{
    collections::HashSet,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::System::Services::*;

const NAME: &str = "AetherisCapture";
const ADDRESS: &str = "127.0.0.1:38947";
const TIMEOUT: Duration = Duration::from_secs(3);
static STOP: AtomicBool = AtomicBool::new(false);

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

pub fn interfaces() -> Result<Vec<Interface>> {
    windows::interfaces()
}

pub struct Client {
    stream: TcpStream,
}

pub fn connect(interface: Option<Interface>) -> Result<Client> {
    if !crate::companion::managed::current_is_managed() {
        return Err("La capture nécessite le service Aetheris. Installez ou réparez Aetheris avec l’installeur Windows ; l’application peut ensuite rester en utilisateur normal.".into());
    }
    let expected_pid = service_pid()?;
    let address: SocketAddr = ADDRESS.parse()?;
    let mut stream = TcpStream::connect_timeout(&address, TIMEOUT).map_err(|e| {
        format!("Service de capture indisponible : {e}. Réparez l’installation Aetheris.")
    })?;
    configure_stream(&stream)?;
    // The server's identity comes from the owning PID in the Windows TCP table
    // and the SCM, not from a secret or PID claimed by a loopback peer.
    let owner = windows::tcp_owner(stream.peer_addr()?, stream.local_addr()?)?;
    if owner == 0 || owner != expected_pid {
        return Err("Le serveur local n’est pas le service de capture Aetheris.".into());
    }
    protocol::write(
        &mut stream,
        &Request::Hello {
            version: protocol::VERSION,
            interface,
        },
    )?;
    match protocol::read(&mut stream, 4096)? {
        Response::Ready { version } if version == protocol::VERSION => Ok(Client { stream }),
        Response::Error { message } => Err(message.into()),
        _ => Err(
            "Version du service de capture incompatible. Réparez l’installation Aetheris.".into(),
        ),
    }
}

impl CaptureSource for Client {
    fn poll(&mut self) -> Result<CaptureBatch> {
        (|| {
            protocol::write(&mut self.stream, &Request::Poll)?;
            protocol::read_batch(&mut self.stream)
        })()
        .map_err(|error| format!("Échange avec le service de capture interrompu : {error}").into())
    }
}

fn configure_stream(stream: &TcpStream) -> Result<()> {
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    Ok(())
}

/// Absolute frame deadline and frequent stop checks, including for peers that
/// trickle bytes. Socket timeouts alone restart on every successful short read.
struct FrameStream<'a> {
    stream: &'a mut TcpStream,
    deadline: Instant,
}
impl<'a> FrameStream<'a> {
    fn new(stream: &'a mut TcpStream) -> Self {
        Self {
            stream,
            deadline: Instant::now() + TIMEOUT,
        }
    }
    fn timeout(&self) -> io::Result<Duration> {
        if STOP.load(Ordering::Acquire) {
            return Err(io::ErrorKind::ConnectionAborted.into());
        }
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|left| !left.is_zero())
            .map(|left| left.min(Duration::from_millis(100)))
            .ok_or_else(|| io::ErrorKind::TimedOut.into())
    }
}
impl Read for FrameStream<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        loop {
            self.stream.set_read_timeout(Some(self.timeout()?))?;
            match self.stream.read(bytes) {
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
                {
                    continue;
                }
                result => return result,
            }
        }
    }
}
impl Write for FrameStream<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        loop {
            self.stream.set_write_timeout(Some(self.timeout()?))?;
            match self.stream.write(bytes) {
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
                {
                    continue;
                }
                result => return result,
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

struct ServiceHandle(SC_HANDLE);
impl Drop for ServiceHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                CloseServiceHandle(self.0);
            }
        }
    }
}

fn service_status() -> Result<Option<SERVICE_STATUS_PROCESS>> {
    let scm = ServiceHandle(unsafe {
        OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT)
    });
    if scm.0.is_null() {
        return Err(std::io::Error::last_os_error().into());
    }
    let service =
        ServiceHandle(unsafe { OpenServiceW(scm.0, wide(NAME).as_ptr(), SERVICE_QUERY_STATUS) });
    if service.0.is_null() {
        let error = std::io::Error::last_os_error();
        return if error.raw_os_error() == Some(1060) {
            Ok(None)
        } else {
            Err(error.into())
        };
    }
    let mut status: SERVICE_STATUS_PROCESS = unsafe { std::mem::zeroed() };
    let mut size = 0;
    if unsafe {
        QueryServiceStatusEx(
            service.0,
            SC_STATUS_PROCESS_INFO,
            (&mut status as *mut SERVICE_STATUS_PROCESS).cast(),
            std::mem::size_of_val(&status) as u32,
            &mut size,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(Some(status))
}

fn service_pid() -> Result<u32> {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let status = service_status()?.ok_or("Service de capture non installé. La mise à jour précédente n’a pas ajouté le service ; réparez Aetheris avec son installeur.")?;
        match status.dwCurrentState {
            SERVICE_RUNNING => return Ok(status.dwProcessId),
            SERVICE_START_PENDING if Instant::now() < deadline => thread::sleep(Duration::from_millis(100)),
            SERVICE_STOPPED => {
                let detail = crate::companion::managed::registered_root()
                    .and_then(|root| std::fs::read_to_string(root.join("service/capture-error")).ok())
                    .filter(|text| !text.trim().is_empty())
                    .unwrap_or_else(|| format!("Code Windows : {}.", status.dwWin32ExitCode));
                return Err(format!("Le service de capture est arrêté. {detail}").into());
            }
            _ => return Err("Le service de capture n’a pas terminé son démarrage. Réessayez dans quelques instants.".into()),
        }
    }
}

fn registered_root() -> Result<PathBuf> {
    let root = crate::companion::managed::registered_root()
        .ok_or("Installation Aetheris non enregistrée.")?;
    crate::companion::managed_security::validate(&root, true)?;
    crate::companion::manifest::verify(&root, &crate::companion::manifest::public_key()?)?;
    Ok(root)
}

/// Called by the elevated installer, after validating the protected, signed installation.
pub fn configure() -> Result<()> {
    let root = registered_root()?;
    let executable = format!(
        "\"{}\" --capture-service",
        root.join("aetheris-update-helper.exe").display()
    );
    let command = crate::companion::managed::service_command;
    let scm = ServiceHandle(unsafe {
        OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT)
    });
    if scm.0.is_null() {
        return Err(std::io::Error::last_os_error().into());
    }
    let service =
        ServiceHandle(unsafe { OpenServiceW(scm.0, wide(NAME).as_ptr(), SERVICE_QUERY_STATUS) });
    let exists = !service.0.is_null();
    if !exists && std::io::Error::last_os_error().raw_os_error() != Some(1060) {
        return Err(std::io::Error::last_os_error().into());
    }
    command(&[
        if exists { "config" } else { "create" },
        NAME,
        "binPath=",
        &executable,
        "start=",
        "auto",
        "obj=",
        "LocalSystem",
        "DisplayName=",
        "Aetheris Capture",
    ])?;
    // Users may query service status but cannot replace or stop privileged code.
    command(&["sdset", NAME, "D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;LC;;;AU)"])?;
    command(&[
        "failure",
        NAME,
        "reset=",
        "86400",
        "actions=",
        "restart/5000/restart/30000/restart/60000",
    ])?;
    command(&["failureflag", NAME, "1"])?;
    windows::firewall::configure(&root.join("aetheris-update-helper.exe"))
}

pub fn stop() -> Result<()> {
    crate::companion::managed::stop_named_service(NAME)
}
pub fn start() -> Result<()> {
    let scm = ServiceHandle(unsafe {
        OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT)
    });
    if scm.0.is_null() {
        return Err(std::io::Error::last_os_error().into());
    }
    let service = ServiceHandle(unsafe {
        OpenServiceW(
            scm.0,
            wide(NAME).as_ptr(),
            SERVICE_START | SERVICE_QUERY_STATUS,
        )
    });
    if service.0.is_null() {
        return Err(std::io::Error::last_os_error().into());
    }
    if unsafe { StartServiceW(service.0, 0, std::ptr::null()) } == 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(1056) {
            return Err(error.into());
        }
    }
    service_pid().map(|_| ())
}

pub fn uninstall() -> Result<()> {
    stop()?;
    let scm = ServiceHandle(unsafe {
        OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT)
    });
    if scm.0.is_null() {
        return Err(std::io::Error::last_os_error().into());
    }
    let service =
        ServiceHandle(unsafe { OpenServiceW(scm.0, wide(NAME).as_ptr(), SERVICE_QUERY_STATUS) });
    if !service.0.is_null() {
        crate::companion::managed::service_command(&["delete", NAME])?;
    } else if std::io::Error::last_os_error().raw_os_error() != Some(1060) {
        return Err(std::io::Error::last_os_error().into());
    }
    let root =
        crate::companion::managed::registered_root().ok_or("Installation Aetheris absente.")?;
    windows::firewall::remove(&root.join("aetheris-update-helper.exe"))
}

/// Release the capture executable's Windows image lock during signed updates.
/// Both successful replacements and recovered failures attempt a restart.
pub(crate) fn with_paused<T>(operation: impl FnOnce() -> Result<T>) -> Result<T> {
    if service_status()?.is_none() {
        return operation();
    }
    stop()?;
    let result = operation();
    let restart = start();
    match result {
        Ok(value) => {
            restart?;
            Ok(value)
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn with_reconfigured<T>(operation: impl FnOnce() -> Result<T>) -> Result<T> {
    struct Host;
    impl super::lifecycle::Service for Host {
        fn exists(&mut self) -> Result<bool> {
            Ok(service_status()?.is_some())
        }
        fn stop(&mut self) -> Result<()> {
            stop()
        }
        fn configure(&mut self) -> Result<()> {
            configure()
        }
        fn start(&mut self) -> Result<()> {
            start()
        }
    }
    super::lifecycle::replace(&mut Host, operation)
}

fn record_failure(message: &str) {
    if let Some(root) = crate::companion::managed::registered_root()
        && crate::companion::managed_security::validate(&root, true).is_ok()
    {
        let _ = crate::companion::durable_write(
            &root.join("service/capture-error"),
            message.as_bytes(),
        );
    }
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

unsafe extern "system" fn handler(
    code: u32,
    _: u32,
    _: *mut std::ffi::c_void,
    _: *mut std::ffi::c_void,
) -> u32 {
    match code {
        SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
            STOP.store(true, Ordering::Release);
            0
        }
        SERVICE_CONTROL_INTERROGATE => 0,
        _ => 120,
    }
}

unsafe extern "system" fn service_main(_: u32, _: *mut *mut u16) {
    STOP.store(false, Ordering::Release);
    let handle = unsafe {
        RegisterServiceCtrlHandlerExW(wide(NAME).as_ptr(), Some(handler), std::ptr::null_mut())
    };
    if handle.is_null() {
        return;
    }
    let mut status = SERVICE_STATUS {
        dwServiceType: SERVICE_WIN32_OWN_PROCESS,
        dwCurrentState: SERVICE_START_PENDING,
        dwControlsAccepted: 0,
        dwWin32ExitCode: 0,
        dwServiceSpecificExitCode: 0,
        dwCheckPoint: 1,
        dwWaitHint: 15000,
    };
    unsafe {
        SetServiceStatus(handle, &status);
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<()> {
        let root = registered_root()?;
        if std::fs::canonicalize(std::env::current_exe()?)?
            != std::fs::canonicalize(root.join("aetheris-update-helper.exe"))?
        {
            return Err("Le service de capture doit utiliser le helper installé.".into());
        }
        windows::check_access()?;
        let listener = TcpListener::bind(ADDRESS)?;
        listener.set_nonblocking(true)?;
        record_failure("");
        status.dwCurrentState = SERVICE_RUNNING;
        status.dwControlsAccepted = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN;
        status.dwCheckPoint = 0;
        status.dwWaitHint = 0;
        unsafe {
            SetServiceStatus(handle, &status);
        }
        serve(listener, &root)
    }));
    let failure = match result {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(error.to_string()),
        Err(_) => Some("Le service de capture a rencontré une erreur interne.".into()),
    };
    if let Some(error) = &failure {
        record_failure(error);
    }
    status.dwCurrentState = SERVICE_STOPPED;
    status.dwControlsAccepted = 0;
    status.dwWin32ExitCode = if failure.is_none() { 0 } else { 1 };
    unsafe {
        SetServiceStatus(handle, &status);
    }
}

fn serve(listener: TcpListener, root: &Path) -> Result<()> {
    let sessions = Arc::new(Mutex::new(HashSet::new()));
    let mut workers: Vec<thread::JoinHandle<()>> = Vec::new();
    let mut failure = None;
    while !STOP.load(Ordering::Acquire) {
        workers.retain(|worker| !worker.is_finished());
        match listener.accept() {
            Ok((mut stream, _)) if workers.len() < 8 => {
                let root = root.to_owned();
                let sessions = sessions.clone();
                workers.push(thread::spawn(move || {
                    if let Err(error) = serve_client(&mut stream, &root, sessions) {
                        let _ = protocol::write(
                            &mut FrameStream::new(&mut stream),
                            &Response::Error {
                                message: error.to_string(),
                            },
                        );
                    }
                }));
            }
            Ok(_) => {} // Bounded connection count, including uncompleted handshakes.
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(25))
            }
            Err(error) => {
                failure = Some(error);
                break;
            }
        }
    }
    STOP.store(true, Ordering::Release);
    for worker in workers {
        let _ = worker.join();
    }
    match failure {
        Some(error) => Err(error.into()),
        None => Ok(()),
    }
}

struct SessionLease {
    identity: windows::identity::Identity,
    sessions: Arc<Mutex<HashSet<windows::identity::Identity>>>,
}
impl Drop for SessionLease {
    fn drop(&mut self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(&self.identity);
        }
    }
}

fn serve_client(
    stream: &mut TcpStream,
    root: &Path,
    sessions: Arc<Mutex<HashSet<windows::identity::Identity>>>,
) -> Result<()> {
    configure_stream(stream)?;
    let pid = windows::tcp_owner(stream.peer_addr()?, stream.local_addr()?)?;
    let client = windows::identity::Process::open(pid)?;
    if client.identity.session == 0
        || std::fs::canonicalize(&client.image)?
            != std::fs::canonicalize(root.join("aetheris.exe"))?
    {
        return Err("Client du service de capture non autorisé.".into());
    }
    let Request::Hello { version, interface } =
        protocol::read(&mut FrameStream::new(stream), 4096)?
    else {
        return Err("Initialisation du service de capture attendue.".into());
    };
    if version != protocol::VERSION {
        return Err("Version du protocole de capture incompatible.".into());
    }
    let interface = match interface {
        Some(requested) => Some(
            windows::interfaces()?
                .into_iter()
                .find(|i| i.address == requested.address && i.index == requested.index)
                .ok_or("Interface de capture indisponible.")?,
        ),
        None => None,
    };
    if !sessions
        .lock()
        .map_err(|_| "État du service indisponible.")?
        .insert(client.identity.clone())
    {
        return Err(
            "Une capture Aetheris est déjà active pour cet utilisateur et cette session Windows."
                .into(),
        );
    }
    let _session = SessionLease {
        identity: client.identity.clone(),
        sessions,
    };
    let mut source = windows::NetworkSource::new(interface, client.identity.clone())?;
    protocol::write(
        &mut FrameStream::new(stream),
        &Response::Ready {
            version: protocol::VERSION,
        },
    )?;
    while !STOP.load(Ordering::Acquire) {
        let Request::Poll = protocol::read(&mut FrameStream::new(stream), 4096)? else {
            return Err("Commande de capture invalide.".into());
        };
        let batch = source.poll().unwrap_or_else(|error| CaptureBatch {
            packets: Vec::new(),
            status: "Erreur de capture".into(),
            finished: true,
            error: Some(error.to_string()),
        });
        protocol::write_batch(&mut FrameStream::new(stream), &batch)?;
        if batch.finished {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let mut random = [0; 16];
            getrandom::fill(&mut random).unwrap();
            let path =
                std::env::temp_dir().join(format!("aetheris-capture-test-{}", hex::encode(random)));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn loopback_identity_is_resolved_by_windows_without_administrator_rights() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (server, _) = listener.accept().unwrap();
        let pid =
            windows::tcp_owner(server.peer_addr().unwrap(), server.local_addr().unwrap()).unwrap();
        assert_eq!(pid, std::process::id());
        let peer = windows::identity::Process::open(pid).unwrap();
        assert_eq!(
            std::fs::canonicalize(peer.image).unwrap(),
            std::fs::canonicalize(std::env::current_exe().unwrap()).unwrap()
        );
        assert_eq!(
            windows::tcp_owner(client.peer_addr().unwrap(), client.local_addr().unwrap()).unwrap(),
            pid
        );
    }

    #[test]
    fn foreign_executable_cannot_initialize_capture_or_claim_a_session() {
        let directory = Directory::new();
        std::fs::write(directory.0.join("aetheris.exe"), b"different application").unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let _client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (mut server, _) = listener.accept().unwrap();
        let sessions = Arc::new(Mutex::new(HashSet::new()));
        let error = serve_client(&mut server, &directory.0, sessions.clone()).unwrap_err();
        assert!(error.to_string().contains("non autorisé"));
        assert!(sessions.lock().unwrap().is_empty());
    }

    #[test]
    fn incomplete_request_cannot_hold_a_service_worker_indefinitely() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (mut server, _) = listener.accept().unwrap();
        client.write_all(&[1]).unwrap();
        let mut frame = FrameStream {
            stream: &mut server,
            deadline: Instant::now() + Duration::from_millis(30),
        };
        assert!(protocol::read::<Request>(&mut frame, 4096).is_err());
    }
}
