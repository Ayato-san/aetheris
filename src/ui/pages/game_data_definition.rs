//! Skill, status and SE/ABE/SEF presentations using the item card component.
use super::catalogue_card;
use crate::ui::{
    components::{Slider, VirtualList},
    theme::{self, tokens as t},
};
use aetheris::game_data::debug_browser::{CardField, DefinitionCard};
use gpui::{AnyElement, App, Entity, FontWeight, Image, div, prelude::*, px, rgb};
use std::{rc::Rc, sync::Arc};

pub(crate) struct DefinitionPresentation {
    data: Rc<DefinitionCard>,
    icon: Option<Arc<Image>>,
    related: Rc<Vec<DefinitionPresentation>>,
    viewport: VirtualList,
    pub level: i64,
}
impl DefinitionPresentation {
    pub fn new(mut data: DefinitionCard, cx: &App) -> Self {
        let icon = catalogue_card::image(data.icon.as_deref(), data.icon_mime.as_deref());
        let related = std::mem::take(&mut data.related)
            .into_iter()
            .map(|data| Self::new(data, cx))
            .collect::<Vec<_>>();
        let viewport = VirtualList::new(related.len() + 1, cx).gap(0.);
        let level = data.levels.first().map(|s| s.level).unwrap_or(1);
        Self {
            data: Rc::new(data),
            icon,
            related: Rc::new(related),
            viewport,
            level,
        }
    }
    pub fn remeasure(&self) {
        self.viewport.remeasure();
    }
    pub fn render(&self, english: bool, slider: Entity<Slider>) -> AnyElement {
        let data = self.data.clone();
        let icon = self.icon.clone();
        let related = self.related.clone();
        let selected = self.level;
        self.viewport
            .element("catalogue-definition-scroll", move |index, _, _| {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(12.))
                    .w_full()
                    .px(px(16.))
                    .pb(px(12.))
                    .when(index == 0, |el| {
                        el.pt(px(16.)).child(card(
                            &data,
                            icon.as_ref(),
                            english,
                            selected,
                            Some(slider.clone()),
                        ))
                    })
                    .when(index == 0 && !related.is_empty(), |el| {
                        el.child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(theme::TEXT_MUTED))
                                .child(if english {
                                    "LINKED EFFECTS"
                                } else {
                                    "EFFETS LIÉS"
                                }),
                        )
                    })
                    .when(index > 0, |el| {
                        let effect = &related[index - 1];
                        el.child(card(
                            &effect.data,
                            effect.icon.as_ref(),
                            english,
                            selected,
                            None,
                        ))
                    })
                    .when(index == related.len() && data.related_truncated, |el| {
                        el.child(if english {
                            "Only the first 200 linked effects are displayed."
                        } else {
                            "Affichage limité aux 200 premiers effets liés."
                        })
                    })
                    .into_any_element()
            })
            .min_w_0()
            .min_h_0()
            .into_any_element()
    }
}
fn card(
    d: &DefinitionCard,
    icon: Option<&Arc<Image>>,
    english: bool,
    selected: i64,
    slider: Option<Entity<Slider>>,
) -> AnyElement {
    // A missing override stays at the base definition; never take a nearby level.
    let level = d
        .levels
        .iter()
        .find(|s| s.level == selected)
        .or_else(|| d.levels.first());
    let name = choose(&d.name_en, &d.name_localized, english)
        .cloned()
        .unwrap_or_else(|| format!("#{}", d.id));
    let header = catalogue_card::header(false)
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(px(2.))
                .text_size(px(t::TOKEN_XS))
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(theme::TEXT))
                        .child(plain_text(&name)),
                )
                .child(
                    div()
                        .text_color(rgb(theme::TEXT_MUTED))
                        .child(d.tag.clone()),
                )
                .when_some(level, |el, level| {
                    el.child(format!(
                        "{} {}",
                        if english { "Level" } else { "Niveau" },
                        level.level
                    ))
                }),
        )
        .child(catalogue_card::icon(icon, 58.));
    let mut card = catalogue_card::card(header);
    if let Some(level) = level {
        if let Some(description) =
            choose(&level.description_en, &level.description_localized, english)
        {
            card = card.child(catalogue_card::section(None).child(plain_text(description)));
        }
        if let Some(slider) = slider
            && d.levels.len() > 1
        {
            card = card.child(
                catalogue_card::section(None).child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().flex_1().min_w_0().child(slider))
                        .child(level.level.to_string()),
                ),
            );
        }
        for section in &level.sections {
            if !section.fields.is_empty() {
                card = card.child(
                    catalogue_card::section(Some(if english {
                        section.name_en
                    } else {
                        section.name_localized
                    }))
                    .children(section.fields.iter().map(|field| field_row(field, english)))
                    .when(section.truncated, |el| {
                        el.child(if english {
                            "Only the first 200 entries are displayed."
                        } else {
                            "Affichage limité aux 200 premières entrées."
                        })
                    }),
                );
            }
        }
    }
    card.into_any_element()
}
fn field_row(field: &CardField, english: bool) -> gpui::Div {
    let value = if english {
        &field.value
    } else {
        field.value_localized.as_ref().unwrap_or(&field.value)
    };
    div()
        .flex()
        .items_start()
        .justify_between()
        .gap(px(12.))
        .w_full()
        .child(
            div()
                .flex_shrink_0()
                .text_color(rgb(theme::TEXT_MUTED))
                .child(if english {
                    field.name_en
                } else {
                    field.name_localized
                }),
        )
        .child(div().flex_1().min_w_0().child(plain_text(value)))
}
fn choose<'a>(
    en: &'a Option<String>,
    localized: &'a Option<String>,
    english: bool,
) -> Option<&'a String> {
    if english {
        en.as_ref().or(localized.as_ref())
    } else {
        localized.as_ref().or(en.as_ref())
    }
}
fn plain_text(text: &str) -> String {
    let mut rendered = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once('<') {
        rendered.push_str(before);
        if let Some((tag, tail)) = after.split_once('>')
            && (tag.starts_with('/') || tag.starts_with(|c: char| c.is_ascii_alphabetic()))
        {
            rest = tail;
        } else {
            rendered.push('<');
            rest = after;
        }
    }
    rendered.push_str(rest);
    rendered
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::*;
    use crate::ui::components::state::SliderState;
    use aetheris::game_data::debug_browser::CardLevel;
    use gpui::{Context, ListOffset, Render, TestAppContext, Window};

    struct Host {
        definition: DefinitionPresentation,
        slider: Entity<Slider>,
        english: bool,
    }
    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .w(px(450.))
                .h(px(300.))
                .child(self.definition.render(self.english, self.slider.clone()))
        }
    }
    fn definition(id: i64) -> DefinitionCard {
        DefinitionCard {
            id,
            name_en: Some(format!("Skill {id}")),
            name_localized: Some(format!("Compétence {id}")),
            tag: "SE".into(),
            icon: None,
            icon_mime: None,
            levels: (1..=2)
                .map(|level| CardLevel {
                    level,
                    description_en: Some("Effect description. ".repeat(level as usize)),
                    description_localized: Some(
                        "Description détaillée de cet effet. ".repeat(level as usize * 20),
                    ),
                    sections: vec![],
                })
                .collect(),
            related: vec![],
            related_truncated: false,
        }
    }
    #[gpui::test]
    fn linked_effects_scroll_to_the_end_and_remeasure_after_level_and_language_changes(
        cx: &mut TestAppContext,
    ) {
        let (host, cx) = cx.add_window_view(|_, cx| {
            let mut data = definition(0);
            data.related = (1..=200).map(definition).collect();
            Host {
                definition: DefinitionPresentation::new(data, cx),
                slider: cx.new(|cx| {
                    Slider::new(
                        "Niveau",
                        SliderState {
                            min: 1.,
                            max: 2.,
                            value: 1.,
                            steps: vec![1., 2.],
                            snap: true,
                            keyboard_increment: 1.,
                        },
                        cx,
                    )
                    .unwrap()
                }),
                english: true,
            }
        });
        let initial_height = host.read_with(cx, |host, _| {
            host.definition
                .viewport
                .state()
                .max_offset_for_scrollbar()
                .y
        });
        assert!(initial_height > px(300.));
        host.update(cx, |host, cx| {
            host.definition.viewport.state().scroll_to(ListOffset {
                item_ix: 200,
                offset_in_item: px(0.),
            });
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        host.read_with(cx, |host, _| {
            assert!(
                host.definition
                    .viewport
                    .state()
                    .scroll_px_offset_for_scrollbar()
                    .y
                    < px(0.)
            );
        });
        host.update(cx, |host, cx| {
            host.definition.level = 2;
            host.english = false;
            host.definition.remeasure();
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        host.read_with(cx, |host, _| {
            let viewport = host.definition.viewport.state();
            assert!(viewport.max_offset_for_scrollbar().y > initial_height);
            assert!(viewport.scroll_px_offset_for_scrollbar().y < px(0.));
            assert_eq!(host.definition.level, 2);
        });
    }
}
