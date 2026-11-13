//! Continuous local detection, independent of rendering and wall-clock time.
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

pub const POSITION_TTL: Duration = Duration::from_secs(4);
const MAX_CUBES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Position {
    pub fn valid(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.z.is_finite()
            && self.x.abs() <= 500_000.
            && self.y.abs() <= 500_000.
            && self.z.abs() <= 120_000.
    }
    fn distance_squared(self, other: Self) -> f64 {
        let dx = f64::from(self.x) - f64::from(other.x);
        let dy = f64::from(self.y) - f64::from(other.y);
        let dz = f64::from(self.z) - f64::from(other.z);
        dx * dx + dy * dy + dz * dz
    }
}
#[derive(Clone, Debug)]
pub struct Cube {
    pub entity: u32,
    pub kind: u32,
    pub position: Position,
    detected: Instant,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Guidance {
    pub entity: u32,
    pub horizontal_m: f64,
    pub elevation_m: f64,
}
#[derive(Default)]
pub struct Tracking {
    cubes: BTreeMap<u32, Cube>,
    player: Option<(Position, Instant)>,
    target: Option<u32>,
    interruption: Option<String>,
}
impl Tracking {
    /// True only for the first announcement of this continuous instance.
    pub fn detect(&mut self, entity: u32, kind: u32, position: Position, now: Instant) -> bool {
        if entity == 0 {
            return false;
        }
        if !is_cube(kind) {
            self.remove(entity);
            return false;
        }
        if !position.valid() {
            return false;
        }
        if let Some(cube) = self.cubes.get_mut(&entity) {
            if cube.kind == kind {
                cube.position = position;
                return false;
            }
            // Reused IDs or changed object identity start a new detection.
            self.remove(entity);
        }
        if self.cubes.len() >= MAX_CUBES {
            self.interrupt("Limite de suivi des cubes atteinte");
            return false;
        }
        self.cubes.insert(
            entity,
            Cube {
                entity,
                kind,
                position,
                detected: now,
            },
        );
        true
    }
    pub fn remove(&mut self, entity: u32) {
        self.cubes.remove(&entity);
        if self.target == Some(entity) {
            self.target = None;
        }
    }
    pub fn position(&mut self, position: Position, now: Instant) {
        if position.valid() {
            self.player = Some((position, now));
        } else {
            self.player = None;
            self.target = None;
        }
    }
    pub fn reset(&mut self) {
        self.cubes.clear();
        self.player = None;
        self.target = None;
    }
    pub fn interrupt(&mut self, reason: impl Into<String>) {
        self.reset();
        self.interruption = Some(reason.into());
    }
    pub fn connected(&mut self) {
        self.interruption = None;
    }
    pub fn entities(&self) -> impl Iterator<Item = u32> + '_ {
        self.cubes.keys().copied()
    }
    /// This capturable text intentionally contains no position or guidance status.
    pub fn presence(&self, now: Instant) -> Option<String> {
        if let Some(error) = &self.interruption {
            return Some(format!("Capture indisponible : {error}"));
        }
        if self.cubes.is_empty() {
            return None;
        }
        let durations = self
            .cubes
            .values()
            .enumerate()
            .map(|(index, cube)| {
                let age = now.saturating_duration_since(cube.detected).as_secs();
                format!(
                    "Cube {} · détection continue {} min {:02} s",
                    index + 1,
                    age / 60,
                    age % 60
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        Some(format!(
            "{} cube(s) détecté(s)\n{durations}",
            self.cubes.len()
        ))
    }
    pub fn guidance(&mut self, now: Instant) -> Option<Guidance> {
        let Some((player, _received)) = self.player.filter(|(_, received)| {
            now.checked_duration_since(*received)
                .is_some_and(|age| age < POSITION_TTL)
        }) else {
            self.target = None;
            return None;
        };
        let nearest = self.cubes.values().min_by(|a, b| {
            player
                .distance_squared(a.position)
                .total_cmp(&player.distance_squared(b.position))
        })?;
        let target = self
            .target
            .and_then(|id| self.cubes.get(&id))
            .filter(|current| {
                // Switch only when the challenger is at least 10% and 2 m closer.
                let current_distance = player.distance_squared(current.position).sqrt();
                let nearest_distance = player.distance_squared(nearest.position).sqrt();
                current_distance - nearest_distance < 200.
                    || nearest_distance >= current_distance * 0.9
            })
            .unwrap_or(nearest);
        self.target = Some(target.entity);
        let dx = f64::from(target.position.x) - f64::from(player.x);
        let dy = f64::from(target.position.y) - f64::from(player.y);
        Some(Guidance {
            entity: target.entity,
            horizontal_m: dx.hypot(dy) / 100.,
            elevation_m: (f64::from(target.position.z) - f64::from(player.z)) / 100.,
        })
    }
}
pub fn is_cube(kind: u32) -> bool {
    matches!(kind, 1000295 | 1000548..=1000566 | 1200039..=1200041 | 1300081..=1300082 | 1300754 | 1301199 | 1301200)
}

#[cfg(test)]
mod tests {
    use super::*;
    const ORIGIN: Position = Position {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    #[test]
    fn continuous_detection_removal_and_resets() {
        let now = Instant::now();
        let mut state = Tracking::default();
        assert_eq!(state.presence(now), None);
        assert!(state.detect(1, 1200039, ORIGIN, now));
        assert!(!state.detect(1, 1200039, ORIGIN, now + Duration::from_secs(10)));
        assert!(state.detect(2, 1200039, ORIGIN, now));
        assert!(
            state
                .presence(now + Duration::from_secs(70))
                .unwrap()
                .contains("1 min 10 s")
        );
        state.remove(1);
        assert!(state.detect(1, 1200039, ORIGIN, now + Duration::from_secs(70)));
        assert!(
            state
                .presence(now + Duration::from_secs(70))
                .unwrap()
                .contains("0 min 00 s")
        );
        state.interrupt("connexion perdue");
        assert_eq!(state.entities().count(), 0);
        assert!(state.presence(now).unwrap().contains("connexion perdue"));
        state.connected();
        assert_eq!(state.presence(now), None);
    }
    #[test]
    fn missing_and_expired_positions_never_remove_presence() {
        let now = Instant::now();
        let mut state = Tracking::default();
        state.detect(
            1,
            1200039,
            Position {
                x: 300.,
                y: 400.,
                z: 1200.,
            },
            now,
        );
        assert_eq!(state.guidance(now), None);
        state.position(ORIGIN, now);
        assert_eq!(
            state.guidance(now),
            Some(Guidance {
                entity: 1,
                horizontal_m: 5.,
                elevation_m: 12.
            })
        );
        assert_eq!(state.guidance(now + POSITION_TTL), None);
        assert!(state.presence(now + POSITION_TTL).is_some());
        state.position(
            Position {
                x: f32::NAN,
                ..ORIGIN
            },
            now,
        );
        assert_eq!(state.guidance(now), None);
        state.reset();
        assert_eq!(state.presence(now), None);
    }
    #[test]
    fn reused_entity_types_and_tracking_limits_release_the_old_instance() {
        let now = Instant::now();
        let mut state = Tracking::default();
        state.detect(1, 1200039, ORIGIN, now);
        assert!(!state.detect(1, 42, ORIGIN, now));
        assert_eq!(state.entities().count(), 0);
        for id in 1..=MAX_CUBES as u32 {
            assert!(state.detect(id, 1200039, ORIGIN, now));
        }
        assert!(!state.detect(MAX_CUBES as u32 + 1, 1200039, ORIGIN, now));
        assert_eq!(state.entities().count(), 0);
        assert!(state.presence(now).unwrap().contains("Limite de suivi"));
    }
    #[test]
    fn target_hysteresis_and_invalid_input() {
        let now = Instant::now();
        let mut state = Tracking::default();
        assert!(!state.detect(1, 42, ORIGIN, now));
        assert!(!state.detect(
            1,
            1200039,
            Position {
                x: f32::INFINITY,
                ..ORIGIN
            },
            now
        ));
        state.detect(1, 1200039, Position { x: 1000., ..ORIGIN }, now);
        state.detect(
            2,
            1200039,
            Position {
                x: -1000.,
                ..ORIGIN
            },
            now,
        );
        state.position(ORIGIN, now);
        assert_eq!(state.guidance(now).unwrap().entity, 1);
        state.position(Position { x: -10., ..ORIGIN }, now);
        assert_eq!(state.guidance(now).unwrap().entity, 1);
        state.position(Position { x: -500., ..ORIGIN }, now);
        assert_eq!(state.guidance(now).unwrap().entity, 2);
        state.remove(2);
        assert_eq!(state.guidance(now).unwrap().entity, 1);
    }
}
