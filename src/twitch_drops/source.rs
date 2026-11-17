use super::*;
use std::collections::HashSet;

fn optional_text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)?
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
}

pub(super) struct Scan {
    pub campaigns: Vec<Campaign>,
    pub refreshed_at: Option<i64>,
    pub reset_at: Option<i64>,
}

/// Validate the whole relevant response before changing presence or freshness.
pub(super) fn parse(body: &[u8], observed: i64) -> Result<Scan> {
    let body: Value = serde_json::from_slice(body)?;
    let games = body
        .get("games")
        .and_then(Value::as_array)
        .ok_or_else(|| error("Liste games absente ou invalide."))?;
    let mut campaigns = Vec::new();
    let mut ids = HashSet::new();
    for game in games {
        let name = game
            .get("gameDisplayName")
            .and_then(Value::as_str)
            .ok_or_else(|| error("Nom de jeu invalide."))?;
        if name != "AION 2" {
            continue;
        }
        let entries = game
            .get("rewards")
            .and_then(Value::as_array)
            .ok_or_else(|| error("Campagnes AION 2 invalides."))?;
        for raw in entries {
            let id =
                optional_text(raw, "id").ok_or_else(|| error("Identité de campagne absente."))?;
            if !ids.insert(id.clone()) {
                return Err(error("Identité de campagne dupliquée."));
            }
            let start =
                timestamp(&raw["startAt"]).ok_or_else(|| error("Début de campagne invalide."))?;
            let end = timestamp(&raw["endAt"]).ok_or_else(|| error("Fin de campagne invalide."))?;
            if start >= end {
                return Err(error("Période de campagne incohérente."));
            }
            let rewards = match raw.get("timeBasedDrops") {
                None | Some(Value::Null) => None,
                Some(Value::Array(tiers)) => {
                    let mut rewards = Vec::new();
                    for tier in tiers {
                        if !tier.is_object() {
                            return Err(error("Palier invalide."));
                        }
                        let edges = match tier.get("benefitEdges") {
                            Some(Value::Array(edges)) if !edges.is_empty() => edges.clone(),
                            None | Some(Value::Null) | Some(Value::Array(_)) => vec![Value::Null],
                            _ => return Err(error("Récompenses invalides.")),
                        };
                        for edge in edges {
                            rewards.push(Reward {
                                name: optional_text(&edge["benefit"], "name")
                                    .or_else(|| optional_text(tier, "name")),
                                tier_name: optional_text(tier, "name"),
                                image_url: optional_text(&edge["benefit"], "imageAssetURL"),
                                required_minutes: tier
                                    .get("requiredMinutesWatched")
                                    .and_then(Value::as_u64),
                                required_subs: tier.get("requiredSubs").and_then(Value::as_u64),
                                raw_tier: tier.clone(),
                            });
                        }
                    }
                    Some(rewards)
                }
                _ => return Err(error("Paliers invalides.")),
            };
            let mut metadata = game.clone();
            metadata.as_object_mut().unwrap().remove("rewards");
            campaigns.push(Campaign {
                id,
                title: optional_text(raw, "name"),
                start_at: start,
                end_at: end,
                status: optional_text(raw, "status"),
                rewards,
                raw: raw.clone(),
                game: metadata,
                source_url: SOURCE_URL.into(),
                first_observed_at: observed,
                last_observed_at: observed,
                last_source_scan_at: timestamp(&body["refreshedAt"]).filter(|t| *t <= observed),
                present: true,
                user: UserState::default(),
            });
        }
    }
    Ok(Scan {
        campaigns,
        refreshed_at: timestamp(&body["refreshedAt"]).filter(|t| *t <= observed),
        reset_at: timestamp(&body["rateLimit"]["resetAt"]),
    })
}
