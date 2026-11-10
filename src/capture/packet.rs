//! Original packet evidence shared by the service and offline analysis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LinkType {
    Ethernet,
    RawIp,
    RawIpv4,
    RawIpv6,
    Other(u16),
}

impl LinkType {
    pub fn from_pcap(value: u16) -> Self {
        match value {
            1 => Self::Ethernet,
            101 => Self::RawIp,
            228 => Self::RawIpv4,
            229 => Self::RawIpv6,
            value => Self::Other(value),
        }
    }

    pub fn number(self) -> u16 {
        match self {
            Self::Ethernet => 1,
            Self::RawIp => 101,
            Self::RawIpv4 => 228,
            Self::RawIpv6 => 229,
            Self::Other(value) => value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CapturedPacket {
    pub number: u64,
    /// PCAPNG section, or live process generation. Interface IDs are scoped here.
    pub section_id: u32,
    pub interface_id: u32,
    /// Known live capture-host address; never inferred from private IP ranges.
    pub local_address: Option<std::net::IpAddr>,
    pub link_type: LinkType,
    /// Timestamp normalized to nanoseconds since the Unix epoch, if present.
    pub timestamp_ns: Option<i128>,
    pub timestamp: Option<PacketTimestamp>,
    pub captured_len: u32,
    pub original_len: u32,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PacketTimestamp {
    pub ticks: u64,
    /// PCAPNG if_tsresol encoding: high bit selects base two, otherwise base ten.
    pub resolution: u8,
    pub offset_seconds: i64,
}

impl CapturedPacket {
    pub fn is_truncated(&self) -> bool {
        self.captured_len < self.original_len
    }
}
