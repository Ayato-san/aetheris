//! Constrained content layout with automatic, persistent overflow scrolling.
use super::scrollbar::ScrollArea;
use gpui::{
    AnyElement, App, Div, ElementId, IntoElement, RenderOnce, StyleRefinement, Window, div,
    prelude::*,
};

enum ScrollMode {
    Content,
    List(gpui::ListState),
    Uniform(gpui::UniformListScrollHandle),
}

/// Supply natural-height content and size this container to its available space.
/// Scroll state belongs to the layout; the track appears only on overflow.
/// Keep the ID stable across renders, as for other stateful GPUI elements.
#[derive(IntoElement)]
pub(crate) struct ScrollContainer {
    id: ElementId,
    container: Div,
    content: AnyElement,
    mode: ScrollMode,
    overlay: bool,
}
impl ScrollContainer {
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            container: div().flex().flex_col().min_w_0().min_h_0(),
            content: content.into_any_element(),
            mode: ScrollMode::Content,
            overlay: false,
        }
    }
    pub(super) fn for_list(
        id: impl Into<ElementId>,
        state: gpui::ListState,
        viewport: impl IntoElement,
    ) -> Self {
        Self {
            mode: ScrollMode::List(state),
            ..Self::new(id, viewport)
        }
    }
    pub(super) fn for_uniform(
        id: impl Into<ElementId>,
        handle: gpui::UniformListScrollHandle,
        viewport: impl IntoElement,
    ) -> Self {
        Self {
            mode: ScrollMode::Uniform(handle),
            ..Self::new(id, viewport)
        }
    }
    pub(super) fn overlay_scrollbar(mut self, overlay: bool) -> Self {
        self.overlay = overlay;
        self
    }
}
impl Styled for ScrollContainer {
    fn style(&mut self) -> &mut StyleRefinement {
        self.container.style()
    }
}
impl RenderOnce for ScrollContainer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let area = window.use_keyed_state((self.id.clone(), "scroll-state"), cx, |_, _| {
            ScrollArea::default()
        });
        let area = area.read(cx);
        let (mut scroll, viewport) = match self.mode {
            ScrollMode::Content => (
                area.clone(),
                div()
                    .id("scroll-viewport")
                    .size_full()
                    .overflow_y_scroll()
                    .child(self.content),
            ),
            ScrollMode::List(state) => (
                area.with_target(ScrollArea::for_list(state)),
                div().id("scroll-viewport").size_full().child(self.content),
            ),
            ScrollMode::Uniform(handle) => (
                area.with_target(ScrollArea::for_uniform(&handle)),
                div().id("scroll-viewport").size_full().child(self.content),
            ),
        };
        if self.overlay {
            scroll = scroll.overlay_scrollbar();
        }
        self.container
            .id(self.id)
            .child(scroll.wrap("scroll-area", viewport).size_full().flex_1())
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod tests {
    use super::*;
    use gpui::{
        Bounds, Context, Modifiers, MouseButton, Pixels, Render, TestAppContext, point, px,
    };
    use std::{cell::Cell, rc::Rc};

    struct Host {
        rows: usize,
        height: f32,
        capped: bool,
        marker: Rc<Cell<Bounds<Pixels>>>,
    }
    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let marker = self.marker.clone();
            ScrollContainer::new(
                "automatic-scroll",
                div()
                    .flex()
                    .flex_col()
                    .children((0..self.rows).map(|index| {
                        let marker = marker.clone();
                        div().h(px(30.)).flex_shrink_0().when(index == 0, |row| {
                            row.child(
                                gpui::canvas(
                                    move |bounds, _, _| marker.set(bounds),
                                    |_, _, _, _| {},
                                )
                                .size_full(),
                            )
                        })
                    })),
            )
            .w(px(300.))
            .when(self.capped, |layout| layout.max_h(px(self.height)))
            .when(!self.capped, |layout| layout.h(px(self.height)))
        }
    }
    fn wheel(cx: &mut gpui::VisualTestContext, position: gpui::Point<Pixels>) {
        cx.simulate_event(gpui::ScrollWheelEvent {
            position,
            delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-90.))),
            modifiers: Modifiers::default(),
            touch_phase: gpui::TouchPhase::Moved,
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }
    #[gpui::test]
    fn overflow_tracks_content_and_container_size_without_page_scroll_state(
        cx: &mut TestAppContext,
    ) {
        let (host, cx) = cx.add_window_view(|_, _| Host {
            rows: 2,
            height: 200.,
            capped: false,
            marker: Rc::new(Cell::new(Bounds::default())),
        });
        let initial = cx.update(|_, cx| host.read(cx).marker.get());
        wheel(cx, initial.center());
        cx.update(|_, cx| assert_eq!(host.read(cx).marker.get(), initial));

        host.update(cx, |host, cx| {
            host.rows = 30;
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        wheel(cx, initial.center());
        let scrolled = cx.update(|_, cx| host.read(cx).marker.get().top());
        assert!(scrolled < initial.top());
        // Ordinary rerenders retain the offset without state in the owner.
        host.update(cx, |_, cx| cx.notify());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|_, cx| assert_eq!(host.read(cx).marker.get().top(), scrolled));

        host.update(cx, |host, cx| {
            host.height = 1200.;
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|_, cx| assert_eq!(host.read(cx).marker.get().top(), initial.top()));
        host.update(cx, |host, cx| {
            host.height = 200.;
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        wheel(cx, initial.center());
        cx.update(|_, cx| assert!(host.read(cx).marker.get().top() < initial.top()));
        host.update(cx, |host, cx| {
            host.rows = 1;
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|_, cx| assert_eq!(host.read(cx).marker.get().top(), initial.top()));
    }
    #[gpui::test]
    fn max_height_and_track_drag_work_without_an_explicit_height(cx: &mut TestAppContext) {
        let (host, cx) = cx.add_window_view(|_, _| Host {
            rows: 30,
            height: 200.,
            capped: true,
            marker: Rc::new(Cell::new(Bounds::default())),
        });
        let before = cx.update(|_, cx| host.read(cx).marker.get());
        wheel(cx, before.center());
        cx.update(|_, cx| assert!(host.read(cx).marker.get().top() < before.top()));
        let track = point(px(290.), px(100.));
        let end = point(px(350.), px(300.));
        cx.simulate_mouse_move(track, None, Modifiers::default());
        cx.simulate_mouse_down(track, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|_, cx| assert_eq!(host.read(cx).marker.get().top(), before.top() - px(700.)));
    }
}
