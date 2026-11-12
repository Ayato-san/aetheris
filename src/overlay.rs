//! Overlay lifecycle. There is deliberately no orientation decoder until a
//! passive source and cardinal calibration have been validated in game.
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
#[cfg(windows)]
mod windows;

mod compass;
pub use compass::{CompassTick, ORIENTATION_UNAVAILABLE, Orientation, STALE_AFTER};

#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub compass: bool,
}
impl Settings {
    pub fn load(path: &Path) -> crate::capture::Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }
    pub fn save(&self, path: &Path) -> crate::capture::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        crate::companion::durable_write(path, &serde_json::to_vec(self)?)
    }
    pub fn path() -> crate::capture::Result<PathBuf> {
        Ok(crate::companion::data_directory()?.join("overlay.json"))
    }
}

pub struct Runtime {
    stop: Arc<AtomicBool>,
    status: Arc<Mutex<String>>,
    worker: Option<std::thread::JoinHandle<()>>,
    orientation: Arc<Mutex<Orientation>>,
    #[cfg(windows)]
    window: windows::OverlayWindow,
}
impl Runtime {
    pub fn start() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let status = Arc::new(Mutex::new("Connexion au service de capture…".into()));
        let orientation = Arc::new(Mutex::new(Orientation::default()));
        #[cfg(windows)]
        let window = windows::OverlayWindow::start(orientation.clone());
        let cancel = stop.clone();
        let message = status.clone();
        let worker = std::thread::spawn(move || {
            let result = (|| -> crate::capture::Result<()> {
                let source = crate::capture::shared::subscribe(None)?;
                while !cancel.load(Ordering::Acquire) {
                    let Some(batch) = source.next_batch()? else {
                        continue;
                    };
                    if let Some(error) = &batch.error {
                        return Err(error.clone().into());
                    }
                    if batch.finished {
                        return Err("Capture arrêtée. Réessayez la connexion.".into());
                    }
                    // No opcode is subscribed yet: none has a validated local
                    // heading. Do not interpret movement as facing or save packets.
                    *message.lock().unwrap() =
                        "Capture connectée · orientation indisponible".into();
                }
                Ok(())
            })();
            if let Err(error) = result {
                *message.lock().unwrap() = format!("Service de capture indisponible : {error}");
            }
        });
        Self {
            stop,
            status,
            worker: Some(worker),
            orientation,
            #[cfg(windows)]
            window,
        }
    }
    /// Input seam for a decoder with validated local actor attribution and
    /// cardinal calibration. The passive source cannot produce this yet.
    pub fn update_orientation(&self, degrees: f32, received: Instant) {
        self.orientation.lock().unwrap().update(degrees, received);
    }
    pub fn status(&self) -> String {
        let capture = self.status.lock().unwrap().clone();
        #[cfg(windows)]
        return format!("{} · {capture}", self.window.status());
        #[cfg(not(windows))]
        format!("Overlay en jeu disponible uniquement sous Windows · {capture}")
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        // Joining happens off the UI thread: an IPC request can take 3 seconds.
        if let Some(worker) = self.worker.take() {
            std::thread::spawn(move || {
                let _ = worker.join();
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn heading_requires_a_finite_fresh_sample() {
        let now = Instant::now();
        let mut orientation = Orientation::default();
        assert_eq!(orientation.heading(now), None);
        orientation.update(-90., now);
        assert_eq!(
            orientation.heading(now + Duration::from_millis(2999)),
            Some(270.)
        );
        assert_eq!(orientation.heading(now + STALE_AFTER), None);
        orientation.update(f32::NAN, now + STALE_AFTER);
        assert_eq!(orientation.heading(now + STALE_AFTER), None);
        assert_eq!(orientation.heading(now - Duration::from_millis(1)), None);
    }
    #[test]
    fn compass_wraps_north_without_a_jump_and_expires() {
        let now = Instant::now();
        let mut orientation = Orientation::default();
        assert!(orientation.ticks(now).is_none());
        orientation.update(359., now);
        let ticks = orientation.ticks(now).unwrap();
        assert_eq!(ticks.iter().find(|t| t.label == "N").unwrap().offset, 1.);
        orientation.update(1., now);
        let ticks = orientation.ticks(now).unwrap();
        assert_eq!(ticks.iter().find(|t| t.label == "N").unwrap().offset, -1.);
        assert!(orientation.ticks(now + STALE_AFTER).is_none());
    }
    #[test]
    fn activation_survives_restart() {
        let path =
            std::env::temp_dir().join(format!("aetheris-overlay-{}.json", std::process::id()));
        Settings { compass: true }.save(&path).unwrap();
        assert!(Settings::load(&path).unwrap().compass);
        Settings { compass: false }.save(&path).unwrap();
        assert!(!Settings::load(&path).unwrap().compass);
        std::fs::remove_file(path).unwrap();
    }
}
