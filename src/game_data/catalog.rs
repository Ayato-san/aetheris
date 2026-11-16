//! Immutable SQLite catalogues, built in staging and activated only after validation.
use super::{ClientIdentity, Inventory};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub const FORMAT_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    Items,
    Skills,
    Buffs,
    Creatures,
    Pets,
    Recipes,
    Maps,
}
impl Family {
    pub const ALL: [Self; 7] = [
        Self::Items,
        Self::Skills,
        Self::Buffs,
        Self::Creatures,
        Self::Pets,
        Self::Recipes,
        Self::Maps,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Self::Items => "items",
            Self::Skills => "skills",
            Self::Buffs => "buffs",
            Self::Creatures => "creatures",
            Self::Pets => "pets",
            Self::Recipes => "recipes",
            Self::Maps => "maps",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum TableStatus {
    Decoded,
    Absent { evidence: String },
    Rejected { error: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TableReport {
    pub family: Family,
    pub table: String,
    pub source: String,
    pub sha256: String,
    pub schema: String,
    pub status: TableStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub family: Family,
    /// Definition ID only. Network IDs require a separately established mapping.
    pub id: String,
    pub technical_name: String,
    pub table: String,
    pub row: u64,
    pub namespace: String,
    pub translation_key: Option<String>,
    /// Original decoded fields, including unknown fields; no estimated statistics.
    pub fields: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Translation {
    pub locale: String,
    pub namespace: String,
    pub key: String,
    pub text: String,
    pub source: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Relation {
    pub from_family: Family,
    pub from_id: String,
    pub to_family: Family,
    pub to_id: String,
    pub field: String,
    /// Conditions, quantities, coordinates and source array index stay intact.
    pub value: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Build {
    pub format: u32,
    pub inventory: Inventory,
    pub schema_set: String,
    pub compatible_clients: Vec<ClientIdentity>,
    /// Declared by the schema/normalizer, before decoding. A missing required
    /// table cannot be hidden by omitting its report from the attempted build.
    pub required_tables: BTreeMap<Family, BTreeSet<String>>,
    pub tables: Vec<TableReport>,
    pub records: Vec<Record>,
    pub translations: Vec<Translation>,
    pub relations: Vec<Relation>,
}

fn error(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        return Err(io::Error::new(io::ErrorKind::Interrupted, "catalogue build cancelled").into());
    }
    Ok(())
}

impl Build {
    pub fn validate(&self) -> Result<()> {
        if self.format != FORMAT_VERSION || self.schema_set.is_empty() {
            return Err(error("unsupported catalogue or missing schema set").into());
        }
        let client = self.inventory.client.as_ref().ok_or_else(|| {
            error("client version and region must be established before activation")
        })?;
        if !self.compatible_clients.contains(client) {
            return Err(error("schema set incompatible with client").into());
        }
        if !self.inventory.problems.is_empty() {
            return Err(error("incomplete source inventory").into());
        }
        for family in Family::ALL {
            let required = self
                .required_tables
                .get(&family)
                .filter(|tables| !tables.is_empty())
                .ok_or_else(|| error("missing required table declaration"))?;
            for name in required {
                if !self
                    .tables
                    .iter()
                    .any(|t| t.family == family && &t.table == name)
                {
                    return Err(
                        error(&format!("missing required table {}:{name}", family.key())).into(),
                    );
                }
            }
            let tables: Vec<_> = self.tables.iter().filter(|t| t.family == family).collect();
            if tables.is_empty()
                || tables
                    .iter()
                    .any(|t| matches!(t.status, TableStatus::Rejected { .. }))
            {
                return Err(error(&format!("missing or rejected family {}", family.key())).into());
            }
        }
        let mut tables = BTreeMap::new();
        for table in &self.tables {
            if table.table.is_empty()
                || table.schema.is_empty()
                || table.source.is_empty()
                || tables
                    .insert((&table.family, &table.table), table)
                    .is_some()
            {
                return Err(error("invalid or duplicate table provenance").into());
            }
            match &table.status {
                TableStatus::Decoded if hex::decode(&table.sha256).is_ok_and(|b| b.len() == 32) => {
                }
                TableStatus::Absent { evidence } if !evidence.trim().is_empty() => (),
                _ => return Err(error("invalid table status or source hash").into()),
            }
        }
        let mut ids = BTreeSet::new();
        let mut rows = BTreeSet::new();
        for record in &self.records {
            let table = tables
                .get(&(&record.family, &record.table))
                .ok_or_else(|| error("record without table provenance"))?;
            if record.id.is_empty()
                || !record.fields.is_object()
                || table.status != TableStatus::Decoded
                || !ids.insert((record.family, record.id.as_str()))
                || !rows.insert((record.family, &record.table, record.row))
            {
                return Err(error("invalid, duplicate or unproven record").into());
            }
        }
        for table in &self.tables {
            if table.status == TableStatus::Decoded
                && !self
                    .records
                    .iter()
                    .any(|r| r.family == table.family && r.table == table.table)
            {
                return Err(error("empty decoded table must declare confirmed absence").into());
            }
        }
        for relation in &self.relations {
            if relation.field.is_empty()
                || !ids.contains(&(relation.from_family, relation.from_id.as_str()))
                || !ids.contains(&(relation.to_family, relation.to_id.as_str()))
            {
                return Err(error("unresolved catalogue relation").into());
            }
        }
        let mut translations = BTreeSet::new();
        for text in &self.translations {
            if text.locale.is_empty()
                || text.key.is_empty()
                || text.source.is_empty()
                || !translations.insert((&text.locale, &text.namespace, &text.key))
            {
                return Err(error("invalid or duplicate translation").into());
            }
        }
        Ok(())
    }

    /// Stable logical content for cache keys and byte-reproducible imports.
    fn canonical_bytes(&self) -> Result<Vec<u8>> {
        let mut build = self.clone();
        build
            .compatible_clients
            .sort_by(|a, b| (&a.version, &a.region).cmp(&(&b.version, &b.region)));
        build
            .tables
            .sort_by(|a, b| (a.family, &a.table).cmp(&(b.family, &b.table)));
        build
            .records
            .sort_by(|a, b| (a.family, &a.id).cmp(&(b.family, &b.id)));
        build.translations.sort_by(|a, b| {
            (&a.locale, &a.namespace, &a.key).cmp(&(&b.locale, &b.namespace, &b.key))
        });
        build
            .relations
            .sort_by_cached_key(|r| serde_json::to_string(r).unwrap());
        Ok(serde_json::to_vec(&build)?)
    }
}

struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let _ = fs::remove_file(format!("{}-journal", self.0.display()));
    }
}

/// Files in `catalogues/` are immutable. `active.json` is the sole activation
/// point and uses the existing Windows/POSIX atomic replacement boundary.
pub fn publish(build: &Build, source: &Path, store: &Path, cancel: &AtomicBool) -> Result<String> {
    publish_with_progress(build, source, store, cancel, |_| {})
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Validating,
    Records,
    Translations,
    Relations,
    Source,
    Activation,
    Complete,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Progress {
    pub phase: Phase,
    pub completed: usize,
    pub total: usize,
}

#[derive(Serialize)]
struct AttemptReport<'a> {
    client: &'a Option<ClientIdentity>,
    fingerprint: &'a str,
    schema_set: &'a str,
    tables: &'a [TableReport],
    languages: BTreeSet<&'a str>,
    rows_by_table: BTreeMap<String, usize>,
    state: &'a str,
    candidate: Option<&'a str>,
    error: Option<String>,
}

fn report(
    build: &Build,
    path: &Path,
    state: &str,
    candidate: Option<&str>,
    error: Option<String>,
) -> Result<()> {
    let report = AttemptReport {
        client: &build.inventory.client,
        fingerprint: &build.inventory.fingerprint,
        schema_set: &build.schema_set,
        tables: &build.tables,
        languages: build
            .translations
            .iter()
            .map(|t| t.locale.as_str())
            .collect(),
        rows_by_table: build
            .tables
            .iter()
            .map(|t| {
                (
                    format!("{}:{}", t.family.key(), t.table),
                    build
                        .records
                        .iter()
                        .filter(|r| r.family == t.family && r.table == t.table)
                        .count(),
                )
            })
            .collect(),
        state,
        candidate,
        error,
    };
    crate::engine_ini::storage::atomic_replace(path, &serde_json::to_vec_pretty(&report)?, None)?;
    Ok(())
}

pub fn publish_with_progress(
    build: &Build,
    source: &Path,
    store: &Path,
    cancel: &AtomicBool,
    mut progress: impl FnMut(Progress),
) -> Result<String> {
    fs::create_dir_all(store)?;
    let _lease = crate::companion::Lease::acquire(&store.join(".catalogue.lock"))?;
    cleanup_staging(store)?;
    let mut random = [0; 16];
    getrandom::fill(&mut random).map_err(|e| io::Error::other(e.to_string()))?;
    fs::create_dir_all(store.join("reports"))?;
    let path = store
        .join("reports")
        .join(format!("{}.json", hex::encode(random)));
    report(build, &path, "started", None, None)?;
    let result = publish_inner(build, source, store, cancel, &mut progress, &path);
    match &result {
        Ok(key) => {
            // Activation is the commit point. If the final report write is
            // interrupted, the earlier validated report and active pointer
            // still establish what was published. Never report a failure after
            // changing the active catalogue successfully.
            let _ = report(build, &path, "activated", Some(key), None);
            progress(Progress {
                phase: Phase::Complete,
                completed: 1,
                total: 1,
            });
        }
        Err(error) => {
            let state = if cancel.load(Ordering::Acquire) {
                "cancelled"
            } else {
                "rejected"
            };
            if let Err(report_error) = report(build, &path, state, None, Some(error.to_string())) {
                return Err(io::Error::other(format!(
                    "{error}; cannot finish local report: {report_error}"
                ))
                .into());
            }
        }
    }
    result
}

fn publish_inner(
    build: &Build,
    source: &Path,
    store: &Path,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
    report_path: &Path,
) -> Result<String> {
    progress(Progress {
        phase: Phase::Validating,
        completed: 0,
        total: 1,
    });
    check_cancel(cancel)?;
    build.validate()?;
    build.inventory.verify_unchanged(source)?;
    let canonical = build.canonical_bytes()?;
    let key = hex::encode(Sha256::digest(&canonical));
    let directory = store.join("catalogues");
    fs::create_dir_all(&directory)?;
    let destination = directory.join(format!("{key}.sqlite3"));
    let mut random = [0; 16];
    getrandom::fill(&mut random).map_err(|e| std::io::Error::other(e.to_string()))?;
    let staging =
        Staging(directory.join(format!(".aetheris-catalogue-{}.tmp", hex::encode(random))));
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging.0)?;
    {
        let mut connection = Connection::open(&staging.0)?;
        connection.execute_batch("PRAGMA foreign_keys=ON;
            CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE records (family TEXT NOT NULL, id TEXT NOT NULL, payload TEXT NOT NULL, PRIMARY KEY(family,id));
            CREATE TABLE translations (locale TEXT NOT NULL, namespace TEXT NOT NULL, key TEXT NOT NULL, text TEXT NOT NULL, source TEXT NOT NULL, PRIMARY KEY(locale,namespace,key));
            CREATE TABLE relations (position INTEGER PRIMARY KEY, from_family TEXT NOT NULL, from_id TEXT NOT NULL, to_family TEXT NOT NULL, to_id TEXT NOT NULL, field TEXT NOT NULL, value TEXT NOT NULL, FOREIGN KEY(from_family,from_id) REFERENCES records(family,id), FOREIGN KEY(to_family,to_id) REFERENCES records(family,id));")?;
        let transaction = connection.transaction()?;
        // Keep the complete manifest in the database so rows and provenance are
        // activated together, without a sidecar consistency window.
        transaction.execute(
            "INSERT INTO metadata VALUES ('build', ?1)",
            [std::str::from_utf8(&canonical)?],
        )?;
        let canonical_build: Build = serde_json::from_slice(&canonical)?;
        for (index, record) in canonical_build.records.iter().enumerate() {
            progress(Progress {
                phase: Phase::Records,
                completed: index,
                total: canonical_build.records.len(),
            });
            check_cancel(cancel)?;
            transaction.execute(
                "INSERT INTO records VALUES (?1,?2,?3)",
                params![
                    record.family.key(),
                    record.id,
                    serde_json::to_string(record)?
                ],
            )?;
        }
        for (index, text) in canonical_build.translations.iter().enumerate() {
            progress(Progress {
                phase: Phase::Translations,
                completed: index,
                total: canonical_build.translations.len(),
            });
            check_cancel(cancel)?;
            transaction.execute(
                "INSERT INTO translations VALUES (?1,?2,?3,?4,?5)",
                params![
                    text.locale,
                    text.namespace,
                    text.key,
                    text.text,
                    text.source
                ],
            )?;
        }
        for (index, relation) in canonical_build.relations.iter().enumerate() {
            progress(Progress {
                phase: Phase::Relations,
                completed: index,
                total: canonical_build.relations.len(),
            });
            check_cancel(cancel)?;
            transaction.execute(
                "INSERT INTO relations VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![
                    index as i64,
                    relation.from_family.key(),
                    relation.from_id,
                    relation.to_family.key(),
                    relation.to_id,
                    relation.field,
                    serde_json::to_string(&relation.value)?
                ],
            )?;
        }
        transaction.commit()?;
        let integrity: String = connection.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        if integrity != "ok" {
            return Err(error("catalogue integrity check failed").into());
        }
    }
    match fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&staging.0)?
        .sync_all()
    {
        Err(e)
            if e.kind() == io::ErrorKind::Unsupported
                || (cfg!(windows) && e.raw_os_error() == Some(50)) => {}
        result => result?,
    }
    check_cancel(cancel)?;
    progress(Progress {
        phase: Phase::Source,
        completed: 0,
        total: 1,
    });
    build.inventory.verify_unchanged(source)?;
    // A cache hit must still validate the actual file, never trust its filename.
    if destination.exists() {
        if crate::companion::manifest::hash(&destination)?
            != crate::companion::manifest::hash(&staging.0)?
        {
            return Err(error("corrupt catalogue cache").into());
        }
    } else {
        fs::rename(&staging.0, &destination)?;
    }
    check_cancel(cancel)?;
    let hash = crate::companion::manifest::hash(&destination)?;
    let pointer = serde_json::to_vec(&Pointer {
        key: key.clone(),
        sha256: hash,
    })?;
    progress(Progress {
        phase: Phase::Activation,
        completed: 0,
        total: 1,
    });
    check_cancel(cancel)?;
    build.inventory.verify_unchanged(source)?;
    report(build, report_path, "validated", Some(&key), None)?;
    check_cancel(cancel)?;
    crate::engine_ini::storage::atomic_replace(&store.join("active.json"), &pointer, None)?;
    Ok(key)
}

fn cleanup_staging(store: &Path) -> Result<()> {
    let directory = store.join("catalogues");
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let name = name.strip_suffix("-journal").unwrap_or(name);
        if name
            .strip_prefix(".aetheris-catalogue-")
            .and_then(|s| s.strip_suffix(".tmp"))
            .is_some_and(|s| s.len() == 32 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

/// Removes only intermediate cache entries. Immutable catalogues, active
/// pointers and reports live outside this directory and remain untouched.
pub fn purge_cache(store: &Path) -> Result<()> {
    fs::create_dir_all(store)?;
    let _lease = crate::companion::Lease::acquire(&store.join(".catalogue.lock"))?;
    let cache = store.join("cache");
    if cache.exists() {
        fs::remove_dir_all(cache)?;
    }
    cleanup_staging(store)
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Pointer {
    key: String,
    sha256: String,
}

pub struct Catalog {
    connection: Connection,
    pub build: Build,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NameSource {
    SelectedLocale,
    English,
    Technical,
    Identifier,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ResolvedName {
    pub text: String,
    pub source: NameSource,
    pub stale: bool,
}

impl Catalog {
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        let bytes: String =
            connection.query_row("SELECT value FROM metadata WHERE key='build'", [], |r| {
                r.get(0)
            })?;
        let build: Build = serde_json::from_str(&bytes)?;
        build.validate()?;
        Ok(Self { connection, build })
    }

    pub fn active(store: &Path) -> Result<Option<Self>> {
        let bytes = match fs::read(store.join("active.json")) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let pointer: Pointer = serde_json::from_slice(&bytes)?;
        if pointer.key.len() != 64 || !pointer.key.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(error("invalid catalogue pointer").into());
        }
        let path = store
            .join("catalogues")
            .join(format!("{}.sqlite3", pointer.key));
        if crate::companion::manifest::hash(&path)? != pointer.sha256 {
            return Err(error("catalogue checksum mismatch").into());
        }
        let catalog = Self::open(&path)?;
        if hex::encode(Sha256::digest(catalog.build.canonical_bytes()?)) != pointer.key {
            return Err(error("catalogue content mismatch").into());
        }
        Ok(Some(catalog))
    }

    pub fn stale(&self, inventory: Option<&Inventory>) -> bool {
        inventory.is_none_or(|i| i.fingerprint != self.build.inventory.fingerprint)
    }

    pub fn record(&self, family: Family, id: &str) -> Result<Option<Record>> {
        let payload: Option<String> = self
            .connection
            .query_row(
                "SELECT payload FROM records WHERE family=?1 AND id=?2",
                params![family.key(), id],
                |r| r.get(0),
            )
            .optional()?;
        payload
            .map(|s| serde_json::from_str(&s).map_err(Into::into))
            .transpose()
    }

    pub fn name(
        &self,
        family: Family,
        id: &str,
        locale: &str,
        inventory: Option<&Inventory>,
    ) -> Result<ResolvedName> {
        let stale = self.stale(inventory);
        if let Some(record) = self.record(family, id)? {
            if let Some(key) = &record.translation_key {
                for (locale, source) in [
                    (locale, NameSource::SelectedLocale),
                    ("en-US", NameSource::English),
                ] {
                    let text: Option<String> = self.connection.query_row("SELECT text FROM translations WHERE locale=?1 AND namespace=?2 AND key=?3 AND length(trim(text))>0", params![locale,record.namespace,key], |r| r.get(0)).optional()?;
                    if let Some(text) = text {
                        return Ok(ResolvedName {
                            text,
                            source,
                            stale,
                        });
                    }
                }
            }
            if !record.technical_name.trim().is_empty() {
                return Ok(ResolvedName {
                    text: record.technical_name,
                    source: NameSource::Technical,
                    stale,
                });
            }
        }
        Ok(ResolvedName {
            text: id.into(),
            source: NameSource::Identifier,
            stale,
        })
    }
}

/// Local catalogue wins only for the matching source; otherwise prefer a
/// supplied matching catalogue, retaining a stale catalogue as a last resort.
pub fn select<'a>(
    local: Option<&'a Catalog>,
    supplied: Option<&'a Catalog>,
    inventory: Option<&Inventory>,
) -> Option<&'a Catalog> {
    local
        .filter(|c| !c.stale(inventory))
        .or_else(|| supplied.filter(|c| !c.stale(inventory)))
        .or(local)
        .or(supplied)
}
