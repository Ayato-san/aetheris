//! #81: world definitions and source placements. No live-state, probabilities,
//! coordinate conversion, NPC combat formula or nearest-route matching is inferred.
use super::catalogue_projection::{Origin, Projection, list, rows};
use super::*;
use serde_json::json;

pub(super) const SOURCES: &[&str] = &[
    "Map",
    "Subzone",
    "Dungeon",
    "DungeonGroup",
    "PartyDungeonLevel",
    "WorldMapUIRegion",
    "WorldMapMapList",
    "WorldMapFieldNamed",
    "NpcData",
    "NpcLoot",
    "NpcAggroData",
    "EnvObjData",
    "GatherSource",
    "GatherSourceMaterial",
    "GatherMastery",
    "GatherSkill",
    "GatherSkillEffect",
    "PeriodSpawn",
];
pub(super) const ROOTS: &[(&str, &str, &str)] = &[
    ("Map", "world_maps", "world_maps"),
    ("Subzone", "world_subzones", "world_subzones"),
    ("Dungeon", "world_dungeons", "world_dungeons"),
    (
        "DungeonGroup",
        "world_dungeon_groups",
        "world_dungeon_groups",
    ),
    ("NpcData", "world_npcs", "world_npcs"),
    (
        "EnvObjData",
        "world_environment_objects",
        "world_environment_objects",
    ),
    (
        "GatherSource",
        "world_gather_sources",
        "world_gather_sources",
    ),
    ("GatherSkill", "world_gather_skills", "world_gather_skills"),
    (
        "PeriodSpawn",
        "world_periodic_spawns",
        "world_periodic_spawns",
    ),
    ("@map-resources", "world_map_assets", "world_map_assets"),
];
fn origin<'a>(family: &'a str, id: i64, source: &'a str, row: usize) -> Origin<'a> {
    Origin {
        family,
        id,
        source,
        row,
    }
}
fn evidence(p: &Projection<'_>, o: &Origin<'_>, r: &Value) -> Result<()> {
    p.source(o.family, o.id, o.source, o.row)?;
    for (path, key) in p.texts(o, r)? {
        p.db.execute(
            "INSERT OR IGNORE INTO catalogue_text_values VALUES(?1,?2,?3,?4,?5)",
            params![
                o.family,
                o.id,
                path,
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
    r: &Value,
    key: &str,
) -> Result<(Option<String>, Option<String>)> {
    p.named_text(o, key, string(&r[key]["Key"]))
}
fn link(
    p: &mut Projection<'_>,
    o: &Origin<'_>,
    r: &Value,
    key: &str,
    target: &str,
    target_field: &str,
    dependency: bool,
) -> Result<Option<i64>> {
    let pos = p.link(o, key, &r[key], (target, target_field), false, dependency)?;
    Ok(pos.and_then(|i| number(&rows(p.batch, target)[i], "ID")))
}
fn row_id(r: &Value) -> Result<i64> {
    catalogue_projection::id(r)
}
fn stable(parts: &[&str]) -> i64 {
    let digest = Sha256::digest(serde_json::to_vec(parts).expect("string serialization"));
    i64::from_le_bytes(digest[..8].try_into().unwrap()) & i64::MAX
}
fn existing(db: &Connection, table: &str, id: Option<i64>) -> Result<Option<i64>> {
    Ok(id
        .filter(|n| *n > 0)
        .map(|n| {
            db.query_row(&format!("SELECT id FROM {table} WHERE id=?1"), [n], |r| {
                r.get(0)
            })
            .optional()
        })
        .transpose()?
        .flatten())
}
fn item(p: &mut Projection<'_>, o: &Origin<'_>, path: &str, v: &Value) -> Result<Option<i64>> {
    let pos = p.link(o, path, v, ("Item", "Name"), false, false)?;
    Ok(pos.and_then(|i| number(&rows(p.batch, "Item")[i], "ID")))
}
fn maps(p: &mut Projection<'_>) -> Result<()> {
    for (pos, r) in rows(p.batch, "Map").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_maps", id, "Map", pos);
        evidence(p, &o, r)?;
        let (en, loc) = named(p, &o, r, "Desc")?;
        let (de, dl) = named(p, &o, r, "DescLong")?;
        let base = link(p, &o, r, "BaseMapId", "Map", "ID", false)?;
        let subzone = link(p, &o, r, "DefaultSubzoneID", "Subzone", "ID", false)?;
        p.db.execute(
            "INSERT INTO world_maps VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![
                id,
                p.build,
                string(&r["Name"]),
                en,
                loc,
                de,
                dl,
                p.code(&r["MapType"])?,
                p.race(r, "MapRace")?,
                number(r, "BaseMapId"),
                base,
                number(r, "DefaultSubzoneID"),
                subzone,
                string(&r["BaseDir"]),
                string(&r["Filename"]),
                string(&r["EntranceName"])
            ],
        )?;
    }
    for (pos, r) in rows(p.batch, "Subzone").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_subzones", id, "Subzone", pos);
        evidence(p, &o, r)?;
        let (en, loc) = named(p, &o, r, "DisplayName")?;
        let map = link(p, &o, r, "MapId", "Map", "ID", false)?;
        p.db.execute(
            "INSERT INTO world_subzones VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![
                id,
                p.build,
                string(&r["Name"]),
                en,
                loc,
                number(r, "MapId"),
                map,
                number(r, "ZoneId"),
                p.code(&r["SubzoneType"])?,
                p.code(&r["PkType"])?,
                string(&r["WorldMapArea"]),
                string(&r["WorldMapNameActor"]),
                r["bMapEnabled"].as_bool()
            ],
        )?;
    }
    for (pos, r) in rows(p.batch, "DungeonGroup").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_dungeon_groups", id, "DungeonGroup", pos);
        evidence(p, &o, r)?;
        let (en, loc) = named(p, &o, r, "Desc")?;
        let (de, dl) = named(p, &o, r, "DescLong")?;
        p.db.execute(
            "INSERT INTO world_dungeon_groups VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                id,
                p.build,
                string(&r["Name"]),
                en,
                loc,
                de,
                dl,
                number(&r["StartDate"], "Ticks"),
                number(r, "Order"),
                r["Show"].as_bool()
            ],
        )?;
    }
    Ok(())
}
fn dungeons(p: &mut Projection<'_>, images: &projection::ImageIndex) -> Result<()> {
    for (pos, r) in rows(p.batch, "Dungeon").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_dungeons", id, "Dungeon", pos);
        evidence(p, &o, r)?;
        let (en, loc) = named(p, &o, r, "Title")?;
        let (de, dl) = named(p, &o, r, "Desc")?;
        let map = link(p, &o, r, "MapId", "Map", "ID", false)?;
        let linked = link(p, &o, r, "LinkedMap", "Map", "Name", false)?;
        let group = link(p, &o, r, "DungeonGroupName", "DungeonGroup", "Name", true)?;
        let thumb = projection::attach_icon(
            p.db,
            "world_dungeons",
            id,
            string(&r["ImageThumnailRes"]),
            images,
        )?;
        let bg = projection::attach_icon(
            p.db,
            "world_dungeon_backgrounds",
            id,
            string(&r["ImageBGRes"]),
            images,
        )?;
        p.db.execute("INSERT INTO world_dungeons VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)",params![id,p.build,string(&r["Name"]),en,loc,de,dl,number(r,"MapId"),map,linked,group,p.code(&r["DungeonType"])?,p.code(&r["DungeonSubType"])?,p.code(&r["DungeonDifficulty"])?,number(r,"RecommendLevel"),number(r,"RecommendMinMember"),number(r,"RecommendMaxMember"),number(r,"Cooltime"),number(r,"DeathLimit"),number(r,"RebirthLimitCount"),thumb,bg])?;
    }
    p.db.execute("INSERT INTO world_dungeon_loot SELECT d.id,l.id FROM world_dungeons d JOIN loot_tables l ON l.source_name='Dungeon' AND l.source_key=CAST(d.id AS TEXT)",[])?;
    for (pos, r) in rows(p.batch, "PartyDungeonLevel").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_dungeon_levels", id, "PartyDungeonLevel", pos);
        let dungeon = link(p, &o, r, "DungeonName", "Dungeon", "Name", false)?;
        p.db.execute(
            "INSERT INTO world_dungeon_levels VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![
                id,
                string(&r["DungeonName"]),
                dungeon,
                string(&r["Group"]),
                number(r, "DungeonLevel"),
                number(r, "EquipItemLevel"),
                p.code(&r["UnlockType"])?
            ],
        )?;
        if let Some(dungeon) = dungeon {
            p.source("world_dungeons", dungeon, "PartyDungeonLevel", pos)?;
        }
    }
    for (pos, r) in rows(p.batch, "WorldMapUIRegion").iter().enumerate() {
        let id = stable(&[
            "WorldMapUIRegion",
            &r["MapId"].to_string(),
            &r["SubzoneId"].to_string(),
            &r["DungeonId"].to_string(),
            r["WorldMapNameActor"].as_str().unwrap_or(""),
        ]);
        let o = origin("world_ui_regions", id, "WorldMapUIRegion", pos);
        evidence(p, &o, r)?;
        let (en, loc) = named(p, &o, r, "Desc")?;
        let (de, dl) = named(p, &o, r, "DetailDesc")?;
        let map = link(p, &o, r, "MapId", "Map", "ID", false)?;
        let sub = link(p, &o, r, "SubzoneId", "Subzone", "ID", false)?;
        let dungeon = link(p, &o, r, "DungeonId", "Dungeon", "ID", false)?;
        p.db.execute("INSERT INTO world_ui_regions VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",params![id,number(r,"MapId"),map,number(r,"SubzoneId"),sub,number(r,"DungeonId"),dungeon,en,loc,de,dl,p.code(&r["WorldMapInfoType"])?,p.code(&r["IconType"])?,string(&r["WorldMapNameActor"]),r["bEnableWorldMap"].as_bool(),r["bTeleportAvailable"].as_bool()])?;
        if let Some(map) = map {
            p.source("world_maps", map, "WorldMapUIRegion", pos)?;
        }
    }
    for (pos, r) in rows(p.batch, "WorldMapMapList").iter().enumerate() {
        let o = origin("world_map_categories", pos as i64, "WorldMapMapList", pos);
        let map = link(p, &o, r, "MapId", "Map", "ID", false)?;
        let (en, loc) = named(p, &o, r, "Desc")?;
        p.db.execute(
            "INSERT INTO world_map_categories VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                map,
                number(r, "MapId").ok_or_else(|| invalid("missing map category ID"))?,
                pos as i64,
                p.code(&r["Category"])?,
                en,
                loc
            ],
        )?;
        if let Some(map) = map {
            p.source("world_maps", map, "WorldMapMapList", pos)?;
        }
    }
    Ok(())
}
fn npcs(p: &mut Projection<'_>, images: &projection::ImageIndex) -> Result<()> {
    for (pos, r) in rows(p.batch, "NpcData").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_npcs", id, "NpcData", pos);
        evidence(p, &o, r)?;
        let (en, loc) = named(p, &o, r, "Desc")?;
        let (de, dl) = named(p, &o, r, "SubDesc")?;
        p.db.execute("INSERT INTO world_npcs VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",params![id,p.build,string(&r["Name"]),en,loc,de,dl,p.code(&r["NpcType"])?,p.code(&r["NpcSubType"])?,p.code(&r["RelationshipEntity"])?,p.code(&r["NpcCreatureType"])?,number(r,"Level"),number(r,"Grade"),string(&r["Tribe"]),string(&r["ResourceKey"]),p.code(&r["LevelScaleType"])?,p.code(&r["WorldLevelScaleType"])?,r["bCanBeAttacked"].as_bool(),r["bNamed"].as_bool()])?;
        projection::attach_icon(p.db, "world_npcs", id, string(&r["Portrait"]), images)?;
        for (i, v) in list(r, "ActiveSkillList").iter().enumerate() {
            let path = format!("ActiveSkillList/{i}");
            let found = p.link(&o, &path, v, ("Skill", "ID"), false, false)?;
            let skill = found.and_then(|i| number(&rows(p.batch, "Skill")[i], "ID"));
            let reference =
                number(&json!({"v":v}), "v").ok_or_else(|| invalid("missing NPC skill ID"))?;
            p.db.execute(
                "INSERT INTO world_npc_skills VALUES(?1,?2,?3,?4)",
                params![id, i as i64, reference, skill],
            )?;
        }
    }
    for (pos, r) in rows(p.batch, "NpcLoot").iter().enumerate() {
        let reference = number(r, "NpcId").ok_or_else(|| invalid("missing NPC loot ID"))?;
        let o = origin("world_npc_loot", reference, "NpcLoot", pos);
        let npc = link(p, &o, r, "NpcId", "NpcData", "ID", false)?;
        let loot: Option<i64> =
            p.db.query_row(
                "SELECT id FROM loot_tables WHERE source_name='NpcLoot' AND source_key=?1",
                [reference.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        if let (Some(npc), Some(loot)) = (npc, loot) {
            let soul = link(p, &o, r, "VehicleSoulItemId", "Item", "ID", false)?;
            p.db.execute(
                "INSERT INTO world_npc_loot VALUES(?1,?2,?3,?4)",
                params![npc, loot, number(r, "VehicleSoulItemId"), soul],
            )?;
        }
    }
    Ok(())
}
fn gathering(p: &mut Projection<'_>, images: &projection::ImageIndex) -> Result<()> {
    for (pos, r) in rows(p.batch, "GatherSource").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_gather_sources", id, "GatherSource", pos);
        evidence(p, &o, r)?;
        let quest = link(p, &o, r, "QuestStepId", "QuestStep", "ID", false)?;
        let conversion = link(
            p,
            &o,
            r,
            "ConversionRecipe",
            "ConversionRecipe",
            "Name",
            false,
        )?;
        p.db.execute(
            "INSERT INTO world_gather_sources VALUES(?1,?2,?3,NULL,NULL,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                id,
                p.build,
                string(&r["Name"]),
                p.code(&r["SourceType"])?,
                p.code(&r["SourceGrade"])?,
                number(r, "MasteryLevel"),
                string(&r["MaterialGroup"]),
                number(r, "QuestStepId"),
                quest,
                conversion.map(|_| "ConversionRecipe"),
                conversion
            ],
        )?;
        let materials = p.group(
            &o,
            "MaterialGroup",
            &r["MaterialGroup"],
            ("GatherSourceMaterial", "MaterialGroup"),
            false,
            true,
        )?;
        for index in materials {
            let material = &rows(p.batch, "GatherSourceMaterial")[index];
            let mo = origin(o.family, id, "GatherSourceMaterial", index);
            let key =
                number(material, "GroupKey").ok_or_else(|| invalid("missing gather GroupKey"))?;
            p.db.execute(
                "INSERT INTO world_gather_materials VALUES(?1,?2,?3)",
                params![id, key, number(material, "GatherGauge")],
            )?;
            gather_result(
                p,
                &mo,
                key,
                "main",
                0,
                "MaterialName",
                &material["MaterialName"],
                number(material, "MaterialCount"),
            )?;
            for (field, role) in [
                ("AddMaterialList", "additional"),
                ("FailMaterialList", "failure"),
            ] {
                for (i, v) in list(material, field).iter().enumerate() {
                    gather_result(
                        p,
                        &mo,
                        key,
                        role,
                        i,
                        &format!("{field}/{i}/Item"),
                        &v["Item"],
                        number(v, "Count"),
                    )?;
                }
            }
        }
        gather_result(
            p,
            &o,
            -1,
            "quest",
            0,
            "QuestMaterial",
            &r["QuestMaterial"],
            number(r, "QuestMaterialCount"),
        )?;
        // A display name may reuse the material only if all principal results
        // identify that same item; do not arbitrarily choose an alternative.
        let material:Option<i64>=p.db.query_row("SELECT CASE WHEN count(DISTINCT item_id)=1 AND sum(item_id IS NULL)=0 THEN min(item_id) END FROM world_gather_results WHERE source_id=?1 AND role='main'",[id],|r|r.get(0))?;
        if let Some(material) = material {
            p.db.execute("UPDATE world_gather_sources SET name_en=(SELECT name_en FROM items WHERE id=?1),name_localized=(SELECT name_localized FROM items WHERE id=?1) WHERE id=?2",params![material,id])?;
            if let Some(item) = rows(p.batch, "Item")
                .iter()
                .find(|r| number(r, "ID") == Some(material))
            {
                p.named_text(&o, "material_label", string(&item["Desc"]["Key"]))?;
            }
        }
    }
    for r in rows(p.batch, "GatherMastery") {
        p.db.execute(
            "INSERT INTO world_gather_mastery VALUES(?1,?2,?3,?4,?5)",
            params![
                number(r, "Level").ok_or_else(|| invalid("missing gather mastery level"))?,
                p.code(&r["Grade"])?,
                number(r, "NeedExp"),
                number(r, "BonusSkillPoint"),
                number(r, "BonusSpecializedPoint")
            ],
        )?;
    }
    for (pos, r) in rows(p.batch, "GatherSkill").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_gather_skills", id, "GatherSkill", pos);
        evidence(p, &o, r)?;
        let (en, loc) = named(p, &o, r, "SkillDesc")?;
        let (de, dl) = named(p, &o, r, "LongDesc")?;
        p.db.execute(
            "INSERT INTO world_gather_skills VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                id,
                p.build,
                string(&r["Name"]),
                en,
                loc,
                de,
                dl,
                p.code(&r["SkillCategory"])?,
                p.code(&r["ConditionType"])?,
                number(r, "DisplayOrder")
            ],
        )?;
        projection::attach_icon(
            p.db,
            "world_gather_skills",
            id,
            string(&r["SkillIcon"]),
            images,
        )?;
    }
    for (pos, r) in rows(p.batch, "GatherSkillEffect").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_gather_effects", id, "GatherSkillEffect", pos);
        let skill = link(p, &o, r, "SkillName", "GatherSkill", "Name", false)?;
        p.db.execute(
            "INSERT INTO world_gather_effects VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                id,
                string(&r["SkillName"]),
                skill,
                number(r, "SkillLevel"),
                p.code(&r["EffectType"])?,
                number(r, "ActiveProb"),
                number(r, "NeedSkillPoint"),
                number(r, "NeedSpecializedPoint")
            ],
        )?;
        if let Some(skill) = skill {
            p.source("world_gather_skills", skill, "GatherSkillEffect", pos)?;
        }
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn gather_result(
    p: &mut Projection<'_>,
    o: &Origin<'_>,
    key: i64,
    role: &str,
    position: usize,
    path: &str,
    value: &Value,
    quantity: Option<i64>,
) -> Result<()> {
    if let Some(reference) = string(value) {
        let item = item(p, o, path, value)?;
        p.db.execute(
            "INSERT INTO world_gather_results VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![o.id, key, role, position as i64, reference, item, quantity],
        )?;
    }
    Ok(())
}
fn objects_and_periods(p: &mut Projection<'_>, images: &projection::ImageIndex) -> Result<()> {
    for (pos, r) in rows(p.batch, "EnvObjData").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_environment_objects", id, "EnvObjData", pos);
        evidence(p, &o, r)?;
        let (en, loc) = named(p, &o, r, "Desc")?;
        let (de, dl) = named(p, &o, r, "InteractDesc")?;
        let gather = if string(&r["Usage"]) == Some("EEnvObjectUsage::GatherSource") {
            link(p, &o, r, "UsageValue", "GatherSource", "Name", false)?
        } else {
            None
        };
        p.db.execute("INSERT INTO world_environment_objects VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",params![id,p.build,string(&r["Name"]),en,loc,de,dl,p.code(&r["Category"])?,p.code(&r["Usage"])?,string(&r["ResourceKey"]),number(r,"Level"),number(r,"ItemLevel"),number(r,"InteractTime"),number(r,"InteractRange"),r["bCanBeAttacked"].as_bool(),gather])?;
        for (i, c) in list(r, "InteractConditionList").iter().enumerate() {
            p.condition(&o, &format!("InteractConditionList/{i}"), &c["Type"], c)?;
        }
        projection::attach_icon(
            p.db,
            "world_environment_objects",
            id,
            string(&r["Portrait"]),
            images,
        )?;
    }
    for (pos, r) in rows(p.batch, "PeriodSpawn").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_periodic_spawns", id, "PeriodSpawn", pos);
        evidence(p, &o, r)?;
        let map = link(p, &o, r, "MapId", "Map", "ID", false)?;
        p.db.execute("INSERT INTO world_periodic_spawns VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",params![id,p.build,string(&r["Name"]),number(r,"MapId"),map,string(&r["SpawnerName"]),p.code(&r["SpawnType"])?,number(r,"Cycle"),number(&r["Time"],"Ticks"),r["bMonday"].as_bool(),r["bTuesday"].as_bool(),r["bWednesday"].as_bool(),r["bThursday"].as_bool(),r["bFriday"].as_bool(),r["bSaturday"].as_bool(),r["bSunday"].as_bool()])?;
    }
    for (pos, r) in rows(p.batch, "WorldMapFieldNamed").iter().enumerate() {
        let id = row_id(r)?;
        let o = origin("world_field_named", id, "WorldMapFieldNamed", pos);
        let map = link(p, &o, r, "MapId", "Map", "ID", false)?;
        let period = link(p, &o, r, "PeriodSpawnId", "PeriodSpawn", "ID", false)?;
        p.db.execute(
            "INSERT INTO world_field_named VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                id,
                number(r, "MapId"),
                map,
                string(&r["FieldNamedName"]),
                string(&r["FieldNamedSpawnerName"]),
                number(r, "PeriodSpawnId"),
                period,
                p.code(&r["SpawnType"])?
            ],
        )?;
    }
    Ok(())
}
fn assets(p: &mut Projection<'_>) -> Result<()> {
    for (name, t) in p
        .batch
        .iter()
        .filter(|(n, _)| super::super::map_resources::is_source(n))
    {
        let id = stable(&[name]);
        let kind = name.split('/').next().unwrap();
        let map_reference = t
            .layout
            .rows
            .first()
            .and_then(|r| number(r, if kind == "MapEvent" { "MapID" } else { "MapId" }));
        let map = existing(p.db, "world_maps", map_reference)?;
        p.db.execute(
            "INSERT INTO world_map_assets VALUES(?1,?2,?3,?4,?5,?6,?7,'source','uninterpreted')",
            params![
                id,
                p.build,
                format!("{name}.bin"),
                kind,
                if kind == "MapDataHierarchy" {
                    None
                } else {
                    map_reference
                },
                if kind == "MapDataHierarchy" {
                    None
                } else {
                    map
                },
                t.payload_sha256
            ],
        )?;
        for (pos, r) in t.layout.rows.iter().enumerate() {
            let o = origin("world_map_assets", id, name, pos);
            evidence(p, &o, r)?;
            if kind != "MapDataHierarchy" {
                link(
                    p,
                    &o,
                    r,
                    if kind == "MapEvent" { "MapID" } else { "MapId" },
                    "Map",
                    "ID",
                    false,
                )?;
            }
            match kind {
                "Map" => placements(p, &o, r, id)?,
                "MapEvent" => event(p, &o, r, "", id, 0)?,
                "WorldMap" => layout(p, &o, r, id)?,
                "MapDataHierarchy" => hierarchy_rows(p, &o, r, id)?,
                _ => (),
            }
        }
    }
    // Exact map + spawner name links only; several resource variants remain
    // several matches, never a fabricated single resolved live placement.
    p.db.execute("INSERT INTO world_periodic_spawner_links SELECT p.id,s.id FROM world_periodic_spawns p JOIN world_map_assets a ON a.map_reference=p.map_reference JOIN world_spawners s ON s.map_asset_id=a.id AND s.source_name=p.spawner_name",[])?;
    p.db.execute("INSERT INTO world_event_spawner_links SELECT c.map_asset_id,c.source_path,s.id FROM world_event_spawner_commands c JOIN world_map_assets e ON e.id=c.map_asset_id JOIN world_map_assets a ON a.kind='Map' AND a.map_reference=e.map_reference JOIN world_spawners s ON s.map_asset_id=a.id AND s.source_name=c.spawner_reference",[])?;
    Ok(())
}
fn layout(p: &mut Projection<'_>, o: &Origin<'_>, r: &Value, asset: i64) -> Result<()> {
    let candidate = p
        .link(
            o,
            "WorldMapDataTableKey",
            &r["WorldMapDataTableKey"],
            ("Map", "Name"),
            true,
            false,
        )?
        .and_then(|i| number(&rows(p.batch, "Map")[i], "ID"));
    let min = &r["WorldBoundBox"]["Min"];
    let max = &r["WorldBoundBox"]["Max"];
    p.db.execute("INSERT INTO world_map_layouts VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,'candidate')",params![asset,string(&r["WorldMapDataTableKey"]),candidate,number(&r["SectorSize"],"X"),number(&r["SectorSize"],"Y"),r["SectorPlaneSize"].as_f64(),number(r,"SectorTextureSize"),r["SectorWorldSize"].as_f64(),r["StartWorldLocation"]["X"].as_f64(),r["StartWorldLocation"]["Y"].as_f64(),min["X"].as_f64(),min["Y"].as_f64(),min["Z"].as_f64(),max["X"].as_f64(),max["Y"].as_f64(),max["Z"].as_f64(),r["WorldBoundBox"]["IsValid"].as_bool()])?;
    for entry in list(&r["PreviewLocations"], "Entries") {
        let key =
            string(&entry["Key"]).ok_or_else(|| invalid("missing map preview location key"))?;
        let v = &entry["Value"];
        p.db.execute(
            "INSERT INTO world_map_preview_locations VALUES(?1,?2,?3,?4,?5)",
            params![
                asset,
                key,
                v["X"].as_f64(),
                v["Y"].as_f64(),
                v["Z"].as_f64()
            ],
        )?;
    }
    Ok(())
}

fn placements(p: &mut Projection<'_>, o: &Origin<'_>, r: &Value, asset: i64) -> Result<()> {
    for (pos, s) in list(r, "SpawnInfoList").iter().enumerate() {
        let name = string(&s["Name"]).ok_or_else(|| invalid("missing map spawner name"))?;
        let id = stable(&[o.source, "spawn", name, &s["UniqueKey"].to_string()]);
        p.db.execute(
            "INSERT INTO world_spawners VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                id,
                asset,
                pos as i64,
                name,
                number(s, "UniqueKey"),
                string(&s["MaintainingTag"]),
                s["bSingleFixedRotation"].as_bool(),
                s["bIsDynamicTerrain"].as_bool()
            ],
        )?;
        let so = origin("world_spawners", id, o.source, o.row);
        p.source(so.family, id, o.source, o.row)?;
        for (field, kind, target) in [
            ("NpcIdList", "npc", "NpcData"),
            ("EnvObjIdList", "environment", "EnvObjData"),
        ] {
            for (i, v) in list(s, field).iter().enumerate() {
                let index = p.link(
                    &so,
                    &format!("SpawnInfoList/{pos}/{field}/{i}"),
                    v,
                    (target, "ID"),
                    false,
                    false,
                )?;
                let found = index.and_then(|i| number(&rows(p.batch, target)[i], "ID"));
                let reference = number(&json!({"v":v}), "v")
                    .ok_or_else(|| invalid("invalid spawn candidate ID"))?;
                p.db.execute(
                    "INSERT INTO world_spawn_candidates VALUES(?1,?2,?3,?4,?5,?6)",
                    params![
                        id,
                        kind,
                        i as i64,
                        reference,
                        if kind == "npc" { found } else { None },
                        if kind == "environment" { found } else { None }
                    ],
                )?;
            }
        }
        for (i, v) in list(s, "Positions").iter().enumerate() {
            let l = &v["Location"];
            let a = &v["Rotation"];
            p.db.execute(
                "INSERT INTO world_spawn_positions VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    id,
                    i as i64,
                    l["X"].as_f64(),
                    l["Y"].as_f64(),
                    l["Z"].as_f64(),
                    a["Pitch"].as_f64(),
                    a["Yaw"].as_f64(),
                    a["Roll"].as_f64()
                ],
            )?;
        }
    }
    for entry in list(&r["WaypointsDataMap"], "Entries") {
        let key = string(&entry["Key"]).ok_or_else(|| invalid("missing waypoint key"))?;
        let v = &entry["Value"];
        let id = stable(&[o.source, "route", key]);
        p.db.execute(
            "INSERT INTO world_routes VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                id,
                asset,
                key,
                p.code(&v["WaypointType"])?,
                p.code(&v["TraverseType"])?,
                number(v, "ID").map(|id| id.to_string()),
                number(v, "MoveType"),
                v["BaseMoveSpeed"].as_f64()
            ],
        )?;
        for (i, point) in list(v, "Points").iter().enumerate() {
            p.db.execute(
                "INSERT INTO world_route_points VALUES(?1,?2,?3,?4,?5)",
                params![
                    id,
                    i as i64,
                    point["X"].as_f64(),
                    point["Y"].as_f64(),
                    point["Z"].as_f64()
                ],
            )?;
        }
    }
    for (i, entry) in list(&r["SubzoneVolumeInfoMap"], "Entries")
        .iter()
        .enumerate()
    {
        let key = string(&entry["Key"]).ok_or_else(|| invalid("missing subzone volume key"))?;
        let v = &entry["Value"];
        let l = &v["Location"];
        let id = stable(&[o.source, "volume", key]);
        let vo = origin("world_subzone_volumes", id, o.source, o.row);
        let target = p
            .link(
                &vo,
                &format!("SubzoneVolumeInfoMap/Entries/{i}/SubzoneTableId"),
                &v["SubzoneTableId"],
                ("Subzone", "ID"),
                false,
                false,
            )?
            .and_then(|i| number(&rows(p.batch, "Subzone")[i], "ID"));
        p.db.execute(
            "INSERT INTO world_subzone_volumes VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                id,
                asset,
                key,
                number(v, "SubzoneTableId"),
                target,
                l["X"].as_f64(),
                l["Y"].as_f64(),
                l["Z"].as_f64(),
                v["HeightMin"].as_f64(),
                v["HeightMax"].as_f64(),
                v["RadiusSquared"].as_f64()
            ],
        )?;
        for (i, point) in list(v, "Points").iter().enumerate() {
            p.db.execute(
                "INSERT INTO world_volume_points VALUES(?1,?2,?3,?4)",
                params![id, i as i64, point["X"].as_f64(), point["Y"].as_f64()],
            )?;
        }
    }
    Ok(())
}
fn event(
    p: &Projection<'_>,
    o: &Origin<'_>,
    v: &Value,
    path: &str,
    asset: i64,
    depth: usize,
) -> Result<()> {
    if depth > 64 {
        return Err(invalid("map event nesting exceeds limit").into());
    }
    match v {
        Value::Object(fields) => {
            if let Some(kind @ ("StartSpawner" | "EndSpawner")) = string(&v["Type"]) {
                p.db.execute(
                    "INSERT INTO world_event_spawner_commands VALUES(?1,?2,?3,?4)",
                    params![asset, path, kind, v["value_list"][0]["value"].as_str()],
                )?;
            }
            for (k, child) in fields {
                let next = format!("{path}/{k}");
                if k.ends_with("Conditions") && child.as_array().is_some_and(|v| !v.is_empty()) {
                    p.condition(o, &next, &Value::Null, child)?;
                }
                event(p, o, child, &next, asset, depth + 1)?;
            }
        }
        Value::Array(a) => {
            for (i, child) in a.iter().enumerate() {
                event(p, o, child, &format!("{path}/{i}"), asset, depth + 1)?;
            }
        }
        _ => (),
    }
    Ok(())
}
fn hierarchy_rows(p: &mut Projection<'_>, o: &Origin<'_>, r: &Value, asset: i64) -> Result<()> {
    let map = link(p, o, r, "MapId", "Map", "ID", false)?;
    let mut index: i64 = p.db.query_row(
        "SELECT count(*) FROM world_hierarchy_entries WHERE map_asset_id=?1",
        [asset],
        |r| r.get(0),
    )?;
    for (si, sub) in list(r, "Subzones").iter().enumerate() {
        let subzone = p
            .link(
                o,
                &format!("Subzones/{si}/SubzoneId"),
                &sub["SubzoneId"],
                ("Subzone", "ID"),
                false,
                false,
            )?
            .and_then(|i| number(&rows(p.batch, "Subzone")[i], "ID"));
        for (gi, g) in list(sub, "Groups").iter().enumerate() {
            for (ni, npc) in list(g, "Npcs").iter().enumerate() {
                let path = format!("Subzones/{si}/Groups/{gi}/Npcs/{ni}/NpcId");
                let target = p
                    .link(o, &path, &npc["NpcId"], ("NpcData", "ID"), false, false)?
                    .and_then(|i| number(&rows(p.batch, "NpcData")[i], "ID"));
                let mut values = list(npc, "Items").to_vec();
                if values.is_empty() {
                    values.push(Value::Null);
                }
                for (ii, item) in values.iter().enumerate() {
                    let found = p
                        .link(
                            o,
                            &format!("Subzones/{si}/Groups/{gi}/Npcs/{ni}/Items/{ii}/ItemId"),
                            &item["ItemId"],
                            ("Item", "ID"),
                            true,
                            false,
                        )?
                        .and_then(|i| number(&rows(p.batch, "Item")[i], "ID"));
                    p.db.execute("INSERT INTO world_hierarchy_entries VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![asset,index,number(r,"MapId"),map,number(sub,"SubzoneId"),subzone,number(g,"NpcTypeOrdinal"),number(npc,"NpcId"),target,number(item,"ItemId"),found,number(item,"UnknownWord")])?;
                    index += 1;
                }
            }
        }
    }
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
    maps(&mut p)?;
    dungeons(&mut p, &images)?;
    npcs(&mut p, &images)?;
    gathering(&mut p, &images)?;
    objects_and_periods(&mut p, &images)?;
    assets(&mut p)?;
    for (subject, diagnostic) in [
        (
            "world.coordinates",
            "MapData coordinates preserved in source frame; units, axis orientation, image projection, layer offsets and runtime presence not inferred",
        ),
        (
            "world.routes",
            "Waypoints and their ordered points retained separately; no spawn/route assignment from proximity or names",
        ),
        (
            "world.npc_stats",
            "NpcData levels, classifications and explicit skills retained; no HP, attack or defence extrapolated from level",
        ),
        (
            "world.hierarchy",
            "Typed hierarchy and unknown item word retained; item membership is not a drop rate or a live spawn",
        ),
        (
            "world.gathering",
            "Only Usage=GatherSource links UsageValue to GatherSource.Name; every material GroupKey and additional/failure result retained without probabilities",
        ),
    ] {
        coverage(db, subject, "uninterpreted", diagnostic)?;
    }
    let missing: Vec<_> = SOURCES
        .iter()
        .filter(|s| !batch.contains_key(**s))
        .collect();
    coverage(
        db,
        "world.projection",
        if batch.contains_key("Map") {
            "partial"
        } else {
            "not_researched"
        },
        &format!(
            "{} undecoded table sources {missing:?}; service regions are not world regions; source IDs and missing references preserved; no live availability inferred",
            missing.len()
        ),
    )?;
    for dir in super::super::map_resources::DIRECTORIES {
        let count = batch
            .keys()
            .filter(|n| n.starts_with(&format!("{dir}/")))
            .count();
        let failed:i64=db.query_row("SELECT count(*) FROM catalogue_coverage WHERE subject LIKE ?1 AND state IN('failed','retained')",[format!("{dir}/%")],|r|r.get(0))?;
        coverage(
            db,
            &format!("world.resources.{dir}"),
            if count > 0 {
                if failed > 0 { "partial" } else { "verified" }
            } else if failed > 0 {
                "failed"
            } else {
                "not_found"
            },
            &format!(
                "{count} structurally decoded resource files, {failed} failed/retained; meaning and coordinate transforms are tracked separately"
            ),
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
    let families: Vec<_> = ROOTS.iter().map(|(_, f, _)| *f).collect();
    let mut result = catalogue_projection::fingerprints(db, batch, en, loc, &families)?;
    let mut keys: BTreeMap<i64, Vec<(String, String)>> = BTreeMap::new();
    for row in db.prepare("SELECT source_id,field,string_key FROM catalogue_texts WHERE family='world_gather_sources' AND string_key IS NOT NULL ORDER BY source_id,field")?.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?)))? {
        let(id,field,key)=row?;keys.entry(id).or_default().push((field,key));
    }
    for (id, keys) in keys {
        if let Some((base, localized)) = result.get_mut(&("world_gather_sources".into(), id)) {
            let english: Vec<_> = keys
                .iter()
                .map(|(f, k)| (f, k, text(en, Some(k))))
                .collect();
            let translated: Vec<_> = keys
                .iter()
                .map(|(f, k)| (f, k, text(loc, Some(k))))
                .collect();
            *base = hash(&serde_json::to_vec(&(&base, english))?);
            *localized = hash(&serde_json::to_vec(&(&localized, translated))?);
        }
    }
    Ok(result)
}
