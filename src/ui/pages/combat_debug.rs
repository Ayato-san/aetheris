//! Debug capture UI. All capture parsing and persistence live in the library.
use crate::ui::{
    components::text_input::TextInput,
    theme::{ACCENT, DANGER, PANEL, PANEL_MUTED, TEXT_MUTED},
};
use aetheris::combat_debug::{
    self,
    live::{self, Interface, LiveHandle},
    storage::{self, DatabaseLease, Filters, Session, Store, StoredRow},
};
use gpui::{
    ClipboardItem, Context, Div, Entity, PathPromptOptions, PromptLevel, Render, Stateful, Window,
    div, prelude::*, px, rgb, uniform_list,
};
use std::{path::PathBuf, sync::Arc, time::Duration};

const PAGE_SIZE: usize = 200;

pub(crate) struct CombatDebugPage {
    path: Option<PathBuf>,
    lease: Option<Arc<DatabaseLease>>,
    sessions: Vec<Session>,
    selected: Option<i64>,
    rows: Arc<Vec<StoredRow>>,
    total: usize,
    offset: usize,
    origin_ns: Option<i128>,
    filters: Filters,
    filter_inputs: Vec<Entity<TextInput>>,
    interfaces: Vec<Interface>,
    interface: Option<Interface>,
    network_mode: bool,
    live: Option<LiveHandle>,
    live_status: String,
    busy: bool,
    error: Option<String>,
    load_token: u64,
    detail_token: u64,
    details: Arc<Vec<String>>,
}

impl CombatDebugPage {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let location = storage::default_path().and_then(|path| {
            let lease = Arc::new(DatabaseLease::acquire(&path)?);
            Ok((path, lease))
        });
        let error = location.as_ref().err().map(ToString::to_string);
        let (path, lease) = match location {
            Ok((path, lease)) => (Some(path), Some(lease)),
            Err(_) => (None, None),
        };
        let initializing = path.is_some();
        let mut page = Self {
            error,
            path,
            lease,
            sessions: Vec::new(),
            selected: None,
            rows: Arc::new(Vec::new()),
            total: 0,
            offset: 0,
            origin_ns: None,
            filters: Filters::default(),
            filter_inputs: [
                "Opcode (code ou nom documentaire)",
                "Rechercher (texte / hex)",
            ]
            .iter()
            .map(|label| cx.new(|cx| TextInput::text(label, cx)))
            .collect(),
            interfaces: Vec::new(),
            interface: None,
            network_mode: false,
            live: None,
            live_status: String::new(),
            busy: initializing,
            load_token: 0,
            detail_token: 0,
            details: Arc::new(Vec::new()),
        };
        page.reload(true, cx);
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
                if this
                    .update(cx, |page, cx| {
                        let updates = page
                            .live
                            .as_ref()
                            .map(LiveHandle::updates)
                            .unwrap_or_default();
                        let mut changed = false;
                        for update in updates {
                            page.live_status = update.status;
                            if update.error.is_some() {
                                page.error = update.error;
                            }
                            if update.finished {
                                page.live = None;
                            }
                            changed = true;
                        }
                        if changed {
                            page.reload(false, cx);
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        page
    }

    pub(crate) fn take_live(&mut self) -> Option<LiveHandle> {
        self.live.take()
    }

    fn reload(&mut self, recover: bool, cx: &mut Context<Self>) {
        if self.busy && !recover {
            return;
        }
        let Some(path) = self.path.clone() else {
            return;
        };
        let lease = self.lease.clone();
        self.load_token += 1;
        let token = self.load_token;
        let selected = self.selected;
        let filters = self.filters.clone();
        let offset = self.offset;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let _lease = lease;
                    let store = Store::open(&path)?;
                    if recover {
                        store.recover_interrupted()?;
                    }
                    let sessions = store.sessions()?;
                    let (total, rows) = if let Some(id) = selected {
                        store.rows(id, &filters, offset, PAGE_SIZE)?
                    } else {
                        (0, Vec::new())
                    };
                    let (interfaces, interface_error) = if recover {
                        match live::interfaces() {
                            Ok(interfaces) => (interfaces, None),
                            Err(error) => (
                                Vec::new(),
                                Some(format!("Liste des interfaces indisponible : {error}")),
                            ),
                        }
                    } else {
                        (Vec::new(), None)
                    };
                    let origin = if let Some(id) = selected {
                        store.capture_start_ns(id)?
                    } else {
                        None
                    };
                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>((
                        sessions,
                        total,
                        rows,
                        interfaces,
                        origin,
                        interface_error,
                    ))
                })
                .await;
            this.update(cx, |page, cx| {
                if token != page.load_token {
                    return;
                }
                if recover {
                    page.busy = false;
                }
                match result {
                    Ok((sessions, total, rows, interfaces, origin, interface_error)) => {
                        page.sessions = sessions;
                        page.total = total;
                        page.rows = Arc::new(rows);
                        page.origin_ns = origin;
                        if recover {
                            page.interfaces = interfaces;
                            if interface_error.is_some() {
                                page.error = interface_error;
                            }
                        }
                    }
                    Err(error) => page.error = Some(error.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn select_session(&mut self, id: i64, cx: &mut Context<Self>) {
        self.selected = Some(id);
        self.offset = 0;
        self.details = Arc::new(Vec::new());
        self.detail_token += 1;
        self.reload(false, cx);
    }

    fn import(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some(db_path) = self.path.clone() else {
            return;
        };
        let lease = self.lease.clone();
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Importer une capture PCAPNG".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let path = match receiver.await {
                Ok(Ok(Some(mut paths))) => paths.pop(),
                Ok(Ok(None)) => return,
                _ => {
                    this.update(cx, |page, cx| {
                        page.error = Some("Sélecteur de fichiers indisponible.".into());
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };
            let Some(path) = path else {
                return;
            };
            if !path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("pcapng"))
            {
                this.update(cx, |page, cx| {
                    page.error = Some("Choisissez un fichier .pcapng.".into());
                    cx.notify();
                })
                .ok();
                return;
            }
            if this
                .update(cx, |page, cx| {
                    page.busy = true;
                    page.error = None;
                    cx.notify();
                })
                .is_err()
            {
                return;
            }
            let result = cx
                .background_spawn(async move {
                    let _lease = lease;
                    Store::open(&db_path)?.import_file(&path)
                })
                .await;
            this.update(cx, |page, cx| {
                page.busy = false;
                match result {
                    Ok(id) => page.select_session(id, cx),
                    Err(error) => page.error = Some(error.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn reanalyze(&mut self, alignment: Option<bool>, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let (Some(id), Some(path)) = (self.selected, self.path.clone()) else {
            return;
        };
        if self
            .sessions
            .iter()
            .any(|session| session.id == id && session.status == "live")
        {
            return;
        }
        self.busy = true;
        let lease = self.lease.clone();
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let _lease = lease;
                    let mut store = Store::open(&path)?;
                    if let Some(experimental) = alignment {
                        store.reanalyze_with_alignment(id, experimental)
                    } else {
                        store.reanalyze(id, true)
                    }
                })
                .await;
            this.update(cx, |page, cx| {
                page.busy = false;
                if let Err(error) = result {
                    page.error = Some(error.to_string());
                }
                page.details = Arc::new(Vec::new());
                page.detail_token += 1;
                page.reload(false, cx);
            })
            .ok();
        })
        .detach();
    }

    fn delete(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let (Some(id), Some(path)) = (self.selected, self.path.clone()) else {
            return;
        };
        if self
            .sessions
            .iter()
            .any(|session| session.id == id && session.status == "live")
        {
            return;
        }
        let lease = self.lease.clone();
        cx.spawn_in(window, async move |this, cx| {
            let answer = cx.prompt(
                PromptLevel::Warning,
                "Supprimer définitivement cette session ?",
                Some("Tous ses paquets bruts, messages et sections décodées seront supprimés."),
                &["Annuler", "Supprimer définitivement"],
            );
            if answer.await != Ok(1) {
                return;
            }
            let result = cx
                .background_spawn(async move {
                    let _lease = lease;
                    Store::open(&path)?.delete_session(id)
                })
                .await;
            this.update(cx, |page, cx| {
                match result {
                    Ok(()) => {
                        page.selected = None;
                        page.details = Arc::new(Vec::new());
                    }
                    Err(error) => page.error = Some(error.to_string()),
                }
                page.detail_token += 1;
                page.reload(false, cx);
            })
            .ok();
        })
        .detach();
    }

    fn start_live(&mut self, cx: &mut Context<Self>) {
        if self.busy || self.live.is_some() {
            return;
        }
        let Some(path) = self.path.clone() else {
            return;
        };
        let interface = self.interface.clone();
        let Some(lease) = self.lease.clone() else {
            return;
        };
        self.busy = true;
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { LiveHandle::start(path, interface, lease) })
                .await;
            this.update(cx, |page, cx| {
                page.busy = false;
                match result {
                    Ok(handle) => {
                        let id = handle.session_id;
                        page.live = Some(handle);
                        page.live_status = "En attente des connexions AION 2".into();
                        page.select_session(id, cx);
                    }
                    Err(error) => page.error = Some(error.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn stop_live(&mut self, cx: &mut Context<Self>) {
        let Some(handle) = self.live.take() else {
            return;
        };
        self.busy = true;
        self.live_status = "Arrêt et finalisation…".into();
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_spawn(async move {
                handle.stop_and_join();
            })
            .await;
            this.update(cx, |page, cx| {
                page.busy = false;
                page.live_status = "Capture arrêtée".into();
                page.reload(false, cx);
            })
            .ok();
        })
        .detach();
    }

    fn apply_filters(&mut self, cx: &mut Context<Self>) {
        self.filters.opcode = self.filter_inputs[0].read(cx).content.to_string();
        self.filters.entity.clear();
        self.filters.search = self.filter_inputs[1].read(cx).content.to_string();
        self.offset = 0;
        self.reload(false, cx);
    }

    fn show_details(&mut self, row: StoredRow, cx: &mut Context<Self>) {
        let (Some(session), Some(path)) = (self.selected, self.path.clone()) else {
            return;
        };
        self.detail_token += 1;
        let token = self.detail_token;
        let lease = self.lease.clone();
        cx.spawn(async move |this,cx| {
            let result = cx.background_spawn(async move {
                let _lease = lease;
                let store = Store::open(&path)?;
                let packets = store.original_packets(session,&row.row.packet_numbers)?;
                let mut lines = vec![format!("{} · {} → {} · {}", row.row.kind,row.row.source,row.row.destination,row.row.status)];
                append_provenance(&row.row, &mut lines);
                for envelope in store.envelope_chain(session, &row.row)? {
                    lines.push(format!("Enveloppe source LZ4 · ligne #{} · {}", envelope.id, envelope.row.status));
                    append_provenance(&envelope.row, &mut lines);
                    lines.push(envelope.row.summary.clone());
                    hex_lines(&envelope.row.bytes, &mut lines);
                }
                for section in row.row.sections {
                    lines.push(format!("{} = {} [{}] · offset dans la ligne {} · {}",section.kind,section.value,section.confidence,section.offset,combat_debug::hex(&section.bytes)));
                }
                lines.push("Octets de la ligne :".into());
                hex_lines(&row.row.bytes,&mut lines);
                for packet in packets {
                    lines.push(format!("Paquet #{} · section {} · interface {} · liaison {} · temps {:?} ns · capturé {}/{} octets",
                        packet.number,packet.section_id,packet.interface_id,packet.link_type.number(),packet.timestamp_ns,packet.captured_len,packet.original_len));
                    hex_lines(&packet.bytes,&mut lines);
                    if let Some(timestamp) = packet.timestamp {
                        lines.push(format!("Horodatage original : {} ticks · résolution {} · décalage {} s", timestamp.ticks,timestamp.resolution,timestamp.offset_seconds));
                    }
                }
                Ok::<_,Box<dyn std::error::Error + Send + Sync>>(lines)
            }).await;
            this.update(cx, |page,cx| {
                if token != page.detail_token { return; }
                match result { Ok(lines) => page.details = Arc::new(lines), Err(error) => page.error = Some(error.to_string()) }
                cx.notify();
            }).ok();
        }).detach();
    }
}

fn append_provenance(row: &combat_debug::analysis::AnalysisRow, lines: &mut Vec<String>) {
    use combat_debug::analysis::ByteProvenance;
    lines.push(match &row.provenance {
        Some(ByteProvenance::TcpStream { offset }) => format!("Position dans le flux TCP réassemblé : {offset} (pas un offset de paquet)"),
        Some(ByteProvenance::Decompressed { envelope, offset }) => format!("Position dans la sortie décompressée de l’enveloppe {envelope} : {offset}"),
        None => "Position : paquet brut ou ancienne analyse sans provenance ; réanalyser pour la préciser".into(),
    });
}

fn hex_lines(bytes: &[u8], lines: &mut Vec<String>) {
    for (index, chunk) in bytes.chunks(16).enumerate() {
        lines.push(format!("{:06x}  {}", index * 16, combat_debug::hex(chunk)));
    }
}

fn button(id: &'static str, text: impl Into<gpui::SharedString>) -> Stateful<Div> {
    div()
        .id(id)
        .px_3()
        .py_2()
        .rounded_md()
        .bg(rgb(PANEL_MUTED))
        .text_sm()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x30374c)))
        .child(text.into())
}

fn cycle(value: &mut String, choices: &[&str]) {
    let index = choices
        .iter()
        .position(|choice| *choice == value)
        .unwrap_or(0);
    *value = choices[(index + 1) % choices.len()].into();
}

impl Render for CombatDebugPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self
            .selected
            .and_then(|id| self.sessions.iter().find(|session| session.id == id));
        let selected_live = selected.is_some_and(|session| session.status == "live");
        let rows = self.rows.clone();
        let page = cx.entity().downgrade();
        let details = self.details.clone();
        let origin_ns = self.origin_ns;
        let range_label = format!(
            "{}–{} / {}",
            if self.total == 0 { 0 } else { self.offset + 1 },
            (self.offset + self.rows.len()).min(self.total),
            self.total
        );
        div().flex().flex_col().flex_1().min_w_0().min_h_0().p_4().gap_3()
            .child(div().flex().items_center().justify_between()
                .child(div().child(div().text_2xl().child("Combat · Debug"))
                    .child(div().text_xs().text_color(rgb(TEXT_MUTED)).child("Preuves brutes et interprétations expérimentales · absent des builds de production")))
                .child(div().text_sm().text_color(rgb(ACCENT)).child(if self.busy {"Traitement…"} else {""})))
            .when_some(self.error.as_ref(), |element,error| element.child(div().text_sm().text_color(rgb(DANGER)).child(error.clone())))
            .when_some(selected.and_then(|session| session.error.as_ref()), |element,error| element.child(div().text_sm().text_color(rgb(DANGER)).child(error.clone())))
            .when(selected.is_some_and(|session|session.decoder_version != combat_debug::DECODER_VERSION), |element| element.child(div().text_xs().text_color(rgb(ACCENT))
                .child("Cette session utilise un ancien décodeur. Cliquez sur Réanalyser pour appliquer le catalogue actuel.")))
            .when(selected.is_some_and(|session|session.experimental_midstream), |element| element.child(div().text_xs().text_color(rgb(DANGER))
                .child("OFFLINE : repli sur l’alignement imposé activé pour les flux sans synchronisation automatique")))
            .child(div().flex().gap_2().items_center()
                .child(button("mode-file",if self.network_mode {"Fichier"} else {"Fichier ✓"})
                    .on_click(cx.listener(|page,_,_,cx| {page.network_mode=false;cx.notify();})))
                .child(button("mode-network",if self.network_mode {"Réseau ✓"} else {"Réseau"})
                    .on_click(cx.listener(|page,_,_,cx| {page.network_mode=true;cx.notify();})))
                .when(!self.network_mode, |element| element.child(button("import-pcapng","Importer un PCAPNG")
                    .on_click(cx.listener(|page,_,window,cx|page.import(window,cx)))))
                .when(self.network_mode, |element| element
                    .child(button("start-capture","Start").on_click(cx.listener(|page,_,_,cx|page.start_live(cx))))
                    .child(button("stop-capture","Stop").on_click(cx.listener(|page,_,_,cx|page.stop_live(cx))))
                    .child(div().text_xs().text_color(rgb(TEXT_MUTED)).child(if cfg!(windows) {self.live_status.clone()} else {"Capture réseau disponible sous Windows uniquement".into()}))))
            .when(self.network_mode, |element| element.child(
                div().id("capture-interfaces").flex().gap_2().overflow_x_scroll().flex_shrink_0()
                    .child(button("auto-interface",if self.interface.is_none() {"Interface auto ✓"} else {"Interface auto"})
                        .on_click(cx.listener(|page,_,_,cx| {if page.live.is_none() {page.interface=None;cx.notify();}})))
                    .children(self.interfaces.iter().map(|interface| {
                        let interface = interface.clone();
                        let active = self.interface.as_ref().is_some_and(|selected| selected.address==interface.address);
                        div().id(("interface",u32::from(interface.address))).px_2().py_2().rounded_md().text_xs().flex_shrink_0()
                            .bg(rgb(if active {0x30374c} else {PANEL})).cursor_pointer()
                            .child(format!("{} · {}",interface.name,interface.address))
                            .on_click(cx.listener(move |page,_,_,cx| {if page.live.is_none() {page.interface=Some(interface.clone());cx.notify();}}))
                    }))
            ))
            .child(div().id("capture-sessions").flex().gap_2().overflow_x_scroll().flex_shrink_0()
                .when(self.sessions.is_empty(), |element| element.child(div().text_sm().text_color(rgb(TEXT_MUTED)).child("Aucune session enregistrée")))
                .children(self.sessions.iter().map(|session| {
                    let id = session.id;
                    div().id(("capture-session",id as u64)).flex_shrink_0().p_2().rounded_md().cursor_pointer()
                        .bg(rgb(if self.selected==Some(id) {PANEL_MUTED} else {PANEL}))
                        .child(div().text_sm().child(format!("#{} · {} · {}",id,session.name,session.status)))
                        .child(div().text_xs().text_color(rgb(TEXT_MUTED)).child(format!("{} · {} · {} paquets / {} lignes",session.source,session.started_at_display,session.packet_count,session.row_count)))
                        .on_click(cx.listener(move |page,_,_,cx|page.select_session(id,cx)))
                })))
            .when_some(selected, |element,session| element.child(div().flex().items_center().gap_2()
                .child(button("reanalyze","Réanalyser").when(!selected_live, |element|element.on_click(cx.listener(|page,_,_,cx|page.reanalyze(None,cx)))))
                .when(session.source=="fichier", |element| {
                    let enabled = session.experimental_midstream;
                    element.child(button("experimental-alignment",if enabled {"Alignement expérimental : ON"} else {"Alignement expérimental : OFF"})
                        .on_click(cx.listener(move|page,_,_,cx|page.reanalyze(Some(!enabled),cx))))
                })
                .child(button("delete-session","Supprimer").when(!selected_live, |element|element.on_click(cx.listener(|page,_,window,cx|page.delete(window,cx)))))
                .child(div().flex_1().min_w_0().overflow_hidden().text_xs().text_color(rgb(TEXT_MUTED)).child(format!("Décodeur : {} · {}",session.decoder_version,session.fingerprint)))))
            .child(div().flex().gap_2()
                .child(button("filter-direction",format!("Sens : {}",if self.filters.direction.is_empty() {"tous"} else {&self.filters.direction}))
                    .on_click(cx.listener(|page,_,_,cx| {cycle(&mut page.filters.direction,&["","Client → serveur","Serveur → client","Inconnue"]);page.offset=0;page.reload(false,cx);})))
                .child(button("filter-protocol",format!("Protocole : {}",if self.filters.protocol.is_empty() {"tous"} else {&self.filters.protocol}))
                    .on_click(cx.listener(|page,_,_,cx| {cycle(&mut page.filters.protocol,&["","TCP","UDP","IP","Inconnu"]);page.offset=0;page.reload(false,cx);})))
                .child(button("filter-status",format!("État : {}",if self.filters.status.is_empty() {"tous"} else {&self.filters.status}))
                    .on_click(cx.listener(|page,_,_,cx| {cycle(&mut page.filters.status,&["","identifié","décodé","inconnu","incomplet / erreur","en attente","observé","non pris en charge"]);page.offset=0;page.reload(false,cx);}))))
            .child(div().flex().gap_2().children(self.filter_inputs.iter().map(|input|div().flex_1().min_w_0().child(input.clone())))
                .child(button("apply-filters","Filtrer").on_click(cx.listener(|page,_,_,cx|page.apply_filters(cx)))))
            .child(div().flex().text_xs().text_color(rgb(TEXT_MUTED))
                .child(div().w(px(105.)).child("TYPE / PAQUET"))
                .child(div().w(px(95.)).child("TEMPS (s)"))
                .child(div().w(px(150.)).child("SENS / PROTOCOLE"))
                .child(div().w(px(205.)).child("OPCODE"))
                .child(div().flex_1().child("ÉTAT · SECTIONS / CONFIANCE")))
            .child(uniform_list("capture-table",rows.len(),move |range,_,_cx| {
                range.map(|index| {
                    let stored = rows[index].clone();
                    let detail = stored.clone();
                    let page = page.clone();
                    div().id(("capture-row",stored.id as u64)).flex().items_center().h(px(42.)).px_2()
                        .border_b_1().border_color(rgb(PANEL_MUTED)).cursor_pointer()
                        .hover(|style|style.bg(rgb(PANEL)))
                        .child(div().w(px(105.)).text_xs().child(format!("{} · #{}",stored.row.kind,stored.row.packet_numbers.first().copied().unwrap_or_default())))
                        .child(div().w(px(95.)).text_xs().child(match (stored.row.timestamp_ns,origin_ns) {
                            (Some(time),Some(origin))=>format!("{:.6}",(time-origin) as f64/1_000_000_000.),
                            _=>"—".into(),
                        }))
                        .child(div().w(px(150.)).text_xs().child(format!("{} · {}",stored.row.direction,stored.row.protocol)))
                        .child(div().w(px(205.)).text_xs().overflow_hidden()
                            .child(div().child(if stored.row.opcode.is_empty() {"—".into()} else {stored.row.opcode.to_uppercase()}))
                            .child(div().child(stored.row.opcode_display_name().to_owned())))
                        .child(div().flex_1().min_w_0().text_xs().overflow_hidden().child(format!("{} · {}",stored.row.status,stored.row.summary)))
                        .on_hover(move |hover,_,cx| {if *hover {page.update(cx,|page,cx|page.show_details(detail.clone(),cx)).ok();}})
                        .into_any_element()
                }).collect::<Vec<_>>()
            }).flex_1().min_h(px(70.)))
            .child(div().flex().gap_2().items_center()
                .child(button("previous-rows","←").on_click(cx.listener(|page,_,_,cx| {page.offset=page.offset.saturating_sub(PAGE_SIZE);page.reload(false,cx);})))
                .child(div().text_xs().child(range_label))
                .child(button("next-rows","→").on_click(cx.listener(|page,_,_,cx| {if page.offset+PAGE_SIZE<page.total {page.offset+=PAGE_SIZE;page.reload(false,cx);}})))
                .child(div().flex_1())
                .child(div().text_xs().text_color(rgb(TEXT_MUTED)).child("Survolez une ligne pour voir ses octets et paquets source"))
                .child(button("copy-raw","Copier le détail").on_click(cx.listener(|page,_,_,cx| {cx.write_to_clipboard(ClipboardItem::new_string(page.details.join("\n")));}))))
            .child(uniform_list("capture-raw-details",details.len(),move |range,_,_| {
                range.map(|index| div().h(px(18.)).text_xs().font_family("monospace").child(details[index].clone())).collect::<Vec<_>>()
            }).h(px(160.)).bg(rgb(PANEL)).min_w_0())
    }
}
