//! Bounded parsing of the layouts observed in the Zikel reference capture.
//! Unsupported switches/forms/tails are evidence to review, never guessed amounts.
use super::*;
use crate::combat_debug::analysis::{AnalysisRow, read_varuint};

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or("Offset hors limites")?;
        let value = self.bytes.get(self.pos..end).ok_or("Message tronqué")?;
        self.pos = end;
        Ok(value)
    }
    fn var(&mut self) -> Result<u64> {
        let (value, width) = read_varuint(&self.bytes[self.pos..]).ok_or("Varint invalide")?;
        if width > 1 && self.bytes[self.pos + width - 1] == 0 {
            return Err("Varint non minimal".into());
        }
        self.pos += width;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn text(&mut self) -> Result<String> {
        let n = usize::try_from(self.var()?)?;
        if n > 128 {
            return Err("Nom hors limites".into());
        }
        let text = std::str::from_utf8(self.take(n)?)?;
        if text.chars().any(char::is_control) {
            return Err("Nom invalide".into());
        }
        Ok(text.into())
    }
    fn finished(&self) -> Result<()> {
        if self.pos == self.bytes.len() {
            Ok(())
        } else {
            Err("Champs supplémentaires non reconnus".into())
        }
    }
}

struct Registry {
    entities: Vec<Entity>,
    scope: String,
    active: BTreeMap<(String, u64), EntityKey>,
}
impl Registry {
    fn ensure(&mut self, id: u64, time: u64) -> EntityKey {
        if let Some(key) = self.active.get(&(self.scope.clone(), id)) {
            return *key;
        }
        let occurrence = self.entities.iter().filter(|e| e.key.id == id).count();
        let key = EntityKey { id, occurrence };
        self.entities.push(Entity {
            key,
            connection: self.scope.clone(),
            name: None,
            identity: Identity::Unknown,
            npc_code: None,
            owner_name: None,
            start_ns: if occurrence == 0 { 0 } else { time },
            end_ns: None,
        });
        self.active.insert((self.scope.clone(), id), key);
        key
    }
    fn remove(&mut self, id: u64, time: u64) {
        if let Some(key) = self.active.remove(&(self.scope.clone(), id)) {
            self.entities
                .iter_mut()
                .find(|e| e.key == key)
                .unwrap()
                .end_ns = Some(time);
        }
    }
    fn spawn(&mut self, mut entity: Entity, time: u64) {
        self.remove(entity.key.id, time);
        entity.key = self.ensure(entity.key.id, time);
        entity.connection = self.scope.clone();
        entity.start_ns = time;
        let slot = self
            .entities
            .iter_mut()
            .find(|e| e.key == entity.key)
            .unwrap();
        *slot = entity;
    }
}

pub(super) fn decode(rows: &[AnalysisRow], origin: i128) -> CaptureAnalysis {
    let mut result = CaptureAnalysis { capture_name: String::new(), fingerprint: String::new(), entities: Vec::new(), damage: Vec::new(), review: Vec::new(), duration_ns: 0, limitations: vec!["Interprétations et indicateurs expérimentaux validés sur Zikel uniquement ; validation sur capture réelle de groupe encore nécessaire. Les structures d’identité non reconnues restent non attribuées.".into()] };
    let mut registry = Registry {
        entities: Vec::new(),
        scope: String::new(),
        active: BTreeMap::new(),
    };
    let mut ordered: Vec<_> = rows.iter().collect();
    ordered.sort_by_key(|r| r.timestamp_ns);
    for row in ordered {
        let time = row
            .timestamp_ns
            .and_then(|t| u64::try_from(t - origin).ok());
        if let Some(time) = time {
            result.duration_ns = result.duration_ns.max(time);
        }
        if row.direction != "Serveur → client" {
            continue;
        }
        if row.status == "incomplet / erreur"
            || (row.kind != "message"
                && row.kind != "enveloppe LZ4"
                && !row.summary.contains("Contrôle TCP")
                && !row.summary.contains("Retransmission TCP identique"))
        {
            result.review.push(Review {
                time_ns: time,
                reason: row.summary.clone(),
                evidence: row.bytes.clone(),
            });
        }
        if row.kind != "message" {
            continue;
        }
        registry.scope = format!("{} → {}", row.source, row.destination);
        let parsed = (|| -> Result<()> {
            let mut r = Reader {
                bytes: &row.bytes,
                pos: 0,
            };
            let encoded_length = r.var()?;
            let body_length =
                usize::try_from(encoded_length.checked_sub(4).ok_or("Longueur invalide")?)?;
            if body_length != r.bytes.len() - r.pos {
                return Err("Longueur du message incohérente".into());
            }
            let opcode: [u8; 2] = r.take(2)?.try_into()?;
            if !matches!(
                opcode,
                [0x04, 0x38] | [0x05, 0x38] | [0x33, 0x36] | [0x41, 0x36] | [0x42, 0x36]
            ) {
                return Ok(());
            }
            let time = time.ok_or("Horodatage manquant ou invalide")?;
            match opcode {
                [0x33, 0x36] => {
                    let id = r.var()?;
                    if r.byte()? != 0x5f {
                        return Err("Variante d’identité joueur non reconnue".into());
                    }
                    r.u32()?;
                    let name = r.text()?;
                    registry.spawn(
                        Entity {
                            key: EntityKey { id, occurrence: 0 },
                            connection: String::new(),
                            name: (!name.is_empty()).then_some(name),
                            identity: Identity::Player,
                            npc_code: None,
                            owner_name: None,
                            start_ns: time,
                            end_ns: None,
                        },
                        time,
                    );
                }
                [0x41, 0x36] => {
                    let id = r.var()?;
                    registry.remove(id, time);
                    let variant = r.var()?;
                    if !matches!(variant, 0x1c | 0x1f | 0x04 | 0x5f) {
                        registry.remove(id, time);
                        return Err("Variante d’apparition non reconnue".into());
                    }
                    let fields = r.var()?;
                    if fields != 0 && !(variant == 4 && fields == 0x22) {
                        registry.remove(id, time);
                        return Err("Champs d’apparition non reconnus".into());
                    }
                    let owner_name = match r.byte()? {
                        0 => None,
                        1 => {
                            let name = r.text()?;
                            (!name.is_empty()).then_some(name)
                        }
                        _ => {
                            registry.remove(id, time);
                            return Err("Structure du propriétaire non reconnue".into());
                        }
                    };
                    let npc_code = r.u32()?;
                    // Only the identity prefix is consumed. Remaining spawn fields
                    // do not participate in damage or ownership interpretation.
                    registry.spawn(
                        Entity {
                            key: EntityKey { id, occurrence: 0 },
                            connection: String::new(),
                            name: None,
                            identity: Identity::Npc,
                            npc_code: Some(npc_code),
                            owner_name,
                            start_ns: time,
                            end_ns: None,
                        },
                        time,
                    );
                }
                [0x42, 0x36] => {
                    let id = r.var()?;
                    registry.remove(id, time);
                }
                [0x04, 0x38] | [0x05, 0x38] => {
                    let target = r.var()?;
                    let sw = r.var()?;
                    let periodic = opcode[0] == 5;
                    let (source, skill, amount, indicators) = if periodic {
                        if !matches!(sw, 9..=11) {
                            return Err("Switch de tick non reconnu".into());
                        }
                        let source = r.var()?;
                        r.var()?;
                        r.u32()?;
                        let amount = r.var()?;
                        if sw == 11 {
                            r.var()?;
                        }
                        let skill = r.u32()?;
                        r.finished()?;
                        (source, skill, amount, Indicators::default())
                    } else {
                        let form = r.var()?;
                        if form != 0 || !matches!(sw, 0 | 4 | 6 | 0x24 | 0x26) {
                            return Err("Forme ou switch d’impact non reconnu".into());
                        }
                        let source = r.var()?;
                        let skill = r.u32()?;
                        r.byte()?;
                        let kind = r.var()?;
                        let (mods, direction) = if sw & 15 == 6 {
                            let mods = r.byte()?;
                            r.var()?;
                            (mods, r.byte()?)
                        } else {
                            (0, 0)
                        };
                        r.u32()?;
                        r.u32()?;
                        r.var()?;
                        let amount = if sw & 15 != 0 { r.var()? } else { 0 };
                        if sw & 0x20 != 0 {
                            let n = r.var()?;
                            if n > 64 {
                                return Err("Liste multi-impact hors limites".into());
                            }
                            for _ in 0..n {
                                r.var()?;
                            }
                        }
                        if r.take(2)? != [1, 0] {
                            return Err("Fin d’impact non reconnue".into());
                        }
                        r.finished()?;
                        (
                            source,
                            skill,
                            amount,
                            Indicators {
                                critical: kind == 3,
                                back: direction == 1,
                                front: direction == 2,
                                perfect: mods & 4 != 0,
                                double: mods & 8 != 0,
                                multi: sw & 0x20 != 0,
                            },
                        )
                    };
                    if amount > 0 {
                        let source = registry.ensure(source, time);
                        let target = registry.ensure(target, time);
                        result.damage.push(Damage {
                            time_ns: time,
                            source,
                            target,
                            skill,
                            amount,
                            periodic,
                            indicators,
                            evidence: row.bytes.clone(),
                        });
                    }
                }
                _ => unreachable!(),
            }
            Ok(())
        })();
        if let Err(error) = parsed {
            result.review.push(Review {
                time_ns: time,
                reason: format!(
                    "{} : {error} — à examiner, exclu des totaux fiables",
                    row.opcode
                ),
                evidence: row.bytes.clone(),
            });
        }
    }
    result.entities = registry.entities;
    result
}

pub(super) fn skill_name(id: u32) -> (u32, Option<&'static str>) {
    let name = match id {
        17352450 => "Condemnation",
        17440037 => "Nobble Aura",
        17040020 => "Judgment Thunder (secondary atk 1)",
        17050020 => "Divine Punishment (secondary atk 2)",
        17150452 => "Divine Aura",
        17060343 => "Bolt",
        17730001 | 17730002 => "Empyrean Lord's Grace (passive)",
        17400040 => "Earth Punishment",
        17080240 => "Debilitating Mark",
        17070240 => "Chain of Torment",
        17010240 => "Earth's Retribution (auto atk 1)",
        17020240 => "Thunder and Lightning (auto atk 2)",
        17030240 => "Discharge (auto atk 3)",
        _ => return (id, None),
    };
    (if id == 17730002 { 17730001 } else { id }, Some(name))
}
