use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct WeaponIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl WeaponIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for WeaponIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(27.7843, 51.7302));
                path.cubic_bezier_to(
                    view.point(26.9113, 50.0302),
                    view.point(27.4167, 51.4316),
                    view.point(27.0951, 50.7883),
                );
                path.cubic_bezier_to(
                    view.point(26.5437, 48.9734),
                    view.point(26.8424, 49.6856),
                    view.point(26.6815, 49.2261),
                );
                path.cubic_bezier_to(
                    view.point(26.1301, 42.1044),
                    view.point(26.0842, 48.0775),
                    view.point(26.0842, 47.8018),
                );
                path.line_to(view.point(26.1761, 36.8435));
                path.line_to(view.point(25.9463, 37.0732));
                path.cubic_bezier_to(
                    view.point(24.5449, 36.6597),
                    view.point(25.5098, 37.5097),
                    view.point(24.7516, 37.303),
                );
                path.cubic_bezier_to(
                    view.point(23.9935, 31.9961),
                    view.point(24.2921, 35.8556),
                    view.point(24.1083, 34.4313),
                );
                path.cubic_bezier_to(
                    view.point(23.6488, 27.4704),
                    view.point(23.9475, 30.9393),
                    view.point(23.7867, 28.8947),
                );
                path.cubic_bezier_to(
                    view.point(23.3272, 23.496),
                    view.point(23.488, 26.0231),
                    view.point(23.3502, 24.2541),
                );
                path.cubic_bezier_to(
                    view.point(23.1204, 20.9459),
                    view.point(23.2812, 22.7608),
                    view.point(23.2123, 21.6122),
                );
                path.cubic_bezier_to(
                    view.point(22.9366, 18.8094),
                    view.point(23.0515, 20.2797),
                    view.point(22.9826, 19.3148),
                );
                path.line_to(view.point(22.8907, 17.8905));
                path.line_to(view.point(23.1664, 17.6148));
                path.cubic_bezier_to(
                    view.point(25.0963, 18.0513),
                    view.point(23.6948, 17.0864),
                    view.point(24.6827, 17.3162),
                );
                path.cubic_bezier_to(
                    view.point(25.9004, 23.6798),
                    view.point(25.4639, 18.6716),
                    view.point(25.7625, 20.7622),
                );
                path.cubic_bezier_to(
                    view.point(26.1301, 25.5636),
                    view.point(26.1301, 28.9866),
                    view.point(26.1301, 28.9866),
                );
                path.cubic_bezier_to(
                    view.point(26.0612, 20.3486),
                    view.point(26.1301, 23.8406),
                    view.point(26.1072, 21.4973),
                );
                path.cubic_bezier_to(
                    view.point(25.7166, 17.5689),
                    view.point(26.0153, 18.304),
                    view.point(26.0153, 18.281),
                );
                path.cubic_bezier_to(
                    view.point(23.6259, 15.754),
                    view.point(25.2801, 16.5121),
                    view.point(24.5449, 15.8688),
                );
                path.cubic_bezier_to(
                    view.point(22.7069, 15.3864),
                    view.point(23.3042, 15.708),
                    view.point(23.0285, 15.5932),
                );
                path.cubic_bezier_to(
                    view.point(20.5472, 14.3296),
                    view.point(22.0406, 14.9499),
                    view.point(20.9608, 14.4215),
                );
                path.cubic_bezier_to(
                    view.point(19.1458, 13.0431),
                    view.point(19.6512, 14.1688),
                    view.point(19.1458, 13.6864),
                );
                path.cubic_bezier_to(
                    view.point(21.9027, 12.0782),
                    view.point(19.1458, 12.1701),
                    view.point(19.7201, 11.9863),
                );
                path.cubic_bezier_to(
                    view.point(24.43, 12.8593),
                    view.point(23.2812, 12.1472),
                    view.point(23.8786, 12.3309),
                );
                path.line_to(view.point(24.7057, 13.158));
                path.line_to(view.point(24.7057, 12.7674));
                path.cubic_bezier_to(
                    view.point(25.8085, 11.2742),
                    view.point(24.7057, 12.3769),
                    view.point(24.7516, 12.3309),
                );
                path.line_to(view.point(26.9113, 10.1944));
                path.line_to(view.point(26.9802, 6.47274));
                path.cubic_bezier_to(
                    view.point(27.0721, 2.5443),
                    view.point(27.0032, 4.40514),
                    view.point(27.0491, 2.65916),
                );
                path.cubic_bezier_to(
                    view.point(26.7734, 2.1997),
                    view.point(27.141, 2.40646),
                    view.point(27.0721, 2.33754),
                );
                path.cubic_bezier_to(
                    view.point(26.6815, 0.08615),
                    view.point(25.7396, 1.71726),
                    view.point(25.6936, 0.338857),
                );
                path.cubic_bezier_to(
                    view.point(30.0818, 0.08615),
                    view.point(27.0951, -0.0287167),
                    view.point(29.6913, -0.0287167),
                );
                path.cubic_bezier_to(
                    view.point(30.7481, 1.09698),
                    view.point(30.5184, 0.22399),
                    view.point(30.7481, 0.545617),
                );
                path.cubic_bezier_to(
                    view.point(29.5764, 2.38348),
                    view.point(30.7481, 1.64834),
                    view.point(30.3116, 2.13078),
                );
                path.line_to(view.point(29.1399, 2.5443));
                path.line_to(view.point(29.3926, 2.84295));
                path.line_to(view.point(29.6683, 3.1416));
                path.line_to(view.point(29.6683, 6.58761));
                path.line_to(view.point(29.6683, 10.0566));
                path.line_to(view.point(30.3575, 10.7917));
                path.cubic_bezier_to(
                    view.point(31.1846, 11.6188),
                    view.point(30.7481, 11.1823),
                    view.point(31.1157, 11.5728),
                );
                path.cubic_bezier_to(
                    view.point(31.9198, 12.7215),
                    view.point(31.805, 12.1242),
                    view.point(31.9198, 12.285),
                );
                path.line_to(view.point(31.9198, 13.135));
                path.line_to(view.point(32.2415, 12.8364));
                path.cubic_bezier_to(
                    view.point(35.2282, 12.0782),
                    view.point(33.0226, 12.1701),
                    view.point(33.4592, 12.0553),
                );
                path.cubic_bezier_to(
                    view.point(37.5717, 13.0201),
                    view.point(37.227, 12.1012),
                    view.point(37.5717, 12.239),
                );
                path.cubic_bezier_to(
                    view.point(36.1702, 14.3296),
                    view.point(37.5717, 13.6634),
                    view.point(37.1122, 14.0999),
                );
                path.cubic_bezier_to(
                    view.point(34.1944, 15.2715),
                    view.point(35.6188, 14.4445),
                    view.point(34.516, 14.9729),
                );
                path.cubic_bezier_to(
                    view.point(33.0226, 15.731),
                    view.point(33.8957, 15.5472),
                    view.point(33.62, 15.6391),
                );
                path.cubic_bezier_to(
                    view.point(30.5184, 19.6824),
                    view.point(31.5063, 15.9378),
                    view.point(30.5873, 17.4081),
                );
                path.cubic_bezier_to(
                    view.point(30.4724, 28.5961),
                    view.point(30.4494, 21.4743),
                    view.point(30.4265, 29.2623),
                );
                path.cubic_bezier_to(
                    view.point(30.5413, 26.5974),
                    view.point(30.5184, 28.2055),
                    view.point(30.5413, 27.3325),
                );
                path.cubic_bezier_to(
                    view.point(31.4374, 18.0283),
                    view.point(30.5873, 23.7257),
                    view.point(31.1157, 18.6027),
                );
                path.cubic_bezier_to(
                    view.point(32.8388, 17.1553),
                    view.point(31.6901, 17.6148),
                    view.point(32.4023, 17.1553),
                );
                path.cubic_bezier_to(
                    view.point(33.5051, 17.477),
                    view.point(33.1145, 17.1553),
                    view.point(33.2524, 17.2243),
                );
                path.line_to(view.point(33.8268, 17.7986));
                path.line_to(view.point(33.7808, 19.177));
                path.cubic_bezier_to(
                    view.point(33.4821, 22.3014),
                    view.point(33.7349, 19.9121),
                    view.point(33.62, 21.3365),
                );
                path.cubic_bezier_to(
                    view.point(33.1605, 25.7933),
                    view.point(33.3673, 23.2433),
                    view.point(33.2064, 24.8284),
                );
                path.cubic_bezier_to(
                    view.point(32.8618, 30.0664),
                    view.point(33.1145, 26.7582),
                    view.point(32.9767, 28.688),
                );
                path.cubic_bezier_to(
                    view.point(32.5861, 33.1218),
                    view.point(32.724, 31.4448),
                    view.point(32.6091, 32.8232),
                );
                path.cubic_bezier_to(
                    view.point(32.4483, 35.557),
                    view.point(32.5631, 33.4205),
                    view.point(32.4942, 34.5232),
                );
                path.cubic_bezier_to(
                    view.point(31.736, 39.6462),
                    view.point(32.2874, 38.7273),
                    view.point(32.1726, 39.4165),
                );
                path.cubic_bezier_to(
                    view.point(30.4954, 39.026),
                    view.point(31.2536, 39.899),
                    view.point(30.6103, 39.5773),
                );
                path.cubic_bezier_to(
                    view.point(30.4494, 43.299),
                    view.point(30.4494, 38.8652),
                    view.point(30.4494, 40.7719),
                );
                path.cubic_bezier_to(
                    view.point(29.8521, 49.9613),
                    view.point(30.4954, 48.3991),
                    view.point(30.4494, 48.7667),
                );
                path.cubic_bezier_to(
                    view.point(29.5304, 50.8343),
                    view.point(29.7142, 50.2599),
                    view.point(29.5534, 50.6505),
                );
                path.cubic_bezier_to(
                    view.point(27.7843, 51.7302),
                    view.point(29.3466, 51.891),
                    view.point(28.4736, 52.3275),
                );
                path.close();
                path.move_to(view.point(28.8871, 13.158));
                path.cubic_bezier_to(
                    view.point(29.3007, 12.6296),
                    view.point(29.1169, 12.9053),
                    view.point(29.3007, 12.6755),
                );
                path.cubic_bezier_to(
                    view.point(28.8182, 12.0553),
                    view.point(29.3007, 12.6066),
                    view.point(29.0939, 12.3309),
                );
                path.line_to(view.point(28.3587, 11.5498));
                path.line_to(view.point(27.8762, 12.0323));
                path.cubic_bezier_to(
                    view.point(27.3938, 12.6066),
                    view.point(27.6235, 12.308),
                    view.point(27.3938, 12.5607),
                );
                path.cubic_bezier_to(
                    view.point(28.4047, 13.6404),
                    view.point(27.3708, 12.6755),
                    view.point(28.3357, 13.6634),
                );
                path.cubic_bezier_to(
                    view.point(28.8871, 13.158),
                    view.point(28.4276, 13.6174),
                    view.point(28.6344, 13.4107),
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
