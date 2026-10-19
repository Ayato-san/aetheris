use gpui::{MouseButton, Window, div, prelude::*, px, rgb};

use super::theme::{PANEL, PANEL_MUTED, TEXT, TEXT_MUTED};

pub(crate) fn render() -> impl IntoElement {
    div()
        .flex()
        .h(px(42.0))
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
                .on_mouse_down(MouseButton::Left, |_, window, _| {
                    window.start_window_move();
                })
                .child("ÆTHERIS"),
        )
        .child(window_button("window-minimize", "—", false, |window| {
            window.minimize_window();
        }))
        .child(window_button("window-maximize", "□", false, |window| {
            window.zoom_window();
        }))
        .child(window_button("window-close", "×", true, |window| {
            window.remove_window();
        }))
}

fn window_button(
    id: &'static str,
    label: &'static str,
    danger: bool,
    action: impl Fn(&mut Window) + 'static,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .w(px(46.0))
        .h_full()
        .text_color(rgb(TEXT_MUTED))
        .when(danger, |element| {
            element.hover(|style| style.bg(rgb(0xc42b3a)).text_color(rgb(0xffffff)))
        })
        .when(!danger, |element| {
            element.hover(|style| style.bg(rgb(PANEL_MUTED)).text_color(rgb(TEXT)))
        })
        .on_click(move |_, window, _| action(window))
        .child(label)
}
