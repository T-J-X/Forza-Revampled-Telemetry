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

    // Swept panoramic triple-wing display
    let wing_w = 180.0 * scale;
    let wing_h = 260.0 * scale;
    let gap = 12.0 * scale;

    // Left wing
    if let Some(left_rect) = Rect::from_xywh(cx - wing_w - gap, cy - wing_h / 2.0, wing_w, wing_h) {
        let mut wing_paint = Paint::default();
        wing_paint.set_color(Color::from_rgba8(18, 22, 32, 220));
        pixmap.fill_rect(left_rect, &wing_paint, Transform::identity(), None);
        // Wing bevel
        let mut bevel = Paint::default();
        bevel.set_color(Color::from_rgba8(255, 255, 255, 40));
        let mut pb = PathBuilder::new();
        pb.move_to(left_rect.x() + 4.0 * scale, left_rect.y() + 4.0 * scale);
        pb.line_to(left_rect.x() + wing_w - 4.0 * scale, left_rect.y() + 4.0 * scale);
        if let Some(path) = pb.finish() {
            let stroke = Stroke { width: 2.0 * scale, ..Default::default() };
            pixmap.stroke_path(&path, &bevel, &stroke, Transform::identity(), None);
        }
    }

    // Center wing
    if let Some(center_rect) = Rect::from_xywh(cx - wing_w / 2.0, cy - wing_h / 2.0, wing_w, wing_h) {
        let mut center_paint = Paint::default();
        center_paint.set_color(Color::from_rgba8(22, 28, 40, 240));
        pixmap.fill_rect(center_rect, &center_paint, Transform::identity(), None);
    }

    // Right wing
    if let Some(right_rect) = Rect::from_xywh(cx + wing_w / 2.0 + gap, cy - wing_h / 2.0, wing_w, wing_h) {
        let mut wing_paint = Paint::default();
        wing_paint.set_color(Color::from_rgba8(18, 22, 32, 220));
        pixmap.fill_rect(right_rect, &wing_paint, Transform::identity(), None);
    }

    // Panoramic sweep line
    let mut sweep = Paint::default();
    sweep.set_color(Color::from_rgba8(0, 229, 255, 180));
    let sweep_stroke = Stroke { width: 3.0 * scale, ..Default::default() };
    let mut sweep_pb = PathBuilder::new();
    sweep_pb.move_to(cx - wing_w - gap, cy);
    sweep_pb.line_to(cx + wing_w + gap, cy);
    if let Some(path) = sweep_pb.finish() {
        pixmap.stroke_path(&path, &sweep, &sweep_stroke, Transform::identity(), None);
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

    // Speed
    let speed_kmh = data.speed_kmh();
    let speed_int = speed_kmh.round() as i32;
    draw_7segment_digits(pixmap, cx - 60.0 * scale, cy + 40.0 * scale, &format!("{:03}", speed_int), Color::from_rgba8(255, 255, 255, 255), scale * 1.2);

    // Gear
    draw_gear(pixmap, cx + 100.0 * scale, cy + 10.0 * scale, data.gear, Color::from_rgba8(255, 255, 255, 255), scale * 1.8);
}
