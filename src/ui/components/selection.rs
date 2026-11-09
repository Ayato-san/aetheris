use super::{
    SegmentGroup,
    state::{Invalid, Selection, Validation},
};
use crate::ui::theme::{self, tokens as t};
use gpui::{
    App, Context, EventEmitter, FocusHandle, Render, SharedString, Window, div, prelude::*, px, rgb,
};
use std::collections::BTreeSet;

type SelectionAcceptance = dyn Fn(&BTreeSet<String>, &mut App) -> Validation<()>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectorKind {
    Segment,
    Multi,
    Select,
}
pub(crate) struct SelectionChanged(pub BTreeSet<String>);
pub(crate) struct Selector {
    state: Selection,
    kind: SelectorKind,
    label: SharedString,
    focus: FocusHandle,
    focused: usize,
    open: bool,
    disabled: bool,
    error: Option<String>,
    trigger_bounds: Option<gpui::Bounds<gpui::Pixels>>,
    scroll: super::ScrollArea,
    option_rows: super::VirtualRows,
    blur: Option<gpui::Subscription>,
    on_change_requested: Box<SelectionAcceptance>,
}
impl EventEmitter<SelectionChanged> for Selector {}
impl Selector {
    pub fn new(
        kind: SelectorKind,
        label: impl Into<SharedString>,
        state: Selection,
        cx: &mut Context<Self>,
    ) -> Validation<Self> {
        Selection::new(
            state.options.clone(),
            state.selected.clone(),
            state.min,
            state.max,
        )?;
        if kind == SelectorKind::Segment && (state.options.len() < 2 || state.max != 1) {
            return Err(
                "Les segments nécessitent au moins deux options et un seul choix possible".into(),
            );
        }
        let focused = if kind == SelectorKind::Segment {
            state
                .options
                .iter()
                .position(|option| state.selected.contains(&option.id))
        } else {
            state.options.iter().position(|option| !option.disabled)
        }
        .unwrap_or(0);
        Ok(Self {
            state,
            kind,
            label: label.into(),
            focus: cx.focus_handle().tab_stop(true),
            focused,
            open: false,
            disabled: false,
            error: None,
            trigger_bounds: None,
            scroll: super::ScrollArea::default(),
            option_rows: super::VirtualRows::default(),
            blur: None,
            on_change_requested: Box::new(|_, _| Ok(())),
        })
    }
    pub fn on_change_requested(
        &mut self,
        callback: impl Fn(&BTreeSet<String>, &mut App) -> Validation<()> + 'static,
    ) {
        self.on_change_requested = Box::new(callback);
    }
    pub fn configure(&mut self, next: Selection, cx: &mut Context<Self>) -> Validation<()> {
        if self.kind == SelectorKind::Segment && (next.options.len() < 2 || next.max != 1) {
            return Err(Invalid::from("Configuration des segments invalide"));
        }
        let focused_id = self
            .state
            .options
            .get(self.focused)
            .map(|option| option.id.clone());
        self.state
            .configure(next.options, next.selected, next.min, next.max)?;
        self.focused = focused_id
            .and_then(|id| self.state.options.iter().position(|option| option.id == id))
            .or_else(|| {
                self.state
                    .options
                    .iter()
                    .position(|option| !option.disabled)
            })
            .unwrap_or(0);
        self.error = None;
        cx.notify();
        Ok(())
    }
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        self.focus = self.focus.clone().tab_stop(!disabled);
        if disabled {
            self.close(cx);
        }
        cx.notify();
    }
    fn show(&mut self, window: &Window, cx: &mut Context<Self>) {
        super::overlay::register(cx.entity_id(), &self.focus, window, cx);
        self.open = true;
        cx.notify();
    }
    fn close(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        super::overlay::unregister(cx.entity_id(), cx);
        cx.notify();
    }
    fn choose(&mut self, index: usize, select_only: bool, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let Some(option) = self.state.options.get(index) else {
            return;
        };
        let candidate = if (select_only || self.kind == SelectorKind::Segment)
            && matches!(self.kind, SelectorKind::Select | SelectorKind::Segment)
            && self.state.max == 1
            && !option.disabled
        {
            Ok(BTreeSet::from([option.id.clone()]))
        } else {
            self.state.candidate(&option.id)
        };
        let result = candidate.and_then(|next| {
            if next != self.state.selected {
                (self.on_change_requested)(&next, cx)?;
            }
            Ok(next)
        });
        match result {
            Ok(next) => {
                if next != self.state.selected {
                    self.state.selected = next.clone();
                    cx.emit(SelectionChanged(next));
                }
                self.error = None;
                if self.kind == SelectorKind::Select && self.state.max == 1 {
                    self.close(cx);
                }
            }
            Err(error) => self.error = Some(error.0),
        }
        cx.notify();
    }
    fn keyboard(&mut self, key: &str, cx: &mut Context<Self>) {
        let enabled: Vec<_> = self
            .state
            .options
            .iter()
            .enumerate()
            .filter(|(_, o)| !o.disabled)
            .map(|(i, _)| i)
            .collect();
        if enabled.is_empty() || self.disabled {
            return;
        }
        let current = enabled.iter().position(|i| *i == self.focused).unwrap_or(0);
        let target = match key {
            "home" => enabled[0],
            "end" => *enabled.last().unwrap(),
            "left" | "up" => enabled[current.saturating_sub(1)],
            "right" | "down" => enabled[(current + 1).min(enabled.len() - 1)],
            "space" | "enter" => {
                self.choose(self.focused, true, cx);
                return;
            }
            _ => return,
        };
        self.focused = target;
        if self.kind == SelectorKind::Segment {
            self.scroll.handle.scroll_to_item(target);
        } else {
            self.option_rows
                .scroll_to_item(target, gpui::ScrollStrategy::Nearest);
        }
        if self.kind == SelectorKind::Segment {
            self.choose(target, true, cx);
        }
        cx.notify();
    }
}
impl Selector {
    fn option(&self, i: usize, window: &Window, cx: &Context<Self>) -> gpui::Stateful<gpui::Div> {
        let option = &self.state.options[i];
        let selected = self.state.selected.contains(&option.id);
        let at_max = !selected && self.state.selected.len() >= self.state.max && self.state.max > 1;
        div()
            .id(option.id.clone())
            .flex()
            .items_center()
            .h(px(t::TOKEN_COMPONENTS_SELECT_ROW_HEIGHT))
            .flex_shrink_0()
            .px(px(t::TOKEN_COMPONENTS_SELECT_PADDING_X))
            .text_color(rgb(t::TOKEN_COMPONENTS_SELECT_TEXT))
            .child(option.label.clone())
            .when(!(self.disabled || option.disabled || at_max), |d| {
                d.cursor_pointer().when(!selected, |d| {
                    d.hover(|s| s.bg(rgb(t::TOKEN_COMPONENTS_SELECT_HOVER)))
                })
            })
            .when(self.disabled || option.disabled || at_max, |d| {
                d.opacity(0.45)
                    .capture_any_mouse_down(|_, _, cx| cx.stop_propagation())
                    .on_click(|_, _, cx| cx.stop_propagation())
            })
            .tab_stop(false)
            .role(gpui::Role::ListBoxOption)
            .aria_selected(selected)
            .when(selected, |d| {
                d.bg(rgb(t::TOKEN_COMPONENTS_SELECT_SELECTED_BACKGROUND))
                    .text_color(rgb(t::TOKEN_COMPONENTS_SELECT_SELECTED_TEXT))
                    .font_weight(gpui::FontWeight(t::TOKEN_FONT_BOLD))
            })
            .when(at_max, |d| {
                d.aria_description("Nombre maximal de choix atteint")
            })
            .when(self.focused == i && self.focus.is_focused(window), |d| {
                d.when(!selected, |d| d.bg(rgb(t::TOKEN_COMPONENTS_SELECT_HOVER)))
                    .aria_active_descendant()
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                window.focus(&this.focus, cx);
                this.focused = i;
                this.choose(i, false, cx);
            }))
    }
}
impl Render for Selector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.open && !super::overlay::registered(cx.entity_id(), cx) {
            self.open = false;
        }
        if self.blur.is_none() {
            self.blur = Some(cx.on_focus_out(&self.focus, window, |this, _, _, cx| {
                this.close(cx);
            }));
        }
        let measure = cx.entity();
        let selected_label = self
            .state
            .options
            .iter()
            .filter(|o| self.state.selected.contains(&o.id))
            .map(|o| o.label.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let show_options = self.kind != SelectorKind::Select || self.open;
        let horizontal = self.kind == SelectorKind::Segment;
        div()
            .id("selector")
            .track_focus(&self.focus)
            .tab_stop(!self.disabled)
            .role(match self.kind {
                SelectorKind::Select => gpui::Role::ComboBox,
                SelectorKind::Segment => gpui::Role::RadioGroup,
                SelectorKind::Multi => gpui::Role::ListBox,
            })
            .aria_label(self.label.clone())
            .flex()
            .flex_col()
            .gap(px(t::TOKEN_COMPONENTS_FIELD_GAP))
            .font_family(theme::FONT_FAMILY)
            .font_weight(gpui::FontWeight(t::TOKEN_COMPONENTS_FIELD_WEIGHT))
            .text_size(px(theme::FONT_SIZE))
            .line_height(px(t::TOKEN_COMPONENTS_COMMON_LINE_HEIGHT))
            .text_color(rgb(theme::TEXT))
            .when(self.disabled, |d| {
                d.when(!horizontal, |d| d.opacity(0.45))
                    .aria_description("Indisponible")
            })
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if this.disabled {
                    return;
                }
                let key = event.keystroke.key.as_str();
                if key == "escape" && this.open {
                    this.close(cx);
                    window.focus(&this.focus, cx);
                    cx.notify();
                    cx.stop_propagation();
                } else if [
                    "left", "right", "up", "down", "home", "end", "space", "enter",
                ]
                .contains(&key)
                {
                    if this.kind == SelectorKind::Select && !this.open {
                        this.show(window, cx);
                    } else {
                        this.keyboard(key, cx);
                    }
                    cx.stop_propagation();
                }
            }))
            .when(!self.label.is_empty(), |d| {
                d.child(div().text_sm().child(self.label.clone()))
            })
            .when(self.kind == SelectorKind::Select, |d| {
                d.child(
                    div()
                        .id("select-trigger")
                        .flex()
                        .items_center()
                        .h(px(t::TOKEN_COMPONENTS_FIELD_HEIGHT))
                        .w(px(t::TOKEN_COMPONENTS_FIELD_WIDTH))
                        .max_w_full()
                        .px(px(t::TOKEN_COMPONENTS_FIELD_PADDING_X))
                        .py(px(t::TOKEN_COMPONENTS_FIELD_PADDING_Y))
                        .rounded(px(theme::RADIUS))
                        .bg(rgb(t::TOKEN_COMPONENTS_FIELD_BACKGROUND))
                        .border(px(
                            if self.error.is_none() && self.focus.is_focused(window) {
                                t::TOKEN_COMPONENTS_COMMON_FOCUS_WIDTH
                            } else {
                                theme::BORDER_WIDTH
                            },
                        ))
                        .border_color(rgb(if self.error.is_some() {
                            t::TOKEN_COMPONENTS_FIELD_ERROR
                        } else if self.focus.is_focused(window) {
                            t::TOKEN_COMPONENTS_COMMON_FOCUS_COLOR
                        } else {
                            t::TOKEN_COMPONENTS_FIELD_BORDER
                        }))
                        .when(selected_label.is_empty(), |d| {
                            d.text_color(rgb(t::TOKEN_COMPONENTS_FIELD_PLACEHOLDER))
                        })
                        .child(if selected_label.is_empty() {
                            "Choisir…".to_owned()
                        } else {
                            selected_label.clone()
                        })
                        .relative()
                        .tab_stop(false)
                        .aria_expanded(self.open)
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
                        .on_click(cx.listener(|this, _, window, cx| {
                            if !this.disabled {
                                if this.open {
                                    this.close(cx);
                                } else {
                                    this.show(window, cx);
                                }
                                window.focus(&this.focus, cx);
                                cx.notify();
                            }
                        }))
                        // The selector root may stretch across the page. Only the
                        // trigger and floating panel count as inside the Select.
                        .when(self.open, |d| {
                            d.on_mouse_down_out(cx.listener(
                                |this, event: &gpui::MouseDownEvent, _, cx| {
                                    if !super::overlay::contains(cx.entity_id(), event.position, cx)
                                    {
                                        this.close(cx);
                                    }
                                },
                            ))
                        }),
                )
            })
            .when(show_options, |d| {
                if horizontal {
                    return d.child(
                        SegmentGroup::element("segment-group")
                            .track_scroll(&self.scroll.handle)
                            .children(self.state.options.iter().enumerate().map(
                                |(index, option)| {
                                    SegmentGroup::segment(
                                        option.id.clone(),
                                        option.label.clone(),
                                        self.state.selected.contains(&option.id),
                                        self.disabled || option.disabled,
                                    )
                                    .when(
                                        self.focused == index && self.focus.is_focused(window),
                                        |d| d.aria_active_descendant(),
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, window, cx| {
                                            window.focus(&this.focus, cx);
                                            this.focused = index;
                                            this.choose(index, true, cx);
                                        },
                                    ))
                                },
                            )),
                    );
                }
                let row_height =
                    t::TOKEN_COMPONENTS_SELECT_ROW_HEIGHT + t::TOKEN_COMPONENTS_SELECT_GAP;
                let total = self.state.options.len() as f32 * row_height
                    - t::TOKEN_COMPONENTS_SELECT_GAP
                    + t::TOKEN_COMPONENTS_SELECT_PADDING_Y * 2.;
                let height = px(total.max(0.)).min(
                    (window.viewport_size().height - px(theme::OVERLAY_MARGIN * 2.))
                        .min(px(t::TOKEN_COMPONENTS_SELECT_MAX_HEIGHT)),
                );
                let owner = cx.entity();
                let options = self
                    .option_rows
                    .element(
                        "selector-options",
                        self.state.options.len(),
                        row_height,
                        move |range, window, cx| {
                            owner.update(cx, |this, cx| {
                                range
                                    .map(|i| this.option(i, window, cx).into_any_element())
                                    .collect()
                            })
                        },
                    )
                    .h(height)
                    .w(px(t::TOKEN_COMPONENTS_FIELD_WIDTH))
                    .max_w(window.viewport_size().width - px(theme::OVERLAY_MARGIN * 2.))
                    .pt(px(t::TOKEN_COMPONENTS_SELECT_PADDING_Y))
                    .pb(px((t::TOKEN_COMPONENTS_SELECT_PADDING_Y
                        - t::TOKEN_COMPONENTS_SELECT_GAP)
                        .max(0.)))
                    .rounded(px(theme::RADIUS))
                    .bg(rgb(theme::PANEL_MUTED));
                if self.kind == SelectorKind::Select {
                    d.child(super::overlay::floating_at(
                        options,
                        super::overlay::Placement::Bottom,
                        self.trigger_bounds,
                        cx.entity_id(),
                    ))
                } else {
                    d.child(options)
                }
            })
            .when_some(self.error.clone(), |d, error| {
                d.child(
                    div()
                        .text_size(px(theme::FONT_SIZE))
                        .font_weight(gpui::FontWeight(t::TOKEN_FONT_MEDIUM))
                        .text_color(rgb(theme::DANGER))
                        .child(error),
                )
            })
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::super::state::OptionItem;
    use super::*;
    use gpui::TestAppContext;
    #[gpui::test]
    fn virtual_select_reveals_and_selects_the_last_of_a_thousand_options(cx: &mut TestAppContext) {
        let (select, cx) = cx.add_window_view(|window, cx| {
            let control = Selector::new(
                SelectorKind::Select,
                "Longue liste",
                Selection::new(
                    (0..1000)
                        .map(|i| OptionItem::new(format!("option-{i}"), format!("Option {i}")))
                        .collect(),
                    BTreeSet::new(),
                    0,
                    1,
                )
                .unwrap(),
                cx,
            )
            .unwrap();
            window.focus(&control.focus, cx);
            control
        });
        cx.simulate_keystrokes("space end");
        let target = cx.update(|_, cx| {
            assert_eq!(select.read(cx).focused, 999);
            let bounds = select.read(cx).option_rows.bounds();
            gpui::point(bounds.center().x, bounds.bottom() - px(16.))
        });
        cx.simulate_mouse_move(target, None, gpui::Modifiers::default());
        cx.simulate_mouse_down(target, gpui::MouseButton::Left, gpui::Modifiers::default());
        cx.simulate_mouse_up(target, gpui::MouseButton::Left, gpui::Modifiers::default());
        cx.update(|_, cx| {
            assert_eq!(
                select.read(cx).state.selected,
                BTreeSet::from(["option-999".into()])
            );
            assert!(!select.read(cx).open);
        });
    }
    #[gpui::test]
    fn select_closes_beside_trigger_but_keeps_inside_clicks(cx: &mut TestAppContext) {
        struct Host {
            select: gpui::Entity<Selector>,
        }
        impl Render for Host {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
                    .w(px(600.))
                    .h(px(500.))
                    .flex()
                    .flex_col()
                    .child(self.select.clone())
            }
        }
        let (host, cx) = cx.add_window_view(|_, cx| Host {
            select: cx.new(|cx| {
                Selector::new(
                    SelectorKind::Select,
                    "Choix",
                    Selection::new(
                        vec![
                            OptionItem::new("a", "Premier"),
                            OptionItem::new("b", "Deuxième"),
                        ],
                        BTreeSet::new(),
                        0,
                        2,
                    )
                    .unwrap(),
                    cx,
                )
                .unwrap()
            }),
        });
        let select = cx.update(|_, cx| host.read(cx).select.clone());
        let trigger = cx.update(|_, cx| select.read(cx).trigger_bounds.unwrap());
        let click = |cx: &mut gpui::VisualTestContext, position| {
            cx.simulate_mouse_move(position, None, gpui::Modifiers::default());
            cx.simulate_mouse_down(
                position,
                gpui::MouseButton::Left,
                gpui::Modifiers::default(),
            );
            cx.simulate_mouse_up(
                position,
                gpui::MouseButton::Left,
                gpui::Modifiers::default(),
            );
        };
        click(cx, trigger.center());
        cx.update(|_, cx| assert!(select.read(cx).open));
        let option = cx.update(|_, cx| {
            let bounds = select.read(cx).option_rows.bounds();
            assert_eq!(bounds.size.height, px(64.));
            gpui::point(bounds.left() + px(30.), bounds.top() + px(20.))
        });
        click(cx, option);
        cx.update(|_, cx| {
            assert!(select.read(cx).open);
            assert_eq!(select.read(cx).state.selected, BTreeSet::from(["a".into()]));
        });
        click(
            cx,
            gpui::point(trigger.right() + px(30.), trigger.center().y),
        );
        cx.update(|_, cx| {
            assert!(!select.read(cx).open);
            assert!(!super::super::overlay::registered(select.entity_id(), cx));
            assert_eq!(select.read(cx).state.selected, BTreeSet::from(["a".into()]));
        });
        click(cx, trigger.center());
        cx.update(|_, cx| assert!(select.read(cx).open));
        click(cx, trigger.center());
        cx.update(|_, cx| assert!(!select.read(cx).open));
    }
    #[gpui::test]
    fn segment_group_preserves_unknown_value_until_a_choice_is_made(cx: &mut TestAppContext) {
        let group = cx.new(|cx| {
            let state = Selection::new(
                vec![
                    OptionItem::new("dlss", "DLSS"),
                    OptionItem::new("fsr", "FSR"),
                ],
                BTreeSet::new(),
                0,
                1,
            )
            .unwrap();
            Selector::new(SelectorKind::Segment, "Profil", state, cx).unwrap()
        });
        group.update(cx, |group, cx| {
            assert!(group.state.selected.is_empty());
            group.choose(1, true, cx);
            assert_eq!(group.state.selected, BTreeSet::from(["fsr".into()]));
            // Clicking or pressing Space again must keep the selected profile.
            group.choose(1, true, cx);
            group.choose(1, false, cx);
            assert_eq!(group.state.selected, BTreeSet::from(["fsr".into()]));
            let mut next = group.state.clone();
            next.selected.clear();
            group.configure(next, cx).unwrap();
            assert!(group.state.selected.is_empty());
        });
    }
    #[gpui::test]
    fn segment_group_arrows_start_at_selection_and_skip_disabled(cx: &mut TestAppContext) {
        cx.update(|cx| crate::ui::theme::register_fonts(cx));
        let (group, cx) = cx.add_window_view(|window, cx| {
            let mut disabled = OptionItem::new("b", "Option 2");
            disabled.disabled = true;
            let state = Selection::new(
                vec![
                    OptionItem::new("a", "Option 1"),
                    disabled,
                    OptionItem::new("c", "Option 3"),
                    OptionItem::new("d", "Option 4"),
                ],
                BTreeSet::from(["c".into()]),
                1,
                1,
            )
            .unwrap();
            let control = Selector::new(SelectorKind::Segment, "", state, cx).unwrap();
            window.focus(&control.focus, cx);
            control
        });
        cx.simulate_keystrokes("left");
        cx.update(|_, cx| assert_eq!(group.read(cx).state.selected, BTreeSet::from(["a".into()])));
        cx.simulate_keystrokes("right");
        cx.update(|_, cx| assert_eq!(group.read(cx).state.selected, BTreeSet::from(["c".into()])));
        cx.simulate_keystrokes("end space");
        cx.update(|_, cx| {
            assert_eq!(group.read(cx).state.selected, BTreeSet::from(["d".into()]));
            group.update(cx, |group, cx| group.set_disabled(true, cx));
        });
        cx.simulate_keystrokes("home");
        cx.update(|_, cx| assert_eq!(group.read(cx).state.selected, BTreeSet::from(["d".into()])));
    }
    #[gpui::test]
    fn keyboard_skips_disabled_options_and_rejection_preserves_selection(cx: &mut TestAppContext) {
        let (selector, cx) = cx.add_window_view(|window, cx| {
            let mut disabled = OptionItem::new("b", "Same");
            disabled.disabled = true;
            let state = Selection::new(
                vec![
                    OptionItem::new("a", "Same"),
                    disabled,
                    OptionItem::new("c", "Same"),
                ],
                BTreeSet::new(),
                0,
                2,
            )
            .unwrap();
            let control = Selector::new(SelectorKind::Multi, "Choix", state, cx).unwrap();
            window.focus(&control.focus, cx);
            control
        });
        cx.simulate_keystrokes("down space");
        cx.update(|_, cx| {
            assert_eq!(
                selector.read(cx).state.selected,
                BTreeSet::from(["c".into()])
            );
            selector.update(cx, |control, _| {
                control.on_change_requested(|_, _| Err("Refus".into()))
            });
        });
        cx.simulate_keystrokes("home space");
        cx.update(|_, cx| {
            assert_eq!(
                selector.read(cx).state.selected,
                BTreeSet::from(["c".into()])
            )
        });
        cx.update(|_, cx| {
            selector.update(cx, |control, cx| {
                let mut next = control.state.clone();
                next.options.reverse();
                control.configure(next, cx).unwrap();
                assert_eq!(control.state.options[control.focused].id, "a");
                control.set_disabled(true, cx);
                let before = control.state.selected.clone();
                control.choose(0, false, cx);
                assert_eq!(control.state.selected, before);
            })
        });
    }
}

impl gpui::Focusable for Selector {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
