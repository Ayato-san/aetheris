use super::{Button, VirtualRows};
use crate::ui::theme::{self, tokens as t};
use gpui::{
    AnyElement, App, Div, ElementId, SharedString, Stateful, Window, div, prelude::*, px, rgb,
};
use std::{ops::Range, rc::Rc};

#[derive(Clone)]
pub(crate) enum ColumnWidth {
    Fixed(f32),
    Flexible { min: f32 },
}
#[derive(Clone, Copy, Default)]
pub(crate) enum ColumnAlign {
    #[default]
    Left,
    Center,
    Right,
}
#[derive(Clone)]
pub(crate) struct Column {
    pub id: SharedString,
    pub title: SharedString,
    pub width: ColumnWidth,
    pub align: ColumnAlign,
    pub hidden: bool,
    pub sortable: bool,
}
impl Column {
    pub fn new(
        id: impl Into<SharedString>,
        title: impl Into<SharedString>,
        width: ColumnWidth,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            width,
            align: ColumnAlign::Left,
            hidden: false,
            sortable: false,
        }
    }
    pub fn align(mut self, align: ColumnAlign) -> Self {
        self.align = align;
        self
    }
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }
}
#[derive(Clone, Copy)]
pub(crate) enum SortDirection {
    Ascending,
    Descending,
}
#[derive(Clone)]
pub(crate) struct SortRequest {
    pub column: SharedString,
    pub direction: SortDirection,
}
pub(crate) struct Table {
    columns: Vec<Column>,
    max_height: f32,
}
impl Table {
    pub fn new(columns: Vec<Column>) -> super::state::Validation<Self> {
        let mut ids = std::collections::BTreeSet::new();
        for column in &columns {
            let width = match column.width {
                ColumnWidth::Fixed(v) | ColumnWidth::Flexible { min: v } => v,
            };
            if !ids.insert(column.id.clone()) || !width.is_finite() || width <= 0. {
                return Err("Colonnes invalides".into());
            }
        }
        Ok(Self {
            columns,
            max_height: theme::TABLE_BODY_HEIGHT,
        })
    }
    pub fn max_height(mut self, height: f32) -> super::state::Validation<Self> {
        if !height.is_finite() || height <= 0. {
            return Err("Hauteur invalide".into());
        }
        self.max_height = height;
        Ok(self)
    }
    fn minimum_width(&self) -> f32 {
        self.columns
            .iter()
            .filter(|column| !column.hidden)
            .map(|column| match column.width {
                ColumnWidth::Fixed(v) | ColumnWidth::Flexible { min: v } => v,
            })
            .sum()
    }
    /// Page supplies both sorting and current sort; no data is reordered here.
    pub fn header(
        &self,
        sort: Option<SortRequest>,
        on_sort: impl Fn(SortRequest, &mut Window, &mut App) + 'static,
    ) -> Div {
        let callback = Rc::new(on_sort);
        self.row(
            self.columns
                .iter()
                .map(|column| {
                    if !column.sortable {
                        return div()
                            .text_size(px(t::TOKEN_XS))
                            .text_color(rgb(theme::TEXT_MUTED))
                            .child(column.title.clone())
                            .into_any_element();
                    }
                    let callback = callback.clone();
                    let id = column.id.clone();
                    let current = sort
                        .as_ref()
                        .filter(|s| s.column == id)
                        .map(|s| s.direction);
                    Button::new(
                        column.id.clone(),
                        format!(
                            "{} {}",
                            column.title,
                            match current {
                                Some(SortDirection::Ascending) => "↑",
                                Some(SortDirection::Descending) => "↓",
                                None => "",
                            }
                        ),
                    )
                    .element()
                    .h_auto()
                    .min_w_0()
                    .p_0()
                    .bg(gpui::transparent_black())
                    .border_0()
                    .text_size(px(t::TOKEN_XS))
                    .text_color(rgb(theme::TEXT))
                    .on_click(move |_, window, cx| {
                        callback(
                            SortRequest {
                                column: id.clone(),
                                direction: if matches!(current, Some(SortDirection::Ascending)) {
                                    SortDirection::Descending
                                } else {
                                    SortDirection::Ascending
                                },
                            },
                            window,
                            cx,
                        )
                    })
                    .into_any_element()
                })
                .collect(),
        )
    }
    pub fn row(&self, cells: Vec<AnyElement>) -> Div {
        div()
            .flex()
            .flex_shrink_0()
            .border_b_1()
            .border_color(rgb(t::TOKEN_COMPONENTS_TABLE_BORDER))
            .font_family(theme::FONT_FAMILY)
            .text_size(px(t::TOKEN_XS))
            .line_height(px(theme::controls::LINE_HEIGHT))
            .font_weight(gpui::FontWeight(t::TOKEN_FONT_MEDIUM))
            .text_color(rgb(theme::TEXT_MUTED))
            .min_h(px(t::TOKEN_COMPONENTS_TABLE_ROW_HEIGHT))
            .min_w(px(self.minimum_width()))
            .children(
                self.columns
                    .iter()
                    .zip(cells)
                    .filter(|(column, _)| !column.hidden)
                    .map(|(column, cell)| {
                        div()
                            .px(px(t::TOKEN_COMPONENTS_TABLE_CELL_PADDING * 2.))
                            .py_0()
                            .when(matches!(column.width, ColumnWidth::Flexible { .. }), |d| {
                                d.flex_1()
                            })
                            .map(|d| match column.width {
                                ColumnWidth::Fixed(v) => d.w(px(v)).flex_shrink_0(),
                                ColumnWidth::Flexible { min } => d.min_w(px(min)),
                            })
                            .flex()
                            .items_center()
                            .map(|d| match column.align {
                                ColumnAlign::Left => d.justify_start(),
                                ColumnAlign::Center => d.justify_center(),
                                ColumnAlign::Right => d.justify_end(),
                            })
                            .child(cell)
                    }),
            )
    }
    pub fn ordinary(
        &self,
        id: impl Into<ElementId>,
        header: Option<AnyElement>,
        rows: Vec<AnyElement>,
        status: Option<AnyElement>,
    ) -> Stateful<Div> {
        let overflowing = Rc::new(std::cell::Cell::new(false));
        let measured_overflow = overflowing.clone();
        let max_height = px(self.max_height);
        div()
            .id(id)
            .bg(rgb(t::TOKEN_COMPONENTS_TABLE_BACKGROUND))
            .border_1()
            .border_color(rgb(t::TOKEN_COMPONENTS_TABLE_BORDER))
            .rounded(px(theme::RADIUS))
            .overflow_x_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t::TOKEN_COMPONENTS_TABLE_HEADER_GAP))
                    .min_w(px(self.minimum_width()))
                    .when_some(header, |d, h| d.child(h))
                    .child(
                        div()
                            .on_children_prepainted(move |bounds, _, _| {
                                let top = bounds.iter().map(|b| b.top()).min().unwrap_or_default();
                                let bottom =
                                    bounds.iter().map(|b| b.bottom()).max().unwrap_or_default();
                                measured_overflow.set(bottom - top > max_height);
                            })
                            .id("table-body")
                            .flex()
                            .flex_col()
                            .gap(px(t::TOKEN_COMPONENTS_TABLE_BODY_GAP))
                            .overflow_y_scroll()
                            .on_scroll_wheel(move |event, _, cx| {
                                if event.delta.pixel_delta(px(20.)).y != px(0.) && overflowing.get()
                                {
                                    cx.stop_propagation();
                                }
                            })
                            .max_h(px(self.max_height))
                            .children(rows)
                            .when_some(status, |d, s| d.child(s)),
                    ),
            )
    }
    pub fn virtualized(
        &self,
        id: impl Into<ElementId>,
        count: usize,
        row_height: f32,
        scroll: &VirtualRows,
        header: Option<AnyElement>,
        render: impl Fn(Range<usize>, &mut Window, &mut App) -> Vec<AnyElement> + 'static,
    ) -> super::state::Validation<Stateful<Div>> {
        if !row_height.is_finite() || row_height <= 0. {
            return Err("Hauteur de ligne invalide".into());
        }
        let id = id.into();
        let list = scroll
            .element((id.clone(), "body"), count, row_height, render)
            .h(px(self.max_height));
        Ok(div()
            .id(id)
            .bg(rgb(t::TOKEN_COMPONENTS_TABLE_BACKGROUND))
            .border_1()
            .border_color(rgb(t::TOKEN_COMPONENTS_TABLE_BORDER))
            .rounded(px(theme::RADIUS))
            .overflow_x_scroll()
            .child(
                div()
                    .min_w(px(self.minimum_width()))
                    .flex()
                    .flex_col()
                    .when_some(header, |d, h| d.child(h))
                    .child(list),
            ))
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::super::text_input::TextInput;
    use super::*;
    use gpui::{Context, Entity, Focusable, Render, ScrollStrategy, TestAppContext};
    use std::cell::Cell;
    struct NestedTableHost {
        page: super::super::ScrollArea,
        visible_row_top: Rc<Cell<gpui::Pixels>>,
    }
    impl Render for NestedTableHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let marker = self.visible_row_top.clone();
            let table = Table::new(vec![Column::new(
                "value",
                "Valeur",
                ColumnWidth::Fixed(200.),
            )])
            .unwrap()
            .max_height(150.)
            .unwrap()
            .ordinary(
                "nested-table",
                None,
                (0..50)
                    .map(|index| {
                        let marker = marker.clone();
                        div()
                            .h(px(30.))
                            .flex_shrink_0()
                            .when(index == 3, |d| {
                                d.child(
                                    gpui::canvas(
                                        move |bounds, _, _| marker.set(bounds.top()),
                                        |_, _, _, _| {},
                                    )
                                    .size_full(),
                                )
                            })
                            .into_any_element()
                    })
                    .collect(),
                None,
            );
            self.page
                .wrap(
                    "outer-page",
                    div()
                        .id("page-content")
                        .h_full()
                        .overflow_y_scroll()
                        .flex()
                        .flex_col()
                        .child(table.flex_shrink_0())
                        .child(div().h(px(800.)).flex_shrink_0()),
                )
                .w(px(400.))
                .h(px(300.))
        }
    }

    #[gpui::test]
    fn ordinary_table_scroll_does_not_move_enclosing_page(cx: &mut TestAppContext) {
        let (host, cx) = cx.add_window_view(|_, _| NestedTableHost {
            page: super::super::ScrollArea::default(),
            visible_row_top: Rc::new(Cell::new(px(0.))),
        });
        let (position, before) = cx.update(|_, cx| {
            let host = host.read(cx);
            let bounds = host.page.handle.bounds();
            (
                gpui::point(bounds.left() + px(50.), bounds.top() + px(50.)),
                host.visible_row_top.get(),
            )
        });
        cx.simulate_event(gpui::ScrollWheelEvent {
            position,
            delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), px(-60.))),
            modifiers: gpui::Modifiers::default(),
            touch_phase: gpui::TouchPhase::Moved,
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            let host = host.read(cx);
            assert!(host.visible_row_top.get() < before);
            assert_eq!(host.page.handle.offset().y, px(0.));
        });
    }

    struct Host {
        input: Entity<TextInput>,
        scroll: VirtualRows,
        reversed: bool,
        rendered: Rc<Cell<usize>>,
    }
    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let input = self.input.clone();
            let reversed = self.reversed;
            let rendered = self.rendered.clone();
            Table::new(vec![Column::new(
                "value",
                "Valeur",
                ColumnWidth::Flexible { min: 600. },
            )])
            .unwrap()
            .virtualized(
                "test-table",
                10_000,
                40.,
                &self.scroll,
                None,
                move |range, _, _| {
                    rendered.set(rendered.get() + range.len());
                    range
                        .map(|index| {
                            let id = if reversed { 9999 - index } else { index };
                            div()
                                .id(("row", id))
                                .child(if id == 0 {
                                    input.clone().into_any_element()
                                } else {
                                    div().child(id.to_string()).into_any_element()
                                })
                                .into_any_element()
                        })
                        .collect()
                },
            )
            .unwrap()
        }
    }
    #[gpui::test]
    fn virtualized_cells_keep_page_owned_state_across_scroll_and_sort(cx: &mut TestAppContext) {
        let (host, cx) = cx.add_window_view(|_, cx| Host {
            input: cx.new(|cx| TextInput::text("Cellule", cx)),
            scroll: VirtualRows::default(),
            reversed: false,
            rendered: Rc::new(Cell::new(0)),
        });
        let input = cx.update(|window, cx| {
            assert!(host.read(cx).rendered.get() < 1000);
            let input = host.read(cx).input.clone();
            let focus = input.read(cx).focus_handle(cx);
            window.focus(&focus, cx);
            input
        });
        cx.simulate_input("brouillon de cellule");
        cx.update(|_, cx| {
            host.update(cx, |host, cx| {
                host.scroll.scroll_to_item(9999, ScrollStrategy::Top);
                cx.notify();
            })
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|_, cx| {
            host.update(cx, |host, cx| {
                host.reversed = true;
                cx.notify();
            })
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|_, cx| assert_eq!(input.read(cx).content.as_ref(), "brouillon de cellule"));
    }
}
