//! The item card projects only catalogue values which have a visual purpose.
use super::*;
use rusqlite::{OptionalExtension, params};

// In-game rule confirmed by the user: each amplification adds five GS.
const AMPLIFY_GS_PER_LEVEL: i64 = 5;

/// A client label keeps both languages so switching the card is immediate.
#[derive(Debug)]
pub struct ItemLabel {
    pub name_en: String,
    pub name_localized: Option<String>,
}
impl ItemLabel {
    fn code(code: String) -> Self {
        Self {
            name_en: code,
            name_localized: None,
        }
    }
    pub fn text(&self, english: bool) -> &str {
        if english {
            &self.name_en
        } else {
            self.name_localized.as_deref().unwrap_or(&self.name_en)
        }
    }
}
fn label(db: &Connection, ty: &str, code: &str) -> Result<ItemLabel> {
    Ok(db
        .query_row(
            "SELECT name_en,name_localized FROM catalogue_labels WHERE enum_type=?1 AND code=?2",
            params![ty, code],
            |r| {
                Ok(ItemLabel {
                    name_en: r
                        .get::<_, Option<String>>(0)?
                        .unwrap_or_else(|| code.into()),
                    name_localized: r.get(1)?,
                })
            },
        )
        .optional()?
        .unwrap_or_else(|| ItemLabel::code(code.into())))
}
#[derive(Debug)]
pub struct ItemStat {
    pub code: String,
    pub name: ItemLabel,
    pub value: String,
}
impl ItemStat {
    pub fn bonus_text(&self, value: i64) -> String {
        stat_value(&self.code, value, true)
    }
}
fn stat_value(code: &str, value: i64, bonus: bool) -> String {
    let number = if code == "EStat::AmplifyAllDamage" {
        // Damage Boost uses hundredths of one percent in the client tables.
        let magnitude = value.unsigned_abs();
        let fraction = magnitude % 100;
        let decimals = if fraction == 0 {
            String::new()
        } else {
            format!(".{fraction:02}").trim_end_matches('0').to_owned()
        };
        format!(
            "{}{}{decimals} %",
            if value < 0 { "-" } else { "" },
            magnitude / 100
        )
    } else {
        value.to_string()
    };
    if bonus && value >= 0 {
        format!("+{number}")
    } else {
        number
    }
}
#[derive(Debug)]
pub struct ItemEnchantment {
    /// Slider position; amplification follows the highest normal level.
    pub level: i64,
    pub amplify_level: i64,
    pub amplification_stats: Vec<ItemStat>,
    pub item_level_bonus: Option<i64>,
    pub stats: BTreeMap<String, i64>,
}
impl ItemEnchantment {
    pub fn normal_level(&self) -> i64 {
        self.level - self.amplify_level
    }
    pub fn title(&self) -> String {
        format!("+{}", self.level)
    }
}
#[derive(Debug)]
pub struct SetBonus {
    pub name: ItemLabel,
    pub count: i64,
    pub description: ItemLabel,
}
#[derive(Debug)]
pub struct ItemAttackRange {
    pub horizontal: bool,
    pub value: String,
    pub extension: Option<String>,
}
impl ItemAttackRange {
    pub fn name(&self, english: bool) -> &'static str {
        match (self.horizontal, english) {
            (true, true) => "Attack Range (Horizontal)",
            (false, true) => "Attack Range (Vertical)",
            (true, false) => "Portée d’attaque (horizontale)",
            (false, false) => "Portée d’attaque (verticale)",
        }
    }
}
#[derive(Debug)]
pub struct ItemCard {
    pub id: i64,
    pub name_en: Option<String>,
    pub name_localized: Option<String>,
    pub description_en: Option<String>,
    pub description_localized: Option<String>,
    pub rarity: String,
    pub rarity_label: ItemLabel,
    pub category: ItemLabel,
    pub armor: Option<ItemLabel>,
    pub level: i64,
    pub required_level: Option<i64>,
    pub race: Option<ItemLabel>,
    pub sell_price: i64,
    pub currency: ItemLabel,
    pub currency_icon: Option<Vec<u8>>,
    pub currency_icon_mime: Option<String>,
    pub manastones: Option<i64>,
    pub theostones: Option<i64>,
    pub icon: Option<Vec<u8>>,
    pub icon_mime: Option<String>,
    pub wide_background: Option<Vec<u8>>,
    pub square_background: Option<Vec<u8>>,
    pub stats: Vec<ItemStat>,
    pub attack_ranges: Vec<ItemAttackRange>,
    pub supports_philosopher_stone: bool,
    pub sets: Vec<SetBonus>,
    pub permissions: Vec<(String, bool)>,
    pub enchantments: Vec<ItemEnchantment>,
}
pub(super) fn load(db: &Connection, id: i64) -> Result<ItemCard> {
    let mut card=db.query_row("SELECT i.id,i.name_en,i.name_localized,i.description_en,i.description_localized,r.rarity,c.code,a.code,i.item_level,d.required_level_min,CASE WHEN race.code='All' THEN NULL ELSE race.code END,i.sell_price,m.code,i.magic_stone_slots,i.godstone_slots,ic.image,ic.mime_type,ci.image,ci.mime_type,r.rarity_source FROM items i JOIN rarities r ON r.id=i.rarity_id JOIN item_categories c ON c.id=i.category_id LEFT JOIN armor_types a ON a.id=i.armor_type_id LEFT JOIN item_details d ON d.item_id=i.id LEFT JOIN races race ON race.id=i.race_id JOIN sell_currencies m ON m.id=i.sell_currency_id LEFT JOIN catalogue_entity_icons ei ON ei.family='items' AND ei.source_id=i.id LEFT JOIN catalogue_icons ic ON ic.id=ei.icon_id LEFT JOIN catalogue_entity_icons ce ON ce.family='sell_currencies' AND ce.source_id=m.id LEFT JOIN catalogue_icons ci ON ci.id=ce.icon_id WHERE i.id=?1",[id],|r| { let source: String = r.get(19)?; let rarity = source.strip_prefix("EItemGrade::").unwrap_or(&source).to_owned(); Ok(ItemCard {
        id:r.get(0)?,name_en:r.get(1)?,name_localized:r.get(2)?,description_en:r.get(3)?,description_localized:r.get(4)?,rarity:rarity.clone(),rarity_label:ItemLabel::code(rarity),category:ItemLabel::code(r.get(6)?),armor:r.get::<_,Option<String>>(7)?.map(ItemLabel::code),level:r.get(8)?,required_level:r.get(9)?,race:r.get::<_,Option<String>>(10)?.map(ItemLabel::code),sell_price:r.get(11)?,currency:ItemLabel::code(r.get(12)?),manastones:r.get(13)?,theostones:r.get(14)?,icon:r.get(15)?,icon_mime:r.get(16)?,currency_icon:r.get(17)?,currency_icon_mime:r.get(18)?,wide_background:None,square_background:None,stats:vec![],attack_ranges:vec![],supports_philosopher_stone:false,sets:vec![],permissions:vec![],enchantments:vec![],
    })})?;
    if let Some((wide,square)) = db.query_row(
        "SELECT w.image,s.image FROM items i JOIN item_backgrounds b ON b.rarity_id=i.rarity_id JOIN catalogue_icons w ON w.id=b.wide_icon_id JOIN catalogue_icons s ON s.id=b.square_icon_id WHERE i.id=?1",
        [id], |r|Ok((r.get(0)?,r.get(1)?))
    ).optional()? {card.wide_background=wide;card.square_background=square;}
    let stats=db.prepare("SELECT s.code,s.name_en,s.name_localized,v.value FROM item_main_stats v JOIN stats s ON s.id=v.stat_id WHERE v.item_id=?1 ORDER BY v.position")?.query_map([id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,Option<String>>(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,i64>(3)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let minimum = stats
        .iter()
        .find(|(c, _, _, _)| c == "EStat::WeaponMinDamage")
        .map(|(_, _, _, v)| *v);
    let maximum = stats
        .iter()
        .find(|(c, _, _, _)| c == "EStat::WeaponDamage")
        .map(|(_, _, _, v)| *v);
    for (code, name_en, name_localized, value) in stats {
        let mut name = ItemLabel {
            name_en: name_en.unwrap_or_else(|| stat_name(&code)),
            name_localized,
        };
        if code == "EStat::WeaponMinDamage" && maximum.is_some() {
            name = label(db, "EStat", "WeaponFixingDamage")?;
        }
        if code == "EStat::WeaponDamage" && minimum.is_some() {
            continue;
        }
        let value = if code == "EStat::WeaponMinDamage" {
            maximum
                .map(|max| format!("{value} ~ {max}"))
                .unwrap_or_else(|| value.to_string())
        } else {
            stat_value(&code, value, false)
        };
        card.stats.push(ItemStat { name, code, value });
    }
    card.permissions=db.prepare("SELECT f.code,1 FROM item_flags p JOIN flags f ON f.id=p.flag_id WHERE p.item_id=?1 UNION ALL SELECT f.code,0 FROM item_false_flags p JOIN flags f ON f.id=p.flag_id WHERE p.item_id=?1 ORDER BY 1")?.query_map([id],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?;
    card.sets=db.prepare("SELECT COALESCE(d.name_en,s.code),d.name_localized,b.equipped_count,COALESCE(a.description_en,a.name_en,'—'),COALESCE(a.description_localized,a.name_localized) FROM item_sets si JOIN sets s ON s.id=si.set_id LEFT JOIN item_set_definitions d ON d.set_id=s.id AND d.source_row=(SELECT MIN(source_row) FROM item_set_definitions WHERE set_id=s.id) JOIN item_set_bonuses b ON b.set_id=s.id LEFT JOIN skill_abnormals a ON a.id=b.abnormal_id WHERE si.item_id=?1 ORDER BY s.id,b.equipped_count")?.query_map([id],|r|Ok(SetBonus{name:ItemLabel{name_en:r.get(0)?,name_localized:r.get(1)?},count:r.get(2)?,description:ItemLabel{name_en:r.get(3)?,name_localized:r.get(4)?}}))?.collect::<rusqlite::Result<_>>()?;
    card.rarity_label = label(db, "EItemGrade", &card.rarity)?;
    let family:String=db.query_row("SELECT f.code FROM item_categories c JOIN item_families f ON f.id=c.family_id JOIN items i ON i.category_id=c.id WHERE i.id=?1",[id],|r|r.get(0))?;
    let category_enum = match family.as_str() {
        "Equip" => "EEquipCategory",
        "Usable" => "EUsableCategory",
        "Misc" => "EMiscCategory",
        "Currency" => "ECurrencyCategory",
        _ => "",
    };
    if family == "Equip" {
        equipment_details(db, &mut card)?;
    }
    card.category = label(db, category_enum, &card.category.name_en)?;
    card.currency = label(db, "ECurrencyCategory", &card.currency.name_en)?;
    if let Some(armor) = &mut card.armor {
        *armor = label(db, "EArmorMaterialType", &armor.name_en)?;
    }
    if let Some(race) = &mut card.race {
        *race = label(db, "ERace", &race.name_en)?;
    }
    card.enchantments = enchantments(db, id)?;
    Ok(card)
}
fn equipment_details(db: &Connection, card: &mut ItemCard) -> Result<()> {
    // These fields are already preserved in the catalogue's Item evidence.
    let rows = target_export::source_rows(db, "Item")?;
    let row = rows
        .iter()
        .find(|row| row["ID"]["Value"].as_i64() == Some(card.id))
        .ok_or_else(|| invalid("item source row is absent"))?;
    let equipment = &row["$variant"];
    for (horizontal, base, extension) in [
        (true, "WeaponAttackRange", "ExtendingWeaponAttackRange"),
        (
            false,
            "WeaponAttackHeightRange",
            "ExtendingWeaponAttackHeightRange",
        ),
    ] {
        if let Some(value) = equipment[base]
            .as_f64()
            .filter(|v| v.is_finite() && *v > 0.)
        {
            let extension = if equipment["ExtendingWeapon"].as_bool() == Some(true) {
                equipment[extension]
                    .as_f64()
                    .filter(|v| v.is_finite() && *v > 0.)
                    .map(|v| format!("+{}", metres(v)))
            } else {
                None
            };
            card.attack_ranges.push(ItemAttackRange {
                horizontal,
                value: metres(value),
                extension,
            });
        }
    }
    if let Some(group) = equipment["SoulAddGroup"]
        .as_str()
        .filter(|s| !s.is_empty() && *s != "None")
    {
        card.supports_philosopher_stone=db.prepare("SELECT 1 FROM item_progression_links WHERE item_id=?1 AND source_name='ItemSoulAdd' AND source_field='SoulAddGroup' AND group_reference=?2")?.exists(params![card.id,group])?;
    }
    Ok(())
}
fn metres(centimetres: f64) -> String {
    let value = format!("{:.2}", centimetres / 100.);
    format!("{}m", value.trim_end_matches('0').trim_end_matches('.'))
}
fn enchantments(db: &Connection, id: i64) -> Result<Vec<ItemEnchantment>> {
    let references=db.prepare("SELECT source_name,source_row,level FROM item_progression_links WHERE item_id=?1 AND source_field='EnchantEffectGroup' AND source_name IN ('EnchantEffect','ArcanaEnchantEffectMainStat') AND level>0 ORDER BY source_name,level")?
        .query_map([id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut sources = BTreeMap::new();
    let mut steps = BTreeMap::new();
    for (name, pos, level) in references {
        if !sources.contains_key(&name) {
            sources.insert(name.clone(), target_export::source_rows(db, &name)?);
        }
        let row = sources[&name]
            .get(usize::try_from(pos)?)
            .ok_or_else(|| invalid("enchantment source row is absent"))?;
        let (list, key, value) = if name == "EnchantEffect" {
            ("StatList", "StatType", "StatValue")
        } else {
            ("List", "Stat", "Value")
        };
        let stats = row[list]
            .as_array()
            .ok_or_else(|| invalid("invalid enchantment statistics"))?
            .iter()
            .map(|s| {
                Ok((
                    s[key]
                        .as_str()
                        .ok_or_else(|| invalid("invalid enchantment stat"))?
                        .to_owned(),
                    s[value]
                        .as_i64()
                        .ok_or_else(|| invalid("invalid enchantment value"))?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        if steps
            .insert(
                level,
                ItemEnchantment {
                    level,
                    amplify_level: 0,
                    amplification_stats: vec![],
                    item_level_bonus: row["ItemLevelValue"].as_i64(),
                    stats,
                },
            )
            .is_some()
        {
            return Err(invalid("ambiguous enchantment level definitions").into());
        }
    }
    let refs=db.prepare("SELECT source_row,level FROM item_progression_links WHERE item_id=?1 AND source_name='ExceedEnchant' AND source_field='ExceedEnchantGroup' AND level>0 ORDER BY level")?
        .query_map([id],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    if !refs.is_empty() {
        let exceed = target_export::source_rows(db, "ExceedEnchant")?;
        let additional = target_export::source_rows(db, "AdditionalStat")?;
        let mut by_name = BTreeMap::new();
        for row in &additional {
            if let Some(key) = row["Name"].as_str()
                && by_name.insert(key, row).is_some()
            {
                return Err(invalid("ambiguous amplification statistics").into());
            }
        }
        let cap = steps.last_key_value().map(|(level, _)| *level).unwrap_or(0);
        for (pos, amplify_level) in refs {
            let row = exceed
                .get(usize::try_from(pos)?)
                .ok_or_else(|| invalid("amplification source row is absent"))?;
            let key = row["AdditionalStatName"]
                .as_str()
                .ok_or_else(|| invalid("amplification stat reference is absent"))?;
            let stats = by_name.get(key).ok_or_else(|| {
                invalid("amplification statistics are absent; regenerate the catalogue")
            })?["AdditionalStats"]
                .as_array()
                .ok_or_else(|| invalid("invalid amplification statistics"))?;
            let mut amplification_stats = vec![];
            for stat in stats {
                let code = stat["Type"]
                    .as_str()
                    .ok_or_else(|| invalid("invalid amplification stat"))?;
                let value = stat["Value"]
                    .as_i64()
                    .ok_or_else(|| invalid("invalid amplification value"))?;
                amplification_stats.push(ItemStat {
                    code: code.into(),
                    name: label(db, "EStat", code.strip_prefix("EStat::").unwrap_or(code))?,
                    value: stat_value(code, value, true),
                });
            }
            let base = steps.get(&cap);
            let step = ItemEnchantment {
                level: cap + amplify_level,
                amplify_level,
                amplification_stats,
                item_level_bonus: Some(
                    base.and_then(|s| s.item_level_bonus).unwrap_or(0)
                        + amplify_level * AMPLIFY_GS_PER_LEVEL,
                ),
                stats: base.map(|s| s.stats.clone()).unwrap_or_default(),
            };
            if steps.insert(step.level, step).is_some() {
                return Err(invalid("ambiguous amplification level").into());
            }
        }
    }
    Ok(steps.into_values().collect())
}
fn stat_name(code: &str) -> String {
    match code.strip_prefix("EStat::").unwrap_or(code) {
        "WeaponMinDamage" | "WeaponDamage" => "Attack".into(),
        "WeaponAccuracy" => "Accuracy".into(),
        "Critical" => "Critical Hit".into(),
        "MaxWeaponBlock" => "Parry Damage Reduction Amount".into(),
        "DecreaseWeaponBlock" => "Parry Damage Reduction Rate".into(),
        "WeaponFixingDamage" => "Attack".into(),
        "AmplifyWeaponDamage" => "Damage Boost".into(),
        other => {
            let mut text = String::new();
            for c in other.chars() {
                if c.is_uppercase() && !text.is_empty() {
                    text.push(' ');
                }
                text.push(c);
            }
            text
        }
    }
}
