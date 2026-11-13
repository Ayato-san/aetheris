//! A separate non-activating, layered Win32 window. No game memory is read.
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::*,
};

#[derive(Clone)]
enum Content {
    Compass(Arc<Mutex<super::Orientation>>),
    Presence(Arc<Mutex<super::cube_decoder::CubeConsumer>>),
    Guidance(Arc<Mutex<super::cube_decoder::CubeConsumer>>),
    PresencePreview,
    GuidancePreview,
}
impl Content {
    fn private(&self) -> bool {
        matches!(self, Self::Guidance(_) | Self::GuidancePreview)
    }
    fn configuration_validated(&self) -> bool {
        // The capture probe displays exclusively fixed fictional data. It does
        // not authorize the real widget or share its tracking/IPC subscription.
        matches!(self, Self::GuidancePreview)
            || super::guidance_visibility::configuration_validated()
    }
    fn panel(&self) -> Option<super::cube_guidance::Panel> {
        match self {
            Self::GuidancePreview => Some(super::cube_guidance::Panel::preview()),
            Self::Guidance(state) if self.configuration_validated() => {
                let mut state = state.lock().unwrap();
                let enabled = state.guidance_enabled;
                super::cube_guidance::panel(&mut state.tracking, enabled, std::time::Instant::now())
            }
            _ => None,
        }
    }
    fn text(&self) -> Option<String> {
        match self {
            Self::Compass(_) => Some(super::ORIENTATION_UNAVAILABLE.into()),
            Self::Presence(state) => state
                .lock()
                .unwrap()
                .tracking
                .presence(std::time::Instant::now()),
            Self::PresencePreview => Some(
                "TEST · Données fictives\n1 cube fictif · détection continue 0 min 00 s".into(),
            ),
            Self::Guidance(_) | Self::GuidancePreview => self.panel().map(|panel| panel.text()),
        }
    }
}
pub(super) struct CubeWindow {
    _presence: OverlayWindow,
    _guidance: OverlayWindow,
}
impl CubeWindow {
    pub fn preview() -> Self {
        Self {
            _presence: OverlayWindow::start_content(Content::PresencePreview),
            _guidance: OverlayWindow::start_content(Content::GuidancePreview),
        }
    }
    pub fn status(&self) -> String {
        self._presence.status()
    }
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
                if state.as_ref().is_some_and(Content::private) {
                    let mut affinity = 0;
                    let mut composition = 0;
                    if !super::guidance_visibility::may_show(
                        state.as_ref().is_some_and(Content::configuration_validated),
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
                if let Some(panel) = state.as_ref().and_then(Content::panel) {
                    draw_guidance(
                        dc,
                        &panel,
                        rect,
                        matches!(state.as_ref(), Some(Content::GuidancePreview)),
                    );
                } else if let Some(ticks) = ticks {
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

/// A fixed world-axis locator: -Y at the top and +X at the right. It never
/// rotates from movement, facing or an unvalidated cardinal calibration.
unsafe fn draw_guidance(
    dc: HDC,
    panel: &super::cube_guidance::Panel,
    mut bounds: RECT,
    preview: bool,
) {
    unsafe {
        let heading = wide(if preview {
            "TEST · GUIDAGE FICTIF"
        } else {
            "GUIDAGE"
        });
        let mut header = RECT {
            left: 120,
            top: 3,
            right: bounds.right - 8,
            bottom: 22,
        };
        DrawTextW(
            dc,
            heading.as_ptr(),
            (heading.len() - 1) as i32,
            &mut header,
            DT_LEFT | DT_SINGLELINE,
        );
        bounds.left = 120;
        bounds.top = 22;
        bounds.right -= 8;
        let text = wide(&panel.text());
        DrawTextW(
            dc,
            text.as_ptr(),
            (text.len() - 1) as i32,
            &mut bounds,
            DT_LEFT | DT_WORDBREAK,
        );
        let Some(direction) = panel.world_direction() else {
            return;
        };
        let pen = CreatePen(PS_SOLID, 1, 0x00b2d165);
        let previous_pen = SelectObject(dc, pen);
        let previous_brush = SelectObject(dc, GetStockObject(HOLLOW_BRUSH));
        Ellipse(dc, 16, 25, 96, 105);
        let x = 56 + (direction[0] * 34.).round() as i32;
        let y = 65 + (direction[1] * 34.).round() as i32;
        MoveToEx(dc, 56, 65, std::ptr::null_mut());
        LineTo(dc, x, y);
        Ellipse(dc, 53, 62, 59, 68); // player
        let brush = CreateSolidBrush(0x00b2d165);
        SelectObject(dc, brush);
        Ellipse(dc, x - 4, y - 4, x + 4, y + 4); // cube
        SelectObject(dc, previous_brush);
        DeleteObject(brush);
        SelectObject(dc, previous_pen);
        DeleteObject(pen);
        let label = wide("−Y");
        let mut top = RECT {
            left: 36,
            top: 5,
            right: 76,
            bottom: 24,
        };
        DrawTextW(
            dc,
            label.as_ptr(),
            (label.len() - 1) as i32,
            &mut top,
            DT_CENTER | DT_SINGLELINE,
        );
        let label = wide("+X");
        let mut right = RECT {
            left: 97,
            top: 56,
            right: 120,
            bottom: 76,
        };
        DrawTextW(
            dc,
            label.as_ptr(),
            (label.len() - 1) as i32,
            &mut right,
            DT_LEFT | DT_SINGLELINE,
        );
        let label = wide("Repère monde");
        let mut bottom = RECT {
            left: 5,
            top: 111,
            right: 116,
            bottom: 136,
        };
        DrawTextW(
            dc,
            label.as_ptr(),
            (label.len() - 1) as i32,
            &mut bottom,
            DT_CENTER | DT_SINGLELINE,
        );
    }
}

/// Accept only a visible foreground root whose owning executable is known.
/// Client bounds exclude window decorations and work on secondary monitors.
fn game_bounds() -> Result<Option<RECT>, String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null()
            || IsWindowVisible(hwnd) == 0
            || IsIconic(hwnd) != 0
            || GetAncestor(hwnd, GA_ROOT) != hwnd
        {
            return Ok(None);
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        match crate::game_process::identify(pid) {
            crate::game_process::GameStatus::Running => {}
            crate::game_process::GameStatus::Stopped => return Ok(None),
            crate::game_process::GameStatus::Unknown => {
                return Err("Identification du processus du jeu indisponible".into());
            }
        }
        let mut rect: RECT = std::mem::zeroed();
        let mut origin = POINT { x: 0, y: 0 };
        if GetClientRect(hwnd, &mut rect) == 0 || ClientToScreen(hwnd, &mut origin) == 0 {
            return Err(format!(
                "Coordonnées de la fenêtre du jeu indisponibles : {}",
                std::io::Error::last_os_error()
            ));
        }
        if rect.right < 360 || rect.bottom < 100 {
            return Err("Fenêtre du jeu trop petite pour l’overlay".into());
        }
        // Recheck ownership of the foreground after querying the process.
        let mut current_pid = 0;
        GetWindowThreadProcessId(hwnd, &mut current_pid);
        if GetForegroundWindow() != hwnd || current_pid != pid {
            return Ok(None);
        }
        Ok(Some(RECT {
            left: origin.x,
            top: origin.y,
            right: origin.x + rect.right,
            bottom: origin.y + rect.bottom,
        }))
    }
}

fn run(stop: Arc<AtomicBool>, status: Arc<Mutex<String>>, content: Content) -> std::io::Result<()> {
    // Stable storage for WM_PAINT, valid until DestroyWindow returns.
    let content = Box::new(content);
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let private = content.private();
        let name = wide(match &*content {
            Content::Compass(_) => "AetherisCompassOverlay",
            Content::Presence(_) => "AetherisCubePresence",
            Content::Guidance(_) => "AetherisCubeGuidance",
            Content::PresencePreview => "AetherisCubePresencePreview",
            Content::GuidancePreview => "AetherisCubeGuidancePreview",
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
        // capture. Version qualification is checked independently below.
        let mut affinity = 0;
        let mut composition = 0;
        let protected = private
            && windows_sys::Win32::Graphics::Dwm::DwmIsCompositionEnabled(&mut composition) == 0
            && composition != 0
            && SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) != 0
            && GetWindowDisplayAffinity(hwnd, &mut affinity) != 0
            && affinity == WDA_EXCLUDEFROMCAPTURE;
        // Keep the last in-game observation when the user switches back to the
        // editor. Otherwise the only observable status is always "waiting".
        let mut last_display = None;
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
                    content.configuration_validated(),
                    protected
                        && GetWindowDisplayAffinity(hwnd, &mut affinity) != 0
                        && affinity == WDA_EXCLUDEFROMCAPTURE,
                    windows_sys::Win32::Graphics::Dwm::DwmIsCompositionEnabled(&mut composition)
                        == 0
                        && composition != 0,
                );
            let Some(text) = content.text().filter(|_| protection_valid) else {
                ShowWindow(hwnd, SW_HIDE);
                std::thread::sleep(Duration::from_millis(50));
                continue;
            };
            match game_bounds() {
                Ok(Some(rect)) => {
                    if SetWindowPos(
                        hwnd,
                        HWND_TOPMOST,
                        rect.left
                            + if matches!(
                                &*content,
                                Content::PresencePreview | Content::GuidancePreview
                            ) {
                                20.min(rect.right - rect.left - 360)
                            } else {
                                (rect.right - rect.left - 360) / 2
                            },
                        rect.top
                            + 20
                            + match &*content {
                                Content::Guidance(state) => state
                                    .lock()
                                    .unwrap()
                                    .tracking
                                    .presence(std::time::Instant::now())
                                    .map(|text| (text.lines().count() as i32 * 20).min(600) + 8)
                                    .unwrap_or(0),
                                Content::GuidancePreview => 48,
                                _ => 0,
                            },
                        360,
                        if content.private() {
                            140
                        } else if matches!(
                            &*content,
                            Content::Presence(_) | Content::PresencePreview
                        ) {
                            (text.lines().count() as i32 * 20).min(600)
                        } else {
                            44
                        },
                        SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    ) == 0
                    {
                        ShowWindow(hwnd, SW_HIDE);
                        *status.lock().unwrap() = "Position de l’overlay indisponible".into();
                    } else {
                        let observation = format!(
                            "Fenêtre affichée en jeu · zone client ({}, {})–({}, {})",
                            rect.left, rect.top, rect.right, rect.bottom
                        );
                        last_display = Some(observation.clone());
                        *status.lock().unwrap() = observation;
                    }
                }
                Ok(None) => {
                    ShowWindow(hwnd, SW_HIDE);
                    let mut message = status.lock().unwrap();
                    if !message.starts_with("Fenêtre du jeu indisponible") {
                        *message = last_display
                            .as_ref()
                            .map(|last| format!("Jeu hors du premier plan · dernier résultat : {last}"))
                            .unwrap_or_else(|| "En attente du jeu au premier plan · aucun affichage en jeu confirmé".into());
                    }
                }
                Err(error) => {
                    ShowWindow(hwnd, SW_HIDE);
                    *status.lock().unwrap() = format!("Fenêtre du jeu indisponible : {error}");
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        DestroyWindow(hwnd);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fictional_capture_probe_does_not_read_or_create_live_targets() {
        let now = std::time::Instant::now();
        let mut consumer = super::super::cube_decoder::CubeConsumer::default();
        consumer.set_guidance(true);
        consumer.tracking.detect(
            99,
            1200039,
            super::super::hidden_cubes::Position {
                x: 1000.,
                y: 0.,
                z: 0.,
            },
            now,
        );
        consumer.tracking.position(
            super::super::hidden_cubes::Position {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            now,
        );
        let live = Arc::new(Mutex::new(consumer));
        let real = Content::Guidance(live.clone());
        let probe = Content::GuidancePreview;
        assert!(probe.private());
        assert!(probe.configuration_validated());
        assert_eq!(
            probe.panel().unwrap(),
            super::super::cube_guidance::Panel::preview()
        );
        assert!(
            Content::PresencePreview
                .text()
                .unwrap()
                .contains("Données fictives")
        );
        assert_eq!(
            live.lock().unwrap().tracking.entities().collect::<Vec<_>>(),
            vec![99]
        );
        live.lock().unwrap().tracking.reset();
        assert!(real.panel().is_none());
        assert!(real.text().is_none());
        assert_eq!(
            probe.panel().unwrap(),
            super::super::cube_guidance::Panel::preview()
        );
    }
}
