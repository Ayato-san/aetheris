//! Source-backed supplements. A resolved reference establishes identity only,
//! not a drop rate, availability in game or a network correspondence.
use super::*;

fn rows<'a>(batch: &'a BTreeMap<String, TableProbe>, name: &str) -> &'a [Value] {
    batch
        .get(name)
        .map(|t| t.layout.rows.as_slice())
        .unwrap_or(&[])
}
fn list<'a>(r: &'a Value, k: &str) -> &'a [Value] {
    r[k].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn icon(db: &Connection, reference: &str, images: &ImageIndex) -> Result<i64> {
    if let Some(id) = db
        .query_row(
            "SELECT id FROM catalogue_icons WHERE reference=?1",
            [reference],
            |r| r.get(0),
        )
        .optional()?
        && !images.searched
    {
        return Ok(id);
    }
    let id: i64 = db.query_row(
        "SELECT COALESCE((SELECT id FROM catalogue_icons WHERE reference=?1),(SELECT COALESCE(MAX(id),0)+1 FROM catalogue_icons))",
        [reference],
        |r| r.get(0),
    )?;
    let candidates = images
        .files
        .get(reference)
        .or_else(|| images.names.get(reference));
    let (data, mime, state) = match candidates.map(Vec::as_slice) {
        Some([p]) => {
            let data = read(p)?;
            if data.len() > 16 * 1024 * 1024 {
                return Err(invalid("icon exceeds 16 MiB").into());
            }
            let mime = if data.starts_with(b"\x89PNG\r\n\x1a\n") {
                "image/png"
            } else if data.starts_with(b"\xff\xd8\xff") {
                "image/jpeg"
            } else if data.starts_with(b"RIFF") && data.get(8..12) == Some(b"WEBP") {
                "image/webp"
            } else {
                return Err(invalid("invalid extracted icon image").into());
            };
            (Some(data), Some(mime), "available")
        }
        Some(_) => (None, None, "ambiguous"),
        None => (
            None,
            None,
            if images.ambiguous.contains(reference) {
                "ambiguous"
            } else if !images.searched {
                "not_researched"
            } else {
                "not_found"
            },
        ),
    };
    let sha = data.as_ref().map(|b| hash(b));
    db.execute(
        "INSERT INTO catalogue_icons VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET image=excluded.image,mime_type=excluded.mime_type,image_state=excluded.image_state,image_sha256=excluded.image_sha256",
        params![id, reference, data, mime, state, sha],
    )?;
    Ok(id)
}
struct ImageIndex {
    files: BTreeMap<String, Vec<PathBuf>>,
    names: BTreeMap<String, Vec<PathBuf>>,
    ambiguous: BTreeSet<String>,
    searched: bool,
}

fn image_index(root: Option<&Path>) -> Result<ImageIndex> {
    let mut index: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let mut names: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let mut ambiguous = BTreeSet::new();
    if let Some(root) = root {
        let report = root.join("extraction.json");
        if report.is_file() {
            if fs::metadata(&report)?.len() > 16 * 1024 * 1024 {
                return Err(invalid("icon extraction report exceeds 16 MiB").into());
            }
            let report: Value = serde_json::from_slice(&read(&report)?)?;
            let results = report["results"]
                .as_array()
                .filter(|rows| rows.len() <= 100000)
                .ok_or_else(|| invalid("invalid icon extraction report"))?;
            for row in results {
                if row["state"] == "ambiguous" {
                    let reference = row["reference"]
                        .as_str()
                        .ok_or_else(|| invalid("invalid ambiguous icon reference"))?;
                    ambiguous.insert(reference.to_owned());
                }
            }
        }
        let mut dirs = vec![root.to_owned()];
        let mut count = 0;
        while let Some(dir) = dirs.pop() {
            for e in fs::read_dir(dir)? {
                let e = e?;
                let ty = e.file_type()?;
                if ty.is_symlink() {
                    continue;
                }
                let p = e.path();
                count += 1;
                if count > 100000 {
                    return Err(invalid("icon directory exceeds bounds").into());
                }
                if ty.is_dir() {
                    dirs.push(p);
                    continue;
                }
                if !matches!(
                    p.extension().and_then(|s| s.to_str()),
                    Some("png" | "webp" | "jpg" | "jpeg")
                ) {
                    continue;
                }
                let key = p
                    .strip_prefix(root)?
                    .with_extension("")
                    .to_string_lossy()
                    .replace('\\', "/");
                index.entry(key.clone()).or_default().push(p.clone());
                if key.contains('/')
                    && let Some(name) = p.file_stem().and_then(|s| s.to_str())
                {
                    names.entry(name.into()).or_default().push(p);
                }
            }
        }
    }
    Ok(ImageIndex {
        files: index,
        names,
        ambiguous,
        searched: root.is_some(),
    })
}
fn ui_sprites(
    db: &Connection,
    root: Option<&Path>,
    images: &ImageIndex,
    mapping: &str,
) -> Result<()> {
    let Some(path) = root
        .map(|p| p.join("ui-sprites.json"))
        .filter(|p| p.is_file())
    else {
        return coverage(
            db,
            "ui.sprites",
            "not_researched",
            "No UI sprite extraction manifest supplied",
        );
    };
    if fs::metadata(&path)?.len() > 16 * 1024 * 1024 {
        return Err(invalid("UI sprite manifest exceeds 16 MiB").into());
    }
    let manifest: Value = serde_json::from_slice(&read(&path)?)?;
    if manifest["format"] != 1 || manifest["mapping_sha256"].as_str() != Some(mapping) {
        return Err(invalid("incompatible UI sprite manifest").into());
    }
    let sprites = manifest["sprites"]
        .as_array()
        .filter(|s| s.len() <= 32767)
        .ok_or_else(|| invalid("invalid UI sprite list"))?;
    let aliases = manifest["aliases"]
        .as_array()
        .filter(|s| s.len() <= 100000)
        .ok_or_else(|| invalid("invalid UI sprite aliases"))?;
    let errors = manifest["errors"]
        .as_array()
        .ok_or_else(|| invalid("invalid UI sprite errors"))?;
    let mut ids = BTreeMap::new();
    for (position, sprite) in sprites.iter().enumerate() {
        let reference = string(&sprite["reference"])
            .filter(|s| s.starts_with("UI/Sprite/"))
            .ok_or_else(|| invalid("invalid UI sprite image reference"))?;
        let asset = string(&sprite["asset_path"])
            .filter(|s| s.starts_with("/Game/UI/Resource/Sprite/"))
            .ok_or_else(|| invalid("invalid UI sprite asset path"))?;
        let id = position as i64 + 1;
        if ids.insert(reference, id).is_some() {
            return Err(invalid("duplicate UI sprite reference").into());
        }
        let image = icon(db, reference, images)?;
        db.execute(
            "INSERT INTO ui_sprites VALUES(?1,?2,?3,?4,?5)",
            params![
                id,
                asset,
                image,
                number(sprite, "width"),
                number(sprite, "height")
            ],
        )?;
    }
    for alias in aliases {
        let reference = string(&alias["reference"])
            .ok_or_else(|| invalid("invalid UI sprite alias reference"))?;
        let id = ids
            .get(reference)
            .ok_or_else(|| invalid("UI sprite alias target missing"))?;
        db.execute(
            "INSERT INTO ui_sprite_aliases VALUES(?1,?2,?3)",
            params![
                string(&alias["table"]).ok_or_else(|| invalid("missing UI sprite table"))?,
                string(&alias["key"]).ok_or_else(|| invalid("missing UI sprite key"))?,
                id
            ],
        )?;
    }
    let available: i64 = db.query_row(
        "SELECT COUNT(*) FROM ui_sprites s JOIN catalogue_icons i ON i.id=s.icon_id WHERE i.image_state='available'",
        [], |r| r.get(0),
    )?;
    coverage(
        db,
        "ui.sprites",
        if errors.is_empty() && available == sprites.len() as i64 {
            "verified"
        } else {
            "partial"
        },
        &format!(
            "{} sprites, {} available images, {} UI table keys, {} extraction errors",
            sprites.len(),
            available,
            aliases.len(),
            errors.len()
        ),
    )
}

fn attach_icon(
    db: &Connection,
    family: &str,
    id: i64,
    reference: Option<&str>,
    images: &ImageIndex,
) -> Result<Option<i64>> {
    let Some(reference) = reference else {
        return Ok(None);
    };
    let key = icon(db, reference, images)?;
    db.execute(
        "INSERT INTO catalogue_entity_icons VALUES(?1,?2,?3)",
        params![family, id, key],
    )?;
    Ok(Some(key))
}
pub(super) fn text_state(
    db: &Connection,
    family: &str,
    id: i64,
    field: &str,
    key: Option<&str>,
    en: &Localization,
    loc: &Localization,
) -> Result<()> {
    let english = text(en, key);
    let localized = text(loc, key);
    let en_state = if key.is_none() {
        "no_reference"
    } else if english.is_some() {
        "available"
    } else {
        "missing"
    };
    let state = if key.is_none() {
        "no_reference"
    } else if localized.is_some() {
        "available"
    } else if english.is_some() {
        "fallback_en"
    } else {
        "missing"
    };
    db.execute(
        "INSERT INTO catalogue_texts VALUES(?1,?2,?3,?4,?5,?6)",
        params![family, id, field, key, en_state, state],
    )?;
    Ok(())
}
pub(super) fn write(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    en: &Localization,
    loc: &Localization,
    image_dir: Option<&Path>,
) -> Result<()> {
    let images = image_index(image_dir)?;
    ui_sprites(
        db,
        image_dir,
        &images,
        batch
            .get("Item")
            .map(|t| t.mapping_sha256.as_str())
            .unwrap_or(""),
    )?;
    for row in rows(batch, "CurrencyInfo") {
        let (Some(category), Some(reference)) = (
            string(&row["CurrencyCategory"]),
            string(&row["IconResource"]),
        ) else {
            continue;
        };
        let id: Option<i64> = db
            .query_row(
                "SELECT id FROM sell_currencies WHERE sell_currency_type=?1",
                [category],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(id) = id {
            attach_icon(db, "sell_currencies", id, Some(reference), &images)?;
        }
    }
    let items = rows(batch, "Item");
    let item_names: BTreeMap<&str, i64> = items
        .iter()
        .filter_map(|r| Some((string(&r["Name"])?, number(r, "ID")?)))
        .collect();
    for row in rows(batch, "EnumString") {
        let (Some(ty), Some(code), Some(key)) = (
            string(&row["Enum"]),
            string(&row["Value"]),
            string(&row["String"]["Key"]),
        ) else {
            continue;
        };
        db.execute(
            "INSERT INTO catalogue_labels VALUES(?1,?2,?3,?4,?5)",
            params![ty, code, key, text(en, Some(key)), text(loc, Some(key))],
        )?;
    }
    // Keep the source enum unchanged and derive the English display name from
    // the client's translation table, never from a hand-written grade mapping.
    db.execute("UPDATE rarities SET rarity=COALESCE((SELECT l.name_en FROM catalogue_labels l WHERE l.enum_type='EItemGrade' AND 'EItemGrade::'||l.code=rarities.rarity_source),rarity)", [])?;
    // StatName keys are supplied by the client localization, independently of EnumString.
    for (key, english) in &en.entries {
        let Some(code) = key
            .strip_prefix("String_StatName_")
            .and_then(|k| k.strip_suffix("_body"))
        else {
            continue;
        };
        let localized = loc.entries.get(key);
        db.execute(
            "INSERT INTO catalogue_labels VALUES('EStat',?1,?2,?3,?4)",
            params![code, key, english, localized],
        )?;
        db.execute(
            "UPDATE stats SET name_en=?1,name_localized=?2 WHERE code=?3",
            params![english, localized, format!("EStat::{code}")],
        )?;
    }
    for r in items {
        let id = number(r, "ID").ok_or_else(|| invalid("invalid item ID"))?;
        db.execute(
            "INSERT INTO item_details VALUES(?1,?2)",
            params![id, number(r, "PermitLevelMin"),],
        )?;
        attach_icon(db, "items", id, string(&r["IconRes"]), &images)?;
        for (field, source) in [("name", "Desc"), ("description", "DescLong")] {
            text_state(db, "items", id, field, string(&r[source]["Key"]), en, loc)?;
        }
        for object in [r, &r["$variant"]] {
            if let Some(object) = object.as_object() {
                for (code, v) in object {
                    if !(code.starts_with("Can") || code.starts_with("bCan"))
                        || code == "CanUseClassList"
                    {
                        continue;
                    }
                    let value = v.as_bool().or_else(|| {
                        v.as_i64().and_then(|v| match v {
                            0 => Some(false),
                            1 => Some(true),
                            _ => None,
                        })
                    });
                    if value == Some(false) {
                        let flag = dictionary(db, "flags", code)?;
                        db.execute(
                            "INSERT INTO item_false_flags VALUES(?1,?2)",
                            params![id, flag],
                        )?;
                    }
                }
            }
        }
    }
    for row in rows(batch, "ItemTypeFilter") {
        let (Some(family), Some(category), Some(filter)) = (
            string(&row["ItemType"]),
            string(&row["Category"]),
            string(&row["Filter"]),
        ) else {
            continue;
        };
        db.execute("INSERT OR IGNORE INTO item_category_filters SELECT c.id,?1 FROM item_categories c JOIN item_families f ON f.id=c.family_id WHERE c.code=?2 AND f.code=?3",params![filter,category,family.strip_prefix("EItemType::").unwrap_or(family)])?;
    }
    for (pos, r) in rows(batch, "ItemSet").iter().enumerate() {
        let Some(code) = string(&r["SetName"]) else {
            continue;
        };
        let id = dictionary(db, "sets", code)?;
        let key = string(&r["SetNameDesc"]["Key"]);
        db.execute(
            "INSERT INTO item_set_definitions VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                id,
                pos as i64,
                text(en, key),
                text(loc, key),
                string(&r["SetType"]),
                string(&r["SetIconRes"])
            ],
        )?;
        for reference in list(r, "SetItems").iter().filter_map(Value::as_str) {
            db.execute(
                "INSERT INTO item_set_members VALUES(?1,?2,?3,?4)",
                params![
                    id,
                    pos as i64,
                    reference,
                    item_names.get(reference).copied()
                ],
            )?;
        }
    }
    for (pos, r) in rows(batch, "ItemSetEffect").iter().enumerate() {
        if let Some(code) = string(&r["SetName"]) {
            let id = dictionary(db, "sets", code)?;
            db.execute(
                "INSERT INTO item_set_bonuses VALUES(?1,?2,?3,?4)",
                params![
                    id,
                    pos as i64,
                    number(r, "SetEquipCount"),
                    number(r, "OptAbnormalId").filter(|n| *n != 0)
                ],
            )?;
        }
    }
    let invalid_sets: i64 = db.query_row(
        "SELECT count(*) FROM (SELECT item_id FROM item_sets GROUP BY item_id HAVING count(*)>1)",
        [],
        |r| r.get(0),
    )?;
    if invalid_sets > 0 {
        coverage(
            db,
            "items.arcana_sets",
            "partial",
            "multiple set references require interpretation; source links retained",
        )?;
    } else {
        coverage(
            db,
            "items.arcana_sets",
            "verified",
            "at most one set reference per item; definitions/bonuses retained independently",
        )?;
    }
    progression(db, batch, items)?;
    obtaining::write(db, batch)?;
    rewards(db, batch, items, &item_names)?;
    relations::write(db, batch, en, loc)?;
    dungeon_rewards::write(db, batch, en, loc)?;
    let texts: BTreeMap<_, _> = rows(batch, "SkillString")
        .iter()
        .filter_map(|r| Some((string(&r["Name"])?, r)))
        .collect();
    for r in rows(batch, "Skill") {
        let id = number(r, "ID").ok_or_else(|| invalid("invalid skill ID"))?;
        attach_icon(db, "skills", id, string(&r["SkillIcon"]), &images)?;
        let t = string(&r["SkillString_Key"]).and_then(|k| texts.get(k));
        for (field, key) in [("name", "SkillName"), ("description", "SkillDescEffect")] {
            text_state(
                db,
                "skills",
                id,
                field,
                t.and_then(|r| string(&r[key]["Key"])),
                en,
                loc,
            )?;
        }
    }
    let strings: BTreeMap<_, _> = rows(batch, "SkillAbnormalString")
        .iter()
        .filter_map(|r| Some((string(&r["Name"])?, r)))
        .collect();
    for r in rows(batch, "SkillAbnormal") {
        let id = number(r, "ID").ok_or_else(|| invalid("invalid abnormal ID"))?;
        let t = string(&r["SkillAbnormalString_Key"]).and_then(|k| strings.get(k));
        let name = t.and_then(|r| string(&r["DescName"]["Key"]));
        let desc = t.and_then(|r| string(&r["DescEffect"]["Key"]));
        let icon = attach_icon(db, "abnormals", id, string(&r["AbnormalIcon"]), &images)?;
        db.execute(
            "INSERT INTO skill_abnormals VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![
                id,
                text(en, name),
                text(loc, name),
                text(en, desc),
                text(loc, desc),
                string(&r["AbnormalType"]),
                string(&r["DisplayCategory"]),
                string(&r["SkillDamageType"]),
                number(r, "AbnormalEffectLevelMax"),
                number(r, "AbnormalOverlapCount"),
                string(&r["AbnormalOverlapTimeType"]),
                string(&r["AbnormalReplaceType"]),
                icon
            ],
        )?;
        text_state(db, "abnormals", id, "name", name, en, loc)?;
        text_state(db, "abnormals", id, "description", desc, en, loc)?;
    }
    let mut group_types: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for row in rows(batch, "SkillEffect") {
        if let (Some(group), Some(ty)) = (
            string(&row["SkillEffectLvGroupId"]),
            string(&row["EffectType"]),
        ) {
            group_types.entry(group).or_default().insert(ty);
        }
    }
    for (name, kind) in [("SkillEffect", "se"), ("SkillEffectLv", "se_level")] {
        for row in rows(batch, name) {
            let types = if name == "SkillEffect" {
                string(&row["EffectType"])
                    .into_iter()
                    .collect::<BTreeSet<_>>()
            } else {
                string(&row["SkillEffectLvGroupId"])
                    .and_then(|g| group_types.get(g))
                    .cloned()
                    .unwrap_or_default()
            };
            for ty in types {
                let pos = match ty {
                    "ESkillEffectType::Abnormal"
                    | "ESkillEffectType::Abnormal_True"
                    | "ESkillEffectType::Abnormal_Cc" => 2,
                    "ESkillEffectType::Abnormal_Passive" | "ESkillEffectType::Toggle" => 0,
                    _ => continue,
                };
                if let Some(id) = list(row, "EffectValueList")
                    .get(pos)
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse::<i64>().ok())
                    .filter(|v| *v > 0)
                {
                    let resolved = db
                        .prepare("SELECT 1 FROM skill_abnormals WHERE id=?1")?
                        .exists([id])?;
                    db.execute("INSERT OR IGNORE INTO skill_abnormal_links VALUES(?1,?2,?3,?4,?5,'candidate')",params![kind,number(row,"ID"),pos as i64,id,resolved])?;
                }
            }
        }
    }
    coverage(
        db,
        "skills.effects",
        "verified",
        "all decoded SE/ABE/SEF and level rows retained, independently of description usage",
    )?;
    coverage(
        db,
        "skills.abnormal_parameters",
        "uninterpreted",
        "original positional values retained; duration units and SE/ABE application positions require semantic validation",
    )?;
    coverage(
        db,
        "skills.network_ids",
        "not_researched",
        "no client/network ID equivalence assumed",
    )?;
    coverage(
        db,
        "icons.images",
        if image_dir.is_some() {
            "partial"
        } else {
            "not_researched"
        },
        "exact references deduplicated; supplied extracted images matched without guessed paths; Unreal texture decoding is not implemented",
    )?;
    coverage(
        db,
        "items.required_level",
        "verified",
        "PermitLevelMin retained without deriving values; other fields remain in source evidence",
    )?;
    coverage(
        db,
        "items.probability_scales",
        "uninterpreted",
        "generic probabilities/weights retained; verified Party/Raid pool interpretation is reported in items.dungeon_rewards",
    )?;
    coverage(
        db,
        "classification.localized_labels",
        "partial",
        "EnumString translations retained when present; no invented parent or class-index mapping",
    )?;
    Ok(())
}

fn progression(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    items: &[Value],
) -> Result<()> {
    for (field, source) in [
        ("EnchantGroup", "Enchant"),
        ("EnchantGroup", "ArcanaEnchant"),
        ("EnchantEffectGroup", "EnchantEffect"),
        ("EnchantEffectGroup", "ArcanaEnchantEffectMainStat"),
        ("ExceedEnchantGroup", "ExceedEnchant"),
        ("SurpassGroup", "ItemSurpass"),
        ("SuccessionGroup", "ItemSuccession"),
        ("SoulAddGroup", "ItemSoulAdd"),
    ] {
        let mut groups: BTreeMap<&str, Vec<(usize, &Value)>> = BTreeMap::new();
        for (pos, row) in rows(batch, source).iter().enumerate() {
            if let Some(group) = string(&row["Group"]) {
                groups.entry(group).or_default().push((pos, row));
            }
        }
        for item in items {
            if let Some(group) = string(&item["$variant"][field])
                && let Some(entries) = groups.get(group)
            {
                for (pos, row) in entries {
                    db.execute(
                        "INSERT INTO item_progression_links VALUES(?1,?2,?3,?4,?5,?6)",
                        params![
                            number(item, "ID"),
                            source,
                            *pos as i64,
                            field,
                            group,
                            number(row, "Level").or_else(|| number(row, "CurrentLevel"))
                        ],
                    )?;
                }
            }
        }
    }
    // Both amplification and surpass rows can reference shared statistics.
    let additional: BTreeMap<&str, usize> = rows(batch, "AdditionalStat")
        .iter()
        .enumerate()
        .filter_map(|(pos, row)| string(&row["Name"]).map(|name| (name, pos)))
        .collect();
    let references=db.prepare("SELECT item_id,source_name,source_row FROM item_progression_links WHERE (source_name='ExceedEnchant' AND source_field='ExceedEnchantGroup') OR (source_name='ItemSurpass' AND source_field='SurpassGroup') ORDER BY item_id,source_name,source_row")?
        .query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut missing = 0;
    for (id, source, pos) in references {
        let row = &rows(batch, &source)[usize::try_from(pos)?];
        if let Some(key) = string(&row["AdditionalStatName"]) {
            if let Some(stat_pos) = additional.get(key) {
                // One dependency can serve several progression levels. The
                // original parent rows retain their individual level/context.
                db.execute("INSERT INTO item_progression_links VALUES(?1,'AdditionalStat',?2,'AdditionalStatName',?3,?4) ON CONFLICT(item_id,source_name,source_row,source_field) DO UPDATE SET level=CASE WHEN level IS excluded.level THEN level ELSE NULL END",params![id,*stat_pos as i64,key,number(row,"CurrentLevel")])?;
            } else {
                missing += 1;
            }
        }
    }
    coverage(
        db,
        "items.progression",
        "partial",
        &format!(
            "explicit group references linked, including ExceedEnchant/ItemSurpass AdditionalStat dependencies; {missing} unmatched additional-stat references; full rows retain costs, stats and probabilities; unrecognized progression fields remain in source evidence"
        ),
    )
}
fn rewards(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    items: &[Value],
    names: &BTreeMap<&str, i64>,
) -> Result<()> {
    let mut containers: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    for r in items {
        if string(&r["$variant"]["UseEffect"]) == Some("EItemUseEffect::RewardBox") {
            for group in list(&r["$variant"], "UseEffectValues") {
                if let Some(group) = string(group) {
                    containers
                        .entry(group)
                        .or_default()
                        .push(number(r, "ID").unwrap());
                }
            }
        }
    }
    for (table, group_field) in [
        ("BoxReward", "Group"),
        ("RewardPackageItem", "PackageName"),
        ("RewardItem", "Name"),
    ] {
        for (pos, r) in rows(batch, table).iter().enumerate() {
            let Some(group) = string(&r[group_field]) else {
                continue;
            };
            for (field, mode, result_kind) in [
                ("Items", "reward", "item"),
                ("SelectItems", "choice", "item"),
                ("PackageItems", "draw", "package"),
            ] {
                for (slot, result) in list(r, field).iter().enumerate() {
                    let reference = string(&result["Item"]).or_else(|| string(&result["ItemName"]));
                    let Some(reference) = reference else { continue };
                    let min = number(result, "ItemCountMin").or_else(|| number(result, "Count"));
                    let max = number(result, "ItemCountMax").or(min);
                    let raw = result
                        .get("ItemProb")
                        .or_else(|| result.get("Prob"))
                        .map(Value::to_string);
                    let owners = containers.get(group).cloned().unwrap_or_default();
                    // A source row is shared by its owners. Keep owner links separately in
                    // source_path to preserve repeated occurrences without collapsing draws.
                    for owner in if owners.is_empty() {
                        vec![None]
                    } else {
                        owners.into_iter().map(Some).collect()
                    } {
                        db.execute("INSERT INTO item_rewards VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![table,pos as i64,format!("{field}/{slot}/owner/{}",owner.map_or(String::new(),|n|n.to_string())),owner,group,mode,result_kind,reference,if result_kind=="item"{names.get(reference).copied()}else{None},min,max,raw])?;
                    }
                }
            }
        }
    }
    coverage(
        db,
        "items.boxes",
        "partial",
        "BoxReward, RewardPackageItem and RewardItem retained with choices, draws, quantities and raw weights; package chains and all conditions remain source-addressable",
    )
}

#[cfg(test)]
mod image_tests {
    use super::*;
    #[test]
    fn ui_sprite_aliases_share_images_and_reject_a_different_mapping() {
        let root = std::env::temp_dir().join(format!("aetheris-ui-sprites-{}", std::process::id()));
        fs::create_dir_all(root.join("UI/Sprite/Common")).unwrap();
        let reference = "UI/Sprite/Common/Coin";
        let png = include_bytes!("../../resources/item-card/can-storage.png");
        fs::write(root.join(format!("{reference}.png")), png).unwrap();
        let manifest = serde_json::json!({"format":1,"mapping_sha256":"mapping",
            "sprites":[{"reference":reference,"asset_path":"/Game/UI/Resource/Sprite/Common/Coin.Coin","width":44,"height":44}],
            "aliases":[{"table":"UI_ImageTable_Composite","key":"Currency_Goldbinding","reference":reference},
                       {"table":"UI_ImageTable_Composite","key":"CoinAlias","reference":reference}],"errors":[]});
        fs::write(
            root.join("ui-sprites.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(include_str!("items.sql")).unwrap();
        db.execute_batch(include_str!("catalogue.sql")).unwrap();
        let images = image_index(Some(&root)).unwrap();
        ui_sprites(&db, Some(&root), &images, "mapping").unwrap();
        let (aliases, sprites): (i64, i64) = db
            .query_row(
                "SELECT COUNT(*),COUNT(DISTINCT sprite_id) FROM ui_sprite_aliases",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((aliases, sprites), (2, 1));
        let image: Vec<u8> = db.query_row("SELECT i.image FROM ui_sprite_aliases a JOIN ui_sprites s ON s.id=a.sprite_id JOIN catalogue_icons i ON i.id=s.icon_id WHERE a.row_key='Currency_Goldbinding'", [], |r|r.get(0)).unwrap();
        assert_eq!(image, png);
        assert!(ui_sprites(&db, Some(&root), &images, "other mapping").is_err());
        fs::remove_file(root.join(format!("{reference}.png"))).unwrap();
        db.execute_batch(
            "DELETE FROM ui_sprite_aliases; DELETE FROM ui_sprites; DELETE FROM catalogue_icons;",
        )
        .unwrap();
        ui_sprites(
            &db,
            Some(&root),
            &image_index(Some(&root)).unwrap(),
            "mapping",
        )
        .unwrap();
        let state: String = db
            .query_row(
                "SELECT state FROM catalogue_coverage WHERE subject='ui.sprites'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(state, "partial");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn exact_icon_paths_override_basename_aliases_and_reported_ambiguities_stay_visible() {
        let root = std::env::temp_dir().join(format!(
            "aetheris-icons-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("Item")).unwrap();
        fs::create_dir_all(root.join("Skill")).unwrap();
        let png = include_bytes!("../../resources/item-card/can-storage.png");
        fs::write(root.join("shared.png"), png).unwrap();
        fs::write(root.join("Item/shared.png"), png).unwrap();
        fs::write(root.join("Item/collision.png"), png).unwrap();
        fs::write(root.join("Skill/collision.png"), png).unwrap();
        fs::write(
            root.join("extraction.json"),
            br#"{"results":[{"reference":"unresolved","state":"ambiguous"}]}"#,
        )
        .unwrap();
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(include_str!("items.sql")).unwrap();
        let images = image_index(Some(&root)).unwrap();
        for (reference, expected) in [
            ("shared", "available"),
            ("Item/shared", "available"),
            ("collision", "ambiguous"),
            ("unresolved", "ambiguous"),
            ("missing", "not_found"),
        ] {
            let id = icon(&db, reference, &images).unwrap();
            let (state, image): (String, Option<Vec<u8>>) = db
                .query_row(
                    "SELECT image_state,image FROM catalogue_icons WHERE id=?1",
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            assert_eq!(state, expected, "{reference}");
            assert_eq!(
                image.as_deref(),
                (expected == "available").then_some(png.as_slice())
            );
        }
        fs::remove_dir_all(root).unwrap();
    }
}
