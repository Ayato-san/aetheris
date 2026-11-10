//! Bounded local IPC. No request accepts paths, PIDs, commands or database names.
use super::{CaptureBatch, CapturedPacket, Interface, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{Read, Write};

pub(super) const VERSION: u32 = 1;
pub(super) const MAX_FRAME: usize = 512 * 1024;
const MAX_BATCH: usize = 32 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum Request {
    Hello {
        version: u32,
        interface: Option<Interface>,
    },
    Poll,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum Response {
    Ready {
        version: u32,
    },
    Batch {
        status: String,
        count: usize,
        finished: bool,
        error: Option<String>,
    },
    Error {
        message: String,
    },
}

pub(super) fn write<T: Serialize>(stream: &mut impl Write, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > MAX_FRAME {
        return Err("Message du service de capture trop volumineux.".into());
    }
    stream.write_all(&(bytes.len() as u32).to_le_bytes())?;
    stream.write_all(&bytes)?;
    Ok(())
}

pub(super) fn read<T: DeserializeOwned>(stream: &mut impl Read, limit: usize) -> Result<T> {
    let mut size = [0; 4];
    stream.read_exact(&mut size)?;
    let size = u32::from_le_bytes(size) as usize;
    if size == 0 || size > limit {
        return Err("Taille de message du service de capture invalide.".into());
    }
    let mut bytes = vec![0; size];
    stream.read_exact(&mut bytes)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn validate(packet: &CapturedPacket) -> Result<()> {
    if packet.bytes.len() > 65535
        || packet.bytes.len() != packet.captured_len as usize
        || packet.captured_len > packet.original_len
    {
        return Err("Paquet du service de capture invalide.".into());
    }
    Ok(())
}

pub(super) fn write_batch(stream: &mut impl Write, batch: &CaptureBatch) -> Result<()> {
    if batch.packets.len() > 65536
        || batch.packets.iter().map(|p| p.bytes.len()).sum::<usize>() > MAX_BATCH
    {
        return Err("Lot du service de capture trop volumineux.".into());
    }
    for packet in &batch.packets {
        validate(packet)?;
    }
    write(
        stream,
        &Response::Batch {
            status: batch.status.clone(),
            count: batch.packets.len(),
            finished: batch.finished,
            error: batch.error.clone(),
        },
    )?;
    for packet in &batch.packets {
        write(stream, packet)?;
    }
    Ok(())
}

pub(super) fn read_batch(stream: &mut impl Read) -> Result<CaptureBatch> {
    let Response::Batch {
        status,
        count,
        finished,
        error,
    } = read(stream, 4096)?
    else {
        return Err("Réponse inattendue du service de capture.".into());
    };
    if count > 65536 {
        return Err("Trop de paquets annoncés par le service de capture.".into());
    }
    let mut batch = CaptureBatch {
        packets: Vec::new(),
        status,
        finished,
        error,
    };
    let mut total = 0usize;
    for _ in 0..count {
        let packet = (|| -> Result<CapturedPacket> {
            let packet: CapturedPacket = read(stream, MAX_FRAME)?;
            validate(&packet)?;
            total += packet.bytes.len();
            if total > MAX_BATCH {
                return Err("Lot du service de capture trop volumineux.".into());
            }
            Ok(packet)
        })();
        match packet {
            Ok(packet) => batch.packets.push(packet),
            Err(error) => {
                // A broken connection must not discard packets already received.
                batch.finished = true;
                batch.error = Some(format!(
                    "Connexion au service de capture interrompue : {error}"
                ));
                break;
            }
        }
    }
    Ok(batch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::{LinkType, PacketTimestamp};
    use std::io::Cursor;

    fn packet() -> CapturedPacket {
        CapturedPacket {
            number: 17,
            section_id: 2,
            interface_id: 3,
            local_address: Some("10.0.0.1".parse().unwrap()),
            link_type: LinkType::RawIpv4,
            timestamp_ns: Some(1_000_000_000),
            timestamp: Some(PacketTimestamp {
                ticks: 1_000_000_000,
                resolution: 9,
                offset_seconds: 0,
            }),
            captured_len: 65535,
            original_len: 65535,
            bytes: vec![255; 65535],
        }
    }

    #[test]
    fn transports_original_evidence_and_final_errors_without_interpretation() {
        let packet = packet();
        let batch = CaptureBatch {
            packets: vec![packet.clone()],
            status: "Capture IPv4".into(),
            finished: true,
            error: Some("source stopped".into()),
        };
        let mut wire = Vec::new();
        write_batch(&mut wire, &batch).unwrap();
        let received = read_batch(&mut Cursor::new(wire)).unwrap();
        assert_eq!(received.packets, vec![packet]);
        assert_eq!(received.error, batch.error);
        assert!(received.finished);
    }

    #[test]
    fn broken_batch_retains_complete_packets_and_reports_partial_capture() {
        let mut wire = Vec::new();
        write(
            &mut wire,
            &Response::Batch {
                status: "live".into(),
                count: 2,
                finished: false,
                error: None,
            },
        )
        .unwrap();
        write(&mut wire, &packet()).unwrap();
        wire.extend(120u32.to_le_bytes());
        wire.extend(b"truncated");
        let received = read_batch(&mut Cursor::new(wire)).unwrap();
        assert_eq!(received.packets, vec![packet()]);
        assert!(received.finished);
        assert!(received.error.unwrap().contains("interrompue"));
    }

    #[test]
    fn rejects_unbounded_frames_counts_and_arbitrary_requests() {
        let mut hello = Vec::new();
        write(
            &mut hello,
            &Request::Hello {
                version: VERSION,
                interface: None,
            },
        )
        .unwrap();
        assert!(matches!(
            read(&mut Cursor::new(hello), 4096).unwrap(),
            Request::Hello {
                version: VERSION,
                interface: None
            }
        ));
        assert!(read::<Request>(&mut Cursor::new(u32::MAX.to_le_bytes()), 4096).is_err());
        assert!(
            serde_json::from_str::<Request>(
                r#"{"Hello":{"version":1,"interface":null,"path":"C:\\Windows"}}"#
            )
            .is_err()
        );
        let mut wire = Vec::new();
        write(
            &mut wire,
            &Response::Batch {
                status: "live".into(),
                count: usize::MAX,
                finished: false,
                error: None,
            },
        )
        .unwrap();
        assert!(read_batch(&mut Cursor::new(wire)).is_err());
        let mut invalid = packet();
        invalid.captured_len = 1;
        let batch = CaptureBatch {
            packets: vec![invalid],
            status: "".into(),
            finished: false,
            error: None,
        };
        assert!(write_batch(&mut Vec::new(), &batch).is_err());
    }
}
