#![recursion_limit = "256"]

use aetheris::game_data::items;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

struct Fixture {
    root: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
impl Fixture {
    fn new(rows: Vec<Value>) -> Self {
        let mut random = [0u8; 16];
        getrandom::fill(&mut random).unwrap();
        let root = std::env::temp_dir().join(format!("aetheris-items-{}", hex::encode(random)));
        fs::create_dir(&root).unwrap();
        let fixture = Self { root };
        fixture.probe(rows);
        for locale in ["en-US", "fr-FR"] {
            let entries = if locale == "en-US" {
                json!({
                    "String_NAME_SWORD_body":"Test Sword", "String_DESC_SWORD_body":"<desc_point>{stat:1}</>",
                    "String_NAME_POTION_body":"Test Potion"
                })
            } else {
                json!({"String_NAME_SWORD_body":"Épée de test", "String_DESC_SWORD_body":"<desc_point>{stat:1}</>"})
            };
            fs::write(
                fixture.root.join(format!("{locale}.json")),
                serde_json::to_vec(&json!({"namespace":"AION2","entries":entries})).unwrap(),
            )
            .unwrap();
        }
        fs::write(
            fixture.root.join("appmanifest_3393110.acf"),
            r#""AppState" {
            "appid" "3393110" "buildid" "26581260" "installdir" "AION2" "StateFlags" "4"
        }"#,
        )
        .unwrap();
        fixture
    }
    fn probe(&self, rows: Vec<Value>) {
        fs::write(self.root.join("probe.json"), serde_json::to_vec(&json!({
            "mapping_sha256":"11".repeat(32), "mapping_version":4, "mapping_ambiguous_structures":[],
            "row_struct":"ItemTableItem", "payload_sha256":"22".repeat(32), "payload_bytes":100,
            "compatibility_verified":false,"table_version":13,"column_hashes":["one"],"rows":rows
        })).unwrap()).unwrap();
    }
    fn export(&self) -> aetheris::game_data::catalog::Result<items::Summary> {
        items::export(
            &self.root.join("probe.json"),
            &self.root.join("en-US.json"),
            &self.root.join("fr-FR.json"),
            &self.root.join("appmanifest_3393110.acf"),
            &self.root.join("items.sqlite3"),
        )
    }
    fn manifest(&self, build: u64) {
        fs::write(
            self.root.join("appmanifest_3393110.acf"),
            format!(
                r#""AppState" {{
            "appid" "3393110" "buildid" "{build}" "installdir" "AION2" "StateFlags" "4"
        }}"#
            ),
        )
        .unwrap();
    }
    fn versions(&self) -> std::collections::BTreeMap<i64, String> {
        self.database().prepare("SELECT i.id,v.steam_build FROM items i JOIN client_version v ON v.id=i.client_version_id ORDER BY i.id").unwrap()
            .query_map([],|r|Ok((r.get(0)?,r.get(1)?))).unwrap().collect::<rusqlite::Result<_>>().unwrap()
    }
    fn database(&self) -> Connection {
        let db = Connection::open(self.root.join("items.sqlite3")).unwrap();
        db.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        db
    }
}

fn item(id: i64) -> Value {
    json!({
        "ID":{"Value":id}, "Name":format!("test_{id}"),"Desc":{"Key":"NAME_SWORD"},"DescLong":{"Key":"DESC_SWORD"},
        "ItemType":"EItemType::Equip","ItemGrade":"EItemGrade::Legend","ItemTier":"EItemTier::t3","ItemLevel":102,
        "IconRes":"Icon_test_sword","SellPrice":250,"SellCurrencyType":"ECurrencyCategory::Gold",
        "CanSell":1,"CanDelete":1,"CanDecompose":1,"CanStorage":1,"CanCharacterStorage":true,"CanGuildStorage":0,"CanLock":1,"CanSoulSelect":false,
        "MaxStackCount":1,"PermitLevelMin":45,"PermitLevelMax":99,"ExchangeType":"EExchangeType::SoulBind_Acquire","ItemRace":"ERace::All",
        "UnknownSourceField":{"still_here":42},
        "$variant":{
            "EquipCategory":"EEquipCategory::Greatsword","ArmorType":"EArmorMaterialType::None","ItemBasicLevel":88,
            "ExtendingWeapon":true,"CanEnchant":1,"bCanExtractionSkin":false,"ExtractionSkinRate":2500,"ExtractionSkinCount":0,
            "ExtractionSkins":["Skin_A","Skin_B"],"SetNames":["Set_A"],"MagicStoneSlotCount":4,"GodStoneSlotCount":1,
            "MainStats":{"Removed":["EStat::OldStat"],"Entries":[{"Key":"EStat::WeaponMinDamage","Value":100},{"Key":"EStat::WeaponDamage","Value":200}]}
        }
    })
}

fn potion(id: i64) -> Value {
    let mut row = item(id);
    row["ItemType"] = json!("EItemType::Usable");
    row["ItemGrade"] = json!("EItemGrade::Rare");
    row["Desc"] = json!({"Key":"NAME_POTION"});
    row["DescLong"] = json!({"Key":"None"});
    row["$variant"] = json!({"UsableCategory":"EUsableCategory::Potion","Cooltime":30000,"UseEffect":"EItemUseEffect::Effect",
        "bCanAutoUse":true,"bCanQuickSlot":true,"CanUseClassList":[true,false,true]});
    row
}

#[test]
fn items_have_relational_references_and_real_boolean_permissions() {
    let fixture = Fixture::new(vec![potion(2), item(1), potion(3)]);
    let summary = fixture.export().unwrap();
    assert_eq!(
        (summary.items, summary.names_en, summary.names_localized),
        (3, 3, 1)
    );
    let db = fixture.database();
    let names: (String, String, String, String) = db
        .query_row(
            "SELECT i.name_en,i.name_localized,r.rarity,r.rarity_source FROM items i JOIN rarities r ON r.id=i.rarity_id WHERE i.id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        names,
        (
            "Test Sword".into(),
            "Épée de test".into(),
            "Unique".into(),
            "EItemGrade::Legend".into()
        )
    );
    let category:(String,String,i64)=db.query_row("SELECT f.code,c.code,d.value FROM items i JOIN item_categories c ON c.id=i.category_id JOIN item_families f ON f.id=c.family_id JOIN cooldowns d ON d.id=i.cooldown_id WHERE i.id=2",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(category, ("Usable".into(), "Potion".into(), 30000));
    assert_eq!(
        db.query_row("SELECT count(*) FROM cooldowns", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    let enabled:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM item_flags p JOIN flags f ON f.id=p.flag_id WHERE item_id=1 AND f.code='CanSell')",[],|r|r.get(0)).unwrap();
    assert!(enabled);
    let skin_enabled:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM item_flags p JOIN flags f ON f.id=p.flag_id WHERE item_id=1 AND f.code='bCanExtractionSkin')",[],|r|r.get(0)).unwrap();
    assert!(!skin_enabled);
    let classes: Vec<i64> = db
        .prepare(
            "SELECT class_index FROM item_class_permissions WHERE item_id=2 ORDER BY class_index",
        )
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(classes, vec![0, 2]);
    assert!(
        db.execute("UPDATE item_flags SET value=2 WHERE item_id=1", [])
            .is_err()
    );
    assert!(
        db.execute("UPDATE items SET category_id=999999 WHERE id=1", [])
            .is_err()
    );
    let stats: Vec<(String, i64)> = db
        .prepare("SELECT s.code,v.value FROM item_main_stats v JOIN stats s ON s.id=v.stat_id WHERE item_id=1 ORDER BY position")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        stats,
        vec![
            ("EStat::WeaponMinDamage".into(), 100),
            ("EStat::WeaponDamage".into(), 200)
        ]
    );
    let description: String = db
        .query_row(
            "SELECT description_localized FROM items WHERE id=1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(description, "<desc_point>{stat:1}</>");
    assert_eq!(
        db.query_row(
            "SELECT skin_extraction_rate FROM items WHERE id=1",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        2500
    );
}

#[test]
fn removed_fields_tables_and_json_columns_are_absent() {
    let fixture = Fixture::new(vec![item(1)]);
    fixture.export().unwrap();
    let db = fixture.database();
    let tables: Vec<(String, String)> = db
        .prepare("SELECT name,type FROM sqlite_master WHERE type IN ('table','view')")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert!(tables.iter().all(|(_, kind)| kind == "table"));
    for banned in ["item_definitions", "metadata", "item_localizations"] {
        assert!(!tables.iter().any(|(name, _)| name == banned));
    }
    let columns: Vec<(String, String)> = db
        .prepare("PRAGMA table_info(items)")
        .unwrap()
        .query_map([], |r| Ok((r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    for banned in [
        "raw_fields_json",
        "source_row",
        "skin_extraction_count_raw",
        "technical_name",
        "required_level_min",
        "required_level_max",
        "name_en_status",
        "name_fr_status",
        "description_en_status",
        "description_fr_status",
        "tier",
        "can_flags_json",
        "localized_locale",
        "equipment_level",
        "kind",
        "exchange_type",
    ] {
        assert!(
            !columns.iter().any(|(name, _)| name == banned),
            "retained {banned}"
        );
    }
    assert!(
        columns
            .iter()
            .all(|(name, _)| !name.ends_with("_fr") && !name.ends_with("_json"))
    );
    assert!(columns.contains(&("extensible".into(), "BOOLEAN".into())));
}

#[test]
fn missing_translations_and_inapplicable_equipment_fields_remain_null() {
    let fixture = Fixture::new(vec![potion(2)]);
    fixture.export().unwrap();
    let db = fixture.database();
    let values:(String,Option<String>,Option<String>,Option<bool>,Option<i64>,String)=db.query_row("SELECT name_en,name_localized,description_localized,extensible,skin_extraction_rate,k.code FROM items i JOIN item_kinds k ON k.id=i.kind_id",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).unwrap();
    assert_eq!(
        values,
        (
            "Test Potion".into(),
            None,
            None,
            None,
            None,
            "potion".into()
        )
    );
}

#[test]
fn latest_modification_version_changes_only_when_stored_item_data_changes() {
    let fixture = Fixture::new(vec![item(10), potion(20), item(30)]);
    fixture.manifest(100);
    fixture.export().unwrap();
    let mut sword = item(10);
    sword["SellPrice"] = json!(300);
    let mut other = item(30);
    other["CanSell"] = json!(false);
    // New lower ID renumbers the family/category references. This must not
    // mark the unchanged potion as modified.
    let rows = vec![potion(1), sword, potion(20), other];
    fixture.probe(rows.clone());
    fixture.manifest(101);
    let summary = fixture.export().unwrap();
    assert_eq!(
        (
            summary.new_items,
            summary.modified_items,
            summary.unchanged_items
        ),
        (1, 2, 1)
    );
    assert_eq!(
        fixture.versions(),
        [(1, "101"), (10, "101"), (20, "100"), (30, "101")]
            .map(|(id, v)| (id, v.to_string()))
            .into_iter()
            .collect()
    );
    assert_eq!(fixture.export().unwrap().unchanged_items, 4);
    let mut discarded_changes = rows;
    discarded_changes[1]["Name"] = json!("renamed technical key");
    discarded_changes[1]["ItemTier"] = json!("EItemTier::t4");
    discarded_changes[0]["$variant"]["ItemBasicLevel"] = json!(999);
    fixture.probe(discarded_changes);
    fixture.manifest(102);
    let summary = fixture.export().unwrap();
    assert_eq!((summary.modified_items, summary.unchanged_items), (0, 4));
    assert_eq!(fixture.versions()[&20], "100");
    let db = fixture.database();
    let changed:i64=db.query_row("SELECT count(*) FROM items WHERE client_version_id=(SELECT max(id) FROM client_version)",[],|r|r.get(0)).unwrap();
    assert_eq!(changed, 0);
}

#[test]
fn changes_to_related_tables_also_mark_the_item_as_modified() {
    for field in ["flag", "stat", "skin", "set", "cooldown", "armor", "class"] {
        let fixture = Fixture::new(vec![item(1), potion(2)]);
        fixture.manifest(100);
        fixture.export().unwrap();
        let mut sword = item(1);
        let mut usable = potion(2);
        match field {
            "flag" => sword["CanLock"] = json!(false),
            "stat" => sword["$variant"]["MainStats"]["Entries"][0]["Value"] = json!(150),
            "skin" => sword["$variant"]["ExtractionSkins"][0] = json!("Skin_C"),
            "set" => sword["$variant"]["SetNames"][0] = json!("Set_B"),
            "cooldown" => usable["$variant"]["Cooltime"] = json!(1000),
            "armor" => sword["$variant"]["ArmorType"] = json!("EArmorMaterialType::Armor"),
            "class" => usable["$variant"]["CanUseClassList"][0] = json!(false),
            _ => unreachable!(),
        }
        fixture.probe(vec![sword, usable]);
        fixture.manifest(101);
        let summary = fixture.export().unwrap();
        assert_eq!(
            (summary.modified_items, summary.unchanged_items),
            (1, 1),
            "{field}"
        );
    }
}

#[test]
fn switching_selected_locale_does_not_mark_items_as_changed() {
    let fixture = Fixture::new(vec![item(1), potion(2)]);
    fixture.manifest(100);
    fixture.export().unwrap();
    fixture.manifest(101);
    let summary = items::export_localized(
        &fixture.root.join("probe.json"),
        &fixture.root.join("en-US.json"),
        &fixture.root.join("en-US.json"),
        "en-US",
        &fixture.root.join("appmanifest_3393110.acf"),
        &fixture.root.join("items.sqlite3"),
    )
    .unwrap();
    assert_eq!((summary.modified_items, summary.unchanged_items), (0, 2));
    assert!(fixture.versions().values().all(|v| v == "100"));
    let name: String = fixture
        .database()
        .query_row("SELECT name_localized FROM items WHERE id=1", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(name, "Test Sword");
}

#[test]
fn failed_imports_preserve_previous_database_and_clean_staging() {
    let fixture = Fixture::new(vec![item(1)]);
    fixture.export().unwrap();
    let original = fs::read(fixture.root.join("items.sqlite3")).unwrap();
    for mutation in [
        "duplicate",
        "negative_price",
        "bad_flag",
        "bad_variant",
        "bad_stat",
        "invalid_id",
        "missing_name_key",
    ] {
        let mut broken = item(2);
        match mutation {
            "duplicate" => broken = item(1),
            "negative_price" => broken["SellPrice"] = json!(-1),
            "bad_flag" => broken["CanSell"] = json!(2),
            "bad_variant" => broken["ItemType"] = json!("EItemType::Future"),
            "bad_stat" => {
                broken["$variant"]["MainStats"]["Entries"][0]["Value"] = json!("approximate")
            }
            "invalid_id" => broken["ID"]["Value"] = json!(0),
            "missing_name_key" => broken["Desc"] = json!(null),
            _ => unreachable!(),
        }
        fixture.probe(vec![item(1), broken]);
        assert!(fixture.export().is_err(), "accepted {mutation}");
        assert_eq!(
            original,
            fs::read(fixture.root.join("items.sqlite3")).unwrap()
        );
        assert!(fs::read_dir(&fixture.root).unwrap().all(|e| {
            !e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".items-")
        }));
    }
    fixture.manifest(100);
    assert!(fixture.export().is_err());
    assert_eq!(
        original,
        fs::read(fixture.root.join("items.sqlite3")).unwrap()
    );
}

#[test]
fn legacy_export_is_rebuilt_with_a_first_observed_version() {
    let fixture = Fixture::new(vec![item(1)]);
    fixture.database().execute_batch("CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT); INSERT INTO metadata VALUES('format_version','1');").unwrap();
    assert_eq!(fixture.export().unwrap().new_items, 1);
    assert_eq!(fixture.versions()[&1], "26581260");
    assert_eq!(
        fixture
            .database()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        4
    );
}

#[test]
fn export_refuses_to_replace_its_inputs() {
    let fixture = Fixture::new(vec![item(1)]);
    for name in [
        "probe.json",
        "en-US.json",
        "fr-FR.json",
        "appmanifest_3393110.acf",
    ] {
        let output = fixture.root.join(name);
        let before = fs::read(&output).unwrap();
        assert!(
            items::export(
                &fixture.root.join("probe.json"),
                &fixture.root.join("en-US.json"),
                &fixture.root.join("fr-FR.json"),
                &fixture.root.join("appmanifest_3393110.acf"),
                &output
            )
            .is_err()
        );
        assert_eq!(before, fs::read(output).unwrap());
    }
}

#[test]
fn cli_rebuilds_the_same_database_reproducibly() {
    let fixture = Fixture::new(vec![item(1), potion(2)]);
    fixture.export().unwrap();
    let before = fs::read(fixture.root.join("items.sqlite3")).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_aetheris-game-data"))
        .arg("export-items")
        .arg(fixture.root.join("probe.json"))
        .arg(fixture.root.join("en-US.json"))
        .arg(fixture.root.join("fr-FR.json"))
        .arg(fixture.root.join("appmanifest_3393110.acf"))
        .arg(fixture.root.join("items.sqlite3"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["unchanged_items"],
        2
    );
    assert_eq!(
        before,
        fs::read(fixture.root.join("items.sqlite3")).unwrap()
    );
}

#[test]
fn reference_tables_use_bounded_smallints_and_flags_store_only_membership() {
    let fixture = Fixture::new(vec![item(1), potion(2)]);
    fixture.export().unwrap();
    let db = fixture.database();
    for table in [
        "client_version",
        "item_families",
        "item_categories",
        "armor_types",
        "cooldowns",
        "flags",
        "rarities",
        "sell_currencies",
        "stats",
        "races",
        "sets",
        "item_kinds",
        "exchange_types",
    ] {
        let kind: String = db
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap()
            .query_map([], |r| Ok((r.get::<_, String>(1)?, r.get::<_, String>(2)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .find(|(name, _)| name == "id")
            .unwrap()
            .1;
        assert_eq!(kind, "SMALLINT", "{table}");
    }
    for id in [0, 65536] {
        assert!(
            db.execute("INSERT INTO flags VALUES (?1,'Invalid')", [id])
                .is_err()
        );
    }
    assert!(
        db.execute("INSERT INTO flags VALUES (1.5,'Fractional')", [])
            .is_err()
    );
    assert!(
        db.execute("INSERT INTO flags VALUES (65535,'UpperBound')", [])
            .is_ok()
    );
    for (table, removed) in [
        ("item_flags", "value"),
        ("item_class_permissions", "can_use"),
        ("item_extraction_skins", "position"),
        ("item_sets", "position"),
    ] {
        let fields: Vec<String> = db
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap()
            .query_map([], |r| r.get(1))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert!(!fields.iter().any(|field| field == removed));
    }
    let currency:(String,String)=db.query_row("SELECT m.code,m.sell_currency_type FROM items i JOIN sell_currencies m ON m.id=i.sell_currency_id WHERE i.id=1",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(currency, ("Gold".into(), "ECurrencyCategory::Gold".into()));
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM flags WHERE code='bCanExtractionSkin'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(db.query_row("SELECT count(*) FROM item_flags p JOIN flags f ON f.id=p.flag_id WHERE f.code='bCanExtractionSkin'",[],|r|r.get::<_,i64>(0)).unwrap(),0);
}

#[test]
fn races_are_explicit_and_unspecified_is_not_all() {
    let mut rows = Vec::new();
    for (id, code) in [(1, "All"), (2, "Light"), (3, "Dark"), (4, "None")] {
        let mut row = item(id);
        row["ItemRace"] = json!(format!("ERace::{code}"));
        rows.push(row);
    }
    let fixture = Fixture::new(rows);
    fixture.export().unwrap();
    let db = fixture.database();
    let races:Vec<(i64,Option<String>,Option<String>)>=db.prepare("SELECT i.id,r.code,r.name FROM items i LEFT JOIN races r ON r.id=i.race_id ORDER BY i.id").unwrap().query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(
        races,
        vec![
            (1, Some("All".into()), Some("All".into())),
            (2, Some("Light".into()), Some("Elyos".into())),
            (3, Some("Dark".into()), Some("Asmodian".into())),
            (4, None, None)
        ]
    );
}

#[test]
fn unordered_skin_and_set_links_are_deduplicated_and_do_not_change_versions() {
    let mut sword = item(1);
    sword["$variant"]["SetNames"] = json!(["Set_B", "Set_A"]);
    let fixture = Fixture::new(vec![sword.clone()]);
    fixture.manifest(100);
    fixture.export().unwrap();
    sword["$variant"]["ExtractionSkins"] = json!(["Skin_B", "Skin_A", "Skin_B"]);
    sword["$variant"]["SetNames"] = json!(["Set_A", "Set_B", "Set_A"]);
    fixture.probe(vec![sword]);
    fixture.manifest(101);
    let summary = fixture.export().unwrap();
    assert_eq!(summary.unchanged_items, 1);
    assert_eq!(summary.extraction_skins, 2);
    assert_eq!(fixture.versions()[&1], "100");
    assert_eq!(
        fixture
            .database()
            .query_row("SELECT count(*) FROM sets", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        2
    );
}

#[test]
fn manually_filled_stat_terms_survive_regeneration_without_marking_items_modified() {
    let fixture = Fixture::new(vec![item(1)]);
    fixture.manifest(100);
    fixture.export().unwrap();
    let db = fixture.database();
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM stats WHERE name_en IS NOT NULL OR name_localized IS NOT NULL",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    let id: i64 = db
        .query_row(
            "SELECT id FROM stats WHERE code='EStat::WeaponDamage'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    db.execute(
        "UPDATE stats SET name_en='Weapon damage',name_localized='Dégâts de l’arme' WHERE id=?1",
        [id],
    )
    .unwrap();
    drop(db);
    fixture.manifest(101);
    assert_eq!(fixture.export().unwrap().unchanged_items, 1);
    let labels: (i64, String, String) = fixture
        .database()
        .query_row(
            "SELECT id,name_en,name_localized FROM stats WHERE code='EStat::WeaponDamage'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        labels,
        (id, "Weapon damage".into(), "Dégâts de l’arme".into())
    );
    assert_eq!(fixture.versions()[&1], "100");
}

#[test]
fn migration_from_v2_preserves_last_modification_links() {
    let fixture = Fixture::new(vec![item(1), potion(2)]);
    fixture.manifest(100);
    fixture.export().unwrap();
    let mut changed = potion(2);
    changed["SellPrice"] = json!(500);
    fixture.probe(vec![item(1), changed]);
    fixture.manifest(101);
    fixture.export().unwrap();
    let versions = fixture.versions();
    fixture.database().execute_batch("PRAGMA foreign_keys=OFF;
        CREATE TABLE old_items AS SELECT i.*,k.code AS kind,x.code AS exchange_type,NULL AS equipment_level,'fr-FR' AS localized_locale,r.rarity,r.rarity_source,m.code AS sell_currency,COALESCE(e.code,'None') AS race
            FROM items i JOIN item_kinds k ON k.id=i.kind_id JOIN exchange_types x ON x.id=i.exchange_type_id JOIN rarities r ON r.id=i.rarity_id JOIN sell_currencies m ON m.id=i.sell_currency_id LEFT JOIN races e ON e.id=i.race_id;
        CREATE TABLE old_permissions AS SELECT item_id,class_index,1 AS can_use FROM item_class_permissions;
        INSERT INTO old_permissions VALUES (2,1,0);
        DROP TABLE item_class_permissions; ALTER TABLE old_permissions RENAME TO item_class_permissions;
        CREATE TABLE old_flags AS SELECT item_id,flag_id,1 AS value FROM item_flags;
        INSERT INTO old_flags SELECT 1,id,0 FROM flags WHERE code='bCanExtractionSkin';
        CREATE TABLE old_stats AS SELECT p.item_id,p.position,s.code AS stat,p.value FROM item_main_stats p JOIN stats s ON s.id=p.stat_id;
        CREATE TABLE old_skins AS SELECT item_id,0 AS position,skin_key FROM item_extraction_skins;
        CREATE TABLE old_sets AS SELECT p.item_id,0 AS position,s.code AS set_key FROM item_sets p JOIN sets s ON s.id=p.set_id;
        DROP TABLE items; ALTER TABLE old_items RENAME TO items;
        CREATE UNIQUE INDEX migration_item_id ON items(id);
        DROP TABLE item_flags; ALTER TABLE old_flags RENAME TO item_flags;
        DROP TABLE item_main_stats; ALTER TABLE old_stats RENAME TO item_main_stats;
        DROP TABLE item_extraction_skins; ALTER TABLE old_skins RENAME TO item_extraction_skins;
        DROP TABLE item_sets; ALTER TABLE old_sets RENAME TO item_sets;
        PRAGMA user_version=2;").unwrap();
    assert_eq!(fixture.export().unwrap().unchanged_items, 2);
    assert_eq!(fixture.versions(), versions);
}

#[test]
fn migration_from_v3_preserves_versions_and_ignores_false_class_permissions() {
    let fixture = Fixture::new(vec![item(1), potion(2)]);
    fixture.manifest(100);
    fixture.export().unwrap();
    fixture.database().execute_batch("PRAGMA foreign_keys=OFF;
        CREATE TABLE old_items AS SELECT i.*,k.code AS kind,x.code AS exchange_type,88 AS equipment_level,'fr-FR' AS localized_locale
            FROM items i JOIN item_kinds k ON k.id=i.kind_id JOIN exchange_types x ON x.id=i.exchange_type_id;
        DROP TABLE items; ALTER TABLE old_items RENAME TO items;
        CREATE UNIQUE INDEX migration_item_id ON items(id);
        CREATE TABLE old_permissions AS SELECT item_id,class_index,1 AS can_use FROM item_class_permissions;
        INSERT INTO old_permissions VALUES (2,1,0);
        DROP TABLE item_class_permissions; ALTER TABLE old_permissions RENAME TO item_class_permissions;
        PRAGMA user_version=3;").unwrap();
    fixture.manifest(101);
    assert_eq!(fixture.export().unwrap().unchanged_items, 2);
    assert!(fixture.versions().values().all(|version| version == "100"));
    let db = fixture.database();
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM item_class_permissions WHERE item_id=2",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        2
    );
}

#[test]
fn stone_slots_are_unsigned_bytes_and_invalid_source_preserves_previous_database() {
    let fixture = Fixture::new(vec![item(1)]);
    fixture.export().unwrap();
    let db = fixture.database();
    for column in ["magic_stone_slots", "godstone_slots"] {
        let declaration: String = db
            .query_row(
                "SELECT type FROM pragma_table_info('items') WHERE name=?1",
                [column],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(declaration, "TINYINT");
        for value in [0, 255] {
            db.execute(&format!("UPDATE items SET {column}=?1"), [value])
                .unwrap();
        }
        for value in [-1.0, 256.0, 1.5] {
            assert!(
                db.execute(&format!("UPDATE items SET {column}=?1"), [value])
                    .is_err()
            );
        }
        db.execute(&format!("UPDATE items SET {column}=NULL"), [])
            .unwrap();
    }
    drop(db);
    let before = fs::read(fixture.root.join("items.sqlite3")).unwrap();
    for field in ["MagicStoneSlotCount", "GodStoneSlotCount"] {
        let mut sword = item(1);
        sword["$variant"][field] = json!(256);
        fixture.probe(vec![sword]);
        assert!(fixture.export().is_err());
        assert_eq!(
            fs::read(fixture.root.join("items.sqlite3")).unwrap(),
            before
        );
    }
}
