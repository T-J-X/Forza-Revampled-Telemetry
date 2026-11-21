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
    let slip = sanitize(data.rear_tire_slip_angle_rad().abs(), 0.0, 0.5);
    let _power_watts = data.power_watts;
    let _pw = _power_watts;
    
    render_magi_matrix(pixmap, cx, cy, scale, data, rpm_ratio);
    render_harmonic_sync(pixmap, cx, cy, scale, rpm_ratio, slip);
    render_at_field(pixmap, cx, cy, scale, data);
    
    let tag = "ØG『死』ŦJ乂";
    let tr = 255; let tg = 153; let tb = 0; let ta = 255;
    let (pr, pg, pb, pa) = premultiply(tr, tg, tb, ta);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx, cy + 130.0*scale, tag, col, scale*0.7, data.timestamp_ms as u64, true);
    } else {
        draw_text(pixmap, cx, cy + 130.0*scale, tag, col, scale*0.7);
    }
}

// TODO(phase-7): `_data` and `_rpm_ratio` were meant to drive the MAGI node states (the hex nodes are static).
fn render_magi_matrix(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, _data: &FH6TelemetryData, _rpm_ratio: f32) {
    let names = ["MELCHIOR", "BALTHASAR", "CASPER"];
    let r = 255; let g = 176; let b = 32; let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    for (i, name) in names.iter().enumerate() {
        let x = cx - 282.0*scale + i as f32 *124.0*scale;
        draw_text(pixmap, x, cy -118.0*scale, name, col, scale*0.66);
    }
    // Hex nodes
    for &i in &[0,1,2] {
        let x = cx - 282.0*scale + i as f32 *124.0*scale;
        let y = cy - 80.0*scale;
        let r_hex = 255; let g_hex = 153; let b_hex = 0; let a_hex = 180;
        let (prh, pgh, pbh, pah) = premultiply(r_hex, g_hex, b_hex, a_hex);
        let col_hex = Color::from_rgba8(prh, pgh, pbh, pah);
        draw_hexagon_cell(pixmap, x, y, 22.0*scale, 1.5*scale, col_hex, col_hex);
    }
}

fn render_harmonic_sync(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, rpm_ratio: f32, slip: f32) {
    let sync = 84.0 + 15.0 * rpm_ratio - 10.0 * slip;
    let r = 0; let g = 255; let b = 102; let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    draw_text(pixmap, cx, cy - 86.0*scale + 28.0*scale, &format!("SYNC {:.0}%", sync), col, scale*0.7);
}

fn render_at_field(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let g_long = sanitize(data.longitudinal_g(), -2.0, 2.0);
    if g_long < -0.2 {
        let r = 255; let g = 136; let b = 0; let a = 40;
        let (pr, pg, pb, pa) = premultiply(r, g, b, a);
        let col = Color::from_rgba8(pr, pg, pb, pa);
        for r_idx in 0..4 {
            let rad = 30.0*scale + r_idx as f32 * 12.0*scale;
            let mut pts = Vec::new();
            for i in 0..8 {
                let a_theta = i as f32 * std::f32::consts::PI / 4.0;
                pts.push((cx + rad * a_theta.cos(), cy + rad * a_theta.sin()));
            }
            fill_polygon(pixmap, &pts, col);
        }
    }
}
