use tiny_skia::{Color, PixmapMut, Rect};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_glitch_text};
use crate::draw_ext::*;

fn sanitize(v: f32, min_v: f32, max_v: f32) -> f32 { if !v.is_finite() { min_v } else { v.clamp(min_v, max_v) } }

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let cx = bounds.x() + bounds.width()*0.5;
    let cy = bounds.y() + bounds.height()*0.5;
    let scale = 1.0;
    fill_rect_xywh(pixmap, bounds.x(), bounds.y(), bounds.width(), bounds.height(), Color::from_rgba8(0,0,0,255));
    let (c_turn, severity) = compute_corner_severity_and_direction(data);
    render_tactical_flank_chevrons_or_horizon(pixmap, cx, cy, scale, c_turn, severity);
    render_minimalist_collimated_pod(pixmap, cx, cy, scale, data);
    let tag = "ØG『死』ŦJ乂";
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx, cy + 130.0*scale, tag, Color::from_rgba8(6,182,214,255), scale*0.6, data.timestamp_ms as u64, true);
    } else {
        draw_text(pixmap, cx, cy + 130.0*scale, tag, Color::from_rgba8(6,182,214,255), scale*0.6);
    }
}

fn compute_corner_severity_and_direction(data: &FH6TelemetryData) -> (f32, f32) {
    let v_safe = sanitize(data.speed_ms,0.5,500.0);
    let yaw_rate = sanitize(data.angular_velocity_y, -10.0,10.0);
    let kappa = yaw_rate / v_safe;
    let c_turn = 0.65*(kappa*28.0) + 0.35*data.steer as f32 / 127.0;
    let severity = (c_turn.abs() + 0.45*data.brake as f32/255.0).clamp(0.0,1.0);
    (c_turn, severity)
}

fn render_tactical_flank_chevrons_or_horizon(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, c_turn: f32, severity: f32) {
    if c_turn.abs() < 0.08 {
        stroke_line(pixmap, cx - 200.0*scale, cy, cx + 200.0*scale, cy, 2.0*scale, Color::from_rgba8(6,182,214,255));
    } else {
        let dir_left = c_turn < 0.0;
        let col = if severity < 0.38 { Color::from_rgba8(6,182,214,255) } else if severity < 0.72 { Color::from_rgba8(245,158,11,255) } else { Color::from_rgba8(239,68,68,255) };
        draw_chevron_arrow(pixmap, cx - 120.0*scale, cy - 40.0*scale, 30.0*scale, 20.0*scale, dir_left, 2.0*scale, col, Color::from_rgba8(255,255,255,100));
        draw_chevron_arrow(pixmap, cx - 120.0*scale, cy, 30.0*scale, 20.0*scale, dir_left, 2.0*scale, col, Color::from_rgba8(255,255,255,100));
        draw_chevron_arrow(pixmap, cx - 120.0*scale, cy + 40.0*scale, 30.0*scale, 20.0*scale, dir_left, 2.0*scale, col, Color::from_rgba8(255,255,255,100));
    }
}

fn render_minimalist_collimated_pod(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    fill_rect_xywh(pixmap, cx - 80.0*scale, cy - 40.0*scale, 160.0*scale, 80.0*scale, Color::from_rgba8(0,0,0,200));
    draw_text(pixmap, cx, cy - 10.0*scale, &format!("{:.0}", data.speed_mph()), Color::from_rgba8(255,255,255,255), scale*2.0);
    draw_text(pixmap, cx, cy + 30.0*scale, &data.gear_string(), Color::from_rgba8(255,255,255,255), scale*1.5);
}
