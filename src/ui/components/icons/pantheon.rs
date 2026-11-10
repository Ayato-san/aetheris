use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct PantheonIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl PantheonIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for PantheonIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(20.1798, 51.8859));
                path.cubic_bezier_to(
                    view.point(12.238, 50.9404),
                    view.point(15.8071, 51.6495),
                    view.point(15.2634, 51.5786),
                );
                path.cubic_bezier_to(
                    view.point(9.02342, 48.9785),
                    view.point(9.77979, 50.4204),
                    view.point(8.83433, 49.8531),
                );
                path.cubic_bezier_to(
                    view.point(10.0871, 46.544),
                    view.point(9.16524, 48.3404),
                    view.point(9.82706, 46.804),
                );
                path.cubic_bezier_to(
                    view.point(12.3561, 45.693),
                    view.point(10.6543, 45.9294),
                    view.point(11.718, 45.5276),
                );
                path.cubic_bezier_to(
                    view.point(12.2616, 46.6149),
                    view.point(12.7343, 45.7876),
                    view.point(12.7107, 46.0712),
                );
                path.cubic_bezier_to(
                    view.point(12.1434, 47.8913),
                    view.point(11.8125, 47.1585),
                    view.point(11.7652, 47.5603),
                );
                path.cubic_bezier_to(
                    view.point(16.0198, 48.9785),
                    view.point(12.5452, 48.2694),
                    view.point(14.5307, 48.8367),
                );
                path.cubic_bezier_to(
                    view.point(36.9852, 48.9785),
                    view.point(22.5671, 49.664),
                    view.point(31.8325, 49.664),
                );
                path.cubic_bezier_to(
                    view.point(40.6016, 47.9149),
                    view.point(37.8362, 48.8604),
                    view.point(39.5852, 48.3404),
                );
                path.cubic_bezier_to(
                    view.point(42.0907, 48.3876),
                    view.point(41.3107, 47.6076),
                    view.point(41.6889, 47.7258),
                );
                path.cubic_bezier_to(
                    view.point(42.0434, 49.8295),
                    view.point(42.4452, 49.0022),
                    view.point(42.4216, 49.4749),
                );
                path.cubic_bezier_to(
                    view.point(39.6089, 50.7749),
                    view.point(41.7362, 50.1368),
                    view.point(40.3889, 50.6568),
                );
                path.cubic_bezier_to(
                    view.point(38.0489, 51.0822),
                    view.point(39.3725, 50.8222),
                    view.point(38.6634, 50.964),
                );
                path.cubic_bezier_to(
                    view.point(20.1798, 51.8859),
                    view.point(34.8816, 51.7913),
                    view.point(25.6871, 52.2168),
                );
                path.close();
                path.move_to(view.point(28.4525, 46.4021));
                path.cubic_bezier_to(
                    view.point(24.3634, 46.2367),
                    view.point(27.3416, 46.1894),
                    view.point(25.4507, 46.1185),
                );
                path.cubic_bezier_to(
                    view.point(21.5507, 46.0476),
                    view.point(23.8198, 46.3076),
                    view.point(23.1343, 46.2603),
                );
                path.cubic_bezier_to(
                    view.point(17.9107, 45.5985),
                    view.point(20.4162, 45.8821),
                    view.point(18.7616, 45.693),
                );
                path.cubic_bezier_to(
                    view.point(15.5943, 43.1166),
                    view.point(14.5543, 45.2439),
                    view.point(14.3416, 45.0076),
                );
                path.cubic_bezier_to(
                    view.point(16.3507, 41.8638),
                    view.point(15.9252, 42.6439),
                    view.point(16.2562, 42.0766),
                );
                path.cubic_bezier_to(
                    view.point(17.5325, 36.2383),
                    view.point(16.5634, 41.3675),
                    view.point(17.3198, 37.7274),
                );
                path.cubic_bezier_to(
                    view.point(17.9107, 33.9455),
                    view.point(17.6271, 35.5765),
                    view.point(17.7925, 34.5601),
                );
                path.cubic_bezier_to(
                    view.point(17.3671, 27.7763),
                    view.point(18.2889, 31.9836),
                    view.point(17.9343, 27.7763),
                );
                path.cubic_bezier_to(
                    view.point(16.658, 29.36),
                    view.point(17.1543, 27.7763),
                    view.point(16.8943, 28.3672),
                );
                path.cubic_bezier_to(
                    view.point(15.6889, 30.7309),
                    view.point(16.3743, 30.5654),
                    view.point(16.138, 30.8964),
                );
                path.cubic_bezier_to(
                    view.point(14.8852, 27.1854),
                    view.point(15.0507, 30.5182),
                    view.point(14.9798, 30.1636),
                );
                path.cubic_bezier_to(
                    view.point(14.6961, 23.8526),
                    view.point(14.838, 25.7199),
                    view.point(14.7671, 24.2071),
                );
                path.cubic_bezier_to(
                    view.point(14.4125, 22.2453),
                    view.point(14.6489, 23.498),
                    view.point(14.5071, 22.7653),
                );
                path.cubic_bezier_to(
                    view.point(15.4289, 17.0688),
                    view.point(13.9634, 19.7398),
                    view.point(14.1761, 18.6761),
                );
                path.cubic_bezier_to(
                    view.point(16.6816, 14.7051),
                    view.point(16.2561, 16.0051),
                    view.point(16.3743, 15.7924),
                );
                path.cubic_bezier_to(
                    view.point(18.0289, 11.3723),
                    view.point(16.9652, 13.6178),
                    view.point(17.438, 12.4832),
                );
                path.cubic_bezier_to(
                    view.point(18.0289, 8.37043),
                    view.point(18.5489, 10.3559),
                    view.point(18.5489, 10.025),
                );
                path.cubic_bezier_to(
                    view.point(18.0289, 4.99035),
                    view.point(17.6034, 7.02312),
                    view.point(17.6034, 6.36129),
                );
                path.cubic_bezier_to(
                    view.point(20.8652, 1.37392),
                    view.point(18.6907, 2.93395),
                    view.point(19.4943, 1.91756),
                );
                path.cubic_bezier_to(
                    view.point(25.3325, 3.14668),
                    view.point(22.7562, 0.617536),
                    view.point(24.3161, 1.23209),
                );
                path.cubic_bezier_to(
                    view.point(26.278, 4.75399),
                    view.point(25.5452, 3.52487),
                    view.point(25.9471, 4.25761),
                );
                path.cubic_bezier_to(
                    view.point(27.058, 7.23586),
                    view.point(26.9398, 5.86492),
                    view.point(27.058, 6.26674),
                );
                path.cubic_bezier_to(
                    view.point(26.1598, 9.36317),
                    view.point(27.058, 8.18133),
                    view.point(26.8925, 8.55952),
                );
                path.cubic_bezier_to(
                    view.point(25.5925, 10.1668),
                    view.point(25.8052, 9.74136),
                    view.point(25.5689, 10.0959),
                );
                path.cubic_bezier_to(
                    view.point(26.3962, 10.3323),
                    view.point(25.6161, 10.2377),
                    view.point(25.9471, 10.3086),
                );
                path.cubic_bezier_to(
                    view.point(28.1216, 11.7505),
                    view.point(27.318, 10.4032),
                    view.point(27.4834, 10.545),
                );
                path.cubic_bezier_to(
                    view.point(29.4925, 16.076),
                    view.point(28.6889, 12.8142),
                    view.point(29.138, 14.256),
                );
                path.cubic_bezier_to(
                    view.point(29.9889, 18.1561),
                    view.point(29.6343, 16.8797),
                    view.point(29.8707, 17.8252),
                );
                path.cubic_bezier_to(
                    view.point(30.2489, 19.6452),
                    view.point(30.1307, 18.5579),
                    view.point(30.2252, 19.1016),
                );
                path.cubic_bezier_to(
                    view.point(30.2725, 20.7561),
                    view.point(30.2489, 20.1179),
                    view.point(30.2725, 20.6143),
                );
                path.cubic_bezier_to(
                    view.point(30.8398, 21.2761),
                    view.point(30.2962, 20.9216),
                    view.point(30.438, 21.0634),
                );
                path.cubic_bezier_to(
                    view.point(32.1871, 25.0344),
                    view.point(31.7143, 21.7253),
                    view.point(32.0216, 22.5289),
                );
                path.cubic_bezier_to(
                    view.point(31.478, 29.4072),
                    view.point(32.2816, 26.4999),
                    view.point(32.1162, 27.5636),
                );
                path.cubic_bezier_to(
                    view.point(30.8871, 31.44),
                    view.point(31.1943, 30.2582),
                    view.point(30.9343, 31.18),
                );
                path.cubic_bezier_to(
                    view.point(31.3598, 36.1674),
                    view.point(30.8398, 31.8891),
                    view.point(31.0762, 34.4892),
                );
                path.cubic_bezier_to(
                    view.point(29.6343, 35.6001),
                    view.point(31.5962, 37.6801),
                    view.point(30.8398, 37.4201),
                );
                path.cubic_bezier_to(
                    view.point(28.7125, 32.4091),
                    view.point(28.8071, 34.3237),
                    view.point(28.6416, 33.7564),
                );
                path.cubic_bezier_to(
                    view.point(29.8234, 28.7927),
                    view.point(28.7598, 31.3455),
                    view.point(28.7834, 31.2982),
                );
                path.cubic_bezier_to(
                    view.point(30.1543, 25.8144),
                    view.point(30.2962, 27.6345),
                    view.point(30.438, 26.4763),
                );
                path.cubic_bezier_to(
                    view.point(29.0198, 26.7599),
                    view.point(29.8471, 25.0817),
                    view.point(29.5634, 25.2944),
                );
                path.cubic_bezier_to(
                    view.point(27.9798, 28.7454),
                    view.point(28.8307, 27.2326),
                    view.point(28.358, 28.1308),
                );
                path.cubic_bezier_to(
                    view.point(26.6562, 31.1564),
                    view.point(27.578, 29.36),
                    view.point(26.9871, 30.4473),
                );
                path.cubic_bezier_to(
                    view.point(26.018, 33.4491),
                    view.point(26.0889, 32.3618),
                    view.point(26.0652, 32.5037),
                );
                path.cubic_bezier_to(
                    view.point(27.578, 36.0492),
                    view.point(25.9234, 35.1273),
                    view.point(26.2071, 35.6001),
                );
                path.cubic_bezier_to(
                    view.point(29.3507, 37.8929),
                    view.point(28.4998, 36.3801),
                    view.point(28.6416, 36.5456),
                );
                path.cubic_bezier_to(
                    view.point(29.7052, 39.6184),
                    view.point(29.6816, 38.5311),
                    view.point(29.7052, 38.6492),
                );
                path.cubic_bezier_to(
                    view.point(28.2634, 42.6675),
                    view.point(29.6816, 41.2729),
                    view.point(29.398, 41.8875),
                );
                path.cubic_bezier_to(
                    view.point(27.1998, 43.4948),
                    view.point(27.9798, 42.8566),
                    view.point(27.5071, 43.2112),
                );
                path.cubic_bezier_to(
                    view.point(26.1598, 44.393),
                    view.point(26.9161, 43.7548),
                    view.point(26.4434, 44.1566),
                );
                path.cubic_bezier_to(
                    view.point(25.6634, 45.0076),
                    view.point(25.8762, 44.6294),
                    view.point(25.6634, 44.8894),
                );
                path.cubic_bezier_to(
                    view.point(26.3489, 44.7476),
                    view.point(25.6634, 45.3148),
                    view.point(25.7107, 45.2912),
                );
                path.cubic_bezier_to(
                    view.point(28.4052, 43.9203),
                    view.point(27.0107, 44.1803),
                    view.point(27.2234, 44.0857),
                );
                path.cubic_bezier_to(
                    view.point(30.8398, 40.2802),
                    view.point(30.7216, 43.5893),
                    view.point(31.6198, 42.242),
                );
                path.cubic_bezier_to(
                    view.point(31.3362, 38.9565),
                    view.point(30.4616, 39.3111),
                    view.point(30.6034, 38.9329),
                );
                path.cubic_bezier_to(
                    view.point(33.1798, 40.4693),
                    view.point(31.9271, 38.9565),
                    view.point(32.4943, 39.4293),
                );
                path.cubic_bezier_to(
                    view.point(34.7634, 42.3602),
                    view.point(33.5107, 40.9657),
                    view.point(34.2198, 41.8402),
                );
                path.cubic_bezier_to(
                    view.point(35.898, 44.7476),
                    view.point(36.2052, 43.8257),
                    view.point(36.3234, 44.0621),
                );
                path.cubic_bezier_to(
                    view.point(29.9889, 46.6149),
                    view.point(35.3071, 45.7167),
                    view.point(32.3052, 46.6621),
                );
                path.cubic_bezier_to(
                    view.point(28.4525, 46.4021),
                    view.point(29.7052, 46.6149),
                    view.point(29.0198, 46.5203),
                );
                path.close();
                path.move_to(view.point(38.6634, 40.0438));
                path.cubic_bezier_to(
                    view.point(34.6452, 34.2291),
                    view.point(37.2452, 39.3111),
                    view.point(35.5907, 36.9001),
                );
                path.cubic_bezier_to(
                    view.point(33.3452, 31.0854),
                    view.point(34.4325, 33.6146),
                    view.point(34.2198, 33.1182),
                );
                path.cubic_bezier_to(
                    view.point(33.4634, 28.7927),
                    view.point(33.0143, 30.3291),
                    view.point(33.038, 29.8091),
                );
                path.cubic_bezier_to(
                    view.point(33.9125, 26.8308),
                    view.point(33.6052, 28.4145),
                    view.point(33.818, 27.5636),
                );
                path.cubic_bezier_to(
                    view.point(33.3925, 21.5834),
                    view.point(34.1489, 25.2944),
                    view.point(33.9598, 23.5453),
                );
                path.cubic_bezier_to(
                    view.point(33.3452, 19.4325),
                    view.point(33.0143, 20.2361),
                    view.point(33.0143, 19.787),
                );
                path.cubic_bezier_to(
                    view.point(33.9125, 19.1725),
                    view.point(33.5107, 19.267),
                    view.point(33.7471, 19.1725),
                );
                path.cubic_bezier_to(
                    view.point(35.3071, 18.487),
                    view.point(34.3616, 19.1725),
                    view.point(35.0943, 18.7943),
                );
                path.cubic_bezier_to(
                    view.point(35.4016, 17.3524),
                    view.point(35.5671, 18.0852),
                    view.point(35.6143, 17.5179),
                );
                path.cubic_bezier_to(
                    view.point(34.0307, 17.8961),
                    view.point(35.2125, 17.187),
                    view.point(34.9998, 17.2579),
                );
                path.cubic_bezier_to(
                    view.point(32.518, 18.0379),
                    view.point(33.4634, 18.2743),
                    view.point(32.9434, 18.3215),
                );
                path.cubic_bezier_to(
                    view.point(30.8398, 13.6651),
                    view.point(31.9034, 17.6361),
                    view.point(30.8398, 14.8469),
                );
                path.cubic_bezier_to(
                    view.point(31.3598, 12.2469),
                    view.point(30.8398, 13.3105),
                    view.point(30.958, 12.9796),
                );
                path.cubic_bezier_to(
                    view.point(32.2343, 7.68496),
                    view.point(32.2107, 10.7814),
                    view.point(32.4707, 9.41045),
                );
                path.cubic_bezier_to(
                    view.point(31.6434, 6.03038),
                    view.point(32.0689, 6.59766),
                    view.point(31.8798, 6.03038),
                );
                path.cubic_bezier_to(
                    view.point(31.1943, 7.37768),
                    view.point(31.3834, 6.03038),
                    view.point(31.2889, 6.33766),
                );
                path.cubic_bezier_to(
                    view.point(30.5562, 9.59954),
                    view.point(31.0998, 8.58316),
                    view.point(30.9343, 9.19772),
                );
                path.cubic_bezier_to(
                    view.point(29.6816, 9.67045),
                    view.point(30.2016, 9.97773),
                    view.point(29.9652, 9.97773),
                );
                path.cubic_bezier_to(
                    view.point(29.398, 7.14131),
                    view.point(29.2089, 9.12681),
                    view.point(29.1143, 8.34679),
                );
                path.cubic_bezier_to(
                    view.point(29.2798, 3.45396),
                    view.point(29.5871, 6.29038),
                    view.point(29.5398, 4.6358),
                );
                path.cubic_bezier_to(
                    view.point(29.3034, 0.995725),
                    view.point(29.0434, 2.43757),
                    view.point(29.0671, 1.4921),
                );
                path.cubic_bezier_to(
                    view.point(34.8816, 2.60303),
                    view.point(30.2016, -0.753402),
                    view.point(32.2107, -0.186117),
                );
                path.cubic_bezier_to(
                    view.point(40.7434, 13.0742),
                    view.point(38.1434, 6.00674),
                    view.point(40.2234, 9.74136),
                );
                path.cubic_bezier_to(
                    view.point(41.1216, 15.5088),
                    view.point(40.8143, 13.5469),
                    view.point(40.9798, 14.6578),
                );
                path.cubic_bezier_to(
                    view.point(41.0743, 28.1072),
                    view.point(41.7598, 19.267),
                    view.point(41.7362, 26.7835),
                );
                path.cubic_bezier_to(
                    view.point(39.7271, 27.8472),
                    view.point(40.6962, 28.8872),
                    view.point(40.3889, 28.8163),
                );
                path.cubic_bezier_to(
                    view.point(38.8525, 27.6345),
                    view.point(39.2071, 27.1381),
                    view.point(38.9471, 27.0672),
                );
                path.cubic_bezier_to(
                    view.point(39.9634, 31.2036),
                    view.point(38.6871, 28.5327),
                    view.point(39.1362, 29.9745),
                );
                path.cubic_bezier_to(
                    view.point(40.7434, 32.9291),
                    view.point(40.5543, 32.0782),
                    view.point(40.6252, 32.22),
                );
                path.cubic_bezier_to(
                    view.point(41.0271, 34.2528),
                    view.point(40.7907, 33.26),
                    view.point(40.9325, 33.851),
                );
                path.cubic_bezier_to(
                    view.point(38.7816, 34.111),
                    view.point(41.4052, 35.8837),
                    view.point(40.6725, 35.8365),
                );
                path.cubic_bezier_to(
                    view.point(37.6707, 33.26),
                    view.point(38.2852, 33.6619),
                    view.point(37.7652, 33.26),
                );
                path.cubic_bezier_to(
                    view.point(39.0889, 36.0965),
                    view.point(37.1507, 33.26),
                    view.point(37.8834, 34.7255),
                );
                path.cubic_bezier_to(
                    view.point(40.0343, 38.6256),
                    view.point(39.9398, 37.0656),
                    view.point(40.0343, 37.3492),
                );
                path.cubic_bezier_to(
                    view.point(39.7271, 40.0202),
                    view.point(40.0343, 39.7129),
                    view.point(40.0343, 39.7602),
                );
                path.cubic_bezier_to(
                    view.point(38.6634, 40.0438),
                    view.point(39.3725, 40.3275),
                    view.point(39.1834, 40.3275),
                );
                path.close();
                path.move_to(view.point(16.5634, 21.678));
                path.cubic_bezier_to(
                    view.point(17.0125, 20.0943),
                    view.point(16.6107, 21.4416),
                    view.point(16.7998, 20.7325),
                );
                path.cubic_bezier_to(
                    view.point(17.1307, 18.3688),
                    view.point(17.4143, 18.8652),
                    view.point(17.438, 18.5579),
                );
                path.cubic_bezier_to(
                    view.point(15.9725, 19.1016),
                    view.point(16.8471, 18.2034),
                    view.point(16.2798, 18.5579),
                );
                path.cubic_bezier_to(
                    view.point(15.9016, 22.2216),
                    view.point(15.618, 19.7161),
                    view.point(15.5707, 21.7253),
                );
                path.line_to(view.point(16.1143, 22.5525));
                path.line_to(view.point(16.3034, 22.3162));
                path.cubic_bezier_to(
                    view.point(16.5634, 21.678),
                    view.point(16.398, 22.1744),
                    view.point(16.5161, 21.8907),
                );
                path.close();
                path.move_to(view.point(24.6234, 19.4797));
                path.cubic_bezier_to(
                    view.point(25.3325, 18.1561),
                    view.point(24.978, 19.2197),
                    view.point(25.3325, 18.5343),
                );
                path.cubic_bezier_to(
                    view.point(24.978, 17.3524),
                    view.point(25.3325, 18.0143),
                    view.point(25.1671, 17.6597),
                );
                path.cubic_bezier_to(
                    view.point(24.5289, 16.407),
                    view.point(24.7889, 17.0688),
                    view.point(24.5762, 16.6433),
                );
                path.cubic_bezier_to(
                    view.point(24.0561, 15.3906),
                    view.point(24.458, 15.9342),
                    view.point(24.198, 15.3906),
                );
                path.cubic_bezier_to(
                    view.point(23.418, 16.2415),
                    view.point(23.8907, 15.3906),
                    view.point(23.5125, 15.9106),
                );
                path.cubic_bezier_to(
                    view.point(23.3707, 17.187),
                    view.point(23.3471, 16.4306),
                    view.point(23.3471, 16.8561),
                );
                path.cubic_bezier_to(
                    view.point(23.1343, 18.3452),
                    view.point(23.418, 17.7306),
                    view.point(23.3707, 17.8961),
                );
                path.cubic_bezier_to(
                    view.point(22.8743, 19.007),
                    view.point(22.9925, 18.6288),
                    view.point(22.8743, 18.9125),
                );
                path.cubic_bezier_to(
                    view.point(24.2216, 19.6925),
                    view.point(22.8743, 19.2197),
                    view.point(23.7489, 19.6688),
                );
                path.cubic_bezier_to(
                    view.point(24.6234, 19.4797),
                    view.point(24.2925, 19.6925),
                    view.point(24.4816, 19.5979),
                );
                path.close();
                path.move_to(view.point(35.7562, 14.0433));
                path.cubic_bezier_to(
                    view.point(35.9689, 12.176),
                    view.point(36.5362, 13.4524),
                    view.point(36.7016, 12.0105),
                );
                path.cubic_bezier_to(
                    view.point(34.1725, 14.0905),
                    view.point(35.7798, 12.2232),
                    view.point(34.1725, 13.9251),
                );
                path.cubic_bezier_to(
                    view.point(35.7562, 14.0433),
                    view.point(34.1725, 14.4924),
                    view.point(35.2125, 14.4687),
                );
                path.close();
                path.move_to(view.point(21.0307, 11.845));
                path.cubic_bezier_to(
                    view.point(25.238, 7.9686),
                    view.point(22.1416, 11.1596),
                    view.point(25.0252, 8.51225),
                );
                path.cubic_bezier_to(
                    view.point(24.2216, 6.4322),
                    view.point(25.4271, 7.54313),
                    view.point(25.1671, 7.14131),
                );
                path.cubic_bezier_to(
                    view.point(21.1489, 5.22672),
                    view.point(22.898, 5.43945),
                    view.point(22.378, 5.25036),
                );
                path.cubic_bezier_to(
                    view.point(19.0689, 5.95947),
                    view.point(20.0143, 5.22672),
                    view.point(19.4707, 5.41582),
                );
                path.cubic_bezier_to(
                    view.point(19.3052, 7.73223),
                    view.point(18.7616, 6.36129),
                    view.point(18.8089, 6.69221),
                );
                path.cubic_bezier_to(
                    view.point(19.8962, 10.6396),
                    view.point(19.8252, 8.8668),
                    view.point(19.9907, 9.59954),
                );
                path.cubic_bezier_to(
                    view.point(21.0307, 11.845),
                    view.point(19.778, 11.9632),
                    view.point(20.1325, 12.3651),
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
