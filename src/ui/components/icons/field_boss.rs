use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct FieldBossIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl FieldBossIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for FieldBossIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(25.5927, 44.746));
                path.cubic_bezier_to(
                    view.point(25.0961, 43.9417),
                    view.point(25.4981, 44.6514),
                    view.point(25.2853, 44.2965),
                );
                path.cubic_bezier_to(
                    view.point(24.4575, 42.9246),
                    view.point(24.9069, 43.5869),
                    view.point(24.6231, 43.1375),
                );
                path.cubic_bezier_to(
                    view.point(23.8899, 42.144),
                    view.point(24.292, 42.7354),
                    view.point(24.0318, 42.3805),
                );
                path.cubic_bezier_to(
                    view.point(23.0858, 41.458),
                    view.point(23.6534, 41.7892),
                    view.point(23.5115, 41.6709),
                );
                path.cubic_bezier_to(
                    view.point(22.3527, 41.0086),
                    view.point(22.7784, 41.3161),
                    view.point(22.4709, 41.1269),
                );
                path.cubic_bezier_to(
                    view.point(20.91, 38.0991),
                    view.point(22.1162, 40.7484),
                    view.point(21.0756, 38.6668),
                );
                path.cubic_bezier_to(
                    view.point(19.964, 36.8691),
                    view.point(20.7681, 37.5787),
                    view.point(20.5316, 37.2712),
                );
                path.cubic_bezier_to(
                    view.point(19.2309, 35.7337),
                    view.point(19.4674, 36.4906),
                    view.point(19.2072, 36.0885),
                );
                path.cubic_bezier_to(
                    view.point(21.0519, 36.7035),
                    view.point(19.3018, 35.0477),
                    view.point(20.3661, 35.6154),
                );
                path.cubic_bezier_to(
                    view.point(22.3527, 37.9335),
                    view.point(21.4303, 37.2949),
                    view.point(21.7851, 37.6497),
                );
                path.cubic_bezier_to(
                    view.point(23.819, 39.2109),
                    view.point(22.5655, 38.0518),
                    view.point(23.1804, 38.5722),
                );
                path.cubic_bezier_to(
                    view.point(28.1469, 39.2582),
                    view.point(25.7346, 41.1032),
                    view.point(26.2076, 41.1269),
                );
                path.cubic_bezier_to(
                    view.point(29.6606, 37.8862),
                    view.point(28.7146, 38.7141),
                    view.point(29.4004, 38.0991),
                );
                path.cubic_bezier_to(
                    view.point(30.5593, 37.2002),
                    view.point(29.9207, 37.697),
                    view.point(30.3228, 37.3895),
                );
                path.cubic_bezier_to(
                    view.point(31.1032, 36.6325),
                    view.point(30.7721, 37.0347),
                    view.point(31.0323, 36.7745),
                );
                path.cubic_bezier_to(
                    view.point(32.0492, 35.5918),
                    view.point(31.4107, 36.0648),
                    view.point(31.6945, 35.7573),
                );
                path.cubic_bezier_to(
                    view.point(32.0492, 36.8691),
                    view.point(33.0425, 35.1187),
                    view.point(33.0425, 36.0175),
                );
                path.cubic_bezier_to(
                    view.point(30.3228, 39.7549),
                    view.point(31.5526, 37.3185),
                    view.point(31.0323, 38.1701),
                );
                path.cubic_bezier_to(
                    view.point(29.0693, 41.3634),
                    view.point(29.8261, 40.843),
                    view.point(29.6133, 41.1032),
                );
                path.cubic_bezier_to(
                    view.point(27.9341, 42.5225),
                    view.point(28.5254, 41.5999),
                    view.point(28.2652, 41.8838),
                );
                path.cubic_bezier_to(
                    view.point(27.2246, 43.5396),
                    view.point(27.7685, 42.8063),
                    view.point(27.4611, 43.2794),
                );
                path.cubic_bezier_to(
                    view.point(26.6097, 44.4148),
                    view.point(26.9881, 43.8234),
                    view.point(26.7279, 44.2019),
                );
                path.cubic_bezier_to(
                    view.point(25.5927, 44.746),
                    view.point(26.3495, 44.9589),
                    view.point(25.9238, 45.0771),
                );
                path.close();
                path.move_to(view.point(24.8832, 37.626));
                path.cubic_bezier_to(
                    view.point(22.8493, 35.0004),
                    view.point(24.2447, 37.3895),
                    view.point(23.1568, 35.9939),
                );
                path.cubic_bezier_to(
                    view.point(21.9506, 34.0542),
                    view.point(22.6601, 34.409),
                    view.point(22.4709, 34.2198),
                );
                path.cubic_bezier_to(
                    view.point(21.454, 33.1081),
                    view.point(21.2411, 33.865),
                    view.point(21.0992, 33.5575),
                );
                path.cubic_bezier_to(
                    view.point(21.454, 32.2328),
                    view.point(21.6668, 32.8242),
                    view.point(21.6668, 32.4221),
                );
                path.cubic_bezier_to(
                    view.point(19.8221, 32.0909),
                    view.point(21.2884, 32.0909),
                    view.point(21.1465, 32.0673),
                );
                path.cubic_bezier_to(
                    view.point(17.5517, 32.6823),
                    view.point(18.1666, 32.1382),
                    view.point(17.9774, 32.1855),
                );
                path.cubic_bezier_to(
                    view.point(15.707, 34.1725),
                    view.point(17.0077, 33.3919),
                    view.point(16.0144, 34.1725),
                );
                path.cubic_bezier_to(
                    view.point(14.5245, 33.2263),
                    view.point(15.3049, 34.1725),
                    view.point(14.761, 33.7467),
                );
                path.cubic_bezier_to(
                    view.point(14.7137, 32.02),
                    view.point(14.3116, 32.8006),
                    view.point(14.3589, 32.5877),
                );
                path.cubic_bezier_to(
                    view.point(15.7543, 28.5901),
                    view.point(15.0921, 31.4523),
                    view.point(15.6597, 29.5836),
                );
                path.cubic_bezier_to(
                    view.point(16.8895, 27.8568),
                    view.point(15.8725, 27.2418),
                    view.point(16.2746, 26.9816),
                );
                path.cubic_bezier_to(
                    view.point(17.9065, 28.8976),
                    view.point(17.0314, 28.046),
                    view.point(17.5044, 28.5191),
                );
                path.cubic_bezier_to(
                    view.point(18.8761, 29.7965),
                    view.point(18.3322, 29.2761),
                    view.point(18.7815, 29.6782),
                );
                path.cubic_bezier_to(
                    view.point(20.8864, 29.6782),
                    view.point(19.6566, 30.5061),
                    view.point(20.0113, 30.4824),
                );
                path.cubic_bezier_to(
                    view.point(22.7311, 28.7557),
                    view.point(21.7378, 28.8739),
                    view.point(21.927, 28.7793),
                );
                path.cubic_bezier_to(
                    view.point(24.292, 27.0762),
                    view.point(23.9372, 28.6847),
                    view.point(24.292, 28.3062),
                );
                path.cubic_bezier_to(
                    view.point(23.0149, 25.2548),
                    view.point(24.292, 26.1064),
                    view.point(24.2683, 26.0591),
                );
                path.cubic_bezier_to(
                    view.point(22.4236, 24.8291),
                    view.point(22.8257, 25.1129),
                    view.point(22.5419, 24.9237),
                );
                path.cubic_bezier_to(
                    view.point(21.0283, 24.5925),
                    view.point(22.2108, 24.6635),
                    view.point(22.0689, 24.6398),
                );
                path.cubic_bezier_to(
                    view.point(19.6566, 24.3796),
                    view.point(20.0586, 24.5689),
                    view.point(19.8694, 24.5216),
                );
                path.cubic_bezier_to(
                    view.point(18.9471, 23.1969),
                    view.point(19.3964, 24.1904),
                    view.point(19.4201, 24.214),
                );
                path.cubic_bezier_to(
                    view.point(17.9065, 20.7369),
                    view.point(18.2612, 21.754),
                    view.point(18.0957, 21.3282),
                );
                path.cubic_bezier_to(
                    view.point(17.3625, 19.4595),
                    view.point(17.8118, 20.4294),
                    view.point(17.5753, 19.838),
                );
                path.cubic_bezier_to(
                    view.point(16.7476, 18.1585),
                    view.point(17.1733, 19.0811),
                    view.point(16.8895, 18.4897),
                );
                path.cubic_bezier_to(
                    view.point(16.0381, 17.1651),
                    view.point(16.4638, 17.5199),
                    view.point(16.2273, 17.1651),
                );
                path.cubic_bezier_to(
                    view.point(15.7543, 18.8209),
                    view.point(15.6597, 17.1651),
                    view.point(15.5178, 17.993),
                );
                path.cubic_bezier_to(
                    view.point(15.9198, 19.7197),
                    view.point(15.8489, 19.1757),
                    view.point(15.9198, 19.5778),
                );
                path.cubic_bezier_to(
                    view.point(16.0854, 20.4767),
                    view.point(15.9198, 19.8853),
                    view.point(15.9908, 20.2165),
                );
                path.cubic_bezier_to(
                    view.point(16.4874, 21.9905),
                    view.point(16.2746, 20.9734),
                    view.point(16.3692, 21.2809),
                );
                path.cubic_bezier_to(
                    view.point(17.3152, 23.7173),
                    view.point(16.6766, 22.9604),
                    view.point(16.7476, 23.1259),
                );
                path.cubic_bezier_to(
                    view.point(17.9301, 25.4914),
                    view.point(18.3795, 24.8291),
                    view.point(18.4504, 25.0656),
                );
                path.cubic_bezier_to(
                    view.point(16.109, 25.8935),
                    view.point(17.2206, 26.0827),
                    view.point(16.5111, 26.2483),
                );
                path.cubic_bezier_to(
                    view.point(15.8725, 24.9237),
                    view.point(15.9198, 25.7043),
                    view.point(15.8962, 25.657),
                );
                path.cubic_bezier_to(
                    view.point(15.1394, 22.298),
                    view.point(15.8489, 24.1431),
                    view.point(15.5178, 22.9604),
                );
                path.cubic_bezier_to(
                    view.point(14.8556, 21.7067),
                    view.point(15.0684, 22.1798),
                    view.point(14.9502, 21.9196),
                );
                path.cubic_bezier_to(
                    view.point(14.3826, 20.9498),
                    view.point(14.761, 21.4938),
                    view.point(14.5481, 21.1626),
                );
                path.cubic_bezier_to(
                    view.point(13.5312, 19.6961),
                    view.point(14.1934, 20.7605),
                    view.point(13.815, 20.1928),
                );
                path.cubic_bezier_to(
                    view.point(12.6325, 18.5607),
                    view.point(13.0818, 18.9391),
                    view.point(12.9399, 18.7499),
                );
                path.cubic_bezier_to(
                    view.point(11.7101, 17.3306),
                    view.point(11.8757, 18.1349),
                    view.point(11.7811, 17.993),
                );
                path.cubic_bezier_to(
                    view.point(11.5209, 16.3845),
                    view.point(11.6628, 16.9758),
                    view.point(11.5919, 16.5501),
                );
                path.cubic_bezier_to(
                    view.point(11.3317, 15.5802),
                    view.point(11.45, 16.1952),
                    view.point(11.379, 15.8404),
                );
                path.cubic_bezier_to(
                    view.point(11.0952, 14.6814),
                    view.point(11.3081, 15.32),
                    view.point(11.2135, 14.9179),
                );
                path.cubic_bezier_to(
                    view.point(10.7878, 13.7825),
                    view.point(10.977, 14.4212),
                    view.point(10.8351, 14.0427),
                );
                path.cubic_bezier_to(
                    view.point(9.29781, 12.5052),
                    view.point(10.6459, 13.0965),
                    view.point(9.96001, 12.5052),
                );
                path.cubic_bezier_to(
                    view.point(7.6896, 14.09),
                    view.point(8.75386, 12.5052),
                    view.point(8.13896, 13.1202),
                );
                path.cubic_bezier_to(
                    view.point(7.47675, 16.8576),
                    view.point(7.47675, 14.5394),
                    view.point(7.47675, 14.5394),
                );
                path.cubic_bezier_to(
                    view.point(6.81455, 23.5517),
                    view.point(7.5004, 20.5476),
                    view.point(7.38215, 21.7303),
                );
                path.line_to(view.point(6.48345, 24.6398));
                path.line_to(view.point(5.89219, 25.2075));
                path.cubic_bezier_to(
                    view.point(2.93593, 26.6504),
                    view.point(4.56779, 26.4849),
                    view.point(3.92923, 26.7924),
                );
                path.cubic_bezier_to(
                    view.point(0.0269658, 24.6635),
                    view.point(1.13852, 26.4139),
                    view.point(-0.209535, 25.4677),
                );
                path.cubic_bezier_to(
                    view.point(0.92567, 24.5925),
                    view.point(0.145216, 24.285),
                    view.point(0.452668, 24.2614),
                );
                path.cubic_bezier_to(
                    view.point(3.92923, 24.1194),
                    view.point(1.91897, 25.2312),
                    view.point(3.50353, 24.9946),
                );
                path.cubic_bezier_to(
                    view.point(4.35494, 20.7842),
                    view.point(4.59144, 22.7948),
                    view.point(4.70969, 21.8013),
                );
                path.cubic_bezier_to(
                    view.point(4.18939, 20.1219),
                    view.point(4.26034, 20.5476),
                    view.point(4.18939, 20.2401),
                );
                path.cubic_bezier_to(
                    view.point(3.90558, 19.3176),
                    view.point(4.18939, 20.0272),
                    view.point(4.04749, 19.6488),
                );
                path.cubic_bezier_to(
                    view.point(3.14878, 17.3306),
                    view.point(3.74003, 18.9628),
                    view.point(3.38528, 18.0639),
                );
                path.line_to(view.point(2.67578, 15.9587));
                path.line_to(view.point(2.62848, 14.6341));
                path.line_to(view.point(2.60483, 13.2858));
                path.line_to(view.point(2.91228, 12.2686));
                path.cubic_bezier_to(
                    view.point(4.63874, 9.12261),
                    view.point(3.36163, 10.7548),
                    view.point(3.66908, 10.1871),
                );
                path.cubic_bezier_to(
                    view.point(6.57805, 7.56143),
                    view.point(5.56109, 8.10548),
                    view.point(5.93949, 7.82162),
                );
                path.cubic_bezier_to(
                    view.point(7.4058, 7.20661),
                    view.point(6.81455, 7.49046),
                    view.point(7.19295, 7.32488),
                );
                path.cubic_bezier_to(
                    view.point(8.75386, 6.80449),
                    view.point(7.61865, 7.112),
                    view.point(8.23355, 6.92276),
                );
                path.line_to(view.point(9.74716, 6.56795));
                path.line_to(view.point(10.4567, 6.70987));
                path.cubic_bezier_to(
                    view.point(11.5919, 6.92276),
                    view.point(10.8587, 6.78084),
                    view.point(11.3554, 6.87545),
                );
                path.cubic_bezier_to(
                    view.point(14.5245, 8.38933),
                    view.point(12.1595, 7.01738),
                    view.point(13.2947, 7.56143),
                );
                path.cubic_bezier_to(
                    view.point(17.599, 11.3698),
                    view.point(14.69, 8.48394),
                    view.point(16.0854, 9.83224),
                );
                path.cubic_bezier_to(
                    view.point(21.0046, 14.5394),
                    view.point(19.964, 13.7589),
                    view.point(20.437, 14.2083),
                );
                path.cubic_bezier_to(
                    view.point(21.9033, 15.1781),
                    view.point(21.3594, 14.7523),
                    view.point(21.7851, 15.0362),
                );
                path.cubic_bezier_to(
                    view.point(22.5182, 15.6985),
                    view.point(22.0452, 15.32),
                    view.point(22.329, 15.5566),
                );
                path.cubic_bezier_to(
                    view.point(22.9203, 18.1822),
                    view.point(23.6534, 16.4554),
                    view.point(23.6771, 16.692),
                );
                path.cubic_bezier_to(
                    view.point(22.3763, 19.7434),
                    view.point(22.4473, 19.0811),
                    view.point(22.4, 19.223),
                );
                path.cubic_bezier_to(
                    view.point(23.1804, 20.9734),
                    view.point(22.329, 20.5003),
                    view.point(22.4709, 20.7132),
                );
                path.cubic_bezier_to(
                    view.point(24.2447, 21.5884),
                    view.point(23.4642, 21.068),
                    view.point(23.9372, 21.3519),
                );
                path.cubic_bezier_to(
                    view.point(27.6503, 21.5884),
                    view.point(25.6873, 22.6765),
                    view.point(26.3968, 22.6529),
                );
                path.cubic_bezier_to(
                    view.point(28.3834, 21.1153),
                    view.point(27.8868, 21.3755),
                    view.point(28.2179, 21.1626),
                );
                path.cubic_bezier_to(
                    view.point(29.6133, 20.453),
                    view.point(28.8565, 20.9734),
                    view.point(29.4004, 20.6659),
                );
                path.cubic_bezier_to(
                    view.point(29.2112, 18.5134),
                    view.point(29.968, 20.0509),
                    view.point(29.9444, 19.9326),
                );
                path.cubic_bezier_to(
                    view.point(29.4714, 15.5802),
                    view.point(28.2652, 16.692),
                    view.point(28.2652, 16.7866),
                );
                path.cubic_bezier_to(
                    view.point(30.5829, 14.5394),
                    view.point(29.9917, 15.0598),
                    view.point(30.4883, 14.5868),
                );
                path.cubic_bezier_to(
                    view.point(32.0729, 14.7287),
                    view.point(30.914, 14.3739),
                    view.point(31.3397, 14.4212),
                );
                path.cubic_bezier_to(
                    view.point(32.9243, 13.9954),
                    view.point(33.3736, 15.2491),
                    view.point(33.6338, 15.0362),
                );
                path.cubic_bezier_to(
                    view.point(33.1371, 11.9138),
                    view.point(32.2148, 12.931),
                    view.point(32.2621, 12.5052),
                );
                path.cubic_bezier_to(
                    view.point(34.2014, 11.0859),
                    view.point(33.3973, 11.7482),
                    view.point(33.8703, 11.3698),
                );
                path.cubic_bezier_to(
                    view.point(37.9617, 8.05817),
                    view.point(35.3839, 10.0924),
                    view.point(37.4178, 8.43664),
                );
                path.cubic_bezier_to(
                    view.point(40.4214, 6.92276),
                    view.point(38.6476, 7.58508),
                    view.point(40.0903, 6.92276),
                );
                path.cubic_bezier_to(
                    view.point(41.1072, 6.80449),
                    view.point(40.5396, 6.92276),
                    view.point(40.8471, 6.8518),
                );
                path.cubic_bezier_to(
                    view.point(44.2527, 7.06469),
                    view.point(41.9113, 6.56795),
                    view.point(42.4789, 6.61526),
                );
                path.cubic_bezier_to(
                    view.point(47.0434, 8.72049),
                    view.point(45.246, 7.32488),
                    view.point(46.5704, 8.10548),
                );
                path.cubic_bezier_to(
                    view.point(47.7292, 9.57204),
                    view.point(47.1616, 8.88607),
                    view.point(47.4691, 9.26454),
                );
                path.cubic_bezier_to(
                    view.point(48.959, 12.0084),
                    view.point(48.4151, 10.4236),
                    view.point(48.5097, 10.6128),
                );
                path.cubic_bezier_to(
                    view.point(49.3611, 14.3029),
                    view.point(49.3611, 13.2621),
                    view.point(49.3611, 13.3094),
                );
                path.cubic_bezier_to(
                    view.point(48.959, 17.1414),
                    view.point(49.3611, 15.7222),
                    view.point(49.2428, 16.5974),
                );
                path.cubic_bezier_to(
                    view.point(48.6279, 17.993),
                    view.point(48.8171, 17.4016),
                    view.point(48.6752, 17.7801),
                );
                path.cubic_bezier_to(
                    view.point(48.2968, 18.8209),
                    view.point(48.5806, 18.1822),
                    view.point(48.4151, 18.5607),
                );
                path.cubic_bezier_to(
                    view.point(47.54, 21.7777),
                    view.point(47.611, 20.1928),
                    view.point(47.54, 20.4767),
                );
                path.cubic_bezier_to(
                    view.point(47.9894, 23.9775),
                    view.point(47.5637, 22.984),
                    view.point(47.611, 23.2442),
                );
                path.cubic_bezier_to(
                    view.point(49.1246, 24.9),
                    view.point(48.2732, 24.5216),
                    view.point(48.4151, 24.6398),
                );
                path.cubic_bezier_to(
                    view.point(50.8984, 24.7108),
                    view.point(49.8105, 25.1366),
                    view.point(49.9051, 25.1129),
                );
                path.cubic_bezier_to(
                    view.point(51.8207, 25.2312),
                    view.point(51.9626, 24.2614),
                    view.point(52.2228, 24.4269),
                );
                path.cubic_bezier_to(
                    view.point(51.0639, 26.1064),
                    view.point(51.5369, 25.7516),
                    view.point(51.3714, 25.9645),
                );
                path.cubic_bezier_to(
                    view.point(47.5164, 26.3666),
                    view.point(49.8578, 26.6977),
                    view.point(48.3678, 26.816),
                );
                path.cubic_bezier_to(
                    view.point(45.5298, 24.4506),
                    view.point(46.8069, 26.0118),
                    view.point(45.6953, 24.9473),
                );
                path.cubic_bezier_to(
                    view.point(45.1514, 23.5517),
                    view.point(45.4352, 24.2377),
                    view.point(45.2696, 23.8356),
                );
                path.cubic_bezier_to(
                    view.point(44.773, 21.825),
                    view.point(44.9622, 23.1496),
                    view.point(44.8912, 22.8421),
                );
                path.cubic_bezier_to(
                    view.point(44.6074, 20.4767),
                    view.point(44.7257, 21.139),
                    view.point(44.6311, 20.524),
                );
                path.cubic_bezier_to(
                    view.point(44.5128, 17.3306),
                    view.point(44.5838, 20.4057),
                    view.point(44.5365, 18.9864),
                );
                path.line_to(view.point(44.4419, 14.3029));
                path.line_to(view.point(44.1344, 13.7352));
                path.cubic_bezier_to(
                    view.point(43.0229, 12.6234),
                    view.point(43.7087, 13.0019),
                    view.point(43.4486, 12.7417),
                );
                path.cubic_bezier_to(
                    view.point(40.989, 14.1846),
                    view.point(42.1005, 12.3869),
                    view.point(41.1545, 13.1202),
                );
                path.cubic_bezier_to(
                    view.point(40.7288, 15.0598),
                    view.point(40.9417, 14.3975),
                    view.point(40.8471, 14.776),
                );
                path.cubic_bezier_to(
                    view.point(40.445, 16.2189),
                    view.point(40.6106, 15.32),
                    view.point(40.4923, 15.8404),
                );
                path.cubic_bezier_to(
                    view.point(40.3031, 17.3306),
                    view.point(40.3977, 16.5737),
                    view.point(40.3504, 17.0704),
                );
                path.cubic_bezier_to(
                    view.point(39.57, 18.4661),
                    view.point(40.2322, 17.9693),
                    view.point(40.1139, 18.1822),
                );
                path.cubic_bezier_to(
                    view.point(38.3874, 19.838),
                    view.point(38.955, 18.7972),
                    view.point(38.9077, 18.8445),
                );
                path.cubic_bezier_to(
                    view.point(37.6779, 20.9498),
                    view.point(38.1509, 20.3111),
                    view.point(37.8198, 20.8078),
                );
                path.cubic_bezier_to(
                    view.point(36.0697, 24.9946),
                    view.point(36.9684, 21.7067),
                    view.point(36.188, 23.67),
                );
                path.cubic_bezier_to(
                    view.point(34.3433, 25.6333),
                    view.point(35.9515, 26.201),
                    view.point(35.4548, 26.3902),
                );
                path.cubic_bezier_to(
                    view.point(33.0189, 24.9237),
                    view.point(34.0358, 25.4204),
                    view.point(33.4446, 25.1129),
                );
                path.cubic_bezier_to(
                    view.point(31.2451, 24.5925),
                    view.point(32.2621, 24.5925),
                    view.point(32.2148, 24.5925),
                );
                path.line_to(view.point(30.2755, 24.5925));
                path.line_to(view.point(29.1639, 25.1839));
                path.cubic_bezier_to(
                    view.point(27.7212, 27.0289),
                    view.point(27.7922, 25.9172),
                    view.point(27.7212, 26.0118),
                );
                path.cubic_bezier_to(
                    view.point(29.1403, 28.7557),
                    view.point(27.7212, 28.3299),
                    view.point(28.005, 28.6611),
                );
                path.cubic_bezier_to(
                    view.point(31.1269, 29.7018),
                    view.point(30.039, 28.803),
                    view.point(30.2991, 28.9449),
                );
                path.cubic_bezier_to(
                    view.point(32.5459, 30.2222),
                    view.point(31.8127, 30.3169),
                    view.point(32.0492, 30.4115),
                );
                path.cubic_bezier_to(
                    view.point(34.8872, 28.046),
                    view.point(32.8297, 30.104),
                    view.point(34.0831, 28.9449),
                );
                path.cubic_bezier_to(
                    view.point(35.715, 27.3128),
                    view.point(35.3366, 27.5256),
                    view.point(35.5731, 27.3128),
                );
                path.cubic_bezier_to(
                    view.point(36.2826, 28.0224),
                    view.point(35.9278, 27.3128),
                    view.point(36.2116, 27.6676),
                );
                path.cubic_bezier_to(
                    view.point(36.4954, 29.3943),
                    view.point(36.2826, 28.1643),
                    view.point(36.4008, 28.7793),
                );
                path.cubic_bezier_to(
                    view.point(37.3705, 32.0436),
                    view.point(36.7319, 30.9319),
                    view.point(36.9211, 31.5232),
                );
                path.cubic_bezier_to(
                    view.point(37.0394, 33.794),
                    view.point(37.9144, 32.6586),
                    view.point(37.8198, 33.1317),
                );
                path.cubic_bezier_to(
                    view.point(35.4075, 33.794),
                    view.point(36.3772, 34.3617),
                    view.point(36.2589, 34.3617),
                );
                path.cubic_bezier_to(
                    view.point(34.2014, 32.8479),
                    view.point(35.0291, 33.5338),
                    view.point(34.4852, 33.1081),
                );
                path.cubic_bezier_to(
                    view.point(32.0729, 32.0436),
                    view.point(33.4209, 32.1146),
                    view.point(33.3027, 32.0909),
                );
                path.cubic_bezier_to(
                    view.point(30.441, 33.0134),
                    view.point(30.4647, 31.9963),
                    view.point(30.0626, 32.2328),
                );
                path.cubic_bezier_to(
                    view.point(29.9444, 34.0779),
                    view.point(30.7721, 33.6285),
                    view.point(30.6539, 33.865),
                );
                path.cubic_bezier_to(
                    view.point(29.1403, 34.9531),
                    view.point(29.566, 34.2198),
                    view.point(29.3531, 34.4564),
                );
                path.cubic_bezier_to(
                    view.point(27.3665, 37.5078),
                    view.point(28.5963, 36.3723),
                    view.point(27.9814, 37.2712),
                );
                path.cubic_bezier_to(
                    view.point(24.8832, 37.626),
                    view.point(27.0117, 37.6733),
                    view.point(25.1907, 37.7443),
                );
                path.close();
                path.move_to(view.point(25.0724, 32.7532));
                path.cubic_bezier_to(
                    view.point(25.4272, 32.1382),
                    view.point(25.167, 32.6586),
                    view.point(25.3089, 32.3748),
                );
                path.cubic_bezier_to(
                    view.point(26.586, 32.1619),
                    view.point(25.7583, 31.3103),
                    view.point(26.3495, 31.334),
                );
                path.cubic_bezier_to(
                    view.point(27.2955, 32.9425),
                    view.point(26.6806, 32.5167),
                    view.point(27.059, 32.9425),
                );
                path.cubic_bezier_to(
                    view.point(28.1469, 31.6415),
                    view.point(27.8868, 32.9425),
                    view.point(28.1469, 32.5404),
                );
                path.cubic_bezier_to(
                    view.point(27.5084, 30.2459),
                    view.point(28.1469, 30.9082),
                    view.point(28.0287, 30.6244),
                );
                path.cubic_bezier_to(
                    view.point(26.7989, 29.6545),
                    view.point(27.3428, 30.104),
                    view.point(27.0354, 29.8438),
                );
                path.cubic_bezier_to(
                    view.point(25.0961, 29.7491),
                    view.point(26.1367, 29.0632),
                    view.point(25.8056, 29.0868),
                );
                path.cubic_bezier_to(
                    view.point(23.8426, 31.6651),
                    view.point(24.0791, 30.6953),
                    view.point(23.8426, 31.0738),
                );
                path.cubic_bezier_to(
                    view.point(24.221, 32.8479),
                    view.point(23.8426, 32.2565),
                    view.point(23.9845, 32.7059),
                );
                path.cubic_bezier_to(
                    view.point(25.0724, 32.7532),
                    view.point(24.5048, 32.9898),
                    view.point(24.8832, 32.9425),
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
