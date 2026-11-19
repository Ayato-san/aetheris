//! Offline reading and reference checks, without publishing a game catalogue.
use super::{
    binary::invalid,
    catalog::Result,
    usmap::{Mapping, TableProbe},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

/// The complete observed skill table profile. Names are not inferred from IDs.
pub const TABLES: &[(&str, &str)] = &[
    ("AbnormalVolume", "AbnormalVolumeTableItem"),
    ("CCSkillMapping", "ClassChangeSkillMappingTableItem"),
    ("CraftSkill", "CraftSkillTableItem"),
    ("CraftSkillEffect", "CraftSkillEffectTableItem"),
    ("CraftSkillEffectFx", "CraftSkillEffectFxTableItem"),
    ("DungeonAbnormalGroup", "DungeonAbnormalGroupTableItem"),
    (
        "DungeonAbnormalGroupSet",
        "DungeonAbnormalGroupSetTableItem",
    ),
    ("GatherSkill", "GatherSkillTableItem"),
    ("GatherSkillEffect", "GatherSkillEffectTableItem"),
    ("GatherSkillEffectFx", "GatherSkillEffectFxTableItem"),
    ("ItemEquipSubstatSkill", "ItemEquipSubstatSkillTableItem"),
    ("PCContextSkillSlot", "PCContextSkillSlotTableItem"),
    ("Skill", "SkillTableItem"),
    ("SkillAbnormal", "AbnormalTableItem"),
    ("SkillAbnormalEffect", "AbnormalEffectTableItem"),
    ("SkillAbnormalEffectLv", "AbnormalEffectLevelTableItem"),
    ("SkillAbnormalEffectType", "AbnormalEffectTypeTableItem"),
    ("SkillAbnormalOverlapFx", "AbnormalOverlapFxTableItem"),
    ("SkillAbnormalProperty", "AbnormalPropertyTableItem"),
    ("SkillAbnormalString", "SkillAbnormalStringTableItem"),
    ("SkillAcquireData", "SkillAcquireTableItem"),
    ("SkillAutoUse", "SkillAutoUseTableItem"),
    ("SkillChanneling", "SkillChannelingTableItem"),
    ("SkillCharge", "SkillChargeTableItem"),
    ("SkillCond", "SkillCondTableItem"),
    ("SkillCondString", "SkillCondStringTableItem"),
    ("SkillControlSetting", "SkillControlSettingTableItem"),
    ("SkillEffect", "SkillEffectTableItem"),
    ("SkillEffectFilter", "SkillEffectFilterTableItem"),
    (
        "SkillEffectFilterLocation",
        "SkillEffectFilterLocationTableItem",
    ),
    ("SkillEffectLv", "SkillEffectLvTableItem"),
    ("SkillHudActionButton", "SkillHudActionButtonTableItem"),
    ("SkillLink", "SkillLinkTableItem"),
    ("SkillList", "SkillListTableItem"),
    ("SkillLv", "SkillLvTableItem"),
    ("SkillProjectile", "ProjectileTableItem"),
    ("SkillString", "SkillStringTableItem"),
    ("SkillTransform", "SkillTransformTableItem"),
    ("SpecializedSkill", "SpecializedSkillTableItem"),
    ("SpecializedSkillParts", "SpecializedSkillPartsTableItem"),
    ("SpecializedSkillSlot", "SpecializedSkillSlotTableItem"),
];

#[derive(Serialize, Deserialize)]
pub struct SkillProbe {
    pub compatibility_verified: bool,
    pub tables: BTreeMap<String, TableProbe>,
    pub relations: Vec<ReferenceCheck>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceCheck {
    pub source_table: String,
    pub source_field: String,
    pub target_table: String,
    pub target_field: String,
    /// Positional effect parameters are hypotheses, even when all IDs resolve.
    pub inferred_parameter: bool,
    /// A type behavior table need not contain metadata for every enum member.
    pub target_may_be_partial: bool,
    pub references: usize,
    pub resolved: usize,
    pub missing: usize,
    /// At most 20 examples; counts always include every occurrence.
    pub missing_examples: Vec<MissingReference>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissingReference {
    pub source_row: usize,
    pub source_id: Option<Value>,
    pub target: Value,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    Integer(i64),
    Text(String),
}

fn key(value: &Value) -> Result<Option<Key>> {
    Ok(match value {
        Value::Null => None,
        Value::Number(n) if n.as_i64() == Some(0) => None,
        Value::Number(n) => Some(Key::Integer(
            n.as_i64()
                .ok_or_else(|| invalid("invalid skill reference integer"))?,
        )),
        Value::String(s) if s.is_empty() || s == "None" || s.ends_with("::None") => None,
        Value::String(s) => Some(Key::Text(s.clone())),
        _ => return Err(invalid("skill reference must be an integer or string").into()),
    })
}

/// A dot path traverses nested structures and every element of arrays.
fn values<'a>(value: &'a Value, path: &[&str], output: &mut Vec<&'a Value>) -> Result<()> {
    if let Value::Array(array) = value {
        for element in array {
            values(element, path, output)?;
        }
    } else if let Some((field, rest)) = path.split_first() {
        values(
            value
                .get(*field)
                .ok_or_else(|| invalid(format!("missing skill reference field {field}")))?,
            rest,
            output,
        )?;
    } else {
        output.push(value);
    }
    Ok(())
}

fn check(
    tables: &BTreeMap<String, TableProbe>,
    source: &str,
    field: &str,
    target: &str,
    target_field: &str,
    parameter: Option<(&str, usize)>,
) -> Result<ReferenceCheck> {
    let mut keys = BTreeSet::new();
    for row in &tables[target].layout.rows {
        let mut found = Vec::new();
        values(
            row,
            &target_field.split('.').collect::<Vec<_>>(),
            &mut found,
        )?;
        for value in found {
            if let Some(key) = key(value)? {
                keys.insert(key);
            }
        }
    }
    let mut report = ReferenceCheck {
        source_table: source.into(),
        source_field: field.into(),
        target_table: target.into(),
        target_field: target_field.into(),
        inferred_parameter: parameter.is_some(),
        target_may_be_partial: target == "SkillAbnormalEffectType",
        references: 0,
        resolved: 0,
        missing: 0,
        missing_examples: Vec::new(),
    };
    for (index, row) in tables[source].layout.rows.iter().enumerate() {
        let mut found = Vec::new();
        let converted;
        if let Some((effect_type, position)) = parameter {
            if row.get("EffectType").and_then(Value::as_str) != Some(effect_type) {
                continue;
            }
            let number = row
                .get("EffectValueList")
                .and_then(|v| v.get(position))
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("missing skill effect parameter"))?
                .parse::<i64>()?;
            converted = Value::from(number);
            found.push(&converted);
        } else {
            values(row, &field.split('.').collect::<Vec<_>>(), &mut found)?;
        }
        for value in found {
            if let Some(key) = key(value)? {
                report.references += 1;
                if keys.contains(&key) {
                    report.resolved += 1;
                } else {
                    report.missing += 1;
                    if report.missing_examples.len() < 20 {
                        report.missing_examples.push(MissingReference {
                            source_row: index,
                            source_id: row.get("ID").cloned(),
                            target: value.clone(),
                        });
                    }
                }
            }
        }
    }
    Ok(report)
}

/// Check explicit field references and separately identified parameter hypotheses.
/// Missing references are diagnostic data, never silently removed or rewritten.
pub fn check_relations(tables: &BTreeMap<String, TableProbe>) -> Result<Vec<ReferenceCheck>> {
    let rules = [
        (
            "Skill",
            "SkillEffectTimeDataList.SkillEffectGroupId.Value",
            "SkillEffect",
            "SkillEffectGroupId.Value",
        ),
        (
            "Skill",
            "SkillEffectTimeDataList.SkillEffectFilterId.Value",
            "SkillEffectFilter",
            "ID.Value",
        ),
        (
            "Skill",
            "SkillEffectTimeDataList.ProjectileId.Value",
            "SkillProjectile",
            "ID.Value",
        ),
        ("Skill", "SkillLvGroupId", "SkillLv", "SkillLvGroupId"),
        ("Skill", "SkillString_Key", "SkillString", "Name"),
        ("Skill", "AutoUseId.Value", "SkillAutoUse", "ID.Value"),
        ("Skill", "ChargeId.Value", "SkillCharge", "ID.Value"),
        ("Skill", "ChannelingId.Value", "SkillChanneling", "ID.Value"),
        ("Skill", "ChainSkillPrevSkillId.Value", "Skill", "ID.Value"),
        (
            "Skill",
            "DamageAnalyzerSkillIdOverride.Value",
            "Skill",
            "ID.Value",
        ),
        (
            "Skill",
            "CancelExceptionSkillIdList.Value",
            "Skill",
            "ID.Value",
        ),
        (
            "Skill",
            "ToggleOnAbnormalId.Value",
            "SkillAbnormal",
            "ID.Value",
        ),
        (
            "Skill",
            "TargetFilterId.Value",
            "SkillEffectFilter",
            "ID.Value",
        ),
        (
            "Skill",
            "ProjectileTargetFilter.Value",
            "SkillEffectFilter",
            "ID.Value",
        ),
        (
            "SkillEffect",
            "SkillEffectLvGroupId",
            "SkillEffectLv",
            "SkillEffectLvGroupId",
        ),
        (
            "SkillAbnormal",
            "SkillAbnormalString_Key",
            "SkillAbnormalString",
            "Name",
        ),
        (
            "SkillAbnormal",
            "AbnormalEffectType",
            "SkillAbnormalEffectType",
            "AbnormalEffectType",
        ),
        (
            "SkillAbnormalEffect",
            "AbnormalId.Value",
            "SkillAbnormal",
            "ID.Value",
        ),
        (
            "SkillAbnormalEffect",
            "AbnormalEffectLvGroupId",
            "SkillAbnormalEffectLv",
            "AbnormalEffectLevelGroupId",
        ),
        (
            "SkillAbnormalEffect",
            "AbnormalEffectType",
            "SkillAbnormalEffectType",
            "AbnormalEffectType",
        ),
        (
            "SkillEffectFilter",
            "TargetAbnormalIdList.Value",
            "SkillAbnormal",
            "ID.Value",
        ),
        (
            "SkillEffectFilter",
            "TargetAbnormalGroupIdList.Value",
            "SkillAbnormal",
            "GroupId",
        ),
        (
            "SkillProjectile",
            "ChainProjectileId.Value",
            "SkillProjectile",
            "ID.Value",
        ),
        (
            "SkillProjectile",
            "ChainSkillEffectGroupId.Value",
            "SkillEffect",
            "SkillEffectGroupId.Value",
        ),
        (
            "SkillProjectile",
            "ChainSkillEffectFilterId.Value",
            "SkillEffectFilter",
            "ID.Value",
        ),
        (
            "SkillProjectile",
            "ChainTargetFilterId.Value",
            "SkillEffectFilter",
            "ID.Value",
        ),
        ("SkillAcquireData", "SkillId.Value", "Skill", "ID.Value"),
        (
            "SkillCharge",
            "SkillChargeDataList.ChargeSkillId.Value",
            "Skill",
            "ID.Value",
        ),
        ("SkillCond", "CondSkillList.Value", "Skill", "ID.Value"),
        (
            "SkillCond",
            "SkillCondString_Key",
            "SkillCondString",
            "Name",
        ),
        (
            "SkillCond",
            "CondAbnormalGroupId.Value",
            "SkillAbnormal",
            "GroupId",
        ),
        (
            "SkillControlSetting",
            "ControlSettingSkillId.Value",
            "Skill",
            "ID.Value",
        ),
        ("SkillHudActionButton", "SkillId.Value", "Skill", "ID.Value"),
        ("SkillLink", "SkillId.Value", "Skill", "ID.Value"),
        (
            "SkillLink",
            "LinkSkillList.LinkSkillid.Value",
            "Skill",
            "ID.Value",
        ),
        ("SkillList", "SkillIdList", "Skill", "ID.Value"),
        (
            "SkillTransform",
            "OriginalSkillId.Value",
            "Skill",
            "ID.Value",
        ),
        (
            "SkillTransform",
            "TransformSkillId.Value",
            "Skill",
            "ID.Value",
        ),
        (
            "SpecializedSkill",
            "ParentSkillId.Value",
            "Skill",
            "ID.Value",
        ),
        (
            "SpecializedSkill",
            "ChildSkillId.Value",
            "Skill",
            "ID.Value",
        ),
        (
            "SpecializedSkillParts",
            "ParentSkillId.Value",
            "Skill",
            "ID.Value",
        ),
        (
            "PCContextSkillSlot",
            "DefaultSkillId.Value",
            "Skill",
            "ID.Value",
        ),
        (
            "PCContextSkillSlot",
            "ContextChangeSkillList.Value",
            "Skill",
            "ID.Value",
        ),
        (
            "ItemEquipSubstatSkill",
            "SkillId.Value",
            "Skill",
            "ID.Value",
        ),
        (
            "AbnormalVolume",
            "AbnormalId.Value",
            "SkillAbnormal",
            "ID.Value",
        ),
        (
            "DungeonAbnormalGroup",
            "AbnormalId.Value",
            "SkillAbnormal",
            "ID.Value",
        ),
        (
            "SkillAbnormalOverlapFx",
            "AbnormalId.Value",
            "SkillAbnormal",
            "ID.Value",
        ),
        ("GatherSkillEffect", "SkillName", "GatherSkill", "Name"),
        ("GatherSkillEffect", "FxName", "GatherSkillEffectFx", "Name"),
    ];
    for &(source, _, target, _) in &rules {
        if !tables.contains_key(source) || !tables.contains_key(target) {
            return Err(
                invalid(format!("missing skill relation table {source} or {target}")).into(),
            );
        }
    }
    let mut reports = Vec::new();
    for (source, field, target, target_field) in rules {
        reports.push(check(tables, source, field, target, target_field, None)?);
    }
    // These indices depend on the effect type, not on the spelling of the ID.
    for (effect_type, position, target) in [
        ("ESkillEffectType::Abnormal", 2, "SkillAbnormal"),
        ("ESkillEffectType::Abnormal_True", 2, "SkillAbnormal"),
        ("ESkillEffectType::Abnormal_Cc", 2, "SkillAbnormal"),
        ("ESkillEffectType::Abnormal_Passive", 0, "SkillAbnormal"),
        ("ESkillEffectType::Toggle", 0, "SkillAbnormal"),
        ("ESkillEffectType::CasterSkill", 0, "Skill"),
    ] {
        reports.push(check(
            tables,
            "SkillEffect",
            &format!("{effect_type}.EffectValueList[{position}]"),
            target,
            "ID.Value",
            Some((effect_type, position)),
        )?);
    }
    Ok(reports)
}

fn bounded_read(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(invalid("skill input exceeds 64 MiB").into());
    }
    Ok(bytes)
}

pub fn source_paths(directory: &Path) -> Vec<PathBuf> {
    TABLES
        .iter()
        .map(|(name, _)| directory.join(format!("{name}.bin")))
        .collect()
}

/// All profile tables are required, including those containing zero rows.
pub fn probe(mapping: &Mapping, directory: &Path) -> Result<SkillProbe> {
    let known: BTreeSet<_> = TABLES.iter().map(|(name, _)| *name).collect();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("bin"))
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && (stem.to_ascii_lowercase().contains("skill")
                || stem.to_ascii_lowercase().contains("abnormal"))
            && !known.contains(stem)
        {
            return Err(invalid(format!("unreviewed skill table {stem}")).into());
        }
    }
    let mut tables = BTreeMap::new();
    let mut total_bytes = 0usize;
    let mut total_rows = 0usize;
    for (name, structure) in TABLES {
        let payload = bounded_read(&directory.join(format!("{name}.bin")))?;
        total_bytes += payload.len();
        if total_bytes > 128 * 1024 * 1024 {
            return Err(invalid("skill batch exceeds 128 MiB").into());
        }
        let table = mapping
            .probe_table(structure, &payload)
            .map_err(|e| invalid(format!("{name}: {e}")))?;
        total_rows += table.layout.rows.len();
        if total_rows > 1_000_000 {
            return Err(invalid("skill batch exceeds one million rows").into());
        }
        tables.insert((*name).into(), table);
    }
    let relations = check_relations(&tables)?;
    Ok(SkillProbe {
        compatibility_verified: false,
        tables,
        relations,
    })
}
