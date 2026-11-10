use aetheris::game_process::GameStatus;
use gpui::{Context, FontWeight, MouseButton, Window, WindowControlArea, div, prelude::*, px, rgb};

use super::app::AetherisApp;

use super::{
    components::icons::{CrossIcon, SquareIcon, WindowMinimizeIcon, WindowRestoreIcon},
    theme::{BORDER, PANEL, PANEL_MUTED, TEXT, TEXT_MUTED, tokens},
};

const WINDOW_ICON: u32 = 0xffffff;

fn app_icon() -> std::sync::Arc<gpui::Image> {
    static ICON: std::sync::OnceLock<std::sync::Arc<gpui::Image>> = std::sync::OnceLock::new();
    ICON.get_or_init(|| {
        std::sync::Arc::new(gpui::Image::from_bytes(
            gpui::ImageFormat::Png,
            super::APP_ICON_PNG.to_vec(),
        ))
    })
    .clone()
}

pub(crate) fn render(
    status: GameStatus,
    window: &Window,
    cx: &mut Context<AetherisApp>,
) -> impl IntoElement {
    let maximize_icon = if window.is_maximized() {
        WindowRestoreIcon::new(px(14.0), px(14.0), rgb(WINDOW_ICON)).into_any_element()
    } else {
        SquareIcon::new(px(14.0), px(14.0), rgb(WINDOW_ICON)).into_any_element()
    };

    let status_color = match status {
        GameStatus::Stopped => BORDER,
        GameStatus::Running => tokens::TOKEN_ACCENT_DARK,
        GameStatus::Unknown => super::theme::DANGER,
    };
    let status_label = match status {
        GameStatus::Stopped => "Aion 2 Arrêté",
        GameStatus::Running => "Aion 2 Lancé",
        GameStatus::Unknown => "Aion 2 Statut inconnu",
    };

    div()
        .flex()
        .items_center()
        .flex_shrink_0()
        .h(px(31.0))
        .bg(rgb(PANEL))
        .border_b_1()
        .border_color(rgb(tokens::TOKEN_GRAY_800))
        .child(
            div()
                .id("window-drag-region")
                .flex()
                .items_center()
                .flex_1()
                .h_full()
                .px(px(8.0))
                .gap(px(4.0))
                .text_size(px(12.0))
                .line_height(px(20.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(TEXT_MUTED))
                .window_control_area(WindowControlArea::Drag)
                .when(!cfg!(target_os = "windows"), |element| {
                    element.on_mouse_down(MouseButton::Left, |_, window, _| {
                        window.start_window_move();
                    })
                })
                .child(gpui::img(app_icon()).w(px(20.0)).h(px(20.0)))
                .child("ÆTHERIS"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .flex_shrink_0()
                .gap(px(4.0))
                .px(px(4.0))
                .py(px(2.0))
                .mr(px(8.0))
                .min_w(px(64.0))
                .rounded_full()
                .bg(rgb(PANEL_MUTED))
                .text_size(px(7.0))
                .line_height(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(status_color))
                .child(div().size(px(4.0)).rounded_full().bg(rgb(status_color)))
                .child(status_label),
        )
        .child(window_button(
            "window-minimize",
            WindowMinimizeIcon::new(px(14.0), px(14.0), rgb(WINDOW_ICON)).into_any_element(),
            false,
            WindowControlArea::Min,
            |window| {
                window.minimize_window();
            },
        ))
        .child(window_button(
            "window-maximize",
            maximize_icon,
            false,
            WindowControlArea::Max,
            |window| {
                window.zoom_window();
            },
        ))
        .child(
            div()
                .id("window-close")
                .flex()
                .items_center()
                .justify_center()
                .w(px(46.))
                .flex_shrink_0()
                .h_full()
                .occlude()
                .cursor_pointer()
                .hover(|style| style.bg(rgb(0xc42b3a)))
                .when(cfg!(target_os = "windows"), |element| {
                    element.window_control_area(WindowControlArea::Close)
                })
                .when(!cfg!(target_os = "windows"), |element| {
                    element.on_click(cx.listener(|app, _, window, cx| {
                        if app.prepare_close(cx) {
                            window.remove_window();
                        }
                    }))
                })
                .child(CrossIcon::new(px(14.), px(14.), rgb(WINDOW_ICON))),
        )
}

fn window_button(
    id: &'static str,
    icon: gpui::AnyElement,
    danger: bool,
    control_area: WindowControlArea,
    action: impl Fn(&mut Window) + 'static,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .w(px(46.0))
        .flex_shrink_0()
        .h_full()
        .occlude()
        .text_color(rgb(TEXT_MUTED))
        .when(danger, |element| {
            element.hover(|style| style.bg(rgb(0xc42b3a)).text_color(rgb(0xffffff)))
        })
        .when(!danger, |element| {
            element.hover(|style| style.bg(rgb(PANEL_MUTED)).text_color(rgb(TEXT)))
        })
        .when(cfg!(target_os = "windows"), |element| {
            element.window_control_area(control_area)
        })
        .when(!cfg!(target_os = "windows"), |element| {
            element.on_click(move |_, window, _| action(window))
        })
        .child(icon)
}
