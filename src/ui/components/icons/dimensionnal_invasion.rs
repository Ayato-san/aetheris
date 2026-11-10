use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct DimensionnalInvasionIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl DimensionnalInvasionIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for DimensionnalInvasionIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(26.3209, 51.8581));
                path.cubic_bezier_to(
                    view.point(25.493, 50.2021),
                    view.point(26.179, 51.7161),
                    view.point(25.6822, 50.6989),
                );
                path.cubic_bezier_to(
                    view.point(25.1382, 49.3505),
                    view.point(25.4457, 50.0365),
                    view.point(25.2801, 49.658),
                );
                path.cubic_bezier_to(
                    view.point(24.5941, 48.1203),
                    view.point(24.9963, 49.0193),
                    view.point(24.7597, 48.4752),
                );
                path.cubic_bezier_to(
                    view.point(23.6953, 46.1332),
                    view.point(24.4522, 47.7892),
                    view.point(24.0501, 46.8902),
                );
                path.line_to(view.point(23.0803, 44.8084));
                path.line_to(view.point(22.3943, 44.0751));
                path.cubic_bezier_to(
                    view.point(18.9171, 43.0342),
                    view.point(21.3535, 42.9869),
                    view.point(21.4954, 43.0342),
                );
                path.cubic_bezier_to(
                    view.point(16.1732, 44.0041),
                    view.point(16.3151, 43.0342),
                    view.point(16.3624, 43.0342),
                );
                path.cubic_bezier_to(
                    view.point(15.7474, 44.4299),
                    view.point(16.0786, 44.5009),
                    view.point(16.0076, 44.5719),
                );
                path.cubic_bezier_to(
                    view.point(14.8249, 42.5611),
                    view.point(15.5345, 44.3353),
                    view.point(15.5345, 44.3353),
                );
                path.cubic_bezier_to(
                    view.point(13.6185, 40.7632),
                    view.point(14.1862, 41.0234),
                    view.point(14.2099, 41.0234),
                );
                path.cubic_bezier_to(
                    view.point(11.7972, 39.6277),
                    view.point(12.6487, 40.3374),
                    view.point(12.3649, 40.1718),
                );
                path.cubic_bezier_to(
                    view.point(10.9693, 38.161),
                    view.point(11.0875, 38.9653),
                    view.point(10.851, 38.5631),
                );
                path.cubic_bezier_to(
                    view.point(11.9391, 35.748),
                    view.point(11.0639, 37.8061),
                    view.point(11.537, 36.647),
                );
                path.cubic_bezier_to(
                    view.point(12.4831, 34.4942),
                    view.point(12.1283, 35.3222),
                    view.point(12.3649, 34.7781),
                );
                path.cubic_bezier_to(
                    view.point(13.4766, 33.6663),
                    view.point(12.8616, 33.5953),
                    view.point(13.311, 33.2168),
                );
                path.cubic_bezier_to(
                    view.point(13.5949, 35.6061),
                    view.point(13.5239, 33.7609),
                    view.point(13.5712, 34.6362),
                );
                path.cubic_bezier_to(
                    view.point(14.0207, 38.019),
                    view.point(13.6185, 37.5459),
                    view.point(13.6658, 37.6879),
                );
                path.cubic_bezier_to(
                    view.point(14.8959, 37.3803),
                    view.point(14.4937, 38.4449),
                    view.point(14.7539, 38.2556),
                );
                path.cubic_bezier_to(
                    view.point(16.7172, 33.9501),
                    view.point(15.0851, 36.0082),
                    view.point(15.2507, 35.7007),
                );
                path.cubic_bezier_to(
                    view.point(17.8053, 32.6017),
                    view.point(17.143, 33.4297),
                    view.point(17.6161, 32.8146),
                );
                path.cubic_bezier_to(
                    view.point(18.7279, 32.3888),
                    view.point(18.2075, 32.034),
                    view.point(18.5623, 31.963),
                );
                path.cubic_bezier_to(
                    view.point(18.7752, 33.8319),
                    view.point(18.7752, 32.5071),
                    view.point(18.7988, 33.0748),
                );
                path.line_to(view.point(18.7515, 35.062));
                path.line_to(view.point(18.3967, 35.819));
                path.cubic_bezier_to(
                    view.point(18.0182, 37.3094),
                    view.point(18.0655, 36.5287),
                    view.point(18.0419, 36.576),
                );
                path.cubic_bezier_to(
                    view.point(18.1838, 38.2793),
                    view.point(17.9946, 38.019),
                    view.point(17.9946, 38.0664),
                );
                path.cubic_bezier_to(
                    view.point(20.0762, 39.0363),
                    view.point(18.4204, 38.5395),
                    view.point(19.2009, 38.8707),
                );
                path.cubic_bezier_to(
                    view.point(21.1169, 39.3438),
                    view.point(20.4073, 39.1072),
                    view.point(20.8804, 39.2492),
                );
                path.cubic_bezier_to(
                    view.point(22.2287, 38.918),
                    view.point(21.8266, 39.6277),
                    view.point(21.9922, 39.5567),
                );
                path.cubic_bezier_to(
                    view.point(23.2695, 37.9717),
                    view.point(22.3706, 38.5158),
                    view.point(22.6072, 38.3266),
                );
                path.cubic_bezier_to(
                    view.point(25.1855, 35.6297),
                    view.point(23.9318, 37.6169),
                    view.point(24.1447, 37.3803),
                );
                path.cubic_bezier_to(
                    view.point(26.6757, 33.6899),
                    view.point(26.0843, 34.0921),
                    view.point(26.4155, 33.6899),
                );
                path.cubic_bezier_to(
                    view.point(28.024, 35.3932),
                    view.point(26.9122, 33.6899),
                    view.point(27.1961, 34.0211),
                );
                path.cubic_bezier_to(
                    view.point(29.4906, 37.5932),
                    view.point(29.1831, 37.2857),
                    view.point(29.2304, 37.3567),
                );
                path.cubic_bezier_to(
                    view.point(30.1529, 38.019),
                    view.point(29.6325, 37.7115),
                    view.point(29.9163, 37.9008),
                );
                path.cubic_bezier_to(
                    view.point(31.1227, 38.918),
                    view.point(30.7206, 38.3029),
                    view.point(30.9808, 38.5395),
                );
                path.cubic_bezier_to(
                    view.point(32.4947, 39.2728),
                    view.point(31.3593, 39.5804),
                    view.point(31.4775, 39.604),
                );
                path.cubic_bezier_to(
                    view.point(33.6774, 38.9416),
                    view.point(32.7785, 39.1782),
                    view.point(33.3226, 39.0363),
                );
                path.cubic_bezier_to(
                    view.point(35.3095, 38.1137),
                    view.point(34.6235, 38.7051),
                    view.point(35.1676, 38.4449),
                );
                path.cubic_bezier_to(
                    view.point(34.931, 35.8427),
                    view.point(35.5224, 37.6405),
                    view.point(35.3568, 36.6943),
                );
                path.line_to(view.point(34.5999, 35.133));
                path.line_to(view.point(34.5762, 33.9265));
                path.cubic_bezier_to(
                    view.point(34.5999, 32.4834),
                    view.point(34.5526, 33.2641),
                    view.point(34.5762, 32.6017),
                );
                path.cubic_bezier_to(
                    view.point(35.5461, 32.6017),
                    view.point(34.7418, 31.963),
                    view.point(35.1203, 32.0103),
                );
                path.cubic_bezier_to(
                    view.point(36.6342, 33.9501),
                    view.point(35.7353, 32.8146),
                    view.point(36.2084, 33.4297),
                );
                path.cubic_bezier_to(
                    view.point(38.4555, 37.333),
                    view.point(38.1244, 35.7244),
                    view.point(38.2426, 35.9846),
                );
                path.cubic_bezier_to(
                    view.point(39.3307, 38.019),
                    view.point(38.5975, 38.2556),
                    view.point(38.834, 38.4449),
                );
                path.cubic_bezier_to(
                    view.point(39.7565, 35.6061),
                    view.point(39.6856, 37.6879),
                    view.point(39.7329, 37.5459),
                );
                path.cubic_bezier_to(
                    view.point(39.8748, 33.6663),
                    view.point(39.7802, 34.6362),
                    view.point(39.8275, 33.7609),
                );
                path.cubic_bezier_to(
                    view.point(40.9392, 34.6125),
                    view.point(40.0404, 33.1931),
                    view.point(40.4662, 33.5716),
                );
                path.cubic_bezier_to(
                    view.point(42.1456, 37.4513),
                    view.point(41.5069, 35.9373),
                    view.point(41.8144, 36.647),
                );
                path.line_to(view.point(42.4531, 38.2793));
                path.line_to(view.point(42.3348, 38.6341));
                path.cubic_bezier_to(
                    view.point(39.8038, 40.7395),
                    view.point(42.051, 39.3438),
                    view.point(40.9629, 40.2664),
                );
                path.cubic_bezier_to(
                    view.point(39.2125, 41.0707),
                    view.point(39.5436, 40.8578),
                    view.point(39.2598, 40.9998),
                );
                path.cubic_bezier_to(
                    view.point(38.148, 43.5073),
                    view.point(39.0705, 41.2127),
                    view.point(38.5028, 42.5138),
                );
                path.cubic_bezier_to(
                    view.point(37.4148, 44.4772),
                    view.point(37.8878, 44.1934),
                    view.point(37.6749, 44.4772),
                );
                path.cubic_bezier_to(
                    view.point(37.2255, 44.217),
                    view.point(37.2965, 44.4772),
                    view.point(37.2492, 44.4299),
                );
                path.cubic_bezier_to(
                    view.point(33.9376, 42.9869),
                    view.point(37.0126, 42.9869),
                    view.point(36.918, 42.9632),
                );
                path.line_to(view.point(32.1635, 43.0106));
                path.line_to(view.point(31.8796, 43.1998));
                path.cubic_bezier_to(
                    view.point(30.1292, 45.1396),
                    view.point(31.241, 43.6493),
                    view.point(30.3658, 44.5955),
                );
                path.cubic_bezier_to(
                    view.point(29.4196, 46.6773),
                    view.point(29.9873, 45.4235),
                    view.point(29.6798, 46.1332),
                );
                path.cubic_bezier_to(
                    view.point(27.8584, 50.2021),
                    view.point(28.5444, 48.5225),
                    view.point(28.024, 49.7526),
                );
                path.cubic_bezier_to(
                    view.point(27.0305, 51.8581),
                    view.point(27.6692, 50.7462),
                    view.point(27.1724, 51.7161),
                );
                path.cubic_bezier_to(
                    view.point(26.3209, 51.8581),
                    view.point(26.8413, 52.0473),
                    view.point(26.5101, 52.0473),
                );
                path.close();
                path.move_to(view.point(26.4392, 28.9113));
                path.cubic_bezier_to(
                    view.point(26.1317, 28.2253),
                    view.point(26.2972, 28.793),
                    view.point(26.2263, 28.6274),
                );
                path.cubic_bezier_to(
                    view.point(25.7768, 26.9005),
                    view.point(26.0843, 27.9177),
                    view.point(25.9188, 27.3263),
                );
                path.cubic_bezier_to(
                    view.point(25.2801, 25.4102),
                    view.point(25.6349, 26.4747),
                    view.point(25.422, 25.7887),
                );
                path.cubic_bezier_to(
                    view.point(24.9489, 24.1327),
                    view.point(25.1618, 25.008),
                    view.point(24.9963, 24.4402),
                );
                path.cubic_bezier_to(
                    view.point(24.6651, 23.1628),
                    view.point(24.878, 23.8015),
                    view.point(24.7597, 23.3757),
                );
                path.cubic_bezier_to(
                    view.point(24.3812, 22.2402),
                    view.point(24.5705, 22.9499),
                    view.point(24.4522, 22.5241),
                );
                path.cubic_bezier_to(
                    view.point(24.121, 21.3413),
                    view.point(24.3103, 21.9327),
                    view.point(24.192, 21.5305),
                );
                path.cubic_bezier_to(
                    view.point(23.8372, 20.5133),
                    view.point(24.0264, 21.152),
                    view.point(23.9082, 20.7735),
                );
                path.cubic_bezier_to(
                    view.point(23.0093, 17.6272),
                    view.point(23.6006, 19.5434),
                    view.point(23.1985, 18.1476),
                );
                path.cubic_bezier_to(
                    view.point(22.7018, 16.5153),
                    view.point(22.9147, 17.3433),
                    view.point(22.7727, 16.8465),
                );
                path.cubic_bezier_to(
                    view.point(22.205, 15.4745),
                    view.point(22.5362, 15.8056),
                    view.point(22.3943, 15.5218),
                );
                path.cubic_bezier_to(
                    view.point(21.8739, 17.3433),
                    view.point(21.6373, 15.2852),
                    view.point(21.4245, 16.3971),
                );
                path.cubic_bezier_to(
                    view.point(22.205, 19.3305),
                    view.point(22.2524, 18.124),
                    view.point(22.2997, 18.2896),
                );
                path.cubic_bezier_to(
                    view.point(21.8029, 22.6424),
                    view.point(22.0631, 21.1283),
                    view.point(21.9922, 21.7434),
                );
                path.cubic_bezier_to(
                    view.point(21.5427, 25.4338),
                    view.point(21.661, 23.3521),
                    view.point(21.6137, 23.9908),
                );
                path.cubic_bezier_to(
                    view.point(21.046, 27.4683),
                    view.point(21.4481, 27.4446),
                    view.point(21.4245, 27.5629),
                );
                path.cubic_bezier_to(
                    view.point(19.7923, 26.3328),
                    view.point(20.8094, 27.421),
                    view.point(20.2417, 26.9005),
                );
                path.cubic_bezier_to(
                    view.point(18.6806, 23.8015),
                    view.point(19.4375, 25.8833),
                    view.point(18.8461, 24.5349),
                );
                path.cubic_bezier_to(
                    view.point(18.444, 23.1628),
                    view.point(18.6569, 23.6832),
                    view.point(18.5386, 23.3757),
                );
                path.cubic_bezier_to(
                    view.point(18.0419, 22.0983),
                    view.point(18.3257, 22.9499),
                    view.point(18.1602, 22.4768),
                );
                path.cubic_bezier_to(
                    view.point(16.2205, 21.223),
                    view.point(17.6161, 20.5369),
                    view.point(17.3323, 20.395),
                );
                path.cubic_bezier_to(
                    view.point(15.3216, 22.5004),
                    view.point(15.5582, 21.7198),
                    view.point(15.3216, 22.0509),
                );
                path.cubic_bezier_to(
                    view.point(14.4701, 22.8553),
                    view.point(15.3216, 23.0209),
                    view.point(14.8959, 23.1865),
                );
                path.cubic_bezier_to(
                    view.point(13.9734, 23.7542),
                    view.point(13.8787, 22.4294),
                    view.point(13.6895, 22.7606),
                );
                path.cubic_bezier_to(
                    view.point(14.1862, 24.8661),
                    view.point(14.0443, 24.0381),
                    view.point(14.1389, 24.5349),
                );
                path.cubic_bezier_to(
                    view.point(15.2507, 27.4919),
                    view.point(14.3045, 25.765),
                    view.point(14.5647, 26.4037),
                );
                path.cubic_bezier_to(
                    view.point(14.0443, 27.7995),
                    view.point(15.8657, 28.5092),
                    view.point(15.3216, 28.6511),
                );
                path.cubic_bezier_to(
                    view.point(11.2531, 25.3628),
                    view.point(11.7498, 26.3328),
                    view.point(11.537, 26.1435),
                );
                path.cubic_bezier_to(
                    view.point(10.3542, 23.2811),
                    view.point(10.9929, 24.6768),
                    view.point(10.5198, 23.5886),
                );
                path.cubic_bezier_to(
                    view.point(10.1414, 22.7133),
                    view.point(10.2833, 23.1155),
                    view.point(10.1887, 22.8789),
                );
                path.cubic_bezier_to(
                    view.point(9.55, 21.3413),
                    view.point(10.094, 22.5477),
                    view.point(9.83385, 21.9327),
                );
                path.line_to(view.point(9.05326, 20.2767));
                path.line_to(view.point(9.02961, 19.4014));
                path.cubic_bezier_to(
                    view.point(9.45538, 16.9885),
                    view.point(8.93499, 17.7691),
                    view.point(9.07691, 16.9885),
                );
                path.cubic_bezier_to(
                    view.point(10.094, 17.4853),
                    view.point(9.52635, 16.9885),
                    view.point(9.8102, 17.225),
                );
                path.cubic_bezier_to(
                    view.point(11.9154, 18.1949),
                    view.point(10.7564, 18.124),
                    view.point(11.0639, 18.2423),
                );
                path.cubic_bezier_to(
                    view.point(13.9024, 17.4379),
                    view.point(12.6724, 18.1476),
                    view.point(12.7197, 18.124),
                );
                path.cubic_bezier_to(
                    view.point(15.9603, 15.9949),
                    view.point(15.2034, 16.6573),
                    view.point(15.7947, 16.2315),
                );
                path.cubic_bezier_to(
                    view.point(16.4571, 13.4873),
                    view.point(16.1022, 15.7347),
                    view.point(16.3624, 14.5282),
                );
                path.cubic_bezier_to(
                    view.point(16.6463, 12.2099),
                    view.point(16.5044, 13.0615),
                    view.point(16.5753, 12.4701),
                );
                path.cubic_bezier_to(
                    view.point(17.4742, 11.7604),
                    view.point(16.8355, 11.3819),
                    view.point(17.214, 11.169),
                );
                path.cubic_bezier_to(
                    view.point(17.758, 12.3991),
                    view.point(17.5451, 11.8787),
                    view.point(17.6634, 12.1626),
                );
                path.cubic_bezier_to(
                    view.point(20.6912, 13.8185),
                    view.point(18.1365, 13.2508),
                    view.point(19.2956, 13.8185),
                );
                path.cubic_bezier_to(
                    view.point(21.8266, 12.2808),
                    view.point(22.0158, 13.8422),
                    view.point(22.1104, 13.7239),
                );
                path.cubic_bezier_to(
                    view.point(21.5191, 8.73237),
                    view.point(21.6137, 11.3346),
                    view.point(21.4245, 9.08722),
                );
                path.cubic_bezier_to(
                    view.point(23.1512, 7.21836),
                    view.point(21.59, 8.42484),
                    view.point(21.8502, 8.18827),
                );
                path.cubic_bezier_to(
                    view.point(25.0199, 5.16025),
                    view.point(24.7361, 6.01188),
                    view.point(24.8543, 5.8936),
                );
                path.cubic_bezier_to(
                    view.point(25.3038, 4.19033),
                    view.point(25.0909, 4.85271),
                    view.point(25.2091, 4.40324),
                );
                path.cubic_bezier_to(
                    view.point(25.5403, 3.19676),
                    view.point(25.3747, 3.95377),
                    view.point(25.493, 3.5043),
                );
                path.cubic_bezier_to(
                    view.point(25.8242, 2.06125),
                    view.point(25.5876, 2.88923),
                    view.point(25.7059, 2.39244),
                );
                path.cubic_bezier_to(
                    view.point(26.108, 0.973058),
                    view.point(25.9188, 1.73006),
                    view.point(26.0607, 1.23328),
                );
                path.cubic_bezier_to(
                    view.point(27.2434, 0.925745),
                    view.point(26.3209, -0.30439),
                    view.point(27.0305, -0.328046),
                );
                path.cubic_bezier_to(
                    view.point(27.5273, 2.06125),
                    view.point(27.2907, 1.18597),
                    view.point(27.409, 1.70641),
                );
                path.cubic_bezier_to(
                    view.point(27.8584, 3.31505),
                    view.point(27.6455, 2.39244),
                    view.point(27.7875, 2.9602),
                );
                path.cubic_bezier_to(
                    view.point(28.095, 4.33227),
                    view.point(27.9057, 3.64624),
                    view.point(28.024, 4.11936),
                );
                path.cubic_bezier_to(
                    view.point(28.3552, 5.23122),
                    view.point(28.1896, 4.56884),
                    view.point(28.2842, 4.971),
                );
                path.cubic_bezier_to(
                    view.point(30.1056, 7.12373),
                    view.point(28.4971, 5.86994),
                    view.point(28.6627, 6.03554),
                );
                path.cubic_bezier_to(
                    view.point(31.7614, 8.56677),
                    view.point(31.4302, 8.14096),
                    view.point(31.6194, 8.30655),
                );
                path.cubic_bezier_to(
                    view.point(31.4066, 13.1325),
                    view.point(31.9506, 8.94528),
                    view.point(31.785, 11.0507),
                );
                path.cubic_bezier_to(
                    view.point(32.8258, 13.8185),
                    view.point(31.2883, 13.7002),
                    view.point(31.6904, 13.8895),
                );
                path.cubic_bezier_to(
                    view.point(35.7353, 12.0916),
                    view.point(34.316, 13.7475),
                    view.point(35.4041, 13.1088),
                );
                path.cubic_bezier_to(
                    view.point(36.3267, 11.4529),
                    view.point(35.9009, 11.5948),
                    view.point(36.1374, 11.3582),
                );
                path.cubic_bezier_to(
                    view.point(37.0126, 14.268),
                    view.point(36.6105, 11.6185),
                    view.point(36.7051, 12.0206),
                );
                path.cubic_bezier_to(
                    view.point(37.4148, 16.0186),
                    view.point(37.1309, 15.2379),
                    view.point(37.2492, 15.782),
                );
                path.cubic_bezier_to(
                    view.point(39.449, 17.4379),
                    view.point(37.5567, 16.2315),
                    view.point(38.219, 16.6809),
                );
                path.cubic_bezier_to(
                    view.point(41.5779, 18.1713),
                    view.point(40.6554, 18.1476),
                    view.point(40.7027, 18.1713),
                );
                path.cubic_bezier_to(
                    view.point(43.3047, 17.4379),
                    view.point(42.4768, 18.1713),
                    view.point(42.6187, 18.1003),
                );
                path.cubic_bezier_to(
                    view.point(44.1326, 17.1304),
                    view.point(43.8014, 16.9648),
                    view.point(43.9197, 16.9175),
                );
                path.cubic_bezier_to(
                    view.point(44.3218, 18.2186),
                    view.point(44.2508, 17.2487),
                    view.point(44.2745, 17.4143),
                );
                path.cubic_bezier_to(
                    view.point(43.8487, 21.2466),
                    view.point(44.3928, 19.7799),
                    view.point(44.3455, 20.1584),
                );
                path.cubic_bezier_to(
                    view.point(42.6897, 24.0144),
                    view.point(43.4466, 22.0746),
                    view.point(43.1154, 22.8553),
                );
                path.cubic_bezier_to(
                    view.point(42.3585, 24.7478),
                    view.point(42.595, 24.251),
                    view.point(42.4295, 24.5822),
                );
                path.cubic_bezier_to(
                    view.point(42.0746, 25.4338),
                    view.point(42.2639, 24.9134),
                    view.point(42.1456, 25.2209),
                );
                path.cubic_bezier_to(
                    view.point(39.3071, 27.7995),
                    view.point(41.8381, 26.1199),
                    view.point(41.5543, 26.3564),
                );
                path.cubic_bezier_to(
                    view.point(38.1007, 27.4919),
                    view.point(38.0061, 28.6511),
                    view.point(37.4857, 28.5092),
                );
                path.cubic_bezier_to(
                    view.point(39.1652, 24.8661),
                    view.point(38.7867, 26.4037),
                    view.point(39.0469, 25.765),
                );
                path.cubic_bezier_to(
                    view.point(39.3781, 23.7542),
                    view.point(39.2125, 24.5349),
                    view.point(39.3071, 24.0381),
                );
                path.cubic_bezier_to(
                    view.point(38.8577, 22.8553),
                    view.point(39.6619, 22.737),
                    view.point(39.4963, 22.4294),
                );
                path.cubic_bezier_to(
                    view.point(38.0061, 22.3112),
                    view.point(38.3609, 23.1865),
                    view.point(38.0771, 23.0209),
                );
                path.cubic_bezier_to(
                    view.point(37.7459, 21.7198),
                    view.point(37.9588, 22.0746),
                    view.point(37.8878, 21.8853),
                );
                path.cubic_bezier_to(
                    view.point(36.2793, 20.7025),
                    view.point(37.5094, 21.4595),
                    view.point(36.5159, 20.7735),
                );
                path.cubic_bezier_to(
                    view.point(35.2622, 22.1692),
                    view.point(35.9009, 20.6079),
                    view.point(35.5224, 21.152),
                );
                path.cubic_bezier_to(
                    view.point(34.931, 23.1391),
                    view.point(35.1912, 22.5004),
                    view.point(35.0257, 22.9262),
                );
                path.cubic_bezier_to(
                    view.point(34.6235, 23.9435),
                    view.point(34.8364, 23.3284),
                    view.point(34.6945, 23.7069),
                );
                path.cubic_bezier_to(
                    view.point(32.7312, 27.208),
                    view.point(34.1978, 25.4811),
                    view.point(33.6301, 26.4747),
                );
                path.cubic_bezier_to(
                    view.point(31.8323, 25.7887),
                    view.point(31.9979, 27.8231),
                    view.point(31.9033, 27.6575),
                );
                path.cubic_bezier_to(
                    view.point(31.5012, 22.4058),
                    view.point(31.785, 24.1564),
                    view.point(31.7141, 23.423),
                );
                path.cubic_bezier_to(
                    view.point(31.1937, 19.0939),
                    view.point(31.2883, 21.2939),
                    view.point(31.1937, 20.395),
                );
                path.line_to(view.point(31.1937, 17.8874));
                path.line_to(view.point(31.4539, 17.367));
                path.cubic_bezier_to(
                    view.point(31.7141, 16.4207),
                    view.point(31.6668, 16.9412),
                    view.point(31.7141, 16.7992),
                );
                path.cubic_bezier_to(
                    view.point(30.7442, 16.1841),
                    view.point(31.7141, 15.2615),
                    view.point(30.9335, 15.0723),
                );
                path.cubic_bezier_to(
                    view.point(30.4367, 17.3197),
                    view.point(30.7206, 16.3261),
                    view.point(30.5787, 16.8465),
                );
                path.cubic_bezier_to(
                    view.point(29.5615, 20.324),
                    view.point(30.1765, 18.124),
                    view.point(29.8927, 19.0466),
                );
                path.cubic_bezier_to(
                    view.point(29.2304, 21.3176),
                    view.point(29.4669, 20.6552),
                    view.point(29.325, 21.1047),
                );
                path.cubic_bezier_to(
                    view.point(28.9702, 22.1929),
                    view.point(29.1594, 21.5305),
                    view.point(29.0411, 21.9327),
                );
                path.cubic_bezier_to(
                    view.point(28.6863, 23.1155),
                    view.point(28.9229, 22.4531),
                    view.point(28.8046, 22.8789),
                );
                path.cubic_bezier_to(
                    view.point(28.3788, 24.2273),
                    view.point(28.5917, 23.3521),
                    view.point(28.4498, 23.8725),
                );
                path.cubic_bezier_to(
                    view.point(28.024, 25.5758),
                    view.point(28.3078, 24.6058),
                    view.point(28.1423, 25.2209),
                );
                path.cubic_bezier_to(
                    view.point(27.5746, 26.9715),
                    view.point(27.8821, 25.9543),
                    view.point(27.6928, 26.593),
                );
                path.cubic_bezier_to(
                    view.point(27.2198, 28.2253),
                    view.point(27.4326, 27.3736),
                    view.point(27.2671, 27.9177),
                );
                path.cubic_bezier_to(
                    view.point(26.4392, 28.9113),
                    view.point(27.0542, 28.9823),
                    view.point(26.794, 29.2188),
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
