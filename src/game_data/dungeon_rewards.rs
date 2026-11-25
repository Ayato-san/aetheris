//! Dungeon rewards have two distinct probability domains: allocation of rolls
//! (100,000,000 units per guaranteed roll) and relative item weights in a pool.
//! The allocation rule was confirmed by the user against the Ludra dungeon.
use super::*;
const ALLOCATION_UNIT: i64 = 100_000_000;
fn rows<'a>(batch: &'a BTreeMap<String, TableProbe>, name: &str) -> &'a [Value] {
    batch
        .get(name)
        .map(|t| t.layout.rows.as_slice())
        .unwrap_or(&[])
}
fn name_index<'a>(
    batch: &'a BTreeMap<String, TableProbe>,
    table: &str,
    key: &str,
) -> BTreeMap<String, (usize, &'a Value)> {
    rows(batch, table)
        .iter()
        .enumerate()
        .filter_map(|(i, r)| Some((string(&r[key])?.to_ascii_lowercase(), (i, r))))
        .collect()
}
fn allocation(raw: i64) -> Option<(i64, f64)> {
    (raw >= 0).then(|| {
        (
            raw / ALLOCATION_UNIT,
            (raw % ALLOCATION_UNIT) as f64 * 100.0 / ALLOCATION_UNIT as f64,
        )
    })
}
struct Context<'a> {
    dungeon: i64,
    name_en: String,
    name_localized: Option<String>,
    difficulty: &'a str,
    group: &'a str,
    race: &'a str,
}
struct Writer<'a> {
    db: &'a Connection,
    batch: &'a BTreeMap<String, TableProbe>,
    item_ids: BTreeMap<&'a str, i64>,
    pools: BTreeMap<String, i64>,
    packages: BTreeMap<String, (usize, &'a Value)>,
    direct: BTreeMap<String, (usize, &'a Value)>,
    missing: usize,
}
impl Writer<'_> {
    fn pool(&mut self, code: &str) -> Result<Option<i64>> {
        let key = code.to_ascii_lowercase();
        if let Some(id) = self.pools.get(&key) {
            return Ok(Some(*id));
        }
        let Some((pos, row)) = rows(self.batch, "RewardPackageItem")
            .iter()
            .enumerate()
            .find(|(_, r)| string(&r["PackageName"]).is_some_and(|n| n.eq_ignore_ascii_case(code)))
        else {
            self.missing += 1;
            return Ok(None);
        };
        let entries = row["Items"]
            .as_array()
            .ok_or_else(|| invalid("reward pool has no item list"))?;
        // Include every entry, including unresolved item references and repeated
        // items. Never normalize only the subset known to the items projection.
        let total = entries.iter().try_fold(0i64, |total, r| {
            let weight = number(r, "ItemProb")
                .filter(|w| *w >= 0)
                .ok_or_else(|| invalid("negative or missing reward pool weight"))?;
            total
                .checked_add(weight)
                .ok_or_else(|| invalid("reward pool weight overflow"))
        })?;
        self.db.execute(
            "INSERT INTO reward_pools(code,total_weight,source_row) VALUES(?1,?2,?3)",
            params![code, total, pos as i64],
        )?;
        let pool = self.db.last_insert_rowid();
        self.pools.insert(key, pool);
        for (slot, entry) in entries.iter().enumerate() {
            let reference = string(&entry["ItemName"]).unwrap_or("None");
            let weight = number(entry, "ItemProb").unwrap();
            if !self.item_ids.contains_key(reference) {
                self.missing += 1;
            }
            let percent = (total > 0).then(|| weight as f64 * 100.0 / total as f64);
            self.db.execute(
                "INSERT INTO reward_pool_entries VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    pool,
                    slot as i64,
                    reference,
                    self.item_ids.get(reference),
                    weight,
                    percent,
                    number(entry, "ItemCountMin"),
                    number(entry, "ItemCountMax")
                ],
            )?;
        }
        Ok(Some(pool))
    }
    fn group(&mut self, ctx: &Context<'_>) -> Result<()> {
        let key = ctx.group.to_ascii_lowercase();
        if let Some((pos, row)) = self.packages.get(&key).copied() {
            let race = string(&row["Race"]).unwrap_or("ERace::All");
            if ctx.race == "ERace::All" || race == "ERace::All" || ctx.race == race {
                for (slot, entry) in row["PackageItems"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .enumerate()
                {
                    let Some(code) = string(&entry["Item"]) else {
                        continue;
                    };
                    let Some(pool) = self.pool(code)? else {
                        continue;
                    };
                    self.roll(
                        ctx,
                        Some(pool),
                        None,
                        "RewardPackage",
                        pos,
                        &format!("PackageItems/{slot}"),
                        entry,
                        "Prob",
                        number(entry, "Count"),
                        number(entry, "Count"),
                        race,
                    )?;
                }
            }
        }
        // RewardItem rows are independent direct rewards, not alternatives in
        // the pool. Summing their ItemProb values would change their meaning.
        if let Some((pos, row)) = self.direct.get(&key).copied() {
            let race = string(&row["Race"]).unwrap_or("ERace::All");
            if ctx.race == "ERace::All" || race == "ERace::All" || ctx.race == race {
                for (slot, entry) in row["Items"].as_array().into_iter().flatten().enumerate() {
                    let Some(item) = string(&entry["ItemName"])
                        .and_then(|n| self.item_ids.get(n))
                        .copied()
                    else {
                        self.missing += 1;
                        continue;
                    };
                    self.roll(
                        ctx,
                        None,
                        Some(item),
                        "RewardItem",
                        pos,
                        &format!("Items/{slot}"),
                        entry,
                        "ItemProb",
                        number(entry, "ItemCountMin"),
                        number(entry, "ItemCountMax"),
                        race,
                    )?;
                }
            }
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn roll(
        &self,
        ctx: &Context<'_>,
        pool: Option<i64>,
        item: Option<i64>,
        source: &str,
        row: usize,
        path: &str,
        value: &Value,
        weight_field: &str,
        min: Option<i64>,
        max: Option<i64>,
        race: &str,
    ) -> Result<()> {
        let race = if race == "ERace::All" { ctx.race } else { race };
        let Some(raw) = number(value, weight_field) else {
            return Err(invalid("dungeon allocation weight is missing").into());
        };
        let Some((guaranteed, extra)) = allocation(raw) else {
            return Err(invalid("negative dungeon allocation weight").into());
        };
        // A zero allocation is preserved for inspection, never promoted to a
        // guaranteed reward or included in the live-source display.
        self.db.execute("INSERT OR IGNORE INTO dungeon_reward_rolls(dungeon_id,name_en,name_localized,difficulty,reward_group,race,pool_id,item_id,allocation_raw,guaranteed_draws,extra_draw_percent,quantity_min,quantity_max,source_name,source_row,source_path) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",params![ctx.dungeon,ctx.name_en,ctx.name_localized,ctx.difficulty,ctx.group,race,pool,item,raw,guaranteed,extra,min,max,source,row as i64,path])?;
        Ok(())
    }
}
pub(super) fn write(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    en: &Localization,
    loc: &Localization,
) -> Result<()> {
    let item_ids = rows(batch, "Item")
        .iter()
        .filter_map(|r| Some((string(&r["Name"])?, number(r, "ID")?)))
        .collect();
    let mut writer = Writer {
        db,
        batch,
        item_ids,
        pools: BTreeMap::new(),
        packages: name_index(batch, "RewardPackage", "Name"),
        direct: name_index(batch, "RewardItem", "Name"),
        missing: 0,
    };
    let cubes = name_index(batch, "RewardOdEnergyCube", "Name");
    for dungeon in rows(batch, "Dungeon") {
        // Dungeon.bin also describes quests, field events and map instances.
        // The confirmed allocation semantics apply only to Party/Raid loot.
        if !matches!(
            string(&dungeon["DungeonType"]),
            Some("EDungeonType::Party" | "EDungeonType::Raid")
        ) {
            continue;
        }
        let Some(id) = number(dungeon, "ID") else {
            continue;
        };
        let fallback = string(&dungeon["Name"]).unwrap_or("Dungeon");
        let title = string(&dungeon["Title"]["Key"]);
        let difficulty =
            string(&dungeon["DungeonDifficulty"]).unwrap_or("EDungeonDifficulty::None");
        let code = difficulty.rsplit("::").next().unwrap_or(difficulty);
        let names:Option<(Option<String>,Option<String>)>=db.query_row("SELECT name_en,name_localized FROM catalogue_labels WHERE enum_type='EDungeonDifficulty' AND code=?1",[code],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        let difficulty_en = names.as_ref().and_then(|n| n.0.as_deref()).unwrap_or(code);
        let difficulty_loc = names
            .as_ref()
            .and_then(|n| n.1.as_deref())
            .unwrap_or(difficulty_en);
        let name_en = format!(
            "{} · {difficulty_en}",
            text(en, title).as_deref().unwrap_or(fallback)
        );
        let name_localized = text(loc, title).map(|s| format!("{s} · {difficulty_loc}"));
        let mut groups = Vec::new();
        for (field, race) in [
            ("RewardLight", "ERace::Light"),
            ("RewardDark", "ERace::Dark"),
        ] {
            if let Some(group) = string(&dungeon[field]) {
                groups.push((group, race, None));
            }
        }
        if let Some((_, cube)) =
            string(&dungeon["RewardOdEnergyCube"]).and_then(|n| cubes.get(&n.to_ascii_lowercase()))
        {
            if let Some(group) = string(&cube["RewardCommon"]) {
                groups.push((group, "ERace::All", string(&cube["FinalBossName"])));
            }
            for boss in cube["RewardMiddleBossList"]
                .as_array()
                .into_iter()
                .flatten()
            {
                if let Some(group) = string(&boss["RewardName"]) {
                    groups.push((group, "ERace::All", string(&boss["BossName"])));
                }
            }
        }
        for (group, race, boss) in groups {
            let boss_key = boss
                .and_then(|n| {
                    rows(batch, "NpcData")
                        .iter()
                        .find(|r| string(&r["Name"]) == Some(n))
                })
                .and_then(|r| string(&r["Desc"]["Key"]));
            let boss_en = text(en, boss_key);
            let boss_loc = text(loc, boss_key);
            let display = |base: &str, boss: Option<&str>| match boss {
                Some(n) if !base.to_lowercase().contains(&n.to_lowercase()) => {
                    format!("{base} · {n}")
                }
                _ => base.to_owned(),
            };
            writer.group(&Context {
                dungeon: id,
                name_en: display(&name_en, boss_en.as_deref()),
                name_localized: name_localized
                    .as_ref()
                    .map(|s| display(s, boss_loc.as_deref().or(boss_en.as_deref()))),
                difficulty,
                group,
                race,
            })?;
        }
    }
    let count: i64 = db.query_row("SELECT COUNT(*) FROM dungeon_reward_rolls", [], |r| {
        r.get(0)
    })?;
    let zero: i64 = db.query_row(
        "SELECT COUNT(*) FROM reward_pools WHERE total_weight=0",
        [],
        |r| r.get(0),
    )?;
    coverage(
        db,
        "items.dungeon_rewards",
        if writer.missing == 0 && zero == 0 {
            "verified"
        } else {
            "partial"
        },
        &format!(
            "{count} Party/Raid allocation records; {} missing package or item references; {zero} zero-weight pools; pool weights normalized within their full source list; allocation unit 100000000 confirmed by user against Ludra; other reward contexts unchanged",
            writer.missing
        ),
    )
}
