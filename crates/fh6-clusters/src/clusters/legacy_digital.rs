use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform, PathBuilder, Stroke};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_7segment_digits, draw_gear, draw_glitch_text};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let mut bg = Paint::default();
    bg.set_color(Color::from_rgba8(6, 8, 12, 255));
    pixmap.fill_rect(bounds, &bg, Transform::identity(), None);

    let cx = bounds.x() + bounds.width() / 2.0;
    let cy = bounds.y() + bounds.height() / 2.0;
    let scale = 1.0;

    let norm_rpm = data.normalized_rpm().clamp(0.0, 1.2);
    let shift_start = config.shift_start_pct;
    let redline = config.redline_pct;

    // Top 40-segment RPM bar
    let bar_w = 560.0 * scale;
    let bar_h = 14.0 * scale;
    let bar_x = cx - bar_w / 2.0;
    let bar_y = cy - 130.0 * scale;
    if Rect::from_xywh(bar_x, bar_y, bar_w, bar_h).is_some() {
        let seg_count = 40;
        let gap = 2.0 * scale;
        let seg_w = (bar_w - gap * (seg_count as f32 - 1.0)) / seg_count as f32;
        let lit = (norm_rpm * seg_count as f32).round() as i32;
        for i in 0..seg_count {
            let sx = bar_x + i as f32 * (seg_w + gap);
            if let Some(seg_rect) = Rect::from_xywh(sx, bar_y, seg_w, bar_h) {
                let t = (i as f32 + 1.0) / seg_count as f32;
                let col = if norm_rpm >= redline {
                    Color::from_rgba8(255, 23, 68, 255)
                } else if i < lit {
                    if t >= 0.90 {
                        Color::from_rgba8(255, 23, 68, 255)
                    } else if t >= 0.82 {
                        Color::from_rgba8(255, 179, 0, 255)
                    } else if t >= 0.72 {
                        Color::from_rgba8(0, 230, 118, 255)
                    } else {
                        Color::from_rgba8(0, 229, 255, 255)
                    }
                } else {
                    Color::from_rgba8(20, 25, 35, 255)
                };
                let mut p = Paint::default();
                p.set_color(col);
                pixmap.fill_rect(seg_rect, &p, Transform::identity(), None);
            }
        }
        // Peak hold tick
        let peak_x = bar_x + norm_rpm * bar_w;
        let mut peak = Paint::default();
        peak.set_color(Color::from_rgba8(255, 255, 255, 200));
        let mut pb = PathBuilder::new();
        pb.move_to(peak_x, bar_y - 4.0 * scale);
        pb.line_to(peak_x, bar_y + bar_h + 4.0 * scale);
        if let Some(path) = pb.finish() {
            let stroke = Stroke { width: 2.0 * scale, ..Default::default() };
            pixmap.stroke_path(&path, &peak, &stroke, Transform::identity(), None);
        }
    }

    // Pedal bricks left/right
    let pedal_w = 12.0 * scale;
    let pedal_h = 120.0 * scale;
    let left_x = cx - bar_w / 2.0 - 20.0 * scale;
    let right_x = cx + bar_w / 2.0 + 8.0 * scale;
    let bricks_y = cy - 50.0 * scale;
    draw_pedal_bricks(pixmap, left_x, bricks_y, pedal_w, pedal_h, data.brake_pct(), Color::from_rgba8(255, 23, 68, 255), scale);
    draw_pedal_bricks(pixmap, right_x, bricks_y, pedal_w, pedal_h, data.throttle_pct(), Color::from_rgba8(0, 230, 118, 255), scale);

    // Hero gear bay
    let gear_bay_w = 104.0 * scale;
    let gear_bay_h = 120.0 * scale;
    let gear_x = cx - gear_bay_w / 2.0;
    let gear_y = bricks_y;
    if let Some(bay_rect) = Rect::from_xywh(gear_x, gear_y, gear_bay_w, gear_bay_h) {
        let mut bay_paint = Paint::default();
        bay_paint.set_color(Color::from_rgba8(15, 20, 30, 255));
        pixmap.fill_rect(bay_rect, &bay_paint, Transform::identity(), None);
        let mut accent = Paint::default();
        accent.set_color(if norm_rpm >= shift_start { Color::from_rgba8(255, 179, 0, 255) } else { Color::from_rgba8(0, 229, 255, 255) });
        if let Some(acc_rect) = Rect::from_xywh(gear_x + 8.0 * scale, gear_y + 4.0 * scale, gear_bay_w - 16.0 * scale, 4.0 * scale) {
            pixmap.fill_rect(acc_rect, &accent, Transform::identity(), None);
        }
        draw_gear(pixmap, cx, gear_y + gear_bay_h * 0.6, data.gear, Color::from_rgba8(255, 255, 255, 255), scale * 2.5);
    }

    // Tabular speed card
    let speed_w = 140.0 * scale;
    let speed_h = 100.0 * scale;
    let speed_x = cx + gear_bay_w / 2.0 + 20.0 * scale;
    let speed_y = bricks_y;
    if let Some(card_rect) = Rect::from_xywh(speed_x, speed_y, speed_w, speed_h) {
        let mut card_paint = Paint::default();
        card_paint.set_color(Color::from_rgba8(12, 16, 24, 255));
        pixmap.fill_rect(card_rect, &card_paint, Transform::identity(), None);
        draw_7segment_digits(pixmap, speed_x + 10.0 * scale, speed_y + 40.0 * scale, "888", Color::from_rgba8(148, 163, 184, 80), scale * 1.2);
        let speed_kmh = data.speed_kmh();
        let speed_int = speed_kmh.round() as i32;
        draw_7segment_digits(pixmap, speed_x + 10.0 * scale, speed_y + 40.0 * scale, &format!("{:03}", speed_int), Color::from_rgba8(255, 255, 255, 255), scale * 1.2);
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
}

fn draw_pedal_bricks(pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, pct: f32, col: Color, scale: f32) {
    let bricks = 12;
    let pad = 2.0 * scale;
    let brick_h = (h - pad * 2.0 - 1.6 * scale * (bricks as f32 - 1.0)) / bricks as f32;
    let lit = (pct.clamp(0.0, 1.0) * bricks as f32).round() as i32;
    for i in 0..bricks {
        let by = y + h - pad - (i as f32 + 1.0) * brick_h - i as f32 * 1.6 * scale;
        if let Some(rect) = Rect::from_xywh(x + pad, by, w - pad * 2.0, brick_h) {
            let mut p = Paint::default();
            if i < lit {
                p.set_color(col);
            } else {
                p.set_color(Color::from_rgba8(22, 27, 38, 255));
            }
            pixmap.fill_rect(rect, &p, Transform::identity(), None);
        }
    }
}
