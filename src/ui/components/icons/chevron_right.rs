use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct ChevronRightIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl ChevronRightIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for ChevronRightIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(40.7266, 24.2734));
                path.cubic_bezier_to(
                    view.point(40.7266, 27.7164),
                    view.point(41.6812, 25.2281),
                    view.point(41.6812, 26.7719),
                );
                path.line_to(view.point(21.2266, 47.2266));
                path.cubic_bezier_to(
                    view.point(17.7836, 47.2266),
                    view.point(20.2719, 48.1812),
                    view.point(18.7281, 48.1812),
                );
                path.cubic_bezier_to(
                    view.point(17.7836, 43.7836),
                    view.point(16.8391, 46.2719),
                    view.point(16.8289, 44.7281),
                );
                path.line_to(view.point(35.557, 26.0102));
                path.line_to(view.point(17.7734, 8.22656));
                path.cubic_bezier_to(
                    view.point(17.7734, 4.78359),
                    view.point(16.8187, 7.27187),
                    view.point(16.8187, 5.72812),
                );
                path.cubic_bezier_to(
                    view.point(21.2164, 4.78359),
                    view.point(18.7281, 3.83906),
                    view.point(20.2719, 3.82891),
                );
                path.line_to(view.point(40.7266, 24.2734));
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
