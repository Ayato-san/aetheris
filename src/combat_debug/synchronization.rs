//! Mid-connection recognition from repeated clear server ticks, not TCP ports.
use super::compression::frame_length;

const MAX_SCAN_BYTES: usize = 256 * 1024;
const MAX_FRAME_STEPS: usize = 1_000_000;
const MAX_CONFIRMATION_FRAMES: usize = 512;
const MIN_UNIX_MS: u64 = 946_684_800_000; // 2000-01-01
const MAX_UNIX_MS: u64 = 4_102_444_800_000; // 2100-01-01

/// A hypothesis supported by three increasing server ticks on valid frame
/// boundaries. The first tick is the boundary; earlier bytes are not decoded.
pub(super) fn server_boundary(stream: &[u8]) -> Option<usize> {
    let mut budget = MAX_FRAME_STEPS;
    for start in 0..stream.len().min(MAX_SCAN_BYTES) {
        let Some(candidate) = stream.get(start..start + 11) else {
            break;
        };
        let Some(mut previous) = tick(candidate) else {
            continue;
        };
        let mut offset = start + 11;
        let mut ticks = 1;
        for _ in 0..MAX_CONFIRMATION_FRAMES {
            budget = budget.checked_sub(1)?;
            while stream.get(offset) == Some(&0) {
                budget = budget.checked_sub(1)?;
                offset += 1;
            }
            let remaining = stream.get(offset..)?;
            let Ok(Some((length, _))) = frame_length(remaining) else {
                break;
            };
            let Some(bytes) = remaining.get(..length) else {
                break;
            };
            if let Some(time) = tick(bytes) {
                if time <= previous || time - previous > 60_000 {
                    break;
                }
                previous = time;
                ticks += 1;
                if ticks == 3 {
                    return Some(start);
                }
            }
            offset += length;
        }
    }
    None
}

fn tick(bytes: &[u8]) -> Option<u64> {
    if bytes.len() != 11 || !bytes.starts_with(&[0x0e, 0, 0x36]) {
        return None;
    }
    let time = u64::from_le_bytes(bytes[3..11].try_into().ok()?);
    (MIN_UNIX_MS..MAX_UNIX_MS).contains(&time).then_some(time)
}

/// Exclude identifiable TLS records before attempting any AION alignment.
pub(super) fn is_tls(stream: &[u8]) -> bool {
    stream.len() >= 5
        && matches!(stream[0], 0x14..=0x17)
        && stream[1] == 3
        && stream[2] <= 4
        && (1..=18_432).contains(&u16::from_be_bytes([stream[3], stream[4]]))
}
