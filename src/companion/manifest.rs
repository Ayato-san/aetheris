//! The signature authenticates the exact manifest bytes, including version and file names.
use super::{Result, error};
use ed25519_dalek::{Signature, VerifyingKey};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::Read, path::Path};

pub const MANIFEST: &str = "release.json";
pub const SIGNATURE: &str = "release.sig";
pub const PROGRAMS: [&str; 3] = [
    "aetheris.exe",
    "aetheris-companion.exe",
    "aetheris-update-helper.exe",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: Version,
    pub target: String,
    pub files: BTreeMap<String, String>,
}

pub fn public_key() -> Result<VerifyingKey> {
    let encoded = option_env!("AETHERIS_PUBLIC_KEY").unwrap_or("");
    let bytes: [u8; 32] = hex::decode(encoded)?
        .try_into()
        .map_err(|_| error("Clé publique de distribution non configurée."))?;
    Ok(VerifyingKey::from_bytes(&bytes)?)
}

pub fn hash(path: &Path) -> Result<String> {
    let mut input = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hex::encode(hash.finalize()))
}

pub fn authenticate(directory: &Path, key: &VerifyingKey) -> Result<Manifest> {
    let bytes = fs::read(directory.join(MANIFEST)).map_err(|e| {
        error(format!(
            "Intégrité impossible à établir : manifeste absent ou illisible ({e})."
        ))
    })?;
    if bytes.len() > 65536 {
        return Err(error("Manifeste trop volumineux."));
    }
    let signature_bytes = fs::read(directory.join(SIGNATURE)).map_err(|e| {
        error(format!(
            "Intégrité impossible à établir : signature absente ou illisible ({e})."
        ))
    })?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| error("Intégrité impossible à établir : signature mal formée."))?;
    key.verify_strict(&bytes, &signature)
        .map_err(|_| error("Intégrité impossible à établir : signature invalide."))?;
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    if manifest.target != "x86_64-pc-windows-msvc"
        || manifest.files.len() != PROGRAMS.len()
        || PROGRAMS
            .iter()
            .any(|name| !manifest.files.contains_key(*name))
    {
        return Err(error(
            "Contenu du manifeste de distribution non pris en charge.",
        ));
    }
    Ok(manifest)
}

pub fn verify(directory: &Path, key: &VerifyingKey) -> Result<Manifest> {
    let manifest = authenticate(directory, key)?;
    for (name, expected) in &manifest.files {
        let path = directory.join(name);
        if fs::symlink_metadata(&path)
            .map_err(|e| {
                error(format!(
                    "Fichier programme absent ou illisible : {name} ({e}). Réparation nécessaire."
                ))
            })?
            .file_type()
            .is_symlink()
            || hash(&path)? != *expected
        {
            return Err(error(format!(
                "Fichier programme modifié ou corrompu : {name}. Réparation nécessaire."
            )));
        }
    }
    Ok(manifest)
}
