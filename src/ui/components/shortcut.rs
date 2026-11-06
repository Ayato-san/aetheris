use super::{
    Button,
    state::{Invalid, Validation},
};
use crate::ui::theme::{self, tokens as t};
use gpui::{
    App, Context, EventEmitter, FocusHandle, Keystroke, Render, SharedString, Window, div,
    prelude::*, px, rgb,
};

type ShortcutAcceptance = dyn Fn(&Option<Shortcut>, &mut App) -> Validation<()>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Shortcut(pub Keystroke);
impl Shortcut {
    pub fn validate(&self) -> Validation<()> {
        let key = self.0.key.as_str();
        let function = key
            .strip_prefix('f')
            .and_then(|v| v.parse::<u8>().ok())
            .is_some_and(|n| (1..=12).contains(&n));
        let modifiers = self.0.modifiers;
        if key == "escape"
            || [
                "shift", "control", "ctrl", "alt", "super", "platform", "cmd", "meta", "fn",
                "function",
            ]
            .contains(&key)
            || !(function
                || modifiers.control
                || modifiers.alt
                || modifiers.platform
                || modifiers.shift
                || modifiers.function)
        {
            return Err(Invalid::from(
                "Une touche principale et un modificateur sont requis",
            ));
        }
        Ok(())
    }
}
pub(crate) struct ShortcutChanged(pub Option<Shortcut>);
pub(crate) struct ShortcutSelector {
    label: SharedString,
    value: Option<Shortcut>,
    capturing: bool,
    disabled: bool,
    focus: FocusHandle,
    on_change_requested: Box<ShortcutAcceptance>,
}
impl EventEmitter<ShortcutChanged> for ShortcutSelector {}
impl ShortcutSelector {
    pub fn new(label: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            label: label.into(),
            value: None,
            capturing: false,
            disabled: false,
            focus: cx.focus_handle().tab_stop(true),
            on_change_requested: Box::new(|_, _| Ok(())),
        }
    }
    pub fn set_value(&mut self, value: Option<Shortcut>, cx: &mut Context<Self>) -> Validation<()> {
        if let Some(value) = &value {
            value.validate()?;
        }
        self.value = value;
        cx.notify();
        Ok(())
    }
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        self.focus = self.focus.clone().tab_stop(!disabled);
        if disabled {
            self.capturing = false;
        }
        cx.notify();
    }
    pub fn on_change_requested(
        &mut self,
        callback: impl Fn(&Option<Shortcut>, &mut App) -> Validation<()> + 'static,
    ) {
        self.on_change_requested = Box::new(callback);
    }
    fn propose(&mut self, value: Option<Shortcut>, cx: &mut Context<Self>) {
        if self.value != value && (self.on_change_requested)(&value, cx).is_ok() {
            self.value = value.clone();
            cx.emit(ShortcutChanged(value));
        }
        self.capturing = false;
        cx.notify();
    }
}
impl Render for ShortcutSelector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.capturing && !self.focus.is_focused(window) {
            self.capturing = false;
        }
        let text = if self.capturing {
            "Saisissez un raccourci…".into()
        } else {
            self.value
                .as_ref()
                .map(|value| value.0.unparse())
                .unwrap_or_else(|| "Non défini".into())
        };
        div()
            .id("shortcut")
            .min_w(px(t::TOKEN_COMPONENTS_SHORTCUT_MIN_WIDTH))
            .flex()
            .items_center()
            .gap(px(t::TOKEN_COMPONENTS_SHORTCUT_GAP))
            .font_family(theme::FONT_FAMILY)
            .text_size(px(theme::FONT_SIZE))
            .text_color(rgb(theme::TEXT))
            .child(self.label.clone())
            .child(
                Button::new("shortcut-trigger", text)
                    .selected(self.capturing)
                    .disabled(self.disabled)
                    .element_with_hover(Some(t::TOKEN_COMPONENTS_FIELD_BACKGROUND), None)
                    .track_focus(&self.focus)
                    .min_w(px(t::TOKEN_COMPONENTS_SHORTCUT_FIELD_MIN_WIDTH))
                    .font_weight(gpui::FontWeight(t::TOKEN_FONT_BOLD))
                    .bg(rgb(t::TOKEN_COMPONENTS_FIELD_BACKGROUND))
                    .border(px(if self.capturing {
                        t::TOKEN_COMPONENTS_COMMON_FOCUS_WIDTH
                    } else {
                        theme::BORDER_WIDTH
                    }))
                    .border_color(rgb(if self.capturing {
                        t::TOKEN_COMPONENTS_COMMON_FOCUS_COLOR
                    } else {
                        t::TOKEN_COMPONENTS_FIELD_BORDER
                    }))
                    .text_color(rgb(if self.capturing {
                        theme::ACCENT
                    } else if self.value.is_none() {
                        t::TOKEN_COMPONENTS_SHORTCUT_PLACEHOLDER
                    } else {
                        t::TOKEN_COMPONENTS_SHORTCUT_TEXT
                    }))
                    .capture_key_down(cx.listener(
                        |this, event: &gpui::KeyDownEvent, window, cx| {
                            if this.disabled {
                                return;
                            }
                            if this.capturing {
                                cx.stop_propagation();
                                window.prevent_default();
                                if event.is_held {
                                    return;
                                }
                                if event.keystroke.key == "escape" {
                                    this.propose(None, cx);
                                } else {
                                    let candidate = Shortcut(event.keystroke.clone());
                                    if candidate.validate().is_ok() {
                                        this.propose(Some(candidate), cx);
                                    }
                                }
                            } else if ["enter", "space"].contains(&event.keystroke.key.as_str()) {
                                this.capturing = true;
                                cx.notify();
                                cx.stop_propagation();
                                window.prevent_default();
                            }
                        },
                    ))
                    .on_click(cx.listener(|this, _, window, cx| {
                        if !this.disabled {
                            window.focus(&this.focus, cx);
                            this.capturing = true;
                            cx.notify();
                        }
                    }))
                    .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                        this.capturing = false;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("clear-shortcut", "×")
                    .disabled(self.disabled)
                    .element()
                    .aria_label("Supprimer le raccourci")
                    .on_click(cx.listener(|this, _, _, cx| {
                        if !this.disabled {
                            this.propose(None, cx);
                        }
                    })),
            )
    }
}
impl gpui::Focusable for ShortcutSelector {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::*;
    use gpui::TestAppContext;
    #[gpui::test]
    fn capture_ignores_opening_key_and_escape_removes_shortcut(cx: &mut TestAppContext) {
        let (selector, cx) = cx.add_window_view(|window, cx| {
            let control = ShortcutSelector::new("Raccourci", cx);
            window.focus(&control.focus, cx);
            control
        });
        cx.simulate_keystrokes("enter");
        cx.update(|_, cx| {
            let control = selector.read(cx);
            assert!(control.capturing);
            assert!(control.value.is_none());
        });
        cx.simulate_keystrokes("ctrl-k");
        cx.update(|_, cx| {
            let control = selector.read(cx);
            assert!(!control.capturing);
            assert_eq!(control.value.as_ref().unwrap().0.key, "k");
        });
        cx.simulate_keystrokes("space escape");
        cx.update(|_, cx| assert!(selector.read(cx).value.is_none()));
    }
}
