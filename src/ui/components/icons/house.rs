use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct HouseIcon {
    width: Pixels,
    height: Pixels,
    first_color: Rgba,
    second_color: Rgba,
}

impl HouseIcon {
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

impl RenderOnce for HouseIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(5.77778, 27.309));
                path.cubic_bezier_to(
                    view.point(26.0, 9.61458),
                    view.point(12.5215, 21.4139),
                    view.point(19.2563, 15.5187),
                );
                path.line_to(view.point(46.2312, 27.3181));
                path.line_to(view.point(46.2674, 45.491));
                path.cubic_bezier_to(
                    view.point(42.6562, 49.1111),
                    view.point(46.2674, 47.4861),
                    view.point(44.6514, 49.1111),
                );
                path.line_to(view.point(35.3889, 49.1111));
                path.cubic_bezier_to(
                    view.point(31.7778, 45.5),
                    view.point(33.3938, 49.1111),
                    view.point(31.7778, 47.4951),
                );
                path.line_to(view.point(31.7778, 37.5285));
                path.cubic_bezier_to(
                    view.point(28.8889, 34.6396),
                    view.point(31.7778, 35.9306),
                    view.point(30.4868, 34.6396),
                );
                path.line_to(view.point(23.1111, 34.6396));
                path.cubic_bezier_to(
                    view.point(20.2222, 37.5285),
                    view.point(21.5132, 34.6396),
                    view.point(20.2222, 35.9306),
                );
                path.line_to(view.point(20.2222, 45.5));
                path.cubic_bezier_to(
                    view.point(16.6111, 49.1111),
                    view.point(20.2222, 47.4951),
                    view.point(18.6063, 49.1111),
                );
                path.line_to(view.point(9.39792, 49.1111));
                path.cubic_bezier_to(
                    view.point(5.78681, 45.5),
                    view.point(7.40278, 49.1111),
                    view.point(5.78681, 47.4951),
                );
                path.line_to(view.point(5.77778, 27.309));
                path.close();

                if let Ok(path) = path.build() {
                    window.paint_path(path, self.first_color);
                }

                let mut path = PathBuilder::fill();
                path.move_to(view.point(24.0994, 3.59982));
                path.cubic_bezier_to(
                    view.point(27.9091, 3.59982),
                    view.point(25.1918, 2.65191),
                    view.point(26.8168, 2.65191),
                );
                path.line_to(view.point(51.0203, 23.822));
                path.cubic_bezier_to(
                    view.point(51.2911, 27.9026),
                    view.point(52.221, 24.8693),
                    view.point(52.3383, 26.7019),
                );
                path.cubic_bezier_to(
                    view.point(47.2105, 28.1734),
                    view.point(50.2439, 29.1033),
                    view.point(48.4112, 29.2207),
                );
                path.line_to(view.point(26.0043, 9.61232));
                path.line_to(view.point(4.79804, 28.1734));
                path.cubic_bezier_to(
                    view.point(0.717482, 27.9026),
                    view.point(3.59734, 29.2207),
                    view.point(1.77373, 29.1033),
                );
                path.cubic_bezier_to(
                    view.point(0.988316, 23.822),
                    view.point(-0.338767, 26.7019),
                    view.point(-0.212379, 24.8783),
                );
                path.line_to(view.point(24.0994, 3.59982));
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
