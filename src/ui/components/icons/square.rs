use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct SquareIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl SquareIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for SquareIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(42.25, 8.125));
                path.cubic_bezier_to(
                    view.point(43.875, 9.75),
                    view.point(43.1437, 8.125),
                    view.point(43.875, 8.85625),
                );
                path.line_to(view.point(43.875, 42.25));
                path.cubic_bezier_to(
                    view.point(42.25, 43.875),
                    view.point(43.875, 43.1437),
                    view.point(43.1437, 43.875),
                );
                path.line_to(view.point(9.75, 43.875));
                path.cubic_bezier_to(
                    view.point(8.125, 42.25),
                    view.point(8.85625, 43.875),
                    view.point(8.125, 43.1437),
                );
                path.line_to(view.point(8.125, 9.75));
                path.cubic_bezier_to(
                    view.point(9.75, 8.125),
                    view.point(8.125, 8.85625),
                    view.point(8.85625, 8.125),
                );
                path.line_to(view.point(42.25, 8.125));
                path.close();
                path.move_to(view.point(9.75, 3.25));
                path.cubic_bezier_to(
                    view.point(3.25, 9.75),
                    view.point(6.16484, 3.25),
                    view.point(3.25, 6.16484),
                );
                path.line_to(view.point(3.25, 42.25));
                path.cubic_bezier_to(
                    view.point(9.75, 48.75),
                    view.point(3.25, 45.8352),
                    view.point(6.16484, 48.75),
                );
                path.line_to(view.point(42.25, 48.75));
                path.cubic_bezier_to(
                    view.point(48.75, 42.25),
                    view.point(45.8352, 48.75),
                    view.point(48.75, 45.8352),
                );
                path.line_to(view.point(48.75, 9.75));
                path.cubic_bezier_to(
                    view.point(42.25, 3.25),
                    view.point(48.75, 6.16484),
                    view.point(45.8352, 3.25),
                );
                path.line_to(view.point(9.75, 3.25));
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
