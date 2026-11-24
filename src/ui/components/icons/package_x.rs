use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct PackageX {
    width: Pixels,
    height: Pixels,
    first_color: Rgba,
    second_color: Rgba,
}

impl PackageX {
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

impl RenderOnce for PackageX {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(47.6703, 30.236));
                path.cubic_bezier_to(
                    view.point(51.2572, 30.236),
                    view.point(48.6608, 29.313),
                    view.point(50.2666, 29.313),
                );
                path.cubic_bezier_to(
                    view.point(51.2572, 33.5786),
                    view.point(52.2476, 31.1591),
                    view.point(52.2476, 32.6555),
                );
                path.line_to(view.point(46.7091, 37.8169));
                path.line_to(view.point(51.2572, 42.0552));
                path.cubic_bezier_to(
                    view.point(51.2572, 45.3979),
                    view.point(52.2476, 42.9783),
                    view.point(52.2476, 44.4747),
                );
                path.cubic_bezier_to(
                    view.point(47.6703, 45.3979),
                    view.point(50.2666, 46.3208),
                    view.point(48.6608, 46.3208),
                );
                path.line_to(view.point(43.1222, 41.1595));
                path.line_to(view.point(38.5742, 45.3979));
                path.cubic_bezier_to(
                    view.point(34.9873, 45.3979),
                    view.point(37.5836, 46.3208),
                    view.point(35.9778, 46.3208),
                );
                path.cubic_bezier_to(
                    view.point(34.9873, 42.0552),
                    view.point(33.9968, 44.4747),
                    view.point(33.9968, 42.9783),
                );
                path.line_to(view.point(39.5353, 37.8169));
                path.line_to(view.point(34.9873, 33.5786));
                path.cubic_bezier_to(
                    view.point(34.9873, 30.236),
                    view.point(33.9968, 32.6555),
                    view.point(33.9968, 31.1591),
                );
                path.cubic_bezier_to(
                    view.point(38.5742, 30.236),
                    view.point(35.9778, 29.313),
                    view.point(37.5836, 29.313),
                );
                path.line_to(view.point(43.1222, 34.4743));
                path.line_to(view.point(47.6703, 30.236));
                path.close();

                if let Ok(path) = path.build() {
                    window.paint_path(path, self.first_color);
                }

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(25.366, 0.0));
                path.cubic_bezier_to(
                    view.point(29.1585, 0.946462),
                    view.point(26.6975, 0.0),
                    view.point(28.0044, 0.327911),
                );
                path.line_to(view.point(29.161, 0.944153));
                path.line_to(view.point(46.9172, 10.3995));
                path.line_to(view.point(46.9271, 10.4065));
                path.cubic_bezier_to(
                    view.point(49.7114, 12.9988),
                    view.point(48.0825, 11.0282),
                    view.point(49.0437, 11.9224),
                );
                path.cubic_bezier_to(
                    view.point(50.732, 16.54),
                    view.point(50.3788, 14.0752),
                    view.point(50.7306, 15.2969),
                );
                path.line_to(view.point(50.732, 22.4519));
                path.cubic_bezier_to(
                    view.point(48.1954, 24.8158),
                    view.point(50.7319, 23.7573),
                    view.point(49.5963, 24.8158),
                );
                path.cubic_bezier_to(
                    view.point(45.6588, 22.4519),
                    view.point(46.7946, 24.8158),
                    view.point(45.6589, 23.7573),
                );
                path.line_to(view.point(45.6588, 17.8674));
                path.line_to(view.point(27.9026, 27.3643));
                path.line_to(view.point(27.9026, 46.4782));
                path.cubic_bezier_to(
                    view.point(30.4367, 47.6532),
                    view.point(28.8942, 46.3574),
                    view.point(29.9113, 46.7934),
                );
                path.cubic_bezier_to(
                    view.point(29.488, 50.8781),
                    view.point(31.1297, 48.7877),
                    view.point(30.7053, 50.2322),
                );
                path.line_to(view.point(29.1585, 51.0512));
                path.line_to(view.point(29.156, 51.0489));
                path.cubic_bezier_to(
                    view.point(25.6459, 51.9838),
                    view.point(28.0833, 51.623),
                    view.point(26.8794, 51.9418),
                );
                path.cubic_bezier_to(
                    view.point(25.366, 52.0),
                    view.point(25.5539, 51.9933),
                    view.point(25.4607, 52.0),
                );
                path.cubic_bezier_to(
                    view.point(25.0836, 51.9838),
                    view.point(25.2705, 52.0),
                    view.point(25.1764, 51.9934),
                );
                path.cubic_bezier_to(
                    view.point(21.5611, 51.042),
                    view.point(23.8456, 51.9406),
                    view.point(22.6362, 51.6209),
                );
                path.line_to(view.point(3.81481, 41.5912));
                path.line_to(view.point(3.8049, 41.5866));
                path.cubic_bezier_to(
                    view.point(1.02306, 38.9965),
                    view.point(2.65012, 40.9652),
                    view.point(1.69078, 40.0722),
                );
                path.cubic_bezier_to(
                    view.point(0.0, 35.4554),
                    view.point(0.355329, 37.9206),
                    view.point(0.00203541, 36.6983),
                );
                path.line_to(view.point(0.0, 16.54));
                path.line_to(view.point(0.01734, 16.0737));
                path.cubic_bezier_to(
                    view.point(1.02059, 12.9988),
                    view.point(0.0949446, 14.9929),
                    view.point(0.43681, 13.9403),
                );
                path.cubic_bezier_to(
                    view.point(3.8049, 10.4065),
                    view.point(1.68832, 11.9224),
                    view.point(2.64948, 11.0282),
                );
                path.line_to(view.point(3.81481, 10.3995));
                path.line_to(view.point(21.571, 0.944153));
                path.cubic_bezier_to(
                    view.point(25.366, 0.0),
                    view.point(22.7256, 0.324955),
                    view.point(24.0338, 0.0),
                );
                path.close();
                path.move_to(view.point(5.0732, 35.4485));
                path.line_to(view.point(5.0955, 35.7578));
                path.cubic_bezier_to(
                    view.point(5.41505, 36.6281),
                    view.point(5.13923, 36.0637),
                    view.point(5.24825, 36.3592),
                );
                path.cubic_bezier_to(
                    view.point(6.33159, 37.4868),
                    view.point(5.63566, 36.9835),
                    view.point(5.95114, 37.2799),
                );
                path.line_to(view.point(22.8294, 46.2704));
                path.line_to(view.point(22.8294, 27.3643));
                path.line_to(view.point(5.0732, 17.8674));
                path.line_to(view.point(5.0732, 35.4485));
                path.close();
                path.move_to(view.point(7.65439, 13.7976));
                path.line_to(view.point(25.366, 23.2714));
                path.line_to(view.point(31.6704, 19.8965));
                path.line_to(view.point(13.9488, 10.4457));
                path.line_to(view.point(7.65439, 13.7976));
                path.close();
                path.move_to(view.point(25.366, 4.72769));
                path.cubic_bezier_to(
                    view.point(24.0977, 5.04395),
                    view.point(24.9209, 4.72769),
                    view.point(24.4832, 4.83659),
                );
                path.line_to(view.point(24.0878, 5.04857));
                path.line_to(view.point(19.0592, 7.72405));
                path.line_to(view.point(36.7708, 17.1702));
                path.line_to(view.point(43.0751, 13.7976));
                path.line_to(view.point(26.6442, 5.04857));
                path.line_to(view.point(26.6343, 5.04395));
                path.cubic_bezier_to(
                    view.point(25.366, 4.72769),
                    view.point(26.2488, 4.83659),
                    view.point(25.8112, 4.72769),
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
