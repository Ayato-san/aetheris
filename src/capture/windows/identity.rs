//! Authenticate capture clients by Windows user/session; locate games in their session.
use crate::capture::Result;
use std::{
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::PathBuf,
};
use windows_sys::Win32::{
    Security::{GetLengthSid, GetTokenInformation, TOKEN_QUERY, TOKEN_USER, TokenUser},
    System::{
        RemoteDesktop::ProcessIdToSessionId,
        Threading::{
            OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
            QueryFullProcessImageNameW,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::capture) struct Identity {
    pub session: u32,
    sid: Vec<u8>,
}

pub(in crate::capture) struct Process {
    pub pid: u32,
    pub identity: Identity,
    pub image: PathBuf,
    // Keep the process object alive while this authenticated connection exists.
    _handle: OwnedHandle,
}

pub(in crate::capture) fn session(pid: u32) -> Result<u32> {
    let mut session = 0;
    if unsafe { ProcessIdToSessionId(pid, &mut session) } == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(session)
}

impl Process {
    pub fn open(pid: u32) -> Result<Self> {
        let raw = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if raw.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
        let session = session(pid)?;
        let mut path = [0; 32768];
        let mut count = path.len() as u32;
        if unsafe { QueryFullProcessImageNameW(raw, 0, path.as_mut_ptr(), &mut count) } == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let image = PathBuf::from(String::from_utf16(&path[..count as usize])?);
        let mut token = std::ptr::null_mut();
        if unsafe { OpenProcessToken(raw, TOKEN_QUERY, &mut token) } == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let token = unsafe { OwnedHandle::from_raw_handle(token) };
        let mut size = 0;
        unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                std::ptr::null_mut(),
                0,
                &mut size,
            );
        }
        if size == 0 || size > 65536 {
            return Err("Identité Windows invalide.".into());
        }
        let mut buffer = vec![0u64; (size as usize).div_ceil(8)];
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                buffer.as_mut_ptr().cast(),
                size,
                &mut size,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        // Windows returned a TOKEN_USER whose SID is contained in this aligned buffer.
        let sid = unsafe { (*buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid };
        let sid =
            unsafe { std::slice::from_raw_parts(sid.cast::<u8>(), GetLengthSid(sid) as usize) }
                .to_vec();
        Ok(Self {
            pid,
            identity: Identity { session, sid },
            image,
            _handle: handle,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Child, Command, Stdio};

    struct Game {
        child: Child,
        directory: PathBuf,
    }
    impl Game {
        fn spawn() -> Self {
            let mut random = [0; 16];
            getrandom::fill(&mut random).unwrap();
            let directory =
                std::env::temp_dir().join(format!("aetheris-game-test-{}", hex::encode(random)));
            std::fs::create_dir(&directory).unwrap();
            let executable = directory.join("AION2.exe");
            std::fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
            let child = Command::new(executable)
                .args([
                    "--exact",
                    "capture::windows::identity::tests::game_process_fixture",
                    "--ignored",
                ])
                .env("AETHERIS_GAME_PROCESS_FIXTURE", "1")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            Self { child, directory }
        }
    }
    impl Drop for Game {
        fn drop(&mut self) {
            // Terminate only the child created by this test.
            let _ = self.child.kill();
            let _ = self.child.wait();
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }

    #[test]
    #[ignore = "child-process fixture invoked only by game discovery tests"]
    fn game_process_fixture() {
        assert!(std::env::var_os("AETHERIS_GAME_PROCESS_FIXTURE").is_some());
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
    }

    #[test]
    fn game_discovery_allows_another_account_but_isolates_windows_sessions() {
        let game = Game::spawn();
        let mut client = Process::open(std::process::id()).unwrap().identity;
        // Model a standard-user client while the real fixture process belongs
        // to another account. Session and process enumeration remain native.
        client.sid.push(0xff);
        let found = super::super::game_pid(&client).unwrap().unwrap();
        assert_eq!(found.pid, game.child.id());
        assert_ne!(found.identity, client);
        let original_session = client.session;
        client.session = original_session.wrapping_add(1);
        assert!(super::super::game_pid(&client).unwrap().is_none());
        client.session = original_session;
        let _second_game = Game::spawn();
        let error = super::super::game_pid(&client).err().unwrap();
        assert!(error.to_string().contains("Plusieurs processus AION 2"));
    }
}
