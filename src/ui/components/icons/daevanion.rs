use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct DaevanionIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl DaevanionIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for DaevanionIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(23.9027, 51.952));
                path.cubic_bezier_to(
                    view.point(17.3031, 50.7215),
                    view.point(23.4532, 51.881),
                    view.point(21.9393, 51.5971),
                );
                path.cubic_bezier_to(
                    view.point(4.19864, 50.2719),
                    view.point(13.8969, 50.059),
                    view.point(10.1359, 49.9407),
                );
                path.line_to(view.point(2.75572, 50.3666));
                path.line_to(view.point(2.28264, 49.9407));
                path.cubic_bezier_to(
                    view.point(1.66763, 47.5744),
                    view.point(1.64397, 49.3728),
                    view.point(1.50205, 48.8049),
                );
                path.cubic_bezier_to(
                    view.point(1.80955, 38.7954),
                    view.point(1.88052, 45.9416),
                    view.point(1.97513, 40.5938),
                );
                path.cubic_bezier_to(
                    view.point(0.532221, 30.963),
                    view.point(1.64397, 36.9734),
                    view.point(1.2655, 34.5834),
                );
                path.cubic_bezier_to(
                    view.point(0.532221, 21.0245),
                    view.point(-0.177407, 27.3662),
                    view.point(-0.177407, 24.6923),
                );
                path.cubic_bezier_to(
                    view.point(1.80955, 12.3876),
                    view.point(1.64397, 15.3454),
                    view.point(1.66763, 15.1088),
                );
                path.cubic_bezier_to(
                    view.point(1.7859, 6.96874),
                    view.point(1.92782, 10.3289),
                    view.point(1.92782, 9.07474),
                );
                path.cubic_bezier_to(
                    view.point(1.85686, 2.49644),
                    view.point(1.54936, 3.11168),
                    view.point(1.54936, 2.92237),
                );
                path.cubic_bezier_to(
                    view.point(5.49962, 1.76289),
                    view.point(2.54284, 1.59725),
                    view.point(2.54284, 1.59725),
                );
                path.cubic_bezier_to(
                    view.point(13.4002, 1.78655),
                    view.point(8.64564, 1.95219),
                    view.point(10.8691, 1.97586),
                );
                path.cubic_bezier_to(
                    view.point(21.4663, 0.461428),
                    view.point(15.1506, 1.66824),
                    view.point(16.5462, 1.45527),
                );
                path.cubic_bezier_to(
                    view.point(30.5495, 0.461428),
                    view.point(24.5413, -0.153809),
                    view.point(27.4508, -0.153809),
                );
                path.cubic_bezier_to(
                    view.point(37.0308, 1.66824),
                    view.point(34.2869, 1.21864),
                    view.point(35.9427, 1.52626),
                );
                path.cubic_bezier_to(
                    view.point(45.5227, 1.78655),
                    view.point(38.9231, 1.92853),
                    view.point(42.897, 1.97586),
                );
                path.cubic_bezier_to(
                    view.point(49.402, 1.8812),
                    view.point(48.5741, 1.57359),
                    view.point(48.8816, 1.57359),
                );
                path.cubic_bezier_to(
                    view.point(50.1826, 6.89775),
                    view.point(50.4428, 2.47278),
                    view.point(50.4428, 2.59109),
                );
                path.cubic_bezier_to(
                    view.point(50.1589, 12.3876),
                    view.point(50.0406, 9.09841),
                    view.point(50.0406, 10.3052),
                );
                path.cubic_bezier_to(
                    view.point(51.4362, 21.0245),
                    view.point(50.3008, 15.1088),
                    view.point(50.3245, 15.3691),
                );
                path.cubic_bezier_to(
                    view.point(51.5072, 30.5607),
                    view.point(52.1222, 24.5503),
                    view.point(52.1459, 27.4135),
                );
                path.cubic_bezier_to(
                    view.point(50.1589, 38.7718),
                    view.point(50.7503, 34.2995),
                    view.point(50.3245, 36.9497),
                );
                path.cubic_bezier_to(
                    view.point(50.3008, 47.5507),
                    view.point(49.9933, 40.6648),
                    view.point(50.088, 45.5394),
                );
                path.cubic_bezier_to(
                    view.point(49.6149, 49.9643),
                    view.point(50.4428, 48.8995),
                    view.point(50.3008, 49.4201),
                );
                path.cubic_bezier_to(
                    view.point(45.5227, 50.201),
                    view.point(49.0472, 50.4376),
                    view.point(48.9052, 50.4376),
                );
                path.cubic_bezier_to(
                    view.point(30.171, 51.5971),
                    view.point(40.366, 49.8224),
                    view.point(37.5512, 50.0826),
                );
                path.cubic_bezier_to(
                    view.point(23.9027, 51.952),
                    view.point(28.5625, 51.9047),
                    view.point(25.18, 52.094),
                );
                path.close();
                path.move_to(view.point(30.5495, 48.166));
                path.cubic_bezier_to(
                    view.point(32.8676, 47.6927),
                    view.point(31.3064, 48.0476),
                    view.point(32.3472, 47.8347),
                );
                path.cubic_bezier_to(
                    view.point(35.7061, 47.1248),
                    view.point(33.3644, 47.5507),
                    view.point(34.6417, 47.2904),
                );
                path.cubic_bezier_to(
                    view.point(38.45, 46.1309),
                    view.point(37.8114, 46.7698),
                    view.point(37.9533, 46.7225),
                );
                path.cubic_bezier_to(
                    view.point(36.7706, 45.1608),
                    view.point(39.0887, 45.3974),
                    view.point(38.6629, 45.1371),
                );
                path.cubic_bezier_to(
                    view.point(34.145, 45.7287),
                    view.point(35.7298, 45.1844),
                    view.point(35.4459, 45.2554),
                );
                path.cubic_bezier_to(
                    view.point(25.795, 47.4797),
                    view.point(31.3064, 46.8408),
                    view.point(28.3024, 47.4561),
                );
                path.cubic_bezier_to(
                    view.point(17.6106, 45.6577),
                    view.point(23.5242, 47.5034),
                    view.point(20.6384, 46.8645),
                );
                path.cubic_bezier_to(
                    view.point(14.985, 45.1608),
                    view.point(16.4752, 45.2081),
                    view.point(16.2623, 45.1608),
                );
                path.cubic_bezier_to(
                    view.point(13.5657, 46.1783),
                    view.point(13.14, 45.1608),
                    view.point(12.8324, 45.3501),
                );
                path.cubic_bezier_to(
                    view.point(15.6473, 46.9828),
                    view.point(13.9679, 46.6515),
                    view.point(14.4882, 46.8408),
                );
                path.cubic_bezier_to(
                    view.point(19.5976, 47.7637),
                    view.point(16.8064, 47.1248),
                    view.point(18.2256, 47.4087),
                );
                path.cubic_bezier_to(
                    view.point(22.9328, 48.2369),
                    view.point(20.4491, 47.9766),
                    view.point(21.5845, 48.1423),
                );
                path.cubic_bezier_to(
                    view.point(25.0617, 48.4499),
                    view.point(24.0682, 48.3316),
                    view.point(25.0381, 48.4262),
                );
                path.cubic_bezier_to(
                    view.point(30.5495, 48.166),
                    view.point(25.2036, 48.5919),
                    view.point(29.2012, 48.3789),
                );
                path.close();
                path.move_to(view.point(46.3979, 46.3912));
                path.cubic_bezier_to(
                    view.point(46.3033, 41.777),
                    view.point(46.5162, 46.1783),
                    view.point(46.4452, 42.1319),
                );
                path.cubic_bezier_to(
                    view.point(45.1442, 42.3685),
                    view.point(46.114, 41.3037),
                    view.point(45.2861, 41.706),
                );
                path.cubic_bezier_to(
                    view.point(45.0496, 43.6227),
                    view.point(45.1205, 42.5342),
                    view.point(45.0732, 43.1021),
                );
                path.cubic_bezier_to(
                    view.point(43.2755, 45.3501),
                    view.point(44.9786, 44.9715),
                    view.point(44.5765, 45.3501),
                );
                path.cubic_bezier_to(
                    view.point(41.5488, 46.0599),
                    view.point(42.3293, 45.3501),
                    view.point(41.5488, 45.6813),
                );
                path.cubic_bezier_to(
                    view.point(42.8497, 46.4385),
                    view.point(41.5488, 46.3676),
                    view.point(41.8089, 46.4622),
                );
                path.cubic_bezier_to(
                    view.point(44.4819, 46.5332),
                    view.point(43.3938, 46.4149),
                    view.point(44.1271, 46.4622),
                );
                path.cubic_bezier_to(
                    view.point(46.3979, 46.3912),
                    view.point(45.3334, 46.6752),
                    view.point(46.256, 46.6042),
                );
                path.close();
                path.move_to(view.point(8.7166, 46.3202));
                path.cubic_bezier_to(
                    view.point(9.92297, 45.7287),
                    view.point(10.1359, 46.2256),
                    view.point(10.3724, 46.1073),
                );
                path.cubic_bezier_to(
                    view.point(8.85853, 45.492),
                    view.point(9.68643, 45.563),
                    view.point(9.40258, 45.492),
                );
                path.cubic_bezier_to(
                    view.point(7.15542, 43.4334),
                    view.point(7.65216, 45.492),
                    view.point(7.15542, 44.9005),
                );
                path.cubic_bezier_to(
                    view.point(5.75982, 41.5876),
                    view.point(7.15542, 42.1556),
                    view.point(6.30387, 41.0434),
                );
                path.cubic_bezier_to(
                    view.point(5.47597, 43.9539),
                    view.point(5.54693, 41.8006),
                    view.point(5.52327, 42.1319),
                );
                path.cubic_bezier_to(
                    view.point(6.68233, 46.4859),
                    view.point(5.42866, 46.5569),
                    view.point(5.45231, 46.6042),
                );
                path.cubic_bezier_to(
                    view.point(8.7166, 46.3202),
                    view.point(7.13177, 46.4386),
                    view.point(8.03063, 46.3676),
                );
                path.close();
                path.move_to(view.point(30.0055, 45.0424));
                path.cubic_bezier_to(
                    view.point(45.3807, 25.9938),
                    view.point(39.0177, 43.2204),
                    view.point(45.3807, 35.3406),
                );
                path.cubic_bezier_to(
                    view.point(22.034, 6.92141),
                    view.point(45.3807, 13.547),
                    view.point(34.2396, 4.46047),
                );
                path.cubic_bezier_to(
                    view.point(8.95315, 16.4812),
                    view.point(16.4989, 8.05724),
                    view.point(11.697, 11.5594),
                );
                path.cubic_bezier_to(
                    view.point(8.95315, 35.4826),
                    view.point(5.73616, 22.2787),
                    view.point(5.71251, 29.7325),
                );
                path.cubic_bezier_to(
                    view.point(23.95, 45.3974),
                    view.point(12.0755, 41.0671),
                    view.point(17.0666, 44.3562),
                );
                path.cubic_bezier_to(
                    view.point(30.0055, 45.0424),
                    view.point(25.0854, 45.563),
                    view.point(28.326, 45.3737),
                );
                path.close();
                path.move_to(view.point(24.0209, 41.8716));
                path.cubic_bezier_to(
                    view.point(17.8708, 38.7718),
                    view.point(21.2297, 41.4457),
                    view.point(18.9589, 40.2862),
                );
                path.cubic_bezier_to(
                    view.point(19.361, 38.7718),
                    view.point(17.1612, 37.7779),
                    view.point(17.3977, 37.7779),
                );
                path.cubic_bezier_to(
                    view.point(25.7004, 40.3335),
                    view.point(21.892, 40.0259),
                    view.point(23.7134, 40.4991),
                );
                path.cubic_bezier_to(
                    view.point(30.0055, 39.0557),
                    view.point(26.9541, 40.2389),
                    view.point(29.0593, 39.6236),
                );
                path.cubic_bezier_to(
                    view.point(32.8203, 36.1688),
                    view.point(30.8334, 38.5588),
                    view.point(32.3472, 37.0207),
                );
                path.cubic_bezier_to(
                    view.point(31.472, 27.8158),
                    view.point(34.4761, 33.3293),
                    view.point(33.6009, 27.8158),
                );
                path.cubic_bezier_to(
                    view.point(26.1498, 32.3118),
                    view.point(30.7151, 27.8158),
                    view.point(29.8635, 28.5494),
                );
                path.cubic_bezier_to(
                    view.point(22.6017, 35.7192),
                    view.point(24.3284, 34.1338),
                    view.point(22.7436, 35.6719),
                );
                path.cubic_bezier_to(
                    view.point(21.6082, 34.6544),
                    view.point(22.1759, 35.8139),
                    view.point(21.8211, 35.4589),
                );
                path.cubic_bezier_to(
                    view.point(17.4923, 27.7212),
                    view.point(21.0168, 32.6667),
                    view.point(19.5739, 30.2058),
                );
                path.cubic_bezier_to(
                    view.point(15.0086, 24.1007),
                    view.point(15.8129, 25.6861),
                    view.point(15.4581, 25.1656),
                );
                path.cubic_bezier_to(
                    view.point(16.0494, 14.9432),
                    view.point(13.8969, 21.4031),
                    view.point(14.3463, 17.4278),
                );
                path.cubic_bezier_to(
                    view.point(22.5307, 10.4235),
                    view.point(17.3031, 13.1211),
                    view.point(20.2126, 11.0861),
                );
                path.cubic_bezier_to(
                    view.point(27.8056, 10.2816),
                    view.point(23.3113, 10.2106),
                    view.point(26.6702, 10.1159),
                );
                path.cubic_bezier_to(
                    view.point(33.9557, 13.618),
                    view.point(30.4549, 10.7075),
                    view.point(33.1751, 12.1746),
                );
                path.cubic_bezier_to(
                    view.point(33.5063, 14.0203),
                    view.point(34.2159, 14.0913),
                    view.point(34.003, 14.2806),
                );
                path.cubic_bezier_to(
                    view.point(24.0919, 12.1746),
                    view.point(29.2958, 11.796),
                    view.point(27.0723, 11.3701),
                );
                path.cubic_bezier_to(
                    view.point(18.0127, 20.4803),
                    view.point(20.0707, 13.2631),
                    view.point(17.4923, 16.7889),
                );
                path.cubic_bezier_to(
                    view.point(20.1889, 24.432),
                    view.point(18.3439, 22.9412),
                    view.point(19.1718, 24.432),
                );
                path.cubic_bezier_to(
                    view.point(25.3219, 20.1727),
                    view.point(20.9459, 24.432),
                    view.point(21.419, 24.0534),
                );
                path.cubic_bezier_to(
                    view.point(29.0829, 16.5996),
                    view.point(27.2379, 18.2796),
                    view.point(28.9174, 16.6706),
                );
                path.cubic_bezier_to(
                    view.point(30.242, 18.0193),
                    view.point(29.6506, 16.2919),
                    view.point(29.9818, 16.6942),
                );
                path.cubic_bezier_to(
                    view.point(34.4525, 24.7869),
                    view.point(30.5022, 19.3681),
                    view.point(32.229, 22.1367),
                );
                path.cubic_bezier_to(
                    view.point(36.7469, 28.2417),
                    view.point(36.1083, 26.7746),
                    view.point(36.2029, 26.9166),
                );
                path.cubic_bezier_to(
                    view.point(36.7706, 34.962),
                    view.point(37.6221, 30.3477),
                    view.point(37.6221, 32.5957),
                );
                path.cubic_bezier_to(
                    view.point(34.618, 38.4168),
                    view.point(36.1319, 36.6894),
                    view.point(35.8008, 37.2337),
                );
                path.cubic_bezier_to(
                    view.point(24.0209, 41.8716),
                    view.point(31.7086, 41.351),
                    view.point(28.1841, 42.5105),
                );
                path.close();
                path.move_to(view.point(7.01349, 39.1267));
                path.cubic_bezier_to(
                    view.point(6.1856, 33.9918),
                    view.point(7.43927, 38.1802),
                    view.point(7.10811, 36.1925),
                );
                path.cubic_bezier_to(
                    view.point(4.45883, 25.6625),
                    view.point(5.12115, 31.4836),
                    view.point(4.34056, 27.7921),
                );
                path.cubic_bezier_to(
                    view.point(6.32752, 17.6407),
                    view.point(4.5771, 23.0832),
                    view.point(5.28673, 20.078),
                );
                path.cubic_bezier_to(
                    view.point(7.15542, 14.0676),
                    view.point(6.98984, 16.1026),
                    view.point(7.15542, 15.3691),
                );
                path.cubic_bezier_to(
                    view.point(4.93192, 15.937),
                    view.point(7.15542, 11.0624),
                    view.point(5.47597, 12.4585),
                );
                path.cubic_bezier_to(
                    view.point(4.2696, 19.2025),
                    view.point(4.78999, 16.8362),
                    view.point(4.48249, 18.3033),
                );
                path.cubic_bezier_to(
                    view.point(4.29325, 32.643),
                    view.point(3.25246, 23.5565),
                    view.point(3.27612, 28.715),
                );
                path.cubic_bezier_to(
                    view.point(5.05019, 36.6184),
                    view.point(4.60076, 33.7789),
                    view.point(4.86096, 35.1986),
                );
                path.cubic_bezier_to(
                    view.point(7.01349, 39.1267),
                    view.point(5.33404, 38.6534),
                    view.point(6.51676, 40.1915),
                );
                path.close();
                path.move_to(view.point(46.1377, 38.5351));
                path.cubic_bezier_to(
                    view.point(46.9656, 36.3818),
                    view.point(46.5398, 38.1329),
                    view.point(46.871, 37.281),
                );
                path.cubic_bezier_to(
                    view.point(47.6752, 32.5957),
                    view.point(47.0839, 35.2933),
                    view.point(47.3677, 33.8262),
                );
                path.cubic_bezier_to(
                    view.point(47.6989, 19.2025),
                    view.point(48.6923, 28.644),
                    view.point(48.716, 23.5801),
                );
                path.cubic_bezier_to(
                    view.point(47.1548, 16.5759),
                    view.point(47.486, 18.3033),
                    view.point(47.2494, 17.1201),
                );
                path.cubic_bezier_to(
                    view.point(45.9011, 13.2158),
                    view.point(46.8, 14.2333),
                    view.point(46.5871, 13.6654),
                );
                path.cubic_bezier_to(
                    view.point(45.57, 17.3568),
                    view.point(44.955, 12.6005),
                    view.point(44.7657, 15.1088),
                );
                path.cubic_bezier_to(
                    view.point(46.3033, 19.3918),
                    view.point(45.7829, 17.972),
                    view.point(46.114, 18.8949),
                );
                path.cubic_bezier_to(
                    view.point(47.2967, 23.8878),
                    view.point(46.6108, 20.2673),
                    view.point(47.0129, 21.9947),
                );
                path.cubic_bezier_to(
                    view.point(45.9485, 33.6132),
                    view.point(47.7225, 26.467),
                    view.point(47.1548, 30.537),
                );
                path.cubic_bezier_to(
                    view.point(46.1377, 38.5351),
                    view.point(44.6948, 36.8077),
                    view.point(44.813, 39.8602),
                );
                path.close();
                path.move_to(view.point(26.7175, 29.8272));
                path.cubic_bezier_to(
                    view.point(28.397, 28.1234),
                    view.point(26.9067, 29.7088),
                    view.point(27.6637, 28.9516),
                );
                path.cubic_bezier_to(
                    view.point(28.1131, 23.7458),
                    view.point(30.4076, 25.9464),
                    view.point(30.3839, 25.8045),
                );
                path.cubic_bezier_to(
                    view.point(25.2983, 22.2313),
                    view.point(26.5046, 22.2787),
                    view.point(25.9133, 21.9947),
                );
                path.cubic_bezier_to(
                    view.point(21.9393, 25.7335),
                    view.point(24.8015, 22.4443),
                    view.point(22.2469, 25.1182),
                );
                path.cubic_bezier_to(
                    view.point(23.335, 28.1944),
                    view.point(21.6318, 26.3724),
                    view.point(21.8447, 26.751),
                );
                path.cubic_bezier_to(
                    view.point(26.7175, 29.8272),
                    view.point(25.3219, 30.1111),
                    view.point(25.866, 30.3714),
                );
                path.close();
                path.move_to(view.point(6.54041, 10.3289));
                path.cubic_bezier_to(
                    view.point(7.15542, 8.57782),
                    view.point(6.98984, 9.85562),
                    view.point(7.15542, 9.38236),
                );
                path.cubic_bezier_to(
                    view.point(9.07142, 6.89775),
                    view.point(7.15542, 7.34735),
                    view.point(7.67582, 6.89775),
                );
                path.cubic_bezier_to(
                    view.point(10.0649, 6.51914),
                    view.point(9.52085, 6.89775),
                    view.point(9.73374, 6.8031),
                );
                path.cubic_bezier_to(
                    view.point(6.32752, 5.40698),
                    view.point(10.9165, 5.73827),
                    view.point(10.3487, 5.57263),
                );
                path.cubic_bezier_to(
                    view.point(5.38135, 8.03357),
                    view.point(5.33404, 5.38332),
                    view.point(5.28673, 5.50164),
                );
                path.cubic_bezier_to(
                    view.point(6.54041, 10.3289),
                    view.point(5.45231, 10.7311),
                    view.point(5.68886, 11.2044),
                );
                path.close();
                path.move_to(view.point(46.3506, 10.3999));
                path.cubic_bezier_to(
                    view.point(46.4215, 5.5253),
                    view.point(46.5871, 10.0923),
                    view.point(46.6581, 5.76193),
                );
                path.cubic_bezier_to(
                    view.point(41.5724, 5.61995),
                    view.point(46.185, 5.28867),
                    view.point(41.8799, 5.38332),
                );
                path.cubic_bezier_to(
                    view.point(43.6067, 7.01607),
                    view.point(40.8864, 6.1642),
                    view.point(42.1165, 7.01607),
                );
                path.cubic_bezier_to(
                    view.point(45.0732, 8.76712),
                    view.point(44.6948, 7.01607),
                    view.point(44.9786, 7.37101),
                );
                path.cubic_bezier_to(
                    view.point(45.4754, 10.2342),
                    view.point(45.1442, 9.71364),
                    view.point(45.2152, 9.95027),
                );
                path.cubic_bezier_to(
                    view.point(46.3506, 10.3999),
                    view.point(45.8065, 10.5892),
                    view.point(46.1377, 10.6602),
                );
                path.close();
                path.move_to(view.point(17.9654, 6.21153));
                path.cubic_bezier_to(
                    view.point(25.9369, 4.50779),
                    view.point(20.4255, 5.14669),
                    view.point(23.4769, 4.48413),
                );
                path.cubic_bezier_to(
                    view.point(33.4353, 5.9749),
                    view.point(28.1368, 4.53146),
                    view.point(31.3774, 5.17036),
                );
                path.cubic_bezier_to(
                    view.point(35.4223, 6.77944),
                    view.point(34.1923, 6.28251),
                    view.point(35.0911, 6.63746),
                );
                path.cubic_bezier_to(
                    view.point(39.0887, 6.8031),
                    view.point(36.2738, 7.11072),
                    view.point(38.7102, 7.13438),
                );
                path.cubic_bezier_to(
                    view.point(38.8995, 5.92757),
                    view.point(39.4199, 6.49548),
                    view.point(39.3726, 6.30618),
                );
                path.cubic_bezier_to(
                    view.point(35.99, 4.91006),
                    view.point(38.1189, 5.35966),
                    view.point(37.5985, 5.17036),
                );
                path.cubic_bezier_to(
                    view.point(32.773, 4.29483),
                    view.point(35.1384, 4.76809),
                    view.point(33.6719, 4.48413),
                );
                path.cubic_bezier_to(
                    view.point(19.1955, 4.34215),
                    view.point(28.4443, 3.27732),
                    view.point(23.5951, 3.30098),
                );
                path.cubic_bezier_to(
                    view.point(15.7892, 5.00471),
                    view.point(18.1547, 4.57878),
                    view.point(16.6171, 4.8864),
                );
                path.cubic_bezier_to(
                    view.point(12.8561, 6.77944),
                    view.point(13.5894, 5.28867),
                    view.point(12.2647, 6.11687),
                );
                path.cubic_bezier_to(
                    view.point(14.6775, 6.9924),
                    view.point(13.069, 7.01607),
                    view.point(13.2109, 7.03973),
                );
                path.line_to(view.point(16.2387, 6.94508));
                path.line_to(view.point(17.9654, 6.21153));
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
