use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform};
use fh6_core::{AppConfig, FH6TelemetryData, WorldMapTrailBuffer};
use fh6_render::text::{draw_text, draw_glitch_text};
use crate::draw_ext::{fill_rect_xywh, sanitize_finite, stroke_line};
use std::sync::OnceLock;

static TRAIL_BUFFER: OnceLock<WorldMapTrailBuffer> = OnceLock::new();

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let cx = bounds.x() + bounds.width() / 2.0;
    let cy = bounds.y() + bounds.height() / 2.0;
    let scale = 1.0;

    let mut bg = Paint::default();
    bg.set_color(Color::from_rgba8(6, 8, 12, 255));
    pixmap.fill_rect(bounds, &bg, Transform::identity(), None);

    let _trail = TRAIL_BUFFER.get_or_init(|| WorldMapTrailBuffer::new());

    let norm_rpm = sanitize_finite(data.normalized_rpm(), 0.0, 0.0, 1.2);
    let speed_ms = sanitize_finite(data.speed_ms, 0.0, 0.0, 250.0);
    let brake_pct = sanitize_finite(data.brake_pct(), 0.0, 0.0, 1.0);
    let _v_safe = speed_ms.max(0.5);
    let kappa = sanitize_finite(data.yaw_curvature(), 0.0, -0.055, 0.055);
    let beta = sanitize_finite(data.car_slip_angle_rad(), 0.0, -0.7854, 0.7854);

    // Collimated backdrop with RPM modulation
    let backdrop_alpha = (92.0 + norm_rpm * 60.0) as u8;
    fill_rect_xywh(pixmap, cx - 232.0 * scale, cy - 140.0 * scale, 464.0 * scale, 300.0 * scale, Color::from_rgba8(4, 7, 14, backdrop_alpha));

    // Driver tag
    let tag = "ØG『死』ŦJ乂";
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx - 130.0 * scale, cy - 240.0 * scale, tag, Color::from_rgba8(255, 255, 255, 255), scale, 0, true);
    } else {
        draw_text(pixmap, cx - 130.0 * scale, cy - 240.0 * scale, tag, Color::from_rgba8(255, 255, 255, 255), scale);
    }

    // Top chevron arc
    let arc_cx = cx;
    let arc_cy = cy - 82.0 * scale;
    let rx = 210.0 * scale;
    let ry = 46.0 * scale;
    for i in 0..20 {
        let ang = 202.0 + i as f32 * 6.8;
        let rad = ang.to_radians();
        let x = arc_cx + rx * rad.cos();
        let y = arc_cy + ry * rad.sin();
        fill_rect_xywh(pixmap, x - 2.0 * scale, y - 2.0 * scale, 4.0 * scale, 4.0 * scale, Color::from_rgba8(0, 229, 255, 180));
    }

    // Pitch ladders
    let core_cy = cy - 6.0 * scale;
    fill_rect_xywh(pixmap, cx - 198.0 * scale, core_cy - 36.0 * scale, 11.0 * scale, 76.0 * scale, Color::from_rgba8(3, 6, 12, 180));
    fill_rect_xywh(pixmap, cx + 187.0 * scale, core_cy - 36.0 * scale, 11.0 * scale, 76.0 * scale, Color::from_rgba8(3, 6, 12, 180));

    // Gear reticle
    if let Some(gear_box) = Rect::from_xywh(cx - 148.0 * scale, core_cy - 38.0 * scale, 78.0 * scale, 76.0 * scale) {
        let mut p = Paint::default();
        p.set_color(Color::from_rgba8(3, 6, 12, 110));
        pixmap.fill_rect(gear_box, &p, Transform::identity(), None);
    }

    // Speed 3-slot with brake modulation
    let display_speed = speed_ms * 3.6;
    let spd_int = (display_speed.round() as i32).clamp(0, 999);
    let spd_str = format!("{:03}", spd_int);
    let slot_alpha = (240.0 - brake_pct * 120.0) as u8;
    for (i, ch) in spd_str.chars().enumerate() {
        let x = cx - 42.0 * scale + i as f32 * 35.0 * scale;
        fill_rect_xywh(pixmap, x, core_cy - 30.0 * scale, 35.0 * scale, 60.0 * scale, Color::from_rgba8(244, 253, 255, slot_alpha));
        // Draw digit
        draw_text(pixmap, x + 5.0 * scale, core_cy + 8.0 * scale, &ch.to_string(), Color::from_rgba8(0,0,0,255), 0.6 * scale);
    }

    // Road map deck
    let map_w = 340.0 * scale;
    let map_h = 104.0 * scale;
    let map_top = cy + 14.0 * scale;
    fill_rect_xywh(pixmap, cx - map_w / 2.0, map_top, map_w, map_h, Color::from_rgba8(3, 6, 12, 68));

    // Perspective road
    let d0 = 16.0;
    let k_scale = 16.5 * scale;
    let horizon_y = map_top + 6.0 * scale;
    let player_screen_y = map_top + map_h - 18.0 * scale;
    let max_z = 65.0;
    let segs = 65;
    let road_half_w = 3.5;
    let mut prev_left: Option<(f32, f32)> = None;
    let mut prev_right: Option<(f32, f32)> = None;
    for i in 0..=segs {
        let z = i as f32 * max_z / segs as f32;
        let curve_x = 0.5 * kappa * z * z + beta.tan() * z * 0.12;
        let depth = (d0 / (d0 + z)).max(0.01);
        let sy = horizon_y + (player_screen_y - horizon_y) * depth;
        let left_x = cx + (curve_x - road_half_w) * k_scale * depth;
        let right_x = cx + (curve_x + road_half_w) * k_scale * depth;
        if let (Some(pl), Some(pr)) = (&prev_left, &prev_right) {
            stroke_line(pixmap, pl.0, pl.1, left_x, sy, 2.2 * scale, Color::from_rgba8(56, 189, 248, 180));
            stroke_line(pixmap, pr.0, pr.1, right_x, sy, 2.2 * scale, Color::from_rgba8(56, 189, 248, 180));
        }
        prev_left = Some((left_x, sy));
        prev_right = Some((right_x, sy));
    }

    // Player arrow
    let arrow_y = player_screen_y;
    fill_rect_xywh(pixmap, cx - 3.0 * scale, arrow_y - 6.0 * scale, 6.0 * scale, 12.0 * scale, Color::from_rgba8(255, 255, 255, 240));

    // Compass tape
    let tape_y = map_top + map_h + 8.0 * scale;
    stroke_line(pixmap, cx - 155.0 * scale, tape_y, cx + 155.0 * scale, tape_y, 1.3 * scale, Color::from_rgba8(0, 229, 255, 110));
}
