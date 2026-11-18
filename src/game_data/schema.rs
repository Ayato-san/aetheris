//! Strict, bounded binary decoding. Schemas declare their client compatibility;
//! successful parsing alone never establishes compatibility or field semantics.
use super::ClientIdentity;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{collections::BTreeMap, io};

const MAX_COUNT: usize = 1_000_000;
const MAX_PAYLOAD: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Schema {
    pub id: String,
    pub compatible_clients: Vec<ClientIdentity>,
    pub table_version: i32,
    /// Exact column hashes in serialization order, established from the client.
    pub column_hashes: Vec<String>,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    pub kind: Kind,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Kind {
    I32,
    U32,
    I64,
    U64,
    U16,
    I16,
    U8,
    I8,
    F32,
    F64,
    Bool,
    String {
        xor_utf16: bool,
    },
    Enum {
        prefix: String,
    },
    Struct {
        fields: Vec<Field>,
    },
    /// Untagged arrays only. Tagged Unreal struct arrays require a different
    /// reader and must not be represented by this variant.
    Array {
        element: Box<Kind>,
    },
    TaggedArray {
        field: String,
        struct_name: String,
        fields: Vec<Field>,
    },
    Map {
        key: Box<Kind>,
        value: Box<Kind>,
    },
    Set {
        element: Box<Kind>,
    },
    Text,
    /// A row extension selected by a previously decoded string field.
    /// Header columns include every case; each row carries only one case.
    Variant {
        discriminator: String,
        cases: BTreeMap<String, Vec<Field>>,
    },
}

/// Structural analysis only: it does not declare client/region compatibility.
#[derive(Debug, Deserialize, Serialize)]
pub struct LayoutProbe {
    pub compatibility_verified: bool,
    pub table_version: i32,
    pub column_hashes: Vec<String>,
    pub rows: Vec<Value>,
}

pub(super) fn probe_layout(fields: Vec<Field>, payload: &[u8]) -> io::Result<LayoutProbe> {
    let mut reader = Reader {
        bytes: payload,
        offset: 0,
        budget: MAX_COUNT,
    };
    let table_version = reader.i32()?;
    let hashes = reader.string(true)?;
    let column_hashes = if hashes.is_empty() {
        Vec::new()
    } else {
        hashes.split(',').map(str::to_owned).collect()
    };
    let schema = Schema {
        id: "layout-probe".into(),
        compatible_clients: Vec::new(),
        table_version,
        column_hashes,
        fields,
    };
    let rows = schema.decode_payload(payload)?;
    Ok(LayoutProbe {
        compatibility_verified: false,
        table_version,
        column_hashes: schema.column_hashes,
        rows,
    })
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
    budget: usize,
}

impl Reader<'_> {
    fn take(&mut self, size: usize) -> io::Result<&[u8]> {
        let end = self
            .offset
            .checked_add(size)
            .ok_or_else(|| invalid("offset overflow"))?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| invalid(format!("truncated payload at {}", self.offset)))?;
        self.offset = end;
        Ok(bytes)
    }

    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn count(&mut self) -> io::Result<usize> {
        let count = usize::try_from(self.i32()?).map_err(|_| invalid("negative count"))?;
        if count > MAX_COUNT {
            return Err(invalid("count exceeds limit"));
        }
        Ok(count)
    }

    fn string(&mut self, xor_utf16: bool) -> io::Result<String> {
        let count = self.i32()?;
        if count == 0 {
            return Ok(String::new());
        }
        let units = count.unsigned_abs() as usize;
        if units > MAX_COUNT {
            return Err(invalid("string exceeds limit"));
        }
        if count > 0 {
            let bytes = self.take(units)?;
            if bytes.last() != Some(&0) {
                return Err(invalid("unterminated UTF-8 string"));
            }
            let value = String::from_utf8(bytes[..bytes.len() - 1].to_vec())
                .map_err(|e| invalid(e.to_string()))?;
            if value.contains('\0') {
                return Err(invalid("embedded NUL in string"));
            }
            Ok(value)
        } else {
            let mut bytes = self.take(units * 2)?.to_vec();
            if bytes[bytes.len() - 2..] != [0, 0] {
                return Err(invalid("unterminated UTF-16 string"));
            }
            bytes.truncate(bytes.len() - 2);
            if xor_utf16 {
                for (i, byte) in bytes.iter_mut().enumerate() {
                    *byte ^= [0x25, 0, 0xa8, 0, 0x7e, 0, 0x91, 0][i % 8];
                }
            }
            let words: Vec<_> = bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect();
            let value = String::from_utf16(&words).map_err(|e| invalid(e.to_string()))?;
            if value.contains('\0') {
                return Err(invalid("embedded NUL in string"));
            }
            Ok(value)
        }
    }

    fn fields(&mut self, fields: &[Field], depth: usize) -> io::Result<Value> {
        if depth > 64 || self.budget == 0 {
            return Err(invalid("decoding complexity exceeds limit"));
        }
        let mut object = Map::new();
        for field in fields {
            if self.budget == 0 {
                return Err(invalid("decoding complexity exceeds limit"));
            }
            if field.name.is_empty() || object.contains_key(&field.name) {
                return Err(invalid("empty or duplicate field"));
            }
            let value = if let Kind::Variant {
                discriminator,
                cases,
            } = &field.kind
            {
                let selected = object
                    .get(discriminator)
                    .and_then(Value::as_str)
                    .and_then(|value| cases.get(value))
                    .ok_or_else(|| invalid("missing or unsupported row discriminator"))?;
                self.budget -= 1;
                self.fields(selected, depth + 1)?
            } else {
                self.value(&field.kind, depth + 1)?
            };
            object.insert(field.name.clone(), value);
        }
        Ok(Value::Object(object))
    }

    fn value(&mut self, kind: &Kind, depth: usize) -> io::Result<Value> {
        if depth > 64 || self.budget == 0 {
            return Err(invalid("decoding complexity exceeds limit"));
        }
        self.budget -= 1;
        macro_rules! number {
            ($ty:ty, $size:expr) => {
                Value::from(<$ty>::from_le_bytes(self.take($size)?.try_into().unwrap()))
            };
        }
        Ok(match kind {
            Kind::I32 => number!(i32, 4),
            Kind::U32 => number!(u32, 4),
            Kind::I64 => number!(i64, 8),
            Kind::U64 => number!(u64, 8),
            Kind::I16 => number!(i16, 2),
            Kind::U16 => number!(u16, 2),
            Kind::I8 => Value::from(self.take(1)?[0] as i8),
            Kind::U8 => Value::from(self.take(1)?[0]),
            Kind::Bool => match self.take(1)?[0] {
                0 => Value::Bool(false),
                1 => Value::Bool(true),
                _ => return Err(invalid("invalid boolean")),
            },
            Kind::F32 => {
                let value = f32::from_le_bytes(self.take(4)?.try_into().unwrap());
                if !value.is_finite() {
                    return Err(invalid("non-finite float"));
                }
                Value::from(value)
            }
            Kind::F64 => {
                let value = f64::from_le_bytes(self.take(8)?.try_into().unwrap());
                if !value.is_finite() {
                    return Err(invalid("non-finite float"));
                }
                Value::from(value)
            }
            Kind::String { xor_utf16 } => Value::String(self.string(*xor_utf16)?),
            Kind::Enum { prefix } => {
                let value = self.string(true)?;
                if !value.is_empty()
                    && value != "None"
                    && !value.starts_with(&format!("{prefix}::"))
                {
                    return Err(invalid("enum prefix mismatch"));
                }
                Value::String(value)
            }
            Kind::Struct { fields } => self.fields(fields, depth)?,
            Kind::Array { element } => {
                let count = self.count()?;
                let mut values = Vec::new();
                for _ in 0..count {
                    values.push(self.value(element, depth + 1)?);
                }
                Value::Array(values)
            }
            Kind::TaggedArray {
                field,
                struct_name,
                fields,
            } => {
                let count = self.count()?;
                let name = self.string(true)?;
                let tag = self.string(true)?;
                let size = i64::from_le_bytes(self.take(8)?.try_into().unwrap());
                let actual_struct = self.string(true)?;
                self.take(16)?; // struct GUID, retained in source payload provenance
                match self.take(1)?[0] {
                    0 => (),
                    1 => {
                        self.take(16)?;
                    }
                    _ => return Err(invalid("invalid property GUID flag")),
                }
                if tag != "StructProperty"
                    || size < 0
                    || actual_struct != *struct_name
                    || (!field.starts_with("Unknown") && !name.eq_ignore_ascii_case(field))
                {
                    return Err(invalid("struct array tag mismatch"));
                }
                let end = self
                    .offset
                    .checked_add(size as usize)
                    .filter(|end| *end <= self.bytes.len())
                    .ok_or_else(|| invalid("struct array size exceeds payload"))?;
                let mut values = Vec::new();
                for _ in 0..count {
                    values.push(self.fields(fields, depth + 1)?);
                }
                if self.offset != end {
                    return Err(invalid("struct array tag size mismatch"));
                }
                Value::Array(values)
            }
            Kind::Map { key, value } => {
                let removed = self.sequence(key, depth)?;
                let count = self.count()?;
                let mut entries = Vec::new();
                for _ in 0..count {
                    entries.push(serde_json::json!({"Key":self.value(key,depth+1)?,"Value":self.value(value,depth+1)?}));
                }
                serde_json::json!({"Removed":removed,"Entries":entries})
            }
            Kind::Set { element } => {
                let removed = self.sequence(element, depth)?;
                let values = self.sequence(element, depth)?;
                serde_json::json!({"Removed":removed,"Values":values})
            }
            Kind::Text => {
                let flags = u32::from_le_bytes(self.take(4)?.try_into().unwrap());
                if flags > 0xffff {
                    return Err(invalid("unsupported text flags"));
                }
                match self.take(1)?[0] as i8 {
                    -1 => {
                        let value = match self.i32()? {
                            0 => None,
                            1 => Some(self.string(false)?),
                            _ => return Err(invalid("invalid culture invariant flag")),
                        };
                        serde_json::json!({"Flags":flags,"CultureInvariantString":value})
                    }
                    0 => {
                        serde_json::json!({"Flags":flags,"Namespace":self.string(false)?,"Key":self.string(false)?,"SourceString":self.string(false)?})
                    }
                    _ => return Err(invalid("unsupported text history")),
                }
            }
            Kind::Variant { .. } => {
                return Err(invalid("row variant requires sibling discriminator"));
            }
        })
    }

    fn sequence(&mut self, kind: &Kind, depth: usize) -> io::Result<Vec<Value>> {
        let count = self.count()?;
        let mut values = Vec::new();
        for _ in 0..count {
            values.push(self.value(kind, depth + 1)?);
        }
        Ok(values)
    }
}

impl Schema {
    pub fn decode(&self, client: &ClientIdentity, payload: &[u8]) -> io::Result<Vec<Value>> {
        if self.id.is_empty() || !self.compatible_clients.contains(client) {
            return Err(invalid(
                "schema does not declare this client version and region compatible",
            ));
        }
        self.decode_payload(payload)
    }

    fn decode_payload(&self, payload: &[u8]) -> io::Result<Vec<Value>> {
        if payload.len() > MAX_PAYLOAD {
            return Err(invalid("payload exceeds limit"));
        }
        let columns = self.fields.iter().try_fold(0usize, |total, f| {
            total
                .checked_add(f.kind.columns(0)?)
                .ok_or_else(|| invalid("column count overflow"))
        })?;
        if columns != self.column_hashes.len() {
            return Err(invalid("schema leaf count does not match column hashes"));
        }
        let mut reader = Reader {
            bytes: payload,
            offset: 0,
            budget: 16 * MAX_COUNT,
        };
        if reader.i32()? != self.table_version {
            return Err(invalid("table version mismatch"));
        }
        let hashes = reader.string(true)?;
        let actual: Vec<_> = if hashes.is_empty() {
            Vec::new()
        } else {
            hashes.split(',').map(str::to_owned).collect()
        };
        if actual != self.column_hashes {
            return Err(invalid("column hashes mismatch"));
        }
        let count = reader.count()?;
        let mut rows = Vec::new();
        for index in 0..count {
            rows.push(
                reader
                    .fields(&self.fields, 0)
                    .map_err(|e| invalid(format!("row {index}, offset {}: {e}", reader.offset)))?,
            );
        }
        if reader.offset != payload.len() {
            return Err(invalid(format!("unconsumed bytes at {}", reader.offset)));
        }
        Ok(rows)
    }
}

impl Kind {
    fn columns(&self, depth: usize) -> io::Result<usize> {
        if depth > 64 {
            return Err(invalid("schema nesting exceeds limit"));
        }
        match self {
            Kind::Variant { cases, .. } => cases.values().try_fold(0usize, |sum, fields| {
                fields.iter().try_fold(sum, |sum, field| {
                    sum.checked_add(field.kind.columns(depth + 1)?)
                        .ok_or_else(|| invalid("column count overflow"))
                })
            }),
            Kind::Struct { fields } | Kind::TaggedArray { fields, .. } => {
                fields.iter().try_fold(0usize, |sum, f| {
                    sum.checked_add(f.kind.columns(depth + 1)?)
                        .ok_or_else(|| invalid("column count overflow"))
                })
            }
            Kind::Array { element } | Kind::Set { element } => element.columns(depth + 1),
            Kind::Map { key, value } => key
                .columns(depth + 1)?
                .checked_add(value.columns(depth + 1)?)
                .ok_or_else(|| invalid("column count overflow")),
            _ => Ok(1),
        }
    }
}
