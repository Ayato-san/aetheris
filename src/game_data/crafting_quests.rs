//! Explicit projections for #79. Reference identity does not establish gameplay
//! units, probability scales, objective parameter semantics or live availability.
use super::*;

pub(super) const SOURCES: &[&str] = &[
    "CraftRecipe",
    "CraftRecipeRate",
    "CraftMastery",
    "CraftRecipeTabUI",
    "CraftComponentReplacement",
    "ComponentCombine",
    "ConversionRecipe",
    "ConversionComponentReplacement",
    "ConversionComponentEquipment",
    "ConversionTabUI",
    "Quest",
    "QuestPart",
    "QuestStep",
    "QuestString",
    "QuestReward",
    "QuestClassReward",
    "QuestType",
    "DistrictQuestGroup",
    "DutyQuestGroup",
];

pub(super) fn family(source: &str) -> Option<&'static str> {
    match source {
        "CraftRecipe" => Some("craft_recipes"),
        "ConversionRecipe" => Some("conversion_recipes"),
        "Quest" => Some("quests"),
        _ => None,
    }
}
use super::catalogue_projection::{Origin, Projection, id, list, rows};

pub(super) fn write(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    en: &Localization,
    loc: &Localization,
    build: i64,
) -> Result<()> {
    let mut p = Projection::new(db, batch, en, loc, build);
    component_options(&mut p)?;
    for row in rows(batch, "CraftRecipeRate") {
        let name = string(&row["Name"]).ok_or_else(|| invalid("missing recipe rate group"))?;
        for (pos, rate) in list(row, "Rates").iter().enumerate() {
            db.execute(
                "INSERT INTO recipe_rates VALUES(?1,?2,?3,?4)",
                params![name, pos as i64, number(rate, "Gap"), number(rate, "Rate")],
            )?;
        }
    }
    for source in ["CraftRecipe", "ConversionRecipe"] {
        recipes(&mut p, source)?;
    }
    quests(&mut p)?;
    for (subject, required) in [("crafting", &SOURCES[..10]), ("quests", &SOURCES[10..])] {
        let absent: Vec<_> = required
            .iter()
            .filter(|s| !batch.contains_key(**s))
            .copied()
            .collect();
        let missing: i64 = db.query_row("SELECT count(*) FROM catalogue_references WHERE resolution_state!='resolved' AND ((?1='crafting' AND family IN('craft_recipes','conversion_recipes','recipe_component_groups')) OR (?1='quests' AND family IN('quests','quest_steps','quest_reward_groups')))",[subject],|r|r.get(0))?;
        coverage(
            db,
            subject,
            if !batch.contains_key(if subject == "crafting" {
                "CraftRecipe"
            } else {
                "Quest"
            }) {
                "not_researched"
            } else {
                "partial"
            },
            &format!(
                "{} undecoded source tables ({:?}); {missing} unresolved/ambiguous references; field-specific interpretation only; probabilities and objective/condition values retain source encoding",
                absent.len(),
                absent
            ),
        )?;
    }
    coverage(
        db,
        "crafting.station",
        "not_researched",
        "no verified station/location relationship; mastery is not character level",
    )?;
    coverage(
        db,
        "quests.objective_values",
        "uninterpreted",
        "typed ordered values retained without inferring counts, IDs or units from parameter positions",
    )?;
    coverage(
        db,
        "quests.reward_groups",
        "uninterpreted",
        "numeric group to Quest.ID is a candidate association; class variants and random pool references retained without claiming reward availability or probability",
    )?;
    Ok(())
}

fn component_options(p: &mut Projection<'_>) -> Result<()> {
    for source in [
        "CraftComponentReplacement",
        "ConversionComponentReplacement",
        "ConversionComponentEquipment",
        "ComponentCombine",
    ] {
        for (row_pos, row) in rows(p.batch, source).iter().enumerate() {
            let Some(group) = string(&row["Name"]) else {
                continue;
            };
            let entries: Vec<(&str, Vec<&Value>)> = if source == "ComponentCombine" {
                vec![
                    ("CombineList", list(row, "CombineList").iter().collect()),
                    ("RepresentItem", vec![&row["RepresentItem"]]),
                ]
            } else if source == "ConversionComponentEquipment" {
                vec![("ItemList", list(row, "ItemList").iter().collect())]
            } else {
                vec![(
                    "ReplacementItemList",
                    list(row, "ReplacementItemList").iter().collect(),
                )]
            };
            for (role, entries) in entries {
                for (pos, entry) in entries.into_iter().enumerate() {
                    let reference = string(entry).or_else(|| string(&entry["Item"]));
                    let Some(reference) = reference else { continue };
                    p.db.execute(
                        "INSERT INTO recipe_component_options VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                        params![
                            source,
                            group,
                            role,
                            pos as i64,
                            reference,
                            p.item(Some(reference)),
                            number(entry, "Count"),
                            number(entry, "EnchantedLevel")
                        ],
                    )?;
                    let o = Origin {
                        family: "recipe_component_groups",
                        id: id(row)?,
                        source,
                        row: row_pos,
                    };
                    p.link(
                        &o,
                        &format!("{role}/{pos}"),
                        &Value::from(reference),
                        ("Item", "Name"),
                        false,
                        false,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn recipes(p: &mut Projection<'_>, source: &str) -> Result<()> {
    let family = family(source).unwrap();
    for (pos, row) in rows(p.batch, source).iter().enumerate() {
        let id = id(row)?;
        let o = Origin {
            family,
            id,
            source,
            row: pos,
        };
        p.source(family, id, source, pos)?;
        let source_key = string(&row["Name"]).ok_or_else(|| invalid("missing recipe key"))?;
        let product = p.item(string(&row["Product"]));
        let product_key = p
            .link(
                &o,
                "Product",
                &row["Product"],
                ("Item", "Name"),
                false,
                false,
            )?
            .and_then(|pos| string(&rows(p.batch, "Item")[pos]["Desc"]["Key"]));
        super::projection::text_state(p.db, family, id, "name", product_key, p.en, p.loc)?;
        let names: (Option<String>, Option<String>) = product
            .map(|id| {
                p.db.query_row(
                    "SELECT name_en,name_localized FROM items WHERE id=?1",
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
            })
            .transpose()?
            .unwrap_or_default();
        let combo = p
            .link(
                &o,
                "ComboRecipe",
                &row["ComboRecipe"],
                (source, "Name"),
                false,
                false,
            )?
            .and_then(|pos| number(&rows(p.batch, source)[pos], "ID"));
        p.db.execute("INSERT INTO recipes VALUES(?1,?2,?3,?17,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",params![source,id,source_key,names.0,names.1,p.code(&row["Type"])?,p.code(&row["Category"])?,p.code(&row["SubCategory"])?,p.race(row,"QualificationRace")?,number(row,"MasteryLevel"),string(&row["ComboRecipe"]),combo,number(row,"ComboProbability"),string(&row["CraftRateGroup"]),row["bShowRecipe"].as_bool(),row["bGuildCraft"].as_bool(),p.build])?;
        p.db.execute("INSERT INTO catalogue_entity_icons SELECT ?1,?2,icon_id FROM catalogue_entity_icons WHERE family='items' AND source_id=?3",params![family,id,product])?;
        for (role, field, count, enchant, probability) in [
            (
                "product",
                "Product",
                "ProductCount",
                "ProductEnchant",
                "SuccessProp",
            ),
            (
                "critical",
                "CriticalSuccessProduct",
                "CriticalSuccessProductCount",
                "CriticalSuccessProductEnchant",
                "CriticalSuccessProp",
            ),
            (
                "failure",
                "FailRewardProduct",
                "FailRewardProductCount",
                "FailRewardProductEnchant",
                "FailRewardProp",
            ),
        ] {
            if let Some(reference) = string(&row[field]) {
                p.db.execute(
                    "INSERT INTO recipe_results VALUES(?1,?2,?3,0,?4,?5,?6,?7,?8)",
                    params![
                        source,
                        id,
                        role,
                        reference,
                        p.item(Some(reference)),
                        number(row, count),
                        number(row, enchant),
                        number(row, probability)
                    ],
                )?;
                if field != "Product" {
                    p.link(&o, field, &row[field], ("Item", "Name"), false, false)?;
                }
            }
        }
        for (component_pos, component) in list(row, "ComponentList").iter().enumerate() {
            p.db.execute(
                "INSERT INTO recipe_components VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    source,
                    id,
                    component_pos as i64,
                    string(&component["Item"]),
                    p.item(string(&component["Item"])),
                    number(component, "Count"),
                    number(component, "Enchant"),
                    string(&component["ReplacementItem"]),
                    string(&component["CombineName"])
                ],
            )?;
            for (field, target) in [
                ("Item", "Item"),
                (
                    "ReplacementItem",
                    if source == "CraftRecipe" {
                        "CraftComponentReplacement"
                    } else {
                        "ConversionComponentReplacement"
                    },
                ),
                ("CombineName", "ComponentCombine"),
            ] {
                p.link(
                    &o,
                    &format!("ComponentList/{component_pos}/{field}"),
                    &component[field],
                    (target, "Name"),
                    false,
                    field != "Item",
                )?;
            }
        }
        for field in ["CraftingFee", "RemoteCraftingFee"] {
            if let Some(amount) = number(row, field) {
                // Only CraftingFee has an explicitly named currency field.
                let currency = if field == "CraftingFee" {
                    p.code(&row["CraftingFeeType"])?
                } else {
                    None
                };
                p.db.execute(
                    "INSERT INTO recipe_costs VALUES(?1,?2,?3,?4,?5)",
                    params![source, id, field, currency, amount],
                )?;
            }
        }
        for field in [
            "LearnType",
            "NeedGuildMemberGrade",
            "LimitAccountCount",
            "LimitCharCount",
            "LimitCharacterCount",
            "LimitServerCount",
            "LimitDateStart",
            "LimitDateEnd",
            "LimitResetCycleType",
            "RegisterType",
            "RegisterValue",
            "RegisterValue2",
            "CraftCount",
            "ConversionCount",
            "ConversionCountType",
            "CraftGauge",
            "FailGaugeValue",
            "FailRewardProbability",
            "FailRewardList",
            "SuccessionComponentCandidateList",
            "MultiSelectComponent",
        ] {
            if let Some(value) = row.get(field) {
                p.condition(&o, field, &Value::Null, value)?;
            }
        }
        p.texts(&o, row)?;
        for (field, target, target_field) in [
            ("CraftRateGroup", "CraftRecipeRate", "Name"),
            (
                "MultiSelectComponent",
                "ConversionComponentEquipment",
                "Name",
            ),
            ("SubTab", "CraftRecipeTabUI", "Name"),
            ("Tab", "ConversionTabUI", "Name"),
        ] {
            p.link(&o, field, &row[field], (target, target_field), false, true)?;
        }
        for (pos, reference) in list(row, "SuccessionComponentCandidateList")
            .iter()
            .enumerate()
        {
            p.link(
                &o,
                &format!("SuccessionComponentCandidateList/{pos}"),
                reference,
                ("Item", "Name"),
                false,
                false,
            )?;
        }
    }
    Ok(())
}

fn quests(p: &mut Projection<'_>) -> Result<()> {
    for (pos, row) in rows(p.batch, "QuestPart").iter().enumerate() {
        let id = id(row)?;
        let key = string(&row["PartText"]["Key"]);
        p.db.execute(
            "INSERT INTO quest_parts VALUES(?1,?2,?3,?4,?5)",
            params![
                id,
                string(&row["Name"]),
                number(row, "PartOrder"),
                text(p.en, key),
                text(p.loc, key)
            ],
        )?;
        p.source("quest_parts", id, "QuestPart", pos)?;
        p.texts(
            &Origin {
                family: "quest_parts",
                id,
                source: "QuestPart",
                row: pos,
            },
            row,
        )?;
    }
    for (pos, row) in rows(p.batch, "Quest").iter().enumerate() {
        let id = id(row)?;
        let o = Origin {
            family: "quests",
            id,
            source: "Quest",
            row: pos,
        };
        p.source("quests", id, "Quest", pos)?;
        let part = p
            .link(&o, "Part", &row["Part"], ("QuestPart", "Name"), false, true)?
            .and_then(|i| number(&rows(p.batch, "QuestPart")[i], "ID"));
        let next = p
            .link(
                &o,
                "NextQuestName",
                &row["NextQuestName"],
                ("Quest", "Name"),
                false,
                false,
            )?
            .and_then(|i| number(&rows(p.batch, "Quest")[i], "ID"));
        let texts = p
            .link(
                &o,
                "QuestText",
                &row["QuestText"],
                ("QuestString", "Name"),
                false,
                true,
            )?
            .map(|i| &rows(p.batch, "QuestString")[i]);
        let name = texts.and_then(|r| string(&r["Title"]["Key"]));
        let description = texts.and_then(|r| string(&r["ShortSummary"]["Key"]));
        super::projection::text_state(p.db, "quests", id, "name", name, p.en, p.loc)?;
        super::projection::text_state(p.db, "quests", id, "description", description, p.en, p.loc)?;
        p.db.execute("INSERT INTO quests VALUES(?1,?2,?18,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",params![id,string(&row["Name"]),text(p.en,name),text(p.loc,name),text(p.en,description),text(p.loc,description),p.code(&row["Type"])?,p.code(&row["Grade"])?,p.race(row,"Race")?,number(row,"UnlockLevel"),number(row,"RecommendedLevel"),p.code(&row["RepeatType"])?,string(&row["Part"]),part,string(&row["NextQuestName"]),next,row["bEnableGiveup"].as_bool(),p.build])?;
        if let Some(texts) = texts {
            for (path, key) in p.texts(&o, texts)? {
                p.db.execute(
                    "INSERT INTO quest_texts VALUES(?1,?2,?3,?4,?5)",
                    params![
                        id,
                        path,
                        key,
                        text(p.en, Some(&key)),
                        text(p.loc, Some(&key))
                    ],
                )?;
            }
        }
        p.link(&o, "Type", &row["Type"], ("QuestType", "Type"), false, true)?;
        for field in ["AcquireMapId", "CompleteMapId"] {
            p.link(&o, field, &row[field], ("Map", "ID"), false, false)?;
        }
        for (pos, condition) in list(row, "AcquirableConditions").iter().enumerate() {
            let path = format!("AcquirableConditions/{pos}");
            p.condition(&o, &path, &condition["Type"], &condition["Value"])?;
            let target = match string(&condition["Type"]) {
                Some(
                    "EQuestAcquirableCondition::ClearQuest"
                    | "EQuestAcquirableCondition::IncompleteQuest",
                ) => Some(("Quest", "Name", false)),
                Some("EQuestAcquirableCondition::ClearQuestStep") => {
                    Some(("QuestStep", "ID", true))
                }
                Some("EQuestAcquirableCondition::SubZone") => Some(("Subzone", "Name", false)),
                Some("EQuestAcquirableCondition::CloseEnvObj") => {
                    Some(("EnvObjData", "Name", false))
                }
                _ => None,
            };
            if let Some((table, field, numeric)) = target {
                let value = if numeric {
                    string(&condition["Value"])
                        .and_then(|v| v.parse::<i64>().ok())
                        .map(Value::from)
                        .unwrap_or_else(|| condition["Value"].clone())
                } else {
                    condition["Value"].clone()
                };
                p.link(&o, &path, &value, (table, field), true, false)?;
            }
        }
        for field in [
            "AcquireType",
            "AcquireValue",
            "AcquireValue2",
            "CompleteType",
            "CompleteValue",
            "AcquireItemList",
            "CompleteTravelType",
            "CompleteTeleportCost",
            "SubzoneGroup",
            "DistrictQuestGroup",
            "RaceTag",
            "ExplorationType",
        ] {
            if let Some(value) = row.get(field) {
                p.condition(&o, field, &Value::Null, value)?;
            }
        }
        for (field, ty) in [
            ("AcquireValue", "AcquireType"),
            ("CompleteValue", "CompleteType"),
        ] {
            if matches!(
                string(&row[ty]),
                Some("EQuestAcquireType::Npc" | "EQuestCompleteType::Npc")
            ) {
                p.link(&o, field, &row[field], ("NpcData", "Name"), true, false)?;
            }
        }
    }
    quest_steps(p)?;
    quest_rewards(p)
}

fn quest_steps(p: &mut Projection<'_>) -> Result<()> {
    for (pos, row) in rows(p.batch, "QuestStep").iter().enumerate() {
        let id = id(row)?;
        let o = Origin {
            family: "quest_steps",
            id,
            source: "QuestStep",
            row: pos,
        };
        let quest = p
            .link(
                &o,
                "QuestName",
                &row["QuestName"],
                ("Quest", "Name"),
                false,
                false,
            )?
            .and_then(|i| number(&rows(p.batch, "Quest")[i], "ID"));
        let fail_order = number(row, "FailGoToStep");
        let matches: Vec<_> = rows(p.batch, "QuestStep")
            .iter()
            .filter(|r| {
                r["QuestName"] == row["QuestName"]
                    && number(r, "Order") == fail_order
                    && fail_order.is_some_and(|v| v > 0)
            })
            .collect();
        let fail_id = if let [target] = matches.as_slice() {
            number(target, "ID")
        } else {
            None
        };
        p.db.execute(
            "INSERT INTO quest_steps VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                id,
                string(&row["QuestName"]),
                quest,
                number(row, "Order"),
                p.code(&row["FailConditionType"])?,
                fail_order,
                fail_id,
                row["bUnifiedObjectives"].as_bool(),
                number(row, "UnifiedObjectiveScore")
            ],
        )?;
        p.source("quest_steps", id, "QuestStep", pos)?;
        if let Some(quest) = quest {
            p.source("quests", quest, "QuestStep", pos)?;
        }
        if fail_order.is_some_and(|n| n > 0) {
            p.condition(
                &o,
                "FailGoToStep",
                &row["FailConditionType"],
                &row["FailGoToStep"],
            )?;
            if let Some(fail_id) = fail_id {
                p.link(
                    &o,
                    "FailGoToStep",
                    &Value::from(fail_id),
                    ("QuestStep", "ID"),
                    true,
                    false,
                )?;
            } else {
                coverage(
                    p.db,
                    &format!("quests.step.{id}.failure_branch"),
                    "partial",
                    "failure target order is missing or ambiguous within its quest; original order retained",
                )?;
            }
        }
        for (goal_pos, goal) in list(row, "GoalList").iter().enumerate() {
            p.db.execute(
                "INSERT INTO quest_objectives VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    id,
                    goal_pos as i64,
                    p.code(&goal["Type"])?,
                    serde_json::to_string(&goal["Value"])?,
                    number(goal, "MapId"),
                    goal["bOptional"].as_bool(),
                    goal["bHideUI"].as_bool(),
                    goal["bMarker"].as_bool()
                ],
            )?;
            p.link(
                &o,
                &format!("GoalList/{goal_pos}/MapId"),
                &goal["MapId"],
                ("Map", "ID"),
                false,
                false,
            )?;
            p.condition(&o, &format!("GoalList/{goal_pos}"), &goal["Type"], goal)?;
        }
    }
    Ok(())
}

fn quest_rewards(p: &mut Projection<'_>) -> Result<()> {
    for source in ["QuestReward", "QuestClassReward"] {
        for (pos, row) in rows(p.batch, source).iter().enumerate() {
            let group =
                string(&row["Group"]).ok_or_else(|| invalid("missing quest reward group"))?;
            let value = group
                .parse::<i64>()
                .map(Value::from)
                .unwrap_or_else(|_| Value::from(group));
            // Source row identity here is provenance, not a stable quest ID.
            let o = Origin {
                family: "quest_reward_groups",
                id: pos as i64,
                source,
                row: pos,
            };
            let quest = p
                .link(&o, "Group", &value, ("Quest", "ID"), true, false)?
                .and_then(|i| number(&rows(p.batch, "Quest")[i], "ID"));
            p.db.execute(
                "INSERT INTO quest_reward_groups VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![
                    source,
                    pos as i64,
                    group,
                    quest,
                    p.code(&row["Class"])?,
                    number(row, "ExpReward"),
                    string(&row["RandomReward"])
                ],
            )?;
            if let Some(quest) = quest {
                p.source("quests", quest, source, pos)?;
            }
            for mode in ["ItemRewards", "ItemNormalRewards", "ItemSelectRewards"] {
                for (item_pos, item) in list(row, mode).iter().enumerate() {
                    let reference = string(&item["Item"])
                        .ok_or_else(|| invalid("missing quest reward item"))?;
                    p.db.execute(
                        "INSERT INTO quest_rewards VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                        params![
                            source,
                            pos as i64,
                            mode,
                            item_pos as i64,
                            reference,
                            p.item(Some(reference)),
                            number(item, "Count"),
                            number(item, "EnchantedLevel")
                        ],
                    )?;
                    p.link(
                        &o,
                        &format!("{mode}/{item_pos}/Item"),
                        &item["Item"],
                        ("Item", "Name"),
                        false,
                        false,
                    )?;
                }
            }
            p.link(
                &o,
                "RandomReward",
                &row["RandomReward"],
                ("RewardPackage", "Name"),
                true,
                false,
            )?;
        }
    }
    Ok(())
}

/// Fingerprint complete supporting definitions and same-language texts. No old
/// contents are retained, and referenced loot definitions keep their own changes.
pub(super) fn fingerprints(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    en: &Localization,
    loc: &Localization,
) -> Result<BTreeMap<(String, i64), (String, String)>> {
    let mut result = super::catalogue_projection::fingerprints(
        db,
        batch,
        en,
        loc,
        &["craft_recipes", "conversion_recipes", "quests"],
    )?;
    // Product names belong to recipe display; other item properties do not.
    for result_row in db
        .prepare("SELECT source_name,id,name_en,name_localized FROM recipes")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?
    {
        let (source, id, en, loc) = result_row?;
        if let Some((base, localized)) = result.get_mut(&(family(&source).unwrap().into(), id)) {
            *base = hash(&serde_json::to_vec(&(&base, en))?);
            *localized = hash(&serde_json::to_vec(&(&localized, loc))?);
        }
    }
    Ok(result)
}
