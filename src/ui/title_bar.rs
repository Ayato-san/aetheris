use aetheris::game_process::GameStatus;
use gpui::{Context, MouseButton, Window, WindowControlArea, div, prelude::*, px, rgb};

use super::app::AetherisApp;

use super::{
    components::icons::{CloseIcon, MaximizeIcon, MinimizeIcon, RestoreIcon},
    theme::{PANEL, PANEL_MUTED, TEXT, TEXT_MUTED},
};

const WINDOW_ICON: u32 = 0xffffff;

pub(crate) fn render(
    status: GameStatus,
    window: &Window,
    cx: &mut Context<AetherisApp>,
) -> impl IntoElement {
    let maximize_icon = if window.is_maximized() {
        RestoreIcon::new(px(14.0), px(14.0), rgb(WINDOW_ICON)).into_any_element()
    } else {
        MaximizeIcon::new(px(14.0), px(14.0), rgb(WINDOW_ICON)).into_any_element()
    };

    div()
        .flex()
        .h(px(31.0))
        .bg(rgb(PANEL))
        .border_b_1()
        .border_color(rgb(PANEL_MUTED))
        .child(
            div()
                .id("window-drag-region")
                .flex()
                .items_center()
                .flex_1()
                .h_full()
                .px_4()
                .text_sm()
                .text_color(rgb(TEXT_MUTED))
                .window_control_area(WindowControlArea::Drag)
                .when(!cfg!(target_os = "windows"), |element| {
                    element.on_mouse_down(MouseButton::Left, |_, window, _| {
                        window.start_window_move();
                    })
                })
                .child("ÆTHERIS"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .px_4()
                .text_xs()
                .text_color(rgb(match status {
                    GameStatus::Stopped => super::theme::SUCCESS,
                    GameStatus::Running => super::theme::ACCENT,
                    GameStatus::Unknown => super::theme::DANGER,
                }))
                .child(status.label()),
        )
        .child(window_button(
            "window-minimize",
            MinimizeIcon::new(px(14.0), px(14.0), rgb(WINDOW_ICON)).into_any_element(),
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
                .child(CloseIcon::new(px(14.), px(14.), rgb(WINDOW_ICON))),
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
