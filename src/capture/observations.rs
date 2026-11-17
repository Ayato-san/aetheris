//! Ephemeral TCP evidence from the existing, process-attributed capture.
//! No RTT calculation, packet retention or debug database access lives here.
use super::{CaptureBatch, CapturedPacket, network, shared, widgets};
use std::{
    collections::BTreeMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const MAX_CONNECTIONS: usize = 8;
const IDLE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Connection {
    pub generation: u32,
    pub interface: u32,
    pub local: SocketAddr,
    pub remote: SocketAddr,
    /// Distinguishes successive uses of the same socket tuple, including SYNs.
    pub incarnation: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Selection {
    #[default]
    Absent,
    Selected(Connection),
    Ambiguous,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reliability {
    Interrupted(String),
    GenerationChanged,
    PacketNumberDiscontinuity,
    Truncated,
    Malformed(String),
    DirectionUnavailable,
    TimestampMissing,
    TimestampIncoherent,
    /// A decoder detected missing/conflicting bytes or lost synchronization.
    /// This is not evidence of a collector drop or a retransmission.
    StreamInterrupted(String),
    CapacityExceeded,
}

#[derive(Clone, Debug)]
pub struct TcpObservation {
    pub connection: Connection,
    pub direction: Direction,
    pub packet_number: u64,
    pub timestamp_ns: Option<i128>,
    pub sequence: u32,
    pub acknowledgement: Option<u32>,
    pub data_len: u32,
    pub flags: u8,
    pub tcp_timestamp: Option<network::TcpTimestamp>,
}

#[derive(Clone, Debug)]
pub enum Event {
    /// Acquisition state, independent of game-connection recognition.
    CaptureState {
        available: bool,
    },
    SelectionChanged(Selection),
    Reliability {
        connection: Option<Connection>,
        issue: Reliability,
    },
    /// All parseable, attributed traffic, including UDP and unselected flows.
    /// Byte count includes the captured link/IP/transport headers.
    Traffic {
        direction: Direction,
        captured_bytes: u32,
        timestamp_ns: Option<i128>,
    },
    Tcp(TcpObservation),
}

#[derive(Default)]
struct Detection {
    interruption: Option<String>,
}
impl widgets::Consumer for Detection {
    fn accepts(&self, _: [u8; 2], _: Option<u32>) -> bool {
        false
    }
    fn message(&mut self, _: [u8; 2], _: &[u8], _: Instant) {}
    fn reset(&mut self, error: Option<&str>) {
        if let Some(error) = error {
            self.interruption = Some(error.to_owned());
        }
    }
}

struct Candidate {
    connection: Connection,
    decoder: widgets::Decoder,
    detection: Arc<Mutex<Detection>>,
    last_seen: Instant,
    last_timestamp: Option<i128>,
    syn: [Option<u32>; 2],
}
impl Candidate {
    fn clients(&self) -> Vec<Arc<Mutex<dyn widgets::Consumer>>> {
        vec![self.detection.clone()]
    }
}

type Key = (u32, u32, SocketAddr, SocketAddr);

/// Feed only evidence already attributed to Aion 2 by the shared capture.
/// At most eight candidate decoders retain bounded reassembly buffers. Events
/// are delivered synchronously; the tracker never queues observations.
#[derive(Default)]
pub struct Tracker {
    candidates: BTreeMap<Key, Candidate>,
    generation: Option<u32>,
    last_number: Option<u64>,
    incarnation: u64,
    selection: Selection,
    capacity_until: Option<Instant>,
    clock: widgets::CaptureClock,
    interruption: Option<String>,
    capture_available: bool,
    skip_connection_tracking: bool,
}
impl Tracker {
    pub(crate) fn set_connection_tracking(&mut self, enabled: bool) {
        let skip = !enabled;
        if self.skip_connection_tracking != skip {
            self.skip_connection_tracking = skip;
            self.candidates.clear();
            self.selection = Selection::Absent;
            self.capacity_until = None;
        }
    }

    fn capture_state(&mut self, available: bool, emit: &mut impl FnMut(Event)) {
        if self.capture_available != available {
            self.capture_available = available;
            emit(Event::CaptureState { available });
        }
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Winsock raw capture currently exposes no reliable drop counter.
    pub fn collector_drops(&self) -> Option<u64> {
        None
    }

    pub fn interrupt(&mut self, reason: impl Into<String>, mut emit: impl FnMut(Event)) {
        self.capture_state(false, &mut emit);
        self.candidates.clear();
        self.capacity_until = None;
        self.generation = None;
        self.last_number = None;
        emit(Event::Reliability {
            connection: None,
            issue: Reliability::Interrupted(reason.into()),
        });
        self.select(&mut emit);
    }

    pub fn batch(&mut self, batch: &CaptureBatch, now: Instant, mut emit: impl FnMut(Event)) {
        if batch.interruption != self.interruption {
            if let Some(reason) = &batch.interruption {
                self.interrupt(reason.clone(), &mut emit);
            }
            self.interruption = batch.interruption.clone();
        }
        if batch.interruption.is_none() && !batch.finished && batch.error.is_none() {
            self.capture_state(true, &mut emit);
        }
        for packet in &batch.packets {
            self.packet(packet, now, &mut emit);
        }
        self.expire(now, &mut emit);
        if batch.finished || batch.error.is_some() {
            self.interrupt(batch.error.as_deref().unwrap_or("Capture terminée"), emit);
        }
    }

    pub fn expire(&mut self, now: Instant, mut emit: impl FnMut(Event)) {
        for candidate in self.candidates.values_mut() {
            candidate
                .decoder
                .expire_reassembly(now, &candidate.clients());
            report_detection(candidate, &mut emit);
        }
        self.candidates.retain(|_, candidate| {
            candidate.decoder.is_synchronized()
                || now.saturating_duration_since(candidate.last_seen) < IDLE_TIMEOUT
        });
        if self.capacity_until.is_some_and(|until| now >= until) {
            self.capacity_until = None;
        }
        self.select(&mut emit);
    }

    fn select(&mut self, emit: &mut impl FnMut(Event)) {
        let mut confirmed = self
            .candidates
            .values()
            .filter(|candidate| candidate.decoder.is_synchronized());
        let next = match (confirmed.next(), confirmed.next()) {
            _ if self.capacity_until.is_some() => Selection::Ambiguous,
            (Some(candidate), None) => Selection::Selected(candidate.connection),
            (None, _) => Selection::Absent,
            _ => Selection::Ambiguous,
        };
        if next != self.selection {
            self.selection = next.clone();
            emit(Event::SelectionChanged(next));
        }
    }

    pub fn packet(&mut self, packet: &CapturedPacket, now: Instant, mut emit: impl FnMut(Event)) {
        self.expire(now, &mut emit);
        if self
            .generation
            .is_some_and(|generation| generation != packet.section_id)
        {
            self.candidates.clear();
            self.capacity_until = None;
            self.last_number = None;
            emit(Event::Reliability {
                connection: None,
                issue: Reliability::GenerationChanged,
            });
            self.select(&mut emit);
        }
        self.generation = Some(packet.section_id);
        if self
            .last_number
            .is_some_and(|number| number.checked_add(1) != Some(packet.number))
        {
            self.candidates.clear();
            emit(Event::Reliability {
                connection: None,
                issue: Reliability::PacketNumberDiscontinuity,
            });
            self.select(&mut emit);
        }
        self.last_number = Some(packet.number);
        let parsed = network::parse(packet);
        if packet.is_truncated() || parsed.issue.is_some() {
            // An unreadable header may hide the selected tuple: invalidate all
            // candidates conservatively, rather than manufacturing clean data.
            self.candidates.clear();
            emit(Event::Reliability {
                connection: None,
                issue: if packet.is_truncated() {
                    Reliability::Truncated
                } else {
                    Reliability::Malformed(parsed.issue.unwrap())
                },
            });
            self.select(&mut emit);
            return;
        }
        let (Some(source), Some(destination)) = (parsed.source, parsed.destination) else {
            return;
        };
        let (direction, local, remote, side) = match packet.local_address {
            Some(ip) if source.ip() == ip && destination.ip() != ip => {
                (Direction::Outgoing, source, destination, 0)
            }
            Some(ip) if destination.ip() == ip && source.ip() != ip => {
                (Direction::Incoming, destination, source, 1)
            }
            _ => {
                self.candidates.clear();
                emit(Event::Reliability {
                    connection: None,
                    issue: Reliability::DirectionUnavailable,
                });
                self.select(&mut emit);
                return;
            }
        };
        emit(Event::Traffic {
            direction,
            captured_bytes: packet.captured_len,
            timestamp_ns: packet.timestamp_ns,
        });
        if self.skip_connection_tracking {
            return;
        }
        let Some(segment) = parsed.tcp else {
            return;
        };
        let key = (packet.section_id, packet.interface_id, local, remote);
        // SYN on an established tuple starts a new incarnation. The opposite
        // handshake SYN is allowed; identical SYN retransmissions do not reset.
        let restart = segment.syn
            && self
                .candidates
                .get(&key)
                .is_some_and(|candidate| match candidate.syn[side] {
                    Some(sequence) => sequence != segment.sequence,
                    None => {
                        candidate.decoder.is_synchronized() || candidate.syn[1 - side].is_none()
                    }
                });
        if restart {
            self.candidates.remove(&key);
            self.select(&mut emit);
        }
        if !self.candidates.contains_key(&key) {
            if self.candidates.len() >= MAX_CONNECTIONS {
                self.capacity_until = now.checked_add(IDLE_TIMEOUT);
                emit(Event::Reliability {
                    connection: None,
                    issue: Reliability::CapacityExceeded,
                });
                self.select(&mut emit);
                return;
            }
            self.incarnation = self.incarnation.wrapping_add(1);
            self.candidates.insert(
                key,
                Candidate {
                    connection: Connection {
                        generation: key.0,
                        interface: key.1,
                        local,
                        remote,
                        incarnation: self.incarnation,
                    },
                    decoder: widgets::Decoder::default(),
                    detection: Arc::new(Mutex::new(Detection::default())),
                    last_seen: now,
                    last_timestamp: None,
                    syn: [None; 2],
                },
            );
        }
        let candidate = self.candidates.get_mut(&key).unwrap();
        candidate.last_seen = now;
        if segment.syn {
            candidate.syn[side] = Some(segment.sequence);
        }
        let timestamp_issue = match packet.timestamp_ns {
            None => Some(Reliability::TimestampMissing),
            Some(time)
                if time < 0
                    || self.clock.received(Some(time), now).is_none()
                    || candidate.last_timestamp.is_some_and(|last| time < last) =>
            {
                Some(Reliability::TimestampIncoherent)
            }
            _ => None,
        };
        if timestamp_issue.is_none() && packet.timestamp_ns.is_some() {
            candidate.last_timestamp = packet.timestamp_ns;
        }
        if let Some(issue) = timestamp_issue.as_ref() {
            emit(Event::Reliability {
                connection: Some(candidate.connection),
                issue: issue.clone(),
            });
        }
        candidate.decoder.feed(packet, now, &candidate.clients());
        report_detection(candidate, &mut emit);
        let observation = TcpObservation {
            connection: candidate.connection,
            direction,
            packet_number: packet.number,
            timestamp_ns: if timestamp_issue.is_none() {
                packet.timestamp_ns
            } else {
                None
            },
            sequence: segment.sequence,
            acknowledgement: segment.ack.then_some(segment.acknowledgement),
            data_len: segment.payload.len() as u32,
            flags: segment.flags,
            tcp_timestamp: segment.timestamp,
        };
        if segment.fin || segment.rst {
            self.candidates.remove(&key);
        }
        self.select(&mut emit);
        emit(Event::Tcp(observation));
    }
}

fn report_detection(candidate: &Candidate, emit: &mut impl FnMut(Event)) {
    let reason = candidate.detection.lock().unwrap().interruption.take();
    if let Some(reason) = reason {
        emit(Event::Reliability {
            connection: Some(candidate.connection),
            issue: Reliability::StreamInterrupted(reason),
        });
    }
}

/// Owns one subscription to the existing capture hub. Dropping it releases that
/// subscription; other consumers (widgets and recorder) continue independently.
pub struct Source {
    subscription: shared::Subscription,
    tracker: Tracker,
}
impl Source {
    pub fn subscribe() -> super::Result<Self> {
        Ok(Self {
            subscription: shared::subscribe(None)?,
            tracker: Tracker::default(),
        })
    }
    pub fn tracker(&self) -> &Tracker {
        &self.tracker
    }
    pub fn poll(&mut self, mut emit: impl FnMut(Event)) -> super::Result<()> {
        match self.subscription.next_batch() {
            Ok(Some(batch)) => self.tracker.batch(&batch, Instant::now(), emit),
            Ok(None) => self.tracker.expire(Instant::now(), emit),
            Err(error) => {
                self.tracker.interrupt(error.to_string(), &mut emit);
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
