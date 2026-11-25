//! Item tooltip layout from the supplied Penpot database page.
use super::catalogue_card::{self, section};
use crate::ui::{
    components::Slider,
    theme::{self, tokens as t},
};
use aetheris::game_data::debug_browser::ItemCard;
use gpui::{
    AnyElement, Entity, FontWeight, Image, PathBuilder, Pixels, Point, canvas, div, img, point,
    prelude::*, px, rgb,
};
use std::sync::Arc;

pub(crate) struct ItemPresentation {
    pub data: ItemCard,
    icon: Option<Arc<Image>>,
    currency_icon: Option<Arc<Image>>,
    permission_icons: Vec<Arc<Image>>,
    pub enchant_level: i64,
}
impl ItemPresentation {
    pub fn new(data: ItemCard) -> Self {
        let icon = catalogue_card::image(data.icon.as_deref(), data.icon_mime.as_deref());
        let currency_icon = catalogue_card::image(
            data.currency_icon.as_deref(),
            data.currency_icon_mime.as_deref(),
        );
        // Keep image identities stable so scrolling reuses GPUI's decoded textures.
        let permission_icons = data
            .permissions
            .iter()
            .filter(|(_, allowed)| *allowed)
            .filter_map(|(code, _)| {
                let bytes: &[u8] = match code.as_str() {
                    "CanDecompose" => {
                        include_bytes!("../../../resources/item-card/can-decompose.png")
                    }
                    "CanStorage" => include_bytes!("../../../resources/item-card/can-storage.png"),
                    _ => return None,
                };
                catalogue_card::image(Some(bytes), Some("image/png"))
            })
            .collect();
        Self {
            data,
            icon,
            currency_icon,
            permission_icons,
            enchant_level: 0,
        }
    }
    pub fn render(&self, english: bool, slider: Entity<Slider>) -> AnyElement {
        let i = &self.data;
        let unique = i.rarity == "Unique";
        let enchantment = i
            .enchantments
            .iter()
            .find(|step| step.level == self.enchant_level);
        let amplify_slots = i
            .enchantments
            .iter()
            .map(|step| step.amplify_level)
            .max()
            .unwrap_or(0);
        let amplify_level = enchantment.map(|step| step.amplify_level).unwrap_or(0);
        let name = if english {
            i.name_en.as_ref().or(i.name_localized.as_ref())
        } else {
            i.name_localized.as_ref().or(i.name_en.as_ref())
        }
        .cloned()
        .unwrap_or_else(|| "Sans nom".into());
        let description = if english {
            i.description_en
                .as_ref()
                .or(i.description_localized.as_ref())
        } else {
            i.description_localized
                .as_ref()
                .or(i.description_en.as_ref())
        };
        let title_color = if unique {
            t::TOKEN_GRADE_UNIQUE_LIGHT
        } else {
            theme::TEXT
        };
        let header = catalogue_card::header(unique)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .text_size(px(t::TOKEN_XS))
                    .line_height(px(20.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(t::TOKEN_WHITE))
                    .child(
                        div()
                            .flex()
                            .gap(px(2.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(title_color))
                            .when(self.enchant_level > 0, |el| {
                                el.child(
                                    enchantment
                                        .map(|s| s.title())
                                        .unwrap_or_else(|| format!("+{}", self.enchant_level)),
                                )
                            })
                            .child(name),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .text_size(px(t::TOKEN_XS))
                            .child(
                                div()
                                    .text_color(rgb(if unique {
                                        t::TOKEN_GRADE_UNIQUE
                                    } else {
                                        theme::TEXT
                                    }))
                                    .child(i.rarity_label.text(english).to_owned()),
                            )
                            .child(
                                div()
                                    .text_color(rgb(t::TOKEN_WHITE))
                                    .child(i.category.text(english).to_owned()),
                            ),
                    )
                    .when_some(i.armor.as_ref(), |el, a| {
                        el.child(
                            div()
                                .text_size(px(t::TOKEN_XS))
                                .child(a.text(english).to_owned()),
                        )
                    })
                    .when(i.level > 0, |el| {
                        el.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(2.))
                                .text_size(px(t::TOKEN_XS))
                                .text_color(rgb(t::TOKEN_WHITE))
                                .child(format!("Item Level  {}", i.level))
                                .when_some(
                                    enchantment
                                        .and_then(|step| step.item_level_bonus)
                                        .filter(|v| *v != 0),
                                    |el, bonus| el.child(format!("  ({bonus:+})")),
                                ),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(4.))
                    .flex_shrink_0()
                    .min_w(px(70.85))
                    .min_h(px(76.14))
                    .child(
                        div()
                            .size(px(58.))
                            .overflow_hidden()
                            .when_some(self.icon.as_ref(), |el, icon| {
                                el.child(img(icon.clone()).size_full())
                            })
                            .when(self.icon.is_none(), |el| {
                                el.flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .text_color(rgb(theme::TEXT_MUTED))
                                    .child("—")
                            }),
                    )
                    .when(amplify_slots > 0, |el| {
                        el.child(div().flex().children(
                            (1..=amplify_slots).map(|level| amplify_slot(level <= amplify_level)),
                        ))
                    }),
            );

        let card = catalogue_card::card(header)
            .when_some(description, |el, text| {
                el.child(section(None).child(text.clone()))
            })
            .when(!i.enchantments.is_empty(), |el| {
                el.child(
                    section(None).px(px(16.)).py(px(4.)).child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(div().flex_1().min_w_0().child(slider))
                            .child(
                                enchantment
                                    .map(|s| s.title())
                                    .unwrap_or_else(|| "+0".into()),
                            ),
                    ),
                )
            })
            .when(
                !i.stats.is_empty()
                    || !i.attack_ranges.is_empty()
                    || enchantment.is_some_and(|step| !step.amplification_stats.is_empty()),
                |el| {
                    el.child(
                        section(Some("MAIN STATS"))
                            .children(i.stats.iter().map(|s| {
                                div()
                                    .flex()
                                    .items_start()
                                    .justify_between()
                                    .gap(px(12.))
                                    .w_full()
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .text_color(rgb(theme::TEXT_MUTED))
                                            .child(s.name.text(english).to_owned()),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_shrink_0()
                                            .gap(px(8.))
                                            .child(s.value.clone())
                                            .when_some(
                                                enchantment
                                                    .and_then(|step| step.stats.get(&s.code))
                                                    .filter(|v| **v != 0),
                                                |el, bonus| {
                                                    el.child(
                                                        div()
                                                            .text_color(rgb(theme::TEXT_MUTED))
                                                            .child(s.bonus_text(*bonus)),
                                                    )
                                                },
                                            ),
                                    )
                            }))
                            .children(
                                enchantment
                                    .into_iter()
                                    .flat_map(|step| step.amplification_stats.iter())
                                    .map(|stat| {
                                        div()
                                            .flex()
                                            .items_start()
                                            .justify_between()
                                            .gap(px(12.))
                                            .w_full()
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .text_color(rgb(theme::TEXT_MUTED))
                                                    .child(stat.name.text(english).to_owned()),
                                            )
                                            .child(
                                                div()
                                                    .flex_shrink_0()
                                                    .text_color(rgb(t::TOKEN_YELLOW_LIGHT))
                                                    .child(
                                                        stat.value
                                                            .trim_start_matches('+')
                                                            .to_owned(),
                                                    ),
                                            )
                                    }),
                            )
                            .children(i.attack_ranges.iter().map(|range| {
                                div()
                                    .flex()
                                    .items_start()
                                    .justify_between()
                                    .gap(px(12.))
                                    .w_full()
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .text_color(rgb(theme::TEXT_MUTED))
                                            .child(range.name(english)),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_shrink_0()
                                            .gap(px(8.))
                                            .child(range.value.clone())
                                            .when_some(range.extension.as_ref(), |el, value| {
                                                el.child(
                                                    div()
                                                        .text_color(rgb(theme::TEXT_MUTED))
                                                        .child(value.clone()),
                                                )
                                            }),
                                    )
                            })),
                    )
                },
            )
            .when(!i.sets.is_empty(), |el| {
                el.child(section(Some("SET EFFECT")).children(i.sets.iter().map(|s| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .child(s.name.text(english).to_owned()),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .w(px(16.))
                                        .text_color(rgb(theme::TEXT_MUTED))
                                        .child(s.count.to_string()),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .child(s.description.text(english).to_owned()),
                                ),
                        )
                })))
            })
            .when_some(i.manastones.filter(|v| *v > 0), |el, count| {
                el.child(slots("MANASTONE", count, false))
            })
            .when_some(i.theostones.filter(|v| *v > 0), |el, count| {
                el.child(slots("THEOSTONE", count, true))
            })
            .when(i.supports_philosopher_stone, |el| {
                el.child(
                    section(None).p(px(4.)).min_h(px(28.)).child(
                        div()
                            .flex()
                            .justify_between()
                            .gap(px(8.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(div().text_color(rgb(theme::TEXT_MUTED)).child(if english {
                                "PHILOSOPHER’S STONE"
                            } else {
                                "PIERRE PHILOSOPHALE"
                            }))
                            .child(div().text_color(rgb(t::TOKEN_GRAY_700)).child(if english {
                                "not fused"
                            } else {
                                "non fusionnée"
                            })),
                    ),
                )
            })
            .when_some(i.race.as_ref(), |el, race| {
                el.child(
                    section(None)
                        .py(px(4.))
                        .px(px(4.))
                        .child(race.text(english).to_owned()),
                )
            })
            .child(
                section(None).p(px(4.)).child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_between()
                        .gap(px(8.))
                        .when_some(i.required_level.filter(|v| *v > 0), |el, level| {
                            el.child(if level == 1 {
                                "Usable at all levels".to_owned()
                            } else {
                                format!("Required Level  {level}")
                            })
                        })
                        .child(
                            div().flex().items_center().gap(px(2.)).children(
                                self.permission_icons
                                    .iter()
                                    .map(|icon| img(icon.clone()).size(px(20.)).into_any_element()),
                            ),
                        )
                        .when(i.sell_price > 0, |el| {
                            el.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(4.))
                                    .child(i.sell_price.to_string())
                                    .when_some(self.currency_icon.as_ref(), |el, icon| {
                                        el.child(img(icon.clone()).size(px(20.)))
                                    })
                                    .when(self.currency_icon.is_none(), |el| {
                                        el.child(i.currency.text(english).to_owned())
                                    }),
                            )
                        }),
                ),
            )
            .into_any_element();
        catalogue_card::scroll(card)
    }
}

fn slots(title: &'static str, count: i64, round: bool) -> gpui::Div {
    section(None).p(px(4.)).child(
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.))
            .child(div().text_color(rgb(theme::TEXT_MUTED)).child(title))
            .child(
                div()
                    .flex()
                    .gap(px(2.))
                    .items_center()
                    .children((0..count.min(16)).map(|_| {
                        div()
                            .size(px(14.))
                            .border_1()
                            .border_color(rgb(theme::TEXT_MUTED))
                            .when(round, |el| el.rounded_full())
                            .when(!round, |el| el.rounded(px(2.)))
                    }))
                    .when(count > 16, |el| el.child(format!("×{count}"))),
            ),
    )
}

// Penpot amplify-slot: a 10 px square rotated 45°, with a 6 px inset.
fn amplify_slot(filled: bool) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let color = rgb(if filled {
                t::TOKEN_ACCENT
            } else {
                t::TOKEN_WHITE
            });
            if let Ok(path) = amplify_diamond(bounds.center(), 9., 0.5, true).build() {
                window.paint_path(path, color);
            }
            if filled && let Ok(path) = amplify_diamond(bounds.center(), 6., 1., false).build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(14.14))
}
fn amplify_diamond(center: Point<Pixels>, side: f32, radius: f32, stroke: bool) -> PathBuilder {
    let mut path = if stroke {
        PathBuilder::stroke(px(1.))
    } else {
        PathBuilder::fill()
    };
    let p = |x: f32, y: f32| {
        center
            + point(
                px((x - y) * std::f32::consts::FRAC_1_SQRT_2),
                px((x + y) * std::f32::consts::FRAC_1_SQRT_2),
            )
    };
    let a = -side / 2.;
    let b = side / 2.;
    let r = radius;
    path.move_to(p(a + r, a));
    path.line_to(p(b - r, a));
    path.curve_to(p(b, a + r), p(b, a));
    path.line_to(p(b, b - r));
    path.curve_to(p(b - r, b), p(b, b));
    path.line_to(p(a + r, b));
    path.curve_to(p(a, b - r), p(a, b));
    path.line_to(p(a, a + r));
    path.curve_to(p(a + r, a), p(a, a));
    path.close();
    path
}
