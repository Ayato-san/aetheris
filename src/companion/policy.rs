//! Deterministic update decisions, separated from clocks, downloads and rendering.
use semver::Version;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Channel {
    #[default]
    Stable,
    Prerelease,
}
impl Channel {
    pub fn accepts(self, version: &Version) -> bool {
        self == Self::Prerelease || version.pre.is_empty()
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Preferences {
    pub channel: Channel,
    pub excluded: Option<Version>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Offer {
    BackgroundDownload,
    Announce,
}
pub fn offer(elapsed: Duration, excluded: bool) -> Offer {
    if elapsed <= Duration::from_secs(3) && !excluded {
        Offer::BackgroundDownload
    } else {
        Offer::Announce
    }
}
pub fn first_launch_crash(normal: bool, window_open: Option<Duration>) -> bool {
    !normal && window_open.is_none_or(|duration| duration < Duration::from_secs(10))
}

/// Closing policy for a single launch. Interaction cancels unattended closure,
/// but never cancels an explicit update request. A user closure while bytes are
/// still downloading always cancels that download, even if it finishes later.
#[derive(Default)]
pub struct UpdateLifecycle {
    interacted: bool,
    requested: bool,
    close_when_ready: bool,
    cancelled: bool,
}
impl UpdateLifecycle {
    pub fn interact(&mut self) {
        self.interacted = true;
        if !self.requested {
            self.close_when_ready = false;
        }
    }
    pub fn background_download(&mut self) {
        self.close_when_ready = !self.interacted;
    }
    pub fn request(&mut self) {
        self.requested = true;
        self.close_when_ready = true;
    }
    pub fn closing(&mut self, downloading: bool) {
        if downloading {
            self.cancelled = true;
            self.close_when_ready = false;
        }
    }
    pub fn blocked(&mut self) {
        self.close_when_ready = false;
    }
    pub fn should_close(&self) -> bool {
        self.close_when_ready && !self.cancelled
    }
    pub fn unattended(&self) -> bool {
        !self.requested
    }
    pub fn cancelled(&self) -> bool {
        self.cancelled
    }
}

/// Compare event creation times, rather than worker/IPC delivery order.
pub fn install_after_close(
    ready_at: std::time::SystemTime,
    closed_at: Option<std::time::SystemTime>,
) -> bool {
    closed_at.is_none_or(|closed| ready_at <= closed)
}
