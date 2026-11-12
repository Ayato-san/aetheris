//! Shared lazy rendering for pages with variable sections and fixed-height rows.
//! Keep entities in the owner; the renderer builds only the requested items.
use super::ScrollContainer;
use crate::ui::theme;
use gpui::{
    AnyElement, App, ElementId, FocusHandle, ListAlignment, ListState, ScrollStrategy,
    UniformListScrollHandle, Window, div, list, prelude::*, px, uniform_list,
};
use std::ops::Range;

pub(crate) struct VirtualList {
    state: ListState,
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
    ) -> ScrollContainer {
        let focus = self.focus.clone();
        let gap = self.gap;
        ScrollContainer::for_list(
            id,
            self.state.clone(),
            list(self.state.clone(), move |index, window, cx| {
                div()
                    .id(("section", index))
                    .track_focus(&focus[index])
                    .pb(px(gap))
                    .child(render(index, window, cx))
                    .into_any_element()
            })
            .size_full(),
        )
        .flex_1()
    }
}

#[derive(Default)]
pub(crate) struct VirtualRows {
    handle: UniformListScrollHandle,
    overlay: bool,
}
impl VirtualRows {
    pub fn overlay_scrollbar(mut self) -> Self {
        self.overlay = true;
        self
    }
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
    ) -> ScrollContainer {
        let id = id.into();
        ScrollContainer::for_uniform(
            id.clone(),
            self.handle.clone(),
            uniform_list((id, "rows"), count, move |range, window, cx| {
                render(range, window, cx)
                    .into_iter()
                    .map(|row| div().h(px(row_height)).child(row))
                    .collect::<Vec<_>>()
            })
            .track_scroll(&self.handle)
            .size_full(),
        )
        .overlay_scrollbar(self.overlay)
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::*;
    use gpui::{Context, Render, ScrollDelta, ScrollWheelEvent, TestAppContext, point};

    struct NestedHost {
        page: VirtualList,
        rows: std::rc::Rc<VirtualRows>,
        count: usize,
    }
    impl Render for NestedHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let rows = self.rows.clone();
            let count = self.count;
            self.page
                .element("page", move |index, _, _| {
                    if index == 0 {
                        rows.element("nested-rows", count, 30., move |range, _, _| {
                            range.map(|_| div().into_any_element()).collect()
                        })
                        .h(px(150.))
                        .into_any_element()
                    } else {
                        div().h(px(800.)).into_any_element()
                    }
                })
                .flex_none()
                .w(px(400.))
                .h(px(300.))
        }
    }

    #[gpui::test]
    fn nested_rows_consume_wheel_before_page_but_empty_rows_allow_page_scroll(
        cx: &mut TestAppContext,
    ) {
        let (host, cx) = cx.add_window_view(|_, cx| NestedHost {
            page: VirtualList::new(2, cx).gap(0.),
            rows: std::rc::Rc::new(VirtualRows::default()),
            count: 100,
        });
        let position = cx.update(|_, cx| host.read(cx).rows.bounds().center());
        let wheel = |cx: &mut gpui::VisualTestContext, position| {
            cx.simulate_event(ScrollWheelEvent {
                position,
                delta: ScrollDelta::Pixels(point(px(0.), px(-60.))),
                modifiers: gpui::Modifiers::default(),
                touch_phase: gpui::TouchPhase::Moved,
            });
        };
        wheel(cx, position);
        cx.update(|_, cx| {
            let host = host.read(cx);
            assert!(host.rows.handle.0.borrow().base_handle.offset().y < px(0.));
            assert_eq!(host.page.state.scroll_px_offset_for_scrollbar().y, px(0.));
        });
        // Even at the bottom, scrolling over the rows must not move the page.
        cx.update(|window, cx| {
            host.read(cx)
                .rows
                .scroll_to_item(99, ScrollStrategy::Bottom);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        wheel(cx, position);
        cx.update(|_, cx| {
            assert_eq!(
                host.read(cx).page.state.scroll_px_offset_for_scrollbar().y,
                px(0.)
            );
            host.update(cx, |host, cx| {
                host.count = 1;
                cx.notify();
            });
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        wheel(cx, position);
        cx.update(|_, cx| {
            assert!(host.read(cx).page.state.scroll_px_offset_for_scrollbar().y < px(0.));
        });
    }
}
