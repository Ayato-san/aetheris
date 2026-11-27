use aetheris::game_data::{
    ClientIdentity,
    assets::{DecodedAsset, Decryption},
    reader::Control,
    schema_set::{self, Data, Entry, InputStage, SchemaSet, Selection},
    usmap::Mapping,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::PathBuf, sync::atomic::AtomicBool};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let mut random = [0; 16];
        getrandom::fill(&mut random).unwrap();
        let path =
            std::env::temp_dir().join(format!("aetheris-schema-set-{}", hex::encode(random)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn client() -> ClientIdentity {
    ClientIdentity {
        version: "test-1".into(),
        region: "synthetic".into(),
    }
}

// Independent, minimal USMAP fixture; no game content or writer round-trip.
fn mapping_bytes() -> Vec<u8> {
    let mut raw = 4u32.to_le_bytes().to_vec();
    for name in ["TestRow", "ID", "UnknownParameter", "Label"] {
        raw.extend((name.len() as u16).to_le_bytes());
        raw.extend(name.as_bytes());
    }
    raw.extend(0u32.to_le_bytes()); // enum count
    raw.extend(1u32.to_le_bytes()); // struct count
    raw.extend(0u32.to_le_bytes()); // name
    raw.extend(u32::MAX.to_le_bytes()); // no parent
    raw.extend(3u16.to_le_bytes());
    raw.extend(3u16.to_le_bytes());
    for (slot, name, tag) in [(0u16, 1u32, 19u8), (1, 2, 2), (2, 3, 10)] {
        raw.extend(slot.to_le_bytes());
        raw.push(1);
        raw.extend(name.to_le_bytes());
        raw.push(tag);
    }
    let mut data = vec![0xc4, 0x30, 4, 0, 0, 0, 0, 0];
    data.extend((raw.len() as u32).to_le_bytes());
    data.extend((raw.len() as u32).to_le_bytes());
    data.extend(raw);
    data
}
fn string(data: &mut Vec<u8>, s: &str) {
    data.extend((s.len() as i32 + 1).to_le_bytes());
    data.extend(s.as_bytes());
    data.push(0);
}
fn table() -> Vec<u8> {
    let mut data = 13i32.to_le_bytes().to_vec(); // same marker as an encrypted container
    string(&mut data, "column-id,column-unknown,column-label");
    data.extend(1i32.to_le_bytes()); // untagged row array
    data.extend(99u32.to_le_bytes()); // row ID distinct from column hashes
    data.extend((-37i32).to_le_bytes());
    string(&mut data, "Épée 🗡"); // StrProperty is plain UTF-8
    data
}
fn localization() -> Vec<u8> {
    let mut data = 1u32.to_le_bytes().to_vec();
    string(&mut data, "UI");
    data.extend(1u32.to_le_bytes());
    string(&mut data, "key");
    string(&mut data, "Texte");
    data.extend([0; 4]);
    data
}
const TABLE: &str = "AION2/Content/Data/Table/Test.dat";
const LOCALE: &str = "AION2/Content/L10N/fr-FR/L10NString.dat";
fn prepare(fixture: &Fixture) -> (PathBuf, Mapping, SchemaSet) {
    let root = fixture.0.join("prepared");
    fs::create_dir(&root).unwrap();
    let mut files = vec![];
    for (path, family, data) in [
        (TABLE, "table", table()),
        (LOCALE, "localization", localization()),
    ] {
        let p = root.join(PathBuf::from(path).with_extension("bin"));
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, &data).unwrap();
        files.push(DecodedAsset {
            path: path.into(),
            family: family.into(),
            source_sha256: hash(b"container"),
            output_sha256: hash(&data),
            bytes: data.len(),
        });
    }
    fs::write(
        root.join("decryption.json"),
        serde_json::to_vec(&Decryption {
            payload_stage: Some("decrypted".into()),
            protection: Some(schema_set::PROTECTION.into()),
            reader_revision: Some(schema_set::reader_revision()),
            app_version: Some("2.0.11".into()),
            files,
            errors: vec!["optional unsupported family".into()],
        })
        .unwrap(),
    )
    .unwrap();
    let mapping = Mapping::parse(&mapping_bytes()).unwrap();
    let selection = BTreeMap::from([
        (
            TABLE.into(),
            Selection::Table {
                row_struct: "TestRow".into(),
            },
        ),
        (LOCALE.into(), Selection::Localization),
    ]);
    let cancel = AtomicBool::new(false);
    let mut progress = |_| {};
    let set = schema_set::draft(
        &mapping,
        &root,
        "synthetic",
        &selection,
        &mut Control::new(&cancel, &mut progress),
    )
    .unwrap();
    (root, mapping, set)
}
fn decode(set: &SchemaSet, mapping: &Mapping, bytes: &[u8]) -> std::io::Result<Data> {
    set.decode(mapping, TABLE, InputStage::Decrypted, bytes, None)
}

#[test]
fn observed_layout_is_not_compatibility_and_input_states_are_explicit() {
    let root = Fixture::new();
    let (_, mapping, set) = prepare(&root);
    assert!(set.compatible_clients.is_empty());
    assert!(set.require_client(&client()).is_err());
    let Data::Table(rows) = decode(&set, &mapping, &table()).unwrap() else {
        panic!()
    };
    assert_eq!(
        rows,
        vec![json!({"ID":99,"UnknownParameter":-37,"Label":"Épée 🗡"})]
    );
    for stage in [InputStage::Container, InputStage::Decoded] {
        assert!(set.decode(&mapping, TABLE, stage, &table(), None).is_err());
    }
    assert!(
        set.decode(
            &mapping,
            TABLE,
            InputStage::Decrypted,
            &table(),
            Some(&client())
        )
        .is_err()
    );
    let mut compatible = set.clone();
    compatible.compatible_clients = vec![client()];
    assert!(
        compatible
            .decode(
                &mapping,
                TABLE,
                InputStage::Decrypted,
                &table(),
                Some(&client())
            )
            .is_ok()
    );
    let mut other = client();
    other.region = "other-region".into();
    assert!(compatible.require_client(&other).is_err());
    other = client();
    other.version = "test-2".into();
    assert!(compatible.require_client(&other).is_err());
}

#[test]
fn pinned_mapping_reader_protection_context_and_headers_cannot_drift() {
    let root = Fixture::new();
    let (_, mapping, set) = prepare(&root);
    for field in ["reader_revision", "protection", "format"] {
        let mut data = serde_json::to_value(&set).unwrap();
        data[field] = if field == "format" {
            json!(2)
        } else {
            json!("different")
        };
        assert!(SchemaSet::parse(&serde_json::to_vec(&data).unwrap()).is_err());
    }
    let mut wrong = set.clone();
    wrong.mapping_sha256 = hash(b"other mapping");
    assert!(decode(&wrong, &mapping, &table()).is_err());
    let mut wrong = set.clone();
    if let Entry::Table { column_hashes, .. } = wrong.entries.get_mut(TABLE).unwrap() {
        column_hashes.reverse();
    }
    assert!(decode(&wrong, &mapping, &table()).is_err());
    let mut wrong = set.clone();
    if let Entry::Table { table_version, .. } = wrong.entries.get_mut(TABLE).unwrap() {
        *table_version = 12;
    }
    assert!(decode(&wrong, &mapping, &table()).is_err());
    let mut wrong = set.clone();
    if let Entry::Table { row_struct, .. } = wrong.entries.get_mut(TABLE).unwrap() {
        *row_struct = "Missing".into();
    }
    assert!(decode(&wrong, &mapping, &table()).is_err());
    for length in 0..table().len() {
        assert!(decode(&set, &mapping, &table()[..length]).is_err());
    }
    let mut extra = table();
    extra.push(0);
    assert!(decode(&set, &mapping, &extra).is_err());
    let mut wrong = set.clone();
    wrong.compatible_clients = vec![ClientIdentity {
        version: "1".into(),
        region: "unknown".into(),
    }];
    assert!(wrong.validate().is_err());
}

#[test]
fn batch_reports_missing_corrupt_and_incompatible_inputs_without_activation() {
    let root = Fixture::new();
    let (prepared, mapping, mut set) = prepare(&root);
    let cancel = AtomicBool::new(false);
    let mut progress = |_| {};
    let mut control = Control::new(&cancel, &mut progress);
    let out = root.0.join("explore");
    let report =
        schema_set::decode_prepared(&set, &mapping, &prepared, None, &out, &mut control).unwrap();
    assert_eq!(report.state, "complete");
    assert_eq!(report.entries.len(), 2);
    assert!(!report.compatibility_verified);
    assert!(report.require_activation(&client()).is_err());
    assert_eq!(report.preparation_errors.len(), 1);
    assert!(
        schema_set::decode_prepared(&set, &mapping, &prepared, None, &out, &mut control).is_err()
    );
    let out = root.0.join("refused");
    assert!(
        schema_set::decode_prepared(
            &set,
            &mapping,
            &prepared,
            Some(&client()),
            &out,
            &mut control
        )
        .is_err()
    );
    assert!(!out.exists());
    set.compatible_clients = vec![client()];
    let report = schema_set::decode_prepared(
        &set,
        &mapping,
        &prepared,
        Some(&client()),
        &root.0.join("valid"),
        &mut control,
    )
    .unwrap();
    report.require_activation(&client()).unwrap();
    fs::write(
        prepared.join(PathBuf::from(TABLE).with_extension("bin")),
        b"tampered",
    )
    .unwrap();
    let report = schema_set::decode_prepared(
        &set,
        &mapping,
        &prepared,
        Some(&client()),
        &root.0.join("partial"),
        &mut control,
    )
    .unwrap();
    assert_eq!(report.state, "partial");
    assert!(!report.compatibility_verified);
    assert!(
        report
            .entries
            .iter()
            .any(|e| e.error.as_deref().is_some_and(|e| e.contains("checksum")))
    );
    assert!(report.require_activation(&client()).is_err());
    assert!(
        !root
            .0
            .join("partial")
            .join(PathBuf::from(TABLE).with_extension("json"))
            .exists()
    );
    let entry = set.entries[TABLE].clone();
    set.entries.insert("Missing.dat".into(), entry);
    let report = schema_set::decode_prepared(
        &set,
        &mapping,
        &prepared,
        None,
        &root.0.join("missing"),
        &mut control,
    )
    .unwrap();
    assert!(
        report
            .entries
            .iter()
            .any(|e| e.path == "Missing.dat" && e.error.is_some())
    );
    set.app_version = Some("other".into());
    assert!(
        schema_set::decode_prepared(
            &set,
            &mapping,
            &prepared,
            None,
            &root.0.join("context"),
            &mut control
        )
        .is_err()
    );
}

#[test]
fn negative_utf16_lengths_truncation_and_localization_namespace_are_bounded() {
    let root = Fixture::new();
    let (_, mapping, set) = prepare(&root);
    let mut data = table();
    data.truncate(4);
    data.extend(i32::MIN.to_le_bytes());
    assert!(
        decode(&set, &mapping, &data)
            .err()
            .unwrap()
            .to_string()
            .contains("string exceeds limit")
    );
    for length in 0..localization().len() - 4 {
        assert!(
            set.decode(
                &mapping,
                LOCALE,
                InputStage::Decrypted,
                &localization()[..length],
                None
            )
            .is_err()
        );
    }
    let mut data = 1u32.to_le_bytes().to_vec();
    data.extend(i32::MIN.to_le_bytes());
    assert!(
        set.decode(&mapping, LOCALE, InputStage::Decrypted, &data, None)
            .is_err()
    );
    let mut wrong = set.clone();
    wrong.entries.insert(
        LOCALE.into(),
        Entry::Localization {
            namespace: "wrong".into(),
            version: 1,
        },
    );
    assert!(
        wrong
            .decode(
                &mapping,
                LOCALE,
                InputStage::Decrypted,
                &localization(),
                None
            )
            .is_err()
    );
}

#[test]
fn cancellation_and_output_overlap_leave_sources_intact() {
    let root = Fixture::new();
    let (prepared, mapping, set) = prepare(&root);
    let cancel = AtomicBool::new(false);
    let mut progress = |_| {};
    let mut control = Control::new(&cancel, &mut progress);
    let out = prepared.join("child");
    assert!(
        schema_set::decode_prepared(&set, &mapping, &prepared, None, &out, &mut control).is_err()
    );
    assert!(!out.exists());
    cancel.store(true, std::sync::atomic::Ordering::Release);
    let out = root.0.join("cancelled");
    assert_eq!(
        schema_set::decode_prepared(&set, &mapping, &prepared, None, &out, &mut control)
            .err()
            .unwrap()
            .kind(),
        std::io::ErrorKind::Interrupted
    );
    assert!(!out.exists());

    cancel.store(false, std::sync::atomic::Ordering::Release);
    let mut progress = |p: aetheris::game_data::reader::Progress| {
        if p.completed == 1 {
            cancel.store(true, std::sync::atomic::Ordering::Release);
        }
    };
    let out = root.0.join("mid-cancelled");
    assert!(
        schema_set::decode_prepared(
            &set,
            &mapping,
            &prepared,
            None,
            &out,
            &mut Control::new(&cancel, &mut progress)
        )
        .is_err()
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(out.join("decoding.json")).unwrap()).unwrap();
    assert_eq!(report["state"], "cancelled");
    assert_eq!(report["compatibility_verified"], false);
}

#[test]
fn signed_schema_sets_bind_compatible_clients_and_the_actual_mapping() {
    use aetheris::game_data::resources::{self, Kind, Manifest, Resource};
    use ed25519_dalek::{Signer, SigningKey};
    let fixture = Fixture::new();
    let (_, _, mut set) = prepare(&fixture);
    set.compatible_clients = vec![client()];
    let source = fixture.0.join("bundle");
    fs::create_dir(&source).unwrap();
    let signing_key = SigningKey::from_bytes(&[7; 32]);
    let write = |set: &SchemaSet, with_mapping: bool| {
        let bytes = serde_json::to_vec(set).unwrap();
        fs::write(source.join("set.json"), &bytes).unwrap();
        fs::write(source.join("mapping.usmap"), mapping_bytes()).unwrap();
        let mut files = vec![Resource {
            path: "set.json".into(),
            sha256: hash(&bytes),
            size: bytes.len() as u64,
            kind: Kind::SchemaSet,
        }];
        if with_mapping {
            files.push(Resource {
                path: "mapping.usmap".into(),
                sha256: hash(&mapping_bytes()),
                size: mapping_bytes().len() as u64,
                kind: Kind::Mapping,
            });
        }
        let bytes = serde_json::to_vec(&Manifest {
            format: 1,
            revision: 1,
            compatible_clients: vec![client()],
            files,
        })
        .unwrap();
        fs::write(source.join("resources.json"), &bytes).unwrap();
        fs::write(
            source.join("resources.sig"),
            signing_key.sign(&bytes).to_bytes(),
        )
        .unwrap();
    };
    let install = |store: &str| {
        resources::install(
            &source,
            &fixture.0.join(store),
            &client(),
            &signing_key.verifying_key(),
        )
    };
    write(&set, true);
    install("valid").unwrap();
    write(&set, false);
    assert!(install("missing").is_err());
    let mut wrong = set.clone();
    wrong.compatible_clients[0].region = "other".into();
    write(&wrong, true);
    assert!(install("wrong-region").is_err());
    let mut wrong = set.clone();
    wrong.compatible_clients.clear();
    write(&wrong, true);
    assert!(install("unverified").is_err());
    let mut wrong = set.clone();
    wrong.mapping_sha256 = hash(b"different mapping");
    write(&wrong, true);
    assert!(install("wrong-mapping").is_err());
    let mut wrong = set.clone();
    if let Entry::Table { column_hashes, .. } = wrong.entries.get_mut(TABLE).unwrap() {
        column_hashes.pop();
    }
    write(&wrong, true);
    assert!(install("incomplete-layout").is_err());
}

#[test]
fn bundled_contract_has_an_exact_client_scope_and_current_reader_revision() {
    let client = ClientIdentity {
        version: "26581260".into(),
        region: "global".into(),
    };
    let set = schema_set::bundled(&client).unwrap();
    assert_eq!(set.entries.len(), 377);
    assert_eq!(
        set.mapping_sha256,
        "019f06284f923aa5f12660c5debbefcb99f3a424a16b4d62b8d2fdfd4997ea44"
    );
    assert!(
        schema_set::bundled(&ClientIdentity {
            version: "26581261".into(),
            ..client.clone()
        })
        .is_err()
    );
    assert!(
        schema_set::bundled(&ClientIdentity {
            region: "unknown".into(),
            ..client
        })
        .is_err()
    );
}

#[test]
fn historical_or_mislabelled_preparation_is_never_inferred_from_payload_bytes() {
    let root = Fixture::new();
    let (prepared, mapping, set) = prepare(&root);
    let path = prepared.join("decryption.json");
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let cancel = AtomicBool::new(false);
    let mut progress = |_| {};
    let mut control = Control::new(&cancel, &mut progress);
    for field in ["payload_stage", "protection", "reader_revision"] {
        let mut historical = original.clone();
        historical.as_object_mut().unwrap().remove(field);
        fs::write(&path, serde_json::to_vec(&historical).unwrap()).unwrap();
        let output = root.0.join(field);
        assert!(
            schema_set::decode_prepared(&set, &mapping, &prepared, None, &output, &mut control)
                .is_err()
        );
        assert!(!output.exists());
    }
    let mut wrong = original;
    for asset in wrong["files"].as_array_mut().unwrap() {
        if asset["path"] == TABLE {
            asset["family"] = json!("localization");
        }
    }
    fs::write(&path, serde_json::to_vec(&wrong).unwrap()).unwrap();
    let report = schema_set::decode_prepared(
        &set,
        &mapping,
        &prepared,
        None,
        &root.0.join("wrong-family"),
        &mut control,
    )
    .unwrap();
    assert_eq!(report.state, "partial");
    assert!(
        report.entries.iter().any(|entry| entry.path == TABLE
            && entry.error.as_deref().is_some_and(|e| e.contains("family")))
    );
}
