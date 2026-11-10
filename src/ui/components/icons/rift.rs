use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct RiftIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl RiftIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for RiftIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(21.455, 51.4893));
                path.cubic_bezier_to(
                    view.point(20.2486, 50.5434),
                    view.point(21.1002, 51.1819),
                    view.point(20.5798, 50.7562),
                );
                path.cubic_bezier_to(
                    view.point(19.7282, 48.0842),
                    view.point(19.1132, 49.7395),
                    view.point(19.0422, 49.4321),
                );
                path.cubic_bezier_to(
                    view.point(20.3669, 46.7837),
                    view.point(20.0121, 47.564),
                    view.point(20.2959, 46.9729),
                );
                path.cubic_bezier_to(
                    view.point(21.455, 46.1216),
                    view.point(20.5561, 46.2872),
                    view.point(20.84, 46.1216),
                );
                path.cubic_bezier_to(
                    view.point(22.5667, 46.6655),
                    view.point(22.07, 46.1453),
                    view.point(22.4248, 46.3108),
                );
                path.cubic_bezier_to(
                    view.point(22.5667, 51.8439),
                    view.point(22.685, 47.0202),
                    view.point(22.685, 51.6075),
                );
                path.cubic_bezier_to(
                    view.point(21.455, 51.4893),
                    view.point(22.4011, 52.1277),
                    view.point(22.1173, 52.0331),
                );
                path.close();
                path.move_to(view.point(29.4028, 51.7021));
                path.cubic_bezier_to(
                    view.point(29.4028, 46.5),
                    view.point(29.2136, 51.5129),
                    view.point(29.2136, 46.8783),
                );
                path.cubic_bezier_to(
                    view.point(31.3188, 46.5473),
                    view.point(29.8049, 45.696),
                    view.point(31.0586, 45.7433),
                );
                path.cubic_bezier_to(
                    view.point(32.0048, 48.0369),
                    view.point(31.3661, 46.7364),
                    view.point(31.6736, 47.3985),
                );
                path.cubic_bezier_to(
                    view.point(32.2886, 49.834),
                    view.point(32.6671, 49.3611),
                    view.point(32.6908, 49.4794),
                );
                path.cubic_bezier_to(
                    view.point(29.7103, 51.8439),
                    view.point(31.2715, 50.8035),
                    view.point(29.9232, 51.8439),
                );
                path.cubic_bezier_to(
                    view.point(29.4028, 51.7021),
                    view.point(29.6157, 51.8439),
                    view.point(29.4738, 51.773),
                );
                path.close();
                path.move_to(view.point(43.8083, 50.307));
                path.cubic_bezier_to(
                    view.point(43.7136, 45.8142),
                    view.point(43.7373, 50.2124),
                    view.point(43.7136, 49.101),
                );
                path.cubic_bezier_to(
                    view.point(43.2406, 38.082),
                    view.point(43.7136, 40.8722),
                    view.point(43.619, 39.217),
                );
                path.cubic_bezier_to(
                    view.point(43.075, 36.7815),
                    view.point(43.0513, 37.5382),
                    view.point(43.0277, 37.3726),
                );
                path.cubic_bezier_to(
                    view.point(42.8621, 35.481),
                    view.point(43.1223, 36.143),
                    view.point(43.1223, 36.0485),
                );
                path.cubic_bezier_to(
                    view.point(42.6019, 34.6297),
                    view.point(42.7202, 35.1499),
                    view.point(42.6019, 34.7716),
                );
                path.cubic_bezier_to(
                    view.point(43.2642, 33.14),
                    view.point(42.6019, 34.3696),
                    view.point(42.9094, 33.6602),
                );
                path.cubic_bezier_to(
                    view.point(43.1933, 30.0897),
                    view.point(44.305, 31.6267),
                    view.point(44.305, 30.9882),
                );
                path.cubic_bezier_to(
                    view.point(40.7569, 28.6236),
                    view.point(42.46, 29.4985),
                    view.point(42.2707, 29.3803),
                );
                path.cubic_bezier_to(
                    view.point(38.7936, 27.3468),
                    view.point(39.243, 27.8433),
                    view.point(38.7936, 27.5596),
                );
                path.cubic_bezier_to(
                    view.point(41.3246, 27.8197),
                    view.point(38.7936, 27.0867),
                    view.point(40.2601, 27.3704),
                );
                path.cubic_bezier_to(
                    view.point(42.4363, 28.198),
                    view.point(41.6321, 27.9616),
                    view.point(42.1288, 28.1271),
                );
                path.cubic_bezier_to(
                    view.point(43.5008, 28.6473),
                    view.point(42.7438, 28.2926),
                    view.point(43.2169, 28.4818),
                );
                path.cubic_bezier_to(
                    view.point(45.2275, 28.1034),
                    view.point(44.4233, 29.1438),
                    view.point(45.0146, 28.9783),
                );
                path.cubic_bezier_to(
                    view.point(45.9608, 25.7861),
                    view.point(45.3931, 27.4177),
                    view.point(45.8425, 26.0226),
                );
                path.cubic_bezier_to(
                    view.point(46.5522, 23.2087),
                    view.point(46.0791, 25.5497),
                    view.point(46.3629, 24.2728),
                );
                path.cubic_bezier_to(
                    view.point(46.7414, 22.3102),
                    view.point(46.5995, 22.8304),
                    view.point(46.6941, 22.4284),
                );
                path.cubic_bezier_to(
                    view.point(47.0016, 21.1279),
                    view.point(46.7887, 22.1919),
                    view.point(46.907, 21.6481),
                );
                path.cubic_bezier_to(
                    view.point(47.2854, 19.8746),
                    view.point(47.1199, 20.584),
                    view.point(47.2381, 20.0165),
                );
                path.cubic_bezier_to(
                    view.point(48.0897, 19.3071),
                    view.point(47.4747, 19.2835),
                    view.point(47.9241, 18.9524),
                );
                path.cubic_bezier_to(
                    view.point(48.137, 24.8639),
                    view.point(48.1133, 19.4017),
                    view.point(48.137, 21.8845),
                );
                path.cubic_bezier_to(
                    view.point(48.752, 32.5016),
                    view.point(48.1133, 31.0119),
                    view.point(48.137, 31.4848),
                );
                path.cubic_bezier_to(
                    view.point(49.6036, 35.7647),
                    view.point(49.5799, 33.873),
                    view.point(49.7455, 34.6061),
                );
                path.cubic_bezier_to(
                    view.point(48.0424, 38.7204),
                    view.point(49.438, 36.9706),
                    view.point(49.1068, 37.6091),
                );
                path.cubic_bezier_to(
                    view.point(46.2683, 42.4092),
                    view.point(47.0726, 39.7609),
                    view.point(46.6941, 40.5175),
                );
                path.cubic_bezier_to(
                    view.point(45.2275, 46.2399),
                    view.point(46.0081, 43.5442),
                    view.point(45.6296, 44.9157),
                );
                path.cubic_bezier_to(
                    view.point(44.92, 47.5877),
                    view.point(45.0856, 46.6891),
                    view.point(44.9673, 47.3039),
                );
                path.cubic_bezier_to(
                    view.point(44.2577, 50.2124),
                    view.point(44.8727, 48.2261),
                    view.point(44.4233, 49.9286),
                );
                path.cubic_bezier_to(
                    view.point(43.8083, 50.307),
                    view.point(44.1158, 50.4252),
                    view.point(43.9502, 50.4488),
                );
                path.close();
                path.move_to(view.point(8.30318, 49.503));
                path.cubic_bezier_to(
                    view.point(7.99568, 48.77),
                    view.point(8.20857, 49.2429),
                    view.point(8.06664, 48.9118),
                );
                path.cubic_bezier_to(
                    view.point(7.73548, 47.7296),
                    view.point(7.92471, 48.6281),
                    view.point(7.80644, 48.1552),
                );
                path.cubic_bezier_to(
                    view.point(7.42797, 46.2635),
                    view.point(7.68817, 47.2803),
                    view.point(7.52259, 46.6418),
                );
                path.cubic_bezier_to(
                    view.point(6.97854, 44.561),
                    view.point(7.3097, 45.9088),
                    view.point(7.12047, 45.1285),
                );
                path.cubic_bezier_to(
                    view.point(6.52911, 42.7403),
                    view.point(6.86027, 43.9935),
                    view.point(6.67104, 43.1659),
                );
                path.cubic_bezier_to(
                    view.point(6.2216, 41.6525),
                    view.point(6.38718, 42.3146),
                    view.point(6.26891, 41.8181),
                );
                path.cubic_bezier_to(
                    view.point(4.42388, 39.3589),
                    view.point(6.10333, 41.085),
                    view.point(5.84314, 40.754),
                );
                path.cubic_bezier_to(
                    view.point(2.53154, 37.0416),
                    view.point(3.00462, 37.9874),
                    view.point(2.98097, 37.9401),
                );
                path.cubic_bezier_to(
                    view.point(3.45405, 31.8395),
                    view.point(1.60902, 35.1736),
                    view.point(1.86922, 33.7548),
                );
                path.cubic_bezier_to(
                    view.point(4.37657, 23.3506),
                    view.point(4.35292, 30.7518),
                    view.point(4.40022, 30.1606),
                );
                path.cubic_bezier_to(
                    view.point(4.5185, 17.912),
                    view.point(4.35292, 18.4086),
                    view.point(4.37657, 18.0539),
                );
                path.cubic_bezier_to(
                    view.point(4.80235, 17.8647),
                    view.point(4.61311, 17.7938),
                    view.point(4.68408, 17.7938),
                );
                path.cubic_bezier_to(
                    view.point(5.53563, 20.2766),
                    view.point(5.03889, 18.0066),
                    view.point(5.3937, 19.1889),
                );
                path.cubic_bezier_to(
                    view.point(5.9141, 22.0737),
                    view.point(5.60659, 20.7968),
                    view.point(5.77217, 21.6244),
                );
                path.cubic_bezier_to(
                    view.point(6.26891, 23.3742),
                    view.point(6.05602, 22.523),
                    view.point(6.2216, 23.1141),
                );
                path.cubic_bezier_to(
                    view.point(6.57642, 24.3437),
                    view.point(6.31622, 23.6343),
                    view.point(6.45815, 24.0836),
                );
                path.cubic_bezier_to(
                    view.point(6.86027, 25.6442),
                    view.point(6.71834, 24.6511),
                    view.point(6.81296, 25.0767),
                );
                path.cubic_bezier_to(
                    view.point(7.47528, 27.7487),
                    view.point(6.95489, 26.5428),
                    view.point(7.09681, 27.0157),
                );
                path.cubic_bezier_to(
                    view.point(9.36762, 28.198),
                    view.point(7.94837, 28.7182),
                    view.point(8.16126, 28.7655),
                );
                path.cubic_bezier_to(
                    view.point(12.7738, 27.0157),
                    view.point(10.1719, 27.8197),
                    view.point(12.1825, 27.1103),
                );
                path.cubic_bezier_to(
                    view.point(13.8146, 27.1339),
                    view.point(13.105, 26.9448),
                    view.point(13.8146, 27.0394),
                );
                path.cubic_bezier_to(
                    view.point(12.1825, 28.1744),
                    view.point(13.8146, 27.2995),
                    view.point(12.7029, 28.0088),
                );
                path.cubic_bezier_to(
                    view.point(9.95898, 29.1675),
                    view.point(11.7331, 28.3162),
                    view.point(10.8342, 28.6946),
                );
                path.cubic_bezier_to(
                    view.point(8.7053, 30.1843),
                    view.point(9.4859, 29.404),
                    view.point(8.89454, 29.9005),
                );
                path.cubic_bezier_to(
                    view.point(9.20204, 32.1232),
                    view.point(8.44511, 30.6099),
                    view.point(8.53972, 30.9646),
                );
                path.cubic_bezier_to(
                    view.point(9.58051, 34.8898),
                    view.point(9.93533, 33.3765),
                    view.point(9.95898, 33.6129),
                );
                path.cubic_bezier_to(
                    view.point(9.32032, 35.9775),
                    view.point(9.43859, 35.3864),
                    view.point(9.32032, 35.8593),
                );
                path.cubic_bezier_to(
                    view.point(9.17839, 37.0179),
                    view.point(9.32032, 36.0721),
                    view.point(9.24935, 36.545),
                );
                path.cubic_bezier_to(
                    view.point(9.88802, 38.2712),
                    view.point(8.9655, 38.3421),
                    view.point(9.13108, 38.6022),
                );
                path.cubic_bezier_to(
                    view.point(11.3782, 38.4603),
                    view.point(10.4321, 38.0347),
                    view.point(10.574, 38.0347),
                );
                path.cubic_bezier_to(
                    view.point(12.7029, 39.0278),
                    view.point(11.7567, 38.6732),
                    view.point(12.3717, 38.9096),
                );
                path.cubic_bezier_to(
                    view.point(14.0275, 39.5008),
                    view.point(13.034, 39.1224),
                    view.point(13.6254, 39.3352),
                );
                path.cubic_bezier_to(
                    view.point(15.6597, 39.9973),
                    view.point(14.406, 39.6663),
                    view.point(15.1393, 39.8791),
                );
                path.cubic_bezier_to(
                    view.point(17.5993, 40.423),
                    view.point(16.1564, 40.1156),
                    view.point(17.0316, 40.3047),
                );
                path.cubic_bezier_to(
                    view.point(29.2845, 41.156),
                    view.point(20.5798, 41.0378),
                    view.point(27.4395, 41.487),
                );
                path.cubic_bezier_to(
                    view.point(31.0823, 40.9432),
                    view.point(29.5447, 41.1087),
                    view.point(30.349, 41.0141),
                );
                path.cubic_bezier_to(
                    view.point(35.5293, 40.2101),
                    view.point(32.2177, 40.8249),
                    view.point(32.9746, 40.7067),
                );
                path.cubic_bezier_to(
                    view.point(36.3335, 39.9973),
                    view.point(35.7895, 40.1629),
                    view.point(36.1443, 40.0683),
                );
                path.cubic_bezier_to(
                    view.point(37.2324, 39.7372),
                    view.point(36.5227, 39.9264),
                    view.point(36.9485, 39.8082),
                );
                path.cubic_bezier_to(
                    view.point(38.4624, 39.3116),
                    view.point(37.5399, 39.6663),
                    view.point(38.1076, 39.4771),
                );
                path.cubic_bezier_to(
                    view.point(40.6149, 39.0515),
                    view.point(39.2666, 38.9569),
                    view.point(40.1892, 38.8387),
                );
                path.cubic_bezier_to(
                    view.point(42.5073, 41.4634),
                    view.point(41.2773, 39.4062),
                    view.point(42.5073, 40.9668),
                );
                path.cubic_bezier_to(
                    view.point(36.6174, 43.8753),
                    view.point(42.5073, 42.3146),
                    view.point(40.6386, 43.0713),
                );
                path.cubic_bezier_to(
                    view.point(34.4412, 44.2063),
                    view.point(35.8604, 44.0171),
                    view.point(34.8906, 44.1827),
                );
                path.cubic_bezier_to(
                    view.point(33.1875, 44.4191),
                    view.point(34.0154, 44.2536),
                    view.point(33.4477, 44.3482),
                );
                path.cubic_bezier_to(
                    view.point(31.4844, 44.6319),
                    view.point(32.9273, 44.4901),
                    view.point(32.1467, 44.5846),
                );
                path.cubic_bezier_to(
                    view.point(29.9469, 44.7975),
                    view.point(30.7984, 44.6556),
                    view.point(30.1124, 44.7502),
                );
                path.cubic_bezier_to(
                    view.point(22.7323, 44.8447),
                    view.point(29.4265, 44.9866),
                    view.point(23.4183, 45.0103),
                );
                path.cubic_bezier_to(
                    view.point(21.3367, 44.7029),
                    view.point(22.3775, 44.7502),
                    view.point(21.8807, 44.7029),
                );
                path.cubic_bezier_to(
                    view.point(16.5822, 44.159),
                    view.point(20.083, 44.7029),
                    view.point(17.5284, 44.4191),
                );
                path.cubic_bezier_to(
                    view.point(15.1156, 43.8753),
                    view.point(16.3456, 44.0881),
                    view.point(15.6833, 43.9698),
                );
                path.cubic_bezier_to(
                    view.point(13.3889, 43.4733),
                    view.point(14.5479, 43.7807),
                    view.point(13.7673, 43.5915),
                );
                path.cubic_bezier_to(
                    view.point(11.9223, 43.0477),
                    view.point(12.9867, 43.3551),
                    view.point(12.3244, 43.1659),
                );
                path.cubic_bezier_to(
                    view.point(10.5267, 42.6457),
                    view.point(11.4965, 42.9531),
                    view.point(10.8815, 42.7639),
                );
                path.cubic_bezier_to(
                    view.point(8.82358, 46.0034),
                    view.point(8.89454, 42.0545),
                    view.point(8.77627, 42.291),
                );
                path.cubic_bezier_to(
                    view.point(8.63434, 49.9523),
                    view.point(8.87088, 48.8409),
                    view.point(8.82358, 49.9286),
                );
                path.cubic_bezier_to(
                    view.point(8.30318, 49.503),
                    view.point(8.53972, 49.9759),
                    view.point(8.44511, 49.8577),
                );
                path.close();
                path.move_to(view.point(34.2283, 47.4222));
                path.cubic_bezier_to(
                    view.point(33.5187, 46.3108),
                    view.point(33.9208, 47.233),
                    view.point(33.5187, 46.6182),
                );
                path.cubic_bezier_to(
                    view.point(34.3702, 45.6014),
                    view.point(33.5187, 45.9325),
                    view.point(33.7315, 45.7433),
                );
                path.cubic_bezier_to(
                    view.point(35.4583, 46.4763),
                    view.point(35.6948, 45.3177),
                    view.point(35.955, 45.5068),
                );
                path.cubic_bezier_to(
                    view.point(34.5594, 47.5877),
                    view.point(35.1744, 47.0202),
                    view.point(34.725, 47.5877),
                );
                path.cubic_bezier_to(
                    view.point(34.2283, 47.4222),
                    view.point(34.5358, 47.5877),
                    view.point(34.3702, 47.5167),
                );
                path.close();
                path.move_to(view.point(16.8424, 47.233));
                path.cubic_bezier_to(
                    view.point(16.0381, 45.9325),
                    view.point(16.4876, 46.9256),
                    view.point(16.0381, 46.1926),
                );
                path.cubic_bezier_to(
                    view.point(17.1262, 45.6251),
                    view.point(16.0381, 45.5541),
                    view.point(16.322, 45.4832),
                );
                path.cubic_bezier_to(
                    view.point(18.3326, 46.2162),
                    view.point(17.9305, 45.7433),
                    view.point(18.2143, 45.8852),
                );
                path.cubic_bezier_to(
                    view.point(17.8359, 47.2566),
                    view.point(18.4272, 46.4763),
                    view.point(18.238, 46.9019),
                );
                path.cubic_bezier_to(
                    view.point(16.8424, 47.233),
                    view.point(17.481, 47.5877),
                    view.point(17.2208, 47.5877),
                );
                path.close();
                path.move_to(view.point(21.7625, 39.7136));
                path.cubic_bezier_to(
                    view.point(20.154, 39.4298),
                    view.point(20.8163, 39.6663),
                    view.point(20.5798, 39.619),
                );
                path.cubic_bezier_to(
                    view.point(18.924, 39.1934),
                    view.point(19.7519, 39.2407),
                    view.point(19.539, 39.1934),
                );
                path.cubic_bezier_to(
                    view.point(17.4337, 38.5786),
                    view.point(18.0487, 39.1934),
                    view.point(17.6703, 39.0515),
                );
                path.cubic_bezier_to(
                    view.point(17.2445, 32.6907),
                    view.point(17.2918, 38.2712),
                    view.point(17.2918, 37.8928),
                );
                path.cubic_bezier_to(
                    view.point(17.1262, 26.9211),
                    view.point(17.2208, 29.4276),
                    view.point(17.1735, 27.063),
                );
                path.cubic_bezier_to(
                    view.point(17.3864, 24.793),
                    view.point(16.9843, 26.5428),
                    view.point(17.1262, 25.4078),
                );
                path.cubic_bezier_to(
                    view.point(18.5218, 25.124),
                    view.point(17.552, 24.4147),
                    view.point(17.9541, 24.5329),
                );
                path.cubic_bezier_to(
                    view.point(19.1605, 25.597),
                    view.point(18.8293, 25.4314),
                    view.point(19.0659, 25.6206),
                );
                path.cubic_bezier_to(
                    view.point(18.9949, 24.7693),
                    view.point(19.3734, 25.5497),
                    view.point(19.3024, 25.1477),
                );
                path.cubic_bezier_to(
                    view.point(18.4036, 23.8472),
                    view.point(18.853, 24.5802),
                    view.point(18.5928, 24.1782),
                );
                path.cubic_bezier_to(
                    view.point(17.7886, 22.7831),
                    view.point(18.238, 23.5398),
                    view.point(17.9541, 23.0432),
                );
                path.cubic_bezier_to(
                    view.point(17.2208, 19.851),
                    view.point(17.2918, 21.9555),
                    view.point(17.2208, 21.6244),
                );
                path.cubic_bezier_to(
                    view.point(17.9305, 17.983),
                    view.point(17.2208, 18.0066),
                    view.point(17.2445, 17.983),
                );
                path.cubic_bezier_to(
                    view.point(18.3326, 17.7465),
                    view.point(18.3089, 17.983),
                    view.point(18.3326, 17.9593),
                );
                path.cubic_bezier_to(
                    view.point(17.2682, 15.9967),
                    view.point(18.3326, 17.4628),
                    view.point(18.0014, 16.9425),
                );
                path.cubic_bezier_to(
                    view.point(15.6597, 11.7404),
                    view.point(15.9672, 14.3178),
                    view.point(15.6833, 13.5612),
                );
                path.cubic_bezier_to(
                    view.point(17.5047, 8.87928),
                    view.point(15.6597, 9.65959),
                    view.point(15.8252, 9.39949),
                );
                path.cubic_bezier_to(
                    view.point(19.0659, 8.7374),
                    view.point(18.6165, 8.50094),
                    view.point(18.8057, 8.50094),
                );
                path.cubic_bezier_to(
                    view.point(19.3734, 10.2744),
                    view.point(19.3024, 8.95022),
                    view.point(19.3261, 9.0448),
                );
                path.cubic_bezier_to(
                    view.point(20.9109, 11.6695),
                    view.point(19.468, 11.8823),
                    view.point(19.7992, 12.1897),
                );
                path.cubic_bezier_to(
                    view.point(21.6442, 11.4094),
                    view.point(21.1711, 11.5513),
                    view.point(21.4786, 11.433),
                );
                path.cubic_bezier_to(
                    view.point(22.6613, 9.58866),
                    view.point(22.4721, 11.2202),
                    view.point(22.6613, 10.9128),
                );
                path.cubic_bezier_to(
                    view.point(24.1279, 7.64969),
                    view.point(22.6613, 8.12261),
                    view.point(22.6377, 8.14625),
                );
                path.cubic_bezier_to(
                    view.point(25.0031, 4.07915),
                    view.point(25.0268, 7.36594),
                    view.point(24.9795, 7.50781),
                );
                path.cubic_bezier_to(
                    view.point(25.0268, 0.910597),
                    view.point(25.0031, 2.44758),
                    view.point(25.0268, 1.02883),
                );
                path.cubic_bezier_to(
                    view.point(25.9966, 0.0120516),
                    view.point(25.0268, 0.508616),
                    view.point(25.5235, 0.0593434),
                );
                path.cubic_bezier_to(
                    view.point(28.6459, 0.248511),
                    view.point(26.3751, -0.0352403),
                    view.point(27.4395, 0.0593435),
                );
                path.cubic_bezier_to(
                    view.point(30.7511, 0.579554),
                    view.point(30.1124, 0.461324),
                    view.point(30.3726, 0.508616),
                );
                path.cubic_bezier_to(
                    view.point(33.1875, 1.38352),
                    view.point(31.5317, 0.72143),
                    view.point(32.8563, 1.1707),
                );
                path.cubic_bezier_to(
                    view.point(35.1745, 2.06925),
                    view.point(33.6606, 1.71456),
                    view.point(33.9917, 1.80914),
                );
                path.cubic_bezier_to(
                    view.point(39.3376, 2.0456),
                    view.point(36.5464, 2.353),
                    view.point(38.2968, 2.353),
                );
                path.cubic_bezier_to(
                    view.point(42.46, 2.54217),
                    view.point(40.3074, 1.73821),
                    view.point(40.7096, 1.80914),
                );
                path.cubic_bezier_to(
                    view.point(44.6125, 4.71759),
                    view.point(43.5481, 2.99144),
                    view.point(44.7308, 4.19738),
                );
                path.cubic_bezier_to(
                    view.point(43.4771, 4.55207),
                    view.point(44.5415, 5.00134),
                    view.point(44.3523, 4.9777),
                );
                path.line_to(view.point(42.7202, 4.19738));
                path.line_to(view.point(41.3246, 4.19738));
                path.cubic_bezier_to(
                    view.point(38.4151, 4.9777),
                    view.point(39.5268, 4.19738),
                    view.point(39.1957, 4.29197),
                );
                path.cubic_bezier_to(
                    view.point(35.6002, 6.77479),
                    view.point(37.4926, 5.78166),
                    view.point(36.6883, 6.30187),
                );
                path.cubic_bezier_to(
                    view.point(32.7144, 7.48417),
                    view.point(34.5121, 7.24771),
                    view.point(34.2046, 7.31865),
                );
                path.cubic_bezier_to(
                    view.point(30.4672, 7.34229),
                    view.point(31.7209, 7.6024),
                    view.point(31.5553, 7.6024),
                );
                path.cubic_bezier_to(
                    view.point(28.7641, 7.295),
                    view.point(29.8286, 7.17677),
                    view.point(29.0007, 7.15313),
                );
                path.cubic_bezier_to(
                    view.point(28.8351, 7.95709),
                    view.point(28.5276, 7.41323),
                    view.point(28.5513, 7.6024),
                );
                path.cubic_bezier_to(
                    view.point(29.2845, 9.51772),
                    view.point(29.2372, 8.45365),
                    view.point(29.2845, 8.59553),
                );
                path.cubic_bezier_to(
                    view.point(29.663, 11.2202),
                    view.point(29.2845, 10.5345),
                    view.point(29.3792, 10.9365),
                );
                path.cubic_bezier_to(
                    view.point(31.4844, 11.7641),
                    view.point(29.9469, 11.4803),
                    view.point(30.7274, 11.7168),
                );
                path.cubic_bezier_to(
                    view.point(32.3596, 10.298),
                    view.point(32.3596, 11.835),
                    view.point(32.4069, 11.7641),
                );
                path.cubic_bezier_to(
                    view.point(32.5252, 8.87928),
                    view.point(32.3123, 9.11574),
                    view.point(32.3123, 9.11574),
                );
                path.cubic_bezier_to(
                    view.point(34.5831, 8.99751),
                    view.point(32.8327, 8.54824),
                    view.point(33.3294, 8.57188),
                );
                path.cubic_bezier_to(
                    view.point(36.0733, 9.99064),
                    view.point(35.6948, 9.3522),
                    view.point(35.9314, 9.54136),
                );
                path.cubic_bezier_to(
                    view.point(36.0497, 13.6321),
                    view.point(36.1679, 10.4163),
                    view.point(36.1679, 13.3247),
                );
                path.cubic_bezier_to(
                    view.point(33.9444, 16.848),
                    view.point(35.955, 13.9159),
                    view.point(34.4648, 16.1622),
                );
                path.cubic_bezier_to(
                    view.point(33.9208, 18.1721),
                    view.point(33.1875, 17.7938),
                    view.point(33.1875, 18.1012),
                );
                path.cubic_bezier_to(
                    view.point(34.5121, 20.182),
                    view.point(34.5358, 18.2431),
                    view.point(34.5594, 18.2667),
                );
                path.cubic_bezier_to(
                    view.point(33.3058, 23.9654),
                    view.point(34.4885, 22.1446),
                    view.point(34.5121, 22.0737),
                );
                path.cubic_bezier_to(
                    view.point(32.7144, 25.6915),
                    view.point(32.5252, 25.195),
                    view.point(32.3596, 25.6442),
                );
                path.cubic_bezier_to(
                    view.point(33.3294, 25.2423),
                    view.point(32.8327, 25.6915),
                    view.point(33.0219, 25.5497),
                );
                path.cubic_bezier_to(
                    view.point(34.3702, 24.7457),
                    view.point(33.8025, 24.7457),
                    view.point(34.1337, 24.6038),
                );
                path.cubic_bezier_to(
                    view.point(34.3229, 37.8692),
                    view.point(34.5831, 24.8876),
                    view.point(34.5594, 37.3963),
                );
                path.cubic_bezier_to(
                    view.point(33.2111, 38.6968),
                    view.point(34.1337, 38.2948),
                    view.point(33.7315, 38.5786),
                );
                path.cubic_bezier_to(
                    view.point(30.5146, 39.4298),
                    view.point(32.0048, 38.9569),
                    view.point(31.035, 39.2407),
                );
                path.cubic_bezier_to(
                    view.point(26.8008, 39.4771),
                    view.point(29.5684, 39.7845),
                    view.point(27.9362, 39.8082),
                );
                path.line_to(view.point(25.9729, 39.2643));
                path.line_to(view.point(25.2397, 39.4771));
                path.cubic_bezier_to(
                    view.point(21.7625, 39.7136),
                    view.point(24.4118, 39.7372),
                    view.point(23.4892, 39.8082),
                );
                path.close();
                path.move_to(view.point(27.203, 28.1744));
                path.cubic_bezier_to(
                    view.point(28.0072, 26.9211),
                    view.point(28.1018, 28.0325),
                    view.point(28.1491, 27.9379),
                );
                path.cubic_bezier_to(
                    view.point(27.9126, 23.9417),
                    view.point(27.9599, 26.4482),
                    view.point(27.9126, 25.124),
                );
                path.cubic_bezier_to(
                    view.point(27.0374, 20.5604),
                    view.point(27.9126, 21.388),
                    view.point(27.9362, 21.4353),
                );
                path.cubic_bezier_to(
                    view.point(24.8139, 20.6313),
                    view.point(26.0676, 19.6145),
                    view.point(25.76, 19.6145),
                );
                path.cubic_bezier_to(
                    view.point(23.9623, 21.9318),
                    view.point(24.2698, 21.1752),
                    view.point(24.0806, 21.4826),
                );
                path.cubic_bezier_to(
                    view.point(24.0096, 24.7221),
                    view.point(23.8914, 22.2865),
                    view.point(23.915, 23.8472),
                );
                path.cubic_bezier_to(
                    view.point(23.9623, 26.33),
                    view.point(24.0806, 25.3605),
                    view.point(24.0806, 25.6442),
                );
                path.cubic_bezier_to(
                    view.point(24.7429, 28.1744),
                    view.point(23.6548, 28.1271),
                    view.point(23.6312, 28.0561),
                );
                path.cubic_bezier_to(
                    view.point(25.5235, 28.2926),
                    view.point(25.145, 28.2217),
                    view.point(25.4998, 28.2689),
                );
                path.cubic_bezier_to(
                    view.point(27.203, 28.1744),
                    view.point(25.5945, 28.3635),
                    view.point(26.3277, 28.2926),
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
