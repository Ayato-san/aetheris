use gpui::{Context, Render, Window, div, prelude::*, rgb};

use super::{
    pages::home,
    sidebar,
    theme::{BACKGROUND, TEXT},
    title_bar,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Page {
    Home,
}

pub(crate) struct AetherisApp {
    page: Page,
}

impl AetherisApp {
    pub(crate) fn new() -> Self {
        Self { page: Page::Home }
    }

    pub(crate) fn navigate(&mut self, page: Page, cx: &mut Context<Self>) {
        self.page = page;
        cx.notify();
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
                    .child(home::render()),
            )
    }
}
