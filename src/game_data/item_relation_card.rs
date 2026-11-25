//! Read-only, aggregated relationships for the item inspection panel.
use super::*;

#[derive(Debug)]
pub struct DungeonReward {
    pub pool_code: Option<String>,
    pub position: i64,
    pub guaranteed_draws: i64,
    pub extra_draw_percent: f64,
    pub pool_quantity: Option<i64>,
    pub race: String,
}
#[derive(Debug)]
pub struct ItemRelation {
    pub kind: String,
    pub name_en: String,
    pub name_localized: Option<String>,
    pub related_item_id: Option<i64>,
    pub quantity_min: Option<i64>,
    pub quantity_max: Option<i64>,
    pub weight_raw: Option<String>,
    pub probability_percent: Option<f64>,
    pub dungeon: Option<DungeonReward>,
}
impl ItemRelation {
    pub fn name(&self, english: bool) -> &str {
        if english {
            &self.name_en
        } else {
            self.name_localized.as_deref().unwrap_or(&self.name_en)
        }
    }
    pub fn quantity(&self) -> String {
        match (self.quantity_min, self.quantity_max) {
            (Some(min), Some(max)) if min != max => format!("×{min}–{max}"),
            (Some(n), _) if n > 0 => format!("×{n}"),
            _ => String::new(),
        }
    }
    pub fn shows_probability(&self) -> bool {
        self.weight_raw.is_some()
            || self.probability_percent.is_some()
            || matches!(
                self.kind.as_str(),
                "MonsterGetRoutes"
                    | "RewardBoxInfo"
                    | "RewardDefinition"
                    | "RewardPackageDefinition"
                    | "RaidInfo"
                    | "PartyDungeonGetRoutes"
                    | "DungeonReward"
            )
    }
    pub fn probability(&self, english: bool) -> String {
        if let Some(percent) = self.probability_percent {
            let number = format!("{percent:.6}");
            let percent = format!("{} %", number.trim_end_matches('0').trim_end_matches('.'));
            if self.dungeon.as_ref().is_some_and(|d| d.pool_code.is_some()) {
                if english {
                    format!("Within pool: {percent}")
                } else {
                    format!("Dans le pool : {percent}")
                }
            } else {
                percent
            }
        } else if self.dungeon.as_ref().is_some_and(|d| d.pool_code.is_none()) {
            if english {
                "Direct reward"
            } else {
                "Récompense directe"
            }
            .into()
        } else if let Some(weight) = &self.weight_raw {
            if english {
                format!("Rate unknown · weight {weight}")
            } else {
                format!("Taux inconnu · poids {weight}")
            }
        } else {
            if english {
                "Rate not provided"
            } else {
                "Taux non renseigné"
            }
            .into()
        }
    }
    pub fn allocation(&self, english: bool) -> Option<String> {
        let d = self.dungeon.as_ref()?;
        let mut label = if d.guaranteed_draws == 0 {
            String::new()
        } else if english {
            format!(
                "{} {}",
                d.guaranteed_draws,
                if d.guaranteed_draws == 1 {
                    "guaranteed draw"
                } else {
                    "guaranteed draws"
                }
            )
        } else {
            format!(
                "{} {}",
                d.guaranteed_draws,
                if d.guaranteed_draws == 1 {
                    "tirage garanti"
                } else {
                    "tirages garantis"
                }
            )
        };
        if d.extra_draw_percent > 0.0 {
            let percent = format!("{:.6}", d.extra_draw_percent);
            let percent = percent.trim_end_matches('0').trim_end_matches('.');
            if !label.is_empty() {
                label.push_str(" + ");
            }
            label.push_str(&if english {
                format!("1 draw at {percent} %")
            } else {
                format!("1 tirage à {percent} %")
            });
        }
        if d.pool_quantity.is_some_and(|q| q > 1) {
            label.push_str(&format!(" · ×{}", d.pool_quantity.unwrap()));
        }
        if d.race != "ERace::All" {
            label.push_str(match d.race.as_str() {
                "ERace::Light" => " · Elyos",
                "ERace::Dark" => " · Asmodian",
                _ => "",
            });
        }
        Some(label)
    }
    pub fn pool_label(&self, english: bool) -> Option<String> {
        let d = self.dungeon.as_ref()?;
        d.pool_code
            .as_ref()
            .map(|code| format!("Pool {} · {code}", d.position + 1))
            .or_else(|| {
                Some(
                    if english {
                        "Independent allocation"
                    } else {
                        "Attribution indépendante"
                    }
                    .into(),
                )
            })
    }
    pub fn kind_label(&self, english: bool) -> &str {
        let (en, fr) = match self.kind.as_str() {
            "DungeonRewardPool" | "DungeonDirectReward" => ("Dungeon loot", "Butin de donjon"),
            "MonsterGetRoutes" => ("Monster", "Monstre"),
            "CraftGetRoutes" | "CraftRecipe" => ("Crafting", "Fabrication"),
            "ConversionRecipeGetRoutes" | "ConversionRecipe" => ("Conversion", "Conversion"),
            "GatherGetRoutes" => ("Gathering", "Collecte"),
            "QuestGetRoutes" => ("Quest", "Quête"),
            "AchievementInfo" => ("Achievement", "Exploit"),
            "PartyDungeonGetRoutes" | "TrialDungeonGetRoutes" | "DungeonReward" => {
                ("Dungeon", "Donjon")
            }
            "RaidInfo" => ("Raid", "Raid"),
            "NPCShopInfo" | "CoinShopInfo" | "GameShopGoods" => ("Shop", "Boutique"),
            "BMShopInfo" => ("Cash shop", "Boutique commerciale"),
            "RandomShopInfo" => ("Random shop", "Boutique aléatoire"),
            "RewardBoxInfo" => ("Chest", "Coffre"),
            "RewardDefinition" => ("Reward definition", "Définition de récompense"),
            "RewardPackageDefinition" => ("Reward package", "Package de récompense"),
            "FieldEventInfo" | "FieldEvent" => ("Event", "Événement"),
            "BattlePassInfo" => ("Battle pass", "Pass de combat"),
            "AttendanceInfo" => ("Attendance", "Connexion"),
            "DailyInfo" => ("Daily activity", "Activité journalière"),
            "AwakenInfo" => ("Awakening", "Éveil"),
            "SuppressionInfo" => ("Suppression", "Suppression"),
            "ExplorationSealInfo" => ("Exploration seals", "Sceaux d’exploration"),
            "ExplorationGarrisionInfo" => ("Exploration garrisons", "Garnisons d’exploration"),
            "SubscribeInfo" | "Subscribe" => ("Subscription", "Abonnement"),
            "MembershipPassInfo" => ("Membership pass", "Pass d’abonnement"),
            "Manual" => ("Client information", "Indication du client"),
            "BattlePass" => ("Battle pass", "Pass de combat"),
            "PeriodCollectionInfo" | "PeriodItemCollectionList" => ("Collection", "Collection"),
            "RankingInfo" => ("Ranking", "Classement"),
            "MonolithInfo" => ("Monolith", "Monolithe"),
            "AscensionInfo" => ("Ascension", "Ascension"),
            "BossChallengeInfo" => ("Boss challenge", "Défi de boss"),
            "SkinInfo" => ("Appearance", "Apparence"),
            "HiddenCubeInfo" => ("Hidden cube", "Cube caché"),
            "DecomposeItemInfos" => ("Decomposition", "Décomposition"),
            "bArena" => ("Arena", "Arène"),
            "bVehicle" | "VehicleCollection" => ("Mount collection", "Collection de montures"),
            "ArcanaCreation" | "ArcanaCreate" => ("Arcana creation", "Création d’arcana"),
            "ItemSoulAdd" => ("Philosopher’s stone", "Pierre philosophale"),
            "ItemSuccession" => ("Succession", "Succession"),
            "ItemSurpass" | "ExceedEnchant" => ("Amplification", "Percée"),
            "Enchant" | "ArcanaEnchant" => ("Enchantment", "Enchantement"),
            "ContentsSweepTicket" => ("Sweep ticket", "Ticket d’accomplissement"),
            _ => return &self.kind,
        };
        if english { en } else { fr }
    }
}
pub const RELATION_PAGE_SIZE: usize = 50;
#[derive(Debug)]
pub struct RelationPage {
    pub entries: Vec<ItemRelation>,
    pub total: usize,
}
// Count and paginate in SQLite. A currency can have tens of thousands of
// relationships; never materialize its complete result set in the UI worker.
const RELATIONS: &str = "WITH display_relations AS (
 SELECT 'DungeonRewardPool' AS kind,r.name_en,r.name_localized,NULL AS related_item_id,MIN(e.quantity_min) AS quantity_min,MAX(e.quantity_max) AS quantity_max,CAST(SUM(e.weight) AS TEXT) AS weight_raw,SUM(e.probability_percent) AS probability_percent,p.code AS pool_code,r.guaranteed_draws,r.extra_draw_percent,r.quantity_min AS pool_quantity,r.race,r.source_path,0 AS priority
 FROM dungeon_reward_rolls r JOIN reward_pools p ON p.id=r.pool_id JOIN reward_pool_entries e ON e.pool_id=p.id WHERE ?2='source' AND e.item_id=?1 AND r.allocation_raw>0 GROUP BY r.id
 UNION ALL SELECT 'DungeonDirectReward',r.name_en,r.name_localized,NULL,r.quantity_min,r.quantity_max,CAST(r.allocation_raw AS TEXT),NULL,NULL,r.guaranteed_draws,r.extra_draw_percent,NULL,r.race,r.source_path,0 FROM dungeon_reward_rolls r WHERE ?2='source' AND r.item_id=?1 AND r.allocation_raw>0
 UNION ALL SELECT kind,name_en,name_localized,related_item_id,quantity_min,quantity_max,weight_raw,probability_percent,NULL,NULL,NULL,NULL,NULL,NULL,1 FROM item_relations b WHERE item_id=?1 AND direction=?2 AND NOT (?2='source' AND ((source_name='RewardPackageItem' AND EXISTS(SELECT 1 FROM reward_pools p JOIN dungeon_reward_rolls r ON r.pool_id=p.id WHERE p.source_row=b.source_row AND r.allocation_raw>0)) OR (source_name='RewardItem' AND EXISTS(SELECT 1 FROM dungeon_reward_rolls r WHERE r.source_name='RewardItem' AND r.source_row=b.source_row AND r.item_id=b.item_id AND r.allocation_raw>0)))) GROUP BY kind,name_en,name_localized,related_item_id,quantity_min,quantity_max,weight_raw,probability_percent
)";
pub(super) fn page(
    db: &Connection,
    id: i64,
    direction: &str,
    offset: usize,
) -> Result<RelationPage> {
    let sql = format!(
        "{RELATIONS} SELECT *,COUNT(*) OVER() FROM display_relations ORDER BY priority,COALESCE(name_localized,name_en),kind,pool_code,source_path,related_item_id,quantity_min,quantity_max,weight_raw,probability_percent LIMIT ?3 OFFSET ?4"
    );
    let mut total = 0;
    let mut entries = Vec::with_capacity(RELATION_PAGE_SIZE);
    let mut statement = db.prepare(&sql)?;
    let mut rows = statement.query(rusqlite::params![
        id,
        direction,
        RELATION_PAGE_SIZE as i64,
        i64::try_from(offset)?
    ])?;
    while let Some(r) = rows.next()? {
        total = r.get::<_, i64>(15)? as usize;
        let dungeon = if let Some(guaranteed_draws) = r.get::<_, Option<i64>>(9)? {
            let path: String = r.get(13)?;
            Some(DungeonReward {
                pool_code: r.get(8)?,
                position: path
                    .rsplit('/')
                    .next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(0),
                guaranteed_draws,
                extra_draw_percent: r.get(10)?,
                pool_quantity: r.get(11)?,
                race: r.get(12)?,
            })
        } else {
            None
        };
        entries.push(ItemRelation {
            kind: r.get(0)?,
            name_en: r.get(1)?,
            name_localized: r.get(2)?,
            related_item_id: r.get(3)?,
            quantity_min: r.get(4)?,
            quantity_max: r.get(5)?,
            weight_raw: r.get(6)?,
            probability_percent: r.get::<_, Option<f64>>(7)?.map(|p| p.clamp(0.0, 100.0)),
            dungeon,
        });
    }
    // A stale page offset must not incorrectly report an empty catalogue.
    if entries.is_empty() && offset > 0 {
        total = db.query_row(
            &format!("{RELATIONS} SELECT COUNT(*) FROM display_relations"),
            rusqlite::params![id, direction],
            |r| r.get::<_, i64>(0),
        )? as usize;
    }
    Ok(RelationPage { entries, total })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relations_are_counted_deduplicated_and_paginated_in_the_database() {
        let mut db = Connection::open_in_memory().unwrap();
        db.execute_batch(include_str!("items.sql")).unwrap();
        db.execute_batch(include_str!("catalogue.sql")).unwrap();
        db.pragma_update(None, "foreign_keys", false).unwrap();
        let tx = db.transaction().unwrap();
        for row in 0..123 {
            tx.execute(
                "INSERT INTO item_relations VALUES(1,'source','QuestGetRoutes','Quest',?1,'reward',?2,NULL,NULL,1,1,NULL,'not_provided',NULL)",
                rusqlite::params![row, format!("Quest {row:03}")],
            ).unwrap();
        }
        // A second source occurrence must not inflate the displayed count.
        tx.execute("INSERT INTO item_relations SELECT item_id,direction,kind,source_name,999,source_path,name_en,name_localized,related_item_id,quantity_min,quantity_max,weight_raw,probability_state,probability_percent FROM item_relations WHERE source_row=0", []).unwrap();
        tx.commit().unwrap();
        let first = page(&db, 1, "source", 0).unwrap();
        let second = page(&db, 1, "source", RELATION_PAGE_SIZE).unwrap();
        let last = page(&db, 1, "source", 2 * RELATION_PAGE_SIZE).unwrap();
        assert_eq!(first.total, 123);
        assert_eq!(second.total, 123);
        assert_eq!(last.total, 123);
        assert_eq!(first.entries.len(), RELATION_PAGE_SIZE);
        assert_eq!(second.entries.len(), RELATION_PAGE_SIZE);
        assert_eq!(last.entries.len(), 23);
        assert_eq!(first.entries[49].name_en, "Quest 049");
        assert_eq!(second.entries[0].name_en, "Quest 050");
        assert_eq!(last.entries.last().unwrap().name_en, "Quest 122");
        let beyond = page(&db, 1, "source", 1000).unwrap();
        assert_eq!(beyond.total, 123);
        assert!(beyond.entries.is_empty());
        assert_eq!(page(&db, 1, "use", 0).unwrap().total, 0);
    }
    #[test]
    fn unknown_weights_are_never_presented_as_drop_percentages() {
        let mut r = ItemRelation {
            kind: "RewardPackageDefinition".into(),
            name_en: "Reward".into(),
            name_localized: Some("Récompense".into()),
            related_item_id: None,
            quantity_min: Some(1),
            quantity_max: Some(3),
            weight_raw: Some("100000000".into()),
            probability_percent: None,
            dungeon: None,
        };
        assert_eq!(r.probability(false), "Taux inconnu · poids 100000000");
        assert_eq!(r.quantity(), "×1–3");
        assert_eq!(r.name(false), "Récompense");
        r.probability_percent = Some(12.125);
        assert_eq!(r.probability(true), "12.125 %");
        r.probability_percent = Some(0.0);
        assert_eq!(r.probability(true), "0 %");
    }
}

#[cfg(test)]
mod dungeon_tests {
    use super::*;
    #[test]
    fn card_sums_repeated_item_weights_and_keeps_guaranteed_and_bonus_rolls_separate() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(include_str!("items.sql")).unwrap();
        db.execute_batch(include_str!("catalogue.sql")).unwrap();
        db.pragma_update(None, "foreign_keys", false).unwrap();
        db.execute_batch("INSERT INTO reward_pools VALUES(1,'pool',8,0);
          INSERT INTO reward_pool_entries VALUES(1,0,'item',1,1,12.5,1,1),(1,1,'item',1,1,12.5,2,2),(1,2,'other',NULL,6,75,1,1);
          INSERT INTO dungeon_reward_rolls VALUES(1,600082,'Ludra','Ludra','Normal','boss','ERace::All',1,NULL,450000000,4,50,1,1,'RewardPackage',0,'PackageItems/3');").unwrap();
        let card = page(&db, 1, "source", 0).unwrap().entries;
        assert_eq!(card.len(), 1);
        assert_eq!(card[0].probability_percent, Some(25.0));
        assert_eq!(card[0].probability(false), "Dans le pool : 25 %");
        assert_eq!(
            card[0].allocation(false).as_deref(),
            Some("4 tirages garantis + 1 tirage à 50 %")
        );
        assert_eq!(card[0].quantity(), "×1–2");
        assert!(card[0].pool_label(false).unwrap().starts_with("Pool 4"));
        assert!(page(&db, 1, "use", 0).unwrap().entries.is_empty());
    }
}
