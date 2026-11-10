//! Live capture lifecycle. The worker persists batches before analyzing them.
use super::{Result, storage::DatabaseLease};
#[cfg(any(windows, test))]
use super::{source::CaptureSource, storage::Store};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
    },
    thread::JoinHandle,
};
#[cfg(any(windows, test))]
use std::{
    sync::{atomic::AtomicUsize, mpsc},
    thread,
    time::{Duration, Instant},
};

pub use crate::capture::Interface;

pub fn interfaces() -> Result<Vec<Interface>> {
    #[cfg(windows)]
    {
        crate::capture::service::interfaces()
    }
    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}

#[derive(Clone, Debug)]
pub struct LiveUpdate {
    pub status: String,
    pub error: Option<String>,
    pub finished: bool,
}

pub struct LiveHandle {
    pub session_id: i64,
    stop: Arc<AtomicBool>,
    updates: Receiver<LiveUpdate>,
    worker: Option<JoinHandle<()>>,
}

impl LiveHandle {
    pub fn start(
        path: PathBuf,
        interface: Option<Interface>,
        lease: Arc<DatabaseLease>,
    ) -> Result<Self> {
        #[cfg(not(windows))]
        {
            let _ = (path, interface, lease);
            Err("La capture réseau native est disponible uniquement sous Windows.".into())
        }
        #[cfg(windows)]
        {
            let source = crate::capture::service::connect(interface)?;
            let store = Store::open(&path)?;
            let session_id = store.create_session("network", "AION 2 — Service de capture", "")?;
            let stop = Arc::new(AtomicBool::new(false));
            let cancel = stop.clone();
            let (send, updates) = mpsc::channel();
            let worker = thread::spawn(move || {
                let _lease = lease;
                let result = record_source(&path, session_id, Box::new(source), cancel, &send);
                let error = result.err().map(|error| error.to_string());
                let finish = Store::open(&path)
                    .and_then(|store| store.finish_session(session_id, error.as_deref()));
                let error = error.or_else(|| finish.err().map(|error| error.to_string()));
                let _ = send.send(LiveUpdate {
                    status: "Capture arrêtée".into(),
                    error,
                    finished: true,
                });
            });
            Ok(Self {
                session_id,
                stop,
                updates,
                worker: Some(worker),
            })
        }
    }

    pub fn updates(&self) -> Vec<LiveUpdate> {
        self.updates.try_iter().collect()
    }

    pub fn request_stop(&self) {
        self.stop.store(true, Ordering::Release);
    }

    pub fn stop_and_join(mut self) {
        self.request_stop();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for LiveHandle {
    fn drop(&mut self) {
        self.request_stop();
    }
}

#[cfg(any(windows, test))]
fn record_source(
    path: &std::path::Path,
    id: i64,
    mut source: Box<dyn CaptureSource>,
    cancel: Arc<AtomicBool>,
    send: &mpsc::Sender<LiveUpdate>,
) -> Result<()> {
    use super::source::CaptureBatch;
    let mut store = Store::open(path)?;
    let (captured, batches) = mpsc::sync_channel::<CaptureBatch>(32);
    let queued = Arc::new(AtomicUsize::new(0));
    let reader_queued = queued.clone();
    let reader_cancel = cancel.clone();
    // Reception continues while SQLite replaces an analysis snapshot. If the
    // bounded queue saturates, the final received batch is still sent and saved.
    let reader = thread::spawn(move || {
        loop {
            let mut batch = if reader_cancel.load(Ordering::Acquire) {
                CaptureBatch {
                    packets: Vec::new(),
                    status: "Arrêt demandé".into(),
                    finished: true,
                    error: None,
                }
            } else {
                source.poll().unwrap_or_else(|error| CaptureBatch {
                    packets: Vec::new(),
                    status: "Erreur de capture".into(),
                    finished: true,
                    error: Some(error.to_string()),
                })
            };
            let bytes = batch
                .packets
                .iter()
                .map(|packet| packet.bytes.len())
                .sum::<usize>();
            if reader_queued
                .fetch_add(bytes, Ordering::AcqRel)
                .saturating_add(bytes)
                > 32 * 1024 * 1024
            {
                batch.error=Some("File d’écriture saturée (32 Mio) : capture arrêtée après conservation des paquets reçus.".into());
                batch.finished = true;
            }
            let finished = batch.finished;
            if captured.send(batch).is_err() || finished {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
    });
    let result = (|| {
        let mut last_analysis = Instant::now();
        let mut last_status = String::new();
        let mut dirty = false;
        let mut source_error = None;
        for batch in &batches {
            queued.fetch_sub(
                batch.packets.iter().map(|packet| packet.bytes.len()).sum(),
                Ordering::AcqRel,
            );
            if !batch.packets.is_empty() {
                store.append_packets(id, &batch.packets)?;
                dirty = true;
            }
            if last_analysis.elapsed() >= Duration::from_secs(1) {
                if dirty {
                    store.reanalyze(id, false)?;
                    dirty = false;
                }
                last_analysis = Instant::now();
                let _ = send.send(LiveUpdate {
                    status: batch.status.clone(),
                    error: None,
                    finished: false,
                });
            } else if last_status != batch.status {
                let _ = send.send(LiveUpdate {
                    status: batch.status.clone(),
                    error: None,
                    finished: false,
                });
            }
            last_status = batch.status;
            if batch.error.is_some() {
                source_error = batch.error;
                break;
            }
            if batch.finished {
                break;
            }
        }
        store.reanalyze(id, true)?;
        if let Some(error) = source_error {
            return Err(error.into());
        }
        Ok(())
    })();
    cancel.store(true, Ordering::Release);
    drop(batches);
    if reader.join().is_err() {
        return Err("Thread de réception interrompu.".into());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat_debug::{
        pcapng::{CapturedPacket, LinkType},
        source::CaptureBatch,
    };
    struct FailingSource;
    impl CaptureSource for FailingSource {
        fn poll(&mut self) -> Result<CaptureBatch> {
            Ok(CaptureBatch {
                packets: vec![CapturedPacket {
                    number: 1,
                    section_id: 0,
                    interface_id: 0,
                    local_address: None,
                    link_type: LinkType::Other(99),
                    timestamp_ns: None,
                    timestamp: None,
                    captured_len: 3,
                    original_len: 3,
                    bytes: vec![1, 2, 3],
                }],
                status: "test".into(),
                finished: true,
                error: Some("erreur simulée".into()),
            })
        }
    }
    #[test]
    fn final_source_error_does_not_discard_its_received_packet() {
        let root = std::env::temp_dir().join(format!(
            "aetheris-live-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("capture.sqlite3");
        let store = Store::open(&path).unwrap();
        let id = store.create_session("network", "test", "").unwrap();
        drop(store);
        let (updates, _) = mpsc::channel();
        assert!(
            record_source(
                &path,
                id,
                Box::new(FailingSource),
                Arc::new(AtomicBool::new(false)),
                &updates
            )
            .is_err()
        );
        let store = Store::open(&path).unwrap();
        assert_eq!(store.packets(id).unwrap()[0].bytes, [1, 2, 3]);
        assert_eq!(store.sessions().unwrap()[0].row_count, 1);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }
}
