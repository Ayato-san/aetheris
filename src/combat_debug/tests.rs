use super::{
    analysis::{self, AnalysisOptions},
    pcapng::{self, CapturedPacket, LinkType},
    storage::{Filters, Store},
};
use std::{collections::BTreeSet, path::Path};

fn tcp(number: u64, sequence: u32, flags: u8, payload: &[u8]) -> CapturedPacket {
    let mut bytes = vec![0u8; 40];
    bytes[0] = 0x45;
    bytes[2..4].copy_from_slice(&((40 + payload.len()) as u16).to_be_bytes());
    bytes[8] = 64;
    bytes[9] = 6;
    bytes[12..16].copy_from_slice(&[10, 0, 0, 1]);
    bytes[16..20].copy_from_slice(&[10, 0, 0, 2]);
    bytes[20..22].copy_from_slice(&12345u16.to_be_bytes());
    bytes[22..24].copy_from_slice(&13328u16.to_be_bytes());
    bytes[24..28].copy_from_slice(&sequence.to_be_bytes());
    bytes[32] = 0x50;
    bytes[33] = flags;
    bytes.extend_from_slice(payload);
    CapturedPacket {
        number,
        section_id: 0,
        interface_id: 0,
        local_address: None,
        link_type: LinkType::RawIpv4,
        timestamp_ns: Some(number as i128),
        timestamp: None,
        captured_len: bytes.len() as u32,
        original_len: bytes.len() as u32,
        bytes,
    }
}

fn completed() -> AnalysisOptions {
    AnalysisOptions {
        finalized: true,
        experimental_midstream: false,
    }
}

#[test]
fn reconstructs_split_and_coalesced_messages_out_of_order_across_sequence_wrap() {
    let packets = vec![
        tcp(1, u32::MAX - 3, 0x02, &[]),
        tcp(2, 1, 0x18, &[9, 7, 0x0b, 0x00, 0x8d, 1, 2, 3, 4, 5]),
        tcp(3, u32::MAX - 2, 0x18, &[8, 4, 0x38, 6]),
    ];
    let rows = analysis::analyze(&packets, completed());
    let messages = rows
        .iter()
        .filter(|row| row.kind == "message")
        .collect::<Vec<_>>();
    // Two frames: length 5, then length 4. The final suffix is incomplete.
    let first = messages
        .iter()
        .find(|row| row.bytes == [8, 4, 0x38, 6, 9])
        .unwrap();
    assert_eq!(first.packet_numbers, [2, 3]);
    assert_eq!(first.direction, "Client → serveur");
    assert!(rows.iter().any(|row| row.status.contains("erreur")));
}

#[test]
fn identical_retransmission_is_not_a_second_message_and_conflicts_fail_closed() {
    let mut packets = vec![tcp(1, 100, 2, &[]), tcp(2, 101, 0x18, &[8, 4, 0x38, 1, 2])];
    packets.push(tcp(3, 101, 0x18, &[8, 4, 0x38, 1, 2]));
    let rows = analysis::analyze(&packets, completed());
    assert_eq!(rows.iter().filter(|row| row.kind == "message").count(), 1);
    assert!(
        rows.iter()
            .any(|row| row.summary.contains("Retransmission"))
    );
    packets.push(tcp(4, 101, 0x18, &[8, 4, 0x38, 7, 2]));
    let rows = analysis::analyze(&packets, completed());
    assert_eq!(rows.iter().filter(|row| row.kind == "message").count(), 0);
    assert!(
        rows.iter()
            .any(|row| row.summary.contains("contradictoire"))
    );
}

#[test]
fn midstream_gaps_and_truncation_do_not_fabricate_messages() {
    let midstream = vec![tcp(1, 101, 0x18, &[8, 4, 0x38, 1, 2])];
    assert!(
        analysis::analyze(&midstream, completed())
            .iter()
            .all(|row| row.kind != "message")
    );
    let gap = vec![tcp(1, 100, 2, &[]), tcp(2, 105, 0x18, &[8, 4, 0x38, 1, 2])];
    assert!(
        analysis::analyze(&gap, completed())
            .iter()
            .any(|row| row.summary.contains("Trou TCP"))
    );
    let mut truncated = vec![tcp(1, 100, 2, &[]), tcp(2, 101, 0x18, &[8, 4, 0x38, 1, 2])];
    truncated[1].original_len += 1;
    assert!(
        analysis::analyze(&truncated, completed())
            .iter()
            .all(|row| row.kind != "message")
    );
    let mut prefix = vec![
        tcp(1, 100, 2, &[]),
        tcp(2, 101, 0x18, &[8, 4, 0x38, 1, 2]),
        tcp(3, 106, 0x18, &[8, 4, 0x38]),
    ];
    prefix[2].original_len += 2;
    let rows = analysis::analyze(&prefix, completed());
    assert_eq!(
        rows.iter().filter(|row| row.kind == "message").count(),
        1,
        "valid prefix survives a later truncation"
    );
}

#[test]
fn uleb128_rejects_overflow_and_missing_termination() {
    assert_eq!(analysis::read_varuint(&[0xfc, 0x10]), Some((2172, 2)));
    assert_eq!(analysis::read_varuint(&[0x86, 0x1c]), Some((3590, 2)));
    assert_eq!(analysis::read_varuint(&[0xff; 10]), None);
    assert_eq!(analysis::read_varuint(&[0x80]), None);
}

#[test]
fn vlan_and_ipv6_extensions_are_parsed_and_ip_fragments_remain_raw() {
    let packet = tcp(1, 100, 2, &[]);
    let mut ethernet = packet.clone();
    let mut bytes = vec![0; 12];
    bytes.extend([0x81, 0, 0, 7, 0x08, 0]);
    bytes.extend(&packet.bytes);
    ethernet.bytes = bytes;
    ethernet.link_type = LinkType::Ethernet;
    ethernet.captured_len = ethernet.bytes.len() as u32;
    ethernet.original_len = ethernet.captured_len;
    let parsed = super::network::parse(&ethernet);
    assert_eq!(parsed.source.unwrap().port(), 12345);
    assert!(parsed.tcp.unwrap().syn);
    let mut ipv6 = packet.clone();
    ipv6.bytes = vec![0; 56];
    ipv6.bytes[0] = 0x60;
    ipv6.bytes[4..6].copy_from_slice(&16u16.to_be_bytes());
    ipv6.bytes[6] = 0;
    ipv6.bytes[23] = 1;
    ipv6.bytes[39] = 2;
    ipv6.bytes[40] = 17; // Hop-by-hop header, followed by UDP.
    ipv6.bytes[48..50].copy_from_slice(&2345u16.to_be_bytes());
    ipv6.bytes[50..52].copy_from_slice(&3456u16.to_be_bytes());
    ipv6.bytes[52..54].copy_from_slice(&8u16.to_be_bytes());
    ipv6.link_type = LinkType::RawIpv6;
    ipv6.captured_len = 56;
    ipv6.original_len = 56;
    let parsed = super::network::parse(&ipv6);
    assert_eq!(parsed.protocol, "UDP");
    assert_eq!(parsed.source.unwrap().port(), 2345);
    assert_eq!(parsed.issue, None);
    ipv6.bytes[6] = 44;
    assert!(
        super::network::parse(&ipv6)
            .issue
            .unwrap()
            .contains("Fragment")
    );
    let mut ipv4 = packet;
    ipv4.bytes[6] = 0x20;
    assert!(
        super::network::parse(&ipv4)
            .issue
            .unwrap()
            .contains("Fragment")
    );
}

#[test]
fn hp_sections_preserve_unknown_tail_and_filters_accept_compact_hex() {
    let message = vec![
        20, 0, 0x8d, 0xb4, 0xed, 1, 2, 1, 0, 0x90, 0x9d, 1, 0, 0xaa, 0xbb, 0xcc, 0xdd,
    ];
    let packets = vec![tcp(1, 100, 2, &[]), tcp(2, 101, 0x18, &message)];
    let rows = analysis::analyze(&packets, completed());
    let hp = rows.iter().find(|row| row.kind == "message").unwrap();
    assert_eq!(hp.sections.len(), 2);
    assert_eq!(hp.sections[1].value, "105872");
    assert_eq!(hp.sections[1].offset, 9);
    assert!(
        hp.sections
            .iter()
            .all(|section| section.confidence == "hypothèse")
    );
    assert_eq!(hp.bytes[13..], [0xaa, 0xbb, 0xcc, 0xdd]);
    let mut store = Store::open(Path::new(":memory:")).unwrap();
    let id = store.create_session("network", "test", "").unwrap();
    store.append_packets(id, &packets).unwrap();
    store.reanalyze(id, true).unwrap();
    assert!(store.reanalyze_with_alignment(id, true).is_err());
    let filters = Filters {
        opcode: "008d".into(),
        entity: "b4ed01".into(),
        ..Default::default()
    };
    assert_eq!(store.rows(id, &filters, 0, 200).unwrap().0, 1);
    let filters = Filters {
        search: "aa bb cc dd".into(),
        ..Default::default()
    };
    assert_eq!(store.rows(id, &filters, 0, 200).unwrap().0, 1);
    store.recover_interrupted().unwrap();
    assert_eq!(store.sessions().unwrap()[0].status, "interrompue");
    assert_eq!(store.packets(id).unwrap()[1].bytes[40..], message);
}

#[test]
fn database_recovery_has_an_exclusive_process_lease() {
    let root = std::env::temp_dir().join(format!(
        "aetheris-lease-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = root.join("captures.sqlite3");
    let first = super::storage::DatabaseLease::acquire(&path).unwrap();
    assert!(super::storage::DatabaseLease::acquire(&path).is_err());
    drop(first);
    drop(super::storage::DatabaseLease::acquire(&path).unwrap());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn arbitrary_short_inputs_do_not_panic_in_packet_or_container_parsing() {
    let mut state = 0x12345678u32;
    for length in 0..512 {
        let mut bytes = Vec::new();
        for _ in 0..length {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            bytes.push(state as u8);
        }
        let _ = pcapng::read(&bytes);
        for link_type in [LinkType::Ethernet, LinkType::RawIp, LinkType::Other(65535)] {
            let packet = CapturedPacket {
                number: 1,
                section_id: 0,
                interface_id: 0,
                local_address: None,
                link_type,
                timestamp_ns: None,
                timestamp: None,
                captured_len: length as u32,
                original_len: length as u32,
                bytes: bytes.clone(),
            };
            let _ = super::network::parse(&packet);
        }
    }
}

#[test]
fn sqlite_preserves_evidence_rolls_back_failed_reanalysis_and_deletes_cascade() {
    let mut store = Store::open(Path::new(":memory:")).unwrap();
    let id = store.create_session("network", "test", "").unwrap();
    let packets = vec![tcp(1, 100, 2, &[]), tcp(2, 101, 0x18, &[8, 4, 0x38, 1, 2])];
    store.append_packets(id, &packets).unwrap();
    store.reanalyze(id, true).unwrap();
    let before = store.rows(id, &Filters::default(), 0, 100).unwrap();
    let mut invalid = before
        .1
        .iter()
        .map(|row| row.row.clone())
        .collect::<Vec<_>>();
    invalid[0].packet_numbers = vec![999];
    assert!(store.replace_analysis(id, &invalid).is_err());
    assert_eq!(
        store.rows(id, &Filters::default(), 0, 100).unwrap().0,
        before.0
    );
    assert_eq!(store.packets(id).unwrap()[1].bytes, packets[1].bytes);
    assert!(store.delete_session(id).is_err());
    assert_eq!(store.sessions().unwrap().len(), 1); // Active capture cannot be deleted.
    store.finish_session(id, None).unwrap();
    store.delete_session(id).unwrap();
    assert!(store.sessions().unwrap().is_empty());
    assert!(store.packets(id).unwrap().is_empty());
    assert_eq!(store.rows(id, &Filters::default(), 0, 100).unwrap().0, 0);
}

#[test]
#[ignore = "requires the two user-provided PCAPNG files in base-doc"]
fn reference_captures_preserve_evidence_and_only_frame_with_explicit_offline_alignment() {
    for (name, count, expected_messages, stream_bytes) in [
        ("capture.pcapng", 3159, 3280, 42640),
        ("capture-2.pcapng", 454, 601, 9604),
    ] {
        let bytes = std::fs::read(Path::new("base-doc").join(name)).unwrap();
        let parsed = pcapng::read(&bytes);
        assert_eq!(parsed.error, None, "{name}");
        assert_eq!(parsed.packets.len(), count, "{name}");
        let rows = analysis::analyze(&parsed.packets, completed());
        assert_eq!(
            rows.iter().filter(|row| row.kind == "message").count(),
            0,
            "capture has no SYN"
        );
        let represented = rows
            .iter()
            .flat_map(|row| row.packet_numbers.iter().copied())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            represented.len(),
            count,
            "all source packets remain visible"
        );
        let mut store = Store::open(Path::new(":memory:")).unwrap();
        let id = store
            .import_file(&Path::new("base-doc").join(name))
            .unwrap();
        assert_eq!(store.packets(id).unwrap(), parsed.packets);
        assert_eq!(store.sessions().unwrap()[0].packet_count, count);
        assert!(!store.sessions().unwrap()[0].experimental_midstream);
        store.reanalyze_with_alignment(id, true).unwrap();
        let (_, rows) = store.rows(id, &Filters::default(), 0, 100_000).unwrap();
        let messages: Vec<_> = rows
            .iter()
            .filter(|row| row.row.kind == "message")
            .collect();
        assert_eq!(messages.len(), expected_messages);
        assert_eq!(
            messages
                .iter()
                .map(|row| row.row.bytes.len())
                .sum::<usize>(),
            stream_bytes
        );
        assert!(
            messages
                .iter()
                .all(|row| row.row.summary.contains("SANS SYN"))
        );
        assert_eq!(
            store.packets(id).unwrap(),
            parsed.packets,
            "reanalysis never changes original evidence"
        );
        assert!(store.sessions().unwrap()[0].experimental_midstream);
        store.reanalyze_with_alignment(id, false).unwrap();
        assert_eq!(
            store
                .rows(id, &Filters::default(), 0, 100_000)
                .unwrap()
                .1
                .iter()
                .filter(|row| row.row.kind == "message")
                .count(),
            0
        );
    }
}
