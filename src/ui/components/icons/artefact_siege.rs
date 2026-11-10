use gpui::{
    FillOptions, FillRule, IntoElement, PathBuilder, PathStyle, Pixels, RenderOnce, Rgba, canvas,
    prelude::*,
};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct ArtefactSiegeIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl ArtefactSiegeIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for ArtefactSiegeIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, 0.0, 0.0, 52.0, 52.0);

                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                ));
                path.move_to(view.point(13.3551, 51.5001));
                path.cubic_bezier_to(
                    view.point(10.7748, 48.2811),
                    view.point(12.3845, 50.7663),
                    view.point(11.6033, 49.7959),
                );
                path.cubic_bezier_to(
                    view.point(9.30712, 48.7071),
                    view.point(9.61486, 46.2219),
                    view.point(9.35446, 46.2929),
                );
                path.cubic_bezier_to(
                    view.point(8.45492, 51.9024),
                    view.point(9.28345, 50.9557),
                    view.point(9.02305, 51.9024),
                );
                path.cubic_bezier_to(
                    view.point(6.06402, 46.0799),
                    view.point(7.93413, 51.9024),
                    view.point(7.9578, 51.9261),
                );
                path.cubic_bezier_to(
                    view.point(5.23549, 43.5473),
                    view.point(5.77996, 45.2041),
                    view.point(5.4012, 44.068),
                );
                path.cubic_bezier_to(
                    view.point(4.24126, 37.3224),
                    view.point(3.79149, 39.4053),
                    view.point(3.72047, 38.9556),
                );
                path.cubic_bezier_to(
                    view.point(4.88041, 34.9792),
                    view.point(4.40697, 36.8254),
                    view.point(4.69103, 35.7603),
                );
                path.cubic_bezier_to(
                    view.point(5.68527, 31.8549),
                    view.point(5.06979, 34.1745),
                    view.point(5.44854, 32.778),
                );
                path.cubic_bezier_to(
                    view.point(6.41911, 28.991),
                    view.point(5.92199, 30.9318),
                    view.point(6.2534, 29.6537),
                );
                path.cubic_bezier_to(
                    view.point(7.08193, 26.5531),
                    view.point(6.58481, 28.3046),
                    view.point(6.89255, 27.2158),
                );
                path.cubic_bezier_to(
                    view.point(7.69741, 24.2572),
                    view.point(7.27131, 25.8903),
                    view.point(7.5317, 24.8489),
                );
                path.cubic_bezier_to(
                    view.point(9.80423, 16.8962),
                    view.point(8.76266, 19.9731),
                    view.point(9.54384, 17.2512),
                );
                path.cubic_bezier_to(
                    view.point(11.2246, 15.9968),
                    view.point(9.9226, 16.7305),
                    view.point(10.5381, 16.3281),
                );
                path.cubic_bezier_to(
                    view.point(16.4088, 13.5352),
                    view.point(12.9053, 15.1447),
                    view.point(15.7933, 13.7955),
                );
                path.cubic_bezier_to(
                    view.point(21.4273, 10.9553),
                    view.point(17.0953, 13.2748),
                    view.point(20.0543, 11.7364),
                );
                path.cubic_bezier_to(
                    view.point(25.996, 4.20965),
                    view.point(25.3095, 8.70674),
                    view.point(25.996, 7.68897),
                );
                path.cubic_bezier_to(
                    view.point(25.2622, 1.01435),
                    view.point(25.996, 2.83685),
                    view.point(25.9014, 2.45815),
                );
                path.line_to(view.point(25.0018, 0.398959));
                path.line_to(view.point(25.2859, 0.16227));
                path.cubic_bezier_to(
                    view.point(27.2743, 0.517303),
                    view.point(25.641, -0.121756),
                    view.point(26.0434, -0.0507496),
                );
                path.cubic_bezier_to(
                    view.point(29.9966, 8.11501),
                    view.point(30.2334, 1.8901),
                    view.point(31.2749, 4.80137),
                );
                path.cubic_bezier_to(
                    view.point(31.2986, 9.01443),
                    view.point(29.4048, 9.60615),
                    view.point(29.7836, 9.86651),
                );
                path.cubic_bezier_to(
                    view.point(34.4944, 6.29251),
                    view.point(32.6243, 8.25703),
                    view.point(33.7132, 7.33394),
                );
                path.cubic_bezier_to(
                    view.point(36.0094, 5.58244),
                    view.point(35.2992, 5.25108),
                    view.point(35.5359, 5.13274),
                );
                path.cubic_bezier_to(
                    view.point(36.1278, 8.96709),
                    view.point(36.6485, 6.2215),
                    view.point(36.6959, 7.68897),
                );
                path.cubic_bezier_to(
                    view.point(31.4407, 13.3458),
                    view.point(35.6306, 10.1032),
                    view.point(34.0919, 11.5233),
                );
                path.cubic_bezier_to(
                    view.point(29.8783, 14.5293),
                    view.point(30.7542, 13.8192),
                    view.point(30.044, 14.3399),
                );
                path.cubic_bezier_to(
                    view.point(29.8309, 15.0027),
                    view.point(29.5942, 14.837),
                    view.point(29.5942, 14.837),
                );
                path.cubic_bezier_to(
                    view.point(36.6012, 12.4701),
                    view.point(30.5411, 15.547),
                    view.point(34.447, 14.0796),
                );
                path.cubic_bezier_to(
                    view.point(38.2819, 11.7364),
                    view.point(37.7611, 11.5943),
                    view.point(38.0215, 11.476),
                );
                path.cubic_bezier_to(
                    view.point(38.2583, 12.6831),
                    view.point(38.424, 11.902),
                    view.point(38.424, 11.9967),
                );
                path.cubic_bezier_to(
                    view.point(28.9077, 19.0737),
                    view.point(37.4297, 15.7127),
                    view.point(34.0209, 18.0323),
                );
                path.cubic_bezier_to(
                    view.point(28.2922, 20.1151),
                    view.point(27.2507, 19.4051),
                    view.point(27.0139, 19.8074),
                );
                path.cubic_bezier_to(
                    view.point(31.9851, 20.0441),
                    view.point(28.8604, 20.2572),
                    view.point(29.3812, 20.2335),
                );
                path.cubic_bezier_to(
                    view.point(33.1451, 21.4879),
                    view.point(33.4765, 19.9258),
                    view.point(33.8552, 20.3755),
                );
                path.cubic_bezier_to(
                    view.point(29.9966, 23.6181),
                    view.point(32.5296, 22.4347),
                    view.point(31.4643, 23.1684),
                );
                path.cubic_bezier_to(
                    view.point(29.1208, 24.8016),
                    view.point(28.3396, 24.1388),
                    view.point(28.1976, 24.3045),
                );
                path.cubic_bezier_to(
                    view.point(28.1265, 26.5531),
                    view.point(30.2334, 25.3933),
                    view.point(29.8309, 26.1034),
                );
                path.cubic_bezier_to(
                    view.point(25.4042, 28.0442),
                    view.point(25.9487, 27.1211),
                    view.point(25.712, 27.2631),
                );
                path.cubic_bezier_to(
                    view.point(22.3268, 29.204),
                    view.point(25.0492, 28.991),
                    view.point(24.481, 29.204),
                );
                path.cubic_bezier_to(
                    view.point(20.8355, 30.6005),
                    view.point(20.149, 29.204),
                    view.point(19.8649, 29.488),
                );
                path.cubic_bezier_to(
                    view.point(19.8886, 31.7129),
                    view.point(21.7114, 31.5709),
                    view.point(21.522, 31.8076),
                );
                path.cubic_bezier_to(
                    view.point(14.9648, 30.3164),
                    view.point(18.6576, 31.6419),
                    view.point(16.6929, 31.0975),
                );
                path.cubic_bezier_to(
                    view.point(13.071, 31.1685),
                    view.point(13.4261, 29.63),
                    view.point(13.1894, 29.7484),
                );
                path.cubic_bezier_to(
                    view.point(11.8637, 32.8963),
                    view.point(12.9763, 32.2573),
                    view.point(12.858, 32.423),
                );
                path.cubic_bezier_to(
                    view.point(10.8221, 34.1034),
                    view.point(11.0825, 33.2987),
                    view.point(10.9168, 33.4881),
                );
                path.cubic_bezier_to(
                    view.point(9.75689, 36.0443),
                    view.point(10.6801, 34.8845),
                    view.point(10.4197, 35.3579),
                );
                path.line_to(view.point(9.09407, 36.707));
                path.line_to(view.point(9.09407, 37.4171));
                path.cubic_bezier_to(
                    view.point(10.5381, 38.4585),
                    view.point(9.11774, 39.1449),
                    view.point(9.82791, 39.642),
                );
                path.cubic_bezier_to(
                    view.point(11.4613, 37.2987),
                    view.point(10.7038, 38.2218),
                    view.point(11.1062, 37.7011),
                );
                path.cubic_bezier_to(
                    view.point(12.3135, 35.9733),
                    view.point(11.9111, 36.8254),
                    view.point(12.1951, 36.3757),
                );
                path.cubic_bezier_to(
                    view.point(13.6628, 34.1744),
                    view.point(12.6449, 34.9792),
                    view.point(12.8106, 34.7662),
                );
                path.cubic_bezier_to(
                    view.point(14.7754, 33.3224),
                    view.point(14.1363, 33.8668),
                    view.point(14.6334, 33.4881),
                );
                path.cubic_bezier_to(
                    view.point(16.1484, 33.1094),
                    view.point(15.1778, 32.8963),
                    view.point(15.6039, 32.8253),
                );
                path.cubic_bezier_to(
                    view.point(17.6634, 34.6478),
                    view.point(16.7402, 33.417),
                    view.point(17.4977, 34.1744),
                );
                path.cubic_bezier_to(
                    view.point(23.6051, 44.7308),
                    view.point(18.6103, 37.1094),
                    view.point(19.6756, 38.9082),
                );
                path.cubic_bezier_to(
                    view.point(25.3569, 48.5888),
                    view.point(25.7593, 47.9497),
                    view.point(25.925, 48.3284),
                );
                path.cubic_bezier_to(
                    view.point(18.3026, 43.9497),
                    view.point(24.0786, 49.1568),
                    view.point(20.1727, 46.6006),
                );
                path.cubic_bezier_to(
                    view.point(16.1721, 42.1035),
                    view.point(17.0243, 42.1745),
                    view.point(16.1721, 41.4171),
                );
                path.cubic_bezier_to(
                    view.point(17.6161, 46.0326),
                    view.point(16.1721, 42.4112),
                    view.point(17.0006, 44.6598),
                );
                path.cubic_bezier_to(
                    view.point(18.9654, 50.8373),
                    view.point(19.2258, 49.6302),
                    view.point(19.5335, 50.7663),
                );
                path.cubic_bezier_to(
                    view.point(13.4261, 44.3521),
                    view.point(18.3026, 50.9557),
                    view.point(15.4382, 47.5947),
                );
                path.cubic_bezier_to(
                    view.point(12.4082, 43.5473),
                    view.point(12.8816, 43.4527),
                    view.point(12.6212, 43.2633),
                );
                path.cubic_bezier_to(
                    view.point(14.0179, 49.5829),
                    view.point(12.0058, 44.0917),
                    view.point(12.7159, 46.719),
                );
                path.cubic_bezier_to(
                    view.point(14.373, 51.9971),
                    view.point(14.9174, 51.5711),
                    view.point(14.9648, 51.9498),
                );
                path.cubic_bezier_to(
                    view.point(13.3551, 51.5001),
                    view.point(14.1126, 52.0208),
                    view.point(13.8759, 51.9024),
                );
                path.close();
                path.move_to(view.point(24.6704, 41.7722));
                path.cubic_bezier_to(
                    view.point(21.3563, 35.2869),
                    view.point(24.2206, 41.3935),
                    view.point(21.3563, 35.7603),
                );
                path.cubic_bezier_to(
                    view.point(22.895, 33.9851),
                    view.point(21.3563, 34.7898),
                    view.point(21.7824, 34.4348),
                );
                path.cubic_bezier_to(
                    view.point(31.6774, 26.5057),
                    view.point(25.4279, 32.9673),
                    view.point(28.6237, 30.2454),
                );
                path.cubic_bezier_to(
                    view.point(33.9499, 24.0678),
                    view.point(32.9557, 24.9199),
                    view.point(33.5475, 24.2809),
                );
                path.cubic_bezier_to(
                    view.point(36.0331, 23.7365),
                    view.point(34.1156, 23.9732),
                    view.point(35.0625, 23.8312),
                );
                path.cubic_bezier_to(
                    view.point(42.4482, 20.8015),
                    view.point(39.7733, 23.3578),
                    view.point(40.9569, 22.8134),
                );
                path.cubic_bezier_to(
                    view.point(43.6555, 18.34),
                    view.point(43.1347, 19.8548),
                    view.point(43.3951, 19.3341),
                );
                path.cubic_bezier_to(
                    view.point(45.6677, 18.2926),
                    view.point(44.0816, 16.8015),
                    view.point(45.0285, 16.7778),
                );
                path.cubic_bezier_to(
                    view.point(44.058, 23.9495),
                    view.point(46.4015, 19.9495),
                    view.point(45.8097, 22.0323),
                );
                path.cubic_bezier_to(
                    view.point(43.3715, 25.0856),
                    view.point(43.3478, 24.7306),
                    view.point(43.2768, 24.8726),
                );
                path.cubic_bezier_to(
                    view.point(47.5851, 24.2572),
                    view.point(43.6555, 25.5826),
                    view.point(46.0701, 25.1093),
                );
                path.cubic_bezier_to(
                    view.point(48.9108, 24.6122),
                    view.point(48.532, 23.7365),
                    view.point(48.9108, 23.8312),
                );
                path.cubic_bezier_to(
                    view.point(43.7502, 28.3992),
                    view.point(48.9108, 26.2217),
                    view.point(46.8513, 27.7365),
                );
                path.cubic_bezier_to(
                    view.point(42.0932, 29.6064),
                    view.point(42.0932, 28.7543),
                    view.point(41.596, 29.1093),
                );
                path.cubic_bezier_to(
                    view.point(46.2358, 29.63),
                    view.point(42.4956, 30.0087),
                    view.point(44.7208, 30.0087),
                );
                path.cubic_bezier_to(
                    view.point(47.2774, 30.6005),
                    view.point(47.9875, 29.1803),
                    view.point(48.3663, 29.5354),
                );
                path.cubic_bezier_to(
                    view.point(43.7976, 32.2809),
                    view.point(46.5435, 31.3342),
                    view.point(45.3599, 31.9022),
                );
                path.line_to(view.point(42.5429, 32.5886));
                path.line_to(view.point(40.7912, 32.3993));
                path.cubic_bezier_to(
                    view.point(38.637, 32.1389),
                    view.point(39.8443, 32.2809),
                    view.point(38.8737, 32.1626),
                );
                path.cubic_bezier_to(
                    view.point(38.0215, 32.2573),
                    view.point(38.4003, 32.0916),
                    view.point(38.1636, 32.1389),
                );
                path.cubic_bezier_to(
                    view.point(38.0215, 32.6596),
                    view.point(37.7848, 32.423),
                    view.point(37.7848, 32.423),
                );
                path.cubic_bezier_to(
                    view.point(40.1994, 33.7484),
                    view.point(38.3293, 33.0147),
                    view.point(39.3708, 33.5354),
                );
                path.cubic_bezier_to(
                    view.point(40.8149, 35.2159),
                    view.point(41.3356, 34.0324),
                    view.point(41.5487, 34.5768),
                );
                path.cubic_bezier_to(
                    view.point(38.1162, 35.8549),
                    view.point(40.3177, 35.6656),
                    view.point(39.7733, 35.8076),
                );
                path.cubic_bezier_to(
                    view.point(36.6722, 36.4703),
                    view.point(36.4355, 35.9259),
                    view.point(36.2935, 35.9733),
                );
                path.cubic_bezier_to(
                    view.point(34.9205, 37.5118),
                    view.point(37.2877, 37.2987),
                    view.point(36.5775, 37.7248),
                );
                path.cubic_bezier_to(
                    view.point(33.1451, 37.7721),
                    view.point(33.6422, 37.3461),
                    view.point(33.6185, 37.3461),
                );
                path.cubic_bezier_to(
                    view.point(30.2807, 38.5295),
                    view.point(32.4586, 38.3875),
                    view.point(32.3402, 38.4112),
                );
                path.cubic_bezier_to(
                    view.point(25.6883, 40.7307),
                    view.point(25.9487, 38.7899),
                    view.point(25.9724, 38.7899),
                );
                path.cubic_bezier_to(
                    view.point(24.6704, 41.7722),
                    view.point(25.4753, 42.1982),
                    view.point(25.3095, 42.3402),
                );
                path.close();
                path.move_to(view.point(15.5566, 25.0619));
                path.cubic_bezier_to(
                    view.point(16.8112, 24.3282),
                    view.point(15.8643, 24.8252),
                    view.point(16.4325, 24.4939),
                );
                path.cubic_bezier_to(
                    view.point(18.6576, 22.198),
                    view.point(18.1132, 23.7365),
                    view.point(18.5156, 23.2868),
                );
                path.cubic_bezier_to(
                    view.point(20.0306, 20.5885),
                    view.point(18.776, 21.4169),
                    view.point(19.0127, 21.1329),
                );
                path.cubic_bezier_to(
                    view.point(21.0959, 18.0796),
                    view.point(21.3089, 19.9258),
                    view.point(21.7587, 18.8607),
                );
                path.cubic_bezier_to(
                    view.point(20.149, 18.3636),
                    view.point(20.7645, 17.7009),
                    view.point(20.5514, 17.7482),
                );
                path.cubic_bezier_to(
                    view.point(17.5924, 19.2867),
                    view.point(19.6045, 19.2394),
                    view.point(18.918, 19.4761),
                );
                path.cubic_bezier_to(
                    view.point(16.4561, 19.3577),
                    view.point(16.8586, 19.1921),
                    view.point(16.6929, 19.192),
                );
                path.cubic_bezier_to(
                    view.point(16.5271, 20.6359),
                    view.point(16.101, 19.5944),
                    view.point(16.101, 19.7838),
                );
                path.cubic_bezier_to(
                    view.point(15.2725, 22.9791),
                    view.point(17.2373, 22.0323),
                    view.point(16.7165, 22.9791),
                );
                path.cubic_bezier_to(
                    view.point(14.1126, 24.5886),
                    view.point(14.2309, 22.9791),
                    view.point(13.9232, 23.4051),
                );
                path.cubic_bezier_to(
                    view.point(14.2783, 25.3933),
                    view.point(14.1599, 24.9673),
                    view.point(14.2309, 25.346),
                );
                path.cubic_bezier_to(
                    view.point(15.5566, 25.0619),
                    view.point(14.444, 25.6537),
                    view.point(15.0121, 25.5116),
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
