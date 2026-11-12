//! Immutable offline combat reports, independent of capture retention.
mod decoder;
pub mod storage;
#[cfg(test)]
mod tests;

use crate::combat_debug::{
    self,
    analysis::{self, AnalysisOptions},
    storage::Store,
};
pub use combat_debug::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const DECODER_VERSION: &str = "aion2-combat-experimental-1";
pub const TIMING_LIMIT: &str = "Horodatages de capture : les messages d’un même segment ou bundle peuvent partager leur horodatage ; précision intra-bundle non garantie.";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityKey {
    pub id: u64,
    pub occurrence: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Identity {
    Player,
    Npc,
    Unknown,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub key: EntityKey,
    pub connection: String,
    pub name: Option<String>,
    pub identity: Identity,
    pub npc_code: Option<u32>,
    pub owner_name: Option<String>,
    pub start_ns: u64,
    pub end_ns: Option<u64>,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Indicators {
    pub critical: bool,
    pub back: bool,
    pub front: bool,
    pub perfect: bool,
    pub double: bool,
    pub multi: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Damage {
    pub time_ns: u64,
    pub source: EntityKey,
    pub target: EntityKey,
    pub skill: u32,
    pub amount: u64,
    pub periodic: bool,
    pub indicators: Indicators,
    /// Original logical message, including its framing prefix.
    pub evidence: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Review {
    pub time_ns: Option<u64>,
    pub reason: String,
    pub evidence: Vec<u8>,
}
#[derive(Clone, Debug)]
pub struct CaptureAnalysis {
    pub capture_name: String,
    pub fingerprint: String,
    pub entities: Vec<Entity>,
    pub damage: Vec<Damage>,
    pub review: Vec<Review>,
    pub limitations: Vec<String>,
    pub duration_ns: u64,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Selection {
    pub target: EntityKey,
    pub local_player: EntityKey,
    pub start_ns: u64,
    pub end_ns: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Totals {
    pub damage: u64,
    pub direct_damage: u64,
    pub summon_damage: u64,
    pub hits: u64,
    pub ticks: u64,
    pub critical: u64,
    pub back: u64,
    pub front: u64,
    pub perfect: u64,
    pub double: u64,
    pub multi: u64,
}
impl Totals {
    fn add(&mut self, event: &Damage, summon: bool) -> Result<()> {
        self.damage = self
            .damage
            .checked_add(event.amount)
            .ok_or("Somme des dégâts hors limites")?;
        if summon {
            self.summon_damage += event.amount;
        } else {
            self.direct_damage += event.amount;
        }
        if event.periodic {
            self.ticks += 1;
        } else {
            self.hits += 1;
            self.critical += u64::from(event.indicators.critical);
            self.back += u64::from(event.indicators.back);
            self.front += u64::from(event.indicators.front);
            self.perfect += u64::from(event.indicators.perfect);
            self.double += u64::from(event.indicators.double);
            self.multi += u64::from(event.indicators.multi);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillTotal {
    pub skill_ids: Vec<u32>,
    pub label: String,
    pub name_provenance: String,
    pub totals: Totals,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Participant {
    pub entity: Entity,
    pub totals: Totals,
    pub skills: Vec<SkillTotal>,
    pub first_ns: u64,
    pub last_ns: u64,
    pub dps: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetainedDamage {
    pub event: Damage,
    pub player: Option<EntityKey>,
    pub attribution: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub decoder_version: String,
    pub capture_name: String,
    pub fingerprint: String,
    pub selection: Selection,
    pub target: Entity,
    pub participants: Vec<Participant>,
    pub events: Vec<RetainedDamage>,
    pub unattributed: Vec<RetainedDamage>,
    pub review: Vec<Review>,
    pub limitations: Vec<String>,
    pub timing_precision: String,
}

/// Reanalysis always reads immutable packets, never a previous decoder's text rows.
pub fn analyze_session(store: &Store, id: i64) -> Result<CaptureAnalysis> {
    let session = store
        .sessions()?
        .into_iter()
        .find(|s| s.id == id)
        .ok_or("Capture introuvable")?;
    if session.status == "live" {
        return Err("Terminez la capture avant de l’analyser.".into());
    }
    let packets = store.packets(id)?;
    let origin = packets
        .iter()
        .filter_map(|p| p.timestamp_ns)
        .min()
        .ok_or("Capture sans horodatage exploitable")?;
    let rows = analysis::analyze(
        &packets,
        AnalysisOptions {
            finalized: true,
            experimental_midstream: session.experimental_midstream,
        },
    );
    let mut result = decoder::decode(&rows, origin);
    result.capture_name = session.name;
    result.fingerprint = session.fingerprint;
    if let Some(error) = session.error {
        result.limitations.push(error);
    }
    if session.status != "terminée" {
        result
            .limitations
            .push(format!("Capture {}", session.status));
    }
    if session.experimental_midstream {
        result
            .limitations
            .push("Alignement sans SYN imposé par l’utilisateur.".into());
    }
    Ok(result)
}

impl CaptureAnalysis {
    /// Manual IDs resolve to one lifetime only; narrower windows disambiguate reuse.
    pub fn select(&self, target: u64, local: u64, start_ns: u64, end_ns: u64) -> Result<Selection> {
        if start_ns > end_ns || end_ns > self.duration_ns {
            return Err("Fenêtre invalide : utilisez les secondes relatives au début de la capture, bornes incluses.".into());
        }
        let resolve = |id| -> Result<EntityKey> {
            let matches: Vec<_> = self
                .entities
                .iter()
                .filter(|e| {
                    e.key.id == id
                        && e.start_ns <= end_ns
                        && e.end_ns.is_none_or(|end| end >= start_ns)
                })
                .collect();
            match matches.as_slice() {
                [entity] => Ok(entity.key),
                [] => Err(format!("Identifiant {id} non observé dans cette fenêtre.").into()),
                _ => Err(format!(
                    "Identifiant {id} réutilisé : sélectionnez une fenêtre plus précise."
                )
                .into()),
            }
        };
        Ok(Selection {
            target: resolve(target)?,
            local_player: resolve(local)?,
            start_ns,
            end_ns,
        })
    }

    pub fn report(&self, selection: Selection) -> Result<Report> {
        let entity = |key| {
            self.entities
                .iter()
                .find(|e| e.key == key)
                .cloned()
                .ok_or("Occurrence d’entité introuvable")
        };
        let target = entity(selection.target)?;
        let local = entity(selection.local_player)?;
        if local.identity == Identity::Npc {
            return Err("Le joueur local désigné est une entité NPC identifiée.".into());
        }
        if selection.start_ns > selection.end_ns || selection.end_ns > self.duration_ns {
            return Err("Fenêtre invalide".into());
        }
        let mut events = Vec::new();
        let mut unattributed = Vec::new();
        let mut participants = BTreeMap::<EntityKey, Participant>::new();
        for event in self.damage.iter().filter(|e| {
            e.target == selection.target
                && e.time_ns >= selection.start_ns
                && e.time_ns <= selection.end_ns
        }) {
            let source = entity(event.source)?;
            let (player, attribution) = if source.key == local.key {
                (
                    Some(local.key),
                    "Joueur local désigné par l’utilisateur".to_string(),
                )
            } else if source.identity == Identity::Player {
                (Some(source.key), "Identité joueur décodée".into())
            } else if let Some(owner) = &source.owner_name {
                let owners: Vec<_> = self
                    .entities
                    .iter()
                    .filter(|e| {
                        e.identity == Identity::Player
                            && e.connection == source.connection
                            && e.name.as_ref() == Some(owner)
                            && e.start_ns <= event.time_ns
                            && e.end_ns.is_none_or(|end| event.time_ns < end)
                    })
                    .collect();
                if owners.len() == 1 {
                    (
                        Some(owners[0].key),
                        format!("Invocation : propriétaire nommé {owner}, unique et actif"),
                    )
                } else {
                    (None, "Propriétaire absent ou ambigu".into())
                }
            } else {
                (None, "Identité ou propriétaire non établi".into())
            };
            let retained = RetainedDamage {
                event: event.clone(),
                player,
                attribution,
            };
            if let Some(key) = player {
                let participant = participants.entry(key).or_insert(Participant {
                    entity: entity(key)?,
                    totals: Totals::default(),
                    skills: Vec::new(),
                    first_ns: event.time_ns,
                    last_ns: event.time_ns,
                    dps: None,
                });
                participant.first_ns = participant.first_ns.min(event.time_ns);
                participant.last_ns = participant.last_ns.max(event.time_ns);
                participant.totals.add(event, source.key != key)?;
                let (group, label) = decoder::skill_name(event.skill);
                let index = participant.skills.iter().position(|s| {
                    s.skill_ids.contains(&event.skill)
                        || (group != 0 && s.skill_ids.contains(&group))
                });
                let index = index.unwrap_or_else(|| {
                    participant.skills.push(SkillTotal {
                        skill_ids: if group == 17730001 {
                            vec![17730001, 17730002]
                        } else {
                            vec![event.skill]
                        },
                        label: label
                            .map(str::to_string)
                            .unwrap_or_else(|| format!("Compétence {}", event.skill)),
                        name_provenance: if label.is_some() {
                            "Rapprochement Questlog, combat Zikel (11 novembre 2026)"
                        } else {
                            "Identifiant réseau ; nom inconnu"
                        }
                        .into(),
                        totals: Totals::default(),
                    });
                    participant.skills.len() - 1
                });
                participant.skills[index]
                    .totals
                    .add(event, source.key != key)?;
                events.push(retained);
            } else {
                unattributed.push(retained);
            }
        }
        if !participants.contains_key(&local.key) {
            return Err(if !self.review.is_empty() || !self.limitations.is_empty() {
                "Analyse incomplète : impossible d’établir que le joueur local a infligé des dégâts fiables à cette cible dans la fenêtre. Affinez la sélection ou examinez les limites."
            } else { "Aucun dégât du joueur local établi contre cette cible dans la fenêtre." }.into());
        }
        for participant in participants.values_mut() {
            let duration = participant.last_ns - participant.first_ns;
            participant.dps =
                (duration > 0).then(|| participant.totals.damage as f64 * 1e9 / duration as f64);
            participant
                .skills
                .sort_by_key(|s| std::cmp::Reverse(s.totals.damage));
        }
        Ok(Report {
            decoder_version: DECODER_VERSION.into(),
            capture_name: self.capture_name.clone(),
            fingerprint: self.fingerprint.clone(),
            selection,
            target,
            participants: participants.into_values().collect(),
            events,
            unattributed,
            review: self.review.clone(),
            limitations: self.limitations.clone(),
            timing_precision: TIMING_LIMIT.into(),
        })
    }
}
