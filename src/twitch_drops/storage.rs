use super::*;
use crate::companion::migrations::{Migration, initialize_personal};
use rusqlite::{Connection, OptionalExtension, params};
use std::{path::Path, time::Duration};

/// Owned by the worker; readers receive an immutable shared snapshot.
pub struct Catalog {
    connection: Connection,
    pub(super) snapshot: Snapshot,
}
impl Catalog {
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_at(path, now())
    }
    pub(super) fn open_at(path: &Path, current: i64) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let mut connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")?;
        initialize_personal(&mut connection, path, &[Migration { id: 1, sql:
            "CREATE TABLE drops_snapshot(id INTEGER PRIMARY KEY CHECK(id=1), json TEXT NOT NULL);
             CREATE TABLE drops_user_state(id TEXT PRIMARY KEY, json TEXT NOT NULL);"
        }])?;
        let json: Option<String> = connection
            .query_row("SELECT json FROM drops_snapshot WHERE id=1", [], |r| {
                r.get(0)
            })
            .optional()?;
        let snapshot = json
            .map(|s| serde_json::from_str(&s))
            .transpose()?
            .unwrap_or_default();
        let mut catalog = Self {
            connection,
            snapshot,
        };
        catalog.prune(current)?;
        Ok(catalog)
    }
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
    fn save(&mut self, next: Snapshot) -> Result<()> {
        self.connection.execute("INSERT INTO drops_snapshot VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json", [serde_json::to_string(&next)?])?;
        self.snapshot = next;
        Ok(())
    }
    /// User state has its own lifetime; retaining a tiny identity tombstone avoids
    /// assigning the same source ID to a new local identity after history expires.
    pub fn set_user_state(&mut self, id: &str, user: UserState) -> Result<()> {
        if user.region != Region::Unconfirmed
            && user
                .region_evidence
                .as_deref()
                .is_none_or(|s| s.trim().is_empty())
        {
            return Err(error("Une région confirmée nécessite une justification."));
        }
        let mut next = self.snapshot.clone();
        if let Some(c) = next.campaigns.iter_mut().find(|c| c.id == id) {
            c.user = user.clone();
        }
        let tx = self.connection.transaction()?;
        tx.execute("INSERT INTO drops_user_state VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET json=excluded.json", params![id, serde_json::to_string(&user)?])?;
        tx.execute("INSERT INTO drops_snapshot VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json", [serde_json::to_string(&next)?])?;
        tx.commit()?;
        self.snapshot = next;
        Ok(())
    }
    pub fn prune(&mut self, now: i64) -> Result<()> {
        let mut next = self.snapshot.clone();
        next.campaigns
            .retain(|c| now.saturating_sub(c.end_at) < YEAR);
        if next.campaigns.len() != self.snapshot.campaigns.len() {
            self.save(next)?;
        }
        Ok(())
    }
    pub(super) fn ingest(
        &mut self,
        body: &[u8],
        etag: Option<String>,
        checked: i64,
        minimum_next: i64,
    ) -> Result<()> {
        let scan = source::parse(body, checked)?;
        // Do not let a replayed older scan overwrite a newer successful observation.
        if let (Some(old), Some(new)) = (self.snapshot.refreshed_at, scan.refreshed_at)
            && new < old
        {
            return Err(error("Scan source antérieur au dernier scan observé."));
        }
        let mut next = self.snapshot.clone();
        let fresh = scan
            .refreshed_at
            .is_some_and(|t| checked.saturating_sub(t) < MAX_AGE);
        if fresh {
            for c in &mut next.campaigns {
                c.present = false;
            }
        }
        for mut campaign in scan.campaigns {
            let user: Option<String> = self
                .connection
                .query_row(
                    "SELECT json FROM drops_user_state WHERE id=?1",
                    [&campaign.id],
                    |r| r.get(0),
                )
                .optional()?;
            campaign.user = user
                .map(|s| serde_json::from_str(&s))
                .transpose()?
                .unwrap_or_default();
            if let Some(old) = next.campaigns.iter_mut().find(|c| c.id == campaign.id) {
                campaign.first_observed_at = old.first_observed_at;
                *old = campaign;
            } else {
                next.campaigns.push(campaign);
            }
        }
        next.campaigns
            .retain(|c| checked.saturating_sub(c.end_at) < YEAR);
        next.refreshed_at = scan.refreshed_at;
        next.downloaded_at = Some(checked);
        next.last_checked_at = Some(checked);
        next.last_error = None;
        next.etag = etag;
        next.failures = 0;
        next.next_attempt_at = (checked + 3_600_000)
            .max(minimum_next)
            .max(scan.reset_at.unwrap_or(0));
        self.save(next)
    }
    pub(super) fn unchanged(&mut self, checked: i64, minimum_next: i64) -> Result<()> {
        if self.snapshot.downloaded_at.is_none() {
            return Err(error("304 sans réponse locale."));
        }
        let mut next = self.snapshot.clone();
        next.last_checked_at = Some(checked);
        next.last_error = None;
        next.failures = 0;
        next.next_attempt_at = (checked + 3_600_000).max(minimum_next);
        // An ETag validates content, not the date of the last Twitch scan.
        self.save(next)
    }
    pub(super) fn failed(
        &mut self,
        checked: i64,
        message: String,
        minimum_next: i64,
    ) -> Result<()> {
        let mut next = self.snapshot.clone();
        next.failures = next.failures.saturating_add(1);
        let backoff = (300_000_i64 * (1_i64 << next.failures.min(5))).min(3_600_000);
        // Jitter prevents installations behind one public IP retrying in lockstep.
        let mut bytes = [0; 2];
        getrandom::fill(&mut bytes).map_err(|e| error(e.to_string()))?;
        let jitter = i64::from(u16::from_le_bytes(bytes)) % 60_000;
        next.next_attempt_at = (checked + backoff + jitter).max(minimum_next);
        next.last_checked_at = Some(checked);
        next.last_error = Some(message);
        self.save(next)
    }
}
