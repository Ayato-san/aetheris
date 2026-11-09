//! Persistent vertical scroll state with Penpot's visible track and thumb.
use crate::ui::theme::tokens as t;
use gpui::{
    Bounds, CursorStyle, Div, ElementId, HitboxBehavior, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ScrollHandle, Stateful, canvas, div, point, prelude::*, px, quad,
    rgb, size,
};
use std::{cell::Cell, rc::Rc};

#[derive(Default)]
pub(crate) struct ScrollArea {
    pub handle: ScrollHandle,
    drag: Rc<Cell<Option<f32>>>,
    list: Option<gpui::ListState>,
    external: bool,
    overlay: bool,
}

// Native divs and virtual lists expose distinct scroll APIs.
#[derive(Clone)]
enum ScrollTarget {
    Div(ScrollHandle),
    List(gpui::ListState),
}
impl ScrollTarget {
    fn bounds(&self) -> Bounds<gpui::Pixels> {
        match self {
            Self::Div(h) => h.bounds(),
            Self::List(h) => h.viewport_bounds(),
        }
    }
    fn offset(&self) -> gpui::Point<gpui::Pixels> {
        match self {
            Self::Div(h) => h.offset(),
            Self::List(h) => h.scroll_px_offset_for_scrollbar(),
        }
    }
    fn max_offset(&self) -> gpui::Point<gpui::Pixels> {
        match self {
            Self::Div(h) => h.max_offset(),
            Self::List(h) => h.max_offset_for_scrollbar(),
        }
    }
    fn set_offset(&self, offset: gpui::Point<gpui::Pixels>) {
        match self {
            Self::Div(h) => h.set_offset(offset),
            Self::List(h) => h.set_offset_from_scrollbar(offset),
        }
    }
    fn start_drag(&self) {
        if let Self::List(list) = self {
            list.scrollbar_drag_started();
        }
    }
    fn end_drag(&self) {
        if let Self::List(list) = self {
            list.scrollbar_drag_ended();
        }
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::*;
    use gpui::{Context, Modifiers, Render, TestAppContext, Window};

    #[gpui::test]
    fn virtual_list_scrollbar_reaches_end_and_keeps_focused_drafts(cx: &mut TestAppContext) {
        use super::super::VirtualList;
        use super::super::text_input::TextInput;
        use gpui::{Focusable, ListOffset};
        struct VirtualHost {
            page: VirtualList,
            input: gpui::Entity<TextInput>,
            rendered: Rc<Cell<usize>>,
        }
        impl Render for VirtualHost {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let rendered = self.rendered.clone();
                let input = self.input.clone();
                self.page
                    .element("virtual-area", move |index, _, _| {
                        rendered.set(rendered.get() + 1);
                        div()
                            .h(px(40.))
                            .when(index == 20, |d| d.child(input.clone()))
                            .into_any_element()
                    })
                    .w(px(300.))
                    .h(px(200.))
            }
        }
        cx.update(TextInput::register);
        let (host, cx) = cx.add_window_view(|_, cx| VirtualHost {
            page: VirtualList::new(100, cx).gap(0.),
            input: cx.new(|cx| TextInput::text("Brouillon", cx)),
            rendered: Rc::new(Cell::new(0)),
        });
        let bounds = cx.update(|window, cx| {
            let host = host.read(cx);
            assert_eq!(host.page.state().max_offset_for_scrollbar().y, px(3800.));
            host.page.state().scroll_to_reveal_item(20);
            window.refresh();
            host.page.state().viewport_bounds()
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, cx| window.focus(&host.read(cx).input.read(cx).focus_handle(cx), cx));
        cx.simulate_input("brouillon conservé");
        cx.update(|window, cx| {
            let host = host.read(cx);
            host.rendered.set(0);
            host.page.state().scroll_to(ListOffset {
                item_ix: 0,
                offset_in_item: px(0.),
            });
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.update(|window, cx| {
            let host = host.read(cx);
            assert!(host.rendered.get() < 20);
            assert_eq!(host.input.read(cx).content.as_ref(), "brouillon conservé");
            assert!(host.input.read(cx).focus_handle(cx).is_focused(window));
        });
        let track = point(bounds.right() + px(10.), bounds.top() + px(16.));
        let end = point(track.x, bounds.bottom() + px(100.));
        cx.simulate_mouse_move(track, None, Modifiers::default());
        cx.simulate_mouse_down(track, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::default());
        cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
        cx.update(|_, cx| {
            let list = host.read(cx).page.state();
            assert_eq!(
                list.scroll_px_offset_for_scrollbar().y,
                -list.max_offset_for_scrollbar().y
            );
            assert!(!list.is_scrollbar_dragging());
        });
    }

    struct Host {
        area: ScrollArea,
        rows: usize,
    }
    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.area
                .wrap(
                    "area",
                    div()
                        .id("content")
                        .h_full()
                        .overflow_y_scroll()
                        .flex()
                        .flex_col()
                        .children((0..self.rows).map(|_| div().h(px(30.)).flex_shrink_0())),
                )
                .w(px(300.))
                .h(px(200.))
        }
    }

    #[gpui::test]
    fn wheel_track_click_and_drag_outside_share_the_scroll_offset(cx: &mut TestAppContext) {
        let (host, cx) = cx.add_window_view(|_, _| Host {
            area: ScrollArea::default(),
            rows: 100,
        });
        let bounds = cx.update(|_, cx| host.read(cx).area.handle.bounds());
        assert_eq!(bounds.size.height, px(200.));
        cx.update(|window, cx| {
            window.dispatch_event(
                gpui::PlatformInput::ScrollWheel(gpui::ScrollWheelEvent {
                    position: bounds.center(),
                    delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-100.))),
                    modifiers: Modifiers::default(),
                    touch_phase: gpui::TouchPhase::Moved,
                }),
                cx,
            );
        });
        cx.update(|_, cx| assert!(host.read(cx).area.handle.offset().y < px(0.)));
        let track = point(bounds.right() - px(10.), bounds.center().y);
        cx.simulate_mouse_move(track, None, Modifiers::default());
        cx.simulate_mouse_down(track, MouseButton::Left, Modifiers::default());
        cx.update(|_, cx| assert!(host.read(cx).area.handle.offset().y < px(-100.)));
        let outside = point(bounds.right() + px(100.), bounds.bottom() + px(100.));
        cx.simulate_mouse_move(outside, Some(MouseButton::Left), Modifiers::default());
        cx.simulate_mouse_up(outside, MouseButton::Left, Modifiers::default());
        cx.update(|_, cx| {
            let area = &host.read(cx).area;
            assert_eq!(area.handle.offset().y, -area.handle.max_offset().y);
            assert!(area.drag.get().is_none());
            host.update(cx, |host, cx| {
                host.rows = 1;
                cx.notify();
            });
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|_, cx| {
            let area = &host.read(cx).area;
            assert_eq!(area.handle.max_offset().y, px(0.));
            assert_eq!(area.handle.offset().y, px(0.));
        });
    }
}

#[derive(Clone, Copy)]
struct Geometry {
    thumb: f32,
    travel: f32,
    top: f32,
    maximum: f32,
}
impl Geometry {
    fn new(viewport: f32, maximum: f32, track: f32, offset: f32) -> Option<Self> {
        if viewport <= 0. || maximum <= 0. || track <= 0. {
            return None;
        }
        let thumb = (track * viewport / (viewport + maximum))
            .max(t::TOKEN_COMPONENTS_SCROLLBAR_MIN_THUMB)
            .min(track);
        let travel = track - thumb;
        Some(Self {
            thumb,
            travel,
            top: (-offset / maximum).clamp(0., 1.) * travel,
            maximum,
        })
    }
    fn offset(self, position: f32) -> f32 {
        if self.travel <= 0. {
            return 0.;
        }
        -(position / self.travel).clamp(0., 1.) * self.maximum
    }
}

impl ScrollArea {
    pub fn overlay_scrollbar(mut self) -> Self {
        self.overlay = true;
        self
    }
    pub fn for_uniform(handle: &gpui::UniformListScrollHandle) -> Self {
        Self {
            handle: handle.0.borrow().base_handle.clone(),
            external: true,
            ..Self::default()
        }
    }
    pub fn for_list(list: gpui::ListState) -> Self {
        Self {
            list: Some(list),
            ..Self::default()
        }
    }
    /// The track is a sibling of the scrolled content, so it stays fixed.
    pub fn wrap(&self, id: impl Into<ElementId>, content: Stateful<Div>) -> Stateful<Div> {
        let handle = self.list.as_ref().map_or_else(
            || ScrollTarget::Div(self.handle.clone()),
            |list| ScrollTarget::List(list.clone()),
        );
        let measure = handle.clone();
        let wheel_target = handle.clone();
        let drag = self.drag.clone();
        let overlay = self.overlay;
        let gutter = t::TOKEN_COMPONENTS_SCROLLBAR_WIDTH + t::TOKEN_COMPONENTS_SCROLLBAR_INSET * 2.;
        div()
            .id(id)
            .relative()
            .flex()
            .flex_col()
            .min_h_0()
            .min_w_0()
            // Children process the wheel first. Keep their scroll from also
            // moving an enclosing page while this viewport can scroll.
            .on_scroll_wheel(move |event, _, cx| {
                let delta = event.delta.pixel_delta(px(20.));
                let maximum = wheel_target.max_offset();
                if (delta.y != px(0.) && maximum.y > px(0.))
                    || (delta.x != px(0.) && maximum.x > px(0.))
                {
                    cx.stop_propagation();
                }
            })
            .child(
                content
                    .when(self.list.is_none() && !self.external, |content| {
                        content.track_scroll(&self.handle)
                    })
                    .when(!overlay, |content| content.pr(px(gutter))),
            )
            .child(
                canvas(
                    move |bounds, window, _| {
                        (measure.max_offset().y > px(0.) && bounds.size.height > px(0.)).then(
                            || {
                                window.insert_hitbox(
                                    bounds,
                                    if overlay {
                                        HitboxBehavior::BlockMouseExceptScroll
                                    } else {
                                        HitboxBehavior::Normal
                                    },
                                )
                            },
                        )
                    },
                    move |bounds, hitbox, window, _| {
                        let Some(hitbox) = hitbox else {
                            drag.set(None);
                            return;
                        };
                        let Some(geometry) = Geometry::new(
                            handle.bounds().size.height.into(),
                            handle.max_offset().y.into(),
                            bounds.size.height.into(),
                            handle.offset().y.into(),
                        ) else {
                            drag.set(None);
                            return;
                        };
                        let track = Bounds::new(
                            point(
                                bounds.left() + px(t::TOKEN_COMPONENTS_SCROLLBAR_INSET),
                                bounds.top(),
                            ),
                            size(px(t::TOKEN_COMPONENTS_SCROLLBAR_WIDTH), bounds.size.height),
                        );
                        let thumb = Bounds::new(
                            point(track.left(), track.top() + px(geometry.top)),
                            size(track.size.width, px(geometry.thumb)),
                        );
                        window.paint_quad(quad(
                            track,
                            px(t::TOKEN_COMPONENTS_SCROLLBAR_WIDTH / 2.),
                            rgb(t::TOKEN_COMPONENTS_SCROLLBAR_TRACK),
                            px(0.),
                            gpui::transparent_black(),
                            gpui::BorderStyle::default(),
                        ));
                        window.paint_quad(quad(
                            thumb,
                            px(t::TOKEN_COMPONENTS_SCROLLBAR_WIDTH / 2.),
                            rgb(t::TOKEN_COMPONENTS_SCROLLBAR_THUMB),
                            px(0.),
                            gpui::transparent_black(),
                            gpui::BorderStyle::default(),
                        ));
                        window.set_cursor_style(CursorStyle::Arrow, &hitbox);
                        let down_handle = handle.clone();
                        let down_drag = drag.clone();
                        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                            if !phase.bubble()
                                || event.button != MouseButton::Left
                                || !hitbox.is_hovered(window)
                            {
                                return;
                            }
                            let y = f32::from(event.position.y - bounds.top());
                            let anchor = if y >= geometry.top && y <= geometry.top + geometry.thumb
                            {
                                y - geometry.top
                            } else {
                                geometry.thumb / 2.
                            };
                            down_handle.start_drag();
                            down_drag.set(Some(anchor));
                            let mut offset = down_handle.offset();
                            offset.y = px(geometry.offset(y - anchor));
                            down_handle.set_offset(offset);
                            window.refresh();
                            cx.stop_propagation();
                        });
                        let move_handle = handle.clone();
                        let move_drag = drag.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                            if !phase.bubble() {
                                return;
                            }
                            let Some(anchor) = move_drag.get() else {
                                return;
                            };
                            if event.pressed_button != Some(MouseButton::Left) {
                                move_drag.set(None);
                                move_handle.end_drag();
                                return;
                            }
                            let mut offset = move_handle.offset();
                            offset.y = px(geometry
                                .offset(f32::from(event.position.y - bounds.top()) - anchor));
                            move_handle.set_offset(offset);
                            window.refresh();
                            cx.stop_propagation();
                        });
                        let up_drag = drag.clone();
                        let up_handle = handle.clone();
                        window.on_mouse_event(move |event: &MouseUpEvent, phase, window, _| {
                            if phase.capture()
                                && event.button == MouseButton::Left
                                && up_drag.take().is_some()
                            {
                                up_handle.end_drag();
                                window.refresh();
                            }
                        });
                    },
                )
                .absolute()
                .right_0()
                .top(px(t::TOKEN_COMPONENTS_SCROLLBAR_INSET))
                .bottom(px(t::TOKEN_COMPONENTS_SCROLLBAR_INSET))
                .w(px(gutter)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn thumb_tracks_content_and_clamps_dragging() {
        assert!(Geometry::new(200., 0., 184., 0.).is_none());
        let g = Geometry::new(200., 800., 184., -400.).unwrap();
        assert_eq!(g.thumb, 36.8);
        assert!((g.offset(g.top) + 400.).abs() < 0.001);
        assert_eq!(g.offset(-100.), 0.);
        assert_eq!(g.offset(1000.), -800.);
        let g = Geometry::new(100., 10000., 10., 0.).unwrap();
        assert_eq!(g.thumb, 10.);
        assert_eq!(g.offset(10.), 0.);
    }
}
