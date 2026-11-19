//! Export only the skill product model and its description dependencies.
use super::{
    assets::Localization, binary::invalid, catalog::Result, discovery, skills, usmap::Mapping,
};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params, types::Value as SqlValue};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};
const APPLICATION_ID: i64 = 0x41454954;
#[path = "skill_projection.rs"]
mod projection;
#[cfg(test)]
#[path = "skill_database_tests.rs"]
mod tests;
#[path = "skill_versions.rs"]
mod versions;
#[derive(Default, Debug, Serialize)]
pub struct Summary {
    pub steam_build: String,
    pub localized_locale: String,
    pub source_tables: usize,
    pub records: usize,
    pub skills: usize,
    pub skill_effects: usize,
    pub skill_abnormals: usize,
    pub skill_abnormal_effects: usize,
    pub new_records: usize,
    pub modified_records: usize,
    pub unchanged_records: usize,
    pub missing_references: usize,
    pub output: PathBuf,
    pub tables: BTreeMap<String, usize>,
    pub source_sha256: BTreeMap<String, String>,
}

fn quote(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
fn skill_tables(db: &Connection) -> Result<Vec<(String, String)>> {
    Ok(db.prepare("SELECT name,sql FROM sqlite_schema WHERE type='table' AND (name='skills' OR name GLOB 'skill_*' OR name GLOB 'skills_*') ORDER BY name")?
        .query_map([],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?)
}
fn integrity(db: &Connection) -> Result<()> {
    if db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))? != "ok"
        || db.prepare("PRAGMA foreign_key_check")?.exists([])?
    {
        return Err(invalid("invalid shared SQLite integrity or foreign keys").into());
    }
    Ok(())
}
pub(crate) fn read_database(path: &Path) -> Result<Vec<u8>> {
    read(path, 512 * 1024 * 1024)
}
fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut data)?;
    if data.len() as u64 > limit {
        return Err(invalid("skill export input exceeds size limit").into());
    }
    Ok(data)
}
struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// The output is the shared database; all item tables survive a skill refresh.
pub fn export(
    mapping_path: &Path,
    directory: &Path,
    english_path: &Path,
    localized_path: &Path,
    manifest: &Path,
    output: &Path,
    locale: &str,
) -> Result<Summary> {
    if locale.is_empty()
        || !locale
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-')
    {
        return Err(invalid("invalid skill locale").into());
    }
    if output.exists() {
        let destination = output.canonicalize()?;
        let mut inputs = skills::source_paths(directory);
        inputs.extend([
            mapping_path.to_owned(),
            english_path.to_owned(),
            localized_path.to_owned(),
            manifest.to_owned(),
        ]);
        for input in inputs {
            if input.canonicalize()? == destination {
                return Err(invalid("skill database must not replace its source").into());
            }
        }
    }
    let mapping_bytes = read(mapping_path, 64 * 1024 * 1024)?;
    let mapping = Mapping::parse(&mapping_bytes)?;
    let probe = skills::probe(&mapping, directory)?;
    let english_bytes = read(english_path, 64 * 1024 * 1024)?;
    let localized_bytes = read(localized_path, 64 * 1024 * 1024)?;
    let english: Localization = serde_json::from_slice(&english_bytes)?;
    let localized: Localization = serde_json::from_slice(&localized_bytes)?;
    if english.namespace != "AION2" || localized.namespace != "AION2" {
        return Err(invalid("unexpected skill localization namespace").into());
    }
    let steam = discovery::steam_manifest(manifest)?;
    let build = steam.build_id.parse::<u64>()?;
    if steam.app_id != "3393110" || build == 0 {
        return Err(invalid("invalid skill client build").into());
    }
    let original = if output.exists() {
        Some(read(output, 512 * 1024 * 1024)?)
    } else {
        None
    };
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|e| std::io::Error::other(e.to_string()))?;
    let temporary = Temporary(parent.join(format!(".skills-{}.sqlite3", hex::encode(random))));
    if original.is_some() {
        let db = Connection::open_with_flags(output, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        if db.query_row("PRAGMA application_id", [], |r| r.get::<_, i64>(0))? != APPLICATION_ID
            || !matches!(
                db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))?,
                4 | 5
            )
        {
            return Err(invalid("unsupported shared database").into());
        }
        integrity(&db)?;
        db.backup("main", &temporary.0, None::<fn(rusqlite::backup::Progress)>)?;
    }
    let mut db = Connection::open(&temporary.0)?;
    if original.is_none() {
        db.execute_batch(include_str!("items.sql"))?;
    }
    let versions = db
        .prepare("SELECT steam_build FROM client_version")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if versions
        .iter()
        .any(|v| v.parse::<u64>().map_or(true, |v| v > build))
    {
        return Err(invalid("older skill client build").into());
    }
    let previous_locale: Option<String> = db
        .query_row(
            "SELECT locale FROM localization_language WHERE id=1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    let same_locale = previous_locale.as_deref() == Some(locale);
    if previous_locale.is_some()
        && !same_locale
        && db.prepare("SELECT 1 FROM items LIMIT 1")?.exists([])?
    {
        return Err(invalid(
            "shared localization language differs: regenerate items and skills in the same locale",
        )
        .into());
    }
    let previous = versions::snapshot(&db, same_locale)?;
    db.execute_batch("PRAGMA foreign_keys=OFF")?;
    let tx = db.transaction()?;
    for sql in skill_views(&tx)? {
        // All views are recreated from the current projection below.
        tx.execute_batch(&format!("DROP VIEW {}", quote(&sql.0)))?;
    }
    for (name, _) in skill_tables(&tx)? {
        tx.execute_batch(&format!("DROP TABLE {}", quote(&name)))?;
    }
    tx.execute_batch(include_str!("skills.sql"))?;
    let version = reference_version(&tx, &steam.build_id)?;
    if !tx
        .prepare("SELECT id FROM localization_language")?
        .exists([])?
    {
        tx.execute("INSERT INTO localization_language VALUES(1,?1)", [locale])?;
    }
    tx.execute(
        "INSERT OR REPLACE INTO localization_language VALUES (1,?1)",
        [locale],
    )?;
    let mut summary = Summary {
        steam_build: steam.build_id,
        localized_locale: locale.into(),
        source_tables: probe.tables.len(),
        missing_references: probe
            .relations
            .iter()
            .filter(|r| !r.target_may_be_partial)
            .map(|r| r.missing)
            .sum(),
        output: output.to_owned(),
        ..Summary::default()
    };
    summary.source_sha256.insert(
        "mapping".into(),
        hex::encode(Sha256::digest(&mapping_bytes)),
    );
    summary.source_sha256.insert(
        "english".into(),
        hex::encode(Sha256::digest(&english_bytes)),
    );
    summary.source_sha256.insert(
        "localized".into(),
        hex::encode(Sha256::digest(&localized_bytes)),
    );
    summary
        .source_sha256
        .insert("steam_manifest".into(), steam.sha256);
    projection::write(&tx, &probe, &english, &localized, version)?;
    let current = versions::snapshot(&tx, same_locale)?;
    for (id, (hash, _)) in current {
        match previous.get(&id) {
            Some((old_hash, old_version)) if *old_hash == hash => {
                tx.execute(
                    "UPDATE skills SET client_version_id=?1 WHERE id=?2",
                    params![old_version, id],
                )?;
                summary.unchanged_records += 1;
            }
            Some(_) => summary.modified_records += 1,
            None => summary.new_records += 1,
        }
    }
    for (source, table) in &probe.tables {
        summary
            .source_sha256
            .insert(source.clone(), table.payload_sha256.clone());
    }
    for (name, _) in skill_tables(&tx)? {
        let count: i64 =
            tx.query_row(&format!("SELECT count(*) FROM {}", quote(&name)), [], |r| {
                r.get(0)
            })?;
        summary.records += count as usize;
        summary.tables.insert(name, count as usize);
    }
    tx.pragma_update(None, "user_version", 5)?;
    tx.commit()?;
    db.execute_batch("PRAGMA foreign_keys=ON")?;
    // Reclaim pages from the former, wider development export before publishing.
    db.execute_batch("VACUUM")?;
    integrity(&db)?;
    summary.skills = summary.tables["skills"];
    summary.skill_effects = db.query_row(
        "SELECT count(*) FROM skill_effects WHERE kind_id=(SELECT id FROM skill_effect_kinds WHERE code='se')",
        [],
        |r| r.get::<_, i64>(0).map(|n| n as usize),
    )?;
    summary.skill_abnormal_effects = db.query_row(
        "SELECT count(*) FROM skill_effects WHERE kind_id=(SELECT id FROM skill_effect_kinds WHERE code='abe')",
        [],
        |r| r.get::<_, i64>(0).map(|n| n as usize),
    )?;
    summary.skill_abnormals = db.query_row(
        "SELECT count(DISTINCT abnormal_id) FROM skill_effects WHERE kind_id=(SELECT id FROM skill_effect_kinds WHERE code='abe')",
        [],
        |r| r.get::<_, i64>(0).map(|n| n as usize),
    )?;
    drop(db);
    let destination = parent.join(
        output
            .file_name()
            .ok_or_else(|| invalid("invalid skill database filename"))?,
    );
    if original.is_none() && destination.exists() {
        return Err(invalid("shared database appeared during skill export").into());
    }
    crate::engine_ini::storage::atomic_replace(
        &destination,
        &fs::read(&temporary.0)?,
        original.as_deref(),
    )?;
    Ok(summary)
}
fn reference_version(db: &Connection, build: &str) -> Result<i64> {
    if let Some(id) = db
        .query_row(
            "SELECT id FROM client_version WHERE steam_build=?1",
            [build],
            |r| r.get(0),
        )
        .optional()?
    {
        return Ok(id);
    }
    let id: i64 = db.query_row(
        "SELECT COALESCE(MAX(id),0)+1 FROM client_version",
        [],
        |r| r.get(0),
    )?;
    db.execute(
        "INSERT INTO client_version VALUES (?1,?2)",
        params![id, build],
    )?;
    Ok(id)
}

/// Preserve skills when the item exporter rebuilds the shared database.
pub(crate) fn preserve(db: &Connection, previous: &Path) -> Result<bool> {
    let source = Connection::open_with_flags(previous, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let tables = skill_tables(&source)?;
    if tables.is_empty() {
        return Ok(false);
    }
    let previous_locale: Option<String> = source
        .query_row(
            "SELECT locale FROM localization_language WHERE id=1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    let current_locale: Option<String> = db
        .query_row(
            "SELECT locale FROM localization_language WHERE id=1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    if previous_locale != current_locale
        && source
            .prepare("SELECT 1 FROM sqlite_schema WHERE name='skills'")?
            .exists([])?
        && source.prepare("SELECT 1 FROM skills LIMIT 1")?.exists([])?
    {
        return Err(invalid(
            "shared localization language differs: regenerate items and skills in the same locale",
        )
        .into());
    }
    db.execute(
        "ATTACH DATABASE ?1 AS previous_skills",
        [previous
            .to_str()
            .ok_or_else(|| invalid("non-UTF8 skill database path"))?],
    )?;
    for (_, sql) in &tables {
        db.execute_batch(sql)?;
    }
    for (name, _) in &tables {
        db.execute_batch(&format!(
            "INSERT INTO {} SELECT * FROM previous_skills.{}",
            quote(name),
            quote(name)
        ))?;
    }
    let indexes=source.prepare("SELECT sql FROM sqlite_schema WHERE type='index' AND sql IS NOT NULL AND (tbl_name='skills' OR tbl_name GLOB 'skill_*' OR tbl_name GLOB 'skills_*')")?.query_map([],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    for sql in indexes {
        db.execute_batch(&sql)?;
    }
    for (_, sql) in skill_views(&source)? {
        db.execute_batch(&sql)?;
    }
    Ok(true)
}

fn skill_views(db: &Connection) -> Result<Vec<(String, String)>> {
    Ok(db.prepare("SELECT name,sql FROM sqlite_schema WHERE type='view' AND (name GLOB 'skill_*' OR name GLOB 'skills_*')")?
        .query_map([], |r| Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?)
}
