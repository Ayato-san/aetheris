//! Little-endian PCAPNG output: SHB, IDB, EPB and untimed SPB packets.
use super::{CapturedPacket, PacketTimestamp};
use std::io::{self, Write};

#[derive(PartialEq, Eq)]
struct InterfaceKey {
    source_id: u32,
    link_type: u16,
    resolution: u8,
    offset: i64,
}

pub struct Writer<W> {
    output: W,
    section: Option<u32>,
    interfaces: Vec<InterfaceKey>,
}

impl<W: Write> Writer<W> {
    pub fn new(output: W) -> io::Result<Self> {
        let mut writer = Self {
            output,
            section: None,
            interfaces: Vec::new(),
        };
        writer.start_section()?;
        Ok(writer)
    }

    fn block(&mut self, kind: u32, body: &[u8]) -> io::Result<()> {
        let length = u32::try_from(
            body.len()
                .checked_add(12)
                .ok_or_else(|| io::Error::other("Bloc PCAPNG trop grand"))?,
        )
        .map_err(io::Error::other)?;
        self.output.write_all(&kind.to_le_bytes())?;
        self.output.write_all(&length.to_le_bytes())?;
        self.output.write_all(body)?;
        self.output.write_all(&length.to_le_bytes())
    }

    fn start_section(&mut self) -> io::Result<()> {
        let mut body = 0x1a2b3c4du32.to_le_bytes().to_vec();
        body.extend(1u16.to_le_bytes());
        body.extend(0u16.to_le_bytes());
        body.extend((-1i64).to_le_bytes());
        self.interfaces.clear();
        self.block(0x0a0d0d0a, &body)
    }

    pub fn write_packet(&mut self, packet: &CapturedPacket) -> io::Result<()> {
        if packet.bytes.len() != packet.captured_len as usize {
            return Err(io::Error::other(
                "Longueur capturée incohérente avec les octets du paquet",
            ));
        }
        let timestamp = match packet.timestamp {
            Some(timestamp) => Some(timestamp),
            None => packet
                .timestamp_ns
                .map(|ns| {
                    if let Ok(ticks) = u64::try_from(ns) {
                        return Ok(PacketTimestamp {
                            ticks,
                            resolution: 9,
                            offset_seconds: 0,
                        });
                    }
                    // A negative epoch requires a signed interface offset, not a fabricated date.
                    let offset_seconds =
                        i64::try_from(ns.div_euclid(1_000_000_000)).map_err(io::Error::other)?;
                    Ok::<_, io::Error>(PacketTimestamp {
                        ticks: ns.rem_euclid(1_000_000_000) as u64,
                        resolution: 9,
                        offset_seconds,
                    })
                })
                .transpose()?,
        };
        if timestamp.is_none() && packet.captured_len == 0 && packet.original_len != 0 {
            return Err(io::Error::other(
                "Un paquet sans horodatage et entièrement tronqué ne peut pas être exporté en SPB",
            ));
        }
        // SPB implicitly uses interface zero and derives captured length from snaplen.
        // Isolate untimed packets so neither timestamps nor padding bytes are invented.
        if self.section != Some(packet.section_id) || timestamp.is_none() {
            if self.section.is_some() {
                self.start_section()?;
            }
            self.section = Some(packet.section_id);
        }
        let key = InterfaceKey {
            source_id: packet.interface_id,
            link_type: packet.link_type.number(),
            resolution: timestamp.map_or(6, |time| time.resolution),
            offset: timestamp.map_or(0, |time| time.offset_seconds),
        };
        let interface_id = match self.interfaces.iter().position(|existing| *existing == key) {
            Some(id) => id as u32,
            None => {
                let id = self.interfaces.len() as u32;
                let mut body = key.link_type.to_le_bytes().to_vec();
                body.extend(0u16.to_le_bytes());
                body.extend(
                    if timestamp.is_none() {
                        packet.captured_len
                    } else {
                        0
                    }
                    .to_le_bytes(),
                );
                body.extend([9, 0, 1, 0, key.resolution, 0, 0, 0]);
                body.extend([14, 0, 8, 0]);
                body.extend(key.offset.to_le_bytes());
                body.extend([0; 4]);
                self.block(1, &body)?;
                self.interfaces.push(key);
                id
            }
        };
        let mut body = Vec::with_capacity(packet.bytes.len() + 24);
        let kind = if let Some(timestamp) = timestamp {
            body.extend(interface_id.to_le_bytes());
            body.extend(((timestamp.ticks >> 32) as u32).to_le_bytes());
            body.extend((timestamp.ticks as u32).to_le_bytes());
            body.extend(packet.captured_len.to_le_bytes());
            body.extend(packet.original_len.to_le_bytes());
            6
        } else {
            body.extend(packet.original_len.to_le_bytes());
            3
        };
        body.extend_from_slice(&packet.bytes);
        body.resize(body.len().next_multiple_of(4), 0);
        self.block(kind, &body)?;
        if timestamp.is_none() {
            // The next packet must not inherit the SPB's snaplen or interface zero.
            self.start_section()?;
            self.section = None;
        }
        Ok(())
    }

    pub fn finish(mut self) -> io::Result<()> {
        self.output.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat_debug::pcapng::{LinkType, read};

    fn packet(number: u64) -> CapturedPacket {
        CapturedPacket {
            number,
            section_id: 0,
            interface_id: 7,
            local_address: None,
            link_type: LinkType::RawIpv4,
            timestamp_ns: Some(number as i128),
            timestamp: None,
            captured_len: 3,
            original_len: 30,
            bytes: vec![1, 2, number as u8],
        }
    }

    #[test]
    fn round_trip_preserves_evidence_across_interfaces_sections_and_timestamp_formats() {
        let mut packets: Vec<_> = (1..=8).map(packet).collect();
        packets[1].interface_id = 42;
        packets[1].link_type = LinkType::Ethernet;
        packets[1].timestamp = Some(PacketTimestamp {
            ticks: u64::MAX,
            resolution: 0x8a,
            offset_seconds: -2,
        });
        packets[1].timestamp_ns = Some((u64::MAX as i128 * 1_000_000_000 / 1024) - 2_000_000_000);
        packets[2].section_id = 1;
        packets[2].link_type = LinkType::RawIpv6;
        packets[3].section_id = 1;
        packets[3].timestamp_ns = None;
        packets[4].section_id = 1;
        packets[4].timestamp_ns = Some(-1);
        packets[5].section_id = 1;
        packets[5].link_type = LinkType::Other(147);
        packets[6].timestamp_ns = None;
        packets[6].captured_len = 0;
        packets[6].original_len = 0;
        packets[6].bytes.clear();
        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output).unwrap();
        for packet in &packets {
            writer.write_packet(packet).unwrap();
        }
        writer.finish().unwrap();
        let imported = read(&output);
        assert_eq!(imported.error, None);
        assert_eq!(imported.packets.len(), packets.len());
        for (actual, expected) in imported.packets.iter().zip(&packets) {
            assert_eq!(actual.number, expected.number);
            assert_eq!(actual.bytes, expected.bytes);
            assert_eq!(actual.captured_len, expected.captured_len);
            assert_eq!(actual.original_len, expected.original_len);
            assert_eq!(actual.timestamp_ns, expected.timestamp_ns);
            assert_eq!(actual.link_type, expected.link_type);
            if expected.timestamp.is_some() {
                assert_eq!(actual.timestamp, expected.timestamp);
            }
        }
        assert_ne!(
            imported.packets[0].interface_id,
            imported.packets[1].interface_id
        );
        assert_ne!(
            imported.packets[1].section_id,
            imported.packets[2].section_id
        );
    }

    #[test]
    fn empty_export_is_a_valid_section_and_io_failures_are_reported() {
        let mut output = Vec::new();
        Writer::new(&mut output).unwrap().finish().unwrap();
        assert_eq!(read(&output).error, None);
        assert!(read(&output).packets.is_empty());
        let mut too_small = [0u8; 4];
        assert!(Writer::new(too_small.as_mut_slice()).is_err());
        let mut writer = Writer::new(Vec::new()).unwrap();
        let mut invalid = packet(1);
        invalid.captured_len = 4;
        assert!(writer.write_packet(&invalid).is_err());
    }
}
