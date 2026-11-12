//! A separate non-activating, layered Win32 window. No game memory is read.
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::{LibraryLoader::GetModuleHandleW, Threading::*},
    UI::WindowsAndMessaging::*,
};

pub(super) struct OverlayWindow {
    stop: Arc<AtomicBool>,
    status: Arc<Mutex<String>>,
}
impl OverlayWindow {
    pub fn start(orientation: Arc<Mutex<super::Orientation>>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let status = Arc::new(Mutex::new("En attente du jeu au premier plan".into()));
        let cancel = stop.clone();
        let message = status.clone();
        std::thread::spawn(move || {
            if let Err(error) = run(cancel, message.clone(), orientation) {
                *message.lock().unwrap() = format!("Fenêtre d’overlay indisponible : {error}");
            }
        });
        Self { stop, status }
    }
    pub fn status(&self) -> String {
        self.status.lock().unwrap().clone()
    }
}
impl Drop for OverlayWindow {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // All painting resources are created and released on this window's thread.
    unsafe {
        match message {
            WM_NCHITTEST => HTTRANSPARENT as LRESULT,
            WM_MOUSEACTIVATE => MA_NOACTIVATE as LRESULT,
            WM_PAINT => {
                let mut paint: PAINTSTRUCT = std::mem::zeroed();
                let dc = BeginPaint(hwnd, &mut paint);
                let mut rect: RECT = std::mem::zeroed();
                GetClientRect(hwnd, &mut rect);
                let brush = CreateSolidBrush(0x00241d18);
                FillRect(dc, &rect, brush);
                DeleteObject(brush);
                SetBkMode(dc, TRANSPARENT as i32);
                SetTextColor(dc, 0x00f5f5f5);
                let state =
                    GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Arc<Mutex<super::Orientation>>;
                let ticks = state
                    .as_ref()
                    .and_then(|state| state.lock().unwrap().ticks(std::time::Instant::now()));
                if let Some(ticks) = ticks {
                    let pen = CreatePen(PS_SOLID, 1, 0x00f5f5f5);
                    let previous = SelectObject(dc, pen);
                    for tick in ticks {
                        let x = 180 + (tick.offset * 1.8).round() as i32;
                        MoveToEx(dc, x, 30, std::ptr::null_mut());
                        LineTo(dc, x, if tick.label.is_empty() { 35 } else { 40 });
                        let text = wide(tick.label);
                        let mut bounds = RECT {
                            left: x - 16,
                            right: x + 16,
                            top: 8,
                            bottom: 28,
                        };
                        DrawTextW(
                            dc,
                            text.as_ptr(),
                            (text.len() - 1) as i32,
                            &mut bounds,
                            DT_CENTER | DT_SINGLELINE,
                        );
                    }
                    MoveToEx(dc, 180, 0, std::ptr::null_mut());
                    LineTo(dc, 180, 7);
                    SelectObject(dc, previous);
                    DeleteObject(pen);
                } else {
                    let text = wide(super::ORIENTATION_UNAVAILABLE);
                    DrawTextW(
                        dc,
                        text.as_ptr(),
                        (text.len() - 1) as i32,
                        &mut rect,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );
                }
                EndPaint(hwnd, &paint);
                0
            }
            _ => DefWindowProcW(hwnd, message, w, l),
        }
    }
}

/// Accept only a visible foreground root whose owning executable is known.
/// Client bounds exclude window decorations and work on secondary monitors.
fn game_bounds() -> Option<RECT> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null()
            || IsWindowVisible(hwnd) == 0
            || IsIconic(hwnd) != 0
            || GetAncestor(hwnd, GA_ROOT) != hwnd
        {
            return None;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return None;
        }
        let mut path = [0u16; 32768];
        let mut length = path.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, path.as_mut_ptr(), &mut length);
        CloseHandle(process);
        if ok == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&path[..length as usize]);
        if !crate::game_process::is_game_executable(path.rsplit('\\').next()?) {
            return None;
        }
        let mut rect: RECT = std::mem::zeroed();
        let mut origin = POINT { x: 0, y: 0 };
        if GetClientRect(hwnd, &mut rect) == 0
            || ClientToScreen(hwnd, &mut origin) == 0
            || rect.right < 360
            || rect.bottom < 100
        {
            return None;
        }
        // Recheck ownership of the foreground after querying the process.
        if GetForegroundWindow() != hwnd {
            return None;
        }
        Some(RECT {
            left: origin.x,
            top: origin.y,
            right: origin.x + rect.right,
            bottom: origin.y + rect.bottom,
        })
    }
}

fn run(
    stop: Arc<AtomicBool>,
    status: Arc<Mutex<String>>,
    orientation: Arc<Mutex<super::Orientation>>,
) -> std::io::Result<()> {
    // Stable storage for WM_PAINT, valid until DestroyWindow returns.
    let orientation = Box::new(orientation);
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let name = wide("AetherisCompassOverlay");
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: name.as_ptr(),
            ..std::mem::zeroed()
        };
        // Class registration is shared across toggle cycles. ERROR_CLASS_ALREADY_EXISTS is harmless.
        if RegisterClassW(&class) == 0 && GetLastError() != 1410 {
            return Err(std::io::Error::last_os_error());
        }
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            name.as_ptr(),
            wide("Aetheris · Boussole").as_ptr(),
            WS_POPUP,
            0,
            0,
            360,
            44,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        SetWindowLongPtrW(
            hwnd,
            GWLP_USERDATA,
            (&*orientation as *const Arc<Mutex<super::Orientation>>) as isize,
        );
        if SetLayeredWindowAttributes(hwnd, 0, 230, LWA_ALPHA) == 0 {
            let error = std::io::Error::last_os_error();
            DestroyWindow(hwnd);
            return Err(error);
        }
        while !stop.load(Ordering::Acquire) {
            let mut message: MSG = std::mem::zeroed();
            while PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            InvalidateRect(hwnd, std::ptr::null(), 0);
            if let Some(rect) = game_bounds() {
                if SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    rect.left + (rect.right - rect.left - 360) / 2,
                    rect.top + 20,
                    360,
                    44,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                ) == 0
                {
                    ShowWindow(hwnd, SW_HIDE);
                    *status.lock().unwrap() = "Position de l’overlay indisponible".into();
                } else {
                    *status.lock().unwrap() = "Jeu au premier plan".into();
                }
            } else {
                ShowWindow(hwnd, SW_HIDE);
                *status.lock().unwrap() = "En attente du jeu au premier plan".into();
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        DestroyWindow(hwnd);
    }
    Ok(())
}
