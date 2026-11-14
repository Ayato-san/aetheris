//! A separate non-activating, layered Win32 window. No game memory is read.
use std::{
    cell::{Cell, RefCell},
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

// Owned pixels-to-paint input. The window procedure never takes the capture
// consumer's mutex or recalculates a target during synchronous Win32 messages.
enum Frame {
    Text {
        text: String,
        height: i32,
    },
    Compass(Vec<super::CompassTick>),
    Guidance {
        panel: super::cube_guidance::Panel,
        offset: i32,
        expires_at: Option<std::time::Instant>,
        probe_report: Option<String>,
    },
}
impl Frame {
    fn presence(text: String) -> Self {
        let height = (text.lines().count() as i32 * 20).min(600);
        Self::Text { text, height }
    }
    fn height(&self) -> i32 {
        match self {
            Self::Text { height, .. } => *height,
            Self::Compass(_) => 44,
            Self::Guidance { probe_report, .. } => {
                if probe_report.is_some() {
                    240
                } else {
                    140
                }
            }
        }
    }
    fn offset(&self) -> i32 {
        match self {
            Self::Guidance { offset, .. } => *offset,
            _ => 0,
        }
    }
    fn guidance_panel(&self, now: std::time::Instant) -> Option<&super::cube_guidance::Panel> {
        match self {
            Self::Guidance {
                panel, expires_at, ..
            } => Some(if expires_at.is_some_and(|deadline| now >= deadline) {
                &super::cube_guidance::Panel::AwaitingPosition
            } else {
                panel
            }),
            _ => None,
        }
    }
}
struct WindowState {
    content: Content,
    frame: RefCell<Option<Frame>>,
    painted: Cell<u64>,
}
impl Content {
    fn preview(&self) -> bool {
        matches!(self, Self::PresencePreview | Self::GuidancePreview)
    }
    fn private(&self) -> bool {
        matches!(self, Self::Guidance(_) | Self::GuidancePreview)
    }
    fn configuration_validated(&self) -> bool {
        // The capture probe displays exclusively fixed fictional data. It does
        // not authorize the real widget or share its tracking/IPC subscription.
        matches!(self, Self::GuidancePreview)
            || super::guidance_visibility::configuration_validated()
    }
    fn snapshot(&self) -> Option<Frame> {
        match self {
            Self::Compass(state) => Some(
                state
                    .lock()
                    .unwrap()
                    .ticks(std::time::Instant::now())
                    .map(Frame::Compass)
                    .unwrap_or_else(|| Frame::Text {
                        text: super::ORIENTATION_UNAVAILABLE.into(),
                        height: 44,
                    }),
            ),
            Self::Presence(state) => state
                .lock()
                .unwrap()
                .tracking
                .presence(std::time::Instant::now())
                .map(Frame::presence),
            Self::PresencePreview => Some(Frame::presence(
                "TEST · Données fictives\n1 cube fictif · détection continue 0 min 00 s".into(),
            )),
            Self::GuidancePreview => Some(Frame::Guidance {
                panel: super::cube_guidance::Panel::preview(),
                offset: 48,
                expires_at: None,
                probe_report: Some(super::guidance_visibility::probe_report()),
            }),
            Self::Guidance(state) if self.configuration_validated() => {
                let mut state = state.lock().unwrap();
                let enabled = state.guidance_enabled;
                let now = std::time::Instant::now();
                let panel = super::cube_guidance::panel(&mut state.tracking, enabled, now)?;
                let offset = state
                    .tracking
                    .presence(now)
                    .map(|text| (text.lines().count() as i32 * 20).min(600) + 8)
                    .unwrap_or(0);
                let expires_at = state.tracking.position_deadline();
                Some(Frame::Guidance {
                    panel,
                    offset,
                    expires_at,
                    probe_report: None,
                })
            }
            _ => None,
        }
    }
    #[cfg(test)]
    fn panel(&self) -> Option<super::cube_guidance::Panel> {
        match self.snapshot()? {
            Frame::Guidance { panel, .. } => Some(panel),
            _ => None,
        }
    }
    #[cfg(test)]
    fn text(&self) -> Option<String> {
        match self.snapshot()? {
            Frame::Text { text, .. } => Some(text),
            Frame::Guidance { panel, .. } => Some(panel.text()),
            _ => None,
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
                let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const WindowState;
                let Some(state) = state.as_ref() else {
                    EndPaint(hwnd, &paint);
                    return 0;
                };
                // Hidden windows can still receive paint requests. Apply the
                // private-content gate here too, before generating any text.
                if state.content.private() {
                    let mut affinity = 0;
                    let mut composition = 0;
                    if !super::guidance_visibility::may_show(
                        state.content.configuration_validated(),
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
                let frame = state.frame.borrow();
                if let Some(panel) = frame
                    .as_ref()
                    .and_then(|frame| frame.guidance_panel(std::time::Instant::now()))
                {
                    draw_guidance(
                        dc,
                        panel,
                        rect,
                        matches!(state.content, Content::GuidancePreview),
                    );
                    if let Some(Frame::Guidance {
                        probe_report: Some(report),
                        ..
                    }) = frame.as_ref()
                    {
                        let text = wide(report);
                        let mut bounds = RECT {
                            left: 8,
                            top: 142,
                            right: rect.right - 8,
                            bottom: rect.bottom - 4,
                        };
                        DrawTextW(
                            dc,
                            text.as_ptr(),
                            (text.len() - 1) as i32,
                            &mut bounds,
                            DT_LEFT | DT_WORDBREAK,
                        );
                    }
                } else if let Some(Frame::Compass(ticks)) = frame.as_ref() {
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
                } else if let Some(Frame::Text { text, .. }) = frame.as_ref() {
                    let text = wide(text);
                    DrawTextW(
                        dc,
                        text.as_ptr(),
                        (text.len() - 1) as i32,
                        &mut rect,
                        DT_CENTER | DT_WORDBREAK,
                    );
                }
                if frame.is_some() {
                    state.painted.set(state.painted.get().saturating_add(1));
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
    run_with_bounds(stop, status, content, game_bounds)
}

fn run_with_bounds(
    stop: Arc<AtomicBool>,
    status: Arc<Mutex<String>>,
    content: Content,
    mut bounds: impl FnMut() -> Result<Option<RECT>, String>,
) -> std::io::Result<()> {
    // Start exactly when there is content and a foreground game, rather than
    // leaving an empty layered window around since GPUI's startup. The probe
    // and live content use the same native class and creation path.
    let initial_frame = loop {
        if stop.load(Ordering::Acquire) {
            return Ok(());
        }
        if let Some(frame) = content.snapshot() {
            match bounds() {
                Ok(Some(_)) => break frame,
                Ok(None) => {}
                Err(error) => {
                    *status.lock().unwrap() = format!("Fenêtre du jeu indisponible : {error}");
                }
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    // Stable storage for WM_PAINT, valid until DestroyWindow returns.
    let state = Box::new(WindowState {
        content,
        frame: RefCell::new(Some(initial_frame)),
        painted: Cell::new(0),
    });
    let content = &state.content;
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let private = content.private();
        let name = wide(match content {
            Content::Compass(_) => "AetherisCompassOverlay",
            Content::Presence(_) | Content::PresencePreview => "AetherisCubePresence",
            Content::Guidance(_) | Content::GuidancePreview => "AetherisCubeGuidance",
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
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
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
        SetWindowLongPtrW(
            hwnd,
            GWLP_USERDATA,
            (&*state as *const WindowState) as isize,
        );
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
            // An active paint queue must not starve visibility/data updates.
            for _ in 0..32 {
                if PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) == 0 {
                    break;
                }
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
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
            let frame = protection_valid.then(|| content.snapshot()).flatten();
            let Some(frame) = frame else {
                ShowWindow(hwnd, SW_HIDE);
                state.frame.replace(None);
                std::thread::sleep(Duration::from_millis(50));
                continue;
            };
            let height = frame.height();
            let offset = frame.offset();
            state.frame.replace(Some(frame));
            match bounds() {
                Ok(Some(rect)) => {
                    if SetWindowPos(
                        hwnd,
                        HWND_TOPMOST,
                        rect.left
                            + (if content.preview() { 400 } else { 20 })
                                .min(rect.right - rect.left - 360),
                        rect.top + 20 + offset,
                        360,
                        height,
                        SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    ) == 0
                    {
                        ShowWindow(hwnd, SW_HIDE);
                        *status.lock().unwrap() = "Position de l’overlay indisponible".into();
                    } else {
                        // Publish the current snapshot to the layered surface
                        // immediately after showing/moving it, including the
                        // first real detection after a long hidden interval.
                        InvalidateRect(hwnd, std::ptr::null(), 0);
                        UpdateWindow(hwnd);
                        let mut actual = RECT::default();
                        let positioned = GetWindowRect(hwnd, &mut actual) != 0;
                        let mut cloaked = 0u32;
                        let cloak_known = windows_sys::Win32::Graphics::Dwm::DwmGetWindowAttribute(
                            hwnd,
                            windows_sys::Win32::Graphics::Dwm::DWMWA_CLOAKED as u32,
                            (&mut cloaked as *mut u32).cast(),
                            std::mem::size_of_val(&cloaked) as u32,
                        ) == 0;
                        let observation = format!(
                            "Fenêtre native ({}, {})–({}, {}) · coordonnées {} · visible {} · masquage DWM {} · {} peintures effectuées",
                            actual.left,
                            actual.top,
                            actual.right,
                            actual.bottom,
                            if positioned {
                                "confirmées"
                            } else {
                                "indisponibles"
                            },
                            if IsWindowVisible(hwnd) != 0 {
                                "oui"
                            } else {
                                "non"
                            },
                            if cloak_known {
                                cloaked.to_string()
                            } else {
                                "indisponible".into()
                            },
                            state.painted.get()
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
    fn cached_guidance_expires_even_before_the_next_network_snapshot() {
        let deadline = std::time::Instant::now();
        let frame = Frame::Guidance {
            panel: super::super::cube_guidance::Panel::preview(),
            probe_report: None,
            offset: 48,
            expires_at: Some(deadline),
        };
        assert!(matches!(
            frame.guidance_panel(deadline - Duration::from_millis(1)),
            Some(super::super::cube_guidance::Panel::Target(_))
        ));
        assert_eq!(
            frame.guidance_panel(deadline),
            Some(&super::super::cube_guidance::Panel::AwaitingPosition)
        );
    }
    #[test]
    #[ignore = "creates native windows; requires an interactive Windows desktop"]
    fn real_presence_is_painted_when_a_cube_arrives_after_window_creation() {
        use crate::capture::widgets::Consumer;
        use std::time::Instant;
        let consumer = Arc::new(Mutex::new(
            super::super::cube_decoder::CubeConsumer::default(),
        ));
        let stop = Arc::new(AtomicBool::new(false));
        struct StopOnDrop(Arc<AtomicBool>);
        impl Drop for StopOnDrop {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Release);
            }
        }
        let _cleanup = StopOnDrop(stop.clone());
        let status = Arc::new(Mutex::new(String::new()));
        let (send, receive) = std::sync::mpsc::channel();
        let cancel = stop.clone();
        let state = consumer.clone();
        let worker_status = status.clone();
        let foreground = Arc::new(AtomicBool::new(false));
        let game_foreground = foreground.clone();
        let worker = std::thread::spawn(move || {
            send.send(unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() })
                .unwrap();
            run_with_bounds(cancel, worker_status, Content::Presence(state), || {
                Ok(game_foreground.load(Ordering::Acquire).then_some(RECT {
                    left: 100,
                    top: 100,
                    right: 1000,
                    bottom: 700,
                }))
            })
        });
        // Exercise the real presence while another layered, protected panel
        // is repainting on its own native thread, as in the in-game probe.
        let preview_status = Arc::new(Mutex::new(String::new()));
        let preview_message = preview_status.clone();
        let preview_stop = stop.clone();
        let preview_worker = std::thread::spawn(move || {
            run_with_bounds(
                preview_stop,
                preview_message,
                Content::GuidancePreview,
                || {
                    Ok(Some(RECT {
                        left: 100,
                        top: 100,
                        right: 1000,
                        bottom: 700,
                    }))
                },
            )
        });
        let thread_id = receive.recv_timeout(Duration::from_secs(3)).unwrap();
        unsafe extern "system" fn find(hwnd: HWND, data: LPARAM) -> i32 {
            unsafe { *(data as *mut HWND) = hwnd };
            0
        }
        let wait = |predicate: &mut dyn FnMut() -> bool| {
            let until = Instant::now() + Duration::from_secs(5);
            while !predicate() {
                assert!(Instant::now() < until, "{}", status.lock().unwrap());
                std::thread::sleep(Duration::from_millis(20));
            }
        };
        let mut hwnd: HWND = std::ptr::null_mut();
        std::thread::sleep(Duration::from_millis(150));
        unsafe {
            EnumThreadWindows(thread_id, Some(find), (&mut hwnd as *mut HWND) as LPARAM);
        }
        assert!(hwnd.is_null(), "no native window before any cube");
        consumer.lock().unwrap().tracking.detect(
            28137,
            1200039,
            super::super::hidden_cubes::Position {
                x: -3998.,
                y: 89851.,
                z: 22067.,
            },
            Instant::now(),
        );
        std::thread::sleep(Duration::from_millis(150));
        unsafe {
            EnumThreadWindows(thread_id, Some(find), (&mut hwnd as *mut HWND) as LPARAM);
        }
        assert!(
            hwnd.is_null(),
            "no native window before the foreground game"
        );
        foreground.store(true, Ordering::Release);
        wait(&mut || unsafe {
            EnumThreadWindows(thread_id, Some(find), (&mut hwnd as *mut HWND) as LPARAM);
            !hwnd.is_null()
        });
        wait(&mut || unsafe { IsWindowVisible(hwnd) != 0 });
        wait(&mut || {
            preview_status
                .lock()
                .unwrap()
                .contains("peintures effectuées")
        });
        let mut rect = RECT::default();
        assert_ne!(unsafe { GetWindowRect(hwnd, &mut rect) }, 0);
        assert_eq!(
            (rect.left, rect.top, rect.right, rect.bottom),
            (120, 120, 480, 160)
        );
        // Inspect the actual window surface, rather than calling WM_PAINT or
        // drawing the expected text into a separate test bitmap.
        wait(&mut || unsafe {
            let dc = GetDC(hwnd);
            if dc.is_null() {
                return false;
            }
            let mut light = 0;
            let mut dark = 0;
            for y in (0..40).step_by(2) {
                for x in (0..360).step_by(2) {
                    let pixel = GetPixel(dc, x, y);
                    if pixel != CLR_INVALID {
                        let channels = [pixel & 255, (pixel >> 8) & 255, (pixel >> 16) & 255];
                        light += usize::from(channels.iter().all(|c| *c > 150));
                        dark += usize::from(channels.iter().all(|c| *c < 100));
                    }
                }
            }
            ReleaseDC(hwnd, dc);
            light > 10 && dark > 100
        });
        assert!(
            status
                .lock()
                .unwrap()
                .contains("visible oui · masquage DWM 0")
        );
        foreground.store(false, Ordering::Release);
        wait(&mut || unsafe { IsWindowVisible(hwnd) == 0 });
        foreground.store(true, Ordering::Release);
        wait(&mut || unsafe { IsWindowVisible(hwnd) != 0 });
        consumer
            .lock()
            .unwrap()
            .message([0x35, 0x36], &[0xe9, 0xdb, 1, 4], Instant::now());
        wait(&mut || unsafe { IsWindowVisible(hwnd) == 0 });
        // A second appearance must repaint and resize the same window.
        for entity in [1, 2] {
            consumer.lock().unwrap().tracking.detect(
                entity,
                1200039,
                super::super::hidden_cubes::Position {
                    x: 0.,
                    y: 0.,
                    z: 0.,
                },
                Instant::now(),
            );
        }
        wait(&mut || unsafe {
            IsWindowVisible(hwnd) != 0
                && GetWindowRect(hwnd, &mut rect) != 0
                && rect.bottom - rect.top == 60
        });
        // A late first Self must also withdraw cubes from an unattributed
        // character session, with guidance disabled in this real consumer.
        consumer
            .lock()
            .unwrap()
            .message([0x33, 0x36], &[10, 0, 0, 0, 0, 0], Instant::now());
        wait(&mut || unsafe { IsWindowVisible(hwnd) == 0 });
        consumer.lock().unwrap().tracking.detect(
            28137,
            1200039,
            super::super::hidden_cubes::Position {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            Instant::now(),
        );
        wait(&mut || unsafe { IsWindowVisible(hwnd) != 0 });
        consumer
            .lock()
            .unwrap()
            .message([0x33, 0x36], &[11, 0, 0, 0, 0, 0], Instant::now());
        wait(&mut || unsafe { IsWindowVisible(hwnd) == 0 });
        stop.store(true, Ordering::Release);
        worker.join().unwrap().unwrap();
        preview_worker.join().unwrap().unwrap();
    }
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
