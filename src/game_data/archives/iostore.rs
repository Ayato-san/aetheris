//! Version-5 IoStore TOC, directory and UCAS chunk reads with bounded allocation.
use super::{MAX_ASSET, MAX_INDEX, Reader, decompress, decrypt, invalid, range, virtual_path};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io,
    path::{Path, PathBuf},
};

const MAX_CHUNK: u64 = 64 * 1024 * 1024;

#[derive(Serialize)]
pub struct Summary {
    pub toc_sha256: String,
    pub version: u8,
    pub chunks: usize,
    pub partitions: usize,
    pub files: usize,
    pub data_files: usize,
    pub codecs: Vec<String>,
    pub encrypted: bool,
    pub signatures_verified: bool,
    pub package_dependencies_verified: bool,
}
pub struct IoStore {
    path: PathBuf,
    toc_sha256: String,
    key: Option<[u8; 32]>,
    encrypted: bool,
    block_size: u64,
    partition_size: u64,
    partitions: usize,
    chunks: Vec<(u64, u64)>,
    chunk_ids: Vec<[u8; 12]>,
    blocks: Vec<(u64, usize, usize, usize)>,
    codecs: Vec<String>,
    pub files: BTreeMap<String, usize>,
}
impl IoStore {
    pub fn open(path: &Path, key: Option<[u8; 32]>) -> io::Result<Self> {
        Self::open_with_hash(path, key, None)
    }

    /// An operator-supplied TOC baseline is an integrity check, not signature
    /// authentication. No package dependency validation is performed here.
    pub fn open_with_hash(
        path: &Path,
        key: Option<[u8; 32]>,
        expected_sha256: Option<&str>,
    ) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let size = file.metadata()?.len();
        let bytes = range(&mut file, 0, size, MAX_INDEX)?;
        let toc_sha256 = hex::encode(Sha256::digest(&bytes));
        if expected_sha256.is_some_and(|expected| !expected.eq_ignore_ascii_case(&toc_sha256)) {
            return Err(invalid("IoStore TOC SHA-256 mismatch"));
        }
        let mut r = Reader::new(&bytes);
        if r.take(16)? != b"-==--==--==--==-" {
            return Err(invalid("invalid IoStore magic"));
        }
        let version = r.u8()?;
        if version != 5 {
            return Err(invalid(format!("unsupported IoStore version {version}")));
        }
        r.u8()?;
        r.u16()?;
        if r.u32()? != 144 {
            return Err(invalid("unsupported IoStore header size"));
        }
        let count = r.count(2_000_000)?;
        let block_count = r.count(2_000_000)?;
        if r.u32()? != 12 {
            return Err(invalid("unsupported IoStore block entry size"));
        }
        let method_count = r.count(32)?;
        let method_len = r.count(256)?;
        let block_size = r.u32()? as u64;
        let dir_size = r.count(MAX_INDEX)?;
        let partitions = r.count(1024)?;
        r.take(24)?;
        let flags = r.u32()?;
        let seed_count = r.count(2_000_000)?;
        let partition_size = r.u64()?;
        let overflow_count = r.count(2_000_000)?;
        r.take(44)?;
        if block_size == 0 || partitions == 0 || partition_size == 0 || flags & !15 != 0 {
            return Err(invalid("unsupported IoStore flags or partition layout"));
        }
        let mut ids = BTreeSet::new();
        let mut chunk_ids = Vec::with_capacity(count);
        for _ in 0..count {
            let id: [u8; 12] = r.take(12)?.try_into().unwrap();
            if !ids.insert(id) {
                return Err(invalid("duplicate IoStore chunk ID"));
            }
            chunk_ids.push(id);
        }
        let mut chunks = Vec::with_capacity(count);
        for _ in 0..count {
            let data = r.take(10)?;
            let be5 = |b: &[u8]| b.iter().fold(0u64, |a, b| (a << 8) | *b as u64);
            chunks.push((be5(&data[..5]), be5(&data[5..])));
        }
        r.take((seed_count + overflow_count) * 4)?;
        let mut blocks = Vec::with_capacity(block_count);
        for _ in 0..block_count {
            let a = r.u64()?;
            let b = r.u32()?;
            blocks.push((
                a & 0xffffffffff,
                (a >> 40) as usize,
                (b & 0xffffff) as usize,
                (b >> 24) as usize,
            ));
        }
        let mut codecs = vec!["None".into()];
        for _ in 0..method_count {
            let bytes = r.take(method_len)?;
            let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
            codecs.push(
                std::str::from_utf8(&bytes[..end])
                    .map_err(|_| invalid("invalid IoStore codec name"))?
                    .to_owned(),
            );
        }
        if flags & 4 != 0 {
            let size = r.count(1024 * 1024)?;
            r.take(size * 2 + block_count * 20)?;
        }
        let encrypted = flags & 2 != 0;
        let mut files = BTreeMap::new();
        if flags & 8 != 0 && dir_size > 0 {
            let mut directory = r.take(dir_size)?.to_vec();
            if encrypted {
                decrypt(&mut directory, key.as_ref())?;
            }
            files = directory_files(&directory, count)?;
        }
        let mut partition_lengths = Vec::with_capacity(partitions);
        for partition in 0..partitions {
            let name = partition_path(path, partition)?;
            if !name.is_file() {
                return Err(invalid(format!(
                    "missing IoStore partition {}",
                    name.display()
                )));
            }
            partition_lengths.push(std::fs::metadata(name)?.len());
        }
        for &(position, packed, raw, codec) in &blocks {
            if packed == 0
                || raw == 0
                || raw as u64 > block_size
                || codec >= codecs.len()
                || (codec == 0 && packed != raw)
            {
                return Err(invalid("invalid IoStore block sizes or codec index"));
            }
            let partition = (position / partition_size) as usize;
            let local = position % partition_size;
            let stored = if encrypted {
                packed.next_multiple_of(16)
            } else {
                packed
            } as u64;
            let end = local
                .checked_add(stored)
                .ok_or_else(|| invalid("IoStore block range overflow"))?;
            if end > partition_size
                || partition_lengths
                    .get(partition)
                    .is_none_or(|len| end > *len)
            {
                return Err(invalid("IoStore block exceeds partition bounds"));
            }
        }
        for &(offset, length) in &chunks {
            if length == 0 {
                continue;
            }
            let end = offset
                .checked_add(length - 1)
                .ok_or_else(|| invalid("IoStore chunk range overflow"))?;
            for cursor in [offset, end] {
                if blocks
                    .get((cursor / block_size) as usize)
                    .is_none_or(|block| cursor % block_size >= block.2 as u64)
                {
                    return Err(invalid("IoStore chunk references missing block or padding"));
                }
            }
        }
        Ok(Self {
            path: path.into(),
            toc_sha256,
            key,
            encrypted,
            block_size,
            partition_size,
            partitions,
            chunks,
            chunk_ids,
            blocks,
            codecs,
            files,
        })
    }
    pub fn summary(&self) -> Summary {
        Summary {
            toc_sha256: self.toc_sha256.clone(),
            version: 5,
            chunks: self.chunks.len(),
            partitions: self.partitions,
            files: self.files.len(),
            data_files: self.files.keys().filter(|s| s.ends_with(".dat")).count(),
            codecs: self.codecs.clone(),
            encrypted: self.encrypted,
            signatures_verified: false,
            package_dependencies_verified: false,
        }
    }
    /// Exact cooked identity, including package, bulk index and chunk type.
    pub fn chunk_id(&self, index: usize) -> Option<[u8; 12]> {
        self.chunk_ids.get(index).copied()
    }

    pub fn read_chunk(&self, index: usize) -> io::Result<Vec<u8>> {
        let &(offset, length) = self
            .chunks
            .get(index)
            .ok_or_else(|| invalid("invalid IoStore chunk index"))?;
        if length > MAX_CHUNK {
            return Err(invalid("IoStore chunk exceeds allocation limit"));
        }
        let end = offset
            .checked_add(length)
            .ok_or_else(|| invalid("IoStore chunk overflow"))?;
        let mut cursor = offset;
        let mut output = Vec::with_capacity(length as usize);
        let mut files = BTreeMap::new();
        while cursor < end {
            let block_index = (cursor / self.block_size) as usize;
            let &(position, packed, raw, codec) = self
                .blocks
                .get(block_index)
                .ok_or_else(|| invalid("missing IoStore compression block"))?;
            if raw as u64 > self.block_size || raw == 0 {
                return Err(invalid("invalid IoStore raw block size"));
            }
            let partition = (position / self.partition_size) as usize;
            if partition >= self.partitions {
                return Err(invalid("IoStore block partition exceeds container"));
            }
            // A texture spans many compression blocks. Reopening its .ucas for
            // each block is particularly expensive on Windows and network disks.
            let file = match files.entry(partition) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(File::open(partition_path(&self.path, partition)?)?)
                }
                std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            };
            let length = if self.encrypted {
                packed.next_multiple_of(16)
            } else {
                packed
            };
            let mut bytes = range(
                file,
                position % self.partition_size,
                length as u64,
                MAX_ASSET as usize,
            )?;
            if self.encrypted {
                decrypt(&mut bytes, self.key.as_ref())?;
            }
            bytes.truncate(packed);
            let name = self
                .codecs
                .get(codec)
                .ok_or_else(|| invalid("invalid IoStore codec index"))?;
            let bytes = decompress(name, &bytes, raw)?;
            let start = (cursor % self.block_size) as usize;
            let take = (end - cursor).min(
                (raw.checked_sub(start)
                    .filter(|n| *n > 0)
                    .ok_or_else(|| invalid("chunk offset exceeds raw block"))?)
                    as u64,
            ) as usize;
            output.extend_from_slice(&bytes[start..start + take]);
            cursor += take as u64;
        }
        Ok(output)
    }
}
fn partition_path(path: &Path, partition: usize) -> io::Result<PathBuf> {
    if partition == 0 {
        return Ok(path.with_extension("ucas"));
    }
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| invalid("non-Unicode partition path"))?;
    Ok(path.with_file_name(format!("{stem}_s{partition}.ucas")))
}
fn directory_files(bytes: &[u8], chunks: usize) -> io::Result<BTreeMap<String, usize>> {
    let mut r = Reader::new(bytes);
    let mount = r.string()?;
    let mut dirs = Vec::new();
    for _ in 0..r.count(2_000_000)? {
        dirs.push([r.u32()?, r.u32()?, r.u32()?, r.u32()?]);
    }
    let mut files = Vec::new();
    for _ in 0..r.count(2_000_000)? {
        files.push([r.u32()?, r.u32()?, r.u32()?]);
    }
    let mut strings = Vec::new();
    for _ in 0..r.count(2_000_000)? {
        strings.push(r.string()?);
    }
    let mut output = BTreeMap::new();
    let mut queue = vec![(0u32, String::new(), 0)];
    let mut visited_dirs = BTreeSet::new();
    let mut visited_files = BTreeSet::new();
    let string = |index: u32| {
        strings
            .get(index as usize)
            .ok_or_else(|| invalid("invalid IoStore string index"))
    };
    while let Some((id, parent, depth)) = queue.pop() {
        if depth > 64 || !visited_dirs.insert(id) {
            return Err(invalid("IoStore directory cycle or depth limit"));
        }
        let &[name, child, sibling, first_file] = dirs
            .get(id as usize)
            .ok_or_else(|| invalid("invalid directory index"))?;
        let folder = if name == u32::MAX {
            parent.clone()
        } else {
            format!("{parent}/{}/", string(name)?)
        };
        if sibling != u32::MAX {
            queue.push((sibling, parent, depth));
        }
        if child != u32::MAX {
            queue.push((child, folder.clone(), depth + 1));
        }
        let mut index = first_file;
        while index != u32::MAX {
            if !visited_files.insert(index) {
                return Err(invalid("IoStore file list cycle"));
            }
            let &[name, next, chunk] = files
                .get(index as usize)
                .ok_or_else(|| invalid("invalid file index"))?;
            if chunk as usize >= chunks {
                return Err(invalid("IoStore file chunk reference exceeds TOC"));
            }
            let path = virtual_path(&mount, &folder, string(name)?)?.to_lowercase();
            if output.insert(path, chunk as usize).is_some() {
                return Err(invalid("duplicate IoStore virtual file"));
            }
            index = next;
        }
    }
    Ok(output)
}
