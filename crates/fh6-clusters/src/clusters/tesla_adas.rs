use tiny_skia::{Color, PixmapMut, Rect};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_glitch_text};
use crate::draw_ext::{fill_rect_xywh, stroke_line, stroke_polyline, fill_polygon, fill_round_rect, sanitize_finite};
use std::sync::OnceLock;
use std::collections::HashMap;

static CAR_BRANDS_CACHE: OnceLock<HashMap<i32, (u8, u8, u8)>> = OnceLock::new();
static ADAS_SPRITE_CACHE: OnceLock<HashMap<String, Vec<u8>>> = OnceLock::new();

#[inline]
fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba8(r, g, b, a)
}

#[inline]
fn clamp_byte(v: f32) -> u8 {
    if !v.is_finite() {
        return 0;
    }
    let i = v.round() as i32;
    if i < 0 { 0 } else if i > 255 { 255 } else { i as u8 }
}

fn project_to_adas_viewport(
    x_m: f32,
    y_m: f32,
    z_m: f32,
    cx: f32,
    _cy: f32,
    fx: f32,
    fy: f32,
    y_horizon: f32,
) -> (f32, f32) {
    let zc = (z_m + 3.4).max(0.1);
    let sx = cx + (fx * x_m) / zc;
    let sy = y_horizon + (fy * (1.85 - y_m)) / zc;
    (sx, sy)
}

fn fill_rect(p: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, c: Color) {
    fill_rect_xywh(p, x, y, w, h, c);
}

fn load_car_brands() -> &'static HashMap<i32, (u8, u8, u8)> {
    CAR_BRANDS_CACHE.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(0, (236, 242, 249));
        m.insert(1, (94, 114, 138));
        m.insert(2, (76, 94, 118));
        m
    })
}

fn load_adas_sprites() {
    let _ = ADAS_SPRITE_CACHE.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("adas_ego_coupe.png".to_string(), vec![]);
        m.insert("adas_traffic_sedan.png".to_string(), vec![]);
        m.insert("adas_traffic_suv.png".to_string(), vec![]);
        m
    });
}

fn draw_driver_tag(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, glitch: bool) {
    let tag = "ØG『死』ŦJ乂";
    if glitch {
        draw_glitch_text(pixmap, cx - 130.0 * scale, cy - 240.0 * scale, tag, rgba(255, 255, 255, 255), scale, 0, true);
    } else {
        draw_text(pixmap, cx - 130.0 * scale, cy - 240.0 * scale, tag, rgba(255, 255, 255, 255), scale);
    }
}

fn render_stage_backdrop(pixmap: &mut PixmapMut, cx: f32, stage_cy: f32, scale: f32) {
    let stage_w = 648.0 * scale;
    let stage_h = 268.0 * scale;
    fill_rect(pixmap, cx - stage_w * 0.5, stage_cy - stage_h * 0.5, stage_w, stage_h, rgba(6, 11, 22, 235));
}

fn render_adas_road_mesh(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    scale: f32,
    fx: f32,
    fy: f32,
    y_horizon: f32,
    kappa: f32,
    tan_beta: f32,
) {
    const SLICES: usize = 18;
    let mut left_pts = Vec::with_capacity(SLICES);
    let mut right_pts = Vec::with_capacity(SLICES);
    for i in 0..SLICES {
        let t = i as f32 / (SLICES as f32 - 1.0);
        let z = 1.8 + (44.0 - 1.8) * t.powf(1.25);
        let xc = 0.5 * kappa * z * z + tan_beta * z * 0.14;
        let (lx, ly) = project_to_adas_viewport(xc - 5.45, 0.0, z, cx, cy, fx, fy, y_horizon);
        let (rx, ry) = project_to_adas_viewport(xc + 5.45, 0.0, z, cx, cy, fx, fy, y_horizon);
        left_pts.push((lx, ly));
        right_pts.push((rx, ry));
    }
    for i in 0..SLICES - 1 {
        stroke_line(pixmap, left_pts[i].0, left_pts[i].1, left_pts[i + 1].0, left_pts[i + 1].1, 1.2 * scale, rgba(148, 210, 255, 185));
        stroke_line(pixmap, right_pts[i].0, right_pts[i].1, right_pts[i + 1].0, right_pts[i + 1].1, 1.2 * scale, rgba(148, 210, 255, 185));
    }
    for i in 0..SLICES - 1 {
        let t = i as f32 / (SLICES as f32 - 1.0);
        let z = 1.8 + (44.0 - 1.8) * t.powf(1.25);
        let xc = 0.5 * kappa * z * z + tan_beta * z * 0.14;
        let (lx, ly) = project_to_adas_viewport(xc - 1.82, 0.0, z, cx, cy, fx, fy, y_horizon);
        let (rx, ry) = project_to_adas_viewport(xc + 1.82, 0.0, z, cx, cy, fx, fy, y_horizon);
        let next_z = 1.8 + (44.0 - 1.8) * (((i + 1) as f32) / (SLICES as f32 - 1.0)).powf(1.25);
        let next_xc = 0.5 * kappa * next_z * next_z + tan_beta * next_z * 0.14;
        let (nlx, nly) = project_to_adas_viewport(next_xc - 1.82, 0.0, next_z, cx, cy, fx, fy, y_horizon);
        let (nrx, nry) = project_to_adas_viewport(next_xc + 1.82, 0.0, next_z, cx, cy, fx, fy, y_horizon);
        stroke_line(pixmap, lx, ly, nlx, nly, 1.25 * scale, rgba(56, 189, 248, 82));
        stroke_line(pixmap, rx, ry, nrx, nry, 1.25 * scale, rgba(56, 189, 248, 82));
    }
}

fn render_dashed_lanes(
    pixmap: &mut PixmapMut,
    cx: f32,
    _cy: f32,
    _scale: f32,
    fx: f32,
    fy: f32,
    y_horizon: f32,
    kappa: f32,
    tan_beta: f32,
    speed_ms: f32,
) {
    let phase = (speed_ms * 0.016).fract();
    for m in -1..8 {
        let z_base = m as f32 * 6.0 - phase;
        if z_base + 2.35 <= 1.85 || z_base >= 42.5 {
            continue;
        }
        let z_start = z_base.max(1.85);
        let z_end = (z_base + 2.35).min(43.5);
        if z_end - z_start < 0.25 {
            continue;
        }
        let xc_start = 0.5 * kappa * z_start * z_start + tan_beta * z_start * 0.14;
        let xc_end = 0.5 * kappa * z_end * z_end + tan_beta * z_end * 0.14;
        let half_w = 0.065;
        let d_alpha = clamp_byte(235.0 * (1.0 - (z_start - 1.85) / 41.5));
        let pts = [
            (cx + (fx * (xc_start - 1.82 - half_w)) / (z_start + 3.4), y_horizon + (fy * 1.85) / (z_start + 3.4)),
            (cx + (fx * (xc_start - 1.82 + half_w)) / (z_start + 3.4), y_horizon + (fy * 1.85) / (z_start + 3.4)),
            (cx + (fx * (xc_end - 1.82 + half_w)) / (z_end + 3.4), y_horizon + (fy * 1.85) / (z_end + 3.4)),
            (cx + (fx * (xc_end - 1.82 - half_w)) / (z_end + 3.4), y_horizon + (fy * 1.85) / (z_end + 3.4)),
        ];
        fill_polygon(pixmap, &pts, rgba(226, 238, 252, d_alpha));
        let pts2 = [
            (cx + (fx * (xc_start + 1.82 - half_w)) / (z_start + 3.4), y_horizon + (fy * 1.85) / (z_start + 3.4)),
            (cx + (fx * (xc_start + 1.82 + half_w)) / (z_start + 3.4), y_horizon + (fy * 1.85) / (z_start + 3.4)),
            (cx + (fx * (xc_end + 1.82 + half_w)) / (z_end + 3.4), y_horizon + (fy * 1.85) / (z_end + 3.4)),
            (cx + (fx * (xc_end + 1.82 - half_w)) / (z_end + 3.4), y_horizon + (fy * 1.85) / (z_end + 3.4)),
        ];
        fill_polygon(pixmap, &pts2, rgba(226, 238, 252, d_alpha));
    }
}

fn render_carpet(
    pixmap: &mut PixmapMut,
    cx: f32,
    scale: f32,
    fx: f32,
    fy: f32,
    y_horizon: f32,
    kappa: f32,
    tan_beta: f32,
    z_lead: f32,
    brake: f32,
) {
    let z_start = 4.55;
    let z_end = (z_lead - 1.35).max(z_start + 3.5).min(24.5);
    const SEG: usize = 10;
    let mut left = Vec::with_capacity(SEG + 1);
    let mut right = Vec::with_capacity(SEG + 1);
    for s in 0..=SEG {
        let st = s as f32 / SEG as f32;
        let cz = z_start + (z_end - z_start) * st;
        let cxc = 0.5 * kappa * cz * cz + tan_beta * cz * 0.14;
        let half = 1.18 - 0.16 * st;
        let (lx, ly) = project_to_adas_viewport(cxc - half, 0.0, cz, cx, 0.0, fx, fy, y_horizon);
        let (rx, ry) = project_to_adas_viewport(cxc + half, 0.0, cz, cx, 0.0, fx, fy, y_horizon);
        left.push((lx, ly));
        right.push((rx, ry));
    }
    let c_r = clamp_byte(14.0 + (255.0 - 14.0) * brake);
    let c_g = clamp_byte(205.0 * (1.0 - 0.58 * brake));
    let c_b = clamp_byte(255.0 * (1.0 - 0.82 * brake));
    for s in (0..SEG).rev() {
        let st = s as f32 / SEG as f32;
        let alpha = clamp_byte((1.0 - st * 0.78) * (if brake > 0.05 { 84.0 } else { 58.0 }));
        let pts = [left[s], right[s], right[s + 1], left[s + 1]];
        fill_polygon(pixmap, &pts, rgba(c_r, c_g, c_b, alpha));
    }
    stroke_polyline(pixmap, &left, 1.6 * scale, rgba(c_r, c_g, c_b, 255), false);
    stroke_polyline(pixmap, &right, 1.6 * scale, rgba(c_r, c_g, c_b, 255), false);
}

fn render_ego_vehicle(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    scale: f32,
    fx: f32,
    fy: f32,
    y_horizon: f32,
    ego_x: f32,
    ego_z: f32,
    brake: f32,
    brand: (u8, u8, u8),
) {
    let (sx, sy) = project_to_adas_viewport(ego_x, 0.0, ego_z, cx, cy, fx, fy, y_horizon);
    let w = 36.0 * scale;
    let h = 18.0 * scale;
    fill_rect(pixmap, sx - w / 2.0, sy - h / 2.0, w, h, rgba(brand.0, brand.1, brand.2, 255));
    if brake > 0.05 {
        fill_rect(pixmap, sx - 8.0 * scale, sy + h / 2.0 - 3.0 * scale, 16.0 * scale, 4.0 * scale, rgba(255, 23, 68, 255));
        fill_rect(pixmap, sx - 40.0 * scale, sy + 40.0 * scale, 80.0 * scale, 20.0 * scale, rgba(255, 23, 68, 60));
    }
}

fn render_traffic_fleet(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    scale: f32,
    fx: f32,
    fy: f32,
    y_horizon: f32,
    kappa: f32,
    tan_beta: f32,
    z_lead: f32,
    anim_time: f32,
) {
    let ego_z = 3.2;
    let ego_x = 0.5 * kappa * ego_z * ego_z + tan_beta * ego_z * 0.14;
    let t1 = (anim_time * 7.2) % 30.0;
    let z1 = 4.2 + t1;
    let z2 = 5.6 + 12.5 * (0.5 + 0.5 * (anim_time * 0.28 + 1.15).sin());
    let vehicles = [
        (ego_x, ego_z, 0),
        (0.5 * kappa * z_lead * z_lead + tan_beta * z_lead * 0.14, z_lead, 1),
        (0.5 * kappa * z1 * z1 + tan_beta * z1 * 0.14 - 3.64, z1, 1),
        (0.5 * kappa * z2 * z2 + tan_beta * z2 * 0.14 + 3.64, z2, 2),
    ];
    for (x, z, typ) in vehicles {
        let (sx, sy) = project_to_adas_viewport(x, 0.0, z, cx, cy, fx, fy, y_horizon);
        let col = match typ {
            0 => (236, 242, 249),
            1 => (112, 130, 152),
            2 => (74, 92, 114),
            _ => (255, 255, 255),
        };
        fill_rect(pixmap, sx - 18.0 * scale, sy - 10.0 * scale, 36.0 * scale, 20.0 * scale, rgba(col.0, col.1, col.2, 200));
    }
}

fn render_ultrasonic_radar_arcs(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    scale: f32,
    fx: f32,
    fy: f32,
    y_horizon: f32,
    ego_x: f32,
    ego_z: f32,
    _beta: f32,
) {
    let arc_center_z = ego_z + 1.35;
    let left_center_x = ego_x - 1.02;
    let right_center_x = ego_x + 1.02;
    for &center_x in &[left_center_x, right_center_x] {
        let col = rgba(56, 189, 248, 120);
        for t in 0..3 {
            let r = 1.15 + t as f32 * 0.62;
            let mut pts = Vec::with_capacity(9);
            for p in 0..9 {
                let ang = -38.0 + 76.0 * p as f32 / 8.0;
                let rad = ang.to_radians();
                let px = center_x - r * rad.cos();
                let pz = arc_center_z + r * rad.sin() * 1.25;
                let (sx, sy) = project_to_adas_viewport(px, 0.06, pz, cx, cy, fx, fy, y_horizon);
                pts.push((sx, sy));
            }
            stroke_polyline(pixmap, &pts, 1.0 * scale, col, true);
        }
    }
}

fn render_overhead_telemetry(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, _speed_ms: f32, _gear: &str) {
    fill_rect(pixmap, cx - 80.0 * scale, cy - 40.0 * scale, 160.0 * scale, 30.0 * scale, rgba(6, 12, 24, 175));
    fill_rect(pixmap, cx + 80.0 * scale, cy + 10.0 * scale, 50.0 * scale, 24.0 * scale, rgba(56, 189, 248, 120));
    fill_round_rect(pixmap, cx - 116.0 * scale, cy - 174.0 * scale, 232.0 * scale, 114.0 * scale, 22.0 * scale, rgba(6, 12, 24, 175));
    fill_rect(pixmap, cx - 88.0 * scale, cy - 142.0 * scale, 176.0 * scale, 50.0 * scale, rgba(0, 0, 0, 0));
}

fn render_fsd_brackets(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32) {
    let w = 60.0 * scale;
    let h = 30.0 * scale;
    fill_rect(pixmap, cx - w / 2.0, cy - h / 2.0, w, h, rgba(255, 255, 0, 180));
}

fn sanitize_all(data: &FH6TelemetryData) -> (f32, f32, f32, f32, f32) {
    let speed = sanitize_finite(data.speed_ms, 0.0, 0.0, 250.0);
    let kappa = sanitize_finite(data.yaw_curvature(), 0.0, -0.055, 0.055);
    let beta = sanitize_finite(data.car_slip_angle_rad(), 0.0, -0.7854, 0.7854);
    let brake = sanitize_finite(data.brake_pct(), 0.0, 0.0, 1.0);
    let rpm = sanitize_finite(data.normalized_rpm(), 0.0, 0.0, 1.2);
    (speed, kappa, beta, brake, rpm)
}

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    load_adas_sprites();
    let brands = load_car_brands();
    let cx = bounds.x() + bounds.width() / 2.0;
    let cy = bounds.y() + bounds.height() / 2.0;
    let scale = 1.0;

    fill_rect(pixmap, bounds.x(), bounds.y(), bounds.width(), bounds.height(), rgba(6, 8, 12, 255));

    let (speed_ms, kappa, beta, brake_pct, _rpm) = sanitize_all(data);
    let tan_beta = beta.tan();
    let fx = 320.0 * scale;
    let fy = 450.0 * scale;
    let y_horizon = cy - 24.0 * scale;
    let stage_cy = cy + 44.0 * scale;

    render_stage_backdrop(pixmap, cx, stage_cy, scale);
    render_adas_road_mesh(pixmap, cx, cy, scale, fx, fy, y_horizon, kappa, tan_beta);
    render_dashed_lanes(pixmap, cx, cy, scale, fx, fy, y_horizon, kappa, tan_beta, speed_ms);

    let ego_z = 3.2;
    let ego_x = 0.5 * kappa * ego_z * ego_z + tan_beta * ego_z * 0.14;
    let brand_col = brands.get(&data.car_ordinal).copied().unwrap_or((236, 242, 249));
    render_ego_vehicle(pixmap, cx, cy, scale, fx, fy, y_horizon, ego_x, ego_z, brake_pct, brand_col);

    let z_lead = 17.5 + 5.8 * (0.0f32).sin();
    render_carpet(pixmap, cx, scale, fx, fy, y_horizon, kappa, tan_beta, z_lead, brake_pct);
    render_traffic_fleet(pixmap, cx, cy, scale, fx, fy, y_horizon, kappa, tan_beta, z_lead, 0.0);
    render_ultrasonic_radar_arcs(pixmap, cx, cy, scale, fx, fy, y_horizon, ego_x, ego_z, beta);
    render_fsd_brackets(pixmap, cx, cy, scale);
    render_overhead_telemetry(pixmap, cx, cy, scale, speed_ms, &data.gear_string());
    draw_driver_tag(pixmap, cx, cy, scale, config.driver_tag_glitch_fx);
}
