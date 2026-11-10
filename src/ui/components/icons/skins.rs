use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct SkinsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl SkinsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for SkinsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(21.6699, 51.8931));
                path.cubic_bezier_to(
                    view.point(13.249, 50.4978),
                    view.point(18.855, 51.7275),
                    view.point(14.4553, 50.9944),
                );
                path.cubic_bezier_to(
                    view.point(11.8297, 48.7004),
                    view.point(12.0426, 49.9775),
                    view.point(11.7587, 49.6228),
                );
                path.cubic_bezier_to(
                    view.point(13.1307, 44.9639),
                    view.point(11.9243, 47.5889),
                    view.point(12.6813, 45.3896),
                );
                path.cubic_bezier_to(
                    view.point(18.4529, 43.545),
                    view.point(13.9349, 44.1598),
                    view.point(16.4896, 43.474),
                );
                path.cubic_bezier_to(
                    view.point(18.6185, 44.3254),
                    view.point(19.8722, 43.5923),
                    view.point(19.9195, 43.7341),
                );
                path.cubic_bezier_to(
                    view.point(17.4358, 45.6261),
                    view.point(17.5777, 44.822),
                    view.point(17.2938, 45.1058),
                );
                path.cubic_bezier_to(
                    view.point(19.3991, 46.4301),
                    view.point(17.554, 46.0754),
                    view.point(18.0981, 46.3119),
                );
                path.cubic_bezier_to(
                    view.point(22.0484, 46.9268),
                    view.point(20.0614, 46.5011),
                    view.point(21.2441, 46.7139),
                );
                path.line_to(view.point(23.5149, 47.3052));
                path.line_to(view.point(26.6609, 47.3052));
                path.cubic_bezier_to(
                    view.point(31.0133, 46.9741),
                    view.point(29.7123, 47.3052),
                    view.point(29.8306, 47.2815),
                );
                path.cubic_bezier_to(
                    view.point(34.1594, 46.4301),
                    view.point(32.338, 46.643),
                    view.point(33.5207, 46.4301),
                );
                path.cubic_bezier_to(
                    view.point(35.9098, 45.3896),
                    view.point(35.1765, 46.4301),
                    view.point(35.9098, 45.9808),
                );
                path.cubic_bezier_to(
                    view.point(34.6797, 44.3254),
                    view.point(35.9098, 44.9639),
                    view.point(35.6969, 44.7747),
                );
                path.cubic_bezier_to(
                    view.point(34.9636, 43.5213),
                    view.point(33.2132, 43.6632),
                    view.point(33.2605, 43.5213),
                );
                path.cubic_bezier_to(
                    view.point(40.1675, 44.9639),
                    view.point(36.9979, 43.5213),
                    view.point(39.387, 44.1835),
                );
                path.cubic_bezier_to(
                    view.point(41.4922, 48.9606),
                    view.point(40.617, 45.4132),
                    view.point(41.4922, 48.0383),
                );
                path.cubic_bezier_to(
                    view.point(34.112, 51.6566),
                    view.point(41.4922, 50.2849),
                    view.point(39.7891, 50.8998),
                );
                path.cubic_bezier_to(
                    view.point(21.6699, 51.8931),
                    view.point(31.5101, 51.9877),
                    view.point(25.3363, 52.1059),
                );
                path.close();
                path.move_to(view.point(25.0288, 45.9099));
                path.cubic_bezier_to(
                    view.point(24.0353, 44.822),
                    view.point(24.3901, 45.6734),
                    view.point(24.0353, 45.295),
                );
                path.cubic_bezier_to(
                    view.point(22.7107, 43.2375),
                    view.point(24.0353, 43.947),
                    view.point(23.6805, 43.5213),
                );
                path.line_to(view.point(22.2376, 43.0956));
                path.line_to(view.point(22.8053, 43.001));
                path.cubic_bezier_to(
                    view.point(24.9578, 42.2916),
                    view.point(24.2482, 42.8118),
                    view.point(24.603, 42.6936),
                );
                path.cubic_bezier_to(
                    view.point(25.3126, 41.7003),
                    view.point(25.1471, 42.0551),
                    view.point(25.3126, 41.7949),
                );
                path.cubic_bezier_to(
                    view.point(26.0696, 40.5179),
                    view.point(25.3126, 41.3692),
                    view.point(25.7621, 40.6598),
                );
                path.cubic_bezier_to(
                    view.point(27.9146, 41.2037),
                    view.point(26.8265, 40.1631),
                    view.point(27.6071, 40.4469),
                );
                path.cubic_bezier_to(
                    view.point(29.7833, 42.9301),
                    view.point(28.4823, 42.5753),
                    view.point(28.5296, 42.6226),
                );
                path.line_to(view.point(30.6112, 43.1193));
                path.line_to(view.point(30.2327, 43.3085));
                path.cubic_bezier_to(
                    view.point(29.3812, 44.6565),
                    view.point(29.665, 43.6159),
                    view.point(29.3812, 44.0416),
                );
                path.cubic_bezier_to(
                    view.point(28.2694, 45.9572),
                    view.point(29.3812, 45.2713),
                    view.point(28.9791, 45.7443),
                );
                path.cubic_bezier_to(
                    view.point(25.0288, 45.9099),
                    view.point(27.6308, 46.1227),
                    view.point(25.5728, 46.0991),
                );
                path.close();
                path.move_to(view.point(20.2979, 41.5348));
                path.cubic_bezier_to(
                    view.point(13.1307, 37.1834),
                    view.point(17.4594, 40.5888),
                    view.point(15.5671, 39.4537),
                );
                path.cubic_bezier_to(
                    view.point(12.6813, 35.3387),
                    view.point(11.9953, 36.1192),
                    view.point(11.877, 35.7171),
                );
                path.cubic_bezier_to(
                    view.point(12.7049, 34.1326),
                    view.point(13.178, 35.0786),
                    view.point(13.2017, 34.8894),
                );
                path.cubic_bezier_to(
                    view.point(12.8468, 31.7677),
                    view.point(12.1845, 33.3049),
                    view.point(12.2082, 32.8556),
                );
                path.cubic_bezier_to(
                    view.point(15.6144, 31.1765),
                    view.point(13.9586, 29.8995),
                    view.point(14.2898, 29.8285),
                );
                path.cubic_bezier_to(
                    view.point(25.9986, 35.457),
                    view.point(17.7433, 33.3995),
                    view.point(22.143, 35.2205),
                );
                path.cubic_bezier_to(
                    view.point(37.3763, 31.6022),
                    view.point(29.5704, 35.6698),
                    view.point(34.9399, 33.8489),
                );
                path.cubic_bezier_to(
                    view.point(40.0729, 31.9806),
                    view.point(38.701, 30.3724),
                    view.point(39.1741, 30.4434),
                );
                path.cubic_bezier_to(
                    view.point(40.4041, 33.6597),
                    view.point(40.5697, 32.7846),
                    view.point(40.6406, 33.1394),
                );
                path.cubic_bezier_to(
                    view.point(37.471, 36.5448),
                    view.point(40.1912, 34.0853),
                    view.point(38.5827, 35.6935),
                );
                path.cubic_bezier_to(
                    view.point(35.1292, 38.2003),
                    view.point(35.9571, 37.7509),
                    view.point(35.7205, 37.9165),
                );
                path.cubic_bezier_to(
                    view.point(34.5142, 38.626),
                    view.point(34.7744, 38.3658),
                    view.point(34.5142, 38.555),
                );
                path.cubic_bezier_to(
                    view.point(35.413, 38.4368),
                    view.point(34.5142, 38.8388),
                    view.point(35.0819, 38.7442),
                );
                path.cubic_bezier_to(
                    view.point(36.17, 38.0347),
                    view.point(35.6259, 38.2949),
                    view.point(35.9571, 38.1057),
                );
                path.cubic_bezier_to(
                    view.point(38.7246, 36.5448),
                    view.point(36.6194, 37.8692),
                    view.point(37.6602, 37.278),
                );
                path.cubic_bezier_to(
                    view.point(41.0664, 35.7408),
                    view.point(40.3568, 35.4333),
                    view.point(40.7826, 35.2914),
                );
                path.cubic_bezier_to(
                    view.point(33.9228, 41.2274),
                    view.point(41.6105, 36.5921),
                    view.point(37.0688, 40.0686),
                );
                path.cubic_bezier_to(
                    view.point(29.7597, 41.0382),
                    view.point(31.2499, 42.197),
                    view.point(30.6112, 42.1733),
                );
                path.cubic_bezier_to(
                    view.point(27.2523, 38.8624),
                    view.point(28.6479, 39.5483),
                    view.point(27.9856, 38.9807),
                );
                path.cubic_bezier_to(
                    view.point(25.3126, 39.0516),
                    view.point(26.59, 38.7442),
                    view.point(25.6438, 38.8388),
                );
                path.cubic_bezier_to(
                    view.point(23.7042, 40.849),
                    view.point(25.0288, 39.2172),
                    view.point(23.9644, 40.3996),
                );
                path.cubic_bezier_to(
                    view.point(20.2979, 41.5348),
                    view.point(22.9472, 42.126),
                    view.point(22.3559, 42.2206),
                );
                path.close();
                path.move_to(view.point(24.4847, 33.9671));
                path.cubic_bezier_to(
                    view.point(15.9219, 30.4907),
                    view.point(20.842, 33.234),
                    view.point(18.1217, 32.1461),
                );
                path.cubic_bezier_to(
                    view.point(14.8575, 29.5447),
                    view.point(15.4252, 30.136),
                    view.point(14.9521, 29.6866),
                );
                path.cubic_bezier_to(
                    view.point(12.7522, 24.0109),
                    view.point(14.2188, 28.2204),
                    view.point(13.3672, 26.0447),
                );
                path.cubic_bezier_to(
                    view.point(12.2791, 22.6156),
                    view.point(12.563, 23.4433),
                    view.point(12.3501, 22.8048),
                );
                path.cubic_bezier_to(
                    view.point(11.2147, 18.5006),
                    view.point(12.0189, 22.0007),
                    view.point(11.2147, 18.8554),
                );
                path.cubic_bezier_to(
                    view.point(12.634, 16.254),
                    view.point(11.2147, 18.1459),
                    view.point(11.4749, 17.7675),
                );
                path.cubic_bezier_to(
                    view.point(15.4488, 7.43289),
                    view.point(14.3371, 14.0783),
                    view.point(15.4015, 10.6965),
                );
                path.cubic_bezier_to(
                    view.point(17.4121, 4.19297),
                    view.point(15.4961, 5.16258),
                    view.point(15.4961, 5.16258),
                );
                path.cubic_bezier_to(
                    view.point(20.8893, 4.16932),
                    view.point(19.5883, 3.12876),
                    view.point(19.6356, 3.12876),
                );
                path.cubic_bezier_to(
                    view.point(23.5149, 5.35177),
                    view.point(22.0957, 5.18623),
                    view.point(22.6161, 5.42272),
                );
                path.cubic_bezier_to(
                    view.point(23.8224, 4.85514),
                    view.point(23.7278, 5.35177),
                    view.point(23.7751, 5.23353),
                );
                path.cubic_bezier_to(
                    view.point(24.4374, 3.90918),
                    view.point(23.8934, 4.45311),
                    view.point(23.988, 4.28757),
                );
                path.cubic_bezier_to(
                    view.point(25.3126, 1.94631),
                    view.point(25.1234, 3.31796),
                    view.point(25.3126, 2.89227),
                );
                path.cubic_bezier_to(
                    view.point(26.1405, 0.148985),
                    view.point(25.3126, 1.024),
                    view.point(25.5728, 0.480072),
                );
                path.cubic_bezier_to(
                    view.point(28.1039, 1.96996),
                    view.point(27.134, -0.371293),
                    view.point(28.1039, 0.52737),
                );
                path.cubic_bezier_to(
                    view.point(28.9791, 3.88553),
                    view.point(28.1039, 2.89227),
                    view.point(28.2458, 3.19971),
                );
                path.cubic_bezier_to(
                    view.point(29.6177, 4.90244),
                    view.point(29.5231, 4.38216),
                    view.point(29.6177, 4.5477),
                );
                path.cubic_bezier_to(
                    view.point(32.8111, 3.90918),
                    view.point(29.6177, 5.8957),
                    view.point(30.9187, 5.49367),
                );
                path.cubic_bezier_to(
                    view.point(35.7915, 4.12202),
                    view.point(33.6626, 3.17606),
                    view.point(33.9465, 3.19971),
                );
                path.cubic_bezier_to(
                    view.point(37.9204, 7.50383),
                    view.point(37.8731, 5.18623),
                    view.point(37.8258, 5.13893),
                );
                path.cubic_bezier_to(
                    view.point(38.7956, 9.89239),
                    view.point(37.9913, 9.96333),
                    view.point(38.2515, 10.6728),
                );
                path.cubic_bezier_to(
                    view.point(44.0232, 9.18291),
                    view.point(39.2923, 9.18292),
                    view.point(39.4343, 9.18291),
                );
                path.cubic_bezier_to(
                    view.point(49.2508, 9.86874),
                    view.point(48.6121, 9.18291),
                    view.point(48.825, 9.23021),
                );
                path.cubic_bezier_to(
                    view.point(48.7304, 11.7843),
                    view.point(49.7239, 10.5546),
                    view.point(49.4873, 11.4059),
                );
                path.cubic_bezier_to(
                    view.point(39.6945, 11.737),
                    view.point(48.1154, 12.1154),
                    view.point(40.5697, 12.0444),
                );
                path.cubic_bezier_to(
                    view.point(38.8192, 12.2573),
                    view.point(38.8902, 11.4296),
                    view.point(38.8192, 11.4769),
                );
                path.cubic_bezier_to(
                    view.point(40.9481, 16.5141),
                    view.point(38.8192, 13.1323),
                    view.point(39.7654, 15.0479),
                );
                path.cubic_bezier_to(
                    view.point(41.847, 20.0142),
                    view.point(42.2728, 18.1459),
                    view.point(42.2964, 18.2641),
                );
                path.cubic_bezier_to(
                    view.point(39.316, 27.7238),
                    view.point(41.3503, 22.0007),
                    view.point(40.3568, 25.0041),
                );
                path.cubic_bezier_to(
                    view.point(34.6797, 32.1225),
                    view.point(38.559, 29.7576),
                    view.point(37.4, 30.8454),
                );
                path.cubic_bezier_to(
                    view.point(24.4847, 33.9671),
                    view.point(31.0843, 33.8016),
                    view.point(27.2286, 34.511),
                );
                path.close();
                path.move_to(view.point(27.0631, 27.3927));
                path.cubic_bezier_to(
                    view.point(27.4652, 26.9906),
                    view.point(27.2286, 27.2981),
                    view.point(27.4179, 27.1325),
                );
                path.cubic_bezier_to(
                    view.point(27.8673, 26.3521),
                    view.point(27.4888, 26.8724),
                    view.point(27.6781, 26.5886),
                );
                path.cubic_bezier_to(
                    view.point(27.9383, 24.673),
                    view.point(28.3167, 25.7845),
                    view.point(28.3167, 25.2643),
                );
                path.cubic_bezier_to(
                    view.point(27.6308, 23.5852),
                    view.point(27.7017, 24.3419),
                    view.point(27.6308, 24.1055),
                );
                path.cubic_bezier_to(
                    view.point(28.6242, 21.3622),
                    view.point(27.6308, 22.7102),
                    view.point(27.749, 22.4737),
                );
                path.cubic_bezier_to(
                    view.point(30.0908, 18.8554),
                    view.point(29.6414, 20.0378),
                    view.point(30.0908, 19.2811),
                );
                path.cubic_bezier_to(
                    view.point(29.121, 18.9027),
                    view.point(30.0908, 18.3587),
                    view.point(29.736, 18.3587),
                );
                path.cubic_bezier_to(
                    view.point(27.7727, 18.95),
                    view.point(28.5296, 19.3993),
                    view.point(28.0329, 19.4229),
                );
                path.cubic_bezier_to(
                    view.point(28.506, 15.379),
                    view.point(27.4179, 18.2641),
                    view.point(27.8437, 16.0884),
                );
                path.cubic_bezier_to(
                    view.point(30.5402, 15.6628),
                    view.point(29.2393, 14.5749),
                    view.point(30.5402, 14.7641),
                );
                path.cubic_bezier_to(
                    view.point(31.3208, 16.8216),
                    view.point(30.5402, 16.1121),
                    view.point(30.9424, 16.727),
                );
                path.cubic_bezier_to(
                    view.point(32.4326, 15.9229),
                    view.point(31.8412, 16.9398),
                    view.point(32.1487, 16.7033),
                );
                path.cubic_bezier_to(
                    view.point(33.3078, 14.5276),
                    view.point(32.5982, 15.4736),
                    view.point(32.9057, 14.9769),
                );
                path.cubic_bezier_to(
                    view.point(33.7809, 13.0614),
                    view.point(33.9465, 13.8181),
                    view.point(34.112, 13.2742),
                );
                path.cubic_bezier_to(
                    view.point(30.1618, 12.8958),
                    view.point(33.6153, 12.9431),
                    view.point(31.8176, 12.8722),
                );
                path.cubic_bezier_to(
                    view.point(28.0565, 11.2167),
                    view.point(28.9318, 12.9195),
                    view.point(28.8844, 12.8722),
                );
                path.cubic_bezier_to(
                    view.point(25.1707, 11.3586),
                    view.point(27.0394, 9.18292),
                    view.point(26.2115, 9.23021),
                );
                path.cubic_bezier_to(
                    view.point(23.1128, 12.8958),
                    view.point(24.4611, 12.8012),
                    view.point(24.3428, 12.8958),
                );
                path.cubic_bezier_to(
                    view.point(19.5173, 13.0614),
                    view.point(21.3624, 12.8722),
                    view.point(19.6593, 12.9668),
                );
                path.cubic_bezier_to(
                    view.point(19.9195, 14.433),
                    view.point(19.2098, 13.2506),
                    view.point(19.3518, 13.8181),
                );
                path.cubic_bezier_to(
                    view.point(20.7947, 15.8046),
                    view.point(20.2033, 14.7641),
                    view.point(20.6054, 15.379),
                );
                path.cubic_bezier_to(
                    view.point(21.3151, 16.7033),
                    view.point(20.9839, 16.2303),
                    view.point(21.2205, 16.6324),
                );
                path.cubic_bezier_to(
                    view.point(22.7343, 15.781),
                    view.point(21.8591, 17.129),
                    view.point(22.5688, 16.656),
                );
                path.cubic_bezier_to(
                    view.point(23.6805, 14.8823),
                    view.point(22.8763, 15.0242),
                    view.point(23.0182, 14.8823),
                );
                path.cubic_bezier_to(
                    view.point(25.36, 16.5378),
                    view.point(24.5321, 14.8823),
                    view.point(24.9815, 15.3317),
                );
                path.cubic_bezier_to(
                    view.point(24.2482, 18.9027),
                    view.point(26.0696, 18.7371),
                    view.point(25.5019, 19.9196),
                );
                path.cubic_bezier_to(
                    view.point(23.2311, 18.879),
                    view.point(23.6568, 18.406),
                    view.point(23.302, 18.3824),
                );
                path.cubic_bezier_to(
                    view.point(24.6267, 21.2912),
                    view.point(23.1838, 19.2574),
                    view.point(23.7751, 20.2743),
                );
                path.cubic_bezier_to(
                    view.point(25.6675, 23.4196),
                    view.point(25.5255, 22.3554),
                    view.point(25.6675, 22.6629),
                );
                path.cubic_bezier_to(
                    view.point(25.36, 24.5075),
                    view.point(25.6675, 23.9872),
                    view.point(25.5965, 24.1764),
                );
                path.cubic_bezier_to(
                    view.point(25.8567, 26.967),
                    view.point(24.7449, 25.3116),
                    view.point(24.8632, 25.8791),
                );
                path.cubic_bezier_to(
                    view.point(27.0631, 27.3927),
                    view.point(26.4717, 27.6292),
                    view.point(26.5663, 27.6528),
                );
                path.close();
                path.move_to(view.point(26.0932, 26.2102));
                path.cubic_bezier_to(
                    view.point(27.0394, 24.5311),
                    view.point(25.3836, 25.3825),
                    view.point(26.0223, 24.271),
                );
                path.cubic_bezier_to(
                    view.point(27.5835, 25.7136),
                    view.point(27.5598, 24.673),
                    view.point(27.7254, 25.0514),
                );
                path.cubic_bezier_to(
                    view.point(26.0932, 26.2102),
                    view.point(27.4415, 26.3521),
                    view.point(26.4954, 26.6595),
                );
                path.close();
                path.move_to(view.point(4.61515, 11.737));
                path.cubic_bezier_to(
                    view.point(4.49688, 9.514),
                    view.point(3.76359, 11.2167),
                    view.point(3.71628, 10.0579),
                );
                path.cubic_bezier_to(
                    view.point(9.15677, 9.23021),
                    view.point(4.87534, 9.23021),
                    view.point(4.87534, 9.23021),
                );
                path.cubic_bezier_to(
                    view.point(13.7457, 9.49035),
                    view.point(13.2963, 9.23021),
                    view.point(13.4382, 9.25386),
                );
                path.cubic_bezier_to(
                    view.point(14.0059, 10.9093),
                    view.point(14.2898, 9.86874),
                    view.point(14.3607, 10.2708),
                );
                path.cubic_bezier_to(
                    view.point(8.96754, 11.9735),
                    view.point(13.4382, 11.9972),
                    view.point(13.5092, 11.9735),
                );
                path.cubic_bezier_to(
                    view.point(4.61515, 11.737),
                    view.point(5.18285, 11.9735),
                    view.point(4.96996, 11.9499),
                );
                path.close();
                path.move_to(view.point(28.1039, 6.10854));
                path.cubic_bezier_to(
                    view.point(28.3167, 5.63556),
                    view.point(28.5769, 5.943),
                    view.point(28.6716, 5.73016),
                );
                path.cubic_bezier_to(
                    view.point(24.9815, 5.68286),
                    view.point(27.9146, 5.54096),
                    view.point(25.1471, 5.56461),
                );
                path.cubic_bezier_to(
                    view.point(28.1039, 6.10854),
                    view.point(24.2955, 6.10854),
                    view.point(27.0158, 6.48693),
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
