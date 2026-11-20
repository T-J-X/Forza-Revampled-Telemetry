//! draw_ext: Reusable, ergonomic tiny-skia drawing primitives bridge.
//! Provides high-level drawing helpers so cluster geometry code stays short and readable.

use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

#[inline]
pub fn sanitize_finite(v: f32, fallback: f32, min_v: f32, max_v: f32) -> f32 {
    if !v.is_finite() {
        fallback
    } else if v < min_v {
        min_v
    } else if v > max_v {
        max_v
    } else {
        v
    }
}

#[inline]
pub fn rect_center(rect: &Rect) -> (f32, f32) {
    (rect.x() + rect.width() / 2.0, rect.y() + rect.height() / 2.0)
}

pub fn stroke_line(
    pixmap: &mut PixmapMut,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    width: f32,
    color: Color,
) {
    let mut pb = PathBuilder::new();
    pb.move_to(x0, y0);
    pb.line_to(x1, y1);
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(color);
        let stroke = Stroke {
            width,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

pub fn stroke_polyline(
    pixmap: &mut PixmapMut,
    pts: &[(f32, f32)],
    width: f32,
    color: Color,
    closed: bool,
) {
    if pts.len() < 2 {
        return;
    }
    let mut pb = PathBuilder::new();
    pb.move_to(pts[0].0, pts[0].1);
    for pt in &pts[1..] {
        pb.line_to(pt.0, pt.1);
    }
    if closed {
        pb.close();
    }
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(color);
        let stroke = Stroke {
            width,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

pub fn fill_polygon(pixmap: &mut PixmapMut, pts: &[(f32, f32)], color: Color) {
    if pts.len() < 3 {
        return;
    }
    let mut pb = PathBuilder::new();
    pb.move_to(pts[0].0, pts[0].1);
    for pt in &pts[1..] {
        pb.line_to(pt.0, pt.1);
    }
    pb.close();
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(color);
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
}

pub fn fill_rect_xywh(pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, color: Color) {
    if let Some(rect) = Rect::from_xywh(x, y, w, h) {
        let mut paint = Paint::default();
        paint.set_color(color);
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

pub fn stroke_rect_xywh(
    pixmap: &mut PixmapMut,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    width: f32,
    color: Color,
) {
    let mut pb = PathBuilder::new();
    pb.move_to(x, y);
    pb.line_to(x + w, y);
    pb.line_to(x + w, y + h);
    pb.line_to(x, y + h);
    pb.close();
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(color);
        let stroke = Stroke {
            width,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

pub fn fill_round_rect(
    pixmap: &mut PixmapMut,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    color: Color,
) {
    let r = radius.min(w / 2.0).min(h / 2.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(color);
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
}

pub fn stroke_round_rect(
    pixmap: &mut PixmapMut,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    width: f32,
    color: Color,
) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = radius.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;
        let mut stroke = Stroke::default();
        stroke.width = width;
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

pub fn stroke_circle(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    r: f32,
    width: f32,
    color: Color,
) {
    stroke_arc_deg(pixmap, cx, cy, r, r, 0.0, 360.0, width, color);
}

pub fn stroke_arc_deg(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    start_deg: f32,
    sweep_deg: f32,
    width: f32,
    color: Color,
) {
    let steps = 40;
    let mut pts = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let ang = (start_deg + t * sweep_deg).to_radians();
        pts.push((cx + rx * ang.cos(), cy + ry * ang.sin()));
    }
    stroke_polyline(pixmap, &pts, width, color, false);
}

pub fn draw_hex_bolt(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    r: f32,
    rotation_rad: f32,
    metal_rgba: Color,
) {
    let mut pts = Vec::with_capacity(6);
    for i in 0..6 {
        let ang = rotation_rad + (i as f32 * std::f32::consts::PI / 3.0);
        pts.push((cx + r * ang.cos(), cy + r * ang.sin()));
    }
    fill_polygon(pixmap, &pts, metal_rgba);
    stroke_polyline(pixmap, &pts, 1.2, Color::from_rgba8(255, 255, 255, 80), true);
    // Inner socket
    fill_round_rect(pixmap, cx - r * 0.35, cy - r * 0.35, r * 0.7, r * 0.7, 1.0, Color::from_rgba8(10, 10, 15, 200));
}

pub fn draw_ruby_jewel_bearing(pixmap: &mut PixmapMut, cx: f32, cy: f32, r: f32) {
    fill_round_rect(pixmap, cx - r, cy - r, r * 2.0, r * 2.0, r, Color::from_rgba8(224, 17, 95, 255));
    fill_round_rect(pixmap, cx - r * 0.6, cy - r * 0.6, r * 1.2, r * 1.2, r * 0.6, Color::from_rgba8(120, 8, 50, 255));
    // Specular pinpoint
    fill_round_rect(pixmap, cx - r * 0.3, cy - r * 0.4, r * 0.35, r * 0.35, r * 0.17, Color::from_rgba8(255, 255, 255, 200));
}

pub fn draw_gear_wheel(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    outer_r: f32,
    root_r: f32,
    num_teeth: usize,
    phase_rad: f32,
    fill_rgba: Color,
    stroke_rgba: Color,
) {
    if num_teeth < 4 {
        return;
    }
    let mut pts = Vec::with_capacity(num_teeth * 4);
    let step = std::f32::consts::TAU / num_teeth as f32;
    for i in 0..num_teeth {
        let base_ang = phase_rad + i as f32 * step;
        let a0 = base_ang;
        let a1 = base_ang + step * 0.3;
        let a2 = base_ang + step * 0.5;
        let a3 = base_ang + step * 0.8;

        pts.push((cx + root_r * a0.cos(), cy + root_r * a0.sin()));
        pts.push((cx + outer_r * a1.cos(), cy + outer_r * a1.sin()));
        pts.push((cx + outer_r * a2.cos(), cy + outer_r * a2.sin()));
        pts.push((cx + root_r * a3.cos(), cy + root_r * a3.sin()));
    }
    fill_polygon(pixmap, &pts, fill_rgba);
    stroke_polyline(pixmap, &pts, 1.5, stroke_rgba, true);
    // Center hole
    fill_round_rect(pixmap, cx - root_r * 0.4, cy - root_r * 0.4, root_r * 0.8, root_r * 0.8, root_r * 0.4, Color::from_rgba8(6, 8, 12, 255));
}

pub fn draw_guilloche_ring(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    r_min: f32,
    r_max: f32,
    num_rings: usize,
    wave_freq: f32,
    wave_amp: f32,
    rgba: Color,
) {
    if num_rings == 0 {
        return;
    }
    let steps = 120;
    for ring in 0..num_rings {
        let r_base = r_min + (r_max - r_min) * (ring as f32 / num_rings as f32);
        let mut pts = Vec::with_capacity(steps + 1);
        let phase = ring as f32 * 0.4;
        for i in 0..=steps {
            let theta = i as f32 * std::f32::consts::TAU / steps as f32;
            let r = r_base + wave_amp * (theta * wave_freq + phase).sin();
            pts.push((cx + r * theta.cos(), cy + r * theta.sin()));
        }
        stroke_polyline(pixmap, &pts, 1.0, rgba, true);
    }
}

pub fn draw_tick_ring(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    r_outer: f32,
    r_inner: f32,
    start_deg: f32,
    sweep_deg: f32,
    count: usize,
    major_every: usize,
    width_minor: f32,
    width_major: f32,
    rgba: Color,
) {
    if count == 0 {
        return;
    }
    let step_deg = if count > 1 {
        sweep_deg / (count - 1) as f32
    } else {
        0.0
    };
    for i in 0..count {
        let ang = (start_deg + i as f32 * step_deg).to_radians();
        let is_major = major_every > 0 && (i % major_every == 0);
        let w = if is_major { width_major } else { width_minor };
        let r_in = if is_major { r_inner - 3.0 } else { r_inner };
        let cos_a = ang.cos();
        let sin_a = ang.sin();
        stroke_line(
            pixmap,
            cx + r_in * cos_a,
            cy + r_in * sin_a,
            cx + r_outer * cos_a,
            cy + r_outer * sin_a,
            w,
            rgba,
        );
    }
}

pub fn draw_tapered_needle(
    pixmap: &mut PixmapMut,
    cx: f32,
    cy: f32,
    angle_rad: f32,
    length: f32,
    tail_len: f32,
    base_w: f32,
    tip_w: f32,
    fill_rgba: Color,
    glow_rgba: Option<Color>,
) {
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    let perp_cos = -sin_a;
    let perp_sin = cos_a;

    let tip_x = cx + length * cos_a;
    let tip_y = cy + length * sin_a;

    let tail_x = cx - tail_len * cos_a;
    let tail_y = cy - tail_len * sin_a;

    let left_base_x = cx + perp_cos * (base_w / 2.0);
    let left_base_y = cy + perp_sin * (base_w / 2.0);

    let right_base_x = cx - perp_cos * (base_w / 2.0);
    let right_base_y = cy - perp_sin * (base_w / 2.0);

    let left_tip_x = tip_x + perp_cos * (tip_w / 2.0);
    let left_tip_y = tip_y + perp_sin * (tip_w / 2.0);

    let right_tip_x = tip_x - perp_cos * (tip_w / 2.0);
    let right_tip_y = tip_y - perp_sin * (tip_w / 2.0);

    let pts = [
        (tail_x, tail_y),
        (left_base_x, left_base_y),
        (left_tip_x, left_tip_y),
        (right_tip_x, right_tip_y),
        (right_base_x, right_base_y),
    ];

    if let Some(glow) = glow_rgba {
        stroke_polyline(pixmap, &pts, 3.0, glow, true);
    }
    fill_polygon(pixmap, &pts, fill_rgba);
    // Hub cap
    fill_round_rect(
        pixmap,
        cx - base_w,
        cy - base_w,
        base_w * 2.0,
        base_w * 2.0,
        base_w,
        Color::from_rgba8(20, 22, 28, 255),
    );
    fill_round_rect(
        pixmap,
        cx - base_w * 0.5,
        cy - base_w * 0.5,
        base_w,
        base_w,
        base_w * 0.5,
        Color::from_rgba8(200, 210, 225, 200),
    );
}

pub fn draw_7seg_ghost_and_active(pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, text: &str, ghost_rgba: Color, active_rgba: Color, glow_rgba: Color) {
    // Ghost wireframe 888 behind digits
    let _ = ghost_rgba;
    let ghost_col = Color::from_rgba8(255, 140, 0, 22);
    for ch_idx in 0..text.chars().count() {
        let ch_w = w / text.chars().count() as f32;
        let cx = x + ch_idx as f32 * ch_w + ch_w * 0.5;
        let cy = y + h * 0.5;
        // Simple 7-seg ghost: draw 7 segments as thin lines
        let seg_h = h * 0.45;
        let _seg_w = w * 0.18;
        // a
        stroke_line(pixmap, cx - ch_w*0.4, cy - seg_h*0.5, cx + ch_w*0.4, cy - seg_h*0.5, 1.0, ghost_col);
        // b
        stroke_line(pixmap, cx + ch_w*0.4, cy - seg_h*0.25, cx + ch_w*0.4, cy + seg_h*0.25, 1.0, ghost_col);
        // c
        stroke_line(pixmap, cx + ch_w*0.4, cy + seg_h*0.25, cx + ch_w*0.4, cy + seg_h*0.5, 1.0, ghost_col);
        // d
        stroke_line(pixmap, cx - ch_w*0.4, cy + seg_h*0.5, cx + ch_w*0.4, cy + seg_h*0.5, 1.0, ghost_col);
        // e
        stroke_line(pixmap, cx - ch_w*0.4, cy + seg_h*0.25, cx - ch_w*0.4, cy + seg_h*0.5, 1.0, ghost_col);
        // f
        stroke_line(pixmap, cx - ch_w*0.4, cy - seg_h*0.25, cx - ch_w*0.4, cy + seg_h*0.25, 1.0, ghost_col);
        // g
        stroke_line(pixmap, cx - ch_w*0.4, cy, cx + ch_w*0.4, cy, 1.0, ghost_col);
    }
    // Active digits: draw filled rectangles representing characters for simplicity
    let char_w = w / text.chars().count() as f32;
    for (i, ch) in text.chars().enumerate() {
        let cx = x + i as f32 * char_w + char_w * 0.5;
        let cy = y + h * 0.5;
        // Glow
        fill_rect_xywh(pixmap, cx - char_w*0.45, cy - h*0.4, char_w*0.9, h*0.8, glow_rgba);
        // Active fill
        fill_rect_xywh(pixmap, cx - char_w*0.4, cy - h*0.35, char_w*0.8, h*0.7, active_rgba);
        // Debug char hint
        let _ = ch;
    }
}

pub fn apply_crt_scanline_attenuation(pixmap: &mut PixmapMut, rect_x: f32, rect_y: f32, rect_w: f32, rect_h: f32, pitch_px: usize, attenuation_alpha: u8) {
    if pitch_px == 0 { return; }
    let y = rect_y as i32;
    let y_end = (rect_y + rect_h) as i32;
    let x_start = rect_x as i32;
    let _x_end = (rect_x + rect_w) as i32;
    let scanline_color = Color::from_rgba8(0, 0, 0, attenuation_alpha);
    let mut paint = Paint::default();
    paint.set_color(scanline_color);
    for line in (y..y_end).step_by(pitch_px*2) {
        if line % (pitch_px as i32) == pitch_px as i32 % 2 {
            if let Some(rect) = tiny_skia::Rect::from_xywh(x_start as f32, line as f32, rect_w, pitch_px as f32) {
                pixmap.fill_rect(rect, &paint, Transform::identity(), None);
            }
        }
    }
}

pub fn draw_chevron_arrow(pixmap: &mut PixmapMut, cx: f32, cy: f32, w: f32, h: f32, dir_left: bool, thickness: f32, fill_rgba: Color, glow_rgba: Color) {
    let pts = if dir_left {
        vec![
            (cx + w*0.5, cy - h*0.5),
            (cx - w*0.5, cy),
            (cx + w*0.5, cy + h*0.5),
        ]
    } else {
        vec![
            (cx - w*0.5, cy - h*0.5),
            (cx + w*0.5, cy),
            (cx - w*0.5, cy + h*0.5),
        ]
    };
    // Glow
    let mut glow_pts = pts.clone();
    for p in &mut glow_pts {
        p.0 += thickness * 0.5;
        p.1 += thickness * 0.5;
    }
    fill_polygon(pixmap, &glow_pts, glow_rgba);
    fill_polygon(pixmap, &pts, fill_rgba);
    // Stroke
    stroke_polyline(pixmap, &pts, thickness, fill_rgba, true);
}

pub fn draw_hexagon_cell(pixmap: &mut PixmapMut, cx: f32, cy: f32, r: f32, stroke_w: f32, stroke_rgba: Color, fill_rgba: Color) {
    let mut pts = Vec::with_capacity(6);
    for i in 0..6 {
        let ang = std::f32::consts::PI / 3.0 * i as f32;
        pts.push((cx + r * ang.cos(), cy + r * ang.sin()));
    }
    fill_polygon(pixmap, &pts, fill_rgba);
    stroke_polyline(pixmap, &pts, stroke_w, stroke_rgba, true);
}

