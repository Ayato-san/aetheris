//! Item projection builder. Build comparisons belong to the unified catalogue export.
use super::{assets::Localization, binary::invalid, catalog::Result, discovery, usmap::TableProbe};
use rusqlite::{Connection, OptionalExtension, Transaction, params, params_from_iter};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, Serialize)]
pub struct Summary {
    pub items: usize,
    pub names_en: usize,
    pub names_localized: usize,
    pub descriptions_en: usize,
    pub descriptions_localized: usize,
    pub main_stats: usize,
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
    rarity_id: i64,
    sell_currency_id: i64,
    race_id: Option<i64>,
    name_en: Option<&'a str>,
    name_localized: Option<&'a str>,
    description_en: Option<&'a str>,
    description_localized: Option<&'a str>,
    item_level: i64,
    sell_price: i64,
    magic_stone_slots: Option<i64>,
    godstone_slots: Option<i64>,
}

struct Normalized<'a> {
    item: Item<'a>,
    family: &'a str,
    category: &'a str,
    armor_type: Option<&'a str>,
    rarity: &'a str,
    rarity_source: &'a str,
    currency: &'a str,
    currency_type: &'a str,
    race: Option<&'a str>,
    flags: BTreeMap<String, bool>,
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
fn permissions(row: &Value, variant: &Value) -> Result<BTreeMap<String, bool>> {
    let mut flags = BTreeMap::new();
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
            if key != "CanUseClassList" {
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
    Ok(flags)
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
    let flags = permissions(row, variant)?;
    Ok(Normalized {
        family,
        category,
        armor_type,
        rarity,
        rarity_source: text(row, "ItemGrade")?,
        currency: enum_value(row, "SellCurrencyType", "ECurrencyCategory::")?,
        currency_type: text(row, "SellCurrencyType")?,
        race: (race != "None").then_some(race),
        flags,
        item: Item {
            id,
            client_version_id: 0,
            category_id: 0,
            armor_type_id: None,
            rarity_id: 0,
            sell_currency_id: 0,
            race_id: None,
            name_en: translation(english, name_key),
            name_localized: translation(localized, name_key),
            description_en: translation(english, description_key),
            description_localized: translation(localized, description_key),
            item_level: nonnegative(row, "ItemLevel")?,
            sell_price: nonnegative(row, "SellPrice")?,
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

fn check_integrity(db: &Connection) -> Result<()> {
    let integrity: String = db.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" || db.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(invalid("invalid item SQLite integrity or foreign keys").into());
    }
    Ok(())
}

/// Compare natural reference values, never surrogate IDs or modification IDs.
pub(crate) fn fingerprints(
    db: &Connection,
    include_localized: bool,
) -> Result<BTreeMap<i64, String>> {
    let queries = [
        "SELECT i.id,f.code,c.code,a.code,i.name_en,i.description_en,
        CASE WHEN ?1 THEN i.name_localized END,CASE WHEN ?1 THEN i.description_localized END,
        i.item_level,r.rarity,r.rarity_source,i.sell_price,m.code,COALESCE(e.code,'None'),i.magic_stone_slots,i.godstone_slots
        FROM items i
        JOIN item_categories c ON c.id=i.category_id JOIN item_families f ON f.id=c.family_id
        JOIN rarities r ON r.id=i.rarity_id JOIN sell_currencies m ON m.id=i.sell_currency_id
        LEFT JOIN races e ON e.id=i.race_id LEFT JOIN armor_types a ON a.id=i.armor_type_id
        ORDER BY i.id",
        "SELECT v.item_id,f.code FROM item_flags v JOIN flags f ON f.id=v.flag_id ORDER BY v.item_id,f.code",
        "SELECT v.item_id,v.position,s.code,v.value FROM item_main_stats v JOIN stats s ON s.id=v.stat_id ORDER BY v.item_id,v.position",
        "SELECT v.item_id,s.code FROM item_sets v JOIN sets s ON s.id=v.set_id ORDER BY v.item_id,s.code"
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
    if id > i16::MAX as i64 {
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
    let original = output
        .exists()
        .then(|| super::target_export::read(output))
        .transpose()?;
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
    if build_number == 0 {
        return Err(invalid("invalid item client build").into());
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
    // Validate the complete staged graph after loading both families. Some
    // skill definitions reference each other and are copied in table order.
    connection.execute_batch("PRAGMA foreign_keys=OFF")?;
    let transaction = connection.transaction()?;
    transaction.execute("INSERT INTO localization_language VALUES (1,?1)", [locale])?;
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
    transaction.commit()?;
    connection.execute_batch("PRAGMA foreign_keys=ON")?;
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
