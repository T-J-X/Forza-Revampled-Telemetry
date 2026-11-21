use fh6_clusters::clusters;
use fh6_core::{AppConfig, ClusterTheme, FH6TelemetryData};
use tiny_skia::{Pixmap, Rect};

fn make_pixmap() -> Pixmap {
    Pixmap::new(3840, 2160).unwrap()
}

fn render_cluster(theme: ClusterTheme, data: &FH6TelemetryData, config: &AppConfig) -> Vec<u8> {
    let mut pix = make_pixmap();
    let mut pixmap = pix.as_mut();
    let bounds = Rect::from_xywh(0.0, 0.0, 3840.0, 2160.0).unwrap();
    match theme {
        ClusterTheme::BoschDdu => clusters::bosch_ddu::render(&mut pixmap, bounds, data, config),
        ClusterTheme::DriftGlass => clusters::drift_glass::render(&mut pixmap, bounds, data, config),
        ClusterTheme::PorscheAnalog => clusters::porsche_analog::render(&mut pixmap, bounds, data, config),
        ClusterTheme::LiquidGlass => clusters::liquid_glass::render(&mut pixmap, bounds, data, config),
        ClusterTheme::WindshieldHud => clusters::windshield_hud::render(&mut pixmap, bounds, data, config),
        ClusterTheme::TeslaAdas => clusters::tesla_adas::render(&mut pixmap, bounds, data, config),
        ClusterTheme::RetroCyber => clusters::retro_cyber::render(&mut pixmap, bounds, data, config),
        ClusterTheme::Gt7HoodPro => clusters::gt7_hood_pro::render(&mut pixmap, bounds, data, config),
        ClusterTheme::AmgMbuxSuperscreen => clusters::amg_mbux_superscreen::render(&mut pixmap, bounds, data, config),
        ClusterTheme::LfaV10Nurburgring => clusters::lfa_v10_nurburgring::render(&mut pixmap, bounds, data, config),
        ClusterTheme::PaganiHorologerie => clusters::pagani_horologerie::render(&mut pixmap, bounds, data, config),
        ClusterTheme::F35HelmetHmd => clusters::f35_helmet_hmd::render(&mut pixmap, bounds, data, config),
        ClusterTheme::BugattiTourbillon => clusters::bugatti_tourbillon::render(&mut pixmap, bounds, data, config),
        ClusterTheme::EvaBioSync => clusters::eva_bio_sync::render(&mut pixmap, bounds, data, config),
        ClusterTheme::KoenigseggGyro => clusters::koenigsegg_gyro::render(&mut pixmap, bounds, data, config),
        ClusterTheme::ValkyrieLmp1 => clusters::valkyrie_lmp1::render(&mut pixmap, bounds, data, config),
        ClusterTheme::SynthwaveOutrun => clusters::synthwave_outrun::render(&mut pixmap, bounds, data, config),
        ClusterTheme::ChronoMultiverse => clusters::chrono_multiverse::render(&mut pixmap, bounds, data, config),
        ClusterTheme::ChronoEventHorizon => clusters::chrono_event_horizon::render(&mut pixmap, bounds, data, config),
        ClusterTheme::NeuralCyberSynapse => clusters::neural_cyber_synapse::render(&mut pixmap, bounds, data, config),
        ClusterTheme::Gt7ModularTripod => clusters::gt7_modular_tripod::render(&mut pixmap, bounds, data, config),
        ClusterTheme::ArNavWayfinder => clusters::ar_nav_wayfinder::render(&mut pixmap, bounds, data, config),
        ClusterTheme::SolarisWarpCore => clusters::solaris_warp_core::render(&mut pixmap, bounds, data, config),
        ClusterTheme::VisionSpatialGlass => clusters::vision_spatial_glass::render(&mut pixmap, bounds, data, config),
        ClusterTheme::GlyphDotMatrix => clusters::glyph_dot_matrix::render(&mut pixmap, bounds, data, config),
        ClusterTheme::AeroPlasmaBlade => clusters::aero_plasma_blade::render(&mut pixmap, bounds, data, config),
        ClusterTheme::CymaticResonanceField => {
            clusters::cymatic_resonance_field::render(&mut pixmap, bounds, data, config)
        }
        ClusterTheme::KineticLiquidMetal => clusters::kinetic_liquid_metal::render(&mut pixmap, bounds, data, config),
        ClusterTheme::ShibuyaChromaticPrisma => {
            clusters::shibuya_chromatic_prisma::render(&mut pixmap, bounds, data, config)
        }
    }
    pix.data().to_vec()
}

fn fnv1a_hash(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

// TODO(phase-7): meant as the `..base_telemetry()` base for the regimes below, which leave engine_max_rpm at 0 (normalized_rpm then falls back to power_watts).
fn _base_telemetry() -> FH6TelemetryData {
    FH6TelemetryData {
        engine_max_rpm: 9000.0,
        engine_idle_rpm: 850.0,
        current_engine_rpm: 850.0,
        speed_ms: 0.0,
        power_watts: 0.0,
        boost_psi: 0.0,
        accel: 0,
        brake: 0,
        gear: 0,
        steer: 0,
        ..Default::default()
    }
}

#[test]
fn telemetry_sensitivity_all_clusters() {
    let config = AppConfig::default();
    let regimes = [
        (
            "idle",
            FH6TelemetryData {
                current_engine_rpm: 850.0,
                speed_ms: 0.0,
                power_watts: 0.0,
                accel: 0,
                brake: 0,
                boost_psi: 0.0,
                gear: 0,
                ..Default::default()
            },
        ),
        (
            "mid",
            FH6TelemetryData {
                current_engine_rpm: 4500.0,
                speed_ms: 50.0,
                power_watts: 250_000.0,
                accel: 128,
                brake: 0,
                boost_psi: 8.0,
                gear: 3,
                ..Default::default()
            },
        ),
        (
            "redline",
            FH6TelemetryData {
                current_engine_rpm: 8500.0,
                speed_ms: 80.0,
                power_watts: 450_000.0,
                accel: 255,
                brake: 0,
                boost_psi: 18.0,
                gear: 6,
                ..Default::default()
            },
        ),
    ];
    for theme in ClusterTheme::all() {
        let hashes: Vec<u64> = regimes
            .iter()
            .map(|(_, d)| {
                let buf = render_cluster(*theme, d, &config);
                fnv1a_hash(&buf)
            })
            .collect();
        // determinism check
        let h0a = render_cluster(*theme, &regimes[0].1, &config);
        let h0b = render_cluster(*theme, &regimes[0].1, &config);
        assert_eq!(fnv1a_hash(&h0a), fnv1a_hash(&h0b), "non-deterministic for {:?}", theme);
        // at least two distinct hashes across regimes
        let distinct = hashes[0] != hashes[1] || hashes[1] != hashes[2] || hashes[0] != hashes[2];
        assert!(distinct, "cluster {:?} shows no telemetry sensitivity", theme);
    }
}
