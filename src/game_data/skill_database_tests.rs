use super::*;
use crate::game_data::{schema::LayoutProbe, skill_descriptions, usmap::TableProbe};
use serde_json::json;

fn fixture() -> (skills::SkillProbe, Localization) {
    let mut probe = skills::SkillProbe {
        compatibility_verified: false,
        tables: BTreeMap::new(),
        relations: vec![],
    };
    for (name, structure) in skills::TABLES {
        probe.tables.insert(
            (*name).into(),
            TableProbe {
                mapping_sha256: String::new(),
                mapping_version: 4,
                mapping_ambiguous_structures: BTreeSet::new(),
                row_struct: (*structure).into(),
                payload_sha256: String::new(),
                payload_bytes: 0,
                layout: LayoutProbe {
                    compatibility_verified: false,
                    table_version: 13,
                    column_hashes: vec![],
                    rows: vec![],
                },
            },
        );
    }
    let skill = json!({"ID":{"Value":1},"SkillString_Key":"text","SkillIcon":"Skill/Icon_1","SkillGrade":"EItemGrade::Legend","SkillType":"ESkillType::Active","SkillDamageType":"ESkillDamageType::Magic","bIsStigmaSkill":true,"NeedCostMp":10,"CastingTime":500,"NeedCoolTime":2000.0,"NeedSkillUseRange":400.0,"NeedWeaponUseRange":false,"SkillLvMax":3,"SkillLvGroupId":"levels","NeedMainWeaponTypes":["EWeaponType::Orb"],"CanUseWhenPeaceState":false,"UnableWhenCanNotMoveControl":true,"SkillEffectTimeDataList":[{"SkillEffectGroupId":{"Value":900},"SkillEffectFilterId":{"Value":30}}]});
    let mut second = skill.clone();
    second["ID"] = json!({"Value":2});
    second["SkillString_Key"] = json!("None");
    second["SkillEffectTimeDataList"] =
        json!([{"SkillEffectGroupId":{"Value":901},"SkillEffectFilterId":{"Value":0}}]);
    second["SkillLvGroupId"] = json!("None");
    second["bIsStigmaSkill"] = json!(false);
    second["SkillGrade"] = json!("EItemGrade::Common");
    probe.tables.get_mut("Skill").unwrap().layout.rows = vec![skill, second];
    probe.tables.get_mut("SkillString").unwrap().layout.rows = vec![
        json!({"Name":"text","SkillName":{"Key":"name"},"SkillDescEffect":{"Key":"description"}}),
    ];
    probe.tables.get_mut("SkillLv").unwrap().layout.rows = vec![
        json!({"SkillLvGroupId":"levels","SkillLv":2,"NeedCostMp":0,"CastingTime":0,"NeedCoolTime":0.0}),
    ];
    probe
        .tables
        .get_mut("SkillAcquireData")
        .unwrap()
        .layout
        .rows = vec![
        json!({"SkillId":{"Value":1},"SkillLevel":2,"CostSkillPoint":4,"CostStigmaPoint":0,"CostStigmaSuperiorPoint":0}),
    ];
    probe
        .tables
        .get_mut("SpecializedSkill")
        .unwrap()
        .layout
        .rows =
        vec![json!({"ID":{"Value":2},"ParentSkillId":{"Value":1},"ChildSkillId":{"Value":2}})];
    probe
        .tables
        .get_mut("SpecializedSkillParts")
        .unwrap()
        .layout
        .rows =
        vec![json!({"ID":{"Value":2},"ParentSkillLv":2,"SpecializedSkillPartsDesc":"spec"})];
    probe.tables.get_mut("SkillEffect").unwrap().layout.rows = vec![
        json!({"ID":{"Value":10},"SkillEffectGroupId":{"Value":900},"EffectType":"ESkillEffectType::Damage","SkillEffectLvGroupId":"effect_levels","EffectValueList":["1500","TRUE","None"]}),
        json!({"ID":{"Value":11},"SkillEffectGroupId":{"Value":900},"EffectType":"ESkillEffectType::GroggyGuardDecreaseEnergy","EffectValueList":["2"]}),
        json!({"ID":{"Value":12},"SkillEffectGroupId":{"Value":900},"EffectType":"ESkillEffectType::Abnormal","EffectValueList":["1000","1000","50"]}),
        json!({"ID":{"Value":998},"SkillEffectGroupId":{"Value":901},"EffectType":"ESkillEffectType::Damage","EffectValueList":["998"]}),
        json!({"ID":{"Value":999},"SkillEffectGroupId":{"Value":999},"EffectType":"ESkillEffectType::Damage","EffectValueList":["999"]}),
    ];
    probe.tables.get_mut("SkillEffectLv").unwrap().layout.rows = vec![
        json!({"ID":{"Value":100},"SkillEffectLvGroupId":"effect_levels","SkillEffectLv":2,"EffectValueList":["0","TRUE","None"]}),
        json!({"ID":{"Value":999},"SkillEffectLvGroupId":"unused","SkillEffectLv":2,"EffectValueList":["999"]}),
    ];
    probe.tables.get_mut("SkillAbnormal").unwrap().layout.rows =
        vec![json!({"ID":{"Value":50},"AbnormalEffectLevelMax":3})];
    probe
        .tables
        .get_mut("SkillAbnormalEffect")
        .unwrap()
        .layout
        .rows = vec![
        json!({"ID":{"Value":20},"AbnormalId":{"Value":50},"AbnormalEffectType":"EAbnormalEffectType::StatChange","AbnormalEffectLvGroupId":"abnormal_levels","Values":["Damage","5"]}),
        json!({"ID":{"Value":999},"AbnormalId":{"Value":999},"AbnormalEffectType":"EAbnormalEffectType::StatChange","Values":["999"]}),
    ];
    probe
        .tables
        .get_mut("SkillAbnormalEffectLv")
        .unwrap()
        .layout
        .rows = vec![
        json!({"AbnormalEffectLevelId":{"Value":200},"AbnormalEffectLevelGroupId":"abnormal_levels","AbnormalEffectLevel":2,"Values":["Damage","10"]}),
    ];
    probe
        .tables
        .get_mut("SkillEffectFilter")
        .unwrap()
        .layout
        .rows = vec![json!({"ID":{"Value":30},"TargetCountMax":4})];
    probe.tables.get_mut("SkillList").unwrap().layout.rows =
        vec![json!({"ClassType":"ECharacterClass::Sorcerer","SkillIdList":[1]})];
    let localization = Localization {
        namespace: "AION2".into(),
        entries: BTreeMap::from([
            ("name".into(), "Test".into()),
            (
                "description".into(),
                "{se:10:effect_value01:divide100}%".into(),
            ),
            (
                "SkillString_spec_specialized_skill_desc".into(),
                "Specialization".into(),
            ),
        ]),
    };
    (probe, localization)
}
fn database(probe: &skills::SkillProbe, l10n: &Localization) -> Connection {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!("items.sql")).unwrap();
    db.execute_batch("PRAGMA foreign_keys=OFF; INSERT INTO client_version VALUES(1,'100');")
        .unwrap();
    db.execute_batch(include_str!("skills.sql")).unwrap();
    projection::write(&db, &probe.tables, l10n, l10n, 1).unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON").unwrap();
    integrity(&db).unwrap();
    db
}
#[test]
fn normalized_schema_has_bounded_dictionaries_no_views_and_only_product_fields() {
    let (probe, l10n) = fixture();
    let db = database(&probe, &l10n);
    assert_eq!(skill_tables(&db).unwrap().len(), 16);
    assert!(skill_views(&db).unwrap().is_empty());
    let values: (String, String, Option<String>, String) = db
        .query_row(
            "SELECT s.name_en,i.reference,r.code,t.code FROM skills s JOIN catalogue_icons i ON i.id=s.icon_id LEFT JOIN skill_rarities r ON r.id=s.rarity_id JOIN skill_types t ON t.id=s.type_id WHERE s.id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        values,
        (
            "Test".into(),
            "Skill/Icon_1".into(),
            Some("Legend".into()),
            "Stigma".into()
        )
    );
    assert_eq!(
        db.query_row(
            "SELECT w.code FROM skill_weapons s JOIN skill_weapon_types w ON w.id=s.weapon_id WHERE s.skill_id=1",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "Orb"
    );
    assert_eq!(
        db.query_row(
            "SELECT description_en FROM skill_specializations",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "Specialization"
    );
    assert!(
        db.execute("UPDATE skills SET uses_weapon_range=2", [])
            .is_err()
    );
    for removed in [
        "skill_enum_skill_type",
        "skill_abnormals",
        "skill_acquisitions",
        "skill_charges",
        "skill_localization_language",
        "skill_level_groups",
    ] {
        assert!(
            !db.prepare("SELECT 1 FROM sqlite_schema WHERE name=?1")
                .unwrap()
                .exists([removed])
                .unwrap()
        );
    }
}
#[test]
fn complete_projection_keeps_levels_abnormals_and_effects_without_description_references() {
    let (probe, l10n) = fixture();
    let db = database(&probe, &l10n);
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM skill_effects WHERE id IN(998,999)",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        4
    );
    assert_eq!(db.query_row("SELECT value FROM skill_effect_values WHERE kind_id=(SELECT id FROM skill_effect_kinds WHERE code='abe_level') AND effect_id=200 AND position=1",[],|r|r.get::<_,String>(0)).unwrap(),"10");
    assert_eq!(
        db.query_row(
            "SELECT value FROM skill_effect_values WHERE kind_id=(SELECT id FROM skill_effect_kinds WHERE code='se') AND effect_id=10 AND position=1",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "TRUE"
    );
    assert_eq!(
        skill_descriptions::description(&db, 1, 1, false)
            .unwrap()
            .text
            .as_deref(),
        Some("15%")
    );
    assert_eq!(
        skill_descriptions::description(&db, 1, 2, false)
            .unwrap()
            .text
            .as_deref(),
        Some("0%")
    );
    let level = skill_descriptions::level(&db, 1, 2).unwrap();
    assert_eq!(
        (
            level.mp_cost,
            level.casting_time,
            level.cooldown,
            level.skill_points
        ),
        (Some(0), 0, 0.0, Some(4))
    );
    assert_eq!(
        skill_descriptions::stagger(&db, 1, 2).unwrap()[0].damage,
        Some(2.0)
    );
    assert_eq!(
        db.query_row("SELECT stagger_damage FROM skills WHERE id=1", [], |r| r
            .get::<_, f64>(0))
            .unwrap(),
        2.0
    );
}
#[test]
fn fingerprints_follow_effect_and_level_dependencies_and_ignore_locale_switches() {
    let (probe, l10n) = fixture();
    let db = database(&probe, &l10n);
    let before = versions::snapshot(&db, true).unwrap();
    db.execute("UPDATE skill_effect_values SET value='20' WHERE kind_id=(SELECT id FROM skill_effect_kinds WHERE code='abe_level') AND effect_id=200 AND position=1",[]).unwrap();
    let after = versions::snapshot(&db, true).unwrap();
    assert_ne!(before[&1].0, after[&1].0);
    assert_eq!(before[&2].0, after[&2].0);
    let without_locale = versions::snapshot(&db, false).unwrap();
    db.execute(
        "UPDATE skills SET description_localized='Different locale'",
        [],
    )
    .unwrap();
    assert_eq!(without_locale, versions::snapshot(&db, false).unwrap());
    assert_ne!(after[&1].0, versions::snapshot(&db, true).unwrap()[&1].0);
}

#[test]
fn class_associations_follow_explicit_catalog_and_specialization_edges() {
    let (probe, l10n) = fixture();
    let db = database(&probe, &l10n);
    let rows:Vec<(i64,String)>=db.prepare("SELECT p.skill_id,c.code FROM skill_class_permissions p JOIN skill_classes c ON c.id=p.class_id ORDER BY p.skill_id").unwrap().query_map([],|r|Ok((r.get(0)?,r.get(1)?))).unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(rows, vec![(1, "Sorcerer".into()), (2, "Sorcerer".into())]);
    for table in [
        "skill_types",
        "skill_rarities",
        "skill_damage_types",
        "skill_effect_kinds",
        "skill_weapon_types",
        "skill_classes",
    ] {
        assert!(
            db.execute(&format!("INSERT INTO {table} VALUES(65536,'invalid')"), [])
                .is_err()
        );
    }
    assert!(
        !db.prepare("PRAGMA table_info(skills)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .any(|v| v.unwrap() == "casting_time")
    );
    assert_eq!(
        db.query_row(
            "SELECT duration FROM skill_cast_times WHERE skill_id=1 AND level=1 AND kind='casting'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        500
    );
}

#[test]
fn dictionary_id_reassignment_does_not_mark_game_data_as_modified() {
    let (probe, l10n) = fixture();
    let db = database(&probe, &l10n);
    let before = versions::snapshot(&db, true).unwrap();
    db.execute_batch("PRAGMA foreign_keys=OFF; UPDATE skill_types SET id=id+100; UPDATE skills SET type_id=type_id+100;
        UPDATE skill_effect_kinds SET id=id+100; UPDATE skill_effects SET kind_id=kind_id+100; UPDATE skill_effect_values SET kind_id=kind_id+100; UPDATE skill_effect_links SET kind_id=kind_id+100;
        PRAGMA foreign_keys=ON;").unwrap();
    integrity(&db).unwrap();
    assert_eq!(before, versions::snapshot(&db, true).unwrap());
}

#[test]
fn rarities_follow_source_including_condemnation_without_manual_overrides() {
    let (mut probe, l10n) = fixture();
    let rows = &mut probe.tables.get_mut("Skill").unwrap().layout.rows;
    let mut condemnation = rows[1].clone();
    condemnation["ID"] = json!({"Value":17350000});
    rows[1]["SkillGrade"] = json!("EItemGrade::Rare");
    rows.push(condemnation);
    let db = database(&probe, &l10n);
    let rarities: Vec<(i64, String)> = db
        .prepare("SELECT s.id,r.code FROM skills s JOIN skill_rarities r ON r.id=s.rarity_id ORDER BY s.id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        rarities,
        vec![
            (1, "Legend".into()),
            (2, "Rare".into()),
            (17350000, "Common".into())
        ]
    );
}

#[test]
fn item_and_system_skills_and_item_description_effects_are_preserved() {
    let (mut probe, l10n) = fixture();
    for (id, ty) in [(3, "Item"), (4, "System")] {
        let mut row = probe.tables["Skill"].layout.rows[0].clone();
        row["ID"] = json!({"Value":id});
        row["SkillType"] = json!(format!("ESkillType::{ty}"));
        // Preserve the source category even for internal stigma variants.
        probe.tables.get_mut("Skill").unwrap().layout.rows.push(row);
        probe
            .tables
            .get_mut("SkillAcquireData")
            .unwrap()
            .layout
            .rows
            .push(json!({"SkillId":{"Value":id},"SkillLevel":1}));
        probe
            .tables
            .get_mut("SkillCond")
            .unwrap()
            .layout
            .rows
            .push(json!({"ID":{"Value":id},"CondSkillList":[{"Value":id}]}));
        probe.tables.get_mut("SkillLink").unwrap().layout.rows.push(json!({"ID":{"Value":id},"SkillId":{"Value":id},"LinkConditionList":[{"Condition":"test"}]}));
        probe
            .tables
            .get_mut("SkillList")
            .unwrap()
            .layout
            .rows
            .push(json!({"ClassType":"ECharacterClass::Sorcerer","SkillIdList":[id]}));
    }
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch(include_str!("items.sql")).unwrap();
    db.execute_batch("INSERT INTO client_version VALUES(1,'100');
        INSERT INTO item_families VALUES(1,'test');
        INSERT INTO item_categories VALUES(1,1,'test');
        INSERT INTO rarities VALUES(1,'Common','Common');
        INSERT INTO sell_currencies VALUES(1,'test','test');
        INSERT INTO items(id,client_version_id,category_id,rarity_id,sell_currency_id,item_level,sell_price,description_en)
        VALUES(1,1,1,1,1,0,0,'{se:999:effect_value01:none} {abe:999:value01:none}');
        PRAGMA foreign_keys=OFF;").unwrap();
    db.execute_batch(include_str!("skills.sql")).unwrap();
    projection::write(&db, &probe.tables, &l10n, &l10n, 1).unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON").unwrap();
    integrity(&db).unwrap();
    assert_eq!(
        db.query_row("SELECT count(*) FROM skills", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        4
    );
    assert_eq!(db.query_row("SELECT count(*) FROM skill_effects WHERE id=999 AND kind_id IN (SELECT id FROM skill_effect_kinds WHERE code IN ('se','abe'))", [], |r| r.get::<_, i64>(0)).unwrap(), 2);
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM skill_levels WHERE skill_id IN (3,4)",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        4
    );
}

#[test]
fn system_variants_remain_available_without_player_catalogue_links() {
    let (mut probe, l10n) = fixture();
    probe.tables.get_mut("Skill").unwrap().layout.rows[1]["SkillType"] =
        json!("ESkillType::System");
    let db = database(&probe, &l10n);
    assert!(
        db.prepare("SELECT 1 FROM skills WHERE id=2")
            .unwrap()
            .exists([])
            .unwrap()
    );
    // Catalogue membership must not decide whether source evidence survives.
    probe
        .tables
        .get_mut("SpecializedSkill")
        .unwrap()
        .layout
        .rows
        .clear();
    let db = database(&probe, &l10n);
    assert!(
        db.prepare("SELECT 1 FROM skills WHERE id=2")
            .unwrap()
            .exists([])
            .unwrap()
    );
}
