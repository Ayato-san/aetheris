use super::*;
use crate::capture::observations::{self, Reliability};
const EPOCH: i128 = 1_000_000_000_000;
fn connection(incarnation: u64) -> Connection {
    Connection {
        generation: 0,
        interface: 0,
        local: "10.0.0.2:12345".parse().unwrap(),
        remote: "10.0.0.1:13328".parse().unwrap(),
        incarnation,
    }
}
struct Fixture {
    metric: Retransmissions,
    start: Instant,
}
impl Fixture {
    fn new() -> Self {
        let start = Instant::now();
        let mut metric = Retransmissions {
            clock: CaptureClock::anchored(EPOCH, start),
            ..Retransmissions::default()
        };
        metric.observe(
            &Event::SelectionChanged(Selection::Selected(connection(1))),
            start,
        );
        Self { metric, start }
    }
    fn now(&self, ms: u64) -> Instant {
        self.start + Duration::from_millis(ms)
    }
    fn event(&mut self, event: Event, ms: u64) {
        self.metric.observe(&event, self.now(ms));
    }
    fn observation(direction: Direction, sequence: u32, length: u32, ms: u64) -> TcpObservation {
        TcpObservation {
            connection: connection(1),
            direction,
            packet_number: 1,
            timestamp_ns: Some(EPOCH + i128::from(ms) * 1_000_000),
            sequence,
            acknowledgement: Some(0),
            data_len: length,
            flags: 0x18,
            tcp_timestamp: None,
        }
    }
    fn data(&mut self, direction: Direction, sequence: u32, length: u32, ms: u64) {
        self.event(
            Event::Tcp(Self::observation(direction, sequence, length, ms)),
            ms,
        );
    }
    fn snapshot(&mut self, ms: u64) -> Snapshot {
        self.metric.snapshot(self.now(ms))
    }
    fn counts(&mut self, ms: u64) -> Counts {
        self.snapshot(ms)
            .counts
            .expect("counts should be available")
    }
    fn issue(&mut self, connection: Option<Connection>, issue: Reliability, ms: u64) {
        self.event(Event::Reliability { connection, issue }, ms);
    }
}

#[test]
fn both_directions_have_independent_sequence_spaces_and_share_one_denominator() {
    let mut f = Fixture::new();
    for (index, direction) in [Direction::Incoming, Direction::Outgoing]
        .into_iter()
        .enumerate()
    {
        f.data(direction, 100, 10, index as u64 * 2);
        f.data(direction, 100, 10, index as u64 * 2 + 1);
    }
    assert_eq!(
        f.counts(3),
        Counts {
            retransmissions: 2,
            data_segments: 4,
            percentage: Some(50.0)
        }
    );
}

#[test]
fn overlaps_partial_retransmissions_and_new_tails_count_once_per_data_segment() {
    let mut f = Fixture::new();
    for (sequence, length) in [(100, 10), (105, 2), (105, 10), (99, 2), (115, 10)] {
        f.data(Direction::Incoming, sequence, length, 0);
    }
    assert_eq!(
        f.counts(0),
        Counts {
            retransmissions: 3,
            data_segments: 5,
            percentage: Some(60.0)
        }
    );
}

#[test]
fn first_observed_segments_arriving_out_of_order_are_not_retransmissions() {
    let mut f = Fixture::new();
    for sequence in [120, 100, 110, 90] {
        f.data(Direction::Outgoing, sequence, 10, 0);
    }
    assert_eq!(
        f.counts(0),
        Counts {
            retransmissions: 0,
            data_segments: 4,
            percentage: Some(0.0)
        }
    );
    assert_eq!(f.metric.histories[1].ranges.len(), 1);
    f.data(Direction::Outgoing, 95, 30, 1);
    assert_eq!(f.counts(1).retransmissions, 1);
    assert!(!f.snapshot(1).recovering);
}

#[test]
fn a_segment_spanning_multiple_seen_ranges_and_unseen_holes_counts_once() {
    let mut f = Fixture::new();
    f.data(Direction::Outgoing, 100, 10, 0);
    f.data(Direction::Outgoing, 120, 10, 1);
    f.data(Direction::Outgoing, 95, 40, 2);
    assert_eq!(f.counts(2).retransmissions, 1);
    assert_eq!(f.counts(2).data_segments, 3);
    assert_eq!(f.metric.histories[1].ranges.len(), 1);
}

#[test]
fn sequence_wrap_and_reordering_on_either_side_of_wrap_preserve_coverage() {
    let mut f = Fixture::new();
    f.data(Direction::Incoming, u32::MAX - 4, 10, 0);
    f.data(Direction::Incoming, 5, 10, 1);
    f.data(Direction::Incoming, u32::MAX - 14, 10, 2);
    assert_eq!(f.counts(2).retransmissions, 0);
    f.data(Direction::Incoming, u32::MAX - 1, 10, 3);
    assert_eq!(f.counts(3).retransmissions, 1);
    assert_eq!(f.counts(3).data_segments, 4);
}

#[test]
fn ack_syn_and_fin_without_data_do_not_enter_the_denominator() {
    let mut f = Fixture::new();
    for flags in [0x10, 0x02, 0x12, 0x11, 0x04] {
        let mut observation = Fixture::observation(Direction::Outgoing, 99, 0, 0);
        observation.flags = flags;
        f.event(Event::Tcp(observation), 0);
    }
    assert_eq!(
        f.counts(0),
        Counts {
            retransmissions: 0,
            data_segments: 0,
            percentage: None
        }
    );
    let mut syn_data = Fixture::observation(Direction::Outgoing, 99, 10, 1);
    syn_data.flags = 0x02;
    f.event(Event::Tcp(syn_data), 1);
    f.data(Direction::Outgoing, 100, 10, 2);
    assert_eq!(f.counts(2).retransmissions, 1);
    assert_eq!(f.counts(2).data_segments, 2);
}

#[test]
fn counters_expire_exactly_at_window_boundary_but_sequence_history_survives() {
    let mut f = Fixture::new();
    assert_eq!(f.metric.window(), Duration::from_secs(900));
    f.data(Direction::Outgoing, 100, 10, 0);
    f.data(Direction::Outgoing, 100, 10, 100);
    assert_eq!(f.counts(899_999).data_segments, 2);
    assert_eq!(
        f.counts(900_000),
        Counts {
            retransmissions: 1,
            data_segments: 1,
            percentage: Some(100.0)
        }
    );
    assert_eq!(
        f.counts(900_100),
        Counts {
            retransmissions: 0,
            data_segments: 0,
            percentage: None
        }
    );
    f.data(Direction::Outgoing, 100, 10, 900_101);
    assert_eq!(f.counts(900_101).retransmissions, 1);
}

#[test]
fn reconnect_and_ambiguity_discard_both_directions_without_mixing_connections() {
    let mut f = Fixture::new();
    f.data(Direction::Incoming, 100, 10, 0);
    f.data(Direction::Outgoing, 100, 10, 0);
    f.data(Direction::Incoming, 100, 10, 1);
    f.event(
        Event::SelectionChanged(Selection::Selected(connection(2))),
        2,
    );
    f.data(Direction::Incoming, 100, 10, 3); // Old incarnation ignored.
    assert_eq!(f.counts(3).data_segments, 0);
    let mut observation = Fixture::observation(Direction::Incoming, 100, 10, 4);
    observation.connection = connection(2);
    f.event(Event::Tcp(observation), 4);
    assert_eq!(
        f.counts(4),
        Counts {
            retransmissions: 0,
            data_segments: 1,
            percentage: Some(0.0)
        }
    );
    f.event(Event::SelectionChanged(Selection::Ambiguous), 5);
    assert!(f.snapshot(5).counts.is_none());
    f.event(Event::SelectionChanged(Selection::Absent), 6);
    assert!(f.snapshot(6).counts.is_none());
}

#[test]
fn window_change_resets_counts_without_forgetting_known_retransmission_coverage() {
    let mut f = Fixture::new();
    f.data(Direction::Incoming, 100, 10, 0);
    assert!(f.metric.set_window(Duration::ZERO, f.now(1)).is_err());
    assert!(
        f.metric
            .set_window(Duration::from_secs(3601), f.now(1))
            .is_err()
    );
    assert_eq!(f.counts(1).data_segments, 1);
    f.metric
        .set_window(Duration::from_secs(2), f.now(2))
        .unwrap();
    assert_eq!(f.counts(2).data_segments, 0);
    f.data(Direction::Incoming, 100, 10, 3);
    assert_eq!(f.counts(3).retransmissions, 1);
    assert_eq!(f.counts(2003).data_segments, 0);
    f.metric
        .set_window(Duration::from_secs(2), f.now(2004))
        .unwrap();
    f.data(Direction::Incoming, 110, 10, 2005);
    f.metric
        .set_window(Duration::from_secs(2), f.now(2006))
        .unwrap();
    assert_eq!(f.counts(2006).data_segments, 1); // Same setting is a no-op.
}

#[test]
fn continuity_faults_mask_counts_for_a_full_clean_window_and_do_not_create_a_loss_counter() {
    for issue in [
        Reliability::PacketNumberDiscontinuity,
        Reliability::Truncated,
        Reliability::StreamInterrupted("missing bytes".into()),
        Reliability::Interrupted("service lost".into()),
    ] {
        let mut f = Fixture::new();
        f.metric
            .set_window(Duration::from_secs(2), f.now(0))
            .unwrap();
        f.data(Direction::Incoming, 100, 10, 0);
        f.issue(None, issue, 100);
        f.data(Direction::Incoming, 100, 10, 200);
        assert!(f.snapshot(2099).recovering);
        assert!(f.snapshot(2099).counts.is_none());
        assert_eq!(
            f.counts(2100),
            Counts {
                retransmissions: 1,
                data_segments: 1,
                percentage: Some(100.0)
            }
        );
        assert_eq!(observations::Tracker::default().collector_drops(), None);
    }
}

#[test]
fn unrelated_faults_do_not_reset_counts_but_repeated_relevant_faults_restart_recovery() {
    let mut f = Fixture::new();
    f.metric
        .set_window(Duration::from_secs(2), f.now(0))
        .unwrap();
    f.data(Direction::Outgoing, 100, 10, 0);
    f.issue(Some(connection(2)), Reliability::TimestampMissing, 100);
    assert_eq!(f.counts(100).data_segments, 1);
    f.issue(Some(connection(1)), Reliability::TimestampMissing, 200);
    f.issue(None, Reliability::CapacityExceeded, 300);
    f.data(Direction::Outgoing, 100, 10, 400);
    assert!(f.snapshot(2299).counts.is_none());
    assert_eq!(f.counts(2300).retransmissions, 1);
}

#[test]
fn disconnected_time_does_not_satisfy_recovery_and_a_new_window_restarts_the_wait() {
    let mut f = Fixture::new();
    f.metric
        .set_window(Duration::from_secs(2), f.now(0))
        .unwrap();
    f.issue(None, Reliability::Interrupted("unavailable".into()), 100);
    f.event(Event::SelectionChanged(Selection::Absent), 100);
    assert!(f.snapshot(10_000).recovering);
    f.event(
        Event::SelectionChanged(Selection::Selected(connection(1))),
        10_000,
    );
    f.data(Direction::Incoming, 100, 10, 10_100);
    f.metric
        .set_window(Duration::from_secs(1), f.now(10_200))
        .unwrap();
    f.data(Direction::Incoming, 100, 10, 10_300);
    assert!(f.snapshot(11_199).counts.is_none());
    assert_eq!(f.counts(11_200).retransmissions, 1);
}

#[test]
fn missing_negative_future_and_backward_timestamps_are_unavailable_instead_of_zero() {
    for timestamp in [
        None,
        Some(-1),
        Some(EPOCH + 300_000_000),
        Some(EPOCH + 50_000_000),
    ] {
        let mut f = Fixture::new();
        f.data(Direction::Incoming, 100, 10, 100);
        let mut observation = Fixture::observation(Direction::Incoming, 110, 10, 200);
        observation.timestamp_ns = timestamp;
        f.event(Event::Tcp(observation), 200);
        assert!(f.snapshot(200).counts.is_none());
        assert!(f.snapshot(200).recovering);
    }
}

#[test]
fn stale_queued_data_is_not_fresh_but_can_prove_a_later_retransmission() {
    let mut f = Fixture::new();
    f.event(
        Event::Tcp(Fixture::observation(Direction::Outgoing, 100, 10, 0)),
        900_000,
    );
    assert_eq!(f.counts(900_000).data_segments, 0);
    f.data(Direction::Outgoing, 100, 10, 900_001);
    assert_eq!(f.counts(900_001).retransmissions, 1);
    f.issue(None, Reliability::Truncated, 900_002);
    f.event(
        Event::Tcp(Fixture::observation(Direction::Outgoing, 110, 10, 900_001)),
        900_003,
    );
    assert!(f.metric.observations.is_empty());
}

#[test]
fn snapshot_cadence_runs_during_silence_and_invalid_settings_preserve_it() {
    let mut f = Fixture::new();
    assert!(f.metric.snapshot_if_due(f.now(0)).is_some());
    assert!(f.metric.snapshot_if_due(f.now(999)).is_none());
    assert!(f.metric.set_window(Duration::ZERO, f.now(999)).is_err());
    assert!(f.metric.snapshot_if_due(f.now(999)).is_none());
    assert!(f.metric.snapshot_if_due(f.now(1000)).is_some());
    f.metric
        .set_window(Duration::from_secs(2), f.now(1001))
        .unwrap();
    assert!(f.metric.snapshot_if_due(f.now(1001)).is_some());
}

#[test]
fn half_sequence_space_is_ambiguous_and_requires_recovery() {
    let mut f = Fixture::new();
    f.data(Direction::Incoming, 100, 10, 0);
    f.data(Direction::Incoming, 110u32.wrapping_add(1 << 31), 10, 1);
    assert!(f.snapshot(1).counts.is_none());
    assert!(f.snapshot(1).recovering);
    f.data(Direction::Incoming, 100, 10, 2);
    assert_eq!(f.counts(900_001).data_segments, 1);
}

#[test]
fn interval_capacity_never_discards_holes_silently_and_contiguous_merging_can_recover() {
    let mut f = Fixture::new();
    for index in 0..MAX_INTERVALS {
        f.data(Direction::Incoming, index as u32 * 2, 1, index as u64);
    }
    assert_eq!(
        f.counts(MAX_INTERVALS as u64 - 1).data_segments,
        MAX_INTERVALS as u64
    );
    f.data(
        Direction::Incoming,
        MAX_INTERVALS as u32 * 2,
        1,
        MAX_INTERVALS as u64,
    );
    assert!(f.snapshot(MAX_INTERVALS as u64).counts.is_none());
    assert_eq!(f.metric.histories[0].ranges.len(), MAX_INTERVALS);
    f.data(Direction::Incoming, 0, MAX_INTERVALS as u32 * 2 + 1, 4100);
    assert_eq!(f.metric.histories[0].ranges.len(), 1);
    assert_eq!(f.counts(900_000 + MAX_INTERVALS as u64).retransmissions, 1);
}

#[test]
fn observation_capacity_masks_the_ratio_instead_of_silently_changing_its_denominator() {
    let mut f = Fixture::new();
    for index in 0..MAX_OBSERVATIONS {
        f.data(Direction::Incoming, index as u32, 1, 0);
    }
    assert_eq!(f.counts(0).data_segments, MAX_OBSERVATIONS as u64);
    f.data(Direction::Incoming, MAX_OBSERVATIONS as u32, 1, 1);
    assert!(f.snapshot(1).counts.is_none());
    assert!(f.snapshot(1).recovering);
    assert!(f.metric.observations.len() <= MAX_OBSERVATIONS);
    f.data(Direction::Incoming, MAX_OBSERVATIONS as u32, 1, 2);
    assert_eq!(f.counts(900_001).retransmissions, 1);
}

#[test]
fn raw_shared_observations_count_server_and_client_data_and_keep_capture_faults_separate() {
    use crate::capture::{
        CapturedPacket,
        widgets::tests::{tcp, ticks},
    };
    let mut f = Fixture::new();
    f.metric.selection = Selection::Absent;
    let mut tracker = observations::Tracker::default();
    let mut number = 0;
    let mut feed = |f: &mut Fixture, mut packet: CapturedPacket, ms: u64| {
        number += 1;
        packet.number = number;
        packet.timestamp_ns = Some(EPOCH + i128::from(ms) * 1_000_000);
        let now = f.now(ms);
        tracker.packet(&packet, now, |event| f.metric.observe(&event, now));
    };
    feed(&mut f, tcp(100, 0x18, &ticks()), 0);
    feed(&mut f, tcp(100, 0x18, &ticks()), 1);
    let outgoing = |sequence| {
        let mut packet = tcp(sequence, 0x18, &[0; 10]);
        packet.bytes[12..20].copy_from_slice(&[10, 0, 0, 2, 10, 0, 0, 1]);
        packet.bytes[20..24].copy_from_slice(&[0x30, 0x39, 0x34, 0x10]);
        packet
    };
    feed(&mut f, outgoing(1000), 2);
    feed(&mut f, outgoing(1000), 3);
    assert_eq!(
        f.counts(3),
        Counts {
            retransmissions: 2,
            data_segments: 4,
            percentage: Some(50.0)
        }
    );
    for (index, sequence) in [1030, 1020, 1010].into_iter().enumerate() {
        feed(&mut f, outgoing(sequence), 4 + index as u64);
    }
    assert_eq!(f.counts(6).retransmissions, 2);
    assert_eq!(f.counts(6).data_segments, 7);
    let mut truncated = outgoing(1040);
    truncated.original_len += 1;
    feed(&mut f, truncated, 7);
    assert!(f.snapshot(7).counts.is_none());
    assert!(f.snapshot(7).recovering);
    assert_eq!(tracker.collector_drops(), None);
}

#[test]
fn historical_coverage_uses_observed_bytes_without_guessing_a_missing_prefix() {
    let mut f = Fixture::new();
    // Seeing a high sequence is not proof that earlier bytes were captured.
    f.data(Direction::Incoming, 500, 10, 0);
    f.data(Direction::Incoming, 100, 10, 1);
    assert_eq!(f.counts(1).retransmissions, 0);
    f.data(Direction::Incoming, 100, 10, 2);
    assert_eq!(f.counts(2).retransmissions, 1);
    assert_eq!(f.counts(2).data_segments, 3);
}

#[test]
fn absence_of_selection_is_unavailable_and_silence_alone_does_not_reset_history() {
    let mut metric = Retransmissions::default();
    assert!(metric.snapshot(Instant::now()).counts.is_none());
    let mut f = Fixture::new();
    f.data(Direction::Incoming, 100, 10, 0);
    assert_eq!(f.counts(2_000_000).data_segments, 0);
    assert!(!f.snapshot(2_000_000).recovering);
    f.data(Direction::Incoming, 100, 10, 2_000_001);
    assert_eq!(f.counts(2_000_001).retransmissions, 1);
}
