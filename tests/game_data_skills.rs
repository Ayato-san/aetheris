use aetheris::game_data::{
    skills::{self, ReferenceCheck},
    usmap::{Mapping, TableProbe},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    process::{Command, Output},
};

struct Fixture {
    root: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
impl Fixture {
    fn new() -> Self {
        let mut random = [0u8; 16];
        getrandom::fill(&mut random).unwrap();
        let root = std::env::temp_dir().join(format!("aetheris-skills-{}", hex::encode(random)));
        fs::create_dir(&root).unwrap();
        // A synthetic v0 mapping with empty structures exercises the complete
        // profile, including tables that legitimately have no rows.
        let mut payload = (skills::TABLES.len() as u32).to_le_bytes().to_vec();
        for (_, structure) in skills::TABLES {
            payload.push(structure.len() as u8);
            payload.extend(structure.as_bytes());
        }
        payload.extend(0u32.to_le_bytes()); // enums
        payload.extend((skills::TABLES.len() as u32).to_le_bytes());
        for (index, (table, _)) in skills::TABLES.iter().enumerate() {
            payload.extend((index as u32).to_le_bytes());
            payload.extend(u32::MAX.to_le_bytes());
            payload.extend(0u16.to_le_bytes());
            payload.extend(0u16.to_le_bytes());
            let mut data = 13i32.to_le_bytes().to_vec();
            data.extend(1i32.to_le_bytes());
            data.push(0); // empty column hashes
            data.extend(0i32.to_le_bytes());
            fs::write(root.join(format!("{table}.bin")), data).unwrap();
        }
        let mut mapping = 0x30c4u16.to_le_bytes().to_vec();
        mapping.extend([0, 0]); // v0, no compression
        mapping.extend((payload.len() as u32).to_le_bytes());
        mapping.extend((payload.len() as u32).to_le_bytes());
        mapping.extend(payload);
        fs::write(root.join("mapping.usmap"), mapping).unwrap();
        Self { root }
    }
    fn run(&self, output: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_aetheris-game-data"))
            .arg("probe-skills")
            .arg(self.root.join("mapping.usmap"))
            .arg(&self.root)
            .arg(self.root.join(output))
            .output()
            .unwrap()
    }
    fn mapping(&self) -> Mapping {
        Mapping::parse(&fs::read(self.root.join("mapping.usmap")).unwrap()).unwrap()
    }
}

fn tables() -> BTreeMap<String, TableProbe> {
    skills::TABLES
        .iter()
        .map(|(name, structure)| {
            (
                (*name).into(),
                serde_json::from_value(json!({
                    "mapping_sha256":"11".repeat(32),"mapping_version":0,
                    "mapping_ambiguous_structures":[],"row_struct":structure,
                    "payload_sha256":"22".repeat(32),"payload_bytes":13,
                    "compatibility_verified":false,"table_version":13,"column_hashes":[],"rows":[]
                }))
                .unwrap(),
            )
        })
        .collect()
}
fn skill(id: i64) -> Value {
    json!({"ID":{"Value":id},"SkillEffectTimeDataList":[],"SkillLvGroupId":"None",
        "SkillString_Key":"None","AutoUseId":{"Value":0},"ChargeId":{"Value":0},
        "ChannelingId":{"Value":0},"ChainSkillPrevSkillId":{"Value":0},
        "DamageAnalyzerSkillIdOverride":{"Value":0},"CancelExceptionSkillIdList":[],
        "ToggleOnAbnormalId":{"Value":0},"TargetFilterId":{"Value":0},"ProjectileTargetFilter":{"Value":0}})
}
fn effect(id: i64, kind: &str, params: Value) -> Value {
    json!({"ID":{"Value":id},"EffectType":format!("ESkillEffectType::{kind}"),
        "EffectValueList":params,"SkillEffectLvGroupId":"None","SkillEffectGroupId":{"Value":990}})
}
fn relation<'a>(checks: &'a [ReferenceCheck], source: &str, field: &str) -> &'a ReferenceCheck {
    checks
        .iter()
        .find(|r| r.source_table == source && r.source_field == field)
        .unwrap()
}

#[test]
fn reads_all_profiles_including_empty_tables_and_cli_is_reproducible() {
    let fixture = Fixture::new();
    let report = skills::probe(&fixture.mapping(), &fixture.root).unwrap();
    assert_eq!(report.tables.len(), 41);
    assert!(!report.compatibility_verified);
    assert!(
        report
            .tables
            .values()
            .all(|t| t.layout.rows.is_empty() && !t.layout.compatibility_verified)
    );
    assert!(report.relations.iter().all(|r| r.references == 0));
    let output = fixture.run("report.json");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let before = fs::read(fixture.root.join("report.json")).unwrap();
    let output = fixture.run("report.json");
    assert!(output.status.success());
    assert_eq!(before, fs::read(fixture.root.join("report.json")).unwrap());
    assert_eq!(
        report.tables["CCSkillMapping"].row_struct,
        "ClassChangeSkillMappingTableItem"
    );
    assert_eq!(
        report.tables["SkillAbnormalEffectLv"].row_struct,
        "AbnormalEffectLevelTableItem"
    );
}

#[test]
fn failed_or_incomplete_reading_preserves_previous_report_and_source_files() {
    let fixture = Fixture::new();
    assert!(fixture.run("report.json").status.success());
    let before = fs::read(fixture.root.join("report.json")).unwrap();
    for source in ["mapping.usmap", "Skill.bin"] {
        let bytes = fs::read(fixture.root.join(source)).unwrap();
        assert!(!fixture.run(source).status.success());
        assert_eq!(bytes, fs::read(fixture.root.join(source)).unwrap());
    }
    let skill = fs::read(fixture.root.join("Skill.bin")).unwrap();
    fs::write(fixture.root.join("Skill.bin"), &skill[..skill.len() - 1]).unwrap();
    assert!(!fixture.run("report.json").status.success());
    assert_eq!(before, fs::read(fixture.root.join("report.json")).unwrap());
    fs::remove_file(fixture.root.join("Skill.bin")).unwrap();
    assert!(skills::probe(&fixture.mapping(), &fixture.root).is_err());
    assert_eq!(before, fs::read(fixture.root.join("report.json")).unwrap());
}

#[test]
fn new_related_tables_cannot_be_silently_omitted() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("SkillNew.bin"), []).unwrap();
    let result = fixture.run("report.json");
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("unreviewed skill table SkillNew"));
    assert!(!fixture.root.join("report.json").exists());
}

#[test]
fn checks_effect_groups_and_level_groups_without_guessing_from_numeric_ids() {
    let mut tables = tables();
    let mut row = skill(1);
    row["SkillEffectTimeDataList"] = json!([
        {"SkillEffectGroupId":{"Value":990},"SkillEffectFilterId":{"Value":0},"ProjectileId":{"Value":0}},
        {"SkillEffectGroupId":{"Value":0},"SkillEffectFilterId":{"Value":0},"ProjectileId":{"Value":0}}
    ]);
    row["SkillLvGroupId"] = json!("shared_levels");
    tables.get_mut("Skill").unwrap().layout.rows = vec![row];
    let mut second = effect(3, "Damage", json!([]));
    second["SkillEffectLvGroupId"] = json!("missing_levels");
    tables.get_mut("SkillEffect").unwrap().layout.rows =
        vec![effect(2, "Damage", json!([])), second];
    tables.get_mut("SkillLv").unwrap().layout.rows = vec![
        json!({"SkillLvGroupId":"shared_levels","SkillLv":2}),
        json!({"SkillLvGroupId":"shared_levels","SkillLv":3}),
    ];
    let checks = skills::check_relations(&tables).unwrap();
    let groups = relation(
        &checks,
        "Skill",
        "SkillEffectTimeDataList.SkillEffectGroupId.Value",
    );
    assert_eq!(
        (groups.references, groups.resolved, groups.missing),
        (1, 1, 0)
    );
    assert_eq!(relation(&checks, "Skill", "SkillLvGroupId").resolved, 1);
    let levels = relation(&checks, "SkillEffect", "SkillEffectLvGroupId");
    assert_eq!(levels.missing, 1);
    assert_eq!(
        levels.missing_examples[0].source_id,
        Some(json!({"Value":3}))
    );
    assert_eq!(levels.missing_examples[0].target, "missing_levels");
}

#[test]
fn effect_parameters_are_type_dependent_and_remain_explicit_hypotheses() {
    let mut tables = tables();
    tables.get_mut("Skill").unwrap().layout.rows = vec![skill(900)];
    tables.get_mut("SkillAbnormal").unwrap().layout.rows = vec![json!({
        "ID":{"Value":700},"SkillAbnormalString_Key":"None","AbnormalEffectType":"EAbnormalEffectType::None","GroupId":70
    })];
    tables.get_mut("SkillEffect").unwrap().layout.rows = vec![
        effect(
            1,
            "Abnormal_True",
            json!(["10000", "10000", "700", "1", "10000"]),
        ),
        effect(2, "Abnormal_Passive", json!(["700"])),
        effect(3, "Toggle", json!(["700", "1"])),
        effect(4, "CasterSkill", json!(["900"])),
    ];
    let checks = skills::check_relations(&tables).unwrap();
    let parameters: Vec<_> = checks
        .iter()
        .filter(|r| r.inferred_parameter && r.references > 0)
        .collect();
    assert_eq!(parameters.len(), 4);
    assert!(parameters.iter().all(|r| r.resolved == 1 && r.missing == 0));
    tables.get_mut("SkillEffect").unwrap().layout.rows[1]["EffectValueList"] = json!(["invalid"]);
    assert!(skills::check_relations(&tables).is_err());
}

#[test]
fn missing_references_are_counted_without_truncating_or_changing_data() {
    let mut tables = tables();
    let rows: Vec<_> = (1..=25)
        .map(
            |id| json!({"ID":{"Value":id},"SkillChargeDataList":[{"ChargeSkillId":{"Value":999}}]}),
        )
        .collect();
    tables.get_mut("SkillCharge").unwrap().layout.rows = rows.clone();
    let checks = skills::check_relations(&tables).unwrap();
    let missing = relation(
        &checks,
        "SkillCharge",
        "SkillChargeDataList.ChargeSkillId.Value",
    );
    assert_eq!(
        (
            missing.references,
            missing.missing,
            missing.missing_examples.len()
        ),
        (25, 25, 20)
    );
    assert_eq!(tables["SkillCharge"].layout.rows, rows);
    tables.get_mut("SkillCharge").unwrap().layout.rows[0] = json!({"ID":{"Value":1}});
    assert!(skills::check_relations(&tables).is_err());
}

#[test]
fn type_metadata_gaps_are_distinguished_from_definition_references() {
    let mut tables = tables();
    tables.get_mut("SkillAbnormal").unwrap().layout.rows = vec![json!({
        "ID":{"Value":700},"SkillAbnormalString_Key":"None","AbnormalEffectType":"EAbnormalEffectType::Snare","GroupId":70
    })];
    let checks = skills::check_relations(&tables).unwrap();
    let metadata = relation(&checks, "SkillAbnormal", "AbnormalEffectType");
    assert!(metadata.target_may_be_partial);
    assert_eq!(metadata.missing, 1);
    assert!(!relation(&checks, "SkillEffect", "SkillEffectLvGroupId").target_may_be_partial);
}

fn export_fixture_skills(
    fixture: &Fixture,
    output: &str,
) -> aetheris::game_data::catalog::Result<aetheris::game_data::skill_database::Summary> {
    for locale in ["en-US", "fr-FR"] {
        fs::write(
            fixture.root.join(format!("{locale}.json")),
            br#"{"namespace":"AION2","entries":{}}"#,
        )
        .unwrap();
    }
    fs::write(
        fixture.root.join("appmanifest.acf"),
        r#""AppState" {"appid" "3393110" "buildid" "100" "installdir" "AION2" "StateFlags" "4"}"#,
    )
    .unwrap();
    aetheris::game_data::skill_database::export(
        &fixture.root.join("mapping.usmap"),
        &fixture.root,
        &fixture.root.join("en-US.json"),
        &fixture.root.join("fr-FR.json"),
        &fixture.root.join("appmanifest.acf"),
        &fixture.root.join(output),
        "fr-FR",
    )
}

#[test]
fn skill_export_preserves_existing_items_and_failures_preserve_the_database() {
    let fixture = Fixture::new();
    let path = fixture.root.join("data.sqlite");
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch(include_str!("../src/game_data/items.sql"))
        .unwrap();
    db.execute_batch("INSERT INTO client_version VALUES(1,'100');
        INSERT INTO item_families VALUES(1,'Equip');
        INSERT INTO item_categories VALUES(1,1,'Greatsword');
        INSERT INTO rarities VALUES(1,'Unique','EItemGrade::Legend');
        INSERT INTO sell_currencies VALUES(1,'Gold','ECurrencyCategory::Gold');
        INSERT INTO item_kinds VALUES(1,'weapon');
        INSERT INTO exchange_types VALUES(1,'Exchangeable');
        INSERT INTO localization_language VALUES(1,'fr-FR');
        INSERT INTO items(id,client_version_id,category_id,rarity_id,sell_currency_id,kind_id,exchange_type_id,item_level,sell_price,max_stack_count,name_en,name_localized)
            VALUES(7,1,1,1,1,1,1,10,50,1,'Test sword','Épée de test');").unwrap();
    drop(db);
    let summary = export_fixture_skills(&fixture, "data.sqlite").unwrap();
    assert_eq!(summary.source_tables, 41);
    let db = rusqlite::Connection::open(&path).unwrap();
    let item:(i64,String,String)=db.query_row("SELECT items.id,name_localized,v.steam_build FROM items JOIN client_version v ON v.id=items.client_version_id",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(item, (7, "Épée de test".into(), "100".into()));
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        5
    );
    assert!(
        !db.prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
    drop(db);
    export_fixture_skills(&fixture, "data.sqlite").unwrap();
    let before = fs::read(&path).unwrap();
    fs::write(fixture.root.join("SkillEffect.bin"), []).unwrap();
    assert!(export_fixture_skills(&fixture, "data.sqlite").is_err());
    assert_eq!(before, fs::read(&path).unwrap());
}

#[test]
fn export_skills_cli_defaults_to_data_sqlite() {
    let fixture = Fixture::new();
    export_fixture_skills(&fixture, "explicit.sqlite").unwrap();
    let profile = fixture.root.join("profile");
    let result = Command::new(env!("CARGO_BIN_EXE_aetheris-game-data"))
        .env("LOCALAPPDATA", &profile)
        .arg("export-skills")
        .arg(fixture.root.join("mapping.usmap"))
        .arg(&fixture.root)
        .arg(fixture.root.join("en-US.json"))
        .arg(fixture.root.join("fr-FR.json"))
        .arg(fixture.root.join("appmanifest.acf"))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let db = rusqlite::Connection::open_with_flags(
        profile.join("Aetheris/data.sqlite"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    assert_eq!(
        db.query_row("SELECT count(*) FROM skills", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}
