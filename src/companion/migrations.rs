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
