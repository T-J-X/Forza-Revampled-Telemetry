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

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let cx = bounds.x() + bounds.width() * 0.5;
    let cy = bounds.y() + bounds.height() * 0.5;
    let scale = 1.0;

    // Background
    fill_rect_xywh(
        pixmap,
        bounds.x(),
        bounds.y(),
        bounds.width(),
        bounds.height(),
        Color::from_rgba8(5, 8, 13, 255),
    );

    // Sapphire bridge housing
    let body_w = 620.0 * scale;
    let body_h = 204.0 * scale;
    fill_round_rect(
        pixmap,
        cx - body_w * 0.5,
        cy - body_h * 0.5,
        body_w,
        body_h,
        98.0 * scale,
        Color::from_rgba8(52, 60, 76, 250),
    );
    stroke_round_rect(
        pixmap,
        cx - body_w * 0.5,
        cy - body_h * 0.5,
        body_w,
        body_h,
        98.0 * scale,
        2.4 * scale,
        Color::from_rgba8(216, 224, 238, 255),
    );

    // Cotes-de-Geneve striping
    for i in 0..70 {
        let x = cx - body_w * 0.5 + i as f32 * 9.0 * scale;
        stroke_line(
            pixmap,
            x,
            cy - body_h * 0.5,
            x,
            cy + body_h * 0.5,
            3.0 * scale,
            Color::from_rgba8(235, 245, 255, 22),
        );
    }

    // M3 socket bolts
    for &bx in &[-278.0, -278.0, 278.0, 278.0] {
        let by = if bx < 0.0 { -48.0 } else { 48.0 };
        fill_round_rect(
            pixmap,
            cx + bx * scale - 4.2 * scale,
            cy + by * scale - 4.2 * scale,
            8.4 * scale,
            8.4 * scale,
            2.0 * scale,
            Color::from_rgba8(216, 224, 238, 255),
        );
    }

    let norm_rpm = sanitize(data.normalized_rpm(), 0.0, 1.2);
    let hp = (data.power_watts / 745.7).max(0.0);
    let hp_frac = sanitize(hp / 1800.0, 0.15, 1.0);
    let speed_kmh = data.speed_kmh();
    let speed_frac = sanitize(speed_kmh / 550.0, 0.0, 1.0);

    // Left sapphire planetary dial - HP
    let sub_r = 86.0 * scale;
    let left_cx = cx - 188.0 * scale;
    fill_round_rect(
        pixmap,
        left_cx - sub_r,
        cy - sub_r,
        sub_r * 2.0,
        sub_r * 2.0,
        sub_r,
        Color::from_rgba8(8, 13, 24, 245),
    );
    stroke_round_rect(
        pixmap,
        left_cx - sub_r,
        cy - sub_r,
        sub_r * 2.0,
        sub_r * 2.0,
        sub_r,
        2.6 * scale,
        Color::from_rgba8(216, 224, 238, 255),
    );
    stroke_arc_deg(
        pixmap,
        left_cx,
        cy,
        sub_r - 9.0 * scale,
        sub_r - 9.0 * scale,
        135.0,
        270.0 * hp_frac,
        5.5 * scale,
        Color::from_rgba8(59, 130, 246, 255),
    );
    draw_text(
        pixmap,
        left_cx,
        cy - 36.0 * scale,
        "POWER  HP",
        Color::from_rgba8(59, 130, 246, 255),
        scale * 0.74,
    );
    draw_text(
        pixmap,
        left_cx,
        cy - 8.0 * scale,
        &format!("{:.0}", hp.max(180.0)),
        Color::from_rgba8(245, 232, 204, 255),
        scale * 1.8,
    );

    // Right sapphire planetary dial - Boost
    let right_cx = cx + 188.0 * scale;
    fill_round_rect(
        pixmap,
        right_cx - sub_r,
        cy - sub_r,
        sub_r * 2.0,
        sub_r * 2.0,
        sub_r,
        Color::from_rgba8(8, 13, 24, 245),
    );
    stroke_round_rect(
        pixmap,
        right_cx - sub_r,
        cy - sub_r,
        sub_r * 2.0,
        sub_r * 2.0,
        sub_r,
        2.6 * scale,
        Color::from_rgba8(216, 224, 238, 255),
    );
    let boost_frac = sanitize((data.boost_psi + 5.0) / 30.0, 0.0, 1.0);
    stroke_arc_deg(
        pixmap,
        right_cx,
        cy,
        sub_r - 9.0 * scale,
        sub_r - 9.0 * scale,
        135.0,
        270.0 * boost_frac,
        5.5 * scale,
        Color::from_rgba8(59, 130, 246, 255),
    );
    draw_text(
        pixmap,
        right_cx,
        cy - 36.0 * scale,
        "BOOST  PSI",
        Color::from_rgba8(59, 130, 246, 255),
        scale * 0.74,
    );
    draw_text(
        pixmap,
        right_cx,
        cy - 8.0 * scale,
        &format!("{:+.0}", data.boost_psi),
        Color::from_rgba8(245, 232, 204, 255),
        scale * 1.8,
    );

    // Central coaxial dial
    let main_r = 122.0 * scale;
    fill_round_rect(
        pixmap,
        cx - main_r,
        cy - main_r,
        main_r * 2.0,
        main_r * 2.0,
        main_r,
        Color::from_rgba8(16, 22, 34, 252),
    );
    stroke_round_rect(
        pixmap,
        cx - main_r,
        cy - main_r,
        main_r * 2.0,
        main_r * 2.0,
        main_r,
        5.2 * scale,
        Color::from_rgba8(216, 224, 238, 255),
    );

    // Outer speed scale ticks 0..9 (represent 0..550 km/h)
    for k in 0..=9 {
        let tf = k as f32 / 9.0;
        let ang = (135.0 + tf * 270.0).to_radians();
        let r_outer = main_r - 20.0 * scale;
        let r_inner = main_r - 9.0 * scale;
        stroke_line(
            pixmap,
            cx + r_inner * ang.cos(),
            cy + r_inner * ang.sin(),
            cx + r_outer * ang.cos(),
            cy + r_outer * ang.sin(),
            2.2 * scale,
            Color::from_rgba8(245, 232, 204, 255),
        );
        draw_text(
            pixmap,
            cx + (main_r - 33.0 * scale) * ang.cos(),
            cy + (main_r - 33.0 * scale) * ang.sin(),
            &k.to_string(),
            Color::from_rgba8(245, 232, 204, 255),
            scale * 0.7,
        );
    }

    // Coaxial hands
    let rpm_rad = (135.0 + norm_rpm * 270.0).to_radians();
    stroke_line(
        pixmap,
        cx,
        cy,
        cx + (main_r - 14.0 * scale) * rpm_rad.cos(),
        cy + (main_r - 14.0 * scale) * rpm_rad.sin(),
        3.6 * scale,
        Color::from_rgba8(59, 130, 246, 255),
    );
    let spd_rad = (45.0 - speed_frac * 270.0).to_radians();
    stroke_line(
        pixmap,
        cx,
        cy,
        cx + (main_r - 24.0 * scale) * spd_rad.cos(),
        cy + (main_r - 24.0 * scale) * spd_rad.sin(),
        2.4 * scale,
        Color::from_rgba8(245, 232, 204, 255),
    );

    // Tourbillon cage gear
    let escape_spin = data.current_engine_rpm * 0.04 + (data.timestamp_ms as f32 * 0.06);
    let gear_phase = escape_spin.to_radians();
    draw_gear_wheel(
        pixmap,
        cx,
        cy + 74.0 * scale,
        21.0 * scale,
        14.0 * scale,
        14,
        gear_phase,
        Color::from_rgba8(245, 232, 204, 220),
        Color::from_rgba8(59, 130, 246, 255),
    );

    // Center medallion
    let med_r = 48.0 * scale;
    fill_round_rect(
        pixmap,
        cx - med_r,
        cy - med_r,
        med_r * 2.0,
        med_r * 2.0,
        med_r,
        Color::from_rgba8(8, 14, 26, 245),
    );
    draw_text(
        pixmap,
        cx,
        cy - 12.0 * scale,
        &format!("{:.0}", data.speed_mph()),
        Color::from_rgba8(245, 232, 204, 255),
        scale * 1.6,
    );
    draw_text(
        pixmap,
        cx,
        cy + 20.0 * scale,
        &data.gear_string(),
        Color::from_rgba8(240, 248, 255, 255),
        scale * 1.1,
    );

    // Driver tag
    let tag = "ØG『死』ŦJ乂";
    let time_ms = data.timestamp_ms as u64;
    if config.driver_tag_glitch_fx {
        draw_glitch_text(
            pixmap,
            cx,
            cy + 120.0 * scale,
            tag,
            Color::from_rgba8(59, 130, 246, 255),
            scale * 0.7,
            time_ms,
            true,
        );
    } else {
        draw_text(
            pixmap,
            cx,
            cy + 120.0 * scale,
            tag,
            Color::from_rgba8(59, 130, 246, 255),
            scale * 0.7,
        );
    }

    // Ruby bearings
    for &bearing in &[(-88.0, -42.0), (88.0, -42.0), (-88.0, 44.0), (88.0, 44.0)] {
        draw_ruby_jewel_bearing(pixmap, cx + bearing.0 * scale, cy + bearing.1 * scale, 5.5 * scale);
    }
}
