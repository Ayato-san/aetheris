use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct ShugoIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl ShugoIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for ShugoIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(11.9859, 50.8736));
                path.cubic_bezier_to(
                    view.point(9.17175, 46.1191),
                    view.point(11.2528, 50.093),
                    view.point(9.57378, 47.2545),
                );
                path.cubic_bezier_to(
                    view.point(8.03662, 40.584),
                    view.point(8.10756, 43.044),
                    view.point(7.96567, 42.2634),
                );
                path.cubic_bezier_to(
                    view.point(11.3001, 37.8164),
                    view.point(8.17851, 38.1712),
                    view.point(8.81702, 37.6035),
                );
                path.cubic_bezier_to(
                    view.point(23.0062, 42.973),
                    view.point(13.8542, 38.0293),
                    view.point(17.8272, 39.7797),
                );
                path.cubic_bezier_to(
                    view.point(24.9454, 43.2332),
                    view.point(24.0231, 43.6117),
                    view.point(24.1886, 43.6354),
                );
                path.cubic_bezier_to(
                    view.point(27.192, 43.2332),
                    view.point(25.7967, 42.8075),
                    view.point(26.388, 42.8075),
                );
                path.cubic_bezier_to(
                    view.point(28.9184, 43.115),
                    view.point(27.9488, 43.6117),
                    view.point(28.0907, 43.6117),
                );
                path.cubic_bezier_to(
                    view.point(30.1245, 42.4053),
                    view.point(29.2967, 42.8784),
                    view.point(29.817, 42.5709),
                );
                path.cubic_bezier_to(
                    view.point(31.5197, 41.5538),
                    view.point(30.4082, 42.2398),
                    view.point(31.0467, 41.8613),
                );
                path.cubic_bezier_to(
                    view.point(36.4386, 39.0937),
                    view.point(32.4657, 40.9861),
                    view.point(35.9184, 39.2593),
                );
                path.cubic_bezier_to(
                    view.point(37.1008, 38.8099),
                    view.point(36.6278, 39.0228),
                    view.point(36.9116, 38.9045),
                );
                path.cubic_bezier_to(
                    view.point(37.9522, 38.4787),
                    view.point(37.2663, 38.6916),
                    view.point(37.6447, 38.5497),
                );
                path.cubic_bezier_to(
                    view.point(38.9454, 38.1712),
                    view.point(38.2359, 38.4078),
                    view.point(38.6853, 38.2658),
                );
                path.cubic_bezier_to(
                    view.point(41.5704, 37.7928),
                    view.point(39.5603, 37.9347),
                    view.point(40.4589, 37.7928),
                );
                path.cubic_bezier_to(
                    view.point(43.8407, 39.0701),
                    view.point(42.9657, 37.7928),
                    view.point(43.4386, 38.053),
                );
                path.cubic_bezier_to(
                    view.point(44.0299, 40.9861),
                    view.point(44.0062, 39.4959),
                    view.point(44.0299, 39.6614),
                );
                path.cubic_bezier_to(
                    view.point(42.4217, 47.1835),
                    view.point(44.0299, 43.1386),
                    view.point(43.6751, 44.5579),
                );
                path.cubic_bezier_to(
                    view.point(39.9386, 51.0155),
                    view.point(41.6177, 48.8866),
                    view.point(40.388, 50.8026),
                );
                path.cubic_bezier_to(
                    view.point(37.7157, 50.5424),
                    view.point(39.7022, 51.1338),
                    view.point(38.4724, 50.8736),
                );
                path.cubic_bezier_to(
                    view.point(36.6042, 50.1403),
                    view.point(37.5028, 50.4478),
                    view.point(37.0062, 50.2586),
                );
                path.cubic_bezier_to(
                    view.point(33.9555, 49.2651),
                    view.point(35.7528, 49.8801),
                    view.point(34.8305, 49.5726),
                );
                path.cubic_bezier_to(
                    view.point(32.7022, 48.8393),
                    view.point(33.6008, 49.1468),
                    view.point(33.0332, 48.9339),
                );
                path.cubic_bezier_to(
                    view.point(31.094, 48.2479),
                    view.point(32.3711, 48.721),
                    view.point(31.638, 48.4608),
                );
                path.cubic_bezier_to(
                    view.point(28.8474, 48.2716),
                    view.point(29.817, 47.7749),
                    view.point(29.5569, 47.7749),
                );
                path.cubic_bezier_to(
                    view.point(26.1278, 48.8866),
                    view.point(27.9961, 48.8393),
                    view.point(27.8542, 48.8866),
                );
                path.cubic_bezier_to(
                    view.point(23.3373, 48.2953),
                    view.point(24.3778, 48.8866),
                    view.point(24.2596, 48.863),
                );
                path.cubic_bezier_to(
                    view.point(20.9724, 48.2953),
                    view.point(22.415, 47.7039),
                    view.point(22.5332, 47.7039),
                );
                path.cubic_bezier_to(
                    view.point(19.2697, 48.8866),
                    view.point(20.4758, 48.4845),
                    view.point(19.719, 48.7447),
                );
                path.cubic_bezier_to(
                    view.point(17.8981, 49.3597),
                    view.point(18.8204, 49.0522),
                    view.point(18.2055, 49.2651),
                );
                path.cubic_bezier_to(
                    view.point(16.8103, 49.7145),
                    view.point(17.5907, 49.478),
                    view.point(17.094, 49.6436),
                );
                path.cubic_bezier_to(
                    view.point(15.2258, 50.2586),
                    view.point(16.5265, 49.8091),
                    view.point(15.817, 50.0693),
                );
                path.cubic_bezier_to(
                    view.point(11.9859, 50.8736),
                    view.point(12.7427, 51.1574),
                    view.point(12.317, 51.2284),
                );
                path.close();
                path.move_to(view.point(24.3778, 41.8376));
                path.cubic_bezier_to(
                    view.point(23.1954, 41.1753),
                    view.point(24.2596, 41.7903),
                    view.point(23.7157, 41.5065),
                );
                path.cubic_bezier_to(
                    view.point(18.8677, 38.9282),
                    view.point(22.3677, 40.6313),
                    view.point(21.54, 40.2055),
                );
                path.cubic_bezier_to(
                    view.point(17.9454, 38.5497),
                    view.point(18.513, 38.7626),
                    view.point(18.0873, 38.597),
                );
                path.cubic_bezier_to(
                    view.point(17.0467, 38.2422),
                    view.point(17.7799, 38.526),
                    view.point(17.3778, 38.3841),
                );
                path.cubic_bezier_to(
                    view.point(16.0062, 37.8401),
                    view.point(16.7157, 38.1003),
                    view.point(16.2427, 37.911),
                );
                path.cubic_bezier_to(
                    view.point(15.2495, 37.5326),
                    view.point(15.7461, 37.7691),
                    view.point(15.415, 37.6272),
                );
                path.cubic_bezier_to(
                    view.point(14.3745, 37.2724),
                    view.point(15.0839, 37.4616),
                    view.point(14.6819, 37.3197),
                );
                path.cubic_bezier_to(
                    view.point(13.2393, 36.9649),
                    view.point(14.0434, 37.2014),
                    view.point(13.5467, 37.0595),
                );
                path.cubic_bezier_to(
                    view.point(11.8204, 36.5864),
                    view.point(12.9555, 36.8466),
                    view.point(12.2934, 36.681),
                );
                path.cubic_bezier_to(
                    view.point(10.2832, 36.2079),
                    view.point(11.3474, 36.4918),
                    view.point(10.6616, 36.3262),
                );
                path.cubic_bezier_to(
                    view.point(6.90148, 37.0358),
                    view.point(9.1008, 35.8294),
                    view.point(7.77648, 36.1606),
                );
                path.cubic_bezier_to(
                    view.point(6.07378, 38.4787),
                    view.point(6.49945, 37.4616),
                    view.point(6.07378, 38.1712),
                );
                path.cubic_bezier_to(
                    view.point(5.17513, 39.212),
                    view.point(6.07378, 39.0701),
                    view.point(5.62445, 39.4249),
                );
                path.cubic_bezier_to(
                    view.point(2.1954, 35.995),
                    view.point(4.63121, 38.9518),
                    view.point(2.95216, 37.1304),
                );
                path.cubic_bezier_to(
                    view.point(0.114318, 30.6255),
                    view.point(0.77648, 33.8188),
                    view.point(0.398102, 32.8017),
                );
                path.cubic_bezier_to(
                    view.point(0.208913, 27.7633),
                    view.point(-0.0512222, 29.2062),
                    view.point(-0.0512223, 29.2299),
                );
                path.cubic_bezier_to(
                    view.point(3.9927, 21.968),
                    view.point(0.61094, 25.2796),
                    view.point(2.28999, 22.725),
                );
                path.cubic_bezier_to(
                    view.point(9.1481, 22.0863),
                    view.point(5.41161, 21.3057),
                    view.point(7.51634, 21.3767),
                );
                path.cubic_bezier_to(
                    view.point(10.5907, 20.3832),
                    view.point(10.4488, 22.654),
                    view.point(10.6143, 22.4411),
                );
                path.line_to(view.point(10.567, 19.1295));
                path.line_to(view.point(10.3542, 19.1059));
                path.cubic_bezier_to(
                    view.point(9.66837, 19.6026),
                    view.point(10.165, 19.0822),
                    view.point(10.0467, 19.1768),
                );
                path.cubic_bezier_to(
                    view.point(7.75283, 20.2649),
                    view.point(9.07716, 20.2886),
                    view.point(8.95891, 20.3359),
                );
                path.cubic_bezier_to(
                    view.point(5.43526, 20.2886),
                    view.point(7.23256, 20.2413),
                    view.point(6.16837, 20.2413),
                );
                path.cubic_bezier_to(
                    view.point(3.68526, 20.0047),
                    view.point(3.96905, 20.3832),
                    view.point(3.77986, 20.3595),
                );
                path.cubic_bezier_to(
                    view.point(4.65486, 16.977),
                    view.point(3.63797, 19.8392),
                    view.point(4.15824, 18.207),
                );
                path.cubic_bezier_to(
                    view.point(5.03324, 16.0072),
                    view.point(4.7731, 16.6931),
                    view.point(4.96229, 16.2674),
                );
                path.cubic_bezier_to(
                    view.point(5.57716, 14.7535),
                    view.point(5.12783, 15.7706),
                    view.point(5.38797, 15.2029),
                );
                path.cubic_bezier_to(
                    view.point(5.83729, 11.9623),
                    view.point(6.05013, 13.76),
                    view.point(6.09743, 13.2396),
                );
                path.cubic_bezier_to(
                    view.point(5.83729, 7.25507),
                    view.point(5.62445, 10.7559),
                    view.point(5.6008, 8.34317),
                );
                path.cubic_bezier_to(
                    view.point(6.02648, 6.14332),
                    view.point(5.90824, 6.85295),
                    view.point(6.00283, 6.35621),
                );
                path.cubic_bezier_to(
                    view.point(7.18526, 2.97365),
                    view.point(6.12107, 5.481),
                    view.point(6.78324, 3.70693),
                );
                path.cubic_bezier_to(
                    view.point(8.81702, 1.0813),
                    view.point(7.61094, 2.21671),
                    view.point(8.46229, 1.24688),
                );
                path.cubic_bezier_to(
                    view.point(14.1853, 3.06826),
                    view.point(9.59743, 0.773798),
                    view.point(11.1109, 1.31785),
                );
                path.cubic_bezier_to(
                    view.point(20.4285, 10.1645),
                    view.point(16.6684, 4.48752),
                    view.point(19.2461, 7.42065),
                );
                path.cubic_bezier_to(
                    view.point(20.4994, 12.0096),
                    view.point(20.9488, 11.3709),
                    view.point(20.9488, 11.5128),
                );
                path.cubic_bezier_to(
                    view.point(20.9724, 13.0504),
                    view.point(20.1684, 12.4117),
                    view.point(20.2157, 12.5063),
                );
                path.cubic_bezier_to(
                    view.point(22.1076, 13.2396),
                    view.point(21.7055, 13.5708),
                    view.point(21.942, 13.6181),
                );
                path.cubic_bezier_to(
                    view.point(21.8711, 11.1107),
                    view.point(22.2731, 12.9558),
                    view.point(22.1312, 11.7967),
                );
                path.cubic_bezier_to(
                    view.point(21.5163, 10.0463),
                    view.point(21.7765, 10.8505),
                    view.point(21.6109, 10.3774),
                );
                path.cubic_bezier_to(
                    view.point(20.1211, 6.52179),
                    view.point(21.1616, 8.83991),
                    view.point(20.6886, 7.60989),
                );
                path.cubic_bezier_to(
                    view.point(21.6109, 4.18002),
                    view.point(19.1042, 4.48752),
                    view.point(19.1278, 4.46387),
                );
                path.cubic_bezier_to(
                    view.point(30.8576, 4.22732),
                    view.point(23.5501, 3.96713),
                    view.point(28.9657, 3.99078),
                );
                path.cubic_bezier_to(
                    view.point(32.0636, 6.45083),
                    view.point(32.9859, 4.48752),
                    view.point(33.0332, 4.53483),
                );
                path.cubic_bezier_to(
                    view.point(30.0299, 12.2934),
                    view.point(31.2832, 8.03566),
                    view.point(30.1717, 11.2053),
                );
                path.cubic_bezier_to(
                    view.point(31.3778, 12.9084),
                    view.point(29.8643, 13.689),
                    view.point(30.1717, 13.8073),
                );
                path.cubic_bezier_to(
                    view.point(31.6616, 12.0096),
                    view.point(31.969, 12.4827),
                    view.point(31.9927, 12.3881),
                );
                path.cubic_bezier_to(
                    view.point(31.9927, 9.64415),
                    view.point(31.1886, 11.4892),
                    view.point(31.2123, 11.229),
                );
                path.cubic_bezier_to(
                    view.point(41.263, 1.41246),
                    view.point(33.79, 5.95409),
                    view.point(36.9353, 3.13923),
                );
                path.cubic_bezier_to(
                    view.point(43.4386, 1.19958),
                    view.point(42.3508, 0.963033),
                    view.point(42.942, 0.89207),
                );
                path.cubic_bezier_to(
                    view.point(45.4488, 4.13271),
                    view.point(44.0299, 1.53074),
                    view.point(45.0231, 2.97365),
                );
                path.cubic_bezier_to(
                    view.point(46.3001, 8.27221),
                    view.point(46.0163, 5.59927),
                    view.point(45.9927, 5.43369),
                );
                path.cubic_bezier_to(
                    view.point(46.2055, 12.0805),
                    view.point(46.3947, 9.07645),
                    view.point(46.3474, 11.158),
                );
                path.cubic_bezier_to(
                    view.point(46.6549, 15.1319),
                    view.point(45.9927, 13.4761),
                    view.point(46.04, 13.8546),
                );
                path.cubic_bezier_to(
                    view.point(48.3339, 20.0047),
                    view.point(47.4116, 16.7404),
                    view.point(48.3812, 19.5553),
                );
                path.cubic_bezier_to(
                    view.point(47.2461, 20.3595),
                    view.point(48.2866, 20.3832),
                    view.point(48.0028, 20.4778),
                );
                path.cubic_bezier_to(
                    view.point(45.0704, 20.2886),
                    view.point(46.8913, 20.3122),
                    view.point(45.9454, 20.2886),
                );
                path.cubic_bezier_to(
                    view.point(43.344, 20.3122),
                    view.point(44.2427, 20.3122),
                    view.point(43.4623, 20.3122),
                );
                path.cubic_bezier_to(
                    view.point(42.5636, 19.6972),
                    view.point(43.2258, 20.2886),
                    view.point(42.942, 20.0757),
                );
                path.cubic_bezier_to(
                    view.point(41.5467, 19.2005),
                    view.point(41.9488, 19.0822),
                    view.point(41.6413, 18.9403),
                );
                path.cubic_bezier_to(
                    view.point(41.7123, 22.1336),
                    view.point(41.4522, 19.4607),
                    view.point(41.594, 21.9444),
                );
                path.cubic_bezier_to(
                    view.point(42.9893, 22.0627),
                    view.point(41.8542, 22.3938),
                    view.point(42.3035, 22.3702),
                );
                path.cubic_bezier_to(
                    view.point(47.1988, 21.6605),
                    view.point(44.1717, 21.5186),
                    view.point(45.8508, 21.353),
                );
                path.cubic_bezier_to(
                    view.point(51.4555, 26.3204),
                    view.point(48.9961, 22.039),
                    view.point(50.4386, 23.6238),
                );
                path.cubic_bezier_to(
                    view.point(51.9049, 31.1932),
                    view.point(51.9758, 27.6451),
                    view.point(52.1177, 29.3009),
                );
                path.cubic_bezier_to(
                    view.point(47.8136, 38.5024),
                    view.point(51.6447, 33.3221),
                    view.point(49.6346, 36.9175),
                );
                path.cubic_bezier_to(
                    view.point(45.9927, 38.4551),
                    view.point(46.6076, 39.5195),
                    view.point(46.3474, 39.5195),
                );
                path.cubic_bezier_to(
                    view.point(41.7359, 36.2552),
                    view.point(45.3778, 36.5391),
                    view.point(43.5805, 35.6166),
                );
                path.cubic_bezier_to(
                    view.point(40.3643, 36.5864),
                    view.point(41.4522, 36.3498),
                    view.point(40.8136, 36.4918),
                );
                path.cubic_bezier_to(
                    view.point(37.3373, 37.4379),
                    view.point(39.5603, 36.7283),
                    view.point(38.8981, 36.9175),
                );
                path.cubic_bezier_to(
                    view.point(35.067, 38.2658),
                    view.point(36.0366, 37.8637),
                    view.point(35.3981, 38.1003),
                );
                path.cubic_bezier_to(
                    view.point(34.1447, 38.597),
                    view.point(34.8778, 38.3605),
                    view.point(34.4758, 38.5024),
                );
                path.cubic_bezier_to(
                    view.point(28.9893, 41.1517),
                    view.point(33.388, 38.8099),
                    view.point(30.219, 40.3711),
                );
                path.cubic_bezier_to(
                    view.point(26.6954, 41.8376),
                    view.point(27.7832, 41.9086),
                    view.point(27.5704, 41.9796),
                );
                path.cubic_bezier_to(
                    view.point(25.513, 41.814),
                    view.point(26.1751, 41.743),
                    view.point(25.915, 41.743),
                );
                path.cubic_bezier_to(
                    view.point(24.3778, 41.8376),
                    view.point(24.9217, 41.9323),
                    view.point(24.6853, 41.9323),
                );
                path.close();
                path.move_to(view.point(22.8643, 30.9567));
                path.cubic_bezier_to(
                    view.point(24.9927, 29.5611),
                    view.point(23.6447, 30.6019),
                    view.point(24.3778, 30.1288),
                );
                path.cubic_bezier_to(
                    view.point(27.3103, 29.632),
                    view.point(25.9386, 28.7095),
                    view.point(26.2934, 28.7332),
                );
                path.cubic_bezier_to(
                    view.point(30.3136, 31.2405),
                    view.point(28.2562, 30.5072),
                    view.point(29.7461, 31.3115),
                );
                path.cubic_bezier_to(
                    view.point(30.3136, 30.862),
                    view.point(30.6211, 31.2169),
                    view.point(30.6447, 31.0276),
                );
                path.cubic_bezier_to(
                    view.point(29.6751, 30.4836),
                    view.point(30.1954, 30.7911),
                    view.point(29.888, 30.6255),
                );
                path.cubic_bezier_to(
                    view.point(28.942, 30.0815),
                    view.point(29.4623, 30.3653),
                    view.point(29.1312, 30.1761),
                );
                path.cubic_bezier_to(
                    view.point(28.0434, 29.3718),
                    view.point(28.7765, 30.0105),
                    view.point(28.3508, 29.6793),
                );
                path.cubic_bezier_to(
                    view.point(28.54, 27.5504),
                    view.point(27.1447, 28.4966),
                    view.point(27.2393, 28.2128),
                );
                path.cubic_bezier_to(
                    view.point(30.5974, 25.0431),
                    view.point(29.8407, 26.9118),
                    view.point(30.5974, 25.9656),
                );
                path.cubic_bezier_to(
                    view.point(29.7224, 23.5765),
                    view.point(30.5974, 24.3571),
                    view.point(30.4082, 24.026),
                );
                path.cubic_bezier_to(
                    view.point(26.8846, 23.1981),
                    view.point(28.7528, 22.8906),
                    view.point(27.9251, 22.7959),
                );
                path.cubic_bezier_to(
                    view.point(25.2292, 23.1981),
                    view.point(26.1278, 23.5056),
                    view.point(26.0096, 23.5056),
                );
                path.cubic_bezier_to(
                    view.point(22.2967, 23.7185),
                    view.point(24.1413, 22.7486),
                    view.point(23.1481, 22.9379),
                );
                path.cubic_bezier_to(
                    view.point(23.6447, 27.5741),
                    view.point(20.9724, 24.9485),
                    view.point(21.54, 26.5097),
                );
                path.cubic_bezier_to(
                    view.point(24.1886, 29.3245),
                    view.point(24.8745, 28.1891),
                    view.point(24.969, 28.4966),
                );
                path.cubic_bezier_to(
                    view.point(22.6515, 30.4126),
                    view.point(23.763, 29.774),
                    view.point(23.4555, 29.9868),
                );
                path.cubic_bezier_to(
                    view.point(21.6582, 31.2642),
                    view.point(21.6346, 30.9567),
                    view.point(21.4454, 31.1222),
                );
                path.cubic_bezier_to(
                    view.point(22.8643, 30.9567),
                    view.point(21.8001, 31.3824),
                    view.point(22.0366, 31.3115),
                );
                path.close();
                path.move_to(view.point(17.7799, 22.3465));
                path.cubic_bezier_to(
                    view.point(20.0501, 21.2821),
                    view.point(18.442, 22.2046),
                    view.point(19.7663, 21.5896),
                );
                path.cubic_bezier_to(
                    view.point(20.3103, 18.278),
                    view.point(20.5704, 20.738),
                    view.point(20.665, 19.6026),
                );
                path.cubic_bezier_to(
                    view.point(19.2224, 16.3383),
                    view.point(20.0501, 17.3318),
                    view.point(19.8609, 16.977),
                );
                path.cubic_bezier_to(
                    view.point(17.0231, 15.3921),
                    view.point(18.4893, 15.6287),
                    view.point(18.0163, 15.4158),
                );
                path.line_to(view.point(16.219, 15.3448));
                path.line_to(view.point(15.5332, 15.6997));
                path.cubic_bezier_to(
                    view.point(13.263, 18.9403),
                    view.point(14.0197, 16.4802),
                    view.point(13.3339, 17.4501),
                );
                path.cubic_bezier_to(
                    view.point(13.4995, 20.3359),
                    view.point(13.2393, 19.7918),
                    view.point(13.2393, 19.8155),
                );
                path.cubic_bezier_to(
                    view.point(17.7799, 22.3465),
                    view.point(14.3508, 21.9917),
                    view.point(15.9353, 22.7486),
                );
                path.close();
                path.move_to(view.point(36.7697, 22.2046));
                path.cubic_bezier_to(
                    view.point(38.8272, 19.8628),
                    view.point(37.5974, 21.7788),
                    view.point(38.567, 20.6907),
                );
                path.cubic_bezier_to(
                    view.point(38.7089, 17.8758),
                    view.point(38.9454, 19.4607),
                    view.point(38.8981, 18.3726),
                );
                path.cubic_bezier_to(
                    view.point(36.344, 15.5577),
                    view.point(38.4015, 16.977),
                    view.point(37.4792, 16.0545),
                );
                path.cubic_bezier_to(
                    view.point(33.9082, 15.6287),
                    view.point(35.7292, 15.2739),
                    view.point(34.5704, 15.2975),
                );
                path.cubic_bezier_to(
                    view.point(31.6616, 19.5316),
                    view.point(32.5603, 16.291),
                    view.point(31.7562, 17.6866),
                );
                path.cubic_bezier_to(
                    view.point(33.1042, 21.8971),
                    view.point(31.5907, 20.9036),
                    view.point(31.8508, 21.3057),
                );
                path.cubic_bezier_to(
                    view.point(36.7697, 22.2046),
                    view.point(34.4758, 22.5357),
                    view.point(35.8474, 22.654),
                );
                path.close();
                path.move_to(view.point(44.1481, 12.3171));
                path.cubic_bezier_to(
                    view.point(44.8103, 7.34969),
                    view.point(44.6447, 11.8203),
                    view.point(44.9521, 9.50223),
                );
                path.cubic_bezier_to(
                    view.point(44.3609, 5.481),
                    view.point(44.7157, 6.23794),
                    view.point(44.7157, 6.19063),
                );
                path.cubic_bezier_to(
                    view.point(43.7697, 4.46387),
                    view.point(44.1717, 5.07888),
                    view.point(43.9116, 4.62945),
                );
                path.cubic_bezier_to(
                    view.point(43.2731, 3.8252),
                    view.point(43.6515, 4.32194),
                    view.point(43.415, 4.01443),
                );
                path.cubic_bezier_to(
                    view.point(41.7832, 3.75424),
                    view.point(42.7765, 3.09192),
                    view.point(42.469, 3.06826),
                );
                path.cubic_bezier_to(
                    view.point(40.1515, 6.66371),
                    view.point(41.1447, 4.34559),
                    view.point(40.4826, 5.55196),
                );
                path.cubic_bezier_to(
                    view.point(39.6785, 8.86356),
                    view.point(39.7494, 8.03566),
                    view.point(39.7258, 8.17759),
                );
                path.cubic_bezier_to(
                    view.point(40.9319, 10.6849),
                    view.point(39.5839, 10.0463),
                    view.point(39.7731, 10.3301),
                );
                path.cubic_bezier_to(
                    view.point(42.9893, 11.8913),
                    view.point(41.7596, 10.9215),
                    view.point(42.4454, 11.3236),
                );
                path.cubic_bezier_to(
                    view.point(43.8643, 12.5536),
                    view.point(43.2967, 12.2225),
                    view.point(43.7224, 12.5536),
                );
                path.cubic_bezier_to(
                    view.point(44.1481, 12.3171),
                    view.point(43.888, 12.5536),
                    view.point(44.0299, 12.4354),
                );
                path.close();
                path.move_to(view.point(9.05351, 11.9386));
                path.cubic_bezier_to(
                    view.point(11.2055, 10.6849),
                    view.point(9.83391, 11.229),
                    view.point(10.2596, 10.9688),
                );
                path.cubic_bezier_to(
                    view.point(12.4589, 9.71512),
                    view.point(12.1042, 10.4011),
                    view.point(12.4353, 10.1409),
                );
                path.cubic_bezier_to(
                    view.point(11.2528, 4.96061),
                    view.point(12.5062, 8.41413),
                    view.point(11.915, 6.07236),
                );
                path.cubic_bezier_to(
                    view.point(9.43189, 3.30481),
                    view.point(10.4961, 3.73058),
                    view.point(9.90486, 3.18654),
                );
                path.cubic_bezier_to(
                    view.point(7.82378, 5.29177),
                    view.point(9.1954, 3.35212),
                    view.point(8.2258, 4.58214),
                );
                path.cubic_bezier_to(
                    view.point(7.42175, 10.8978),
                    view.point(7.20891, 6.42717),
                    view.point(7.01972, 8.76895),
                );
                path.cubic_bezier_to(
                    view.point(9.05351, 11.9386),
                    view.point(7.72918, 12.6009),
                    view.point(8.10756, 12.8611),
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
