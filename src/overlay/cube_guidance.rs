//! Private presentation derived exclusively from fresh, attributed positions.
use super::hidden_cubes::{Guidance, Tracking};
use std::time::Instant;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Panel {
    Target(Guidance),
    AwaitingPosition,
    AwaitingIdentity,
    ExpiredPosition { age_secs: u64 },
}
impl Panel {
    #[cfg(windows)]
    pub fn preview() -> Self {
        Self::Target(Guidance {
            entity: 0,
            number: 1,
            horizontal_m: 5.,
            elevation_m: 12.,
            offset_m: [3., -4.],
        })
    }
    pub fn text(&self) -> String {
        match self {
            Self::AwaitingPosition => {
                "Aucune position du personnage reçue.\nEn attente d’une mise à jour.".into()
            }
            Self::AwaitingIdentity => {
                "Personnage non identifié.\nReconnecte ton personnage avec Aetheris ouvert.".into()
            }
            Self::ExpiredPosition { age_secs } => format!(
                "Dernière position reçue il y a {age_secs} s.\nEn attente d’une mise à jour."
            ),
            Self::Target(target) => format!(
                "Cube {}\nDistance horizontale {:.1} m\n{}",
                target.number,
                target.horizontal_m,
                elevation(target.elevation_m)
            ),
        }
    }
    /// World-space axes, never a camera-relative or movement-relative heading.
    /// At the same XY there is no arbitrary horizontal direction.
    pub fn world_direction(&self) -> Option<[f64; 2]> {
        let Self::Target(target) = self else {
            return None;
        };
        (target.horizontal_m >= 0.01).then(|| {
            [
                target.offset_m[0] / target.horizontal_m,
                target.offset_m[1] / target.horizontal_m,
            ]
        })
    }
}
fn elevation(meters: f64) -> String {
    if meters >= 0.8 {
        format!("Au-dessus de {:.1} m", meters)
    } else if meters <= -0.8 {
        format!("En dessous de {:.1} m", -meters)
    } else {
        "Même hauteur (±0,8 m)".into()
    }
}
pub(super) fn panel(
    tracking: &mut Tracking,
    enabled: bool,
    identified: bool,
    now: Instant,
) -> Option<Panel> {
    if !enabled || tracking.entities().next().is_none() {
        return None;
    }
    Some(
        tracking
            .guidance(now)
            .map(Panel::Target)
            .unwrap_or_else(|| {
                tracking
                    .position_age(now)
                    .map(|age| Panel::ExpiredPosition {
                        age_secs: age.as_secs(),
                    })
                    .unwrap_or(if identified {
                        Panel::AwaitingPosition
                    } else {
                        Panel::AwaitingIdentity
                    })
            }),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay::hidden_cubes::{POSITION_TTL, Position};
    #[test]
    fn panel_tracks_height_and_drops_expired_target_without_removing_presence() {
        let now = Instant::now();
        let mut tracking = Tracking::default();
        tracking.detect(
            7,
            1200039,
            Position {
                x: 300.,
                y: -400.,
                z: 1200.,
            },
            now,
        );
        assert_eq!(panel(&mut tracking, false, true, now), None);
        assert_eq!(
            panel(&mut tracking, true, true, now),
            Some(Panel::AwaitingPosition)
        );
        assert_eq!(
            panel(&mut tracking, true, false, now),
            Some(Panel::AwaitingIdentity)
        );
        tracking.position(
            Position {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            now,
        );
        let view = panel(&mut tracking, true, true, now).unwrap();
        assert_eq!(view.world_direction(), Some([0.6, -0.8]));
        assert!(view.text().contains("5.0 m"));
        assert!(view.text().contains("Au-dessus de 12.0 m"));
        assert_eq!(
            panel(&mut tracking, true, true, now + POSITION_TTL),
            Some(Panel::ExpiredPosition {
                age_secs: POSITION_TTL.as_secs()
            })
        );
        assert!(tracking.presence(now + POSITION_TTL).is_some());
        tracking.remove(7);
        assert_eq!(panel(&mut tracking, true, true, now), None);
    }
    #[test]
    fn vertical_targets_have_height_but_no_invented_horizontal_direction() {
        let now = Instant::now();
        let mut tracking = Tracking::default();
        tracking.detect(
            7,
            1200039,
            Position {
                x: 0.,
                y: 0.,
                z: -1200.,
            },
            now,
        );
        tracking.position(
            Position {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            now,
        );
        let view = panel(&mut tracking, true, true, now).unwrap();
        assert_eq!(view.world_direction(), None);
        assert!(view.text().contains("En dessous de 12.0 m"));
    }
}
