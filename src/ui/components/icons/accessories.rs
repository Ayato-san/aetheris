use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct AccessoriesIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl AccessoriesIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for AccessoriesIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(18.2772, 51.9446));
                path.cubic_bezier_to(
                    view.point(13.7869, 48.588),
                    view.point(16.7727, 51.5974),
                    view.point(14.6665, 50.0233),
                );
                path.cubic_bezier_to(
                    view.point(13.0, 45.1851),
                    view.point(13.0694, 47.4306),
                    view.point(13.0, 47.1759),
                );
                path.cubic_bezier_to(
                    view.point(14.4813, 40.509),
                    view.point(13.0, 42.8471),
                    view.point(13.0463, 42.6619),
                );
                path.cubic_bezier_to(
                    view.point(14.9211, 39.6988),
                    view.point(14.6433, 40.2544),
                    view.point(14.8516, 39.9071),
                );
                path.cubic_bezier_to(
                    view.point(15.4534, 38.796),
                    view.point(15.0137, 39.4904),
                    view.point(15.2451, 39.0969),
                );
                path.cubic_bezier_to(
                    view.point(15.8237, 38.1478),
                    view.point(15.6617, 38.5182),
                    view.point(15.8237, 38.2173),
                );
                path.cubic_bezier_to(
                    view.point(16.1709, 37.4996),
                    view.point(15.8237, 38.1015),
                    view.point(15.9858, 37.8006),
                );
                path.cubic_bezier_to(
                    view.point(16.7264, 36.3653),
                    view.point(16.3792, 37.1755),
                    view.point(16.6107, 36.6894),
                );
                path.cubic_bezier_to(
                    view.point(17.1662, 35.3468),
                    view.point(16.819, 36.0644),
                    view.point(17.0273, 35.6014),
                );
                path.cubic_bezier_to(
                    view.point(17.6059, 34.0504),
                    view.point(17.305, 35.069),
                    view.point(17.4902, 34.4903),
                );
                path.cubic_bezier_to(
                    view.point(18.0457, 32.7772),
                    view.point(17.7217, 33.5875),
                    view.point(17.93, 33.0319),
                );
                path.cubic_bezier_to(
                    view.point(18.4623, 31.2957),
                    view.point(18.1846, 32.5226),
                    view.point(18.3697, 31.8513),
                );
                path.cubic_bezier_to(
                    view.point(20.3371, 29.5595),
                    view.point(18.8095, 29.328),
                    view.point(19.4807, 28.703),
                );
                path.cubic_bezier_to(
                    view.point(21.0546, 32.1522),
                    view.point(20.8, 30.0225),
                    view.point(20.9157, 30.4392),
                );
                path.cubic_bezier_to(
                    view.point(21.5638, 36.1107),
                    view.point(21.2166, 34.0736),
                    view.point(21.4018, 35.3931),
                );
                path.cubic_bezier_to(
                    view.point(21.8415, 37.7543),
                    view.point(21.6564, 36.4348),
                    view.point(21.7721, 37.1755),
                );
                path.cubic_bezier_to(
                    view.point(22.3507, 39.884),
                    view.point(21.911, 38.3098),
                    view.point(22.1424, 39.2821),
                );
                path.cubic_bezier_to(
                    view.point(22.8137, 46.5278),
                    view.point(23.022, 41.8517),
                    view.point(23.2303, 45.0925),
                );
                path.cubic_bezier_to(
                    view.point(19.8279, 50.0233),
                    view.point(22.5128, 47.5),
                    view.point(20.3371, 50.0233),
                );
                path.cubic_bezier_to(
                    view.point(19.7585, 50.0927),
                    view.point(19.7585, 50.0233),
                    view.point(19.7122, 50.0695),
                );
                path.cubic_bezier_to(
                    view.point(21.8415, 49.6992),
                    view.point(19.8742, 50.2084),
                    view.point(21.4249, 49.9075),
                );
                path.cubic_bezier_to(
                    view.point(22.7442, 49.3751),
                    view.point(22.073, 49.5834),
                    view.point(22.4665, 49.4445),
                );
                path.cubic_bezier_to(
                    view.point(23.6006, 49.0047),
                    view.point(23.022, 49.2825),
                    view.point(23.3923, 49.1436),
                );
                path.cubic_bezier_to(
                    view.point(24.5496, 49.213),
                    view.point(24.0172, 48.7501),
                    view.point(24.3644, 48.8195),
                );
                path.cubic_bezier_to(
                    view.point(24.2255, 50.1158),
                    view.point(24.6653, 49.4445),
                    view.point(24.619, 49.5603),
                );
                path.cubic_bezier_to(
                    view.point(22.5822, 51.3659),
                    view.point(23.832, 50.6946),
                    view.point(23.6932, 50.8103),
                );
                path.line_to(view.point(21.3555, 51.9678));
                path.line_to(view.point(20.0131, 51.9909));
                path.cubic_bezier_to(
                    view.point(18.2772, 51.9446),
                    view.point(19.2724, 52.0141),
                    view.point(18.4855, 51.9909),
                );
                path.close();
                path.move_to(view.point(31.7709, 44.236));
                path.cubic_bezier_to(
                    view.point(29.9887, 42.685),
                    view.point(30.4979, 43.8425),
                    view.point(29.9887, 43.3795),
                );
                path.cubic_bezier_to(
                    view.point(29.0398, 40.7868),
                    view.point(29.9887, 42.4072),
                    view.point(29.7573, 41.9442),
                );
                path.cubic_bezier_to(
                    view.point(27.0956, 37.1524),
                    view.point(27.9288, 38.9812),
                    view.point(27.2576, 37.7311),
                );
                path.cubic_bezier_to(
                    view.point(27.7205, 34.5829),
                    view.point(26.9104, 36.5968),
                    view.point(27.2113, 35.3931),
                );
                path.cubic_bezier_to(
                    view.point(30.9377, 28.5178),
                    view.point(29.3638, 31.9439),
                    view.point(30.7062, 29.4206),
                );
                path.cubic_bezier_to(
                    view.point(31.3774, 27.2909),
                    view.point(31.0303, 28.1937),
                    view.point(31.2386, 27.6382),
                );
                path.cubic_bezier_to(
                    view.point(31.7709, 26.1335),
                    view.point(31.5163, 26.9668),
                    view.point(31.7015, 26.4344),
                );
                path.cubic_bezier_to(
                    view.point(33.1596, 22.337),
                    view.point(32.1644, 24.189),
                    view.point(32.743, 22.5917),
                );
                path.cubic_bezier_to(
                    view.point(34.4558, 23.7954),
                    view.point(33.7846, 21.9203),
                    view.point(34.2706, 22.4759),
                );
                path.cubic_bezier_to(
                    view.point(34.8493, 25.3233),
                    view.point(34.5021, 24.2815),
                    view.point(34.6872, 24.976),
                );
                path.cubic_bezier_to(
                    view.point(35.2659, 26.7353),
                    view.point(34.9881, 25.6705),
                    view.point(35.1733, 26.3187),
                );
                path.cubic_bezier_to(
                    view.point(35.7056, 28.0548),
                    view.point(35.3585, 27.1752),
                    view.point(35.5668, 27.7539),
                );
                path.cubic_bezier_to(
                    view.point(36.0991, 29.0734),
                    view.point(35.8445, 28.3558),
                    view.point(36.0297, 28.7956),
                );
                path.cubic_bezier_to(
                    view.point(37.5804, 31.9902),
                    view.point(36.2148, 29.4901),
                    view.point(36.8629, 30.7864),
                );
                path.cubic_bezier_to(
                    view.point(38.2516, 33.1476),
                    view.point(37.6961, 32.1985),
                    view.point(38.0202, 32.7309),
                );
                path.cubic_bezier_to(
                    view.point(39.1312, 34.606),
                    view.point(38.5062, 33.5875),
                    view.point(38.8997, 34.2356),
                );
                path.cubic_bezier_to(
                    view.point(39.8024, 39.6988),
                    view.point(39.8487, 35.7403),
                    view.point(40.1496, 38.0552),
                );
                path.cubic_bezier_to(
                    view.point(35.5668, 44.0971),
                    view.point(39.4089, 41.435),
                    view.point(37.4415, 43.4952),
                );
                path.cubic_bezier_to(
                    view.point(31.7709, 44.236),
                    view.point(34.3632, 44.4906),
                    view.point(32.743, 44.5369),
                );
                path.close();
                path.move_to(view.point(25.2671, 44.1203));
                path.cubic_bezier_to(
                    view.point(24.9662, 42.222),
                    view.point(24.7579, 43.8193),
                    view.point(24.5496, 42.6387),
                );
                path.cubic_bezier_to(
                    view.point(26.1466, 42.4535),
                    view.point(25.3365, 41.8517),
                    view.point(25.8688, 41.9674),
                );
                path.cubic_bezier_to(
                    view.point(25.2671, 44.1203),
                    view.point(26.6095, 43.3795),
                    view.point(26.0077, 44.5369),
                );
                path.close();
                path.move_to(view.point(35.3353, 42.0368));
                path.cubic_bezier_to(
                    view.point(37.6961, 39.7682),
                    view.point(36.0297, 41.7128),
                    view.point(37.3953, 40.3933),
                );
                path.cubic_bezier_to(
                    view.point(37.4415, 36.9672),
                    view.point(37.997, 39.1201),
                    view.point(37.9276, 38.4487),
                );
                path.cubic_bezier_to(
                    view.point(36.7703, 35.2079),
                    view.point(37.1869, 36.2264),
                    view.point(36.8861, 35.4394),
                );
                path.cubic_bezier_to(
                    view.point(36.3306, 34.1662),
                    view.point(36.6546, 34.9764),
                    view.point(36.4463, 34.4903),
                );
                path.cubic_bezier_to(
                    view.point(36.0065, 34.0736),
                    view.point(36.0065, 33.3328),
                    view.point(35.9139, 33.3097),
                );
                path.cubic_bezier_to(
                    view.point(35.9371, 37.8932),
                    view.point(36.1454, 35.1847),
                    view.point(36.1223, 37.4533),
                );
                path.cubic_bezier_to(
                    view.point(34.2012, 40.9025),
                    view.point(35.1964, 39.8377),
                    view.point(35.0576, 40.046),
                );
                path.cubic_bezier_to(
                    view.point(32.9976, 41.9442),
                    view.point(33.7383, 41.3887),
                    view.point(33.2059, 41.8516),
                );
                path.cubic_bezier_to(
                    view.point(33.9466, 42.3841),
                    view.point(32.3264, 42.2915),
                    view.point(32.6736, 42.4767),
                );
                path.cubic_bezier_to(
                    view.point(35.3353, 42.0368),
                    view.point(34.4789, 42.3609),
                    view.point(34.9187, 42.2452),
                );
                path.close();
                path.move_to(view.point(19.041, 26.7816));
                path.cubic_bezier_to(
                    view.point(18.7632, 24.7908),
                    view.point(18.3003, 26.3881),
                    view.point(18.1846, 25.3696),
                );
                path.cubic_bezier_to(
                    view.point(20.8926, 26.2955),
                    view.point(19.8742, 23.6797),
                    view.point(21.6101, 24.9297),
                );
                path.cubic_bezier_to(
                    view.point(19.041, 26.7816),
                    view.point(20.5917, 26.9205),
                    view.point(19.689, 27.152),
                );
                path.close();
                path.move_to(view.point(19.0872, 23.3556));
                path.cubic_bezier_to(
                    view.point(18.7632, 21.3185),
                    view.point(18.3234, 23.0084),
                    view.point(18.1614, 21.9435),
                );
                path.cubic_bezier_to(
                    view.point(21.0777, 22.1055),
                    view.point(19.5733, 20.5314),
                    view.point(21.0777, 21.0407),
                );
                path.cubic_bezier_to(
                    view.point(19.0872, 23.3556),
                    view.point(21.0777, 23.1935),
                    view.point(20.1056, 23.7954),
                );
                path.close();
                path.move_to(view.point(32.5347, 20.1842));
                path.cubic_bezier_to(
                    view.point(33.0208, 18.1008),
                    view.point(31.7941, 19.4434),
                    view.point(32.0487, 18.3554),
                );
                path.cubic_bezier_to(
                    view.point(34.6641, 19.2814),
                    view.point(33.7151, 17.9156),
                    view.point(34.6641, 18.61),
                );
                path.cubic_bezier_to(
                    view.point(32.5347, 20.1842),
                    view.point(34.6641, 20.4851),
                    view.point(33.3911, 21.0407),
                );
                path.close();
                path.move_to(view.point(19.041, 19.8832));
                path.cubic_bezier_to(
                    view.point(19.3418, 17.4989),
                    view.point(18.0688, 19.3508),
                    view.point(18.2772, 17.8693),
                );
                path.cubic_bezier_to(
                    view.point(20.8926, 19.3971),
                    view.point(20.4297, 17.1285),
                    view.point(21.4481, 18.3323),
                );
                path.cubic_bezier_to(
                    view.point(19.041, 19.8832),
                    view.point(20.5917, 19.999),
                    view.point(19.689, 20.2305),
                );
                path.close();
                path.move_to(view.point(32.8125, 17.1054));
                path.cubic_bezier_to(
                    view.point(33.7846, 14.8136),
                    view.point(31.3774, 16.4803),
                    view.point(32.2801, 14.3506),
                );
                path.cubic_bezier_to(
                    view.point(33.9234, 17.1054),
                    view.point(34.8724, 15.1377),
                    view.point(34.9881, 16.6655),
                );
                path.cubic_bezier_to(
                    view.point(32.8125, 17.1054),
                    view.point(33.4605, 17.3137),
                    view.point(33.2522, 17.3137),
                );
                path.close();
                path.move_to(view.point(19.041, 16.5729));
                path.cubic_bezier_to(
                    view.point(19.0641, 14.2812),
                    view.point(18.2309, 16.0173),
                    view.point(18.2309, 14.8367),
                );
                path.cubic_bezier_to(
                    view.point(20.9389, 15.8553),
                    view.point(20.0825, 13.6098),
                    view.point(21.3555, 14.6747),
                );
                path.cubic_bezier_to(
                    view.point(19.7353, 16.8276),
                    view.point(20.7769, 16.3877),
                    view.point(20.1982, 16.8276),
                );
                path.cubic_bezier_to(
                    view.point(19.041, 16.5729),
                    view.point(19.5733, 16.8276),
                    view.point(19.2493, 16.7118),
                );
                path.close();
                path.move_to(view.point(32.9282, 13.9802));
                path.cubic_bezier_to(
                    view.point(33.0902, 11.5727),
                    view.point(31.7478, 13.4941),
                    view.point(31.8635, 11.9431),
                );
                path.cubic_bezier_to(
                    view.point(34.4326, 13.4478),
                    view.point(34.1318, 11.2718),
                    view.point(35.0113, 12.4987),
                );
                path.cubic_bezier_to(
                    view.point(32.9282, 13.9802),
                    view.point(34.0623, 14.0497),
                    view.point(33.53, 14.258),
                );
                path.close();
                path.move_to(view.point(17.4902, 11.4107));
                path.cubic_bezier_to(
                    view.point(17.2588, 7.19757),
                    view.point(15.2451, 9.16524),
                    view.point(15.2451, 9.23468),
                );
                path.cubic_bezier_to(
                    view.point(21.8878, 7.12812),
                    view.point(19.5501, 4.88267),
                    view.point(19.6196, 4.88267),
                );
                path.cubic_bezier_to(
                    view.point(23.4617, 9.32728),
                    view.point(23.2997, 8.51707),
                    view.point(23.5774, 8.88745),
                );
                path.cubic_bezier_to(
                    view.point(21.6795, 9.9523),
                    view.point(23.346, 9.83656),
                    view.point(22.8137, 10.0218),
                );
                path.cubic_bezier_to(
                    view.point(20.962, 10.2995),
                    view.point(20.5223, 9.88286),
                    view.point(20.2677, 9.9986),
                );
                path.cubic_bezier_to(
                    view.point(21.1009, 12.0357),
                    view.point(21.8184, 10.6931),
                    view.point(21.8415, 11.2486),
                );
                path.cubic_bezier_to(
                    view.point(19.5039, 12.9154),
                    view.point(20.4528, 12.7302),
                    view.point(20.1056, 12.9154),
                );
                path.cubic_bezier_to(
                    view.point(17.4902, 11.4107),
                    view.point(19.0409, 12.9154),
                    view.point(18.9715, 12.8691),
                );
                path.close();
                path.move_to(view.point(32.7893, 10.7162));
                path.cubic_bezier_to(
                    view.point(32.7662, 8.51707),
                    view.point(31.8635, 10.2301),
                    view.point(31.8635, 9.00319),
                );
                path.cubic_bezier_to(
                    view.point(34.6641, 9.55877),
                    view.point(33.6226, 8.07723),
                    view.point(34.6641, 8.65596),
                );
                path.cubic_bezier_to(
                    view.point(32.7893, 10.7162),
                    view.point(34.6641, 10.6005),
                    view.point(33.692, 11.2024),
                );
                path.close();
                path.move_to(view.point(32.8125, 7.22072));
                path.cubic_bezier_to(
                    view.point(29.6878, 3.93356),
                    view.point(32.419, 7.01238),
                    view.point(29.7573, 4.21135),
                );
                path.cubic_bezier_to(
                    view.point(31.0303, 1.80385),
                    view.point(29.5721, 3.47058),
                    view.point(29.873, 2.98445),
                );
                path.cubic_bezier_to(
                    view.point(34.965, 1.03993),
                    view.point(33.1828, -0.441605),
                    view.point(33.3911, -0.464754),
                );
                path.cubic_bezier_to(
                    view.point(34.2243, 3.37798),
                    view.point(36.4926, 2.47517),
                    view.point(36.1454, 3.56317),
                );
                path.cubic_bezier_to(
                    view.point(33.5763, 3.49373),
                    view.point(33.5763, 3.30854),
                    view.point(33.5068, 3.33168),
                );
                path.cubic_bezier_to(
                    view.point(33.6688, 3.74837),
                    view.point(33.6226, 3.60947),
                    view.point(33.6688, 3.70207),
                );
                path.cubic_bezier_to(
                    view.point(34.3169, 4.11875),
                    view.point(33.6688, 3.77152),
                    view.point(33.9466, 3.93356),
                );
                path.cubic_bezier_to(
                    view.point(34.5715, 6.57255),
                    view.point(35.5899, 4.76692),
                    view.point(35.6593, 5.48454),
                );
                path.cubic_bezier_to(
                    view.point(32.8125, 7.22072),
                    view.point(33.7846, 7.35961),
                    view.point(33.3448, 7.52166),
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
