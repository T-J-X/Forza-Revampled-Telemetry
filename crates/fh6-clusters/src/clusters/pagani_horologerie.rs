use tiny_skia::{Color, PixmapMut, Rect};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_glitch_text};
use crate::draw_ext::*;

fn sanitize(v: f32, min_v: f32, max_v: f32) -> f32 { if !v.is_finite() { min_v } else { v.clamp(min_v, max_v) } }

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let cx = bounds.x() + bounds.width()*0.5;
    let cy = bounds.y() + bounds.height()*0.5;
    let scale = 1.0;

    fill_rect_xywh(pixmap, bounds.x(), bounds.y(), bounds.width(), bounds.height(), Color::from_rgba8(10,14,22,255));

    let norm_rpm = sanitize(data.normalized_rpm(),0.0,1.2);
    let in_redline = norm_rpm >= 0.94;

    // Milled case
    let case_w = 620.0*scale;
    let case_h = 256.0*scale;
    fill_round_rect(pixmap, cx - case_w*0.5, cy - case_h*0.5, case_w, case_h, 118.0*scale, Color::from_rgba8(42,46,56,252));
    stroke_round_rect(pixmap, cx - case_w*0.5, cy - case_h*0.5, case_w, case_h, 118.0*scale, 3.0*scale, Color::from_rgba8(245,198,112,230));

    // Guilloche engraving
    for gx in (0..13).map(|i| cx - 310.0*scale + 40.0*scale + i as f32 *46.0*scale) {
        for r in [18.0,36.0,54.0,72.0].iter() {
            stroke_circle(pixmap, gx, cy, *r*scale, 1.1*scale, Color::from_rgba8(245,206,128,24));
        }
    }

    // Skeleton gears
    let now = data.timestamp_ms as f32;
    let a1 = (data.current_engine_rpm * 0.045 + now*0.04).to_radians();
    let a2 = -(a1 * 1.35);
    let a3 = (data.speed_ms *4.2 + now*0.03).to_radians();
    draw_gear_wheel(pixmap, cx -156.0*scale, cy +4.0*scale, 82.0*scale, 20.0*scale, 20, a1, Color::from_rgba8(204,152,74,145), Color::from_rgba8(225,29,72,255));
    draw_gear_wheel(pixmap, cx, cy -34.0*scale, 54.0*scale, 14.0*scale, 14, a2, Color::from_rgba8(212,222,236,165), Color::from_rgba8(225,29,72,255));
    draw_gear_wheel(pixmap, cx +156.0*scale, cy +4.0*scale, 82.0*scale, 20.0*scale, 20, a3, Color::from_rgba8(245,198,112,145), Color::from_rgba8(225,29,72,255));

    // Bridge plate
    let bridge_w = 568.0*scale;
    let bridge_h = 28.0*scale;
    fill_round_rect(pixmap, cx - bridge_w*0.5, cy - bridge_h*0.5, bridge_w, bridge_h, 4.0*scale, Color::from_rgba8(165,176,194,235));
    stroke_round_rect(pixmap, cx - bridge_w*0.5, cy - bridge_h*0.5, bridge_w, bridge_h, 4.0*scale, 1.2*scale, Color::from_rgba8(245,198,112,255));

    // 8 blued steel screws
    let screw_xs = [-270.0, -235.0, -72.0, -54.0, 54.0, 72.0, 235.0, 270.0];
    for (i, &sx) in screw_xs.iter().enumerate() {
        let sy = if i%2==0 { -5.0 } else { 5.0 };
        fill_round_rect(pixmap, cx + sx*scale -4.2*scale, cy + sy*scale -4.2*scale, 8.4*scale, 8.4*scale, 4.2*scale, Color::from_rgba8(56,189,248,255));
    }

    // Ruby bearings
    for &(rx, ry) in &[(-88.0,-42.0),(88.0,-42.0),(-88.0,44.0),(88.0,44.0)] {
        draw_ruby_jewel_bearing(pixmap, cx + rx*scale, cy + ry*scale, 5.5*scale);
    }

    // Sub dials
    let dial_r = 96.0*scale;
    let speed_frac = sanitize(data.speed_kmh()/240.0,0.0,1.0);
    let left_cx = cx -156.0*scale;
    let right_cx = cx +156.0*scale;
    // left dial
    fill_round_rect(pixmap, left_cx - dial_r, cy - dial_r +2.0*scale, dial_r*2.0, dial_r*2.0, dial_r, Color::from_rgba8(22,26,34,238));
    stroke_round_rect(pixmap, left_cx - dial_r, cy - dial_r +2.0*scale, dial_r*2.0, dial_r*2.0, dial_r, 2.4*scale, Color::from_rgba8(245,198,112,255));
    let hand_rad = (135.0 + speed_frac*270.0).to_radians();
    stroke_line(pixmap, left_cx, cy+2.0*scale, left_cx + (dial_r -24.0*scale)*hand_rad.cos(), cy+2.0*scale + (dial_r -24.0*scale)*hand_rad.sin(), 3.2*scale, Color::from_rgba8(56,189,248,255));
    draw_text(pixmap, left_cx, cy + dial_r + 8.0*scale, &format!("{:.0}", data.speed_mph()), Color::from_rgba8(255,248,228,255), scale*1.5);
    // right dial rpm
    fill_round_rect(pixmap, right_cx - dial_r, cy - dial_r +2.0*scale, dial_r*2.0, dial_r*2.0, dial_r, Color::from_rgba8(22,26,34,238));
    stroke_round_rect(pixmap, right_cx - dial_r, cy - dial_r +2.0*scale, dial_r*2.0, dial_r*2.0, dial_r, 2.4*scale, Color::from_rgba8(245,198,112,255));
    let rpm_rad = (135.0 + norm_rpm*270.0).to_radians();
    let rpm_col = if in_redline { Color::from_rgba8(225, 29, 72, 255) } else { Color::from_rgba8(56, 189, 248, 255) };
    stroke_line(pixmap, right_cx, cy+2.0*scale, right_cx + (dial_r -24.0*scale)*rpm_rad.cos(), cy+2.0*scale + (dial_r -24.0*scale)*rpm_rad.sin(), 3.2*scale, rpm_col);
    draw_text(pixmap, right_cx, cy + dial_r + 8.0*scale, &format!("{:.1}", data.current_engine_rpm/1000.0), Color::from_rgba8(255,248,228,255), scale*1.5);

    // Central shifter tower
    fill_round_rect(pixmap, cx -56.0*scale, cy -64.0*scale, 112.0*scale,128.0*scale,16.0*scale, Color::from_rgba8(36,40,48,248));
    stroke_round_rect(pixmap, cx -56.0*scale, cy -64.0*scale,112.0*scale,128.0*scale,16.0*scale,2.2*scale, Color::from_rgba8(245,198,112,255));
    // gear display
    let gear_col = if in_redline { Color::from_rgba8(225, 29, 72, 255) } else { Color::from_rgba8(255, 248, 228, 255) };
    draw_text(pixmap, cx, cy - 22.0*scale, &data.gear_string(), gear_col, scale*2.0);
    draw_text(pixmap, cx, cy + 48.0*scale, &format!("{:+.0} PSI PAGANI", data.boost_psi), Color::from_rgba8(56,189,248,255), scale*0.7);

    // Driver tag
    let tag = "ØG『死』ŦJ乂";
    let time_ms = data.timestamp_ms as u64;
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx, cy + 120.0*scale, tag, Color::from_rgba8(245,198,112,255), scale*0.7, time_ms, true);
    } else {
        draw_text(pixmap, cx, cy + 120.0*scale, tag, Color::from_rgba8(245,198,112,255), scale*0.7);
    }
}
