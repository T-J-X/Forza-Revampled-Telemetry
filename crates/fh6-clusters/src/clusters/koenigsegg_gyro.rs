use tiny_skia::{Color, PixmapMut, Rect};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_glitch_text};
use crate::draw_ext::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn sanitize(v: f32, min_v: f32, max_v: f32) -> f32 { if !v.is_finite() { min_v } else { v.clamp(min_v, max_v) } }

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let cx = bounds.x() + bounds.width()*0.5;
    let cy = bounds.y() + bounds.height()*0.5;
    let scale = 1.0;

    fill_rect_xywh(pixmap, bounds.x(), bounds.y(), bounds.width(), bounds.height(), Color::from_rgba8(7,9,13,255));

    let norm_rpm = sanitize(data.normalized_rpm(),0.0,1.2);
    let in_redline = norm_rpm >= 0.94;

    // Forged carbon outer frame
    let outer_w = 624.0*scale;
    let outer_h = 284.0*scale;
    fill_round_rect(pixmap, cx - outer_w*0.5, cy - outer_h*0.5, outer_w, outer_h, 48.0*scale, Color::from_rgba8(22,26,34,250));
    stroke_round_rect(pixmap, cx - outer_w*0.5, cy - outer_h*0.5, outer_w, outer_h, 48.0*scale, 2.4*scale, Color::from_rgba8(108,124,148,215));

    // Carbon facets
    for i in -34..=34 {
        let x = cx + i as f32 * 18.0*scale;
        stroke_line(pixmap, x, cy - outer_h*0.5, x -95.0*scale, cy + outer_h*0.5, 1.8*scale, Color::from_rgba8(175,198,226,24));
    }

    // Left wing card
    let l_w = 136.0*scale;
    let l_h = 208.0*scale;
    let l_x = cx - 294.0*scale;
    fill_round_rect(pixmap, l_x, cy - l_h*0.5, l_w, l_h, 12.0*scale, Color::from_rgba8(10,14,20,205));
    stroke_round_rect(pixmap, l_x, cy - l_h*0.5, l_w, l_h, 12.0*scale, 1.3*scale, Color::from_rgba8(250,204,21,145));
    draw_text(pixmap, cx - 158.0*scale, cy - 70.0*scale, "KOENIGSEGG E85", Color::from_rgba8(250,204,21,255), scale*0.72);
    draw_text(pixmap, cx - 158.0*scale, cy - 44.0*scale, "1600 HP", Color::from_rgba8(255,255,255,255), scale*1.2);
    draw_text(pixmap, cx - 158.0*scale, cy - 6.0*scale, "AERO DOWNFORCE", Color::from_rgba8(0,229,255,255), scale*0.68);
    draw_text(pixmap, cx - 158.0*scale, cy + 16.0*scale, "385 KG", Color::from_rgba8(16,185,129,255), scale*1.1);

    // Right wing card
    let r_x = cx + 158.0*scale;
    fill_round_rect(pixmap, r_x, cy - l_h*0.5, l_w, l_h, 12.0*scale, Color::from_rgba8(10,14,20,205));
    stroke_round_rect(pixmap, r_x, cy - l_h*0.5, l_w, l_h, 12.0*scale, 1.3*scale, Color::from_rgba8(0,229,255,145));
    // G-force target
    let g_cx = r_x + 68.0*scale;
    let g_cy = cy - 58.0*scale;
    let g_r = 30.0*scale;
    stroke_circle(pixmap, g_cx, g_cy, g_r, 1.2*scale, Color::from_rgba8(0,229,255,120));
    let dot_x = g_cx + sanitize(data.lateral_g()/2.0, -1.0,1.0)*g_r*0.8;
    let dot_y = g_cy - sanitize(data.longitudinal_g()/2.0, -1.0,1.0)*g_r*0.8;
    fill_round_rect(pixmap, dot_x -4.0*scale, dot_y -4.0*scale, 8.0*scale, 8.0*scale, 4.0*scale, Color::from_rgba8(250,204,21,255));
    draw_text(pixmap, g_cx, g_cy + 42.0*scale, &format!("PEAK G {:.2}G", data.lateral_g().abs()), Color::from_rgba8(250,204,21,255), scale*0.74);

    // Gyroscopic gimbal
    let gimbal_r = 136.0*scale;
    stroke_circle(pixmap, cx, cy, gimbal_r, 2.2*scale, Color::from_rgba8(16,185,129,165));
    stroke_circle(pixmap, cx, cy, gimbal_r -5.0*scale, 1.1*scale, Color::from_rgba8(0,229,255,85));

    // Orbital ticks
    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as f32;
    let orb_ang = (now_ms * 0.016) % 360.0;
    for deg in (-60..=60).step_by(15) {
        let a = (deg as f32 -90.0 + orb_ang*0.12).to_radians();
        stroke_line(pixmap, cx + (gimbal_r -5.0*scale)*a.cos(), cy + (gimbal_r -5.0*scale)*a.sin(),
                    cx + (gimbal_r +4.0*scale)*a.cos(), cy + (gimbal_r +4.0*scale)*a.sin(),
                    2.0*scale, if deg==0 { Color::from_rgba8(250,204,21,255) } else { Color::from_rgba8(16,185,129,255) });
    }

    // Gyro tilt
    let gyro_tilt = sanitize(-data.lateral_g()*13.5, -22.0,22.0);
    // Simplified rotation via offset for visual effect (full rotation would need transform)
    let tilt_offset = gyro_tilt * 2.0;

    // Central drum
    let drum_r = 128.0*scale;
    let theme_col = if in_redline { Color::from_rgba8(255, 23, 68, 255) } else { Color::from_rgba8(16, 185, 129, 255) };
    let gear_col = if in_redline { Color::from_rgba8(255, 23, 68, 255) } else { Color::from_rgba8(255, 255, 255, 255) };
    fill_round_rect(pixmap, cx - drum_r + tilt_offset, cy - drum_r + tilt_offset, drum_r*2.0, drum_r*2.0, drum_r, Color::from_rgba8(5,8,12,252));
    stroke_round_rect(pixmap, cx - drum_r + tilt_offset, cy - drum_r + tilt_offset, drum_r*2.0, drum_r*2.0, drum_r, 2.6*scale, theme_col);

    // RPM arc
    stroke_arc_deg(pixmap, cx + tilt_offset, cy + tilt_offset, drum_r -11.0*scale, drum_r -11.0*scale, 145.0, 250.0 * norm_rpm.min(1.0), 8.5*scale, theme_col);

    // Boost arc
    let boost_frac = sanitize((data.boost_psi +4.0)/28.0,0.05,1.0);
    stroke_arc_deg(pixmap, cx + tilt_offset, cy + tilt_offset, drum_r -23.0*scale, drum_r -23.0*scale, 155.0, 230.0*boost_frac, 5.0*scale, Color::from_rgba8(250,204,21,255));

    // Speed
    draw_text(pixmap, cx + tilt_offset, cy - 40.0*scale + tilt_offset, &format!("{:.0}", data.speed_mph()), Color::from_rgba8(255,255,255,255), scale*2.5);
    draw_text(pixmap, cx + tilt_offset, cy - 8.0*scale + tilt_offset, "MPH", Color::from_rgba8(250,204,21,255), scale*0.82);

    // Gear badge
    let gear_box_w = 80.0*scale;
    let gear_box_h = 48.0*scale;
    fill_round_rect(pixmap, cx - gear_box_w*0.5 + tilt_offset, cy + 12.0*scale + tilt_offset, gear_box_w, gear_box_h, 8.0*scale, Color::from_rgba8(10,16,24,180));
    stroke_round_rect(pixmap, cx - gear_box_w*0.5 + tilt_offset, cy +12.0*scale + tilt_offset, gear_box_w, gear_box_h, 8.0*scale, 1.4*scale, theme_col);
    draw_text(pixmap, cx + tilt_offset, cy + 36.0*scale + tilt_offset, &data.gear_string(), gear_col, scale*2.1);

    draw_text(pixmap, cx + tilt_offset, cy + 72.0*scale + tilt_offset, &format!("GYRO {:+.0}°", gyro_tilt), Color::from_rgba8(0,229,255,255), scale*0.74);

    // Driver tag
    let tag = "ØG『死』ŦJ乂";
    let time_ms = data.timestamp_ms as u64;
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx, cy + 120.0*scale, tag, Color::from_rgba8(16,185,129,255), scale*0.7, time_ms, true);
    } else {
        draw_text(pixmap, cx, cy + 120.0*scale, tag, Color::from_rgba8(16,185,129,255), scale*0.7);
    }

    // Live 2D G-force crosshair ball (simplified)
    let ball_r = 36.0*scale;
    fill_round_rect(pixmap, cx - ball_r, cy + 100.0*scale - ball_r, ball_r*2.0, ball_r*2.0, ball_r, Color::from_rgba8(10,16,24,200));
    stroke_circle(pixmap, cx, cy + 100.0*scale, ball_r, 1.0*scale, Color::from_rgba8(0,229,255,255));
    let bx = cx + sanitize(data.lateral_g(), -1.0,1.0)*ball_r*0.7;
    let by = cy + 100.0*scale - sanitize(data.longitudinal_g(), -1.0,1.0)*ball_r*0.7;
    fill_round_rect(pixmap, bx -3.0*scale, by -3.0*scale, 6.0*scale, 6.0*scale, 3.0*scale, Color::from_rgba8(250,204,21,255));
}
