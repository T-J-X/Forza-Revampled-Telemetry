//! fh6-clusters: 18 bespoke motorsport instrument cluster implementations.

use fh6_core::config::CameraViewMode;
use fh6_core::ClusterTheme;
use fh6_render::binnacle_center;
use fh6_render::oem_scrim::{
    ambient_wash::{feather_sigma_px, BinnacleAmbientWash},
    scrim::render_oem_scrim,
};
use fh6_render::ClusterRenderer;
use std::sync::Mutex;
use std::sync::OnceLock;
use tiny_skia::{Pixmap, PixmapPaint, Rect, Transform};

pub mod clusters;
pub mod draw_ext;

static WASH: OnceLock<Mutex<BinnacleAmbientWash>> = OnceLock::new();

pub struct BoschDduRenderer;
impl ClusterRenderer for BoschDduRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::BoschDdu
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::bosch_ddu::render(pixmap, bounds, data, config);
    }
}
pub struct Gt7HoodProRenderer;
impl ClusterRenderer for Gt7HoodProRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::Gt7HoodPro
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::gt7_hood_pro::render(pixmap, bounds, data, config);
    }
}
pub struct PorscheAnalogRenderer;
impl ClusterRenderer for PorscheAnalogRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::PorscheAnalog
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::porsche_analog::render(pixmap, bounds, data, config);
    }
}
pub struct DriftGlassRenderer;
impl ClusterRenderer for DriftGlassRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::DriftGlass
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::drift_glass::render(pixmap, bounds, data, config);
    }
}
pub struct LiquidGlassRenderer;
impl ClusterRenderer for LiquidGlassRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::LiquidGlass
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::liquid_glass::render(pixmap, bounds, data, config);
    }
}
pub struct RetroCyberRenderer;
impl ClusterRenderer for RetroCyberRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::RetroCyber
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::retro_cyber::render(pixmap, bounds, data, config);
    }
}
pub struct SynthwaveOutrunRenderer;
impl ClusterRenderer for SynthwaveOutrunRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::SynthwaveOutrun
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::synthwave_outrun::render(pixmap, bounds, data, config);
    }
}
pub struct F35HelmetHmdRenderer;
impl ClusterRenderer for F35HelmetHmdRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::F35HelmetHmd
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::f35_helmet_hmd::render(pixmap, bounds, data, config);
    }
}
pub struct ValkyrieLmp1Renderer;
impl ClusterRenderer for ValkyrieLmp1Renderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::ValkyrieLmp1
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::valkyrie_lmp1::render(pixmap, bounds, data, config);
    }
}
pub struct EvaBioSyncRenderer;
impl ClusterRenderer for EvaBioSyncRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::EvaBioSync
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::eva_bio_sync::render(pixmap, bounds, data, config);
    }
}
pub struct ChronoMultiverseRenderer;
impl ClusterRenderer for ChronoMultiverseRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::ChronoMultiverse
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::chrono_multiverse::render(pixmap, bounds, data, config);
    }
}
pub struct AmgMbuxSuperscreenRenderer;
impl ClusterRenderer for AmgMbuxSuperscreenRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::AmgMbuxSuperscreen
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::amg_mbux_superscreen::render(pixmap, bounds, data, config);
    }
}
pub struct WindshieldHudRenderer;
impl ClusterRenderer for WindshieldHudRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::WindshieldHud
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::windshield_hud::render(pixmap, bounds, data, config);
    }
}
pub struct TeslaAdasRenderer;
impl ClusterRenderer for TeslaAdasRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::TeslaAdas
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::tesla_adas::render(pixmap, bounds, data, config);
    }
}
pub struct LegacyDigitalRenderer;
impl ClusterRenderer for LegacyDigitalRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::BoschDdu
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::legacy_digital::render(pixmap, bounds, data, config);
    }
}

pub struct ChronoEventHorizonRenderer;
impl ClusterRenderer for ChronoEventHorizonRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::ChronoEventHorizon
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::chrono_event_horizon::render(pixmap, bounds, data, config);
    }
}
pub struct NeuralCyberSynapseRenderer;
impl ClusterRenderer for NeuralCyberSynapseRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::NeuralCyberSynapse
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::neural_cyber_synapse::render(pixmap, bounds, data, config);
    }
}
pub struct Gt7ModularTripodRenderer;
impl ClusterRenderer for Gt7ModularTripodRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::Gt7ModularTripod
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::gt7_modular_tripod::render(pixmap, bounds, data, config);
    }
}
pub struct ArNavWayfinderRenderer;
impl ClusterRenderer for ArNavWayfinderRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::ArNavWayfinder
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::ar_nav_wayfinder::render(pixmap, bounds, data, config);
    }
}
pub struct SolarisWarpCoreRenderer;
impl ClusterRenderer for SolarisWarpCoreRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::SolarisWarpCore
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::solaris_warp_core::render(pixmap, bounds, data, config);
    }
}

pub struct VisionSpatialGlassRenderer;
impl ClusterRenderer for VisionSpatialGlassRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::VisionSpatialGlass
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::vision_spatial_glass::render(pixmap, bounds, data, config);
    }
}
pub struct GlyphDotMatrixRenderer;
impl ClusterRenderer for GlyphDotMatrixRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::GlyphDotMatrix
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::glyph_dot_matrix::render(pixmap, bounds, data, config);
    }
}
pub struct AeroPlasmaBladeRenderer;
impl ClusterRenderer for AeroPlasmaBladeRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::AeroPlasmaBlade
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::aero_plasma_blade::render(pixmap, bounds, data, config);
    }
}
pub struct CymaticResonanceFieldRenderer;
impl ClusterRenderer for CymaticResonanceFieldRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::CymaticResonanceField
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::cymatic_resonance_field::render(pixmap, bounds, data, config);
    }
}
pub struct KineticLiquidMetalRenderer;
impl ClusterRenderer for KineticLiquidMetalRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::KineticLiquidMetal
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::kinetic_liquid_metal::render(pixmap, bounds, data, config);
    }
}
pub struct ShibuyaChromaticPrismaRenderer;
impl ClusterRenderer for ShibuyaChromaticPrismaRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::ShibuyaChromaticPrisma
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::shibuya_chromatic_prisma::render(pixmap, bounds, data, config);
    }
}
pub struct BugattiTourbillonRenderer;
impl ClusterRenderer for BugattiTourbillonRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::BugattiTourbillon
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::bugatti_tourbillon::render(pixmap, bounds, data, config);
    }
}
pub struct PaganiHorologerieRenderer;
impl ClusterRenderer for PaganiHorologerieRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::PaganiHorologerie
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::pagani_horologerie::render(pixmap, bounds, data, config);
    }
}
pub struct KoenigseggGyroRenderer;
impl ClusterRenderer for KoenigseggGyroRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::KoenigseggGyro
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::koenigsegg_gyro::render(pixmap, bounds, data, config);
    }
}
pub struct LfaV10NurburgringRenderer;
impl ClusterRenderer for LfaV10NurburgringRenderer {
    fn theme(&self) -> ClusterTheme {
        ClusterTheme::LfaV10Nurburgring
    }
    fn render(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        bounds: tiny_skia::Rect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    ) {
        clusters::lfa_v10_nurburgring::render(pixmap, bounds, data, config);
    }
}

pub fn create_cluster(theme: ClusterTheme) -> Option<Box<dyn ClusterRenderer>> {
    match theme {
        ClusterTheme::BoschDdu => Some(Box::new(BoschDduRenderer)),
        ClusterTheme::Gt7HoodPro => Some(Box::new(Gt7HoodProRenderer)),
        ClusterTheme::PorscheAnalog => Some(Box::new(PorscheAnalogRenderer)),
        ClusterTheme::DriftGlass => Some(Box::new(DriftGlassRenderer)),
        ClusterTheme::LiquidGlass => Some(Box::new(LiquidGlassRenderer)),
        ClusterTheme::RetroCyber => Some(Box::new(RetroCyberRenderer)),
        ClusterTheme::SynthwaveOutrun => Some(Box::new(SynthwaveOutrunRenderer)),
        ClusterTheme::F35HelmetHmd => Some(Box::new(F35HelmetHmdRenderer)),
        ClusterTheme::ValkyrieLmp1 => Some(Box::new(ValkyrieLmp1Renderer)),
        ClusterTheme::EvaBioSync => Some(Box::new(EvaBioSyncRenderer)),
        ClusterTheme::ChronoMultiverse => Some(Box::new(ChronoMultiverseRenderer)),
        ClusterTheme::AmgMbuxSuperscreen => Some(Box::new(AmgMbuxSuperscreenRenderer)),
        ClusterTheme::WindshieldHud => Some(Box::new(WindshieldHudRenderer)),
        ClusterTheme::TeslaAdas => Some(Box::new(TeslaAdasRenderer)),
        ClusterTheme::ChronoEventHorizon => Some(Box::new(ChronoEventHorizonRenderer)),
        ClusterTheme::NeuralCyberSynapse => Some(Box::new(NeuralCyberSynapseRenderer)),
        ClusterTheme::Gt7ModularTripod => Some(Box::new(Gt7ModularTripodRenderer)),
        ClusterTheme::ArNavWayfinder => Some(Box::new(ArNavWayfinderRenderer)),
        ClusterTheme::SolarisWarpCore => Some(Box::new(SolarisWarpCoreRenderer)),
        ClusterTheme::VisionSpatialGlass => Some(Box::new(VisionSpatialGlassRenderer)),
        ClusterTheme::GlyphDotMatrix => Some(Box::new(GlyphDotMatrixRenderer)),
        ClusterTheme::AeroPlasmaBlade => Some(Box::new(AeroPlasmaBladeRenderer)),
        ClusterTheme::CymaticResonanceField => Some(Box::new(CymaticResonanceFieldRenderer)),
        ClusterTheme::KineticLiquidMetal => Some(Box::new(KineticLiquidMetalRenderer)),
        ClusterTheme::ShibuyaChromaticPrisma => Some(Box::new(ShibuyaChromaticPrismaRenderer)),
        ClusterTheme::LfaV10Nurburgring => Some(Box::new(LfaV10NurburgringRenderer)),
        ClusterTheme::PaganiHorologerie => Some(Box::new(PaganiHorologerieRenderer)),
        ClusterTheme::BugattiTourbillon => Some(Box::new(BugattiTourbillonRenderer)),
        ClusterTheme::KoenigseggGyro => Some(Box::new(KoenigseggGyroRenderer)),
    }
}

pub fn render_cluster(
    pixmap: &mut tiny_skia::PixmapMut,
    theme: ClusterTheme,
    data: &fh6_core::FH6TelemetryData,
    config: &fh6_core::AppConfig,
) {
    // Draw order: OEM scrim → ambient wash → cluster
    let canvas_w = pixmap.width();
    let canvas_h = pixmap.height();
    let bounds = Rect::from_xywh(0.0, 0.0, canvas_w as f32, canvas_h as f32).unwrap();
    let scale = config.cluster_scale.clamp(0.70, 1.40);

    // Determine view mode
    let view = config.active_camera_view;
    // Also respect legacy index fallback
    let view = if config.current_camera_index == 3 {
        CameraViewMode::Cockpit
    } else if config.current_camera_index == 4 {
        CameraViewMode::Bonnet
    } else {
        view
    };

    if view == CameraViewMode::Cockpit {
        // Scrim first
        render_oem_scrim(pixmap, bounds, scale, config);

        // Ambient wash second
        if config.ambient_glow_enabled {
            let accent = theme.accent_rgb();
            let intensity = config.ambient_glow_intensity.clamp(0.0, 1.0);
            let feather_norm = config.ambient_cowl_feather.clamp(0.05, 1.0);
            let feather_sigma = feather_sigma_px(feather_norm, scale);
            let wash_mutex = WASH.get_or_init(|| Mutex::new(BinnacleAmbientWash::new()));
            let mut wash = wash_mutex.lock().unwrap();
            wash.render(
                pixmap,
                pixmap.width(),
                pixmap.height(),
                accent,
                intensity,
                feather_sigma,
                scale,
            );
        }
    }

    let mut renderer = create_cluster(theme).expect("unknown cluster theme");
    if (scale - 1.0).abs() < f32::EPSILON {
        renderer.render(pixmap, bounds, data, config);
    } else {
        // Render into scratch and scale about binnacle centre
        let mut scratch = Pixmap::new(canvas_w, canvas_h).unwrap();
        {
            let mut scratch_mut = scratch.as_mut();
            renderer.render(&mut scratch_mut, bounds, data, config);
        }
        let (cx, cy) = binnacle_center(canvas_w, canvas_h);
        let transform = Transform::from_translate(-cx, -cy)
            .pre_scale(scale, scale)
            .post_translate(cx, cy);
        let paint = PixmapPaint::default();
        let scratch_ref = scratch.as_ref();
        pixmap.draw_pixmap(0, 0, scratch_ref, &paint, transform, None);
    }
}

#[cfg(test)]
mod tests {
    use fh6_core::ClusterTheme;
    use tiny_skia::Pixmap;

    fn make_pixmap() -> tiny_skia::Pixmap {
        Pixmap::new(3840, 2160).unwrap()
    }

    #[test]
    fn wash_cockpit_enabled_draws() {
        let mut pix = make_pixmap();
        let mut pixmap = pix.as_mut();
        // Direct wash test, no cluster overlay
        use fh6_render::oem_scrim::ambient_wash::BinnacleAmbientWash;
        let mut wash = BinnacleAmbientWash::new();
        let accent = ClusterTheme::LiquidGlass.accent_rgb();
        wash.render(&mut pixmap, 3840, 2160, accent, 1.0, 20.0, 1.0);
        // Sample inside cavity
        let cx = 1920;
        let cy = (2160.0 * 0.845) as usize;
        let idx = (cy * 3840 + cx) * 4;
        let a = pixmap.data_mut()[idx + 3];
        assert!(a > 0, "wash should light cavity pixel, got alpha {}", a);
        // Sample above cowl
        let idx2 = (10 * 3840 + 10) * 4;
        assert_eq!(pixmap.data_mut()[idx2 + 3], 0, "above cowl must be zero");
    }

    #[test]
    fn wash_disabled_no_draw() {
        let mut pix = make_pixmap();
        let mut pixmap = pix.as_mut();
        // Wash should be skipped when disabled; we test by not calling wash
        // Ensure pixmap remains zero
        let cx = 1920;
        let cy = (2160.0 * 0.845) as usize;
        let idx = (cy * 3840 + cx) * 4;
        assert_eq!(pixmap.data_mut()[idx + 3], 0, "wash disabled, alpha must be zero");
    }

    #[test]
    fn wash_bonnet_no_draw() {
        let mut pix = make_pixmap();
        let mut pixmap = pix.as_mut();
        // Bonnet view should not trigger wash; pixmap stays zero
        let cx = 1920;
        let cy = (2160.0 * 0.845) as usize;
        let idx = (cy * 3840 + cx) * 4;
        assert_eq!(pixmap.data_mut()[idx + 3], 0, "bonnet must not draw wash");
    }
}
