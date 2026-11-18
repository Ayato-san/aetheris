use super::*;
use crate::capture::widgets::tests::{tcp, ticks};
use std::sync::{atomic::AtomicUsize, mpsc};

fn healthy(packets: Vec<crate::capture::CapturedPacket>) -> Arc<CaptureBatch> {
    Arc::new(CaptureBatch {
        packets,
        status: "capture active".into(),
        interruption: None,
        finished: false,
        error: None,
    })
}
fn packet(
    number: u64,
    outgoing: bool,
    sequence: u32,
    payload: &[u8],
) -> crate::capture::CapturedPacket {
    let mut packet = tcp(sequence, 0x18, payload);
    packet.number = number;
    packet.timestamp_ns = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i128,
    );
    if outgoing {
        packet.bytes[12..20].copy_from_slice(&[10, 0, 0, 2, 10, 0, 0, 1]);
        packet.bytes[20..24].copy_from_slice(&[0x30, 0x39, 0x34, 0x10]);
    }
    packet
}
fn configuration(interests: Interests) -> Configuration {
    Configuration {
        windows: Windows::default(),
        interests,
    }
}
fn wait_until(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(4);
    while !predicate() {
        assert!(
            Instant::now() < deadline,
            "worker did not reach expected state"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
struct ChannelSource {
    receive: Arc<Mutex<mpsc::Receiver<Arc<CaptureBatch>>>>,
    active: Arc<AtomicUsize>,
}
impl BatchSource for ChannelSource {
    fn next_batch(&mut self) -> Result<Option<Arc<CaptureBatch>>> {
        match self
            .receive
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_millis(20))
        {
            Ok(batch) => Ok(Some(batch)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(_) => Err("source closed".into()),
        }
    }
}
impl Drop for ChannelSource {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::AcqRel);
    }
}
struct Host {
    coordinator: Arc<Mutex<Coordinator>>,
    connector: Connector,
    send: mpsc::SyncSender<Arc<CaptureBatch>>,
    connects: Arc<AtomicUsize>,
    active: Arc<AtomicUsize>,
}
impl Host {
    fn new() -> Self {
        let (send, receive) = mpsc::sync_channel(8);
        let receive = Arc::new(Mutex::new(receive));
        let connects = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        let connector = {
            let connects = connects.clone();
            let active = active.clone();
            Arc::new(move || {
                connects.fetch_add(1, Ordering::AcqRel);
                active.fetch_add(1, Ordering::AcqRel);
                Ok(Box::new(ChannelSource {
                    receive: receive.clone(),
                    active: active.clone(),
                }) as Box<dyn BatchSource>)
            }) as Connector
        };
        Self {
            coordinator: Arc::new(Mutex::new(Coordinator::default())),
            connector,
            send,
            connects,
            active,
        }
    }
    fn subscribe(&self, interests: Interests) -> Subscription {
        subscribe_with(self.coordinator.clone(), interests, self.connector.clone()).unwrap()
    }
}

#[test]
fn no_traffic_can_be_zero_without_a_selected_game_connection_but_not_before_a_real_batch() {
    let now = Instant::now();
    let mut aggregates = Aggregates::new(configuration(Interests::ALL), now);
    assert!(aggregates.snapshot(now).traffic.rates.is_none());
    aggregates.batch(&healthy(vec![]), now);
    let snapshot = aggregates.snapshot(now + Duration::from_secs(1));
    assert!(snapshot.capture_available);
    assert_eq!(
        snapshot.traffic.rates.unwrap().incoming_bytes_per_second,
        0.0
    );
    assert_eq!(snapshot.latency.ping, None);
    assert!(snapshot.retransmissions.counts.is_none());
    assert_eq!(snapshot.collector_drops, None);
}
#[test]
fn traffic_includes_non_selected_tcp_and_udp_and_does_not_require_protocol_recognition() {
    let now = Instant::now();
    let mut aggregates = Aggregates::new(configuration(Interests::TRAFFIC), now);
    let mut udp = packet(2, false, 0, &[]);
    udp.bytes.truncate(28);
    udp.bytes[9] = 17;
    udp.bytes[2..4].copy_from_slice(&28u16.to_be_bytes());
    udp.bytes[24..26].copy_from_slice(&8u16.to_be_bytes());
    udp.captured_len = 28;
    udp.original_len = 28;
    let incoming = packet(1, false, 10, &[0; 60]);
    let outgoing = packet(3, true, 20, &[0; 60]);
    let mut packets = vec![incoming, udp, outgoing];
    let timestamp = packets[2].timestamp_ns;
    for packet in &mut packets {
        packet.timestamp_ns = timestamp;
    }
    aggregates.batch(&healthy(packets), now + Duration::from_secs(1));
    let snapshot = aggregates.snapshot(now + Duration::from_secs(2));
    let rates = snapshot.traffic.rates.unwrap();
    assert!(rates.incoming_bytes_per_second > rates.outgoing_bytes_per_second);
    assert!(
        (rates.incoming_bytes_per_second / rates.outgoing_bytes_per_second - 1.28).abs() < 1e-9
    );
    assert!(matches!(
        snapshot.latency.selection,
        observations::Selection::Absent
    ));
    assert!(aggregates.tracker.selection() == &observations::Selection::Absent);
}
#[test]
fn changing_only_traffic_window_preserves_latency_and_retransmission_history() {
    let now = Instant::now();
    let mut aggregates = Aggregates::new(configuration(Interests::ALL), now);
    let mut first = packet(1, false, 100, &ticks());
    first.bytes[28..32].copy_from_slice(&1000u32.to_be_bytes());
    let base = first.timestamp_ns.unwrap();
    aggregates.batch(&healthy(vec![first]), now + Duration::from_secs(1));
    let mut outgoing = packet(2, true, 1000, &[0; 10]);
    outgoing.timestamp_ns = Some(base + 1_100_000_000);
    aggregates.batch(&healthy(vec![outgoing]), now + Duration::from_secs(2));
    let mut ack = packet(3, false, 133, &[]);
    ack.timestamp_ns = Some(base + 1_200_000_000);
    ack.bytes[28..32].copy_from_slice(&1010u32.to_be_bytes());
    aggregates.batch(&healthy(vec![ack]), now + Duration::from_secs(2));
    let before = aggregates.snapshot(now + Duration::from_secs(3));
    assert!(before.latency.ping.is_some());
    let mut config = configuration(Interests::ALL);
    config.windows.traffic = Duration::from_secs(2);
    aggregates.configure(config, now + Duration::from_secs(3));
    let after = aggregates.snapshot(now + Duration::from_secs(3));
    assert_eq!(before.latency.ping, after.latency.ping);
    assert_eq!(before.retransmissions.counts, after.retransmissions.counts);
    assert!(after.traffic.rates.is_none());
    config.interests.traffic = false;
    aggregates.configure(config, now + Duration::from_secs(4));
    assert!(
        aggregates
            .snapshot(now + Duration::from_secs(3))
            .traffic
            .rates
            .is_none()
    );
    assert_eq!(
        aggregates
            .snapshot(now + Duration::from_secs(3))
            .latency
            .ping,
        before.latency.ping
    );
}
#[test]
fn suspended_capture_and_recovery_never_publish_a_false_zero() {
    let now = Instant::now();
    let mut config = configuration(Interests::ALL);
    config.windows.traffic = Duration::from_secs(2);
    let mut aggregates = Aggregates::new(config, now);
    aggregates.batch(&healthy(vec![]), now);
    let paused = CaptureBatch {
        packets: vec![],
        status: "suspended".into(),
        interruption: Some("ambiguous process".into()),
        finished: false,
        error: None,
    };
    aggregates.batch(&paused, now + Duration::from_secs(1));
    assert!(
        aggregates
            .snapshot(now + Duration::from_secs(10))
            .traffic
            .rates
            .is_none()
    );
    aggregates.batch(&healthy(vec![]), now + Duration::from_secs(10));
    assert!(
        aggregates
            .snapshot(now + Duration::from_millis(11_999))
            .traffic
            .rates
            .is_none()
    );
    assert_eq!(
        aggregates
            .snapshot(now + Duration::from_secs(12))
            .traffic
            .rates
            .unwrap()
            .incoming_bytes_per_second,
        0.0
    );
}
#[test]
fn multiple_widgets_share_one_acquisition_and_one_counter_and_release_the_last_owner() {
    let host = Host::new();
    let first = host.subscribe(Interests::ALL);
    let second = host.subscribe(Interests::RETRANSMISSIONS);
    let clone = first.clone();
    assert!(Arc::ptr_eq(&first.hub, &second.hub));
    host.send
        .send(healthy(vec![packet(1, false, 100, &ticks())]))
        .unwrap();
    wait_until(|| {
        first
            .snapshot()
            .retransmissions
            .counts
            .as_ref()
            .is_some_and(|counts| counts.data_segments == 1)
    });
    assert_eq!(host.connects.load(Ordering::Acquire), 1);
    assert_eq!(
        second
            .snapshot()
            .retransmissions
            .counts
            .as_ref()
            .unwrap()
            .data_segments,
        1
    );
    drop(first);
    drop(clone);
    wait_until(|| second.snapshot().active == Interests::RETRANSMISSIONS);
    assert!(second.snapshot().traffic.rates.is_none());
    assert_eq!(host.active.load(Ordering::Acquire), 1);
    let weak = Arc::downgrade(&second.hub);
    drop(second);
    wait_until(|| host.active.load(Ordering::Acquire) == 0 && weak.upgrade().is_none());
    let fresh = host.subscribe(Interests::ALL);
    assert!(fresh.snapshot().retransmissions.counts.is_none());
    assert!(fresh.snapshot().traffic.rates.is_none());
    drop(fresh);
    wait_until(|| host.active.load(Ordering::Acquire) == 0);
}
#[test]
fn windows_are_global_validated_atomically_and_survive_a_collection_restart() {
    let host = Host::new();
    let first = host.subscribe(Interests::TRAFFIC);
    let second = host.subscribe(Interests::TRAFFIC);
    let windows = Windows {
        traffic: Duration::from_secs(2),
        ..Windows::default()
    };
    first.set_windows(windows).unwrap();
    assert_eq!(second.windows(), windows);
    host.send.send(healthy(vec![])).unwrap();
    wait_until(|| second.snapshot().windows == windows);
    let mut invalid = windows;
    invalid.latency = Duration::ZERO;
    invalid.traffic = Duration::from_secs(3);
    assert!(second.set_windows(invalid).is_err());
    assert_eq!(first.windows(), windows);
    drop(first);
    drop(second);
    wait_until(|| host.active.load(Ordering::Acquire) == 0);
    let new = host.subscribe(Interests::TRAFFIC);
    assert_eq!(new.snapshot().windows, windows);
    assert!(new.snapshot().traffic.rates.is_none());
}
#[test]
fn an_empty_interest_does_not_start_acquisition_and_failures_stay_unavailable() {
    let host = Host::new();
    assert!(
        subscribe_with(
            host.coordinator.clone(),
            Interests::default(),
            host.connector.clone()
        )
        .is_err()
    );
    assert_eq!(host.connects.load(Ordering::Acquire), 0);
    let errors = Arc::new(AtomicUsize::new(0));
    let connector = {
        let errors = errors.clone();
        Arc::new(move || {
            errors.fetch_add(1, Ordering::AcqRel);
            Err("service unavailable".into())
        }) as Connector
    };
    let subscription = subscribe_with(host.coordinator.clone(), Interests::ALL, connector).unwrap();
    wait_until(|| subscription.snapshot().error.is_some());
    assert!(subscription.snapshot().traffic.rates.is_none());
    assert!(errors.load(Ordering::Acquire) > 0);
    let weak = Arc::downgrade(&subscription.hub);
    drop(subscription);
    wait_until(|| weak.upgrade().is_none());
}
#[test]
fn a_missing_batch_heartbeat_invalidates_zero_and_recovery_waits_for_a_full_window() {
    let host = Host::new();
    let subscription = host.subscribe(Interests::TRAFFIC);
    let windows = Windows {
        traffic: Duration::from_millis(50),
        ..Windows::default()
    };
    subscription.set_windows(windows).unwrap();
    host.send.send(healthy(vec![])).unwrap();
    wait_until(|| subscription.snapshot().traffic.rates.is_some());
    wait_until(|| !subscription.snapshot().capture_available);
    assert!(subscription.snapshot().traffic.rates.is_none());
    host.send.send(healthy(vec![])).unwrap();
    wait_until(|| subscription.snapshot().capture_available);
    wait_until(|| subscription.snapshot().traffic.rates.is_some());
    assert_eq!(host.connects.load(Ordering::Acquire), 1);
}
#[test]
fn a_terminal_source_reconnects_while_widgets_remain_active() {
    let host = Host::new();
    let subscription = host.subscribe(Interests::TRAFFIC);
    host.send.send(healthy(vec![])).unwrap();
    wait_until(|| subscription.snapshot().capture_available);
    host.send
        .send(Arc::new(CaptureBatch {
            packets: vec![],
            status: "failed".into(),
            interruption: None,
            finished: true,
            error: Some("service lost".into()),
        }))
        .unwrap();
    wait_until(|| subscription.snapshot().error.as_deref() == Some("service lost"));
    assert!(subscription.snapshot().traffic.rates.is_none());
    wait_until(|| host.connects.load(Ordering::Acquire) == 2);
    host.send.send(healthy(vec![])).unwrap();
    wait_until(|| subscription.snapshot().capture_available);
    assert!(subscription.snapshot().traffic.recovering);
    assert!(subscription.snapshot().traffic.rates.is_none());
}

#[test]
fn overlay_composition_shares_collection_and_keeps_history_when_reordered_or_duplicated() {
    use crate::overlay::network::{Measure, Runtime, Settings};
    let host = Host::new();
    let mut runtime = Runtime::default();
    let mut settings = Settings::default();
    runtime
        .configure_with(&settings, |i| Ok(host.subscribe(i)))
        .unwrap();
    assert_eq!(host.connects.load(Ordering::Acquire), 0);
    settings.individual[4] = true;
    settings.grouped = true;
    settings.measures = vec![Measure::Incoming, Measure::Retransmissions];
    runtime
        .configure_with(&settings, |i| Ok(host.subscribe(i)))
        .unwrap();
    host.send
        .send(healthy(vec![packet(1, false, 100, &ticks())]))
        .unwrap();
    wait_until(|| {
        runtime
            .frames()
            .iter()
            .all(|f| f.text.contains("Retransmissions TCP : 0 · 0.0 %"))
    });
    assert_eq!(runtime.frames().len(), 2);
    assert_eq!(host.connects.load(Ordering::Acquire), 1);
    settings.measures.reverse();
    runtime
        .configure_with(&settings, |i| Ok(host.subscribe(i)))
        .unwrap();
    assert!(
        runtime.frames()[1]
            .text
            .starts_with("Retransmissions TCP : 0 · 0.0 %")
    );
    settings.individual[4] = false;
    runtime
        .configure_with(&settings, |i| Ok(host.subscribe(i)))
        .unwrap();
    assert_eq!(host.active.load(Ordering::Acquire), 1);
    assert_eq!(runtime.frames().len(), 1);
    assert!(
        runtime.frames()[0]
            .text
            .contains("Retransmissions TCP : 0 · 0.0 %")
    );
    // Changing interests must acquire before releasing the previous subscription.
    settings.measures = vec![Measure::Retransmissions];
    runtime
        .configure_with(&settings, |i| Ok(host.subscribe(i)))
        .unwrap();
    assert!(
        runtime.frames()[0]
            .text
            .contains("Retransmissions TCP : 0 · 0.0 %")
    );
    assert_eq!(host.connects.load(Ordering::Acquire), 1);
    settings.measures.clear();
    runtime
        .configure_with(&settings, |i| Ok(host.subscribe(i)))
        .unwrap();
    assert!(runtime.frames().is_empty());
    wait_until(|| host.active.load(Ordering::Acquire) == 0);
    // A later activation must begin without values from the previous session.
    settings.individual[4] = true;
    runtime
        .configure_with(&settings, |i| Ok(host.subscribe(i)))
        .unwrap();
    assert_eq!(runtime.frames()[0].text, "Retransmissions TCP : —");
    drop(runtime);
    wait_until(|| host.active.load(Ordering::Acquire) == 0);
}

#[test]
fn overlay_frames_follow_silence_interruption_recovery_and_reconnection() {
    use crate::overlay::network::{Measure, frame};
    let now = Instant::now();
    let mut config = configuration(Interests::ALL);
    config.windows.traffic = Duration::from_secs(2);
    let mut aggregates = Aggregates::new(config, now);
    let render = |aggregates: &mut Aggregates, elapsed| {
        frame(
            &Measure::ALL,
            Some(&aggregates.snapshot(now + Duration::from_secs(elapsed))),
        )
        .text
    };
    aggregates.batch(
        &healthy(vec![packet(1, false, 100, &ticks())]),
        now + Duration::from_secs(1),
    );
    assert!(render(&mut aggregates, 1).contains("Retransmissions TCP : 0 · 0.0 %"));
    // Healthy silence does not invent an RTT or imply that capture is broken.
    aggregates.batch(&healthy(vec![]), now + Duration::from_secs(1));
    let silent = render(&mut aggregates, 1);
    assert!(silent.starts_with("Ping : moy. — · dernier — · max —\n"));
    assert!(silent.contains("Jitter : —"));
    let interruption = CaptureBatch {
        packets: vec![],
        status: "paused".into(),
        interruption: Some("capture gap".into()),
        finished: false,
        error: None,
    };
    aggregates.batch(&interruption, now + Duration::from_secs(2));
    assert_eq!(render(&mut aggregates, 2), frame(&Measure::ALL, None).text);
    aggregates.batch(&healthy(vec![]), now + Duration::from_secs(3));
    assert!(render(&mut aggregates, 4).contains("Débit entrant : —"));
    assert!(render(&mut aggregates, 5).contains("Débit entrant : 0 o/s"));
    aggregates.interrupt("reconnected".into(), now + Duration::from_secs(6));
    assert_eq!(render(&mut aggregates, 6), frame(&Measure::ALL, None).text);
}

#[test]
fn debug_window_edits_reset_only_affected_metrics_until_new_observations() {
    let now = Instant::now();
    let mut aggregates = Aggregates::new(configuration(Interests::ALL), now);
    let mut first = packet(1, false, 100, &ticks());
    first.bytes[28..32].copy_from_slice(&1000u32.to_be_bytes());
    let base = first.timestamp_ns.unwrap();
    aggregates.batch(&healthy(vec![first]), now + Duration::from_secs(1));
    let mut outgoing = packet(2, true, 1000, &[0; 10]);
    outgoing.timestamp_ns = Some(base + 1_100_000_000);
    let mut ack = packet(3, false, 133, &[]);
    ack.timestamp_ns = Some(base + 1_200_000_000);
    ack.bytes[28..32].copy_from_slice(&1010u32.to_be_bytes());
    aggregates.batch(&healthy(vec![outgoing, ack]), now + Duration::from_secs(2));
    let mut outgoing = packet(4, true, 1010, &[0; 10]);
    outgoing.timestamp_ns = Some(base + 2_100_000_000);
    let mut ack = packet(5, false, 133, &[]);
    ack.timestamp_ns = Some(base + 2_300_000_000);
    ack.bytes[28..32].copy_from_slice(&1020u32.to_be_bytes());
    aggregates.batch(&healthy(vec![outgoing, ack]), now + Duration::from_secs(3));
    let before = aggregates.snapshot(now + Duration::from_secs(4));
    assert_eq!(before.latency.ping, Some(Duration::from_millis(150)));
    assert_eq!(before.latency.jitter, Some(Duration::from_millis(100)));
    assert!(before.traffic.rates.is_some());
    let mut config = configuration(Interests::ALL);
    config.windows.latency = Duration::from_secs(20);
    config.windows.traffic = Duration::from_secs(20);
    aggregates.configure(config, now + Duration::from_secs(4));
    let reset = aggregates.snapshot(now + Duration::from_secs(4));
    assert!(reset.latency.ping.is_none() && reset.latency.jitter.is_none());
    assert!(reset.traffic.rates.is_none());
    assert_eq!(reset.retransmissions.counts, before.retransmissions.counts);
    // A new healthy empty batch suffices to confirm a zero rate after reset.
    aggregates.batch(&healthy(vec![]), now + Duration::from_millis(4500));
    assert_eq!(
        aggregates
            .snapshot(now + Duration::from_millis(4500))
            .traffic
            .rates
            .unwrap()
            .incoming_bytes_per_second,
        0.
    );
    let mut outgoing = packet(6, true, 1020, &[0; 10]);
    outgoing.timestamp_ns = Some(base + 4_100_000_000);
    let mut ack = packet(7, false, 133, &[]);
    ack.timestamp_ns = Some(base + 4_300_000_000);
    ack.bytes[28..32].copy_from_slice(&1030u32.to_be_bytes());
    aggregates.batch(&healthy(vec![outgoing, ack]), now + Duration::from_secs(5));
    let fresh = aggregates.snapshot(now + Duration::from_secs(6));
    assert_eq!(fresh.latency.ping, Some(Duration::from_millis(200)));
    assert!(fresh.latency.jitter.is_none()); // No bridge to samples before reset.
    config.windows.retransmissions = Duration::from_secs(1200);
    aggregates.configure(config, now + Duration::from_secs(6));
    let reset = aggregates.snapshot(now + Duration::from_secs(6));
    assert_eq!(reset.latency.ping, fresh.latency.ping);
    assert_eq!(reset.traffic.rates, fresh.traffic.rates);
    assert!(reset.retransmissions.counts.is_none());
    aggregates.batch(&healthy(vec![]), now + Duration::from_millis(6500));
    assert!(
        aggregates
            .snapshot(now + Duration::from_millis(6500))
            .retransmissions
            .counts
            .is_none()
    );
    let mut outgoing = packet(8, true, 1030, &[0; 10]);
    outgoing.timestamp_ns = Some(base + 6_100_000_000);
    aggregates.batch(&healthy(vec![outgoing]), now + Duration::from_secs(7));
    let counts = aggregates
        .snapshot(now + Duration::from_secs(7))
        .retransmissions
        .counts
        .unwrap();
    assert_eq!(counts.data_segments, 1);
    assert_eq!(counts.retransmissions, 0);
    // Restoring defaults follows exactly the same invalidation path.
    aggregates.configure(configuration(Interests::ALL), now + Duration::from_secs(8));
    let restored = aggregates.snapshot(now + Duration::from_secs(8));
    assert_eq!(restored.windows, Windows::default());
    assert!(
        restored.latency.ping.is_none()
            && restored.traffic.rates.is_none()
            && restored.retransmissions.counts.is_none()
    );
}
