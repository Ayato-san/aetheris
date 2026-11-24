use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct LayerGroupIcon {
    width: Pixels,
    height: Pixels,
    first_color: Rgba,
    second_color: Rgba,
}

impl LayerGroupIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            first_color: color,
            second_color: color,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn with_second_color(mut self, color: Rgba) -> Self {
        self.second_color = color;
        self
    }
}

impl RenderOnce for LayerGroupIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 4359.0, 392.0, 2e+01, 2e+01);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(4359.0, 402.0876));
                path.cubic_bezier_to(
                    view.point(4359.4824, 402.84448),
                    view.point(4359.0, 402.41394),
                    view.point(4359.1875, 402.7091),
                );
                path.line_to(view.point(4367.073, 406.35144));
                path.cubic_bezier_to(
                    view.point(4367.8887, 406.53198),
                    view.point(4367.333, 406.46948),
                    view.point(4367.6113, 406.53198),
                );
                path.cubic_bezier_to(
                    view.point(4368.705, 406.35144),
                    view.point(4368.167, 406.53198),
                    view.point(4368.4443, 406.47296),
                );
                path.line_to(view.point(4376.295, 402.84448));
                path.cubic_bezier_to(
                    view.point(4376.7773, 402.0876),
                    view.point(4376.59, 402.7091),
                    view.point(4376.7773, 402.41394),
                );
                path.cubic_bezier_to(
                    view.point(4376.295, 401.33063),
                    view.point(4376.7773, 401.76117),
                    view.point(4376.59, 401.46606),
                );
                path.line_to(view.point(4374.448, 400.47644));
                path.line_to(view.point(4369.17, 402.91394));
                path.cubic_bezier_to(
                    view.point(4366.6074, 402.91394),
                    view.point(4368.3574, 403.28894),
                    view.point(4367.42, 403.28894),
                );
                path.line_to(view.point(4361.33, 400.47644));
                path.line_to(view.point(4359.4824, 401.33063));
                path.cubic_bezier_to(
                    view.point(4359.0, 402.0876),
                    view.point(4359.1875, 401.46606),
                    view.point(4359.0, 401.76117),
                );
                path.close();

                if let Ok(path) = path.build() {
                    window.paint_path(path, self.first_color);
                }

                let mut path = PathBuilder::fill();
                path.move_to(view.point(4367.073, 393.3792));
                path.cubic_bezier_to(
                    view.point(4368.705, 393.3792),
                    view.point(4367.59, 393.13965),
                    view.point(4368.1875, 393.13965),
                );
                path.line_to(view.point(4376.295, 396.88617));
                path.cubic_bezier_to(
                    view.point(4376.7773, 397.64313),
                    view.point(4376.59, 397.0216),
                    view.point(4376.7773, 397.3167),
                );
                path.cubic_bezier_to(
                    view.point(4376.295, 398.4001),
                    view.point(4376.7773, 397.96948),
                    view.point(4376.59, 398.26465),
                );
                path.line_to(view.point(4368.705, 401.90698));
                path.cubic_bezier_to(
                    view.point(4367.073, 401.90698),
                    view.point(4368.1875, 402.1466),
                    view.point(4367.59, 402.1466),
                );
                path.line_to(view.point(4359.4824, 398.4001));
                path.cubic_bezier_to(
                    view.point(4359.0, 397.64313),
                    view.point(4359.1875, 398.26117),
                    view.point(4359.0, 397.966),
                );
                path.cubic_bezier_to(
                    view.point(4359.4824, 396.88617),
                    view.point(4359.0, 397.3202),
                    view.point(4359.1875, 397.0216),
                );
                path.line_to(view.point(4367.073, 393.3792));
                path.close();
                path.move_to(view.point(4374.448, 404.9209));
                path.line_to(view.point(4376.295, 405.7751));
                path.cubic_bezier_to(
                    view.point(4376.7773, 406.53198),
                    view.point(4376.59, 405.91046),
                    view.point(4376.7773, 406.20563),
                );
                path.cubic_bezier_to(
                    view.point(4376.295, 407.28894),
                    view.point(4376.7773, 406.8584),
                    view.point(4376.59, 407.1535),
                );
                path.line_to(view.point(4368.705, 410.7959));
                path.cubic_bezier_to(
                    view.point(4367.073, 410.7959),
                    view.point(4368.1875, 411.03546),
                    view.point(4367.59, 411.03546),
                );
                path.line_to(view.point(4359.4824, 407.28894));
                path.cubic_bezier_to(
                    view.point(4359.0, 406.53198),
                    view.point(4359.1875, 407.1501),
                    view.point(4359.0, 406.85492),
                );
                path.cubic_bezier_to(
                    view.point(4359.4824, 405.7751),
                    view.point(4359.0, 406.2091),
                    view.point(4359.1875, 405.91046),
                );
                path.line_to(view.point(4361.33, 404.9209));
                path.line_to(view.point(4366.6074, 407.3584));
                path.cubic_bezier_to(
                    view.point(4369.17, 407.3584),
                    view.point(4367.42, 407.7334),
                    view.point(4368.3574, 407.7334),
                );
                path.line_to(view.point(4374.448, 404.9209));
                path.close();

                if let Ok(path) = path.build() {
                    window.paint_path(path, self.second_color);
                }
            },
        )
        .w(self.width)
        .h(self.height)
    }
}
