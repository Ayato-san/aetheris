//! Skill projection builder. Catalogue publication and comparisons have one owner.
use super::{assets::Localization, binary::invalid, catalog::Result, usmap::TableProbe};
use rusqlite::{Connection, OptionalExtension, params, types::Value as SqlValue};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
#[path = "skill_projection.rs"]
mod projection;
#[path = "skill_versions.rs"]
mod versions;
#[cfg(test)]
use super::skills;
#[cfg(test)]
#[path = "skill_database_tests.rs"]
mod tests;
pub(crate) fn fingerprints(
    db: &Connection,
    localized: bool,
) -> Result<BTreeMap<i64, (String, i64)>> {
    versions::snapshot(db, localized)
}
pub(crate) fn write(
    db: &Connection,
    tables: &BTreeMap<String, TableProbe>,
    en: &Localization,
    localized: &Localization,
    version: i64,
) -> Result<()> {
    db.execute_batch(include_str!("skills.sql"))?;
    projection::write(db, tables, en, localized, version)
}
#[cfg(test)]
fn integrity(db: &Connection) -> Result<()> {
    if db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))? != "ok"
        || db.prepare("PRAGMA foreign_key_check")?.exists([])?
    {
        return Err(invalid("invalid skill graph").into());
    }
    Ok(())
}
#[cfg(test)]
fn skill_tables(db: &Connection) -> Result<Vec<(String, String)>> {
    Ok(db.prepare("SELECT name,sql FROM sqlite_schema WHERE type='table' AND (name='skills' OR name GLOB 'skill_*')")?.query_map([],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?)
}
#[cfg(test)]
fn skill_views(db: &Connection) -> Result<Vec<(String, String)>> {
    Ok(db
        .prepare("SELECT name,sql FROM sqlite_schema WHERE type='view' AND name GLOB 'skill_*'")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?)
}
