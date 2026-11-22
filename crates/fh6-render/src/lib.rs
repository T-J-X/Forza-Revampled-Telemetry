//! fh6-render: the v1 CPU layer kept for the OEM scrim, ambient wash, layout kit, tokens and
//! text until their Phase 7–9 rebuilds on `fh6-kit`.
//!
//! The cluster contract moved to `fh6_kit::Cluster` (contract v2) and the cluster registry to
//! `fh6_clusters::registry`; the v1 `ClusterRenderer` trait, `StubRenderer` and `registry.rs`
//! were removed in Phase 5.
use tiny_skia::Color;

pub mod assets;
pub mod dispatch;
pub mod kit;
pub mod layout;
pub mod oem_scrim;
pub mod pool;
pub mod text;
pub mod tokens;

pub struct RenderContext {
    pub clear_color: Color,
}

impl Default for RenderContext {
    fn default() -> Self {
        Self {
            clear_color: Color::from_rgba8(0, 0, 0, 0),
        }
    }
}

/// Returns the optical centre of the binnacle for the given canvas size.
/// Used as the pivot for cluster scaling and ambient wash geometry.
pub fn binnacle_center(canvas_w: u32, canvas_h: u32) -> (f32, f32) {
    let cx = canvas_w as f32 * 0.500;
    let cy = canvas_h as f32 * 0.845;
    (cx, cy)
}
