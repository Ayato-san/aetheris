//! One bounded, incremental S2C pipeline for active overlay consumers. Raw
//! Combat Debug subscribers continue to receive the original packets separately.
use super::{CapturedPacket, network};
use crate::combat_debug::{analysis::read_varuint, compression::frame_length, synchronization};
use std::{
    collections::BTreeMap,
    net::SocketAddr,
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const MAX_BUFFER: usize = 1024 * 1024;
const MAX_FLOWS: usize = 8;
const MAX_PENDING: usize = 128;
const MAX_OUTPUT: usize = 8 * 1024 * 1024;
const GAP_TTL: Duration = Duration::from_secs(3);

pub trait Consumer: Send {
    /// Entity filters are evaluated immediately before delivery, so an appearance
    /// can subscribe to its removal even within the same compressed envelope.
    fn accepts(&self, opcode: [u8; 2], entity: Option<u32>) -> bool;
    fn message(&mut self, opcode: [u8; 2], payload: &[u8], now: Instant);
    fn reset(&mut self, error: Option<&str>);
}
type Client = Arc<Mutex<dyn Consumer>>;
struct Hub {
    clients: Mutex<Vec<Weak<Mutex<dyn Consumer>>>>,
    closed: AtomicBool,
}
static HUB: OnceLock<Mutex<Weak<Hub>>> = OnceLock::new();
pub struct Subscription {
    _hub: Arc<Hub>,
    _client: Client,
}

pub fn subscribe(client: Client) -> Subscription {
    let mut current = HUB.get_or_init(Default::default).lock().unwrap();
    let mut start = false;
    let hub = current
        .upgrade()
        .filter(|hub| !hub.closed.load(Ordering::Acquire))
        .unwrap_or_else(|| {
            let hub = Arc::new(Hub {
                clients: Mutex::new(Vec::new()),
                closed: AtomicBool::new(false),
            });
            *current = Arc::downgrade(&hub);
            start = true;
            hub
        });
    hub.clients.lock().unwrap().push(Arc::downgrade(&client));
    if start {
        let worker_hub = hub.clone();
        std::thread::spawn(move || run(worker_hub));
    }
    Subscription {
        _hub: hub,
        _client: client,
    }
}
impl Drop for Subscription {
    fn drop(&mut self) {
        self._hub
            .clients
            .lock()
            .unwrap()
            .retain(|client| !client.ptr_eq(&Arc::downgrade(&self._client)));
    }
}
fn run(hub: Arc<Hub>) {
    let result = (|| -> super::Result<()> {
        let source = super::shared::subscribe(None)?;
        let mut decoder = Decoder::default();
        loop {
            {
                let _current = HUB.get_or_init(Default::default).lock().unwrap();
                if Arc::strong_count(&hub) == 1 {
                    hub.closed.store(true, Ordering::Release);
                    break;
                }
            }
            let batch = source.next_batch()?;
            let clients: Vec<_> = {
                let mut clients = hub.clients.lock().unwrap();
                clients.retain(|client| client.strong_count() > 0);
                clients.iter().filter_map(Weak::upgrade).collect()
            };
            if clients.is_empty() {
                continue;
            }
            if let Some(batch) = batch {
                if let Some(error) = &batch.error {
                    return Err(error.clone().into());
                }
                if batch.finished {
                    return Err("Capture interrompue".into());
                }
                for packet in &batch.packets {
                    decoder.feed(packet, Instant::now(), &clients);
                }
            }
            decoder.expire(Instant::now(), &clients);
        }
        Ok(())
    })();
    let _current = HUB.get_or_init(Default::default).lock().unwrap();
    hub.closed.store(true, Ordering::Release);
    let clients = hub.clients.lock().unwrap();
    for client in clients.iter().filter_map(Weak::upgrade) {
        client
            .lock()
            .unwrap()
            .reset(result.as_ref().err().map(|e| e.to_string()).as_deref());
    }
}
fn reset(clients: &[Client], error: Option<&str>) {
    for client in clients {
        client.lock().unwrap().reset(error);
    }
}
#[derive(Default)]
struct Flow {
    next: Option<u32>,
    initial_sequence: Option<u32>,
    excluded: bool,
    pending: BTreeMap<u32, Vec<u8>>,
    buffer: Vec<u8>,
    history: Vec<u8>,
    synchronized: bool,
    gap: Option<Instant>,
    last_activity: Option<Instant>,
}
// Anchor wall timestamps once, then measure live age on the monotonic clock.
// Replaying a queued batch must not make an old position fresh again.
struct CaptureClock {
    wall_ns: i128,
    monotonic: Instant,
}
impl Default for CaptureClock {
    fn default() -> Self {
        Self {
            wall_ns: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as i128)
                .unwrap_or(0),
            monotonic: Instant::now(),
        }
    }
}
impl CaptureClock {
    fn received(&self, timestamp: Option<i128>, now: Instant) -> Option<Instant> {
        let Some(timestamp) = timestamp else {
            return Some(now);
        };
        let delta = timestamp.checked_sub(self.wall_ns)?;
        let duration = Duration::from_nanos(u64::try_from(delta.unsigned_abs()).ok()?);
        let received = if delta >= 0 {
            self.monotonic.checked_add(duration)?
        } else {
            self.monotonic.checked_sub(duration)?
        };
        (received <= now).then_some(received)
    }
}
type FlowKey = (u32, u32, SocketAddr, SocketAddr);
#[derive(Default)]
pub(crate) struct Decoder {
    flows: BTreeMap<FlowKey, Flow>,
    active: Option<FlowKey>,
    section: Option<u32>,
    clock: CaptureClock,
}
impl Decoder {
    fn expire(&mut self, now: Instant, clients: &[Client]) {
        let interrupted =
            self.active
                .and_then(|key| self.flows.get(&key))
                .and_then(|flow| {
                    if flow
                        .gap
                        .is_some_and(|t| now.saturating_duration_since(t) >= GAP_TTL)
                    {
                        Some("Flux réseau incomplet")
                    } else if flow.last_activity.is_some_and(|t| {
                        now.saturating_duration_since(t) >= Duration::from_secs(10)
                    }) {
                        Some("Trafic du jeu interrompu")
                    } else {
                        None
                    }
                });
        if let Some(reason) = interrupted {
            self.flows.clear();
            self.active = None;
            reset(clients, Some(reason));
        }
        self.flows.retain(|key, flow| {
            self.active == Some(*key)
                || !(flow
                    .gap
                    .is_some_and(|t| now.saturating_duration_since(t) >= GAP_TTL)
                    || flow.last_activity.is_some_and(|t| {
                        now.saturating_duration_since(t) >= Duration::from_secs(10)
                    }))
        });
    }
    pub(crate) fn feed(&mut self, packet: &CapturedPacket, now: Instant, clients: &[Client]) {
        let parsed = network::parse(packet);
        let (Some(source), Some(destination)) = (parsed.source, parsed.destination) else {
            return;
        };
        // No guessing from ports, private address ranges, loot or C2S ciphertext.
        if packet.local_address != Some(destination.ip()) || source.ip() == destination.ip() {
            return;
        }
        if self
            .section
            .is_some_and(|section| section != packet.section_id)
        {
            self.flows.clear();
            self.active = None;
            reset(clients, None);
        }
        self.section = Some(packet.section_id);
        let key = (packet.section_id, packet.interface_id, source, destination);
        if packet.is_truncated() {
            self.flows.remove(&key);
            if self.active == Some(key) {
                self.flows.clear();
                self.active = None;
                reset(clients, Some("Paquet réseau tronqué"));
            }
            return;
        }
        let Some(segment) = parsed.tcp else {
            return;
        };
        // Bind entity identities to one synchronized game connection. Traffic
        // from another endpoint must never enter the current world's registry.
        // A SYN can belong to another connection. Keep it as a candidate;
        // only validated game synchronization may replace the active world.
        if self.active.is_some_and(|active| active != key)
            && !segment.syn
            && !self
                .flows
                .get(&key)
                .is_some_and(|flow| flow.initial_sequence.is_some())
        {
            return;
        }
        if !self.flows.contains_key(&key) && self.flows.len() >= MAX_FLOWS {
            return;
        }
        if segment.syn
            && !self
                .flows
                .get(&key)
                .is_some_and(|flow| flow.initial_sequence == Some(segment.sequence))
        {
            if self.flows.get(&key).is_some_and(|f| f.synchronized) {
                reset(clients, None);
            }
            self.flows.insert(
                key,
                Flow {
                    next: Some(segment.sequence.wrapping_add(1)),
                    initial_sequence: Some(segment.sequence),
                    ..Flow::default()
                },
            );
        }
        let Some(received) = self.clock.received(packet.timestamp_ns, now) else {
            self.flows.remove(&key);
            if self.active == Some(key) {
                self.flows.clear();
                self.active = None;
                reset(clients, Some("Horodatage réseau incohérent"));
            }
            return;
        };
        let flow = self.flows.entry(key).or_default();
        flow.last_activity = Some(now);
        let result = flow.feed(
            segment.sequence.wrapping_add(u32::from(segment.syn)),
            &segment.payload,
            now,
            received,
            clients,
        );
        if let Err(error) = result {
            let affects_world = flow.synchronized || self.active == Some(key);
            self.flows.remove(&key);
            if affects_world {
                self.flows.clear();
                self.active = None;
                reset(clients, Some(error));
            }
        } else if segment.fin || segment.rst {
            let active = flow.synchronized || self.active == Some(key);
            self.flows.remove(&key);
            if active {
                self.flows.clear();
                self.active = None;
                reset(clients, Some("Connexion au jeu interrompue"));
            }
        } else if flow.synchronized {
            self.active = Some(key);
            self.flows
                .retain(|other, flow| *other == key || !flow.synchronized);
        }
    }
}
impl Flow {
    fn feed(
        &mut self,
        sequence: u32,
        bytes: &[u8],
        now: Instant,
        received: Instant,
        clients: &[Client],
    ) -> Result<(), &'static str> {
        if bytes.is_empty() || self.excluded {
            return Ok(());
        }
        let next = *self.next.get_or_insert(sequence);
        let delta = sequence.wrapping_sub(next) as i32;
        if delta > 0 {
            if let Some(old) = self.pending.get(&sequence)
                && old != bytes
            {
                return Err("Retransmission contradictoire");
            }
            self.pending.insert(sequence, bytes.to_vec());
            if self.pending.len() > MAX_PENDING
                || self.pending.values().map(Vec::len).sum::<usize>() > MAX_BUFFER
            {
                return Err("Limite de réassemblage atteinte");
            }
            self.gap.get_or_insert(now);
            return Ok(());
        }
        let overlap = (-(i64::from(delta))) as usize;
        let check = overlap.min(bytes.len());
        if check > 0 {
            let start = self
                .history
                .len()
                .checked_sub(overlap)
                .ok_or("Retransmission trop ancienne")?;
            if self.history.get(start..start + check) != Some(&bytes[..check]) {
                return Err("Retransmission contradictoire");
            }
        }
        if overlap < bytes.len() {
            self.append(&bytes[overlap..])?;
        }
        loop {
            let next = self.next.unwrap();
            let ready = self
                .pending
                .keys()
                .copied()
                .find(|seq| seq.wrapping_sub(next) as i32 <= 0);
            let Some(seq) = ready else {
                break;
            };
            let pending = self.pending.remove(&seq).unwrap();
            let overlap = next.wrapping_sub(seq) as usize;
            let check = overlap.min(pending.len());
            let start = self
                .history
                .len()
                .checked_sub(overlap)
                .ok_or("Segment en attente trop ancien")?;
            if self.history.get(start..start + check) != Some(&pending[..check]) {
                return Err("Chevauchement TCP contradictoire");
            }
            if overlap < pending.len() {
                self.append(&pending[overlap..])?;
            }
        }
        if self.pending.is_empty() {
            self.gap = None;
        }
        if !self.synchronized {
            if synchronization::is_tls(&self.buffer) {
                self.excluded = true;
                self.buffer.clear();
                self.history.clear();
                self.pending.clear();
                return Ok(());
            }
            let Some(boundary) = synchronization::server_boundary(&self.buffer) else {
                return Ok(());
            };
            self.buffer.drain(..boundary);
            self.synchronized = true;
            reset(clients, None);
        }
        let mut offset = 0;
        let mut budget = MAX_OUTPUT;
        while offset < self.buffer.len() {
            if self.buffer[offset] == 0 {
                offset += 1;
                continue;
            }
            let Some((length, width)) = frame_length(&self.buffer[offset..])? else {
                break;
            };
            if length > self.buffer.len() - offset {
                break;
            }
            deliver(
                &self.buffer[offset + width..offset + length],
                0,
                &mut budget,
                clients,
                received,
            )?;
            offset += length;
        }
        self.buffer.drain(..offset);
        if !self.buffer.is_empty() {
            self.gap.get_or_insert(now);
        }
        Ok(())
    }
    fn append(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        if self.buffer.len() + bytes.len() > MAX_BUFFER {
            return Err("Limite de flux atteinte");
        }
        self.buffer.extend_from_slice(bytes);
        self.history.extend_from_slice(bytes);
        if self.history.len() > MAX_BUFFER {
            self.history.drain(..self.history.len() - MAX_BUFFER);
        }
        self.next = Some(self.next.unwrap().wrapping_add(bytes.len() as u32));
        Ok(())
    }
}
fn deliver(
    body: &[u8],
    depth: usize,
    budget: &mut usize,
    clients: &[Client],
    now: Instant,
) -> Result<(), &'static str> {
    let marker = if body.starts_with(&[0xff, 0xff]) {
        Some(2)
    } else if matches!(body.first(), Some(0xf0..=0xfe)) && body.get(1..3) == Some(&[0xff, 0xff]) {
        Some(3)
    } else {
        None
    };
    if let Some(marker) = marker {
        if depth >= 4 {
            return Err("Profondeur LZ4 hors limites");
        }
        let size = u32::from_le_bytes(
            body.get(marker..marker + 4)
                .ok_or("En-tête LZ4 incomplet")?
                .try_into()
                .unwrap(),
        ) as usize;
        let compressed = &body[marker + 4..];
        if size == 0
            || size > MAX_BUFFER
            || size > *budget
            || size > compressed.len().saturating_mul(256)
        {
            return Err("Budget LZ4 dépassé");
        }
        *budget -= size;
        let plain =
            lz4_flex::block::decompress(compressed, size).map_err(|_| "Bloc LZ4 invalide")?;
        let mut offset = 0;
        while offset < plain.len() {
            if plain[offset] == 0 {
                offset += 1;
                continue;
            }
            let (length, width) = frame_length(&plain[offset..])?.ok_or("Préfixe LZ4 incomplet")?;
            let frame = plain
                .get(offset + width..offset + length)
                .ok_or("Trame LZ4 incomplète")?;
            deliver(frame, depth + 1, budget, clients, now)?;
            offset += length;
        }
    } else if let Some(opcode) = body.get(..2) {
        let opcode = opcode.try_into().unwrap();
        let payload = &body[2..];
        let entity = entity(payload).map(|(id, _)| id);
        for client in clients {
            let mut client = client.lock().unwrap();
            if client.accepts(opcode, entity) {
                client.message(opcode, payload, now);
            }
        }
    }
    Ok(())
}
pub(crate) fn entity(bytes: &[u8]) -> Option<(u32, usize)> {
    let (id, width) = read_varuint(bytes)?;
    if width > 5 || (width > 1 && bytes[width - 1] == 0) {
        return None;
    }
    Some((u32::try_from(id).ok()?, width))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::capture::LinkType;
    #[derive(Default)]
    struct Observed {
        messages: Vec<Vec<u8>>,
        resets: Vec<Option<String>>,
        tracked: Option<u32>,
    }
    impl Consumer for Observed {
        fn accepts(&self, opcode: [u8; 2], id: Option<u32>) -> bool {
            opcode == [0x34, 0x36] || (opcode == [0x35, 0x36] && self.tracked == id && id.is_some())
        }
        fn message(&mut self, opcode: [u8; 2], bytes: &[u8], _: Instant) {
            self.messages.push(bytes.to_vec());
            if opcode == [0x34, 0x36] {
                self.tracked = entity(bytes).map(|(id, _)| id);
            } else {
                self.tracked = None;
            }
        }
        fn reset(&mut self, error: Option<&str>) {
            self.resets.push(error.map(str::to_owned));
            self.tracked = None;
        }
    }
    pub(crate) fn tcp(sequence: u32, flags: u8, payload: &[u8]) -> CapturedPacket {
        let mut bytes = vec![0; 40];
        bytes[0] = 0x45;
        bytes[2..4].copy_from_slice(&((40 + payload.len()) as u16).to_be_bytes());
        bytes[8] = 64;
        bytes[9] = 6;
        bytes[12..20].copy_from_slice(&[10, 0, 0, 1, 10, 0, 0, 2]);
        bytes[20..24].copy_from_slice(&[0x34, 0x10, 0x30, 0x39]);
        bytes[24..28].copy_from_slice(&sequence.to_be_bytes());
        bytes[32] = 0x50;
        bytes[33] = flags;
        bytes.extend_from_slice(payload);
        CapturedPacket {
            number: 1,
            section_id: 0,
            interface_id: 0,
            local_address: Some("10.0.0.2".parse().unwrap()),
            link_type: LinkType::RawIpv4,
            timestamp_ns: None,
            timestamp: None,
            captured_len: bytes.len() as u32,
            original_len: bytes.len() as u32,
            bytes,
        }
    }
    pub(crate) fn frame(body: &[u8]) -> Vec<u8> {
        let mut length = body.len() + 4;
        let mut bytes = Vec::new();
        while length >= 128 {
            bytes.push((length as u8 & 127) | 128);
            length >>= 7;
        }
        bytes.push(length as u8);
        bytes.extend_from_slice(body);
        bytes
    }
    pub(crate) fn ticks() -> Vec<u8> {
        (0..3u64)
            .flat_map(|i| {
                frame(
                    &[
                        &[0, 0x36][..],
                        &(1_793_200_000_000u64 + i * 50).to_le_bytes(),
                    ]
                    .concat(),
                )
            })
            .collect()
    }
    pub(crate) fn compressed(plain: &[u8]) -> Vec<u8> {
        frame(
            &[
                &[0xff, 0xff][..],
                &(plain.len() as u32).to_le_bytes(),
                &lz4_flex::block::compress(plain),
            ]
            .concat(),
        )
    }
    #[test]
    fn fragmented_reordered_compressed_retransmitted_stream_filters_before_delivery() {
        let now = Instant::now();
        let observer = Arc::new(Mutex::new(Observed::default()));
        let clients: Vec<Client> = vec![observer.clone()];
        let mut decoder = Decoder::default();
        let plain = [
            frame(&[0x35, 0x36, 99]),
            frame(&[0x34, 0x36, 42]),
            frame(&[0x35, 0x36, 42]),
            frame(&[4, 0x38, 42]),
        ]
        .concat();
        let wire = [ticks(), compressed(&plain)].concat();
        // Start just before sequence wrap. The SYN gives the initial boundary;
        // application synchronization still waits for three server ticks.
        let start = u32::MAX - 16;
        decoder.feed(&tcp(start.wrapping_sub(1), 0x12, &[]), now, &clients);
        decoder.feed(
            &tcp(start.wrapping_add(15), 0x18, &wire[15..]),
            now,
            &clients,
        );
        assert!(observer.lock().unwrap().messages.is_empty());
        decoder.feed(&tcp(start, 0x18, &wire[..15]), now, &clients);
        decoder.feed(&tcp(start, 0x18, &wire), now, &clients);
        assert_eq!(observer.lock().unwrap().messages, vec![vec![42], vec![42]]);
        assert!(observer.lock().unwrap().resets.iter().all(Option::is_none));
    }
    #[test]
    fn gaps_conflicts_disconnect_and_client_ciphertext_invalidate_or_stay_unread() {
        let now = Instant::now();
        let observer = Arc::new(Mutex::new(Observed::default()));
        let clients: Vec<Client> = vec![observer.clone()];
        let mut decoder = Decoder::default();
        let wire = [ticks(), frame(&[0x34, 0x36, 42])].concat();
        let mut client = tcp(100, 0x18, &wire);
        client.local_address = Some("10.0.0.1".parse().unwrap());
        decoder.feed(&client, now, &clients);
        assert!(observer.lock().unwrap().messages.is_empty());
        decoder.feed(&tcp(100, 0x18, &wire), now, &clients);
        decoder.feed(&tcp(100, 0x18, &[99]), now, &clients);
        assert!(
            observer
                .lock()
                .unwrap()
                .resets
                .last()
                .unwrap()
                .as_ref()
                .unwrap()
                .contains("contradictoire")
        );
        decoder.feed(&tcp(100, 0x18, &wire), now, &clients);
        decoder.feed(&tcp(200, 0x18, &[1, 2]), now, &clients);
        decoder.expire(now + GAP_TTL, &clients);
        assert!(
            observer
                .lock()
                .unwrap()
                .resets
                .last()
                .unwrap()
                .as_ref()
                .unwrap()
                .contains("incomplet")
        );
        decoder.feed(&tcp(100, 0x18, &wire), now, &clients);
        decoder.feed(&tcp(100 + wire.len() as u32, 0x11, &[]), now, &clients);
        assert!(
            observer
                .lock()
                .unwrap()
                .resets
                .last()
                .unwrap()
                .as_ref()
                .unwrap()
                .contains("interrompue")
        );
    }
    #[test]
    fn connections_generations_and_silence_do_not_retain_world_state() {
        let now = Instant::now();
        let observer = Arc::new(Mutex::new(Observed::default()));
        let clients: Vec<Client> = vec![observer.clone()];
        let mut decoder = Decoder::default();
        let wire = [ticks(), frame(&[0x34, 0x36, 42])].concat();
        decoder.feed(&tcp(100, 0x18, &wire), now, &clients);
        assert_eq!(observer.lock().unwrap().tracked, Some(42));
        let mut other = tcp(100, 0x18, &wire);
        other.bytes[12] = 11;
        decoder.feed(&other, now, &clients);
        assert_eq!(observer.lock().unwrap().messages.len(), 1);
        let mut reconnect = tcp(99, 0x12, &[]);
        reconnect.bytes[12] = 11;
        decoder.feed(&reconnect, now, &clients);
        assert_eq!(observer.lock().unwrap().tracked, Some(42));
        decoder.feed(&other, now, &clients);
        assert_eq!(observer.lock().unwrap().tracked, Some(42));
        let resets = observer.lock().unwrap().resets.len();
        decoder.feed(&tcp(100, 0x18, &wire), now, &clients);
        assert_eq!(observer.lock().unwrap().resets.len(), resets);
        other.section_id = 1;
        decoder.feed(&other, now, &clients);
        assert_eq!(observer.lock().unwrap().messages.len(), 3);
        decoder.expire(now + Duration::from_secs(10), &clients);
        assert_eq!(observer.lock().unwrap().tracked, None);
        assert!(
            observer
                .lock()
                .unwrap()
                .resets
                .last()
                .unwrap()
                .as_ref()
                .unwrap()
                .contains("interrompu")
        );
    }
    #[test]
    fn unrelated_connections_cannot_clear_the_active_world() {
        let now = Instant::now();
        let observer = Arc::new(Mutex::new(Observed::default()));
        let clients: Vec<Client> = vec![observer.clone()];
        let mut decoder = Decoder::default();
        let wire = [ticks(), frame(&[0x34, 0x36, 42])].concat();
        decoder.feed(&tcp(100, 0x18, &wire), now, &clients);
        let resets = observer.lock().unwrap().resets.len();
        let foreign = |address, sequence, flags, payload: &[u8]| {
            let mut packet = tcp(sequence, flags, payload);
            packet.bytes[12] = address;
            packet
        };
        // Established TLS, missing bytes in another candidate, conflicting
        // retransmissions, truncation and a closed foreign socket are unrelated.
        decoder.feed(&foreign(11, 99, 0x12, &[]), now, &clients);
        decoder.feed(&foreign(11, 100, 0x18, &[0x17, 3, 3, 0, 33]), now, &clients);
        decoder.feed(&foreign(12, 99, 0x12, &[]), now, &clients);
        decoder.feed(&foreign(12, 200, 0x18, &[1, 2]), now, &clients);
        decoder.feed(&foreign(13, 99, 0x12, &[]), now, &clients);
        decoder.feed(&foreign(13, 100, 0x18, &[1, 2]), now, &clients);
        decoder.feed(&foreign(13, 100, 0x18, &[3, 4]), now, &clients);
        let mut truncated = foreign(14, 100, 0x18, &[1, 2]);
        truncated.original_len += 1;
        decoder.feed(&truncated, now, &clients);
        decoder.feed(&foreign(15, 99, 0x12, &[]), now, &clients);
        decoder.feed(&foreign(15, 100, 0x14, &[]), now, &clients);
        decoder.expire(now + GAP_TTL, &clients);
        assert_eq!(observer.lock().unwrap().tracked, Some(42));
        assert_eq!(observer.lock().unwrap().resets.len(), resets);
        // The original game stream still delivers withdrawals normally.
        decoder.feed(
            &tcp(100 + wire.len() as u32, 0x18, &frame(&[0x35, 0x36, 42])),
            now + GAP_TTL,
            &clients,
        );
        assert_eq!(observer.lock().unwrap().messages, vec![vec![42], vec![42]]);
        assert_eq!(observer.lock().unwrap().tracked, None);
    }
    #[test]
    fn tls_stays_excluded_and_duplicate_syn_does_not_restart_detection() {
        let now = Instant::now();
        let observer = Arc::new(Mutex::new(Observed::default()));
        let clients: Vec<Client> = vec![observer.clone()];
        let mut decoder = Decoder::default();
        let wire = [ticks(), frame(&[0x34, 0x36, 42])].concat();
        decoder.feed(&tcp(100, 0x18, &[0x17, 3, 3, 0, 33]), now, &clients);
        decoder.feed(&tcp(105, 0x18, &wire), now, &clients);
        assert!(observer.lock().unwrap().messages.is_empty());
        let mut decoder = Decoder::default();
        decoder.feed(&tcp(99, 0x12, &[]), now, &clients);
        decoder.feed(&tcp(100, 0x18, &wire), now, &clients);
        let resets = observer.lock().unwrap().resets.len();
        decoder.feed(&tcp(99, 0x12, &[]), now, &clients);
        assert_eq!(observer.lock().unwrap().resets.len(), resets);
        assert_eq!(observer.lock().unwrap().tracked, Some(42));
    }
    #[test]
    fn queued_packet_time_is_preserved_for_position_freshness() {
        let now = Instant::now();
        let clock = CaptureClock {
            wall_ns: 1_000_000_000_000,
            monotonic: now,
        };
        let processing = now + Duration::from_secs(5);
        assert_eq!(clock.received(Some(clock.wall_ns), processing), Some(now));
        assert_eq!(
            clock.received(Some(clock.wall_ns + 6_000_000_000), processing),
            None
        );
    }
    #[test]
    fn malformed_compression_is_bounded_and_never_delivered() {
        let now = Instant::now();
        let observer = Arc::new(Mutex::new(Observed::default()));
        let clients: Vec<Client> = vec![observer.clone()];
        for body in [
            vec![0xff, 0xff],
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
            vec![0xff, 0xff, 16, 0, 0, 0, 1, 2],
        ] {
            let wire = [ticks(), frame(&body)].concat();
            Decoder::default().feed(&tcp(100, 0x18, &wire), now, &clients);
        }
        assert!(observer.lock().unwrap().messages.is_empty());
        assert_eq!(
            observer
                .lock()
                .unwrap()
                .resets
                .iter()
                .filter(|r| r.is_some())
                .count(),
            3
        );
    }
    #[test]
    fn unsubscribe_detaches_even_when_owner_retains_state() {
        let hub = Arc::new(Hub {
            clients: Mutex::new(Vec::new()),
            closed: AtomicBool::new(false),
        });
        let client: Client = Arc::new(Mutex::new(Observed::default()));
        hub.clients.lock().unwrap().push(Arc::downgrade(&client));
        let subscription = Subscription {
            _hub: hub.clone(),
            _client: client.clone(),
        };
        drop(subscription);
        assert!(hub.clients.lock().unwrap().is_empty());
        assert_eq!(Arc::strong_count(&hub), 1);
    }
}
