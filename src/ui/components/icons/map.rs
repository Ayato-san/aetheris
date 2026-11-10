use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct MapIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl MapIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for MapIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 53.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(51.23, 8.86722));
                path.cubic_bezier_to(
                    view.point(50.4143, 8.48808),
                    view.point(50.9904, 8.68055),
                    view.point(50.7115, 8.55089),
                );
                path.cubic_bezier_to(
                    view.point(49.515, 8.50472),
                    view.point(50.1172, 8.42527),
                    view.point(49.8096, 8.43096),
                );
                path.line_to(view.point(34.2325, 12.3247));
                path.line_to(view.point(18.895, 4.65472));
                path.cubic_bezier_to(
                    view.point(17.515, 4.50472),
                    view.point(18.4677, 4.44157),
                    view.point(17.9782, 4.38836),
                );
                path.line_to(view.point(1.515, 8.50472));
                path.cubic_bezier_to(
                    view.point(0.423705, 9.21405),
                    view.point(1.08232, 8.61287),
                    view.point(0.698206, 8.86255),
                );
                path.cubic_bezier_to(
                    view.point(0.0, 10.4447),
                    view.point(0.149204, 9.56556),
                    view.point(6.75268e-05, 9.99873),
                );
                path.line_to(view.point(0.0, 46.4447));
                path.cubic_bezier_to(
                    view.point(0.202642, 47.3216),
                    view.point(4.60118e-05, 46.7486),
                    view.point(0.0693478, 47.0485),
                );
                path.cubic_bezier_to(
                    view.point(0.769266, 48.0209),
                    view.point(0.335936, 47.5947),
                    view.point(0.529717, 47.8339),
                );
                path.cubic_bezier_to(
                    view.point(1.58512, 48.4009),
                    view.point(1.00882, 48.2079),
                    view.point(1.28783, 48.3379),
                );
                path.cubic_bezier_to(
                    view.point(2.485, 48.3847),
                    view.point(1.88242, 48.464),
                    view.point(2.19017, 48.4584),
                );
                path.line_to(view.point(17.7675, 44.5647));
                path.line_to(view.point(33.105, 52.2347));
                path.cubic_bezier_to(
                    view.point(34.0, 52.4447),
                    view.point(33.3834, 52.372),
                    view.point(33.6896, 52.4438),
                );
                path.cubic_bezier_to(
                    view.point(34.485, 52.3847),
                    view.point(34.1635, 52.4446),
                    view.point(34.3264, 52.4245),
                );
                path.line_to(view.point(50.485, 48.3847));
                path.cubic_bezier_to(
                    view.point(51.5763, 47.6754),
                    view.point(50.9177, 48.2766),
                    view.point(51.3018, 48.0269),
                );
                path.cubic_bezier_to(
                    view.point(52.0, 46.4447),
                    view.point(51.8508, 47.3239),
                    view.point(51.9999, 46.8907),
                );
                path.line_to(view.point(52.0, 10.4447));
                path.cubic_bezier_to(
                    view.point(51.7973, 9.56698),
                    view.point(52.0001, 10.1405),
                    view.point(51.9307, 9.84034),
                );
                path.cubic_bezier_to(
                    view.point(51.23, 8.86722),
                    view.point(51.6639, 9.29362),
                    view.point(51.4699, 9.0543),
                );
                path.close();
                path.move_to(view.point(18.0, 40.4447));
                path.cubic_bezier_to(
                    view.point(17.515, 40.5047),
                    view.point(17.8365, 40.4448),
                    view.point(17.6736, 40.465),
                );
                path.line_to(view.point(4.0, 43.8822));
                path.line_to(view.point(4.0, 12.0072));
                path.line_to(view.point(17.7675, 8.56472));
                path.line_to(view.point(18.0, 8.67972));
                path.line_to(view.point(18.0, 40.4447));
                path.close();
                path.move_to(view.point(48.0, 44.8822));
                path.line_to(view.point(34.2325, 48.3247));
                path.line_to(view.point(34.0, 48.2097));
                path.line_to(view.point(34.0, 16.4447));
                path.cubic_bezier_to(
                    view.point(34.485, 16.3872),
                    view.point(34.1634, 16.4454),
                    view.point(34.3263, 16.4261),
                );
                path.line_to(view.point(48.0, 13.0072));
                path.line_to(view.point(48.0, 44.8822));
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
