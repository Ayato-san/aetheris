//! #82: pets, wings and appearances; definitions never imply player ownership.
use super::catalogue_projection::{Origin, Projection, list, rows};
use super::*;

pub(super) const SOURCES: &[&str] = &[
    "VehicleList",
    "VehicleCreatureSlot",
    "VehicleCreatureLevel",
    "VehicleCreatureOption",
    "VehicleStat",
    "VehicleLevel",
    "VehicleCollection",
    "Wing",
    "WingEnchant",
    "WingEnchantEffect",
    "Skin",
    "SkinSet",
    "SkinCollection",
    "SkinCombine",
    "SkinCombineCost",
    "SkinGacha",
    "SkinGachaGroup",
    "SkinMaterial",
    "SkinPattern",
    "SkinMotion",
];
// Only source IDs are stable roots; groups and positional rules remain children.
pub(super) const ROOTS: &[(&str, &str, &str)] = &[
    ("VehicleList", "pets", "pets"),
    (
        "VehicleCreatureLevel",
        "pet_family_levels",
        "pet_family_levels",
    ),
    ("VehicleCreatureOption", "pet_options", "pet_options"),
    ("VehicleStat", "pet_stat_levels", "pet_stat_levels"),
    ("VehicleCollection", "pet_collections", "pet_collections"),
    ("Wing", "wings", "wings"),
    ("Skin", "skins", "skins"),
    ("SkinSet", "skin_sets", "skin_sets"),
    ("SkinGacha", "skin_gacha_results", "skin_gacha_results"),
];
fn origin<'a>(family: &'a str, id: i64, source: &'a str, row: usize) -> Origin<'a> {
    Origin {
        family,
        id,
        source,
        row,
    }
}
fn id(row: &Value) -> Result<i64> {
    catalogue_projection::id(row)
}
fn required<'a>(row: &'a Value, field: &str) -> Result<&'a str> {
    string(&row[field]).ok_or_else(|| invalid(format!("missing #82 key {field}")).into())
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
fn named(
    p: &Projection<'_>,
    o: &Origin<'_>,
    row: &Value,
    field: &str,
) -> Result<(Option<String>, Option<String>)> {
    p.named_text(o, field, string(&row[field]["Key"]))
}
fn date(p: &Projection<'_>, o: &Origin<'_>, row: &Value) -> Result<()> {
    if let Some(ticks) = number(&row["ShowDate"], "Ticks") {
        p.db.execute(
            "INSERT INTO catalogue_periods VALUES(?1,?2,'show',?3,NULL)",
            params![o.family, o.id, ticks],
        )?;
    }
    Ok(())
}
fn stat(p: &Projection<'_>, code: &str) -> Result<i64> {
    let id = dictionary(p.db, "stats", code)?;
    let key = code
        .strip_prefix("EStat::")
        .map(|s| format!("String_StatName_{s}_body"));
    p.db.execute(
        "UPDATE stats SET name_en=?1,name_localized=?2 WHERE id=?3",
        params![text(p.en, key.as_deref()), text(p.loc, key.as_deref()), id],
    )?;
    Ok(id)
}
fn stats(
    p: &Projection<'_>,
    o: &Origin<'_>,
    role: &str,
    values: &[Value],
    key: &str,
    value: &str,
) -> Result<()> {
    for (pos, row) in values.iter().enumerate() {
        p.db.execute(
            "INSERT INTO appearance_stats VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                o.family,
                o.id,
                role,
                pos as i64,
                stat(p, required(row, key)?)?,
                number(row, value)
            ],
        )?;
    }
    Ok(())
}
fn target_id(p: &Projection<'_>, table: &str, row: Option<usize>) -> Option<i64> {
    row.and_then(|pos| number(&rows(p.batch, table)[pos], "ID"))
}
fn group(
    p: &mut Projection<'_>,
    o: &Origin<'_>,
    row: &Value,
    field: &str,
    target: (&str, &str),
    dictionary_table: &str,
) -> Result<Option<i64>> {
    let found = p.group(o, field, &row[field], target, false, true)?;
    if found.is_empty() {
        return Ok(None);
    }
    Ok(Some(dictionary(
        p.db,
        dictionary_table,
        required(row, field)?,
    )?))
}
fn item_link(
    p: &mut Projection<'_>,
    o: &Origin<'_>,
    path: &str,
    reference: &Value,
    role: &str,
    quantity: Option<i64>,
) -> Result<Option<i64>> {
    let Some(reference) = string(reference) else {
        return Ok(None);
    };
    p.link(
        o,
        path,
        &Value::from(reference),
        ("Item", "Name"),
        false,
        false,
    )?;
    let item = p.item(Some(reference));
    p.db.execute(
        "INSERT INTO catalogue_item_links VALUES(?1,?2,?3,0,?4,?5,?6,?7,?8,?9,NULL,NULL)",
        params![
            o.family,
            o.id,
            role,
            o.source,
            o.row as i64,
            path,
            reference,
            item,
            quantity
        ],
    )?;
    Ok(item)
}
fn family(p: &Projection<'_>, value: &Value) -> Result<Option<i64>> {
    let Some(code) = string(value) else {
        return Ok(None);
    };
    let id = dictionary(p.db, "pet_families", code)?;
    let label = p.code(value)?;
    p.db.execute("UPDATE pet_families SET name_en=(SELECT name_en FROM catalogue_codes WHERE id=?1),name_localized=(SELECT name_localized FROM catalogue_codes WHERE id=?1) WHERE id=?2",params![label,id])?;
    Ok(Some(id))
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
    pets(&mut p, &images)?;
    wings(&mut p, &images)?;
    skins(&mut p, &images)?;
    acquisition(&mut p)?;
    skin_rules(&mut p)?;
    for (subject, names) in [
        ("pets", &SOURCES[..7]),
        ("wings", &SOURCES[7..10]),
        ("skins", &SOURCES[10..]),
    ] {
        let absent: Vec<_> = names.iter().filter(|s| !batch.contains_key(**s)).collect();
        let mut unresolved = 0i64;
        for name in names {
            unresolved+=db.query_row("SELECT count(*) FROM catalogue_references WHERE source_name=?1 AND resolution_state NOT IN('resolved','group')",[name],|r|r.get::<_,i64>(0))?;
        }
        coverage(
            db,
            &format!("{subject}.projection"),
            if names.iter().any(|s| batch.contains_key(*s)) {
                "partial"
            } else {
                "not_researched"
            },
            &format!(
                "{} undecoded sources ({absent:?}); {unresolved} unresolved source references; source IDs, typed definitions, relations and diagnostics retained; ownership, eligibility and probability scales not inferred",
                absent.len()
            ),
        )?;
    }
    let absent: i64 = db.query_row(
        "SELECT count(*) FROM wings WHERE enchant_group_id IS NULL",
        [],
        |r| r.get(0),
    )?;
    coverage(
        db,
        "wings.enchant_association",
        "partial",
        &format!(
            "{absent} wings without a resolved explicit EnchantGroup; costs retained separately, never inferred from grade or EffectGroup"
        ),
    )?;
    coverage(
        db,
        "pets.family_labels",
        "partial",
        "exact ECreatureType enum labels and separate BuffTitle/BuffDesc texts; never extract a family name from a bonus sentence",
    )?;
    coverage(
        db,
        "appearances.probabilities",
        "uninterpreted",
        "source weights/scales retained; no inferred success/drop probability, weapon-index mapping or live gacha availability",
    )?;
    coverage(
        db,
        "skins.decorations",
        "uninterpreted",
        "SkinMaterial, SkinPattern and SkinMotion retained as complete sources; prices, resource layers, motions and customization application rules have no business projection",
    )?;
    Ok(())
}

fn pets(p: &mut Projection<'_>, images: &projection::ImageIndex) -> Result<()> {
    for (pos, row) in rows(p.batch, "VehicleCreatureOption").iter().enumerate() {
        let id = id(row)?;
        let o = origin("pet_options", id, "VehicleCreatureOption", pos);
        source(p, &o, row)?;
        let group = dictionary(p.db, "pet_option_groups", required(row, "GroupName")?)?;
        p.db.execute(
            "INSERT INTO pet_options VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                id,
                p.build,
                group,
                p.code(&row["OptionGrade"])?,
                stat(p, required(row, "StatType")?)?,
                number(row, "StatValueMin"),
                number(row, "StatValueMax"),
                number(row, "Prob")
            ],
        )?;
        p.codes(&o, row, &["OptionGrade"])?;
    }
    for (pos, row) in rows(p.batch, "VehicleStat").iter().enumerate() {
        let id = id(row)?;
        let o = origin("pet_stat_levels", id, "VehicleStat", pos);
        source(p, &o, row)?;
        let group = dictionary(p.db, "pet_stat_groups", required(row, "Group")?)?;
        p.db.execute(
            "INSERT INTO pet_stat_levels VALUES(?1,?2,?3,?4)",
            params![id, p.build, group, number(row, "Level")],
        )?;
        for (i, s) in list(row, "StatList").iter().enumerate() {
            p.db.execute(
                "INSERT INTO pet_stat_values VALUES(?1,?2,?3,?4)",
                params![
                    id,
                    i as i64,
                    stat(p, required(s, "StatType")?)?,
                    number(s, "StatValue")
                ],
            )?;
        }
    }
    for (pos, row) in rows(p.batch, "VehicleCreatureSlot").iter().enumerate() {
        let id = family(p, &row["CreatureType"])?.ok_or_else(|| invalid("missing pet family"))?;
        let o = origin("pet_families", id, "VehicleCreatureSlot", pos);
        source(p, &o, row)?;
        let (title_en, title_loc) = named(p, &o, row, "BuffTitle")?;
        let (desc_en, desc_loc) = named(p, &o, row, "BuffDesc")?;
        p.db.execute("UPDATE pet_families SET buff_title_en=?1,buff_title_localized=?2,buff_description_en=?3,buff_description_localized=?4,currency_id=?5,currency_count_raw=?6,soul_type_1_id=?7,soul_type_2_id=?8,soul_count_raw=?9 WHERE id=?10",params![title_en,title_loc,desc_en,desc_loc,p.code(&row["CostCurrencyType"])?,number(row,"GoldCount"),p.code(&row["CostSoulCrystalType01"])?,p.code(&row["CostSoulCrystalType02"])?,number(row,"SoulCrystalCount"),id])?;
        p.codes(
            &o,
            row,
            &[
                "CreatureType",
                "CostCurrencyType",
                "CostSoulCrystalType01",
                "CostSoulCrystalType02",
            ],
        )?;
        projection::attach_icon(p.db, o.family, id, string(&row["BuffIcon"]), images)?;
        let levels = list(row, "SlotOpenLevel");
        let groups = list(row, "SlotOptionGroup");
        for i in 0..levels.len().max(groups.len()) {
            let reference = groups.get(i).unwrap_or(&Value::Null);
            let found = p.group(
                &o,
                &format!("SlotOptionGroup/{i}"),
                reference,
                ("VehicleCreatureOption", "GroupName"),
                false,
                true,
            )?;
            let group = if found.is_empty() {
                None
            } else {
                string(reference)
                    .map(|s| dictionary(p.db, "pet_option_groups", s))
                    .transpose()?
            };
            p.db.execute(
                "INSERT INTO pet_family_slots VALUES(?1,?2,?3,?4)",
                params![id, i as i64, levels.get(i).and_then(Value::as_i64), group],
            )?;
        }
        let souls = list(row, "LockCost");
        let gold = list(row, "LockCostGold");
        for i in 0..souls.len().max(gold.len()) {
            p.db.execute(
                "INSERT INTO pet_family_lock_costs VALUES(?1,?2,?3,?4)",
                params![
                    id,
                    i as i64,
                    souls.get(i).and_then(Value::as_i64),
                    gold.get(i).and_then(Value::as_i64)
                ],
            )?;
        }
    }
    for (pos, row) in rows(p.batch, "VehicleCreatureLevel").iter().enumerate() {
        let id = id(row)?;
        let o = origin("pet_family_levels", id, "VehicleCreatureLevel", pos);
        source(p, &o, row)?;
        let family = family(p, &row["CreatureType"])?;
        p.db.execute(
            "INSERT INTO pet_family_levels VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                id,
                p.build,
                family,
                number(row, "Level"),
                number(row, "NeedExp"),
                number(row, "GetExp")
            ],
        )?;
        p.codes(&o, row, &["CreatureType"])?;
        if let Some(obj) = row.as_object() {
            for (field, value) in obj {
                if let Some(grade) = field.strip_prefix("Prob_") {
                    p.db.execute(
                        "INSERT INTO pet_grade_weights VALUES(?1,?2,?3)",
                        params![id, grade, value.as_i64()],
                    )?;
                }
            }
        }
    }
    for (pos, row) in rows(p.batch, "VehicleLevel").iter().enumerate() {
        let level = number(row, "Level").ok_or_else(|| invalid("missing pet level"))?;
        source(p, &origin("pet_levels", level, "VehicleLevel", pos), row)?;
        p.db.execute(
            "INSERT INTO pet_levels VALUES(?1,?2,?3)",
            params![
                level,
                number(row, "NeedSoulCount"),
                number(row, "GetCreatureExp")
            ],
        )?;
    }
    for (pos, row) in rows(p.batch, "VehicleList").iter().enumerate() {
        let id = id(row)?;
        let o = origin("pets", id, "VehicleList", pos);
        source(p, &o, row)?;
        let (en, loc) = named(p, &o, row, "Desc")?;
        let family = family(p, &row["CreatureType"])?;
        p.group(
            &o,
            "CreatureType",
            &row["CreatureType"],
            ("VehicleCreatureSlot", "CreatureType"),
            false,
            true,
        )?;
        p.group(
            &o,
            "CreatureType/@levels",
            &row["CreatureType"],
            ("VehicleCreatureLevel", "CreatureType"),
            false,
            true,
        )?;
        if let Some(family) = family {
            let evidence:Vec<(String,i64)>=p.db.prepare("SELECT source_name,source_row FROM catalogue_entity_sources WHERE family='pet_families' AND source_id=?1")?.query_map([family],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?;
            for (name, pos) in evidence {
                p.source(o.family, id, &name, usize::try_from(pos)?)?;
            }
        }
        let group = group(
            p,
            &o,
            row,
            "StatGroupName",
            ("VehicleStat", "Group"),
            "pet_stat_groups",
        )?;
        let item = item_link(
            p,
            &o,
            "VehicleItemName",
            &row["VehicleItemName"],
            "granting_item",
            None,
        )?;
        let soul = item_link(
            p,
            &o,
            "SoulItemName",
            &row["SoulItemName"],
            "soul_item",
            None,
        )?;
        // Global pet levels explicitly define soul costs for all pets.
        for i in 0..rows(p.batch, "VehicleLevel").len() {
            p.source(o.family, id, "VehicleLevel", i)?;
        }
        p.db.execute("INSERT INTO pets VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",params![id,required(row,"Name")?,p.build,en,loc,family,string(&row["VehicleItemName"]),item,string(&row["SoulItemName"]),soul,string(&row["StatGroupName"]),group,number(row,"NeedSoul"),p.code(&row["SoulCrystalType"])?,number(row,"PolymorphId"),string(&row["ResourceKey"]),number(row,"Order"),row["bIsOpen"].as_bool(),row["bPolymorphPlan"].as_bool()])?;
        p.codes(&o, row, &["CreatureType", "SoulCrystalType"])?;
        date(p, &o, row)?;
        stats(
            p,
            &o,
            "equipped",
            list(row, "EquipStatList"),
            "Stat",
            "Value",
        )?;
    }
    for (pos, row) in rows(p.batch, "VehicleCollection").iter().enumerate() {
        let id = id(row)?;
        let o = origin("pet_collections", id, "VehicleCollection", pos);
        source(p, &o, row)?;
        let item = item_link(
            p,
            &o,
            "RewardItem",
            &row["RewardItem"],
            "reward",
            number(row, "RewardItemCount"),
        )?;
        let pet = p.link(
            &o,
            "RewardVehicle",
            &row["RewardVehicle"],
            ("VehicleList", "Name"),
            true,
            false,
        )?;
        p.db.execute(
            "INSERT INTO pet_collections VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                id,
                p.build,
                number(row, "CollectNum"),
                string(&row["RewardItem"]),
                item,
                number(row, "RewardItemCount"),
                string(&row["RewardVehicle"]),
                target_id(p, "VehicleList", pet)
            ],
        )?;
    }
    Ok(())
}

fn wings(p: &mut Projection<'_>, images: &projection::ImageIndex) -> Result<()> {
    for (pos, row) in rows(p.batch, "WingEnchantEffect").iter().enumerate() {
        let group = dictionary(p.db, "wing_effect_groups", required(row, "EffectGroup")?)?;
        let level = number(row, "Level").ok_or_else(|| invalid("missing wing effect level"))?;
        source(
            p,
            &origin("wing_effect_rules", pos as i64, "WingEnchantEffect", pos),
            row,
        )?;
        p.db.execute(
            "INSERT INTO wing_effect_levels VALUES(?1,?2)",
            params![group, level],
        )?;
        for (i, s) in list(&row["Stats"], "Entries").iter().enumerate() {
            p.db.execute(
                "INSERT INTO wing_effect_stats VALUES(?1,?2,?3,?4,?5)",
                params![
                    group,
                    level,
                    i as i64,
                    stat(p, required(s, "Key")?)?,
                    s["Value"].as_i64()
                ],
            )?;
        }
    }
    for (pos, row) in rows(p.batch, "WingEnchant").iter().enumerate() {
        let group = dictionary(p.db, "wing_enchant_groups", required(row, "EnchantGroup")?)?;
        let level = number(row, "Level").ok_or_else(|| invalid("missing wing enchant level"))?;
        let o = origin("wing_enchant_rules", pos as i64, "WingEnchant", pos);
        source(p, &o, row)?;
        p.db.execute(
            "INSERT INTO wing_enchant_levels VALUES(?1,?2,?3,?4,?5)",
            params![
                group,
                level,
                number(row, "BasicProb"),
                number(row, "AddProb"),
                number(row, "MaxProb")
            ],
        )?;
        for (i, cost) in list(row, "CostItemList").iter().enumerate() {
            let item = item_link(
                p,
                &o,
                &format!("CostItemList/{i}/CostItemName"),
                &cost["CostItemName"],
                "cost",
                number(cost, "CostItemCount"),
            )?;
            p.db.execute(
                "INSERT INTO wing_enchant_costs VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![
                    group,
                    level,
                    i as i64,
                    string(&cost["CostItemName"]),
                    item,
                    number(cost, "CostItemCount"),
                    number(cost, "AddProbItemCount")
                ],
            )?;
        }
    }
    for (pos, row) in rows(p.batch, "Wing").iter().enumerate() {
        let id = id(row)?;
        let o = origin("wings", id, "Wing", pos);
        source(p, &o, row)?;
        let (en, loc) = named(p, &o, row, "Desc")?;
        let effect = group(
            p,
            &o,
            row,
            "EffectGroup",
            ("WingEnchantEffect", "EffectGroup"),
            "wing_effect_groups",
        )?;
        let enchant = group(
            p,
            &o,
            row,
            "EnchantGroup",
            ("WingEnchant", "EnchantGroup"),
            "wing_enchant_groups",
        )?;
        p.db.execute(
            "INSERT INTO wings VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                id,
                required(row, "Name")?,
                p.build,
                en,
                loc,
                p.code(&row["Grade"])?,
                p.race(row, "PcRace")?,
                string(&row["CostumeKey"]),
                string(&row["EffectGroup"]),
                effect,
                string(&row["EnchantGroup"]),
                enchant,
                number(row, "Order"),
                row["bShow"].as_bool()
            ],
        )?;
        p.codes(&o, row, &["Grade", "PcRace"])?;
        date(p, &o, row)?;
        projection::attach_icon(p.db, o.family, id, string(&row["IconImg"]), images)?;
        stats(
            p,
            &o,
            "equipped",
            list(row, "EquipStatList"),
            "Stat",
            "Value",
        )?;
    }
    Ok(())
}

fn skins(p: &mut Projection<'_>, images: &projection::ImageIndex) -> Result<()> {
    for (pos, row) in rows(p.batch, "Skin").iter().enumerate() {
        let id = id(row)?;
        let o = origin("skins", id, "Skin", pos);
        source(p, &o, row)?;
        let abnormal = p.link(
            &o,
            "SkillAbnormal",
            &row["SkillAbnormal"],
            ("SkillAbnormal", "ID"),
            false,
            false,
        )?;
        p.db.execute(
            "INSERT INTO skins VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![
                id,
                required(row, "Name")?,
                p.build,
                p.code(&row["SkinCategory"])?,
                p.code(&row["Grade"])?,
                p.code(&row["Style"])?,
                number(row, "SkillAbnormal"),
                target_id(p, "SkillAbnormal", abnormal),
                string(&row["ReleaseDate"]),
                number(row, "Order"),
                row["bShow"].as_bool(),
                row["bCanDeco"].as_bool(),
                row["bIsMultiOwn"].as_bool()
            ],
        )?;
        p.codes(&o, row, &["Grade", "SkinCategory", "Style"])?;
        date(p, &o, row)?;
        stats(
            p,
            &o,
            "source_stat_list",
            list(row, "StatList"),
            "Type",
            "Value",
        )?;
        for (race, race_id) in [("Light", 2), ("Dark", 3)] {
            let (en, loc) = named(p, &o, row, &format!("Desc{race}"))?;
            for (sex, sex_id, res) in [("Male", 1, "M"), ("Female", 2, "F")] {
                let reference = string(&row[format!("Icon{race}{sex}")]);
                let icon = projection::attach_icon(
                    p.db,
                    &format!("skins_{race}_{sex}"),
                    id,
                    reference,
                    images,
                )?;
                p.db.execute(
                    "INSERT INTO skin_variants VALUES(?1,?2,?3,?4,?5,?6,?7)",
                    params![
                        id,
                        race_id,
                        sex_id,
                        en,
                        loc,
                        icon,
                        string(&row[format!("CostumeResourceKey_{res}")])
                    ],
                )?;
            }
        }
    }
    for (pos, row) in rows(p.batch, "SkinSet").iter().enumerate() {
        let id = id(row)?;
        let o = origin("skin_sets", id, "SkinSet", pos);
        source(p, &o, row)?;
        let (le, ll) = named(p, &o, row, "DescLight")?;
        let (de, dl) = named(p, &o, row, "DescDark")?;
        let (desc, descl) = named(p, &o, row, "DescLong")?;
        p.db.execute(
            "INSERT INTO skin_sets VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![
                id,
                required(row, "Name")?,
                p.build,
                le,
                ll,
                de,
                dl,
                desc,
                descl,
                p.code(&row["Grade"])?,
                p.code(&row["SetCategory"])?,
                string(&row["ReleaseDate"]),
                number(row, "Order"),
                row["bShow"].as_bool(),
                string(&row["PcAppearanceOverride"]),
                row["bPcAppearanceOverrideChange"].as_bool()
            ],
        )?;
        p.codes(&o, row, &["Grade"])?;
        date(p, &o, row)?;
        for (i, reference) in list(row, "Skins").iter().enumerate() {
            let skin = p.link(
                &o,
                &format!("Skins/{i}"),
                reference,
                ("Skin", "Name"),
                false,
                true,
            )?;
            p.db.execute(
                "INSERT INTO skin_set_members VALUES(?1,?2,?3,?4)",
                params![id, i as i64, string(reference), target_id(p, "Skin", skin)],
            )?;
        }
    }
    Ok(())
}

fn acquisition(p: &mut Projection<'_>) -> Result<()> {
    for (pos, row) in rows(p.batch, "Item").iter().enumerate() {
        let variant = &row["$variant"];
        // Only position 0 encodes the granted definition. Additional values
        // (notably soul counts) are not interpreted as further target IDs.
        let effect = string(&variant["UseEffect"]).unwrap_or("");
        let route = match effect {
            "EItemUseEffect::GetVehicle" => {
                Some(("pets", "VehicleList", "ID", true, "granting_item"))
            }
            "EItemUseEffect::GetVehicleSoul" => {
                Some(("pets", "VehicleList", "ID", true, "soul_item"))
            }
            "EItemUseEffect::GetWing" => Some(("wings", "Wing", "ID", true, "granting_item")),
            "EItemUseEffect::GetSkin" => Some(("skins", "Skin", "Name", false, "granting_item")),
            "EItemUseEffect::GetSkinSet" => {
                Some(("skin_sets", "SkinSet", "Name", false, "granting_item"))
            }
            _ => None,
        };
        if let Some((family, table, field, numeric, role)) = route
            && let Some(reference) = list(variant, "UseEffectValues").first().and_then(string)
        {
            let value = if numeric {
                reference.parse::<i64>().ok().map(Value::from)
            } else {
                Some(Value::from(reference))
            };
            if let Some(value) = value {
                let diagnostic = origin("appearance_grants", id(row)?, "Item", pos);
                let found = p.link(
                    &diagnostic,
                    "$variant/UseEffectValues/0",
                    &value,
                    (table, field),
                    true,
                    false,
                )?;
                if let Some(target) = target_id(p, table, found) {
                    item_link(
                        p,
                        &origin(family, target, "Item", pos),
                        "Name/@grant",
                        &row["Name"],
                        role,
                        None,
                    )?;
                }
            }
        }
        for (i, reference) in list(variant, "ExtractionSkins").iter().enumerate() {
            let diagnostic = origin("appearance_extractions", id(row)?, "Item", pos);
            let found = p.link(
                &diagnostic,
                &format!("$variant/ExtractionSkins/{i}"),
                reference,
                ("Skin", "Name"),
                false,
                false,
            )?;
            if let Some(target) = target_id(p, "Skin", found) {
                item_link(
                    p,
                    &origin("skins", target, "Item", pos),
                    &format!("Name/@extraction/{i}"),
                    &row["Name"],
                    "extraction_item",
                    None,
                )?;
            }
        }
    }
    Ok(())
}

fn skin_rules(p: &mut Projection<'_>) -> Result<()> {
    for (pos, row) in rows(p.batch, "SkinCollection").iter().enumerate() {
        let o = origin("skin_collection_rules", pos as i64, "SkinCollection", pos);
        source(p, &o, row)?;
        for (i, s) in list(row, "RewardStats").iter().enumerate() {
            p.db.execute(
                "INSERT INTO skin_collection_stats VALUES(?1,?2,?3,?4,?5)",
                params![
                    p.code(&row["SkinCategory"])?,
                    number(row, "CollectPoint"),
                    i as i64,
                    stat(p, required(s, "Stat")?)?,
                    number(s, "StatValue")
                ],
            )?;
        }
    }
    for (pos, row) in rows(p.batch, "SkinCombine").iter().enumerate() {
        let o = origin("skin_combinations", pos as i64, "SkinCombine", pos);
        source(p, &o, row)?;
        let mut variants = vec![(0, false, &row["SkinId"])];
        variants.extend(
            list(row, "WeaponSkinIds")
                .iter()
                .enumerate()
                .map(|(i, v)| (i + 1, true, v)),
        );
        for (i, weapon, value) in variants {
            let Some(reference) = value
                .as_i64()
                .or_else(|| number(value, "Value"))
                .filter(|v| *v != 0)
            else {
                continue;
            };
            let path = if weapon {
                format!("WeaponSkinIds/{}", i - 1)
            } else {
                "SkinId".into()
            };
            let target = p.link(
                &o,
                &path,
                &Value::from(reference),
                ("Skin", "ID"),
                false,
                false,
            )?;
            let target = target_id(p, "Skin", target);
            p.db.execute(
                "INSERT INTO skin_combine_results VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    pos as i64,
                    i as i64,
                    weapon,
                    p.code(&row["Grade"])?,
                    p.code(&row["ResultGroup"])?,
                    number(row, "Prob"),
                    reference,
                    target
                ],
            )?;
            if let Some(target) = target {
                p.source("skins", target, "SkinCombine", pos)?;
            }
        }
    }
    for (pos, row) in rows(p.batch, "SkinCombineCost").iter().enumerate() {
        let o = origin("skin_combine_costs", pos as i64, "SkinCombineCost", pos);
        source(p, &o, row)?;
        let item = p.link(
            &o,
            "CostTypeItem",
            &row["CostTypeItem"],
            ("Item", "ID"),
            false,
            false,
        )?;
        p.db.execute(
            "INSERT INTO skin_combine_costs VALUES(?1,?2,?3,?4,?5)",
            params![
                p.code(&row["Grade"])?,
                p.code(&row["CostTypeCurrency"])?,
                number(row, "CostTypeItem"),
                target_id(p, "Item", item),
                number(row, "CostCount")
            ],
        )?;
    }
    for (pos, row) in rows(p.batch, "SkinGacha").iter().enumerate() {
        let id = id(row)?;
        let o = origin("skin_gacha_results", id, "SkinGacha", pos);
        source(p, &o, row)?;
        let target = if string(&row["ResultType"]) == Some("ESkinGachaResultType::Skin") {
            p.link(
                &o,
                "ResultName",
                &row["ResultName"],
                ("Skin", "Name"),
                true,
                false,
            )?
        } else {
            None
        };
        let target = target_id(p, "Skin", target);
        p.db.execute(
            "INSERT INTO skin_gacha_results VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![
                id,
                required(row, "Name")?,
                p.build,
                p.code(&row["ResultType"])?,
                string(&row["ResultName"]),
                target,
                number(row, "Prob")
            ],
        )?;
        if let Some(target) = target {
            p.source("skins", target, "SkinGacha", pos)?;
        }
    }
    for (pos, row) in rows(p.batch, "SkinGachaGroup").iter().enumerate() {
        let o = origin("skin_gacha_groups", pos as i64, "SkinGachaGroup", pos);
        source(p, &o, row)?;
        let found = p.link(
            &o,
            "GachaName",
            &row["GachaName"],
            ("SkinGacha", "Name"),
            true,
            false,
        )?;
        p.db.execute(
            "INSERT INTO skin_gacha_groups VALUES(?1,?2,?3,?4,?5)",
            params![
                pos as i64,
                string(&row["Group"]),
                string(&row["GachaName"]),
                target_id(p, "SkinGacha", found),
                number(row, "Prob")
            ],
        )?;
        if let Some(i) = found {
            p.source(
                "skin_gacha_results",
                id(&rows(p.batch, "SkinGacha")[i])?,
                "SkinGachaGroup",
                pos,
            )?;
            let gacha_id = id(&rows(p.batch, "SkinGacha")[i])?;
            let skin: Option<i64> = p.db.query_row(
                "SELECT skin_id FROM skin_gacha_results WHERE id=?1",
                [gacha_id],
                |r| r.get(0),
            )?;
            if let Some(skin) = skin {
                p.source("skins", skin, "SkinGachaGroup", pos)?;
            }
        }
    }
    Ok(())
}

pub(super) fn fingerprints(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    en: &Localization,
    loc: &Localization,
) -> Result<catalogue_projection::Fingerprints> {
    let families: Vec<_> = ROOTS.iter().map(|(_, f, _)| *f).collect();
    let mut result = catalogue_projection::fingerprints(db, batch, en, loc, &families)?;
    let mut contexts: BTreeMap<(String, i64), Vec<Value>> = BTreeMap::new();
    for row in db.prepare("SELECT family,source_id,role,item_reference,quantity_raw FROM catalogue_item_links WHERE family IN('pets','wings','skins','skin_sets','pet_collections')")?.query_map([],|r|Ok(((r.get::<_,String>(0)?,r.get::<_,i64>(1)?),serde_json::json!([r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,Option<i64>>(4)?]))))? {
        let (key,value)=row?;contexts.entry(key).or_default().push(value);
    }
    for (key, mut values) in contexts {
        values.sort_by_cached_key(Value::to_string);
        if let Some((base, _)) = result.get_mut(&key) {
            *base = hash(&serde_json::to_vec(&(&base, values))?);
        }
    }
    // Shared collection definitions are hashed once per natural category,
    // rather than duplicating hundreds of evidence links for every skin.
    let mut collections: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in rows(batch, "SkinCollection") {
        if let Some(category) = string(&row["SkinCategory"]) {
            collections
                .entry(category.into())
                .or_default()
                .push(hash(&serde_json::to_vec(row)?));
        }
    }
    let collection_hashes: BTreeMap<_, _> = collections
        .into_iter()
        .map(|(category, mut hashes)| {
            hashes.sort();
            (category, hash(hashes.join("").as_bytes()))
        })
        .collect();
    for row in rows(batch, "Skin") {
        if let Some(context) = string(&row["SkinCategory"]).and_then(|c| collection_hashes.get(c))
            && let Some((base, _)) = result.get_mut(&("skins".into(), id(row)?))
        {
            *base = hash(&serde_json::to_vec(&(&base, context))?);
        }
    }
    // Stat labels use localization keys outside the source FText fields.
    type StatLabels = BTreeMap<(String, i64), Vec<(String, Option<String>, Option<String>)>>;
    let mut labels = StatLabels::new();
    let sql="WITH owners(family,id,stat_id) AS (
        SELECT family,source_id,stat_id FROM appearance_stats
        UNION SELECT 'pet_options',id,stat_id FROM pet_options WHERE stat_id IS NOT NULL
        UNION SELECT 'pet_stat_levels',level_id,stat_id FROM pet_stat_values
        UNION SELECT 'pets',p.id,v.stat_id FROM pets p JOIN pet_stat_levels l ON l.group_id=p.stat_group_id JOIN pet_stat_values v ON v.level_id=l.id
        UNION SELECT 'wings',w.id,s.stat_id FROM wings w JOIN wing_effect_stats s ON s.group_id=w.effect_group_id
        UNION SELECT 'skins',s.id,c.stat_id FROM skins s JOIN skin_collection_stats c ON c.category_id=s.category_id
    ) SELECT o.family,o.id,s.code,s.name_en,s.name_localized FROM owners o JOIN stats s ON s.id=o.stat_id ORDER BY o.family,o.id,s.code";
    for row in db.prepare(sql)?.query_map([], |r| {
        Ok((
            (r.get::<_, String>(0)?, r.get::<_, i64>(1)?),
            (
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
            ),
        ))
    })? {
        let (key, label) = row?;
        labels.entry(key).or_default().push(label);
    }
    for (key, labels) in labels {
        if let Some((base, localized)) = result.get_mut(&key) {
            let english: Vec<_> = labels.iter().map(|(code, en, _)| (code, en)).collect();
            let translated: Vec<_> = labels.iter().map(|(code, _, loc)| (code, loc)).collect();
            *base = hash(&serde_json::to_vec(&(&base, english))?);
            *localized = hash(&serde_json::to_vec(&(&localized, translated))?);
        }
    }
    Ok(result)
}
