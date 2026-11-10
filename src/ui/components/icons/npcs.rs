use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct NpcsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl NpcsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for NpcsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(45.7016, 27.5921));
                path.line_to(view.point(35.9328, 16.5143));
                path.cubic_bezier_to(
                    view.point(33.4303, 14.6246),
                    view.point(35.2352, 15.7213),
                    view.point(34.3838, 15.0784),
                );
                path.cubic_bezier_to(
                    view.point(30.3859, 13.8743),
                    view.point(32.4768, 14.1708),
                    view.point(31.441, 13.9155),
                );
                path.cubic_bezier_to(
                    view.point(32.9993, 9.93419),
                    view.point(31.6438, 12.8632),
                    view.point(32.5569, 11.4865),
                );
                path.cubic_bezier_to(
                    view.point(32.8557, 5.20793),
                    view.point(33.4417, 8.38189),
                    view.point(33.3915, 6.7305),
                );
                path.cubic_bezier_to(
                    view.point(30.0081, 1.43382),
                    view.point(32.32, 3.68536),
                    view.point(31.325, 2.36666),
                );
                path.cubic_bezier_to(
                    view.point(25.5037, 0.0),
                    view.point(28.6912, 0.500988),
                    view.point(27.1173, 0.0),
                );
                path.cubic_bezier_to(
                    view.point(20.9993, 1.43382),
                    view.point(23.89, 0.0),
                    view.point(22.3162, 0.500988),
                );
                path.cubic_bezier_to(
                    view.point(18.1516, 5.20793),
                    view.point(19.6824, 2.36666),
                    view.point(18.6874, 3.68536),
                );
                path.cubic_bezier_to(
                    view.point(18.0081, 9.93419),
                    view.point(17.6159, 6.7305),
                    view.point(17.5657, 8.38189),
                );
                path.cubic_bezier_to(
                    view.point(20.6215, 13.8743),
                    view.point(18.4505, 11.4865),
                    view.point(19.3636, 12.8632),
                );
                path.cubic_bezier_to(
                    view.point(17.5771, 14.6246),
                    view.point(19.5663, 13.9155),
                    view.point(18.5306, 14.1708),
                );
                path.cubic_bezier_to(
                    view.point(15.0746, 16.5143),
                    view.point(16.6235, 15.0784),
                    view.point(15.7722, 15.7213),
                );
                path.line_to(view.point(5.30581, 27.5921));
                path.cubic_bezier_to(
                    view.point(3.8492, 31.23),
                    view.point(4.36438, 28.5681),
                    view.point(3.84165, 29.8737),
                );
                path.cubic_bezier_to(
                    view.point(5.34622, 34.8514),
                    view.point(3.85675, 32.5863),
                    view.point(4.39398, 33.8859),
                );
                path.cubic_bezier_to(
                    view.point(8.9459, 36.3979),
                    view.point(6.29845, 35.8169),
                    view.point(7.59026, 36.3719),
                );
                path.cubic_bezier_to(
                    view.point(12.6021, 34.9902),
                    view.point(10.3015, 36.4238),
                    view.point(11.6136, 35.9186),
                );
                path.line_to(view.point(14.0592, 33.8229));
                path.line_to(view.point(11.158, 44.8444));
                path.cubic_bezier_to(
                    view.point(11.1475, 48.7341),
                    view.point(10.6513, 46.0907),
                    view.point(10.6476, 47.485),
                );
                path.cubic_bezier_to(
                    view.point(13.8385, 51.5419),
                    view.point(11.6474, 49.9831),
                    view.point(12.612, 50.9896),
                );
                path.cubic_bezier_to(
                    view.point(17.7239, 51.6954),
                    view.point(15.0649, 52.0942),
                    view.point(16.4577, 52.1492),
                );
                path.cubic_bezier_to(
                    view.point(20.628, 49.1087),
                    view.point(18.9901, 51.2417),
                    view.point(20.0311, 50.3144),
                );
                path.line_to(view.point(25.5037, 40.7078));
                path.line_to(view.point(30.3794, 49.1087));
                path.cubic_bezier_to(
                    view.point(33.2524, 51.6751),
                    view.point(30.9698, 50.3032),
                    view.point(31.9994, 51.2229),
                );
                path.cubic_bezier_to(
                    view.point(36.199, 51.8564),
                    view.point(34.1964, 52.0192),
                    view.point(35.2199, 52.0822),
                );
                path.cubic_bezier_to(
                    view.point(38.7687, 50.4029),
                    view.point(37.178, 51.6306),
                    view.point(38.0706, 51.1258),
                );
                path.cubic_bezier_to(
                    view.point(40.1322, 47.7838),
                    view.point(39.4668, 49.6801),
                    view.point(39.9404, 48.7703),
                );
                path.cubic_bezier_to(
                    view.point(39.8494, 44.8444),
                    view.point(40.324, 46.7972),
                    view.point(40.2258, 45.7762),
                );
                path.line_to(view.point(36.9482, 33.8229));
                path.line_to(view.point(38.4053, 34.9902));
                path.cubic_bezier_to(
                    view.point(42.0615, 36.3979),
                    view.point(39.3937, 35.9186),
                    view.point(40.7058, 36.4238),
                );
                path.cubic_bezier_to(
                    view.point(45.6612, 34.8514),
                    view.point(43.4171, 36.3719),
                    view.point(44.7089, 35.8169),
                );
                path.cubic_bezier_to(
                    view.point(47.1582, 31.23),
                    view.point(46.6134, 33.8859),
                    view.point(47.1506, 32.5863),
                );
                path.cubic_bezier_to(
                    view.point(45.7016, 27.5921),
                    view.point(47.1657, 29.8737),
                    view.point(46.643, 28.5681),
                );
                path.close();
                path.move_to(view.point(25.5037, 5.21129));
                path.cubic_bezier_to(
                    view.point(26.9471, 5.64928),
                    view.point(26.0175, 5.21129),
                    view.point(26.5198, 5.36371),
                );
                path.cubic_bezier_to(
                    view.point(27.904, 6.81563),
                    view.point(27.3743, 5.93485),
                    view.point(27.7073, 6.34074),
                );
                path.cubic_bezier_to(
                    view.point(28.0518, 8.3172),
                    view.point(28.1006, 7.29051),
                    view.point(28.1521, 7.81306),
                );
                path.cubic_bezier_to(
                    view.point(27.3408, 9.64787),
                    view.point(27.9516, 8.82133),
                    view.point(27.7041, 9.28441),
                );
                path.cubic_bezier_to(
                    view.point(26.0105, 10.3591),
                    view.point(26.9775, 10.0113),
                    view.point(26.5145, 10.2589),
                );
                path.cubic_bezier_to(
                    view.point(24.5094, 10.2112),
                    view.point(25.5066, 10.4594),
                    view.point(24.9842, 10.4079),
                );
                path.cubic_bezier_to(
                    view.point(23.3435, 9.25405),
                    view.point(24.0347, 10.0145),
                    view.point(23.6289, 9.68143),
                );
                path.cubic_bezier_to(
                    view.point(22.9056, 7.81018),
                    view.point(23.058, 8.82666),
                    view.point(22.9056, 8.32419),
                );
                path.cubic_bezier_to(
                    view.point(23.6666, 5.97249),
                    view.point(22.9056, 7.12091),
                    view.point(23.1793, 6.45987),
                );
                path.cubic_bezier_to(
                    view.point(25.5037, 5.21129),
                    view.point(24.1538, 5.4851),
                    view.point(24.8146, 5.21129),
                );
                path.close();
                path.move_to(view.point(41.7417, 31.0096));
                path.line_to(view.point(34.0514, 24.8394));
                path.cubic_bezier_to(
                    view.point(32.5554, 24.2714),
                    view.point(33.6248, 24.4974),
                    view.point(33.1014, 24.2986),
                );
                path.cubic_bezier_to(
                    view.point(31.0105, 24.688),
                    view.point(32.0094, 24.2443),
                    view.point(31.4688, 24.39),
                );
                path.cubic_bezier_to(
                    view.point(30.0025, 25.9311),
                    view.point(30.5521, 24.986),
                    view.point(30.1994, 25.421),
                );
                path.cubic_bezier_to(
                    view.point(29.9139, 27.5293),
                    view.point(29.8057, 26.4412),
                    view.point(29.7746, 27.0005),
                );
                path.line_to(view.point(34.8719, 46.3604));
                path.cubic_bezier_to(
                    view.point(35.03, 46.7935),
                    view.point(34.9117, 46.5092),
                    view.point(34.9646, 46.6541),
                );
                path.cubic_bezier_to(
                    view.point(34.9217, 46.577),
                    view.point(34.9976, 46.7195),
                    view.point(34.9614, 46.6473),
                );
                path.line_to(view.point(27.751, 34.2322));
                path.cubic_bezier_to(
                    view.point(26.8004, 33.2842),
                    view.point(27.5226, 33.8384),
                    view.point(27.1948, 33.5115),
                );
                path.cubic_bezier_to(
                    view.point(25.5037, 32.9374),
                    view.point(26.406, 33.057),
                    view.point(25.9588, 32.9374),
                );
                path.cubic_bezier_to(
                    view.point(24.207, 33.2842),
                    view.point(25.0485, 32.9374),
                    view.point(24.6014, 33.057),
                );
                path.cubic_bezier_to(
                    view.point(23.2563, 34.2322),
                    view.point(23.8125, 33.5115),
                    view.point(23.4847, 33.8384),
                );
                path.line_to(view.point(16.0857, 46.577));
                path.cubic_bezier_to(
                    view.point(15.9774, 46.7935),
                    view.point(16.0459, 46.6473),
                    view.point(16.0098, 46.7195),
                );
                path.cubic_bezier_to(
                    view.point(16.1355, 46.3604),
                    view.point(16.0418, 46.6537),
                    view.point(16.0947, 46.5088),
                );
                path.line_to(view.point(21.087, 27.5358));
                path.cubic_bezier_to(
                    view.point(20.9983, 25.9376),
                    view.point(21.2262, 27.007),
                    view.point(21.1952, 26.4477),
                );
                path.cubic_bezier_to(
                    view.point(19.9904, 24.6945),
                    view.point(20.8015, 25.4275),
                    view.point(20.4488, 24.9925),
                );
                path.cubic_bezier_to(
                    view.point(18.4455, 24.2779),
                    view.point(19.5321, 24.3965),
                    view.point(18.9914, 24.2508),
                );
                path.cubic_bezier_to(
                    view.point(16.9495, 24.8459),
                    view.point(17.8995, 24.3051),
                    view.point(17.376, 24.5039),
                );
                path.line_to(view.point(9.26571, 31.0096));
                path.cubic_bezier_to(
                    view.point(9.04921, 31.198),
                    view.point(9.19013, 31.0684),
                    view.point(9.11785, 31.1313),
                );
                path.cubic_bezier_to(
                    view.point(9.16179, 31.0811),
                    view.point(9.08818, 31.1612),
                    view.point(9.12499, 31.1201),
                );
                path.line_to(view.point(18.9717, 19.9513));
                path.cubic_bezier_to(
                    view.point(19.8514, 19.3019),
                    view.point(19.2156, 19.6749),
                    view.point(19.5154, 19.4535),
                );
                path.cubic_bezier_to(
                    view.point(20.9202, 19.072),
                    view.point(20.1873, 19.1503),
                    view.point(20.5517, 19.072),
                );
                path.line_to(view.point(30.0871, 19.072));
                path.cubic_bezier_to(
                    view.point(31.156, 19.3019),
                    view.point(30.4557, 19.072),
                    view.point(30.82, 19.1503),
                );
                path.cubic_bezier_to(
                    view.point(32.0357, 19.9513),
                    view.point(31.4919, 19.4535),
                    view.point(31.7918, 19.6749),
                );
                path.line_to(view.point(41.8477, 31.0811));
                path.cubic_bezier_to(
                    view.point(41.9603, 31.198),
                    view.point(41.8846, 31.1201),
                    view.point(41.9214, 31.1612),
                );
                path.cubic_bezier_to(
                    view.point(41.7417, 31.0096),
                    view.point(41.891, 31.1312),
                    view.point(41.818, 31.0683),
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
