use crate::ui::{
    components::{Button, ScrollContainer, Switch, SwitchChanged, SwitchVariant},
    theme,
};
use aetheris::overlay::{CubeRuntime, Settings};
use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, rgb};

pub(crate) struct OverlayEditor {
    settings: Settings,
    toggle: Entity<Switch>,
    sound: Entity<Switch>,
    runtime: Option<CubeRuntime>,
    error: Option<String>,
    last_presence: Option<String>,
    _observers: Vec<Subscription>,
}
impl OverlayEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let loaded = Settings::path().and_then(|path| Settings::load(&path));
        let error = loaded.as_ref().err().map(ToString::to_string);
        let settings = loaded.unwrap_or_default();
        let toggle = cx.new(|cx| {
            Switch::new(
                "Cubes cachés · présence détectée",
                settings.hidden_cubes,
                SwitchVariant::Card,
                cx,
            )
        });
        let sound = cx.new(|cx| {
            Switch::new(
                "Son à la première détection de chaque cube",
                settings.cube_sound,
                SwitchVariant::Standard,
                cx,
            )
        });
        let observers = vec![
            cx.subscribe(&toggle, |this, _, event: &SwitchChanged, cx| {
                this.change(Some(event.0), None, cx)
            }),
            cx.subscribe(&sound, |this, _, event: &SwitchChanged, cx| {
                this.change(None, Some(event.0), cx)
            }),
        ];
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(250))
                    .await;
                if this
                    .update(cx, |this, cx| {
                        let presence = this.runtime.as_ref().and_then(CubeRuntime::presence);
                        if presence != this.last_presence {
                            this.last_presence = presence;
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
        let runtime = settings
            .hidden_cubes
            .then(|| CubeRuntime::start(settings.cube_sound));
        Self {
            settings,
            toggle,
            sound,
            runtime,
            error,
            last_presence: None,
            _observers: observers,
        }
    }
    fn change(&mut self, enabled: Option<bool>, sound: Option<bool>, cx: &mut Context<Self>) {
        let mut settings = self.settings.clone();
        if let Some(enabled) = enabled {
            settings.hidden_cubes = enabled;
        }
        if let Some(sound) = sound {
            settings.cube_sound = sound;
        }
        match Settings::path().and_then(|path| settings.save(&path)) {
            Ok(()) => {
                if enabled.is_some() {
                    self.runtime = None;
                    self.runtime = settings
                        .hidden_cubes
                        .then(|| CubeRuntime::start(settings.cube_sound));
                } else if let Some(runtime) = &self.runtime {
                    runtime.set_sound(settings.cube_sound);
                }
                self.settings = settings;
                self.error = None;
            }
            Err(error) => {
                self.error = Some(format!("Choix non sauvegardé : {error}"));
                self.toggle.update(cx, |toggle, cx| {
                    toggle.set_value(self.settings.hidden_cubes, cx)
                });
                self.sound.update(cx, |toggle, cx| {
                    toggle.set_value(self.settings.cube_sound, cx)
                });
            }
        }
        cx.notify();
    }
}
impl Render for OverlayEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = div().flex().flex_col().gap_4().p_6()
            .child(div().text_3xl().child("Overlay Editor"))
            .child(div().text_color(rgb(theme::TEXT_MUTED)).child("Choisissez les widgets à afficher en jeu."))
            .child(self.toggle.clone()).child(self.sound.clone())
            .child(div().text_sm().text_color(rgb(theme::TEXT_MUTED)).child("La présence et le son peuvent être enregistrés. La durée indique la détection continue par Aetheris, pas l’âge réel du cube. Le retrait d’un cube ne prouve pas sa collecte."))
            .when_some(self.runtime.as_ref().and_then(CubeRuntime::presence), |d, presence| d.child(div().p_4().rounded_lg().bg(rgb(theme::PANEL)).child(presence)))
            .when(!self.settings.hidden_cubes, |d| d.child(div().text_sm().child("Cubes cachés désactivés")))
            .when(self.settings.hidden_cubes, |d| d.child(Button::new("retry-overlay", "Réessayer la connexion").element().on_click(cx.listener(|this, _, _, cx| {
                this.runtime = None;
                this.runtime = Some(CubeRuntime::start(this.settings.cube_sound));
                cx.notify();
            }))))
            .when_some(self.error.clone(), |d, error| d.child(div().text_color(rgb(theme::ACCENT)).child(error)));
        ScrollContainer::new("overlay-editor", content).size_full()
    }
}
