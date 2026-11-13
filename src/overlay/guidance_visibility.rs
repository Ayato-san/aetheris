//! Empirical capture qualification is separate from per-window affinity checks.
//! An unlisted Windows build or running OBS version never inherits a validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Configuration {
    windows: [u32; 2], // build and update revision
    obs: [u16; 4],
}
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
    use std::{
        sync::{Mutex, OnceLock},
        time::{Duration, Instant},
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_STYLE, GetForegroundWindow, GetWindowLongPtrW, WS_CAPTION,
    };
    let foreground = unsafe { GetForegroundWindow() };
    let borderless = !foreground.is_null()
        && unsafe { GetWindowLongPtrW(foreground, GWL_STYLE) as u32 & WS_CAPTION == 0 };
    if !borderless {
        return false;
    }
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
        .and_then(|current| current.environment.as_ref())
        .is_some_and(|environment| matches(environment, borderless, VALIDATED))
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
