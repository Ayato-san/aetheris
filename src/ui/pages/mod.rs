pub(crate) mod engine_ini;
pub(crate) mod home;

#[cfg(debug_assertions)]
pub(crate) mod combat_debug;

#[cfg(debug_assertions)]
pub(crate) mod gallery;

pub(crate) mod combat_history;

// The orientation overlay is temporarily withdrawn until passive tracking is
// validated. Keep overlay_editor.rs unregistered so saved activation cannot
// start its capture or native window during application startup.
