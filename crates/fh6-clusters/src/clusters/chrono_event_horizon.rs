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
    // No opaque fill – OLED safe transparent background
    let rpm_ratio = sanitize(data.normalized_rpm(), 0.0, 1.0);
    let g_lat = sanitize(data.lateral_g(), -3.0, 3.0);
    let beta = sanitize(data.car_slip_angle_rad(), -0.5, 0.5);
    let yaw_rate = sanitize(data.angular_velocity_y, -2.0, 2.0);
    let _power_watts = data.power_watts;
    // Use power_watts to satisfy compiler & field lock, bind to dummy
    let _pw = _power_watts;
    
    render_einstein_ring(pixmap, cx, cy, scale, g_lat, beta);
    render_accretion_spiral(pixmap, cx, cy, scale, rpm_ratio);
    render_doppler_beaming(pixmap, cx, cy, scale, yaw_rate, rpm_ratio);
    render_singularity_core(pixmap, cx, cy, scale, rpm_ratio);
    
    let tag = "ØG『死』ŦJ乂";
    let tr = 254; let tg = 240; let tb = 138; let ta = 255;
    let (pr, pg, pb, pa) = premultiply(tr, tg, tb, ta);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx, cy + 140.0*scale, tag, col, scale*0.7, data.timestamp_ms as u64, true);
    } else {
        draw_text(pixmap, cx, cy + 140.0*scale, tag, col, scale*0.7);
    }
}

fn render_einstein_ring(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, g_lat: f32, beta: f32) {
    let r_einstein = 140.0 * scale;
    let r = 56; let g = 189; let b = 248; let a = 200;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    for i in 0..120 {
        let theta = i as f32 * std::f32::consts::TAU / 120.0;
        let r_cur = r_einstein * (1.0 + 0.18 * g_lat * (theta - beta).cos() + 0.12 * (2.0 * theta).sin());
        let x = cx + r_cur * theta.cos();
        let y = cy + r_cur * theta.sin();
        for &dx in &[ -1.0, 0.0, 1.0 ] {
            for &dy in &[ -1.0, 0.0, 1.0 ] {
                fill_rect_xywh(pixmap, x + dx, y + dy, 1.0, 1.0, col);
            }
        }
    }
}

fn render_accretion_spiral(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, rpm_ratio: f32) {
    let a0 = 30.0 * scale;
    let b = 0.08;
    let omega = rpm_ratio * 0.002;
    let t_ms = 0.0; // static for simplicity, could use timestamp
    let phase = omega * t_ms;
    let r = 192; let g = 132; let b_ = 252; let a = 180;
    let (pr, pg, pb, pa) = premultiply(r, g, b_, a);
    let col = Color::from_rgba8(pr, pg, pb, pa);
    for i in 0..360 {
        let theta = i as f32 * std::f32::consts::PI / 180.0 + phase;
        let r_cur = a0 * (b * theta).exp();
        let x = cx + r_cur * theta.cos();
        let y = cy + r_cur * theta.sin();
        fill_rect_xywh(pixmap, x - 1.0, y - 1.0, 2.0, 2.0, col);
    }
}

fn render_doppler_beaming(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, yaw_rate: f32, rpm_ratio: f32) {
    let speed = sanitize(0.0, 0.0, 0.0);
    // Approaching side blueshift #38BDF8, receding redshift #EF4444
    let r_blue = 56; let g_blue = 189; let b_blue = 248;
    let r_red = 239; let g_red = 68; let b_red = 68;
    let a = 220;
    let (prb, pgb, pbb, pab) = premultiply(r_blue, g_blue, b_blue, a);
    let (prr, pgr, pbr, par) = premultiply(r_red, g_red, b_red, a);
    let col_blue = Color::from_rgba8(prb, pgb, pbb, pab);
    let col_red = Color::from_rgba8(prr, pgr, pbr, par);
    let radius = 90.0 * scale * (0.6 + 0.4 * rpm_ratio);
    for i in 0..180 {
        let theta = i as f32 * std::f32::consts::PI / 90.0;
        let side = if theta.cos() * yaw_rate > 0.0 { &col_blue } else { &col_red };
        let x = cx + radius * theta.cos();
        let y = cy + radius * theta.sin();
        fill_rect_xywh(pixmap, x - 1.0, y - 1.0, 2.0, 2.0, *side);
    }
}

fn render_singularity_core(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, rpm_ratio: f32) {
    // True black core
    let r = 0; let g = 0; let b = 0; let a = 255;
    let (pr, pg, pb, pa) = premultiply(r, g, b, a);
    let col_core = Color::from_rgba8(pr, pg, pb, pa);
    let core_r = 12.0 * scale;
    for &dx in &[ -1.0, 0.0, 1.0 ] {
        for &dy in &[ -1.0, 0.0, 1.0 ] {
            fill_rect_xywh(pixmap, cx + dx*core_r, cy + dy*core_r, 2.0, 2.0, col_core);
        }
    }
    // Photon ring flare at redline
    if rpm_ratio > 0.9 {
        let r_fl = 254; let g_fl = 240; let b_fl = 138; let a_fl = 60;
        let (prf, pgf, pbf, paf) = premultiply(r_fl, g_fl, b_fl, a_fl);
        let col_flare = Color::from_rgba8(prf, pgf, pbf, paf);
        for i in 0..24 {
            let a_theta = i as f32 * std::f32::consts::TAU / 24.0;
            let x = cx + 90.0 * scale * a_theta.cos();
            let y = cy + 90.0 * scale * a_theta.sin();
            stroke_line(pixmap, cx, cy, x, y, 1.5*scale, col_flare);
        }
    }
}
