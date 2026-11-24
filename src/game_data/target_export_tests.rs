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
fn project(
    batch: &BTreeMap<String, TableProbe>,
    old: Option<&Connection>,
    build: &str,
    locale: &str,
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
        ]),
    };
    for row in &batch["Item"].layout.rows {
        let id = number(row, "ID").unwrap();
        tx.execute("INSERT INTO items(id,client_version_id,category_id,rarity_id,sell_currency_id,item_level,sell_price,name_en,name_localized) VALUES(?1,1,1,1,1,1,0,'Name',?2)",params![id,format!("{locale} name")]).unwrap();
        if row["CanSell"].as_bool() == Some(true) {
            tx.execute("INSERT OR IGNORE INTO flags VALUES(1,'CanSell')", [])
                .unwrap();
            tx.execute("INSERT INTO item_flags VALUES(?1,1)", [id])
                .unwrap();
        }
    }
    super::super::skill_database::write(&tx, batch, &l10n, &l10n, 1).unwrap();
    tx.execute_batch(include_str!("catalogue.sql")).unwrap();
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
    projection::write(&tx, batch, &l10n, &l10n, None).unwrap();
    let mut summary = Summary::default();
    changes::record(&tx, old, batch, &BTreeSet::new(), 1, locale, &mut summary).unwrap();
    tx.commit().unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON").unwrap();
    integrity(&db).unwrap();
    (db, summary)
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
    assert_eq!((s.modified, s.deleted, s.comparison_impossible), (0, 0, 1));
    let state:(bool,String)=c.query_row("SELECT present,comparison_state FROM catalogue_entries WHERE family='items' AND source_id=2",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(state, (true, "impossible".into()));
}
#[test]
fn target_preserves_unknown_fields_false_permissions_and_shared_icon_references() {
    let mut r = item(1);
    r["UnknownSource"] = json!({"Removed":["a"],"Entries":[{"Key":"x","Value":"raw"}]});
    let first = batch(vec![r.clone(), item(2)]);
    let (db, _) = project(&first, None, "100", "fr-FR");
    assert_eq!(source_rows(&db, "Item").unwrap()[0], r);
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
