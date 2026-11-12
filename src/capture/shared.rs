//! One IPC reader per application. Subscribers have bounded, independent queues;
//! a slow recorder fails explicitly without interrupting other consumers.
use super::{CaptureBatch, CaptureSource, Interface, Result};
#[cfg(any(windows, test))]
use std::sync::Mutex;
#[cfg(any(windows, test))]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
#[cfg(windows)]
use std::sync::{OnceLock, Weak};
use std::time::Duration;

struct Hub {
    #[cfg(windows)]
    interface: Option<Interface>,
    #[cfg(any(windows, test))]
    closed: AtomicBool,
    #[cfg(any(windows, test))]
    subscribers: Mutex<Vec<mpsc::SyncSender<Arc<CaptureBatch>>>>,
}
#[cfg(windows)]
static HUB: OnceLock<Mutex<Weak<Hub>>> = OnceLock::new();

pub struct Subscription {
    _hub: Arc<Hub>,
    batches: mpsc::Receiver<Arc<CaptureBatch>>,
}

#[cfg(windows)]
pub fn subscribe(interface: Option<Interface>) -> Result<Subscription> {
    let mut current = HUB.get_or_init(Default::default).lock().unwrap();
    let hub = if let Some(hub) = current
        .upgrade()
        .filter(|hub| !hub.closed.load(Ordering::Acquire))
    {
        if interface.is_some()
            && interface.as_ref().map(|i| (i.index, i.address))
                != hub.interface.as_ref().map(|i| (i.index, i.address))
        {
            return Err("Une capture utilise déjà une autre interface réseau.".into());
        }
        hub
    } else {
        let source = crate::capture::service::connect(interface.clone())?;
        {
            let hub = Arc::new(Hub {
                interface,
                closed: AtomicBool::new(false),
                subscribers: Mutex::new(Vec::new()),
            });
            *current = Arc::downgrade(&hub);
            // Register the first subscriber before starting the reader.
            let subscription = attach(hub.clone());
            std::thread::spawn(move || run(Box::new(source), hub));
            return Ok(subscription);
        }
    };
    Ok(attach(hub))
}

#[cfg(not(windows))]
pub fn subscribe(_: Option<Interface>) -> Result<Subscription> {
    Err("La capture réseau native est disponible uniquement sous Windows.".into())
}

#[cfg(any(windows, test))]
fn attach(hub: Arc<Hub>) -> Subscription {
    let (send, batches) = mpsc::sync_channel(8);
    hub.subscribers.lock().unwrap().push(send);
    Subscription { _hub: hub, batches }
}

#[cfg(any(windows, test))]
fn run(mut source: Box<dyn CaptureSource>, hub: Arc<Hub>) {
    loop {
        // The reader retains the hub until IPC has closed. A new subscriber
        // must never connect while the previous reader is still releasing it.
        #[cfg(windows)]
        let current = HUB.get_or_init(Default::default).lock().unwrap();
        if Arc::strong_count(&hub) == 1 {
            drop(source);
            hub.closed.store(true, Ordering::Release);
            hub.subscribers.lock().unwrap().clear();
            return;
        }
        #[cfg(windows)]
        drop(current);
        let batch = Arc::new(source.poll().unwrap_or_else(|error| CaptureBatch {
            packets: Vec::new(),
            status: "Capture indisponible".into(),
            finished: true,
            error: Some(error.to_string()),
        }));
        let finished = batch.finished || batch.error.is_some();
        let mut subscribers = hub.subscribers.lock().unwrap();
        subscribers.retain(|subscriber| subscriber.try_send(batch.clone()).is_ok());
        drop(subscribers);
        if finished {
            #[cfg(windows)]
            let _current = HUB.get_or_init(Default::default).lock().unwrap();
            drop(source);
            hub.closed.store(true, Ordering::Release);
            hub.subscribers.lock().unwrap().clear();
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

impl Subscription {
    /// Share evidence without copying packet bodies for ephemeral consumers.
    pub fn next_batch(&self) -> Result<Option<Arc<CaptureBatch>>> {
        match self.batches.recv_timeout(Duration::from_millis(100)) {
            Ok(batch) => Ok(Some(batch)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(_) => Err(
                "Capture partagée interrompue ou consommateur trop lent ; reconnectez-vous.".into(),
            ),
        }
    }
}

impl CaptureSource for Subscription {
    fn poll(&mut self) -> Result<CaptureBatch> {
        match self.next_batch()? {
            Some(batch) => Ok(CaptureBatch {
                packets: batch.packets.clone(),
                status: batch.status.clone(),
                finished: batch.finished,
                error: batch.error.clone(),
            }),
            None => Ok(CaptureBatch {
                packets: Vec::new(),
                status: "Capture partagée en attente".into(),
                finished: false,
                error: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slow_or_removed_subscriber_does_not_stop_others() {
        let hub = Arc::new(Hub {
            #[cfg(windows)]
            interface: None,
            closed: AtomicBool::new(false),
            subscribers: Mutex::new(Vec::new()),
        });
        let slow = attach(hub.clone());
        let mut active = attach(hub.clone());
        for _ in 0..9 {
            let batch = Arc::new(CaptureBatch {
                packets: vec![],
                status: "ok".into(),
                finished: false,
                error: None,
            });
            hub.subscribers
                .lock()
                .unwrap()
                .retain(|sender| sender.try_send(batch.clone()).is_ok());
            assert_eq!(active.poll().unwrap().status, "ok");
        }
        assert_eq!(hub.subscribers.lock().unwrap().len(), 1);
        drop(slow);
        assert_eq!(hub.subscribers.lock().unwrap().len(), 1);
    }
    struct Controlled(mpsc::Receiver<CaptureBatch>);
    impl CaptureSource for Controlled {
        fn poll(&mut self) -> Result<CaptureBatch> {
            Ok(self.0.recv().unwrap())
        }
    }
    #[test]
    fn dropping_one_subscriber_preserves_capture_and_terminal_error_is_delivered() {
        let hub = Arc::new(Hub {
            #[cfg(windows)]
            interface: None,
            closed: AtomicBool::new(false),
            subscribers: Mutex::new(Vec::new()),
        });
        let first = attach(hub.clone());
        let mut second = attach(hub.clone());
        let (send, receive) = mpsc::channel();
        let reader_hub = hub.clone();
        let worker = std::thread::spawn(move || run(Box::new(Controlled(receive)), reader_hub));
        drop(first);
        send.send(CaptureBatch {
            packets: vec![],
            status: "still running".into(),
            finished: false,
            error: None,
        })
        .unwrap();
        assert_eq!(second.poll().unwrap().status, "still running");
        send.send(CaptureBatch {
            packets: vec![],
            status: "failed".into(),
            finished: true,
            error: Some("service lost".into()),
        })
        .unwrap();
        assert_eq!(
            second.poll().unwrap().error.as_deref(),
            Some("service lost")
        );
        worker.join().unwrap();
        assert!(hub.closed.load(Ordering::Acquire));
    }
    struct MustNotPoll(Arc<AtomicBool>);
    impl CaptureSource for MustNotPoll {
        fn poll(&mut self) -> Result<CaptureBatch> {
            panic!("capture polled without subscribers")
        }
    }
    impl Drop for MustNotPoll {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }
    #[test]
    fn last_subscriber_releases_source_without_an_extra_poll() {
        let hub = Arc::new(Hub {
            #[cfg(windows)]
            interface: None,
            closed: AtomicBool::new(false),
            subscribers: Mutex::new(Vec::new()),
        });
        let subscriber = attach(hub.clone());
        drop(subscriber);
        let released = Arc::new(AtomicBool::new(false));
        run(Box::new(MustNotPoll(released.clone())), hub);
        assert!(released.load(Ordering::Acquire));
    }
}
