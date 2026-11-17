use super::*;
use crate::capture::widgets::tests::{tcp, ticks};

fn feed(tracker: &mut Tracker, mut packet: CapturedPacket, now: Instant) -> Vec<Event> {
    packet.number = tracker.last_number.unwrap_or(0) + 1;
    let mut events = vec![];
    tracker.packet(&packet, now, |event| events.push(event));
    events
}
fn has_issue(events: &[Event], expected: Reliability) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::Reliability { issue, .. } if issue == &expected))
}
fn selected(tracker: &Tracker) -> Connection {
    match tracker.selection() {
        Selection::Selected(connection) => *connection,
        selection => panic!("expected a selected connection, got {selection:?}"),
    }
}

#[test]
fn ack_sequence_flags_options_and_direction_are_preserved_without_inventing_time() {
    let now = Instant::now();
    let mut tracker = Tracker::default();
    let mut packet = tcp(u32::MAX, 0xd8, &[1, 2, 3]);
    packet.bytes[28..32].copy_from_slice(&1234u32.to_be_bytes());
    packet
        .bytes
        .splice(40..40, [1, 1, 8, 10, 0, 0, 0, 42, 0, 0, 0, 19]);
    packet.bytes[32] = 0x80;
    let length = packet.bytes.len() as u16;
    packet.bytes[2..4].copy_from_slice(&length.to_be_bytes());
    packet.captured_len = length as u32;
    packet.original_len = length as u32;
    let events = feed(&mut tracker, packet.clone(), now);
    assert!(has_issue(&events, Reliability::TimestampMissing));
    let Event::Tcp(observation) = events.last().unwrap() else {
        panic!()
    };
    assert_eq!(observation.direction, Direction::Incoming);
    assert_eq!(observation.sequence, u32::MAX);
    assert_eq!(observation.acknowledgement, Some(1234));
    assert_eq!(observation.data_len, 3);
    assert_eq!(observation.flags, 0xd8);
    assert_eq!(observation.timestamp_ns, None);
    assert_eq!(
        observation.tcp_timestamp,
        Some(network::TcpTimestamp {
            value: 42,
            echo_reply: 19
        })
    );
    packet.local_address = Some("10.0.0.1".parse().unwrap());
    packet.bytes[33] = 0x08;
    let events = feed(&mut tracker, packet, now);
    let Event::Tcp(observation) = events.last().unwrap() else {
        panic!()
    };
    assert_eq!(observation.direction, Direction::Outgoing);
    assert_eq!(observation.acknowledgement, None);
    assert_eq!(tracker.selection(), &Selection::Absent);
    assert_eq!(tracker.collector_drops(), None);
}

#[test]
fn game_recognition_is_required_and_two_confirmed_connections_are_ambiguous() {
    let now = Instant::now();
    let mut tracker = Tracker::default();
    feed(&mut tracker, tcp(100, 0x18, &[1, 2, 3]), now);
    assert_eq!(tracker.selection(), &Selection::Absent);
    let mut second = tcp(100, 0x18, &ticks());
    second.bytes[12] = 11;
    feed(&mut tracker, second.clone(), now);
    let connection = selected(&tracker);
    assert_eq!(connection.remote.ip().to_string(), "11.0.0.1");
    let mut third = second.clone();
    third.bytes[12] = 12;
    let events = feed(&mut tracker, third, now);
    assert_eq!(tracker.selection(), &Selection::Ambiguous);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::SelectionChanged(Selection::Ambiguous)))
    );
    second.bytes[33] = 0x14;
    feed(&mut tracker, second, now);
    assert_eq!(selected(&tracker).remote.ip().to_string(), "12.0.0.1");
}

#[test]
fn syn_reconnect_generation_and_disconnect_change_connection_identity() {
    let now = Instant::now();
    let mut tracker = Tracker::default();
    feed(&mut tracker, tcp(99, 0x12, &[]), now);
    feed(&mut tracker, tcp(100, 0x18, &ticks()), now);
    let first = selected(&tracker);
    feed(&mut tracker, tcp(99, 0x12, &[]), now);
    assert_eq!(selected(&tracker), first);
    let events = feed(&mut tracker, tcp(199, 0x12, &[]), now);
    assert_eq!(tracker.selection(), &Selection::Absent);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::SelectionChanged(Selection::Absent)))
    );
    feed(&mut tracker, tcp(200, 0x18, &ticks()), now);
    assert_ne!(selected(&tracker).incarnation, first.incarnation);
    let mut next = tcp(100, 0x18, &ticks());
    next.section_id = 1;
    assert!(has_issue(
        &feed(&mut tracker, next, now),
        Reliability::GenerationChanged
    ));
    assert_eq!(selected(&tracker).generation, 1);
    let mut close = tcp(200, 0x11, &[]);
    close.section_id = 1;
    feed(&mut tracker, close, now);
    assert_eq!(tracker.selection(), &Selection::Absent);
}

#[test]
fn malformed_truncated_and_unattributable_packets_invalidate_selection() {
    let now = Instant::now();
    for case in 0..4 {
        let mut tracker = Tracker::default();
        feed(&mut tracker, tcp(100, 0x18, &ticks()), now);
        selected(&tracker);
        let mut packet = tcp(200, 0x18, &[]);
        match case {
            0 => packet.original_len += 1,
            1 => packet.bytes[32] = 0xf0,
            2 => packet.local_address = None,
            _ => packet.captured_len += 1,
        }
        let events = feed(&mut tracker, packet, now);
        assert_eq!(tracker.selection(), &Selection::Absent);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::Reliability { .. }))
        );
        assert!(!events.iter().any(|event| matches!(event, Event::Tcp(_))));
    }
}

#[test]
fn malformed_tcp_options_are_rejected_including_duplicate_timestamps() {
    for options in [
        vec![8, 9, 0, 0],
        vec![2, 0, 0, 0],
        vec![1, 1, 1, 2],
        [vec![8, 10], vec![0; 8], vec![8, 10], vec![0; 8]].concat(),
    ] {
        let mut packet = tcp(100, 0x18, &[]);
        packet.bytes.extend_from_slice(&options);
        packet.bytes[32] = ((20 + options.len()) / 4 * 16) as u8;
        let length = packet.bytes.len() as u16;
        packet.bytes[2..4].copy_from_slice(&length.to_be_bytes());
        packet.captured_len = length as u32;
        packet.original_len = length as u32;
        let parsed = network::parse(&packet);
        assert!(parsed.issue.is_some(), "{options:?}");
        assert!(parsed.tcp.is_none());
    }
}

#[test]
fn missing_backward_and_future_capture_timestamps_never_become_usable_samples() {
    let now = Instant::now();
    let mut tracker = Tracker::default();
    let mut packet = tcp(100, 0x10, &[]);
    packet.timestamp_ns = Some(1_000_000_000);
    let events = feed(&mut tracker, packet.clone(), now);
    assert!(matches!(
        events.last(),
        Some(Event::Tcp(TcpObservation {
            timestamp_ns: Some(1_000_000_000),
            ..
        }))
    ));
    packet.timestamp_ns = Some(999_999_999);
    assert!(has_issue(
        &feed(&mut tracker, packet.clone(), now),
        Reliability::TimestampIncoherent
    ));
    packet.timestamp_ns = Some(i128::MAX);
    let events = feed(&mut tracker, packet, now);
    assert!(has_issue(&events, Reliability::TimestampIncoherent));
    assert!(matches!(
        events.last(),
        Some(Event::Tcp(TcpObservation {
            timestamp_ns: None,
            ..
        }))
    ));
}

#[test]
fn sequence_holes_are_reported_separately_from_packet_number_discontinuity() {
    let now = Instant::now();
    let mut tracker = Tracker::default();
    feed(&mut tracker, tcp(100, 0x18, &ticks()), now);
    feed(&mut tracker, tcp(200, 0x18, &[1, 2]), now);
    let mut events = vec![];
    tracker.expire(now + Duration::from_secs(3), |event| events.push(event));
    assert!(events.iter().any(|event| matches!(event, Event::Reliability { issue: Reliability::StreamInterrupted(reason), .. } if reason.contains("TCP manquants"))));
    assert!(!has_issue(&events, Reliability::PacketNumberDiscontinuity));
    assert_eq!(tracker.selection(), &Selection::Absent);
    let mut packet = tcp(300, 0x18, &ticks());
    packet.number = 100;
    events.clear();
    tracker.packet(&packet, now, |event| events.push(event));
    assert!(has_issue(&events, Reliability::PacketNumberDiscontinuity));
    assert_eq!(tracker.collector_drops(), None);
}

#[test]
fn capacity_is_bounded_and_cannot_hide_a_second_game_connection() {
    let now = Instant::now();
    let mut tracker = Tracker::default();
    for index in 0..MAX_CONNECTIONS {
        let mut packet = tcp(100, 0x18, &ticks());
        packet.bytes[12] = index as u8 + 20;
        feed(&mut tracker, packet, now);
    }
    let events = feed(&mut tracker, tcp(100, 0x18, &ticks()), now);
    assert!(has_issue(&events, Reliability::CapacityExceeded));
    assert_eq!(tracker.candidates.len(), MAX_CONNECTIONS);
    assert_eq!(tracker.selection(), &Selection::Ambiguous);
    tracker.expire(now + IDLE_TIMEOUT, |_| {});
    assert!(tracker.candidates.is_empty());
    assert_eq!(tracker.selection(), &Selection::Absent);
}

#[test]
fn empty_suspended_batches_and_terminal_errors_invalidate_immediately() {
    let now = Instant::now();
    let mut tracker = Tracker::default();
    feed(&mut tracker, tcp(100, 0x18, &ticks()), now);
    let mut events = vec![];
    let mut batch = CaptureBatch {
        packets: vec![],
        status: "suspended".into(),
        interruption: Some("attribution ambiguë".into()),
        finished: false,
        error: None,
    };
    tracker.batch(&batch, now, |event| events.push(event));
    assert_eq!(tracker.selection(), &Selection::Absent);
    assert!(has_issue(
        &events,
        Reliability::Interrupted("attribution ambiguë".into())
    ));
    events.clear();
    tracker.batch(&batch, now, |event| events.push(event));
    assert!(events.is_empty());
    batch.interruption = None;
    batch.packets.push(tcp(100, 0x18, &ticks()));
    batch.finished = true;
    batch.error = Some("service arrêté".into());
    tracker.batch(&batch, now, |event| events.push(event));
    assert!(events.iter().any(|event| matches!(event, Event::Tcp(_))));
    assert!(has_issue(
        &events,
        Reliability::Interrupted("service arrêté".into())
    ));
    assert_eq!(tracker.selection(), &Selection::Absent);
}

#[test]
fn recovered_suffix_still_reports_the_hole_before_new_tcp_samples() {
    let now = Instant::now();
    let mut tracker = Tracker::default();
    let initial = ticks();
    feed(&mut tracker, tcp(100, 0x18, &initial), now);
    let first = selected(&tracker);
    feed(
        &mut tracker,
        tcp(100 + initial.len() as u32 + 8, 0x18, &ticks()),
        now,
    );
    let mut events = vec![];
    tracker.expire(now + Duration::from_secs(3), |event| events.push(event));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Reliability {
            issue: Reliability::StreamInterrupted(_),
            ..
        }
    )));
    assert_eq!(selected(&tracker), first);
}

#[test]
fn handshake_both_directions_and_missing_timestamp_preserve_existing_identity_and_clock() {
    let now = Instant::now();
    let mut tracker = Tracker::default();
    let mut syn = tcp(400, 0x02, &[]);
    let source: [u8; 4] = syn.bytes[12..16].try_into().unwrap();
    let destination: [u8; 4] = syn.bytes[16..20].try_into().unwrap();
    let ports: [u8; 4] = syn.bytes[20..24].try_into().unwrap();
    syn.bytes[12..16].copy_from_slice(&destination);
    syn.bytes[16..20].copy_from_slice(&source);
    syn.bytes[20..24].copy_from_slice(&[ports[2], ports[3], ports[0], ports[1]]);
    feed(&mut tracker, syn, now);
    let incarnation = tracker.incarnation;
    feed(&mut tracker, tcp(99, 0x12, &[]), now);
    feed(&mut tracker, tcp(100, 0x18, &ticks()), now);
    assert_eq!(selected(&tracker).incarnation, incarnation);
    let mut ack = tcp(133, 0x10, &[]);
    ack.timestamp_ns = Some(2_000_000_000);
    feed(&mut tracker, ack.clone(), now);
    ack.timestamp_ns = None;
    feed(&mut tracker, ack.clone(), now);
    ack.timestamp_ns = Some(1_000_000_000);
    assert!(has_issue(
        &feed(&mut tracker, ack, now),
        Reliability::TimestampIncoherent
    ));
}

#[test]
fn traffic_survives_candidate_capacity_and_includes_udp() {
    let now = Instant::now();
    let mut tracker = Tracker::default();
    for index in 0..MAX_CONNECTIONS {
        let mut packet = tcp(100, 0x10, &[]);
        packet.bytes[12] = 20 + index as u8;
        feed(&mut tracker, packet, now);
    }
    let events = feed(&mut tracker, tcp(100, 0x10, &[]), now);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Traffic {
            direction: Direction::Incoming,
            captured_bytes: 40
        }
    )));
    assert!(has_issue(&events, Reliability::CapacityExceeded));
    let mut udp = tcp(0, 0, &[]);
    udp.bytes.truncate(28);
    udp.bytes[9] = 17;
    udp.bytes[2..4].copy_from_slice(&28u16.to_be_bytes());
    udp.bytes[24..26].copy_from_slice(&8u16.to_be_bytes());
    udp.captured_len = 28;
    udp.original_len = 28;
    let events = feed(&mut tracker, udp, now);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Traffic {
            captured_bytes: 28,
            ..
        }
    )));
    assert!(!events.iter().any(|event| matches!(event, Event::Tcp(_))));
}
