use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct ChevronDownIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl ChevronDownIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for ChevronDownIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(24.2734, 40.7266));
                path.cubic_bezier_to(
                    view.point(27.7164, 40.7266),
                    view.point(25.2281, 41.6812),
                    view.point(26.7719, 41.6812),
                );
                path.line_to(view.point(47.2266, 21.2266));
                path.cubic_bezier_to(
                    view.point(47.2266, 17.7836),
                    view.point(48.1812, 20.2719),
                    view.point(48.1812, 18.7281),
                );
                path.cubic_bezier_to(
                    view.point(43.7836, 17.7836),
                    view.point(46.2719, 16.8391),
                    view.point(44.7281, 16.8289),
                );
                path.line_to(view.point(26.0102, 35.557));
                path.line_to(view.point(8.22656, 17.7734));
                path.cubic_bezier_to(
                    view.point(4.78359, 17.7734),
                    view.point(7.27187, 16.8187),
                    view.point(5.72812, 16.8187),
                );
                path.cubic_bezier_to(
                    view.point(4.78359, 21.2164),
                    view.point(3.83906, 18.7281),
                    view.point(3.82891, 20.2719),
                );
                path.line_to(view.point(24.2734, 40.7266));
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
