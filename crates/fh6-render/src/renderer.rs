//! Renderer traits for cluster rendering and style abstraction.
use crate::ClusterRenderer;
use fh6_core::{AppConfig, ClusterTheme, FH6TelemetryData};
use tiny_skia::{PixmapMut, Rect};

pub trait ClusterStyle {
    fn style_name(&self) -> &'static str;
    fn is_overhead_projection(&self) -> bool {
        false
    }
}

pub struct StubRenderer {
    theme: ClusterTheme,
    name: &'static str,
}

impl StubRenderer {
    pub fn new(theme: ClusterTheme, name: &'static str) -> Self {
        Self { theme, name }
    }
}

impl ClusterRenderer for StubRenderer {
    fn theme(&self) -> ClusterTheme {
        self.theme
    }

    fn render(&mut self, _pixmap: &mut PixmapMut, _bounds: Rect, _data: &FH6TelemetryData, _config: &AppConfig) {
        // Stub: draw label only in real implementation.
        // For now, no-op to keep compile-ready.
        let _ = self.name;
    }
}

impl ClusterStyle for StubRenderer {
    fn style_name(&self) -> &'static str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_core::ClusterTheme;

    #[test]
    fn stub_theme_matches() {
        let r = StubRenderer::new(ClusterTheme::LiquidGlass, "LiquidGlass");
        assert_eq!(r.theme(), ClusterTheme::LiquidGlass);
        assert_eq!(r.style_name(), "LiquidGlass");
    }
}
