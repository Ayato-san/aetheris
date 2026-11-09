use aetheris::game_process::{self, GameStatus};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
    time::Duration,
};

use super::components::state::{Invalid, OptionItem, Selection, Validation};
use super::components::text_input::TextInput;
use super::components::{
    SelectionChanged, Selector, SelectorKind, Switch, SwitchChanged, SwitchVariant,
};

#[derive(Clone)]
enum EngineChoice {
    Setting(usize, &'static str),
    Profile,
    Display,
    Gpu,
}

use super::components::text_input::{
    ContentChanged, DraftCancelled, NumericSyntax as NumericInput, ValueChanged,
};
use aetheris::engine_ini::settings::Kind;
use aetheris::engine_ini::{
    EngineIni, discover_engine_ini,
    hardware::{self, Gpu},
    settings::{CATEGORIES, DisplayMode, Resolution, Upscaling},
    storage, validate_engine_ini_path,
};
use gpui::{
    Context, Entity, PathPromptOptions, Render, Subscription, Window, div, prelude::*, rgb,
};

use super::{
    pages::{engine_ini, home},
    sidebar,
    theme::{BACKGROUND, TEXT},
    title_bar,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Page {
    Home,
    EngineIni,
    #[cfg(debug_assertions)]
    CombatDebug,
    #[cfg(debug_assertions)]
    Gallery,
}

pub(crate) struct AetherisApp {
    page: Page,
    #[cfg(debug_assertions)]
    capture_debug: Entity<super::pages::combat_debug::CombatDebugPage>,
    #[cfg(debug_assertions)]
    gallery: Entity<super::pages::gallery::Gallery>,
    pub(crate) engine_viewport: super::components::VirtualList,
    home_viewport: super::components::VirtualList,
    pub(crate) game_status: GameStatus,
    engine_ini_path: Option<PathBuf>,
    engine_ini_error: Option<String>,
    pub(crate) document: Option<EngineIni>,
    pub(crate) baseline: Option<EngineIni>,
    pub(crate) gpu: Gpu,
    pub(crate) native: Option<(u64, u64)>,
    pub(crate) inputs: HashMap<String, Entity<TextInput>>,
    input_observers: Vec<Subscription>,
    pub(crate) switches: HashMap<String, Entity<Switch>>,
    pub(crate) selectors: HashMap<String, Entity<Selector>>,
    control_observers: Vec<Subscription>,
    pub(crate) unsaved: bool,
    pub(crate) gpu_unsaved: bool,
    persisted_document: Option<EngineIni>,
    pending: BTreeMap<String, (usize, Vec<String>)>,
    dirty_fields: BTreeSet<String>,
    pub(crate) notice: Option<String>,
    pub(crate) feedback_category: Option<usize>,
    companion_status: String,
    update_available: bool,
    interacted: bool,
    update_channel: aetheris::companion::policy::Channel,
    channel_selector_open: bool,
}

impl AetherisApp {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let mut app = Self {
            page: Page::Home,
            #[cfg(debug_assertions)]
            capture_debug: cx.new(super::pages::combat_debug::CombatDebugPage::new),
            #[cfg(debug_assertions)]
            gallery: cx.new(super::pages::gallery::Gallery::new),
            engine_viewport: super::components::VirtualList::new(0, cx)
                .gap(super::theme::SPACE * 6.),
            home_viewport: super::components::VirtualList::new(3, cx).gap(super::theme::SPACE * 6.),
            game_status: GameStatus::Unknown,
            engine_ini_path: discover_engine_ini(),
            engine_ini_error: None,
            document: None,
            baseline: None,
            gpu: Gpu::load(&hardware::preferences_directory()),
            native: None,
            inputs: HashMap::new(),
            input_observers: Vec::new(),
            switches: HashMap::new(),
            selectors: HashMap::new(),
            control_observers: Vec::new(),
            unsaved: false,
            gpu_unsaved: false,
            persisted_document: None,
            pending: BTreeMap::new(),
            dirty_fields: BTreeSet::new(),
            notice: None,
            feedback_category: None,
            companion_status: String::new(),
            update_available: false,
            interacted: false,
            update_channel: aetheris::companion::policy::Channel::Stable,
            channel_selector_open: false,
        };
        match std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
        {
            Some(programs) => match storage::baseline_directory(&programs) {
                Ok(directory)
                    if aetheris::companion::managed::is_managed(&programs)
                        && !directory.join(storage::BASELINE_NAME).exists() =>
                {
                    let receiver = cx.prompt_for_paths(PathPromptOptions {
                        files: false, directories: true, multiple: false,
                        prompt: Some("Facultatif : importer la référence d’un ancien dossier ZIP (Annuler pour ignorer)".into()),
                    });
                    cx.spawn(async move |this, cx| {
                        let selected = receiver.await;
                        let migration = match selected {
                            Ok(Ok(Some(paths))) => paths
                                .first()
                                .map(|source| storage::migrate_baseline(source, &directory))
                                .transpose(),
                            Ok(Ok(None)) => Ok(None),
                            _ => Err(std::io::Error::other("Sélecteur de dossier indisponible.")),
                        };
                        this.update(cx, |app, cx| {
                            match storage::load_baseline(&directory) {
                                Ok(baseline) => app.baseline = Some(baseline),
                                Err(error) => {
                                    app.engine_ini_error =
                                        Some(format!("Référence d’origine : {error}"))
                                }
                            }
                            if let Err(error) = migration {
                                app.notice = Some(format!("Import de la référence : {error}"));
                            }
                            cx.notify();
                        })
                        .ok();
                    })
                    .detach();
                }
                Ok(directory) => match storage::load_baseline(&directory) {
                    Ok(baseline) => app.baseline = Some(baseline),
                    Err(error) => {
                        app.engine_ini_error = Some(format!("Référence d’origine : {error}"))
                    }
                },
                Err(error) => app.engine_ini_error = Some(format!("Référence d’origine : {error}")),
            },
            None => app.engine_ini_error = Some("Dossier de l’application introuvable.".into()),
        }
        app.reload(cx);
        cx.spawn(async move |this, cx| {
            loop {
                let status = cx.background_spawn(async { game_process::detect() }).await;
                if this
                    .update(cx, |app, cx| app.update_game_status(status, cx))
                    .is_err()
                {
                    break;
                }
                cx.background_executor().timer(Duration::from_secs(1)).await;
            }
        })
        .detach();
        cx.on_app_quit(|app, cx| {
            app.flush_pending(cx);
            #[cfg(debug_assertions)]
            let capture = app
                .capture_debug
                .update(cx, |page, _| page.take_live())
                .map(|handle| {
                    cx.background_spawn(async move {
                        handle.stop_and_join();
                    })
                });
            async move {
                #[cfg(debug_assertions)]
                if let Some(capture) = capture {
                    capture.await;
                }
            }
        })
        .detach();
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(3)).await;
                let resolution = cx
                    .background_spawn(async { hardware::native_resolution() })
                    .await;
                if this
                    .update(cx, |app, cx| {
                        if app.native != resolution {
                            app.native = resolution;
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        app
    }

    fn update_game_status(&mut self, status: GameStatus, cx: &mut Context<Self>) {
        if self.game_status == status {
            return;
        }
        self.game_status = status;
        if status.is_locked() {
            self.notice = None;
        }
        for input in self.inputs.values() {
            input.update(cx, |input, cx| input.set_read_only(status.is_locked(), cx));
        }
        for control in self.switches.values() {
            control.update(cx, |control, cx| {
                control.set_disabled(status.is_locked(), cx)
            });
        }
        for control in self.selectors.values() {
            control.update(cx, |control, cx| {
                control.set_disabled(status.is_locked(), cx)
            });
        }
        // Read the game's latest values when it exits, preserving drafts.
        if status == GameStatus::Stopped {
            self.reload(cx);
        }
        cx.notify();
    }

    pub(crate) fn navigate(&mut self, page: Page, cx: &mut Context<Self>) {
        if !self.flush_pending(cx) {
            self.page = Page::EngineIni;
            cx.notify();
            return;
        }
        if page == Page::EngineIni {
            self.native = hardware::native_resolution();
            self.reload(cx);
        }
        super::components::dismiss_all(cx);
        self.page = page;
        cx.notify();
    }

    pub(crate) fn engine_ini_path(&self) -> Option<&Path> {
        self.engine_ini_path.as_deref()
    }

    pub(crate) fn engine_ini_error(&self) -> Option<&str> {
        self.engine_ini_error.as_deref()
    }

    pub(crate) fn reload(&mut self, cx: &mut Context<Self>) {
        if self.unsaved {
            return;
        }
        self.document =
            self.engine_ini_path
                .as_deref()
                .and_then(|path| match storage::read(path) {
                    Ok(doc) => Some(doc),
                    Err(error) => {
                        self.engine_ini_error = Some(error.to_string());
                        None
                    }
                });
        self.persisted_document = self.document.clone();
        self.sync_inputs(cx);
        self.sync_controls(cx);
    }
    pub(crate) fn reload_discarding_changes(&mut self, cx: &mut Context<Self>) {
        self.unsaved = false;
        self.engine_ini_error = None;
        self.notice = None;
        self.pending.clear();
        self.dirty_fields.clear();
        self.reload(cx);
        for input in self.inputs.values() {
            input.update(cx, |input, cx| input.reset_reference(cx));
        }
        cx.notify();
    }
    fn sync_inputs(&mut self, cx: &mut Context<Self>) {
        let count = if self.document.is_some() && self.baseline.is_some() {
            CATEGORIES.len() + 1
        } else {
            0
        };
        self.engine_viewport.set_count(count, cx);
        let Some(doc) = &self.document else {
            return;
        };
        let mut fields = Vec::new();
        for (index, category) in CATEGORIES.iter().enumerate() {
            for setting in category.settings {
                let numeric = match setting.kind {
                    Kind::Integer { positive } => NumericInput {
                        decimal: false,
                        signed: !positive,
                    },
                    Kind::Decimal { positive } => NumericInput {
                        decimal: true,
                        signed: !positive,
                    },
                    Kind::Milliseconds => NumericInput {
                        decimal: true,
                        signed: false,
                    },
                    _ => continue,
                };
                let value = doc
                    .get(category.section, setting.key)
                    .and_then(|raw| setting.display_value(raw))
                    .unwrap_or_default();
                let recommended = if matches!(setting.kind, Kind::Milliseconds) {
                    "1"
                } else {
                    setting.recommended
                };
                fields.push((
                    setting.key.to_owned(),
                    value,
                    format!("Conseillé : {recommended}"),
                    numeric,
                    index,
                ));
            }
        }
        let resolution = doc
            .get("SystemSettings", "r.SetRes")
            .and_then(Resolution::parse);
        for (key, value) in [
            ("display-width", resolution.map(|r| r.width.to_string())),
            ("display-height", resolution.map(|r| r.height.to_string())),
        ] {
            fields.push((
                key.to_owned(),
                value.unwrap_or_default(),
                "Pixels".into(),
                NumericInput::PIXELS,
                3,
            ));
        }
        for (key, value, placeholder, numeric, index) in fields {
            let config = super::components::state::NumberConfig {
                decimals: numeric.decimal,
                negative: numeric.signed,
                allow_empty: true,
                ..Default::default()
            };
            // A file value outside the control's range stays unknown and intact.
            let value = config.normalize(&value).unwrap_or_default();
            if let Some(input) = self.inputs.get(&key) {
                input
                    .update(cx, |input, cx| input.set_content(value, cx))
                    .expect("valid numeric reference");
            } else {
                let input = cx.new(|cx| {
                    let mut input =
                        super::components::NumberInput::create(value, placeholder, config, cx)
                            .expect("normalized numeric reference");
                    let label = if key == "display-width" {
                        "Largeur (px)"
                    } else if key == "display-height" {
                        "Hauteur (px)"
                    } else {
                        CATEGORIES[index]
                            .settings
                            .iter()
                            .find(|setting| setting.key == key)
                            .map(|setting| setting.description)
                            .unwrap_or(&key)
                    };
                    input.set_label(label.to_owned(), cx);
                    input.set_read_only(self.game_status.is_locked(), cx);
                    input
                });
                let input_key = key.clone();
                self.input_observers.push(
                    cx.subscribe(&input, move |app, _, _: &ContentChanged, cx| {
                        app.input_changed(index, &input_key, cx)
                    }),
                );
                let validated_key = key.clone();
                self.input_observers.push(cx.subscribe(
                    &input,
                    move |app, _, _: &ValueChanged, cx| {
                        app.commit_input(index, &validated_key, cx);
                    },
                ));
                let cancelled_key = key.clone();
                self.input_observers.push(cx.subscribe(
                    &input,
                    move |app, _, _: &DraftCancelled, cx| {
                        app.dirty_fields.remove(&cancelled_key);
                        if let Some((group, _)) = app
                            .pending
                            .iter()
                            .find(|(_, (_, keys))| keys.contains(&cancelled_key))
                            .map(|(g, v)| (g.clone(), v.clone()))
                            && app.pending[&group]
                                .1
                                .iter()
                                .all(|key| !app.dirty_fields.contains(key))
                        {
                            app.pending.remove(&group);
                        }
                        cx.notify();
                    },
                ));
                let owner = cx.entity().downgrade();
                let validation_key = key.clone();
                input.update(cx, |input, _| {
                    input.on_change_requested(move |candidate, cx| {
                        owner
                            .update(cx, |app, cx| {
                                app.validate_input(index, &validation_key, candidate, cx)
                            })
                            .unwrap_or_else(|_| Err("Page indisponible".into()))
                    })
                });
                self.inputs.insert(key, input);
            }
        }
    }
    fn sync_controls(&mut self, cx: &mut Context<Self>) {
        let Some(doc) = self.document.clone() else {
            return;
        };
        for (index, category) in CATEGORIES.iter().enumerate() {
            for setting in category.settings {
                let recognized = doc
                    .get(category.section, setting.key)
                    .and_then(|v| setting.display_value(v));
                match setting.kind {
                    Kind::Boolean { textual } => {
                        let value = recognized
                            .as_deref()
                            .is_some_and(|v| v == "1" || v == "True");
                        if let Some(control) = self.switches.get(setting.key) {
                            control.update(cx, |control, cx| control.set_value(value, cx));
                        } else {
                            let control = cx.new(|cx| {
                                let mut control = Switch::new(
                                    setting.description,
                                    value,
                                    SwitchVariant::Standard,
                                    cx,
                                );
                                control.set_disabled(self.game_status.is_locked(), cx);
                                control
                            });
                            let owner = cx.entity().downgrade();
                            let key = setting.key;
                            control.update(cx, |control, _| {
                                control.on_change_requested(move |value, cx| {
                                    let raw = if textual {
                                        if value { "True" } else { "False" }
                                    } else if value {
                                        "1"
                                    } else {
                                        "0"
                                    };
                                    owner
                                        .update(cx, |app, cx| {
                                            app.request_setting(index, key, raw, cx)
                                        })
                                        .unwrap_or_else(|_| Err(Invalid::from("Page indisponible")))
                                })
                            });
                            self.control_observers.push(cx.subscribe(
                                &control,
                                |app, _, _: &SwitchChanged, cx| {
                                    app.persist_accepted(cx);
                                },
                            ));
                            self.switches.insert(setting.key.into(), control);
                        }
                    }
                    Kind::Choice(choices) => self.sync_selector(
                        setting.key,
                        choices
                            .iter()
                            .map(|(value, label)| OptionItem::new(*value, *label))
                            .collect(),
                        recognized,
                        EngineChoice::Setting(index, setting.key),
                        cx,
                    ),
                    _ => {}
                }
            }
        }
        self.sync_selector(
            "profile",
            Upscaling::ALL
                .into_iter()
                .enumerate()
                .map(|(i, p)| OptionItem::new(i.to_string(), p.label()))
                .collect(),
            Upscaling::read(&doc)
                .and_then(|p| Upscaling::ALL.iter().position(|v| *v == p))
                .map(|i| i.to_string()),
            EngineChoice::Profile,
            cx,
        );
        self.sync_selector(
            "gpu",
            Gpu::ALL
                .into_iter()
                .enumerate()
                .map(|(i, p)| OptionItem::new(i.to_string(), p.label()))
                .collect(),
            Gpu::ALL
                .iter()
                .position(|v| *v == self.gpu)
                .map(|i| i.to_string()),
            EngineChoice::Gpu,
            cx,
        );
        self.sync_selector(
            "display",
            DisplayMode::ALL
                .into_iter()
                .enumerate()
                .map(|(i, p)| OptionItem::new(i.to_string(), p.label()))
                .collect(),
            DisplayMode::read(&doc)
                .and_then(|p| DisplayMode::ALL.iter().position(|v| *v == p))
                .map(|i| i.to_string()),
            EngineChoice::Display,
            cx,
        );
    }
    fn sync_selector(
        &mut self,
        key: &str,
        options: Vec<OptionItem>,
        selected: Option<String>,
        action: EngineChoice,
        cx: &mut Context<Self>,
    ) {
        let state = Selection::new(options, selected.into_iter().collect(), 0, 1)
            .expect("static Engine.ini options");
        if let Some(control) = self.selectors.get(key) {
            control
                .update(cx, |control, cx| control.configure(state, cx))
                .expect("static Engine.ini options");
            return;
        }
        let control = cx.new(|cx| {
            let mut control = Selector::new(SelectorKind::Select, "Choisir une valeur", state, cx)
                .expect("static Engine.ini options");
            control.set_disabled(self.game_status.is_locked(), cx);
            control
        });
        let owner = cx.entity().downgrade();
        let callback_action = action.clone();
        control.update(cx, |control, _| {
            control.on_change_requested(move |selection, cx| {
                let id = selection
                    .first()
                    .ok_or_else(|| Invalid::from("Choisissez une valeur"))?;
                owner
                    .update(cx, |app, cx| app.request_choice(&callback_action, id, cx))
                    .unwrap_or_else(|_| Err(Invalid::from("Page indisponible")))
            })
        });
        self.control_observers.push(cx.subscribe(
            &control,
            move |app, _, _: &SelectionChanged, cx| {
                if matches!(action, EngineChoice::Gpu) {
                    app.persist_gpu(cx);
                } else {
                    app.persist_accepted(cx);
                }
            },
        ));
        self.selectors.insert(key.into(), control);
    }
    fn request_setting(
        &mut self,
        index: usize,
        key: &str,
        value: &str,
        _: &mut Context<Self>,
    ) -> Validation<()> {
        if let Some(reason) = self.game_status.lock_reason() {
            return Err(Invalid::from(reason));
        }
        let mut candidate = self
            .document
            .clone()
            .ok_or_else(|| Invalid::from("Document indisponible"))?;
        CATEGORIES[index]
            .set_values(&mut candidate, &[(key, value)])
            .map_err(Invalid)?;
        self.document = Some(candidate);
        self.unsaved = true;
        self.feedback_category = Some(index);
        Ok(())
    }
    fn request_choice(
        &mut self,
        action: &EngineChoice,
        id: &str,
        cx: &mut Context<Self>,
    ) -> Validation<()> {
        if let Some(reason) = self.game_status.lock_reason() {
            return Err(Invalid::from(reason));
        }
        if let EngineChoice::Setting(index, key) = action {
            return self.request_setting(*index, key, id, cx);
        }
        let index = id
            .parse::<usize>()
            .map_err(|_| Invalid::from("Choix inconnu"))?;
        match action {
            EngineChoice::Gpu => {
                self.gpu = *Gpu::ALL
                    .get(index)
                    .ok_or_else(|| Invalid::from("GPU inconnu"))?;
                self.gpu_unsaved = true;
                return Ok(());
            }
            EngineChoice::Profile => {
                let profile = *Upscaling::ALL
                    .get(index)
                    .ok_or_else(|| Invalid::from("Profil inconnu"))?;
                let doc = self
                    .document
                    .as_mut()
                    .ok_or_else(|| Invalid::from("Document indisponible"))?;
                CATEGORIES[0].set_upscaling(doc, profile).map_err(Invalid)?;
            }
            EngineChoice::Display => {
                let mode = *DisplayMode::ALL
                    .get(index)
                    .ok_or_else(|| Invalid::from("Mode inconnu"))?;
                let current = self
                    .document
                    .as_ref()
                    .and_then(|doc| doc.get("SystemSettings", "r.SetRes"))
                    .and_then(Resolution::parse);
                let dimension = |key: &str, fallback: Option<u64>| -> Validation<u64> {
                    let draft = self.input(key, cx);
                    let value = if draft.is_empty() && !self.dirty_fields.contains(key) {
                        fallback
                    } else {
                        draft.parse().ok()
                    };
                    value.filter(|v| *v > 0).ok_or_else(|| {
                        Invalid::from("Renseignez les dimensions positives en pixels")
                    })
                };
                let width = dimension(
                    "display-width",
                    current.map(|r| r.width).or(self.native.map(|r| r.0)),
                )?;
                let height = dimension(
                    "display-height",
                    current.map(|r| r.height).or(self.native.map(|r| r.1)),
                )?;
                CATEGORIES[3]
                    .set_display(
                        self.document.as_mut().unwrap(),
                        Resolution {
                            width,
                            height,
                            mode,
                        },
                    )
                    .map_err(Invalid)?;
            }
            EngineChoice::Setting(..) => unreachable!(),
        }
        self.unsaved = true;
        Ok(())
    }
    fn input_changed(&mut self, index: usize, key: &str, cx: &mut Context<Self>) {
        if self.game_status.is_locked() {
            return;
        }
        let keys = if index == 5 {
            CATEGORIES[5]
                .settings
                .iter()
                .map(|s| s.key.to_owned())
                .collect()
        } else if index == 2 && matches!(key, "MinSmoothedFrameRate" | "MaxSmoothedFrameRate") {
            vec!["MinSmoothedFrameRate".into(), "MaxSmoothedFrameRate".into()]
        } else if index == 3 {
            vec!["display-width".into(), "display-height".into()]
        } else {
            vec![key.to_owned()]
        };
        self.dirty_fields.insert(key.to_owned());
        let group = keys[0].clone();
        self.pending.insert(group, (index, keys));
        self.notice = None;
        self.engine_ini_error = None;
        self.feedback_category = Some(index);
        cx.notify();
    }
    fn validate_input(
        &mut self,
        index: usize,
        key: &str,
        candidate: &str,
        cx: &Context<Self>,
    ) -> Result<String, String> {
        if let Some(reason) = self.game_status.lock_reason() {
            return Err(reason.into());
        }
        let mut document = self.document.clone().ok_or("Document indisponible")?;
        if index == 3 {
            let current = document
                .get("SystemSettings", "r.SetRes")
                .and_then(Resolution::parse);
            let dimension = |other: &str, fallback: Option<u64>| -> Result<u64, String> {
                let draft = if key == other {
                    candidate.into()
                } else {
                    self.input(other, cx)
                };
                let value =
                    if draft.is_empty() && !self.dirty_fields.contains(other) && key != other {
                        fallback
                    } else {
                        draft.parse().ok()
                    };
                value
                    .filter(|v| *v > 0)
                    .ok_or_else(|| "Renseignez les dimensions positives en pixels".into())
            };
            let width = dimension(
                "display-width",
                current.map(|r| r.width).or(self.native.map(|r| r.0)),
            )?;
            let height = dimension(
                "display-height",
                current.map(|r| r.height).or(self.native.map(|r| r.1)),
            )?;
            let mode = DisplayMode::read(&document).ok_or("Choisissez un mode d’affichage")?;
            CATEGORIES[3].set_display(
                &mut document,
                Resolution {
                    width,
                    height,
                    mode,
                },
            )?;
        } else {
            let keys: Vec<&str> = if index == 5 {
                CATEGORIES[5].settings.iter().map(|s| s.key).collect()
            } else if index == 2 && matches!(key, "MinSmoothedFrameRate" | "MaxSmoothedFrameRate") {
                vec!["MinSmoothedFrameRate", "MaxSmoothedFrameRate"]
            } else {
                vec![key]
            };
            let values: Vec<_> = keys
                .iter()
                .map(|other| {
                    (
                        *other,
                        if *other == key {
                            candidate.to_owned()
                        } else {
                            self.input(other, cx)
                        },
                    )
                })
                .collect();
            let entries: Vec<_> = values.iter().map(|(k, v)| (*k, v.as_str())).collect();
            CATEGORIES[index].set_values(&mut document, &entries)?;
        }
        self.document = Some(document);
        self.unsaved = true;
        Ok(candidate.into())
    }
    fn commit_input(&mut self, _: usize, key: &str, cx: &mut Context<Self>) {
        // The synchronous request already accepted the whole linked group.
        // Clear every peer draft against that reference before attempting I/O.
        if let Some((group, (_, keys))) = self
            .pending
            .iter()
            .find(|(_, (_, keys))| keys.iter().any(|k| k == key))
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.pending.remove(&group);
            for key in &keys {
                self.dirty_fields.remove(key);
            }
            self.sync_inputs(cx);
            for key in keys {
                if let Some(input) = self.inputs.get(&key) {
                    input.update(cx, |input, cx| input.reset_reference(cx));
                }
            }
        }
        self.persist_accepted(cx);
        cx.notify();
    }
    pub(crate) fn flush_pending(&mut self, cx: &mut Context<Self>) -> bool {
        if self.game_status.is_locked() {
            return true; // Retain drafts without blocking reading, navigation or exit.
        }
        // Snapshot groups so validation failures remain pending without blocking
        // independent valid groups. Successful saves never rebuild input entities.
        let groups = self.pending.clone();
        let mut failure = None;
        for (group, (index, keys)) in groups {
            let saved = if index == 3 {
                self.save_display(None, cx)
            } else {
                let refs: Vec<_> = keys.iter().map(String::as_str).collect();
                self.save_fields(index, &refs, cx)
            };
            if saved {
                for key in &keys {
                    self.dirty_fields.remove(key);
                    if let Some(input) = self.inputs.get(key) {
                        input.update(cx, |input, cx| input.reset_reference(cx));
                    }
                }
                self.pending.remove(&group);
            } else if failure.is_none() {
                failure = Some((self.engine_ini_error.clone(), self.feedback_category));
            }
        }
        if let Some((error, category)) = failure {
            self.engine_ini_error = error;
            self.feedback_category = category;
        }
        if !self.pending.is_empty() {
            self.notice = None;
            if self.engine_ini_error.is_none() {
                self.engine_ini_error = Some(
                    "Certaines valeurs ne sont pas valides. Corrigez-les avant de fermer.".into(),
                );
            }
        }
        let saved = self.persist_accepted(cx);
        cx.notify();
        self.pending.is_empty() && saved
    }
    fn discard_pending(&mut self, index: usize, key: Option<&str>) {
        let removed: Vec<_> = self
            .pending
            .values()
            .filter(|(category, keys)| {
                *category == index && key.is_none_or(|key| keys.iter().any(|k| k == key))
            })
            .flat_map(|(_, keys)| keys.iter().cloned())
            .collect();
        for key in removed {
            self.dirty_fields.remove(&key);
        }
        self.pending.retain(|_, (category, keys)| {
            *category != index || key.is_some_and(|key| !keys.iter().any(|k| k == key))
        });
    }
    pub(crate) fn prepare_close(&mut self, cx: &mut Context<Self>) -> bool {
        if self.flush_pending(cx) {
            aetheris::companion::client::authorize_close();
            true
        } else {
            self.page = Page::EngineIni;
            cx.notify();
            false
        }
    }
    pub(crate) fn companion_command(
        &mut self,
        command: aetheris::companion::client::Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use aetheris::companion::client::{self, Command, Event};
        match command {
            Command::Activate => window.activate_window(),
            Command::Channel(channel) => {
                self.update_channel = channel;
                cx.notify();
            }
            Command::Status { message, available } => {
                self.companion_status = message;
                self.update_available = available;
                cx.notify();
            }
            Command::CloseForUpdate { unattended } => {
                if (!unattended || !self.interacted) && self.prepare_close(cx) {
                    window.remove_window();
                } else {
                    client::send(Event::CloseBlocked);
                }
            }
        }
    }

    pub(crate) fn toggle_channel_selector(&mut self, cx: &mut Context<Self>) {
        self.channel_selector_open = !self.channel_selector_open;
        cx.notify();
    }

    pub(crate) fn dismiss_channel_selector(&mut self, cx: &mut Context<Self>) {
        if self.channel_selector_open {
            self.channel_selector_open = false;
            cx.notify();
        }
    }

    pub(crate) fn select_update_channel(
        &mut self,
        channel: aetheris::companion::policy::Channel,
        cx: &mut Context<Self>,
    ) {
        aetheris::companion::client::send(aetheris::companion::client::Event::Channel(channel));
        self.dismiss_channel_selector(cx);
    }

    fn interaction(&mut self) {
        if !self.interacted {
            self.interacted = true;
            aetheris::companion::client::send(aetheris::companion::client::Event::Interaction);
        }
    }
    pub(crate) fn input(&self, key: &str, cx: &Context<Self>) -> String {
        self.inputs
            .get(key)
            .map(|v| v.read(cx).content.to_string())
            .unwrap_or_default()
    }
    fn change(
        &mut self,
        change: impl FnOnce(&mut EngineIni) -> Result<(), String>,
        cx: &mut Context<Self>,
    ) -> bool {
        self.update_game_status(game_process::detect(), cx);
        if let Some(reason) = self.game_status.lock_reason() {
            self.engine_ini_error = Some(reason.into());
            self.notice = None;
            cx.notify();
            return false;
        }
        if self.baseline.is_none() {
            return false;
        }
        if self.engine_ini_path.is_none() {
            return false;
        }
        let Some(mut candidate) = self.document.clone() else {
            return false;
        };
        if let Err(error) = change(&mut candidate) {
            self.engine_ini_error = Some(error);
            self.notice = None;
            cx.notify();
            return false;
        }
        self.document = Some(candidate);
        self.unsaved = true;
        self.sync_inputs(cx);
        self.sync_controls(cx);
        self.persist_accepted(cx)
    }
    fn persist_gpu(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.gpu_unsaved {
            return true;
        }
        match self.gpu.save(&hardware::preferences_directory()) {
            Ok(()) => {
                self.gpu_unsaved = false;
                self.engine_ini_error = None;
                cx.notify();
                true
            }
            Err(error) => {
                self.engine_ini_error = Some(format!("GPU validé, non enregistré : {error}"));
                cx.notify();
                false
            }
        }
    }
    pub(crate) fn persist_accepted(&mut self, cx: &mut Context<Self>) -> bool {
        let gpu_saved = self.persist_gpu(cx);
        if !self.unsaved {
            return gpu_saved;
        }
        let status = game_process::detect();
        self.update_game_status(status, cx);
        if let Some(reason) = status.lock_reason() {
            self.engine_ini_error = Some(reason.into());
            self.notice = None;
            cx.notify();
            return false;
        }
        let Some(path) = self.engine_ini_path.as_deref() else {
            return false;
        };
        let Some(document) = self.document.clone() else {
            return false;
        };
        let expected = self.persisted_document.clone();
        match storage::edit(path, move |target| {
            if expected.as_ref().is_some_and(|expected| expected != target) {
                return Err(
                    "Engine.ini a changé sur disque. Rechargez le fichier avant de recommencer."
                        .into(),
                );
            }
            *target = document;
            Ok(())
        }) {
            Ok(document) => {
                self.persisted_document = Some(document.clone());
                self.document = Some(document);
                self.unsaved = false;
                if gpu_saved {
                    self.engine_ini_error = None;
                    self.notice = Some("Modifications enregistrées.".into());
                } else {
                    self.notice = None;
                }
                self.sync_inputs(cx);
                self.sync_controls(cx);
                cx.notify();
                gpu_saved
            }
            Err(error) => {
                self.engine_ini_error =
                    Some(format!("Valeurs validées, non enregistrées : {error}"));
                self.notice = None;
                cx.notify();
                false
            }
        }
    }
    fn change_replacing(
        &mut self,
        index: usize,
        key: Option<&str>,
        change: impl FnOnce(&mut EngineIni) -> Result<(), String>,
        cx: &mut Context<Self>,
    ) {
        let pending = self.pending.clone();
        let dirty_fields = self.dirty_fields.clone();
        self.discard_pending(index, key);
        if !self.change(change, cx) {
            self.pending = pending;
            self.dirty_fields = dirty_fields;
        } else {
            let keys: Vec<_> = self
                .inputs
                .keys()
                .filter(|input_key| {
                    if index == 3 {
                        input_key.starts_with("display-")
                    } else {
                        CATEGORIES[index].settings.iter().any(|setting| {
                            setting.key == input_key.as_str()
                                && key.is_none_or(|key| key == input_key.as_str())
                        })
                    }
                })
                .cloned()
                .collect();
            for key in keys {
                self.inputs[&key].update(cx, |input, cx| input.reset_reference(cx));
            }
        }
    }
    fn change_all(
        &mut self,
        change: impl FnOnce(&mut EngineIni) -> Result<(), String>,
        cx: &mut Context<Self>,
    ) {
        self.feedback_category = None;
        let pending = std::mem::take(&mut self.pending);
        let dirty_fields = std::mem::take(&mut self.dirty_fields);
        if !self.change(change, cx) {
            self.pending = pending;
            self.dirty_fields = dirty_fields;
        } else {
            for input in self.inputs.values() {
                input.update(cx, |input, cx| input.reset_reference(cx));
            }
        }
    }
    pub(crate) fn apply_all(&mut self, cx: &mut Context<Self>) {
        let native = self.native;
        let profile = self.gpu.recommended(false, false);
        self.change_all(
            |doc| aetheris::engine_ini::settings::apply_all_recommended(doc, native, profile),
            cx,
        );
    }
    pub(crate) fn reset_all(&mut self, cx: &mut Context<Self>) {
        let Some(baseline) = self.baseline.clone() else {
            return;
        };
        self.change_all(
            |doc| {
                aetheris::engine_ini::settings::reset_all(doc, &baseline);
                Ok(())
            },
            cx,
        );
    }
    pub(crate) fn recommended(&mut self, index: usize, cx: &mut Context<Self>) {
        self.feedback_category = Some(index);
        let native = self.native;
        let profile = self.gpu.recommended(false, false);
        self.change_replacing(
            index,
            None,
            |doc| CATEGORIES[index].apply_recommended(doc, native, profile),
            cx,
        );
    }
    pub(crate) fn reset(&mut self, index: usize, key: Option<&str>, cx: &mut Context<Self>) {
        self.feedback_category = Some(index);
        let Some(baseline) = self.baseline.clone() else {
            return;
        };
        self.change_replacing(
            index,
            key,
            |doc| match key {
                Some(key) => CATEGORIES[index].reset_setting(doc, &baseline, key),
                None => {
                    CATEGORIES[index].reset(doc, &baseline);
                    Ok(())
                }
            },
            cx,
        );
    }
    pub(crate) fn save_fields(
        &mut self,
        index: usize,
        keys: &[&str],
        cx: &mut Context<Self>,
    ) -> bool {
        self.feedback_category = Some(index);
        let values: Vec<_> = keys.iter().map(|k| (*k, self.input(k, cx))).collect();
        let entries: Vec<_> = values.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.change(|doc| CATEGORIES[index].set_values(doc, &entries), cx)
    }
    pub(crate) fn save_display(
        &mut self,
        mode: Option<DisplayMode>,
        cx: &mut Context<Self>,
    ) -> bool {
        self.feedback_category = Some(3);
        let result = (|| {
            let current = self
                .document
                .as_ref()
                .and_then(|doc| doc.get("SystemSettings", "r.SetRes"))
                .and_then(Resolution::parse);
            let read_dimension = |key: &str, suggested: Option<u64>| -> Result<u64, String> {
                let value = self.input(key, cx);
                let number = if value.trim().is_empty() {
                    if self.dirty_fields.contains(key) {
                        return Err("Les dimensions doivent être des entiers positifs.".into());
                    }
                    suggested.ok_or("Renseignez les dimensions en pixels.")?
                } else {
                    value
                        .trim()
                        .parse::<u64>()
                        .map_err(|_| "Dimensions invalides.")?
                };
                if number == 0 {
                    return Err("Les dimensions doivent être positives.".into());
                }
                Ok(number)
            };
            let width = read_dimension(
                "display-width",
                current.map(|r| r.width).or(self.native.map(|n| n.0)),
            )?;
            let height = read_dimension(
                "display-height",
                current.map(|r| r.height).or(self.native.map(|n| n.1)),
            )?;
            let mode = mode
                .or_else(|| self.document.as_ref().and_then(DisplayMode::read))
                .ok_or("Choisissez un mode d’affichage.")?;
            Ok(Resolution {
                width,
                height,
                mode,
            })
        })();
        match result {
            Ok(resolution) => {
                let saved = self.change(|doc| CATEGORIES[3].set_display(doc, resolution), cx);
                if saved {
                    self.discard_pending(3, None);
                }
                saved
            }
            Err(error) => {
                self.engine_ini_error = Some(error);
                self.notice = None;
                cx.notify();
                false
            }
        }
    }
    pub(crate) fn select_engine_ini(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.flush_pending(cx) {
            return;
        }
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Sélectionner Engine.ini".into()),
        });

        cx.spawn_in(window, async move |this, cx| {
            let selected = match receiver.await {
                Ok(Ok(Some(mut paths))) => paths.pop(),
                Ok(Ok(None)) => return,
                _ => {
                    this.update(cx, |app, cx| {
                        app.engine_ini_error = Some(
                            "Impossible d’ouvrir le sélecteur de fichiers système.".to_owned(),
                        );
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };

            let Some(path) = selected else {
                return;
            };

            this.update(cx, |app, cx| {
                match validate_engine_ini_path(path) {
                    Ok(path) => {
                        app.inputs.clear();
                        app.input_observers.clear();
                        app.switches.clear();
                        app.selectors.clear();
                        app.control_observers.clear();
                        app.engine_ini_path = Some(path);
                        app.engine_ini_error = None;
                        app.reload(cx);
                    }
                    Err(error) => app.engine_ini_error = Some(error.to_owned()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

impl Render for AetherisApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("application")
            .capture_any_mouse_down(cx.listener(|app, _, _, _| app.interaction()))
            .capture_key_down(cx.listener(|app, _: &gpui::KeyDownEvent, _, _| app.interaction()))
            .on_key_down(cx.listener(|app, event: &gpui::KeyDownEvent, window, cx| {
                if event.keystroke.key == "tab" {
                    if event.keystroke.modifiers.shift {
                        window.focus_prev(cx);
                    } else {
                        window.focus_next(cx);
                    }
                    cx.stop_propagation();
                } else if event.keystroke.key == "escape" {
                    app.dismiss_channel_selector(cx);
                }
            }))
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(BACKGROUND))
            .text_color(rgb(TEXT))
            .font_family(super::theme::FONT_FAMILY)
            .child(title_bar::render(self.game_status, window, cx))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .px_4()
                    .py_2()
                    .text_xs()
                    .child(self.companion_status.clone())
                    .when(self.update_available, |element| {
                        element.child(
                            div()
                                .id("install-update")
                                .cursor_pointer()
                                .child("Mettre à jour / réessayer")
                                .on_click(|_, _, _| {
                                    aetheris::companion::client::send(
                                        aetheris::companion::client::Event::Update,
                                    )
                                }),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(sidebar::render(
                        self.page,
                        self.update_channel,
                        self.channel_selector_open,
                        cx,
                    ))
                    .child(match self.page {
                        Page::Home => self
                            .home_viewport
                            .element("home-viewport", |section, _, _| home::section(section))
                            .p_8()
                            .into_any_element(),
                        Page::EngineIni => engine_ini::render(self, cx).into_any_element(),
                        #[cfg(debug_assertions)]
                        Page::CombatDebug => self.capture_debug.clone().into_any_element(),
                        #[cfg(debug_assertions)]
                        Page::Gallery => self.gallery.clone().into_any_element(),
                    }),
            )
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod performance_tests {
    use super::*;
    #[gpui::test]
    #[ignore = "manual Engine.ini frame timing; set XDG_DATA_HOME and XDG_CONFIG_HOME to temporary directories"]
    fn engine_frame_timing(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            super::super::theme::register_fonts(cx);
            TextInput::register(cx);
        });
        let (app, cx) = cx.add_window_view(|_, cx| {
            let mut app = AetherisApp::new(cx);
            let mut doc = EngineIni::parse("");
            aetheris::engine_ini::settings::apply_all_recommended(
                &mut doc,
                Some((1920, 1080)),
                Upscaling::Off,
            )
            .unwrap();
            app.engine_ini_path = None;
            app.baseline = Some(doc.clone());
            app.document = Some(doc);
            app.page = Page::EngineIni;
            app.sync_inputs(cx);
            app.sync_controls(cx);
            app
        });
        let started = std::time::Instant::now();
        for index in 0..30 {
            cx.update(|window, cx| {
                let handle = app.read(cx).engine_viewport.state();
                handle.set_offset_from_scrollbar(gpui::point(
                    gpui::px(0.),
                    gpui::px(-(index as f32 * 30.)),
                ));
                window.refresh();
                window.draw(cx).clear(cx);
            });
        }
        eprintln!("ENGINE 30 frames: {:?}", started.elapsed());
    }
}
