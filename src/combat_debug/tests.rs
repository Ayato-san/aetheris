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
fn catalog_names_hp_without_guessing_fields_and_filters_accept_names_and_compact_hex() {
    let message = vec![
        20, 0, 0x8d, 0xb4, 0xed, 1, 2, 1, 0, 0x90, 0x9d, 1, 0, 0xaa, 0xbb, 0xcc, 0xdd,
    ];
    let packets = vec![tcp(1, 100, 0x12, &[]), tcp(2, 101, 0x18, &message)];
    let rows = analysis::analyze(&packets, completed());
    let hp = rows.iter().find(|row| row.kind == "message").unwrap();
    assert_eq!(hp.opcode_name(), "HP_UPDATE");
    assert_eq!(hp.status, "identifié");
    assert!(hp.entity.is_empty());
    assert_eq!(
        hp.sections
            .iter()
            .filter(|section| !section.bytes.is_empty())
            .count(),
        1
    );
    assert!(hp.summary.contains("non interprétés"));
    assert_eq!(hp.bytes[13..], [0xaa, 0xbb, 0xcc, 0xdd]);
    let mut store = Store::open(Path::new(":memory:")).unwrap();
    let id = store.create_session("network", "test", "").unwrap();
    store.append_packets(id, &packets).unwrap();
    store.reanalyze(id, true).unwrap();
    assert!(store.reanalyze_with_alignment(id, true).is_err());
    let filters = Filters {
        opcode: "008d".into(),
        ..Default::default()
    };
    assert_eq!(store.rows(id, &filters, 0, 200).unwrap().0, 1);
    for name in ["HP_UPDATE", "hp_update", "008D", "00 8D"] {
        assert_eq!(
            store
                .rows(
                    id,
                    &Filters {
                        opcode: name.into(),
                        ..Default::default()
                    },
                    0,
                    200
                )
                .unwrap()
                .0,
            1
        );
    }
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
fn reference_captures_synchronize_midstream_without_changing_evidence() {
    for (name, count) in [("capture.pcapng", 3159), ("capture-2.pcapng", 454)] {
        let path = Path::new("base-doc").join(name);
        let parsed = pcapng::read(&std::fs::read(&path).unwrap());
        assert_eq!(parsed.error, None, "{name}");
        assert_eq!(parsed.packets.len(), count, "{name}");
        let rows = analysis::analyze(&parsed.packets, completed());
        let messages: Vec<_> = rows.iter().filter(|row| row.kind == "message").collect();
        assert!(!messages.is_empty(), "{name}");
        assert!(
            messages
                .iter()
                .all(|row| row.direction == "Serveur → client"
                    && row.source.starts_with("193.202.112.101:")
                    && row.summary.contains("synchronisation AION 2"))
        );
        let mut names = std::collections::BTreeMap::new();
        for row in &messages {
            *names.entry(row.opcode_name()).or_insert(0usize) += 1;
        }
        assert!(
            names.len() > 3,
            "catalog should identify more than the old candidates"
        );
        println!("{name}: {names:?}");
        let represented = rows
            .iter()
            .flat_map(|row| row.packet_numbers.iter().copied())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            represented.len(),
            count,
            "all source packets remain represented"
        );
        println!(
            "{name}: {} server messages, {} LZ4 envelopes",
            messages.len(),
            rows.iter()
                .filter(|row| row.kind == "enveloppe LZ4")
                .count()
        );
        let mut store = Store::open(Path::new(":memory:")).unwrap();
        let id = store.import_file(&path).unwrap();
        for experimental in [false, true, false] {
            store.reanalyze_with_alignment(id, experimental).unwrap();
            let stored = store.rows(id, &Filters::default(), 0, 100_000).unwrap().1;
            assert_eq!(stored.len(), rows.len());
            for (stored, expected) in stored.iter().zip(&rows) {
                assert_eq!(stored.row.bytes, expected.bytes);
                assert_eq!(stored.row.direction, expected.direction);
                assert_eq!(stored.row.provenance, expected.provenance);
            }
            assert_eq!(store.packets(id).unwrap(), parsed.packets);
            assert_eq!(
                store.sessions().unwrap()[0].experimental_midstream,
                experimental
            );
        }
    }
}

fn frame(body: &[u8]) -> Vec<u8> {
    let mut length = body.len() + 4;
    let mut bytes = Vec::new();
    loop {
        let mut byte = (length & 127) as u8;
        length >>= 7;
        if length != 0 {
            byte |= 128;
        }
        bytes.push(byte);
        if length == 0 {
            break;
        }
    }
    bytes.extend_from_slice(body);
    bytes
}

fn bundle(plain: &[u8], marked: bool) -> Vec<u8> {
    let mut body = if marked {
        vec![0xf2, 0xff, 0xff]
    } else {
        vec![0xff, 0xff]
    };
    body.extend_from_slice(&(plain.len() as u32).to_le_bytes());
    body.extend(lz4_flex::block::compress(plain));
    frame(&body)
}

fn server_rows(bytes: &[u8]) -> Vec<analysis::AnalysisRow> {
    analysis::analyze(
        &[tcp(1, 100, 0x12, &[]), tcp(2, 101, 0x18, bytes)],
        completed(),
    )
}

#[test]
fn lz4_literal_vector_retains_envelope_and_decompressed_positions() {
    use analysis::ByteProvenance::*;
    // LZ4 token 0x60 followed by six literals: two complete three-byte frames.
    let wire = [17, 255, 255, 6, 0, 0, 0, 0x60, 6, 3, 0x36, 6, 0xab, 0xcd];
    let rows = server_rows(&wire);
    let envelope = rows
        .iter()
        .position(|row| row.kind == "enveloppe LZ4")
        .unwrap();
    assert_eq!(rows[envelope].bytes, wire);
    assert_eq!(rows[envelope].provenance, Some(TcpStream { offset: 0 }));
    let children: Vec<_> = rows.iter().filter(|row| row.kind == "message").collect();
    assert_eq!(children.len(), 2);
    for (index, child) in children.iter().enumerate() {
        assert_eq!(
            child.provenance,
            Some(Decompressed {
                envelope,
                offset: index * 3
            })
        );
        assert_eq!(child.packet_numbers, [2]);
    }
    assert_eq!(children[1].opcode, "ab cd");
}

#[test]
fn nested_marked_bundles_survive_every_tcp_split_and_round_trip_through_sqlite() {
    use analysis::ByteProvenance::*;
    let mut body: Vec<_> = (0..200).map(|index| (index * 73) as u8).collect();
    body[..2].copy_from_slice(&[0x37, 0x37]);
    let inner = frame(&body); // Multibyte lengths in both inner and outer frames.
    let nested = bundle(&inner, true);
    let wire = bundle(&[nested, frame(&[0xab, 0xcd])].concat(), false);
    let baseline = server_rows(&wire);
    for split in 1..wire.len() {
        let packets = vec![
            tcp(1, 100, 0x12, &[]),
            tcp(2, 101, 0x18, &wire[..split]),
            tcp(3, 101 + split as u32, 0x18, &wire[split..]),
        ];
        let rows = analysis::analyze(&packets, completed());
        assert_eq!(rows.len(), baseline.len());
        for (row, expected) in rows.iter().zip(&baseline) {
            assert_eq!(row.bytes, expected.bytes);
            assert_eq!(row.provenance, expected.provenance);
            if row.kind != "paquet brut" {
                assert_eq!(row.packet_numbers, [2, 3]);
            }
        }
    }
    let packets = vec![tcp(1, 100, 0x12, &[]), tcp(2, 101, 0x18, &wire)];
    let mut store = Store::open(Path::new(":memory:")).unwrap();
    let id = store.create_session("network", "nested", "").unwrap();
    store.append_packets(id, &packets).unwrap();
    for _ in 0..2 {
        store.reanalyze(id, true).unwrap();
        let stored = store
            .rows(
                id,
                &Filters {
                    opcode: "3737".into(),
                    ..Default::default()
                },
                0,
                10,
            )
            .unwrap()
            .1;
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].row.bytes, inner);
        let chain = store.envelope_chain(id, &stored[0].row).unwrap();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[1].row.bytes, wire);
        assert!(matches!(chain[0].row.provenance, Some(Decompressed { .. })));
        assert_eq!(store.packets(id).unwrap(), packets);
    }
    let mut invalid = baseline.clone();
    let child = invalid
        .iter_mut()
        .find(|row| row.kind == "message")
        .unwrap();
    child.provenance = Some(Decompressed {
        envelope: usize::MAX,
        offset: 0,
    });
    assert!(store.replace_analysis(id, &invalid).is_err());
    assert_eq!(
        store.rows(id, &Filters::default(), 0, 100).unwrap().0,
        baseline.len()
    );
    store.finish_session(id, None).unwrap();
    store.delete_session(id).unwrap();
    assert_eq!(store.rows(id, &Filters::default(), 0, 100).unwrap().0, 0);
}

#[test]
fn malformed_or_excessive_lz4_keeps_evidence_and_stops_only_affected_branch() {
    let valid = frame(&[0x03, 0x36]);
    let mut malformed = vec![0xff, 0xff, 100, 0, 0, 0, 0];
    for declared in [100u32, 9 * 1024 * 1024, 1024] {
        malformed[2..6].copy_from_slice(&declared.to_le_bytes());
        let bad = frame(&malformed);
        let rows = server_rows(&[bad.clone(), valid.clone()].concat());
        assert!(rows.iter().any(|row| row.kind == "enveloppe LZ4"
            && row.status.contains("erreur")
            && row.bytes == bad));
        assert_eq!(rows.iter().filter(|row| row.kind == "message").count(), 1);
    }
    // A complete child followed by a nonminimal length: no resynchronization.
    let plain = [valid.clone(), vec![0x86, 0, 3, 0x36], valid.clone()].concat();
    let rows = server_rows(&bundle(&plain, false));
    assert_eq!(rows.iter().filter(|row| row.kind == "message").count(), 1);
    let tail = rows
        .iter()
        .find(|row| row.kind == "octets décompressés non interprétés")
        .unwrap();
    assert_eq!(tail.bytes, plain[3..]);
    let mut deep = valid;
    for _ in 0..6 {
        deep = bundle(&deep, false);
    }
    let rows = server_rows(&deep);
    assert!(
        rows.iter()
            .any(|row| row.summary.contains("profondeur LZ4"))
    );
    assert_eq!(rows.iter().filter(|row| row.kind == "message").count(), 0);
}

#[test]
fn provenance_migration_keeps_old_interpretations_and_capture_bytes() {
    let path = std::env::temp_dir().join(format!(
        "aetheris-provenance-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    {
        let mut connection = rusqlite::Connection::open(&path).unwrap();
        crate::companion::migrations::initialize(
            &mut connection,
            &crate::companion::install::database_history()[..1],
        )
        .unwrap();
        connection.execute_batch("INSERT INTO sessions(id,name,source,started_at,status) VALUES(1,'old','fichier',0,'terminée'); INSERT INTO analysis_rows(session_id,ordinal,kind,source,destination,direction,protocol,opcode,entity,status,summary,bytes) VALUES(1,0,'message','','','','TCP','','','inconnu','legacy',X'060336');").unwrap();
    }
    {
        let store = Store::open(&path).unwrap();
        let rows = store.rows(1, &Filters::default(), 0, 10).unwrap().1;
        assert_eq!(rows[0].row.bytes, [6, 3, 0x36]);
        assert_eq!(rows[0].row.provenance, None);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn compressed_frames_wait_for_complete_input_and_are_not_opened_as_client_traffic() {
    let wire = bundle(&frame(&[0xab, 0xcd]), false);
    for length in 1..wire.len() {
        let rows = analysis::analyze(
            &[tcp(1, 100, 0x12, &[]), tcp(2, 101, 0x18, &wire[..length])],
            AnalysisOptions::default(),
        );
        assert!(rows.iter().all(|row| row.kind == "paquet brut"));
        assert!(rows.iter().all(|row| !row.status.contains("erreur")));
    }
    let rows = analysis::analyze(
        &[tcp(1, 100, 2, &[]), tcp(2, 101, 0x18, &wire)],
        completed(),
    );
    assert!(rows.iter().all(|row| row.kind != "enveloppe LZ4"));
}

fn tick(time: u64) -> Vec<u8> {
    frame(&[vec![0, 0x36], time.to_le_bytes().to_vec()].concat())
}

fn tick_train() -> Vec<u8> {
    [
        tick(1_793_200_000_000),
        tick(1_793_200_000_050),
        tick(1_793_200_000_100),
    ]
    .concat()
}

#[test]
fn midstream_sync_survives_all_tcp_splits_and_preserves_skipped_prefix_and_bundle_provenance() {
    let prefix = [0xde, 0xad, 0xbe, 0xef]; // A partial message, never decoded.
    let wire = [
        prefix.to_vec(),
        tick_train(),
        bundle(&frame(&[0xab, 0xcd]), false),
    ]
    .concat();
    for split in 1..wire.len() {
        let packets = vec![
            tcp(
                1,
                (u32::MAX - 10).wrapping_add(split as u32),
                0x18,
                &wire[split..],
            ),
            tcp(2, u32::MAX - 10, 0x18, &wire[..split]),
            tcp(3, u32::MAX - 10, 0x18, &wire[..split]), // retransmission
        ];
        let rows = analysis::analyze(&packets, completed());
        let messages: Vec<_> = rows.iter().filter(|row| row.kind == "message").collect();
        assert_eq!(messages.len(), 4, "split {split}");
        assert!(
            messages
                .iter()
                .all(|row| row.direction == "Serveur → client")
        );
        assert!(
            rows.iter()
                .any(|row| row.kind == "paquet brut"
                    && row.summary.contains("Préfixe non interprété"))
        );
        assert!(
            rows.iter()
                .any(|row| row.summary.contains("Retransmission"))
        );
        let envelope = rows
            .iter()
            .position(|row| row.kind == "enveloppe LZ4")
            .unwrap();
        assert_eq!(
            messages
                .iter()
                .find(|row| row.opcode == "ab cd")
                .unwrap()
                .provenance,
            Some(analysis::ByteProvenance::Decompressed {
                envelope,
                offset: 0
            })
        );
        assert_eq!(
            rows[envelope].provenance,
            Some(analysis::ByteProvenance::TcpStream {
                offset: prefix.len() + 33
            })
        );
    }
}

#[test]
fn live_midstream_waits_for_three_coherent_ticks_and_keeps_client_bodies_raw() {
    let wire = tick_train();
    let mut client = tcp(1, 100, 0x18, &frame(&[0xab, 0xcd]));
    // Reverse the endpoint pair to include both directions of one connection.
    client.bytes[12..20].copy_from_slice(&[10, 0, 0, 2, 10, 0, 0, 1]);
    client.bytes[20..24].copy_from_slice(&[0x34, 0x10, 0x30, 0x39]);
    for length in 1..=wire.len() {
        let packets = vec![client.clone(), tcp(2, 100, 0x18, &wire[..length])];
        let rows = analysis::analyze(&packets, AnalysisOptions::default());
        assert_eq!(
            rows.iter().filter(|row| row.kind == "message").count(),
            if length == wire.len() { 3 } else { 0 }
        );
        if length == wire.len() {
            let raw_client = rows.iter().find(|row| row.packet_numbers == [1]).unwrap();
            assert_eq!(raw_client.direction, "Client → serveur");
            assert!(raw_client.opcode.is_empty());
            assert!(raw_client.summary.contains("non déchiffré"));
            let mut store = Store::open(Path::new(":memory:")).unwrap();
            let id = store.create_session("network", "midstream", "").unwrap();
            store.append_packets(id, &packets).unwrap();
            store.reanalyze(id, false).unwrap();
            let stored = store
                .rows(
                    id,
                    &Filters {
                        direction: "Serveur → client".into(),
                        ..Default::default()
                    },
                    0,
                    100,
                )
                .unwrap()
                .1;
            assert_eq!(stored.len(), 3);
            assert!(
                stored
                    .iter()
                    .all(|row| row.row.summary.contains("synchronisation AION 2"))
            );
            assert_eq!(store.packets(id).unwrap(), packets);
        }
    }
}

#[test]
fn midstream_rejects_unrelated_traffic_bad_ticks_tls_and_contradicting_local_direction() {
    let time = 1_793_200_000_000;
    for wire in [
        [tick(time), tick(time)].concat(),
        [tick(time), tick(time), tick(time + 1)].concat(),
        [tick(time), tick(time - 1), tick(time - 2)].concat(),
        [tick(0), tick(50), tick(100)].concat(),
        [tick(time), tick(time + 60_001), tick(time + 120_002)].concat(),
        [tick(time), vec![1], tick(time + 50), tick(time + 100)].concat(),
        frame(&[0xab, 0xcd]).repeat(20),
        [vec![0x17, 3, 3, 0, 33], tick_train()].concat(),
    ] {
        let rows = analysis::analyze(&[tcp(1, 100, 0x18, &wire)], completed());
        assert!(rows.iter().all(|row| row.kind == "paquet brut"));
    }
    let mut packet = tcp(1, 100, 0x18, &tick_train());
    packet.local_address = Some("10.0.0.1".parse().unwrap());
    let rows = analysis::analyze(&[packet.clone()], completed());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].direction, "Client → serveur");
    packet.local_address = Some("10.0.0.2".parse().unwrap());
    let rows = analysis::analyze(&[packet], completed());
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| row.direction == "Serveur → client"));
}

#[test]
fn midstream_never_joins_ticks_across_tcp_gaps_or_conflicting_overlaps() {
    let wire = tick_train();
    let gaps = [
        tcp(1, 100, 0x18, &wire[..11]),
        tcp(2, 112, 0x18, &wire[11..]),
    ];
    let conflicts = [tcp(1, 100, 0x18, &wire), tcp(2, 111, 0x18, &[7, 8])];
    for packets in [&gaps, &conflicts] {
        let rows = analysis::analyze(packets, completed());
        assert!(rows.iter().all(|row| row.kind == "paquet brut"));
        assert!(
            rows.iter()
                .any(|row| row.summary.contains("Trou TCP")
                    || row.summary.contains("contradictoire"))
        );
    }
    let tail = [tick_train(), vec![1], tick_train()].concat();
    let rows = analysis::analyze(&[tcp(1, 100, 0x18, &tail)], completed());
    assert_eq!(rows.iter().filter(|row| row.kind == "message").count(), 3);
    assert!(
        rows.iter()
            .any(|row| row.summary.contains("Longueur AION invalide"))
    );
}

#[test]
fn tls_and_ambiguous_direction_disable_the_entire_connection() {
    let mut reverse = tcp(2, 500, 0x18, &tick_train());
    reverse.bytes[12..20].copy_from_slice(&[10, 0, 0, 2, 10, 0, 0, 1]);
    reverse.bytes[20..24].copy_from_slice(&[0x34, 0x10, 0x30, 0x39]);
    for first in [tick_train(), vec![0x17, 3, 3, 0, 1, 0xff]] {
        let rows = analysis::analyze(&[tcp(1, 100, 0x18, &first), reverse.clone()], completed());
        assert!(rows.iter().all(|row| row.kind == "paquet brut"));
        assert!(
            rows.iter().all(
                |row| row.summary.contains("ambiguë") || row.summary.contains("TLS identifiée")
            )
        );
    }
    // A different interface is independent, even with exactly the same tuple.
    reverse.interface_id = 1;
    let rows = analysis::analyze(
        &[tcp(1, 100, 0x18, &[0x17, 3, 3, 0, 1, 0xff]), reverse],
        completed(),
    );
    assert_eq!(rows.iter().filter(|row| row.kind == "message").count(), 3);
}

#[test]
fn synchronization_search_is_bounded_and_can_skip_an_invalid_early_candidate() {
    use super::synchronization::server_boundary;
    let prefix = [tick(1_793_200_000_000), vec![1]].concat();
    assert_eq!(
        server_boundary(&[prefix.clone(), tick_train()].concat()),
        Some(prefix.len())
    );
    assert_eq!(
        server_boundary(&[vec![0x55; 256 * 1024], tick_train()].concat()),
        None
    );
    let time = 1_793_200_000_000;
    let too_many_frames = [
        tick(time),
        frame(&[0xab, 0xcd]).repeat(512),
        tick(time + 50),
        tick(time + 100),
    ]
    .concat();
    assert_eq!(server_boundary(&too_many_frames), None);
}

#[test]
fn complete_s2c_catalog_matches_supplied_registry_and_is_shared_by_compressed_messages() {
    use super::opcode::Opcode;
    let fixture = include_str!("fixtures/s2c-opcodes.tsv");
    let mut seen = BTreeSet::new();
    for line in fixture.lines().filter(|line| !line.starts_with('#')) {
        let parts: Vec<_> = line.split('\t').collect();
        let code: [u8; 2] = hex::decode(parts[0]).unwrap().try_into().unwrap();
        assert!(seen.insert(code));
        let opcode = Opcode::from_wire(code);
        assert_eq!(opcode.wire(), code);
        let info = opcode.info().unwrap();
        assert_eq!(info.name, parts[1]);
        assert_eq!(info.confidence, parts[2]);
        assert!(
            info.source
                .contains("fe2122f0e54e1de854c11e91e3c7bf51916a4f52")
        );
        let wire = frame(&[code.to_vec(), 1_793_200_000_000u64.to_le_bytes().to_vec()].concat());
        let direct = server_rows(&wire)
            .into_iter()
            .find(|row| row.kind == "message")
            .unwrap();
        let compressed = server_rows(&bundle(&wire, false))
            .into_iter()
            .find(|row| row.kind == "message")
            .unwrap();
        assert_eq!(direct.opcode_name(), parts[1]);
        assert_eq!(direct.opcode_display_name(), info.display_name());
        assert_eq!(direct.sections, compressed.sections);
        assert_eq!(direct.status, compressed.status);
        assert_eq!(direct.bytes, compressed.bytes);
        assert!(direct.entity.is_empty());
        assert_eq!(direct.sections[0].offset, 1);
        assert_eq!(direct.sections[0].bytes, code);
        assert!(direct.sections[0].confidence.contains(parts[2]));
    }
    assert_eq!(seen.len(), 47);
    assert_eq!(seen.len(), Opcode::KNOWN.len());
    let unknown = Opcode::from_wire([0xab, 0xcd]);
    assert_eq!(unknown, Opcode::Unknown([0xab, 0xcd]));
    assert_eq!(unknown.wire(), [0xab, 0xcd]);
    assert_eq!(unknown.info(), None);
    let info = Opcode::DirectDamage.info().unwrap();
    assert_eq!(
        super::opcode::OpcodeInfo {
            translated_name: Some("Dégâts directs"),
            ..info
        }
        .display_name(),
        "Dégâts directs"
    );
    assert_eq!(
        super::opcode::OpcodeInfo {
            translated_name: Some(""),
            ..info
        }
        .display_name(),
        "DIRECT_DAMAGE"
    );
}

#[test]
fn interpretation_keeps_unknown_bodies_multibyte_offsets_and_unsupported_directions() {
    let mut body = vec![0xab, 0xcd];
    body.extend([0xe6; 200]);
    let rows = server_rows(&frame(&body));
    let unknown = rows.iter().find(|row| row.kind == "message").unwrap();
    assert_eq!(unknown.opcode, "ab cd");
    assert_eq!(unknown.opcode_name(), "Opcode inconnu");
    assert_eq!(unknown.sections[0].offset, 2);
    assert_eq!(unknown.bytes[2..], body);
    assert_eq!(unknown.status, "inconnu");
    assert!(
        rows.iter()
            .find(|row| row.kind == "paquet brut")
            .unwrap()
            .opcode
            .is_empty()
    );
    let wire = frame(&[0x04, 0x38, 1, 2, 3, 0xe6, 0x64, 50]);
    let rows = analysis::analyze(
        &[tcp(1, 100, 2, &[]), tcp(2, 101, 0x18, &wire)],
        completed(),
    );
    let client = rows.iter().find(|row| row.kind == "message").unwrap();
    assert!(client.opcode.is_empty());
    assert!(client.opcode_name().is_empty());
    assert!(client.sections.is_empty());
    assert_eq!(client.status, "non pris en charge");
    let rows = server_rows(&frame(&[0x00, 0x36, 1, 2]));
    let invalid_tick = rows.iter().find(|row| row.kind == "message").unwrap();
    assert_eq!(invalid_tick.opcode_name(), "SERVER_TICK");
    assert!(invalid_tick.status.contains("erreur"));
    assert!(
        invalid_tick
            .sections
            .iter()
            .all(|section| !section.kind.contains("Horodatage"))
    );
}

#[test]
fn catalog_and_decoded_timestamp_survive_reanalysis_and_filtering_by_name() {
    let time = 1_793_200_000_000;
    let plain = [
        tick(time),
        frame(&[0x02, 0x38, 4]),
        frame(&[0x05, 0x38, 5]),
        frame(&[0xab, 0xcd, 6]),
    ]
    .concat();
    let packets = [
        tcp(1, 100, 0x12, &[]),
        tcp(2, 101, 0x18, &bundle(&plain, true)),
    ];
    let mut store = Store::open(Path::new(":memory:")).unwrap();
    let id = store.create_session("network", "catalog", "").unwrap();
    store.append_packets(id, &packets).unwrap();
    for _ in 0..2 {
        store.reanalyze(id, true).unwrap();
        for (filter, expected) in [
            ("action_or_cast", "ACTION_OR_CAST"),
            ("DAMAGE_OVER_TIME", "DAMAGE_OVER_TIME"),
            ("05 38", "DAMAGE_OVER_TIME"),
            ("0538", "DAMAGE_OVER_TIME"),
            ("SERVER_TICK", "SERVER_TICK"),
            ("ABCD", "Opcode inconnu"),
        ] {
            let stored = store
                .rows(
                    id,
                    &Filters {
                        opcode: filter.into(),
                        ..Default::default()
                    },
                    0,
                    10,
                )
                .unwrap()
                .1;
            assert_eq!(stored.len(), 1, "{filter}");
            assert_eq!(stored[0].row.opcode_name(), expected);
            assert_eq!(store.envelope_chain(id, &stored[0].row).unwrap().len(), 1);
            if expected == "SERVER_TICK" {
                assert_eq!(stored[0].row.status, "décodé");
                let timestamp = stored[0]
                    .row
                    .sections
                    .iter()
                    .find(|section| section.kind.contains("Horodatage"))
                    .unwrap();
                assert_eq!(timestamp.value, time.to_string());
                assert_eq!(timestamp.offset, 3);
            }
        }
        assert_eq!(store.packets(id).unwrap(), packets);
    }
}
