use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct CurrenciesIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl CurrenciesIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for CurrenciesIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(44.2381, 11.6161));
                path.cubic_bezier_to(
                    view.point(26.0, 7.96935),
                    view.point(39.4662, 9.229),
                    view.point(33.1724, 7.96935),
                );
                path.cubic_bezier_to(
                    view.point(7.7619, 11.6161),
                    view.point(18.8276, 7.96935),
                    view.point(12.5338, 9.229),
                );
                path.cubic_bezier_to(
                    view.point(0.0, 21.4176),
                    view.point(2.82862, 14.0816),
                    view.point(0.0, 17.6543),
                );
                path.line_to(view.point(0.0, 32.1762));
                path.cubic_bezier_to(
                    view.point(7.7619, 41.9778),
                    view.point(0.0, 35.9395),
                    view.point(2.82862, 39.5123),
                );
                path.cubic_bezier_to(
                    view.point(26.0, 45.6245),
                    view.point(12.5338, 44.3649),
                    view.point(18.8388, 45.6245),
                );
                path.cubic_bezier_to(
                    view.point(44.2381, 41.9778),
                    view.point(33.1612, 45.6245),
                    view.point(39.4662, 44.3649),
                );
                path.cubic_bezier_to(
                    view.point(52.0, 32.1762),
                    view.point(49.1691, 39.5123),
                    view.point(52.0, 35.9395),
                );
                path.line_to(view.point(52.0, 21.4176));
                path.cubic_bezier_to(
                    view.point(44.2381, 11.6161),
                    view.point(52.0, 17.6543),
                    view.point(49.1714, 14.0816),
                );
                path.close();
                path.move_to(view.point(10.1691, 16.4261));
                path.cubic_bezier_to(
                    view.point(26.0, 13.3487),
                    view.point(14.1386, 14.4424),
                    view.point(19.76, 13.3487),
                );
                path.cubic_bezier_to(
                    view.point(41.8309, 16.4261),
                    view.point(32.24, 13.3487),
                    view.point(37.8614, 14.4424),
                );
                path.cubic_bezier_to(
                    view.point(46.6207, 21.4176),
                    view.point(44.785, 17.9031),
                    view.point(46.6207, 19.8173),
                );
                path.cubic_bezier_to(
                    view.point(41.8309, 26.4092),
                    view.point(46.6207, 23.018),
                    view.point(44.785, 24.9321),
                );
                path.cubic_bezier_to(
                    view.point(26.0, 29.4866),
                    view.point(37.8614, 28.3928),
                    view.point(32.24, 29.4866),
                );
                path.cubic_bezier_to(
                    view.point(10.1691, 26.4092),
                    view.point(19.76, 29.4866),
                    view.point(14.1386, 28.3928),
                );
                path.cubic_bezier_to(
                    view.point(5.37931, 21.4176),
                    view.point(7.215, 24.9321),
                    view.point(5.37931, 23.018),
                );
                path.cubic_bezier_to(
                    view.point(10.1691, 16.4261),
                    view.point(5.37931, 19.8173),
                    view.point(7.215, 17.9031),
                );
                path.close();
                path.move_to(view.point(37.6552, 38.7614));
                path.cubic_bezier_to(
                    view.point(28.6897, 40.1735),
                    view.point(34.7291, 39.5671),
                    view.point(31.7217, 40.0407),
                );
                path.line_to(view.point(28.6897, 34.7942));
                path.cubic_bezier_to(
                    view.point(37.6552, 33.5592),
                    view.point(31.7108, 34.6709),
                    view.point(34.7132, 34.2573),
                );
                path.line_to(view.point(37.6552, 38.7614));
                path.close();
                path.move_to(view.point(14.3448, 33.5704));
                path.cubic_bezier_to(
                    view.point(23.3103, 34.8054),
                    view.point(17.2868, 34.2685),
                    view.point(20.2892, 34.6821),
                );
                path.line_to(view.point(23.3103, 40.1847));
                path.cubic_bezier_to(
                    view.point(14.3448, 38.7726),
                    view.point(20.2783, 40.052),
                    view.point(17.2709, 39.5783),
                );
                path.line_to(view.point(14.3448, 33.5704));
                path.close();
                path.move_to(view.point(5.37931, 32.1762));
                path.line_to(view.point(5.37931, 29.8318));
                path.cubic_bezier_to(
                    view.point(7.7619, 31.2192),
                    view.point(6.1427, 30.3452),
                    view.point(6.93856, 30.8087),
                );
                path.cubic_bezier_to(
                    view.point(8.96552, 31.7795),
                    view.point(8.1519, 31.4142),
                    view.point(8.55534, 31.6002),
                );
                path.line_to(view.point(8.96552, 36.5021));
                path.cubic_bezier_to(
                    view.point(5.37931, 32.1762),
                    view.point(6.72414, 35.1438),
                    view.point(5.37931, 33.5457),
                );
                path.close();
                path.move_to(view.point(43.0345, 36.5021));
                path.line_to(view.point(43.0345, 31.7795));
                path.cubic_bezier_to(
                    view.point(44.2381, 31.2192),
                    view.point(43.4447, 31.6002),
                    view.point(43.8481, 31.4142),
                );
                path.cubic_bezier_to(
                    view.point(46.6207, 29.8318),
                    view.point(45.0614, 30.8087),
                    view.point(45.8573, 30.3452),
                );
                path.line_to(view.point(46.6207, 32.1762));
                path.cubic_bezier_to(
                    view.point(43.0345, 36.5021),
                    view.point(46.6207, 33.5457),
                    view.point(45.2759, 35.1438),
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
