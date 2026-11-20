use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform, PathBuilder, Stroke};
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_render::text::{draw_text, draw_glitch_text};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn render(pixmap: &mut PixmapMut, bounds: Rect, data: &FH6TelemetryData, config: &AppConfig) {
    // OLED true-black
    let mut bg = Paint::default();
    bg.set_color(Color::from_rgba8(0, 0, 0, 255));
    pixmap.fill_rect(bounds, &bg, Transform::identity(), None);

    let cx = bounds.x() + bounds.width() / 2.0;
    let cy = bounds.y() + bounds.height() / 2.0;
    let scale = 1.0;

    render_exposed_pcb_and_glyph_strips(pixmap, cx, cy, scale, data);
    render_circular_dot_matrix_tach(pixmap, cx, cy - 40.0*scale, scale, data);
    render_dot_matrix_speed_and_op1_synthesizer_graphs(pixmap, cx, cy, scale, data, config);
}

fn render_exposed_pcb_and_glyph_strips(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    // PCB traces
    let mut pcb_paint = Paint::default();
    pcb_paint.set_color(Color::from_rgba8(30, 30, 35, 255));
    for i in -3..=3 {
        let x = cx + i as f32 * 40.0 * scale;
        let mut pb = PathBuilder::new();
        pb.move_to(x, cy - 200.0*scale);
        pb.line_to(x, cy + 200.0*scale);
        if let Some(path) = pb.finish() {
            let stroke = Stroke { width: 1.0 * scale, ..Default::default() };
            pixmap.stroke_path(&path, &pcb_paint, &stroke, Transform::identity(), None);
        }
    }
    // Registration crosses
    for i in -2..=2 {
        for j in -2..=2 {
            let x = cx + i as f32 * 80.0*scale;
            let y = cy + j as f32 * 80.0*scale;
            let mut cross_paint = Paint::default();
            cross_paint.set_color(Color::from_rgba8(200, 200, 210, 120));
            let stroke = Stroke { width: 1.0 * scale, ..Default::default() };
            let mut pb = PathBuilder::new();
            pb.move_to(x-3.0*scale, y);
            pb.line_to(x+3.0*scale, y);
            pb.move_to(x, y-3.0*scale);
            pb.line_to(x, y+3.0*scale);
            if let Some(path)=pb.finish() {
                pixmap.stroke_path(&path, &cross_paint, &stroke, Transform::identity(), None);
            }
        }
    }

    // 5 perimeter glyph strips
    let rpm_ratio = data.normalized_rpm().clamp(0.0, 1.2);
    let strips = [
        (cx - 300.0*scale, cy - 200.0*scale, 200.0*scale, 8.0*scale),
        (cx + 100.0*scale, cy - 220.0*scale, 200.0*scale, 8.0*scale),
        (cx + 300.0*scale, cy - 200.0*scale, 200.0*scale, 8.0*scale),
        (cx + 300.0*scale, cy + 200.0*scale, 200.0*scale, 8.0*scale),
        (cx - 300.0*scale, cy + 200.0*scale, 200.0*scale, 8.0*scale),
    ];
    for (i, (x, y, w, h)) in strips.iter().enumerate() {
        let progress = (rpm_ratio * (i as f32 + 1.0) / 5.0).clamp(0.0, 1.0);
        let lit_len = w * progress;
        let mut paint = Paint::default();
        let col = if rpm_ratio > 0.95 { Color::from_rgba8(255, 0, 0, 255) } else { Color::from_rgba8(255, 255, 255, 220) };
        paint.set_color(col);
        let rect = Rect::from_xywh(*x, *y, lit_len, *h);
        if let Some(r)=rect {
            pixmap.fill_rect(r, &paint, Transform::identity(), None);
        }
        // Rounded caps
        let mut pb = PathBuilder::new();
        let r_ = 4.0*scale;
        pb.move_to(*x + r_, *y);
        pb.line_to(*x + w - r_, *y);
        pb.quad_to(*x + w, *y, *x + w, *y + r_);
        pb.line_to(*x + w, *y + h - r_);
        pb.quad_to(*x + w, *y + h, *x + w - r_, *y + h);
        pb.line_to(*x + r_, *y + h);
        pb.quad_to(*x, *y + h, *x, *y + h - r_);
        pb.line_to(*x, *y + r_);
        pb.quad_to(*x, *y, *x + r_, *y);
        pb.close();
        if let Some(path)=pb.finish() {
            let mut stroke_paint = Paint::default();
            stroke_paint.set_color(Color::from_rgba8(255,255,255,60));
            let stroke = Stroke { width: 1.0*scale, ..Default::default() };
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
        }
    }
}

fn render_circular_dot_matrix_tach(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData) {
    let rpm_ratio = data.normalized_rpm().clamp(0.0, 1.2);
    let rings = 3;
    let stations = 64;
    for ring_idx in 0..rings {
        let r = 80.0*scale + ring_idx as f32 * 22.0*scale;
        for s in 0..stations {
            let ang = s as f32 / stations as f32 * std::f32::consts::TAU - std::f32::consts::PI/2.0;
            let x = cx + r * ang.cos();
            let y = cy + r * ang.sin();
            let dot_active = (s as f32 / stations as f32) < rpm_ratio;
            let col = if dot_active {
                if rpm_ratio > 0.93 && s as f32 / stations as f32 > 0.93 { Color::from_rgba8(239,68,68,255) } else { Color::from_rgba8(248,250,252,255) }
            } else {
                Color::from_rgba8(255,255,255,18)
            };
            let mut paint = Paint::default();
            paint.set_color(col);
            let rad = 2.0*scale;
            let rect = Rect::from_xywh(x - rad, y - rad, rad*2.0, rad*2.0);
            if let Some(r)=rect {
                pixmap.fill_rect(r, &paint, Transform::identity(), None);
            }
        }
    }
    // Inner power ring
    let inner_r = 60.0*scale;
    let power_ratio = ((data.power_watts/745.7)/800.0).clamp(0.0, 1.0);
    for s in 0..stations {
        let ang = s as f32 / stations as f32 * std::f32::consts::TAU - std::f32::consts::PI/2.0;
        let x = cx + inner_r * ang.cos();
        let y = cy + inner_r * ang.sin();
        let dot_active = (s as f32 / stations as f32) < power_ratio;
        let col = if dot_active { Color::from_rgba8(56,189,248,255) } else { Color::from_rgba8(255,255,255,12) };
        let mut paint = Paint::default();
        paint.set_color(col);
        let rad = 1.5*scale;
        let rect = Rect::from_xywh(x - rad, y - rad, rad*2.0, rad*2.0);
        if let Some(r)=rect { pixmap.fill_rect(r, &paint, Transform::identity(), None); }
    }
}

fn render_dot_matrix_speed_and_op1_synthesizer_graphs(pixmap: &mut PixmapMut, cx: f32, cy: f32, scale: f32, data: &FH6TelemetryData, config: &AppConfig) {
    // Hero gear dot matrix 5x7
    let gear_char = match data.gear {
        0 => 'R',
        g if g < 10 => (b'0' + g) as char,
        _ => 'N',
    };
    render_dot_char(pixmap, cx, cy + 60.0*scale, gear_char, scale, Color::from_rgba8(248,250,252,255));

    // Speed dot matrix 7x9 per digit
    let speed_kmh = data.speed_kmh();
    let speed_str = format!("{:03}", speed_kmh.round() as i32);
    let mut x_off = -120.0*scale;
    for ch in speed_str.chars() {
        render_dot_char(pixmap, cx + x_off, cy + 120.0*scale, ch, scale*0.9, Color::from_rgba8(248,250,252,255));
        x_off += 30.0*scale;
    }

    // OP-1 modules left/right
    let hp = (data.power_watts / 745.7).max(0.0);
    let rpm = data.current_engine_rpm.max(1.0);
    let torque = hp * 5252.0 / rpm;
    let boost = data.boost_psi;
    let modules = [
        (cx - 200.0*scale, cy + 200.0*scale, format!("{:.0}", hp)),
        (cx - 200.0*scale, cy + 240.0*scale, format!("{:.0}", torque)),
        (cx + 200.0*scale, cy + 200.0*scale, format!("{:.1}", boost)),
        (cx + 200.0*scale, cy + 240.0*scale, format!("RPM")),
    ];
    for (mx, my, txt) in modules.iter() {
        draw_text(pixmap, *mx, *my, txt, Color::from_rgba8(255,255,255,200), scale*0.7);
    }

    // Driver tag
    let tag = &config.custom_driver_name;
    let time_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    if config.driver_tag_glitch_fx {
        draw_glitch_text(pixmap, cx - 80.0*scale, cy + 280.0*scale, tag, Color::from_rgba8(255,255,255,255), scale*0.7, time_ms, true);
    } else {
        draw_text(pixmap, cx - 80.0*scale, cy + 280.0*scale, tag, Color::from_rgba8(255,255,255,255), scale*0.7);
    }
}

fn render_dot_char(pixmap: &mut PixmapMut, x: f32, y: f32, ch: char, scale: f32, color: Color) {
    // Simplified 5x7 bitmap for digits/letters
    let pattern = match ch {
        '0' => vec!["111","101","101","101","111"],
        '1' => vec!["010","110","010","010","111"],
        '2' => vec!["111","001","111","100","111"],
        '3' => vec!["111","001","010","001","111"],
        '4' => vec!["101","101","111","001","001"],
        'R' => vec!["111","100","110","101","100"],
        'N' => vec!["101","111","101","101","101"],
        _ => vec!["000","000","000","000","000"],
    };
    let dot_r = 1.8*scale;
    let mut paint = Paint::default();
    paint.set_color(color);
    for (row_idx, row) in pattern.iter().enumerate() {
        for (col_idx, c) in row.chars().enumerate() {
            if c == '1' {
                let dx = x + (col_idx as f32 - 2.0) * 3.0*scale;
                let dy = y + (row_idx as f32 - 2.0) * 3.0*scale;
                let rect = Rect::from_xywh(dx - dot_r, dy - dot_r, dot_r*2.0, dot_r*2.0);
                if let Some(r)=rect { pixmap.fill_rect(r, &paint, Transform::identity(), None); }
            }
        }
    }
}
