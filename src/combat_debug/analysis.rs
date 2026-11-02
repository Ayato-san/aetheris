//! Conservative TCP reconstruction and explicitly provisional AION framing.
use super::{hex, network, pcapng::CapturedPacket};
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

#[derive(Clone, Debug)]
pub struct AnalysisRow {
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

#[derive(Clone, Copy, Debug, Default)]
pub struct AnalysisOptions {
    /// Pending trailing bytes become incomplete errors only on finalization.
    pub finalized: bool,
    /// Explicit offline experiment only. A missing SYN is never trusted by default.
    pub experimental_midstream: bool,
}

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

    for ((_, _, source, destination, _), flow) in flows {
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
        let experimental = flow.start_sequence.is_none() && options.experimental_midstream;
        let assumed_start = if experimental {
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
        let mut offset = 0usize;
        while offset < stream.len() {
            let Some(length) = stream[offset].checked_sub(3).filter(|length| *length >= 3) else {
                stream_error.get_or_insert_with(|| {
                    format!("Longueur AION provisoire invalide à l’offset {offset}")
                });
                break;
            };
            let length = length as usize;
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
            let sections = decode(&bytes);
            let entity = sections
                .iter()
                .find(|section| section.kind == "Entité candidate")
                .map(|section| section.value.clone())
                .unwrap_or_default();
            let opcode = hex(&bytes[1..3]);
            rows.push(AnalysisRow {
                kind: "message".into(),
                timestamp_ns: packets[first].timestamp_ns,
                source: source.to_string(),
                destination: destination.to_string(),
                direction: direction.into(),
                protocol: "TCP".into(),
                opcode,
                entity,
                status: if sections.is_empty() {
                    "inconnu"
                } else {
                    "candidat"
                }
                .into(),
                summary: format!(
                    "{}{}",
                    if experimental {
                        "ALIGNEMENT SANS SYN IMPOSÉ — "
                    } else {
                        ""
                    },
                    if sections.is_empty() {
                        "Message inconnu — framing expérimental".into()
                    } else {
                        sections
                            .iter()
                            .map(|section| {
                                format!(
                                    "{} : {} ({})",
                                    section.kind, section.value, section.confidence
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" · ")
                    }
                ),
                packet_numbers: source_indexes
                    .iter()
                    .map(|index| packets[*index].number)
                    .collect(),
                sections,
                bytes,
            });
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
        } else if summary.contains("non pris en charge") {
            "non pris en charge"
        } else if summary.contains("Retransmission") || summary == "Contrôle TCP" {
            "observé"
        } else if reasons[index].is_some() {
            "incomplet / erreur"
        } else {
            "inconnu"
        };
        rows.push(AnalysisRow {
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
    rows.sort_by_key(|row| {
        (
            row.timestamp_ns,
            row.packet_numbers.first().copied().unwrap_or_default(),
        )
    });
    rows
}

fn section(kind: &str, value: String, offset: usize, bytes: &[u8]) -> DecodedSection {
    DecodedSection {
        kind: kind.into(),
        value,
        confidence: "hypothèse".into(),
        offset,
        bytes: bytes.to_vec(),
    }
}

fn decode(bytes: &[u8]) -> Vec<DecodedSection> {
    let mut sections = Vec::new();
    if bytes.len() >= 6 && matches!(&bytes[1..3], [0x00, 0x8d] | [0x04, 0x38]) {
        sections.push(section(
            "Entité candidate",
            hex(&bytes[3..6]),
            3,
            &bytes[3..6],
        ));
    }
    if bytes.len() == 17 && bytes[1..3] == [0x00, 0x8d] {
        sections.push(section(
            "HP candidats (u32 LE)",
            u32::from_le_bytes(bytes[9..13].try_into().unwrap()).to_string(),
            9,
            &bytes[9..13],
        ));
    }
    if bytes.len() >= 20 && bytes[1..3] == [0x04, 0x38] {
        for offset in 18..bytes.len() - 1 {
            if bytes[offset..offset + 2] != [0xe6, 0x64] {
                continue;
            }
            if let Some((value, length)) = read_varuint(&bytes[offset + 2..]) {
                sections.push(section(
                    "Montant candidat après E6 64 (ULEB128)",
                    value.to_string(),
                    offset + 2,
                    &bytes[offset + 2..offset + 2 + length],
                ));
            }
        }
    }
    sections
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
