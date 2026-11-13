//! Attributed position sources, independent of capture and rendering.
//! Cast layout and reference vectors: a2kit b1ffe3d6 (MIT, LICENSES/a2kit-MIT.txt).
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
