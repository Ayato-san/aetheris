use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct QuestsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl QuestsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for QuestsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(19.7241, 18.3295));
                path.cubic_bezier_to(
                    view.point(20.5119, 16.4276),
                    view.point(19.7241, 17.6162),
                    view.point(20.0075, 16.932),
                );
                path.cubic_bezier_to(
                    view.point(22.4138, 15.6398),
                    view.point(21.0163, 15.9232),
                    view.point(21.7005, 15.6398),
                );
                path.line_to(view.point(35.8621, 15.6398));
                path.cubic_bezier_to(
                    view.point(37.7639, 16.4276),
                    view.point(36.5754, 15.6398),
                    view.point(37.2595, 15.9232),
                );
                path.cubic_bezier_to(
                    view.point(38.5517, 18.3295),
                    view.point(38.2683, 16.932),
                    view.point(38.5517, 17.6162),
                );
                path.cubic_bezier_to(
                    view.point(37.7639, 20.2314),
                    view.point(38.5517, 19.0428),
                    view.point(38.2683, 19.727),
                );
                path.cubic_bezier_to(
                    view.point(35.8621, 21.0192),
                    view.point(37.2595, 20.7358),
                    view.point(36.5754, 21.0192),
                );
                path.line_to(view.point(22.4138, 21.0192));
                path.cubic_bezier_to(
                    view.point(20.5119, 20.2314),
                    view.point(21.7005, 21.0192),
                    view.point(21.0163, 20.7358),
                );
                path.cubic_bezier_to(
                    view.point(19.7241, 18.3295),
                    view.point(20.0075, 19.727),
                    view.point(19.7241, 19.0428),
                );
                path.close();
                path.move_to(view.point(22.4138, 29.9847));
                path.line_to(view.point(35.8621, 29.9847));
                path.cubic_bezier_to(
                    view.point(37.7639, 29.1969),
                    view.point(36.5754, 29.9847),
                    view.point(37.2595, 29.7013),
                );
                path.cubic_bezier_to(
                    view.point(38.5517, 27.295),
                    view.point(38.2683, 28.6925),
                    view.point(38.5517, 28.0084),
                );
                path.cubic_bezier_to(
                    view.point(37.7639, 25.3931),
                    view.point(38.5517, 26.5817),
                    view.point(38.2683, 25.8976),
                );
                path.cubic_bezier_to(
                    view.point(35.8621, 24.6054),
                    view.point(37.2595, 24.8887),
                    view.point(36.5754, 24.6054),
                );
                path.line_to(view.point(22.4138, 24.6054));
                path.cubic_bezier_to(
                    view.point(20.5119, 25.3931),
                    view.point(21.7005, 24.6054),
                    view.point(21.0163, 24.8887),
                );
                path.cubic_bezier_to(
                    view.point(19.7241, 27.295),
                    view.point(20.0075, 25.8976),
                    view.point(19.7241, 26.5817),
                );
                path.cubic_bezier_to(
                    view.point(20.5119, 29.1969),
                    view.point(19.7241, 28.0084),
                    view.point(20.0075, 28.6925),
                );
                path.cubic_bezier_to(
                    view.point(22.4138, 29.9847),
                    view.point(21.0163, 29.7013),
                    view.point(21.7005, 29.9847),
                );
                path.close();
                path.move_to(view.point(52.0, 40.7433));
                path.cubic_bezier_to(
                    view.point(49.6367, 46.4489),
                    view.point(52.0, 42.8833),
                    view.point(51.1499, 44.9357),
                );
                path.cubic_bezier_to(
                    view.point(43.931, 48.8123),
                    view.point(48.1234, 47.9621),
                    view.point(46.0711, 48.8123),
                );
                path.line_to(view.point(18.8276, 48.8123));
                path.cubic_bezier_to(
                    view.point(13.122, 46.4489),
                    view.point(16.6876, 48.8123),
                    view.point(14.6352, 47.9621),
                );
                path.cubic_bezier_to(
                    view.point(10.7586, 40.7433),
                    view.point(11.6087, 44.9357),
                    view.point(10.7586, 42.8833),
                );
                path.line_to(view.point(10.7586, 12.0536));
                path.cubic_bezier_to(
                    view.point(9.97084, 10.1518),
                    view.point(10.7586, 11.3403),
                    view.point(10.4752, 10.6562),
                );
                path.cubic_bezier_to(
                    view.point(8.06897, 9.36398),
                    view.point(9.46643, 9.64736),
                    view.point(8.78231, 9.36398),
                );
                path.cubic_bezier_to(
                    view.point(6.16709, 10.1518),
                    view.point(7.35562, 9.36398),
                    view.point(6.6715, 9.64736),
                );
                path.cubic_bezier_to(
                    view.point(5.37931, 12.0536),
                    view.point(5.66268, 10.6562),
                    view.point(5.37931, 11.3403),
                );
                path.cubic_bezier_to(
                    view.point(6.1369, 13.5195),
                    view.point(5.37931, 12.8897),
                    view.point(6.13017, 13.5128),
                );
                path.line_to(view.point(6.09655, 13.4881));
                path.cubic_bezier_to(
                    view.point(6.83577, 14.2644),
                    view.point(6.39255, 13.6947),
                    view.point(6.64395, 13.9587),
                );
                path.cubic_bezier_to(
                    view.point(7.21307, 15.2678),
                    view.point(7.02759, 14.5702),
                    view.point(7.1559, 14.9114),
                );
                path.cubic_bezier_to(
                    view.point(7.16853, 16.3389),
                    view.point(7.27024, 15.6242),
                    view.point(7.25509, 15.9885),
                );
                path.cubic_bezier_to(
                    view.point(6.70923, 17.3074),
                    view.point(7.08197, 16.6893),
                    view.point(6.92577, 17.0187),
                );
                path.cubic_bezier_to(
                    view.point(5.90811, 18.0197),
                    view.point(6.49269, 17.5962),
                    view.point(6.22024, 17.8385),
                );
                path.cubic_bezier_to(
                    view.point(4.89244, 18.3625),
                    view.point(5.59599, 18.201),
                    view.point(5.25057, 18.3176),
                );
                path.cubic_bezier_to(
                    view.point(3.82354, 18.2814),
                    view.point(4.53431, 18.4075),
                    view.point(4.17079, 18.3799),
                );
                path.cubic_bezier_to(
                    view.point(2.87121, 17.7893),
                    view.point(3.47629, 18.183),
                    view.point(3.15242, 18.0156),
                );
                path.cubic_bezier_to(
                    view.point(0.0, 12.0536),
                    view.point(2.57534, 17.5719),
                    view.point(0.0, 15.539),
                );
                path.cubic_bezier_to(
                    view.point(2.36335, 6.34802),
                    view.point(0.0, 9.91362),
                    view.point(0.850121, 7.86124),
                );
                path.cubic_bezier_to(
                    view.point(8.06897, 3.98467),
                    view.point(3.87657, 4.83479),
                    view.point(5.92894, 3.98467),
                );
                path.line_to(view.point(38.5517, 3.98467));
                path.cubic_bezier_to(
                    view.point(44.2573, 6.34802),
                    view.point(40.6917, 3.98467),
                    view.point(42.7441, 4.83479),
                );
                path.cubic_bezier_to(
                    view.point(46.6207, 12.0536),
                    view.point(45.7706, 7.86124),
                    view.point(46.6207, 9.91362),
                );
                path.line_to(view.point(46.6207, 34.4674));
                path.line_to(view.point(47.5172, 34.4674));
                path.cubic_bezier_to(
                    view.point(49.131, 35.0054),
                    view.point(48.0992, 34.4674),
                    view.point(48.6655, 34.6562),
                );
                path.cubic_bezier_to(
                    view.point(52.0, 40.7433),
                    view.point(49.4247, 35.225),
                    view.point(52.0, 37.2579),
                );
                path.close();
                path.move_to(view.point(19.8631, 36.3054));
                path.cubic_bezier_to(
                    view.point(20.842, 34.9749),
                    view.point(20.0418, 35.7702),
                    view.point(20.3842, 35.3048),
                );
                path.cubic_bezier_to(
                    view.point(22.4138, 34.4674),
                    view.point(21.2997, 34.6451),
                    view.point(21.8496, 34.4675),
                );
                path.line_to(view.point(41.2414, 34.4674));
                path.line_to(view.point(41.2414, 12.0536));
                path.cubic_bezier_to(
                    view.point(40.4536, 10.1518),
                    view.point(41.2414, 11.3403),
                    view.point(40.958, 10.6562),
                );
                path.cubic_bezier_to(
                    view.point(38.5517, 9.36398),
                    view.point(39.9492, 9.64736),
                    view.point(39.2651, 9.36398),
                );
                path.line_to(view.point(15.6762, 9.36398));
                path.cubic_bezier_to(
                    view.point(16.1379, 12.0536),
                    view.point(15.9821, 10.2277),
                    view.point(16.1383, 11.1373),
                );
                path.line_to(view.point(16.1379, 40.7433));
                path.cubic_bezier_to(
                    view.point(16.9257, 42.6452),
                    view.point(16.1379, 41.4566),
                    view.point(16.4213, 42.1408),
                );
                path.cubic_bezier_to(
                    view.point(18.8276, 43.4329),
                    view.point(17.4301, 43.1496),
                    view.point(18.1142, 43.4329),
                );
                path.cubic_bezier_to(
                    view.point(20.7295, 42.6452),
                    view.point(19.5409, 43.4329),
                    view.point(20.2251, 43.1496),
                );
                path.cubic_bezier_to(
                    view.point(21.5172, 40.7433),
                    view.point(21.2339, 42.1408),
                    view.point(21.5172, 41.4566),
                );
                path.cubic_bezier_to(
                    view.point(20.8, 39.3088),
                    view.point(21.5172, 39.9409),
                    view.point(20.8067, 39.3133),
                );
                path.cubic_bezier_to(
                    view.point(19.8466, 37.9584),
                    view.point(20.3483, 38.9699),
                    view.point(20.0148, 38.4974),
                );
                path.cubic_bezier_to(
                    view.point(19.8631, 36.3054),
                    view.point(19.6784, 37.4193),
                    view.point(19.6842, 36.841),
                );
                path.close();
                path.move_to(view.point(46.6207, 40.7433));
                path.cubic_bezier_to(
                    view.point(46.3428, 39.8467),
                    view.point(46.6127, 40.4245),
                    view.point(46.5164, 40.1142),
                );
                path.line_to(view.point(26.8428, 39.8467));
                path.cubic_bezier_to(
                    view.point(26.901, 40.7433),
                    view.point(26.8822, 40.144),
                    view.point(26.9016, 40.4435),
                );
                path.cubic_bezier_to(
                    view.point(26.4393, 43.4329),
                    view.point(26.9018, 41.6596),
                    view.point(26.7457, 42.5693),
                );
                path.line_to(view.point(43.931, 43.4329));
                path.cubic_bezier_to(
                    view.point(45.8329, 42.6452),
                    view.point(44.6444, 43.4329),
                    view.point(45.3285, 43.1496),
                );
                path.cubic_bezier_to(
                    view.point(46.6207, 40.7433),
                    view.point(46.3373, 42.1408),
                    view.point(46.6207, 41.4566),
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
