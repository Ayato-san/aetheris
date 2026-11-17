//! Locally observed campaigns, shared by the calendar, overlay and reminders.
//! Times are UTC Unix milliseconds. No account progress is inferred.
pub mod service;
mod source;
mod storage;

use crate::companion::{Result, error};
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub use storage::Catalog;

pub const SOURCE_URL: &str = "https://drops.sunkwi.com/api/twitch/drops";
pub const MAX_AGE: i64 = 48 * 60 * 60 * 1000;
const YEAR: i64 = 365 * 24 * 60 * 60 * 1000;

pub(crate) fn timestamp(value: &Value) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value.as_str()?)
        .ok()
        .map(|t| t.timestamp_millis())
}
pub(crate) fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Region {
    #[default]
    Unconfirmed,
    Europe,
    Incompatible,
}
impl Region {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Unconfirmed => "région non confirmée",
            Self::Europe => "Europe",
            Self::Incompatible => "incompatible avec l’Europe",
        }
    }
}

/// User decisions survive disappearance, pruning of history and reappearance.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserState {
    pub hidden: bool,
    pub region: Region,
    pub region_evidence: Option<String>,
    pub delivered_reminders: std::collections::BTreeSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reward {
    pub name: Option<String>,
    pub tier_name: Option<String>,
    pub image_url: Option<String>,
    pub required_minutes: Option<u64>,
    pub required_subs: Option<u64>,
    /// Includes unknown preconditions, repeat rules and benefit metadata.
    pub raw_tier: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Campaign {
    pub id: String,
    pub title: Option<String>,
    pub start_at: i64,
    pub end_at: i64,
    pub status: Option<String>,
    pub rewards: Option<Vec<Reward>>,
    /// Source metadata, including description, links, images and channel conditions.
    pub raw: Value,
    pub game: Value,
    pub source_url: String,
    pub first_observed_at: i64,
    pub last_observed_at: i64,
    pub last_source_scan_at: Option<i64>,
    pub present: bool,
    pub user: UserState,
}
impl Campaign {
    pub fn active(&self, snapshot: &Snapshot, now: i64) -> bool {
        snapshot.fresh(now)
            && self.present
            && self.status.as_deref() == Some("ACTIVE")
            && self.start_at <= now
            && now < self.end_at
            && self.user.region != Region::Incompatible
    }
    pub fn visible_in_widget(&self, snapshot: &Snapshot, now: i64) -> bool {
        self.active(snapshot, now) && !self.user.hidden
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataState {
    Loading,
    Empty,
    Available,
    Stale,
    UnknownFreshness,
    Unavailable,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    pub campaigns: Vec<Campaign>,
    pub refreshed_at: Option<i64>,
    pub downloaded_at: Option<i64>,
    pub last_checked_at: Option<i64>,
    pub last_error: Option<String>,
    pub etag: Option<String>,
    pub next_attempt_at: i64,
    pub failures: u32,
}
impl Snapshot {
    pub fn fresh(&self, now: i64) -> bool {
        self.refreshed_at
            .is_some_and(|t| t <= now && now.saturating_sub(t) < MAX_AGE)
    }
    /// Availability and freshness remain separately accessible through last_error.
    pub fn state(&self, now: i64) -> DataState {
        if self.downloaded_at.is_none() {
            return if self.last_error.is_some() {
                DataState::Unavailable
            } else {
                DataState::Loading
            };
        }
        if self.refreshed_at.is_none() {
            return DataState::UnknownFreshness;
        }
        if !self.fresh(now) {
            return DataState::Stale;
        }
        if self.campaigns.iter().any(|c| c.active(self, now)) {
            DataState::Available
        } else {
            DataState::Empty
        }
    }
}

#[cfg(test)]
mod tests;
