//! fh6-osd: Gamepad-first Unified OSD (4 tabs, live dyno preview + settings).

pub mod binding;
pub mod dyno;
pub mod model;
pub mod render;
pub mod toggle;
pub mod text;

pub use model::{
    parse_gamepad_action, OsdNavAction, OsdTab, UnifiedOsdState, BONNET_STYLES, COCKPIT_STYLES, DRIFT_MODES,
    SHIFT_PATTERNS, TAB_NAMES,
};
pub use render::OsdRenderer;
pub use toggle::{KeyboardInjector, LiveDynoGenerator, MockKeyboardInjector, OsdToggleHandler};

use fh6_core::config::AppConfig;
use fh6_core::telemetry::FH6TelemetryData;
use tiny_skia::Pixmap;
/// Renders a full 3840×2160 Unified OSD frame into a new Pixmap.
/// Used by `preview-export` and Phase 5 PNG exporter.
pub fn render_unified_osd_4k(
    state: &model::UnifiedOsdState,
    config: &AppConfig,
    telemetry: Option<&FH6TelemetryData>,
) -> Pixmap {
    let mut pixmap = Pixmap::new(3840, 2160).expect("4K pixmap alloc");
    let renderer = render::OsdRenderer::new(3840, 2160);
    renderer.render_frame(&mut pixmap.as_mut(), state, config, telemetry);
    pixmap
}
