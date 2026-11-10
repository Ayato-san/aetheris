use gpui::{IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*};

use super::ViewBox;

#[derive(IntoElement)]
pub(crate) struct CodeIcon {
    width: Pixels,
    height: Pixels,
    color: Rgba,
}

impl CodeIcon {
    pub(crate) fn new(width: Pixels, height: Pixels, color: Rgba) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

impl RenderOnce for CodeIcon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let view = ViewBox::new(bounds, -713.0, 22.62, 20.0, 17.258);

                let mut path = PathBuilder::fill();
                path.move_to(view.point(-705.4142456, 28.14244652));
                path.cubic_bezier_to(
                    view.point(-705.1110229, 27.41030312),
                    view.point(-705.4142456, 27.86776924),
                    view.point(-705.3048706, 27.60433578),
                );
                path.cubic_bezier_to(
                    view.point(-704.3795776, 27.10703659),
                    view.point(-704.9171753, 27.21602631),
                    view.point(-704.6539917, 27.10703659),
                );
                path.line_to(view.point(-699.2072144, 27.10703659));
                path.cubic_bezier_to(
                    view.point(-698.475769, 27.41030312),
                    view.point(-698.9328003, 27.10703659),
                    view.point(-698.6696167, 27.21602631),
                );
                path.cubic_bezier_to(
                    view.point(-698.1725464, 28.14244652),
                    view.point(-698.2819214, 27.60433578),
                    view.point(-698.1725464, 27.86776924),
                );
                path.cubic_bezier_to(
                    view.point(-698.475769, 28.87458801),
                    view.point(-698.1725464, 28.41712189),
                    view.point(-698.2819214, 28.68055534),
                );
                path.cubic_bezier_to(
                    view.point(-699.2072144, 29.17785645),
                    view.point(-698.6696167, 29.06886482),
                    view.point(-698.9328003, 29.17785645),
                );
                path.line_to(view.point(-704.3795776, 29.17785645));
                path.cubic_bezier_to(
                    view.point(-705.1110229, 28.87458801),
                    view.point(-704.6539917, 29.17785645),
                    view.point(-704.9171753, 29.06886482),
                );
                path.cubic_bezier_to(
                    view.point(-705.4142456, 28.14244652),
                    view.point(-705.3048706, 28.68055534),
                    view.point(-705.4142456, 28.41712189),
                );
                path.close();
                path.move_to(view.point(-704.3795776, 32.6293869));
                path.line_to(view.point(-699.2072144, 32.6293869));
                path.cubic_bezier_to(
                    view.point(-698.475769, 32.32611847),
                    view.point(-698.9328003, 32.6293869),
                    view.point(-698.6696167, 32.52039337),
                );
                path.cubic_bezier_to(
                    view.point(-698.1725464, 31.59397507),
                    view.point(-698.2819214, 32.13184357),
                    view.point(-698.1725464, 31.86865044),
                );
                path.cubic_bezier_to(
                    view.point(-698.475769, 30.86183357),
                    view.point(-698.1725464, 31.3192997),
                    view.point(-698.2819214, 31.05611038),
                );
                path.cubic_bezier_to(
                    view.point(-699.2072144, 30.55856514),
                    view.point(-698.6696167, 30.66755676),
                    view.point(-698.9328003, 30.55856514),
                );
                path.line_to(view.point(-704.3795776, 30.55856514));
                path.cubic_bezier_to(
                    view.point(-705.1110229, 30.86183357),
                    view.point(-704.6539917, 30.55856514),
                    view.point(-704.9171753, 30.66755676),
                );
                path.cubic_bezier_to(
                    view.point(-705.4142456, 31.59397507),
                    view.point(-705.3048706, 31.05611038),
                    view.point(-705.4142456, 31.3192997),
                );
                path.cubic_bezier_to(
                    view.point(-705.1110229, 32.32611847),
                    view.point(-705.4142456, 31.86865044),
                    view.point(-705.3048706, 32.13184357),
                );
                path.cubic_bezier_to(
                    view.point(-704.3795776, 32.6293869),
                    view.point(-704.9171753, 32.52039337),
                    view.point(-704.6539917, 32.6293869),
                );
                path.close();
                path.move_to(view.point(-693.0001831, 36.77127075));
                path.cubic_bezier_to(
                    view.point(-693.9093628, 38.96794128),
                    view.point(-693.0001831, 37.59529877),
                    view.point(-693.3273315, 38.38535309),
                );
                path.cubic_bezier_to(
                    view.point(-696.1036987, 39.87774658),
                    view.point(-694.491394, 39.55052948),
                    view.point(-695.2804565, 39.87774658),
                );
                path.line_to(view.point(-705.7589722, 39.87774658));
                path.cubic_bezier_to(
                    view.point(-707.9533081, 38.96794128),
                    view.point(-706.5817261, 39.87774658),
                    view.point(-707.3712769, 39.55052948),
                );
                path.cubic_bezier_to(
                    view.point(-708.8619995, 36.77127075),
                    view.point(-708.5353394, 38.38535309),
                    view.point(-708.8619995, 37.59529877),
                );
                path.line_to(view.point(-708.8619995, 25.72632599));
                path.cubic_bezier_to(
                    view.point(-709.1652222, 24.99418449),
                    view.point(-708.8619995, 25.45165062),
                    view.point(-708.9713745, 25.1884613),
                );
                path.cubic_bezier_to(
                    view.point(-709.8966675, 24.69091606),
                    view.point(-709.3595581, 24.79990768),
                    view.point(-709.6222534, 24.69091606),
                );
                path.cubic_bezier_to(
                    view.point(-710.6281128, 24.99418449),
                    view.point(-710.1710815, 24.69091606),
                    view.point(-710.4342651, 24.79990768),
                );
                path.cubic_bezier_to(
                    view.point(-710.9313354, 25.72632599),
                    view.point(-710.8224487, 25.1884613),
                    view.point(-710.9313354, 25.45165062),
                );
                path.cubic_bezier_to(
                    view.point(-710.6398315, 26.29058456),
                    view.point(-710.9313354, 26.04816628),
                    view.point(-710.6422729, 26.28814125),
                );
                path.line_to(view.point(-710.6554565, 26.27861023));
                path.cubic_bezier_to(
                    view.point(-710.3707886, 26.57747841),
                    view.point(-710.541687, 26.35803223),
                    view.point(-710.4450073, 26.45969009),
                );
                path.cubic_bezier_to(
                    view.point(-710.2262573, 26.96383286),
                    view.point(-710.2970581, 26.69526672),
                    view.point(-710.24823, 26.82649612),
                );
                path.cubic_bezier_to(
                    view.point(-710.2433472, 27.376091),
                    view.point(-710.2042847, 27.10092735),
                    view.point(-710.2096558, 27.24119759),
                );
                path.cubic_bezier_to(
                    view.point(-710.4196167, 27.74900436),
                    view.point(-710.2765503, 27.51098442),
                    view.point(-710.3366089, 27.63781548),
                );
                path.cubic_bezier_to(
                    view.point(-710.7277222, 28.02319145),
                    view.point(-710.5031128, 27.86019516),
                    view.point(-710.607605, 27.95330048),
                );
                path.cubic_bezier_to(
                    view.point(-711.1183472, 28.15515327),
                    view.point(-710.8478394, 28.09283829),
                    view.point(-710.9806519, 28.13780212),
                );
                path.cubic_bezier_to(
                    view.point(-711.52948, 28.12387276),
                    view.point(-711.2565308, 28.17250443),
                    view.point(-711.3961792, 28.16175079),
                );
                path.cubic_bezier_to(
                    view.point(-711.8961792, 27.93448448),
                    view.point(-711.663269, 28.08599472),
                    view.point(-711.7877808, 28.02148056),
                );
                path.cubic_bezier_to(
                    view.point(-713.0001831, 25.72632599),
                    view.point(-712.0099487, 27.85066414),
                    view.point(-713.0001831, 27.06818008),
                );
                path.cubic_bezier_to(
                    view.point(-712.0914917, 23.52965546),
                    view.point(-713.0001831, 24.90254402),
                    view.point(-712.6730347, 24.11248398),
                );
                path.cubic_bezier_to(
                    view.point(-709.8966675, 22.61985207),
                    view.point(-711.5094604, 22.94706726),
                    view.point(-710.7199097, 22.61985207),
                );
                path.line_to(view.point(-698.1725464, 22.61985207));
                path.cubic_bezier_to(
                    view.point(-695.9782104, 23.52965546),
                    view.point(-697.3493042, 22.61985207),
                    view.point(-696.5602417, 22.94706726),
                );
                path.cubic_bezier_to(
                    view.point(-695.0690308, 25.72632599),
                    view.point(-695.3961792, 24.11248398),
                    view.point(-695.0690308, 24.90254402),
                );
                path.line_to(view.point(-695.0690308, 34.35514832));
                path.line_to(view.point(-694.7243042, 34.35514832));
                path.cubic_bezier_to(
                    view.point(-694.1036987, 34.56237793),
                    view.point(-694.5006714, 34.35514832),
                    view.point(-694.2828979, 34.42797089),
                );
                path.cubic_bezier_to(
                    view.point(-693.0001831, 36.77127075),
                    view.point(-693.9909058, 34.64693451),
                    view.point(-693.0001831, 35.42941284),
                );
                path.close();
                path.move_to(view.point(-705.3605347, 35.06285477));
                path.cubic_bezier_to(
                    view.point(-704.9840698, 34.55064774),
                    view.point(-705.291687, 34.85684967),
                    view.point(-705.1603394, 34.67747879),
                );
                path.cubic_bezier_to(
                    view.point(-704.3795776, 34.35514832),
                    view.point(-704.8078003, 34.42357635),
                    view.point(-704.5963745, 34.35539246),
                );
                path.line_to(view.point(-697.1383667, 34.35514832));
                path.line_to(view.point(-697.1383667, 25.72632599));
                path.cubic_bezier_to(
                    view.point(-697.4411011, 24.99418449),
                    view.point(-697.1383667, 25.45165062),
                    view.point(-697.2472534, 25.1884613),
                );
                path.cubic_bezier_to(
                    view.point(-698.1725464, 24.69091606),
                    view.point(-697.6349487, 24.79990768),
                    view.point(-697.8981323, 24.69091606),
                );
                path.line_to(view.point(-706.9708862, 24.69091606));
                path.cubic_bezier_to(
                    view.point(-706.7936401, 25.72632599),
                    view.point(-706.8532104, 25.02350807),
                    view.point(-706.7931519, 25.37345123),
                );
                path.line_to(view.point(-706.7936401, 36.77127075));
                path.cubic_bezier_to(
                    view.point(-706.4904175, 37.50365448),
                    view.point(-706.7936401, 37.04594421),
                    view.point(-706.6842651, 37.30937958),
                );
                path.cubic_bezier_to(
                    view.point(-705.7589722, 37.80667877),
                    view.point(-706.2960815, 37.69768906),
                    view.point(-706.0333862, 37.80667877),
                );
                path.cubic_bezier_to(
                    view.point(-705.0275269, 37.50365448),
                    view.point(-705.4845581, 37.80667877),
                    view.point(-705.2213745, 37.69768906),
                );
                path.cubic_bezier_to(
                    view.point(-704.7243042, 36.77127075),
                    view.point(-704.8331909, 37.30937958),
                    view.point(-704.7243042, 37.04594421),
                );
                path.cubic_bezier_to(
                    view.point(-705.0001831, 36.21898651),
                    view.point(-704.7243042, 36.46237946),
                    view.point(-704.9977417, 36.2206955),
                );
                path.cubic_bezier_to(
                    view.point(-705.3668823, 35.69920349),
                    view.point(-705.1740112, 36.08873749),
                    view.point(-705.3024292, 35.90667725),
                );
                path.cubic_bezier_to(
                    view.point(-705.3605347, 35.06285477),
                    view.point(-705.4318237, 35.49172974),
                    view.point(-705.4293823, 35.269104),
                );
                path.close();
                path.move_to(view.point(-695.0690308, 36.77127075));
                path.cubic_bezier_to(
                    view.point(-695.1759644, 36.42621613),
                    view.point(-695.0724487, 36.6485939),
                    view.point(-695.1090698, 36.5290947),
                );
                path.line_to(view.point(-702.6759644, 36.42621613));
                path.cubic_bezier_to(
                    view.point(-702.6535034, 36.77127075),
                    view.point(-702.6608276, 36.54058075),
                    view.point(-702.6535034, 36.65592575),
                );
                path.cubic_bezier_to(
                    view.point(-702.8312378, 37.80667877),
                    view.point(-702.6535034, 37.12414551),
                    view.point(-702.713562, 37.4743309),
                );
                path.line_to(view.point(-696.1036987, 37.80667877));
                path.cubic_bezier_to(
                    view.point(-695.3722534, 37.50365448),
                    view.point(-695.8292847, 37.80667877),
                    view.point(-695.5661011, 37.69768906),
                );
                path.cubic_bezier_to(
                    view.point(-695.0690308, 36.77127075),
                    view.point(-695.1779175, 37.30937958),
                    view.point(-695.0690308, 37.04594421),
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
