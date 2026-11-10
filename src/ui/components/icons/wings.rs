use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct WingsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl WingsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for WingsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(50.8839, 49.249));
                path.cubic_bezier_to(
                    view.point(50.0558, 48.3028),
                    view.point(50.6473, 48.9888),
                    view.point(50.2687, 48.5866),
                );
                path.cubic_bezier_to(
                    view.point(47.9026, 46.2685),
                    view.point(49.8192, 48.0426),
                    view.point(48.8727, 47.1201),
                );
                path.cubic_bezier_to(
                    view.point(45.2999, 43.7848),
                    view.point(46.9562, 45.4406),
                    view.point(45.7731, 44.3052),
                );
                path.cubic_bezier_to(
                    view.point(43.5963, 42.8386),
                    view.point(44.2588, 42.6494),
                    view.point(43.9039, 42.4365),
                );
                path.cubic_bezier_to(
                    view.point(43.3833, 43.4537),
                    view.point(43.478, 42.9806),
                    view.point(43.3833, 43.2644),
                );
                path.cubic_bezier_to(
                    view.point(41.3485, 44.0214),
                    view.point(43.3833, 45.0148),
                    view.point(42.5079, 45.275),
                );
                path.cubic_bezier_to(
                    view.point(35.5278, 35.4585),
                    view.point(38.5091, 40.9936),
                    view.point(36.7109, 38.3207),
                );
                path.cubic_bezier_to(
                    view.point(32.8068, 31.5792),
                    view.point(34.676, 33.4242),
                    view.point(34.1082, 32.5963),
                );
                path.cubic_bezier_to(
                    view.point(31.1269, 32.1942),
                    view.point(31.8604, 30.7986),
                    view.point(31.2925, 31.0115),
                );
                path.cubic_bezier_to(
                    view.point(29.9438, 33.5425),
                    view.point(30.9612, 33.4479),
                    view.point(30.701, 33.7317),
                );
                path.cubic_bezier_to(
                    view.point(25.3299, 29.0718),
                    view.point(29.5416, 33.4479),
                    view.point(26.8442, 30.8223),
                );
                path.cubic_bezier_to(
                    view.point(20.716, 22.0465),
                    view.point(23.7446, 27.2505),
                    view.point(22.9165, 25.9495),
                );
                path.cubic_bezier_to(
                    view.point(18.7994, 20.4617),
                    view.point(20.3374, 21.3842),
                    view.point(19.6039, 20.7928),
                );
                path.cubic_bezier_to(
                    view.point(12.3399, 15.1631),
                    view.point(16.3623, 19.5155),
                    view.point(14.6351, 18.0963),
                );
                path.cubic_bezier_to(
                    view.point(10.1158, 12.5375),
                    view.point(11.5591, 14.146),
                    view.point(10.5417, 12.9633),
                );
                path.cubic_bezier_to(
                    view.point(2.11834, 7.04971),
                    view.point(8.60149, 11.0236),
                    view.point(5.50188, 8.89474),
                );
                path.cubic_bezier_to(
                    view.point(0.0361638, 5.15736),
                    view.point(0.225453, 6.00892),
                    view.point(-0.129464, 5.67776),
                );
                path.cubic_bezier_to(
                    view.point(3.27774, 3.951),
                    view.point(0.272775, 4.44774),
                    view.point(1.26654, 4.09292),
                );
                path.cubic_bezier_to(
                    view.point(9.87919, 4.68428),
                    view.point(4.67374, 3.88003),
                    view.point(8.22291, 4.2585),
                );
                path.cubic_bezier_to(
                    view.point(12.269, 5.22833),
                    view.point(10.4944, 4.84986),
                    view.point(11.5828, 5.0864),
                );
                path.cubic_bezier_to(
                    view.point(14.2092, 5.67776),
                    view.point(12.9551, 5.3466),
                    view.point(13.8306, 5.55949),
                );
                path.cubic_bezier_to(
                    view.point(15.9364, 6.10354),
                    view.point(14.6114, 5.81968),
                    view.point(15.3922, 6.00892),
                );
                path.cubic_bezier_to(
                    view.point(17.9003, 6.57662),
                    view.point(16.4806, 6.22181),
                    view.point(17.3798, 6.4347),
                );
                path.cubic_bezier_to(
                    view.point(22.7982, 7.23894),
                    view.point(19.2727, 6.97874),
                    view.point(20.0535, 7.09702),
                );
                path.cubic_bezier_to(
                    view.point(28.1929, 7.8303),
                    view.point(25.4482, 7.38087),
                    view.point(27.7197, 7.61741),
                );
                path.cubic_bezier_to(
                    view.point(28.4295, 8.75282),
                    view.point(28.5715, 8.01953),
                    view.point(28.6898, 8.46896),
                );
                path.cubic_bezier_to(
                    view.point(26.2527, 9.60437),
                    view.point(28.2166, 8.98936),
                    view.point(27.3884, 9.29686),
                );
                path.cubic_bezier_to(
                    view.point(29.8728, 10.8817),
                    view.point(24.904, 9.93553),
                    view.point(26.8442, 10.6452),
                );
                path.cubic_bezier_to(
                    view.point(39.7159, 15.1395),
                    view.point(33.351, 11.1892),
                    view.point(39.7159, 13.9331),
                );
                path.cubic_bezier_to(
                    view.point(38.3908, 16.4405),
                    view.point(39.7159, 15.5179),
                    view.point(39.0533, 16.1566),
                );
                path.cubic_bezier_to(
                    view.point(34.9836, 16.6297),
                    view.point(37.681, 16.748),
                    view.point(37.1131, 16.7953),
                );
                path.cubic_bezier_to(
                    view.point(33.351, 16.7243),
                    view.point(33.7296, 16.5587),
                    view.point(33.4693, 16.5587),
                );
                path.cubic_bezier_to(
                    view.point(33.4457, 17.1028),
                    view.point(33.2327, 16.8662),
                    view.point(33.2564, 16.9372),
                );
                path.cubic_bezier_to(
                    view.point(37.2314, 18.4747),
                    view.point(33.7296, 17.363),
                    view.point(35.9537, 18.1909),
                );
                path.cubic_bezier_to(
                    view.point(40.5677, 19.5392),
                    view.point(38.5565, 18.8059),
                    view.point(40.0234, 19.2553),
                );
                path.cubic_bezier_to(
                    view.point(47.8789, 27.9128),
                    view.point(41.7507, 20.1305),
                    view.point(47.2874, 26.4935),
                );
                path.cubic_bezier_to(
                    view.point(47.0508, 28.717),
                    view.point(48.1156, 28.4805),
                    view.point(47.7843, 28.788),
                );
                path.cubic_bezier_to(
                    view.point(40.0708, 26.7301),
                    view.point(46.2227, 28.6461),
                    view.point(43.1704, 27.7709),
                );
                path.cubic_bezier_to(
                    view.point(36.5926, 25.5),
                    view.point(38.0596, 26.0678),
                    view.point(37.0185, 25.6893),
                );
                path.cubic_bezier_to(
                    view.point(33.422, 25.0506),
                    view.point(35.7408, 25.1452),
                    view.point(33.6349, 24.8377),
                );
                path.cubic_bezier_to(
                    view.point(34.2975, 25.9258),
                    view.point(33.2327, 25.2162),
                    view.point(33.6113, 25.571),
                );
                path.cubic_bezier_to(
                    view.point(35.8828, 26.9193),
                    view.point(34.7234, 26.1387),
                    view.point(35.4095, 26.5645),
                );
                path.cubic_bezier_to(
                    view.point(41.2538, 29.7105),
                    view.point(37.3261, 27.9837),
                    view.point(39.29, 29.0245),
                );
                path.cubic_bezier_to(
                    view.point(42.6025, 30.2309),
                    view.point(41.6087, 29.8288),
                    view.point(42.2239, 30.0653),
                );
                path.cubic_bezier_to(
                    view.point(44.3061, 30.8223),
                    view.point(43.0048, 30.3728),
                    view.point(43.7619, 30.6567),
                );
                path.cubic_bezier_to(
                    view.point(47.0981, 32.8092),
                    view.point(45.6075, 31.2244),
                    view.point(46.0807, 31.5555),
                );
                path.cubic_bezier_to(
                    view.point(48.1392, 36.8541),
                    view.point(49.8428, 36.2391),
                    view.point(50.1267, 37.3745),
                );
                path.cubic_bezier_to(
                    view.point(47.8316, 38.8647),
                    view.point(46.6249, 36.4756),
                    view.point(46.5776, 36.8778),
                );
                path.cubic_bezier_to(
                    view.point(51.4281, 46.2685),
                    view.point(49.0147, 40.7571),
                    view.point(50.8129, 44.4235),
                );
                path.cubic_bezier_to(
                    view.point(50.8839, 49.249),
                    view.point(52.3746, 49.0597),
                    view.point(52.1143, 50.5026),
                );
                path.close();
                path.move_to(view.point(43.8329, 40.4969));
                path.cubic_bezier_to(
                    view.point(44.5427, 38.3443),
                    view.point(44.2588, 40.213),
                    view.point(44.4717, 39.5743),
                );
                path.cubic_bezier_to(
                    view.point(43.6199, 36.6885),
                    view.point(44.661, 35.8606),
                    view.point(43.8092, 34.2994),
                );
                path.cubic_bezier_to(
                    view.point(42.7918, 38.0841),
                    view.point(43.549, 37.7057),
                    view.point(43.3123, 38.0841),
                );
                path.cubic_bezier_to(
                    view.point(39.4792, 35.08),
                    view.point(42.2476, 38.0841),
                    view.point(40.7096, 36.7122),
                );
                path.cubic_bezier_to(
                    view.point(37.539, 33.0221),
                    view.point(38.6511, 33.9919),
                    view.point(37.6337, 32.9275),
                );
                path.cubic_bezier_to(
                    view.point(38.1306, 35.1747),
                    view.point(37.4207, 33.1404),
                    view.point(37.823, 34.5596),
                );
                path.cubic_bezier_to(
                    view.point(41.7034, 39.5743),
                    view.point(38.935, 36.7358),
                    view.point(40.1654, 38.2734),
                );
                path.cubic_bezier_to(
                    view.point(43.8329, 40.4969),
                    view.point(42.9574, 40.6624),
                    view.point(43.3597, 40.828),
                );
                path.close();
                path.move_to(view.point(16.1021, 38.2261));
                path.cubic_bezier_to(
                    view.point(14.4221, 36.9724),
                    view.point(15.7235, 38.0368),
                    view.point(14.9663, 37.4691),
                );
                path.cubic_bezier_to(
                    view.point(12.1743, 36.0735),
                    view.point(13.4047, 36.0499),
                    view.point(13.381, 36.0262),
                );
                path.cubic_bezier_to(
                    view.point(8.62515, 32.2179),
                    view.point(10.3051, 36.1445),
                    view.point(8.74345, 34.465),
                );
                path.line_to(view.point(8.57783, 31.2244));
                path.line_to(view.point(10.7546, 29.0718));
                path.cubic_bezier_to(
                    view.point(15.132, 23.655),
                    view.point(13.1208, 26.7301),
                    view.point(14.4694, 25.0743),
                );
                path.cubic_bezier_to(
                    view.point(17.711, 23.2056),
                    view.point(15.9364, 21.9756),
                    view.point(16.7173, 21.8573),
                );
                path.cubic_bezier_to(
                    view.point(17.1432, 26.7774),
                    view.point(18.5392, 24.3646),
                    view.point(18.4682, 24.9087),
                );
                path.cubic_bezier_to(
                    view.point(17.7347, 27.8418),
                    view.point(15.2503, 29.4267),
                    view.point(15.5342, 29.9234),
                );
                path.cubic_bezier_to(
                    view.point(20.716, 29.0718),
                    view.point(19.32, 26.3279),
                    view.point(19.3436, 26.3516),
                );
                path.cubic_bezier_to(
                    view.point(21.4258, 32.2888),
                    view.point(21.6861, 31.0115),
                    view.point(21.828, 31.6265),
                );
                path.cubic_bezier_to(
                    view.point(21.4968, 33.9683),
                    view.point(21.1892, 32.7146),
                    view.point(21.2129, 33.4242),
                );
                path.cubic_bezier_to(
                    view.point(20.2664, 35.4348),
                    view.point(22.0883, 35.1037),
                    view.point(21.7097, 35.5531),
                );
                path.cubic_bezier_to(
                    view.point(18.8468, 35.695),
                    view.point(19.2726, 35.3639),
                    view.point(19.249, 35.3639),
                );
                path.cubic_bezier_to(
                    view.point(18.0186, 36.3574),
                    view.point(18.6338, 35.8843),
                    view.point(18.2552, 36.1681),
                );
                path.cubic_bezier_to(
                    view.point(17.6164, 37.4691),
                    view.point(17.5927, 36.6885),
                    view.point(17.5927, 36.6885),
                );
                path.cubic_bezier_to(
                    view.point(16.1021, 38.2261),
                    view.point(17.6637, 38.6518),
                    view.point(17.3088, 38.8174),
                );
                path.close();
                path.move_to(view.point(36.3796, 30.5148));
                path.cubic_bezier_to(
                    view.point(35.0546, 29.5213),
                    view.point(36.427, 30.0653),
                    view.point(36.0011, 29.7342),
                );
                path.cubic_bezier_to(
                    view.point(34.3921, 29.7815),
                    view.point(33.9662, 29.2847),
                    view.point(33.9425, 29.2847),
                );
                path.cubic_bezier_to(
                    view.point(36.3796, 30.5148),
                    view.point(35.3385, 30.775),
                    view.point(36.3087, 31.1298),
                );
                path.close();
                path.move_to(view.point(7.51308, 27.3687));
                path.cubic_bezier_to(
                    view.point(6.8269, 26.3279),
                    view.point(7.34745, 27.1795),
                    view.point(7.03985, 26.7064),
                );
                path.cubic_bezier_to(
                    view.point(2.4496, 21.3132),
                    view.point(5.80948, 24.5066),
                    view.point(4.38981, 22.8744),
                );
                path.cubic_bezier_to(
                    view.point(1.26654, 19.4209),
                    view.point(1.21922, 20.3198),
                    view.point(1.02993, 20.0123),
                );
                path.cubic_bezier_to(
                    view.point(3.82194, 18.1436),
                    view.point(1.52681, 18.7822),
                    view.point(2.11834, 18.4984),
                );
                path.cubic_bezier_to(
                    view.point(9.83187, 18.2855),
                    view.point(5.76215, 17.7651),
                    view.point(7.41843, 17.8124),
                );
                path.cubic_bezier_to(
                    view.point(13.3337, 21.3369),
                    view.point(13.3574, 18.9715),
                    view.point(13.9016, 19.4682),
                );
                path.cubic_bezier_to(
                    view.point(12.056, 22.6142),
                    view.point(13.0498, 22.2358),
                    view.point(12.8605, 22.425),
                );
                path.cubic_bezier_to(
                    view.point(11.3462, 22.8744),
                    view.point(11.7248, 22.6852),
                    view.point(11.4172, 22.8035),
                );
                path.cubic_bezier_to(
                    view.point(10.7073, 25.2399),
                    view.point(11.1805, 23.0873),
                    view.point(10.7546, 24.6012),
                );
                path.cubic_bezier_to(
                    view.point(9.12203, 27.2268),
                    view.point(10.6363, 25.8785),
                    view.point(10.2578, 26.3279),
                );
                path.cubic_bezier_to(
                    view.point(7.51308, 27.3687),
                    view.point(8.38854, 27.7945),
                    view.point(7.93898, 27.8182),
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
