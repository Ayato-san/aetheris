use super::*;
use serde_json::json;

struct Directory(std::path::PathBuf);
impl Directory {
    fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("aetheris-drops-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../tests/fixtures/twitch-drops/sunkwi.json"
    ))
    .unwrap()
}
fn observed() -> i64 {
    timestamp(&fixture()["refreshedAt"]).unwrap()
}
fn catalog() -> Catalog {
    Catalog::open_at(Path::new(":memory:"), observed()).unwrap()
}
use std::path::Path;
fn ingest(c: &mut Catalog, value: Value, now: i64) {
    c.ingest(
        &serde_json::to_vec(&value).unwrap(),
        Some("tag".into()),
        now,
        0,
    )
    .unwrap();
}

#[test]
fn real_fixture_filters_game_and_preserves_benefit_names_and_unknown_conditions() {
    let scan = source::parse(&serde_json::to_vec(&fixture()).unwrap(), observed()).unwrap();
    assert_eq!(scan.campaigns.len(), 1);
    let c = &scan.campaigns[0];
    assert_eq!(c.id, "fab904fd-96c0-4b4c-8775-d2cb829f4563");
    assert_eq!(c.status.as_deref(), Some("ACTIVE"));
    assert_eq!(c.user.region.label(), "région non confirmée");
    assert!(
        c.rewards
            .as_ref()
            .unwrap()
            .iter()
            .any(|r| r.name.as_deref() == Some("Rare Theostone Chest x2")
                && r.tier_name.as_deref() == Some("Rare Theostone Chest x1"))
    );
    assert!(!c.raw["detailsURL"].is_null());
    let mut unknown = fixture();
    unknown["games"][0]["rewards"][0]["timeBasedDrops"] = Value::Null;
    unknown["games"][0]["rewards"][0]["allow"] = json!({"futureCondition": 42});
    let scan = source::parse(&serde_json::to_vec(&unknown).unwrap(), observed()).unwrap();
    assert!(scan.campaigns[0].rewards.is_none());
    assert_eq!(scan.campaigns[0].raw["allow"]["futureCondition"], 42);
}

#[test]
fn simultaneous_campaigns_keep_source_identity_despite_title_and_date_changes() {
    let mut value = fixture();
    let mut second = value["games"][0]["rewards"][0].clone();
    second["id"] = json!("distinct");
    value["games"][0]["rewards"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let mut c = catalog();
    ingest(&mut c, value.clone(), observed());
    assert_eq!(c.snapshot().campaigns.len(), 2);
    value["games"][0]["rewards"][0]["name"] = json!("renamed");
    value["games"][0]["rewards"][0]["startAt"] = json!("2026-11-03T14:00:00Z");
    ingest(&mut c, value, observed());
    assert_eq!(c.snapshot().campaigns.len(), 2);
}

#[test]
fn invalid_responses_are_atomic_and_do_not_imply_disappearance() {
    let mut c = catalog();
    ingest(&mut c, fixture(), observed());
    let before = serde_json::to_value(c.snapshot()).unwrap();
    let mut invalids = vec![
        json!({}),
        json!({"games":null}),
        json!({"games":[{"gameDisplayName":"AION 2"}]}),
    ];
    for field in ["startAt", "endAt"] {
        let mut v = fixture();
        v["games"][0]["rewards"][0][field] = json!("invalid");
        invalids.push(v);
    }
    let mut reversed = fixture();
    reversed["games"][0]["rewards"][0]["endAt"] = json!("2000-01-01T00:00:00Z");
    invalids.push(reversed);
    let mut duplicate = fixture();
    let entry = duplicate["games"][0]["rewards"][0].clone();
    duplicate["games"][0]["rewards"]
        .as_array_mut()
        .unwrap()
        .push(entry);
    invalids.push(duplicate);
    for invalid in invalids {
        assert!(
            c.ingest(&serde_json::to_vec(&invalid).unwrap(), None, observed(), 0)
                .is_err()
        );
        assert_eq!(serde_json::to_value(c.snapshot()).unwrap(), before);
    }
}

#[test]
fn disappearance_reappearance_and_user_state_survive_database_reopen() {
    let directory = Directory::new();
    let path = directory.0.join("drops.sqlite3");
    let mut c = Catalog::open_at(&path, observed()).unwrap();
    ingest(&mut c, fixture(), observed());
    let id = c.snapshot().campaigns[0].id.clone();
    let mut user = UserState {
        hidden: true,
        ..UserState::default()
    };
    user.delivered_reminders.insert("start".into());
    c.set_user_state(&id, user).unwrap();
    ingest(
        &mut c,
        json!({"games":[],"refreshedAt":fixture()["refreshedAt"]}),
        observed(),
    );
    assert!(!c.snapshot().campaigns[0].present);
    assert_eq!(c.snapshot().state(observed()), DataState::Empty);
    drop(c);
    let mut c = Catalog::open_at(&path, observed()).unwrap();
    ingest(&mut c, fixture(), observed());
    let campaign = &c.snapshot().campaigns[0];
    assert!(campaign.present && campaign.user.hidden);
    assert!(campaign.user.delivered_reminders.contains("start"));
    assert!(campaign.active(c.snapshot(), observed()));
    assert!(!campaign.visible_in_widget(c.snapshot(), observed()));
    c.prune(campaign.end_at + YEAR).unwrap();
    assert!(c.snapshot().campaigns.is_empty());
    ingest(&mut c, fixture(), observed());
    assert!(c.snapshot().campaigns[0].user.hidden);
}

#[test]
fn freshness_expires_offline_and_304_never_advances_scan_time() {
    let mut c = catalog();
    assert_eq!(c.snapshot().state(observed()), DataState::Loading);
    ingest(&mut c, fixture(), observed());
    let scan = c.snapshot().refreshed_at;
    c.unchanged(observed() + MAX_AGE, 0).unwrap();
    assert_eq!(c.snapshot().refreshed_at, scan);
    assert_eq!(c.snapshot().state(observed() + MAX_AGE), DataState::Stale);
    assert!(!c.snapshot().campaigns[0].active(c.snapshot(), observed() + MAX_AGE));
    let mut v = fixture();
    v["refreshedAt"] = json!("invalid");
    ingest(&mut c, v, observed());
    assert_eq!(c.snapshot().state(observed()), DataState::UnknownFreshness);
    assert!(!c.snapshot().campaigns[0].active(c.snapshot(), observed()));
}

#[test]
fn stale_or_unknown_scan_does_not_mark_campaign_missing() {
    for date in [Value::Null, json!("invalid"), json!("2020-01-01T00:00:00Z")] {
        let mut c = catalog();
        ingest(&mut c, fixture(), observed());
        let result = c.ingest(
            &serde_json::to_vec(&json!({"games":[],"refreshedAt":date})).unwrap(),
            None,
            observed(),
            0,
        );
        if result.is_ok() {
            assert!(c.snapshot().campaigns[0].present);
        }
        assert!(c.snapshot().campaigns[0].present);
    }
}

#[test]
fn dates_status_and_region_gate_activity_independently_of_hiding() {
    let mut c = catalog();
    ingest(&mut c, fixture(), observed());
    let campaign = &c.snapshot().campaigns[0];
    assert!(campaign.active(c.snapshot(), observed()));
    assert!(!campaign.active(c.snapshot(), campaign.start_at - 1));
    assert!(!campaign.active(c.snapshot(), campaign.end_at));
    let mut changed = campaign.clone();
    changed.status = None;
    assert!(!changed.active(c.snapshot(), observed()));
    changed.status = Some("ACTIVE".into());
    changed.user.region = Region::Incompatible;
    assert!(!changed.active(c.snapshot(), observed()));
    assert!(
        c.set_user_state(
            "id",
            UserState {
                region: Region::Europe,
                ..UserState::default()
            }
        )
        .is_err()
    );
}

/// Local HTTP fixtures are independent from the real-provider validation.
fn http_reply(
    status: &str,
    headers: &str,
    body: &str,
    expected_etag: bool,
) -> (String, std::thread::JoinHandle<()>) {
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let response = format!(
        "HTTP/1.1 {status}\r\nConnection: close\r\nContent-Length: {}\r\n{headers}\r\n{body}",
        body.len()
    );
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        let mut request = Vec::new();
        let mut byte = [0];
        while !request.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        assert_eq!(
            String::from_utf8(request)
                .unwrap()
                .to_lowercase()
                .contains("if-none-match: tag"),
            expected_etag
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, handle)
}

#[test]
fn http_429_invalid_body_and_304_preserve_history_and_honor_limits() {
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap();
    let mut c = catalog();
    ingest(&mut c, fixture(), observed());
    for (status, headers, body) in [
        ("429 Too Many Requests", "Retry-After: 7200\r\n", "{}"),
        ("200 OK", "", "{broken"),
        ("304 Not Modified", "", ""),
    ] {
        let (url, handle) = http_reply(status, headers, body, true);
        service::refresh(&client, &mut c, &url, observed()).unwrap();
        handle.join().unwrap();
        assert!(c.snapshot().campaigns[0].present);
        assert_eq!(c.snapshot().refreshed_at, Some(observed()));
        if status.starts_with("429") {
            assert!(c.snapshot().next_attempt_at >= observed() + 7_200_000);
        }
        if status.starts_with("304") {
            assert!(c.snapshot().last_error.is_none());
        } else {
            assert!(c.snapshot().last_error.is_some());
        }
    }
}

#[test]
fn http_success_empty_and_initial_failure_have_distinct_states() {
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .build()
        .unwrap();
    let mut c = catalog();
    let (url, handle) = http_reply("503 Service Unavailable", "", "", false);
    service::refresh(&client, &mut c, &url, observed()).unwrap();
    handle.join().unwrap();
    assert_eq!(c.snapshot().state(observed()), DataState::Unavailable);
    let (url, handle) = http_reply(
        "200 OK",
        "ETag: empty\r\n",
        &json!({"games":[],"refreshedAt":fixture()["refreshedAt"]}).to_string(),
        false,
    );
    service::refresh(&client, &mut c, &url, observed()).unwrap();
    handle.join().unwrap();
    assert_eq!(c.snapshot().state(observed()), DataState::Empty);
    assert_eq!(c.snapshot().etag.as_deref(), Some("empty"));
}

#[test]
fn expired_etag_fetches_body_and_http_date_limits_survive_restart() {
    let directory = Directory::new();
    let path = directory.0.join("drops.sqlite3");
    let mut c = Catalog::open_at(&path, observed()).unwrap();
    ingest(&mut c, fixture(), observed());
    let checked = observed() + MAX_AGE;
    let retry_at = checked + 8 * 3_600_000;
    let retry_date = chrono::DateTime::from_timestamp_millis(retry_at)
        .unwrap()
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string();
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .build()
        .unwrap();
    let (url, handle) = http_reply(
        "429 Too Many Requests",
        &format!("Retry-After: {retry_date}\r\n"),
        "{}",
        false,
    );
    service::refresh(&client, &mut c, &url, checked).unwrap();
    handle.join().unwrap();
    let deadline = c.snapshot().next_attempt_at;
    assert!(deadline >= retry_at - 1000);
    drop(c);
    let mut c = Catalog::open_at(&path, checked).unwrap();
    assert_eq!(c.snapshot().next_attempt_at, deadline);
    assert_eq!(c.snapshot().refreshed_at, Some(observed()));
    let mut value = fixture();
    value["refreshedAt"] = json!(
        chrono::DateTime::from_timestamp_millis(checked)
            .unwrap()
            .to_rfc3339()
    );
    let (url, handle) = http_reply("200 OK", "ETag: fresh\r\n", &value.to_string(), false);
    service::refresh(&client, &mut c, &url, checked).unwrap();
    handle.join().unwrap();
    assert_eq!(c.snapshot().refreshed_at, Some(checked));
    assert_eq!(c.snapshot().failures, 0);
    assert_eq!(c.snapshot().downloaded_at, Some(checked));
    assert_eq!(c.snapshot().next_attempt_at, checked + 3_600_000);
}

#[test]
fn backoff_increases_without_advancing_freshness_and_unproven_304_is_rejected() {
    let mut c = catalog();
    assert!(c.unchanged(observed(), 0).is_err());
    for (attempt, delay) in [(1, 600_000), (2, 1_200_000), (3, 2_400_000), (4, 3_600_000)] {
        c.failed(observed(), "offline".into(), 0).unwrap();
        assert_eq!(c.snapshot().failures, attempt);
        let actual = c.snapshot().next_attempt_at - observed();
        assert!((delay..delay + 60_000).contains(&actual));
        assert!(c.snapshot().refreshed_at.is_none());
    }
}
