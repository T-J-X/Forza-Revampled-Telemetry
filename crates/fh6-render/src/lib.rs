//! fh6-render: 4K rendering engine, ClusterRenderer trait, and optical layout primitives.
use fh6_core::{AppConfig, ClusterTheme, FH6TelemetryData};
use tiny_skia::{Color, PixmapMut, Rect};

pub mod assets;
pub mod dispatch;
pub mod kit;
pub mod layout;
pub mod oem_scrim;
pub mod pool;
pub mod registry;
pub mod renderer;
pub mod text;
pub mod tokens;

pub trait ClusterRenderer: Send + Sync {
    fn theme(&self) -> ClusterTheme;
    fn render(&mut self, pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig);
}

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
