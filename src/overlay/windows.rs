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

#[derive(Clone)]
enum Content {
    Compass(Arc<Mutex<super::Orientation>>),
    Presence(Arc<Mutex<super::cube_decoder::CubeConsumer>>),
    Guidance(Arc<Mutex<super::cube_decoder::CubeConsumer>>),
}
impl Content {
    fn text(&self) -> Option<String> {
        match self {
            Self::Compass(_) => Some(super::ORIENTATION_UNAVAILABLE.into()),
            Self::Presence(state) => state
                .lock()
                .unwrap()
                .tracking
                .presence(std::time::Instant::now()),
            Self::Guidance(_) if !super::guidance_visibility::configuration_validated() => None,
            Self::Guidance(state) => state
                .lock()
                .unwrap()
                .tracking
                .guidance(std::time::Instant::now())
                .map(|g| {
                    format!(
                        "Distance horizontale {:.1} m · hauteur {:+.1} m",
                        g.horizontal_m, g.elevation_m
                    )
                }),
        }
    }
}
pub(super) struct CubeWindow {
    _presence: OverlayWindow,
    _guidance: OverlayWindow,
}
impl CubeWindow {
    pub fn start(state: Arc<Mutex<super::cube_decoder::CubeConsumer>>) -> Self {
        Self {
            _presence: OverlayWindow::start_content(Content::Presence(state.clone())),
            _guidance: OverlayWindow::start_content(Content::Guidance(state)),
        }
    }
}
pub(super) struct OverlayWindow {
    stop: Arc<AtomicBool>,
    status: Arc<Mutex<String>>,
}
impl OverlayWindow {
    pub fn start(orientation: Arc<Mutex<super::Orientation>>) -> Self {
        Self::start_content(Content::Compass(orientation))
    }
    fn start_content(content: Content) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let status = Arc::new(Mutex::new("En attente du jeu au premier plan".into()));
        let cancel = stop.clone();
        let message = status.clone();
        std::thread::spawn(move || {
            if let Err(error) = run(cancel, message.clone(), content) {
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
                let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Content;
                // Hidden windows can still receive paint requests. Apply the
                // private-content gate here too, before generating any text.
                if state
                    .as_ref()
                    .is_some_and(|state| matches!(state, Content::Guidance(_)))
                {
                    let mut affinity = 0;
                    let mut composition = 0;
                    if !super::guidance_visibility::may_show(
                        super::guidance_visibility::configuration_validated(),
                        GetWindowDisplayAffinity(hwnd, &mut affinity) != 0
                            && affinity == WDA_EXCLUDEFROMCAPTURE,
                        windows_sys::Win32::Graphics::Dwm::DwmIsCompositionEnabled(
                            &mut composition,
                        ) == 0
                            && composition != 0,
                    ) {
                        EndPaint(hwnd, &paint);
                        return 0;
                    }
                }
                let ticks = state.as_ref().and_then(|state| match state {
                    Content::Compass(orientation) => {
                        orientation.lock().unwrap().ticks(std::time::Instant::now())
                    }
                    _ => None,
                });
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
                    let text = wide(&state.as_ref().and_then(|s| s.text()).unwrap_or_default());
                    DrawTextW(
                        dc,
                        text.as_ptr(),
                        (text.len() - 1) as i32,
                        &mut rect,
                        DT_CENTER | DT_WORDBREAK,
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

fn run(stop: Arc<AtomicBool>, status: Arc<Mutex<String>>, content: Content) -> std::io::Result<()> {
    // Stable storage for WM_PAINT, valid until DestroyWindow returns.
    let content = Box::new(content);
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let private = matches!(&*content, Content::Guidance(_));
        let name = wide(match &*content {
            Content::Compass(_) => "AetherisCompassOverlay",
            Content::Presence(_) => "AetherisCubePresence",
            Content::Guidance(_) => "AetherisCubeGuidance",
        });
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
            wide("Aetheris · Overlay").as_ptr(),
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
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, (&*content as *const Content) as isize);
        if SetLayeredWindowAttributes(hwnd, 0, 230, LWA_ALPHA) == 0 {
            let error = std::io::Error::last_os_error();
            DestroyWindow(hwnd);
            return Err(error);
        }
        // A successful API call alone does not validate OBS game/window/display
        // capture. The supported configuration matrix has no validated entry yet.
        let mut affinity = 0;
        let mut composition = 0;
        let protected = private
            && windows_sys::Win32::Graphics::Dwm::DwmIsCompositionEnabled(&mut composition) == 0
            && composition != 0
            && SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) != 0
            && GetWindowDisplayAffinity(hwnd, &mut affinity) != 0
            && affinity == WDA_EXCLUDEFROMCAPTURE;
        while !stop.load(Ordering::Acquire) {
            let mut message: MSG = std::mem::zeroed();
            while PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            InvalidateRect(hwnd, std::ptr::null(), 0);
            // Recheck affinity/composition while running; never put an explanation
            // for private guidance into the capturable presence window.
            let protection_valid = !private
                || super::guidance_visibility::may_show(
                    super::guidance_visibility::configuration_validated(),
                    protected
                        && GetWindowDisplayAffinity(hwnd, &mut affinity) != 0
                        && affinity == WDA_EXCLUDEFROMCAPTURE,
                    windows_sys::Win32::Graphics::Dwm::DwmIsCompositionEnabled(&mut composition)
                        == 0
                        && composition != 0,
                );
            let text = content.text();
            if !protection_valid || text.is_none() {
                ShowWindow(hwnd, SW_HIDE);
            } else if let Some(rect) = game_bounds() {
                if SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    rect.left + (rect.right - rect.left - 360) / 2,
                    rect.top + 20,
                    360,
                    if matches!(&*content, Content::Presence(_)) {
                        (text.as_ref().unwrap().lines().count() as i32 * 20).min(600)
                    } else {
                        44
                    },
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
