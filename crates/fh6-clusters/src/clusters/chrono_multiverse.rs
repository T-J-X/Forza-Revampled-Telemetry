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
    let rpm_ratio = sanitize(data.normalized_rpm(),0.0,1.2);
    render_spacetime_polar_grid(pixmap, cx, cy, scale);
    render_procedural_ferrofluid_ring(pixmap, cx, cy, scale, rpm_ratio, data);
    render_orbital_rings_and_tag(pixmap, cx, cy, scale, data, rpm_ratio);
    let tag = "ØG『死』ŦJ乂";
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx, cy + 120.0*scale, tag, Color::from_rgba8(0,229,255,255), scale*0.7, data.timestamp_ms as u64, true);
    } else {
        draw_text(pixmap, cx, cy + 120.0*scale, tag, Color::from_rgba8(0,229,255,255), scale*0.7);
    }
}

fn render_spacetime_polar_grid(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32) {
    for i in 0..12 {
        let a = i as f32 * std::f32::consts::PI / 6.0;
        stroke_line(pixmap, cx, cy, cx + 150.0*scale*a.cos(), cy + 150.0*scale*a.sin(), 0.8*scale, Color::from_rgba8(0,229,255,80));
    }
    fill_rect_xywh(pixmap, cx-64.0*scale, cy-64.0*scale,128.0*scale,128.0*scale, Color::from_rgba8(0,0,0,255));
}

fn render_procedural_ferrofluid_ring(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, rpm_ratio: f32, _data: &FH6TelemetryData) {
    let base_r = 80.0*scale;
    for i in 0..180 {
        let theta = i as f32 * std::f32::consts::PI / 90.0;
        let k = 6.0 + rpm_ratio*12.0;
        let r = base_r + (theta * k).sin() * 10.0*scale * rpm_ratio;
        let x = cx + r * theta.cos();
        let y = cy + r * theta.sin();
        fill_rect_xywh(pixmap, x-1.0, y-1.0,2.0,2.0, Color::from_rgba8(0,229,255,200));
    }
}

fn render_orbital_rings_and_tag(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData, rpm_ratio: f32) {
    for r in [60.0,90.0,120.0] {
        stroke_line(pixmap, cx, cy, cx+r*scale, cy, 1.0*scale, Color::from_rgba8(0,255,102,120));
    }
    draw_text(pixmap, cx, cy - 44.0*scale, &data.gear_string(), Color::from_rgba8(255,184,0,255), scale*2.0);
    draw_text(pixmap, cx, cy + 6.0*scale, &format!("{:.0}", data.speed_mph()), Color::from_rgba8(248,252,255,255), scale*1.8);
    if rpm_ratio >= 0.96 {
        fill_rect_xywh(pixmap, cx-62.0*scale, cy-55.0*scale,124.0*scale,112.0*scale, Color::from_rgba8(255,30,56,60));
    }
}
