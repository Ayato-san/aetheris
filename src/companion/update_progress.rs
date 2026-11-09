//! Native update feedback stays usable while the GPUI application is stopped.

pub(super) const TITLE: &str = "Mise à jour d’Aetheris";

pub(super) struct Progress {
    #[cfg(windows)]
    state: std::sync::Arc<State>,
    #[cfg(windows)]
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Progress {
    pub fn start() -> super::Result<Self> {
        #[cfg(windows)]
        {
            use std::sync::{Arc, Mutex, atomic::AtomicBool, mpsc};
            let state = Arc::new(State {
                message: Mutex::new("Fermeture de l’application…".into()),
                finished: AtomicBool::new(false),
            });
            let dialog_state = state.clone();
            let (ready, receiver) = mpsc::sync_channel(1);
            let thread = std::thread::spawn(move || show(dialog_state, ready));
            receiver
                .recv()
                .map_err(|_| super::error("La fenêtre de mise à jour s’est arrêtée."))??;
            Ok(Self {
                state,
                thread: Some(thread),
            })
        }
        #[cfg(not(windows))]
        Ok(Self {})
    }

    pub fn set(&self, message: &str) {
        #[cfg(windows)]
        {
            *self.state.message.lock().unwrap() = message.into();
        }
        #[cfg(not(windows))]
        eprintln!("{TITLE} : {message}");
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            self.state
                .finished
                .store(true, std::sync::atomic::Ordering::Release);
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

#[cfg(windows)]
struct State {
    message: std::sync::Mutex<String>,
    finished: std::sync::atomic::AtomicBool,
}

#[cfg(windows)]
fn show(state: std::sync::Arc<State>, ready: std::sync::mpsc::SyncSender<super::Result<()>>) {
    use windows_sys::Win32::UI::Controls::{
        TASKDIALOGCONFIG, TDF_CALLBACK_TIMER, TDF_SHOW_MARQUEE_PROGRESS_BAR, TaskDialogIndirect,
    };
    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let title = wide(TITLE);
    let instruction = wide("Mise à jour en cours…");
    let content = wide("Fermeture de l’application…");
    let footer =
        wide("Aetheris redémarrera automatiquement. Patientez sans éteindre votre ordinateur.");
    let callback = Callback {
        state,
        ready: std::sync::Mutex::new(Some(ready)),
    };
    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        dwFlags: TDF_CALLBACK_TIMER | TDF_SHOW_MARQUEE_PROGRESS_BAR,
        pszWindowTitle: title.as_ptr(),
        pszMainInstruction: instruction.as_ptr(),
        pszContent: content.as_ptr(),
        pszFooter: footer.as_ptr(),
        pfCallback: Some(dialog_callback),
        lpCallbackData: &callback as *const Callback as isize,
        ..Default::default()
    };
    // The buffers and callback data are owned by this thread until the dialog exits.
    let result = unsafe {
        TaskDialogIndirect(
            &config,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if let Some(ready) = callback.ready.lock().unwrap().take() {
        let _ = ready.send(Err(super::error(format!(
            "Impossible d’afficher la progression (0x{:08X}).",
            result as u32
        ))));
    }
}

#[cfg(windows)]
struct Callback {
    state: std::sync::Arc<State>,
    ready: std::sync::Mutex<Option<std::sync::mpsc::SyncSender<super::Result<()>>>>,
}

#[cfg(windows)]
unsafe extern "system" fn dialog_callback(
    window: windows_sys::Win32::Foundation::HWND,
    message: u32,
    _button: usize,
    _parameter: isize,
    data: isize,
) -> windows_sys::core::HRESULT {
    use std::sync::atomic::Ordering;
    use windows_sys::Win32::UI::{
        Controls::{
            TDE_CONTENT, TDM_CLICK_BUTTON, TDM_ENABLE_BUTTON, TDM_SET_ELEMENT_TEXT,
            TDM_SET_PROGRESS_BAR_MARQUEE, TDN_BUTTON_CLICKED, TDN_CREATED, TDN_TIMER,
        },
        WindowsAndMessaging::{IDOK, SendMessageW},
    };
    let callback = unsafe { &*(data as *const Callback) };
    if message == TDN_CREATED as u32 {
        unsafe {
            SendMessageW(window, TDM_ENABLE_BUTTON as u32, IDOK as usize, 0);
            SendMessageW(window, TDM_SET_PROGRESS_BAR_MARQUEE as u32, 1, 30);
        }
        if let Some(ready) = callback.ready.lock().unwrap().take() {
            let _ = ready.send(Ok(()));
        }
    } else if message == TDN_TIMER as u32 {
        if callback.state.finished.load(Ordering::Acquire) {
            unsafe {
                SendMessageW(window, TDM_ENABLE_BUTTON as u32, IDOK as usize, 1);
                SendMessageW(window, TDM_CLICK_BUTTON as u32, IDOK as usize, 0);
            }
        } else {
            let text = callback
                .state
                .message
                .lock()
                .unwrap()
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();
            unsafe {
                SendMessageW(
                    window,
                    TDM_SET_ELEMENT_TEXT as u32,
                    TDE_CONTENT as usize,
                    text.as_ptr() as isize,
                );
            }
        }
    } else if message == TDN_BUTTON_CLICKED as u32
        && !callback.state.finished.load(Ordering::Acquire)
    {
        return 1; // S_FALSE prevents closing the dialog in the middle of replacement.
    }
    0
}
