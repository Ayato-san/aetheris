use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct DaevaPassIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl DaevaPassIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for DaevaPassIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(49.3103, 22.3142));
                path.cubic_bezier_to(
                    view.point(51.2122, 21.5264),
                    view.point(50.0237, 22.3142),
                    view.point(50.7078, 22.0308),
                );
                path.cubic_bezier_to(
                    view.point(52.0, 19.6245),
                    view.point(51.7166, 21.022),
                    view.point(52.0, 20.3379),
                );
                path.line_to(view.point(52.0, 12.4521));
                path.cubic_bezier_to(
                    view.point(50.687, 9.28232),
                    view.point(52.0, 11.2632),
                    view.point(51.5277, 10.123),
                );
                path.cubic_bezier_to(
                    view.point(47.5172, 7.96935),
                    view.point(49.8464, 8.44164),
                    view.point(48.7061, 7.96935),
                );
                path.line_to(view.point(4.48276, 7.96935));
                path.cubic_bezier_to(
                    view.point(1.31297, 9.28232),
                    view.point(3.29386, 7.96935),
                    view.point(2.15365, 8.44164),
                );
                path.cubic_bezier_to(
                    view.point(0.0, 12.4521),
                    view.point(0.472289, 10.123),
                    view.point(0.0, 11.2632),
                );
                path.line_to(view.point(0.0, 19.6245));
                path.cubic_bezier_to(
                    view.point(0.787782, 21.5264),
                    view.point(0.0, 20.3379),
                    view.point(0.283374, 21.022),
                );
                path.cubic_bezier_to(
                    view.point(2.68966, 22.3142),
                    view.point(1.29219, 22.0308),
                    view.point(1.97631, 22.3142),
                );
                path.cubic_bezier_to(
                    view.point(5.85944, 23.6271),
                    view.point(3.87856, 22.3142),
                    view.point(5.01876, 22.7865),
                );
                path.cubic_bezier_to(
                    view.point(7.17241, 26.7969),
                    view.point(6.70012, 24.4678),
                    view.point(7.17241, 25.608),
                );
                path.cubic_bezier_to(
                    view.point(5.85944, 29.9667),
                    view.point(7.17241, 27.9858),
                    view.point(6.70012, 29.126),
                );
                path.cubic_bezier_to(
                    view.point(2.68966, 31.2797),
                    view.point(5.01876, 30.8074),
                    view.point(3.87856, 31.2797),
                );
                path.cubic_bezier_to(
                    view.point(0.787782, 32.0675),
                    view.point(1.97631, 31.2797),
                    view.point(1.29219, 31.5631),
                );
                path.cubic_bezier_to(
                    view.point(0.0, 33.9693),
                    view.point(0.283374, 32.5719),
                    view.point(0.0, 33.256),
                );
                path.line_to(view.point(0.0, 41.1418));
                path.cubic_bezier_to(
                    view.point(1.31297, 44.3116),
                    view.point(0.0, 42.3307),
                    view.point(0.472289, 43.4709),
                );
                path.cubic_bezier_to(
                    view.point(4.48276, 45.6245),
                    view.point(2.15365, 45.1522),
                    view.point(3.29386, 45.6245),
                );
                path.line_to(view.point(47.5172, 45.6245));
                path.cubic_bezier_to(
                    view.point(50.687, 44.3116),
                    view.point(48.7061, 45.6245),
                    view.point(49.8464, 45.1522),
                );
                path.cubic_bezier_to(
                    view.point(52.0, 41.1418),
                    view.point(51.5277, 43.4709),
                    view.point(52.0, 42.3307),
                );
                path.line_to(view.point(52.0, 33.9693));
                path.cubic_bezier_to(
                    view.point(51.2122, 32.0675),
                    view.point(52.0, 33.256),
                    view.point(51.7166, 32.5719),
                );
                path.cubic_bezier_to(
                    view.point(49.3103, 31.2797),
                    view.point(50.7078, 31.5631),
                    view.point(50.0237, 31.2797),
                );
                path.cubic_bezier_to(
                    view.point(46.1406, 29.9667),
                    view.point(48.1214, 31.2797),
                    view.point(46.9812, 30.8074),
                );
                path.cubic_bezier_to(
                    view.point(44.8276, 26.7969),
                    view.point(45.2999, 29.126),
                    view.point(44.8276, 27.9858),
                );
                path.cubic_bezier_to(
                    view.point(46.1406, 23.6271),
                    view.point(44.8276, 25.608),
                    view.point(45.2999, 24.4678),
                );
                path.cubic_bezier_to(
                    view.point(49.3103, 22.3142),
                    view.point(46.9812, 22.7865),
                    view.point(48.1214, 22.3142),
                );
                path.close();
                path.move_to(view.point(5.37931, 36.2869));
                path.cubic_bezier_to(
                    view.point(10.5612, 32.7466),
                    view.point(7.44603, 35.7026),
                    view.point(9.26549, 34.4595),
                );
                path.cubic_bezier_to(
                    view.point(12.558, 26.7969),
                    view.point(11.8569, 31.0337),
                    view.point(12.558, 28.9447),
                );
                path.cubic_bezier_to(
                    view.point(10.5612, 20.8473),
                    view.point(12.558, 24.6492),
                    view.point(11.8569, 22.5601),
                );
                path.cubic_bezier_to(
                    view.point(5.37931, 17.3069),
                    view.point(9.26549, 19.1344),
                    view.point(7.44603, 17.8913),
                );
                path.line_to(view.point(5.37931, 13.3487));
                path.line_to(view.point(17.0345, 13.3487));
                path.line_to(view.point(17.0345, 40.2452));
                path.line_to(view.point(5.37931, 40.2452));
                path.line_to(view.point(5.37931, 36.2869));
                path.close();
                path.move_to(view.point(46.6207, 36.2869));
                path.line_to(view.point(46.6207, 40.2452));
                path.line_to(view.point(22.4138, 40.2452));
                path.line_to(view.point(22.4138, 13.3487));
                path.line_to(view.point(46.6207, 13.3487));
                path.line_to(view.point(46.6207, 17.3069));
                path.cubic_bezier_to(
                    view.point(41.4388, 20.8473),
                    view.point(44.554, 17.8913),
                    view.point(42.7345, 19.1344),
                );
                path.cubic_bezier_to(
                    view.point(39.442, 26.7969),
                    view.point(40.1431, 22.5601),
                    view.point(39.442, 24.6492),
                );
                path.cubic_bezier_to(
                    view.point(41.4388, 32.7466),
                    view.point(39.442, 28.9447),
                    view.point(40.1431, 31.0337),
                );
                path.cubic_bezier_to(
                    view.point(46.6207, 36.2869),
                    view.point(42.7345, 34.4595),
                    view.point(44.554, 35.7026),
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
