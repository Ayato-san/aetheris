use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct AchievementsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl AchievementsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for AchievementsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(26.902, 43.2492));
                path.cubic_bezier_to(
                    view.point(23.2365, 42.8707),
                    view.point(25.4831, 43.1309),
                    view.point(24.0406, 42.989),
                );
                path.cubic_bezier_to(
                    view.point(21.4392, 42.6105),
                    view.point(22.8108, 42.7998),
                    view.point(22.0068, 42.6815),
                );
                path.cubic_bezier_to(
                    view.point(19.6893, 42.2557),
                    view.point(20.8717, 42.5396),
                    view.point(20.0913, 42.374),
                );
                path.cubic_bezier_to(
                    view.point(18.4832, 41.9719),
                    view.point(19.2872, 42.1611),
                    view.point(18.7433, 42.0192),
                );
                path.cubic_bezier_to(
                    view.point(14.3684, 40.6709),
                    view.point(17.9393, 41.8772),
                    view.point(15.5508, 41.1203),
                );
                path.cubic_bezier_to(
                    view.point(12.8786, 40.1032),
                    view.point(13.9427, 40.5053),
                    view.point(13.2569, 40.2451),
                );
                path.cubic_bezier_to(
                    view.point(11.4124, 39.3935),
                    view.point(12.4765, 39.9376),
                    view.point(11.8144, 39.6301),
                );
                path.cubic_bezier_to(
                    view.point(8.33808, 38.7312),
                    view.point(10.3955, 38.8258),
                    view.point(9.94616, 38.7312),
                );
                path.cubic_bezier_to(
                    view.point(5.00367, 37.9506),
                    view.point(6.72999, 38.7312),
                    view.point(6.39892, 38.6603),
                );
                path.cubic_bezier_to(
                    view.point(3.51382, 36.8625),
                    view.point(3.63206, 37.2647),
                    view.point(3.46653, 37.1464),
                );
                path.cubic_bezier_to(
                    view.point(5.50028, 36.4368),
                    view.point(3.53747, 36.5077),
                    view.point(3.91584, 36.4368),
                );
                path.cubic_bezier_to(
                    view.point(6.96648, 36.3185),
                    view.point(6.44621, 36.4131),
                    view.point(6.89553, 36.3894),
                );
                path.cubic_bezier_to(
                    view.point(6.6827, 35.396),
                    view.point(7.15566, 36.1292),
                    view.point(7.06107, 35.8454),
                );
                path.cubic_bezier_to(
                    view.point(4.83813, 34.5208),
                    view.point(6.02054, 34.6627),
                    view.point(5.87865, 34.6154),
                );
                path.cubic_bezier_to(
                    view.point(1.24359, 31.8242),
                    view.point(3.30099, 34.4261),
                    view.point(2.68613, 33.9531),
                );
                path.cubic_bezier_to(
                    view.point(0.794268, 30.4049),
                    view.point(0.676027, 30.9726),
                    view.point(0.605082, 30.7124),
                );
                path.cubic_bezier_to(
                    view.point(2.09493, 30.3576),
                    view.point(0.959807, 30.1447),
                    view.point(1.14899, 30.1447),
                );
                path.cubic_bezier_to(
                    view.point(3.04086, 30.5232),
                    view.point(2.49695, 30.4759),
                    view.point(2.92262, 30.5232),
                );
                path.line_to(view.point(3.25369, 30.4995));
                path.line_to(view.point(2.96991, 30.0974));
                path.cubic_bezier_to(
                    view.point(2.11857, 28.962),
                    view.point(2.80437, 29.8845),
                    view.point(2.426, 29.3641),
                );
                path.cubic_bezier_to(
                    view.point(0.0138742, 23.7344),
                    view.point(1.0544, 27.5664),
                    view.point(0.108467, 25.2483),
                );
                path.cubic_bezier_to(
                    view.point(1.48007, 23.4269),
                    view.point(-0.0807191, 22.4098),
                    view.point(0.297654, 22.3388),
                );
                path.cubic_bezier_to(
                    view.point(2.40235, 22.9302),
                    view.point(2.33141, 24.2548),
                    view.point(2.426, 24.2075),
                );
                path.cubic_bezier_to(
                    view.point(3.01721, 17.9391),
                    view.point(2.3787, 21.8894),
                    view.point(2.75708, 18.8143),
                );
                path.cubic_bezier_to(
                    view.point(3.96314, 17.3477),
                    view.point(3.23004, 17.2058),
                    view.point(3.67936, 16.922),
                );
                path.cubic_bezier_to(
                    view.point(4.5307, 18.8143),
                    view.point(4.03409, 17.466),
                    view.point(4.29422, 18.1283),
                );
                path.cubic_bezier_to(
                    view.point(6.28068, 19.2637),
                    view.point(5.19285, 20.7776),
                    view.point(5.2638, 20.8013),
                );
                path.cubic_bezier_to(
                    view.point(8.29078, 16.6854),
                    view.point(6.77729, 18.5068),
                    view.point(8.03065, 16.922),
                );
                path.cubic_bezier_to(
                    view.point(8.95293, 16.7564),
                    view.point(8.52727, 16.4725),
                    view.point(8.85834, 16.4962),
                );
                path.cubic_bezier_to(
                    view.point(8.97658, 18.223),
                    view.point(8.97658, 16.851),
                    view.point(9.00023, 17.5133),
                );
                path.cubic_bezier_to(
                    view.point(9.09482, 19.6895),
                    view.point(8.92929, 19.453),
                    view.point(8.92929, 19.5003),
                );
                path.cubic_bezier_to(
                    view.point(11.2232, 18.6251),
                    view.point(9.3786, 19.9497),
                    view.point(9.18942, 20.0443),
                );
                path.cubic_bezier_to(
                    view.point(12.6184, 18.1283),
                    view.point(12.1455, 17.9864),
                    view.point(12.3819, 17.9155),
                );
                path.cubic_bezier_to(
                    view.point(12.5711, 19.9261),
                    view.point(12.8313, 18.3176),
                    view.point(12.8313, 18.6724),
                );
                path.cubic_bezier_to(
                    view.point(13.5171, 22.7173),
                    view.point(12.0036, 22.8119),
                    view.point(12.0036, 22.8355),
                );
                path.cubic_bezier_to(
                    view.point(14.6522, 23.0957),
                    view.point(14.4157, 22.6463),
                    view.point(14.6522, 22.7409),
                );
                path.cubic_bezier_to(
                    view.point(12.5002, 25.5085),
                    view.point(14.6522, 23.6161),
                    view.point(13.3752, 25.0354),
                );
                path.cubic_bezier_to(
                    view.point(10.6793, 27.2825),
                    view.point(11.8144, 25.8633),
                    view.point(11.5542, 26.1235),
                );
                path.cubic_bezier_to(
                    view.point(9.70968, 28.891),
                    view.point(10.0171, 28.1578),
                    view.point(9.70968, 28.6782),
                );
                path.cubic_bezier_to(
                    view.point(9.99346, 29.2695),
                    view.point(9.70968, 28.9383),
                    view.point(9.85157, 29.1039),
                );
                path.cubic_bezier_to(
                    view.point(11.2232, 31.7059),
                    view.point(10.3718, 29.648),
                    view.point(10.7266, 30.3576),
                );
                path.cubic_bezier_to(
                    view.point(11.4597, 35.7744),
                    view.point(11.6961, 32.9832),
                    view.point(11.7434, 33.7875),
                );
                path.cubic_bezier_to(
                    view.point(13.2569, 38.4947),
                    view.point(11.2232, 37.4302),
                    view.point(11.2232, 37.4066),
                );
                path.cubic_bezier_to(
                    view.point(16.7805, 40.0086),
                    view.point(14.7941, 39.3226),
                    view.point(15.9055, 39.772),
                );
                path.cubic_bezier_to(
                    view.point(18.0575, 40.3397),
                    view.point(17.1825, 40.1032),
                    view.point(17.7501, 40.2687),
                );
                path.cubic_bezier_to(
                    view.point(19.5001, 40.6472),
                    view.point(18.3413, 40.4343),
                    view.point(18.9798, 40.5763),
                );
                path.cubic_bezier_to(
                    view.point(21.3446, 40.9784),
                    view.point(19.9967, 40.7182),
                    view.point(20.848, 40.8601),
                );
                path.cubic_bezier_to(
                    view.point(23.7095, 41.2859),
                    view.point(21.9595, 41.1203),
                    view.point(22.7872, 41.2149),
                );
                path.cubic_bezier_to(
                    view.point(25.9561, 41.4515),
                    view.point(24.4662, 41.3332),
                    view.point(25.4831, 41.4042),
                );
                path.cubic_bezier_to(
                    view.point(30.9932, 41.5934),
                    view.point(28.3445, 41.688),
                    view.point(29.7634, 41.7117),
                );
                path.cubic_bezier_to(
                    view.point(34.6586, 41.3332),
                    view.point(31.6553, 41.5224),
                    view.point(33.3107, 41.4042),
                );
                path.cubic_bezier_to(
                    view.point(37.9931, 41.0257),
                    view.point(36.3613, 41.2386),
                    view.point(37.4018, 41.144),
                );
                path.cubic_bezier_to(
                    view.point(39.9559, 40.6945),
                    view.point(38.466, 40.9311),
                    view.point(39.341, 40.7655),
                );
                path.cubic_bezier_to(
                    view.point(41.8004, 40.2688),
                    view.point(40.5471, 40.5999),
                    view.point(41.3984, 40.4107),
                );
                path.cubic_bezier_to(
                    view.point(43.0065, 39.9139),
                    view.point(42.2025, 40.1268),
                    view.point(42.7464, 39.9849),
                );
                path.cubic_bezier_to(
                    view.point(46.2936, 38.5656),
                    view.point(43.6214, 39.7957),
                    view.point(45.4896, 39.0151),
                );
                path.cubic_bezier_to(
                    view.point(49.5334, 35.7035),
                    view.point(47.7835, 37.7141),
                    view.point(49.1314, 36.5314),
                );
                path.cubic_bezier_to(
                    view.point(49.0368, 33.0778),
                    view.point(49.9118, 34.9465),
                    view.point(49.6517, 33.7165),
                );
                path.cubic_bezier_to(
                    view.point(47.6179, 33.8348),
                    view.point(48.4693, 32.5101),
                    view.point(48.1855, 32.6521),
                );
                path.cubic_bezier_to(
                    view.point(44.047, 36.7679),
                    view.point(46.8612, 35.3487),
                    view.point(45.9152, 36.1293),
                );
                path.cubic_bezier_to(
                    view.point(42.5099, 35.7035),
                    view.point(42.1079, 37.4066),
                    view.point(41.7531, 37.17),
                );
                path.cubic_bezier_to(
                    view.point(44.7801, 32.3209),
                    view.point(43.4085, 33.9767),
                    view.point(43.9997, 33.0778),
                );
                path.cubic_bezier_to(
                    view.point(45.5369, 31.4457),
                    view.point(45.2058, 31.9188),
                    view.point(45.5369, 31.5167),
                );
                path.cubic_bezier_to(
                    view.point(44.3781, 30.9963),
                    view.point(45.5369, 31.1382),
                    view.point(44.8747, 30.878),
                );
                path.cubic_bezier_to(
                    view.point(42.9828, 32.2263),
                    view.point(44.2362, 31.0436),
                    view.point(43.6687, 31.5167),
                );
                path.cubic_bezier_to(
                    view.point(40.0268, 34.095),
                    view.point(41.6822, 33.5509),
                    view.point(41.4221, 33.7165),
                );
                path.cubic_bezier_to(
                    view.point(37.2127, 34.3315),
                    view.point(39.2228, 34.3079),
                    view.point(39.0809, 34.3315),
                );
                path.cubic_bezier_to(
                    view.point(34.9661, 33.8821),
                    view.point(35.0843, 34.3315),
                    view.point(34.9661, 34.3079),
                );
                path.cubic_bezier_to(
                    view.point(35.0843, 33.4327),
                    view.point(34.9661, 33.7402),
                    view.point(35.0134, 33.5509),
                );
                path.cubic_bezier_to(
                    view.point(35.2026, 33.1488),
                    view.point(35.1553, 33.3144),
                    view.point(35.2026, 33.1961),
                );
                path.cubic_bezier_to(
                    view.point(34.233, 33.409),
                    view.point(35.2262, 33.0542),
                    view.point(34.7296, 33.1725),
                );
                path.cubic_bezier_to(
                    view.point(28.7702, 34.095),
                    view.point(31.9154, 34.4971),
                    view.point(30.7567, 34.639),
                );
                path.cubic_bezier_to(
                    view.point(27.4459, 33.3854),
                    view.point(27.6824, 33.7875),
                    view.point(27.4459, 33.6692),
                );
                path.cubic_bezier_to(
                    view.point(29.3378, 32.2026),
                    view.point(27.4459, 33.1252),
                    view.point(27.6824, 32.9596),
                );
                path.cubic_bezier_to(
                    view.point(31.3242, 31.2801),
                    view.point(30.0945, 31.8478),
                    view.point(30.9932, 31.422),
                );
                path.cubic_bezier_to(
                    view.point(31.0877, 30.2393),
                    view.point(32.2938, 30.8307),
                    view.point(32.2702, 30.6178),
                );
                path.cubic_bezier_to(
                    view.point(28.9831, 29.2695),
                    view.point(30.7094, 30.0974),
                    view.point(29.7634, 29.6716),
                );
                path.cubic_bezier_to(
                    view.point(26.5, 29.1985),
                    view.point(27.3277, 28.4416),
                    view.point(27.3986, 28.4416),
                );
                path.cubic_bezier_to(
                    view.point(23.1183, 29.8609),
                    view.point(25.3176, 30.1684),
                    view.point(24.7264, 30.2866),
                );
                path.cubic_bezier_to(
                    view.point(20.7771, 28.9857),
                    view.point(22.4325, 29.6953),
                    view.point(21.6284, 29.3878),
                );
                path.cubic_bezier_to(
                    view.point(18.9798, 27.7556),
                    view.point(19.2636, 28.2997),
                    view.point(18.9798, 28.1104),
                );
                path.cubic_bezier_to(
                    view.point(22.598, 26.5256),
                    view.point(18.9798, 27.2352),
                    view.point(19.5001, 27.0697),
                );
                path.cubic_bezier_to(
                    view.point(22.4561, 25.13),
                    view.point(24.2534, 26.2418),
                    view.point(24.1824, 25.6031),
                );
                path.cubic_bezier_to(
                    view.point(19.3582, 23.9946),
                    view.point(21.6284, 24.8935),
                    view.point(19.9494, 24.2785),
                );
                path.cubic_bezier_to(
                    view.point(16.2839, 20.1153),
                    view.point(18.1758, 23.4032),
                    view.point(16.8278, 21.7001),
                );
                path.cubic_bezier_to(
                    view.point(15.8582, 18.9799),
                    view.point(16.142, 19.7132),
                    view.point(15.9528, 19.2164),
                );
                path.cubic_bezier_to(
                    view.point(17.0407, 17.6789),
                    view.point(15.2197, 17.4897),
                    view.point(15.5981, 17.0639),
                );
                path.cubic_bezier_to(
                    view.point(18.0339, 18.081),
                    view.point(17.3954, 17.8445),
                    view.point(17.8447, 18.0101),
                );
                path.cubic_bezier_to(
                    view.point(21.0136, 19.5239),
                    view.point(18.5541, 18.2466),
                    view.point(20.3751, 19.1218),
                );
                path.cubic_bezier_to(
                    view.point(22.598, 18.4595),
                    view.point(22.4798, 20.4228),
                    view.point(22.4798, 20.4228),
                );
                path.cubic_bezier_to(
                    view.point(23.8514, 15.9048),
                    view.point(22.7162, 16.2833),
                    view.point(23.1419, 15.4081),
                );
                path.cubic_bezier_to(
                    view.point(24.8446, 17.0876),
                    view.point(23.946, 15.9758),
                    view.point(24.3953, 16.5198),
                );
                path.cubic_bezier_to(
                    view.point(26.7128, 18.6487),
                    view.point(25.7669, 18.2703),
                    view.point(25.8615, 18.3412),
                );
                path.cubic_bezier_to(
                    view.point(28.1317, 17.8681),
                    view.point(28.0371, 19.0982),
                    view.point(28.1554, 19.0509),
                );
                path.cubic_bezier_to(
                    view.point(31.5371, 19.5713),
                    view.point(28.0844, 16.2123),
                    view.point(28.652, 16.4962),
                );
                path.cubic_bezier_to(
                    view.point(32.7668, 20.5647),
                    view.point(32.1283, 20.1863),
                    view.point(32.3648, 20.3992),
                );
                path.cubic_bezier_to(
                    view.point(34.4695, 21.3453),
                    view.point(33.0506, 20.683),
                    view.point(33.8073, 21.0142),
                );
                path.cubic_bezier_to(
                    view.point(36.2194, 21.7948),
                    view.point(35.6519, 21.913),
                    view.point(36.1012, 22.0313),
                );
                path.cubic_bezier_to(
                    view.point(36.1485, 20.7303),
                    view.point(36.2431, 21.7238),
                    view.point(36.2194, 21.2507),
                );
                path.cubic_bezier_to(
                    view.point(36.6688, 17.6079),
                    view.point(36.0066, 19.4766),
                    view.point(36.1012, 18.8616),
                );
                path.cubic_bezier_to(
                    view.point(39.1282, 18.152),
                    view.point(37.662, 15.3844),
                    view.point(38.1113, 15.4791),
                );
                path.cubic_bezier_to(
                    view.point(39.1045, 22.3151),
                    view.point(39.8376, 19.9497),
                    view.point(39.8376, 20.8013),
                );
                path.cubic_bezier_to(
                    view.point(38.6316, 23.4742),
                    view.point(38.8444, 22.8829),
                    view.point(38.6316, 23.3796),
                );
                path.cubic_bezier_to(
                    view.point(39.3647, 24.1838),
                    view.point(38.6316, 23.7108),
                    view.point(38.9626, 24.0419),
                );
                path.cubic_bezier_to(
                    view.point(42.77, 25.6031),
                    view.point(40.4525, 24.5623),
                    view.point(41.8477, 25.1537),
                );
                path.cubic_bezier_to(
                    view.point(45.2767, 28.5362),
                    view.point(44.3308, 26.36),
                    view.point(44.7801, 26.8804),
                );
                path.cubic_bezier_to(
                    view.point(46.4828, 30.2866),
                    view.point(45.6551, 29.8609),
                    view.point(45.9389, 30.2866),
                );
                path.cubic_bezier_to(
                    view.point(46.9794, 29.9318),
                    view.point(46.7193, 30.2866),
                    view.point(46.9794, 30.0974),
                );
                path.cubic_bezier_to(
                    view.point(46.2463, 28.4653),
                    view.point(46.9794, 29.7426),
                    view.point(46.6956, 29.1749),
                );
                path.cubic_bezier_to(
                    view.point(45.4896, 25.4139),
                    view.point(45.5842, 27.4245),
                    view.point(45.5369, 27.1879),
                );
                path.cubic_bezier_to(
                    view.point(45.3477, 23.3086),
                    view.point(45.4659, 24.586),
                    view.point(45.395, 23.6398),
                );
                path.cubic_bezier_to(
                    view.point(45.7733, 22.2678),
                    view.point(45.2294, 22.4807),
                    view.point(45.324, 22.2678),
                );
                path.cubic_bezier_to(
                    view.point(48.4456, 24.2311),
                    view.point(46.3882, 22.2678),
                    view.point(47.8071, 23.3086),
                );
                path.cubic_bezier_to(
                    view.point(49.7463, 23.4979),
                    view.point(49.1314, 25.201),
                    view.point(49.4861, 24.9881),
                );
                path.cubic_bezier_to(
                    view.point(50.0537, 22.5517),
                    view.point(49.8409, 22.9538),
                    view.point(49.9355, 22.6936),
                );
                path.line_to(view.point(50.2429, 22.3861));
                path.line_to(view.point(50.574, 22.7646));
                path.cubic_bezier_to(
                    view.point(51.0469, 23.5688),
                    view.point(50.7395, 22.9538),
                    view.point(50.9523, 23.3323),
                );
                path.cubic_bezier_to(
                    view.point(51.4726, 24.7279),
                    view.point(51.1179, 23.8054),
                    view.point(51.3071, 24.3258),
                );
                path.cubic_bezier_to(
                    view.point(51.9929, 28.276),
                    view.point(51.851, 25.7214),
                    view.point(52.0401, 27.0697),
                );
                path.cubic_bezier_to(
                    view.point(50.9996, 31.1145),
                    view.point(51.9456, 29.3641),
                    view.point(51.8983, 29.4824),
                );
                path.cubic_bezier_to(
                    view.point(50.7158, 33.3144),
                    view.point(50.2902, 32.4155),
                    view.point(50.2665, 32.5811),
                );
                path.cubic_bezier_to(
                    view.point(51.0469, 36.2948),
                    view.point(51.3071, 34.2842),
                    view.point(51.4253, 35.4433),
                );
                path.cubic_bezier_to(
                    view.point(46.3882, 39.8903),
                    view.point(50.5267, 37.4302),
                    view.point(48.9422, 38.6603),
                );
                path.cubic_bezier_to(
                    view.point(40.4761, 42.0428),
                    view.point(43.5031, 41.2859),
                    view.point(42.368, 41.7117),
                );
                path.cubic_bezier_to(
                    view.point(38.7262, 42.3976),
                    view.point(39.9322, 42.1374),
                    view.point(39.1518, 42.303),
                );
                path.cubic_bezier_to(
                    view.point(37.1417, 42.6578),
                    view.point(38.3241, 42.4686),
                    view.point(37.591, 42.5869),
                );
                path.cubic_bezier_to(
                    view.point(34.9897, 42.9417),
                    view.point(36.6687, 42.7051),
                    view.point(35.6992, 42.8471),
                );
                path.cubic_bezier_to(
                    view.point(26.902, 43.2492),
                    view.point(32.7195, 43.3202),
                    view.point(29.5506, 43.4384),
                );
                path.close();
                path.move_to(view.point(35.912, 32.7703));
                path.cubic_bezier_to(
                    view.point(36.5742, 32.4628),
                    view.point(36.0302, 32.6757),
                    view.point(36.314, 32.5338),
                );
                path.cubic_bezier_to(
                    view.point(37.8512, 31.1382),
                    view.point(37.6383, 32.108),
                    view.point(38.5133, 31.2328),
                );
                path.cubic_bezier_to(
                    view.point(36.3613, 32.1553),
                    view.point(37.5674, 31.1145),
                    view.point(36.6924, 31.6822),
                );
                path.cubic_bezier_to(
                    view.point(35.9829, 32.5338),
                    view.point(36.2194, 32.3446),
                    view.point(36.0539, 32.5101),
                );
                path.cubic_bezier_to(
                    view.point(35.6046, 32.9123),
                    view.point(35.7938, 32.6048),
                    view.point(35.4863, 32.9123),
                );
                path.cubic_bezier_to(
                    view.point(35.912, 32.7703),
                    view.point(35.6755, 32.9123),
                    view.point(35.7938, 32.8413),
                );
                path.close();
                path.move_to(view.point(6.37527, 31.8715));
                path.cubic_bezier_to(
                    view.point(7.29755, 30.5705),
                    view.point(6.6354, 31.7296),
                    view.point(7.20296, 30.9253),
                );
                path.cubic_bezier_to(
                    view.point(5.64217, 30.7597),
                    view.point(7.48674, 29.8136),
                    view.point(5.97325, 29.9791),
                );
                path.cubic_bezier_to(
                    view.point(5.9023, 31.8478),
                    view.point(5.50028, 31.1382),
                    view.point(5.59487, 31.6586),
                );
                path.cubic_bezier_to(
                    view.point(6.37527, 31.8715),
                    view.point(6.11514, 32.0134),
                    view.point(6.09149, 32.0134),
                );
                path.close();
                path.move_to(view.point(9.02388, 30.6178));
                path.cubic_bezier_to(
                    view.point(9.3786, 29.6007),
                    view.point(9.18942, 30.4286),
                    view.point(9.3786, 29.9082),
                );
                path.cubic_bezier_to(
                    view.point(8.33808, 29.7426),
                    view.point(9.3786, 29.1985),
                    view.point(8.85834, 29.2695),
                );
                path.cubic_bezier_to(
                    view.point(8.24348, 30.5468),
                    view.point(7.93606, 30.0974),
                    view.point(7.93606, 30.2393),
                );
                path.cubic_bezier_to(
                    view.point(9.02388, 30.6178),
                    view.point(8.50362, 30.8307),
                    view.point(8.7874, 30.8543),
                );
                path.close();
                path.move_to(view.point(37.0471, 29.6007));
                path.cubic_bezier_to(
                    view.point(36.6451, 28.5599),
                    view.point(37.8512, 29.4587),
                    view.point(37.7329, 29.1985),
                );
                path.cubic_bezier_to(
                    view.point(34.8478, 27.0933),
                    view.point(36.0302, 28.2051),
                    view.point(35.6282, 27.8739),
                );
                path.cubic_bezier_to(
                    view.point(33.6181, 26.1708),
                    view.point(33.8546, 26.0998),
                    view.point(33.8073, 26.0525),
                );
                path.cubic_bezier_to(
                    view.point(33.358, 27.5901),
                    view.point(33.3816, 26.3364),
                    view.point(33.3107, 26.6439),
                );
                path.cubic_bezier_to(
                    view.point(32.6958, 28.2287),
                    view.point(33.3816, 28.6782),
                    view.point(33.358, 28.7018),
                );
                path.cubic_bezier_to(
                    view.point(31.0641, 27.1643),
                    view.point(31.5134, 27.3535),
                    view.point(31.2769, 27.2116),
                );
                path.cubic_bezier_to(
                    view.point(30.6148, 28.1578),
                    view.point(30.5911, 27.0697),
                    view.point(30.4256, 27.4481),
                );
                path.cubic_bezier_to(
                    view.point(32.4121, 29.5297),
                    view.point(30.7803, 28.7964),
                    view.point(31.4661, 29.3168),
                );
                path.cubic_bezier_to(
                    view.point(33.287, 29.3878),
                    view.point(32.7904, 29.6007),
                    view.point(32.8614, 29.6007),
                );
                path.cubic_bezier_to(
                    view.point(34.0674, 29.3168),
                    view.point(33.7364, 29.1749),
                    view.point(33.76, 29.1749),
                );
                path.cubic_bezier_to(
                    view.point(34.7059, 29.5534),
                    view.point(34.2566, 29.3878),
                    view.point(34.5404, 29.5061),
                );
                path.cubic_bezier_to(
                    view.point(37.0471, 29.6007),
                    view.point(35.037, 29.6953),
                    view.point(36.385, 29.7189),
                );
                path.close();
                path.move_to(view.point(28.9831, 27.4245));
                path.cubic_bezier_to(
                    view.point(28.2736, 26.4546),
                    view.point(28.9831, 27.2825),
                    view.point(28.7702, 26.975),
                );
                path.cubic_bezier_to(
                    view.point(27.1858, 25.2719),
                    view.point(27.8952, 26.0525),
                    view.point(27.3986, 25.5085),
                );
                path.cubic_bezier_to(
                    view.point(26.0507, 25.2719),
                    view.point(26.6655, 24.6806),
                    view.point(26.2635, 24.6806),
                );
                path.cubic_bezier_to(
                    view.point(27.8479, 27.1879),
                    view.point(25.9088, 25.6977),
                    view.point(26.5946, 26.431),
                );
                path.cubic_bezier_to(
                    view.point(28.9831, 27.4245),
                    view.point(28.6993, 27.6847),
                    view.point(28.9831, 27.7556),
                );
                path.close();
                path.move_to(view.point(24.7973, 24.6333));
                path.cubic_bezier_to(
                    view.point(24.1115, 23.0957),
                    view.point(25.0338, 24.3967),
                    view.point(24.6081, 23.4506),
                );
                path.cubic_bezier_to(
                    view.point(24.1824, 24.4677),
                    view.point(23.4021, 22.5517),
                    view.point(23.4493, 23.829),
                );
                path.cubic_bezier_to(
                    view.point(24.7973, 24.6333),
                    view.point(24.4899, 24.7515),
                    view.point(24.6318, 24.7988),
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
