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

    // Frosted sapphire glass panel
    let panel_w = 560.0 * scale;
    let panel_h = 320.0 * scale;
    let panel_rect = Rect::from_xywh(cx - panel_w / 2.0, cy - panel_h / 2.0, panel_w, panel_h);
    let panel_rect = match panel_rect {
        Some(r) => r,
        None => return,
    };
    let mut panel_paint = Paint::default();
    panel_paint.set_color(Color::from_rgba8(18, 24, 38, 220));
    pixmap.fill_rect(panel_rect, &panel_paint, Transform::identity(), None);

    // Specular bevel rim - outer white to cyan gradient
    let mut bevel = Paint::default();
    bevel.set_color(Color::from_rgba8(255, 255, 255, 80));
    // Simulate rim with stroked rect
    let stroke = Stroke { width: 2.0 * scale, ..Default::default() };
    let mut pb = PathBuilder::new();
    let r = 24.0 * scale;
    let x = panel_rect.x();
    let y = panel_rect.y();
    let w = panel_rect.width();
    let h = panel_rect.height();
    // Rounded rect approximated by rect
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.line_to(x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.line_to(x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.line_to(x, y + h - r);
    pb.line_to(x, y + r);
    pb.close();
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &bevel, &stroke, Transform::identity(), None);
    }

    // Arched optical sheen
    let mut sheen = Paint::default();
    sheen.set_color(Color::from_rgba8(56, 189, 248, 60));
    if let Some(sheen_rect) = Rect::from_xywh(cx - panel_w * 0.45, cy - panel_h * 0.35, panel_w * 0.9, panel_h * 0.15) {
        pixmap.fill_rect(sheen_rect, &sheen, Transform::identity(), None);
    }

    // Luminous fluid light-ribbon tachometer
    let _norm_rpm = data.normalized_rpm().clamp(0.0, 1.2);
    let tach_radius = 110.0 * scale;
    let start_angle = -135.0_f32;
    let sweep = 270.0_f32;
    let steps = 120;
    let mut prev_pt = None;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let ang = start_angle + t * sweep;
        let rad = ang.to_radians();
        let x_off = tach_radius * rad.cos();
        let y_off = tach_radius * rad.sin();
        let px = cx + x_off;
        let py = cy - 40.0 * scale + y_off;
        // Ribbon color gradient cyan -> amber -> crimson
        let col = if t < 0.6 {
            Color::from_rgba8(0, 229, 255, 200)
        } else if t < 0.85 {
            Color::from_rgba8(255, 179, 0, 200)
        } else {
            Color::from_rgba8(255, 23, 68, 200)
        };
        let mut ribbon_paint = Paint::default();
        ribbon_paint.set_color(col);
        // Draw small line segment
        if let Some((px0, py0)) = prev_pt {
            let mut line_pb = PathBuilder::new();
            line_pb.move_to(px0, py0);
            line_pb.line_to(px, py);
            if let Some(line_path) = line_pb.finish() {
                let line_stroke = Stroke { width: 4.0 * scale, ..Default::default() };
                pixmap.stroke_path(&line_path, &ribbon_paint, &line_stroke, Transform::identity(), None);
            }
        }
        prev_pt = Some((px, py));
    }

    // Radial meniscus orb
    let orb_r = 36.0 * scale;
    let mut orb_paint = Paint::default();
    orb_paint.set_color(Color::from_rgba8(56, 189, 248, 180));
    if let Some(orb_rect) = Rect::from_xywh(cx - orb_r, cy + 70.0 * scale - orb_r, orb_r * 2.0, orb_r * 2.0) {
        pixmap.fill_rect(orb_rect, &orb_paint, Transform::identity(), None);
    }
    // Inner highlight
    let mut inner = Paint::default();
    inner.set_color(Color::from_rgba8(255, 255, 255, 40));
    if let Some(inner_rect) = Rect::from_xywh(cx - orb_r * 0.5, cy + 70.0 * scale - orb_r * 0.5, orb_r, orb_r) {
        pixmap.fill_rect(inner_rect, &inner, Transform::identity(), None);
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
