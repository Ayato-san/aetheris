//! Display projection built once during export; card reads never decode bulk tables.
use super::*;

fn rows<'a>(batch: &'a BTreeMap<String, TableProbe>, table: &str) -> &'a [Value] {
    batch
        .get(table)
        .map(|t| t.layout.rows.as_slice())
        .unwrap_or(&[])
}
struct Labels<'a> {
    en: &'a Localization,
    loc: &'a Localization,
    items: BTreeMap<&'a str, (i64, String, Option<String>)>,
    currencies: BTreeMap<&'a str, i64>,
    entities: BTreeMap<(&'a str, String), &'a Value>,
}
impl Labels<'_> {
    fn name(
        &self,
        table: &str,
        row: &Value,
        fallback: &str,
    ) -> (String, Option<String>, Option<i64>) {
        if table == "Item"
            && let Some((id, en, loc)) = string(&row["Name"]).and_then(|n| self.items.get(n))
        {
            return (en.clone(), loc.clone(), Some(*id));
        }
        let product = ["Product", "ItemName", "ArcanaName"]
            .into_iter()
            .find_map(|f| string(&row[f]).and_then(|n| self.items.get(n)));
        if let Some((id, en, loc)) = product {
            return (en.clone(), loc.clone(), Some(*id));
        }
        for field in ["Title", "Desc", "QuestText", "ProductString"] {
            if let Some(key) = string(&row[field]["Key"])
                && let Some(en) = text(self.en, Some(key))
            {
                return (en, text(self.loc, Some(key)), None);
            }
        }
        if let Some(npc) =
            number(row, "NpcId").and_then(|id| self.entities.get(&("NpcData", id.to_string())))
        {
            return self.name("NpcData", npc, fallback);
        }
        let name = ["Name", "Group", "PackageName", "GroupName"]
            .into_iter()
            .find_map(|f| string(&row[f]))
            .unwrap_or(fallback);
        (name.into(), None, None)
    }
}
struct Relation<'a> {
    item: i64,
    direction: &'a str,
    kind: &'a str,
    table: &'a str,
    row: usize,
    path: &'a str,
    name: (String, Option<String>, Option<i64>),
    min: Option<i64>,
    max: Option<i64>,
    weight: Option<String>,
}
fn insert(db: &Connection, r: Relation<'_>) -> Result<()> {
    db.execute("INSERT OR IGNORE INTO item_relations VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,NULL)", params![r.item,r.direction,r.kind,r.table,r.row as i64,r.path,r.name.0,r.name.1,r.name.2,r.min,r.max,r.weight,if r.weight.is_some(){"unknown_scale"}else{"not_provided"}])?;
    Ok(())
}
pub(super) fn write(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    en: &Localization,
    loc: &Localization,
) -> Result<()> {
    let mut labels = Labels {
        en,
        loc,
        items: BTreeMap::new(),
        currencies: BTreeMap::new(),
        entities: BTreeMap::new(),
    };
    for row in rows(batch, "Item") {
        if let (Some(id), Some(name)) = (number(row, "ID"), string(&row["Name"])) {
            let names: (Option<String>, Option<String>) = db.query_row(
                "SELECT name_en,name_localized FROM items WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            if let Some(category) = string(&row["$variant"]["CurrencyCategory"]) {
                labels.currencies.insert(category, id);
            }
            labels
                .items
                .insert(name, (id, names.0.unwrap_or_else(|| name.into()), names.1));
        }
    }
    for (table, data) in batch {
        for row in &data.layout.rows {
            for field in ["ID", "AchievementId", "DataId", "Name", "Group"] {
                let key = number(row, field)
                    .map(|n| n.to_string())
                    .or_else(|| string(&row[field]).map(str::to_owned));
                if let Some(key) = key {
                    labels.entities.insert((table, key), row);
                }
            }
        }
    }
    let routes=db.prepare("SELECT item_id,route_type,source_row,source_path,target_table,target_reference FROM item_obtaining_routes WHERE target_table IS NOT NULL AND target_table NOT IN ('Map','Subzone','SeasonSchedule') AND source_path NOT LIKE '%KibeliskIds%' ORDER BY item_id,source_row,source_path")?.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)? as usize,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut routed = BTreeSet::new();
    let mut unique = BTreeSet::new();
    for (item, kind, pos, path, table, reference) in routes {
        routed.insert((item, kind.clone()));
        if !unique.insert((item, kind.clone(), table.clone(), reference.clone())) {
            continue;
        }
        let fallback = format!("{table} #{reference}");
        let name = labels
            .entities
            .get(&(table.as_str(), reference))
            .map(|row| labels.name(&table, row, &fallback))
            .unwrap_or((fallback, None, None));
        insert(
            db,
            Relation {
                item,
                direction: "source",
                kind: &kind,
                table: "ItemGetRoute",
                row: pos,
                path: &path,
                name,
                min: None,
                max: None,
                weight: None,
            },
        )?;
    }
    let sources=db.prepare("SELECT s.item_id,t.code,s.source_name,s.source_row,s.source_path,s.evidence,s.quantity_min,s.quantity_max,s.weight_raw FROM item_obtaining_sources s JOIN item_obtaining_types t ON t.id=s.type_id")?.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)? as usize,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,Option<i64>>(6)?,r.get::<_,Option<i64>>(7)?,r.get::<_,Option<String>>(8)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    for (item, kind, table, pos, path, evidence, min, max, weight) in sources {
        if evidence == "client_route" && routed.contains(&(item, kind.clone())) {
            continue;
        }
        let row = &rows(batch, &table)[pos];
        let fallback = format!("{table} #{}", pos + 1);
        let name = if evidence == "client_route" {
            (String::new(), None, None)
        } else {
            labels.name(&table, row, &fallback)
        };
        if table == "BoxReward" {
            let owners=db.prepare("SELECT DISTINCT container_item_id FROM item_rewards WHERE source_name='BoxReward' AND source_row=?1 AND container_item_id IS NOT NULL")?.query_map([pos as i64],|r|r.get::<_,i64>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
            for owner in &owners {
                let owner_name = labels
                    .entities
                    .get(&("Item", owner.to_string()))
                    .map(|r| labels.name("Item", r, "Chest"))
                    .unwrap_or((format!("Item #{owner}"), None, Some(*owner)));
                insert(
                    db,
                    Relation {
                        item,
                        direction: "source",
                        kind: &kind,
                        table: &table,
                        row: pos,
                        path: &format!("{path}/owner/{owner}"),
                        name: owner_name,
                        min,
                        max,
                        weight: weight.clone(),
                    },
                )?;
            }
            if !owners.is_empty() {
                continue;
            }
        }
        insert(
            db,
            Relation {
                item,
                direction: "source",
                kind: &kind,
                table: &table,
                row: pos,
                path: &path,
                name,
                min,
                max,
                weight,
            },
        )?;
    }
    let contents=db.prepare("SELECT container_item_id,result_item_id,source_name,source_row,source_path,quantity_min,quantity_max,weight_raw FROM item_rewards WHERE container_item_id IS NOT NULL AND result_item_id IS NOT NULL")?.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)? as usize,r.get::<_,String>(4)?,r.get::<_,Option<i64>>(5)?,r.get::<_,Option<i64>>(6)?,r.get::<_,Option<String>>(7)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    for (owner, result, table, pos, path, min, max, weight) in contents {
        if let Some(row) = labels.entities.get(&("Item", result.to_string())) {
            insert(
                db,
                Relation {
                    item: owner,
                    direction: "use",
                    kind: "RewardBoxInfo",
                    table: &table,
                    row: pos,
                    path: &path,
                    name: labels.name("Item", row, "Item"),
                    min,
                    max,
                    weight,
                },
            )?;
        }
    }
    for (table, data) in batch {
        for (pos, row) in data.layout.rows.iter().enumerate() {
            let fields: &[&str] = match table.as_str() {
                "CraftRecipe" | "ConversionRecipe" => {
                    &["ComponentList", "SuccessionComponentCandidateList"]
                }
                "ArcanaCreate" => &["MaterialName"],
                "PeriodItemCollectionList" => &["TargetItems"],
                "ItemSoulAdd" => &["CostItem"],
                "ItemSuccession" => &["CostItem"],
                "ItemSurpass" | "Enchant" | "ExceedEnchant" | "ArcanaEnchant" => {
                    &["CostItems", "CostItem", "MaterialName"]
                }
                "FieldEvent" => &["EntranceTeleportItem"],
                "ContentsSweepTicket" => &["ItemName"],
                _ => &[],
            };
            for (slot, price) in row["ItemPriceList"]
                .as_array()
                .into_iter()
                .flatten()
                .enumerate()
            {
                if let Some(item) =
                    string(&price["ItemCurrencyType"]).and_then(|c| labels.currencies.get(c))
                {
                    let count = number(price, "ItemPrice");
                    insert(
                        db,
                        Relation {
                            item: *item,
                            direction: "use",
                            kind: table,
                            table,
                            row: pos,
                            path: &format!("ItemPriceList/{slot}"),
                            name: labels.name(table, row, table),
                            min: count,
                            max: count,
                            weight: None,
                        },
                    )?;
                }
            }
            if let Some(item) =
                string(&row["CostCurrencyType"]).and_then(|c| labels.currencies.get(c))
            {
                for field in ["CostGold", "CostCurrencyValue"] {
                    if let Some(count) = number(row, field).filter(|v| *v > 0) {
                        insert(
                            db,
                            Relation {
                                item: *item,
                                direction: "use",
                                kind: table,
                                table,
                                row: pos,
                                path: field,
                                name: labels.name(table, row, table),
                                min: Some(count),
                                max: Some(count),
                                weight: None,
                            },
                        )?;
                    }
                }
            }
            for field in fields {
                let count = match *field {
                    "MaterialName" => number(row, "MaterialCount"),
                    "CostItem" => number(row, "CostItemCount"),
                    "EntranceTeleportItem" => number(row, "EntranceTeleportItemCount"),
                    _ => None,
                };
                inputs(
                    db,
                    &labels,
                    &Relation {
                        item: 0,
                        direction: "use",
                        kind: table,
                        table,
                        row: pos,
                        path: field,
                        name: labels.name(table, row, &format!("{table} #{}", pos + 1)),
                        min: count,
                        max: count,
                        weight: None,
                    },
                    &row[field],
                    field,
                )?;
            }
        }
    }
    coverage(
        db,
        "items.relations",
        "partial",
        "display names and explicit material uses indexed; route hints are distinct from reward definitions; drop scales and package-chain probabilities remain unverified",
    )
}
fn inputs(
    db: &Connection,
    labels: &Labels<'_>,
    origin: &Relation<'_>,
    value: &Value,
    path: &str,
) -> Result<()> {
    if let Some(name) = string(value).or_else(|| string(&value["Item"])) {
        if let Some((item, _, _)) = labels.items.get(name) {
            let count = number(value, "Count").or(origin.min);
            insert(
                db,
                Relation {
                    item: *item,
                    path,
                    min: count,
                    max: count,
                    name: origin.name.clone(),
                    weight: None,
                    ..*origin
                },
            )?;
        }
        // ReplacementItem is an alternative component, not an output.
        if let Some(replacement) = string(&value["ReplacementItem"]) {
            inputs(
                db,
                labels,
                &Relation {
                    min: number(value, "Count").or(origin.min),
                    name: origin.name.clone(),
                    weight: None,
                    ..*origin
                },
                &Value::String(replacement.into()),
                &format!("{path}/ReplacementItem"),
            )?;
        }
    } else if let Some(array) = value.as_array() {
        for (i, entry) in array.iter().enumerate() {
            inputs(db, labels, origin, entry, &format!("{path}/{i}"))?;
        }
    }
    Ok(())
}
