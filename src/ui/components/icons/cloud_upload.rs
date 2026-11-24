use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct CloudUpload {
    width: Pixels,
    height: Pixels,
    first_color: Rgba,
    second_color: Rgba,
}

impl CloudUpload {
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

impl RenderOnce for CloudUpload {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(25.9935, 25.9922));
                path.cubic_bezier_to(
                    view.point(27.6646, 26.6847),
                    view.point(26.6198, 25.9923),
                    view.point(27.2215, 26.2421),
                );
                path.line_to(view.point(37.1192, 36.1392));
                path.cubic_bezier_to(
                    view.point(37.1192, 39.4816),
                    view.point(38.042, 37.0621),
                    view.point(38.0416, 38.5585),
                );
                path.cubic_bezier_to(
                    view.point(33.7768, 39.4816),
                    view.point(36.1961, 40.4045),
                    view.point(34.6999, 40.4046),
                );
                path.line_to(view.point(28.3571, 34.0618));
                path.line_to(view.point(28.3571, 47.2649));
                path.cubic_bezier_to(
                    view.point(25.9935, 49.6286),
                    view.point(28.3571, 48.5702),
                    view.point(27.2987, 49.6283),
                );
                path.cubic_bezier_to(
                    view.point(23.6298, 47.2649),
                    view.point(24.6884, 49.6281),
                    view.point(23.6298, 48.5701),
                );
                path.line_to(view.point(23.6298, 34.0618));
                path.line_to(view.point(18.2101, 39.4816));
                path.cubic_bezier_to(
                    view.point(14.8677, 39.4816),
                    view.point(17.287, 40.4045),
                    view.point(15.7908, 40.4046),
                );
                path.cubic_bezier_to(
                    view.point(14.8677, 36.1392),
                    view.point(13.9456, 38.5585),
                    view.point(13.9451, 37.062),
                );
                path.line_to(view.point(24.3223, 26.6847));
                path.cubic_bezier_to(
                    view.point(25.9935, 25.9922),
                    view.point(24.7653, 26.242),
                    view.point(25.3672, 25.9924),
                );
                path.close();

                if let Ok(path) = path.build() {
                    window.paint_path(path, self.first_color);
                }

                let mut path = PathBuilder::fill();
                path.move_to(view.point(17.4137, 2.42279));
                path.cubic_bezier_to(
                    view.point(25.8203, 3.67155),
                    view.point(20.2738, 2.19604),
                    view.point(23.1497, 2.62304),
                );
                path.cubic_bezier_to(
                    view.point(32.8305, 8.47499),
                    view.point(28.4907, 4.7201),
                    view.point(30.8888, 6.36309),
                );
                path.cubic_bezier_to(
                    view.point(36.429, 14.174),
                    view.point(34.3666, 10.1461),
                    view.point(35.5803, 12.0793),
                );
                path.line_to(view.point(38.9935, 14.174));
                path.cubic_bezier_to(
                    view.point(46.7399, 16.7316),
                    view.point(41.7825, 14.1738),
                    view.point(44.4996, 15.0702),
                );
                path.cubic_bezier_to(
                    view.point(51.4349, 23.4001),
                    view.point(48.9798, 18.3926),
                    view.point(50.625, 20.7317),
                );
                path.cubic_bezier_to(
                    view.point(51.234, 31.5551),
                    view.point(52.2444, 26.0689),
                    view.point(52.1737, 28.9292),
                );
                path.cubic_bezier_to(
                    view.point(46.2159, 37.9858),
                    view.point(50.2942, 34.1809),
                    view.point(48.535, 36.4366),
                );
                path.cubic_bezier_to(
                    view.point(42.9382, 37.3326),
                    view.point(45.1306, 38.7103),
                    view.point(43.6633, 38.4176),
                );
                path.cubic_bezier_to(
                    view.point(43.5915, 34.0549),
                    view.point(42.2135, 36.2471),
                    view.point(42.5061, 34.7799),
                );
                path.cubic_bezier_to(
                    view.point(46.7838, 29.9624),
                    view.point(45.067, 33.0691),
                    view.point(46.1857, 31.6331),
                );
                path.cubic_bezier_to(
                    view.point(46.913, 24.7735),
                    view.point(47.3816, 28.2916),
                    view.point(47.428, 26.4716),
                );
                path.cubic_bezier_to(
                    view.point(43.9239, 20.5286),
                    view.point(46.3977, 23.0752),
                    view.point(45.3494, 21.5858),
                );
                path.cubic_bezier_to(
                    view.point(38.9935, 18.9013),
                    view.point(42.4983, 19.4715),
                    view.point(40.7682, 18.9012),
                );
                path.line_to(view.point(34.7625, 18.9013));
                path.cubic_bezier_to(
                    view.point(32.4981, 17.214),
                    view.point(33.7179, 18.9009),
                    view.point(32.7969, 18.215),
                );
                path.cubic_bezier_to(
                    view.point(29.3496, 11.6742),
                    view.point(31.8824, 15.1526),
                    view.point(30.8056, 13.258),
                );
                path.cubic_bezier_to(
                    view.point(24.0915, 8.07105),
                    view.point(27.8933, 10.0902),
                    view.point(26.0944, 8.85744),
                );
                path.cubic_bezier_to(
                    view.point(17.7877, 7.13391),
                    view.point(22.0887, 7.28488),
                    view.point(19.9324, 6.96388),
                );
                path.cubic_bezier_to(
                    view.point(11.7101, 9.05205),
                    view.point(15.6429, 7.30409),
                    view.point(13.5638, 7.96005),
                );
                path.cubic_bezier_to(
                    view.point(7.08667, 13.44),
                    view.point(9.85628, 10.1442),
                    view.point(8.27512, 11.6464),
                );
                path.cubic_bezier_to(
                    view.point(4.84999, 19.4068),
                    view.point(5.89863, 15.2335),
                    view.point(5.13301, 17.2743),
                );
                path.cubic_bezier_to(
                    view.point(5.45475, 25.7522),
                    view.point(4.56708, 21.5398),
                    view.point(4.77512, 23.7106),
                );
                path.cubic_bezier_to(
                    view.point(8.77399, 31.1927),
                    view.point(6.13452, 27.7936),
                    view.point(7.26901, 29.655),
                );
                path.cubic_bezier_to(
                    view.point(8.73937, 34.535),
                    view.point(9.68656, 32.1254),
                    view.point(9.67151, 33.622),
                );
                path.cubic_bezier_to(
                    view.point(5.39704, 34.4981),
                    view.point(7.80659, 35.448),
                    view.point(6.31017, 35.4306),
                );
                path.cubic_bezier_to(
                    view.point(0.967531, 27.2456),
                    view.point(3.39019, 32.4476),
                    view.point(1.87388, 29.9678),
                );
                path.cubic_bezier_to(
                    view.point(0.164264, 18.7859),
                    view.point(0.061336, 24.5235),
                    view.point(-0.21291, 21.6299),
                );
                path.cubic_bezier_to(
                    view.point(3.14651, 10.8294),
                    view.point(0.541572, 15.9421),
                    view.point(1.56219, 13.2209),
                );
                path.cubic_bezier_to(
                    view.point(9.31181, 4.98032),
                    view.point(4.73114, 8.43782),
                    view.point(6.83998, 6.43661),
                );
                path.cubic_bezier_to(
                    view.point(17.4137, 2.42279),
                    view.point(11.7834, 3.52441),
                    view.point(14.5542, 2.64971),
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
