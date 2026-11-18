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
    latency: Latency,
    start: Instant,
}
impl Fixture {
    fn new() -> Self {
        let start = Instant::now();
        let mut latency = Latency {
            clock: CaptureClock::anchored(EPOCH, start),
            ..Latency::default()
        };
        latency.observe(
            &Event::SelectionChanged(Selection::Selected(connection(1))),
            start,
        );
        Self { latency, start }
    }
    fn now(&self, ms: u64) -> Instant {
        self.start + Duration::from_millis(ms)
    }
    fn event(&mut self, event: Event, ms: u64) {
        self.latency.observe(&event, self.now(ms));
    }
    fn observation(
        direction: Direction,
        sequence: u32,
        data_len: u32,
        ack: Option<u32>,
        ms: u64,
    ) -> TcpObservation {
        TcpObservation {
            connection: connection(1),
            direction,
            packet_number: 1,
            timestamp_ns: Some(EPOCH + i128::from(ms) * 1_000_000),
            sequence,
            acknowledgement: ack,
            data_len,
            flags: 0x10,
            tcp_timestamp: None,
        }
    }
    fn sent(&mut self, sequence: u32, length: u32, ms: u64) {
        self.event(
            Event::Tcp(Self::observation(
                Direction::Outgoing,
                sequence,
                length,
                None,
                ms,
            )),
            ms,
        );
    }
    fn ack(&mut self, ack: u32, ms: u64) {
        self.event(
            Event::Tcp(Self::observation(
                Direction::Incoming,
                100,
                0,
                Some(ack),
                ms,
            )),
            ms,
        );
    }
    fn snapshot(&mut self, ms: u64) -> Snapshot {
        self.latency.snapshot(self.now(ms))
    }
    fn issue(&mut self, ms: u64) {
        self.event(
            Event::Reliability {
                connection: None,
                issue: Reliability::Interrupted("capture interrupted".into()),
            },
            ms,
        );
    }
}
fn millis(value: u64) -> Option<Duration> {
    Some(Duration::from_millis(value))
}

#[test]
fn ping_is_mean_rtt_and_jitter_is_mean_absolute_difference_between_successive_samples() {
    let mut f = Fixture::new();
    assert_eq!(f.snapshot(0).ping, None);
    f.sent(100, 10, 0);
    f.ack(110, 100);
    assert_eq!(f.snapshot(100).ping, millis(100));
    assert_eq!(f.snapshot(100).jitter, None);
    f.sent(110, 10, 200);
    f.ack(120, 400);
    assert_eq!(f.snapshot(400).ping, millis(150));
    assert_eq!(f.snapshot(400).jitter, millis(100));
    f.sent(120, 10, 500);
    f.ack(130, 650);
    let snapshot = f.snapshot(650);
    assert_eq!(snapshot.ping, millis(150));
    assert_eq!(snapshot.jitter, millis(75));
    assert_eq!(snapshot.samples, 3);
    assert!(!snapshot.recovering);
}

#[test]
fn cumulative_ack_samples_newest_fully_acknowledged_data_once_and_ignores_stale_acks() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.sent(110, 10, 20);
    f.sent(120, 10, 40);
    f.ack(125, 100); // Covers two whole segments and part of the third.
    assert_eq!(f.snapshot(100).ping, millis(80));
    assert_eq!(f.snapshot(100).samples, 1);
    f.ack(125, 110);
    f.ack(110, 120);
    assert_eq!(f.snapshot(120).samples, 1);
    f.ack(130, 140);
    assert_eq!(f.snapshot(140).ping, millis(90));
    assert_eq!(f.snapshot(140).jitter, millis(20));
}

#[test]
fn partial_ack_does_not_fabricate_a_sample_and_server_payload_is_not_required() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.ack(105, 10);
    assert_eq!(f.snapshot(10).ping, None);
    f.ack(110, 25);
    assert_eq!(f.snapshot(25).ping, millis(25));
}

#[test]
fn retransmission_taints_entire_cumulative_ack_and_fresh_data_can_recover_without_cooldown() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.sent(110, 10, 10);
    f.sent(100, 10, 20);
    f.sent(120, 10, 30);
    f.ack(130, 100);
    assert_eq!(f.snapshot(100).samples, 0);
    assert!(!f.snapshot(100).recovering); // Retransmission alone is not capture loss.
    f.sent(130, 10, 200);
    f.ack(140, 240);
    assert_eq!(f.snapshot(240).ping, millis(40));
    // A duplicate of already acknowledged data cannot generate another RTT.
    f.sent(100, 10, 250);
    f.sent(140, 10, 260);
    f.ack(150, 300);
    assert_eq!(f.snapshot(300).samples, 2);
}

#[test]
fn partial_retransmission_with_a_new_tail_is_also_ambiguous() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.sent(105, 15, 50);
    f.ack(120, 100);
    assert_eq!(f.snapshot(100).ping, None);
    f.sent(120, 10, 200);
    f.ack(130, 220);
    assert_eq!(f.snapshot(220).ping, millis(20));
}

#[test]
fn sequence_wrap_is_supported_without_sampling_acknowledged_duplicates() {
    let mut f = Fixture::new();
    let start = u32::MAX - 5;
    f.sent(start, 10, 0);
    f.ack(4, 100);
    f.sent(start, 10, 150);
    f.ack(4, 200);
    assert_eq!(f.snapshot(200).samples, 1);
    f.sent(4, 10, 250);
    f.ack(14, 300);
    assert_eq!(f.snapshot(300).ping, millis(75));
}

#[test]
fn absence_of_ack_and_window_expiration_do_not_expire_outstanding_sends() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.ack(110, 100);
    f.sent(110, 10, 200);
    assert_eq!(f.snapshot(10_100).ping, None);
    assert_eq!(f.snapshot(10_100).jitter, None);
    f.ack(120, 12_200);
    assert_eq!(f.snapshot(12_200).ping, millis(12_000));
    assert_eq!(f.snapshot(12_200).jitter, None);
    assert_eq!(f.snapshot(22_200).ping, None);
}

#[test]
fn expiration_never_bridges_jitter_to_an_expired_sample() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.ack(110, 100);
    f.sent(110, 10, 200);
    f.ack(120, 400);
    assert_eq!(f.snapshot(10_100).samples, 1);
    assert_eq!(f.snapshot(10_100).jitter, None);
    f.sent(120, 10, 10_150);
    f.ack(130, 10_200);
    assert_eq!(f.snapshot(10_200).jitter, millis(150));
}

#[test]
fn interruption_requires_a_full_clean_window_and_repeated_faults_restart_it() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.ack(110, 100);
    f.issue(200);
    f.sent(110, 10, 300);
    f.ack(120, 400);
    assert_eq!(f.snapshot(400).ping, None);
    assert!(f.snapshot(10_199).recovering);
    assert_eq!(f.snapshot(10_200).ping, millis(100));
    f.issue(10_300);
    f.issue(10_400);
    assert!(f.snapshot(20_399).recovering);
    assert!(!f.snapshot(20_400).recovering);
    assert_eq!(f.snapshot(20_400).ping, None);
}

#[test]
fn missing_backward_future_and_zero_duration_times_never_supply_rtt() {
    for case in 0..4 {
        let mut f = Fixture::new();
        f.sent(100, 10, 100);
        let mut ack = Fixture::observation(Direction::Incoming, 100, 0, Some(110), 200);
        ack.timestamp_ns = match case {
            0 => None,
            1 => Some(EPOCH + 50_000_000),
            2 => Some(EPOCH + 300_000_000),
            _ => Some(EPOCH + 100_000_000),
        };
        f.event(Event::Tcp(ack), 200);
        assert_eq!(f.snapshot(200).ping, None);
        assert!(f.snapshot(200).recovering);
    }
}

#[test]
fn queued_acks_expire_by_capture_time_and_old_queued_data_cannot_rebuild_recovery() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.event(
        Event::Tcp(Fixture::observation(
            Direction::Incoming,
            100,
            0,
            Some(110),
            100,
        )),
        20_000,
    );
    assert_eq!(f.snapshot(20_000).ping, None);
    f.issue(20_100);
    f.event(
        Event::Tcp(Fixture::observation(
            Direction::Outgoing,
            110,
            10,
            None,
            20_000,
        )),
        20_200,
    );
    assert!(f.latency.pending.is_empty());
}

#[test]
fn reconnection_and_ambiguity_clear_jitter_and_all_outstanding_pairs() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.ack(110, 100);
    f.sent(110, 10, 200);
    f.event(
        Event::SelectionChanged(Selection::Selected(connection(2))),
        250,
    );
    f.ack(120, 300); // Wrong incarnation.
    assert_eq!(f.snapshot(300).ping, None);
    for (direction, sequence, ack, length, ms) in [
        (Direction::Outgoing, 100, None, 10, 400),
        (Direction::Incoming, 0, Some(110), 0, 450),
    ] {
        let mut observation = Fixture::observation(direction, sequence, length, ack, ms);
        observation.connection = connection(2);
        f.event(Event::Tcp(observation), ms);
    }
    assert_eq!(f.snapshot(450).ping, millis(50));
    assert_eq!(f.snapshot(450).jitter, None);
    f.event(Event::SelectionChanged(Selection::Ambiguous), 500);
    assert_eq!(f.snapshot(500).ping, None);
}

#[test]
fn global_fault_survives_selection_reset_but_unrelated_connection_faults_do_not() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.ack(110, 100);
    f.event(
        Event::Reliability {
            connection: Some(connection(2)),
            issue: Reliability::TimestampMissing,
        },
        200,
    );
    assert_eq!(f.snapshot(200).ping, millis(100));
    f.issue(300);
    f.event(Event::SelectionChanged(Selection::Absent), 300);
    f.event(
        Event::SelectionChanged(Selection::Selected(connection(1))),
        400,
    );
    assert!(f.snapshot(10_399).recovering);
    assert!(!f.snapshot(10_400).recovering);
    f.event(
        Event::SelectionChanged(Selection::Selected(connection(2))),
        10_500,
    );
    assert!(!f.snapshot(10_500).recovering);
}

#[test]
fn outgoing_holes_and_acks_covering_unseen_data_trigger_recovery_without_rtt() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.sent(120, 10, 100);
    f.ack(130, 200);
    assert_eq!(f.snapshot(200).ping, None);
    assert!(f.snapshot(200).recovering);
    f.sent(130, 10, 300);
    f.ack(150, 400); // ACK beyond all observed sends.
    assert_eq!(f.snapshot(400).ping, None);
    f.sent(150, 10, 500);
    f.ack(160, 600);
    assert!(f.snapshot(10_399).recovering);
    assert_eq!(f.snapshot(10_400).ping, millis(100));
}

#[test]
fn cadence_and_window_changes_reset_samples_and_pending_without_partial_mutation_on_error() {
    let mut f = Fixture::new();
    assert!(f.latency.snapshot_if_due(f.now(0)).is_some());
    assert!(f.latency.snapshot_if_due(f.now(999)).is_none());
    assert!(f.latency.snapshot_if_due(f.now(1000)).is_some());
    f.sent(100, 10, 1000);
    f.ack(110, 1100);
    assert!(f.latency.set_window(Duration::ZERO, f.now(1200)).is_err());
    assert!(
        f.latency
            .set_window(Duration::from_secs(901), f.now(1200))
            .is_err()
    );
    assert_eq!(f.snapshot(1200).ping, millis(100));
    f.latency
        .set_window(Duration::from_secs(2), f.now(1200))
        .unwrap();
    assert_eq!(f.latency.window(), Duration::from_secs(2));
    assert_eq!(f.snapshot(1200).ping, None);
    assert!(f.latency.snapshot_if_due(f.now(1200)).is_some());
    f.sent(110, 10, 1300);
    f.ack(120, 1400);
    assert_eq!(f.snapshot(1400).ping, millis(100));
    assert_eq!(f.snapshot(3400).ping, None);
}

#[test]
fn pending_and_sample_capacity_fail_explicitly_without_silently_biasing_means() {
    let mut f = Fixture::new();
    for index in 0..=MAX_PENDING {
        f.sent(100 + index as u32, 1, index as u64);
    }
    assert!(f.latency.pending.len() <= MAX_PENDING);
    assert!(f.snapshot(MAX_PENDING as u64).recovering);
    let mut f = Fixture::new();
    for index in 0..=MAX_SAMPLES {
        f.sent(100 + index as u32, 1, 2 * index as u64);
        f.ack(101 + index as u32, 2 * index as u64 + 1);
    }
    assert!(f.latency.samples.len() <= MAX_SAMPLES);
    assert_eq!(f.snapshot(2 * MAX_SAMPLES as u64 + 1).ping, None);
    assert!(f.snapshot(2 * MAX_SAMPLES as u64 + 1).recovering);
}

#[test]
fn raw_observation_pipeline_keeps_a_confirmed_connection_for_an_ack_after_twelve_seconds() {
    use crate::capture::{
        CapturedPacket,
        widgets::tests::{tcp, ticks},
    };
    let mut f = Fixture::new();
    let mut tracker = observations::Tracker::default();
    // Start from Absent so the real tracker drives selection and incarnation.
    f.latency.selection = Selection::Absent;
    let mut number = 0;
    let mut feed = |f: &mut Fixture, mut packet: CapturedPacket, ms: u64| {
        number += 1;
        packet.number = number;
        packet.timestamp_ns = Some(EPOCH + i128::from(ms) * 1_000_000);
        let now = f.now(ms);
        tracker.packet(&packet, now, |event| f.latency.observe(&event, now));
    };
    let mut initial = tcp(100, 0x18, &ticks());
    initial.bytes[28..32].copy_from_slice(&1000u32.to_be_bytes());
    feed(&mut f, initial, 0);
    assert!(matches!(f.snapshot(0).selection, Selection::Selected(_)));
    let mut outgoing = tcp(1000, 0x18, &[0; 10]);
    outgoing.bytes[12..20].copy_from_slice(&[10, 0, 0, 2, 10, 0, 0, 1]);
    outgoing.bytes[20..24].copy_from_slice(&[0x30, 0x39, 0x34, 0x10]);
    feed(&mut f, outgoing, 100);
    assert_eq!(f.snapshot(11_000).ping, None);
    let mut ack = tcp(133, 0x10, &[]);
    ack.bytes[28..32].copy_from_slice(&1010u32.to_be_bytes());
    feed(&mut f, ack, 12_100);
    assert_eq!(f.snapshot(12_100).ping, millis(12_000));
}

#[test]
fn an_old_ack_after_reconnection_or_window_change_cannot_pair_with_a_previous_send() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.latency
        .set_window(Duration::from_secs(2), f.now(100))
        .unwrap();
    f.ack(110, 200);
    assert_eq!(f.snapshot(200).ping, None);
    f.sent(110, 10, 300);
    f.ack(120, 400);
    assert_eq!(f.snapshot(400).ping, millis(100));
    f.event(Event::SelectionChanged(Selection::Absent), 500);
    f.ack(120, 600);
    assert_eq!(f.snapshot(600).ping, None);
}

#[test]
fn syn_fin_sequence_space_and_duplicate_syn_never_create_a_control_packet_rtt() {
    let mut f = Fixture::new();
    let mut syn = Fixture::observation(Direction::Outgoing, 99, 0, None, 0);
    syn.flags = 0x02;
    f.event(Event::Tcp(syn.clone()), 0);
    f.sent(100, 10, 10);
    syn.timestamp_ns = Some(EPOCH + 20_000_000);
    f.event(Event::Tcp(syn), 20);
    f.ack(110, 50);
    assert_eq!(f.snapshot(50).ping, millis(40));
    let mut fin = Fixture::observation(Direction::Outgoing, 110, 0, None, 100);
    fin.flags = 0x11;
    f.event(Event::Tcp(fin), 100);
    f.ack(111, 150);
    assert_eq!(f.snapshot(150).samples, 1);
}

#[test]
fn ambiguous_half_sequence_space_and_outgoing_timestamp_faults_are_rejected() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.ack(110u32.wrapping_add(1 << 31), 100);
    assert_eq!(f.snapshot(100).ping, None);
    assert!(f.snapshot(100).recovering);
    let mut missing = Fixture::observation(Direction::Outgoing, 100, 10, None, 200);
    missing.timestamp_ns = None;
    f.event(Event::Tcp(missing), 200);
    f.ack(110, 300);
    assert_eq!(f.snapshot(300).ping, None);
}

#[test]
fn disconnected_time_cannot_satisfy_the_required_clean_recovery_window() {
    let mut f = Fixture::new();
    f.issue(100);
    f.event(Event::SelectionChanged(Selection::Absent), 100);
    assert!(f.snapshot(20_000).recovering);
    f.event(
        Event::SelectionChanged(Selection::Selected(connection(1))),
        20_000,
    );
    f.sent(100, 10, 20_100);
    f.ack(110, 20_200);
    assert!(f.snapshot(29_999).recovering);
    assert_eq!(f.snapshot(30_000).ping, millis(100));
}

#[test]
fn retransmission_of_an_acknowledged_prefix_can_taint_a_later_cumulative_ack() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.ack(105, 20);
    f.sent(100, 5, 30);
    f.ack(110, 50);
    assert_eq!(f.snapshot(50).ping, None);
}

#[test]
fn a_known_retransmission_cannot_become_a_new_send_after_a_reliability_reset() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.issue(100);
    f.sent(100, 10, 200);
    f.ack(110, 300);
    assert_eq!(f.snapshot(10_200).ping, None);
    f.sent(110, 10, 10_300);
    f.ack(120, 10_400);
    assert_eq!(f.snapshot(10_400).ping, millis(100));
}

#[test]
fn a_brief_spike_remains_visible_as_latest_and_maximum_despite_a_low_mean() {
    let mut f = Fixture::new();
    for index in 0..99 {
        f.sent(100 + index * 10, 10, u64::from(index) * 20);
        f.ack(110 + index * 10, u64::from(index) * 20 + 15);
    }
    f.sent(1090, 10, 1980);
    f.ack(1100, 2980);
    let snapshot = f.snapshot(2980);
    assert_eq!(snapshot.samples, 100);
    assert_eq!(snapshot.ping, Some(Duration::from_micros(24_850)));
    assert_eq!(snapshot.last_rtt, millis(1000));
    assert_eq!(snapshot.max_rtt, millis(1000));
    assert_eq!(snapshot.last_sample_age, Some(Duration::ZERO));
    // While another send waits, the old RTT is explicitly aged, never raised
    // to the wait duration as though an ACK had already confirmed it.
    f.sent(1100, 10, 3000);
    let waiting = f.snapshot(4500);
    assert_eq!(waiting.ping, snapshot.ping);
    assert_eq!(waiting.last_rtt, millis(1000));
    assert_eq!(waiting.last_sample_age, millis(1520));
    assert_eq!(waiting.awaiting_ack, millis(1500));
    f.ack(1105, 4600); // A partial ACK leaves data awaiting confirmation.
    assert_eq!(f.snapshot(4700).awaiting_ack, millis(1700));
    f.ack(1110, 5000);
    let resumed = f.snapshot(5000);
    assert_eq!(resumed.last_rtt, millis(2000));
    assert_eq!(resumed.max_rtt, millis(2000));
    assert_eq!(resumed.awaiting_ack, None);
}

#[test]
fn last_rtt_keeps_its_age_after_window_expiration_but_the_recent_maximum_expires() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.ack(110, 1000);
    let expired = f.snapshot(11_000);
    assert_eq!(expired.ping, None);
    assert_eq!(expired.jitter, None);
    assert_eq!(expired.max_rtt, None);
    assert_eq!(expired.last_rtt, millis(1000));
    assert_eq!(expired.last_sample_age, millis(10_000));
    assert_eq!(expired.samples, 0);
    assert_eq!(expired.awaiting_ack, None);
}

#[test]
fn latency_diagnostics_are_cleared_on_reconnection_window_edit_and_capture_fault() {
    for reset in 0..3 {
        let mut f = Fixture::new();
        f.sent(100, 10, 0);
        f.ack(110, 100);
        f.sent(110, 10, 200);
        match reset {
            0 => f.event(
                Event::SelectionChanged(Selection::Selected(connection(2))),
                300,
            ),
            1 => f
                .latency
                .set_window(Duration::from_secs(2), f.now(300))
                .unwrap(),
            _ => f.issue(300),
        }
        let snapshot = f.snapshot(400);
        assert_eq!(snapshot.ping, None);
        assert_eq!(snapshot.last_rtt, None);
        assert_eq!(snapshot.max_rtt, None);
        assert_eq!(snapshot.last_sample_age, None);
        assert_eq!(snapshot.awaiting_ack, None);
    }
}

#[test]
fn ambiguous_retransmissions_can_show_an_ack_wait_but_never_a_fabricated_latest_rtt() {
    let mut f = Fixture::new();
    f.sent(100, 10, 0);
    f.sent(100, 10, 400);
    let snapshot = f.snapshot(600);
    assert_eq!(snapshot.last_rtt, None);
    assert_eq!(snapshot.max_rtt, None);
    assert_eq!(snapshot.awaiting_ack, millis(600));
    f.ack(110, 1000);
    let snapshot = f.snapshot(1000);
    assert_eq!(snapshot.last_rtt, None);
    assert_eq!(snapshot.last_sample_age, None);
    assert_eq!(snapshot.awaiting_ack, None);
}
