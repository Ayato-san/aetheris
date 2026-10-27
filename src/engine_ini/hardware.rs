//! Best-effort hardware discovery. A GPU vendor alone proves no game compatibility.
use super::{settings::Upscaling, storage};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpu {
    Unknown,
    Nvidia,
    Amd,
    Intel,
}
impl Gpu {
    pub const ALL: [Self; 4] = [Self::Unknown, Self::Nvidia, Self::Amd, Self::Intel];
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "Non identifié",
            Self::Nvidia => "NVIDIA",
            Self::Amd => "AMD",
            Self::Intel => "Intel",
        }
    }
    fn code(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Nvidia => "nvidia",
            Self::Amd => "amd",
            Self::Intel => "intel",
        }
    }
    pub fn recommended(self, dlss_confirmed: bool, fsr_confirmed: bool) -> Upscaling {
        if self == Self::Nvidia && dlss_confirmed {
            Upscaling::Dlss
        } else if self != Self::Unknown && fsr_confirmed {
            Upscaling::Fsr
        } else {
            Upscaling::Off
        }
    }
    pub fn load(directory: &Path) -> Self {
        if let Ok(code) = fs::read_to_string(directory.join("gpu-selection.txt"))
            && let Some(gpu) = Self::ALL.into_iter().find(|g| g.code() == code.trim())
        {
            return gpu;
        }
        detect_gpu()
    }
    pub fn save(self, directory: &Path) -> std::io::Result<()> {
        fs::create_dir_all(directory)?;
        storage::atomic_replace(
            &directory.join("gpu-selection.txt"),
            self.code().as_bytes(),
            None,
        )
    }
}
pub fn preferences_directory() -> PathBuf {
    if let Some(root) = std::env::var_os("LOCALAPPDATA") {
        PathBuf::from(root).join("Aetheris")
    } else if let Some(root) = std::env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(root).join("aetheris")
    } else {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(".config/aetheris")
    }
}
#[cfg(not(windows))]
fn detect_gpu() -> Gpu {
    let mut vendors = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            if let Ok(vendor) = fs::read_to_string(entry.path().join("device/vendor")) {
                let gpu = match vendor.trim() {
                    "0x10de" => Gpu::Nvidia,
                    "0x1002" => Gpu::Amd,
                    "0x8086" => Gpu::Intel,
                    _ => Gpu::Unknown,
                };
                if gpu != Gpu::Unknown && !vendors.contains(&gpu) {
                    vendors.push(gpu);
                }
            }
        }
    }
    if vendors.len() == 1 {
        vendors[0]
    } else {
        Gpu::Unknown
    }
}
#[cfg(windows)]
fn detect_gpu() -> Gpu {
    use std::os::windows::process::CommandExt;
    let result = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name",
        ])
        .creation_flags(0x08000000)
        .output();
    let Ok(output) = result else {
        return Gpu::Unknown;
    };
    let name = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
    let vendors: Vec<_> = [
        (Gpu::Nvidia, "nvidia"),
        (Gpu::Amd, "amd"),
        (Gpu::Intel, "intel"),
    ]
    .into_iter()
    .filter(|(_, needle)| name.contains(needle))
    .collect();
    if vendors.len() == 1 {
        vendors[0].0
    } else {
        Gpu::Unknown
    }
}

#[cfg(not(windows))]
pub fn native_resolution() -> Option<(u64, u64)> {
    let output = std::process::Command::new("xrandr")
        .arg("--query")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut primary = false;
    let mut first = None;
    for line in text.lines() {
        if !line.starts_with(' ') {
            primary = line.contains(" connected primary ");
        } else if line.contains('+') {
            let (w, h) = line.split_whitespace().next()?.split_once('x')?;
            if let (Ok(w), Ok(h)) = (w.parse(), h.parse()) {
                if primary {
                    return Some((w, h));
                }
                first.get_or_insert((w, h));
            }
        }
    }
    first
}

#[cfg(windows)]
mod windows_display;

#[cfg(any(windows, test))]
fn oriented_resolution(width: u32, height: u32, rotation: u32) -> Option<(u64, u64)> {
    if width == 0 || height == 0 {
        return None;
    }
    match rotation {
        1 | 3 => Some((u64::from(width), u64::from(height))),
        2 | 4 => Some((u64::from(height), u64::from(width))),
        _ => None,
    }
}

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;
    type Handle = *mut c_void;
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnumWindows(
            callback: unsafe extern "system" fn(Handle, isize) -> i32,
            data: isize,
        ) -> i32;
        fn IsWindowVisible(window: Handle) -> i32;
        fn GetWindowThreadProcessId(window: Handle, pid: *mut u32) -> u32;
        fn GetWindowRect(window: Handle, rect: *mut Rect) -> i32;
        fn MonitorFromRect(rect: *const Rect, flags: u32) -> Handle;
        fn MonitorFromPoint(point: i64, flags: u32) -> Handle;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> Handle;
        fn QueryFullProcessImageNameW(
            process: Handle,
            flags: u32,
            name: *mut u16,
            size: *mut u32,
        ) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
    }
    unsafe extern "system" fn find_game(window: Handle, data: isize) -> i32 {
        // Callback context points to the stack-owned monitor handle during EnumWindows.
        unsafe {
            if IsWindowVisible(window) == 0 {
                return 1;
            }
            let mut pid = 0;
            GetWindowThreadProcessId(window, &mut pid);
            let process = OpenProcess(0x1000, 0, pid);
            if process.is_null() {
                return 1;
            }
            let mut name = [0u16; 32768];
            let mut length = name.len() as u32;
            let ok = QueryFullProcessImageNameW(process, 0, name.as_mut_ptr(), &mut length);
            CloseHandle(process);
            if ok == 0 {
                return 1;
            }
            let path = String::from_utf16_lossy(&name[..length as usize]);
            let name = path.rsplit('\\').next().unwrap_or("").to_ascii_lowercase();
            if !matches!(name.as_str(), "aion2.exe" | "aion2-win64-shipping.exe") {
                return 1;
            }
            let mut rect = Rect::default();
            if GetWindowRect(window, &mut rect) != 0
                && rect.right > rect.left
                && rect.bottom > rect.top
            {
                // MonitorFromRect chooses the monitor with the greatest intersection.
                *(data as *mut Handle) = MonitorFromRect(&rect, 1);
                return 0;
            }
            1
        }
    }
    pub fn native_resolution() -> Option<(u64, u64)> {
        unsafe {
            let mut monitor: Handle = std::ptr::null_mut();
            EnumWindows(find_game, &mut monitor as *mut Handle as isize);
            if monitor.is_null() {
                monitor = MonitorFromPoint(0, 1);
            }
            super::windows_display::preferred_resolution(monitor)
        }
    }
}
#[cfg(windows)]
pub use windows::native_resolution;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_dimensions_follow_monitor_rotation_without_using_desktop_dimensions() {
        assert_eq!(oriented_resolution(3840, 2160, 1), Some((3840, 2160)));
        assert_eq!(oriented_resolution(3840, 2160, 2), Some((2160, 3840)));
        assert_eq!(oriented_resolution(3840, 2160, 3), Some((3840, 2160)));
        assert_eq!(oriented_resolution(3840, 2160, 4), Some((2160, 3840)));
        assert_eq!(oriented_resolution(0, 2160, 1), None);
        assert_eq!(oriented_resolution(3840, 2160, 0), None);
    }
    #[test]
    fn vendor_detection_is_insufficient_to_recommend_upscaling() {
        for gpu in Gpu::ALL {
            assert_eq!(gpu.recommended(false, false), Upscaling::Off);
        }
        assert_eq!(Gpu::Unknown.recommended(true, true), Upscaling::Off);
        assert_eq!(Gpu::Nvidia.recommended(true, false), Upscaling::Dlss);
        assert_eq!(Gpu::Amd.recommended(false, true), Upscaling::Fsr);
    }
    #[test]
    fn corrected_gpu_is_restored_on_next_launch() {
        let root = std::env::temp_dir().join(format!("aetheris-gpu-{}", std::process::id()));
        Gpu::Amd.save(&root).unwrap();
        assert_eq!(Gpu::load(&root), Gpu::Amd);
        Gpu::Nvidia.save(&root).unwrap();
        assert_eq!(Gpu::load(&root), Gpu::Nvidia);
        fs::remove_dir_all(root).unwrap();
    }
}
