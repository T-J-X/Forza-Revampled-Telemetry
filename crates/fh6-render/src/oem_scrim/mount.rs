//! Cockpit mount, visor, hub cap and hood view gate.
//! Layers: physical mount, visor-lip shading and steering hub cap.

use tiny_skia::{Color, PixmapMut, Rect};

pub struct MountParams {
    pub bounds: Rect,
    pub scale: f32,
    pub night_factor: f32,
}

pub fn render_cockpit_physical_mount(pixmap: &mut PixmapMut, params: &MountParams) {
    // Stub: physical mount rendering
    // Real implementation would draw AO ellipse, steering column collar, bolts, etc.
    // Not implemented yet: no-op so the call sites compile.
    let _ = pixmap;
    let _ = params;
}

pub fn render_cockpit_visor_lip_shading(pixmap: &mut PixmapMut, bounds: Rect, scale: f32) {
    let _ = pixmap;
    let _ = bounds;
    let _ = scale;
}

pub fn render_steering_hub_cap(
    pixmap: &mut PixmapMut,
    hub_cx: f32,
    hub_cy: f32,
    scale: f32,
    norm_rpm: f32,
    redline_pct: f32,
    accent: Color,
) {
    let _ = pixmap;
    let _ = hub_cx;
    let _ = hub_cy;
    let _ = scale;
    let _ = norm_rpm;
    let _ = redline_pct;
    let _ = accent;
}

pub fn hood_view_forces_scrim_zero(camera_index: i32) -> bool {
    camera_index == 4
}

pub fn should_draw_scrim(camera_index: i32, oem_pct: i32) -> bool {
    camera_index == 3 && oem_pct > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hood_forces_scrim_zero() {
        assert!(hood_view_forces_scrim_zero(4));
        assert!(!hood_view_forces_scrim_zero(3));
    }

    #[test]
    fn scrim_gate() {
        assert!(should_draw_scrim(3, 50));
        assert!(!should_draw_scrim(4, 50));
        assert!(!should_draw_scrim(3, 0));
    }
}
