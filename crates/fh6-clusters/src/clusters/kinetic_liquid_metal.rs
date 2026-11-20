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
    let g_lat = sanitize(data.lateral_g(), -3.0, 3.0);
    let g_long = sanitize(data.longitudinal_g(), -2.0, 2.0);
    let _power_watts = data.power_watts;
    let _pw = _power_watts;
    
    let theta_slosh = (g_lat / (1.0 + g_long.abs())).atan();
    render_liquid_metal_arc(pixmap, cx, cy, scale, rpm_ratio, theta_slosh);
    render_liquid_metal_slosh(pixmap, cx, cy, scale, theta_slosh, rpm_ratio);
    
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

fn render_liquid_metal_arc(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, rpm_ratio: f32, _theta: f32) {
    let radius = 120.0 * scale;
    let r = 200; let g = 200; let b = 220; let a = 180;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    for i in 0..180 {
        let theta = i as f32 * std::f32::consts::PI / 180.0;
        let x = cx + radius * theta.cos();
        let y = cy + radius * theta.sin();
        fill_rect_xywh(pixmap, x-1.0, y-1.0, 2.0, 2.0, col);
    }
}

fn render_liquid_metal_slosh(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, theta_slosh: f32, rpm_ratio: f32) {
    let level = 0.5 + 0.5 * rpm_ratio;
    let omega = rpm_ratio * 0.08;
    let r = 220; let g = 220; let b = 240; let a = 200;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    for i in 0..100 {
        let x = cx - 100.0*scale + i as f32 * 2.0*scale;
        let y = cy + theta_slosh * 40.0*scale + (i as f32 * omega).sin() * 10.0*scale;
        fill_rect_xywh(pixmap, x-1.0, y-1.0, 2.0, 2.0, col);
    }
}
