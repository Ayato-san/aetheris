//! Attributed position sources, independent of capture and rendering.
//! Cast layout and reference vectors: a2kit b1ffe3d6 (MIT, LICENSES/a2kit-MIT.txt).
//! Map-load/correction/effect layouts: Daevas 32e57ea3 (MIT, LICENSES/Aion-DPS-Meter-MIT.txt).
use super::hidden_cubes::Position;
use crate::capture::widgets::entity;

pub(super) fn coordinates(bytes: &[u8]) -> Option<Position> {
    let read = |offset| {
        Some(f32::from_le_bytes(
            bytes.get(offset..offset + 4)?.try_into().ok()?,
        ))
    };
    let position = Position {
        x: read(0)?,
        y: read(4)?,
        z: read(8)?,
    };
    position.valid().then_some(position)
}
pub(super) fn map_load(payload: &[u8]) -> Option<(u32, Option<Position>)> {
    if payload.len() < 28 {
        return None;
    }
    let zone = u32::from_le_bytes(payload[4..8].try_into().ok()?);
    (zone > 0 && zone <= i32::MAX as u32).then(|| (zone, coordinates(&payload[16..28])))
}
/// 3c 37 is an entity-attributed correction, not a continuous local movement feed.
pub(super) fn correction(payload: &[u8], local: u32) -> Option<Position> {
    let (entity, width) = entity(payload)?;
    if entity != local || payload.get(width) != Some(&3) {
        return None;
    }
    coordinates(payload.get(width + 1..width + 13)?)
}
struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Option<&'a [u8]> {
        let value = self.0.get(..length)?;
        self.0 = &self.0[length..];
        Some(value)
    }
    fn byte(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn var(&mut self) -> Option<u32> {
        let (value, width) = entity(self.0)?;
        self.take(width)?;
        Some(value)
    }
}
/// The supported effect layout is the MIT Daevas SelfCheckCombatOverlay
/// vector: target | [on: 01] | type | instance | effect/duration/stays/clock
/// | caster | 01 00 | XYZ. Both identities must be local: sources disagree
/// on attribution for effects between different entities. No tail scanning.
pub(super) fn self_effect(payload: &[u8], local: u32, renewed: bool) -> Option<Position> {
    let mut r = Reader(payload);
    if r.var()? != local || local == 0 {
        return None;
    }
    if !renewed && r.byte()? != 1 {
        return None;
    }
    r.byte()?; // type, fixed-width in this layout
    r.var()?; // instance
    let effect = u32::from_le_bytes(r.take(4)?.try_into().ok()?);
    if !(10..=2_999_999_999).contains(&effect) {
        return None;
    }
    r.take(16)?; // duration, stays, server clock
    if r.var()? != local || r.take(2)? != [1, 0] {
        return None;
    }
    let position = coordinates(r.take(12)?)?;
    r.0.is_empty().then_some(position)
}
/// 02 38 coordinates belong to the target, even when the caster is local.
/// Do not refresh our position when the local player casts at somebody else.
pub(super) fn cast_target(payload: &[u8], local: u32) -> Option<Position> {
    let mut r = Reader(payload);
    if r.var()? == 0 {
        return None;
    }
    let flags = r.byte()?;
    if flags & !0x0c != 0 {
        return None;
    }
    r.take(5)?; // skill and cast sequence
    if r.byte()? != 2 || r.var()? != local {
        return None;
    }
    r.take(4)?; // target heading, never used as player/camera orientation
    let position = coordinates(r.take(12)?)?;
    r.var()?; // actor speed
    if !matches!(r.byte()?, 1 | 2) {
        return None;
    }
    r.var()?;
    if flags & 4 != 0 {
        r.byte()?;
    }
    if flags & 8 != 0 {
        r.var()?;
    }
    r.0.is_empty().then_some(position)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn hex(value: &str) -> Vec<u8> {
        value
            .split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect()
    }
    #[test]
    fn map_load_layout_rejects_truncation_invalid_z_and_invalid_zone() {
        // Synthetic layout vector: no complete captured 21 36 fixture was available.
        let mut payload = vec![0; 28];
        payload[..4].copy_from_slice(&999u32.to_le_bytes());
        payload[4..8].copy_from_slice(&1010u32.to_le_bytes());
        for (offset, value) in [(16, 59953f32), (20, 17085.), (24, 22067.)] {
            payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        assert_eq!(
            map_load(&payload),
            Some((
                1010,
                Some(Position {
                    x: 59953.,
                    y: 17085.,
                    z: 22067.
                })
            ))
        );
        for end in 0..payload.len() {
            assert_eq!(map_load(&payload[..end]), None);
        }
        payload[24..28].copy_from_slice(&f32::NAN.to_le_bytes());
        assert_eq!(map_load(&payload), Some((1010, None)));
        payload[4..8].fill(0);
        assert_eq!(map_load(&payload), None);
    }
    #[test]
    fn captured_correction_requires_matching_identity_marker_and_complete_xyz() {
        // Daevas SelfCheckReplay, MIT. The original test's actor is not local;
        // assigning it as local here checks attribution, not own-update cadence.
        let mut payload = hex(
            "d1 9a 01 03 76 d2 3a 47 71 04 b0 46 00 40 03 47 14 46 e9 c2 07 02 01 a4 aa f5 00 ef a6 f6 5f a4 03",
        );
        let pos = correction(&payload, 19793).unwrap();
        assert!((pos.x - 47826.46).abs() < 0.01);
        assert!((pos.y - 22530.22).abs() < 0.01);
        assert_eq!(pos.z, 33600.);
        assert_eq!(correction(&payload, 19794), None);
        for end in 0..16 {
            assert_eq!(correction(&payload[..end], 19793), None);
        }
        payload[3] = 0;
        assert_eq!(correction(&payload, 19793), None);
    }
    #[test]
    fn self_effect_reference_rejects_foreign_participants_and_partial_or_unknown_layouts() {
        let payload = hex(
            "b6 28 01 11 37 c8 00 00 00 2c 01 00 00 00 00 00 00 7e 62 57 2c a1 01 00 00 b6 28 01 00 c8 fb 3a 47 34 11 b4 46 00 1b 03 47",
        );
        let expected = Some(Position {
            x: 47867.78,
            y: 23048.602,
            z: 33563.,
        });
        assert_eq!(self_effect(&payload, 5174, false), expected);
        let mut renewed = payload.clone();
        renewed.remove(2);
        assert_eq!(self_effect(&renewed, 5174, true), expected);
        for end in 0..payload.len() {
            assert_eq!(self_effect(&payload[..end], 5174, false), None);
        }
        for offset in [0, 25] {
            let mut foreign = payload.clone();
            foreign[offset] = 0xb7;
            assert_eq!(self_effect(&foreign, 5174, false), None);
        }
        let mut invalid = payload.clone();
        invalid.push(0);
        assert_eq!(self_effect(&invalid, 5174, false), None);
        invalid = payload.clone();
        invalid[27] = 3;
        assert_eq!(self_effect(&invalid, 5174, false), None);
        invalid = payload;
        invalid[37..41].copy_from_slice(&f32::NAN.to_le_bytes());
        assert_eq!(self_effect(&invalid, 5174, false), None);
    }
    #[test]
    fn real_cast_positions_follow_the_target_and_reject_every_truncation() {
        let self_cast = hex(
            "c7 7c 00 f0 4d a8 00 04 02 c7 7c 22 7f a4 c2 2d 71 c9 c6 fd 85 04 c7 b6 5b 8d 46 90 4e 01 00",
        );
        let enemy_cast = hex(
            "f5 a3 02 00 74 c9 12 00 01 02 c7 7c 1a ea eb 42 cd cd c9 c6 21 9d 04 c7 00 4c 8d 46 90 4e 01 e8 07",
        );
        let cast_on_enemy = hex(
            "c7 7c 00 e0 26 a8 00 01 02 f5 a3 02 1a 4f 83 c2 e4 e0 c8 c6 19 85 05 c7 9e 27 8d 46 90 4e 01 00",
        );
        assert_eq!(
            cast_target(&self_cast, 15943),
            Some(Position {
                x: -25784.588,
                y: -33925.99,
                z: 18093.855
            })
        );
        assert_eq!(
            cast_target(&enemy_cast, 15943),
            Some(Position {
                x: -25830.9,
                y: -33949.13,
                z: 18086.
            })
        );
        assert_eq!(cast_target(&cast_on_enemy, 15943), None);
        assert_eq!(cast_target(&self_cast, 37365), None);
        for end in 0..self_cast.len() {
            assert_eq!(cast_target(&self_cast[..end], 15943), None);
        }
        let mut extra = self_cast.clone();
        extra.push(0);
        assert_eq!(cast_target(&extra, 15943), None);
        let no_position = hex("e4 72 00 d8 62 d6 00 0b 00 e4 72 6d d9 90 43 90 4e 01 00");
        assert_eq!(cast_target(&no_position, 14692), None);
    }
}
