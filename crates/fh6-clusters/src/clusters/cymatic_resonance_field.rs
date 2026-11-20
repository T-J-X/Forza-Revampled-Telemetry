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
    let rpm = sanitize(data.current_engine_rpm, 0.0, 10000.0);
    let v_kmh = sanitize(data.speed_kmh(), 0.0, 300.0);
    let _power_watts = data.power_watts;
    let _pw = _power_watts;
    
    let n = ((rpm / 1200.0).clamp(1.0, 6.0)) as f32;
    let m = ((v_kmh / 40.0).clamp(1.0, 6.0)) as f32;
    
    render_chladni_field(pixmap, cx, cy, scale, n, m);
    render_telemetry(pixmap, cx, cy, scale, data);
    
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

fn render_chladni_field(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, n: f32, m: f32) {
    let L = 260.0 * scale;
    let r = 56; let g = 189; let b = 248; let a = 200;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    for i in 0..200 {
        for j in 0..200 {
            let nx = (i as f32 / 200.0 - 0.5) * 2.0;
            let ny = (j as f32 / 200.0 - 0.5) * 2.0;
            let term1 = (n * std::f32::consts::PI * nx).cos() * (m * std::f32::consts::PI * ny).cos();
            let term2 = (m * std::f32::consts::PI * nx).cos() * (n * std::f32::consts::PI * ny).cos();
            let val = term1 - term2;
            if val.abs() < 0.1 {
                let x = cx + nx * L * 0.5;
                let y = cy + ny * L * 0.5;
                fill_rect_xywh(pixmap, x-1.0, y-1.0, 2.0, 2.0, col);
            }
        }
    }
}

fn render_telemetry(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let speed = sanitize(data.speed_mph(), 0.0, 300.0);
    let r = 255; let g = 255; let b = 255; let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    draw_text(pixmap, cx, cy - 40.0*scale, &format!("{:.0}", speed), col, scale*2.0);
    draw_text(pixmap, cx, cy + 40.0*scale, &data.gear_string(), col, scale*2.0);
}
