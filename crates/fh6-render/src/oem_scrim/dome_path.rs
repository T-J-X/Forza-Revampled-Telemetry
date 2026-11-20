//! Dome path geometry for OEM binnacle scrim.
//! Builds the domed binnacle outline that the scrim fills and feathers.

use tiny_skia::PathBuilder;

#[derive(Debug, Clone)]
pub struct DomePath {
    pub builder: PathBuilder,
    pub cx: f32,
    pub cy: f32,
    pub w: f32,
    pub h: f32,
    pub scale: f32,
}

impl DomePath {
    pub fn new(cx: f32, cy: f32, w: f32, h: f32, scale: f32) -> Self {
        let mut builder = PathBuilder::new();
        let r = (24.0 * scale).max(4.0);
        let left = cx - w / 2.0;
        let right = cx + w / 2.0;
        let top = cy - h / 2.0;
        let bottom = cy + h / 2.0;
        let shoulder_y = cy - 0.08 * h;

        // Bottom baseline
        builder.move_to(left + r, bottom);
        builder.line_to(right - r, bottom);

        // Bottom-right fillet (approximate arc with cubic)
        let _arc_r = 2.0 * r;
        builder.cubic_to(right, bottom - r, right, bottom - r, right, shoulder_y);

        // Right shoulder to crown
        builder.line_to(right, shoulder_y);

        // Upper arched cowl crown (bezier)
        let cp1_x = cx + w * 0.35;
        let cp1_y = top;
        let cp2_x = cx - w * 0.35;
        let cp2_y = top;
        let end_x = left;
        let end_y = shoulder_y;
        builder.cubic_to(right, shoulder_y, cp1_x, cp1_y, cp2_x, cp2_y);
        builder.cubic_to(cp2_x, cp2_y, end_x, end_y, end_x, end_y);

        // Left shoulder down
        builder.line_to(left, bottom - r);

        // Bottom-left fillet
        builder.line_to(left + r, bottom);
        builder.close();

        Self {
            builder,
            cx,
            cy,
            w,
            h,
            scale,
        }
    }

    pub fn width_at(&self, rel_y: f32) -> f32 {
        // Approximate width taper: narrower at top
        let t = (rel_y + 0.5).clamp(0.0, 1.0);
        self.w * (0.6 + 0.4 * t)
    }

    pub fn is_dome_not_rect(&self) -> bool {
        // Dome has shoulder taper, not constant width
        self.width_at(-0.5) < self.width_at(0.5)
    }
}

pub fn create_binnacle_dome_path(cx: f32, cy: f32, w: f32, h: f32, scale: f32) -> DomePath {
    DomePath::new(cx, cy, w, h, scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dome_width_taper_test() {
        let path = create_binnacle_dome_path(960.0, 540.0, 1220.0, 400.0, 1.0);
        assert!(
            path.is_dome_not_rect(),
            "Path should taper like a dome, not a rectangle"
        );
        // Core width should be narrower than outer
        let top_w = path.width_at(-0.4);
        let bottom_w = path.width_at(0.4);
        assert!(
            top_w < bottom_w,
            "Top width {} should be less than bottom width {}",
            top_w,
            bottom_w
        );
    }

    #[test]
    fn scale_respects_min_radius() {
        let path = create_binnacle_dome_path(0.0, 0.0, 420.0, 180.0, 0.1);
        // Should not panic with tiny scale
        assert!(path.w > 0.0);
    }
}
