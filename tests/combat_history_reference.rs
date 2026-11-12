//! End-to-end validation against the private capture, when available locally.
use aetheris::{combat_debug::storage::Store, combat_history};

#[test]
#[ignore = "requires base-doc/first-analyze/aion2-capture-1.pcapng"]
fn imported_zikel_capture_produces_an_independent_report() {
    let directory = std::env::temp_dir().join(format!(
        "aetheris-zikel-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let mut captures = Store::open(&directory.join("captures.sqlite3")).unwrap();
    let capture = captures
        .import_file(std::path::Path::new(
            "base-doc/first-analyze/aion2-capture-1.pcapng",
        ))
        .unwrap();
    let analysis = combat_history::analyze_session(&captures, capture).unwrap();
    let selection = analysis
        .select(38153, 4085, 0, analysis.duration_ns)
        .unwrap();
    let history = combat_history::storage::History::open(
        &directory.join(combat_history::storage::DATABASE_NAME),
    )
    .unwrap();
    history.save(&analysis, selection).unwrap();
    captures.delete_session(capture).unwrap();
    drop(captures);
    let entries = history.entries().unwrap();
    let player = &entries[0].report.participants[0];
    assert_eq!(
        (
            player.totals.damage,
            player.totals.hits,
            player.totals.ticks
        ),
        (4_807_725, 482, 347)
    );
    assert_eq!(player.skills.len(), 13);
    assert_eq!(player.totals.summon_damage, 369_626);
    assert_eq!(entries[0].report.events.len(), 829);
    assert!(entries[0].report.unattributed.is_empty());
    drop(history);
    std::fs::remove_dir_all(directory).unwrap();
}
