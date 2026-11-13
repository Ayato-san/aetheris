//! Aion 2 process status. Only a complete process snapshot permits editing.
//! Windows API: https://learn.microsoft.com/windows/win32/toolhelp/process-walking
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Running,
    Stopped,
    Unknown,
}

impl GameStatus {
    pub fn is_locked(self) -> bool {
        self != Self::Stopped
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Running => "Aion 2 · En cours",
            Self::Stopped => "Aion 2 · Arrêté",
            Self::Unknown => "Aion 2 · Statut inconnu",
        }
    }

    pub fn lock_reason(self) -> Option<&'static str> {
        match self {
            Self::Stopped => None,
            Self::Running => Some("Fermez Aion 2 pour modifier Engine.ini."),
            Self::Unknown => {
                Some("Statut d’Aion 2 indisponible : modification d’Engine.ini verrouillée.")
            }
        }
    }
}

#[cfg(any(windows, test))]
pub(crate) fn is_game_executable(name: &str) -> bool {
    name.eq_ignore_ascii_case("AION2.exe") || name.eq_ignore_ascii_case("Aion2-Win64-Shipping.exe")
}

#[cfg(not(windows))]
pub fn detect() -> GameStatus {
    // Linux/WSL cannot establish the state of the host's Windows processes.
    GameStatus::Unknown
}

#[cfg(windows)]
pub fn detect() -> GameStatus {
    windows::detect()
}

/// Identify a foreground owner from a process snapshot without opening the
/// game process. Elevated or protected clients may deny image-path queries.
#[cfg(windows)]
pub(crate) fn identify(pid: u32) -> GameStatus {
    windows::detect_pid(Some(pid))
}

#[cfg(windows)]
mod windows {
    use super::{GameStatus, is_game_executable};
    use std::ffi::c_void;

    #[repr(C)]
    struct ProcessEntry {
        size: u32,
        usage: u32,
        process_id: u32,
        default_heap_id: usize,
        module_id: u32,
        threads: u32,
        parent_process_id: u32,
        priority: i32,
        flags: u32,
        exe_file: [u16; 260],
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateToolhelp32Snapshot(flags: u32, process_id: u32) -> *mut c_void;
        fn Process32FirstW(snapshot: *mut c_void, entry: *mut ProcessEntry) -> i32;
        fn Process32NextW(snapshot: *mut c_void, entry: *mut ProcessEntry) -> i32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }

    struct Snapshot(*mut c_void);
    impl Drop for Snapshot {
        fn drop(&mut self) {
            // This valid snapshot handle is owned and closed exactly once.
            unsafe { CloseHandle(self.0) };
        }
    }

    pub(super) fn detect() -> GameStatus {
        detect_pid(None)
    }

    pub(super) fn detect_pid(pid: Option<u32>) -> GameStatus {
        // TH32CS_SNAPPROCESS enumerates processes without opening them or
        // depending on visible windows (including elevated/minimized clients).
        let handle = unsafe { CreateToolhelp32Snapshot(0x00000002, 0) };
        if handle as isize == -1 {
            return GameStatus::Unknown;
        }
        let snapshot = Snapshot(handle);
        // All fields are integer values; zero initialization is valid.
        let mut entry: ProcessEntry = unsafe { std::mem::zeroed() };
        entry.size = std::mem::size_of::<ProcessEntry>() as u32;
        // The handle is valid and the correctly sized entry lives for the call.
        let mut available = unsafe { Process32FirstW(snapshot.0, &mut entry) };
        loop {
            if available == 0 {
                return if std::io::Error::last_os_error().raw_os_error() == Some(18) {
                    GameStatus::Stopped // ERROR_NO_MORE_FILES: complete enumeration.
                } else {
                    GameStatus::Unknown
                };
            }
            let length = entry.exe_file.iter().position(|&c| c == 0).unwrap_or(260);
            if pid.is_none_or(|pid| entry.process_id == pid)
                && is_game_executable(&String::from_utf16_lossy(&entry.exe_file[..length]))
            {
                return GameStatus::Running;
            }
            // The snapshot and entry remain valid throughout enumeration.
            available = unsafe { Process32NextW(snapshot.0, &mut entry) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_client_names_match_case_insensitively() {
        for name in ["AION2.exe", "aion2-WIN64-shipping.EXE"] {
            assert!(is_game_executable(name));
        }
        for name in ["AION2Launcher.exe", "AION2.exe.bak", "Other.exe"] {
            assert!(!is_game_executable(name));
        }
    }

    #[cfg(windows)]
    #[test]
    fn process_identification_matches_pid_instead_of_an_unrelated_game() {
        assert_eq!(identify(std::process::id()), GameStatus::Stopped);
        assert_eq!(identify(0), GameStatus::Stopped);
    }
    #[test]
    fn only_confirmed_stopped_status_unlocks_editing() {
        for status in [
            GameStatus::Running,
            GameStatus::Stopped,
            GameStatus::Unknown,
        ] {
            assert_eq!(status.is_locked(), status.lock_reason().is_some());
            assert_eq!(status.is_locked(), status != GameStatus::Stopped);
        }
    }
}
