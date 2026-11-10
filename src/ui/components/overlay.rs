//! Shared overlay placement, focus ownership and dismissal. A panel remains in
//! the trigger's logical subtree, so nested outside-click and Escape stay local.
use super::{Button, state::Validation};
use crate::ui::theme;
use gpui::{
    App, Context, Entity, FocusHandle, IntoElement, Window, anchored, deferred, div, prelude::*,
    px, rgb, rgba,
};

// A rectangle alone is insufficient for outside clicks: a deferred child can
// extend beyond its parent's rectangle. Track ancestry once, at opening.
#[derive(Default)]
struct OverlayRegistry {
    layers: Vec<Layer>,
}
impl gpui::Global for OverlayRegistry {}
struct Layer {
    id: gpui::EntityId,
    parent: Option<gpui::EntityId>,
    focus: gpui::WeakFocusHandle,
    bounds: Option<gpui::Bounds<gpui::Pixels>>,
}
pub(super) fn registered(id: gpui::EntityId, cx: &App) -> bool {
    cx.try_global::<OverlayRegistry>()
        .is_some_and(|registry| registry.layers.iter().any(|layer| layer.id == id))
}
pub(super) fn register(id: gpui::EntityId, focus: &FocusHandle, window: &Window, cx: &mut App) {
    let parent = cx.try_global::<OverlayRegistry>().and_then(|registry| {
        registry
            .layers
            .iter()
            .rev()
            .find(|layer| {
                layer.id != id
                    && layer
                        .focus
                        .upgrade()
                        .is_some_and(|focus| focus.contains_focused(window, cx))
            })
            .map(|layer| layer.id)
    });
    let registry = cx.default_global::<OverlayRegistry>();
    registry
        .layers
        .retain(|layer| layer.id != id && layer.focus.upgrade().is_some());
    registry.layers.push(Layer {
        id,
        parent,
        focus: focus.downgrade(),
        bounds: None,
    });
}
pub(super) fn unregister(id: gpui::EntityId, cx: &mut App) {
    let registry = cx.default_global::<OverlayRegistry>();
    let mut removed = std::collections::BTreeSet::from([id]);
    for layer in &registry.layers {
        if layer.parent.is_some_and(|parent| removed.contains(&parent)) {
            removed.insert(layer.id);
        }
    }
    registry.layers.retain(|layer| !removed.contains(&layer.id));
}
pub(crate) fn dismiss_all(cx: &mut App) {
    cx.default_global::<OverlayRegistry>().layers.clear();
}
fn region(id: gpui::EntityId, bounds: gpui::Bounds<gpui::Pixels>, cx: &mut App) {
    if let Some(layer) = cx
        .default_global::<OverlayRegistry>()
        .layers
        .iter_mut()
        .find(|layer| layer.id == id)
    {
        layer.bounds = Some(bounds);
    }
}
pub(super) fn contains(id: gpui::EntityId, position: gpui::Point<gpui::Pixels>, cx: &App) -> bool {
    let Some(registry) = cx.try_global::<OverlayRegistry>() else {
        return false;
    };
    registry.layers.iter().any(|layer| {
        if !layer
            .bounds
            .is_some_and(|bounds| bounds.contains(&position))
        {
            return false;
        }
        let mut current = Some(layer.id);
        while let Some(parent) = current {
            if parent == id {
                return true;
            }
            current = registry
                .layers
                .iter()
                .find(|layer| layer.id == parent)
                .and_then(|layer| layer.parent);
        }
        false
    })
}

/// Shared appearance for popup panels and non-modal notifications.
pub(crate) fn popup_surface() -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .p_4()
        .bg(rgb(theme::PANEL))
        .border_1()
        .border_color(rgb(theme::BORDER))
        .rounded(px(theme::RADIUS))
}

type CloseAcceptance = dyn Fn(CloseReason, &mut App) -> Validation<()>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CloseReason {
    Escape,
    Outside,
    Button,
    Programmatic,
}
#[derive(Clone, Copy)]
pub(crate) enum Placement {
    Bottom,
    Top,
    Left,
    Right,
}
#[derive(Clone, Copy)]
pub(crate) enum OverlayKind {
    Popover,
    Popup { modal: bool },
}
pub(crate) struct Overlay {
    scroll: super::ScrollArea,
    pub open: bool,
    initial_focus_pending: bool,
    pub kind: OverlayKind,
    pub placement: Placement,
    pub close_on_outside: bool,
    focus: FocusHandle,
    trigger_focus: FocusHandle,
    trigger_bounds: Option<gpui::Bounds<gpui::Pixels>>,
    width: f32,
    max_height: f32,
    trigger: Option<gpui::WeakFocusHandle>,
    first: FocusHandle,
    last: FocusHandle,
    _focus_subscriptions: Vec<gpui::Subscription>,
    on_close_requested: Box<CloseAcceptance>,
}
impl Overlay {
    pub fn new(kind: OverlayKind, cx: &mut Context<Self>) -> Self {
        let first = cx.focus_handle().tab_stop(true);
        let last = cx.focus_handle().tab_stop(true);
        Self {
            scroll: super::ScrollArea::default(),
            open: false,
            initial_focus_pending: false,
            kind,
            placement: Placement::Bottom,
            close_on_outside: matches!(
                kind,
                OverlayKind::Popover | OverlayKind::Popup { modal: true }
            ),
            focus: cx
                .focus_handle()
                .tab_stop(matches!(kind, OverlayKind::Popover)),
            trigger_focus: cx.focus_handle().tab_stop(true),
            trigger_bounds: None,
            width: theme::OVERLAY_WIDTH,
            max_height: theme::OVERLAY_HEIGHT,
            trigger: None,
            first,
            last,
            _focus_subscriptions: Vec::new(),
            on_close_requested: Box::new(|_, _| Ok(())),
        }
    }
    pub fn dimensions(
        &mut self,
        width: f32,
        max_height: f32,
        cx: &mut Context<Self>,
    ) -> Validation<()> {
        if !width.is_finite() || !max_height.is_finite() || width <= 0. || max_height <= 0. {
            return Err("Dimensions invalides".into());
        }
        self.width = width;
        self.max_height = max_height;
        cx.notify();
        Ok(())
    }
    pub fn trigger(
        entity: &Entity<Self>,
        label: impl Into<gpui::SharedString>,
        cx: &App,
    ) -> gpui::Stateful<gpui::Div> {
        let control = entity.read(cx);
        let focus = control.trigger_focus.clone();
        let open = control.open;
        let measure = entity.clone();
        let owner = entity.clone();
        Button::new(("overlay-trigger", entity.entity_id()), label)
            .element()
            .relative()
            .track_focus(&focus)
            .aria_expanded(open)
            .child(
                gpui::canvas(
                    move |bounds, _, cx| {
                        measure.update(cx, |this, _| this.trigger_bounds = Some(bounds))
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .on_click(move |_, window, cx| {
                owner.update(cx, |this, cx| {
                    if this.open {
                        this.close(CloseReason::Programmatic, window, cx);
                    } else {
                        this.show(window, cx);
                    }
                })
            })
    }
    pub fn on_close_requested(
        &mut self,
        callback: impl Fn(CloseReason, &mut App) -> Validation<()> + 'static,
    ) {
        self.on_close_requested = Box::new(callback);
    }
    pub fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            return;
        }
        if self._focus_subscriptions.is_empty()
            && matches!(self.kind, OverlayKind::Popup { modal: true })
        {
            self._focus_subscriptions
                .push(cx.on_focus(&self.first, window, |this, window, cx| {
                    if this.open && this.first.is_focused(window) {
                        window.focus(&this.last, cx);
                        window.focus_prev(cx);
                    }
                }));
            self._focus_subscriptions
                .push(cx.on_focus(&self.last, window, |this, window, cx| {
                    if this.open && this.last.is_focused(window) {
                        window.focus(&this.first, cx);
                        window.focus_next(cx);
                    }
                }));
            self._focus_subscriptions.push(cx.on_focus_out(
                &self.focus,
                window,
                |this, _, window, cx| {
                    if this.open && !this.focus.contains_focused(window, cx) {
                        window.focus(&this.first, cx);
                        window.focus_next(cx);
                    }
                },
            ));
        }
        self.trigger = window.focused(cx).map(|h| h.downgrade());
        register(cx.entity_id(), &self.focus, window, cx);
        self.open = true;
        self.initial_focus_pending = matches!(self.kind, OverlayKind::Popup { modal: true });
        window.focus(&self.focus, cx);
        cx.notify();
    }
    pub fn close(&mut self, reason: CloseReason, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open || (self.on_close_requested)(reason, cx).is_err() {
            return;
        }
        self.open = false;
        unregister(cx.entity_id(), cx);
        // Removed focus handles do not survive as valid tab stops in the window.
        if let Some(trigger) = self
            .trigger
            .take()
            .and_then(|h| h.upgrade())
            .filter(|h| h.tab_stop)
        {
            window.focus(&trigger, cx);
        }
        cx.notify();
    }
    pub fn panel(
        entity: &Entity<Self>,
        content: impl IntoElement,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyElement {
        if entity.read(cx).open && !registered(entity.entity_id(), cx) {
            entity.update(cx, |this, cx| {
                this.open = false;
                cx.notify();
            });
        }
        if entity.read(cx).initial_focus_pending && entity.read(cx).open {
            let first = entity.read(cx).first.clone();
            entity.update(cx, |this, _| this.initial_focus_pending = false);
            window.on_next_frame(move |window, cx| {
                window.focus(&first, cx);
                window.focus_next(cx);
            });
        }
        let this = entity.read(cx);
        if !this.open {
            return div().into_any_element();
        }
        let modal = matches!(this.kind, OverlayKind::Popup { modal: true });
        let outside = this.close_on_outside;
        let handle = this.focus.clone();
        let owner = entity.clone();
        let close = entity.clone();
        let close_button = entity.clone();
        let id = entity.entity_id();
        let panel = popup_surface()
            .id(("overlay", entity.entity_id()))
            .track_focus(&handle)
            .tab_group()
            .tab_stop(!modal)
            .role(if matches!(this.kind, OverlayKind::Popup { .. }) {
                gpui::Role::Dialog
            } else {
                gpui::Role::Group
            })
            .aria_label("Panneau")
            .relative()
            .w(px(this.width))
            .max_w(window.viewport_size().width - px(theme::OVERLAY_MARGIN * 2.))
            .max_h(
                (window.viewport_size().height - px(theme::OVERLAY_MARGIN * 2.))
                    .min(px(this.max_height)),
            )
            .overflow_y_scroll()
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" {
                    owner.update(cx, |this, cx| this.close(CloseReason::Escape, window, cx));
                    cx.stop_propagation();
                } else if event.keystroke.key == "tab" && modal {
                    if event.keystroke.modifiers.shift {
                        window.focus_prev(cx);
                    } else {
                        window.focus_next(cx);
                    }
                    cx.stop_propagation();
                }
            })
            .on_mouse_down_out(move |event, window, cx| {
                if outside && !contains(id, event.position, cx) {
                    close.update(cx, |this, cx| this.close(CloseReason::Outside, window, cx));
                }
            })
            .when(modal, |d| {
                d.child(
                    div()
                        .id("focus-start")
                        .track_focus(&this.first)
                        .tab_stop(true)
                        .size(px(0.)),
                )
            })
            .child(
                gpui::canvas(move |bounds, _, cx| region(id, bounds, cx), |_, _, _, _| {})
                    .absolute()
                    .size_full(),
            )
            .child(content)
            .when(matches!(this.kind, OverlayKind::Popup { .. }), |d| {
                d.child(Button::new("close-popup", "Fermer").element().on_click(
                    move |_, window, cx| {
                        close_button
                            .update(cx, |this, cx| this.close(CloseReason::Button, window, cx))
                    },
                ))
            })
            .when(modal, |d| {
                d.child(
                    div()
                        .id("focus-end")
                        .track_focus(&this.last)
                        .tab_stop(true)
                        .size(px(0.)),
                )
            });
        let panel = this.scroll.wrap(("overlay-scroll-area", id), panel);
        match this.kind {
            OverlayKind::Popover => {
                floating_at(panel, this.placement, this.trigger_bounds, id).into_any_element()
            }
            OverlayKind::Popup { modal } => deferred(
                anchored().position(gpui::point(px(0.), px(0.))).child(
                    div()
                        .w(window.viewport_size().width)
                        .h(window.viewport_size().height)
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(modal, |d| d.bg(rgba(theme::SCRIM)).occlude())
                        .child(panel),
                ),
            )
            .with_priority(1)
            .into_any_element(),
        }
    }
}
/// GPUI switches the anchor when the preferred side overflows the viewport.
pub(crate) fn floating_at(
    content: impl IntoElement,
    placement: Placement,
    bounds: Option<gpui::Bounds<gpui::Pixels>>,
    id: gpui::EntityId,
) -> impl IntoElement {
    let anchor = match placement {
        Placement::Bottom => gpui::Anchor::TopLeft,
        Placement::Top => gpui::Anchor::BottomLeft,
        Placement::Left => gpui::Anchor::TopRight,
        Placement::Right => gpui::Anchor::TopLeft,
    };
    let mut panel = anchored().anchor(anchor);
    if let Some(bounds) = bounds {
        let position = match placement {
            Placement::Bottom => gpui::point(bounds.left(), bounds.bottom()),
            Placement::Top | Placement::Left => bounds.origin,
            Placement::Right => gpui::point(bounds.right(), bounds.top()),
        };
        panel = panel.position(position);
    }
    deferred(
        panel.child(
            div()
                .relative()
                .occlude()
                .child(
                    gpui::canvas(move |bounds, _, cx| region(id, bounds, cx), |_, _, _, _| {})
                        .absolute()
                        .size_full(),
                )
                .child(content),
        ),
    )
    .with_priority(2)
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::super::{
        Selector, SelectorKind,
        state::{OptionItem, Selection},
        text_input::TextInput,
    };
    use super::*;
    use gpui::{Focusable, Render, TestAppContext};
    use std::{cell::Cell, collections::BTreeSet, rc::Rc};
    struct Host {
        popup: Entity<Overlay>,
        select: Entity<Selector>,
        input: Entity<TextInput>,
        outside: FocusHandle,
    }
    impl Render for Host {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .id("test-host")
                .relative()
                .size_full()
                .child(
                    Button::new("outside", "Extérieur")
                        .element()
                        .track_focus(&self.outside),
                )
                .child(Overlay::panel(
                    &self.popup,
                    div()
                        .flex()
                        .flex_col()
                        .child(self.input.clone())
                        .child(self.select.clone()),
                    window,
                    cx,
                ))
                .on_key_down(|event, window, cx| {
                    if event.keystroke.key == "tab" {
                        if event.keystroke.modifiers.shift {
                            window.focus_prev(cx);
                        } else {
                            window.focus_next(cx);
                        }
                        cx.stop_propagation();
                    }
                })
        }
    }
    #[gpui::test]
    fn nested_escape_refused_close_and_modal_focus(cx: &mut TestAppContext) {
        cx.update(TextInput::register);
        let (host, cx) = cx.add_window_view(|window, cx| {
            let popup = cx.new(|cx| Overlay::new(OverlayKind::Popup { modal: true }, cx));
            let input = cx.new(|cx| TextInput::text("Champ", cx));
            let select = cx.new(|cx| {
                Selector::new(
                    SelectorKind::Select,
                    "Select",
                    Selection::new(
                        vec![OptionItem::new("a", "A"), OptionItem::new("b", "B")],
                        BTreeSet::new(),
                        0,
                        1,
                    )
                    .unwrap(),
                    cx,
                )
                .unwrap()
            });
            let outside = cx.focus_handle().tab_stop(true);
            window.focus(&outside, cx);
            Host {
                popup,
                input,
                select,
                outside,
            }
        });
        let popup = cx.update(|_, cx| host.read(cx).popup.clone());
        let refused = Rc::new(Cell::new(true));
        let refusal = refused.clone();
        let close_requests = Rc::new(Cell::new(0));
        let requests = close_requests.clone();
        cx.update(|window, cx| {
            popup.update(cx, |popup, cx| {
                popup.on_close_requested(move |_, _| {
                    requests.set(requests.get() + 1);
                    if refusal.get() {
                        Err("Refus".into())
                    } else {
                        Ok(())
                    }
                });
                popup.show(window, cx);
            })
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, cx| {
            window.simulate_next_frame(cx);
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            assert!(
                host.read(cx)
                    .input
                    .read(cx)
                    .focus_handle(cx)
                    .is_focused(window)
            )
        });
        cx.simulate_keystrokes("tab");
        cx.update(|window, cx| {
            assert!(
                host.read(cx)
                    .select
                    .read(cx)
                    .focus_handle(cx)
                    .is_focused(window)
            )
        });
        cx.simulate_keystrokes("enter");
        assert_eq!(close_requests.get(), 0);
        cx.simulate_keystrokes("escape");
        assert_eq!(
            close_requests.get(),
            0,
            "the inner Select must consume the first Escape"
        );
        cx.update(|_, cx| assert!(popup.read(cx).open));
        cx.simulate_keystrokes("escape");
        assert_eq!(
            close_requests.get(),
            1,
            "the second Escape requests Popup closure"
        );
        cx.update(|_, cx| assert!(popup.read(cx).open));
        for _ in 0..10 {
            cx.simulate_keystrokes("tab");
        }
        cx.update(|window, cx| assert!(popup.read(cx).focus.contains_focused(window, cx)));
        for _ in 0..10 {
            cx.simulate_keystrokes("shift-tab");
        }
        cx.update(|window, cx| assert!(popup.read(cx).focus.contains_focused(window, cx)));
        refused.set(false);
        cx.simulate_keystrokes("escape");
        cx.update(|window, cx| {
            assert!(!popup.read(cx).open);
            assert!(host.read(cx).outside.is_focused(window));
        });
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod nested_pointer_tests {
    use super::super::{
        Selector, SelectorKind,
        state::{OptionItem, Selection},
    };
    use super::*;
    use gpui::{Focusable, Render, TestAppContext};
    use std::collections::BTreeSet;
    struct Host {
        popover: Entity<Overlay>,
        select: Entity<Selector>,
    }
    impl Render for Host {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().id("pointer-host").size_full().child(Overlay::panel(
                &self.popover,
                self.select.clone(),
                window,
                cx,
            ))
        }
    }
    #[gpui::test]
    fn clicking_a_child_panel_outside_parent_bounds_keeps_parent_open(cx: &mut TestAppContext) {
        let (host, cx) = cx.add_window_view(|_, cx| Host {
            popover: cx.new(|cx| Overlay::new(OverlayKind::Popover, cx)),
            select: cx.new(|cx| {
                Selector::new(
                    SelectorKind::Select,
                    "Choix",
                    Selection::new(
                        vec![OptionItem::new("a", "A"), OptionItem::new("b", "B")],
                        BTreeSet::new(),
                        0,
                        1,
                    )
                    .unwrap(),
                    cx,
                )
                .unwrap()
            }),
        });
        let (popover, select) =
            cx.update(|_, cx| (host.read(cx).popover.clone(), host.read(cx).select.clone()));
        cx.update(|window, cx| popover.update(cx, |popover, cx| popover.show(window, cx)));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, cx| {
            let focus = select.read(cx).focus_handle(cx);
            window.focus(&focus, cx);
        });
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let position = cx.update(|_, cx| {
            let registry = cx.global::<OverlayRegistry>();
            let parent = registry
                .layers
                .iter()
                .find(|layer| layer.id == popover.entity_id())
                .unwrap();
            let child = registry
                .layers
                .iter()
                .find(|layer| layer.id == select.entity_id())
                .unwrap();
            assert_eq!(child.parent, Some(parent.id));
            let child_bounds = child.bounds.unwrap();
            let position = gpui::point(child_bounds.center().x, child_bounds.bottom() - px(12.));
            assert!(!parent.bounds.unwrap().contains(&position));
            assert!(contains(parent.id, position, cx));
            position
        });
        cx.simulate_click(position, gpui::Modifiers::default());
        cx.update(|_, cx| assert!(popover.read(cx).open));
    }
}
