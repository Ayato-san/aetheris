//! One on-demand acquisition, observation tracker and set of network aggregates.
//! Subscribers read one immutable latest snapshot, never a queue of raw packets.
use super::{
    CaptureBatch, Result, latency,
    observations::{self, Event},
    retransmissions, shared, throughput,
};
use std::{
    sync::{
        Arc, Condvar, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const UPDATE_INTERVAL: Duration = Duration::from_secs(1);
const RETRY_INTERVAL: Duration = Duration::from_secs(1);
const BATCH_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Interests {
    pub latency: bool,
    pub traffic: bool,
    pub retransmissions: bool,
}
impl Interests {
    pub const LATENCY: Self = Self {
        latency: true,
        traffic: false,
        retransmissions: false,
    };
    pub const TRAFFIC: Self = Self {
        latency: false,
        traffic: true,
        retransmissions: false,
    };
    pub const RETRANSMISSIONS: Self = Self {
        latency: false,
        traffic: false,
        retransmissions: true,
    };
    pub const ALL: Self = Self {
        latency: true,
        traffic: true,
        retransmissions: true,
    };
    fn flags(self) -> [bool; 3] {
        [self.latency, self.traffic, self.retransmissions]
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Windows {
    pub latency: Duration,
    pub traffic: Duration,
    pub retransmissions: Duration,
}
impl Default for Windows {
    fn default() -> Self {
        Self {
            latency: latency::DEFAULT_WINDOW,
            traffic: throughput::DEFAULT_WINDOW,
            retransmissions: retransmissions::DEFAULT_WINDOW,
        }
    }
}
impl Windows {
    fn validate(self) -> Result<()> {
        if self.latency.is_zero()
            || self.latency > latency::MAX_WINDOW
            || self.traffic.is_zero()
            || self.traffic > throughput::MAX_WINDOW
            || self.retransmissions.is_zero()
            || self.retransmissions > retransmissions::MAX_WINDOW
        {
            return Err("Durée de fenêtre réseau invalide.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Configuration {
    windows: Windows,
    interests: Interests,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub windows: Windows,
    pub active: Interests,
    pub capture_available: bool,
    pub error: Option<String>,
    pub collector_drops: Option<u64>,
    pub latency: latency::Snapshot,
    pub traffic: throughput::Snapshot,
    pub retransmissions: retransmissions::Snapshot,
}
impl Snapshot {
    fn availability(&self) -> (bool, bool, bool, bool) {
        (
            self.capture_available,
            self.traffic.rates.is_some(),
            self.latency.ping.is_some(),
            self.retransmissions.counts.is_some(),
        )
    }
}

struct Aggregates {
    configuration: Configuration,
    tracker: observations::Tracker,
    latency: latency::Latency,
    traffic: throughput::Throughput,
    retransmissions: retransmissions::Retransmissions,
    capture_available: bool,
    error: Option<String>,
}
impl Aggregates {
    fn new(configuration: Configuration, now: Instant) -> Self {
        let mut aggregates = Self {
            configuration: Configuration {
                windows: Windows::default(),
                interests: Interests::default(),
            },
            tracker: observations::Tracker::default(),
            latency: latency::Latency::default(),
            traffic: throughput::Throughput::default(),
            retransmissions: retransmissions::Retransmissions::default(),
            capture_available: false,
            error: None,
        };
        aggregates.configure(configuration, now);
        aggregates
    }
    fn configure(&mut self, configuration: Configuration, now: Instant) {
        let previous = self.configuration;
        let active = configuration.interests;
        self.tracker
            .set_connection_tracking(active.latency || active.retransmissions);
        if active.latency != previous.interests.latency {
            self.latency = latency::Latency::default();
            if active.latency {
                self.latency.observe(
                    &Event::SelectionChanged(self.tracker.selection().clone()),
                    now,
                );
            }
        }
        if active.traffic != previous.interests.traffic {
            self.traffic = throughput::Throughput::default();
            if active.traffic {
                self.traffic.observe(
                    &Event::CaptureState {
                        available: self.capture_available,
                    },
                    now,
                );
            }
        }
        if active.retransmissions != previous.interests.retransmissions {
            self.retransmissions = retransmissions::Retransmissions::default();
            if active.retransmissions {
                self.retransmissions.observe(
                    &Event::SelectionChanged(self.tracker.selection().clone()),
                    now,
                );
            }
        }
        self.latency
            .set_window(configuration.windows.latency, now)
            .unwrap();
        self.traffic
            .set_window(configuration.windows.traffic, now)
            .unwrap();
        self.retransmissions
            .set_window(configuration.windows.retransmissions, now)
            .unwrap();
        self.configuration = configuration;
    }
    fn dispatch(
        &mut self,
        action: impl FnOnce(&mut observations::Tracker, &mut dyn FnMut(Event)),
        now: Instant,
    ) {
        let active = self.configuration.interests;
        let Self {
            tracker,
            latency,
            traffic,
            retransmissions,
            capture_available,
            ..
        } = self;
        action(tracker, &mut |event| {
            if let Event::CaptureState { available } = &event {
                *capture_available = *available;
            }
            if active.latency {
                latency.observe(&event, now);
            }
            if active.traffic {
                traffic.observe(&event, now);
            }
            if active.retransmissions {
                retransmissions.observe(&event, now);
            }
        });
    }
    fn batch(&mut self, batch: &CaptureBatch, now: Instant) {
        self.error = batch.error.clone().or_else(|| batch.interruption.clone());
        self.dispatch(|tracker, emit| tracker.batch(batch, now, emit), now);
        if self.configuration.interests.traffic
            && batch.interruption.is_none()
            && batch.error.is_none()
            && !batch.finished
        {
            self.traffic.capture_tick();
        }
    }
    fn interrupt(&mut self, reason: String, now: Instant) {
        self.error = Some(reason.clone());
        self.dispatch(|tracker, emit| tracker.interrupt(reason, emit), now);
    }
    fn snapshot(&mut self, now: Instant) -> Snapshot {
        self.dispatch(|tracker, emit| tracker.expire(now, emit), now);
        Snapshot {
            windows: self.configuration.windows,
            active: self.configuration.interests,
            capture_available: self.capture_available,
            error: self.error.clone(),
            collector_drops: self.tracker.collector_drops(),
            latency: self.latency.snapshot(now),
            traffic: self.traffic.snapshot(now),
            retransmissions: self.retransmissions.snapshot(now),
        }
    }
}

trait BatchSource: Send {
    fn next_batch(&mut self) -> Result<Option<Arc<CaptureBatch>>>;
}
impl BatchSource for shared::Subscription {
    fn next_batch(&mut self) -> Result<Option<Arc<CaptureBatch>>> {
        shared::Subscription::next_batch(self)
    }
}
type Connector = Arc<dyn Fn() -> Result<Box<dyn BatchSource>> + Send + Sync>;
struct Control {
    windows: Windows,
    consumers: [usize; 3],
}
impl Control {
    fn configuration(&self) -> Configuration {
        Configuration {
            windows: self.windows,
            interests: Interests {
                latency: self.consumers[0] > 0,
                traffic: self.consumers[1] > 0,
                retransmissions: self.consumers[2] > 0,
            },
        }
    }
    fn add(&mut self, interests: Interests) {
        for (count, enabled) in self.consumers.iter_mut().zip(interests.flags()) {
            *count += usize::from(enabled);
        }
    }
    fn remove(&mut self, interests: Interests) {
        for (count, enabled) in self.consumers.iter_mut().zip(interests.flags()) {
            *count -= usize::from(enabled);
        }
    }
}
struct Hub {
    latest: Mutex<Arc<Snapshot>>,
    control: Mutex<Control>,
    wake: Condvar,
    closed: AtomicBool,
}
impl Hub {
    fn publish(&self, snapshot: Snapshot) {
        let mut latest = self.latest.lock().unwrap();
        if !self.closed.load(Ordering::Acquire) {
            *latest = Arc::new(snapshot);
        }
    }
    fn wait_retry(&self) {
        let control = self.control.lock().unwrap();
        if !self.closed.load(Ordering::Acquire) {
            drop(self.wake.wait_timeout(control, RETRY_INTERVAL).unwrap());
        }
    }
}
#[derive(Default)]
struct Coordinator {
    current: Weak<Hub>,
    windows: Windows,
}
static COORDINATOR: OnceLock<Arc<Mutex<Coordinator>>> = OnceLock::new();

/// Declare only the calculations needed by this active network widget. The
/// first subscriber starts collection; all subscribers share its aggregates.
pub fn subscribe(interests: Interests) -> Result<Subscription> {
    subscribe_with(
        COORDINATOR.get_or_init(Default::default).clone(),
        interests,
        Arc::new(|| Ok(Box::new(shared::subscribe(None)?) as Box<dyn BatchSource>)),
    )
}
fn subscribe_with(
    coordinator: Arc<Mutex<Coordinator>>,
    interests: Interests,
    connector: Connector,
) -> Result<Subscription> {
    if !interests.flags().into_iter().any(|enabled| enabled) {
        return Err("Aucune mesure réseau demandée.".into());
    }
    let hub = {
        let mut current = coordinator.lock().unwrap();
        if let Some(hub) = current
            .current
            .upgrade()
            .filter(|hub| !hub.closed.load(Ordering::Acquire))
        {
            hub.control.lock().unwrap().add(interests);
            hub.wake.notify_all();
            hub
        } else {
            let mut control = Control {
                windows: current.windows,
                consumers: [0; 3],
            };
            control.add(interests);
            let configuration = control.configuration();
            let now = Instant::now();
            let hub = Arc::new(Hub {
                latest: Mutex::new(Arc::new(Aggregates::new(configuration, now).snapshot(now))),
                control: Mutex::new(control),
                wake: Condvar::new(),
                closed: AtomicBool::new(false),
            });
            current.current = Arc::downgrade(&hub);
            let worker_hub = hub.clone();
            std::thread::spawn(move || run(worker_hub, connector));
            hub
        }
    };
    Ok(Subscription {
        hub,
        coordinator,
        interests,
    })
}

pub struct Subscription {
    hub: Arc<Hub>,
    coordinator: Arc<Mutex<Coordinator>>,
    interests: Interests,
}
impl Subscription {
    pub fn snapshot(&self) -> Arc<Snapshot> {
        self.hub.latest.lock().unwrap().clone()
    }
    pub fn windows(&self) -> Windows {
        self.coordinator.lock().unwrap().windows
    }
    /// All active network widgets use these same windows. Applied atomically by
    /// the worker at its next poll; Snapshot::windows identifies the applied set.
    pub fn set_windows(&self, windows: Windows) -> Result<()> {
        windows.validate()?;
        let mut coordinator = self.coordinator.lock().unwrap();
        coordinator.windows = windows;
        self.hub.control.lock().unwrap().windows = windows;
        self.hub.wake.notify_all();
        Ok(())
    }
}
impl Clone for Subscription {
    fn clone(&self) -> Self {
        let _coordinator = self.coordinator.lock().unwrap();
        self.hub.control.lock().unwrap().add(self.interests);
        Self {
            hub: self.hub.clone(),
            coordinator: self.coordinator.clone(),
            interests: self.interests,
        }
    }
}
impl Drop for Subscription {
    fn drop(&mut self) {
        let _coordinator = self.coordinator.lock().unwrap();
        let mut control = self.hub.control.lock().unwrap();
        control.remove(self.interests);
        if control.consumers == [0; 3] {
            self.hub.closed.store(true, Ordering::Release);
            let configuration = control.configuration();
            let now = Instant::now();
            *self.hub.latest.lock().unwrap() =
                Arc::new(Aggregates::new(configuration, now).snapshot(now));
        }
        self.hub.wake.notify_all();
    }
}
fn run(hub: Arc<Hub>, connector: Connector) {
    let now = Instant::now();
    let mut aggregates = Aggregates::new(hub.control.lock().unwrap().configuration(), now);
    let mut source: Option<Box<dyn BatchSource>> = None;
    let mut last_batch = now;
    let mut last_published = now;
    while !hub.closed.load(Ordering::Acquire) {
        let configuration = hub.control.lock().unwrap().configuration();
        let configured = configuration != aggregates.configuration;
        if configured {
            aggregates.configure(configuration, Instant::now());
        }
        if source.is_none() {
            match connector() {
                Ok(connected) => {
                    source = Some(connected);
                    last_batch = Instant::now();
                }
                Err(error) => {
                    aggregates.interrupt(error.to_string(), Instant::now());
                    hub.publish(aggregates.snapshot(Instant::now()));
                    hub.wait_retry();
                    continue;
                }
            }
        }
        if hub.closed.load(Ordering::Acquire) {
            break;
        }
        match source.as_mut().unwrap().next_batch() {
            Ok(Some(batch)) => {
                last_batch = Instant::now();
                aggregates.batch(&batch, last_batch);
                if batch.finished || batch.error.is_some() {
                    source = None;
                }
            }
            Ok(None) => {
                if aggregates.capture_available && last_batch.elapsed() >= BATCH_TIMEOUT {
                    aggregates.interrupt(
                        "Aucun lot reçu de la capture depuis deux secondes.".into(),
                        Instant::now(),
                    );
                }
            }
            Err(error) => {
                aggregates.interrupt(error.to_string(), Instant::now());
                source = None;
            }
        }
        let now = Instant::now();
        let snapshot = aggregates.snapshot(now);
        let previous = hub.latest.lock().unwrap().clone();
        if configured
            || snapshot.availability() != previous.availability()
            || snapshot.error != previous.error
            || now.saturating_duration_since(last_published) >= UPDATE_INTERVAL
        {
            hub.publish(snapshot);
            last_published = now;
        }
        if source.is_none() {
            hub.wait_retry();
        }
    }
    // Dropping the private tracker and calculators releases their histories;
    // dropping this single raw subscription leaves other hub consumers intact.
}

#[cfg(test)]
pub(super) fn subscribe_source(source: shared::Subscription) -> Subscription {
    let source = Mutex::new(Some(source));
    subscribe_with(
        Arc::new(Mutex::new(Coordinator::default())),
        Interests::ALL,
        Arc::new(move || {
            source
                .lock()
                .unwrap()
                .take()
                .map(|source| Box::new(source) as Box<dyn BatchSource>)
                .ok_or_else(|| "test source already consumed".into())
        }),
    )
    .unwrap()
}

#[cfg(test)]
impl Subscription {
    pub(super) fn release_probe_for_test(&self) -> Box<dyn Fn() -> bool> {
        let hub = Arc::downgrade(&self.hub);
        Box::new(move || hub.upgrade().is_none())
    }
}

#[cfg(test)]
mod tests;
