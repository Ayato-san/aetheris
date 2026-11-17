use super::OverlayEditor;
use crate::ui::{components::Button, theme};
#[cfg(debug_assertions)]
use aetheris::overlay::network::WindowSettings;
use aetheris::overlay::{
    network::{Measure, Settings},
    widget::{Growth, Privacy},
};
use gpui::{Context, div, prelude::*, rgb};

impl OverlayEditor {
    fn edit_network(&mut self, edit: impl FnOnce(&mut Settings), cx: &mut Context<Self>) {
        let mut settings = self.settings.clone();
        edit(&mut settings.network);
        let result = aetheris::overlay::Settings::path()
            .and_then(|path| settings.save(&path))
            .and_then(|()| self.network.configure(&settings.network));
        match result {
            Ok(()) => {
                self.settings = settings;
                self.error = None;
                self.network_frames = self.network.frames();
            }
            Err(error) => self.error = Some(format!("Choix réseau non appliqué : {error}")),
        }
        cx.notify();
    }
    pub(super) fn network_controls(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let settings = &self.settings.network;
        let mut panel = div().flex().flex_col().gap_4()
            .child(div().text_2xl().child("Réseau"))
            .child(div().text_sm().text_color(rgb(theme::TEXT_MUTED)).child("Un débit seul ne qualifie pas la connexion. Ces mesures ne déterminent pas à elles seules la cause d’un freeze : le jeu, le serveur ou l’ordinateur peuvent aussi intervenir. Aucune alerte ni seuil de couleur n’est appliqué."))
            .child(div().text_sm().child("Fenêtres par défaut : ping et jitter 10 s, débits 10 s, retransmissions 15 min. Actualisation environ chaque seconde. « — » signifie indisponible ; 0 est une valeur valide. Après un trou de capture, les mesures touchées attendent une fenêtre complète fiable. Ping, jitter et retransmissions nécessitent une connexion de jeu identifiée sans ambiguïté ; une reconnexion efface leur historique."))
            .child(div().text_sm().child("Les débits comptent les octets capturés du jeu, en-têtes réseau inclus. 1 Kio/s = 1 024 o/s ; 1 Mio/s = 1 024 Kio/s. Seuls les widgets activés demandent la collecte ; désactiver le dernier efface l’historique réseau."));
        for (index, measure) in Measure::ALL.into_iter().enumerate() {
            panel = panel.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        Button::new(
                            ("network-individual", index),
                            format!(
                                "{} · {}",
                                measure.label(),
                                if settings.individual[index] {
                                    "Activé"
                                } else {
                                    "Désactivé"
                                }
                            ),
                        )
                        .selected(settings.individual[index])
                        .element()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.edit_network(|s| s.individual[index] = !s.individual[index], cx);
                        })),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(theme::TEXT_MUTED))
                            .child(measure.explanation()),
                    )
                    .when(settings.individual[index], |d| {
                        d.child(self.network_presentation(index, cx))
                    }),
            );
        }
        panel = panel.child(Button::new("network-grouped", if settings.grouped { "Widget réseau regroupé · Activé" } else { "Widget réseau regroupé · Désactivé" })
            .selected(settings.grouped).element().on_click(cx.listener(|this, _, _, cx| this.edit_network(|s| s.grouped = !s.grouped, cx))))
            .child(div().text_sm().child("Choisissez les mesures puis utilisez Monter et Descendre pour changer leur ordre. Le regroupement peut afficher les mêmes mesures que les widgets individuels. Une liste vide n’affiche rien et ne demande aucune collecte."));
        for (index, measure) in Measure::ALL.into_iter().enumerate() {
            let selected = settings.measures.contains(&measure);
            panel = panel.child(
                Button::new(
                    ("network-include", index),
                    format!("{} {}", if selected { "✓" } else { "+" }, measure.label()),
                )
                .selected(selected)
                .element()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.edit_network(
                        |s| {
                            if let Some(position) = s.measures.iter().position(|m| *m == measure) {
                                s.measures.remove(position);
                            } else {
                                s.measures.push(measure);
                            }
                        },
                        cx,
                    )
                })),
            );
        }
        for (index, measure) in settings.measures.iter().enumerate() {
            panel = panel.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(format!("{}. {}", index + 1, measure.label()))
                    .child(
                        Button::new(("network-up", index), "Monter")
                            .disabled(index == 0)
                            .element()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.edit_network(
                                    |s| {
                                        if index > 0 && index < s.measures.len() {
                                            s.measures.swap(index, index - 1);
                                        }
                                    },
                                    cx,
                                );
                            })),
                    )
                    .child(
                        Button::new(("network-down", index), "Descendre")
                            .disabled(index + 1 == settings.measures.len())
                            .element()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.edit_network(
                                    |s| {
                                        if index + 1 < s.measures.len() {
                                            s.measures.swap(index, index + 1);
                                        }
                                    },
                                    cx,
                                );
                            })),
                    ),
            );
        }
        panel = panel.when(settings.grouped, |d| {
            d.child(self.network_presentation(5, cx))
        });
        #[cfg(debug_assertions)]
        {
            panel = panel.child(div().text_lg().child("Debug · Fenêtres réseau (secondes)"));
            for (index, (label, value, max)) in [
                ("Ping et jitter", settings.windows.latency_secs, 900),
                ("Débits", settings.windows.traffic_secs, 900),
                (
                    "Retransmissions",
                    settings.windows.retransmissions_secs,
                    3600,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                panel = panel.child(
                    div()
                        .flex()
                        .gap_2()
                        .items_center()
                        .child(format!("{label} : {value} s"))
                        .children([-1i64, 1].into_iter().map(|delta| {
                            Button::new(
                                ("network-window", index * 2 + usize::from(delta > 0)),
                                if delta > 0 { "+ 1 s" } else { "− 1 s" },
                            )
                            .disabled(if delta > 0 { value >= max } else { value <= 1 })
                            .element()
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    this.edit_network(
                                        |s| {
                                            let value = match index {
                                                0 => &mut s.windows.latency_secs,
                                                1 => &mut s.windows.traffic_secs,
                                                _ => &mut s.windows.retransmissions_secs,
                                            };
                                            *value =
                                                value.saturating_add_signed(delta).clamp(1, max);
                                        },
                                        cx,
                                    )
                                },
                            ))
                        })),
                );
            }
            panel = panel.child(Button::new("network-window-defaults", "Rétablir les fenêtres par défaut").element()
                .on_click(cx.listener(|this, _, _, cx| this.edit_network(|s| s.windows = WindowSettings::default(), cx))))
                .child(div().text_sm().child("Réglages de développement sauvegardés. Changer une fenêtre réinitialise uniquement les calculs concernés."));
        }
        for frame in &self.network_frames {
            panel = panel.child(
                div()
                    .p_3()
                    .rounded_lg()
                    .bg(rgb(theme::PANEL))
                    .child(frame.text.clone()),
            );
        }
        panel
    }
    fn network_presentation(
        &self,
        index: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let config = self.settings.network.presentation[index];
        let mut controls = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_sm().child(format!(
                "Position : {}, {} px · Zoom : {:.0} % · Opacité : {:.0} % · Fond : {:.0} %",
                config.x,
                config.y,
                config.zoom * 100.,
                config.opacity * 100.,
                config.background_opacity * 100.
            )));
        for (row, labels) in [
            ["← 20 px", "20 px →", "↑ 20 px", "20 px ↓"],
            ["Zoom −", "Zoom +", "Opacité −", "Opacité +"],
            [
                "Fond −",
                "Fond +",
                "Sens de croissance",
                "Visibilité en capture",
            ],
        ]
        .into_iter()
        .enumerate()
        {
            controls = controls.child(div().flex().flex_wrap().gap_2().children(
                labels.into_iter().enumerate().map(|(column, label)| {
                    Button::new(("network-layout", index * 12 + row * 4 + column), label)
                        .element()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.edit_network(
                                |s| {
                                    let c = &mut s.presentation[index];
                                    match row * 4 + column {
                                        0 => c.x = c.x.saturating_sub(20),
                                        1 => c.x = c.x.saturating_add(20),
                                        2 => c.y = c.y.saturating_sub(20),
                                        3 => c.y = c.y.saturating_add(20),
                                        4 => c.zoom = (c.zoom - 0.25).max(0.25),
                                        5 => c.zoom = (c.zoom + 0.25).min(4.),
                                        6 => c.opacity = (c.opacity - 0.1).max(0.),
                                        7 => c.opacity = (c.opacity + 0.1).min(1.),
                                        8 => {
                                            c.background_opacity =
                                                (c.background_opacity - 0.1).max(0.)
                                        }
                                        9 => {
                                            c.background_opacity =
                                                (c.background_opacity + 0.1).min(1.)
                                        }
                                        10 => {
                                            c.growth = if c.growth == Growth::TopDown {
                                                Growth::BottomUp
                                            } else {
                                                Growth::TopDown
                                            }
                                        }
                                        _ => {
                                            c.privacy = if c.privacy == Privacy::Public {
                                                Privacy::ExcludeFromCapture
                                            } else {
                                                Privacy::Public
                                            }
                                        }
                                    }
                                },
                                cx,
                            );
                        }))
                }),
            ));
        }
        controls.child(div().text_sm().child(format!(
            "Croissance : {} · Capture : {}",
            if config.growth == Growth::TopDown {
                "vers le bas"
            } else {
                "vers le haut"
            },
            if config.privacy == Privacy::Public {
                "visible"
            } else {
                "exclu si la protection est validée"
            }
        )))
    }
}
