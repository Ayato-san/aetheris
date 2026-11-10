use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct AuctionHouseIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl AuctionHouseIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for AuctionHouseIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(51.9999, 16.3704));
                path.cubic_bezier_to(
                    view.point(51.894, 15.5759),
                    view.point(52.0017, 16.1019),
                    view.point(51.9661, 15.8345),
                );
                path.line_to(view.point(48.437, 3.49315));
                path.cubic_bezier_to(
                    view.point(46.7072, 0.977396),
                    view.point(48.1473, 2.48972),
                    view.point(47.5404, 1.6071),
                );
                path.cubic_bezier_to(
                    view.point(43.8148, 0.0),
                    view.point(45.8739, 0.347696),
                    view.point(44.8592, 0.00478865),
                );
                path.line_to(view.point(8.18518, 0.0));
                path.cubic_bezier_to(
                    view.point(5.29278, 0.977396),
                    view.point(7.14077, 0.00478865),
                    view.point(6.12601, 0.347696),
                );
                path.cubic_bezier_to(
                    view.point(3.56296, 3.49315),
                    view.point(4.45954, 1.6071),
                    view.point(3.85266, 2.48972),
                );
                path.line_to(view.point(0.110741, 15.5759));
                path.cubic_bezier_to(
                    view.point(8.73919e-07, 16.3704),
                    view.point(0.0370644, 15.8343),
                    view.point(-0.000208107, 16.1017),
                );
                path.line_to(view.point(8.73919e-07, 20.2222));
                path.cubic_bezier_to(
                    view.point(1.01094, 24.7368),
                    view.point(-0.00025385, 21.7832),
                    view.point(0.34498, 23.325),
                );
                path.cubic_bezier_to(
                    view.point(3.85185, 28.3881),
                    view.point(1.67689, 26.1486),
                    view.point(2.64706, 27.3956),
                );
                path.line_to(view.point(3.85185, 45.2593));
                path.cubic_bezier_to(
                    view.point(4.69798, 47.302),
                    view.point(3.85185, 46.0254),
                    view.point(4.15621, 46.7602),
                );
                path.cubic_bezier_to(
                    view.point(6.74073, 48.1482),
                    view.point(5.23975, 47.8438),
                    view.point(5.97455, 48.1482),
                );
                path.line_to(view.point(45.2592, 48.1482));
                path.cubic_bezier_to(
                    view.point(47.302, 47.302),
                    view.point(46.0254, 48.1482),
                    view.point(46.7602, 47.8438),
                );
                path.cubic_bezier_to(
                    view.point(48.1481, 45.2593),
                    view.point(47.8437, 46.7602),
                    view.point(48.1481, 46.0254),
                );
                path.line_to(view.point(48.1481, 28.3881));
                path.cubic_bezier_to(
                    view.point(50.989, 24.7368),
                    view.point(49.3529, 27.3956),
                    view.point(50.323, 26.1486),
                );
                path.cubic_bezier_to(
                    view.point(51.9999, 20.2222),
                    view.point(51.655, 23.325),
                    view.point(52.0002, 21.7832),
                );
                path.line_to(view.point(51.9999, 16.3704));
                path.close();
                path.move_to(view.point(8.91943, 5.77778));
                path.line_to(view.point(43.0925, 5.77778));
                path.line_to(view.point(45.2929, 13.4815));
                path.line_to(view.point(6.71907, 13.4815));
                path.line_to(view.point(8.91943, 5.77778));
                path.close();
                path.move_to(view.point(30.8268, 19.2593));
                path.line_to(view.point(30.8268, 20.2222));
                path.cubic_bezier_to(
                    view.point(29.4166, 23.6268),
                    view.point(30.8268, 21.4992),
                    view.point(30.3195, 22.7239),
                );
                path.cubic_bezier_to(
                    view.point(26.012, 25.037),
                    view.point(28.5136, 24.5298),
                    view.point(27.289, 25.037),
                );
                path.cubic_bezier_to(
                    view.point(22.6074, 23.6268),
                    view.point(24.735, 25.037),
                    view.point(23.5104, 24.5298),
                );
                path.cubic_bezier_to(
                    view.point(21.1972, 20.2222),
                    view.point(21.7045, 22.7239),
                    view.point(21.1972, 21.4992),
                );
                path.line_to(view.point(21.1972, 19.2593));
                path.line_to(view.point(30.8268, 19.2593));
                path.close();
                path.move_to(view.point(7.94444, 24.245));
                path.cubic_bezier_to(
                    view.point(6.35353, 22.507),
                    view.point(7.27838, 23.8064),
                    view.point(6.7317, 23.2091),
                );
                path.cubic_bezier_to(
                    view.point(5.77777, 20.2222),
                    view.point(5.97535, 21.8048),
                    view.point(5.7775, 21.0197),
                );
                path.line_to(view.point(5.77777, 19.2593));
                path.line_to(view.point(15.4074, 19.2593));
                path.line_to(view.point(15.4074, 20.2222));
                path.cubic_bezier_to(
                    view.point(13.9972, 23.6268),
                    view.point(15.4074, 21.4992),
                    view.point(14.9001, 22.7239),
                );
                path.cubic_bezier_to(
                    view.point(10.5926, 25.037),
                    view.point(13.0942, 24.5298),
                    view.point(11.8695, 25.037),
                );
                path.cubic_bezier_to(
                    view.point(8.40907, 24.5074),
                    view.point(9.83289, 25.0374),
                    view.point(9.08417, 24.8558),
                );
                path.cubic_bezier_to(
                    view.point(7.94444, 24.245),
                    view.point(8.263, 24.4052),
                    view.point(8.1074, 24.3173),
                );
                path.close();
                path.move_to(view.point(42.3703, 42.3704));
                path.line_to(view.point(9.62962, 42.3704));
                path.line_to(view.point(9.62962, 30.7691));
                path.cubic_bezier_to(
                    view.point(10.5926, 30.8148),
                    view.point(9.9474, 30.798),
                    view.point(10.2676, 30.8148),
                );
                path.cubic_bezier_to(
                    view.point(14.79, 29.9461),
                    view.point(12.0364, 30.8153),
                    view.point(13.465, 30.5196),
                );
                path.cubic_bezier_to(
                    view.point(18.2963, 27.4806),
                    view.point(16.115, 29.3726),
                    view.point(17.3084, 28.5335),
                );
                path.cubic_bezier_to(
                    view.point(21.8028, 29.9457),
                    view.point(19.2844, 28.5332),
                    view.point(20.4778, 29.3722),
                );
                path.cubic_bezier_to(
                    view.point(26.0, 30.8151),
                    view.point(23.1277, 30.5192),
                    view.point(24.5562, 30.8151),
                );
                path.cubic_bezier_to(
                    view.point(30.1972, 29.9457),
                    view.point(27.4437, 30.8151),
                    view.point(28.8722, 30.5192),
                );
                path.cubic_bezier_to(
                    view.point(33.7037, 27.4806),
                    view.point(31.5221, 29.3722),
                    view.point(32.7155, 28.5332),
                );
                path.cubic_bezier_to(
                    view.point(37.2099, 29.9461),
                    view.point(34.6916, 28.5335),
                    view.point(35.8849, 29.3726),
                );
                path.cubic_bezier_to(
                    view.point(41.4074, 30.8148),
                    view.point(38.535, 30.5196),
                    view.point(39.9635, 30.8153),
                );
                path.cubic_bezier_to(
                    view.point(42.3703, 30.7691),
                    view.point(41.7324, 30.8148),
                    view.point(42.0525, 30.798),
                );
                path.line_to(view.point(42.3703, 42.3704));
                path.close();
                path.move_to(view.point(46.2222, 20.2222));
                path.cubic_bezier_to(
                    view.point(45.6464, 22.507),
                    view.point(46.2224, 21.0197),
                    view.point(46.0246, 21.8048),
                );
                path.cubic_bezier_to(
                    view.point(44.0555, 24.245),
                    view.point(45.2682, 23.2091),
                    view.point(44.7216, 23.8064),
                );
                path.cubic_bezier_to(
                    view.point(43.6029, 24.5074),
                    view.point(43.8968, 24.3182),
                    view.point(43.7453, 24.4061),
                );
                path.cubic_bezier_to(
                    view.point(41.2264, 25.0337),
                    view.point(42.8689, 24.8835),
                    view.point(42.0506, 25.0647),
                );
                path.cubic_bezier_to(
                    view.point(38.8962, 24.3304),
                    view.point(40.4023, 25.0027),
                    view.point(39.5999, 24.7606),
                );
                path.cubic_bezier_to(
                    view.point(37.2077, 22.5772),
                    view.point(38.1925, 23.9003),
                    view.point(37.6111, 23.2966),
                );
                path.cubic_bezier_to(
                    view.point(36.5925, 20.2222),
                    view.point(36.8044, 21.8579),
                    view.point(36.5925, 21.047),
                );
                path.line_to(view.point(36.5925, 19.2593));
                path.line_to(view.point(46.2222, 19.2593));
                path.line_to(view.point(46.2222, 20.2222));
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
