//! Read-only CCD queries: identify an active monitor and ask for its preferred mode.
//! https://learn.microsoft.com/windows/win32/api/wingdi/ns-wingdi-displayconfig_target_preferred_mode
use std::{ffi::c_void, mem::size_of};

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Luid {
    low: u32,
    high: i32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Rational {
    numerator: u32,
    denominator: u32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Source {
    adapter: Luid,
    id: u32,
    mode: u32,
    flags: u32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Target {
    adapter: Luid,
    id: u32,
    mode: u32,
    technology: i32,
    rotation: u32,
    scaling: u32,
    refresh: Rational,
    scanline: u32,
    available: i32,
    flags: u32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct DisplayPath {
    source: Source,
    target: Target,
    flags: u32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Region {
    width: u32,
    height: u32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Signal {
    pixel_rate: u64,
    horizontal: Rational,
    vertical: Rational,
    active: Region,
    total: Region,
    video_standard: u32,
    scanline: u32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Mode {
    kind: u32,
    id: u32,
    adapter: Luid,
    data: Signal,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Header {
    kind: u32,
    size: u32,
    adapter: Luid,
    id: u32,
}
#[repr(C)]
#[derive(Default)]
struct SourceName {
    header: Header,
    name: [u16; 32],
}
#[repr(C)]
#[derive(Default)]
struct PreferredMode {
    header: Header,
    width: u32,
    height: u32,
    mode: Signal,
}
#[repr(C)]
#[derive(Default)]
struct MonitorInfo {
    size: u32,
    monitor: [i32; 4],
    work: [i32; 4],
    flags: u32,
    name: [u16; 32],
}

const ACTIVE_PATHS: u32 = 2;
const INSUFFICIENT_BUFFER: i32 = 122;
#[link(name = "user32")]
unsafe extern "system" {
    fn GetDisplayConfigBufferSizes(flags: u32, paths: *mut u32, modes: *mut u32) -> i32;
    fn QueryDisplayConfig(
        flags: u32,
        path_count: *mut u32,
        paths: *mut DisplayPath,
        mode_count: *mut u32,
        modes: *mut Mode,
        topology: *mut u32,
    ) -> i32;
    fn DisplayConfigGetDeviceInfo(header: *mut Header) -> i32;
    fn GetMonitorInfoW(monitor: *mut c_void, info: *mut MonitorInfo) -> i32;
}

fn header<T>(kind: u32, adapter: Luid, id: u32) -> Header {
    Header {
        kind,
        size: size_of::<T>() as u32,
        adapter,
        id,
    }
}

pub(super) fn preferred_resolution(monitor: *mut c_void) -> Option<(u64, u64)> {
    // All FFI buffers are fully initialized repr(C) structures with sizes
    // checked against the Windows ABI below. These calls never change modes.
    unsafe {
        let mut info = MonitorInfo {
            size: size_of::<MonitorInfo>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(monitor, &mut info) == 0 {
            return None;
        }
        // Displays can connect or disconnect between buffer sizing and querying.
        for _ in 0..3 {
            let (mut path_count, mut mode_count) = (0, 0);
            if GetDisplayConfigBufferSizes(ACTIVE_PATHS, &mut path_count, &mut mode_count) != 0 {
                return None;
            }
            let mut paths = vec![DisplayPath::default(); path_count as usize];
            let mut modes = vec![Mode::default(); mode_count as usize];
            let result = QueryDisplayConfig(
                ACTIVE_PATHS,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                std::ptr::null_mut(),
            );
            if result == INSUFFICIENT_BUFFER {
                continue;
            }
            if result != 0 {
                return None;
            }
            for path in paths.iter().take(path_count as usize) {
                let mut name = SourceName {
                    header: header::<SourceName>(1, path.source.adapter, path.source.id),
                    ..Default::default()
                };
                if DisplayConfigGetDeviceInfo(&mut name.header) != 0
                    || !name
                        .name
                        .iter()
                        .take_while(|ch| **ch != 0)
                        .eq(info.name.iter().take_while(|ch| **ch != 0))
                {
                    continue;
                }
                let mut preferred = PreferredMode {
                    header: header::<PreferredMode>(3, path.target.adapter, path.target.id),
                    ..Default::default()
                };
                if DisplayConfigGetDeviceInfo(&mut preferred.header) == 0 {
                    return super::oriented_resolution(
                        preferred.width,
                        preferred.height,
                        path.target.rotation,
                    );
                }
            }
            return None;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_structures_match_the_windows_abi() {
        assert_eq!(size_of::<DisplayPath>(), 72);
        assert_eq!(size_of::<Mode>(), 64);
        assert_eq!(size_of::<Header>(), 20);
        assert_eq!(size_of::<SourceName>(), 84);
        assert_eq!(size_of::<PreferredMode>(), 80);
        assert_eq!(size_of::<MonitorInfo>(), 104);
    }
}
