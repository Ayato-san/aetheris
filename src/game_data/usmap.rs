//! Bounded USMAP reader for offline layout analysis. Mappings do not establish
//! the client version, region, or semantic accuracy of the decoded fields.
use super::{
    binary::{Reader, invalid},
    schema::{self, Field, Kind, LayoutProbe},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

const LIMIT: usize = 1_000_000;
const MAX_BYTES: usize = 64 * 1024 * 1024;

pub struct Mapping {
    sha256: String,
    version: u8,
    structs: BTreeMap<String, Structure>,
    ambiguous_structs: BTreeSet<String>,
}

#[derive(Eq, PartialEq)]
struct Structure {
    parent: Option<String>,
    properties: BTreeMap<u16, Property>,
}

#[derive(Eq, PartialEq)]
struct Property {
    name: String,
    dimension: u8,
    kind: PropertyType,
}

#[derive(Eq, PartialEq)]
enum PropertyType {
    Primitive(u8),
    Struct(String),
    Enum(String),
    Array(Box<PropertyType>),
    Set(Box<PropertyType>),
    Map(Box<PropertyType>, Box<PropertyType>),
    Unsupported(u8),
}

#[derive(Deserialize, Serialize)]
pub struct TableProbe {
    pub mapping_sha256: String,
    pub mapping_version: u8,
    pub mapping_ambiguous_structures: BTreeSet<String>,
    pub row_struct: String,
    pub payload_sha256: String,
    pub payload_bytes: usize,
    #[serde(flatten)]
    pub layout: LayoutProbe,
}

fn name(reader: &mut Reader<'_>, names: &[String]) -> io::Result<Option<String>> {
    let index = reader.u32()?;
    if index == u32::MAX {
        return Ok(None);
    }
    names
        .get(index as usize)
        .cloned()
        .map(Some)
        .ok_or_else(|| invalid("USMAP name index out of bounds"))
}

fn required_name(reader: &mut Reader<'_>, names: &[String]) -> io::Result<String> {
    name(reader, names)?
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid("missing USMAP name"))
}

fn property_type(
    reader: &mut Reader<'_>,
    names: &[String],
    depth: usize,
    budget: &mut usize,
) -> io::Result<PropertyType> {
    if depth > 64 || *budget == 0 {
        return Err(invalid("USMAP type complexity exceeds limit"));
    }
    *budget -= 1;
    let tag = reader.u8()?;
    Ok(match tag {
        9 => PropertyType::Struct(required_name(reader, names)?),
        26 => {
            property_type(reader, names, depth + 1, budget)?;
            PropertyType::Enum(required_name(reader, names)?)
        }
        8 => PropertyType::Array(Box::new(property_type(reader, names, depth + 1, budget)?)),
        25 => PropertyType::Set(Box::new(property_type(reader, names, depth + 1, budget)?)),
        24 => PropertyType::Map(
            Box::new(property_type(reader, names, depth + 1, budget)?),
            Box::new(property_type(reader, names, depth + 1, budget)?),
        ),
        28 => {
            property_type(reader, names, depth + 1, budget)?;
            PropertyType::Unsupported(tag)
        }
        0..=36 | 253..=255 => PropertyType::Primitive(tag),
        _ => return Err(invalid("unknown USMAP property encoding")),
    })
}

impl Mapping {
    /// Versions 0..4, uncompressed. Package/custom version metadata is bounded
    /// and skipped; compressed mappings and engine-version headers are refused.
    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(invalid("USMAP exceeds size limit"));
        }
        let mut reader = Reader::new(bytes);
        if reader.u16()? != 0x30c4 {
            return Err(invalid("invalid USMAP magic"));
        }
        let version = reader.u8()?;
        if version > 4 {
            return Err(invalid("unsupported USMAP version"));
        }
        if version >= 1 && reader.flag()? {
            reader.take(8)?; // UE4 and UE5 package versions
            let count = reader.count(LIMIT)?;
            reader.take(count * 20)?; // GUID and custom version
            reader.u32()?; // network changelist
        }
        if reader.u8()? != 0 {
            return Err(invalid("compressed USMAP is unsupported"));
        }
        let stored = reader.count(MAX_BYTES)?;
        let unpacked = reader.count(MAX_BYTES)?;
        if stored != unpacked {
            return Err(invalid("USMAP size mismatch"));
        }
        let payload = reader.take(stored)?;
        if reader.pos != bytes.len() {
            return Err(invalid("trailing USMAP bytes"));
        }
        let mut reader = Reader::new(payload);
        let count = reader.count(LIMIT)?;
        let mut names = Vec::new();
        for _ in 0..count {
            let len = if version >= 2 {
                reader.u16()? as usize
            } else {
                reader.u8()? as usize
            };
            let value = std::str::from_utf8(reader.take(len)?)
                .map_err(|_| invalid("invalid USMAP UTF-8"))?;
            if value.contains('\0') {
                return Err(invalid("NUL in USMAP name"));
            }
            names.push(value.to_owned());
        }
        // Enums are serialized as names by Aion table payloads. Their numeric
        // lookup is not needed, but all bytes and name references are checked.
        let mut budget = LIMIT;
        for _ in 0..reader.count(LIMIT)? {
            required_name(&mut reader, &names)?;
            let count = if version >= 3 {
                reader.u16()? as usize
            } else {
                reader.u8()? as usize
            };
            budget = budget
                .checked_sub(count)
                .ok_or_else(|| invalid("USMAP enum complexity exceeds limit"))?;
            for _ in 0..count {
                if version >= 4 {
                    reader.u64()?;
                }
                required_name(&mut reader, &names)?;
            }
        }
        let mut structs = BTreeMap::new();
        let mut ambiguous_structs = BTreeSet::new();
        budget = LIMIT;
        for _ in 0..reader.count(LIMIT)? {
            let structure_name = required_name(&mut reader, &names)?;
            let parent = name(&mut reader, &names)?;
            let property_count = reader.u16()?;
            let count = reader.u16()?;
            let mut properties = BTreeMap::new();
            let mut occupied = BTreeSet::new();
            for _ in 0..count {
                let index = reader.u16()?;
                let dimension = reader.u8()?;
                if dimension == 0
                    || usize::from(index) + usize::from(dimension) > usize::from(property_count)
                {
                    return Err(invalid("USMAP property range out of bounds"));
                }
                for slot in index..index + u16::from(dimension) {
                    if !occupied.insert(slot) {
                        return Err(invalid("overlapping USMAP properties"));
                    }
                }
                let property = Property {
                    name: required_name(&mut reader, &names)?,
                    dimension,
                    kind: property_type(&mut reader, &names, 0, &mut budget)?,
                };
                properties.insert(index, property);
            }
            let definition = Structure { parent, properties };
            let key = structure_name.to_ascii_lowercase();
            if let Some(previous) = structs.get(&key) {
                if previous != &definition {
                    ambiguous_structs.insert(key);
                }
            } else {
                structs.insert(key, definition);
            }
        }
        if reader.pos != payload.len() {
            return Err(invalid("unconsumed USMAP payload"));
        }
        Ok(Self {
            sha256: hex::encode(Sha256::digest(bytes)),
            version,
            structs,
            ambiguous_structs,
        })
    }

    fn fields(&self, name: &str, depth: usize, budget: &mut usize) -> io::Result<Vec<Field>> {
        if depth > 64 || *budget == 0 {
            return Err(invalid("USMAP structure complexity or cycle exceeds limit"));
        }
        *budget -= 1;
        if self.ambiguous_structs.contains(&name.to_ascii_lowercase()) {
            return Err(invalid(format!("ambiguous USMAP structure {name}")));
        }
        let structure = self
            .structs
            .get(&name.to_ascii_lowercase())
            .ok_or_else(|| invalid(format!("missing USMAP structure {name}")))?;
        let mut fields = match &structure.parent {
            Some(parent) => self.fields(parent, depth + 1, budget)?,
            None => Vec::new(),
        };
        for property in structure.properties.values() {
            // Fixed arrays have no count/tag in the table. Expand into distinct
            // source-indexed fields, retaining every element.
            for index in 0..property.dimension {
                *budget = budget
                    .checked_sub(1)
                    .ok_or_else(|| invalid("USMAP layout complexity exceeds limit"))?;
                let field_name = if property.dimension == 1 {
                    property.name.clone()
                } else {
                    format!("{}[{index}]", property.name)
                };
                let kind = self.kind(&property.kind, &property.name, true, depth + 1, budget)?;
                fields.push(Field {
                    name: field_name,
                    kind,
                });
            }
        }
        let mut unique = BTreeSet::new();
        if fields.iter().any(|f| !unique.insert(&f.name)) {
            return Err(invalid("duplicate USMAP layout field"));
        }
        Ok(fields)
    }

    fn kind(
        &self,
        kind: &PropertyType,
        field: &str,
        tagged: bool,
        depth: usize,
        budget: &mut usize,
    ) -> io::Result<Kind> {
        if depth > 64 || *budget == 0 {
            return Err(invalid("USMAP layout complexity exceeds limit"));
        }
        *budget -= 1;
        Ok(match kind {
            PropertyType::Primitive(tag) => match tag {
                0 => Kind::U8,
                1 => Kind::Bool,
                2 => Kind::I32,
                3 => Kind::F32,
                5 => Kind::String { xor_utf16: true },
                7 => Kind::F64,
                10 => Kind::String { xor_utf16: false },
                11 => Kind::Text,
                18 => Kind::U64,
                19 => Kind::U32,
                20 => Kind::U16,
                21 => Kind::I64,
                22 => Kind::I16,
                23 => Kind::I8,
                _ => {
                    return Err(invalid(format!(
                        "unsupported USMAP property {tag} in {field}"
                    )));
                }
            },
            PropertyType::Unsupported(tag) => {
                return Err(invalid(format!(
                    "unsupported USMAP property {tag} in {field}"
                )));
            }
            PropertyType::Struct(name) => Kind::Struct {
                fields: self.fields(name, depth + 1, budget)?,
            },
            PropertyType::Enum(name) => Kind::Enum {
                prefix: name.clone(),
            },
            PropertyType::Array(inner) => {
                if tagged && let PropertyType::Struct(name) = &**inner {
                    Kind::TaggedArray {
                        field: field.into(),
                        struct_name: name.clone(),
                        fields: self.fields(name, depth + 1, budget)?,
                    }
                } else {
                    Kind::Array {
                        element: Box::new(self.kind(inner, field, tagged, depth + 1, budget)?),
                    }
                }
            }
            PropertyType::Set(inner) => Kind::Set {
                element: Box::new(self.kind(inner, field, tagged, depth + 1, budget)?),
            },
            PropertyType::Map(key, value) => Kind::Map {
                key: Box::new(self.kind(key, field, false, depth + 1, budget)?),
                value: Box::new(self.kind(value, field, false, depth + 1, budget)?),
            },
        })
    }

    /// Try a layout without adding a compatibility declaration to a schema.
    /// ItemTableItem uses the explicit Aion v13 discriminated row profile.
    pub fn probe_table(&self, row_struct: &str, payload: &[u8]) -> io::Result<TableProbe> {
        let mut budget = LIMIT;
        let mut fields = self.fields(row_struct, 0, &mut budget)?;
        if row_struct.eq_ignore_ascii_case("ItemTableItem") {
            if payload.get(..4) != Some(&13i32.to_le_bytes()) {
                return Err(invalid(
                    "Item row variant profile requires table version 13",
                ));
            }
            let mut cases = BTreeMap::new();
            for (value, structure) in [
                ("Currency", "ItemTableCurrencyInfo"),
                ("Equip", "ItemTableEquipmentInfo"),
                ("Usable", "ItemTableUsableInfo"),
                ("Misc", "ItemTableMiscInfo"),
            ] {
                cases.insert(
                    format!("EItemType::{value}"),
                    self.fields(structure, 0, &mut budget)?,
                );
            }
            fields.push(Field {
                name: "$variant".into(),
                kind: Kind::Variant {
                    discriminator: "ItemType".into(),
                    cases,
                },
            });
        }
        let layout = schema::probe_layout(fields, payload)?;
        Ok(TableProbe {
            mapping_sha256: self.sha256.clone(),
            mapping_version: self.version,
            mapping_ambiguous_structures: self.ambiguous_structs.clone(),
            row_struct: row_struct.into(),
            payload_sha256: hex::encode(Sha256::digest(payload)),
            payload_bytes: payload.len(),
            layout,
        })
    }
}
