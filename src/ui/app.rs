use std::path::{Path, PathBuf};

use aetheris::engine_ini::{discover_engine_ini, validate_engine_ini_path};
use gpui::{Context, PathPromptOptions, Render, Window, div, prelude::*, rgb};

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
}

pub(crate) struct AetherisApp {
    page: Page,
    engine_ini_path: Option<PathBuf>,
    engine_ini_error: Option<String>,
}

impl AetherisApp {
    pub(crate) fn new() -> Self {
        Self {
            page: Page::Home,
            engine_ini_path: discover_engine_ini(),
            engine_ini_error: None,
        }
    }

    pub(crate) fn navigate(&mut self, page: Page, cx: &mut Context<Self>) {
        self.page = page;
        cx.notify();
    }

    pub(crate) fn engine_ini_path(&self) -> Option<&Path> {
        self.engine_ini_path.as_deref()
    }

    pub(crate) fn engine_ini_error(&self) -> Option<&str> {
        self.engine_ini_error.as_deref()
    }

    pub(crate) fn select_engine_ini(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
                        app.engine_ini_path = Some(path);
                        app.engine_ini_error = None;
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(BACKGROUND))
            .text_color(rgb(TEXT))
            .font_family("Arial")
            .child(title_bar::render())
            .child(
                div()
                    .flex()
                    .flex_1()
                    .child(sidebar::render(self.page, cx))
                    .child(match self.page {
                        Page::Home => home::render().into_any_element(),
                        Page::EngineIni => engine_ini::render(self, cx).into_any_element(),
                    }),
            )
    }
}
