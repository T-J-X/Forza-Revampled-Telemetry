use crate::draw_ext::*;
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_glitch_text, draw_text};
use tiny_skia::{Color, PixmapMut, Rect};

fn sanitize(v: f32, min_v: f32, max_v: f32) -> f32 {
    if !v.is_finite() {
        min_v
    } else {
        v.clamp(min_v, max_v)
    }
}

fn premultiply(r: u8, g: u8, b: u8, a: u8) -> (u8, u8, u8, u8) {
    if a < 2 {
        return (0, 0, 0, 0);
    }
    let a_f = a as f32 / 255.0;
    let r_p = ((r as f32 * a_f).round() as u8).min(r);
    let g_p = ((g as f32 * a_f).round() as u8).min(g);
    let b_p = ((b as f32 * a_f).round() as u8).min(b);
    (r_p, g_p, b_p, a)
}

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let cx = bounds.x() + bounds.width() * 0.5;
    let cy = bounds.y() + bounds.height() * 0.5;
    let scale = 1.0;
    let norm = sanitize(data.normalized_rpm(), 0.0, 1.0);
    let _power_watts = data.power_watts;
    let _pw = _power_watts;

    render_bezel(pixmap, cx, cy, scale);
    render_shift_lights(pixmap, cx, cy, scale, norm);
    render_telemetry_pods(pixmap, cx, cy, scale, data);

    let tag = "ØG『死』ŦJ乂";
    let tr = 255;
    let tg = 255;
    let tb = 255;
    let ta = 200;
    let (pr, pg, pb, pa) = premultiply(tr, tg, tb, ta);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    if config.driver_tag_glitch_fx {
        draw_glitch_text(
            pixmap,
            cx - 120.0 * scale,
            cy - 130.0 * scale,
            tag,
            col,
            scale,
            data.timestamp_ms as u64,
            true,
        );
    } else {
        draw_text(pixmap, cx - 120.0 * scale, cy - 130.0 * scale, tag, col, scale);
    }
}

fn render_bezel(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32) {
    let r = 255;
    let g = 255;
    let b = 255;
    let a = 40;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    fill_rect_xywh(
        pixmap,
        cx - 320.0 * scale,
        cy - 150.0 * scale,
        640.0 * scale,
        300.0 * scale,
        col,
    );
}

fn render_shift_lights(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, norm: f32) {
    let shift_start = 0.72;
    let redline = 0.94;
    let prog = ((norm - shift_start).max(0.0) / ((redline - shift_start).max(0.01))).min(1.0);
    let lit_count = (prog * 16.0).ceil() as usize;
    for i in 0..16 {
        let lx = cx - 195.0 * scale + i as f32 * 26.0 * scale;
        let ly = cy - 115.0 * scale;
        if i < lit_count {
            let (r, g, b) = if i < 8 { (0, 176, 255) } else { (255, 23, 68) };
            let (pr, pg, pb, pa) = premultiply(r, g, b, 255);
            let col = Color::from_rgba8(pr, pg, pb, pa);
            fill_rect_xywh(
                pixmap,
                lx - 5.8 * scale,
                ly - 5.8 * scale,
                11.6 * scale,
                11.6 * scale,
                col,
            );
        }
    }
}

fn render_telemetry_pods(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let speed = sanitize(data.speed_mph(), 0.0, 250.0);
    let r = 255;
    let g = 255;
    let b = 255;
    let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    draw_text(
        pixmap,
        cx - 200.0 * scale,
        cy + 50.0 * scale,
        &format!("{:.0}", speed),
        col,
        scale * 1.5,
    );
    draw_text(
        pixmap,
        cx + 200.0 * scale,
        cy + 50.0 * scale,
        &data.gear_string(),
        col,
        scale * 1.5,
    );
}
