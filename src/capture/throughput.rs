//! Process-attributed captured bytes per second, including non-selected flows.
use super::{
    observations::{Direction, Event, Reliability},
    widgets::CaptureClock,
};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

pub const DEFAULT_WINDOW: Duration = Duration::from_secs(10);
pub(crate) const MAX_WINDOW: Duration = Duration::from_secs(15 * 60);
const MAX_TIMESTAMPS: usize = 262_144;

#[derive(Clone, Debug, PartialEq)]
pub struct Rates {
    pub incoming_bytes_per_second: f64,
    pub outgoing_bytes_per_second: f64,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub rates: Option<Rates>,
    pub recovering: bool,
}

pub struct Throughput {
    window: Duration,
    clock: CaptureClock,
    available: bool,
    period_started: Option<Instant>,
    capture_floor: Option<Instant>,
    recovery_required: bool,
    /// Attribution may deliver old packets after newer ones from other flows.
    /// Ordered timestamps keep expiration exact without mistaking that for loss.
    bytes: BTreeMap<Instant, [u64; 2]>,
    totals: [u64; 2],
}
impl Default for Throughput {
    fn default() -> Self {
        Self {
            window: DEFAULT_WINDOW,
            clock: CaptureClock::default(),
            available: false,
            period_started: None,
            capture_floor: None,
            recovery_required: false,
            bytes: BTreeMap::new(),
            totals: [0; 2],
        }
    }
}
impl Throughput {
    pub fn window(&self) -> Duration {
        self.window
    }
    pub fn set_window(&mut self, window: Duration, now: Instant) -> Result<(), &'static str> {
        if window.is_zero() || window > MAX_WINDOW {
            return Err("La fenêtre de débit doit être comprise entre 1 ns et 15 minutes.");
        }
        if self.window != window {
            self.advance_recovery(now);
            self.window = window;
            self.clear();
            self.period_started = self.available.then_some(now);
            self.capture_floor = Some(now);
        }
        Ok(())
    }
    pub fn observe(&mut self, event: &Event, now: Instant) {
        self.advance_recovery(now);
        match event {
            Event::CaptureState { available } if *available != self.available => {
                self.clear();
                self.available = *available;
                self.period_started = available.then_some(now);
                if !available {
                    self.recovery_required = true;
                }
                self.capture_floor = self.recovery_required.then_some(now);
            }
            // Candidate capacity only prevents choosing a game connection;
            // Traffic was already delivered and its byte coverage is intact.
            Event::Reliability {
                issue: Reliability::CapacityExceeded,
                ..
            } => {}
            Event::Reliability { .. } => self.invalidate(now),
            Event::Traffic {
                direction,
                captured_bytes,
                timestamp_ns,
            } if self.available => {
                self.traffic(*direction, *captured_bytes, *timestamp_ns, now);
            }
            _ => {}
        }
    }
    fn clear(&mut self) {
        self.bytes.clear();
        self.totals = [0; 2];
    }
    fn invalidate(&mut self, now: Instant) {
        self.clear();
        self.recovery_required = true;
        self.period_started = self.available.then_some(now);
        self.capture_floor = Some(now);
    }
    fn advance_recovery(&mut self, now: Instant) {
        if self.available
            && self
                .period_started
                .is_some_and(|start| now.saturating_duration_since(start) >= self.window)
        {
            self.recovery_required = false;
        }
    }
    fn traffic(&mut self, direction: Direction, count: u32, timestamp: Option<i128>, now: Instant) {
        let Some(timestamp) = timestamp.filter(|time| *time >= 0) else {
            self.invalidate(now);
            return;
        };
        let Some(received) = self.clock.received(Some(timestamp), now) else {
            self.invalidate(now);
            return;
        };
        if self.capture_floor.is_some_and(|floor| received < floor) {
            return;
        }
        self.expire(now);
        if now.saturating_duration_since(received) >= self.window {
            return;
        }
        // At normal startup the first batch can contain packets captured just
        // before the availability event was delivered. Include that partial span.
        if !self.recovery_required
            && self.capture_floor.is_none()
            && self.period_started.is_some_and(|start| received < start)
        {
            self.period_started = Some(received);
        }
        let side = match direction {
            Direction::Incoming => 0,
            Direction::Outgoing => 1,
        };
        if !self.bytes.contains_key(&received) && self.bytes.len() == MAX_TIMESTAMPS {
            self.invalidate(now);
            return;
        }
        let Some(total) = self.totals[side].checked_add(u64::from(count)) else {
            self.invalidate(now);
            return;
        };
        self.bytes.entry(received).or_default()[side] += u64::from(count);
        self.totals[side] = total;
    }
    fn expire(&mut self, now: Instant) {
        while self
            .bytes
            .first_key_value()
            .is_some_and(|(received, _)| now.saturating_duration_since(*received) >= self.window)
        {
            let (_, counts) = self.bytes.pop_first().unwrap();
            for (total, count) in self.totals.iter_mut().zip(counts) {
                *total -= count;
            }
        }
    }
    pub fn snapshot(&mut self, now: Instant) -> Snapshot {
        self.expire(now);
        self.advance_recovery(now);
        let rates = self
            .period_started
            .filter(|_| self.available && !self.recovery_required)
            .and_then(|start| {
                let duration = now
                    .saturating_duration_since(start)
                    .min(self.window)
                    .as_secs_f64();
                if duration == 0.0 && self.totals != [0; 2] {
                    return None;
                }
                Some(Rates {
                    incoming_bytes_per_second: if duration == 0.0 {
                        0.0
                    } else {
                        self.totals[0] as f64 / duration
                    },
                    outgoing_bytes_per_second: if duration == 0.0 {
                        0.0
                    } else {
                        self.totals[1] as f64 / duration
                    },
                })
            });
        Snapshot {
            rates,
            recovering: self.recovery_required,
        }
    }
}

#[cfg(test)]
mod tests;
