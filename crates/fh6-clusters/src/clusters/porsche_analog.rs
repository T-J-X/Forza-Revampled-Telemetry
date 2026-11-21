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
    let rpm = sanitize(data.current_engine_rpm, 0.0, 9000.0);
    let _power_watts = data.power_watts;
    let _pw = _power_watts;

    render_tach_dial(pixmap, cx, cy, scale, rpm);
    render_carbon_wings(pixmap, cx, cy, scale);
    render_needle(pixmap, cx, cy, scale, rpm);
    render_side_pods(pixmap, cx, cy, scale, data);

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
            cx - 130.0 * scale,
            cy - 140.0 * scale,
            tag,
            col,
            scale,
            data.timestamp_ms as u64,
            true,
        );
    } else {
        draw_text(pixmap, cx - 130.0 * scale, cy - 140.0 * scale, tag, col, scale);
    }
}

// TODO(phase-7): `_rpm` was meant to light the dial ticks or the sequential shift ring up to the current RPM.
fn render_tach_dial(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, _rpm: f32) {
    let r = 140.0 * scale;
    // Major ticks 1000 RPM
    for i in 0..=9 {
        let angle = i as f32 * std::f32::consts::PI / 9.0 - std::f32::consts::PI / 2.0;
        let x1 = cx + r * angle.cos();
        let y1 = cy + r * angle.sin();
        let x2 = cx + (r - 10.0 * scale) * angle.cos();
        let y2 = cy + (r - 10.0 * scale) * angle.sin();
        let rr = 255;
        let gg = 255;
        let bb = 255;
        let aa = 255;
        let (pr, pg, pb, pa) = premultiply(rr, gg, bb, aa);
        let col = Color::from_rgba8(pr, pg, pb, pa);
        stroke_line(pixmap, x1, y1, x2, y2, 2.0 * scale, col);
    }
    // Minor ticks 250 RPM
    for i in 0..36 {
        let angle = i as f32 * std::f32::consts::PI / 18.0 - std::f32::consts::PI / 2.0;
        let x1 = cx + r * angle.cos();
        let y1 = cy + r * angle.sin();
        let x2 = cx + (r - 5.0 * scale) * angle.cos();
        let y2 = cy + (r - 5.0 * scale) * angle.sin();
        let rr = 180;
        let gg = 180;
        let bb = 180;
        let aa = 200;
        let (pr, pg, pb, pa) = premultiply(rr, gg, bb, aa);
        let col = Color::from_rgba8(pr, pg, pb, pa);
        stroke_line(pixmap, x1, y1, x2, y2, 1.0 * scale, col);
    }
}

fn render_carbon_wings(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32) {
    // Left wing
    let r = 255;
    let g = 255;
    let b = 255;
    let a = 60;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    for i in 0..20 {
        let x = cx - 180.0 * scale + i as f32 * 8.0 * scale;
        fill_rect_xywh(pixmap, x, cy - 20.0 * scale, 4.0 * scale, 40.0 * scale, col);
    }
    // Right wing
    for i in 0..20 {
        let x = cx + 180.0 * scale - i as f32 * 8.0 * scale;
        fill_rect_xywh(pixmap, x, cy - 20.0 * scale, 4.0 * scale, 40.0 * scale, col);
    }
}

fn render_needle(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, rpm: f32) {
    let angle = (rpm / 9000.0).clamp(0.0, 1.0) * std::f32::consts::PI - std::f32::consts::PI / 2.0;
    let x = cx + 120.0 * scale * angle.cos();
    let y = cy + 120.0 * scale * angle.sin();
    let r = 255;
    let g = 255;
    let b = 255;
    let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    stroke_line(pixmap, cx, cy, x, y, 2.0 * scale, col);
}

fn render_side_pods(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let speed = sanitize(data.speed_mph(), 0.0, 250.0);
    let r = 255;
    let g = 255;
    let b = 255;
    let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    draw_text(
        pixmap,
        cx - 120.0 * scale,
        cy + 80.0 * scale,
        &format!("{:.0}", speed),
        col,
        scale * 1.3,
    );
    draw_text(pixmap, cx - 120.0 * scale, cy + 110.0 * scale, "888", col, scale * 0.8);
    draw_text(
        pixmap,
        cx + 120.0 * scale,
        cy + 80.0 * scale,
        &data.gear_string(),
        col,
        scale * 1.3,
    );
    draw_text(pixmap, cx + 120.0 * scale, cy + 110.0 * scale, "oil", col, scale * 0.8);
}
