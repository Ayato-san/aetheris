//! Loot definitions own their changes; reward changes do not revise items.
use super::*;

const SOURCES: &[(&str, &str)] = &[
    ("NpcLoot", "NpcId"),
    ("RewardData", "ID"),
    ("RewardItem", "Name"),
    ("RewardPackage", "Name"),
    ("RewardPackageItem", "PackageName"),
    ("BoxReward", "Group"),
    ("RewardOdEnergyCube", "Name"),
    ("DungeonRewardInfo", "ID"),
    ("RewardDecomposePackage", "Name"),
    ("RewardLevelScaleData", "ID"),
    ("Dungeon", "ID"),
];

pub(super) fn is_source(name: &str) -> bool {
    SOURCES.iter().any(|(source, _)| *source == name)
}

type Definitions<'a> = BTreeMap<(String, String), Vec<&'a Value>>;
fn definitions(batch: &BTreeMap<String, TableProbe>) -> Result<Definitions<'_>> {
    let mut result = Definitions::new();
    for (name, field) in SOURCES {
        let Some(table) = batch.get(*name) else {
            continue;
        };
        for row in &table.layout.rows {
            if *name == "Dungeon"
                && !matches!(
                    string(&row["DungeonType"]),
                    Some("EDungeonType::Party" | "EDungeonType::Raid")
                )
            {
                continue;
            }
            let key = string(&row[*field])
                .map(str::to_owned)
                .or_else(|| {
                    number(row, field)
                        .filter(|id| *id > 0)
                        .map(|id| id.to_string())
                })
                .ok_or_else(|| invalid(format!("missing loot identity {name}.{field}")))?;
            result.entry(((*name).into(), key)).or_default().push(row);
        }
    }
    Ok(result)
}

pub(super) fn prepare(
    db: &Connection,
    previous: Option<&Connection>,
    batch: &BTreeMap<String, TableProbe>,
    build: i64,
) -> Result<()> {
    if let Some(old) = previous {
        let mut versions = BTreeMap::new();
        let mut insert = db.prepare("INSERT INTO loot_tables VALUES(?1,?2,?3,?4)")?;
        for row in old.prepare("SELECT t.id,t.source_name,t.source_key,v.steam_build FROM loot_tables t JOIN client_version v ON v.id=t.client_version_id")?
            .query_map([], |r| Ok((r.get::<_, i64>(0)?,r.get::<_, String>(1)?,r.get::<_, String>(2)?,r.get::<_, String>(3)?)))? {
            let (id, source, key, old_build) = row?;
            let old_version = match versions.get(&old_build) {
                Some(id) => *id,
                None => {
                    let id = version(db, &old_build)?;
                    versions.insert(old_build, id);
                    id
                }
            };
            insert.execute(params![id,source,key,old_version])?;
        }
    }
    let definitions = definitions(batch)?;
    let mut insert = db.prepare("INSERT OR IGNORE INTO loot_tables(source_name,source_key,client_version_id) VALUES(?1,?2,?3)")?;
    for (name, key) in definitions.keys() {
        insert.execute(params![name, key, build])?;
    }
    coverage(
        db,
        "loot.changes",
        "verified",
        &format!(
            "{} source loot definitions identified by table and natural key; race/level variants grouped; weights, quantities and allocation rules tracked independently from items; no old values retained",
            definitions.len()
        ),
    )?;
    Ok(())
}

pub(super) fn fingerprints(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
) -> Result<BTreeMap<i64, String>> {
    let ids: BTreeMap<(String, String), i64> = db
        .prepare("SELECT source_name,source_key,id FROM loot_tables")?
        .query_map([], |r| Ok(((r.get(0)?, r.get(1)?), r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut result = BTreeMap::new();
    for ((name, key), rows) in definitions(batch)? {
        let mut hashes = Vec::new();
        for row in rows {
            let value = if name == "Dungeon" {
                // The parent dungeon also has unrelated combat/map settings.
                // Only reward bindings and their dungeon context belong here.
                Value::Object(
                    row.as_object()
                        .ok_or_else(|| invalid("invalid dungeon row"))?
                        .iter()
                        .filter(|(field, _)| {
                            field.starts_with("Reward")
                                || matches!(
                                    field.as_str(),
                                    "ID" | "DungeonType" | "DungeonDifficulty"
                                )
                        })
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                )
            } else {
                row.clone()
            };
            hashes.push(hash(&serde_json::to_vec(&value)?));
        }
        hashes.sort();
        let id = ids
            .get(&(name.clone(), key.clone()))
            .ok_or_else(|| invalid("unregistered loot definition"))?;
        result.insert(*id, hash(&serde_json::to_vec(&(name, key, hashes))?));
    }
    Ok(result)
}
