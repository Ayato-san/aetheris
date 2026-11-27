//! Reviewed decoding contracts. Observing a layout never grants compatibility.
//! Inputs are explicitly decrypted payloads, even when their first word is 13.
use super::{
    ClientIdentity,
    assets::{self, Decryption},
    binary::invalid,
    reader::{Control, Phase},
    schema::Schema,
    usmap::Mapping,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::Path,
};

const MAX_FILE: usize = 64 * 1024 * 1024;
const MAX_REPORT: usize = 16 * 1024 * 1024;
pub const PROTECTION: &str = "aion2-kiskemu-versioned-permutation-v1";

/// Initial reviewed contract; mapping bytes and client keys remain local inputs.
/// Updated contracts may also arrive through the signed resources mechanism.
pub fn bundled(client: &ClientIdentity) -> io::Result<SchemaSet> {
    let set = SchemaSet::parse(include_bytes!(
        "../../resources/game-data/steam-26581260-global.schemas.json"
    ))?;
    set.require_client(client)?;
    Ok(set)
}

/// Fingerprint the implementation actually built, including local patches.
pub fn reader_revision() -> String {
    let mut hash = Sha256::new();
    for source in [
        include_bytes!("schema.rs").as_slice(),
        include_bytes!("usmap.rs").as_slice(),
        include_bytes!("binary.rs").as_slice(),
        include_bytes!("assets.rs").as_slice(),
        include_bytes!("protection.rs").as_slice(),
        include_bytes!("schema_set.rs").as_slice(),
    ] {
        hash.update((source.len() as u64).to_le_bytes());
        hash.update(source);
    }
    hex::encode(hash.finalize())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaSet {
    pub format: u32,
    pub id: String,
    pub mapping_sha256: String,
    pub reader_revision: String,
    pub protection: String,
    /// Cryptographic context, distinct from the client build/region declaration.
    pub app_version: Option<String>,
    pub compatible_clients: Vec<ClientIdentity>,
    pub entries: BTreeMap<String, Entry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Entry {
    Table {
        row_struct: String,
        table_version: i32,
        column_hashes: Vec<String>,
    },
    Localization {
        namespace: String,
        version: u32,
    },
}

/// Maintainer selection, before observing payloads. Paths are original .dat
/// virtual paths from decryption.json, never guessed from a table's first word.
#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selection {
    Table { row_struct: String },
    Localization,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputStage {
    Container,
    Decrypted,
    Decoded,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum Data {
    Table(Vec<serde_json::Value>),
    Localization(assets::Localization),
}

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':', '\0'])
        && path
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
        && path.ends_with(".dat")
}
pub fn read_bounded(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(invalid("schema input exceeds size limit"));
    }
    Ok(bytes)
}

impl SchemaSet {
    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() > MAX_REPORT {
            return Err(invalid("schema set exceeds size limit"));
        }
        let set: Self = serde_json::from_slice(bytes)?;
        set.validate()?;
        Ok(set)
    }

    pub fn validate(&self) -> io::Result<()> {
        if self.format != 1
            || self.id.trim().is_empty()
            || !valid_hash(&self.mapping_sha256)
            || self.reader_revision != reader_revision()
            || self.protection != PROTECTION
            || self.entries.is_empty()
            || self.entries.len() > 4096
            || self
                .app_version
                .as_ref()
                .is_some_and(|v| v.trim().is_empty())
        {
            return Err(invalid(
                "unsupported schema set, mapping or reader/protection revision",
            ));
        }
        let mut clients = BTreeSet::new();
        for client in &self.compatible_clients {
            if client.version.trim().is_empty()
                || client.region.trim().is_empty()
                || client.region.eq_ignore_ascii_case("unknown")
                || !clients.insert((&client.version, &client.region))
            {
                return Err(invalid("invalid or duplicate compatible client identity"));
            }
        }
        let mut paths = BTreeSet::new();
        for (path, entry) in &self.entries {
            if !valid_path(path) || !paths.insert(path.to_lowercase()) {
                return Err(invalid("unsafe or duplicate schema entry path"));
            }
            match entry {
                Entry::Table {
                    row_struct,
                    table_version,
                    column_hashes,
                } => {
                    if row_struct.trim().is_empty()
                        || column_hashes.len() > 100_000
                        || column_hashes
                            .iter()
                            .any(|h| h.is_empty() || h.contains([',', '\0']))
                        || (row_struct.eq_ignore_ascii_case("ItemTableItem")
                            && *table_version != 13)
                    {
                        return Err(invalid("invalid table schema declaration"));
                    }
                }
                Entry::Localization { version, .. } if *version != 1 => {
                    return Err(invalid("unsupported localization schema version"));
                }
                _ => (),
            }
        }
        Ok(())
    }

    pub fn require_client(&self, client: &ClientIdentity) -> io::Result<()> {
        self.validate()?;
        if !self.compatible_clients.contains(client) {
            return Err(invalid(
                "schema set does not declare this client version and region compatible",
            ));
        }
        Ok(())
    }

    pub fn validate_mapping(&self, mapping: &Mapping) -> io::Result<()> {
        self.validate()?;
        if mapping.sha256() != self.mapping_sha256 {
            return Err(invalid("schema set mapping checksum mismatch"));
        }
        for entry in self.entries.values() {
            if let Entry::Table {
                row_struct,
                table_version,
                column_hashes,
            } = entry
            {
                Schema {
                    id: self.id.clone(),
                    compatible_clients: vec![],
                    table_version: *table_version,
                    column_hashes: column_hashes.clone(),
                    fields: mapping.table_fields(row_struct)?,
                }
                .validate_layout()?;
            }
        }
        Ok(())
    }

    /// Explicit input state prevents double decryption and ambiguous probing.
    /// The optional client is mandatory for a production-compatible decode.
    pub fn decode(
        &self,
        mapping: &Mapping,
        path: &str,
        stage: InputStage,
        payload: &[u8],
        client: Option<&ClientIdentity>,
    ) -> io::Result<Data> {
        self.validate()?;
        if let Some(client) = client {
            self.require_client(client)?;
        }
        if stage != InputStage::Decrypted {
            return Err(invalid(
                "schema decoder requires an explicitly decrypted payload",
            ));
        }
        if mapping.sha256() != self.mapping_sha256 {
            return Err(invalid("schema set mapping checksum mismatch"));
        }
        if payload.len() > MAX_FILE {
            return Err(invalid("schema payload exceeds limit"));
        }
        match self
            .entries
            .get(path)
            .ok_or_else(|| invalid("undeclared schema entry"))?
        {
            Entry::Table {
                row_struct,
                table_version,
                column_hashes,
            } => {
                let schema = Schema {
                    id: self.id.clone(),
                    compatible_clients: vec![],
                    table_version: *table_version,
                    column_hashes: column_hashes.clone(),
                    fields: mapping.table_fields(row_struct)?,
                };
                Ok(Data::Table(schema.decode_payload(payload)?))
            }
            Entry::Localization { namespace, .. } => {
                let data = assets::localization(payload)?;
                if data.namespace != *namespace {
                    return Err(invalid("localization namespace mismatch"));
                }
                Ok(Data::Localization(data))
            }
        }
    }
}

struct Prepared {
    root: std::path::PathBuf,
    report: Decryption,
}
impl Prepared {
    fn open(root: &Path) -> io::Result<Self> {
        let root = root.canonicalize()?;
        let report: Decryption =
            serde_json::from_slice(&read_bounded(&root.join("decryption.json"), MAX_REPORT)?)?;
        if report.payload_stage.as_deref() != Some("decrypted")
            || report.protection.as_deref() != Some(PROTECTION)
            || report.reader_revision.as_deref() != Some(reader_revision().as_str())
        {
            return Err(invalid(
                "prepared input state or reader/protection revision is not established",
            ));
        }
        let mut unique = BTreeSet::new();
        if report.files.len() > 100_000
            || report.files.iter().any(|f| {
                !valid_path(&f.path)
                    || !unique.insert(f.path.to_lowercase())
                    || !valid_hash(&f.output_sha256)
                    || !valid_hash(&f.source_sha256)
            })
        {
            return Err(invalid("invalid prepared payload provenance"));
        }
        Ok(Self { root, report })
    }
    fn read(&self, path: &str, table: bool) -> io::Result<Vec<u8>> {
        let asset = self
            .report
            .files
            .iter()
            .find(|f| f.path == path)
            .ok_or_else(|| invalid("declared payload missing from decryption report"))?;
        if asset.family != if table { "table" } else { "localization" } {
            return Err(invalid("prepared payload family does not match schema"));
        }
        let destination = self
            .root
            .join(Path::new(path).with_extension("bin"))
            .canonicalize()?;
        if !destination.starts_with(&self.root) {
            return Err(invalid("prepared payload resolves outside source"));
        }
        let bytes = read_bounded(&destination, MAX_FILE)?;
        if bytes.len() != asset.bytes || hash(&bytes) != asset.output_sha256 {
            return Err(invalid("prepared payload checksum or size mismatch"));
        }
        Ok(bytes)
    }
}

/// Create a candidate only. Compatibility must subsequently be reviewed; it
/// cannot be inferred from payload hashes or successful layout consumption.
pub fn draft(
    mapping: &Mapping,
    prepared: &Path,
    id: &str,
    selection: &BTreeMap<String, Selection>,
    control: &mut Control<'_>,
) -> io::Result<SchemaSet> {
    if selection.is_empty() || selection.len() > 4096 || selection.keys().any(|p| !valid_path(p)) {
        return Err(invalid("invalid schema selection"));
    }
    let source = Prepared::open(prepared)?;
    let mut set = SchemaSet {
        format: 1,
        id: id.into(),
        mapping_sha256: mapping.sha256().into(),
        reader_revision: reader_revision(),
        protection: PROTECTION.into(),
        app_version: source.report.app_version.clone(),
        compatible_clients: vec![],
        entries: BTreeMap::new(),
    };
    for (index, (path, entry)) in selection.iter().enumerate() {
        control.emit(Phase::Decode, index, selection.len(), Some(path))?;
        let payload = source.read(path, matches!(entry, Selection::Table { .. }))?;
        let entry = match entry {
            Selection::Table { row_struct } => {
                let probe = mapping.probe_table(row_struct, &payload)?;
                Entry::Table {
                    row_struct: row_struct.clone(),
                    table_version: probe.layout.table_version,
                    column_hashes: probe.layout.column_hashes,
                }
            }
            Selection::Localization => {
                let data = assets::localization(&payload)?;
                Entry::Localization {
                    namespace: data.namespace,
                    version: 1,
                }
            }
        };
        set.entries.insert(path.clone(), entry);
    }
    control.check()?;
    set.validate()?;
    Ok(set)
}

#[derive(Serialize)]
pub struct EntryReport {
    pub path: String,
    pub payload_sha256: Option<String>,
    pub rows: Option<usize>,
    pub error: Option<String>,
}
#[derive(Serialize)]
pub struct Report {
    pub schema_set: String,
    pub schema_set_sha256: String,
    pub mapping_sha256: String,
    pub reader_revision: String,
    pub app_version: Option<String>,
    pub client: Option<ClientIdentity>,
    pub compatibility_verified: bool,
    pub state: String,
    pub entries: Vec<EntryReport>,
    /// Upstream errors stay visible, including unsupported optional families.
    pub preparation_errors: Vec<String>,
}
impl Report {
    pub fn require_activation(&self, client: &ClientIdentity) -> io::Result<()> {
        if !self.compatibility_verified
            || self.state != "complete"
            || self.client.as_ref() != Some(client)
        {
            return Err(invalid(
                "unverified or incomplete decoded set cannot be activated",
            ));
        }
        Ok(())
    }
}

/// Stage immutable JSON results outside sources. Missing/rejected declared
/// entries are reported individually; no partly decoded table is written.
/// This prepares inputs for the normalizer; it never activates SQLite itself.
pub fn decode_prepared(
    set: &SchemaSet,
    mapping: &Mapping,
    prepared: &Path,
    client: Option<&ClientIdentity>,
    output: &Path,
    control: &mut Control<'_>,
) -> io::Result<Report> {
    control.check()?;
    set.validate()?;
    if let Some(client) = client {
        set.require_client(client)?;
    }
    if set.mapping_sha256 != mapping.sha256() {
        return Err(invalid("schema set mapping checksum mismatch"));
    }
    let source = Prepared::open(prepared)?;
    if source.report.app_version != set.app_version {
        return Err(invalid(
            "prepared protection context does not match schema set",
        ));
    }
    if output.exists()
        || output
            .components()
            .any(|p| matches!(p, std::path::Component::ParentDir))
    {
        return Err(invalid(
            "decoded set requires a fresh output directory without parent traversal",
        ));
    }
    // Reject overlap before creating anything, including existing ancestors.
    let output = super::archives::output_directory(&source.root, output)?;
    let mut report = Report {
        schema_set: set.id.clone(),
        schema_set_sha256: hash(&serde_json::to_vec(set)?),
        mapping_sha256: mapping.sha256().into(),
        reader_revision: reader_revision(),
        app_version: set.app_version.clone(),
        client: client.cloned(),
        compatibility_verified: false,
        state: "incomplete".into(),
        entries: vec![],
        preparation_errors: source.report.errors.clone(),
    };
    let write = |report: &Report| {
        crate::engine_ini::storage::atomic_replace(
            &output.join("decoding.json"),
            &serde_json::to_vec_pretty(report)?,
            None,
        )
    };
    write(&report)?;
    for (index, (path, entry)) in set.entries.iter().enumerate() {
        if let Err(error) = control.emit(Phase::Decode, index, set.entries.len(), Some(path)) {
            report.state = "cancelled".into();
            write(&report)?;
            return Err(error);
        }
        let result = (|| -> io::Result<(String, usize)> {
            let payload = source.read(path, matches!(entry, Entry::Table { .. }))?;
            let data = set.decode(mapping, path, InputStage::Decrypted, &payload, client)?;
            let rows = match &data {
                Data::Table(rows) => rows.len(),
                Data::Localization(data) => data.entries.len(),
            };
            let destination =
                super::archives::destination(&output, &Path::new(path).with_extension("json"))?;
            crate::engine_ini::storage::atomic_replace(
                &destination,
                &serde_json::to_vec(&data)?,
                None,
            )?;
            Ok((hash(&payload), rows))
        })();
        report.entries.push(match result {
            Ok((hash, rows)) => EntryReport {
                path: path.clone(),
                payload_sha256: Some(hash),
                rows: Some(rows),
                error: None,
            },
            Err(error) => EntryReport {
                path: path.clone(),
                payload_sha256: None,
                rows: None,
                error: Some(error.to_string()),
            },
        });
    }
    if let Err(error) = control.emit(Phase::Decode, set.entries.len(), set.entries.len(), None) {
        report.state = "cancelled".into();
        write(&report)?;
        return Err(error);
    }
    report.state = if report.entries.iter().all(|e| e.error.is_none()) {
        "complete"
    } else {
        "partial"
    }
    .into();
    report.compatibility_verified = client.is_some() && report.state == "complete";
    write(&report)?;
    Ok(report)
}
