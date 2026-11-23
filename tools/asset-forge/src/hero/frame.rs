//! Camera matching: a render frame registered 1:1 to the kit's local 4K px.
//!
//! A [`Frame`] is a rectangle in **local px** (the cluster's drawing units round its anchor, x
//! right, y down) plus an output scale. With [`View::Ortho`] every output pixel maps to exactly
//! `1 / scale` local px, so a beauty plate drops into `ImagePaint { rect }` with no fitting: the
//! plate's rect *is* the frame's rect. [`View::Persp`] keeps the z = 0 plane registered at tilt 0
//! and adds real parallax for parts that stand proud of it (use it for hero stills and turntables,
//! ortho for anything the kit composites).
use serde_json::{json, Value};

/// Blender units per local px. Everything in Blender is built at this scale, so a 244 px bezel
/// is 2.44 BU: big enough for Cycles' ray offsets, small enough for 32-bit precision.
pub const PX: f32 = 0.01;

/// The kit's view anchors in 4K screen px and their layout envelopes in local px
/// (`docs/kit/README.md`: cockpit and hood).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    /// Anchor on the 3840x2160 screen.
    pub screen: [f32; 2],
    /// Envelope `(x, y, w, h)` in local px round the anchor.
    pub envelope: [f32; 4],
}

/// Cockpit `[3]`: the steering-column gap.
pub const COCKPIT: Anchor = Anchor {
    screen: [1920.0, 1835.0],
    envelope: [-720.0, -320.0, 1440.0, 560.0],
};

/// Hood `[4]`.
pub const HOOD: Anchor = Anchor {
    screen: [1920.0, 1760.0],
    envelope: [-880.0, -220.0, 1760.0, 400.0],
};

/// Projection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    /// Straight down, pixel-exact. The default for anything the kit composites.
    Ortho,
    /// A real lens. `lens_mm` on a 36 mm sensor; `tilt_deg` rotates the camera about the frame's
    /// horizontal centre line (positive looks from below, like the driver's eye on a binnacle).
    Persp {
        /// Focal length (mm, 36 mm sensor width).
        lens_mm: f32,
        /// Tilt about the x axis (degrees).
        tilt_deg: f32,
    },
}

/// A render frame in local px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// Left edge (local px).
    pub x: f32,
    /// Top edge (local px).
    pub y: f32,
    /// Width (local px).
    pub w: f32,
    /// Height (local px).
    pub h: f32,
    /// Output px per local px (1.0 = native 4K; 0.5 for previews).
    pub scale: f32,
    /// Projection.
    pub view: View,
}

impl Frame {
    /// An ortho frame over `(x, y, w, h)` local px at native 4K scale.
    #[must_use]
    pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x,
            y,
            w,
            h,
            scale: 1.0,
            view: View::Ortho,
        }
    }

    /// A square ortho frame of `size` px centred on `(cx, cy)`: sprites (needles, LED caps).
    #[must_use]
    pub fn square(cx: f32, cy: f32, size: f32) -> Self {
        Self::rect(cx - size * 0.5, cy - size * 0.5, size, size)
    }

    /// The whole envelope of an anchor.
    #[must_use]
    pub fn envelope(a: &Anchor) -> Self {
        let [x, y, w, h] = a.envelope;
        Self::rect(x, y, w, h)
    }

    /// The same frame at another output scale.
    #[must_use]
    pub fn scaled(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.01);
        self
    }

    /// The same frame through a lens.
    #[must_use]
    pub fn persp(mut self, lens_mm: f32, tilt_deg: f32) -> Self {
        self.view = View::Persp { lens_mm, tilt_deg };
        self
    }

    /// Output resolution (rounded, at least 1 px).
    #[must_use]
    pub fn resolution(&self) -> (u32, u32) {
        let r = |v: f32| (v * self.scale).round().max(1.0) as u32;
        (r(self.w), r(self.h))
    }

    /// Local px covered by one output pixel (exact in ortho: width / res_x).
    #[must_use]
    pub fn px_per_pixel(&self) -> f32 {
        let (rx, _) = self.resolution();
        self.w / rx as f32
    }

    /// Centre in local px.
    #[must_use]
    pub fn centre(&self) -> [f32; 2] {
        [self.x + self.w * 0.5, self.y + self.h * 0.5]
    }

    /// The camera for Blender: resolution, ortho scale or lens and distance, all in BU.
    /// The ortho scale uses the rounded resolution so pixel registration stays exact.
    #[must_use]
    pub fn to_json(self) -> Value {
        let (rx, ry) = self.resolution();
        let [cx, cy] = self.centre();
        let width_bu = rx as f32 / self.scale * PX;
        match self.view {
            View::Ortho => json!({
                "type": "ORTHO", "res": [rx, ry], "centre": [cx * PX, -cy * PX],
                "ortho_scale": width_bu,
            }),
            View::Persp { lens_mm, tilt_deg } => {
                let dist = width_bu * lens_mm / 36.0;
                json!({
                    "type": "PERSP", "res": [rx, ry], "centre": [cx * PX, -cy * PX],
                    "lens": lens_mm, "distance": dist, "tilt": tilt_deg,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ortho_frame_is_pixel_exact_at_4k() {
        let f = Frame::envelope(&COCKPIT);
        assert_eq!(f.resolution(), (1440, 560));
        assert!((f.px_per_pixel() - 1.0).abs() < 1e-6);
        let j = f.to_json();
        assert_eq!(j["type"], "ORTHO");
        assert!((j["ortho_scale"].as_f64().unwrap_or(0.0) - 14.4).abs() < 1e-4);
        // Centre (0, -40) local px → Blender y is flipped.
        assert!((j["centre"][1].as_f64().unwrap_or(0.0) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn preview_scale_halves_the_resolution_but_not_the_coverage() {
        let f = Frame::square(0.0, 0.0, 512.0).scaled(0.5);
        assert_eq!(f.resolution(), (256, 256));
        assert!((f.px_per_pixel() - 2.0).abs() < 1e-6);
        assert!((f.to_json()["ortho_scale"].as_f64().unwrap_or(0.0) - 5.12).abs() < 1e-4);
    }

    #[test]
    fn perspective_distance_fills_the_width() {
        let f = Frame::rect(-500.0, -100.0, 1000.0, 200.0).persp(50.0, 0.0);
        let j = f.to_json();
        // 10 BU wide at 50 mm on a 36 mm sensor → 13.89 BU away.
        assert!((j["distance"].as_f64().unwrap_or(0.0) - 13.888_889).abs() < 1e-3);
    }
}
