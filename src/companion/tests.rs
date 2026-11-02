use super::{
    manifest::{self, Manifest, PROGRAMS},
    migrations::{self, Migration},
    policy::{self, Channel, Offer},
    releases::{self, Release},
};
use ed25519_dalek::{Signer, SigningKey};
use semver::Version;
use std::{collections::BTreeMap, fs, path::PathBuf, time::Duration};

pub(super) struct Directory(pub PathBuf);
impl Directory {
    pub fn new() -> Self {
        let mut random = [0; 16];
        getrandom::fill(&mut random).unwrap();
        let path = std::env::temp_dir().join(format!("aetheris-test-{}", hex::encode(random)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub(super) fn signed(directory: &std::path::Path, version: &str, key: &SigningKey) {
    fs::create_dir_all(directory).unwrap();
    let mut files = BTreeMap::new();
    for name in PROGRAMS {
        fs::write(directory.join(name), format!("{name} {version}")).unwrap();
        files.insert(name.into(), manifest::hash(&directory.join(name)).unwrap());
    }
    let bytes = serde_json::to_vec(&Manifest {
        version: Version::parse(version).unwrap(),
        target: "x86_64-pc-windows-msvc".into(),
        files,
    })
    .unwrap();
    fs::write(directory.join(manifest::MANIFEST), &bytes).unwrap();
    fs::write(
        directory.join(manifest::SIGNATURE),
        key.sign(&bytes).to_bytes(),
    )
    .unwrap();
}
#[test]
fn signatures_bind_versions_and_every_program_file() {
    let directory = Directory::new();
    let key = SigningKey::from_bytes(&[7; 32]);
    signed(&directory.0, "1.2.3", &key);
    assert!(manifest::verify(&directory.0, &key.verifying_key()).is_ok());
    assert!(
        manifest::verify(
            &directory.0,
            &SigningKey::from_bytes(&[8; 32]).verifying_key()
        )
        .is_err()
    );
    for program in PROGRAMS {
        signed(&directory.0, "1.2.3", &key);
        fs::write(directory.0.join(program), b"tampered").unwrap();
        assert!(manifest::verify(&directory.0, &key.verifying_key()).is_err());
    }
    signed(&directory.0, "1.2.3", &key);
    let bytes = fs::read_to_string(directory.0.join(manifest::MANIFEST))
        .unwrap()
        .replace("1.2.3", "9.9.9");
    fs::write(directory.0.join(manifest::MANIFEST), bytes).unwrap();
    assert!(manifest::authenticate(&directory.0, &key.verifying_key()).is_err());
    fs::remove_file(directory.0.join(manifest::SIGNATURE)).unwrap();
    assert!(manifest::authenticate(&directory.0, &key.verifying_key()).is_err());
}
fn release(version: &str) -> Release {
    Release {
        tag_name: format!("v{version}"),
        draft: false,
        prerelease: version.contains('-'),
        assets: vec![],
    }
}
#[test]
fn channels_order_semver_without_downgrading_or_trusting_dates() {
    let versions = || {
        vec![
            release("1.1.0-beta.2"),
            release("1.0.1"),
            release("1.1.0-beta.10"),
            release("1.1.0-beta.3"),
        ]
    };
    let installed = Version::parse("1.0.0").unwrap();
    assert_eq!(
        releases::select(versions(), Channel::Stable, &installed)
            .unwrap()
            .tag_name,
        "v1.0.1"
    );
    assert_eq!(
        releases::select(versions(), Channel::Prerelease, &installed)
            .unwrap()
            .tag_name,
        "v1.1.0-beta.10"
    );
    let beta = Version::parse("1.1.0-beta.10").unwrap();
    assert!(releases::select(versions(), Channel::Stable, &beta).is_none());
    assert!(releases::select(vec![release("1.1.0")], Channel::Prerelease, &beta).is_some());
    let mut draft = release("9.0.0");
    draft.draft = true;
    assert!(releases::select(vec![draft], Channel::Prerelease, &beta).is_none());
    assert!(releases::select(vec![release("1.0.0+other")], Channel::Stable, &installed).is_none());
}
#[test]
fn update_deadline_exclusion_and_first_window_grace_are_explicit() {
    assert_eq!(
        policy::offer(Duration::from_secs(3), false),
        Offer::BackgroundDownload
    );
    assert_eq!(
        policy::offer(Duration::from_millis(3001), false),
        Offer::Announce
    );
    assert_eq!(policy::offer(Duration::ZERO, true), Offer::Announce);
    assert!(!policy::first_launch_crash(true, None));
    assert!(policy::first_launch_crash(false, None));
    assert!(policy::first_launch_crash(
        false,
        Some(Duration::from_secs(9))
    ));
    assert!(!policy::first_launch_crash(
        false,
        Some(Duration::from_secs(10))
    ));
}
#[test]
fn migrations_catch_up_in_order_and_roll_back_on_failure() {
    let mut connection = rusqlite::Connection::open_in_memory().unwrap();
    let history = [
        Migration {
            id: 1,
            sql: "CREATE TABLE data(value INTEGER); INSERT INTO data VALUES(1);",
        },
        Migration {
            id: 2,
            sql: "UPDATE data SET value=value+2;",
        },
        Migration {
            id: 3,
            sql: "UPDATE data SET value=value+3;",
        },
    ];
    migrations::initialize(&mut connection, &history[..1]).unwrap();
    migrations::initialize(&mut connection, &history).unwrap();
    migrations::initialize(&mut connection, &history).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT value FROM data", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        6
    );
    assert!(migrations::initialize(&mut connection, &history[..2]).is_err());
    let failing = [
        Migration { id: 1, sql: "" },
        Migration { id: 2, sql: "" },
        Migration { id: 3, sql: "" },
        Migration {
            id: 4,
            sql: "UPDATE data SET value=99; INSERT INTO missing VALUES(1);",
        },
    ];
    assert!(migrations::initialize(&mut connection, &failing).is_err());
    assert_eq!(
        connection
            .query_row("SELECT value FROM data", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        6
    );
    let mut fresh = rusqlite::Connection::open_in_memory().unwrap();
    migrations::initialize(&mut fresh, &history).unwrap();
    assert_eq!(
        fresh
            .query_row("SELECT COUNT(*) FROM aetheris_migrations", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        3
    );
}
#[test]
fn process_owned_lease_excludes_a_second_instance_and_releases_on_drop() {
    let directory = Directory::new();
    let path = directory.0.join("lock");
    let lease = super::Lease::acquire(&path).unwrap();
    assert!(super::Lease::acquire(&path).is_err());
    drop(lease);
    assert!(super::Lease::acquire(&path).is_ok());
}

#[test]
fn update_lifecycle_respects_interaction_explicit_requests_and_closure() {
    let mut lifecycle = policy::UpdateLifecycle::default();
    lifecycle.background_download();
    assert!(lifecycle.should_close());
    assert!(lifecycle.unattended());
    lifecycle.interact();
    assert!(!lifecycle.should_close());
    lifecycle.request();
    lifecycle.interact();
    assert!(lifecycle.should_close());
    assert!(!lifecycle.unattended());
    lifecycle.blocked();
    assert!(!lifecycle.should_close());
    lifecycle.request();
    assert!(lifecycle.should_close());
    lifecycle.closing(true);
    assert!(lifecycle.cancelled());
    assert!(!lifecycle.should_close());
    let mut already_interacting = policy::UpdateLifecycle::default();
    already_interacting.interact();
    already_interacting.background_download();
    assert!(!already_interacting.should_close());
    let mut ready = policy::UpdateLifecycle::default();
    ready.request();
    ready.closing(false);
    assert!(!ready.cancelled());
}

#[test]
fn a_late_download_result_cannot_install_after_the_user_closed_during_download() {
    use std::time::{Duration, UNIX_EPOCH};
    let closing = UNIX_EPOCH + Duration::from_secs(10);
    assert!(policy::install_after_close(
        closing - Duration::from_secs(1),
        Some(closing)
    ));
    assert!(!policy::install_after_close(
        closing + Duration::from_secs(1),
        Some(closing)
    ));
}
