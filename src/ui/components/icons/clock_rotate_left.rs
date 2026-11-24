use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct ClockRotateLeftIcon {
    width: Pixels,
    height: Pixels,
    first_color: Rgba,
    second_color: Rgba,
}

impl ClockRotateLeftIcon {
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

impl RenderOnce for ClockRotateLeftIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 4359.0, 352.0, 2e+01, 2e+01);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(4369.0, 357.0));
                path.cubic_bezier_to(
                    view.point(4368.0625, 357.9375),
                    view.point(4368.4805, 357.0),
                    view.point(4368.0625, 357.41797),
                );
                path.line_to(view.point(4368.0625, 362.0));
                path.cubic_bezier_to(
                    view.point(4368.336, 362.66406),
                    view.point(4368.0625, 362.25),
                    view.point(4368.16, 362.48828),
                );
                path.line_to(view.point(4371.1484, 365.47656));
                path.cubic_bezier_to(
                    view.point(4372.4727, 365.47656),
                    view.point(4371.5156, 365.84375),
                    view.point(4372.1094, 365.84375),
                );
                path.cubic_bezier_to(
                    view.point(4372.4727, 364.15234),
                    view.point(4372.836, 365.10938),
                    view.point(4372.84, 364.51562),
                );
                path.line_to(view.point(4369.9336, 361.61328));
                path.line_to(view.point(4369.9336, 357.9375));
                path.cubic_bezier_to(
                    view.point(4368.996, 357.0),
                    view.point(4369.9336, 357.41797),
                    view.point(4369.5156, 357.0),
                );
                path.line_to(view.point(4369.0, 357.0));
                path.close();

                if let Ok(path) = path.build() {
                    window.paint_path(path, self.first_color);
                }

                let mut path = PathBuilder::fill();
                path.move_to(view.point(4360.875, 356.16797));
                path.line_to(view.point(4360.875, 354.1875));
                path.cubic_bezier_to(
                    view.point(4359.9375, 353.25),
                    view.point(4360.875, 353.66797),
                    view.point(4360.457, 353.25),
                );
                path.cubic_bezier_to(
                    view.point(4359.0, 354.1875),
                    view.point(4359.418, 353.25),
                    view.point(4359.0, 353.66797),
                );
                path.line_to(view.point(4359.0, 358.5625));
                path.cubic_bezier_to(
                    view.point(4359.9375, 359.5),
                    view.point(4359.0, 359.08203),
                    view.point(4359.418, 359.5),
                );
                path.line_to(view.point(4364.3125, 359.5));
                path.cubic_bezier_to(
                    view.point(4365.25, 358.5625),
                    view.point(4364.832, 359.5),
                    view.point(4365.25, 359.08203),
                );
                path.cubic_bezier_to(
                    view.point(4364.3125, 357.625),
                    view.point(4365.25, 358.04297),
                    view.point(4364.832, 357.625),
                );
                path.line_to(view.point(4362.1523, 357.625));
                path.cubic_bezier_to(
                    view.point(4369.0, 353.875),
                    view.point(4363.5977, 355.3672),
                    view.point(4366.125, 353.875),
                );
                path.cubic_bezier_to(
                    view.point(4377.125, 362.0),
                    view.point(4373.4883, 353.875),
                    view.point(4377.125, 357.51172),
                );
                path.cubic_bezier_to(
                    view.point(4369.0, 370.125),
                    view.point(4377.125, 366.48828),
                    view.point(4373.4883, 370.125),
                );
                path.cubic_bezier_to(
                    view.point(4364.5195, 368.77734),
                    view.point(4367.34, 370.125),
                    view.point(4365.801, 369.6289),
                );
                path.cubic_bezier_to(
                    view.point(4363.2188, 369.03906),
                    view.point(4364.09, 368.4922),
                    view.point(4363.508, 368.60938),
                );
                path.cubic_bezier_to(
                    view.point(4363.4805, 370.33984),
                    view.point(4362.9297, 369.46875),
                    view.point(4363.051, 370.05078),
                );
                path.cubic_bezier_to(
                    view.point(4369.0, 372.0),
                    view.point(4365.0625, 371.39062),
                    view.point(4366.961, 372.0),
                );
                path.cubic_bezier_to(
                    view.point(4379.0, 362.0),
                    view.point(4374.5234, 372.0),
                    view.point(4379.0, 367.52344),
                );
                path.cubic_bezier_to(
                    view.point(4369.0, 352.0),
                    view.point(4379.0, 356.47656),
                    view.point(4374.5234, 352.0),
                );
                path.cubic_bezier_to(
                    view.point(4360.875, 356.16797),
                    view.point(4365.6523, 352.0),
                    view.point(4362.6875, 353.64453),
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
