use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct StatusEffectIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl StatusEffectIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for StatusEffectIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(40.518, 26.2807));
                path.line_to(view.point(29.712, 22.2967));
                path.line_to(view.point(25.7193, 11.482));
                path.cubic_bezier_to(
                    view.point(24.1381, 9.43469),
                    view.point(25.414, 10.6539),
                    view.point(24.8621, 9.93938),
                );
                path.cubic_bezier_to(
                    view.point(21.6703, 8.65942),
                    view.point(23.4142, 8.93),
                    view.point(22.5528, 8.65942),
                );
                path.cubic_bezier_to(
                    view.point(19.2024, 9.43469),
                    view.point(20.7877, 8.65942),
                    view.point(19.9264, 8.93),
                );
                path.cubic_bezier_to(
                    view.point(17.6213, 11.482),
                    view.point(18.4784, 9.93938),
                    view.point(17.9266, 10.6539),
                );
                path.line_to(view.point(13.6372, 22.2967));
                path.line_to(view.point(2.82254, 26.2807));
                path.cubic_bezier_to(
                    view.point(0.775275, 27.8618),
                    view.point(1.9945, 26.586),
                    view.point(1.27997, 27.1379),
                );
                path.cubic_bezier_to(
                    view.point(0.0, 30.3297),
                    view.point(0.270584, 28.5858),
                    view.point(0.0, 29.4472),
                );
                path.cubic_bezier_to(
                    view.point(0.775275, 32.7976),
                    view.point(0.0, 31.2122),
                    view.point(0.270584, 32.0736),
                );
                path.cubic_bezier_to(
                    view.point(2.82254, 34.3787),
                    view.point(1.27997, 33.5216),
                    view.point(1.9945, 34.0734),
                );
                path.line_to(view.point(13.6286, 38.3627));
                path.line_to(view.point(17.6213, 49.1775));
                path.cubic_bezier_to(
                    view.point(19.2024, 51.2247),
                    view.point(17.9266, 50.0055),
                    view.point(18.4784, 50.72),
                );
                path.cubic_bezier_to(
                    view.point(21.6703, 52.0),
                    view.point(19.9264, 51.7294),
                    view.point(20.7877, 52.0),
                );
                path.cubic_bezier_to(
                    view.point(24.1381, 51.2247),
                    view.point(22.5528, 52.0),
                    view.point(23.4142, 51.7294),
                );
                path.cubic_bezier_to(
                    view.point(25.7193, 49.1775),
                    view.point(24.8621, 50.72),
                    view.point(25.414, 50.0055),
                );
                path.line_to(view.point(29.7033, 38.3714));
                path.line_to(view.point(40.518, 34.3787));
                path.cubic_bezier_to(
                    view.point(42.5653, 32.7976),
                    view.point(41.3461, 34.0734),
                    view.point(42.0606, 33.5216),
                );
                path.cubic_bezier_to(
                    view.point(43.3406, 30.3297),
                    view.point(43.07, 32.0736),
                    view.point(43.3406, 31.2122),
                );
                path.cubic_bezier_to(
                    view.point(42.5653, 27.8618),
                    view.point(43.3406, 29.4472),
                    view.point(43.07, 28.5858),
                );
                path.cubic_bezier_to(
                    view.point(40.518, 26.2807),
                    view.point(42.0606, 27.1379),
                    view.point(41.3461, 26.586),
                );
                path.close();
                path.move_to(view.point(26.7873, 33.8978));
                path.cubic_bezier_to(
                    view.point(25.8481, 34.4988),
                    view.point(26.4345, 34.0278),
                    view.point(26.114, 34.2329),
                );
                path.cubic_bezier_to(
                    view.point(25.247, 35.4381),
                    view.point(25.5821, 34.7648),
                    view.point(25.3771, 35.0852),
                );
                path.line_to(view.point(21.6703, 45.1219));
                path.line_to(view.point(18.1022, 35.4381));
                path.cubic_bezier_to(
                    view.point(17.5012, 34.4988),
                    view.point(17.9722, 35.0852),
                    view.point(17.7671, 34.7648),
                );
                path.cubic_bezier_to(
                    view.point(16.5619, 33.8978),
                    view.point(17.2352, 34.2329),
                    view.point(16.9148, 34.0278),
                );
                path.line_to(view.point(6.87806, 30.3297));
                path.line_to(view.point(16.5619, 26.7616));
                path.cubic_bezier_to(
                    view.point(17.5012, 26.1606),
                    view.point(16.9148, 26.6316),
                    view.point(17.2352, 26.4265),
                );
                path.cubic_bezier_to(
                    view.point(18.1022, 25.2213),
                    view.point(17.7671, 25.8947),
                    view.point(17.9722, 25.5742),
                );
                path.line_to(view.point(21.6703, 15.5375));
                path.line_to(view.point(25.2384, 25.2213));
                path.cubic_bezier_to(
                    view.point(25.8394, 26.1606),
                    view.point(25.3684, 25.5742),
                    view.point(25.5735, 25.8947),
                );
                path.cubic_bezier_to(
                    view.point(26.7787, 26.7616),
                    view.point(26.1053, 26.4265),
                    view.point(26.4258, 26.6316),
                );
                path.line_to(view.point(36.4625, 30.3297));
                path.line_to(view.point(26.7873, 33.8978));
                path.close();
                path.move_to(view.point(27.7362, 7.79907));
                path.cubic_bezier_to(
                    view.point(28.4977, 5.96081),
                    view.point(27.7362, 7.10959),
                    view.point(28.0101, 6.44835),
                );
                path.cubic_bezier_to(
                    view.point(30.3359, 5.19938),
                    view.point(28.9852, 5.47327),
                    view.point(29.6464, 5.19938),
                );
                path.line_to(view.point(32.9356, 5.19938));
                path.line_to(view.point(32.9356, 2.59969));
                path.cubic_bezier_to(
                    view.point(33.697, 0.761431),
                    view.point(32.9356, 1.91021),
                    view.point(33.2095, 1.24897),
                );
                path.cubic_bezier_to(
                    view.point(35.5353, 0.0),
                    view.point(34.1846, 0.273895),
                    view.point(34.8458, 0.0),
                );
                path.cubic_bezier_to(
                    view.point(37.3736, 0.761431),
                    view.point(36.2248, 0.0),
                    view.point(36.886, 0.273895),
                );
                path.cubic_bezier_to(
                    view.point(38.135, 2.59969),
                    view.point(37.8611, 1.24897),
                    view.point(38.135, 1.91021),
                );
                path.line_to(view.point(38.135, 5.19938));
                path.line_to(view.point(40.7347, 5.19938));
                path.cubic_bezier_to(
                    view.point(42.5729, 5.96081),
                    view.point(41.4242, 5.19938),
                    view.point(42.0854, 5.47327),
                );
                path.cubic_bezier_to(
                    view.point(43.3344, 7.79907),
                    view.point(43.0605, 6.44835),
                    view.point(43.3344, 7.10959),
                );
                path.cubic_bezier_to(
                    view.point(42.5729, 9.63733),
                    view.point(43.3344, 8.48855),
                    view.point(43.0605, 9.14979),
                );
                path.cubic_bezier_to(
                    view.point(40.7347, 10.3988),
                    view.point(42.0854, 10.1249),
                    view.point(41.4242, 10.3988),
                );
                path.line_to(view.point(38.135, 10.3988));
                path.line_to(view.point(38.135, 12.9984));
                path.cubic_bezier_to(
                    view.point(37.3736, 14.8367),
                    view.point(38.135, 13.6879),
                    view.point(37.8611, 14.3492),
                );
                path.cubic_bezier_to(
                    view.point(35.5353, 15.5981),
                    view.point(36.886, 15.3242),
                    view.point(36.2248, 15.5981),
                );
                path.cubic_bezier_to(
                    view.point(33.697, 14.8367),
                    view.point(34.8458, 15.5981),
                    view.point(34.1846, 15.3242),
                );
                path.cubic_bezier_to(
                    view.point(32.9356, 12.9984),
                    view.point(33.2095, 14.3492),
                    view.point(32.9356, 13.6879),
                );
                path.line_to(view.point(32.9356, 10.3988));
                path.line_to(view.point(30.3359, 10.3988));
                path.cubic_bezier_to(
                    view.point(28.4977, 9.63733),
                    view.point(29.6464, 10.3988),
                    view.point(28.9852, 10.1249),
                );
                path.cubic_bezier_to(
                    view.point(27.7362, 7.79907),
                    view.point(28.0101, 9.14979),
                    view.point(27.7362, 8.48855),
                );
                path.close();
                path.move_to(view.point(52.0, 18.1978));
                path.cubic_bezier_to(
                    view.point(51.2386, 20.0361),
                    view.point(52.0, 18.8873),
                    view.point(51.7261, 19.5485),
                );
                path.cubic_bezier_to(
                    view.point(49.4003, 20.7975),
                    view.point(50.751, 20.5236),
                    view.point(50.0898, 20.7975),
                );
                path.line_to(view.point(48.5337, 20.7975));
                path.line_to(view.point(48.5337, 21.6641));
                path.cubic_bezier_to(
                    view.point(47.7723, 23.5023),
                    view.point(48.5337, 22.3536),
                    view.point(48.2598, 23.0148),
                );
                path.cubic_bezier_to(
                    view.point(45.9341, 24.2638),
                    view.point(47.2848, 23.9899),
                    view.point(46.6235, 24.2638),
                );
                path.cubic_bezier_to(
                    view.point(44.0958, 23.5023),
                    view.point(45.2446, 24.2638),
                    view.point(44.5833, 23.9899),
                );
                path.cubic_bezier_to(
                    view.point(43.3344, 21.6641),
                    view.point(43.6083, 23.0148),
                    view.point(43.3344, 22.3536),
                );
                path.line_to(view.point(43.3344, 20.7975));
                path.line_to(view.point(42.4678, 20.7975));
                path.cubic_bezier_to(
                    view.point(40.6296, 20.0361),
                    view.point(41.7783, 20.7975),
                    view.point(41.1171, 20.5236),
                );
                path.cubic_bezier_to(
                    view.point(39.8681, 18.1978),
                    view.point(40.142, 19.5485),
                    view.point(39.8681, 18.8873),
                );
                path.cubic_bezier_to(
                    view.point(40.6296, 16.3596),
                    view.point(39.8681, 17.5083),
                    view.point(40.142, 16.8471),
                );
                path.cubic_bezier_to(
                    view.point(42.4678, 15.5981),
                    view.point(41.1171, 15.872),
                    view.point(41.7783, 15.5981),
                );
                path.line_to(view.point(43.3344, 15.5981));
                path.line_to(view.point(43.3344, 14.7316));
                path.cubic_bezier_to(
                    view.point(44.0958, 12.8933),
                    view.point(43.3344, 14.0421),
                    view.point(43.6083, 13.3809),
                );
                path.cubic_bezier_to(
                    view.point(45.9341, 12.1319),
                    view.point(44.5833, 12.4058),
                    view.point(45.2446, 12.1319),
                );
                path.cubic_bezier_to(
                    view.point(47.7723, 12.8933),
                    view.point(46.6235, 12.1319),
                    view.point(47.2848, 12.4058),
                );
                path.cubic_bezier_to(
                    view.point(48.5337, 14.7316),
                    view.point(48.2598, 13.3809),
                    view.point(48.5337, 14.0421),
                );
                path.line_to(view.point(48.5337, 15.5981));
                path.line_to(view.point(49.4003, 15.5981));
                path.cubic_bezier_to(
                    view.point(51.2386, 16.3596),
                    view.point(50.0898, 15.5981),
                    view.point(50.751, 15.872),
                );
                path.cubic_bezier_to(
                    view.point(52.0, 18.1978),
                    view.point(51.7261, 16.8471),
                    view.point(52.0, 17.5083),
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
