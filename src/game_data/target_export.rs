//! Complete catalogue export. Legacy projections remain usable as independent
//! diagnostics; this coordinator owns evidence, coverage and build comparisons.
use super::{
    assets::Localization,
    binary::invalid,
    catalog::Result,
    discovery, items, skills,
    usmap::{Mapping, TableProbe},
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};
#[path = "achievements.rs"]
mod achievements;
#[cfg(test)]
#[path = "achievements_tests.rs"]
mod achievements_tests;
#[path = "catalogue_projection.rs"]
mod catalogue_projection;
#[path = "target_changes.rs"]
mod changes;
#[path = "companions.rs"]
mod companions;
#[cfg(test)]
#[path = "companions_tests.rs"]
mod companions_tests;
#[path = "crafting_quests.rs"]
mod crafting_quests;
#[path = "daevanion.rs"]
mod daevanion;
#[cfg(test)]
#[path = "daevanion_tests.rs"]
mod daevanion_tests;
#[path = "dungeon_rewards.rs"]
mod dungeon_rewards;
#[path = "target_loot_changes.rs"]
mod loot_changes;
#[path = "item_obtaining.rs"]
mod obtaining;
#[path = "target_projection.rs"]
mod projection;
#[path = "item_relations.rs"]
mod relations;
#[cfg(test)]
#[path = "target_export_tests.rs"]
mod tests;

pub(super) fn read(path: &Path) -> Result<Vec<u8>> {
    read_bounded(path, 512 * 1024 * 1024)
}
fn read_database(path: &Path) -> Result<Vec<u8>> {
    // The combined SQLite includes all source families, icons and projections.
    // Its bound is separate from the individual untrusted input table bound.
    read_bounded(path, 1024 * 1024 * 1024)
}
fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(invalid(format!("catalogue file exceeds {limit} byte limit")).into());
    }
    Ok(bytes)
}

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
// Reader/model revisions are independent of the game build and Git metadata.
fn reader_revision() -> String {
    let mut sha = Sha256::new();
    for source in [
        include_str!("target_export.rs"),
        include_str!("target_projection.rs"),
        include_str!("item_obtaining.rs"),
        include_str!("item_relations.rs"),
        include_str!("dungeon_rewards.rs"),
        include_str!("target_changes.rs"),
        include_str!("target_loot_changes.rs"),
        include_str!("crafting_quests.rs"),
        include_str!("crafting_quests.sql"),
        include_str!("catalogue_projection.rs"),
        include_str!("catalogue_projection.sql"),
        include_str!("achievements.rs"),
        include_str!("achievements.sql"),
        include_str!("companions.rs"),
        include_str!("companions.sql"),
        include_str!("daevanion.rs"),
        include_str!("daevanion.sql"),
        include_str!("daevanion_graph.rs"),
        include_str!("catalogue.sql"),
        include_str!("items.rs"),
        include_str!("items.sql"),
        include_str!("skill_projection.rs"),
        include_str!("skill_versions.rs"),
        include_str!("skills.sql"),
        include_str!("usmap.rs"),
        include_str!("schema.rs"),
    ] {
        sha.update((source.len() as u64).to_le_bytes());
        sha.update(source.as_bytes());
    }
    hex::encode(sha.finalize())
}
pub(super) fn number(row: &Value, field: &str) -> Option<i64> {
    row[field].as_i64().or_else(|| row[field]["Value"].as_i64())
}
pub(super) fn string(v: &Value) -> Option<&str> {
    v.as_str().filter(|s| !s.is_empty() && *s != "None")
}
pub(super) fn text(l: &Localization, key: Option<&str>) -> Option<String> {
    key.and_then(|k| {
        l.entries
            .get(k)
            .or_else(|| l.entries.get(&format!("String_{k}_body")))
    })
    .cloned()
}
pub(super) fn coverage(
    db: &Connection,
    subject: &str,
    state: &str,
    diagnostic: &str,
) -> Result<()> {
    db.execute(
        "INSERT OR REPLACE INTO catalogue_coverage VALUES(?1,?2,?3)",
        params![subject, state, diagnostic],
    )?;
    Ok(())
}
pub(super) fn dictionary(db: &Connection, table: &str, code: &str) -> Result<i64> {
    if let Some(id) = db
        .query_row(
            &format!("SELECT id FROM {table} WHERE code=?1"),
            [code],
            |r| r.get(0),
        )
        .optional()?
    {
        return Ok(id);
    }
    let id: i64 = db.query_row(
        &format!("SELECT COALESCE(MAX(id),0)+1 FROM {table}"),
        [],
        |r| r.get(0),
    )?;
    db.execute(
        &format!("INSERT INTO {table}(id,code) VALUES(?1,?2)"),
        params![id, code],
    )?;
    Ok(id)
}
pub(super) fn version(db: &Connection, build: &str) -> Result<i64> {
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
        "INSERT INTO client_version VALUES(?1,?2)",
        params![id, build],
    )?;
    Ok(id)
}

#[derive(Debug, Serialize, Default)]
pub struct Summary {
    pub output: PathBuf,
    pub steam_build: String,
    pub locale: String,
    pub items: usize,
    pub skills: usize,
    pub abnormals: usize,
    pub recipes: usize,
    pub quests: usize,
    pub achievements: usize,
    pub titles: usize,
    pub passes: usize,
    pub supply_requests: usize,
    pub seasons: usize,
    pub journals: usize,
    pub pets: usize,
    pub wings: usize,
    pub skins: usize,
    pub skin_sets: usize,
    pub daevanion_boards: usize,
    pub daevanion_nodes: usize,
    pub sources: usize,
    pub initial: usize,
    pub added: usize,
    pub modified: usize,
    pub deleted: usize,
    pub unchanged: usize,
    pub comparison_impossible: usize,
    pub coverage: BTreeMap<String, (String, String)>,
}
struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The directory must contain decrypted table payloads, not encrypted DATs.
/// Optional image_directory supplies already extracted PNG/WebP/JPEG files;
/// absence of such images is reported rather than synthesized.
#[allow(clippy::too_many_arguments)]
pub fn export(
    mapping_path: &Path,
    directory: &Path,
    english_path: &Path,
    localized_path: &Path,
    manifest: &Path,
    output: &Path,
    locale: &str,
    image_directory: Option<&Path>,
) -> Result<Summary> {
    let original = output.exists().then(|| read_database(output)).transpose()?;
    let previous = if output.exists() {
        Some(Connection::open_with_flags(
            output,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?)
    } else {
        None
    };
    if let Some(db) = &previous {
        if db.query_row("PRAGMA application_id", [], |r| r.get::<_, i64>(0))? != 0x41454954
            || !matches!(
                db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))?,
                15
            )
        {
            return Err(invalid("unsupported catalogue database").into());
        }
        integrity(db)?;
    }
    let steam = discovery::steam_manifest(manifest)?;
    let build = steam.build_id.parse::<u64>()?;
    if steam.app_id != "3393110" || build == 0 {
        return Err(invalid("expected identified AION 2 client").into());
    }
    if let Some(db) = &previous {
        for b in db
            .prepare("SELECT steam_build FROM client_version")?
            .query_map([], |r| r.get::<_, String>(0))?
        {
            if b?.parse::<u64>()? > build {
                return Err(invalid("older target client build").into());
            }
        }
    }
    let mut inputs = BTreeMap::new();
    for p in [mapping_path, english_path, localized_path, manifest] {
        inputs.insert(p.to_owned(), hash(&read(p)?));
    }
    if let Some(destination) = output.exists().then(|| output.canonicalize()).transpose()?
        && inputs
            .keys()
            .any(|p| p.canonicalize().is_ok_and(|p| p == destination))
    {
        return Err(invalid("catalogue cannot replace an input").into());
    }
    let mapping = Mapping::parse(&read(mapping_path)?)?;
    let mut batch = skills::probe(&mapping, directory)?.tables;
    let item_path = directory.join("Item.bin");
    let bytes = read(&item_path)?;
    batch.insert("Item".into(), mapping.probe_table("ItemTableItem", &bytes)?);
    let mut errors: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut candidates: BTreeSet<String> = [
        "BoxReward",
        "RewardItem",
        "RewardPackageItem",
        "EnumString",
        "UIIcon",
        "CurrencyInfo",
        "Enchant",
        "EnchantEffect",
        "ExceedEnchant",
        "AdditionalStat",
        "ArcanaEnchant",
        "ArcanaEnchantEffectMainStat",
        "ArcanaEnchantEffectSubstat",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    candidates.extend(crafting_quests::SOURCES.iter().map(|s| (*s).to_owned()));
    candidates.extend(achievements::SOURCES.iter().map(|s| (*s).to_owned()));
    candidates.extend(companions::SOURCES.iter().map(|s| (*s).to_owned()));
    candidates.extend(daevanion::SOURCES.iter().map(|s| (*s).to_owned()));
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) == Some("bin")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && (stem.starts_with("Item") || obtaining::is_source(stem))
        {
            candidates.insert(stem.into());
        }
    }
    for name in candidates {
        if batch.contains_key(&name) {
            continue;
        }
        let path = directory.join(format!("{name}.bin"));
        match read(&path) {
            Ok(bytes) => {
                inputs.insert(path, hash(&bytes));
                match mapping.probe_table(&obtaining::structure(&name), &bytes) {
                    Ok(table) => {
                        batch.insert(name, table);
                    }
                    Err(e) => {
                        errors.insert(name, ("failed".into(), e.to_string()));
                    }
                }
            }
            Err(e) => {
                errors.insert(
                    name,
                    (
                        if path.exists() { "failed" } else { "not_found" }.into(),
                        e.to_string(),
                    ),
                );
            }
        }
    }
    if batch.values().map(|t| t.payload_bytes).sum::<usize>() > 512 * 1024 * 1024
        || batch.values().map(|t| t.layout.rows.len()).sum::<usize>() > 1_000_000
    {
        return Err(invalid("catalogue batch exceeds its byte/row bounds").into());
    }
    for (name, table) in &batch {
        inputs.insert(
            directory.join(format!("{name}.bin")),
            table.payload_sha256.clone(),
        );
    }
    if let Some(destination) = output.exists().then(|| output.canonicalize()).transpose()?
        && inputs
            .keys()
            .any(|p| p.canonicalize().is_ok_and(|p| p == destination))
    {
        return Err(invalid("catalogue cannot replace a source table").into());
    }
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let mut random = [0; 16];
    getrandom::fill(&mut random).map_err(|e| std::io::Error::other(e.to_string()))?;
    let temporary = Temporary(parent.join(format!(".catalogue-{}", hex::encode(random))));
    fs::create_dir(&temporary.0)?;
    let item_probe = temporary.0.join("Item.json");
    fs::write(&item_probe, serde_json::to_vec(&batch["Item"])?)?;
    let staged = temporary.0.join("data.sqlite");
    items::export_localized(
        &item_probe,
        english_path,
        localized_path,
        locale,
        manifest,
        &staged,
    )?;
    let en: Localization = serde_json::from_slice(&read(english_path)?)?;
    let localized: Localization = serde_json::from_slice(&read(localized_path)?)?;
    let mut db = Connection::open(&staged)?;
    db.execute_batch("PRAGMA foreign_keys=OFF")?;
    let tx = db.transaction()?;
    tx.execute_batch(include_str!("catalogue.sql"))?;
    tx.execute_batch(include_str!("catalogue_projection.sql"))?;
    tx.execute_batch(include_str!("crafting_quests.sql"))?;
    tx.execute_batch(include_str!("achievements.sql"))?;
    tx.execute_batch(include_str!("companions.sql"))?;
    tx.execute_batch(include_str!("daevanion.sql"))?;
    let current = version(&tx, &steam.build_id)?;
    tx.execute(
        "INSERT INTO catalogue_manifest VALUES(1,?1,?2,?3,?4,?5,?6,NULL,'unknown')",
        params![
            current,
            locale,
            steam.sha256,
            hash(&read(mapping_path)?),
            env!("CARGO_PKG_VERSION"),
            reader_revision()
        ],
    )?;
    super::skill_database::write(&tx, &batch, &en, &localized, current)?;
    let mut summary = Summary {
        output: output.to_owned(),
        steam_build: steam.build_id,
        locale: locale.into(),
        ..Summary::default()
    };
    // Preserve the evidence of optional sources which failed this time. Their old
    // build remains explicit. Product rows from failed auxiliary families are not
    // silently treated as an empty current source.
    let mut retained = BTreeSet::new();
    for (name, (state, error)) in errors {
        if let Some(old) = &previous
            && old
                .prepare("SELECT 1 FROM sqlite_schema WHERE name='catalogue_sources'")?
                .exists([])?
        {
            let record=old.query_row("SELECT v.steam_build,s.mapping_sha256,s.payload_sha256,s.row_struct,s.compatibility_verified,s.row_count,s.decoded_lz4 FROM catalogue_sources s JOIN client_version v ON v.id=s.build_id WHERE s.name=?1",[&name],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,bool>(4)?,r.get::<_,i64>(5)?,r.get::<_,Vec<u8>>(6)?))).optional()?;
            if let Some((build, mapping, payload, structure, compatible, count, data)) = record {
                let old_version = version(&tx, &build)?;
                let rows = decode_rows(&data)?;
                batch.insert(
                    name.clone(),
                    TableProbe {
                        mapping_sha256: mapping.clone(),
                        mapping_version: 0,
                        mapping_ambiguous_structures: BTreeSet::new(),
                        row_struct: structure.clone(),
                        payload_sha256: payload.clone(),
                        payload_bytes: 0,
                        layout: super::schema::LayoutProbe {
                            compatibility_verified: compatible,
                            table_version: 13,
                            column_hashes: vec![],
                            rows,
                        },
                    },
                );
                tx.execute(
                    "INSERT INTO catalogue_sources VALUES(?1,?2,?3,?4,?5,?6,'retained',?7,?8)",
                    params![
                        name,
                        old_version,
                        mapping,
                        payload,
                        structure,
                        compatible,
                        count,
                        data
                    ],
                )?;
                retained.insert(name.clone());
                coverage(
                    &tx,
                    &name,
                    "partial",
                    &format!("retained build {build}; current source: {error}"),
                )?;
                continue;
            }
        }
        coverage(&tx, &name, &state, &error)?;
    }
    for (name, table) in &batch {
        if retained.contains(name) {
            continue;
        }
        let compressed = lz4_flex::compress_prepend_size(&serde_json::to_vec(&table.layout.rows)?);
        tx.execute(
            "INSERT INTO catalogue_sources VALUES(?1,?2,?3,?4,?5,?6,'decoded',?7,?8)",
            params![
                name,
                current,
                table.mapping_sha256,
                table.payload_sha256,
                table.row_struct,
                table.layout.compatibility_verified,
                table.layout.rows.len() as i64,
                compressed
            ],
        )?;
        coverage(
            &tx,
            name,
            "uninterpreted",
            "structurally decoded; complete source retained; semantic interpretation is field-specific",
        )?;
    }
    if !retained.is_empty()
        && let Some(old) = &previous
    {
        for (name, table) in &batch {
            if retained.contains(name) {
                continue;
            }
            let prior: Option<String> = old
                .query_row(
                    "SELECT payload_sha256 FROM catalogue_sources WHERE name=?1",
                    [name],
                    |r| r.get(0),
                )
                .optional()?;
            if prior
                .as_deref()
                .is_some_and(|sha| sha != table.payload_sha256)
            {
                return Err(invalid(format!("cannot combine changed {name} with retained sources; previous catalogue preserved")).into());
            }
        }
    }
    loot_changes::prepare(&tx, previous.as_ref(), &batch, current)?;
    projection::write(&tx, &batch, &en, &localized, image_directory)?;
    crafting_quests::write(&tx, &batch, &en, &localized, current)?;
    achievements::write(&tx, &batch, &en, &localized, current, image_directory)?;
    companions::write(&tx, &batch, &en, &localized, current, image_directory)?;
    daevanion::write(&tx, &batch, &en, &localized, current, image_directory)?;
    if let Some(old) = &previous {
        for row in old
            .prepare("SELECT code,name_en,name_localized FROM stats")?
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            })?
        {
            let (code, en, localized) = row?;
            let same_locale = old
                .query_row(
                    "SELECT locale FROM localization_language WHERE id=1",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .optional()?
                .as_deref()
                == Some(locale);
            tx.execute("UPDATE stats SET name_en=COALESCE(?1,name_en),name_localized=COALESCE(?2,name_localized) WHERE code=?3",params![en,if same_locale{localized}else{None},code])?;
        }
    }
    changes::record(
        &tx,
        previous.as_ref(),
        &batch,
        &retained,
        current,
        locale,
        &mut summary,
        &en,
        &localized,
    )?;
    tx.pragma_update(None, "user_version", 15)?;
    tx.commit()?;
    db.execute_batch("PRAGMA foreign_keys=ON")?;
    db.execute_batch("VACUUM")?;
    integrity(&db)?;
    summary.items =
        db.query_row("SELECT count(*) FROM items", [], |r| r.get::<_, i64>(0))? as usize;
    summary.skills =
        db.query_row("SELECT count(*) FROM skills", [], |r| r.get::<_, i64>(0))? as usize;
    summary.abnormals = db.query_row("SELECT count(*) FROM skill_abnormals", [], |r| {
        r.get::<_, i64>(0)
    })? as usize;
    summary.sources = batch.len();
    summary.recipes =
        db.query_row("SELECT count(*) FROM recipes", [], |r| r.get::<_, i64>(0))? as usize;
    summary.quests =
        db.query_row("SELECT count(*) FROM quests", [], |r| r.get::<_, i64>(0))? as usize;
    summary.achievements = db.query_row("SELECT count(*) FROM achievements", [], |r| {
        r.get::<_, i64>(0)
    })? as usize;
    summary.titles =
        db.query_row("SELECT count(*) FROM titles", [], |r| r.get::<_, i64>(0))? as usize;
    summary.passes = db.query_row("SELECT count(*) FROM battle_passes", [], |r| {
        r.get::<_, i64>(0)
    })? as usize;
    summary.supply_requests = db.query_row("SELECT count(*) FROM supply_requests", [], |r| {
        r.get::<_, i64>(0)
    })? as usize;
    summary.seasons =
        db.query_row("SELECT count(*) FROM seasons", [], |r| r.get::<_, i64>(0))? as usize;
    summary.journals =
        db.query_row("SELECT count(*) FROM journals", [], |r| r.get::<_, i64>(0))? as usize;
    summary.pets = db.query_row("SELECT count(*) FROM pets", [], |r| r.get::<_, i64>(0))? as usize;
    summary.wings =
        db.query_row("SELECT count(*) FROM wings", [], |r| r.get::<_, i64>(0))? as usize;
    summary.skins =
        db.query_row("SELECT count(*) FROM skins", [], |r| r.get::<_, i64>(0))? as usize;
    summary.skin_sets =
        db.query_row("SELECT count(*) FROM skin_sets", [], |r| r.get::<_, i64>(0))? as usize;
    summary.coverage = db
        .prepare("SELECT subject,state,diagnostic FROM catalogue_coverage ORDER BY subject")?
        .query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))?
        .collect::<rusqlite::Result<_>>()?;
    summary.daevanion_boards = db.query_row("SELECT count(*) FROM daevanion_boards", [], |r| {
        r.get::<_, i64>(0)
    })? as usize;
    summary.daevanion_nodes = db.query_row("SELECT count(*) FROM daevanion_nodes", [], |r| {
        r.get::<_, i64>(0)
    })? as usize;
    drop(db);
    drop(previous);
    for (path, expected) in inputs {
        if hash(&read(&path)?) != expected {
            return Err(invalid(format!("catalogue source changed: {}", path.display())).into());
        }
    }
    if original.is_none() && output.exists() {
        return Err(invalid("catalogue appeared during export").into());
    }
    crate::engine_ini::storage::atomic_replace(
        output,
        &read_database(&staged)?,
        original.as_deref(),
    )?;
    Ok(summary)
}
fn integrity(db: &Connection) -> Result<()> {
    if db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))? != "ok"
        || db.prepare("PRAGMA foreign_key_check")?.exists([])?
    {
        return Err(invalid("invalid target catalogue integrity").into());
    }
    Ok(())
}
fn decode_rows(data: &[u8]) -> Result<Vec<Value>> {
    if data.len() < 4 || u32::from_le_bytes(data[..4].try_into()?) > 512 * 1024 * 1024 {
        return Err(invalid("oversized compressed source").into());
    }
    Ok(serde_json::from_slice(
        &lz4_flex::decompress_size_prepended(data)?,
    )?)
}
/// Read complete decoded source rows, including fields without an interpretation.
pub fn source_rows(db: &Connection, name: &str) -> Result<Vec<Value>> {
    let data: Vec<u8> = db.query_row(
        "SELECT decoded_lz4 FROM catalogue_sources WHERE name=?1",
        [name],
        |r| r.get(0),
    )?;
    decode_rows(&data)
}
/// Missing translation fallback is computed; originals remain unchanged.
pub fn localized_text(localized: Option<&str>, english: Option<&str>) -> (Option<String>, bool) {
    match localized {
        Some(s) => (Some(s.into()), false),
        None => (english.map(str::to_owned), english.is_some()),
    }
}
/// Three states, without expanding true and false into duplicate memberships.
pub fn item_permission(db: &Connection, item_id: i64, code: &str) -> Result<Option<bool>> {
    if db.prepare("SELECT 1 FROM item_flags p JOIN flags f ON f.id=p.flag_id WHERE p.item_id=?1 AND f.code=?2")?.exists(params![item_id,code])?{return Ok(Some(true));}
    if db.prepare("SELECT 1 FROM item_false_flags p JOIN flags f ON f.id=p.flag_id WHERE p.item_id=?1 AND f.code=?2")?.exists(params![item_id,code])?{return Ok(Some(false));}
    Ok(None)
}
