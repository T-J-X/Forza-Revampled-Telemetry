//! Cockpit fit: the dome scrim's parameters and each view's cluster placement, resolved from
//! [`AppConfig`] (after any per-car profile overlay, see [`crate::profiles`]).
//!
//! Every length is in 4K reference pixels. The scrim is authored in the cluster's local space
//! (around the anchor, at user scale 1.0), so it moves and scales with the cluster height and
//! scale, as `docs/design/SCRIM_FIT.md` §4 asks.
use crate::config::{AppConfig, SafeZone, HEIGHT_OFFSET_MAX_PX};

/// Smallest and largest effective user scale of a kit cluster.
pub const USER_SCALE_MIN: f32 = 0.70;
/// See [`USER_SCALE_MIN`].
pub const USER_SCALE_MAX: f32 = 1.40;

/// Dome semi-axis a (half width) at width scale 1.0 (SCRIM_FIT §2).
pub const DOME_SEMI_A: f32 = 640.0;
/// Dome semi-axis b (half height) at height scale 1.0.
pub const DOME_SEMI_B: f32 = 470.0;
/// Dome centre relative to the cluster anchor: 75 px above it.
pub const DOME_CENTER_DY: f32 = -75.0;
/// Side feather as a multiple of the top feather (120 / 96).
pub const FEATHER_SIDE_RATIO: f32 = 1.25;
/// Bottom-corner feather as a multiple of the top feather (140 / 96).
pub const FEATHER_BOTTOM_RATIO: f32 = 140.0 / 96.0;
/// The column void starts this far below the anchor (just under the cockpit envelope, +240).
pub const COLUMN_VOID_TOP_DY: f32 = 300.0;
/// Softness of the column void's edge (px).
pub const COLUMN_VOID_FEATHER: f32 = 24.0;

/// The camera view families the overlay places a cluster in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum FitView {
    /// Wheel-less cockpit `[3]`: cockpit cluster plus the dome scrim.
    #[default]
    Cockpit,
    /// Hood `[4]`: hood cluster, scrim forced to 0.
    Hood,
    /// Chase cameras: optional floating HUD, no scrim.
    Chase,
}

impl FitView {
    /// The view for a camera ring index (3 cockpit, 4 hood, others chase).
    #[must_use]
    pub fn from_camera_index(index: i32) -> Self {
        match index {
            3 => Self::Cockpit,
            4 => Self::Hood,
            _ => Self::Chase,
        }
    }
}

/// The dome scrim, resolved and clamped. Lengths in local units (4K px at user scale 1.0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrimFit {
    /// Core opacity 0..=1 (`OemDashOpaquePct / 100`); 0 in hood and chase.
    pub opacity: f32,
    /// Dome centre relative to the cluster anchor.
    pub center: (f32, f32),
    /// Semi-axes (a, b).
    pub semi_axes: (f32, f32),
    /// Superellipse exponent of the upper half (2 round … 4 flat crown).
    pub exponent_top: f32,
    /// Superellipse exponent of the lower half (2 round … 4 square corners).
    pub exponent_bottom: f32,
    /// Feather at the top, the sides and the bottom corners.
    pub feather: (f32, f32, f32),
    /// Column void half width (0 = none).
    pub void_half_width: f32,
    /// Top of the column void relative to the anchor.
    pub void_top: f32,
    /// Softness of the void edge.
    pub void_feather: f32,
}

impl ScrimFit {
    /// The scrim for `view` from `cfg`. Hood and chase get opacity 0 (SCRIM_FIT §4).
    #[must_use]
    pub fn from_config(cfg: &AppConfig, view: FitView) -> Self {
        let opacity = match view {
            FitView::Cockpit => (cfg.oem_dash_opaque_pct.clamp(0, 100) as f32) / 100.0,
            FitView::Hood | FitView::Chase => 0.0,
        };
        let w = finite_clamp(cfg.oem_dash_width_scale, 1.0, 0.5, 1.6);
        let h = finite_clamp(cfg.oem_dash_height_scale, 1.0, 0.5, 1.6);
        let feather = finite_clamp(cfg.oem_dash_feather_px, 96.0, 8.0, 240.0);
        let void_w = finite_clamp(cfg.oem_dash_column_void_px, 220.0, 0.0, 600.0);
        Self {
            opacity,
            center: (
                cfg.oem_dash_offset_x.clamp(-600, 600) as f32,
                DOME_CENTER_DY + cfg.oem_dash_binnacle_offset_y.clamp(-600, 600) as f32,
            ),
            semi_axes: (DOME_SEMI_A * w, DOME_SEMI_B * h),
            exponent_top: exponent(cfg.oem_dash_top_arch),
            exponent_bottom: exponent(cfg.oem_dash_corner_softness),
            feather: (feather, feather * FEATHER_SIDE_RATIO, feather * FEATHER_BOTTOM_RATIO),
            void_half_width: void_w * 0.5,
            void_top: COLUMN_VOID_TOP_DY,
            void_feather: COLUMN_VOID_FEATHER,
        }
    }

    /// Whether anything would be drawn.
    #[must_use]
    pub fn visible(&self) -> bool {
        self.opacity > 0.0
    }

    /// Signed distance to the core edge (negative inside), superellipse first-order estimate,
    /// before the column void. Mirrors `sd_dome` in `fh6-kit`'s `shape.wgsl`.
    #[must_use]
    pub fn dome_distance(&self, x: f32, y: f32) -> f32 {
        let (cx, cy) = self.center;
        let (a, b) = self.semi_axes;
        let n = if y < cy { self.exponent_top } else { self.exponent_bottom };
        let u = ((x - cx) / a).abs().max(1e-6);
        let v = ((y - cy) / b).abs().max(1e-6);
        let s = u.powf(n) + v.powf(n);
        let r = s.powf(1.0 / n);
        // ∇r = r^(1-n) · (u^(n-1)/a, v^(n-1)/b)
        let k = r.powf(1.0 - n);
        let gx = k * u.powf(n - 1.0) / a;
        let gy = k * v.powf(n - 1.0) / b;
        (r - 1.0) / (gx * gx + gy * gy).sqrt().max(1e-6)
    }

    /// The feather width that applies at a point (blend of top, side and bottom by direction).
    #[must_use]
    pub fn feather_at(&self, x: f32, y: f32) -> f32 {
        let (cx, cy) = self.center;
        let (a, b) = self.semi_axes;
        let dx = (x - cx) / a;
        let dy = (y - cy) / b;
        let len = (dx * dx + dy * dy).sqrt().max(1e-6);
        let (side_w, vert) = ((dx / len).abs(), dy / len);
        let (top, side, bottom) = self.feather;
        let vertical = if vert < 0.0 { top } else { bottom };
        side * side_w + vertical * (1.0 - side_w)
    }

    /// Signed distance to the column void (negative inside the void).
    #[must_use]
    pub fn void_distance(&self, x: f32, y: f32) -> f32 {
        let hw = self.void_half_width;
        if hw <= 0.0 {
            return f32::INFINITY;
        }
        let dx = x - self.center.0;
        let cap_y = self.void_top + hw;
        if y < cap_y {
            (dx * dx + (y - cap_y) * (y - cap_y)).sqrt() - hw
        } else {
            dx.abs() - hw
        }
    }

    /// Coverage 0..=1 at a local point (before `opacity`): 1 in the core, a smoothstep fall-off
    /// across the feather, cut by the column void. Mirrors the dome fill in `shape.wgsl`.
    #[must_use]
    pub fn coverage(&self, x: f32, y: f32) -> f32 {
        let d = self.dome_distance(x, y);
        let dome = 1.0 - smoothstep(0.0, self.feather_at(x, y), d);
        let dv = self.void_distance(x, y);
        let f = self.void_feather;
        let void = smoothstep(-f * 0.5, f * 0.5, dv);
        dome * void
    }
}

/// Maps a 0..1 slider (0 = flat/square, 1 = round) to a superellipse exponent 4..2.
#[must_use]
pub fn exponent(slider: f32) -> f32 {
    4.0 - 2.0 * finite_clamp(slider, 0.7, 0.0, 1.0)
}

/// Hermite smoothstep.
#[must_use]
pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    if e1 <= e0 {
        return if x < e0 { 0.0 } else { 1.0 };
    }
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn finite_clamp(v: f32, d: f32, lo: f32, hi: f32) -> f32 {
    if v.is_finite() {
        v.clamp(lo, hi)
    } else {
        d
    }
}

/// Where a view's cluster goes, before the envelope clamp (which needs the envelope from the
/// tokens and happens in `fh6_kit::Layout::from_config`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewPlacement {
    /// The view.
    pub view: FitView,
    /// Anchor in 4K px.
    pub anchor: (f32, f32),
    /// Height offset in px at 4K (positive raises), ±240.
    pub height_offset_px: f32,
    /// Effective user scale (`ClusterScale` × the view's scale), 0.70–1.40.
    pub scale: f32,
    /// Safe zone in px at 4K.
    pub safe: SafeZone,
    /// Whether this view shows a cluster at all (chase HUD can be switched off).
    pub enabled: bool,
}

impl ViewPlacement {
    /// The placement for `view` from `cfg`.
    #[must_use]
    pub fn from_config(cfg: &AppConfig, view: FitView) -> Self {
        let (ax, ay, h, s, safe, enabled) = match view {
            FitView::Cockpit => (
                cfg.cockpit_anchor_x,
                cfg.cockpit_anchor_y,
                cfg.cockpit_height_offset_px,
                cfg.cockpit_cluster_scale,
                cfg.cockpit_safe_zone,
                true,
            ),
            FitView::Hood => (
                cfg.bonnet_anchor_x,
                cfg.bonnet_anchor_y,
                cfg.bonnet_height_offset_px,
                cfg.bonnet_cluster_scale,
                cfg.bonnet_safe_zone,
                true,
            ),
            FitView::Chase => (
                cfg.chase_anchor_x,
                cfg.chase_anchor_y,
                cfg.chase_height_offset_px,
                cfg.chase_cluster_scale,
                cfg.chase_safe_zone,
                cfg.chase_hud_enabled,
            ),
        };
        let global = finite_clamp(cfg.cluster_scale, 1.0, 0.05, 4.0);
        let view_scale = finite_clamp(s, 1.0, 0.05, 4.0);
        Self {
            view,
            anchor: (ax as f32, ay as f32),
            height_offset_px: finite_clamp(h, 0.0, -HEIGHT_OFFSET_MAX_PX, HEIGHT_OFFSET_MAX_PX),
            scale: (global * view_scale).clamp(USER_SCALE_MIN, USER_SCALE_MAX),
            safe,
            enabled,
        }
    }
}

/// Clamps an anchor so `envelope` (local units, scaled by `scale` px per unit, raised by
/// `height_offset` units) stays inside `screen` minus `safe` (all in target px). When the
/// envelope is larger than the safe area on an axis, it is centred on that axis.
///
/// `envelope` is `(min_x, min_y, max_x, max_y)`; `screen` is `(width, height)`; `safe` is the
/// zone already scaled to target px. Returns the clamped anchor in target px.
#[must_use]
pub fn clamp_anchor(
    anchor: (f32, f32),
    envelope: (f32, f32, f32, f32),
    scale: f32,
    height_offset: f32,
    screen: (f32, f32),
    safe: SafeZone,
) -> (f32, f32) {
    let (x0, y0, x1, y1) = envelope;
    let lift = height_offset * scale;
    let axis = |a: f32, lo_off: f32, hi_off: f32, lo: f32, hi: f32| -> f32 {
        // The envelope spans [a + lo_off, a + hi_off]; keep it in [lo, hi].
        let span = hi_off - lo_off;
        if span >= hi - lo {
            return (lo + hi) * 0.5 - (lo_off + hi_off) * 0.5;
        }
        a.clamp(lo - lo_off, hi - hi_off)
    };
    let x = axis(anchor.0, x0 * scale, x1 * scale, safe.side_px, screen.0 - safe.side_px);
    let y = axis(
        anchor.1,
        y0 * scale - lift,
        y1 * scale - lift,
        safe.top_px,
        screen.1 - safe.bottom_px,
    );
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cockpit() -> ScrimFit {
        ScrimFit::from_config(&AppConfig::default(), FitView::Cockpit)
    }

    #[test]
    fn default_dome_matches_scrim_fit_spec() {
        let s = cockpit();
        assert_eq!(s.opacity, 1.0);
        assert_eq!(s.center, (0.0, -75.0), "75 px above the anchor (1920, 1760 at 4K)");
        assert_eq!(s.semi_axes, (640.0, 470.0), "default width works as shipped");
        assert!((s.exponent_top - 2.6).abs() < 1e-5);
        assert!((s.exponent_bottom - 2.6).abs() < 1e-5);
        assert_eq!(s.feather.0, 96.0);
        assert_eq!(s.feather.1, 120.0);
        assert!((s.feather.2 - 140.0).abs() < 1e-3);
    }

    #[test]
    fn dome_shape_test_at_the_envelope() {
        // SCRIM_FIT §2: core half-width 592 at y 1515, 640 at 1760, 541 at 2075 (4K, anchor 1835).
        let s = cockpit();
        for (y4k, half) in [(1515.0, 592.0), (1760.0, 640.0), (2075.0, 541.0)] {
            let y = y4k - 1835.0;
            let d_in = s.dome_distance(half - 6.0, y);
            let d_out = s.dome_distance(half + 6.0, y);
            assert!(d_in < 0.0 && d_out > 0.0, "edge near x={half} at y={y4k}: {d_in} {d_out}");
        }
    }

    #[test]
    fn core_is_opaque_and_feather_falls_to_zero() {
        let s = cockpit();
        assert_eq!(s.coverage(0.0, -75.0), 1.0, "centre is OLED core");
        assert_eq!(s.coverage(0.0, 150.0), 1.0, "behind the cluster");
        // Top edge at local y = -75 - 470 = -545; feather 96 above it.
        assert_eq!(s.coverage(0.0, -545.0 + 5.0), 1.0);
        let mid = s.coverage(0.0, -545.0 - 48.0);
        assert!(mid > 0.3 && mid < 0.7, "half way through the feather: {mid}");
        assert_eq!(s.coverage(0.0, -545.0 - 100.0), 0.0);
        assert_eq!(s.coverage(-1000.0, 0.0), 0.0, "A-pillar vent stays clear");
    }

    #[test]
    fn coverage_is_monotonic_outwards() {
        let s = cockpit();
        let mut prev = 1.0;
        for i in 0..200 {
            let y = -400.0 - i as f32 * 2.0;
            let c = s.coverage(0.0, y);
            assert!(c <= prev + 1e-6, "no banding steps back up at y={y}");
            prev = c;
        }
    }

    #[test]
    fn core_covers_about_80_percent_of_the_envelope() {
        let s = cockpit();
        let (mut inside, mut n) = (0, 0);
        for iy in 0..56 {
            for ix in 0..144 {
                let (x, y) = (-720.0 + ix as f32 * 10.0 + 5.0, -320.0 + iy as f32 * 10.0 + 5.0);
                n += 1;
                if s.dome_distance(x, y) <= 0.0 && s.void_distance(x, y) > 0.0 {
                    inside += 1;
                }
            }
        }
        let share = inside as f32 / n as f32;
        assert!(share > 0.78 && share < 0.92, "core covers {share:.3} of the envelope");
    }

    #[test]
    fn column_void_opens_below_the_cluster_only() {
        let s = cockpit();
        assert_eq!(s.coverage(0.0, 240.0), 1.0, "cluster bottom still on black");
        assert!(s.coverage(0.0, 330.0) < 0.05, "column void below the envelope");
        assert_eq!(s.coverage(200.0, 330.0), 1.0, "void is narrow");
        let none = ScrimFit {
            void_half_width: 0.0,
            ..s
        };
        assert_eq!(none.coverage(0.0, 330.0), 1.0);
    }

    #[test]
    fn hood_and_chase_force_zero() {
        let cfg = AppConfig::default();
        assert!(!ScrimFit::from_config(&cfg, FitView::Hood).visible());
        assert!(!ScrimFit::from_config(&cfg, FitView::Chase).visible());
        let half = AppConfig {
            oem_dash_opaque_pct: 50,
            ..AppConfig::default()
        };
        assert_eq!(ScrimFit::from_config(&half, FitView::Cockpit).opacity, 0.5);
    }

    #[test]
    fn parameters_move_and_shape_the_dome() {
        let cfg = AppConfig {
            oem_dash_offset_x: 40,
            oem_dash_binnacle_offset_y: -30,
            oem_dash_width_scale: 1.2,
            oem_dash_height_scale: 0.8,
            oem_dash_top_arch: 0.0,
            oem_dash_corner_softness: 1.0,
            oem_dash_feather_px: 64.0,
            ..AppConfig::default()
        };
        let s = ScrimFit::from_config(&cfg, FitView::Cockpit);
        assert_eq!(s.center, (40.0, -105.0));
        assert_eq!(s.semi_axes, (768.0, 376.0));
        assert_eq!((s.exponent_top, s.exponent_bottom), (4.0, 2.0));
        assert_eq!(s.feather.0, 64.0);
    }

    #[test]
    fn placement_per_view_and_scale_clamp() {
        let cfg = AppConfig {
            cluster_scale: 1.2,
            cockpit_cluster_scale: 1.5,
            bonnet_height_offset_px: 999.0,
            chase_hud_enabled: false,
            ..AppConfig::default()
        };
        let c = ViewPlacement::from_config(&cfg, FitView::Cockpit);
        assert_eq!(c.anchor, (1920.0, 1835.0));
        assert_eq!(c.scale, USER_SCALE_MAX);
        let h = ViewPlacement::from_config(&cfg, FitView::Hood);
        assert_eq!(h.anchor, (1920.0, 1760.0));
        assert_eq!(h.height_offset_px, 240.0);
        let ch = ViewPlacement::from_config(&cfg, FitView::Chase);
        assert_eq!(ch.anchor, (1920.0, 1900.0));
        assert!(!ch.enabled);
        assert!((ch.scale - 0.96).abs() < 1e-5);
    }

    #[test]
    fn clamp_keeps_envelope_on_screen() {
        let env = (-720.0, -320.0, 720.0, 240.0);
        let safe = SafeZone::default();
        // Default cockpit: already inside.
        assert_eq!(
            clamp_anchor((1920.0, 1835.0), env, 1.0, 0.0, (3840.0, 2160.0), safe),
            (1920.0, 1835.0)
        );
        // Lowered by 240 at scale 1.4: bottom 1835 + 240*1.4 + 240*1.4 would leave the screen.
        let (_, y) = clamp_anchor((1920.0, 1835.0), env, 1.4, -240.0, (3840.0, 2160.0), safe);
        assert!(y + 240.0 * 1.4 + 240.0 * 1.4 <= 2120.0 + 1e-3);
        // Anchor dragged off the left edge.
        let (x, _) = clamp_anchor((100.0, 1835.0), env, 1.0, 0.0, (3840.0, 2160.0), safe);
        assert_eq!(x, 160.0 + 720.0);
        // Wider than the screen: centred.
        let (x, _) = clamp_anchor((0.0, 1000.0), env, 4.0, 0.0, (3840.0, 2160.0), safe);
        assert_eq!(x, 1920.0);
    }
}
