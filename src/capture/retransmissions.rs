//! Observed TCP payload-sequence overlap, not a measurement of path packet loss.
//! Duplicate capture delivery and genuine TCP retransmission are indistinguishable
//! here; a segment counts once as retransmitted if any payload byte was seen before.
use super::{
    observations::{Connection, Direction, Event, Selection, TcpObservation},
    widgets::CaptureClock,
};
use std::{
    collections::{BTreeMap, VecDeque},
    time::{Duration, Instant},
};

pub const DEFAULT_WINDOW: Duration = Duration::from_secs(15 * 60);
pub(crate) const MAX_WINDOW: Duration = Duration::from_secs(60 * 60);
const SNAPSHOT_INTERVAL: Duration = Duration::from_secs(1);
const MAX_INTERVALS: usize = 4096;
const MAX_OBSERVATIONS: usize = 262_144;

#[derive(Clone, Debug, PartialEq)]
pub struct Counts {
    pub retransmissions: u64,
    /// All observed TCP data segments, including retransmitted ones.
    pub data_segments: u64,
    /// None with no data; this is never an exact path-loss percentage.
    pub percentage: Option<f64>,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub selection: Selection,
    /// None without a uniquely selected connection or during recovery. An
    /// available empty window has zero counts and an unavailable percentage.
    pub counts: Option<Counts>,
    pub recovering: bool,
}

#[derive(Clone, Copy)]
struct Reference {
    sequence: u32,
    position: i64,
}
#[derive(Default)]
struct History {
    reference: Option<Reference>,
    /// Disjoint, non-adjacent, half-open payload intervals. Contiguous traffic
    /// occupies one interval regardless of its packet count or lifetime.
    ranges: BTreeMap<i64, i64>,
}
#[derive(Debug, PartialEq, Eq)]
enum ClassificationError {
    AmbiguousSequence,
    Capacity,
}
impl History {
    fn observe(&mut self, sequence: u32, length: u32) -> Result<bool, ClassificationError> {
        if length == 0 || length >= (1 << 31) {
            return Err(ClassificationError::AmbiguousSequence);
        }
        let start = match self.reference {
            None => 0,
            Some(reference) => {
                let delta = sequence.wrapping_sub(reference.sequence) as i32;
                if delta == i32::MIN {
                    return Err(ClassificationError::AmbiguousSequence);
                }
                reference
                    .position
                    .checked_add(i64::from(delta))
                    .ok_or(ClassificationError::AmbiguousSequence)?
            }
        };
        let end = start
            .checked_add(i64::from(length))
            .ok_or(ClassificationError::AmbiguousSequence)?;
        let lower = self
            .ranges
            .range(..=start)
            .next_back()
            .filter(|(_, previous_end)| **previous_end >= start)
            .map_or(start, |(previous_start, _)| *previous_start);
        let mut merged_start = start;
        let mut merged_end = end;
        let mut retransmitted = false;
        let mut remove = Vec::new();
        for (&previous_start, &previous_end) in self.ranges.range(lower..=end) {
            retransmitted |= previous_start < end && previous_end > start;
            merged_start = merged_start.min(previous_start);
            merged_end = merged_end.max(previous_end);
            remove.push(previous_start);
        }
        if self.ranges.len() - remove.len() + 1 > MAX_INTERVALS {
            return Err(ClassificationError::Capacity);
        }
        for key in remove {
            self.ranges.remove(&key);
        }
        self.ranges.insert(merged_start, merged_end);
        if self
            .reference
            .is_none_or(|reference| end > reference.position)
        {
            self.reference = Some(Reference {
                sequence: sequence.wrapping_add(length),
                position: end,
            });
        }
        Ok(retransmitted)
    }
}
struct Observation {
    received: Instant,
    retransmitted: bool,
}

/// Consume the ordered event stream from capture::observations. Histories are
/// independent per direction and outlive the counter window; a retransmission
/// remains recognizable even after its original segment has expired.
pub struct Retransmissions {
    selection: Selection,
    window: Duration,
    clock: CaptureClock,
    histories: [History; 2],
    observations: VecDeque<Observation>,
    awaiting_data: bool,
    data_segments: u64,
    retransmissions: u64,
    last_timestamp: Option<i128>,
    capture_floor: Option<Instant>,
    recovery_required: bool,
    recovery_started: Option<Instant>,
    last_snapshot: Option<Instant>,
}
impl Default for Retransmissions {
    fn default() -> Self {
        Self {
            selection: Selection::Absent,
            window: DEFAULT_WINDOW,
            clock: CaptureClock::default(),
            histories: Default::default(),
            observations: VecDeque::new(),
            awaiting_data: false,
            data_segments: 0,
            retransmissions: 0,
            last_timestamp: None,
            capture_floor: None,
            recovery_required: false,
            recovery_started: None,
            last_snapshot: None,
        }
    }
}
impl Retransmissions {
    pub fn window(&self) -> Duration {
        self.window
    }
    pub fn set_window(&mut self, window: Duration, now: Instant) -> Result<(), &'static str> {
        if window.is_zero() || window > MAX_WINDOW {
            return Err(
                "La fenêtre de retransmissions doit être comprise entre 1 ns et une heure.",
            );
        }
        if self.window != window {
            self.advance_recovery(now);
            self.awaiting_data = true;
            self.window = window;
            self.clear_counts();
            self.capture_floor = Some(now);
            self.recovery_started =
                (self.recovery_required && self.connection().is_some()).then_some(now);
            self.last_snapshot = None;
        }
        Ok(())
    }
    pub fn observe(&mut self, event: &Event, now: Instant) {
        self.advance_recovery(now);
        match event {
            Event::SelectionChanged(selection) if selection != &self.selection => {
                self.clear_counts();
                self.histories = Default::default();
                self.last_timestamp = None;
                self.selection = selection.clone();
                // Include the segment that proves selection at normal startup.
                // After a continuity fault, queued old traffic cannot rebuild
                // the required clean window merely by being delivered late.
                self.capture_floor = self.recovery_required.then_some(now);
                self.recovery_started =
                    (self.recovery_required && self.connection().is_some()).then_some(now);
            }
            Event::Reliability { connection, .. }
                if connection.is_none() || *connection == self.connection() =>
            {
                self.invalidate(now)
            }
            Event::Tcp(observation) if Some(observation.connection) == self.connection() => {
                self.tcp(observation, now)
            }
            _ => {}
        }
    }
    fn connection(&self) -> Option<Connection> {
        match self.selection {
            Selection::Selected(connection) => Some(connection),
            _ => None,
        }
    }
    fn clear_counts(&mut self) {
        self.observations.clear();
        self.data_segments = 0;
        self.retransmissions = 0;
    }
    fn invalidate(&mut self, now: Instant) {
        self.clear_counts();
        // Retain known sequence coverage within this connection: a reliability
        // reset must not turn a previously seen retransmission into fresh data.
        self.recovery_required = true;
        self.recovery_started = self.connection().map(|_| now);
        self.capture_floor = Some(now);
    }
    fn advance_recovery(&mut self, now: Instant) {
        if self.connection().is_some()
            && self
                .recovery_started
                .is_some_and(|start| now.saturating_duration_since(start) >= self.window)
        {
            self.recovery_required = false;
            self.recovery_started = None;
        }
    }
    fn tcp(&mut self, observation: &TcpObservation, now: Instant) {
        let Some(timestamp) = observation.timestamp_ns else {
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
        if timestamp < 0 || self.last_timestamp.is_some_and(|last| timestamp < last) {
            self.invalidate(now);
            return;
        }
        self.last_timestamp = Some(timestamp);
        if observation.data_len == 0 {
            return;
        }
        let direction = match observation.direction {
            Direction::Incoming => 0,
            Direction::Outgoing => 1,
        };
        let start = observation
            .sequence
            .wrapping_add(u32::from(observation.flags & 0x02 != 0));
        let retransmitted = match self.histories[direction].observe(start, observation.data_len) {
            Ok(retransmitted) => retransmitted,
            Err(error) => {
                if error == ClassificationError::AmbiguousSequence {
                    self.histories[direction] = History::default();
                }
                self.invalidate(now);
                return;
            }
        };
        self.expire(now);
        // Old queued data can improve sequence coverage, but never refresh counts.
        if now.saturating_duration_since(received) >= self.window {
            return;
        }
        if self.observations.len() == MAX_OBSERVATIONS {
            self.invalidate(now);
            return;
        }
        self.observations.push_back(Observation {
            received,
            retransmitted,
        });
        self.awaiting_data = false;
        self.data_segments += 1;
        self.retransmissions += u64::from(retransmitted);
    }
    fn expire(&mut self, now: Instant) {
        while self.observations.front().is_some_and(|observation| {
            now.saturating_duration_since(observation.received) >= self.window
        }) {
            let observation = self.observations.pop_front().unwrap();
            self.data_segments -= 1;
            self.retransmissions -= u64::from(observation.retransmitted);
        }
    }
    pub fn snapshot(&mut self, now: Instant) -> Snapshot {
        self.expire(now);
        self.advance_recovery(now);
        let counts = (self.connection().is_some()
            && !self.recovery_required
            && !self.awaiting_data)
            .then(|| Counts {
                retransmissions: self.retransmissions,
                data_segments: self.data_segments,
                percentage: (self.data_segments != 0)
                    .then(|| 100.0 * self.retransmissions as f64 / self.data_segments as f64),
            });
        Snapshot {
            selection: self.selection.clone(),
            counts,
            recovering: self.recovery_required,
        }
    }
    /// First snapshot immediately, then at most one per second, including silence.
    pub fn snapshot_if_due(&mut self, now: Instant) -> Option<Snapshot> {
        if self
            .last_snapshot
            .is_some_and(|last| now.saturating_duration_since(last) < SNAPSHOT_INTERVAL)
        {
            return None;
        }
        self.last_snapshot = Some(now);
        Some(self.snapshot(now))
    }
}

#[cfg(test)]
mod tests;
