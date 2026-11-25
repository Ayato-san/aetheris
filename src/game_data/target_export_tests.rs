use super::*;
use serde_json::json;

fn batch(items: Vec<Value>) -> BTreeMap<String, TableProbe> {
    let mut all = BTreeMap::new();
    for (name, structure) in skills::TABLES
        .iter()
        .copied()
        .chain([("Item", "ItemTableItem")])
    {
        all.insert(
            name.into(),
            TableProbe {
                mapping_sha256: "a".repeat(64),
                mapping_version: 4,
                mapping_ambiguous_structures: BTreeSet::new(),
                row_struct: structure.into(),
                payload_sha256: "b".repeat(64),
                payload_bytes: 0,
                layout: super::super::schema::LayoutProbe {
                    compatibility_verified: false,
                    table_version: 13,
                    column_hashes: vec![],
                    rows: vec![],
                },
            },
        );
    }
    all.get_mut("Item").unwrap().layout.rows = items;
    all
}
fn add(batch: &mut BTreeMap<String, TableProbe>, name: &str, rows: Vec<Value>) {
    let mut table: TableProbe =
        serde_json::from_value(serde_json::to_value(&batch["Item"]).unwrap()).unwrap();
    table.row_struct = format!("{name}TableItem");
    table.layout.rows = rows;
    batch.insert(name.into(), table);
}
fn item(id: i64) -> Value {
    json!({"ID":{"Value":id},"Name":format!("item_{id}"),"Desc":{"Key":"item_name"},"DescLong":{"Key":"item_desc"},"IconRes":"shared","ItemTier":"EItemTier::t3","PermitLevelMin":10,"PermitLevelMax":99,"CanSell":true,"CanLock":false,"$variant":{}})
}

fn crafting_fixture() -> BTreeMap<String, TableProbe> {
    let mut data = batch(vec![item(1), item(2)]);
    add(
        &mut data,
        "CraftRecipe",
        vec![
            json!({"ID":{"Value":1},"Name":"recipe_a","Product":"item_1","ProductCount":2,"Category":"ECraftCategory::Alchemy","MasteryLevel":55,"QualificationRace":"ERace::Light","ComboRecipe":"recipe_combo","ComboProbability":2500,"CraftRateGroup":"rate_a","CraftingFee":0,"CraftingFeeType":"ECurrencyCategory::GoldCombined","RemoteCraftingFee":10,"bShowRecipe":true,"bGuildCraft":false,"LimitCharCount":0,"ComponentList":[{"Item":"item_2","Count":3,"Enchant":0,"ReplacementItem":"alternatives","CombineName":"None"}]}),
            json!({"ID":{"Value":2},"Name":"recipe_combo","Product":"item_2","ProductCount":1}),
        ],
    );
    add(
        &mut data,
        "ConversionRecipe",
        vec![
            json!({"ID":{"Value":1},"Name":"conversion_a","Product":"item_2","ProductCount":1,"SuccessProp":10000,"CriticalSuccessProduct":"item_1","CriticalSuccessProductCount":2,"CriticalSuccessProp":100,"RegisterType":"EConversionRegisterType::BlacksmithingMastery","RegisterValue":30,"ComponentList":[{"Item":"missing_item","Count":0,"ReplacementItem":"absent_group"}]}),
        ],
    );
    add(
        &mut data,
        "CraftComponentReplacement",
        vec![
            json!({"ID":{"Value":10},"Name":"alternatives","ReplacementItemList":[{"Item":"item_1","Count":4,"EnchantedLevel":0}]}),
        ],
    );
    add(
        &mut data,
        "CraftRecipeRate",
        vec![json!({"Name":"rate_a","Rates":[{"Gap":0,"Rate":74011698}]})],
    );
    data
}

fn quest_fixture() -> BTreeMap<String, TableProbe> {
    let mut data = batch(vec![item(1), item(2)]);
    add(
        &mut data,
        "Quest",
        vec![
            json!({"ID":{"Value":11},"Name":"quest_a","QuestText":"quest_text","Part":"part_a","NextQuestName":"quest_b","Type":"EQuestType::Hero","Grade":"EItemGrade::Rare","Race":"ERace::Light","UnlockLevel":1,"RecommendedLevel":5,"RepeatType":"EQuestRepeatType::None","bEnableGiveup":false,"CompleteMapId":{"Value":21},"AcquirableConditions":[{"Type":"EQuestAcquirableCondition::ClearQuest","Value":"quest_b"},{"Type":"EQuestAcquirableCondition::ClearQuestStep","Value":"31"}]}),
            json!({"ID":{"Value":12},"Name":"quest_b"}),
        ],
    );
    add(
        &mut data,
        "QuestPart",
        vec![
            json!({"ID":{"Value":20},"Name":"part_a","PartOrder":0,"PartText":{"Key":"item_name"}}),
        ],
    );
    add(
        &mut data,
        "QuestString",
        vec![
            json!({"Name":"quest_text","Title":{"Key":"item_name"},"ShortSummary":{"Key":"item_desc"},"ObjDescArray":[{"StepDesc":{"Key":"item_desc"},"ObjDescList":[{"Key":"item_name"}]}]}),
        ],
    );
    add(
        &mut data,
        "QuestType",
        vec![json!({"Type":"EQuestType::Hero","GetLimitCount":1})],
    );
    add(
        &mut data,
        "QuestStep",
        vec![
            json!({"ID":{"Value":30},"QuestName":"quest_a","Order":1,"FailGoToStep":2,"FailConditionType":"EQuestFailCondition::None","GoalList":[{"Type":"EQuestObjectiveType::ClearQuest","Value":["quest_b","2","","", ""],"MapId":{"Value":21},"bOptional":true,"bHideUI":false,"bMarker":true}]}),
            json!({"ID":{"Value":31},"QuestName":"quest_a","Order":2,"FailGoToStep":0,"GoalList":[]}),
            json!({"ID":{"Value":32},"QuestName":"quest_b","Order":2,"GoalList":[]}),
        ],
    );
    add(
        &mut data,
        "Map",
        vec![json!({"ID":{"Value":1000},"Name":"map_a"})],
    );
    add(
        &mut data,
        "QuestReward",
        vec![
            json!({"Group":"11","ExpReward":1133,"ItemRewards":[{"Item":"item_1","Count":0,"EnchantedLevel":0}],"ItemSelectRewards":[{"Item":"item_2","Count":2}],"RandomReward":"pool_a"}),
        ],
    );
    add(
        &mut data,
        "QuestClassReward",
        vec![
            json!({"Group":"11","Class":"ECharacterClass::Gladiator","ItemNormalRewards":[{"Item":"item_1","Count":1}]}),
            json!({"Group":"11","Class":"ECharacterClass::Templar","ItemNormalRewards":[{"Item":"item_2","Count":1}]}),
        ],
    );
    add(&mut data, "RewardPackage", vec![json!({"Name":"pool_a"})]);
    data
}

#[test]
fn recipes_scope_ids_keep_substitutions_raw_rates_costs_and_unresolved_references() {
    let data = crafting_fixture();
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert_eq!(
        db.query_row("SELECT count(*) FROM recipes WHERE id=1", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
    let recipe:(i64,i64,i64,bool,bool)=db.query_row("SELECT mastery_level,combo_id,combo_probability_raw,show_recipe,guild_craft FROM recipes WHERE source_name='CraftRecipe' AND id=1",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap();
    assert_eq!(recipe, (55, 2, 2500, true, false));
    assert_eq!(
        db.query_row("SELECT rate_raw FROM recipe_rates", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        74011698
    );
    let cost: (i64, Option<i64>) = db
        .query_row(
            "SELECT amount_raw,currency_id FROM recipe_costs WHERE kind='RemoteCraftingFee'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(cost, (10, None));
    assert_eq!(
        db.query_row(
            "SELECT amount_raw FROM recipe_costs WHERE kind='CraftingFee'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    let missing:(String,Option<i64>,i64)=db.query_row("SELECT item_reference,item_id,quantity FROM recipe_components WHERE source_name='ConversionRecipe'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(missing, ("missing_item".into(), None, 0));
    let option:(i64,i64)=db.query_row("SELECT item_id,quantity FROM recipe_component_options WHERE group_reference='alternatives'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(option, (1, 4));
    assert_eq!(db.query_row("SELECT resolution_state FROM catalogue_references WHERE source_name='ConversionRecipe' AND source_path='ComponentList/0/ReplacementItem'",[],|r|r.get::<_,String>(0)).unwrap(),"not_researched");
    assert!(
        db.execute(
            "INSERT INTO catalogue_codes(id,code) VALUES(32768,'outside_bound')",
            []
        )
        .is_err()
    );
}

#[test]
fn quests_keep_branches_ordered_objectives_original_markers_and_class_reward_variants() {
    let (db, _) = project(&quest_fixture(), None, "100", "fr-FR");
    let quest: (i64, i64, String) = db
        .query_row(
            "SELECT part_id,next_quest_id,description_en FROM quests WHERE id=11",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(quest, (20, 12, "{se:1:effect_value01:none}".into()));
    assert_eq!(
        db.query_row(
            "SELECT fail_go_to_step_id FROM quest_steps WHERE id=30",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        31
    );
    let objective: (String, bool, i64) = db
        .query_row(
            "SELECT values_json,optional,map_source_id FROM quest_objectives",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&objective.0).unwrap(),
        json!(["quest_b", "2", "", "", ""])
    );
    assert_eq!((objective.1, objective.2), (true, 21));
    assert_eq!(db.query_row("SELECT resolution_state FROM catalogue_references WHERE family='quests' AND source_path='CompleteMapId'",[],|r|r.get::<_,String>(0)).unwrap(),"missing");
    assert_eq!(
        db.query_row(
            "SELECT count(DISTINCT class_id) FROM quest_reward_groups WHERE quest_id=11",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        2
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM quest_rewards", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        4
    );
    assert_eq!(db.query_row("SELECT quantity FROM quest_rewards WHERE source_name='QuestReward' AND mode='ItemRewards'",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    let pool:(String,String)=db.query_row("SELECT target_table,interpretation_state FROM catalogue_references WHERE source_path='RandomReward'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(pool, ("RewardPackage".into(), "candidate".into()));
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM quest_texts WHERE source_path LIKE '/ObjDescArray/%'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        2
    );
}

#[test]
fn missing_quest_translation_keeps_english_fallback_separate_from_original_text() {
    let localized = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::new(),
    };
    let (db, _) = project_with(
        &quest_fixture(),
        None,
        "100",
        "fr-FR",
        &BTreeSet::new(),
        Some(&localized),
    );
    let names: (Option<String>, Option<String>) = db
        .query_row(
            "SELECT name_en,name_localized FROM quests WHERE id=11",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(names, (Some("Name".into()), None));
    assert_eq!(db.query_row("SELECT localized_state FROM catalogue_texts WHERE family='quests' AND source_id=11 AND field='/Title'",[],|r|r.get::<_,String>(0)).unwrap(),"fallback_en");
}

#[test]
fn recipe_rates_and_quest_steps_participate_in_latest_build_changes() {
    let mut data = crafting_fixture();
    let quests = quest_fixture();
    for (name, table) in quests {
        if name != "Item" && !skills::TABLES.iter().any(|(n, _)| *n == name) {
            data.insert(name, table);
        }
    }
    let (first, _) = project(&data, None, "100", "fr-FR");
    assert_eq!(first.query_row("SELECT count(*) FROM catalogue_entries WHERE family IN('craft_recipes','conversion_recipes','quests') AND change_type IS NOT NULL",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    data.get_mut("CraftRecipeRate").unwrap().layout.rows[0]["Rates"][0]["Rate"] = json!(75000000);
    data.get_mut("QuestStep").unwrap().layout.rows[0]["GoalList"][0]["Value"][1] = json!("3");
    let (second, _) = project(&data, Some(&first), "200", "fr-FR");
    for (family, id) in [("craft_recipes", 1), ("quests", 11)] {
        let change:(String,String)=second.query_row("SELECT e.change_type,v.steam_build FROM catalogue_entries e JOIN client_version v ON v.id=e.change_reference_build_id WHERE family=?1 AND source_id=?2",params![family,id],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!(change, ("modified".into(), "100".into()));
    }
    let (third, _) = project(&data, Some(&second), "300", "fr-FR");
    assert_eq!(third.query_row("SELECT v.steam_build FROM quests q JOIN client_version v ON v.id=q.client_version_id WHERE q.id=11",[],|r|r.get::<_,String>(0)).unwrap(),"200");
}

#[test]
fn ambiguous_item_names_never_choose_an_arbitrary_recipe_product() {
    let mut data = crafting_fixture();
    data.get_mut("Item").unwrap().layout.rows[1]["Name"] = json!("item_1");
    let (db, _) = project(&data, None, "100", "fr-FR");
    let result:(Option<i64>,String)=db.query_row("SELECT r.item_id,l.resolution_state FROM recipe_results r JOIN catalogue_references l ON l.source_name=r.source_name AND l.source_id=r.recipe_id AND l.source_path='Product' WHERE r.source_name='CraftRecipe' AND r.recipe_id=1",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(result, (None, "ambiguous".into()));
}

#[test]
fn deleting_and_readding_a_recipe_preserves_its_scoped_identity_and_last_change() {
    let mut data = crafting_fixture();
    let (first, _) = project(&data, None, "100", "fr-FR");
    let removed = data.get_mut("CraftRecipe").unwrap().layout.rows.remove(0);
    let (second, _) = project(&data, Some(&first), "200", "fr-FR");
    let deleted:(bool,String)=second.query_row("SELECT present,change_type FROM catalogue_entries WHERE family='craft_recipes' AND source_id=1",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(deleted, (false, "deleted".into()));
    assert_eq!(
        second
            .query_row(
                "SELECT count(*) FROM recipes WHERE id=1 AND source_name='ConversionRecipe'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    data.get_mut("CraftRecipe")
        .unwrap()
        .layout
        .rows
        .insert(0, removed);
    let (third, _) = project(&data, Some(&second), "300", "fr-FR");
    let returned:(bool,String,String)=third.query_row("SELECT e.present,e.change_type,v.steam_build FROM catalogue_entries e JOIN client_version v ON v.id=e.change_reference_build_id WHERE family='craft_recipes' AND source_id=1",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(returned, (true, "added".into(), "200".into()));
}
fn project(
    batch: &BTreeMap<String, TableProbe>,
    old: Option<&Connection>,
    build: &str,
    locale: &str,
) -> (Connection, Summary) {
    project_with(batch, old, build, locale, &BTreeSet::new(), None)
}
fn project_with(
    batch: &BTreeMap<String, TableProbe>,
    old: Option<&Connection>,
    build: &str,
    locale: &str,
    retained: &BTreeSet<String>,
    localized: Option<&Localization>,
) -> (Connection, Summary) {
    let mut db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!("items.sql")).unwrap();
    db.execute_batch("PRAGMA foreign_keys=OFF;
 INSERT INTO item_families VALUES(1,'Equip'); INSERT INTO item_categories VALUES(1,1,'Greatsword');
 INSERT INTO rarities VALUES(1,'Common','EItemGrade::Common'); INSERT INTO sell_currencies VALUES(1,'Gold','ECurrencyCategory::Gold');").unwrap();
    db.execute("INSERT INTO client_version VALUES(1,?1)", [build])
        .unwrap();
    db.execute("INSERT INTO localization_language VALUES(1,?1)", [locale])
        .unwrap();
    let tx = db.transaction().unwrap();
    let l10n = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::from([
            ("item_name".into(), "Name".into()),
            ("item_desc".into(), "{se:1:effect_value01:none}".into()),
            ("npc_name".into(), "Creature".into()),
        ]),
    };
    let localized = localized.unwrap_or(&l10n);
    for row in &batch["Item"].layout.rows {
        let id = number(row, "ID").unwrap();
        tx.execute("INSERT INTO items(id,client_version_id,category_id,rarity_id,sell_currency_id,item_level,sell_price,name_en,name_localized) VALUES(?1,1,1,1,1,1,0,'Name',?2)",params![id,text(localized,Some("item_name"))]).unwrap();
        if row["CanSell"].as_bool() == Some(true) {
            tx.execute("INSERT OR IGNORE INTO flags VALUES(1,'CanSell')", [])
                .unwrap();
            tx.execute("INSERT INTO item_flags VALUES(?1,1)", [id])
                .unwrap();
        }
    }
    super::super::skill_database::write(&tx, batch, &l10n, localized, 1).unwrap();
    tx.execute_batch(include_str!("catalogue.sql")).unwrap();
    tx.execute_batch(include_str!("crafting_quests.sql"))
        .unwrap();
    for (name, table) in batch {
        let data =
            lz4_flex::compress_prepend_size(&serde_json::to_vec(&table.layout.rows).unwrap());
        tx.execute(
            "INSERT INTO catalogue_sources VALUES(?1,1,?2,?3,?4,0,'decoded',?5,?6)",
            params![
                name,
                table.mapping_sha256,
                table.payload_sha256,
                table.row_struct,
                table.layout.rows.len() as i64,
                data
            ],
        )
        .unwrap();
    }
    loot_changes::prepare(&tx, old, batch, 1).unwrap();
    projection::write(&tx, batch, &l10n, localized, None).unwrap();
    crafting_quests::write(&tx, batch, &l10n, localized, 1).unwrap();
    let mut summary = Summary::default();
    changes::record(
        &tx,
        old,
        batch,
        retained,
        1,
        locale,
        &mut summary,
        &l10n,
        localized,
    )
    .unwrap();
    tx.commit().unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON").unwrap();
    integrity(&db).unwrap();
    (db, summary)
}
#[test]
fn rarity_dictionary_uses_client_english_names_and_keeps_the_original_source() {
    let mut data = batch(vec![item(1)]);
    add(
        &mut data,
        "EnumString",
        vec![json!({"Enum":"EItemGrade","Value":"Legend","String":{"Key":"item_name"}})],
    );
    let (db, _) = project(&data, None, "100", "fr-FR");
    db.execute(
        "UPDATE rarities SET rarity='Legend',rarity_source='EItemGrade::Legend'",
        [],
    )
    .unwrap();
    let en = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::from([("item_name".into(), "Epic".into())]),
    };
    let loc = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::from([("item_name".into(), "Épique".into())]),
    };
    db.execute_batch("DELETE FROM catalogue_labels; DELETE FROM item_details; DELETE FROM item_false_flags; DELETE FROM catalogue_entity_icons; DELETE FROM catalogue_texts;").unwrap();
    projection::write(&db, &data, &en, &loc, None).unwrap();
    let rarity: (String, String) = db
        .query_row("SELECT rarity,rarity_source FROM rarities", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(rarity, ("Epic".into(), "EItemGrade::Legend".into()));
}

#[test]
fn sell_currency_icons_follow_currency_info_category_not_display_order() {
    let mut data = batch(vec![item(1)]);
    add(
        &mut data,
        "CurrencyInfo",
        vec![
            json!({"CurrencyCategory":"ECurrencyCategory::GoldCombined","IconResource":"Currency_Gold"}),
            json!({"CurrencyCategory":"ECurrencyCategory::Gold","IconResource":"Currency_Goldbinding"}),
        ],
    );
    let (db, _) = project(&data, None, "100", "fr-FR");
    let reference: String = db.query_row(
        "SELECT c.reference FROM sell_currencies m JOIN catalogue_entity_icons e ON e.family='sell_currencies' AND e.source_id=m.id JOIN catalogue_icons c ON c.id=e.icon_id WHERE m.sell_currency_type='ECurrencyCategory::Gold'",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(reference, "Currency_Goldbinding");
    assert_eq!(
        db.query_row(
            "SELECT COUNT(*) FROM catalogue_entity_icons WHERE family='sell_currencies'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
}

#[test]
fn target_changes_track_initial_add_modify_delete_reappearance_and_last_data_build() {
    let first = batch(vec![item(1), item(2)]);
    let (a, s) = project(&first, None, "100", "fr-FR");
    assert_eq!((s.initial, s.added, s.modified, s.deleted), (2, 0, 0, 0));
    let mut changed = item(1);
    changed["PermitLevelMin"] = json!(11);
    let second = batch(vec![changed, item(3)]);
    let (b, s) = project(&second, Some(&a), "200", "fr-FR");
    assert_eq!((s.added, s.modified, s.deleted), (1, 1, 1));
    let (c, s) = project(&second, Some(&b), "300", "fr-FR");
    assert_eq!((s.unchanged, s.modified, s.deleted), (2, 0, 0));
    let build:String=c.query_row("SELECT v.steam_build FROM items i JOIN client_version v ON v.id=i.client_version_id WHERE i.id=1",[],|r|r.get(0)).unwrap();
    assert_eq!(build, "200");
    let mut third = batch(vec![item(1), item(2), item(3)]);
    third.get_mut("Item").unwrap().layout.rows[0] = second["Item"].layout.rows[0].clone();
    let (d, s) = project(&third, Some(&c), "400", "fr-FR");
    assert_eq!((s.added, s.modified, s.deleted), (1, 0, 0));
    let change:(String,String,String)=d.query_row("SELECT e.change_type,v.steam_build,r.steam_build FROM catalogue_entries e JOIN client_version v ON v.id=e.change_build_id JOIN client_version r ON r.id=e.change_reference_build_id WHERE family='items' AND source_id=2",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(change, ("added".into(), "400".into(), "300".into()));
}
#[test]
fn target_language_and_source_coverage_changes_do_not_become_game_changes() {
    let first = batch(vec![item(1), item(2)]);
    let (a, _) = project(&first, None, "100", "fr-FR");
    let (b, s) = project(&first, Some(&a), "200", "es-ES");
    assert_eq!((s.modified, s.unchanged), (0, 2));
    let mut wider = batch(vec![item(1)]);
    add(&mut wider, "ItemSet", vec![]);
    let (c, s) = project(&wider, Some(&b), "300", "es-ES");
    assert_eq!((s.modified, s.deleted, s.comparison_impossible), (0, 0, 2));
    let state:(bool,String)=c.query_row("SELECT present,comparison_state FROM catalogue_entries WHERE family='items' AND source_id=2",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(state, (true, "impossible".into()));
}
#[test]
fn target_preserves_unknown_fields_false_permissions_and_shared_icon_references() {
    let mut r = item(1);
    r["UnknownSource"] = json!({"Removed":["a"],"Entries":[{"Key":"x","Value":"raw"}]});
    let mut first = batch(vec![r.clone(), item(2)]);
    let skill = json!({"ID":{"Value":10},"SkillLvMax":1,"SkillType":"ESkillType::Active","SkillGrade":"EItemGrade::Common","SkillEffectTimeDataList":[]});
    add(&mut first, "Skill", vec![skill.clone()]);
    let (db, _) = project(&first, None, "100", "fr-FR");
    assert_eq!(source_rows(&db, "Item").unwrap()[0], r);
    assert_eq!(source_rows(&db, "Skill").unwrap()[0], skill);
    assert_eq!(item_permission(&db, 1, "CanSell").unwrap(), Some(true));
    assert_eq!(item_permission(&db, 1, "CanLock").unwrap(), Some(false));
    assert_eq!(item_permission(&db, 1, "CanEnchant").unwrap(), None);
    assert_eq!(
        db.query_row("SELECT count(*) FROM catalogue_icons", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        localized_text(None, Some("English")),
        (Some("English".into()), true)
    );
    assert_eq!(localized_text(None, None), (None, false));
    assert!(decode_rows(&u32::MAX.to_le_bytes()).is_err());
    assert!(
        db.execute(
            "INSERT INTO catalogue_icons(id,reference) VALUES(32768,'bad')",
            []
        )
        .is_err()
    );
}
#[test]
fn target_preserves_box_choices_weights_duplicate_set_definitions_and_unresolved_routes() {
    let mut box_item = item(1);
    box_item["$variant"] =
        json!({"UseEffect":"EItemUseEffect::RewardBox","UseEffectValues":["box"]});
    let mut first = batch(vec![box_item, item(2)]);
    add(
        &mut first,
        "BoxReward",
        vec![
            json!({"Group":"box","SelectItems":[{"Item":"item_2","Count":2}],"PackageItems":[{"Item":"package","Count":1,"Prob":1234}],"LevelMin":5}),
        ],
    );
    add(
        &mut first,
        "ItemGetRoute",
        vec![
            json!({"ItemId":{"Value":2},"MonsterGetRoutes":[{"NpcId":{"Value":9999},"RegionInfo":{"MapId":{"Value":12}}}]}),
        ],
    );
    add(
        &mut first,
        "ItemSet",
        vec![
            json!({"SetName":"set","SetItems":["item_1"]}),
            json!({"SetName":"set","SetItems":["item_2"]}),
        ],
    );
    let (db, _) = project(&first, None, "100", "fr-FR");
    assert_eq!(
        db.query_row("SELECT count(*) FROM item_set_definitions", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM item_set_members", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
    let reward:(i64,String,i64)=db.query_row("SELECT quantity_min,mode,result_item_id FROM item_rewards WHERE result_reference='item_2'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(reward, (2, "choice".into(), 2));
    assert_eq!(
        db.query_row(
            "SELECT weight_raw FROM item_rewards WHERE result_reference='package'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "1234"
    );
    assert!(
        !db.query_row(
            "SELECT resolved FROM item_obtaining_routes WHERE target_table='NpcData'",
            [],
            |r| r.get::<_, bool>(0)
        )
        .unwrap()
    );
    let mut changed = batch(vec![item(1), item(2)]);
    add(
        &mut changed,
        "BoxReward",
        source_rows(&db, "BoxReward").unwrap(),
    );
    add(
        &mut changed,
        "ItemGetRoute",
        source_rows(&db, "ItemGetRoute").unwrap(),
    );
    add(
        &mut changed,
        "ItemSet",
        source_rows(&db, "ItemSet").unwrap(),
    );
    changed.get_mut("ItemGetRoute").unwrap().layout.rows[0]["MonsterGetRoutes"][0]["NpcId"] =
        json!({"Value":10000});
    let (_, summary) = project(&changed, Some(&db), "200", "fr-FR");
    assert!(summary.modified >= 1);
}

#[test]
fn target_changes_include_unknown_fields_of_related_effects_only() {
    let mut first = batch(vec![item(1)]);
    first.get_mut("Skill").unwrap().layout.rows = vec![
        json!({"ID":{"Value":10},"SkillLvMax":1,"SkillType":"ESkillType::Active","SkillGrade":"EItemGrade::Rare","SkillEffectTimeDataList":[{"SkillEffectGroupId":{"Value":1}}]}),
    ];
    first.get_mut("SkillEffect").unwrap().layout.rows = vec![
        json!({"ID":{"Value":11},"SkillEffectGroupId":{"Value":1},"EffectType":"ESkillEffectType::Damage","EffectValueList":["100"],"UnknownClientRule":"old"}),
        json!({"ID":{"Value":12},"SkillEffectGroupId":{"Value":2},"EffectType":"ESkillEffectType::Damage","EffectValueList":["200"]}),
    ];
    let (a, _) = project(&first, None, "100", "fr-FR");
    first.get_mut("SkillEffect").unwrap().layout.rows[1]["EffectValueList"] = json!(["999"]);
    let (b, s) = project(&first, Some(&a), "200", "fr-FR");
    assert_eq!(s.modified, 0);
    first.get_mut("SkillEffect").unwrap().layout.rows[0]["UnknownClientRule"] = json!("new");
    let (_, s) = project(&first, Some(&b), "300", "fr-FR");
    assert_eq!(s.modified, 1);
}

#[test]
fn stat_dictionary_uses_client_localization_in_both_languages() {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!("items.sql")).unwrap();
    db.execute_batch(include_str!("catalogue.sql")).unwrap();
    db.execute(
        "INSERT INTO stats VALUES(1,'EStat::WeaponAccuracy',NULL,NULL)",
        [],
    )
    .unwrap();
    let en = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::from([(
            "String_StatName_WeaponAccuracy_body".into(),
            "Accuracy".into(),
        )]),
    };
    let loc = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::from([(
            "String_StatName_WeaponAccuracy_body".into(),
            "Précision".into(),
        )]),
    };
    projection::write(&db, &BTreeMap::new(), &en, &loc, None).unwrap();
    let names: (String, String) = db
        .query_row(
            "SELECT name_en,name_localized FROM stats WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(names, ("Accuracy".into(), "Précision".into()));
    let names:(String,String)=db.query_row("SELECT name_en,name_localized FROM catalogue_labels WHERE enum_type='EStat' AND code='WeaponAccuracy'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(names, ("Accuracy".into(), "Précision".into()));
}

#[test]
fn surpass_statistics_are_shared_dependencies_and_modify_only_their_owners() {
    let mut row = item(1);
    row["$variant"]["SurpassGroup"] = json!("surpass");
    let mut data = batch(vec![row, item(2)]);
    add(
        &mut data,
        "ItemSurpass",
        vec![
            json!({"Group":"surpass","CurrentLevel":0,"AdditionalStatName":"bonus"}),
            json!({"Group":"surpass","CurrentLevel":1,"AdditionalStatName":"bonus"}),
            json!({"Group":"surpass","CurrentLevel":2,"AdditionalStatName":"missing"}),
        ],
    );
    add(
        &mut data,
        "AdditionalStat",
        vec![json!({"Name":"bonus","AdditionalStats":[{"Type":"EStat::WeaponDamage","Value":10}]})],
    );
    let (old, _) = project(&data, None, "100", "fr-FR");
    let dependency: (i64,Option<i64>) = old.query_row("SELECT count(*),level FROM item_progression_links WHERE item_id=1 AND source_name='AdditionalStat'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(dependency, (1, None));
    let diagnostic: String = old
        .query_row(
            "SELECT diagnostic FROM catalogue_coverage WHERE subject='items.progression'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(diagnostic.contains("1 unmatched additional-stat references"));
    data.get_mut("AdditionalStat").unwrap().layout.rows[0]["AdditionalStats"][0]["Value"] =
        json!(20);
    let (new, summary) = project(&data, Some(&old), "101", "fr-FR");
    assert_eq!(summary.modified, 1);
    assert_eq!(summary.unchanged, 1);
    let changed: i64 = new
        .query_row(
            "SELECT source_id FROM catalogue_entries WHERE change_type='modified'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(changed, 1);
}

#[test]
fn amplification_stats_are_linked_and_participate_in_item_changes() {
    let mut row = item(1);
    row["$variant"]["ExceedEnchantGroup"] = json!("amp");
    let mut first = batch(vec![row, item(2)]);
    add(
        &mut first,
        "ExceedEnchant",
        vec![json!({"Group":"amp","CurrentLevel":1,"AdditionalStatName":"amp1"})],
    );
    add(
        &mut first,
        "AdditionalStat",
        vec![
            json!({"Name":"amp1","AdditionalStats":[{"Type":"EStat::WeaponFixingDamage","Value":14}]}),
        ],
    );
    let (a, _) = project(&first, None, "100", "fr-FR");
    assert_eq!(a.query_row("SELECT level FROM item_progression_links WHERE item_id=1 AND source_name='AdditionalStat' AND source_field='AdditionalStatName'",[],|r|r.get::<_,i64>(0)).unwrap(),1);
    first.get_mut("AdditionalStat").unwrap().layout.rows[0]["AdditionalStats"][0]["Value"] =
        json!(28);
    let (_, summary) = project(&first, Some(&a), "101", "fr-FR");
    assert_eq!(summary.modified, 1);
    assert_eq!(summary.unchanged, 1);
}

#[test]
fn item_obtaining_indexes_hints_manual_routes_and_contextual_targets() {
    let mut data = batch(vec![item(1)]);
    add(
        &mut data,
        "ItemGetRoute",
        vec![json!({
            "ItemId":{"Value":1}, "bArena":true,"bVehicle":false,
            "HiddenCubeInfo":["ERace::Light"],
            "BossChallengeInfo":{"bBossChallenge":false,"Race":"ERace::All"},
            "AchievementInfo":[{"TableId":{"Value":7},"Step":1},{"TableId":{"Value":70},"Step":1}],
            "RandomShopInfo":[{"Value":8}],
            "UnknownFutureRoute":[{"Value":42}],
            "NPCShopInfo":{"Entries":[{"Key":{"Value":9},"Value":{"MapIdList":[{"Value":10}]}}],"Removed":[{"Value":999}]},
            "CoinShopInfo":{"Entries":[{"Key":"shop","Value":{"GoodsIds":[{"Value":11}]}}],"Removed":[]}
        })],
    );
    add(
        &mut data,
        "ItemGetRouteManual",
        vec![json!({"ItemId":{"Value":1},"Categories":["test"],"Infos":["hint"]})],
    );
    add(&mut data, "Achievement", vec![json!({"AchievementId":7})]);
    add(
        &mut data,
        "AchievementGroup",
        vec![json!({"ID":{"Value":70}})],
    );
    add(
        &mut data,
        "RandomShopGoods",
        vec![json!({"ID":{"Value":8}})],
    );
    add(&mut data, "NpcData", vec![json!({"ID":{"Value":9}})]);
    add(&mut data, "Map", vec![json!({"ID":{"Value":10}})]);
    add(&mut data, "GameShop", vec![json!({"Name":"shop"})]);
    add(&mut data, "GameShopGoods", vec![json!({"ID":{"Value":11}})]);
    let (db, _) = project(&data, None, "100", "fr-FR");
    let kinds:Vec<String> = db.prepare("SELECT DISTINCT t.code FROM item_obtaining_sources s JOIN item_obtaining_types t ON t.id=s.type_id ORDER BY t.code").unwrap().query_map([],|r|r.get(0)).unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(
        kinds,
        vec![
            "AchievementInfo",
            "CoinShopInfo",
            "HiddenCubeInfo",
            "Manual",
            "NPCShopInfo",
            "RandomShopInfo",
            "UnknownFutureRoute",
            "bArena"
        ]
    );
    assert_eq!(
        db.query_row(
            "SELECT COUNT(*) FROM item_obtaining_routes WHERE resolved",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        7
    );
    assert_eq!(
        db.query_row(
            "SELECT target_table FROM item_obtaining_routes WHERE target_reference='70'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "AchievementGroup"
    );
    assert_eq!(
        db.query_row(
            "SELECT target_table FROM item_obtaining_routes WHERE target_reference='42'",
            [],
            |r| r.get::<_, Option<String>>(0)
        )
        .unwrap(),
        None
    );
    assert_eq!(
        db.query_row(
            "SELECT COUNT(*) FROM item_obtaining_routes WHERE target_reference='999'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
}

#[test]
fn item_obtaining_keeps_output_quantities_and_raw_weights_and_excludes_inputs() {
    let mut data = batch(vec![item(1), item(2)]);
    add(
        &mut data,
        "CraftRecipe",
        vec![
            json!({"ID":{"Value":30},"Name":"recipe_30","Product":"item_1","ProductCount":3,"ComponentList":[{"Item":"item_2","Count":4}]}),
        ],
    );
    add(
        &mut data,
        "RewardPackageItem",
        vec![
            json!({"PackageName":"package","Items":[{"ItemName":"item_1","ItemCountMin":2,"ItemCountMax":5,"ItemProb":123456}]}),
        ],
    );
    add(
        &mut data,
        "PeriodItemCollectionList",
        vec![
            json!({"ID":{"Value":40},"RewardItems":[{"Item":"item_1","Count":7}],"TargetItems":[{"Item":"item_2","Count":8}]}),
        ],
    );
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert_eq!(
        db.query_row(
            "SELECT COUNT(*) FROM item_obtaining_sources WHERE item_id=2",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    let values:(i64,i64,String)=db.query_row("SELECT quantity_min,quantity_max,weight_raw FROM item_obtaining_sources WHERE source_name='RewardPackageItem'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(values, (2, 5, "123456".into()));
    assert_eq!(
        db.query_row(
            "SELECT quantity_min FROM item_obtaining_sources WHERE source_name='CraftRecipe'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        3
    );
    data.get_mut("CraftRecipe").unwrap().layout.rows[0]["ProductCount"] = json!(4);
    let (_, changes) = project(&data, Some(&db), "101", "fr-FR");
    // The recipe and both ends of the item relation change.
    assert_eq!(changes.modified, 3);
    assert_eq!(changes.unchanged, 1);
}

#[test]
fn item_relation_projection_names_sources_and_separates_consumed_materials_and_chest_contents() {
    let mut chest = item(3);
    chest["$variant"] = json!({"UseEffect":"EItemUseEffect::RewardBox","UseEffectValues":["box"]});
    let mut currency = item(5);
    currency["$variant"] = json!({"CurrencyCategory":"ECurrencyCategory::Gold"});
    let mut data = batch(vec![item(1), item(2), chest, item(4), currency]);
    add(
        &mut data,
        "NpcData",
        vec![json!({"ID":{"Value":99},"Name":"internal_monster","Desc":{"Key":"item_name"}})],
    );
    add(
        &mut data,
        "NpcLoot",
        vec![json!({"NpcId":{"Value":99},"LootList":[{"Value":1}]})],
    );
    add(
        &mut data,
        "CraftRecipe",
        vec![
            json!({"ID":{"Value":30},"Name":"recipe_30","Product":"item_1","ProductCount":2,"ComponentList":[{"Item":"item_2","ReplacementItem":"replacement_group","Count":4}]}),
        ],
    );
    add(
        &mut data,
        "CraftComponentReplacement",
        vec![
            json!({"ID":{"Value":1},"Name":"replacement_group","ReplacementItemList":[{"Item":"item_4","Count":4}]}),
        ],
    );
    add(
        &mut data,
        "GameShopGoods",
        vec![
            json!({"ItemName":"item_1","ItemPriceList":[{"ItemCurrencyType":"ECurrencyCategory::Gold","ItemPrice":150}]}),
        ],
    );
    add(
        &mut data,
        "BoxReward",
        vec![
            json!({"Group":"box","Items":[{"Item":"item_1","Count":2}],"SelectItems":[],"PackageItems":[]}),
        ],
    );
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert_eq!(
        db.query_row(
            "SELECT name_en FROM item_relations WHERE item_id=1 AND source_name='NpcLoot'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "Name"
    );
    for id in [2, 4] {
        assert_eq!(db.query_row("SELECT quantity_min FROM item_relations WHERE item_id=?1 AND direction='use' AND source_name='CraftRecipe'",[id],|r|r.get::<_,i64>(0)).unwrap(),4);
        assert_eq!(
            db.query_row(
                "SELECT COUNT(*) FROM item_relations WHERE item_id=?1 AND direction='source'",
                [id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
    }
    assert_eq!(
        db.query_row(
            "SELECT quantity_min FROM item_relations WHERE item_id=5 AND direction='use'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        150
    );
    assert_eq!(
        db.query_row(
            "SELECT related_item_id FROM item_relations WHERE item_id=3 AND direction='use'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(db.query_row("SELECT related_item_id FROM item_relations WHERE item_id=1 AND source_name='BoxReward'",[],|r|r.get::<_,i64>(0)).unwrap(),3);
    assert_eq!(db.query_row("SELECT probability_percent FROM item_relations WHERE item_id=1 AND source_name='NpcLoot'",[],|r|r.get::<_,Option<f64>>(0)).unwrap(),None);
}

#[test]
fn dungeon_pool_weights_and_allocations_use_separate_domains_and_include_unknown_items() {
    let mut data = batch(vec![item(1), item(2)]);
    add(
        &mut data,
        "Dungeon",
        vec![
            json!({"ID":{"Value":600082},"Name":"Ludra","DungeonType":"EDungeonType::Raid","DungeonDifficulty":"EDungeonDifficulty::Normal","RewardOdEnergyCube":"cube"}),
            json!({"ID":{"Value":11},"Name":"Field","DungeonType":"EDungeonType::FieldEvent","RewardOdEnergyCube":"cube"}),
        ],
    );
    add(
        &mut data,
        "RewardOdEnergyCube",
        vec![json!({"Name":"cube","RewardCommon":"boss","RewardMiddleBossList":[]})],
    );
    add(
        &mut data,
        "RewardPackage",
        vec![json!({"Name":"boss","Race":"ERace::All","PackageItems":[
            {"Item":"pool","Prob":5000000,"Count":1},
            {"Item":"pool","Prob":100000000,"Count":1},
            {"Item":"pool","Prob":300000000,"Count":1},
            {"Item":"pool","Prob":450000000,"Count":1},
            {"Item":"pool","Prob":100000000,"Count":1}
        ]})],
    );
    add(
        &mut data,
        "RewardPackageItem",
        vec![json!({"PackageName":"pool","Items":[
            {"ItemName":"item_1","ItemProb":1,"ItemCountMin":1,"ItemCountMax":1},
            {"ItemName":"item_1","ItemProb":1,"ItemCountMin":2,"ItemCountMax":2},
            {"ItemName":"item_2","ItemProb":2,"ItemCountMin":1,"ItemCountMax":1},
            {"ItemName":"unresolved_item","ItemProb":4,"ItemCountMin":1,"ItemCountMax":1}
        ]})],
    );
    add(
        &mut data,
        "RewardItem",
        vec![json!({"Name":"boss","Race":"ERace::All","Items":[
            {"ItemName":"item_2","ItemProb":600000000,"ItemCountMin":1,"ItemCountMax":1},
            {"ItemName":"item_1","ItemProb":100000000,"ItemCountMin":4,"ItemCountMax":9}
        ]})],
    );
    let (db, _) = project(&data, None, "100", "fr-FR");
    assert_eq!(
        db.query_row("SELECT total_weight FROM reward_pools", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        8
    );
    assert_eq!(
        db.query_row(
            "SELECT SUM(probability_percent) FROM reward_pool_entries WHERE item_id=1",
            [],
            |r| r.get::<_, f64>(0)
        )
        .unwrap(),
        25.0
    );
    let allocations:Vec<(i64,f64)>=db.prepare("SELECT guaranteed_draws,extra_draw_percent FROM dungeon_reward_rolls WHERE pool_id IS NOT NULL ORDER BY source_path").unwrap().query_map([],|r|Ok((r.get(0)?,r.get(1)?))).unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(
        allocations,
        vec![(0, 5.0), (1, 0.0), (3, 0.0), (4, 50.0), (1, 0.0)]
    );
    assert_eq!(
        db.query_row(
            "SELECT guaranteed_draws FROM dungeon_reward_rolls WHERE item_id=2",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        6
    );
    assert_eq!(
        db.query_row(
            "SELECT COUNT(*) FROM dungeon_reward_rolls WHERE dungeon_id=11",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        db.query_row(
            "SELECT COUNT(*) FROM reward_pool_entries WHERE item_id IS NULL",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    // A reachable pool's unresolved item remains in its full weight denominator.
    assert_eq!(
        db.query_row(
            "SELECT state FROM catalogue_coverage WHERE subject='items.dungeon_rewards'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "partial"
    );
}

#[test]
fn dungeon_allocation_pool_weights_and_context_changes_belong_to_their_loot_tables() {
    let mut data = batch(vec![item(1), item(2)]);
    add(
        &mut data,
        "Dungeon",
        vec![
            json!({"ID":{"Value":1},"Name":"dungeon","DungeonType":"EDungeonType::Party","RewardOdEnergyCube":"cube"}),
        ],
    );
    add(
        &mut data,
        "RewardOdEnergyCube",
        vec![json!({"Name":"cube","RewardCommon":"boss"})],
    );
    add(
        &mut data,
        "RewardPackage",
        vec![
            json!({"Name":"boss","Race":"ERace::All","PackageItems":[{"Item":"pool","Prob":100000000,"Count":1}]}),
        ],
    );
    add(
        &mut data,
        "RewardPackageItem",
        vec![
            json!({"PackageName":"pool","Items":[{"ItemName":"item_1","ItemProb":1},{"ItemName":"item_2","ItemProb":1}]}),
        ],
    );
    let (first, _) = project(&data, None, "100", "fr-FR");
    data.get_mut("RewardPackage").unwrap().layout.rows[0]["PackageItems"][0]["Prob"] =
        json!(450000000);
    let (second, changes) = project(&data, Some(&first), "101", "fr-FR");
    assert_eq!(changes.modified, 1);
    assert_eq!(
        changed_loot(&second),
        vec![("RewardPackage".into(), "boss".into())]
    );
    assert_eq!(changed_items(&second), 0);
    // A dungeon context change belongs to its allocation definition.
    data.get_mut("Dungeon").unwrap().layout.rows[0]["DungeonDifficulty"] =
        json!("EDungeonDifficulty::Hard");
    let (third, changes) = project(&data, Some(&second), "102", "fr-FR");
    assert_eq!(changes.modified, 1);
    assert_eq!(
        last_changed_loot(&third, "102"),
        vec![("Dungeon".into(), "1".into())]
    );
    assert_eq!(changed_items(&third), 0);
    // Changing an alternative's weight revises the pool, never its items.
    data.get_mut("RewardPackageItem").unwrap().layout.rows[0]["Items"][0]["ItemProb"] = json!(3);
    let (fourth, changes) = project(&data, Some(&third), "103", "fr-FR");
    assert_eq!(changes.modified, 1);
    assert_eq!(
        last_changed_loot(&fourth, "103"),
        vec![("RewardPackageItem".into(), "pool".into())]
    );
    assert_eq!(changed_items(&fourth), 0);
    let linked: (String,String) = fourth.query_row("SELECT t.source_key,v.steam_build FROM reward_pools p JOIN loot_tables t ON t.id=p.loot_table_id JOIN client_version v ON v.id=t.client_version_id", [],|r| Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(linked, ("pool".into(), "103".into()));
}

#[test]
fn empty_families_have_a_baseline_and_new_entries_are_additions() {
    let (empty, summary) = project(&batch(vec![]), None, "100", "fr-FR");
    assert_eq!(summary.added, 0);
    assert_eq!(
        empty
            .query_row("SELECT COUNT(*) FROM catalogue_comparisons", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        4
    );
    let (populated, summary) = project(&batch(vec![item(1)]), Some(&empty), "200", "fr-FR");
    assert_eq!((summary.added, summary.comparison_impossible), (1, 0));
    let change: (String, String) = populated.query_row("SELECT e.change_type,v.steam_build FROM catalogue_entries e JOIN client_version v ON v.id=e.change_reference_build_id", [], |r| Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(change, ("added".into(), "100".into()));
}

#[test]
fn retained_sources_preserve_the_last_reliable_comparison_and_report_the_gap() {
    let data = batch(vec![item(1), item(2)]);
    let (first, _) = project(&data, None, "100", "fr-FR");
    let retained = BTreeSet::from(["ItemSet".into()]);
    let (partial, summary) = project_with(&data, Some(&first), "200", "fr-FR", &retained, None);
    assert_eq!(
        (
            summary.modified,
            summary.unchanged,
            summary.comparison_impossible
        ),
        (0, 0, 2)
    );
    let observed: String = partial.query_row("SELECT v.steam_build FROM catalogue_comparisons c JOIN client_version v ON v.id=c.observed_build_id WHERE family='items'", [], |r| r.get(0)).unwrap();
    assert_eq!(observed, "100");
    let mut data = data;
    data.get_mut("Item").unwrap().layout.rows[0]["PermitLevelMin"] = json!(20);
    let (complete, summary) = project(&data, Some(&partial), "300", "fr-FR");
    assert_eq!((summary.modified, summary.unchanged), (1, 1));
    let comparison: (String,String,bool) = complete.query_row("SELECT o.steam_build,r.steam_build,c.interval_incomplete FROM catalogue_comparisons c JOIN client_version o ON o.id=c.observed_build_id JOIN client_version r ON r.id=c.reference_build_id WHERE c.family='items'", [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(comparison, ("300".into(), "100".into(), true));
    let reference: String = complete.query_row("SELECT r.steam_build FROM catalogue_entries e JOIN client_version r ON r.id=e.change_reference_build_id WHERE source_id=1", [], |r| r.get(0)).unwrap();
    assert_eq!(reference, "100");
    let (next, _) = project(&data, Some(&complete), "400", "fr-FR");
    assert!(
        !next
            .query_row(
                "SELECT interval_incomplete FROM catalogue_comparisons WHERE family='items'",
                [],
                |r| r.get::<_, bool>(0)
            )
            .unwrap()
    );
}

#[test]
fn an_initial_partial_read_is_not_a_reliable_baseline() {
    let data = batch(vec![item(1)]);
    let retained = BTreeSet::from(["ItemSet".into()]);
    let (partial, summary) = project_with(&data, None, "100", "fr-FR", &retained, None);
    assert_eq!((summary.initial, summary.comparison_impossible), (0, 1));
    let (baseline, summary) = project(&data, Some(&partial), "200", "fr-FR");
    assert_eq!(
        (
            summary.added,
            summary.modified,
            summary.comparison_impossible
        ),
        (0, 0, 1)
    );
    let (_, summary) = project(&data, Some(&baseline), "300", "fr-FR");
    assert_eq!((summary.unchanged, summary.comparison_impossible), (1, 0));
}

#[test]
fn translations_are_compared_only_with_the_same_locale() {
    let data = batch(vec![item(1)]);
    let (first, _) = project(&data, None, "100", "fr-FR");
    let localized = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::from([("item_name".into(), "Nouveau nom".into())]),
    };
    let (changed, summary) = project_with(
        &data,
        Some(&first),
        "200",
        "fr-FR",
        &BTreeSet::new(),
        Some(&localized),
    );
    assert_eq!(summary.modified, 1);
    let (switched, summary) = project(&data, Some(&changed), "300", "es-ES");
    assert_eq!((summary.modified, summary.unchanged), (0, 1));
    let (_, summary) = project_with(
        &data,
        Some(&switched),
        "400",
        "es-ES",
        &BTreeSet::new(),
        Some(&localized),
    );
    assert_eq!(summary.modified, 1);
}

#[test]
fn abnormal_effects_and_levels_modify_the_abnormal_without_unrelated_entries() {
    let mut data = batch(vec![item(1)]);
    data.get_mut("SkillAbnormal").unwrap().layout.rows =
        vec![json!({"ID":{"Value":10}}), json!({"ID":{"Value":20}})];
    data.get_mut("SkillAbnormalEffect").unwrap().layout.rows = vec![
        json!({"ID":{"Value":11},"AbnormalId":{"Value":10},"AbnormalEffectLvGroupId":"levels","EffectValueList":["100"]}),
    ];
    data.get_mut("SkillAbnormalEffectLv").unwrap().layout.rows = vec![
        json!({"AbnormalEffectLevelId":{"Value":12},"AbnormalEffectLevelGroupId":"levels","AbnormalEffectLevel":2,"Values":["200"]}),
    ];
    let (first, _) = project(&data, None, "100", "fr-FR");
    data.get_mut("SkillAbnormalEffectLv").unwrap().layout.rows[0]["UnknownRule"] = json!(42);
    let (changed, summary) = project(&data, Some(&first), "200", "fr-FR");
    assert_eq!((summary.modified, summary.unchanged), (1, 2));
    let owner: (String, i64) = changed
        .query_row(
            "SELECT family,source_id FROM catalogue_entries WHERE change_type='modified'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(owner, ("abnormals".into(), 10));
}

#[test]
fn incompatible_profiles_report_partial_coverage_even_for_empty_families() {
    let data = batch(vec![]);
    let (first, _) = project(&data, None, "100", "fr-FR");
    let mut changed = data;
    changed.get_mut("Item").unwrap().mapping_sha256 = "c".repeat(64);
    let (baseline, summary) = project(&changed, Some(&first), "200", "fr-FR");
    assert_eq!(
        (summary.added, summary.deleted, summary.modified),
        (0, 0, 0)
    );
    let state: String = baseline
        .query_row(
            "SELECT state FROM catalogue_coverage WHERE subject='build_comparison'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(state, "partial");
    let (comparable, _) = project(&changed, Some(&baseline), "300", "fr-FR");
    let state: String = comparable
        .query_row(
            "SELECT comparison_state FROM catalogue_comparisons WHERE family='items'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(state, "comparable");
}

#[test]
fn source_row_reordering_and_same_build_reexports_do_not_create_changes() {
    let data = batch(vec![item(1), item(2)]);
    let (first, _) = project(&data, None, "100", "fr-FR");
    let mut reordered = data;
    reordered.get_mut("Item").unwrap().layout.rows.reverse();
    let (second, summary) = project(&reordered, Some(&first), "100", "fr-FR");
    assert_eq!(
        (summary.unchanged, summary.modified, summary.added),
        (2, 0, 0)
    );
    let (_, summary) = project(&reordered, Some(&second), "500", "fr-FR");
    assert_eq!((summary.unchanged, summary.modified), (2, 0));
}

#[test]
fn tombstones_survive_partial_reads_without_repeating_the_deletion() {
    let data = batch(vec![item(1)]);
    let (first, _) = project(&data, None, "100", "fr-FR");
    let empty = batch(vec![]);
    let (deleted, summary) = project(&empty, Some(&first), "200", "fr-FR");
    assert_eq!(summary.deleted, 1);
    let retained = BTreeSet::from(["ItemSet".into()]);
    let (partial, summary) = project_with(&empty, Some(&deleted), "300", "fr-FR", &retained, None);
    assert_eq!((summary.deleted, summary.comparison_impossible), (0, 1));
    let (complete, summary) = project(&empty, Some(&partial), "400", "fr-FR");
    assert_eq!((summary.deleted, summary.modified), (0, 0));
    let trace:(bool,String,String,String) = complete.query_row("SELECT e.present,e.change_type,c.steam_build,r.steam_build FROM catalogue_entries e JOIN client_version c ON c.id=e.change_build_id JOIN client_version r ON r.id=e.change_reference_build_id", [], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap();
    assert_eq!(trace, (false, "deleted".into(), "200".into(), "100".into()));
}

#[test]
fn loot_source_translations_do_not_revise_items() {
    let mut data = batch(vec![item(1), item(2)]);
    add(
        &mut data,
        "NpcData",
        vec![json!({"ID":{"Value":99},"Name":"monster","Desc":{"Key":"npc_name"}})],
    );
    add(
        &mut data,
        "NpcLoot",
        vec![json!({"NpcId":{"Value":99},"LootList":[{"Value":1}]})],
    );
    let mut localized = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::from([
            ("item_name".into(), "Name".into()),
            ("npc_name".into(), "Créature".into()),
        ]),
    };
    let (first, _) = project_with(
        &data,
        None,
        "100",
        "fr-FR",
        &BTreeSet::new(),
        Some(&localized),
    );
    localized
        .entries
        .insert("npc_name".into(), "Autre nom".into());
    let (second, summary) = project_with(
        &data,
        Some(&first),
        "200",
        "fr-FR",
        &BTreeSet::new(),
        Some(&localized),
    );
    assert_eq!((summary.modified, summary.unchanged), (0, 3));
    let name: String = second
        .query_row(
            "SELECT name_localized FROM item_relations WHERE source_name='NpcLoot'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(name, "Autre nom");
    assert_eq!(changed_items(&second), 0);
}

fn changed_items(db: &Connection) -> i64 {
    db.query_row(
        "SELECT count(*) FROM catalogue_entries WHERE family='items' AND change_type IS NOT NULL",
        [],
        |r| r.get(0),
    )
    .unwrap()
}
fn changed_loot(db: &Connection) -> Vec<(String, String)> {
    db.prepare("SELECT t.source_name,t.source_key FROM loot_tables t JOIN catalogue_entries e ON e.family='loot_tables' AND e.source_id=t.id WHERE e.change_type IS NOT NULL ORDER BY t.source_name,t.source_key").unwrap().query_map([],|r|Ok((r.get(0)?,r.get(1)?))).unwrap().collect::<rusqlite::Result<_>>().unwrap()
}
fn last_changed_loot(db: &Connection, build: &str) -> Vec<(String, String)> {
    db.prepare("SELECT t.source_name,t.source_key FROM loot_tables t JOIN catalogue_entries e ON e.family='loot_tables' AND e.source_id=t.id JOIN client_version v ON v.id=e.change_build_id WHERE v.steam_build=?1 ORDER BY t.source_name,t.source_key").unwrap().query_map([build],|r|Ok((r.get(0)?,r.get(1)?))).unwrap().collect::<rusqlite::Result<_>>().unwrap()
}

#[test]
fn loot_identity_survives_reordering_deletion_and_reappearance_without_revising_items() {
    let mut data = batch(vec![item(1), item(2)]);
    let a = json!({"PackageName":"pool_a","Items":[{"ItemName":"item_1","ItemProb":1}]});
    let b = json!({"PackageName":"pool_b","Items":[{"ItemName":"item_2","ItemProb":1}]});
    add(&mut data, "RewardPackageItem", vec![a.clone(), b.clone()]);
    let (first, summary) = project(&data, None, "100", "fr-FR");
    assert_eq!((summary.initial, summary.added), (4, 0));
    let id: i64 = first
        .query_row(
            "SELECT id FROM loot_tables WHERE source_key='pool_a'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    data.get_mut("RewardPackageItem")
        .unwrap()
        .layout
        .rows
        .reverse();
    let (second, summary) = project(&data, Some(&first), "200", "fr-FR");
    assert_eq!((summary.modified, summary.unchanged), (0, 4));
    data.get_mut("RewardPackageItem").unwrap().layout.rows = vec![b];
    let (third, summary) = project(&data, Some(&second), "300", "fr-FR");
    assert_eq!((summary.deleted, summary.modified), (1, 0));
    assert_eq!(changed_items(&third), 0);
    assert_eq!(
        last_changed_loot(&third, "300"),
        vec![("RewardPackageItem".into(), "pool_a".into())]
    );
    data.get_mut("RewardPackageItem")
        .unwrap()
        .layout
        .rows
        .push(a);
    let (fourth, summary) = project(&data, Some(&third), "400", "fr-FR");
    assert_eq!((summary.added, summary.modified), (1, 0));
    let trace:(i64,String)=fourth.query_row("SELECT t.id,e.change_type FROM loot_tables t JOIN catalogue_entries e ON e.family='loot_tables' AND e.source_id=t.id WHERE t.source_key='pool_a'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(trace, (id, "added".into()));
    assert_eq!(changed_items(&fourth), 0);
}

#[test]
fn loot_variants_are_grouped_and_empty_pools_keep_their_identity() {
    let mut data = batch(vec![item(1)]);
    add(
        &mut data,
        "BoxReward",
        vec![
            json!({"Group":"box","Race":"ERace::Light","SelectItems":[{"Item":"item_1","Count":1}]}),
            json!({"Group":"box","Race":"ERace::Dark","SelectItems":[{"Item":"item_1","Count":2}]}),
        ],
    );
    add(
        &mut data,
        "RewardPackageItem",
        vec![json!({"PackageName":"empty","Items":[]})],
    );
    let (first, summary) = project(&data, None, "100", "fr-FR");
    assert_eq!(summary.initial, 3);
    data.get_mut("BoxReward").unwrap().layout.rows.reverse();
    let (second, summary) = project(&data, Some(&first), "200", "fr-FR");
    assert_eq!((summary.modified, summary.unchanged), (0, 3));
    data.get_mut("RewardPackageItem").unwrap().layout.rows[0]["Items"] =
        json!([{"ItemName":"item_1","ItemProb":50}]);
    let (third, summary) = project(&data, Some(&second), "300", "fr-FR");
    assert_eq!((summary.modified, summary.added), (1, 0));
    assert_eq!(
        changed_loot(&third),
        vec![("RewardPackageItem".into(), "empty".into())]
    );
    assert_eq!(changed_items(&third), 0);
    data.get_mut("BoxReward").unwrap().layout.rows[0]["SelectItems"][0]["Count"] = json!(3);
    let (fourth, summary) = project(&data, Some(&third), "400", "fr-FR");
    assert_eq!(summary.modified, 1);
    assert_eq!(
        last_changed_loot(&fourth, "400"),
        vec![("BoxReward".into(), "box".into())]
    );
    assert_eq!(changed_items(&fourth), 0);
}
