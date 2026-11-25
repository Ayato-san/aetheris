//! Shared source provenance, typed references, texts and fingerprints.
use super::*;

pub(super) fn rows<'a>(batch: &'a BTreeMap<String, TableProbe>, source: &str) -> &'a [Value] {
    batch
        .get(source)
        .map(|t| t.layout.rows.as_slice())
        .unwrap_or(&[])
}
pub(super) fn list<'a>(row: &'a Value, field: &str) -> &'a [Value] {
    row[field].as_array().map(Vec::as_slice).unwrap_or(&[])
}
pub(super) fn id(row: &Value) -> Result<i64> {
    number(row, "ID")
        .filter(|id| *id > 0)
        .ok_or_else(|| invalid("missing positive recipe/quest source ID").into())
}
fn key(value: &Value) -> Option<String> {
    let value = value
        .as_object()
        .filter(|v| v.len() == 1)
        .and_then(|v| v.get("Value"))
        .unwrap_or(value);
    match value {
        Value::String(s) if !s.is_empty() && s != "None" => Some(format!("s:{s}")),
        Value::Number(n) if n.as_i64().is_some_and(|n| n != 0) => Some(format!("n:{n}")),
        _ => None,
    }
}
pub(super) fn text_keys(value: &Value, path: &str, output: &mut Vec<(String, String)>) {
    match value {
        Value::Object(o) => {
            if !o.contains_key("Value")
                && let Some(k) = string(&value["Key"])
            {
                output.push((path.to_owned(), k.to_owned()));
            }
            for (field, child) in o {
                text_keys(child, &format!("{path}/{field}"), output);
            }
        }
        Value::Array(a) => {
            for (pos, child) in a.iter().enumerate() {
                text_keys(child, &format!("{path}/{pos}"), output);
            }
        }
        _ => {}
    }
}
pub(super) struct Origin<'a> {
    pub(super) family: &'a str,
    pub(super) id: i64,
    pub(super) source: &'a str,
    pub(super) row: usize,
}
type ReferenceIndex = BTreeMap<(String, String), BTreeMap<String, Vec<usize>>>;
type EntryEvidence = Vec<(String, String, String)>;
pub(super) struct Projection<'a> {
    pub(super) db: &'a Connection,
    pub(super) batch: &'a BTreeMap<String, TableProbe>,
    pub(super) en: &'a Localization,
    pub(super) loc: &'a Localization,
    indexes: ReferenceIndex,
    labels: BTreeMap<String, Vec<usize>>,
    pub(super) items: BTreeMap<&'a str, Option<i64>>,
    pub(super) build: i64,
}
impl<'a> Projection<'a> {
    pub(super) fn new(
        db: &'a Connection,
        batch: &'a BTreeMap<String, TableProbe>,
        en: &'a Localization,
        loc: &'a Localization,
        build: i64,
    ) -> Self {
        let mut items = BTreeMap::new();
        for row in rows(batch, "Item") {
            if let (Some(name), Some(id)) = (string(&row["Name"]), number(row, "ID")) {
                items
                    .entry(name)
                    .and_modify(|v| *v = None)
                    .or_insert(Some(id));
            }
        }
        let mut labels: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (pos, row) in rows(batch, "EnumString").iter().enumerate() {
            if let (Some(ty), Some(code)) = (string(&row["Enum"]), string(&row["Value"])) {
                labels.entry(format!("{ty}::{code}")).or_default().push(pos);
            }
        }
        Self {
            db,
            batch,
            en,
            loc,
            build,
            indexes: BTreeMap::new(),
            labels,
            items,
        }
    }

    pub(super) fn source(&self, family: &str, id: i64, source: &str, row: usize) -> Result<()> {
        self.db.execute(
            "INSERT OR IGNORE INTO catalogue_entity_sources VALUES(?1,?2,?3,?4)",
            params![family, id, source, row as i64],
        )?;
        Ok(())
    }
    pub(super) fn code(&self, value: &Value) -> Result<Option<i64>> {
        let Some(code) = string(value) else {
            return Ok(None);
        };
        let id = dictionary(self.db, "catalogue_codes", code)?;
        if let Some((ty, code)) = code.split_once("::") {
            self.db.execute("UPDATE catalogue_codes SET name_en=(SELECT name_en FROM catalogue_labels WHERE enum_type=?1 AND code=?2),name_localized=(SELECT name_localized FROM catalogue_labels WHERE enum_type=?1 AND code=?2) WHERE id=?3",params![ty,code,id])?;
        }
        Ok(Some(id))
    }
    pub(super) fn race(&self, row: &Value, field: &str) -> Result<Option<i64>> {
        let Some(code) = string(&row[field]).and_then(|s| s.strip_prefix("ERace::")) else {
            return Ok(None);
        };
        Ok(self
            .db
            .query_row("SELECT id FROM races WHERE code=?1", [code], |r| r.get(0))
            .optional()?)
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn link(
        &mut self,
        origin: &Origin<'_>,
        path: &str,
        value: &Value,
        target: (&str, &str),
        candidate: bool,
        dependency: bool,
    ) -> Result<Option<usize>> {
        Ok(self
            .links(origin, path, value, target, candidate, dependency, false)?
            .first()
            .copied())
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn group(
        &mut self,
        origin: &Origin<'_>,
        path: &str,
        value: &Value,
        target: (&str, &str),
        candidate: bool,
        dependency: bool,
    ) -> Result<Vec<usize>> {
        self.links(origin, path, value, target, candidate, dependency, true)
    }
    #[allow(clippy::too_many_arguments)]
    fn links(
        &mut self,
        origin: &Origin<'_>,
        path: &str,
        value: &Value,
        target: (&str, &str),
        candidate: bool,
        dependency: bool,
        group: bool,
    ) -> Result<Vec<usize>> {
        let Some(k) = key(value) else {
            return Ok(Vec::new());
        };
        let index_key = (target.0.to_owned(), target.1.to_owned());
        if !self.indexes.contains_key(&index_key) {
            let mut index: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for (pos, row) in rows(self.batch, target.0).iter().enumerate() {
                if let Some(k) = key(&row[target.1]) {
                    index.entry(k).or_default().push(pos);
                }
            }
            self.indexes.insert(index_key.clone(), index);
        }
        let matches = self.indexes[&index_key]
            .get(&k)
            .cloned()
            .unwrap_or_default();
        let state = match matches.as_slice() {
            [_] => "resolved",
            [] if !self.batch.contains_key(target.0) => "not_researched",
            [] => "missing",
            _ if group => "group",
            _ => "ambiguous",
        };
        let target_row = if matches.len() == 1 {
            Some(matches[0] as i64)
        } else {
            None
        };
        self.db.execute(
            "INSERT INTO catalogue_references VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                origin.family,
                origin.id,
                origin.source,
                origin.row as i64,
                path,
                target.0,
                target.1,
                &k[2..],
                target_row,
                state,
                if candidate { "candidate" } else { "verified" }
            ],
        )?;
        if !matches.is_empty() && (matches.len() == 1 || group) {
            if dependency {
                for pos in &matches {
                    self.source(origin.family, origin.id, target.0, *pos)?;
                }
            }
            Ok(matches)
        } else {
            Ok(Vec::new())
        }
    }
    pub(super) fn named_text(
        &self,
        origin: &Origin<'_>,
        field: &str,
        key: Option<&str>,
    ) -> Result<(Option<String>, Option<String>)> {
        super::projection::text_state(
            self.db,
            origin.family,
            origin.id,
            field,
            key,
            self.en,
            self.loc,
        )?;
        Ok((text(self.en, key), text(self.loc, key)))
    }
    pub(super) fn codes(&self, origin: &Origin<'_>, row: &Value, fields: &[&str]) -> Result<()> {
        for field in fields {
            if let Some(positions) = string(&row[*field]).and_then(|s| self.labels.get(s)) {
                for pos in positions {
                    self.source(origin.family, origin.id, "EnumString", *pos)?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn condition(
        &self,
        origin: &Origin<'_>,
        path: &str,
        kind: &Value,
        value: &Value,
    ) -> Result<()> {
        self.db.execute(
            "INSERT INTO catalogue_conditions VALUES(?1,?2,?3,?4,?5,?6,?7,'uninterpreted')",
            params![
                origin.family,
                origin.id,
                origin.source,
                origin.row as i64,
                path,
                self.code(kind)?,
                serde_json::to_string(value)?
            ],
        )?;
        Ok(())
    }
    pub(super) fn texts(&self, origin: &Origin<'_>, row: &Value) -> Result<Vec<(String, String)>> {
        let mut keys = Vec::new();
        text_keys(row, "", &mut keys);
        for (path, key) in &keys {
            super::projection::text_state(
                self.db,
                origin.family,
                origin.id,
                path,
                Some(key),
                self.en,
                self.loc,
            )?;
        }
        Ok(keys)
    }
    pub(super) fn item(&self, reference: Option<&str>) -> Option<i64> {
        reference.and_then(|s| self.items.get(s).copied().flatten())
    }
}

pub(super) type Fingerprints = BTreeMap<(String, i64), (String, String)>;
pub(super) fn fingerprints(
    db: &Connection,
    batch: &BTreeMap<String, TableProbe>,
    en: &Localization,
    loc: &Localization,
    families: &[&str],
) -> Result<Fingerprints> {
    let mut evidence: BTreeMap<(String, i64), EntryEvidence> = BTreeMap::new();
    for result in db.prepare("SELECT family,source_id,source_name,source_row FROM catalogue_entity_sources ORDER BY family,source_id,source_name,source_row")?.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?)))? {
        let (family,id,source,pos)=result?;
        if !families.contains(&family.as_str()) {continue;}
        let row=&batch[&source].layout.rows[usize::try_from(pos)?];
        let mut keys=Vec::new();text_keys(row,"",&mut keys);
        let english:Vec<_>=keys.iter().map(|(path,key)|(path,key,text(en,Some(key)))).collect();
        let localized:Vec<_>=keys.iter().map(|(path,key)|(path,key,text(loc,Some(key)))).collect();
        evidence.entry((family,id)).or_default().push((hash(&serde_json::to_vec(&(source,row))?),hash(&serde_json::to_vec(&english)?),hash(&serde_json::to_vec(&localized)?)));
    }
    let mut result = BTreeMap::new();
    for (key, mut parts) in evidence {
        parts.sort();
        let english: Vec<_> = parts.iter().map(|(source, en, _)| (source, en)).collect();
        let localized: Vec<_> = parts.iter().map(|(_, _, loc)| loc).collect();
        result.insert(
            key,
            (
                hash(&serde_json::to_vec(&english)?),
                hash(&serde_json::to_vec(&localized)?),
            ),
        );
    }
    Ok(result)
}
