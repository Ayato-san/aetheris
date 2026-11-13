//! Shared, optional widget sound. No entity, distance or direction is encoded.
pub enum Notification {
    Detected,
}
pub fn play(notification: Notification) {
    match notification {
        Notification::Detected => {
            #[cfg(windows)]
            unsafe {
                windows_sys::Win32::System::Diagnostics::Debug::MessageBeep(0);
            }
        }
    }
}
