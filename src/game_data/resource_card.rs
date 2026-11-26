//! Read-only inspection of the additional projected catalogue families.
use super::*;
use rusqlite::OptionalExtension;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceFamily {
    key: &'static str,
    label: &'static str,
    table: &'static str,
    source: &'static str,
}
impl ResourceFamily {
    const fn new(
        key: &'static str,
        label: &'static str,
        table: &'static str,
        source: &'static str,
    ) -> Self {
        Self {
            key,
            label,
            table,
            source,
        }
    }
    pub const ALL: &'static [Self] = &[
        Self::new(
            "craft_recipes",
            "Recettes de fabrication",
            "recipes",
            "CraftRecipe",
        ),
        Self::new(
            "conversion_recipes",
            "Recettes de conversion",
            "recipes",
            "ConversionRecipe",
        ),
        Self::new("quests", "Quêtes", "quests", "Quest"),
        Self::new(
            "achievements",
            "Exploits et collections",
            "achievements",
            "Achievement",
        ),
        Self::new(
            "achievement_groups",
            "Groupes d’exploits",
            "achievement_groups",
            "AchievementGroup",
        ),
        Self::new("titles", "Titres", "titles", "Title"),
        Self::new("battle_passes", "Pass Daeva", "battle_passes", "BattlePass"),
        Self::new(
            "battle_pass_groups",
            "Groupes de pass",
            "battle_pass_groups",
            "BattlePassGroup",
        ),
        Self::new(
            "event_missions",
            "Missions d’événement",
            "event_missions",
            "EventMission",
        ),
        Self::new("seasons", "Saisons", "seasons", "SeasonSchedule"),
        Self::new(
            "season_schedule_groups",
            "Groupes de calendrier",
            "season_schedule_groups",
            "SeasonScheduleGroup",
        ),
        Self::new(
            "season_contents",
            "Contenus saisonniers",
            "season_contents",
            "SeasonContents",
        ),
        Self::new(
            "supply_requests",
            "Requêtes de ravitaillement",
            "supply_requests",
            "PeriodItemCollectionList",
        ),
        Self::new(
            "supply_request_groups",
            "Variantes de ravitaillement",
            "supply_request_groups",
            "PeriodItemCollection",
        ),
        Self::new("journals", "Journaux", "journals", "Journal"),
        Self::new("loot_tables", "Tables de butin", "loot_tables", ""),
        Self::new("pets", "Familiers", "pets", "VehicleList"),
        Self::new(
            "pet_family_levels",
            "Niveaux des familles de familiers",
            "pet_family_levels",
            "VehicleCreatureLevel",
        ),
        Self::new(
            "pet_options",
            "Options de familiers",
            "pet_options",
            "VehicleCreatureOption",
        ),
        Self::new(
            "pet_stat_levels",
            "Statistiques des familiers par niveau",
            "pet_stat_levels",
            "VehicleStat",
        ),
        Self::new(
            "pet_collections",
            "Collections de familiers",
            "pet_collections",
            "VehicleCollection",
        ),
        Self::new("wings", "Ailes", "wings", "Wing"),
        Self::new("skins", "Apparences", "skins", "Skin"),
        Self::new(
            "skin_sets",
            "Ensembles d’apparences",
            "skin_sets",
            "SkinSet",
        ),
        Self::new(
            "skin_gacha_results",
            "Tirages d’apparences",
            "skin_gacha_results",
            "SkinGacha",
        ),
        Self::new(
            "daevanion_boards",
            "Plateaux Daevanion",
            "daevanion_boards",
            "DaevanionBoard",
        ),
        Self::new(
            "daevanion_nodes",
            "Cases Daevanion",
            "daevanion_nodes",
            "DaevanionNode",
        ),
    ];
    pub fn key(self) -> &'static str {
        self.key
    }
    pub fn label(self) -> &'static str {
        self.label
    }
    pub(super) fn table(self) -> &'static str {
        self.table
    }
    pub(super) fn source(self) -> &'static str {
        self.source
    }
    pub(super) fn scope(self) -> &'static str {
        match self.key {
            "craft_recipes" => "source_name='CraftRecipe'",
            "conversion_recipes" => "source_name='ConversionRecipe'",
            _ => "1=1",
        }
    }
    pub(super) fn predicate(self) -> String {
        format!("{} AND id=?", self.scope())
    }
}

fn display_column(cols: &[(String, String)], localized: bool) -> Option<&str> {
    let candidates = if localized {
        &["name_localized", "name_light_localized"][..]
    } else {
        &["name_en", "name_light_en"][..]
    };
    candidates.iter().find_map(|name| {
        cols.iter()
            .find(|(c, _)| c == name)
            .map(|(c, _)| c.as_str())
    })
}
pub(super) fn listing_query(db: &Connection, family: ResourceFamily) -> Result<String> {
    let cols = columns(db, family.table)?;
    let mut en = Vec::new();
    let mut loc = Vec::new();
    if family.key == "skins" {
        en.push("(SELECT name_en FROM skin_variants WHERE skin_id=t.id ORDER BY race_id,sex_id LIMIT 1)".into());
        loc.push("(SELECT name_localized FROM skin_variants WHERE skin_id=t.id ORDER BY race_id,sex_id LIMIT 1)".into());
    }
    for (out, localized) in [(&mut en, false), (&mut loc, true)] {
        if let Some(name) = display_column(&cols, localized) {
            out.push(format!("t.{}", quote(name)));
        }
        // Options without a title are searchable by their actual stat label.
        if family.key == "pet_options" {
            out.push(format!(
                "(SELECT {} FROM stats WHERE id=t.stat_id)",
                if localized {
                    "name_localized"
                } else {
                    "name_en"
                }
            ));
        }
    }
    let mut technical = Vec::new();
    for candidate in ["source_key", "group_reference", "source_name"] {
        if cols.iter().any(|(c, _)| c == candidate) {
            technical.push(format!("NULLIF(t.{},'')", quote(candidate)));
        }
    }
    en.extend(technical.iter().cloned());
    en.push("'#' || t.id".into());
    loc.extend(en.iter().cloned());
    let aliases: Vec<_> = cols
        .iter()
        .filter(|(name, ty)| {
            ty == "TEXT"
                && (name.starts_with("name_")
                    || name.ends_with("_reference")
                    || name == "source_key"
                    || name == "source_name")
        })
        .map(|(name, _)| format!("COALESCE(t.{},'')", quote(name)))
        .collect();
    let aliases = if aliases.is_empty() {
        "''".into()
    } else {
        aliases.join(" || ' ' || ")
    };
    Ok(format!(
        "SELECT t.id AS id,COALESCE({},NULL) AS localized,COALESCE({},NULL) AS english,{aliases} AS aliases FROM {} t WHERE {} ORDER BY t.id",
        loc.join(","),
        en.join(","),
        quote(family.table),
        family.scope()
    ))
}

fn as_text(value: &Value) -> String {
    match value {
        Value::Null => "—".into(),
        Value::Text(s) => s.clone(),
        Value::Integer(n) => n.to_string(),
        Value::Real(n) => n.to_string(),
        Value::Blob(_) => "[image]".into(),
    }
}
fn short(value: impl AsRef<str>) -> String {
    let s = value.as_ref();
    let mut chars = s.chars();
    let prefix: String = chars.by_ref().take(2048).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}
fn name(row: &Record, localized: bool) -> Option<String> {
    let candidates = if localized {
        &[
            "name_localized",
            "name_light_localized",
            "name_en",
            "name_light_en",
        ][..]
    } else {
        &["name_en", "name_light_en"][..]
    };
    candidates
        .iter()
        .find_map(|key| {
            row.get(*key)
                .filter(|v| !matches!(v, Value::Null))
                .map(as_text)
        })
        .or_else(|| {
            [
                "code",
                "source_key",
                "steam_build",
                "source_name",
                "item_reference",
                "group_reference",
            ]
            .iter()
            .find_map(|key| {
                row.get(*key)
                    .filter(|v| !matches!(v, Value::Null))
                    .map(as_text)
            })
        })
}
fn label(key: &str) -> (&'static str, &'static str) {
    match key {
        "client_version_id" => ("Last changed in build", "Dernier build de modification"),
        "required_level" | "level" => ("Level", "Niveau"),
        "grade_id" | "rarity_id" => ("Rarity", "Rareté"),
        "type_id" => ("Type", "Type"),
        "category_id" => ("Category", "Catégorie"),
        "family_id" => ("Family", "Famille"),
        "class_id" => ("Class", "Classe"),
        "race_id" => ("Race", "Race"),
        "group_id" | "set_id" => ("Group", "Groupe"),
        "item_id" | "reward_item_id" => ("Item", "Objet"),
        "skill_id" => ("Skill", "Compétence"),
        "level_bonus" => (
            "Additional skill levels",
            "Niveaux de compétence supplémentaires",
        ),
        "neighbour_id" => ("Adjacent node", "Case adjacente"),
        "stat_id" => ("Stat", "Statistique"),
        "board_id" => ("Board", "Plateau"),
        "row_number" => ("Row", "Ligne"),
        "column_number" => ("Column", "Colonne"),
        "point_cost_raw" => ("Point cost (raw)", "Coût en points (brut)"),
        "reset_cost_raw" => ("Reset cost (raw)", "Coût de réinitialisation (brut)"),
        "value_raw" | "value_integer_raw" => ("Value (raw)", "Valeur (brute)"),
        "weight_raw" => ("Weight (raw)", "Poids (brut)"),
        "interpretation_state" => ("Interpretation", "Interprétation"),
        "resolution_state" => ("Reference state", "État de la référence"),
        "auto_learn" => (
            "Automatic learning (source)",
            "Apprentissage automatique (source)",
        ),
        "description_en" => ("Description", "Description"),
        "name_en" => ("Name", "Nom"),
        _ => ("Field", "Champ"),
    }
}
fn fields(
    db: &Connection,
    table: &str,
    row: &Record,
    skip: &BTreeSet<String>,
) -> Result<Vec<CardField>> {
    let fks = foreign_keys(db, table)?;
    let types: BTreeMap<_, _> = columns(db, table)?.into_iter().collect();
    let locale: String = db.query_row(
        "SELECT locale FROM catalogue_manifest WHERE id=1",
        [],
        |r| r.get(0),
    )?;
    let mut fields = Vec::new();
    for (key, value) in row {
        let localized_key = key
            .strip_suffix("_en")
            .map(|prefix| format!("{prefix}_localized"));
        let localized_value = localized_key
            .as_ref()
            .and_then(|key| row.get(key))
            .filter(|v| !matches!(v, Value::Null));
        if skip.contains(key)
            || key.ends_with("_localized")
            || (matches!(value, Value::Null) && localized_value.is_none())
        {
            continue;
        }
        let (en, loc) = label(key);
        let mut english = as_text(value);
        let mut localized = None;
        if key.ends_with("_en") {
            localized = row
                .get(&format!("{}_localized", key.trim_end_matches("_en")))
                .filter(|v| !matches!(v, Value::Null))
                .map(as_text);
        }
        if types.get(key).is_some_and(|ty| ty == "BOOLEAN") {
            english = if *value == Value::Integer(0) {
                "No"
            } else {
                "Yes"
            }
            .into();
            localized = Some(
                if *value == Value::Integer(0) {
                    "Non"
                } else {
                    "Oui"
                }
                .into(),
            );
        }
        if let Some(fk) = fks
            .iter()
            .find(|fk| fk.fields.len() == 1 && fk.fields[0].0 == *key)
        {
            let predicate = format!("{}=?", quote(&fk.fields[0].1));
            if let Some(target) =
                records(db, &fk.table, &predicate, std::slice::from_ref(value), 1)?.first()
            {
                if let Some(n) = name(target, false) {
                    english = format!("{} · {n}", as_text(value));
                }
                localized = name(target, true)
                    .or_else(|| name(target, false))
                    .map(|n| format!("{} · {n}", as_text(value)));
            }
        }
        let numeric = key != "id"
            && matches!(value, Value::Integer(_) | Value::Real(_))
            && !key.ends_with("_id")
            && !key.ends_with("_reference")
            && !key.ends_with("_raw")
            && !key.ends_with("_row")
            && !fks
                .iter()
                .any(|fk| fk.fields.iter().any(|(from, _)| from == key))
            && !types.get(key).is_some_and(|ty| ty == "BOOLEAN");
        if numeric {
            english = display_numbers::number(
                "en-US",
                as_text(value),
                display_numbers::Presentation::Decimal(12),
            );
            localized = Some(display_numbers::number(
                &locale,
                as_text(value),
                display_numbers::Presentation::Decimal(12),
            ));
        }
        if en == "Field" {
            let fallback = english.clone();
            english = format!("{key} : {english}");
            localized = Some(format!("{key} : {}", localized.unwrap_or(fallback)));
        }
        fields.push(CardField {
            name_en: en,
            name_localized: loc,
            value: short(english),
            value_localized: localized.map(short),
            numeric: false,
        });
    }
    Ok(fields)
}
fn card(
    db: &Connection,
    table: &str,
    row: &Record,
    id: i64,
    tag: String,
    skip: BTreeSet<String>,
) -> Result<DefinitionCard> {
    Ok(DefinitionCard {
        id,
        name_en: name(row, false),
        name_localized: name(row, true),
        tag,
        icon: None,
        icon_mime: None,
        levels: vec![CardLevel {
            level: 0,
            description_en: row
                .get("description_en")
                .filter(|v| !matches!(v, Value::Null))
                .map(as_text),
            description_localized: row
                .get("description_localized")
                .filter(|v| !matches!(v, Value::Null))
                .map(as_text),
            sections: vec![CardSection {
                truncated: false,
                name_en: "DETAILS",
                name_localized: "DÉTAILS",
                fields: fields(db, table, row, &skip)?,
            }],
        }],
        related: vec![],
        related_truncated: false,
    })
}
pub(super) fn load(db: &Connection, family: ResourceFamily, id: i64) -> Result<DefinitionCard> {
    let row = records(
        db,
        family.table,
        &family.predicate(),
        &[Value::Integer(id)],
        1,
    )?
    .pop()
    .ok_or_else(|| invalid("Ressource absente du catalogue ; actualisez la liste."))?;
    let skip = BTreeSet::from(["id".into(), "name_en".into(), "description_en".into()]);
    let mut root = card(
        db,
        family.table,
        &row,
        id,
        format!("{} · #{id}", family.label),
        skip,
    )?;
    let names: (String, String) = db.query_row(
        &format!(
            "SELECT localized,english FROM ({}) AS names WHERE id=?1",
            listing_query(db, family)?
        ),
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    root.name_en = Some(names.1);
    root.name_localized = Some(names.0);
    if let Some((image,mime))=db.query_row("SELECT i.image,i.mime_type FROM catalogue_entity_icons e JOIN catalogue_icons i ON i.id=e.icon_id WHERE e.family=?1 AND e.source_id=?2",rusqlite::params![family.key,id],|r|Ok((r.get(0)?,r.get(1)?))).optional()? {root.icon=image;root.icon_mime=mime;}
    if family.key == "skins" && root.icon.is_none() {
        let image: Option<(Option<Vec<u8>>, Option<String>)> = db.query_row(
            "SELECT i.image,i.mime_type FROM (SELECT icon_id FROM skin_variants WHERE skin_id=?1 ORDER BY race_id,sex_id LIMIT 1) v JOIN catalogue_icons i ON i.id=v.icon_id",
            [id], |r| Ok((r.get(0)?,r.get(1)?))
        ).optional()?;
        if let Some((image, mime)) = image {
            root.icon = image;
            root.icon_mime = mime;
        }
    }
    let tables: Vec<String> = db
        .prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for table in tables {
        for fk in foreign_keys(db, &table)?
            .into_iter()
            .filter(|f| f.table == family.table)
        {
            let predicate = fk
                .fields
                .iter()
                .map(|(from, _)| format!("{}=?", quote(from)))
                .collect::<Vec<_>>()
                .join(" AND ");
            let args: Vec<_> = fk
                .fields
                .iter()
                .map(|(_, to)| row.get(to).cloned().unwrap_or(Value::Null))
                .collect();
            let remaining = RELATION_LIMIT.saturating_sub(root.related.len());
            let related = records(db, &table, &predicate, &args, remaining + 1)?;
            if related.len() > remaining {
                root.related_truncated = true;
            }
            for (i, child) in related.into_iter().take(remaining).enumerate() {
                let child_id = child
                    .get("id")
                    .and_then(|v| {
                        if let Value::Integer(n) = v {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(i as i64 + 1);
                let skip = fk
                    .fields
                    .iter()
                    .map(|(from, _)| from.clone())
                    .chain(["id".into(), "name_en".into(), "description_en".into()])
                    .collect();
                root.related
                    .push(card(db, &table, &child, child_id, table.clone(), skip)?);
            }
        }
    }
    // Polymorphic evidence/cost/item links have no artificial SQL FK to every root.
    for table in [
        "catalogue_item_links",
        "catalogue_costs",
        "catalogue_periods",
        "catalogue_conditions",
        "catalogue_references",
        "appearance_stats",
        "catalogue_text_values",
        "mission_group_sets",
    ] {
        let remaining = RELATION_LIMIT.saturating_sub(root.related.len());
        let children = records(
            db,
            table,
            "family=? AND source_id=?",
            &[Value::Text(family.key.into()), Value::Integer(id)],
            remaining + 1,
        )?;
        if children.len() > remaining {
            root.related_truncated = true;
        }
        for (i, child) in children.into_iter().take(remaining).enumerate() {
            root.related.push(card(
                db,
                table,
                &child,
                i as i64 + 1,
                table.into(),
                BTreeSet::from(["family".into(), "source_id".into()]),
            )?);
        }
    }
    // These owned rules sit behind groups or steps rather than a direct root FK.
    // Use only explicit links; in particular a wing's grade cannot select its enchant group.
    let extra: &[(&str, &str)] = match family.key {
        "quests" => &[
            (
                "quest_objectives",
                "step_id IN(SELECT id FROM quest_steps WHERE quest_id=?)",
            ),
            (
                "quest_rewards",
                "(source_name,source_row) IN(SELECT source_name,source_row FROM quest_reward_groups WHERE quest_id=?)",
            ),
            (
                "catalogue_conditions",
                "family='quest_steps' AND source_id IN(SELECT id FROM quest_steps WHERE quest_id=?)",
            ),
        ],
        "achievements" => &[(
            "catalogue_item_links",
            "family='achievement_rewards' AND source_id IN(SELECT id FROM achievement_rewards WHERE achievement_id=?)",
        )],
        "pets" => &[
            (
                "pet_families",
                "id IN(SELECT family_id FROM pets WHERE id=?)",
            ),
            (
                "pet_family_slots",
                "family_id IN(SELECT family_id FROM pets WHERE id=?)",
            ),
            (
                "pet_family_lock_costs",
                "family_id IN(SELECT family_id FROM pets WHERE id=?)",
            ),
            (
                "pet_stat_values",
                "level_id IN(SELECT l.id FROM pet_stat_levels l JOIN pets p ON p.stat_group_id=l.group_id WHERE p.id=?)",
            ),
        ],
        "wings" => &[
            (
                "wing_effect_stats",
                "group_id IN(SELECT effect_group_id FROM wings WHERE id=?)",
            ),
            (
                "wing_enchant_levels",
                "group_id IN(SELECT enchant_group_id FROM wings WHERE id=?)",
            ),
            (
                "wing_enchant_costs",
                "group_id IN(SELECT enchant_group_id FROM wings WHERE id=?)",
            ),
        ],
        "loot_tables" => &[
            (
                "item_rewards",
                "(source_name,group_reference) IN(SELECT source_name,source_key FROM loot_tables WHERE id=?)",
            ),
            (
                "reward_pool_entries",
                "pool_id IN(SELECT id FROM reward_pools WHERE loot_table_id=?)",
            ),
            (
                "dungeon_reward_rolls",
                "pool_id IN(SELECT id FROM reward_pools WHERE loot_table_id=?)",
            ),
        ],
        "craft_recipes" => &[(
            "recipe_rates",
            "group_reference IN(SELECT rate_group_reference FROM recipes WHERE source_name='CraftRecipe' AND id=?)",
        )],
        "conversion_recipes" => &[(
            "recipe_rates",
            "group_reference IN(SELECT rate_group_reference FROM recipes WHERE source_name='ConversionRecipe' AND id=?)",
        )],
        _ => &[],
    };
    for (table, predicate) in extra {
        append(db, &mut root, table, predicate, &[Value::Integer(id)])?;
    }
    Ok(root)
}
fn append(
    db: &Connection,
    root: &mut DefinitionCard,
    table: &str,
    predicate: &str,
    args: &[Value],
) -> Result<()> {
    let remaining = RELATION_LIMIT.saturating_sub(root.related.len());
    let rows = records(db, table, predicate, args, remaining + 1)?;
    root.related_truncated |= rows.len() > remaining;
    for (i, row) in rows.into_iter().take(remaining).enumerate() {
        let id = match row.get("id") {
            Some(Value::Integer(id)) => *id,
            _ => i as i64 + 1,
        };
        root.related.push(card(
            db,
            table,
            &row,
            id,
            table.into(),
            BTreeSet::from(["id".into(), "name_en".into(), "description_en".into()]),
        )?);
    }
    Ok(())
}

// Loot identities predate catalogue_entity_sources and use natural keys, not a
// numeric source ID. Expand the selected definition only in the opt-in view.
pub(super) fn loot_sources(db: &Connection, row: &Record, lines: &mut Vec<String>) -> Result<()> {
    let (Some(Value::Text(source)), Some(Value::Text(key))) =
        (row.get("source_name"), row.get("source_key"))
    else {
        return Ok(());
    };
    let field = match source.as_str() {
        "NpcLoot" => "NpcId",
        "RewardItem" | "RewardPackage" | "RewardOdEnergyCube" | "RewardDecomposePackage" => "Name",
        "RewardPackageItem" => "PackageName",
        "BoxReward" => "Group",
        "RewardData" | "DungeonRewardInfo" | "RewardLevelScaleData" | "Dungeon" => "ID",
        _ => return Ok(()),
    };
    let rows = target_export::source_rows(db, source)?;
    let mut count = 0;
    for (pos, row) in rows.iter().enumerate() {
        let matches = target_export::string(&row[field]).is_some_and(|value| value == key)
            || target_export::number(row, field).is_some_and(|value| value.to_string() == *key);
        if matches {
            if count == RELATION_LIMIT {
                lines.push(format!(
                    "Sources limitées aux {RELATION_LIMIT} premières variantes."
                ));
                break;
            }
            lines.push(format!("\n━━ Source liée {source} · ligne {pos} ━━"));
            lines.extend(
                serde_json::to_string_pretty(row)?
                    .lines()
                    .map(str::to_owned),
            );
            count += 1;
        }
    }
    Ok(())
}
