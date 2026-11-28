//! The Phase 7 cluster catalog as data: one row per cluster in
//! `program/phases/07-cluster-factory/CATALOG.md` (29 rebuilds and 19 new concepts), with its
//! module name, family, views, build order and pattern-setter flag.
//!
//! The registry, the golden tests, `preview-export --cluster` and `FH6_WheelDash --cluster`
//! all read this table, so a cluster chat never edits a shared file. Status lives in CATALOG.md.
use fh6_core::ClusterTheme;

pub use fh6_kit::Family;

/// Whether the catalog row rebuilds a v1 theme or is a new concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// One of the 29 v1 themes, which Phase 7 rebuilds on the kit (the v1 source is at commit
    /// `21e8a06`).
    Rebuild,
    /// A next-gen concept new in Phase 7.
    New,
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
    /// Row number in CATALOG.md (1..=48).
    pub row: u8,
    /// The config identifier.
    pub theme: ClusterTheme,
    /// Module directory: `crates/fh6-clusters/src/clusters/<snake>/`.
    pub snake: &'static str,
    /// Display name (OSD, gallery).
    pub display: &'static str,
    /// Design family.
    pub family: Family,
    /// Rebuild or new concept.
    pub kind: Kind,
    /// Views it is designed for.
    pub views: Views,
    /// Position in the build order (1 = the pilot).
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

/// Every catalog row, in CATALOG.md order.
pub static CATALOG: [CatalogEntry; 48] = [
    CatalogEntry {
        row: 1,
        theme: ClusterTheme::LiquidGlass,
        snake: "liquid_glass",
        display: "Liquid Glass",
        family: Family::LiquidGlass,
        kind: Kind::Rebuild,
        views: Views::BOTH,
        build_order: 1,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 2,
        theme: ClusterTheme::VisionSpatialGlass,
        snake: "vision_spatial_glass",
        display: "Vision Spatial Glass",
        family: Family::LiquidGlass,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 8,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 3,
        theme: ClusterTheme::Gt7HoodPro,
        snake: "gt7_hood_pro",
        display: "Bonnet Line",
        family: Family::ModernDigital,
        kind: Kind::Rebuild,
        views: Views::HOOD,
        build_order: 2,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 4,
        theme: ClusterTheme::AmgMbuxSuperscreen,
        snake: "amg_mbux_superscreen",
        display: "AMG MBUX Superscreen",
        family: Family::ModernDigital,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 9,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 5,
        theme: ClusterTheme::DriftGlass,
        snake: "drift_glass",
        display: "Drift Glass",
        family: Family::ModernDigital,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 10,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 6,
        theme: ClusterTheme::GlyphDotMatrix,
        snake: "glyph_dot_matrix",
        display: "Glyph Dot Matrix",
        family: Family::ModernDigital,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 11,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 7,
        theme: ClusterTheme::PorscheAnalog,
        snake: "porsche_analog",
        display: "Porsche Analog",
        family: Family::ClassicAnalog,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 3,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 8,
        theme: ClusterTheme::PaganiHorologerie,
        snake: "pagani_horologerie",
        display: "Pagani Horologerie",
        family: Family::ClassicAnalog,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 13,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 9,
        theme: ClusterTheme::BugattiTourbillon,
        snake: "bugatti_tourbillon",
        display: "Bugatti Tourbillon",
        family: Family::ClassicAnalog,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 14,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 10,
        theme: ClusterTheme::LfaV10Nurburgring,
        snake: "lfa_v10_nurburgring",
        display: "LFA V10 Nürburgring",
        family: Family::ClassicAnalog,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 12,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 11,
        theme: ClusterTheme::KoenigseggGyro,
        snake: "koenigsegg_gyro",
        display: "Koenigsegg Gyro",
        family: Family::ClassicAnalog,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 15,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 12,
        theme: ClusterTheme::BoschDdu,
        snake: "bosch_ddu",
        display: "Bosch DDU",
        family: Family::RaceDash,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 4,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 13,
        theme: ClusterTheme::ValkyrieLmp1,
        snake: "valkyrie_lmp1",
        display: "Valkyrie LMP1",
        family: Family::RaceDash,
        kind: Kind::Rebuild,
        views: Views::BOTH,
        build_order: 16,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 14,
        theme: ClusterTheme::Gt7ModularTripod,
        snake: "gt7_modular_tripod",
        display: "GT7 Modular Tripod",
        family: Family::RaceDash,
        kind: Kind::Rebuild,
        views: Views::BOTH,
        build_order: 17,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 15,
        theme: ClusterTheme::TeslaAdas,
        snake: "tesla_adas",
        display: "Tesla ADAS",
        family: Family::AdasHud,
        kind: Kind::Rebuild,
        views: Views::BOTH,
        build_order: 5,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 16,
        theme: ClusterTheme::WindshieldHud,
        snake: "windshield_hud",
        display: "Windshield HUD",
        family: Family::AdasHud,
        kind: Kind::Rebuild,
        views: Views::RAISED,
        build_order: 18,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 17,
        theme: ClusterTheme::ArNavWayfinder,
        snake: "ar_nav_wayfinder",
        display: "AR Nav Wayfinder",
        family: Family::AdasHud,
        kind: Kind::Rebuild,
        views: Views::BOTH,
        build_order: 20,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 18,
        theme: ClusterTheme::F35HelmetHmd,
        snake: "f35_helmet_hmd",
        display: "F-35 Helmet HMD",
        family: Family::AdasHud,
        kind: Kind::Rebuild,
        views: Views::BOTH,
        build_order: 19,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 19,
        theme: ClusterTheme::RetroCyber,
        snake: "retro_cyber",
        display: "Retro Cyber",
        family: Family::SciFi,
        kind: Kind::Rebuild,
        views: Views::BOTH,
        build_order: 6,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 20,
        theme: ClusterTheme::SynthwaveOutrun,
        snake: "synthwave_outrun",
        display: "Synthwave Outrun",
        family: Family::SciFi,
        kind: Kind::Rebuild,
        views: Views::BOTH,
        build_order: 21,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 21,
        theme: ClusterTheme::EvaBioSync,
        snake: "eva_bio_sync",
        display: "EVA Bio-Sync",
        family: Family::SciFi,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 22,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 22,
        theme: ClusterTheme::ChronoEventHorizon,
        snake: "chrono_event_horizon",
        display: "Chrono Event Horizon",
        family: Family::SciFi,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 27,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 23,
        theme: ClusterTheme::NeuralCyberSynapse,
        snake: "neural_cyber_synapse",
        display: "Neural Cyber Synapse",
        family: Family::SciFi,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 23,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 24,
        theme: ClusterTheme::SolarisWarpCore,
        snake: "solaris_warp_core",
        display: "Solaris Warp Core",
        family: Family::SciFi,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 26,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 25,
        theme: ClusterTheme::AeroPlasmaBlade,
        snake: "aero_plasma_blade",
        display: "Aero Plasma Blade",
        family: Family::SciFi,
        kind: Kind::Rebuild,
        views: Views::HOOD,
        build_order: 25,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 26,
        theme: ClusterTheme::ShibuyaChromaticPrisma,
        snake: "shibuya_chromatic_prisma",
        display: "Shibuya Chromatic Prisma",
        family: Family::SciFi,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 24,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 27,
        theme: ClusterTheme::KineticLiquidMetal,
        snake: "kinetic_liquid_metal",
        display: "Kinetic Liquid Metal",
        family: Family::FluidGenerative,
        kind: Kind::Rebuild,
        views: Views::COCKPIT,
        build_order: 7,
        pattern_setter: true,
    },
    CatalogEntry {
        row: 28,
        theme: ClusterTheme::CymaticResonanceField,
        snake: "cymatic_resonance_field",
        display: "Cymatic Resonance Field",
        family: Family::FluidGenerative,
        kind: Kind::Rebuild,
        views: Views::BOTH,
        build_order: 28,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 29,
        theme: ClusterTheme::ChronoMultiverse,
        snake: "chrono_multiverse",
        display: "Chrono Multiverse",
        family: Family::FluidGenerative,
        kind: Kind::Rebuild,
        views: Views::BOTH,
        build_order: 29,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 30,
        theme: ClusterTheme::LumenEtch,
        snake: "lumen_etch",
        display: "Lumen Etch",
        family: Family::LiquidGlass,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 43,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 31,
        theme: ClusterTheme::VernierLoupe,
        snake: "vernier_loupe",
        display: "Vernier Loupe",
        family: Family::LiquidGlass,
        kind: Kind::New,
        views: Views::BOTH,
        build_order: 30,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 32,
        theme: ClusterTheme::DichroicLouvre,
        snake: "dichroic_louvre",
        display: "Dichroic Louvre",
        family: Family::LiquidGlass,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 37,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 33,
        theme: ClusterTheme::FibreOpticCorona,
        snake: "fibre_optic_corona",
        display: "Fibre-Optic Corona",
        family: Family::LiquidGlass,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 48,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 34,
        theme: ClusterTheme::ApertureNine,
        snake: "aperture_nine",
        display: "Aperture Nine",
        family: Family::ModernDigital,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 31,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 35,
        theme: ClusterTheme::AeroSmokeTunnel,
        snake: "aero_smoke_tunnel",
        display: "Aero Smoke Tunnel",
        family: Family::ModernDigital,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 39,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 36,
        theme: ClusterTheme::RattrapanteSector,
        snake: "rattrapante_sector",
        display: "Rattrapante Sector",
        family: Family::ClassicAnalog,
        kind: Kind::New,
        views: Views::BOTH,
        build_order: 32,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 37,
        theme: ClusterTheme::SapphireMystery,
        snake: "sapphire_mystery",
        display: "Sapphire Mystery",
        family: Family::ClassicAnalog,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 38,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 38,
        theme: ClusterTheme::RackFocus,
        snake: "rack_focus",
        display: "Rack Focus",
        family: Family::RaceDash,
        kind: Kind::New,
        views: Views::BOTH,
        build_order: 33,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 39,
        theme: ClusterTheme::BionicSpaceframe,
        snake: "bionic_spaceframe",
        display: "Bionic Spaceframe",
        family: Family::RaceDash,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 44,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 40,
        theme: ClusterTheme::GeodesicWell,
        snake: "geodesic_well",
        display: "Geodesic Well",
        family: Family::AdasHud,
        kind: Kind::New,
        views: Views::BOTH,
        build_order: 34,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 41,
        theme: ClusterTheme::Lightpainter,
        snake: "lightpainter",
        display: "Lightpainter",
        family: Family::AdasHud,
        kind: Kind::New,
        views: Views::BOTH,
        build_order: 40,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 42,
        theme: ClusterTheme::TreadSeismograph,
        snake: "tread_seismograph",
        display: "Tread Seismograph",
        family: Family::AdasHud,
        kind: Kind::New,
        views: Views::BOTH,
        build_order: 45,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 43,
        theme: ClusterTheme::StarSailOrigami,
        snake: "star_sail_origami",
        display: "Star Sail Origami",
        family: Family::SciFi,
        kind: Kind::New,
        views: Views::BOTH,
        build_order: 35,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 44,
        theme: ClusterTheme::ArcLadderCascade,
        snake: "arc_ladder_cascade",
        display: "Arc Ladder Cascade",
        family: Family::SciFi,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 46,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 45,
        theme: ClusterTheme::AuroraVeil,
        snake: "aurora_veil",
        display: "Aurora Veil",
        family: Family::SciFi,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 41,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 46,
        theme: ClusterTheme::IsochromaticLoadPath,
        snake: "isochromatic_load_path",
        display: "Isochromatic Load Path",
        family: Family::FluidGenerative,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 36,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 47,
        theme: ClusterTheme::AbyssalShearBloom,
        snake: "abyssal_shear_bloom",
        display: "Abyssal Shear Bloom",
        family: Family::FluidGenerative,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 42,
        pattern_setter: false,
    },
    CatalogEntry {
        row: 48,
        theme: ClusterTheme::HarmonographDrive,
        snake: "harmonograph_drive",
        display: "Harmonograph Drive",
        family: Family::FluidGenerative,
        kind: Kind::New,
        views: Views::COCKPIT,
        build_order: 47,
        pattern_setter: false,
    },
];

/// The catalog row for `theme` (every theme has one).
#[must_use]
pub fn entry(theme: ClusterTheme) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|e| e.theme == theme)
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
pub fn in_build_order() -> [&'static CatalogEntry; 48] {
    let mut out = [&CATALOG[0]; 48];
    for e in &CATALOG {
        out[usize::from(e.build_order) - 1] = e;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_has_exactly_one_row() {
        assert_eq!(CATALOG.len(), ClusterTheme::all().len());
        for t in ClusterTheme::all() {
            assert_eq!(CATALOG.iter().filter(|e| e.theme == *t).count(), 1, "{t}");
        }
    }

    #[test]
    fn rows_and_build_order_are_permutations() {
        let mut rows = [false; 48];
        let mut order = [false; 48];
        for e in &CATALOG {
            rows[usize::from(e.row) - 1] = true;
            order[usize::from(e.build_order) - 1] = true;
        }
        assert!(rows.iter().all(|&x| x) && order.iter().all(|&x| x));
        let first = in_build_order();
        assert_eq!(first[0].theme, ClusterTheme::LiquidGlass);
        // The seven pattern-setters lead the order.
        assert!(first[..7].iter().all(|e| e.pattern_setter));
        assert_eq!(CATALOG.iter().filter(|e| e.pattern_setter).count(), 7);
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
        }
    }
}
