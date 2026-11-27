//! Read-only catalogue inspection for the development UI. No connection crosses
//! the worker boundary and no inspection creates or upgrades a database.
#[path = "display_numbers.rs"]
pub mod display_numbers;

use super::{binary::invalid, catalog::Result, target_export};
use rusqlite::{Connection, OpenFlags, types::Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Duration,
};

pub const PAGE_SIZE: usize = 100;
#[path = "sprite_card.rs"]
mod sprite_card;
pub use sprite_card::SpriteCard;
pub fn sprite(path: &Path, id: i64) -> Result<SpriteCard> {
    let mut db = open(path)?;
    let tx = db.transaction()?;
    sprite_card::load(&tx, id)
}
#[path = "item_card.rs"]
mod item_card;
#[path = "item_relation_card.rs"]
mod item_relation_card;
pub use item_card::{ItemAttackRange, ItemCard, ItemEnchantment, ItemStat, SetBonus};
pub use item_relation_card::{DungeonReward, ItemRelation, RELATION_PAGE_SIZE, RelationPage};
pub fn item_relations(path: &Path, id: i64, uses: bool, offset: usize) -> Result<RelationPage> {
    let mut db = open(path)?;
    let tx = db.transaction()?;
    item_relation_card::page(&tx, id, if uses { "use" } else { "source" }, offset)
}
pub fn item(path: &Path, id: i64) -> Result<ItemCard> {
    let mut db = open(path)?;
    let tx = db.transaction()?;
    item_card::load(&tx, id)
}
#[path = "definition_card.rs"]
mod definition_card;
pub use definition_card::{CardField, CardLevel, CardSection, DefinitionCard};
#[path = "resource_card.rs"]
mod resource_card;
pub use resource_card::ResourceFamily;
pub fn definition(path: &Path, family: Family, id: i64) -> Result<DefinitionCard> {
    let mut db = open(path)?;
    let tx = db.transaction()?;
    let mut card = if let Family::Resources(resource) = family {
        resource_card::load(&tx, resource, id)?
    } else {
        definition_card::load(&tx, family, id)?
    };
    let locale: String = tx.query_row(
        "SELECT locale FROM catalogue_manifest WHERE id=1",
        [],
        |r| r.get(0),
    )?;
    fn format_fields(card: &mut DefinitionCard, locale: &str) {
        for level in &mut card.levels {
            for field in level
                .sections
                .iter_mut()
                .flat_map(|section| &mut section.fields)
            {
                if field.numeric {
                    let raw = field.value.clone();
                    field.value = display_numbers::number(
                        "en-US",
                        &raw,
                        display_numbers::Presentation::Decimal(12),
                    );
                    field.value_localized = Some(display_numbers::number(
                        locale,
                        &raw,
                        display_numbers::Presentation::Decimal(12),
                    ));
                }
            }
        }
        for related in &mut card.related {
            format_fields(related, locale);
        }
    }
    format_fields(&mut card, &locale);
    Ok(card)
}
const RELATION_LIMIT: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Items,
    Skills,
    Abnormals,
    Sprites,
    Resources(ResourceFamily),
}
impl Family {
    pub const ALL: [Self; 4] = [Self::Items, Self::Skills, Self::Abnormals, Self::Sprites];
    pub fn label(self) -> &'static str {
        match self {
            Self::Items => "Items",
            Self::Skills => "Skills",
            Self::Abnormals => "Altérations d’état",
            Self::Sprites => "Sprites UI",
            Self::Resources(family) => family.label(),
        }
    }
    fn table(self) -> &'static str {
        match self {
            Self::Items => "items",
            Self::Skills => "skills",
            Self::Abnormals => "skill_abnormals",
            Self::Sprites => "ui_sprites",
            Self::Resources(family) => family.table(),
        }
    }
    fn evidence_family(self) -> &'static str {
        match self {
            Self::Abnormals => "abnormals",
            Self::Resources(family) => family.key(),
            _ => self.table(),
        }
    }
    fn source(self) -> &'static str {
        match self {
            Self::Items => "Item",
            Self::Skills => "Skill",
            Self::Abnormals => "SkillAbnormal",
            Self::Sprites => "ui_sprites",
            Self::Resources(family) => family.source(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub id: i64,
    pub name: String,
    pub name_en: String,
}
#[derive(Debug)]
pub struct Listing {
    pub summary: String,
    pub summary_en: String,
    pub locale: String,
    pub total: usize,
    pub entries: Vec<Entry>,
    pub coverage: Vec<String>,
}

fn open(path: &Path) -> Result<Connection> {
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    db.busy_timeout(Duration::from_secs(2))?;
    db.execute_batch("PRAGMA query_only=ON; PRAGMA trusted_schema=OFF;")?;
    if db.query_row("PRAGMA application_id", [], |r| r.get::<_, i64>(0))? != 0x41454954
        || db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))? != 17
    {
        return Err(invalid(
            "Catalogue incompatible : régénérez data.sqlite avec export-catalogue.",
        )
        .into());
    }
    Ok(db)
}

pub fn list(path: &Path, family: Family, search: &str, offset: usize) -> Result<Listing> {
    let mut db = open(path)?;
    let tx = db.transaction()?;
    let (build,locale,items,skills,abnormals): (String,String,i64,i64,i64) = tx.query_row(
        "SELECT v.steam_build,m.locale,(SELECT count(*) FROM items),(SELECT count(*) FROM skills),(SELECT count(*) FROM skill_abnormals) FROM catalogue_manifest m JOIN client_version v ON v.id=m.build_id WHERE m.id=1",
        [],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))?;
    let summary_for = |language: &str| {
        format!(
            "Build {build} · {locale} · {} items · {} skills · {} états",
            display_numbers::number(language, items, display_numbers::Presentation::Integer),
            display_numbers::number(language, skills, display_numbers::Presentation::Integer),
            display_numbers::number(language, abnormals, display_numbers::Presentation::Integer)
        )
    };
    let summary = summary_for(&locale);
    let summary_en = summary_for("en-US");
    // SQLite lower() is ASCII-only. Stream just the names on the worker so
    // localized searches also fold É/é, without loading the whole catalogue.
    let search = search.trim().to_lowercase();
    let searched_id = search.parse::<i64>().ok();
    let mut total = 0;
    let mut entries = Vec::new();
    let sql = if let Family::Resources(resource) = family {
        resource_card::listing_query(&tx, resource)?
    } else if family == Family::Sprites {
        "SELECT s.id,s.asset_path,s.asset_path,COALESCE(a.names,'') FROM ui_sprites s LEFT JOIN (SELECT sprite_id,group_concat(table_name || '/' || row_key,' ') AS names FROM ui_sprite_aliases GROUP BY sprite_id) a ON a.sprite_id=s.id ORDER BY s.id".into()
    } else {
        format!(
            "SELECT id,COALESCE(name_localized,name_en,'Sans nom'),COALESCE(name_en,'—'),'' FROM {} ORDER BY id",
            family.table()
        )
    };
    let mut query = tx.prepare(&sql)?;
    for entry in query.query_map([], |r| {
        Ok((
            Entry {
                id: r.get(0)?,
                name: r.get(1)?,
                name_en: r.get(2)?,
            },
            r.get::<_, String>(3)?,
        ))
    })? {
        let (mut entry, aliases) = entry?;
        if search.is_empty()
            || searched_id == Some(entry.id)
            || entry.name.to_lowercase().contains(&search)
            || entry.name_en.to_lowercase().contains(&search)
            || aliases.to_lowercase().contains(&search)
        {
            if total >= offset && entries.len() < PAGE_SIZE {
                if family == Family::Sprites {
                    entry.name = entry
                        .name
                        .rsplit('/')
                        .next()
                        .unwrap_or(&entry.name)
                        .split('.')
                        .next()
                        .unwrap_or(&entry.name)
                        .to_owned();
                }
                entries.push(entry);
            }
            total += 1;
        }
    }
    let coverage = tx
        .prepare("SELECT subject,state,diagnostic FROM catalogue_coverage ORDER BY subject")?
        .query_map([], |r| {
            Ok(format!(
                "{} · {}\n{}",
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(Listing {
        summary,
        summary_en,
        locale,
        total,
        entries,
        coverage,
    })
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
type Record = BTreeMap<String, Value>;
fn columns(db: &Connection, table: &str) -> Result<Vec<(String, String)>> {
    Ok(db
        .prepare(&format!("PRAGMA table_info({})", quote(table)))?
        .query_map([], |r| Ok((r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?)
}
fn records(
    db: &Connection,
    table: &str,
    predicate: &str,
    args: &[Value],
    limit: usize,
) -> Result<Vec<Record>> {
    let cols = columns(db, table)?;
    // Never fetch image bytes or compressed source archives to format a field.
    let projection = cols
        .iter()
        .map(|(name, _)| {
            let q = quote(name);
            format!(
                "CASE WHEN typeof({q})='blob' THEN '[BLOB '||length({q})||' octets]' ELSE {q} END"
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let mut query = db.prepare(&format!(
        "SELECT {projection} FROM {} WHERE {predicate} LIMIT {limit}",
        quote(table)
    ))?;
    Ok(query
        .query_map(rusqlite::params_from_iter(args), |row| {
            cols.iter()
                .enumerate()
                .map(|(i, (name, _))| Ok((name.clone(), row.get(i)?)))
                .collect()
        })?
        .collect::<rusqlite::Result<_>>()?)
}
#[derive(Debug)]
struct ForeignKey {
    table: String,
    fields: Vec<(String, String)>,
}
fn foreign_keys(db: &Connection, table: &str) -> Result<Vec<ForeignKey>> {
    let mut groups: BTreeMap<i64, ForeignKey> = BTreeMap::new();
    let mut stmt = db.prepare(&format!("PRAGMA foreign_key_list({})", quote(table)))?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;
    for row in rows {
        let (id, table, from, to) = row?;
        groups
            .entry(id)
            .or_insert_with(|| ForeignKey {
                table,
                fields: vec![],
            })
            .fields
            .push((from, to));
    }
    Ok(groups.into_values().collect())
}
fn display(value: &Value, ty: &str) -> String {
    match value {
        Value::Null => "NULL · absent / inconnu / non applicable (voir la source)".into(),
        Value::Integer(v) if ty.eq_ignore_ascii_case("BOOLEAN") => match v {
            0 => "false".into(),
            1 => "true".into(),
            _ => v.to_string(),
        },
        Value::Integer(v) => v.to_string(),
        Value::Real(v) => v.to_string(),
        Value::Text(v) => v.clone(),
        Value::Blob(v) => format!("[BLOB {} octets]", v.len()),
    }
}
fn describe(
    db: &Connection,
    table: &str,
    row: &Record,
    lines: &mut Vec<String>,
    indent: &str,
    resolve: bool,
) -> Result<()> {
    for (name, ty) in columns(db, table)? {
        if let Some(value) = row.get(&name) {
            lines.push(format!("{indent}{name} : {}", display(value, &ty)));
        }
    }
    if resolve {
        for fk in foreign_keys(db, table)? {
            let args: Vec<_> = fk
                .fields
                .iter()
                .map(|(from, _)| row.get(from).cloned().unwrap_or(Value::Null))
                .collect();
            if args.iter().any(|v| matches!(v, Value::Null)) {
                continue;
            }
            let predicate = fk
                .fields
                .iter()
                .map(|(_, to)| format!("{}=?", quote(to)))
                .collect::<Vec<_>>()
                .join(" AND ");
            for target in records(db, &fk.table, &predicate, &args, 1)? {
                // Source metadata is useful; its compressed payload is never expanded here.
                lines.push(format!("{indent}↳ {}", fk.table));
                describe(db, &fk.table, &target, lines, &format!("{indent}  "), false)?;
            }
        }
    }
    Ok(())
}
fn section(
    db: &Connection,
    table: &str,
    predicate: &str,
    args: &[Value],
    lines: &mut Vec<String>,
) -> Result<Vec<Record>> {
    let rows = records(db, table, predicate, args, RELATION_LIMIT + 1)?;
    if !rows.is_empty() {
        lines.push(String::new());
        lines.push(format!("━━ {table} ━━"));
        for (i, row) in rows.iter().take(RELATION_LIMIT).enumerate() {
            lines.push(format!("  [{}]", i + 1));
            describe(db, table, row, lines, "  ", true)?;
        }
        if rows.len() > RELATION_LIMIT {
            lines.push(format!("… affichage limité à {RELATION_LIMIT} relations ; la source complète reste dans SQLite."));
        }
    }
    Ok(rows.into_iter().take(RELATION_LIMIT).collect())
}

/// Product fields, resolved dictionaries, incoming relations and effect values.
/// Sources are opt-in: decompressing them can be substantially more expensive.
pub fn detail(path: &Path, family: Family, id: i64, sources: bool) -> Result<Vec<String>> {
    let mut db = open(path)?;
    let db = db.transaction()?;
    if family == Family::Sprites {
        let card = sprite_card::load(&db, id)?;
        let mut lines = vec![
            format!("Sprite #{} · {}", card.id, card.asset_path),
            format!("{} × {} px · {}", card.width, card.height, card.image_state),
        ];
        lines.extend(
            card.aliases
                .into_iter()
                .map(|(table, key)| format!("{table} · {key}")),
        );
        return Ok(lines);
    }
    let predicate = match family {
        Family::Resources(resource) => resource.predicate(),
        _ => "id=?".into(),
    };
    let row = records(&db, family.table(), &predicate, &[Value::Integer(id)], 1)?
        .pop()
        .ok_or_else(|| invalid("Entrée absente du catalogue ; actualisez la liste."))?;
    let mut lines = vec![format!("{} · #{id}", family.label())];
    if family == Family::Items {
        lines.push("Permissions : item_flags = true ; item_false_flags = false ; absentes des deux = inconnues.".into());
    }
    lines.push("Valeurs et unités conservées telles qu’extraites ; une référence résolue ne valide pas son interprétation.".into());
    describe(&db, family.table(), &row, &mut lines, "", true)?;
    let tables = db
        .prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut effects = Vec::new();
    let mut source_links: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    for table in tables {
        for fk in foreign_keys(&db, &table)? {
            if fk.table != family.table() {
                continue;
            }
            let args: Vec<_> = fk
                .fields
                .iter()
                .map(|(_, to)| row.get(to).cloned().unwrap_or(Value::Null))
                .collect();
            let predicate = fk
                .fields
                .iter()
                .map(|(from, _)| format!("{}=?", quote(from)))
                .collect::<Vec<_>>()
                .join(" AND ");
            let related = section(&db, &table, &predicate, &args, &mut lines)?;
            if sources {
                for row in &related {
                    if let (Some(Value::Text(name)), Some(Value::Integer(pos))) =
                        (row.get("source_name"), row.get("source_row"))
                        && let Ok(pos) = usize::try_from(*pos)
                    {
                        source_links.entry(name.clone()).or_default().insert(pos);
                    }
                }
            }
            if table == "skill_effect_links" {
                effects.extend(related);
            } else if table == "item_sets" {
                for set in related {
                    for name in [
                        "item_set_definitions",
                        "item_set_members",
                        "item_set_bonuses",
                    ] {
                        section(&db, name, "set_id=?", &[set["set_id"].clone()], &mut lines)?;
                    }
                }
            }
        }
    }
    for table in [
        "catalogue_entries",
        "catalogue_texts",
        "catalogue_entity_icons",
    ] {
        section(
            &db,
            table,
            "family=? AND source_id=?",
            &[
                Value::Text(family.evidence_family().into()),
                Value::Integer(id),
            ],
            &mut lines,
        )?;
    }
    section(
        &db,
        "catalogue_comparisons",
        "family=?",
        &[Value::Text(family.evidence_family().into())],
        &mut lines,
    )?;
    if family == Family::Abnormals {
        effects.extend(section(
            &db,
            "skill_effects",
            "abnormal_id=?",
            &[Value::Integer(id)],
            &mut lines,
        )?);
        section(
            &db,
            "skill_abnormal_links",
            "abnormal_id=?",
            &[Value::Integer(id)],
            &mut lines,
        )?;
    }
    for effect in effects {
        let kind = effect["kind_id"].clone();
        let eid = effect
            .get("effect_id")
            .or_else(|| effect.get("id"))
            .unwrap()
            .clone();
        section(
            &db,
            "skill_effect_values",
            "kind_id=? AND effect_id=?",
            &[kind.clone(), eid.clone()],
            &mut lines,
        )?;
        let code = records(
            &db,
            "skill_effect_kinds",
            "id=?",
            std::slice::from_ref(&kind),
            1,
        )?
        .first()
        .and_then(|r| r.get("code"))
        .and_then(|v| {
            if let Value::Text(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });
        if code.as_deref() == Some("se") {
            section(
                &db,
                "skill_conditions",
                "effect_id=?",
                std::slice::from_ref(&eid),
                &mut lines,
            )?;
        }
        if let Some(code) = &code {
            let applications = section(
                &db,
                "skill_abnormal_links",
                "effect_kind=? AND effect_id=?",
                &[Value::Text(code.clone()), eid.clone()],
                &mut lines,
            )?;
            for application in applications {
                section(
                    &db,
                    "skill_abnormals",
                    "id=?",
                    &[application["abnormal_id"].clone()],
                    &mut lines,
                )?;
            }
        }
        let level_kind = match code.as_deref() {
            Some("se") => "se_level",
            Some("abe") => "abe_level",
            _ => continue,
        };
        for definition in records(&db, "skill_effects", "kind_id=? AND id=?", &[kind, eid], 1)? {
            if let Some(Value::Text(group)) = definition.get("level_group") {
                let levels = section(
                    &db,
                    "skill_effects",
                    "level_group=? AND level IS NOT NULL AND kind_id=(SELECT id FROM skill_effect_kinds WHERE code=?)",
                    &[Value::Text(group.clone()), Value::Text(level_kind.into())],
                    &mut lines,
                )?;
                for level in levels {
                    section(
                        &db,
                        "skill_effect_values",
                        "kind_id=? AND effect_id=?",
                        &[level["kind_id"].clone(), level["id"].clone()],
                        &mut lines,
                    )?;
                }
            }
        }
    }
    if sources && let Family::Resources(resource) = family {
        let links = records(
            &db,
            "catalogue_entity_sources",
            "family=? AND source_id=?",
            &[Value::Text(resource.key().into()), Value::Integer(id)],
            RELATION_LIMIT + 1,
        )?;
        if links.len() > RELATION_LIMIT {
            lines.push(format!("Sources liées limitées aux {RELATION_LIMIT} premières lignes ; les autres restent dans SQLite."));
        }
        for link in links.into_iter().take(RELATION_LIMIT) {
            if let (Some(Value::Text(name)), Some(Value::Integer(pos))) =
                (link.get("source_name"), link.get("source_row"))
                && let Ok(pos) = usize::try_from(*pos)
            {
                source_links.entry(name.clone()).or_default().insert(pos);
            }
        }
    }
    if sources {
        if matches!(family, Family::Resources(resource) if resource.key() == "loot_tables") {
            resource_card::loot_sources(&db, &row, &mut lines)?;
        }
        if !matches!(family, Family::Resources(_)) {
            lines.push(format!(
                "\n━━ Source complète {} · #{id} ━━",
                family.source()
            ));
            let rows = target_export::source_rows(&db, family.source())?;
            for source in rows
                .iter()
                .filter(|r| r["ID"].as_i64().or_else(|| r["ID"]["Value"].as_i64()) == Some(id))
            {
                lines.extend(
                    serde_json::to_string_pretty(source)?
                        .lines()
                        .map(str::to_owned),
                );
            }
        }
        for (name, positions) in source_links {
            let rows = target_export::source_rows(&db, &name)?;
            for pos in positions {
                lines.push(format!("\n━━ Source liée {name} · ligne {pos} ━━"));
                if let Some(row) = rows.get(pos) {
                    lines.extend(
                        serde_json::to_string_pretty(row)?
                            .lines()
                            .map(str::to_owned),
                    );
                }
            }
        }
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(path: &Path) {
        let db = Connection::open(path).unwrap();
        db.execute_batch(include_str!("items.sql")).unwrap();
        db.execute_batch(include_str!("skills.sql")).unwrap();
        db.execute_batch(include_str!("catalogue.sql")).unwrap();
        db.execute_batch("INSERT INTO client_version VALUES(1,'100'); INSERT INTO catalogue_manifest VALUES(1,1,'fr-FR','a','b','c','d',NULL,'unknown');
        INSERT INTO item_families VALUES(1,'Equip'); INSERT INTO item_categories VALUES(1,1,'Sword');
        INSERT INTO rarities VALUES(1,'Unique','EItemGrade::Unique'); INSERT INTO sell_currencies VALUES(1,'Gold','Gold');
        INSERT INTO flags VALUES(1,'CanSell'),(2,'CanLock');
        INSERT INTO items(id,client_version_id,category_id,rarity_id,sell_currency_id,item_level,sell_price,name_en,name_localized) VALUES(42,1,1,1,1,1,0,'Sword','Épée');
        INSERT INTO item_flags VALUES(42,1); INSERT INTO item_false_flags VALUES(42,2);
        INSERT INTO catalogue_coverage VALUES('items','partial','example');").unwrap();
        let data =
            lz4_flex::compress_prepend_size(br#"[{"ID":{"Value":42},"Unknown":"retained"}]"#);
        db.execute("INSERT INTO catalogue_sources VALUES('Item',1,'a','b','ItemTableItem',0,'decoded',1,?1)",[data]).unwrap();
    }
    #[test]
    fn sprite_browser_searches_aliases_and_reads_only_the_selected_image() {
        let path =
            std::env::temp_dir().join(format!("browser-sprites-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let db = Connection::open(&path).unwrap();
        let png = include_bytes!("../../resources/item-card/can-storage.png");
        db.execute("INSERT INTO catalogue_icons(id,reference,image,mime_type,image_state) VALUES(1,'Sprite_Storage',?1,'image/png','available')", [png.as_slice()]).unwrap();
        db.execute_batch("INSERT INTO ui_sprites VALUES(1,'/Game/UI/Storage.Storage',1,32,64);
            INSERT INTO ui_sprite_aliases VALUES('UI_ImageTable','Storage_Button',1),('UI_ImageTable_Composite','Shared_Storage',1);").unwrap();
        drop(db);
        let before = std::fs::read(&path).unwrap();
        for query in [
            "",
            "1",
            "storage",
            "shared_storage",
            "/game/ui",
            "ui_imagetable",
        ] {
            let results = list(&path, Family::Sprites, query, 0).unwrap();
            assert_eq!(results.total, 1, "{query}");
            assert_eq!(results.entries[0].name, "Storage");
        }
        assert_eq!(list(&path, Family::Sprites, "missing", 0).unwrap().total, 0);
        assert!(
            list(&path, Family::Sprites, "", 100)
                .unwrap()
                .entries
                .is_empty()
        );
        let card = sprite(&path, 1).unwrap();
        assert_eq!((card.width, card.height), (32, 64));
        assert_eq!(card.aliases.len(), 2);
        assert_eq!(card.image.as_deref(), Some(png.as_slice()));
        assert_eq!(card.mime.as_deref(), Some("image/png"));
        assert_eq!(before, std::fs::read(&path).unwrap());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn item_card_loads_the_sell_currency_image_independently_of_the_item_icon() {
        let path =
            std::env::temp_dir().join(format!("browser-currency-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let db = Connection::open(&path).unwrap();
        let png = include_bytes!("../../resources/item-card/can-storage.png");
        db.execute("INSERT INTO catalogue_icons(id,reference,image,mime_type,image_state) VALUES(1,'Currency_Goldbinding',?1,'image/png','available')", [png.as_slice()]).unwrap();
        db.execute_batch("INSERT INTO catalogue_entity_icons VALUES('sell_currencies',1,1); UPDATE items SET sell_price=250 WHERE id=42;").unwrap();
        drop(db);
        let before = std::fs::read(&path).unwrap();
        let card = item(&path, 42).unwrap();
        assert_eq!(card.sell_price, 250);
        assert_eq!(card.currency_icon.as_deref(), Some(png.as_slice()));
        assert_eq!(card.currency_icon_mime.as_deref(), Some("image/png"));
        assert!(card.icon.is_none());
        assert_eq!(before, std::fs::read(&path).unwrap());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn browser_searches_localized_names_and_exact_ids_and_resolves_relations_without_writes() {
        let path = std::env::temp_dir().join(format!("browser-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let before = std::fs::read(&path).unwrap();
        for query in ["Épée", "épée", "SWORD", "42"] {
            let data = list(&path, Family::Items, query, 0).unwrap();
            assert_eq!(data.total, 1);
            assert_eq!(data.entries[0].id, 42);
        }
        assert_eq!(list(&path, Family::Items, "%", 0).unwrap().total, 0);
        assert!(
            list(&path, Family::Items, "", 100)
                .unwrap()
                .entries
                .is_empty()
        );
        let lines = detail(&path, Family::Items, 42, true).unwrap().join("\n");
        for expected in [
            "Unique",
            "CanSell",
            "item_false_flags",
            "CanLock",
            "retained",
            "Épée",
        ] {
            assert!(lines.contains(expected), "missing {expected}");
        }
        assert_eq!(before, std::fs::read(&path).unwrap());
        let card = item(&path, 42).unwrap();
        assert_eq!(card.name_localized.as_deref(), Some("Épée"));
        assert_eq!(card.rarity, "Unique");
        assert!(card.required_level.is_none());
        assert!(card.stats.is_empty());
        assert!(card.icon.is_none());
        assert!(card.permissions.contains(&("CanLock".into(), false)));
        assert!(detail(&path, Family::Items, 99, false).is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(list(&path, Family::Items, "", 0).is_err());
        assert!(!path.exists());
    }
    #[test]
    fn item_rarity_labels_follow_all_client_grades_without_aliases() {
        let path =
            std::env::temp_dir().join(format!("browser-rarities-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let db = Connection::open(&path).unwrap();
        for (source, english, localized) in [
            ("Unique", "Unique", "Unique"),
            ("Legend", "Epic", "Épique"),
            ("Rare", "Rare", "Rare"),
            ("Common", "Common", "Général"),
            ("Special", "Special", "Spécial"),
            ("Epic", "Heroic", "Héroïque"),
        ] {
            db.execute(
                "INSERT INTO catalogue_labels VALUES('EItemGrade',?1,?2,?3,?4)",
                rusqlite::params![
                    source,
                    format!("Tooltip_Grade_{source}"),
                    english,
                    localized
                ],
            )
            .unwrap();
            // A stale display value must never override the source identity.
            db.execute(
                "UPDATE rarities SET rarity='wrong display value',rarity_source=?1",
                [format!("EItemGrade::{source}")],
            )
            .unwrap();
            let card = item(&path, 42).unwrap();
            assert_eq!(card.rarity, source);
            assert_eq!(card.rarity_label.text(true), english);
            assert_eq!(card.rarity_label.text(false), localized);
        }
        drop(db);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn item_terms_switch_languages_using_source_grades_and_english_fallback() {
        let path =
            std::env::temp_dir().join(format!("browser-languages-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let db = Connection::open(&path).unwrap();
        db.execute_batch("UPDATE item_categories SET code='Greatsword';
        INSERT INTO catalogue_labels VALUES
        ('EItemGrade','Unique','grade','Unique','Unique'),
        ('EItemGrade','Common','common','Common','Général'),
        ('EEquipCategory','Greatsword','weapon','Greatsword','Espadon'),
        ('EMiscCategory','Greatsword','wrong_family','Wrong category','Mauvaise catégorie'),
        ('EStat','WeaponFixingDamage','attack','Attack','ATQ');
        INSERT INTO stats(id,code,name_en,name_localized) VALUES(1,'EStat::WeaponAccuracy','Accuracy','Précision'),(2,'EStat::Critical','Critical Hit',NULL),
        (3,'EStat::WeaponMinDamage','Min Attack','ATQ min'),(4,'EStat::WeaponDamage','Max Attack','ATQ max');
        INSERT INTO item_main_stats VALUES(42,0,1,100),(42,1,2,150),(42,2,3,100),(42,3,4,200);").unwrap();
        let card = item(&path, 42).unwrap();
        assert_eq!(card.rarity_label.text(false), "Unique");
        assert_eq!(card.rarity_label.text(true), "Unique");
        assert_eq!(card.category.text(true), "Greatsword");
        assert_eq!(card.category.text(false), "Espadon");
        assert_eq!(card.stats[0].name.text(true), "Accuracy");
        assert_eq!(card.stats[0].name.text(false), "Précision");
        assert_eq!(card.stats[1].name.text(false), "Critical Hit");
        assert_eq!(card.stats[2].name.text(true), "Attack");
        assert_eq!(card.stats[2].name.text(false), "ATQ");
        assert_eq!(card.stats[2].value, "100 ~ 200");
        db.execute(
            "UPDATE rarities SET rarity='Common',rarity_source='EItemGrade::Common'",
            [],
        )
        .unwrap();
        let card = item(&path, 42).unwrap();
        assert_eq!(card.rarity_label.text(true), "Common");
        assert_eq!(card.rarity_label.text(false), "Général");
        drop(db);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn browser_reads_skill_evolutions_without_confusing_se_and_abe_groups() {
        let path =
            std::env::temp_dir().join(format!("browser-skills-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let db = Connection::open(&path).unwrap();
        db.execute_batch("INSERT INTO skills(id,client_version_id,name_en,max_level) VALUES(4003,1,'Example',2);
        INSERT INTO skill_effect_kinds VALUES(1,'se'),(2,'se_level'),(3,'abe_level');
        INSERT INTO skill_effects(kind_id,id,level_group) VALUES(1,55,'shared');
        INSERT INTO skill_effects(kind_id,id,level_group,level) VALUES(2,56,'shared',2),(3,57,'shared',2);
        INSERT INTO skill_effect_values VALUES(1,55,0,'base value'),(2,56,0,'level value'),(3,57,0,'unrelated abnormal');
        INSERT INTO skill_effect_links VALUES(4003,0,1,55,NULL);
        INSERT INTO skill_levels VALUES(4003,2,10,20,3,0);").unwrap();
        drop(db);
        let text = detail(&path, Family::Skills, 4003, false)
            .unwrap()
            .join("\n");
        assert!(text.contains("base value"));
        assert!(text.contains("level value"));
        assert!(text.contains("skill_points : 3"));
        assert!(!text.contains("unrelated abnormal"));
        std::fs::remove_file(&path).unwrap();
    }
    #[test]
    fn effect_cards_apply_level_aggro_overrides_including_zero() {
        let path =
            std::env::temp_dir().join(format!("browser-aggro-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let db = Connection::open(&path).unwrap();
        db.execute_batch("INSERT INTO skills(id,client_version_id,max_level) VALUES(1,1,3);
            INSERT INTO skill_effect_kinds VALUES(1,'se'),(2,'se_level');
            INSERT INTO skill_effects(kind_id,id,level_group,effect_type,aggro_absolute,aggro_ratio) VALUES(1,10,'levels','Damage',100,50);
            INSERT INTO skill_effects(kind_id,id,level_group,level,aggro_absolute,aggro_ratio) VALUES(2,20,'levels',2,200,0),(2,30,'levels',3,0,NULL);
            INSERT INTO skill_effect_links VALUES(1,0,1,10,NULL);").unwrap();
        drop(db);
        let card = definition(&path, Family::Skills, 1).unwrap();
        let effect = &card.related[0];
        let values = effect
            .levels
            .iter()
            .map(|l| {
                let fields = &l.sections[0].fields;
                let get = |name| {
                    fields
                        .iter()
                        .find(|f| f.name_en == name)
                        .unwrap()
                        .value
                        .clone()
                };
                (get("Aggro"), get("Aggro ratio"))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            vec![
                ("100".into(), "50".into()),
                ("200".into(), "0".into()),
                ("0".into(), "50".into())
            ]
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn definition_cards_resolve_each_level_and_keep_effect_families_separate_without_writes() {
        let path =
            std::env::temp_dir().join(format!("browser-definition-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let db = Connection::open(&path).unwrap();
        db.execute_batch("INSERT INTO skills(id,client_version_id,name_en,name_localized,description_en,description_localized,mp_cost,max_level) VALUES(4003,1,'Skill','Compétence','Power {se:55:effect_value01:none}','Puissance {se:55:effect_value01:none}',8,2);
        INSERT INTO skill_levels VALUES(4003,2,0,20,3,0);
        INSERT INTO skill_cast_times VALUES(4003,1,0,'casting',1500,NULL),(4003,2,0,'casting',0,NULL),(4003,1,0,'charge_minimum',1000,NULL);
        INSERT INTO skill_effect_kinds VALUES(1,'se'),(2,'se_level'),(3,'abe_level'),(4,'abe');
        INSERT INTO skill_effects(kind_id,id,level_group,effect_type) VALUES(1,55,'shared','Damage');
        INSERT INTO skill_effects(kind_id,id,level_group,level) VALUES(2,56,'shared',2),(3,57,'shared',2);
        INSERT INTO skill_effect_values VALUES(1,55,0,'10'),(1,55,1,'99'),(2,56,0,'20'),(3,57,0,'999');
        INSERT INTO skill_effect_links VALUES(4003,0,1,55,NULL);
        INSERT INTO skill_abnormals(id,name_en,name_localized,description_en,max_level) VALUES(100,'Status','Altération','Value {se:55:effect_value01:none}',2);
        INSERT INTO skill_effects(kind_id,id,abnormal_id,effect_type) VALUES(4,55,100,'Change');
        INSERT INTO skill_effect_values VALUES(4,55,0,'123');
        INSERT INTO skill_conditions(effect_id,kind,condition_id,type,expression,parameter,position,value) VALUES(55,'effect',0,'TargetHp',0,'value',0,'50');").unwrap();
        drop(db);
        let before = std::fs::read(&path).unwrap();
        let skill = definition(&path, Family::Skills, 4003).unwrap();
        assert_eq!(skill.name_localized.as_deref(), Some("Compétence"));
        assert_eq!(skill.levels[0].description_en.as_deref(), Some("Power 10"));
        assert_eq!(
            skill.levels[1].description_localized.as_deref(),
            Some("Puissance 20")
        );
        let costs = &skill.levels[1].sections[0].fields;
        assert!(
            costs
                .iter()
                .any(|f| f.name_en == "Cast time (source)" && f.value == "casting · 0 · 0")
        );
        assert!(
            costs
                .iter()
                .any(|f| f.name_en == "Cast time (source)"
                    && f.value == "charge_minimum · 0 · 1,000")
        );
        assert!(
            costs
                .iter()
                .any(|f| f.name_en == "MP cost" && f.value == "0")
        );
        assert!(
            costs
                .iter()
                .any(|f| f.name_en == "Stigma points" && f.value == "0")
        );
        assert_eq!(skill.related.len(), 1);
        assert_eq!(skill.related[0].tag, "SE · #55");
        assert_eq!(skill.related[0].levels[0].sections[1].fields.len(), 2);
        assert_eq!(skill.related[0].levels[1].sections[1].fields.len(), 1);
        for level in &skill.related[0].levels {
            assert_eq!(level.sections[2].fields.len(), 1);
            assert!(
                level.sections[2].fields[0]
                    .value
                    .contains("expression=false")
            );
            assert!(level.sections[2].fields[0].value.contains("50"));
        }
        assert_eq!(
            skill.related[0].levels[1].sections[1].fields[0].value,
            "1 · 20"
        );
        let status = definition(&path, Family::Abnormals, 100).unwrap();
        assert_eq!(status.levels[1].description_en.as_deref(), Some("Value 20"));
        assert_eq!(status.related[0].tag, "ABE · #55");
        assert!(status.related[0].levels[0].sections[2].fields.is_empty());
        assert_eq!(
            status.related[0].levels[0].sections[1].fields[0].value,
            "1 · 123"
        );
        assert_eq!(before, std::fs::read(&path).unwrap());
        let db = Connection::open(&path).unwrap();
        db.execute_batch("UPDATE skill_effect_values SET value='1234.5' WHERE kind_id=1 AND effect_id=55 AND position=0;").unwrap();
        drop(db);
        let skill = definition(&path, Family::Skills, 4003).unwrap();
        assert_eq!(
            skill.levels[0].description_en.as_deref(),
            Some("Power 1,234.5")
        );
        assert_eq!(
            skill.levels[0].description_localized.as_deref(),
            Some("Puissance 1\u{202f}234,5")
        );
        assert_eq!(skill.related[0].tag, "SE · #55");
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn equipment_ranges_and_philosopher_stone_follow_source_fields_and_resolved_group() {
        let path =
            std::env::temp_dir().join(format!("browser-equipment-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let db = Connection::open(&path).unwrap();
        let mut rows = serde_json::json!([{"ID":{"Value":42},"$variant":{
            "WeaponAttackRange":150.0,"WeaponAttackHeightRange":300.0,
            "ExtendingWeapon":true,"ExtendingWeaponAttackRange":200.0,"ExtendingWeaponAttackHeightRange":200.0,
            "SoulAddGroup":"soul_group"
        }}]);
        let store = |rows: &serde_json::Value| {
            let data = lz4_flex::compress_prepend_size(&serde_json::to_vec(rows).unwrap());
            db.execute(
                "UPDATE catalogue_sources SET decoded_lz4=?1 WHERE name='Item'",
                [data],
            )
            .unwrap();
        };
        store(&rows);
        let soul = lz4_flex::compress_prepend_size(br#"[{"Group":"soul_group"}]"#);
        db.execute("INSERT INTO catalogue_sources VALUES('ItemSoulAdd',1,'a','b','ItemSoulAddTableItem',0,'decoded',1,?1)",[soul]).unwrap();
        db.execute("INSERT INTO item_progression_links VALUES(42,'ItemSoulAdd',0,'SoulAddGroup','soul_group',NULL)",[]).unwrap();
        let before = std::fs::read(&path).unwrap();
        let card = item(&path, 42).unwrap();
        assert_eq!(card.attack_ranges.len(), 2);
        assert!(card.attack_ranges[0].horizontal);
        assert!(!card.attack_ranges[1].horizontal);
        assert_eq!(
            card.attack_ranges[0].name(true),
            "Attack Range (Horizontal)"
        );
        assert_eq!(
            card.attack_ranges[0].name(false),
            "Portée d’attaque (horizontale)"
        );
        assert_eq!(card.attack_ranges[0].value, "1.5m");
        assert_eq!(card.attack_ranges[0].value(false), "1,5m");
        assert_eq!(card.attack_ranges[1].value, "3m");
        assert_eq!(card.attack_ranges[0].extension.as_deref(), Some("+2m"));
        assert_eq!(card.attack_ranges[1].extension.as_deref(), Some("+2m"));
        assert!(card.supports_philosopher_stone);
        assert_eq!(before, std::fs::read(&path).unwrap());
        rows[0]["$variant"]["ExtendingWeapon"] = serde_json::json!(false);
        store(&rows);
        let card = item(&path, 42).unwrap();
        assert_eq!(card.attack_ranges.len(), 2);
        assert!(card.attack_ranges.iter().all(|r| r.extension.is_none()));
        db.execute(
            "DELETE FROM item_progression_links WHERE source_name='ItemSoulAdd'",
            [],
        )
        .unwrap();
        assert!(!item(&path, 42).unwrap().supports_philosopher_stone);
        rows[0]["$variant"]["WeaponAttackRange"] = serde_json::json!(0);
        rows[0]["$variant"]["WeaponAttackHeightRange"] = serde_json::json!(0);
        store(&rows);
        assert!(item(&path, 42).unwrap().attack_ranges.is_empty());
        drop(db);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn item_slider_levels_and_bonuses_follow_linked_source_definitions() {
        let path =
            std::env::temp_dir().join(format!("browser-enchant-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let db = Connection::open(&path).unwrap();
        let rows = serde_json::json!([
            {"Level":1,"ItemLevelValue":2,"StatList":[{"StatType":"EStat::WeaponMinDamage","StatValue":11},{"StatType":"EStat::WeaponDamage","StatValue":11}]},
            {"Level":3,"ItemLevelValue":4,"StatList":[{"StatType":"EStat::WeaponMinDamage","StatValue":30},{"StatType":"EStat::WeaponDamage","StatValue":30}]}
        ]);
        let data = lz4_flex::compress_prepend_size(&serde_json::to_vec(&rows).unwrap());
        db.execute("INSERT INTO catalogue_sources VALUES('EnchantEffect',1,'a','b','EnchantEffectTableItem',0,'decoded',2,?1)",[data]).unwrap();
        db.execute_batch("INSERT INTO item_progression_links VALUES(42,'EnchantEffect',0,'EnchantEffectGroup','group',1),(42,'EnchantEffect',1,'EnchantEffectGroup','group',3);
        INSERT INTO stats(id,code,name_en,name_localized) VALUES(1,'EStat::WeaponMinDamage',NULL,NULL),(2,'EStat::WeaponDamage',NULL,NULL);
        INSERT INTO stats(id,code,expression_format) VALUES(3,'EStat::AmplifyAllDamage','percentage');
        INSERT INTO item_main_stats VALUES(42,0,1,100),(42,1,2,200);").unwrap();
        for (name, rows) in [
            (
                "ExceedEnchant",
                serde_json::json!([
                    {"CurrentLevel":1,"AdditionalStatName":"amp1"},
                    {"CurrentLevel":3,"AdditionalStatName":"amp3"}
                ]),
            ),
            (
                "AdditionalStat",
                serde_json::json!([
                    {"Name":"amp1","AdditionalStats":[{"Type":"EStat::WeaponFixingDamage","Value":14},{"Type":"EStat::AmplifyAllDamage","Value":100}]},
                    {"Name":"amp3","AdditionalStats":[{"Type":"EStat::WeaponFixingDamage","Value":43},{"Type":"EStat::AmplifyAllDamage","Value":500}]}
                ]),
            ),
        ] {
            let data = lz4_flex::compress_prepend_size(&serde_json::to_vec(&rows).unwrap());
            db.execute(
                "INSERT INTO catalogue_sources VALUES(?1,1,'a','b','test',0,'decoded',2,?2)",
                rusqlite::params![name, data],
            )
            .unwrap();
        }
        db.execute_batch("INSERT INTO item_progression_links VALUES(42,'ExceedEnchant',0,'ExceedEnchantGroup','amp',1),(42,'ExceedEnchant',1,'ExceedEnchantGroup','amp',3);
        INSERT INTO catalogue_labels VALUES('EStat','WeaponFixingDamage','attack','Attack','ATQ');").unwrap();
        drop(db);
        let card = item(&path, 42).unwrap();
        assert_eq!(card.stats[0].value, "100 ~ 200");
        assert_eq!(
            card.enchantments
                .iter()
                .map(|s| s.level)
                .collect::<Vec<_>>(),
            vec![1, 3, 4, 6]
        );
        assert_eq!(card.enchantments[0].item_level_bonus, Some(2));
        assert_eq!(card.enchantments[0].stats["EStat::WeaponMinDamage"], 11);
        assert_eq!(card.enchantments[1].stats["EStat::WeaponDamage"], 30);
        assert_eq!(card.enchantments[2].amplify_level, 1);
        assert_eq!(card.enchantments[2].item_level_bonus, Some(9));
        assert_eq!(card.enchantments[3].amplify_level, 3);
        assert_eq!(card.enchantments[3].normal_level(), 3);
        assert_eq!(card.enchantments[3].title(), "+6");
        assert_eq!(card.enchantments[2].title(), "+4");
        assert_eq!(card.enchantments[3].stats["EStat::WeaponDamage"], 30);
        assert_eq!(card.enchantments[3].item_level_bonus, Some(19));
        assert_eq!(card.enchantments[2].amplification_stats[0].value, "+14");
        assert_eq!(card.enchantments[3].amplification_stats[0].value, "+43");
        assert_eq!(card.enchantments[2].amplification_stats[1].value, "+1%");
        assert_eq!(card.enchantments[3].amplification_stats[1].value, "+5%");
        assert_eq!(
            card.enchantments[3].amplification_stats[1].bonus_text(125, true),
            "+1.25%"
        );
        assert_eq!(
            card.enchantments[3].amplification_stats[1].bonus_text(50, true),
            "+0.5%"
        );
        assert_eq!(
            card.enchantments[3].amplification_stats[0].name.text(false),
            "ATQ"
        );
        std::fs::remove_file(&path).unwrap();
    }
    fn resource_fixture(path: &Path) {
        fixture(path);
        let db = Connection::open(path).unwrap();
        for sql in [
            include_str!("catalogue_projection.sql"),
            include_str!("crafting_quests.sql"),
            include_str!("achievements.sql"),
            include_str!("companions.sql"),
            include_str!("daevanion.sql"),
            include_str!("world.sql"),
        ] {
            db.execute_batch(sql).unwrap();
        }
        for source in [
            "CraftRecipe",
            "ConversionRecipe",
            "Achievement",
            "BoxReward",
        ] {
            let rows =
                serde_json::json!([{"ID":7,"AchievementId":7,"Group":"Box_1","Marker":source}]);
            let data = lz4_flex::compress_prepend_size(&serde_json::to_vec(&rows).unwrap());
            db.execute(
                "INSERT INTO catalogue_sources VALUES(?1,1,'a','b','Example',0,'decoded',1,?2)",
                rusqlite::params![source, data],
            )
            .unwrap();
        }
        db.execute_batch("INSERT INTO recipes(source_name,id,source_key,client_version_id,name_en,name_localized) VALUES
            ('CraftRecipe',7,'Craft_7',1,'Craft sword','Fabriquer une épée'),('ConversionRecipe',7,'Convert_7',1,'Convert sword','Convertir une épée');
            INSERT INTO recipe_results(source_name,recipe_id,role,position,item_reference,item_id,quantity) VALUES
            ('CraftRecipe',7,'result',1,'Item_42',42,2),('ConversionRecipe',7,'result',1,'Item_42',42,9);
            INSERT INTO achievements(id,client_version_id,name_en,name_localized,objective_values_json) VALUES(7,1,'Explorer','Explorateur','{}');
            INSERT INTO catalogue_entity_sources VALUES('achievements',7,'Achievement',0),('craft_recipes',7,'CraftRecipe',0),('conversion_recipes',7,'ConversionRecipe',0);
            INSERT INTO loot_tables VALUES(1,'BoxReward','Box_1',1);
            INSERT INTO item_rewards VALUES('BoxReward',0,'Rewards[0]',NULL,'Box_1','random','item','Item_42',42,1,1,'100');
            INSERT INTO daevanion_boards(id,client_version_id,name_en,name_localized) VALUES(1,1,'Board','Plateau');
            INSERT INTO stats(id,code,name_en,name_localized) VALUES(1,'Attack','Attack','Attaque');
            INSERT INTO daevanion_nodes(id,client_version_id,board_id,auto_learn,condition_value_source) VALUES(11,1,1,0,'');
            INSERT INTO daevanion_stat_parameters VALUES(11,1,'Attack',1,'500',500,'candidate');
            INSERT INTO skins(id,source_key,client_version_id) VALUES(1,'Skin_1',1);
            INSERT INTO skin_variants VALUES(1,1,1,'Costume','Tenue',NULL,'Resource');
            INSERT INTO seasons(id,client_version_id,season_number) VALUES(1,1,42);
            INSERT INTO catalogue_conditions VALUES('achievements',7,'Achievement',0,'Condition',NULL,'{}','uninterpreted');
        ").unwrap();
    }
    fn resource(key: &str) -> Family {
        Family::Resources(*ResourceFamily::ALL.iter().find(|f| f.key() == key).unwrap())
    }
    #[test]
    fn resources_cover_every_projected_family_and_keep_scoped_recipe_ids_separate() {
        let path =
            std::env::temp_dir().join(format!("browser-resources-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        resource_fixture(&path);
        let before = std::fs::read(&path).unwrap();
        for family in ResourceFamily::ALL {
            list(&path, Family::Resources(*family), "", 0).unwrap();
        }
        for (key, name, quantity) in [
            ("craft_recipes", "Fabriquer une épée", "2"),
            ("conversion_recipes", "Convertir une épée", "9"),
        ] {
            let results = list(&path, resource(key), "ÉPÉE", 0).unwrap();
            assert_eq!(results.total, 1);
            assert_eq!(results.entries[0].name, name);
            let card = definition(&path, resource(key), 7).unwrap();
            assert_eq!(card.name_localized.as_deref(), Some(name));
            let results: Vec<_> = card
                .related
                .iter()
                .filter(|c| c.tag == "recipe_results")
                .collect();
            assert_eq!(results.len(), 1);
            assert!(
                results[0].levels[0].sections[0]
                    .fields
                    .iter()
                    .any(|f| f.value == format!("quantity : {quantity}"))
            );
            let lines = detail(&path, resource(key), 7, false).unwrap().join("\n");
            assert!(lines.contains(name));
        }
        assert_eq!(list(&path, resource("skins"), "tenue", 0).unwrap().total, 1);
        assert_eq!(
            definition(&path, resource("skins"), 1)
                .unwrap()
                .name_en
                .as_deref(),
            Some("Costume")
        );
        assert_eq!(
            list(&path, resource("seasons"), "1", 0).unwrap().entries[0].name,
            "#1"
        );
        let card = definition(&path, resource("daevanion_nodes"), 11).unwrap();
        assert!(
            card.levels[0].sections[0]
                .fields
                .iter()
                .any(|f| f.value == "No" && f.value_localized.as_deref() == Some("Non"))
        );
        let stats = card
            .related
            .iter()
            .find(|c| c.tag == "daevanion_stat_parameters")
            .unwrap();
        assert!(stats.levels[0].sections[0].fields.iter().any(
            |f| f.value == "1 · Attack" && f.value_localized.as_deref() == Some("1 · Attaque")
        ));
        let normal = definition(&path, resource("achievements"), 7).unwrap();
        assert!(
            normal
                .related
                .iter()
                .any(|c| c.tag == "catalogue_conditions")
        );
        let source = detail(&path, resource("achievements"), 7, true)
            .unwrap()
            .join("\n");
        assert!(source.contains("Achievement"));
        let source = detail(&path, resource("craft_recipes"), 7, true)
            .unwrap()
            .join("\n");
        assert!(source.contains("CraftRecipe"));
        assert!(!source.contains("ConversionRecipe"));
        let loot = definition(&path, resource("loot_tables"), 1).unwrap();
        assert!(loot.related.iter().any(|c| c.tag == "item_rewards"));
        let source = detail(&path, resource("loot_tables"), 1, true)
            .unwrap()
            .join("\n");
        assert!(source.contains("Source liée BoxReward"));
        assert!(source.contains("Box_1"));
        assert_eq!(std::fs::read(&path).unwrap(), before);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn resource_pagination_and_relation_limits_are_explicit() {
        let path = std::env::temp_dir().join(format!(
            "browser-resource-limits-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        resource_fixture(&path);
        let db = Connection::open(&path).unwrap();
        for id in 100..350 {
            db.execute(
                "INSERT INTO daevanion_nodes(id,client_version_id,board_id) VALUES(?1,1,1)",
                [id],
            )
            .unwrap();
        }
        drop(db);
        let list = list(&path, resource("daevanion_nodes"), "", 100).unwrap();
        assert_eq!(list.total, 251);
        assert_eq!(list.entries.len(), PAGE_SIZE);
        assert_eq!(list.entries[0].id, 199);
        let card = definition(&path, resource("daevanion_boards"), 1).unwrap();
        assert_eq!(card.related.len(), RELATION_LIMIT);
        assert!(card.related_truncated);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn item_stat_values_use_the_declared_expression_format() {
        let path =
            std::env::temp_dir().join(format!("item-stat-format-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let db = Connection::open(&path).unwrap();
        db.execute_batch(
            "INSERT INTO stats(id,code,expression_format) VALUES
            (1,'EStat::DecreaseCriticalDamage','percentage'),
            (2,'EStat::HPMax','integer'),(3,'EStat::FutureStat',NULL);
            INSERT INTO item_main_stats VALUES(42,0,1,150),(42,1,2,25),(42,2,3,150);",
        )
        .unwrap();
        drop(db);
        let card = item(&path, 42).unwrap();
        assert_eq!(card.stats[0].value, "1.5%");
        assert_eq!(card.stats[0].text(false), "1,5 %");
        assert_eq!(card.stats[0].bonus_text(-125, false), "-1,25 %");
        assert_eq!(card.stats[0].bonus_text(-125, true), "-1.25%");
        assert_eq!(card.stats[0].bonus_text(0, true), "+0%");
        assert_eq!(card.stats[1].value, "25");
        assert_eq!(card.stats[1].bonus_text(25, true), "+25");
        assert_eq!(card.stats[2].value, "150");
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn item_card_loads_wide_and_square_backgrounds_through_source_rarity() {
        let path = std::env::temp_dir().join(format!(
            "item-background-card-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        fixture(&path);
        let db = Connection::open(&path).unwrap();
        db.execute_batch("UPDATE rarities SET rarity='Epic',rarity_source='EItemGrade::Legend' WHERE id=1;
            INSERT INTO catalogue_icons VALUES(101,'UT_ItemTooltipGrade_Legend',X'0102','image/png','available',NULL),(102,'UT_SlotGrade_Legend',X'0304','image/png','available',NULL);
            INSERT INTO item_backgrounds VALUES(1,101,102);").unwrap();
        drop(db);
        let before = std::fs::read(&path).unwrap();
        let card = item(&path, 42).unwrap();
        assert_eq!(card.rarity, "Legend");
        assert_eq!(card.wide_background, Some(vec![1, 2]));
        assert_eq!(card.square_background, Some(vec![3, 4]));
        assert_eq!(std::fs::read(&path).unwrap(), before);
        std::fs::remove_file(path).unwrap();
    }
}
