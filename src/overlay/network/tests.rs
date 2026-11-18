use super::*;
use crate::capture::{latency, observations::Selection, retransmissions, throughput};
use std::time::Duration;

fn snapshot() -> Snapshot {
    Snapshot {
        windows: statistics::Windows::default(),
        active: Interests::ALL,
        capture_available: true,
        error: None,
        collector_drops: None,
        latency: latency::Snapshot {
            selection: Selection::Absent,
            ping: Some(Duration::from_micros(42300)),
            jitter: Some(Duration::from_micros(1700)),
            last_rtt: Some(Duration::from_micros(42300)),
            max_rtt: Some(Duration::from_micros(42300)),
            last_sample_age: Some(Duration::from_millis(250)),
            awaiting_ack: None,
            samples: 2,
            recovering: false,
        },
        traffic: throughput::Snapshot {
            rates: Some(throughput::Rates {
                incoming_bytes_per_second: 2048.,
                outgoing_bytes_per_second: 3. * 1024. * 1024.,
            }),
            recovering: false,
        },
        retransmissions: retransmissions::Snapshot {
            selection: Selection::Absent,
            counts: Some(retransmissions::Counts {
                retransmissions: 2,
                data_segments: 8,
                percentage: Some(25.),
            }),
            recovering: false,
        },
    }
}
#[test]
fn values_units_retransmission_count_and_percentage_follow_the_requested_order() {
    let data = snapshot();
    assert_eq!(frame(&[Measure::Outgoing, Measure::Ping, Measure::Retransmissions, Measure::Jitter, Measure::Incoming], Some(&data)), TextFrame {
        text: "Débit sortant : 3.0 Mio/s\nPing : moy. 42 ms · dernier 42 ms · max 42 ms\nÂge échantillon : 0.2 s · attente ACK : —\nRetransmissions TCP : 2 · 25.0 %\nJitter : 2 ms\nDébit entrant : 2.0 Kio/s".into(), height: 120,
    });
    for measure in Measure::ALL {
        assert_eq!(
            frame(&[measure], Some(&data)).height,
            if measure == Measure::Ping { 40 } else { 20 }
        );
    }
}
#[test]
fn unavailable_zero_and_no_percentage_remain_distinct() {
    let mut data = snapshot();
    data.latency.ping = None;
    data.latency.last_rtt = None;
    data.latency.max_rtt = None;
    data.latency.last_sample_age = None;
    data.latency.jitter = Some(Duration::ZERO);
    data.traffic.rates = Some(throughput::Rates {
        incoming_bytes_per_second: 0.,
        outgoing_bytes_per_second: 1023.,
    });
    data.retransmissions.counts = Some(retransmissions::Counts {
        retransmissions: 0,
        data_segments: 0,
        percentage: None,
    });
    let values = frame(&Measure::ALL, Some(&data));
    assert_eq!(
        values.text,
        "Ping : moy. — · dernier — · max —\nÂge échantillon : — · attente ACK : —\nJitter : 0 ms\nDébit entrant : 0 o/s\nDébit sortant : 1023 o/s\nRetransmissions TCP : 0 · —"
    );
    data.capture_available = false;
    assert_eq!(
        frame(&Measure::ALL, Some(&data)),
        frame(&Measure::ALL, None)
    );
    data.capture_available = true;
    data.active = Interests::default();
    assert_eq!(
        frame(&Measure::ALL, Some(&data)),
        frame(&Measure::ALL, None)
    );
}
#[test]
fn settings_persist_composition_duplicates_and_common_presentation() {
    let old: super::super::Settings = serde_json::from_str(r#"{"hidden_cubes":true}"#).unwrap();
    assert!(!old.network.grouped);
    assert_eq!(old.network.individual, [false; 5]);
    let mut settings = super::super::Settings::default();
    settings.network.individual = [true, false, true, false, true];
    settings.network.grouped = true;
    settings.network.measures = vec![Measure::Retransmissions, Measure::Incoming, Measure::Ping];
    settings.network.presentation[5].zoom = 1.5;
    settings.network.presentation[5].x = -20;
    settings.network.presentation[5].privacy = super::super::widget::Privacy::ExcludeFromCapture;
    settings.network.windows.short_secs = 30;
    let path =
        std::env::temp_dir().join(format!("aetheris-network-52-{}.json", std::process::id()));
    settings.save(&path).unwrap();
    let loaded = super::super::Settings::load(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(loaded.network, settings.network);
    assert!(loaded.network.widget(0).0.enabled);
    assert!(loaded.network.widget(5).0.enabled);
    assert!(loaded.network.widget(5).1.contains(&Measure::Ping));
}
#[test]
fn empty_group_has_no_interest_and_invalid_settings_are_rejected() {
    let mut settings = Settings {
        grouped: true,
        measures: vec![],
        ..Settings::default()
    };
    assert!(!settings.widget(5).0.enabled);
    assert_eq!(interests(&settings.measures), Interests::default());
    settings.measures = vec![Measure::Ping, Measure::Ping];
    assert!(settings.validate().is_err());
    settings.measures = vec![Measure::Ping];
    settings.windows.short_secs = 0;
    assert!(settings.validate().is_err());
    settings.windows = WindowSettings::default();
    settings.presentation[0].zoom = f32::NAN;
    assert!(settings.validate().is_err());
}
#[test]
fn unavailable_and_updated_frames_keep_widget_visibility_and_layout_rules() {
    use super::super::widget::{Growth, Widget};
    let mut config = WidgetConfig {
        y: 400,
        growth: Growth::BottomUp,
        zoom: 2.,
        ..WidgetConfig::default()
    };
    let mut widget = Widget::new(config, frame(&Measure::ALL, None)).unwrap();
    assert_eq!(widget.show().unwrap().height, 120);
    assert_eq!(config.layout(100, 0).y, 200);
    widget.update_content(frame(&[Measure::Ping], Some(&snapshot())));
    assert_eq!(
        widget.show().unwrap().text,
        "Ping : moy. 42 ms · dernier 42 ms · max 42 ms\nÂge échantillon : 0.2 s · attente ACK : —"
    );
    config.enabled = false;
    widget.configure(config).unwrap();
    assert!(widget.show().is_none());
}

#[test]
fn debug_windows_validate_boundaries_and_reject_invalid_stored_values() {
    for short_secs in [1, 900] {
        for retransmissions_secs in [1, 3600] {
            assert!(
                WindowSettings {
                    short_secs,
                    retransmissions_secs
                }
                .validate()
                .is_ok()
            );
        }
    }
    for invalid in [
        WindowSettings {
            short_secs: 0,
            ..WindowSettings::default()
        },
        WindowSettings {
            short_secs: 901,
            ..WindowSettings::default()
        },
        WindowSettings {
            retransmissions_secs: 0,
            ..WindowSettings::default()
        },
        WindowSettings {
            retransmissions_secs: 3601,
            ..WindowSettings::default()
        },
        WindowSettings {
            short_secs: u64::MAX,
            ..WindowSettings::default()
        },
    ] {
        assert!(invalid.validate().is_err());
    }
    for json in [
        r#"{"short_secs":0}"#,
        r#"{"short_secs":901}"#,
        r#"{"short_secs":-1}"#,
        r#"{"short_secs":1.5}"#,
        r#"{"short_secs":"10"}"#,
        r#"{"short_secs":null}"#,
        r#"{"retransmissions_secs":0}"#,
        r#"{"retransmissions_secs":3601}"#,
        r#"{"unknown":10}"#,
    ] {
        assert!(
            serde_json::from_str::<WindowSettings>(json).is_err(),
            "{json}"
        );
    }
}
#[test]
fn legacy_debug_windows_migrate_to_one_short_window() {
    let settings: WindowSettings =
        serde_json::from_str(r#"{"latency_secs":20,"traffic_secs":30,"retransmissions_secs":600}"#)
            .unwrap();
    assert_eq!(
        settings,
        WindowSettings {
            short_secs: 20,
            retransmissions_secs: 600
        }
    );
    let stored = serde_json::to_value(settings).unwrap();
    assert_eq!(
        stored,
        serde_json::json!({"short_secs":20,"retransmissions_secs":600})
    );
    let traffic_only: WindowSettings = serde_json::from_str(r#"{"traffic_secs":25}"#).unwrap();
    assert_eq!(traffic_only.short_secs, 25);
    assert_eq!(
        serde_json::from_str::<WindowSettings>("{}").unwrap(),
        WindowSettings::default()
    );
}
#[test]
fn debug_window_preferences_survive_disk_roundtrip_and_reset_without_active_widgets() {
    let path =
        std::env::temp_dir().join(format!("aetheris-network-53-{}.json", std::process::id()));
    let mut settings = super::super::Settings::default();
    settings.network.windows = WindowSettings {
        short_secs: 30,
        retransmissions_secs: 1200,
    };
    settings.save(&path).unwrap();
    let mut loaded = super::super::Settings::load(&path).unwrap();
    assert_eq!(loaded.network.windows, settings.network.windows);
    assert_eq!(loaded.network.individual, [false; 5]);
    loaded.network.windows.short_secs = 0;
    assert!(loaded.save(&path).is_err());
    assert_eq!(
        super::super::Settings::load(&path).unwrap().network.windows,
        settings.network.windows
    );
    loaded.network.windows = WindowSettings::default();
    loaded.save(&path).unwrap();
    assert_eq!(
        super::super::Settings::load(&path).unwrap().network.windows,
        WindowSettings::default()
    );
    std::fs::remove_file(path).unwrap();
}
#[test]
fn changing_requested_windows_immediately_hides_only_affected_published_values() {
    let data = snapshot();
    let mut windows = data.windows;
    windows.latency = Duration::from_secs(30);
    windows.traffic = Duration::from_secs(30);
    let pending = frame_for_windows(&Measure::ALL, &data, windows);
    assert!(
        pending
            .text
            .starts_with("Ping : moy. — · dernier — · max —\nÂge échantillon : — · attente ACK : —\nJitter : —\nDébit entrant : —\nDébit sortant : —")
    );
    assert!(pending.text.ends_with("Retransmissions TCP : 2 · 25.0 %"));
    windows = data.windows;
    windows.retransmissions = Duration::from_secs(1200);
    let pending = frame_for_windows(&Measure::ALL, &data, windows);
    assert!(
        pending
            .text
            .starts_with("Ping : moy. 42 ms · dernier 42 ms · max 42 ms\nÂge échantillon : 0.2 s · attente ACK : —\nJitter : 2 ms\nDébit entrant : 2.0 Kio/s")
    );
    assert!(pending.text.ends_with("Retransmissions TCP : —"));
    assert_eq!(
        frame_for_windows(&Measure::ALL, &data, data.windows),
        frame(&Measure::ALL, Some(&data))
    );
}
#[test]
fn effective_windows_are_custom_in_debug_and_defaults_in_production() {
    let settings = WindowSettings {
        short_secs: 30,
        retransmissions_secs: 1200,
    };
    let effective = settings.windows();
    if cfg!(debug_assertions) {
        assert_eq!(effective.latency, Duration::from_secs(30));
        assert_eq!(effective.traffic, Duration::from_secs(30));
        assert_eq!(effective.retransmissions, Duration::from_secs(1200));
    } else {
        assert_eq!(effective, statistics::Windows::default());
    }
}

#[test]
fn ping_spike_and_pending_ack_are_visible_in_individual_and_grouped_widgets() {
    let mut data = snapshot();
    data.latency.ping = Some(Duration::from_micros(24_850));
    data.latency.last_rtt = Some(Duration::from_secs(1));
    data.latency.max_rtt = Some(Duration::from_secs(1));
    data.latency.last_sample_age = Some(Duration::from_millis(1500));
    data.latency.awaiting_ack = Some(Duration::from_millis(1200));
    let individual = frame(&[Measure::Ping], Some(&data));
    assert_eq!(
        individual.text,
        "Ping : moy. 25 ms · dernier 1000 ms · max 1000 ms\nÂge échantillon : 1.5 s · attente ACK : 1.2 s"
    );
    let grouped = frame(&Measure::ALL, Some(&data));
    assert!(grouped.text.starts_with(&individual.text));
    let settings = Settings::default();
    assert!(individual.height <= settings.presentation[0].height);
    assert!(grouped.height <= settings.presentation[5].height);
    // Existing default positions leave 40 px for the now two-line ping.
    let ping = settings.presentation[0].layout(i32::from(individual.height), 0);
    assert!(ping.y + ping.height <= settings.presentation[1].y);
}

#[test]
fn an_expired_last_sample_is_aged_without_reusing_it_as_a_current_mean_or_peak() {
    let mut data = snapshot();
    data.latency.ping = None;
    data.latency.jitter = None;
    data.latency.max_rtt = None;
    data.latency.last_sample_age = Some(Duration::from_secs(12));
    data.latency.awaiting_ack = Some(Duration::from_secs(11));
    let text = frame(&[Measure::Ping], Some(&data)).text;
    assert_eq!(
        text,
        "Ping : moy. — · dernier 42 ms · max —\nÂge échantillon : 12.0 s · attente ACK : 11.0 s"
    );
    data.latency.recovering = true;
    assert_eq!(
        frame(&[Measure::Ping], Some(&data)),
        frame(&[Measure::Ping], None)
    );
    data.latency.recovering = false;
    data.capture_available = false;
    assert_eq!(
        frame(&[Measure::Ping], Some(&data)),
        frame(&[Measure::Ping], None)
    );
}
