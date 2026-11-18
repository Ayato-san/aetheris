//! Relational item export, with observed modification versions and no raw rows.
use super::{assets::Localization, binary::invalid, catalog::Result, discovery, usmap::TableProbe};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, params, params_from_iter};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const APPLICATION_ID: i64 = 0x41454954;

#[derive(Debug, Default, Serialize)]
pub struct Summary {
    pub items: usize,
    pub names_en: usize,
    pub names_localized: usize,
    pub descriptions_en: usize,
    pub descriptions_localized: usize,
    pub main_stats: usize,
    pub extraction_skins: usize,
    pub new_items: usize,
    pub modified_items: usize,
    pub unchanged_items: usize,
    pub steam_build: String,
    pub localized_locale: String,
    pub source_sha256: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct Item<'a> {
    id: i64,
    client_version_id: i64,
    category_id: i64,
    armor_type_id: Option<i64>,
    cooldown_id: Option<i64>,
    rarity_id: i64,
    sell_currency_id: i64,
    race_id: Option<i64>,
    name_en: Option<&'a str>,
    name_localized: Option<&'a str>,
    description_en: Option<&'a str>,
    description_localized: Option<&'a str>,
    kind_id: i64,
    item_level: i64,
    icon_key: Option<&'a str>,
    sell_price: i64,
    extensible: Option<bool>,
    skin_extraction_rate: Option<i64>,
    max_stack_count: i64,
    exchange_type_id: i64,
    use_effect: Option<&'a str>,
    magic_stone_slots: Option<i64>,
    godstone_slots: Option<i64>,
}

struct Normalized<'a> {
    item: Item<'a>,
    kind: &'static str,
    exchange_type: &'a str,
    family: &'a str,
    category: &'a str,
    armor_type: Option<&'a str>,
    cooldown: Option<i64>,
    rarity: &'a str,
    rarity_source: &'a str,
    currency: &'a str,
    currency_type: &'a str,
    race: Option<&'a str>,
    flags: BTreeMap<String, bool>,
    classes: Vec<bool>,
}

fn text<'a>(row: &'a Value, field: &str) -> Result<&'a str> {
    row.get(field)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid(format!("missing or invalid item field {field}")).into())
}
fn integer(row: &Value, field: &str) -> Result<i64> {
    row.get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| invalid(format!("missing or invalid item integer {field}")).into())
}
fn nonnegative(row: &Value, field: &str) -> Result<i64> {
    let value = integer(row, field)?;
    if value < 0 {
        return Err(invalid(format!("negative item field {field}")).into());
    }
    Ok(value)
}
fn optional_integer(row: &Value, field: &str) -> Result<Option<i64>> {
    row.get(field).map(|_| nonnegative(row, field)).transpose()
}
fn optional_byte(row: &Value, field: &str) -> Result<Option<i64>> {
    let value = optional_integer(row, field)?;
    if value.is_some_and(|value| value > 255) {
        return Err(invalid(format!("item field {field} exceeds 255")).into());
    }
    Ok(value)
}
fn optional_text<'a>(row: &'a Value, field: &str) -> Result<Option<&'a str>> {
    row.get(field)
        .map(|_| text(row, field))
        .transpose()
        .map(|v| v.filter(|s| *s != "None"))
}
fn enum_value<'a>(row: &'a Value, field: &str, prefix: &str) -> Result<&'a str> {
    text(row, field)?
        .strip_prefix(prefix)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid(format!("invalid item enum {field}")).into())
}
fn string_key<'a>(row: &'a Value, field: &str) -> Result<Option<&'a str>> {
    let key = text(
        row.get(field)
            .ok_or_else(|| invalid(format!("missing item string {field}")))?,
        "Key",
    )?;
    Ok((key != "None").then_some(key))
}
fn boolean(value: &Value) -> Result<bool> {
    value
        .as_bool()
        .or_else(|| match value.as_i64() {
            Some(0) => Some(false),
            Some(1) => Some(true),
            _ => None,
        })
        .ok_or_else(|| invalid("invalid item boolean").into())
}
fn array<'a>(row: &'a Value, field: &str) -> Result<&'a Vec<Value>> {
    row.get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(format!("invalid item array {field}")).into())
}
fn kind(family: &str, category: &str) -> &'static str {
    match (family, category) {
        (
            "Equip",
            "Greatsword" | "Sword" | "Dagger" | "Bow" | "Magicbook" | "Orb" | "Mace" | "Staff",
        ) => "weapon",
        ("Equip", "Torso" | "Pants" | "Helmet" | "Shoulder" | "Gloves" | "Boots" | "Cape") => {
            "armor"
        }
        ("Equip", "Necklace" | "Earring" | "Ring" | "Bracelet" | "Belt" | "Amulet") => "accessory",
        ("Equip", _) => "equipment",
        ("Usable", "Potion") => "potion",
        ("Usable", _) => "usable",
        ("Currency", _) => "currency",
        (
            "Misc",
            "Material"
            | "CraftResource"
            | "GatherResource"
            | "ConversionResource"
            | "ConversionResource_Special",
        ) => "material",
        _ => "misc",
    }
}

fn permissions(row: &Value, variant: &Value) -> Result<(BTreeMap<String, bool>, Vec<bool>)> {
    let mut flags = BTreeMap::new();
    let mut classes = Vec::new();
    let mut seen = BTreeSet::new();
    for object in [row, variant] {
        for (key, value) in object
            .as_object()
            .ok_or_else(|| invalid("invalid item object"))?
        {
            if !key.starts_with("Can") && !key.starts_with("bCan") {
                continue;
            }
            if !seen.insert(key) {
                return Err(invalid(format!("duplicate item permission {key}")).into());
            }
            if key == "CanUseClassList" {
                classes = value
                    .as_array()
                    .ok_or_else(|| invalid("invalid item class permissions"))?
                    .iter()
                    .map(boolean)
                    .collect::<Result<_>>()?;
            } else {
                flags.insert(key.clone(), boolean(value)?);
            }
        }
    }
    for key in [
        "CanSell",
        "CanDelete",
        "CanDecompose",
        "CanStorage",
        "CanCharacterStorage",
        "CanGuildStorage",
        "CanLock",
        "CanSoulSelect",
    ] {
        if !flags.contains_key(key) {
            return Err(invalid(format!("missing item permission {key}")).into());
        }
    }
    Ok((flags, classes))
}

fn translation<'a>(localization: &'a Localization, key: Option<&str>) -> Option<&'a str> {
    key.and_then(|key| localization.entries.get(&format!("String_{key}_body")))
        .map(String::as_str)
}

fn normalize<'a>(
    row: &'a Value,
    english: &'a Localization,
    localized: &'a Localization,
) -> Result<Normalized<'a>> {
    let id = nonnegative(
        row.get("ID").ok_or_else(|| invalid("missing item ID"))?,
        "Value",
    )?;
    if id == 0 {
        return Err(invalid("zero item ID").into());
    }
    let variant = row
        .get("$variant")
        .filter(|v| v.is_object())
        .ok_or_else(|| invalid("missing item variant"))?;
    let family = enum_value(row, "ItemType", "EItemType::")?;
    let (category_field, prefix) = match family {
        "Equip" => ("EquipCategory", "EEquipCategory::"),
        "Usable" => ("UsableCategory", "EUsableCategory::"),
        "Misc" => ("MiscCategory", "EMiscCategory::"),
        "Currency" => ("CurrencyCategory", "ECurrencyCategory::"),
        _ => return Err(invalid("unsupported item family").into()),
    };
    let category = enum_value(variant, category_field, prefix)?;
    let item_kind = kind(family, category);
    let rarity = enum_value(row, "ItemGrade", "EItemGrade::")?;
    let race = enum_value(row, "ItemRace", "ERace::")?;
    if !matches!(race, "All" | "Light" | "Dark" | "None") {
        return Err(invalid("unsupported item race").into());
    }
    let armor_type = if variant.get("ArmorType").is_some() {
        let value = enum_value(variant, "ArmorType", "EArmorMaterialType::")?;
        (value != "None").then_some(value)
    } else {
        None
    };
    let name_key = string_key(row, "Desc")?;
    let description_key = string_key(row, "DescLong")?;
    let (flags, classes) = permissions(row, variant)?;
    Ok(Normalized {
        kind: item_kind,
        exchange_type: enum_value(row, "ExchangeType", "EExchangeType::")?,
        family,
        category,
        armor_type,
        cooldown: optional_integer(variant, "Cooltime")?,
        rarity: if rarity == "Legend" { "Unique" } else { rarity },
        rarity_source: text(row, "ItemGrade")?,
        currency: enum_value(row, "SellCurrencyType", "ECurrencyCategory::")?,
        currency_type: text(row, "SellCurrencyType")?,
        race: (race != "None").then_some(race),
        flags,
        classes,
        item: Item {
            id,
            client_version_id: 0,
            category_id: 0,
            armor_type_id: None,
            cooldown_id: None,
            rarity_id: 0,
            sell_currency_id: 0,
            race_id: None,
            name_en: translation(english, name_key),
            name_localized: translation(localized, name_key),
            description_en: translation(english, description_key),
            description_localized: translation(localized, description_key),
            kind_id: 0,
            item_level: nonnegative(row, "ItemLevel")?,
            icon_key: optional_text(row, "IconRes")?,
            sell_price: nonnegative(row, "SellPrice")?,
            extensible: if item_kind == "weapon" {
                Some(boolean(
                    variant
                        .get("ExtendingWeapon")
                        .ok_or_else(|| invalid("missing extending weapon flag"))?,
                )?)
            } else {
                None
            },
            skin_extraction_rate: if family == "Equip" {
                Some(nonnegative(variant, "ExtractionSkinRate")?)
            } else {
                None
            },
            max_stack_count: nonnegative(row, "MaxStackCount")?,
            exchange_type_id: 0,
            use_effect: optional_text(variant, "UseEffect")?,
            magic_stone_slots: optional_byte(variant, "MagicStoneSlotCount")?,
            godstone_slots: optional_byte(variant, "GodStoneSlotCount")?,
        },
    })
}

fn read(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(invalid("item export input exceeds 64 MiB").into());
    }
    Ok(bytes)
}
fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[derive(Default)]
struct Previous {
    versions: Vec<(i64, String)>,
    item_versions: BTreeMap<i64, i64>,
    fingerprints: BTreeMap<i64, String>,
    include_localized: bool,
    stats: Vec<StatTerms>,
}

struct StatTerms {
    id: i64,
    code: String,
    name_en: Option<String>,
    name_localized: Option<String>,
}

fn check_integrity(db: &Connection) -> Result<()> {
    let integrity: String = db.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" || db.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(invalid("invalid item SQLite integrity or foreign keys").into());
    }
    Ok(())
}

/// Compare natural reference values, never surrogate IDs or modification IDs.
fn fingerprints(
    db: &Connection,
    include_localized: bool,
    version: i64,
) -> Result<BTreeMap<i64, String>> {
    let main_query = if version == 2 {
        "SELECT i.id,f.code,c.code,a.code,d.value,i.name_en,i.description_en,
        CASE WHEN ?1 THEN i.name_localized END,CASE WHEN ?1 THEN i.description_localized END,
        i.kind,i.item_level,i.rarity,i.rarity_source,i.icon_key,i.sell_price,i.sell_currency,i.extensible,
        i.skin_extraction_rate,i.max_stack_count,i.exchange_type,i.race,i.use_effect,i.magic_stone_slots,i.godstone_slots
        FROM items i JOIN item_categories c ON c.id=i.category_id JOIN item_families f ON f.id=c.family_id
        LEFT JOIN armor_types a ON a.id=i.armor_type_id LEFT JOIN cooldowns d ON d.id=i.cooldown_id ORDER BY i.id"
    } else {
        "SELECT i.id,f.code,c.code,a.code,d.value,i.name_en,i.description_en,
        CASE WHEN ?1 THEN i.name_localized END,CASE WHEN ?1 THEN i.description_localized END,
        i.kind,i.item_level,r.rarity,r.rarity_source,i.icon_key,i.sell_price,m.code,i.extensible,
        i.skin_extraction_rate,i.max_stack_count,i.exchange_type,COALESCE(e.code,'None'),i.use_effect,i.magic_stone_slots,i.godstone_slots
        FROM items i JOIN item_categories c ON c.id=i.category_id JOIN item_families f ON f.id=c.family_id
        JOIN rarities r ON r.id=i.rarity_id JOIN sell_currencies m ON m.id=i.sell_currency_id
        LEFT JOIN races e ON e.id=i.race_id
        LEFT JOIN armor_types a ON a.id=i.armor_type_id LEFT JOIN cooldowns d ON d.id=i.cooldown_id ORDER BY i.id"
    };
    let main_query = if version >= 4 {
        main_query.replace("i.kind,", "k.code,")
            .replace("i.exchange_type,", "x.code,")
            .replace("FROM items i JOIN", "FROM items i JOIN item_kinds k ON k.id=i.kind_id JOIN exchange_types x ON x.id=i.exchange_type_id JOIN")
    } else {
        main_query.to_owned()
    };
    let queries = [
        main_query.as_str(),
        if version == 2 {
            "SELECT v.item_id,f.code FROM item_flags v JOIN flags f ON f.id=v.flag_id WHERE v.value=1 ORDER BY v.item_id,f.code"
        } else {
            "SELECT v.item_id,f.code FROM item_flags v JOIN flags f ON f.id=v.flag_id ORDER BY v.item_id,f.code"
        },
        if version >= 4 {
            "SELECT item_id,class_index FROM item_class_permissions ORDER BY item_id,class_index"
        } else {
            "SELECT item_id,class_index FROM item_class_permissions WHERE can_use=1 ORDER BY item_id,class_index"
        },
        if version == 2 {
            "SELECT item_id,position,stat,value FROM item_main_stats ORDER BY item_id,position"
        } else {
            "SELECT v.item_id,v.position,s.code,v.value FROM item_main_stats v JOIN stats s ON s.id=v.stat_id ORDER BY v.item_id,v.position"
        },
        "SELECT DISTINCT item_id,skin_key FROM item_extraction_skins ORDER BY item_id,skin_key",
        if version == 2 {
            "SELECT DISTINCT item_id,set_key FROM item_sets ORDER BY item_id,set_key"
        } else {
            "SELECT v.item_id,s.code FROM item_sets v JOIN sets s ON s.id=v.set_id ORDER BY v.item_id,s.code"
        },
    ];
    let mut hashes: BTreeMap<i64, Sha256> = BTreeMap::new();
    for (section, query) in queries.iter().enumerate() {
        let mut statement = db.prepare(query)?;
        let count = statement.column_count();
        let mut rows = if section == 0 {
            statement.query([include_localized])?
        } else {
            statement.query([])?
        };
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let mut values = Vec::new();
            for index in 1..count {
                use rusqlite::types::ValueRef;
                values.push(match row.get_ref(index)? {
                    ValueRef::Null => Value::Null,
                    ValueRef::Integer(value) => Value::from(value),
                    ValueRef::Text(value) => Value::from(std::str::from_utf8(value)?),
                    _ => return Err(invalid("unexpected item database value type").into()),
                });
            }
            let hash = hashes.entry(id).or_default();
            let bytes = serde_json::to_vec(&(section, values))?;
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
    }
    Ok(hashes
        .into_iter()
        .map(|(id, hash)| (id, hex::encode(hash.finalize())))
        .collect())
}

fn previous(path: &Path, locale: &str) -> Result<Previous> {
    if !path.exists() {
        return Ok(Previous::default());
    }
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let application: i64 = db.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if application == 0 && version == 0 {
        // The first export had no modification history. Rebuild it from the
        // source, starting the first observation at the supplied client build.
        let legacy: Option<String> = db
            .query_row(
                "SELECT value FROM metadata WHERE key='format_version'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        if legacy.as_deref() == Some("1") {
            check_integrity(&db)?;
            return Ok(Previous::default());
        }
    }
    if application != APPLICATION_ID || !matches!(version, 2..=4) {
        return Err(invalid("unsupported previous item database").into());
    }
    check_integrity(&db)?;
    let versions = db
        .prepare("SELECT id,steam_build FROM client_version ORDER BY id")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let item_versions = db
        .prepare("SELECT id,client_version_id FROM items ORDER BY id")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<BTreeMap<_, _>>>()?;
    let locales = db
        .prepare(if version >= 4 {
            "SELECT locale FROM localization_language"
        } else {
            "SELECT DISTINCT localized_locale FROM items"
        })?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if locales.len() > 1 {
        return Err(invalid("inconsistent previous item locale").into());
    }
    let include_localized = locales.first().is_some_and(|value| value == locale);
    let stats = if version >= 3 {
        db.prepare("SELECT id,code,name_en,name_localized FROM stats ORDER BY id")?
            .query_map([], |r| {
                Ok(StatTerms {
                    id: r.get(0)?,
                    code: r.get(1)?,
                    name_en: r.get(2)?,
                    name_localized: if include_localized { r.get(3)? } else { None },
                })
            })?
            .collect::<rusqlite::Result<_>>()?
    } else {
        Vec::new()
    };
    Ok(Previous {
        versions,
        item_versions,
        fingerprints: fingerprints(&db, include_localized, version)?,
        include_localized,
        stats,
    })
}

fn reference(db: &Transaction<'_>, table: &str, code: &str) -> Result<i64> {
    // Table names are internal constants; codes are always bound parameters.
    let existing = db
        .query_row(
            &format!("SELECT id FROM {table} WHERE code=?1"),
            [code],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let id = next_reference_id(db, table)?;
    db.execute(
        &format!("INSERT INTO {table}(id,code) VALUES (?1,?2)"),
        params![id, code],
    )?;
    Ok(id)
}

fn next_reference_id(db: &Transaction<'_>, table: &str) -> Result<i64> {
    let id: i64 = db.query_row(
        &format!("SELECT COALESCE(MAX(id),0)+1 FROM {table}"),
        [],
        |r| r.get(0),
    )?;
    if id > u16::MAX as i64 {
        return Err(invalid(format!("{table} exceeds 16-bit reference capacity")).into());
    }
    Ok(id)
}

fn insert_item(db: &Transaction<'_>, item: &Item<'_>) -> Result<()> {
    let value = serde_json::to_value(item)?;
    let object = value
        .as_object()
        .ok_or_else(|| invalid("invalid normalized item"))?;
    let columns = object
        .keys()
        .map(|key| format!("\"{key}\""))
        .collect::<Vec<_>>()
        .join(",");
    let placeholders = vec!["?"; object.len()].join(",");
    let values: Vec<rusqlite::types::Value> = object
        .values()
        .map(|v| match v {
            Value::Null => Ok(rusqlite::types::Value::Null),
            Value::Bool(b) => Ok(rusqlite::types::Value::Integer(i64::from(*b))),
            Value::Number(n) => n
                .as_i64()
                .map(rusqlite::types::Value::Integer)
                .ok_or_else(|| invalid("item number exceeds SQLite integer")),
            Value::String(s) => Ok(rusqlite::types::Value::Text(s.clone())),
            _ => Err(invalid("invalid normalized item SQL value")),
        })
        .collect::<std::io::Result<_>>()?;
    db.prepare(&format!(
        "INSERT INTO items ({columns}) VALUES ({placeholders})"
    ))?
    .execute(params_from_iter(values))?;
    Ok(())
}

struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// French is the default selected locale; callers can explicitly select another
/// locale with export_localized. A locale switch alone is not an item revision.
pub fn export(
    probe: &Path,
    english: &Path,
    localized: &Path,
    manifest: &Path,
    output: &Path,
) -> Result<Summary> {
    export_localized(probe, english, localized, "fr-FR", manifest, output)
}

pub fn export_localized(
    probe: &Path,
    english: &Path,
    localized: &Path,
    locale: &str,
    manifest: &Path,
    output: &Path,
) -> Result<Summary> {
    if locale.is_empty()
        || !locale
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err(invalid("invalid item localization locale").into());
    }
    if output.exists() {
        let destination = output.canonicalize()?;
        for source in [probe, english, localized, manifest] {
            if source.canonicalize()? == destination {
                return Err(invalid("item database must not replace its source").into());
            }
        }
    }
    let original = output.exists().then(|| read(output)).transpose()?;
    let previous = previous(output, locale)?;
    let probe_bytes = read(probe)?;
    let probe: TableProbe = serde_json::from_slice(&probe_bytes)?;
    if probe.row_struct != "ItemTableItem"
        || probe.layout.table_version != 13
        || probe.layout.rows.is_empty()
    {
        return Err(invalid("expected a nonempty Item v13 structural probe").into());
    }
    for hash in [&probe.mapping_sha256, &probe.payload_sha256] {
        if !hex::decode(hash).is_ok_and(|bytes| bytes.len() == 32) {
            return Err(invalid("invalid item provenance hash").into());
        }
    }
    let steam = discovery::steam_manifest(manifest)?;
    if steam.app_id != "3393110" {
        return Err(invalid("expected AION 2 Steam manifest").into());
    }
    let build_number = steam.build_id.parse::<u64>()?;
    if build_number == 0
        || previous.versions.iter().any(|(_, build)| {
            build
                .parse::<u64>()
                .map_or(true, |value| value > build_number)
        })
    {
        return Err(invalid("invalid or older item client build").into());
    }
    let english_bytes = read(english)?;
    let localized_bytes = read(localized)?;
    let english: Localization = serde_json::from_slice(&english_bytes)?;
    let localized: Localization = serde_json::from_slice(&localized_bytes)?;
    if english.namespace != "AION2" || localized.namespace != "AION2" {
        return Err(invalid("unexpected item localization namespace").into());
    }
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|e| std::io::Error::other(e.to_string()))?;
    let temporary = Temporary(parent.join(format!(".items-{}.sqlite3", hex::encode(random))));
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary.0)?;
    let mut connection = Connection::open(&temporary.0)?;
    connection.execute_batch(include_str!("items.sql"))?;
    let transaction = connection.transaction()?;
    transaction.execute("INSERT INTO localization_language VALUES (1,?1)", [locale])?;
    for (id, build) in &previous.versions {
        transaction.execute(
            "INSERT INTO client_version VALUES (?1,?2)",
            params![id, build],
        )?;
    }
    let existing_version: Option<i64> = transaction
        .query_row(
            "SELECT id FROM client_version WHERE steam_build=?1",
            [&steam.build_id],
            |r| r.get(0),
        )
        .optional()?;
    let version_id = if let Some(id) = existing_version {
        id
    } else {
        let id = next_reference_id(&transaction, "client_version")?;
        transaction.execute(
            "INSERT INTO client_version VALUES (?1,?2)",
            params![id, steam.build_id],
        )?;
        id
    };
    for stat in &previous.stats {
        transaction.execute(
            "INSERT INTO stats VALUES (?1,?2,?3,?4)",
            params![stat.id, stat.code, stat.name_en, stat.name_localized],
        )?;
    }
    let mut summary = Summary {
        steam_build: steam.build_id,
        localized_locale: locale.into(),
        source_sha256: BTreeMap::from([
            ("item_probe".into(), sha(&probe_bytes)),
            ("english".into(), sha(&english_bytes)),
            ("localized".into(), sha(&localized_bytes)),
            ("steam_manifest".into(), steam.sha256),
            ("mapping".into(), probe.mapping_sha256),
            ("payload".into(), probe.payload_sha256),
        ]),
        ..Summary::default()
    };
    let mut rows: Vec<_> = probe.layout.rows.iter().collect();
    rows.sort_by_key(|row| row["ID"]["Value"].as_i64());
    for row in rows {
        let mut normalized = normalize(row, &english, &localized)?;
        normalized.item.kind_id = reference(&transaction, "item_kinds", normalized.kind)?;
        normalized.item.exchange_type_id =
            reference(&transaction, "exchange_types", normalized.exchange_type)?;
        let family_id = reference(&transaction, "item_families", normalized.family)?;
        let existing_category: Option<i64> = transaction
            .query_row(
                "SELECT id FROM item_categories WHERE family_id=?1 AND code=?2",
                params![family_id, normalized.category],
                |r| r.get(0),
            )
            .optional()?;
        normalized.item.category_id = if let Some(id) = existing_category {
            id
        } else {
            let id = next_reference_id(&transaction, "item_categories")?;
            transaction.execute(
                "INSERT INTO item_categories VALUES (?1,?2,?3)",
                params![id, family_id, normalized.category],
            )?;
            id
        };
        normalized.item.armor_type_id = normalized
            .armor_type
            .map(|code| reference(&transaction, "armor_types", code))
            .transpose()?;
        normalized.item.cooldown_id = if let Some(value) = normalized.cooldown {
            let existing: Option<i64> = transaction
                .query_row("SELECT id FROM cooldowns WHERE value=?1", [value], |r| {
                    r.get(0)
                })
                .optional()?;
            Some(if let Some(id) = existing {
                id
            } else {
                let id = next_reference_id(&transaction, "cooldowns")?;
                transaction.execute("INSERT INTO cooldowns VALUES (?1,?2)", params![id, value])?;
                id
            })
        } else {
            None
        };
        let rarity: Option<i64> = transaction
            .query_row(
                "SELECT id FROM rarities WHERE rarity_source=?1",
                [normalized.rarity_source],
                |r| r.get(0),
            )
            .optional()?;
        normalized.item.rarity_id = if let Some(id) = rarity {
            id
        } else {
            let id = next_reference_id(&transaction, "rarities")?;
            transaction.execute(
                "INSERT INTO rarities VALUES (?1,?2,?3)",
                params![id, normalized.rarity, normalized.rarity_source],
            )?;
            id
        };
        let currency: Option<i64> = transaction
            .query_row(
                "SELECT id FROM sell_currencies WHERE sell_currency_type=?1",
                [normalized.currency_type],
                |r| r.get(0),
            )
            .optional()?;
        normalized.item.sell_currency_id = if let Some(id) = currency {
            id
        } else {
            let id = next_reference_id(&transaction, "sell_currencies")?;
            transaction.execute(
                "INSERT INTO sell_currencies VALUES (?1,?2,?3)",
                params![id, normalized.currency, normalized.currency_type],
            )?;
            id
        };
        normalized.item.race_id = normalized
            .race
            .map(|code| reference(&transaction, "races", code))
            .transpose()?;
        let item = &mut normalized.item;
        item.client_version_id = version_id;
        insert_item(&transaction, item)?;
        for (code, value) in normalized.flags {
            let flag_id = reference(&transaction, "flags", &code)?;
            if value {
                transaction.execute(
                    "INSERT INTO item_flags VALUES (?1,?2)",
                    params![item.id, flag_id],
                )?;
            }
        }
        for (index, value) in normalized.classes.iter().enumerate() {
            if *value {
                transaction.execute(
                    "INSERT INTO item_class_permissions VALUES (?1,?2)",
                    params![item.id, index as i64],
                )?;
            }
        }
        let variant = &row["$variant"];
        if normalized.family == "Equip" {
            let stats = variant
                .get("MainStats")
                .ok_or_else(|| invalid("missing main stats"))?;
            for (position, stat) in array(stats, "Entries")?.iter().enumerate() {
                let stat_id = reference(&transaction, "stats", text(stat, "Key")?)?;
                transaction.execute(
                    "INSERT INTO item_main_stats VALUES (?1,?2,?3,?4)",
                    params![item.id, position as i64, stat_id, integer(stat, "Value")?],
                )?;
                summary.main_stats += 1;
            }
            for skin in array(variant, "ExtractionSkins")? {
                let skin = skin
                    .as_str()
                    .ok_or_else(|| invalid("invalid extraction skin key"))?;
                summary.extraction_skins += transaction.execute(
                    "INSERT OR IGNORE INTO item_extraction_skins VALUES (?1,?2)",
                    params![item.id, skin],
                )?;
            }
            for set in array(variant, "SetNames")? {
                let set = set
                    .as_str()
                    .ok_or_else(|| invalid("invalid item set key"))?;
                let set_id = reference(&transaction, "sets", set)?;
                transaction.execute(
                    "INSERT OR IGNORE INTO item_sets VALUES (?1,?2)",
                    params![item.id, set_id],
                )?;
            }
        }
        summary.names_en += usize::from(item.name_en.is_some());
        summary.names_localized += usize::from(item.name_localized.is_some());
        summary.descriptions_en += usize::from(item.description_en.is_some());
        summary.descriptions_localized += usize::from(item.description_localized.is_some());
        summary.items += 1;
    }
    for (id, fingerprint) in fingerprints(&transaction, previous.include_localized, 4)? {
        if previous.fingerprints.get(&id) == Some(&fingerprint) {
            let original_version = previous
                .item_versions
                .get(&id)
                .ok_or_else(|| invalid("missing previous item version"))?;
            transaction.execute(
                "UPDATE items SET client_version_id=?1 WHERE id=?2",
                params![original_version, id],
            )?;
            summary.unchanged_items += 1;
        } else if previous.item_versions.contains_key(&id) {
            summary.modified_items += 1;
        } else {
            summary.new_items += 1;
        }
    }
    transaction.commit()?;
    check_integrity(&connection)?;
    drop(connection);
    let destination = if output.parent().is_some_and(|p| !p.as_os_str().is_empty()) {
        output.to_path_buf()
    } else {
        Path::new(".").join(output)
    };
    if original.is_none() && destination.exists() {
        return Err(invalid("item database appeared during export").into());
    }
    crate::engine_ini::storage::atomic_replace(
        &destination,
        &fs::read(&temporary.0)?,
        original.as_deref(),
    )?;
    Ok(summary)
}
