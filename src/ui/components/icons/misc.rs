use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct MiscIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl MiscIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for MiscIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(22.7236, 51.872));
                path.cubic_bezier_to(
                    view.point(19.1991, 51.47),
                    view.point(21.8011, 51.8247),
                    view.point(20.1926, 51.6355),
                );
                path.cubic_bezier_to(
                    view.point(16.5971, 51.0679),
                    view.point(18.182, 51.2808),
                    view.point(16.9992, 51.1152),
                );
                path.cubic_bezier_to(
                    view.point(14.5629, 50.5477),
                    view.point(16.1713, 51.0206),
                    view.point(15.2725, 50.7842),
                );
                path.cubic_bezier_to(
                    view.point(12.6232, 50.0037),
                    view.point(13.8769, 50.3348),
                    view.point(13.0017, 50.0747),
                );
                path.cubic_bezier_to(
                    view.point(11.2276, 49.4835),
                    view.point(12.2684, 49.9328),
                    view.point(11.6297, 49.6963),
                );
                path.cubic_bezier_to(
                    view.point(10.0449, 49.0341),
                    view.point(10.8491, 49.2943),
                    view.point(10.3051, 49.0814),
                );
                path.cubic_bezier_to(
                    view.point(9.02775, 48.5848),
                    view.point(9.78469, 48.9868),
                    view.point(9.3116, 48.774),
                );
                path.cubic_bezier_to(
                    view.point(7.916, 48.0172),
                    view.point(8.72024, 48.3956),
                    view.point(8.2235, 48.1355),
                );
                path.cubic_bezier_to(
                    view.point(6.66232, 47.2368),
                    view.point(7.58484, 47.9226),
                    view.point(7.01713, 47.5679),
                );
                path.line_to(view.point(6.0, 46.6219));
                path.line_to(view.point(6.21289, 44.1625));
                path.cubic_bezier_to(
                    view.point(6.80425, 40.5442),
                    view.point(6.37847, 42.2232),
                    view.point(6.52039, 41.4428),
                );
                path.cubic_bezier_to(
                    view.point(7.34829, 38.3448),
                    view.point(6.99348, 39.9293),
                    view.point(7.25368, 38.9124),
                );
                path.cubic_bezier_to(
                    view.point(7.86869, 36.7603),
                    view.point(7.46657, 37.7536),
                    view.point(7.70311, 37.0441),
                );
                path.cubic_bezier_to(
                    view.point(8.36543, 35.365),
                    view.point(8.01061, 36.4765),
                    view.point(8.24716, 35.838),
                );
                path.cubic_bezier_to(
                    view.point(8.81486, 34.1353),
                    view.point(8.46005, 34.8921),
                    view.point(8.67293, 34.3481),
                );
                path.cubic_bezier_to(
                    view.point(9.3116, 33.0001),
                    view.point(8.95679, 33.9461),
                    view.point(9.19333, 33.4258),
                );
                path.cubic_bezier_to(
                    view.point(9.8793, 31.7704),
                    view.point(9.45353, 32.5745),
                    view.point(9.69007, 32.0305),
                );
                path.cubic_bezier_to(
                    view.point(10.376, 30.7535),
                    view.point(10.0449, 31.5339),
                    view.point(10.2814, 31.0609),
                );
                path.cubic_bezier_to(
                    view.point(10.8964, 29.7839),
                    view.point(10.4943, 30.4461),
                    view.point(10.7309, 29.9967),
                );
                path.cubic_bezier_to(
                    view.point(11.3695, 28.9325),
                    view.point(11.0857, 29.5474),
                    view.point(11.2749, 29.169),
                );
                path.cubic_bezier_to(
                    view.point(11.8426, 28.0812),
                    view.point(11.4405, 28.7197),
                    view.point(11.6534, 28.3413),
                );
                path.cubic_bezier_to(
                    view.point(12.4103, 27.2062),
                    view.point(12.0318, 27.8447),
                    view.point(12.292, 27.4426),
                );
                path.cubic_bezier_to(
                    view.point(14.4446, 24.1081),
                    view.point(12.6232, 26.7805),
                    view.point(13.593, 25.3142),
                );
                path.cubic_bezier_to(
                    view.point(15.3434, 22.8074),
                    view.point(14.6575, 23.8007),
                    view.point(15.0596, 23.2331),
                );
                path.cubic_bezier_to(
                    view.point(17.0229, 21.1284),
                    view.point(15.8638, 22.098),
                    view.point(16.81, 21.1284),
                );
                path.cubic_bezier_to(
                    view.point(17.6852, 21.5068),
                    view.point(17.0702, 21.1284),
                    view.point(17.3777, 21.2939),
                );
                path.cubic_bezier_to(
                    view.point(19.1281, 22.0743),
                    view.point(17.9927, 21.6959),
                    view.point(18.6314, 21.9561),
                );
                path.cubic_bezier_to(
                    view.point(20.7839, 22.5946),
                    view.point(19.6012, 22.1926),
                    view.point(20.3581, 22.4054),
                );
                path.cubic_bezier_to(
                    view.point(26.461, 23.2095),
                    view.point(21.8484, 22.9966),
                    view.point(23.5988, 23.1858),
                );
                path.cubic_bezier_to(
                    view.point(32.3745, 22.4527),
                    view.point(29.4177, 23.2095),
                    view.point(30.908, 23.0203),
                );
                path.cubic_bezier_to(
                    view.point(34.6217, 22.3345),
                    view.point(33.7228, 21.9324),
                    view.point(34.1013, 21.9088),
                );
                path.cubic_bezier_to(
                    view.point(35.4023, 27.6555),
                    view.point(35.4023, 22.9966),
                    view.point(35.6152, 24.3919),
                );
                path.cubic_bezier_to(
                    view.point(35.0711, 30.8008),
                    view.point(35.3313, 28.8379),
                    view.point(35.1657, 30.2569),
                );
                path.cubic_bezier_to(
                    view.point(35.8281, 33.5914),
                    view.point(34.7873, 32.1488),
                    view.point(34.9528, 32.7164),
                );
                path.cubic_bezier_to(
                    view.point(39.3999, 35.0576),
                    view.point(36.6796, 34.4191),
                    view.point(38.2408, 35.0576),
                );
                path.cubic_bezier_to(
                    view.point(40.6772, 35.6015),
                    view.point(40.1331, 35.0576),
                    view.point(40.1804, 35.0812),
                );
                path.cubic_bezier_to(
                    view.point(41.4105, 37.1387),
                    view.point(41.1739, 36.0745),
                    view.point(41.2449, 36.24),
                );
                path.cubic_bezier_to(
                    view.point(42.262, 38.8178),
                    view.point(41.6234, 38.1793),
                    view.point(41.6943, 38.3448),
                );
                path.cubic_bezier_to(
                    view.point(42.6878, 39.6692),
                    view.point(42.4986, 39.0306),
                    view.point(42.6168, 39.2908),
                );
                path.cubic_bezier_to(
                    view.point(43.2318, 40.5678),
                    view.point(42.7824, 40.3077),
                    view.point(43.019, 40.7097),
                );
                path.cubic_bezier_to(
                    view.point(43.1136, 38.936),
                    view.point(43.4684, 40.4259),
                    view.point(43.3974, 39.5036),
                );
                path.cubic_bezier_to(
                    view.point(42.7351, 37.0914),
                    view.point(42.9007, 38.5576),
                    view.point(42.8061, 38.0374),
                );
                path.cubic_bezier_to(
                    view.point(42.1911, 34.5846),
                    view.point(42.6168, 35.6725),
                    view.point(42.5222, 35.2231),
                );
                path.cubic_bezier_to(
                    view.point(42.1674, 33.4022),
                    view.point(41.9309, 34.0643),
                    view.point(41.9072, 33.4022),
                );
                path.cubic_bezier_to(
                    view.point(42.4276, 33.7569),
                    view.point(42.262, 33.4022),
                    view.point(42.3803, 33.5677),
                );
                path.cubic_bezier_to(
                    view.point(42.9716, 35.034),
                    view.point(42.5932, 34.49),
                    view.point(42.6405, 34.6083),
                );
                path.cubic_bezier_to(
                    view.point(43.634, 36.4292),
                    view.point(43.1372, 35.2704),
                    view.point(43.4447, 35.8853),
                );
                path.cubic_bezier_to(
                    view.point(45.0059, 39.9766),
                    view.point(44.5801, 39.0306),
                    view.point(44.793, 39.5746),
                );
                path.cubic_bezier_to(
                    view.point(45.408, 41.0644),
                    view.point(45.1478, 40.2367),
                    view.point(45.3134, 40.7097),
                );
                path.cubic_bezier_to(
                    view.point(45.9048, 42.3888),
                    view.point(45.5027, 41.4192),
                    view.point(45.7155, 42.0104),
                );
                path.cubic_bezier_to(
                    view.point(45.8811, 45.0375),
                    view.point(46.8036, 44.1861),
                    view.point(46.8036, 44.6118),
                );
                path.cubic_bezier_to(
                    view.point(44.5801, 45.9361),
                    view.point(45.5736, 45.1794),
                    view.point(44.9823, 45.5814),
                );
                path.cubic_bezier_to(
                    view.point(43.2555, 46.9057),
                    view.point(44.1544, 46.2909),
                    view.point(43.563, 46.7165),
                );
                path.cubic_bezier_to(
                    view.point(42.3566, 47.5206),
                    view.point(42.9243, 47.0949),
                    view.point(42.5222, 47.3787),
                );
                path.cubic_bezier_to(
                    view.point(41.4105, 48.1355),
                    view.point(42.1911, 47.6861),
                    view.point(41.7653, 47.9463),
                );
                path.cubic_bezier_to(
                    view.point(40.6062, 48.6321),
                    view.point(41.0557, 48.301),
                    view.point(40.6772, 48.5375),
                );
                path.cubic_bezier_to(
                    view.point(39.9676, 48.9868),
                    view.point(40.5116, 48.7503),
                    view.point(40.2278, 48.8922),
                );
                path.cubic_bezier_to(
                    view.point(38.8795, 49.5544),
                    view.point(39.731, 49.0578),
                    view.point(39.2343, 49.3179),
                );
                path.cubic_bezier_to(
                    view.point(37.4839, 50.122),
                    view.point(38.501, 49.7909),
                    view.point(37.9333, 50.0274),
                );
                path.cubic_bezier_to(
                    view.point(35.6388, 50.6186),
                    view.point(37.0581, 50.1929),
                    view.point(36.2302, 50.4294),
                );
                path.cubic_bezier_to(
                    view.point(33.5572, 51.0679),
                    view.point(35.0475, 50.8078),
                    view.point(34.1249, 51.0206),
                );
                path.cubic_bezier_to(
                    view.point(30.5295, 51.47),
                    view.point(33.0132, 51.1152),
                    view.point(31.6412, 51.2808),
                );
                path.cubic_bezier_to(
                    view.point(24.5686, 51.9903),
                    view.point(28.8737, 51.7301),
                    view.point(25.1127, 52.0612),
                );
                path.cubic_bezier_to(
                    view.point(22.7236, 51.872),
                    view.point(24.4977, 51.9903),
                    view.point(23.6698, 51.943),
                );
                path.close();
                path.move_to(view.point(15.2252, 48.774));
                path.cubic_bezier_to(
                    view.point(15.1542, 48.3483),
                    view.point(15.2725, 48.6558),
                    view.point(15.2488, 48.4666),
                );
                path.cubic_bezier_to(
                    view.point(14.7521, 47.6388),
                    view.point(15.0832, 48.2537),
                    view.point(14.894, 47.9226),
                );
                path.cubic_bezier_to(
                    view.point(13.9715, 46.551),
                    view.point(14.5865, 47.3787),
                    view.point(14.2553, 46.8821),
                );
                path.cubic_bezier_to(
                    view.point(12.8124, 41.7503),
                    view.point(13.0963, 45.5104),
                    view.point(12.8124, 44.3989),
                );
                path.cubic_bezier_to(
                    view.point(13.0253, 38.4158),
                    view.point(12.8124, 40.2131),
                    view.point(12.8834, 39.1962),
                );
                path.cubic_bezier_to(
                    view.point(13.3565, 35.838),
                    view.point(13.1436, 37.7772),
                    view.point(13.2855, 36.6184),
                );
                path.cubic_bezier_to(
                    view.point(13.9242, 33.0474),
                    view.point(13.4511, 34.8921),
                    view.point(13.6403, 33.9697),
                );
                path.cubic_bezier_to(
                    view.point(14.5155, 30.8717),
                    view.point(14.1607, 32.3143),
                    view.point(14.4209, 31.3211),
                );
                path.cubic_bezier_to(
                    view.point(15.0123, 29.4292),
                    view.point(14.5865, 30.4224),
                    view.point(14.823, 29.7602),
                );
                path.cubic_bezier_to(
                    view.point(15.4617, 27.9629),
                    view.point(15.2488, 29.0035),
                    view.point(15.4144, 28.4832),
                );
                path.cubic_bezier_to(
                    view.point(15.8402, 26.7805),
                    view.point(15.5327, 27.3717),
                    view.point(15.6509, 27.0406),
                );
                path.cubic_bezier_to(
                    view.point(16.0058, 25.8345),
                    view.point(16.1713, 26.3784),
                    view.point(16.2187, 26.071),
                );
                path.cubic_bezier_to(
                    view.point(15.3671, 26.1419),
                    view.point(15.7456, 25.5744),
                    view.point(15.5563, 25.669),
                );
                path.cubic_bezier_to(
                    view.point(14.7048, 26.946),
                    view.point(15.2488, 26.4021),
                    view.point(14.965, 26.7568),
                );
                path.cubic_bezier_to(
                    view.point(13.9951, 27.9393),
                    view.point(14.3263, 27.1825),
                    view.point(14.1607, 27.419),
                );
                path.cubic_bezier_to(
                    view.point(13.4511, 28.9562),
                    view.point(13.8769, 28.3176),
                    view.point(13.6403, 28.767),
                );
                path.cubic_bezier_to(
                    view.point(12.9307, 29.9258),
                    view.point(13.2855, 29.1454),
                    view.point(13.049, 29.571),
                );
                path.cubic_bezier_to(
                    view.point(12.434, 31.0136),
                    view.point(12.8124, 30.2805),
                    view.point(12.5995, 30.7535),
                );
                path.cubic_bezier_to(
                    view.point(11.9845, 32.0069),
                    view.point(12.2684, 31.2501),
                    view.point(12.0792, 31.6995),
                );
                path.cubic_bezier_to(
                    view.point(11.5351, 33.0947),
                    view.point(11.8899, 32.338),
                    view.point(11.7007, 32.811),
                );
                path.cubic_bezier_to(
                    view.point(11.062, 34.2299),
                    view.point(11.3695, 33.3785),
                    view.point(11.1566, 33.8988),
                );
                path.cubic_bezier_to(
                    view.point(10.5653, 35.5542),
                    view.point(10.9911, 34.5846),
                    view.point(10.7545, 35.1758),
                );
                path.cubic_bezier_to(
                    view.point(10.0212, 37.2097),
                    view.point(10.376, 35.9326),
                    view.point(10.1395, 36.6894),
                );
                path.cubic_bezier_to(
                    view.point(9.50083, 39.0306),
                    view.point(9.92661, 37.7299),
                    view.point(9.69007, 38.534),
                );
                path.cubic_bezier_to(
                    view.point(9.00409, 41.5611),
                    view.point(9.26429, 39.6219),
                    view.point(9.12237, 40.3786),
                );
                path.cubic_bezier_to(
                    view.point(7.51387, 44.8956),
                    view.point(8.83851, 43.4057),
                    view.point(8.83851, 43.4057),
                );
                path.cubic_bezier_to(
                    view.point(8.43639, 46.4328),
                    view.point(6.68597, 45.8179),
                    view.point(6.96983, 46.2909),
                );
                path.cubic_bezier_to(
                    view.point(10.0212, 46.8584),
                    view.point(9.0514, 46.5037),
                    view.point(9.59545, 46.6456),
                );
                path.cubic_bezier_to(
                    view.point(11.2986, 47.3314),
                    view.point(10.376, 47.0476),
                    view.point(10.9437, 47.2605),
                );
                path.cubic_bezier_to(
                    view.point(12.7178, 48.0172),
                    view.point(12.0082, 47.5206),
                    view.point(12.5049, 47.7571),
                );
                path.cubic_bezier_to(
                    view.point(13.5221, 48.4193),
                    view.point(12.8124, 48.1118),
                    view.point(13.1672, 48.301),
                );
                path.cubic_bezier_to(
                    view.point(14.4446, 48.7976),
                    view.point(13.9005, 48.5375),
                    view.point(14.3027, 48.703),
                );
                path.cubic_bezier_to(
                    view.point(15.2252, 48.774),
                    view.point(14.7757, 49.0341),
                    view.point(15.1306, 49.0105),
                );
                path.close();
                path.move_to(view.point(38.0516, 32.8346));
                path.cubic_bezier_to(
                    view.point(37.1054, 30.044),
                    view.point(36.8688, 32.2907),
                    view.point(36.656, 31.6995),
                );
                path.cubic_bezier_to(
                    view.point(37.2946, 23.7298),
                    view.point(37.744, 27.6318),
                    view.point(37.815, 25.196),
                );
                path.cubic_bezier_to(
                    view.point(36.9161, 22.2162),
                    view.point(37.1527, 23.3041),
                    view.point(36.9635, 22.6183),
                );
                path.cubic_bezier_to(
                    view.point(36.4194, 21.0811),
                    view.point(36.8215, 21.6486),
                    view.point(36.7033, 21.3649),
                );
                path.cubic_bezier_to(
                    view.point(35.8281, 19.8986),
                    view.point(36.0409, 20.679),
                    view.point(35.7098, 20.0169),
                );
                path.cubic_bezier_to(
                    view.point(37.5548, 20.7027),
                    view.point(36.0173, 19.6858),
                    view.point(36.8452, 20.0642),
                );
                path.cubic_bezier_to(
                    view.point(38.1935, 21.8851),
                    view.point(38.4537, 21.4595),
                    view.point(38.572, 21.6723),
                );
                path.cubic_bezier_to(
                    view.point(38.8795, 24.1081),
                    view.point(37.7441, 22.1216),
                    view.point(37.9806, 22.8547),
                );
                path.cubic_bezier_to(
                    view.point(39.4235, 25.4325),
                    view.point(39.045, 24.321),
                    view.point(39.2816, 24.9122),
                );
                path.cubic_bezier_to(
                    view.point(39.9676, 27.0643),
                    view.point(39.5891, 25.9528),
                    view.point(39.8256, 26.6859),
                );
                path.cubic_bezier_to(
                    view.point(40.2987, 29.9967),
                    view.point(40.2041, 27.6791),
                    view.point(40.2514, 28.0812),
                );
                path.cubic_bezier_to(
                    view.point(40.0622, 32.5745),
                    view.point(40.3697, 32.1961),
                    view.point(40.3697, 32.2197),
                );
                path.cubic_bezier_to(
                    view.point(38.0516, 32.8346),
                    view.point(39.6837, 33.0238),
                    view.point(38.7375, 33.142),
                );
                path.close();
                path.move_to(view.point(44.7694, 29.642));
                path.cubic_bezier_to(
                    view.point(43.9415, 28.6014),
                    view.point(44.3909, 29.5237),
                    view.point(44.1307, 29.1927),
                );
                path.cubic_bezier_to(
                    view.point(40.0385, 23.0203),
                    view.point(43.5394, 27.3717),
                    view.point(41.4578, 24.4156),
                );
                path.cubic_bezier_to(
                    view.point(39.6837, 22.2635),
                    view.point(39.3999, 22.4054),
                    view.point(39.3289, 22.2635),
                );
                path.cubic_bezier_to(
                    view.point(41.6707, 23.7771),
                    view.point(39.8966, 22.2635),
                    view.point(40.4406, 22.6892),
                );
                path.cubic_bezier_to(
                    view.point(45.4553, 29.24),
                    view.point(43.6813, 25.598),
                    view.point(45.6919, 28.5068),
                );
                path.cubic_bezier_to(
                    view.point(44.7694, 29.642),
                    view.point(45.3607, 29.5237),
                    view.point(45.0296, 29.7129),
                );
                path.close();
                path.move_to(view.point(28.377, 20.75));
                path.cubic_bezier_to(
                    view.point(29.465, 20.0642),
                    view.point(28.235, 20.5371),
                    view.point(28.6371, 20.3007),
                );
                path.cubic_bezier_to(
                    view.point(32.682, 18.1013),
                    view.point(30.1983, 19.8513),
                    view.point(32.2089, 18.6216),
                );
                path.cubic_bezier_to(
                    view.point(33.6519, 17.7229),
                    view.point(32.9895, 17.7229),
                    view.point(33.368, 17.581),
                );
                path.cubic_bezier_to(
                    view.point(31.9251, 19.6858),
                    view.point(34.1959, 18.0067),
                    view.point(33.6755, 18.6216),
                );
                path.cubic_bezier_to(
                    view.point(30.7187, 20.4425),
                    view.point(31.5703, 19.9223),
                    view.point(31.0262, 20.2534),
                );
                path.cubic_bezier_to(
                    view.point(28.377, 20.75),
                    view.point(30.0801, 20.8446),
                    view.point(28.5425, 21.0338),
                );
                path.close();
                path.move_to(view.point(27.4781, 18.0777));
                path.cubic_bezier_to(
                    view.point(25.3019, 14.7904),
                    view.point(26.8867, 17.9121),
                    view.point(26.7921, 17.7466),
                );
                path.cubic_bezier_to(
                    view.point(23.788, 11.8107),
                    view.point(24.545, 13.3006),
                    view.point(23.859, 11.9526),
                );
                path.cubic_bezier_to(
                    view.point(21.9903, 7.814),
                    view.point(23.0074, 10.3917),
                    view.point(21.9666, 8.07414),
                );
                path.cubic_bezier_to(
                    view.point(24.6869, 6.8917),
                    view.point(22.0613, 7.43562),
                    view.point(22.8655, 7.15183),
                );
                path.cubic_bezier_to(
                    view.point(26.7448, 6.37142),
                    view.point(25.4675, 6.77345),
                    view.point(26.2954, 6.58426),
                );
                path.cubic_bezier_to(
                    view.point(28.377, 5.87479),
                    view.point(27.1469, 6.20588),
                    view.point(27.8802, 5.96939),
                );
                path.cubic_bezier_to(
                    view.point(30.0328, 5.33086),
                    view.point(28.8737, 5.78019),
                    view.point(29.6306, 5.52006),
                );
                path.cubic_bezier_to(
                    view.point(32.067, 4.83424),
                    view.point(31.2155, 4.73964),
                    view.point(31.8068, 4.5741),
                );
                path.cubic_bezier_to(
                    view.point(31.7122, 11.0303),
                    view.point(32.3272, 5.09438),
                    view.point(32.1143, 8.78361),
                );
                path.cubic_bezier_to(
                    view.point(31.4047, 14.483),
                    view.point(31.5939, 11.8107),
                    view.point(31.452, 13.3715),
                );
                path.cubic_bezier_to(
                    view.point(31.1918, 16.9189),
                    view.point(31.3574, 15.5945),
                    view.point(31.2628, 16.6824),
                );
                path.cubic_bezier_to(
                    view.point(27.4781, 18.0777),
                    view.point(30.9553, 17.7229),
                    view.point(28.6608, 18.456),
                );
                path.close();
                path.move_to(view.point(23.9773, 18.0304));
                path.cubic_bezier_to(
                    view.point(22.9128, 17.6993),
                    view.point(23.9536, 18.0067),
                    view.point(23.4805, 17.8648),
                );
                path.cubic_bezier_to(
                    view.point(21.4699, 17.1553),
                    view.point(22.3688, 17.5574),
                    view.point(21.7064, 17.3209),
                );
                path.cubic_bezier_to(
                    view.point(20.4291, 16.6114),
                    view.point(21.2334, 16.9898),
                    view.point(20.7603, 16.7533),
                );
                path.cubic_bezier_to(
                    view.point(13.0726, 9.162),
                    view.point(19.4356, 16.1621),
                    view.point(16.195, 12.8985),
                );
                path.cubic_bezier_to(
                    view.point(12.7178, 7.83765),
                    view.point(12.434, 8.40523),
                    view.point(12.292, 7.83765),
                );
                path.cubic_bezier_to(
                    view.point(14.4446, 8.09779),
                    view.point(12.8597, 7.83765),
                    view.point(13.6167, 7.9559),
                );
                path.cubic_bezier_to(
                    view.point(16.881, 8.33428),
                    view.point(15.2488, 8.21604),
                    view.point(16.3606, 8.33428),
                );
                path.cubic_bezier_to(
                    view.point(20.1453, 11.0066),
                    view.point(18.3712, 8.33428),
                    view.point(18.4895, 8.42888),
                );
                path.cubic_bezier_to(
                    view.point(20.8076, 11.9526),
                    view.point(20.3582, 11.3377),
                    view.point(20.642, 11.7634),
                );
                path.cubic_bezier_to(
                    view.point(22.1086, 13.8445),
                    view.point(21.1861, 12.4256),
                    view.point(21.6118, 13.0641),
                );
                path.cubic_bezier_to(
                    view.point(23.1967, 15.429),
                    view.point(22.3215, 14.1992),
                    view.point(22.8182, 14.9087),
                );
                path.cubic_bezier_to(
                    view.point(24.6396, 17.9594),
                    view.point(23.859, 16.3749),
                    view.point(24.6396, 17.7229),
                );
                path.cubic_bezier_to(
                    view.point(23.9773, 18.0304),
                    view.point(24.6396, 18.0777),
                    view.point(24.0955, 18.1486),
                );
                path.close();
                path.move_to(view.point(33.6755, 9.91876));
                path.cubic_bezier_to(
                    view.point(34.3378, 4.12477),
                    view.point(33.5099, 9.30389),
                    view.point(34.0067, 5.18897),
                );
                path.cubic_bezier_to(
                    view.point(35.0475, 3.03692),
                    view.point(34.4561, 3.74638),
                    view.point(34.7163, 3.3207),
                );
                path.cubic_bezier_to(
                    view.point(35.6152, 2.04366),
                    view.point(35.4023, 2.68218),
                    view.point(35.5679, 2.42204),
                );
                path.line_to(view.point(35.6861, 1.52338));
                path.line_to(view.point(35.1894, 1.45244));
                path.cubic_bezier_to(
                    view.point(33.368, 1.0504),
                    view.point(34.9292, 1.40514),
                    view.point(34.1013, 1.2396),
                );
                path.cubic_bezier_to(
                    view.point(31.1445, 0.624723),
                    view.point(32.6347, 0.884861),
                    view.point(31.6412, 0.69567),
                );
                path.cubic_bezier_to(
                    view.point(30.151, 0.364584),
                    view.point(30.6714, 0.530127),
                    view.point(30.222, 0.435531),
                );
                path.cubic_bezier_to(
                    view.point(32.682, 0.00984984),
                    view.point(29.9145, 0.128095),
                    view.point(30.5058, 0.0571477),
                );
                path.cubic_bezier_to(
                    view.point(35.7098, 0.932159),
                    view.point(35.2604, -0.0374481),
                    view.point(35.5442, 0.0571477),
                );
                path.cubic_bezier_to(
                    view.point(36.1119, 1.61798),
                    view.point(35.7571, 1.16865),
                    view.point(35.9463, 1.47609),
                );
                path.cubic_bezier_to(
                    view.point(35.7098, 4.66869),
                    view.point(36.5613, 2.02001),
                    view.point(36.3958, 3.34435),
                );
                path.cubic_bezier_to(
                    view.point(35.4732, 5.42546),
                    view.point(35.6152, 4.83424),
                    view.point(35.5205, 5.18897),
                );
                path.cubic_bezier_to(
                    view.point(34.9528, 7.31738),
                    view.point(35.4259, 5.66195),
                    view.point(35.1894, 6.53696),
                );
                path.cubic_bezier_to(
                    view.point(34.3142, 9.51673),
                    view.point(34.7163, 8.12144),
                    view.point(34.4324, 9.1147),
                );
                path.cubic_bezier_to(
                    view.point(33.6755, 9.91876),
                    view.point(34.1013, 10.2971),
                    view.point(33.8174, 10.4863),
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
