//! Inherited, authenticated loopback IPC. No GPUI types cross this boundary.
use super::{Result, error};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{Mutex, OnceLock},
    time::Duration,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Hello(String),
    Progress(String),
    WindowReady,
    Interaction,
    NormalShutdown,
    Closing(std::time::SystemTime),
    Update,
    Channel(super::policy::Channel),
    CloseBlocked,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Activate,
    Channel(super::policy::Channel),
    CloseForUpdate { unattended: bool },
    Status { message: String, available: bool },
}
static STREAM: OnceLock<Mutex<TcpStream>> = OnceLock::new();
static COMMANDS: Mutex<Vec<Command>> = Mutex::new(Vec::new());

pub fn attach() -> Result<bool> {
    let Ok(address) = std::env::var("AETHERIS_COMPANION_ADDRESS") else {
        return Ok(false);
    };
    let token = std::env::var("AETHERIS_COMPANION_TOKEN")?;
    let stream = TcpStream::connect(address)?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let reader = stream.try_clone()?;
    STREAM
        .set(Mutex::new(stream))
        .map_err(|_| error("Compagnon déjà connecté."))?;
    send(Event::Hello(token));
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            let Ok(line) = line else {
                break;
            };
            if let Ok(command) = serde_json::from_str(&line) {
                COMMANDS.lock().unwrap().push(command);
            }
        }
    });
    Ok(true)
}
pub fn send(event: Event) {
    if let Some(stream) = STREAM.get()
        && let Ok(mut stream) = stream.lock()
    {
        let _ = serde_json::to_writer(&mut *stream, &event);
        let _ = stream.write_all(b"\n");
    }
}
pub fn drain() -> Vec<Command> {
    if let Ok(data) = super::data_directory() {
        let activate = data.join("activate");
        if activate.exists() && std::fs::remove_file(activate).is_ok() {
            COMMANDS.lock().unwrap().push(Command::Activate);
        }
    }
    std::mem::take(&mut *COMMANDS.lock().unwrap())
}

pub fn delegate() -> Result<()> {
    let companion = std::env::current_exe()?.with_file_name(if cfg!(windows) {
        "aetheris-companion.exe"
    } else {
        "aetheris-companion"
    });
    std::process::Command::new(companion).spawn()?;
    Ok(())
}

static CLOSE_AUTHORIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn authorize_close() {
    send(Event::Closing(std::time::SystemTime::now()));
    CLOSE_AUTHORIZED.store(true, std::sync::atomic::Ordering::Relaxed);
}
pub fn finish_shutdown() {
    if CLOSE_AUTHORIZED.load(std::sync::atomic::Ordering::Relaxed) {
        send(Event::NormalShutdown);
    }
}
