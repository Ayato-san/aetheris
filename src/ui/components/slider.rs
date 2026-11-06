use super::state::{SliderState, Validation};
use crate::ui::theme::{self, tokens as t};
use gpui::{
    App, Bounds, Context, EventEmitter, FocusHandle, Pixels, Render, SharedString, Window, canvas,
    div, prelude::*, px, relative, rgb,
};
type SliderAcceptance = dyn Fn(f64, &mut App) -> Validation<()>;

pub(crate) struct SliderChanged(pub f64);
pub(crate) struct InteractionEnded;
pub(crate) struct Slider {
    state: SliderState,
    label: SharedString,
    disabled: bool,
    dragging: bool,
    bounds: Option<Bounds<Pixels>>,
    focus: FocusHandle,
    blur: Option<gpui::Subscription>,
    on_change_requested: Box<SliderAcceptance>,
}
impl EventEmitter<SliderChanged> for Slider {}
impl EventEmitter<InteractionEnded> for Slider {}
impl Slider {
    pub fn new(
        label: impl Into<SharedString>,
        state: SliderState,
        cx: &mut Context<Self>,
    ) -> Validation<Self> {
        state.validate()?;
        Ok(Self {
            state,
            label: label.into(),
            disabled: false,
            dragging: false,
            bounds: None,
            focus: cx.focus_handle().tab_stop(true),
            blur: None,
            on_change_requested: Box::new(|_, _| Ok(())),
        })
    }
    pub fn configure(&mut self, state: SliderState, cx: &mut Context<Self>) -> Validation<()> {
        self.state.configure(state)?;
        cx.notify();
        Ok(())
    }
    pub fn on_change_requested(
        &mut self,
        callback: impl Fn(f64, &mut App) -> Validation<()> + 'static,
    ) {
        self.on_change_requested = Box::new(callback);
    }
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        self.focus = self.focus.clone().tab_stop(!disabled);
        if disabled {
            self.dragging = false;
        }
        cx.notify();
    }
    fn propose(&mut self, value: f64, cx: &mut Context<Self>) {
        if self.disabled || value == self.state.value {
            return;
        }
        if (self.on_change_requested)(value, cx).is_ok() {
            self.state.value = value;
            cx.emit(SliderChanged(value));
            cx.notify();
        }
    }
    fn pointer(&mut self, x: Pixels, cx: &mut Context<Self>) {
        if let Some(bounds) = self.bounds {
            let ratio = f64::from(
                (x - bounds.left() - px(theme::CONTROL_HANDLE / 2.))
                    / (bounds.size.width - px(theme::CONTROL_HANDLE)),
            )
            .clamp(0., 1.);
            self.propose(
                self.state
                    .nearest(self.state.min + ratio * (self.state.max - self.state.min)),
                cx,
            );
        }
    }
}
impl Render for Slider {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.blur.is_none() {
            self.blur = Some(cx.on_blur(&self.focus, window, |this, _, cx| {
                this.dragging = false;
                cx.notify();
            }));
        }
        let owner = cx.entity();
        let pointer = owner.downgrade();
        let ratio =
            ((self.state.value - self.state.min) / (self.state.max - self.state.min)) as f32;
        div()
            .id("slider")
            .group("slider")
            .track_focus(&self.focus)
            .tab_stop(!self.disabled)
            .role(gpui::Role::Slider)
            .aria_label(self.label.clone())
            .aria_numeric_value(self.state.value)
            .aria_min_numeric_value(self.state.min)
            .aria_max_numeric_value(self.state.max)
            .flex()
            .items_center()
            .gap(px(theme::SPACE * 2.))
            .font_family(theme::FONT_FAMILY)
            .text_size(px(theme::FONT_SIZE))
            .text_color(rgb(theme::TEXT))
            .rounded(px(theme::RADIUS))
            .focus_visible(|s| s.border_color(rgb(theme::ACCENT)))
            .when(self.disabled, |d| {
                d.opacity(0.45).aria_description("Indisponible")
            })
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                let value = match event.keystroke.key.as_str() {
                    "left" | "down" => this.state.arrow(false),
                    "right" | "up" => this.state.arrow(true),
                    "home" => this.state.min,
                    "end" => this.state.max,
                    _ => return,
                };
                this.propose(value, cx);
                if !this.disabled {
                    cx.emit(InteractionEnded);
                }
                cx.stop_propagation();
            }))
            .child(self.label.clone())
            .child(
                div()
                    .id("slider-track")
                    .min_w(px(theme::CONTROL_HANDLE * 2.))
                    .relative()
                    .flex_1()
                    .h(px(theme::SLIDER_HEIGHT))
                    .child(
                        canvas(
                            move |bounds, _, cx| {
                                owner.update(cx, |this, _| this.bounds = Some(bounds))
                            },
                            move |_, _, window, _| {
                                window.on_mouse_event(
                                    move |event: &gpui::MouseMoveEvent, phase, _, cx| {
                                        if phase.bubble() {
                                            pointer
                                                .update(cx, |this, cx| {
                                                    if this.dragging {
                                                        this.pointer(event.position.x, cx);
                                                    }
                                                })
                                                .ok();
                                        }
                                    },
                                );
                            },
                        )
                        .absolute()
                        .size_full(),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px((theme::SLIDER_HEIGHT - theme::SLIDER_TRACK) / 2.))
                            .h(px(theme::SLIDER_TRACK))
                            .w_full()
                            .rounded_full()
                            .bg(rgb(t::TOKEN_COMPONENTS_SLIDER_BACKGROUND)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px((theme::SLIDER_HEIGHT - theme::SLIDER_TRACK) / 2.))
                            .h(px(theme::SLIDER_TRACK))
                            .w(relative(ratio))
                            .rounded_full()
                            .bg(rgb(t::TOKEN_COMPONENTS_SLIDER_PROGRESS))
                            .when(!self.disabled, |d| {
                                d.group_hover("slider", |s| {
                                    s.bg(rgb(t::TOKEN_COMPONENTS_SLIDER_HOVER_PROGRESS))
                                })
                            }),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px((theme::SLIDER_HEIGHT - theme::CONTROL_HANDLE) / 2.))
                            .left(relative(ratio))
                            .ml(px(-theme::CONTROL_HANDLE * ratio))
                            .size(px(theme::CONTROL_HANDLE))
                            .rounded_full()
                            .bg(rgb(t::TOKEN_COMPONENTS_SLIDER_PROGRESS))
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(!self.disabled, |d| {
                                d.group_hover("slider", |s| {
                                    s.bg(rgb(t::TOKEN_COMPONENTS_SLIDER_HOVER_PROGRESS))
                                })
                            })
                            .child(
                                div()
                                    .size(px(t::TOKEN_COMPONENTS_SLIDER_HANDLE_INNER))
                                    .rounded_full()
                                    .bg(rgb(t::TOKEN_COMPONENTS_SLIDER_HANDLE_COLOR))
                                    .when(!self.disabled, |d| {
                                        d.group_hover("slider", |s| {
                                            s.bg(rgb(t::TOKEN_COMPONENTS_SLIDER_HOVER_HANDLE))
                                        })
                                    }),
                            ),
                    )
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                            if !this.disabled {
                                window.focus(&this.focus, cx);
                                this.dragging = true;
                                this.pointer(event.position.x, cx);
                                cx.notify();
                            }
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                        if this.dragging {
                            this.pointer(event.position.x, cx);
                        }
                    }))
                    .on_mouse_up(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            if this.dragging {
                                this.dragging = false;
                                cx.emit(InteractionEnded);
                            }
                        }),
                    )
                    .on_mouse_up_out(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            if this.dragging {
                                this.dragging = false;
                                cx.emit(InteractionEnded);
                            }
                        }),
                    ),
            )
            .child(format!("{}", self.state.value))
    }
}

impl gpui::Focusable for Slider {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::*;
    use gpui::{Modifiers, MouseButton, TestAppContext, point};
    use std::{cell::Cell, rc::Rc};
    #[gpui::test]
    fn drag_continues_outside_track_and_ends_once(cx: &mut TestAppContext) {
        let (slider, cx) = cx.add_window_view(|_, cx| {
            Slider::new(
                "Volume",
                SliderState {
                    min: 0.,
                    max: 100.,
                    value: 0.,
                    steps: vec![0., 20., 100.],
                    snap: true,
                    keyboard_increment: 1.,
                },
                cx,
            )
            .unwrap()
        });
        let ends = Rc::new(Cell::new(0));
        let output = ends.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&slider, move |_, _: &InteractionEnded, _| {
                output.set(output.get() + 1)
            })
        });
        let bounds = cx.update(|_, cx| slider.read(cx).bounds.unwrap());
        let start = point(
            bounds.left() + px(theme::CONTROL_HANDLE / 2.),
            bounds.center().y,
        );
        let outside = point(bounds.right() + px(100.), bounds.center().y + px(100.));
        cx.simulate_mouse_move(start, None, Modifiers::default());
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(outside, Some(MouseButton::Left), Modifiers::default());
        cx.simulate_mouse_up(outside, MouseButton::Left, Modifiers::default());
        cx.update(|_, cx| assert_eq!(slider.read(cx).state.value, 100.));
        assert_eq!(ends.get(), 1);
        cx.update(|_, cx| {
            slider.update(cx, |slider, cx| {
                slider.on_change_requested(|_, _| Err("Refus".into()));
                slider.propose(20., cx);
                assert_eq!(slider.state.value, 100.);
                slider.set_disabled(true, cx);
                slider.propose(0., cx);
                assert_eq!(slider.state.value, 100.);
            })
        });
        assert_eq!(ends.get(), 1);
    }
}
