use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct SettingsIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl SettingsIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for SettingsIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, -713.0, -54.952, 20.0, 20.019);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(-702.9996338, -49.73262405));
                path.cubic_bezier_to(
                    view.point(-705.6588135, -48.92546082),
                    view.point(-703.9464111, -49.73262405),
                    view.point(-704.8717041, -49.45183945),
                );
                path.cubic_bezier_to(
                    view.point(-707.4219971, -46.77570724),
                    view.point(-706.4464111, -48.39907837),
                    view.point(-707.0596924, -47.65105057),
                );
                path.cubic_bezier_to(
                    view.point(-707.694458, -44.00817871),
                    view.point(-707.7843018, -45.90036011),
                    view.point(-707.8790283, -44.93728638),
                );
                path.cubic_bezier_to(
                    view.point(-706.3843994, -41.5556488),
                    view.point(-707.5098877, -43.07907104),
                    view.point(-707.053833, -42.22547531),
                );
                path.cubic_bezier_to(
                    view.point(-703.9337158, -40.24482727),
                    view.point(-705.7149658, -40.88581848),
                    view.point(-704.8619385, -40.42957306),
                );
                path.cubic_bezier_to(
                    view.point(-701.1680908, -40.51730347),
                    view.point(-703.0050049, -40.05983734),
                    view.point(-702.0426025, -40.15465546),
                );
                path.cubic_bezier_to(
                    view.point(-699.0196533, -42.28143692),
                    view.point(-700.2935791, -40.87971115),
                    view.point(-699.5455322, -41.49382019),
                );
                path.cubic_bezier_to(
                    view.point(-698.2130127, -44.94266129),
                    view.point(-698.4937744, -43.06929779),
                    view.point(-698.2130127, -43.99547195),
                );
                path.cubic_bezier_to(
                    view.point(-699.6168213, -48.32821274),
                    view.point(-698.2144775, -46.21266937),
                    view.point(-698.7193604, -47.43013763),
                );
                path.cubic_bezier_to(
                    view.point(-702.9996338, -49.73262405),
                    view.point(-700.5137939, -49.22603989),
                    view.point(-701.7305908, -49.73115921),
                );
                path.close();
                path.move_to(view.point(-702.9996338, -42.36354446));
                path.cubic_bezier_to(
                    view.point(-704.4317627, -42.7980423),
                    view.point(-703.5093994, -42.36354446),
                    view.point(-704.0079346, -42.51481247),
                );
                path.cubic_bezier_to(
                    view.point(-705.3809814, -43.95563889),
                    view.point(-704.8555908, -43.08151627),
                    view.point(-705.1861572, -43.48424149),
                );
                path.cubic_bezier_to(
                    view.point(-705.5279541, -45.44582748),
                    view.point(-705.5762939, -44.42703629),
                    view.point(-705.6270752, -44.94559479),
                );
                path.cubic_bezier_to(
                    view.point(-704.8223877, -46.76642227),
                    view.point(-705.4283447, -45.94630432),
                    view.point(-705.1827393, -46.40572739),
                );
                path.cubic_bezier_to(
                    view.point(-703.5025635, -47.47241592),
                    view.point(-704.4615479, -47.12711716),
                    view.point(-704.0025635, -47.37295532),
                );
                path.cubic_bezier_to(
                    view.point(-702.0133057, -47.32554626),
                    view.point(-703.0025635, -47.57187653),
                    view.point(-702.4844971, -47.52080154),
                );
                path.cubic_bezier_to(
                    view.point(-700.8565674, -46.37566757),
                    view.point(-701.5426025, -47.13029099),
                    view.point(-701.1397705, -46.79990005),
                );
                path.cubic_bezier_to(
                    view.point(-700.4224854, -44.94266129),
                    view.point(-700.5733643, -45.95143509),
                    view.point(-700.4224854, -45.45291519),
                );
                path.cubic_bezier_to(
                    view.point(-701.1773682, -43.11890411),
                    view.point(-700.4224854, -44.25866318),
                    view.point(-700.6939697, -43.60251999),
                );
                path.cubic_bezier_to(
                    view.point(-702.9996338, -42.36354446),
                    view.point(-701.6607666, -42.6350441),
                    view.point(-702.31604, -42.36354446),
                );
                path.close();
                path.move_to(view.point(-694.5308838, -44.86983871));
                path.line_to(view.point(-694.5308838, -45.01548386));
                path.line_to(view.point(-693.2423096, -46.6283493));
                path.cubic_bezier_to(
                    view.point(-693.0235596, -47.09168243),
                    view.point(-693.1339111, -46.76348877),
                    view.point(-693.0592041, -46.92233276),
                );
                path.cubic_bezier_to(
                    view.point(-693.0367432, -47.60388947),
                    view.point(-692.987915, -47.2610321),
                    view.point(-692.9923096, -47.43649292),
                );
                path.cubic_bezier_to(
                    view.point(-694.0709229, -50.11116028),
                    view.point(-693.2696533, -48.48167801),
                    view.point(-693.6168213, -49.32476425),
                );
                path.cubic_bezier_to(
                    view.point(-694.4249268, -50.48284912),
                    view.point(-694.1583252, -50.26120377),
                    view.point(-694.2794189, -50.3885231),
                );
                path.cubic_bezier_to(
                    view.point(-694.9093018, -50.65391159),
                    view.point(-694.5709229, -50.57717896),
                    view.point(-694.7369385, -50.63558578),
                );
                path.line_to(view.point(-696.960083, -50.88411331));
                path.line_to(view.point(-697.0626221, -50.9865036));
                path.line_to(view.point(-697.2926025, -53.03973389));
                path.cubic_bezier_to(
                    view.point(-697.463501, -53.5235939),
                    view.point(-697.3106689, -53.2122612),
                    view.point(-697.3692627, -53.37794495),
                );
                path.cubic_bezier_to(
                    view.point(-697.8345947, -53.8779335),
                    view.point(-697.5577393, -53.66923904),
                    view.point(-697.6851807, -53.79069138),
                );
                path.cubic_bezier_to(
                    view.point(-700.3414307, -54.91603088),
                    view.point(-698.6207275, -54.33320236),
                    view.point(-699.463501, -54.68241119),
                );
                path.cubic_bezier_to(
                    view.point(-700.8531494, -54.92751694),
                    view.point(-700.5084229, -54.96001816),
                    view.point(-700.6842041, -54.96392822),
                );
                path.cubic_bezier_to(
                    view.point(-701.3150635, -54.70709229),
                    view.point(-701.0220947, -54.89134979),
                    view.point(-701.1802979, -54.81583786),
                );
                path.line_to(view.point(-702.9268799, -53.41728973));
                path.line_to(view.point(-703.0723877, -53.41728973));
                path.line_to(view.point(-704.6842041, -54.70709229));
                path.cubic_bezier_to(
                    view.point(-705.1470947, -54.92580795),
                    view.point(-704.819458, -54.81534958),
                    view.point(-704.9776611, -54.89012909),
                );
                path.cubic_bezier_to(
                    view.point(-705.6593018, -54.91236496),
                    view.point(-705.3165283, -54.96173096),
                    view.point(-705.4918213, -54.95684052),
                );
                path.cubic_bezier_to(
                    view.point(-708.1646729, -53.87329102),
                    view.point(-706.5367432, -54.67825699),
                    view.point(-707.3790283, -54.32880402),
                );
                path.cubic_bezier_to(
                    view.point(-708.5352783, -53.5206604),
                    view.point(-708.3140869, -53.78629303),
                    view.point(-708.44104, -53.66557312),
                );
                path.cubic_bezier_to(
                    view.point(-708.7071533, -53.03875732),
                    view.point(-708.6290283, -53.37574768),
                    view.point(-708.6881104, -53.21055222),
                );
                path.line_to(view.point(-708.9371338, -50.9865036));
                path.line_to(view.point(-709.0391846, -50.88411331));
                path.line_to(view.point(-711.0909424, -50.65391159));
                path.cubic_bezier_to(
                    view.point(-711.5748291, -50.48260498),
                    view.point(-711.2633057, -50.63558578),
                    view.point(-711.4293213, -50.57693481),
                );
                path.cubic_bezier_to(
                    view.point(-711.928833, -50.11116028),
                    view.point(-711.7203369, -50.38827896),
                    view.point(-711.8414307, -50.26120377),
                );
                path.cubic_bezier_to(
                    view.point(-712.9625244, -47.60290909),
                    view.point(-712.3824463, -49.32452011),
                    view.point(-712.7301025, -48.48118973),
                );
                path.cubic_bezier_to(
                    view.point(-712.975708, -47.09119415),
                    view.point(-713.006958, -47.4357605),
                    view.point(-713.0113525, -47.26054382),
                );
                path.cubic_bezier_to(
                    view.point(-712.7574463, -46.6283493),
                    view.point(-712.9400635, -46.92208481),
                    view.point(-712.8653564, -46.76348877),
                );
                path.line_to(view.point(-711.4683838, -45.01548386));
                path.line_to(view.point(-711.4683838, -44.86983871));
                path.line_to(view.point(-712.7574463, -43.25697327));
                path.cubic_bezier_to(
                    view.point(-712.9761963, -42.79364395),
                    view.point(-712.8653564, -43.12183762),
                    view.point(-712.9405518, -42.96299362),
                );
                path.cubic_bezier_to(
                    view.point(-712.9625244, -42.28143692),
                    view.point(-713.0118408, -42.62429047),
                    view.point(-713.006958, -42.44883347),
                );
                path.cubic_bezier_to(
                    view.point(-711.9234619, -39.77416611),
                    view.point(-712.7281494, -41.40340424),
                    view.point(-712.37854, -40.56031418),
                );
                path.cubic_bezier_to(
                    view.point(-711.5709229, -39.40369415),
                    view.point(-711.8365479, -39.62485504),
                    view.point(-711.7159424, -39.49802399),
                );
                path.cubic_bezier_to(
                    view.point(-711.0899658, -39.23141098),
                    view.point(-711.4263916, -39.30961227),
                    view.point(-711.2618408, -39.25047302),
                );
                path.line_to(view.point(-709.0391846, -39.00292206));
                path.line_to(view.point(-708.9371338, -38.90077591));
                path.line_to(view.point(-708.7071533, -36.84584427));
                path.cubic_bezier_to(
                    view.point(-708.5357666, -36.36174011),
                    view.point(-708.6885986, -36.67331696),
                    view.point(-708.6300049, -36.50738525),
                );
                path.cubic_bezier_to(
                    view.point(-708.1646729, -36.0073967),
                    view.point(-708.4415283, -36.21633911),
                    view.point(-708.3145752, -36.09488297),
                );
                path.cubic_bezier_to(
                    view.point(-705.6583252, -34.96929932),
                    view.point(-707.37854, -35.55213165),
                    view.point(-706.5357666, -35.20292282),
                );
                path.cubic_bezier_to(
                    view.point(-705.1461182, -34.95781326),
                    view.point(-705.4908447, -34.92531204),
                    view.point(-705.3155518, -34.92140198),
                );
                path.cubic_bezier_to(
                    view.point(-704.6842041, -35.17848206),
                    view.point(-704.9771729, -34.99398041),
                    view.point(-704.8189697, -35.06973648),
                );
                path.line_to(view.point(-703.0723877, -36.46804047));
                path.line_to(view.point(-702.9268799, -36.46804047));
                path.line_to(view.point(-701.3150635, -35.17848206));
                path.cubic_bezier_to(
                    view.point(-700.8521729, -34.95952606),
                    view.point(-701.1802979, -35.07022476),
                    view.point(-701.0216064, -34.99520493),
                );
                path.cubic_bezier_to(
                    view.point(-700.3404541, -34.97296524),
                    view.point(-700.6827393, -34.9238472),
                    view.point(-700.5074463, -34.92848969),
                );
                path.cubic_bezier_to(
                    view.point(-697.8345947, -36.0073967),
                    view.point(-699.463501, -35.20585251),
                    view.point(-698.6207275, -35.5535965),
                );
                path.cubic_bezier_to(
                    view.point(-697.463501, -36.36002731),
                    view.point(-697.6851807, -36.09415054),
                    view.point(-697.5577393, -36.21511459),
                );
                path.cubic_bezier_to(
                    view.point(-697.2906494, -36.8419342),
                    view.point(-697.3687744, -36.50494385),
                    view.point(-697.3096924, -36.66989517),
                );
                path.line_to(view.point(-697.0626221, -38.89442062));
                path.line_to(view.point(-696.960083, -38.99657059));
                path.line_to(view.point(-694.9083252, -39.23141098));
                path.cubic_bezier_to(
                    view.point(-694.4268799, -39.40443039),
                    view.point(-694.7364502, -39.25047302),
                    view.point(-694.5714111, -39.30985641),
                );
                path.cubic_bezier_to(
                    view.point(-694.0743408, -39.77587509),
                    view.point(-694.2818604, -39.49900055),
                    view.point(-694.1612549, -39.62607574),
                );
                path.cubic_bezier_to(
                    view.point(-693.0367432, -42.28412628),
                    view.point(-693.6192627, -40.56251526),
                    view.point(-693.2706299, -41.40584564),
                );
                path.cubic_bezier_to(
                    view.point(-693.0240479, -42.7951088),
                    view.point(-692.9927979, -42.45103073),
                    view.point(-692.9884033, -42.62600327),
                );
                path.cubic_bezier_to(
                    view.point(-693.2423096, -43.25697327),
                    view.point(-693.0596924, -42.96397018),
                    view.point(-693.1343994, -43.12208176),
                );
                path.line_to(view.point(-694.5308838, -44.86983871));
                path.close();
                path.move_to(view.point(-696.7510986, -45.32022095));
                path.cubic_bezier_to(
                    view.point(-696.7510986, -44.56510544),
                    view.point(-696.7364502, -45.06875992),
                    view.point(-696.7364502, -44.81656647),
                );
                path.cubic_bezier_to(
                    view.point(-696.5108643, -43.80779266),
                    view.point(-696.7677002, -44.29165268),
                    view.point(-696.682251, -44.02161789),
                );
                path.line_to(view.point(-695.3292236, -42.32933426));
                path.cubic_bezier_to(
                    view.point(-695.7276611, -41.36308289),
                    view.point(-695.4405518, -41.99893951),
                    view.point(-695.5738525, -41.67612457),
                );
                path.line_to(view.point(-697.6080322, -41.15389633));
                path.cubic_bezier_to(
                    view.point(-698.3126221, -40.78562546),
                    view.point(-697.8804932, -41.12286377),
                    view.point(-698.1314697, -40.99139023),
                );
                path.cubic_bezier_to(
                    view.point(-698.8463135, -40.25118256),
                    view.point(-698.4796143, -40.5969696),
                    view.point(-698.6578369, -40.4185791),
                );
                path.cubic_bezier_to(
                    view.point(-699.2144775, -39.54665375),
                    view.point(-699.0523682, -40.07010269),
                    view.point(-699.1832275, -39.81912994),
                );
                path.line_to(view.point(-699.4224854, -37.66546631));
                path.cubic_bezier_to(
                    view.point(-700.3883057, -37.26469421),
                    view.point(-699.7354736, -37.51102448),
                    view.point(-700.0577393, -37.37710571),
                );
                path.line_to(view.point(-701.8658447, -38.44844055));
                path.cubic_bezier_to(
                    view.point(-702.5557861, -38.68988037),
                    view.point(-702.0616455, -38.60508347),
                    view.point(-702.3052979, -38.69012451),
                );
                path.line_to(view.point(-702.6221924, -38.68988037));
                path.cubic_bezier_to(
                    view.point(-703.3770752, -38.68988037),
                    view.point(-702.8736572, -38.67497253),
                    view.point(-703.1256104, -38.67497253),
                );
                path.cubic_bezier_to(
                    view.point(-704.1339111, -38.45039368),
                    view.point(-703.6505127, -38.70600891),
                    view.point(-703.9195557, -38.62072372),
                );
                path.line_to(view.point(-705.6114502, -37.26665115));
                path.cubic_bezier_to(
                    view.point(-706.5767822, -37.66546631),
                    view.point(-705.9415283, -37.37833023),
                    view.point(-706.2642822, -37.51151276),
                );
                path.line_to(view.point(-706.7857666, -39.54738617));
                path.cubic_bezier_to(
                    view.point(-707.1539307, -40.25215912),
                    view.point(-706.8170166, -39.81986237),
                    view.point(-706.9483643, -40.07107925),
                );
                path.cubic_bezier_to(
                    view.point(-707.6881104, -40.78636169),
                    view.point(-707.3424072, -40.41955566),
                    view.point(-707.5206299, -40.59794998),
                );
                path.cubic_bezier_to(
                    view.point(-708.3922119, -41.15487671),
                    view.point(-707.8687744, -40.99236679),
                    view.point(-708.119751, -41.12359619),
                );
                path.line_to(view.point(-710.2720947, -41.36308289));
                path.cubic_bezier_to(
                    view.point(-710.6724854, -42.32933426),
                    view.point(-710.4263916, -41.67612457),
                    view.point(-710.5601807, -41.99893951),
                );
                path.line_to(view.point(-709.4903564, -43.80779266));
                path.cubic_bezier_to(
                    view.point(-709.2501221, -44.56510544),
                    view.point(-709.319458, -44.02161789),
                    view.point(-709.2335205, -44.29165268),
                );
                path.cubic_bezier_to(
                    view.point(-709.2501221, -45.32022095),
                    view.point(-709.2647705, -44.81656647),
                    view.point(-709.2647705, -45.06875992),
                );
                path.cubic_bezier_to(
                    view.point(-709.4903564, -46.07753372),
                    view.point(-709.2335205, -45.59391785),
                    view.point(-709.319458, -45.86370468),
                );
                path.line_to(view.point(-710.6705322, -47.55599213));
                path.cubic_bezier_to(
                    view.point(-710.2720947, -48.5222435),
                    view.point(-710.5587158, -47.88638306),
                    view.point(-710.4259033, -48.20920181),
                );
                path.line_to(view.point(-708.3912354, -48.73142624));
                path.cubic_bezier_to(
                    view.point(-707.6871338, -49.09994125),
                    view.point(-708.1187744, -48.76246262),
                    view.point(-707.8677979, -48.89393616),
                );
                path.cubic_bezier_to(
                    view.point(-707.1529541, -49.63414383),
                    view.point(-707.5201416, -49.28835297),
                    view.point(-707.3414307, -49.46674728),
                );
                path.cubic_bezier_to(
                    view.point(-706.78479, -50.33891678),
                    view.point(-706.9473877, -49.81522369),
                    view.point(-706.81604, -50.06644058),
                );
                path.line_to(view.point(-706.5767822, -52.21985626));
                path.cubic_bezier_to(
                    view.point(-705.6114502, -52.62062836),
                    view.point(-706.2642822, -52.37454605),
                    view.point(-705.9415283, -52.50821686),
                );
                path.line_to(view.point(-704.1339111, -51.43688583));
                path.cubic_bezier_to(
                    view.point(-703.3770752, -51.19739914),
                    view.point(-703.9195557, -51.26655579),
                    view.point(-703.6505127, -51.1812706),
                );
                path.cubic_bezier_to(
                    view.point(-702.6221924, -51.19739914),
                    view.point(-703.1256104, -51.21206284),
                    view.point(-702.8736572, -51.21206284),
                );
                path.cubic_bezier_to(
                    view.point(-701.8658447, -51.43688583),
                    view.point(-702.3492432, -51.18053818),
                    view.point(-702.0797119, -51.2660675),
                );
                path.line_to(view.point(-700.3883057, -52.62062836));
                path.cubic_bezier_to(
                    view.point(-699.4224854, -52.21985626),
                    view.point(-700.0577393, -52.50821686),
                    view.point(-699.7354736, -52.37454605),
                );
                path.line_to(view.point(-699.213501, -50.3379364));
                path.cubic_bezier_to(
                    view.point(-698.8453369, -49.63316345),
                    view.point(-699.1827393, -50.06546021),
                    view.point(-699.0513916, -49.81424713),
                );
                path.cubic_bezier_to(
                    view.point(-698.3116455, -49.09896469),
                    view.point(-698.6568604, -49.46577072),
                    view.point(-698.4786377, -49.2873764),
                );
                path.cubic_bezier_to(
                    view.point(-697.6070557, -48.73044968),
                    view.point(-698.1304932, -48.89295959),
                    view.point(-697.8795166, -48.76173019),
                );
                path.line_to(view.point(-695.7276611, -48.5222435));
                path.cubic_bezier_to(
                    view.point(-695.3272705, -47.55599213),
                    view.point(-695.5733643, -48.20920181),
                    view.point(-695.4395752, -47.8866272),
                );
                path.line_to(view.point(-696.5093994, -46.07753372));
                path.cubic_bezier_to(
                    view.point(-696.7510986, -45.32022095),
                    view.point(-696.6807861, -45.86394882),
                    view.point(-696.7672119, -45.59391785),
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
