use gpui::{div, prelude::*, rgb};

use crate::ui::theme::{ACCENT, PANEL, PANEL_MUTED, TEXT_MUTED};

pub(crate) fn render() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .p_8()
        .gap_6()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_3xl().child("Aetheris"))
                .child(
                    div()
                        .text_color(rgb(TEXT_MUTED))
                        .child("Votre compagnon de jeu, au même endroit."),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_6()
                .rounded_xl()
                .bg(rgb(PANEL))
                .border_1()
                .border_color(rgb(PANEL_MUTED))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(ACCENT))
                        .child("PREMIER MODULE"),
                )
                .child(div().text_2xl().child("Configuration Engine.ini"))
                .child(
                    div()
                        .text_color(rgb(TEXT_MUTED))
                        .child("Chargez, modifiez et sauvegardez les paramètres du jeu sans éditer le fichier à la main."),
                ),
        )
        .child(
            div()
                .flex()
                .gap_4()
                .child(feature_card("Historique", "Donjons réalisés"))
                .child(feature_card("Combat meter", "Analyse en direct"))
                .child(feature_card("Carte", "Familiers et ressources"))
                .child(feature_card("Timers", "Événements à suivre")),
        )
}

fn feature_card(title: &'static str, subtitle: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_1()
        .p_4()
        .rounded_lg()
        .bg(rgb(PANEL))
        .border_1()
        .border_color(rgb(PANEL_MUTED))
        .child(title)
        .child(div().text_sm().text_color(rgb(TEXT_MUTED)).child(subtitle))
}
