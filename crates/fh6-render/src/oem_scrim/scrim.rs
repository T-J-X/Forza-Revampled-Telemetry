//! The OEM dash scrim on the CPU (tiny-skia pixmaps, v1 clusters and tools).
//!
//! Same dome as the kit (`fh6_kit::scrim`): the geometry is `fh6_core::fit::ScrimFit`, centred
//! on the cluster anchor + (0, −75) in cluster local units, so both paths agree pixel for pixel
//! up to dither (`docs/design/SCRIM_FIT.md` §2).
use fh6_core::fit::{clamp_anchor, FitView, ScrimFit, ViewPlacement};
use fh6_core::{AppConfig, SafeZone};
use tiny_skia::PixmapMut;

/// Reference size of the authoring space (4K).
const REF: (f32, f32) = (3840.0, 2160.0);
/// The cockpit envelope from the tokens (`layout.cockpit.envelope`), local units.
const COCKPIT_ENVELOPE: (f32, f32, f32, f32) = (-720.0, -320.0, 720.0, 240.0);
/// The hood envelope from the tokens, local units.
const HOOD_ENVELOPE: (f32, f32, f32, f32) = (-880.0, -220.0, 880.0, 180.0);

/// Where cluster local space lands on a canvas: the origin (anchor, after the height offset)
/// in canvas px and the canvas px per local unit. Mirrors `fh6_kit::Layout::from_config`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalFrame {
    /// Local (0, 0) in canvas px, height offset applied.
    pub origin: (f32, f32),
    /// Canvas px per local unit.
    pub unit: f32,
}

impl LocalFrame {
    /// The frame for `view` on a `w` x `h` canvas.
    #[must_use]
    pub fn from_config(cfg: &AppConfig, view: FitView, w: u32, h: u32) -> Self {
        let (w, h) = (w.max(1) as f32, h.max(1) as f32);
        let place = ViewPlacement::from_config(cfg, view);
        let k = h / REF.1;
        let unit = place.scale * k;
        let safe = SafeZone {
            side_px: place.safe.side_px * k,
            top_px: place.safe.top_px * k,
            bottom_px: place.safe.bottom_px * k,
        };
        let env = match view {
            FitView::Hood => HOOD_ENVELOPE,
            FitView::Cockpit | FitView::Chase => COCKPIT_ENVELOPE,
        };
        let (ax, ay) = clamp_anchor(
            (place.anchor.0 * w / REF.0, place.anchor.1 * k),
            env,
            unit,
            place.height_offset_px,
            (w, h),
            safe,
        );
        Self {
            origin: (ax, ay - place.height_offset_px * unit),
            unit,
        }
    }

    /// Canvas px to local units.
    #[must_use]
    pub fn to_local(&self, x: f32, y: f32) -> (f32, f32) {
        let u = self.unit.max(1e-6);
        ((x - self.origin.0) / u, (y - self.origin.1) / u)
    }

    /// The canvas rect (x0, y0, x1, y1) covering a local rect, clipped to `w` x `h`.
    fn pixel_rect(&self, l: (f32, f32, f32, f32), w: u32, h: u32) -> (u32, u32, u32, u32) {
        let px = |v: f32, o: f32, max: u32| ((o + v * self.unit).floor().max(0.0) as u32).min(max);
        (
            px(l.0, self.origin.0, w),
            px(l.1, self.origin.1, h),
            px(l.2, self.origin.0, w).saturating_add(1).min(w),
            px(l.3, self.origin.1, h).saturating_add(1).min(h),
        )
    }
}

/// Triangular dither in [-1, 1) LSB from a pixel position (the CPU twin of the shader's).
pub(crate) fn dither(x: u32, y: u32) -> f32 {
    let h = |a: u32, b: u32| -> f32 {
        let mut v = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77);
        v ^= v >> 15;
        v = v.wrapping_mul(0xC2B2_AE3D);
        v ^= v >> 13;
        (v & 0xFFFF) as f32 / 65536.0
    };
    h(x, y) + h(x.wrapping_add(37), y.wrapping_add(37)) - 1.0
}

/// Draws the scrim for `cfg`'s cockpit view over a premultiplied pixmap (source-over black).
/// Nothing is drawn at 0%.
pub fn render_oem_scrim(pixmap: &mut PixmapMut, cfg: &AppConfig) {
    let fit = ScrimFit::from_config(cfg, FitView::Cockpit);
    let frame = LocalFrame::from_config(cfg, FitView::Cockpit, pixmap.width(), pixmap.height());
    render_scrim_fit(pixmap, &fit, &frame);
}

/// Draws a scrim fit in a local frame over a premultiplied pixmap.
pub fn render_scrim_fit(pixmap: &mut PixmapMut, fit: &ScrimFit, frame: &LocalFrame) {
    if !fit.visible() {
        return;
    }
    let (w, h) = (pixmap.width(), pixmap.height());
    let fmax = fit.feather.0.max(fit.feather.1).max(fit.feather.2);
    let (cx, cy) = fit.center;
    let (a, b) = fit.semi_axes;
    let (x0, y0, x1, y1) = frame.pixel_rect((cx - a - fmax, cy - b - fmax, cx + a + fmax, cy + b + fmax), w, h);
    let opacity = fit.opacity.clamp(0.0, 1.0);
    let data = pixmap.data_mut();
    for y in y0..y1 {
        for x in x0..x1 {
            let (lx, ly) = frame.to_local(x as f32 + 0.5, y as f32 + 0.5);
            let cov = fit.coverage(lx, ly) * opacity;
            if cov <= 0.0 {
                continue;
            }
            let a8 = if cov >= 1.0 {
                255.0
            } else {
                (cov * 255.0 + dither(x, y) * 0.5).round().clamp(0.0, 255.0)
            };
            let keep = 255.0 - a8;
            let i = (y as usize * w as usize + x as usize) * 4;
            if let Some(px) = data.get_mut(i..i + 4) {
                for (c, v) in px.iter_mut().enumerate() {
                    let src = if c == 3 { a8 } else { 0.0 };
                    *v = (src + f32::from(*v) * keep / 255.0).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Pixmap;

    fn alpha(p: &Pixmap, x: u32, y: u32) -> u8 {
        p.pixel(x, y).map_or(0, |c| c.alpha())
    }

    #[test]
    fn default_scrim_sits_on_the_binnacle_anchor_not_a_third_of_the_screen() {
        let cfg = AppConfig {
            oem_dash_opaque_pct: 100,
            ..AppConfig::default()
        };
        let mut pm = Pixmap::new(1920, 1080).unwrap();
        render_oem_scrim(&mut pm.as_mut(), &cfg);
        // Dome centre at 4K (1920, 1760) -> (960, 880); top at 4K y 1290 -> 645.
        assert_eq!(alpha(&pm, 960, 880), 255);
        assert_eq!(alpha(&pm, 960, 917), 255, "anchor (1835 at 4K) is in the core");
        assert_eq!(alpha(&pm, 960, 650), 255, "just inside the top");
        assert_eq!(alpha(&pm, 960, 590), 0, "above the top feather");
        assert_eq!(alpha(&pm, 960, 1080 / 3), 0, "nothing at the old 0.34 height");
        assert_eq!(alpha(&pm, 550, 880), 0, "clear beyond the side feather (x 1160 at 4K)");
        let feather = alpha(&pm, 960, 645 - 24);
        assert!((60..=200).contains(&feather), "feathered top edge {feather}");
    }

    #[test]
    fn core_is_pure_black_and_scales_with_opacity() {
        let mut cfg = AppConfig {
            oem_dash_opaque_pct: 50,
            ..AppConfig::default()
        };
        let mut pm = Pixmap::new(960, 540).unwrap();
        render_oem_scrim(&mut pm.as_mut(), &cfg);
        let c = pm.pixel(480, 440).unwrap();
        assert_eq!((c.red(), c.green(), c.blue()), (0, 0, 0));
        assert!((126..=129).contains(&c.alpha()), "50% core {}", c.alpha());
        cfg.oem_dash_opaque_pct = 0;
        let mut pm = Pixmap::new(960, 540).unwrap();
        render_oem_scrim(&mut pm.as_mut(), &cfg);
        assert!(pm.data().iter().all(|&v| v == 0), "0% draws nothing");
    }

    #[test]
    fn hood_and_chase_have_no_scrim() {
        let cfg = AppConfig {
            oem_dash_opaque_pct: 100,
            ..AppConfig::default()
        };
        for view in [FitView::Hood, FitView::Chase] {
            let fit = ScrimFit::from_config(&cfg, view);
            let frame = LocalFrame::from_config(&cfg, view, 960, 540);
            let mut pm = Pixmap::new(960, 540).unwrap();
            render_scrim_fit(&mut pm.as_mut(), &fit, &frame);
            assert!(pm.data().iter().all(|&v| v == 0));
        }
    }

    #[test]
    fn column_void_shows_the_game() {
        let cfg = AppConfig {
            oem_dash_opaque_pct: 100,
            ..AppConfig::default()
        };
        let mut pm = Pixmap::new(3840, 2160).unwrap();
        render_oem_scrim(&mut pm.as_mut(), &cfg);
        assert!(alpha(&pm, 1920, 2155) < 8, "void below anchor + 300");
        assert_eq!(alpha(&pm, 1920 + 300, 2155), 255);
    }

    #[test]
    fn frame_follows_height_offset_and_scale() {
        let cfg = AppConfig {
            cockpit_height_offset_px: 100.0,
            cluster_scale: 1.2,
            ..AppConfig::default()
        };
        let f = LocalFrame::from_config(&cfg, FitView::Cockpit, 3840, 2160);
        assert!((f.unit - 1.2).abs() < 1e-5);
        assert!((f.origin.0 - 1920.0).abs() < 1e-3);
        assert!(
            f.origin.1 < 1835.0 - 100.0,
            "raised by the offset (and clamped if needed)"
        );
        let (lx, ly) = f.to_local(f.origin.0, f.origin.1);
        assert!(lx.abs() < 1e-4 && ly.abs() < 1e-4);
    }
}
