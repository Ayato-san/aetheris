//! Ordered schema history. Call inside an updater-owned transaction, or a fresh
//! database's creation transaction; never infer history from release numbers.
use super::{Result, error};
use rusqlite::{Connection, Transaction};

pub struct Migration {
    pub id: i64,
    pub sql: &'static str,
}
pub fn reconcile(tx: &Transaction<'_>, history: &[Migration]) -> Result<()> {
    tx.execute_batch("CREATE TABLE IF NOT EXISTS aetheris_migrations (id INTEGER PRIMARY KEY);")?;
    let applied: Vec<i64> = tx
        .prepare("SELECT id FROM aetheris_migrations ORDER BY id")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    if history.windows(2).any(|pair| pair[0].id >= pair[1].id) || history.iter().any(|m| m.id <= 0)
    {
        return Err(error("Historique de migrations invalide."));
    }
    if applied.len() > history.len() || applied.iter().zip(history).any(|(id, m)| *id != m.id) {
        return Err(error("Historique SQLite incompatible avec cette version."));
    }
    for migration in &history[applied.len()..] {
        tx.execute_batch(migration.sql)?;
        tx.execute(
            "INSERT INTO aetheris_migrations(id) VALUES (?1)",
            [migration.id],
        )?;
    }
    Ok(())
}
pub fn initialize(connection: &mut Connection, history: &[Migration]) -> Result<()> {
    let tx = connection.transaction()?;
    reconcile(&tx, history)?;
    tx.commit()?;
    Ok(())
}

/// Managed updates never open another Windows user's databases as SYSTEM.
/// Schema backups and restoration run under the user's application lease.
pub fn initialize_personal(
    connection: &mut Connection,
    path: &std::path::Path,
    history: &[Migration],
) -> Result<()> {
    let has_history: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='aetheris_migrations')", [], |row| row.get(0)
    )?;
    let applied: Vec<i64> = if has_history {
        connection
            .prepare("SELECT id FROM aetheris_migrations ORDER BY id")?
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?
    } else {
        Vec::new()
    };
    let expected: Vec<i64> = history.iter().map(|m| m.id).collect();
    if applied == expected {
        return initialize(connection, history);
    }
    let snapshot = |ids: &[i64]| {
        use sha2::{Digest, Sha256};
        path.with_extension(format!(
            "schema-{}.sqlite3",
            hex::encode(Sha256::digest(serde_json::to_vec(ids).unwrap()))
        ))
    };
    if applied.len() > expected.len() && applied.starts_with(&expected) {
        let backup = snapshot(&expected);
        if !backup.is_file() {
            return Err(error(
                "Sauvegarde du schéma précédent absente ; restauration manuelle nécessaire.",
            ));
        }
        let preserved = path.with_extension(format!(
            "post-rollback-{}.sqlite3",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        connection.backup("main", &preserved, None::<fn(rusqlite::backup::Progress)>)?;
        std::fs::OpenOptions::new()
            .write(true)
            .open(&preserved)?
            .sync_all()?;
        connection.restore("main", backup, None::<fn(rusqlite::backup::Progress)>)?;
    } else if !applied.is_empty() && expected.starts_with(&applied) {
        let backup = snapshot(&applied);
        let temporary = backup.with_extension("pending");
        connection.backup("main", &temporary, None::<fn(rusqlite::backup::Progress)>)?;
        std::fs::OpenOptions::new()
            .write(true)
            .open(&temporary)?
            .sync_all()?;
        super::durable_rename(&temporary, &backup)?;
    }
    initialize(connection, history)
}

#[cfg(test)]
mod personal_tests {
    use super::*;
    use crate::companion::tests::Directory;
    #[test]
    fn personal_schema_rollback_restores_old_data_and_preserves_new_data() {
        let directory = Directory::new();
        let path = directory.0.join("personal.sqlite3");
        let history = [
            Migration {
                id: 1,
                sql: "CREATE TABLE entries(value TEXT);",
            },
            Migration {
                id: 2,
                sql: "ALTER TABLE entries ADD COLUMN extra TEXT;",
            },
        ];
        let mut connection = Connection::open(&path).unwrap();
        initialize_personal(&mut connection, &path, &history[..1]).unwrap();
        connection
            .execute("INSERT INTO entries(value) VALUES ('before')", [])
            .unwrap();
        initialize_personal(&mut connection, &path, &history).unwrap();
        connection
            .execute("INSERT INTO entries(value) VALUES ('after')", [])
            .unwrap();
        initialize_personal(&mut connection, &path, &history[..1]).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM entries", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
        let preserved = std::fs::read_dir(&directory.0)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.to_string_lossy().contains("post-rollback"))
            .unwrap();
        let saved = Connection::open(preserved).unwrap();
        assert_eq!(
            saved
                .query_row("SELECT count(*) FROM entries", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            2
        );
    }
}
