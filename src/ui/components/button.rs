use crate::ui::theme::controls::{self, button as tokens};
use crate::ui::theme::{self, ControlSize};
use gpui::{Div, ElementId, SharedString, Stateful, div, prelude::*, px, rgb};

#[derive(Clone, Copy, Default)]
pub(crate) enum ButtonVariant {
    Primary,
    #[default]
    Secondary,
}

/// Returns a normal GPUI interactive element: pages attach their own actions.
pub(crate) struct Button {
    id: ElementId,
    label: SharedString,
    variant: ButtonVariant,
    size: ControlSize,
    disabled: bool,
    selected: bool,
    icon: Option<gpui::AnyElement>,
    icon_only: bool,
}
impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::Secondary,
            size: ControlSize::Medium,
            disabled: false,
            selected: false,
            icon: None,
            icon_only: false,
        }
    }
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }
    /// `label` remains the accessible name of an icon-only button.
    pub fn icon_only(mut self) -> Self {
        self.icon_only = true;
        self
    }
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn element(self) -> Stateful<Div> {
        self.element_with_hover(None, None)
    }
    pub(super) fn element_with_hover(
        self,
        hover_override: Option<u32>,
        hover_text: Option<u32>,
    ) -> Stateful<Div> {
        let primary = matches!(self.variant, ButtonVariant::Primary);
        let height = match self.size {
            ControlSize::Small => tokens::SMALL_HEIGHT,
            ControlSize::Medium => tokens::HEIGHT,
            ControlSize::Large => tokens::LARGE_HEIGHT,
        };
        let border = if primary {
            tokens::PRIMARY_BORDER
        } else {
            tokens::SECONDARY_BORDER
        };
        let hover = hover_override.unwrap_or(if primary {
            tokens::PRIMARY_HOVER
        } else {
            tokens::SECONDARY_HOVER
        });
        div()
            .id(self.id)
            .role(gpui::Role::Button)
            .aria_label(self.label.clone())
            .focusable()
            .tab_stop(!self.disabled)
            .flex()
            .items_center()
            .justify_center()
            .gap(px(theme::SPACE * 2.))
            .h(px(height))
            .min_w(px(tokens::MIN_WIDTH))
            .flex_shrink_0()
            .px(px(tokens::PADDING_X))
            .rounded(px(tokens::RADIUS))
            .overflow_hidden()
            .whitespace_nowrap()
            .border(px(tokens::BORDER_WIDTH))
            .border_color(rgb(if self.selected { theme::ACCENT } else { border }))
            .bg(rgb(if primary {
                tokens::PRIMARY_BACKGROUND
            } else {
                tokens::SECONDARY_BACKGROUND
            }))
            .font_family(controls::FONT_FAMILY)
            .font_weight(tokens::WEIGHT)
            .text_size(px(controls::FONT_SIZE))
            .line_height(px(controls::LINE_HEIGHT))
            .text_color(rgb(tokens::TEXT))
            .focus_visible(|s| s.border_color(rgb(theme::TEXT)).border_2())
            .when(!self.disabled, |d| {
                d.cursor_pointer()
                    .hover(|s| {
                        let style = s.bg(rgb(hover));
                        if let Some(color) = hover_text {
                            style.text_color(rgb(color))
                        } else {
                            style
                        }
                    })
                    .active(|s| s.bg(rgb(hover)))
            })
            .when(self.disabled, |d| {
                d.opacity(0.45)
                    .aria_description("Indisponible")
                    .capture_any_mouse_down(|_, _, cx| cx.stop_propagation())
                    .capture_any_mouse_up(|_, _, cx| cx.stop_propagation())
                    .capture_key_down(|event, window, cx| {
                        if ["enter", "space"].contains(&event.keystroke.key.as_str()) {
                            window.prevent_default();
                            cx.stop_propagation();
                        }
                    })
                    .capture_key_up(|event, window, cx| {
                        if ["enter", "space"].contains(&event.keystroke.key.as_str()) {
                            window.prevent_default();
                            cx.stop_propagation();
                        }
                    })
            })
            .when_some(self.icon, |d, icon| d.child(icon))
            .when(!self.icon_only, |d| d.child(self.label))
    }
}

pub(crate) struct NavigationButton {
    button: Button,
}
impl NavigationButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            button: Button::new(id, label),
        }
    }
    pub fn active(mut self, active: bool) -> Self {
        self.button = self.button.selected(active);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.button = self.button.disabled(disabled);
        self
    }
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.button = self.button.icon(icon);
        self
    }
    pub fn element(self) -> Stateful<Div> {
        use theme::tokens as t;
        let active = self.button.selected;
        self.button
            .element_with_hover(
                Some(if active {
                    t::TOKEN_COMPONENTS_NAVIGATION_ACTIVE_HOVER_BACKGROUND
                } else {
                    t::TOKEN_COMPONENTS_NAVIGATION_HOVER_BACKGROUND
                }),
                Some(t::TOKEN_COMPONENTS_NAVIGATION_HOVER_TEXT),
            )
            .aria_selected(active)
            .justify_start()
            .relative()
            .px(px(t::TOKEN_COMPONENTS_NAVIGATION_GAP))
            .gap(px(t::TOKEN_COMPONENTS_NAVIGATION_GAP))
            .h(px(t::TOKEN_COMPONENTS_NAVIGATION_HEIGHT))
            .min_w(px(t::TOKEN_COMPONENTS_NAVIGATION_WIDTH))
            .border_0()
            .bg(if active {
                rgb(t::TOKEN_COMPONENTS_NAVIGATION_ACTIVE_BACKGROUND).into()
            } else {
                gpui::transparent_black()
            })
            .text_color(rgb(if active {
                t::TOKEN_COMPONENTS_NAVIGATION_ACTIVE_TEXT
            } else {
                t::TOKEN_COMPONENTS_NAVIGATION_TEXT
            }))
            .font_weight(gpui::FontWeight(t::TOKEN_COMPONENTS_NAVIGATION_WEIGHT))
            .when(active, |d| {
                d.child(
                    div()
                        .absolute()
                        .left_0()
                        .top_0()
                        .h_full()
                        .w(px(t::TOKEN_COMPONENTS_NAVIGATION_MARKER))
                        .bg(rgb(theme::ACCENT)),
                )
            })
    }
}
