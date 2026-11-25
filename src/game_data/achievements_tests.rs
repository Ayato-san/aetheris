use super::tests::{add, batch, item, project, project_with};
use super::*;
use serde_json::json;

fn fixture() -> BTreeMap<String, TableProbe> {
    let mut grant = item(2);
    grant["$variant"] = json!({"UseEffect":"EItemUseEffect::GetTitle","UseEffectValues":["50"]});
    let mut data = batch(vec![item(1), grant]);
    for name in achievements::SOURCES {
        add(&mut data, name, vec![]);
    }
    add(
        &mut data,
        "AchievementGroup",
        vec![
            json!({"ID":{"Value":10},"Name":"group","Title":{"Key":"item_name"},"Category":"EAchievementCategory::SeasonMission","bIsAccountShared":false}),
        ],
    );
    add(
        &mut data,
        "AchievementGroupSet",
        vec![
            json!({"Name":"set","GroupList":["group"],"Title":{"Key":"item_name"},"Race":"ERace::Light"}),
        ],
    );
    add(
        &mut data,
        "Achievement",
        vec![
            json!({"AchievementId":{"Value":20},"GroupName":"group","ObjType":"EAchievementObjectiveType::ClearQuest","ObjValues":["unresolved_quest","opaque"],"UnlockAchievementId":{"Value":21},"Grade":"EAchievementGrade::Rare","ResetCycleType":"EResetCycleType::Weekly"}),
            json!({"AchievementId":{"Value":21},"GroupName":"group","ObjValues":[]}),
        ],
    );
    add(
        &mut data,
        "AchievementStep",
        vec![
            json!({"DataId":{"Value":200},"AchievementId":{"Value":20},"Step":1,"Count":100,"AchievementText":"step_text"}),
            json!({"DataId":{"Value":201},"AchievementId":{"Value":20},"Step":2,"Count":250,"AchievementText":"step_text"}),
        ],
    );
    add(
        &mut data,
        "AchievementString",
        vec![
            json!({"Name":"step_text","AchievementTitle":{"Key":"item_name"},"ObjDesc":{"Key":"item_desc"}}),
        ],
    );
    add(
        &mut data,
        "AchievementReward",
        vec![
            json!({"ID":{"Value":300},"AchievementId":20,"Step":1,"RewardPoint":0,"ItemRewards":[{"Item":"item_1","Count":2}]}),
            json!({"ID":{"Value":301},"AchievementId":20,"Step":2,"RewardPoint":5,"ItemRewards":[{"Item":"item_2","Count":1}]}),
        ],
    );
    add(
        &mut data,
        "AchievementGroupReward",
        vec![
            json!({"GroupId":{"Value":10},"ItemRewards":[{"CompleteCount":5,"RewardItem":{"Item":"item_1","Count":0}}]}),
        ],
    );
    add(
        &mut data,
        "TitleCategory",
        vec![json!({"Name":"Quest","Order":1,"Desc":{"Key":"item_name"}})],
    );
    add(
        &mut data,
        "Title",
        vec![
            json!({"ID":{"Value":50},"Name":"title_a","CategoryName":"Quest","Desc":{"Key":"item_name"},"DescLong":{"Key":"item_desc"},"Grade":"ETitleGrade::Rare","Race":"ERace::Light","StatList":[{"Stat":"EStat::Accuracy","Value":5}],"EquipStatList":[{"Stat":"EStat::FixingDamage","Value":500}],"bShow":false,"bGetDisplay":false,"StartTime":{"Ticks":0},"EndTime":{"Ticks":999}}),
        ],
    );
    add(
        &mut data,
        "Map",
        vec![json!({"ID":{"Value":1000},"Name":"map","Desc":{"Key":"item_name"}})],
    );
    add(
        &mut data,
        "Journal",
        vec![
            json!({"ID":{"Value":70},"Name":"journal","MapId":{"Value":1000},"Race":"ERace::Light","DefaultMapX":-100.5,"DefaultMapY":200,"DefaultMapZoomLevel":2}),
        ],
    );
    add(
        &mut data,
        "SeasonScheduleGroup",
        vec![json!({"ID":{"Value":80},"Name":"Season_Main"})],
    );
    add(
        &mut data,
        "SeasonSchedule",
        vec![
            json!({"ID":{"Value":81},"GroupName":"Season_Main","SeasonNumber":1,"StartDate":{"Ticks":0},"EndDate":{"Ticks":100},"RewardStartDate":{"Ticks":200},"RewardEndDate":{"Ticks":300}}),
            json!({"ID":{"Value":82},"GroupName":"Season_Main","SeasonNumber":0,"StartDate":{"Ticks":-100},"EndDate":{"Ticks":0}}),
        ],
    );
    add(
        &mut data,
        "SeasonContents",
        vec![
            json!({"ID":{"Value":90},"ContentsType":"ESeasonContentsType::SeasonMission","ScheduleGroupName":"Season_Main","SeasonNumber":1,"PickupContents":"pickup"}),
        ],
    );
    add(
        &mut data,
        "SeasonPickupContents",
        vec![
            json!({"ID":{"Value":92},"Name":"other"}),
            json!({"ID":{"Value":91},"Name":"pickup","LightContentsNameList":["set","20"],"DarkContentsNameList":["missing_set"]}),
        ],
    );
    add(
        &mut data,
        "EventMission",
        vec![
            json!({"ID":{"Value":100},"Name":"event","Title":{"Key":"item_name"},"SeasonId":{"Value":81},"LightAchievementGroupSetName":"set","DarkAchievementGroupSetName":"None"}),
        ],
    );
    add(
        &mut data,
        "BattlePassGroup",
        vec![
            json!({"ID":{"Value":110},"Name":"pass_group","Title":{"Key":"item_name"},"Race":"ERace::Light","SeasonId":{"Value":81},"BuyLimitCount":1,"ExpLimitValue":100,"ExpLimitType":"EResetCycleType::Weekly","ExpPurchaseItemName":"item_1","ExpCosts":[{"CostType":"ECurrencyCategory::GameCash","CostCount":50},{"CostType":"ECurrencyCategory::GoldCombined","CostCount":0}],"ActivationStartDate":{"Ticks":10},"ActivationEndDate":{"Ticks":20},"SubscriptionStartDate":{"Ticks":30},"SubscriptionEndDate":{"Ticks":40}}),
            json!({"ID":{"Value":111},"Name":"old_pass_group"}),
        ],
    );
    add(
        &mut data,
        "BattlePass",
        vec![
            json!({"ID":{"Value":120},"Name":"pass_Pay","Title":{"Key":"item_name"},"GroupName":"pass_group","ExpAchievementId":{"Value":20},"bIsPremium":false,"UnlockCostType":"ECurrencyCategory::GameCash","UnlockCostCount":1000,"UnlockItemItemName":"item_1","UnlockLevel":1,"UnlockQuestName":"missing_quest","ItemReward":[{"Item":"item_2","Count":3}]}),
        ],
    );
    add(
        &mut data,
        "PeriodItemCollection",
        vec![
            json!({"ID":{"Value":130},"GroupName":"supply","Title":{"Key":"item_name"},"Desc":{"Key":"item_desc"},"Race":"ERace::Light","LevelMin":1,"bShowDate":false}),
            json!({"ID":{"Value":131},"GroupName":"supply","Race":"ERace::Light","LevelMin":40}),
        ],
    );
    add(
        &mut data,
        "PeriodItemCollectionList",
        vec![
            json!({"ID":{"Value":140},"GroupName":"supply","Title":{"Key":"item_name"},"Desc":{"Key":"item_desc"},"TargetCount":3,"RewardExp":0,"TargetItems":[{"Item":"item_1","Count":0},{"Item":"item_2","Count":0}],"RewardItems":[{"Item":"item_1","Count":5}]}),
        ],
    );
    data
}
fn scalar<T: rusqlite::types::FromSql>(db: &Connection, query: &str) -> T {
    db.query_row(query, [], |r| r.get(0)).unwrap()
}

#[test]
fn achievements_link_the_correct_steps_rewards_and_keep_opaque_objective_values() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT unlock_achievement_id FROM achievements WHERE id=20"
        ),
        21
    );
    assert_eq!(
        scalar::<i64>(&db, "SELECT step_id FROM achievement_rewards WHERE id=301"),
        201
    );
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT threshold_raw FROM achievement_steps WHERE id=201"
        ),
        250
    );
    assert_eq!(
        scalar::<String>(
            &db,
            "SELECT description_en FROM achievement_steps WHERE id=200"
        ),
        "{se:1:effect_value01:none}"
    );
    let values: Value = serde_json::from_str(&scalar::<String>(
        &db,
        "SELECT objective_values_json FROM achievements WHERE id=20",
    ))
    .unwrap();
    assert_eq!(values, json!(["unresolved_quest", "opaque"]));
    let group: (i64, i64) = db
        .query_row(
            "SELECT quantity_raw,threshold_raw FROM catalogue_item_links WHERE role='group_reward'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(group, (0, 5));
    assert_eq!(
        scalar::<String>(
            &db,
            "SELECT text_en FROM catalogue_text_values WHERE family='achievement_groups' AND source_id=10 AND field='/Title'"
        ),
        "Name"
    );
}

#[test]
fn titles_keep_possession_and_equipped_effects_separate_and_find_granting_items() {
    let mut data = fixture();
    let (first, _) = project(&data, None, "100", "fr-FR");
    let stats:Vec<(String,String,i64)>=first.prepare("SELECT t.role,s.code,t.value_raw FROM title_stats t JOIN stats s ON s.id=t.stat_id ORDER BY t.role").unwrap().query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(
        stats,
        vec![
            ("equipped".into(), "EStat::FixingDamage".into(), 500),
            ("possession".into(), "EStat::Accuracy".into(), 5)
        ]
    );
    assert!(!scalar::<bool>(
        &first,
        "SELECT show FROM titles WHERE id=50"
    ));
    assert_eq!(
        scalar::<i64>(
            &first,
            "SELECT item_id FROM catalogue_item_links WHERE family='titles' AND source_id=50 AND role='granting_item'"
        ),
        2
    );
    data.get_mut("Item").unwrap().layout.rows[1]["SellPrice"] = json!(999);
    let (second, _) = project(&data, Some(&first), "200", "fr-FR");
    assert_eq!(
        scalar::<i64>(
            &second,
            "SELECT count(*) FROM catalogue_entries WHERE family='titles' AND change_type IS NOT NULL"
        ),
        0
    );
    data.get_mut("Title").unwrap().layout.rows[0]["EquipStatList"][0]["Value"] = json!(600);
    let (third, _) = project(&data, Some(&second), "300", "fr-FR");
    assert_eq!(
        scalar::<String>(
            &third,
            "SELECT change_type FROM catalogue_entries WHERE family='titles' AND source_id=50"
        ),
        "modified"
    );
}

#[test]
fn passes_preserve_paid_costs_false_premium_raw_periods_and_old_definitions() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    assert!(!scalar::<bool>(
        &db,
        "SELECT premium_source FROM battle_passes WHERE id=120"
    ));
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT amount_raw FROM catalogue_costs WHERE family='battle_passes' AND source_id=120"
        ),
        1000
    );
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT experience_achievement_id FROM battle_passes WHERE id=120"
        ),
        20
    );
    assert_eq!(
        scalar::<i64>(&db, "SELECT count(*) FROM battle_pass_groups"),
        2
    );
    assert_eq!(scalar::<i64>(&db, "SELECT count(*) FROM seasons"), 2);
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT start_ticks FROM catalogue_periods WHERE family='seasons' AND source_id=81 AND role='season'"
        ),
        0
    );
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT end_ticks FROM catalogue_periods WHERE family='battle_pass_groups' AND source_id=110 AND role='subscription'"
        ),
        40
    );
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT amount_raw FROM catalogue_costs WHERE family='battle_pass_groups' AND source_path='ExpCosts/1'"
        ),
        0
    );
    assert_eq!(
        scalar::<Option<i64>>(
            &db,
            "SELECT unlock_quest_id FROM battle_passes WHERE id=120"
        ),
        None
    );
}

#[test]
fn supply_requests_preserve_all_level_variants_and_zero_per_item_counts() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT count(*) FROM supply_request_variants WHERE request_id=140"
        ),
        2
    );
    assert_eq!(
        scalar::<String>(
            &db,
            "SELECT resolution_state FROM catalogue_references WHERE family='supply_requests' AND source_path='GroupName'"
        ),
        "group"
    );
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT target_count_raw FROM supply_requests WHERE id=140"
        ),
        3
    );
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT count(*) FROM catalogue_item_links WHERE family='supply_requests' AND role='requested_item' AND quantity_raw=0"
        ),
        2
    );
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT quantity_raw FROM catalogue_item_links WHERE family='supply_requests' AND role='reward'"
        ),
        5
    );
}

#[test]
fn mission_relations_keep_types_provenance_missing_names_and_map_coordinates() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT achievement_id FROM mission_group_sets WHERE family='season_contents' AND race_id=2 AND position=1"
        ),
        20
    );
    assert_eq!(
        scalar::<String>(
            &db,
            "SELECT resolution_state FROM catalogue_references WHERE family='season_contents' AND source_name='SeasonPickupContents' AND source_path='DarkContentsNameList/0'"
        ),
        "missing"
    );
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT source_row FROM catalogue_references WHERE family='season_contents' AND source_name='SeasonPickupContents' AND source_path='LightContentsNameList/1'"
        ),
        1
    );
    assert_eq!(
        scalar::<i64>(
            &db,
            "SELECT season_id FROM season_content_schedules WHERE content_id=90"
        ),
        81
    );
    assert_eq!(
        scalar::<f64>(&db, "SELECT default_map_x_raw FROM journals WHERE id=70"),
        -100.5
    );
    assert_eq!(
        scalar::<String>(&db, "SELECT name_en FROM journals WHERE id=70"),
        "Name"
    );
}

#[test]
fn source_dependencies_update_the_achievement_and_its_pass_without_false_locale_changes() {
    let mut data = fixture();
    let (first, _) = project(&data, None, "100", "fr-FR");
    data.get_mut("AchievementStep").unwrap().layout.rows[0]["Count"] = json!(101);
    let (second, _) = project(&data, Some(&first), "200", "fr-FR");
    for (family, id) in [("achievements", 20), ("battle_passes", 120)] {
        let change: String = second
            .query_row(
                "SELECT change_type FROM catalogue_entries WHERE family=?1 AND source_id=?2",
                params![family, id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(change, "modified");
    }
    assert_eq!(
        scalar::<i64>(
            &second,
            "SELECT count(*) FROM catalogue_entries WHERE family='titles' AND change_type IS NOT NULL"
        ),
        0
    );
    let loc = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::new(),
    };
    let (third, _) = project_with(
        &data,
        Some(&second),
        "300",
        "de-DE",
        &BTreeSet::new(),
        Some(&loc),
    );
    assert_eq!(
        scalar::<String>(
            &third,
            "SELECT v.steam_build FROM battle_passes p JOIN client_version v ON v.id=p.client_version_id WHERE p.id=120"
        ),
        "200"
    );
    assert_eq!(
        scalar::<Option<String>>(&third, "SELECT name_localized FROM titles WHERE id=50"),
        None
    );
    assert_eq!(
        scalar::<String>(
            &third,
            "SELECT localized_state FROM catalogue_texts WHERE family='titles' AND source_id=50 AND field='name'"
        ),
        "fallback_en"
    );
}
