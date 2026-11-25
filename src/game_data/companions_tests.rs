use super::tests::{add, batch, item, project, project_with};
use super::*;
use serde_json::json;

fn fixture() -> BTreeMap<String, TableProbe> {
    let mut data = batch(vec![item(1), item(2)]);
    for name in companions::SOURCES {
        add(&mut data, name, vec![]);
    }
    add(
        &mut data,
        "EnumString",
        vec![json!({"Enum":"ECreatureType","Value":"Intellect","String":{"Key":"item_name"}})],
    );
    add(
        &mut data,
        "VehicleCreatureOption",
        vec![
            json!({"ID":{"Value":1},"GroupName":"options","OptionGrade":"ECreatureOptionGrade::Rare","Prob":0,"StatType":"EStat::Accuracy","StatValueMin":5,"StatValueMax":10}),
        ],
    );
    add(
        &mut data,
        "VehicleStat",
        vec![
            json!({"ID":{"Value":10},"Group":"stats","Level":1,"StatList":[{"StatType":"EStat::HPMax","StatValue":50}]}),
        ],
    );
    add(
        &mut data,
        "VehicleCreatureSlot",
        vec![
            json!({"CreatureType":"ECreatureType::Intellect","BuffTitle":{"Key":"item_desc"},"BuffDesc":{"Key":"item_name"},"SlotOpenLevel":[1,3],"SlotOptionGroup":["options"],"LockCost":[8],"LockCostGold":[2000,4000],"GoldCount":1000,"SoulCrystalCount":4}),
        ],
    );
    add(
        &mut data,
        "VehicleCreatureLevel",
        vec![
            json!({"ID":{"Value":20},"CreatureType":"ECreatureType::Intellect","Level":1,"NeedExp":500,"GetExp":100,"Prob_Common":9000,"Prob_Rare":1000,"Prob_Unique":0}),
        ],
    );
    add(
        &mut data,
        "VehicleLevel",
        vec![json!({"Level":1,"NeedSoulCount":25,"GetCreatureExp":500})],
    );
    add(
        &mut data,
        "VehicleList",
        vec![
            json!({"ID":{"Value":1001},"Name":"pet","Desc":{"Key":"item_name"},"CreatureType":"ECreatureType::Intellect","VehicleItemName":"item_1","SoulItemName":"item_2","StatGroupName":"stats","NeedSoul":5,"bIsOpen":false,"bPolymorphPlan":false,"ShowDate":{"Ticks":0},"EquipStatList":[{"Stat":"EStat::Accuracy","Value":5}]}),
        ],
    );
    add(
        &mut data,
        "VehicleCollection",
        vec![
            json!({"ID":{"Value":1},"CollectNum":5,"RewardItem":"item_2","RewardItemCount":0,"RewardVehicle":"pet"}),
        ],
    );
    add(
        &mut data,
        "WingEnchantEffect",
        vec![
            json!({"EffectGroup":"effect","Level":0,"Stats":{"Entries":[{"Key":"EStat::HPMax","Value":0}],"Removed":[]}}),
            json!({"EffectGroup":"effect","Level":1,"Stats":{"Entries":[{"Key":"EStat::HPMax","Value":50}],"Removed":[]}}),
        ],
    );
    add(
        &mut data,
        "WingEnchant",
        vec![
            json!({"EnchantGroup":"unlinked","Level":1,"BasicProb":10000,"AddProb":0,"MaxProb":10000,"CostItemList":[{"CostItemName":"item_1","CostItemCount":100,"AddProbItemCount":0}]}),
        ],
    );
    add(
        &mut data,
        "Wing",
        vec![
            json!({"ID":{"Value":1001},"Name":"wing","Desc":{"Key":"item_name"},"EffectGroup":"effect","EnchantGroup":"None","Grade":"EItemGrade::Legend","PcRace":"ERace::Light","EquipStatList":[{"Stat":"EStat::Accuracy","Value":300}],"bShow":false}),
        ],
    );
    add(
        &mut data,
        "Skin",
        vec![
            json!({"ID":{"Value":1001},"Name":"skin","DescLight":{"Key":"item_name"},"DescDark":{"Key":"item_desc"},"Grade":"ESkinGrade::Legend","SkinCategory":"ESkinCategory::Greatsword","SkillAbnormal":{"Value":0},"IconLightMale":"male","IconLightFemale":"female","StatList":[{"Type":"EStat::PvPAmplifyDamage","Value":300}],"bShow":true,"bIsMultiOwn":false,"bCanDeco":false}),
        ],
    );
    add(
        &mut data,
        "SkinSet",
        vec![
            json!({"ID":{"Value":1001},"Name":"set","DescLight":{"Key":"item_name"},"DescDark":{"Key":"item_desc"},"Skins":["skin","missing_skin"],"bShow":false}),
        ],
    );
    add(
        &mut data,
        "SkinCollection",
        vec![
            json!({"SkinCategory":"ESkinCategory::Greatsword","CollectPoint":1,"RewardStats":[{"Stat":"EStat::Accuracy","StatValue":10}]}),
        ],
    );
    add(
        &mut data,
        "SkinCombine",
        vec![
            json!({"Grade":"ESkinGrade::Legend","ResultGroup":"ESkinCombineResultGroup::Base","Prob":502513,"SkinId":{"Value":0},"WeaponSkinIds":[{"Value":1001},{"Value":0},{"Value":9999}]}),
        ],
    );
    add(
        &mut data,
        "SkinCombineCost",
        vec![
            json!({"Grade":"ESkinGrade::Legend","CostCount":600,"CostTypeCurrency":"ECurrencyCategory::GoldCombined","CostTypeItem":{"Value":0}}),
        ],
    );
    add(
        &mut data,
        "SkinGacha",
        vec![
            json!({"ID":{"Value":1001},"Name":"TEST_gacha","ResultType":"ESkinGachaResultType::Skin","ResultName":"skin","Prob":10000}),
        ],
    );
    add(
        &mut data,
        "SkinGachaGroup",
        vec![json!({"Group":"test","GachaName":"TEST_gacha","Prob":10000})],
    );
    data
}

#[test]
fn pet_sources_keep_family_labels_independent_and_all_parallel_array_entries() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    let values:(String,String,i64,i64,bool)=db.query_row("SELECT f.name_en,f.buff_title_en,p.id,p.item_id,p.is_open FROM pets p JOIN pet_families f ON f.id=p.family_id",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap();
    assert_eq!(
        values,
        (
            "Name".into(),
            "{se:1:effect_value01:none}".into(),
            1001,
            1,
            false
        )
    );
    assert_eq!(
        db.query_row(
            "SELECT open_level,option_group_id FROM pet_family_slots WHERE position=1",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?))
        )
        .unwrap(),
        (3, None)
    );
    assert_eq!(
        db.query_row(
            "SELECT soul_count_raw,currency_count_raw FROM pet_family_lock_costs WHERE position=1",
            [],
            |r| Ok((r.get::<_, Option<i64>>(0)?, r.get::<_, i64>(1)?))
        )
        .unwrap(),
        (None, 4000)
    );
    assert_eq!(
        db.query_row(
            "SELECT weight_raw FROM pet_grade_weights WHERE grade_source='Unique'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        db.query_row(
            "SELECT reward_pet_id,reward_item_count_raw FROM pet_collections",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        )
        .unwrap(),
        (1001, 0)
    );
}

#[test]
fn wing_effects_do_not_imply_an_enchant_group_or_merge_equipped_stats() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    assert_eq!(
        db.query_row("SELECT enchant_group_id FROM wings", [], |r| r
            .get::<_, Option<i64>>(0))
            .unwrap(),
        None
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM wing_enchant_costs", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        db.query_row(
            "SELECT value_raw FROM wing_effect_stats WHERE level=0",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        db.query_row(
            "SELECT value_raw FROM appearance_stats WHERE family='wings'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        300
    );
}

#[test]
fn appearance_variants_sets_and_unknown_results_keep_identity_and_source_encoding() {
    let (db, _) = project(&fixture(), None, "100", "fr-FR");
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM skin_variants WHERE skin_id=1001",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        4
    );
    assert_eq!(
        db.query_row(
            "SELECT name_en FROM skin_variants WHERE race_id=3 LIMIT 1",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "{se:1:effect_value01:none}"
    );
    assert_eq!(
        db.query_row(
            "SELECT skin_id FROM skin_set_members WHERE position=1",
            [],
            |r| r.get::<_, Option<i64>>(0)
        )
        .unwrap(),
        None
    );
    let result:(i64,i64,Option<i64>)=db.query_row("SELECT position,weight_raw,skin_id FROM skin_combine_results WHERE skin_reference=9999",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(result, (3, 502513, None));
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM skin_gacha_results WHERE source_key='TEST_gacha'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(db.query_row("SELECT resolution_state FROM catalogue_references WHERE family='skin_sets' AND source_path='Skins/1'",[],|r|r.get::<_,String>(0)).unwrap(),"missing");
}

#[test]
fn granting_items_read_only_the_target_parameter_and_keep_item_ids_distinct() {
    let mut data = fixture();
    let mut grant = item(3);
    grant["$variant"] =
        json!({"UseEffect":"EItemUseEffect::GetVehicleSoul","UseEffectValues":["1001","1"]});
    data.get_mut("Item").unwrap().layout.rows.push(grant);
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM catalogue_references WHERE family='appearance_grants'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        db.query_row(
            "SELECT item_id FROM catalogue_item_links WHERE family='pets' AND source_name='Item'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        3
    );
    data.get_mut("Item").unwrap().layout.rows[2]["SellPrice"] = json!(999);
    let (next, _) = project(&data, Some(&db), "200", "fr-FR");
    assert_eq!(
        next.query_row(
            "SELECT v.steam_build FROM pets p JOIN client_version v ON v.id=p.client_version_id",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "100"
    );
}

#[test]
fn changes_follow_progression_dependencies_and_ignore_source_order_and_locale() {
    let mut data = fixture();
    let (first, _) = project(&data, None, "100", "fr-FR");
    data.get_mut("WingEnchantEffect")
        .unwrap()
        .layout
        .rows
        .reverse();
    let (reordered, _) = project(&data, Some(&first), "200", "fr-FR");
    assert_eq!(reordered.query_row("SELECT v.steam_build FROM wings w JOIN client_version v ON v.id=w.client_version_id",[],|r|r.get::<_,String>(0)).unwrap(),"100");
    data.get_mut("WingEnchantEffect").unwrap().layout.rows[0]["Stats"]["Entries"][0]["Value"] =
        json!(60);
    let (changed, _) = project(&data, Some(&reordered), "300", "fr-FR");
    assert_eq!(changed.query_row("SELECT v.steam_build FROM wings w JOIN client_version v ON v.id=w.client_version_id",[],|r|r.get::<_,String>(0)).unwrap(),"300");
    let localized = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::new(),
    };
    let (switched, _) = project_with(
        &data,
        Some(&changed),
        "400",
        "de-DE",
        &BTreeSet::new(),
        Some(&localized),
    );
    assert_eq!(switched.query_row("SELECT v.steam_build FROM wings w JOIN client_version v ON v.id=w.client_version_id",[],|r|r.get::<_,String>(0)).unwrap(),"300");
    assert_eq!(
        switched
            .query_row("SELECT name_localized FROM pets", [], |r| r
                .get::<_, Option<String>>(0))
            .unwrap(),
        None
    );
}

#[test]
fn option_weights_and_collection_bonuses_modify_their_related_definitions() {
    let mut data = fixture();
    let (first, _) = project(&data, None, "100", "fr-FR");
    data.get_mut("VehicleCreatureOption").unwrap().layout.rows[0]["Prob"] = json!(50);
    data.get_mut("SkinCollection").unwrap().layout.rows[0]["RewardStats"][0]["StatValue"] =
        json!(20);
    let (next, _) = project(&data, Some(&first), "200", "fr-FR");
    for table in ["pets", "pet_options", "skins"] {
        assert_eq!(next.query_row(&format!("SELECT v.steam_build FROM {table} t JOIN client_version v ON v.id=t.client_version_id"),[],|r|r.get::<_,String>(0)).unwrap(),"200");
    }
    assert_eq!(
        next.query_row(
            "SELECT v.steam_build FROM wings w JOIN client_version v ON v.id=w.client_version_id",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "100"
    );
}
