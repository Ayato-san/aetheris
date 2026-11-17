//! Player-facing network widgets over the single on-demand statistics hub.
use super::widget::{TextFrame, WidgetConfig, WidgetContent};
use crate::capture::{
    Result,
    statistics::{self, Interests, Snapshot},
};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Measure {
    Ping,
    Jitter,
    Incoming,
    Outgoing,
    Retransmissions,
}
impl Measure {
    pub const ALL: [Self; 5] = [
        Self::Ping,
        Self::Jitter,
        Self::Incoming,
        Self::Outgoing,
        Self::Retransmissions,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Ping => "Ping",
            Self::Jitter => "Jitter",
            Self::Incoming => "Débit entrant",
            Self::Outgoing => "Débit sortant",
            Self::Retransmissions => "Retransmissions TCP",
        }
    }
    pub fn explanation(self) -> &'static str {
        match self {
            Self::Ping => {
                "Temps aller-retour moyen entre les données envoyées par le jeu et leur confirmation par le serveur. Un délai élevé peut retarder la réaction aux actions. Il inclut le temps de réponse TCP et un éventuel retard de lecture de la capture."
            }
            Self::Jitter => {
                "Variation moyenne entre deux temps aller-retour successifs. Une variation élevée peut rendre les réactions irrégulières. Il faut au moins deux mesures fiables."
            }
            Self::Incoming => {
                "Quantité de données reçues par seconde par le jeu, sur l’ensemble de ses connexions. Une hausse peut accompagner un chargement ou une scène plus active."
            }
            Self::Outgoing => {
                "Quantité de données envoyées par seconde par le jeu, sur l’ensemble de ses connexions. Elle dépend notamment de l’activité du jeu."
            }
            Self::Retransmissions => {
                "Nombre de segments TCP de données observés comme renvoyés, dans les deux sens, et leur part parmi tous les segments de données observés. Ces renvois peuvent accompagner des retards ; ce pourcentage ne mesure pas exactement les paquets perdus sur le réseau."
            }
        }
    }
}
fn interests(measures: &[Measure]) -> Interests {
    Interests {
        latency: measures
            .iter()
            .any(|m| matches!(m, Measure::Ping | Measure::Jitter)),
        traffic: measures
            .iter()
            .any(|m| matches!(m, Measure::Incoming | Measure::Outgoing)),
        retransmissions: measures.contains(&Measure::Retransmissions),
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub individual: [bool; 5],
    pub grouped: bool,
    /// Order is significant; duplicates across individual and grouped widgets are intentional.
    pub measures: Vec<Measure>,
    pub presentation: [WidgetConfig; 6],
    // Kept in release serialization so a release does not erase debug preferences.
    pub windows: WindowSettings,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            individual: [false; 5],
            grouped: false,
            measures: Measure::ALL.to_vec(),
            presentation: std::array::from_fn(|i| WidgetConfig {
                y: 180 + i as i32 * 40,
                width: 480,
                height: 120,
                ..WidgetConfig::default()
            }),
            windows: WindowSettings::default(),
        }
    }
}
/// Debug preferences remain serializable in release, but are never applied there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct WindowSettings {
    pub short_secs: u64,
    pub retransmissions_secs: u64,
}
impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            short_secs: 10,
            retransmissions_secs: 900,
        }
    }
}
impl<'de> serde::Deserialize<'de> for WindowSettings {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        // #52 stored separate latency/traffic windows. Migrate to the common
        // short window, preferring latency when both legacy values are present.
        #[derive(Default, serde::Deserialize)]
        #[serde(default, deny_unknown_fields)]
        struct Stored {
            #[serde(deserialize_with = "seconds")]
            short_secs: Option<u64>,
            #[serde(deserialize_with = "seconds")]
            latency_secs: Option<u64>,
            #[serde(deserialize_with = "seconds")]
            traffic_secs: Option<u64>,
            #[serde(deserialize_with = "seconds")]
            retransmissions_secs: Option<u64>,
        }
        fn seconds<'de, D: serde::Deserializer<'de>>(
            deserializer: D,
        ) -> std::result::Result<Option<u64>, D::Error> {
            serde::Deserialize::deserialize(deserializer).map(Some)
        }
        let stored = Stored::deserialize(deserializer)?;
        for short in [stored.short_secs, stored.latency_secs, stored.traffic_secs]
            .into_iter()
            .flatten()
        {
            if !(1..=900).contains(&short) {
                return Err(serde::de::Error::custom(
                    "La fenêtre courte doit être comprise entre 1 et 900 secondes.",
                ));
            }
        }
        let settings = Self {
            short_secs: stored
                .short_secs
                .or(stored.latency_secs)
                .or(stored.traffic_secs)
                .unwrap_or(10),
            retransmissions_secs: stored.retransmissions_secs.unwrap_or(900),
        };
        settings.validate().map_err(serde::de::Error::custom)?;
        Ok(settings)
    }
}
impl WindowSettings {
    pub fn validate(self) -> Result<()> {
        if !(1..=900).contains(&self.short_secs) || !(1..=3600).contains(&self.retransmissions_secs)
        {
            return Err(
                "Durée de fenêtre réseau invalide : courte 1–900 s, retransmissions 1–3 600 s."
                    .into(),
            );
        }
        Ok(())
    }
    fn windows(self) -> statistics::Windows {
        #[cfg(debug_assertions)]
        {
            use std::time::Duration;
            statistics::Windows {
                latency: Duration::from_secs(self.short_secs),
                traffic: Duration::from_secs(self.short_secs),
                retransmissions: Duration::from_secs(self.retransmissions_secs),
            }
        }
        #[cfg(not(debug_assertions))]
        statistics::Windows::default()
    }
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        if self.measures.len() > 5
            || self
                .measures
                .iter()
                .enumerate()
                .any(|(i, m)| self.measures[..i].contains(m))
        {
            return Err(
                "Chaque mesure ne peut figurer qu’une fois dans le widget regroupé.".into(),
            );
        }
        for config in &self.presentation {
            config.validate()?;
        }
        self.windows.validate()
    }
    fn widget(&self, index: usize) -> (WidgetConfig, Vec<Measure>) {
        let mut config = self.presentation[index];
        let measures = if index == 5 {
            self.measures.clone()
        } else {
            vec![Measure::ALL[index]]
        };
        config.enabled &= if index == 5 {
            self.grouped && !measures.is_empty()
        } else {
            self.individual[index]
        };
        (config, measures)
    }
}

fn rate(value: f64) -> String {
    if value < 1024. {
        format!("{value:.0} o/s")
    } else if value < 1024. * 1024. {
        format!("{:.1} Kio/s", value / 1024.)
    } else {
        format!("{:.1} Mio/s", value / (1024. * 1024.))
    }
}
/// Unavailable and zero are distinct. No warning thresholds or inferred loss.
pub fn frame(measures: &[Measure], snapshot: Option<&Snapshot>) -> TextFrame {
    let snapshot = snapshot.filter(|s| s.capture_available);
    let text = measures
        .iter()
        .map(|measure| {
            let value = snapshot
                .and_then(|s| match measure {
                    Measure::Ping if s.active.latency => s
                        .latency
                        .ping
                        .map(|v| format!("{:.0} ms", v.as_secs_f64() * 1000.)),
                    Measure::Jitter if s.active.latency => s
                        .latency
                        .jitter
                        .map(|v| format!("{:.0} ms", v.as_secs_f64() * 1000.)),
                    Measure::Incoming if s.active.traffic => s
                        .traffic
                        .rates
                        .as_ref()
                        .map(|v| rate(v.incoming_bytes_per_second)),
                    Measure::Outgoing if s.active.traffic => s
                        .traffic
                        .rates
                        .as_ref()
                        .map(|v| rate(v.outgoing_bytes_per_second)),
                    Measure::Retransmissions if s.active.retransmissions => {
                        s.retransmissions.counts.as_ref().map(|v| {
                            format!(
                                "{} · {}",
                                v.retransmissions,
                                v.percentage
                                    .map(|p| format!("{p:.1} %"))
                                    .unwrap_or_else(|| "—".into())
                            )
                        })
                    }
                    _ => None,
                })
                .unwrap_or_else(|| "—".into());
            format!("{} : {value}", measure.label())
        })
        .collect::<Vec<_>>()
        .join("\n");
    TextFrame {
        text,
        height: (measures.len() * 20) as u16,
    }
}

/// A window edit reaches the worker asynchronously. Hide only the values still
/// published with the previous window while it applies the requested settings.
fn frame_for_windows(
    measures: &[Measure],
    snapshot: &Snapshot,
    requested: statistics::Windows,
) -> TextFrame {
    if snapshot.windows == requested {
        return frame(measures, Some(snapshot));
    }
    let mut pending = snapshot.clone();
    if pending.windows.latency != requested.latency {
        pending.latency.ping = None;
        pending.latency.jitter = None;
    }
    if pending.windows.traffic != requested.traffic {
        pending.traffic.rates = None;
    }
    if pending.windows.retransmissions != requested.retransmissions {
        pending.retransmissions.counts = None;
    }
    frame(measures, Some(&pending))
}

struct State {
    measures: Vec<Measure>,
    subscription: Option<statistics::Subscription>,
}
#[derive(Clone)]
struct Source(Arc<Mutex<State>>);
impl WidgetContent for Source {
    type Frame = TextFrame;
    fn snapshot(&self) -> Option<TextFrame> {
        let state = self.0.lock().unwrap();
        state.subscription.as_ref().map(|subscription| {
            frame_for_windows(
                &state.measures,
                &subscription.snapshot(),
                subscription.windows(),
            )
        })
    }
}
struct LiveWidget {
    source: Source,
    #[cfg(windows)]
    window: super::WidgetWindow,
}
impl Drop for LiveWidget {
    fn drop(&mut self) {
        // Native rendering owns a source clone until its thread exits. Release
        // capture synchronously, independently of that thread's lifetime.
        self.source.0.lock().unwrap().subscription = None;
    }
}
#[derive(Default)]
pub struct Runtime {
    widgets: [Option<LiveWidget>; 6],
}
impl Runtime {
    pub fn configure(&mut self, settings: &Settings) -> Result<()> {
        self.configure_with(settings, statistics::subscribe)
    }
    pub(crate) fn configure_with(
        &mut self,
        settings: &Settings,
        mut subscribe: impl FnMut(Interests) -> Result<statistics::Subscription>,
    ) -> Result<()> {
        settings.validate()?;
        for index in 0..6 {
            let (config, measures) = settings.widget(index);
            if !config.enabled {
                self.widgets[index] = None;
                continue;
            }
            if let Some(widget) = &self.widgets[index] {
                let mut state = widget.source.0.lock().unwrap();
                if interests(&state.measures) != interests(&measures) {
                    // Acquire first so common histories survive composition edits.
                    state.subscription = Some(subscribe(interests(&measures))?);
                }
                state.measures = measures;
                state
                    .subscription
                    .as_ref()
                    .unwrap()
                    .set_windows(settings.windows.windows())?;
                #[cfg(windows)]
                widget.window.configure(config)?;
            } else {
                let subscription = subscribe(interests(&measures))?;
                subscription.set_windows(settings.windows.windows())?;
                let source = Source(Arc::new(Mutex::new(State {
                    measures,
                    subscription: Some(subscription),
                })));
                self.widgets[index] = Some(LiveWidget {
                    #[cfg(windows)]
                    window: super::WidgetWindow::new(config, source.clone())?,
                    source,
                });
            }
        }
        Ok(())
    }
    /// Latest values for the editor; reading these does not acquire capture.
    pub fn frames(&self) -> Vec<TextFrame> {
        self.widgets
            .iter()
            .flatten()
            .filter_map(|w| w.source.snapshot())
            .collect()
    }
}

#[cfg(test)]
mod tests;
