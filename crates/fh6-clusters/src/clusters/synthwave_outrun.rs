use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform, PathBuilder, Stroke};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_7segment_digits, draw_gear, draw_glitch_text};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    // Strict 420x190 binnacle constraint
    let binnacle_w = 420.0;
    let binnacle_h = 190.0;
    let cx = bounds.x() + bounds.width() / 2.0;
    let cy = bounds.y() + bounds.height() / 2.0;
    let scale = 1.0;
    let bin_x = cx - binnacle_w / 2.0 * scale;
    let bin_y = cy - binnacle_h / 2.0 * scale;

    // Background
    let mut bg = Paint::default();
    bg.set_color(Color::from_rgba8(6, 8, 12, 255));
    pixmap.fill_rect(bounds, &bg, Transform::identity(), None);

    // Binnacle panel
    if let Some(panel_rect) = Rect::from_xywh(bin_x, bin_y, binnacle_w * scale, binnacle_h * scale) {
        let mut panel = Paint::default();
        panel.set_color(Color::from_rgba8(10, 12, 20, 255));
        pixmap.fill_rect(panel_rect, &panel, Transform::identity(), None);
    }

    // Outrun sun gradient top right
    let sun_radius = 70.0 * scale;
    let sun_cx = cx + binnacle_w * 0.35 * scale;
    let sun_cy = bin_y + 40.0 * scale;
    let mut sun_paint = Paint::default();
    sun_paint.set_color(Color::from_rgba8(255, 120, 0, 200));
    if let Some(sun_rect) = Rect::from_xywh(sun_cx - sun_radius, sun_cy - sun_radius, sun_radius * 2.0, sun_radius * 2.0) {
        pixmap.fill_rect(sun_rect, &sun_paint, Transform::identity(), None);
    }
    // Sun halo
    let mut halo = Paint::default();
    halo.set_color(Color::from_rgba8(255, 200, 80, 80));
    if let Some(halo_rect) = Rect::from_xywh(sun_cx - sun_radius * 1.5, sun_cy - sun_radius * 1.5, sun_radius * 3.0, sun_radius * 3.0) {
        pixmap.fill_rect(halo_rect, &halo, Transform::identity(), None);
    }

    // Neon horizon grid
    let grid_color = Color::from_rgba8(0, 229, 255, 180);
    let mut grid_paint = Paint::default();
    grid_paint.set_color(grid_color);
    let grid_stroke = Stroke { width: 1.2 * scale, ..Default::default() };
    let horizon_y = cy + 20.0 * scale;
    // Horizontal lines receding
    for i in 0..5 {
        let t = i as f32 / 4.0;
        let y = horizon_y + t * 80.0 * scale;
        let w = binnacle_w * (1.0 - t * 0.3) * scale;
        let x0 = cx - w / 2.0;
        let mut line_pb = PathBuilder::new();
        line_pb.move_to(x0, y);
        line_pb.line_to(x0 + w, y);
        if let Some(path) = line_pb.finish() {
            pixmap.stroke_path(&path, &grid_paint, &grid_stroke, Transform::identity(), None);
        }
    }
    // Vertical lines
    for i in 0..7 {
        let t = i as f32 / 6.0;
        let x = cx - binnacle_w * 0.45 * scale + t * binnacle_w * 0.9 * scale;
        let mut v_pb = PathBuilder::new();
        v_pb.move_to(x, horizon_y);
        v_pb.line_to(x * 0.9 + cx * 0.1, cy + 90.0 * scale);
        if let Some(path) = v_pb.finish() {
            pixmap.stroke_path(&path, &grid_paint, &grid_stroke, Transform::identity(), None);
        }
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
    draw_7segment_digits(pixmap, cx - 60.0 * scale, cy + 40.0 * scale, &format!("{:03}", speed_int), Color::from_rgba8(0, 229, 255, 255), scale * 1.2);

    // Gear
    draw_gear(pixmap, cx + 100.0 * scale, cy + 10.0 * scale, data.gear, Color::from_rgba8(255, 255, 255, 255), scale * 1.8);
}
