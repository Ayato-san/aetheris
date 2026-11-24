//! Development-only, read-only view of the generated game catalogue.
use crate::ui::{
    components::{
        Button, Slider, SliderChanged, VirtualRows,
        state::SliderState,
        text_input::{TextInput, ValueChanged},
    },
    theme::{ACCENT, DANGER, PANEL, PANEL_MUTED, TEXT_MUTED},
};
use aetheris::game_data::{
    self,
    debug_browser::{self, Entry, Family, PAGE_SIZE},
};
use gpui::{
    ClipboardItem, Context, Entity, PathPromptOptions, Render, ScrollStrategy, Subscription,
    Window, div, prelude::*, px, rgb,
};
use std::{path::PathBuf, sync::Arc};

pub(crate) struct GameDataDebugPage {
    item: Option<super::game_data_item::ItemPresentation>,
    definition: Option<super::game_data_definition::DefinitionPresentation>,
    english: bool,
    item_slider: Entity<Slider>,
    _slider_subscription: Subscription,
    path: Option<PathBuf>,
    family: Family,
    search: Entity<TextInput>,
    _search_subscription: Subscription,
    query: String,
    offset: usize,
    total: usize,
    entries: Arc<Vec<Entry>>,
    selected: Option<i64>,
    summary: String,
    coverage: Vec<String>,
    details: Arc<Vec<String>>,
    detail_rows: Arc<Vec<String>>,
    sections: Vec<(String, usize)>,
    list_viewport: VirtualRows,
    detail_viewport: VirtualRows,
    show_coverage: bool,
    sources: bool,
    busy: bool,
    detail_busy: bool,
    error: Option<String>,
    load_token: u64,
    detail_token: u64,
}
impl GameDataDebugPage {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let path = game_data::database_path();
        let error = path.as_ref().err().map(ToString::to_string);
        let search = cx.new(|cx| TextInput::text("Nom anglais, nom localisé ou ID", cx));
        let subscription = cx.subscribe(&search, |page, _, _: &ValueChanged, cx| {
            page.apply_search(cx);
        });
        let item_slider = cx.new(|cx| {
            Slider::new(
                "Niveau",
                SliderState {
                    min: 0.,
                    max: 1.,
                    value: 0.,
                    steps: vec![0., 1.],
                    snap: true,
                    keyboard_increment: 1.,
                },
                cx,
            )
            .expect("valid default item slider")
            .compact()
            .hide_value()
        });
        let slider_subscription =
            cx.subscribe(&item_slider, |page, _, change: &SliderChanged, cx| {
                if let Some(item) = page.item.as_mut() {
                    item.enchant_level = change.0 as i64;
                    cx.notify();
                }
                if let Some(definition) = page.definition.as_mut() {
                    definition.level = change.0 as i64;
                    definition.remeasure();
                    cx.notify();
                }
            });
        Self {
            path: path.ok(),
            item: None,
            definition: None,
            english: false,
            item_slider,
            _slider_subscription: slider_subscription,
            error,
            family: Family::Items,
            search,
            _search_subscription: subscription,
            query: String::new(),
            offset: 0,
            total: 0,
            entries: Arc::new(vec![]),
            selected: None,
            summary: String::new(),
            coverage: vec![],
            details: Arc::new(vec![]),
            detail_rows: Arc::new(vec![]),
            sections: vec![],
            list_viewport: VirtualRows::default(),
            detail_viewport: VirtualRows::default(),
            show_coverage: false,
            sources: false,
            busy: false,
            detail_busy: false,
            load_token: 0,
            detail_token: 0,
        }
    }
    pub(crate) fn activate(&mut self, cx: &mut Context<Self>) {
        // Returning to this page must preserve the selected record and its level.
        // The refresh button explicitly reloads the catalogue when needed.
        if self.load_token == 0 {
            self.reload(cx);
        }
    }
    fn apply_search(&mut self, cx: &mut Context<Self>) {
        let query = self.search.read(cx).content.to_string();
        if query == self.query {
            return;
        }
        self.query = query;
        self.offset = 0;
        self.reload(cx);
    }
    fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.path.clone() else {
            return;
        };
        self.load_token += 1;
        self.detail_token += 1;
        let token = self.load_token;
        let (family, query, offset) = (self.family, self.query.clone(), self.offset);
        self.busy = true;
        self.detail_busy = false;
        self.error = None;
        self.selected = None;
        self.item = None;
        self.definition = None;
        self.entries = Arc::new(vec![]);
        self.total = 0;
        self.summary.clear();
        self.coverage.clear();
        self.details = Arc::new(vec![]);
        self.detail_rows = Arc::new(vec![]);
        self.sections.clear();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { debug_browser::list(&path, family, &query, offset) })
                .await;
            this.update(cx, |page, cx| {
                if token != page.load_token {
                    return;
                }
                page.busy = false;
                match result {
                    Ok(list) => {
                        page.summary = list.summary;
                        page.total = list.total;
                        page.coverage = list.coverage;
                        page.entries = Arc::new(list.entries);
                        page.list_viewport.scroll_to_item(0, ScrollStrategy::Top);
                        if page.show_coverage {
                            page.coverage_details();
                        } else if let Some(id) = page.entries.first().map(|entry| entry.id) {
                            page.select(id, cx);
                        }
                    }
                    Err(e) => page.error = Some(format!("Lecture du catalogue : {e}")),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
    fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Choisir data.sqlite (lecture seule)".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = receiver.await;
            this.update(cx, |page, cx| {
                match result {
                    Ok(Ok(Some(paths))) => {
                        if let Some(path) = paths.first() {
                            page.path = Some(path.clone());
                            page.offset = 0;
                            page.reload(cx);
                        }
                    }
                    Ok(Ok(None)) => {}
                    _ => page.error = Some("Sélecteur de fichiers indisponible.".into()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
    fn select(&mut self, id: i64, cx: &mut Context<Self>) {
        let Some(path) = self.path.clone() else {
            return;
        };
        self.selected = Some(id);
        self.show_coverage = false;
        self.detail_token += 1;
        let token = self.detail_token;
        let family = self.family;
        let sources = self.sources;
        self.detail_busy = true;
        self.item_slider
            .update(cx, |slider, cx| slider.set_disabled(true, cx));
        self.item = None;
        self.definition = None;
        self.details = Arc::new(vec![]);
        self.detail_rows = Arc::new(vec![]);
        self.sections.clear();
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    if family == Family::Items {
                        debug_browser::item(&path, id).map(|item| (vec![], Some(item), None))
                    } else if sources {
                        debug_browser::detail(&path, family, id, true)
                            .map(|lines| (lines, None, None))
                    } else {
                        debug_browser::definition(&path, family, id)
                            .map(|card| (vec![], None, Some(card)))
                    }
                })
                .await;
            this.update(cx, |page, cx| {
                if token != page.detail_token {
                    return;
                }
                page.detail_busy = false;
                match result {
                    Ok((lines, item, definition)) => {
                        let levels = item
                            .as_ref()
                            .map(|item| {
                                item.enchantments
                                    .iter()
                                    .map(|step| step.level as f64)
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_else(|| {
                                definition
                                    .as_ref()
                                    .map(|d| d.levels.iter().map(|s| s.level as f64).collect())
                                    .unwrap_or_default()
                            });
                        page.item_slider.update(cx, |slider, cx| {
                            let mut steps = if item.is_some() { vec![0.] } else { vec![] };
                            steps.extend(levels.iter().copied());
                            if steps.is_empty() {
                                steps.push(0.);
                            }
                            let min = steps[0];
                            let max = steps.last().copied().unwrap_or(0.).max(min + 1.);
                            if steps.len() == 1 {
                                steps.push(max);
                            }
                            slider
                                .configure(
                                    SliderState {
                                        min,
                                        max,
                                        value: min,
                                        steps,
                                        snap: true,
                                        keyboard_increment: 1.,
                                    },
                                    cx,
                                )
                                .expect("source levels are sorted positive integers");
                            slider.set_disabled(
                                levels.is_empty() || (item.is_none() && levels.len() < 2),
                                cx,
                            );
                        });
                        page.item = item.map(super::game_data_item::ItemPresentation::new);
                        page.definition = definition.map(|data| {
                            super::game_data_definition::DefinitionPresentation::new(data, cx)
                        });
                        page.set_details(lines);
                    }
                    Err(e) => page.error = Some(e.to_string()),
                }
                page.detail_viewport.scroll_to_item(0, ScrollStrategy::Top);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
    fn coverage_details(&mut self) {
        self.detail_token += 1;
        self.selected = None;
        self.item = None;
        self.definition = None;
        self.detail_busy = false;
        self.set_details(
            self.coverage
                .iter()
                .flat_map(|s| s.lines().map(str::to_owned).chain([String::new()]))
                .collect(),
        );
        self.detail_viewport.scroll_to_item(0, ScrollStrategy::Top);
    }
    fn set_details(&mut self, lines: Vec<String>) {
        // Preserve the original for copying; wrap the visual rows so long
        // descriptions and source values remain readable in a virtual list.
        self.detail_rows = Arc::new(
            lines
                .iter()
                .flat_map(|line| {
                    line.split('\n').flat_map(|line| {
                        let chars: Vec<_> = line.chars().collect();
                        if chars.is_empty() {
                            vec![String::new()]
                        } else {
                            chars
                                .chunks(160)
                                .map(|c| c.iter().collect::<String>())
                                .collect()
                        }
                    })
                })
                .collect(),
        );
        self.details = Arc::new(lines);
        self.sections = vec![("Fiche".into(), 0)];
        for (index, line) in self.detail_rows.iter().enumerate() {
            if line.starts_with("━━ ") {
                let label = line.trim_matches('━').trim().to_owned();
                if !self.sections.iter().any(|(old, _)| old == &label) {
                    self.sections.push((label, index));
                }
            }
        }
    }
}
impl Render for GameDataDebugPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entries = self.entries.clone();
        let details = self.detail_rows.clone();
        let selected = self.selected;
        let weak = cx.entity().downgrade();
        let range = format!(
            "{}–{} / {}",
            if self.total == 0 { 0 } else { self.offset + 1 },
            (self.offset + self.entries.len()).min(self.total),
            self.total
        );
        div()
            .flex()
            .flex_col()
            .size_full()
            .min_w_0()
            .min_h_0()
            .p_8()
            .gap_3()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Button::new("catalogue-open", "Ouvrir une base…")
                            .element()
                            .on_click(cx.listener(|page, _, window, cx| page.open(window, cx))),
                    )
                    .child(
                        Button::new("catalogue-reload", "Actualiser")
                            .disabled(self.busy)
                            .element()
                            .on_click(cx.listener(|page, _, _, cx| page.reload(cx))),
                    )
                    .child(
                        div().flex_1().min_w_0().text_xs().overflow_hidden().child(
                            self.path
                                .as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "Aucun fichier sélectionné".into()),
                        ),
                    ),
            )
            .when_some(self.error.as_ref(), |el, error| {
                el.child(div().text_sm().text_color(rgb(DANGER)).child(error.clone()))
            })
            .child(div().text_sm().text_color(rgb(ACCENT)).child(if self.busy {
                "Chargement…".into()
            } else {
                self.summary.clone()
            }))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .children(Family::ALL.into_iter().enumerate().map(|(index, family)| {
                        Button::new(("catalogue-family", index), family.label())
                            .selected(self.family == family)
                            .element()
                            .on_click(cx.listener(move |page, _, _, cx| {
                                if page.family == family && !page.show_coverage {
                                    return;
                                }
                                page.family = family;
                                page.offset = 0;
                                page.show_coverage = false;
                                page.reload(cx);
                            }))
                    }))
                    .child(
                        Button::new("catalogue-coverage", "Couverture / limites")
                            .selected(self.show_coverage)
                            .disabled(self.busy)
                            .element()
                            .on_click(cx.listener(|page, _, _, cx| {
                                page.show_coverage = true;
                                page.coverage_details();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(div().flex_1().min_w_0().child(self.search.clone()))
                    .child(
                        Button::new("catalogue-search", "Rechercher")
                            .element()
                            .on_click(cx.listener(|page, _, _, cx| {
                                page.apply_search(cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(290.))
                            .flex_shrink_0()
                            .min_h_0()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child("ID · nom localisé / nom anglais"),
                            )
                            .when(!self.busy && entries.is_empty(), |el| {
                                el.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .gap_2()
                                        .py_4()
                                        .child(crate::ui::components::icons::PackageX::new(
                                            px(32.),
                                            px(32.),
                                            rgb(TEXT_MUTED),
                                        ))
                                        .child(div().text_sm().child("Aucun résultat")),
                                )
                            })
                            .child(
                                self.list_viewport
                                    .element(
                                        "catalogue-list",
                                        entries.len(),
                                        58.,
                                        move |range, _, _| {
                                            range
                                                .map(|index| {
                                                    let entry = &entries[index];
                                                    let id = entry.id;
                                                    let weak = weak.clone();
                                                    div()
                                                        .id(("catalogue-entry", id as u64))
                                                        .h(px(58.))
                                                        .px_2()
                                                        .py_1()
                                                        .cursor_pointer()
                                                        .overflow_hidden()
                                                        .bg(rgb(if selected == Some(id) {
                                                            PANEL_MUTED
                                                        } else {
                                                            PANEL
                                                        }))
                                                        .hover(|s| s.bg(rgb(PANEL_MUTED)))
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .overflow_hidden()
                                                                .child(format!(
                                                                    "#{id} · {}",
                                                                    entry.name
                                                                )),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(TEXT_MUTED))
                                                                .overflow_hidden()
                                                                .child(entry.name_en.clone()),
                                                        )
                                                        .on_click(move |_, _, cx| {
                                                            weak.update(cx, |page, cx| {
                                                                page.select(id, cx)
                                                            })
                                                            .ok();
                                                        })
                                                        .into_any_element()
                                                })
                                                .collect()
                                        },
                                    )
                                    .flex_1()
                                    .min_h_0(),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        Button::new("catalogue-previous", "←")
                                            .disabled(self.busy || self.offset == 0)
                                            .element()
                                            .on_click(cx.listener(|page, _, _, cx| {
                                                page.offset = page.offset.saturating_sub(PAGE_SIZE);
                                                page.reload(cx);
                                            })),
                                    )
                                    .child(div().text_xs().child(range))
                                    .child(
                                        Button::new("catalogue-next", "→")
                                            .disabled(
                                                self.busy || self.offset + PAGE_SIZE >= self.total,
                                            )
                                            .element()
                                            .on_click(cx.listener(|page, _, _, cx| {
                                                if page.offset + PAGE_SIZE < page.total {
                                                    page.offset += PAGE_SIZE;
                                                    page.reload(cx);
                                                }
                                            })),
                                    ),
                            ),
                    )
                    .when(
                        !self.show_coverage && (self.family == Family::Items || !self.sources),
                        |el| {
                            el.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .min_w_0()
                                    .min_h_0()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px_4()
                                            .child(
                                                div().text_xs().text_color(rgb(TEXT_MUTED)).child(
                                                    self.selected
                                                        .map(|id| format!("#{id}"))
                                                        .unwrap_or_else(|| {
                                                            "Sélectionnez un item".into()
                                                        }),
                                                ),
                                            )
                                            .child(
                                                Button::new(
                                                    "item-language",
                                                    if self.english {
                                                        "EN ✓"
                                                    } else {
                                                        "Localisé ✓"
                                                    },
                                                )
                                                .element()
                                                .on_click(cx.listener(|page, _, _, cx| {
                                                    page.english = !page.english;
                                                    if let Some(definition) =
                                                        page.definition.as_ref()
                                                    {
                                                        definition.remeasure();
                                                    }
                                                    cx.notify();
                                                })),
                                            ),
                                    )
                                    .when(self.family != Family::Items, |el| {
                                        el.child(
                                            Button::new("catalogue-card-source", "Source complète")
                                                .disabled(self.selected.is_none())
                                                .element()
                                                .on_click(cx.listener(|page, _, _, cx| {
                                                    page.sources = true;
                                                    if let Some(id) = page.selected {
                                                        page.select(id, cx);
                                                    }
                                                })),
                                        )
                                    })
                                    .when_some(self.item.as_ref(), |el, item| {
                                        el.child(
                                            item.render(self.english, self.item_slider.clone()),
                                        )
                                    })
                                    .when_some(self.definition.as_ref(), |el, definition| {
                                        el.child(
                                            definition
                                                .render(self.english, self.item_slider.clone()),
                                        )
                                    })
                                    .when(self.detail_busy, |el| el.child("Chargement…")),
                            )
                        },
                    )
                    .when(
                        self.show_coverage || (self.family != Family::Items && self.sources),
                        |el| {
                            el.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .min_w_0()
                                    .min_h_0()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex()
                                            .gap_2()
                                            .items_center()
                                            .child(div().flex_1().text_sm().child(
                                                if self.show_coverage {
                                                    "Couverture de l’extraction".into()
                                                } else {
                                                    selected
                                                        .map(|id| format!("Fiche #{id}"))
                                                        .unwrap_or_else(|| {
                                                            "Sélectionnez une entrée".into()
                                                        })
                                                },
                                            ))
                                            .child(
                                                Button::new(
                                                    "catalogue-sources",
                                                    if self.sources {
                                                        "Source complète ✓"
                                                    } else {
                                                        "Source complète"
                                                    },
                                                )
                                                .selected(self.sources)
                                                .disabled(selected.is_none() || self.detail_busy)
                                                .element()
                                                .on_click(cx.listener(|page, _, _, cx| {
                                                    page.sources = !page.sources;
                                                    if let Some(id) = page.selected {
                                                        page.select(id, cx);
                                                    }
                                                })),
                                            )
                                            .child(
                                                Button::new("catalogue-copy", "Copier")
                                                    .disabled(self.details.is_empty())
                                                    .element()
                                                    .on_click(cx.listener(|page, _, _, cx| {
                                                        cx.write_to_clipboard(
                                                            ClipboardItem::new_string(
                                                                page.details.join("\n"),
                                                            ),
                                                        );
                                                    })),
                                            ),
                                    )
                                    .when(self.detail_busy, |el| {
                                        el.child(div().text_sm().child("Lecture de la fiche…"))
                                    })
                                    .child(
                                        div()
                                            .id("catalogue-section-navigation")
                                            .flex()
                                            .gap_2()
                                            .overflow_x_scroll()
                                            .flex_shrink_0()
                                            .children(self.sections.iter().enumerate().map(
                                                |(number, (label, index))| {
                                                    let index = *index;
                                                    Button::new(
                                                        ("catalogue-section", number),
                                                        label.clone(),
                                                    )
                                                    .element()
                                                    .on_click(cx.listener(move |page, _, _, cx| {
                                                        page.detail_viewport.scroll_to_item(
                                                            index,
                                                            ScrollStrategy::Top,
                                                        );
                                                        cx.notify();
                                                    }))
                                                },
                                            )),
                                    )
                                    .child(
                                        div()
                                            .id("catalogue-detail-horizontal")
                                            .flex()
                                            .flex_1()
                                            .min_h_0()
                                            .min_w_0()
                                            .overflow_x_scroll()
                                            .bg(rgb(PANEL))
                                            .child(
                                                self.detail_viewport
                                                    .element(
                                                        "catalogue-details",
                                                        details.len(),
                                                        22.,
                                                        move |range, _, _| {
                                                            range
                                                                .map(|index| {
                                                                    div()
                                                                        .h(px(22.))
                                                                        .px_2()
                                                                        .text_xs()
                                                                        .whitespace_nowrap()
                                                                        .font_family("monospace")
                                                                        .child(
                                                                            details[index].replace(
                                                                                '\n', " ↵ ",
                                                                            ),
                                                                        )
                                                                        .into_any_element()
                                                                })
                                                                .collect()
                                                        },
                                                    )
                                                    .w(px(1600.))
                                                    .flex_shrink_0()
                                                    .h_full(),
                                            ),
                                    ),
                            )
                        },
                    ),
            )
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::*;
    use gpui::{Focusable, TestAppContext};
    use std::{cell::Cell, rc::Rc};

    struct SearchAndSlider {
        page: Entity<GameDataDebugPage>,
    }
    impl Render for SearchAndSlider {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let page = self.page.read(cx);
            div()
                .flex()
                .flex_col()
                .size_full()
                .child(page.search.clone())
                .child(page.item_slider.clone())
        }
    }
    #[gpui::test]
    fn unchanged_search_focus_then_slider_preserves_the_selected_record(cx: &mut TestAppContext) {
        cx.update(TextInput::register);
        let other_window = cx.add_window(|_, _| gpui::Empty);
        let (view, cx) = cx.add_window_view(|window, cx| {
            let page = cx.new(GameDataDebugPage::new);
            let search = page.read(cx).search.clone();
            window.focus(&search.read(cx).focus_handle(cx), cx);
            SearchAndSlider { page }
        });
        let page = view.update(cx, |view, _| view.page.clone());
        let slider = page.update(cx, |page, _| {
            // A reload would increment these tokens and clear the selection.
            page.selected = Some(42);
            page.load_token = 7;
            page.detail_token = 9;
            page.item_slider.clone()
        });
        let changed = Rc::new(Cell::new(0.));
        let events = changed.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&slider, move |_, change: &SliderChanged, _| {
                events.set(change.0)
            })
        });
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| window.focus(&slider.read(cx).focus_handle(cx), cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("right");
        cx.run_until_parked();
        other_window
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
        cx.run_until_parked();
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        cx.update(|window, cx| window.blur(cx));
        cx.run_until_parked();
        page.update(cx, |page, cx| {
            // The search button must also ignore an already applied query.
            page.apply_search(cx);
            // Re-entering the catalogue must not reload it either.
            page.activate(cx);
            assert_eq!(page.selected, Some(42));
            assert_eq!(page.load_token, 7);
            assert_eq!(page.detail_token, 9);
        });
        assert_eq!(changed.get(), 1.);
    }
}
