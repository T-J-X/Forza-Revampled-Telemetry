//! Rectangles in frame pixels, and the panel geometry that turns visual angles into pixels.
use crate::error::{usage, Result};
use std::fmt;

/// The largest coordinate or size a rectangle may give (far beyond any frame this tool reads).
pub const COORD_LIMIT: i64 = 1_000_000;

/// A rectangle in pixels. `x` and `y` may be negative (anchor-local coordinates) until it is
/// clamped to an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// Left edge.
    pub x: i64,
    /// Top edge.
    pub y: i64,
    /// Width.
    pub w: i64,
    /// Height.
    pub h: i64,
}

impl Rect {
    /// A rectangle.
    pub const fn new(x: i64, y: i64, w: i64, h: i64) -> Self {
        Self { x, y, w, h }
    }

    /// Parses `X,Y,W,H`. Each value must lie within [`COORD_LIMIT`] of zero.
    ///
    /// # Errors
    /// A usage error when it is not four integers in range or has no area.
    pub fn parse(s: &str) -> Result<Self> {
        let bad = || usage(format!("bad rectangle {s:?}: expected X,Y,W,H in pixels"));
        let mut v = [0i64; 4];
        let mut n = 0;
        for part in s.split(',') {
            let slot = v.get_mut(n).ok_or_else(bad)?;
            *slot = part.trim().parse().map_err(|_| bad())?;
            n += 1;
        }
        if n != 4 {
            return Err(bad());
        }
        if v.iter().any(|c| c.abs() > COORD_LIMIT) {
            return Err(usage(format!(
                "rectangle {s:?} is out of range: values must be within {COORD_LIMIT} px"
            )));
        }
        let r = Self::new(v[0], v[1], v[2], v[3]);
        if r.w <= 0 || r.h <= 0 {
            return Err(usage(format!("rectangle {s:?} has no area")));
        }
        Ok(r)
    }

    /// Right edge (exclusive).
    pub const fn right(self) -> i64 {
        self.x.saturating_add(self.w)
    }

    /// Bottom edge (exclusive).
    pub const fn bottom(self) -> i64 {
        self.y.saturating_add(self.h)
    }

    /// Moved by `(dx, dy)`.
    pub const fn offset(self, dx: i64, dy: i64) -> Self {
        Self::new(self.x.saturating_add(dx), self.y.saturating_add(dy), self.w, self.h)
    }

    /// Grown by `m` on every side.
    pub const fn grow(self, m: i64) -> Self {
        Self::new(
            self.x.saturating_sub(m),
            self.y.saturating_sub(m),
            self.w.saturating_add(m.saturating_mul(2)),
            self.h.saturating_add(m.saturating_mul(2)),
        )
    }

    /// The part inside a `width` x `height` image, or `None` when they do not overlap.
    pub fn clamp_to(self, width: u32, height: u32) -> Option<Self> {
        let x0 = self.x.max(0);
        let y0 = self.y.max(0);
        let x1 = self.right().min(i64::from(width));
        let y1 = self.bottom().min(i64::from(height));
        (x1 > x0 && y1 > y0).then(|| Self::new(x0, y0, x1 - x0, y1 - y0))
    }

    /// Scaled by `k`, rounding outwards (maps a rectangle between resolutions).
    pub fn scaled(self, k: f64) -> Self {
        let x0 = (self.x as f64 * k).floor() as i64;
        let y0 = (self.y as f64 * k).floor() as i64;
        let x1 = (self.right() as f64 * k).ceil() as i64;
        let y1 = (self.bottom() as f64 * k).ceil() as i64;
        Self::new(x0, y0, (x1 - x0).max(1), (y1 - y0).max(1))
    }

    /// `(x, y, w, h)` unsigned, for a rectangle already inside an image.
    pub fn parts(self) -> (u32, u32, u32, u32) {
        let u = |v: i64| u32::try_from(v.max(0)).unwrap_or(u32::MAX);
        (u(self.x), u(self.y), u(self.w), u(self.h))
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{} {}x{}", self.x, self.y, self.w, self.h)
    }
}

/// Diagonal of the target panel, the 55-inch LG C1, in metres. The panel itself measures
/// 54.6 in; 55 in is the figure `docs/design/RESEARCH.md` uses (0.317 mm a pixel), kept so every
/// number in the repo agrees. It makes the flash field 0.7% smaller: the stricter side.
const PANEL_DIAGONAL_M: f64 = 55.0 * 0.0254;

/// Pixels per metre across a 16:9 panel of the target size with `width_px` columns: 3840 px over
/// 1.2175 m, so 1 px = 0.317 mm (the display constants of `docs/design/RESEARCH.md`).
pub fn px_per_m(width_px: u32) -> f64 {
    let width_m = PANEL_DIAGONAL_M * 16.0 / 16f64.hypot(9.0);
    f64::from(width_px) / width_m
}

/// The 4K pixels a visual angle of `deg` degrees spans at `eye_m` metres from the panel.
pub fn angle_px(deg: f64, eye_m: f64) -> f64 {
    2.0 * eye_m * (deg.to_radians() * 0.5).tan() * px_per_m(3840)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_rejects_rectangles() {
        assert_eq!(Rect::parse("10, 20,30,40").ok(), Some(Rect::new(10, 20, 30, 40)));
        assert_eq!(
            Rect::parse("-720,-320,1440,560").ok(),
            Some(Rect::new(-720, -320, 1440, 560))
        );
        for bad in ["", "1,2,3", "1,2,3,4,5", "a,b,c,d", "1,2,0,4", "1,2,3,-4"] {
            assert!(Rect::parse(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn clamps_scales_and_grows() {
        let r = Rect::new(-10, 5, 40, 40);
        assert_eq!(r.clamp_to(20, 20), Some(Rect::new(0, 5, 20, 15)));
        assert_eq!(Rect::new(30, 30, 5, 5).clamp_to(20, 20), None);
        assert_eq!(Rect::new(1, 1, 6, 6).scaled(0.25), Rect::new(0, 0, 2, 2));
        assert_eq!(Rect::new(10, 10, 4, 4).grow(2), Rect::new(8, 8, 8, 8));
        assert_eq!(Rect::new(3, 4, 5, 6).parts(), (3, 4, 5, 6));
    }

    #[test]
    fn panel_geometry_matches_the_research_constants() {
        // RESEARCH.md: 1 px = 0.317 mm; a 10-degree field is about 331 px across at 0.6 m and
        // about 497 px at 0.9 m.
        assert!((1000.0 / px_per_m(3840) - 0.317).abs() < 0.001);
        assert!((angle_px(10.0, 0.6) - 331.0).abs() < 1.5);
        assert!((angle_px(10.0, 0.9) - 497.0).abs() < 1.5);
    }
}
