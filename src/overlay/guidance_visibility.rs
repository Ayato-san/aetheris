//! Empirical capture qualification is separate from per-window affinity checks.
//! An unlisted Windows build or running OBS version never inherits a validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Configuration {
    windows: [u32; 2], // build and update revision
    obs: [u16; 4],
}
#[derive(Clone)]
struct Environment {
    windows: [u32; 2],
    obs: Vec<[u16; 4]>,
}
// Populate only with the exact environment of a completed screenshot + OBS
// display/window/game capture probe. See docs/overlay.md for its evidence.
#[cfg(any(windows, test))]
const VALIDATED: &[Configuration] = &[Configuration {
    windows: [26200, 9711], // Windows 11 25H2; user capture tests, 2026-11-13
    obs: [33, 0, 1, 0],
}];

fn matches(environment: &Environment, borderless: bool, profiles: &[Configuration]) -> bool {
    borderless
        && profiles
            .iter()
            .any(|profile| profile.windows == environment.windows)
        && environment.obs.iter().all(|obs| {
            profiles
                .iter()
                .any(|profile| profile.windows == environment.windows && profile.obs == *obs)
        })
}
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(super) fn may_show(validated_configuration: bool, affinity: bool, composition: bool) -> bool {
    validated_configuration && affinity && composition
}
#[cfg(windows)]
pub(super) fn configuration_validated() -> bool {
    let (_, borderless) = foreground_style();
    borderless
        && observed_environment()
            .is_some_and(|environment| matches(&environment, borderless, VALIDATED))
}
#[cfg(windows)]
fn foreground_style() -> (u32, bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_STYLE, GetForegroundWindow, GetWindowLongPtrW, WS_CAPTION,
    };
    let foreground = unsafe { GetForegroundWindow() };
    let style = unsafe { GetWindowLongPtrW(foreground, GWL_STYLE) as u32 };
    (style, !foreground.is_null() && style & WS_CAPTION == 0)
}
// Only the fictional, capture-excluded probe may present qualification details.
// This does not read a cube, a player position or the real guidance state.
#[cfg(windows)]
pub(super) fn probe_report() -> String {
    let (style, borderless) = foreground_style();
    let mode = if borderless {
        "sans bordure reconnu"
    } else {
        "style de fenêtre non qualifié"
    };
    let Some(environment) = observed_environment() else {
        return format!("TEST · environnement Windows/OBS illisible\n{mode} · style {style:08x}");
    };
    let obs = if environment.obs.is_empty() {
        "arrêté".into()
    } else {
        environment
            .obs
            .iter()
            .map(|version| {
                format!(
                    "{}.{}.{}.{}",
                    version[0], version[1], version[2], version[3]
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let qualified = matches(&environment, borderless, VALIDATED);
    format!(
        "TEST · Windows {}.{} · OBS {obs}\n{mode} · style {style:08x}\nQualification : {}",
        environment.windows[0],
        environment.windows[1],
        if qualified { "validée" } else { "refusée" }
    )
}
#[cfg(windows)]
fn observed_environment() -> Option<Environment> {
    use std::{
        sync::{Mutex, OnceLock},
        time::{Duration, Instant},
    };
    struct Observation {
        checked: Instant,
        environment: Option<Environment>,
    }
    static CURRENT: OnceLock<Mutex<Option<Observation>>> = OnceLock::new();
    let mut current = CURRENT.get_or_init(Default::default).lock().unwrap();
    let now = Instant::now();
    if current.as_ref().is_none_or(|current| {
        now.saturating_duration_since(current.checked) >= Duration::from_millis(250)
    }) {
        *current = Some(Observation {
            checked: now,
            environment: windows::observe(),
        });
    }
    current
        .as_ref()
        .and_then(|current| current.environment.clone())
}
#[cfg(test)]
mod tests {
    use super::*;
    const PROFILE: Configuration = Configuration {
        windows: [26000, 1234],
        obs: [30, 2, 3, 0],
    };
    #[test]
    fn user_tested_profile_enables_guidance_without_qualifying_the_build_host_obs() {
        let mut environment = Environment {
            windows: [26200, 9711],
            obs: vec![[33, 0, 1, 0]],
        };
        assert!(matches(&environment, true, VALIDATED));
        assert!(!matches(&environment, false, VALIDATED));
        environment.obs = vec![[30, 2, 3, 0]];
        assert!(!matches(&environment, true, VALIDATED));
        environment.obs = vec![[33, 0, 1, 0]];
        environment.windows[1] += 1;
        assert!(!matches(&environment, true, VALIDATED));
    }
    #[test]
    fn captures_only_inherit_validation_from_the_exact_tested_environment() {
        let environment = Environment {
            windows: PROFILE.windows,
            obs: vec![PROFILE.obs],
        };
        assert!(matches(&environment, true, &[PROFILE]));
        assert!(!matches(&environment, false, &[PROFILE]));
        assert!(!matches(&environment, true, &[]));
        assert!(!matches(
            &Environment {
                windows: [26000, 1235],
                obs: vec![PROFILE.obs]
            },
            true,
            &[PROFILE]
        ));
        assert!(!matches(
            &Environment {
                windows: PROFILE.windows,
                obs: vec![[31, 0, 0, 0]]
            },
            true,
            &[PROFILE]
        ));
        assert!(!matches(
            &Environment {
                windows: PROFILE.windows,
                obs: vec![PROFILE.obs, [31, 0, 0, 0]]
            },
            true,
            &[PROFILE]
        ));
        assert!(matches(
            &Environment {
                windows: PROFILE.windows,
                obs: vec![]
            },
            true,
            &[PROFILE]
        ));
    }
}
