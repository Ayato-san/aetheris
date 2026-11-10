use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct ArmorIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl ArmorIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for ArmorIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(14.3238, 51.9262));
                path.cubic_bezier_to(
                    view.point(11.2376, 51.42),
                    view.point(12.1297, 51.8539),
                    view.point(12.0332, 51.8298),
                );
                path.cubic_bezier_to(
                    view.point(9.98377, 50.8897),
                    view.point(10.8036, 51.1789),
                    view.point(10.2249, 50.9379),
                );
                path.cubic_bezier_to(
                    view.point(9.64622, 49.5156),
                    view.point(9.21221, 50.7209),
                    view.point(9.16399, 50.504),
                );
                path.cubic_bezier_to(
                    view.point(10.2972, 48.479),
                    view.point(9.88733, 49.0576),
                    view.point(10.1767, 48.5755),
                );
                path.cubic_bezier_to(
                    view.point(17.6271, 48.6237),
                    view.point(10.5383, 48.2862),
                    view.point(15.457, 48.3826),
                );
                path.cubic_bezier_to(
                    view.point(25.9937, 48.5996),
                    view.point(19.6283, 48.8406),
                    view.point(22.377, 48.8406),
                );
                path.cubic_bezier_to(
                    view.point(32.1662, 48.3344),
                    view.point(27.8985, 48.4549),
                    view.point(30.6954, 48.3344),
                );
                path.cubic_bezier_to(
                    view.point(35.7829, 49.5156),
                    view.point(35.373, 48.2862),
                    view.point(35.4453, 48.3103),
                );
                path.cubic_bezier_to(
                    view.point(34.3121, 51.7334),
                    view.point(36.1927, 50.8897),
                    view.point(35.9275, 51.2754),
                );
                path.cubic_bezier_to(
                    view.point(14.3238, 51.9262),
                    view.point(33.5164, 51.9503),
                    view.point(18.6879, 52.095),
                );
                path.close();
                path.move_to(view.point(15.023, 46.1648));
                path.cubic_bezier_to(
                    view.point(13.0218, 45.8756),
                    view.point(14.2997, 46.0925),
                    view.point(13.4076, 45.972),
                );
                path.cubic_bezier_to(
                    view.point(11.4787, 45.6104),
                    view.point(12.6601, 45.8032),
                    view.point(11.9609, 45.6586),
                );
                path.cubic_bezier_to(
                    view.point(10.0802, 44.5497),
                    view.point(10.4419, 45.4658),
                    view.point(10.0802, 45.2006),
                );
                path.cubic_bezier_to(
                    view.point(9.86322, 43.6578),
                    view.point(10.0802, 44.2846),
                    view.point(9.98377, 43.8989),
                );
                path.cubic_bezier_to(
                    view.point(9.1881, 40.4758),
                    view.point(9.47744, 42.9105),
                    view.point(9.28455, 42.0186),
                );
                path.cubic_bezier_to(
                    view.point(8.77821, 38.4268),
                    view.point(9.11577, 39.174),
                    view.point(9.06754, 38.8848),
                );
                path.cubic_bezier_to(
                    view.point(9.76677, 37.8482),
                    view.point(8.07898, 37.3902),
                    view.point(8.85054, 36.9563),
                );
                path.cubic_bezier_to(
                    view.point(11.2858, 38.3544),
                    view.point(10.2731, 38.3544),
                    view.point(10.466, 38.4026),
                );
                path.cubic_bezier_to(
                    view.point(12.1538, 38.7401),
                    view.point(11.5992, 38.3303),
                    view.point(11.8162, 38.4268),
                );
                path.cubic_bezier_to(
                    view.point(12.9736, 39.0294),
                    view.point(12.6601, 39.2223),
                    view.point(12.6119, 39.2223),
                );
                path.cubic_bezier_to(
                    view.point(12.8289, 38.1375),
                    view.point(13.3594, 38.8125),
                    view.point(13.3111, 38.3785),
                );
                path.cubic_bezier_to(
                    view.point(12.2743, 37.4625),
                    view.point(12.5878, 38.0169),
                    view.point(12.3708, 37.7518),
                );
                path.cubic_bezier_to(
                    view.point(11.1411, 36.6429),
                    view.point(12.1056, 36.9081),
                    view.point(11.9127, 36.7875),
                );
                path.cubic_bezier_to(
                    view.point(10.3454, 36.0885),
                    view.point(10.7312, 36.5706),
                    view.point(10.5624, 36.4259),
                );
                path.cubic_bezier_to(
                    view.point(9.16399, 35.3894),
                    view.point(10.0561, 35.6063),
                    view.point(9.98377, 35.5581),
                );
                path.cubic_bezier_to(
                    view.point(8.19954, 33.1716),
                    view.point(8.34421, 35.2206),
                    view.point(7.50031, 33.2921),
                );
                path.cubic_bezier_to(
                    view.point(8.68176, 33.4127),
                    view.point(8.32009, 33.1475),
                    view.point(8.5371, 33.2439),
                );
                path.cubic_bezier_to(
                    view.point(9.93555, 34.0876),
                    view.point(9.1881, 34.0394),
                    view.point(9.28455, 34.0876),
                );
                path.cubic_bezier_to(
                    view.point(11.0206, 34.5939),
                    view.point(10.5142, 34.0876),
                    view.point(10.6348, 34.16),
                );
                path.cubic_bezier_to(
                    view.point(11.9609, 35.0037),
                    view.point(11.4063, 35.0278),
                    view.point(11.5269, 35.076),
                );
                path.cubic_bezier_to(
                    view.point(12.0332, 33.485),
                    view.point(13.2388, 34.8108),
                    view.point(13.287, 33.7502),
                );
                path.cubic_bezier_to(
                    view.point(11.3099, 32.8582),
                    view.point(11.6957, 33.4127),
                    view.point(11.5269, 33.2439),
                );
                path.cubic_bezier_to(
                    view.point(10.466, 32.2315),
                    view.point(11.0688, 32.4002),
                    view.point(10.9482, 32.3038),
                );
                path.cubic_bezier_to(
                    view.point(9.4051, 31.6047),
                    view.point(10.032, 32.1591),
                    view.point(9.76677, 31.9904),
                );
                path.cubic_bezier_to(
                    view.point(8.68176, 31.0744),
                    view.point(9.13988, 31.3154),
                    view.point(8.80232, 31.0744),
                );
                path.cubic_bezier_to(
                    view.point(7.13864, 28.9289),
                    view.point(7.81376, 31.0744),
                    view.point(6.70464, 29.5316),
                );
                path.cubic_bezier_to(
                    view.point(7.81376, 29.1941),
                    view.point(7.33153, 28.6879),
                    view.point(7.35564, 28.712),
                );
                path.cubic_bezier_to(
                    view.point(8.85054, 29.7003),
                    view.point(8.22365, 29.628),
                    view.point(8.36832, 29.7003),
                );
                path.cubic_bezier_to(
                    view.point(9.98377, 30.4476),
                    view.point(9.4051, 29.7003),
                    view.point(9.52566, 29.7967),
                );
                path.cubic_bezier_to(
                    view.point(10.9964, 30.6887),
                    view.point(10.1284, 30.6405),
                    view.point(10.3454, 30.7128),
                );
                path.cubic_bezier_to(
                    view.point(12.2502, 31.1226),
                    view.point(11.7198, 30.6887),
                    view.point(11.8403, 30.7369),
                );
                path.cubic_bezier_to(
                    view.point(12.9253, 31.4119),
                    view.point(12.6119, 31.4601),
                    view.point(12.7325, 31.5324),
                );
                path.cubic_bezier_to(
                    view.point(12.8048, 30.4235),
                    view.point(13.3111, 31.1708),
                    view.point(13.2388, 30.6405),
                );
                path.cubic_bezier_to(
                    view.point(12.2743, 29.7726),
                    view.point(12.5878, 30.3271),
                    view.point(12.3708, 30.0619),
                );
                path.cubic_bezier_to(
                    view.point(11.1411, 28.9771),
                    view.point(12.1056, 29.2423),
                    view.point(11.9127, 29.0977),
                );
                path.cubic_bezier_to(
                    view.point(10.4178, 28.4709),
                    view.point(10.7794, 28.9048),
                    view.point(10.5866, 28.7602),
                );
                path.cubic_bezier_to(
                    view.point(9.4051, 27.7959),
                    view.point(10.2008, 28.037),
                    view.point(9.8391, 27.7959),
                );
                path.cubic_bezier_to(
                    view.point(8.60943, 27.3138),
                    view.point(9.23632, 27.7959),
                    view.point(8.87465, 27.579),
                );
                path.cubic_bezier_to(
                    view.point(7.74142, 26.7353),
                    view.point(8.34421, 27.0245),
                    view.point(7.93431, 26.7835),
                );
                path.cubic_bezier_to(
                    view.point(6.89753, 24.0595),
                    view.point(7.01808, 26.5906),
                    view.point(6.94575, 26.3978),
                );
                path.line_to(view.point(6.87342, 21.914));
                path.line_to(view.point(5.7643, 20.8534));
                path.cubic_bezier_to(
                    view.point(2.60572, 17.1651),
                    view.point(4.14884, 19.2865),
                    view.point(2.84683, 17.7678),
                );
                path.cubic_bezier_to(
                    view.point(2.07527, 16.2732),
                    view.point(2.48516, 16.8517),
                    view.point(2.24405, 16.466),
                );
                path.cubic_bezier_to(
                    view.point(1.4966, 15.2607),
                    view.point(1.88238, 16.0803),
                    view.point(1.61716, 15.6223),
                );
                path.cubic_bezier_to(
                    view.point(0.869708, 14.0554),
                    view.point(1.35193, 14.875),
                    view.point(1.0626, 14.3447),
                );
                path.cubic_bezier_to(
                    view.point(0.411593, 12.6814),
                    view.point(0.628595, 13.7179),
                    view.point(0.483927, 13.2599),
                );
                path.cubic_bezier_to(
                    view.point(0.122258, 11.3073),
                    view.point(0.363371, 12.2233),
                    view.point(0.218703, 11.5966),
                );
                path.cubic_bezier_to(
                    view.point(0.604483, 9.95736),
                    view.point(-0.142966, 10.6564),
                    view.point(0.0258125, 10.1984),
                );
                path.cubic_bezier_to(
                    view.point(1.47249, 9.35471),
                    view.point(0.797374, 9.86094),
                    view.point(1.20727, 9.59577),
                );
                path.cubic_bezier_to(
                    view.point(2.50927, 8.82437),
                    view.point(1.76183, 9.08954),
                    view.point(2.19583, 8.87258),
                );
                path.cubic_bezier_to(
                    view.point(3.78717, 8.24582),
                    view.point(2.82272, 8.77616),
                    view.point(3.40139, 8.51099),
                );
                path.cubic_bezier_to(
                    view.point(5.40263, 8.75205),
                    view.point(5.08918, 7.42621),
                    view.point(5.88485, 7.66727),
                );
                path.cubic_bezier_to(
                    view.point(5.18563, 13.8143),
                    view.point(5.1374, 9.3306),
                    view.point(5.1374, 9.57166),
                );
                path.cubic_bezier_to(
                    view.point(5.90897, 18.9731),
                    view.point(5.23385, 18.5633),
                    view.point(5.28207, 18.8043),
                );
                path.cubic_bezier_to(
                    view.point(7.06631, 15.0197),
                    view.point(6.72875, 19.1659),
                    view.point(6.89753, 18.5392),
                );
                path.cubic_bezier_to(
                    view.point(7.59676, 11.476),
                    view.point(7.18686, 12.4644),
                    view.point(7.21098, 12.2957),
                );
                path.cubic_bezier_to(
                    view.point(8.15131, 9.66809),
                    view.point(7.81376, 10.9939),
                    view.point(8.05487, 10.1743),
                );
                path.cubic_bezier_to(
                    view.point(9.57388, 7.08872),
                    view.point(8.29598, 8.75205),
                    view.point(9.1881, 7.13694),
                );
                path.cubic_bezier_to(
                    view.point(11.117, 7.01641),
                    view.point(9.67033, 7.06462),
                    view.point(10.3696, 7.04051),
                );
                path.cubic_bezier_to(
                    view.point(12.636, 6.96819),
                    view.point(11.8645, 7.01641),
                    view.point(12.5637, 6.9923),
                );
                path.cubic_bezier_to(
                    view.point(13.2388, 6.6066),
                    view.point(12.7325, 6.96819),
                    view.point(12.9977, 6.79945),
                );
                path.cubic_bezier_to(
                    view.point(13.9862, 7.23336),
                    view.point(13.7451, 6.17269),
                    view.point(13.8416, 6.24501),
                );
                path.cubic_bezier_to(
                    view.point(14.5649, 8.51099),
                    view.point(14.1068, 7.93244),
                    view.point(14.2756, 8.34225),
                );
                path.cubic_bezier_to(
                    view.point(14.9507, 9.40292),
                    view.point(14.6614, 8.5592),
                    view.point(14.8301, 8.96901),
                );
                path.cubic_bezier_to(
                    view.point(16.108, 10.8011),
                    view.point(15.3847, 10.9698),
                    view.point(15.5053, 11.1145),
                );
                path.cubic_bezier_to(
                    view.point(16.759, 7.30568),
                    view.point(16.6385, 10.5118),
                    view.point(16.7832, 9.74041),
                );
                path.cubic_bezier_to(
                    view.point(15.5053, 3.81028),
                    view.point(16.759, 4.62989),
                    view.point(16.6626, 4.38883),
                );
                path.cubic_bezier_to(
                    view.point(13.6005, 1.39967),
                    view.point(14.3961, 3.27995),
                    view.point(13.6969, 2.41213),
                );
                path.cubic_bezier_to(
                    view.point(14.4444, 0.0256147),
                    view.point(13.4799, 0.411314),
                    view.point(13.6246, 0.146146),
                );
                path.cubic_bezier_to(
                    view.point(14.6131, 0.652375),
                    view.point(15.1436, -0.07081),
                    view.point(15.1918, 0.0979332),
                );
                path.cubic_bezier_to(
                    view.point(14.589, 2.19517),
                    view.point(14.0827, 1.1586),
                    view.point(14.0827, 1.61662),
                );
                path.cubic_bezier_to(
                    view.point(16.6867, 3.40048),
                    view.point(15.023, 2.67729),
                    view.point(16.2768, 3.40048),
                );
                path.cubic_bezier_to(
                    view.point(17.2172, 9.69219),
                    view.point(17.1689, 3.40048),
                    view.point(17.193, 3.73797),
                );
                path.cubic_bezier_to(
                    view.point(17.8199, 21.7935),
                    view.point(17.2172, 14.5134),
                    view.point(17.4342, 18.6838),
                );
                path.cubic_bezier_to(
                    view.point(18.3022, 28.5191),
                    view.point(18.0611, 23.6256),
                    view.point(18.2057, 25.8433),
                );
                path.cubic_bezier_to(
                    view.point(19.2666, 34.2323),
                    view.point(18.4709, 33.4127),
                    view.point(18.6156, 34.2323),
                );
                path.cubic_bezier_to(
                    view.point(20.2311, 29.2905),
                    view.point(19.8453, 34.2323),
                    view.point(20.2311, 32.3038),
                );
                path.cubic_bezier_to(
                    view.point(20.7856, 21.8417),
                    view.point(20.2311, 27.3861),
                    view.point(20.4722, 24.0595),
                );
                path.cubic_bezier_to(
                    view.point(21.1955, 15.5982),
                    view.point(20.9544, 20.6123),
                    view.point(21.0991, 18.3463),
                );
                path.cubic_bezier_to(
                    view.point(21.6777, 10.3913),
                    view.point(21.3643, 10.7047),
                    view.point(21.3884, 10.3913),
                );
                path.cubic_bezier_to(
                    view.point(21.8706, 10.5118),
                    view.point(21.7742, 10.3913),
                    view.point(21.8706, 10.4395),
                );
                path.cubic_bezier_to(
                    view.point(22.3529, 10.9457),
                    view.point(21.8706, 10.5841),
                    view.point(22.0876, 10.777),
                );
                path.line_to(view.point(22.8351, 11.235));
                path.line_to(view.point(23.4138, 10.8493));
                path.cubic_bezier_to(
                    view.point(25.0774, 9.35471),
                    view.point(23.7272, 10.6323),
                    view.point(24.4747, 9.95736),
                );
                path.cubic_bezier_to(
                    view.point(30.3819, 4.41294),
                    view.point(27.0063, 7.40211),
                    view.point(29.8756, 4.75043),
                );
                path.cubic_bezier_to(
                    view.point(31.0329, 3.68975),
                    view.point(30.6713, 4.2442),
                    view.point(30.9606, 3.90671),
                );
                path.cubic_bezier_to(
                    view.point(32.0456, 3.06299),
                    view.point(31.1776, 3.25584),
                    view.point(31.7563, 2.89425),
                );
                path.cubic_bezier_to(
                    view.point(32.142, 4.12366),
                    view.point(32.1179, 3.11121),
                    view.point(32.142, 3.59333),
                );
                path.cubic_bezier_to(
                    view.point(32.7689, 6.26912),
                    view.point(32.1179, 5.16023),
                    view.point(32.3349, 5.90752),
                );
                path.cubic_bezier_to(
                    view.point(35.0836, 6.6066),
                    view.point(32.9377, 6.41375),
                    view.point(33.6369, 6.51018),
                );
                path.cubic_bezier_to(
                    view.point(36.6509, 7.86012),
                    view.point(37.4948, 6.77534),
                    view.point(37.5671, 6.82356),
                );
                path.cubic_bezier_to(
                    view.point(35.807, 9.42702),
                    view.point(36.1927, 8.36635),
                    view.point(35.9757, 8.77616),
                );
                path.cubic_bezier_to(
                    view.point(35.2042, 10.8493),
                    view.point(35.6623, 9.90915),
                    view.point(35.3971, 10.56),
                );
                path.cubic_bezier_to(
                    view.point(34.8425, 12.5608),
                    view.point(34.8666, 11.3314),
                    view.point(34.8184, 11.5002),
                );
                path.cubic_bezier_to(
                    view.point(35.7346, 14.7545),
                    view.point(34.8425, 14.1518),
                    view.point(35.1077, 14.8027),
                );
                path.cubic_bezier_to(
                    view.point(36.6026, 13.2358),
                    view.point(36.2169, 14.7063),
                    view.point(36.4339, 14.3447),
                );
                path.cubic_bezier_to(
                    view.point(39.7612, 9.35471),
                    view.point(36.7232, 12.5367),
                    view.point(38.1458, 10.777),
                );
                path.cubic_bezier_to(
                    view.point(40.9668, 8.24582),
                    view.point(40.0988, 9.06543),
                    view.point(40.6533, 8.5592),
                );
                path.cubic_bezier_to(
                    view.point(43.7155, 7.64317),
                    view.point(41.7142, 7.54674),
                    view.point(42.1964, 7.42621),
                );
                path.cubic_bezier_to(
                    view.point(45.3068, 7.86012),
                    view.point(44.3182, 7.71549),
                    view.point(45.0416, 7.81191),
                );
                path.cubic_bezier_to(
                    view.point(46.2954, 8.26993),
                    view.point(45.5479, 7.88423),
                    view.point(46.006, 8.07708),
                );
                path.cubic_bezier_to(
                    view.point(47.742, 8.82437),
                    view.point(46.5847, 8.48688),
                    view.point(47.2357, 8.72795),
                );
                path.cubic_bezier_to(
                    view.point(49.2128, 9.40292),
                    view.point(48.2966, 8.9449),
                    view.point(48.8994, 9.18596),
                );
                path.cubic_bezier_to(
                    view.point(50.4425, 9.95736),
                    view.point(49.5263, 9.61987),
                    view.point(50.0808, 9.86094),
                );
                path.cubic_bezier_to(
                    view.point(51.4793, 11.8376),
                    view.point(51.6481, 10.2707),
                    view.point(51.8651, 10.6564),
                );
                path.cubic_bezier_to(
                    view.point(51.1417, 13.0188),
                    view.point(51.3346, 12.2957),
                    view.point(51.19, 12.8019),
                );
                path.cubic_bezier_to(
                    view.point(50.5872, 13.959),
                    view.point(51.0935, 13.2117),
                    view.point(50.8524, 13.6215),
                );
                path.cubic_bezier_to(
                    view.point(50.1049, 14.6581),
                    view.point(50.322, 14.2724),
                    view.point(50.1049, 14.5857),
                );
                path.cubic_bezier_to(
                    view.point(47.7903, 16.8517),
                    view.point(50.1049, 14.9714),
                    view.point(48.2725, 16.683),
                );
                path.cubic_bezier_to(
                    view.point(46.9464, 17.3097),
                    view.point(47.525, 16.924),
                    view.point(47.1393, 17.141),
                );
                path.cubic_bezier_to(
                    view.point(45.4997, 17.9847),
                    view.point(46.4883, 17.7195),
                    view.point(46.1748, 17.8642),
                );
                path.cubic_bezier_to(
                    view.point(44.27, 18.4909),
                    view.point(45.1862, 18.0329),
                    view.point(44.6317, 18.274),
                );
                path.cubic_bezier_to(
                    view.point(42.317, 19.0454),
                    view.point(43.7396, 18.8525),
                    view.point(43.4502, 18.949),
                );
                path.cubic_bezier_to(
                    view.point(39.9782, 19.4311),
                    view.point(41.5937, 19.1177),
                    view.point(40.5328, 19.2865),
                );
                path.cubic_bezier_to(
                    view.point(38.2181, 19.7927),
                    view.point(39.3995, 19.5757),
                    view.point(38.6039, 19.7445),
                );
                path.cubic_bezier_to(
                    view.point(36.1927, 22.7336),
                    view.point(36.675, 19.9855),
                    view.point(36.3133, 20.5159),
                );
                path.cubic_bezier_to(
                    view.point(34.8907, 25.6746),
                    view.point(36.0722, 24.9996),
                    view.point(35.9034, 25.3612),
                );
                path.cubic_bezier_to(
                    view.point(34.0468, 26.229),
                    view.point(34.6978, 25.7228),
                    view.point(34.3121, 25.988),
                );
                path.cubic_bezier_to(
                    view.point(33.3235, 26.687),
                    view.point(33.7575, 26.4942),
                    view.point(33.4441, 26.687),
                );
                path.cubic_bezier_to(
                    view.point(32.576, 27.1692),
                    view.point(33.2271, 26.687),
                    view.point(32.8895, 26.904),
                );
                path.cubic_bezier_to(
                    view.point(31.5152, 27.7718),
                    view.point(32.2626, 27.4343),
                    view.point(31.7804, 27.6995),
                );
                path.cubic_bezier_to(
                    view.point(30.5748, 28.3986),
                    view.point(31.2499, 27.8441),
                    view.point(30.8159, 28.1334),
                );
                path.cubic_bezier_to(
                    view.point(29.755, 28.8807),
                    view.point(30.2614, 28.7361),
                    view.point(30.0203, 28.8807),
                );
                path.cubic_bezier_to(
                    view.point(28.9111, 29.3628),
                    view.point(29.4898, 28.8807),
                    view.point(29.2246, 29.0495),
                );
                path.cubic_bezier_to(
                    view.point(27.8261, 30.086),
                    view.point(28.67, 29.6521),
                    view.point(28.1637, 29.9655),
                );
                path.cubic_bezier_to(
                    view.point(26.8858, 30.7851),
                    view.point(27.368, 30.2307),
                    view.point(27.1028, 30.4235),
                );
                path.cubic_bezier_to(
                    view.point(26.2107, 31.5324),
                    view.point(26.717, 31.0744),
                    view.point(26.4036, 31.4119),
                );
                path.cubic_bezier_to(
                    view.point(26.5482, 32.5931),
                    view.point(25.6079, 31.9422),
                    view.point(25.8249, 32.5931),
                );
                path.cubic_bezier_to(
                    view.point(27.1992, 32.2556),
                    view.point(26.7411, 32.5931),
                    view.point(27.0305, 32.4484),
                );
                path.cubic_bezier_to(
                    view.point(28.2601, 31.8699),
                    view.point(27.4162, 32.0145),
                    view.point(27.7056, 31.9181),
                );
                path.cubic_bezier_to(
                    view.point(29.3934, 31.3636),
                    view.point(28.9111, 31.7976),
                    view.point(29.104, 31.7011),
                );
                path.cubic_bezier_to(
                    view.point(30.4784, 30.8333),
                    view.point(29.6827, 31.0262),
                    view.point(29.9238, 30.9056),
                );
                path.cubic_bezier_to(
                    view.point(31.5634, 30.303),
                    view.point(31.0088, 30.7369),
                    view.point(31.274, 30.6164),
                );
                path.cubic_bezier_to(
                    view.point(32.576, 29.7726),
                    view.point(31.8286, 30.0137),
                    view.point(32.142, 29.845),
                );
                path.cubic_bezier_to(
                    view.point(33.6611, 29.2905),
                    view.point(32.9136, 29.7244),
                    view.point(33.3958, 29.5075),
                );
                path.cubic_bezier_to(
                    view.point(34.8184, 28.6879),
                    view.point(33.9504, 29.0977),
                    view.point(34.4567, 28.8084),
                );
                path.cubic_bezier_to(
                    view.point(35.8552, 28.2781),
                    view.point(35.1801, 28.5673),
                    view.point(35.6382, 28.3745),
                );
                path.cubic_bezier_to(
                    view.point(36.5303, 28.2781),
                    view.point(36.3615, 28.0129),
                    view.point(36.5303, 28.0129),
                );
                path.cubic_bezier_to(
                    view.point(34.4808, 29.845),
                    view.point(36.5303, 28.495),
                    view.point(35.373, 29.3869),
                );
                path.cubic_bezier_to(
                    view.point(33.3958, 30.544),
                    view.point(34.1433, 30.0137),
                    view.point(33.6611, 30.3271),
                );
                path.cubic_bezier_to(
                    view.point(32.3832, 31.0744),
                    view.point(33.1306, 30.7369),
                    view.point(32.6725, 30.9779),
                );
                path.cubic_bezier_to(
                    view.point(31.6598, 31.5565),
                    view.point(32.0938, 31.1708),
                    view.point(31.7804, 31.3878),
                );
                path.cubic_bezier_to(
                    view.point(30.5025, 32.1833),
                    view.point(31.4428, 31.894),
                    view.point(30.9124, 32.1833),
                );
                path.cubic_bezier_to(
                    view.point(29.7309, 32.7136),
                    view.point(30.3337, 32.1833),
                    view.point(29.9961, 32.4243),
                );
                path.cubic_bezier_to(
                    view.point(28.7182, 33.3162),
                    view.point(29.3934, 33.0993),
                    view.point(29.104, 33.268),
                );
                path.cubic_bezier_to(
                    view.point(27.8985, 33.8948),
                    view.point(28.2842, 33.3886),
                    view.point(28.1155, 33.5091),
                );
                path.cubic_bezier_to(
                    view.point(27.0305, 34.5216),
                    view.point(27.6815, 34.2323),
                    view.point(27.4645, 34.401),
                );
                path.cubic_bezier_to(
                    view.point(26.1142, 35.534),
                    view.point(26.3553, 34.6903),
                    view.point(26.1142, 34.9555),
                );
                path.cubic_bezier_to(
                    view.point(26.5482, 36.0644),
                    view.point(26.1142, 35.9197),
                    view.point(26.1866, 35.992),
                );
                path.cubic_bezier_to(
                    view.point(28.3807, 35.6786),
                    view.point(27.4886, 36.2572),
                    view.point(27.8743, 36.1608),
                );
                path.cubic_bezier_to(
                    view.point(29.4175, 35.1965),
                    view.point(28.7665, 35.2688),
                    view.point(28.9594, 35.1965),
                );
                path.cubic_bezier_to(
                    view.point(30.3337, 34.7867),
                    view.point(29.8515, 35.1965),
                    view.point(30.0444, 35.1001),
                );
                path.cubic_bezier_to(
                    view.point(31.8286, 34.0876),
                    view.point(30.7436, 34.3528),
                    view.point(31.2981, 34.0876),
                );
                path.cubic_bezier_to(
                    view.point(32.576, 33.6778),
                    view.point(31.9974, 34.0876),
                    view.point(32.359, 33.9189),
                );
                path.cubic_bezier_to(
                    view.point(33.7093, 33.0752),
                    view.point(32.8172, 33.4609),
                    view.point(33.3235, 33.1716),
                );
                path.cubic_bezier_to(
                    view.point(34.8907, 32.5448),
                    view.point(34.0951, 32.9547),
                    view.point(34.6255, 32.7136),
                );
                path.cubic_bezier_to(
                    view.point(36.9884, 31.7493),
                    view.point(35.6864, 32.0386),
                    view.point(36.8196, 31.5806),
                );
                path.cubic_bezier_to(
                    view.point(34.4326, 33.7743),
                    view.point(37.2295, 31.9904),
                    view.point(37.2536, 31.9663),
                );
                path.cubic_bezier_to(
                    view.point(33.2994, 34.3769),
                    view.point(33.9022, 34.0876),
                    view.point(33.3958, 34.3769),
                );
                path.cubic_bezier_to(
                    view.point(32.6002, 34.859),
                    view.point(33.2029, 34.3769),
                    view.point(32.8895, 34.5939),
                );
                path.cubic_bezier_to(
                    view.point(31.6357, 35.4135),
                    view.point(32.3349, 35.1001),
                    view.point(31.9009, 35.3653),
                );
                path.cubic_bezier_to(
                    view.point(30.6471, 36.0402),
                    view.point(31.3464, 35.4617),
                    view.point(30.9606, 35.7028),
                );
                path.cubic_bezier_to(
                    view.point(29.755, 36.5706),
                    view.point(30.2855, 36.3777),
                    view.point(29.9961, 36.5706),
                );
                path.cubic_bezier_to(
                    view.point(28.9111, 37.0768),
                    view.point(29.4898, 36.5706),
                    view.point(29.2246, 36.7152),
                );
                path.cubic_bezier_to(
                    view.point(27.8743, 37.7277),
                    view.point(28.5977, 37.4143),
                    view.point(28.236, 37.6313),
                );
                path.cubic_bezier_to(
                    view.point(27.0063, 38.4027),
                    view.point(27.4162, 37.8482),
                    view.point(27.2233, 37.9928),
                );
                path.cubic_bezier_to(
                    view.point(26.283, 39.174),
                    view.point(26.8376, 38.6678),
                    view.point(26.5241, 39.0294),
                );
                path.cubic_bezier_to(
                    view.point(26.0178, 40.0901),
                    view.point(25.8249, 39.4392),
                    view.point(25.7284, 39.8008),
                );
                path.cubic_bezier_to(
                    view.point(27.2475, 39.9454),
                    view.point(26.283, 40.3553),
                    view.point(26.8858, 40.2829),
                );
                path.cubic_bezier_to(
                    view.point(28.3084, 39.5115),
                    view.point(27.4886, 39.7285),
                    view.point(27.8502, 39.5839),
                );
                path.cubic_bezier_to(
                    view.point(29.3692, 39.1017),
                    view.point(28.7906, 39.4633),
                    view.point(29.1281, 39.3187),
                );
                path.cubic_bezier_to(
                    view.point(30.6471, 38.475),
                    view.point(29.8274, 38.6437),
                    view.point(30.1408, 38.475),
                );
                path.cubic_bezier_to(
                    view.point(31.5634, 38.017),
                    view.point(30.9124, 38.475),
                    view.point(31.2258, 38.3303),
                );
                path.cubic_bezier_to(
                    view.point(32.6484, 37.4384),
                    view.point(31.8768, 37.7277),
                    view.point(32.3108, 37.5107),
                );
                path.cubic_bezier_to(
                    view.point(34.0709, 36.7875),
                    view.point(33.2753, 37.342),
                    view.point(33.8057, 37.0768),
                );
                path.cubic_bezier_to(
                    view.point(35.156, 36.2813),
                    view.point(34.1674, 36.667),
                    view.point(34.6496, 36.45),
                );
                path.cubic_bezier_to(
                    view.point(36.6026, 35.6063),
                    view.point(35.6864, 36.1367),
                    view.point(36.3133, 35.8233),
                );
                path.cubic_bezier_to(
                    view.point(37.5912, 36.3777),
                    view.point(37.76, 34.6903),
                    view.point(38.6762, 35.4135),
                );
                path.cubic_bezier_to(
                    view.point(37.0125, 37.6554),
                    view.point(37.1813, 36.7393),
                    view.point(37.109, 36.884),
                );
                path.cubic_bezier_to(
                    view.point(36.3374, 40.1383),
                    view.point(36.8679, 39.0776),
                    view.point(36.6991, 39.7285),
                );
                path.cubic_bezier_to(
                    view.point(35.8793, 41.3677),
                    view.point(36.1445, 40.3794),
                    view.point(35.9757, 40.8374),
                );
                path.cubic_bezier_to(
                    view.point(35.3247, 42.9587),
                    view.point(35.7828, 41.8257),
                    view.point(35.5417, 42.5489),
                );
                path.cubic_bezier_to(
                    view.point(34.8184, 44.3087),
                    view.point(35.0836, 43.3685),
                    view.point(34.8666, 43.9712),
                );
                path.cubic_bezier_to(
                    view.point(31.5634, 45.4658),
                    view.point(34.6737, 45.3211),
                    view.point(34.722, 45.297),
                );
                path.cubic_bezier_to(
                    view.point(26.3553, 45.8997),
                    view.point(29.9961, 45.5381),
                    view.point(27.6573, 45.7309),
                );
                path.cubic_bezier_to(
                    view.point(15.023, 46.1648),
                    view.point(23.7513, 46.2131),
                    view.point(17.3859, 46.3577),
                );
                path.close();
                path.move_to(view.point(23.0521, 4.2442));
                path.cubic_bezier_to(
                    view.point(23.1244, 3.8585),
                    view.point(23.028, 4.19598),
                    view.point(23.0521, 4.02724),
                );
                path.cubic_bezier_to(
                    view.point(25.4873, 3.06299),
                    view.point(23.245, 3.4969),
                    view.point(23.7031, 3.35227),
                );
                path.cubic_bezier_to(
                    view.point(27.2475, 2.62908),
                    view.point(26.2348, 2.94246),
                    view.point(27.0305, 2.74961),
                );
                path.cubic_bezier_to(
                    view.point(28.5495, 2.14696),
                    view.point(27.4645, 2.50855),
                    view.point(28.0431, 2.2916),
                );
                path.cubic_bezier_to(
                    view.point(31.3464, 0.700588),
                    view.point(29.7791, 1.76126),
                    view.point(30.6713, 1.30324),
                );
                path.cubic_bezier_to(
                    view.point(32.0938, 0.43542),
                    view.point(31.8768, 0.218464),
                    view.point(31.925, 0.194358),
                );
                path.cubic_bezier_to(
                    view.point(32.142, 1.06218),
                    view.point(32.2144, 0.604163),
                    view.point(32.2385, 0.797013),
                );
                path.cubic_bezier_to(
                    view.point(31.5634, 1.76126),
                    view.point(31.9974, 1.64073),
                    view.point(31.9009, 1.76126),
                );
                path.cubic_bezier_to(
                    view.point(30.5748, 2.07464),
                    view.point(31.3946, 1.76126),
                    view.point(30.9606, 1.9059),
                );
                path.cubic_bezier_to(
                    view.point(26.5965, 3.40048),
                    view.point(29.9479, 2.36391),
                    view.point(28.8147, 2.72551),
                );
                path.cubic_bezier_to(
                    view.point(25.0533, 3.71386),
                    view.point(26.1624, 3.54512),
                    view.point(25.4632, 3.68975),
                );
                path.cubic_bezier_to(
                    view.point(23.8719, 4.07545),
                    view.point(24.6434, 3.76207),
                    view.point(24.113, 3.93082),
                );
                path.cubic_bezier_to(
                    view.point(23.0521, 4.2442),
                    view.point(23.3897, 4.36473),
                    view.point(23.1726, 4.41294),
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
