use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct RestoreIcon {
    width: Pixels,
    height: Pixels,
    first_color: Rgba,
    second_color: Rgba,
}

impl RestoreIcon {
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

impl RenderOnce for RestoreIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(39.0, 45.5));
                path.cubic_bezier_to(
                    view.point(32.5, 52.0),
                    view.point(39.0, 49.0852),
                    view.point(36.0852, 52.0),
                );
                path.line_to(view.point(6.5, 52.0));
                path.cubic_bezier_to(
                    view.point(0.0, 45.5),
                    view.point(2.91484, 52.0),
                    view.point(0.0, 49.0852),
                );
                path.line_to(view.point(0.0, 19.5));
                path.cubic_bezier_to(
                    view.point(6.5, 13.0),
                    view.point(1.5346e-06, 15.9148),
                    view.point(2.91485, 13.0),
                );
                path.line_to(view.point(32.5, 13.0));
                path.cubic_bezier_to(
                    view.point(39.0, 19.5),
                    view.point(36.0852, 13.0),
                    view.point(39.0, 15.9148),
                );
                path.line_to(view.point(39.0, 45.5));
                path.close();
                path.move_to(view.point(34.125, 19.5));
                path.cubic_bezier_to(
                    view.point(32.5, 17.875),
                    view.point(34.125, 18.6063),
                    view.point(33.3937, 17.875),
                );
                path.line_to(view.point(6.5, 17.875));
                path.cubic_bezier_to(
                    view.point(4.875, 19.5),
                    view.point(5.60625, 17.875),
                    view.point(4.875, 18.6063),
                );
                path.line_to(view.point(4.875, 45.5));
                path.cubic_bezier_to(
                    view.point(6.5, 47.125),
                    view.point(4.875, 46.3937),
                    view.point(5.60625, 47.125),
                );
                path.line_to(view.point(32.5, 47.125));
                path.cubic_bezier_to(
                    view.point(34.125, 45.5),
                    view.point(33.3937, 47.125),
                    view.point(34.125, 46.3937),
                );
                path.line_to(view.point(34.125, 19.5));
                path.close();

                if let Ok(path) = path.build() {
                    window.paint_path(path, self.first_color);
                }

                let mut path = PathBuilder::fill();
                path.move_to(view.point(43.875, 4.875));
                path.line_to(view.point(21.125, 4.875));
                path.cubic_bezier_to(
                    view.point(17.875, 8.125),
                    view.point(19.3273, 4.875),
                    view.point(17.875, 6.32734),
                );
                path.line_to(view.point(17.875, 9.75));
                path.line_to(view.point(13.0, 9.75));
                path.line_to(view.point(13.0, 8.125));
                path.cubic_bezier_to(
                    view.point(21.125, 0.0),
                    view.point(13.0, 3.63594),
                    view.point(16.6359, 0.0),
                );
                path.line_to(view.point(43.875, 0.0));
                path.cubic_bezier_to(
                    view.point(52.0, 8.125),
                    view.point(48.3641, 0.0),
                    view.point(52.0, 3.63594),
                );
                path.line_to(view.point(52.0, 30.875));
                path.cubic_bezier_to(
                    view.point(43.875, 39.0),
                    view.point(52.0, 35.3641),
                    view.point(48.3641, 39.0),
                );
                path.line_to(view.point(42.25, 39.0));
                path.line_to(view.point(42.25, 34.125));
                path.line_to(view.point(43.875, 34.125));
                path.cubic_bezier_to(
                    view.point(47.125, 30.875),
                    view.point(45.6727, 34.125),
                    view.point(47.125, 32.6727),
                );
                path.line_to(view.point(47.125, 8.125));
                path.cubic_bezier_to(
                    view.point(43.875, 4.875),
                    view.point(47.125, 6.32734),
                    view.point(45.6727, 4.875),
                );
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
