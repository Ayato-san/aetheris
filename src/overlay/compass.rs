use std::time::{Duration, Instant};

pub const ORIENTATION_UNAVAILABLE: &str = "orientation indisponible";
pub const STALE_AFTER: Duration = Duration::from_secs(3);

/// Only the future validated decoder may supply a local character heading.
#[derive(Default)]
pub struct Orientation {
    sample: Option<(f32, Instant)>,
}
pub struct CompassTick {
    /// Degrees relative to the character, in the visible -90..=90 range.
    pub offset: f32,
    pub label: &'static str,
}
impl Orientation {
    pub fn update(&mut self, degrees: f32, received: Instant) {
        if degrees.is_finite() {
            self.sample = Some((degrees.rem_euclid(360.), received));
        }
    }
    pub fn heading(&self, now: Instant) -> Option<f32> {
        self.sample
            .filter(|(_, time)| {
                now.checked_duration_since(*time)
                    .is_some_and(|age| age < STALE_AFTER)
            })
            .map(|(angle, _)| angle)
    }
    pub fn ticks(&self, now: Instant) -> Option<Vec<CompassTick>> {
        let heading = self.heading(now)?;
        let center = (heading / 15.).floor() as i32;
        Some(
            (center - 6..=center + 6)
                .filter_map(|step| {
                    let offset = step as f32 * 15. - heading;
                    if offset.abs() > 90. {
                        return None;
                    }
                    let label = match (step * 15).rem_euclid(360) {
                        0 => "N",
                        45 => "NE",
                        90 => "E",
                        135 => "SE",
                        180 => "S",
                        225 => "SO",
                        270 => "O",
                        315 => "NO",
                        _ => "",
                    };
                    Some(CompassTick { offset, label })
                })
                .collect(),
        )
    }
}
