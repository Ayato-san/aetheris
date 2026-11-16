//! Aion 2 payload protection, independent of archive mounting and row schemas.
//! Key material is supplied locally; this module ships no client keys.
use aes::{
    Aes256, Block,
    cipher::{BlockDecrypt, BlockEncrypt, KeyInit},
};
use std::{collections::BTreeMap, io};

const MAX_PAYLOAD: usize = 64 * 1024 * 1024;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn i32_at(bytes: &[u8], offset: usize) -> io::Result<i32> {
    Ok(i32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(|| invalid("truncated protection header"))?
            .try_into()
            .unwrap(),
    ))
}
fn size(bytes: &[u8], offset: usize) -> io::Result<usize> {
    usize::try_from(i32_at(bytes, offset)?)
        .ok()
        .filter(|s| *s <= MAX_PAYLOAD)
        .ok_or_else(|| invalid("invalid or excessive protection length"))
}
fn seed(bytes: &[u8], offset: usize) -> io::Result<u64> {
    Ok(u64::from_le_bytes(
        bytes
            .get(offset..offset + 8)
            .ok_or_else(|| invalid("truncated key seed"))?
            .try_into()
            .unwrap(),
    ))
}
fn ecb(bytes: &[u8], key: &[u8; 32]) -> io::Result<Vec<u8>> {
    if !bytes.len().is_multiple_of(16) {
        return Err(invalid("AES payload is not block aligned"));
    }
    let cipher = Aes256::new_from_slice(key).unwrap();
    let mut output = Vec::with_capacity(bytes.len());
    for chunk in bytes.as_chunks::<16>().0 {
        let mut block = Block::from(*chunk);
        cipher.decrypt_block(&mut block);
        output.extend_from_slice(&block);
    }
    Ok(output)
}

/// Intentionally has no Debug/Serialize implementation: keys must not appear
/// in UI diagnostics, catalogue manifests or local build reports.
pub struct KeyRing {
    keys: BTreeMap<u64, [u8; 32]>,
}
impl KeyRing {
    pub fn from_manifest(bytes: &[u8], material: &[u8; 32]) -> io::Result<Self> {
        if bytes.len() > MAX_PAYLOAD || i32_at(bytes, 0)? != 2 {
            return Err(invalid("unsupported key manifest version or size"));
        }
        let count = size(bytes, 4)?;
        let length = size(bytes, 8)?;
        if count.checked_mul(48) != Some(length) || bytes.len() != 12 + length {
            return Err(invalid("invalid key manifest length"));
        }
        let derived = blake3::hash(material);
        let plaintext = ecb(&bytes[12..], derived.as_bytes())?;
        let mut keys = BTreeMap::new();
        for entry in plaintext.as_chunks::<48>().0 {
            let id = seed(entry, 0)?;
            if keys.insert(id, entry[8..40].try_into().unwrap()).is_some() {
                return Err(invalid("duplicate key manifest seed"));
            }
        }
        Ok(Self { keys })
    }

    fn key(&self, seed: u64) -> io::Result<&[u8; 32]> {
        self.keys
            .get(&seed)
            .ok_or_else(|| invalid(format!("content key missing for seed {seed:016x}")))
    }

    /// Table marker 13; v2 uses AES-ECB then raw LZ4, v3 an AES-derived
    /// counter stream with a little-endian 64-bit counter in the final half.
    pub fn table(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        if bytes.len() > MAX_PAYLOAD || bytes.len() < 28 || i32_at(bytes, 0)? != 13 {
            return Err(invalid("invalid table protection header"));
        }
        let version = i32_at(bytes, 4)?;
        if version != 2 && version != 3 {
            return Err(invalid(format!(
                "unsupported table cipher version {version}"
            )));
        }
        let key = self.key(seed(bytes, 8)?)?;
        if version == 2 {
            return compressed(bytes, 28, 16, key);
        }
        let length = size(bytes, 24)?;
        if bytes.len() != 28 + length {
            return Err(invalid("counter stream length mismatch"));
        }
        let cipher = Aes256::new_from_slice(key).unwrap();
        let mut output = Vec::with_capacity(length);
        for (index, chunk) in bytes[28..].chunks(16).enumerate() {
            let mut counter = [0u8; 16];
            counter[..8].copy_from_slice(&bytes[16..24]);
            counter[8..].copy_from_slice(&(index as u64).to_le_bytes());
            let mut block = Block::from(counter);
            cipher.encrypt_block(&mut block);
            output.extend(chunk.iter().zip(block).map(|(a, b)| a ^ b));
        }
        Ok(output)
    }

    /// Version-2 localisation has a distinct 24-byte header and seed offset.
    pub fn localization(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        if bytes.len() > MAX_PAYLOAD || bytes.len() < 24 || i32_at(bytes, 0)? != 2 {
            return Err(invalid("unsupported localisation protection header"));
        }
        compressed(bytes, 24, 12, self.key(seed(bytes, 4)?)?)
    }
}

fn compressed(bytes: &[u8], header: usize, lengths: usize, key: &[u8; 32]) -> io::Result<Vec<u8>> {
    let raw = size(bytes, lengths)?;
    let packed = size(bytes, lengths + 4)?;
    let aligned = size(bytes, lengths + 8)?;
    if raw == 0
        || packed == 0
        || !aligned.is_multiple_of(16)
        || packed.checked_add(16).is_none_or(|n| n > aligned)
        || bytes.len() != header + aligned
    {
        return Err(invalid("invalid compressed protection lengths"));
    }
    let plaintext = ecb(&bytes[header..], key)?;
    let output = lz4_flex::block::decompress(&plaintext[16..16 + packed], raw)
        .map_err(|e| invalid(format!("invalid protected LZ4 block: {e}")))?;
    if output.len() != raw {
        return Err(invalid("decompressed length mismatch"));
    }
    Ok(output)
}

/// MapData/MapEvent use their own fixed XOR branch. Callers must select this
/// from a verified file family; never apply it to arbitrary .dat files.
pub fn map_payload(bytes: &[u8]) -> io::Result<Vec<u8>> {
    if bytes.len() > MAX_PAYLOAD {
        return Err(invalid("map payload exceeds limit"));
    }
    Ok(bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ [0x25, 0xa8, 0x7e, 0x91][i % 4])
        .collect())
}
