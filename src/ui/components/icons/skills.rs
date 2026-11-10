use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct SkillsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl SkillsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for SkillsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(5.92078, 51.9227));
                path.cubic_bezier_to(
                    view.point(5.35351, 51.5681),
                    view.point(5.77896, 51.8991),
                    view.point(5.51896, 51.7336),
                );
                path.line_to(view.point(5.04624, 51.2372));
                path.line_to(view.point(5.0226, 26.1809));
                path.cubic_bezier_to(
                    view.point(5.70806, 0.155408),
                    view.point(4.97533, -1.23924),
                    view.point(4.92806, 0.509979),
                );
                path.cubic_bezier_to(
                    view.point(47.2135, 0.202685),
                    view.point(6.22806, -0.080972),
                    view.point(46.7408, -0.0336957),
                );
                path.cubic_bezier_to(
                    view.point(47.8517, 25.9918),
                    view.point(47.899, 0.557255),
                    view.point(47.8753, -0.861027),
                );
                path.cubic_bezier_to(
                    view.point(46.9535, 51.8518),
                    view.point(47.8517, 53.9083),
                    view.point(47.9462, 51.5208),
                );
                path.cubic_bezier_to(
                    view.point(5.92078, 51.9227),
                    view.point(46.5517, 51.9936),
                    view.point(6.65351, 52.0645),
                );
                path.close();
                path.move_to(view.point(42.2262, 49.3225));
                path.cubic_bezier_to(
                    view.point(42.4626, 48.0933),
                    view.point(42.8644, 48.9679),
                    view.point(43.0062, 48.2824),
                );
                path.cubic_bezier_to(
                    view.point(10.4353, 48.117),
                    view.point(41.919, 47.8806),
                    view.point(10.8371, 47.9279),
                );
                path.cubic_bezier_to(
                    view.point(11.3099, 49.488),
                    view.point(9.7026, 48.4952),
                    view.point(10.2226, 49.3461),
                );
                path.cubic_bezier_to(
                    view.point(26.8626, 49.5589),
                    view.point(11.6408, 49.5353),
                    view.point(18.6371, 49.5589),
                );
                path.cubic_bezier_to(
                    view.point(42.2262, 49.3225),
                    view.point(40.7371, 49.5352),
                    view.point(41.8481, 49.5353),
                );
                path.close();
                path.move_to(view.point(8.68624, 46.6041));
                path.cubic_bezier_to(
                    view.point(8.68624, 5.19031),
                    view.point(8.9226, 46.3205),
                    view.point(8.94624, 5.45033),
                );
                path.cubic_bezier_to(
                    view.point(7.59896, 5.71035),
                    view.point(8.44987, 4.95393),
                    view.point(7.8826, 5.21395),
                );
                path.cubic_bezier_to(
                    view.point(7.29169, 25.6845),
                    view.point(7.33896, 6.15947),
                    view.point(7.33896, 6.25402),
                );
                path.cubic_bezier_to(
                    view.point(7.40987, 45.7059),
                    view.point(7.26806, 39.3709),
                    view.point(7.31533, 45.3513),
                );
                path.cubic_bezier_to(
                    view.point(8.68624, 46.6041),
                    view.point(7.64624, 46.5805),
                    view.point(8.30805, 47.0533),
                );
                path.close();
                path.move_to(view.point(44.9917, 46.4859));
                path.cubic_bezier_to(
                    view.point(45.4171, 45.8241),
                    view.point(45.1808, 46.2968),
                    view.point(45.3699, 45.9895),
                );
                path.cubic_bezier_to(
                    view.point(45.5117, 25.9445),
                    view.point(45.4644, 45.635),
                    view.point(45.5117, 36.6998),
                );
                path.cubic_bezier_to(
                    view.point(44.779, 5.42669),
                    view.point(45.5117, 4.57572),
                    view.point(45.559, 5.994),
                );
                path.cubic_bezier_to(
                    view.point(44.2117, 5.37941),
                    view.point(44.4481, 5.16667),
                    view.point(44.4244, 5.16667),
                );
                path.cubic_bezier_to(
                    view.point(43.9281, 12.1635),
                    view.point(43.999, 5.59216),
                    view.point(43.9753, 6.15947),
                );
                path.cubic_bezier_to(
                    view.point(43.3371, 19.8695),
                    view.point(43.8335, 18.7822),
                    view.point(43.7626, 19.8695),
                );
                path.cubic_bezier_to(
                    view.point(42.9353, 18.924),
                    view.point(43.2899, 19.8695),
                    view.point(43.1008, 19.444),
                );
                path.cubic_bezier_to(
                    view.point(42.2735, 16.7966),
                    view.point(42.7699, 18.3803),
                    view.point(42.4626, 17.4348),
                );
                path.cubic_bezier_to(
                    view.point(36.9081, 8.97239),
                    view.point(41.2808, 13.6291),
                    view.point(39.4608, 10.958),
                );
                path.cubic_bezier_to(
                    view.point(11.9717, 15.4492),
                    view.point(28.3753, 2.35375),
                    view.point(16.0844, 5.54488),
                );
                path.cubic_bezier_to(
                    view.point(11.8535, 27.5282),
                    view.point(10.5062, 18.9476),
                    view.point(10.459, 24.1716),
                );
                path.cubic_bezier_to(
                    view.point(20.6462, 36.5343),
                    view.point(13.579, 31.7358),
                    view.point(16.5335, 34.7378),
                );
                path.cubic_bezier_to(
                    view.point(26.8626, 37.6217),
                    view.point(22.6553, 37.4089),
                    view.point(23.8608, 37.6217),
                );
                path.cubic_bezier_to(
                    view.point(33.339, 36.4398),
                    view.point(29.9353, 37.6217),
                    view.point(31.2117, 37.3853),
                );
                path.cubic_bezier_to(
                    view.point(42.7226, 24.9517),
                    view.point(38.1608, 34.2887),
                    view.point(41.3517, 30.4121),
                );
                path.cubic_bezier_to(
                    view.point(43.4317, 23.4389),
                    view.point(43.0535, 23.6752),
                    view.point(43.2426, 23.2498),
                );
                path.cubic_bezier_to(
                    view.point(43.9281, 35.4706),
                    view.point(43.8099, 23.8644),
                    view.point(43.8571, 24.6208),
                );
                path.cubic_bezier_to(
                    view.point(44.4953, 46.8169),
                    view.point(43.999, 46.8405),
                    view.point(43.999, 46.8169),
                );
                path.cubic_bezier_to(
                    view.point(44.9917, 46.4859),
                    view.point(44.5899, 46.8169),
                    view.point(44.8026, 46.675),
                );
                path.close();
                path.move_to(view.point(26.1535, 34.9506));
                path.cubic_bezier_to(
                    view.point(23.5062, 33.6269),
                    view.point(25.6099, 34.3596),
                    view.point(25.1371, 34.1233),
                );
                path.cubic_bezier_to(
                    view.point(18.2353, 30.5539),
                    view.point(21.5681, 33.0359),
                    view.point(19.7244, 31.9486),
                );
                path.cubic_bezier_to(
                    view.point(14.8081, 24.9281),
                    view.point(16.4862, 28.8756),
                    view.point(15.6826, 27.5755),
                );
                path.cubic_bezier_to(
                    view.point(13.6026, 22.4933),
                    view.point(14.2881, 23.297),
                    view.point(14.2171, 23.1788),
                );
                path.cubic_bezier_to(
                    view.point(13.5317, 21.2405),
                    view.point(12.9881, 21.8315),
                    view.point(12.9881, 21.6897),
                );
                path.cubic_bezier_to(
                    view.point(14.5008, 19.515),
                    view.point(14.1226, 20.7441),
                    view.point(14.3353, 20.3659),
                );
                path.cubic_bezier_to(
                    view.point(17.0535, 13.8891),
                    view.point(14.9026, 17.5294),
                    view.point(15.8953, 15.331),
                );
                path.cubic_bezier_to(
                    view.point(23.6244, 9.51607),
                    view.point(18.7553, 11.738),
                    view.point(20.6462, 10.4852),
                );
                path.cubic_bezier_to(
                    view.point(26.059, 8.57055),
                    view.point(24.7117, 9.18513),
                    view.point(25.799, 8.75965),
                );
                path.cubic_bezier_to(
                    view.point(27.5481, 8.61782),
                    view.point(26.6735, 8.1687),
                    view.point(26.839, 8.1687),
                );
                path.cubic_bezier_to(
                    view.point(28.8717, 9.13786),
                    view.point(27.879, 8.83057),
                    view.point(28.4699, 9.06695),
                );
                path.cubic_bezier_to(
                    view.point(39.4608, 19.9641),
                    view.point(34.4971, 10.2961),
                    view.point(38.0662, 13.9127),
                );
                path.cubic_bezier_to(
                    view.point(39.9335, 21.0514),
                    view.point(39.5553, 20.295),
                    view.point(39.7681, 20.7914),
                );
                path.cubic_bezier_to(
                    view.point(39.8862, 22.517),
                    view.point(40.3353, 21.6424),
                    view.point(40.3353, 21.7133),
                );
                path.cubic_bezier_to(
                    view.point(39.2481, 24.1716),
                    view.point(39.6735, 22.8716),
                    view.point(39.3899, 23.628),
                );
                path.cubic_bezier_to(
                    view.point(29.6517, 33.816),
                    view.point(37.9953, 29.0647),
                    view.point(34.4735, 32.6104),
                );
                path.cubic_bezier_to(
                    view.point(27.3117, 34.9033),
                    view.point(27.9499, 34.2414),
                    view.point(27.7844, 34.336),
                );
                path.cubic_bezier_to(
                    view.point(26.1535, 34.9506),
                    view.point(26.8153, 35.5179),
                    view.point(26.7208, 35.5179),
                );
                path.close();
                path.move_to(view.point(30.7626, 31.4522));
                path.cubic_bezier_to(
                    view.point(36.7662, 25.4008),
                    view.point(33.4808, 30.4121),
                    view.point(35.7499, 28.1192),
                );
                path.cubic_bezier_to(
                    view.point(36.8608, 23.4861),
                    view.point(37.2862, 24.0062),
                    view.point(37.3099, 23.8171),
                );
                path.cubic_bezier_to(
                    view.point(34.9935, 23.5571),
                    view.point(36.4826, 23.1788),
                    view.point(35.9862, 23.2025),
                );
                path.cubic_bezier_to(
                    view.point(32.6062, 24.1007),
                    view.point(34.7335, 23.6516),
                    view.point(33.6462, 23.9116),
                );
                path.cubic_bezier_to(
                    view.point(29.6753, 25.2826),
                    view.point(30.6208, 24.5026),
                    view.point(30.1008, 24.7153),
                );
                path.cubic_bezier_to(
                    view.point(28.2335, 31.3576),
                    view.point(29.2026, 25.9209),
                    view.point(28.0681, 30.6957),
                );
                path.cubic_bezier_to(
                    view.point(30.7626, 31.4522),
                    view.point(28.4226, 32.0667),
                    view.point(29.1081, 32.0904),
                );
                path.close();
                path.move_to(view.point(24.7117, 31.7595));
                path.cubic_bezier_to(
                    view.point(24.8535, 29.8448),
                    view.point(25.1371, 31.6176),
                    view.point(25.1608, 31.1449),
                );
                path.cubic_bezier_to(
                    view.point(24.3335, 27.481),
                    view.point(24.6881, 29.1593),
                    view.point(24.4517, 28.0956),
                );
                path.cubic_bezier_to(
                    view.point(21.8753, 24.3608),
                    view.point(23.9553, 25.4008),
                    view.point(23.3644, 24.668),
                );
                path.cubic_bezier_to(
                    view.point(19.7717, 23.888),
                    view.point(21.4735, 24.2662),
                    view.point(20.5281, 24.0771),
                );
                path.cubic_bezier_to(
                    view.point(16.8408, 23.7934),
                    view.point(17.5499, 23.3916),
                    view.point(17.1008, 23.368),
                );
                path.cubic_bezier_to(
                    view.point(19.7953, 29.4193),
                    view.point(16.2971, 24.6208),
                    view.point(18.0226, 27.8828),
                );
                path.cubic_bezier_to(
                    view.point(24.7117, 31.7595),
                    view.point(21.6153, 30.9794),
                    view.point(23.9081, 32.0667),
                );
                path.close();
                path.move_to(view.point(36.9081, 20.3659));
                path.cubic_bezier_to(
                    view.point(36.8135, 17.9312),
                    view.point(37.3099, 20.0823),
                    view.point(37.2862, 19.3731),
                );
                path.cubic_bezier_to(
                    view.point(29.2735, 11.3362),
                    view.point(35.7735, 14.7873),
                    view.point(32.299, 11.7617),
                );
                path.cubic_bezier_to(
                    view.point(28.0681, 12.0217),
                    view.point(28.5644, 11.2416),
                    view.point(28.0681, 11.5253),
                );
                path.cubic_bezier_to(
                    view.point(29.1553, 17.1984),
                    view.point(28.0681, 12.4708),
                    view.point(28.7771, 15.8274),
                );
                path.cubic_bezier_to(
                    view.point(33.9771, 19.9877),
                    view.point(29.6517, 18.9713),
                    view.point(29.959, 19.1604),
                );
                path.cubic_bezier_to(
                    view.point(35.8681, 20.4132),
                    view.point(34.8044, 20.1532),
                    view.point(35.6553, 20.3423),
                );
                path.cubic_bezier_to(
                    view.point(36.9081, 20.3659),
                    view.point(36.4117, 20.6023),
                    view.point(36.6008, 20.5787),
                );
                path.close();
                path.move_to(view.point(18.779, 20.1059));
                path.cubic_bezier_to(
                    view.point(23.1044, 18.9004),
                    view.point(20.4099, 19.7986),
                    view.point(22.6553, 19.184),
                );
                path.cubic_bezier_to(
                    view.point(24.1917, 16.7729),
                    view.point(23.6717, 18.5458),
                    view.point(23.9553, 18.0258),
                );
                path.cubic_bezier_to(
                    view.point(24.7826, 14.1255),
                    view.point(24.3099, 16.182),
                    view.point(24.5935, 14.9765),
                );
                path.cubic_bezier_to(
                    view.point(22.2771, 12.069),
                    view.point(25.539, 11.1235),
                    view.point(25.0662, 10.7452),
                );
                path.cubic_bezier_to(
                    view.point(17.0062, 17.9076),
                    view.point(19.8662, 13.2036),
                    view.point(17.9044, 15.4019),
                );
                path.cubic_bezier_to(
                    view.point(16.8644, 20.0823),
                    view.point(16.6044, 19.0658),
                    view.point(16.5571, 19.7277),
                );
                path.cubic_bezier_to(
                    view.point(18.779, 20.1059),
                    view.point(17.1244, 20.3659),
                    view.point(17.3844, 20.3659),
                );
                path.close();
                path.move_to(view.point(42.3681, 3.84294));
                path.cubic_bezier_to(
                    view.point(41.5881, 2.61376),
                    view.point(42.9826, 3.48837),
                    view.point(42.5099, 2.73195),
                );
                path.cubic_bezier_to(
                    view.point(26.0353, 2.51921),
                    view.point(41.3044, 2.56649),
                    view.point(34.3081, 2.51921),
                );
                path.cubic_bezier_to(
                    view.point(10.1044, 3.25199),
                    view.point(9.4426, 2.49557),
                    view.point(10.3171, 2.4483),
                );
                path.cubic_bezier_to(
                    view.point(10.7899, 3.96113),
                    view.point(10.0099, 3.65384),
                    view.point(10.1753, 3.8193),
                );
                path.cubic_bezier_to(
                    view.point(42.3681, 3.84294),
                    view.point(11.6881, 4.1266),
                    view.point(42.0371, 4.03204),
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
