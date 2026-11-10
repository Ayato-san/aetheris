use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct DungeonsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl DungeonsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for DungeonsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 53.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(14.7157, 52.0346));
                path.cubic_bezier_to(
                    view.point(13.1563, 51.6331),
                    view.point(14.5976, 51.9874),
                    view.point(13.8888, 51.822),
                );
                path.cubic_bezier_to(
                    view.point(6.49344, 48.6808),
                    view.point(10.77, 51.0662),
                    view.point(7.39128, 49.5547),
                );
                path.cubic_bezier_to(
                    view.point(6.65883, 44.0752),
                    view.point(5.31208, 47.5471),
                    view.point(5.38296, 45.634),
                );
                path.cubic_bezier_to(
                    view.point(9.65949, 41.8314),
                    view.point(6.8951, 43.7918),
                    view.point(8.05284, 42.9179),
                );
                path.cubic_bezier_to(
                    view.point(13.4162, 37.6037),
                    view.point(13.3453, 39.3279),
                    view.point(13.5107, 39.1389),
                );
                path.cubic_bezier_to(
                    view.point(12.1876, 35.6906),
                    view.point(13.369, 36.7298),
                    view.point(13.2036, 36.47),
                );
                path.cubic_bezier_to(
                    view.point(10.1557, 30.5418),
                    view.point(9.82488, 33.8484),
                    view.point(9.2342, 32.3604),
                );
                path.cubic_bezier_to(
                    view.point(10.6755, 29.1483),
                    view.point(10.3447, 30.1639),
                    view.point(10.581, 29.5498),
                );
                path.cubic_bezier_to(
                    view.point(13.8179, 25.204),
                    view.point(11.1716, 27.2116),
                    view.point(12.0931, 26.0543),
                );
                path.cubic_bezier_to(
                    view.point(19.2285, 25.1568),
                    view.point(15.8026, 24.2357),
                    view.point(17.2911, 24.212),
                );
                path.cubic_bezier_to(
                    view.point(22.2292, 28.7468),
                    view.point(20.9061, 25.9834),
                    view.point(21.8748, 27.1171),
                );
                path.cubic_bezier_to(
                    view.point(22.8199, 30.4237),
                    view.point(22.3473, 29.1719),
                    view.point(22.6072, 29.9277),
                );
                path.cubic_bezier_to(
                    view.point(21.3786, 35.3127),
                    view.point(23.6704, 32.2896),
                    view.point(23.127, 34.1082),
                );
                path.cubic_bezier_to(
                    view.point(19.6066, 38.3123),
                    view.point(19.7247, 36.4228),
                    view.point(19.3467, 37.0841),
                );
                path.cubic_bezier_to(
                    view.point(23.2924, 41.7369),
                    view.point(19.8192, 39.3042),
                    view.point(20.1736, 39.6113),
                );
                path.cubic_bezier_to(
                    view.point(27.1909, 46.7204),
                    view.point(26.4821, 43.9098),
                    view.point(27.0728, 44.6656),
                );
                path.cubic_bezier_to(
                    view.point(24.3556, 49.7908),
                    view.point(27.2618, 48.1848),
                    view.point(27.0255, 48.421),
                );
                path.cubic_bezier_to(
                    view.point(16.4641, 52.0346),
                    view.point(21.4022, 51.2788),
                    view.point(18.9214, 51.9874),
                );
                path.cubic_bezier_to(
                    view.point(14.7157, 52.0346),
                    view.point(15.6372, 52.0582),
                    view.point(14.8575, 52.0582),
                );
                path.close();
                path.move_to(view.point(2.3823, 48.7989));
                path.cubic_bezier_to(
                    view.point(0.255849, 45.0908),
                    view.point(0.255849, 47.7597),
                    view.point(-0.429341, 46.5551),
                );
                path.cubic_bezier_to(
                    view.point(3.58729, 41.9731),
                    view.point(0.799276, 43.9807),
                    view.point(1.31908, 43.5083),
                );
                path.cubic_bezier_to(
                    view.point(6.42256, 38.8791),
                    view.point(5.99727, 40.3671),
                    view.point(6.42256, 39.8947),
                );
                path.cubic_bezier_to(
                    view.point(5.28845, 36.9896),
                    view.point(6.42256, 38.0997),
                    view.point(6.1154, 37.6037),
                );
                path.cubic_bezier_to(
                    view.point(3.75268, 32.9509),
                    view.point(3.61092, 35.7379),
                    view.point(3.06749, 34.2971),
                );
                path.cubic_bezier_to(
                    view.point(4.24885, 31.5338),
                    view.point(3.89444, 32.6438),
                    view.point(4.13072, 32.0061),
                );
                path.cubic_bezier_to(
                    view.point(6.8951, 28.3925),
                    view.point(4.65052, 30.0458),
                    view.point(5.64286, 28.8885),
                );
                path.cubic_bezier_to(
                    view.point(7.76931, 29.7388),
                    view.point(7.86382, 28.0146),
                    view.point(8.17097, 28.4634),
                );
                path.cubic_bezier_to(
                    view.point(7.69843, 33.1398),
                    view.point(7.48578, 30.7071),
                    view.point(7.46216, 31.7463),
                );
                path.cubic_bezier_to(
                    view.point(8.95067, 35.9977),
                    view.point(7.95833, 34.6042),
                    view.point(8.1946, 35.1474),
                );
                path.cubic_bezier_to(
                    view.point(9.61224, 38.4067),
                    view.point(9.58861, 36.7298),
                    view.point(9.65949, 37.0133),
                );
                path.cubic_bezier_to(
                    view.point(7.72206, 41.0993),
                    view.point(9.56498, 39.6349),
                    view.point(9.54135, 39.6821),
                );
                path.cubic_bezier_to(
                    view.point(4.24885, 46.7441),
                    view.point(4.43787, 43.65),
                    view.point(4.34336, 43.8154),
                );
                path.cubic_bezier_to(
                    view.point(3.9417, 49.0823),
                    view.point(4.2016, 48.7989),
                    view.point(4.17797, 48.8697),
                );
                path.cubic_bezier_to(
                    view.point(2.3823, 48.7989),
                    view.point(3.56366, 49.3657),
                    view.point(3.54004, 49.3657),
                );
                path.close();
                path.move_to(view.point(32.9796, 44.2169));
                path.cubic_bezier_to(
                    view.point(30.2152, 44.1933),
                    view.point(32.4361, 44.1697),
                    view.point(31.1839, 44.1697),
                );
                path.cubic_bezier_to(
                    view.point(27.9942, 43.8154),
                    view.point(28.3723, 44.2641),
                    view.point(27.9942, 44.2169),
                );
                path.cubic_bezier_to(
                    view.point(30.2152, 43.4139),
                    view.point(27.9942, 43.3666),
                    view.point(28.2305, 43.343),
                );
                path.cubic_bezier_to(
                    view.point(46.1399, 35.9268),
                    view.point(36.689, 43.65),
                    view.point(42.2651, 41.0284),
                );
                path.cubic_bezier_to(
                    view.point(48.6681, 29.5262),
                    view.point(47.9829, 33.4941),
                    view.point(49.5423, 29.5262),
                );
                path.cubic_bezier_to(
                    view.point(47.9829, 30.3765),
                    view.point(48.5263, 29.5262),
                    view.point(48.3136, 29.7624),
                );
                path.cubic_bezier_to(
                    view.point(44.5097, 35.0765),
                    view.point(46.8251, 32.4785),
                    view.point(45.0531, 34.864),
                );
                path.cubic_bezier_to(
                    view.point(43.2574, 34.2499),
                    view.point(44.1316, 35.2182),
                    view.point(43.8245, 35.0293),
                );
                path.line_to(view.point(42.8558, 33.6594));
                path.line_to(view.point(42.8558, 32.3368));
                path.cubic_bezier_to(
                    view.point(43.0211, 30.6363),
                    view.point(42.8558, 31.3212),
                    view.point(42.8794, 30.8961),
                );
                path.cubic_bezier_to(
                    view.point(43.7772, 29.2192),
                    view.point(43.1157, 30.4709),
                    view.point(43.4701, 29.8096),
                );
                path.cubic_bezier_to(
                    view.point(45.2421, 25.0387),
                    view.point(44.3679, 28.1563),
                    view.point(44.9822, 26.4086),
                );
                path.cubic_bezier_to(
                    view.point(47.4158, 23.4326),
                    view.point(45.5493, 23.598),
                    view.point(45.7855, 23.4326),
                );
                path.cubic_bezier_to(
                    view.point(49.495, 23.9759),
                    view.point(48.7153, 23.4326),
                    view.point(49.2824, 23.5743),
                );
                path.cubic_bezier_to(
                    view.point(49.6131, 25.9598),
                    view.point(49.5659, 24.0939),
                    view.point(49.6131, 24.897),
                );
                path.cubic_bezier_to(
                    view.point(49.9439, 28.298),
                    view.point(49.6131, 27.684),
                    view.point(49.6368, 27.7312),
                );
                path.cubic_bezier_to(
                    view.point(50.8418, 33.3996),
                    view.point(50.4874, 29.2428),
                    view.point(50.6055, 29.8805),
                );
                path.cubic_bezier_to(
                    view.point(51.1962, 36.8243),
                    view.point(50.9126, 34.2971),
                    view.point(51.0544, 35.8323),
                );
                path.cubic_bezier_to(
                    view.point(51.5742, 40.4379),
                    view.point(51.3143, 37.7927),
                    view.point(51.4797, 39.4223),
                );
                path.cubic_bezier_to(
                    view.point(51.1962, 44.1224),
                    view.point(51.8577, 43.3902),
                    view.point(51.8105, 43.8862),
                );
                path.cubic_bezier_to(
                    view.point(32.9796, 44.2169),
                    view.point(50.8418, 44.2641),
                    view.point(34.4681, 44.3586),
                );
                path.close();
                path.move_to(view.point(27.7107, 40.556));
                path.cubic_bezier_to(
                    view.point(23.6468, 39.1389),
                    view.point(26.6711, 40.3435),
                    view.point(24.3084, 39.5168),
                );
                path.cubic_bezier_to(
                    view.point(23.316, 36.7771),
                    view.point(22.5363, 38.5012),
                    view.point(22.4891, 38.1469),
                );
                path.cubic_bezier_to(
                    view.point(25.7496, 35.7142),
                    view.point(24.1902, 35.3363),
                    view.point(24.4974, 35.1946),
                );
                path.cubic_bezier_to(
                    view.point(30.6877, 36.8479),
                    view.point(26.8129, 36.1866),
                    view.point(29.1992, 36.7298),
                );
                path.cubic_bezier_to(
                    view.point(31.2548, 39.7058),
                    view.point(32.8378, 37.0369),
                    view.point(32.9323, 37.462),
                );
                path.cubic_bezier_to(
                    view.point(29.1756, 40.6505),
                    view.point(30.6405, 40.5324),
                    view.point(30.3333, 40.6741),
                );
                path.cubic_bezier_to(
                    view.point(27.7107, 40.556),
                    view.point(28.6085, 40.6505),
                    view.point(27.9706, 40.6033),
                );
                path.close();
                path.move_to(view.point(32.507, 40.4616));
                path.cubic_bezier_to(
                    view.point(32.7906, 38.8319),
                    view.point(32.1526, 40.1073),
                    view.point(32.1999, 39.8947),
                );
                path.cubic_bezier_to(
                    view.point(35.5077, 36.1866),
                    view.point(33.5939, 37.3439),
                    view.point(34.4681, 36.4936),
                );
                path.cubic_bezier_to(
                    view.point(40.4458, 33.4469),
                    view.point(37.0671, 35.7142),
                    view.point(39.1227, 34.5806),
                );
                path.cubic_bezier_to(
                    view.point(42.4541, 33.4232),
                    view.point(41.3436, 32.6911),
                    view.point(41.7216, 32.6911),
                );
                path.cubic_bezier_to(
                    view.point(42.7612, 35.1002),
                    view.point(42.7376, 33.7067),
                    view.point(42.7612, 33.7775),
                );
                path.cubic_bezier_to(
                    view.point(41.4854, 37.769),
                    view.point(42.7849, 36.7771),
                    view.point(42.714, 36.966),
                );
                path.cubic_bezier_to(
                    view.point(37.5869, 39.7766),
                    view.point(40.5403, 38.3831),
                    view.point(37.8232, 39.7766),
                );
                path.cubic_bezier_to(
                    view.point(36.1693, 40.1781),
                    view.point(37.4924, 39.7766),
                    view.point(36.8544, 39.9419),
                );
                path.cubic_bezier_to(
                    view.point(32.507, 40.4616),
                    view.point(34.4681, 40.6977),
                    view.point(32.8614, 40.8394),
                );
                path.close();
                path.move_to(view.point(29.4591, 34.6514));
                path.cubic_bezier_to(
                    view.point(25.2298, 33.1398),
                    view.point(27.5217, 34.368),
                    view.point(25.7969, 33.7539),
                );
                path.cubic_bezier_to(
                    view.point(25.0881, 31.4629),
                    view.point(24.8518, 32.7147),
                    view.point(24.7573, 31.8644),
                );
                path.cubic_bezier_to(
                    view.point(26.9074, 31.77),
                    view.point(25.3952, 31.0614),
                    view.point(25.3716, 31.0614),
                );
                path.cubic_bezier_to(
                    view.point(38.3666, 29.8569),
                    view.point(30.8059, 33.5886),
                    view.point(35.3895, 32.8328),
                );
                path.cubic_bezier_to(
                    view.point(41.0365, 25.0859),
                    view.point(39.6425, 28.5815),
                    view.point(40.6821, 26.7156),
                );
                path.cubic_bezier_to(
                    view.point(42.7376, 23.4799),
                    view.point(41.32, 23.8814),
                    view.point(41.887, 23.3145),
                );
                path.cubic_bezier_to(
                    view.point(43.1865, 26.007),
                    view.point(43.6118, 23.6452),
                    view.point(43.73, 24.2829),
                );
                path.cubic_bezier_to(
                    view.point(29.4591, 34.6514),
                    view.point(41.2727, 32.0061),
                    view.point(35.5786, 35.5961),
                );
                path.close();
                path.move_to(view.point(20.3863, 22.3462));
                path.cubic_bezier_to(
                    view.point(15.6608, 22.2753),
                    view.point(20.2917, 22.3226),
                    view.point(18.1653, 22.2989),
                );
                path.cubic_bezier_to(
                    view.point(10.8645, 22.0391),
                    view.point(11.3607, 22.2517),
                    view.point(11.1008, 22.2281),
                );
                path.cubic_bezier_to(
                    view.point(11.7387, 20.811),
                    view.point(10.2974, 21.5195),
                    view.point(10.6991, 20.9527),
                );
                path.cubic_bezier_to(
                    view.point(13.1563, 18.8506),
                    view.point(12.731, 20.6457),
                    view.point(12.92, 20.4095),
                );
                path.cubic_bezier_to(
                    view.point(17.362, 10.2771),
                    view.point(13.6052, 16.1109),
                    view.point(15.4718, 12.3083),
                );
                path.cubic_bezier_to(
                    view.point(18.5197, 9.59221),
                    view.point(17.9763, 9.61583),
                    view.point(18.0235, 9.59221),
                );
                path.cubic_bezier_to(
                    view.point(19.4884, 10.7259),
                    view.point(19.2994, 9.61583),
                    view.point(19.4884, 9.85201),
                );
                path.cubic_bezier_to(
                    view.point(19.7011, 12.1194),
                    view.point(19.4884, 11.1038),
                    view.point(19.5829, 11.7415),
                );
                path.cubic_bezier_to(
                    view.point(19.1813, 15.3787),
                    view.point(20.1027, 13.3948),
                    view.point(19.9846, 14.2214),
                );
                path.cubic_bezier_to(
                    view.point(17.4801, 19.9135),
                    view.point(18.3543, 16.5833),
                    view.point(17.4801, 18.9215),
                );
                path.cubic_bezier_to(
                    view.point(18.5906, 20.5039),
                    view.point(17.4801, 20.5039),
                    view.point(18.0472, 20.811),
                );
                path.cubic_bezier_to(
                    view.point(19.323, 18.9215),
                    view.point(18.8977, 20.3386),
                    view.point(19.0159, 20.0788),
                );
                path.cubic_bezier_to(
                    view.point(28.7976, 10.2063),
                    view.point(20.4571, 14.7647),
                    view.point(24.521, 11.0093),
                );
                path.cubic_bezier_to(
                    view.point(33.712, 10.2063),
                    view.point(29.7426, 10.0173),
                    view.point(32.7906, 10.0173),
                );
                path.cubic_bezier_to(
                    view.point(43.1865, 18.827),
                    view.point(38.0831, 11.0329),
                    view.point(41.7453, 14.3631),
                );
                path.cubic_bezier_to(
                    view.point(44.7223, 20.622),
                    view.point(43.73, 20.4803),
                    view.point(44.108, 20.9291),
                );
                path.cubic_bezier_to(
                    view.point(44.6987, 18.2838),
                    view.point(45.2657, 20.315),
                    view.point(45.2657, 20.0552),
                );
                path.cubic_bezier_to(
                    view.point(42.7849, 14.4576),
                    view.point(44.2025, 16.7722),
                    view.point(43.6118, 15.6149),
                );
                path.cubic_bezier_to(
                    view.point(43.2574, 12.0721),
                    view.point(41.6508, 12.9224),
                    view.point(41.7925, 12.2139),
                );
                path.cubic_bezier_to(
                    view.point(44.5333, 11.836),
                    view.point(43.6591, 12.0249),
                    view.point(44.2498, 11.9304),
                );
                path.cubic_bezier_to(
                    view.point(47.4867, 13.5601),
                    view.point(45.9509, 11.4108),
                    view.point(46.4471, 11.6943),
                );
                path.cubic_bezier_to(
                    view.point(49.2824, 18.3783),
                    view.point(48.2664, 14.9536),
                    view.point(48.8335, 16.4652),
                );
                path.cubic_bezier_to(
                    view.point(50.9126, 20.8582),
                    view.point(49.7785, 20.5039),
                    view.point(49.873, 20.622),
                );
                path.cubic_bezier_to(
                    view.point(50.6291, 22.2045),
                    view.point(52.4957, 21.1653),
                    view.point(52.3067, 21.9919),
                );
                path.cubic_bezier_to(
                    view.point(41.1073, 22.11),
                    view.point(49.0933, 22.3698),
                    view.point(41.5799, 22.2989),
                );
                path.cubic_bezier_to(
                    view.point(40.7766, 20.6693),
                    view.point(40.4694, 21.8502),
                    view.point(40.3985, 21.5432),
                );
                path.cubic_bezier_to(
                    view.point(40.4458, 18.4964),
                    view.point(41.0601, 20.0316),
                    view.point(41.0365, 19.819),
                );
                path.cubic_bezier_to(
                    view.point(33.9719, 12.9224),
                    view.point(39.2644, 15.8039),
                    view.point(36.689, 13.6073),
                );
                path.cubic_bezier_to(
                    view.point(28.6085, 12.9224),
                    view.point(32.8851, 12.6626),
                    view.point(29.719, 12.6626),
                );
                path.cubic_bezier_to(
                    view.point(21.8748, 19.016),
                    view.point(25.7024, 13.6546),
                    view.point(23.0561, 16.0164),
                );
                path.line_to(view.point(21.4731, 20.0788));
                path.line_to(view.point(21.7094, 20.6457));
                path.cubic_bezier_to(
                    view.point(21.6621, 22.0391),
                    view.point(22.0402, 21.4487),
                    view.point(22.0165, 21.803),
                );
                path.cubic_bezier_to(
                    view.point(20.3863, 22.3462),
                    view.point(21.3786, 22.2281),
                    view.point(20.6934, 22.3934),
                );
                path.close();
                path.move_to(view.point(20.8115, 12.2139));
                path.cubic_bezier_to(
                    view.point(20.3863, 11.34),
                    view.point(20.6934, 12.1666),
                    view.point(20.5044, 11.7651),
                );
                path.cubic_bezier_to(
                    view.point(19.8665, 10.2063),
                    view.point(20.2681, 10.9385),
                    view.point(20.0318, 10.4189),
                );
                path.cubic_bezier_to(
                    view.point(19.8901, 8.19872),
                    view.point(19.5357, 9.75754),
                    view.point(19.5357, 8.83642),
                );
                path.cubic_bezier_to(
                    view.point(29.0338, 4.04187),
                    view.point(20.6462, 6.71075),
                    view.point(25.4897, 4.51424),
                );
                path.cubic_bezier_to(
                    view.point(32.3653, 4.2072),
                    view.point(30.4751, 3.85292),
                    view.point(32.0581, 3.92377),
                );
                path.cubic_bezier_to(
                    view.point(32.3653, 5.45898),
                    view.point(32.6724, 4.49062),
                    view.point(32.6724, 5.01022),
                );
                path.cubic_bezier_to(
                    view.point(31.7982, 6.45095),
                    view.point(32.2471, 5.62431),
                    view.point(31.9872, 6.09667),
                );
                path.cubic_bezier_to(
                    view.point(29.1283, 8.64747),
                    view.point(31.0658, 7.86806),
                    view.point(30.3806, 8.41128),
                );
                path.cubic_bezier_to(
                    view.point(22.6781, 11.4581),
                    view.point(26.931, 9.00175),
                    view.point(24.6392, 10.0173),
                );
                path.cubic_bezier_to(
                    view.point(20.8115, 12.2139),
                    view.point(21.6149, 12.2375),
                    view.point(21.1423, 12.4264),
                );
                path.close();
                path.move_to(view.point(40.0441, 11.4581));
                path.cubic_bezier_to(
                    view.point(38.9809, 10.8204),
                    view.point(39.737, 11.2927),
                    view.point(39.2644, 11.0093),
                );
                path.cubic_bezier_to(
                    view.point(33.6648, 8.69471),
                    view.point(37.705, 9.94649),
                    view.point(35.4368, 9.02536),
                );
                path.cubic_bezier_to(
                    view.point(32.1999, 7.25398),
                    view.point(32.2471, 8.4349),
                    view.point(31.7746, 7.96253),
                );
                path.cubic_bezier_to(
                    view.point(32.7669, 6.28562),
                    view.point(32.318, 7.04141),
                    view.point(32.5779, 6.61628),
                );
                path.cubic_bezier_to(
                    view.point(34.7752, 4.39614),
                    view.point(33.334, 5.27003),
                    view.point(34.2554, 4.39614),
                );
                path.cubic_bezier_to(
                    view.point(44.7696, 10.4897),
                    view.point(37.8704, 4.39614),
                    view.point(45.1712, 8.83642),
                );
                path.cubic_bezier_to(
                    view.point(43.6355, 11.1038),
                    view.point(44.6987, 10.7495),
                    view.point(44.4388, 10.8912),
                );
                path.cubic_bezier_to(
                    view.point(42.6904, 11.5053),
                    view.point(43.3756, 11.1746),
                    view.point(42.9739, 11.34),
                );
                path.cubic_bezier_to(
                    view.point(40.0441, 11.4581),
                    view.point(41.9816, 11.9068),
                    view.point(40.8238, 11.8832),
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
