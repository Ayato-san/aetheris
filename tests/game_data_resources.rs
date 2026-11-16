use aetheris::game_data::{
    ClientIdentity,
    resources::{self, Kind, Manifest, Resource},
    schema::{Field, Kind as FieldKind, Schema},
};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let mut random = [0; 16];
        getrandom::fill(&mut random).unwrap();
        let path = std::env::temp_dir().join(format!("aetheris-resources-{}", hex::encode(random)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn client() -> ClientIdentity {
    ClientIdentity {
        version: "synthetic-1".into(),
        region: "test".into(),
    }
}
fn key() -> SigningKey {
    SigningKey::from_bytes(&[7; 32])
}
fn fixture(root: &Fixture, revision: u64) -> Manifest {
    let schema = Schema {
        id: "synthetic".into(),
        compatible_clients: vec![client()],
        table_version: 0,
        column_hashes: vec!["h1".into()],
        fields: vec![Field {
            name: "Unknown1".into(),
            kind: FieldKind::U32,
        }],
    };
    let bytes = serde_json::to_vec(&schema).unwrap();
    fs::create_dir_all(root.0.join("schemas")).unwrap();
    fs::write(root.0.join("schemas/test.json"), &bytes).unwrap();
    let manifest = Manifest {
        format: 1,
        revision,
        compatible_clients: vec![client()],
        files: vec![Resource {
            path: "schemas/test.json".into(),
            sha256: hex::encode(Sha256::digest(&bytes)),
            size: bytes.len() as u64,
            kind: Kind::Schema,
        }],
    };
    sign(root, &manifest);
    manifest
}
fn sign(root: &Fixture, manifest: &Manifest) {
    let bytes = serde_json::to_vec(manifest).unwrap();
    let signature = key().sign(&bytes);
    fs::write(root.0.join("resources.json"), bytes).unwrap();
    fs::write(root.0.join("resources.sig"), signature.to_bytes()).unwrap();
}

#[test]
fn signed_resource_installation_and_offline_reuse_need_no_game_files() {
    let source = Fixture::new();
    let store = Fixture::new();
    fixture(&source, 1);
    let bundle =
        resources::install(&source.0, &store.0, &client(), &key().verifying_key()).unwrap();
    assert_eq!(bundle.manifest.revision, 1);
    drop(source); // unavailable remote/source directory has no bearing on offline use
    let active = resources::active(&store.0, &key().verifying_key())
        .unwrap()
        .unwrap();
    assert_eq!(active.directory, bundle.directory);
    assert!(active.directory.join("schemas/test.json").exists());
}

#[test]
fn corrupt_incompatible_unavailable_and_replayed_updates_preserve_the_previous_bundle() {
    let source = Fixture::new();
    let update = Fixture::new();
    let store = Fixture::new();
    fixture(&source, 2);
    resources::install(&source.0, &store.0, &client(), &key().verifying_key()).unwrap();
    let previous = fs::read(store.0.join("active.json")).unwrap();
    let manifest = fixture(&update, 3);
    fs::write(update.0.join("resources.sig"), [0; 64]).unwrap();
    assert!(resources::install(&update.0, &store.0, &client(), &key().verifying_key()).is_err());
    sign(&update, &manifest);
    fs::write(update.0.join("schemas/test.json"), b"corrupt").unwrap();
    assert!(resources::install(&update.0, &store.0, &client(), &key().verifying_key()).is_err());
    fixture(&update, 3);
    let incompatible = ClientIdentity {
        version: "synthetic-2".into(),
        region: "test".into(),
    };
    assert!(
        resources::install(&update.0, &store.0, &incompatible, &key().verifying_key()).is_err()
    );
    fixture(&update, 1);
    assert!(resources::install(&update.0, &store.0, &client(), &key().verifying_key()).is_err());
    assert!(
        resources::install(
            &update.0.join("missing"),
            &store.0,
            &client(),
            &key().verifying_key()
        )
        .is_err()
    );
    assert_eq!(fs::read(store.0.join("active.json")).unwrap(), previous);
    assert_eq!(
        resources::active(&store.0, &key().verifying_key())
            .unwrap()
            .unwrap()
            .manifest
            .revision,
        2
    );
}

#[test]
fn signed_paths_and_schema_compatibility_are_checked_before_activation() {
    let source = Fixture::new();
    let store = Fixture::new();
    let mut manifest = fixture(&source, 1);
    manifest.files[0].path = "../outside.json".into();
    sign(&source, &manifest);
    assert!(resources::install(&source.0, &store.0, &client(), &key().verifying_key()).is_err());
    let mut manifest = fixture(&source, 1);
    manifest.compatible_clients[0].version = "other".into();
    sign(&source, &manifest);
    assert!(
        resources::install(
            &source.0,
            &store.0,
            &manifest.compatible_clients[0],
            &key().verifying_key()
        )
        .is_err()
    );
    assert!(!store.0.join("active.json").exists());
}

#[test]
fn active_resource_tampering_is_detected_and_signatures_bind_the_revision() {
    let source = Fixture::new();
    let store = Fixture::new();
    fixture(&source, 1);
    let bundle =
        resources::install(&source.0, &store.0, &client(), &key().verifying_key()).unwrap();
    fs::write(bundle.directory.join("schemas/test.json"), b"modified").unwrap();
    assert!(resources::active(&store.0, &key().verifying_key()).is_err());
    fixture(&source, 2);
    resources::install(&source.0, &store.0, &client(), &key().verifying_key()).unwrap();
    assert_eq!(
        resources::active(&store.0, &key().verifying_key())
            .unwrap()
            .unwrap()
            .manifest
            .revision,
        2
    );
    let mut manifest = fixture(&source, 1);
    manifest.revision = 100;
    fs::write(
        source.0.join("resources.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let fresh = Fixture::new();
    assert!(resources::install(&source.0, &fresh.0, &client(), &key().verifying_key()).is_err());
}
