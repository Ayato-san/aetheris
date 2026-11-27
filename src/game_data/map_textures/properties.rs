//! Cooked unversioned properties. Schema indices come from USMAP or the actual
//! Blueprint class, not from string searches in actor payloads.
use super::package::{Package, mapped};
use crate::game_data::{
    binary::{Reader, invalid},
    usmap::{Mapping, PropertyType},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, io};

pub(super) type Blueprint = (String, Vec<(String, PropertyType)>);
pub(super) struct Context<'a> {
    pub mapping: &'a Mapping,
    pub blueprints: &'a BTreeMap<String, Blueprint>,
    pub package: &'a Package,
}
impl Context<'_> {
    fn property<'a>(
        &'a self,
        class: &str,
        index: usize,
    ) -> io::Result<(&'a str, &'a PropertyType)> {
        if let Some((parent, properties)) = self.blueprints.get(class) {
            if let Some((name, kind)) = properties.get(index) {
                return Ok((name, kind));
            }
            return self
                .mapping
                .package_property(parent, index - properties.len());
        }
        self.mapping.package_property(class, index)
    }
    pub fn fields(
        &self,
        r: &mut Reader<'_>,
        class: &str,
        depth: usize,
    ) -> io::Result<BTreeMap<String, Value>> {
        if depth > 32 {
            return Err(invalid("cooked property nesting exceeds limit"));
        }
        let mut fragments = Vec::new();
        let mut zero_count = 0usize;
        loop {
            if fragments.len() >= 4096 {
                return Err(invalid("unversioned fragments exceed limit"));
            }
            let f = r.u16()?;
            if f & 128 != 0 {
                zero_count += usize::from(f >> 9);
            }
            fragments.push(f);
            if f & 256 != 0 {
                break;
            }
        }
        let mask_size = match zero_count {
            0 => 0,
            1..=8 => 1,
            9..=16 => 2,
            n => n.div_ceil(32) * 4,
        };
        let mask = r.take(mask_size)?.to_vec();
        let mut bit = 0;
        let mut index = 0usize;
        let mut fields = BTreeMap::new();
        for f in fragments {
            index += usize::from(f & 127);
            for _ in 0..f >> 9 {
                let zero = if f & 128 != 0 {
                    let z = mask[bit / 8] & (1 << (bit % 8)) != 0;
                    bit += 1;
                    z
                } else {
                    false
                };
                let (name, kind) = self
                    .property(class, index)
                    .map_err(|e| invalid(format!("{class}[{index}]: {e}")))?;
                let value = if zero {
                    Value::Null
                } else {
                    self.value(r, kind, depth + 1)
                        .map_err(|e| invalid(format!("{class}.{name}: {e}")))?
                };
                if fields.insert(name.to_owned(), value).is_some() {
                    return Err(invalid("duplicate cooked field"));
                }
                index += 1;
            }
        }
        Ok(fields)
    }
    fn value(&self, r: &mut Reader<'_>, kind: &PropertyType, depth: usize) -> io::Result<Value> {
        if depth > 32 {
            return Err(invalid("cooked value nesting exceeds limit"));
        }
        Ok(match kind {
            PropertyType::Primitive(tag) => match tag {
                0 | 1 | 23 => json!(r.u8()?),
                2 | 4 | 14 | 19 => json!(r.i32()?),
                3 => json!(f32::from_bits(r.u32()?)),
                5 => json!(mapped(&self.package.names, r)?),
                7 => json!(f64::from_bits(r.u64()?)),
                10 => json!(r.string()?),
                18 | 21 => json!(r.u64()?),
                20 | 22 => json!(r.u16()?),
                _ => return Err(invalid(format!("unsupported cooked property type {tag}"))),
            },
            PropertyType::Enum(_) => json!(r.u8()?),
            PropertyType::Struct(name) => match name.as_str() {
                "Guid" => json!(hex::encode(r.take(16)?)),
                "IntPoint" => json!([r.i32()?, r.i32()?]),
                "Vector" | "Rotator" => json!([
                    f64::from_bits(r.u64()?),
                    f64::from_bits(r.u64()?),
                    f64::from_bits(r.u64()?)
                ]),
                "Vector2D" => json!([f64::from_bits(r.u64()?), f64::from_bits(r.u64()?)]),
                "PerPlatformFloat" => {
                    if !r.flag()? {
                        return Err(invalid("uncooked per-platform float"));
                    }
                    json!(f32::from_bits(r.u32()?))
                }
                _ => serde_json::to_value(self.fields(r, name, depth + 1)?)
                    .map_err(io::Error::other)?,
            },
            PropertyType::Array(inner) => {
                let n = r.count(65536)?;
                let mut values = Vec::with_capacity(n);
                for _ in 0..n {
                    values.push(self.value(r, inner, depth + 1)?);
                }
                Value::Array(values)
            }
            PropertyType::Map(key, value) => {
                if r.u32()? != 0 {
                    return Err(invalid("cooked map removals are unsupported"));
                }
                let n = r.count(65536)?;
                let mut values = Vec::with_capacity(n);
                for _ in 0..n {
                    values.push(json!([
                        self.value(r, key, depth + 1)?,
                        self.value(r, value, depth + 1)?
                    ]));
                }
                Value::Array(values)
            }
            _ => return Err(invalid("unsupported cooked collection type")),
        })
    }
    pub fn object(
        &self,
        r: &mut Reader<'_>,
        class: &str,
        flags: u32,
    ) -> io::Result<BTreeMap<String, Value>> {
        let fields = self.fields(r, class, 0)?;
        if flags & 16 == 0 && r.flag()? {
            r.take(16)?;
        }
        Ok(fields)
    }
}
