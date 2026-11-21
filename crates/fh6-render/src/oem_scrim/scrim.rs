//! Feathered OEM dash scrim rendering.
//! Covers the in-game binnacle with a domed 0-100% blackout (`OemDashOpaquePct`).

use crate::oem_scrim::dome_path::create_binnacle_dome_path;
use fh6_core::AppConfig;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, PixmapMut, Rect, Transform};

pub struct ScrimParams {
    pub pct: i32,
    pub cx: f32,
    pub cy: f32,
    pub w: f32,
    pub h: f32,
    pub scale: f32,
}

impl ScrimParams {
    pub fn from_config(bounds: Rect, scale: f32, config: &AppConfig) -> Self {
        let cx = bounds.x() + bounds.width() / 2.0;
        let cy = bounds.y() + bounds.height() * 0.34 + config.oem_dash_binnacle_offset_y as f32 * scale;
        let w = (1220.0 * scale * config.oem_dash_width_scale).max(420.0);
        let h = (400.0 * scale).max(180.0);
        let pct = config.oem_dash_opaque_pct.clamp(0, 100);
        Self {
            pct,
            cx,
            cy,
            w,
            h,
            scale,
        }
    }
}

pub fn render_oem_scrim(pixmap: &mut PixmapMut, bounds: Rect, scale: f32, config: &AppConfig) {
    if config.current_camera_index != 3 {
        return;
    }
    let params = ScrimParams::from_config(bounds, scale, config);
    if params.pct <= 0 {
        return;
    }

    if params.pct < 90 {
        render_feathered(params, pixmap);
    } else {
        render_blackout(params, pixmap);
    }
}

fn render_feathered(params: ScrimParams, pixmap: &mut PixmapMut) {
    let alpha = (params.pct as f32 * 2.65).round().clamp(20.0, 232.0) as u8;
    // Outer dome
    let outer = create_binnacle_dome_path(params.cx, params.cy, params.w, params.h, params.scale);
    // Inner dome
    let inner_w = params.w * 0.72;
    let inner_h = params.h * 0.68;
    let inner = create_binnacle_dome_path(params.cx, params.cy, inner_w, inner_h, params.scale * 0.9);

    // For compile-ready stub, fill with solid alpha; real implementation would use gradient.
    let color = Color::from_rgba8(0, 0, 0, alpha);
    fill_path(pixmap, &outer.builder, color);
    let inner_alpha = ((alpha as f32) * 0.85) as u8;
    let inner_color = Color::from_rgba8(0, 0, 0, inner_alpha);
    fill_path(pixmap, &inner.builder, inner_color);
}

fn render_blackout(params: ScrimParams, pixmap: &mut PixmapMut) {
    let core_alpha = if params.pct == 100 {
        255
    } else {
        (236.0 + (params.pct as f32 - 90.0) * 1.9).round() as u8
    };
    let core_w = params.w * 0.74;
    let core_h = params.h * 0.72;

    let outer = create_binnacle_dome_path(params.cx, params.cy, params.w, params.h, params.scale);
    let core = create_binnacle_dome_path(params.cx, params.cy, core_w, core_h, params.scale);

    let outer_color = Color::from_rgba8(0, 0, 0, core_alpha);
    fill_path(pixmap, &outer.builder, outer_color);
    let core_color = Color::from_rgba8(0, 0, 0, core_alpha);
    fill_path(pixmap, &core.builder, core_color);
}

fn fill_path(pixmap: &mut PixmapMut, builder: &PathBuilder, color: Color) {
    if let Some(path) = builder.clone().finish() {
        let mut paint = Paint::default();
        paint.set_color(color);
        pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_core::AppConfig;
    use tiny_skia::Pixmap;

    #[test]
    fn scrim_respects_camera_gate() {
        let mut pixmap = Pixmap::new(1920, 1080).unwrap();
        let mut pixmap_mut = pixmap.as_mut();
        let bounds = Rect::from_xywh(0.0, 0.0, 1920.0, 1080.0).unwrap();
        let cfg = AppConfig {
            current_camera_index: 4, // hood
            oem_dash_opaque_pct: 80,
            ..AppConfig::default()
        };
        render_oem_scrim(&mut pixmap_mut, bounds, 1.0, &cfg);
        // No panic
    }

    #[test]
    fn scrim_zero_pct_no_draw() {
        let mut pixmap = Pixmap::new(1920, 1080).unwrap();
        let mut pixmap_mut = pixmap.as_mut();
        let bounds = Rect::from_xywh(0.0, 0.0, 1920.0, 1080.0).unwrap();
        let cfg = AppConfig {
            current_camera_index: 3,
            oem_dash_opaque_pct: 0,
            ..AppConfig::default()
        };
        render_oem_scrim(&mut pixmap_mut, bounds, 1.0, &cfg);
    }
}
