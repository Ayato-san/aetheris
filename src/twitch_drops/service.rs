//! A single process-wide worker owns networking and SQLite. HTTP runs off the UI.
use super::*;
use reqwest::{blocking::Client, header::HeaderMap};
use std::{
    io::Read,
    path::Path,
    sync::{Arc, OnceLock, RwLock, mpsc},
    time::Duration,
};

const BODY_LIMIT: u64 = 8 * 1024 * 1024;
type UpdateResult = std::result::Result<(), String>;
struct Update {
    id: String,
    user: UserState,
    reply: mpsc::Sender<UpdateResult>,
}
pub struct Service {
    shared: Arc<RwLock<Snapshot>>,
    updates: mpsc::Sender<Update>,
}
impl Service {
    pub fn snapshot(&self) -> Snapshot {
        self.shared.read().unwrap().clone()
    }
    /// Enqueue durable user decisions without blocking the UI on HTTP or SQLite.
    pub fn set_user_state(
        &self,
        id: String,
        user: UserState,
    ) -> Result<mpsc::Receiver<UpdateResult>> {
        let (reply, receiver) = mpsc::channel();
        self.updates
            .send(Update { id, user, reply })
            .map_err(|e| error(e.to_string()))?;
        Ok(receiver)
    }
    fn start(path: &Path) -> Result<Self> {
        let catalog = Catalog::open(path)?;
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(concat!("Aetheris/", env!("CARGO_PKG_VERSION")))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let shared = Arc::new(RwLock::new(catalog.snapshot().clone()));
        let worker_shared = shared.clone();
        let (updates, receiver) = mpsc::channel::<Update>();
        std::thread::Builder::new()
            .name("twitch-drops".into())
            .spawn(move || {
                let mut catalog = catalog;
                loop {
                    let current = now();
                    if current >= catalog.snapshot().next_attempt_at
                        && let Err(e) = refresh(&client, &mut catalog, SOURCE_URL, current)
                    {
                        // Even a disk failure must not cause a tight network retry loop.
                        catalog.snapshot.last_error = Some(e.to_string());
                        catalog.snapshot.next_attempt_at = current + 3_600_000;
                    }
                    if let Err(e) = catalog.prune(current) {
                        let mut snapshot = catalog.snapshot().clone();
                        snapshot.last_error = Some(e.to_string());
                        *worker_shared.write().unwrap() = snapshot;
                    } else {
                        *worker_shared.write().unwrap() = catalog.snapshot().clone();
                    }
                    match receiver.recv_timeout(Duration::from_secs(30)) {
                        Ok(update) => {
                            let result = catalog
                                .set_user_state(&update.id, update.user)
                                .map_err(|e| e.to_string());
                            *worker_shared.write().unwrap() = catalog.snapshot().clone();
                            let _ = update.reply.send(result);
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                    }
                }
            })?;
        Ok(Self { shared, updates })
    }
}

pub fn shared() -> Result<&'static Service> {
    static SERVICE: OnceLock<std::result::Result<Service, String>> = OnceLock::new();
    SERVICE
        .get_or_init(|| {
            crate::companion::data_directory()
                .and_then(|p| Service::start(&p.join("twitch-drops.sqlite3")))
                .map_err(|e| e.to_string())
        })
        .as_ref()
        .map_err(|e| error(e.clone()))
}

fn header_delay(headers: &HeaderMap, key: &str, checked: i64) -> i64 {
    let Some(value) = headers.get(key).and_then(|v| v.to_str().ok()) else {
        return 0;
    };
    if let Ok(seconds) = value.parse::<u64>() {
        return checked.saturating_add(seconds.saturating_mul(1000).min(i64::MAX as u64) as i64);
    }
    // Retry-After also permits an HTTP-date.
    chrono::DateTime::parse_from_rfc2822(value)
        .map(|t| t.timestamp_millis())
        .unwrap_or(0)
}

pub(super) fn refresh(
    client: &Client,
    catalog: &mut Catalog,
    url: &str,
    checked: i64,
) -> Result<()> {
    let mut minimum_next = 0;
    let outcome = (|| {
        let mut request = client.get(url);
        // Fetch a body before the known scan expires: an unchanged list can
        // otherwise yield 304 forever even while Twitch scans keep succeeding.
        if catalog.snapshot().fresh(checked.saturating_add(3_600_000))
            && let Some(etag) = &catalog.snapshot().etag
        {
            request = request.header("If-None-Match", etag);
        }
        let response = request.send()?;
        minimum_next = header_delay(response.headers(), "ratelimit-reset", checked)
            .max(header_delay(response.headers(), "retry-after", checked));
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            return catalog.unchanged(checked, minimum_next);
        }
        let response = response.error_for_status()?;
        if response.status() != reqwest::StatusCode::OK {
            return Err(error("Réponse source inattendue."));
        }
        let etag = response
            .headers()
            .get("etag")
            .and_then(|s| s.to_str().ok())
            .map(str::to_owned);
        let mut body = Vec::new();
        response.take(BODY_LIMIT + 1).read_to_end(&mut body)?;
        if body.len() as u64 > BODY_LIMIT {
            return Err(error("Réponse source trop volumineuse."));
        }
        catalog.ingest(&body, etag, checked, minimum_next)
    })();
    if let Err(e) = outcome {
        catalog.failed(checked, e.to_string(), minimum_next)?;
    }
    Ok(())
}
