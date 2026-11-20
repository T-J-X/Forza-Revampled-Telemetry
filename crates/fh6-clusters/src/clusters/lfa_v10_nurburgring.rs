use tiny_skia::{Color, PixmapMut, Rect};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_glitch_text};
use crate::draw_ext::*;

fn sanitize(v: f32, min_v: f32, max_v: f32) -> f32 { if !v.is_finite() { min_v } else { v.clamp(min_v, max_v) } }

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let cx = bounds.x() + bounds.width()*0.5;
    let cy = bounds.y() + bounds.height()*0.5;
    let scale = 1.0;

    fill_rect_xywh(pixmap, bounds.x(), bounds.y(), bounds.width(), bounds.height(), Color::from_rgba8(5,8,13,255));

    let norm_rpm = sanitize(data.normalized_rpm(),0.0,1.2);
    let in_redline = norm_rpm >= 0.94;
    let mood_col = if in_redline { Color::from_rgba8(255, 23, 68, 255) } else { Color::from_rgba8(245, 158, 11, 255) };
    let mood_arc_col = if in_redline { Color::from_rgba8(255, 23, 68, 185) } else { Color::from_rgba8(245, 158, 11, 185) };

    // Carbon fiber wings housing
    let total_w = 652.0*scale;
    let wing_h = 216.0*scale;
    fill_round_rect(pixmap, cx - total_w*0.5, cy - wing_h*0.5 + 2.0*scale, total_w, wing_h, 22.0*scale, Color::from_rgba8(15,19,27,250));
    stroke_round_rect(pixmap, cx - total_w*0.5, cy - wing_h*0.5 + 2.0*scale, total_w, wing_h, 22.0*scale, 2.0*scale, Color::from_rgba8(92,108,132,190));

    // Left wing telemetry
    let lwx = cx - 232.0*scale;
    draw_text(pixmap, lwx, cy - 82.0*scale, "LEXUS LFA  V10", Color::from_rgba8(96,165,250,255), scale*0.8);
    draw_text(pixmap, lwx, cy - 58.0*scale, "OIL  98C", Color::from_rgba8(215,230,248,255), scale*0.72);
    draw_text(pixmap, lwx, cy - 18.0*scale, &format!("ITB PLENUM  {:+.0} PSI", data.boost_psi), Color::from_rgba8(255,255,255,255), scale*0.76);
    draw_text(pixmap, lwx, cy + 26.0*scale, "NURBURGRING LAP", Color::from_rgba8(96,165,250,255), scale*0.72);
    draw_text(pixmap, lwx, cy + 78.0*scale, "SPORT // N-MODE", mood_col, scale*0.74);

    // Right wing telemetry
    let rwx = cx + 232.0*scale;
    draw_text(pixmap, rwx, cy - 82.0*scale, "TIRE & CHASSIS", Color::from_rgba8(96,165,250,255), scale*0.8);
    draw_text(pixmap, rwx, cy - 68.0*scale, &format!("{:+.02}G", data.lateral_g()), Color::from_rgba8(96,165,250,255), scale*1.2);
    draw_text(pixmap, rwx, cy + 6.0*scale, "FL 94C   FR 96C", Color::from_rgba8(255,255,255,255), scale*0.72);
    draw_text(pixmap, rwx, cy + 40.0*scale, "RL 91C   RR 93C", Color::from_rgba8(255,255,255,255), scale*0.72);

    // Motorized titanium ring
    let dial_r = 136.0*scale;
    // Motorized sliding offset (+68px in high RPM)
    let offset = if norm_rpm > 0.85 { 68.0*scale } else { 0.0 };
    let ring_cx = cx + offset;
    fill_round_rect(pixmap, ring_cx - dial_r, cy - dial_r, dial_r*2.0, dial_r*2.0, dial_r, Color::from_rgba8(5,8,13,252));
    stroke_round_rect(pixmap, ring_cx - dial_r, cy - dial_r, dial_r*2.0, dial_r*2.0, dial_r, 8.5*scale, Color::from_rgba8(215,226,240,250));

    // Redline notch
    fill_rect_xywh(pixmap, ring_cx -10.0*scale, cy - dial_r -3.0*scale, 20.0*scale, 9.0*scale, mood_col);

    // Laser-etched ticks 0..10k
    for q in 0..=50 {
        let frac = q as f32 / 50.0;
        let ang = (135.0 + frac*270.0).to_radians();
        let is_major = q % 5 == 0;
        let in_r = dial_r - (if is_major {23.0} else {16.0})*scale;
        let out_r = dial_r - 9.0*scale;
        let col = if frac >= 0.90 { Color::from_rgba8(255,23,68,255) } else if is_major { Color::from_rgba8(255,255,255,255) } else { Color::from_rgba8(145,200,218,238) };
        stroke_line(pixmap, ring_cx + in_r*ang.cos(), cy + in_r*ang.sin(), ring_cx + out_r*ang.cos(), cy + out_r*ang.sin(), if is_major {2.4} else {1.0}*scale, col);
        if is_major {
            draw_text(pixmap, ring_cx + (dial_r -36.0*scale)*ang.cos(), cy + (dial_r -36.0*scale)*ang.sin(), &(q/5).to_string(), col, scale*0.7);
        }
    }

    // Active arc wake
    stroke_arc_deg(pixmap, ring_cx, cy, dial_r -18.0*scale, dial_r -18.0*scale, 135.0, 270.0 * norm_rpm.min(1.0), 5.5*scale, mood_arc_col);

    // Sweeping needle
    let needle_rad = (135.0 + norm_rpm.min(1.02)*270.0).to_radians();
    stroke_line(pixmap, ring_cx + 20.0*scale*needle_rad.cos(), cy + 20.0*scale*needle_rad.sin(), ring_cx + (dial_r -10.0*scale)*needle_rad.cos(), cy + (dial_r -10.0*scale)*needle_rad.sin(), 3.6*scale, Color::from_rgba8(255,23,68,255));

    // Hub cap
    let hub_r = 58.0*scale;
    fill_round_rect(pixmap, ring_cx - hub_r, cy - hub_r, hub_r*2.0, hub_r*2.0, hub_r, Color::from_rgba8(10,14,22,250));
    stroke_round_rect(pixmap, ring_cx - hub_r, cy - hub_r, hub_r*2.0, hub_r*2.0, hub_r, 2.4*scale, Color::from_rgba8(185,200,220,240));

    // Center speed and gear
    draw_text(pixmap, ring_cx, cy - 22.0*scale, &format!("{:.0}", data.speed_mph()), Color::from_rgba8(255,255,255,255), scale*2.2);
    draw_text(pixmap, ring_cx, cy + 2.0*scale, "MPH", Color::from_rgba8(96,165,250,255), scale*0.74);
    draw_text(pixmap, ring_cx, cy + 32.0*scale, &data.gear_string(), mood_col, scale*1.9);

    // Crimson redline bloom at >=8500 RPM
    if norm_rpm >= 0.85 {
        let bloom_r = 180.0*scale + (norm_rpm - 0.85)*120.0*scale;
        fill_round_rect(pixmap, ring_cx - bloom_r, cy - bloom_r, bloom_r*2.0, bloom_r*2.0, bloom_r, Color::from_rgba8(255,23,68,30));
    }

    // Driver tag
    let tag = "ØG『死』ŦJ乂";
    let time_ms = data.timestamp_ms as u64;
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx, cy + 120.0*scale, tag, Color::from_rgba8(96,165,250,255), scale*0.7, time_ms, true);
    } else {
        draw_text(pixmap, cx, cy + 120.0*scale, tag, Color::from_rgba8(96,165,250,255), scale*0.7);
    }
}
