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
        text: "Débit sortant : 3.0 Mio/s\nPing : 42 ms\nRetransmissions TCP : 2 · 25.0 %\nJitter : 2 ms\nDébit entrant : 2.0 Kio/s".into(), height: 100,
    });
    for measure in Measure::ALL {
        assert_eq!(frame(&[measure], Some(&data)).height, 20);
    }
}
#[test]
fn unavailable_zero_and_no_percentage_remain_distinct() {
    let mut data = snapshot();
    data.latency.ping = None;
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
        "Ping : —\nJitter : 0 ms\nDébit entrant : 0 o/s\nDébit sortant : 1023 o/s\nRetransmissions TCP : 0 · —"
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
    settings.network.windows.latency_secs = 30;
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
    settings.windows.traffic_secs = 0;
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
    assert_eq!(widget.show().unwrap().height, 100);
    assert_eq!(config.layout(100, 0).y, 200);
    widget.update_content(frame(&[Measure::Ping], Some(&snapshot())));
    assert_eq!(widget.show().unwrap().text, "Ping : 42 ms");
    config.enabled = false;
    widget.configure(config).unwrap();
    assert!(widget.show().is_none());
}
