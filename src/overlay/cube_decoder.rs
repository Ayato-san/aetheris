//! Layouts documented in base-doc/widgets/TRACKING_CUBES_CACHES.md.
//! Object-layout validation and the reference vector follow a2kit (MIT),
//! revision b1ffe3d6a1248bc26d906983f96896f99d740195; see LICENSES/a2kit-MIT.txt.
//! The alternate 3641 spawn and guessed player/loot positions are excluded.
use super::hidden_cubes::{Position, Tracking};
use crate::capture::widgets::{Consumer, entity};
use std::time::Instant;

#[derive(Default)]
pub(super) struct CubeConsumer {
    pub tracking: Tracking,
    local_entity: Option<u32>,
    pub sound: bool,
}
fn position(bytes: &[u8]) -> Option<Position> {
    let read = |offset| {
        Some(f32::from_le_bytes(
            bytes.get(offset..offset + 4)?.try_into().ok()?,
        ))
    };
    let pos = Position {
        x: read(0)?,
        y: read(4)?,
        z: read(8)?,
    };
    pos.valid().then_some(pos)
}
impl Consumer for CubeConsumer {
    fn accepts(&self, opcode: [u8; 2], id: Option<u32>) -> bool {
        match opcode {
            [0x34, 0x36] | [0x23, 0x36] | [0x21, 0x36] | [0x33, 0x36] => true,
            [0x35, 0x36] | [0x3a, 0x36] | [0x42, 0x36] => {
                id.is_some_and(|id| self.tracking.entities().any(|tracked| tracked == id))
            }
            [0x1a, 0x37] => id.is_some() && id == self.local_entity,
            _ => false,
        }
    }
    fn message(&mut self, opcode: [u8; 2], payload: &[u8], now: Instant) {
        self.tracking.connected();
        if matches!(opcode, [0x23, 0x36] | [0x21, 0x36]) {
            self.tracking.reset();
            // a2kit Zone: the zero entity tag identifies the local character;
            // another entity's zone message must not become our position.
            if opcode == [0x23, 0x36]
                && payload.first() == Some(&0)
                && let Some(pos) = payload.get(6..18).and_then(position)
            {
                self.tracking.position(pos, now);
            }
            return;
        }
        let Some((id, n)) = entity(payload) else {
            return;
        };
        if id == 0 {
            return;
        }
        match opcode {
            [0x34, 0x36] => {
                let Some(tail) = payload.get(n..) else {
                    return;
                };
                // Validate the complete object layout, including optional mask
                // fields, before using the identity/coordinates prefix.
                if tail.len() < 2 {
                    return;
                }
                let mask = u16::from_le_bytes(tail[..2].try_into().unwrap());
                let expected = 52 + usize::from(mask & 1 != 0) * 4 + usize::from(mask & 8 != 0) * 4;
                if mask & !0xd != 0
                    || tail.len() != expected
                    || tail.get(26..28) != Some(&[1, 1])
                    || tail.last() != Some(&0)
                {
                    return;
                }
                let kind = u32::from_le_bytes(tail[2..6].try_into().unwrap());
                if let Some(pos) = position(&tail[6..18])
                    && self.tracking.detect(id, kind, pos, now)
                    && self.sound
                {
                    super::notifications::play(super::notifications::Notification::Detected);
                }
            }
            [0x33, 0x36] => {
                // Structured local-player identity prefix; never search arbitrary
                // packets for a character name or adopt the first moving entity.
                if payload.len() >= n + 5 {
                    if self.local_entity.is_some_and(|old| old != id) {
                        self.tracking.reset();
                    }
                    self.local_entity = Some(id);
                }
            }
            [0x1a, 0x37] if Some(id) == self.local_entity => {
                if let Some(pos) = payload.get(n + 2..n + 14).and_then(position) {
                    self.tracking.position(pos, now);
                }
            }
            [0x35, 0x36] | [0x3a, 0x36] | [0x42, 0x36] if payload.len() > n => {
                self.tracking.remove(id);
            }
            _ => {}
        }
    }
    fn reset(&mut self, error: Option<&str>) {
        self.local_entity = None;
        self.tracking.reset();
        match error {
            Some(error) => self.tracking.interrupt(error),
            None => self.tracking.connected(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const CUBE: &str = "e9 db 01 04 00 a7 4f 12 00 00 e0 79 c5 80 7d af 47 00 66 ac 46 00 00 00 00 00 00 00 00 01 01 01 dc 1a 0a 55 a1 01 00 00 00 00 00 00 00 00 00 00 24 4d c5 12 07 00 00";
    pub(super) fn fixture() -> Vec<u8> {
        CUBE.split_whitespace()
            .map(|s| u8::from_str_radix(s, 16).unwrap())
            .collect()
    }
    #[test]
    fn a2kit_reference_object_and_repeated_announcements() {
        let now = Instant::now();
        let mut decoder = CubeConsumer::default();
        let bytes = fixture();
        decoder.message([0x34, 0x36], &bytes, now);
        assert_eq!(decoder.tracking.entities().collect::<Vec<_>>(), vec![28137]);
        decoder.tracking.position(
            Position {
                x: -3998.,
                y: 89851.,
                z: 22067.,
            },
            now,
        );
        let guidance = decoder.tracking.guidance(now).unwrap();
        assert_eq!(guidance.horizontal_m, 0.);
        assert_eq!(guidance.elevation_m, 0.);
        decoder.message(
            [0x34, 0x36],
            &bytes,
            now + std::time::Duration::from_secs(2),
        );
        assert!(
            decoder
                .tracking
                .presence(now + std::time::Duration::from_secs(3))
                .unwrap()
                .contains("0 min 03 s")
        );
        assert!(decoder.accepts([0x35, 0x36], Some(28137)));
        decoder.message([0x35, 0x36], &[0xe9, 0xdb, 1, 4], now);
        assert!(!decoder.accepts([0x35, 0x36], Some(28137)));
        assert_eq!(decoder.tracking.presence(now), None);
    }
    #[test]
    fn reference_cube_survives_compression_splits_reordering_and_reannouncements() {
        use crate::capture::widgets::{
            Consumer, Decoder,
            tests::{compressed, frame, tcp, ticks},
        };
        use std::sync::{Arc, Mutex};
        let now = Instant::now();
        let state = Arc::new(Mutex::new(CubeConsumer::default()));
        let clients: Vec<Arc<Mutex<dyn Consumer>>> = vec![state.clone()];
        let body = [&[0x34, 0x36][..], &fixture()].concat();
        let wire = [ticks(), compressed(&frame(&body))].concat();
        for split in 1..wire.len() {
            let mut decoder = Decoder::default();
            decoder.feed(&tcp(99, 0x12, &[]), now, &clients);
            decoder.feed(
                &tcp(100 + split as u32, 0x18, &wire[split..]),
                now,
                &clients,
            );
            decoder.feed(&tcp(100, 0x18, &wire[..split]), now, &clients);
            decoder.feed(&tcp(100, 0x18, &wire), now, &clients);
            assert_eq!(
                state
                    .lock()
                    .unwrap()
                    .tracking
                    .entities()
                    .collect::<Vec<_>>(),
                vec![28137]
            );
            state.lock().unwrap().tracking.position(
                Position {
                    x: -3998.,
                    y: 89851.,
                    z: 22067.,
                },
                now,
            );
            assert_eq!(
                state
                    .lock()
                    .unwrap()
                    .tracking
                    .guidance(now)
                    .unwrap()
                    .horizontal_m,
                0.
            );
            state.lock().unwrap().reset(None);
        }
    }
    #[test]
    fn malformed_objects_loot_and_foreign_movement_never_guide() {
        let now = Instant::now();
        let mut decoder = CubeConsumer::default();
        for len in 0..fixture().len() {
            decoder.message([0x34, 0x36], &fixture()[..len], now);
        }
        assert_eq!(decoder.tracking.presence(now), None);
        let mut bytes = fixture();
        bytes[9..13].copy_from_slice(&f32::NAN.to_le_bytes());
        decoder.message([0x34, 0x36], &bytes, now);
        assert_eq!(decoder.tracking.presence(now), None);
        assert!(!decoder.accepts([0x1a, 0x37], Some(123)));
        assert!(!decoder.accepts([0x1e, 0x56], Some(123)));
        decoder.message([0x34, 0x36], &fixture(), now);
        decoder.message([0x23, 0x36], &[1; 18], now);
        assert_eq!(decoder.tracking.presence(now), None);
        assert_eq!(decoder.tracking.guidance(now), None);
    }
}
