//! Compile Penpot tokens to native constants. No runtime filesystem dependency.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

type Result<T> = std::result::Result<T, String>;

fn flatten(value: &Value, prefix: &str, tokens: &mut BTreeMap<String, Value>) {
    if value.get("$value").is_some() {
        tokens.insert(prefix.to_owned(), value.clone());
    } else if let Some(object) = value.as_object() {
        for (key, child) in object {
            if !key.starts_with('$') {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten(child, &path, tokens);
            }
        }
    }
}

fn resolve(
    value: &Value,
    tokens: &BTreeMap<String, Value>,
    stack: &mut BTreeSet<String>,
) -> Result<Value> {
    match value {
        Value::String(text) => {
            let mut result = text.clone();
            while let Some(start) = result.find('{') {
                let end = result[start..]
                    .find('}')
                    .ok_or("unclosed token reference")?
                    + start;
                let key = result[start + 1..end].to_owned();
                if !stack.insert(key.clone()) {
                    return Err(format!("cyclic token reference: {key}"));
                }
                let token = tokens
                    .get(&key)
                    .ok_or_else(|| format!("unknown token: {key}"))?;
                let resolved = resolve(&token["$value"], tokens, stack)?;
                stack.remove(&key);
                if start == 0 && end == result.len() - 1 {
                    return Ok(resolved);
                }
                let replacement = match &resolved {
                    Value::String(text) => text.clone(),
                    Value::Number(number) => number.to_string(),
                    _ => return Err("embedded reference must be scalar".into()),
                };
                result.replace_range(start..=end, &format!("({replacement})"));
            }
            Ok(Value::String(result))
        }
        Value::Array(values) => values.iter().map(|v| resolve(v, tokens, stack)).collect(),
        Value::Object(values) => values
            .iter()
            .map(|(k, v)| Ok((k.clone(), resolve(v, tokens, stack)?)))
            .collect(),
        _ => Ok(value.clone()),
    }
}

// Arithmetic used by the export (e.g. {size-base}/4*3px), with normal precedence.
fn number(value: &Value) -> Result<f32> {
    if let Some(n) = value.as_f64() {
        let n = n as f32;
        return if n.is_finite() {
            Ok(n)
        } else {
            Err("non-finite numeric token".into())
        };
    }
    let text = value
        .as_str()
        .ok_or("expected numeric token")?
        .replace("px", "");
    struct Parser<'a> {
        bytes: &'a [u8],
        pos: usize,
    }
    impl Parser<'_> {
        fn whitespace(&mut self) {
            while self
                .bytes
                .get(self.pos)
                .is_some_and(u8::is_ascii_whitespace)
            {
                self.pos += 1;
            }
        }
        fn take(&mut self, ch: u8) -> bool {
            self.whitespace();
            if self.bytes.get(self.pos) == Some(&ch) {
                self.pos += 1;
                true
            } else {
                false
            }
        }
        fn sum(&mut self) -> Result<f32> {
            let mut n = self.product()?;
            loop {
                if self.take(b'+') {
                    n += self.product()?;
                } else if self.take(b'-') {
                    n -= self.product()?;
                } else {
                    return Ok(n);
                }
            }
        }
        fn product(&mut self) -> Result<f32> {
            let mut n = self.atom()?;
            loop {
                if self.take(b'*') {
                    n *= self.atom()?;
                } else if self.take(b'/') {
                    n /= self.atom()?;
                } else {
                    return Ok(n);
                }
            }
        }
        fn atom(&mut self) -> Result<f32> {
            if self.take(b'(') {
                let n = self.sum()?;
                if !self.take(b')') {
                    return Err("unclosed expression".into());
                }
                return Ok(n);
            }
            if self.take(b'-') {
                return Ok(-self.atom()?);
            }
            self.whitespace();
            let start = self.pos;
            while self
                .bytes
                .get(self.pos)
                .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
            {
                self.pos += 1;
            }
            std::str::from_utf8(&self.bytes[start..self.pos])
                .map_err(|e| e.to_string())?
                .parse()
                .map_err(|_| "invalid numeric expression".into())
        }
    }
    let mut parser = Parser {
        bytes: text.as_bytes(),
        pos: 0,
    };
    let n = parser.sum()?;
    parser.whitespace();
    if parser.pos != parser.bytes.len() || !n.is_finite() {
        return Err(format!("invalid numeric token: {text}"));
    }
    Ok(n)
}

fn identifier(path: &str) -> String {
    let mut result = String::from("TOKEN_");
    for ch in path.chars() {
        if ch.is_uppercase() {
            result.push('_');
        }
        result.push(if ch.is_ascii_alphanumeric() {
            ch.to_ascii_uppercase()
        } else {
            '_'
        });
    }
    result
}

pub fn generate(source: &str) -> Result<String> {
    let document: Value = serde_json::from_str(source).map_err(|e| e.to_string())?;
    let mut tokens = BTreeMap::new();
    flatten(&document["Global"], "", &mut tokens);
    flatten(&document["Components"], "components", &mut tokens);
    if tokens.is_empty() {
        return Err("no design tokens".into());
    }
    let mut output = String::new();
    let mut names = BTreeSet::new();
    for (path, token) in &tokens {
        let name = identifier(path);
        if !names.insert(name.clone()) {
            return Err(format!("duplicate generated identifier: {name}"));
        }
        let value = resolve(
            &token["$value"],
            &tokens,
            &mut BTreeSet::from([path.clone()]),
        )?;
        let declaration = match token["$type"].as_str().ok_or("missing token type")? {
            "color" => {
                let hex = value
                    .as_str()
                    .and_then(|s| s.strip_prefix('#'))
                    .ok_or("expected hex color")?;
                if ![6, 8].contains(&hex.len()) {
                    return Err(format!("invalid color: {path}"));
                }
                let color = u32::from_str_radix(hex, 16).map_err(|e| e.to_string())?;
                format!("u32 = 0x{color:08x}")
            }
            "fontFamilies" => {
                let family = value
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(Value::as_str)
                    .ok_or("expected font family list")?;
                // The embedded faces use a private family to avoid system collisions.
                let family = if family == "Montserrat" {
                    "Aetheris Montserrat"
                } else {
                    family
                };
                format!("&str = {family:?}")
            }
            "typography" => continue, // Composite values remain in JSON for interchange.
            "number" | "dimension" | "fontSizes" | "borderRadius" | "fontWeights"
            | "letterSpacing" => {
                format!(
                    "f32 = {:?}",
                    number(&value).map_err(|e| format!("{path}: {e}"))?
                )
            }
            kind => return Err(format!("unsupported token type {kind}: {path}")),
        };
        output.push_str(&format!("pub const {name}: {declaration};\n"));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aliases_and_arithmetic_follow_source_changes() {
        let source = include_str!("../resources/design-tokens.json");
        let generated = generate(source).unwrap();
        assert!(generated.contains("TOKEN_COMPONENTS_FIELD_HEIGHT: f32 = 40.0"));
        assert!(generated.contains("TOKEN_SM: f32 = 14.0"));
        let mut edited: Value = serde_json::from_str(source).unwrap();
        edited["Global"]["size-base"]["$value"] = serde_json::json!(20);
        edited["Global"]["accent"]["$value"] = Value::String("#123456".into());
        let changed = generate(&edited.to_string()).unwrap();
        assert!(changed.contains("TOKEN_SM: f32 = 17.5"));
        assert!(changed.contains("TOKEN_COMPONENTS_BUTTON_PRIMARY_BACKGROUND: u32 = 0x00123456"));
    }
    #[test]
    fn invalid_references_cycles_and_dimensions_fail() {
        for value in ["{missing}", "{x}", "1/0", "hello"] {
            let source = serde_json::json!({"Global":{"x":{"$type":"dimension","$value":value}}});
            assert!(generate(&source.to_string()).is_err(), "{value}");
        }
    }
}
