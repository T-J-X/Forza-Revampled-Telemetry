use tiny_skia::{Color, PixmapMut, Rect};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_glitch_text};
use crate::draw_ext::*;

fn sanitize(v: f32, min_v: f32, max_v: f32) -> f32 {
    if !v.is_finite() { min_v } else { v.clamp(min_v, max_v) }
}

fn premultiply(r: u8, g: u8, b: u8, a: u8) -> (u8, u8, u8, u8) {
    if a < 2 { return (0,0,0,0); }
    let a_f = a as f32 / 255.0;
    let r_p = ((r as f32 * a_f).round() as u8).min(r);
    let g_p = ((g as f32 * a_f).round() as u8).min(g);
    let b_p = ((b as f32 * a_f).round() as u8).min(b);
    (r_p, g_p, b_p, a)
}

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    let cx = bounds.x() + bounds.width()*0.5;
    let cy = bounds.y() + bounds.height()*0.5;
    let scale = 1.0;
    // No opaque background
    let rpm_ratio = sanitize(data.normalized_rpm(), 0.0, 1.2);
    let throttle_pct = sanitize(data.throttle_pct(), 0.0, 1.0);
    let v_ms = sanitize(data.speed_ms, 0.0, 100.0);
    let _power_watts = data.power_watts;
    let _pw = _power_watts;
    
    render_action_potential(pixmap, cx, cy, scale, rpm_ratio, throttle_pct, v_ms);
    render_synaptic_gear_node(pixmap, cx, cy, scale, data);
    render_axon_strobe(pixmap, cx, cy, scale, rpm_ratio);
    
    let tag = "ØG『死』ŦJ乂";
    let tr = 0; let tg = 240; let tb = 255; let ta = 255;
    let (pr, pg, pb, pa) = premultiply(tr, tg, tb, ta);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx + 200.0*scale, cy - 150.0*scale, tag, col, scale*0.6, data.timestamp_ms as u64, true);
    } else {
        draw_text(pixmap, cx + 200.0*scale, cy - 150.0*scale, tag, col, scale*0.6);
    }
}

fn render_action_potential(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, rpm_ratio: f32, throttle_pct: f32, v_ms: f32) {
    let amplitude = (12.0 + 35.0 * throttle_pct) * scale;
    let phase_advance = v_ms * 0.15;
    let nodes = 32;
    let mut points = Vec::with_capacity(nodes);
    for i in 0..nodes {
        let t = i as f32 / (nodes as f32 - 1.0);
        let x = cx - 200.0*scale + t * 400.0*scale;
        let y = cy + (t * std::f32::consts::TAU * 2.0 + phase_advance).sin() * amplitude * (1.0 + rpm_ratio);
        points.push((x, y));
    }
    // Cubic Bézier approximation via line segments
    let r = 0; let g = 240; let b = 255; let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    for i in 0..nodes-1 {
        let x1 = points[i].0;
        let y1 = points[i].1;
        let x2 = points[i+1].0;
        let y2 = points[i+1].1;
        for &dx in &[ -1.0, 0.0, 1.0 ] {
            for &dy in &[ -1.0, 0.0, 1.0 ] {
                stroke_line(pixmap, x1+dx, y1+dy, x2+dx, y2+dy, 1.5*scale, col);
            }
        }
    }
}

fn render_synaptic_gear_node(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    // Dendritic node ring around gear number
    let r_ring = 44.0 * scale;
    let r = 56; let g = 189; let b = 248; let a = 130;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    for i in 0..8 {
        let theta = i as f32 * std::f32::consts::TAU / 8.0;
        let x = cx + r_ring * theta.cos();
        let y = cy + r_ring * theta.sin();
        fill_rect_xywh(pixmap, x-2.0, y-2.0, 4.0,4.0, col);
    }
    // Upper-left chamfer highlight
    let rh = 255; let gh = 255; let bh = 255; let ah = 210;
    let (prh, pgh, pbh, pah) = premultiply(rh, gh, bh, ah);
    let col_h = Color::from_rgba8(prh, pgh, pbh, pah);
    stroke_line(pixmap, cx - r_ring, cy - r_ring, cx - r_ring*0.5, cy - r_ring*0.5, 1.5*scale, col_h);
    
    // Live gear string
    let gear = data.gear_string();
    let r_txt = 255; let g_txt = 255; let b_txt = 255; let a_txt = 255;
    let (prt, pgt, pbt, pat) = premultiply(r_txt, g_txt, b_txt, a_txt);
    let col_txt = Color::from_rgba8(prt, pgt, pbt, pat);
    draw_text(pixmap, cx, cy, &gear, col_txt, scale*3.0);
}

fn render_axon_strobe(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, rpm_ratio: f32) {
    if rpm_ratio >= 0.9 {
        let r = 255; let g = 0; let b = 85; let a = 255;
        let (pr, pg, pb, pa) = premultiply(r, g, b, a);
        let col = Color::from_rgba8(pr, pg, pb, pa);
        for i in 0..12 {
            let theta = i as f32 * std::f32::consts::TAU / 12.0;
            let x1 = cx + 60.0*scale * theta.cos();
            let y1 = cy + 60.0*scale * theta.sin();
            let x2 = cx + 120.0*scale * theta.cos();
            let y2 = cy + 120.0*scale * theta.sin();
            stroke_line(pixmap, x1, y1, x2, y2, 2.0*scale, col);
        }
    }
}
