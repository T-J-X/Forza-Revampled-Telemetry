//! The binnacle ambient wash on the CPU (tiny-skia pixmaps, v1 clusters and tools).
//!
//! A 1.5 px lip of the cluster colour along the dome's top edge plus a faint halo just outside
//! it, alpha 12–35 of 255, fading towards the ends of the arc (`docs/design/SCRIM_FIT.md` §3).
//! Never a fill. The CPU twin of `fh6_kit::scrim::draw_wash`; the result is cached and only
//! recomputed when its inputs change.
use crate::oem_scrim::scrim::LocalFrame;
use fh6_core::fit::{FitView, ScrimFit};
use fh6_core::AppConfig;
use tiny_skia::PixmapMut;

/// Lowest visible alpha (of 255).
pub const WASH_ALPHA_MIN: f32 = 12.0;
/// Highest alpha (of 255).
pub const WASH_ALPHA_MAX: f32 = 35.0;
/// Lip width in canvas px.
pub const WASH_LIP_PX: f32 = 1.5;
/// The lip only runs above `center.y - WASH_TOP_FRACTION * b`.
pub const WASH_TOP_FRACTION: f32 = 0.45;

/// The wash's peak alpha (0..=35 of 255) from `AmbientGlowEnabled`, `AmbientGlowIntensity` and
/// the night lift (`NightAmbientGlowEnabled`, `NightAmbientGlowIntensityPct`).
#[must_use]
pub fn wash_alpha(cfg: &AppConfig) -> f32 {
    if !cfg.ambient_glow_enabled {
        return 0.0;
    }
    let mut k = if cfg.ambient_glow_intensity.is_finite() {
        cfg.ambient_glow_intensity.clamp(0.0, 1.0)
    } else {
        0.25
    };
    if cfg.night_ambient_glow_enabled {
        k = k.max((cfg.night_ambient_glow_intensity_pct.clamp(0, 100) as f32) / 100.0);
    }
    WASH_ALPHA_MIN + (WASH_ALPHA_MAX - WASH_ALPHA_MIN) * k
}

/// The horizontal fade along the arc (0 at the ends, 1 in the middle), matching the kit
/// gradient stops 0 / 0.22 / 0.5 / 0.78 / 1 with 0 / 0.6 / 1 / 0.6 / 0.
fn arc_fade(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let t = if t > 0.5 { 1.0 - t } else { t };
    if t < 0.22 {
        0.6 * t / 0.22
    } else {
        0.6 + 0.4 * (t - 0.22) / 0.28
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Key {
    fit: ScrimFit,
    frame: LocalFrame,
    color: (u8, u8, u8),
    alpha: f32,
    size: (u32, u32),
}

/// Cached wash raster (premultiplied RGBA8 over its region of interest).
#[derive(Debug, Default)]
pub struct BinnacleAmbientWash {
    key: Option<Key>,
    roi: (u32, u32, u32, u32),
    buf: Vec<u8>,
}

impl BinnacleAmbientWash {
    /// An empty wash; the first `render` computes it.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Draws the wash for `cfg`'s cockpit view in `color` over a premultiplied pixmap.
    pub fn render_for(&mut self, pixmap: &mut PixmapMut, cfg: &AppConfig, color: (u8, u8, u8)) {
        let fit = ScrimFit::from_config(cfg, FitView::Cockpit);
        let frame = LocalFrame::from_config(cfg, FitView::Cockpit, pixmap.width(), pixmap.height());
        self.render(pixmap, &fit, &frame, color, wash_alpha(cfg));
    }

    /// Draws the wash along `fit`'s top edge. Its alpha scales with the scrim opacity.
    pub fn render(
        &mut self,
        pixmap: &mut PixmapMut,
        fit: &ScrimFit,
        frame: &LocalFrame,
        color: (u8, u8, u8),
        alpha: f32,
    ) {
        let a = alpha.clamp(0.0, WASH_ALPHA_MAX) * fit.opacity.clamp(0.0, 1.0);
        if !fit.visible() || a <= 0.0 {
            return;
        }
        let key = Key {
            fit: *fit,
            frame: *frame,
            color,
            alpha: a,
            size: (pixmap.width(), pixmap.height()),
        };
        if self.key != Some(key) {
            self.recompute(&key);
            self.key = Some(key);
        }
        self.blit(pixmap);
    }

    fn recompute(&mut self, k: &Key) {
        let (cx, cy) = k.fit.center;
        let (sa, sb) = k.fit.semi_axes;
        let unit = k.frame.unit.max(1e-6);
        let sigma_px = (k.fit.feather.0 * 0.12 * unit).max(1.0);
        let reach = (sigma_px * 3.0 / unit).max(4.0);
        let feather = k.fit.feather.0.max(k.fit.feather.1);
        let (w, h) = k.size;
        let to_px = |lx: f32, ly: f32| (k.frame.origin.0 + lx * unit, k.frame.origin.1 + ly * unit);
        let (x0f, y0f) = to_px(cx - sa - feather, cy - sb - reach);
        let (x1f, y1f) = to_px(cx + sa + feather, cy - WASH_TOP_FRACTION * sb);
        let x0 = (x0f.floor().max(0.0) as u32).min(w);
        let y0 = (y0f.floor().max(0.0) as u32).min(h);
        let x1 = (x1f.ceil().max(0.0) as u32).min(w);
        let y1 = (y1f.ceil().max(0.0) as u32).min(h);
        self.roi = (x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0));
        let (rw, rh) = (self.roi.2 as usize, self.roi.3 as usize);
        self.buf.clear();
        self.buf.resize(rw * rh * 4, 0);
        let half_lip = WASH_LIP_PX * 0.5;
        for ry in 0..rh {
            for rx in 0..rw {
                let (px, py) = ((x0 as usize + rx) as f32 + 0.5, (y0 as usize + ry) as f32 + 0.5);
                let (lx, ly) = k.frame.to_local(px, py);
                let d = k.fit.dome_distance(lx, ly) * unit; // canvas px, + outside
                let lip = (half_lip + 0.5 - d.abs()).clamp(0.0, 1.0);
                let halo = if d > 0.0 {
                    0.5 * (-(d * d) / (2.0 * sigma_px * sigma_px)).exp()
                } else {
                    0.0
                };
                let fade = arc_fade((lx - (cx - sa)) / (2.0 * sa));
                let a8 = (k.alpha * fade * lip.max(halo)).round();
                if a8 < 1.0 {
                    continue;
                }
                let i = (ry * rw + rx) * 4;
                let pm = |c: u8| (f32::from(c) * a8 / 255.0).round() as u8;
                if let Some(px) = self.buf.get_mut(i..i + 4) {
                    px.copy_from_slice(&[pm(k.color.0), pm(k.color.1), pm(k.color.2), a8 as u8]);
                }
            }
        }
    }

    fn blit(&self, pixmap: &mut PixmapMut) {
        let w = pixmap.width() as usize;
        let (x0, y0, rw, rh) = (
            self.roi.0 as usize,
            self.roi.1 as usize,
            self.roi.2 as usize,
            self.roi.3 as usize,
        );
        let dst = pixmap.data_mut();
        for ry in 0..rh {
            for rx in 0..rw {
                let s = (ry * rw + rx) * 4;
                let Some(src) = self.buf.get(s..s + 4) else { continue };
                let sa = u32::from(src[3]);
                if sa == 0 {
                    continue;
                }
                let d = ((y0 + ry) * w + x0 + rx) * 4;
                let Some(out) = dst.get_mut(d..d + 4) else { continue };
                let inv = 255 - sa;
                for c in 0..4 {
                    out[c] = (u32::from(src[c]) + u32::from(out[c]) * inv / 255).min(255) as u8;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Pixmap;

    fn cfg() -> AppConfig {
        AppConfig {
            oem_dash_opaque_pct: 100,
            ..AppConfig::default()
        }
    }

    #[test]
    fn alpha_stays_in_the_spec_band() {
        let mut c = cfg();
        assert!((WASH_ALPHA_MIN..=WASH_ALPHA_MAX).contains(&wash_alpha(&c)));
        c.ambient_glow_intensity = 5.0;
        assert_eq!(wash_alpha(&c), WASH_ALPHA_MAX);
        c.ambient_glow_enabled = false;
        assert_eq!(wash_alpha(&c), 0.0);
    }

    #[test]
    fn lip_runs_along_the_dome_top_and_is_never_a_fill() {
        let c = cfg();
        let mut pm = Pixmap::new(3840, 2160).unwrap();
        BinnacleAmbientWash::new().render_for(&mut pm.as_mut(), &c, (56, 189, 248));
        // Dome top at (1920, 1290): the lip is lit, premultiplied, within 12..=35.
        let lip = (1286..1294)
            .filter_map(|y| pm.pixel(1920, y))
            .map(|p| p.alpha())
            .max()
            .unwrap_or(0);
        assert!((12..=35).contains(&lip), "lip alpha {lip}");
        let p = pm.pixel(1920, 1290).unwrap();
        assert!(p.red() <= p.alpha() && p.blue() <= p.alpha());
        assert_eq!(pm.pixel(1920, 1500).unwrap().alpha(), 0, "no fill inside the dome");
        assert_eq!(pm.pixel(1920, 1760).unwrap().alpha(), 0, "no fill at the centre");
        assert_eq!(pm.pixel(1920, 1100).unwrap().alpha(), 0, "nothing far above");
        assert_eq!(pm.pixel(1300, 1700).unwrap().alpha(), 0, "nothing down the sides");
    }

    #[test]
    fn zero_scrim_means_no_wash() {
        let mut c = cfg();
        c.oem_dash_opaque_pct = 0;
        let mut pm = Pixmap::new(960, 540).unwrap();
        BinnacleAmbientWash::new().render_for(&mut pm.as_mut(), &c, (255, 255, 255));
        assert!(pm.data().iter().all(|&v| v == 0));
    }

    #[test]
    fn same_inputs_reuse_the_cache() {
        let c = cfg();
        let mut wash = BinnacleAmbientWash::new();
        let mut pm = Pixmap::new(960, 540).unwrap();
        wash.render_for(&mut pm.as_mut(), &c, (255, 0, 0));
        let ptr = wash.buf.as_ptr();
        wash.render_for(&mut pm.as_mut(), &c, (255, 0, 0));
        assert_eq!(ptr, wash.buf.as_ptr());
        assert!(wash.key.is_some());
    }
}
