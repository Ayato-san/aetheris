use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct RegionsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl RegionsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for RegionsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(26.0, 0.0));
                path.cubic_bezier_to(
                    view.point(11.5552, 4.38179),
                    view.point(20.8577, 0.0),
                    view.point(15.8309, 1.52487),
                );
                path.cubic_bezier_to(
                    view.point(1.97914, 16.0502),
                    view.point(7.27951, 7.23871),
                    view.point(3.94702, 11.2993),
                );
                path.cubic_bezier_to(
                    view.point(0.499594, 31.0723),
                    view.point(0.0112654, 20.8011),
                    view.point(-0.503621, 26.0288),
                );
                path.cubic_bezier_to(
                    view.point(7.61523, 44.3848),
                    view.point(1.50281, 36.1158),
                    view.point(3.97907, 40.7486),
                );
                path.cubic_bezier_to(
                    view.point(20.9277, 51.5004),
                    view.point(11.2514, 48.0209),
                    view.point(15.8842, 50.4972),
                );
                path.cubic_bezier_to(
                    view.point(35.9498, 50.0209),
                    view.point(25.9712, 52.5036),
                    view.point(31.1989, 51.9887),
                );
                path.cubic_bezier_to(
                    view.point(47.6182, 40.4448),
                    view.point(40.7006, 48.053),
                    view.point(44.7613, 44.7205),
                );
                path.cubic_bezier_to(
                    view.point(52.0, 26.0),
                    view.point(50.4751, 36.1691),
                    view.point(52.0, 31.1423),
                );
                path.cubic_bezier_to(
                    view.point(44.3763, 7.62367),
                    view.point(51.9924, 19.1067),
                    view.point(49.2506, 12.498),
                );
                path.cubic_bezier_to(
                    view.point(26.0, 0.0),
                    view.point(39.502, 2.74938),
                    view.point(32.8933, 0.00764646),
                );
                path.close();
                path.move_to(view.point(46.2222, 26.0));
                path.cubic_bezier_to(
                    view.point(45.1461, 32.5),
                    view.point(46.223, 28.2111),
                    view.point(45.8594, 30.4071),
                );
                path.line_to(view.point(35.3889, 26.4815));
                path.cubic_bezier_to(
                    view.point(33.5111, 25.8122),
                    view.point(34.8176, 26.1275),
                    view.point(34.1775, 25.8994),
                );
                path.line_to(view.point(28.0174, 25.0707));
                path.cubic_bezier_to(
                    view.point(25.5162, 25.4113),
                    view.point(27.1682, 24.9614),
                    view.point(26.3053, 25.0789),
                );
                path.cubic_bezier_to(
                    view.point(23.5252, 26.963),
                    view.point(24.7272, 25.7436),
                    view.point(24.0402, 26.279),
                );
                path.line_to(view.point(22.5454, 26.963));
                path.line_to(view.point(21.893, 25.6148));
                path.cubic_bezier_to(
                    view.point(20.5354, 23.9236),
                    view.point(21.5758, 24.9546),
                    view.point(21.1114, 24.376),
                );
                path.cubic_bezier_to(
                    view.point(18.5707, 23.0052),
                    view.point(19.9594, 23.4712),
                    view.point(19.2873, 23.1569),
                );
                path.line_to(view.point(17.9256, 22.8704));
                path.line_to(view.point(18.8885, 21.1852));
                path.line_to(view.point(22.3528, 21.1852));
                path.cubic_bezier_to(
                    view.point(24.6783, 20.5857),
                    view.point(23.1662, 21.185),
                    view.point(23.9662, 20.9788),
                );
                path.line_to(view.point(27.6274, 18.9583));
                path.cubic_bezier_to(
                    view.point(28.5278, 18.3131),
                    view.point(27.9507, 18.7776),
                    view.point(28.2527, 18.5612),
                );
                path.line_to(view.point(35.0085, 12.4559));
                path.cubic_bezier_to(
                    view.point(36.1997, 10.7925),
                    view.point(35.5202, 11.9932),
                    view.point(35.9264, 11.4259),
                );
                path.cubic_bezier_to(
                    view.point(36.5926, 8.78463),
                    view.point(36.473, 10.1591),
                    view.point(36.607, 9.47433),
                );
                path.cubic_bezier_to(
                    view.point(43.6488, 16.1386),
                    view.point(39.5339, 10.5933),
                    view.point(41.9632, 13.125),
                );
                path.cubic_bezier_to(
                    view.point(46.2222, 26.0),
                    view.point(45.3345, 19.1521),
                    view.point(46.2204, 22.5471),
                );
                path.close();
                path.move_to(view.point(29.0719, 6.01852));
                path.line_to(view.point(30.5644, 8.69074));
                path.line_to(view.point(24.7361, 13.963));
                path.line_to(view.point(22.1048, 15.4074));
                path.line_to(view.point(18.33, 15.4074));
                path.cubic_bezier_to(
                    view.point(15.9156, 16.058),
                    view.point(17.482, 15.4084),
                    view.point(16.6493, 15.6328),
                );
                path.cubic_bezier_to(
                    view.point(14.1507, 17.8293),
                    view.point(15.182, 16.4831),
                    view.point(14.5732, 17.094),
                );
                path.line_to(view.point(13.0698, 19.7191));
                path.line_to(view.point(11.4785, 15.4748));
                path.line_to(view.point(13.8643, 9.83666));
                path.cubic_bezier_to(
                    view.point(21.0824, 6.3917),
                    view.point(16.0166, 8.21868),
                    view.point(18.4708, 7.0474),
                );
                path.cubic_bezier_to(
                    view.point(29.0719, 6.01852),
                    view.point(23.6941, 5.736),
                    view.point(26.4105, 5.60912),
                );
                path.close();
                path.move_to(view.point(5.77779, 26.0));
                path.cubic_bezier_to(
                    view.point(6.83705, 19.5554),
                    view.point(5.7772, 23.8084),
                    view.point(6.13501, 21.6315),
                );
                path.line_to(view.point(8.70038, 24.5267));
                path.cubic_bezier_to(
                    view.point(10.0606, 26.4841),
                    view.point(8.98154, 25.2833),
                    view.point(9.44943, 25.9567),
                );
                path.cubic_bezier_to(
                    view.point(12.1959, 27.5431),
                    view.point(10.6717, 27.0114),
                    view.point(11.4063, 27.3757),
                );
                path.line_to(view.point(16.8976, 28.5543));
                path.line_to(view.point(17.6198, 30.0228));
                path.cubic_bezier_to(
                    view.point(18.9576, 31.6881),
                    view.point(17.9362, 30.6706),
                    view.point(18.3933, 31.2395),
                );
                path.cubic_bezier_to(
                    view.point(20.8819, 32.6155),
                    view.point(19.522, 32.1366),
                    view.point(20.1794, 32.4535),
                );
                path.line_to(view.point(19.6782, 35.2926));
                path.cubic_bezier_to(
                    view.point(19.3258, 38.035),
                    view.point(19.2951, 36.1526),
                    view.point(19.1726, 37.1061),
                );
                path.cubic_bezier_to(
                    view.point(20.54, 40.5191),
                    view.point(19.4789, 38.9639),
                    view.point(19.9011, 39.8276),
                );
                path.line_to(view.point(20.5906, 40.572));
                path.line_to(view.point(24.9624, 45.0739));
                path.line_to(view.point(24.7482, 46.1789));
                path.cubic_bezier_to(
                    view.point(11.2722, 39.8412),
                    view.point(19.6141, 45.8553),
                    view.point(14.7958, 43.5892),
                );
                path.cubic_bezier_to(
                    view.point(5.77779, 26.0),
                    view.point(7.74864, 36.0931),
                    view.point(5.78412, 31.1442),
                );
                path.close();
                path.move_to(view.point(30.7306, 45.6589));
                path.cubic_bezier_to(
                    view.point(30.6209, 43.4342),
                    view.point(30.8674, 44.9189),
                    view.point(30.8299, 44.1572),
                );
                path.cubic_bezier_to(
                    view.point(29.5269, 41.4941),
                    view.point(30.412, 42.7112),
                    view.point(30.0374, 42.047),
                );
                path.line_to(view.point(29.4763, 41.4411));
                path.line_to(view.point(25.2104, 37.0476));
                path.line_to(view.point(27.9476, 30.9039));
                path.line_to(view.point(32.5217, 31.5202));
                path.line_to(view.point(42.4956, 37.6543));
                path.cubic_bezier_to(
                    view.point(30.7306, 45.6589),
                    view.point(39.6701, 41.6565),
                    view.point(35.4907, 44.5001),
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
