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
fn previous(db: Option<&Connection>) -> Result<BTreeMap<(String, i64), Entry>> {
    let Some(db) = db else {
        return Ok(BTreeMap::new());
    };
    Ok(db.prepare("SELECT e.family,e.source_id,e.content_hash,e.localized_hash,e.locale,e.comparison_profile,e.present,o.steam_build,e.change_type,c.steam_build,r.steam_build,d.steam_build FROM catalogue_entries e JOIN client_version o ON o.id=e.observed_build_id LEFT JOIN client_version c ON c.id=e.change_build_id LEFT JOIN client_version r ON r.id=e.change_reference_build_id JOIN client_version d ON d.id=e.data_build_id")?.query_map([],|r|Ok(((r.get(0)?,r.get(1)?),Entry{hash:r.get(2)?,localized:r.get(3)?,locale:r.get(4)?,profile:r.get(5)?,present:r.get(6)?,observed:r.get(7)?,change:r.get(8)?,change_build:r.get(9)?,change_reference:r.get(10)?,data_build:r.get(11)?})))?.collect::<rusqlite::Result<_>>()?)
}
pub(super) fn record(
    db: &Connection,
    old_db: Option<&Connection>,
    batch: &BTreeMap<String, TableProbe>,
    retained: &BTreeSet<String>,
    build: i64,
    locale: &str,
    summary: &mut Summary,
) -> Result<()> {
    let old = previous(old_db)?;
    let mut current = BTreeMap::<(String, i64), (String, String)>::new();
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
    // Additional source fields and relationships participate even when their
    // display formatting or meaning has not yet been implemented.
    let mut related: BTreeMap<i64, BTreeSet<(String, usize)>> = BTreeMap::new();
    for table in [
        "item_obtaining_routes",
        "item_obtaining_sources",
        "item_progression_links",
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
            related.entry(id).or_default().insert((name, pos as usize));
        }
    }
    for result in db
        .prepare(
            "SELECT result_item_id,container_item_id,source_name,source_row FROM item_rewards",
        )?
        .query_map([], |r| {
            Ok((
                r.get::<_, Option<i64>>(0)?,
                r.get::<_, Option<i64>>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?
    {
        let (result, owner, source, pos) = result?;
        for id in [result, owner].into_iter().flatten() {
            related
                .entry(id)
                .or_default()
                .insert((source.clone(), pos as usize));
        }
    }
    // Allocation changes affect every item reachable through the dungeon pool,
    // independently of whether its own relative weight changed.
    for result in db.prepare("SELECT DISTINCT COALESCE(e.item_id,r.item_id),r.source_name,r.source_row FROM dungeon_reward_rolls r LEFT JOIN reward_pool_entries e ON e.pool_id=r.pool_id WHERE COALESCE(e.item_id,r.item_id) IS NOT NULL")?.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?)))? {
        let (id,name,pos)=result?;
        related.entry(id).or_default().insert((name,pos as usize));
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
    let skill_evidence = skill_sources(db, batch)?;
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
    let mut family_previous: BTreeMap<(&str, &str), &Entry> = BTreeMap::new();
    for ((family, _), entry) in &old {
        let key = (family.as_str(), entry.profile.as_str());
        if family_previous.get(&key).is_none_or(|prior| {
            prior.observed.parse::<u64>().unwrap_or(0) <= entry.observed.parse::<u64>().unwrap_or(0)
        }) {
            family_previous.insert(key, entry);
        }
    }
    for ((family, id), (content, localized)) in &current {
        let prior = old.get(&(family.clone(), *id));
        let family_prior = family_previous
            .get(&(family.as_str(), profile.as_str()))
            .copied();
        let reference = prior
            .or(family_prior)
            .map(|e| version(db, &e.observed))
            .transpose()?;
        let can_compare =
            comparable && prior.or(family_prior).is_some_and(|e| e.profile == profile);
        let (state, change) = if !can_compare {
            if old_db.is_none() {
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
            "INSERT INTO catalogue_entries VALUES(?1,?2,?3,?4,?5,?6,1,?7,?8,?9,?10,?11,?12,?13)",
            params![
                family,
                id,
                content,
                localized,
                locale,
                profile,
                build,
                reference,
                latest,
                change_build,
                change_reference,
                state,
                modified_build
            ],
        )?;
        if matches!(family.as_str(), "items" | "skills") {
            db.execute(
                &format!("UPDATE {family} SET client_version_id=?1 WHERE id=?2"),
                params![modified_build, id],
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
    coverage(
        db,
        "build_comparison",
        if comparable { "verified" } else { "partial" },
        "latest observed change only; first import is a baseline; source/reader coverage changes are not attributed to a game patch; missing intermediate builds form an observation interval",
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
    let result = hash(&serde_json::to_vec(&batch[table].layout.rows[position])?);
    cache.insert(key, result.clone());
    Ok(result)
}

type Evidence = BTreeSet<(String, usize)>;
fn skill_sources(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
) -> Result<BTreeMap<i64, Evidence>> {
    let mut ids = BTreeMap::new();
    let mut groups: BTreeMap<(String, String), Evidence> = BTreeMap::new();
    let mut abnormals: BTreeMap<i64, Evidence> = BTreeMap::new();
    let mut acquired: BTreeMap<i64, Evidence> = BTreeMap::new();
    for (name, table) in batch {
        if !matches!(
            name.as_str(),
            "SkillEffect"
                | "SkillEffectLv"
                | "SkillAbnormal"
                | "SkillAbnormalEffect"
                | "SkillAbnormalEffectLv"
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
                "SkillAbnormalEffectLv" => Some("AbnormalEffectLevelGroupId"),
                "SkillLv" => Some("SkillLvGroupId"),
                _ => None,
            };
            if let Some(group) = field.and_then(|f| string(&row[f])) {
                groups
                    .entry((name.clone(), group.into()))
                    .or_default()
                    .insert((name.clone(), pos));
            }
            if name == "SkillAbnormalEffect"
                && let Some(id) = number(row, "AbnormalId")
            {
                abnormals.entry(id).or_default().insert((name.clone(), pos));
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
                    if let Some(pos) = ids.get(&("SkillAbnormal".into(), *abnormal)) {
                        refs.insert(("SkillAbnormal".into(), *pos));
                    }
                    if let Some(effects) = abnormals.get(abnormal) {
                        refs.extend(effects.clone());
                        for (_, pos) in effects {
                            if let Some(group) = string(
                                &batch["SkillAbnormalEffect"].layout.rows[*pos]["AbnormalEffectLvGroupId"],
                            ) && let Some(levels) =
                                groups.get(&("SkillAbnormalEffectLv".into(), group.into()))
                            {
                                refs.extend(levels.clone());
                            }
                        }
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
