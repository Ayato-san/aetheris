//! Native, bounded extraction of cooked PC world-map textures and explicit sector
//! bindings. No external runtime or converter is launched on the player's PC.
mod package;
mod pixels;
mod properties;
use super::{
    archives::{iostore::IoStore, read_key},
    binary::invalid,
    usmap::Mapping,
};
use package::{Export, Package};
use properties::{Blueprint, Context};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs, io,
    io::Read,
    path::{Path, PathBuf},
};

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn input(path: &Path) -> io::Result<Vec<u8>> {
    let mut b = Vec::new();
    fs::File::open(path)?
        .take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut b)?;
    if b.len() > 64 * 1024 * 1024 {
        return Err(invalid("mapping exceeds limit"));
    }
    Ok(b)
}
fn toc_paths(root: &Path, paths: &mut Vec<PathBuf>, depth: usize) -> io::Result<()> {
    if depth > 64 || paths.len() > 100_000 {
        return Err(invalid("archive tree exceeds limit"));
    }
    for entry in fs::read_dir(root)? {
        let e = entry?;
        let ty = e.file_type()?;
        if ty.is_dir() {
            toc_paths(&e.path(), paths, depth + 1)?;
        } else if ty.is_file()
            && e.path()
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("utoc"))
        {
            paths.push(e.path());
        }
    }
    Ok(())
}
struct Provider {
    stores: Vec<IoStore>,
    files: BTreeMap<String, (usize, usize)>,
    chunks: BTreeMap<[u8; 12], (usize, usize)>,
    names: RefCell<BTreeMap<(u64, u64), String>>,
    packages: BTreeMap<u64, (usize, usize)>,
    dependencies: BTreeMap<u64, Vec<u64>>,
    scripts: BTreeMap<u64, String>,
}
impl Provider {
    fn open(root: &Path, key: [u8; 32]) -> io::Result<Self> {
        let mut paths = Vec::new();
        toc_paths(root, &mut paths, 0)?;
        paths.sort_by_key(|p| {
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let priority = stem
                .strip_suffix("_P")
                .and_then(|s| s.rsplit('_').next())
                .and_then(|s| s.parse::<u64>().ok())
                .map_or(0, |n| n.saturating_add(1));
            (priority, p.clone())
        });
        let mut result = Self {
            stores: Vec::new(),
            files: BTreeMap::new(),
            chunks: BTreeMap::new(),
            names: RefCell::new(BTreeMap::new()),
            packages: BTreeMap::new(),
            dependencies: BTreeMap::new(),
            scripts: BTreeMap::new(),
        };
        for path in paths {
            let store = IoStore::open(&path, Some(key))?;
            let s = result.stores.len();
            for i in 0..store.summary().chunks {
                let id = store
                    .chunk_id(i)
                    .ok_or_else(|| invalid("missing chunk identity"))?;
                result.chunks.insert(id, (s, i));
                match id[11] {
                    1 => {
                        result
                            .packages
                            .insert(u64::from_le_bytes(id[..8].try_into().unwrap()), (s, i));
                    }
                    5 => {
                        let scripts = package::script_objects(&store.read_chunk(i)?)?;
                        if !result.scripts.is_empty() && result.scripts != scripts {
                            return Err(invalid("conflicting script object maps"));
                        }
                        result.scripts = scripts;
                    }
                    6 => result
                        .dependencies
                        .extend(package::dependencies(&store.read_chunk(i)?)?),
                    _ => (),
                }
            }
            for (name, i) in &store.files {
                result.files.insert(name.to_lowercase(), (s, *i));
            }
            result.stores.push(store);
        }
        if result.scripts.is_empty() {
            return Err(invalid("missing IoStore script objects"));
        }
        Ok(result)
    }
    fn package(&self, id: u64) -> io::Result<Package> {
        let &(s, i) = self
            .packages
            .get(&id)
            .ok_or_else(|| invalid(format!("missing imported package {id:016x}")))?;
        Package::parse(self.stores[s].read_chunk(i)?)
    }
    fn imported(&self, owner: u64, p: &Package, index: u64) -> io::Result<(u64, Package, usize)> {
        if index >> 62 == 0 {
            let i = usize::try_from(index).map_err(|_| invalid("export index overflow"))?;
            if i >= p.exports.len() {
                return Err(invalid("local export index out of bounds"));
            }
            return Ok((owner, self.package(owner)?, i));
        }
        if index >> 62 != 2 {
            return Err(invalid("expected package import reference"));
        }
        let package_index = ((index & 0x3fffffffffffffff) >> 32) as usize;
        let hash_index = (index & 0xffffffff) as usize;
        let id = *self
            .dependencies
            .get(&owner)
            .and_then(|d| d.get(package_index))
            .ok_or_else(|| invalid("missing container package dependency"))?;
        let hash = p
            .hashes
            .get(hash_index)
            .ok_or_else(|| invalid("public export hash index out of bounds"))?;
        let imported = self.package(id)?;
        let i = imported
            .exports
            .iter()
            .position(|e| e.hash == *hash)
            .ok_or_else(|| invalid("imported public export hash not found"))?;
        Ok((id, imported, i))
    }
    fn object_name(&self, owner: u64, p: &Package, index: u64) -> io::Result<String> {
        if index >> 62 == 1 {
            return self
                .scripts
                .get(&index)
                .cloned()
                .ok_or_else(|| invalid("missing native script object"));
        }
        if let Some(name) = self.names.borrow().get(&(owner, index)) {
            return Ok(name.clone());
        }
        let (_, p, i) = self.imported(owner, p, index)?;
        let name = p.exports[i].name.clone();
        self.names.borrow_mut().insert((owner, index), name.clone());
        Ok(name)
    }
    fn bulk(&self, owner: u64, b: &package::Bulk) -> io::Result<Vec<u8>> {
        let ty = if b.flags & (1 << 11) != 0 {
            3
        } else if b.flags & (1 << 12) != 0 {
            4
        } else {
            2
        };
        let mut id = [0; 12];
        id[..8].copy_from_slice(&owner.to_le_bytes());
        id[8..10].copy_from_slice(&u16::from(b.index).to_be_bytes());
        id[11] = ty;
        let &(s, i) = self
            .chunks
            .get(&id)
            .ok_or_else(|| invalid("missing texture bulk chunk"))?;
        let bytes = self.stores[s].read_chunk(i)?;
        let offset = usize::try_from(b.offset).map_err(|_| invalid("bulk offset overflow"))?;
        Ok(bytes
            .get(
                offset
                    ..offset
                        .checked_add(b.size)
                        .ok_or_else(|| invalid("bulk range overflow"))?,
            )
            .ok_or_else(|| invalid("bulk range exceeds chunk"))?
            .to_vec())
    }
}
struct Extractor<'a> {
    provider: &'a Provider,
    mapping: &'a Mapping,
    blueprints: BTreeMap<String, Blueprint>,
    images: BTreeMap<String, Value>,
    output: &'a Path,
    total: usize,
}
impl Extractor<'_> {
    fn context<'a>(&'a self, p: &'a Package) -> Context<'a> {
        Context {
            mapping: self.mapping,
            blueprints: &self.blueprints,
            package: p,
        }
    }
    fn blueprint(&mut self, id: u64, p: &Package, class: u64) -> io::Result<String> {
        let name = self.provider.object_name(id, p, class)?;
        if self.blueprints.contains_key(&name) || class >> 62 != 2 {
            return Ok(name);
        }
        let (owner, bp, i) = self.provider.imported(id, p, class)?;
        let export = &bp.exports[i];
        let native = self.provider.object_name(owner, &bp, export.class)?;
        if native != "BlueprintGeneratedClass" {
            return Err(invalid(format!("unsupported generated class {native}")));
        }
        let mut r = bp.body(export);
        self.context(&bp).object(&mut r, &native, export.flags)?;
        let parent = self
            .provider
            .object_name(owner, &bp, bp.reference(r.i32()?)?)?;
        let children = r.count(65536)?;
        r.take(children * 4)?;
        let mut fields = Vec::new();
        for _ in 0..r.count(4096)? {
            let kind = package::mapped(&bp.names, &mut r)?;
            let field = package::mapped(&bp.names, &mut r)?;
            r.u32()?;
            let dim = r.count(1024)?;
            r.u32()?;
            r.u64()?;
            r.u16()?;
            package::mapped(&bp.names, &mut r)?;
            r.u8()?;
            let kind = match kind.as_str() {
                "ObjectProperty" => {
                    r.i32()?;
                    super::usmap::PropertyType::Primitive(4)
                }
                "StructProperty" => {
                    let name = self
                        .provider
                        .object_name(owner, &bp, bp.reference(r.i32()?)?)?;
                    super::usmap::PropertyType::Struct(name)
                }
                _ => return Err(invalid(format!("unsupported Blueprint field type {kind}"))),
            };
            if dim != 1 {
                return Err(invalid("Blueprint fixed arrays are unsupported"));
            }
            fields.push((field, kind));
        }
        self.blueprints.insert(name.clone(), (parent, fields));
        Ok(name)
    }
    fn texture(&mut self, owner: u64, p: &Package, export: &Export) -> io::Result<String> {
        let object = format!("{}.{}", p.name, export.name);
        let reference = format!("WorldMaps/Textures/{}", hash(object.as_bytes()));
        if self.images.contains_key(&reference) {
            return Ok(reference);
        }
        if self.images.len() >= 8192 {
            return Err(invalid("map texture count exceeds limit"));
        }
        let mut r = p.body(export);
        self.context(p).object(&mut r, "Texture2D", export.flags)?;
        let stripped = r.u8()?;
        r.u8()?;
        if stripped & 1 == 0 {
            return Err(invalid("editor texture data is unsupported"));
        }
        r.take(2)?;
        if !r.flag()? || !r.flag()? {
            return Err(invalid("uncooked or missing texture mip data"));
        }
        let format = package::mapped(&p.names, &mut r)?;
        let skip_base = r.pos;
        let skip = usize::try_from(r.u64()?).map_err(|_| invalid("platform skip overflow"))?;
        let platform_end = skip_base
            .checked_add(skip)
            .filter(|n| *n <= r.bytes.len())
            .ok_or_else(|| invalid("platform data exceeds export"))?;
        if r.u8()? != 0 {
            return Err(invalid("derived texture data is unsupported"));
        }
        r.take(15)?;
        let width = r.count(4096)?;
        let height = r.count(4096)?;
        let packed = r.u32()?;
        if packed != 1 {
            return Err(invalid(
                "array, optional or virtual map textures are unsupported",
            ));
        }
        if r.string()? != format {
            return Err(invalid("texture pixel formats disagree"));
        }
        r.u32()?; // first mip to serialize
        let mips = r.count(16)?;
        if mips == 0 {
            return Err(invalid("texture has no mips"));
        }
        let mut first = None;
        for _ in 0..mips {
            let data_index = r.count(p.bulk.len())?;
            let b = p
                .bulk
                .get(data_index)
                .ok_or_else(|| invalid("bulk data map index exceeds range"))?;
            if b.flags & ((1 << 1) | (1 << 5) | (1 << 9)) != 0 {
                return Err(invalid("compressed or unused mip payload is unsupported"));
            }
            let inline = b.flags & (1 << 6) != 0;
            let data = if inline {
                r.take(b.size)?.to_vec()
            } else {
                self.provider.bulk(owner, b)?
            };
            let w = r.count(4096)?;
            let h = r.count(4096)?;
            let z = r.u32()?;
            if z != 1 {
                return Err(invalid("non-2D mip"));
            }
            if first.is_none() {
                first = Some((w, h, data));
            }
        }
        if r.flag()? {
            return Err(invalid("virtual map texture is unsupported"));
        }
        if r.pos != platform_end
            || package::mapped(&p.names, &mut r)? != "None"
            || r.pos != r.bytes.len()
        {
            return Err(invalid("unconsumed texture platform data"));
        }
        let (w, h, data) = first.ok_or_else(|| invalid("missing first mip"))?;
        if w == 0 || h == 0 || w > width || h > height {
            return Err(invalid("invalid available mip dimensions"));
        }
        let rgba = pixels::decode(&format, w, h, &data)?;
        let mut png = Vec::new();
        use image::ImageEncoder;
        image::codecs::png::PngEncoder::new_with_quality(
            &mut png,
            image::codecs::png::CompressionType::Default,
            image::codecs::png::FilterType::Adaptive,
        )
        .write_image(&rgba, w as u32, h as u32, image::ExtendedColorType::Rgba8)
        .map_err(io::Error::other)?;
        if png.len() > 16 * 1024 * 1024
            || self
                .total
                .checked_add(png.len())
                .is_none_or(|n| n > 1024 * 1024 * 1024)
        {
            return Err(invalid("map PNG allocation limit exceeded"));
        }
        let file = format!("{reference}.png");
        let path = self.output.join(&file);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, &png)?;
        self.total += png.len();
        self.images.insert(reference.clone(),json!({"reference":reference,"object_path":object,"file":file,"width":w,"height":h,"pixel_format":format,"sha256":hash(&png)}));
        Ok(reference)
    }
    fn level(&mut self, id: u64, p: &Package) -> io::Result<Value> {
        let mut actors = Vec::new();
        for export in &p.exports {
            let class = self.provider.object_name(id, p, export.class)?;
            // Class metadata establishes the actor's schema. Other exports do
            // not contribute sector bindings and need no UObject deserialization.
            if class != "BP_WorldMapPlane_C" {
                continue;
            }
            self.blueprint(id, p, export.class)?;
            let mut r = p.body(export);
            let fields = self.context(p).object(&mut r, &class, export.flags)?;
            let Some(texture) = fields.get("SectorTexture") else {
                continue;
            };
            let index = texture
                .as_i64()
                .and_then(|v| i32::try_from(v).ok())
                .ok_or_else(|| invalid("invalid sector texture index"))?;
            let (owner, texture, i) = self.provider.imported(id, p, p.reference(index)?)?;
            if self
                .provider
                .object_name(owner, &texture, texture.exports[i].class)?
                != "Texture2D"
            {
                return Err(invalid("sector reference is not Texture2D"));
            }
            let reference = self.texture(owner, &texture, &texture.exports[i])?;
            let (x, y, state) = match fields.get("SectorIndex") {
                Some(Value::Array(a)) if a.len() == 2 => (a[0].clone(), a[1].clone(), "explicit"),
                Some(Value::Null) => (json!(0), json!(0), "explicit"),
                None => (Value::Null, Value::Null, "inherited_unresolved"),
                _ => return Err(invalid("invalid sector index")),
            };
            actors.push(json!({"name":export.name,"actor_class":class,"sector_x":x,"sector_y":y,"index_state":state,"texture_reference":reference}));
        }
        Ok(
            json!({"source_path":format!("AION2/Content/{}.umap",p.name.strip_prefix("/Game/").unwrap()),"payload_sha256":hash(&p.bytes),"actors":actors}),
        )
    }
}

/// Extract into a fresh directory; only publish the manifest after all inputs
/// have been inspected. Per-asset failures stay explicit for catalogue coverage.
/// The manifest matches the existing SQLite image importer.
pub fn extract(paks: &Path, mapping: &Path, key_file: &Path, output: &Path) -> io::Result<Value> {
    let root = fs::canonicalize(paks)?;
    let mapping_bytes = input(mapping)?;
    let mapping = Mapping::parse(&mapping_bytes)?;
    if output.exists() {
        return Err(invalid("map output directory must not already exist"));
    }
    let provider = Provider::open(&root, read_key(key_file)?)?;
    let parent = fs::canonicalize(
        output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )?;
    if parent.starts_with(&root) {
        return Err(invalid("map output must be outside the game archives"));
    }
    fs::create_dir(output)?;
    let mut extractor = Extractor {
        provider: &provider,
        mapping: &mapping,
        blueprints: BTreeMap::new(),
        images: BTreeMap::new(),
        output,
        total: 0,
    };
    let mut levels = Vec::new();
    let mut errors = Vec::new();
    let mut texture_errors = Vec::new();
    let mut packages = 0;
    let mut texture_packages = 0;
    for (path, &(s, i)) in &provider.files {
        let level = path.starts_with("aion2/content/ui/map/") && path.ends_with(".umap");
        let texture =
            path.starts_with("aion2/content/ui/map/worldmap/") && path.ends_with(".uasset");
        if !level && !texture {
            continue;
        }
        if level {
            packages += 1;
        } else {
            texture_packages += 1;
        }
        if packages > 4096 || texture_packages > 8192 {
            return Err(invalid("map inventory exceeds limits"));
        }
        let mut process = || -> io::Result<Value> {
            let id = provider.stores[s]
                .chunk_id(i)
                .ok_or_else(|| invalid("missing package chunk"))?;
            let owner = u64::from_le_bytes(id[..8].try_into().unwrap());
            let p = provider.package(owner)?;
            if level {
                extractor.level(owner, &p)
            } else {
                for e in &p.exports {
                    if provider.object_name(owner, &p, e.class)? == "Texture2D" {
                        extractor.texture(owner, &p, e)?;
                    }
                }
                Ok(Value::Null)
            }
        };
        match process() {
            Ok(value) => {
                if level {
                    levels.push(value)
                }
            }
            Err(error) => {
                if errors.len() + texture_errors.len() < 5 {
                    eprintln!("Map asset {path}: {error}");
                }
                let error = json!({"source_path":path,"error":error.to_string()});
                if level {
                    errors.push(error)
                } else {
                    texture_errors.push(error)
                }
            }
        }
        if (packages + texture_packages) % 100 == 0 {
            eprintln!(
                "Read {} map packages; {} textures, {} errors",
                packages + texture_packages,
                extractor.images.len(),
                errors.len() + texture_errors.len()
            );
        }
    }
    if packages == 0 || texture_packages == 0 {
        return Err(invalid("no world-map package inventory found"));
    }
    let report = json!({"format":1,"mapping_sha256":hash(&mapping_bytes),"packages":packages,"texture_packages":texture_packages,"levels":levels,"images":extractor.images.values().collect::<Vec<_>>(),"errors":errors,"texture_errors":texture_errors});
    crate::engine_ini::storage::atomic_replace(
        &output.join("world-maps.json"),
        &serde_json::to_vec_pretty(&report).map_err(io::Error::other)?,
        None,
    )?;
    Ok(
        json!({"levels":levels.len(),"textures":extractor.images.len(),"level_errors":errors.len(),"texture_errors":texture_errors.len(),"png_bytes":extractor.total,"output":output.display().to_string()}),
    )
}
