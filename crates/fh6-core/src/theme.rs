//! ClusterTheme enum. Keep the existing numeric values stable; add new themes at the end.
use std::fmt;
use std::str::FromStr;

use crate::serde_str::serde_by_name;

/// A cluster design. In `config.json` it is stored by [`ClusterTheme::name`] and read with
/// [`FromStr`], so the short aliases load too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum ClusterTheme {
    BoschDdu = 0,
    DriftGlass = 1,
    PorscheAnalog = 2,
    #[default]
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
    // Phase 7 next-gen concepts (catalog rows 30-48). Append only; never renumber.
    LumenEtch = 29,
    VernierLoupe = 30,
    DichroicLouvre = 31,
    FibreOpticCorona = 32,
    ApertureNine = 33,
    AeroSmokeTunnel = 34,
    RattrapanteSector = 35,
    SapphireMystery = 36,
    RackFocus = 37,
    BionicSpaceframe = 38,
    GeodesicWell = 39,
    Lightpainter = 40,
    TreadSeismograph = 41,
    StarSailOrigami = 42,
    ArcLadderCascade = 43,
    AuroraVeil = 44,
    IsochromaticLoadPath = 45,
    AbyssalShearBloom = 46,
    HarmonographDrive = 47,
    // The 24 dashes of the 2026-11-28 re-cut (catalog rows 49-72), in the order of their catalog
    // rows. Append only; never renumber.
    AmgMbuxSupersport = 48,
    SkylineR34Mfd = 49,
    VisionPetals = 50,
    AmgOneF1 = 51,
    CarplayIsland = 52,
    Gt3RsApogee = 53,
    Gt7CockpitPro = 54,
    LiquidMorph = 55,
    AmgMbuxTrackPace = 56,
    GtrNismoMfd = 57,
    SimInputTrace = 58,
    LorenzButterfly = 59,
    AmgMbuxClassic = 60,
    SuminagashiMarble = 61,
    T50Floodlit = 62,
    HakosukaS20 = 63,
    SimPitWall = 64,
    KaleidoscopeMirror = 65,
    RevueltoCorsa = 66,
    ZNismoSport = 67,
    GoldenAngle = 68,
    MclarenSlimDisplay = 69,
    HopfFibration = 70,
    HyperbolicTiles = 71,
    // The liquid-metal dash added later on 2026-11-28 (catalog row 73).
    FaradayMandala = 72,
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
            ClusterTheme::LumenEtch => "LumenEtch",
            ClusterTheme::VernierLoupe => "VernierLoupe",
            ClusterTheme::DichroicLouvre => "DichroicLouvre",
            ClusterTheme::FibreOpticCorona => "FibreOpticCorona",
            ClusterTheme::ApertureNine => "ApertureNine",
            ClusterTheme::AeroSmokeTunnel => "AeroSmokeTunnel",
            ClusterTheme::RattrapanteSector => "RattrapanteSector",
            ClusterTheme::SapphireMystery => "SapphireMystery",
            ClusterTheme::RackFocus => "RackFocus",
            ClusterTheme::BionicSpaceframe => "BionicSpaceframe",
            ClusterTheme::GeodesicWell => "GeodesicWell",
            ClusterTheme::Lightpainter => "Lightpainter",
            ClusterTheme::TreadSeismograph => "TreadSeismograph",
            ClusterTheme::StarSailOrigami => "StarSailOrigami",
            ClusterTheme::ArcLadderCascade => "ArcLadderCascade",
            ClusterTheme::AuroraVeil => "AuroraVeil",
            ClusterTheme::IsochromaticLoadPath => "IsochromaticLoadPath",
            ClusterTheme::AbyssalShearBloom => "AbyssalShearBloom",
            ClusterTheme::HarmonographDrive => "HarmonographDrive",
            ClusterTheme::AmgMbuxSupersport => "AmgMbuxSupersport",
            ClusterTheme::SkylineR34Mfd => "SkylineR34Mfd",
            ClusterTheme::VisionPetals => "VisionPetals",
            ClusterTheme::AmgOneF1 => "AmgOneF1",
            ClusterTheme::CarplayIsland => "CarplayIsland",
            ClusterTheme::Gt3RsApogee => "Gt3RsApogee",
            ClusterTheme::Gt7CockpitPro => "Gt7CockpitPro",
            ClusterTheme::LiquidMorph => "LiquidMorph",
            ClusterTheme::AmgMbuxTrackPace => "AmgMbuxTrackPace",
            ClusterTheme::GtrNismoMfd => "GtrNismoMfd",
            ClusterTheme::SimInputTrace => "SimInputTrace",
            ClusterTheme::LorenzButterfly => "LorenzButterfly",
            ClusterTheme::AmgMbuxClassic => "AmgMbuxClassic",
            ClusterTheme::SuminagashiMarble => "SuminagashiMarble",
            ClusterTheme::T50Floodlit => "T50Floodlit",
            ClusterTheme::HakosukaS20 => "HakosukaS20",
            ClusterTheme::SimPitWall => "SimPitWall",
            ClusterTheme::KaleidoscopeMirror => "KaleidoscopeMirror",
            ClusterTheme::RevueltoCorsa => "RevueltoCorsa",
            ClusterTheme::ZNismoSport => "ZNismoSport",
            ClusterTheme::GoldenAngle => "GoldenAngle",
            ClusterTheme::MclarenSlimDisplay => "MclarenSlimDisplay",
            ClusterTheme::HopfFibration => "HopfFibration",
            ClusterTheme::HyperbolicTiles => "HyperbolicTiles",
            ClusterTheme::FaradayMandala => "FaradayMandala",
        }
    }
}

impl fmt::Display for ClusterTheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            // New concepts: the family default primary (tokens paletteModes.familyDefaults).
            ClusterTheme::LumenEtch => (56, 189, 248),
            ClusterTheme::VernierLoupe => (56, 189, 248),
            ClusterTheme::DichroicLouvre => (56, 189, 248),
            ClusterTheme::FibreOpticCorona => (56, 189, 248),
            ClusterTheme::ApertureNine => (0, 210, 255),
            ClusterTheme::AeroSmokeTunnel => (0, 210, 255),
            ClusterTheme::RattrapanteSector => (255, 122, 26),
            ClusterTheme::SapphireMystery => (255, 122, 26),
            ClusterTheme::RackFocus => (0, 180, 255),
            ClusterTheme::BionicSpaceframe => (0, 180, 255),
            ClusterTheme::GeodesicWell => (59, 130, 246),
            ClusterTheme::Lightpainter => (59, 130, 246),
            ClusterTheme::TreadSeismograph => (59, 130, 246),
            ClusterTheme::StarSailOrigami => (0, 245, 212),
            ClusterTheme::ArcLadderCascade => (0, 245, 212),
            ClusterTheme::AuroraVeil => (0, 245, 212),
            ClusterTheme::IsochromaticLoadPath => (154, 216, 255),
            ClusterTheme::AbyssalShearBloom => (154, 216, 255),
            ClusterTheme::HarmonographDrive => (154, 216, 255),
            // The re-cut's dashes: their family default primary, as for the concepts above.
            ClusterTheme::AmgMbuxSupersport => (0, 210, 255),
            ClusterTheme::SkylineR34Mfd => (255, 122, 26),
            ClusterTheme::VisionPetals => (56, 189, 248),
            ClusterTheme::AmgOneF1 => (0, 180, 255),
            ClusterTheme::CarplayIsland => (56, 189, 248),
            ClusterTheme::Gt3RsApogee => (255, 122, 26),
            ClusterTheme::Gt7CockpitPro => (0, 210, 255),
            ClusterTheme::LiquidMorph => (56, 189, 248),
            ClusterTheme::AmgMbuxTrackPace => (0, 210, 255),
            ClusterTheme::GtrNismoMfd => (0, 210, 255),
            ClusterTheme::SimInputTrace => (0, 180, 255),
            ClusterTheme::LorenzButterfly => (154, 216, 255),
            ClusterTheme::AmgMbuxClassic => (0, 210, 255),
            ClusterTheme::SuminagashiMarble => (154, 216, 255),
            ClusterTheme::T50Floodlit => (255, 122, 26),
            ClusterTheme::HakosukaS20 => (255, 122, 26),
            ClusterTheme::SimPitWall => (0, 180, 255),
            ClusterTheme::KaleidoscopeMirror => (154, 216, 255),
            ClusterTheme::RevueltoCorsa => (0, 210, 255),
            ClusterTheme::ZNismoSport => (0, 210, 255),
            ClusterTheme::GoldenAngle => (154, 216, 255),
            ClusterTheme::MclarenSlimDisplay => (0, 210, 255),
            ClusterTheme::HopfFibration => (154, 216, 255),
            ClusterTheme::HyperbolicTiles => (154, 216, 255),
            ClusterTheme::FaradayMandala => (154, 216, 255),
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
            ClusterTheme::LumenEtch,
            ClusterTheme::VernierLoupe,
            ClusterTheme::DichroicLouvre,
            ClusterTheme::FibreOpticCorona,
            ClusterTheme::ApertureNine,
            ClusterTheme::AeroSmokeTunnel,
            ClusterTheme::RattrapanteSector,
            ClusterTheme::SapphireMystery,
            ClusterTheme::RackFocus,
            ClusterTheme::BionicSpaceframe,
            ClusterTheme::GeodesicWell,
            ClusterTheme::Lightpainter,
            ClusterTheme::TreadSeismograph,
            ClusterTheme::StarSailOrigami,
            ClusterTheme::ArcLadderCascade,
            ClusterTheme::AuroraVeil,
            ClusterTheme::IsochromaticLoadPath,
            ClusterTheme::AbyssalShearBloom,
            ClusterTheme::HarmonographDrive,
            ClusterTheme::AmgMbuxSupersport,
            ClusterTheme::SkylineR34Mfd,
            ClusterTheme::VisionPetals,
            ClusterTheme::AmgOneF1,
            ClusterTheme::CarplayIsland,
            ClusterTheme::Gt3RsApogee,
            ClusterTheme::Gt7CockpitPro,
            ClusterTheme::LiquidMorph,
            ClusterTheme::AmgMbuxTrackPace,
            ClusterTheme::GtrNismoMfd,
            ClusterTheme::SimInputTrace,
            ClusterTheme::LorenzButterfly,
            ClusterTheme::AmgMbuxClassic,
            ClusterTheme::SuminagashiMarble,
            ClusterTheme::T50Floodlit,
            ClusterTheme::HakosukaS20,
            ClusterTheme::SimPitWall,
            ClusterTheme::KaleidoscopeMirror,
            ClusterTheme::RevueltoCorsa,
            ClusterTheme::ZNismoSport,
            ClusterTheme::GoldenAngle,
            ClusterTheme::MclarenSlimDisplay,
            ClusterTheme::HopfFibration,
            ClusterTheme::HyperbolicTiles,
            ClusterTheme::FaradayMandala,
        ]
    }
}

impl ClusterTheme {
    /// The theme whose config name is `name` (case-insensitive, surrounding spaces ignored).
    /// Only the theme's own name matches; [`str::parse`] also accepts the short aliases.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.trim();
        Self::all()
            .iter()
            .copied()
            .find(|t| t.name().eq_ignore_ascii_case(name))
    }
}

/// Short and legacy style names accepted besides each theme's own name. No alias equals a
/// theme's own name, so the order of the two lookups never matters (a test checks this).
const ALIASES: &[(&str, ClusterTheme)] = &[
    ("bosch", ClusterTheme::BoschDdu),
    ("drift", ClusterTheme::DriftGlass),
    ("porsche", ClusterTheme::PorscheAnalog),
    ("analog", ClusterTheme::PorscheAnalog),
    ("windshield", ClusterTheme::WindshieldHud),
    ("tesla", ClusterTheme::TeslaAdas),
    ("retro", ClusterTheme::RetroCyber),
    ("gt7hood", ClusterTheme::Gt7HoodPro),
    ("hood", ClusterTheme::Gt7HoodPro),
    ("amg", ClusterTheme::AmgMbuxSuperscreen),
    ("mbux", ClusterTheme::AmgMbuxSuperscreen),
    ("lfa", ClusterTheme::LfaV10Nurburgring),
    ("pagani horologerie", ClusterTheme::PaganiHorologerie),
    ("pagani", ClusterTheme::PaganiHorologerie),
    ("f35helmet hmd", ClusterTheme::F35HelmetHmd),
    ("f35", ClusterTheme::F35HelmetHmd),
    ("helmet", ClusterTheme::F35HelmetHmd),
    ("bugatti", ClusterTheme::BugattiTourbillon),
    ("eva", ClusterTheme::EvaBioSync),
    ("koenigsegg", ClusterTheme::KoenigseggGyro),
    ("valkyrie", ClusterTheme::ValkyrieLmp1),
    ("synthwave", ClusterTheme::SynthwaveOutrun),
    ("outrun", ClusterTheme::SynthwaveOutrun),
    ("chrono", ClusterTheme::ChronoMultiverse),
];

/// The error [`ClusterTheme`]'s [`FromStr`] returns for a style name that is neither a theme's
/// own name nor one of the short aliases. It holds no copy of the input, so a failed parse never
/// allocates; callers that log the failure log the input themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct ParseClusterThemeError(());

impl fmt::Display for ParseClusterThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown cluster style name")
    }
}

impl std::error::Error for ParseClusterThemeError {}

impl FromStr for ClusterTheme {
    type Err = ParseClusterThemeError;

    /// Parses a config style name: any theme's own name or a short alias (`"bosch"`, `"hood"`,
    /// `"mbux"`), case-insensitive with surrounding spaces ignored. Allocation-free.
    ///
    /// # Errors
    ///
    /// [`ParseClusterThemeError`] when `s` names no theme.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let name = s.trim();
        Self::from_name(name)
            .or_else(|| {
                ALIASES
                    .iter()
                    .find(|(alias, _)| alias.eq_ignore_ascii_case(name))
                    .map(|&(_, theme)| theme)
            })
            .ok_or(ParseClusterThemeError(()))
    }
}

serde_by_name!(ClusterTheme, "a cluster style name");

#[cfg(test)]
mod tests {
    use super::{ClusterTheme, ParseClusterThemeError, ALIASES};

    #[test]
    fn numbering_is_dense_and_stable() {
        // Existing config values must never move: the numeric value is the index in all().
        for (i, t) in ClusterTheme::all().iter().enumerate() {
            assert_eq!(*t as usize, i, "{t}");
        }
        assert_eq!(ClusterTheme::ShibuyaChromaticPrisma as u8, 28);
        assert_eq!(ClusterTheme::LumenEtch as u8, 29);
        assert_eq!(ClusterTheme::HarmonographDrive as u8, 47);
        assert_eq!(ClusterTheme::AmgMbuxSupersport as u8, 48);
        assert_eq!(ClusterTheme::HyperbolicTiles as u8, 71);
        assert_eq!(ClusterTheme::FaradayMandala as u8, 72);
        assert_eq!(ClusterTheme::all().len(), 73);
    }

    #[test]
    fn names_round_trip() {
        for t in ClusterTheme::all() {
            assert_eq!(ClusterTheme::from_name(t.name()), Some(*t));
            assert_eq!(ClusterTheme::from_name(&t.name().to_lowercase()), Some(*t));
        }
        assert_eq!(ClusterTheme::from_name("nope"), None);
    }

    #[test]
    fn serde_uses_the_config_name_and_reads_aliases() {
        for t in ClusterTheme::all() {
            let json = serde_json::to_string(t).unwrap_or_default();
            assert_eq!(json, format!("\"{}\"", t.name()));
            assert_eq!(serde_json::from_str::<ClusterTheme>(&json).ok(), Some(*t));
        }
        assert_eq!(
            serde_json::from_str::<ClusterTheme>("\"mbux\"").ok(),
            Some(ClusterTheme::AmgMbuxSuperscreen)
        );
        // The four later AMG dashes never take the short aliases.
        assert_eq!("amg".parse::<ClusterTheme>(), Ok(ClusterTheme::AmgMbuxSuperscreen));
        let err = serde_json::from_str::<ClusterTheme>("\"NoSuchDash\"").map(|_| ());
        assert!(err.is_err(), "an unknown name is an error, never a silent default");
    }

    #[test]
    fn parse_accepts_every_own_name_in_any_case() {
        for t in ClusterTheme::all() {
            assert_eq!(t.name().parse::<ClusterTheme>(), Ok(*t));
            let shouty = format!("  {}  ", t.name().to_uppercase());
            assert_eq!(shouty.parse::<ClusterTheme>(), Ok(*t), "{shouty:?}");
        }
    }

    #[test]
    fn parse_accepts_the_short_aliases() {
        let cases = [
            ("bosch", ClusterTheme::BoschDdu),
            ("Hood", ClusterTheme::Gt7HoodPro),
            (" mbux ", ClusterTheme::AmgMbuxSuperscreen),
            ("Pagani Horologerie", ClusterTheme::PaganiHorologerie),
            ("F35Helmet HMD", ClusterTheme::F35HelmetHmd),
            ("analog", ClusterTheme::PorscheAnalog),
            ("CHRONO", ClusterTheme::ChronoMultiverse),
        ];
        for (s, want) in cases {
            assert_eq!(s.parse::<ClusterTheme>(), Ok(want), "{s:?}");
        }
    }

    #[test]
    fn parse_rejects_unknown_names() {
        for s in ["", "   ", "nope", "Bosch Ddu", "LiquidGlass2"] {
            assert_eq!(s.parse::<ClusterTheme>(), Err(ParseClusterThemeError(())), "{s:?}");
        }
        assert_eq!(ParseClusterThemeError(()).to_string(), "unknown cluster style name");
    }

    #[test]
    fn aliases_are_distinct_and_never_shadow_a_name() {
        for (i, (alias, _)) in ALIASES.iter().enumerate() {
            assert_eq!(ClusterTheme::from_name(alias), None, "{alias} is a theme's own name");
            assert_eq!(
                *alias,
                alias.trim().to_ascii_lowercase(),
                "{alias} is not trimmed lowercase"
            );
            let later = &ALIASES[i + 1..];
            assert!(later.iter().all(|(other, _)| other != alias), "{alias} is listed twice");
        }
    }
}
