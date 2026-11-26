//! #83: source definitions, positions and candidate parameter roles.
//! Neither ownership, an activation graph nor formulas follow from a grid.
use super::catalogue_projection::{Origin, Projection, list, rows};
use super::*;

pub(super) const SOURCES: &[&str] = &[
    "DaevanionBoard",
    "DaevanionNode",
    "DaevanionMaxCapacity",
    "Exp",
];
pub(super) const ROOTS: &[(&str, &str, &str)] = &[
    ("DaevanionBoard", "daevanion_boards", "daevanion_boards"),
    ("DaevanionNode", "daevanion_nodes", "daevanion_nodes"),
];
fn origin<'a>(family: &'a str, id: i64, source: &'a str, row: usize) -> Origin<'a> {
    Origin {
        family,
        id,
        source,
        row,
    }
}
fn source(p: &Projection<'_>, o: &Origin<'_>, row: &Value) -> Result<()> {
    p.source(o.family, o.id, o.source, o.row)?;
    for (field, key) in p.texts(o, row)? {
        p.db.execute(
            "INSERT INTO catalogue_text_values VALUES(?1,?2,?3,?4,?5)",
            params![
                o.family,
                o.id,
                field,
                text(p.en, Some(&key)),
                text(p.loc, Some(&key))
            ],
        )?;
    }
    Ok(())
}
fn point(p: &Projection<'_>, value: &Value) -> Result<Option<i64>> {
    string(value)
        .map(|code| dictionary(p.db, "daevanion_point_types", code))
        .transpose()
}
fn named(
    p: &Projection<'_>,
    o: &Origin<'_>,
    row: &Value,
    field: &str,
) -> Result<(Option<String>, Option<String>)> {
    p.named_text(o, field, string(&row[field]["Key"]))
}
fn parameter<'a>(row: &'a Value, field: &str) -> Result<Option<&'a str>> {
    match row.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        _ => Err(invalid(format!("non-text Daevanion parameter {field}")).into()),
    }
}
fn integer(value: Option<&str>) -> Option<i64> {
    value.and_then(|s| s.parse().ok())
}

pub(super) fn node_image(
    db: &Connection,
    id: i64,
    reference: Option<&str>,
    images: &projection::ImageIndex,
) -> Result<Option<i64>> {
    // Node.Image is an exact UI image-table key. Reuse its extracted sprite
    // without treating it as a texture basename or selecting a similar key.
    let sprite:Option<i64>=db.query_row("SELECT s.icon_id FROM ui_sprite_aliases a JOIN ui_sprites s ON s.id=a.sprite_id WHERE a.table_name='UI_ImageTable_Composite' AND a.row_key=?1",[reference],|r|r.get(0)).optional()?;
    if let Some(icon) = sprite {
        db.execute(
            "INSERT INTO catalogue_entity_icons VALUES('daevanion_nodes',?1,?2)",
            params![id, icon],
        )?;
        return Ok(Some(icon));
    }
    projection::attach_icon(db, "daevanion_nodes", id, reference, images)
}

pub(super) fn write(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    en: &Localization,
    loc: &Localization,
    build: i64,
    images: Option<&Path>,
) -> Result<()> {
    let mut p = Projection::new(db, batch, en, loc, build);
    let images = projection::image_index(images)?;
    let mut capacity_types = BTreeSet::new();
    for (pos, row) in rows(batch, "DaevanionMaxCapacity").iter().enumerate() {
        let code = string(&row["DaevanionPointType"])
            .ok_or_else(|| invalid("missing Daevanion point type"))?;
        if !capacity_types.insert(code) {
            return Err(invalid(format!("duplicate Daevanion capacity for {code}")).into());
        }
        let id = point(&p, &row["DaevanionPointType"])?
            .ok_or_else(|| invalid("missing Daevanion point type"))?;
        let o = origin("daevanion_point_types", id, "DaevanionMaxCapacity", pos);
        source(&p, &o, row)?;
        db.execute("UPDATE daevanion_point_types SET source_name=?1,max_value_raw=?2,currency_id=?3 WHERE id=?4",params![string(&row["DaevanionPointName"]),number(row,"MaxValue"),p.code(&row["CurrencyCategory"])?,id])?;
        p.codes(&o, row, &["DaevanionPointType", "CurrencyCategory"])?;
    }
    for (pos, row) in rows(batch, "Exp").iter().enumerate() {
        let level = number(row, "Level").ok_or_else(|| invalid("missing Exp.Level"))?;
        let o = origin("daevanion_level_points", level, "Exp", pos);
        source(&p, &o, row)?;
        for (i, entry) in list(&row["DaevanionPointMap"], "Entries")
            .iter()
            .enumerate()
        {
            p.link(
                &o,
                &format!("DaevanionPointMap/Entries/{i}/Key"),
                &entry["Key"],
                ("DaevanionMaxCapacity", "DaevanionPointType"),
                false,
                false,
            )?;
            db.execute(
                "INSERT INTO daevanion_level_points VALUES(?1,?2,?3,?4)",
                params![
                    level,
                    i as i64,
                    point(&p, &entry["Key"])?,
                    entry["Value"].as_i64()
                ],
            )?;
        }
    }
    for (pos, row) in rows(batch, "DaevanionBoard").iter().enumerate() {
        let id = catalogue_projection::id(row)?;
        let o = origin("daevanion_boards", id, "DaevanionBoard", pos);
        source(&p, &o, row)?;
        let (en, loc) = named(&p, &o, row, "Title")?;
        let (te, tl) = named(&p, &o, row, "BuffTitle")?;
        let (de, dl) = named(&p, &o, row, "BuffDesc")?;
        let class = string(&row["Class"])
            .and_then(|s| s.strip_prefix("ECharacterClass::"))
            .map(|s| dictionary(db, "skill_classes", s))
            .transpose()?;
        p.link(
            &o,
            "CostPointType",
            &row["CostPointType"],
            ("DaevanionMaxCapacity", "DaevanionPointType"),
            false,
            true,
        )?;
        db.execute(
            "INSERT INTO daevanion_boards VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                id,
                build,
                en,
                loc,
                te,
                tl,
                de,
                dl,
                class,
                number(row, "NeedLevel"),
                string(&row["CostPointType"]),
                point(&p, &row["CostPointType"])?,
                p.code(&row["ResetCostCurrencyType"])?,
                number(row, "Order")
            ],
        )?;
        p.codes(
            &o,
            row,
            &["Class", "CostPointType", "ResetCostCurrencyType"],
        )?;
        projection::attach_icon(db, o.family, id, string(&row["Icon"]), &images)?;
        projection::attach_icon(
            db,
            "daevanion_board_buffs",
            id,
            string(&row["BuffIcon"]),
            &images,
        )?;
    }
    for (pos, row) in rows(batch, "DaevanionNode").iter().enumerate() {
        let id = catalogue_projection::id(row)?;
        let o = origin("daevanion_nodes", id, "DaevanionNode", pos);
        source(&p, &o, row)?;
        let board = p.link(
            &o,
            "Board",
            &row["Board"],
            ("DaevanionBoard", "ID"),
            false,
            true,
        )?;
        let board = board.and_then(|i| number(&rows(batch, "DaevanionBoard")[i], "ID"));
        let (en, loc) = named(&p, &o, row, "Title")?;
        let (start_en, start_loc) = if string(&row["Type"]) == Some("EDaevanionNodeType::Start") {
            p.named_text(&o, "start_text", string(&row["Value01"]))?
        } else {
            (None, None)
        };
        db.execute("INSERT INTO daevanion_nodes VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",params![id,build,number(row,"Board"),board,number(row,"Row"),number(row,"Col"),p.code(&row["Type"])?,p.code(&row["Grade"])?,number(row,"NeedLevel"),number(row,"CostDaevanionPoint"),number(row,"ResetGold"),en,loc,start_en,start_loc,p.code(&row["NeedConditionType"])?,parameter(row,"NeedConditionValue")?,row["bAutoLearn"].as_bool()])?;
        p.codes(&o, row, &["Type", "Grade", "NeedConditionType"])?;
        node_image(db, id, string(&row["Image"]), &images)?;
        for i in 1..=10 {
            let field = format!("Value{i:02}");
            if row.get(&field).is_some() {
                db.execute(
                    "INSERT INTO daevanion_node_parameters VALUES(?1,?2,?3)",
                    params![id, i, parameter(row, &field)?],
                )?;
            }
        }
        // The condition enum and opaque value are never turned into a node edge.
        if row.get("NeedConditionType").is_some() || row.get("NeedConditionValue").is_some() {
            p.condition(
                &o,
                "NeedConditionValue",
                &row["NeedConditionType"],
                &row["NeedConditionValue"],
            )?;
        }
        match string(&row["Type"]) {
            Some("EDaevanionNodeType::Stat") => stat_parameters(&mut p, &o, row, board)?,
            Some("EDaevanionNodeType::SkillLevel") => skill_parameter(&mut p, &o, row)?,
            _ => {}
        }
        if let Some(board) = board {
            p.source("daevanion_boards", board, "DaevanionNode", pos)?;
            // Point type/capacity and board text/condition definitions are
            // explicit context; no numeric relation is inferred from IDs.
            let evidence:Vec<(String,i64)>=db.prepare("SELECT source_name,source_row FROM catalogue_entity_sources WHERE family='daevanion_boards' AND source_id=?1 AND source_name!='DaevanionNode'")?.query_map([board],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?;
            for (name, pos) in evidence {
                p.source(o.family, id, &name, usize::try_from(pos)?)?;
            }
            if string(&row["Type"]) == Some("EDaevanionNodeType::Start") {
                p.named_text(
                    &origin("daevanion_boards", board, "DaevanionNode", pos),
                    &format!("node/{id}/start_text"),
                    string(&row["Value01"]),
                )?;
            }
        }
    }
    point_items(&mut p)?;
    let missing: Vec<_> = SOURCES
        .iter()
        .filter(|s| !batch.contains_key(**s))
        .collect();
    let references:i64=db.query_row("SELECT count(*) FROM catalogue_references WHERE family LIKE 'daevanion_%' AND resolution_state NOT IN('resolved','group')",[],|r|r.get(0))?;
    coverage(
        db,
        "daevanion.projection",
        if batch.contains_key("DaevanionBoard") || batch.contains_key("DaevanionNode") {
            "partial"
        } else {
            "not_researched"
        },
        &format!(
            "{} undecoded sources ({missing:?}); {references} unresolved references; blank/start/stat/skill nodes retained separately, with source costs and positions; no character ownership inferred",
            missing.len()
        ),
    )?;
    let mut candidate_fields = BTreeSet::new();
    for name in ["DaevanionBoard", "DaevanionNode"] {
        for row in rows(batch, name) {
            if let Some(fields) = row.as_object() {
                for field in fields.keys() {
                    let lower = field.to_ascii_lowercase();
                    if ["connect", "edge", "neighbor", "neighbour", "link"]
                        .iter()
                        .any(|s| lower.contains(s))
                    {
                        candidate_fields.insert(format!("{name}.{field}"));
                    }
                }
            }
        }
    }
    coverage(
        db,
        "daevanion.connections.tables",
        if !candidate_fields.is_empty() {
            "uninterpreted"
        } else if batch.contains_key("DaevanionBoard") && batch.contains_key("DaevanionNode") {
            "not_found"
        } else {
            "not_researched"
        },
        &format!(
            "uninterpreted edge-field candidates {candidate_fields:?}; no verified edges extracted from inspected DaevanionBoard/DaevanionNode layouts; row/column proximity and condition values do not establish edges"
        ),
    )?;
    coverage(
        db,
        "daevanion.connections.client_logic",
        "not_researched",
        "no verified activation/adjacency algorithm extracted from client executable or Unreal logic; no inferred graph published",
    )?;
    coverage(
        db,
        "daevanion.parameter_roles",
        "uninterpreted",
        "stat/value pairs and SkillLevel Value01 to Skill.ID are candidate associations; Value02 is not assumed to be an additive skill level; numeric values retain original scale",
    )?;
    coverage(
        db,
        "daevanion.level_points",
        "uninterpreted",
        "explicit Exp.DaevanionPointMap values retained, without calling them per-level gains or deriving a formula; removed map entries remain in complete source",
    )?;
    Ok(())
}

fn stat_parameters(
    p: &mut Projection<'_>,
    o: &Origin<'_>,
    row: &Value,
    board: Option<i64>,
) -> Result<()> {
    for i in (1..=9).step_by(2) {
        let field = format!("Value{i:02}");
        let value_field = format!("Value{:02}", i + 1);
        let Some(reference) = parameter(row, &field)?.filter(|s| !s.is_empty() && *s != "None")
        else {
            continue;
        };
        let value = parameter(row, &value_field)?;
        let key = format!("String_StatName_{reference}_body");
        let (en, loc) = p.named_text(o, &format!("stat/{i}"), Some(&key))?;
        let code = format!("EStat::{reference}");
        let existing: Option<i64> =
            p.db.query_row("SELECT id FROM stats WHERE code=?1", [&code], |r| r.get(0))
                .optional()?;
        let stat = if existing.is_some() || en.is_some() || loc.is_some() {
            Some(dictionary(p.db, "stats", &code)?)
        } else {
            None
        };
        if let Some(stat) = stat {
            p.db.execute(
                "UPDATE stats SET name_en=?1,name_localized=?2 WHERE id=?3",
                params![en, loc, stat],
            )?;
        }
        p.db.execute(
            "INSERT INTO daevanion_stat_parameters VALUES(?1,?2,?3,?4,?5,?6,'candidate')",
            params![o.id, i, reference, stat, value, integer(value)],
        )?;
        if let Some(board) = board {
            p.named_text(
                &origin("daevanion_boards", board, o.source, o.row),
                &format!("node/{}/stat/{i}", o.id),
                Some(&key),
            )?;
        }
    }
    Ok(())
}
fn skill_parameter(p: &mut Projection<'_>, o: &Origin<'_>, row: &Value) -> Result<()> {
    let Some(reference) = parameter(row, "Value01")?.filter(|s| !s.is_empty()) else {
        return Ok(());
    };
    let numeric = integer(Some(reference));
    let value = numeric
        .map(Value::from)
        .unwrap_or_else(|| Value::from(reference));
    let skill = p.link(o, "Value01", &value, ("Skill", "ID"), true, false)?;
    let skill = skill.and_then(|i| number(&rows(p.batch, "Skill")[i], "ID"));
    let amount = parameter(row, "Value02")?;
    p.db.execute(
        "INSERT INTO daevanion_skill_parameters VALUES(?1,?2,?3,?4,?5,?6,'candidate')",
        params![o.id, reference, numeric, skill, amount, integer(amount)],
    )?;
    Ok(())
}
fn point_items(p: &mut Projection<'_>) -> Result<()> {
    for (pos, row) in rows(p.batch, "Item").iter().enumerate() {
        let v = &row["$variant"];
        if !matches!(
            string(&v["UseEffect"]),
            Some("EItemUseEffect::GetDaevanionPoint" | "EItemUseEffect::Fix_DaevanionPoint")
        ) {
            continue;
        }
        let id = catalogue_projection::id(row)?;
        let o = origin("daevanion_point_items", id, "Item", pos);
        let values = list(v, "UseEffectValues");
        let reference = values.first().and_then(string);
        let found = p.link(
            &o,
            "$variant/UseEffectValues/0",
            &reference.map(Value::from).unwrap_or_default(),
            ("DaevanionMaxCapacity", "DaevanionPointName"),
            true,
            false,
        )?;
        let target = found
            .map(|i| {
                point(
                    p,
                    &rows(p.batch, "DaevanionMaxCapacity")[i]["DaevanionPointType"],
                )
            })
            .transpose()?
            .flatten();
        let amount = values.get(1).and_then(Value::as_str);
        p.db.execute(
            "INSERT INTO daevanion_point_items VALUES(?1,?2,?3,?4,?5,?6,'candidate')",
            params![
                id,
                p.code(&v["UseEffect"])?,
                reference,
                target,
                amount,
                integer(amount)
            ],
        )?;
    }
    Ok(())
}

pub(super) fn fingerprints(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    en: &Localization,
    loc: &Localization,
) -> Result<catalogue_projection::Fingerprints> {
    let mut result = catalogue_projection::fingerprints(
        db,
        batch,
        en,
        loc,
        &["daevanion_boards", "daevanion_nodes"],
    )?;
    // Indirect localization keys (stat/start parameters) are not source FText.
    type Texts = BTreeMap<(String, i64), Vec<(String, String)>>;
    let mut texts = Texts::new();
    for row in db.prepare("SELECT family,source_id,field,string_key FROM catalogue_texts WHERE family IN('daevanion_boards','daevanion_nodes') AND string_key IS NOT NULL ORDER BY family,source_id,field")?.query_map([],|r|Ok(((r.get::<_,String>(0)?,r.get::<_,i64>(1)?),(r.get::<_,String>(2)?,r.get::<_,String>(3)?))))? {
        let (key,field)=row?;texts.entry(key).or_default().push(field);
    }
    for (key, fields) in texts {
        if let Some((base, localized)) = result.get_mut(&key) {
            let english: Vec<_> = fields
                .iter()
                .map(|(field, key)| (field, key, text(en, Some(key))))
                .collect();
            let translated: Vec<_> = fields
                .iter()
                .map(|(field, key)| (field, key, text(loc, Some(key))))
                .collect();
            *base = hash(&serde_json::to_vec(&(&base, english))?);
            *localized = hash(&serde_json::to_vec(&(&localized, translated))?);
        }
    }
    // Only the Daevanion map participates; unrelated Exp fields must not revise boards.
    let mut point_values: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in rows(batch, "Exp") {
        for entry in list(&row["DaevanionPointMap"], "Entries") {
            if let Some(code) = string(&entry["Key"]) {
                point_values
                    .entry(code.into())
                    .or_default()
                    .push(serde_json::json!([row["Level"], "entry", entry["Value"]]));
            }
        }
        for key in list(&row["DaevanionPointMap"], "Removed") {
            if let Some(code) = string(key) {
                point_values
                    .entry(code.into())
                    .or_default()
                    .push(serde_json::json!([row["Level"], "removed", key]));
            }
        }
    }
    let mut hashes = BTreeMap::new();
    for (code, mut values) in point_values {
        values.sort_by_cached_key(Value::to_string);
        hashes.insert(code, hash(&serde_json::to_vec(&values)?));
    }
    let mut boards = BTreeMap::new();
    for row in rows(batch, "DaevanionBoard") {
        if let Some(code) = string(&row["CostPointType"]) {
            boards.insert(catalogue_projection::id(row)?, code);
        }
    }
    for ((family, id), (base, _)) in &mut result {
        let board = if family == "daevanion_boards" {
            Some(*id)
        } else {
            db.query_row(
                "SELECT board_id FROM daevanion_nodes WHERE id=?1",
                [id],
                |r| r.get::<_, Option<i64>>(0),
            )?
        };
        let context = board
            .and_then(|b| boards.get(&b))
            .and_then(|code| hashes.get(*code));
        *base = hash(&serde_json::to_vec(&(&base, context))?);
    }
    Ok(result)
}
