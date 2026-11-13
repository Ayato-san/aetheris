use super::components::NavigationButton;
use aetheris::companion::policy::Channel;
use gpui::{Context, FontWeight, div, prelude::*, px, rgb};

use super::{
    app::{AetherisApp, Page},
    components::icons::{CodeIcon, HouseIcon, SettingsIcon},
    theme::{ACCENT, BORDER, PANEL, PANEL_MUTED, TEXT, TEXT_MUTED},
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
        .w(px(172.0))
        .flex_shrink_0()
        .h_full()
        .px(px(8.0))
        .pt(px(16.0))
        .pb(px(8.0))
        .gap(px(4.0))
        .bg(rgb(PANEL))
        .child(
            nav_item("nav-home", "Accueil", current_page == Page::Home).on_click(cx.listener(
                |app, _, _, cx| {
                    app.navigate(Page::Home, cx);
                },
            )),
        )
        .child(section_label("ENREGISTREMENT"))
        .child(
            nav_item(
                "nav-combat-history",
                "Historique",
                current_page == Page::CombatHistory,
            )
            .on_click(cx.listener(|app, _, _, cx| app.navigate(Page::CombatHistory, cx))),
        )
        .child(section_label("OPTIMISATION"))
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
    let sidebar = sidebar.child(section_label("DEBUG")).child(
        nav_item(
            "nav-combat-debug",
            "Combat Debug",
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
                                    .child(if aetheris::companion::managed::current_is_managed() {
                                        "Canal commun à tous les utilisateurs"
                                    } else {
                                        "Canal de mise à jour"
                                    }),
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
                    .text_size(px(12.0))
                    .line_height(px(20.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(BORDER))
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
    let icon = match id {
        "nav-home" => HouseIcon::new(px(20.0), px(20.0), rgb(TEXT)).into_any_element(),
        "nav-engine-ini" => SettingsIcon::new(px(20.0), px(20.0), rgb(TEXT)).into_any_element(),
        _ => CodeIcon::new(px(20.0), px(20.0), rgb(TEXT)).into_any_element(),
    };
    NavigationButton::new(id, label)
        .icon(icon)
        .active(active)
        .element()
        .w_full()
        .min_w(px(156.0))
}

fn section_label(label: &'static str) -> impl IntoElement {
    div()
        .flex_shrink_0()
        .mt(px(8.0))
        .ml(px(8.0))
        .text_size(px(12.0))
        .line_height(px(20.0))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(BORDER))
        .child(label)
}
