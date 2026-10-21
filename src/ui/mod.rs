mod app;
mod components;
mod pages;
mod sidebar;
mod theme;
mod title_bar;

use gpui::{App, AppContext, Bounds, WindowBounds, WindowDecorations, WindowOptions, px, size};
use gpui_platform::application;

use self::app::AetherisApp;

pub(crate) fn run() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.0), px(700.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Aetheris".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_decorations: Some(WindowDecorations::Client),
                ..Default::default()
            },
            |_, cx| cx.new(|_| AetherisApp::new()),
        )
        .expect("impossible d’ouvrir la fenêtre principale");

        cx.activate(true);
    });
}
