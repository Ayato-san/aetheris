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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WindowSettings {
    pub latency_secs: u64,
    pub traffic_secs: u64,
    pub retransmissions_secs: u64,
}
impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            latency_secs: 10,
            traffic_secs: 10,
            retransmissions_secs: 900,
        }
    }
}
impl WindowSettings {
    #[cfg(debug_assertions)]
    fn windows(self) -> statistics::Windows {
        use std::time::Duration;
        statistics::Windows {
            latency: Duration::from_secs(self.latency_secs),
            traffic: Duration::from_secs(self.traffic_secs),
            retransmissions: Duration::from_secs(self.retransmissions_secs),
        }
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
        if !(1..=900).contains(&self.windows.latency_secs)
            || !(1..=900).contains(&self.windows.traffic_secs)
            || !(1..=3600).contains(&self.windows.retransmissions_secs)
        {
            return Err("Durée de fenêtre réseau invalide.".into());
        }
        Ok(())
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
        state
            .subscription
            .as_ref()
            .map(|subscription| frame(&state.measures, Some(&subscription.snapshot())))
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
                #[cfg(debug_assertions)]
                state
                    .subscription
                    .as_ref()
                    .unwrap()
                    .set_windows(settings.windows.windows())?;
                #[cfg(windows)]
                widget.window.configure(config)?;
            } else {
                let subscription = subscribe(interests(&measures))?;
                #[cfg(debug_assertions)]
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
