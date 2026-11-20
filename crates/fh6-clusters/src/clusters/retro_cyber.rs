use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform, PathBuilder, Stroke};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_7segment_digits, draw_gear, draw_glitch_text};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    // Background
    let mut bg = Paint::default();
    bg.set_color(Color::from_rgba8(6, 8, 12, 255));
    pixmap.fill_rect(bounds, &bg, Transform::identity(), None);

    let cx = bounds.x() + bounds.width() / 2.0;
    let cy = bounds.y() + bounds.height() / 2.0;
    let scale = 1.0;

    // 45° chamfered cybernetic bezel - dark cavity
    let bezel_w = 520.0 * scale;
    let bezel_h = 300.0 * scale;
    let bezel_rect = match Rect::from_xywh(cx - bezel_w / 2.0, cy - bezel_h / 2.0, bezel_w, bezel_h) {
        Some(r) => r,
        None => return,
    };
    let mut cavity = Paint::default();
    cavity.set_color(Color::from_rgba8(2, 2, 4, 255));
    pixmap.fill_rect(bezel_rect, &cavity, Transform::identity(), None);

    // Bezel rim
    let mut rim = Paint::default();
    rim.set_color(Color::from_rgba8(80, 80, 100, 200));
    let rim_stroke = Stroke { width: 3.0 * scale, ..Default::default() };
    let mut rim_pb = PathBuilder::new();
    rim_pb.move_to(bezel_rect.x() + 8.0 * scale, bezel_rect.y() + 8.0 * scale);
    rim_pb.line_to(bezel_rect.x() + bezel_w - 8.0 * scale, bezel_rect.y() + 8.0 * scale);
    rim_pb.line_to(bezel_rect.x() + bezel_w - 8.0 * scale, bezel_rect.y() + bezel_h - 8.0 * scale);
    rim_pb.line_to(bezel_rect.x() + 8.0 * scale, bezel_rect.y() + bezel_h - 8.0 * scale);
    rim_pb.close();
    if let Some(path) = rim_pb.finish() {
        pixmap.stroke_path(&path, &rim, &rim_stroke, Transform::identity(), None);
    }

    // CRT scanlines
    let mut scan_paint = Paint::default();
    scan_paint.set_color(Color::from_rgba8(0, 0, 0, 60));
    for i in 0..(bezel_h / 4.0) as i32 {
        let y = bezel_rect.y() + i as f32 * 4.0 * scale;
        if let Some(line_rect) = Rect::from_xywh(bezel_rect.x(), y, bezel_w, 2.0 * scale) {
            pixmap.fill_rect(line_rect, &scan_paint, Transform::identity(), None);
        }
    }

    // 24-segment chevron LED arc
    let arc_radius = 110.0 * scale;
    let start_angle = -135.0_f32;
    let total_sweep = 270.0_f32;
    let segments = 24;
    for i in 0..segments {
        let t = i as f32 / segments as f32;
        let ang = start_angle + t * total_sweep;
        let rad = ang.to_radians();
        let x = cx + arc_radius * rad.cos();
        let y = cy - 60.0 * scale + arc_radius * rad.sin() * 0.6;
        // Color bands: 14 Cyan, 6 Yellow, 4 Neon Magenta
        let col = if i < 14 {
            Color::from_rgba8(0, 229, 255, 255)
        } else if i < 20 {
            Color::from_rgba8(255, 179, 0, 255)
        } else {
            Color::from_rgba8(255, 0, 255, 255)
        };
        let mut seg_paint = Paint::default();
        seg_paint.set_color(col);
        let seg_size = 6.0 * scale;
        if let Some(seg_rect) = Rect::from_xywh(x - seg_size / 2.0, y - seg_size / 2.0, seg_size, seg_size) {
            pixmap.fill_rect(seg_rect, &seg_paint, Transform::identity(), None);
        }
    }

    // Octagonal gear bay [4]
    let gear_bay_x = cx - 140.0 * scale;
    let gear_bay_y = cy + 50.0 * scale;
    let mut bay_paint = Paint::default();
    bay_paint.set_color(Color::from_rgba8(20, 20, 30, 255));
    if let Some(bay_rect) = Rect::from_xywh(gear_bay_x, gear_bay_y, 80.0 * scale, 80.0 * scale) {
        pixmap.fill_rect(bay_rect, &bay_paint, Transform::identity(), None);
    }
    // Octagon outline
    let mut oct_pb = PathBuilder::new();
    let s = 40.0 * scale;
    oct_pb.move_to(gear_bay_x + s, gear_bay_y);
    oct_pb.line_to(gear_bay_x + 80.0 * scale - s, gear_bay_y);
    oct_pb.line_to(gear_bay_x + 80.0 * scale, gear_bay_y + s);
    oct_pb.line_to(gear_bay_x + 80.0 * scale, gear_bay_y + 80.0 * scale - s);
    oct_pb.line_to(gear_bay_x + 80.0 * scale - s, gear_bay_y + 80.0 * scale);
    oct_pb.line_to(gear_bay_x + s, gear_bay_y + 80.0 * scale);
    oct_pb.line_to(gear_bay_x, gear_bay_y + 80.0 * scale - s);
    oct_pb.line_to(gear_bay_x, gear_bay_y + s);
    oct_pb.close();
    let mut oct_paint = Paint::default();
    oct_paint.set_color(Color::from_rgba8(0, 229, 255, 180));
    let oct_stroke = Stroke { width: 2.0 * scale, ..Default::default() };
    if let Some(oct_path) = oct_pb.finish() {
        pixmap.stroke_path(&oct_path, &oct_paint, &oct_stroke, Transform::identity(), None);
    }

    // Driver tag
    let tag = &config.custom_driver_name;
    let time_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as u64,
        Err(_) => 0,
    };
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx - 130.0 * scale, cy - 140.0 * scale, tag, Color::from_rgba8(255, 255, 255, 255), scale, time_ms, true);
    } else {
        draw_text(pixmap, cx - 130.0 * scale, cy - 140.0 * scale, tag, Color::from_rgba8(255, 255, 255, 255), scale);
    }

    // 7-segment speed
    let speed_kmh = data.speed_kmh();
    let speed_int = speed_kmh.round() as i32;
    draw_7segment_digits(pixmap, cx - 60.0 * scale, cy + 40.0 * scale, &format!("{:03}", speed_int), Color::from_rgba8(0, 229, 255, 255), scale * 1.2);

    // Gear
    draw_gear(pixmap, cx + 100.0 * scale, cy + 10.0 * scale, data.gear, Color::from_rgba8(255, 255, 255, 255), scale * 1.8);
}
