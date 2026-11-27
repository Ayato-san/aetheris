//! Minimal latest observed change, not a history of old content.
use super::*;

#[derive(Clone)]
struct Entry {
    hash: String,
    localized: String,
    locale: String,
    profile: String,
    present: bool,
    observed: String,
    data_build: String,
    change: Option<String>,
    change_build: Option<String>,
    change_reference: Option<String>,
}
struct Baseline {
    profile: String,
    observed: String,
    state: String,
}

fn baselines(db: Option<&Connection>) -> Result<BTreeMap<String, Baseline>> {
    let Some(db) = db else {
        return Ok(BTreeMap::new());
    };
    Ok(db.prepare("SELECT c.family,c.comparison_profile,v.steam_build,c.comparison_state FROM catalogue_comparisons c JOIN client_version v ON v.id=c.observed_build_id")?
        .query_map([], |r| Ok((r.get(0)?, Baseline { profile:r.get(1)?, observed:r.get(2)?, state:r.get(3)? })))?
        .collect::<rusqlite::Result<_>>()?)
}
fn previous(db: Option<&Connection>) -> Result<BTreeMap<(String, i64), Entry>> {
    let Some(db) = db else {
        return Ok(BTreeMap::new());
    };
    Ok(db.prepare("SELECT e.family,e.source_id,e.content_hash,e.localized_hash,e.locale,e.comparison_profile,e.present,o.steam_build,e.change_type,c.steam_build,r.steam_build,d.steam_build FROM catalogue_entries e JOIN client_version o ON o.id=e.observed_build_id LEFT JOIN client_version c ON c.id=e.change_build_id LEFT JOIN client_version r ON r.id=e.change_reference_build_id JOIN client_version d ON d.id=e.data_build_id")?.query_map([],|r|Ok(((r.get(0)?,r.get(1)?),Entry{hash:r.get(2)?,localized:r.get(3)?,locale:r.get(4)?,profile:r.get(5)?,present:r.get(6)?,observed:r.get(7)?,change:r.get(8)?,change_build:r.get(9)?,change_reference:r.get(10)?,data_build:r.get(11)?})))?.collect::<rusqlite::Result<_>>()?)
}
#[allow(clippy::too_many_arguments)]
pub(super) fn record(
    db: &Connection,
    old_db: Option<&Connection>,
    batch: &BTreeMap<String, TableProbe>,
    retained: &BTreeSet<String>,
    build: i64,
    locale: &str,
    summary: &mut Summary,
    english: &Localization,
    localized: &Localization,
) -> Result<()> {
    let old = previous(old_db)?;
    let baselines = baselines(old_db)?;
    let mut current = BTreeMap::<(String, i64), (String, String)>::new();
    current.extend(crafting_quests::fingerprints(
        db, batch, english, localized,
    )?);
    current.extend(achievements::fingerprints(db, batch, english, localized)?);
    current.extend(companions::fingerprints(db, batch, english, localized)?);
    current.extend(daevanion::fingerprints(db, batch, english, localized)?);
    current.extend(world::fingerprints(db, batch, english, localized)?);
    for (id, h) in items::fingerprints(db, false)? {
        current.insert(("items".into(), id), (h, String::new()));
    }
    for (id, h) in items::fingerprints(db, true)? {
        current.get_mut(&("items".into(), id)).unwrap().1 = h;
    }
    for (id, (h, _)) in super::super::skill_database::fingerprints(db, false)? {
        current.insert(("skills".into(), id), (h, String::new()));
    }
    for (id, (h, _)) in super::super::skill_database::fingerprints(db, true)? {
        current.get_mut(&("skills".into(), id)).unwrap().1 = h;
    }
    for (id, hash) in loot_changes::fingerprints(db, batch)? {
        current.insert(("loot_tables".into(), id), (hash, String::new()));
    }
    item_contexts(db, &mut current)?;
    // Additional source fields and relationships participate even when their
    // display formatting or meaning has not yet been implemented.
    let mut related: BTreeMap<i64, BTreeSet<(String, usize)>> = BTreeMap::new();
    for table in [
        "item_obtaining_routes",
        "item_obtaining_sources",
        "item_progression_links",
        "item_relations",
    ] {
        for result in db
            .prepare(&format!(
                "SELECT item_id,source_name,source_row FROM {table}"
            ))?
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?
        {
            let (id, name, pos) = result?;
            if !loot_changes::is_source(&name) {
                related.entry(id).or_default().insert((name, pos as usize));
            }
        }
    }
    let set_rows: BTreeMap<String, Vec<(String, usize)>> = {
        let mut m: BTreeMap<String, Vec<(String, usize)>> = BTreeMap::new();
        for name in ["ItemSet", "ItemSetEffect"] {
            if let Some(table) = batch.get(name) {
                for (i, row) in table.layout.rows.iter().enumerate() {
                    if let Some(key) = string(&row["SetName"]) {
                        m.entry(key.into()).or_default().push((name.into(), i));
                    }
                }
            }
        }
        m
    };
    let mut row_cache = BTreeMap::new();
    let abnormal_evidence = abnormal_sources(batch);
    let skill_evidence = skill_sources(db, batch, &abnormal_evidence)?;
    for (source, family) in [
        ("Item", "items"),
        ("Skill", "skills"),
        ("SkillAbnormal", "abnormals"),
    ] {
        for (position, row) in batch[source].layout.rows.iter().enumerate() {
            let Some(id) = number(row, "ID") else {
                continue;
            };
            let key = (family.to_owned(), id);
            let (base, localized) = current.entry(key).or_default();
            let mut extra = vec![source_hash(batch, source, position, &mut row_cache)?];
            if family == "items" {
                if let Some(refs) = related.get(&id) {
                    for (table, pos) in refs {
                        extra.push(source_hash(batch, table, *pos, &mut row_cache)?);
                    }
                }
                if let Some(sets) = row["$variant"]["SetNames"].as_array() {
                    for set in sets {
                        if let Some(refs) = set.as_str().and_then(|key| set_rows.get(key)) {
                            for (table, pos) in refs {
                                extra.push(source_hash(batch, table, *pos, &mut row_cache)?);
                            }
                        }
                    }
                }
            }
            if family == "skills"
                && let Some(references) = skill_evidence.get(&id)
            {
                for (table, pos) in references {
                    extra.push(source_hash(batch, table, *pos, &mut row_cache)?);
                }
            }
            if family == "abnormals" {
                if let Some(references) = abnormal_evidence.get(&id) {
                    for (table, pos) in references {
                        extra.push(source_hash(batch, table, *pos, &mut row_cache)?);
                    }
                }
                let texts: Option<[Option<String>;4]>=db.query_row("SELECT name_en,description_en,name_localized,description_localized FROM skill_abnormals WHERE id=?1",[id],|r|Ok([r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?])).optional()?;
                if let Some([name, desc, loc_name, loc_desc]) = texts {
                    *base = hash(&serde_json::to_vec(&(name, desc))?);
                    *localized = hash(&serde_json::to_vec(&(loc_name, loc_desc))?);
                }
            }
            extra.sort();
            *base = hash(&serde_json::to_vec(&(&base, extra))?);
        }
    }
    let profile = hash(&serde_json::to_vec(&(
        reader_revision(),
        batch
            .iter()
            .map(|(name, table)| (name, &table.row_struct, &table.mapping_sha256))
            .collect::<Vec<_>>(),
    ))?);
    let comparable = retained.is_empty();
    let mut impossible_families = 0;
    let mut families = vec!["items", "skills", "abnormals", "loot_tables"];
    for source in ["CraftRecipe", "ConversionRecipe", "Quest"] {
        let family = crafting_quests::family(source).unwrap();
        if batch.contains_key(source) || baselines.contains_key(family) {
            families.push(family);
        }
    }
    for (source, family, _) in achievements::ROOTS {
        if batch.contains_key(*source) || baselines.contains_key(*family) {
            families.push(*family);
        }
    }
    for (source, family, _) in companions::ROOTS {
        if batch.contains_key(*source) || baselines.contains_key(*family) {
            families.push(*family);
        }
    }
    for (source, family, _) in daevanion::ROOTS {
        if batch.contains_key(*source) || baselines.contains_key(*family) {
            families.push(*family);
        }
    }
    for (source, family, _) in world::ROOTS {
        if batch.contains_key(*source)
            || (*source == "@map-resources"
                && batch
                    .keys()
                    .any(|n| super::super::map_resources::is_source(n)))
            || baselines.contains_key(*family)
        {
            families.push(family);
        }
    }
    for family in families {
        let prior = baselines.get(family);
        let reference = prior.map(|p| version(db, &p.observed)).transpose()?;
        let state = if old_db.is_none() && comparable {
            "initial"
        } else if comparable && prior.is_some_and(|p| p.profile == profile) {
            "comparable"
        } else {
            "impossible"
        };
        let incomplete = state == "impossible" || prior.is_some_and(|p| p.state == "impossible");
        impossible_families += usize::from(state == "impossible");
        db.execute(
            "INSERT INTO catalogue_comparisons VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                family,
                if comparable {
                    profile.as_str()
                } else {
                    prior.map(|p| p.profile.as_str()).unwrap_or("")
                },
                if comparable {
                    build
                } else {
                    reference.unwrap_or(build)
                },
                reference,
                state,
                incomplete
            ],
        )?;
    }
    for ((family, id), (content, localized)) in &current {
        let prior = old.get(&(family.clone(), *id));
        let family_prior = baselines.get(family);
        let reference = prior
            .map(|e| version(db, &e.observed))
            .or_else(|| family_prior.map(|e| version(db, &e.observed)))
            .transpose()?;
        let can_compare = comparable
            && family_prior.is_some_and(|e| e.profile == profile)
            && prior.is_none_or(|e| e.profile == profile);
        let (state, change) = if !can_compare {
            if old_db.is_none() && comparable {
                summary.initial += 1;
                ("initial", None)
            } else {
                summary.comparison_impossible += 1;
                ("impossible", None)
            }
        } else {
            match prior {
                None => {
                    summary.added += 1;
                    ("comparable", Some("added"))
                }
                Some(e) if !e.present => {
                    summary.added += 1;
                    ("comparable", Some("added"))
                }
                Some(e)
                    if e.hash != *content || (e.locale == locale && e.localized != *localized) =>
                {
                    summary.modified += 1;
                    ("comparable", Some("modified"))
                }
                Some(_) => {
                    summary.unchanged += 1;
                    ("comparable", None)
                }
            }
        };
        let latest = change
            .map(str::to_owned)
            .or_else(|| prior.and_then(|e| e.change.clone()));
        let change_build = if change.is_some() {
            Some(build)
        } else {
            prior
                .and_then(|e| e.change_build.as_deref())
                .map(|b| version(db, b))
                .transpose()?
        };
        let change_reference = if change.is_some() {
            reference
        } else {
            prior
                .and_then(|e| e.change_reference.as_deref())
                .map(|b| version(db, b))
                .transpose()?
        };
        // Retain the existing convenient link to the last build with an observed
        // data change. Initial/non-comparable baselines use their observed build.
        let modified_build = if change.is_none() {
            prior
                .map(|e| version(db, &e.data_build))
                .transpose()?
                .unwrap_or(build)
        } else {
            build
        };
        db.execute(
            "INSERT INTO catalogue_entries VALUES(?1,?2,?3,?4,?5,?6,?14,?7,?8,?9,?10,?11,?12,?13)",
            params![
                family,
                id,
                if !comparable {
                    prior.map(|e| e.hash.as_str()).unwrap_or(content)
                } else {
                    content
                },
                if !comparable {
                    prior.map(|e| e.localized.as_str()).unwrap_or(localized)
                } else {
                    localized
                },
                if !comparable {
                    prior.map(|e| e.locale.as_str()).unwrap_or(locale)
                } else {
                    locale
                },
                if comparable {
                    profile.as_str()
                } else {
                    prior.map(|e| e.profile.as_str()).unwrap_or("")
                },
                if comparable {
                    build
                } else {
                    reference.unwrap_or(build)
                },
                reference,
                latest,
                change_build,
                change_reference,
                state,
                modified_build,
                comparable || prior.is_none_or(|e| e.present)
            ],
        )?;
        if matches!(family.as_str(), "items" | "skills" | "loot_tables") {
            db.execute(
                &format!("UPDATE {family} SET client_version_id=?1 WHERE id=?2"),
                params![modified_build, id],
            )?;
        }
        if let Some((_, _, table)) = achievements::ROOTS
            .iter()
            .chain(companions::ROOTS)
            .chain(daevanion::ROOTS)
            .chain(world::ROOTS)
            .find(|(_, f, _)| *f == family)
        {
            db.execute(
                &format!("UPDATE {table} SET client_version_id=?1 WHERE id=?2"),
                params![modified_build, id],
            )?;
        }
        if family == "quests" {
            db.execute(
                "UPDATE quests SET client_version_id=?1 WHERE id=?2",
                params![modified_build, id],
            )?;
        }
        if matches!(family.as_str(), "craft_recipes" | "conversion_recipes") {
            db.execute(
                "UPDATE recipes SET client_version_id=?1 WHERE source_name=?2 AND id=?3",
                params![
                    modified_build,
                    if family == "craft_recipes" {
                        "CraftRecipe"
                    } else {
                        "ConversionRecipe"
                    },
                    id
                ],
            )?;
        }
    }
    for ((family, id), entry) in &old {
        if current.contains_key(&(family.clone(), *id)) {
            continue;
        }
        let can_compare = comparable && entry.profile == profile;
        let reference = version(db, &entry.observed)?;
        let deleted = can_compare && entry.present;
        if deleted {
            summary.deleted += 1;
        } else if !can_compare {
            summary.comparison_impossible += 1;
        }
        let change = if deleted {
            Some("deleted")
        } else {
            entry.change.as_deref()
        };
        let change_build = if deleted {
            Some(build)
        } else {
            entry
                .change_build
                .as_deref()
                .map(|s| version(db, s))
                .transpose()?
        };
        let change_reference = if deleted {
            Some(reference)
        } else {
            entry
                .change_reference
                .as_deref()
                .map(|s| version(db, s))
                .transpose()?
        };
        db.execute(
            "INSERT INTO catalogue_entries VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                family,
                id,
                entry.hash,
                entry.localized,
                entry.locale,
                entry.profile,
                if can_compare { false } else { entry.present },
                if can_compare { build } else { reference },
                reference,
                change,
                change_build,
                change_reference,
                if can_compare {
                    "comparable"
                } else {
                    "impossible"
                },
                version(db, &entry.data_build)?
            ],
        )?;
    }
    let impossible = summary.comparison_impossible;
    coverage(
        db,
        "build_comparison",
        if impossible_families == 0 && impossible == 0 {
            "verified"
        } else {
            "partial"
        },
        &format!(
            "{impossible} entries and {impossible_families} families not comparable; {} retained sources; latest observed change only; first import is a baseline; source/reader coverage changes are not attributed to a game patch; family reference/observed builds bound an interval, not an exact patch; interval_incomplete records known interrupted comparisons",
            retained.len()
        ),
    )?;
    // No older contents are kept for tombstones; hashes and identity suffice.
    Ok(())
}

fn source_hash(
    batch: &BTreeMap<String, TableProbe>,
    table: &str,
    position: usize,
    cache: &mut BTreeMap<(String, usize), String>,
) -> Result<String> {
    let key = (table.to_owned(), position);
    if let Some(hash) = cache.get(&key) {
        return Ok(hash.clone());
    }
    let result = hash(&serde_json::to_vec(&(
        table,
        &batch[table].layout.rows[position],
    ))?);
    cache.insert(key, result.clone());
    Ok(result)
}

type Evidence = BTreeSet<(String, usize)>;

// Context and translated labels can change without changing a reward's own
// source row. Compare natural values, excluding local IDs and file positions.
fn item_contexts(
    db: &Connection,
    current: &mut BTreeMap<(String, i64), (String, String)>,
) -> Result<()> {
    for query in [
        "SELECT item_id,direction,kind,source_name,source_path,name_en,CASE WHEN ?1 THEN name_localized END,related_item_id,quantity_min,quantity_max,weight_raw,probability_state,probability_percent FROM item_relations WHERE source_name NOT IN (SELECT source_name FROM loot_tables)",
        "SELECT i.item_id,s.code,d.name_en,CASE WHEN ?1 THEN d.name_localized END,d.type_source,d.icon_reference FROM item_sets i JOIN sets s ON s.id=i.set_id JOIN item_set_definitions d ON d.set_id=i.set_id",
    ] {
        for localized in [false, true] {
            let mut rows: BTreeMap<i64, Vec<Vec<u8>>> = BTreeMap::new();
            let mut statement = db.prepare(query)?;
            let count = statement.column_count();
            let mut cursor = statement.query([localized])?;
            while let Some(row) = cursor.next()? {
                let mut values = Vec::new();
                for index in 1..count {
                    use rusqlite::types::ValueRef;
                    values.push(match row.get_ref(index)? {
                        ValueRef::Null => Value::Null,
                        ValueRef::Integer(v) => Value::from(v),
                        ValueRef::Real(v) => Value::from(v),
                        ValueRef::Text(v) => Value::from(std::str::from_utf8(v)?),
                        ValueRef::Blob(_) => return Err(invalid("unexpected context BLOB").into()),
                    });
                }
                rows.entry(row.get(0)?)
                    .or_default()
                    .push(serde_json::to_vec(&values)?);
            }
            for (id, mut values) in rows {
                values.sort();
                if let Some((base, translation)) = current.get_mut(&("items".into(), id)) {
                    let target = if localized { translation } else { base };
                    *target = hash(&serde_json::to_vec(&(&target, query, values))?);
                }
            }
        }
    }
    Ok(())
}

fn abnormal_sources(batch: &BTreeMap<String, TableProbe>) -> BTreeMap<i64, Evidence> {
    let mut result: BTreeMap<i64, Evidence> = BTreeMap::new();
    let strings: BTreeMap<_, _> = batch["SkillAbnormalString"]
        .layout
        .rows
        .iter()
        .enumerate()
        .filter_map(|(pos, r)| Some((string(&r["Name"])?, pos)))
        .collect();
    let mut levels: BTreeMap<&str, Evidence> = BTreeMap::new();
    for (pos, row) in batch["SkillAbnormalEffectLv"]
        .layout
        .rows
        .iter()
        .enumerate()
    {
        if let Some(group) = string(&row["AbnormalEffectLevelGroupId"]) {
            levels
                .entry(group)
                .or_default()
                .insert(("SkillAbnormalEffectLv".into(), pos));
        }
    }
    for (pos, row) in batch["SkillAbnormal"].layout.rows.iter().enumerate() {
        if let Some(id) = number(row, "ID") {
            let refs = result.entry(id).or_default();
            refs.insert(("SkillAbnormal".into(), pos));
            if let Some(pos) = string(&row["SkillAbnormalString_Key"]).and_then(|k| strings.get(k))
            {
                refs.insert(("SkillAbnormalString".into(), *pos));
            }
        }
    }
    for (pos, row) in batch["SkillAbnormalEffect"].layout.rows.iter().enumerate() {
        if let Some(id) = number(row, "AbnormalId") {
            let refs = result.entry(id).or_default();
            refs.insert(("SkillAbnormalEffect".into(), pos));
            if let Some(levels) =
                string(&row["AbnormalEffectLvGroupId"]).and_then(|g| levels.get(g))
            {
                refs.extend(levels.clone());
            }
        }
    }
    result
}
fn skill_sources(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    abnormal_evidence: &BTreeMap<i64, Evidence>,
) -> Result<BTreeMap<i64, Evidence>> {
    let mut ids = BTreeMap::new();
    let mut groups: BTreeMap<(String, String), Evidence> = BTreeMap::new();
    let mut acquired: BTreeMap<i64, Evidence> = BTreeMap::new();
    for (name, table) in batch {
        if !matches!(
            name.as_str(),
            "SkillEffect"
                | "SkillEffectLv"
                | "SkillLv"
                | "SkillAcquireData"
                | "SkillCharge"
                | "SkillEffectFilter"
        ) {
            continue;
        }
        for (pos, row) in table.layout.rows.iter().enumerate() {
            if let Some(id) = number(row, "ID") {
                ids.insert((name.clone(), id), pos);
            }
            let field = match name.as_str() {
                "SkillEffectLv" => Some("SkillEffectLvGroupId"),
                "SkillLv" => Some("SkillLvGroupId"),
                _ => None,
            };
            if let Some(group) = field.and_then(|f| string(&row[f])) {
                groups
                    .entry((name.clone(), group.into()))
                    .or_default()
                    .insert((name.clone(), pos));
            }
            if name == "SkillAcquireData"
                && let Some(id) = number(row, "SkillId")
            {
                acquired.entry(id).or_default().insert((name.clone(), pos));
            }
        }
    }
    let mut applications: BTreeMap<(String, i64), Vec<i64>> = BTreeMap::new();
    for row in db
        .prepare("SELECT effect_kind,effect_id,abnormal_id FROM skill_abnormal_links")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
    {
        let (kind, id, abnormal) = row?;
        applications.entry((kind, id)).or_default().push(abnormal);
    }
    let mut result: BTreeMap<i64, Evidence> = BTreeMap::new();
    for row in db
        .prepare("SELECT skill_id,effect_id,filter_id FROM skill_effect_links")?
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, Option<i64>>(2)?,
            ))
        })?
    {
        let (skill, effect, filter) = row?;
        let refs = result.entry(skill).or_default();
        if let Some(pos) = ids.get(&("SkillEffect".into(), effect)) {
            refs.insert(("SkillEffect".into(), *pos));
            let base = &batch["SkillEffect"].layout.rows[*pos];
            let mut selected = vec![("se".to_owned(), effect)];
            if let Some(levels) = string(&base["SkillEffectLvGroupId"])
                .and_then(|g| groups.get(&("SkillEffectLv".into(), g.into())))
            {
                refs.extend(levels.clone());
                for (_, pos) in levels {
                    if let Some(id) = number(&batch["SkillEffectLv"].layout.rows[*pos], "ID") {
                        selected.push(("se_level".into(), id));
                    }
                }
            }
            for key in selected {
                for abnormal in applications.get(&key).into_iter().flatten() {
                    if let Some(evidence) = abnormal_evidence.get(abnormal) {
                        refs.extend(evidence.clone());
                    }
                }
            }
        }
        if let Some(pos) = filter.and_then(|id| ids.get(&("SkillEffectFilter".into(), id))) {
            refs.insert(("SkillEffectFilter".into(), *pos));
        }
    }
    for row in &batch["Skill"].layout.rows {
        let Some(id) = number(row, "ID") else {
            continue;
        };
        let refs = result.entry(id).or_default();
        if let Some(levels) =
            string(&row["SkillLvGroupId"]).and_then(|g| groups.get(&("SkillLv".into(), g.into())))
        {
            refs.extend(levels.clone());
        }
        if let Some(rows) = acquired.get(&id) {
            refs.extend(rows.clone());
        }
        if let Some(pos) =
            number(row, "ChargeId").and_then(|id| ids.get(&("SkillCharge".into(), id)))
        {
            refs.insert(("SkillCharge".into(), *pos));
        }
    }
    Ok(result)
}
