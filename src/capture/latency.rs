//! Bounded passive data/ACK RTT samples for one confidently selected connection.
//! A sample includes TCP delayed-ACK and capture-reader delay; it is not a freeze
//! duration. TCP timestamp options are deliberately not used to resolve Karn
//! ambiguity without validated clock/echo semantics.
use super::{
    observations::{self, Connection, Direction, Event, Selection, TcpObservation},
    widgets::CaptureClock,
};
use std::{
    cmp::Ordering,
    collections::VecDeque,
    time::{Duration, Instant},
};

pub const DEFAULT_WINDOW: Duration = Duration::from_secs(10);
const MAX_WINDOW: Duration = Duration::from_secs(15 * 60);
const SNAPSHOT_INTERVAL: Duration = Duration::from_secs(1);
const MAX_PENDING: usize = 4096;
const MAX_SAMPLES: usize = 4096;

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub selection: Selection,
    /// `None` is rendered as “—” by a future widget, never as zero latency.
    pub ping: Option<Duration>,
    pub jitter: Option<Duration>,
    pub samples: usize,
    /// A compromised capture requires a full clean window before publication.
    pub recovering: bool,
}

struct Pending {
    end: u32,
    timestamp_ns: i128,
}
struct Sample {
    received: Instant,
    rtt: Duration,
}

/// No packets or unbounded history are retained. Outstanding data can outlive
/// the statistics window so an ACK arriving several seconds later still counts.
pub struct Latency {
    selection: Selection,
    window: Duration,
    clock: CaptureClock,
    next_sequence: Option<u32>,
    last_ack: Option<u32>,
    last_timestamp: Option<i128>,
    /// Cumulative ACKs through this boundary are excluded after overlap/gaps.
    barrier: Option<u32>,
    pending: VecDeque<Pending>,
    samples: VecDeque<Sample>,
    recovery_started: Option<Instant>,
    recovery_required: bool,
    capture_floor: Option<Instant>,
    last_snapshot: Option<Instant>,
}
impl Default for Latency {
    fn default() -> Self {
        Self {
            selection: Selection::Absent,
            window: DEFAULT_WINDOW,
            clock: CaptureClock::default(),
            next_sequence: None,
            last_ack: None,
            last_timestamp: None,
            barrier: None,
            pending: VecDeque::new(),
            samples: VecDeque::new(),
            recovery_started: None,
            recovery_required: false,
            capture_floor: None,
            last_snapshot: None,
        }
    }
}
impl Latency {
    pub fn window(&self) -> Duration {
        self.window
    }

    /// Debug configuration can persist this duration at its UI integration point.
    /// Reject unbounded/zero windows before mutating any state.
    pub fn set_window(&mut self, window: Duration, now: Instant) -> Result<(), &'static str> {
        if window.is_zero() || window > MAX_WINDOW {
            return Err("La fenêtre de latence doit être comprise entre 1 ns et 15 minutes.");
        }
        if self.window != window {
            self.advance_recovery(now);
            self.window = window;
            self.clear_measurements();
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
                self.reset_connection();
                self.selection = selection.clone();
                self.capture_floor = Some(now);
                self.recovery_started =
                    (self.recovery_required && self.connection().is_some()).then_some(now);
            }
            Event::Reliability { connection, .. }
                if connection.is_none() || *connection == self.connection() =>
            {
                self.invalidate(now);
            }
            Event::Tcp(observation) if Some(observation.connection) == self.connection() => {
                self.tcp(observation, now);
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

    fn reset_connection(&mut self) {
        self.next_sequence = None;
        self.last_ack = None;
        self.last_timestamp = None;
        self.barrier = None;
        self.pending.clear();
        self.samples.clear();
    }

    fn clear_measurements(&mut self) {
        // Keep constant-size sequence frontiers within the same connection.
        // Forgetting them would turn a known old retransmission into a fresh
        // send after an interruption, overflow or configuration change.
        self.pending.clear();
        self.samples.clear();
        self.barrier = self
            .next_sequence
            .filter(|end| !self.last_ack.is_some_and(|ack| at_or_before(*end, ack)));
    }

    fn invalidate(&mut self, now: Instant) {
        self.clear_measurements();
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
        if timestamp < 0 || self.last_timestamp.is_some_and(|last| timestamp < last) {
            self.invalidate(now);
            return;
        }
        // Queued evidence captured before a detected interruption cannot rebuild
        // the required clean window, even if delivered after the interruption.
        if self.capture_floor.is_some_and(|start| received < start) {
            return;
        }
        self.last_timestamp = Some(timestamp);
        match observation.direction {
            Direction::Outgoing => self.sent(observation, timestamp, now),
            Direction::Incoming => {
                if let Some(ack) = observation.acknowledgement {
                    self.acknowledged(ack, timestamp, received, now);
                }
            }
        }
    }

    fn sent(&mut self, observation: &TcpObservation, timestamp: i128, now: Instant) {
        let syn = u32::from(observation.flags & 0x02 != 0);
        let fin = u32::from(observation.flags & 0x01 != 0);
        if observation.data_len == 0 {
            if self.next_sequence.is_none() && syn > 0 {
                self.next_sequence = Some(observation.sequence.wrapping_add(1));
            }
            if fin > 0 {
                let next = observation.sequence.wrapping_add(syn);
                if self.next_sequence.is_none_or(|expected| expected == next) {
                    self.next_sequence = Some(next.wrapping_add(1));
                } else {
                    self.invalidate(now);
                }
            }
            return;
        }
        if observation.data_len >= (1 << 31) {
            self.invalidate(now);
            return;
        }
        let start = observation.sequence.wrapping_add(syn);
        let data_end = start.wrapping_add(observation.data_len);
        let end = data_end.wrapping_add(fin);
        let next = self.next_sequence.unwrap_or(start);
        match serial_cmp(start, next) {
            Some(Ordering::Equal) => {
                if self.pending.len() == MAX_PENDING {
                    self.invalidate(now);
                    self.next_sequence = Some(end);
                    self.barrier = Some(end);
                    return;
                }
                self.next_sequence = Some(end);
                self.pending.push_back(Pending {
                    end: data_end,
                    timestamp_ns: timestamp,
                });
            }
            Some(Ordering::Less) => {
                // Already acknowledged duplicate data cannot create a new sample.
                // With outstanding data, even an old duplicate may trigger an
                // advancing cumulative ACK; exclude that ambiguity as well.
                if self.pending.is_empty()
                    && self.last_ack.is_some_and(|ack| at_or_before(end, ack))
                {
                    return;
                }
                let high = if serial_cmp(end, next) == Some(Ordering::Greater) {
                    end
                } else {
                    next
                };
                self.next_sequence = Some(high);
                // Suppress the entire cumulative ACK covering ambiguous data,
                // including any freshly sent tail of a partial retransmission.
                self.barrier = Some(high);
            }
            Some(Ordering::Greater) => {
                // Missing outgoing observations are a continuity failure, not a
                // measured packet loss. Wait for ACK to cross the unseen span.
                self.invalidate(now);
                self.next_sequence = Some(end);
                self.barrier = Some(end);
            }
            None => self.invalidate(now),
        }
    }

    fn acknowledged(&mut self, ack: u32, timestamp: i128, received: Instant, now: Instant) {
        if let Some(last) = self.last_ack {
            match serial_cmp(ack, last) {
                Some(Ordering::Less | Ordering::Equal) => return,
                None => {
                    self.invalidate(now);
                    return;
                }
                _ => {}
            }
        }
        let Some(next) = self.next_sequence else {
            self.last_ack = Some(ack);
            self.next_sequence = Some(ack);
            return;
        };
        match serial_cmp(ack, next) {
            Some(Ordering::Greater) | None => {
                // This ACK covers bytes we never saw sent. It cannot supply RTT.
                self.invalidate(now);
                self.last_ack = Some(ack);
                self.next_sequence = Some(ack);
                self.barrier = None;
                return;
            }
            _ => {}
        }
        self.last_ack = Some(ack);
        let excluded = self.barrier.is_some();
        if self
            .barrier
            .is_some_and(|barrier| at_or_before(barrier, ack))
        {
            self.barrier = None;
        }
        let mut latest = None;
        while self
            .pending
            .front()
            .is_some_and(|pending| at_or_before(pending.end, ack))
        {
            latest = self.pending.pop_front();
        }
        if !excluded && let Some(pending) = latest {
            let delta = timestamp
                .checked_sub(pending.timestamp_ns)
                .and_then(|delta| u64::try_from(delta).ok())
                .filter(|delta| *delta > 0);
            let Some(delta) = delta else {
                self.invalidate(now);
                return;
            };
            self.expire_samples(now);
            if self.samples.len() == MAX_SAMPLES {
                self.invalidate(now);
                return;
            }
            // One sample per advancing ACK, from the newest fully ACKed segment.
            self.samples.push_back(Sample {
                received,
                rtt: Duration::from_nanos(delta),
            });
        }
    }

    fn expire_samples(&mut self, now: Instant) {
        while self
            .samples
            .front()
            .is_some_and(|sample| now.saturating_duration_since(sample.received) >= self.window)
        {
            self.samples.pop_front();
        }
    }

    pub fn snapshot(&mut self, now: Instant) -> Snapshot {
        self.expire_samples(now);
        self.advance_recovery(now);
        let recovering = self.recovery_required;
        // The separate capture-time floor still prevents replayed old evidence
        // from being accepted once a clean window has elapsed.
        let count = self.samples.len();
        let available = self.connection().is_some() && !recovering;
        let ping = (available && count > 0).then(|| {
            mean(
                self.samples.iter().map(|sample| sample.rtt.as_nanos()),
                count,
            )
        });
        let jitter = (available && count > 1).then(|| {
            let values = self.samples.iter().map(|sample| sample.rtt.as_nanos());
            mean(
                values
                    .clone()
                    .zip(values.skip(1))
                    .map(|(a, b)| a.abs_diff(b)),
                count - 1,
            )
        });
        Snapshot {
            selection: self.selection.clone(),
            ping,
            jitter,
            samples: count,
            recovering,
        }
    }

    /// First snapshot immediately, then at most once per second. Call even when
    /// no packets arrive so statistics expire without inventing new samples.
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

fn serial_cmp(a: u32, b: u32) -> Option<Ordering> {
    let delta = a.wrapping_sub(b) as i32;
    (delta != i32::MIN).then(|| delta.cmp(&0))
}
fn at_or_before(a: u32, b: u32) -> bool {
    matches!(serial_cmp(a, b), Some(Ordering::Less | Ordering::Equal))
}
fn mean(values: impl Iterator<Item = u128>, count: usize) -> Duration {
    // Each sample is a u64 nanosecond duration and the queue is bounded, so the
    // u128 sum cannot overflow and its average fits into u64.
    Duration::from_nanos((values.sum::<u128>() / count as u128) as u64)
}

/// A lazily created consumer of the same capture hub used by existing widgets.
/// Future network widgets share this owner; dropping it clears all RTT history.
pub struct Source {
    observations: observations::Source,
    latency: Latency,
}
impl Source {
    pub fn subscribe() -> super::Result<Self> {
        Ok(Self {
            observations: observations::Source::subscribe()?,
            latency: Latency::default(),
        })
    }
    pub fn set_window(&mut self, window: Duration) -> Result<(), &'static str> {
        self.latency.set_window(window, Instant::now())
    }
    pub fn snapshot(&mut self) -> Snapshot {
        self.latency.snapshot(Instant::now())
    }
    pub fn poll(&mut self) -> super::Result<Option<Snapshot>> {
        let latency = &mut self.latency;
        self.observations
            .poll(|event| latency.observe(&event, Instant::now()))?;
        Ok(latency.snapshot_if_due(Instant::now()))
    }
}

#[cfg(test)]
mod tests;
