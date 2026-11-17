//! Compiled only in development builds. These windows govern network widgets,
//! not cube decoding, packet acquisition, Combat Debug or combat history.
use super::OverlayEditor;
use crate::ui::components::Button;
use aetheris::overlay::network::WindowSettings;
use gpui::{Context, div, prelude::*};

impl OverlayEditor {
    pub(super) fn network_debug_controls(
        &self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let settings = self.settings.network.windows;
        let mut panel = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_lg().child("Debug · Fenêtres réseau (secondes)"));
        for (index, (label, value, max)) in [
            ("Courte · ping, jitter et débits", settings.short_secs, 900),
            ("Retransmissions TCP", settings.retransmissions_secs, 3600),
        ]
        .into_iter()
        .enumerate()
        {
            panel = panel.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(format!("{label} : {value} s (1–{max} s)"))
                    .child(div().flex().flex_wrap().gap_2().children(
                        [-60i64, -10, -1, 1, 10, 60].into_iter().enumerate().map(
                            |(step, delta)| {
                                Button::new(
                                    ("network-window", index * 6 + step),
                                    format!("{delta:+} s"),
                                )
                                .disabled(if delta > 0 { value >= max } else { value <= 1 })
                                .element()
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.edit_network(
                                            |s| {
                                                let value = if index == 0 {
                                                    &mut s.windows.short_secs
                                                } else {
                                                    &mut s.windows.retransmissions_secs
                                                };
                                                *value = value
                                                    .saturating_add_signed(delta)
                                                    .clamp(1, max);
                                            },
                                            cx,
                                        );
                                    },
                                ))
                            },
                        ),
                    )),
            );
        }
        panel.child(Button::new("network-window-defaults", "Rétablir les fenêtres par défaut").element()
            .on_click(cx.listener(|this, _, _, cx| this.edit_network(|s| s.windows = WindowSettings::default(), cx))))
            .child(div().text_sm().child("Valeurs par défaut : courte 10 s, retransmissions 900 s (15 min). Réglages propres aux widgets réseau, sauvegardés entre les lancements de développement. Changer une fenêtre réinitialise les calculs concernés et les masque jusqu’aux nouvelles observations nécessaires."))
    }
}
