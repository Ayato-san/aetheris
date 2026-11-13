//! Read-only environment identification. Errors fail qualification closed.
use super::Environment;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_NO_MORE_FILES, GetLastError, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VS_FIXEDFILEINFO, VerQueryValueW,
    },
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
        Registry::{
            HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RRF_RT_REG_SZ, RRF_SUBKEY_WOW6464KEY,
            RegGetValueW,
        },
        Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW},
    },
};
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn windows_version() -> Option<[u32; 2]> {
    let key = wide("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
    let mut build = [0u16; 64];
    let mut size = std::mem::size_of_val(&build) as u32;
    let mut revision = 0u32;
    let mut revision_size = std::mem::size_of_val(&revision) as u32;
    unsafe {
        if RegGetValueW(
            HKEY_LOCAL_MACHINE,
            key.as_ptr(),
            wide("CurrentBuildNumber").as_ptr(),
            RRF_RT_REG_SZ | RRF_SUBKEY_WOW6464KEY,
            std::ptr::null_mut(),
            build.as_mut_ptr().cast(),
            &mut size,
        ) != 0
        {
            return None;
        }
        if RegGetValueW(
            HKEY_LOCAL_MACHINE,
            key.as_ptr(),
            wide("UBR").as_ptr(),
            RRF_RT_REG_DWORD | RRF_SUBKEY_WOW6464KEY,
            std::ptr::null_mut(),
            (&mut revision as *mut u32).cast(),
            &mut revision_size,
        ) != 0
            || revision_size != 4
        {
            return None;
        }
    }
    let build = String::from_utf16(build.split(|c| *c == 0).next()?)
        .ok()?
        .parse()
        .ok()?;
    Some([build, revision])
}
fn obs_version(pid: u32) -> Option<[u16; 4]> {
    let raw = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if raw.is_null() {
        return None;
    }
    let process = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut path = [0u16; 32768];
    let mut length = path.len() as u32;
    unsafe {
        if QueryFullProcessImageNameW(process.as_raw_handle(), 0, path.as_mut_ptr(), &mut length)
            == 0
        {
            return None;
        }
        let size = GetFileVersionInfoSizeW(path.as_ptr(), std::ptr::null_mut());
        if size == 0 || size > 1024 * 1024 {
            return None;
        }
        let mut data = vec![0u64; (size as usize).div_ceil(8)];
        if GetFileVersionInfoW(path.as_ptr(), 0, size, data.as_mut_ptr().cast()) == 0 {
            return None;
        }
        let mut info = std::ptr::null_mut();
        let mut info_size = 0;
        if VerQueryValueW(
            data.as_ptr().cast(),
            wide("\\").as_ptr(),
            &mut info,
            &mut info_size,
        ) == 0
            || info.is_null()
            || info_size < std::mem::size_of::<VS_FIXEDFILEINFO>() as u32
        {
            return None;
        }
        let info = info.cast::<VS_FIXEDFILEINFO>().read_unaligned();
        if info.dwSignature != 0xfeef04bd {
            return None;
        }
        Some([
            (info.dwFileVersionMS >> 16) as u16,
            info.dwFileVersionMS as u16,
            (info.dwFileVersionLS >> 16) as u16,
            info.dwFileVersionLS as u16,
        ])
    }
}
pub(super) fn observe() -> Option<Environment> {
    let windows = windows_version()?;
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if raw == INVALID_HANDLE_VALUE {
        return None;
    }
    struct Snapshot(windows_sys::Win32::Foundation::HANDLE);
    impl Drop for Snapshot {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    let snapshot = Snapshot(raw);
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut obs = Vec::new();
    let mut available = unsafe { Process32FirstW(snapshot.0, &mut entry) };
    while available != 0 {
        let name = String::from_utf16(entry.szExeFile.split(|c| *c == 0).next()?).ok()?;
        if ["obs64.exe", "obs32.exe", "obs.exe"]
            .iter()
            .any(|exe| name.eq_ignore_ascii_case(exe))
        {
            obs.push(obs_version(entry.th32ProcessID)?);
        }
        available = unsafe { Process32NextW(snapshot.0, &mut entry) };
    }
    if unsafe { GetLastError() } != ERROR_NO_MORE_FILES {
        return None;
    }
    Some(Environment { windows, obs })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_environment_is_identified_without_enabling_a_profile() {
        let windows = windows_version().expect("read-only Windows build and update revision");
        assert!(windows[0] >= 7600);
    }
}
