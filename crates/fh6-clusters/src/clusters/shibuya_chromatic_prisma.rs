use tiny_skia::{Color, PixmapMut, Rect};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_glitch_text};
use crate::draw_ext::*;

fn sanitize(v: f32, min_v: f32, max_v: f32) -> f32 {
    if !v.is_finite() { min_v } else { v.clamp(min_v, max_v) }
}

fn premultiply(r: u8, g: u8, b: u8, a: u8) -> (u8, u8, u8, u8) {
    if a < 2 { return (0,0,0,0); }
    let a_f = a as f32 / 255.0;
    let r_p = ((r as f32 * a_f).round() as u8).min(r);
    let g_p = ((g as f32 * a_f).round() as u8).min(g);
    let b_p = ((b as f32 * a_f).round() as u8).min(b);
    (r_p, g_p, b_p, a)
}

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let cx = bounds.x() + bounds.width()*0.5;
    let cy = bounds.y() + bounds.height()*0.5;
    let scale = 1.0;
    let rpm_ratio = sanitize(data.normalized_rpm(), 0.0, 1.2);
    let yaw_rate = sanitize(data.angular_velocity_y, -2.0, 2.0);
    let slip = sanitize(data.car_slip_angle_rad().abs(), 0.0, 0.5);
    let _power_watts = data.power_watts;
    let _pw = _power_watts;
    
    let dx_rgb = (yaw_rate * 6.0 + slip * 18.0) * scale;
    render_chromatic_passes(pixmap, cx, cy, scale, dx_rgb, rpm_ratio);
    
    let tag = "ØG『死』ŦJ乂";
    let tr = 255; let tg = 255; let tb = 255; let ta = 200;
    let (pr, pg, pb, pa) = premultiply(tr, tg, tb, ta);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx, cy + 140.0*scale, tag, col, scale*0.6, data.timestamp_ms as u64, true);
    } else {
        draw_text(pixmap, cx, cy + 140.0*scale, tag, col, scale*0.6);
    }
}

fn render_chromatic_passes(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, dx: f32, rpm_ratio: f32) {
    let passes = [
        (-dx, 255, 0, 0, 120),
        (0.0, 0, 255, 0, 120),
        (dx, 0, 0, 255, 120),
    ];
    for &(offset, r, g, b, a) in passes.iter() {
        let (pr, pg, pb, pa) = premultiply(r, g, b, a);
        let col = Color::from_rgba8(pr, pg, pb, pa);
        let ox = offset;
        // Grid
        for i in 0..=10 {
            let x = cx - 250.0*scale + i as f32 * 50.0*scale + ox;
            stroke_line(pixmap, x, cy - 150.0*scale, x, cy + 150.0*scale, 1.0*scale, col);
        }
        // Tach sweep
        let sweep = 180.0*scale * rpm_ratio;
        for i in 0..=10 {
            let x = cx - sweep/2.0 + i as f32 * sweep/10.0 + ox;
            fill_rect_xywh(pixmap, x, cy - 80.0*scale - 3.0, 2.0, 6.0, col);
        }
    }
}
