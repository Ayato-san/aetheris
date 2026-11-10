use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct CrossIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl CrossIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for CrossIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(46.7188, 9.88542));
                path.cubic_bezier_to(
                    view.point(46.7188, 5.2948),
                    view.point(47.9917, 8.61251),
                    view.point(47.9917, 6.55417),
                );
                path.cubic_bezier_to(
                    view.point(42.1281, 5.2948),
                    view.point(45.4458, 4.03542),
                    view.point(43.3875, 4.02188),
                );
                path.line_to(view.point(26.0135, 21.4094));
                path.line_to(view.point(9.88542, 5.28126));
                path.cubic_bezier_to(
                    view.point(5.2948, 5.28126),
                    view.point(8.61251, 4.00834),
                    view.point(6.55417, 4.00834),
                );
                path.cubic_bezier_to(
                    view.point(5.2948, 9.87188),
                    view.point(4.03542, 6.55417),
                    view.point(4.02188, 8.6125),
                );
                path.line_to(view.point(21.4094, 25.9865));
                path.line_to(view.point(5.28126, 42.1146));
                path.cubic_bezier_to(
                    view.point(5.28126, 46.7052),
                    view.point(4.00834, 43.3875),
                    view.point(4.00834, 45.4458),
                );
                path.cubic_bezier_to(
                    view.point(9.87188, 46.7052),
                    view.point(6.55417, 47.9646),
                    view.point(8.61251, 47.9781),
                );
                path.line_to(view.point(25.9865, 30.5906));
                path.line_to(view.point(42.1146, 46.7188));
                path.cubic_bezier_to(
                    view.point(46.7052, 46.7188),
                    view.point(43.3875, 47.9917),
                    view.point(45.4458, 47.9917),
                );
                path.cubic_bezier_to(
                    view.point(46.7052, 42.1281),
                    view.point(47.9646, 45.4458),
                    view.point(47.9781, 43.3875),
                );
                path.line_to(view.point(30.5906, 26.0135));
                path.line_to(view.point(46.7188, 9.88542));
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
