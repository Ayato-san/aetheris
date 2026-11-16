//! Entirely synthetic, redistributable fixtures. No real client compatibility
//! is asserted by these tests.
use aetheris::game_data::{self, ClientIdentity, catalog::*};
use serde_json::json;
use std::{fs, path::PathBuf, sync::atomic::AtomicBool};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let mut random = [0; 16];
        getrandom::fill(&mut random).unwrap();
        let root = std::env::temp_dir().join(format!("aetheris-catalog-{}", hex::encode(random)));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn identity() -> ClientIdentity {
    ClientIdentity {
        version: "synthetic-1".into(),
        region: "test".into(),
    }
}
fn build(source: &Fixture) -> Build {
    fs::write(
        source.0.join("test.pak"),
        b"synthetic archive, not an Unreal container",
    )
    .unwrap();
    fs::write(
        source.0.join("aetheris-client.json"),
        serde_json::to_vec(&identity()).unwrap(),
    )
    .unwrap();
    Build {
        format: FORMAT_VERSION,
        inventory: game_data::inventory(&source.0).unwrap(),
        schema_set: "synthetic-v1".into(),
        compatible_clients: vec![identity()],
        required_tables: Family::ALL
            .into_iter()
            .map(|f| (f, std::collections::BTreeSet::from([f.key().into()])))
            .collect(),
        tables: Family::ALL
            .into_iter()
            .map(|family| TableReport {
                family,
                table: family.key().into(),
                source: "synthetic/test.bin".into(),
                sha256: "ab".repeat(32),
                schema: "synthetic-v1".into(),
                status: TableStatus::Decoded,
            })
            .collect(),
        records: Family::ALL
            .into_iter()
            .map(|family| Record {
                family,
                id: "1".into(),
                technical_name: format!("test_{}", family.key()),
                table: family.key().into(),
                row: 0,
                namespace: "test".into(),
                translation_key: Some("label".into()),
                fields: json!({"ID":{"Value":1},"Unknown1":17}),
            })
            .collect(),
        translations: vec![
            Translation {
                locale: "fr-FR".into(),
                namespace: "test".into(),
                key: "label".into(),
                text: "Nom français".into(),
                source: "synthetic/l10n-fr".into(),
            },
            Translation {
                locale: "en-US".into(),
                namespace: "test".into(),
                key: "label".into(),
                text: "English name".into(),
                source: "synthetic/l10n-en".into(),
            },
        ],
        relations: vec![Relation {
            from_family: Family::Skills,
            from_id: "1".into(),
            to_family: Family::Buffs,
            to_id: "1".into(),
            field: "Effect.BuffId".into(),
            value: json!({"quantity":2,"index":0}),
        }],
    }
}

#[test]
fn publishes_all_families_with_provenance_relations_and_language_fallback() {
    let source = Fixture::new();
    let store = Fixture::new();
    let mut build = build(&source);
    let key = publish(&build, &source.0, &store.0, &AtomicBool::new(false)).unwrap();
    let catalog = Catalog::active(&store.0).unwrap().unwrap();
    for family in Family::ALL {
        assert_eq!(
            catalog.record(family, "1").unwrap().unwrap().fields["Unknown1"],
            17
        );
    }
    assert_eq!(
        catalog
            .name(Family::Skills, "1", "fr-FR", Some(&build.inventory))
            .unwrap(),
        ResolvedName {
            text: "Nom français".into(),
            source: NameSource::SelectedLocale,
            stale: false
        }
    );
    assert_eq!(
        catalog.name(Family::Skills, "1", "de-DE", None).unwrap(),
        ResolvedName {
            text: "English name".into(),
            source: NameSource::English,
            stale: true
        }
    );
    assert_eq!(
        catalog
            .name(Family::Skills, "missing", "en-US", None)
            .unwrap()
            .source,
        NameSource::Identifier
    );
    build.records.reverse();
    build.tables.reverse();
    build.translations.reverse();
    assert_eq!(
        publish(&build, &source.0, &store.0, &AtomicBool::new(false)).unwrap(),
        key
    );
    build.translations.clear();
    publish(&build, &source.0, &store.0, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        Catalog::active(&store.0)
            .unwrap()
            .unwrap()
            .name(Family::Skills, "1", "fr-FR", None)
            .unwrap()
            .source,
        NameSource::Technical
    );
}

#[test]
fn failures_keep_the_previous_catalogue_and_clean_staging() {
    let source = Fixture::new();
    let store = Fixture::new();
    let build = build(&source);
    publish(&build, &source.0, &store.0, &AtomicBool::new(false)).unwrap();
    let previous = fs::read(store.0.join("active.json")).unwrap();
    let mut broken = build.clone();
    broken.tables.pop();
    assert!(publish(&broken, &source.0, &store.0, &AtomicBool::new(false)).is_err());
    let mut broken = build.clone();
    broken.compatible_clients.clear();
    assert!(publish(&broken, &source.0, &store.0, &AtomicBool::new(false)).is_err());
    let mut broken = build.clone();
    broken.relations[0].to_id = "missing".into();
    assert!(publish(&broken, &source.0, &store.0, &AtomicBool::new(false)).is_err());
    assert!(publish(&build, &source.0, &store.0, &AtomicBool::new(true)).is_err());
    fs::write(source.0.join("test.pak"), b"modified").unwrap();
    assert!(publish(&build, &source.0, &store.0, &AtomicBool::new(false)).is_err());
    assert_eq!(fs::read(store.0.join("active.json")).unwrap(), previous);
    assert!(
        fs::read_dir(store.0.join("catalogues")).unwrap().all(|e| !e
            .unwrap()
            .path()
            .to_string_lossy()
            .ends_with(".tmp"))
    );
}

#[test]
fn rejected_tables_are_distinct_from_confirmed_absence() {
    let source = Fixture::new();
    let mut build = build(&source);
    build.tables[0].status = TableStatus::Absent {
        evidence: "validated table contains zero records".into(),
    };
    assert!(build.validate().is_err()); // contradictory record still present
    build.records.retain(|r| r.family != Family::Items);
    build.validate().unwrap();
    build.tables[0].status = TableStatus::Rejected {
        error: "truncated payload".into(),
    };
    assert!(build.validate().is_err());
}

#[test]
fn corrupt_cache_or_active_database_is_never_accepted() {
    let source = Fixture::new();
    let store = Fixture::new();
    let build = build(&source);
    let key = publish(&build, &source.0, &store.0, &AtomicBool::new(false)).unwrap();
    fs::write(
        store.0.join("catalogues").join(format!("{key}.sqlite3")),
        b"corrupted",
    )
    .unwrap();
    assert!(Catalog::active(&store.0).is_err());
    assert!(publish(&build, &source.0, &store.0, &AtomicBool::new(false)).is_err());
}

#[test]
fn copied_installations_and_source_additions_are_detected_without_launcher() {
    let source = Fixture::new();
    let build = build(&source);
    fs::create_dir_all(source.0.join("Aion2/Content/Paks")).unwrap();
    assert_eq!(
        game_data::discover(std::slice::from_ref(&source.0)),
        vec![source.0.canonicalize().unwrap()]
    );
    build.inventory.verify_unchanged(&source.0).unwrap();
    fs::write(source.0.join("new.utoc"), b"added").unwrap();
    assert!(build.inventory.verify_unchanged(&source.0).is_err());
    assert!(!game_data::inventory(&source.0).unwrap().problems.is_empty());
}

#[test]
fn catalogue_selection_prefers_matching_sources_and_marks_stale_names() {
    let first = Fixture::new();
    let second = Fixture::new();
    let local_store = Fixture::new();
    let supplied_store = Fixture::new();
    let local_build = build(&first);
    let mut supplied_build = build(&second);
    fs::write(second.0.join("test.pak"), b"different client content").unwrap();
    supplied_build.inventory = game_data::inventory(&second.0).unwrap();
    publish(
        &local_build,
        &first.0,
        &local_store.0,
        &AtomicBool::new(false),
    )
    .unwrap();
    publish(
        &supplied_build,
        &second.0,
        &supplied_store.0,
        &AtomicBool::new(false),
    )
    .unwrap();
    let local = Catalog::active(&local_store.0).unwrap().unwrap();
    let supplied = Catalog::active(&supplied_store.0).unwrap().unwrap();
    assert!(std::ptr::eq(
        select(Some(&local), Some(&supplied), Some(&local_build.inventory)).unwrap(),
        &local
    ));
    assert!(std::ptr::eq(
        select(
            Some(&local),
            Some(&supplied),
            Some(&supplied_build.inventory)
        )
        .unwrap(),
        &supplied
    ));
    let stale = select(Some(&local), None, Some(&supplied_build.inventory)).unwrap();
    assert!(
        stale
            .name(
                Family::Skills,
                "1",
                "fr-FR",
                Some(&supplied_build.inventory)
            )
            .unwrap()
            .stale
    );
    assert!(select(None, None, None).is_none());
}

#[test]
fn command_line_reports_unknown_identity_and_failures_without_creating_a_catalogue() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("a.pak"), b"synthetic").unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_aetheris-game-data"))
        .args(["inspect", fixture.0.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(result["client"].is_null());
    assert_eq!(result["archives"].as_array().unwrap().len(), 1);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_aetheris-game-data"))
        .args(["inspect", fixture.0.join("missing").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!fixture.0.join("active.json").exists());
}

#[test]
fn cancellation_at_every_build_phase_keeps_the_active_catalogue_and_writes_a_report() {
    use std::sync::atomic::Ordering;
    for phase in [
        Phase::Records,
        Phase::Translations,
        Phase::Relations,
        Phase::Source,
        Phase::Activation,
    ] {
        let source = Fixture::new();
        let store = Fixture::new();
        let build = build(&source);
        publish(&build, &source.0, &store.0, &AtomicBool::new(false)).unwrap();
        let previous = fs::read(store.0.join("active.json")).unwrap();
        let cancel = AtomicBool::new(false);
        assert!(
            publish_with_progress(&build, &source.0, &store.0, &cancel, |p| {
                if p.phase == phase {
                    cancel.store(true, Ordering::Release);
                }
            })
            .is_err()
        );
        assert_eq!(fs::read(store.0.join("active.json")).unwrap(), previous);
        let reports: Vec<serde_json::Value> = fs::read_dir(store.0.join("reports"))
            .unwrap()
            .map(|e| serde_json::from_slice(&fs::read(e.unwrap().path()).unwrap()).unwrap())
            .collect();
        assert!(
            reports
                .iter()
                .any(|r| r["state"] == "cancelled" && r["tables"].as_array().unwrap().len() == 7)
        );
        assert!(
            fs::read_dir(store.0.join("catalogues")).unwrap().all(|e| !e
                .unwrap()
                .path()
                .to_string_lossy()
                .ends_with(".tmp"))
        );
    }
}

#[test]
fn a_source_change_at_activation_and_concurrent_publisher_cannot_replace_active_data() {
    let source = Fixture::new();
    let store = Fixture::new();
    let build = build(&source);
    publish(&build, &source.0, &store.0, &AtomicBool::new(false)).unwrap();
    let previous = fs::read(store.0.join("active.json")).unwrap();
    let result = publish_with_progress(&build, &source.0, &store.0, &AtomicBool::new(false), |p| {
        if p.phase == Phase::Records && p.completed == 0 {
            assert!(publish(&build, &source.0, &store.0, &AtomicBool::new(false)).is_err());
        }
        if p.phase == Phase::Activation {
            fs::write(source.0.join("test.pak"), b"updated").unwrap();
        }
    });
    assert!(result.is_err());
    assert_eq!(fs::read(store.0.join("active.json")).unwrap(), previous);
}

#[test]
fn cached_decoding_is_hash_addressed_and_purge_preserves_active_catalogues() {
    use aetheris::game_data::cache;
    let source = Fixture::new();
    let store = Fixture::new();
    let build = build(&source);
    let cancel = AtomicBool::new(false);
    publish(&build, &source.0, &store.0, &cancel).unwrap();
    let first = cache::decode(
        &store.0,
        b"source",
        b"schema",
        &identity(),
        "test-v1",
        &cancel,
        || Ok(b"decoded".to_vec()),
    )
    .unwrap();
    assert!(!first.reused);
    let second = cache::decode(
        &store.0,
        b"source",
        b"schema",
        &identity(),
        "test-v1",
        &cancel,
        || panic!("must use cache"),
    )
    .unwrap();
    assert!(second.reused);
    assert_eq!(second.bytes, b"decoded");
    let changed = cache::decode(
        &store.0,
        b"source",
        b"different-schema",
        &identity(),
        "test-v1",
        &cancel,
        || Ok(b"different".to_vec()),
    )
    .unwrap();
    assert!(!changed.reused);
    let path = fs::read_dir(store.0.join("cache"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(&path, b"invalid").unwrap();
    // Both cache keys are checked; at least one must report corruption.
    let a = cache::decode(
        &store.0,
        b"source",
        b"schema",
        &identity(),
        "test-v1",
        &cancel,
        || unreachable!(),
    );
    let b = cache::decode(
        &store.0,
        b"source",
        b"different-schema",
        &identity(),
        "test-v1",
        &cancel,
        || unreachable!(),
    );
    assert!(a.is_err() || b.is_err());
    let abandoned = store
        .0
        .join("catalogues/.aetheris-catalogue-00000000000000000000000000000000.tmp");
    fs::write(&abandoned, b"abandoned").unwrap();
    purge_cache(&store.0).unwrap();
    assert!(!store.0.join("cache").exists());
    assert!(!abandoned.exists());
    assert!(Catalog::active(&store.0).unwrap().is_some());
}

#[test]
fn omitting_a_required_table_cannot_hide_its_failure() {
    let source = Fixture::new();
    let mut build = build(&source);
    build
        .required_tables
        .get_mut(&Family::Creatures)
        .unwrap()
        .insert("spawn_table".into());
    assert!(build.validate().is_err());
    build.tables.push(TableReport {
        family: Family::Creatures,
        table: "spawn_table".into(),
        source: "synthetic/spawn.bin".into(),
        sha256: "ab".repeat(32),
        schema: "synthetic-v1".into(),
        status: TableStatus::Absent {
            evidence: "verified empty source table".into(),
        },
    });
    build.validate().unwrap();
}

#[test]
#[ignore = "child-process fixture invoked by interrupted_publication_recovers_on_next_attempt"]
fn publication_child() {
    let root = PathBuf::from(std::env::var_os("AETHERIS_CATALOGUE_INTERRUPT_FIXTURE").unwrap());
    let build: Build = serde_json::from_slice(&fs::read(root.join("build.json")).unwrap()).unwrap();
    publish_with_progress(
        &build,
        &root,
        &root.join("store"),
        &AtomicBool::new(false),
        |p| {
            if p.phase == Phase::Records {
                fs::write(root.join("ready"), b"ready").unwrap();
                loop {
                    std::thread::park();
                }
            }
        },
    )
    .unwrap();
}

#[test]
fn interrupted_publication_recovers_on_next_attempt() {
    let source = Fixture::new();
    let build = build(&source);
    let store = source.0.join("store");
    publish(&build, &source.0, &store, &AtomicBool::new(false)).unwrap();
    fs::write(
        source.0.join("build.json"),
        serde_json::to_vec(&build).unwrap(),
    )
    .unwrap();
    let previous = fs::read(store.join("active.json")).unwrap();
    struct Child(std::process::Child);
    impl Drop for Child {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut child = Child(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "publication_child", "--ignored", "--nocapture"])
            .env("AETHERIS_CATALOGUE_INTERRUPT_FIXTURE", &source.0)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !source.0.join("ready").exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "child did not reach active transaction"
        );
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "child exited before interruption"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    assert_eq!(fs::read(store.join("active.json")).unwrap(), previous);
    assert!(Catalog::active(&store).unwrap().is_some());
    publish(&build, &source.0, &store, &AtomicBool::new(false)).unwrap();
    assert!(
        fs::read_dir(store.join("catalogues")).unwrap().all(|e| !e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".tmp"))
    );
}
