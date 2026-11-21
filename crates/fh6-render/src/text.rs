//! Vector text and 7-segment digit renderer for clusters and OSD in tiny-skia.
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Stroke, Transform};

pub struct TextRenderer;

impl TextRenderer {
    pub fn measure_char_width(_c: char, font_size: f32) -> f32 {
        font_size * 0.62
    }

    pub fn optical_offset(line_spacing: f32) -> f32 {
        line_spacing * 0.14
    }
}

/// Draws an ASCII or special driver-tag string using clean vector strokes in tiny-skia.
pub fn draw_text(pixmap: &mut PixmapMut, x: f32, y: f32, text: &str, color: Color, scale: f32) {
    let mut cur_x = x;
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    for ch in text.chars() {
        if ch == ' ' {
            cur_x += 6.0 * scale;
            continue;
        }
        draw_glyph(pixmap, cur_x, y, ch, &paint, scale);
        cur_x += match ch {
            '『' | '』' | '死' => 10.0 * scale,
            _ => 7.5 * scale,
        };
    }
}

/// Draws glitch text with chromatic aberration (Red/Cyan shift) and subtle periodic jitter.
pub fn draw_glitch_text(
    pixmap: &mut PixmapMut,
    x: f32,
    y: f32,
    text: &str,
    base_color: Color,
    scale: f32,
    time_ms: u64,
    enabled: bool,
) {
    if !enabled {
        draw_text(pixmap, x, y, text, base_color, scale);
        return;
    }

    let cycle = time_ms % 2400;
    let is_glitching = cycle < 120;
    let shift_x = if is_glitching { 2.0 * scale } else { 0.75 * scale };

    // 1. Red channel chromatic ghost (-x)
    let red_ghost = Color::from_rgba8(255, 23, 68, 150);
    draw_text(pixmap, x - shift_x, y, text, red_ghost, scale);

    // 2. Cyan channel chromatic ghost (+x)
    let cyan_ghost = Color::from_rgba8(0, 229, 255, 150);
    draw_text(pixmap, x + shift_x, y, text, cyan_ghost, scale);

    // 3. Main crisp text
    draw_text(pixmap, x, y, text, base_color, scale);
}

/// Draws 7-segment motorsport digital numbers (0-9, R, N, -).
pub fn draw_7segment_digits(pixmap: &mut PixmapMut, x: f32, y: f32, digits: &str, color: Color, scale: f32) {
    let mut cur_x = x;
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    for ch in digits.chars() {
        draw_7segment_char(pixmap, cur_x, y, ch, &paint, scale);
        cur_x += 12.0 * scale;
    }
}

/// Draws the central Hero Gear character ('R', 'N', '1'..'9').
pub fn draw_gear(pixmap: &mut PixmapMut, x: f32, y: f32, gear: u8, color: Color, scale: f32) {
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    let ch = match gear {
        0 => 'R',
        g if g < 10 => (b'0' + g) as char,
        _ => 'N',
    };
    draw_glyph(pixmap, x, y, ch, &paint, scale);
}

fn draw_7segment_char(pixmap: &mut PixmapMut, x: f32, y: f32, ch: char, paint: &Paint, scale: f32) {
    let stroke = Stroke {
        width: (2.0 * scale).max(1.0),
        ..Default::default()
    };
    let w = 8.0 * scale;
    let h = 14.0 * scale;
    let half_h = h * 0.5;

    // Segment definitions (a: top, b: tr, c: br, d: bot, e: bl, f: tl, g: mid)
    let mask = match ch {
        '0' => 0b0111111,
        '1' => 0b0000110,
        '2' => 0b1011011,
        '3' => 0b1001111,
        '4' => 0b1100110,
        '5' => 0b1101101,
        '6' => 0b1111101,
        '7' => 0b0000111,
        '8' => 0b1111111,
        '9' => 0b1101111,
        '-' => 0b1000000,
        'R' => 0b1110111,
        'N' => 0b0110111,
        _ => 0b0000000,
    };

    let mut pb = PathBuilder::new();
    if (mask & 0b0000001) != 0 {
        // a: top
        pb.move_to(x + 1.0 * scale, y);
        pb.line_to(x + w - 1.0 * scale, y);
    }
    if (mask & 0b0000010) != 0 {
        // b: top-right
        pb.move_to(x + w, y + 1.0 * scale);
        pb.line_to(x + w, y + half_h - 0.5 * scale);
    }
    if (mask & 0b0000100) != 0 {
        // c: bot-right
        pb.move_to(x + w, y + half_h + 0.5 * scale);
        pb.line_to(x + w, y + h - 1.0 * scale);
    }
    if (mask & 0b0001000) != 0 {
        // d: bottom
        pb.move_to(x + 1.0 * scale, y + h);
        pb.line_to(x + w - 1.0 * scale, y + h);
    }
    if (mask & 0b0010000) != 0 {
        // e: bot-left
        pb.move_to(x, y + half_h + 0.5 * scale);
        pb.line_to(x, y + h - 1.0 * scale);
    }
    if (mask & 0b0100000) != 0 {
        // f: top-left
        pb.move_to(x, y + 1.0 * scale);
        pb.line_to(x, y + half_h - 0.5 * scale);
    }
    if (mask & 0b1000000) != 0 {
        // g: middle
        pb.move_to(x + 1.0 * scale, y + half_h);
        pb.line_to(x + w - 1.0 * scale, y + half_h);
    }

    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, &stroke, Transform::identity(), None);
    }
}

fn draw_glyph(pixmap: &mut PixmapMut, x: f32, y: f32, ch: char, paint: &Paint, scale: f32) {
    let stroke = Stroke {
        width: (1.3 * scale).max(1.0),
        ..Default::default()
    };
    let mut pb = PathBuilder::new();
    let w = 5.0 * scale;
    let h = 8.0 * scale;

    match ch {
        '0'..='9' => {
            draw_7segment_char(pixmap, x, y, ch, paint, scale * 0.65);
            return;
        }
        'A' => {
            pb.move_to(x, y + h);
            pb.line_to(x, y + h * 0.3);
            pb.line_to(x + w * 0.5, y);
            pb.line_to(x + w, y + h * 0.3);
            pb.line_to(x + w, y + h);
            pb.move_to(x, y + h * 0.55);
            pb.line_to(x + w, y + h * 0.55);
        }
        'B' => {
            pb.move_to(x, y);
            pb.line_to(x + w * 0.75, y);
            pb.line_to(x + w, y + h * 0.25);
            pb.line_to(x, y + h * 0.5);
            pb.line_to(x + w, y + h * 0.75);
            pb.line_to(x + w * 0.75, y + h);
            pb.line_to(x, y + h);
            pb.close();
        }
        'C' => {
            pb.move_to(x + w, y);
            pb.line_to(x, y);
            pb.line_to(x, y + h);
            pb.line_to(x + w, y + h);
        }
        'D' => {
            pb.move_to(x, y);
            pb.line_to(x + w * 0.7, y);
            pb.line_to(x + w, y + h * 0.5);
            pb.line_to(x + w * 0.7, y + h);
            pb.line_to(x, y + h);
            pb.close();
        }
        'E' => {
            pb.move_to(x + w, y);
            pb.line_to(x, y);
            pb.line_to(x, y + h);
            pb.line_to(x + w, y + h);
            pb.move_to(x, y + h * 0.5);
            pb.line_to(x + w * 0.7, y + h * 0.5);
        }
        'F' => {
            pb.move_to(x + w, y);
            pb.line_to(x, y);
            pb.line_to(x, y + h);
            pb.move_to(x, y + h * 0.5);
            pb.line_to(x + w * 0.65, y + h * 0.5);
        }
        'G' => {
            pb.move_to(x + w, y);
            pb.line_to(x, y);
            pb.line_to(x, y + h);
            pb.line_to(x + w, y + h);
            pb.line_to(x + w, y + h * 0.5);
            pb.line_to(x + w * 0.5, y + h * 0.5);
        }
        'H' => {
            pb.move_to(x, y);
            pb.line_to(x, y + h);
            pb.move_to(x + w, y);
            pb.line_to(x + w, y + h);
            pb.move_to(x, y + h * 0.5);
            pb.line_to(x + w, y + h * 0.5);
        }
        'I' => {
            pb.move_to(x, y);
            pb.line_to(x + w, y);
            pb.move_to(x + w * 0.5, y);
            pb.line_to(x + w * 0.5, y + h);
            pb.move_to(x, y + h);
            pb.line_to(x + w, y + h);
        }
        'J' => {
            pb.move_to(x + w, y);
            pb.line_to(x + w, y + h * 0.8);
            pb.line_to(x + w * 0.7, y + h);
            pb.line_to(x + w * 0.2, y + h);
            pb.line_to(x, y + h * 0.7);
        }
        'K' => {
            pb.move_to(x, y);
            pb.line_to(x, y + h);
            pb.move_to(x + w, y);
            pb.line_to(x, y + h * 0.5);
            pb.line_to(x + w, y + h);
        }
        'L' => {
            pb.move_to(x, y);
            pb.line_to(x, y + h);
            pb.line_to(x + w, y + h);
        }
        'M' => {
            pb.move_to(x, y + h);
            pb.line_to(x, y);
            pb.line_to(x + w * 0.5, y + h * 0.6);
            pb.line_to(x + w, y);
            pb.line_to(x + w, y + h);
        }
        'N' => {
            pb.move_to(x, y + h);
            pb.line_to(x, y);
            pb.line_to(x + w, y + h);
            pb.line_to(x + w, y);
        }
        'O' => {
            pb.move_to(x, y);
            pb.line_to(x + w, y);
            pb.line_to(x + w, y + h);
            pb.line_to(x, y + h);
            pb.close();
        }
        'P' => {
            pb.move_to(x, y + h);
            pb.line_to(x, y);
            pb.line_to(x + w, y);
            pb.line_to(x + w, y + h * 0.5);
            pb.line_to(x, y + h * 0.5);
        }
        'R' => {
            pb.move_to(x, y + h);
            pb.line_to(x, y);
            pb.line_to(x + w, y);
            pb.line_to(x + w, y + h * 0.5);
            pb.line_to(x, y + h * 0.5);
            pb.move_to(x + w * 0.5, y + h * 0.5);
            pb.line_to(x + w, y + h);
        }
        'S' => {
            pb.move_to(x + w, y);
            pb.line_to(x, y);
            pb.line_to(x, y + h * 0.5);
            pb.line_to(x + w, y + h * 0.5);
            pb.line_to(x + w, y + h);
            pb.line_to(x, y + h);
        }
        'T' | 'Ŧ' => {
            pb.move_to(x, y);
            pb.line_to(x + w, y);
            pb.move_to(x + w * 0.5, y);
            pb.line_to(x + w * 0.5, y + h);
            if ch == 'Ŧ' {
                pb.move_to(x + w * 0.1, y + h * 0.35);
                pb.line_to(x + w * 0.9, y + h * 0.35);
            }
        }
        'Ø' => {
            pb.move_to(x, y);
            pb.line_to(x + w, y);
            pb.line_to(x + w, y + h);
            pb.line_to(x, y + h);
            pb.close();
            pb.move_to(x - 1.0 * scale, y + h + 1.0 * scale);
            pb.line_to(x + w + 1.0 * scale, y - 1.0 * scale);
        }
        '『' => {
            pb.move_to(x + w, y);
            pb.line_to(x, y);
            pb.line_to(x, y + h * 0.5);
        }
        '』' => {
            pb.move_to(x, y + h);
            pb.line_to(x + w, y + h);
            pb.line_to(x + w, y + h * 0.5);
        }
        '死' => {
            // Authentic Kanji "Shi" (Death) vector strokes
            let kw = w * 1.5;
            // Top horizontal line
            pb.move_to(x, y + h * 0.15);
            pb.line_to(x + kw, y + h * 0.15);
            // Left vertical down
            pb.move_to(x + kw * 0.25, y + h * 0.15);
            pb.line_to(x + kw * 0.25, y + h * 0.9);
            // Middle horizontal
            pb.move_to(x + kw * 0.25, y + h * 0.45);
            pb.line_to(x + kw * 0.8, y + h * 0.45);
            // Right slant curve
            pb.move_to(x + kw * 0.65, y + h * 0.15);
            pb.line_to(x + kw * 0.85, y + h * 0.85);
            // Bottom hook
            pb.move_to(x + kw * 0.25, y + h * 0.9);
            pb.line_to(x + kw * 0.95, y + h * 0.9);
        }
        '乂' => {
            pb.move_to(x, y);
            pb.line_to(x + w, y + h);
            pb.move_to(x + w, y);
            pb.line_to(x, y + h);
        }
        ':' => {
            pb.move_to(x + w * 0.5, y + h * 0.3);
            pb.line_to(x + w * 0.5, y + h * 0.35);
            pb.move_to(x + w * 0.5, y + h * 0.7);
            pb.line_to(x + w * 0.5, y + h * 0.75);
        }
        _ => {
            pb.move_to(x, y);
            pb.line_to(x + w, y);
            pb.line_to(x + w, y + h);
            pb.line_to(x, y + h);
            pb.close();
        }
    }

    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, paint, &stroke, Transform::identity(), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Pixmap;

    #[test]
    fn optical_offset_positive() {
        assert!(TextRenderer::optical_offset(20.0) > 0.0);
    }

    #[test]
    fn test_draw_tag_and_digits_allocates() {
        let mut pixmap = Pixmap::new(200, 50).unwrap();
        let color = Color::from_rgba8(0, 229, 255, 255);
        draw_text(&mut pixmap.as_mut(), 10.0, 10.0, "ØG『死』ŦJ乂", color, 1.5);
        draw_7segment_digits(&mut pixmap.as_mut(), 100.0, 10.0, "142", color, 1.5);

        let non_zero = pixmap.data().iter().filter(|&&b| b != 0).count();
        assert!(non_zero > 50, "Drawn tag and digits must modify pixels");
    }
}
