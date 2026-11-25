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
            "Legend".into(),
            "EItemGrade::Legend".into()
        )
    );
    let category:(String,String)=db.query_row("SELECT f.code,c.code FROM items i JOIN item_categories c ON c.id=i.category_id JOIN item_families f ON f.id=c.family_id WHERE i.id=2",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(category, ("Usable".into(), "Potion".into()));
    let enabled:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM item_flags p JOIN flags f ON f.id=p.flag_id WHERE item_id=1 AND f.code='CanSell')",[],|r|r.get(0)).unwrap();
    assert!(enabled);
    let skin_enabled:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM item_flags p JOIN flags f ON f.id=p.flag_id WHERE item_id=1 AND f.code='bCanExtractionSkin')",[],|r|r.get(0)).unwrap();
    assert!(!skin_enabled);
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
    for banned in [
        "item_definitions",
        "metadata",
        "item_localizations",
        "cooldowns",
        "item_kinds",
        "exchange_types",
        "item_class_permissions",
        "item_extraction_skins",
    ] {
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
        "cooldown_id",
        "kind_id",
        "exchange_type_id",
        "max_stack_count",
        "use_effect",
        "extensible",
        "skin_extraction_rate",
        "icon_key",
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
}

#[test]
fn missing_translations_and_inapplicable_equipment_fields_remain_null() {
    let fixture = Fixture::new(vec![potion(2)]);
    fixture.export().unwrap();
    let db = fixture.database();
    let values:(String,Option<String>,Option<String>,Option<i64>,Option<i64>)=db.query_row("SELECT name_en,name_localized,description_localized,magic_stone_slots,godstone_slots FROM items",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap();
    assert_eq!(values, ("Test Potion".into(), None, None, None, None));
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
fn reference_tables_use_bounded_smallints_and_flags_store_only_membership() {
    let fixture = Fixture::new(vec![item(1), potion(2)]);
    fixture.export().unwrap();
    let db = fixture.database();
    for table in [
        "client_version",
        "item_families",
        "item_categories",
        "armor_types",
        "flags",
        "rarities",
        "sell_currencies",
        "stats",
        "races",
        "sets",
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
        db.execute("INSERT INTO flags VALUES (32767,'UpperBound')", [])
            .is_ok()
    );
    for (table, removed) in [("item_flags", "value"), ("item_sets", "position")] {
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
