//! Read-only, level-aware cards shared by skills and their related effects.
use super::*;
use rusqlite::{OptionalExtension, params};

#[derive(Debug)]
pub struct CardField {
    pub name_en: &'static str,
    pub name_localized: &'static str,
    pub value: String,
    pub value_localized: Option<String>,
}
#[derive(Debug)]
pub struct CardSection {
    pub truncated: bool,
    pub name_en: &'static str,
    pub name_localized: &'static str,
    pub fields: Vec<CardField>,
}
#[derive(Debug)]
pub struct CardLevel {
    pub level: i64,
    pub description_en: Option<String>,
    pub description_localized: Option<String>,
    pub sections: Vec<CardSection>,
}
#[derive(Debug)]
pub struct DefinitionCard {
    pub id: i64,
    pub name_en: Option<String>,
    pub name_localized: Option<String>,
    pub tag: String,
    pub icon: Option<Vec<u8>>,
    pub icon_mime: Option<String>,
    pub levels: Vec<CardLevel>,
    pub related: Vec<DefinitionCard>,
    pub related_truncated: bool,
}
fn field(en: &'static str, loc: &'static str, value: impl ToString) -> CardField {
    CardField {
        name_en: en,
        name_localized: loc,
        value: value.to_string(),
        value_localized: None,
    }
}
fn optional<T: ToString>(
    fields: &mut Vec<CardField>,
    en: &'static str,
    loc: &'static str,
    value: Option<T>,
) {
    if let Some(value) = value {
        fields.push(field(en, loc, value));
    }
}
fn section(en: &'static str, loc: &'static str, mut fields: Vec<CardField>) -> CardSection {
    let truncated = fields.len() > RELATION_LIMIT;
    fields.truncate(RELATION_LIMIT);
    CardSection {
        truncated,
        name_en: en,
        name_localized: loc,
        fields,
    }
}
fn image(db: &Connection, card: &mut DefinitionCard, family: &str) -> Result<()> {
    if let Some((data,mime))=db.query_row("SELECT i.image,i.mime_type FROM catalogue_entity_icons e JOIN catalogue_icons i ON i.id=e.icon_id WHERE e.family=?1 AND e.source_id=?2",params![family,card.id],|r|Ok((r.get(0)?,r.get(1)?))).optional()? {
        card.icon=data;card.icon_mime=mime;
    }
    Ok(())
}
fn conditions(db: &Connection, id: i64) -> Result<Vec<CardField>> {
    Ok(db.prepare("SELECT type,target,parameter,position,value,description_en,description_localized FROM skill_conditions WHERE skill_id=?1 ORDER BY kind,condition_id,parameter,position LIMIT 201")?
        .query_map([id],|r|{
            let raw=format!("{} · {} · {}[{}] = {}",r.get::<_,String>(0)?,r.get::<_,Option<String>>(1)?.unwrap_or_default(),r.get::<_,String>(2)?,r.get::<_,i64>(3)?,r.get::<_,Option<String>>(4)?.unwrap_or_else(||"—".into()));
            let mut f=field("Condition","Condition",r.get::<_,Option<String>>(5)?.map(|s|format!("{s}\n{raw}")).unwrap_or_else(||raw.clone()));
            f.value_localized=r.get::<_,Option<String>>(6)?.map(|s|format!("{s}\n{raw}"));Ok(f)
        })?.collect::<rusqlite::Result<_>>()?)
}
fn enum_field(
    db: &Connection,
    en: &'static str,
    loc: &'static str,
    ty: &str,
    code: &str,
) -> Result<CardField> {
    let mut f = field(en, loc, code);
    let code = code.rsplit("::").next().unwrap_or(code);
    if let Some((english, localized)) = db
        .query_row(
            "SELECT name_en,name_localized FROM catalogue_labels WHERE enum_type=?1 AND code=?2",
            params![ty, code],
            |r| {
                Ok((
                    r.get::<_, Option<String>>(0)?,
                    r.get::<_, Option<String>>(1)?,
                ))
            },
        )
        .optional()?
    {
        if let Some(value) = english {
            f.value = value;
        }
        f.value_localized = localized;
    }
    Ok(f)
}
pub(super) fn load(db: &Connection, family: Family, id: i64) -> Result<DefinitionCard> {
    let mut card = DefinitionCard {
        id,
        name_en: None,
        name_localized: None,
        tag: format!("{} · #{id}", family.label()),
        icon: None,
        icon_mime: None,
        levels: vec![],
        related: vec![],
        related_truncated: false,
    };
    let related: Vec<(i64, i64)> = if family == Family::Skills {
        let (name,localized,rarity,kind,damage,mp,cooldown,range,weapon_range,max_level,stagger)=db.query_row("SELECT s.name_en,s.name_localized,r.code,t.code,d.code,s.mp_cost,s.cooldown,s.range,s.uses_weapon_range,s.max_level,s.stagger_damage FROM skills s LEFT JOIN skill_rarities r ON r.id=s.rarity_id LEFT JOIN skill_types t ON t.id=s.type_id LEFT JOIN skill_damage_types d ON d.id=s.damage_type_id WHERE s.id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,Option<String>>(4)?,r.get::<_,Option<i64>>(5)?,r.get::<_,Option<f64>>(6)?,r.get::<_,Option<f64>>(7)?,r.get::<_,Option<bool>>(8)?,r.get::<_,u32>(9)?,r.get::<_,Option<f64>>(10)?)))?;
        card.name_en = name;
        card.name_localized = localized;
        let mut common = vec![];
        optional(&mut common, "Rarity (source)", "Rareté (source)", rarity);
        if let Some(kind) = kind {
            common.push(enum_field(db, "Type", "Type", "ESkillListCategory", &kind)?);
        }
        optional(&mut common, "Damage type", "Type de dégâts", damage);
        optional(&mut common, "Range (source)", "Portée (source)", range);
        if weapon_range == Some(true) {
            common.push(field("Range", "Portée", "Weapon range / Portée de l’arme"));
        }
        optional(&mut common, "Stagger", "Déséquilibre", stagger);
        for (sql, en, loc) in [
            (
                "SELECT w.code FROM skill_weapons s JOIN skill_weapon_types w ON w.id=s.weapon_id WHERE s.skill_id=?1 ORDER BY w.code",
                "Required weapon",
                "Arme requise",
            ),
            (
                "SELECT c.code FROM skill_class_permissions p JOIN skill_classes c ON c.id=p.class_id WHERE p.skill_id=?1 AND p.is_listed=1 ORDER BY c.code",
                "Listed class",
                "Classe du catalogue",
            ),
        ] {
            let values = db
                .prepare(sql)?
                .query_map([id], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            for value in values {
                common.push(enum_field(
                    db,
                    en,
                    loc,
                    if en == "Required weapon" {
                        "EWeaponType"
                    } else {
                        "ECharacterClass"
                    },
                    &value,
                )?);
            }
        }
        let cond = conditions(db, id)?;
        let specializations=db.prepare("SELECT s.specialization_id,t.name_en,t.name_localized,s.required_level,s.description_en,s.description_localized FROM skill_specializations s JOIN skills t ON t.id=s.specialization_id WHERE s.skill_id=?1 ORDER BY s.specialization_id LIMIT 201")?.query_map([id],|r| {
            let sid:i64=r.get(0)?;let required:Option<i64>=r.get(3)?;
            let value=|name:Option<String>,description:Option<String>|format!("{} · #{sid}{}\n{}",name.unwrap_or_else(||"—".into()),required.map(|v|format!(" · Lv {v}")).unwrap_or_default(),description.unwrap_or_default());
            Ok(CardField {name_en:"Specialization",name_localized:"Spécialisation",value:value(r.get(1)?,r.get(4)?),value_localized:Some(value(r.get(2)?,r.get(5)?))})
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
        // Each level is established by the skill's max_level, not a player level.
        for level in 1..=max_level {
            let overrides=db.query_row("SELECT mp_cost,cooldown,skill_points,stigma_points FROM skill_levels WHERE skill_id=?1 AND level=?2",params![id,level],|r|Ok((r.get::<_,Option<i64>>(0)?,r.get::<_,Option<f64>>(1)?,r.get::<_,Option<i64>>(2)?,r.get::<_,Option<i64>>(3)?))).optional()?;
            let mut fields = common.iter().map(clone_field).collect::<Vec<_>>();
            optional(
                &mut fields,
                "MP cost",
                "Coût en PM",
                overrides.as_ref().and_then(|r| r.0).or(mp),
            );
            optional(
                &mut fields,
                "Cooldown (source)",
                "Recharge (source)",
                overrides.as_ref().and_then(|r| r.1).or(cooldown),
            );
            optional(
                &mut fields,
                "Skill points",
                "Points de compétence",
                overrides.as_ref().and_then(|r| r.2),
            );
            optional(
                &mut fields,
                "Stigma points",
                "Points de stigma",
                overrides.as_ref().and_then(|r| r.3),
            );
            let casts=db.prepare("SELECT c.kind,c.stage,c.duration FROM skill_cast_times c WHERE c.skill_id=?1 AND (c.level=?2 OR (c.level=1 AND NOT EXISTS(SELECT 1 FROM skill_cast_times o WHERE o.skill_id=c.skill_id AND o.level=?2 AND o.kind=c.kind AND o.stage=c.stage))) ORDER BY c.kind,c.stage")?.query_map(params![id,level],|r|Ok(format!("{} · {} · {}",r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
            for value in casts {
                fields.push(field("Cast time (source)", "Incantation (source)", value));
            }
            let en = super::super::skill_descriptions::description(db, id, level, false)?;
            let loc = super::super::skill_descriptions::description(db, id, level, true)?;
            let mut sections = vec![
                section("SKILL", "COMPÉTENCE", fields),
                section(
                    "ACTIVATION",
                    "ACTIVATION",
                    cond.iter().map(clone_field).collect(),
                ),
                section(
                    "SPECIALIZATIONS",
                    "SPÉCIALISATIONS",
                    specializations.iter().map(clone_field).collect(),
                ),
            ];
            let unresolved = en
                .unresolved
                .into_iter()
                .chain(loc.unresolved)
                .collect::<BTreeSet<_>>();
            if !unresolved.is_empty() {
                sections.push(section(
                    "UNRESOLVED PARAMETERS",
                    "PARAMÈTRES NON RÉSOLUS",
                    unresolved
                        .into_iter()
                        .map(|s| field("Parameter", "Paramètre", s))
                        .collect(),
                ));
            }
            card.levels.push(CardLevel {
                level: i64::from(level),
                description_en: en.text,
                description_localized: loc.text,
                sections,
            });
        }
        db.prepare("SELECT DISTINCT kind_id,effect_id FROM skill_effect_links WHERE skill_id=?1 ORDER BY kind_id,effect_id LIMIT 201")?.query_map([id],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?
    } else if family == Family::Abnormals {
        let (name,loc,en_desc,loc_desc,kind,category,damage,max_level,overlap,overlap_type,replace)=db.query_row("SELECT name_en,name_localized,description_en,description_localized,type_source,display_category_source,damage_type_source,max_level,overlap_count,overlap_time_type_source,replace_type_source FROM skill_abnormals WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,Option<String>>(4)?,r.get::<_,Option<String>>(5)?,r.get::<_,Option<String>>(6)?,r.get::<_,Option<i64>>(7)?,r.get::<_,Option<i64>>(8)?,r.get::<_,Option<String>>(9)?,r.get::<_,Option<String>>(10)?)))?;
        card.name_en = name;
        card.name_localized = loc;
        let mut fields = vec![];
        optional(&mut fields, "Type", "Type", kind);
        optional(&mut fields, "Category", "Catégorie", category);
        optional(&mut fields, "Damage type", "Type de dégâts", damage);
        optional(&mut fields, "Max level", "Niveau maximal", max_level);
        optional(&mut fields, "Stacks", "Cumul", overlap);
        optional(
            &mut fields,
            "Stack time type",
            "Durée du cumul",
            overlap_type,
        );
        optional(&mut fields, "Replacement", "Remplacement", replace);
        let maximum = u32::try_from(max_level.unwrap_or(1))?;
        if !(1..=255).contains(&maximum) {
            return Err(invalid("abnormal level outside supported limits").into());
        }
        for level in 1..=maximum {
            card.levels.push(CardLevel {
                level: i64::from(level),
                description_en: super::super::skill_descriptions::render_template(
                    db,
                    en_desc.clone(),
                    level,
                )?
                .text,
                description_localized: super::super::skill_descriptions::render_template(
                    db,
                    loc_desc.clone(),
                    level,
                )?
                .text,
                sections: vec![section(
                    "STATUS",
                    "ALTÉRATION",
                    fields.iter().map(clone_field).collect(),
                )],
            });
        }
        db.prepare("SELECT e.kind_id,e.id FROM skill_effects e JOIN skill_effect_kinds k ON k.id=e.kind_id WHERE e.abnormal_id=?1 AND k.code IN ('se','abe','sef') ORDER BY e.kind_id,e.id LIMIT 201")?.query_map([id],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?
    } else {
        return Err(invalid("item cards use their own projection").into());
    };
    image(db, &mut card, family.evidence_family())?;
    card.related_truncated = related.len() > RELATION_LIMIT;
    for (kind, eid) in related.into_iter().take(RELATION_LIMIT) {
        card.related.push(effect(db, kind, eid)?);
    }
    Ok(card)
}
fn clone_field(f: &CardField) -> CardField {
    CardField {
        name_en: f.name_en,
        name_localized: f.name_localized,
        value: f.value.clone(),
        value_localized: f.value_localized.clone(),
    }
}
fn effect(db: &Connection, kind: i64, id: i64) -> Result<DefinitionCard> {
    let (code,group,level,ty,abnormal,max_level,aggro,ratio,targets)=db.query_row("SELECT k.code,e.level_group,e.level,e.effect_type,e.abnormal_id,e.max_level,e.aggro_absolute,e.aggro_ratio,e.target_count_max FROM skill_effects e JOIN skill_effect_kinds k ON k.id=e.kind_id WHERE e.kind_id=?1 AND e.id=?2",params![kind,id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,Option<String>>(1)?,r.get::<_,Option<i64>>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,Option<i64>>(4)?,r.get::<_,Option<i64>>(5)?,r.get::<_,Option<i64>>(6)?,r.get::<_,Option<i64>>(7)?,r.get::<_,Option<i64>>(8)?)))?;
    let mut fields = vec![field("Kind", "Famille", &code)];
    optional(&mut fields, "Effect type", "Type d’effet", ty.clone());
    optional(&mut fields, "Abnormal ID", "ID d’altération", abnormal);
    optional(&mut fields, "Max level", "Niveau maximal", max_level);
    optional(&mut fields, "Aggro", "Hostilité", aggro);
    optional(&mut fields, "Aggro ratio", "Ratio d’hostilité", ratio);
    optional(&mut fields, "Max targets", "Cibles maximales", targets);
    let values = |kind: i64, id: i64| -> Result<BTreeMap<i64, String>> {
        Ok(db.prepare("SELECT position,value FROM skill_effect_values WHERE kind_id=?1 AND effect_id=?2 ORDER BY position LIMIT 201")?.query_map(params![kind,id],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?)
    };
    let base = values(kind, id)?;
    let mut levels = BTreeMap::from([(level.unwrap_or(1), (kind, id))]);
    if let Some(group) = group
        && matches!(code.as_str(), "se" | "abe")
    {
        for row in db.prepare("SELECT e.level,e.kind_id,e.id FROM skill_effects e JOIN skill_effect_kinds k ON k.id=e.kind_id WHERE k.code=?1 AND e.level_group=?2 ORDER BY e.level")?.query_map(params![format!("{code}_level"),group],|r|Ok((r.get::<_,i64>(0)?,(r.get::<_,i64>(1)?,r.get::<_,i64>(2)?))))? {let (level,key)=row?;levels.insert(level,key);}
    }
    let mut card = DefinitionCard {
        id,
        name_en: Some(ty.clone().unwrap_or_else(|| code.to_uppercase())),
        name_localized: None,
        tag: format!("{} · #{id}", code.to_uppercase()),
        icon: None,
        icon_mime: None,
        levels: vec![],
        related: vec![],
        related_truncated: false,
    };
    for (level, (key, eid)) in levels {
        let parameters = if (key, eid) == (kind, id) {
            base.clone()
        } else {
            values(key, eid)?
        };
        let params = parameters
            .into_iter()
            .map(|(position, value)| {
                field(
                    "Parameter",
                    "Paramètre",
                    format!("{} · {value}", position + 1),
                )
            })
            .collect();
        card.levels.push(CardLevel {
            level,
            description_en: None,
            description_localized: None,
            sections: vec![
                section("EFFECT", "EFFET", fields.iter().map(clone_field).collect()),
                section("PARAMETERS", "PARAMÈTRES", params),
            ],
        });
    }
    Ok(card)
}
