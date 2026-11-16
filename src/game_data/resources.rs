//! Signed, independently versioned data resources. Installation is entirely
//! local; callers can download a bundle separately without uploading game data.
use super::{
    ClientIdentity,
    catalog::{Catalog, Result},
    schema::Schema,
};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

const MANIFEST: &str = "resources.json";
const SIGNATURE: &str = "resources.sig";
const MAX_MANIFEST: usize = 64 * 1024;
const MAX_FILE: u64 = 512 * 1024 * 1024;
const MAX_BUNDLE: u64 = 1024 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub revision: u64,
    pub compatible_clients: Vec<ClientIdentity>,
    pub files: Vec<Resource>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub path: String,
    pub sha256: String,
    pub size: u64,
    pub kind: Kind,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Schema,
    Mapping,
    Catalogue,
}

pub struct Bundle {
    pub directory: PathBuf,
    pub manifest: Manifest,
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(invalid("resource exceeds size limit").into());
    }
    Ok(bytes)
}
fn hash_key(key: &str) -> bool {
    key.len() == 64 && key.bytes().all(|b| b.is_ascii_hexdigit())
}
fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':', '\0'])
        && path
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != ".." && !p.ends_with(['.', ' ']))
        && path != MANIFEST
        && path != SIGNATURE
}

fn authenticate(bytes: &[u8], signature: &[u8], key: &VerifyingKey) -> Result<Manifest> {
    if bytes.len() > MAX_MANIFEST {
        return Err(invalid("resource manifest exceeds limit").into());
    }
    key.verify_strict(bytes, &Signature::from_slice(signature)?)
        .map_err(|_| invalid("invalid resource signature"))?;
    let manifest: Manifest = serde_json::from_slice(bytes)?;
    if manifest.format != 1
        || manifest.revision == 0
        || manifest.compatible_clients.is_empty()
        || manifest.files.is_empty()
    {
        return Err(invalid("unsupported or incomplete resource manifest").into());
    }
    if manifest
        .compatible_clients
        .iter()
        .any(|c| c.version.is_empty() || c.region.is_empty())
    {
        return Err(invalid("empty resource client identity").into());
    }
    let mut paths = BTreeSet::new();
    let mut total = 0u64;
    for file in &manifest.files {
        total = total
            .checked_add(file.size)
            .ok_or_else(|| invalid("resource bundle size overflow"))?;
        if !valid_path(&file.path)
            || !hash_key(&file.sha256)
            || file.size > MAX_FILE
            || total > MAX_BUNDLE
            || !paths.insert(file.path.to_lowercase())
        {
            return Err(invalid("invalid resource path, size or duplicate file").into());
        }
    }
    Ok(manifest)
}

fn check_path(root: &Path, path: &str) -> Result<PathBuf> {
    let mut current = root.to_path_buf();
    if !fs::symlink_metadata(root)?.is_dir() {
        return Err(invalid("resource root is not a directory").into());
    }
    for part in path.split('/') {
        current.push(part);
        if fs::symlink_metadata(&current)?.file_type().is_symlink() {
            return Err(invalid("symbolic resource paths are not supported").into());
        }
    }
    if !fs::metadata(&current)?.is_file() {
        return Err(invalid("resource is not a regular file").into());
    }
    Ok(current)
}

fn verify_files(directory: &Path, manifest: &Manifest) -> Result<()> {
    for file in &manifest.files {
        let path = check_path(directory, &file.path)?;
        if fs::metadata(&path)?.len() != file.size
            || crate::companion::manifest::hash(&path)? != file.sha256
        {
            return Err(invalid("resource checksum or size mismatch").into());
        }
        match file.kind {
            Kind::Schema => {
                let schema: Schema =
                    serde_json::from_slice(&read_bounded(&path, MAX_MANIFEST * 16)?)?;
                if schema.id.is_empty()
                    || schema.compatible_clients.is_empty()
                    || schema
                        .compatible_clients
                        .iter()
                        .any(|c| !manifest.compatible_clients.contains(c))
                {
                    return Err(
                        invalid("schema compatibility is not covered by signed manifest").into(),
                    );
                }
            }
            Kind::Catalogue => {
                let catalog = Catalog::open(&path)?;
                if !catalog
                    .build
                    .inventory
                    .client
                    .as_ref()
                    .is_some_and(|c| manifest.compatible_clients.contains(c))
                {
                    return Err(invalid(
                        "catalogue compatibility is not covered by signed manifest",
                    )
                    .into());
                }
            }
            Kind::Mapping => (), // Integrity checked; parsing/compatibility still belongs to its reader.
        }
    }
    Ok(())
}

struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Authenticity, all file hashes and client compatibility are checked before
/// changing a pointer. Earlier bundles remain available after any failure.
pub fn install(
    source: &Path,
    store: &Path,
    client: &ClientIdentity,
    key: &VerifyingKey,
) -> Result<Bundle> {
    let bytes = read_bounded(&source.join(MANIFEST), MAX_MANIFEST)?;
    let signature = read_bounded(&source.join(SIGNATURE), 64)?;
    let manifest = authenticate(&bytes, &signature, key)?;
    if !manifest.compatible_clients.contains(client) {
        return Err(invalid("resource bundle incompatible with selected client").into());
    }
    fs::create_dir_all(store)?;
    let _lease = crate::companion::Lease::acquire(&store.join(".resources.lock"))?;
    if let Some(id) = active_id(store)? {
        // Authenticate the previous revision without requiring its payloads to
        // be intact: a new signed bundle must be able to repair damaged data.
        let directory = store.join("versions").join(id);
        let previous_bytes = read_bounded(&directory.join(MANIFEST), MAX_MANIFEST)?;
        let previous_signature = read_bounded(&directory.join(SIGNATURE), 64)?;
        let previous = authenticate(&previous_bytes, &previous_signature, key)?;
        if previous.revision > manifest.revision {
            return Err(invalid("resource rollback rejected").into());
        }
        if previous.revision == manifest.revision && previous_bytes != bytes {
            return Err(invalid("resource revision reused with different content").into());
        }
    }
    let versions = store.join("versions");
    fs::create_dir_all(&versions)?;
    let mut random = [0; 16];
    getrandom::fill(&mut random).map_err(|e| io::Error::other(e.to_string()))?;
    let staging = Staging(versions.join(format!(".resources-{}", hex::encode(random))));
    fs::create_dir(&staging.0)?;
    for file in &manifest.files {
        let source = check_path(source, &file.path)?;
        let destination = staging.0.join(&file.path);
        fs::create_dir_all(destination.parent().unwrap())?;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)?;
        let mut input = fs::File::open(source)?.take(file.size + 1);
        if io::copy(&mut input, &mut output)? != file.size {
            return Err(invalid("resource changed while copying").into());
        }
        sync(&output)?;
    }
    for (name, contents) in [
        (MANIFEST, bytes.as_slice()),
        (SIGNATURE, signature.as_slice()),
    ] {
        let mut output = fs::File::create(staging.0.join(name))?;
        output.write_all(contents)?;
        sync(&output)?;
    }
    verify_files(&staging.0, &manifest)?;
    let id = hex::encode(Sha256::digest(&bytes));
    let directory = versions.join(&id);
    if directory.exists() {
        let existing = read_bundle(&directory, key)?;
        if read_bounded(&existing.directory.join(MANIFEST), MAX_MANIFEST)? != bytes {
            return Err(invalid("resource store content mismatch").into());
        }
    } else {
        fs::rename(&staging.0, &directory)?;
    }
    crate::engine_ini::storage::atomic_replace(
        &store.join("active.json"),
        &serde_json::to_vec(&id)?,
        None,
    )?;
    Ok(Bundle {
        directory,
        manifest,
    })
}

fn sync(file: &fs::File) -> io::Result<()> {
    match file.sync_all() {
        Err(error)
            if error.kind() == io::ErrorKind::Unsupported
                || (cfg!(windows) && error.raw_os_error() == Some(50)) =>
        {
            Ok(())
        }
        result => result,
    }
}
fn read_bundle(directory: &Path, key: &VerifyingKey) -> Result<Bundle> {
    let bytes = read_bounded(&directory.join(MANIFEST), MAX_MANIFEST)?;
    let signature = read_bounded(&directory.join(SIGNATURE), 64)?;
    let manifest = authenticate(&bytes, &signature, key)?;
    verify_files(directory, &manifest)?;
    Ok(Bundle {
        directory: directory.into(),
        manifest,
    })
}

/// Offline verification uses the pinned public key and the installed bytes;
/// an unavailable update source never requires deleting these resources.
pub fn active(store: &Path, key: &VerifyingKey) -> Result<Option<Bundle>> {
    let Some(id) = active_id(store)? else {
        return Ok(None);
    };
    let directory = store.join("versions").join(&id);
    let bundle = read_bundle(&directory, key)?;
    if hex::encode(Sha256::digest(read_bounded(
        &directory.join(MANIFEST),
        MAX_MANIFEST,
    )?)) != id
    {
        return Err(invalid("resource pointer hash mismatch").into());
    }
    Ok(Some(bundle))
}

fn active_id(store: &Path) -> Result<Option<String>> {
    let pointer = store.join("active.json");
    let bytes = match read_bounded(&pointer, 1024) {
        Ok(bytes) => bytes,
        Err(error)
            if error
                .downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::NotFound) =>
        {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    let id: String = serde_json::from_slice(&bytes)?;
    if !hash_key(&id) {
        return Err(invalid("invalid active resource pointer").into());
    }
    Ok(Some(id))
}
