//! A durable write-ahead journal and complete backups bridge SQLite and file
//! replacement. Recovery is idempotent and runs before integrity checking.
use super::{
    Result, durable_write, error,
    manifest::{self, MANIFEST, PROGRAMS, SIGNATURE},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const TRANSACTION: &str = ".aetheris-transaction";
const PREVIOUS: &str = ".aetheris-previous";
const FILES: [&str; 5] = [PROGRAMS[0], PROGRAMS[1], PROGRAMS[2], MANIFEST, SIGNATURE];
#[derive(Serialize, Deserialize)]
struct Journal {
    #[serde(default)]
    restoring: bool,
    staged: PathBuf,
    present: Vec<String>,
    databases: Vec<DatabaseBackup>,
}
#[derive(Serialize, Deserialize)]
struct DatabaseBackup {
    path: PathBuf,
    backup: String,
}

fn copy_sync(from: &Path, to: &Path) -> Result<()> {
    if fs::symlink_metadata(from)?.file_type().is_symlink() {
        return Err(error("Lien symbolique interdit dans une installation."));
    }
    // Every executable replacement must be atomic, especially the companion:
    // a power loss must leave a runnable old or new companion to read the journal.
    // create_new also ensures a temporary name cannot overwrite personal data.
    let mut random = [0; 16];
    getrandom::fill(&mut random).map_err(|cause| error(cause.to_string()))?;
    let temporary = to
        .parent()
        .ok_or_else(|| error("Dossier cible absent."))?
        .join(format!(".aetheris-copy-{}", hex::encode(random)));
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| -> Result<()> {
        let mut input = fs::File::open(from)?;
        std::io::copy(&mut input, &mut output)?;
        fs::set_permissions(&temporary, input.metadata()?.permissions())?;
        output.sync_all()?;
        drop(output);
        super::program_files::replace(&temporary, to)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    #[cfg(unix)]
    fs::File::open(to.parent().unwrap())?.sync_all()?;
    Ok(())
}

pub fn recover(root: &Path) -> Result<bool> {
    let transaction = root.join(TRANSACTION);
    if !transaction.join("journal.json").exists() {
        return Ok(false);
    }
    super::program_files::wait(root, |_| {})?;
    restore(root, &transaction)?;
    let journal: Journal = serde_json::from_slice(&fs::read(transaction.join("journal.json"))?)?;
    if journal.restoring && root.join(PREVIOUS).join("first-launch").exists() {
        fs::remove_file(root.join(PREVIOUS).join("first-launch"))?;
    }
    fs::remove_dir_all(transaction)?;
    Ok(true)
}

/// Called only by the temporary helper, with the installation lease held and
/// both application and companion stopped. Personal files are never enumerated
/// for deletion or replaced by package contents.
pub fn apply(root: &Path, staged: &Path, databases: &[PathBuf]) -> Result<()> {
    apply_with_key(root, staged, databases, &manifest::public_key()?)
}

pub fn apply_with_progress(
    root: &Path,
    staged: &Path,
    databases: &[PathBuf],
    progress: impl Fn(&str),
) -> Result<()> {
    apply_with_key_and_progress(root, staged, databases, &manifest::public_key()?, progress)
}

fn apply_with_key(
    root: &Path,
    staged: &Path,
    databases: &[PathBuf],
    key: &ed25519_dalek::VerifyingKey,
) -> Result<()> {
    apply_with_key_and_progress(root, staged, databases, key, |_| {})
}

fn apply_with_key_and_progress(
    root: &Path,
    staged: &Path,
    databases: &[PathBuf],
    key: &ed25519_dalek::VerifyingKey,
    progress: impl Fn(&str),
) -> Result<()> {
    progress("Vérification de la mise à jour…");
    manifest::verify(staged, key)?;
    progress("Préparation de l’installation…");
    super::program_files::wait(root, &progress)?;
    recover(root)?;
    let transaction = root.join(TRANSACTION);
    if transaction.exists() {
        fs::remove_dir_all(&transaction)?;
    }
    fs::create_dir(&transaction)?; // Also probes write permission before touching programs.
    let mut journal = Journal {
        restoring: false,
        staged: staged.to_owned(),
        present: Vec::new(),
        databases: Vec::new(),
    };
    progress("Sauvegarde de la version actuelle et des données…");
    for name in FILES {
        if root.join(name).exists() {
            copy_sync(&root.join(name), &transaction.join(name))?;
            journal.present.push(name.into());
        }
    }
    // SQLite online backup includes committed WAL data. All connections in the
    // application have stopped; no raw copying of a live WAL database is used.
    for (index, path) in databases.iter().enumerate() {
        if !path.exists() {
            continue;
        }
        let backup = format!("database-{index}.sqlite3");
        let connection = rusqlite::Connection::open(path)?;
        connection.backup(
            "main",
            transaction.join(&backup),
            None::<fn(rusqlite::backup::Progress)>,
        )?;
        fs::OpenOptions::new()
            .write(true)
            .open(transaction.join(&backup))?
            .sync_all()?;
        journal.databases.push(DatabaseBackup {
            path: path.clone(),
            backup,
        });
    }
    durable_write(&transaction.join("first-launch"), b"pending")?;
    durable_write(
        &transaction.join("journal.json"),
        &serde_json::to_vec(&journal)?,
    )?;
    let result = (|| -> Result<()> {
        progress("Mise à jour des données…");
        // Migrations run in the NEW helper, so releases skipping intermediates
        // still have the complete history compiled into that release.
        let mut connections = Vec::new();
        for database in &journal.databases {
            connections.push(rusqlite::Connection::open(&database.path)?);
        }
        let mut transactions = Vec::new();
        for (connection, database) in connections.iter_mut().zip(&journal.databases) {
            let tx = connection.transaction()?;
            let history = if database
                .path
                .file_name()
                .is_some_and(|name| name == crate::combat_history::storage::DATABASE_NAME)
            {
                crate::combat_history::storage::history()
            } else {
                database_history()
            };
            crate::companion::migrations::reconcile(&tx, history)?;
            transactions.push(tx);
        }
        progress("Installation des nouveaux fichiers…");
        for name in FILES {
            copy_sync(&staged.join(name), &root.join(name))?;
        }
        progress("Vérification de l’installation…");
        manifest::verify(root, key)?;
        for tx in transactions {
            tx.commit()?;
        }
        Ok(())
    })();
    if let Err(cause) = result {
        progress("Restauration de la version précédente après un échec…");
        if let Err(rollback) = restore(root, &transaction) {
            return Err(error(format!(
                "Installation échouée : {cause}. Restauration également impossible : {rollback}. Le journal de récupération est conservé."
            )));
        }
        fs::remove_dir_all(&transaction)?;
        return Err(cause);
    }
    let previous = root.join(PREVIOUS);
    if previous.exists() {
        fs::remove_dir_all(&previous)?;
    }
    // The journal remains a rollback instruction until the successful directory
    // rename. A power loss before it completes restores the old installation.
    super::durable_rename(&transaction, &previous)?;
    #[cfg(unix)]
    fs::File::open(root)?.sync_all()?;
    Ok(())
}

pub fn database_history() -> &'static [super::migrations::Migration] {
    &[
        super::migrations::Migration {
            id: 1,
            sql: include_str!("capture_schema.sql"),
        },
        super::migrations::Migration {
            id: 2,
            sql: include_str!("capture_provenance.sql"),
        },
        super::migrations::Migration {
            id: 3,
            sql: include_str!("capture_opcode_sections.sql"),
        },
        super::migrations::Migration {
            id: 4,
            sql: include_str!("capture_delete_indexes.sql"),
        },
    ]
}

pub fn has_previous(root: &Path) -> bool {
    root.join(PREVIOUS).join("journal.json").is_file()
}
pub fn can_restore(root: &Path) -> bool {
    manifest::public_key()
        .and_then(|key| manifest::verify(&root.join(PREVIOUS), &key))
        .is_ok()
}
pub fn restore_previous(root: &Path) -> Result<()> {
    restore_previous_with_key(root, &manifest::public_key()?)
}
fn restore_previous_with_key(root: &Path, key: &ed25519_dalek::VerifyingKey) -> Result<()> {
    let previous = root.join(PREVIOUS);
    manifest::verify(&previous, key)?;
    super::program_files::wait(root, |_| {})?;
    let mut journal: Journal = serde_json::from_slice(&fs::read(previous.join("journal.json"))?)?;
    journal.restoring = true;
    for database in &journal.databases {
        if database.path.exists() {
            let connection = rusqlite::Connection::open(&database.path)?;
            let path = database.path.with_extension(format!(
                "post-crash-{}.sqlite3",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_nanos()
            ));
            connection.backup("main", &path, None::<fn(rusqlite::backup::Progress)>)?;
            fs::OpenOptions::new().write(true).open(path)?.sync_all()?;
        }
    }
    // Keep a recovery journal while restoring, including across a power loss.
    let transaction = root.join(TRANSACTION);
    if transaction.exists() {
        return Err(error("Récupération déjà en attente."));
    }
    fs::create_dir(&transaction)?;
    for entry in fs::read_dir(&previous)? {
        let entry = entry?;
        if entry.file_name() != "journal.json" {
            copy_sync(&entry.path(), &transaction.join(entry.file_name()))?;
        }
    }
    durable_write(
        &transaction.join("journal.json"),
        &serde_json::to_vec(&journal)?,
    )?;
    restore(root, &transaction)?;
    if previous.join("first-launch").exists() {
        fs::remove_file(previous.join("first-launch"))?;
    }
    fs::remove_dir_all(transaction)?;
    Ok(())
}

fn restore(root: &Path, backup: &Path) -> Result<()> {
    let journal: Journal = serde_json::from_slice(&fs::read(backup.join("journal.json"))?)?;
    for name in FILES {
        if journal.present.iter().any(|present| present == name) {
            copy_sync(&backup.join(name), &root.join(name))?;
        } else if root.join(name).exists() {
            fs::remove_file(root.join(name))?;
        }
    }
    for database in journal.databases {
        let mut connection = rusqlite::Connection::open(&database.path)?;
        connection.restore(
            "main",
            backup.join(database.backup),
            None::<fn(rusqlite::backup::Progress)>,
        )?;
    }
    Ok(())
}

/// Recovery must run from an authenticated, complete copy of the helper,
/// because executable replacement may have stopped halfway through.
pub fn recovery_source(root: &Path) -> Result<PathBuf> {
    let backup = root.join(TRANSACTION);
    let journal: Journal = serde_json::from_slice(&fs::read(backup.join("journal.json"))?)?;
    let key = manifest::public_key()?;
    if manifest::verify(&journal.staged, &key).is_ok() {
        return Ok(journal.staged);
    }
    manifest::verify(&backup, &key)?;
    Ok(backup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion::tests::{Directory, signed};
    use ed25519_dalek::SigningKey;
    #[cfg(windows)]
    #[test]
    fn repair_and_update_wait_for_mapped_programs_after_their_lease_is_released() {
        use crate::companion::program_files::tests::Fixture;
        use ed25519_dalek::Signer;
        use std::{sync::mpsc, time::Duration};
        for (program, incoming) in [
            (PROGRAMS[0], "1.1.0"),
            (PROGRAMS[1], "1.0.0"),
            (PROGRAMS[1], "1.1.0"),
        ] {
            let directory = Directory::new();
            let root = directory.0.join("installed");
            let stage = directory.0.join("stage");
            let key = SigningKey::from_bytes(&[7; 32]);
            signed(&root, "1.0.0", &key);
            signed(&stage, incoming, &key);
            Fixture::prepare(&root, program);
            // Authenticate the real fixture image as the old installed companion.
            let mut old: manifest::Manifest =
                serde_json::from_slice(&fs::read(root.join(MANIFEST)).unwrap()).unwrap();
            old.files
                .insert(program.into(), manifest::hash(&root.join(program)).unwrap());
            let bytes = serde_json::to_vec(&old).unwrap();
            fs::write(root.join(MANIFEST), &bytes).unwrap();
            fs::write(root.join(SIGNATURE), key.sign(&bytes).to_bytes()).unwrap();
            let mut child = Fixture::spawn(&root, program);
            let _cooperative =
                crate::companion::Lease::acquire(&root.join("cooperative.lock")).unwrap();
            let (waiting, waited) = mpsc::channel();
            let install_root = root.clone();
            let worker = std::thread::spawn(move || {
                apply_with_key_and_progress(
                    &install_root,
                    &stage,
                    &[],
                    &key.verifying_key(),
                    |message| {
                        if message.starts_with("Attente de la fermeture") {
                            assert!(!install_root.join(TRANSACTION).exists());
                            let _ = waiting.send(());
                        }
                    },
                )
            });
            if waited.recv_timeout(Duration::from_secs(10)).is_err() {
                child.release();
                panic!(
                    "replacement failed to wait for the image: {:?}",
                    worker.join()
                );
            }
            assert!(child.child.try_wait().unwrap().is_none());
            child.release();
            worker.join().unwrap().unwrap();
            assert_eq!(
                manifest::verify(&root, &SigningKey::from_bytes(&[7; 32]).verifying_key())
                    .unwrap()
                    .version
                    .to_string(),
                incoming
            );
            assert!(!root.join(TRANSACTION).exists());
        }
    }
    #[test]
    fn installation_backs_up_history_and_uses_its_own_schema() {
        let directory = Directory::new();
        let root = directory.0.join("installed");
        let stage = directory.0.join("stage");
        let key = SigningKey::from_bytes(&[7; 32]);
        signed(&root, "1.0.0", &key);
        signed(&stage, "1.1.0", &key);
        let capture = directory.0.join("debug-captures.sqlite3");
        let history = directory
            .0
            .join(crate::combat_history::storage::DATABASE_NAME);
        drop(crate::combat_debug::storage::Store::open(&capture).unwrap());
        drop(crate::combat_history::storage::History::open(&history).unwrap());
        let connection = rusqlite::Connection::open(&history).unwrap();
        connection
            .execute("INSERT INTO combat_reports(report) VALUES ('snapshot')", [])
            .unwrap();
        apply_with_key(
            &root,
            &stage,
            &[capture, history.clone()],
            &key.verifying_key(),
        )
        .unwrap();
        let saved =
            rusqlite::Connection::open(root.join(PREVIOUS).join("database-1.sqlite3")).unwrap();
        assert_eq!(
            saved
                .query_row("SELECT report FROM combat_reports", [], |row| row
                    .get::<_, String>(0))
                .unwrap(),
            "snapshot"
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM aetheris_migrations", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE name='sessions'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        connection
            .execute("UPDATE combat_reports SET report='after update'", [])
            .unwrap();
        restore_previous_with_key(&root, &key.verifying_key()).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT report FROM combat_reports", [], |row| row
                    .get::<_, String>(0))
                .unwrap(),
            "snapshot"
        );
    }
    #[test]
    fn installation_preserves_personal_files_and_backs_up_wal_data() {
        let directory = Directory::new();
        let root = directory.0.join("installed");
        let stage = directory.0.join("stage");
        let key = SigningKey::from_bytes(&[7; 32]);
        signed(&root, "1.0.0", &key);
        signed(&stage, "1.1.0", &key);
        fs::write(root.join("Engine.baseline.ini"), "personal baseline").unwrap();
        fs::write(root.join("settings.txt"), "personal settings").unwrap();
        let database = directory.0.join("data.sqlite3");
        let connection = rusqlite::Connection::open(&database).unwrap();
        connection.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE personal(value); INSERT INTO personal VALUES('committed WAL');").unwrap();
        let saw_backup = std::cell::Cell::new(false);
        let saw_installed = std::cell::Cell::new(false);
        apply_with_key_and_progress(
            &root,
            &stage,
            std::slice::from_ref(&database),
            &key.verifying_key(),
            |message| {
                if message.starts_with("Sauvegarde") {
                    assert_eq!(
                        manifest::verify(&root, &key.verifying_key())
                            .unwrap()
                            .version
                            .to_string(),
                        "1.0.0"
                    );
                    saw_backup.set(true);
                }
                if message.starts_with("Vérification de l’installation") {
                    assert_eq!(
                        manifest::verify(&root, &key.verifying_key())
                            .unwrap()
                            .version
                            .to_string(),
                        "1.1.0"
                    );
                    saw_installed.set(true);
                }
            },
        )
        .unwrap();
        assert!(saw_backup.get() && saw_installed.get());
        assert_eq!(
            manifest::verify(&root, &key.verifying_key())
                .unwrap()
                .version
                .to_string(),
            "1.1.0"
        );
        assert_eq!(
            fs::read_to_string(root.join("Engine.baseline.ini")).unwrap(),
            "personal baseline"
        );
        assert_eq!(
            fs::read_to_string(root.join("settings.txt")).unwrap(),
            "personal settings"
        );
        assert!(root.join(PREVIOUS).join("first-launch").exists());
        let backup =
            rusqlite::Connection::open(root.join(PREVIOUS).join("database-0.sqlite3")).unwrap();
        assert_eq!(
            backup
                .query_row("SELECT value FROM personal", [], |row| row
                    .get::<_, String>(0))
                .unwrap(),
            "committed WAL"
        );
        assert!(has_previous(&root));
        assert!(!recover(&root).unwrap());
    }
    #[test]
    fn migration_failure_restores_programs_and_all_databases() {
        let directory = Directory::new();
        let root = directory.0.join("installed");
        let stage = directory.0.join("stage");
        let key = SigningKey::from_bytes(&[7; 32]);
        signed(&root, "1.0.0", &key);
        signed(&stage, "1.1.0", &key);
        let paths = [
            directory.0.join("first.sqlite3"),
            directory.0.join("second.sqlite3"),
        ];
        for path in &paths {
            rusqlite::Connection::open(path).unwrap().execute_batch("CREATE TABLE aetheris_migrations(id INTEGER PRIMARY KEY); INSERT INTO aetheris_migrations VALUES(999);").unwrap();
        }
        assert!(apply_with_key(&root, &stage, &paths, &key.verifying_key()).is_err());
        assert_eq!(
            manifest::verify(&root, &key.verifying_key())
                .unwrap()
                .version
                .to_string(),
            "1.0.0"
        );
        for path in &paths {
            assert_eq!(
                rusqlite::Connection::open(path)
                    .unwrap()
                    .query_row("SELECT id FROM aetheris_migrations", [], |r| r
                        .get::<_, i64>(0))
                    .unwrap(),
                999
            );
        }
        assert!(!root.join(TRANSACTION).exists());
    }
    #[test]
    fn interrupted_replacement_restores_consistently_and_recovery_is_idempotent() {
        let directory = Directory::new();
        let root = directory.0.join("installed");
        let stage = directory.0.join("stage");
        let key = SigningKey::from_bytes(&[7; 32]);
        signed(&root, "1.0.0", &key);
        signed(&stage, "1.1.0", &key);
        let transaction = root.join(TRANSACTION);
        fs::create_dir(&transaction).unwrap();
        for name in FILES {
            copy_sync(&root.join(name), &transaction.join(name)).unwrap();
        }
        let journal = Journal {
            restoring: false,
            staged: stage.clone(),
            present: FILES.iter().map(|s| s.to_string()).collect(),
            databases: vec![],
        };
        durable_write(
            &transaction.join("journal.json"),
            &serde_json::to_vec(&journal).unwrap(),
        )
        .unwrap();
        // Power loss after only one of the executable files was replaced.
        copy_sync(&stage.join(PROGRAMS[0]), &root.join(PROGRAMS[0])).unwrap();
        assert!(manifest::verify(&root, &key.verifying_key()).is_err());
        assert!(recover(&root).unwrap());
        assert_eq!(
            manifest::verify(&root, &key.verifying_key())
                .unwrap()
                .version
                .to_string(),
            "1.0.0"
        );
        assert!(!recover(&root).unwrap());
    }
    #[test]
    fn tampered_download_never_touches_the_installed_version() {
        let directory = Directory::new();
        let root = directory.0.join("installed");
        let stage = directory.0.join("stage");
        let key = SigningKey::from_bytes(&[7; 32]);
        signed(&root, "1.0.0", &key);
        signed(&stage, "1.1.0", &key);
        fs::write(stage.join(PROGRAMS[2]), "corrupt helper").unwrap();
        assert!(apply_with_key(&root, &stage, &[], &key.verifying_key()).is_err());
        assert_eq!(
            manifest::verify(&root, &key.verifying_key())
                .unwrap()
                .version
                .to_string(),
            "1.0.0"
        );
        assert!(!root.join(TRANSACTION).exists());
    }
}

#[cfg(test)]
mod restoration_tests {
    use super::*;
    use crate::companion::tests::{Directory, signed};
    use ed25519_dalek::SigningKey;
    #[test]
    fn first_crash_restoration_preserves_post_crash_data_and_activates_old_backup() {
        let directory = Directory::new();
        let root = directory.0.join("installed");
        let stage = directory.0.join("stage");
        let key = SigningKey::from_bytes(&[7; 32]);
        signed(&root, "1.0.0", &key);
        signed(&stage, "1.1.0", &key);
        let database = directory.0.join("data.sqlite3");
        let connection = rusqlite::Connection::open(&database).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE personal(value); INSERT INTO personal VALUES('before update');",
            )
            .unwrap();
        drop(connection);
        apply_with_key(
            &root,
            &stage,
            std::slice::from_ref(&database),
            &key.verifying_key(),
        )
        .unwrap();
        let connection = rusqlite::Connection::open(&database).unwrap();
        connection
            .execute_batch("UPDATE personal SET value='after crash';")
            .unwrap();
        drop(connection);
        restore_previous_with_key(&root, &key.verifying_key()).unwrap();
        assert_eq!(
            manifest::verify(&root, &key.verifying_key())
                .unwrap()
                .version
                .to_string(),
            "1.0.0"
        );
        let connection = rusqlite::Connection::open(&database).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT value FROM personal", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "before update"
        );
        let post_crash = fs::read_dir(&directory.0)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .find(|entry| entry.file_name().to_string_lossy().contains("post-crash"))
            .unwrap();
        let preserved = rusqlite::Connection::open(post_crash.path()).unwrap();
        assert_eq!(
            preserved
                .query_row("SELECT value FROM personal", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "after crash"
        );
        assert!(!root.join(PREVIOUS).join("first-launch").exists());
        assert!(!recover(&root).unwrap());
    }
}
