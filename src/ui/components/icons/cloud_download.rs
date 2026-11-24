use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct CloudDownload {
    width: Pixels,
    height: Pixels,
    first_color: Rgba,
    second_color: Rgba,
}

impl CloudDownload {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            first_color: color,
            second_color: color,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn with_second_color(mut self, color: Rgba) -> Self {
        self.second_color = color;
        self
    }
}

impl RenderOnce for CloudDownload {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(25.9962, 25.9993));
                path.cubic_bezier_to(
                    view.point(28.3598, 28.363),
                    view.point(27.3012, 25.9994),
                    view.point(28.3593, 27.0581),
                );
                path.line_to(view.point(28.3598, 41.5638));
                path.line_to(view.point(33.7795, 36.144));
                path.cubic_bezier_to(
                    view.point(37.1219, 36.144),
                    view.point(34.7023, 35.2225),
                    view.point(36.1991, 35.2225),
                );
                path.cubic_bezier_to(
                    view.point(37.1219, 39.4887),
                    view.point(38.0447, 37.0669),
                    view.point(38.0444, 38.5656),
                );
                path.line_to(view.point(27.6673, 48.9432));
                path.cubic_bezier_to(
                    view.point(24.325, 48.9432),
                    view.point(26.7444, 49.8656),
                    view.point(25.2479, 49.8656),
                );
                path.line_to(view.point(14.8704, 39.4887));
                path.cubic_bezier_to(
                    view.point(14.8704, 36.144),
                    view.point(13.9479, 38.5656),
                    view.point(13.9475, 37.0669),
                );
                path.cubic_bezier_to(
                    view.point(18.2128, 36.144),
                    view.point(15.7932, 35.2225),
                    view.point(17.29, 35.2225),
                );
                path.line_to(view.point(23.6325, 41.5638));
                path.line_to(view.point(23.6325, 28.363));
                path.cubic_bezier_to(
                    view.point(25.9962, 25.9993),
                    view.point(23.6331, 27.0581),
                    view.point(24.6911, 25.9994),
                );
                path.close();

                if let Ok(path) = path.build() {
                    window.paint_path(path, self.first_color);
                }

                let mut path = PathBuilder::fill();
                path.move_to(view.point(16.8717, 2.47376));
                path.cubic_bezier_to(
                    view.point(25.4837, 3.54247),
                    view.point(19.7858, 2.15793),
                    view.point(22.7354, 2.52364),
                );
                path.cubic_bezier_to(
                    view.point(32.7085, 8.34361),
                    view.point(28.2313, 4.56124),
                    view.point(30.7049, 6.20526),
                );
                path.cubic_bezier_to(
                    view.point(36.4317, 14.1811),
                    view.point(34.3032, 10.0458),
                    view.point(35.5604, 12.0285),
                );
                path.line_to(view.point(38.9962, 14.1811));
                path.line_to(view.point(39.5224, 14.1904));
                path.cubic_bezier_to(
                    view.point(46.7865, 16.771),
                    view.point(42.1443, 14.2963),
                    view.point(44.6788, 15.1942),
                );
                path.cubic_bezier_to(
                    view.point(51.4699, 23.5134),
                    view.point(49.0344, 18.4531),
                    view.point(50.6776, 20.8199),
                );
                path.cubic_bezier_to(
                    view.point(51.1814, 31.7168),
                    view.point(52.2618, 26.2069),
                    view.point(52.1609, 29.0857),
                );
                path.cubic_bezier_to(
                    view.point(46.034, 38.1107),
                    view.point(50.2016, 34.3479),
                    view.point(48.3946, 36.5909),
                );
                path.cubic_bezier_to(
                    view.point(42.7655, 37.4043),
                    view.point(44.9364, 38.8171),
                    view.point(43.4721, 38.5018),
                );
                path.cubic_bezier_to(
                    view.point(43.4742, 34.1359),
                    view.point(42.0596, 36.3068),
                    view.point(42.3768, 34.8423),
                );
                path.cubic_bezier_to(
                    view.point(46.7495, 30.0664),
                    view.point(44.9766, 33.1687),
                    view.point(46.126, 31.7409),
                );
                path.cubic_bezier_to(
                    view.point(46.9342, 24.8452),
                    view.point(47.3727, 28.3921),
                    view.point(47.4382, 26.5592),
                );
                path.cubic_bezier_to(
                    view.point(43.9543, 20.5565),
                    view.point(46.4299, 23.1318),
                    view.point(45.3843, 21.6266),
                );
                path.cubic_bezier_to(
                    view.point(38.9962, 18.9084),
                    view.point(42.5235, 19.4862),
                    view.point(40.783, 18.9082),
                );
                path.line_to(view.point(34.7652, 18.9084));
                path.cubic_bezier_to(
                    view.point(32.5008, 17.2188),
                    view.point(33.72, 18.9083),
                    view.point(32.7995, 18.2204),
                );
                path.cubic_bezier_to(
                    view.point(29.26, 11.5751),
                    view.point(31.8725, 15.1125),
                    view.point(30.7628, 13.1792),
                );
                path.cubic_bezier_to(
                    view.point(23.8403, 7.97429),
                    view.point(27.7572, 9.97126),
                    view.point(25.9011, 8.73838),
                );
                path.cubic_bezier_to(
                    view.point(17.3818, 7.17333),
                    view.point(21.7791, 7.2102),
                    view.point(19.5673, 6.93649),
                );
                path.cubic_bezier_to(
                    view.point(11.2465, 9.34077),
                    view.point(15.1964, 7.4102),
                    view.point(13.0961, 8.15289),
                );
                path.cubic_bezier_to(
                    view.point(6.72237, 14.0196),
                    view.point(9.39701, 10.5287),
                    view.point(7.84673, 12.1307),
                );
                path.cubic_bezier_to(
                    view.point(4.7673, 20.2241),
                    view.point(5.59867, 15.9079),
                    view.point(4.92983, 18.0327),
                );
                path.cubic_bezier_to(
                    view.point(5.78754, 26.6526),
                    view.point(4.60504, 22.4163),
                    view.point(4.95378, 24.6187),
                );
                path.cubic_bezier_to(
                    view.point(9.57305, 31.9453),
                    view.point(6.62147, 28.6864),
                    view.point(7.91866, 30.498),
                );
                path.cubic_bezier_to(
                    view.point(9.79464, 35.2808),
                    view.point(10.5553, 32.8046),
                    view.point(10.6534, 34.2983),
                );
                path.cubic_bezier_to(
                    view.point(6.45924, 35.5047),
                    view.point(8.93514, 36.2633),
                    view.point(7.44175, 36.3642),
                );
                path.cubic_bezier_to(
                    view.point(1.41343, 28.4461),
                    view.point(4.25331, 33.5748),
                    view.point(2.52535, 31.1579),
                );
                path.cubic_bezier_to(
                    view.point(0.0515647, 19.8756),
                    view.point(0.301672, 25.7341),
                    view.point(-0.164789, 22.7986),
                );
                path.cubic_bezier_to(
                    view.point(2.65987, 11.6005),
                    view.point(0.268151, 16.9529),
                    view.point(1.16099, 14.119),
                );
                path.cubic_bezier_to(
                    view.point(8.6913, 5.36136),
                    view.point(4.15903, 9.08196),
                    view.point(6.2252, 6.94536),
                );
                path.cubic_bezier_to(
                    view.point(16.8717, 2.47376),
                    view.point(11.1572, 3.77778),
                    view.point(13.9582, 2.78965),
                );
                path.close();

                if let Ok(path) = path.build() {
                    window.paint_path(path, self.second_color);
                }
            },
        )
        .w(self.width)
        .h(self.height)
    }
}
