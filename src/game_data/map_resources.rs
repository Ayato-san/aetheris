//! Strict readers for already decrypted map resources. Coordinate meanings are
//! deliberately separate from structural decoding and never inferred here.
use super::{
    binary::{Reader, invalid},
    catalog::Result,
    schema::LayoutProbe,
    usmap::{Mapping, TableProbe},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub(crate) const DIRECTORIES: &[&str] = &["Map", "MapEvent", "MapDataHierarchy", "WorldMap"];
pub(crate) fn is_source(name: &str) -> bool {
    DIRECTORIES
        .iter()
        .any(|d| name.starts_with(&format!("{d}/")))
}
pub(crate) fn paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for name in DIRECTORIES {
        let dir = root.join(name);
        if !dir.is_dir() {
            continue;
        }
        let mut pending = vec![dir];
        let mut visited = 0;
        while let Some(dir) = pending.pop() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let ty = entry.file_type()?;
                visited += 1;
                if visited > 20000 {
                    return Err(invalid("map resource directory exceeds bounds").into());
                }
                if ty.is_symlink() {
                    return Err(invalid("symlink in map resource directory").into());
                }
                if ty.is_dir() {
                    pending.push(entry.path());
                } else if ty.is_file() && entry.path().extension().is_some_and(|e| e == "bin") {
                    files.push(entry.path());
                }
            }
        }
    }
    files.sort();
    Ok(files)
}
fn tag(r: &mut Reader<'_>, expected: u8) -> Result<()> {
    if r.u8()? != expected {
        return Err(invalid("invalid map hierarchy node tag").into());
    }
    Ok(())
}
/// Hierarchy v1 is a typed tree, not JSON. Preserve the last item word whose
/// role is unknown rather than silently skipping it as the reference does.
pub fn hierarchy(bytes: &[u8]) -> Result<Vec<Value>> {
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(invalid("map hierarchy exceeds limit").into());
    }
    let mut r = Reader::new(bytes);
    if r.u32()? != 1 {
        return Err(invalid("unsupported map hierarchy version").into());
    }
    let mut budget = 1_000_000usize;
    fn count(r: &mut Reader<'_>, budget: &mut usize) -> Result<usize> {
        let n = r.count(*budget)?;
        *budget -= n;
        Ok(n)
    }
    let mut worlds = Vec::new();
    for _ in 0..count(&mut r, &mut budget)? {
        let key = r.u32()?;
        tag(&mut r, 1)?;
        let map = r.u32()?;
        if key != map {
            return Err(invalid("hierarchy map ID mismatch").into());
        }
        let mut subzones = Vec::new();
        for _ in 0..count(&mut r, &mut budget)? {
            tag(&mut r, 2)?;
            let subzone = r.u32()?;
            let mut groups = Vec::new();
            for _ in 0..count(&mut r, &mut budget)? {
                tag(&mut r, 3)?;
                let kind = r.u32()?;
                let mut npcs = Vec::new();
                for _ in 0..count(&mut r, &mut budget)? {
                    tag(&mut r, 4)?;
                    let npc = r.u32()?;
                    let mut items = Vec::new();
                    for _ in 0..count(&mut r, &mut budget)? {
                        tag(&mut r, 5)?;
                        items.push(json!({"ItemId":r.u32()?,"UnknownWord":r.u32()?}));
                    }
                    npcs.push(json!({"NpcId":npc,"Items":items}));
                }
                groups.push(json!({"NpcTypeOrdinal":kind,"Npcs":npcs}));
            }
            subzones.push(json!({"SubzoneId":subzone,"Groups":groups}));
        }
        worlds.push(json!({"MapId":map,"Subzones":subzones}));
    }
    if r.pos != bytes.len() {
        return Err(invalid("unconsumed hierarchy bytes").into());
    }
    Ok(worlds)
}

pub(crate) struct Resources {
    pub files: Vec<PathBuf>,
    pub inputs: BTreeMap<PathBuf, String>,
    pub tables: BTreeMap<String, TableProbe>,
    pub errors: BTreeMap<String, (String, String)>,
}
pub(crate) fn read(root: &Path, mapping: &Mapping, mapping_hash: &str) -> Result<Resources> {
    let files = paths(root)?;
    let mut result = Resources {
        files,
        inputs: BTreeMap::new(),
        tables: BTreeMap::new(),
        errors: BTreeMap::new(),
    };
    let mut bytes_total = 0;
    for path in &result.files {
        let name = path
            .strip_prefix(root)?
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = super::target_export::read(path)?;
        bytes_total += bytes.len();
        if bytes_total > 256 * 1024 * 1024 {
            return Err(invalid("map resource batch exceeds 256 MiB").into());
        }
        let sha = hex::encode(Sha256::digest(&bytes));
        result.inputs.insert(path.clone(), sha.clone());
        let decode = || -> Result<TableProbe> {
            if name.starts_with("Map/") || name.starts_with("WorldMap/") {
                return Ok(mapping.probe_map(
                    if name.starts_with("Map/") {
                        "MapData"
                    } else {
                        "AionWorldMapExportInfo"
                    },
                    &bytes,
                )?);
            }
            let (rows, structure, version) = if name.starts_with("MapDataHierarchy/") {
                (hierarchy(&bytes)?, "MapHierarchy", 1)
            } else {
                let value: Value = serde_json::from_slice(&bytes)?;
                if !value.is_object() {
                    return Err(invalid("map event must be a JSON object").into());
                }
                (vec![value], "MapEvent", 0)
            };
            Ok(TableProbe {
                mapping_sha256: mapping_hash.into(),
                mapping_version: 0,
                mapping_ambiguous_structures: BTreeSet::new(),
                row_struct: structure.into(),
                payload_sha256: sha.clone(),
                payload_bytes: bytes.len(),
                layout: LayoutProbe {
                    compatibility_verified: false,
                    table_version: version,
                    column_hashes: vec![],
                    rows,
                },
            })
        };
        match decode() {
            Ok(t) => {
                result.tables.insert(name, t);
            }
            Err(e) => {
                result.errors.insert(name, ("failed".into(), e.to_string()));
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hierarchy_checks_tags_ids_counts_and_keeps_unknown_item_word() {
        let mut b = Vec::new();
        b.extend(1u32.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend(20u32.to_le_bytes());
        b.push(1);
        b.extend(20u32.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.push(2);
        b.extend(30u32.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.push(3);
        b.extend(1u32.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.push(4);
        b.extend(60u32.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.push(5);
        b.extend(70u32.to_le_bytes());
        b.extend(987u32.to_le_bytes());
        assert_eq!(
            hierarchy(&b).unwrap()[0]["Subzones"][0]["Groups"][0]["Npcs"][0]["Items"][0],
            json!({"ItemId":70,"UnknownWord":987})
        );
        for i in 0..b.len() {
            assert!(hierarchy(&b[..i]).is_err(), "truncation {i}");
        }
        let mut bad = b.clone();
        bad[12] = 2;
        assert!(hierarchy(&bad).is_err());
        let mut bad = b.clone();
        bad[13] = 21;
        assert!(hierarchy(&bad).is_err());
        let mut bad = b.clone();
        bad[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(hierarchy(&bad).is_err());
        let mut bad = b.clone();
        bad.push(0);
        assert!(hierarchy(&bad).is_err());
    }
    #[test]
    fn map_struct_header_is_distinct_from_tables_and_consumes_all_bytes() {
        use super::super::schema::{Field, Kind, probe_map_layout};
        let fields = || {
            vec![Field {
                name: "MapId".into(),
                kind: Kind::I32,
            }]
        };
        let b = [
            1u32.to_le_bytes().as_slice(),
            b"7",
            20i32.to_le_bytes().as_slice(),
        ]
        .concat();
        let decoded = probe_map_layout(fields(), &b).unwrap();
        assert_eq!(decoded.rows, vec![json!({"MapId":20})]);
        assert!(!decoded.compatibility_verified);
        for i in 0..b.len() {
            assert!(probe_map_layout(fields(), &b[..i]).is_err());
        }
        let mut bad = b.clone();
        bad.push(0);
        assert!(probe_map_layout(fields(), &bad).is_err());
        let bad = [
            3u32.to_le_bytes().as_slice(),
            b"7,8",
            20i32.to_le_bytes().as_slice(),
        ]
        .concat();
        assert!(probe_map_layout(fields(), &bad).is_err());
        let bad = [u32::MAX.to_le_bytes().as_slice(), b"7"].concat();
        assert!(probe_map_layout(fields(), &bad).is_err());
    }
}
