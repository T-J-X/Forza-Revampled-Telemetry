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
    let _power_watts = data.power_watts;
    let _pw = _power_watts;
    
    render_slip_arc(pixmap, cx, cy, scale, data);
    render_g_force_circle(pixmap, cx, cy, scale, data);
    render_speed_gear(pixmap, cx, cy, scale, data);
    
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

fn render_slip_arc(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let slip = sanitize(data.car_slip_angle_rad(), -0.5, 0.5);
    let norm = (slip + 0.5) / 1.0;
    let start = 135.0_f32.to_radians();
    let end = start + 270.0_f32.to_radians() * norm;
    let r = 120.0 * scale;
    let r_col = 255; let g_col = 255; let b_col = 255; let a_col = 200;
    let (pr, pg, pb, pa) = premultiply(r_col, g_col, b_col, a_col);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    for i in 0..100 {
        let t = i as f32 / 99.0;
        let angle = start + t * (end - start);
        let x = cx + r * angle.cos();
        let y = cy + r * angle.sin();
        fill_rect_xywh(pixmap, x-1.0, y-1.0, 2.0, 2.0, col);
    }
}

fn render_g_force_circle(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let g_lat = sanitize(data.lateral_g(), -1.5, 1.5);
    let g_long = sanitize(data.longitudinal_g(), -1.5, 1.5);
    let r = 80.0 * scale;
    let r_col = 255; let g_col = 100; let b_col = 100; let a_col = 200;
    let (pr, pg, pb, pa) = premultiply(r_col, g_col, b_col, a_col);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    for i in 0..360 {
        let angle = i as f32 * std::f32::consts::PI / 180.0;
        let x = cx + r * angle.cos();
        let y = cy + r * angle.sin();
        fill_rect_xywh(pixmap, x-1.0, y-1.0, 2.0, 2.0, col);
    }
    let x_ball = cx + g_lat * r * 0.5;
    let y_ball = cy + g_long * r * 0.5;
    let r_ball = 10; let g_ball = 255; let b_ball = 100; let a_ball = 255;
    let (prb, pgb, pbb, pab) = premultiply(r_ball, g_ball, b_ball, a_ball);
    let col_ball = Color::from_rgba8(prb, pgb, pbb, pab);
    fill_rect_xywh(pixmap, x_ball-5.0, y_ball-5.0, 10.0, 10.0, col_ball);
}

fn render_speed_gear(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let speed = sanitize(data.speed_mph(), 0.0, 250.0);
    let r = 255; let g = 255; let b = 255; let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    draw_text(pixmap, cx - 60.0*scale, cy + 40.0*scale, &format!("{:.0}", speed), col, scale*1.2);
    draw_text(pixmap, cx + 100.0*scale, cy + 10.0*scale, &data.gear_string(), col, scale*1.8);
}
