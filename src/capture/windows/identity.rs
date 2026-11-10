//! Bind capture to the authenticated Windows user and session, never a client PID argument.
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
