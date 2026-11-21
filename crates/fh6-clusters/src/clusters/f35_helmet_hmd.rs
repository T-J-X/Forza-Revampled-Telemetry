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
    // TODO(phase-7): meant to drive an RPM or shift cue on the HMD (computed but never used).
    let _rpm_ratio = sanitize(data.normalized_rpm(), 0.0, 1.2);
    let _power_watts = data.power_watts;
    let _pw = _power_watts;

    render_pitch_ladder(pixmap, cx, cy, scale, data);
    render_flight_path_marker(pixmap, cx, cy, scale, data);
    render_airspeed_altitude_tapes(pixmap, cx, cy, scale, data);
    render_reticle(pixmap, cx, cy, scale);

    let tag = "ØG『死』ŦJ乂";
    let tr = 0;
    let tg = 255;
    let tb = 102;
    let ta = 255;
    let (pr, pg, pb, pa) = premultiply(tr, tg, tb, ta);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    if config.driver_tag_glitch_fx {
        draw_glitch_text(
            pixmap,
            cx,
            cy + 120.0 * scale,
            tag,
            col,
            scale * 0.8,
            data.timestamp_ms as u64,
            true,
        );
    } else {
        draw_text(pixmap, cx, cy + 120.0 * scale, tag, col, scale * 0.8);
    }
}

fn rotate_point(x: f32, y: f32, roll: f32) -> (f32, f32) {
    let c = (-roll).cos();
    let s = (-roll).sin();
    let x_p = c * x - s * y;
    let y_p = s * x + c * y;
    (x_p, y_p)
}

fn render_pitch_ladder(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let pitch = sanitize(data.pitch, -0.5, 0.5);
    let roll = sanitize(data.roll, -0.5, 0.5);
    let r = 0;
    let g = 255;
    let b = 102;
    let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    let k = 40.0;
    for i in -3..=3 {
        let y_local = i as f32 * 26.0 * scale + pitch * k * scale;
        let x1 = -40.0 * scale;
        let y1 = y_local;
        let x2 = 40.0 * scale;
        let y2 = y_local;
        let (rx1, ry1) = rotate_point(x1, y1, roll);
        let (rx2, ry2) = rotate_point(x2, y2, roll);
        stroke_line(pixmap, cx + rx1, cy + ry1, cx + rx2, cy + ry2, 1.5 * scale, col);
    }
}

fn render_flight_path_marker(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let beta_deg = sanitize(data.car_slip_angle_deg(), -90.0, 90.0);
    let x_tip = cx + beta_deg * 1.8 * scale;
    let y_tip = cy;
    let r = 0;
    let g = 255;
    let b = 102;
    let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    stroke_line(
        pixmap,
        x_tip - 11.5 * scale,
        y_tip,
        x_tip + 11.5 * scale,
        y_tip,
        2.0 * scale,
        col,
    );
    stroke_line(
        pixmap,
        x_tip,
        y_tip - 11.5 * scale,
        x_tip,
        y_tip + 11.5 * scale,
        2.0 * scale,
        col,
    );
}

fn render_airspeed_altitude_tapes(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let speed = sanitize(data.speed_mph(), 0.0, 400.0);
    let r = 0;
    let g = 255;
    let b = 102;
    let a = 180;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col_bg = Color::from_rgba8(pr, pg, pb, pa);
    let left_x = cx - 196.0 * scale;
    fill_rect_xywh(
        pixmap,
        left_x - 56.0 * scale,
        cy - 25.0 * scale,
        112.0 * scale,
        50.0 * scale,
        col_bg,
    );
    let r_txt = 0;
    let g_txt = 255;
    let b_txt = 102;
    let a_txt = 255;
    let (prt, pgt, pbt, pat) = premultiply(r_txt, g_txt, b_txt, a_txt);
    let col_txt = Color::from_rgba8(prt, pgt, pbt, pat);
    draw_text(pixmap, left_x, cy, &format!("{:.0}", speed), col_txt, scale * 1.5);
}

fn render_reticle(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32) {
    let r = 0;
    let g = 255;
    let b = 102;
    let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    // Collimated green reticle
    for &x in &[-10.0 * scale, 10.0 * scale] {
        stroke_line(
            pixmap,
            cx + x,
            cy - 5.0 * scale,
            cx + x,
            cy + 5.0 * scale,
            1.0 * scale,
            col,
        );
    }
    for &y in &[-10.0 * scale, 10.0 * scale] {
        stroke_line(
            pixmap,
            cx - 5.0 * scale,
            cy + y,
            cx + 5.0 * scale,
            cy + y,
            1.0 * scale,
            col,
        );
    }
}
