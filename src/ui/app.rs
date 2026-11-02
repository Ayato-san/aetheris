use aetheris::game_process::{self, GameStatus};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
    time::Duration,
};

use super::components::text_input::TextInput;
use aetheris::engine_ini::{
    EngineIni, discover_engine_ini,
    hardware::{self, Gpu},
    settings::{CATEGORIES, DisplayMode, Resolution, Upscaling},
    storage, validate_engine_ini_path,
};
use aetheris::engine_ini::{numeric::NumericInput, settings::Kind};
use gpui::{
    Context, Entity, PathPromptOptions, Render, Subscription, Task, Window, div, prelude::*, rgb,
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
}

pub(crate) struct AetherisApp {
    page: Page,
    #[cfg(debug_assertions)]
    capture_debug: Entity<super::pages::combat_debug::CombatDebugPage>,
    pub(crate) game_status: GameStatus,
    engine_ini_path: Option<PathBuf>,
    engine_ini_error: Option<String>,
    pub(crate) document: Option<EngineIni>,
    pub(crate) baseline: Option<EngineIni>,
    pub(crate) gpu: Gpu,
    pub(crate) native: Option<(u64, u64)>,
    pub(crate) inputs: HashMap<String, Entity<TextInput>>,
    input_observers: Vec<Subscription>,
    pending: BTreeMap<String, (usize, Vec<String>)>,
    autosave_task: Option<Task<()>>,
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
            game_status: GameStatus::Unknown,
            engine_ini_path: discover_engine_ini(),
            engine_ini_error: None,
            document: None,
            baseline: None,
            gpu: Gpu::load(&hardware::preferences_directory()),
            native: None,
            inputs: HashMap::new(),
            input_observers: Vec::new(),
            pending: BTreeMap::new(),
            autosave_task: None,
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
            Some(directory) => match storage::load_baseline(&directory) {
                Ok(baseline) => app.baseline = Some(baseline),
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
            self.autosave_task = None;
            self.notice = None;
        }
        for input in self.inputs.values() {
            input.update(cx, |input, cx| input.set_read_only(status.is_locked(), cx));
        }
        // Read the game's latest values when it exits; pending drafts remain
        // visible and require a new edit or an explicit flush to save.
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
        self.sync_inputs(cx);
    }
    fn sync_inputs(&mut self, cx: &mut Context<Self>) {
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
            if let Some(input) = self.inputs.get(&key) {
                if !self.pending.values().any(|(_, keys)| keys.contains(&key)) {
                    input.update(cx, |input, cx| input.set_content(value, cx));
                }
            } else {
                let input = cx.new(|cx| {
                    let mut input = TextInput::new(value, placeholder, numeric, cx);
                    input.set_read_only(self.game_status.is_locked(), cx);
                    input
                });
                let input_key = key.clone();
                self.input_observers
                    .push(cx.subscribe(&input, move |app, _, _, cx| {
                        app.input_changed(index, &input_key, cx)
                    }));
                self.inputs.insert(key, input);
            }
        }
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
        self.autosave_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(2)).await;
            this.update(cx, |app, cx| {
                app.flush_pending(cx);
            })
            .ok();
        }));
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
        cx.notify();
        self.pending.is_empty()
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
        let Some(path) = self.engine_ini_path.as_deref() else {
            return false;
        };
        match storage::edit(path, change) {
            Ok(document) => {
                self.engine_ini_error = None;
                self.notice = Some("Modifications enregistrées.".into());
                self.document = Some(document);
                self.sync_inputs(cx);
                cx.notify();
                return true;
            }
            Err(error) => {
                self.engine_ini_error = Some(error);
                self.notice = None;
            }
        }
        cx.notify();
        false
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
        if self.change(change, cx) {
            self.autosave_task = None;
        } else {
            self.pending = pending;
            self.dirty_fields = dirty_fields;
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
    pub(crate) fn set_value(
        &mut self,
        index: usize,
        key: &str,
        value: String,
        cx: &mut Context<Self>,
    ) {
        self.feedback_category = Some(index);
        self.change(
            |doc| CATEGORIES[index].set_values(doc, &[(key, &value)]),
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
    pub(crate) fn set_profile(&mut self, profile: Upscaling, cx: &mut Context<Self>) {
        self.feedback_category = Some(0);
        self.change(|doc| CATEGORIES[0].set_upscaling(doc, profile), cx);
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
    pub(crate) fn select_gpu(&mut self, gpu: Gpu, cx: &mut Context<Self>) {
        if self.game_status.is_locked() {
            return;
        }
        self.feedback_category = Some(0);
        match gpu.save(&hardware::preferences_directory()) {
            Ok(()) => {
                self.gpu = gpu;
                self.engine_ini_error = None;
            }
            Err(error) => self.engine_ini_error = Some(format!("Enregistrement du GPU : {error}")),
        }
        cx.notify();
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
            .capture_key_down(cx.listener(|app, event: &gpui::KeyDownEvent, _, cx| {
                app.interaction();
                if event.keystroke.key == "escape" {
                    app.dismiss_channel_selector(cx);
                }
            }))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(BACKGROUND))
            .text_color(rgb(TEXT))
            .font_family("Arial")
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
                        Page::Home => home::render().into_any_element(),
                        Page::EngineIni => engine_ini::render(self, cx).into_any_element(),
                        #[cfg(debug_assertions)]
                        Page::CombatDebug => self.capture_debug.clone().into_any_element(),
                    }),
            )
    }
}
