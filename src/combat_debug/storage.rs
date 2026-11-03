//! SQLite owns immutable capture evidence and replaceable analysis results.
use super::{
    DECODER_VERSION, Result,
    analysis::{self, AnalysisOptions, AnalysisRow, ByteProvenance, DecodedSection},
    pcapng::{CapturedPacket, LinkType, PacketTimestamp},
    source::{CaptureSource, PcapngSource},
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Held for the whole debug-page lifetime. The OS releases the lease on a
/// crash, so startup recovery cannot interfere with another running instance.
pub struct DatabaseLease {
    _file: std::fs::File,
}

impl DatabaseLease {
    pub fn acquire(path: &Path) -> Result<Self> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        let mut name = path.as_os_str().to_owned();
        name.push(".lock");
        let mut options = std::fs::OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(0);
        }
        let file=options.open(PathBuf::from(name))
            .map_err(|error|format!("Accès exclusif à la base de capture indisponible (autre instance ou permissions) : {error}"))?;
        #[cfg(unix)]
        {
            use std::os::{fd::AsRawFd, raw::c_int};
            unsafe extern "C" {
                fn flock(fd: c_int, operation: c_int) -> c_int;
            }
            // LOCK_EX | LOCK_NB. The owned File releases the lock when closed.
            if unsafe { flock(file.as_raw_fd(), 6) } != 0 {
                return Err(
                    "La base de capture est déjà utilisée par une autre instance d’Aetheris."
                        .into(),
                );
            }
        }
        Ok(Self { _file: file })
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub id: i64,
    pub name: String,
    pub source: String,
    pub started_at: i64,
    pub started_at_display: String,
    pub status: String,
    pub packet_count: usize,
    pub row_count: usize,
    pub error: Option<String>,
    pub decoder_version: String,
    pub fingerprint: String,
    pub experimental_midstream: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Filters {
    pub direction: String,
    pub protocol: String,
    pub opcode: String,
    pub entity: String,
    pub status: String,
    pub search: String,
}

#[derive(Clone, Debug)]
pub struct StoredRow {
    pub id: i64,
    pub row: AnalysisRow,
}

pub struct Store {
    connection: Connection,
}

pub fn default_path() -> Result<PathBuf> {
    if let Some(root) = env::var_os("LOCALAPPDATA") {
        return Ok(PathBuf::from(root).join("Aetheris/debug-captures.sqlite3"));
    }
    #[cfg(not(windows))]
    {
        if let Some(root) = env::var_os("XDG_DATA_HOME") {
            return Ok(PathBuf::from(root).join("Aetheris/debug-captures.sqlite3"));
        }
        if let Some(root) = env::var_os("HOME") {
            return Ok(PathBuf::from(root).join(".local/share/Aetheris/debug-captures.sqlite3"));
        }
    }
    Err("Dossier local des données utilisateur introuvable.".into())
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;")?;
        let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version > 1 {
            return Err("Version de base SQLite plus récente que cette application.".into());
        }
        let mut connection = connection;
        crate::companion::migrations::initialize(
            &mut connection,
            crate::companion::install::database_history(),
        )?;
        Ok(Self { connection })
    }

    /// Call once at application startup, never when another worker opens a connection.
    pub fn recover_interrupted(&self) -> Result<()> {
        self.connection.execute(
            "UPDATE sessions SET status='interrompue', error=COALESCE(error, 'Application interrompue pendant la capture') WHERE status='live'", []
        )?;
        Ok(())
    }

    pub fn sessions(&self) -> Result<Vec<Session>> {
        let mut statement = self.connection.prepare(
            "SELECT s.id,s.name,s.source,s.started_at,s.status,
             (SELECT COUNT(*) FROM packets p WHERE p.session_id=s.id),
             (SELECT COUNT(*) FROM analysis_rows r WHERE r.session_id=s.id),
             s.error,s.decoder_version,s.fingerprint,strftime('%Y-%m-%d %H:%M:%S UTC',s.started_at,'unixepoch'),s.experimental_midstream FROM sessions s ORDER BY s.id DESC"
        )?;
        Ok(statement
            .query_map([], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    source: row.get(2)?,
                    started_at: row.get(3)?,
                    status: row.get(4)?,
                    packet_count: row.get::<_, i64>(5)? as usize,
                    row_count: row.get::<_, i64>(6)? as usize,
                    error: row.get(7)?,
                    decoder_version: row.get(8)?,
                    fingerprint: row.get(9)?,
                    experimental_midstream: row.get(11)?,
                    started_at_display: row.get(10)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn create_session(&self, source: &str, name: &str, fingerprint: &str) -> Result<i64> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        self.connection.execute(
            "INSERT INTO sessions(name,source,started_at,status,fingerprint) VALUES (?1,?2,?3,'live',?4)",
            params![name,source,timestamp,fingerprint]
        )?;
        Ok(self.connection.last_insert_rowid())
    }

    pub fn finish_session(&self, id: i64, error: Option<&str>) -> Result<()> {
        self.connection.execute(
            "UPDATE sessions SET status=?1,error=?2 WHERE id=?3",
            params![
                if error.is_some() {
                    "partielle"
                } else {
                    "terminée"
                },
                error,
                id
            ],
        )?;
        Ok(())
    }

    pub fn import_file(&mut self, path: &Path) -> Result<i64> {
        let mut source = PcapngSource::open(path)?;
        let batch = source.poll()?;
        let rows = analysis::analyze(
            &batch.packets,
            AnalysisOptions {
                finalized: true,
                ..Default::default()
            },
        );
        let tx = self.connection.transaction()?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        tx.execute(
            "INSERT INTO sessions(name,source,started_at,status,fingerprint,error,decoder_version) VALUES (?1,'fichier',?2,?3,?4,?5,?6)",
            params![source.name,timestamp,batch.status,source.fingerprint,batch.error,DECODER_VERSION]
        )?;
        let id = tx.last_insert_rowid();
        insert_packets(&tx, id, &batch.packets)?;
        insert_rows(&tx, id, &rows)?;
        tx.commit()?;
        Ok(id)
    }

    pub fn append_packets(&mut self, id: i64, packets: &[CapturedPacket]) -> Result<()> {
        let tx = self.connection.transaction()?;
        insert_packets(&tx, id, packets)?;
        tx.commit()?;
        Ok(())
    }

    pub fn packets(&self, id: i64) -> Result<Vec<CapturedPacket>> {
        let mut statement = self.connection.prepare(
            "SELECT number,section_id,interface_id,link_type,timestamp_ns,captured_len,original_len,bytes,timestamp_ticks,timestamp_resolution,timestamp_offset,local_address FROM packets WHERE session_id=?1 ORDER BY number"
        )?;
        Ok(statement
            .query_map([id], |row| {
                let timestamp: Option<String> = row.get(4)?;
                Ok(CapturedPacket {
                    number: row.get::<_, i64>(0)? as u64,
                    section_id: row.get(1)?,
                    interface_id: row.get(2)?,
                    link_type: LinkType::from_pcap(row.get(3)?),
                    timestamp_ns: timestamp.and_then(|value| value.parse().ok()),
                    timestamp: raw_timestamp(row)?,
                    local_address: row
                        .get::<_, Option<String>>(11)?
                        .and_then(|value| value.parse().ok()),
                    captured_len: row.get(5)?,
                    original_len: row.get(6)?,
                    bytes: row.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn reanalyze(&mut self, id: i64, finalized: bool) -> Result<()> {
        let (source, experimental): (String, bool) = self.connection.query_row(
            "SELECT source,experimental_midstream FROM sessions WHERE id=?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        self.analyze_and_replace(id, finalized, source == "fichier" && experimental)
    }

    pub fn reanalyze_with_alignment(&mut self, id: i64, experimental: bool) -> Result<()> {
        let source: String =
            self.connection
                .query_row("SELECT source FROM sessions WHERE id=?1", [id], |row| {
                    row.get(0)
                })?;
        if source != "fichier" {
            return Err("L’alignement expérimental est réservé aux imports offline.".into());
        }
        self.analyze_and_replace(id, true, experimental)
    }

    fn analyze_and_replace(&mut self, id: i64, finalized: bool, experimental: bool) -> Result<()> {
        let packets = self.packets(id)?;
        let rows = analysis::analyze(
            &packets,
            AnalysisOptions {
                finalized,
                experimental_midstream: experimental,
            },
        );
        self.replace_analysis_with_option(id, &rows, Some(experimental))
    }

    pub fn replace_analysis(&mut self, id: i64, rows: &[AnalysisRow]) -> Result<()> {
        self.replace_analysis_with_option(id, rows, None)
    }

    fn replace_analysis_with_option(
        &mut self,
        id: i64,
        rows: &[AnalysisRow],
        experimental: Option<bool>,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM analysis_rows WHERE session_id=?1", [id])?;
        insert_rows(&tx, id, rows)?;
        tx.execute(
            "UPDATE sessions SET decoder_version=?1,experimental_midstream=COALESCE(?2,experimental_midstream) WHERE id=?3",
            params![DECODER_VERSION, experimental, id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn delete_session(&self, id: i64) -> Result<()> {
        if self
            .connection
            .execute("DELETE FROM sessions WHERE id=?1 AND status<>'live'", [id])?
            == 0
        {
            return Err(
                "Session inexistante ou capture encore active ; arrêtez-la avant de supprimer."
                    .into(),
            );
        }
        Ok(())
    }

    pub fn capture_start_ns(&self, id: i64) -> Result<Option<i128>> {
        let value: Option<String> = self.connection.query_row(
            "SELECT timestamp_ns FROM packets WHERE session_id=?1 AND timestamp_ns IS NOT NULL ORDER BY number LIMIT 1",
            [id], |row|row.get(0),
        ).optional()?;
        Ok(value.and_then(|value| value.parse().ok()))
    }

    pub fn rows(
        &self,
        session_id: i64,
        filters: &Filters,
        offset: usize,
        limit: usize,
    ) -> Result<(usize, Vec<StoredRow>)> {
        let mut conditions = String::from("session_id=?");
        let mut values = vec![rusqlite::types::Value::Integer(session_id)];
        for (column, value) in [
            ("direction", &filters.direction),
            ("protocol", &filters.protocol),
            ("opcode", &filters.opcode),
            ("entity", &filters.entity),
            ("status", &filters.status),
        ] {
            if !value.is_empty() {
                if column == "opcode" {
                    conditions.push_str(" AND (replace(opcode,' ','') LIKE ? ESCAPE '\\' OR EXISTS (SELECT 1 FROM decoded_sections d WHERE d.row_id=analysis_rows.id AND d.kind='Opcode' AND d.value LIKE ? ESCAPE '\\'))");
                    values.push(like(&value.replace(' ', "")).into());
                    values.push(like(value).into());
                } else if column == "entity" {
                    conditions.push_str(" AND replace(entity,' ','') LIKE ? ESCAPE '\\'");
                    values.push(like(&value.replace(' ', "")).into());
                } else {
                    conditions.push_str(&format!(" AND {column} LIKE ? ESCAPE '\\'"));
                    values.push(like(value).into());
                }
            }
        }
        if !filters.search.is_empty() {
            conditions.push_str(" AND ((summary || ' ' || source || ' ' || destination || ' ' || kind) LIKE ? ESCAPE '\\' OR hex(bytes) LIKE ? ESCAPE '\\')");
            values.push(like(&filters.search).into());
            values.push(like(&filters.search.replace(' ', "")).into());
        }
        let count: i64 = self.connection.query_row(
            &format!("SELECT COUNT(*) FROM analysis_rows WHERE {conditions}"),
            rusqlite::params_from_iter(&values),
            |row| row.get(0),
        )?;
        let sql = format!(
            "SELECT id,kind,timestamp_ns,source,destination,direction,protocol,opcode,entity,status,summary,bytes FROM analysis_rows WHERE {conditions} ORDER BY ordinal LIMIT ? OFFSET ?"
        );
        values.push((limit as i64).into());
        values.push((offset as i64).into());
        let mut statement = self.connection.prepare(&sql)?;
        let mut rows = statement
            .query_map(rusqlite::params_from_iter(&values), |row| {
                let timestamp: Option<String> = row.get(2)?;
                Ok(StoredRow {
                    id: row.get(0)?,
                    row: AnalysisRow {
                        provenance: None,
                        kind: row.get(1)?,
                        timestamp_ns: timestamp.and_then(|value| value.parse().ok()),
                        source: row.get(3)?,
                        destination: row.get(4)?,
                        direction: row.get(5)?,
                        protocol: row.get(6)?,
                        opcode: row.get(7)?,
                        entity: row.get(8)?,
                        status: row.get(9)?,
                        summary: row.get(10)?,
                        bytes: row.get(11)?,
                        packet_numbers: Vec::new(),
                        sections: Vec::new(),
                    },
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for stored in &mut rows {
            stored.row.provenance = self
                .connection
                .query_row(
                    "SELECT envelope_ordinal,byte_offset FROM row_provenance WHERE row_id=?1",
                    [stored.id],
                    |row| {
                        let offset = row.get::<_, i64>(1)? as usize;
                        Ok(match row.get::<_, Option<i64>>(0)? {
                            Some(envelope) => ByteProvenance::Decompressed {
                                envelope: envelope as usize,
                                offset,
                            },
                            None => ByteProvenance::TcpStream { offset },
                        })
                    },
                )
                .optional()?;
            let mut statement = self.connection.prepare(
                "SELECT packet_number FROM row_packets WHERE row_id=?1 ORDER BY packet_number",
            )?;
            stored.row.packet_numbers = statement
                .query_map([stored.id], |row| Ok(row.get::<_, i64>(0)? as u64))?
                .collect::<rusqlite::Result<_>>()?;
            let mut statement = self.connection.prepare("SELECT kind,value,confidence,byte_offset,bytes FROM decoded_sections WHERE row_id=?1 ORDER BY id")?;
            stored.row.sections = statement
                .query_map([stored.id], |row| {
                    Ok(DecodedSection {
                        kind: row.get(0)?,
                        value: row.get(1)?,
                        confidence: row.get(2)?,
                        offset: row.get::<_, i64>(3)? as usize,
                        bytes: row.get(4)?,
                    })
                })?
                .collect::<rusqlite::Result<_>>()?;
        }
        Ok((count as usize, rows))
    }

    /// Envelopes remain inspectable even when excluded by the active filters.
    pub fn envelope_chain(&self, session_id: i64, row: &AnalysisRow) -> Result<Vec<StoredRow>> {
        let mut provenance = row.provenance.clone();
        let mut chain = Vec::new();
        while let Some(ByteProvenance::Decompressed { envelope, .. }) = provenance {
            let stored = self
                .rows(session_id, &Filters::default(), envelope, 1)?
                .1
                .pop()
                .ok_or("Enveloppe source absente")?;
            provenance = stored.row.provenance.clone();
            chain.push(stored);
        }
        Ok(chain)
    }

    pub fn original_packets(
        &self,
        session_id: i64,
        numbers: &[u64],
    ) -> Result<Vec<CapturedPacket>> {
        let mut packets = Vec::new();
        for number in numbers {
            let packet = self.connection.query_row(
                "SELECT number,section_id,interface_id,link_type,timestamp_ns,captured_len,original_len,bytes,timestamp_ticks,timestamp_resolution,timestamp_offset,local_address FROM packets WHERE session_id=?1 AND number=?2",
                params![session_id,i64::try_from(*number)?], |row| {
                    let timestamp: Option<String> = row.get(4)?;
                    Ok(CapturedPacket {
                        number: row.get::<_, i64>(0)? as u64, section_id: row.get(1)?, interface_id: row.get(2)?,
                        link_type: LinkType::from_pcap(row.get(3)?), timestamp_ns: timestamp.and_then(|value| value.parse().ok()),
                        timestamp: raw_timestamp(row)?,
                        local_address: row.get::<_,Option<String>>(11)?.and_then(|value|value.parse().ok()),
                        captured_len: row.get(5)?, original_len: row.get(6)?, bytes: row.get(7)?,
                    })
                }
            )?;
            packets.push(packet);
        }
        Ok(packets)
    }
}

fn like(text: &str) -> String {
    format!(
        "%{}%",
        text.replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

fn raw_timestamp(row: &rusqlite::Row<'_>) -> rusqlite::Result<Option<PacketTimestamp>> {
    let ticks: Option<String> = row.get(8)?;
    match ticks {
        Some(ticks) => {
            let ticks = ticks.parse().map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;
            Ok(Some(PacketTimestamp {
                ticks,
                resolution: row.get(9)?,
                offset_seconds: row.get(10)?,
            }))
        }
        None => Ok(None),
    }
}

fn insert_packets(tx: &Transaction<'_>, id: i64, packets: &[CapturedPacket]) -> Result<()> {
    let mut statement = tx.prepare(
        "INSERT INTO packets(session_id,number,section_id,interface_id,link_type,timestamp_ns,captured_len,original_len,bytes,timestamp_ticks,timestamp_resolution,timestamp_offset,local_address) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)"
    )?;
    for packet in packets {
        statement.execute(params![
            id,
            i64::try_from(packet.number)?,
            packet.section_id,
            packet.interface_id,
            packet.link_type.number(),
            packet.timestamp_ns.map(|value| value.to_string()),
            packet.captured_len,
            packet.original_len,
            packet.bytes,
            packet.timestamp.map(|time| time.ticks.to_string()),
            packet.timestamp.map(|time| time.resolution),
            packet.timestamp.map(|time| time.offset_seconds),
            packet.local_address.map(|address| address.to_string())
        ])?;
    }
    Ok(())
}

fn insert_rows(tx: &Transaction<'_>, id: i64, rows: &[AnalysisRow]) -> Result<()> {
    let mut row_statement = tx.prepare(
        "INSERT INTO analysis_rows(session_id,ordinal,kind,timestamp_ns,source,destination,direction,protocol,opcode,entity,status,summary,bytes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)"
    )?;
    let mut link_statement =
        tx.prepare("INSERT INTO row_packets(row_id,session_id,packet_number) VALUES (?1,?2,?3)")?;
    let mut section_statement = tx.prepare("INSERT INTO decoded_sections(row_id,kind,value,confidence,byte_offset,bytes) VALUES (?1,?2,?3,?4,?5,?6)")?;
    for (ordinal, row) in rows.iter().enumerate() {
        row_statement.execute(params![
            id,
            ordinal as i64,
            row.kind,
            row.timestamp_ns.map(|value| value.to_string()),
            row.source,
            row.destination,
            row.direction,
            row.protocol,
            row.opcode,
            row.entity,
            row.status,
            row.summary,
            row.bytes
        ])?;
        let row_id = tx.last_insert_rowid();
        if let Some(provenance) = &row.provenance {
            let (envelope, offset) = match provenance {
                ByteProvenance::TcpStream { offset } => (None, *offset),
                ByteProvenance::Decompressed { envelope, offset } => {
                    if *envelope >= ordinal
                        || rows[*envelope].kind != "enveloppe LZ4"
                        || rows[*envelope].packet_numbers != row.packet_numbers
                    {
                        return Err("Provenance d’enveloppe invalide".into());
                    }
                    (Some(i64::try_from(*envelope)?), *offset)
                }
            };
            tx.execute("INSERT INTO row_provenance(row_id,session_id,ordinal,envelope_ordinal,byte_offset) VALUES (?1,?2,?3,?4,?5)",
                params![row_id, id, ordinal as i64, envelope, i64::try_from(offset)?])?;
        }
        for packet in &row.packet_numbers {
            link_statement.execute(params![row_id, id, i64::try_from(*packet)?])?;
        }
        for section in &row.sections {
            section_statement.execute(params![
                row_id,
                section.kind,
                section.value,
                section.confidence,
                section.offset as i64,
                section.bytes
            ])?;
        }
    }
    Ok(())
}
