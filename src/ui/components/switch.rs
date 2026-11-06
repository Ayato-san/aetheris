use super::state::Validation;
use crate::ui::theme::{self, tokens as t};
use gpui::{
    App, Context, EventEmitter, FocusHandle, Render, SharedString, Window, div, prelude::*, px, rgb,
};
type SwitchAcceptance = dyn Fn(bool, &mut App) -> Validation<()>;

#[derive(Clone, Copy)]
pub(crate) enum SwitchVariant {
    Standard,
    Card,
}
pub(crate) struct SwitchChanged(pub bool);
pub(crate) struct Switch {
    value: bool,
    label: SharedString,
    disabled: bool,
    variant: SwitchVariant,
    focus: FocusHandle,
    on_change_requested: Box<SwitchAcceptance>,
}
impl EventEmitter<SwitchChanged> for Switch {}
impl Switch {
    pub fn new(
        label: impl Into<SharedString>,
        value: bool,
        variant: SwitchVariant,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            value,
            label: label.into(),
            disabled: false,
            variant,
            focus: cx.focus_handle().tab_stop(true),
            on_change_requested: Box::new(|_, _| Ok(())),
        }
    }
    pub fn on_change_requested(
        &mut self,
        callback: impl Fn(bool, &mut App) -> Validation<()> + 'static,
    ) {
        self.on_change_requested = Box::new(callback);
    }
    pub fn set_value(&mut self, value: bool, cx: &mut Context<Self>) {
        self.value = value;
        cx.notify();
    }
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        self.focus = self.focus.clone().tab_stop(!disabled);
        cx.notify();
    }
    fn toggle(&mut self, cx: &mut Context<Self>) {
        if !self.disabled && (self.on_change_requested)(!self.value, cx).is_ok() {
            self.value = !self.value;
            cx.emit(SwitchChanged(self.value));
            cx.notify();
        }
    }
}
impl Render for Switch {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("switch")
            .group("switch")
            .track_focus(&self.focus)
            .tab_stop(!self.disabled)
            .role(gpui::Role::Switch)
            .aria_label(self.label.clone())
            .aria_toggled(if self.value {
                gpui::Toggled::True
            } else {
                gpui::Toggled::False
            })
            .flex()
            .items_center()
            .gap(px(t::TOKEN_COMPONENTS_SWITCH_GAP))
            .font_family(theme::FONT_FAMILY)
            .text_size(px(theme::FONT_SIZE))
            .line_height(px(t::TOKEN_COMPONENTS_COMMON_LINE_HEIGHT))
            .text_color(rgb(theme::TEXT))
            .rounded(px(theme::RADIUS))
            .focus_visible(|s| s.border_color(rgb(theme::ACCENT)).border_2())
            .when(matches!(self.variant, SwitchVariant::Card), |d| {
                d.p(px(t::TOKEN_COMPONENTS_CARD_PADDING))
                    .border_1()
                    .bg(rgb(theme::PANEL))
                    .border_color(rgb(if self.value {
                        t::TOKEN_COMPONENTS_SWITCH_ACTIVE_BACKGROUND
                    } else {
                        t::TOKEN_COMPONENTS_SWITCH_BACKGROUND
                    }))
            })
            .when(!self.disabled, |d| d.cursor_pointer())
            .when(self.disabled, |d| {
                d.opacity(0.45).aria_description("Indisponible")
            })
            .on_click(cx.listener(|this, _, _, cx| this.toggle(cx)))
            .on_key_down(cx.listener(|this, e: &gpui::KeyDownEvent, window, cx| {
                if e.keystroke.key == "space" {
                    this.toggle(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                }
            }))
            .child(
                div()
                    .w(px(theme::SWITCH_WIDTH))
                    .h(px(theme::SWITCH_HEIGHT))
                    .p(px(theme::SWITCH_PADDING))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .when(self.value, |d| d.justify_end())
                    .bg(rgb(if self.value {
                        t::TOKEN_COMPONENTS_SWITCH_ACTIVE_BACKGROUND
                    } else {
                        t::TOKEN_COMPONENTS_SWITCH_BACKGROUND
                    }))
                    .when(!self.disabled, |d| {
                        d.hover(|s| {
                            s.bg(rgb(if self.value {
                                t::TOKEN_COMPONENTS_SWITCH_ACTIVE_HOVER_BACKGROUND
                            } else {
                                t::TOKEN_COMPONENTS_SWITCH_HOVER_BACKGROUND
                            }))
                        })
                    })
                    .child(
                        div()
                            .size(px(t::TOKEN_COMPONENTS_SWITCH_HANDLE))
                            .rounded_full()
                            .bg(rgb(if self.value {
                                t::TOKEN_COMPONENTS_SWITCH_HANDLE_ON
                            } else {
                                t::TOKEN_COMPONENTS_SWITCH_HANDLE_OFF
                            }))
                            .when(!self.disabled, |d| {
                                d.group_hover("switch", |s| {
                                    s.bg(rgb(if self.value {
                                        t::TOKEN_COMPONENTS_SWITCH_HANDLE_ON_HOVER
                                    } else {
                                        t::TOKEN_COMPONENTS_SWITCH_HANDLE_OFF_HOVER
                                    }))
                                })
                            }),
                    ),
            )
            .child(self.label.clone())
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::*;
    use gpui::TestAppContext;
    use std::{cell::Cell, rc::Rc};
    #[gpui::test]
    fn space_toggles_once_and_programmatic_updates_are_silent(cx: &mut TestAppContext) {
        let (switch, cx) = cx.add_window_view(|window, cx| {
            let control = Switch::new("Activer", false, SwitchVariant::Standard, cx);
            window.focus(&control.focus, cx);
            control
        });
        let count = Rc::new(Cell::new(0));
        let output = count.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&switch, move |_, _: &SwitchChanged, _| {
                output.set(output.get() + 1)
            })
        });
        cx.simulate_keystrokes("space");
        assert_eq!(count.get(), 1);
        cx.update(|_, cx| {
            switch.update(cx, |control, cx| {
                assert!(control.value);
                control.set_disabled(true, cx);
                control.set_value(false, cx);
            })
        });
        cx.simulate_keystrokes("space");
        assert_eq!(count.get(), 1);
    }
}

impl gpui::Focusable for Switch {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
