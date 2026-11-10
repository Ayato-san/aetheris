use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct SearchIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl SearchIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for SearchIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(37.375, 21.125));
                path.cubic_bezier_to(
                    view.point(32.6155, 9.63451),
                    view.point(37.375, 16.8152),
                    view.point(35.663, 12.682),
                );
                path.cubic_bezier_to(
                    view.point(21.125, 4.875),
                    view.point(29.568, 6.58705),
                    view.point(25.4348, 4.875),
                );
                path.cubic_bezier_to(
                    view.point(9.63451, 9.63451),
                    view.point(16.8152, 4.875),
                    view.point(12.682, 6.58705),
                );
                path.cubic_bezier_to(
                    view.point(4.875, 21.125),
                    view.point(6.58705, 12.682),
                    view.point(4.875, 16.8152),
                );
                path.cubic_bezier_to(
                    view.point(9.63451, 32.6155),
                    view.point(4.875, 25.4348),
                    view.point(6.58705, 29.568),
                );
                path.cubic_bezier_to(
                    view.point(21.125, 37.375),
                    view.point(12.682, 35.663),
                    view.point(16.8152, 37.375),
                );
                path.cubic_bezier_to(
                    view.point(32.6155, 32.6155),
                    view.point(25.4348, 37.375),
                    view.point(29.568, 35.663),
                );
                path.cubic_bezier_to(
                    view.point(37.375, 21.125),
                    view.point(35.663, 29.568),
                    view.point(37.375, 25.4348),
                );
                path.close();
                path.move_to(view.point(34.2367, 37.6898));
                path.cubic_bezier_to(
                    view.point(21.125, 42.25),
                    view.point(30.6414, 40.5438),
                    view.point(26.0812, 42.25),
                );
                path.cubic_bezier_to(
                    view.point(0.0, 21.125),
                    view.point(9.45547, 42.25),
                    view.point(0.0, 32.7945),
                );
                path.cubic_bezier_to(
                    view.point(21.125, 0.0),
                    view.point(0.0, 9.45547),
                    view.point(9.45547, 0.0),
                );
                path.cubic_bezier_to(
                    view.point(42.25, 21.125),
                    view.point(32.7945, 0.0),
                    view.point(42.25, 9.45547),
                );
                path.cubic_bezier_to(
                    view.point(37.6898, 34.2367),
                    view.point(42.25, 26.0812),
                    view.point(40.5438, 30.6414),
                );
                path.line_to(view.point(51.2891, 47.8359));
                path.cubic_bezier_to(
                    view.point(51.2891, 51.2789),
                    view.point(52.2438, 48.7906),
                    view.point(52.2438, 50.3344),
                );
                path.cubic_bezier_to(
                    view.point(47.8461, 51.2789),
                    view.point(50.3344, 52.2234),
                    view.point(48.7906, 52.2336),
                );
                path.line_to(view.point(34.2367, 37.6898));
                path.close();

                if let Ok(path) = path.build() {
                    window.paint_path(path, self.color);
                }
            },
        )
        .w(self.width)
        .h(self.height)
    }
}
