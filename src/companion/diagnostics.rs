//! Failure reporting must stay available without initializing GPUI.
use std::path::Path;

/// Opening a log keeps this dialog available and never restarts the application.
pub(super) fn show(message: &str, log: &Path, action: &str) -> bool {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::Controls::{
            TASKDIALOG_BUTTON, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0, TD_ERROR_ICON,
            TDF_ALLOW_DIALOG_CANCELLATION, TaskDialogIndirect,
        };
        const OPEN_LOG: i32 = 100;
        const ACTION: i32 = 101;
        const CLOSE: i32 = 102;
        let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let title = wide("Aetheris");
        let instruction = wide("Aetheris n’a pas pu continuer");
        let content = wide(message);
        let open_label = wide("Ouvrir le journal");
        let action_label = wide(action);
        let close_label = wide("Fermer");
        let buttons = [
            TASKDIALOG_BUTTON {
                nButtonID: OPEN_LOG,
                pszButtonText: open_label.as_ptr(),
            },
            TASKDIALOG_BUTTON {
                nButtonID: ACTION,
                pszButtonText: action_label.as_ptr(),
            },
            TASKDIALOG_BUTTON {
                nButtonID: CLOSE,
                pszButtonText: close_label.as_ptr(),
            },
        ];
        let log = log.to_path_buf();
        let config = TASKDIALOGCONFIG {
            cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
            dwFlags: TDF_ALLOW_DIALOG_CANCELLATION,
            pszWindowTitle: title.as_ptr(),
            pszMainInstruction: instruction.as_ptr(),
            pszContent: content.as_ptr(),
            Anonymous1: TASKDIALOGCONFIG_0 {
                pszMainIcon: TD_ERROR_ICON,
            },
            cButtons: buttons.len() as u32,
            pButtons: buttons.as_ptr(),
            nDefaultButton: CLOSE,
            pfCallback: Some(dialog_callback),
            lpCallbackData: &log as *const std::path::PathBuf as isize,
            ..Default::default()
        };
        let mut selected = CLOSE;
        // Configuration buffers and callback data live until the modal dialog returns.
        let result = unsafe {
            TaskDialogIndirect(
                &config,
                &mut selected,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if result < 0 {
            super::runtime::report(&format!(
                "{message}\nImpossible d’ouvrir le diagnostic Windows (0x{:08X}).",
                result as u32
            ));
            return false;
        }
        selected == ACTION
    }
    #[cfg(not(windows))]
    {
        eprintln!("{message}");
        loop {
            eprintln!("[j] Ouvrir le journal · [r] {action} · [Entrée] Fermer");
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                return false;
            }
            match input.trim() {
                "j" => {
                    if let Err(cause) = open_log(log) {
                        eprintln!("Impossible d’ouvrir le journal : {cause}");
                    }
                }
                "r" => return true,
                _ => return false,
            }
        }
    }
}

fn open_log(path: &Path) -> super::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};
        let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        // Use the registered file association and pass the path as a file, with
        // no shell command or interpolated arguments (including for spaces/Unicode).
        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                std::ptr::null(),
                path.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        } as isize;
        if result <= 32 {
            return Err(super::error(format!(
                "Windows a refusé l’ouverture (code {result})."
            )));
        }
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

#[cfg(windows)]
unsafe extern "system" fn dialog_callback(
    _window: windows_sys::Win32::Foundation::HWND,
    message: u32,
    button: usize,
    _parameter: isize,
    data: isize,
) -> windows_sys::core::HRESULT {
    if message == windows_sys::Win32::UI::Controls::TDN_BUTTON_CLICKED as u32 && button == 100 {
        // TaskDialogIndirect is synchronous; show() owns this PathBuf until return.
        let log = unsafe { &*(data as *const std::path::PathBuf) };
        if let Err(cause) = open_log(log) {
            super::runtime::report(&format!(
                "Impossible d’ouvrir le journal : {cause}\n{}",
                log.display()
            ));
        }
        return 1; // S_FALSE keeps the diagnostic open while the editor is opened.
    }
    0 // S_OK lets restart, close, Escape and the window close button finish normally.
}
