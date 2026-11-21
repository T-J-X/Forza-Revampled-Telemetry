use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform, PathBuilder, Stroke};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_gear, draw_glitch_text};
use crate::draw_ext::fill_polygon;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let mut bg = Paint::default();
    bg.set_color(Color::from_rgba8(0,0,0,255));
    pixmap.fill_rect(bounds, &bg, Transform::identity(), None);

    let cx = bounds.x() + bounds.width() / 2.0;
    let cy = bounds.y() + bounds.height() / 2.0;
    let scale = 1.0;

    render_sculpted_aero_blade_chassis(pixmap, cx, cy, scale);
    let rpm_ratio = data.normalized_rpm().clamp(0.0,1.2);
    render_convergent_collision_tachometer(pixmap, cx, cy, scale, rpm_ratio, data);
    render_active_aero_drs_and_downforce_vectors(pixmap, cx, cy, scale, data, config);
}

fn render_sculpted_aero_blade_chassis(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32) {
    let w = 720.0 * scale;
    let h = 140.0 * scale;
    let pts = [
        (cx - w/2.0, cy - h/2.0),
        (cx - w/2.0 + 120.0*scale, cy - h/2.0),
        (cx, cy - h/2.0 + 20.0*scale),
        (cx + w/2.0 - 120.0*scale, cy - h/2.0),
        (cx + w/2.0, cy - h/2.0),
        (cx + w/2.0, cy + h/2.0),
        (cx + w/2.0 - 120.0*scale, cy + h/2.0),
        (cx, cy + h/2.0 - 20.0*scale),
        (cx - w/2.0 + 120.0*scale, cy + h/2.0),
        (cx - w/2.0, cy + h/2.0),
    ];
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(15,18,24,255));
    fill_polygon(pixmap, &pts, Color::from_rgba8(15,18,24,255));
    // Venturi cutouts
    for i in [-1,0,1] {
        let vx = cx + i as f32 * 180.0*scale;
        let mut cut_paint = Paint::default();
        cut_paint.set_color(Color::from_rgba8(0,0,0,255));
        let cut_pts = [
            (vx - 40.0*scale, cy - 20.0*scale),
            (vx + 40.0*scale, cy - 20.0*scale),
            (vx + 20.0*scale, cy + 20.0*scale),
            (vx - 20.0*scale, cy + 20.0*scale),
        ];
        fill_polygon(pixmap, &cut_pts, Color::from_rgba8(0,0,0,255));
    }
    // Laser accent line
    let mut line_paint = Paint::default();
    line_paint.set_color(Color::from_rgba8(255, 140, 0, 255));
    let stroke = Stroke { width: 1.0*scale, ..Default::default() };
    let mut pb = PathBuilder::new();
    pb.move_to(cx - w/2.0, cy);
    pb.line_to(cx + w/2.0, cy);
    if let Some(path)=pb.finish() { pixmap.stroke_path(&path, &line_paint, &stroke, Transform::identity(), None); }
}

fn render_convergent_collision_tachometer(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, rpm_ratio: f32, data: &FH6TelemetryData) {
    let left_x = cx - 360.0*scale;
    let right_x = cx + 360.0*scale;
    let y_top = cy - 60.0*scale;
    let y_bottom = cy + 60.0*scale;
    let t = rpm_ratio.clamp(0.0,1.0);
    let meet_x = left_x + (right_x - left_x) * t;
    // Left ribbon
    let left_pts = [(left_x, y_top), (meet_x, y_top + (y_bottom - y_top)*0.5), (left_x, y_bottom)];
    let color_left = if t < 0.6 { Color::from_rgba8(0,240,255,220) } else if t < 0.85 { Color::from_rgba8(245,158,11,220) } else { Color::from_rgba8(255,0,60,220) };
    fill_polygon(pixmap, &left_pts, color_left);
    // Right ribbon
    let right_pts = [(right_x, y_top), (meet_x, y_top + (y_bottom - y_top)*0.5), (right_x, y_bottom)];
    fill_polygon(pixmap, &right_pts, color_left);
    // Shockwave on collision
    if t >= 0.94 {
        let mut shock_paint = Paint::default();
        shock_paint.set_color(Color::from_rgba8(255,255,255,200));
        let stroke = Stroke { width: 2.0*scale, ..Default::default() };
        let mut pb = PathBuilder::new();
        pb.move_to(meet_x - 40.0*scale, cy);
        pb.line_to(meet_x + 40.0*scale, cy);
        if let Some(path)=pb.finish() { pixmap.stroke_path(&path, &shock_paint, &stroke, Transform::identity(), None); }
        // Diamond shift lights
        for i in -2..=2 {
            let x = meet_x + i as f32 * 18.0*scale;
            let mut dp = Paint::default();
            dp.set_color(Color::from_rgba8(255,255,255,255));
            let rect = Rect::from_xywh(x-3.0*scale, cy-3.0*scale, 6.0*scale, 6.0*scale);
            if let Some(r)=rect { pixmap.fill_rect(r, &dp, Transform::identity(), None); }
        }
    }
    // Hero gear at center
    draw_gear(pixmap, cx, cy + 70.0*scale, data.gear, Color::from_rgba8(255,255,255,255), scale*1.5);
}

fn render_active_aero_drs_and_downforce_vectors(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData, config: &AppConfig) {
    // Left wing module
    let left_x = cx - 280.0*scale;
    let left_y = cy + 120.0*scale;
    let speed_sq = data.speed_ms * data.speed_ms;
    let load = (speed_sq * 0.0005).clamp(0.0, 1.0);
    // Wireframe silhouette
    let mut wire_paint = Paint::default();
    wire_paint.set_color(Color::from_rgba8(200,200,210,180));
    let stroke = Stroke { width: 1.2*scale, ..Default::default() };
    let pts = [
        (left_x - 40.0*scale, left_y),
        (left_x + 40.0*scale, left_y - 20.0*scale),
        (left_x + 80.0*scale, left_y),
        (left_x + 40.0*scale, left_y + 20.0*scale),
    ];
    let mut pb = PathBuilder::new();
    pb.move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] { pb.line_to(p.0, p.1); }
    pb.close();
    if let Some(path)=pb.finish() { pixmap.stroke_path(&path, &wire_paint, &stroke, Transform::identity(), None); }
    // Load bars
    let bar_h = 60.0*scale*load;
    let mut bar_paint = Paint::default();
    bar_paint.set_color(Color::from_rgba8(34,197,94,200));
    let rect = Rect::from_xywh(left_x - 10.0*scale, left_y + 20.0*scale - bar_h, 6.0*scale, bar_h);
    if let Some(r)=rect { pixmap.fill_rect(r, &bar_paint, Transform::identity(), None); }
    // Airbrake indicator
    if data.brake as f32 / 255.0 > 0.20 {
        let mut bp = Paint::default();
        bp.set_color(Color::from_rgba8(239,68,68,255));
        let rect = Rect::from_xywh(left_x - 30.0*scale, left_y - 50.0*scale, 60.0*scale, 10.0*scale);
        if let Some(r)=rect { pixmap.fill_rect(r, &bp, Transform::identity(), None); }
    }

    // Right wing module
    let right_x = cx + 280.0*scale;
    let right_y = cy + 120.0*scale;
    let hp = (data.power_watts / 745.7).max(0.0);
    draw_text(pixmap, right_x - 40.0*scale, right_y, &format!("{:.0}HP", hp), Color::from_rgba8(56,189,248,255), scale*0.8);
    // Tire thermal pills
    let tires = [(-1.0,-1.0),(1.0,-1.0),(-1.0,1.0),(1.0,1.0)];
    for (sx,sy) in tires.iter() {
        let tx = right_x + sx * 30.0*scale;
        let ty = right_y + sy * 30.0*scale;
        let temp = (data.normalized_suspension_travel_fl + data.normalized_suspension_travel_fr + data.normalized_suspension_travel_rl + data.normalized_suspension_travel_rr)/4.0;
        let col = if temp > 0.7 { Color::from_rgba8(239,68,68,255) } else { Color::from_rgba8(56,189,248,255) };
        let mut tp = Paint::default();
        tp.set_color(col);
        let rect = Rect::from_xywh(tx - 8.0*scale, ty - 6.0*scale, 16.0*scale, 12.0*scale);
        if let Some(r)=rect { pixmap.fill_rect(r, &tp, Transform::identity(), None); }
    }

    // Driver tag
    let tag = &config.custom_driver_name;
    let time_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx - 90.0*scale, cy + 200.0*scale, tag, Color::from_rgba8(255,255,255,255), scale*0.7, time_ms, true);
    } else {
        draw_text(pixmap, cx - 90.0*scale, cy + 200.0*scale, tag, Color::from_rgba8(255,255,255,255), scale*0.7);
    }
}
