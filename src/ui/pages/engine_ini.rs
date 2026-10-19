use gpui::{Context, div, prelude::*, px, rgb};

use crate::ui::{
    app::AetherisApp,
    theme::{DANGER, PANEL, PANEL_MUTED, SUCCESS, TEXT_MUTED},
};

pub(crate) fn render(app: &AetherisApp, cx: &mut Context<AetherisApp>) -> impl IntoElement {
    let found = app.engine_ini_path().is_some();
    let status_label = if found {
        "Engine.ini · Trouvé"
    } else {
        "Engine.ini · Introuvable"
    };
    let status_color = if found { SUCCESS } else { DANGER };
    let status_pill = div()
        .id("engine-ini-status")
        .px_3()
        .py_2()
        .rounded_full()
        .border_1()
        .border_color(rgb(status_color))
        .text_xs()
        .text_color(rgb(status_color))
        .when(!found, |element| {
            element
                .cursor_pointer()
                .hover(|style| style.bg(rgb(0x351b29)))
                .on_click(cx.listener(|app, _, window, cx| {
                    app.select_engine_ini(window, cx);
                }))
        })
        .child(status_label);

    div()
        .flex()
        .flex_col()
        .flex_1()
        .p_8()
        .gap_6()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().text_3xl().child("Engine.ini"))
                        .child(
                            div()
                                .text_color(rgb(TEXT_MUTED))
                                .child("Configuration du moteur d’Aion 2"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_end()
                        .gap_2()
                        .when_some(app.engine_ini_error(), |element, error| {
                            element.child(
                                div()
                                    .max_w(px(360.0))
                                    .text_sm()
                                    .text_right()
                                    .text_color(rgb(DANGER))
                                    .child(error.to_owned()),
                            )
                        })
                        .child(status_pill),
                ),
        )
}
