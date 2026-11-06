//! Penpot's segmented control. Selector owns acceptance and keyboard state;
//! these elements provide the pill geometry independently of action buttons.
use crate::ui::theme::{
    self,
    controls::{self, segment_group as tokens},
};
use gpui::{Div, ElementId, SharedString, Stateful, div, prelude::*, px, rgb};

pub(crate) struct SegmentGroup;
impl SegmentGroup {
    pub fn element(id: impl Into<ElementId>) -> Stateful<Div> {
        div()
            .id(id)
            .flex()
            .items_center()
            .w(px(tokens::WIDTH))
            .max_w_full()
            .h(px(tokens::HEIGHT))
            .p(px(tokens::PADDING))
            .rounded_full()
            .bg(rgb(tokens::BACKGROUND))
            .overflow_x_scroll()
            .font_family(controls::FONT_FAMILY)
            .font_weight(tokens::WEIGHT)
            .text_size(px(controls::FONT_SIZE))
            .line_height(px(controls::LINE_HEIGHT))
    }
    pub fn segment(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        selected: bool,
        disabled: bool,
    ) -> Stateful<Div> {
        let label = label.into();
        div()
            .id(id)
            .role(gpui::Role::RadioButton)
            .aria_label(label.clone())
            .aria_toggled(if selected {
                gpui::Toggled::True
            } else {
                gpui::Toggled::False
            })
            .aria_selected(selected)
            .tab_stop(false)
            .flex()
            .flex_1()
            .flex_shrink_0()
            .min_w(px(theme::OPTION_MIN_WIDTH))
            .h(px(tokens::PILL_HEIGHT))
            .items_center()
            .justify_center()
            .rounded_full()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_color(rgb(if disabled {
                tokens::DISABLED_TEXT
            } else if selected {
                tokens::ACTIVE_TEXT
            } else {
                tokens::TEXT
            }))
            .when(selected, |d| d.bg(rgb(tokens::ACTIVE_BACKGROUND)))
            .when(!disabled, |d| {
                d.cursor_pointer().when(!selected, |d| {
                    d.hover(|s| {
                        s.bg(rgb(tokens::HOVER_BACKGROUND))
                            .text_color(rgb(tokens::HOVER_TEXT))
                    })
                })
            })
            .when(disabled, |d| {
                d.aria_description("Indisponible")
                    .capture_any_mouse_down(|_, _, cx| cx.stop_propagation())
                    .capture_any_mouse_up(|_, _, cx| cx.stop_propagation())
            })
            .child(label)
    }
}
