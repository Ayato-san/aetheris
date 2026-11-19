//! Transient comparisons, never a stored change journal or a stored row hash.
use super::*;

#[derive(Default)]
struct Effect {
    hash: Sha256,
    group: Option<String>,
    ty: Option<String>,
    abnormal: Option<i64>,
    parameters: BTreeMap<usize, String>,
}
#[derive(Default)]
struct Skill {
    hash: Sha256,
    version: i64,
    needed: projection::Needed,
}

fn values(
    row: &rusqlite::Row<'_>,
    columns: &[String],
    localized: bool,
) -> rusqlite::Result<Vec<Value>> {
    let mut result = Vec::new();
    for (i, name) in columns.iter().enumerate() {
        if name == "client_version_id" || (!localized && name.ends_with("_localized")) {
            continue;
        }
        result.push(match row.get::<_, SqlValue>(i)? {
            SqlValue::Null => Value::Null,
            SqlValue::Integer(v) => Value::from(v),
            SqlValue::Real(v) => Value::from(v),
            SqlValue::Text(v) => Value::from(v),
            SqlValue::Blob(_) => return Err(rusqlite::Error::InvalidQuery),
        });
    }
    Ok(result)
}
fn update(skill: &mut Skill, table: &str, row: Vec<Value>) -> Result<()> {
    skill.hash.update(table.as_bytes());
    skill.hash.update(serde_json::to_vec(&row)?);
    for value in row {
        if let Value::String(text) = value {
            skill.needed.text(&text);
        }
    }
    Ok(())
}

pub(super) fn snapshot(db: &Connection, localized: bool) -> Result<BTreeMap<i64, (String, i64)>> {
    // Legacy development projections establish a new baseline once, rather than
    // pretending their removed technical columns represent product changes.
    let compact = db
        .prepare("PRAGMA table_info(skills)")?
        .query_map([], |r| r.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .iter()
        .any(|n| n == "rarity_id");
    if !compact {
        return Ok(BTreeMap::new());
    }
    if !db
        .prepare("PRAGMA table_info(skill_class_permissions)")?
        .query_map([], |r| r.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .iter()
        .any(|n| n == "is_listed")
    {
        return Ok(BTreeMap::new());
    }
    let mut skills = BTreeMap::<i64, Skill>::new();
    let mut statement = db.prepare("SELECT s.id,s.client_version_id,s.name_en,s.name_localized,i.path AS icon,r.code AS rarity,t.code AS type,d.code AS damage_type,s.description_en,s.description_localized,s.mp_cost,s.cooldown,s.range,s.uses_weapon_range,s.max_level,s.stagger_damage FROM skills s LEFT JOIN skill_icons i ON i.id=s.icon_id LEFT JOIN skill_rarities r ON r.id=s.rarity_id LEFT JOIN skill_types t ON t.id=s.type_id LEFT JOIN skill_damage_types d ON d.id=s.damage_type_id ORDER BY s.id")?;
    let columns = statement
        .column_names()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut cursor = statement.query([])?;
    while let Some(row) = cursor.next()? {
        let mut skill = Skill {
            version: row.get(1)?,
            ..Skill::default()
        };
        update(&mut skill, "skills", values(row, &columns, localized)?)?;
        skills.insert(row.get(0)?, skill);
    }
    for (table, order) in [
        ("skill_levels", "skill_id,level"),
        ("skill_weapons", "skill_id,weapon_id"),
        ("skill_cast_times", "skill_id,level,kind,stage"),
        ("skill_class_permissions", "skill_id,class_id"),
        ("skill_specializations", "skill_id,specialization_id"),
        (
            "skill_conditions",
            "skill_id,effect_id,kind,condition_id,type,parameter,position",
        ),
        ("skill_effect_links", "skill_id,position,effect_id"),
    ] {
        let sql=match table {
            "skill_weapons"=>"SELECT s.skill_id,w.code FROM skill_weapons s JOIN skill_weapon_types w ON w.id=s.weapon_id ORDER BY s.skill_id,w.code".into(),
            "skill_class_permissions"=>"SELECT s.skill_id,c.code,s.is_listed FROM skill_class_permissions s JOIN skill_classes c ON c.id=s.class_id ORDER BY s.skill_id,c.code".into(),
            "skill_effect_links"=>"SELECT l.skill_id,l.position,k.code AS kind,l.effect_id,l.filter_id FROM skill_effect_links l JOIN skill_effect_kinds k ON k.id=l.kind_id ORDER BY l.skill_id,l.position,l.effect_id".into(),
            _=>format!("SELECT * FROM {table} WHERE skill_id IS NOT NULL ORDER BY {order}")
        };
        let mut statement = db.prepare(&sql)?;
        let columns = statement
            .column_names()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let mut cursor = statement.query([])?;
        while let Some(row) = cursor.next()? {
            if let Some(skill) = skills.get_mut(&row.get::<_, i64>(0)?) {
                if table == "skill_effect_links" {
                    skill.needed.se.insert(row.get(3)?);
                    if let Some(id) = row.get::<_, Option<i64>>(4)? {
                        skill.needed.sef.insert(id);
                    }
                }
                update(skill, table, values(row, &columns, localized)?)?;
            }
        }
    }
    let mut effects = BTreeMap::<(String, i64), Effect>::new();
    let mut variants = BTreeMap::<(String, String), Vec<(String, i64)>>::new();
    let mut abnormal_effects = BTreeMap::<i64, Vec<(String, i64)>>::new();
    let mut statement = db.prepare("SELECT k.code AS kind,e.id,e.level_group,e.level,e.effect_type,e.abnormal_id,e.max_level,e.aggro_absolute,e.aggro_ratio,e.target_count_max FROM skill_effects e JOIN skill_effect_kinds k ON k.id=e.kind_id ORDER BY k.code,e.id")?;
    let columns = statement
        .column_names()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut cursor = statement.query([])?;
    while let Some(row) = cursor.next()? {
        let key: (String, i64) = (row.get(0)?, row.get(1)?);
        let mut effect = Effect {
            group: row.get(2)?,
            ty: row.get(4)?,
            abnormal: row.get(5)?,
            ..Effect::default()
        };
        effect
            .hash
            .update(serde_json::to_vec(&values(row, &columns, true)?)?);
        if key.0.ends_with("_level")
            && let Some(group) = &effect.group
        {
            variants
                .entry((key.0.clone(), group.clone()))
                .or_default()
                .push(key.clone());
        }
        if key.0 == "abe"
            && let Some(abnormal) = effect.abnormal
        {
            abnormal_effects
                .entry(abnormal)
                .or_default()
                .push(key.clone());
        }
        effects.insert(key, effect);
    }
    let mut statement=db.prepare("SELECT k.code,v.effect_id,v.position,v.value FROM skill_effect_values v JOIN skill_effect_kinds k ON k.id=v.kind_id ORDER BY k.code,v.effect_id,v.position")?;
    let mut cursor = statement.query([])?;
    while let Some(row) = cursor.next()? {
        let key = (row.get::<_, String>(0)?, row.get::<_, i64>(1)?);
        if let Some(effect) = effects.get_mut(&key) {
            let position = row.get::<_, i64>(2)? as usize;
            let value: String = row.get(3)?;
            effect.hash.update(serde_json::to_vec(&(position, &value))?);
            if position <= 2 {
                effect.parameters.insert(position, value);
            }
        }
    }
    let mut statement=db.prepare("SELECT * FROM skill_conditions WHERE skill_id IS NULL ORDER BY effect_id,kind,condition_id,type,parameter,position")?;
    let columns = statement
        .column_names()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut cursor = statement.query([])?;
    while let Some(row) = cursor.next()? {
        if let Some(effect) = effects.get_mut(&("se".into(), row.get(1)?)) {
            effect
                .hash
                .update(serde_json::to_vec(&values(row, &columns, localized)?)?);
        }
    }
    let mut hashes = BTreeMap::new();
    for (key, effect) in &effects {
        let mut hash = effect.hash.clone();
        if let Some(group) = &effect.group
            && let Some(levels) = variants.get(&(format!("{}_level", key.0), group.clone()))
        {
            for level in levels {
                hash.update(effects[level].hash.clone().finalize());
            }
        }
        hashes.insert(key.clone(), hash.finalize());
    }
    let mut enriched = hashes.clone();
    for (key, effect) in &effects {
        if key.0 != "se" {
            continue;
        }
        let index = match effect.ty.as_deref() {
            Some("Abnormal" | "Abnormal_True" | "Abnormal_Cc") => 2,
            Some("Abnormal_Passive" | "Toggle") => 0,
            _ => continue,
        };
        let mut definitions = vec![key.clone()];
        if let Some(group) = &effect.group
            && let Some(levels) = variants.get(&("se_level".into(), group.clone()))
        {
            definitions.extend(levels.clone());
        }
        let mut ids = BTreeSet::new();
        for definition in definitions {
            if let Some(id) = effects[&definition]
                .parameters
                .get(&index)
                .and_then(|v| v.parse::<i64>().ok())
            {
                ids.insert(id);
            }
        }
        let mut hash = Sha256::new();
        hash.update(hashes[key]);
        for id in ids {
            if let Some(abes) = abnormal_effects.get(&id) {
                for abe in abes {
                    hash.update(hashes[abe]);
                }
            }
        }
        enriched.insert(key.clone(), hash.finalize());
    }
    let mut result = BTreeMap::new();
    for (id, mut skill) in skills {
        for (kind, ids) in [
            ("se", skill.needed.se),
            ("abe", skill.needed.abe),
            ("sef", skill.needed.sef),
        ] {
            for effect_id in ids {
                if let Some(hash) = enriched.get(&(kind.to_owned(), effect_id)) {
                    skill.hash.update(hash);
                }
            }
        }
        result.insert(id, (hex::encode(skill.hash.finalize()), skill.version));
    }
    Ok(result)
}
