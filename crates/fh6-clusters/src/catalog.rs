//! The Phase 7 dash catalog as data: one row per dash in
//! `program/phases/07-cluster-factory/CATALOG.md` (72 dashes in 9 categories of 8: the 48 of the
//! first catalog and the 24 added on 2026-11-28), with its module name, display name, category,
//! shelf, family, target, views, build order and pattern-setter flag.
//!
//! This table is the only source of display names: the registry, the OSD, the golden tests,
//! `preview-export` and `FH6_WheelDash --cluster` all read it, so a dash thread never edits a
//! shared file. Status lives in CATALOG.md, which also owns the build order; a change of order
//! touches only that page and this table.
use fh6_core::ClusterTheme;

pub use fh6_kit::Family;

/// Number of dashes in the catalog.
pub const COUNT: usize = 72;
/// Dashes per category.
pub const PER_CATEGORY: u8 = 8;

/// Whether the catalog row rebuilds a v1 theme or is new.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// One of the 29 v1 themes, which Phase 7 rebuilds on the kit (the v1 source is at commit
    /// `21e8a06`).
    Rebuild,
    /// A dash new in Phase 7: the 19 concepts of the first catalog and the 24 of the re-cut.
    New,
}

/// What the dash is measured against (CATALOG.md, "Target: what a dash is measured against").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// Rebuilds one named real display as closely as the sources allow.
    Recreation,
    /// Keeps named parts of a real car's display and departs from it on purpose.
    Inspired,
    /// No real car: the idea comes from the brief and the category.
    Concept,
}

impl Target {
    /// The catalog's wording (`recreation`, `inspired`, `concept`).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Target::Recreation => "recreation",
            Target::Inspired => "inspired",
            Target::Concept => "concept",
        }
    }
}

/// TJX's categories (2026-11-28): where a dash is listed and, for a dash with no real-car
/// target, where its idea comes from. Independent of [`Family`], which decides how it is built.
/// The variants are in TJX's order, the order of CATALOG.md's Categories table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Category {
    /// Road-car driver displays.
    Supercar,
    /// Hypercar instruments shown as machinery.
    Hypercar,
    /// Race, drift and track-day tools and sim overlays.
    Motorsport,
    /// Heritage dials and the electronics of the 1970s to 1990s.
    Retro,
    /// Clear optical glass over OLED black (not the Liquid Glass family).
    LiquidGlass,
    /// Spatial displays in the manner of visionOS and CarPlay.
    AppleVision,
    /// Living patterns computed from real physics and geometry.
    Psychedelic,
    /// Film props and real hardware that looks like the future (not the Sci-Fi family).
    SciFi,
    /// Apple-style motion design and pure mathematics.
    Abstract,
}

impl Category {
    /// Every category, in TJX's order.
    pub const ALL: [Category; 9] = [
        Category::Supercar,
        Category::Hypercar,
        Category::Motorsport,
        Category::Retro,
        Category::LiquidGlass,
        Category::AppleVision,
        Category::Psychedelic,
        Category::SciFi,
        Category::Abstract,
    ];

    /// The display name (`Liquid Glass`, `Sci-Fi`).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Category::Supercar => "Supercar",
            Category::Hypercar => "Hypercar",
            Category::Motorsport => "Motorsport",
            Category::Retro => "Retro",
            Category::LiquidGlass => "Liquid Glass",
            Category::AppleVision => "Apple Vision",
            Category::Psychedelic => "Psychedelic",
            Category::SciFi => "Sci-Fi",
            Category::Abstract => "Abstract",
        }
    }

    /// True for the two categories whose figure follows the colour law of CATALOG.md
    /// ("Psychedelic and Abstract: pattern first, colour from the model"): their colour budget is
    /// measured outside the rank-4 figure ([`fh6_kit::Cluster::figure_zone`]).
    #[must_use]
    pub const fn pattern_first(self) -> bool {
        matches!(self, Category::Psychedelic | Category::Abstract)
    }
}

/// The camera views a cluster is designed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Views {
    /// Wheel-less cockpit `[3]` (the binnacle anchor).
    pub cockpit: bool,
    /// Hood `[4]` (wider, lower, frameless; the scrim is 0%).
    pub hood: bool,
    /// A raised, collimated HUD on the sightline (`WindshieldHud`); drawn in the cockpit layout.
    pub raised_hud: bool,
}

impl Views {
    const COCKPIT: Self = Self {
        cockpit: true,
        hood: false,
        raised_hud: false,
    };
    const HOOD: Self = Self {
        cockpit: false,
        hood: true,
        raised_hud: false,
    };
    const BOTH: Self = Self {
        cockpit: true,
        hood: true,
        raised_hud: false,
    };
    const RAISED: Self = Self {
        cockpit: false,
        hood: false,
        raised_hud: true,
    };

    /// The catalog's wording (`cockpit`, `hood`, `cockpit+hood`, `raised HUD`).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match (self.cockpit, self.hood, self.raised_hud) {
            (_, _, true) => "raised HUD",
            (true, true, _) => "cockpit+hood",
            (false, true, _) => "hood",
            _ => "cockpit",
        }
    }
}

/// One catalog row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogEntry {
    /// Catalog row in CATALOG.md (1..=72): permanent, unlike the build order.
    pub row: u8,
    /// The config identifier (the dash's key).
    pub theme: ClusterTheme,
    /// Module directory: `crates/fh6-clusters/src/clusters/<snake>/`.
    pub snake: &'static str,
    /// Display name (OSD, gallery, product page). The only copy in the code.
    pub display: &'static str,
    /// TJX's category.
    pub category: Category,
    /// Place on its category's shelf (1..=8; 1 is the category's lead dash).
    pub shelf: u8,
    /// Design family.
    pub family: Family,
    /// Rebuild or new.
    pub kind: Kind,
    /// What the dash is measured against.
    pub target: Target,
    /// Views it is designed for.
    pub views: Views,
    /// Position in the build order (1 = the pilot); may change, see CATALOG.md.
    pub build_order: u8,
    /// The family's pattern-setter (★), built before its siblings.
    pub pattern_setter: bool,
}

impl CatalogEntry {
    /// The cluster's brief, relative to the repo root.
    #[must_use]
    pub fn spec_path(&self) -> String {
        format!("crates/fh6-clusters/src/clusters/{}/spec.md", self.snake)
    }
}

/// Every catalog row, in CATALOG.md order (catalog rows 1 to 72).
pub static CATALOG: [CatalogEntry; 72] = [
    CatalogEntry {
        row: 1,
        theme: ClusterTheme::LiquidGlass,
        snake: "liquid_glass",
        display: "Liquid Glass",
        category: Category::LiquidGlass,
        shelf: 1,
        family: Family::LiquidGlass,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 1,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 2,
        theme: ClusterTheme::VisionSpatialGlass,
        snake: "vision_spatial_glass",
        display: "Vision Spatial Glass",
        category: Category::AppleVision,
        shelf: 1,
        family: Family::LiquidGlass,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 10,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 3,
        theme: ClusterTheme::Gt7HoodPro,
        snake: "gt7_hood_pro",
        display: "Bonnet Line",
        category: Category::Motorsport,
        shelf: 1,
        family: Family::ModernDigital,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::HOOD,
        build_order: 3,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 4,
        theme: ClusterTheme::AmgMbuxSuperscreen,
        snake: "amg_mbux_superscreen",
        display: "AMG MBUX Superscreen",
        category: Category::Supercar,
        shelf: 3,
        family: Family::ModernDigital,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::COCKPIT,
        build_order: 8,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 5,
        theme: ClusterTheme::DriftGlass,
        snake: "drift_glass",
        display: "Drift Glass",
        category: Category::Motorsport,
        shelf: 5,
        family: Family::ModernDigital,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 13,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 6,
        theme: ClusterTheme::GlyphDotMatrix,
        snake: "glyph_dot_matrix",
        display: "Glyph Dot Matrix",
        category: Category::Retro,
        shelf: 5,
        family: Family::ModernDigital,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 21,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 7,
        theme: ClusterTheme::PorscheAnalog,
        snake: "porsche_analog",
        display: "Porsche Analog",
        category: Category::Retro,
        shelf: 1,
        family: Family::ClassicAnalog,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::COCKPIT,
        build_order: 2,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 8,
        theme: ClusterTheme::PaganiHorologerie,
        snake: "pagani_horologerie",
        display: "Pagani Horologerie",
        category: Category::Hypercar,
        shelf: 3,
        family: Family::ClassicAnalog,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::COCKPIT,
        build_order: 34,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 9,
        theme: ClusterTheme::BugattiTourbillon,
        snake: "bugatti_tourbillon",
        display: "Bugatti Tourbillon",
        category: Category::Hypercar,
        shelf: 1,
        family: Family::ClassicAnalog,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::COCKPIT,
        build_order: 25,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 10,
        theme: ClusterTheme::LfaV10Nurburgring,
        snake: "lfa_v10_nurburgring",
        display: "LFA V10 Nürburgring",
        category: Category::Supercar,
        shelf: 4,
        family: Family::ClassicAnalog,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::COCKPIT,
        build_order: 47,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 11,
        theme: ClusterTheme::KoenigseggGyro,
        snake: "koenigsegg_gyro",
        display: "Koenigsegg Gyro",
        category: Category::Hypercar,
        shelf: 5,
        family: Family::ClassicAnalog,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::COCKPIT,
        build_order: 52,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 12,
        theme: ClusterTheme::BoschDdu,
        snake: "bosch_ddu",
        display: "Bosch DDU",
        category: Category::Motorsport,
        shelf: 2,
        family: Family::RaceDash,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::COCKPIT,
        build_order: 4,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 13,
        theme: ClusterTheme::ValkyrieLmp1,
        snake: "valkyrie_lmp1",
        display: "Valkyrie LMP1",
        category: Category::Hypercar,
        shelf: 4,
        family: Family::RaceDash,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::BOTH,
        build_order: 9,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 14,
        theme: ClusterTheme::Gt7ModularTripod,
        snake: "gt7_modular_tripod",
        display: "GT7 Modular Tripod",
        category: Category::Motorsport,
        shelf: 7,
        family: Family::RaceDash,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::BOTH,
        build_order: 40,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 15,
        theme: ClusterTheme::TeslaAdas,
        snake: "tesla_adas",
        display: "Tesla ADAS",
        category: Category::SciFi,
        shelf: 2,
        family: Family::AdasHud,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::BOTH,
        build_order: 5,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 16,
        theme: ClusterTheme::WindshieldHud,
        snake: "windshield_hud",
        display: "Windshield HUD",
        category: Category::AppleVision,
        shelf: 3,
        family: Family::AdasHud,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::RAISED,
        build_order: 27,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 17,
        theme: ClusterTheme::ArNavWayfinder,
        snake: "ar_nav_wayfinder",
        display: "AR Nav Wayfinder",
        category: Category::AppleVision,
        shelf: 5,
        family: Family::AdasHud,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 36,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 18,
        theme: ClusterTheme::F35HelmetHmd,
        snake: "f35_helmet_hmd",
        display: "F-35 Helmet HMD",
        category: Category::SciFi,
        shelf: 3,
        family: Family::AdasHud,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::BOTH,
        build_order: 28,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 19,
        theme: ClusterTheme::RetroCyber,
        snake: "retro_cyber",
        display: "Retro Cyber",
        category: Category::Retro,
        shelf: 3,
        family: Family::SciFi,
        kind: Kind::Rebuild,
        target: Target::Recreation,
        views: Views::BOTH,
        build_order: 6,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 20,
        theme: ClusterTheme::SynthwaveOutrun,
        snake: "synthwave_outrun",
        display: "Synthwave Outrun",
        category: Category::Retro,
        shelf: 7,
        family: Family::SciFi,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 39,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 21,
        theme: ClusterTheme::EvaBioSync,
        snake: "eva_bio_sync",
        display: "EVA Bio-Sync",
        category: Category::SciFi,
        shelf: 6,
        family: Family::SciFi,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 64,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 22,
        theme: ClusterTheme::ChronoEventHorizon,
        snake: "chrono_event_horizon",
        display: "Chrono Event Horizon",
        category: Category::SciFi,
        shelf: 1,
        family: Family::SciFi,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 19,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 23,
        theme: ClusterTheme::NeuralCyberSynapse,
        snake: "neural_cyber_synapse",
        display: "Neural Cyber Synapse",
        category: Category::SciFi,
        shelf: 5,
        family: Family::SciFi,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 37,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 24,
        theme: ClusterTheme::SolarisWarpCore,
        snake: "solaris_warp_core",
        display: "Solaris Warp Core",
        category: Category::SciFi,
        shelf: 4,
        family: Family::SciFi,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 46,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 25,
        theme: ClusterTheme::AeroPlasmaBlade,
        snake: "aero_plasma_blade",
        display: "Aero Plasma Blade",
        category: Category::SciFi,
        shelf: 7,
        family: Family::SciFi,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::HOOD,
        build_order: 55,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 26,
        theme: ClusterTheme::ShibuyaChromaticPrisma,
        snake: "shibuya_chromatic_prisma",
        display: "Shibuya Chromatic Prisma",
        category: Category::LiquidGlass,
        shelf: 3,
        family: Family::SciFi,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 26,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 27,
        theme: ClusterTheme::KineticLiquidMetal,
        snake: "kinetic_liquid_metal",
        display: "Kinetic Liquid Metal",
        category: Category::Psychedelic,
        shelf: 2,
        family: Family::FluidGenerative,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 7,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 28,
        theme: ClusterTheme::CymaticResonanceField,
        snake: "cymatic_resonance_field",
        display: "Cymatic Resonance Field",
        category: Category::Psychedelic,
        shelf: 1,
        family: Family::FluidGenerative,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 14,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 29,
        theme: ClusterTheme::ChronoMultiverse,
        snake: "chrono_multiverse",
        display: "Chrono Multiverse",
        category: Category::Psychedelic,
        shelf: 3,
        family: Family::FluidGenerative,
        kind: Kind::Rebuild,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 23,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 30,
        theme: ClusterTheme::LumenEtch,
        snake: "lumen_etch",
        display: "Lumen Etch",
        category: Category::LiquidGlass,
        shelf: 6,
        family: Family::LiquidGlass,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 62,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 31,
        theme: ClusterTheme::VernierLoupe,
        snake: "vernier_loupe",
        display: "Vernier Loupe",
        category: Category::LiquidGlass,
        shelf: 2,
        family: Family::LiquidGlass,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 17,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 32,
        theme: ClusterTheme::DichroicLouvre,
        snake: "dichroic_louvre",
        display: "Dichroic Louvre",
        category: Category::LiquidGlass,
        shelf: 5,
        family: Family::LiquidGlass,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 44,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 33,
        theme: ClusterTheme::FibreOpticCorona,
        snake: "fibre_optic_corona",
        display: "Fibre-Optic Corona",
        category: Category::LiquidGlass,
        shelf: 8,
        family: Family::LiquidGlass,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 69,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 34,
        theme: ClusterTheme::ApertureNine,
        snake: "aperture_nine",
        display: "Aperture Nine",
        category: Category::LiquidGlass,
        shelf: 7,
        family: Family::ModernDigital,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 53,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 35,
        theme: ClusterTheme::AeroSmokeTunnel,
        snake: "aero_smoke_tunnel",
        display: "Aero Smoke Tunnel",
        category: Category::Psychedelic,
        shelf: 8,
        family: Family::ModernDigital,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 59,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 36,
        theme: ClusterTheme::RattrapanteSector,
        snake: "rattrapante_sector",
        display: "Rattrapante Sector",
        category: Category::Hypercar,
        shelf: 7,
        family: Family::ClassicAnalog,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 61,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 37,
        theme: ClusterTheme::SapphireMystery,
        snake: "sapphire_mystery",
        display: "Sapphire Mystery",
        category: Category::LiquidGlass,
        shelf: 4,
        family: Family::ClassicAnalog,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 35,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 38,
        theme: ClusterTheme::RackFocus,
        snake: "rack_focus",
        display: "Rack Focus",
        category: Category::Motorsport,
        shelf: 8,
        family: Family::RaceDash,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 58,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 39,
        theme: ClusterTheme::BionicSpaceframe,
        snake: "bionic_spaceframe",
        display: "Bionic Spaceframe",
        category: Category::Hypercar,
        shelf: 8,
        family: Family::RaceDash,
        kind: Kind::New,
        target: Target::Inspired,
        views: Views::COCKPIT,
        build_order: 68,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 40,
        theme: ClusterTheme::GeodesicWell,
        snake: "geodesic_well",
        display: "Geodesic Well",
        category: Category::AppleVision,
        shelf: 7,
        family: Family::AdasHud,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 45,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 41,
        theme: ClusterTheme::Lightpainter,
        snake: "lightpainter",
        display: "Lightpainter",
        category: Category::AppleVision,
        shelf: 6,
        family: Family::AdasHud,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 54,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 42,
        theme: ClusterTheme::TreadSeismograph,
        snake: "tread_seismograph",
        display: "Tread Seismograph",
        category: Category::AppleVision,
        shelf: 8,
        family: Family::AdasHud,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 63,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 43,
        theme: ClusterTheme::StarSailOrigami,
        snake: "star_sail_origami",
        display: "Star Sail Origami",
        category: Category::AppleVision,
        shelf: 4,
        family: Family::SciFi,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 70,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 44,
        theme: ClusterTheme::ArcLadderCascade,
        snake: "arc_ladder_cascade",
        display: "Arc Ladder Cascade",
        category: Category::SciFi,
        shelf: 8,
        family: Family::SciFi,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 71,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 45,
        theme: ClusterTheme::AuroraVeil,
        snake: "aurora_veil",
        display: "Aurora Veil",
        category: Category::Psychedelic,
        shelf: 4,
        family: Family::SciFi,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 32,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 46,
        theme: ClusterTheme::IsochromaticLoadPath,
        snake: "isochromatic_load_path",
        display: "Isochromatic Load Path",
        category: Category::Psychedelic,
        shelf: 6,
        family: Family::FluidGenerative,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 50,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 47,
        theme: ClusterTheme::AbyssalShearBloom,
        snake: "abyssal_shear_bloom",
        display: "Abyssal Shear Bloom",
        category: Category::Psychedelic,
        shelf: 7,
        family: Family::FluidGenerative,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 66,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 48,
        theme: ClusterTheme::HarmonographDrive,
        snake: "harmonograph_drive",
        display: "Harmonograph Drive",
        category: Category::Psychedelic,
        shelf: 5,
        family: Family::FluidGenerative,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 41,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 49,
        theme: ClusterTheme::AmgMbuxSupersport,
        snake: "amg_mbux_supersport",
        display: "AMG MBUX Supersport",
        category: Category::Supercar,
        shelf: 1,
        family: Family::ModernDigital,
        kind: Kind::New,
        target: Target::Recreation,
        views: Views::COCKPIT,
        build_order: 11,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 50,
        theme: ClusterTheme::SkylineR34Mfd,
        snake: "skyline_r34_mfd",
        display: "Skyline R34 MFD",
        category: Category::Retro,
        shelf: 2,
        family: Family::ClassicAnalog,
        kind: Kind::New,
        target: Target::Inspired,
        views: Views::COCKPIT,
        build_order: 12,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 51,
        theme: ClusterTheme::VisionPetals,
        snake: "vision_petals",
        display: "Vision Petals",
        category: Category::Abstract,
        shelf: 1,
        family: Family::LiquidGlass,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 15,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 52,
        theme: ClusterTheme::AmgOneF1,
        snake: "amg_one_f1",
        display: "AMG ONE F1",
        category: Category::Hypercar,
        shelf: 2,
        family: Family::RaceDash,
        kind: Kind::New,
        target: Target::Recreation,
        views: Views::COCKPIT,
        build_order: 16,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 53,
        theme: ClusterTheme::CarplayIsland,
        snake: "carplay_island",
        display: "CarPlay Island",
        category: Category::AppleVision,
        shelf: 2,
        family: Family::LiquidGlass,
        kind: Kind::New,
        target: Target::Inspired,
        views: Views::COCKPIT,
        build_order: 18,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 54,
        theme: ClusterTheme::Gt3RsApogee,
        snake: "gt3_rs_apogee",
        display: "GT3 RS Apogee",
        category: Category::Supercar,
        shelf: 2,
        family: Family::ClassicAnalog,
        kind: Kind::New,
        target: Target::Inspired,
        views: Views::COCKPIT,
        build_order: 20,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 55,
        theme: ClusterTheme::Gt7CockpitPro,
        snake: "gt7_cockpit_pro",
        display: "GT7 Cockpit Pro",
        category: Category::Motorsport,
        shelf: 3,
        family: Family::ModernDigital,
        kind: Kind::New,
        target: Target::Inspired,
        views: Views::COCKPIT,
        build_order: 22,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 56,
        theme: ClusterTheme::LiquidMorph,
        snake: "liquid_morph",
        display: "Liquid Morph",
        category: Category::Abstract,
        shelf: 2,
        family: Family::LiquidGlass,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 24,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 57,
        theme: ClusterTheme::AmgMbuxTrackPace,
        snake: "amg_mbux_track_pace",
        display: "AMG MBUX Track Pace",
        category: Category::Supercar,
        shelf: 7,
        family: Family::ModernDigital,
        kind: Kind::New,
        target: Target::Recreation,
        views: Views::BOTH,
        build_order: 29,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 58,
        theme: ClusterTheme::GtrNismoMfd,
        snake: "gtr_nismo_mfd",
        display: "GT-R Nismo MFD",
        category: Category::Retro,
        shelf: 6,
        family: Family::ModernDigital,
        kind: Kind::New,
        target: Target::Inspired,
        views: Views::COCKPIT,
        build_order: 30,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 59,
        theme: ClusterTheme::SimInputTrace,
        snake: "sim_input_trace",
        display: "Sim Input Trace",
        category: Category::Motorsport,
        shelf: 6,
        family: Family::RaceDash,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 31,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 60,
        theme: ClusterTheme::LorenzButterfly,
        snake: "lorenz_butterfly",
        display: "Lorenz Butterfly",
        category: Category::Abstract,
        shelf: 3,
        family: Family::FluidGenerative,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 33,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 61,
        theme: ClusterTheme::AmgMbuxClassic,
        snake: "amg_mbux_classic",
        display: "AMG MBUX Classic",
        category: Category::Supercar,
        shelf: 5,
        family: Family::ModernDigital,
        kind: Kind::New,
        target: Target::Recreation,
        views: Views::COCKPIT,
        build_order: 38,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 62,
        theme: ClusterTheme::SuminagashiMarble,
        snake: "suminagashi_marble",
        display: "Suminagashi Marble",
        category: Category::Abstract,
        shelf: 4,
        family: Family::FluidGenerative,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 42,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 63,
        theme: ClusterTheme::T50Floodlit,
        snake: "t50_floodlit",
        display: "T.50 Floodlit",
        category: Category::Hypercar,
        shelf: 6,
        family: Family::ClassicAnalog,
        kind: Kind::New,
        target: Target::Inspired,
        views: Views::COCKPIT,
        build_order: 43,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 64,
        theme: ClusterTheme::HakosukaS20,
        snake: "hakosuka_s20",
        display: "Hakosuka S20",
        category: Category::Retro,
        shelf: 4,
        family: Family::ClassicAnalog,
        kind: Kind::New,
        target: Target::Inspired,
        views: Views::COCKPIT,
        build_order: 48,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 65,
        theme: ClusterTheme::SimPitWall,
        snake: "sim_pit_wall",
        display: "Sim Pit Wall",
        category: Category::Motorsport,
        shelf: 4,
        family: Family::RaceDash,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 49,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 66,
        theme: ClusterTheme::KaleidoscopeMirror,
        snake: "kaleidoscope_mirror",
        display: "Kaleidoscope Mirror",
        category: Category::Abstract,
        shelf: 5,
        family: Family::FluidGenerative,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 51,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 67,
        theme: ClusterTheme::RevueltoCorsa,
        snake: "revuelto_corsa",
        display: "Revuelto Corsa",
        category: Category::Supercar,
        shelf: 6,
        family: Family::ModernDigital,
        kind: Kind::New,
        target: Target::Inspired,
        views: Views::COCKPIT,
        build_order: 56,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 68,
        theme: ClusterTheme::ZNismoSport,
        snake: "z_nismo_sport",
        display: "Z Nismo Sport",
        category: Category::Retro,
        shelf: 8,
        family: Family::ModernDigital,
        kind: Kind::New,
        target: Target::Inspired,
        views: Views::COCKPIT,
        build_order: 57,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 69,
        theme: ClusterTheme::GoldenAngle,
        snake: "golden_angle",
        display: "Golden Angle",
        category: Category::Abstract,
        shelf: 6,
        family: Family::FluidGenerative,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::BOTH,
        build_order: 60,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 70,
        theme: ClusterTheme::MclarenSlimDisplay,
        snake: "mclaren_slim_display",
        display: "McLaren Slim Display",
        category: Category::Supercar,
        shelf: 8,
        family: Family::ModernDigital,
        kind: Kind::New,
        target: Target::Inspired,
        views: Views::BOTH,
        build_order: 65,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 71,
        theme: ClusterTheme::HopfFibration,
        snake: "hopf_fibration",
        display: "Hopf Fibration",
        category: Category::Abstract,
        shelf: 7,
        family: Family::FluidGenerative,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 67,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 72,
        theme: ClusterTheme::HyperbolicTiles,
        snake: "hyperbolic_tiles",
        display: "Hyperbolic Tiles",
        category: Category::Abstract,
        shelf: 8,
        family: Family::FluidGenerative,
        kind: Kind::New,
        target: Target::Concept,
        views: Views::COCKPIT,
        build_order: 72,
        pattern_setter: false,
    },
];

/// The catalog row for `theme` (every theme has one).
#[must_use]
pub fn entry(theme: ClusterTheme) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|e| e.theme == theme)
}

/// The display name of `theme` (its catalog row's; the config name if it had none).
#[must_use]
pub fn display_name(theme: ClusterTheme) -> &'static str {
    entry(theme).map_or_else(|| theme.name(), |e| e.display)
}

/// The row whose theme name (`LiquidGlass`), module name (`liquid_glass`) or display name
/// matches `name`, ignoring ASCII case.
#[must_use]
pub fn find(name: &str) -> Option<&'static CatalogEntry> {
    let name = name.trim();
    CATALOG.iter().find(|e| {
        e.theme.name().eq_ignore_ascii_case(name)
            || e.snake.eq_ignore_ascii_case(name)
            || e.display.eq_ignore_ascii_case(name)
    })
}

/// The rows sorted by build order (the pilot first).
#[must_use]
pub fn in_build_order() -> [&'static CatalogEntry; COUNT] {
    let mut out = [&CATALOG[0]; COUNT];
    for e in &CATALOG {
        out[usize::from(e.build_order) - 1] = e;
    }
    out
}

/// The one browse order: category in TJX's order, then shelf (the configurator's listing and the
/// `preview-export` gallery).
#[must_use]
pub fn in_browse_order() -> [&'static CatalogEntry; COUNT] {
    let mut out = [&CATALOG[0]; COUNT];
    for e in &CATALOG {
        let slot = e.category as usize * usize::from(PER_CATEGORY) + usize::from(e.shelf) - 1;
        out[slot] = e;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn every_theme_has_exactly_one_row() {
        assert_eq!(CATALOG.len(), ClusterTheme::all().len());
        for t in ClusterTheme::all() {
            assert_eq!(CATALOG.iter().filter(|e| e.theme == *t).count(), 1, "{t}");
        }
    }

    #[test]
    fn rows_follow_the_theme_values() {
        for (i, e) in CATALOG.iter().enumerate() {
            assert_eq!(usize::from(e.row), i + 1, "{}", e.theme);
        }
        // Rows 30 to 72 were appended in theme order; the first 29 keep their v1 values.
        for e in &CATALOG[29..] {
            assert_eq!(usize::from(e.row), e.theme as usize + 1, "{}", e.theme);
        }
    }

    #[test]
    fn rows_and_build_order_are_permutations() {
        let mut rows = [false; COUNT];
        let mut order = [false; COUNT];
        for e in &CATALOG {
            rows[usize::from(e.row) - 1] = true;
            order[usize::from(e.build_order) - 1] = true;
        }
        assert!(rows.iter().all(|&x| x) && order.iter().all(|&x| x));
        let first = in_build_order();
        assert_eq!(first[0].theme, ClusterTheme::LiquidGlass);
        // The seven pattern-setters hold builds 1 to 7.
        assert!(first[..7].iter().all(|e| e.pattern_setter));
        assert_eq!(CATALOG.iter().filter(|e| e.pattern_setter).count(), 7);
    }

    #[test]
    fn every_family_has_one_pattern_setter_built_before_its_siblings() {
        for e in &CATALOG {
            let setters: Vec<_> = CATALOG
                .iter()
                .filter(|s| s.family == e.family && s.pattern_setter)
                .collect();
            assert_eq!(setters.len(), 1, "{:?}", e.family);
            assert!(setters[0].build_order <= e.build_order, "{} before its ★", e.theme);
        }
    }

    #[test]
    fn categories_hold_eight_dashes_on_shelves_one_to_eight() {
        for c in Category::ALL {
            let mut shelves = [0u8; PER_CATEGORY as usize];
            for e in CATALOG.iter().filter(|e| e.category == c) {
                assert!((1..=PER_CATEGORY).contains(&e.shelf), "{} shelf {}", e.theme, e.shelf);
                shelves[usize::from(e.shelf) - 1] += 1;
            }
            assert_eq!(shelves, [1; PER_CATEGORY as usize], "{c:?}");
        }
        assert_eq!(Category::ALL.len() * usize::from(PER_CATEGORY), COUNT);
    }

    #[test]
    fn browse_order_is_category_then_shelf() {
        let order = in_browse_order();
        assert_eq!(order[0].theme, ClusterTheme::AmgMbuxSupersport);
        assert_eq!(order[COUNT - 1].theme, ClusterTheme::HyperbolicTiles);
        for pair in order.windows(2) {
            assert!(
                (pair[0].category, pair[0].shelf) < (pair[1].category, pair[1].shelf),
                "{} then {}",
                pair[0].theme,
                pair[1].theme
            );
        }
    }

    #[test]
    fn the_re_cut_dashes_are_new_and_follow_their_build_order() {
        let added = &CATALOG[48..];
        assert_eq!(added.len(), 24);
        assert!(added.iter().all(|e| e.kind == Kind::New && !e.pattern_setter));
        // Theme values and catalog rows follow the build order of the added dashes.
        for pair in added.windows(2) {
            assert!(pair[0].build_order < pair[1].build_order, "{}", pair[1].theme);
        }
        assert_eq!(CATALOG.iter().filter(|e| e.category == Category::Abstract).count(), 8);
        assert!(CATALOG
            .iter()
            .filter(|e| e.category == Category::Abstract)
            .all(|e| e.row > 48));
    }

    #[test]
    fn display_names_are_unique_and_none_starts_another() {
        for a in &CATALOG {
            for b in &CATALOG {
                if a.theme == b.theme {
                    continue;
                }
                let (x, y) = (a.display.to_ascii_lowercase(), b.display.to_ascii_lowercase());
                assert!(!y.starts_with(&x), "{:?} starts {:?}", a.display, b.display);
            }
        }
        assert_eq!(display_name(ClusterTheme::AmgMbuxSuperscreen), "AMG MBUX Superscreen");
    }

    #[test]
    fn snake_names_match_the_theme_names() {
        for e in &CATALOG {
            let joined: String = e.snake.split('_').collect();
            assert!(
                joined.eq_ignore_ascii_case(e.theme.name()),
                "{} vs {}",
                e.snake,
                e.theme
            );
            assert_eq!(find(e.snake).map(|f| f.theme), Some(e.theme));
            assert_eq!(find(e.theme.name()).map(|f| f.theme), Some(e.theme));
            assert_eq!(find(e.display).map(|f| f.theme), Some(e.theme));
        }
    }

    /// A scaffold module names no build number, so a change of order touches only CATALOG.md and
    /// this table; each holds its brief and README.
    #[test]
    fn scaffold_modules_name_no_build_number() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/clusters");
        for e in &CATALOG {
            let dir = root.join(e.snake);
            let module = std::fs::read_to_string(dir.join("mod.rs")).unwrap_or_default();
            assert!(!module.is_empty(), "{} has no mod.rs", e.snake);
            if !module.contains("common::scaffold::Scaffold") {
                continue;
            }
            for file in ["mod.rs", "README.md", "spec.md"] {
                let text = std::fs::read_to_string(dir.join(file)).unwrap_or_default();
                assert!(!text.is_empty(), "{}/{file} is missing", e.snake);
                assert!(
                    !text.contains("build #") && !text.contains("Build order:"),
                    "{}/{file} names a build number",
                    e.snake
                );
                assert!(text.contains(e.theme.name()), "{}/{file}", e.snake);
            }
        }
    }
}
