//! Offline capture selection and immutable combat history.
use crate::ui::{
    components::{
        Button, Column, ColumnAlign, ColumnWidth, ScrollContainer, Table,
        text_input::{ContentChanged, TextInput},
    },
    theme::{ACCENT, DANGER, PANEL, TEXT_MUTED},
};
use aetheris::{
    combat_debug::storage::{self as captures, DatabaseLease, Session, Store},
    combat_history::{
        self as combat, CaptureAnalysis, Report, Selection,
        storage::{self, Entry, History},
    },
};
use gpui::{
    AnyElement, Context, Div, Entity, PathPromptOptions, Render, Stateful, Window, div, prelude::*,
    rgb,
};
use std::{path::PathBuf, sync::Arc};

pub(crate) struct CombatHistoryPage {
    path: Option<PathBuf>,
    _lease: Option<Arc<DatabaseLease>>,
    entries: Vec<Entry>,
    sessions: Vec<Session>,
    analysis: Option<Arc<CaptureAnalysis>>,
    preview: Option<Report>,
    preview_selection: Option<Selection>,
    selected_entry: Option<i64>,
    participant: Option<usize>,
    inputs: Vec<Entity<TextInput>>,
    _subscriptions: Vec<gpui::Subscription>,
    busy: bool,
    error: Option<String>,
    notice: Option<String>,
}
impl CombatHistoryPage {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let location = storage::default_path()
            .and_then(|path| Ok((path.clone(), Arc::new(DatabaseLease::acquire(&path)?))));
        let error = location.as_ref().err().map(ToString::to_string);
        let (path, lease) = match location {
            Ok((p, l)) => (Some(p), Some(l)),
            Err(_) => (None, None),
        };
        let mut page = Self {
            path,
            _lease: lease,
            entries: Vec::new(),
            sessions: Vec::new(),
            analysis: None,
            preview: None,
            preview_selection: None,
            selected_entry: None,
            participant: None,
            inputs: [
                "Identifiant de la cible",
                "Identifiant du joueur local",
                "Début (secondes)",
                "Fin (secondes)",
            ]
            .into_iter()
            .map(|label| cx.new(|cx| TextInput::text(label, cx)))
            .collect(),
            _subscriptions: Vec::new(),
            busy: false,
            error,
            notice: None,
        };
        for input in &page.inputs {
            page._subscriptions
                .push(cx.subscribe(input, |page, _, _: &ContentChanged, cx| {
                    page.preview_selection = None;
                    if page.selected_entry.is_none() {
                        page.preview = None;
                    }
                    cx.notify();
                }));
        }
        page.reload(cx);
        page
    }
    fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.path.clone() else {
            return;
        };
        self.busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let history = History::open(&path)?;
                    let capture_path = captures::default_path()?;
                    let sessions = if capture_path.exists() {
                        Store::open(&capture_path).and_then(|store| store.sessions())
                    } else {
                        Ok(Vec::new())
                    };
                    // Capture storage must not prevent reading saved reports.
                    let (sessions, capture_error) = match sessions {
                        Ok(sessions) => (sessions, None),
                        Err(error) => (
                            Vec::new(),
                            Some(format!("Captures indisponibles : {error}")),
                        ),
                    };
                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>((
                        history.entries()?,
                        sessions,
                        capture_error,
                    ))
                })
                .await;
            this.update(cx, |page, cx| {
                page.busy = false;
                match result {
                    Ok((entries, sessions, capture_error)) => {
                        page.entries = entries;
                        page.sessions = sessions;
                        page.error = capture_error;
                    }
                    Err(error) => page.error = Some(error.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
    fn analyze(&mut self, id: i64, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.error = None;
        self.notice = None;
        self.preview = None;
        self.preview_selection = None;
        self.selected_entry = None;
        self.analysis = None;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    combat::analyze_session(&Store::open(&captures::default_path()?)?, id)
                })
                .await;
            this.update(cx, |page, cx| {
                page.busy = false;
                match result {
                    Ok(analysis) => {
                        let start = "0".to_string();
                        let end = format!(
                            "{}.{:09}",
                            analysis.duration_ns / 1_000_000_000,
                            analysis.duration_ns % 1_000_000_000
                        );
                        for (index, value) in [(2, start), (3, end)] {
                            page.inputs[index].update(cx, |input, cx| {
                                let _ = input.set_content(value, cx);
                            });
                        }
                        page.analysis = Some(Arc::new(analysis));
                    }
                    Err(error) => page.error = Some(error.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
    fn prepare(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some(analysis) = &self.analysis else {
            return;
        };
        let result = (|| -> combat::Result<(Selection, Report)> {
            let target = self.inputs[0].read(cx).content.trim().parse::<u64>()?;
            let local = self.inputs[1].read(cx).content.trim().parse::<u64>()?;
            let start = seconds(&self.inputs[2].read(cx).content)?;
            let end = seconds(&self.inputs[3].read(cx).content)?;
            let selection = analysis.select(target, local, start, end)?;
            Ok((selection, analysis.report(selection)?))
        })();
        self.error = None;
        self.selected_entry = None;
        self.preview = None;
        self.preview_selection = None;
        match result {
            Ok((selection, report)) => {
                self.participant = (report.participants.len() == 1).then_some(0);
                self.preview = Some(report);
                self.preview_selection = Some(selection);
            }
            Err(error) => self.error = Some(error.to_string()),
        }
        cx.notify();
    }
    fn save(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let (Some(path), Some(analysis), Some(selection)) = (
            self.path.clone(),
            self.analysis.clone(),
            self.preview_selection,
        ) else {
            return;
        };
        self.busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { History::open(&path)?.save(&analysis, selection) })
                .await;
            this.update(cx, |page, cx| {
                page.busy = false;
                match result {
                    Ok(id) => {
                        page.notice = Some(format!("Bilan #{id} enregistré."));
                        page.preview_selection = None;
                        page.reload(cx);
                    }
                    Err(error) => page.error = Some(error.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
    fn delete(&mut self, id: i64, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some(path) = self.path.clone() else {
            return;
        };
        self.busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { History::open(&path)?.delete(id) })
                .await;
            this.update(cx, |page, cx| {
                page.busy = false;
                match result {
                    Ok(()) => {
                        if page.selected_entry == Some(id) {
                            page.selected_entry = None;
                            page.preview = None;
                        }
                        page.reload(cx);
                    }
                    Err(error) => page.error = Some(error.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
    fn import(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Importer une capture PCAPNG".into()),
        });
        self.busy = true;
        cx.spawn(async move |this, cx| {
            let result = match receiver.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.first().cloned() {
                        cx.background_spawn(async move {
                            Store::open(&captures::default_path()?)?.import_file(&path)
                        })
                        .await
                        .map(Some)
                    } else {
                        Ok(None)
                    }
                }
                Ok(Ok(None)) => Ok(None),
                _ => Err("Sélecteur de capture indisponible".into()),
            };
            this.update(cx, |page, cx| {
                page.busy = false;
                match result {
                    Ok(_) => page.reload(cx),
                    Err(error) => page.error = Some(error.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}
fn seconds(text: &str) -> combat::Result<u64> {
    let text = text.trim().replace(',', ".");
    let (whole, fraction) = text.split_once('.').unwrap_or((&text, ""));
    if fraction.len() > 9 || !fraction.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Durée invalide : au plus neuf décimales.".into());
    }
    let whole = whole.parse::<u64>()?;
    let fractional = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<u64>()? * 10u64.pow(9 - fraction.len() as u32)
    };
    whole
        .checked_mul(1_000_000_000)
        .and_then(|n| n.checked_add(fractional))
        .ok_or_else(|| "Durée hors limites".into())
}
fn button(id: impl Into<gpui::ElementId>, label: impl Into<gpui::SharedString>) -> Stateful<Div> {
    Button::new(id, label).element()
}
fn line(text: impl Into<gpui::SharedString>) -> Div {
    div().text_sm().child(text.into())
}
fn name(entity: &combat::Entity) -> String {
    entity.name.clone().unwrap_or_else(|| {
        format!(
            "Identifiant {} (occurrence {})",
            entity.key.id,
            entity.key.occurrence + 1
        )
    })
}
fn number(value: impl Into<u128>) -> String {
    let digits = value.into().to_string();
    let mut formatted = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            formatted.push('\u{202f}');
        }
        formatted.push(digit);
    }
    formatted
}
fn numeric_column(id: &'static str, label: &'static str, width: f32) -> Column {
    Column::new(id, label, ColumnWidth::Fixed(width)).align(ColumnAlign::Right)
}
fn skill_cell(skill: &combat::SkillTotal) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .py_2()
        .child(skill.label.clone())
        .child(
            line(
                skill
                    .skill_ids
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
            )
            .text_xs()
            .text_color(rgb(TEXT_MUTED)),
        )
        .into_any_element()
}
fn skill_table(player: &combat::Participant) -> Div {
    let table = Table::new(vec![
        Column::new("skill", "Compétence", ColumnWidth::Flexible { min: 200. }),
        numeric_column("damage", "Dégâts", 100.),
        numeric_column("share", "Répartition", 100.),
        numeric_column("hits", "Coups", 65.),
        numeric_column("critical", "Critiques", 70.),
        numeric_column("back", "Dos", 55.),
        numeric_column("front", "Face", 55.),
        numeric_column("perfect", "Perfect", 70.),
        numeric_column("double", "Double", 70.),
        numeric_column("multi", "Multi", 60.),
    ])
    .expect("valid damage columns")
    .fit_content();
    let rows = player
        .skills
        .iter()
        .map(|skill| {
            let totals = &skill.totals;
            let share = if player.totals.damage > 0 {
                format!(
                    "{:.1} %",
                    totals.damage as f64 / player.totals.damage as f64 * 100.
                )
            } else {
                "—".into()
            };
            let rate = |count: u64| {
                if totals.hits > 0 {
                    format!("{:.0} %", count as f64 / totals.hits as f64 * 100.)
                } else {
                    "—".into()
                }
            };
            table
                .row(vec![
                    skill_cell(skill),
                    line(number(totals.damage)).into_any_element(),
                    line(share).into_any_element(),
                    line(number(totals.hits)).into_any_element(),
                    line(rate(totals.critical)).into_any_element(),
                    line(rate(totals.back)).into_any_element(),
                    line(rate(totals.front)).into_any_element(),
                    line(rate(totals.perfect)).into_any_element(),
                    line(rate(totals.double)).into_any_element(),
                    line(rate(totals.multi)).into_any_element(),
                ])
                .into_any_element()
        })
        .collect();
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(div().text_lg().child("Dégâts par compétence"))
        .child(table.ordinary(
            "skill-damage-table",
            Some(table.header(None, |_, _, _| {}).into_any_element()),
            rows,
            None,
        ))
        .child(
            line("Indicateurs : pourcentage des coups directs reconnus de chaque compétence.")
                .text_color(rgb(TEXT_MUTED)),
        )
        .children(
            player
                .skills
                .iter()
                .map(|skill| &skill.name_provenance)
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .map(|source| line(format!("Noms : {source}")).text_color(rgb(TEXT_MUTED))),
        )
}

impl Render for CombatHistoryPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut view = div()
            .id("combat-history-content")
            .w_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_xl().child("Historique des combats"))
            .child(line(
                "Bilans autonomes conservés après suppression des captures.",
            ))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        button("history-refresh", "Actualiser").on_click(cx.listener(
                            |page, _, _, cx| {
                                if !page.busy {
                                    page.reload(cx);
                                }
                            },
                        )),
                    )
                    .child(
                        button("history-import", "Importer une capture")
                            .on_click(cx.listener(|page, _, _, cx| page.import(cx))),
                    ),
            );
        if self.busy {
            view = view.child(line("Chargement…"));
        }
        if let Some(error) = &self.error {
            view = view.child(line(error.clone()).text_color(rgb(DANGER)));
        }
        if let Some(notice) = &self.notice {
            view = view.child(line(notice.clone()).text_color(rgb(ACCENT)));
        }
        view = view.child(div().text_lg().child("Bilans enregistrés"));
        if self.entries.is_empty() {
            view = view.child(line("Aucun bilan enregistré."));
        }
        for entry in &self.entries {
            let id = entry.id;
            view = view.child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        button(
                            ("report", id as u64),
                            format!(
                                "#{} · {} · {} · {}",
                                id,
                                entry.created_at,
                                entry.report.capture_name,
                                name(&entry.report.target)
                            ),
                        )
                        .on_click(cx.listener(move |page, _, _, cx| {
                            if let Some(entry) = page.entries.iter().find(|e| e.id == id) {
                                page.preview = Some(entry.report.clone());
                                page.selected_entry = Some(id);
                                page.preview_selection = None;
                                page.participant =
                                    (entry.report.participants.len() == 1).then_some(0);
                            }
                            cx.notify();
                        })),
                    )
                    .child(
                        button(("delete-report", id as u64), "Supprimer")
                            .on_click(cx.listener(move |page, _, _, cx| page.delete(id, cx))),
                    ),
            );
        }
        view = view.child(div().text_lg().child("Nouvelle analyse hors ligne"));
        for session in &self.sessions {
            if session.status == "live" {
                continue;
            }
            let id = session.id;
            view = view.child(
                button(
                    ("analyze-capture", id as u64),
                    format!("Analyser {} · {}", session.name, session.status),
                )
                .on_click(cx.listener(move |page, _, _, cx| page.analyze(id, cx))),
            );
        }
        if let Some(analysis) = &self.analysis {
            view = view.child(line(format!(
                "{} · {} événements fiables · {} éléments à examiner",
                analysis.capture_name,
                analysis.damage.len(),
                analysis.review.len()
            )));
            for (index, entity) in analysis.entities.iter().enumerate() {
                if analysis
                    .damage
                    .iter()
                    .any(|d| d.target == entity.key || d.source == entity.key)
                {
                    let mut candidate = div().flex().flex_col().gap_1().child(
                        line(format!(
                            "{} · identifiant {} · occurrence {} · {:?} · NPC {:?} · vie {:.6} → {} s",
                            name(entity),
                            entity.key.id,
                            entity.key.occurrence + 1,
                            entity.identity,
                            entity.npc_code,
                            entity.start_ns as f64 / 1e9,
                            entity
                                .end_ns
                                .map(|n| format!("{:.6}", n as f64 / 1e9))
                                .unwrap_or_else(|| "fin non observée".into())
                        ))
                        .text_color(rgb(TEXT_MUTED)),
                    );
                    let id = entity.key.id;
                    let mut actions = div().flex().gap_2();
                    if analysis
                        .damage
                        .iter()
                        .any(|event| event.target == entity.key)
                    {
                        actions = actions.child(
                            button(("choose-target", index), "Choisir comme cible").on_click(
                                cx.listener(move |page, _, _, cx| {
                                    page.inputs[0].update(cx, |input, cx| {
                                        let _ = input.set_content(id.to_string(), cx);
                                    });
                                    page.preview_selection = None;
                                    cx.notify();
                                }),
                            ),
                        );
                    }
                    if entity.identity != combat::Identity::Npc
                        && analysis
                            .damage
                            .iter()
                            .any(|event| event.source == entity.key)
                    {
                        actions = actions.child(
                            button(("choose-local", index), "Désigner comme joueur local")
                                .on_click(cx.listener(move |page, _, _, cx| {
                                    page.inputs[1].update(cx, |input, cx| {
                                        let _ = input.set_content(id.to_string(), cx);
                                    });
                                    page.preview_selection = None;
                                    cx.notify();
                                })),
                        );
                    }
                    candidate = candidate.child(actions);
                    view = view.child(candidate);
                }
            }
            view=view.child(line("Désignez la cible et le joueur local. Fenêtre relative au début de la capture, bornes incluses. Si un identifiant est réutilisé, resserrez la fenêtre."));
            for input in &self.inputs {
                view = view.child(input.clone());
            }
            view = view.child(
                button("preview-report", "Préparer le bilan")
                    .on_click(cx.listener(|page, _, _, cx| page.prepare(cx))),
            );
            for limitation in &analysis.limitations {
                view = view.child(line(limitation.clone()).text_color(rgb(TEXT_MUTED)));
            }
        }
        if self.preview_selection.is_some() {
            view = view.child(
                button("save-report", "Enregistrer ce bilan")
                    .on_click(cx.listener(|page, _, _, cx| page.save(cx))),
            );
        }
        if let Some(report) = &self.preview {
            view = view
                .child(
                    div()
                        .text_lg()
                        .child(format!("Bilan · {}", name(&report.target))),
                )
                .child(line(format!(
                    "Fenêtre {:.9} → {:.9} s · décodeur {}",
                    report.selection.start_ns as f64 / 1e9,
                    report.selection.end_ns as f64 / 1e9,
                    report.decoder_version
                )))
                .child(line(format!(
                    "Capture : {} · {}",
                    report.capture_name, report.fingerprint
                )))
                .child(line(report.timing_precision.clone()).text_color(rgb(TEXT_MUTED)));
            if !report.review.is_empty() || !report.limitations.is_empty() {
                view = view.child(
                    line(format!(
                        "Avertissement : analyse expérimentale, {} éléments exclus à examiner.",
                        report.review.len()
                    ))
                    .text_color(rgb(DANGER)),
                );
            }
            for limitation in &report.limitations {
                view = view.child(line(limitation.clone()).text_color(rgb(TEXT_MUTED)));
            }
            for (index, player) in report.participants.iter().enumerate() {
                view = view.child(
                    button(
                        ("participant", index),
                        format!("{} · {} dégâts", name(&player.entity), player.totals.damage),
                    )
                    .on_click(cx.listener(move |page, _, _, cx| {
                        page.participant = Some(index);
                        cx.notify();
                    })),
                );
            }
            if let Some(player) = self
                .participant
                .and_then(|index| report.participants.get(index))
            {
                let t = &player.totals;
                view = view.child(
                    div()
                        .p_3()
                        .bg(rgb(PANEL))
                        .child(line(format!(
                            "{} : {} dégâts · {} coups",
                            name(&player.entity),
                            number(t.damage),
                            number(t.hits)
                        )))
                        .child(line(format!(
                            "Durée {:.9} s · DPS {}",
                            (player.last_ns - player.first_ns) as f64 / 1e9,
                            player
                                .dps
                                .map(|n| format!("{n:.2}"))
                                .unwrap_or_else(|| "indisponible (durée nulle)".into())
                        ))),
                );
                view = view.child(skill_table(player));
            } else {
                view = view.child(line("Sélectionnez un joueur pour consulter ses détails."));
            }
            #[cfg(debug_assertions)]
            {
                view = view.child(div().text_lg().child(format!(
                    "Sources non attribuées ({})",
                    report.unattributed.len()
                )));
                let mut unknown =
                    std::collections::BTreeMap::<combat::EntityKey, (u128, u64)>::new();
                for retained in &report.unattributed {
                    let total = unknown.entry(retained.event.source).or_default();
                    total.0 += u128::from(retained.event.amount);
                    total.1 += 1;
                }
                if unknown.is_empty() {
                    view = view
                        .child(line("Aucune source non attribuée.").text_color(rgb(TEXT_MUTED)));
                } else {
                    view = view.child(line("Identité ou propriétaire incertain : ces dégâts sont exclus des totaux des joueurs.").text_color(rgb(TEXT_MUTED)));
                    for (key, (damage, count)) in unknown {
                        view = view.child(line(format!(
                            "Source {} · occurrence {} · {} dégâts · {} événements",
                            key.id,
                            key.occurrence + 1,
                            number(damage),
                            number(count)
                        )));
                    }
                }
                view = view.child(
                    div()
                        .text_lg()
                        .child("À examiner (exclus des totaux fiables)"),
                );
                for review in &report.review {
                    view = view.child(
                        line(format!(
                            "{} · {}",
                            review
                                .time_ns
                                .map(|n| format!("T+{:.6} s", n as f64 / 1e9))
                                .unwrap_or_else(|| "Horodatage absent".into()),
                            review.reason
                        ))
                        .text_color(rgb(TEXT_MUTED)),
                    );
                }
            }
        }
        ScrollContainer::new("combat-history", view).size_full()
    }
}
