use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct AbyssesSupplyRequestsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl AbyssesSupplyRequestsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for AbyssesSupplyRequestsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(48.0828, 11.3499));
                path.line_to(view.point(28.3586, 0.55302));
                path.cubic_bezier_to(
                    view.point(26.2069, 0.0),
                    view.point(27.6995, 0.190235),
                    view.point(26.9593, 0.0),
                );
                path.cubic_bezier_to(
                    view.point(24.0552, 0.55302),
                    view.point(25.4545, 0.0),
                    view.point(24.7143, 0.190235),
                );
                path.line_to(view.point(4.33103, 11.3499));
                path.cubic_bezier_to(
                    view.point(2.62628, 12.9971),
                    view.point(3.62526, 11.736),
                    view.point(3.03642, 12.305),
                );
                path.cubic_bezier_to(
                    view.point(2.0, 15.2835),
                    view.point(2.21614, 13.6893),
                    view.point(1.99981, 14.479),
                );
                path.line_to(view.point(2.0, 36.7158));
                path.cubic_bezier_to(
                    view.point(2.62628, 39.0022),
                    view.point(1.99981, 37.5204),
                    view.point(2.21614, 38.3101),
                );
                path.cubic_bezier_to(
                    view.point(4.33103, 40.6495),
                    view.point(3.03642, 39.6943),
                    view.point(3.62526, 40.2633),
                );
                path.line_to(view.point(24.0552, 51.4463));
                path.cubic_bezier_to(
                    view.point(26.2069, 52.0),
                    view.point(24.7142, 51.8095),
                    view.point(25.4544, 52.0),
                );
                path.cubic_bezier_to(
                    view.point(28.3586, 51.4463),
                    view.point(26.9594, 52.0),
                    view.point(27.6996, 51.8095),
                );
                path.line_to(view.point(48.0828, 40.6495));
                path.cubic_bezier_to(
                    view.point(49.7875, 39.0022),
                    view.point(48.7885, 40.2633),
                    view.point(49.3774, 39.6943),
                );
                path.cubic_bezier_to(
                    view.point(50.4138, 36.7158),
                    view.point(50.1977, 38.3101),
                    view.point(50.414, 37.5204),
                );
                path.line_to(view.point(50.4138, 15.2835));
                path.cubic_bezier_to(
                    view.point(49.7875, 12.9971),
                    view.point(50.414, 14.479),
                    view.point(50.1977, 13.6893),
                );
                path.cubic_bezier_to(
                    view.point(48.0828, 11.3499),
                    view.point(49.3774, 12.305),
                    view.point(48.7885, 11.736),
                );
                path.close();
                path.move_to(view.point(26.2069, 5.50652));
                path.line_to(view.point(42.3448, 14.3444));
                path.line_to(view.point(37.5415, 16.9735));
                path.line_to(view.point(21.4036, 8.13793));
                path.line_to(view.point(26.2069, 5.50652));
                path.close();
                path.move_to(view.point(26.2069, 23.1755));
                path.line_to(view.point(10.069, 14.3444));
                path.line_to(view.point(15.7979, 11.2064));
                path.line_to(view.point(31.9359, 20.0398));
                path.line_to(view.point(26.2069, 23.1755));
                path.close();
                path.move_to(view.point(7.37931, 19.0043));
                path.line_to(view.point(23.5172, 27.8354));
                path.line_to(view.point(23.5172, 45.0202));
                path.line_to(view.point(7.37931, 36.1846));
                path.line_to(view.point(7.37931, 19.0043));
                path.close();
                path.move_to(view.point(28.8965, 45.0202));
                path.line_to(view.point(28.8965, 27.8354));
                path.line_to(view.point(34.2759, 24.8924));
                path.line_to(view.point(34.2759, 31.3791));
                path.cubic_bezier_to(
                    view.point(35.0636, 33.2809),
                    view.point(34.2759, 32.0924),
                    view.point(34.5592, 32.7765),
                );
                path.cubic_bezier_to(
                    view.point(36.9655, 34.0687),
                    view.point(35.5681, 33.7854),
                    view.point(36.2522, 34.0687),
                );
                path.cubic_bezier_to(
                    view.point(38.8674, 33.2809),
                    view.point(37.6789, 34.0687),
                    view.point(38.363, 33.7854),
                );
                path.cubic_bezier_to(
                    view.point(39.6552, 31.3791),
                    view.point(39.3718, 32.7765),
                    view.point(39.6552, 32.0924),
                );
                path.line_to(view.point(39.6552, 21.9472));
                path.line_to(view.point(45.0345, 19.0043));
                path.line_to(view.point(45.0345, 36.1846));
                path.line_to(view.point(28.8965, 45.0202));
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
