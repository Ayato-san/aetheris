//! Protected assets with provenance from a verified archive extraction report.
use super::{
    archives::Extraction,
    binary::{Reader, invalid},
    protection::{KeyRing, map_payload},
    reader::{Control, Phase},
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
    /// Cryptographic AppVersion candidate; not verified installation metadata.
    #[serde(default)]
    pub app_version: Option<String>,
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
    // Versioned Steam files include one reserved zero word after the map.
    // Preserve the raw payload and reject any other trailing content.
    let trailing = &payload[r.pos..];
    if !trailing.is_empty() && trailing != [0; 4] {
        return Err(invalid("trailing localisation bytes"));
    }
    Ok(Localization { namespace, entries })
}

pub fn decrypt_extraction(
    source: &Path,
    material: &[u8; 32],
    output: &Path,
) -> io::Result<Decryption> {
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let mut progress = |_| {};
    decrypt_extraction_with_control(
        source,
        material,
        output,
        &mut Control::new(&cancel, &mut progress),
    )
}
pub fn decrypt_extraction_with_control(
    source: &Path,
    material: &[u8; 32],
    output: &Path,
    control: &mut Control<'_>,
) -> io::Result<Decryption> {
    control.check()?;
    let source = source.canonicalize()?;
    let output = super::archives::output_directory(&source, output)?;
    crate::engine_ini::storage::atomic_replace(
        &output.join("decryption.json"),
        br#"{"files":[],"errors":["decryption incomplete"]}"#,
        None,
    )?;
    let mut source_report = Vec::new();
    fs::File::open(source.join("extraction.json"))?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut source_report)?;
    if source_report.len() > 16 * 1024 * 1024 {
        return Err(invalid("extraction report exceeds limit"));
    }
    let extracted: Extraction = serde_json::from_slice(&source_report)?;
    if extracted.files.len() > 100_000 {
        return Err(invalid("extraction file count exceeds limit"));
    }
    let mut paths = std::collections::BTreeSet::new();
    if extracted
        .files
        .iter()
        .any(|f| !paths.insert(f.path.to_lowercase()))
    {
        return Err(invalid("duplicate extracted asset path"));
    }
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
    let keys =
        KeyRing::from_versioned_manifest_with_control(&read(manifests[0])?, material, control)?;
    let mut report = Decryption {
        app_version: keys.app_version().map(str::to_owned),
        files: Vec::new(),
        errors: Vec::new(),
    };
    for (index, file) in extracted.files.iter().enumerate() {
        control.emit(
            Phase::Decrypt,
            index,
            extracted.files.len(),
            Some(&file.path),
        )?;
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
            } else if lower.contains("/data/worldmap/") {
                ("world_map", map_payload(&bytes)?)
            } else {
                return Err(invalid("unsupported protected asset family"));
            };
            control.check()?;
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
            Err(error) if error.kind() == io::ErrorKind::Interrupted => return Err(error),
            Err(error) => report.errors.push(format!("{}: {error}", file.path)),
        }
    }
    control.emit(
        Phase::Decrypt,
        extracted.files.len(),
        extracted.files.len(),
        None,
    )?;
    crate::engine_ini::storage::atomic_replace(
        &output.join("decryption.json"),
        &serde_json::to_vec_pretty(&report)?,
        None,
    )?;
    Ok(report)
}
