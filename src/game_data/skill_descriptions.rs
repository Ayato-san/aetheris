//! Resolve direct description selectors at a source effect level. Client damage
//! formulas require additional semantics; they remain visible and are reported.
use super::{binary::invalid, catalog::Result};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Description {
    pub text: Option<String>,
    pub unresolved: Vec<String>,
}

/// `level` selects the client's effect-level overrides, not a character level.
pub fn description(
    db: &Connection,
    skill: i64,
    level: u32,
    localized: bool,
) -> Result<Description> {
    let column = if localized {
        "description_localized"
    } else {
        "description_en"
    };
    let (text, maximum): (Option<String>, u32) = db.query_row(
        &format!("SELECT {column},max_level FROM skills WHERE id=?1"),
        [skill],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if level == 0 || level > maximum {
        return Err(invalid("skill level outside source limits").into());
    }
    render_template(db, text, level)
}

/// Resolve an already selected template without inventing an entity's level limits.
pub(super) fn render_template(
    db: &Connection,
    text: Option<String>,
    level: u32,
) -> Result<Description> {
    let mut unresolved = Vec::new();
    let text = if let Some(text) = text {
        let mut rendered = String::new();
        let mut rest = text.as_str();
        while let Some((before, after)) = rest.split_once('{') {
            rendered.push_str(before);
            let Some((token, tail)) = after.split_once('}') else {
                rendered.push('{');
                rendered.push_str(after);
                rest = "";
                break;
            };
            if let Some(value) = resolve(db, token, level)? {
                rendered.push_str(&value);
            } else {
                rendered.push('{');
                rendered.push_str(token);
                rendered.push('}');
                if !unresolved.iter().any(|s| s == token) {
                    unresolved.push(token.to_owned());
                }
            }
            rest = tail;
        }
        rendered.push_str(rest);
        Some(rendered)
    } else {
        None
    };
    Ok(Description { text, unresolved })
}

fn resolve(db: &Connection, token: &str, level: u32) -> Result<Option<String>> {
    let parts: Vec<_> = token.split(':').collect();
    let (prefix, id, selector, format) = match parts.as_slice() {
        [prefix @ ("se" | "abe" | "sef"), id, selector] => (*prefix, *id, *selector, "none"),
        [prefix @ ("se" | "abe" | "sef"), id, selector, format] => {
            (*prefix, *id, *selector, *format)
        }
        // se_abe includes the application effect; its level can itself be an
        // effect parameter. Do not silently equate it with the skill's level.
        _ => return Ok(None),
    };
    let Ok(id) = id.parse::<i64>() else {
        return Ok(None);
    };
    let (selector, format) = if format == "none" {
        if let Some(selector) = selector.strip_suffix("_1000") {
            (selector, "divide1000")
        } else if let Some(selector) = selector.strip_suffix("_100") {
            (selector, "divide100")
        } else {
            (selector, format)
        }
    } else {
        (selector, format)
    };
    let value = if prefix == "sef" && selector == "target_count_max" {
        db.query_row(
            "SELECT target_count_max FROM skill_effects WHERE kind_id=(SELECT id FROM skill_effect_kinds WHERE code='sef') AND id=?1",
            [id],
            |r| r.get::<_, i64>(0).map(|v| v.to_string()),
        )
        .optional()?
    } else if prefix != "sef"
        && let Some(index) = selector
            .strip_prefix(if prefix == "se" {
                "effect_value"
            } else {
                "value"
            })
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|n| *n > 0)
    {
        parameter(db, prefix, id, level, index - 1)?
    } else if prefix == "se" && matches!(selector, "aggro_absolute" | "aggro_ratio") {
        if let Some((kind, ident)) = definition(db, "se", id, level)? {
            db.query_row(
                &format!("SELECT {selector} FROM skill_effects WHERE kind_id=(SELECT id FROM skill_effect_kinds WHERE code=?1) AND id=?2"),
                params![kind, ident],
                |r| r.get::<_, i64>(0).map(|v| v.to_string()),
            )
            .optional()?
        } else {
            None
        }
    } else {
        None
    };
    Ok(value.and_then(|value| convert(&value, format)))
}

fn definition(db: &Connection, kind: &str, id: i64, level: u32) -> Result<Option<(String, i64)>> {
    let group: Option<Option<String>> = db
        .query_row(
            "SELECT level_group FROM skill_effects WHERE kind_id=(SELECT id FROM skill_effect_kinds WHERE code=?1) AND id=?2",
            params![kind, id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(group) = group else { return Ok(None) };
    if let Some(group) = group {
        let variant = db
            .query_row(
                "SELECT k.code,e.id FROM skill_effects e JOIN skill_effect_kinds k ON k.id=e.kind_id WHERE k.code=?1 AND e.level_group=?2 AND e.level=?3",
                params![format!("{kind}_level"), group, level],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        if variant.is_some() {
            return Ok(variant);
        }
    }
    Ok(Some((kind.to_owned(), id)))
}
fn parameter(
    db: &Connection,
    kind: &str,
    id: i64,
    level: u32,
    position: u32,
) -> Result<Option<String>> {
    let Some((kind, id)) = definition(db, kind, id, level)? else {
        return Ok(None);
    };
    Ok(db
        .query_row(
            "SELECT value FROM skill_effect_values WHERE kind_id=(SELECT id FROM skill_effect_kinds WHERE code=?1) AND effect_id=?2 AND position=?3",
            params![kind, id, position],
            |r| r.get(0),
        )
        .optional()?)
}

#[derive(Debug, Serialize)]
pub struct SkillLevel {
    pub mp_cost: Option<i64>,
    pub casting_time: i64,
    pub cooldown: f64,
    pub skill_points: Option<i64>,
    pub stigma_points: Option<i64>,
}
/// Effective values for one selected skill level, without materializing a view.
pub fn level(db: &Connection, skill: i64, level: u32) -> Result<SkillLevel> {
    let maximum: u32 = db.query_row("SELECT max_level FROM skills WHERE id=?1", [skill], |r| {
        r.get(0)
    })?;
    if level == 0 || level > maximum {
        return Err(invalid("skill level outside source limits").into());
    }
    Ok(db.query_row("SELECT COALESCE(l.mp_cost,s.mp_cost),COALESCE((SELECT duration FROM skill_cast_times c WHERE c.skill_id=s.id AND c.level=?2 AND c.kind='casting'),(SELECT duration FROM skill_cast_times c WHERE c.skill_id=s.id AND c.level=1 AND c.kind='casting')),COALESCE(l.cooldown,s.cooldown),l.skill_points,l.stigma_points FROM skills s LEFT JOIN skill_levels l ON l.skill_id=s.id AND l.level=?2 WHERE s.id=?1",params![skill,level],|r|Ok(SkillLevel{mp_cost:r.get(0)?,casting_time:r.get(1)?,cooldown:r.get(2)?,skill_points:r.get(3)?,stigma_points:r.get(4)?}))?)
}
#[derive(Debug, Serialize)]
pub struct Stagger {
    pub effect_id: i64,
    pub damage: Option<f64>,
    pub conditional: bool,
}
/// Individual source stagger effects: do not guess hit counts or condition results.
pub fn stagger(db: &Connection, skill: i64, level: u32) -> Result<Vec<Stagger>> {
    let _ = self::level(db, skill, level)?;
    let ids=db.prepare("SELECT l.effect_id,EXISTS(SELECT 1 FROM skill_conditions c WHERE c.effect_id=e.id AND c.kind='effect') FROM skill_effect_links l JOIN skill_effects e ON e.kind_id=l.kind_id AND e.id=l.effect_id WHERE l.skill_id=?1 AND e.effect_type='GroggyGuardDecreaseEnergy' ORDER BY l.position,e.id")?.query_map([skill],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,bool>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    ids.into_iter()
        .map(|(id, conditional)| {
            Ok(Stagger {
                effect_id: id,
                conditional,
                damage: parameter(db, "se", id, level, 0)?
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|n| n.is_finite()),
            })
        })
        .collect()
}

fn convert(value: &str, format: &str) -> Option<String> {
    if format == "none" {
        return Some(value.to_owned());
    }
    let number = value.parse::<f64>().ok().filter(|n| n.is_finite())?;
    let result = match format {
        "divide10" => number / 10.0,
        "divide100" => number / 100.0,
        "divide1000" | "time" => number / 1000.0,
        "divide100abs" => number.abs() / 100.0,
        "noneabs" => number.abs(),
        _ => return None,
    };
    Some(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Connection {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE skills(id INTEGER,description_en TEXT,description_localized TEXT,max_level INTEGER);
        CREATE TABLE skill_effect_kinds(id INTEGER PRIMARY KEY,code TEXT);
        INSERT INTO skill_effect_kinds VALUES(1,'se'),(2,'abe'),(3,'sef'),(4,'se_level'),(5,'abe_level');
        CREATE TABLE skill_effects(kind_id SMALLINT,id INTEGER,level_group TEXT,level INTEGER,aggro_absolute INTEGER,aggro_ratio INTEGER,target_count_max INTEGER);
        CREATE TABLE skill_effect_values(kind_id SMALLINT,effect_id INTEGER,position INTEGER,value TEXT);
        INSERT INTO skills VALUES(1,'{se:10:effect_value01:divide100}% / {abe:20:value02:time}s / {se:10:aggro_absolute} / {sef:30:target_count_max}',NULL,3);
        INSERT INTO skill_effects VALUES(1,10,'se_group',NULL,12,0,NULL),(4,11,'se_group',2,0,0,NULL),(2,20,'abe_group',NULL,NULL,NULL,NULL),(5,21,'abe_group',2,NULL,NULL,NULL),(3,30,NULL,NULL,NULL,NULL,4);
        INSERT INTO skill_effect_values VALUES(1,10,0,'1500'),(4,11,0,'0'),(2,20,1,'1950'),(5,21,1,'3000');").unwrap();
        db
    }
    #[test]
    fn selected_level_replaces_parameters_and_zero_overrides_and_preserves_missing_translation() {
        let db = fixture();
        assert_eq!(
            description(&db, 1, 1, false).unwrap().text.as_deref(),
            Some("15% / 1.95s / 12 / 4")
        );
        let second = description(&db, 1, 2, false).unwrap();
        assert_eq!(second.text.as_deref(), Some("0% / 3s / 0 / 4"));
        assert!(second.unresolved.is_empty());
        assert!(description(&db, 1, 2, true).unwrap().text.is_none());
        assert!(description(&db, 1, 0, false).is_err());
        assert!(description(&db, 1, 4, false).is_err());
        // No level 3 override exists: retain the base, without extrapolation.
        assert_eq!(
            description(&db, 1, 3, false).unwrap().text,
            description(&db, 1, 1, false).unwrap().text
        );
    }
    #[test]
    fn missing_parameters_and_client_formulas_remain_visible_and_are_reported() {
        let db = fixture();
        let text = "{se_dmg:10:SkillUIMinDmgSum} {se:10:effect_value09} {se_abe:10:20:value02} {se:999:effect_value01} {se:10:effect_value01:unknown}";
        db.execute("UPDATE skills SET description_en=?1", [text])
            .unwrap();
        let result = description(&db, 1, 2, false).unwrap();
        assert_eq!(result.text.as_deref(), Some(text));
        assert_eq!(result.unresolved.len(), 5);
        assert_eq!(convert("-550", "divide100abs").as_deref(), Some("5.5"));
        assert_eq!(convert("TRUE", "divide100"), None);
    }
}
