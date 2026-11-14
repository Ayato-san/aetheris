//! Layouts documented in base-doc/widgets/TRACKING_CUBES_CACHES.md.
//! Object-layout validation and the reference vector follow a2kit (MIT),
//! revision b1ffe3d6a1248bc26d906983f96896f99d740195; see LICENSES/a2kit-MIT.txt.
//! The alternate 3641 spawn and guessed player/loot positions are excluded.
use super::{
    hidden_cubes::Tracking,
    player_position::{cast_target, coordinates as position, correction, map_load, self_effect},
};
use crate::capture::widgets::{Consumer, entity};
use std::time::Instant;

#[derive(Default)]
pub(super) struct CubeConsumer {
    pub tracking: Tracking,
    local_entity: Option<u32>,
    loaded_world: bool,
    zone: Option<u32>,
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
            self.tracking.clear_position();
        }
        self.guidance_enabled = enabled;
    }
    #[cfg(windows)]
    pub fn has_local_identity(&self) -> bool {
        self.local_entity.is_some()
    }
}
impl Consumer for CubeConsumer {
    fn accepts(&self, opcode: [u8; 2], id: Option<u32>) -> bool {
        match opcode {
            [0x34, 0x36] | [0x21, 0x36] | [0x23, 0x36] | [0x33, 0x36] => true,
            [0x02, 0x38] => self.guidance_enabled && self.local_entity.is_some(),
            [0x2a..=0x2b, 0x38] => self.guidance_enabled && id.is_some() && id == self.local_entity,
            [0x35, 0x36] | [0x3a, 0x36] | [0x42, 0x36] => {
                id.is_some_and(|id| self.tracking.entities().any(|tracked| tracked == id))
            }
            [0x1a..=0x1c, 0x37] | [0x3c, 0x37] => {
                self.guidance_enabled && id.is_some() && id == self.local_entity
            }
            _ => false,
        }
    }
    fn message(&mut self, opcode: [u8; 2], payload: &[u8], now: Instant) {
        self.tracking.connected();
        self.capture_status = "Flux du jeu synchronisé".into();
        if opcode == [0x34, 0x36] {
            self.announcements = self.announcements.saturating_add(1);
        }
        // 21 36 starts with a counter and map ID, never an entity varint.
        // Its local XYZ prefix is independently documented by Daevas and Nyerk.
        if opcode == [0x21, 0x36] {
            if let Some((zone, pos)) = map_load(payload) {
                // A zone report can repeat while standing still. Only a change
                // from a previously observed zone invalidates the world; the
                // first report establishes a baseline for a mid-session capture.
                if self.zone.is_some_and(|previous| previous != zone) {
                    if self.tracking.entities().next().is_some() {
                        self.last_presence_event =
                            Some("Suivi effacé par un changement de zone confirmé 21 36".into());
                    }
                    self.tracking.reset();
                    self.local_entity = None;
                    self.loaded_world = true;
                } else if self.zone.is_none() && self.local_entity.is_none() {
                    self.loaded_world = true;
                }
                self.zone = Some(zone);
                if self.guidance_enabled
                    && let Some(pos) = pos
                {
                    self.tracking.position(pos, now);
                }
            }
            return;
        }
        if opcode == [0x23, 0x36] {
            // a2kit rejects zone reports for another entity. Validate the
            // local tag and complete XYZ before applying a destructive boundary.
            if payload.first() != Some(&0) {
                return;
            }
            let Some(pos) = payload.get(6..18).and_then(position) else {
                return;
            };
            self.zone = None;
            self.local_entity = None;
            self.loaded_world = true;
            if self.tracking.entities().next().is_some() {
                self.last_presence_event = Some("Suivi effacé par le message de zone 23 36".into());
            }
            self.tracking.reset();
            if self.guidance_enabled {
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
            [0x33, 0x36] => {
                // Structured local-player identity prefix; never search arbitrary
                // packets for a character name or adopt the first moving entity.
                if payload.len() >= n + 5 {
                    // Presence belongs to a character even with guidance off.
                    // If capture began mid-session, the first confirmed Self
                    // cannot attribute previously seen cubes to this character.
                    if self.local_entity != Some(id)
                        && (self.local_entity.is_some()
                            || (!self.loaded_world && self.tracking.entities().next().is_some()))
                    {
                        if self.tracking.entities().next().is_some() {
                            self.last_presence_event = Some(
                                "Suivi effacé à la confirmation d’une nouvelle identité locale 33 36".into(),
                            );
                        }
                        self.tracking.reset();
                    }
                    self.local_entity = Some(id);
                    self.loaded_world = false;
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
            [0x2a..=0x2b, 0x38] if self.guidance_enabled => {
                if let Some(local) = self.local_entity
                    && let Some(pos) = self_effect(payload, local, opcode[0] == 0x2b)
                {
                    self.tracking.position(pos, now);
                }
            }
            [0x3c, 0x37] if self.guidance_enabled => {
                if let Some(local) = self.local_entity
                    && let Some(pos) = correction(payload, local)
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
        self.zone = None;
        self.loaded_world = false;
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
    fn malformed_map_load_does_not_erase_a_detected_cube() {
        let now = Instant::now();
        let mut state = CubeConsumer::default();
        state.set_guidance(true);
        state.message([0x34, 0x36], &fixture(), now);
        assert!(state.tracking.presence(now).is_some());
        assert!(state.accepts([0x21, 0x36], Some(15943)));
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
    fn compressed_map_load_seeds_position_before_identity_and_correction_refreshes_it() {
        use crate::capture::widgets::{
            Decoder,
            tests::{compressed, frame, tcp, ticks},
        };
        use std::{
            sync::{Arc, Mutex},
            time::Duration,
        };
        let now = Instant::now();
        let mut consumer = CubeConsumer::default();
        consumer.set_guidance(true);
        consumer.zone = Some(1009);
        consumer.message([0x33, 0x36], &[10, 0, 0, 0, 0, 0], now);
        consumer.message([0x34, 0x36], &fixture(), now);
        let state = Arc::new(Mutex::new(consumer));
        let clients: Vec<Arc<Mutex<dyn Consumer>>> = vec![state.clone()];
        let mut load = vec![0x21, 0x36];
        load.extend_from_slice(&[0; 28]);
        load[6..10].copy_from_slice(&1010u32.to_le_bytes());
        for (offset, value) in [(18, -3998f32), (22, 89851.), (26, 22067.)] {
            load[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        let cube = [&[0x34, 0x36][..], &fixture()].concat();
        let wire = [
            ticks(),
            compressed(
                &[
                    frame(&load),
                    frame(&cube),
                    frame(&[0x33, 0x36, 11, 0, 0, 0, 0, 0]),
                ]
                .concat(),
            ),
        ]
        .concat();
        let mut decoder = Decoder::default();
        decoder.feed(&tcp(100, 0x18, &wire), now, &clients);
        let mut state = state.lock().unwrap();
        assert_eq!(state.local_entity, Some(11));
        assert_eq!(state.tracking.entities().count(), 1);
        assert_eq!(state.tracking.guidance(now).unwrap().horizontal_m, 0.);
        let later = now + Duration::from_secs(5);
        assert!(state.tracking.guidance(later).is_none());
        let mut correction = vec![11, 3];
        correction.extend_from_slice(&load[18..30]);
        assert!(state.accepts([0x3c, 0x37], Some(11)));
        assert!(!state.accepts([0x3c, 0x37], Some(12)));
        state.message([0x3c, 0x37], &correction, later);
        assert_eq!(state.tracking.guidance(later).unwrap().horizontal_m, 0.);
        // A later character switch still clears the newly loaded world.
        state.message([0x33, 0x36], &[12, 0, 0, 0, 0, 0], later);
        assert!(state.tracking.presence(later).is_none());
        assert!(state.tracking.guidance(later).is_none());
    }
    #[test]
    fn repeated_zone_reports_keep_cubes_timer_and_identity_until_zone_really_changes() {
        let now = Instant::now();
        for enabled in [false, true] {
            let mut state = CubeConsumer::default();
            state.set_guidance(enabled);
            state.message([0x33, 0x36], &[11, 0, 0, 0, 0, 0], now);
            state.message([0x34, 0x36], &fixture(), now);
            let mut report = vec![0; 28];
            report[4..8].copy_from_slice(&1010u32.to_le_bytes());
            // First observation establishes a baseline, even after a cube.
            state.message([0x21, 0x36], &report, now);
            for seconds in [1, 5, 70] {
                let later = now + std::time::Duration::from_secs(seconds);
                state.message([0x21, 0x36], &report, later);
                assert_eq!(state.local_entity, Some(11));
                assert_eq!(state.tracking.entities().collect::<Vec<_>>(), vec![28137]);
                assert!(!state.status().contains("changement de zone"));
            }
            assert!(
                state
                    .tracking
                    .presence(now + std::time::Duration::from_secs(70))
                    .unwrap()
                    .contains("1 min 10 s")
            );
            report[4..8].copy_from_slice(&1011u32.to_le_bytes());
            state.message([0x21, 0x36], &report, now);
            assert_eq!(state.local_entity, None);
            assert!(state.tracking.presence(now).is_none());
            assert!(state.status().contains("changement de zone confirmé"));
            state.message([0x34, 0x36], &fixture(), now);
            state.message([0x33, 0x36], &[12, 0, 0, 0, 0, 0], now);
            assert!(state.tracking.presence(now).is_some());
            state.reset(None);
            assert_eq!(state.zone, None);
        }
    }
    #[test]
    fn self_effect_refreshes_guidance_only_while_subscribed_and_identified() {
        let now = Instant::now();
        let mut state = CubeConsumer::default();
        state.set_guidance(true);
        assert!(!state.accepts([0x2a, 0x38], Some(5174)));
        state.message([0x33, 0x36], &[0xb6, 0x28, 0, 0, 0, 0, 0], now);
        state.message([0x34, 0x36], &fixture(), now);
        assert!(state.accepts([0x2a, 0x38], Some(5174)));
        assert!(state.accepts([0x2b, 0x38], Some(5174)));
        assert!(!state.accepts([0x2a, 0x38], Some(5175)));
        let effect = "b6 28 01 11 37 c8 00 00 00 2c 01 00 00 00 00 00 00 7e 62 57 2c a1 01 00 00 b6 28 01 00 c8 fb 3a 47 34 11 b4 46 00 1b 03 47".split_whitespace().map(|b| u8::from_str_radix(b, 16).unwrap()).collect::<Vec<_>>();
        state.message([0x2a, 0x38], &effect, now);
        assert!(state.tracking.guidance(now).is_some());
        state.set_guidance(false);
        assert!(!state.accepts([0x2a, 0x38], Some(5174)));
        state.message([0x2a, 0x38], &effect, now);
        assert!(state.tracking.guidance(now).is_none());
    }
    #[test]
    fn foreign_and_incomplete_zone_messages_do_not_invalidate_the_local_world() {
        let now = Instant::now();
        let mut state = CubeConsumer::default();
        state.set_guidance(true);
        state.message([0x33, 0x36], &[11, 0, 0, 0, 0, 0], now);
        state.message([0x34, 0x36], &fixture(), now);
        state.tracking.position(
            Position {
                x: 1.,
                y: 2.,
                z: 3.,
            },
            now,
        );
        // a2kit's explicit rejected other-entity zone fixture.
        let foreign = "c7 7c 00 00 00 00 00 7e 87 ca c6 82 a0 03 c7 00 50 8d 46"
            .split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect::<Vec<_>>();
        state.message([0x23, 0x36], &foreign, now);
        for length in 0..18 {
            state.message([0x23, 0x36], &vec![0; length], now);
        }
        let mut invalid = [0; 18];
        invalid[14..18].copy_from_slice(&f32::NAN.to_le_bytes());
        state.message([0x23, 0x36], &invalid, now);
        assert_eq!(state.local_entity, Some(11));
        assert!(state.tracking.presence(now).is_some());
        assert!(state.tracking.guidance(now).is_some());
        state.message([0x23, 0x36], &[0; 18], now);
        assert!(state.tracking.presence(now).is_none());
        assert_eq!(state.local_entity, None);
        state.message([0x34, 0x36], &fixture(), now);
        state.message([0x33, 0x36], &[12, 0, 0, 0, 0, 0], now);
        assert!(state.tracking.guidance(now).is_some());
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
    fn character_changes_clear_remaining_cubes_with_or_without_guidance() {
        use crate::capture::widgets::{
            Decoder,
            tests::{frame, tcp, ticks},
        };
        use std::sync::{Arc, Mutex};
        for guidance in [false, true] {
            for identity_seen in [false, true] {
                let now = Instant::now();
                let mut consumer = CubeConsumer::default();
                consumer.set_guidance(guidance);
                let state = Arc::new(Mutex::new(consumer));
                let clients: Vec<Arc<Mutex<dyn Consumer>>> = vec![state.clone()];
                let identity = [0x33, 0x36, 10, 0, 0, 0, 0, 0];
                let cube = [&[0x34, 0x36][..], &fixture()].concat();
                let mut other_cube = cube.clone();
                other_cube[2] = 0xea; // distinct entity, same valid object body
                let wire = [
                    ticks(),
                    if identity_seen {
                        frame(&identity)
                    } else {
                        vec![]
                    },
                    frame(&cube),
                    frame(&other_cube),
                ]
                .concat();
                let mut decoder = Decoder::default();
                decoder.feed(&tcp(100, 0x18, &wire), now, &clients);
                assert_eq!(state.lock().unwrap().tracking.entities().count(), 2);
                let removed = frame(&[0x35, 0x36, 0xe9, 0xdb, 1, 4]);
                let next = 100 + wire.len() as u32;
                decoder.feed(&tcp(next, 0x18, &removed), now, &clients);
                assert_eq!(state.lock().unwrap().tracking.entities().count(), 1);
                assert!(
                    state
                        .lock()
                        .unwrap()
                        .status()
                        .contains("retiré du suivi par le message 35 36")
                );
                let next = next + removed.len() as u32;
                let new_character = frame(&[0x33, 0x36, 11, 0, 0, 0, 0, 0]);
                decoder.feed(&tcp(next, 0x18, &new_character), now, &clients);
                assert_eq!(state.lock().unwrap().tracking.entities().count(), 0);
                assert!(state.lock().unwrap().tracking.presence(now).is_none());
                assert!(state.lock().unwrap().tracking.guidance(now).is_none());
                // A new announcement starts a fresh instance; repeated Self
                // for the same character must not erase it or its timer.
                let next = next + new_character.len() as u32;
                let wire = [frame(&other_cube), new_character].concat();
                let later = now + std::time::Duration::from_secs(70);
                decoder.feed(&tcp(next, 0x18, &wire), later, &clients);
                assert_eq!(state.lock().unwrap().tracking.entities().count(), 1);
                assert!(
                    state
                        .lock()
                        .unwrap()
                        .tracking
                        .presence(later)
                        .unwrap()
                        .contains("0 min 00 s")
                );
            }
        }
    }
    #[test]
    fn switching_guidance_preserves_the_character_boundary() {
        let now = Instant::now();
        let mut state = CubeConsumer::default();
        state.message([0x33, 0x36], &[10, 0, 0, 0, 0, 0], now);
        state.message([0x34, 0x36], &fixture(), now);
        state.set_guidance(true);
        state.set_guidance(false);
        state.message([0x33, 0x36], &[11, 0, 0, 0, 0, 0], now);
        assert!(state.tracking.presence(now).is_none());
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
        assert!(decoder.tracking.presence(now).is_some());
        assert_eq!(decoder.tracking.guidance(now), None);
    }
}
