use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct DaevanionNodesIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl DaevanionNodesIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for DaevanionNodesIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(45.9748, 23.7739));
                path.cubic_bezier_to(
                    view.point(45.4356, 22.693),
                    view.point(45.8855, 23.3752),
                    view.point(45.7005, 23.0042),
                );
                path.cubic_bezier_to(
                    view.point(44.4546, 21.988),
                    view.point(45.1707, 22.3819),
                    view.point(44.834, 22.1399),
                );
                path.line_to(view.point(33.7233, 17.6973));
                path.line_to(view.point(35.9438, 2.89654));
                path.cubic_bezier_to(
                    view.point(35.6841, 1.34546),
                    view.point(36.0253, 2.36506),
                    view.point(35.9343, 1.82145),
                );
                path.cubic_bezier_to(
                    view.point(34.5538, 0.251692),
                    view.point(35.434, 0.869465),
                    view.point(35.0378, 0.486134),
                );
                path.cubic_bezier_to(
                    view.point(32.9946, 0.0427906),
                    view.point(34.0698, 0.0172499),
                    view.point(33.5233, -0.0559635),
                );
                path.cubic_bezier_to(
                    view.point(31.6161, 0.800416),
                    view.point(32.4659, 0.141545),
                    view.point(31.9828, 0.407069),
                );
                path.line_to(view.point(8.13218, 25.9539));
                path.cubic_bezier_to(
                    view.point(7.5435, 27.0096),
                    view.point(7.85304, 26.2527),
                    view.point(7.65098, 26.6151),
                );
                path.cubic_bezier_to(
                    view.point(7.51538, 28.218),
                    view.point(7.43602, 27.4041),
                    view.point(7.42637, 27.8189),
                );
                path.cubic_bezier_to(
                    view.point(8.05433, 29.3),
                    view.point(7.6044, 28.6171),
                    view.point(7.78938, 28.9885),
                );
                path.cubic_bezier_to(
                    view.point(9.03589, 30.0057),
                    view.point(8.31928, 29.6115),
                    view.point(8.65619, 29.8537),
                );
                path.line_to(view.point(19.7672, 34.2965));
                path.line_to(view.point(17.5467, 49.1035));
                path.cubic_bezier_to(
                    view.point(17.8063, 50.6545),
                    view.point(17.4652, 49.6349),
                    view.point(17.5562, 50.1785),
                );
                path.cubic_bezier_to(
                    view.point(18.9367, 51.7483),
                    view.point(18.0565, 51.1305),
                    view.point(18.4527, 51.5139),
                );
                path.cubic_bezier_to(
                    view.point(20.4959, 51.9572),
                    view.point(19.4207, 51.9827),
                    view.point(19.9672, 52.056),
                );
                path.cubic_bezier_to(
                    view.point(21.8744, 51.1996),
                    view.point(21.0246, 51.8585),
                    view.point(21.5077, 51.5929),
                );
                path.line_to(view.point(45.3583, 26.0461));
                path.cubic_bezier_to(
                    view.point(45.9489, 24.9866),
                    view.point(45.6387, 25.7464),
                    view.point(45.8414, 25.3826),
                );
                path.cubic_bezier_to(
                    view.point(45.9748, 23.7739),
                    view.point(46.0563, 24.5905),
                    view.point(46.0652, 24.1742),
                );
                path.close();
                path.move_to(view.point(23.7259, 41.8362));
                path.line_to(view.point(25.0406, 33.0807));
                path.cubic_bezier_to(
                    view.point(24.7349, 31.4565),
                    view.point(25.1246, 32.5203),
                    view.point(25.0169, 31.948),
                );
                path.cubic_bezier_to(
                    view.point(23.4869, 30.3725),
                    view.point(24.4529, 30.9649),
                    view.point(24.0131, 30.583),
                );
                path.line_to(view.point(14.3198, 26.7064));
                path.line_to(view.point(29.7646, 10.1638));
                path.line_to(view.point(28.4499, 18.9193));
                path.cubic_bezier_to(
                    view.point(28.7556, 20.5435),
                    view.point(28.3659, 19.4797),
                    view.point(28.4736, 20.052),
                );
                path.cubic_bezier_to(
                    view.point(30.0036, 21.6275),
                    view.point(29.0376, 21.0351),
                    view.point(29.4774, 21.417),
                );
                path.line_to(view.point(39.1707, 25.2936));
                path.line_to(view.point(23.7259, 41.8362));
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
