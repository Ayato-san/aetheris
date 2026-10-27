//! Catalog and validation for the thirteen logical categories in ADR 0001.
use super::EngineIni;

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Boolean { textual: bool },
    Integer { positive: bool },
    Decimal { positive: bool },
    Milliseconds,
    Choice(&'static [(&'static str, &'static str)]),
    Resolution,
}
#[derive(Debug)]
pub struct Setting {
    pub key: &'static str,
    pub recommended: &'static str,
    pub kind: Kind,
    pub description: &'static str,
}
#[derive(Debug)]
pub struct Category {
    pub name: &'static str,
    pub section: &'static str,
    pub warning: &'static str,
    pub settings: &'static [Setting],
}

// Levels: Epic's Unreal Engine console-variable and scalability references.
// https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-console-variables-reference
// Compatibility is deliberately not inferred from GPU vendor alone.
pub const CATEGORIES: &[Category] = &[
    Category {
        name: "1. DLSS / FSR",
        section: "/Script/Engine.RendererSettings",
        warning: "",
        settings: &[
            Setting {
                key: "r.ngx.dlss.enable",
                recommended: "0",
                kind: Kind::Boolean { textual: false },
                description: "Active la reconstruction DLSS.",
            },
            Setting {
                key: "r.fidelityfx.fsr.enabled",
                recommended: "0",
                kind: Kind::Boolean { textual: false },
                description: "Active la reconstruction FSR.",
            },
        ],
    },
    Category {
        name: "2. Entrées",
        section: "/Script/Engine.InputSettings",
        warning: "",
        settings: &[
            Setting {
                key: "InitialButtonRepeatDelay",
                recommended: "0.001",
                kind: Kind::Milliseconds,
                description: "Délai avant la première répétition d’une touche maintenue, en millisecondes.",
            },
            Setting {
                key: "ButtonRepeatDelay",
                recommended: "0.001",
                kind: Kind::Milliseconds,
                description: "Intervalle entre les répétitions d’une touche maintenue, en millisecondes.",
            },
            Setting {
                key: "bEnableMouseSmoothing",
                recommended: "False",
                kind: Kind::Boolean { textual: true },
                description: "Lisse les mouvements de la souris.",
            },
            Setting {
                key: "bCaptureMouseOnLaunch",
                recommended: "True",
                kind: Kind::Boolean { textual: true },
                description: "Capture la souris au lancement du jeu.",
            },
        ],
    },
    Category {
        name: "3. Fluidité",
        section: "/Script/Engine.Engine",
        warning: "",
        settings: &[
            Setting {
                key: "bUseVSync",
                recommended: "False",
                kind: Kind::Boolean { textual: true },
                description: "Synchronise les images avec le rafraîchissement de l’écran.",
            },
            Setting {
                key: "bSmoothFrameRate",
                recommended: "False",
                kind: Kind::Boolean { textual: true },
                description: "Active le lissage de la fréquence d’images entre les limites définies.",
            },
            Setting {
                key: "MinSmoothedFrameRate",
                recommended: "5",
                kind: Kind::Decimal { positive: true },
                description: "Limite minimale du lissage en FPS.",
            },
            Setting {
                key: "MaxSmoothedFrameRate",
                recommended: "360",
                kind: Kind::Decimal { positive: true },
                description: "Limite maximale du lissage en FPS.",
            },
        ],
    },
    Category {
        name: "4. Affichage",
        section: "SystemSettings",
        warning: "",
        settings: &[
            Setting {
                key: "FullscreenMode",
                recommended: "1",
                kind: Kind::Integer { positive: false },
                description: "Mode d’affichage.",
            },
            Setting {
                key: "PreferredFullscreenMode",
                recommended: "1",
                kind: Kind::Integer { positive: false },
                description: "Mode plein écran préféré.",
            },
            Setting {
                key: "r.SetRes",
                recommended: "",
                kind: Kind::Resolution,
                description: "Dimensions de sortie en pixels et mode d’affichage.",
            },
        ],
    },
    Category {
        name: "5. Ombres",
        section: "SystemSettings",
        warning: "Ce profil réduit fortement la qualité des ombres.",
        settings: &[
            Setting {
                key: "r.ShadowQuality",
                recommended: "1",
                kind: Kind::Integer { positive: false },
                description: "Niveau de qualité des ombres.",
            },
            Setting {
                key: "r.Shadow.MaxResolution",
                recommended: "256",
                kind: Kind::Integer { positive: false },
                description: "Résolution maximale des cartes d’ombres.",
            },
            Setting {
                key: "r.Shadow.CSM.MaxCascades",
                recommended: "1",
                kind: Kind::Integer { positive: false },
                description: "Nombre maximal de cascades des ombres directionnelles.",
            },
            Setting {
                key: "r.Shadow.CSM.TransitionScale",
                recommended: "0.2",
                kind: Kind::Decimal { positive: false },
                description: "Échelle de transition entre les cascades d’ombres.",
            },
            Setting {
                key: "r.Shadow.DistanceScale",
                recommended: "0.2",
                kind: Kind::Decimal { positive: false },
                description: "Échelle de distance d’affichage des ombres.",
            },
            Setting {
                key: "r.Shadow.MinResolution",
                recommended: "62",
                kind: Kind::Integer { positive: false },
                description: "Résolution minimale des cartes d’ombres.",
            },
            Setting {
                key: "r.DistanceFieldShadowing",
                recommended: "0",
                kind: Kind::Boolean { textual: false },
                description: "Active les ombres calculées par champs de distance.",
            },
            Setting {
                key: "r.ContactShadows",
                recommended: "0",
                kind: Kind::Boolean { textual: false },
                description: "Active les petites ombres de contact.",
            },
            Setting {
                key: "r.AOQuality",
                recommended: "0",
                kind: Kind::Choice(&[("0", "Off"), ("1", "Moyen"), ("2", "Élevé")]),
                description: "Qualité de l’occlusion ambiante par champs de distance.",
            },
            Setting {
                key: "r.Shadow.FilterMethod",
                recommended: "1",
                kind: Kind::Integer { positive: false },
                description: "Méthode de filtrage des ombres : 1 adoucit les contours avec le filtrage PCSS.",
            },
        ],
    },
    Category {
        name: "6. Visibilité des personnages",
        section: "SystemSettings",
        warning: "",
        settings: &[
            Setting {
                key: "g.OptimizeChar.ServerVisible.MinLength",
                recommended: "1200",
                kind: Kind::Integer { positive: true },
                description: "Seuil minimal utilisé pour la visibilité des personnages.",
            },
            Setting {
                key: "g.OptimizeChar.ServerVisible.Length",
                recommended: "18000",
                kind: Kind::Integer { positive: true },
                description: "Seuil global de visibilité des personnages, supérieur ou égal au seuil minimal.",
            },
        ],
    },
    Category {
        name: "7. Qualité d’image et anticrénelage",
        section: "SystemSettings",
        warning: "",
        settings: &[
            Setting {
                key: "r.FilmGrain",
                recommended: "0",
                kind: Kind::Boolean { textual: false },
                description: "Active le grain de film.",
            },
            Setting {
                key: "r.FilmGrainIntensity",
                recommended: "0",
                kind: Kind::Decimal { positive: false },
                description: "Intensité du grain de film.",
            },
            Setting {
                key: "r.SceneColorFringeQuality",
                recommended: "0",
                kind: Kind::Integer { positive: false },
                description: "Qualité des aberrations chromatiques.",
            },
            Setting {
                key: "r.Tonemapper.Quality",
                recommended: "0",
                kind: Kind::Integer { positive: false },
                description: "Qualité du traitement des couleurs et de la luminosité.",
            },
            Setting {
                key: "r.PostProcessQuality",
                recommended: "0",
                kind: Kind::Integer { positive: false },
                description: "Niveau général des effets de post-traitement.",
            },
            Setting {
                key: "r.DOF.Gather.EnableBokehSettings",
                recommended: "0",
                kind: Kind::Boolean { textual: false },
                description: "Active les réglages de bokeh lors de la collecte de profondeur de champ.",
            },
            Setting {
                key: "r.Tonemapper.Sharpen",
                recommended: "0",
                kind: Kind::Decimal { positive: false },
                description: "Intensité de l’accentuation de netteté.",
            },
            Setting {
                key: "r.DefaultFeature.AntiAliasing",
                recommended: "0",
                kind: Kind::Integer { positive: false },
                description: "Valeur numérique de la méthode d’anticrénelage par défaut.",
            },
            Setting {
                key: "r.TemporalAA.Upsampling",
                recommended: "0",
                kind: Kind::Boolean { textual: false },
                description: "Active la mise à l’échelle temporelle.",
            },
            Setting {
                key: "r.TemporalAA.Algorithm",
                recommended: "0",
                kind: Kind::Integer { positive: false },
                description: "Choix numérique de l’algorithme d’anticrénelage temporel.",
            },
            Setting {
                key: "r.TemporalAA.FilterSize",
                recommended: "0",
                kind: Kind::Decimal { positive: false },
                description: "Taille du filtre d’anticrénelage temporel.",
            },
        ],
    },
    Category {
        name: "8. Distance, LOD et streaming",
        section: "SystemSettings",
        warning: "Une distance accrue augmente la charge. Les biais LOD réduisent les détails. Le streaming implique un compromis entre qualité des textures et pression sur la VRAM.",
        settings: &[
            Setting {
                key: "r.ViewDistanceScale",
                recommended: "3",
                kind: Kind::Decimal { positive: false },
                description: "Multiplie la distance d’affichage des objets.",
            },
            Setting {
                key: "r.SkeletalMeshLODBias",
                recommended: "1",
                kind: Kind::Integer { positive: false },
                description: "Biais du niveau de détail des modèles animés ; une valeur positive réduit les détails.",
            },
            Setting {
                key: "r.StaticMeshLODDistanceScale",
                recommended: "10",
                kind: Kind::Decimal { positive: false },
                description: "Multiplie la distance de transition des modèles statiques ; une valeur élevée réduit les détails plus tôt.",
            },
            Setting {
                key: "r.ParticleLODBias",
                recommended: "1",
                kind: Kind::Integer { positive: false },
                description: "Biais du niveau de détail des particules.",
            },
            Setting {
                key: "r.Streaming.PoolSize",
                recommended: "0",
                kind: Kind::Integer { positive: false },
                description: "Taille du pool de textures en Mo ; 0 signifie illimité pour ce réglage seul.",
            },
            Setting {
                key: "r.Streaming.LimitPoolSizeToVRAM",
                recommended: "1",
                kind: Kind::Boolean { textual: false },
                description: "Demande une limite du pool fondée sur la VRAM disponible.",
            },
            Setting {
                key: "r.Streaming.DefragDynamicBounds",
                recommended: "1",
                kind: Kind::Boolean { textual: false },
                description: "Retire les bornes dynamiques inutilisées de la boucle de mise à jour du streaming.",
            },
            Setting {
                key: "r.Streaming.FramesForFullUpdate",
                recommended: "5",
                kind: Kind::Integer { positive: false },
                description: "Répartit les mises à jour complètes du streaming sur ce nombre d’images.",
            },
        ],
    },
    Category {
        name: "9. Latence des images",
        section: "SystemSettings",
        warning: "Activer les deux options modifie le compromis latence / performances : terminer immédiatement une image peut réduire le débit malgré le travail en parallèle.",
        settings: &[
            Setting {
                key: "r.OneFrameThreadLag",
                recommended: "1",
                kind: Kind::Boolean { textual: false },
                description: "Autorise jusqu’à une image de décalage entre les threads pour leur exécution en parallèle.",
            },
            Setting {
                key: "r.FinishCurrentFrame",
                recommended: "1",
                kind: Kind::Boolean { textual: false },
                description: "Force la fin et le rendu de l’image courante au lieu de la conserver en tampon ; peut réduire la latence au prix de performances.",
            },
        ],
    },
    Category {
        name: "10. Post-traitement",
        section: "SystemSettings",
        warning: "",
        settings: &[
            Setting {
                key: "r.PostProcessAAQuality",
                recommended: "0",
                kind: Kind::Choice(&[
                    ("0", "Off"),
                    ("1", "1"),
                    ("2", "2"),
                    ("3", "3"),
                    ("4", "4"),
                    ("5", "5"),
                    ("6", "6"),
                ]),
                description: "Qualité de l’anticrénelage de post-traitement ; 0 le désactive.",
            },
            Setting {
                key: "r.ScreenPercentage",
                recommended: "100",
                kind: Kind::Decimal { positive: false },
                description: "Échelle de résolution de sortie en pourcentage ; 100 correspond à la résolution de sortie complète.",
            },
            Setting {
                key: "r.Tonemapper.GrainQuantization",
                recommended: "0",
                kind: Kind::Boolean { textual: false },
                description: "Active le bruit de quantification du tonemapper, distinct du grain de film.",
            },
            Setting {
                key: "r.LensFlareQuality",
                recommended: "0",
                kind: Kind::Choice(&[
                    ("0", "Off"),
                    ("1", "Faible"),
                    ("2", "Bon"),
                    ("3", "Très bon"),
                ]),
                description: "Qualité des halos lumineux.",
            },
            Setting {
                key: "r.BloomQuality",
                recommended: "0",
                kind: Kind::Choice(&[
                    ("0", "Off"),
                    ("1", "Minimum"),
                    ("2", "Faible"),
                    ("3", "Moyen"),
                    ("4", "Bon"),
                    ("5", "Très bon"),
                ]),
                description: "Qualité de la diffusion des zones lumineuses.",
            },
            Setting {
                key: "r.MotionBlurQuality",
                recommended: "0",
                kind: Kind::Choice(&[
                    ("0", "Off"),
                    ("1", "Faible"),
                    ("2", "Moyen"),
                    ("3", "Élevé"),
                    ("4", "Très élevé"),
                ]),
                description: "Qualité du flou de mouvement.",
            },
            Setting {
                key: "r.DepthOfFieldQuality",
                recommended: "0",
                kind: Kind::Choice(&[
                    ("0", "Off"),
                    ("1", "Faible"),
                    ("2", "Élevé"),
                    ("3", "Cinématique"),
                    ("4", "Cinématique extrême"),
                ]),
                description: "Qualité de la profondeur de champ.",
            },
            Setting {
                key: "r.SSR.Quality",
                recommended: "0",
                kind: Kind::Choice(&[
                    ("0", "Off"),
                    ("1", "Faible"),
                    ("2", "Moyen"),
                    ("3", "Élevé"),
                    ("4", "Très élevé"),
                ]),
                description: "Qualité des réflexions calculées dans l’espace écran.",
            },
            Setting {
                key: "r.SSAO",
                recommended: "0",
                kind: Kind::Boolean { textual: false },
                description: "Active l’occlusion ambiante en espace écran.",
            },
            Setting {
                key: "r.AmbientOcclusionLevels",
                recommended: "1",
                kind: Kind::Choice(&[
                    ("-1", "Automatique"),
                    ("0", "Off"),
                    ("1", "1 niveau"),
                    ("2", "2 niveaux"),
                    ("3", "3 niveaux"),
                ]),
                description: "Nombre de niveaux d’occlusion ambiante ; 1 conserve un niveau.",
            },
            Setting {
                key: "r.SSS.Scale",
                recommended: "0",
                kind: Kind::Decimal { positive: false },
                description: "Échelle de diffusion sous la surface ; 0 désactive cet effet.",
            },
            Setting {
                key: "r.SSS.SampleSet",
                recommended: "0",
                kind: Kind::Choice(&[("0", "Faible"), ("1", "Moyen"), ("2", "Élevé")]),
                description: "Jeu d’échantillons de diffusion sous la surface ; reste modifiable lorsque l’échelle vaut 0.",
            },
        ],
    },
    Category {
        name: "11. Ramasse-miettes",
        section: "/Script/Engine.GarbageCollectionSettings",
        warning: "",
        settings: &[Setting {
            key: "TimeBetweenPurgingPendingKillObjects",
            recommended: "30",
            kind: Kind::Decimal { positive: true },
            description: "Intervalle de purge des objets en attente, en secondes. Un intervalle plus court ou plus long change le comportement du ramasse-miettes.",
        }],
    },
    Category {
        name: "12. Textures et VRAM",
        section: "UI",
        warning: "Autoriser tous les niveaux de mip et augmenter les textures résidentes peut accroître la pression sur la VRAM.",
        settings: &[
            Setting {
                key: "r.Streaming.MipBias",
                recommended: "0",
                kind: Kind::Decimal { positive: false },
                description: "Biais global des niveaux de mip ; 0 n’applique aucun biais global.",
            },
            Setting {
                key: "r.Streaming.UseAllMips",
                recommended: "1",
                kind: Kind::Boolean { textual: false },
                description: "Autorise tous les niveaux de mip disponibles.",
            },
            Setting {
                key: "MinTextureResidentMipCount",
                recommended: "16",
                kind: Kind::Integer { positive: false },
                description: "Nombre minimal de niveaux de mip gardés en mémoire.",
            },
            Setting {
                key: "MaxTextureSize",
                recommended: "0",
                kind: Kind::Integer { positive: false },
                description: "Taille maximale des textures ; 0 retire la limite.",
            },
            Setting {
                key: "PoolSizeVRAMPercentage",
                recommended: "95",
                kind: Kind::Integer { positive: false },
                description: "Part de la VRAM réservée au pool de textures, en pourcentage.",
            },
        ],
    },
    Category {
        name: "13. Shaders et cache de pipelines",
        section: "Character",
        warning: "",
        settings: &[
            Setting {
                key: "r.CompileShadersForDevelopment",
                recommended: "False",
                kind: Kind::Boolean { textual: true },
                description: "Active la compilation des shaders destinés au développement.",
            },
            Setting {
                key: "r.ShaderPipelineCache.Enabled",
                recommended: "1",
                kind: Kind::Boolean { textual: false },
                description: "Active le cache de pipelines de shaders.",
            },
            Setting {
                key: "r.ShaderPipelineCache.StartupMode",
                recommended: "3",
                kind: Kind::Choice(&[
                    ("0", "Paused"),
                    ("1", "Fast"),
                    ("2", "Background"),
                    ("3", "Precompile"),
                ]),
                description: "Mode de démarrage du cache : pause, rapide, arrière-plan ou précompilation.",
            },
            Setting {
                key: "r.ShaderPipelineCache.GameFileMaskEnabled",
                recommended: "1",
                kind: Kind::Boolean { textual: false },
                description: "Active le filtrage du cache par masque de fichier du jeu.",
            },
            Setting {
                key: "r.ShaderPipelineCache.BackgroundBatchingTime",
                recommended: "-1",
                kind: Kind::Decimal { positive: false },
                description: "Durée de traitement des lots en arrière-plan ; accepte la valeur spéciale -1.",
            },
            Setting {
                key: "r.XGEShaderCompile",
                recommended: "1",
                kind: Kind::Boolean { textual: false },
                description: "Active la compilation distribuée des shaders avec XGE.",
            },
        ],
    },
];

impl Setting {
    /// Converts an explicit UI entry to the canonical INI representation.
    pub fn parse_input(&self, input: &str) -> Result<String, String> {
        let input = input.trim().replace(',', ".");
        let invalid = || format!("Valeur invalide pour {}.", self.key);
        match self.kind {
            Kind::Boolean { textual } => {
                let enabled = if input == "1" || input.eq_ignore_ascii_case("true") {
                    true
                } else if input == "0" || input.eq_ignore_ascii_case("false") {
                    false
                } else {
                    return Err(invalid());
                };
                Ok(if textual {
                    if enabled { "True" } else { "False" }
                } else if enabled {
                    "1"
                } else {
                    "0"
                }
                .into())
            }
            Kind::Choice(choices) => choices
                .iter()
                .find(|(value, _)| *value == input)
                .map(|(value, _)| value.to_string())
                .ok_or_else(invalid),
            Kind::Integer { positive } => {
                let digits = input.strip_prefix('-').unwrap_or(&input);
                if digits.is_empty()
                    || !digits.bytes().all(|b| b.is_ascii_digit())
                    || (positive && (input.starts_with('-') || digits.bytes().all(|b| b == b'0')))
                {
                    return Err(invalid());
                }
                Ok(input)
            }
            Kind::Decimal { .. } | Kind::Milliseconds => {
                let number: f64 = input.parse().map_err(|_| invalid())?;
                if !number.is_finite()
                    || (matches!(self.kind, Kind::Decimal { positive: true }) && number <= 0.)
                    || (matches!(self.kind, Kind::Milliseconds) && number < 1.)
                {
                    return Err(invalid());
                }
                Ok(if matches!(self.kind, Kind::Milliseconds) {
                    (number / 1000.).to_string()
                } else {
                    number.to_string()
                })
            }
            Kind::Resolution => Resolution::parse(&input)
                .map(|r| r.to_string())
                .ok_or_else(invalid),
        }
    }

    pub fn display_value(&self, raw: &str) -> Option<String> {
        if matches!(self.kind, Kind::Milliseconds) {
            let seconds: f64 = raw.parse().ok()?;
            let millis = seconds * 1000.;
            return (millis.is_finite() && millis >= 1.).then(|| millis.to_string());
        }
        self.parse_input(raw).ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Windowed,
    Borderless,
    FullScreen,
}
impl DisplayMode {
    pub const ALL: [Self; 3] = [Self::Windowed, Self::Borderless, Self::FullScreen];
    pub fn label(self) -> &'static str {
        match self {
            Self::Windowed => "Fenêtré",
            Self::Borderless => "Sans bordures",
            Self::FullScreen => "Plein écran",
        }
    }
    fn values(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Windowed => ("2", "1", "w"),
            Self::Borderless => ("1", "1", "wf"),
            Self::FullScreen => ("0", "0", "f"),
        }
    }
    pub fn read(ini: &EngineIni) -> Option<Self> {
        let resolution = Resolution::parse(ini.get("SystemSettings", "r.SetRes")?)?;
        Self::ALL.into_iter().find(|mode| {
            let (full, preferred, _) = mode.values();
            resolution.mode == *mode
                && ini.get("SystemSettings", "FullscreenMode") == Some(full)
                && ini.get("SystemSettings", "PreferredFullscreenMode") == Some(preferred)
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    pub width: u64,
    pub height: u64,
    pub mode: DisplayMode,
}
impl Resolution {
    pub fn parse(raw: &str) -> Option<Self> {
        let (dimensions, mode) = if let Some(s) = raw.strip_suffix("wf") {
            (s, DisplayMode::Borderless)
        } else if let Some(s) = raw.strip_suffix('w') {
            (s, DisplayMode::Windowed)
        } else {
            (raw.strip_suffix('f')?, DisplayMode::FullScreen)
        };
        let (w, h) = dimensions.split_once('x')?;
        let width = w.parse().ok()?;
        let height = h.parse().ok()?;
        (width > 0 && height > 0).then_some(Self {
            width,
            height,
            mode,
        })
    }
    pub fn exceeds(self, native: (u64, u64)) -> bool {
        self.width > native.0 || self.height > native.1
    }
}
impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}{}", self.width, self.height, self.mode.values().2)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Upscaling {
    Off,
    Dlss,
    Fsr,
}
impl Upscaling {
    pub const ALL: [Self; 3] = [Self::Off, Self::Dlss, Self::Fsr];
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Dlss => "DLSS",
            Self::Fsr => "FSR",
        }
    }
    pub fn read(ini: &EngineIni) -> Option<Self> {
        match (
            ini.get(CATEGORIES[0].section, CATEGORIES[0].settings[0].key),
            ini.get(CATEGORIES[0].section, CATEGORIES[0].settings[1].key),
        ) {
            (Some("0"), Some("0")) => Some(Self::Off),
            (Some("1"), Some("0")) => Some(Self::Dlss),
            (Some("0"), Some("1")) => Some(Self::Fsr),
            _ => None,
        }
    }
}

/// Applies the complete recommended profile as one operation.
pub fn apply_all_recommended(
    ini: &mut EngineIni,
    native: Option<(u64, u64)>,
    profile: Upscaling,
) -> Result<(), String> {
    let mut next = ini.clone();
    for category in CATEGORIES {
        category.apply_recommended(&mut next, native, profile)?;
    }
    *ini = next;
    Ok(())
}

pub fn reset_all(ini: &mut EngineIni, baseline: &EngineIni) {
    for category in CATEGORIES {
        category.reset(ini, baseline);
    }
}

impl Category {
    fn marker(&self) -> String {
        format!("; Aetheris — {}", self.name)
    }
    fn keys(&self) -> Vec<&str> {
        self.settings.iter().map(|s| s.key).collect()
    }
    fn label(&self, ini: &mut EngineIni) {
        ini.label_block(self.section, &self.keys(), &self.marker());
    }
    pub fn reset(&self, ini: &mut EngineIni, baseline: &EngineIni) {
        ini.remove_marker(&self.marker());
        ini.restore_properties_from(baseline, self.section, &self.keys());
        ini.remove_empty_sections(self.section);
        ini.space_category_blocks(self.section);
    }
    pub fn reset_setting(
        &self,
        ini: &mut EngineIni,
        baseline: &EngineIni,
        key: &str,
    ) -> Result<(), String> {
        if !self.settings.iter().any(|s| s.key == key) {
            return Err("Réglage inconnu.".into());
        }
        // The visibility pair and display triple are indivisible logical settings.
        let grouped = self.section == "SystemSettings"
            && (self.name == CATEGORIES[3].name || self.name == CATEGORIES[5].name);
        let keys = if grouped { self.keys() } else { vec![key] };
        ini.restore_properties_from(baseline, self.section, &keys);
        if self
            .settings
            .iter()
            .all(|s| ini.get(self.section, s.key).is_none())
        {
            ini.remove_marker(&self.marker());
        }
        ini.remove_empty_sections(self.section);
        ini.space_category_blocks(self.section);
        Ok(())
    }
    pub fn apply_recommended(
        &self,
        ini: &mut EngineIni,
        native: Option<(u64, u64)>,
        profile: Upscaling,
    ) -> Result<(), String> {
        if self.name == CATEGORIES[3].name {
            let (width, height) =
                native.ok_or("Résolution native indisponible : renseignez les dimensions.")?;
            return self.set_display(
                ini,
                Resolution {
                    width,
                    height,
                    mode: DisplayMode::Borderless,
                },
            );
        }
        if self.name == CATEGORIES[0].name {
            return self.set_upscaling(ini, profile);
        }
        for setting in self.settings {
            ini.remove(self.section, setting.key);
            ini.set(self.section, setting.key, setting.recommended);
        }
        self.label(ini);
        Ok(())
    }
    pub fn set_upscaling(&self, ini: &mut EngineIni, profile: Upscaling) -> Result<(), String> {
        if self.name != CATEGORIES[0].name {
            return Err("Catégorie incorrecte.".into());
        }
        for setting in self.settings {
            ini.remove(self.section, setting.key);
        }
        ini.set(
            self.section,
            self.settings[0].key,
            if profile == Upscaling::Dlss { "1" } else { "0" },
        );
        ini.set(
            self.section,
            self.settings[1].key,
            if profile == Upscaling::Fsr { "1" } else { "0" },
        );
        self.label(ini);
        Ok(())
    }
    pub fn set_display(&self, ini: &mut EngineIni, resolution: Resolution) -> Result<(), String> {
        if self.name != CATEGORIES[3].name || resolution.width == 0 || resolution.height == 0 {
            return Err("Dimensions invalides.".into());
        }
        let (full, preferred, _) = resolution.mode.values();
        for setting in self.settings {
            ini.remove(self.section, setting.key);
        }
        ini.set(self.section, "FullscreenMode", full);
        ini.set(self.section, "PreferredFullscreenMode", preferred);
        ini.set(self.section, "r.SetRes", resolution.to_string());
        self.label(ini);
        Ok(())
    }
    /// Validate all submitted values together; never defaults missing members of a pair.
    pub fn set_values(&self, ini: &mut EngineIni, entries: &[(&str, &str)]) -> Result<(), String> {
        if self.name == CATEGORIES[0].name || self.name == CATEGORIES[3].name {
            return Err("Utilisez le contrôle de profil.".into());
        }
        let mut next = ini.clone();
        for (key, input) in entries {
            let setting = self
                .settings
                .iter()
                .find(|s| s.key == *key)
                .ok_or("Réglage inconnu.")?;
            let value = setting.parse_input(input)?;
            next.remove(self.section, key);
            next.set(self.section, key, value);
        }
        let pair = if self.name == CATEGORIES[5].name {
            Some((self.settings[0].key, self.settings[1].key))
        } else if self.name == CATEGORIES[2].name
            && entries
                .iter()
                .any(|(k, _)| *k == "MinSmoothedFrameRate" || *k == "MaxSmoothedFrameRate")
        {
            Some(("MinSmoothedFrameRate", "MaxSmoothedFrameRate"))
        } else {
            None
        };
        if let Some((min, max)) = pair {
            let min_raw = next
                .get(self.section, min)
                .ok_or("Renseignez les deux limites.")?;
            let max_raw = next
                .get(self.section, max)
                .ok_or("Renseignez les deux limites.")?;
            for (key, raw) in [(min, min_raw), (max, max_raw)] {
                self.settings
                    .iter()
                    .find(|s| s.key == key)
                    .unwrap()
                    .parse_input(raw)?;
            }
            if self.name == CATEGORIES[5].name {
                // Compare arbitrary positive integers without a floating-point round trip.
                let a = min_raw.trim_start_matches('0');
                let b = max_raw.trim_start_matches('0');
                if a.len() > b.len() || (a.len() == b.len() && a > b) {
                    return Err("La limite minimale doit être ≤ à la limite maximale.".into());
                }
            } else if min_raw.parse::<f64>().map_err(|_| "Limite invalide.")?
                > max_raw.parse::<f64>().map_err(|_| "Limite invalide.")?
            {
                return Err("La limite minimale doit être ≤ à la limite maximale.".into());
            }
        }
        self.label(&mut next);
        *ini = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_ini::storage::BASELINE;
    #[test]
    fn recommendations_and_resets_cover_every_category_without_touching_paths_or_unknown_keys() {
        let baseline = EngineIni::parse(BASELINE);
        let initial = "; Player comment\r\n[Core.System]\r\nPaths=../../../Custom\r\n[SystemSettings]\r\nr.Custom=7\r\n; Another comment\r\n[/Script/Engine.RendererSettings]\r\nr.Unrelated=True\r\n";
        let mut ini = EngineIni::parse(initial);
        for category in CATEGORIES {
            category
                .apply_recommended(&mut ini, Some((2560, 1440)), Upscaling::Off)
                .unwrap();
        }
        let applied = ini.to_string();
        assert_eq!(applied.matches("; Aetheris — ").count(), 13);
        for category in CATEGORIES {
            for setting in category.settings {
                assert!(
                    ini.get(category.section, setting.key).is_some(),
                    "{}",
                    setting.key
                );
            }
        }
        for category in CATEGORIES {
            let mut isolated = ini.clone();
            category.reset(&mut isolated, &baseline);
            assert!(!isolated.to_string().contains(&category.marker()));
            for other in CATEGORIES
                .iter()
                .filter(|other| other.name != category.name)
            {
                assert!(isolated.to_string().contains(&other.marker()));
                for setting in other.settings {
                    assert_eq!(
                        isolated.get(other.section, setting.key),
                        ini.get(other.section, setting.key)
                    );
                }
            }
        }
        for category in CATEGORIES {
            category.reset(&mut ini, &baseline);
        }
        assert_eq!(ini.get("Core.System", "Paths"), Some("../../../Custom"));
        assert_eq!(ini.get("SystemSettings", "r.Custom"), Some("7"));
        assert_eq!(ini.get(CATEGORIES[0].section, "r.Unrelated"), Some("True"));
        assert_eq!(Upscaling::read(&ini), Some(Upscaling::Dlss));
        assert!(!ini.to_string().contains("Aetheris —"));
        assert!(ini.to_string().contains("; Another comment\r\n"));
    }
    #[test]
    fn blocks_are_contiguous_and_category_reset_removes_preexisting_duplicate_overrides() {
        let mut ini = EngineIni::parse(
            "[SystemSettings]\nr.ShadowQuality=5\n; custom\nr.FilmGrain=1\nr.ContactShadows=1\n[systemsettings]\nr.shadowquality=8\nr.Unknown=9\n",
        );
        CATEGORIES[4]
            .set_values(&mut ini, &[("r.ShadowQuality", "1")])
            .unwrap();
        assert!(
            ini.to_string()
                .contains("; Aetheris — 5. Ombres\nr.ContactShadows=1\nr.ShadowQuality=1")
        );
        assert_eq!(
            ini.to_string()
                .to_ascii_lowercase()
                .matches("r.shadowquality=")
                .count(),
            1
        );
        CATEGORIES[4].reset(&mut ini, &EngineIni::parse(BASELINE));
        assert_eq!(ini.get("SystemSettings", "r.ShadowQuality"), None);
        assert_eq!(ini.get("SystemSettings", "r.FilmGrain"), Some("1"));
        assert_eq!(ini.get("SystemSettings", "r.Unknown"), Some("9"));
        assert!(ini.to_string().contains("; custom"));
    }
    #[test]
    fn setting_reset_keeps_marker_until_last_managed_key_is_removed() {
        let mut ini = EngineIni::parse("");
        let baseline = EngineIni::parse(BASELINE);
        let category = &CATEGORIES[8];
        category
            .apply_recommended(&mut ini, None, Upscaling::Off)
            .unwrap();
        category
            .reset_setting(&mut ini, &baseline, "r.OneFrameThreadLag")
            .unwrap();
        assert!(ini.to_string().contains(&category.marker()));
        category
            .reset_setting(&mut ini, &baseline, "r.FinishCurrentFrame")
            .unwrap();
        assert!(!ini.to_string().contains(&category.marker()));
    }
    #[test]
    fn input_delay_converts_milliseconds_and_preserves_unknown_values_until_explicit_edit() {
        let setting = &CATEGORIES[1].settings[0];
        assert_eq!(setting.parse_input("1,5").unwrap(), "0.0015");
        assert_eq!(setting.display_value("0.0015"), Some("1.5".into()));
        assert_eq!(setting.display_value("0.0009"), None);
        for invalid in ["0", "-1", "0.9", "NaN", "inf"] {
            assert!(setting.parse_input(invalid).is_err());
        }
        assert!(setting.parse_input("10000000000").is_ok());
        let mut ini = EngineIni::parse(
            "[/Script/Engine.InputSettings]\nInitialButtonRepeatDelay=0.0009\n  ButtonRepeatDelay = older-custom  \n  ButtonRepeatDelay = custom  \n",
        );
        CATEGORIES[1]
            .set_values(&mut ini, &[("bEnableMouseSmoothing", "False")])
            .unwrap();
        assert_eq!(
            ini.get(CATEGORIES[1].section, "InitialButtonRepeatDelay"),
            Some("0.0009")
        );
        assert_eq!(
            ini.get(CATEGORIES[1].section, "ButtonRepeatDelay"),
            Some("custom")
        );
        assert!(
            ini.to_string()
                .contains("  ButtonRepeatDelay = older-custom  \n  ButtonRepeatDelay = custom  ")
        );
    }
    #[test]
    fn invalid_pairs_never_partially_mutate_the_document() {
        for (index, small, big) in [(5, "1200", "18000"), (2, "5", "360")] {
            let cat = &CATEGORIES[index];
            let start = if index == 2 { 2 } else { 0 };
            let a = cat.settings[start].key;
            let b = cat.settings[start + 1].key;
            let mut ini = EngineIni::parse("; intact");
            let original = ini.clone();
            assert!(cat.set_values(&mut ini, &[(a, small)]).is_err());
            assert_eq!(ini, original);
            assert!(cat.set_values(&mut ini, &[(a, big), (b, small)]).is_err());
            assert_eq!(ini, original);
            cat.set_values(&mut ini, &[(a, small), (b, big)]).unwrap();
            assert_eq!(ini.get(cat.section, a), Some(small));
            assert_eq!(ini.get(cat.section, b), Some(big));
        }
    }
    #[test]
    fn visibility_integers_are_not_compared_with_lossy_floating_point_precision() {
        let cat = &CATEGORIES[5];
        let mut ini = EngineIni::parse("");
        assert!(
            cat.set_values(
                &mut ini,
                &[
                    (
                        cat.settings[0].key,
                        "9999999999999999999999999999999999999999"
                    ),
                    (
                        cat.settings[1].key,
                        "9999999999999999999999999999999999999998"
                    )
                ]
            )
            .is_err()
        );
    }
    #[test]
    fn profile_and_display_changes_preserve_unrelated_keys() {
        let mut ini = EngineIni::parse(
            "[/Script/Engine.RendererSettings]\nr.Custom=15\n[SystemSettings]\nr.ScreenPercentage=85\n",
        );
        for profile in Upscaling::ALL {
            CATEGORIES[0].set_upscaling(&mut ini, profile).unwrap();
            assert_eq!(Upscaling::read(&ini), Some(profile));
            let output = ini.to_string();
            assert!(output.contains("r.ngx.dlss.enable="));
            assert!(output.contains("r.fidelityfx.fsr.enabled="));
            assert!(!output.contains("r.NGX.DLSS.Enable="));
            assert!(!output.contains("r.FidelityFX.FSR.Enabled="));
        }
        for mode in DisplayMode::ALL {
            let resolution = Resolution {
                width: 7680,
                height: 4320,
                mode,
            };
            CATEGORIES[3].set_display(&mut ini, resolution).unwrap();
            assert_eq!(DisplayMode::read(&ini), Some(mode));
            assert_eq!(
                Resolution::parse(ini.get("SystemSettings", "r.SetRes").unwrap()),
                Some(resolution)
            );
            assert!(resolution.exceeds((1920, 1080)));
        }
        assert_eq!(ini.get(CATEGORIES[0].section, "r.Custom"), Some("15"));
        assert_eq!(ini.get("SystemSettings", "r.ScreenPercentage"), Some("85"));
    }
    #[test]
    fn quality_selectors_preserve_unrecognized_raw_values_and_shader_negative_one_is_valid() {
        let setting = &CATEGORIES[9].settings[0];
        assert_eq!(setting.display_value("9"), None);
        assert!(setting.parse_input("6").is_ok());
        assert_eq!(CATEGORIES[12].settings[4].parse_input("-1").unwrap(), "-1");
    }
    #[test]
    fn category_reset_removes_empty_section_headers() {
        let baseline = EngineIni::parse(BASELINE);
        let mut ini = EngineIni::parse(
            "[/Script/Engine.InputSettings]\n; Aetheris — 2. Entrées\nInitialButtonRepeatDelay=0.001\nButtonRepeatDelay=0.001\nbEnableMouseSmoothing=False\nbCaptureMouseOnLaunch=True\n",
        );
        CATEGORIES[1].reset(&mut ini, &baseline);
        assert!(ini.to_string().trim().is_empty());
    }

    #[test]
    fn last_setting_reset_removes_only_empty_occurrences_of_its_section() {
        let baseline = EngineIni::parse(BASELINE);
        let mut ini = EngineIni::parse(
            "[SystemSettings]\n; Aetheris — 9. Latence des images\nr.OneFrameThreadLag=1\n\n[systemsettings]\n  \n[SystemSettings]\n; Player comment\n[SystemSettings]\nr.Unknown=7\n[Core.System]\nPaths=Keep\n",
        );
        CATEGORIES[8]
            .reset_setting(&mut ini, &baseline, "r.OneFrameThreadLag")
            .unwrap();
        assert_eq!(
            ini.to_string(),
            "[SystemSettings]\n; Player comment\n[SystemSettings]\nr.Unknown=7\n[Core.System]\nPaths=Keep\n"
        );
    }

    #[test]
    fn resetting_a_shared_section_preserves_other_categories_and_user_content() {
        let baseline = EngineIni::parse(BASELINE);
        for content in [
            "; Player comment",
            "# Comment",
            "Unknown line",
            "r.Unknown=7",
            "; Aetheris — 9. Latence des images\nr.OneFrameThreadLag=1",
        ] {
            let mut ini =
                EngineIni::parse(&format!("[SystemSettings]\nr.ShadowQuality=1\n{content}\n"));
            CATEGORIES[4].reset(&mut ini, &baseline);
            assert_eq!(ini.to_string(), format!("[SystemSettings]\n{content}\n"));
        }
    }
    #[test]
    fn global_apply_is_atomic_when_resolution_is_unavailable() {
        let mut ini =
            EngineIni::parse("[/Script/Engine.InputSettings]\nbCaptureMouseOnLaunch=False\n");
        let original = ini.clone();
        assert!(apply_all_recommended(&mut ini, None, Upscaling::Off).is_err());
        assert_eq!(ini, original);
    }

    #[test]
    fn global_actions_cover_all_categories_and_preserve_unmanaged_content() {
        let mut ini = EngineIni::parse(
            "[Core.System]\nPaths=Keep\n[SystemSettings]\nr.Unknown=7\n; Player comment\n",
        );
        apply_all_recommended(&mut ini, Some((2560, 1440)), Upscaling::Off).unwrap();
        for category in CATEGORIES {
            for setting in category.settings {
                assert!(ini.get(category.section, setting.key).is_some());
            }
        }
        reset_all(&mut ini, &EngineIni::parse(BASELINE));
        assert_eq!(Upscaling::read(&ini), Some(Upscaling::Dlss));
        assert_eq!(ini.get("Core.System", "Paths"), Some("Keep"));
        assert_eq!(ini.get("SystemSettings", "r.Unknown"), Some("7"));
        let output = ini.to_string();
        assert!(output.contains("; Player comment"));
        assert!(!output.contains("Aetheris —"));
        assert!(!output.contains("[/Script/Engine.InputSettings]"));
        for category in &CATEGORIES[1..] {
            for setting in category.settings {
                assert!(ini.get(category.section, setting.key).is_none());
            }
        }
    }
    #[test]
    fn renderer_resets_restore_the_exact_corrected_reference_without_blank_lines() {
        let baseline = EngineIni::parse(BASELINE);
        let expected =
            "[/Script/Engine.RendererSettings]\nr.ngx.dlss.enable=1\nr.fidelityfx.fsr.enabled=0\n";
        for global in [false, true] {
            let mut ini = EngineIni::parse(
                "[/Script/Engine.RendererSettings]\n\n\nr.NGX.DLSS.Enable=0\nr.FidelityFX.FSR.Enabled=0\n\n\n",
            );
            for _ in 0..3 {
                CATEGORIES[0]
                    .set_upscaling(&mut ini, Upscaling::Off)
                    .unwrap();
                if global {
                    reset_all(&mut ini, &baseline);
                } else {
                    CATEGORIES[0].reset(&mut ini, &baseline);
                }
                assert_eq!(ini.to_string(), expected);
            }
        }
    }

    #[test]
    fn shared_section_has_one_blank_line_between_categories_after_repeated_edits() {
        let mut ini = EngineIni::parse("[SystemSettings]\n");
        for _ in 0..3 {
            CATEGORIES[8]
                .apply_recommended(&mut ini, None, Upscaling::Off)
                .unwrap();
            CATEGORIES[5]
                .apply_recommended(&mut ini, None, Upscaling::Off)
                .unwrap();
        }
        assert_eq!(
            ini.to_string(),
            "[SystemSettings]\n; Aetheris — 9. Latence des images\nr.OneFrameThreadLag=1\nr.FinishCurrentFrame=1\n\n; Aetheris — 6. Visibilité des personnages\ng.OptimizeChar.ServerVisible.MinLength=1200\ng.OptimizeChar.ServerVisible.Length=18000\n"
        );
        CATEGORIES[4]
            .apply_recommended(&mut ini, None, Upscaling::Off)
            .unwrap();
        CATEGORIES[5].reset(&mut ini, &EngineIni::parse(BASELINE));
        assert!(
            ini.to_string()
                .contains("r.FinishCurrentFrame=1\n\n; Aetheris — 5. Ombres")
        );
    }
}
