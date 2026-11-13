//! Overlay settings and lifecycles. Hidden cubes consume shared server messages;
//! the compass remains withdrawn until a passive heading source is validated.
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
mod cube_decoder;
#[cfg(any(windows, test))]
mod cube_guidance;
#[cfg(any(windows, test))]
mod guidance_visibility;
pub mod hidden_cubes;
pub mod notifications;
mod player_position;
pub use compass::{CompassTick, ORIENTATION_UNAVAILABLE, Orientation, STALE_AFTER};

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(default)]
    pub compass: bool,
    #[serde(default)]
    pub hidden_cubes: bool,
    #[serde(default)]
    pub cube_sound: bool,
    #[serde(default)]
    pub cube_guidance: bool,
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

/// Lifecycle for the hidden-cube widget, separate from the withdrawn compass.
pub struct CubeRuntime {
    consumer: Arc<Mutex<cube_decoder::CubeConsumer>>,
    _subscription: crate::capture::widgets::Subscription,
    #[cfg(windows)]
    _window: windows::CubeWindow,
}
impl CubeRuntime {
    pub fn start(sound: bool, guidance: bool) -> Self {
        let mut state = cube_decoder::CubeConsumer::default();
        state.sound = sound;
        state.set_guidance(guidance);
        let consumer = Arc::new(Mutex::new(state));
        let subscription = crate::capture::widgets::subscribe(consumer.clone());
        #[cfg(windows)]
        let window = windows::CubeWindow::start(consumer.clone());
        Self {
            consumer,
            _subscription: subscription,
            #[cfg(windows)]
            _window: window,
        }
    }
    pub fn status(&self) -> String {
        let capture = self.consumer.lock().unwrap().status();
        #[cfg(windows)]
        return format!("{} · {capture}", self._window.status());
        #[cfg(not(windows))]
        format!("Overlay en jeu disponible uniquement sous Windows · {capture}")
    }
    pub fn set_sound(&self, enabled: bool) {
        self.consumer.lock().unwrap().sound = enabled;
    }
    pub fn set_guidance(&self, enabled: bool) {
        self.consumer.lock().unwrap().set_guidance(enabled);
    }
    pub fn presence(&self) -> Option<String> {
        self.consumer
            .lock()
            .unwrap()
            .tracking
            .presence(Instant::now())
    }
}

/// A capture-validation probe with fixed fictional data and no game subscription.
pub struct CubePreview {
    #[cfg(windows)]
    window: windows::CubeWindow,
}
impl CubePreview {
    pub fn start() -> crate::capture::Result<Self> {
        #[cfg(windows)]
        return Ok(Self {
            window: windows::CubeWindow::preview(),
        });
        #[cfg(not(windows))]
        Err("Le test d’affichage en jeu est disponible uniquement sous Windows.".into())
    }
    pub fn status(&self) -> String {
        #[cfg(windows)]
        return self.window.status();
        #[cfg(not(windows))]
        "Test d’affichage disponible uniquement sous Windows".into()
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
    fn old_settings_do_not_enable_widgets_and_new_preferences_roundtrip() {
        let old: Settings = serde_json::from_str(r#"{"compass":true}"#).unwrap();
        assert!(old.compass);
        assert!(!old.hidden_cubes && !old.cube_sound && !old.cube_guidance);
        let defaults = Settings::default();
        assert!(!defaults.hidden_cubes && !defaults.cube_sound && !defaults.cube_guidance);
        let settings = Settings {
            hidden_cubes: true,
            cube_sound: true,
            cube_guidance: true,
            ..defaults
        };
        let loaded: Settings =
            serde_json::from_slice(&serde_json::to_vec(&settings).unwrap()).unwrap();
        assert!(loaded.hidden_cubes && loaded.cube_sound && loaded.cube_guidance);
    }
    #[test]
    fn activation_survives_restart() {
        let path =
            std::env::temp_dir().join(format!("aetheris-overlay-{}.json", std::process::id()));
        Settings {
            compass: true,
            ..Settings::default()
        }
        .save(&path)
        .unwrap();
        assert!(Settings::load(&path).unwrap().compass);
        Settings {
            compass: false,
            ..Settings::default()
        }
        .save(&path)
        .unwrap();
        assert!(!Settings::load(&path).unwrap().compass);
        std::fs::remove_file(path).unwrap();
    }
}
