use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform, PathBuilder, Stroke};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_7segment_digits, draw_gear, draw_glitch_text};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::draw_ext::{fill_polygon, sanitize_finite};

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    // OLED true-black canvas
    let mut bg = Paint::default();
    bg.set_color(Color::from_rgba8(0, 0, 0, 255));
    pixmap.fill_rect(bounds, &bg, Transform::identity(), None);

    let cx = bounds.x() + bounds.width() / 2.0;
    let cy = bounds.y() + bounds.height() / 2.0;
    let scale = 1.0;

    // Sanitize inputs
    let accel_x = sanitize_finite(data.acceleration_x, 0.0, -200.0, 200.0);
    let accel_z = sanitize_finite(data.acceleration_z, 0.0, -200.0, 200.0);
    let roll_rad = sanitize_finite(data.roll, 0.0, -1.0, 1.0);
    let steer = sanitize_finite(data.steer as f32, 0.0, -127.0, 127.0) / 127.0;

    let lateral_g = (accel_x / 9.81).clamp(-2.0, 2.0);
    let longitudinal_g = (accel_z / 9.81).clamp(-2.0, 2.0);

    let parallax = compute_spatial_parallax_offset(lateral_g, longitudinal_g);
    let px = parallax.0;
    let py = parallax.1;

    // Background shadow layer
    let shadow_offset = 6.0;
    let base_rect = Rect::from_xywh(cx - 520.0 * scale + px + shadow_offset, cy - 300.0 * scale + py + shadow_offset, 1040.0 * scale, 600.0 * scale);
    if let Some(r) = base_rect {
        let mut p = Paint::default();
        p.set_color(Color::from_rgba8(8, 10, 18, 180));
        pixmap.fill_rect(r, &p, Transform::identity(), None);
    }

    // Foreground glass plane
    let fg_rect = Rect::from_xywh(cx - 520.0 * scale + px, cy - 300.0 * scale + py, 1040.0 * scale, 600.0 * scale);
    if let Some(r) = fg_rect {
        let mut p = Paint::default();
        p.set_color(Color::from_rgba8(22, 28, 44, 34));
        pixmap.fill_rect(r, &p, Transform::identity(), None);
    }

    // Spatial glass capsules
    let center_x = cx + px * 0.3;
    let center_y = cy + py * 0.3;
    render_vision_glass_capsules(pixmap, center_x, center_y, scale, roll_rad, steer, data);

    // Dynamic Island Fluid Ribbon tach
    let rpm_ratio = data.normalized_rpm().clamp(0.0, 1.2);
    render_fluid_dynamic_island_tach(pixmap, center_x, center_y - 120.0 * scale, scale, rpm_ratio);

    // Typography and widgets
    render_spatial_typography_and_widgets(pixmap, cx, cy, scale, data, config, px, py);

    // Driver tag watermark
    let tag = &config.custom_driver_name;
    let time_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    let tag_x = cx - 140.0 * scale + px;
    let tag_y = cy + 240.0 * scale + py;
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, tag_x, tag_y, tag, Color::from_rgba8(255, 255, 255, 220), scale * 0.8, time_ms, true);
    } else {
        draw_text(pixmap, tag_x, tag_y, tag, Color::from_rgba8(255, 255, 255, 220), scale * 0.8);
    }
}

fn compute_spatial_parallax_offset(lateral_g: f32, longitudinal_g: f32) -> (f32, f32) {
    let px = (lateral_g * 8.0).clamp(-8.0, 8.0);
    let py = (longitudinal_g * -8.0).clamp(-8.0, 8.0);
    (px, py)
}

fn render_vision_glass_capsules(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, roll_rad: f32, steer: f32, data: &FH6TelemetryData) {
    let capsules = [
        (0.0, -40.0, "Center Hero Pill"),
        (-260.0, 30.0, "Left Dynamics Pill"),
        (260.0, 30.0, "Right Power/Boost Pill"),
    ];
    for (dx, dy, _label) in capsules.iter() {
        let x = cx + dx * scale;
        let y = cy + dy * scale;
        let w = 180.0 * scale;
        let h = 72.0 * scale;
        // Frosted fill
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(22, 28, 44, 34));
        let rect = Rect::from_xywh(x - w/2.0, y - h/2.0, w, h);
        if let Some(r) = rect {
            pixmap.fill_rect(r, &paint, Transform::identity(), None);
        }
        // Specular caustic rim
        let theta_spec = roll_rad + steer * 0.5;
        let rim_color = if theta_spec.sin() > 0.0 {
            Color::from_rgba8(255, 255, 255, 120)
        } else {
            Color::from_rgba8(56, 189, 248, 120)
        };
        let mut rim_paint = Paint::default();
        rim_paint.set_color(rim_color);
        let stroke = Stroke { width: 1.5 * scale, ..Default::default() };
        let mut pb = PathBuilder::new();
        let r_ = 12.0 * scale;
        pb.move_to(x - w/2.0 + r_, y - h/2.0);
        pb.line_to(x + w/2.0 - r_, y - h/2.0);
        pb.quad_to(x + w/2.0, y - h/2.0, x + w/2.0, y - h/2.0 + r_);
        pb.line_to(x + w/2.0, y + h/2.0 - r_);
        pb.quad_to(x + w/2.0, y + h/2.0, x + w/2.0 - r_, y + h/2.0);
        pb.line_to(x - w/2.0 + r_, y + h/2.0);
        pb.quad_to(x - w/2.0, y + h/2.0, x - w/2.0, y + h/2.0 - r_);
        pb.line_to(x - w/2.0, y - h/2.0 + r_);
        pb.quad_to(x - w/2.0, y - h/2.0, x - w/2.0 + r_, y - h/2.0);
        pb.close();
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &rim_paint, &stroke, Transform::identity(), None);
        }

        // Capsule specific content
        if *dx == 0.0 {
            // Hero gear
            draw_gear(pixmap, x, y - 10.0 * scale, data.gear, Color::from_rgba8(255, 255, 255, 255), scale * 1.5);
        } else if *dx < 0.0 {
            // Dynamics: throttle/brake bars
            let t = data.throttle_pct().clamp(0.0, 1.0);
            let b = data.brake_pct().clamp(0.0, 1.0);
            let mut tp = Paint::default();
            tp.set_color(Color::from_rgba8(34, 197, 94, 200));
            fill_polygon(pixmap, &[(x - 60.0*scale, y + 10.0*scale), (x - 60.0*scale + 120.0*t*scale, y + 10.0*scale), (x - 60.0*scale + 120.0*t*scale, y + 22.0*scale), (x - 60.0*scale, y + 22.0*scale)], Color::from_rgba8(34, 197, 94, 200));
            let mut bp = Paint::default();
            bp.set_color(Color::from_rgba8(239, 68, 68, 200));
            fill_polygon(pixmap, &[(x - 60.0*scale, y + 30.0*scale), (x - 60.0*scale + 120.0*b*scale, y + 30.0*scale), (x - 60.0*scale + 120.0*b*scale, y + 42.0*scale), (x - 60.0*scale, y + 42.0*scale)], Color::from_rgba8(239, 68, 68, 200));
        } else {
            // Power/Boost
            let hp = (data.power_watts / 745.7).max(0.0);
            let hp_str = format!("{:.0}", hp);
            draw_text(pixmap, x - 40.0*scale, y, &hp_str, Color::from_rgba8(56, 189, 248, 255), scale);
        }
    }
}

fn render_fluid_dynamic_island_tach(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, rpm_ratio: f32) {
    let t = rpm_ratio.clamp(0.0, 1.0);
    let width = 320.0 * scale * t;
    let height = 8.0 * scale;
    let x = cx - width / 2.0;
    let y = cy;
    // Color gradient
    let color = if t < 0.6 {
        Color::from_rgba8(56, 189, 248, 220)
    } else if t < 0.85 {
        Color::from_rgba8(129, 140, 248, 220)
    } else {
        Color::from_rgba8(244, 63, 94, 220)
    };
    let mut paint = Paint::default();
    paint.set_color(color);
    let rect = Rect::from_xywh(x, y, width, height);
    if let Some(r) = rect {
        pixmap.fill_rect(r, &paint, Transform::identity(), None);
    }
    // Bloom strobe at redline
    if t > 0.95 {
        let mut bloom = Paint::default();
        bloom.set_color(Color::from_rgba8(255, 255, 255, 80));
        let bloom_rect = Rect::from_xywh(x - 10.0*scale, y - 4.0*scale, width + 20.0*scale, height + 8.0*scale);
        if let Some(r) = bloom_rect {
            pixmap.fill_rect(r, &bloom, Transform::identity(), None);
        }
    }
}

fn render_spatial_typography_and_widgets(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData, _config: &AppConfig, px: f32, py: f32) {
    let speed_kmh = data.speed_kmh();
    let speed_int = speed_kmh.round() as i32;
    // Parallax drop shadow
    draw_7segment_digits(pixmap, cx - 80.0*scale + px*0.5, cy + 80.0*scale + py*0.5, &format!("{:03}", speed_int), Color::from_rgba8(255, 255, 255, 60), scale * 1.4);
    draw_7segment_digits(pixmap, cx - 80.0*scale + px, cy + 80.0*scale + py, &format!("{:03}", speed_int), Color::from_rgba8(255, 255, 255, 255), scale * 1.4);

    // Hero gear
    draw_gear(pixmap, cx + 120.0*scale + px, cy + 80.0*scale + py, data.gear, Color::from_rgba8(255, 255, 255, 255), scale * 1.8);

    // Concentric rings for throttle, brake, power
    let rings = [
        (data.throttle_pct().clamp(0.0, 1.0), Color::from_rgba8(34, 197, 94, 200)),
        (data.brake_pct().clamp(0.0, 1.0), Color::from_rgba8(239, 68, 68, 200)),
        ((data.power_watts / 745.7 / 800.0).clamp(0.0, 1.0), Color::from_rgba8(56, 189, 248, 200)),
    ];
    let base_cx = cx;
    let base_cy = cy - 80.0*scale;
    for (i, (val, col)) in rings.iter().enumerate() {
        let r = 40.0 * scale + i as f32 * 12.0 * scale;
        let mut pb = PathBuilder::new();
        let start = -1.5708;
        let end = start + 2.0 * std::f32::consts::PI * val;
        let steps = 40;
        for j in 0..=steps {
            let ang = start + (end - start) * j as f32 / steps as f32;
            let x_ = base_cx + r * ang.cos();
            let y_ = base_cy + r * ang.sin();
            if j == 0 { pb.move_to(x_, y_); } else { pb.line_to(x_, y_); }
        }
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(*col);
            let stroke = Stroke { width: 2.0 * scale, ..Default::default() };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}
