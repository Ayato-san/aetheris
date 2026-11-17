//! Aion 2 payload protection, independent of archive mounting and row schemas.
//! Key material is supplied locally; this module ships no client keys.
use aes::{
    Aes256, Block,
    cipher::{BlockDecrypt, BlockEncrypt, KeyInit},
};
use std::{collections::BTreeMap, io};

const MAX_PAYLOAD: usize = 256 * 1024 * 1024;

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
    version_context: Option<[u8; 32]>,
    app_version: Option<String>,
}
impl KeyRing {
    /// Current manifests carry an opaque eight-byte prefix, followed by 48-byte
    /// encrypted records. This is a distinct format, never a fallback for a
    /// rejected legacy manifest.
    pub fn from_current_manifest(bytes: &[u8], material: &[u8; 32]) -> io::Result<Self> {
        if bytes.len() < 8 || bytes.len() > MAX_PAYLOAD || !(bytes.len() - 8).is_multiple_of(48) {
            return Err(invalid("invalid current key manifest length"));
        }
        let plaintext = ecb(&bytes[8..], blake3::hash(material).as_bytes())?;
        Self::from_current_records(&plaintext)
    }

    fn from_current_records(plaintext: &[u8]) -> io::Result<Self> {
        if plaintext.is_empty()
            || !plaintext.len().is_multiple_of(48)
            || plaintext
                .as_chunks::<48>()
                .0
                .iter()
                .any(|r| r[40..] != [0; 8])
        {
            return Err(invalid("invalid reserved key manifest record tails"));
        }
        let mut keys = BTreeMap::new();
        for record in plaintext.as_chunks::<48>().0 {
            let id = seed(record, 0)?;
            let value: [u8; 32] = record[8..40].try_into().unwrap();
            if let Some(previous) = keys.insert(id, value)
                && previous != value
            {
                return Err(invalid(format!("conflicting key manifest seed {id:016x}")));
            }
        }
        if keys.is_empty() {
            return Err(invalid("empty key manifest"));
        }
        Ok(Self {
            keys,
            version_context: None,
            app_version: None,
        })
    }

    /// The manifest contains no AppVersion. Accept a numeric candidate only
    /// when every record has the expected reserved zero tail; never infer a
    /// supported client/schema identity from this cryptographic discriminator.
    pub fn from_versioned_manifest(bytes: &[u8], material: &[u8; 32]) -> io::Result<Self> {
        if bytes.len() < 56 || bytes.len() > MAX_PAYLOAD || !(bytes.len() - 8).is_multiple_of(48) {
            return Err(invalid("invalid versioned key manifest length"));
        }
        if let Ok(keys) = Self::from_current_manifest(bytes, material) {
            return Ok(keys);
        }
        for major in 0..10 {
            for minor in 0..10 {
                for patch in 0..1000 {
                    let version = format!("{major}.{minor}.{patch}");
                    let context =
                        *blake3::hash(format!("AION2.TableKey.AppVersion{version}").as_bytes())
                            .as_bytes();
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(material);
                    hasher.update(&context);
                    let key = hasher.finalize();
                    let first = ecb(&bytes[8..56], key.as_bytes())?;
                    if first[40..] != [0; 8] {
                        continue;
                    }
                    let plaintext = ecb(&bytes[8..], key.as_bytes())?;
                    if let Ok(mut keys) = Self::from_current_records(&plaintext) {
                        keys.version_context = Some(context);
                        keys.app_version = Some(version);
                        return Ok(keys);
                    }
                }
            }
        }
        Err(invalid(
            "manifest key not found in supported numeric AppVersion range (0..9.0..9.0..999)",
        ))
    }

    pub fn app_version(&self) -> Option<&str> {
        self.app_version.as_deref()
    }

    fn permutation_root(&self) -> Option<u64> {
        self.version_context.map(|context| {
            let mut bytes = 0xCD02190910CE83F6u64.to_le_bytes().to_vec();
            bytes.extend(context);
            hash_seed(&bytes)
        })
    }

    fn header_key(&self, seed: u64, kind: i32) -> [u8; 32] {
        let root = self.permutation_root().map_or_else(
            || hash_seed(&0xCD02190910CE83F6u64.to_le_bytes()),
            |root| hash_seed(&root.to_le_bytes()),
        );
        header_key_with_root(root, seed, kind)
    }

    pub fn current_table(&self, bytes: &[u8], virtual_path: &str) -> io::Result<Vec<u8>> {
        if bytes.len() < 12 || bytes.len() > MAX_PAYLOAD || i32_at(bytes, 0)? != 13 {
            return Err(invalid("invalid current table header"));
        }
        let normalized = virtual_path.replace('\\', "/");
        let name = normalized
            .rsplit('/')
            .next()
            .and_then(|n| n.strip_suffix(".dat"))
            .filter(|n| !n.is_empty())
            .ok_or_else(|| invalid("missing table definition name"))?;
        let id = hash_seed(name.as_bytes());
        let key = self.key(id)?;
        let compressed_key = self.header_key(id, 1);
        let prefix = xor_header(&bytes[4..12], &compressed_key);
        if i32_at(&prefix, 4)? == 2 {
            if bytes.len() < 20 {
                return Err(invalid("truncated compressed table header"));
            }
            let suffix = xor_header(&bytes[12..20], &compressed_key[8..]);
            return current_compressed(
                &bytes[20..],
                size(&prefix, 0)?,
                size(&suffix, 0)?,
                size(&suffix, 4)?,
                key,
            );
        }
        let header = xor_header(&bytes[4..12], &self.header_key(id, 2));
        let length = size(&header, 0)?;
        if i32_at(&header, 4)? != 3 || length == 0 || length != bytes.len() - 12 {
            return Err(invalid("unsupported current table cipher or stream length"));
        }
        let mut nonce_input = key.to_vec();
        nonce_input.extend(b"nonc");
        let nonce = hash_seed(&nonce_input);
        let cipher = Aes256::new_from_slice(key).unwrap();
        let mut output = Vec::with_capacity(length);
        for (index, chunk) in bytes[12..].chunks(16).enumerate() {
            let mut counter = [0u8; 16];
            counter[..8].copy_from_slice(&nonce.to_le_bytes());
            counter[8..].copy_from_slice(&(index as u64).to_le_bytes());
            let mut block = Block::from(counter);
            cipher.encrypt_block(&mut block);
            output.extend(chunk.iter().zip(block).map(|(a, b)| a ^ b));
        }
        Ok(output)
    }

    pub fn current_localization(&self, bytes: &[u8], virtual_path: &str) -> io::Result<Vec<u8>> {
        if bytes.len() < 20 || bytes.len() > MAX_PAYLOAD || i32_at(bytes, 0)? != 2 {
            return Err(invalid("invalid current localisation header"));
        }
        let normalized = virtual_path.replace('\\', "/");
        let parts: Vec<_> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        let locale = parts
            .get(
                parts
                    .len()
                    .checked_sub(2)
                    .ok_or_else(|| invalid("localisation locale missing"))?,
            )
            .ok_or_else(|| invalid("localisation locale missing"))?;
        let id = hash_seed(format!("L10NString_{locale}").as_bytes());
        let mut header = xor_header(&bytes[4..20], &self.header_key(id, 3));
        if let Some(root) = self.permutation_root() {
            let mut input = root.to_le_bytes().to_vec();
            input.extend(0x51i32.to_le_bytes());
            let shuffle = blake3::hash(&input);
            let mut permutation = [0usize, 1, 2, 3];
            for i in (1..4).rev() {
                permutation.swap(i, shuffle.as_bytes()[3 - i] as usize % (i + 1));
            }
            let mut fields = [0i32; 4];
            for (i, &field) in permutation.iter().enumerate() {
                fields[field] = i32_at(&header, i * 4)?;
            }
            // Normalize to the legacy order: packed, cipher, aligned, raw.
            header = [fields[2], fields[0], fields[3], fields[1]]
                .into_iter()
                .flat_map(i32::to_le_bytes)
                .collect();
        }
        if i32_at(&header, 4)? != 2 {
            return Err(invalid("unsupported current localisation cipher"));
        }
        current_compressed(
            &bytes[20..],
            size(&header, 0)?,
            size(&header, 8)?,
            size(&header, 12)?,
            self.key(id)?,
        )
    }
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
        Ok(Self {
            keys,
            version_context: None,
            app_version: None,
        })
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

fn hash_seed(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(blake3::hash(bytes).as_bytes()[..8].try_into().unwrap())
}
fn header_key_with_root(root: u64, seed: u64, kind: i32) -> [u8; 32] {
    let mut bytes = [0u8; 24];
    bytes[..8].copy_from_slice(&root.to_le_bytes());
    bytes[8..16].copy_from_slice(&seed.to_le_bytes());
    bytes[16..20].copy_from_slice(&kind.to_le_bytes());
    *blake3::hash(&bytes).as_bytes()
}
fn xor_header(bytes: &[u8], key: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect()
}
fn current_compressed(
    bytes: &[u8],
    packed: usize,
    aligned: usize,
    raw: usize,
    key: &[u8; 32],
) -> io::Result<Vec<u8>> {
    if packed == 0
        || raw == 0
        || !aligned.is_multiple_of(16)
        || aligned != bytes.len()
        || packed.checked_add(32).is_none_or(|n| n > aligned)
    {
        return Err(invalid("invalid current compressed lengths"));
    }
    let plaintext = ecb(bytes, key)?;
    let output = lz4_flex::block::decompress(&plaintext[32..32 + packed], raw)
        .map_err(|e| invalid(format!("invalid current LZ4 payload: {e}")))?;
    if output.len() != raw {
        return Err(invalid("current LZ4 output length mismatch"));
    }
    Ok(output)
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
