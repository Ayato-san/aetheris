//! Conservative TCP reconstruction and explicitly provisional AION framing.
use super::{network, pcapng::CapturedPacket};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
};

const MAX_STREAM_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedSection {
    pub kind: String,
    pub value: String,
    pub confidence: String,
    pub offset: usize,
    pub bytes: Vec<u8>,
}

/// Positions refer to a reconstructed TCP stream or to one envelope's output,
/// never to an offset in a captured packet. Envelope references are row ordinals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ByteProvenance {
    TcpStream { offset: usize },
    Decompressed { envelope: usize, offset: usize },
}

#[derive(Clone, Debug)]
pub struct AnalysisRow {
    pub provenance: Option<ByteProvenance>,
    pub kind: String,
    pub timestamp_ns: Option<i128>,
    pub source: String,
    pub destination: String,
    pub direction: String,
    pub protocol: String,
    pub opcode: String,
    pub entity: String,
    pub status: String,
    pub summary: String,
    pub bytes: Vec<u8>,
    pub packet_numbers: Vec<u64>,
    pub sections: Vec<DecodedSection>,
}

impl AnalysisRow {
    /// Stored documentary names remain tied to the decoder used for this row.
    pub fn opcode_display_name(&self) -> &str {
        let name = self.opcode_name();
        super::opcode::Opcode::KNOWN
            .iter()
            .filter_map(|opcode| opcode.info())
            .find(|info| info.name == name)
            .map(|info| info.display_name())
            .unwrap_or(name)
    }

    pub fn opcode_name(&self) -> &str {
        self.sections
            .iter()
            .find(|section| section.kind == "Opcode")
            .map(|section| section.value.as_str())
            .unwrap_or("")
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AnalysisOptions {
    /// Pending trailing bytes become incomplete errors only on finalization.
    pub finalized: bool,
    /// Explicit offline fallback when automatic server synchronization fails.
    pub experimental_midstream: bool,
}

type ConnectionKey = (u32, u32, SocketAddr, SocketAddr, u32);

#[derive(Default)]
struct Connection {
    generation: u32,
    client: Option<SocketAddr>,
    initial_syn: Option<(SocketAddr, u32)>,
    closed: bool,
}

#[derive(Default)]
struct Flow {
    start_sequence: Option<u32>,
    segments: Vec<(usize, network::TcpSegment)>,
    client: Option<SocketAddr>,
    issue: Option<String>,
}

struct PreparedFlow {
    key: ConnectionKey,
    source: SocketAddr,
    destination: SocketAddr,
    flow: Flow,
    stream: Vec<u8>,
    owners: Vec<usize>,
    stream_error: Option<String>,
    synchronized: Option<usize>,
}

/// Every source packet is retained separately by the store. Rows represent
/// reconstructed messages, or packets not fully consumed by a message.
pub fn analyze(packets: &[CapturedPacket], options: AnalysisOptions) -> Vec<AnalysisRow> {
    let info: Vec<_> = packets.iter().map(network::parse).collect();
    let mut connections = BTreeMap::<(u32, u32, SocketAddr, SocketAddr), Connection>::new();
    let mut flows = BTreeMap::<(u32, u32, SocketAddr, SocketAddr, u32), Flow>::new();
    let mut reasons: Vec<_> = info.iter().map(|packet| packet.issue.clone()).collect();
    let mut consumed = vec![0usize; packets.len()];
    let mut directions: Vec<String> = packets
        .iter()
        .zip(&info)
        .map(|(packet, parsed)| {
            match packet.local_address {
                Some(local) if parsed.source.is_some_and(|source| source.ip() == local) => {
                    "Client → serveur"
                }
                Some(local)
                    if parsed
                        .destination
                        .is_some_and(|destination| destination.ip() == local) =>
                {
                    "Serveur → client"
                }
                _ => "Inconnue",
            }
            .into()
        })
        .collect();
    let mut rows = Vec::new();
    let mut packet_connections = vec![None; packets.len()];
    let mut inferred_clients = BTreeMap::<ConnectionKey, SocketAddr>::new();
    for (index, parsed) in info.iter().enumerate() {
        let (Some(source), Some(destination)) = (parsed.source, parsed.destination) else {
            continue;
        };
        if parsed.protocol != "TCP" {
            continue;
        }
        let (left, right) = if source < destination {
            (source, destination)
        } else {
            (destination, source)
        };
        let packet = &packets[index];
        let connection = connections
            .entry((packet.section_id, packet.interface_id, left, right))
            .or_default();
        if let Some(segment) = &parsed.tcp {
            if segment.syn && !segment.ack {
                let syn = (source, segment.sequence);
                if connection
                    .initial_syn
                    .is_some_and(|previous| previous != syn)
                    || connection.closed
                {
                    connection.generation += 1;
                    connection.closed = false;
                }
                connection.initial_syn = Some(syn);
                connection.client = Some(source);
            } else if segment.syn && segment.ack && connection.client.is_none() {
                connection.client = Some(destination);
            }
        }
        if connection.client.is_none() {
            connection.client = match packet.local_address {
                Some(local) if source.ip() == local && destination.ip() != local => Some(source),
                Some(local) if destination.ip() == local && source.ip() != local => {
                    Some(destination)
                }
                _ => None,
            };
        }
        packet_connections[index] = Some((
            packet.section_id,
            packet.interface_id,
            left,
            right,
            connection.generation,
        ));
        let flow = flows
            .entry((
                packet.section_id,
                packet.interface_id,
                source,
                destination,
                connection.generation,
            ))
            .or_default();
        flow.client = connection.client;
        if let Some(issue) = &parsed.issue
            && parsed.tcp.is_none()
        {
            flow.issue = Some(issue.clone());
        }
        if let Some(segment) = &parsed.tcp {
            if segment.syn {
                flow.start_sequence = Some(segment.sequence.wrapping_add(1));
            }
            if segment.fin || segment.rst {
                connection.closed = true;
            }
            flow.segments.push((index, segment.clone()));
        }
    }

    let mut prepared = Vec::new();
    let mut tls_connections = BTreeSet::new();
    let mut ambiguous_connections = BTreeSet::new();
    for ((section, interface, source, destination, generation), flow) in flows {
        let direction = match flow.client {
            Some(client) if source == client => "Client → serveur",
            Some(_) => "Serveur → client",
            None => "Inconnue",
        };
        for (index, _) in &flow.segments {
            if flow.client.is_some() {
                directions[*index] = direction.into();
            }
        }
        let assumed_start = if flow.start_sequence.is_none() {
            let first = flow
                .segments
                .iter()
                .find(|(_, segment)| !segment.payload.is_empty())
                .map(|(_, segment)| segment.sequence);
            first.and_then(|anchor| {
                flow.segments
                    .iter()
                    .filter(|(_, segment)| !segment.payload.is_empty())
                    .min_by_key(|(_, segment)| segment.sequence.wrapping_sub(anchor) as i32)
                    .map(|(_, segment)| segment.sequence)
            })
        } else {
            None
        };
        let Some(base) = flow.start_sequence.or(assumed_start) else {
            for (index, segment) in &flow.segments {
                if !segment.payload.is_empty() {
                    reasons[*index] =
                        Some("Flux incomplet : SYN absent, aucun alignement supposé".into());
                }
            }
            continue;
        };
        if let Some(issue) = &flow.issue {
            for (index, segment) in &flow.segments {
                if !segment.payload.is_empty() {
                    reasons[*index] = Some(issue.clone());
                }
            }
            continue;
        }
        let mut segments = flow
            .segments
            .iter()
            .filter(|(index, segment)| !segment.payload.is_empty() || info[*index].issue.is_some())
            .collect::<Vec<_>>();
        segments.sort_by_key(|(_, segment)| {
            segment
                .sequence
                .wrapping_add(u32::from(segment.syn))
                .wrapping_sub(base) as i32
        });
        let mut stream = Vec::<u8>::new();
        let mut owners = Vec::<usize>::new();
        let mut stream_error = None;
        for (index, segment) in segments {
            if let Some(issue) = &info[*index].issue {
                stream_error = Some(issue.clone());
                break;
            }
            let relative = segment
                .sequence
                .wrapping_add(u32::from(segment.syn))
                .wrapping_sub(base) as i32;
            if relative < 0 {
                stream_error = Some("Séquence TCP antérieure au SYN capturé".to_owned());
                break;
            }
            let offset = relative as usize;
            if offset > stream.len() {
                stream_error = Some(format!(
                    "Trou TCP : {} octets manquants",
                    offset - stream.len()
                ));
                break;
            }
            let overlap = segment.payload.len().min(stream.len() - offset);
            if let Some(conflict) = segment.payload[..overlap]
                .iter()
                .zip(&stream[offset..offset + overlap])
                .position(|(left, right)| left != right)
            {
                stream.truncate(offset + conflict);
                owners.truncate(stream.len());
                stream_error = Some("Recouvrement TCP contradictoire".into());
                break;
            }
            if overlap == segment.payload.len() {
                reasons[*index] = Some("Retransmission TCP identique".into());
            }
            let suffix = &segment.payload[overlap..];
            if stream.len().saturating_add(suffix.len()) > MAX_STREAM_BYTES {
                stream_error =
                    Some("Limite de réassemblage atteinte (32 Mio par direction)".into());
                break;
            }
            stream.extend_from_slice(suffix);
            owners.extend(std::iter::repeat_n(*index, suffix.len()));
        }
        let (left, right) = if source < destination {
            (source, destination)
        } else {
            (destination, source)
        };
        let key = (section, interface, left, right, generation);
        if super::synchronization::is_tls(&stream) {
            tls_connections.insert(key);
        }
        let synchronized = if flow.start_sequence.is_none() && direction != "Client → serveur" {
            super::synchronization::server_boundary(&stream)
        } else {
            None
        };
        if synchronized.is_some()
            && inferred_clients
                .insert(key, destination)
                .is_some_and(|client| client != destination)
        {
            ambiguous_connections.insert(key);
        }
        prepared.push(PreparedFlow {
            key,
            source,
            destination,
            flow,
            stream,
            owners,
            stream_error,
            synchronized,
        });
    }

    for key in ambiguous_connections.union(&tls_connections) {
        inferred_clients.remove(key);
    }
    for PreparedFlow {
        key,
        source,
        destination,
        flow,
        stream,
        owners,
        mut stream_error,
        synchronized,
    } in prepared
    {
        if tls_connections.contains(&key) || ambiguous_connections.contains(&key) {
            for (index, segment) in &flow.segments {
                if !segment.payload.is_empty() {
                    reasons[*index] = Some(if tls_connections.contains(&key) {
                        "Connexion TLS identifiée : décodage AION 2 exclu"
                    } else {
                        "Synchronisation AION 2 ambiguë : ticks reconnus dans les deux directions"
                    }.into());
                }
            }
            continue;
        }
        let client = flow.client.or_else(|| inferred_clients.get(&key).copied());
        let direction = match client {
            Some(client) if client == source => "Client → serveur",
            Some(_) => "Serveur → client",
            None => "Inconnue",
        };
        for (index, _) in &flow.segments {
            directions[*index] = direction.into();
        }
        if flow.start_sequence.is_none() && direction == "Client → serveur" {
            for (index, segment) in &flow.segments {
                if !segment.payload.is_empty() {
                    reasons[*index] = Some(stream_error.clone().unwrap_or_else(|| {
                        "Corps client non déchiffré : capture en cours de connexion".into()
                    }));
                }
            }
            continue;
        }
        let experimental = flow.start_sequence.is_none() && options.experimental_midstream;
        let mut offset = if let Some(boundary) = synchronized {
            boundary
        } else if flow.start_sequence.is_none() && !experimental {
            for (index, segment) in &flow.segments {
                if !segment.payload.is_empty() {
                    reasons[*index] = Some(stream_error.clone().unwrap_or_else(||
                        "SYN absent : en attente de synchronisation AION 2 (trois ticks serveur cohérents)".into()));
                }
            }
            continue;
        } else {
            0
        };
        if offset > 0 {
            for index in &owners[..offset] {
                reasons[*index] = Some(format!(
                    "Préfixe non interprété avant synchronisation AION 2 à l’offset {offset}"
                ));
            }
        }
        while offset < stream.len() {
            if stream[offset] == 0 {
                consumed[owners[offset]] += 1;
                offset += 1;
                continue;
            }
            let (length, width) = match super::compression::frame_length(&stream[offset..]) {
                Ok(Some(frame)) => frame,
                Ok(None) if !options.finalized => break,
                result => {
                    stream_error.get_or_insert_with(|| {
                        format!(
                            "Longueur AION invalide à l’offset {offset} : {}",
                            result.err().unwrap_or("préfixe incomplet")
                        )
                    });
                    break;
                }
            };
            if length > stream.len() - offset {
                if options.finalized {
                    stream_error.get_or_insert_with(|| "Message applicatif final incomplet".into());
                }
                break;
            }
            let source_indexes = owners[offset..offset + length]
                .iter()
                .copied()
                .collect::<BTreeSet<_>>();
            let first = owners[offset];
            let bytes = stream[offset..offset + length].to_vec();
            let row = AnalysisRow {
                provenance: Some(ByteProvenance::TcpStream { offset }),
                kind: "message".into(),
                timestamp_ns: packets[first].timestamp_ns,
                source: source.to_string(),
                destination: destination.to_string(),
                direction: direction.into(),
                protocol: "TCP".into(),
                opcode: String::new(),
                entity: String::new(),
                status: "non pris en charge".into(),
                summary: format!(
                    "{}{}",
                    if synchronized.is_some() {
                        "SYN absent · synchronisation AION 2 déduite de trois ticks serveur cohérents — "
                    } else if experimental {
                        "ALIGNEMENT SANS SYN IMPOSÉ — "
                    } else {
                        ""
                    },
                    "Message reconstruit · framing expérimental"
                ),
                packet_numbers: source_indexes
                    .iter()
                    .map(|index| packets[*index].number)
                    .collect(),
                sections: Vec::new(),
                bytes,
            };
            if direction == "Serveur → client" {
                super::compression::expand(row, width, &mut rows);
            } else {
                let mut row = row;
                row.summary.push_str(if direction == "Client → serveur" {
                    " · corps client non déchiffré"
                } else {
                    " · direction inconnue : corps non interprété"
                });
                rows.push(row);
            }
            for index in &owners[offset..offset + length] {
                consumed[*index] += 1;
            }
            offset += length;
        }
        for (index, segment) in &flow.segments {
            if consumed[*index] < segment.payload.len() && reasons[*index].is_none() {
                reasons[*index] = Some(
                    stream_error
                        .clone()
                        .unwrap_or_else(|| "Octets TCP en attente de message complet".into()),
                );
            }
        }
    }
    for (index, packet) in packets.iter().enumerate() {
        let parsed = &info[index];
        if let Some(client) = packet_connections[index].and_then(|key| inferred_clients.get(&key)) {
            directions[index] = if parsed.source == Some(*client) {
                "Client → serveur"
            } else {
                "Serveur → client"
            }
            .into();
        }
        let payload_length = parsed
            .tcp
            .as_ref()
            .map_or(0, |segment| segment.payload.len());
        if payload_length > 0 && consumed[index] == payload_length && reasons[index].is_none() {
            continue;
        }
        let summary = reasons[index].clone().unwrap_or_else(|| {
            if parsed.protocol == "TCP" {
                "Contrôle TCP".into()
            } else if parsed.protocol == "UDP" {
                "Datagramme UDP brut".into()
            } else {
                "Paquet brut".into()
            }
        });
        let status = if summary.contains("attente") {
            "en attente"
        } else if summary.contains("non pris en charge")
            || summary.contains("non déchiffré")
            || summary.contains("TLS identifiée")
        {
            "non pris en charge"
        } else if summary.contains("Retransmission") || summary == "Contrôle TCP" {
            "observé"
        } else if reasons[index].is_some() {
            "incomplet / erreur"
        } else {
            "inconnu"
        };
        rows.push(AnalysisRow {
            provenance: None,
            kind: "paquet brut".into(),
            timestamp_ns: packet.timestamp_ns,
            source: parsed
                .source
                .map(|value| value.to_string())
                .unwrap_or_default(),
            destination: parsed
                .destination
                .map(|value| value.to_string())
                .unwrap_or_default(),
            direction: directions[index].clone(),
            protocol: parsed.protocol.clone(),
            opcode: String::new(),
            entity: String::new(),
            status: status.into(),
            summary,
            bytes: packet.bytes.clone(),
            packet_numbers: vec![packet.number],
            sections: Vec::new(),
        });
    }
    let mut indexed: Vec<_> = rows.into_iter().enumerate().collect();
    indexed.sort_by_key(|(_, row)| {
        (
            row.timestamp_ns,
            row.packet_numbers.first().copied().unwrap_or_default(),
        )
    });
    let mut ordinals = vec![0; indexed.len()];
    for (new, (old, _)) in indexed.iter().enumerate() {
        ordinals[*old] = new;
    }
    indexed
        .into_iter()
        .map(|(_, mut row)| {
            if let Some(ByteProvenance::Decompressed { envelope, .. }) = &mut row.provenance {
                *envelope = ordinals[*envelope];
            }
            row
        })
        .collect()
}

pub fn read_varuint(bytes: &[u8]) -> Option<(u64, usize)> {
    let mut result = 0u64;
    for (index, value) in bytes.iter().copied().take(10).enumerate() {
        if index == 9 && value > 1 {
            return None;
        }
        result |= ((value & 0x7f) as u64) << (index * 7);
        if value & 0x80 == 0 {
            return Some((result, index + 1));
        }
    }
    None
}
