//! Layouts documented in base-doc/widgets/TRACKING_CUBES_CACHES.md.
//! Object-layout validation and the reference vector follow a2kit (MIT),
//! revision b1ffe3d6a1248bc26d906983f96896f99d740195; see LICENSES/a2kit-MIT.txt.
//! The alternate 3641 spawn and guessed player/loot positions are excluded.
use super::{
    hidden_cubes::Tracking,
    player_position::{cast_target, coordinates as position},
};
use crate::capture::widgets::{Consumer, entity};
use std::time::Instant;

#[derive(Default)]
pub(super) struct CubeConsumer {
    pub tracking: Tracking,
    local_entity: Option<u32>,
    pub sound: bool,
    pub guidance_enabled: bool,
    announcements: u64,
    capture_status: String,
    last_presence_event: Option<String>,
}
impl CubeConsumer {
    pub fn status(&self) -> String {
        if self.capture_status.is_empty() {
            return "En attente de synchronisation du flux du jeu".into();
        }
        let mut status = format!(
            "{} · {} annonces d’objets reçues · {} cube(s) détecté(s)",
            self.capture_status,
            self.announcements,
            self.tracking.entities().count()
        );
        if let Some(event) = &self.last_presence_event {
            status.push_str(" · Dernier événement de présence : ");
            status.push_str(event);
        }
        status
    }
    pub fn set_guidance(&mut self, enabled: bool) {
        if enabled != self.guidance_enabled {
            self.local_entity = None;
            self.tracking.clear_position();
        }
        self.guidance_enabled = enabled;
    }
}
impl Consumer for CubeConsumer {
    fn accepts(&self, opcode: [u8; 2], id: Option<u32>) -> bool {
        match opcode {
            [0x34, 0x36] | [0x23, 0x36] => true,
            [0x33, 0x36] => self.guidance_enabled,
            [0x02, 0x38] => self.guidance_enabled && self.local_entity.is_some(),
            [0x35, 0x36] | [0x3a, 0x36] | [0x42, 0x36] => {
                id.is_some_and(|id| self.tracking.entities().any(|tracked| tracked == id))
            }
            [0x1a..=0x1c, 0x37] => self.guidance_enabled && id.is_some() && id == self.local_entity,
            _ => false,
        }
    }
    fn message(&mut self, opcode: [u8; 2], payload: &[u8], now: Instant) {
        self.tracking.connected();
        self.capture_status = "Flux du jeu synchronisé".into();
        if opcode == [0x34, 0x36] {
            self.announcements = self.announcements.saturating_add(1);
        }
        // Only 23 36 is a documented zone boundary. The unrelated 21 36
        // position/identity candidate must never erase detected cubes.
        if opcode == [0x23, 0x36] {
            if self.tracking.entities().next().is_some() {
                self.last_presence_event = Some("Suivi effacé par le message de zone 23 36".into());
            }
            self.tracking.reset();
            // a2kit Zone: the zero entity tag identifies the local character;
            // another entity's zone message must not become our position.
            if self.guidance_enabled
                && opcode == [0x23, 0x36]
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
                if tail.len() < 6 {
                    return;
                }
                let mask = u16::from_le_bytes(tail[..2].try_into().unwrap());
                let kind = u32::from_le_bytes(tail[2..6].try_into().unwrap());
                let expected = 52 + usize::from(mask & 1 != 0) * 4 + usize::from(mask & 8 != 0) * 4;
                if mask & !0xd != 0
                    || tail.len() != expected
                    || tail.get(26..28) != Some(&[1, 1])
                    || tail.last() != Some(&0)
                {
                    if super::hidden_cubes::is_cube(kind) {
                        self.last_presence_event = Some(format!(
                            "Annonce de type cube rejetée : format 34 36 non reconnu (masque {mask:04x}, {} octets, {expected} attendus)",
                            tail.len()
                        ));
                    }
                    return;
                }
                if let Some(pos) = position(&tail[6..18]) {
                    if self.tracking.detect(id, kind, pos, now) {
                        self.last_presence_event = Some("Cube détecté et ajouté au suivi".into());
                        if self.sound {
                            super::notifications::play(
                                super::notifications::Notification::Detected,
                            );
                        }
                    }
                } else if super::hidden_cubes::is_cube(kind) {
                    self.last_presence_event =
                        Some("Annonce de type cube rejetée : coordonnées invalides".into());
                }
            }
            [0x33, 0x36] if self.guidance_enabled => {
                // Structured local-player identity prefix; never search arbitrary
                // packets for a character name or adopt the first moving entity.
                if payload.len() >= n + 5 {
                    if self.local_entity.is_some_and(|old| old != id) {
                        if self.tracking.entities().next().is_some() {
                            self.last_presence_event = Some(
                                "Suivi effacé par un changement d’identité locale 33 36".into(),
                            );
                        }
                        self.tracking.reset();
                    }
                    self.local_entity = Some(id);
                }
            }
            [0x1a..=0x1c, 0x37] if self.guidance_enabled && Some(id) == self.local_entity => {
                let offset = if opcode[0] == 0x1a {
                    n + 2
                } else {
                    let Some(flags) = payload.get(n) else {
                        return;
                    };
                    n + 1 + usize::from(flags & 1 != 0)
                };
                if let Some(pos) = payload.get(offset..offset + 12).and_then(position) {
                    self.tracking.position(pos, now);
                }
            }
            [0x02, 0x38] if self.guidance_enabled => {
                if let Some(local) = self.local_entity
                    && let Some(pos) = cast_target(payload, local)
                {
                    self.tracking.position(pos, now);
                }
            }
            [0x35, 0x36] | [0x3a, 0x36] | [0x42, 0x36] if payload.len() > n => {
                if self.tracking.entities().any(|tracked| tracked == id) {
                    self.last_presence_event = Some(format!(
                        "Cube retiré du suivi par le message {:02x} {:02x}",
                        opcode[0], opcode[1]
                    ));
                }
                self.tracking.remove(id);
            }
            _ => {}
        }
    }
    fn reset(&mut self, error: Option<&str>) {
        if self.tracking.entities().next().is_some() {
            self.last_presence_event = Some(match error {
                Some(error) => format!("Suivi effacé par une interruption : {error}"),
                None => "Suivi effacé par une resynchronisation du flux".into(),
            });
        }
        self.local_entity = None;
        self.announcements = 0;
        self.capture_status = error
            .map(|error| format!("Capture interrompue : {error}"))
            .unwrap_or_else(|| "Flux du jeu synchronisé".into());
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
    use crate::overlay::hidden_cubes::Position;
    const CUBE: &str = "e9 db 01 04 00 a7 4f 12 00 00 e0 79 c5 80 7d af 47 00 66 ac 46 00 00 00 00 00 00 00 00 01 01 01 dc 1a 0a 55 a1 01 00 00 00 00 00 00 00 00 00 00 24 4d c5 12 07 00 00";
    pub(super) fn fixture() -> Vec<u8> {
        CUBE.split_whitespace()
            .map(|s| u8::from_str_radix(s, 16).unwrap())
            .collect()
    }
    #[test]
    fn unvalidated_identity_candidate_does_not_erase_a_detected_cube() {
        let now = Instant::now();
        let mut state = CubeConsumer::default();
        state.set_guidance(true);
        state.message([0x34, 0x36], &fixture(), now);
        assert!(state.tracking.presence(now).is_some());
        assert!(!state.accepts([0x21, 0x36], Some(15943)));
        state.message([0x21, 0x36], &[0; 28], now);
        assert_eq!(state.tracking.entities().collect::<Vec<_>>(), vec![28137]);
        state.message([0x23, 0x36], &[0; 18], now);
        assert!(state.tracking.presence(now).is_none());
        assert!(
            state
                .status()
                .contains("effacé par le message de zone 23 36")
        );
    }
    #[test]
    fn diagnostics_distinguish_rejected_announcements_from_removed_cubes() {
        let now = Instant::now();
        let mut state = CubeConsumer::default();
        let mut malformed = fixture();
        malformed.push(0);
        state.message([0x34, 0x36], &malformed, now);
        assert!(state.tracking.presence(now).is_none());
        assert!(state.status().contains("Annonce de type cube rejetée"));
        state.message([0x34, 0x36], &fixture(), now);
        assert!(state.status().contains("1 cube(s) détecté(s)"));
        state.message([0x35, 0x36], &[0xe9, 0xdb, 1, 4], now);
        assert!(state.tracking.presence(now).is_none());
        assert!(
            state
                .status()
                .contains("retiré du suivi par le message 35 36")
        );
        state.reset(None);
        assert!(
            state
                .status()
                .contains("retiré du suivi par le message 35 36")
        );
        state.message([0x34, 0x36], &fixture(), now);
        state.reset(Some("Flux réseau incomplet"));
        state.reset(None);
        assert!(
            state
                .status()
                .contains("Suivi effacé par une interruption : Flux réseau incomplet")
        );
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
    fn guidance_subscriptions_attribute_cast_targets_and_release_on_disable() {
        let now = Instant::now();
        let mut state = CubeConsumer::default();
        assert!(!state.accepts([0x02, 0x38], Some(15943)));
        state.set_guidance(true);
        assert!(!state.accepts([0x02, 0x38], Some(15943)));
        state.message([0x33, 0x36], &[0xc7, 0x7c, 0, 0, 0, 0, 0], now);
        assert!(state.accepts([0x02, 0x38], Some(37365)));
        let hex = |value: &str| {
            value
                .split_whitespace()
                .map(|b| u8::from_str_radix(b, 16).unwrap())
                .collect::<Vec<_>>()
        };
        let self_cast = hex(
            "c7 7c 00 f0 4d a8 00 04 02 c7 7c 22 7f a4 c2 2d 71 c9 c6 fd 85 04 c7 b6 5b 8d 46 90 4e 01 00",
        );
        let cast_on_enemy = hex(
            "c7 7c 00 e0 26 a8 00 01 02 f5 a3 02 1a 4f 83 c2 e4 e0 c8 c6 19 85 05 c7 9e 27 8d 46 90 4e 01 00",
        );
        state.tracking.detect(
            1,
            1200039,
            Position {
                x: -25784.588 + 300.,
                y: -33925.99 + 400.,
                z: 18093.855 + 1200.,
            },
            now,
        );
        state.message([0x02, 0x38], &self_cast, now);
        let guidance = state.tracking.guidance(now).unwrap();
        assert_eq!(guidance.horizontal_m, 5.);
        assert_eq!(guidance.elevation_m, 12.);
        let later = now + std::time::Duration::from_secs(3);
        state.message([0x02, 0x38], &cast_on_enemy, later);
        assert_eq!(
            state
                .tracking
                .guidance(now + super::super::hidden_cubes::POSITION_TTL),
            None
        );
        state.message([0x02, 0x38], &self_cast, later);
        state.set_guidance(false);
        assert!(!state.accepts([0x02, 0x38], Some(15943)));
        assert!(!state.accepts([0x1a, 0x37], Some(15943)));
        assert_eq!(state.tracking.guidance(later), None);
        assert!(state.tracking.presence(later).is_some());
    }
    #[test]
    fn movement_variants_update_only_the_known_local_entity() {
        let now = Instant::now();
        let mut state = CubeConsumer::default();
        state.set_guidance(true);
        state.message([0x33, 0x36], &[0xe4, 0x72, 0, 0, 0, 0, 0], now);
        state.tracking.detect(
            1,
            1200039,
            Position {
                x: -24170.584,
                y: -42095.004,
                z: 17953.207,
            },
            now,
        );
        let movement = "e4 72 05 03 2b d5 bc c6 01 6f 24 c7 6a 42 8c 46 75 11 9e 5b"
            .split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect::<Vec<_>>();
        assert!(state.accepts([0x1b, 0x37], Some(14692)));
        state.message([0x1b, 0x37], &movement, now);
        assert_eq!(state.tracking.guidance(now).unwrap().horizontal_m, 0.);
        let foreign = [0xc7, 0x7c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        state.message([0x1b, 0x37], &foreign, now);
        assert_eq!(state.tracking.guidance(now).unwrap().horizontal_m, 0.);
        state.reset(None);
        assert!(!state.accepts([0x1b, 0x37], Some(14692)));
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
