use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct RecipesIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl RecipesIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for RecipesIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 54.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(15.0141, 53.5994));
                path.cubic_bezier_to(
                    view.point(14.8486, 52.3703),
                    view.point(14.8486, 53.363),
                    view.point(14.8013, 53.0794),
                );
                path.cubic_bezier_to(
                    view.point(17.2367, 48.0685),
                    view.point(14.8959, 51.3303),
                    view.point(15.0614, 51.023),
                );
                path.cubic_bezier_to(
                    view.point(20.0504, 44.1921),
                    view.point(17.6859, 47.4539),
                    view.point(18.9391, 45.7048),
                );
                path.cubic_bezier_to(
                    view.point(22.8641, 40.2685),
                    view.point(21.138, 42.6558),
                    view.point(22.3912, 40.9067),
                );
                path.cubic_bezier_to(
                    view.point(24.2827, 38.283),
                    view.point(23.3133, 39.6539),
                    view.point(23.9517, 38.7558),
                );
                path.cubic_bezier_to(
                    view.point(26.6944, 37.8339),
                    view.point(25.1812, 37.0067),
                    view.point(25.7014, 36.9121),
                );
                path.cubic_bezier_to(
                    view.point(27.5929, 39.5121),
                    view.point(27.3801, 38.4721),
                    view.point(27.7111, 39.0867),
                );
                path.cubic_bezier_to(
                    view.point(22.9823, 45.303),
                    view.point(27.522, 39.8194),
                    view.point(25.2285, 42.703),
                );
                path.cubic_bezier_to(
                    view.point(20.8543, 47.8794),
                    view.point(22.3912, 46.0121),
                    view.point(21.4454, 47.1703),
                );
                path.cubic_bezier_to(
                    view.point(15.8417, 53.7649),
                    view.point(19.5066, 49.5576),
                    view.point(16.1018, 53.5521),
                );
                path.cubic_bezier_to(
                    view.point(15.0141, 53.5994),
                    view.point(15.5343, 54.0012),
                    view.point(15.2269, 53.9539),
                );
                path.close();
                path.move_to(view.point(41.5431, 50.3612));
                path.cubic_bezier_to(
                    view.point(32.3691, 40.7412),
                    view.point(38.2092, 47.0048),
                    view.point(32.9838, 41.5212),
                );
                path.cubic_bezier_to(
                    view.point(27.5929, 35.7776),
                    view.point(31.9671, 40.2212),
                    view.point(29.8155, 37.9758),
                );
                path.cubic_bezier_to(
                    view.point(19.4356, 27.5758),
                    view.point(25.3704, 33.5558),
                    view.point(21.7055, 29.8685),
                );
                path.cubic_bezier_to(
                    view.point(15.1087, 21.7376),
                    view.point(14.8723, 22.8958),
                    view.point(14.7541, 22.7303),
                );
                path.cubic_bezier_to(
                    view.point(17.0239, 18.4994),
                    view.point(16.2436, 18.6176),
                    view.point(16.3146, 18.4994),
                );
                path.cubic_bezier_to(
                    view.point(19.9558, 21.0285),
                    view.point(17.4495, 18.4994),
                    view.point(17.6623, 18.6885),
                );
                path.cubic_bezier_to(
                    view.point(23.5024, 23.8885),
                    view.point(22.5094, 23.6285),
                    view.point(22.8404, 23.8885),
                );
                path.cubic_bezier_to(
                    view.point(24.1408, 23.4394),
                    view.point(23.8098, 23.8885),
                    view.point(23.928, 23.8176),
                );
                path.cubic_bezier_to(
                    view.point(30.9031, 18.7358),
                    view.point(24.4246, 22.9903),
                    view.point(24.8029, 22.7303),
                );
                path.cubic_bezier_to(
                    view.point(37.6418, 15.3085),
                    view.point(33.1257, 17.2939),
                    view.point(36.4359, 15.6158),
                );
                path.cubic_bezier_to(
                    view.point(39.3678, 14.8358),
                    view.point(38.1146, 15.1903),
                    view.point(38.8949, 14.9776),
                );
                path.line_to(view.point(40.219, 14.5758));
                path.line_to(view.point(40.6682, 14.883));
                path.cubic_bezier_to(
                    view.point(42.1578, 15.6158),
                    view.point(40.9283, 15.0485),
                    view.point(41.5904, 15.3794),
                );
                path.cubic_bezier_to(
                    view.point(45.7045, 17.8848),
                    view.point(43.1982, 16.0885),
                    view.point(44.3567, 16.8212),
                );
                path.cubic_bezier_to(
                    view.point(48.4472, 17.8612),
                    view.point(46.7921, 18.7358),
                    view.point(47.6669, 18.7121),
                );
                path.cubic_bezier_to(
                    view.point(48.5891, 14.2685),
                    view.point(49.2984, 16.9158),
                    view.point(49.3457, 15.7812),
                );
                path.cubic_bezier_to(
                    view.point(48.9437, 11.7158),
                    view.point(47.8561, 12.803),
                    view.point(47.998, 11.7158),
                );
                path.cubic_bezier_to(
                    view.point(50.9062, 13.5594),
                    view.point(49.5348, 11.7158),
                    view.point(50.1496, 12.3067),
                );
                path.cubic_bezier_to(
                    view.point(51.9939, 17.1758),
                    view.point(51.8283, 15.0485),
                    view.point(52.0411, 15.8048),
                );
                path.cubic_bezier_to(
                    view.point(50.5043, 20.4139),
                    view.point(51.9466, 18.6176),
                    view.point(51.6392, 19.2558),
                );
                path.cubic_bezier_to(
                    view.point(42.8671, 21.2885),
                    view.point(48.3999, 22.5176),
                    view.point(45.468, 22.8485),
                );
                path.cubic_bezier_to(
                    view.point(38.753, 20.4848),
                    view.point(41.1648, 20.2485),
                    view.point(40.4318, 20.1067),
                );
                path.cubic_bezier_to(
                    view.point(36.8142, 20.8867),
                    view.point(38.1383, 20.6267),
                    view.point(37.2634, 20.8158),
                );
                path.cubic_bezier_to(
                    view.point(31.778, 23.2503),
                    view.point(35.9394, 21.0521),
                    view.point(35.6556, 21.1703),
                );
                path.cubic_bezier_to(
                    view.point(26.5053, 25.6139),
                    view.point(26.8127, 25.8976),
                    view.point(27.2855, 25.6848),
                );
                path.cubic_bezier_to(
                    view.point(25.5595, 25.5194),
                    view.point(26.127, 25.5903),
                    view.point(25.7014, 25.5194),
                );
                path.cubic_bezier_to(
                    view.point(29.579, 30.7194),
                    view.point(24.4482, 25.3303),
                    view.point(27.5456, 29.3485),
                );
                path.cubic_bezier_to(
                    view.point(32.6764, 33.8158),
                    view.point(30.6903, 31.4521),
                    view.point(32.109, 32.8703),
                );
                path.cubic_bezier_to(
                    view.point(36.1285, 37.7158),
                    view.point(33.0075, 34.383),
                    view.point(34.1188, 35.6358),
                );
                path.cubic_bezier_to(
                    view.point(45.8227, 48.0212),
                    view.point(43.0327, 44.8539),
                    view.point(45.6808, 47.6667),
                );
                path.cubic_bezier_to(
                    view.point(44.5695, 50.243),
                    view.point(46.0591, 48.5885),
                    view.point(45.799, 49.0376),
                );
                path.cubic_bezier_to(
                    view.point(41.5431, 50.3612),
                    view.point(43.1036, 51.6612),
                    view.point(42.8435, 51.6848),
                );
                path.close();
                path.move_to(view.point(3.16834, 38.0703));
                path.cubic_bezier_to(
                    view.point(3.71216, 36.4394),
                    view.point(2.31714, 37.8339),
                    view.point(2.5063, 37.2194),
                );
                path.cubic_bezier_to(
                    view.point(10.3799, 32.1139),
                    view.point(4.18504, 36.1321),
                    view.point(7.18787, 34.1939),
                );
                path.cubic_bezier_to(
                    view.point(16.5983, 28.2376),
                    view.point(13.5955, 30.0103),
                    view.point(16.3855, 28.2848),
                );
                path.cubic_bezier_to(
                    view.point(18.0879, 30.0103),
                    view.point(17.3076, 28.0721),
                    view.point(18.0879, 28.9939),
                );
                path.cubic_bezier_to(
                    view.point(12.366, 33.9576),
                    view.point(18.0879, 30.6721),
                    view.point(17.2131, 31.2867),
                );
                path.cubic_bezier_to(
                    view.point(7.96813, 36.3921),
                    view.point(11.2074, 34.5958),
                    view.point(9.22128, 35.7067),
                );
                path.cubic_bezier_to(
                    view.point(3.16834, 38.0703),
                    view.point(4.91802, 38.1176),
                    view.point(4.1614, 38.3776),
                );
                path.close();
                path.move_to(view.point(33.4804, 31.0503));
                path.cubic_bezier_to(
                    view.point(31.3997, 29.3721),
                    view.point(32.771, 30.6958),
                    view.point(31.5415, 29.7267),
                );
                path.cubic_bezier_to(
                    view.point(33.0784, 25.8503),
                    view.point(31.1396, 28.8048),
                    view.point(31.3997, 28.2612),
                );
                path.cubic_bezier_to(
                    view.point(37.6654, 22.6358),
                    view.point(35.23, 22.8012),
                    view.point(35.4428, 22.6358),
                );
                path.cubic_bezier_to(
                    view.point(39.1077, 25.9448),
                    view.point(40.3372, 22.6358),
                    view.point(40.7864, 23.6048),
                );
                path.cubic_bezier_to(
                    view.point(34.6862, 31.1921),
                    view.point(38.3747, 26.9848),
                    view.point(34.9463, 31.0503),
                );
                path.cubic_bezier_to(
                    view.point(33.4804, 31.0503),
                    view.point(34.3315, 31.3576),
                    view.point(34.0478, 31.3339),
                );
                path.close();
                path.move_to(view.point(5.58006, 25.2594));
                path.cubic_bezier_to(
                    view.point(2.05706, 23.1085),
                    view.point(3.59394, 24.2194),
                    view.point(2.36443, 23.463),
                );
                path.cubic_bezier_to(
                    view.point(0.0, 18.6885),
                    view.point(1.41866, 22.3521),
                    view.point(0.0, 19.303),
                );
                path.cubic_bezier_to(
                    view.point(3.7358, 14.0794),
                    view.point(0.0, 17.9558),
                    view.point(0.354665, 17.5303),
                );
                path.cubic_bezier_to(
                    view.point(6.38397, 11.1248),
                    view.point(5.58006, 12.2121),
                    view.point(6.40761, 11.2667),
                );
                path.cubic_bezier_to(
                    view.point(4.587, 10.983),
                    view.point(6.33668, 10.8885),
                    view.point(6.21846, 10.8885),
                );
                path.cubic_bezier_to(
                    view.point(3.83038, 10.6758),
                    view.point(3.99589, 11.0303),
                    view.point(3.9486, 11.0067),
                );
                path.cubic_bezier_to(
                    view.point(4.94166, 8.66667),
                    view.point(3.66487, 10.1794),
                    view.point(4.11411, 9.37576),
                );
                path.cubic_bezier_to(
                    view.point(7.6844, 8.33576),
                    view.point(6.17117, 7.62667),
                    view.point(6.6677, 7.55576),
                );
                path.cubic_bezier_to(
                    view.point(11.4911, 6.46848),
                    view.point(8.77204, 9.13939),
                    view.point(8.67746, 9.16303),
                );
                path.cubic_bezier_to(
                    view.point(14.8013, 3.93939),
                    view.point(13.8556, 4.17576),
                    view.point(14.1866, 3.93939),
                );
                path.cubic_bezier_to(
                    view.point(19.1992, 6.25576),
                    view.point(15.4634, 3.93939),
                    view.point(18.6317, 5.61757),
                );
                path.cubic_bezier_to(
                    view.point(21.1144, 10.3685),
                    view.point(19.6011, 6.72848),
                    view.point(20.4996, 8.66667),
                );
                path.cubic_bezier_to(
                    view.point(18.7263, 14.363),
                    view.point(21.54, 11.5267),
                    view.point(21.469, 11.6212),
                );
                path.cubic_bezier_to(
                    view.point(15.6998, 17.4358),
                    view.point(17.4495, 15.6394),
                    view.point(16.0781, 17.0103),
                );
                path.cubic_bezier_to(
                    view.point(7.35338, 25.4958),
                    view.point(14.7068, 18.523),
                    view.point(7.6844, 25.3067),
                );
                path.cubic_bezier_to(
                    view.point(5.58006, 25.2594),
                    view.point(6.92778, 25.7558),
                    view.point(6.36032, 25.6848),
                );
                path.close();
                path.move_to(view.point(7.75534, 23.4394));
                path.cubic_bezier_to(
                    view.point(8.79568, 22.2812),
                    view.point(7.99178, 23.2739),
                    view.point(8.46466, 22.7303),
                );
                path.cubic_bezier_to(
                    view.point(7.87356, 21.7612),
                    view.point(9.81239, 20.9339),
                    view.point(9.62324, 20.8158),
                );
                path.cubic_bezier_to(
                    view.point(5.93472, 22.5176),
                    view.point(7.18787, 22.1158),
                    view.point(6.40761, 22.423),
                );
                path.cubic_bezier_to(
                    view.point(5.29633, 23.1558),
                    view.point(5.13082, 22.683),
                    view.point(5.05988, 22.7303),
                );
                path.cubic_bezier_to(
                    view.point(7.75534, 23.4394),
                    view.point(5.67464, 23.8885),
                    view.point(6.97507, 24.0303),
                );
                path.close();
                path.move_to(view.point(40.4791, 18.1685));
                path.cubic_bezier_to(
                    view.point(39.9589, 16.3721),
                    view.point(40.9756, 17.7194),
                    view.point(40.5736, 16.3721),
                );
                path.cubic_bezier_to(
                    view.point(39.2259, 18.0739),
                    view.point(39.1077, 16.3721),
                    view.point(38.6348, 17.4594),
                );
                path.cubic_bezier_to(
                    view.point(40.4791, 18.1685),
                    view.point(39.5569, 18.4285),
                    view.point(40.1481, 18.4758),
                );
                path.close();
                path.move_to(view.point(39.1786, 13.9139));
                path.cubic_bezier_to(
                    view.point(37.5708, 13.843),
                    view.point(39.1313, 13.8667),
                    view.point(38.422, 13.843),
                );
                path.cubic_bezier_to(
                    view.point(34.9936, 11.3612),
                    view.point(35.1827, 13.843),
                    view.point(34.9936, 13.6539),
                );
                path.cubic_bezier_to(
                    view.point(35.4665, 8.99757),
                    view.point(34.9936, 9.89576),
                    view.point(34.9936, 9.89576),
                );
                path.cubic_bezier_to(
                    view.point(39.3205, 5.66485),
                    view.point(36.3177, 7.41394),
                    view.point(37.9964, 5.94848),
                );
                path.cubic_bezier_to(
                    view.point(42.3943, 5.87757),
                    view.point(40.1717, 5.49939),
                    view.point(41.7322, 5.59394),
                );
                path.cubic_bezier_to(
                    view.point(45.0897, 7.74485),
                    view.point(43.1982, 6.20848),
                    view.point(44.9006, 7.3903),
                );
                path.cubic_bezier_to(
                    view.point(44.5459, 8.90303),
                    view.point(45.3734, 8.26485),
                    view.point(45.1843, 8.64303),
                );
                path.cubic_bezier_to(
                    view.point(42.276, 8.64303),
                    view.point(43.8839, 9.13939),
                    view.point(43.3637, 9.09212),
                );
                path.cubic_bezier_to(
                    view.point(39.9352, 8.61939),
                    view.point(41.3066, 8.24121),
                    view.point(40.5027, 8.21757),
                );
                path.cubic_bezier_to(
                    view.point(39.6988, 12.2594),
                    view.point(38.7057, 9.44667),
                    view.point(38.6112, 10.7467),
                );
                path.cubic_bezier_to(
                    view.point(39.9589, 13.7721),
                    view.point(40.1244, 12.8503),
                    view.point(40.219, 13.4648),
                );
                path.cubic_bezier_to(
                    view.point(39.1786, 13.9139),
                    view.point(39.7934, 13.9612),
                    view.point(39.3205, 14.0558),
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
