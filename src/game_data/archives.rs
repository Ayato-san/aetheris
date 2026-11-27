//! Bounded archive readers. Paths are virtual: extraction never writes into
//! the game installation, nor uses untrusted archive paths as host paths.
use super::{
    binary::{Reader, invalid},
    reader::{Control, Phase},
};
use aes::{
    Aes256, Block,
    cipher::{BlockDecrypt, KeyInit},
};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

const MAX_INDEX: usize = 64 * 1024 * 1024;
const MAX_ASSET: u64 = 256 * 1024 * 1024;
pub mod iostore;

#[derive(Clone, Debug, Serialize)]
pub struct Entry {
    pub path: String,
    pub size: u64,
    pub compressed_size: u64,
    pub codec: String,
    pub encrypted: bool,
    offset: u64,
    header_size: u64,
    block_size: u64,
    blocks: Vec<(u64, u64)>,
}

pub struct Pak {
    path: PathBuf,
    key: Option<[u8; 32]>,
    pub entries: BTreeMap<String, Entry>,
    pub deleted: BTreeSet<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Extracted {
    pub path: String,
    pub container: String,
    pub size: u64,
    pub sha256: String,
}
#[derive(Serialize, Deserialize)]
pub struct Extraction {
    #[serde(default)]
    pub containers: Vec<Container>,
    pub files: Vec<Extracted>,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Container {
    pub path: String,
    pub format: String,
    pub named_files: usize,
    pub toc_sha256: Option<String>,
    pub signatures_verified: bool,
}
fn archive_paths(root: &Path, output: &mut Vec<PathBuf>, depth: usize) -> io::Result<()> {
    if depth > 64 || output.len() > 100_000 {
        return Err(invalid("archive search depth or count limit exceeded"));
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(invalid(
                "symbolic links are not supported in the archive tree",
            ));
        }
        if kind.is_dir() {
            archive_paths(&entry.path(), output, depth + 1)?;
        } else if kind.is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case("pak") || s.eq_ignore_ascii_case("utoc"))
        {
            output.push(entry.path());
        }
    }
    Ok(())
}
enum Mounted {
    Pak(Pak),
    IoStore(iostore::IoStore),
}
enum Location {
    Pak(usize, Entry),
    IoStore(usize, usize, String),
}
/// Deterministic base/patch precedence across PAK and IoStore. Only requested
/// DAT/configuration payloads are read, retaining case for name-derived keys.
pub fn extract_data(root: &Path, output: &Path, key: [u8; 32]) -> io::Result<Extraction> {
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let mut progress = |_| {};
    extract_data_with_control(root, output, key, &mut Control::new(&cancel, &mut progress))
}
pub fn extract_data_with_control(
    root: &Path,
    output: &Path,
    key: [u8; 32],
    control: &mut Control<'_>,
) -> io::Result<Extraction> {
    control.check()?;
    let root = root.canonicalize()?;
    let output = output_directory(&root, output)?;
    crate::engine_ini::storage::atomic_replace(
        &output.join("extraction.json"),
        br#"{"files":[],"errors":["extraction incomplete"]}"#,
        None,
    )?;
    let mut paths = Vec::new();
    archive_paths(&root, &mut paths, 0)?;
    if paths.is_empty() {
        return Err(invalid("no PAK or IoStore archives found"));
    }
    paths.sort_by_key(|p| {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let priority = stem
            .strip_suffix("_P")
            .and_then(|s| s.rsplit('_').next())
            .and_then(|s| s.parse::<u64>().ok())
            .map_or(0, |n| n.saturating_add(1));
        (priority, p.clone())
    });
    let mut selected = BTreeMap::new();
    let mut mounted = Vec::new();
    let mut report = Extraction {
        containers: Vec::new(),
        files: Vec::new(),
        errors: Vec::new(),
    };
    let wanted = |name: &str| {
        let lower = name.to_lowercase();
        lower.ends_with(".dat") || lower.ends_with("/defaultbuildconfig.ini")
    };
    for (n, path) in paths.iter().enumerate() {
        let relative = path
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        control.emit(Phase::Mount, n, paths.len(), Some(&relative))?;
        let index = mounted.len();
        let result = (|| -> io::Result<Mounted> {
            if path
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case("pak"))
            {
                let archive = Pak::open(path, Some(key))?;
                for name in &archive.deleted {
                    selected.remove(name);
                }
                for (name, entry) in &archive.entries {
                    if wanted(name) {
                        selected.insert(name.clone(), Location::Pak(index, entry.clone()));
                    }
                }
                report.containers.push(Container {
                    path: relative.clone(),
                    format: "pak_v11".into(),
                    named_files: archive.entries.len(),
                    toc_sha256: None,
                    signatures_verified: false,
                });
                Ok(Mounted::Pak(archive))
            } else {
                let archive = iostore::IoStore::open(path, Some(key))?;
                for (name, chunk) in &archive.files {
                    if wanted(name) {
                        selected.insert(
                            name.to_lowercase(),
                            Location::IoStore(index, *chunk, name.clone()),
                        );
                    }
                }
                report.containers.push(Container {
                    path: relative.clone(),
                    format: "iostore_v5".into(),
                    named_files: archive.files.len(),
                    toc_sha256: Some(archive.summary().toc_sha256),
                    signatures_verified: false,
                });
                Ok(Mounted::IoStore(archive))
            }
        })();
        match result {
            Ok(archive) => mounted.push(archive),
            Err(error) => {
                report.errors.push(format!("{relative}: {error}"));
                crate::engine_ini::storage::atomic_replace(
                    &output.join("extraction.json"),
                    &serde_json::to_vec_pretty(&report)?,
                    None,
                )?;
                return Err(invalid(format!("{relative}: {error}")));
            }
        }
    }
    control.emit(Phase::Mount, paths.len(), paths.len(), None)?;
    let total = selected.len();
    for (n, (_, location)) in selected.into_iter().enumerate() {
        let (index, path) = match &location {
            Location::Pak(i, e) => (*i, e.path.as_str()),
            Location::IoStore(i, _, p) => (*i, p.as_str()),
        };
        control.emit(Phase::Extract, n, total, Some(path))?;
        let result = (|| -> io::Result<Extracted> {
            let bytes = match (&mounted[index], &location) {
                (Mounted::Pak(p), Location::Pak(_, e)) => p.read(e)?,
                (Mounted::IoStore(s), Location::IoStore(_, chunk, _)) => s.read_chunk(*chunk)?,
                _ => return Err(invalid("archive entry backend mismatch")),
            };
            control.check()?;
            let destination = destination(&output, Path::new(path))?;
            crate::engine_ini::storage::atomic_replace(&destination, &bytes, None)?;
            Ok(Extracted {
                path: path.to_owned(),
                container: report.containers[index].path.clone(),
                size: bytes.len() as u64,
                sha256: hex::encode(sha2::Sha256::digest(&bytes)),
            })
        })();
        match result {
            Ok(file) => report.files.push(file),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => return Err(error),
            Err(error) => report.errors.push(format!("{path}: {error}")),
        }
    }
    control.emit(Phase::Extract, total, total, None)?;
    crate::engine_ini::storage::atomic_replace(
        &output.join("extraction.json"),
        &serde_json::to_vec_pretty(&report)?,
        None,
    )?;
    Ok(report)
}

/// Resolve the output before writing, including a not-yet-created directory.
/// Parent traversals are refused so missing intermediate folders cannot hide
/// a destination inside the source tree.
pub(super) fn output_directory(source: &Path, output: &Path) -> io::Result<PathBuf> {
    if output
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(invalid("output path must not contain parent traversal"));
    }
    let mut ancestor = output;
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
    }
    let resolved = ancestor.canonicalize()?;
    if resolved.starts_with(source) || (output.exists() && source.starts_with(&resolved)) {
        return Err(invalid("source and output directories must not overlap"));
    }
    fs::create_dir_all(output)?;
    let output = output.canonicalize()?;
    if output.starts_with(source) || source.starts_with(&output) {
        return Err(invalid("source and output directories must not overlap"));
    }
    Ok(output)
}

/// Resolve each output directory before creating descendants, rejecting links
/// that would redirect an extraction outside its dedicated output tree.
pub(super) fn destination(root: &Path, relative: &Path) -> io::Result<PathBuf> {
    let mut parent = root.to_path_buf();
    let mut parts = relative.components().peekable();
    while let Some(part) = parts.next() {
        let std::path::Component::Normal(name) = part else {
            return Err(invalid("unsafe extraction destination"));
        };
        parent.push(name);
        if parts.peek().is_some() {
            match fs::create_dir(&parent) {
                Ok(()) => (),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
                Err(e) => return Err(e),
            }
            parent = parent.canonicalize()?;
            if !parent.starts_with(root) {
                return Err(invalid("extraction destination resolves outside output"));
            }
        }
    }
    Ok(parent)
}

pub fn read_key(path: &Path) -> io::Result<[u8; 32]> {
    let text = fs::read_to_string(path)?;
    let lines: Vec<_> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("//"))
        .collect();
    if lines.len() != 1 {
        return Err(invalid("archive key file must contain one hexadecimal key"));
    }
    let text = lines[0].strip_prefix("0x").unwrap_or(lines[0]);
    hex::decode(text)
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or_else(|| invalid("archive key must contain exactly 32 hexadecimal bytes"))
}

pub(super) fn decrypt(bytes: &mut [u8], key: Option<&[u8; 32]>) -> io::Result<()> {
    let key = key.ok_or_else(|| invalid("encrypted archive requires a local AES key"))?;
    if !bytes.len().is_multiple_of(16) {
        return Err(invalid("archive AES block alignment mismatch"));
    }
    let cipher = Aes256::new_from_slice(key).unwrap();
    for chunk in bytes.as_chunks_mut::<16>().0 {
        let mut block = Block::from(*chunk);
        cipher.decrypt_block(&mut block);
        chunk.copy_from_slice(&block);
    }
    Ok(())
}

fn range(file: &mut File, offset: u64, size: u64, limit: usize) -> io::Result<Vec<u8>> {
    let file_len = file.metadata()?.len();
    if size > limit as u64 || offset.checked_add(size).is_none_or(|end| end > file_len) {
        return Err(invalid("archive range exceeds file or allocation limit"));
    }
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = vec![0; size as usize];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn index(
    file: &mut File,
    offset: u64,
    size: u64,
    hash: &[u8],
    encrypted: bool,
    key: Option<&[u8; 32]>,
) -> io::Result<Vec<u8>> {
    let mut bytes = range(file, offset, size, MAX_INDEX)?;
    if encrypted {
        decrypt(&mut bytes, key)?;
    }
    if Sha1::digest(&bytes).as_slice() != hash {
        return Err(invalid("archive index SHA-1 mismatch"));
    }
    Ok(bytes)
}

fn virtual_path(mount: &str, directory: &str, name: &str) -> io::Result<String> {
    let mut mount = mount;
    while let Some(rest) = mount.strip_prefix("../") {
        mount = rest;
    }
    let path = format!("{mount}/{directory}/{name}");
    let mut parts = Vec::new();
    for part in path.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.contains(['\\', ':', '\0']) {
            return Err(invalid("unsafe virtual archive path"));
        }
        parts.push(part);
    }
    if parts.is_empty() {
        return Err(invalid("empty virtual archive path"));
    }
    Ok(parts.join("/"))
}

impl Pak {
    /// The supplied client uses standard v11 trailers and path-hash indices.
    /// Other versions fail explicitly rather than trying a nearby layout.
    pub fn open(path: &Path, key: Option<[u8; 32]>) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let length = file.metadata()?.len();
        if length < 221 {
            return Err(invalid("truncated PAK trailer"));
        }
        let trailer = range(&mut file, length - 221, 221, 221)?;
        let mut t = Reader::new(&trailer);
        t.take(16)?;
        let encrypted = match t.u8()? {
            0 => false,
            1 => true,
            _ => return Err(invalid("invalid PAK encryption flag")),
        };
        if t.u32()? != 0x5a6f12e1 {
            return Err(invalid("unsupported PAK magic"));
        }
        let version = t.u32()?;
        if version != 11 {
            return Err(invalid(format!("unsupported PAK version {version}")));
        }
        let offset = t.u64()?;
        let size = t.u64()?;
        let hash = t.take(20)?;
        let mut codecs = vec!["None".to_owned()];
        for _ in 0..5 {
            let name = t.take(32)?;
            let end = name.iter().position(|b| *b == 0).unwrap_or(32);
            codecs.push(
                std::str::from_utf8(&name[..end])
                    .map_err(|_| invalid("invalid compression method"))?
                    .to_owned(),
            );
        }
        let bytes = index(&mut file, offset, size, hash, encrypted, key.as_ref())?;
        let mut r = Reader::new(&bytes);
        let mount = r.string()?;
        let count = r.count(2_000_000)?;
        r.take(8)?;
        if r.flag()? {
            r.take(36)?;
        }
        if !r.flag()? {
            return Err(invalid("PAK full directory index unavailable"));
        }
        let dir_offset = r.u64()?;
        let dir_size = r.u64()?;
        let dir_hash = r.take(20)?;
        let encoded_size = r.count(MAX_INDEX)?;
        let encoded = r.take(encoded_size)?;
        let nonencoded = r.count(2_000_000)?;
        if nonencoded != 0 {
            return Err(invalid(
                "non-encoded PAK entries are not supported by this reader",
            ));
        }
        let directory = index(
            &mut file,
            dir_offset,
            dir_size,
            dir_hash,
            encrypted,
            key.as_ref(),
        )?;
        let mut d = Reader::new(&directory);
        let mut entries = BTreeMap::new();
        let mut deleted = BTreeSet::new();
        for _ in 0..d.count(2_000_000)? {
            let folder = d.string()?;
            for _ in 0..d.count(2_000_000)? {
                let name = d.string()?;
                let entry_offset = d.i32()?;
                let path = virtual_path(&mount, &folder, &name)?;
                if entry_offset == i32::MIN {
                    deleted.insert(path.to_lowercase());
                    continue;
                }
                let offset = usize::try_from(entry_offset)
                    .map_err(|_| invalid("unsupported non-encoded entry reference"))?;
                let entry = decode_entry(&encoded[offset.min(encoded.len())..], path, &codecs)?;
                if entry
                    .offset
                    .checked_add(entry.header_size)
                    .is_none_or(|end| end > length - 221)
                {
                    return Err(invalid("PAK entry offset exceeds container"));
                }
                if entries.insert(entry.path.to_lowercase(), entry).is_some() {
                    return Err(invalid("duplicate PAK virtual path"));
                }
            }
        }
        if entries.len() > count {
            return Err(invalid("PAK directory count mismatch"));
        }
        Ok(Self {
            path: path.into(),
            key,
            entries,
            deleted,
        })
    }

    pub fn read(&self, entry: &Entry) -> io::Result<Vec<u8>> {
        if entry.size > MAX_ASSET {
            return Err(invalid("asset exceeds supported allocation limit"));
        }
        let mut file = File::open(&self.path)?;
        if entry.codec == "None" {
            let length = if entry.encrypted {
                entry.size.next_multiple_of(16)
            } else {
                entry.size
            };
            let mut bytes = range(
                &mut file,
                entry.offset + entry.header_size,
                length,
                MAX_ASSET as usize + 16,
            )?;
            if entry.encrypted {
                decrypt(&mut bytes, self.key.as_ref())?;
            }
            bytes.truncate(entry.size as usize);
            return Ok(bytes);
        }
        if entry.block_size == 0 {
            return Err(invalid("compressed entry has no block size"));
        }
        let mut output = Vec::with_capacity(entry.size as usize);
        for &(start, end) in &entry.blocks {
            let packed = end
                .checked_sub(start)
                .ok_or_else(|| invalid("negative compressed block size"))?;
            let length = if entry.encrypted {
                packed.next_multiple_of(16)
            } else {
                packed
            };
            let mut bytes = range(&mut file, start, length, MAX_ASSET as usize)?;
            if entry.encrypted {
                decrypt(&mut bytes, self.key.as_ref())?;
            }
            bytes.truncate(packed as usize);
            let raw = entry.block_size.min(
                entry
                    .size
                    .checked_sub(output.len() as u64)
                    .ok_or_else(|| invalid("compressed output exceeds entry"))?,
            ) as usize;
            output.extend(decompress(&entry.codec, &bytes, raw)?);
        }
        if output.len() as u64 != entry.size {
            return Err(invalid("asset decompressed length mismatch"));
        }
        Ok(output)
    }
}

fn decode_entry(bytes: &[u8], path: String, codecs: &[String]) -> io::Result<Entry> {
    let mut r = Reader::new(bytes);
    let bits = r.u32()?;
    let block_size = if bits & 63 == 63 {
        r.u32()? as u64
    } else {
        ((bits & 63) << 11) as u64
    };
    let codec = codecs
        .get(((bits >> 23) & 63) as usize)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid("unknown PAK codec index"))?
        .clone();
    let offset = if bits >> 31 & 1 != 0 {
        r.u32()? as u64
    } else {
        r.u64()?
    };
    let size = if bits >> 30 & 1 != 0 {
        r.u32()? as u64
    } else {
        r.u64()?
    };
    let compressed_size = if codec == "None" {
        size
    } else if bits >> 29 & 1 != 0 {
        r.u32()? as u64
    } else {
        r.u64()?
    };
    let encrypted = bits >> 22 & 1 != 0;
    let count = ((bits >> 6) & 65535) as usize;
    let header_size = 53
        + if codec == "None" {
            0
        } else {
            4 + count as u64 * 16
        };
    let mut cursor = offset
        .checked_add(header_size)
        .ok_or_else(|| invalid("entry offset overflow"))?;
    let mut blocks = Vec::new();
    for _ in 0..count {
        let length = if count == 1 && !encrypted {
            compressed_size
        } else {
            r.u32()? as u64
        };
        let end = cursor
            .checked_add(length)
            .ok_or_else(|| invalid("compressed block overflow"))?;
        blocks.push((cursor, end));
        cursor = cursor
            .checked_add(if encrypted {
                length.next_multiple_of(16)
            } else {
                length
            })
            .ok_or_else(|| invalid("compressed block offset overflow"))?;
    }
    Ok(Entry {
        path,
        size,
        compressed_size,
        codec,
        encrypted,
        offset,
        header_size,
        block_size: if count == 1 { size } else { block_size },
        blocks,
    })
}

fn decompress(codec: &str, bytes: &[u8], raw: usize) -> io::Result<Vec<u8>> {
    if raw > MAX_ASSET as usize {
        return Err(invalid("decompression exceeds allocation limit"));
    }
    let mut output = vec![0; raw];
    match codec {
        "None" => {
            if bytes.len() != raw {
                return Err(invalid("uncompressed archive block size mismatch"));
            }
            output.copy_from_slice(bytes);
        }
        "Oodle" => {
            let written = oozextract::Extractor::new()
                .read_from_slice(bytes, &mut output)
                .map_err(|e| invalid(format!("Oodle decompression failed: {e}")))?;
            if written != raw {
                return Err(invalid("Oodle decompressed size mismatch"));
            }
        }
        _ => return Err(invalid(format!("unsupported archive codec {codec}"))),
    }
    Ok(output)
}
