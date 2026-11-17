use super::*;
const EPOCH: i128 = 1_000_000_000_000;
struct Fixture {
    metric: Throughput,
    start: Instant,
}
impl Fixture {
    fn new() -> Self {
        let start = Instant::now();
        Self {
            metric: Throughput {
                clock: CaptureClock::anchored(EPOCH, start),
                ..Throughput::default()
            },
            start,
        }
    }
    fn now(&self, ms: u64) -> Instant {
        self.start + Duration::from_millis(ms)
    }
    fn event(&mut self, event: Event, ms: u64) {
        self.metric.observe(&event, self.now(ms));
    }
    fn available(&mut self, available: bool, ms: u64) {
        self.event(Event::CaptureState { available }, ms);
    }
    fn traffic(&mut self, direction: Direction, bytes: u32, capture_ms: u64, processing_ms: u64) {
        self.event(
            Event::Traffic {
                direction,
                captured_bytes: bytes,
                timestamp_ns: Some(EPOCH + i128::from(capture_ms) * 1_000_000),
            },
            processing_ms,
        );
    }
    fn snapshot(&mut self, ms: u64) -> Snapshot {
        self.metric.snapshot(self.now(ms))
    }
    fn rates(&mut self, ms: u64) -> Rates {
        self.snapshot(ms).rates.expect("available rates")
    }
}
#[test]
fn idle_capture_is_zero_but_unknown_or_unavailable_capture_is_not() {
    let mut f = Fixture::new();
    assert!(f.snapshot(0).rates.is_none());
    f.available(true, 0);
    assert_eq!(
        f.rates(0),
        Rates {
            incoming_bytes_per_second: 0.0,
            outgoing_bytes_per_second: 0.0
        }
    );
    assert_eq!(f.rates(15_000).incoming_bytes_per_second, 0.0);
    f.available(false, 15_001);
    assert!(f.snapshot(30_000).rates.is_none());
}
#[test]
fn directions_and_units_use_actual_partial_duration_then_the_full_window() {
    let mut f = Fixture::new();
    f.available(true, 0);
    f.traffic(Direction::Incoming, 2048, 100, 100);
    f.traffic(Direction::Outgoing, 1024, 200, 200);
    assert_eq!(
        f.rates(1000),
        Rates {
            incoming_bytes_per_second: 2048.0,
            outgoing_bytes_per_second: 1024.0
        }
    );
    assert_eq!(f.rates(5000).incoming_bytes_per_second, 409.6);
    assert_eq!(f.rates(10_000).incoming_bytes_per_second, 204.8);
    assert_eq!(f.rates(10_100).incoming_bytes_per_second, 0.0);
    assert_eq!(f.rates(10_100).outgoing_bytes_per_second, 102.4);
    assert_eq!(f.rates(10_200).outgoing_bytes_per_second, 0.0);
}
#[test]
fn queued_and_reordered_packets_expire_by_capture_time_not_delivery_order() {
    let mut f = Fixture::new();
    f.available(true, 0);
    f.traffic(Direction::Incoming, 100, 500, 1000);
    f.traffic(Direction::Incoming, 200, 100, 1000);
    assert_eq!(f.rates(1000).incoming_bytes_per_second, 300.0);
    assert_eq!(f.rates(10_100).incoming_bytes_per_second, 10.0);
    f.traffic(Direction::Outgoing, 200, 0, 10_100);
    assert_eq!(f.rates(10_100).outgoing_bytes_per_second, 0.0);
}
#[test]
fn selected_game_changes_and_candidate_capacity_do_not_change_process_rates() {
    let mut f = Fixture::new();
    f.available(true, 0);
    f.traffic(Direction::Incoming, 100, 0, 0);
    f.event(
        Event::SelectionChanged(super::super::observations::Selection::Ambiguous),
        100,
    );
    f.event(
        Event::Reliability {
            connection: None,
            issue: Reliability::CapacityExceeded,
        },
        200,
    );
    assert_eq!(f.rates(1000).incoming_bytes_per_second, 100.0);
}
#[test]
fn continuity_fault_waits_a_clean_window_and_disconnected_time_does_not_count() {
    let mut f = Fixture::new();
    f.available(true, 0);
    f.event(
        Event::Reliability {
            connection: None,
            issue: Reliability::PacketNumberDiscontinuity,
        },
        100,
    );
    f.traffic(Direction::Incoming, 100, 200, 200);
    assert!(f.snapshot(10_099).rates.is_none());
    assert_eq!(f.rates(10_100).incoming_bytes_per_second, 10.0);
    f.available(false, 10_200);
    assert!(f.snapshot(30_000).rates.is_none());
    f.available(true, 30_000);
    f.traffic(Direction::Incoming, 100, 30_100, 30_100);
    assert!(f.snapshot(39_999).rates.is_none());
    assert_eq!(f.rates(40_000).incoming_bytes_per_second, 10.0);
}
#[test]
fn missing_or_future_timestamps_invalidate_and_old_backlog_cannot_rebuild_recovery() {
    for timestamp in [None, Some(-1), Some(EPOCH + 2_000_000_000)] {
        let mut f = Fixture::new();
        f.available(true, 0);
        f.event(
            Event::Traffic {
                direction: Direction::Incoming,
                captured_bytes: 100,
                timestamp_ns: timestamp,
            },
            1000,
        );
        assert!(f.snapshot(1000).rates.is_none());
        f.traffic(Direction::Incoming, 100, 500, 1500);
        assert!(f.metric.bytes.is_empty());
        assert_eq!(f.rates(11_000).incoming_bytes_per_second, 0.0);
    }
}
#[test]
fn window_change_only_clears_traffic_and_same_or_invalid_value_is_a_noop() {
    let mut f = Fixture::new();
    f.available(true, 0);
    f.traffic(Direction::Incoming, 100, 100, 100);
    assert!(f.metric.set_window(Duration::ZERO, f.now(200)).is_err());
    assert!(
        f.metric
            .set_window(Duration::from_secs(901), f.now(200))
            .is_err()
    );
    f.metric.set_window(DEFAULT_WINDOW, f.now(200)).unwrap();
    assert_eq!(f.rates(1000).incoming_bytes_per_second, 100.0);
    f.metric
        .set_window(Duration::from_secs(2), f.now(1000))
        .unwrap();
    assert_eq!(f.rates(1000).incoming_bytes_per_second, 0.0);
    f.traffic(Direction::Outgoing, 200, 1100, 1100);
    assert_eq!(f.rates(2000).outgoing_bytes_per_second, 200.0);
    assert_eq!(f.rates(3100).outgoing_bytes_per_second, 0.0);
}
#[test]
fn a_nonempty_zero_duration_is_unavailable_instead_of_an_infinite_rate() {
    let mut f = Fixture::new();
    f.available(true, 0);
    f.traffic(Direction::Incoming, 100, 0, 0);
    assert!(f.snapshot(0).rates.is_none());
    assert_eq!(f.rates(1000).incoming_bytes_per_second, 100.0);
}
#[test]
fn timestamp_capacity_invalidates_instead_of_biasing_the_rate() {
    let mut f = Fixture::new();
    f.available(true, 0);
    for index in 0..=MAX_TIMESTAMPS {
        let now = f.start + Duration::from_nanos(index as u64);
        f.metric.observe(
            &Event::Traffic {
                direction: Direction::Incoming,
                captured_bytes: 1,
                timestamp_ns: Some(EPOCH + index as i128),
            },
            now,
        );
    }
    assert!(f.snapshot(1).rates.is_none());
    assert!(f.metric.bytes.len() <= MAX_TIMESTAMPS);
    assert_eq!(f.rates(10_001).incoming_bytes_per_second, 0.0);
}
