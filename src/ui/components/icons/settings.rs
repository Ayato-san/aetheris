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
                path.move_to(view.point(-702.999_63, -49.732_624));
                path.cubic_bezier_to(
                    view.point(-705.658_8, -48.925_46),
                    view.point(-703.946_4, -49.732_624),
                    view.point(-704.871_7, -49.451_84),
                );
                path.cubic_bezier_to(
                    view.point(-707.422, -46.775_707),
                    view.point(-706.446_4, -48.399_08),
                    view.point(-707.059_7, -47.651_05),
                );
                path.cubic_bezier_to(
                    view.point(-707.694_46, -44.008_18),
                    view.point(-707.784_3, -45.900_36),
                    view.point(-707.879, -44.937_286),
                );
                path.cubic_bezier_to(
                    view.point(-706.384_4, -41.555_65),
                    view.point(-707.509_9, -43.079_07),
                    view.point(-707.053_83, -42.225_475),
                );
                path.cubic_bezier_to(
                    view.point(-703.933_7, -40.244_827),
                    view.point(-705.714_97, -40.885_82),
                    view.point(-704.861_94, -40.429_573),
                );
                path.cubic_bezier_to(
                    view.point(-701.168_1, -40.517_303),
                    view.point(-703.005, -40.059_837),
                    view.point(-702.042_6, -40.154_655),
                );
                path.cubic_bezier_to(
                    view.point(-699.019_65, -42.281_437),
                    view.point(-700.293_6, -40.879_71),
                    view.point(-699.545_53, -41.493_82),
                );
                path.cubic_bezier_to(
                    view.point(-698.213, -44.942_66),
                    view.point(-698.493_8, -43.069_298),
                    view.point(-698.213, -43.995_472),
                );
                path.cubic_bezier_to(
                    view.point(-699.616_8, -48.328_213),
                    view.point(-698.214_5, -46.212_67),
                    view.point(-698.719_36, -47.430_138),
                );
                path.cubic_bezier_to(
                    view.point(-702.999_63, -49.732_624),
                    view.point(-700.513_8, -49.226_04),
                    view.point(-701.730_6, -49.731_16),
                );
                path.close();
                path.move_to(view.point(-702.999_63, -42.363_544));
                path.cubic_bezier_to(
                    view.point(-704.431_76, -42.798_042),
                    view.point(-703.509_4, -42.363_544),
                    view.point(-704.007_93, -42.514_812),
                );
                path.cubic_bezier_to(
                    view.point(-705.381, -43.955_64),
                    view.point(-704.855_6, -43.081_516),
                    view.point(-705.186_16, -43.484_24),
                );
                path.cubic_bezier_to(
                    view.point(-705.527_95, -45.445_827),
                    view.point(-705.576_3, -44.427_036),
                    view.point(-705.627_1, -44.945_595),
                );
                path.cubic_bezier_to(
                    view.point(-704.822_4, -46.766_422),
                    view.point(-705.428_34, -45.946_304),
                    view.point(-705.182_74, -46.405_727),
                );
                path.cubic_bezier_to(
                    view.point(-703.502_56, -47.472_416),
                    view.point(-704.461_55, -47.127_117),
                    view.point(-704.002_56, -47.372_955),
                );
                path.cubic_bezier_to(
                    view.point(-702.013_3, -47.325_546),
                    view.point(-703.002_56, -47.571_877),
                    view.point(-702.484_5, -47.520_8),
                );
                path.cubic_bezier_to(
                    view.point(-700.856_57, -46.375_668),
                    view.point(-701.542_6, -47.130_29),
                    view.point(-701.139_8, -46.799_9),
                );
                path.cubic_bezier_to(
                    view.point(-700.422_5, -44.942_66),
                    view.point(-700.573_36, -45.951_435),
                    view.point(-700.422_5, -45.452_915),
                );
                path.cubic_bezier_to(
                    view.point(-701.177_37, -43.118_904),
                    view.point(-700.422_5, -44.258_663),
                    view.point(-700.694, -43.602_52),
                );
                path.cubic_bezier_to(
                    view.point(-702.999_63, -42.363_544),
                    view.point(-701.660_77, -42.635_044),
                    view.point(-702.31604, -42.363_544),
                );
                path.close();
                path.move_to(view.point(-694.530_9, -44.869_84));
                path.line_to(view.point(-694.530_9, -45.015_484));
                path.line_to(view.point(-693.242_3, -46.628_35));
                path.cubic_bezier_to(
                    view.point(-693.023_56, -47.091_682),
                    view.point(-693.133_9, -46.763_49),
                    view.point(-693.059_2, -46.922_333),
                );
                path.cubic_bezier_to(
                    view.point(-693.036_74, -47.603_89),
                    view.point(-692.987_9, -47.261_032),
                    view.point(-692.992_3, -47.436_493),
                );
                path.cubic_bezier_to(
                    view.point(-694.070_9, -50.111_16),
                    view.point(-693.269_65, -48.481_678),
                    view.point(-693.616_8, -49.324_764),
                );
                path.cubic_bezier_to(
                    view.point(-694.424_9, -50.482_85),
                    view.point(-694.158_3, -50.261_204),
                    view.point(-694.279_4, -50.388_523),
                );
                path.cubic_bezier_to(
                    view.point(-694.909_3, -50.653_91),
                    view.point(-694.570_9, -50.577_18),
                    view.point(-694.736_94, -50.635_586),
                );
                path.line_to(view.point(-696.960_1, -50.884_113));
                path.line_to(view.point(-697.062_6, -50.986_504));
                path.line_to(view.point(-697.292_6, -53.039_734));
                path.cubic_bezier_to(
                    view.point(-697.463_5, -53.523_594),
                    view.point(-697.310_67, -53.212_26),
                    view.point(-697.369_26, -53.377_945),
                );
                path.cubic_bezier_to(
                    view.point(-697.834_6, -53.877_934),
                    view.point(-697.557_74, -53.669_24),
                    view.point(-697.685_2, -53.790_69),
                );
                path.cubic_bezier_to(
                    view.point(-700.341_43, -54.916_03),
                    view.point(-698.620_7, -54.333_202),
                    view.point(-699.463_5, -54.682_41),
                );
                path.cubic_bezier_to(
                    view.point(-700.853_15, -54.927_517),
                    view.point(-700.508_4, -54.960_02),
                    view.point(-700.684_2, -54.963_93),
                );
                path.cubic_bezier_to(
                    view.point(-701.315_06, -54.707_092),
                    view.point(-701.022_1, -54.891_35),
                    view.point(-701.180_3, -54.815_838),
                );
                path.line_to(view.point(-702.926_9, -53.417_29));
                path.line_to(view.point(-703.072_4, -53.417_29));
                path.line_to(view.point(-704.684_2, -54.707_092));
                path.cubic_bezier_to(
                    view.point(-705.147_1, -54.925_808),
                    view.point(-704.819_46, -54.815_35),
                    view.point(-704.977_66, -54.890_13),
                );
                path.cubic_bezier_to(
                    view.point(-705.659_3, -54.912_365),
                    view.point(-705.316_5, -54.961_73),
                    view.point(-705.491_8, -54.956_84),
                );
                path.cubic_bezier_to(
                    view.point(-708.164_7, -53.873_29),
                    view.point(-706.536_74, -54.678_257),
                    view.point(-707.379, -54.328_804),
                );
                path.cubic_bezier_to(
                    view.point(-708.535_3, -53.520_66),
                    view.point(-708.314_1, -53.786_293),
                    view.point(-708.44104, -53.665_573),
                );
                path.cubic_bezier_to(
                    view.point(-708.707_15, -53.038_757),
                    view.point(-708.629, -53.375_748),
                    view.point(-708.688_1, -53.210_552),
                );
                path.line_to(view.point(-708.937_13, -50.986_504));
                path.line_to(view.point(-709.039_2, -50.884_113));
                path.line_to(view.point(-711.090_94, -50.653_91));
                path.cubic_bezier_to(
                    view.point(-711.574_8, -50.482_605),
                    view.point(-711.263_3, -50.635_586),
                    view.point(-711.429_3, -50.576_935),
                );
                path.cubic_bezier_to(
                    view.point(-711.928_83, -50.111_16),
                    view.point(-711.720_34, -50.388_28),
                    view.point(-711.841_43, -50.261_204),
                );
                path.cubic_bezier_to(
                    view.point(-712.962_5, -47.602_91),
                    view.point(-712.382_45, -49.324_52),
                    view.point(-712.730_1, -48.481_19),
                );
                path.cubic_bezier_to(
                    view.point(-712.975_7, -47.091_194),
                    view.point(-713.006_96, -47.435_76),
                    view.point(-713.011_35, -47.260_544),
                );
                path.cubic_bezier_to(
                    view.point(-712.757_45, -46.628_35),
                    view.point(-712.940_06, -46.922_085),
                    view.point(-712.865_36, -46.763_49),
                );
                path.line_to(view.point(-711.468_4, -45.015_484));
                path.line_to(view.point(-711.468_4, -44.869_84));
                path.line_to(view.point(-712.757_45, -43.256_973));
                path.cubic_bezier_to(
                    view.point(-712.976_2, -42.793_644),
                    view.point(-712.865_36, -43.121_838),
                    view.point(-712.940_55, -42.962_994),
                );
                path.cubic_bezier_to(
                    view.point(-712.962_5, -42.281_437),
                    view.point(-713.011_84, -42.624_29),
                    view.point(-713.006_96, -42.448_833),
                );
                path.cubic_bezier_to(
                    view.point(-711.923_46, -39.774_166),
                    view.point(-712.728_15, -41.403_404),
                    view.point(-712.37854, -40.560_314),
                );
                path.cubic_bezier_to(
                    view.point(-711.570_9, -39.403_694),
                    view.point(-711.836_55, -39.624_855),
                    view.point(-711.715_94, -39.498_024),
                );
                path.cubic_bezier_to(
                    view.point(-711.089_97, -39.231_41),
                    view.point(-711.426_4, -39.309_612),
                    view.point(-711.261_84, -39.250_473),
                );
                path.line_to(view.point(-709.039_2, -39.002_922));
                path.line_to(view.point(-708.937_13, -38.900_776));
                path.line_to(view.point(-708.707_15, -36.845_844));
                path.cubic_bezier_to(
                    view.point(-708.535_77, -36.361_74),
                    view.point(-708.688_6, -36.673_317),
                    view.point(-708.63, -36.507_385),
                );
                path.cubic_bezier_to(
                    view.point(-708.164_7, -36.007_397),
                    view.point(-708.441_5, -36.216_34),
                    view.point(-708.314_6, -36.094_883),
                );
                path.cubic_bezier_to(
                    view.point(-705.658_3, -34.969_3),
                    view.point(-707.37854, -35.552_13),
                    view.point(-706.535_77, -35.202_923),
                );
                path.cubic_bezier_to(
                    view.point(-705.146_1, -34.957_813),
                    view.point(-705.490_84, -34.925_312),
                    view.point(-705.315_55, -34.921_402),
                );
                path.cubic_bezier_to(
                    view.point(-704.684_2, -35.178_482),
                    view.point(-704.977_2, -34.993_98),
                    view.point(-704.819, -35.069_736),
                );
                path.line_to(view.point(-703.072_4, -36.468_04));
                path.line_to(view.point(-702.926_9, -36.468_04));
                path.line_to(view.point(-701.315_06, -35.178_482));
                path.cubic_bezier_to(
                    view.point(-700.852_2, -34.959_526),
                    view.point(-701.180_3, -35.070_225),
                    view.point(-701.021_6, -34.995_205),
                );
                path.cubic_bezier_to(
                    view.point(-700.340_45, -34.972_965),
                    view.point(-700.682_74, -34.923_847),
                    view.point(-700.507_45, -34.928_49),
                );
                path.cubic_bezier_to(
                    view.point(-697.834_6, -36.007_397),
                    view.point(-699.463_5, -35.205_853),
                    view.point(-698.620_7, -35.553_596),
                );
                path.cubic_bezier_to(
                    view.point(-697.463_5, -36.360_027),
                    view.point(-697.685_2, -36.094_15),
                    view.point(-697.557_74, -36.215_115),
                );
                path.cubic_bezier_to(
                    view.point(-697.290_65, -36.841_934),
                    view.point(-697.368_8, -36.504_944),
                    view.point(-697.309_7, -36.669_895),
                );
                path.line_to(view.point(-697.062_6, -38.894_42));
                path.line_to(view.point(-696.960_1, -38.996_57));
                path.line_to(view.point(-694.908_3, -39.231_41));
                path.cubic_bezier_to(
                    view.point(-694.426_9, -39.404_43),
                    view.point(-694.736_45, -39.250_473),
                    view.point(-694.571_4, -39.309_856),
                );
                path.cubic_bezier_to(
                    view.point(-694.074_34, -39.775_875),
                    view.point(-694.281_86, -39.499),
                    view.point(-694.161_25, -39.626_076),
                );
                path.cubic_bezier_to(
                    view.point(-693.036_74, -42.284_126),
                    view.point(-693.619_26, -40.562_515),
                    view.point(-693.270_6, -41.405_846),
                );
                path.cubic_bezier_to(
                    view.point(-693.024_05, -42.795_11),
                    view.point(-692.992_8, -42.451_03),
                    view.point(-692.988_4, -42.626_003),
                );
                path.cubic_bezier_to(
                    view.point(-693.242_3, -43.256_973),
                    view.point(-693.059_7, -42.963_97),
                    view.point(-693.134_4, -43.122_08),
                );
                path.line_to(view.point(-694.530_9, -44.869_84));
                path.close();
                path.move_to(view.point(-696.751_1, -45.320_22));
                path.cubic_bezier_to(
                    view.point(-696.751_1, -44.565_105),
                    view.point(-696.736_45, -45.068_76),
                    view.point(-696.736_45, -44.816_566),
                );
                path.cubic_bezier_to(
                    view.point(-696.510_86, -43.807_793),
                    view.point(-696.767_7, -44.291_653),
                    view.point(-696.682_25, -44.021_618),
                );
                path.line_to(view.point(-695.329_2, -42.329_334));
                path.cubic_bezier_to(
                    view.point(-695.727_66, -41.363_083),
                    view.point(-695.440_55, -41.998_94),
                    view.point(-695.573_85, -41.676_125),
                );
                path.line_to(view.point(-697.608_03, -41.153_896));
                path.cubic_bezier_to(
                    view.point(-698.312_6, -40.785_625),
                    view.point(-697.880_5, -41.122_864),
                    view.point(-698.131_5, -40.991_39),
                );
                path.cubic_bezier_to(
                    view.point(-698.846_3, -40.251_183),
                    view.point(-698.479_6, -40.596_97),
                    view.point(-698.657_84, -40.418_58),
                );
                path.cubic_bezier_to(
                    view.point(-699.214_5, -39.546_654),
                    view.point(-699.052_37, -40.070_103),
                    view.point(-699.183_2, -39.819_13),
                );
                path.line_to(view.point(-699.422_5, -37.665_466));
                path.cubic_bezier_to(
                    view.point(-700.388_3, -37.264_694),
                    view.point(-699.735_5, -37.511_024),
                    view.point(-700.057_74, -37.377_106),
                );
                path.line_to(view.point(-701.865_84, -38.448_44));
                path.cubic_bezier_to(
                    view.point(-702.555_8, -38.689_88),
                    view.point(-702.061_65, -38.605_083),
                    view.point(-702.305_3, -38.690_125),
                );
                path.line_to(view.point(-702.622_2, -38.689_88));
                path.cubic_bezier_to(
                    view.point(-703.377_1, -38.689_88),
                    view.point(-702.873_66, -38.674_973),
                    view.point(-703.125_6, -38.674_973),
                );
                path.cubic_bezier_to(
                    view.point(-704.133_9, -38.450_394),
                    view.point(-703.650_5, -38.706_01),
                    view.point(-703.919_56, -38.620_724),
                );
                path.line_to(view.point(-705.611_45, -37.266_65));
                path.cubic_bezier_to(
                    view.point(-706.576_8, -37.665_466),
                    view.point(-705.941_5, -37.378_33),
                    view.point(-706.264_3, -37.511_513),
                );
                path.line_to(view.point(-706.785_77, -39.547_386));
                path.cubic_bezier_to(
                    view.point(-707.153_93, -40.252_16),
                    view.point(-706.817, -39.819_862),
                    view.point(-706.948_36, -40.071_08),
                );
                path.cubic_bezier_to(
                    view.point(-707.688_1, -40.786_36),
                    view.point(-707.342_4, -40.419_556),
                    view.point(-707.520_6, -40.597_95),
                );
                path.cubic_bezier_to(
                    view.point(-708.392_2, -41.154_877),
                    view.point(-707.868_8, -40.992_367),
                    view.point(-708.119_75, -41.123_596),
                );
                path.line_to(view.point(-710.272_1, -41.363_083));
                path.cubic_bezier_to(
                    view.point(-710.672_5, -42.329_334),
                    view.point(-710.426_4, -41.676_125),
                    view.point(-710.560_2, -41.998_94),
                );
                path.line_to(view.point(-709.490_36, -43.807_793));
                path.cubic_bezier_to(
                    view.point(-709.250_1, -44.565_105),
                    view.point(-709.319_46, -44.021_618),
                    view.point(-709.233_5, -44.291_653),
                );
                path.cubic_bezier_to(
                    view.point(-709.250_1, -45.320_22),
                    view.point(-709.264_8, -44.816_566),
                    view.point(-709.264_8, -45.068_76),
                );
                path.cubic_bezier_to(
                    view.point(-709.490_36, -46.077_534),
                    view.point(-709.233_5, -45.593_918),
                    view.point(-709.319_46, -45.863_705),
                );
                path.line_to(view.point(-710.670_53, -47.555_992));
                path.cubic_bezier_to(
                    view.point(-710.272_1, -48.522_243),
                    view.point(-710.558_7, -47.886_383),
                    view.point(-710.425_9, -48.209_2),
                );
                path.line_to(view.point(-708.391_24, -48.731_426));
                path.cubic_bezier_to(
                    view.point(-707.687_13, -49.099_94),
                    view.point(-708.118_8, -48.762_463),
                    view.point(-707.867_8, -48.893_936),
                );
                path.cubic_bezier_to(
                    view.point(-707.152_95, -49.634_144),
                    view.point(-707.520_14, -49.288_353),
                    view.point(-707.341_43, -49.466_747),
                );
                path.cubic_bezier_to(
                    view.point(-706.784_8, -50.338_917),
                    view.point(-706.947_4, -49.815_224),
                    view.point(-706.81604, -50.066_44),
                );
                path.line_to(view.point(-706.576_8, -52.219_856));
                path.cubic_bezier_to(
                    view.point(-705.611_45, -52.620_63),
                    view.point(-706.264_3, -52.374_546),
                    view.point(-705.941_5, -52.508_217),
                );
                path.line_to(view.point(-704.133_9, -51.436_886));
                path.cubic_bezier_to(
                    view.point(-703.377_1, -51.197_4),
                    view.point(-703.919_56, -51.266_556),
                    view.point(-703.650_5, -51.181_27),
                );
                path.cubic_bezier_to(
                    view.point(-702.622_2, -51.197_4),
                    view.point(-703.125_6, -51.212_063),
                    view.point(-702.873_66, -51.212_063),
                );
                path.cubic_bezier_to(
                    view.point(-701.865_84, -51.436_886),
                    view.point(-702.349_24, -51.180_54),
                    view.point(-702.079_7, -51.266_068),
                );
                path.line_to(view.point(-700.388_3, -52.620_63));
                path.cubic_bezier_to(
                    view.point(-699.422_5, -52.219_856),
                    view.point(-700.057_74, -52.508_217),
                    view.point(-699.735_5, -52.374_546),
                );
                path.line_to(view.point(-699.213_5, -50.337_936));
                path.cubic_bezier_to(
                    view.point(-698.845_34, -49.633_163),
                    view.point(-699.182_74, -50.065_46),
                    view.point(-699.051_4, -49.814_247),
                );
                path.cubic_bezier_to(
                    view.point(-698.311_65, -49.098_965),
                    view.point(-698.656_86, -49.465_77),
                    view.point(-698.478_64, -49.287_376),
                );
                path.cubic_bezier_to(
                    view.point(-697.607_06, -48.730_45),
                    view.point(-698.130_5, -48.892_96),
                    view.point(-697.879_5, -48.761_73),
                );
                path.line_to(view.point(-695.727_66, -48.522_243));
                path.cubic_bezier_to(
                    view.point(-695.327_3, -47.555_992),
                    view.point(-695.573_36, -48.209_2),
                    view.point(-695.439_6, -47.886_627),
                );
                path.line_to(view.point(-696.509_4, -46.077_534));
                path.cubic_bezier_to(
                    view.point(-696.751_1, -45.320_22),
                    view.point(-696.680_8, -45.863_95),
                    view.point(-696.767_2, -45.593_918),
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
