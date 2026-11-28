//! The cluster registry (contract v2): one row per catalog cluster, keyed by [`ClusterTheme`].
//!
//! Generated once by the Phase 7 setup chat from [`crate::catalog::CATALOG`]; every row points at
//! `clusters::<snake>::make`, so a cluster chat changes only its own module and never this table.
//! The two Phase 5 kit samples ([`crate::kit`]) are no longer registered; they stay as guide
//! examples.
use crate::clusters;
use fh6_core::ClusterTheme;
use fh6_kit::cluster::{find_entry, ClusterEntry, Setup};
use fh6_kit::{Cluster, Family};

/// Every cluster, in catalog order.
pub static REGISTRY: &[ClusterEntry] = &[
    ClusterEntry {
        theme: ClusterTheme::LiquidGlass,
        name: "Liquid Glass",
        family: Family::LiquidGlass,
        make: clusters::liquid_glass::make,
    },
    ClusterEntry {
        theme: ClusterTheme::VisionSpatialGlass,
        name: "Vision Spatial Glass",
        family: Family::LiquidGlass,
        make: clusters::vision_spatial_glass::make,
    },
    ClusterEntry {
        theme: ClusterTheme::Gt7HoodPro,
        name: "Bonnet Line",
        family: Family::ModernDigital,
        make: clusters::gt7_hood_pro::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AmgMbuxSuperscreen,
        name: "AMG MBUX Superscreen",
        family: Family::ModernDigital,
        make: clusters::amg_mbux_superscreen::make,
    },
    ClusterEntry {
        theme: ClusterTheme::DriftGlass,
        name: "Drift Glass",
        family: Family::ModernDigital,
        make: clusters::drift_glass::make,
    },
    ClusterEntry {
        theme: ClusterTheme::GlyphDotMatrix,
        name: "Glyph Dot Matrix",
        family: Family::ModernDigital,
        make: clusters::glyph_dot_matrix::make,
    },
    ClusterEntry {
        theme: ClusterTheme::PorscheAnalog,
        name: "Porsche Analog",
        family: Family::ClassicAnalog,
        make: clusters::porsche_analog::make,
    },
    ClusterEntry {
        theme: ClusterTheme::PaganiHorologerie,
        name: "Pagani Horologerie",
        family: Family::ClassicAnalog,
        make: clusters::pagani_horologerie::make,
    },
    ClusterEntry {
        theme: ClusterTheme::BugattiTourbillon,
        name: "Bugatti Tourbillon",
        family: Family::ClassicAnalog,
        make: clusters::bugatti_tourbillon::make,
    },
    ClusterEntry {
        theme: ClusterTheme::LfaV10Nurburgring,
        name: "LFA V10 Nürburgring",
        family: Family::ClassicAnalog,
        make: clusters::lfa_v10_nurburgring::make,
    },
    ClusterEntry {
        theme: ClusterTheme::KoenigseggGyro,
        name: "Koenigsegg Gyro",
        family: Family::ClassicAnalog,
        make: clusters::koenigsegg_gyro::make,
    },
    ClusterEntry {
        theme: ClusterTheme::BoschDdu,
        name: "Bosch DDU",
        family: Family::RaceDash,
        make: clusters::bosch_ddu::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ValkyrieLmp1,
        name: "Valkyrie LMP1",
        family: Family::RaceDash,
        make: clusters::valkyrie_lmp1::make,
    },
    ClusterEntry {
        theme: ClusterTheme::Gt7ModularTripod,
        name: "GT7 Modular Tripod",
        family: Family::RaceDash,
        make: clusters::gt7_modular_tripod::make,
    },
    ClusterEntry {
        theme: ClusterTheme::TeslaAdas,
        name: "Tesla ADAS",
        family: Family::AdasHud,
        make: clusters::tesla_adas::make,
    },
    ClusterEntry {
        theme: ClusterTheme::WindshieldHud,
        name: "Windshield HUD",
        family: Family::AdasHud,
        make: clusters::windshield_hud::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ArNavWayfinder,
        name: "AR Nav Wayfinder",
        family: Family::AdasHud,
        make: clusters::ar_nav_wayfinder::make,
    },
    ClusterEntry {
        theme: ClusterTheme::F35HelmetHmd,
        name: "F-35 Helmet HMD",
        family: Family::AdasHud,
        make: clusters::f35_helmet_hmd::make,
    },
    ClusterEntry {
        theme: ClusterTheme::RetroCyber,
        name: "Retro Cyber",
        family: Family::SciFi,
        make: clusters::retro_cyber::make,
    },
    ClusterEntry {
        theme: ClusterTheme::SynthwaveOutrun,
        name: "Synthwave Outrun",
        family: Family::SciFi,
        make: clusters::synthwave_outrun::make,
    },
    ClusterEntry {
        theme: ClusterTheme::EvaBioSync,
        name: "EVA Bio-Sync",
        family: Family::SciFi,
        make: clusters::eva_bio_sync::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ChronoEventHorizon,
        name: "Chrono Event Horizon",
        family: Family::SciFi,
        make: clusters::chrono_event_horizon::make,
    },
    ClusterEntry {
        theme: ClusterTheme::NeuralCyberSynapse,
        name: "Neural Cyber Synapse",
        family: Family::SciFi,
        make: clusters::neural_cyber_synapse::make,
    },
    ClusterEntry {
        theme: ClusterTheme::SolarisWarpCore,
        name: "Solaris Warp Core",
        family: Family::SciFi,
        make: clusters::solaris_warp_core::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AeroPlasmaBlade,
        name: "Aero Plasma Blade",
        family: Family::SciFi,
        make: clusters::aero_plasma_blade::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ShibuyaChromaticPrisma,
        name: "Shibuya Chromatic Prisma",
        family: Family::SciFi,
        make: clusters::shibuya_chromatic_prisma::make,
    },
    ClusterEntry {
        theme: ClusterTheme::KineticLiquidMetal,
        name: "Kinetic Liquid Metal",
        family: Family::FluidGenerative,
        make: clusters::kinetic_liquid_metal::make,
    },
    ClusterEntry {
        theme: ClusterTheme::CymaticResonanceField,
        name: "Cymatic Resonance Field",
        family: Family::FluidGenerative,
        make: clusters::cymatic_resonance_field::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ChronoMultiverse,
        name: "Chrono Multiverse",
        family: Family::FluidGenerative,
        make: clusters::chrono_multiverse::make,
    },
    ClusterEntry {
        theme: ClusterTheme::LumenEtch,
        name: "Lumen Etch",
        family: Family::LiquidGlass,
        make: clusters::lumen_etch::make,
    },
    ClusterEntry {
        theme: ClusterTheme::VernierLoupe,
        name: "Vernier Loupe",
        family: Family::LiquidGlass,
        make: clusters::vernier_loupe::make,
    },
    ClusterEntry {
        theme: ClusterTheme::DichroicLouvre,
        name: "Dichroic Louvre",
        family: Family::LiquidGlass,
        make: clusters::dichroic_louvre::make,
    },
    ClusterEntry {
        theme: ClusterTheme::FibreOpticCorona,
        name: "Fibre-Optic Corona",
        family: Family::LiquidGlass,
        make: clusters::fibre_optic_corona::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ApertureNine,
        name: "Aperture Nine",
        family: Family::ModernDigital,
        make: clusters::aperture_nine::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AeroSmokeTunnel,
        name: "Aero Smoke Tunnel",
        family: Family::ModernDigital,
        make: clusters::aero_smoke_tunnel::make,
    },
    ClusterEntry {
        theme: ClusterTheme::RattrapanteSector,
        name: "Rattrapante Sector",
        family: Family::ClassicAnalog,
        make: clusters::rattrapante_sector::make,
    },
    ClusterEntry {
        theme: ClusterTheme::SapphireMystery,
        name: "Sapphire Mystery",
        family: Family::ClassicAnalog,
        make: clusters::sapphire_mystery::make,
    },
    ClusterEntry {
        theme: ClusterTheme::RackFocus,
        name: "Rack Focus",
        family: Family::RaceDash,
        make: clusters::rack_focus::make,
    },
    ClusterEntry {
        theme: ClusterTheme::BionicSpaceframe,
        name: "Bionic Spaceframe",
        family: Family::RaceDash,
        make: clusters::bionic_spaceframe::make,
    },
    ClusterEntry {
        theme: ClusterTheme::GeodesicWell,
        name: "Geodesic Well",
        family: Family::AdasHud,
        make: clusters::geodesic_well::make,
    },
    ClusterEntry {
        theme: ClusterTheme::Lightpainter,
        name: "Lightpainter",
        family: Family::AdasHud,
        make: clusters::lightpainter::make,
    },
    ClusterEntry {
        theme: ClusterTheme::TreadSeismograph,
        name: "Tread Seismograph",
        family: Family::AdasHud,
        make: clusters::tread_seismograph::make,
    },
    ClusterEntry {
        theme: ClusterTheme::StarSailOrigami,
        name: "Star Sail Origami",
        family: Family::SciFi,
        make: clusters::star_sail_origami::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ArcLadderCascade,
        name: "Arc Ladder Cascade",
        family: Family::SciFi,
        make: clusters::arc_ladder_cascade::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AuroraVeil,
        name: "Aurora Veil",
        family: Family::SciFi,
        make: clusters::aurora_veil::make,
    },
    ClusterEntry {
        theme: ClusterTheme::IsochromaticLoadPath,
        name: "Isochromatic Load Path",
        family: Family::FluidGenerative,
        make: clusters::isochromatic_load_path::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AbyssalShearBloom,
        name: "Abyssal Shear Bloom",
        family: Family::FluidGenerative,
        make: clusters::abyssal_shear_bloom::make,
    },
    ClusterEntry {
        theme: ClusterTheme::HarmonographDrive,
        name: "Harmonograph Drive",
        family: Family::FluidGenerative,
        make: clusters::harmonograph_drive::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AmgMbuxSupersport,
        name: "AMG MBUX Supersport",
        family: Family::ModernDigital,
        make: clusters::amg_mbux_supersport::make,
    },
    ClusterEntry {
        theme: ClusterTheme::SkylineR34Mfd,
        name: "Skyline R34 MFD",
        family: Family::ClassicAnalog,
        make: clusters::skyline_r34_mfd::make,
    },
    ClusterEntry {
        theme: ClusterTheme::VisionPetals,
        name: "Vision Petals",
        family: Family::LiquidGlass,
        make: clusters::vision_petals::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AmgOneF1,
        name: "AMG ONE F1",
        family: Family::RaceDash,
        make: clusters::amg_one_f1::make,
    },
    ClusterEntry {
        theme: ClusterTheme::CarplayIsland,
        name: "CarPlay Island",
        family: Family::LiquidGlass,
        make: clusters::carplay_island::make,
    },
    ClusterEntry {
        theme: ClusterTheme::Gt3RsApogee,
        name: "GT3 RS Apogee",
        family: Family::ClassicAnalog,
        make: clusters::gt3_rs_apogee::make,
    },
    ClusterEntry {
        theme: ClusterTheme::Gt7CockpitPro,
        name: "GT7 Cockpit Pro",
        family: Family::ModernDigital,
        make: clusters::gt7_cockpit_pro::make,
    },
    ClusterEntry {
        theme: ClusterTheme::LiquidMorph,
        name: "Liquid Morph",
        family: Family::LiquidGlass,
        make: clusters::liquid_morph::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AmgMbuxTrackPace,
        name: "AMG MBUX Track Pace",
        family: Family::ModernDigital,
        make: clusters::amg_mbux_track_pace::make,
    },
    ClusterEntry {
        theme: ClusterTheme::GtrNismoMfd,
        name: "GT-R Nismo MFD",
        family: Family::ModernDigital,
        make: clusters::gtr_nismo_mfd::make,
    },
    ClusterEntry {
        theme: ClusterTheme::SimInputTrace,
        name: "Sim Input Trace",
        family: Family::RaceDash,
        make: clusters::sim_input_trace::make,
    },
    ClusterEntry {
        theme: ClusterTheme::LorenzButterfly,
        name: "Lorenz Butterfly",
        family: Family::FluidGenerative,
        make: clusters::lorenz_butterfly::make,
    },
    ClusterEntry {
        theme: ClusterTheme::AmgMbuxClassic,
        name: "AMG MBUX Classic",
        family: Family::ModernDigital,
        make: clusters::amg_mbux_classic::make,
    },
    ClusterEntry {
        theme: ClusterTheme::SuminagashiMarble,
        name: "Suminagashi Marble",
        family: Family::FluidGenerative,
        make: clusters::suminagashi_marble::make,
    },
    ClusterEntry {
        theme: ClusterTheme::T50Floodlit,
        name: "T.50 Floodlit",
        family: Family::ClassicAnalog,
        make: clusters::t50_floodlit::make,
    },
    ClusterEntry {
        theme: ClusterTheme::HakosukaS20,
        name: "Hakosuka S20",
        family: Family::ClassicAnalog,
        make: clusters::hakosuka_s20::make,
    },
    ClusterEntry {
        theme: ClusterTheme::SimPitWall,
        name: "Sim Pit Wall",
        family: Family::RaceDash,
        make: clusters::sim_pit_wall::make,
    },
    ClusterEntry {
        theme: ClusterTheme::KaleidoscopeMirror,
        name: "Kaleidoscope Mirror",
        family: Family::FluidGenerative,
        make: clusters::kaleidoscope_mirror::make,
    },
    ClusterEntry {
        theme: ClusterTheme::RevueltoCorsa,
        name: "Revuelto Corsa",
        family: Family::ModernDigital,
        make: clusters::revuelto_corsa::make,
    },
    ClusterEntry {
        theme: ClusterTheme::ZNismoSport,
        name: "Z Nismo Sport",
        family: Family::ModernDigital,
        make: clusters::z_nismo_sport::make,
    },
    ClusterEntry {
        theme: ClusterTheme::GoldenAngle,
        name: "Golden Angle",
        family: Family::FluidGenerative,
        make: clusters::golden_angle::make,
    },
    ClusterEntry {
        theme: ClusterTheme::MclarenSlimDisplay,
        name: "McLaren Slim Display",
        family: Family::ModernDigital,
        make: clusters::mclaren_slim_display::make,
    },
    ClusterEntry {
        theme: ClusterTheme::HopfFibration,
        name: "Hopf Fibration",
        family: Family::FluidGenerative,
        make: clusters::hopf_fibration::make,
    },
    ClusterEntry {
        theme: ClusterTheme::HyperbolicTiles,
        name: "Hyperbolic Tiles",
        family: Family::FluidGenerative,
        make: clusters::hyperbolic_tiles::make,
    },
    ClusterEntry {
        theme: ClusterTheme::OilSlickChrome,
        name: "Oil-Slick Chrome",
        family: Family::FluidGenerative,
        make: clusters::oil_slick_chrome::make,
    },
    ClusterEntry {
        theme: ClusterTheme::FaradayMandala,
        name: "Faraday Mandala",
        family: Family::FluidGenerative,
        make: clusters::faraday_mandala::make,
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
            assert_eq!(row.family, cat.family);
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
