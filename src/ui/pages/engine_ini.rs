use aetheris::engine_ini::{
    EngineIni,
    settings::{CATEGORIES, Category, DisplayMode, Kind, Setting, Upscaling},
};
use gpui::{Context, Div, Stateful, div, prelude::*, px, rgb};

use crate::ui::{
    app::AetherisApp,
    components::{Button, Card},
    theme::{DANGER, PANEL_MUTED, SUCCESS, TEXT_MUTED, WARNING},
};

pub(crate) fn render(app: &AetherisApp, cx: &mut Context<AetherisApp>) -> Stateful<Div> {
    let found = app.engine_ini_path().is_some();
    let status_label = if found {
        "Engine.ini · Trouvé"
    } else {
        "Engine.ini · Introuvable"
    };
    let status_color = if found { SUCCESS } else { DANGER };
    let status_pill = div()
        .id("engine-ini-status")
        .px_3()
        .py_2()
        .rounded_full()
        .border_1()
        .border_color(rgb(status_color))
        .text_xs()
        .text_color(rgb(status_color))
        .when(!found, |element| {
            element
                .cursor_pointer()
                .hover(|style| style.bg(rgb(PANEL_MUTED)))
                .on_click(cx.listener(|app, _, window, cx| {
                    app.select_engine_ini(window, cx);
                }))
        })
        .child(status_label);

    div()
        .id("engine-ini-page")
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .p_8()
        .gap_6()
        .child(
            div()
                .flex()
                .flex_shrink_0()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().text_3xl().child("Engine.ini"))
                        .child(
                            div()
                                .text_color(rgb(TEXT_MUTED))
                                .child("Configuration du moteur d’Aion 2"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_end()
                        .gap_2()
                        .when_some(app.engine_ini_error(), |element, error| {
                            element.child(
                                div()
                                    .max_w(px(360.0))
                                    .text_sm()
                                    .text_right()
                                    .text_color(rgb(DANGER))
                                    .child(error.to_owned()),
                            )
                        })
                        .child(status_pill)
                        .when(app.unsaved || app.gpu_unsaved, |d| {
                            d.child(
                                Button::new(
                                    "retry-engine-save",
                                    "Valeurs non enregistrées · Réessayer",
                                )
                                .element()
                                .on_click(cx.listener(
                                    |app, _, _, cx| {
                                        app.persist_accepted(cx);
                                    },
                                )),
                            )
                        })
                        .when(app.unsaved, |d| {
                            d.child(
                                Button::new(
                                    "reload-engine-file",
                                    "Recharger le fichier et abandonner les modifications",
                                )
                                .element()
                                .on_click(
                                    cx.listener(|app, _, _, cx| app.reload_discarding_changes(cx)),
                                ),
                            )
                        })
                        .when_some(app.notice.as_ref(), |element, notice| {
                            element.child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(SUCCESS))
                                    .child(notice.clone()),
                            )
                        }),
                ),
        )
        .when_some(app.game_status.lock_reason(), |d, reason| {
            d.child(div().text_sm().text_color(rgb(WARNING)).child(reason))
        })
        .child(app.engine_viewport.element("engine-scroll-area", {
            let owner = cx.entity();
            move |item, _, cx| {
                owner.update(cx, |app, cx| {
                    if item == 0 {
                        global_controls(app, cx).into_any_element()
                    } else {
                        category_card(app, item - 1, &CATEGORIES[item - 1], cx).into_any_element()
                    }
                })
            }
        }))
}

fn global_controls(app: &AetherisApp, cx: &mut Context<AetherisApp>) -> Div {
    div().flex().flex_col().flex_shrink_0().gap_3()
        .child(div().text_sm().text_color(rgb(TEXT_MUTED))
            .child("Appliquer les réglages conseillés ou rétablir l’état d’origine pour les 13 catégories."))
        .child(div().text_sm().text_color(rgb(WARNING))
            .child("Le profil conseillé réduit fortement la qualité des ombres et les détails LOD. Une distance accrue et les textures peuvent augmenter la charge et la pression sur la VRAM. Les réglages de latence impliquent un compromis avec les performances."))
        .child(div().flex().flex_wrap().gap_3()
            .child(button(app, "apply-all".into(), "Tout appliquer", false)
                .on_click(cx.listener(|app, _, _, cx| app.apply_all(cx))))
            .child(button(app, "reset-all".into(), "Tout réinitialiser", false)
                .on_click(cx.listener(|app, _, _, cx| app.reset_all(cx)))))
}

fn button(
    app: &AetherisApp,
    id: String,
    label: impl Into<gpui::SharedString>,
    active: bool,
) -> Stateful<Div> {
    Button::new(id, label)
        .selected(active)
        .disabled(app.game_status.is_locked())
        .element()
}
fn category_card(
    app: &AetherisApp,
    index: usize,
    category: &'static Category,
    cx: &mut Context<AetherisApp>,
) -> Stateful<Div> {
    let doc = app.document.as_ref().unwrap();
    Card::element(format!("category-{index}"), category.name)
        .child(div().flex().flex_wrap().items_center().justify_between().gap_3()
            .child(div().flex().gap_2()
                .child(button(app, format!("recommend-{index}"),"Appliquer les réglages conseillés",false)
                    .on_click(cx.listener(move |app,_,_,cx|app.recommended(index,cx))))
                .child(button(app, format!("reset-{index}"),"État d’origine",false)
                    .on_click(cx.listener(move |app,_,_,cx|app.reset(index,None,cx))))))
        .when_some(app.engine_ini_error().filter(|_|app.feedback_category==Some(index)), |d,error|
            d.child(div().text_sm().text_color(rgb(DANGER)).child(error.to_owned())))
        .when_some(app.notice.as_ref().filter(|_|app.feedback_category==Some(index)), |d,notice|
            d.child(div().text_xs().text_color(rgb(SUCCESS)).child(notice.clone())))
        .when(!category.warning.is_empty(),|d|d.child(div().text_sm().text_color(rgb(WARNING)).child(category.warning)))
        .when(index == 2 && doc.get(category.section,"bSmoothFrameRate").is_some_and(|v|v.eq_ignore_ascii_case("false") || v=="0"),|d|
            d.child(div().text_sm().text_color(rgb(TEXT_MUTED)).child("Lissage désactivé : les limites FPS restent modifiables mais ne devraient pas s’appliquer.")))
        .child(match index {
            0 => renderer_controls(app,doc,cx).into_any_element(),
            3 => display_controls(app,doc,cx).into_any_element(),
            5 => pair_controls(app,index,category,cx).into_any_element(),
            _ => div().flex().flex_col().gap_4()
                .children(category.settings.iter().filter(|s| !(index==2 && matches!(s.key,"MinSmoothedFrameRate"|"MaxSmoothedFrameRate")))
                    .map(|setting| setting_row(app,doc,index,category,setting,cx)))
                .when(index==2,|d|d.child(pair_controls(app,index,category,cx)))
                .into_any_element(),
        })
}
fn state(doc: &EngineIni, category: &Category, setting: &Setting) -> String {
    match doc.get(category.section, setting.key) {
        None => "Non défini".into(),
        Some(raw) if setting.display_value(raw).is_none() => {
            format!("Non reconnu : {raw} — valeur conservée")
        }
        Some(raw) => format!("Valeur actuelle : {raw}"),
    }
}
fn setting_row(
    app: &AetherisApp,
    doc: &EngineIni,
    index: usize,
    category: &Category,
    setting: &'static Setting,
    cx: &mut Context<AetherisApp>,
) -> Div {
    let key = setting.key;
    let control = match setting.kind {
        Kind::Boolean { .. } => app.switches[key].clone().into_any_element(),
        Kind::Choice(_) => app.selectors[key].clone().into_any_element(),
        _ => div()
            .flex()
            .gap_2()
            .items_center()
            .child(div().w(px(200.)).child(app.inputs[key].clone()))
            .into_any_element(),
    };
    let recommended = if matches!(setting.kind, Kind::Milliseconds) {
        "1 ms"
    } else {
        setting.recommended
    };
    div()
        .flex()
        .flex_col()
        .gap_2()
        .border_t_1()
        .border_color(rgb(PANEL_MUTED))
        .pt_3()
        .child(
            div()
                .child(div().text_sm().child(setting.description))
                .child(div().text_xs().text_color(rgb(TEXT_MUTED)).child(key)),
        )
        .child(div().text_xs().text_color(rgb(TEXT_MUTED)).child(format!(
            "{} · Conseillé : {recommended}",
            state(doc, category, setting)
        )))
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_3()
                .child(control)
                .child(
                    button(app, format!("reset-setting-{key}"), "Origine", false).on_click(
                        cx.listener(move |app, _, _, cx| app.reset(index, Some(key), cx)),
                    ),
                ),
        )
}
fn pair_controls(
    app: &AetherisApp,
    index: usize,
    category: &'static Category,
    cx: &mut Context<AetherisApp>,
) -> Div {
    let settings = if index == 2 {
        &category.settings[2..]
    } else {
        category.settings
    };
    let keys = [settings[0].key, settings[1].key];
    let doc = app.document.as_ref().unwrap();
    let values: Vec<_> = settings
        .iter()
        .map(|s| {
            doc.get(category.section, s.key)
                .and_then(|v| s.display_value(v))
        })
        .collect();
    let mut candidate = doc.clone();
    let valid = values.iter().all(Option::is_some)
        && category
            .set_values(
                &mut candidate,
                &[
                    (keys[0], values[0].as_deref().unwrap_or("")),
                    (keys[1], values[1].as_deref().unwrap_or("")),
                ],
            )
            .is_ok();
    div().flex().flex_col().gap_3()
        .when(!valid,|d|d.child(div().text_sm().text_color(rgb(TEXT_MUTED)).child("Paire non définie ou invalide — les valeurs présentes sont conservées. Renseignez les deux limites (minimum ≤ maximum).")))
        .children(settings.iter().map(|setting|div().flex().flex_col().gap_2()
            .child(div().text_sm().child(setting.key))
            .child(div().text_sm().text_color(rgb(TEXT_MUTED)).child(setting.description))
            .child(div().text_xs().text_color(rgb(TEXT_MUTED)).child(format!("{} · Conseillé : {}",state(doc,category,setting),setting.recommended)))
            .child(div().w(px(200.)).child(app.inputs[setting.key].clone()))
            .when(index==2,|d|d.child(button(app, format!("reset-fps-{}",setting.key),"Origine",false).on_click(cx.listener(move |app,_,_,cx|app.reset(index,Some(setting.key),cx)))))))
        .child(div().flex().gap_2()

            .when(index==5,|d|d.child(button(app, "reset-visibility".into(),"Origine des deux valeurs",false)
                .on_click(cx.listener(|app,_,_,cx|app.reset(5,Some(CATEGORIES[5].settings[0].key),cx))))))
}
fn renderer_controls(app: &AetherisApp, doc: &EngineIni, cx: &mut Context<AetherisApp>) -> Div {
    let profile = Upscaling::read(doc);
    let current = profile.map(|p| p.label().to_owned()).unwrap_or_else(|| {
        format!(
            "Non défini ou non reconnu — DLSS : {}, FSR : {}",
            doc.get(CATEGORIES[0].section, CATEGORIES[0].settings[0].key)
                .unwrap_or("absent"),
            doc.get(CATEGORIES[0].section, CATEGORIES[0].settings[1].key)
                .unwrap_or("absent")
        )
    });
    div().flex().flex_col().gap_3()
        .child(div().text_sm().text_color(rgb(TEXT_MUTED)).child("DLSS et FSR reconstruisent l’image à partir d’une résolution interne plus faible. Le profil règle uniquement leur activation."))
        .child(div().text_sm().child(format!("Profil actuel : {current} · Conseillé : {}",app.gpu.recommended(false,false).label())))
        .child(app.selectors["profile"].clone())
        .child(button(app, "reset-profile".into(),"Origine",false).on_click(cx.listener(|app,_,_,cx|app.reset(0,None,cx))))
        .child(div().text_sm().text_color(rgb(TEXT_MUTED)).child("Carte graphique — sélection mémorisée"))
        .child(app.selectors["gpu"].clone())
}
fn display_controls(app: &AetherisApp, doc: &EngineIni, cx: &mut Context<AetherisApp>) -> Div {
    let mode = DisplayMode::read(doc);
    let width = app
        .input("display-width", cx)
        .parse::<u64>()
        .ok()
        .or(app.native.map(|n| n.0));
    let height = app
        .input("display-height", cx)
        .parse::<u64>()
        .ok()
        .or(app.native.map(|n| n.1));
    let exceeds = match (app.native, width, height) {
        (Some((nw, nh)), Some(w), Some(h)) => w > nw || h > nh,
        _ => false,
    };
    div().flex().flex_col().gap_3()
        .child(div().text_sm().text_color(rgb(TEXT_MUTED)).child("Mode d’affichage conseillé : Sans bordures. La résolution est exprimée en pixels."))
        .when(mode.is_none(),|d|d.child(div().text_sm().text_color(rgb(TEXT_MUTED)).child(format!("Mode non défini ou non reconnu — valeurs conservées : {}, {}, {}",doc.get("SystemSettings","FullscreenMode").unwrap_or("absent"),doc.get("SystemSettings","PreferredFullscreenMode").unwrap_or("absent"),doc.get("SystemSettings","r.SetRes").unwrap_or("absent")))))
        .child(app.selectors["display"].clone())
        .child(div().text_sm().text_color(rgb(TEXT_MUTED)).child(app.native.map(|(w,h)|format!("Résolution native conseillée : {w} × {h} px")).unwrap_or_else(||"Résolution native indisponible — saisissez les dimensions.".into())))
        .when(exceeds,|d|d.child(div().text_sm().text_color(rgb(WARNING)).child("Cette résolution dépasse les dimensions natives détectées et peut augmenter la charge graphique.")))
        .child(div().flex().flex_wrap().gap_3().items_center()
            .child(div().flex().flex_col().gap_1().child("Largeur (px)").child(div().w(px(160.)).child(app.inputs["display-width"].clone())))
            .child(div().flex().flex_col().gap_1().child("Hauteur (px)").child(div().w(px(160.)).child(app.inputs["display-height"].clone())))

            .child(button(app, "reset-resolution".into(),"Origine",false).on_click(cx.listener(|app,_,_,cx|app.reset(3,None,cx)))))
}
