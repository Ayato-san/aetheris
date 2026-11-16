//! Bounded content-addressed cache for decrypted/decoded intermediate bytes.
use super::{ClientIdentity, catalog::Result};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

const MAGIC: &[u8; 9] = b"AETHDATA1";
const MAX_BYTES: usize = 64 * 1024 * 1024;

pub struct Entry {
    pub bytes: Vec<u8>,
    pub reused: bool,
}

/// All inputs affecting meaning belong to the key, including exact schema
/// bytes and decoder revision. Client version/region are never inferred here.
pub fn decode(
    store: &Path,
    source: &[u8],
    schema: &[u8],
    client: &ClientIdentity,
    decoder_revision: &str,
    cancel: &AtomicBool,
    compute: impl FnOnce() -> Result<Vec<u8>>,
) -> Result<Entry> {
    let check = || -> Result<()> {
        if cancel.load(Ordering::Acquire) {
            return Err(
                io::Error::new(io::ErrorKind::Interrupted, "cache operation cancelled").into(),
            );
        }
        Ok(())
    };
    check()?;
    if decoder_revision.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "missing decoder revision").into());
    }
    fs::create_dir_all(store)?;
    let _lease = crate::companion::Lease::acquire(&store.join(".catalogue.lock"))?;
    let key = hex::encode(Sha256::digest(serde_json::to_vec(&(
        hex::encode(Sha256::digest(source)),
        hex::encode(Sha256::digest(schema)),
        client,
        decoder_revision,
    ))?));
    let directory = store.join("cache");
    fs::create_dir_all(&directory)?;
    let path = directory.join(format!("{key}.bin"));
    match fs::File::open(&path) {
        Ok(file) => {
            let mut bytes = Vec::new();
            file.take((MAX_BYTES + 42) as u64).read_to_end(&mut bytes)?;
            if bytes.len() < 41
                || bytes.len() > MAX_BYTES + 41
                || &bytes[..9] != MAGIC
                || Sha256::digest(&bytes[41..]).as_slice() != &bytes[9..41]
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "corrupt intermediate cache entry; clear the cache before retrying",
                )
                .into());
            }
            check()?;
            Ok(Entry {
                bytes: bytes.split_off(41),
                reused: true,
            })
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let bytes = compute()?;
            check()?;
            if bytes.len() > MAX_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "intermediate cache payload exceeds limit",
                )
                .into());
            }
            let mut encoded = Vec::with_capacity(41 + bytes.len());
            encoded.extend(MAGIC);
            encoded.extend(Sha256::digest(&bytes));
            encoded.extend(&bytes);
            crate::engine_ini::storage::atomic_replace(&path, &encoded, None)?;
            Ok(Entry {
                bytes,
                reused: false,
            })
        }
        Err(error) => Err(error.into()),
    }
}
