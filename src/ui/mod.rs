mod app;
mod components;
mod pages;
mod sidebar;
mod theme;
mod title_bar;

use gpui::{App, AppContext, Bounds, WindowBounds, WindowDecorations, WindowOptions, px, size};
use gpui_platform::application;

use self::app::AetherisApp;

const APP_ICON_PNG: &[u8] = include_bytes!("../../resources/aetheris.png");

pub(crate) fn run() {
    application().run(|cx: &mut App| {
        components::text_input::TextInput::register(cx);
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
                icon: Some(std::sync::Arc::new(
                    image::load_from_memory_with_format(APP_ICON_PNG, image::ImageFormat::Png)
                        .expect("icône PNG d’Aetheris invalide")
                        .into_rgba8(),
                )),
                ..Default::default()
            },
            |window, cx| {
                let app = cx.new(AetherisApp::new);
                let weak = app.downgrade();
                window.on_window_should_close(cx, move |window, cx| {
                    // Finish the current input/event cycle before validating drafts.
                    // Remove through GPUI so native close also follows its window
                    // lifecycle and last-window shutdown path.
                    let weak = weak.clone();
                    window.defer(cx, move |window, cx| {
                        if weak
                            .update(cx, |app, cx| app.prepare_close(cx))
                            .unwrap_or(true)
                        {
                            window.remove_window();
                        }
                    });
                    false
                });
                app
            },
        )
        .expect("impossible d’ouvrir la fenêtre principale");

        cx.activate(true);
    });
}
