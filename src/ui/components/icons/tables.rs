use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct TablesIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl TablesIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for TablesIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(49.1111, 4.27983));
                path.line_to(view.point(2.88889, 4.27983));
                path.cubic_bezier_to(
                    view.point(0.846136, 5.12597),
                    view.point(2.12271, 4.27983),
                    view.point(1.38791, 4.58419),
                );
                path.cubic_bezier_to(
                    view.point(0.0, 7.16872),
                    view.point(0.304364, 5.66774),
                    view.point(0.0, 6.40254),
                );
                path.line_to(view.point(0.0, 39.9095));
                path.cubic_bezier_to(
                    view.point(1.41023, 43.314),
                    view.point(0.0, 41.1864),
                    view.point(0.507274, 42.4111),
                );
                path.cubic_bezier_to(
                    view.point(4.81482, 44.7243),
                    view.point(2.31318, 44.217),
                    view.point(3.53785, 44.7243),
                );
                path.line_to(view.point(47.1852, 44.7243));
                path.cubic_bezier_to(
                    view.point(50.5898, 43.314),
                    view.point(48.4622, 44.7243),
                    view.point(49.6868, 44.217),
                );
                path.cubic_bezier_to(
                    view.point(52.0, 39.9095),
                    view.point(51.4927, 42.4111),
                    view.point(52.0, 41.1864),
                );
                path.line_to(view.point(52.0, 7.16872));
                path.cubic_bezier_to(
                    view.point(51.1539, 5.12597),
                    view.point(52.0, 6.40254),
                    view.point(51.6956, 5.66774),
                );
                path.cubic_bezier_to(
                    view.point(49.1111, 4.27983),
                    view.point(50.6121, 4.58419),
                    view.point(49.8773, 4.27983),
                );
                path.close();
                path.move_to(view.point(5.77778, 21.6132));
                path.line_to(view.point(13.4815, 21.6132));
                path.line_to(view.point(13.4815, 27.3909));
                path.line_to(view.point(5.77778, 27.3909));
                path.line_to(view.point(5.77778, 21.6132));
                path.close();
                path.move_to(view.point(19.2593, 21.6132));
                path.line_to(view.point(46.2222, 21.6132));
                path.line_to(view.point(46.2222, 27.3909));
                path.line_to(view.point(19.2593, 27.3909));
                path.line_to(view.point(19.2593, 21.6132));
                path.close();
                path.move_to(view.point(46.2222, 10.0576));
                path.line_to(view.point(46.2222, 15.8354));
                path.line_to(view.point(5.77778, 15.8354));
                path.line_to(view.point(5.77778, 10.0576));
                path.line_to(view.point(46.2222, 10.0576));
                path.close();
                path.move_to(view.point(5.77778, 33.1687));
                path.line_to(view.point(13.4815, 33.1687));
                path.line_to(view.point(13.4815, 38.9465));
                path.line_to(view.point(5.77778, 38.9465));
                path.line_to(view.point(5.77778, 33.1687));
                path.close();
                path.move_to(view.point(19.2593, 38.9465));
                path.line_to(view.point(19.2593, 33.1687));
                path.line_to(view.point(46.2222, 33.1687));
                path.line_to(view.point(46.2222, 38.9465));
                path.line_to(view.point(19.2593, 38.9465));
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
