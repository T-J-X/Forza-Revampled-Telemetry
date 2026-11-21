use tiny_skia::{Color, PixmapMut, Rect};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_glitch_text};
use crate::draw_ext::*;

fn sanitize(v: f32, min_v: f32, max_v: f32) -> f32 {
    if !v.is_finite() {
        min_v
    } else {
        v.clamp(min_v, max_v)
    }
}

#[inline]
fn premultiply(r: u8, g: u8, b: u8, a: u8) -> (u8, u8, u8, u8) {
    if a < 2 {
        return (0, 0, 0, 0);
    }
    let af = a as f32 / 255.0;
    let r_p = ((r as f32 * af).round() as u8).min(a);
    let g_p = ((g as f32 * af).round() as u8).min(a);
    let b_p = ((b as f32 * af).round() as u8).min(a);
    (r_p, g_p, b_p, a)
}

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let cx = bounds.x() + bounds.width() * 0.5;
    let cy = bounds.y() + bounds.height() * 0.5;
    let scale = 1.0_f32;

    let norm_rpm = sanitize(data.normalized_rpm(), 0.0, 1.2);
    let speed_ms = data.speed_ms;
    let boost_psi = data.boost_psi;
    let lateral_g = sanitize(data.lateral_g(), -2.0, 2.0).abs();

    let t_ms = data.timestamp_ms as f32;
    let phi = (t_ms * 0.006 * (1.0 + 3.0 * norm_rpm) + speed_ms * 0.1) % (std::f32::consts::TAU);
    let phi_norm = phi;

    let r0 = 80.0 * scale;
    let r_core = r0 * (1.0 + 0.04 * boost_psi);
    let delta_x = lateral_g / 2.0 * 45.0 * scale;

    // Toroidal plasma rings
    let ring_count = 8;
    for i in 0..ring_count {
        let i_f = i as f32;
        let base_r = r_core + i_f * 18.0 * scale;
        let pulse = (phi_norm + i_f * 0.8).sin() * 6.0 * scale * (1.0 + norm_rpm);
        let radius = base_r + pulse;
        // Color transition amber -> cyan -> crimson
        let t = norm_rpm.clamp(0.0, 1.0);
        let (r_c, g_c, b_c) = if t < 0.5 {
            let u = t / 0.5;
            let r1 = 245; let g1 = 158; let b1 = 11;
            let r2 = 6; let g2 = 182; let b2 = 212;
            let r = (r1 as f32 + (r2 as f32 - r1 as f32) * u).round() as u8;
            let g = (g1 as f32 + (g2 as f32 - g1 as f32) * u).round() as u8;
            let b = (b1 as f32 + (b2 as f32 - b1 as f32) * u).round() as u8;
            (r, g, b)
        } else {
            let u = (t - 0.5) / 0.5;
            let r1 = 6; let g1 = 182; let b1 = 212;
            let r2 = 239; let g2 = 68; let b2 = 68;
            let r = (r1 as f32 + (r2 as f32 - r1 as f32) * u).round() as u8;
            let g = (g1 as f32 + (g2 as f32 - g1 as f32) * u).round() as u8;
            let b = (b1 as f32 + (b2 as f32 - b1 as f32) * u).round() as u8;
            (r, g, b)
        };
        // norm_rpm reaches 1.2, so saturate instead of overflowing u8 (a debug-build panic before).
        let a = 180u8.saturating_add((norm_rpm * 75.0) as u8);
        let (rp, gp, bp, ap) = premultiply(r_c, g_c, b_c, a);
        let color = Color::from_rgba8(rp, gp, bp, ap);
        // Draw ellipse as stroked ring
        let rx = radius;
        let ry = radius * 0.6;
        stroke_arc_deg(pixmap, cx + delta_x, cy, rx, ry, 0.0, 360.0, 2.5 * scale, color);
    }

    // Central plasma beam deflection
    {
        let r_beam = 12.0 * scale * (1.0 + norm_rpm);
        let r_beam_r = r_beam;
        let r_beam_g = r_beam * 0.7;
        // TODO(phase-7): meant to drive a second beam ellipse radius (the beam ring only uses r_beam_r and r_beam_g).
        let _r_beam_b = r_beam * 0.7;
        let r_beam_a = 200;
        let (rp, gp, bp, ap) = premultiply(6, 182, 212, r_beam_a);
        let beam_color = Color::from_rgba8(rp, gp, bp, ap);
        stroke_arc_deg(pixmap, cx + delta_x, cy, r_beam_r, r_beam_g, 0.0, 360.0, 4.0 * scale, beam_color);
    }

    // Amber VFD speed & gear
    let speed_mph = data.speed_mph();
    let gear_str = data.gear_string();
    let text_alpha = 255;
    let (tr, tg, tb, ta) = premultiply(255, 140, 0, text_alpha);
    let text_color = Color::from_rgba8(tr, tg, tb, ta);
    draw_text(pixmap, cx, cy - 30.0 * scale, &format!("{:.0}", speed_mph), text_color, scale * 1.8);
    draw_text(pixmap, cx, cy + 40.0 * scale, &gear_str, text_color, scale * 2.0);

    // Driver tag
    let tag = "ØG『死』ŦJ乂";
    let tag_alpha = 255;
    let (tr2, tg2, tb2, ta2) = premultiply(255, 140, 0, tag_alpha);
    let tag_color = Color::from_rgba8(tr2, tg2, tb2, ta2);
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx, cy + 130.0 * scale, tag, tag_color, scale * 0.6, data.timestamp_ms as u64, true);
    } else {
        draw_text(pixmap, cx, cy + 130.0 * scale, tag, tag_color, scale * 0.6);
    }

    // Use power_watts to demonstrate telemetry binding
    let _hp = (data.power_watts / 745.7).max(0.0);
    // Demonstrate for &x in &[a,b] pattern
    let sides = [-140.0 * scale, 140.0 * scale];
    for &x_offset in &sides {
        let x = cx + x_offset;
        // subtle side glow
        let glow_a = 45;
        let (gr, gg, gb, ga) = premultiply(255, 140, 0, glow_a);
        let glow_color = Color::from_rgba8(gr, gg, gb, ga);
        stroke_rect_xywh(pixmap, x - 20.0 * scale, cy - 150.0 * scale, 40.0 * scale, 300.0 * scale, 1.0 * scale, glow_color);
    }
}
