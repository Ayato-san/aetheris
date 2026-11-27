use super::tests::{add, batch, item, project};
use super::*;
use serde_json::json;
fn fixture() -> BTreeMap<String, TableProbe> {
    let mut data = batch(vec![item(1)]);
    for (name, values) in [
        (
            "Map",
            vec![
                json!({"ID":{"Value":20},"Name":"map_a","Desc":{"Key":"item_name"},"MapRace":"ERace::All","DefaultSubzoneID":30}),
            ],
        ),
        (
            "Subzone",
            vec![
                json!({"ID":{"Value":30},"Name":"sub_a","MapId":{"Value":20},"DisplayName":{"Key":"item_name"},"ZoneId":{"Value":999}}),
            ],
        ),
        (
            "Dungeon",
            vec![
                json!({"ID":{"Value":50},"Name":"dungeon_a","MapId":{"Value":20},"Title":{"Key":"item_name"},"RecommendLevel":0,"DungeonType":"EDungeonType::InstanceLayer"}),
            ],
        ),
        (
            "NpcData",
            vec![
                json!({"ID":{"Value":60},"Name":"test_dummy","Desc":{"Key":"item_name"},"Level":10,"NpcType":"ENpcType::Monster","ActiveSkillList":[],"bCanBeAttacked":false}),
            ],
        ),
        (
            "GatherSource",
            vec![
                json!({"ID":{"Value":70},"Name":"gather_a","MaterialGroup":"materials","MasteryLevel":1}),
            ],
        ),
        (
            "GatherSourceMaterial",
            vec![
                json!({"MaterialGroup":"materials","GroupKey":1,"MaterialName":"item_1","MaterialCount":2,"AddMaterialList":[{"Item":"missing","Count":0}]}),
                json!({"MaterialGroup":"materials","GroupKey":2,"MaterialName":"item_1","MaterialCount":3}),
            ],
        ),
        (
            "EnvObjData",
            vec![
                json!({"ID":{"Value":80},"Name":"obj_a","Usage":"EEnvObjectUsage::GatherSource","UsageValue":"gather_a"}),
                json!({"ID":{"Value":81},"Name":"obj_b","Usage":"EEnvObjectUsage::Normal","UsageValue":"gather_a"}),
            ],
        ),
        (
            "Map/A/MapData",
            vec![
                json!({"MapId":20,"SpawnInfoList":[{"Name":"spawn_a","UniqueKey":1,"NpcIdList":[{"Value":60},{"Value":999}],"EnvObjIdList":[{"Value":80}],"Positions":[{"Location":{"X":10.5,"Y":-20.0,"Z":30.0},"Rotation":{"Yaw":90.0}},{"Location":{"X":11.0,"Y":-21.0,"Z":31.0}}]}],"WaypointsDataMap":{"Entries":[{"Key":"route","Value":{"Points":[{"X":1.0,"Y":2.0,"Z":3.0}]}}]}}),
            ],
        ),
        (
            "MapEvent/event",
            vec![
                json!({"MapID":20,"StartConditions":[[{"Type":"Unknown","Value":["x"]}]],"Stages":[{"Steps":[{"PreDirectives":[{"Type":"StartSpawner","value_list":[{"value":"spawn_a"}]}]}]}]}),
            ],
        ),
    ] {
        add(&mut data, name, values);
    }
    data
}
#[test]
fn world_keeps_all_spawn_candidates_positions_and_typed_gather_links() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    assert_eq!(
        db.query_row("SELECT count(*) FROM world_spawn_candidates", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM world_spawn_candidates WHERE npc_id IS NULL AND kind='npc'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        db.query_row(
            "SELECT x_raw,y_raw,z_raw,yaw_raw FROM world_spawn_positions WHERE position=0",
            [],
            |r| Ok((
                r.get::<_, f64>(0)?,
                r.get::<_, f64>(1)?,
                r.get::<_, f64>(2)?,
                r.get::<_, f64>(3)?
            ))
        )
        .unwrap(),
        (10.5, -20., 30., 90.)
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM world_spawn_positions", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        db.query_row(
            "SELECT gather_source_id FROM world_environment_objects WHERE id=80",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        70
    );
    assert!(
        db.query_row(
            "SELECT gather_source_id FROM world_environment_objects WHERE id=81",
            [],
            |r| r.get::<_, Option<i64>>(0)
        )
        .unwrap()
        .is_none()
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM world_gather_results", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        db.query_row(
            "SELECT quantity FROM world_gather_results WHERE role='additional'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM world_event_spawner_links", [], |r| {
            r.get::<_, i64>(0)
        })
        .unwrap(),
        1
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM world_routes", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM catalogue_conditions WHERE family='world_map_assets'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        db.query_row(
            "SELECT state FROM catalogue_coverage WHERE subject='world.coordinates'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "uninterpreted"
    );
    assert_eq!(
        db.query_row("SELECT recommended_level FROM world_dungeons", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn unknown_world_references_do_not_create_definitions_or_combat_stats() {
    let mut data = fixture();
    data.get_mut("Subzone").unwrap().layout.rows[0]["MapId"] = json!({"Value":999});
    data.get_mut("NpcData").unwrap().layout.rows[0]["ActiveSkillList"] = json!([{"Value":999}]);
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert_eq!(
        db.query_row("SELECT map_reference,map_id FROM world_subzones", [], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?))
        })
        .unwrap(),
        (999, None)
    );
    assert_eq!(
        db.query_row(
            "SELECT skill_reference,skill_id FROM world_npc_skills",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?))
        )
        .unwrap(),
        (999, None)
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM world_maps", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        db.query_row("SELECT level,attackable FROM world_npcs", [], |r| Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, bool>(1)?
        )))
        .unwrap(),
        (10, false)
    );
    let columns: Vec<String> = db
        .prepare("PRAGMA table_info(world_npcs)")
        .unwrap()
        .query_map([], |r| r.get(1))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert!(
        !columns
            .iter()
            .any(|c| matches!(c.as_str(), "hp" | "attack" | "defence"))
    );
    assert_eq!(
        db.query_row(
            "SELECT state FROM catalogue_coverage WHERE subject='world.resources.WorldMap'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "not_found"
    );
}
#[test]
fn npc_loot_changes_belong_to_the_loot_table_and_world_definitions_keep_their_build() {
    let mut data = fixture();
    add(
        &mut data,
        "NpcLoot",
        vec![json!({"NpcId":{"Value":60},"LootList":[{"Value":1}]})],
    );
    let (first, _) = project(&data, None, "100", "fr-FR");
    let old_id: i64 = first
        .query_row(
            "SELECT id FROM world_map_assets WHERE kind='Map'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    data.get_mut("NpcLoot").unwrap().layout.rows[0]["LootList"] = json!([]);
    let (second, summary) = project(&data, Some(&first), "200", "fr-FR");
    assert_eq!(summary.modified, 1);
    assert_eq!(
        second
            .query_row(
                "SELECT family FROM catalogue_entries WHERE change_type='modified'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "loot_tables"
    );
    assert_eq!(
        second
            .query_row(
                "SELECT id FROM world_map_assets WHERE kind='Map'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        old_id
    );
    assert_eq!(second.query_row("SELECT v.steam_build FROM world_npcs n JOIN client_version v ON v.id=n.client_version_id",[],|r|r.get::<_,String>(0)).unwrap(),"100");
    assert_eq!(
        second
            .query_row("SELECT count(*) FROM world_npc_loot", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    data.get_mut("NpcData").unwrap().layout.rows[0]["Level"] = json!(11);
    let (third, _) = project(&data, Some(&second), "300", "fr-FR");
    assert_eq!(third.query_row("SELECT v.steam_build FROM world_npcs n JOIN client_version v ON v.id=n.client_version_id",[],|r|r.get::<_,String>(0)).unwrap(),"300");
}
#[test]
fn hierarchy_keeps_unknown_words_and_missing_ids_without_converting_them_to_rates() {
    let mut data = fixture();
    add(
        &mut data,
        "MapDataHierarchy/MapDataHierarchy",
        vec![
            json!({"MapId":20,"Subzones":[{"SubzoneId":30,"Groups":[{"NpcTypeOrdinal":99,"Npcs":[{"NpcId":60,"Items":[{"ItemId":1,"UnknownWord":500},{"ItemId":999,"UnknownWord":0}]},{"NpcId":999,"Items":[]}]}]}]}),
        ],
    );
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert_eq!(
        db.query_row("SELECT count(*) FROM world_hierarchy_entries", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        db.query_row(
            "SELECT unknown_word,item_id FROM world_hierarchy_entries WHERE item_reference=999",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?))
        )
        .unwrap(),
        (0, None)
    );
    assert_eq!(
        db.query_row(
            "SELECT npc_type_ordinal FROM world_hierarchy_entries WHERE item_id=1",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        99
    );
}

#[test]
fn world_map_layout_preserves_source_frame_and_candidate_map_binding() {
    let mut data = fixture();
    add(
        &mut data,
        "WorldMap/a",
        vec![
            json!({"WorldMapDataTableKey":"map_a","SectorSize":{"X":8,"Y":8},"SectorWorldSize":102000.0,"StartWorldLocation":{"X":-408000.0,"Y":-408000.0},"PreviewLocations":{"Entries":[{"Key":"Label","Value":{"X":60.0,"Y":440.0,"Z":0.0}}]},"WorldBoundBox":{"IsValid":false}}),
        ],
    );
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert_eq!(db.query_row("SELECT candidate_map_id,start_x_raw,sector_world_size_raw,bounds_valid,interpretation_state FROM world_map_layouts",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,f64>(1)?,r.get::<_,f64>(2)?,r.get::<_,bool>(3)?,r.get::<_,String>(4)?))).unwrap(),(20,-408000.0,102000.0,false,"candidate".into()));
    assert_eq!(
        db.query_row(
            "SELECT x_raw,y_raw FROM world_map_preview_locations",
            [],
            |r| Ok((r.get::<_, f64>(0)?, r.get::<_, f64>(1)?))
        )
        .unwrap(),
        (60., 440.)
    );
    assert_eq!(db.query_row("SELECT interpretation_state FROM catalogue_references WHERE source_path='WorldMapDataTableKey'",[],|r|r.get::<_,String>(0)).unwrap(),"candidate");
}
