//! Protected assets with provenance from a verified archive extraction report.
use super::{
    archives::Extraction,
    binary::{Reader, invalid},
    protection::{KeyRing, map_payload},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    path::Path,
};

#[derive(Serialize, Deserialize)]
pub struct Localization {
    pub namespace: String,
    pub entries: BTreeMap<String, String>,
}
#[derive(Serialize, Deserialize)]
pub struct DecodedAsset {
    pub path: String,
    pub source_sha256: String,
    pub output_sha256: String,
    pub family: String,
    pub bytes: usize,
}
#[derive(Serialize, Deserialize)]
pub struct Decryption {
    pub files: Vec<DecodedAsset>,
    pub errors: Vec<String>,
}

pub fn localization(payload: &[u8]) -> io::Result<Localization> {
    let mut r = Reader::new(payload);
    if r.u32()? != 1 {
        return Err(invalid("unsupported localisation table version"));
    }
    let namespace = r.string()?;
    let mut entries = BTreeMap::new();
    for _ in 0..r.count(1_000_000)? {
        let key = r.string()?;
        let text = r.string()?;
        if entries.insert(key, text).is_some() {
            return Err(invalid("duplicate localisation key"));
        }
    }
    if r.pos != payload.len() {
        return Err(invalid("trailing localisation bytes"));
    }
    Ok(Localization { namespace, entries })
}

pub fn decrypt_extraction(
    source: &Path,
    material: &[u8; 32],
    output: &Path,
) -> io::Result<Decryption> {
    let source = source.canonicalize()?;
    fs::create_dir_all(output)?;
    let output = output.canonicalize()?;
    if output.starts_with(&source) || source.starts_with(&output) {
        return Err(invalid(
            "decryption source and destination must not overlap",
        ));
    }
    crate::engine_ini::storage::atomic_replace(
        &output.join("decryption.json"),
        br#"{"files":[],"errors":["decryption incomplete"]}"#,
        None,
    )?;
    let extracted: Extraction = serde_json::from_slice(&fs::read(source.join("extraction.json"))?)?;
    if !extracted.errors.is_empty() {
        return Err(invalid("source extraction is incomplete"));
    }
    let manifests: Vec<_> = extracted
        .files
        .iter()
        .filter(|f| f.path.to_lowercase().ends_with("/key_manifest.dat"))
        .collect();
    if manifests.len() != 1 {
        return Err(invalid("exactly one extracted key manifest is required"));
    }
    let read = |asset: &super::archives::Extracted| -> io::Result<Vec<u8>> {
        if asset
            .path
            .split('/')
            .any(|p| p.is_empty() || p == ".." || p == ".")
            || asset.path.contains(['\\', ':'])
        {
            return Err(invalid("unsafe extracted asset path"));
        }
        if asset.size > 256 * 1024 * 1024 {
            return Err(invalid("extracted asset exceeds allocation limit"));
        }
        let path = source.join(&asset.path).canonicalize()?;
        if !path.starts_with(&source) {
            return Err(invalid("extracted asset resolves outside source"));
        }
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(asset.size + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != asset.size || hex::encode(Sha256::digest(&bytes)) != asset.sha256 {
            return Err(invalid("extracted asset no longer matches provenance"));
        }
        Ok(bytes)
    };
    let keys = KeyRing::from_current_manifest(&read(manifests[0])?, material)?;
    let mut report = Decryption {
        files: Vec::new(),
        errors: Vec::new(),
    };
    for file in &extracted.files {
        let lower = file.path.to_lowercase();
        if !lower.ends_with(".dat") || lower.ends_with("/key_manifest.dat") {
            continue;
        }
        let result = (|| -> io::Result<DecodedAsset> {
            let bytes = read(file)?;
            let (family, payload) = if lower.contains("/l10n/") {
                let raw = keys.current_localization(&bytes, &file.path)?;
                let strings = localization(&raw)?;
                let json = super::archives::destination(
                    &output,
                    &Path::new(&file.path).with_extension("json"),
                )?;
                crate::engine_ini::storage::atomic_replace(
                    &json,
                    &serde_json::to_vec(&strings)?,
                    None,
                )?;
                ("localization", raw)
            } else if lower.contains("/data/table/") || lower.contains("/startupdata/table/") {
                ("table", keys.current_table(&bytes, &file.path)?)
            } else if lower.contains("/data/mapdatahierarchy/") {
                ("map_hierarchy", bytes)
            } else if lower.contains("/data/mapevent/") {
                let payload = map_payload(&bytes)?;
                serde_json::from_slice::<serde_json::Value>(&payload)?;
                ("map_event", payload)
            } else if lower.contains("/data/map/") && lower.ends_with("/mapdata.dat") {
                ("map", map_payload(&bytes)?)
            } else {
                return Err(invalid("unsupported protected asset family"));
            };
            let destination = super::archives::destination(
                &output,
                &Path::new(&file.path).with_extension("bin"),
            )?;
            crate::engine_ini::storage::atomic_replace(&destination, &payload, None)?;
            Ok(DecodedAsset {
                path: file.path.clone(),
                source_sha256: file.sha256.clone(),
                output_sha256: hex::encode(Sha256::digest(&payload)),
                family: family.into(),
                bytes: payload.len(),
            })
        })();
        match result {
            Ok(asset) => report.files.push(asset),
            Err(error) => report.errors.push(format!("{}: {error}", file.path)),
        }
    }
    crate::engine_ini::storage::atomic_replace(
        &output.join("decryption.json"),
        &serde_json::to_vec_pretty(&report)?,
        None,
    )?;
    Ok(report)
}
