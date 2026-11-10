use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct TitlesIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl TitlesIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for TitlesIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(16.916, 51.8081));
                path.cubic_bezier_to(
                    view.point(17.7912, 47.3134),
                    view.point(16.6085, 51.5006),
                    view.point(16.774, 50.6726),
                );
                path.cubic_bezier_to(
                    view.point(18.5008, 44.8531),
                    view.point(17.9804, 46.6983),
                    view.point(18.2879, 45.5864),
                );
                path.cubic_bezier_to(
                    view.point(19.2104, 42.3218),
                    view.point(18.69, 44.1197),
                    view.point(19.0212, 42.9842),
                );
                path.cubic_bezier_to(
                    view.point(19.3524, 39.6013),
                    view.point(19.5889, 40.9734),
                    view.point(19.6362, 40.1454),
                );
                path.cubic_bezier_to(
                    view.point(14.3613, 35.2721),
                    view.point(19.1631, 39.2228),
                    view.point(17.6256, 37.898),
                );
                path.cubic_bezier_to(
                    view.point(10.1035, 31.3451),
                    view.point(10.7422, 32.3387),
                    view.point(10.1035, 31.7473),
                );
                path.cubic_bezier_to(
                    view.point(12.2561, 30.4698),
                    view.point(10.1035, 30.801),
                    view.point(10.3637, 30.7064),
                );
                path.cubic_bezier_to(
                    view.point(15.8279, 30.0203),
                    view.point(13.1549, 30.3515),
                    view.point(14.7634, 30.1623),
                );
                path.cubic_bezier_to(
                    view.point(19.6126, 29.6182),
                    view.point(16.8923, 29.8784),
                    view.point(18.5954, 29.6891),
                );
                path.cubic_bezier_to(
                    view.point(24.3434, 25.9514),
                    view.point(22.9478, 29.358),
                    view.point(22.9478, 29.358),
                );
                path.cubic_bezier_to(
                    view.point(26.8508, 19.848),
                    view.point(25.2896, 23.6567),
                    view.point(25.6207, 22.8524),
                );
                path.cubic_bezier_to(
                    view.point(23.8703, 16.3704),
                    view.point(27.7733, 17.6006),
                    view.point(27.7733, 17.6006),
                );
                path.cubic_bezier_to(
                    view.point(20.2749, 15.2349),
                    view.point(22.7822, 16.0392),
                    view.point(21.1737, 15.5188),
                );
                path.cubic_bezier_to(
                    view.point(17.7439, 14.4542),
                    view.point(19.3997, 14.951),
                    view.point(18.2643, 14.5962),
                );
                path.cubic_bezier_to(
                    view.point(15.5677, 13.0112),
                    view.point(16.6558, 14.1467),
                    view.point(15.8042, 13.5789),
                );
                path.cubic_bezier_to(
                    view.point(15.4494, 9.08417),
                    view.point(15.4494, 12.7273),
                    view.point(15.4257, 11.6391),
                );
                path.cubic_bezier_to(
                    view.point(16.3719, 4.61305),
                    view.point(15.4967, 5.20447),
                    view.point(15.5204, 5.06253),
                );
                path.cubic_bezier_to(
                    view.point(39.5058, 4.61305),
                    view.point(16.845, 4.37649),
                    view.point(39.0327, 4.37649),
                );
                path.cubic_bezier_to(
                    view.point(40.4283, 9.08417),
                    view.point(40.3574, 5.06253),
                    view.point(40.381, 5.20447),
                );
                path.cubic_bezier_to(
                    view.point(40.3101, 13.0112),
                    view.point(40.452, 11.6391),
                    view.point(40.4283, 12.7273),
                );
                path.cubic_bezier_to(
                    view.point(38.1339, 14.4542),
                    view.point(40.0735, 13.5789),
                    view.point(39.222, 14.1467),
                );
                path.cubic_bezier_to(
                    view.point(32.102, 16.3468),
                    view.point(37.2113, 14.7145),
                    view.point(35.8867, 15.1403),
                );
                path.cubic_bezier_to(
                    view.point(30.0441, 16.9618),
                    view.point(31.0849, 16.7016),
                    view.point(30.1387, 16.9618),
                );
                path.cubic_bezier_to(
                    view.point(28.5302, 18.0974),
                    view.point(29.6656, 16.9618),
                    view.point(28.6012, 17.7425),
                );
                path.cubic_bezier_to(
                    view.point(29.0743, 19.9189),
                    view.point(28.4593, 18.3339),
                    view.point(28.6248, 18.878),
                );
                path.cubic_bezier_to(
                    view.point(30.0677, 22.3556),
                    view.point(29.4054, 20.7469),
                    view.point(29.8549, 21.8351),
                );
                path.cubic_bezier_to(
                    view.point(32.9772, 28.8375),
                    view.point(31.6999, 26.4955),
                    view.point(32.6224, 28.5063),
                );
                path.cubic_bezier_to(
                    view.point(36.2652, 29.6182),
                    view.point(33.5213, 29.3106),
                    view.point(34.018, 29.4289),
                );
                path.cubic_bezier_to(
                    view.point(40.0499, 30.0203),
                    view.point(37.2823, 29.6891),
                    view.point(38.9854, 29.8784),
                );
                path.cubic_bezier_to(
                    view.point(43.6217, 30.4698),
                    view.point(41.1143, 30.1623),
                    view.point(42.7228, 30.3515),
                );
                path.cubic_bezier_to(
                    view.point(45.7742, 31.3451),
                    view.point(45.514, 30.7064),
                    view.point(45.7742, 30.801),
                );
                path.cubic_bezier_to(
                    view.point(41.5164, 35.2721),
                    view.point(45.7742, 31.7473),
                    view.point(45.1355, 32.3387),
                );
                path.cubic_bezier_to(
                    view.point(36.5254, 39.6013),
                    view.point(38.2521, 37.898),
                    view.point(36.7146, 39.2228),
                );
                path.cubic_bezier_to(
                    view.point(36.6673, 42.3218),
                    view.point(36.2415, 40.1454),
                    view.point(36.2888, 40.9734),
                );
                path.cubic_bezier_to(
                    view.point(37.3769, 44.8531),
                    view.point(36.8565, 42.9842),
                    view.point(37.1877, 44.1197),
                );
                path.cubic_bezier_to(
                    view.point(38.1575, 47.5263),
                    view.point(37.5898, 45.5864),
                    view.point(37.9446, 46.7929),
                );
                path.cubic_bezier_to(
                    view.point(35.8631, 50.7199),
                    view.point(39.695, 52.6361),
                    view.point(39.6004, 52.7781),
                );
                path.cubic_bezier_to(
                    view.point(27.9389, 46.722),
                    view.point(28.2227, 46.5564),
                    view.point(28.5775, 46.722),
                );
                path.cubic_bezier_to(
                    view.point(24.7928, 48.1177),
                    view.point(27.4185, 46.722),
                    view.point(26.969, 46.9349),
                );
                path.cubic_bezier_to(
                    view.point(16.916, 51.8081),
                    view.point(17.1525, 52.3049),
                    view.point(17.3417, 52.234),
                );
                path.close();
                path.move_to(view.point(9.9143, 11.9703));
                path.cubic_bezier_to(
                    view.point(7.73811, 8.20887),
                    view.point(7.6908, 11.0713),
                    view.point(7.78541, 11.2133),
                );
                path.cubic_bezier_to(
                    view.point(10.482, 4.4238),
                    view.point(7.6908, 4.56574),
                    view.point(7.90368, 4.28186),
                );
                path.cubic_bezier_to(
                    view.point(12.8947, 8.72932),
                    view.point(12.9184, 4.54208),
                    view.point(12.8947, 4.51843),
                );
                path.cubic_bezier_to(
                    view.point(9.9143, 11.9703),
                    view.point(12.8947, 13.0348),
                    view.point(12.8001, 13.1531),
                );
                path.close();
                path.move_to(view.point(43.2432, 12.2305));
                path.cubic_bezier_to(
                    view.point(42.983, 8.72932),
                    view.point(42.983, 11.8993),
                    view.point(42.983, 11.7101),
                );
                path.cubic_bezier_to(
                    view.point(45.3957, 4.4238),
                    view.point(42.983, 4.51843),
                    view.point(42.9593, 4.54208),
                );
                path.cubic_bezier_to(
                    view.point(48.1396, 8.20887),
                    view.point(47.974, 4.28186),
                    view.point(48.1869, 4.56574),
                );
                path.cubic_bezier_to(
                    view.point(45.9634, 11.9703),
                    view.point(48.0923, 11.2133),
                    view.point(48.1869, 11.0713),
                );
                path.cubic_bezier_to(
                    view.point(43.2432, 12.2305),
                    view.point(44.2367, 12.68),
                    view.point(43.6453, 12.7273),
                );
                path.close();
                path.move_to(view.point(4.82863, 4.99156));
                path.cubic_bezier_to(
                    view.point(4.07169, 2.62589),
                    view.point(4.45016, 4.63671),
                    view.point(4.28458, 4.16358),
                );
                path.cubic_bezier_to(
                    view.point(4.42651, 0.260224),
                    view.point(3.83515, 1.08821),
                    view.point(3.90611, 0.662387),
                );
                path.line_to(view.point(4.80497, 0.0));
                path.line_to(view.point(27.9389, 0.0));
                path.line_to(view.point(51.0728, 0.0));
                path.line_to(view.point(51.4512, 0.260224));
                path.cubic_bezier_to(
                    view.point(51.806, 2.72052),
                    view.point(51.9716, 0.662387),
                    view.point(52.0426, 1.11186),
                );
                path.cubic_bezier_to(
                    view.point(50.8126, 5.18081),
                    view.point(51.5932, 4.30552),
                    view.point(51.2856, 4.99156),
                );
                path.cubic_bezier_to(
                    view.point(49.5116, 4.44746),
                    view.point(50.1975, 5.39372),
                    view.point(49.8427, 5.18081),
                );
                path.cubic_bezier_to(
                    view.point(48.9439, 3.57216),
                    view.point(49.3223, 4.09261),
                    view.point(49.0858, 3.69044),
                );
                path.cubic_bezier_to(
                    view.point(6.95751, 3.52485),
                    view.point(48.6127, 3.26462),
                    view.point(7.33598, 3.24097),
                );
                path.cubic_bezier_to(
                    view.point(6.48443, 4.23455),
                    view.point(6.8629, 3.61947),
                    view.point(6.62635, 3.95067),
                );
                path.cubic_bezier_to(
                    view.point(4.82863, 4.99156),
                    view.point(5.96403, 5.2991),
                    view.point(5.39633, 5.53567),
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
