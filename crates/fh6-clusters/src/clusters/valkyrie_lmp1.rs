use crate::draw_ext::*;
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_glitch_text, draw_text};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_skia::{Color, PixmapMut, Rect};

fn sanitize(v: f32, min_v: f32, max_v: f32) -> f32 {
    if !v.is_finite() {
        return min_v;
    }
    v.clamp(min_v, max_v)
}

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let cx = bounds.x() + bounds.width() * 0.5;
    let cy = bounds.y() + bounds.height() * 0.5;
    let scale = 1.0_f32;

    fill_rect_xywh(
        pixmap,
        bounds.x(),
        bounds.y(),
        bounds.width(),
        bounds.height(),
        Color::from_rgba8(0, 0, 0, 255),
    );

    let _norm_rpm = sanitize(data.normalized_rpm(), 0.0, 1.2);
    let speed_mph = sanitize(data.speed_mph(), 0.0, 500.0);
    let speed_kmh = sanitize(data.speed_kmh(), 0.0, 800.0);
    let rpm = sanitize(data.current_engine_rpm, 0.0, 12000.0);
    let max_rpm = sanitize(data.engine_max_rpm, 1.0, 15000.0);
    let rpm_ratio = (rpm / max_rpm).clamp(0.0, 1.2);

    render_carbon_wheel_enclosure(pixmap, cx, cy, scale);
    render_15_led_marshal_array(pixmap, cx, cy, scale, rpm_ratio);
    render_4_corner_tire_matrix(pixmap, cx, cy, scale, data);
    render_central_lmp1_telemetry(pixmap, cx, cy, scale, data, speed_mph, speed_kmh, rpm_ratio);

    let tag = "ØG『死』ŦJ乂";
    let tag_x = cx - 130.0 * scale;
    let tag_y = cy - 140.0 * scale;
    if config.driver_tag_glitch_fx {
        draw_glitch_text(
            pixmap,
            tag_x,
            tag_y,
            tag,
            Color::from_rgba8(212, 255, 0, 255),
            scale,
            data.timestamp_ms as u64,
            true,
        );
    } else {
        draw_text(pixmap, tag_x, tag_y, tag, Color::from_rgba8(212, 255, 0, 255), scale);
    }
}

fn render_carbon_wheel_enclosure(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32) {
    let w = 592.0 * scale;
    let h = 268.0 * scale;
    let x = cx - w * 0.5;
    let y = cy - h * 0.5;
    fill_rect_xywh(pixmap, x, y, w, h, Color::from_rgba8(17, 19, 24, 255));
    let hatch_color = Color::from_rgba8(0, 0, 0, 60);
    for i in -10..10 {
        let offset = i as f32 * 12.0 * scale;
        stroke_line(
            pixmap,
            cx + offset,
            y,
            cx + offset - 120.0 * scale,
            y + h,
            1.6 * scale,
            hatch_color,
        );
    }
    let cavity_w = w * 0.92;
    let cavity_h = h * 0.88;
    fill_rect_xywh(
        pixmap,
        cx - cavity_w * 0.5,
        cy - cavity_h * 0.5,
        cavity_w,
        cavity_h,
        Color::from_rgba8(0, 0, 0, 255),
    );
    let stripe = Color::from_rgba8(212, 255, 0, 255);
    stroke_rect_xywh(pixmap, x, y, w, h, 2.5 * scale, stripe);
}

fn render_15_led_marshal_array(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, rpm_ratio: f32) {
    let top_y = cy - 110.0 * scale;
    let led_count = 15;
    let total_w = 432.0 * scale;
    let step = total_w / led_count as f32;
    let lit_cnt = ((rpm_ratio.clamp(0.0, 1.0) * led_count as f32).round() as usize).min(led_count);
    let now = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f32)
        / 1000.0;
    let strobe = rpm_ratio > 0.96 && ((now * 18.0) as i32 % 2 == 0);
    for i in 0..led_count {
        let lx = cx - total_w * 0.5 + (i as f32 + 0.5) * step;
        let led_y = top_y;
        let lit = strobe || (i < lit_cnt);
        let alpha = if lit { 250 } else { 42 };
        let active = if strobe {
            Color::from_rgba8(255, 255, 255, alpha)
        } else if i < 5 {
            Color::from_rgba8(16, 185, 129, alpha)
        } else if i < 10 {
            Color::from_rgba8(239, 68, 68, alpha)
        } else {
            Color::from_rgba8(59, 130, 246, alpha)
        };
        fill_rect_xywh(
            pixmap,
            lx - 6.0 * scale,
            led_y - 6.0 * scale,
            12.0 * scale,
            12.0 * scale,
            active,
        );
        stroke_rect_xywh(
            pixmap,
            lx - 7.8 * scale,
            led_y - 7.8 * scale,
            15.6 * scale,
            15.6 * scale,
            1.1 * scale,
            Color::from_rgba8(120, 136, 156, 145),
        );
    }
}

fn render_4_corner_tire_matrix(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, _data: &FH6TelemetryData) {
    let corners = [
        ("FL", cx - 270.0 * scale, cy + 16.0 * scale),
        ("RL", cx - 178.0 * scale, cy + 16.0 * scale),
        ("FR", cx + 92.0 * scale, cy + 16.0 * scale),
        ("RR", cx + 184.0 * scale, cy + 16.0 * scale),
    ];
    for (tag, tx, ty) in corners.iter() {
        let w = 86.0 * scale;
        let h = 78.0 * scale;
        let temp_c = 85.0 + (tx.abs() % 12.0);
        let fill = if temp_c < 70.0 {
            Color::from_rgba8(2, 132, 199, 205)
        } else if temp_c < 90.0 {
            Color::from_rgba8(22, 163, 74, 205)
        } else if temp_c < 110.0 {
            Color::from_rgba8(217, 119, 6, 205)
        } else {
            Color::from_rgba8(220, 38, 38, 205)
        };
        fill_rect_xywh(pixmap, *tx, *ty, w, h, fill);
        stroke_rect_xywh(pixmap, *tx, *ty, w, h, 1.6 * scale, Color::from_rgba8(0, 230, 118, 255));
        draw_text(
            pixmap,
            *tx + 5.0 * scale,
            *ty + 10.0 * scale,
            tag,
            Color::from_rgba8(255, 255, 255, 255),
            scale * 0.6,
        );
        draw_text(
            pixmap,
            *tx + w * 0.5,
            *ty + 30.0 * scale,
            &format!("{:.0}C", temp_c),
            Color::from_rgba8(255, 255, 255, 255),
            scale * 1.0,
        );
    }
}

fn render_central_lmp1_telemetry(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    scale: f32,
    data: &FH6TelemetryData,
    speed_mph: f32,
    _speed_kmh: f32,
    rpm_ratio: f32,
) {
    let gear_box_w = 104.0 * scale;
    let gear_box_h = 86.0 * scale;
    let gx = cx - gear_box_w * 0.5;
    let gy = cy - 84.0 * scale;
    fill_rect_xywh(
        pixmap,
        gx,
        gy,
        gear_box_w,
        gear_box_h,
        Color::from_rgba8(18, 22, 12, 245),
    );
    let gear_col = if rpm_ratio >= 0.96 {
        Color::from_rgba8(255, 23, 68, 255)
    } else {
        Color::from_rgba8(212, 255, 0, 255)
    };
    stroke_rect_xywh(pixmap, gx, gy, gear_box_w, gear_box_h, 2.2 * scale, gear_col);
    draw_text(
        pixmap,
        cx,
        gy + 40.0 * scale,
        &data.gear_string(),
        gear_col,
        scale * 2.5,
    );

    let spd_w = 136.0 * scale;
    let spd_h = 42.0 * scale;
    let sx = cx - spd_w * 0.5;
    let sy = cy + 8.0 * scale;
    fill_rect_xywh(pixmap, sx, sy, spd_w, spd_h, Color::from_rgba8(14, 18, 26, 240));
    stroke_rect_xywh(
        pixmap,
        sx,
        sy,
        spd_w,
        spd_h,
        1.2 * scale,
        Color::from_rgba8(96, 110, 132, 160),
    );
    draw_text(
        pixmap,
        cx,
        sy + 22.0 * scale,
        &format!("{:.0}", speed_mph),
        Color::from_rgba8(255, 255, 255, 255),
        scale * 1.8,
    );

    let delta_w = 144.0 * scale;
    let delta_h = 26.0 * scale;
    let dx = cx - delta_w * 0.5;
    let dy = cy + 68.0 * scale;
    fill_rect_xywh(pixmap, dx, dy, delta_w, delta_h, Color::from_rgba8(8, 52, 28, 230));
    stroke_rect_xywh(
        pixmap,
        dx,
        dy,
        delta_w,
        delta_h,
        1.4 * scale,
        Color::from_rgba8(0, 230, 118, 255),
    );
    draw_text(
        pixmap,
        cx,
        dy + 13.0 * scale,
        "-0.284s QUALI",
        Color::from_rgba8(0, 255, 136, 255),
        scale * 0.8,
    );

    let bar_w = 80.0 * scale;
    let bar_h = 6.0 * scale;
    let bx = cx - bar_w * 0.5;
    let by = cy + 110.0 * scale;
    fill_rect_xywh(pixmap, bx, by, bar_w, bar_h, Color::from_rgba8(60, 60, 60, 255));
    let ers_fill = bar_w * sanitize(data.throttle_pct(), 0.0, 1.0);
    fill_rect_xywh(pixmap, bx, by, ers_fill, bar_h, Color::from_rgba8(56, 189, 248, 255));
    let bias_fill = bar_w * sanitize(data.brake_pct(), 0.0, 1.0);
    fill_rect_xywh(
        pixmap,
        bx,
        by + 12.0 * scale,
        bias_fill,
        bar_h,
        Color::from_rgba8(239, 68, 68, 255),
    );

    // Additional sub-pods: power and torque readout
    let hp = (data.power_watts / 745.7).max(0.0);
    let hp_str = format!("PWR {:.0}HP", hp);
    draw_text(
        pixmap,
        cx - 120.0 * scale,
        by + 28.0 * scale,
        &hp_str,
        Color::from_rgba8(212, 255, 0, 255),
        scale * 0.7,
    );
    // Slip indicator bar
    let slip = sanitize(data.car_slip_angle_deg().abs(), 0.0, 45.0);
    let slip_w = bar_w * (slip / 45.0);
    fill_rect_xywh(
        pixmap,
        bx,
        by + 36.0 * scale,
        slip_w,
        bar_h,
        Color::from_rgba8(245, 158, 11, 255),
    );
    // G-force mini display
    let g_lat = sanitize(data.lateral_g(), 0.0, 3.0);
    draw_text(
        pixmap,
        cx + 90.0 * scale,
        cy + 20.0 * scale,
        &format!("G {:.1}", g_lat),
        Color::from_rgba8(255, 255, 255, 255),
        scale * 0.7,
    );
    // Redline strobe overlay
    if rpm_ratio >= 0.96 {
        fill_rect_xywh(
            pixmap,
            gx,
            gy,
            gear_box_w,
            gear_box_h,
            Color::from_rgba8(255, 23, 68, 30),
        );
    }
}
