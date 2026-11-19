//! Explicit product projection. The offline probe owns the complete source data.
use super::*;
use serde_json::json;

pub(super) fn number(row: &Value, field: &str) -> i64 {
    let value = &row[field];
    value
        .as_i64()
        .or_else(|| value["Value"].as_i64())
        .unwrap_or(0)
}
fn identifier(row: &Value, field: &str) -> Result<i64> {
    let id = number(row, field);
    if id <= 0 {
        return Err(invalid(format!("invalid skill {field}")).into());
    }
    Ok(id)
}
fn string(value: &Value) -> Option<&str> {
    value.as_str().filter(|s| !s.is_empty() && *s != "None")
}
fn code(value: &Value) -> Option<&str> {
    string(value)
        .and_then(|s| s.split_once("::").map(|(_, s)| s))
        .filter(|s| *s != "None")
}
fn list<'a>(row: &'a Value, field: &str) -> &'a [Value] {
    row[field].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn translated<'a>(l10n: &'a Localization, key: &Value) -> Option<&'a str> {
    string(&key["Key"])
        .and_then(|key| {
            l10n.entries
                .get(key)
                .or_else(|| l10n.entries.get(&format!("String_{key}_body")))
        })
        .map(String::as_str)
}
fn rows<'a>(probe: &'a skills::SkillProbe, table: &str) -> &'a [Value] {
    &probe.tables[table].layout.rows
}
fn indexed<'a>(probe: &'a skills::SkillProbe, table: &str) -> BTreeMap<i64, &'a Value> {
    rows(probe, table)
        .iter()
        .map(|r| (number(r, "ID"), r))
        .collect()
}
fn nonzero(value: i64) -> Option<i64> {
    (value != 0).then_some(value)
}

// Item/System definitions are not catalogue entries. Keep an exception only
// when a retained skill explicitly needs it as a specialization or charge.
// SE/ABE referenced by item descriptions are selected independently below.
fn catalogue_ids(probe: &skills::SkillProbe) -> BTreeSet<i64> {
    let mut ids: BTreeSet<_> = rows(probe, "Skill")
        .iter()
        .filter(|r| !matches!(code(&r["SkillType"]), Some("Item" | "System")))
        .map(|r| number(r, "ID"))
        .collect();
    let mut edges: Vec<_> = rows(probe, "SpecializedSkill")
        .iter()
        .map(|r| (number(r, "ParentSkillId"), number(r, "ChildSkillId")))
        .collect();
    let charges = indexed(probe, "SkillCharge");
    for r in rows(probe, "Skill") {
        if let Some(charge) = charges.get(&number(r, "ChargeId")) {
            for stage in list(charge, "SkillChargeDataList") {
                edges.push((number(r, "ID"), number(stage, "ChargeSkillId")));
            }
        }
    }
    loop {
        let mut changed = false;
        for (parent, child) in &edges {
            if *child > 0 && ids.contains(parent) {
                changed |= ids.insert(*child);
            }
        }
        if !changed {
            return ids;
        }
    }
}

fn reference(
    db: &Connection,
    table: &str,
    column: &str,
    value: Option<&str>,
) -> Result<Option<i64>> {
    let Some(value) = value else { return Ok(None) };
    if let Some(id) = db
        .query_row(
            &format!("SELECT id FROM {table} WHERE {column}=?1"),
            [value],
            |r| r.get(0),
        )
        .optional()?
    {
        return Ok(Some(id));
    }
    let id: i64 = db.query_row(
        &format!("SELECT COALESCE(MAX(id),0)+1 FROM {table}"),
        [],
        |r| r.get(0),
    )?;
    db.execute(
        &format!("INSERT INTO {table} VALUES(?1,?2)"),
        params![id, value],
    )?;
    Ok(Some(id))
}

#[derive(Default)]
pub(super) struct Needed {
    pub se: BTreeSet<i64>,
    pub abe: BTreeSet<i64>,
    pub sef: BTreeSet<i64>,
}
impl Needed {
    pub(super) fn text(&mut self, text: &str) {
        for part in text.split('{').skip(1) {
            let Some((token, _)) = part.split_once('}') else {
                continue;
            };
            let mut parts = token.split(':');
            let prefix = parts.next().unwrap_or("");
            let ids: Vec<i64> = parts.filter_map(|s| s.parse().ok()).collect();
            match prefix {
                "se" | "se_dmg" => {
                    if let Some(id) = ids.first() {
                        self.se.insert(*id);
                    }
                }
                "abe" => {
                    if let Some(id) = ids.first() {
                        self.abe.insert(*id);
                    }
                }
                "sef" => {
                    if let Some(id) = ids.first() {
                        self.sef.insert(*id);
                    }
                }
                "se_abe" | "se_abe_dmg" => {
                    if let Some(id) = ids.first() {
                        self.se.insert(*id);
                    }
                    if let Some(id) = ids.get(1) {
                        self.abe.insert(*id);
                    }
                }
                _ => {}
            }
        }
    }
}

struct Writer<'a> {
    db: &'a Connection,
    en: &'a Localization,
    localized: &'a Localization,
}
impl Writer<'_> {
    #[allow(clippy::too_many_arguments)]
    fn condition(
        &self,
        skill: Option<i64>,
        effect: Option<i64>,
        kind: &str,
        group: i64,
        ty: &str,
        target: Option<&str>,
        expression: Option<bool>,
        parameter: &str,
        position: usize,
        value: Option<&str>,
        descriptions: (Option<&str>, Option<&str>),
    ) -> Result<()> {
        self.db.prepare("INSERT OR IGNORE INTO skill_conditions VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)")?
            .execute(params![skill,effect,kind,group,ty,target,expression,parameter,position as i64,value,descriptions.0,descriptions.1])?;
        Ok(())
    }
    fn activation(&self, row: &Value) -> Result<()> {
        let id = number(row, "ID");
        for (group, field) in ["CanUseWhenPeaceState", "UnableWhenCanNotMoveControl"]
            .iter()
            .enumerate()
        {
            self.condition(
                Some(id),
                None,
                "activation",
                group as i64,
                field,
                None,
                row[*field].as_bool(),
                "",
                0,
                None,
                (None, None),
            )?;
        }
        let previous = number(row, "ChainSkillPrevSkillId");
        if previous != 0 {
            for field in [
                "ChainSkillPrevSkillId",
                "ChainSkillActivateRate",
                "ChainSkillAvailableTime",
            ] {
                self.condition(
                    Some(id),
                    None,
                    "activation",
                    2,
                    "ChainSkill",
                    None,
                    None,
                    field,
                    0,
                    Some(&number(row, field).to_string()),
                    (None, None),
                )?;
            }
        }
        if let Some(abnormal) = nonzero(number(row, "ToggleOnAbnormalId")) {
            self.condition(
                Some(id),
                None,
                "activation",
                3,
                "Toggle",
                None,
                None,
                "abnormal_id",
                0,
                Some(&abnormal.to_string()),
                (None, None),
            )?;
        }
        for (i, value) in list(row, "CanUseWhenElementalSummonSpawned")
            .iter()
            .enumerate()
        {
            let value = value
                .as_str()
                .and_then(|s| s.split_once("::").map(|(_, s)| s));
            self.condition(
                Some(id),
                None,
                "activation",
                4,
                "ElementalSummon",
                None,
                None,
                "allowed",
                i,
                value,
                (None, None),
            )?;
        }
        Ok(())
    }
    fn effect_conditions(&self, row: &Value) -> Result<()> {
        for (i, c) in list(row, "SkillEffectConditionList").iter().enumerate() {
            let Some(ty) = code(&c["ConditionType"]) else {
                continue;
            };
            let values = list(c, "ConditionValues");
            for j in 0..values.len().max(1) {
                self.condition(
                    None,
                    Some(number(row, "ID")),
                    "effect",
                    i as i64,
                    ty,
                    None,
                    c["ConditionalExpression"].as_bool(),
                    "value",
                    j,
                    values.get(j).and_then(Value::as_str),
                    (None, None),
                )?;
            }
        }
        Ok(())
    }
    fn targets(&self, skill: i64, effect: i64, filter: &Value) -> Result<()> {
        for field in [
            "TargetFilterType",
            "TargetAbnormalType",
            "TargetAbnormalEffectTypeList",
            "TargetAbnormalGroupIdList",
            "TargetAbnormalIdList",
            "TargetPcDieTypeList",
            "bRelationshipEnemy",
            "bRelationshipFriendly",
            "bRelationshipNorelation",
        ] {
            let values = filter[field]
                .as_array()
                .cloned()
                .unwrap_or_else(|| vec![filter[field].clone()]);
            for (i, value) in values.iter().enumerate() {
                if value.is_null() || value.as_bool() == Some(false) {
                    continue;
                }
                let scalar = if value.is_string() {
                    let Some(value) = code(value) else { continue };
                    value.to_owned()
                } else if value.is_boolean() {
                    String::new()
                } else {
                    let Some(value) = nonzero(number(&json!({"value":value}), "value")) else {
                        continue;
                    };
                    value.to_string()
                };
                self.condition(
                    Some(skill),
                    Some(effect),
                    "target",
                    number(filter, "ID"),
                    "TargetFilter",
                    None,
                    value
                        .as_bool()
                        .or_else(|| filter["bTargetAbnormalExpression"].as_bool()),
                    field,
                    i,
                    if value.is_boolean() {
                        None
                    } else {
                        Some(&scalar)
                    },
                    (None, None),
                )?;
            }
        }
        Ok(())
    }
    fn effect(&self, row: &Value, kind: &str, abnormal: Option<&Value>) -> Result<()> {
        let (id_key, group_key, level_key, values_key, type_key) = match kind {
            "se" => (
                "ID",
                "SkillEffectLvGroupId",
                "",
                "EffectValueList",
                "EffectType",
            ),
            "se_level" => (
                "ID",
                "SkillEffectLvGroupId",
                "SkillEffectLv",
                "EffectValueList",
                "",
            ),
            "abe" => (
                "ID",
                "AbnormalEffectLvGroupId",
                "",
                "Values",
                "AbnormalEffectType",
            ),
            "abe_level" => (
                "AbnormalEffectLevelId",
                "AbnormalEffectLevelGroupId",
                "AbnormalEffectLevel",
                "Values",
                "",
            ),
            _ => ("ID", "", "", "", ""),
        };
        let id = identifier(row, id_key)?;
        let kind_id = reference(self.db, "skill_effect_kinds", "code", Some(kind))?;
        self.db
            .prepare("INSERT INTO skill_effects VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)")?
            .execute(params![
                kind_id,
                id,
                string(&row[group_key]),
                nonzero(number(row, level_key)),
                code(&row[type_key]),
                nonzero(number(row, "AbnormalId")),
                abnormal.map(|r| number(r, "AbnormalEffectLevelMax")),
                if kind.starts_with("se") && kind != "sef" {
                    Some(number(row, "AggroAbsolute"))
                } else {
                    None
                },
                if kind.starts_with("se") && kind != "sef" {
                    Some(number(row, "AggroRatio"))
                } else {
                    None
                },
                if kind == "sef" {
                    Some(number(row, "TargetCountMax"))
                } else {
                    None
                }
            ])?;
        let mut parameters = self
            .db
            .prepare("INSERT INTO skill_effect_values VALUES(?1,?2,?3,?4)")?;
        for (i, value) in list(row, values_key).iter().enumerate() {
            parameters.execute(params![
                kind_id,
                id,
                i as i64,
                value
                    .as_str()
                    .ok_or_else(|| invalid("non-text effect parameter"))?
            ])?;
        }
        if kind == "se" {
            self.effect_conditions(row)?;
        }
        Ok(())
    }
}

pub(super) fn write(
    db: &Connection,
    probe: &skills::SkillProbe,
    en: &Localization,
    localized: &Localization,
    version: i64,
) -> Result<()> {
    let w = Writer { db, en, localized };
    for kind in ["se", "abe", "sef", "se_level", "abe_level"] {
        reference(db, "skill_effect_kinds", "code", Some(kind))?;
    }
    let retained = catalogue_ids(probe);
    let texts: BTreeMap<_, _> = rows(probe, "SkillString")
        .iter()
        .filter_map(|r| r["Name"].as_str().map(|k| (k, r)))
        .collect();
    let charges = indexed(probe, "SkillCharge");
    let mut needed = Needed::default();
    let mut group_owners: BTreeMap<i64, Vec<(i64, usize, i64)>> = BTreeMap::new();
    let mut level_owners: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    for r in rows(probe, "Skill") {
        let id = identifier(r, "ID")?;
        if !retained.contains(&id) {
            continue;
        }
        let text = r["SkillString_Key"].as_str().and_then(|k| texts.get(k));
        let name = text.map(|r| &r["SkillName"]).unwrap_or(&Value::Null);
        let description = text.map(|r| &r["SkillDescEffect"]).unwrap_or(&Value::Null);
        let ty = if r["bIsStigmaSkill"].as_bool() == Some(true) {
            Some("Stigma")
        } else {
            code(&r["SkillType"])
        };
        // Preserve the client's value. SkillGrade is not yet proven to be
        // the rarity displayed by the UI; do not patch individual skills.
        let rarity = code(&r["SkillGrade"]).map(|r| if r == "Legend" { "Unique" } else { r });
        let max_level = number(r, "SkillLvMax");
        db.prepare(
            "INSERT INTO skills VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,NULL)",
        )?
        .execute(params![
            id,
            version,
            translated(en, name),
            translated(localized, name),
            reference(db, "skill_icons", "path", string(&r["SkillIcon"]))?,
            reference(db, "skill_rarities", "code", rarity)?,
            reference(db, "skill_types", "code", ty)?,
            reference(
                db,
                "skill_damage_types",
                "code",
                code(&r["SkillDamageType"])
            )?,
            translated(en, description),
            translated(localized, description),
            nonzero(number(r, "NeedCostMp")),
            r["NeedCoolTime"].as_f64(),
            r["NeedSkillUseRange"].as_f64(),
            r["NeedWeaponUseRange"].as_bool().unwrap_or(false),
            max_level
        ])?;
        db.execute(
            "INSERT INTO skill_cast_times VALUES(?1,1,0,'casting',?2,NULL)",
            params![id, number(r, "CastingTime")],
        )?;
        for l in [en, localized] {
            if let Some(text) = translated(l, description) {
                needed.text(text);
            }
        }
        if let Some(group) = string(&r["SkillLvGroupId"]) {
            level_owners.entry(group).or_default().push(id);
        }
        for weapon in list(r, "NeedMainWeaponTypes") {
            if let Some(weapon) = code(weapon) {
                db.prepare("INSERT OR IGNORE INTO skill_weapons VALUES(?1,?2)")?
                    .execute(params![
                        id,
                        reference(db, "skill_weapon_types", "code", Some(weapon))?
                    ])?;
            }
        }
        if let Some(charge) = charges.get(&number(r, "ChargeId")) {
            for (kind, field) in [
                ("charge_minimum", "ChargeMinTime"),
                ("charge_limit", "AddChargeLimitTime"),
            ] {
                if let Some(duration) = nonzero(number(charge, field)) {
                    db.prepare("INSERT INTO skill_cast_times VALUES(?1,1,0,?2,?3,NULL)")?
                        .execute(params![id, kind, duration])?;
                }
            }
            for (stage, c) in list(charge, "SkillChargeDataList").iter().enumerate() {
                db.prepare("INSERT INTO skill_cast_times VALUES(?1,1,?2,'charge_stage',?3,?4)")?
                    .execute(params![
                        id,
                        stage as i64,
                        number(c, "ChargeTime"),
                        nonzero(number(c, "ChargeSkillId"))
                    ])?;
            }
        }
        w.activation(r)?;
        for (i, link) in list(r, "SkillEffectTimeDataList").iter().enumerate() {
            let group = number(link, "SkillEffectGroupId");
            if group != 0 {
                group_owners.entry(group).or_default().push((
                    id,
                    i,
                    number(link, "SkillEffectFilterId"),
                ));
            }
        }
    }
    // Item descriptions can also reference SE/ABE; refreshing skills must retain
    // the resolution dependencies of the items already in the shared database.
    for text in db
        .prepare("SELECT description_en FROM items UNION SELECT description_localized FROM items")?
        .query_map([], |r| r.get::<_, Option<String>>(0))?
    {
        if let Some(text) = text? {
            needed.text(&text);
        }
    }
    for r in rows(probe, "SkillLv") {
        if let Some(owners) = string(&r["SkillLvGroupId"]).and_then(|g| level_owners.get(g)) {
            for owner in owners {
                db.prepare(
                    "INSERT INTO skill_levels(skill_id,level,mp_cost,cooldown) VALUES(?1,?2,?3,?4)",
                )?
                .execute(params![
                    owner,
                    number(r, "SkillLv"),
                    number(r, "NeedCostMp"),
                    r["NeedCoolTime"].as_f64()
                ])?;
                db.execute(
                    "INSERT INTO skill_cast_times VALUES(?1,?2,0,'casting',?3,NULL)",
                    params![owner, number(r, "SkillLv"), number(r, "CastingTime")],
                )?;
            }
        }
    }
    for r in rows(probe, "SkillAcquireData") {
        if !retained.contains(&number(r, "SkillId")) {
            continue;
        }
        db.prepare("INSERT INTO skill_levels(skill_id,level,skill_points,stigma_points) VALUES(?1,?2,?3,?4) ON CONFLICT(skill_id,level) DO UPDATE SET skill_points=excluded.skill_points,stigma_points=excluded.stigma_points")?
            .execute(params![number(r,"SkillId"),number(r,"SkillLevel"),number(r,"CostSkillPoint"),number(r,"CostStigmaPoint")])?;
    }
    let parts = indexed(probe, "SpecializedSkillParts");
    for r in rows(probe, "SpecializedSkill") {
        if !retained.contains(&number(r, "ParentSkillId")) {
            continue;
        }
        let part = parts.get(&number(r, "ID"));
        let key = part
            .and_then(|p| string(&p["SpecializedSkillPartsDesc"]))
            .map(|s| json!({"Key":format!("SkillString_{s}_specialized_skill_desc")}))
            .unwrap_or(Value::Null);
        for l in [en, localized] {
            if let Some(text) = translated(l, &key) {
                needed.text(text);
            }
        }
        db.prepare("INSERT INTO skill_specializations VALUES(?1,?2,?3,?4,?5)")?
            .execute(params![
                number(r, "ParentSkillId"),
                number(r, "ChildSkillId"),
                part.map(|p| number(p, "ParentSkillLv")),
                translated(en, &key),
                translated(localized, &key)
            ])?;
    }
    let filters = indexed(probe, "SkillEffectFilter");
    // A skill's gameplay graph contains much more than its displayed data.
    // Keep groups referenced by description selectors (including companions
    // potentially used by Sum formulas), plus the requested stagger effects.
    let description_groups: BTreeSet<i64> = rows(probe, "SkillEffect")
        .iter()
        .filter(|r| needed.se.contains(&number(r, "ID")))
        .map(|r| number(r, "SkillEffectGroupId"))
        .collect();
    for r in rows(probe, "SkillEffect") {
        if let Some(owners) = group_owners.get(&number(r, "SkillEffectGroupId")) {
            if !description_groups.contains(&number(r, "SkillEffectGroupId"))
                && code(&r["EffectType"]) != Some("GroggyGuardDecreaseEnergy")
            {
                continue;
            }
            let effect = identifier(r, "ID")?;
            needed.se.insert(effect);
            for (skill, position, filter_id) in owners {
                db.prepare("INSERT INTO skill_effect_links VALUES(?1,?2,(SELECT id FROM skill_effect_kinds WHERE code='se'),?3,?4)")?
                    .execute(params![
                        skill,
                        *position as i64,
                        effect,
                        nonzero(*filter_id)
                    ])?;
                if let Some(filter) = filters.get(filter_id) {
                    needed.sef.insert(*filter_id);
                    w.targets(*skill, effect, filter)?;
                }
            }
        }
    }
    let mut se_groups = BTreeMap::new();
    let mut abnormal_ids = BTreeSet::new();
    for r in rows(probe, "SkillEffect")
        .iter()
        .filter(|r| needed.se.contains(&number(r, "ID")))
    {
        w.effect(r, "se", None)?;
        if let Some(group) = string(&r["SkillEffectLvGroupId"]) {
            se_groups
                .entry(group)
                .or_insert_with(BTreeSet::new)
                .insert(code(&r["EffectType"]).unwrap_or(""));
        }
        application_ids(
            code(&r["EffectType"]).unwrap_or(""),
            list(r, "EffectValueList"),
            &mut abnormal_ids,
        );
    }
    for r in rows(probe, "SkillEffectLv") {
        if let Some(types) = string(&r["SkillEffectLvGroupId"]).and_then(|g| se_groups.get(g)) {
            w.effect(r, "se_level", None)?;
            for ty in types {
                application_ids(ty, list(r, "EffectValueList"), &mut abnormal_ids);
            }
        }
    }
    let abnormals = indexed(probe, "SkillAbnormal");
    let mut abe_groups = BTreeSet::new();
    for r in rows(probe, "SkillAbnormalEffect") {
        if needed.abe.contains(&number(r, "ID")) || abnormal_ids.contains(&number(r, "AbnormalId"))
        {
            w.effect(r, "abe", abnormals.get(&number(r, "AbnormalId")).copied())?;
            if let Some(group) = string(&r["AbnormalEffectLvGroupId"]) {
                abe_groups.insert(group);
            }
        }
    }
    for r in rows(probe, "SkillAbnormalEffectLv") {
        if string(&r["AbnormalEffectLevelGroupId"]).is_some_and(|g| abe_groups.contains(g)) {
            w.effect(r, "abe_level", None)?;
        }
    }
    for id in &needed.sef {
        if let Some(filter) = filters.get(id) {
            w.effect(filter, "sef", None)?;
        }
    }
    classes(db, probe, &retained)?;
    conditions(&w, probe, &retained)?;
    // A single unconditional stagger effect can be exposed directly. Several
    // effects remain individually linked, without inventing a hit-count total.
    db.execute_batch("UPDATE skills SET stagger_damage=(SELECT CAST(v.value AS REAL) FROM skill_effect_links l JOIN skill_effects e ON e.kind_id=l.kind_id AND e.id=l.effect_id JOIN skill_effect_values v ON v.kind_id=e.kind_id AND v.effect_id=e.id AND v.position=0 WHERE l.skill_id=skills.id AND e.effect_type='GroggyGuardDecreaseEnergy' AND NOT EXISTS(SELECT 1 FROM skill_conditions c WHERE c.effect_id=e.id AND c.kind='effect')) WHERE (SELECT count(*) FROM skill_effect_links l JOIN skill_effects e ON e.kind_id=l.kind_id AND e.id=l.effect_id WHERE l.skill_id=skills.id AND e.effect_type='GroggyGuardDecreaseEnergy')=1;")?;
    Ok(())
}

fn application_ids(ty: &str, values: &[Value], ids: &mut BTreeSet<i64>) {
    let index = match ty {
        "Abnormal" | "Abnormal_True" | "Abnormal_Cc" => 2,
        "Abnormal_Passive" | "Toggle" => 0,
        _ => return,
    };
    if let Some(id) = values
        .get(index)
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<i64>().ok())
        .filter(|n| *n > 0)
    {
        ids.insert(id);
    }
}

fn conditions(w: &Writer<'_>, probe: &skills::SkillProbe, retained: &BTreeSet<i64>) -> Result<()> {
    let texts: BTreeMap<_, _> = rows(probe, "SkillCondString")
        .iter()
        .filter_map(|r| r["Name"].as_str().map(|k| (k, r)))
        .collect();
    for r in rows(probe, "SkillCond") {
        let text = r["SkillCondString_Key"]
            .as_str()
            .and_then(|k| texts.get(k))
            .map(|t| &t["CondDesc"])
            .unwrap_or(&Value::Null);
        let desc = (translated(w.en, text), translated(w.localized, text));
        for skill in list(r, "CondSkillList") {
            let skill = skill["Value"].as_i64();
            if !skill.is_some_and(|id| retained.contains(&id)) {
                continue;
            }
            for (parameter, value) in [
                (
                    "abnormal_effect_type",
                    code(&r["CondAbnormalEffectType"]).map(str::to_owned),
                ),
                (
                    "abnormal_group",
                    nonzero(number(r, "CondAbnormalGroupId")).map(|v| v.to_string()),
                ),
            ] {
                w.condition(
                    skill,
                    None,
                    "activation",
                    number(r, "ID"),
                    code(&r["CondType"]).unwrap_or("None"),
                    code(&r["CondSelectTarget"]),
                    None,
                    parameter,
                    0,
                    value.as_deref(),
                    desc,
                )?;
            }
        }
    }
    for r in rows(probe, "SkillLink") {
        let skill = Some(number(r, "SkillId"));
        if !retained.contains(&number(r, "SkillId")) {
            continue;
        }
        for (i, c) in list(r, "LinkConditionList").iter().enumerate() {
            if let Some(value) = string(&c["Condition"]) {
                w.condition(
                    skill,
                    None,
                    "activation",
                    number(r, "ID"),
                    "Link",
                    None,
                    c["Expression"].as_bool(),
                    "condition",
                    i,
                    Some(value),
                    (None, None),
                )?;
            }
        }
        for (i, c) in list(r, "LinkSkillList").iter().enumerate() {
            if number(c, "LinkSkillid") != 0 {
                for field in ["LinkSkillid", "Prob"] {
                    w.condition(
                        skill,
                        None,
                        "activation",
                        number(r, "ID"),
                        "Link",
                        None,
                        None,
                        field,
                        i,
                        Some(&number(c, field).to_string()),
                        (None, None),
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn classes(db: &Connection, probe: &skills::SkillProbe, retained: &BTreeSet<i64>) -> Result<()> {
    let mut owners: BTreeMap<i64, BTreeSet<String>> = BTreeMap::new();
    for r in rows(probe, "SkillList") {
        if let Some(class) = code(&r["ClassType"]) {
            for id in list(r, "SkillIdList").iter().filter_map(Value::as_i64) {
                owners.entry(id).or_default().insert(class.into());
            }
        }
    }
    for r in rows(probe, "SkillAcquireData") {
        if r["bSkillListUI"].as_bool() == Some(true)
            && let Some(class) = code(&r["ClassType"])
        {
            owners
                .entry(number(r, "SkillId"))
                .or_default()
                .insert(class.into());
        }
    }
    let direct: BTreeSet<(i64, String)> = owners
        .iter()
        .flat_map(|(id, classes)| classes.iter().map(move |class| (*id, class.clone())))
        .collect();
    let mut edges = Vec::new();
    for r in rows(probe, "SpecializedSkill") {
        edges.push((number(r, "ParentSkillId"), number(r, "ChildSkillId")));
    }
    let charges = indexed(probe, "SkillCharge");
    for r in rows(probe, "Skill") {
        if let Some(charge) = charges.get(&number(r, "ChargeId")) {
            for c in list(charge, "SkillChargeDataList") {
                edges.push((number(r, "ID"), number(c, "ChargeSkillId")));
            }
        }
    }
    loop {
        let mut changed = false;
        for (parent, child) in &edges {
            if *parent <= 0 || *child <= 0 {
                continue;
            }
            if let Some(classes) = owners.get(parent).cloned() {
                for class in classes {
                    changed |= owners.entry(*child).or_default().insert(class);
                }
            }
        }
        if !changed {
            break;
        }
    }
    for (skill, classes) in owners {
        if !retained.contains(&skill) {
            continue;
        }
        for class in classes {
            db.execute(
                "INSERT INTO skill_class_permissions VALUES(?1,?2,?3)",
                params![
                    skill,
                    reference(db, "skill_classes", "code", Some(&class))?,
                    direct.contains(&(skill, class.clone()))
                ],
            )?;
        }
    }
    Ok(())
}
