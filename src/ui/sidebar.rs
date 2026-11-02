use gpui::{Context, div, prelude::*, px, rgb};

use super::{
    app::{AetherisApp, Page},
    theme::{ACCENT, PANEL, PANEL_MUTED, TEXT_MUTED},
};

pub(crate) fn render(current_page: Page, cx: &mut Context<AetherisApp>) -> impl IntoElement {
    let sidebar = div()
        .flex()
        .flex_col()
        .w(px(220.0))
        .h_full()
        .p_5()
        .gap_3()
        .bg(rgb(PANEL))
        .border_r_1()
        .border_color(rgb(PANEL_MUTED))
        .child(
            nav_item("nav-home", "Accueil", current_page == Page::Home).on_click(cx.listener(
                |app, _, _, cx| {
                    app.navigate(Page::Home, cx);
                },
            )),
        )
        .child(
            nav_item(
                "nav-engine-ini",
                "Engine.ini",
                current_page == Page::EngineIni,
            )
            .on_click(cx.listener(|app, _, _, cx| {
                app.navigate(Page::EngineIni, cx);
            })),
        );
    #[cfg(debug_assertions)]
    let sidebar = sidebar.child(
        nav_item(
            "nav-combat-debug",
            "Combat · Debug",
            current_page == Page::CombatDebug,
        )
        .on_click(cx.listener(|app, _, _, cx| app.navigate(Page::CombatDebug, cx))),
    );
    sidebar
}

fn nav_item(id: &'static str, label: &'static str, active: bool) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .px_3()
        .py_2()
        .rounded_md()
        .cursor_pointer()
        .when(active, |element| {
            element.bg(rgb(PANEL_MUTED)).text_color(rgb(ACCENT))
        })
        .when(!active, |element| {
            element
                .text_color(rgb(TEXT_MUTED))
                .hover(|style| style.bg(rgb(PANEL_MUTED)))
        })
        .child(label)
}
