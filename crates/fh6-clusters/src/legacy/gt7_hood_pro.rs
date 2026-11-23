use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_7segment_digits, draw_gear, draw_glitch_text, draw_text};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let mut bg = Paint::default();
    bg.set_color(Color::from_rgba8(4, 6, 10, 255));
    pixmap.fill_rect(bounds, &bg, Transform::identity(), None);

    let cx = bounds.x() + bounds.width() / 2.0;
    let cy = bounds.y() + bounds.height() / 2.0;
    let scale = 1.0;

    // Driver tag with FX
    let tag = &config.custom_driver_name;
    let time_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    if config.driver_tag_glitch_fx {
        draw_glitch_text(
            pixmap,
            cx - 150.0 * scale,
            cy - 140.0 * scale,
            tag,
            Color::from_rgba8(255, 255, 255, 255),
            scale,
            time_ms,
            true,
        );
    } else {
        draw_text(
            pixmap,
            cx - 150.0 * scale,
            cy - 140.0 * scale,
            tag,
            Color::from_rgba8(255, 255, 255, 255),
            scale,
        );
    }

    let norm = data.normalized_rpm().clamp(0.0, 1.0);
    let start_angle = -147.0_f32;
    let total_sweep = 114.0_f32;
    let _fill_sweep = norm * total_sweep;

    // Swept tach arc using PathBuilder
    let tach_cx = cx;
    let tach_cy = cy + 98.0 * scale;
    let tach_r = 166.0 * scale;
    let mut pb = PathBuilder::new();
    // Approximate arc with line segments for simplicity
    let steps = 60;
    let mut first = true;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let ang = (start_angle + total_sweep * t).to_radians();
        let x = tach_cx + tach_r * ang.cos();
        let y = tach_cy + tach_r * ang.sin();
        if first {
            pb.move_to(x, y);
            first = false;
        } else {
            pb.line_to(x, y);
        }
    }
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(0, 229, 255, 255));
        let stroke = Stroke {
            width: 11.0 * scale,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    // 16 diamond shift LEDs
    let shift_r = tach_r + 15.5 * scale;
    for i in 0..16 {
        let t = (i as f32 + 0.5) / 16.0;
        let ang = start_angle + t * total_sweep;
        let rad = ang.to_radians();
        let x = tach_cx + shift_r * rad.cos();
        let y = tach_cy + shift_r * rad.sin();
        let r = 4.6 * scale;
        if let Some(rrect) = Rect::from_xywh(x - r, y - r, r * 2.0, r * 2.0) {
            let mut p = Paint::default();
            let lit = norm > 0.72 + (i as f32 / 16.0) * 0.22;
            p.set_color(if lit {
                Color::from_rgba8(0, 229, 255, 255)
            } else {
                Color::from_rgba8(20, 24, 34, 255)
            });
            pixmap.fill_rect(rrect, &p, Transform::identity(), None);
        }
    }

    // Center gear pod
    let gear_w = 80.0 * scale;
    let gear_h = 88.0 * scale;
    if let Some(gear_rect) = Rect::from_xywh(cx - 108.0 * scale, cy + 2.0 * scale, gear_w, gear_h) {
        let mut p = Paint::default();
        p.set_color(Color::from_rgba8(9, 12, 19, 255));
        pixmap.fill_rect(gear_rect, &p, Transform::identity(), None);
    }
    draw_gear(
        pixmap,
        cx - 68.0 * scale,
        cy + 40.0 * scale,
        data.gear,
        Color::from_rgba8(248, 250, 252, 255),
        scale * 2.0,
    );

    // Speed pod
    let speed_w = 140.0 * scale;
    let speed_h = gear_h;
    if let Some(speed_rect) = Rect::from_xywh(cx - 14.0 * scale, cy + 2.0 * scale, speed_w, speed_h) {
        let mut p = Paint::default();
        p.set_color(Color::from_rgba8(9, 12, 19, 255));
        pixmap.fill_rect(speed_rect, &p, Transform::identity(), None);
    }
    let speed_kmh = data.speed_ms * 3.6;
    let speed_int = speed_kmh.round() as i32;
    draw_7segment_digits(
        pixmap,
        cx + 10.0 * scale,
        cy + 30.0 * scale,
        &format!("{:03}", speed_int),
        Color::from_rgba8(248, 250, 252, 255),
        scale * 1.5,
    );

    // Left wing tire slip indicators
    let left_x = cx - 294.0 * scale;
    let left_y = cy - 4.0 * scale;
    for i in 0..4 {
        let bx = left_x + [-36.0, 22.0, -36.0, 22.0][i] * scale;
        let by = left_y + [-39.0, -39.0, 6.0, 6.0][i] * scale;
        if let Some(bar) = Rect::from_xywh(bx, by, 14.0 * scale, 34.0 * scale) {
            let mut p = Paint::default();
            p.set_color(Color::from_rgba8(10, 14, 22, 255));
            pixmap.fill_rect(bar, &p, Transform::identity(), None);
        }
    }

    // Right wing G reticle
    let right_x = cx + 294.0 * scale;
    let right_y = cy - 4.0 * scale;
    let g_r = 36.0 * scale;
    if let Some(g_rect) = Rect::from_xywh(right_x - g_r, right_y - g_r, g_r * 2.0, g_r * 2.0) {
        let mut p = Paint::default();
        p.set_color(Color::from_rgba8(8, 12, 18, 255));
        pixmap.fill_rect(g_rect, &p, Transform::identity(), None);
    }
}
