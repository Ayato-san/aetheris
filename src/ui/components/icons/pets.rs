use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct PetsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl PetsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for PetsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(22.0914, 49.7421));
                path.cubic_bezier_to(
                    view.point(21.6883, 47.5373),
                    view.point(21.6646, 49.4339),
                    view.point(21.5698, 48.9124),
                );
                path.cubic_bezier_to(
                    view.point(21.3089, 43.009),
                    view.point(21.8306, 46.1859),
                    view.point(21.7595, 45.4035),
                );
                path.cubic_bezier_to(
                    view.point(20.8821, 40.3063),
                    view.point(21.1429, 42.1792),
                    view.point(20.9533, 40.9464),
                );
                path.cubic_bezier_to(
                    view.point(20.3605, 37.4613),
                    view.point(20.811, 39.6662),
                    view.point(20.5739, 38.3859),
                );
                path.cubic_bezier_to(
                    view.point(19.6965, 34.5926),
                    view.point(20.1471, 36.5604),
                    view.point(19.8388, 35.2564),
                );
                path.cubic_bezier_to(
                    view.point(18.4635, 30.6808),
                    view.point(19.2934, 32.6248),
                    view.point(18.8903, 31.4157),
                );
                path.cubic_bezier_to(
                    view.point(19.9574, 29.7087),
                    view.point(17.4202, 29.0212),
                    view.point(17.9418, 28.6656),
                );
                path.cubic_bezier_to(
                    view.point(21.9017, 30.2777),
                    view.point(20.5739, 30.0169),
                    view.point(21.0481, 30.1592),
                );
                path.cubic_bezier_to(
                    view.point(23.6801, 30.9415),
                    view.point(22.9451, 30.42),
                    view.point(23.0636, 30.4674),
                );
                path.cubic_bezier_to(
                    view.point(25.174, 32.0795),
                    view.point(24.0358, 31.226),
                    view.point(24.6997, 31.7239),
                );
                path.cubic_bezier_to(
                    view.point(26.3596, 33.0516),
                    view.point(25.6245, 32.4115),
                    view.point(26.1699, 32.8619),
                );
                path.cubic_bezier_to(
                    view.point(26.1936, 36.2285),
                    view.point(26.7627, 33.502),
                    view.point(26.739, 33.7154),
                );
                path.cubic_bezier_to(
                    view.point(25.8853, 41.7525),
                    view.point(25.7193, 38.4808),
                    view.point(25.6245, 40.0692),
                );
                path.line_to(view.point(26.0987, 43.009));
                path.line_to(view.point(25.7193, 44.6686));
                path.cubic_bezier_to(
                    view.point(24.4389, 49.0309),
                    view.point(25.2688, 46.7075),
                    view.point(24.7234, 48.5567),
                );
                path.cubic_bezier_to(
                    view.point(22.0914, 49.7421),
                    view.point(23.9884, 49.8133),
                    view.point(22.7554, 50.1926),
                );
                path.close();
                path.move_to(view.point(41.464, 49.3391));
                path.cubic_bezier_to(
                    view.point(40.9424, 48.8887),
                    view.point(41.3218, 49.268),
                    view.point(41.0846, 49.0783),
                );
                path.cubic_bezier_to(
                    view.point(41.1558, 45.3087),
                    view.point(40.6341, 48.4856),
                    view.point(40.6341, 48.2959),
                );
                path.cubic_bezier_to(
                    view.point(41.7249, 40.7567),
                    view.point(41.3692, 44.1944),
                    view.point(41.6063, 42.1318),
                );
                path.line_to(view.point(41.9146, 38.2437));
                path.line_to(view.point(41.63, 37.6273));
                path.cubic_bezier_to(
                    view.point(40.7052, 36.2285),
                    view.point(41.464, 37.2716),
                    view.point(41.0372, 36.6552),
                );
                path.cubic_bezier_to(
                    view.point(39.2825, 33.3361),
                    view.point(39.9702, 35.3276),
                    view.point(39.2825, 33.9525),
                );
                path.cubic_bezier_to(
                    view.point(40.8001, 31.4157),
                    view.point(39.2825, 32.9567),
                    view.point(39.4248, 32.7908),
                );
                path.cubic_bezier_to(
                    view.point(44.4517, 23.0467),
                    view.point(43.361, 28.8789),
                    view.point(44.2857, 26.7452),
                );
                path.cubic_bezier_to(
                    view.point(44.7837, 21.0552),
                    view.point(44.5229, 21.3397),
                    view.point(44.5466, 21.2212),
                );
                path.cubic_bezier_to(
                    view.point(46.4198, 21.8613),
                    view.point(45.1631, 20.8181),
                    view.point(45.5662, 21.0078),
                );
                path.cubic_bezier_to(
                    view.point(47.4868, 23.6631),
                    view.point(47.0837, 22.5251),
                    view.point(47.1786, 22.6911),
                );
                path.cubic_bezier_to(
                    view.point(47.7002, 26.7452),
                    view.point(47.9611, 25.133),
                    view.point(47.9848, 25.5361),
                );
                path.cubic_bezier_to(
                    view.point(47.3209, 28.7367),
                    view.point(47.558, 27.3379),
                    view.point(47.392, 28.2388),
                );
                path.cubic_bezier_to(
                    view.point(46.894, 30.989),
                    view.point(47.2497, 29.2583),
                    view.point(47.06, 30.2777),
                );
                path.cubic_bezier_to(
                    view.point(46.2538, 35.3987),
                    view.point(46.4672, 32.8382),
                    view.point(46.1353, 35.0194),
                );
                path.cubic_bezier_to(
                    view.point(46.6569, 36.347),
                    view.point(46.3012, 35.5646),
                    view.point(46.4909, 35.9914),
                );
                path.cubic_bezier_to(
                    view.point(46.7992, 38.3385),
                    view.point(47.0126, 37.1057),
                    view.point(47.0363, 37.4139),
                );
                path.cubic_bezier_to(
                    view.point(46.3724, 40.4722),
                    view.point(46.7043, 38.7178),
                    view.point(46.5147, 39.6662),
                );
                path.cubic_bezier_to(
                    view.point(45.8507, 43.0801),
                    view.point(46.2301, 41.302),
                    view.point(45.993, 42.4637),
                );
                path.cubic_bezier_to(
                    view.point(45.4239, 45.1902),
                    view.point(45.6847, 43.6966),
                    view.point(45.495, 44.6449),
                );
                path.cubic_bezier_to(
                    view.point(43.6218, 49.2206),
                    view.point(45.0919, 47.561),
                    view.point(44.5229, 48.8175),
                );
                path.cubic_bezier_to(
                    view.point(41.464, 49.3391),
                    view.point(43.1001, 49.4576),
                    view.point(41.9146, 49.5288),
                );
                path.close();
                path.move_to(view.point(34.2556, 47.1105));
                path.cubic_bezier_to(
                    view.point(33.6865, 45.0479),
                    view.point(33.4968, 46.8972),
                    view.point(33.3783, 46.423),
                );
                path.cubic_bezier_to(
                    view.point(34.0896, 42.7008),
                    view.point(33.7814, 44.6449),
                    view.point(33.9711, 43.578),
                );
                path.cubic_bezier_to(
                    view.point(34.5876, 39.9981),
                    view.point(34.2319, 41.7999),
                    view.point(34.4453, 40.5908),
                );
                path.cubic_bezier_to(
                    view.point(34.7536, 37.4613),
                    view.point(34.801, 39.0498),
                    view.point(34.8247, 38.6704),
                );
                path.line_to(view.point(34.6824, 36.0625));
                path.line_to(view.point(35.0381, 35.4461));
                path.cubic_bezier_to(
                    view.point(38.1918, 34.7349),
                    view.point(35.9154, 33.9288),
                    view.point(37.2907, 33.6206),
                );
                path.cubic_bezier_to(
                    view.point(39.1877, 38.5519),
                    view.point(39.4722, 36.3707),
                    view.point(39.6856, 37.082),
                );
                path.cubic_bezier_to(
                    view.point(37.8361, 43.6728),
                    view.point(38.6186, 40.3063),
                    view.point(38.0258, 42.4874),
                );
                path.cubic_bezier_to(
                    view.point(36.8639, 46.3519),
                    view.point(37.5753, 45.1902),
                    view.point(37.3619, 45.7829),
                );
                path.cubic_bezier_to(
                    view.point(34.2556, 47.1105),
                    view.point(36.2711, 47.0394),
                    view.point(35.133, 47.3713),
                );
                path.close();
                path.move_to(view.point(13.0098, 45.5458));
                path.cubic_bezier_to(
                    view.point(12.1799, 43.9811),
                    view.point(12.2747, 45.1902),
                    view.point(12.1087, 44.8346),
                );
                path.cubic_bezier_to(
                    view.point(12.7964, 39.5002),
                    view.point(12.3221, 42.3926),
                    view.point(12.5355, 40.9464),
                );
                path.cubic_bezier_to(
                    view.point(12.7964, 33.3124),
                    view.point(13.1995, 37.3428),
                    view.point(13.1995, 34.9245),
                );
                path.cubic_bezier_to(
                    view.point(12.0376, 29.2346),
                    view.point(12.5355, 32.2218),
                    view.point(12.251, 30.7282),
                );
                path.cubic_bezier_to(
                    view.point(13.0572, 27.6698),
                    view.point(11.8479, 27.978),
                    view.point(12.2984, 27.2668),
                );
                path.cubic_bezier_to(
                    view.point(15.8552, 30.4437),
                    view.point(13.5077, 27.9069),
                    view.point(15.6418, 30.0169),
                );
                path.cubic_bezier_to(
                    view.point(17.1594, 34.9008),
                    view.point(16.2583, 31.226),
                    view.point(16.9697, 33.668),
                );
                path.cubic_bezier_to(
                    view.point(17.1594, 37.4613),
                    view.point(17.3253, 36.1099),
                    view.point(17.3253, 36.3233),
                );
                path.cubic_bezier_to(
                    view.point(16.9934, 39.761),
                    view.point(17.0645, 38.1251),
                    view.point(16.9934, 39.1683),
                );
                path.cubic_bezier_to(
                    view.point(16.5903, 42.1081),
                    view.point(16.9934, 40.6619),
                    view.point(16.9222, 41.0412),
                );
                path.cubic_bezier_to(
                    view.point(15.9738, 44.0285),
                    view.point(16.3531, 42.7956),
                    view.point(16.0923, 43.6728),
                );
                path.cubic_bezier_to(
                    view.point(13.0098, 45.5458),
                    view.point(15.4995, 45.5932),
                    view.point(14.3376, 46.2096),
                );
                path.close();
                path.move_to(view.point(28.3276, 39.1209));
                path.cubic_bezier_to(
                    view.point(27.9483, 38.1963),
                    view.point(27.8771, 38.6941),
                    view.point(27.7111, 38.3385),
                );
                path.cubic_bezier_to(
                    view.point(28.9204, 38.457),
                    view.point(28.0194, 38.1488),
                    view.point(28.4462, 38.2674),
                );
                path.cubic_bezier_to(
                    view.point(30.106, 38.8838),
                    view.point(29.3947, 38.6467),
                    view.point(29.9401, 38.8601),
                );
                path.cubic_bezier_to(
                    view.point(29.3947, 39.5476),
                    view.point(30.8648, 39.0735),
                    view.point(30.3432, 39.5476),
                );
                path.cubic_bezier_to(
                    view.point(28.3276, 39.1209),
                    view.point(28.873, 39.5476),
                    view.point(28.7307, 39.4765),
                );
                path.close();
                path.move_to(view.point(31.7422, 38.2437));
                path.cubic_bezier_to(
                    view.point(32.0267, 37.3902),
                    view.point(31.7422, 38.1725),
                    view.point(31.8607, 37.7932),
                );
                path.cubic_bezier_to(
                    view.point(33.0226, 32.7671),
                    view.point(32.8092, 35.3039),
                    view.point(33.2123, 33.4309),
                );
                path.cubic_bezier_to(
                    view.point(31.1019, 31.6765),
                    view.point(32.8803, 32.1981),
                    view.point(32.6195, 32.0558),
                );
                path.cubic_bezier_to(
                    view.point(25.6482, 29.0449),
                    view.point(28.4462, 31.0364),
                    view.point(27.3555, 30.5148),
                );
                path.cubic_bezier_to(
                    view.point(23.1822, 27.9306),
                    view.point(24.6049, 28.1677),
                    view.point(24.2966, 28.0254),
                );
                path.cubic_bezier_to(
                    view.point(17.4439, 25.8443),
                    view.point(21.6409, 27.7646),
                    view.point(18.5346, 26.6504),
                );
                path.cubic_bezier_to(
                    view.point(16.8037, 25.4887),
                    view.point(17.1831, 25.6546),
                    view.point(16.8985, 25.4887),
                );
                path.cubic_bezier_to(
                    view.point(16.1397, 25.1093),
                    view.point(16.7325, 25.4887),
                    view.point(16.4243, 25.3227),
                );
                path.cubic_bezier_to(
                    view.point(15.0253, 24.3744),
                    view.point(15.8552, 24.9197),
                    view.point(15.3572, 24.5877),
                );
                path.cubic_bezier_to(
                    view.point(11.7293, 21.9087),
                    view.point(14.1242, 23.8528),
                    view.point(12.417, 22.5725),
                );
                path.line_to(view.point(11.1365, 21.3397));
                path.line_to(view.point(11.6582, 21.6242));
                path.cubic_bezier_to(
                    view.point(12.6067, 22.2643),
                    view.point(11.9427, 21.7665),
                    view.point(12.3696, 22.0747),
                );
                path.cubic_bezier_to(
                    view.point(14.0531, 23.2364),
                    view.point(12.8438, 22.4777),
                    view.point(13.5077, 22.9045),
                );
                path.cubic_bezier_to(
                    view.point(15.4521, 24.0899),
                    view.point(14.5985, 23.5683),
                    view.point(15.215, 23.9476),
                );
                path.cubic_bezier_to(
                    view.point(20.4553, 26.0102),
                    view.point(16.3531, 24.6589),
                    view.point(19.2223, 25.7494),
                );
                path.cubic_bezier_to(
                    view.point(22.8265, 25.9154),
                    view.point(21.8306, 26.2947),
                    view.point(22.4708, 26.271),
                );
                path.cubic_bezier_to(
                    view.point(22.945, 24.6115),
                    view.point(23.1822, 25.5835),
                    view.point(23.1822, 25.5361),
                );
                path.cubic_bezier_to(
                    view.point(22.7079, 23.189),
                    view.point(22.8739, 24.2321),
                    view.point(22.7554, 23.592),
                );
                path.cubic_bezier_to(
                    view.point(22.2337, 21.0078),
                    view.point(22.6605, 22.8096),
                    view.point(22.4471, 21.8139),
                );
                path.cubic_bezier_to(
                    view.point(21.7595, 19.1111),
                    view.point(22.0203, 20.2017),
                    view.point(21.8069, 19.3482),
                );
                path.cubic_bezier_to(
                    view.point(20.3367, 15.3178),
                    view.point(21.6409, 18.5896),
                    view.point(20.6924, 16.0291),
                );
                path.cubic_bezier_to(
                    view.point(20.4553, 14.0613),
                    view.point(19.9574, 14.5592),
                    view.point(20.0048, 14.1324),
                );
                path.cubic_bezier_to(
                    view.point(23.3956, 16.1476),
                    view.point(21.0718, 13.9902),
                    view.point(21.8069, 14.5118),
                );
                path.cubic_bezier_to(
                    view.point(27.3317, 19.1586),
                    view.point(25.174, 17.9494),
                    view.point(25.9565, 18.5659),
                );
                path.cubic_bezier_to(
                    view.point(29.4895, 19.206),
                    view.point(28.5173, 19.7039),
                    view.point(28.8493, 19.7039),
                );
                path.cubic_bezier_to(
                    view.point(31.505, 19.3008),
                    view.point(30.5803, 18.3762),
                    view.point(30.8885, 18.3999),
                );
                path.cubic_bezier_to(
                    view.point(33.8999, 19.9883),
                    view.point(32.003, 20.0121),
                    view.point(32.4772, 20.1543),
                );
                path.cubic_bezier_to(
                    view.point(36.2237, 18.6844),
                    view.point(35.133, 19.8224),
                    view.point(35.4886, 19.609),
                );
                path.cubic_bezier_to(
                    view.point(37.2433, 17.8072),
                    view.point(36.5082, 18.3288),
                    view.point(36.9351, 17.9732),
                );
                path.line_to(view.point(37.765, 17.5701));
                path.line_to(view.point(38.5712, 17.8072));
                path.cubic_bezier_to(
                    view.point(41.8197, 20.913),
                    view.point(40.1836, 18.3051),
                    view.point(41.3692, 19.4431),
                );
                path.cubic_bezier_to(
                    view.point(41.8908, 26.3659),
                    view.point(42.4599, 23.0467),
                    view.point(42.4836, 24.4929),
                );
                path.cubic_bezier_to(
                    view.point(36.7217, 31.5105),
                    view.point(41.1321, 28.7367),
                    view.point(39.5908, 30.2777),
                );
                path.cubic_bezier_to(
                    view.point(34.4453, 34.4029),
                    view.point(34.6587, 32.4115),
                    view.point(34.7061, 32.3403),
                );
                path.cubic_bezier_to(
                    view.point(33.4494, 37.0346),
                    view.point(34.2556, 35.9677),
                    view.point(34.1845, 36.1573),
                );
                path.cubic_bezier_to(
                    view.point(31.7422, 38.2437),
                    view.point(32.5721, 38.0777),
                    view.point(31.7422, 38.6704),
                );
                path.close();
                path.move_to(view.point(51.0673, 36.2522));
                path.cubic_bezier_to(
                    view.point(49.8817, 33.4546),
                    view.point(50.5694, 35.4935),
                    view.point(50.4271, 35.1616),
                );
                path.cubic_bezier_to(
                    view.point(48.957, 26.6266),
                    view.point(49.1941, 31.3683),
                    view.point(49.0993, 30.5385),
                );
                path.cubic_bezier_to(
                    view.point(48.791, 22.9282),
                    view.point(48.9096, 24.8722),
                    view.point(48.8147, 23.2127),
                );
                path.cubic_bezier_to(
                    view.point(49.2889, 22.0747),
                    view.point(48.6961, 22.3355),
                    view.point(48.8621, 22.0747),
                );
                path.cubic_bezier_to(
                    view.point(51.921, 25.6309),
                    view.point(50.3323, 22.0747),
                    view.point(51.589, 23.758),
                );
                path.cubic_bezier_to(
                    view.point(51.7787, 36.5604),
                    view.point(52.0869, 26.5555),
                    view.point(51.9684, 36.3233),
                );
                path.cubic_bezier_to(
                    view.point(51.0673, 36.2522),
                    view.point(51.5653, 36.8212),
                    view.point(51.3519, 36.7263),
                );
                path.close();
                path.move_to(view.point(8.76535, 24.6589));
                path.cubic_bezier_to(
                    view.point(3.99926, 18.7792),
                    view.point(7.03438, 23.8054),
                    view.point(3.99926, 20.0595),
                );
                path.cubic_bezier_to(
                    view.point(4.82918, 19.2534),
                    view.point(3.99926, 18.1628),
                    view.point(4.35494, 18.3762),
                );
                path.cubic_bezier_to(
                    view.point(7.98286, 23.3075),
                    view.point(5.65909, 20.7944),
                    view.point(6.51272, 21.9087),
                );
                path.cubic_bezier_to(
                    view.point(8.76535, 24.6589),
                    view.point(9.69011, 24.9908),
                    view.point(9.78496, 25.1804),
                );
                path.close();
                path.move_to(view.point(17.4676, 23.8054));
                path.cubic_bezier_to(
                    view.point(11.9665, 20.1069),
                    view.point(15.4521, 22.8808),
                    view.point(13.2232, 21.3871),
                );
                path.cubic_bezier_to(
                    view.point(9.95094, 17.2856),
                    view.point(11.5871, 19.7039),
                    view.point(10.7334, 18.5184),
                );
                path.cubic_bezier_to(
                    view.point(7.20037, 14.4169),
                    view.point(8.31482, 14.7014),
                    view.point(8.19626, 14.5829),
                );
                path.cubic_bezier_to(
                    view.point(4.78175, 11.975),
                    view.point(6.2756, 14.2747),
                    view.point(6.32303, 14.3221),
                );
                path.cubic_bezier_to(
                    view.point(3.57245, 9.65157),
                    view.point(3.61987, 10.268),
                    view.point(3.52502, 10.0546),
                );
                path.cubic_bezier_to(
                    view.point(5.63538, 6.37984),
                    view.point(3.64358, 9.17741),
                    view.point(4.4735, 7.82604),
                );
                path.cubic_bezier_to(
                    view.point(6.65499, 4.74397),
                    view.point(6.03848, 5.88197),
                    view.point(6.48901, 5.14701),
                );
                path.cubic_bezier_to(
                    view.point(8.17255, 4.60172),
                    view.point(7.10552, 3.72452),
                    view.point(7.41377, 3.70081),
                );
                path.cubic_bezier_to(
                    view.point(11.6108, 7.77862),
                    view.point(9.21587, 5.78713),
                    view.point(10.1169, 6.64063),
                );
                path.cubic_bezier_to(
                    view.point(15.5944, 11.6194),
                    view.point(13.8397, 9.46191),
                    view.point(13.9582, 9.55674),
                );
                path.cubic_bezier_to(
                    view.point(17.6336, 14.085),
                    view.point(16.3769, 12.5914),
                    view.point(17.3016, 13.7057),
                );
                path.cubic_bezier_to(
                    view.point(19.08, 17.2145),
                    view.point(18.5821, 15.1282),
                    view.point(18.8666, 15.7446),
                );
                path.cubic_bezier_to(
                    view.point(19.4831, 19.4194),
                    view.point(19.1749, 17.902),
                    view.point(19.3408, 18.8978),
                );
                path.cubic_bezier_to(
                    view.point(20.1708, 23.4497),
                    view.point(19.7914, 20.6522),
                    view.point(20.1708, 22.8333),
                );
                path.cubic_bezier_to(
                    view.point(17.4676, 23.8054),
                    view.point(20.1708, 24.4929),
                    view.point(19.246, 24.6115),
                );
                path.close();
                path.move_to(view.point(1.01157, 16.4084));
                path.cubic_bezier_to(
                    view.point(0.205362, 13.4212),
                    view.point(0.110515, 15.6023),
                    view.point(-0.268876, 14.2035),
                );
                path.cubic_bezier_to(
                    view.point(1.72292, 12.5677),
                    view.point(0.513616, 12.9233),
                    view.point(1.31982, 12.4728),
                );
                path.cubic_bezier_to(
                    view.point(3.8807, 15.223),
                    view.point(2.38685, 12.7336),
                    view.point(3.8807, 14.5592),
                );
                path.cubic_bezier_to(
                    view.point(1.01157, 16.4084),
                    view.point(3.8807, 16.3847),
                    view.point(1.91262, 17.1908),
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
