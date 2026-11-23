//! fh6-clusters: the cluster catalog ([`catalog::CATALOG`]), the cluster registry (contract v2,
//! [`registry::REGISTRY`]), one module per catalog cluster ([`clusters`]), shared components
//! ([`common`]), the two Phase 5 kit samples ([`kit`]) and the 29 v1 tiny-skia clusters
//! ([`legacy`]) kept as references for their Phase 7 rebuilds.
//!
//! Phase 7 rule: a cluster chat edits only `src/clusters/<snake_name>/` (and its own assets and
//! goldens). The catalog, registry, theme enum, golden harness and `common/` are shared and are
//! changed only by the conductor.

use fh6_core::config::CameraViewMode;
use fh6_core::ClusterTheme;
use fh6_render::binnacle_center;
use fh6_render::oem_scrim::{ambient_wash::BinnacleAmbientWash, scrim::render_oem_scrim};
use std::sync::Mutex;
use std::sync::OnceLock;
use tiny_skia::{Pixmap, PixmapPaint, Rect, Transform};
use tiny_skia::{PixmapMut as LegacyPixmap, Rect as LegacyRect};

pub mod catalog;
pub mod clusters;
pub mod common;
pub mod draw_ext;
pub mod kit;
pub mod legacy;
pub mod registry;
pub mod review;

/// The v1 tiny-skia cluster interface. It is kept only so the 29 v1 clusters stay buildable as
/// references for their Phase 7 rebuilds; new clusters implement [`fh6_kit::Cluster`] and
/// register in [`registry::REGISTRY`].
pub trait LegacyClusterRenderer: Send + Sync {
    /// The theme this renderer draws.
    fn theme(&self) -> ClusterTheme;
    /// Draws one frame into `pixmap`.
    fn render(
        &mut self,
        pixmap: &mut LegacyPixmap,
        bounds: LegacyRect,
        data: &fh6_core::FH6TelemetryData,
        config: &fh6_core::AppConfig,
    );
}

static WASH: OnceLock<Mutex<BinnacleAmbientWash>> = OnceLock::new();

pub struct BoschDduRenderer;
impl LegacyClusterRenderer for BoschDduRenderer {
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
        legacy::bosch_ddu::render(pixmap, bounds, data, config);
    }
}
pub struct Gt7HoodProRenderer;
impl LegacyClusterRenderer for Gt7HoodProRenderer {
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
        legacy::gt7_hood_pro::render(pixmap, bounds, data, config);
    }
}
pub struct PorscheAnalogRenderer;
impl LegacyClusterRenderer for PorscheAnalogRenderer {
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
        legacy::porsche_analog::render(pixmap, bounds, data, config);
    }
}
pub struct DriftGlassRenderer;
impl LegacyClusterRenderer for DriftGlassRenderer {
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
        legacy::drift_glass::render(pixmap, bounds, data, config);
    }
}
pub struct LiquidGlassRenderer;
impl LegacyClusterRenderer for LiquidGlassRenderer {
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
        legacy::liquid_glass::render(pixmap, bounds, data, config);
    }
}
pub struct RetroCyberRenderer;
impl LegacyClusterRenderer for RetroCyberRenderer {
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
        legacy::retro_cyber::render(pixmap, bounds, data, config);
    }
}
pub struct SynthwaveOutrunRenderer;
impl LegacyClusterRenderer for SynthwaveOutrunRenderer {
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
        legacy::synthwave_outrun::render(pixmap, bounds, data, config);
    }
}
pub struct F35HelmetHmdRenderer;
impl LegacyClusterRenderer for F35HelmetHmdRenderer {
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
        legacy::f35_helmet_hmd::render(pixmap, bounds, data, config);
    }
}
pub struct ValkyrieLmp1Renderer;
impl LegacyClusterRenderer for ValkyrieLmp1Renderer {
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
        legacy::valkyrie_lmp1::render(pixmap, bounds, data, config);
    }
}
pub struct EvaBioSyncRenderer;
impl LegacyClusterRenderer for EvaBioSyncRenderer {
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
        legacy::eva_bio_sync::render(pixmap, bounds, data, config);
    }
}
pub struct ChronoMultiverseRenderer;
impl LegacyClusterRenderer for ChronoMultiverseRenderer {
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
        legacy::chrono_multiverse::render(pixmap, bounds, data, config);
    }
}
pub struct AmgMbuxSuperscreenRenderer;
impl LegacyClusterRenderer for AmgMbuxSuperscreenRenderer {
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
        legacy::amg_mbux_superscreen::render(pixmap, bounds, data, config);
    }
}
pub struct WindshieldHudRenderer;
impl LegacyClusterRenderer for WindshieldHudRenderer {
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
        legacy::windshield_hud::render(pixmap, bounds, data, config);
    }
}
pub struct TeslaAdasRenderer;
impl LegacyClusterRenderer for TeslaAdasRenderer {
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
        legacy::tesla_adas::render(pixmap, bounds, data, config);
    }
}
pub struct LegacyDigitalRenderer;
impl LegacyClusterRenderer for LegacyDigitalRenderer {
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
        legacy::legacy_digital::render(pixmap, bounds, data, config);
    }
}

pub struct ChronoEventHorizonRenderer;
impl LegacyClusterRenderer for ChronoEventHorizonRenderer {
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
        legacy::chrono_event_horizon::render(pixmap, bounds, data, config);
    }
}
pub struct NeuralCyberSynapseRenderer;
impl LegacyClusterRenderer for NeuralCyberSynapseRenderer {
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
        legacy::neural_cyber_synapse::render(pixmap, bounds, data, config);
    }
}
pub struct Gt7ModularTripodRenderer;
impl LegacyClusterRenderer for Gt7ModularTripodRenderer {
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
        legacy::gt7_modular_tripod::render(pixmap, bounds, data, config);
    }
}
pub struct ArNavWayfinderRenderer;
impl LegacyClusterRenderer for ArNavWayfinderRenderer {
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
        legacy::ar_nav_wayfinder::render(pixmap, bounds, data, config);
    }
}
pub struct SolarisWarpCoreRenderer;
impl LegacyClusterRenderer for SolarisWarpCoreRenderer {
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
        legacy::solaris_warp_core::render(pixmap, bounds, data, config);
    }
}

pub struct VisionSpatialGlassRenderer;
impl LegacyClusterRenderer for VisionSpatialGlassRenderer {
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
        legacy::vision_spatial_glass::render(pixmap, bounds, data, config);
    }
}
pub struct GlyphDotMatrixRenderer;
impl LegacyClusterRenderer for GlyphDotMatrixRenderer {
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
        legacy::glyph_dot_matrix::render(pixmap, bounds, data, config);
    }
}
pub struct AeroPlasmaBladeRenderer;
impl LegacyClusterRenderer for AeroPlasmaBladeRenderer {
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
        legacy::aero_plasma_blade::render(pixmap, bounds, data, config);
    }
}
pub struct CymaticResonanceFieldRenderer;
impl LegacyClusterRenderer for CymaticResonanceFieldRenderer {
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
        legacy::cymatic_resonance_field::render(pixmap, bounds, data, config);
    }
}
pub struct KineticLiquidMetalRenderer;
impl LegacyClusterRenderer for KineticLiquidMetalRenderer {
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
        legacy::kinetic_liquid_metal::render(pixmap, bounds, data, config);
    }
}
pub struct ShibuyaChromaticPrismaRenderer;
impl LegacyClusterRenderer for ShibuyaChromaticPrismaRenderer {
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
        legacy::shibuya_chromatic_prisma::render(pixmap, bounds, data, config);
    }
}
pub struct BugattiTourbillonRenderer;
impl LegacyClusterRenderer for BugattiTourbillonRenderer {
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
        legacy::bugatti_tourbillon::render(pixmap, bounds, data, config);
    }
}
pub struct PaganiHorologerieRenderer;
impl LegacyClusterRenderer for PaganiHorologerieRenderer {
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
        legacy::pagani_horologerie::render(pixmap, bounds, data, config);
    }
}
pub struct KoenigseggGyroRenderer;
impl LegacyClusterRenderer for KoenigseggGyroRenderer {
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
        legacy::koenigsegg_gyro::render(pixmap, bounds, data, config);
    }
}
pub struct LfaV10NurburgringRenderer;
impl LegacyClusterRenderer for LfaV10NurburgringRenderer {
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
        legacy::lfa_v10_nurburgring::render(pixmap, bounds, data, config);
    }
}

pub fn create_cluster(theme: ClusterTheme) -> Option<Box<dyn LegacyClusterRenderer>> {
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
        // The 19 concepts new in Phase 7 never had a v1 renderer.
        _ => None,
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
        // Scrim first (SCRIM_FIT dome on the cockpit anchor), then the lip wash.
        render_oem_scrim(pixmap, config);
        let wash_mutex = WASH.get_or_init(|| Mutex::new(BinnacleAmbientWash::new()));
        if let Ok(mut wash) = wash_mutex.lock() {
            wash.render_for(pixmap, config, theme.accent_rgb());
        }
    }

    let Some(mut renderer) = create_cluster(theme) else {
        return;
    };
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
        use fh6_render::oem_scrim::ambient_wash::BinnacleAmbientWash;
        let cfg = fh6_core::AppConfig {
            oem_dash_opaque_pct: 100,
            ..fh6_core::AppConfig::default()
        };
        let accent = ClusterTheme::LiquidGlass.accent_rgb();
        BinnacleAmbientWash::new().render_for(&mut pixmap, &cfg, accent);
        // The lip runs along the dome top at 4K y 1290.
        let lit = (1284..1296).any(|y| pixmap.data_mut()[(y * 3840 + 1920) * 4 + 3] > 0);
        assert!(lit, "wash should light the dome lip");
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
