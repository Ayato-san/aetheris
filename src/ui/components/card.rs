use crate::ui::theme::{self, tokens as t};
use gpui::{Div, ElementId, SharedString, Stateful, div, prelude::*, px, rgb};

pub(crate) struct Card;
impl Card {
    pub fn interactive(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        disabled: bool,
        navigation: bool,
        on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    ) -> Stateful<Div> {
        Self::element(id, "")
            .hover(|s| s.border_color(rgb(t::TOKEN_COMPONENTS_CARD_HOVER_BORDER)))
            .child(
                super::Button::new("card-primary-action", title)
                    .disabled(disabled)
                    .element()
                    .when(navigation, |d| d.child("→"))
                    .on_click(on_click),
            )
    }
    /// Static composable container. Add a Button as the distinct primary action
    /// so child controls never trigger navigation through bubbling.
    pub fn element(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Stateful<Div> {
        div()
            .id(id)
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(px(t::TOKEN_COMPONENTS_CARD_GAP))
            .p(px(t::TOKEN_COMPONENTS_CARD_PADDING))
            .rounded(px(theme::RADIUS))
            .bg(rgb(t::TOKEN_COMPONENTS_CARD_BACKGROUND))
            .border_1()
            .border_color(rgb(t::TOKEN_COMPONENTS_CARD_BORDER))
            .font_family(theme::FONT_FAMILY)
            .text_size(px(theme::FONT_SIZE))
            .text_color(rgb(theme::TEXT))
            .when_some(
                Some(title.into()).filter(|title: &SharedString| !title.is_empty()),
                |d, title| {
                    d.child(
                        div()
                            .text_size(px(t::TOKEN_COMPONENTS_CARD_TITLE_SIZE))
                            .font_weight(gpui::FontWeight(t::TOKEN_FONT_BOLD))
                            .child(title),
                    )
                },
            )
    }
}
pub(crate) fn divider() -> Div {
    div()
        .h(px(theme::BORDER_WIDTH))
        .w_full()
        .bg(rgb(theme::BORDER))
}
