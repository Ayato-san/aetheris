//! Development-only integration surface. Entities outlive individual renders.
use crate::ui::components::{
    state::{NumberConfig, OptionItem, Selection, SliderState},
    text_input::TextInput,
};
use crate::ui::{components::*, theme::ControlSize};
use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*};
use std::{
    collections::BTreeSet,
    time::{Duration, SystemTime},
};

pub(crate) struct Gallery {
    text: Entity<TextInput>,
    number: Entity<TextInput>,
    switches: Vec<Entity<Switch>>,
    selectors: Vec<Entity<Selector>>,
    slider: Entity<Slider>,
    shortcut: Entity<ShortcutSelector>,
    timers: Vec<Entity<Timer>>,
    searches: Vec<Entity<SearchBar>>,
    nested_select: Entity<Selector>,
    popover_switch: Entity<Switch>,
    table_switch: Entity<Switch>,
    popover: Entity<Overlay>,
    popup: Entity<Overlay>,
    disabled: bool,
    refuse: bool,
    feedback: String,
    sort: Option<SortRequest>,
    scroll: VirtualRows,
    page_viewport: VirtualList,
    _subscriptions: Vec<Subscription>,
}
fn options() -> Vec<OptionItem> {
    let mut values = vec![
        OptionItem::new("first", "Premier"),
        OptionItem::new("second", "Deuxième"),
        OptionItem::new("third", "Troisième"),
    ];
    values[2].disabled = true;
    values
}
impl Gallery {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let text = cx.new(|cx| TextInput::text("Texte libre", cx));
        let number = cx.new(|cx| {
            NumberInput::create(
                "12".into(),
                "Nombre entre 0 et 100".into(),
                NumberConfig {
                    min: Some(0.),
                    max: Some(100.),
                    decimals: true,
                    ..Default::default()
                },
                cx,
            )
            .unwrap()
        });
        let switches = vec![
            cx.new(|cx| Switch::new("Standard", false, SwitchVariant::Standard, cx)),
            cx.new(|cx| Switch::new("Carte indépendante", true, SwitchVariant::Card, cx)),
        ];
        let selectors = [
            SelectorKind::Segment,
            SelectorKind::Multi,
            SelectorKind::Select,
        ]
        .into_iter()
        .map(|kind| {
            cx.new(|cx| {
                Selector::new(
                    kind,
                    match kind {
                        SelectorKind::Segment => "",
                        SelectorKind::Multi => "Choix multiples",
                        SelectorKind::Select => "Select",
                    },
                    Selection::new(
                        if kind == SelectorKind::Segment {
                            let mut values = vec![
                                OptionItem::new("first", "Option 1"),
                                OptionItem::new("second", "Option 2"),
                                OptionItem::new("third", "Option 3"),
                                OptionItem::new("fourth", "Option 4"),
                            ];
                            values[3].disabled = true;
                            values
                        } else {
                            options()
                        },
                        BTreeSet::from(["first".into()]),
                        if kind == SelectorKind::Segment { 1 } else { 0 },
                        if kind == SelectorKind::Segment { 1 } else { 2 },
                    )
                    .unwrap(),
                    cx,
                )
                .unwrap()
            })
        })
        .collect::<Vec<_>>();
        let nested_select = cx.new(|cx| {
            Selector::new(
                SelectorKind::Select,
                "Select dans la Popup",
                Selection::new(options(), BTreeSet::new(), 0, 1).unwrap(),
                cx,
            )
            .unwrap()
        });
        let popover_switch =
            cx.new(|cx| Switch::new("Réglage dans le Popover", true, SwitchVariant::Standard, cx));
        let table_switch = cx.new(|cx| {
            Switch::new(
                "État conservé dans la table",
                false,
                SwitchVariant::Standard,
                cx,
            )
        });
        let slider = cx.new(|cx| {
            Slider::new(
                "Curseur à pas irréguliers",
                SliderState {
                    min: 0.,
                    max: 100.,
                    value: 20.,
                    steps: vec![0., 20., 50., 100.],
                    snap: true,
                    keyboard_increment: 1.,
                },
                cx,
            )
            .unwrap()
        });
        let shortcut = cx.new(|cx| ShortcutSelector::new("Raccourci local", cx));
        let timers = vec![
            cx.new(|_| Timer::new(Deadline::duration(Duration::from_secs(90)))),
            cx.new(|_| Timer::new(Deadline::Calendar(SystemTime::UNIX_EPOCH))),
        ];
        let searches = vec![cx.new(|cx| SearchBar::new("Recherche à la validation", cx))];
        let popover = cx.new(|cx| Overlay::new(OverlayKind::Popover, cx));
        let popup = cx.new(|cx| Overlay::new(OverlayKind::Popup { modal: true }, cx));
        let owner = cx.entity().downgrade();
        popup.update(cx, |popup, _| {
            popup.on_close_requested(move |_, cx| {
                owner
                    .update(cx, |page, _| {
                        if page.refuse {
                            Err("Fermeture refusée par la page".into())
                        } else {
                            Ok(())
                        }
                    })
                    .unwrap_or(Ok(()))
            })
        });
        popup.update(cx, |popup, cx| popup.dimensions(520., 500., cx).unwrap());
        timers[1].update(cx, |timer, cx| {
            timer.expired_text("Déjà disponible", cx);
            timer.set_max_units(2, cx).unwrap();
        });
        let mut subscriptions = Vec::new();
        let owner = cx.entity().downgrade();
        slider.update(cx, |control, _| {
            control.on_change_requested(move |_, cx| {
                owner
                    .update(cx, |page, _| {
                        if page.refuse {
                            Err("Valeur refusée".into())
                        } else {
                            Ok(())
                        }
                    })
                    .unwrap_or(Ok(()))
            })
        });
        let owner = cx.entity().downgrade();
        shortcut.update(cx, |control, _| {
            control.on_change_requested(move |_, cx| {
                owner
                    .update(cx, |page, _| {
                        if page.refuse {
                            Err("Raccourci refusé".into())
                        } else {
                            Ok(())
                        }
                    })
                    .unwrap_or(Ok(()))
            })
        });
        for control in &switches {
            let owner = cx.entity().downgrade();
            control.update(cx, |control, _| {
                control.on_change_requested(move |_, cx| {
                    owner
                        .update(cx, |page, _| {
                            if page.refuse {
                                Err("Valeur refusée".into())
                            } else {
                                Ok(())
                            }
                        })
                        .unwrap_or(Ok(()))
                })
            });
        }
        for control in &selectors {
            let owner = cx.entity().downgrade();
            control.update(cx, |control, _| {
                control.on_change_requested(move |_, cx| {
                    owner
                        .update(cx, |page, _| {
                            if page.refuse {
                                Err("Sélection refusée".into())
                            } else {
                                Ok(())
                            }
                        })
                        .unwrap_or(Ok(()))
                })
            });
        }
        for input in [&text, &number] {
            let owner = cx.entity().downgrade();
            input.update(cx, |input, _| {
                input.on_change_requested(move |value, cx| {
                    owner
                        .update(cx, |page, _| {
                            if page.refuse {
                                Err("Valeur refusée par la page".into())
                            } else {
                                Ok(value.into())
                            }
                        })
                        .unwrap_or_else(|_| Err("Page fermée".into()))
                })
            });
        }
        for selector in &selectors {
            subscriptions.push(
                cx.subscribe(selector, |page, _, event: &SelectionChanged, cx| {
                    page.feedback = format!("Sélection acceptée : {:?}", event.0);
                    cx.notify();
                }),
            );
        }
        for switch in &switches {
            subscriptions.push(cx.subscribe(switch, |page, _, event: &SwitchChanged, cx| {
                page.feedback = format!("Switch accepté : {}", event.0);
                cx.notify();
            }));
        }
        subscriptions.push(cx.subscribe(&slider, |page, _, event: &SliderChanged, cx| {
            page.feedback = format!("Valeur du curseur : {}", event.0);
            cx.notify();
        }));
        subscriptions.push(cx.subscribe(&slider, |page, _, _: &InteractionEnded, cx| {
            page.feedback.push_str(" · interaction terminée");
            cx.notify();
        }));
        subscriptions.push(
            cx.subscribe(&shortcut, |page, _, event: &ShortcutChanged, cx| {
                page.feedback = format!("Raccourci accepté : {:?}", event.0);
                cx.notify();
            }),
        );
        for timer in &timers {
            subscriptions.push(cx.subscribe(timer, |page, _, _: &Expired, cx| {
                page.feedback = "Échéance atteinte une fois".into();
                cx.notify();
            }));
        }
        for search in &searches {
            subscriptions.push(
                cx.subscribe(search, |page, _, event: &SearchRequested, cx| {
                    page.feedback = format!("Recherche : {}", event.0);
                    cx.notify();
                }),
            );
        }
        Self {
            text,
            number,
            switches,
            selectors,
            slider,
            shortcut,
            timers,
            searches,
            nested_select,
            popover_switch,
            table_switch,
            popover,
            popup,
            disabled: false,
            refuse: false,
            feedback: String::new(),
            sort: None,
            scroll: VirtualRows::default(),
            page_viewport: VirtualList::new(14, cx),
            _subscriptions: subscriptions,
        }
    }
}
fn table() -> Table {
    Table::new(vec![
        Column::new("id", "Identifiant", ColumnWidth::Fixed(150.)).sortable(true),
        Column::new("name", "Libellé", ColumnWidth::Flexible { min: 200. })
            .sortable(true)
            .align(ColumnAlign::Center),
        Column::new("state", "État", ColumnWidth::Fixed(240.)).align(ColumnAlign::Right),
        Column::new("hidden", "Colonne masquée", ColumnWidth::Fixed(100.)).hidden(true),
    ])
    .unwrap()
    .max_height(300.)
    .unwrap()
}
impl Gallery {
    fn section(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let descending = matches!(
            self.sort.as_ref().map(|s| s.direction),
            Some(SortDirection::Descending)
        );
        let table_switch = self.table_switch.clone();
        let owner = cx.entity();
        match index {
            0 => (div().text_2xl().child("Design System · Développement")).into_any_element(),
            1 => (self.feedback.clone()).into_any_element(),
            2 => (div()
                .flex()
                .gap_2()
                .child(
                    Button::new(
                        "toggle-disabled",
                        if self.disabled {
                            "Activer les contrôles"
                        } else {
                            "Désactiver les contrôles"
                        },
                    )
                    .element()
                    .on_click(cx.listener(|page, _, _, cx| {
                        page.disabled = !page.disabled;
                        for input in [&page.text, &page.number] {
                            input.update(cx, |input, cx| input.set_disabled(page.disabled, cx));
                        }
                        for control in &page.switches {
                            control
                                .update(cx, |control, cx| control.set_disabled(page.disabled, cx));
                        }
                        for control in &page.selectors {
                            control
                                .update(cx, |control, cx| control.set_disabled(page.disabled, cx));
                        }
                        page.slider
                            .update(cx, |control, cx| control.set_disabled(page.disabled, cx));
                        page.shortcut
                            .update(cx, |control, cx| control.set_disabled(page.disabled, cx));
                        for control in &page.searches {
                            control
                                .update(cx, |control, cx| control.set_disabled(page.disabled, cx));
                        }
                        cx.notify();
                    })),
                )
                .child(
                    Button::new(
                        "toggle-refusal",
                        if self.refuse {
                            "Autoriser les valeurs et fermetures"
                        } else {
                            "Refuser les valeurs et fermetures"
                        },
                    )
                    .element()
                    .on_click(cx.listener(|page, _, _, cx| {
                        page.refuse = !page.refuse;
                        cx.notify();
                    })),
                ))
            .into_any_element(),
            3 => (Card::element("gallery-buttons", "Button · référence Penpot")
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap_3()
                        .child(
                            Button::new("primary", "text")
                                .variant(ButtonVariant::Primary)
                                .element(),
                        )
                        .child(Button::new("secondary", "text").element()),
                )
                .child(
                    div()
                        .text_sm()
                        .child("Survolez les boutons pour comparer les états de l’export."),
                )
                .child(
                    div()
                        .flex()
                        .gap_3()
                        .child(
                            Button::new("button-small", "Petit")
                                .size(ControlSize::Small)
                                .element(),
                        )
                        .child(
                            Button::new("button-large", "Grand")
                                .size(ControlSize::Large)
                                .element(),
                        ),
                )
                .child(
                    Button::new("disabled", "Action indisponible")
                        .disabled(true)
                        .element(),
                )
                .child(
                    Button::new("icon-only", "Informations")
                        .icon(div().child("ⓘ"))
                        .icon_only()
                        .element(),
                )
                .child(
                    NavigationButton::new("navigation-demo", "Page active")
                        .active(true)
                        .icon(div().child("⌂"))
                        .element(),
                )
                .child(
                    NavigationButton::new("navigation-disabled", "Page indisponible")
                        .disabled(true)
                        .element(),
                ))
            .into_any_element(),
            4 => (Card::interactive(
                "interactive-card",
                "Action principale de Card",
                false,
                true,
                cx.listener(|page, _, _, cx| {
                    page.feedback = "Action principale de Card".into();
                    cx.notify();
                }),
            )
            .child(
                Button::new("card-secondary", "Action enfant indépendante")
                    .element()
                    .on_click(cx.listener(|page, _, _, cx| {
                        page.feedback = "Action enfant uniquement".into();
                        cx.notify();
                    })),
            ))
            .into_any_element(),
            5 => (Card::element("gallery-fields", "TextInput et NumberInput")
                .child(
                    div()
                        .w(gpui::px(
                            crate::ui::theme::tokens::TOKEN_COMPONENTS_FIELD_WIDTH,
                        ))
                        .max_w_full()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().child("Texte (optionnel)"))
                        .child(self.text.clone()),
                )
                .child(
                    div()
                        .w(gpui::px(
                            crate::ui::theme::tokens::TOKEN_COMPONENTS_FIELD_WIDTH,
                        ))
                        .max_w_full()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().child("Nombre (optionnel)"))
                        .child(self.number.clone()),
                )
                .child(
                    Button::new(
                        "program-update",
                        "Valeur de référence → 42 (conserve le brouillon)",
                    )
                    .element()
                    .on_click(cx.listener(|page, _, _, cx| {
                        page.number
                            .update(cx, |input, cx| input.set_content("42".into(), cx).unwrap())
                    })),
                )
                .child(
                    Button::new("program-reset", "Réinitialiser le champ")
                        .element()
                        .on_click(cx.listener(|page, _, _, cx| {
                            page.number
                                .update(cx, |input, cx| input.reset("12".into(), cx).unwrap())
                        })),
                ))
            .into_any_element(),
            6 => (Card::element("gallery-segment-group", "SegmentGroup · référence Penpot")
                .child(self.selectors[0].clone()))
            .into_any_element(),
            7 => (Card::element(
                "gallery-controls",
                "Switch, Slider, sélecteurs et ShortcutSelector",
            )
            .children(self.switches.iter().cloned())
            .child(
                div()
                    .w(gpui::px(
                        crate::ui::theme::tokens::TOKEN_COMPONENTS_FIELD_WIDTH,
                    ))
                    .max_w_full()
                    .child(self.slider.clone()),
            )
            .children(self.selectors.iter().skip(1).cloned())
            .child(self.shortcut.clone()))
            .into_any_element(),
            8 => (Button::new(
                "program-controls",
                "Mettre à jour les contrôles par programmation",
            )
            .element()
            .on_click(cx.listener(|page, _, _, cx| {
                for control in &page.switches {
                    control.update(cx, |control, cx| control.set_value(true, cx));
                }
                page.slider.update(cx, |control, cx| {
                    control
                        .configure(
                            SliderState {
                                min: 0.,
                                max: 100.,
                                value: 50.,
                                steps: vec![0., 20., 50., 100.],
                                snap: true,
                                keyboard_increment: 1.,
                            },
                            cx,
                        )
                        .unwrap()
                });
                page.shortcut.update(cx, |control, cx| {
                    control
                        .set_value(
                            Some(Shortcut(gpui::Keystroke::parse("ctrl-k").unwrap())),
                            cx,
                        )
                        .unwrap()
                });
                for search in &page.searches {
                    search.update(cx, |search, cx| {
                        search.set_value("Programmé sans recherche".into(), cx)
                    });
                }
                page.feedback = "Mise à jour programmée sans événement utilisateur".into();
                cx.notify();
            })))
            .into_any_element(),
            9 => (divider()).into_any_element(),
            10 => (Card::element("gallery-time-search", "Timer et SearchBar")
                .children(self.timers.iter().cloned())
                .children(self.searches.iter().cloned().map(|search| {
                    div()
                        .w(gpui::px(
                            crate::ui::theme::tokens::TOKEN_COMPONENTS_FIELD_WIDTH,
                        ))
                        .max_w_full()
                        .child(search)
                }))
                .child(
                    Button::new("restart-timer", "Relancer le décompte")
                        .element()
                        .on_click(cx.listener(|page, _, _, cx| {
                            page.timers[0].update(cx, |timer, cx| {
                                timer.restart(Deadline::duration(Duration::from_secs(90)), cx)
                            })
                        })),
                ))
            .into_any_element(),
            11 => (Card::element(
                "gallery-table",
                "Table · données et tri détenus par la page",
            )
            .child(
                table()
                    .virtualized(
                        "gallery-virtual-table",
                        10_000,
                        48.,
                        &self.scroll,
                        Some(
                            table()
                                .header(self.sort.clone(), move |sort, _, cx| {
                                    owner.update(cx, |page, cx| {
                                        page.sort = Some(sort);
                                        cx.notify();
                                    })
                                })
                                .into_any_element(),
                        ),
                        move |range, _, _| {
                            range
                                .map(|index| {
                                    let i = if descending { 9999 - index } else { index };
                                    table()
                                        .row(vec![
                                            div().child(i.to_string()).into_any_element(),
                                            div().child(format!("Ligne {i}")).into_any_element(),
                                            if i == 0 {
                                                table_switch.clone().into_any_element()
                                            } else {
                                                div().child("Disponible").into_any_element()
                                            },
                                        ])
                                        .into_any_element()
                                })
                                .collect()
                        },
                    )
                    .unwrap(),
            ))
            .into_any_element(),
            12 => (table().ordinary(
                "gallery-ordinary-table",
                None,
                vec![
                    table()
                        .row(vec![
                            div().child("Variable").into_any_element(),
                            div()
                                .child("Ligne ordinaire avec du contenu libre sur plusieurs lignes")
                                .into_any_element(),
                            div().child("OK").into_any_element(),
                        ])
                        .into_any_element(),
                ],
                Some(
                    div()
                        .child("État de chargement, vide ou erreur fourni par la page")
                        .into_any_element(),
                ),
            ))
            .into_any_element(),
            13 => (Card::element("gallery-overlays", "Popover et Popup")
                .child(Overlay::trigger(&self.popover, "Ouvrir le Popover", cx))
                .child(Overlay::panel(
                    &self.popover,
                    div()
                        .child("Contenu composable")
                        .child(self.popover_switch.clone()),
                    window,
                    cx,
                ))
                .child(Overlay::trigger(&self.popup, "Ouvrir la Popup modale", cx))
                .child(
                    div().flex().gap_2().children(
                        [
                            Placement::Bottom,
                            Placement::Top,
                            Placement::Left,
                            Placement::Right,
                        ]
                        .into_iter()
                        .enumerate()
                        .map(|(i, placement)| {
                            Button::new(
                                format!("placement-{i}"),
                                ["Bas", "Haut", "Gauche", "Droite"][i],
                            )
                            .element()
                            .on_click(cx.listener(
                                move |page, _, _, cx| {
                                    page.popover.update(cx, |overlay, cx| {
                                        overlay.placement = placement;
                                        cx.notify();
                                    })
                                },
                            ))
                        }),
                    ),
                ))
            .into_any_element(),
            _ => unreachable!("unknown gallery section"),
        }
    }
}
impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        div()
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .p_6()
            .child(
                self.page_viewport
                    .element("gallery-scroll-area", move |index, window, cx| {
                        owner.update(cx, |page, cx| page.section(index, window, cx))
                    }),
            )
            .child(Overlay::panel(
                &self.popup,
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child("Popup · Select imbriqué")
                    .child(self.nested_select.clone())
                    .child(
                        Button::new(
                            "toggle-popup-refusal",
                            if self.refuse {
                                "Autoriser la fermeture"
                            } else {
                                "Refuser la fermeture"
                            },
                        )
                        .element()
                        .on_click(cx.listener(|page, _, _, cx| {
                            page.refuse = !page.refuse;
                            cx.notify();
                        })),
                    )
                    .child(
                        Button::new(
                            "program-close-popup",
                            "Demander la fermeture par programmation",
                        )
                        .element()
                        .on_click(cx.listener(|page, _, window, cx| {
                            page.popup.update(cx, |overlay, cx| {
                                overlay.close(CloseReason::Programmatic, window, cx)
                            })
                        })),
                    ),
                window,
                cx,
            ))
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod tests {
    use super::*;
    #[gpui::test]
    fn gallery_composes_all_controls(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            TextInput::register(cx);
            crate::ui::theme::register_fonts(cx);
        });
        let (gallery, cx) = cx.add_window_view(|_, cx| Gallery::new(cx));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|_, cx| {
            assert!(
                gallery
                    .read(cx)
                    .page_viewport
                    .state()
                    .max_offset_for_scrollbar()
                    .y
                    > gpui::px(0.)
            )
        });
    }
}
