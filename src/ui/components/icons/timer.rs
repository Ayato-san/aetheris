use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct TimerIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl TimerIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for TimerIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(26.2987, 8.96552));
                path.cubic_bezier_to(
                    view.point(14.3444, 12.5918),
                    view.point(22.043, 8.96552),
                    view.point(17.8829, 10.2275),
                );
                path.cubic_bezier_to(
                    view.point(6.41941, 22.2485),
                    view.point(10.8059, 14.9562),
                    view.point(8.04799, 18.3167),
                );
                path.cubic_bezier_to(
                    view.point(5.19495, 34.6806),
                    view.point(4.79082, 26.1802),
                    view.point(4.3647, 30.5066),
                );
                path.cubic_bezier_to(
                    view.point(11.0838, 45.6977),
                    view.point(6.0252, 38.8545),
                    view.point(8.07452, 42.6885),
                );
                path.cubic_bezier_to(
                    view.point(22.1009, 51.5865),
                    view.point(14.093, 48.707),
                    view.point(17.927, 50.7563),
                );
                path.cubic_bezier_to(
                    view.point(34.533, 50.3621),
                    view.point(26.2749, 52.4168),
                    view.point(30.6013, 51.9907),
                );
                path.cubic_bezier_to(
                    view.point(44.1897, 42.4371),
                    view.point(38.4648, 48.7335),
                    view.point(41.8253, 45.9756),
                );
                path.cubic_bezier_to(
                    view.point(47.816, 30.4828),
                    view.point(46.554, 38.8986),
                    view.point(47.816, 34.7385),
                );
                path.cubic_bezier_to(
                    view.point(41.5065, 15.275),
                    view.point(47.8095, 24.778),
                    view.point(45.5404, 19.3088),
                );
                path.cubic_bezier_to(
                    view.point(26.2987, 8.96552),
                    view.point(37.4727, 11.2411),
                    view.point(32.0035, 8.97204),
                );
                path.close();
                path.move_to(view.point(26.2987, 46.6207));
                path.cubic_bezier_to(
                    view.point(17.333, 43.901),
                    view.point(23.107, 46.6207),
                    view.point(19.9869, 45.6742),
                );
                path.cubic_bezier_to(
                    view.point(11.3892, 36.6585),
                    view.point(14.6791, 42.1277),
                    view.point(12.6107, 39.6073),
                );
                path.cubic_bezier_to(
                    view.point(10.4709, 27.3344),
                    view.point(10.1678, 33.7097),
                    view.point(9.84821, 30.4649),
                );
                path.cubic_bezier_to(
                    view.point(14.8875, 19.0715),
                    view.point(11.0936, 24.2039),
                    view.point(12.6306, 21.3284),
                );
                path.cubic_bezier_to(
                    view.point(23.1504, 14.6549),
                    view.point(17.1444, 16.8146),
                    view.point(20.0199, 15.2776),
                );
                path.cubic_bezier_to(
                    view.point(32.4745, 15.5733),
                    view.point(26.2808, 14.0322),
                    view.point(29.5256, 14.3518),
                );
                path.cubic_bezier_to(
                    view.point(39.7169, 21.517),
                    view.point(35.4233, 16.7947),
                    view.point(37.9437, 18.8631),
                );
                path.cubic_bezier_to(
                    view.point(42.4367, 30.4828),
                    view.point(41.4902, 24.1709),
                    view.point(42.4367, 27.291),
                );
                path.cubic_bezier_to(
                    view.point(37.7047, 41.8888),
                    view.point(42.4319, 34.7613),
                    view.point(40.7302, 38.8633),
                );
                path.cubic_bezier_to(
                    view.point(26.2987, 46.6207),
                    view.point(34.6793, 44.9142),
                    view.point(30.5773, 46.6159),
                );
                path.close();
                path.move_to(view.point(34.4775, 21.4074));
                path.cubic_bezier_to(
                    view.point(35.063, 22.2811),
                    view.point(34.7283, 21.6573),
                    view.point(34.9272, 21.9542),
                );
                path.cubic_bezier_to(
                    view.point(35.2686, 23.3126),
                    view.point(35.1987, 22.6081),
                    view.point(35.2686, 22.9586),
                );
                path.cubic_bezier_to(
                    view.point(35.063, 24.344),
                    view.point(35.2686, 23.6666),
                    view.point(35.1987, 24.0171),
                );
                path.cubic_bezier_to(
                    view.point(34.4775, 25.2178),
                    view.point(34.9272, 24.671),
                    view.point(34.7283, 24.9679),
                );
                path.line_to(view.point(28.2017, 31.4936));
                path.cubic_bezier_to(
                    view.point(26.2965, 32.2828),
                    view.point(27.6964, 31.9989),
                    view.point(27.0111, 32.2828),
                );
                path.cubic_bezier_to(
                    view.point(24.3913, 31.4936),
                    view.point(25.5819, 32.2828),
                    view.point(24.8966, 31.9989),
                );
                path.cubic_bezier_to(
                    view.point(23.6022, 29.5884),
                    view.point(23.886, 30.9883),
                    view.point(23.6022, 30.303),
                );
                path.cubic_bezier_to(
                    view.point(24.3913, 27.6833),
                    view.point(23.6022, 28.8739),
                    view.point(23.886, 28.1886),
                );
                path.line_to(view.point(30.6672, 21.4074));
                path.cubic_bezier_to(
                    view.point(31.5409, 20.822),
                    view.point(30.9171, 21.1567),
                    view.point(31.214, 20.9577),
                );
                path.cubic_bezier_to(
                    view.point(32.5724, 20.6163),
                    view.point(31.8679, 20.6862),
                    view.point(32.2184, 20.6163),
                );
                path.cubic_bezier_to(
                    view.point(33.6038, 20.822),
                    view.point(32.9264, 20.6163),
                    view.point(33.2769, 20.6862),
                );
                path.cubic_bezier_to(
                    view.point(34.4775, 21.4074),
                    view.point(33.9307, 20.9577),
                    view.point(34.2277, 21.1567),
                );
                path.close();
                path.move_to(view.point(18.2298, 2.68965));
                path.cubic_bezier_to(
                    view.point(19.0176, 0.787782),
                    view.point(18.2298, 1.97631),
                    view.point(18.5132, 1.29219),
                );
                path.cubic_bezier_to(
                    view.point(20.9194, 0.0),
                    view.point(19.522, 0.283374),
                    view.point(20.2061, 0.0),
                );
                path.line_to(view.point(31.6781, 0.0));
                path.cubic_bezier_to(
                    view.point(33.5799, 0.787782),
                    view.point(32.3914, 1.06296e-08),
                    view.point(33.0755, 0.283374),
                );
                path.cubic_bezier_to(
                    view.point(34.3677, 2.68965),
                    view.point(34.0843, 1.29219),
                    view.point(34.3677, 1.97631),
                );
                path.cubic_bezier_to(
                    view.point(33.5799, 4.59153),
                    view.point(34.3677, 3.403),
                    view.point(34.0843, 4.08712),
                );
                path.cubic_bezier_to(
                    view.point(31.6781, 5.37931),
                    view.point(33.0755, 5.09594),
                    view.point(32.3914, 5.37931),
                );
                path.line_to(view.point(20.9194, 5.37931));
                path.cubic_bezier_to(
                    view.point(19.0176, 4.59153),
                    view.point(20.2061, 5.37931),
                    view.point(19.522, 5.09594),
                );
                path.cubic_bezier_to(
                    view.point(18.2298, 2.68965),
                    view.point(18.5132, 4.08712),
                    view.point(18.2298, 3.403),
                );
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
