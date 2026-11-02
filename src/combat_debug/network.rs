//! Packet parsing only; no operating-system capture or UI dependencies.
use super::pcapng::{CapturedPacket, LinkType};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

#[derive(Clone, Debug)]
pub struct TcpSegment {
    pub sequence: u32,
    pub syn: bool,
    pub ack: bool,
    pub fin: bool,
    pub rst: bool,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct PacketInfo {
    pub source: Option<SocketAddr>,
    pub destination: Option<SocketAddr>,
    pub protocol: String,
    pub issue: Option<String>,
    pub tcp: Option<TcpSegment>,
}

impl PacketInfo {
    fn unsupported(reason: impl Into<String>) -> Self {
        Self {
            protocol: "Inconnu".into(),
            issue: Some(reason.into()),
            ..Self::default()
        }
    }
}

pub fn parse(packet: &CapturedPacket) -> PacketInfo {
    if packet.bytes.len() != packet.captured_len as usize
        || packet.captured_len > packet.original_len
    {
        return PacketInfo::unsupported("Longueurs de capture incohérentes");
    }
    let bytes = &packet.bytes;
    let ip = match packet.link_type {
        LinkType::Ethernet => {
            if bytes.len() < 14 {
                return PacketInfo::unsupported("En-tête Ethernet tronqué");
            }
            let mut ether_type = u16::from_be_bytes([bytes[12], bytes[13]]);
            let mut start = 14;
            while matches!(ether_type, 0x8100 | 0x88a8) {
                if bytes.len() < start + 4 {
                    return PacketInfo::unsupported("En-tête VLAN tronqué");
                }
                ether_type = u16::from_be_bytes([bytes[start + 2], bytes[start + 3]]);
                start += 4;
            }
            if !matches!(ether_type, 0x0800 | 0x86dd) {
                return PacketInfo::unsupported(format!(
                    "EtherType {ether_type:04x} non pris en charge"
                ));
            }
            &bytes[start..]
        }
        LinkType::RawIp | LinkType::RawIpv4 | LinkType::RawIpv6 => bytes,
        LinkType::Other(number) => {
            return PacketInfo::unsupported(format!("Liaison {number} non prise en charge"));
        }
    };
    let Some(version) = ip.first().map(|byte| byte >> 4) else {
        return PacketInfo::unsupported("Paquet IP vide");
    };
    match version {
        4 => ipv4(ip, packet.is_truncated()),
        6 => ipv6(ip, packet.is_truncated()),
        _ => PacketInfo::unsupported(format!("Version IP {version} inconnue")),
    }
}

fn ipv4(bytes: &[u8], captured_truncated: bool) -> PacketInfo {
    if bytes.len() < 20 {
        return PacketInfo::unsupported("En-tête IPv4 tronqué");
    }
    let header = (bytes[0] as usize & 15) * 4;
    let total = u16::from_be_bytes([bytes[2], bytes[3]]) as usize;
    if header < 20 || header > bytes.len() || total < header {
        return PacketInfo::unsupported("Longueur IPv4 invalide");
    }
    let source = IpAddr::V4(Ipv4Addr::new(bytes[12], bytes[13], bytes[14], bytes[15]));
    let destination = IpAddr::V4(Ipv4Addr::new(bytes[16], bytes[17], bytes[18], bytes[19]));
    let fragment = u16::from_be_bytes([bytes[6], bytes[7]]);
    if fragment & 0x3fff != 0 {
        return PacketInfo {
            source: Some(SocketAddr::new(source, 0)),
            destination: Some(SocketAddr::new(destination, 0)),
            protocol: "IPv4".into(),
            issue: Some("Fragment IP non pris en charge".into()),
            tcp: None,
        };
    }
    transport(
        source,
        destination,
        bytes[9],
        &bytes[header..total.min(bytes.len())],
        captured_truncated || total > bytes.len(),
    )
}

fn ipv6(bytes: &[u8], captured_truncated: bool) -> PacketInfo {
    if bytes.len() < 40 {
        return PacketInfo::unsupported("En-tête IPv6 tronqué");
    }
    let source = IpAddr::V6(Ipv6Addr::from(<[u8; 16]>::try_from(&bytes[8..24]).unwrap()));
    let destination = IpAddr::V6(Ipv6Addr::from(
        <[u8; 16]>::try_from(&bytes[24..40]).unwrap(),
    ));
    let payload_len = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    if payload_len == 0 {
        return PacketInfo::unsupported("Jumbogramme IPv6 non pris en charge");
    }
    let total = 40 + payload_len;
    let available = total.min(bytes.len());
    let mut next = bytes[6];
    let mut offset = 40usize;
    // Each step consumes at least 8 bytes. Bounds limit extension chains.
    while matches!(next, 0 | 43 | 60 | 51) {
        if offset + 2 > available {
            return PacketInfo::unsupported("Extension IPv6 tronquée");
        }
        let length = if next == 51 {
            (bytes[offset + 1] as usize + 2) * 4
        } else {
            (bytes[offset + 1] as usize + 1) * 8
        };
        next = bytes[offset];
        offset += length;
        if offset > available {
            return PacketInfo::unsupported("Longueur d’extension IPv6 invalide");
        }
    }
    if next == 44 {
        return PacketInfo {
            source: Some(SocketAddr::new(source, 0)),
            destination: Some(SocketAddr::new(destination, 0)),
            protocol: "IPv6".into(),
            issue: Some("Fragment IP non pris en charge".into()),
            tcp: None,
        };
    }
    transport(
        source,
        destination,
        next,
        &bytes[offset..available],
        captured_truncated || total > bytes.len(),
    )
}

fn transport(
    source: IpAddr,
    destination: IpAddr,
    protocol: u8,
    bytes: &[u8],
    truncated: bool,
) -> PacketInfo {
    let mut info = PacketInfo {
        source: Some(SocketAddr::new(source, 0)),
        destination: Some(SocketAddr::new(destination, 0)),
        protocol: match protocol {
            6 => "TCP",
            17 => "UDP",
            _ => "IP",
        }
        .into(),
        ..PacketInfo::default()
    };
    if !matches!(protocol, 6 | 17) {
        info.issue = Some(format!("Protocole IP {protocol} non pris en charge"));
        return info;
    }
    if bytes.len() < 4 {
        info.issue = Some("Ports de transport tronqués".into());
        return info;
    }
    info.source = Some(SocketAddr::new(
        source,
        u16::from_be_bytes([bytes[0], bytes[1]]),
    ));
    info.destination = Some(SocketAddr::new(
        destination,
        u16::from_be_bytes([bytes[2], bytes[3]]),
    ));
    if truncated {
        info.issue = Some("Paquet tronqué : octets manquants".into());
    }
    if protocol == 17 {
        if bytes.len() < 8 {
            info.issue = Some("En-tête UDP tronqué".into());
        } else {
            let length = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
            if length < 8 || length > bytes.len() {
                info.issue = Some("Longueur UDP invalide ou tronquée".into());
            }
        }
        return info;
    }
    if bytes.len() < 20 {
        info.issue = Some("En-tête TCP tronqué".into());
        return info;
    }
    let header = (bytes[12] as usize >> 4) * 4;
    if header < 20 || header > bytes.len() {
        info.issue = Some("Longueur d’en-tête TCP invalide".into());
        return info;
    }
    info.tcp = Some(TcpSegment {
        sequence: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
        syn: bytes[13] & 0x02 != 0,
        ack: bytes[13] & 0x10 != 0,
        fin: bytes[13] & 0x01 != 0,
        rst: bytes[13] & 0x04 != 0,
        payload: bytes[header..].to_vec(),
    });
    info
}
