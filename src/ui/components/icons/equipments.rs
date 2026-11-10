use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct EquipmentsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl EquipmentsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for EquipmentsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(26.0, 13.5578));
                path.cubic_bezier_to(
                    view.point(19.0857, 15.6547),
                    view.point(23.5385, 13.5578),
                    view.point(21.1323, 14.2875),
                );
                path.cubic_bezier_to(
                    view.point(14.5019, 21.2386),
                    view.point(17.039, 17.0218),
                    view.point(15.4439, 18.965),
                );
                path.cubic_bezier_to(
                    view.point(13.7937, 28.4274),
                    view.point(13.5599, 23.5121),
                    view.point(13.3135, 26.0138),
                );
                path.cubic_bezier_to(
                    view.point(17.1997, 34.798),
                    view.point(14.2739, 30.8409),
                    view.point(15.4592, 33.0579),
                );
                path.cubic_bezier_to(
                    view.point(23.572, 38.2032),
                    view.point(18.9403, 36.5381),
                    view.point(21.1578, 37.7231),
                );
                path.cubic_bezier_to(
                    view.point(30.7627, 37.4951),
                    view.point(25.9862, 38.6832),
                    view.point(28.4886, 38.4368),
                );
                path.cubic_bezier_to(
                    view.point(36.348, 32.9125),
                    view.point(33.0368, 36.5534),
                    view.point(34.9805, 34.9586),
                );
                path.cubic_bezier_to(
                    view.point(38.4454, 26.0),
                    view.point(37.7155, 30.8664),
                    view.point(38.4454, 28.4608),
                );
                path.cubic_bezier_to(
                    view.point(34.7961, 17.2062),
                    view.point(38.4416, 22.7013),
                    view.point(37.1292, 19.5388),
                );
                path.cubic_bezier_to(
                    view.point(26.0, 13.5578),
                    view.point(32.4629, 14.8737),
                    view.point(29.2996, 13.5616),
                );
                path.close();
                path.move_to(view.point(26.0, 32.6997));
                path.cubic_bezier_to(
                    view.point(22.2769, 31.5706),
                    view.point(24.6746, 32.6997),
                    view.point(23.3789, 32.3067),
                );
                path.cubic_bezier_to(
                    view.point(19.8087, 28.5638),
                    view.point(21.1749, 30.8344),
                    view.point(20.3159, 29.788),
                );
                path.cubic_bezier_to(
                    view.point(19.4274, 24.693),
                    view.point(19.3015, 27.3396),
                    view.point(19.1688, 25.9926),
                );
                path.cubic_bezier_to(
                    view.point(21.2614, 21.2626),
                    view.point(19.6859, 23.3934),
                    view.point(20.3242, 22.1996),
                );
                path.cubic_bezier_to(
                    view.point(24.6926, 19.4291),
                    view.point(22.1986, 20.3257),
                    view.point(23.3927, 19.6876),
                );
                path.cubic_bezier_to(
                    view.point(28.5645, 19.8103),
                    view.point(25.9926, 19.1706),
                    view.point(27.34, 19.3032),
                );
                path.cubic_bezier_to(
                    view.point(31.572, 22.2779),
                    view.point(29.789, 20.3174),
                    view.point(30.8356, 21.1761),
                );
                path.cubic_bezier_to(
                    view.point(32.7014, 26.0),
                    view.point(32.3084, 23.3796),
                    view.point(32.7014, 24.6749),
                );
                path.cubic_bezier_to(
                    view.point(30.7386, 30.7374),
                    view.point(32.7014, 27.7769),
                    view.point(31.9954, 29.4809),
                );
                path.cubic_bezier_to(
                    view.point(26.0, 32.6997),
                    view.point(29.4818, 31.9938),
                    view.point(27.7773, 32.6997),
                );
                path.close();
                path.move_to(view.point(48.0189, 26.189));
                path.line_to(view.point(48.0189, 25.811));
                path.line_to(view.point(51.3696, 21.6213));
                path.cubic_bezier_to(
                    view.point(51.9384, 20.4179),
                    view.point(51.6508, 21.2702),
                    view.point(51.8457, 20.858),
                );
                path.cubic_bezier_to(
                    view.point(51.9033, 19.0874),
                    view.point(52.0311, 19.9778),
                    view.point(52.0191, 19.522),
                );
                path.cubic_bezier_to(
                    view.point(49.2155, 12.5744),
                    view.point(51.2985, 16.8074),
                    view.point(50.3948, 14.6174),
                );
                path.cubic_bezier_to(
                    view.point(48.2938, 11.6093),
                    view.point(48.9883, 12.1848),
                    view.point(48.6726, 11.8542),
                );
                path.cubic_bezier_to(
                    view.point(47.0352, 11.165),
                    view.point(47.915, 11.3645),
                    view.point(47.4838, 11.2123),
                );
                path.line_to(view.point(41.7028, 10.5669));
                path.line_to(view.point(41.4371, 10.3013));
                path.line_to(view.point(40.8388, 4.96785));
                path.cubic_bezier_to(
                    view.point(40.3939, 3.71096),
                    view.point(40.7912, 4.51979),
                    view.point(40.6388, 4.08922),
                );
                path.cubic_bezier_to(
                    view.point(39.4291, 2.79047),
                    view.point(40.149, 3.3327),
                    view.point(39.8185, 3.01737),
                );
                path.cubic_bezier_to(
                    view.point(32.912, 0.0938527),
                    view.point(37.3854, 1.60779),
                    view.point(35.194, 0.701025),
                );
                path.cubic_bezier_to(
                    view.point(31.5812, 0.063891),
                    view.point(32.4769, -0.0201887),
                    view.point(32.021, -0.0304516),
                );
                path.cubic_bezier_to(
                    view.point(30.3798, 0.637004),
                    view.point(31.1414, 0.158234),
                    view.point(30.7299, 0.354544),
                );
                path.line_to(view.point(26.1891, 3.98683));
                path.line_to(view.point(25.8109, 3.98683));
                path.line_to(view.point(21.6202, 0.637004));
                path.cubic_bezier_to(
                    view.point(20.4165, 0.0683457),
                    view.point(21.269, 0.355813),
                    view.point(20.8567, 0.161027),
                );
                path.cubic_bezier_to(
                    view.point(19.0856, 0.103424),
                    view.point(19.9762, -0.0243356),
                    view.point(19.5203, -0.0123199),
                );
                path.cubic_bezier_to(
                    view.point(12.5709, 2.80243),
                    view.point(16.8044, 0.711817),
                    view.point(14.6138, 1.61936),
                );
                path.cubic_bezier_to(
                    view.point(11.6081, 3.71872),
                    view.point(12.1827, 3.02839),
                    view.point(11.853, 3.34223),
                );
                path.cubic_bezier_to(
                    view.point(11.1612, 4.97025),
                    view.point(11.3633, 4.09521),
                    view.point(11.2102, 4.52387),
                );
                path.line_to(view.point(10.5629, 10.3013));
                path.line_to(view.point(10.2972, 10.5669));
                path.line_to(view.point(4.96241, 11.165));
                path.cubic_bezier_to(
                    view.point(3.70519, 11.6098),
                    view.point(4.51423, 11.2126),
                    view.point(4.08355, 11.365),
                );
                path.cubic_bezier_to(
                    view.point(2.78446, 12.5744),
                    view.point(3.32683, 11.8547),
                    view.point(3.01142, 12.1851),
                );
                path.cubic_bezier_to(
                    view.point(0.0967209, 19.0898),
                    view.point(1.6048, 14.618),
                    view.point(0.701028, 16.8089),
                );
                path.cubic_bezier_to(
                    view.point(0.0622244, 20.419),
                    view.point(-0.0186576, 19.5241),
                    view.point(-0.0304734, 19.9794),
                );
                path.cubic_bezier_to(
                    view.point(0.630439, 21.6213),
                    view.point(0.154922, 20.8587),
                    view.point(0.349545, 21.2705),
                );
                path.line_to(view.point(3.98114, 25.811));
                path.line_to(view.point(3.98114, 26.189));
                path.line_to(view.point(0.630439, 30.3787));
                path.cubic_bezier_to(
                    view.point(0.0616337, 31.5821),
                    view.point(0.349175, 30.7298),
                    view.point(0.154339, 31.142),
                );
                path.cubic_bezier_to(
                    view.point(0.0967209, 32.9126),
                    view.point(-0.0310716, 32.0222),
                    view.point(-0.0190528, 32.478),
                );
                path.cubic_bezier_to(
                    view.point(2.79882, 39.4256),
                    view.point(0.706027, 35.1933),
                    view.point(1.61462, 37.3833),
                );
                path.cubic_bezier_to(
                    view.point(3.7143, 40.3876),
                    view.point(3.0246, 39.8134),
                    view.point(3.33815, 40.1429),
                );
                path.cubic_bezier_to(
                    view.point(4.9648, 40.835),
                    view.point(4.09045, 40.6324),
                    view.point(4.51873, 40.7856),
                );
                path.line_to(view.point(10.2972, 41.4284));
                path.line_to(view.point(10.5629, 41.694));
                path.line_to(view.point(11.1612, 47.0321));
                path.cubic_bezier_to(
                    view.point(11.6061, 48.289),
                    view.point(11.2088, 47.4802),
                    view.point(11.3612, 47.9108),
                );
                path.cubic_bezier_to(
                    view.point(12.5709, 49.2095),
                    view.point(11.851, 48.6673),
                    view.point(12.1815, 48.9826),
                );
                path.cubic_bezier_to(
                    view.point(19.088, 51.9062),
                    view.point(14.6146, 50.3922),
                    view.point(16.806, 51.299),
                );
                path.cubic_bezier_to(
                    view.point(20.4188, 51.9361),
                    view.point(19.5231, 52.0202),
                    view.point(19.979, 52.0305),
                );
                path.cubic_bezier_to(
                    view.point(21.6202, 51.363),
                    view.point(20.8586, 51.8418),
                    view.point(21.2701, 51.6455),
                );
                path.line_to(view.point(25.8109, 48.0132));
                path.line_to(view.point(26.1891, 48.0132));
                path.line_to(view.point(30.3798, 51.363));
                path.cubic_bezier_to(
                    view.point(31.5835, 51.9317),
                    view.point(30.731, 51.6442),
                    view.point(31.1433, 51.839),
                );
                path.cubic_bezier_to(
                    view.point(32.9144, 51.8966),
                    view.point(32.0238, 52.0243),
                    view.point(32.4797, 52.0123),
                );
                path.cubic_bezier_to(
                    view.point(39.4291, 49.2095),
                    view.point(35.195, 51.292),
                    view.point(37.3855, 50.3884),
                );
                path.cubic_bezier_to(
                    view.point(40.3945, 48.294),
                    view.point(39.8182, 48.9841),
                    view.point(40.1489, 48.6705),
                );
                path.cubic_bezier_to(
                    view.point(40.8436, 47.0417),
                    view.point(40.6402, 47.9175),
                    view.point(40.794, 47.4885),
                );
                path.line_to(view.point(41.4371, 41.7107));
                path.line_to(view.point(41.7028, 41.4451));
                path.line_to(view.point(47.0376, 40.835));
                path.cubic_bezier_to(
                    view.point(48.2902, 40.386),
                    view.point(47.4845, 40.7854),
                    view.point(47.9136, 40.6316),
                );
                path.cubic_bezier_to(
                    view.point(49.206, 39.4209),
                    view.point(48.6668, 40.1404),
                    view.point(48.9805, 39.8098),
                );
                path.cubic_bezier_to(
                    view.point(51.9033, 32.9054),
                    view.point(50.389, 37.3777),
                    view.point(51.2959, 35.1868),
                );
                path.cubic_bezier_to(
                    view.point(51.9366, 31.5787),
                    view.point(52.0179, 32.4719),
                    view.point(52.0293, 32.0175),
                );
                path.cubic_bezier_to(
                    view.point(51.3696, 30.3787),
                    view.point(51.8439, 31.1399),
                    view.point(51.6497, 30.7289),
                );
                path.line_to(view.point(48.0189, 26.189));
                path.close();
                path.move_to(view.point(42.2461, 25.019));
                path.cubic_bezier_to(
                    view.point(42.2461, 26.981),
                    view.point(42.2844, 25.6724),
                    view.point(42.2844, 26.3276),
                );
                path.cubic_bezier_to(
                    view.point(42.8708, 28.9479),
                    view.point(42.2033, 27.6914),
                    view.point(42.4259, 28.3923),
                );
                path.line_to(view.point(45.9438, 32.7882));
                path.cubic_bezier_to(
                    view.point(44.9075, 35.2982),
                    view.point(45.6537, 33.6466),
                    view.point(45.3075, 34.4851),
                );
                path.line_to(view.point(40.0179, 35.8413));
                path.cubic_bezier_to(
                    view.point(38.187, 36.7984),
                    view.point(39.3099, 35.9222),
                    view.point(38.6574, 36.2634),
                );
                path.cubic_bezier_to(
                    view.point(36.7988, 38.1862),
                    view.point(37.7522, 37.2881),
                    view.point(37.2887, 37.7515),
                );
                path.cubic_bezier_to(
                    view.point(35.8415, 40.0166),
                    view.point(36.2636, 38.6565),
                    view.point(35.9224, 39.3089),
                );
                path.line_to(view.point(35.3006, 44.9026));
                path.cubic_bezier_to(
                    view.point(32.7899, 45.9435),
                    view.point(34.4874, 45.3042),
                    view.point(33.6487, 45.6519),
                );
                path.line_to(view.point(28.9486, 42.8688));
                path.cubic_bezier_to(
                    view.point(27.1536, 42.2419),
                    view.point(28.4388, 42.4621),
                    view.point(27.8058, 42.241),
                );
                path.line_to(view.point(26.9813, 42.2419));
                path.cubic_bezier_to(
                    view.point(25.0187, 42.2419),
                    view.point(26.3277, 42.2802),
                    view.point(25.6723, 42.2802),
                );
                path.cubic_bezier_to(
                    view.point(23.0514, 42.864),
                    view.point(24.3087, 42.1998),
                    view.point(23.6081, 42.4214),
                );
                path.line_to(view.point(19.2101, 45.9387));
                path.cubic_bezier_to(
                    view.point(16.6994, 44.9026),
                    view.point(18.3514, 45.6486),
                    view.point(17.5127, 45.3025),
                );
                path.line_to(view.point(16.1561, 40.0143));
                path.cubic_bezier_to(
                    view.point(15.1988, 38.1838),
                    view.point(16.0752, 39.3065),
                    view.point(15.734, 38.6541),
                );
                path.cubic_bezier_to(
                    view.point(13.8106, 36.796),
                    view.point(14.709, 37.7491),
                    view.point(14.2454, 37.2857),
                );
                path.cubic_bezier_to(
                    view.point(11.9797, 35.8389),
                    view.point(13.3402, 36.261),
                    view.point(12.6877, 35.9198),
                );
                path.line_to(view.point(7.0925, 35.2982));
                path.cubic_bezier_to(
                    view.point(6.05139, 32.7882),
                    view.point(6.69084, 34.4852),
                    view.point(6.34306, 33.6467),
                );
                path.line_to(view.point(9.12446, 28.9479));
                path.cubic_bezier_to(
                    view.point(9.74912, 26.981),
                    view.point(9.56928, 28.3923),
                    view.point(9.79189, 27.6914),
                );
                path.cubic_bezier_to(
                    view.point(9.74912, 25.019),
                    view.point(9.71077, 26.3276),
                    view.point(9.71077, 25.6724),
                );
                path.cubic_bezier_to(
                    view.point(9.12446, 23.0521),
                    view.point(9.79189, 24.3086),
                    view.point(9.56928, 23.6077),
                );
                path.line_to(view.point(6.05617, 19.2118));
                path.cubic_bezier_to(
                    view.point(7.0925, 16.7018),
                    view.point(6.34629, 18.3534),
                    view.point(6.69247, 17.5149),
                );
                path.line_to(view.point(11.9821, 16.1587));
                path.cubic_bezier_to(
                    view.point(13.813, 15.2016),
                    view.point(12.6901, 16.0778),
                    view.point(13.3426, 15.7366),
                );
                path.cubic_bezier_to(
                    view.point(15.2012, 13.8138),
                    view.point(14.2478, 14.7119),
                    view.point(14.7113, 14.2485),
                );
                path.cubic_bezier_to(
                    view.point(16.1585, 11.9834),
                    view.point(15.7364, 13.3435),
                    view.point(16.0776, 12.6911),
                );
                path.line_to(view.point(16.6994, 7.09739));
                path.cubic_bezier_to(
                    view.point(19.2101, 6.05655),
                    view.point(17.5126, 6.69584),
                    view.point(18.3513, 6.34814),
                );
                path.line_to(view.point(23.0514, 9.13121));
                path.cubic_bezier_to(
                    view.point(25.0187, 9.75333),
                    view.point(23.6081, 9.57385),
                    view.point(24.3087, 9.79538),
                );
                path.cubic_bezier_to(
                    view.point(26.9813, 9.75333),
                    view.point(25.6723, 9.71499),
                    view.point(26.3277, 9.71499),
                );
                path.cubic_bezier_to(
                    view.point(28.9486, 9.13121),
                    view.point(27.6915, 9.79676),
                    view.point(28.3926, 9.57507),
                );
                path.line_to(view.point(32.7899, 6.05655));
                path.cubic_bezier_to(
                    view.point(35.3006, 7.09739),
                    view.point(33.6487, 6.34814),
                    view.point(34.4874, 6.69584),
                );
                path.line_to(view.point(35.8439, 11.9857));
                path.cubic_bezier_to(
                    view.point(36.8012, 13.8162),
                    view.point(35.9248, 12.6935),
                    view.point(36.266, 13.3459),
                );
                path.cubic_bezier_to(
                    view.point(38.1894, 15.204),
                    view.point(37.291, 14.2509),
                    view.point(37.7546, 14.7143),
                );
                path.cubic_bezier_to(
                    view.point(40.0203, 16.1611),
                    view.point(38.6598, 15.739),
                    view.point(39.3123, 16.0802),
                );
                path.line_to(view.point(44.9075, 16.7018));
                path.cubic_bezier_to(
                    view.point(45.9486, 19.2118),
                    view.point(45.3092, 17.5148),
                    view.point(45.6569, 18.3533),
                );
                path.line_to(view.point(42.8755, 23.0521));
                path.cubic_bezier_to(
                    view.point(42.2461, 25.019),
                    view.point(42.429, 23.607),
                    view.point(42.2047, 24.308),
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
