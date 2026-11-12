use crate::ui::{
    components::{Button, ScrollContainer, Switch, SwitchChanged, SwitchVariant},
    theme,
};
use aetheris::overlay::{ORIENTATION_UNAVAILABLE, Runtime, Settings};
use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, rgb};

pub(crate) struct OverlayEditor {
    settings: Settings,
    toggle: Entity<Switch>,
    runtime: Option<Runtime>,
    error: Option<String>,
    last_status: String,
    _observer: Subscription,
}
impl OverlayEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let loaded = Settings::path().and_then(|path| Settings::load(&path));
        let error = loaded.as_ref().err().map(ToString::to_string);
        let settings = loaded.unwrap_or_default();
        let toggle = cx.new(|cx| {
            Switch::new(
                "Boussole d’orientation du personnage",
                settings.compass,
                SwitchVariant::Card,
                cx,
            )
        });
        let observer = cx.subscribe(&toggle, |this, _, event: &SwitchChanged, cx| {
            let settings = Settings { compass: event.0 };
            match Settings::path().and_then(|path| settings.save(&path)) {
                Ok(()) => {
                    this.settings = settings;
                    this.error = None;
                    this.runtime = this.settings.compass.then(Runtime::start);
                }
                Err(error) => {
                    this.error = Some(format!("Activation non sauvegardée : {error}"));
                    this.toggle
                        .update(cx, |toggle, cx| toggle.set_value(this.settings.compass, cx));
                }
            }
            cx.notify();
        });
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(250))
                    .await;
                if this
                    .update(cx, |this, cx| {
                        let status = this
                            .runtime
                            .as_ref()
                            .map(Runtime::status)
                            .unwrap_or_default();
                        if status != this.last_status {
                            this.last_status = status;
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
        let runtime = settings.compass.then(Runtime::start);
        Self {
            settings,
            toggle,
            runtime,
            error,
            last_status: String::new(),
            _observer: observer,
        }
    }
}
impl Render for OverlayEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = div().flex().flex_col().gap_4().p_6()
            .child(div().text_3xl().child("Overlay Editor"))
            .child(div().text_color(rgb(theme::TEXT_MUTED)).child("Choisissez les widgets à afficher en jeu."))
            .child(self.toggle.clone())
            .child(div().text_sm().child("Aperçu"))
            .child(div().p_4().rounded_lg().bg(rgb(theme::PANEL)).child(ORIENTATION_UNAVAILABLE))
            .child(div().text_sm().text_color(rgb(theme::TEXT_MUTED)).child("La source d’orientation et les directions cardinales restent à valider en jeu. La bande s’affiche sous Windows, en haut au centre du jeu au premier plan, et laisse passer les clics."))
            .child(div().text_sm().child(self.runtime.as_ref().map(Runtime::status).unwrap_or_else(|| "Boussole désactivée".into())))
            .when(self.settings.compass, |d| d.child(Button::new("retry-overlay", "Réessayer la connexion").element().on_click(cx.listener(|this, _, _, cx| {
                this.runtime = None;
                this.runtime = Some(Runtime::start());
                cx.notify();
            }))))
            .when_some(self.error.clone(), |d, error| d.child(div().text_color(rgb(theme::ACCENT)).child(error)));
        ScrollContainer::new("overlay-editor", content).size_full()
    }
}
