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
    render_titanium_interconnect_brackets(pixmap, cx, cy, scale);
    render_pod1_left_wing_tires_and_g_ballistics(pixmap, cx, cy, scale, data);
    render_pod2_center_hero_parabolic_gt7(pixmap, cx, cy, scale, data);
    render_pod3_right_wing_power_and_delta(pixmap, cx, cy, scale, data);
    let tag = "ØG『死』ŦJ乂";
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx, cy + 140.0*scale, tag, Color::from_rgba8(56,189,248,255), scale*0.6, data.timestamp_ms as u64, true);
    } else {
        draw_text(pixmap, cx, cy + 140.0*scale, tag, Color::from_rgba8(56,189,248,255), scale*0.6);
    }
}

fn render_titanium_interconnect_brackets(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32) {
    stroke_line(pixmap, cx - 180.0*scale, cy, cx - 60.0*scale, cy, 3.0*scale, Color::from_rgba8(39,39,42,255));
    stroke_line(pixmap, cx + 60.0*scale, cy, cx + 180.0*scale, cy, 3.0*scale, Color::from_rgba8(39,39,42,255));
}

fn render_pod1_left_wing_tires_and_g_ballistics(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let pods = [("FL", -240.0), ("FR", -160.0), ("RL", -240.0), ("RR", -160.0)];
    for (tag, off) in pods.iter() {
        let x = cx + off*scale;
        fill_rect_xywh(pixmap, x-40.0*scale, cy - 80.0*scale, 80.0*scale, 60.0*scale, Color::from_rgba8(6,182,214,180));
        draw_text(pixmap, x, cy - 50.0*scale, tag, Color::from_rgba8(255,255,255,255), scale*0.7);
    }
    let g_lat = sanitize(data.lateral_g(),0.0,2.0);
    let g_lon = sanitize(data.longitudinal_g(),0.0,2.0);
    fill_rect_xywh(pixmap, cx - 200.0*scale, cy - 20.0*scale, 120.0*scale,120.0*scale, Color::from_rgba8(0,0,0,150));
    stroke_line(pixmap, cx-200.0*scale, cy, cx+200.0*scale, cy, 1.0*scale, Color::from_rgba8(255,255,255,150));
    stroke_line(pixmap, cx, cy-200.0*scale, cx, cy+200.0*scale, 1.0*scale, Color::from_rgba8(255,255,255,150));
    fill_rect_xywh(pixmap, cx + g_lat*40.0*scale - 3.0, cy + g_lon*40.0*scale - 3.0,6.0,6.0, Color::from_rgba8(250,204,21,255));
}

fn render_pod2_center_hero_parabolic_gt7(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    fill_rect_xywh(pixmap, cx - 100.0*scale, cy - 60.0*scale, 200.0*scale,120.0*scale, Color::from_rgba8(0,0,0,200));
    stroke_rect_xywh(pixmap, cx - 100.0*scale, cy - 60.0*scale, 200.0*scale,120.0*scale, 1.0*scale, Color::from_rgba8(255,255,255,120));
    draw_text(pixmap, cx, cy - 20.0*scale, &data.gear_string(), Color::from_rgba8(255,255,255,255), scale*2.5);
    draw_text(pixmap, cx, cy + 30.0*scale, &format!("{:.0}", data.speed_mph()), Color::from_rgba8(255,255,255,255), scale*1.8);
    for i in 0..16 {
        let y = cy - 40.0*scale + i as f32 * 4.0*scale;
        let col = if i < 8 { Color::from_rgba8(16,185,129,255) } else if i < 12 { Color::from_rgba8(250,204,21,255) } else { Color::from_rgba8(239,68,68,255) };
        fill_rect_xywh(pixmap, cx + 110.0*scale, y, 8.0*scale, 4.0*scale, col);
    }
}

fn render_pod3_right_wing_power_and_delta(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let hp = (data.power_watts / 745.7).max(0.0);
    draw_text(pixmap, cx + 200.0*scale, cy - 40.0*scale, &format!("{:.0} HP", hp), Color::from_rgba8(255,255,255,255), scale*1.0);
    fill_rect_xywh(pixmap, cx + 140.0*scale, cy - 20.0*scale, 120.0*scale, 10.0*scale, Color::from_rgba8(60,60,60,255));
    fill_rect_xywh(pixmap, cx + 140.0*scale, cy - 20.0*scale, 120.0*scale * 0.6, 10.0*scale, Color::from_rgba8(59,130,246,255));
    draw_text(pixmap, cx + 200.0*scale, cy + 20.0*scale, "-0.123s", Color::from_rgba8(34,197,94,255), scale*0.9);
}
