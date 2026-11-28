//! The asset sets. Each `build` runs its jobs through [`crate::ctx::Ctx::job`], so unchanged
//! jobs are skipped.
use crate::ctx::Ctx;
use crate::error::Result;

pub mod adas;
pub mod classic;
pub mod comfyset;
pub mod fluid;
pub mod fonts;
pub mod glass;
pub mod logos;
pub mod scifi;

/// Builds one named set.
pub fn build(ctx: &mut Ctx, set: &str) -> Result<()> {
    match set {
        "glass" => glass::build(ctx),
        "fluid" => fluid::build(ctx),
        "scifi" => scifi::build(ctx),
        "classic" => classic::build(ctx),
        "adas" => adas::build(ctx),
        "logos" => logos::build(ctx),
        "fonts" => fonts::build(ctx),
        "comfy" => comfyset::build(ctx),
        "tesla_adas" => crate::recipes::tesla_adas::build(ctx),
        "gt7_hood_pro" => crate::recipes::gt7_hood_pro::build(ctx),
        "retro_cyber" => crate::recipes::retro_cyber::build(ctx),
        "bosch_ddu" => crate::recipes::bosch_ddu::build(ctx),
        "liquid_glass" => crate::recipes::liquid_glass::build(ctx),
        "porsche_analog" => crate::recipes::porsche_analog::build(ctx),
        "osd" => crate::recipes::osd::build(ctx),
        // Registered by the code re-cut (2026-11-28) so a dash thread never edits this list;
        // each starts empty and its thread fills it.
        "vision_spatial_glass" => crate::recipes::vision_spatial_glass::build(ctx),
        "amg_mbux_superscreen" => crate::recipes::amg_mbux_superscreen::build(ctx),
        "drift_glass" => crate::recipes::drift_glass::build(ctx),
        "glyph_dot_matrix" => crate::recipes::glyph_dot_matrix::build(ctx),
        "pagani_horologerie" => crate::recipes::pagani_horologerie::build(ctx),
        "bugatti_tourbillon" => crate::recipes::bugatti_tourbillon::build(ctx),
        "lfa_v10_nurburgring" => crate::recipes::lfa_v10_nurburgring::build(ctx),
        "koenigsegg_gyro" => crate::recipes::koenigsegg_gyro::build(ctx),
        "valkyrie_lmp1" => crate::recipes::valkyrie_lmp1::build(ctx),
        "gt7_modular_tripod" => crate::recipes::gt7_modular_tripod::build(ctx),
        "windshield_hud" => crate::recipes::windshield_hud::build(ctx),
        "ar_nav_wayfinder" => crate::recipes::ar_nav_wayfinder::build(ctx),
        "f35_helmet_hmd" => crate::recipes::f35_helmet_hmd::build(ctx),
        "synthwave_outrun" => crate::recipes::synthwave_outrun::build(ctx),
        "eva_bio_sync" => crate::recipes::eva_bio_sync::build(ctx),
        "chrono_event_horizon" => crate::recipes::chrono_event_horizon::build(ctx),
        "neural_cyber_synapse" => crate::recipes::neural_cyber_synapse::build(ctx),
        "solaris_warp_core" => crate::recipes::solaris_warp_core::build(ctx),
        "aero_plasma_blade" => crate::recipes::aero_plasma_blade::build(ctx),
        "shibuya_chromatic_prisma" => crate::recipes::shibuya_chromatic_prisma::build(ctx),
        "kinetic_liquid_metal" => crate::recipes::kinetic_liquid_metal::build(ctx),
        "cymatic_resonance_field" => crate::recipes::cymatic_resonance_field::build(ctx),
        "chrono_multiverse" => crate::recipes::chrono_multiverse::build(ctx),
        "lumen_etch" => crate::recipes::lumen_etch::build(ctx),
        "vernier_loupe" => crate::recipes::vernier_loupe::build(ctx),
        "dichroic_louvre" => crate::recipes::dichroic_louvre::build(ctx),
        "fibre_optic_corona" => crate::recipes::fibre_optic_corona::build(ctx),
        "aperture_nine" => crate::recipes::aperture_nine::build(ctx),
        "aero_smoke_tunnel" => crate::recipes::aero_smoke_tunnel::build(ctx),
        "rattrapante_sector" => crate::recipes::rattrapante_sector::build(ctx),
        "sapphire_mystery" => crate::recipes::sapphire_mystery::build(ctx),
        "rack_focus" => crate::recipes::rack_focus::build(ctx),
        "bionic_spaceframe" => crate::recipes::bionic_spaceframe::build(ctx),
        "geodesic_well" => crate::recipes::geodesic_well::build(ctx),
        "lightpainter" => crate::recipes::lightpainter::build(ctx),
        "tread_seismograph" => crate::recipes::tread_seismograph::build(ctx),
        "star_sail_origami" => crate::recipes::star_sail_origami::build(ctx),
        "arc_ladder_cascade" => crate::recipes::arc_ladder_cascade::build(ctx),
        "aurora_veil" => crate::recipes::aurora_veil::build(ctx),
        "isochromatic_load_path" => crate::recipes::isochromatic_load_path::build(ctx),
        "abyssal_shear_bloom" => crate::recipes::abyssal_shear_bloom::build(ctx),
        "harmonograph_drive" => crate::recipes::harmonograph_drive::build(ctx),
        "amg_mbux_supersport" => crate::recipes::amg_mbux_supersport::build(ctx),
        "skyline_r34_mfd" => crate::recipes::skyline_r34_mfd::build(ctx),
        "vision_petals" => crate::recipes::vision_petals::build(ctx),
        "amg_one_f1" => crate::recipes::amg_one_f1::build(ctx),
        "carplay_island" => crate::recipes::carplay_island::build(ctx),
        "gt3_rs_apogee" => crate::recipes::gt3_rs_apogee::build(ctx),
        "gt7_cockpit_pro" => crate::recipes::gt7_cockpit_pro::build(ctx),
        "liquid_morph" => crate::recipes::liquid_morph::build(ctx),
        "amg_mbux_track_pace" => crate::recipes::amg_mbux_track_pace::build(ctx),
        "gtr_nismo_mfd" => crate::recipes::gtr_nismo_mfd::build(ctx),
        "sim_input_trace" => crate::recipes::sim_input_trace::build(ctx),
        "lorenz_butterfly" => crate::recipes::lorenz_butterfly::build(ctx),
        "amg_mbux_classic" => crate::recipes::amg_mbux_classic::build(ctx),
        "suminagashi_marble" => crate::recipes::suminagashi_marble::build(ctx),
        "t50_floodlit" => crate::recipes::t50_floodlit::build(ctx),
        "hakosuka_s20" => crate::recipes::hakosuka_s20::build(ctx),
        "sim_pit_wall" => crate::recipes::sim_pit_wall::build(ctx),
        "kaleidoscope_mirror" => crate::recipes::kaleidoscope_mirror::build(ctx),
        "revuelto_corsa" => crate::recipes::revuelto_corsa::build(ctx),
        "z_nismo_sport" => crate::recipes::z_nismo_sport::build(ctx),
        "golden_angle" => crate::recipes::golden_angle::build(ctx),
        "mclaren_slim_display" => crate::recipes::mclaren_slim_display::build(ctx),
        "hopf_fibration" => crate::recipes::hopf_fibration::build(ctx),
        "hyperbolic_tiles" => crate::recipes::hyperbolic_tiles::build(ctx),
        _ => Ok(()),
    }
}
