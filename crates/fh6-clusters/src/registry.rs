//! The cluster registry (contract v2): one row per catalog cluster, keyed by [`ClusterTheme`].
//!
//! Generated once by the Phase 7 setup chat from [`crate::catalog::CATALOG`]; every row points at
//! `clusters::<snake>::make`, so a cluster chat changes only its own module and never this table.
//! The two Phase 5 kit samples ([`crate::kit`]) are no longer registered; they stay as guide
//! examples.
use crate::clusters;
use fh6_core::ClusterTheme;
use fh6_kit::cluster::{find_entry, ClusterEntry, Setup};
use fh6_kit::Cluster;

/// Every cluster, in catalog order.
pub static REGISTRY: &[ClusterEntry] = &[
    ClusterEntry {
        theme: ClusterTheme::LiquidGlass,
        name: "Liquid Glass",
        family: "liquidGlass",
        make: clusters::liquid_glass::make,
    },
    ClusterEntry {
        theme: ClusterTheme::VisionSpatialGlass,
        name: "Vision Spatial Glass",
        family: "liquidGlass",
        make: clusters::vision_spatial_glass::make,
    },
    ClusterEntry {
        theme: ClusterTheme::Gt7HoodPro,
        name: "GT7 Hood Pro",
        family: "modernDigital",
        make: clusters::gt7_hood_pro::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AmgMbuxSuperscreen,
        name: "AMG MBUX Superscreen",
        family: "modernDigital",
        make: clusters::amg_mbux_superscreen::make,
    },
    ClusterEntry {
        theme: ClusterTheme::DriftGlass,
        name: "Drift Glass",
        family: "modernDigital",
        make: clusters::drift_glass::make,
    },
    ClusterEntry {
        theme: ClusterTheme::GlyphDotMatrix,
        name: "Glyph Dot Matrix",
        family: "modernDigital",
        make: clusters::glyph_dot_matrix::make,
    },
    ClusterEntry {
        theme: ClusterTheme::PorscheAnalog,
        name: "Porsche Analog",
        family: "classicAnalog",
        make: clusters::porsche_analog::make,
    },
    ClusterEntry {
        theme: ClusterTheme::PaganiHorologerie,
        name: "Pagani Horologerie",
        family: "classicAnalog",
        make: clusters::pagani_horologerie::make,
    },
    ClusterEntry {
        theme: ClusterTheme::BugattiTourbillon,
        name: "Bugatti Tourbillon",
        family: "classicAnalog",
        make: clusters::bugatti_tourbillon::make,
    },
    ClusterEntry {
        theme: ClusterTheme::LfaV10Nurburgring,
        name: "LFA V10 Nürburgring",
        family: "classicAnalog",
        make: clusters::lfa_v10_nurburgring::make,
    },
    ClusterEntry {
        theme: ClusterTheme::KoenigseggGyro,
        name: "Koenigsegg Gyro",
        family: "classicAnalog",
        make: clusters::koenigsegg_gyro::make,
    },
    ClusterEntry {
        theme: ClusterTheme::BoschDdu,
        name: "Bosch DDU",
        family: "raceDash",
        make: clusters::bosch_ddu::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ValkyrieLmp1,
        name: "Valkyrie LMP1",
        family: "raceDash",
        make: clusters::valkyrie_lmp1::make,
    },
    ClusterEntry {
        theme: ClusterTheme::Gt7ModularTripod,
        name: "GT7 Modular Tripod",
        family: "raceDash",
        make: clusters::gt7_modular_tripod::make,
    },
    ClusterEntry {
        theme: ClusterTheme::TeslaAdas,
        name: "Tesla ADAS",
        family: "adasHud",
        make: clusters::tesla_adas::make,
    },
    ClusterEntry {
        theme: ClusterTheme::WindshieldHud,
        name: "Windshield HUD",
        family: "adasHud",
        make: clusters::windshield_hud::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ArNavWayfinder,
        name: "AR Nav Wayfinder",
        family: "adasHud",
        make: clusters::ar_nav_wayfinder::make,
    },
    ClusterEntry {
        theme: ClusterTheme::F35HelmetHmd,
        name: "F-35 Helmet HMD",
        family: "adasHud",
        make: clusters::f35_helmet_hmd::make,
    },
    ClusterEntry {
        theme: ClusterTheme::RetroCyber,
        name: "Retro Cyber",
        family: "sciFi",
        make: clusters::retro_cyber::make,
    },
    ClusterEntry {
        theme: ClusterTheme::SynthwaveOutrun,
        name: "Synthwave Outrun",
        family: "sciFi",
        make: clusters::synthwave_outrun::make,
    },
    ClusterEntry {
        theme: ClusterTheme::EvaBioSync,
        name: "EVA Bio-Sync",
        family: "sciFi",
        make: clusters::eva_bio_sync::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ChronoEventHorizon,
        name: "Chrono Event Horizon",
        family: "sciFi",
        make: clusters::chrono_event_horizon::make,
    },
    ClusterEntry {
        theme: ClusterTheme::NeuralCyberSynapse,
        name: "Neural Cyber Synapse",
        family: "sciFi",
        make: clusters::neural_cyber_synapse::make,
    },
    ClusterEntry {
        theme: ClusterTheme::SolarisWarpCore,
        name: "Solaris Warp Core",
        family: "sciFi",
        make: clusters::solaris_warp_core::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AeroPlasmaBlade,
        name: "Aero Plasma Blade",
        family: "sciFi",
        make: clusters::aero_plasma_blade::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ShibuyaChromaticPrisma,
        name: "Shibuya Chromatic Prisma",
        family: "sciFi",
        make: clusters::shibuya_chromatic_prisma::make,
    },
    ClusterEntry {
        theme: ClusterTheme::KineticLiquidMetal,
        name: "Kinetic Liquid Metal",
        family: "fluidGenerative",
        make: clusters::kinetic_liquid_metal::make,
    },
    ClusterEntry {
        theme: ClusterTheme::CymaticResonanceField,
        name: "Cymatic Resonance Field",
        family: "fluidGenerative",
        make: clusters::cymatic_resonance_field::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ChronoMultiverse,
        name: "Chrono Multiverse",
        family: "fluidGenerative",
        make: clusters::chrono_multiverse::make,
    },
    ClusterEntry {
        theme: ClusterTheme::LumenEtch,
        name: "Lumen Etch",
        family: "liquidGlass",
        make: clusters::lumen_etch::make,
    },
    ClusterEntry {
        theme: ClusterTheme::VernierLoupe,
        name: "Vernier Loupe",
        family: "liquidGlass",
        make: clusters::vernier_loupe::make,
    },
    ClusterEntry {
        theme: ClusterTheme::DichroicLouvre,
        name: "Dichroic Louvre",
        family: "liquidGlass",
        make: clusters::dichroic_louvre::make,
    },
    ClusterEntry {
        theme: ClusterTheme::FibreOpticCorona,
        name: "Fibre-Optic Corona",
        family: "liquidGlass",
        make: clusters::fibre_optic_corona::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ApertureNine,
        name: "Aperture Nine",
        family: "modernDigital",
        make: clusters::aperture_nine::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AeroSmokeTunnel,
        name: "Aero Smoke Tunnel",
        family: "modernDigital",
        make: clusters::aero_smoke_tunnel::make,
    },
    ClusterEntry {
        theme: ClusterTheme::RattrapanteSector,
        name: "Rattrapante Sector",
        family: "classicAnalog",
        make: clusters::rattrapante_sector::make,
    },
    ClusterEntry {
        theme: ClusterTheme::SapphireMystery,
        name: "Sapphire Mystery",
        family: "classicAnalog",
        make: clusters::sapphire_mystery::make,
    },
    ClusterEntry {
        theme: ClusterTheme::RackFocus,
        name: "Rack Focus",
        family: "raceDash",
        make: clusters::rack_focus::make,
    },
    ClusterEntry {
        theme: ClusterTheme::BionicSpaceframe,
        name: "Bionic Spaceframe",
        family: "raceDash",
        make: clusters::bionic_spaceframe::make,
    },
    ClusterEntry {
        theme: ClusterTheme::GeodesicWell,
        name: "Geodesic Well",
        family: "adasHud",
        make: clusters::geodesic_well::make,
    },
    ClusterEntry {
        theme: ClusterTheme::Lightpainter,
        name: "Lightpainter",
        family: "adasHud",
        make: clusters::lightpainter::make,
    },
    ClusterEntry {
        theme: ClusterTheme::TreadSeismograph,
        name: "Tread Seismograph",
        family: "adasHud",
        make: clusters::tread_seismograph::make,
    },
    ClusterEntry {
        theme: ClusterTheme::StarSailOrigami,
        name: "Star Sail Origami",
        family: "sciFi",
        make: clusters::star_sail_origami::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ArcLadderCascade,
        name: "Arc Ladder Cascade",
        family: "sciFi",
        make: clusters::arc_ladder_cascade::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AuroraVeil,
        name: "Aurora Veil",
        family: "sciFi",
        make: clusters::aurora_veil::make,
    },
    ClusterEntry {
        theme: ClusterTheme::IsochromaticLoadPath,
        name: "Isochromatic Load Path",
        family: "fluidGenerative",
        make: clusters::isochromatic_load_path::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AbyssalShearBloom,
        name: "Abyssal Shear Bloom",
        family: "fluidGenerative",
        make: clusters::abyssal_shear_bloom::make,
    },
    ClusterEntry {
        theme: ClusterTheme::HarmonographDrive,
        name: "Harmonograph Drive",
        family: "fluidGenerative",
        make: clusters::harmonograph_drive::make,
    },
];

/// The registry row for `theme`.
#[must_use]
pub fn entry(theme: ClusterTheme) -> Option<&'static ClusterEntry> {
    find_entry(REGISTRY, theme)
}

/// The row to use for `theme`: its own (every theme has one), else the pilot.
#[must_use]
pub fn resolve(theme: ClusterTheme) -> &'static ClusterEntry {
    entry(theme)
        .or_else(|| entry(ClusterTheme::LiquidGlass))
        .unwrap_or(&REGISTRY[0])
}

/// Creates the cluster for `theme` (see [`resolve`]).
pub fn create(theme: ClusterTheme, setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    (resolve(theme).make)(setup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::CATALOG;

    #[test]
    fn every_theme_has_its_own_row() {
        assert_eq!(REGISTRY.len(), ClusterTheme::all().len());
        for theme in ClusterTheme::all() {
            assert_eq!(resolve(*theme).theme, *theme, "{theme}");
        }
    }

    #[test]
    fn rows_follow_the_catalog() {
        for (row, cat) in REGISTRY.iter().zip(CATALOG.iter()) {
            assert_eq!(row.theme, cat.theme);
            assert_eq!(row.name, cat.display);
            assert_eq!(row.family, cat.family.key());
        }
    }

    #[test]
    fn themes_are_unique() {
        for (i, a) in REGISTRY.iter().enumerate() {
            assert!(
                REGISTRY[i + 1..].iter().all(|b| b.theme != a.theme),
                "{} twice",
                a.theme
            );
        }
    }
}
