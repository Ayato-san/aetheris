use crate::ui::{
    components::{Button, ScrollContainer, Switch, SwitchChanged, SwitchVariant},
    theme,
};
use aetheris::overlay::{CubePreview, CubeRuntime, Settings};
use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, rgb};

pub(crate) struct OverlayEditor {
    settings: Settings,
    toggle: Entity<Switch>,
    sound: Entity<Switch>,
    guidance: Entity<Switch>,
    runtime: Option<CubeRuntime>,
    error: Option<String>,
    last_presence: Option<String>,
    last_status: String,
    preview: Option<CubePreview>,
    preview_until: Option<std::time::Instant>,
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
        let guidance = cx.new(|cx| {
            Switch::new(
                "Guidage privé vers le cube le plus proche",
                settings.cube_guidance,
                SwitchVariant::Card,
                cx,
            )
        });
        let observers = vec![
            cx.subscribe(&guidance, |this, _, event: &SwitchChanged, cx| {
                this.change(None, None, Some(event.0), cx)
            }),
            cx.subscribe(&toggle, |this, _, event: &SwitchChanged, cx| {
                this.change(Some(event.0), None, None, cx)
            }),
            cx.subscribe(&sound, |this, _, event: &SwitchChanged, cx| {
                this.change(None, Some(event.0), None, cx)
            }),
        ];
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(250))
                    .await;
                if this
                    .update(cx, |this, cx| {
                        if this
                            .preview_until
                            .is_some_and(|until| std::time::Instant::now() >= until)
                        {
                            this.preview = None;
                            this.preview_until = None;
                            cx.notify();
                        }
                        let status = this
                            .runtime
                            .as_ref()
                            .map(CubeRuntime::status)
                            .unwrap_or_default();
                        let preview_status = this
                            .preview
                            .as_ref()
                            .map(CubePreview::status)
                            .unwrap_or_default();
                        let status = format!("{status} {preview_status}");
                        if status != this.last_status {
                            this.last_status = status;
                            cx.notify();
                        }
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
            .then(|| CubeRuntime::start(settings.cube_sound, settings.cube_guidance));
        Self {
            settings,
            toggle,
            sound,
            guidance,
            runtime,
            error,
            last_presence: None,
            last_status: String::new(),
            preview: None,
            preview_until: None,
            _observers: observers,
        }
    }
    fn change(
        &mut self,
        enabled: Option<bool>,
        sound: Option<bool>,
        guidance: Option<bool>,
        cx: &mut Context<Self>,
    ) {
        let mut settings = self.settings.clone();
        if let Some(enabled) = enabled {
            settings.hidden_cubes = enabled;
        }
        if let Some(sound) = sound {
            settings.cube_sound = sound;
        }
        if let Some(guidance) = guidance {
            settings.cube_guidance = guidance;
        }
        match Settings::path().and_then(|path| settings.save(&path)) {
            Ok(()) => {
                if enabled.is_some() {
                    self.runtime = None;
                    self.runtime = settings
                        .hidden_cubes
                        .then(|| CubeRuntime::start(settings.cube_sound, settings.cube_guidance));
                } else if let Some(runtime) = &self.runtime {
                    runtime.set_sound(settings.cube_sound);
                    runtime.set_guidance(settings.cube_guidance);
                }
                self.settings = settings;
                self.error = None;
            }
            Err(error) => {
                self.error = Some(format!("Choix non sauvegardé : {error}"));
                self.toggle.update(cx, |toggle, cx| {
                    toggle.set_value(self.settings.hidden_cubes, cx)
                });
                self.guidance.update(cx, |toggle, cx| {
                    toggle.set_value(self.settings.cube_guidance, cx)
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
            .child(self.toggle.clone()).child(self.guidance.clone()).child(self.sound.clone())
            .child(div().text_sm().text_color(rgb(theme::TEXT_MUTED)).child("La présence et le son peuvent être enregistrés. La durée indique la détection continue par Aetheris, pas l’âge réel du cube. Le retrait d’un cube ne prouve pas sa collecte."))
            .when_some(self.runtime.as_ref().map(CubeRuntime::status), |d, status| d.child(div().text_sm().text_color(rgb(theme::TEXT_MUTED)).child(status)))
            .child(Button::new("test-overlay", if self.preview.is_some() { "Arrêter le test d’affichage" } else { "Tester l’affichage pendant 60 s" }).element().on_click(cx.listener(|this, _, _, cx| {
                if this.preview.is_some() { this.preview = None; this.preview_until = None; }
                else { match CubePreview::start() {
                    Ok(preview) => { this.preview = Some(preview); this.preview_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(60)); this.error = None; }
                    Err(error) => this.error = Some(error.to_string()),
                } }
                cx.notify();
            })))
            .when_some(self.preview.as_ref().map(CubePreview::status), |d, status| d.child(div().text_sm().child(format!("Test fictif · {status}"))))
            .child(div().text_sm().text_color(rgb(theme::TEXT_MUTED)).child("Les cubes réels s’affichent en haut à gauche du jeu. Le test affiche un cube fictif à côté lorsque le jeu est au premier plan, pour vérifier l’affichage et les captures sans attendre un cube réel."))
            .when_some(self.runtime.as_ref().and_then(CubeRuntime::presence), |d, presence| d.child(div().p_4().rounded_lg().bg(rgb(theme::PANEL)).child(presence)))
            .when(!self.settings.hidden_cubes, |d| d.child(div().text_sm().child("Cubes cachés désactivés")))
            .when(self.settings.hidden_cubes, |d| d.child(Button::new("retry-overlay", "Réessayer la connexion").element().on_click(cx.listener(|this, _, _, cx| {
                this.runtime = None;
                this.runtime = Some(CubeRuntime::start(this.settings.cube_sound, this.settings.cube_guidance));
                cx.notify();
            }))))
            .when_some(self.error.clone(), |d, error| d.child(div().text_color(rgb(theme::ACCENT)).child(error)));
        ScrollContainer::new("overlay-editor", content).size_full()
    }
}
