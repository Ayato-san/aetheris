use super::tests::{add, batch, item, project, project_with};
use super::*;
use serde_json::json;

fn node(id: i64, kind: &str, row: i64, col: i64) -> Value {
    json!({"ID":{"Value":id},"Board":{"Value":11},"Row":row,"Col":col,"Type":format!("EDaevanionNodeType::{kind}"),"Grade":"EItemGrade::Common","NeedLevel":12,"CostDaevanionPoint":0,"ResetGold":0,"NeedConditionType":"EDaevanionNodeConditionType::None","NeedConditionValue":"","Title":{"Key":"item_name"},"bAutoLearn":false,"Value01":"","Value02":"","Value03":"","Value04":"","Value05":"","Value06":"","Value07":"","Value08":"","Value09":"","Value10":""})
}
fn fixture() -> BTreeMap<String, TableProbe> {
    let mut charge = item(1);
    charge["$variant"] = json!({"UseEffect":"EItemUseEffect::GetDaevanionPoint","UseEffectValues":["DaevanionCrystal","3"]});
    let mut data = batch(vec![charge]);
    for name in daevanion::SOURCES {
        add(&mut data, name, vec![]);
    }
    add(
        &mut data,
        "DaevanionMaxCapacity",
        vec![
            json!({"DaevanionPointType":"EDaevanionPointType::DaevanionCrystal","DaevanionPointName":"DaevanionCrystal","CurrencyCategory":"ECurrencyCategory::DaevanionPoint","MaxValue":840}),
        ],
    );
    add(
        &mut data,
        "Exp",
        vec![
            json!({"Level":12,"NeedExp":100,"DaevanionPointMap":{"Entries":[{"Key":"EDaevanionPointType::DaevanionCrystal","Value":4}],"Removed":[]}}),
        ],
    );
    add(
        &mut data,
        "DaevanionBoard",
        vec![
            json!({"ID":{"Value":11},"Class":"ECharacterClass::Gladiator","NeedLevel":12,"CostPointType":"EDaevanionPointType::DaevanionCrystal","ResetCostCurrencyType":"ECurrencyCategory::GoldCombined","Title":{"Key":"item_name"},"BuffTitle":{"Key":"item_desc"},"BuffDesc":{"Key":"item_name"},"Order":1,"Icon":"Weapon_Sword"}),
        ],
    );
    let mut start = node(100, "Start", 8, 8);
    start["bAutoLearn"] = json!(true);
    start["Value01"] = json!("item_desc");
    let blank = node(7, "None", 8, 9);
    let mut stat = node(500, "Stat", 8, 10);
    stat["Value01"] = json!("CombatSpeed");
    stat["Value02"] = json!("150");
    stat["Value03"] = json!("Accuracy");
    stat["Value04"] = json!("0");
    stat["CostDaevanionPoint"] = json!(4);
    stat["ResetGold"] = json!(500);
    let mut skill = node(8, "SkillLevel", 9, 8);
    skill["Value01"] = json!("5");
    skill["Value02"] = json!("1");
    add(&mut data, "DaevanionNode", vec![start, blank, stat, skill]);
    add(
        &mut data,
        "Skill",
        vec![json!({"ID":{"Value":5},"SkillLvMax":1,"SkillType":"ESkillType::Passive"})],
    );
    add(
        &mut data,
        "Title",
        vec![
            json!({"ID":{"Value":77},"Name":"stat_provider","StatList":[{"Stat":"EStat::CombatSpeed","Value":5},{"Stat":"EStat::Accuracy","Value":1}]}),
        ],
    );
    data
}

#[test]
fn nodes_preserve_source_ids_positions_and_costs_but_exclude_none_cells() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    assert_eq!(db.query_row("SELECT row_number,column_number,board_id,point_cost_raw,reset_cost_raw FROM daevanion_nodes WHERE id=500",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?))).unwrap(),(8,10,11,4,500));
    assert_eq!(
        db.query_row(
            "SELECT c.code FROM daevanion_boards b JOIN skill_classes c ON c.id=b.class_id",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "Gladiator"
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM daevanion_nodes", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM daevanion_node_parameters WHERE node_id=7 AND value_source=''",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        db.query_row(
            "SELECT auto_learn,start_text_en FROM daevanion_nodes WHERE id=100",
            [],
            |r| Ok((r.get::<_, bool>(0)?, r.get::<_, String>(1)?))
        )
        .unwrap(),
        (true, "{se:1:effect_value01:none}".into())
    );
    assert_eq!(
        db.query_row(
            "SELECT state FROM catalogue_coverage WHERE subject='daevanion.connections.tables'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "not_found"
    );
    assert_eq!(db.query_row("SELECT state FROM catalogue_coverage WHERE subject='daevanion.connections.client_logic'",[],|r|r.get::<_,String>(0)).unwrap(),"not_researched");
}

#[test]
fn stat_units_remain_raw_while_skill_level_bonuses_are_verified() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    let params:Vec<(i64,String,i64,String)>=db.prepare("SELECT d.parameter_position,s.code,d.value_integer_raw,d.interpretation_state FROM daevanion_stat_parameters d JOIN stats s ON s.id=d.stat_id ORDER BY d.parameter_position").unwrap().query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(
        params,
        vec![
            (1, "EStat::CombatSpeed".into(), 150, "candidate".into()),
            (3, "EStat::Accuracy".into(), 0, "candidate".into())
        ]
    );
    assert_eq!(
        db.query_row(
            "SELECT skill_id,level_bonus,interpretation_state FROM daevanion_skill_parameters",
            [],
            |r| Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?
            ))
        )
        .unwrap(),
        (5, 1, "verified".into())
    );
    // A Daevanion parameter never modifies the source skill's level definitions.
    assert_eq!(
        db.query_row("SELECT max_level FROM skills WHERE id=5", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(db.query_row("SELECT interpretation_state FROM catalogue_references WHERE family='daevanion_nodes' AND source_path='Value01'",[],|r|r.get::<_,String>(0)).unwrap(),"verified");
}

#[test]
fn missing_boards_skills_unknown_stats_and_opaque_conditions_are_not_replaced() {
    let mut data = fixture();
    let rows = &mut data.get_mut("DaevanionNode").unwrap().layout.rows;
    rows[2]["Value01"] = json!("NotAStat");
    rows[2]["Value02"] = json!("1.5");
    rows[2]["Board"] = json!({"Value":999});
    rows[2]["NeedConditionType"] = json!("EDaevanionNodeConditionType::Future");
    rows[2]["NeedConditionValue"] = json!("100");
    rows[3]["Value01"] = json!("9999");
    rows[3]["Value02"] = json!("1.5");
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert_eq!(db.query_row("SELECT board_reference,board_id,condition_value_source FROM daevanion_nodes WHERE id=500",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,Option<i64>>(1)?,r.get::<_,String>(2)?))).unwrap(),(999,None,"100".into()));
    assert_eq!(db.query_row("SELECT stat_id,value_source,value_integer_raw FROM daevanion_stat_parameters WHERE parameter_position=1",[],|r|Ok((r.get::<_,Option<i64>>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<i64>>(2)?))).unwrap(),(None,"1.5".into(),None));
    assert_eq!(
        db.query_row(
            "SELECT skill_reference,skill_id FROM daevanion_skill_parameters",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?))
        )
        .unwrap(),
        (9999, None)
    );
    assert_eq!(db.query_row("SELECT resolution_state FROM catalogue_references WHERE family='daevanion_nodes' AND source_id=500 AND source_path='Board'",[],|r|r.get::<_,String>(0)).unwrap(),"missing");
    assert_eq!(
        db.query_row(
            "SELECT value_source,level_bonus FROM daevanion_skill_parameters",
            [],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<i64>>(1)?))
        )
        .unwrap(),
        ("1.5".into(), None)
    );
    assert_eq!(
        db.query_row(
            "SELECT state FROM catalogue_coverage WHERE subject='daevanion.skill_level_bonus'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "partial"
    );
}

#[test]
fn capacities_level_values_and_item_effects_keep_their_source_encoding() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    assert_eq!(
        db.query_row("SELECT max_value_raw FROM daevanion_point_types", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        840
    );
    assert_eq!(
        db.query_row(
            "SELECT character_level,points_raw FROM daevanion_level_points",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        )
        .unwrap(),
        (12, 4)
    );
    assert_eq!(
        db.query_row(
            "SELECT item_id,point_reference,value_source FROM daevanion_point_items",
            [],
            |r| Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?
            ))
        )
        .unwrap(),
        (1, "DaevanionCrystal".into(), "3".into())
    );
}

#[test]
fn changes_track_node_definitions_and_related_point_maps_but_not_unrelated_exp() {
    let mut data = fixture();
    let (first, _) = project(&data, None, "100", "fr-FR");
    data.get_mut("Exp").unwrap().layout.rows[0]["NeedExp"] = json!(999);
    data.get_mut("DaevanionNode").unwrap().layout.rows.reverse();
    let (same, _) = project(&data, Some(&first), "200", "fr-FR");
    assert_eq!(same.query_row("SELECT v.steam_build FROM daevanion_boards b JOIN client_version v ON v.id=b.client_version_id",[],|r|r.get::<_,String>(0)).unwrap(),"100");
    data.get_mut("Exp").unwrap().layout.rows[0]["DaevanionPointMap"]["Entries"][0]["Value"] =
        json!(8);
    let (changed, _) = project(&data, Some(&same), "300", "fr-FR");
    assert_eq!(changed.query_row("SELECT v.steam_build FROM daevanion_boards b JOIN client_version v ON v.id=b.client_version_id",[],|r|r.get::<_,String>(0)).unwrap(),"300");
    let node = data
        .get_mut("DaevanionNode")
        .unwrap()
        .layout
        .rows
        .iter_mut()
        .find(|r| number(r, "ID") == Some(500))
        .unwrap();
    node["Value02"] = json!("200");
    let (next, _) = project(&data, Some(&changed), "400", "fr-FR");
    assert_eq!(next.query_row("SELECT v.steam_build FROM daevanion_nodes n JOIN client_version v ON v.id=n.client_version_id WHERE n.id=500",[],|r|r.get::<_,String>(0)).unwrap(),"400");
    assert_eq!(next.query_row("SELECT v.steam_build FROM daevanion_boards b JOIN client_version v ON v.id=b.client_version_id",[],|r|r.get::<_,String>(0)).unwrap(),"400");
}

#[test]
fn indirect_stat_translations_follow_their_language_without_false_locale_changes() {
    let data = fixture();
    let (first, _) = project(&data, None, "100", "fr-FR");
    let mut localized = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::from([
            ("item_name".into(), "Name".into()),
            ("item_desc".into(), "{se:1:effect_value01:none}".into()),
            (
                "String_StatName_CombatSpeed_body".into(),
                "Vitesse de combat".into(),
            ),
        ]),
    };
    let (translated, _) = project_with(
        &data,
        Some(&first),
        "200",
        "fr-FR",
        &BTreeSet::new(),
        Some(&localized),
    );
    assert_eq!(translated.query_row("SELECT v.steam_build FROM daevanion_nodes n JOIN client_version v ON v.id=n.client_version_id WHERE n.id=500",[],|r|r.get::<_,String>(0)).unwrap(),"200");
    localized.entries.remove("String_StatName_CombatSpeed_body");
    let (switched, _) = project_with(
        &data,
        Some(&translated),
        "300",
        "de-DE",
        &BTreeSet::new(),
        Some(&localized),
    );
    assert_eq!(switched.query_row("SELECT v.steam_build FROM daevanion_nodes n JOIN client_version v ON v.id=n.client_version_id WHERE n.id=500",[],|r|r.get::<_,String>(0)).unwrap(),"200");
    assert_eq!(
        switched
            .query_row("SELECT name_localized FROM daevanion_boards", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap(),
        "Name"
    );
}

#[test]
fn new_connection_fields_remain_uninterpreted_and_removed_point_entries_affect_tracking() {
    let mut data = fixture();
    let (first, _) = project(&data, None, "100", "fr-FR");
    data.get_mut("DaevanionNode").unwrap().layout.rows[0]["ConnectionIds"] = json!([7]);
    data.get_mut("Exp").unwrap().layout.rows[0]["DaevanionPointMap"]["Removed"] =
        json!(["EDaevanionPointType::DaevanionCrystal"]);
    let (next, _) = project(&data, Some(&first), "200", "fr-FR");
    assert_eq!(
        next.query_row(
            "SELECT state FROM catalogue_coverage WHERE subject='daevanion.connections.tables'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "uninterpreted"
    );
    assert_eq!(next.query_row("SELECT v.steam_build FROM daevanion_nodes n JOIN client_version v ON v.id=n.client_version_id WHERE n.id=500",[],|r|r.get::<_,String>(0)).unwrap(),"200");
    let original = source_rows(&next, "DaevanionNode").unwrap();
    assert_eq!(original[0]["ConnectionIds"], json!([7]));
}

#[test]
fn node_images_reuse_exact_ui_sprite_aliases_without_guessing_similar_keys() {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!("items.sql")).unwrap();
    db.execute_batch(include_str!("catalogue.sql")).unwrap();
    db.execute(
        "INSERT INTO catalogue_icons VALUES(1,'UI/Sprite/test',NULL,NULL,'not_researched',NULL)",
        [],
    )
    .unwrap();
    db.execute(
        "INSERT INTO ui_sprites VALUES(1,'/Game/UI/Resource/Sprite/test',1,16,16)",
        [],
    )
    .unwrap();
    db.execute(
        "INSERT INTO ui_sprite_aliases VALUES('UI_ImageTable_Composite','Daevanion_Node_Common',1)",
        [],
    )
    .unwrap();
    let images = projection::image_index(None).unwrap();
    assert_eq!(
        daevanion::node_image(&db, 1, Some("Daevanion_Node_Common"), &images).unwrap(),
        Some(1)
    );
    assert_ne!(
        daevanion::node_image(&db, 2, Some("Daevanion_Node_Common_New"), &images).unwrap(),
        Some(1)
    );
    assert_eq!(db.query_row("SELECT icon_id FROM catalogue_entity_icons WHERE family='daevanion_nodes' AND source_id=1",[],|r|r.get::<_,i64>(0)).unwrap(),1);
}

#[test]
fn connections_use_cardinal_neighbours_and_never_cross_blanks_diagonals_or_boards() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    let edges:Vec<(i64,i64)>=db.prepare("SELECT node_id,neighbour_id FROM daevanion_node_connections ORDER BY node_id,neighbour_id").unwrap().query_map([],|r|Ok((r.get(0)?,r.get(1)?))).unwrap().collect::<rusqlite::Result<_>>().unwrap();
    // 100 (8,8) touches 8 (9,8). None at (8,9) cannot bridge to 500 (8,10).
    assert_eq!(edges, vec![(8, 100)]);
    let board = super::super::daevanion_graph::Board::load(&db, 11).unwrap();
    assert_eq!(board.start(), 100);
    assert!(board.can_activate(8, &BTreeSet::new()));
    assert!(!board.can_activate(500, &BTreeSet::new()));
    assert_eq!(board.unreachable(), BTreeSet::from([500]));
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM catalogue_entries WHERE family='daevanion_nodes' AND source_id=7",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    // Source evidence stays lossless even though None is absent from business tables.
    assert_eq!(source_rows(&db, "DaevanionNode").unwrap().len(), 4);
    let mut data = fixture();
    data.get_mut("DaevanionNode")
        .unwrap()
        .layout
        .rows
        .push(node(501, "Stat", 9, 9));
    let mut other_board = data["DaevanionBoard"].layout.rows[0].clone();
    other_board["ID"] = json!({"Value":12});
    data.get_mut("DaevanionBoard")
        .unwrap()
        .layout
        .rows
        .push(other_board);
    let mut isolated = node(502, "Stat", 8, 7);
    isolated["Board"] = json!({"Value":12});
    data.get_mut("DaevanionNode")
        .unwrap()
        .layout
        .rows
        .push(isolated);
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert_eq!(db.query_row("SELECT count(*) FROM daevanion_node_connections WHERE node_id=502 OR neighbour_id=502 OR (node_id=100 AND neighbour_id=501)",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    let mut data = fixture();
    data.get_mut("DaevanionNode")
        .unwrap()
        .layout
        .rows
        .retain(|r| number(r, "ID") != Some(100));
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert!(super::super::daevanion_graph::Board::load(&db, 11).is_err());
    let mut data = fixture();
    data.get_mut("DaevanionNode")
        .unwrap()
        .layout
        .rows
        .push(node(600, "Start", 1, 1));
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert!(super::super::daevanion_graph::Board::load(&db, 11).is_err());
    let mut data = fixture();
    data.get_mut("DaevanionNode").unwrap().layout.rows[2]
        .as_object_mut()
        .unwrap()
        .remove("Col");
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert!(super::super::daevanion_graph::Board::load(&db, 11).is_err());
}
#[test]
fn neighbour_changes_update_both_endpoints_and_blank_metadata_is_ignored() {
    let mut data = fixture();
    let (first, _) = project(&data, None, "100", "fr-FR");
    data.get_mut("DaevanionNode").unwrap().layout.rows[1]["ResetGold"] = json!(900);
    let (same, _) = project(&data, Some(&first), "200", "fr-FR");
    assert_eq!(same.query_row("SELECT v.steam_build FROM daevanion_boards b JOIN client_version v ON v.id=b.client_version_id",[],|r|r.get::<_,String>(0)).unwrap(),"100");
    data.get_mut("DaevanionNode").unwrap().layout.rows[3]["Row"] = json!(10);
    let (changed, _) = project(&data, Some(&same), "300", "fr-FR");
    assert_eq!(changed.query_row("SELECT v.steam_build FROM daevanion_nodes n JOIN client_version v ON v.id=n.client_version_id WHERE n.id=100",[],|r|r.get::<_,String>(0)).unwrap(),"300");
}
