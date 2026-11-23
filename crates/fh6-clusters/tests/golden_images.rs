//! Golden-image tests, one test per catalog cluster (named after its module, so
//! `cargo test -p fh6-clusters <snake_name>` runs exactly that cluster).
//!
//! Each cluster is settled on the four review states (`fh6_clusters::review::STATES`) in every view
//! it is designed for (cockpit, and hood when the catalog says so), rendered on the GPU at
//! 1920x1080, cropped to the view envelope and compared perceptually with
//! `tests/golden/images/<Theme>[_hood]_<state>.png`.
//!
//! Run with `FH6_UPDATE_GOLDEN=1` to (re)write the goldens after an intended visual change; review
//! the new PNGs before committing them. A cluster chat only ever rewrites its own goldens.
#![cfg(not(target_arch = "wasm32"))]

mod support;

use fh6_clusters::{catalog, registry, review::STATES};
use fh6_core::ClusterTheme;
use fh6_kit::View;

/// Renders every view and state of `theme` and compares them with the goldens.
fn check(theme: ClusterTheme) {
    let entry = registry::entry(theme).expect("registered");
    let views = catalog::entry(theme).expect("in the catalog").views;
    let mut failures = Vec::new();
    let mut gpu = support::gpu();
    for (view, suffix) in [(View::Cockpit, ""), (View::Hood, "_hood")] {
        let wanted = match view {
            View::Hood => views.hood,
            _ => views.cockpit || views.raised_hud,
        };
        if !wanted {
            continue;
        }
        for state in &STATES {
            let img = gpu.render_cluster(entry, view, state);
            let name = format!("{theme}{suffix}_{}", state.name);
            if let Some(f) = support::check_golden(&name, &img) {
                failures.push(f);
            }
        }
    }
    assert!(failures.is_empty(), "golden mismatches:\n{}", failures.join("\n"));
}

macro_rules! goldens {
    ($($name:ident => $theme:ident),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                check(ClusterTheme::$theme);
            }
        )*

        #[test]
        fn every_catalog_cluster_has_a_golden_test() {
            let listed = [$(ClusterTheme::$theme),*];
            for e in &catalog::CATALOG {
                assert!(listed.contains(&e.theme), "{} has no golden test", e.theme);
            }
            let names = [$(stringify!($name)),*];
            for (n, t) in names.iter().zip(listed.iter()) {
                assert_eq!(catalog::entry(*t).map(|e| e.snake), Some(*n));
            }
        }
    };
}

goldens! {
    liquid_glass => LiquidGlass,
    vision_spatial_glass => VisionSpatialGlass,
    gt7_hood_pro => Gt7HoodPro,
    amg_mbux_superscreen => AmgMbuxSuperscreen,
    drift_glass => DriftGlass,
    glyph_dot_matrix => GlyphDotMatrix,
    porsche_analog => PorscheAnalog,
    pagani_horologerie => PaganiHorologerie,
    bugatti_tourbillon => BugattiTourbillon,
    lfa_v10_nurburgring => LfaV10Nurburgring,
    koenigsegg_gyro => KoenigseggGyro,
    bosch_ddu => BoschDdu,
    valkyrie_lmp1 => ValkyrieLmp1,
    gt7_modular_tripod => Gt7ModularTripod,
    tesla_adas => TeslaAdas,
    windshield_hud => WindshieldHud,
    ar_nav_wayfinder => ArNavWayfinder,
    f35_helmet_hmd => F35HelmetHmd,
    retro_cyber => RetroCyber,
    synthwave_outrun => SynthwaveOutrun,
    eva_bio_sync => EvaBioSync,
    chrono_event_horizon => ChronoEventHorizon,
    neural_cyber_synapse => NeuralCyberSynapse,
    solaris_warp_core => SolarisWarpCore,
    aero_plasma_blade => AeroPlasmaBlade,
    shibuya_chromatic_prisma => ShibuyaChromaticPrisma,
    kinetic_liquid_metal => KineticLiquidMetal,
    cymatic_resonance_field => CymaticResonanceField,
    chrono_multiverse => ChronoMultiverse,
    lumen_etch => LumenEtch,
    vernier_loupe => VernierLoupe,
    dichroic_louvre => DichroicLouvre,
    fibre_optic_corona => FibreOpticCorona,
    aperture_nine => ApertureNine,
    aero_smoke_tunnel => AeroSmokeTunnel,
    rattrapante_sector => RattrapanteSector,
    sapphire_mystery => SapphireMystery,
    rack_focus => RackFocus,
    bionic_spaceframe => BionicSpaceframe,
    geodesic_well => GeodesicWell,
    lightpainter => Lightpainter,
    tread_seismograph => TreadSeismograph,
    star_sail_origami => StarSailOrigami,
    arc_ladder_cascade => ArcLadderCascade,
    aurora_veil => AuroraVeil,
    isochromatic_load_path => IsochromaticLoadPath,
    abyssal_shear_bloom => AbyssalShearBloom,
    harmonograph_drive => HarmonographDrive,
}
