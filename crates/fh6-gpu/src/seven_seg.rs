//! Seven-segment text on the quad pipeline: digits, a usable subset of letters, `.`, `-`, ` `.
//!
//! Glyphs use a fixed advance (tabular), so numbers never shift sideways. Formatting numbers
//! goes through [`StackStr`], so drawing text allocates nothing.
use crate::quad::{QuadBatch, QuadInstance, QuadKind};
use std::fmt;

/// Segment bits: a=top, b=top-right, c=bottom-right, d=bottom, e=bottom-left, f=top-left, g=middle.
pub const ALL_SEGMENTS: u8 = 0x7F;

/// The segment mask for `c`, or `None` if it cannot be shown on seven segments.
pub fn mask_for(c: char) -> Option<u8> {
    Some(match c {
        '0' | 'O' => 0x3F,
        '1' | 'I' => 0x06,
        '2' => 0x5B,
        '3' => 0x4F,
        '4' => 0x66,
        '5' | 'S' | 's' => 0x6D,
        '6' => 0x7D,
        '7' => 0x07,
        '8' | 'B' => 0x7F,
        '9' => 0x6F,
        'A' | 'a' => 0x77,
        'b' => 0x7C,
        'C' => 0x39,
        'c' => 0x58,
        'd' | 'D' => 0x5E,
        'E' | 'e' => 0x79,
        'F' | 'f' => 0x71,
        'G' | 'g' => 0x3D,
        'H' => 0x76,
        'h' => 0x74,
        'J' | 'j' => 0x1E,
        'L' | 'l' => 0x38,
        'N' | 'n' => 0x54,
        'o' => 0x5C,
        'P' | 'p' => 0x73,
        'q' | 'Q' => 0x67,
        'R' | 'r' => 0x50,
        't' | 'T' => 0x78,
        'U' => 0x3E,
        'u' | 'v' | 'V' => 0x1C,
        'y' | 'Y' => 0x6E,
        '-' => 0x40,
        '_' => 0x08,
        '=' => 0x48,
        ' ' => 0x00,
        _ => return None,
    })
}

/// How text is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegStyle {
    /// Glyph height in pixels.
    pub height: f32,
    /// Lit segment colour (straight alpha, sRGB-encoded).
    pub color: [f32; 4],
    /// If set, unlit segments are drawn in this colour (the "ghost 8" look).
    pub ghost: Option<[f32; 4]>,
}

impl SegStyle {
    /// Glyph width for this height.
    pub fn glyph_width(&self) -> f32 {
        self.height * 0.56
    }
    /// Segment thickness for this height.
    pub fn thickness(&self) -> f32 {
        (self.height * 0.13).max(1.0)
    }
    /// Horizontal advance per glyph.
    pub fn advance(&self) -> f32 {
        self.glyph_width() + self.height * 0.18
    }
    /// Width of `text` when drawn with [`push_text`].
    pub fn text_width(&self, text: &str) -> f32 {
        text.chars()
            .map(|c| if c == '.' { self.thickness() * 1.8 } else { self.advance() })
            .sum()
    }
}

/// Draws one glyph mask at (x, y) (top-left).
pub fn push_glyph(batch: &mut QuadBatch, mask: u8, x: f32, y: f32, style: &SegStyle) {
    let rect = [x, y, style.glyph_width(), style.height];
    let thickness = style.thickness();
    if let Some(ghost) = style.ghost {
        batch.push(QuadInstance::new(
            rect,
            ghost,
            QuadKind::Segments {
                thickness,
                mask: ALL_SEGMENTS,
            },
        ));
    }
    if mask != 0 {
        batch.push(QuadInstance::new(rect, style.color, QuadKind::Segments { thickness, mask }));
    }
}

/// Draws `text` starting at (x, y) and returns the x after the last glyph. Characters with no
/// seven-segment form are drawn as blanks; `.` is a small square that takes little space.
pub fn push_text(batch: &mut QuadBatch, text: &str, x: f32, y: f32, style: &SegStyle) -> f32 {
    let mut pen = x;
    for c in text.chars() {
        if c == '.' {
            let t = style.thickness();
            batch.push(QuadInstance::new(
                [pen + t * 0.2, y + style.height - t, t, t],
                style.color,
                QuadKind::RoundedRect {
                    radius: t * 0.25,
                    rim: 0.0,
                },
            ));
            pen += t * 1.8;
            continue;
        }
        push_glyph(batch, mask_for(c).unwrap_or(0), pen, y, style);
        pen += style.advance();
    }
    pen
}

/// A small string on the stack, for formatting numbers without allocating.
#[derive(Clone, Copy)]
pub struct StackStr<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Default for StackStr<N> {
    fn default() -> Self {
        Self { buf: [0; N], len: 0 }
    }
}

impl<const N: usize> StackStr<N> {
    /// An empty string.
    pub fn new() -> Self {
        Self::default()
    }

    /// The text written so far (always valid UTF-8, since only whole `&str`s are copied in).
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    /// Clears the string.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl<const N: usize> fmt::Write for StackStr<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        if end > N {
            return Err(fmt::Error);
        }
        self.buf[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

impl<const N: usize> fmt::Debug for StackStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    #[test]
    fn digits_have_classic_masks() {
        let masks: Vec<u8> = "0123456789".chars().filter_map(mask_for).collect();
        assert_eq!(masks, [0x3F, 0x06, 0x5B, 0x4F, 0x66, 0x6D, 0x7D, 0x07, 0x7F, 0x6F]);
    }

    #[test]
    fn stats_labels_are_representable() {
        for label in ["P50", "P99", "CPU", "FPS", "r", "n"] {
            assert!(label.chars().all(|c| mask_for(c).is_some()), "{label}");
        }
    }

    #[test]
    fn stack_str_formats_without_heap() {
        let mut s = StackStr::<16>::new();
        write!(s, "{:5.2}", 3.14159).expect("fits");
        assert_eq!(s.as_str(), " 3.14");
        let mut tiny = StackStr::<2>::new();
        assert!(write!(tiny, "123").is_err(), "overflow is an error, not a panic");
    }

    #[test]
    fn tabular_advance_is_fixed() {
        let style = SegStyle {
            height: 100.0,
            color: [1.0; 4],
            ghost: None,
        };
        assert_eq!(style.text_width("111"), style.text_width("888"));
    }
}
