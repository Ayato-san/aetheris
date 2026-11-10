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
                path.move_to(view.point(-705.414_25, 28.142_447));
                path.cubic_bezier_to(
                    view.point(-705.111, 27.410_303),
                    view.point(-705.414_25, 27.867_77),
                    view.point(-705.304_9, 27.604_336),
                );
                path.cubic_bezier_to(
                    view.point(-704.379_6, 27.107_037),
                    view.point(-704.917_2, 27.216_026),
                    view.point(-704.654, 27.107_037),
                );
                path.line_to(view.point(-699.207_2, 27.107_037));
                path.cubic_bezier_to(
                    view.point(-698.475_77, 27.410_303),
                    view.point(-698.932_8, 27.107_037),
                    view.point(-698.669_6, 27.216_026),
                );
                path.cubic_bezier_to(
                    view.point(-698.172_55, 28.142_447),
                    view.point(-698.281_9, 27.604_336),
                    view.point(-698.172_55, 27.867_77),
                );
                path.cubic_bezier_to(
                    view.point(-698.475_77, 28.874_588),
                    view.point(-698.172_55, 28.417_122),
                    view.point(-698.281_9, 28.680_555),
                );
                path.cubic_bezier_to(
                    view.point(-699.207_2, 29.177_856),
                    view.point(-698.669_6, 29.068_865),
                    view.point(-698.932_8, 29.177_856),
                );
                path.line_to(view.point(-704.379_6, 29.177_856));
                path.cubic_bezier_to(
                    view.point(-705.111, 28.874_588),
                    view.point(-704.654, 29.177_856),
                    view.point(-704.917_2, 29.068_865),
                );
                path.cubic_bezier_to(
                    view.point(-705.414_25, 28.142_447),
                    view.point(-705.304_9, 28.680_555),
                    view.point(-705.414_25, 28.417_122),
                );
                path.close();
                path.move_to(view.point(-704.379_6, 32.629_387));
                path.line_to(view.point(-699.207_2, 32.629_387));
                path.cubic_bezier_to(
                    view.point(-698.475_77, 32.326_12),
                    view.point(-698.932_8, 32.629_387),
                    view.point(-698.669_6, 32.520_393),
                );
                path.cubic_bezier_to(
                    view.point(-698.172_55, 31.593_975),
                    view.point(-698.281_9, 32.131_844),
                    view.point(-698.172_55, 31.868_65),
                );
                path.cubic_bezier_to(
                    view.point(-698.475_77, 30.861_834),
                    view.point(-698.172_55, 31.319_3),
                    view.point(-698.281_9, 31.056_11),
                );
                path.cubic_bezier_to(
                    view.point(-699.207_2, 30.558_565),
                    view.point(-698.669_6, 30.667_557),
                    view.point(-698.932_8, 30.558_565),
                );
                path.line_to(view.point(-704.379_6, 30.558_565));
                path.cubic_bezier_to(
                    view.point(-705.111, 30.861_834),
                    view.point(-704.654, 30.558_565),
                    view.point(-704.917_2, 30.667_557),
                );
                path.cubic_bezier_to(
                    view.point(-705.414_25, 31.593_975),
                    view.point(-705.304_9, 31.056_11),
                    view.point(-705.414_25, 31.319_3),
                );
                path.cubic_bezier_to(
                    view.point(-705.111, 32.326_12),
                    view.point(-705.414_25, 31.868_65),
                    view.point(-705.304_9, 32.131_844),
                );
                path.cubic_bezier_to(
                    view.point(-704.379_6, 32.629_387),
                    view.point(-704.917_2, 32.520_393),
                    view.point(-704.654, 32.629_387),
                );
                path.close();
                path.move_to(view.point(-693.000_2, 36.771_27));
                path.cubic_bezier_to(
                    view.point(-693.909_36, 38.967_94),
                    view.point(-693.000_2, 37.595_3),
                    view.point(-693.327_33, 38.385_353),
                );
                path.cubic_bezier_to(
                    view.point(-696.103_7, 39.877_747),
                    view.point(-694.491_4, 39.550_53),
                    view.point(-695.280_46, 39.877_747),
                );
                path.line_to(view.point(-705.759, 39.877_747));
                path.cubic_bezier_to(
                    view.point(-707.953_3, 38.967_94),
                    view.point(-706.581_7, 39.877_747),
                    view.point(-707.371_3, 39.550_53),
                );
                path.cubic_bezier_to(
                    view.point(-708.862, 36.771_27),
                    view.point(-708.535_34, 38.385_353),
                    view.point(-708.862, 37.595_3),
                );
                path.line_to(view.point(-708.862, 25.726_326));
                path.cubic_bezier_to(
                    view.point(-709.165_2, 24.994_184),
                    view.point(-708.862, 25.451_65),
                    view.point(-708.971_4, 25.188_461),
                );
                path.cubic_bezier_to(
                    view.point(-709.896_67, 24.690_916),
                    view.point(-709.359_56, 24.799_908),
                    view.point(-709.622_25, 24.690_916),
                );
                path.cubic_bezier_to(
                    view.point(-710.628_1, 24.994_184),
                    view.point(-710.171_1, 24.690_916),
                    view.point(-710.434_27, 24.799_908),
                );
                path.cubic_bezier_to(
                    view.point(-710.931_34, 25.726_326),
                    view.point(-710.822_45, 25.188_461),
                    view.point(-710.931_34, 25.451_65),
                );
                path.cubic_bezier_to(
                    view.point(-710.639_83, 26.290_585),
                    view.point(-710.931_34, 26.048_166),
                    view.point(-710.642_3, 26.288_141),
                );
                path.line_to(view.point(-710.655_46, 26.278_61));
                path.cubic_bezier_to(
                    view.point(-710.370_8, 26.577_478),
                    view.point(-710.541_7, 26.358_032),
                    view.point(-710.445, 26.459_69),
                );
                path.cubic_bezier_to(
                    view.point(-710.226_26, 26.963_833),
                    view.point(-710.297_06, 26.695_267),
                    view.point(-710.248_2, 26.826_496),
                );
                path.cubic_bezier_to(
                    view.point(-710.243_35, 27.376091),
                    view.point(-710.204_3, 27.100_927),
                    view.point(-710.209_66, 27.241_198),
                );
                path.cubic_bezier_to(
                    view.point(-710.419_6, 27.749_004),
                    view.point(-710.276_55, 27.510_984),
                    view.point(-710.336_6, 27.637_815),
                );
                path.cubic_bezier_to(
                    view.point(-710.727_7, 28.023_191),
                    view.point(-710.503_1, 27.860_195),
                    view.point(-710.607_6, 27.953_3),
                );
                path.cubic_bezier_to(
                    view.point(-711.118_35, 28.155_153),
                    view.point(-710.847_84, 28.092_838),
                    view.point(-710.980_65, 28.137_802),
                );
                path.cubic_bezier_to(
                    view.point(-711.529_5, 28.123_873),
                    view.point(-711.256_53, 28.172_504),
                    view.point(-711.396_2, 28.161_75),
                );
                path.cubic_bezier_to(
                    view.point(-711.896_2, 27.934_484),
                    view.point(-711.663_27, 28.085_995),
                    view.point(-711.787_8, 28.021_48),
                );
                path.cubic_bezier_to(
                    view.point(-713.000_2, 25.726_326),
                    view.point(-712.009_95, 27.850_664),
                    view.point(-713.000_2, 27.068_18),
                );
                path.cubic_bezier_to(
                    view.point(-712.091_5, 23.529_655),
                    view.point(-713.000_2, 24.902_544),
                    view.point(-712.673_03, 24.112_484),
                );
                path.cubic_bezier_to(
                    view.point(-709.896_67, 22.619_852),
                    view.point(-711.509_46, 22.947_067),
                    view.point(-710.719_9, 22.619_852),
                );
                path.line_to(view.point(-698.172_55, 22.619_852));
                path.cubic_bezier_to(
                    view.point(-695.978_2, 23.529_655),
                    view.point(-697.349_3, 22.619_852),
                    view.point(-696.560_24, 22.947_067),
                );
                path.cubic_bezier_to(
                    view.point(-695.069_03, 25.726_326),
                    view.point(-695.396_2, 24.112_484),
                    view.point(-695.069_03, 24.902_544),
                );
                path.line_to(view.point(-695.069_03, 34.355_15));
                path.line_to(view.point(-694.724_3, 34.355_15));
                path.cubic_bezier_to(
                    view.point(-694.103_7, 34.562_378),
                    view.point(-694.500_7, 34.355_15),
                    view.point(-694.282_9, 34.427_97),
                );
                path.cubic_bezier_to(
                    view.point(-693.000_2, 36.771_27),
                    view.point(-693.990_9, 34.646_935),
                    view.point(-693.000_2, 35.429_413),
                );
                path.close();
                path.move_to(view.point(-705.360_53, 35.062_855));
                path.cubic_bezier_to(
                    view.point(-704.984_1, 34.550_648),
                    view.point(-705.291_7, 34.856_85),
                    view.point(-705.160_34, 34.677_48),
                );
                path.cubic_bezier_to(
                    view.point(-704.379_6, 34.355_15),
                    view.point(-704.807_8, 34.423_576),
                    view.point(-704.596_4, 34.355_392),
                );
                path.line_to(view.point(-697.138_37, 34.355_15));
                path.line_to(view.point(-697.138_37, 25.726_326));
                path.cubic_bezier_to(
                    view.point(-697.441_1, 24.994_184),
                    view.point(-697.138_37, 25.451_65),
                    view.point(-697.247_25, 25.188_461),
                );
                path.cubic_bezier_to(
                    view.point(-698.172_55, 24.690_916),
                    view.point(-697.634_95, 24.799_908),
                    view.point(-697.898_13, 24.690_916),
                );
                path.line_to(view.point(-706.970_9, 24.690_916));
                path.cubic_bezier_to(
                    view.point(-706.793_64, 25.726_326),
                    view.point(-706.853_2, 25.023_508),
                    view.point(-706.793_15, 25.373_451),
                );
                path.line_to(view.point(-706.793_64, 36.771_27));
                path.cubic_bezier_to(
                    view.point(-706.490_4, 37.503_654),
                    view.point(-706.793_64, 37.045_944),
                    view.point(-706.684_27, 37.309_38),
                );
                path.cubic_bezier_to(
                    view.point(-705.759, 37.806_68),
                    view.point(-706.296_1, 37.697_69),
                    view.point(-706.033_4, 37.806_68),
                );
                path.cubic_bezier_to(
                    view.point(-705.027_5, 37.503_654),
                    view.point(-705.484_56, 37.806_68),
                    view.point(-705.221_4, 37.697_69),
                );
                path.cubic_bezier_to(
                    view.point(-704.724_3, 36.771_27),
                    view.point(-704.833_2, 37.309_38),
                    view.point(-704.724_3, 37.045_944),
                );
                path.cubic_bezier_to(
                    view.point(-705.000_2, 36.218_987),
                    view.point(-704.724_3, 36.462_38),
                    view.point(-704.997_74, 36.220_695),
                );
                path.cubic_bezier_to(
                    view.point(-705.366_9, 35.699_203),
                    view.point(-705.174, 36.088_737),
                    view.point(-705.302_4, 35.906_677),
                );
                path.cubic_bezier_to(
                    view.point(-705.360_53, 35.062_855),
                    view.point(-705.431_8, 35.491_73),
                    view.point(-705.429_4, 35.269104),
                );
                path.close();
                path.move_to(view.point(-695.069_03, 36.771_27));
                path.cubic_bezier_to(
                    view.point(-695.175_96, 36.426_216),
                    view.point(-695.072_45, 36.648_594),
                    view.point(-695.109_1, 36.529_095),
                );
                path.line_to(view.point(-702.675_96, 36.426_216));
                path.cubic_bezier_to(
                    view.point(-702.653_5, 36.771_27),
                    view.point(-702.660_8, 36.540_58),
                    view.point(-702.653_5, 36.655_926),
                );
                path.cubic_bezier_to(
                    view.point(-702.831_24, 37.806_68),
                    view.point(-702.653_5, 37.124_146),
                    view.point(-702.713_56, 37.474_33),
                );
                path.line_to(view.point(-696.103_7, 37.806_68));
                path.cubic_bezier_to(
                    view.point(-695.372_25, 37.503_654),
                    view.point(-695.829_3, 37.806_68),
                    view.point(-695.566_1, 37.697_69),
                );
                path.cubic_bezier_to(
                    view.point(-695.069_03, 36.771_27),
                    view.point(-695.177_9, 37.309_38),
                    view.point(-695.069_03, 37.045_944),
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
