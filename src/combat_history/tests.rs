use super::*;
use crate::combat_debug::analysis::AnalysisRow;

fn row(time: u64, bytes: Vec<u8>) -> AnalysisRow {
    AnalysisRow {
        timestamp_ns: Some(time.into()),
        bytes,
        direction: "Serveur → client".into(),
        kind: "message".into(),
        provenance: None,
        source: String::new(),
        destination: String::new(),
        protocol: "TCP".into(),
        opcode: String::new(),
        entity: String::new(),
        status: "identifié".into(),
        summary: String::new(),
        packet_numbers: Vec::new(),
        sections: Vec::new(),
    }
}
fn reference() -> CaptureAnalysis {
    let messages: Vec<(u64, String)> =
        serde_json::from_str(include_str!("../../tests/fixtures/zikel-messages.json")).unwrap();
    let rows: Vec<_> = messages
        .into_iter()
        .map(|(time, bytes)| row(time, ::hex::decode(bytes).unwrap()))
        .collect();
    decoder::decode(&rows, 0)
}
#[test]
fn zikel_matches_all_questlog_lines_and_indicators() {
    let analysis = reference();
    let selection = analysis
        .select(38153, 4085, 7_500_000_000, analysis.duration_ns)
        .unwrap();
    let report = analysis.report(selection).unwrap();
    assert_eq!(report.participants.len(), 1);
    assert!(report.unattributed.is_empty());
    let player = &report.participants[0];
    assert_eq!(
        (
            player.totals.damage,
            player.totals.hits,
            player.totals.ticks
        ),
        (4_807_725, 482, 347)
    );
    assert_eq!(player.totals.summon_damage, 369_626);
    assert_eq!(
        (
            player.totals.critical,
            player.totals.back,
            player.totals.perfect,
            player.totals.double
        ),
        (127, 66, 39, 77)
    );
    assert_eq!(player.last_ns - player.first_ns, 151_090_140_200);
    assert_eq!(player.skills.len(), 13);
    for (id, damage, hits, ticks) in [
        (17352450, 2038720, 80, 0),
        (17440037, 484551, 55, 0),
        (17040020, 424836, 57, 0),
        (17050020, 381010, 53, 0),
        (17150452, 369626, 68, 0),
        (17060343, 270523, 4, 0),
        (17730001, 258473, 93, 0),
        (17400040, 183100, 6, 112),
        (17080240, 162101, 15, 133),
        (17070240, 122405, 8, 102),
        (17010240, 43214, 24, 0),
        (17020240, 39253, 16, 0),
        (17030240, 29913, 3, 0),
    ] {
        let skill = player
            .skills
            .iter()
            .find(|s| s.skill_ids.contains(&id))
            .unwrap();
        assert_eq!(
            (skill.totals.damage, skill.totals.hits, skill.totals.ticks),
            (damage, hits, ticks),
            "skill {id}"
        );
    }
}
fn key(id: u64) -> EntityKey {
    EntityKey { id, occurrence: 0 }
}
fn entity(id: u64, identity: Identity, name: Option<&str>, owner: Option<&str>) -> Entity {
    Entity {
        key: key(id),
        connection: String::new(),
        identity,
        name: name.map(str::to_string),
        owner_name: owner.map(str::to_string),
        npc_code: None,
        start_ns: 0,
        end_ns: None,
    }
}
fn hit(source: u64, time_ns: u64, periodic: bool) -> Damage {
    Damage {
        source: key(source),
        target: key(9),
        time_ns,
        amount: 100,
        skill: 999,
        periodic,
        indicators: Indicators::default(),
        evidence: vec![1, 2, 3],
    }
}
fn group() -> CaptureAnalysis {
    CaptureAnalysis {
        capture_name: "Controlled multiplayer".into(),
        fingerprint: "test".into(),
        entities: vec![
            entity(1, Identity::Player, Some("Alice"), None),
            entity(2, Identity::Player, None, None),
            entity(3, Identity::Npc, None, Some("Alice")),
            entity(4, Identity::Unknown, None, None),
            entity(9, Identity::Npc, None, None),
        ],
        damage: vec![
            hit(1, 10, false),
            hit(3, 20, true),
            hit(2, 30, false),
            hit(4, 40, false),
        ],
        review: Vec::new(),
        limitations: Vec::new(),
        duration_ns: 100,
    }
}
#[test]
fn multiplayer_keeps_unknown_sources_separate_and_inclusive_bounds() {
    let analysis = group();
    let report = analysis
        .report(analysis.select(9, 1, 10, 40).unwrap())
        .unwrap();
    assert_eq!(report.participants.len(), 2);
    let alice = &report.participants[0];
    assert_eq!(alice.totals.damage, 200);
    assert_eq!(alice.totals.summon_damage, 100);
    assert_eq!((alice.totals.hits, alice.totals.ticks), (1, 1));
    assert!(alice.dps.is_some());
    assert!(report.participants[1].entity.name.is_none());
    assert!(report.participants[1].dps.is_none());
    assert_eq!(report.unattributed.len(), 1);
    assert_eq!(report.events.len(), 3);
    assert_eq!(
        alice.skills[0].name_provenance,
        "Identifiant réseau ; nom inconnu"
    );
}
#[test]
fn ambiguous_owner_and_reused_ids_do_not_leak_damage() {
    let mut analysis = group();
    analysis.entities[1].name = Some("Alice".into());
    let report = analysis
        .report(analysis.select(9, 1, 0, 100).unwrap())
        .unwrap();
    assert_eq!(report.participants[0].totals.damage, 100);
    assert_eq!(report.unattributed.len(), 2);
    analysis.entities[0].end_ns = Some(50);
    let mut recycled = entity(1, Identity::Npc, None, None);
    recycled.key.occurrence = 1;
    recycled.start_ns = 50;
    analysis.entities.push(recycled);
    assert!(
        analysis
            .select(9, 1, 0, 100)
            .unwrap_err()
            .to_string()
            .contains("réutilisé")
    );
    assert!(analysis.select(9, 1, 0, 49).is_ok());
}
#[test]
fn incomplete_analysis_can_only_save_with_local_damage_proven() {
    let mut analysis = group();
    analysis.review.push(Review {
        time_ns: None,
        reason: "TCP gap".into(),
        evidence: vec![],
    });
    assert!(
        !analysis
            .report(analysis.select(9, 1, 0, 100).unwrap())
            .unwrap()
            .review
            .is_empty()
    );
    let error = analysis
        .report(analysis.select(9, 1, 50, 100).unwrap())
        .unwrap_err()
        .to_string();
    assert!(error.contains("incomplète"));
    assert!(!error.contains("n’a pas"));
}
#[test]
fn snapshots_survive_capture_deletion_and_each_reanalysis_appends() {
    let dir = std::env::temp_dir().join(format!(
        "aetheris-history-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let capture = dir.join("captures.sqlite3");
    let captures = Store::open(&capture).unwrap();
    let id = captures.create_session("test", "capture", "test").unwrap();
    let analysis = group();
    let selection = analysis.select(9, 1, 0, 100).unwrap();
    let history = storage::History::open(&dir.join(storage::DATABASE_NAME)).unwrap();
    let first = history.save(&analysis, selection).unwrap();
    let second = history.save(&analysis, selection).unwrap();
    assert_ne!(first, second);
    captures.finish_session(id, None).unwrap();
    captures.delete_session(id).unwrap();
    drop(captures);
    assert_eq!(history.entries().unwrap().len(), 2);
    let before = serde_json::to_string(&history.entries().unwrap()[0].report).unwrap();
    drop(history);
    let history = storage::History::open(&dir.join(storage::DATABASE_NAME)).unwrap();
    assert_eq!(
        serde_json::to_string(&history.entries().unwrap()[0].report).unwrap(),
        before
    );
    history.delete(first).unwrap();
    assert_eq!(history.entries().unwrap()[0].id, second);
    drop(history);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn truncated_and_unknown_layouts_are_reviewed_without_panics() {
    let messages: Vec<(u64, String)> =
        serde_json::from_str(include_str!("../../tests/fixtures/zikel-messages.json")).unwrap();
    let bytes = messages
        .iter()
        .map(|(_, s)| ::hex::decode(s).unwrap())
        .find(|b| b.get(1..3) == Some(&[4, 0x38]))
        .unwrap();
    for length in 0..bytes.len() {
        let analysis = decoder::decode(&[row(1, bytes[..length].to_vec())], 0);
        assert!(analysis.damage.is_empty());
    }
    let mut invalid = bytes;
    invalid.push(0xff);
    let analysis = decoder::decode(&[row(1, invalid)], 0);
    assert_eq!(analysis.review.len(), 1);
    assert!(analysis.damage.is_empty());
}

fn var(mut value: u64, bytes: &mut Vec<u8>) {
    loop {
        let next = (value & 127) as u8;
        value >>= 7;
        bytes.push(next | if value > 0 { 128 } else { 0 });
        if value == 0 {
            break;
        }
    }
}
fn framed(body: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    var(body.len() as u64 + 4, &mut bytes);
    bytes.extend_from_slice(body);
    bytes
}
fn player_message(id: u64, name: &str) -> Vec<u8> {
    let mut body = vec![0x33, 0x36];
    var(id, &mut body);
    body.push(0x5f);
    body.extend_from_slice(&[0; 4]);
    var(name.len() as u64, &mut body);
    body.extend_from_slice(name.as_bytes());
    framed(&body)
}
fn damage_message(source: u64, amount: u64) -> Vec<u8> {
    let mut body = vec![4, 0x38];
    var(9, &mut body);
    var(6, &mut body);
    var(0, &mut body);
    var(source, &mut body);
    body.extend_from_slice(&999u32.to_le_bytes());
    body.push(1);
    var(3, &mut body);
    body.push(4);
    var(0, &mut body);
    body.push(1);
    body.extend_from_slice(&[0; 8]);
    var(100, &mut body);
    var(amount, &mut body);
    body.extend_from_slice(&[1, 0]);
    framed(&body)
}
#[test]
fn controlled_multiplayer_decodes_identities_and_keeps_unnamed_players() {
    let rows = [
        row(0, player_message(1, "Alice")),
        row(0, player_message(2, "")),
        row(10, damage_message(1, 100)),
        row(10, damage_message(2, 200)),
        row(20, damage_message(99, 300)),
    ];
    let analysis = decoder::decode(&rows, 0);
    let report = analysis
        .report(analysis.select(9, 1, 0, 20).unwrap())
        .unwrap();
    assert_eq!(report.participants.len(), 2);
    assert_eq!(report.participants[1].totals.damage, 200);
    assert!(report.participants[1].entity.name.is_none());
    assert_eq!(report.unattributed[0].event.amount, 300);
}
#[test]
fn recycled_source_lifetime_cannot_inherit_an_old_owner() {
    let spawn = |id| {
        let mut body = vec![0x41, 0x36];
        var(id, &mut body);
        body.extend_from_slice(&[0x1c, 0, 0]);
        body.extend_from_slice(&123u32.to_le_bytes());
        framed(&body)
    };
    let rows = [
        row(0, player_message(1, "Alice")),
        row(1, damage_message(1, 100)),
        row(10, spawn(1)),
        row(20, damage_message(1, 200)),
    ];
    let analysis = decoder::decode(&rows, 0);
    assert!(analysis.select(9, 1, 0, 20).is_err());
    assert_eq!(analysis.damage[0].source.occurrence, 0);
    assert_eq!(analysis.damage[1].source.occurrence, 1);
    assert_eq!(
        analysis
            .entities
            .iter()
            .find(|e| e.key == analysis.damage[1].source)
            .unwrap()
            .identity,
        Identity::Npc
    );
}

#[test]
fn equal_ids_on_different_connections_are_ambiguous_not_merged() {
    let mut alice = row(0, player_message(1, "Alice"));
    alice.source = "192.0.2.1:13328".into();
    let mut first = row(10, damage_message(1, 100));
    first.source = alice.source.clone();
    let mut other = row(20, damage_message(1, 200));
    other.source = "192.0.2.2:13328".into();
    let analysis = decoder::decode(&[alice, first, other], 0);
    assert_ne!(analysis.damage[0].source, analysis.damage[1].source);
    assert_ne!(analysis.damage[0].target, analysis.damage[1].target);
    assert!(analysis.select(9, 1, 0, 20).is_err());
}
