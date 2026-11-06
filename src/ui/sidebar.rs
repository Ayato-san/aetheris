use super::components::NavigationButton;
use aetheris::companion::policy::Channel;
use gpui::{Context, div, prelude::*, px, rgb};

use super::{
    app::{AetherisApp, Page},
    theme::{ACCENT, PANEL, PANEL_MUTED, TEXT_MUTED},
};

pub(crate) fn render(
    current_page: Page,
    channel: Channel,
    selector_open: bool,
    cx: &mut Context<AetherisApp>,
) -> impl IntoElement {
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
    #[cfg(debug_assertions)]
    let sidebar = sidebar.child(
        nav_item(
            "nav-gallery",
            "Design System",
            current_page == Page::Gallery,
        )
        .on_click(cx.listener(|app, _, _, cx| app.navigate(Page::Gallery, cx))),
    );
    sidebar.child(div().flex_1()).child(
        div()
            .id("version-channel-selector")
            .flex()
            .flex_col()
            .gap_2()
            .when(selector_open, |element| {
                element
                    .on_mouse_down_out(
                        cx.listener(|app, _, _, cx| app.dismiss_channel_selector(cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .p_2()
                            .rounded_md()
                            .bg(rgb(PANEL_MUTED))
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child("Canal de mise à jour"),
                            )
                            .child(channel_option(
                                "stable-channel",
                                "Stable",
                                Channel::Stable,
                                channel,
                                cx,
                            ))
                            .child(channel_option(
                                "prerelease-channel",
                                "Préversion",
                                Channel::Prerelease,
                                channel,
                                cx,
                            )),
                    )
            })
            .child(
                div()
                    .id("application-version")
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .text_sm()
                    .text_color(rgb(TEXT_MUTED))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(PANEL_MUTED)))
                    .child(format!(
                        "v{}",
                        env!("CARGO_PKG_VERSION").replace("-beta.", "-beta-")
                    ))
                    .on_click(cx.listener(|app, _, _, cx| app.toggle_channel_selector(cx))),
            ),
    )
}

fn channel_option(
    id: &'static str,
    label: &'static str,
    channel: Channel,
    selected: Channel,
    cx: &mut Context<AetherisApp>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .px_3()
        .py_2()
        .rounded_md()
        .text_sm()
        .cursor_pointer()
        .text_color(rgb(if selected == channel {
            ACCENT
        } else {
            TEXT_MUTED
        }))
        .hover(|style| style.bg(rgb(PANEL)))
        .child(if selected == channel {
            format!("✓ {label}")
        } else {
            label.to_owned()
        })
        .on_click(cx.listener(move |app, _, _, cx| app.select_update_channel(channel, cx)))
}

fn nav_item(id: &'static str, label: &'static str, active: bool) -> gpui::Stateful<gpui::Div> {
    NavigationButton::new(id, label).active(active).element()
}
