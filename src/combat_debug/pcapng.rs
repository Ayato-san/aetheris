//! Minimal PCAPNG reader and writer preserving captured packet evidence.

mod writer;
pub use writer::Writer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteOrder {
    Little,
    Big,
}

impl ByteOrder {
    fn u16(self, bytes: &[u8]) -> u16 {
        let pair = [bytes[0], bytes[1]];
        match self {
            Self::Little => u16::from_le_bytes(pair),
            Self::Big => u16::from_be_bytes(pair),
        }
    }

    fn u32(self, bytes: &[u8]) -> u32 {
        let quad = [bytes[0], bytes[1], bytes[2], bytes[3]];
        match self {
            Self::Little => u32::from_le_bytes(quad),
            Self::Big => u32::from_be_bytes(quad),
        }
    }

    fn i64(self, bytes: &[u8]) -> i64 {
        let octets = [
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ];
        match self {
            Self::Little => i64::from_le_bytes(octets),
            Self::Big => i64::from_be_bytes(octets),
        }
    }
}

pub use crate::capture::{CapturedPacket, LinkType, PacketTimestamp};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PcapngImport {
    pub packets: Vec<CapturedPacket>,
    pub blocks_read: u64,
    /// Set when parsing stopped at an invalid boundary or malformed packet block.
    pub error: Option<String>,
}

impl PcapngImport {
    pub fn is_partial(&self) -> bool {
        self.error.is_some()
    }
}

#[derive(Clone, Copy, Debug)]
struct Interface {
    link_type: LinkType,
    snaplen: u32,
    timestamp_resolution: TimestampResolution,
    timestamp_offset_seconds: i64,
}

#[derive(Clone, Copy, Debug)]
struct TimestampResolution {
    base: u32,
    exponent: u8,
}

impl TimestampResolution {
    fn encoded(self) -> u8 {
        self.exponent | if self.base == 2 { 0x80 } else { 0 }
    }
    fn to_unix_nanoseconds(self, ticks: u64, offset_seconds: i64) -> Option<i128> {
        let scale = (self.base as i128).checked_pow(self.exponent as u32)?;
        let tick_ns = (ticks as i128).checked_mul(1_000_000_000)? / scale;
        (offset_seconds as i128)
            .checked_mul(1_000_000_000)?
            .checked_add(tick_ns)
    }
}

const SECTION_HEADER: [u8; 4] = [0x0a, 0x0d, 0x0d, 0x0a];

/// Reads known packet blocks and safely skips other blocks. When a block
/// boundary cannot be trusted, it returns all packets read so far with an
/// error instead of searching arbitrary payload bytes for another boundary.
pub fn read(bytes: &[u8]) -> PcapngImport {
    let mut result = PcapngImport {
        packets: Vec::new(),
        blocks_read: 0,
        error: None,
    };
    let mut cursor = 0usize;
    let mut order: Option<ByteOrder> = None;
    let mut interfaces: Vec<Interface> = Vec::new();
    let mut section_id = 0;

    while cursor < bytes.len() {
        if bytes.len() - cursor < 12 {
            result.error = Some(format!(
                "PCAPNG incomplet à l’octet {cursor} : en-tête de bloc tronqué."
            ));
            break;
        }

        let raw_type: [u8; 4] = bytes[cursor..cursor + 4].try_into().unwrap();
        let new_section = raw_type == SECTION_HEADER;
        let block_order = if new_section {
            if bytes.len() - cursor < 16 {
                result.error = Some(format!(
                    "PCAPNG incomplet à l’octet {cursor} : en-tête de section tronqué."
                ));
                break;
            }
            match bytes[cursor + 8..cursor + 12] {
                [0x1a, 0x2b, 0x3c, 0x4d] => ByteOrder::Big,
                [0x4d, 0x3c, 0x2b, 0x1a] => ByteOrder::Little,
                _ => {
                    result.error = Some(format!(
                        "Marque d’ordre des octets invalide à l’octet {}.",
                        cursor + 8
                    ));
                    break;
                }
            }
        } else if let Some(order) = order {
            order
        } else {
            result.error = Some("Le fichier ne commence pas par un bloc Section Header.".into());
            break;
        };

        let block_type = if new_section {
            0x0a0d0d0a
        } else {
            block_order.u32(&bytes[cursor..cursor + 4])
        };
        let block_len = block_order.u32(&bytes[cursor + 4..cursor + 8]) as usize;
        if block_len < 12 || !block_len.is_multiple_of(4) {
            result.error = Some(format!(
                "Longueur de bloc invalide ({block_len}) à l’octet {cursor}."
            ));
            break;
        }
        let Some(end) = cursor.checked_add(block_len) else {
            result.error = Some(format!("Débordement de longueur à l’octet {cursor}."));
            break;
        };
        if end > bytes.len() {
            result.error = Some(format!(
                "Bloc tronqué à l’octet {cursor} : {block_len} octets annoncés."
            ));
            break;
        }
        if block_order.u32(&bytes[end - 4..end]) as usize != block_len {
            result.error = Some(format!(
                "Longueur de fin de bloc incohérente à l’octet {cursor}."
            ));
            break;
        }

        let body = &bytes[cursor + 8..end - 4];
        if new_section {
            if block_len < 28 {
                result.error = Some(format!(
                    "Bloc Section Header trop court à l’octet {cursor}."
                ));
                break;
            }
            if order.is_some() {
                section_id += 1;
            }
            order = Some(block_order);
            interfaces.clear();
        } else {
            let first_packet = result.packets.len();
            match block_type {
                1 => match parse_interface(body, block_order) {
                    Ok(interface) => interfaces.push(interface),
                    Err(error) => {
                        result.error = Some(format!("Interface PCAPNG invalide : {error}"));
                        break;
                    }
                },
                6 => {
                    if let Err(error) =
                        parse_enhanced_packet(body, block_order, &interfaces, &mut result.packets)
                    {
                        result.error = Some(error);
                        break;
                    }
                }
                3 => {
                    if let Err(error) =
                        parse_simple_packet(body, block_order, &interfaces, &mut result.packets)
                    {
                        result.error = Some(error);
                        break;
                    }
                }
                2 => {
                    if let Err(error) =
                        parse_packet_block(body, block_order, &interfaces, &mut result.packets)
                    {
                        result.error = Some(error);
                        break;
                    }
                }
                _ => {}
            }
            for packet in &mut result.packets[first_packet..] {
                packet.section_id = section_id;
            }
        }

        result.blocks_read += 1;
        cursor = end;
    }

    if cursor == bytes.len() && order.is_none() && result.error.is_none() {
        result.error = Some("Le fichier PCAPNG ne contient aucune section.".into());
    }
    result
}

fn parse_interface(body: &[u8], order: ByteOrder) -> Result<Interface, String> {
    if body.len() < 8 {
        return Err("bloc Interface Description incomplet".into());
    }
    let link_type = LinkType::from_pcap(order.u16(&body[0..2]));
    let snaplen = order.u32(&body[4..8]);
    let mut interface = Interface {
        link_type,
        snaplen,
        timestamp_resolution: TimestampResolution {
            base: 10,
            exponent: 6,
        },
        timestamp_offset_seconds: 0,
    };
    parse_options(&body[8..], order, |code, value| match code {
        9 if value.len() == 1 => {
            let resolution = value[0];
            interface.timestamp_resolution = TimestampResolution {
                base: if resolution & 0x80 == 0 { 10 } else { 2 },
                exponent: resolution & 0x7f,
            };
        }
        14 if value.len() == 8 => {
            interface.timestamp_offset_seconds = order.i64(value);
        }
        _ => {}
    })?;
    Ok(interface)
}

fn parse_options(
    bytes: &[u8],
    order: ByteOrder,
    mut visit: impl FnMut(u16, &[u8]),
) -> Result<(), String> {
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes.len() - cursor < 4 {
            return Err("option PCAPNG tronquée".into());
        }
        let code = order.u16(&bytes[cursor..cursor + 2]);
        let length = order.u16(&bytes[cursor + 2..cursor + 4]) as usize;
        cursor += 4;
        if code == 0 {
            return Ok(());
        }
        let padded = length.checked_add(3).ok_or("longueur d’option invalide")? & !3;
        let Some(end) = cursor.checked_add(padded) else {
            return Err("débordement de longueur d’option".into());
        };
        if end > bytes.len() {
            return Err("valeur d’option tronquée".into());
        }
        visit(code, &bytes[cursor..cursor + length]);
        cursor = end;
    }
    Ok(())
}

fn parse_enhanced_packet(
    body: &[u8],
    order: ByteOrder,
    interfaces: &[Interface],
    packets: &mut Vec<CapturedPacket>,
) -> Result<(), String> {
    if body.len() < 20 {
        return Err("Enhanced Packet Block incomplet".into());
    }
    let interface_id = order.u32(&body[0..4]);
    let interface = interfaces
        .get(interface_id as usize)
        .ok_or_else(|| format!("Interface {interface_id} inexistante."))?;
    let ticks = ((order.u32(&body[4..8]) as u64) << 32) | order.u32(&body[8..12]) as u64;
    let captured_len = order.u32(&body[12..16]);
    let original_len = order.u32(&body[16..20]);
    let payload_len = captured_len as usize;
    let padded_len = payload_len
        .checked_add(3)
        .ok_or("longueur de paquet invalide")?
        & !3;
    if 20usize
        .checked_add(padded_len)
        .is_none_or(|end| end > body.len())
    {
        return Err(format!(
            "Paquet {} tronqué dans le bloc PCAPNG.",
            packets.len() + 1
        ));
    }
    packets.push(CapturedPacket {
        number: packets.len() as u64 + 1,
        section_id: 0,
        interface_id,
        local_address: None,
        link_type: interface.link_type,
        timestamp_ns: interface
            .timestamp_resolution
            .to_unix_nanoseconds(ticks, interface.timestamp_offset_seconds),
        timestamp: Some(PacketTimestamp {
            ticks,
            resolution: interface.timestamp_resolution.encoded(),
            offset_seconds: interface.timestamp_offset_seconds,
        }),
        captured_len,
        original_len,
        bytes: body[20..20 + payload_len].to_vec(),
    });
    Ok(())
}

fn parse_simple_packet(
    body: &[u8],
    order: ByteOrder,
    interfaces: &[Interface],
    packets: &mut Vec<CapturedPacket>,
) -> Result<(), String> {
    if body.len() < 4 {
        return Err("Simple Packet Block incomplet".into());
    }
    let interface = interfaces
        .first()
        .ok_or("Simple Packet Block sans interface déclarée")?;
    let original_len = order.u32(&body[0..4]);
    let captured_len = if interface.snaplen == 0 {
        original_len
    } else {
        original_len.min(interface.snaplen)
    };
    let payload_len = captured_len as usize;
    if payload_len > body.len() - 4 {
        return Err(format!(
            "Paquet {} tronqué dans le bloc PCAPNG.",
            packets.len() + 1
        ));
    }
    packets.push(CapturedPacket {
        number: packets.len() as u64 + 1,
        section_id: 0,
        interface_id: 0,
        local_address: None,
        link_type: interface.link_type,
        timestamp_ns: None,
        timestamp: None,
        captured_len,
        original_len,
        bytes: body[4..4 + payload_len].to_vec(),
    });
    Ok(())
}

fn parse_packet_block(
    body: &[u8],
    order: ByteOrder,
    interfaces: &[Interface],
    packets: &mut Vec<CapturedPacket>,
) -> Result<(), String> {
    if body.len() < 20 {
        return Err("Packet Block incomplet".into());
    }
    let interface_id = order.u16(&body[0..2]) as u32;
    let interface = interfaces
        .get(interface_id as usize)
        .ok_or_else(|| format!("Interface {interface_id} inexistante."))?;
    let ticks = ((order.u32(&body[4..8]) as u64) << 32) | order.u32(&body[8..12]) as u64;
    let captured_len = order.u32(&body[12..16]);
    let original_len = order.u32(&body[16..20]);
    let payload_len = captured_len as usize;
    let padded_len = payload_len
        .checked_add(3)
        .ok_or("longueur de paquet invalide")?
        & !3;
    if 20usize
        .checked_add(padded_len)
        .is_none_or(|end| end > body.len())
    {
        return Err(format!(
            "Paquet {} tronqué dans le bloc PCAPNG.",
            packets.len() + 1
        ));
    }
    packets.push(CapturedPacket {
        number: packets.len() as u64 + 1,
        section_id: 0,
        interface_id,
        local_address: None,
        link_type: interface.link_type,
        timestamp_ns: interface
            .timestamp_resolution
            .to_unix_nanoseconds(ticks, interface.timestamp_offset_seconds),
        timestamp: Some(PacketTimestamp {
            ticks,
            resolution: interface.timestamp_resolution.encoded(),
            offset_seconds: interface.timestamp_offset_seconds,
        }),
        captured_len,
        original_len,
        bytes: body[20..20 + payload_len].to_vec(),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn le_block(block_type: u32, body: &[u8]) -> Vec<u8> {
        let length = body.len() + 12;
        let mut block = Vec::new();
        block.extend_from_slice(&block_type.to_le_bytes());
        block.extend_from_slice(&(length as u32).to_le_bytes());
        block.extend_from_slice(body);
        block.extend_from_slice(&(length as u32).to_le_bytes());
        block
    }

    fn minimal_pcapng(packet: &[u8], timestamp_ticks: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut shb = Vec::new();
        shb.extend_from_slice(&0x1a2b3c4du32.to_le_bytes());
        shb.extend_from_slice(&1u16.to_le_bytes());
        shb.extend_from_slice(&0u16.to_le_bytes());
        shb.extend_from_slice(&(-1i64).to_le_bytes());
        bytes.extend(le_block(0x0a0d0d0a, &shb));
        let mut idb = Vec::new();
        idb.extend_from_slice(&1u16.to_le_bytes());
        idb.extend_from_slice(&0u16.to_le_bytes());
        idb.extend_from_slice(&65535u32.to_le_bytes());
        bytes.extend(le_block(1, &idb));
        let mut epb = Vec::new();
        epb.extend_from_slice(&0u32.to_le_bytes());
        epb.extend_from_slice(&((timestamp_ticks >> 32) as u32).to_le_bytes());
        epb.extend_from_slice(&(timestamp_ticks as u32).to_le_bytes());
        epb.extend_from_slice(&(packet.len() as u32).to_le_bytes());
        epb.extend_from_slice(&(packet.len() as u32).to_le_bytes());
        epb.extend_from_slice(packet);
        epb.resize(20 + ((packet.len() + 3) & !3), 0);
        bytes.extend(le_block(6, &epb));
        bytes
    }

    #[test]
    fn reads_enhanced_packet_and_normalizes_default_microsecond_timestamps() {
        let result = read(&minimal_pcapng(&[1, 2, 3], 1_234_567));
        assert_eq!(result.error, None);
        assert_eq!(result.packets.len(), 1);
        assert_eq!(result.packets[0].bytes, [1, 2, 3]);
        assert_eq!(result.packets[0].timestamp_ns, Some(1_234_567_000));
        assert_eq!(result.packets[0].link_type, LinkType::Ethernet);
    }

    #[test]
    fn reports_truncated_file_and_preserves_packets_read_before_it() {
        let mut bytes = minimal_pcapng(&[1, 2, 3], 0);
        bytes.extend_from_slice(&[0, 1, 2, 3]);
        let result = read(&bytes);
        assert_eq!(result.packets.len(), 1);
        assert!(result.is_partial());
    }

    #[test]
    fn rejects_an_untrusted_block_boundary_without_scanning_ahead() {
        let mut bytes = minimal_pcapng(&[1, 2, 3], 0);
        bytes.extend_from_slice(&[1, 0, 0, 0, 4, 0, 0, 0]);
        bytes.extend_from_slice(&[0xaa; 16]);
        let result = read(&bytes);
        assert_eq!(result.packets.len(), 1);
        assert!(result.error.unwrap().contains("Longueur de bloc invalide"));
    }

    #[test]
    fn switches_byte_order_and_interface_namespace_between_sections() {
        fn be_block(kind: u32, body: &[u8]) -> Vec<u8> {
            let length = (body.len() + 12) as u32;
            let mut bytes = kind.to_be_bytes().to_vec();
            bytes.extend(length.to_be_bytes());
            bytes.extend(body);
            bytes.extend(length.to_be_bytes());
            bytes
        }
        let mut bytes = minimal_pcapng(&[1, 2, 3], 1);
        let mut section = 0x1a2b3c4du32.to_be_bytes().to_vec();
        section.extend(1u16.to_be_bytes());
        section.extend(0u16.to_be_bytes());
        section.extend((-1i64).to_be_bytes());
        bytes.extend(be_block(0x0a0d0d0a, &section));
        let mut interface = 229u16.to_be_bytes().to_vec();
        interface.extend(0u16.to_be_bytes());
        interface.extend(65535u32.to_be_bytes());
        interface.extend([0, 9, 0, 1, 0x8a, 0, 0, 0]);
        interface.extend([0, 14, 0, 8]);
        interface.extend((-2i64).to_be_bytes());
        interface.extend([0; 4]);
        bytes.extend(be_block(1, &interface));
        let mut packet = Vec::new();
        for value in [0u32, 0, 1024, 3, 3] {
            packet.extend(value.to_be_bytes());
        }
        packet.extend([4, 5, 6, 0]);
        bytes.extend(be_block(6, &packet));
        let result = read(&bytes);
        assert_eq!(result.error, None);
        assert_eq!(result.packets.len(), 2);
        assert_eq!(result.packets[1].section_id, 1);
        assert_eq!(result.packets[1].interface_id, 0);
        assert_eq!(result.packets[1].link_type, LinkType::RawIpv6);
        assert_eq!(result.packets[1].timestamp_ns, Some(-1_000_000_000));
        assert_eq!(
            result.packets[1].timestamp,
            Some(PacketTimestamp {
                ticks: 1024,
                resolution: 0x8a,
                offset_seconds: -2
            })
        );
        assert_eq!(result.packets[1].bytes, [4, 5, 6]);
    }
}
