//! #80: definitions and source rules, never character ownership or live progress.
use super::catalogue_projection::{Origin, Projection, list, rows};
use super::*;

pub(super) const SOURCES: &[&str] = &[
    "Achievement",
    "AchievementStep",
    "AchievementReward",
    "AchievementString",
    "AchievementGroup",
    "AchievementGroupSet",
    "AchievementGroupReward",
    "Journal",
    "Title",
    "TitleCategory",
    "BattlePass",
    "BattlePassGroup",
    "SeasonSchedule",
    "SeasonScheduleGroup",
    "SeasonContents",
    "SeasonPickupContents",
    "EventMission",
    "PeriodItemCollection",
    "PeriodItemCollectionList",
];
// Source, evidence family and product table. All these roots have source IDs.
pub(super) const ROOTS: &[(&str, &str, &str)] = &[
    ("Achievement", "achievements", "achievements"),
    (
        "AchievementGroup",
        "achievement_groups",
        "achievement_groups",
    ),
    ("Title", "titles", "titles"),
    ("Journal", "journals", "journals"),
    (
        "SeasonScheduleGroup",
        "season_schedule_groups",
        "season_schedule_groups",
    ),
    ("SeasonSchedule", "seasons", "seasons"),
    ("SeasonContents", "season_contents", "season_contents"),
    ("EventMission", "event_missions", "event_missions"),
    (
        "BattlePassGroup",
        "battle_pass_groups",
        "battle_pass_groups",
    ),
    ("BattlePass", "battle_passes", "battle_passes"),
    (
        "PeriodItemCollection",
        "supply_request_groups",
        "supply_request_groups",
    ),
    (
        "PeriodItemCollectionList",
        "supply_requests",
        "supply_requests",
    ),
];
fn identity(row: &Value, field: &str) -> Result<i64> {
    number(row, field)
        .filter(|v| *v > 0)
        .ok_or_else(|| invalid(format!("missing positive #80 identity: {field}")).into())
}
fn key<'a>(row: &'a Value, field: &str) -> Result<&'a str> {
    string(&row[field]).ok_or_else(|| invalid(format!("missing #80 source key: {field}")).into())
}
fn origin<'a>(family: &'a str, id: i64, source: &'a str, row: usize) -> Origin<'a> {
    Origin {
        family,
        id,
        source,
        row,
    }
}
fn resolved_id(p: &Projection<'_>, table: &str, pos: Option<usize>, field: &str) -> Option<i64> {
    pos.and_then(|pos| number(&rows(p.batch, table)[pos], field))
}
fn conditions(p: &Projection<'_>, o: &Origin<'_>, row: &Value, fields: &[&str]) -> Result<()> {
    for field in fields {
        if let Some(value) = row.get(*field) {
            p.condition(o, field, &Value::Null, value)?;
        }
    }
    Ok(())
}
fn period(
    p: &Projection<'_>,
    o: &Origin<'_>,
    role: &str,
    row: &Value,
    start: &str,
    end: &str,
) -> Result<()> {
    if row.get(start).is_some() || row.get(end).is_some() {
        p.db.execute(
            "INSERT INTO catalogue_periods VALUES(?1,?2,?3,?4,?5)",
            params![
                o.family,
                o.id,
                role,
                number(&row[start], "Ticks"),
                number(&row[end], "Ticks")
            ],
        )?;
    }
    Ok(())
}
fn cost(
    p: &Projection<'_>,
    o: &Origin<'_>,
    path: &str,
    pos: Option<usize>,
    currency: &Value,
    amount: Option<i64>,
) -> Result<()> {
    p.db.execute(
        "INSERT INTO catalogue_costs VALUES(?1,?2,?3,?4,?5,?6)",
        params![
            o.family,
            o.id,
            path,
            pos.map(|n| n as i64),
            p.code(currency)?,
            amount
        ],
    )?;
    Ok(())
}
fn item(
    p: &mut Projection<'_>,
    o: &Origin<'_>,
    path: &str,
    role: &str,
    pos: usize,
    value: &Value,
    threshold: Option<i64>,
) -> Result<()> {
    let Some(reference) = string(value).or_else(|| string(&value["Item"])) else {
        return Ok(());
    };
    p.link(
        o,
        path,
        &Value::from(reference),
        ("Item", "Name"),
        false,
        false,
    )?;
    p.db.execute(
        "INSERT INTO catalogue_item_links VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            o.family,
            o.id,
            role,
            pos as i64,
            o.source,
            o.row as i64,
            path,
            reference,
            p.item(Some(reference)),
            number(value, "Count"),
            number(value, "EnchantedLevel"),
            threshold
        ],
    )?;
    Ok(())
}
fn root(p: &Projection<'_>, o: &Origin<'_>, row: &Value) -> Result<()> {
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
type LocalizedFields = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);
fn names(
    p: &Projection<'_>,
    o: &Origin<'_>,
    row: &Value,
    name: &str,
    desc: &str,
) -> Result<LocalizedFields> {
    let (en, loc) = p.named_text(o, "name", string(&row[name]["Key"]))?;
    let (de, dl) = p.named_text(o, "description", string(&row[desc]["Key"]))?;
    Ok((en, loc, de, dl))
}
fn image(
    p: &Projection<'_>,
    o: &Origin<'_>,
    reference: &Value,
    images: &projection::ImageIndex,
) -> Result<()> {
    projection::attach_icon(p.db, o.family, o.id, string(reference), images)?;
    Ok(())
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
    groups(&mut p, &images)?;
    achievements(&mut p, &images)?;
    titles(&mut p)?;
    seasons(&mut p)?;
    passes(&mut p)?;
    supplies(&mut p)?;
    journals(&mut p)?;
    for (subject, required) in [
        ("achievements", &SOURCES[..7]),
        ("journals", &SOURCES[7..8]),
        ("titles", &SOURCES[8..10]),
        ("passes", &SOURCES[10..12]),
        ("seasons", &SOURCES[12..17]),
        ("supply_requests", &SOURCES[17..]),
    ] {
        let absent: Vec<_> = required
            .iter()
            .filter(|s| !batch.contains_key(**s))
            .copied()
            .collect();
        let families: Vec<_> = ROOTS
            .iter()
            .filter(|(source, _, _)| required.contains(source))
            .map(|(_, family, _)| *family)
            .collect();
        let mut unresolved = 0i64;
        for family in families {
            unresolved+=db.query_row("SELECT count(*) FROM catalogue_references WHERE family=?1 AND resolution_state NOT IN('resolved','group')",[family],|r|r.get::<_,i64>(0))?;
        }
        coverage(
            db,
            &format!("{subject}.projection"),
            if required.iter().any(|s| batch.contains_key(*s)) {
                "partial"
            } else {
                "not_researched"
            },
            &format!(
                "{} undecoded sources ({absent:?}); {unresolved} unresolved root references; complete definitions and child diagnostics retained; progress/eligibility, opaque objectives, date conversion and probability scales not inferred",
                absent.len()
            ),
        )?;
    }
    coverage(
        db,
        "passes.track_classification",
        "uninterpreted",
        "bIsPremium, unlock costs, items and conditions remain separate source fields; a paid track may have bIsPremium=false",
    )?;
    coverage(
        db,
        "supply_requests.eligibility",
        "uninterpreted",
        "all matching GroupName variants retained with race/minimum level; TargetCount and per-item Count remain distinct; no reset period inferred from technical names",
    )?;
    coverage(
        db,
        "journals.coordinates",
        "uninterpreted",
        "default map view coordinates retained without assuming world units, a transform or collection-to-journal membership",
    )?;
    Ok(())
}

fn groups(p: &mut Projection<'_>, images: &projection::ImageIndex) -> Result<()> {
    for (pos, row) in rows(p.batch, "AchievementGroup").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("achievement_groups", id, "AchievementGroup", pos);
        root(p, &o, row)?;
        let (en, loc, de, dl) = names(p, &o, row, "Title", "Desc")?;
        let unlock = p.link(
            &o,
            "UnlockAchievementId",
            &row["UnlockAchievementId"],
            ("Achievement", "AchievementId"),
            false,
            false,
        )?;
        let unlock = resolved_id(p, "Achievement", unlock, "AchievementId");
        p.db.execute(
            "INSERT INTO achievement_groups VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                id,
                key(row, "Name")?,
                p.build,
                en,
                loc,
                de,
                dl,
                p.code(&row["Category"])?,
                p.code(&row["Type"])?,
                p.code(&row["CollectionMajorType"])?,
                p.race(row, "Race")?,
                number(row, "UnlockAchievementId"),
                unlock,
                row["bIsAccountShared"].as_bool()
            ],
        )?;
        p.codes(
            &o,
            row,
            &["Category", "Type", "CollectionMajorType", "Race"],
        )?;
        image(p, &o, &row["IconImageKey"], images)?;
    }
    for (pos, row) in rows(p.batch, "AchievementGroupSet").iter().enumerate() {
        let code = key(row, "Name")?;
        let id = dictionary(p.db, "achievement_group_sets", code)?;
        let o = origin("achievement_group_sets", id, "AchievementGroupSet", pos);
        root(p, &o, row)?;
        let (en, loc) = p.named_text(&o, "name", string(&row["Title"]["Key"]))?;
        p.db.execute(
            "UPDATE achievement_group_sets SET name_en=?1,name_localized=?2,race_id=?3 WHERE id=?4",
            params![en, loc, p.race(row, "Race")?, id],
        )?;
        for (i, reference) in list(row, "GroupList").iter().enumerate() {
            let target = p.link(
                &o,
                &format!("GroupList/{i}"),
                reference,
                ("AchievementGroup", "Name"),
                false,
                false,
            )?;
            p.db.execute(
                "INSERT INTO achievement_group_members VALUES(?1,?2,?3,?4)",
                params![
                    id,
                    i as i64,
                    string(reference),
                    resolved_id(p, "AchievementGroup", target, "ID")
                ],
            )?;
        }
    }
    for (pos, row) in rows(p.batch, "AchievementGroupReward").iter().enumerate() {
        let id = identity(row, "GroupId")?;
        let o = origin("achievement_groups", id, "AchievementGroupReward", pos);
        let target = p.link(
            &o,
            "GroupId",
            &row["GroupId"],
            ("AchievementGroup", "ID"),
            false,
            false,
        )?;
        if target.is_some() {
            p.source(o.family, id, o.source, pos)?;
        }
        for (i, reward) in list(row, "ItemRewards").iter().enumerate() {
            item(
                p,
                &o,
                &format!("ItemRewards/{i}/RewardItem"),
                "group_reward",
                i,
                &reward["RewardItem"],
                number(reward, "CompleteCount"),
            )?;
        }
    }
    Ok(())
}

fn achievements(p: &mut Projection<'_>, images: &projection::ImageIndex) -> Result<()> {
    let mut steps: BTreeMap<i64, Vec<(usize, &Value)>> = BTreeMap::new();
    for (pos, row) in rows(p.batch, "AchievementStep").iter().enumerate() {
        if let Some(id) = number(row, "AchievementId") {
            steps.entry(id).or_default().push((pos, row));
        }
    }
    for children in steps.values_mut() {
        children.sort_by_key(|(_, r)| (number(r, "Step"), number(r, "DataId")));
    }
    for (pos, row) in rows(p.batch, "Achievement").iter().enumerate() {
        let id = identity(row, "AchievementId")?;
        let o = origin("achievements", id, "Achievement", pos);
        root(p, &o, row)?;
        let group = p.link(
            &o,
            "GroupName",
            &row["GroupName"],
            ("AchievementGroup", "Name"),
            false,
            false,
        )?;
        let group = resolved_id(p, "AchievementGroup", group, "ID");
        let unlock = p.link(
            &o,
            "UnlockAchievementId",
            &row["UnlockAchievementId"],
            ("Achievement", "AchievementId"),
            false,
            false,
        )?;
        let unlock = resolved_id(p, "Achievement", unlock, "AchievementId");
        let first = steps
            .get(&id)
            .and_then(|r| r.first())
            .and_then(|(_, r)| string(&r["AchievementText"]));
        let text_pos = p.link(
            &o,
            "FirstStepText",
            &first.map(Value::from).unwrap_or_default(),
            ("AchievementString", "Name"),
            false,
            true,
        )?;
        let texts = text_pos.map(|i| &rows(p.batch, "AchievementString")[i]);
        let title = string(&row["CollectionTitle"]["Key"])
            .or_else(|| texts.and_then(|r| string(&r["AchievementTitle"]["Key"])));
        let desc = string(&row["CollectionDesc"]["Key"])
            .or_else(|| texts.and_then(|r| string(&r["ObjDesc"]["Key"])));
        let (en, loc) = p.named_text(&o, "name", title)?;
        let (de, dl) = p.named_text(&o, "description", desc)?;
        p.db.execute("INSERT INTO achievements VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",params![id,p.build,en,loc,de,dl,string(&row["GroupName"]),group,string(&row["SubGroupName"]),p.code(&row["ObjType"])?,serde_json::to_string(&row["ObjValues"])?,p.code(&row["Grade"])?,p.code(&row["ResetCycleType"])?,number(row,"UnlockAchievementId"),unlock,p.code(&row["CollectionContentType"])?,p.code(&row["CollectionSubType"])?,p.code(&row["CollectionMinorType"])?,number(row,"DisplayOrder")])?;
        p.codes(
            &o,
            row,
            &[
                "ObjType",
                "Grade",
                "ResetCycleType",
                "CollectionContentType",
                "CollectionSubType",
                "CollectionMinorType",
            ],
        )?;
        image(p, &o, &row["IconImageKey"], images)?;
    }
    for (pos, row) in rows(p.batch, "AchievementStep").iter().enumerate() {
        let id = identity(row, "DataId")?;
        let o = origin("achievement_steps", id, "AchievementStep", pos);
        root(p, &o, row)?;
        let parent = p.link(
            &o,
            "AchievementId",
            &row["AchievementId"],
            ("Achievement", "AchievementId"),
            false,
            false,
        )?;
        let parent = resolved_id(p, "Achievement", parent, "AchievementId");
        let txt = p.link(
            &o,
            "AchievementText",
            &row["AchievementText"],
            ("AchievementString", "Name"),
            false,
            true,
        )?;
        let text_row = txt
            .map(|i| &rows(p.batch, "AchievementString")[i])
            .unwrap_or(&Value::Null);
        let (en, loc, de, dl) = names(p, &o, text_row, "AchievementTitle", "ObjDesc")?;
        p.db.execute(
            "INSERT INTO achievement_steps VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                id,
                identity(row, "AchievementId")?,
                parent,
                number(row, "Step"),
                number(row, "Count"),
                string(&row["AchievementText"]),
                en,
                loc,
                de,
                dl
            ],
        )?;
        if let Some(parent) = parent {
            p.source("achievements", parent, "AchievementStep", pos)?;
            if let Some(txt) = txt {
                p.source("achievements", parent, "AchievementString", txt)?;
            }
        }
    }
    for (pos, row) in rows(p.batch, "AchievementReward").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("achievement_rewards", id, "AchievementReward", pos);
        root(p, &o, row)?;
        let parent = p.link(
            &o,
            "AchievementId",
            &row["AchievementId"],
            ("Achievement", "AchievementId"),
            false,
            false,
        )?;
        let parent = resolved_id(p, "Achievement", parent, "AchievementId");
        let step: Option<i64> =
            p.db.query_row(
                "SELECT id FROM achievement_steps WHERE achievement_id=?1 AND step_number=?2",
                params![parent, number(row, "Step")],
                |r| r.get(0),
            )
            .optional()?;
        p.db.execute(
            "INSERT INTO achievement_rewards VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                id,
                identity(row, "AchievementId")?,
                parent,
                number(row, "Step"),
                step,
                number(row, "RewardPoint")
            ],
        )?;
        if let Some(parent) = parent {
            p.source("achievements", parent, "AchievementReward", pos)?;
        }
        if step.is_none() {
            coverage(
                p.db,
                &format!("achievement_rewards.{id}.step"),
                "partial",
                "reward step is not resolved within its achievement; source association retained",
            )?;
        }
        for (i, reward) in list(row, "ItemRewards").iter().enumerate() {
            item(
                p,
                &o,
                &format!("ItemRewards/{i}"),
                "reward",
                i,
                reward,
                number(row, "Step"),
            )?;
        }
    }
    Ok(())
}

fn titles(p: &mut Projection<'_>) -> Result<()> {
    for (pos, row) in rows(p.batch, "TitleCategory").iter().enumerate() {
        let id = dictionary(p.db, "title_categories", key(row, "Name")?)?;
        let o = origin("title_categories", id, "TitleCategory", pos);
        root(p, &o, row)?;
        let (en, loc) = p.named_text(&o, "name", string(&row["Desc"]["Key"]))?;
        p.db.execute(
            "UPDATE title_categories SET name_en=?1,name_localized=?2,display_order=?3 WHERE id=?4",
            params![en, loc, number(row, "Order"), id],
        )?;
    }
    for (pos, row) in rows(p.batch, "Title").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("titles", id, "Title", pos);
        root(p, &o, row)?;
        let (en, loc, de, dl) = names(p, &o, row, "Desc", "DescLong")?;
        let category = p.link(
            &o,
            "CategoryName",
            &row["CategoryName"],
            ("TitleCategory", "Name"),
            false,
            true,
        )?;
        let category = category
            .map(|i| {
                dictionary(
                    p.db,
                    "title_categories",
                    key(&rows(p.batch, "TitleCategory")[i], "Name")?,
                )
            })
            .transpose()?;
        p.db.execute(
            "INSERT INTO titles VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![
                id,
                key(row, "Name")?,
                p.build,
                en,
                loc,
                de,
                dl,
                string(&row["CategoryName"]),
                category,
                p.code(&row["Grade"])?,
                p.race(row, "Race")?,
                p.code(&row["ApplyTargetType"])?,
                p.code(&row["EquipCategory"])?,
                number(row, "Order"),
                row["bShow"].as_bool(),
                row["bGetDisplay"].as_bool()
            ],
        )?;
        p.codes(
            &o,
            row,
            &["Grade", "Race", "ApplyTargetType", "EquipCategory"],
        )?;
        period(p, &o, "availability", row, "StartTime", "EndTime")?;
        for (field, role) in [("StatList", "possession"), ("EquipStatList", "equipped")] {
            for (i, stat) in list(row, field).iter().enumerate() {
                let code = key(stat, "Stat")?;
                let stat_id = dictionary(p.db, "stats", code)?;
                let label = code
                    .strip_prefix("EStat::")
                    .map(|s| format!("String_StatName_{s}_body"));
                p.db.execute(
                    "UPDATE stats SET name_en=?1,name_localized=?2 WHERE id=?3",
                    params![
                        text(p.en, label.as_deref()),
                        text(p.loc, label.as_deref()),
                        stat_id
                    ],
                )?;
                p.db.execute(
                    "INSERT INTO title_stats VALUES(?1,?2,?3,?4,?5)",
                    params![id, role, i as i64, stat_id, number(stat, "Value")],
                )?;
            }
        }
        // Decorations/backgrounds are distinct resources, not assumed icons.
    }
    for (pos, row) in rows(p.batch, "Item").iter().enumerate() {
        if string(&row["$variant"]["UseEffect"]) != Some("EItemUseEffect::GetTitle") {
            continue;
        }
        for (i, value) in list(&row["$variant"], "UseEffectValues").iter().enumerate() {
            let Some(target) = string(value).and_then(|s| s.parse::<i64>().ok()) else {
                continue;
            };
            let o = origin("title_grants", identity(row, "ID")?, "Item", pos);
            let found = p.link(
                &o,
                &format!("$variant/UseEffectValues/{i}"),
                &Value::from(target),
                ("Title", "ID"),
                true,
                false,
            )?;
            if let Some(target) = resolved_id(p, "Title", found, "ID") {
                item(
                    p,
                    &origin("titles", target, "Item", pos),
                    &format!("Name/@grant/{i}"),
                    "granting_item",
                    i,
                    &row["Name"],
                    None,
                )?;
            }
        }
    }
    Ok(())
}

fn seasons(p: &mut Projection<'_>) -> Result<()> {
    for (pos, row) in rows(p.batch, "SeasonScheduleGroup").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("season_schedule_groups", id, "SeasonScheduleGroup", pos);
        root(p, &o, row)?;
        p.db.execute(
            "INSERT INTO season_schedule_groups VALUES(?1,?2,?3)",
            params![id, key(row, "Name")?, p.build],
        )?;
    }
    for (pos, row) in rows(p.batch, "SeasonSchedule").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("seasons", id, "SeasonSchedule", pos);
        root(p, &o, row)?;
        let group = p.link(
            &o,
            "GroupName",
            &row["GroupName"],
            ("SeasonScheduleGroup", "Name"),
            false,
            false,
        )?;
        p.db.execute(
            "INSERT INTO seasons VALUES(?1,?2,?3,?4,?5)",
            params![
                id,
                p.build,
                string(&row["GroupName"]),
                resolved_id(p, "SeasonScheduleGroup", group, "ID"),
                number(row, "SeasonNumber")
            ],
        )?;
        period(p, &o, "season", row, "StartDate", "EndDate")?;
        period(p, &o, "rewards", row, "RewardStartDate", "RewardEndDate")?;
    }
    for (pos, row) in rows(p.batch, "SeasonContents").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("season_contents", id, "SeasonContents", pos);
        root(p, &o, row)?;
        let group = p.link(
            &o,
            "ScheduleGroupName",
            &row["ScheduleGroupName"],
            ("SeasonScheduleGroup", "Name"),
            false,
            false,
        )?;
        let pickup = p.link(
            &o,
            "PickupContents",
            &row["PickupContents"],
            ("SeasonPickupContents", "Name"),
            true,
            true,
        )?;
        p.db.execute(
            "INSERT INTO season_contents VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                id,
                p.build,
                p.code(&row["ContentsType"])?,
                number(row, "SeasonNumber"),
                string(&row["ScheduleGroupName"]),
                resolved_id(p, "SeasonScheduleGroup", group, "ID"),
                string(&row["PickupContents"]),
                resolved_id(p, "SeasonPickupContents", pickup, "ID")
            ],
        )?;
        p.codes(&o, row, &["ContentsType"])?;
        let schedule = p.group(
            &o,
            "ScheduleGroupName/@schedules",
            &row["ScheduleGroupName"],
            ("SeasonSchedule", "GroupName"),
            true,
            false,
        )?;
        for i in schedule {
            let schedule = &rows(p.batch, "SeasonSchedule")[i];
            if number(schedule, "SeasonNumber") == number(row, "SeasonNumber") {
                p.db.execute(
                    "INSERT INTO season_content_schedules VALUES(?1,?2)",
                    params![id, identity(schedule, "ID")?],
                )?;
            }
        }
        if string(&row["ContentsType"]) == Some("ESeasonContentsType::SeasonMission")
            && let Some(i) = pickup
        {
            let pickup_row = i;
            let definition = &rows(p.batch, "SeasonPickupContents")[i];
            for (field, race) in [("LightContentsNameList", 2), ("DarkContentsNameList", 3)] {
                for (i, reference) in list(definition, field).iter().enumerate() {
                    mission_link(
                        p,
                        &o,
                        &origin(o.family, id, "SeasonPickupContents", pickup_row),
                        race,
                        i,
                        reference,
                        field,
                    )?;
                }
            }
        }
    }
    for (pos, row) in rows(p.batch, "EventMission").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("event_missions", id, "EventMission", pos);
        root(p, &o, row)?;
        let (en, loc) = p.named_text(&o, "name", string(&row["Title"]["Key"]))?;
        let season = p.link(
            &o,
            "SeasonId",
            &row["SeasonId"],
            ("SeasonSchedule", "ID"),
            true,
            false,
        )?;
        p.db.execute(
            "INSERT INTO event_missions VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                id,
                string(&row["Name"]),
                p.build,
                en,
                loc,
                number(row, "SeasonId"),
                resolved_id(p, "SeasonSchedule", season, "ID"),
                p.code(&row["SeasonContentsType"])?,
                number(row, "TabOrder")
            ],
        )?;
        period(
            p,
            &o,
            "activation",
            row,
            "ActivationStartDate",
            "ActivationEndDate",
        )?;
        for (field, race) in [
            ("LightAchievementGroupSetName", 2),
            ("DarkAchievementGroupSetName", 3),
        ] {
            if let Some(reference) = string(&row[field]) {
                mission_link(p, &o, &o, race, 0, &Value::from(reference), field)?;
            }
        }
    }
    Ok(())
}

fn mission_link(
    p: &mut Projection<'_>,
    owner: &Origin<'_>,
    source: &Origin<'_>,
    race: i64,
    pos: usize,
    reference: &Value,
    field: &str,
) -> Result<()> {
    let Some(reference) = string(reference) else {
        return Ok(());
    };
    // Mission pickup lists mix group-set names and numeric achievement IDs.
    // Each form has an explicit encoding; neither is inferred by name prefixes.
    let (group, achievement) = if let Ok(id) = reference.parse::<i64>() {
        let target = p.link(
            source,
            &format!("{field}/{pos}"),
            &Value::from(id),
            ("Achievement", "AchievementId"),
            true,
            false,
        )?;
        (None, resolved_id(p, "Achievement", target, "AchievementId"))
    } else {
        let target = p.link(
            source,
            &format!("{field}/{pos}"),
            &Value::from(reference),
            ("AchievementGroupSet", "Name"),
            true,
            true,
        )?;
        (
            target
                .map(|i| {
                    dictionary(
                        p.db,
                        "achievement_group_sets",
                        key(&rows(p.batch, "AchievementGroupSet")[i], "Name")?,
                    )
                })
                .transpose()?,
            None,
        )
    };
    p.db.execute(
        "INSERT INTO mission_group_sets VALUES(?1,?2,?3,?4,?5,?6,?7)",
        params![
            owner.family,
            owner.id,
            race,
            pos as i64,
            reference,
            group,
            achievement
        ],
    )?;
    Ok(())
}

fn passes(p: &mut Projection<'_>) -> Result<()> {
    for (pos, row) in rows(p.batch, "BattlePassGroup").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("battle_pass_groups", id, "BattlePassGroup", pos);
        root(p, &o, row)?;
        let (en, loc) = p.named_text(&o, "name", string(&row["Title"]["Key"]))?;
        let season = p.link(
            &o,
            "SeasonId",
            &row["SeasonId"],
            ("SeasonSchedule", "ID"),
            true,
            false,
        )?;
        let purchase = p.link(
            &o,
            "ExpPurchaseItemName",
            &row["ExpPurchaseItemName"],
            ("Item", "Name"),
            false,
            false,
        )?;
        p.db.execute("INSERT INTO battle_pass_groups VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",params![id,key(row,"Name")?,p.build,en,loc,p.race(row,"Race")?,number(row,"SeasonId"),resolved_id(p,"SeasonSchedule",season,"ID"),p.code(&row["SeasonContentsType"])?,number(row,"BuyLimitCount"),p.code(&row["BuyLimitTargetType"])?,number(row,"ExpLimitValue"),p.code(&row["ExpLimitType"])?,string(&row["ExpPurchaseItemName"]),resolved_id(p,"Item",purchase,"ID"),number(row,"TabOrder")])?;
        period(
            p,
            &o,
            "activation",
            row,
            "ActivationStartDate",
            "ActivationEndDate",
        )?;
        period(
            p,
            &o,
            "subscription",
            row,
            "SubscriptionStartDate",
            "SubscriptionEndDate",
        )?;
        for (i, value) in list(row, "ExpCosts").iter().enumerate() {
            cost(
                p,
                &o,
                &format!("ExpCosts/{i}"),
                Some(i),
                &value["CostType"],
                number(value, "CostCount"),
            )?;
        }
        p.codes(
            &o,
            row,
            &[
                "Race",
                "SeasonContentsType",
                "BuyLimitTargetType",
                "ExpLimitType",
            ],
        )?;
    }
    for (pos, row) in rows(p.batch, "BattlePass").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("battle_passes", id, "BattlePass", pos);
        root(p, &o, row)?;
        let (en, loc) = p.named_text(&o, "name", string(&row["Title"]["Key"]))?;
        let group = p.link(
            &o,
            "GroupName",
            &row["GroupName"],
            ("BattlePassGroup", "Name"),
            false,
            false,
        )?;
        let exp = p.link(
            &o,
            "ExpAchievementId",
            &row["ExpAchievementId"],
            ("Achievement", "AchievementId"),
            false,
            true,
        )?;
        let exp = resolved_id(p, "Achievement", exp, "AchievementId");
        let quest = p.link(
            &o,
            "UnlockQuestName",
            &row["UnlockQuestName"],
            ("Quest", "Name"),
            false,
            false,
        )?;
        let abnormal = p.link(
            &o,
            "AbnormalId",
            &row["AbnormalId"],
            ("SkillAbnormal", "ID"),
            false,
            false,
        )?;
        p.db.execute(
            "INSERT INTO battle_passes VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
            params![
                id,
                key(row, "Name")?,
                p.build,
                en,
                loc,
                string(&row["GroupName"]),
                resolved_id(p, "BattlePassGroup", group, "ID"),
                number(row, "ExpAchievementId"),
                exp,
                row["bIsPremium"].as_bool(),
                number(row, "UnlockLevel"),
                string(&row["UnlockQuestName"]),
                resolved_id(p, "Quest", quest, "ID"),
                number(row, "AbnormalId"),
                resolved_id(p, "SkillAbnormal", abnormal, "ID")
            ],
        )?;
        if row.get("UnlockCostCount").is_some() {
            cost(
                p,
                &o,
                "UnlockCostCount",
                None,
                &row["UnlockCostType"],
                number(row, "UnlockCostCount"),
            )?;
        }
        item(
            p,
            &o,
            "UnlockItemItemName",
            "unlock_item",
            0,
            &row["UnlockItemItemName"],
            None,
        )?;
        for (i, reward) in list(row, "ItemReward").iter().enumerate() {
            item(p, &o, &format!("ItemReward/{i}"), "reward", i, reward, None)?;
        }
        conditions(
            p,
            &o,
            row,
            &[
                "UnlockCombatPowerMin",
                "UnlockCombatPowerMax",
                "UnlockItemLevelMin",
                "UnlockItemLevelMax",
                "ActiveSubscribeTypes",
            ],
        )?;
        p.link(
            &o,
            "ContentsAchievementGroupSetName",
            &row["ContentsAchievementGroupSetName"],
            ("AchievementGroupSet", "Name"),
            true,
            true,
        )?;
        if let Some(exp) = exp {
            // Pass progression uses its explicit experience achievement and all
            // of that definition's levels/texts/rewards, not fabricated levels.
            let evidence:Vec<(String,i64)>=p.db.prepare("SELECT source_name,source_row FROM catalogue_entity_sources WHERE family='achievements' AND source_id=?1")?.query_map([exp],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?;
            for (source, pos) in evidence {
                p.source(o.family, id, &source, usize::try_from(pos)?)?;
            }
        }
    }
    Ok(())
}

fn supplies(p: &mut Projection<'_>) -> Result<()> {
    for (pos, row) in rows(p.batch, "PeriodItemCollection").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("supply_request_groups", id, "PeriodItemCollection", pos);
        root(p, &o, row)?;
        let (en, loc, de, dl) = names(p, &o, row, "Title", "Desc")?;
        p.db.execute(
            "INSERT INTO supply_request_groups VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                id,
                key(row, "GroupName")?,
                p.build,
                en,
                loc,
                de,
                dl,
                p.race(row, "Race")?,
                number(row, "LevelMin"),
                p.code(&row["TabType"])?,
                row["bShowDate"].as_bool()
            ],
        )?;
        p.codes(&o, row, &["Race", "TabType"])?;
    }
    for (pos, row) in rows(p.batch, "PeriodItemCollectionList").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("supply_requests", id, "PeriodItemCollectionList", pos);
        root(p, &o, row)?;
        let (en, loc, de, dl) = names(p, &o, row, "Title", "Desc")?;
        p.db.execute(
            "INSERT INTO supply_requests VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                id,
                p.build,
                string(&row["GroupName"]),
                en,
                loc,
                de,
                dl,
                number(row, "TargetCount"),
                number(row, "RewardExp")
            ],
        )?;
        for index in p.group(
            &o,
            "GroupName",
            &row["GroupName"],
            ("PeriodItemCollection", "GroupName"),
            false,
            false,
        )? {
            p.db.execute(
                "INSERT INTO supply_request_variants VALUES(?1,?2)",
                params![
                    id,
                    identity(&rows(p.batch, "PeriodItemCollection")[index], "ID")?
                ],
            )?;
        }
        for (field, role) in [("TargetItems", "requested_item"), ("RewardItems", "reward")] {
            for (i, value) in list(row, field).iter().enumerate() {
                item(p, &o, &format!("{field}/{i}"), role, i, value, None)?;
            }
        }
    }
    Ok(())
}

fn journals(p: &mut Projection<'_>) -> Result<()> {
    for (pos, row) in rows(p.batch, "Journal").iter().enumerate() {
        let id = identity(row, "ID")?;
        let o = origin("journals", id, "Journal", pos);
        root(p, &o, row)?;
        let map = p.link(&o, "MapId", &row["MapId"], ("Map", "ID"), false, false)?;
        let name = map.and_then(|i| string(&rows(p.batch, "Map")[i]["Desc"]["Key"]));
        let (en, loc) = p.named_text(&o, "name", name)?;
        if let Some(map) = map {
            p.source(o.family, id, "Map", map)?;
        }
        p.db.execute(
            "INSERT INTO journals VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                id,
                string(&row["Name"]),
                p.build,
                number(row, "MapId"),
                en,
                loc,
                p.race(row, "Race")?,
                number(row, "Order"),
                row["DefaultMapX"].as_f64(),
                row["DefaultMapY"].as_f64(),
                row["DefaultMapZoomLevel"].as_f64()
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
    let families: Vec<_> = ROOTS.iter().map(|(_, family, _)| *family).collect();
    let mut result = catalogue_projection::fingerprints(db, batch, en, loc, &families)?;
    // Title effects use shared localized stats; grants use natural item names,
    // excluding source row order and unrelated granting-item properties.
    for localized in [false, true] {
        let mut contexts: BTreeMap<i64, Vec<Value>> = BTreeMap::new();
        for row in db.prepare("SELECT t.title_id,t.role,t.position,s.code,t.value_raw,CASE WHEN ?1 THEN s.name_localized ELSE s.name_en END FROM title_stats t JOIN stats s ON s.id=t.stat_id")?.query_map([localized],|r|Ok((r.get::<_,i64>(0)?,serde_json::json!([r.get::<_,String>(1)?,r.get::<_,i64>(2)?,r.get::<_,String>(3)?,r.get::<_,Option<i64>>(4)?,r.get::<_,Option<String>>(5)?]))))? {
            let (id,value)=row?;contexts.entry(id).or_default().push(value);
        }
        for row in db.prepare("SELECT source_id,item_reference FROM catalogue_item_links WHERE family='titles' AND role='granting_item'")?.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?)))? {
            let (id,reference)=row?;contexts.entry(id).or_default().push(Value::from(reference));
        }
        for (id, mut values) in contexts {
            values.sort_by_cached_key(Value::to_string);
            if let Some((base, translation)) = result.get_mut(&("titles".into(), id)) {
                let target = if localized { translation } else { base };
                *target = hash(&serde_json::to_vec(&(&target, values))?);
            }
        }
    }
    Ok(result)
}
