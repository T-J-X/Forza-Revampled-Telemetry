#[derive(Debug, Clone, Copy)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub struct FigmaLayout;

impl FigmaLayout {
    pub const OPTICAL_CAP_RATIO: f32 = 0.72;
    pub const TOP_LEADING_RATIO: f32 = 0.14;

    pub fn pixel_snap(x: f32, y: f32, w: f32, h: f32) -> RectF {
        RectF {
            x: x.round(),
            y: y.round(),
            w: w.round(),
            h: h.round(),
        }
    }

    pub fn optical_centered_top_y(box_h: f32, line_spacing_px: f32) -> f32 {
        let cap_height = line_spacing_px * Self::OPTICAL_CAP_RATIO;
        let top_leading = line_spacing_px * Self::TOP_LEADING_RATIO;
        (box_h - cap_height) * 0.5 - top_leading
    }

    pub fn tabular_digit_cell_width(line_spacing_px: f32) -> f32 {
        line_spacing_px * 0.56
    }

    pub fn tabular_punct_cell_width(line_spacing_px: f32) -> f32 {
        Self::tabular_digit_cell_width(line_spacing_px) * 0.48
    }

    pub fn total_tabular_width(text: &str, line_spacing_px: f32) -> f32 {
        let digit_w = Self::tabular_digit_cell_width(line_spacing_px);
        let punct_w = Self::tabular_punct_cell_width(line_spacing_px);
        text.chars().fold(0.0, |acc, c| {
            if matches!(c, '.' | ':' | '-' | '+') {
                acc + punct_w
            } else {
                acc + digit_w
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tabular_digits_same_advance() {
        let line = 20.0;
        let w1 = FigmaLayout::tabular_digit_cell_width(line);
        let w8 = FigmaLayout::tabular_digit_cell_width(line);
        assert_eq!(w1, w8);
        let total = FigmaLayout::total_tabular_width("123", line);
        assert_eq!(total, w1 * 3.0);
    }

    #[test]
    fn optical_centered_top_y() {
        let box_h = 100.0;
        let line = 30.0;
        let y = FigmaLayout::optical_centered_top_y(box_h, line);
        let cap = line * 0.72;
        let expected = (box_h - cap) * 0.5 - line * 0.14;
        assert!((y - expected).abs() < 1e-6);
    }
}
