//! Reports are immutable JSON snapshots in a separate, migrated user database.
use super::*;
use crate::companion::migrations::{Migration, initialize_personal};
use rusqlite::{Connection, params};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

pub const DATABASE_NAME: &str = "combat-history.sqlite3";
pub fn default_path() -> Result<PathBuf> {
    Ok(crate::combat_debug::storage::default_path()?.with_file_name(DATABASE_NAME))
}
pub fn history() -> &'static [Migration] {
    &[Migration {
        id: 1,
        sql: "CREATE TABLE combat_reports(id INTEGER PRIMARY KEY AUTOINCREMENT, created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S UTC','now')), report TEXT NOT NULL);",
    }]
}
pub struct History {
    connection: Connection,
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub id: i64,
    pub created_at: String,
    pub report: Report,
}
impl History {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let mut connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")?;
        initialize_personal(&mut connection, path, history())?;
        Ok(Self { connection })
    }
    /// Construct and validate the snapshot before inserting; each call appends.
    pub fn save(&self, analysis: &CaptureAnalysis, selection: Selection) -> Result<i64> {
        let report = analysis.report(selection)?;
        self.connection.execute(
            "INSERT INTO combat_reports(report) VALUES (?1)",
            [serde_json::to_string(&report)?],
        )?;
        Ok(self.connection.last_insert_rowid())
    }
    pub fn entries(&self) -> Result<Vec<Entry>> {
        let mut statement = self
            .connection
            .prepare("SELECT id,created_at,report FROM combat_reports ORDER BY id DESC")?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        rows.map(|row| {
            let (id, created_at, json) = row?;
            Ok(Entry {
                id,
                created_at,
                report: serde_json::from_str(&json)?,
            })
        })
        .collect()
    }
    pub fn delete(&self, id: i64) -> Result<()> {
        self.connection
            .execute("DELETE FROM combat_reports WHERE id=?1", params![id])?;
        Ok(())
    }
}
