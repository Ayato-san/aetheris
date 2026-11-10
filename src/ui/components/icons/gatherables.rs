use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct GatherablesIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl GatherablesIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for GatherablesIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(40.6789, 51.3942));
                path.cubic_bezier_to(
                    view.point(33.6783, 48.8869),
                    view.point(38.3612, 51.0867),
                    view.point(35.594, 50.0932),
                );
                path.cubic_bezier_to(
                    view.point(29.9416, 47.6805),
                    view.point(31.8573, 47.7278),
                    view.point(31.6681, 47.6568),
                );
                path.cubic_bezier_to(
                    view.point(27.6475, 47.8461),
                    view.point(29.1138, 47.7041),
                    view.point(28.0732, 47.7751),
                );
                path.cubic_bezier_to(
                    view.point(16.1533, 46.2376),
                    view.point(24.5729, 48.3428),
                    view.point(19.9137, 47.7041),
                );
                path.cubic_bezier_to(
                    view.point(12.4402, 43.9431),
                    view.point(13.4808, 45.1968),
                    view.point(12.4402, 44.5581),
                );
                path.cubic_bezier_to(
                    view.point(14.9708, 42.9496),
                    view.point(12.4402, 43.4464),
                    view.point(12.8422, 43.2808),
                );
                path.cubic_bezier_to(
                    view.point(25.6608, 43.8722),
                    view.point(18.8968, 42.3346),
                    view.point(22.8701, 42.6658),
                );
                path.cubic_bezier_to(
                    view.point(28.2151, 44.3216),
                    view.point(28.1678, 44.9602),
                    view.point(28.3806, 44.9839),
                );
                path.cubic_bezier_to(
                    view.point(25.0459, 41.2702),
                    view.point(28.0022, 43.47),
                    view.point(26.8433, 42.3583),
                );
                path.cubic_bezier_to(
                    view.point(18.4711, 40.0638),
                    view.point(23.1302, 40.1111),
                    view.point(23.0829, 40.0875),
                );
                path.cubic_bezier_to(
                    view.point(4.82471, 35.4749),
                    view.point(11.2577, 40.0165),
                    view.point(8.9872, 39.2596),
                );
                path.cubic_bezier_to(
                    view.point(3.64218, 33.275),
                    view.point(3.26377, 34.032),
                    view.point(3.12187, 33.7718),
                );
                path.cubic_bezier_to(
                    view.point(11.0921, 32.7073),
                    view.point(4.5409, 32.4471),
                    view.point(8.63244, 32.1396),
                );
                path.cubic_bezier_to(
                    view.point(17.6906, 35.1674),
                    view.point(13.3862, 33.2277),
                    view.point(16.2479, 34.3158),
                );
                path.cubic_bezier_to(
                    view.point(20.6469, 37.4382),
                    view.point(18.613, 35.7351),
                    view.point(18.9914, 36.0426),
                );
                path.cubic_bezier_to(
                    view.point(22.1605, 37.0124),
                    view.point(21.9713, 38.5736),
                    view.point(22.7282, 38.3844),
                );
                path.cubic_bezier_to(
                    view.point(22.7755, 37.178),
                    view.point(21.9004, 36.4211),
                    view.point(21.9477, 36.4211),
                );
                path.cubic_bezier_to(
                    view.point(24.8094, 37.4382),
                    view.point(23.887, 38.1715),
                    view.point(24.5256, 38.2424),
                );
                path.cubic_bezier_to(
                    view.point(24.1945, 31.2408),
                    view.point(24.9513, 37.0361),
                    view.point(24.6202, 33.7245),
                );
                path.cubic_bezier_to(
                    view.point(27.5529, 17.9234),
                    view.point(23.3667, 26.3207),
                    view.point(24.5256, 21.6608),
                );
                path.cubic_bezier_to(
                    view.point(30.4619, 18.2309),
                    view.point(29.2084, 15.9128),
                    view.point(29.9416, 15.9837),
                );
                path.cubic_bezier_to(
                    view.point(28.2151, 34.6706),
                    view.point(31.8573, 24.1918),
                    view.point(31.0295, 30.2946),
                );
                path.cubic_bezier_to(
                    view.point(26.6541, 40.253),
                    view.point(26.867, 36.7522),
                    view.point(26.4176, 38.3844),
                );
                path.cubic_bezier_to(
                    view.point(29.4686, 44.227),
                    view.point(26.867, 42.0035),
                    view.point(27.3164, 42.6185),
                );
                path.cubic_bezier_to(
                    view.point(31.7627, 43.0442),
                    view.point(32.2357, 46.2849),
                    view.point(32.756, 46.001),
                );
                path.cubic_bezier_to(
                    view.point(31.0768, 33.89),
                    view.point(30.4619, 39.1886),
                    view.point(30.32, 37.2016),
                );
                path.cubic_bezier_to(
                    view.point(33.5128, 28.213),
                    view.point(31.5971, 31.6902),
                    view.point(32.9215, 28.5915),
                );
                path.cubic_bezier_to(
                    view.point(35.0501, 30.0817),
                    view.point(34.1514, 27.7872),
                    view.point(34.8845, 28.6861),
                );
                path.cubic_bezier_to(
                    view.point(35.594, 31.9504),
                    view.point(35.0974, 30.4365),
                    view.point(35.3575, 31.2881),
                );
                path.cubic_bezier_to(
                    view.point(35.5231, 40.7971),
                    view.point(36.8948, 35.333),
                    view.point(36.8712, 37.722),
                );
                path.cubic_bezier_to(
                    view.point(34.6953, 47.1838),
                    view.point(34.4115, 43.3281),
                    view.point(34.0331, 46.1666),
                );
                path.cubic_bezier_to(
                    view.point(36.8239, 48.3192),
                    view.point(34.9791, 47.6332),
                    view.point(35.2866, 47.7988),
                );
                path.cubic_bezier_to(
                    view.point(42.6419, 48.9815),
                    view.point(38.5031, 48.9105),
                    view.point(40.2059, 49.0997),
                );
                path.line_to(view.point(44.2738, 48.9105));
                path.line_to(view.point(44.6049, 49.218));
                path.cubic_bezier_to(
                    view.point(43.8954, 51.134),
                    view.point(45.1962, 49.8094),
                    view.point(44.9123, 50.6136),
                );
                path.cubic_bezier_to(
                    view.point(40.6789, 51.3942),
                    view.point(43.3987, 51.3942),
                    view.point(41.7668, 51.5361),
                );
                path.close();
                path.move_to(view.point(47.5612, 46.3085));
                path.cubic_bezier_to(
                    view.point(43.2805, 42.7841),
                    view.point(46.0712, 45.9301),
                    view.point(44.5339, 44.6764),
                );
                path.cubic_bezier_to(
                    view.point(40.2059, 40.1111),
                    view.point(42.1216, 41.0573),
                    view.point(41.5776, 40.5842),
                );
                path.cubic_bezier_to(
                    view.point(37.7935, 37.4855),
                    view.point(38.03, 39.4015),
                    view.point(37.557, 38.8811),
                );
                path.cubic_bezier_to(
                    view.point(39.4727, 36.492),
                    view.point(38.0773, 35.877),
                    view.point(38.4085, 35.6878),
                );
                path.cubic_bezier_to(
                    view.point(40.3241, 36.2791),
                    view.point(40.2295, 37.0597),
                    view.point(40.3241, 37.0361),
                );
                path.cubic_bezier_to(
                    view.point(37.3915, 31.501),
                    view.point(40.3241, 35.0491),
                    view.point(38.645, 32.3052),
                );
                path.cubic_bezier_to(
                    view.point(35.5231, 28.284),
                    view.point(36.5401, 30.9569),
                    view.point(35.594, 29.3248),
                );
                path.cubic_bezier_to(
                    view.point(36.5637, 28.0474),
                    view.point(35.4521, 27.1013),
                    view.point(35.7832, 27.0303),
                );
                path.cubic_bezier_to(
                    view.point(38.03, 29.3721),
                    view.point(37.557, 29.3957),
                    view.point(37.5334, 29.3721),
                );
                path.cubic_bezier_to(
                    view.point(38.6923, 29.8925),
                    view.point(38.4558, 29.3721),
                    view.point(38.5031, 29.4194),
                );
                path.cubic_bezier_to(
                    view.point(40.7499, 30.6967),
                    view.point(39.1889, 31.1225),
                    view.point(40.3005, 31.5719),
                );
                path.cubic_bezier_to(
                    view.point(41.4121, 28.7334),
                    view.point(40.8445, 30.4838),
                    view.point(41.1519, 29.6086),
                );
                path.cubic_bezier_to(
                    view.point(44.7468, 21.0931),
                    view.point(42.9257, 23.7424),
                    view.point(43.2095, 23.0564),
                );
                path.cubic_bezier_to(
                    view.point(51.771, 16.9536),
                    view.point(47.0645, 18.089),
                    view.point(51.1561, 15.6762),
                );
                path.cubic_bezier_to(
                    view.point(50.8013, 21.5188),
                    view.point(52.0075, 17.474),
                    view.point(51.5345, 19.6502),
                );
                path.cubic_bezier_to(
                    view.point(44.0373, 30.3892),
                    view.point(49.1458, 25.7057),
                    view.point(47.1355, 28.3313),
                );
                path.cubic_bezier_to(
                    view.point(41.2465, 35.5931),
                    view.point(41.4357, 32.116),
                    view.point(40.4188, 34.0083),
                );
                path.cubic_bezier_to(
                    view.point(43.9663, 39.6144),
                    view.point(42.7365, 38.4553),
                    view.point(43.4697, 39.5671),
                );
                path.cubic_bezier_to(
                    view.point(44.7941, 37.4145),
                    view.point(44.4393, 39.6617),
                    view.point(44.5339, 39.3778),
                );
                path.cubic_bezier_to(
                    view.point(47.916, 30.3182),
                    view.point(45.1489, 34.5997),
                    view.point(46.0476, 32.5654),
                );
                path.cubic_bezier_to(
                    view.point(51.6764, 27.4088),
                    view.point(49.3113, 28.6388),
                    view.point(51.0851, 27.2432),
                );
                path.cubic_bezier_to(
                    view.point(51.8656, 29.4667),
                    view.point(52.0548, 27.5034),
                    view.point(52.0785, 27.9528),
                );
                path.cubic_bezier_to(
                    view.point(51.5581, 32.1869),
                    view.point(51.7474, 30.2236),
                    view.point(51.6054, 31.4536),
                );
                path.cubic_bezier_to(
                    view.point(48.0342, 38.2661),
                    view.point(51.3926, 34.3158),
                    view.point(50.5175, 35.8297),
                );
                path.cubic_bezier_to(
                    view.point(45.8111, 42.1454),
                    view.point(46.3077, 39.9455),
                    view.point(45.5746, 41.2229),
                );
                path.cubic_bezier_to(
                    view.point(48.0342, 44.156),
                    view.point(45.9293, 42.6185),
                    view.point(47.0172, 43.612),
                );
                path.cubic_bezier_to(
                    view.point(49.0985, 46.1193),
                    view.point(49.3587, 44.8893),
                    view.point(49.7134, 45.5516),
                );
                path.cubic_bezier_to(
                    view.point(47.5612, 46.3085),
                    view.point(48.7437, 46.4505),
                    view.point(48.318, 46.5214),
                );
                path.close();
                path.move_to(view.point(20.7179, 34.9545));
                path.cubic_bezier_to(
                    view.point(19.2515, 33.1804),
                    view.point(19.7245, 33.6535),
                    view.point(19.5353, 33.417),
                );
                path.cubic_bezier_to(
                    view.point(18.14, 31.8558),
                    view.point(19.086, 33.0385),
                    view.point(18.5657, 32.4471),
                );
                path.cubic_bezier_to(
                    view.point(15.1127, 30.2),
                    view.point(17.0757, 30.4602),
                    view.point(16.8865, 30.3419),
                );
                path.cubic_bezier_to(
                    view.point(1.34808, 23.3639),
                    view.point(9.1764, 29.7269),
                    view.point(3.99694, 27.1722),
                );
                path.cubic_bezier_to(
                    view.point(0.543962, 22.3468),
                    view.point(1.20618, 23.151),
                    view.point(0.827769, 22.7016),
                );
                path.cubic_bezier_to(
                    view.point(0.0, 21.3296),
                    view.point(0.141903, 21.8737),
                    view.point(0.0, 21.6135),
                );
                path.cubic_bezier_to(
                    view.point(2.71981, 20.8565),
                    view.point(0.0, 20.62),
                    view.point(0.402059, 20.5727),
                );
                path.cubic_bezier_to(
                    view.point(13.2916, 26.8647),
                    view.point(6.85865, 21.4006),
                    view.point(10.5954, 23.5295),
                );
                path.cubic_bezier_to(
                    view.point(15.5384, 28.7807),
                    view.point(14.3322, 28.142),
                    view.point(15.0654, 28.7807),
                );
                path.cubic_bezier_to(
                    view.point(15.3492, 26.8411),
                    view.point(16.0351, 28.7807),
                    view.point(15.9641, 27.8819),
                );
                path.cubic_bezier_to(
                    view.point(14.3795, 25.2089),
                    view.point(15.089, 26.4153),
                    view.point(14.6633, 25.682),
                );
                path.cubic_bezier_to(
                    view.point(13.7883, 24.0735),
                    view.point(14.1194, 24.7358),
                    view.point(13.8356, 24.2391),
                );
                path.cubic_bezier_to(
                    view.point(11.0921, 20.7856),
                    view.point(13.67, 23.766),
                    view.point(12.6294, 22.4887),
                );
                path.cubic_bezier_to(
                    view.point(6.76405, 14.1151),
                    view.point(8.82165, 18.2782),
                    view.point(7.35531, 16.0074),
                );
                path.cubic_bezier_to(
                    view.point(6.6931, 5.19739),
                    view.point(6.03088, 11.7733),
                    view.point(6.00723, 8.36707),
                );
                path.cubic_bezier_to(
                    view.point(7.11881, 3.13947),
                    view.point(6.85865, 4.44045),
                    view.point(7.04786, 3.51794),
                );
                path.cubic_bezier_to(
                    view.point(8.56149, 2.42984),
                    view.point(7.35531, 1.90945),
                    view.point(7.85197, 1.6729),
                );
                path.cubic_bezier_to(
                    view.point(14.7343, 15.9837),
                    view.point(12.251, 6.28549),
                    view.point(14.3559, 10.8981),
                );
                path.cubic_bezier_to(
                    view.point(14.4978, 21.235),
                    view.point(14.8525, 17.6395),
                    view.point(14.7343, 20.1942),
                );
                path.cubic_bezier_to(
                    view.point(15.633, 25.9422),
                    view.point(14.1667, 22.5596),
                    view.point(14.3322, 23.2693),
                );
                path.cubic_bezier_to(
                    view.point(17.596, 28.142),
                    view.point(16.65, 28.0238),
                    view.point(17.1466, 28.5915),
                );
                path.cubic_bezier_to(
                    view.point(17.3831, 25.3272),
                    view.point(17.8561, 27.9055),
                    view.point(17.7615, 26.7938),
                );
                path.cubic_bezier_to(
                    view.point(18.4474, 14.0441),
                    view.point(16.3425, 21.3533),
                    view.point(16.65, 18.0653),
                );
                path.cubic_bezier_to(
                    view.point(19.5117, 11.6314),
                    view.point(18.8022, 13.2162),
                    view.point(19.2988, 12.1281),
                );
                path.cubic_bezier_to(
                    view.point(21.522, 11.7023),
                    view.point(20.3868, 9.62074),
                    view.point(20.978, 9.6444),
                );
                path.cubic_bezier_to(
                    view.point(22.1369, 13.8076),
                    view.point(21.6875, 12.2937),
                    view.point(21.9713, 13.2398),
                );
                path.cubic_bezier_to(
                    view.point(22.5153, 21.164),
                    view.point(22.7045, 15.7235),
                    view.point(22.8464, 18.6803),
                );
                path.cubic_bezier_to(
                    view.point(20.9307, 25.753),
                    view.point(22.3261, 22.6779),
                    view.point(22.1132, 23.2929),
                );
                path.cubic_bezier_to(
                    view.point(19.6299, 30.815),
                    view.point(19.7718, 28.1894),
                    view.point(19.6299, 28.7571),
                );
                path.cubic_bezier_to(
                    view.point(20.5996, 34.3868),
                    view.point(19.6536, 32.8256),
                    view.point(19.7245, 33.1094),
                );
                path.cubic_bezier_to(
                    view.point(20.7179, 34.9545),
                    view.point(21.3328, 35.4985),
                    view.point(21.4274, 35.9007),
                );
                path.close();
                path.move_to(view.point(37.6043, 25.0433));
                path.cubic_bezier_to(
                    view.point(37.2259, 23.7424),
                    view.point(37.4861, 24.9014),
                    view.point(37.3442, 24.3574),
                );
                path.cubic_bezier_to(
                    view.point(35.7596, 21.3296),
                    view.point(36.9658, 22.1102),
                    view.point(36.682, 21.6608),
                );
                path.cubic_bezier_to(
                    view.point(32.6614, 19.4609),
                    view.point(34.8136, 21.0221),
                    view.point(33.9149, 20.4781),
                );
                path.cubic_bezier_to(
                    view.point(31.266, 17.1901),
                    view.point(31.5971, 18.6094),
                    view.point(31.5025, 18.4201),
                );
                path.cubic_bezier_to(
                    view.point(29.2084, 14.1151),
                    view.point(30.8639, 14.9903),
                    view.point(30.3909, 14.257),
                );
                path.cubic_bezier_to(
                    view.point(27.2218, 11.6314),
                    view.point(28.0259, 13.9731),
                    view.point(27.8367, 13.7129),
                );
                path.cubic_bezier_to(
                    view.point(27.34, 8.60361),
                    view.point(26.5359, 9.19497),
                    view.point(26.5359, 8.60361),
                );
                path.cubic_bezier_to(
                    view.point(31.6208, 11.6314),
                    view.point(27.9076, 8.60361),
                    view.point(30.1781, 10.2121),
                );
                path.cubic_bezier_to(
                    view.point(35.0974, 17.2611),
                    view.point(33.2053, 13.2162),
                    view.point(34.7663, 15.7472),
                );
                path.cubic_bezier_to(
                    view.point(35.8542, 18.9878),
                    view.point(35.3102, 18.3492),
                    view.point(35.5231, 18.7986),
                );
                path.cubic_bezier_to(
                    view.point(35.8305, 16.6934),
                    view.point(36.4455, 19.319),
                    view.point(36.4455, 18.2546),
                );
                path.cubic_bezier_to(
                    view.point(34.6244, 11.0873),
                    view.point(35.5231, 15.8891),
                    view.point(35.0028, 13.4527),
                );
                path.cubic_bezier_to(
                    view.point(37.6516, 2.28791),
                    view.point(34.2696, 8.84015),
                    view.point(35.3102, 5.8124),
                );
                path.cubic_bezier_to(
                    view.point(40.6553, 1.55463),
                    view.point(39.5437, -0.550601),
                    view.point(40.1113, -0.692527),
                );
                path.cubic_bezier_to(
                    view.point(40.5134, 12.9797),
                    view.point(41.7905, 6.04894),
                    view.point(41.7668, 7.63378),
                );
                path.cubic_bezier_to(
                    view.point(41.9087, 12.6485),
                    view.point(40.1822, 14.3989),
                    view.point(40.3005, 14.3516),
                );
                path.cubic_bezier_to(
                    view.point(46.0476, 8.79284),
                    view.point(43.1622, 11.3002),
                    view.point(45.3381, 9.28958),
                );
                path.cubic_bezier_to(
                    view.point(47.2065, 9.71536),
                    view.point(46.9699, 8.17783),
                    view.point(47.3957, 8.50899),
                );
                path.cubic_bezier_to(
                    view.point(40.7499, 21.5898),
                    view.point(46.1895, 15.8891),
                    view.point(45.409, 17.3084),
                );
                path.cubic_bezier_to(
                    view.point(38.8815, 23.837),
                    view.point(39.591, 22.6543),
                    view.point(39.3545, 22.9618),
                );
                path.cubic_bezier_to(
                    view.point(37.6043, 25.0433),
                    view.point(38.1246, 25.3035),
                    view.point(37.9827, 25.4455),
                );
                path.close();
                path.move_to(view.point(38.3848, 20.3125));
                path.cubic_bezier_to(
                    view.point(39.1653, 18.089),
                    view.point(38.6923, 20.1706),
                    view.point(38.9524, 19.39),
                );
                path.cubic_bezier_to(
                    view.point(39.4254, 16.5514),
                    view.point(39.2599, 17.4976),
                    view.point(39.3781, 16.788),
                );
                path.cubic_bezier_to(
                    view.point(39.4018, 16.102),
                    view.point(39.4727, 16.2913),
                    view.point(39.4491, 16.102),
                );
                path.cubic_bezier_to(
                    view.point(37.6989, 18.1599),
                    view.point(39.2362, 16.102),
                    view.point(38.0064, 17.6159),
                );
                path.cubic_bezier_to(
                    view.point(37.7462, 20.1942),
                    view.point(37.2969, 18.9169),
                    view.point(37.3205, 19.863),
                );
                path.cubic_bezier_to(
                    view.point(38.3848, 20.3125),
                    view.point(38.101, 20.4781),
                    view.point(38.101, 20.4781),
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
