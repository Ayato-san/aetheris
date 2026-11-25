//! Item acquisition evidence: client routes are hints; output definitions are
//! not a claim of live availability or a normalized drop probability.
use super::*;

pub(super) fn is_source(name: &str) -> bool {
    name.contains("Reward")
        || name.ends_with("Loot")
        || [
            "CraftRecipe",
            "ConversionRecipe",
            "GatherSource",
            "Quest",
            "Achievement",
            "Dungeon",
            "BattlePass",
            "SeasonRanking",
            "SeasonSchedule",
            "GameShop",
            "RandomShop",
            "PeriodItemCollection",
            "FieldEvent",
            "BossChallenge",
            "Monolith",
        ]
        .iter()
        .any(|p| name.starts_with(p))
        || matches!(
            name,
            "Map"
                | "Subzone"
                | "NpcData"
                | "EnvObjData"
                | "BMShopGoods"
                | "Subscribe"
                | "MembershipPass"
                | "Attendance"
                | "Ascension"
                | "LevelScaleDropGroup"
                | "ArcanaCreate"
                | "VehicleCollection"
                | "ContentsNavigatorLevel"
                | "ContentsSweepTicket"
                | "Matching"
                | "GlobalSetting"
                | "SeasonContents"
                | "SkinCollection"
        )
}
pub(super) fn structure(name: &str) -> String {
    match name {
        "VehicleList" => "VehicleDataTableItem",
        "EnvObjData" => "EnvObjTableItem",
        "RewardData" => "RewardTableItem",
        "RewardLevelScaleData" => "RewardLevelScaleTableItem",
        "PeriodItemCollection" => "PeriodCollectionTableItem",
        "PeriodItemCollectionList" => "PeriodCollectionListTableItem",
        "RandomShop" => "RandomShopCategoryTableItem",
        "RewardDataMonsterCube" => "MonsterCubeTableItem",
        "SeasonRankingReward" => "SeasonRankingRewardCommonTableItem",
        _ => return format!("{name}TableItem"),
    }
    .into()
}
fn rows<'a>(batch: &'a BTreeMap<String, TableProbe>, name: &str) -> &'a [Value] {
    batch
        .get(name)
        .map(|t| t.layout.rows.as_slice())
        .unwrap_or(&[])
}
fn active(value: &Value) -> bool {
    match value {
        Value::Object(o) if o.contains_key("Entries") => o["Entries"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| active(&v["Value"]))),
        Value::Object(o) => o.iter().any(|(k, v)| k != "Removed" && active(v)),
        Value::Array(a) => a.iter().any(active),
        Value::Bool(v) => *v,
        Value::Number(n) => n.as_i64().is_some_and(|n| n != 0),
        Value::String(s) => !s.is_empty() && s != "None" && !s.ends_with("::None"),
        Value::Null => false,
    }
}
struct Origin<'a> {
    kind: &'a str,
    table: &'a str,
    row: usize,
}
fn source(
    db: &Connection,
    item: i64,
    origin: &Origin<'_>,
    path: &str,
    evidence: &str,
    quantity: (Option<i64>, Option<i64>),
    weight: Option<String>,
) -> Result<()> {
    let Origin { kind, table, row } = *origin;
    let (min, max) = quantity;
    let ty = dictionary(db, "item_obtaining_types", kind)?;
    db.execute(
        "INSERT OR IGNORE INTO item_obtaining_sources VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            item, ty, table, row as i64, path, evidence, min, max, weight
        ],
    )?;
    Ok(())
}
fn id_field(table: &str) -> &str {
    match table {
        "Achievement" => "AchievementId",
        "Ascension" | "AscensionReward" => "DataId",
        _ => "ID",
    }
}
fn target(kind: &str, path: &str, field: &str) -> Option<&'static str> {
    match field {
        "NpcId" => Some("NpcData"),
        "CraftRecipeId" => Some("CraftRecipe"),
        "QuestId" => Some("Quest"),
        "EnvObjId" => Some("EnvObjData"),
        "ItemId" | "ItemIds" => Some("Item"),
        "MapId" | "MapIdList" => Some("Map"),
        "SubzoneId" | "SubZoneIdList" => Some("Subzone"),
        "ConversionRecipeId" => Some("ConversionRecipe"),
        "ScheduleId" => Some("SeasonSchedule"),
        "MembershipPassId" => Some("MembershipPass"),
        "AchievementId" => Some("Achievement"),
        "GoodsIds" => Some("GameShopGoods"),
        "KibeliskIds" => Some("EnvObjData"),
        "FiledBattleEvents" => Some("FieldEvent"),
        "GrowthDungeonIds" | "DailyDungeonIds" => Some("Dungeon"),
        "TableId" if kind == "AchievementInfo" => Some("Achievement"),
        "Key" if kind == "NPCShopInfo" => Some("NpcData"),
        "Key" if kind == "CoinShopInfo" => Some("GameShop"),
        "Key" if kind == "MonolithInfo" && path.contains("SubzoneInfos/") => Some("Subzone"),
        "Key" if kind == "MonolithInfo" => Some("Map"),
        "TableIds" => match kind {
            "PartyDungeonGetRoutes"
            | "TrialDungeonGetRoutes"
            | "SuppressionInfo"
            | "AwakenInfo" => Some("Dungeon"),
            "ExplorationSealInfo" | "ExplorationGarrisionInfo" => Some("Map"),
            "AscensionInfo" => Some("Quest"),
            "RewardBoxInfo" => Some("Item"),
            "MonolithInfo" => Some("EnvObjData"),
            "AttendanceInfo" => Some("Attendance"),
            _ => None,
        },
        _ => match kind {
            "BattlePassInfo" => Some("BattlePass"),
            "RandomShopInfo" => Some("RandomShopGoods"),
            "RaidInfo" => Some("Dungeon"),
            "DecomposeItemInfos" => Some("Item"),
            "PeriodCollectionInfo" => Some("PeriodItemCollectionList"),
            "SubscribeInfo" => Some("Subscribe"),
            _ => None,
        },
    }
}
fn route_links(
    db: &Connection,
    keys: &BTreeMap<&str, BTreeSet<String>>,
    item_row: (i64, usize),
    kind: &str,
    value: &Value,
    path: &str,
    field: &str,
) -> Result<()> {
    let (item, row) = item_row;
    let leaf = if let Some(n) = value["Value"].as_i64().filter(|n| *n > 0) {
        Some(n.to_string())
    } else if value.is_number()
        && matches!(
            field,
            "NpcId"
                | "QuestId"
                | "CraftRecipeId"
                | "EnvObjId"
                | "MapId"
                | "SubzoneId"
                | "ConversionRecipeId"
                | "ScheduleId"
                | "AchievementId"
        )
    {
        value.as_i64().filter(|n| *n > 0).map(|n| n.to_string())
    } else if field == "Key" && kind == "CoinShopInfo" {
        string(value).map(str::to_owned)
    } else {
        None
    };
    if let Some(reference) = leaf {
        let table = target(kind, path, field);
        // AchievementInfo mixes individual achievements and achievement groups.
        let table = if table == Some("Achievement")
            && kind == "AchievementInfo"
            && !keys
                .get("Achievement")
                .is_some_and(|k| k.contains(&reference))
            && keys
                .get("AchievementGroup")
                .is_some_and(|k| k.contains(&reference))
        {
            Some("AchievementGroup")
        } else {
            table
        };
        let resolved = table
            .and_then(|t| keys.get(t))
            .is_some_and(|k| k.contains(&reference));
        db.execute(
            "INSERT INTO item_obtaining_routes VALUES(?1,?2,'ItemGetRoute',?3,?4,?5,?6,?7)",
            params![item, kind, row as i64, path, table, reference, resolved],
        )?;
        return Ok(());
    }
    match value {
        Value::Array(a) => {
            for (i, v) in a.iter().enumerate() {
                route_links(db, keys, item_row, kind, v, &format!("{path}/{i}"), field)?;
            }
        }
        Value::Object(o) => {
            for (k, v) in o {
                if k == "Removed" {
                    continue;
                }
                // Map wrappers don't replace the semantic field of a nested ID.
                let next = if k == "Value" { field } else { k };
                route_links(db, keys, item_row, kind, v, &format!("{path}/{k}"), next)?;
            }
        }
        _ => (),
    }
    Ok(())
}

pub(super) fn write(db: &Connection, batch: &BTreeMap<String, TableProbe>) -> Result<()> {
    let names: BTreeMap<&str, i64> = rows(batch, "Item")
        .iter()
        .filter_map(|r| Some((string(&r["Name"])?, number(r, "ID")?)))
        .collect();
    let ids: BTreeSet<i64> = names.values().copied().collect();
    let mut keys: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for (name, table) in batch {
        for row in &table.layout.rows {
            for field in [id_field(name), "Name", "Group"] {
                if let Some(n) = number(row, field) {
                    keys.entry(name).or_default().insert(n.to_string());
                } else if let Some(s) = string(&row[field]) {
                    keys.entry(name).or_default().insert(s.into());
                }
            }
        }
    }
    for (position, row) in rows(batch, "ItemGetRoute").iter().enumerate() {
        let Some(item) = number(row, "ItemId").filter(|id| ids.contains(id)) else {
            continue;
        };
        if let Some(object) = row.as_object() {
            for (kind, value) in object {
                if kind == "ItemId" {
                    continue;
                }
                let present = if kind == "BossChallengeInfo" {
                    value["bBossChallenge"] == true
                } else {
                    active(value)
                };
                if present {
                    source(
                        db,
                        item,
                        &Origin {
                            kind,
                            table: "ItemGetRoute",
                            row: position,
                        },
                        kind,
                        "client_route",
                        (None, None),
                        None,
                    )?;
                }
                route_links(db, &keys, (item, position), kind, value, kind, kind)?;
            }
        }
    }
    for (position, row) in rows(batch, "ItemGetRouteManual").iter().enumerate() {
        if let Some(item) = number(row, "ItemId").filter(|id| ids.contains(id)) {
            for field in ["Categories", "Infos"] {
                if active(&row[field]) {
                    source(
                        db,
                        item,
                        &Origin {
                            kind: "Manual",
                            table: "ItemGetRouteManual",
                            row: position,
                        },
                        field,
                        "manual",
                        (None, None),
                        None,
                    )?;
                }
            }
        }
    }
    outputs(db, batch, &names, &ids)?;
    let count: i64 = db.query_row("SELECT COUNT(*) FROM item_obtaining_sources", [], |r| {
        r.get(0)
    })?;
    coverage(
        db,
        "items.obtaining",
        "partial",
        &format!(
            "{count} acquisition evidence records; all nonempty client route fields and explicit output definitions indexed; hints do not prove live availability, unknown IDs and raw probability scales remain unresolved"
        ),
    )
}

struct Outputs<'a> {
    names: &'a BTreeMap<&'a str, i64>,
    ids: &'a BTreeSet<i64>,
    matched: usize,
    missing: usize,
}
fn output(
    db: &Connection,
    outputs: &mut Outputs<'_>,
    origin: &Origin<'_>,
    path: &str,
    value: &Value,
    fallback_count: Option<i64>,
) -> Result<()> {
    let name = string(value)
        .or_else(|| string(&value["ItemName"]))
        .or_else(|| string(&value["Item"]));
    let numeric = value["Value"].as_i64().filter(|n| *n > 0);
    if name.is_some() || numeric.is_some() {
        let item = name
            .and_then(|n| outputs.names.get(n).copied())
            .or_else(|| numeric.filter(|id| outputs.ids.contains(id)));
        if let Some(item) = item {
            let min = number(value, "ItemCountMin")
                .or_else(|| number(value, "Count"))
                .or(fallback_count);
            let max = number(value, "ItemCountMax").or(min);
            let weight = value
                .get("ItemProb")
                .or_else(|| value.get("Prob"))
                .map(Value::to_string);
            source(db, item, origin, path, "output", (min, max), weight)?;
            outputs.matched += 1;
        } else {
            outputs.missing += 1;
        }
        return Ok(());
    }
    match value {
        Value::Array(a) => {
            for (i, v) in a.iter().enumerate() {
                output(
                    db,
                    outputs,
                    origin,
                    &format!("{path}/{i}"),
                    v,
                    fallback_count,
                )?;
            }
        }
        Value::Object(o) => {
            for (k, v) in o {
                // Explicit output substructures only: never turn component costs,
                // required items, tickets or purchase currencies into obtained loot.
                if matches!(
                    k.as_str(),
                    "Item"
                        | "RewardItem"
                        | "Items"
                        | "RewardItems"
                        | "LightItemReward"
                        | "DarkItemReward"
                ) {
                    output(
                        db,
                        outputs,
                        origin,
                        &format!("{path}/{k}"),
                        v,
                        fallback_count,
                    )?;
                }
            }
        }
        _ => (),
    }
    Ok(())
}
fn outputs(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    names: &BTreeMap<&str, i64>,
    ids: &BTreeSet<i64>,
) -> Result<()> {
    let mut outputs = Outputs {
        names,
        ids,
        matched: 0,
        missing: 0,
    };
    // Paths identify the exact output occurrence. Conditions, race, class,
    // dates, requirements, costs and scaling rules remain in its source row.
    for (table, kind, fields) in [
        (
            "NpcLoot",
            "MonsterGetRoutes",
            &["LootList", "VehicleSoulItemId"][..],
        ),
        (
            "CraftRecipe",
            "CraftGetRoutes",
            &["Product", "FailRewardList"],
        ),
        (
            "ConversionRecipe",
            "ConversionRecipeGetRoutes",
            &["Product", "CriticalSuccessProduct", "FailRewardProduct"],
        ),
        (
            "GatherSourceMaterial",
            "GatherGetRoutes",
            &["MaterialName", "AddMaterialList", "FailMaterialList"],
        ),
        ("GatherSource", "GatherGetRoutes", &["QuestMaterial"]),
        ("ArcanaCreate", "ArcanaCreation", &["ArcanaName"]),
        ("VehicleCollection", "VehicleCollection", &["RewardItem"]),
        ("DungeonRewardInfo", "DungeonReward", &["ItemReward"]),
        ("GameShopGoods", "NPCShopInfo", &["ItemName"]),
        ("RandomShopGoods", "RandomShopInfo", &["ItemName"]),
        ("BMShopGoods", "BMShopInfo", &["ItemName"]),
        (
            "QuestReward",
            "QuestGetRoutes",
            &["ItemRewards", "ItemNormalRewards", "ItemSelectRewards"],
        ),
        (
            "QuestClassReward",
            "QuestGetRoutes",
            &["ItemRewards", "ItemNormalRewards", "ItemSelectRewards"],
        ),
        ("AchievementReward", "AchievementInfo", &["ItemRewards"]),
        (
            "AchievementGroupReward",
            "AchievementInfo",
            &["ItemRewards"],
        ),
        ("BattlePass", "BattlePassInfo", &["ItemReward"]),
        ("Subscribe", "SubscribeInfo", &["RewardItemList"]),
        (
            "PeriodItemCollectionList",
            "PeriodCollectionInfo",
            &["RewardItems"],
        ),
        ("SeasonRankingRewardItem", "RankingInfo", &["Items"]),
        ("RewardDungeon", "PartyDungeonGetRoutes", &["Items"]),
        ("RewardRaceRanking", "RankingInfo", &["RewardItems"]),
        ("RewardArena", "bArena", &["Items"]),
        ("RewardMentor", "Mentor", &["RewardItemName"]),
        ("RewardMonolith", "MonolithInfo", &["RewardList"]),
        ("RewardOdEnergyCube", "DungeonReward", &["CeilingItems"]),
        ("AscensionReward", "AscensionInfo", &["ItemRewardList"]),
        ("BossChallengeReward", "BossChallengeInfo", &["ItemRewards"]),
        ("BoxReward", "RewardBoxInfo", &["Items", "SelectItems"]),
        ("BoxClassReward", "RewardBoxInfo", &["Items", "SelectItems"]),
        ("RewardItem", "RewardDefinition", &["Items"]),
        ("RewardPackageItem", "RewardPackageDefinition", &["Items"]),
    ] {
        for (position, row) in rows(batch, table).iter().enumerate() {
            for field in fields {
                let count_field = match *field {
                    "Product" => "ProductCount",
                    "CriticalSuccessProduct" => "CriticalSuccessProductCount",
                    "FailRewardProduct" => "FailRewardProductCount",
                    "MaterialName" => "MaterialCount",
                    "QuestMaterial" => "QuestMaterialCount",
                    "RewardItemName" | "RewardItem" => "RewardItemCount",
                    "ItemName" if table == "GameShopGoods" => "ItemBundleCount",
                    "ItemName" => "ItemCount",
                    _ => "",
                };
                output(
                    db,
                    &mut outputs,
                    &Origin {
                        kind,
                        table,
                        row: position,
                    },
                    field,
                    &row[field],
                    number(row, count_field),
                )?;
            }
        }
    }
    let Outputs {
        matched, missing, ..
    } = outputs;
    coverage(
        db,
        "items.obtaining.outputs",
        if missing == 0 { "verified" } else { "partial" },
        &format!(
            "{matched} explicit item output occurrences resolved, {missing} unmatched item references; inputs are excluded; reward-package composition and probability scales are not inferred"
        ),
    )
}
