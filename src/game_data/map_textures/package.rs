//! Bounded UE 5.3 Zen headers. Unsupported versions fail instead of scanning for
//! plausible texture dimensions or guessing binary offsets.
use crate::game_data::binary::{Reader, invalid};
use std::{collections::BTreeMap, io};

pub(super) fn names(r: &mut Reader<'_>) -> io::Result<Vec<String>> {
    let count = r.count(100_000)?;
    if count == 0 {
        return Ok(Vec::new());
    }
    let bytes = r.count(16 * 1024 * 1024)?;
    r.u64()?; // name hash algorithm version
    r.take(count * 8)?;
    let headers = r.take(count * 2)?.to_vec();
    let start = r.pos;
    let mut result = Vec::with_capacity(count);
    for h in headers.as_chunks::<2>().0 {
        let h = u16::from_be_bytes(*h);
        let n = usize::from(h & 0x7fff);
        let value = if h & 0x8000 != 0 {
            let words = r
                .take(n * 2)?
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u16::from_le_bytes(*b))
                .collect::<Vec<_>>();
            String::from_utf16(&words).map_err(|_| invalid("invalid cooked UTF-16 name"))?
        } else {
            std::str::from_utf8(r.take(n)?)
                .map_err(|_| invalid("invalid cooked UTF-8 name"))?
                .to_owned()
        };
        if value.contains('\0') {
            return Err(invalid("NUL in cooked name"));
        }
        result.push(value);
    }
    if r.pos - start != bytes {
        return Err(invalid("cooked name byte count mismatch"));
    }
    Ok(result)
}
pub(super) fn mapped(names: &[String], r: &mut Reader<'_>) -> io::Result<String> {
    let index = r.u32()?;
    let number = r.u32()?;
    if index >> 30 != 0 {
        return Err(invalid("unsupported mapped name scope"));
    }
    let base = names
        .get(index as usize)
        .ok_or_else(|| invalid("cooked name index out of bounds"))?;
    Ok(if number == 0 {
        base.clone()
    } else {
        format!("{base}_{}", number - 1)
    })
}
#[derive(Clone)]
pub(super) struct Export {
    pub name: String,
    pub class: u64,
    pub hash: u64,
    pub flags: u32,
    pub offset: usize,
    pub size: usize,
}
pub(super) struct Bulk {
    pub offset: u64,
    pub size: usize,
    pub flags: u32,
    pub index: u8,
}
pub(super) struct Package {
    pub bytes: Vec<u8>,
    pub name: String,
    pub names: Vec<String>,
    pub exports: Vec<Export>,
    pub imports: Vec<u64>,
    pub hashes: Vec<u64>,
    pub bulk: Vec<Bulk>,
}
impl Package {
    pub fn parse(bytes: Vec<u8>) -> io::Result<Self> {
        let mut r = Reader::new(&bytes);
        if r.u32()? != 0 {
            return Err(invalid("versioned Zen headers are not supported"));
        }
        let header = r.count(64 * 1024 * 1024)?;
        let name_index = r.u32()?;
        let name_number = r.u32()?;
        let flags = r.u32()?;
        if flags & 0x2000 == 0 {
            return Err(invalid("expected unversioned cooked properties"));
        }
        r.u32()?; // original cooked header size
        let hashes_start = r.count(header)?;
        let imports_start = r.count(header)?;
        let exports_start = r.count(header)?;
        let bundles_start = r.count(header)?;
        let dependencies_start = r.count(header)?;
        let dependencies_end = r.count(header)?;
        let imported_names = r.count(header)?;
        let names = names(&mut r)?;
        let mut nm = Vec::from(name_index.to_le_bytes());
        nm.extend(name_number.to_le_bytes());
        let name = mapped(&names, &mut Reader::new(&nm))?;
        if !name.starts_with("/Game/") {
            return Err(invalid("unexpected cooked package root"));
        }
        if !(r.pos <= hashes_start
            && hashes_start <= imports_start
            && imports_start <= exports_start
            && exports_start <= bundles_start
            && bundles_start <= dependencies_start
            && dependencies_start <= dependencies_end
            && dependencies_end <= imported_names
            && imported_names <= header
            && header <= bytes.len())
        {
            return Err(invalid("invalid Zen section order"));
        }
        // Aion2 uses the UE5.3 data resource map immediately after names.
        let bulk_size = r.u64()? as usize;
        if !bulk_size.is_multiple_of(32) || bulk_size / 32 > 16384 {
            return Err(invalid("invalid bulk data map size"));
        }
        let mut bulk = Vec::new();
        for _ in 0..bulk_size / 32 {
            let offset = r.u64()?;
            r.u64()?;
            let size = usize::try_from(r.u64()?).map_err(|_| invalid("bulk size overflow"))?;
            let flags = r.u32()?;
            let index = r.u8()?;
            r.take(3)?;
            if size > 64 * 1024 * 1024 {
                return Err(invalid("bulk allocation exceeds limit"));
            }
            bulk.push(Bulk {
                offset,
                size,
                flags,
                index,
            });
        }
        if r.pos != hashes_start
            || !(imports_start - hashes_start).is_multiple_of(8)
            || !(exports_start - imports_start).is_multiple_of(8)
            || !(bundles_start - exports_start).is_multiple_of(72)
        {
            return Err(invalid("invalid Zen section size"));
        }
        let hashes = (0..(imports_start - hashes_start) / 8)
            .map(|_| r.u64())
            .collect::<io::Result<Vec<_>>>()?;
        let imports = (0..(exports_start - imports_start) / 8)
            .map(|_| r.u64())
            .collect::<io::Result<Vec<_>>>()?;
        let mut exports = Vec::new();
        for _ in 0..(bundles_start - exports_start) / 72 {
            let start = r.pos;
            let offset = usize::try_from(r.u64()?)
                .ok()
                .and_then(|n| header.checked_add(n))
                .ok_or_else(|| invalid("export offset overflow"))?;
            let size = usize::try_from(r.u64()?).map_err(|_| invalid("export size overflow"))?;
            let name = mapped(&names, &mut r)?;
            r.u64()?;
            let class = r.u64()?;
            r.take(16)?;
            let hash = r.u64()?;
            let flags = r.u32()?;
            r.take(4)?;
            if offset.checked_add(size).is_none_or(|end| end > bytes.len()) || r.pos - start != 72 {
                return Err(invalid("export range exceeds package"));
            }
            exports.push(Export {
                name,
                class,
                hash,
                flags,
                offset,
                size,
            });
        }
        Ok(Self {
            bytes,
            name,
            names,
            exports,
            imports,
            hashes,
            bulk,
        })
    }
    pub fn body(&self, e: &Export) -> Reader<'_> {
        Reader::new(&self.bytes[e.offset..e.offset + e.size])
    }
    pub fn reference(&self, index: i32) -> io::Result<u64> {
        if index > 0 {
            let i = (index - 1) as usize;
            if i >= self.exports.len() {
                return Err(invalid("export reference exceeds package"));
            }
            Ok(i as u64)
        } else if index < 0 {
            self.imports
                .get(index.unsigned_abs() as usize - 1)
                .copied()
                .ok_or_else(|| invalid("import reference exceeds package"))
        } else {
            Ok(u64::MAX)
        }
    }
}
pub(super) fn script_objects(bytes: &[u8]) -> io::Result<BTreeMap<u64, String>> {
    let mut r = Reader::new(bytes);
    let names = names(&mut r)?;
    let mut result = BTreeMap::new();
    for _ in 0..r.count(100_000)? {
        let index = r.u32()?;
        let number = r.u32()?;
        let id = r.u64()?;
        r.take(16)?;
        let name = names
            .get((index & 0x3fffffff) as usize)
            .ok_or_else(|| invalid("invalid script object name"))?;
        let name = if number == 0 {
            name.clone()
        } else {
            format!("{name}_{}", number - 1)
        };
        if result.insert(id, name).is_some() {
            return Err(invalid("duplicate script object"));
        }
    }
    if r.pos != bytes.len() {
        return Err(invalid("trailing script object data"));
    }
    Ok(result)
}
/// Container version 3: relative CArrayView offsets supply import order; paths
/// or alphabetical sorting never substitute for this package dependency list.
pub(super) fn dependencies(bytes: &[u8]) -> io::Result<BTreeMap<u64, Vec<u64>>> {
    let mut r = Reader::new(bytes);
    if r.u32()? != 0x496f436e || r.u32()? != 3 {
        return Err(invalid("unsupported IoStore container header version"));
    }
    r.u64()?;
    let count = r.count(2_000_000)?;
    let ids = (0..count)
        .map(|_| r.u64())
        .collect::<io::Result<Vec<_>>>()?;
    let size = r.count(64 * 1024 * 1024)?;
    let start = r.pos;
    let data = r.take(size)?;
    if count.checked_mul(16).is_none_or(|n| n > data.len()) {
        return Err(invalid("truncated container store entries"));
    }
    let mut result = BTreeMap::new();
    for (i, id) in ids.into_iter().enumerate() {
        let mut entry = Reader::new(&bytes[start + i * 16..start + size]);
        let n = entry.count(100_000)?;
        let offset = entry.count(size)?;
        let pos = i * 16 + offset;
        let end = pos
            .checked_add(n * 8)
            .filter(|e| *e <= data.len())
            .ok_or_else(|| invalid("invalid relative dependency view"))?;
        let mut list = Reader::new(&data[pos..end]);
        result.insert(
            id,
            (0..n).map(|_| list.u64()).collect::<io::Result<Vec<_>>>()?,
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<u8> {
        let strings = ["None", "Tile", "/Game/Test/Tile"];
        let mut batch = 3u32.to_le_bytes().to_vec();
        batch.extend((strings.iter().map(|s| s.len()).sum::<usize>() as u32).to_le_bytes());
        batch.extend([0; 8 + 3 * 8]);
        for s in strings {
            batch.extend((s.len() as u16).to_be_bytes());
        }
        for s in strings {
            batch.extend(s.as_bytes());
        }
        let hashes = 52 + batch.len() + 8;
        let bundles = hashes + 72;
        let header = bundles + 16;
        let mut b = Vec::new();
        for n in [
            0,
            header as u32,
            2,
            0,
            0x2000,
            0,
            hashes as u32,
            hashes as u32,
            hashes as u32,
            bundles as u32,
            header as u32,
            header as u32,
            header as u32,
        ] {
            b.extend(n.to_le_bytes());
        }
        b.extend(batch);
        b.extend(0u64.to_le_bytes());
        b.extend(0u64.to_le_bytes());
        b.extend(4u64.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend(0u32.to_le_bytes());
        b.extend([0; 40]);
        b.extend(0u32.to_le_bytes());
        b.extend([0; 4]);
        b.extend([0; 16]);
        b.extend([1, 2, 3, 4]);
        b
    }
    #[test]
    fn export_ranges_are_bounded_and_truncations_are_rejected() {
        let bytes = fixture();
        let p = Package::parse(bytes.clone()).unwrap();
        assert_eq!(p.name, "/Game/Test/Tile");
        assert_eq!(p.exports[0].name, "Tile");
        assert_eq!(p.body(&p.exports[0]).take(4).unwrap(), [1, 2, 3, 4]);
        assert!(p.reference(2).is_err());
        assert!(p.reference(-1).is_err());
        for end in 0..bytes.len() {
            assert!(Package::parse(bytes[..end].to_vec()).is_err(), "{end}");
        }
        let mut huge = bytes;
        huge[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Package::parse(huge).is_err());
    }
    #[test]
    fn dependencies_use_relative_views_and_reject_out_of_range_views() {
        let mut b = Vec::new();
        for n in [0x496f436eu32, 3, 0, 0, 1] {
            b.extend(n.to_le_bytes());
        }
        b.extend(10u64.to_le_bytes());
        b.extend(32u32.to_le_bytes());
        for n in [2u32, 16, 0, 0] {
            b.extend(n.to_le_bytes());
        }
        b.extend(30u64.to_le_bytes());
        b.extend(20u64.to_le_bytes());
        assert_eq!(dependencies(&b).unwrap()[&10], vec![30, 20]);
        for end in 0..b.len() {
            assert!(dependencies(&b[..end]).is_err());
        }
        b[36..40].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(dependencies(&b).is_err());
    }
    #[test]
    fn name_batches_handle_utf16_and_validate_the_declared_byte_count() {
        let mut b = 1u32.to_le_bytes().to_vec();
        b.extend(2u32.to_le_bytes());
        b.extend([0; 16]);
        b.extend([128, 1, 0xe9, 0]);
        assert_eq!(names(&mut Reader::new(&b)).unwrap(), vec!["é"]);
        b[4..8].copy_from_slice(&3u32.to_le_bytes());
        assert!(names(&mut Reader::new(&b)).is_err());
    }
}
