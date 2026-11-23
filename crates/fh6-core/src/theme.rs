//! ClusterTheme enum. Keep the existing numeric values stable; add new themes at the end.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ClusterTheme {
    BoschDdu = 0,
    DriftGlass = 1,
    PorscheAnalog = 2,
    LiquidGlass = 3,
    WindshieldHud = 4,
    TeslaAdas = 5,
    RetroCyber = 6,
    Gt7HoodPro = 7,
    AmgMbuxSuperscreen = 8,
    LfaV10Nurburgring = 9,
    PaganiHorologerie = 10,
    F35HelmetHmd = 11,
    BugattiTourbillon = 12,
    EvaBioSync = 13,
    KoenigseggGyro = 14,
    ValkyrieLmp1 = 15,
    SynthwaveOutrun = 16,
    ChronoMultiverse = 17,
    ChronoEventHorizon = 18,
    NeuralCyberSynapse = 19,
    Gt7ModularTripod = 20,
    ArNavWayfinder = 21,
    SolarisWarpCore = 22,
    VisionSpatialGlass = 23,
    GlyphDotMatrix = 24,
    AeroPlasmaBlade = 25,
    CymaticResonanceField = 26,
    KineticLiquidMetal = 27,
    ShibuyaChromaticPrisma = 28,
}

#[allow(clippy::derivable_impls)]
impl Default for ClusterTheme {
    fn default() -> Self {
        ClusterTheme::LiquidGlass
    }
}

impl ClusterTheme {
    /// The config name (`"LiquidGlass"`), allocation-free.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ClusterTheme::BoschDdu => "BoschDdu",
            ClusterTheme::DriftGlass => "DriftGlass",
            ClusterTheme::PorscheAnalog => "PorscheAnalog",
            ClusterTheme::LiquidGlass => "LiquidGlass",
            ClusterTheme::WindshieldHud => "WindshieldHud",
            ClusterTheme::TeslaAdas => "TeslaAdas",
            ClusterTheme::RetroCyber => "RetroCyber",
            ClusterTheme::Gt7HoodPro => "Gt7HoodPro",
            ClusterTheme::AmgMbuxSuperscreen => "AmgMbuxSuperscreen",
            ClusterTheme::LfaV10Nurburgring => "LfaV10Nurburgring",
            ClusterTheme::PaganiHorologerie => "PaganiHorologerie",
            ClusterTheme::F35HelmetHmd => "F35HelmetHmd",
            ClusterTheme::BugattiTourbillon => "BugattiTourbillon",
            ClusterTheme::EvaBioSync => "EvaBioSync",
            ClusterTheme::KoenigseggGyro => "KoenigseggGyro",
            ClusterTheme::ValkyrieLmp1 => "ValkyrieLmp1",
            ClusterTheme::SynthwaveOutrun => "SynthwaveOutrun",
            ClusterTheme::ChronoMultiverse => "ChronoMultiverse",
            ClusterTheme::ChronoEventHorizon => "ChronoEventHorizon",
            ClusterTheme::NeuralCyberSynapse => "NeuralCyberSynapse",
            ClusterTheme::Gt7ModularTripod => "Gt7ModularTripod",
            ClusterTheme::ArNavWayfinder => "ArNavWayfinder",
            ClusterTheme::SolarisWarpCore => "SolarisWarpCore",
            ClusterTheme::VisionSpatialGlass => "VisionSpatialGlass",
            ClusterTheme::GlyphDotMatrix => "GlyphDotMatrix",
            ClusterTheme::AeroPlasmaBlade => "AeroPlasmaBlade",
            ClusterTheme::CymaticResonanceField => "CymaticResonanceField",
            ClusterTheme::KineticLiquidMetal => "KineticLiquidMetal",
            ClusterTheme::ShibuyaChromaticPrisma => "ShibuyaChromaticPrisma",
        }
    }
}

impl std::fmt::Display for ClusterTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl ClusterTheme {
    pub fn accent_rgb(&self) -> (u8, u8, u8) {
        match self {
            ClusterTheme::BoschDdu => (0, 176, 255),
            ClusterTheme::DriftGlass => (255, 60, 0),
            ClusterTheme::PorscheAnalog => (255, 214, 0),
            ClusterTheme::LiquidGlass => (56, 189, 248),
            ClusterTheme::WindshieldHud => (0, 229, 255),
            ClusterTheme::TeslaAdas => (255, 23, 68),
            ClusterTheme::RetroCyber => (255, 0, 128),
            ClusterTheme::Gt7HoodPro => (255, 214, 0),
            ClusterTheme::AmgMbuxSuperscreen => (0, 200, 150),
            ClusterTheme::LfaV10Nurburgring => (255, 60, 0),
            ClusterTheme::PaganiHorologerie => (255, 214, 0),
            ClusterTheme::F35HelmetHmd => (0, 229, 255),
            ClusterTheme::BugattiTourbillon => (56, 189, 248),
            ClusterTheme::EvaBioSync => (0, 176, 255),
            ClusterTheme::KoenigseggGyro => (255, 60, 0),
            ClusterTheme::ValkyrieLmp1 => (255, 23, 68),
            ClusterTheme::SynthwaveOutrun => (255, 0, 128),
            ClusterTheme::ChronoMultiverse => (56, 189, 248),
            ClusterTheme::ChronoEventHorizon => (0, 176, 255),
            ClusterTheme::NeuralCyberSynapse => (255, 23, 68),
            ClusterTheme::Gt7ModularTripod => (255, 214, 0),
            ClusterTheme::ArNavWayfinder => (56, 189, 248),
            ClusterTheme::SolarisWarpCore => (0, 229, 255),
            ClusterTheme::VisionSpatialGlass => (56, 189, 248),
            ClusterTheme::GlyphDotMatrix => (0, 176, 255),
            ClusterTheme::AeroPlasmaBlade => (255, 60, 0),
            ClusterTheme::CymaticResonanceField => (255, 0, 128),
            ClusterTheme::KineticLiquidMetal => (56, 189, 248),
            ClusterTheme::ShibuyaChromaticPrisma => (255, 214, 0),
        }
    }

    pub fn all() -> &'static [ClusterTheme] {
        &[
            ClusterTheme::BoschDdu,
            ClusterTheme::DriftGlass,
            ClusterTheme::PorscheAnalog,
            ClusterTheme::LiquidGlass,
            ClusterTheme::WindshieldHud,
            ClusterTheme::TeslaAdas,
            ClusterTheme::RetroCyber,
            ClusterTheme::Gt7HoodPro,
            ClusterTheme::AmgMbuxSuperscreen,
            ClusterTheme::LfaV10Nurburgring,
            ClusterTheme::PaganiHorologerie,
            ClusterTheme::F35HelmetHmd,
            ClusterTheme::BugattiTourbillon,
            ClusterTheme::EvaBioSync,
            ClusterTheme::KoenigseggGyro,
            ClusterTheme::ValkyrieLmp1,
            ClusterTheme::SynthwaveOutrun,
            ClusterTheme::ChronoMultiverse,
            ClusterTheme::ChronoEventHorizon,
            ClusterTheme::NeuralCyberSynapse,
            ClusterTheme::Gt7ModularTripod,
            ClusterTheme::ArNavWayfinder,
            ClusterTheme::SolarisWarpCore,
            ClusterTheme::VisionSpatialGlass,
            ClusterTheme::GlyphDotMatrix,
            ClusterTheme::AeroPlasmaBlade,
            ClusterTheme::CymaticResonanceField,
            ClusterTheme::KineticLiquidMetal,
            ClusterTheme::ShibuyaChromaticPrisma,
        ]
    }
}
