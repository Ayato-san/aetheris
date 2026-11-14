//! Empirical capture qualification is separate from per-window affinity checks.
//! An unlisted Windows build or running OBS version never inherits a validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Configuration {
    windows: [u32; 2], // build and update revision
    obs: [u16; 4],
    captioned_styles: &'static [u32],
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
    // The user confirmed the same borderless game configuration on 2026-11-14
    // and reported this native style through the protected fictional probe.
    // WS_CAPTION alone does not describe the game's actual presentation mode.
    captioned_styles: &[0x14cf0000],
}];

fn matches(
    environment: &Environment,
    captionless: bool,
    style: u32,
    profiles: &[Configuration],
) -> bool {
    profiles.iter().any(|profile| {
        profile.windows == environment.windows
            && (captionless || profile.captioned_styles.contains(&style))
            && environment.obs.iter().all(|obs| profile.obs == *obs)
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
    let (style, captionless) = foreground_style();
    observed_environment()
        .is_some_and(|environment| matches(&environment, captionless, style, VALIDATED))
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
    let (style, captionless) = foreground_style();
    let Some(environment) = observed_environment() else {
        return format!("TEST · environnement Windows/OBS illisible\nStyle {style:08x}");
    };
    let known_style = captionless
        || VALIDATED.iter().any(|profile| {
            profile.windows == environment.windows && profile.captioned_styles.contains(&style)
        });
    let mode = if known_style {
        "style de jeu reconnu"
    } else {
        "style de fenêtre non qualifié"
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
    let qualified = matches(&environment, captionless, style, VALIDATED);
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
        captioned_styles: &[],
    };
    #[test]
    fn reported_game_style_belongs_only_to_the_user_tested_environment() {
        let mut environment = Environment {
            windows: [26200, 9711],
            obs: vec![],
        };
        assert!(matches(&environment, false, 0x14cf0000, VALIDATED));
        environment.obs = vec![[33, 0, 1, 0]];
        assert!(matches(&environment, false, 0x14cf0000, VALIDATED));
        assert!(!matches(&environment, false, 0x14cf0001, VALIDATED));
        assert!(!matches(&environment, false, 0, VALIDATED));
        environment.obs.push([30, 2, 3, 0]);
        assert!(!matches(&environment, false, 0x14cf0000, VALIDATED));
        environment.obs.clear();
        environment.windows[1] += 1;
        assert!(!matches(&environment, false, 0x14cf0000, VALIDATED));
    }
    #[test]
    fn user_tested_profile_enables_guidance_without_qualifying_the_build_host_obs() {
        let mut environment = Environment {
            windows: [26200, 9711],
            obs: vec![[33, 0, 1, 0]],
        };
        assert!(matches(&environment, true, 0, VALIDATED));
        assert!(!matches(&environment, false, 0, VALIDATED));
        environment.obs = vec![[30, 2, 3, 0]];
        assert!(!matches(&environment, true, 0, VALIDATED));
        environment.obs = vec![[33, 0, 1, 0]];
        environment.windows[1] += 1;
        assert!(!matches(&environment, true, 0, VALIDATED));
    }
    #[test]
    fn captures_only_inherit_validation_from_the_exact_tested_environment() {
        let environment = Environment {
            windows: PROFILE.windows,
            obs: vec![PROFILE.obs],
        };
        assert!(matches(&environment, true, 0, &[PROFILE]));
        assert!(!matches(&environment, false, 0, &[PROFILE]));
        assert!(!matches(&environment, true, 0, &[]));
        assert!(!matches(
            &Environment {
                windows: [26000, 1235],
                obs: vec![PROFILE.obs]
            },
            true,
            0,
            &[PROFILE]
        ));
        assert!(!matches(
            &Environment {
                windows: PROFILE.windows,
                obs: vec![[31, 0, 0, 0]]
            },
            true,
            0,
            &[PROFILE]
        ));
        assert!(!matches(
            &Environment {
                windows: PROFILE.windows,
                obs: vec![PROFILE.obs, [31, 0, 0, 0]]
            },
            true,
            0,
            &[PROFILE]
        ));
        assert!(matches(
            &Environment {
                windows: PROFILE.windows,
                obs: vec![]
            },
            true,
            0,
            &[PROFILE]
        ));
    }
}
