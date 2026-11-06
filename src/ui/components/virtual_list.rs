//! Shared lazy rendering for pages with variable sections and fixed-height rows.
//! Keep entities in the owner; the renderer builds only the requested items.
use super::ScrollArea;
use crate::ui::theme;
use gpui::{
    AnyElement, App, Div, ElementId, FocusHandle, ListAlignment, ListState, ScrollStrategy,
    Stateful, UniformListScrollHandle, Window, div, list, prelude::*, px, uniform_list,
};
use std::ops::Range;

pub(crate) struct VirtualList {
    state: ListState,
    scroll: ScrollArea,
    focus: Vec<FocusHandle>,
    gap: f32,
}
impl VirtualList {
    pub fn new(count: usize, cx: &App) -> Self {
        let state = ListState::new(
            0,
            ListAlignment::Top,
            px(theme::tokens::TOKEN_COMPONENTS_LAYOUT_VIRTUAL_OVERSCAN),
        )
        .measure_all();
        let mut this = Self {
            scroll: ScrollArea::for_list(state.clone()),
            state,
            focus: Vec::new(),
            gap: theme::SPACE * 4.,
        };
        this.set_count(count, cx);
        this
    }
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
    #[cfg(all(test, feature = "ui-tests"))]
    pub fn state(&self) -> &ListState {
        &self.state
    }
    pub fn set_count(&mut self, count: usize, cx: &App) {
        if self.state.item_count() == count {
            return;
        }
        let previous = self.focus.len();
        self.focus
            .resize_with(count, || cx.focus_handle().tab_stop(false));
        if count > previous {
            self.state.splice_focusable(
                previous..previous,
                self.focus[previous..].iter().cloned().map(Some),
            );
        } else {
            self.state.splice(count..previous, 0);
        }
        self.state.remeasure();
    }
    pub fn element(
        &self,
        id: impl Into<ElementId>,
        mut render: impl FnMut(usize, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Stateful<Div> {
        let focus = self.focus.clone();
        let gap = self.gap;
        self.scroll
            .wrap(
                id,
                div().id("virtual-viewport").size_full().child(
                    list(self.state.clone(), move |index, window, cx| {
                        div()
                            .id(("section", index))
                            .track_focus(&focus[index])
                            .pb(px(gap))
                            .child(render(index, window, cx))
                            .into_any_element()
                    })
                    .size_full(),
                ),
            )
            .flex_1()
    }
}

pub(crate) struct VirtualRows {
    handle: UniformListScrollHandle,
    scroll: ScrollArea,
}
impl Default for VirtualRows {
    fn default() -> Self {
        let handle = UniformListScrollHandle::default();
        Self {
            scroll: ScrollArea::for_uniform(&handle),
            handle,
        }
    }
}
impl VirtualRows {
    pub fn scroll_to_item(&self, index: usize, strategy: ScrollStrategy) {
        self.handle.scroll_to_item(index, strategy);
    }
    #[cfg(all(test, feature = "ui-tests"))]
    pub fn bounds(&self) -> gpui::Bounds<gpui::Pixels> {
        self.handle.0.borrow().base_handle.bounds()
    }
    pub fn element(
        &self,
        id: impl Into<ElementId>,
        count: usize,
        row_height: f32,
        render: impl Fn(Range<usize>, &mut Window, &mut App) -> Vec<AnyElement> + 'static,
    ) -> Stateful<Div> {
        let id = id.into();
        self.scroll.wrap(
            id.clone(),
            div().id("rows-viewport").size_full().child(
                uniform_list((id, "rows"), count, move |range, window, cx| {
                    render(range, window, cx)
                        .into_iter()
                        .map(|row| div().h(px(row_height)).child(row))
                        .collect::<Vec<_>>()
                })
                .track_scroll(&self.handle)
                .size_full(),
            ),
        )
    }
}
