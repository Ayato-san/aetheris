//! Installer-owned inbound TCP permission for the capture service executable.
use crate::capture::Result;
use sha2::{Digest, Sha256};
use std::{
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW;

pub(in crate::capture) fn configure() -> Result<()> {
    configure_rule("configure")
}
pub(in crate::capture) fn remove() -> Result<()> {
    configure_rule("remove")
}

fn configure_rule(action: &str) -> Result<()> {
    run(action).map_err(|error| {
        format!("Configuration du pare-feu pour le service de capture impossible : {error}.").into()
    })
}

fn run(action: &str) -> Result<()> {
    let executable = std::env::current_exe()?;
    // Different executable locations must not overwrite each other's permission.
    // Keep the name stable across launches and updates at the same location.
    let rule_name = format!(
        "Aetheris.Capture.TCP.{}",
        hex::encode(Sha256::digest(executable.as_os_str().as_encoded_bytes()))
    );
    let mut directory = [0u16; 32768];
    // Windows writes at most directory.len() UTF-16 code units into this buffer.
    let count = unsafe { GetSystemDirectoryW(directory.as_mut_ptr(), directory.len() as u32) };
    if count == 0 || count as usize >= directory.len() {
        return Err("Dossier système Windows introuvable".into());
    }
    let powershell = PathBuf::from(String::from_utf16(&directory[..count as usize])?)
        .join("WindowsPowerShell/v1.0/powershell.exe");
    let mut child = Command::new(powershell)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            include_str!("firewall.ps1"),
        ])
        // Paths are data, never interpolated into PowerShell source code.
        .env("AETHERIS_CAPTURE_EXECUTABLE", executable)
        .env("AETHERIS_CAPTURE_FIREWALL_RULE", rule_name)
        .env("AETHERIS_CAPTURE_FIREWALL_ACTION", action)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    let started = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if started.elapsed() >= Duration::from_secs(20) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Windows n’a pas répondu dans le délai de 20 secondes".into());
        }
        thread::sleep(Duration::from_millis(25));
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "Windows a refusé la configuration : {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(())
}
