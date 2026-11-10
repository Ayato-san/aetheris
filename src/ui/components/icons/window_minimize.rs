use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct WindowMinimizeIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl WindowMinimizeIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for WindowMinimizeIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(7.04171, 23.54));
                path.cubic_bezier_to(
                    view.point(4.33337, 26.0021),
                    view.point(5.54366, 23.54),
                    view.point(4.33337, 24.6403),
                );
                path.cubic_bezier_to(
                    view.point(7.04171, 28.4642),
                    view.point(4.33337, 27.364),
                    view.point(5.54366, 28.4642),
                );
                path.line_to(view.point(44.9584, 28.4642));
                path.cubic_bezier_to(
                    view.point(47.6667, 26.0021),
                    view.point(46.4564, 28.4642),
                    view.point(47.6667, 27.364),
                );
                path.cubic_bezier_to(
                    view.point(44.9584, 23.54),
                    view.point(47.6667, 24.6403),
                    view.point(46.4564, 23.54),
                );
                path.line_to(view.point(7.04171, 23.54));
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
