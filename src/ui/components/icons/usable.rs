use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct UsableIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl UsableIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for UsableIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(23.3333, 51.9508));
                path.cubic_bezier_to(
                    view.point(17.1832, 50.7445),
                    view.point(21.8431, 51.8089),
                    view.point(17.8691, 51.052),
                );
                path.cubic_bezier_to(
                    view.point(16.0005, 50.3424),
                    view.point(16.7574, 50.5789),
                    view.point(16.237, 50.3897),
                );
                path.cubic_bezier_to(
                    view.point(15.0306, 49.9166),
                    view.point(15.7639, 50.2951),
                    view.point(15.3145, 50.1059),
                );
                path.cubic_bezier_to(
                    view.point(14.0135, 49.4199),
                    view.point(14.7231, 49.7274),
                    view.point(14.2737, 49.4909),
                );
                path.cubic_bezier_to(
                    view.point(13.3039, 48.9942),
                    view.point(13.7533, 49.3253),
                    view.point(13.4458, 49.1361),
                );
                path.cubic_bezier_to(
                    view.point(12.6415, 48.5211),
                    view.point(13.1383, 48.8759),
                    view.point(12.8544, 48.6394),
                );
                path.cubic_bezier_to(
                    view.point(9.68476, 45.6828),
                    view.point(11.6954, 47.9535),
                    view.point(9.68476, 46.0139),
                );
                path.cubic_bezier_to(
                    view.point(9.3536, 45.1861),
                    view.point(9.68476, 45.6355),
                    view.point(9.54284, 45.399),
                );
                path.cubic_bezier_to(
                    view.point(8.7859, 44.2636),
                    view.point(9.16437, 44.9732),
                    view.point(8.90417, 44.5475),
                );
                path.cubic_bezier_to(
                    view.point(8.31281, 43.3885),
                    view.point(8.64397, 43.9561),
                    view.point(8.43108, 43.5777),
                );
                path.cubic_bezier_to(
                    view.point(7.86338, 42.1822),
                    view.point(8.19454, 43.2229),
                    view.point(7.98165, 42.6789),
                );
                path.cubic_bezier_to(
                    view.point(7.3903, 40.3846),
                    view.point(7.72146, 41.6618),
                    view.point(7.50857, 40.8576),
                );
                path.cubic_bezier_to(
                    view.point(7.3903, 34.2822),
                    view.point(6.8699, 38.4687),
                    view.point(6.8699, 34.2822),
                );
                path.cubic_bezier_to(
                    view.point(7.74511, 36.2453),
                    view.point(7.55588, 34.2822),
                    view.point(7.60318, 34.5896),
                );
                path.cubic_bezier_to(
                    view.point(8.02896, 38.4924),
                    view.point(7.81607, 37.0968),
                    view.point(7.93434, 38.1139),
                );
                path.cubic_bezier_to(
                    view.point(8.97513, 41.4963),
                    view.point(8.45474, 40.1954),
                    view.point(8.64397, 40.834),
                );
                path.cubic_bezier_to(
                    view.point(9.40091, 42.466),
                    view.point(9.16437, 41.8747),
                    view.point(9.3536, 42.3004),
                );
                path.cubic_bezier_to(
                    view.point(9.80303, 43.152),
                    view.point(9.44822, 42.6316),
                    view.point(9.63745, 42.9391),
                );
                path.cubic_bezier_to(
                    view.point(10.2761, 43.8379),
                    view.point(9.96861, 43.3885),
                    view.point(10.1815, 43.696),
                );
                path.cubic_bezier_to(
                    view.point(14.5812, 48.0008),
                    view.point(10.9148, 45.0678),
                    view.point(12.9964, 47.0547),
                );
                path.cubic_bezier_to(
                    view.point(15.3145, 48.4738),
                    view.point(14.8651, 48.1427),
                    view.point(15.1726, 48.3792),
                );
                path.cubic_bezier_to(
                    view.point(16.1897, 48.8996),
                    view.point(15.4328, 48.5921),
                    view.point(15.8349, 48.7813),
                );
                path.cubic_bezier_to(
                    view.point(17.2305, 49.3726),
                    view.point(16.5682, 49.0415),
                    view.point(17.0176, 49.2307),
                );
                path.cubic_bezier_to(
                    view.point(24.7289, 50.1532),
                    view.point(18.3895, 50.0822),
                    view.point(23.1204, 50.5789),
                );
                path.cubic_bezier_to(
                    view.point(26.3374, 49.7747),
                    view.point(25.1074, 50.0349),
                    view.point(25.8406, 49.8693),
                );
                path.cubic_bezier_to(
                    view.point(27.9459, 49.2544),
                    view.point(26.8341, 49.6801),
                    view.point(27.5674, 49.4436),
                );
                path.cubic_bezier_to(
                    view.point(29.1522, 48.6867),
                    view.point(28.3243, 49.0651),
                    view.point(28.8684, 48.805),
                );
                path.cubic_bezier_to(
                    view.point(35.6335, 42.1112),
                    view.point(31.494, 47.6223),
                    view.point(34.782, 44.2873),
                );
                path.cubic_bezier_to(
                    view.point(36.0593, 41.2597),
                    view.point(35.7518, 41.8274),
                    view.point(35.941, 41.449),
                );
                path.cubic_bezier_to(
                    view.point(36.4141, 40.3373),
                    view.point(36.1776, 41.0705),
                    view.point(36.3431, 40.6448),
                );
                path.cubic_bezier_to(
                    view.point(36.8162, 38.8471),
                    view.point(36.4851, 40.0298),
                    view.point(36.6743, 39.3439),
                );
                path.cubic_bezier_to(
                    view.point(36.8872, 32.4609),
                    view.point(37.3366, 37.1441),
                    view.point(37.3839, 33.7618),
                );
                path.cubic_bezier_to(
                    view.point(36.5087, 31.2073),
                    view.point(36.7453, 32.1298),
                    view.point(36.5797, 31.5621),
                );
                path.cubic_bezier_to(
                    view.point(33.1735, 25.5306),
                    view.point(36.2249, 29.6935),
                    view.point(34.7583, 27.21),
                );
                path.cubic_bezier_to(
                    view.point(32.0381, 24.0878),
                    view.point(32.5585, 24.8447),
                    view.point(32.0381, 24.2061),
                );
                path.cubic_bezier_to(
                    view.point(36.1776, 20.3034),
                    view.point(32.0381, 23.2127),
                    view.point(34.9002, 20.5635),
                );
                path.cubic_bezier_to(
                    view.point(37.1474, 19.9959),
                    view.point(36.3905, 20.2561),
                    view.point(36.8399, 20.1141),
                );
                path.cubic_bezier_to(
                    view.point(40.2461, 22.1483),
                    view.point(38.0936, 19.6174),
                    view.point(38.614, 19.9959),
                );
                path.cubic_bezier_to(
                    view.point(42.5642, 26.0037),
                    view.point(40.9557, 23.0944),
                    view.point(42.3986, 25.507),
                );
                path.cubic_bezier_to(
                    view.point(43.1083, 27.1154),
                    view.point(42.6115, 26.1456),
                    view.point(42.8481, 26.666),
                );
                path.cubic_bezier_to(
                    view.point(43.534, 28.2034),
                    view.point(43.3448, 27.5884),
                    view.point(43.534, 28.0851),
                );
                path.cubic_bezier_to(
                    view.point(43.8889, 29.2441),
                    view.point(43.534, 28.3453),
                    view.point(43.6996, 28.7947),
                );
                path.cubic_bezier_to(
                    view.point(44.4566, 31.16),
                    view.point(44.0781, 29.6935),
                    view.point(44.3383, 30.545),
                );
                path.cubic_bezier_to(
                    view.point(44.9533, 33.1941),
                    view.point(44.5748, 31.775),
                    view.point(44.7877, 32.6974),
                );
                path.cubic_bezier_to(
                    view.point(45.2135, 37.0495),
                    view.point(45.1898, 34.0456),
                    view.point(45.2135, 34.3295),
                );
                path.cubic_bezier_to(
                    view.point(44.8823, 40.9049),
                    view.point(45.2135, 39.9825),
                    view.point(45.2135, 40.0061),
                );
                path.cubic_bezier_to(
                    view.point(44.4566, 42.3951),
                    view.point(44.7168, 41.4016),
                    view.point(44.5039, 42.0639),
                );
                path.cubic_bezier_to(
                    view.point(44.0071, 43.5067),
                    view.point(44.3856, 42.7262),
                    view.point(44.1727, 43.2229),
                );
                path.cubic_bezier_to(
                    view.point(43.5577, 44.3582),
                    view.point(43.8179, 43.7906),
                    view.point(43.6287, 44.169),
                );
                path.cubic_bezier_to(
                    view.point(40.0332, 48.3083),
                    view.point(43.1792, 45.3044),
                    view.point(41.1686, 47.575),
                );
                path.cubic_bezier_to(
                    view.point(38.9924, 49.0178),
                    view.point(39.7257, 48.4975),
                    view.point(39.2526, 48.8286),
                );
                path.cubic_bezier_to(
                    view.point(38.1409, 49.4672),
                    view.point(38.7322, 49.1834),
                    view.point(38.3538, 49.3963),
                );
                path.cubic_bezier_to(
                    view.point(37.313, 49.893),
                    view.point(37.9043, 49.5619),
                    view.point(37.5495, 49.7274),
                );
                path.cubic_bezier_to(
                    view.point(36.012, 50.3897),
                    view.point(37.1001, 50.0349),
                    view.point(36.5087, 50.2714),
                );
                path.cubic_bezier_to(
                    view.point(34.6637, 50.8864),
                    view.point(35.5152, 50.5316),
                    view.point(34.9239, 50.7445),
                );
                path.cubic_bezier_to(
                    view.point(29.2705, 51.8562),
                    view.point(34.2379, 51.0993),
                    view.point(33.5519, 51.2175),
                );
                path.cubic_bezier_to(
                    view.point(23.3333, 51.9508),
                    view.point(28.277, 51.9981),
                    view.point(24.6579, 52.0454),
                );
                path.close();
                path.move_to(view.point(22.0086, 47.7642));
                path.cubic_bezier_to(
                    view.point(18.9572, 47.1493),
                    view.point(20.3292, 47.646),
                    view.point(19.5959, 47.5041),
                );
                path.cubic_bezier_to(
                    view.point(17.8928, 46.6762),
                    view.point(18.7207, 46.9837),
                    view.point(18.224, 46.7945),
                );
                path.cubic_bezier_to(
                    view.point(17.2305, 46.3687),
                    view.point(17.5616, 46.5579),
                    view.point(17.2541, 46.416),
                );
                path.cubic_bezier_to(
                    view.point(16.4972, 45.8247),
                    view.point(17.1832, 46.2978),
                    view.point(16.852, 46.0612),
                );
                path.cubic_bezier_to(
                    view.point(13.0673, 42.3004),
                    view.point(15.2435, 45.0205),
                    view.point(14.5812, 44.3346),
                );
                path.cubic_bezier_to(
                    view.point(12.4523, 41.1651),
                    view.point(12.8544, 41.993),
                    view.point(12.5706, 41.4726),
                );
                path.cubic_bezier_to(
                    view.point(11.9556, 39.9352),
                    view.point(12.334, 40.834),
                    view.point(12.1212, 40.29),
                );
                path.cubic_bezier_to(
                    view.point(12.0029, 31.9405),
                    view.point(11.2223, 38.3031),
                    view.point(11.2459, 33.478),
                );
                path.cubic_bezier_to(
                    view.point(12.4996, 30.7579),
                    view.point(12.1921, 31.5621),
                    view.point(12.405, 31.0181),
                );
                path.cubic_bezier_to(
                    view.point(12.9491, 29.8827),
                    view.point(12.5942, 30.4741),
                    view.point(12.8071, 30.0956),
                );
                path.cubic_bezier_to(
                    view.point(13.4221, 29.1732),
                    view.point(13.1146, 29.6935),
                    view.point(13.3275, 29.3624),
                );
                path.cubic_bezier_to(
                    view.point(17.0886, 25.7908),
                    view.point(13.8243, 28.3926),
                    view.point(16.237, 26.1456),
                );
                path.cubic_bezier_to(
                    view.point(17.9638, 25.3178),
                    view.point(17.3961, 25.6489),
                    view.point(17.7745, 25.436),
                );
                path.cubic_bezier_to(
                    view.point(20.3528, 24.5845),
                    view.point(18.3659, 25.0103),
                    view.point(20.1163, 24.4899),
                );
                path.cubic_bezier_to(
                    view.point(19.9744, 26.1929),
                    view.point(20.7077, 24.7264),
                    view.point(20.4711, 25.7672),
                );
                path.cubic_bezier_to(
                    view.point(19.5486, 27.1627),
                    view.point(19.5959, 26.524),
                    view.point(19.5486, 26.595),
                );
                path.line_to(view.point(19.5486, 27.7776));
                path.line_to(view.point(20.0453, 27.7776));
                path.cubic_bezier_to(
                    view.point(20.9679, 27.0444),
                    view.point(20.5894, 27.754),
                    view.point(20.9679, 27.4465),
                );
                path.cubic_bezier_to(
                    view.point(21.2754, 26.4531),
                    view.point(20.9679, 26.9498),
                    view.point(21.1098, 26.666),
                );
                path.cubic_bezier_to(
                    view.point(21.7958, 25.5779),
                    view.point(21.4409, 26.2402),
                    view.point(21.6538, 25.8381),
                );
                path.cubic_bezier_to(
                    view.point(22.3635, 24.8684),
                    view.point(21.914, 25.2704),
                    view.point(22.1506, 25.0103),
                );
                path.cubic_bezier_to(
                    view.point(23.1441, 24.3716),
                    view.point(22.5527, 24.7737),
                    view.point(22.9075, 24.5372),
                );
                path.cubic_bezier_to(
                    view.point(24.4214, 24.2061),
                    view.point(23.5935, 24.0169),
                    view.point(24.2085, 23.9459),
                );
                path.cubic_bezier_to(
                    view.point(24.2558, 24.5845),
                    view.point(24.5397, 24.348),
                    view.point(24.4923, 24.4189),
                );
                path.cubic_bezier_to(
                    view.point(23.4989, 25.6489),
                    view.point(23.6408, 25.0103),
                    view.point(23.4989, 25.1995),
                );
                path.cubic_bezier_to(
                    view.point(23.215, 26.7369),
                    view.point(23.4989, 25.8854),
                    view.point(23.3806, 26.3821),
                );
                path.cubic_bezier_to(
                    view.point(22.4817, 27.4938),
                    view.point(22.9785, 27.3046),
                    view.point(22.8839, 27.3992),
                );
                path.cubic_bezier_to(
                    view.point(21.8904, 28.2507),
                    view.point(21.8904, 27.6594),
                    view.point(21.7011, 27.8723),
                );
                path.cubic_bezier_to(
                    view.point(21.7958, 29.2914),
                    view.point(22.0796, 28.6764),
                    view.point(22.0323, 29.0549),
                );
                path.cubic_bezier_to(
                    view.point(21.4883, 29.8827),
                    view.point(21.6538, 29.386),
                    view.point(21.5356, 29.6699),
                );
                path.cubic_bezier_to(
                    view.point(21.0861, 30.616),
                    view.point(21.4409, 30.0956),
                    view.point(21.2517, 30.4268),
                );
                path.cubic_bezier_to(
                    view.point(20.8969, 31.7277),
                    view.point(20.7313, 30.9708),
                    view.point(20.6367, 31.4911),
                );
                path.cubic_bezier_to(
                    view.point(21.6775, 31.0654),
                    view.point(21.2281, 32.0824),
                    view.point(21.6775, 31.704),
                );
                path.cubic_bezier_to(
                    view.point(21.9613, 30.4741),
                    view.point(21.6775, 30.6869),
                    view.point(21.7248, 30.5687),
                );
                path.cubic_bezier_to(
                    view.point(22.529, 29.9064),
                    view.point(22.1033, 30.4031),
                    view.point(22.3635, 30.1429),
                );
                path.cubic_bezier_to(
                    view.point(23.8537, 29.386),
                    view.point(22.8129, 29.4806),
                    view.point(23.5225, 29.1968),
                );
                path.cubic_bezier_to(
                    view.point(23.6881, 30.5687),
                    view.point(24.0666, 29.5279),
                    view.point(23.9719, 30.2375),
                );
                path.cubic_bezier_to(
                    view.point(23.286, 31.3256),
                    view.point(23.5462, 30.7579),
                    view.point(23.3569, 31.089),
                );
                path.cubic_bezier_to(
                    view.point(22.7656, 32.0351),
                    view.point(23.215, 31.5621),
                    view.point(22.9785, 31.8696),
                );
                path.cubic_bezier_to(
                    view.point(22.3635, 32.9103),
                    view.point(22.3871, 32.319),
                    view.point(22.3635, 32.4136),
                );
                path.cubic_bezier_to(
                    view.point(22.0086, 33.8091),
                    view.point(22.3635, 33.3834),
                    view.point(22.3162, 33.5253),
                );
                path.cubic_bezier_to(
                    view.point(21.8667, 34.8498),
                    view.point(21.5592, 34.2112),
                    view.point(21.4883, 34.8025),
                );
                path.cubic_bezier_to(
                    view.point(22.4817, 34.0929),
                    view.point(22.1506, 34.8971),
                    view.point(22.4817, 34.495),
                );
                path.cubic_bezier_to(
                    view.point(23.0967, 33.5489),
                    view.point(22.4817, 33.8801),
                    view.point(22.6, 33.7618),
                );
                path.cubic_bezier_to(
                    view.point(23.7354, 33.1232),
                    view.point(23.4516, 33.407),
                    view.point(23.7354, 33.2178),
                );
                path.cubic_bezier_to(
                    view.point(24.8235, 30.9235),
                    view.point(23.7354, 32.9339),
                    view.point(24.6816, 31.0181),
                );
                path.cubic_bezier_to(
                    view.point(25.7697, 30.9708),
                    view.point(25.0127, 30.8052),
                    view.point(25.6751, 30.8289),
                );
                path.cubic_bezier_to(
                    view.point(26.3374, 31.3019),
                    view.point(25.7933, 31.0417),
                    view.point(26.0772, 31.1836),
                );
                path.cubic_bezier_to(
                    view.point(26.8578, 31.9878),
                    view.point(26.8105, 31.5148),
                    view.point(26.8578, 31.5384),
                );
                path.cubic_bezier_to(
                    view.point(25.5331, 33.8091),
                    view.point(26.8578, 32.4609),
                    view.point(26.1008, 33.5016),
                );
                path.cubic_bezier_to(
                    view.point(25.1783, 34.8025),
                    view.point(25.2966, 33.9274),
                    view.point(25.2493, 34.0693),
                );
                path.cubic_bezier_to(
                    view.point(24.7762, 36.0325),
                    view.point(25.1074, 35.5594),
                    view.point(25.06, 35.725),
                );
                path.cubic_bezier_to(
                    view.point(24.445, 37.357),
                    view.point(24.3741, 36.4346),
                    view.point(24.2085, 37.1205),
                );
                path.cubic_bezier_to(
                    view.point(25.2493, 36.6474),
                    view.point(24.7052, 37.6172),
                    view.point(25.1074, 37.2624),
                );
                path.cubic_bezier_to(
                    view.point(25.6514, 35.9615),
                    view.point(25.3202, 36.3399),
                    view.point(25.4858, 36.0798),
                );
                path.cubic_bezier_to(
                    view.point(26.1955, 35.5121),
                    view.point(25.7933, 35.8669),
                    view.point(26.0535, 35.654),
                );
                path.cubic_bezier_to(
                    view.point(27.0707, 35.5594),
                    view.point(26.5739, 35.1337),
                    view.point(26.7395, 35.1337),
                );
                path.cubic_bezier_to(
                    view.point(27.4728, 35.9142),
                    view.point(27.2126, 35.7486),
                    view.point(27.3782, 35.9142),
                );
                path.cubic_bezier_to(
                    view.point(27.8749, 36.1034),
                    view.point(27.5674, 35.9142),
                    view.point(27.733, 36.0088),
                );
                path.line_to(view.point(28.1351, 36.3163));
                path.line_to(view.point(27.8986, 37.1205));
                path.cubic_bezier_to(
                    view.point(27.4255, 38.1376),
                    view.point(27.7566, 37.5699),
                    view.point(27.5437, 38.0429),
                );
                path.cubic_bezier_to(
                    view.point(26.9997, 38.8945),
                    view.point(27.3072, 38.2558),
                    view.point(27.118, 38.587),
                );
                path.cubic_bezier_to(
                    view.point(26.4793, 39.7933),
                    view.point(26.9051, 39.1783),
                    view.point(26.6685, 39.5804),
                );
                path.cubic_bezier_to(
                    view.point(26.0772, 40.6211),
                    view.point(26.3137, 40.0061),
                    view.point(26.1245, 40.3846),
                );
                path.cubic_bezier_to(
                    view.point(25.6278, 41.4726),
                    view.point(26.0299, 40.9286),
                    view.point(25.8879, 41.2124),
                );
                path.cubic_bezier_to(
                    view.point(25.4622, 42.9154),
                    view.point(25.1547, 41.9693),
                    view.point(25.0364, 42.9154),
                );
                path.cubic_bezier_to(
                    view.point(26.1481, 42.1822),
                    view.point(25.7933, 42.9154),
                    view.point(26.1481, 42.5133),
                );
                path.cubic_bezier_to(
                    view.point(26.6685, 41.378),
                    view.point(26.1481, 41.9693),
                    view.point(26.3374, 41.7091),
                );
                path.cubic_bezier_to(
                    view.point(27.1889, 40.6448),
                    view.point(26.9524, 41.0942),
                    view.point(27.1889, 40.763),
                );
                path.cubic_bezier_to(
                    view.point(27.662, 39.8879),
                    view.point(27.1889, 40.5028),
                    view.point(27.4018, 40.1717),
                );
                path.cubic_bezier_to(
                    view.point(28.1115, 39.1783),
                    view.point(27.8986, 39.5804),
                    view.point(28.1115, 39.2729),
                );
                path.cubic_bezier_to(
                    view.point(28.5136, 38.587),
                    view.point(28.1115, 39.0837),
                    view.point(28.3007, 38.8235),
                );
                path.cubic_bezier_to(
                    view.point(29.0813, 37.7118),
                    view.point(28.7265, 38.3504),
                    view.point(28.9867, 37.9483),
                );
                path.cubic_bezier_to(
                    view.point(29.7909, 37.4043),
                    view.point(29.2469, 37.2861),
                    view.point(29.6253, 37.1205),
                );
                path.cubic_bezier_to(
                    view.point(30.3113, 37.7591),
                    view.point(29.8146, 37.4753),
                    view.point(30.0511, 37.6172),
                );
                path.cubic_bezier_to(
                    view.point(30.1221, 39.7459),
                    view.point(30.95, 38.0666),
                    view.point(30.9027, 38.7525),
                );
                path.cubic_bezier_to(
                    view.point(29.6963, 40.5975),
                    view.point(29.9565, 39.9588),
                    view.point(29.7673, 40.3373),
                );
                path.cubic_bezier_to(
                    view.point(29.1995, 41.449),
                    view.point(29.649, 40.834),
                    view.point(29.4124, 41.2124),
                );
                path.cubic_bezier_to(
                    view.point(28.7974, 42.2531),
                    view.point(28.9394, 41.7564),
                    view.point(28.7974, 41.993),
                );
                path.cubic_bezier_to(
                    view.point(28.3953, 43.0573),
                    view.point(28.7974, 42.4897),
                    view.point(28.6792, 42.7499),
                );
                path.cubic_bezier_to(
                    view.point(28.0878, 44.24),
                    view.point(28.0168, 43.4831),
                    view.point(27.8513, 44.0981),
                );
                path.cubic_bezier_to(
                    view.point(28.9157, 43.6014),
                    view.point(28.3007, 44.3582),
                    view.point(28.7028, 44.0508),
                );
                path.cubic_bezier_to(
                    view.point(29.4597, 42.8445),
                    view.point(29.034, 43.3175),
                    view.point(29.2705, 42.9864),
                );
                path.cubic_bezier_to(
                    view.point(29.9328, 42.1349),
                    view.point(29.6726, 42.7025),
                    view.point(29.8619, 42.4187),
                );
                path.cubic_bezier_to(
                    view.point(30.3823, 41.4726),
                    view.point(30.0274, 41.851),
                    view.point(30.2167, 41.5672),
                );
                path.cubic_bezier_to(
                    view.point(30.95, 40.6684),
                    view.point(30.5478, 41.3543),
                    view.point(30.808, 40.9995),
                );
                path.cubic_bezier_to(
                    view.point(31.3284, 40.0534),
                    view.point(31.1155, 40.3136),
                    view.point(31.2811, 40.0534),
                );
                path.cubic_bezier_to(
                    view.point(31.6596, 39.5804),
                    view.point(31.3757, 40.0534),
                    view.point(31.5177, 39.8406),
                );
                path.cubic_bezier_to(
                    view.point(32.8423, 39.2966),
                    view.point(32.0617, 38.8708),
                    view.point(32.2746, 38.8235),
                );
                path.cubic_bezier_to(
                    view.point(33.4573, 39.6986),
                    view.point(33.1025, 39.5331),
                    view.point(33.3864, 39.6986),
                );
                path.cubic_bezier_to(
                    view.point(33.9068, 39.9352),
                    view.point(33.5519, 39.6986),
                    view.point(33.7412, 39.8169),
                );
                path.cubic_bezier_to(
                    view.point(33.8831, 41.3307),
                    view.point(34.3089, 40.2663),
                    view.point(34.3089, 40.6684),
                );
                path.cubic_bezier_to(
                    view.point(33.3154, 42.3714),
                    view.point(33.7175, 41.6145),
                    view.point(33.4573, 42.0876),
                );
                path.cubic_bezier_to(
                    view.point(31.8725, 44.1454),
                    view.point(33.1498, 42.6789),
                    view.point(32.5348, 43.4594),
                );
                path.cubic_bezier_to(
                    view.point(27.5674, 47.0074),
                    view.point(30.4059, 45.7065),
                    view.point(29.2705, 46.4397),
                );
                path.cubic_bezier_to(
                    view.point(22.0086, 47.7642),
                    view.point(25.2729, 47.7879),
                    view.point(24.3504, 47.9062),
                );
                path.close();
                path.move_to(view.point(33.9068, 38.1612));
                path.cubic_bezier_to(
                    view.point(33.7412, 36.4819),
                    view.point(33.1971, 37.8301),
                    view.point(33.1262, 37.1441),
                );
                path.cubic_bezier_to(
                    view.point(34.8293, 36.7893),
                    view.point(34.1669, 36.0561),
                    view.point(34.3562, 36.1034),
                );
                path.cubic_bezier_to(
                    view.point(33.9068, 38.1612),
                    view.point(35.5152, 37.8064),
                    view.point(34.9712, 38.6106),
                );
                path.close();
                path.move_to(view.point(30.879, 36.4819));
                path.cubic_bezier_to(
                    view.point(30.4296, 36.1034),
                    view.point(30.7134, 36.4582),
                    view.point(30.5242, 36.269),
                );
                path.cubic_bezier_to(
                    view.point(30.7134, 35.1573),
                    view.point(30.264, 35.8196),
                    view.point(30.2876, 35.7723),
                );
                path.cubic_bezier_to(
                    view.point(32.0144, 34.9681),
                    view.point(31.2338, 34.4004),
                    view.point(31.3521, 34.3768),
                );
                path.line_to(view.point(32.5348, 35.4175));
                path.line_to(view.point(32.3929, 35.8905));
                path.cubic_bezier_to(
                    view.point(30.879, 36.4819),
                    view.point(32.2036, 36.5528),
                    view.point(31.7306, 36.742),
                );
                path.close();
                path.move_to(view.point(27.9459, 34.8025));
                path.cubic_bezier_to(
                    view.point(27.2362, 33.9037),
                    view.point(27.2599, 34.5423),
                    view.point(27.1653, 34.4241),
                );
                path.cubic_bezier_to(
                    view.point(28.0878, 32.7684),
                    view.point(27.2836, 33.4307),
                    view.point(27.5437, 33.0759),
                );
                path.cubic_bezier_to(
                    view.point(28.8447, 32.8866),
                    view.point(28.419, 32.5792),
                    view.point(28.4426, 32.5792),
                );
                path.cubic_bezier_to(
                    view.point(29.5071, 33.3834),
                    view.point(29.0813, 33.0522),
                    view.point(29.3651, 33.2887),
                );
                path.cubic_bezier_to(
                    view.point(28.7265, 34.9681),
                    view.point(30.0511, 33.7618),
                    view.point(29.4361, 34.9917),
                );
                path.cubic_bezier_to(
                    view.point(27.9459, 34.8025),
                    view.point(28.5372, 34.9681),
                    view.point(28.2061, 34.8735),
                );
                path.close();
                path.move_to(view.point(32.4402, 33.478));
                path.cubic_bezier_to(
                    view.point(32.5585, 32.1534),
                    view.point(32.0381, 33.1468),
                    view.point(32.0617, 32.7684),
                );
                path.cubic_bezier_to(
                    view.point(33.7885, 32.0824),
                    view.point(33.0788, 31.5384),
                    view.point(33.2444, 31.5148),
                );
                path.cubic_bezier_to(
                    view.point(33.8594, 33.3834),
                    view.point(34.3089, 32.6265),
                    view.point(34.3325, 32.9339),
                );
                path.cubic_bezier_to(
                    view.point(32.4402, 33.478),
                    view.point(33.4337, 33.7854),
                    view.point(32.8896, 33.8328),
                );
                path.close();
                path.move_to(view.point(29.7436, 31.9642));
                path.cubic_bezier_to(
                    view.point(29.578, 30.1193),
                    view.point(28.963, 31.704),
                    view.point(28.8684, 30.8525),
                );
                path.cubic_bezier_to(
                    view.point(30.95, 30.2375),
                    view.point(29.9801, 29.6935),
                    view.point(30.335, 29.7172),
                );
                path.cubic_bezier_to(
                    view.point(30.9973, 31.6567),
                    view.point(31.6123, 30.7815),
                    view.point(31.6123, 31.0654),
                );
                path.cubic_bezier_to(
                    view.point(29.7436, 31.9642),
                    view.point(30.5005, 32.1298),
                    view.point(30.3586, 32.1534),
                );
                path.close();
                path.move_to(view.point(7.93434, 31.7986));
                path.cubic_bezier_to(
                    view.point(8.12358, 30.0956),
                    view.point(7.74511, 31.6094),
                    view.point(7.86338, 30.6396),
                );
                path.cubic_bezier_to(
                    view.point(8.64397, 28.5345),
                    view.point(8.28916, 29.7645),
                    view.point(8.5257, 29.0785),
                );
                path.cubic_bezier_to(
                    view.point(9.28264, 26.8788),
                    view.point(8.7859, 27.9905),
                    view.point(9.06975, 27.2573),
                );
                path.cubic_bezier_to(
                    view.point(9.80303, 25.6489),
                    view.point(9.47187, 26.4767),
                    view.point(9.70842, 25.9327),
                );
                path.cubic_bezier_to(
                    view.point(10.2288, 24.7737),
                    view.point(9.874, 25.3651),
                    view.point(10.0632, 24.963),
                );
                path.cubic_bezier_to(
                    view.point(10.6782, 23.9459),
                    view.point(10.3944, 24.5609),
                    view.point(10.5836, 24.2061),
                );
                path.cubic_bezier_to(
                    view.point(11.1986, 22.9998),
                    view.point(10.7492, 23.7094),
                    view.point(10.9857, 23.2836),
                );
                path.cubic_bezier_to(
                    view.point(11.9083, 21.9118),
                    view.point(11.3879, 22.7159),
                    view.point(11.719, 22.2429),
                );
                path.cubic_bezier_to(
                    view.point(13.3985, 22.101),
                    view.point(12.476, 21.0603),
                    view.point(12.8308, 21.0839),
                );
                path.cubic_bezier_to(
                    view.point(13.8243, 23.1417),
                    view.point(13.635, 22.5031),
                    view.point(13.8243, 22.9761),
                );
                path.cubic_bezier_to(
                    view.point(12.7125, 24.5609),
                    view.point(13.8479, 23.4728),
                    view.point(13.3512, 24.0642),
                );
                path.cubic_bezier_to(
                    view.point(9.42456, 28.8184),
                    view.point(11.719, 25.2941),
                    view.point(10.2525, 27.21),
                );
                path.cubic_bezier_to(
                    view.point(8.90417, 29.5989),
                    view.point(9.28264, 29.1495),
                    view.point(9.0461, 29.5043),
                );
                path.cubic_bezier_to(
                    view.point(8.59666, 30.4741),
                    view.point(8.76224, 29.7408),
                    view.point(8.64397, 30.0483),
                );
                path.cubic_bezier_to(
                    view.point(7.93434, 31.7986),
                    view.point(8.47839, 31.4911),
                    view.point(8.19454, 32.0588),
                );
                path.close();
                path.move_to(view.point(26.976, 30.3085));
                path.cubic_bezier_to(
                    view.point(26.361, 30.0247),
                    view.point(26.8105, 30.2139),
                    view.point(26.5266, 30.0956),
                );
                path.cubic_bezier_to(
                    view.point(26.2428, 29.0312),
                    view.point(25.9826, 29.9064),
                    view.point(25.9589, 29.6462),
                );
                path.cubic_bezier_to(
                    view.point(27.9695, 28.3217),
                    view.point(26.8105, 27.7776),
                    view.point(27.1889, 27.6121),
                );
                path.cubic_bezier_to(
                    view.point(28.3007, 29.5279),
                    view.point(28.7501, 28.9839),
                    view.point(28.7501, 29.0312),
                );
                path.cubic_bezier_to(
                    view.point(27.7803, 30.2375),
                    view.point(28.0878, 29.7881),
                    view.point(27.8513, 30.0956),
                );
                path.cubic_bezier_to(
                    view.point(26.976, 30.3085),
                    view.point(27.6384, 30.5214),
                    view.point(27.3782, 30.545),
                );
                path.close();
                path.move_to(view.point(23.972, 28.6528));
                path.cubic_bezier_to(
                    view.point(24.0429, 27.7067),
                    view.point(23.8064, 28.3217),
                    view.point(23.83, 27.8959),
                );
                path.cubic_bezier_to(
                    view.point(24.516, 27.1154),
                    view.point(24.1375, 27.6357),
                    view.point(24.3504, 27.3519),
                );
                path.cubic_bezier_to(
                    view.point(25.2256, 26.7133),
                    view.point(24.7525, 26.7133),
                    view.point(24.8235, 26.6896),
                );
                path.cubic_bezier_to(
                    view.point(25.4149, 27.6121),
                    view.point(25.7697, 26.7606),
                    view.point(25.8406, 27.1154),
                );
                path.cubic_bezier_to(
                    view.point(25.06, 28.227),
                    view.point(25.2729, 27.8013),
                    view.point(25.1074, 28.0851),
                );
                path.cubic_bezier_to(
                    view.point(23.972, 28.6528),
                    view.point(24.8708, 28.842),
                    view.point(24.2085, 29.1022),
                );
                path.close();
                path.move_to(view.point(28.5845, 27.2336));
                path.cubic_bezier_to(
                    view.point(28.1351, 26.666),
                    view.point(28.2297, 27.1154),
                    view.point(28.1588, 27.0444),
                );
                path.cubic_bezier_to(
                    view.point(29.2942, 25.8145),
                    view.point(28.0641, 25.8381),
                    view.point(28.5372, 25.4833),
                );
                path.cubic_bezier_to(
                    view.point(29.649, 26.9498),
                    view.point(29.8382, 26.0273),
                    view.point(29.9565, 26.4294),
                );
                path.cubic_bezier_to(
                    view.point(28.5845, 27.2336),
                    view.point(29.3888, 27.3755),
                    view.point(29.1522, 27.4465),
                );
                path.close();
                path.move_to(view.point(29.4361, 22.645));
                path.cubic_bezier_to(
                    view.point(28.7974, 22.2665),
                    view.point(29.3178, 22.574),
                    view.point(29.034, 22.4085),
                );
                path.cubic_bezier_to(
                    view.point(27.662, 21.7935),
                    view.point(28.5845, 22.101),
                    view.point(28.0641, 21.9118),
                );
                path.cubic_bezier_to(
                    view.point(26.503, 21.415),
                    view.point(27.2599, 21.6989),
                    view.point(26.7395, 21.5097),
                );
                path.cubic_bezier_to(
                    view.point(18.224, 21.4387),
                    view.point(24.9891, 20.6345),
                    view.point(20.4238, 20.6582),
                );
                path.cubic_bezier_to(
                    view.point(16.0005, 22.1719),
                    view.point(16.4026, 22.101),
                    view.point(16.3316, 22.1246),
                );
                path.cubic_bezier_to(
                    view.point(15.622, 21.7935),
                    view.point(15.7166, 22.2192),
                    view.point(15.693, 22.1956),
                );
                path.cubic_bezier_to(
                    view.point(14.7941, 19.8303),
                    view.point(15.4564, 20.8474),
                    view.point(15.1962, 20.2088),
                );
                path.cubic_bezier_to(
                    view.point(15.0779, 18.1983),
                    view.point(14.1791, 19.2863),
                    view.point(14.2264, 18.9788),
                );
                path.cubic_bezier_to(
                    view.point(17.0649, 16.6135),
                    view.point(15.9768, 17.3468),
                    view.point(16.5682, 16.8737),
                );
                path.cubic_bezier_to(
                    view.point(17.8691, 16.0932),
                    view.point(17.2778, 16.4953),
                    view.point(17.6563, 16.2824),
                );
                path.cubic_bezier_to(
                    view.point(18.6734, 15.6438),
                    view.point(18.082, 15.9276),
                    view.point(18.4605, 15.7147),
                );
                path.cubic_bezier_to(
                    view.point(19.5249, 15.218),
                    view.point(18.8863, 15.5728),
                    view.point(19.2647, 15.3836),
                );
                path.cubic_bezier_to(
                    view.point(20.613, 14.8159),
                    view.point(19.7615, 15.0761),
                    view.point(20.2582, 14.8869),
                );
                path.cubic_bezier_to(
                    view.point(22.2452, 14.4138),
                    view.point(20.9679, 14.745),
                    view.point(21.7011, 14.5557),
                );
                path.cubic_bezier_to(
                    view.point(28.5845, 14.3665),
                    view.point(24.2795, 13.8462),
                    view.point(26.8105, 13.8225),
                );
                path.cubic_bezier_to(
                    view.point(30.2403, 14.7686),
                    view.point(29.0103, 14.5084),
                    view.point(29.7436, 14.674),
                );
                path.cubic_bezier_to(
                    view.point(31.4467, 15.1471),
                    view.point(30.7371, 14.8632),
                    view.point(31.2811, 15.0288),
                );
                path.cubic_bezier_to(
                    view.point(32.5585, 15.6438),
                    view.point(31.6123, 15.2417),
                    view.point(32.109, 15.4782),
                );
                path.cubic_bezier_to(
                    view.point(32.9369, 17.0393),
                    view.point(33.5992, 16.0222),
                    view.point(33.6466, 16.2824),
                );
                path.cubic_bezier_to(
                    view.point(30.5951, 21.5806),
                    view.point(31.6359, 18.4111),
                    view.point(30.7371, 20.1614),
                );
                path.cubic_bezier_to(
                    view.point(30.4059, 22.574),
                    view.point(30.5478, 22.0064),
                    view.point(30.4532, 22.4794),
                );
                path.cubic_bezier_to(
                    view.point(29.4361, 22.645),
                    view.point(30.2876, 22.8106),
                    view.point(29.7909, 22.8342),
                );
                path.close();
                path.move_to(view.point(19.9034, 12.2851));
                path.cubic_bezier_to(
                    view.point(19.7142, 11.339),
                    view.point(19.7615, 12.1195),
                    view.point(19.7142, 11.883),
                );
                path.cubic_bezier_to(
                    view.point(21.4646, 10.109),
                    view.point(19.7142, 10.2273),
                    view.point(19.7615, 10.18),
                );
                path.cubic_bezier_to(
                    view.point(22.0086, 8.97369),
                    view.point(24.2085, 9.9671),
                    view.point(24.5397, 9.30482),
                );
                path.cubic_bezier_to(
                    view.point(20.5657, 8.59524),
                    view.point(21.4646, 8.90273),
                    view.point(20.8259, 8.73716),
                );
                path.cubic_bezier_to(
                    view.point(19.5959, 8.09853),
                    view.point(20.3292, 8.47698),
                    view.point(19.8798, 8.24045),
                );
                path.cubic_bezier_to(
                    view.point(18.0347, 3.86469),
                    view.point(18.1293, 7.3653),
                    view.point(17.4907, 5.6623),
                );
                path.cubic_bezier_to(
                    view.point(21.37, 0.884435),
                    view.point(18.3422, 2.80031),
                    view.point(19.9744, 1.33384),
                );
                path.cubic_bezier_to(
                    view.point(22.4817, 0.435032),
                    view.point(21.7484, 0.766171),
                    view.point(22.2688, 0.553296),
                );
                path.cubic_bezier_to(
                    view.point(29.034, 0.316768),
                    view.point(23.4989, -0.085329),
                    view.point(27.733, -0.156288),
                );
                path.cubic_bezier_to(
                    view.point(31.494, 1.31018),
                    view.point(30.8553, 0.979046),
                    view.point(31.2575, 1.14462),
                );
                path.cubic_bezier_to(
                    view.point(32.1563, 1.75959),
                    view.point(31.6359, 1.4048),
                    view.point(31.9198, 1.61767),
                );
                path.cubic_bezier_to(
                    view.point(34.1906, 4.95271),
                    view.point(33.2444, 2.49282),
                    view.point(34.1433, 3.88834),
                );
                path.cubic_bezier_to(
                    view.point(33.481, 7.10512),
                    view.point(34.2616, 5.92248),
                    view.point(34.1433, 6.22996),
                );
                path.cubic_bezier_to(
                    view.point(31.9434, 8.2641),
                    view.point(33.0079, 7.69644),
                    view.point(32.795, 7.862),
                );
                path.cubic_bezier_to(
                    view.point(28.7265, 9.02099),
                    view.point(30.7134, 8.83177),
                    view.point(30.7371, 8.80812),
                );
                path.cubic_bezier_to(
                    view.point(27.3072, 9.94345),
                    view.point(26.4793, 9.23387),
                    view.point(26.1008, 9.47039),
                );
                path.cubic_bezier_to(
                    view.point(28.419, 10.5111),
                    view.point(27.662, 10.0854),
                    view.point(28.1588, 10.3219),
                );
                path.cubic_bezier_to(
                    view.point(29.7199, 11.0315),
                    view.point(28.6792, 10.6767),
                    view.point(29.2705, 10.9132),
                );
                path.cubic_bezier_to(
                    view.point(31.9671, 12.1905),
                    view.point(30.4296, 11.2207),
                    view.point(31.8015, 11.9303),
                );
                path.cubic_bezier_to(
                    view.point(25.9353, 12.4743),
                    view.point(32.1327, 12.427),
                    view.point(31.3757, 12.4743),
                );
                path.cubic_bezier_to(
                    view.point(19.9034, 12.2851),
                    view.point(20.5421, 12.4743),
                    view.point(20.0926, 12.4506),
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
