//! LiquidSurface and the `liquidChannel` tach (COMPONENTS.md §16 and §3; family spec
//! `docs/design/families/fluid-generative.md`; tokens `materials.liquidMetal`,
//! `motion.springs.fluidSlosh` and `light`).
//!
//! Liquid metal built from the kit's analytic shapes. The kit has no metaball primitive, so the
//! smooth union is composed: shapes that overlap share one position-based paint (a body arc and a
//! meniscus polygon filled with the same concentric band gradient read as one surface), and where
//! two bodies approach each other the waist of the polynomial smooth minimum (`k` 8–12 px) is
//! solved ([`neck_half_width`]) and bridged with tapered capsules, so a droplet pinches off the
//! meniscus and a falling droplet grows a foot before it merges back.
//!
//! - [`LiquidMetal`]: the dark studio environment bands (floor, softbox line, falloff, chrome
//!   horizon, lower band) derived from the palette ink and tinted 18% toward `primary`, the
//!   Blinn-Phong specular from the virtual light and the fresnel kicker in `primary`.
//! - [`LiquidSurface`]: a free surface (the mercury pool in a well, or a level column) clipped to
//!   a rounded container, tilting `atan(gLat / (1 + |gLong|))` on the under-damped `fluidSlosh`
//!   spring and throwing droplets at the limiter that fall back and merge.
//! - [`LiquidChannel`]: the `liquidChannel` tach: a 240° glass tube (thin rims, no fill) with a
//!   mercury column filling to the rpm fraction, a convex meniscus that tilts with G and surges
//!   with the rpm rate, droplets that detach at the limiter, and a steady hot glow under the
//!   redline end (the family's one steady limiter cue; only the beads strobe).
//!
//! Both accept an optional [`Underlay`] (a Phase 6 flipbook atlas from `assets/fluid/`, or a live
//! GPU fluid dye registered as an image) drawn inside the liquid over the procedural base;
//! everything works fully procedurally without it. Motion runs from `dt` only and comes to rest
//! when the input stops: the flipbook advances only while the liquid moves.
use fh6_kit::math::finite_or;
use fh6_kit::{
    deg, vec2, Align, BlendMode, Cap, Color, FontId, Glow, Gradient, ImageId, ImagePaint, LayerOptions, Painter,
    Palette, Rect, Shape, Signals, Spring, Stroke, TextStyle, Tokens, VAlign, Vec2,
};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// Smooth-union blend radius in local px (COMPONENTS.md §16: `k` 8–12 px).
pub const SMOOTH_K: f32 = 10.0;
/// Channel start (rpm 0) in degrees clockwise from 3 o'clock: the 240° tube is centred on
/// 12 o'clock and opens at the bottom.
pub const CHANNEL_START_DEG: f32 = 150.0;
/// Channel sweep in degrees (COMPONENTS.md §3 `liquidChannel`: a 240° glass tube).
pub const CHANNEL_SWEEP_DEG: f32 = 240.0;
/// Pool tilt gain (family spec Z4: tilt = 0.7 x atan(-gLat / (1 + |gLong|))).
pub const POOL_TILT_GAIN: f32 = 0.7;
/// Pool tilt limit in degrees (family spec Z4: at most 22°).
pub const POOL_MAX_TILT_DEG: f32 = 22.0;
/// Meniscus tilt gain inside the channel (the pool's gain, so every surface leans alike).
pub const MENISCUS_TILT_GAIN: f32 = 0.7;
/// Meniscus tilt limit in degrees (capillary forces keep the cap across the bore).
pub const MENISCUS_MAX_TILT_DEG: f32 = 30.0;
/// Start of the redline sector as a fraction of the limiter rpm (tokens
/// `components.tach.redlineFromRpmPct`).
pub const REDLINE_FROM_RPM_PCT: f32 = 0.92;
/// Scale numerals every 1000 rpm (tokens `components.tach.tickEvery`).
pub const TICK_EVERY_RPM: f32 = 1000.0;
/// Blinn-Phong exponent of the sharp specular (family spec: about 120).
pub const SPECULAR_EXPONENT: f32 = 120.0;
/// Light elevation in degrees (tokens `light.elevationDeg`).
pub const LIGHT_ELEVATION_DEG: f32 = 40.0;
/// Share of `primary` mixed into the environment bands (tokens `materials.liquidMetal.tintMix`).
pub const TINT_MIX: f32 = 0.18;
/// Droplets one surface keeps alive at most.
pub const MAX_DROPLETS: usize = 8;
/// Flipbook rate of the Phase 6 atlases (assets/ASSETS.md: 64 frames at 30 fps).
pub const FLIPBOOK_FPS: f32 = 30.0;
/// A screened underlay's texels are multiplied by the falloff band lifted this far toward the
/// lower band: its brightest texel adds a grey reflection under the softbox line and the
/// specular, so the image never turns the metal into satin aluminium.
const UNDERLAY_LIFT: f32 = 0.3;

/// Environment band levels relative to the palette `ink` (linear luminance ratios of
/// `materials.liquidMetal.envBands`: floor `#0B0D10`, softbox `#E8EEF5`, falloff `#5E6773`,
/// chrome horizon `#14171C`, lower band `#C9D3DE`).
const BAND_FLOOR: f32 = 0.0043;
/// See [`BAND_FLOOR`].
const BAND_SOFTBOX: f32 = 0.917;
/// See [`BAND_FLOOR`].
const BAND_FALLOFF: f32 = 0.144;
/// See [`BAND_FLOOR`].
const BAND_HORIZON: f32 = 0.0092;
/// See [`BAND_FLOOR`].
const BAND_LOWER: f32 = 0.694;
/// Opacity of the fresnel kicker in `primary` at the grazing edge.
const KICKER_ALPHA: f32 = 0.42;
/// Fresnel kicker width (local px).
const KICKER_WIDTH: f32 = 1.5;
/// Share of the kicker kept as a silhouette all round the column (the rim light does the rest).
const KICKER_SILHOUETTE: f32 = 0.3;
/// Share of the kicker the meniscus cap keeps while it faces the light.
const KICKER_CAP_FLOOR: f32 = 0.3;
/// Stops per gradient (the kit's limit).
const GRADIENT_STOPS: usize = 8;
/// Softbox shoulder: width (in softbox line widths) and opacity of the falloff band.
const SHOULDER: (f32, f32) = (2.8, 0.55);

/// Limiter cue switches off this far below the shift rpm (no flicker at the threshold).
const LIMITER_OFF_HYSTERESIS_RPM: f32 = 120.0;
/// Column half width as a share of the tube bore when the caller gives none (16 of 46 px).
const COLUMN_SHARE: f32 = 0.35;
/// Glass wall thickness of the tube (local px): the inner rim sits this far inside the outer.
const TUBE_WALL: f32 = 4.0;
/// Resting meniscus bulge, in column half widths ahead of the wall contact line (mercury does
/// not wet glass: the cap is nearly a half circle).
const MENISCUS_BULGE: f32 = 0.85;
/// Flattest and fullest meniscus bulge while it surges.
const BULGE_RANGE: (f32, f32) = (0.12, 1.0);
/// Meniscus bulge per unit of fill speed (rpm fraction per second).
const SURGE_GAIN: f32 = 0.45;
/// Surge limits (half widths).
const SURGE_RANGE: (f32, f32) = (-0.4, 0.4);
/// Samples across the meniscus cap.
const MENISCUS_SAMPLES: usize = 13;
/// Overlap of the meniscus polygon onto the body arc (px) so their shared paint has no seam.
const MENISCUS_OVERLAP: f32 = 1.0;
/// Samples along each wall of the full column outline (underlay mask).
const WALL_SAMPLES: usize = 32;
/// Samples around the back cap of the column outline.
const BACK_SAMPLES: usize = 9;
/// Points of the full column outline.
const OUTLINE_POINTS: usize = 2 * WALL_SAMPLES + MENISCUS_SAMPLES + BACK_SAMPLES;
/// Points along the softbox line.
const STREAK_POINTS: usize = 28;
/// Softbox line width (family spec: 3–4 px).
const SOFTBOX_WIDTH: f32 = 3.5;
/// Softbox line offset across the column where it faces the light (half widths).
const SOFTBOX_U: f32 = 0.62;
/// Softbox line brightness where the column faces away from the light.
const SOFTBOX_FLOOR: f32 = 0.18;
/// Softbox lobe exponent (an area light: broader than the point specular).
const SOFTBOX_EXPONENT: f32 = 12.0;
/// Softbox line opacity at full brightness (thin streaks, not chrome slabs).
const SOFTBOX_ALPHA: f32 = 0.92;
/// Distance the softbox line stops short of the meniscus front (half widths).
const STREAK_TIP_GAP: f32 = 0.55;
/// Half length of the specular hotspot along the tube (degrees; the exponent-120 lobe falls
/// to half at about 15°).
const SPEC_HALF_SWEEP_DEG: f32 = 22.0;
/// Specular hotspot width (px).
const SPEC_WIDTH: f32 = 4.5;
/// Meniscus cap highlight: inset from the front (px), width (px), opacity.
const CAP_HIGHLIGHT: (f32, f32, f32) = (2.6, 2.4, 0.62);
/// Glass glint half sweep on the outer wall facing the light (degrees).
const GLINT_HALF_SWEEP_DEG: f32 = 30.0;
/// Share of the `glow.medium` lobes the steady limiter glow uses (a cue under the beads' strobe,
/// not a second strobe).
const HOT_GLOW: f32 = 1.0;
/// Redline sector: gap outside the tube and band width (px).
const REDLINE_BAND: (f32, f32) = (5.0, 3.0);
/// Most numerals on the scale.
const MAX_THOUSANDS: u32 = 30;

/// Channel droplets (frame: channel centre, unit = centre-line radius): emission interval (s),
/// inward and forward launch speeds (units/s, base and jitter), radius, life (s, base and
/// jitter), gravity and the lateral felt-force gain (units/s² per g).
const CHANNEL_DROP_INTERVAL: f32 = 0.16;
/// See [`CHANNEL_DROP_INTERVAL`].
const CHANNEL_DROP_INWARD: (f32, f32) = (0.5, 0.3);
/// See [`CHANNEL_DROP_INTERVAL`].
const CHANNEL_DROP_FORWARD: (f32, f32) = (0.1, 0.18);
/// See [`CHANNEL_DROP_INTERVAL`].
const CHANNEL_DROP_RADIUS: f32 = 0.032;
/// See [`CHANNEL_DROP_INTERVAL`].
const CHANNEL_DROP_LIFE: (f32, f32) = (0.42, 0.26);
/// See [`CHANNEL_DROP_INTERVAL`].
const CHANNEL_DROP_GRAVITY: f32 = 1.6;
/// The column throws droplets once it is within this rpm fraction of the shift point.
const DROP_AT_END_SLACK: f32 = 0.03;
/// Pool droplets (frame: surface pivot, unit = container half width): emission interval (s),
/// launch speed (units/s, base and jitter), radius, life cap (s), gravity and lateral gain.
const POOL_DROP_INTERVAL: f32 = 0.22;
/// See [`POOL_DROP_INTERVAL`].
const POOL_DROP_SPEED: (f32, f32) = (0.62, 0.3);
/// See [`POOL_DROP_INTERVAL`].
const POOL_DROP_RADIUS: f32 = 0.045;
/// See [`POOL_DROP_INTERVAL`].
const POOL_DROP_LIFE: f32 = 1.6;
/// See [`POOL_DROP_INTERVAL`].
const POOL_DROP_GRAVITY: f32 = 3.0;
/// Droplets shrink away over this last share of their life.
const DROP_SHRINK: f32 = 0.4;
/// A droplet this far from its frame origin (units) has left the scene.
const DROP_MAX_DISTANCE: f32 = 4.0;
/// Radius standing in for a plane in [`neck_half_width`] (the pool surface).
const PLANE_RADIUS: f32 = 1.0e4;
/// The drawn neck is at most this share of the smaller body's radius.
const NECK_MAX_SHARE: f32 = 0.9;

/// Points across the pool surface.
const SURFACE_POINTS: usize = 25;
/// Standing-wave amplitude per unit of tilt velocity (container half widths per rad/s) and its
/// limit: the surface heaves while it sloshes and is flat at rest.
const WAVE: (f32, f32) = (0.045, 0.05);
/// Width of the primary kicker halo on the pool surface (px).
const POOL_KICKER_WIDTH: f32 = 5.0;
/// Pool specular glint: shift toward the light (container half widths) and half length (px).
const POOL_SPEC: (f32, f32) = (0.32, 26.0);

/// Polynomial smooth minimum of two signed distances (Quilez `smin`, RESEARCH.md): `min(a, b)`
/// where they differ by `k` or more, rounded into a fillet of size `k` where two surfaces meet.
/// `k <= 0` is a hard union.
#[must_use]
pub fn smooth_union(a: f32, b: f32, k: f32) -> f32 {
    if k.is_nan() || k <= 0.0 {
        return a.min(b);
    }
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - h * h * k * 0.25
}

/// Half width of the liquid bridge between two circles of radii `ra` and `rb` whose surfaces
/// are `gap` apart (negative when they overlap) under [`smooth_union`] with blend radius `k`,
/// measured at the waist where both surfaces are equally far. 0 once the bodies have separated
/// (`gap >= k / 2`); a plane is a circle with a very large radius. One circle inside the other
/// returns the smaller radius.
#[must_use]
pub fn neck_half_width(ra: f32, rb: f32, gap: f32, k: f32) -> f32 {
    if !(ra.is_finite() && rb.is_finite() && gap.is_finite()) || ra <= 0.0 || rb <= 0.0 {
        return 0.0;
    }
    let k = finite_or(k, 0.0).max(0.0);
    let d = ra + rb + gap;
    if d <= (ra - rb).abs() {
        return ra.min(rb);
    }
    // The waist plane, measured from each centre.
    let xa = (d + ra - rb) * 0.5;
    let xb = d - xa;
    let field = |y: f32| smooth_union((xa * xa + y * y).sqrt() - ra, (xb * xb + y * y).sqrt() - rb, k);
    if field(0.0) >= 0.0 {
        return 0.0;
    }
    // The field grows with the distance from the axis; at `ra + rb + k` both distances exceed
    // `k`, so the fillet term cannot pull it below zero.
    let (mut lo, mut hi) = (0.0_f32, ra + rb + k);
    for _ in 0..28 {
        let mid = 0.5 * (lo + hi);
        if field(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// The liquid surface tilt for the felt G in the kit's screen convention (radians, clockwise
/// positive). This is COMPONENTS.md's `atan(-gLat / (1 + |gLong|))` measured counter-clockwise:
/// acceleration to the right (`g_lat > 0`) pushes the liquid left, so the surface rises on the
/// left; longitudinal G stiffens it. Non-finite input reads as 0.
#[must_use]
pub fn slosh_angle(g_lat: f32, g_long: f32) -> f32 {
    let g_lat = finite_or(g_lat, 0.0);
    let g_long = finite_or(g_long, 0.0);
    (g_lat / (1.0 + g_long.abs())).atan()
}

/// Where the meniscus front sits along a channel (arc length from rpm 0, px) at `u` across the
/// column (-1 inner wall … +1 outer wall): a convex mercury cap centred on the reading `s` that
/// bulges `bulge` half widths ahead of the wall contact line, turned by `tan_tilt` (positive
/// turns the front clockwise, so the outer wall leads).
#[must_use]
pub fn meniscus_front(s: f32, half_width: f32, bulge: f32, tan_tilt: f32, u: f32) -> f32 {
    let u = u.clamp(-1.0, 1.0);
    let c = (1.0 - u * u).max(0.0).sqrt();
    s - bulge * half_width * (1.0 - c) + u * half_width * tan_tilt
}

/// The Blinn-Phong half vector for a light at screen angle `light` (pointing toward the light)
/// and the token elevation, the viewer straight on: its in-plane part and its z.
fn half_vector(light: f32) -> (Vec2, f32) {
    let el = deg(LIGHT_ELEVATION_DEG);
    let l = Vec2::from_angle(finite_or(light, 0.0)) * el.cos();
    let hz = el.sin() + 1.0;
    let len = (l.dot(l) + hz * hz).sqrt().max(1e-6);
    (l * (1.0 / len), hz / len)
}

/// The reflection of the light across a tube of liquid lying in the screen plane, at the cross
/// section whose outward normal points at `theta` (radians): the peak Blinn-Phong term over the
/// section `N·H` (1 where the section faces the light or faces exactly away from it, both sides
/// of a cylinder catch it) and where it sits across the width (`u`: -1 inner … +1 outer).
#[must_use]
pub fn tube_reflection(theta: f32, light: f32) -> (f32, f32) {
    let (hxy, hz) = half_vector(light);
    let a = hxy.dot(Vec2::from_angle(finite_or(theta, 0.0)));
    let m = (a * a + hz * hz).sqrt().clamp(1e-6, 1.0);
    (m, (a / m).clamp(-1.0, 1.0))
}

/// Brightness of the softbox line at section angle `theta` (`SOFTBOX_FLOOR` … 1).
fn softbox_intensity(theta: f32, light: f32) -> f32 {
    let (m, _) = tube_reflection(theta, light);
    SOFTBOX_FLOOR + (1.0 - SOFTBOX_FLOOR) * m.powf(SOFTBOX_EXPONENT)
}

/// Where the softbox line sits across the column at section angle `theta` (half widths).
fn softbox_u(theta: f32, light: f32) -> f32 {
    let (hxy, _) = half_vector(light);
    let (_, u) = tube_reflection(theta, light);
    (u / hxy.length().max(1e-3) * SOFTBOX_U).clamp(-0.8, 0.8)
}

/// Rim-light weight of a surface whose outward normal is `n` (unit) for the in-plane light
/// direction `to_light` (unit): 1 where it faces away from the light, 0 where it faces it.
fn rim_light(n: Vec2, to_light: Vec2) -> f32 {
    let t = (0.5 - 0.5 * n.dot(to_light)).clamp(0.0, 1.0);
    t * t
}

/// `angle` moved by whole turns into `[from, from + TAU)`.
fn wrap_from(angle: f32, from: f32) -> f32 {
    from + (angle - from).rem_euclid(TAU)
}

/// A deterministic hash of `n` in `[0, 1)` (droplet jitter without a random source).
fn hash01(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// The liquid-metal look (tokens `materials.liquidMetal`): a dark studio environment of bands,
/// a sharp specular and a fresnel kicker. All colours derive from the palette: the bands are the
/// neutral `ink` at the token levels, tinted toward `primary` by `tintMix`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidMetal {
    /// The studio floor: the darkest band, most of the surface.
    pub floor: Color,
    /// The softbox line: the one bright band (drawn 3–4 px wide).
    pub softbox: Color,
    /// The falloff beside the softbox.
    pub falloff: Color,
    /// The chrome horizon: a near-black band.
    pub horizon: Color,
    /// The lower band: the lit floor seen at grazing angles.
    pub lower: Color,
    /// The specular highlight (`ink`).
    pub specular: Color,
    /// The fresnel kicker at the grazing edge (`primary`, translucent).
    pub kicker: Color,
}

impl LiquidMetal {
    /// The material for `palette`.
    #[must_use]
    pub fn from_palette(palette: &Palette) -> Self {
        let ink = palette.roles.ink.with_alpha(1.0);
        let primary = palette.primary.with_alpha(1.0);
        // The tint keeps the band's luminance, so a dark brand primary does not darken the metal.
        let lift = (ink.luminance() / primary.luminance().max(1e-3)).clamp(0.0, 3.0);
        let band = |level: f32| {
            ink.intensity(level)
                .lerp(primary.intensity(level * lift), TINT_MIX)
                .with_alpha(1.0)
        };
        Self {
            floor: band(BAND_FLOOR),
            softbox: band(BAND_SOFTBOX),
            falloff: band(BAND_FALLOFF),
            horizon: band(BAND_HORIZON),
            lower: band(BAND_LOWER),
            specular: palette.roles.ink,
            kicker: palette.primary.fade(KICKER_ALPHA),
        }
    }

    /// Bands across a tube of liquid around `center` spanning radii `inner..outer`: a dark core,
    /// the chrome horizon on both flanks and edges lit by the environment. The gradient is
    /// concentric, so every shape filled with it inside the tube joins without a seam.
    #[must_use]
    pub fn tube_bands(&self, center: Vec2, inner: f32, outer: f32) -> Gradient {
        let outer = outer.max(inner + 1e-3).max(1e-3);
        let mid = (inner + outer) * 0.5;
        let hw = (outer - inner) * 0.5;
        let at = |u: f32| ((mid + u * hw) / outer).clamp(0.0, 1.0);
        Gradient::radial(center, outer)
            .stop(at(-1.0), self.falloff.lerp(self.horizon, 0.3))
            .stop(at(-0.76), self.horizon)
            .stop(at(-0.3), self.floor)
            .stop(at(0.3), self.floor)
            .stop(at(0.7), self.horizon)
            .stop(at(0.9), self.falloff.lerp(self.horizon, 0.45))
            .stop(at(1.0), self.lower.lerp(self.horizon, 0.62))
    }

    /// Bands of a body of liquid below a free surface: from `surface` along `down` (unit) for
    /// `depth` px: a thin falloff under the surface line, the chrome horizon, the dark floor
    /// and a faint lower band where the container wall reflects at the bottom. The surface
    /// itself gets its softbox line separately; above the surface the paint holds the falloff.
    #[must_use]
    pub fn depth_bands(&self, surface: Vec2, down: Vec2, depth: f32) -> Gradient {
        let depth = depth.max(1.0);
        let mut last = 0.0_f32;
        let mut at = |px: f32| {
            last = (px / depth).clamp(last, 1.0);
            last
        };
        let (a, b, c) = (at(3.0), at(12.0), at(depth * 0.4));
        let d = at(depth * 0.85);
        Gradient::linear(surface, surface + down * depth)
            .stop(0.0, self.falloff.lerp(self.horizon, 0.35))
            .stop(a, self.falloff.lerp(self.horizon, 0.7))
            .stop(b, self.horizon)
            .stop(c, self.floor)
            .stop(d, self.horizon)
            .stop(1.0, self.lower.lerp(self.horizon, 0.82))
    }

    /// A droplet (a sphere) of radius `r` at `c`, its bright side toward `to_light` (unit).
    #[must_use]
    pub fn bead(&self, c: Vec2, r: f32, to_light: Vec2) -> Gradient {
        let r = r.max(0.5);
        Gradient::radial(c + to_light * (0.38 * r), 1.38 * r)
            .stop(0.0, self.softbox.lerp(self.falloff, 0.5))
            .stop(0.22, self.falloff.lerp(self.horizon, 0.4))
            .stop(0.52, self.floor)
            .stop(0.82, self.horizon)
            .stop(1.0, self.falloff.lerp(self.horizon, 0.35))
    }
}

/// How an [`Underlay`] combines with the procedural metal. Neither mode can turn the dark
/// studio floor into a bright slab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnderlayBlend {
    /// Multiplied over the finished liquid (bands, softbox line and cap highlight), tinted
    /// `ink`: the image's dark texels run moving ripples through the highlights and its light
    /// texels leave them alone. For bright renders such as the Phase 6 mercury pool.
    #[default]
    Multiply,
    /// Screened over the bands, under the highlights, tinted down to the falloff band: the
    /// image's light adds moving reflections and its dark texels leave the floor black. For
    /// dark renders with highlights, such as the ferrofluid flipbook or a live GPU fluid dye.
    Screen,
}

/// An image inside the liquid over the procedural base: a Phase 6 flipbook atlas from
/// `assets/fluid/` (`mercury_pool_beauty_8x8.png`: 8 x 8 cells, 64 frames at 30 fps, a bright
/// top-down render, so [`UnderlayBlend::Multiply`]), or a live GPU fluid dye registered with
/// `Images::register_view` (one cell, [`UnderlayBlend::Screen`]). Load it once in the cluster's
/// constructor; the liquid works fully procedurally without it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Underlay {
    /// The image.
    pub image: ImageId,
    /// Atlas columns.
    pub columns: u32,
    /// Atlas rows.
    pub rows: u32,
    /// Frames used (at most `columns x rows`, row-major from the top left).
    pub frames: u32,
    /// Frames per second of liquid motion (the flipbook holds still while the liquid rests).
    pub fps: f32,
    /// Strength over the procedural metal (0..=1).
    pub opacity: f32,
    /// How it combines with the metal.
    pub blend: UnderlayBlend,
}

impl Underlay {
    /// A `columns x rows` flipbook atlas at 30 fps, multiplied at opacity 0.6.
    #[must_use]
    pub fn flipbook(image: ImageId, columns: u32, rows: u32) -> Self {
        let columns = columns.max(1);
        let rows = rows.max(1);
        Self {
            image,
            columns,
            rows,
            frames: columns * rows,
            fps: FLIPBOOK_FPS,
            opacity: 0.6,
            blend: UnderlayBlend::Multiply,
        }
    }

    /// A single image (or a live fluid dye), multiplied at opacity 0.6.
    #[must_use]
    pub fn still(image: ImageId) -> Self {
        Self::flipbook(image, 1, 1)
    }

    /// With an opacity.
    #[must_use]
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = finite_or(opacity, 0.0).clamp(0.0, 1.0);
        self
    }

    /// With a blend mode.
    #[must_use]
    pub fn with_blend(mut self, blend: UnderlayBlend) -> Self {
        self.blend = blend;
        self
    }

    /// The atlas cell (texture coordinates, inset by half a percent against mip bleeding)
    /// after `motion_time` seconds of liquid motion.
    #[must_use]
    pub fn cell(&self, motion_time: f32) -> Rect {
        atlas_cell(self.columns, self.rows, self.frames, self.fps, motion_time)
    }

    /// The tint for the blend: `ink` for a multiply (the image keeps its own values), the
    /// falloff band lifted a little for a screen (its brightest texel adds a grey reflection).
    fn tint(&self, mat: &LiquidMetal) -> Color {
        let base = match self.blend {
            UnderlayBlend::Multiply => mat.specular.with_alpha(1.0),
            UnderlayBlend::Screen => mat.falloff.lerp(mat.lower, UNDERLAY_LIFT),
        };
        base.fade(self.opacity)
    }

    /// Fills `shape` with the current cell mapped onto `rect` when this underlay draws at
    /// `stage` (screens go under the highlights, multiplies over them).
    fn fill_at(
        &self,
        stage: UnderlayBlend,
        p: &mut Painter<'_>,
        shape: &Shape<'_>,
        rect: Rect,
        motion_time: f32,
        mat: &LiquidMetal,
    ) {
        if self.blend != stage || self.opacity <= 0.0 {
            return;
        }
        let paint = ImagePaint {
            uv: self.cell(motion_time),
            ..ImagePaint::new(self.image, rect)
        }
        .tint(self.tint(mat));
        let mode = match self.blend {
            UnderlayBlend::Multiply => BlendMode::Multiply,
            UnderlayBlend::Screen => BlendMode::Screen,
        };
        p.scoped(|p| {
            p.set_blend(mode);
            p.fill(shape, paint);
        });
    }
}

/// The texture-coordinate cell of a `columns x rows` flipbook atlas showing `frames` frames
/// (row-major from the top left) at `fps`, after `motion_time` seconds; inset by half a percent
/// of a cell against mip bleeding. Loops after the last frame.
#[must_use]
pub fn atlas_cell(columns: u32, rows: u32, frames: u32, fps: f32, motion_time: f32) -> Rect {
    let cols = columns.max(1);
    let rows = rows.max(1);
    let frames = frames.clamp(1, cols.saturating_mul(rows));
    let frame = (finite_or(motion_time, 0.0).max(0.0) * finite_or(fps, 0.0).max(0.0)).floor() as u64;
    let i = (frame % u64::from(frames)) as u32;
    let (w, h) = (1.0 / cols as f32, 1.0 / rows as f32);
    Rect::from_xywh((i % cols) as f32 * w, (i / cols) as f32 * h, w, h).inset(0.005 * w.min(h))
}

/// One droplet in its owner's normalised frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Droplet {
    pos: Vec2,
    vel: Vec2,
    radius: f32,
    age: f32,
    life: f32,
}

impl Droplet {
    fn alive(&self) -> bool {
        self.age < self.life
    }

    /// The radius, shrinking away over the last part of the life.
    fn shown_radius(&self) -> f32 {
        let left = (1.0 - self.age / self.life.max(1e-6)).clamp(0.0, 1.0);
        self.radius * (left / DROP_SHRINK).min(1.0)
    }
}

/// A fixed pool of ballistic droplets (no allocation). Positions, speeds and radii are in the
/// owner's normalised frame, so the same state draws at any size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Droplets {
    drops: [Droplet; MAX_DROPLETS],
    clock: f32,
    emitting: bool,
    seed: u32,
}

impl Default for Droplets {
    fn default() -> Self {
        Self::new()
    }
}

impl Droplets {
    /// An empty pool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            drops: [Droplet::default(); MAX_DROPLETS],
            clock: 0.0,
            emitting: false,
            seed: 0,
        }
    }

    /// Launches a droplet at `pos` with velocity `vel`, `radius` and `life` seconds, reusing a
    /// free slot or replacing the oldest droplet.
    pub fn spawn(&mut self, pos: Vec2, vel: Vec2, radius: f32, life: f32) {
        let slot = self
            .drops
            .iter()
            .position(|d| !d.alive())
            .or_else(|| {
                self.drops
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.age.total_cmp(&b.1.age))
                    .map(|(i, _)| i)
            })
            .unwrap_or(0);
        self.drops[slot] = Droplet {
            pos: pos.sanitized(),
            vel: vel.sanitized(),
            radius: finite_or(radius, 0.0).max(0.0),
            age: 0.0,
            life: finite_or(life, 0.0).max(0.0),
        };
    }

    /// Advances every droplet `dt` seconds under the acceleration `accel`; droplets that ran out
    /// of life or left the scene die.
    pub fn step(&mut self, dt: f32, accel: Vec2) {
        let dt = finite_or(dt, 0.0).clamp(0.0, 0.25);
        let accel = accel.sanitized();
        for d in self.drops.iter_mut().filter(|d| d.alive()) {
            // Semi-implicit Euler: exact enough for a parabola at display rates.
            d.vel += accel * dt;
            d.pos += d.vel * dt;
            d.age += dt;
            if d.pos.length() > DROP_MAX_DISTANCE {
                d.age = d.life;
            }
        }
    }

    /// Ends the droplets for which `merged(position, radius, velocity)` is true (they merged
    /// back into the liquid).
    pub fn merge_where(&mut self, merged: impl Fn(Vec2, f32, Vec2) -> bool) {
        for d in self.drops.iter_mut().filter(|d| d.alive()) {
            if merged(d.pos, d.radius, d.vel) {
                d.age = d.life;
            }
        }
    }

    /// The live droplets as (position, radius), the radius already shrinking near the end.
    pub fn iter(&self) -> impl Iterator<Item = (Vec2, f32)> + '_ {
        self.drops
            .iter()
            .filter(|d| d.alive())
            .map(|d| (d.pos, d.shown_radius()))
    }

    /// Number of live droplets.
    #[must_use]
    pub fn len(&self) -> usize {
        self.drops.iter().filter(|d| d.alive()).count()
    }

    /// True when no droplet is alive.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes every droplet.
    pub fn clear(&mut self) {
        self.drops = [Droplet::default(); MAX_DROPLETS];
        self.clock = 0.0;
    }

    /// How many droplets are due this frame while emitting every `interval` seconds; the first
    /// one leaves as soon as emission starts.
    fn due(&mut self, dt: f32, interval: f32, emitting: bool) -> u32 {
        if !emitting {
            self.emitting = false;
            self.clock = 0.0;
            return 0;
        }
        let interval = interval.max(1e-3);
        if !self.emitting {
            self.emitting = true;
            self.clock = interval;
        }
        self.clock += finite_or(dt, 0.0).max(0.0);
        let n = (self.clock / interval).floor().clamp(0.0, 2.0);
        self.clock = (self.clock - n * interval).min(interval);
        n as u32
    }

    /// A fresh jitter seed.
    fn next_seed(&mut self) -> u32 {
        self.seed = self.seed.wrapping_add(1);
        self.seed.wrapping_mul(0x2C1B_3C6D)
    }
}

/// The limiter latch: on at the shift rpm, off 120 rpm below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct LimiterLatch {
    on: bool,
}

impl LimiterLatch {
    fn update(&mut self, sig: &Signals) -> bool {
        let shift_rpm = finite_or(sig.shift_rpm, 0.0);
        if shift_rpm < 1.0 {
            self.on = false;
        } else if finite_or(sig.shift_fraction, 0.0) >= 1.0 {
            self.on = true;
        } else if finite_or(sig.rpm, 0.0) < shift_rpm - LIMITER_OFF_HYSTERESIS_RPM {
            self.on = false;
        }
        self.on
    }
}

/// Fills the stroked outline of a tapered capsule pair from body `a` (radius `ra`) to body `b`
/// (radius `rb`) through the smooth-union waist, when the bodies are within `k / 2`.
fn draw_neck(p: &mut Painter<'_>, a: Vec2, ra: f32, b: Vec2, rb: f32, paint: Gradient) {
    let d = b - a;
    let len = d.length();
    if len < 1e-3 {
        return;
    }
    // The round caps at the waist reach `w` in every direction; deep overlaps (a droplet still
    // inside the meniscus) are capped below the smaller body so the bridge never bulges past it.
    let w = neck_half_width(ra, rb, len - ra - rb, SMOOTH_K).min(NECK_MAX_SHARE * ra.min(rb));
    if w <= 0.05 {
        return;
    }
    let dir = d * (1.0 / len);
    let waist = a + dir * ((len + ra - rb) * 0.5).clamp(0.0, len);
    // Capsule ends sit inside each body (a circle of 0.6 r, 0.4 r from the centre, stays inside).
    let pa = a + dir * (0.4 * ra);
    let pb = b - dir * (0.4 * rb);
    p.fill(
        &Shape::Line {
            a: pa,
            b: waist,
            width_a: 1.2 * ra,
            width_b: 2.0 * w,
            cap: Cap::Round,
        },
        paint,
    );
    p.fill(
        &Shape::Line {
            a: waist,
            b: pb,
            width_a: 2.0 * w,
            width_b: 1.2 * rb,
            cap: Cap::Round,
        },
        paint,
    );
}

/// The foot a droplet of radius `r` at `c` pulls up from a free surface through `on_surface`
/// with the upward unit normal `up`, under [`smooth_union`] with a plane: a fillet mound that
/// rises from inside the liquid to the waist, then the thread to the droplet. Nothing once the
/// droplet is `k / 2` clear of the surface. `paint` is the pool body's own gradient, so the
/// mound's root vanishes into the body and only the raised liquid shows. Returns whether a foot
/// was drawn.
fn draw_foot(p: &mut Painter<'_>, on_surface: Vec2, up: Vec2, c: Vec2, r: f32, paint: Gradient) -> bool {
    let height = (c - on_surface).dot(up);
    let gap = height - r;
    let w = neck_half_width(PLANE_RADIUS, r, gap, SMOOTH_K).min(NECK_MAX_SHARE * r);
    if w <= 0.05 || height <= 0.0 {
        return false;
    }
    // The point of the surface under the droplet, the waist halfway across the gap and the
    // mound's root half a blend radius down in the liquid (the pool body hides its lower half).
    let base = c - up * height;
    let waist = base + up * (gap * 0.5).max(0.0);
    let root = base - up * (SMOOTH_K * 0.5);
    let tip = c - up * (0.4 * r);
    p.fill(
        &Shape::Line {
            a: root,
            b: waist,
            width_a: 2.0 * (w + SMOOTH_K * 0.5),
            width_b: 2.0 * w,
            cap: Cap::Round,
        },
        paint,
    );
    p.fill(
        &Shape::Line {
            a: waist,
            b: tip,
            width_a: 2.0 * w,
            width_b: 1.2 * r,
            cap: Cap::Round,
        },
        paint,
    );
    true
}

/// Draws a droplet bead: the banded sphere, its 1 px kicker rim (a rim light on the side away
/// from the light) and the specular dot (the dot blooms when `glint` is a bloom layer request).
fn draw_bead(p: &mut Painter<'_>, c: Vec2, r: f32, mat: &LiquidMetal, to_light: Vec2, glint: Glint) {
    if r < 0.4 {
        return;
    }
    let shape = Shape::circle(c, r);
    p.fill(&shape, mat.bead(c, r, to_light));
    let rim = Gradient::linear(c + to_light * r, c - to_light * r)
        .stop(0.0, mat.kicker.fade(0.1))
        .stop(0.45, mat.kicker.fade(0.25))
        .stop(1.0, mat.kicker);
    p.stroke(&shape, rim, Stroke::rim_px(1.0));
    let dot = Shape::circle(c + to_light * (0.42 * r), (0.24 * r).max(0.8));
    match glint {
        Glint::Bloom => p.layer(LayerOptions::bloom(), |p| p.fill(&dot, mat.specular)),
        Glint::Matte => p.fill(&dot, mat.specular.fade(0.9)),
    }
}

/// Whether specular glints write to the emissive (bloom) buffer: only the tach (glance rank 3)
/// may; the pool sits under the readouts and stays matte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Glint {
    Bloom,
    Matte,
}

/// Where a [`LiquidSurface`] sits, in local px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoolGeometry {
    /// The container; the liquid is clipped to it.
    pub rect: Rect,
    /// Corner radius of the container (half the side for a round well).
    pub corner: f32,
    /// Rest level of the surface (local y); the surface tilts about its middle.
    pub level: f32,
}

impl PoolGeometry {
    /// A container `rect` with rounded corners, the surface resting at `level`.
    #[must_use]
    pub fn new(rect: Rect, corner: f32, level: f32) -> Self {
        Self { rect, corner, level }
    }

    /// A round well of `radius` at `center` with the surface resting `rest_offset` below the
    /// centre (family spec Z4: the gear well R 152 at (0, -40), rest level centre + 96).
    #[must_use]
    pub fn well(center: Vec2, radius: f32, rest_offset: f32) -> Self {
        let radius = finite_or(radius, 0.0).max(0.0);
        Self {
            rect: Rect::from_center_size(center, Vec2::splat(radius * 2.0)),
            corner: radius,
            level: center.y + rest_offset,
        }
    }
}

/// LiquidSurface (COMPONENTS.md §16): a mercury pool with a free surface, clipped to a rounded
/// container. The surface tilts `0.7 x atan(gLat / (1 + |gLong|))` (at most 22°) on the
/// under-damped `fluidSlosh` spring (the target is limited; the spring may overshoot it briefly,
/// which is the slosh) and heaves while it sloshes; at the limiter droplets hop off
/// the surface and merge back through a smooth-union foot. Idle is still.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidSurface {
    tilt: Spring,
    tilt_target: f32,
    gain: f32,
    max_tilt: f32,
    latch: LimiterLatch,
    drops: Droplets,
    motion_time: f32,
}

impl LiquidSurface {
    /// A pool at rest with the family tilt (gain 0.7, at most 22°) on the tokens' `fluidSlosh`
    /// spring.
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            tilt: Spring::new(tokens.springs.fluid_slosh, 0.0),
            tilt_target: 0.0,
            gain: POOL_TILT_GAIN,
            max_tilt: deg(POOL_MAX_TILT_DEG),
            latch: LimiterLatch::default(),
            drops: Droplets::new(),
            motion_time: 0.0,
        }
    }

    /// With another tilt gain and limit (degrees), e.g. a narrow level tube that barely tilts.
    #[must_use]
    pub fn with_tilt(mut self, gain: f32, max_deg: f32) -> Self {
        self.gain = finite_or(gain, POOL_TILT_GAIN).max(0.0);
        self.max_tilt = deg(finite_or(max_deg, POOL_MAX_TILT_DEG).clamp(0.0, 80.0));
        self
    }

    /// Advances with this frame's smoothed G and the limiter (shift rpm, 120 rpm off-hysteresis).
    pub fn update(&mut self, sig: &Signals, dt: f32) {
        let dt = finite_or(dt, 0.0).clamp(0.0, 0.25);
        self.tilt_target = (self.gain * slosh_angle(sig.g_lat, sig.g_long)).clamp(-self.max_tilt, self.max_tilt);
        self.tilt.update(self.tilt_target, dt);
        let limiter = self.latch.update(sig);

        // Droplets: frame origin at the surface pivot, unit = container half width, y down.
        let accel = vec2(-finite_or(sig.g_lat, 0.0) * POOL_DROP_GRAVITY, POOL_DROP_GRAVITY);
        self.drops.step(dt, accel);
        let slope = self.tilt.value().tan();
        // A falling droplet merges once its centre sinks below the (tilted) surface.
        self.drops
            .merge_where(|pos, r, vel| vel.y > 0.0 && pos.y - r * 0.3 > slope * pos.x);
        for _ in 0..self.drops.due(dt, POOL_DROP_INTERVAL, limiter) {
            let seed = self.drops.next_seed();
            let x = (hash01(seed) - 0.5) * 1.1;
            let speed = POOL_DROP_SPEED.0 + POOL_DROP_SPEED.1 * hash01(seed ^ 0x5A5A);
            let side = (hash01(seed ^ 0xA5A5) - 0.5) * 0.5;
            let r = POOL_DROP_RADIUS * (0.75 + 0.5 * hash01(seed ^ 0x3C3C));
            self.drops
                .spawn(vec2(x, slope * x), vec2(side, -speed), r, POOL_DROP_LIFE);
        }

        // The flipbook underlay runs only while the liquid moves.
        let motion = (self.tilt.velocity().abs() * 2.0 + if limiter { 0.6 } else { 0.0 }).min(1.0);
        self.motion_time += dt * motion;
    }

    /// The surface tilt (radians, clockwise positive).
    #[must_use]
    pub fn tilt(&self) -> f32 {
        self.tilt.value()
    }

    /// Live droplets.
    #[must_use]
    pub fn droplet_count(&self) -> usize {
        self.drops.len()
    }

    /// True while the surface sloshes or droplets fly (the host keeps rendering).
    #[must_use]
    pub fn animating(&self) -> bool {
        !self.tilt.settled(self.tilt_target, 1e-4) || !self.drops.is_empty() || self.latch.on
    }

    /// Draws the pool in `geom` lit from `light` (screen angle toward the light, e.g.
    /// `VirtualLight::angle`), with an optional flipbook or fluid `underlay`.
    pub fn draw(
        &self,
        p: &mut Painter<'_>,
        geom: &PoolGeometry,
        palette: &Palette,
        light: f32,
        underlay: Option<&Underlay>,
    ) {
        let rect = geom.rect;
        if rect.is_empty() || rect.width() < 2.0 {
            return;
        }
        let mat = LiquidMetal::from_palette(palette);
        let to_light = Vec2::from_angle(finite_or(light, 0.0));
        let half_w = rect.width() * 0.5;
        let cx = rect.center().x;
        let level = finite_or(geom.level, rect.center().y);
        let tilt = self.tilt.value();
        let slope = tilt.tan();
        let heave = (self.tilt.velocity() * WAVE.0).clamp(-WAVE.1, WAVE.1) * half_w;
        let surface_y = |x: f32| level + slope * (x - cx) + heave * (PI * (x - cx) / half_w).cos();

        // The body: the surface polyline closed far below the container.
        let mut pts = [Vec2::ZERO; SURFACE_POINTS + 2];
        let (x0, x1) = (rect.min.x - 2.0, rect.max.x + 2.0);
        let mut lowest = rect.max.y;
        for (i, pt) in pts.iter_mut().take(SURFACE_POINTS).enumerate() {
            let x = x0 + (x1 - x0) * i as f32 / (SURFACE_POINTS - 1) as f32;
            *pt = vec2(x, surface_y(x));
            lowest = lowest.max(pt.y);
        }
        let bottom = lowest + 8.0;
        pts[SURFACE_POINTS] = vec2(x1, bottom);
        pts[SURFACE_POINTS + 1] = vec2(x0, bottom);
        let pivot = vec2(cx, level);
        let down = vec2(-tilt.sin(), tilt.cos());

        p.save();
        p.clip(rect, geom.corner.max(0.0));
        let body = Shape::Polygon { points: &pts };
        let body_paint = mat.depth_bands(pivot, down, (bottom - level).max(8.0));
        p.fill(&body, body_paint);

        // Droplets close to the surface pull up a smooth-union foot of the body's own liquid,
        // under the surface line so the line rides over the raised liquid.
        let up = vec2(tilt.sin(), -tilt.cos());
        for (pos, r) in self.drops.iter() {
            let c = pivot + pos * half_w;
            let rd = r * half_w;
            if rd >= 0.4 {
                draw_foot(p, vec2(c.x, surface_y(c.x)), up, c, rd, body_paint);
            }
        }
        if let Some(u) = underlay {
            u.fill_at(UnderlayBlend::Screen, p, &body, rect, self.motion_time, &mat);
        }

        // The surface edge: the primary fresnel kicker halo, then the softbox line, brightest
        // under the light.
        let surface = &pts[..SURFACE_POINTS];
        p.fill(
            &Shape::Polyline {
                points: surface,
                width: POOL_KICKER_WIDTH,
                closed: false,
            },
            mat.kicker.fade(0.55),
        );
        let spec_x = cx + to_light.x * POOL_SPEC.0 * half_w;
        let t_spec = ((spec_x - rect.min.x) / rect.width()).clamp(0.2, 0.8);
        let line = Gradient::linear(vec2(rect.min.x, level), vec2(rect.max.x, level))
            .stop(0.0, mat.softbox.fade(0.18))
            .stop(t_spec - 0.18, mat.softbox.fade(0.5))
            .stop(t_spec, mat.softbox.fade(0.92))
            .stop(t_spec + 0.18, mat.softbox.fade(0.5))
            .stop(1.0, mat.softbox.fade(0.18));
        p.fill(
            &Shape::Polyline {
                points: surface,
                width: SOFTBOX_WIDTH * 0.75,
                closed: false,
            },
            line,
        );
        // The specular glint: a tapered streak on the surface under the light.
        let g = vec2(spec_x, surface_y(spec_x));
        let along = vec2(tilt.cos(), tilt.sin()) * POOL_SPEC.1;
        for end in [g - along, g + along] {
            p.fill(
                &Shape::Line {
                    a: g,
                    b: end,
                    width_a: 3.6,
                    width_b: 0.6,
                    cap: Cap::Round,
                },
                mat.specular.fade(0.85),
            );
        }

        if let Some(u) = underlay {
            u.fill_at(UnderlayBlend::Multiply, p, &body, rect, self.motion_time, &mat);
        }

        // The droplets themselves.
        for (pos, r) in self.drops.iter() {
            draw_bead(p, pivot + pos * half_w, r * half_w, &mat, to_light, Glint::Matte);
        }
        p.restore();
    }
}

/// Where a [`LiquidChannel`] sits, in local px (sizes only; the sweep belongs to the tach).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelGeometry {
    /// Centre of the tube's arc.
    pub center: Vec2,
    /// Radius of the tube's inner wall.
    pub inner: f32,
    /// Radius of the tube's outer wall.
    pub outer: f32,
    /// Half width of the liquid column (it floats in the bore: mercury does not wet glass).
    pub column: f32,
}

impl ChannelGeometry {
    /// A tube between radii `inner` and `outer` around `center`; the column fills 70% of the bore.
    #[must_use]
    pub fn new(center: Vec2, inner: f32, outer: f32) -> Self {
        let (inner, outer) = (inner.min(outer).max(0.0), inner.max(outer).max(0.0));
        Self {
            center,
            inner,
            outer,
            column: (outer - inner) * COLUMN_SHARE,
        }
    }

    /// The family layout (KineticLiquidMetal Z3): around the gear well at (0, -40), tube radii
    /// 176–222, column half width 16.
    #[must_use]
    pub fn fluid_family() -> Self {
        Self {
            center: vec2(0.0, -40.0),
            inner: 176.0,
            outer: 222.0,
            column: 16.0,
        }
    }

    /// With another column half width.
    #[must_use]
    pub fn with_column(mut self, half_width: f32) -> Self {
        self.column = finite_or(half_width, 0.0).max(0.0);
        self
    }

    /// With the arc centred on `center`.
    #[must_use]
    pub fn at(mut self, center: Vec2) -> Self {
        self.center = center;
        self
    }

    /// Radius of the tube's centre line.
    #[must_use]
    pub fn radius(&self) -> f32 {
        (self.inner + self.outer) * 0.5
    }

    /// Half the bore (centre line to wall).
    #[must_use]
    pub fn half_bore(&self) -> f32 {
        (self.outer - self.inner).abs() * 0.5
    }

    /// The column half width, kept inside the bore.
    fn column_half_width(&self) -> f32 {
        let bore = self.half_bore().max(1.0);
        self.column.clamp(1.0, bore)
    }
}

/// The column's shape this frame (arc length `s` from rpm 0 along the centre line, `u` across).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Column {
    center: Vec2,
    start: f32,
    r: f32,
    hw: f32,
    tan_t: f32,
    bulge: f32,
    /// The reading: the meniscus front on the centre line.
    s_read: f32,
    /// Where the body arc hands over to the meniscus polygon.
    s_back: f32,
}

impl Column {
    fn angle(&self, s: f32) -> f32 {
        self.start + s / self.r
    }

    fn at(&self, s: f32, u: f32) -> Vec2 {
        Vec2::polar(self.center, self.r + u * self.hw, self.angle(s))
    }

    fn front(&self, u: f32) -> f32 {
        meniscus_front(self.s_read, self.hw, self.bulge, self.tan_t, u)
    }

    /// `u` of meniscus sample `k` (outer wall to inner wall, denser near the walls).
    fn sample_u(k: usize) -> f32 {
        (PI * k as f32 / (MENISCUS_SAMPLES - 1) as f32).cos()
    }

    /// The meniscus polygon: back corners on the walls (overlapping the body) and the cap.
    fn meniscus(&self) -> [Vec2; MENISCUS_SAMPLES + 2] {
        let mut pts = [Vec2::ZERO; MENISCUS_SAMPLES + 2];
        let back = (self.s_back - MENISCUS_OVERLAP).max(0.0);
        pts[0] = self.at(back, 1.0);
        for k in 0..MENISCUS_SAMPLES {
            let u = Self::sample_u(k);
            pts[k + 1] = self.at(self.front(u), u);
        }
        pts[MENISCUS_SAMPLES + 1] = self.at(back, -1.0);
        pts
    }

    /// The whole column outline: outer wall, cap, inner wall, back cap.
    fn outline(&self) -> [Vec2; OUTLINE_POINTS] {
        let mut pts = [Vec2::ZERO; OUTLINE_POINTS];
        let mut i = 0;
        for k in 0..WALL_SAMPLES {
            let s = self.s_back * k as f32 / (WALL_SAMPLES - 1) as f32;
            pts[i] = self.at(s, 1.0);
            i += 1;
        }
        for k in 0..MENISCUS_SAMPLES {
            let u = Self::sample_u(k);
            pts[i] = self.at(self.front(u), u);
            i += 1;
        }
        for k in 0..WALL_SAMPLES {
            let s = self.s_back * (1.0 - k as f32 / (WALL_SAMPLES - 1) as f32);
            pts[i] = self.at(s, -1.0);
            i += 1;
        }
        // The back cap: a semicircle behind rpm 0, from the inner wall round to the outer.
        let back_dir = -Vec2::from_angle(self.start + FRAC_PI_2);
        let base = Vec2::polar(self.center, self.r, self.start);
        let radial = Vec2::from_angle(self.start);
        for k in 0..BACK_SAMPLES {
            let a = PI * (k + 1) as f32 / (BACK_SAMPLES + 1) as f32;
            pts[i] = base + radial * (-a.cos() * self.hw) + back_dir * (a.sin() * self.hw);
            i += 1;
        }
        pts
    }
}

/// The `liquidChannel` tach (COMPONENTS.md §3, Fluid family): a 240° glass tube with thin
/// light-driven rims and no fill, a mercury column filling to the rpm fraction on the `ribbon`
/// spring (feed-forward, no lag), a convex meniscus that tilts with G on `fluidSlosh` and surges
/// with the rpm rate, droplets that detach at the limiter and fall toward the gear well, and a
/// steady hot glow under the redline end while the limiter holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidChannel {
    start: f32,
    sweep: f32,
    level: Spring,
    target: f32,
    tilt: Spring,
    tilt_target: f32,
    surge: Spring,
    surge_target: f32,
    hot: Spring,
    latch: LimiterLatch,
    max_rpm: f32,
    drops: Droplets,
    motion_time: f32,
}

impl LiquidChannel {
    /// An empty 240° channel (rpm 0 at 150°, clockwise) on the tokens' springs.
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            start: deg(CHANNEL_START_DEG),
            sweep: deg(CHANNEL_SWEEP_DEG),
            level: Spring::new(tokens.springs.ribbon, 0.0),
            target: 0.0,
            tilt: Spring::new(tokens.springs.fluid_slosh, 0.0),
            tilt_target: 0.0,
            surge: Spring::new(tokens.springs.fluid_slosh, 0.0),
            surge_target: 0.0,
            hot: Spring::new(tokens.springs.snap, 0.0),
            latch: LimiterLatch::default(),
            max_rpm: 0.0,
            drops: Droplets::new(),
            motion_time: 0.0,
        }
    }

    /// With another start angle and sweep (radians, clockwise; the sweep is kept within 30°..330°).
    #[must_use]
    pub fn with_sweep(mut self, start: f32, sweep: f32) -> Self {
        self.start = finite_or(start, deg(CHANNEL_START_DEG));
        self.sweep = finite_or(sweep, deg(CHANNEL_SWEEP_DEG)).clamp(deg(30.0), deg(330.0));
        self
    }

    /// Advances with this frame's signals: the rpm fraction and rate, the limiter (shift rpm,
    /// 120 rpm off-hysteresis) and the smoothed G.
    pub fn update(&mut self, sig: &Signals, dt: f32) {
        let dt = finite_or(dt, 0.0).clamp(0.0, 0.25);
        let max_rpm = finite_or(sig.max_rpm, 0.0).max(0.0);
        self.max_rpm = max_rpm;
        self.target = finite_or(sig.rpm_fraction, 0.0).clamp(0.0, 1.0);
        let rate = if max_rpm >= 1.0 {
            finite_or(sig.rpm_rate, 0.0) / max_rpm
        } else {
            0.0
        };
        self.level.update_tracking(self.target, rate, dt);

        let max_tilt = deg(MENISCUS_MAX_TILT_DEG);
        self.tilt_target = (MENISCUS_TILT_GAIN * slosh_angle(sig.g_lat, sig.g_long)).clamp(-max_tilt, max_tilt);
        self.tilt.update(self.tilt_target, dt);
        self.surge_target = (self.level.velocity() * SURGE_GAIN).clamp(SURGE_RANGE.0, SURGE_RANGE.1);
        self.surge.update(self.surge_target, dt);

        let limiter = self.latch.update(sig);
        self.hot.update(if limiter { 1.0 } else { 0.0 }, dt);

        // Droplets: frame origin at the channel centre, unit = centre-line radius, y down.
        let accel = vec2(-finite_or(sig.g_lat, 0.0) * CHANNEL_DROP_GRAVITY, CHANNEL_DROP_GRAVITY);
        self.drops.step(dt, accel);
        let tip = self.angle_at(self.fraction());
        // Droplets leave the meniscus only once the column stands at the shift point, so a
        // column still climbing never throws one across the gear well.
        let shift_fraction = if max_rpm >= 1.0 {
            (finite_or(sig.shift_rpm, 0.0) / max_rpm).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let at_end = self.fraction() >= shift_fraction - DROP_AT_END_SLACK;
        for _ in 0..self.drops.due(dt, CHANNEL_DROP_INTERVAL, limiter && at_end) {
            let seed = self.drops.next_seed();
            let out = Vec2::from_angle(tip);
            let forward = Vec2::from_angle(tip + FRAC_PI_2);
            let inward = CHANNEL_DROP_INWARD.0 + CHANNEL_DROP_INWARD.1 * hash01(seed);
            let ahead = CHANNEL_DROP_FORWARD.0 + CHANNEL_DROP_FORWARD.1 * hash01(seed ^ 0x5A5A);
            let r = CHANNEL_DROP_RADIUS * (0.75 + 0.5 * hash01(seed ^ 0x3C3C));
            let life = CHANNEL_DROP_LIFE.0 + CHANNEL_DROP_LIFE.1 * hash01(seed ^ 0xA5A5);
            self.drops.spawn(out, forward * ahead - out * inward, r, life);
        }

        let motion = ((self.level.velocity().abs() * 4.0) + self.tilt.velocity().abs() + self.hot.value()).min(1.0);
        self.motion_time += dt * motion;
    }

    /// The shown rpm fraction, 0..=1.
    #[must_use]
    pub fn fraction(&self) -> f32 {
        self.level.value().clamp(0.0, 1.0)
    }

    /// The steady limiter cue (the hot glow under the redline end), 0..=1.
    #[must_use]
    pub fn limiter(&self) -> f32 {
        self.hot.value().clamp(0.0, 1.0)
    }

    /// The meniscus tilt (radians, clockwise positive).
    #[must_use]
    pub fn tilt(&self) -> f32 {
        self.tilt.value()
    }

    /// The screen angle of rpm `fraction` along the tube.
    #[must_use]
    pub fn angle_at(&self, fraction: f32) -> f32 {
        self.start + self.sweep * finite_or(fraction, 0.0).clamp(0.0, 1.0)
    }

    /// Live droplets.
    #[must_use]
    pub fn droplet_count(&self) -> usize {
        self.drops.len()
    }

    /// The live droplets in local px for `geom` as (centre, radius), e.g. to splat them into a
    /// live fluid or to hand them to a pool.
    pub fn droplets(&self, geom: &ChannelGeometry) -> impl Iterator<Item = (Vec2, f32)> + '_ {
        let (center, r) = (geom.center, geom.radius());
        self.drops.iter().map(move |(pos, rad)| (center + pos * r, rad * r))
    }

    /// True while the column moves, the meniscus sloshes or droplets fly.
    #[must_use]
    pub fn animating(&self) -> bool {
        !self.level.settled(self.target, 1e-4)
            || !self.tilt.settled(self.tilt_target, 1e-4)
            || !self.surge.settled(self.surge_target, 1e-3)
            || !self.hot.settled(if self.latch.on { 1.0 } else { 0.0 }, 1e-3)
            || !self.drops.is_empty()
            || self.latch.on
    }

    /// The column's shape for `geom`.
    fn column(&self, geom: &ChannelGeometry) -> Column {
        let r = geom.radius().max(1.0);
        let hw = geom.column_half_width();
        let tan_t = self.tilt.value().tan().clamp(-1.0, 1.0);
        let bulge = (MENISCUS_BULGE + self.surge.value()).clamp(BULGE_RANGE.0, BULGE_RANGE.1);
        let lead = hw * (1.0 + tan_t.abs());
        // Even at rpm 0 a bead of mercury rests in the closed end of the tube.
        let s_read = (self.fraction() * self.sweep * r).max(lead + 1.0);
        Column {
            center: geom.center,
            start: self.start,
            r,
            hw,
            tan_t,
            bulge,
            s_read,
            s_back: s_read - lead,
        }
    }

    /// Draws the channel in `geom`, clipped to `clip`, lit from `light` (screen angle toward the
    /// light, e.g. `VirtualLight::angle`), with an optional flipbook or fluid `underlay`.
    pub fn draw(
        &self,
        p: &mut Painter<'_>,
        geom: &ChannelGeometry,
        clip: Rect,
        palette: &Palette,
        light: f32,
        underlay: Option<&Underlay>,
    ) {
        let mat = LiquidMetal::from_palette(palette);
        let light = finite_or(light, deg(-135.0));
        let col = self.column(geom);
        p.save();
        p.clip(clip, 0.0);
        self.draw_hot_glow(p, geom, palette);
        self.draw_column(p, &col, &mat, light, underlay, geom);
        self.draw_droplets(p, &col, &mat, light);
        self.draw_tube(p, geom, palette, light);
        p.restore();
    }

    /// The steady limiter cue: the redline end of the bore glows in `shift` under the mercury.
    /// It is drawn first and stays out of the bloom buffer, so the hot glass shows round the
    /// column and past its end while the liquid itself stays dark metal (the beads strobe).
    fn draw_hot_glow(&self, p: &mut Painter<'_>, geom: &ChannelGeometry, palette: &Palette) {
        let k = self.limiter();
        if k <= 2e-3 {
            return;
        }
        let from = self.angle_at(REDLINE_FROM_RPM_PCT);
        let seg = Shape::arc(
            geom.center,
            geom.radius(),
            geom.half_bore() * 2.0,
            from,
            self.sweep * (1.0 - REDLINE_FROM_RPM_PCT),
            Cap::Round,
        );
        let lobes = p.tokens().glow.medium;
        for lobe in lobes.lobes() {
            p.glow(
                &seg,
                palette.shift,
                Glow {
                    sigma: lobe.sigma,
                    intensity: lobe.intensity * k * HOT_GLOW,
                    fill_inside: true,
                },
            );
        }
    }

    /// The mercury column: body and meniscus, kicker, softbox line, cap highlight, specular.
    fn draw_column(
        &self,
        p: &mut Painter<'_>,
        col: &Column,
        mat: &LiquidMetal,
        light: f32,
        underlay: Option<&Underlay>,
        geom: &ChannelGeometry,
    ) {
        let to_light = Vec2::from_angle(light);
        let bands = mat.tube_bands(col.center, col.r - col.hw, col.r + col.hw);
        let body = Shape::arc(
            col.center,
            col.r,
            col.hw * 2.0,
            col.start,
            col.s_back / col.r,
            Cap::Round,
        );
        p.fill(&body, bands);
        // A faint silhouette round the walls and the back cap (the body's tip cap lies inside
        // the meniscus, which covers its part of the stroke).
        p.stroke(&body, mat.kicker.fade(KICKER_SILHOUETTE), Stroke::inner(1.0));
        let meniscus = col.meniscus();
        p.fill(&Shape::Polygon { points: &meniscus }, bands);
        let outline = col.outline();
        let square = Rect::from_center_size(geom.center, Vec2::splat(geom.outer * 2.0));
        if let Some(u) = underlay {
            u.fill_at(
                UnderlayBlend::Screen,
                p,
                &Shape::Polygon { points: &outline },
                square,
                self.motion_time,
                mat,
            );
        }
        // The fresnel kicker on each wall where it turns away from the light: a rim light that
        // moves with the light, not an outline.
        let wall_sweep = (col.s_back - MENISCUS_OVERLAP).max(0.0) / col.r;
        if wall_sweep > 1e-3 {
            let cs = col.start - 0.01;
            for side in [1.0_f32, -1.0] {
                let mut paint = Gradient::conic(col.center, cs);
                for k in 0..GRADIENT_STOPS {
                    let a = col.start + wall_sweep * k as f32 / (GRADIENT_STOPS - 1) as f32;
                    let weight = rim_light(Vec2::from_angle(a) * side, to_light);
                    paint = paint.stop((a - cs) / TAU, mat.kicker.fade(weight));
                }
                let radius = col.r + side * (col.hw - KICKER_WIDTH * 0.5);
                p.fill(
                    &Shape::arc(col.center, radius, KICKER_WIDTH, col.start, wall_sweep, Cap::Butt),
                    paint,
                );
            }
        }
        // The kicker round the cap, inset half its width, by how far the cap turns from the light.
        let tip_angle = col.angle(col.s_read);
        let cap_normal = Vec2::from_angle(tip_angle + FRAC_PI_2 + col.tan_t.atan());
        let cap_kicker = mat
            .kicker
            .fade(KICKER_CAP_FLOOR + (1.0 - KICKER_CAP_FLOOR) * rim_light(cap_normal, to_light));
        let inset = KICKER_WIDTH * 0.5;
        let edge_u = 1.0 - inset / col.hw;
        let mut rim = [Vec2::ZERO; MENISCUS_SAMPLES + 2];
        let back = (col.s_back - MENISCUS_OVERLAP).max(0.0);
        rim[0] = col.at(back, edge_u);
        for (k, pt) in rim.iter_mut().skip(1).take(MENISCUS_SAMPLES).enumerate() {
            let u = Column::sample_u(k) * edge_u;
            *pt = col.at(col.front(u) - inset, u);
        }
        rim[MENISCUS_SAMPLES + 1] = col.at(back, -edge_u);
        p.fill(
            &Shape::Polyline {
                points: &rim,
                width: KICKER_WIDTH,
                closed: false,
            },
            cap_kicker,
        );

        // The softbox line: it crosses the column with the light and brightens where the
        // section faces it (both flanks of a cylinder catch it).
        let s1 = col.front(softbox_u(col.angle(col.s_read), light)) - STREAK_TIP_GAP * col.hw;
        if s1 > 2.0 {
            let mut streak = [Vec2::ZERO; STREAK_POINTS];
            for (j, pt) in streak.iter_mut().enumerate() {
                let s = s1 * j as f32 / (STREAK_POINTS - 1) as f32;
                *pt = col.at(s, softbox_u(col.angle(s), light));
            }
            let (a0, a1) = (col.angle(0.0), col.angle(s1));
            let cs = a0 - 0.01;
            let mut line = Gradient::conic(col.center, cs);
            let mut shoulder = Gradient::conic(col.center, cs);
            for k in 0..GRADIENT_STOPS {
                let a = a0 + (a1 - a0) * k as f32 / (GRADIENT_STOPS - 1) as f32;
                let i = softbox_intensity(a, light);
                line = line.stop((a - cs) / TAU, mat.softbox.fade(SOFTBOX_ALPHA * i));
                shoulder = shoulder.stop((a - cs) / TAU, mat.falloff.fade(SHOULDER.1 * i));
            }
            // The falloff band either side of the softbox: the line sits on a curved surface.
            p.fill(
                &Shape::Polyline {
                    points: &streak,
                    width: SOFTBOX_WIDTH * SHOULDER.0,
                    closed: false,
                },
                shoulder,
            );
            p.fill(
                &Shape::Polyline {
                    points: &streak,
                    width: SOFTBOX_WIDTH,
                    closed: false,
                },
                line,
            );
        }

        // The meniscus cap catches the softbox along its front, brighter when it faces the light.
        let facing = to_light.dot(cap_normal).max(0.0);
        let (cap_inset, cap_width, cap_alpha) = CAP_HIGHLIGHT;
        let mut cap = [Vec2::ZERO; MENISCUS_SAMPLES - 4];
        for (k, pt) in cap.iter_mut().enumerate() {
            let u = Column::sample_u(k + 2) * 0.92;
            *pt = col.at(col.front(u) - cap_inset, u);
        }
        p.fill(
            &Shape::Polyline {
                points: &cap,
                width: cap_width,
                closed: false,
            },
            mat.softbox.fade(cap_alpha * (0.3 + 0.7 * facing)),
        );

        // The sharp specular (exponent 120) where a section faces the light or faces exactly
        // away from it, on the softbox line, and the glint on the cap.
        let (a0, a1) = (col.angle(0.0), col.angle((col.s_read - col.hw).max(0.0)));
        let half = deg(SPEC_HALF_SWEEP_DEG);
        p.layer(LayerOptions::bloom(), |p| {
            for peak in [light, light + PI] {
                let peak = wrap_from(peak, a0 - half);
                let from = (peak - half).max(a0);
                let to = (peak + half).min(a1);
                if to - from < deg(3.0) {
                    continue;
                }
                let radius = col.r + softbox_u(peak, light) * col.hw;
                let cs = peak - half - 0.01;
                let lobe = |a: f32| {
                    let (m, _) = tube_reflection(a, light);
                    mat.specular.fade(m.powf(SPECULAR_EXPONENT))
                };
                let mut paint = Gradient::conic(col.center, cs);
                for k in 0..7 {
                    let a = peak - half + 2.0 * half * k as f32 / 6.0;
                    paint = paint.stop((a - cs) / TAU, lobe(a));
                }
                p.fill(
                    &Shape::arc(col.center, radius, SPEC_WIDTH, from, to - from, Cap::Round),
                    paint,
                );
            }
            if facing > 0.05 {
                let cap_center = col.at(col.s_read - col.hw * (0.35 + 0.5 * col.bulge), 0.0);
                let dot = cap_center + to_light * (0.5 * col.hw);
                p.fill(
                    &Shape::circle(dot, (0.2 * col.hw).max(1.0)),
                    mat.specular.fade(0.4 + 0.6 * facing),
                );
            }
        });
        if let Some(u) = underlay {
            u.fill_at(
                UnderlayBlend::Multiply,
                p,
                &Shape::Polygon { points: &outline },
                square,
                self.motion_time,
                mat,
            );
        }
    }

    /// Droplets: the smooth-union neck while they pinch off the meniscus, then the bead.
    fn draw_droplets(&self, p: &mut Painter<'_>, col: &Column, mat: &LiquidMetal, light: f32) {
        if self.drops.is_empty() {
            return;
        }
        let to_light = Vec2::from_angle(light);
        let tip = col.at(col.s_read - col.hw, 0.0);
        let bands = mat.tube_bands(col.center, col.r - col.hw, col.r + col.hw);
        for (pos, rad) in self.drops.iter() {
            let c = col.center + pos * col.r;
            let rd = rad * col.r;
            if rd < 0.4 {
                continue;
            }
            draw_neck(p, tip, col.hw, c, rd, bands);
            draw_bead(p, c, rd, mat, to_light, Glint::Bloom);
        }
    }

    /// The glass tube: a 1 px outer rim lit toward the light, a fainter inner (wall) rim, the
    /// glint on the outer wall facing the light, and the redline sector just outside.
    fn draw_tube(&self, p: &mut Painter<'_>, geom: &ChannelGeometry, palette: &Palette, light: f32) {
        let roles = palette.roles;
        let r = geom.radius();
        let bore = geom.half_bore();
        let to_light = Vec2::from_angle(light);
        let tube = Shape::arc(geom.center, r, bore * 2.0, self.start, self.sweep, Cap::Round);
        let rim = Gradient::linear(geom.center + to_light * geom.outer, geom.center - to_light * geom.outer)
            .stop(0.0, roles.rim_specular.fade(0.62))
            .stop(0.45, roles.hairline)
            .stop(1.0, roles.hairline.fade(0.4));
        p.stroke(&tube, rim, Stroke::rim_px(1.0));
        if bore > TUBE_WALL * 2.0 {
            let wall = Shape::arc(
                geom.center,
                r,
                (bore - TUBE_WALL) * 2.0,
                self.start,
                self.sweep,
                Cap::Round,
            );
            p.stroke(&wall, roles.hairline.fade(0.32), Stroke::rim_px(1.0));
        }

        // The glass catches the light on its outer wall.
        let half = deg(GLINT_HALF_SWEEP_DEG);
        let peak = wrap_from(light, self.start - half);
        let from = (peak - half).max(self.start);
        let to = (peak + half).min(self.start + self.sweep);
        if to - from > deg(4.0) {
            let cs = peak - half - 0.01;
            let at = |a: f32| (a - cs) / TAU;
            let paint = Gradient::conic(geom.center, cs)
                .stop(at(peak - half), roles.rim_specular.fade(0.0))
                .stop(at(peak - half * 0.4), roles.rim_specular.fade(0.35))
                .stop(at(peak), roles.rim_specular.fade(0.8))
                .stop(at(peak + half * 0.4), roles.rim_specular.fade(0.35))
                .stop(at(peak + half), roles.rim_specular.fade(0.0));
            p.fill(
                &Shape::arc(geom.center, geom.outer - 2.5, 2.0, from, to - from, Cap::Round),
                paint,
            );
        }

        // The redline sector: a thin warning band just outside the tube.
        let (gap, band) = REDLINE_BAND;
        p.fill(
            &Shape::arc(
                geom.center,
                geom.outer + gap + band * 0.5,
                band,
                self.angle_at(REDLINE_FROM_RPM_PCT),
                self.sweep * (1.0 - REDLINE_FROM_RPM_PCT),
                Cap::Butt,
            ),
            roles.warning.fade(0.75),
        );
    }

    /// Draws the thousands numerals outside the tube (`cap` px tall, label ink; text-safe warning
    /// in the redline sector). Nothing is drawn until the limiter rpm is known.
    pub fn draw_scale(&self, p: &mut Painter<'_>, geom: &ChannelGeometry, font: FontId, cap: f32, palette: &Palette) {
        let max = self.max_rpm;
        if max < TICK_EVERY_RPM {
            return;
        }
        let thousands = ((max / TICK_EVERY_RPM).floor() as u32).min(MAX_THOUSANDS);
        let base = TextStyle::new(font, cap)
            .paint(palette.roles.ink_label)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        let red = base.paint(palette.roles.warning.text_safe());
        let radius = geom.outer + REDLINE_BAND.0 + REDLINE_BAND.1 + cap * 1.3;
        for k in 0..=thousands {
            let rpm = k as f32 * TICK_EVERY_RPM;
            let style = if rpm >= REDLINE_FROM_RPM_PCT * max - 1.0 {
                &red
            } else {
                &base
            };
            let at = Vec2::polar(geom.center, radius, self.angle_at(rpm / max));
            p.number(at, i64::from(k), if k >= 10 { 2 } else { 1 }, style, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_kit::Gear;

    const DT: f32 = 1.0 / 60.0;

    fn tokens() -> Tokens {
        Tokens::default()
    }

    fn signals(rpm: f32, g_lat: f32, g_long: f32) -> Signals {
        Signals {
            race_on: true,
            rpm,
            max_rpm: 8000.0,
            shift_rpm: 7600.0,
            rpm_fraction: rpm / 8000.0,
            shift_fraction: rpm / 7600.0,
            gear: Gear::Forward(4),
            g_lat,
            g_long,
            ..Signals::default()
        }
    }

    #[test]
    fn smooth_union_is_min_apart_and_fillets_close() {
        let k = SMOOTH_K;
        assert_eq!(smooth_union(3.0, 20.0, k), 3.0);
        assert_eq!(smooth_union(-4.0, 9.0, k), -4.0);
        assert!((smooth_union(2.0, 2.0, k) - (2.0 - k / 4.0)).abs() < 1e-5);
        for (a, b) in [(0.5, 1.5), (-1.0, 3.0), (4.0, 4.5)] {
            let s = smooth_union(a, b, k);
            assert!(s <= a.min(b) + 1e-6, "smin never exceeds min: {a} {b} -> {s}");
            assert!((s - smooth_union(b, a, k)).abs() < 1e-6, "symmetric");
        }
        assert_eq!(smooth_union(1.0, 2.0, 0.0), 1.0, "k 0 is a hard union");
    }

    #[test]
    fn neck_pinches_off_at_half_k() {
        let k = SMOOTH_K;
        assert_eq!(neck_half_width(16.0, 6.0, k * 0.5 + 0.01, k), 0.0, "separated");
        assert_eq!(neck_half_width(16.0, 6.0, 30.0, k), 0.0);
        let close = neck_half_width(16.0, 6.0, 0.5, k);
        let touching = neck_half_width(16.0, 6.0, 0.0, k);
        let overlap = neck_half_width(16.0, 6.0, -4.0, k);
        assert!(close > 0.0, "a bridge forms within k/2: {close}");
        assert!(touching > close && overlap > touching, "the neck widens as they merge");
        // Just before pinch-off the droplet hangs by a thread much thinner than itself.
        let thread = neck_half_width(16.0, 6.0, k * 0.49, k);
        assert!(
            thread > 0.0 && thread < 3.0,
            "a separating droplet hangs by a thread: {thread}"
        );
        assert!(thread < close, "the thread thins as the droplet leaves");
        // Overlapping by 4 px the waist plane lies 14 px from the big centre, inside it: the
        // smooth union is at least the hard union's chord there and adds at most `k`.
        let chord = (16.0_f32 * 16.0 - 14.0 * 14.0).sqrt();
        assert!(
            overlap >= chord - 1e-3 && overlap <= chord + k,
            "overlap neck {overlap} vs chord {chord}"
        );
        assert!((neck_half_width(4.0, 20.0, -30.0, k) - 4.0).abs() < 1e-6, "contained");
        // A droplet over a plane grows a foot as it lands.
        let foot = neck_half_width(6.0, PLANE_RADIUS, 1.0, k);
        assert!(foot > 0.0 && foot < 6.0 + k, "plane foot {foot}");
        assert_eq!(neck_half_width(f32::NAN, 6.0, 1.0, k), 0.0);
    }

    #[test]
    fn slosh_tilts_away_from_the_turn() {
        assert_eq!(slosh_angle(0.0, 0.0), 0.0);
        let right = slosh_angle(1.0, 0.0);
        assert!((right - std::f32::consts::FRAC_PI_4).abs() < 1e-5);
        assert!(slosh_angle(-1.0, 0.0) < 0.0);
        assert!(slosh_angle(1.0, -1.0) < right, "braking stiffens the surface");
        assert!((slosh_angle(1.0, -1.0) - slosh_angle(1.0, 1.0)).abs() < 1e-6, "|gLong|");
        assert_eq!(slosh_angle(f32::NAN, f32::INFINITY), 0.0);
    }

    #[test]
    fn meniscus_front_reads_the_rpm_and_leads_with_tilt() {
        let (s, hw) = (400.0, 16.0);
        assert!(
            (meniscus_front(s, hw, 0.55, 0.3, 0.0) - s).abs() < 1e-4,
            "centre line is the reading"
        );
        let outer = meniscus_front(s, hw, 0.55, 0.3, 1.0);
        let inner = meniscus_front(s, hw, 0.55, 0.3, -1.0);
        assert!(outer > inner, "positive tilt: the outer wall leads");
        assert!(
            meniscus_front(s, hw, 0.55, 0.0, 0.7) < s,
            "convex cap: the walls trail the centre"
        );
    }

    #[test]
    fn reflection_catches_both_flanks() {
        let light = deg(-135.0);
        let (peak, u) = tube_reflection(light, light);
        assert!(
            (peak - 1.0).abs() < 1e-4 && u > 0.35,
            "facing the light: outer side, u {u}"
        );
        let (back, u_back) = tube_reflection(light + PI, light);
        assert!((back - 1.0).abs() < 1e-4 && u_back < -0.35, "facing away: inner side");
        let (side, u_side) = tube_reflection(light + FRAC_PI_2, light);
        assert!(
            side.powf(SPECULAR_EXPONENT) < 1e-3 && u_side.abs() < 1e-4,
            "no hotspot side-on"
        );
        // The exponent-120 lobe is half bright about 15° off the peak.
        let (m, _) = tube_reflection(light + deg(15.0), light);
        let half = m.powf(SPECULAR_EXPONENT);
        assert!((0.3..0.7).contains(&half), "lobe at 15°: {half}");
        assert!(softbox_intensity(light, light) > softbox_intensity(light + FRAC_PI_2, light));
    }

    #[test]
    fn material_bands_are_dark_with_one_bright_line() {
        let t = tokens();
        let pal = Palette::family(&t, "fluidGenerative");
        let m = LiquidMetal::from_palette(&pal);
        let l = |c: Color| c.luminance();
        assert!(l(m.floor) < l(m.horizon) && l(m.horizon) < l(m.falloff));
        assert!(l(m.falloff) < l(m.lower) && l(m.lower) < l(m.softbox));
        assert!(l(m.floor) < 0.01, "the studio floor is near black");
        assert!((m.softbox.a - 1.0).abs() < 1e-6, "bands are opaque");
        // The tint leans toward primary (fluidGenerative primary is a cool blue).
        let (r, _, b) = m.softbox.straight_rgb();
        let (ir, _, ib) = pal.roles.ink.straight_rgb();
        assert!(b / r > ib / ir, "tinted toward primary");
    }

    #[test]
    fn pool_sloshes_under_damped_and_settles() {
        let t = tokens();
        let mut pool = LiquidSurface::new(&t);
        // 0.4 g to the right: 0.7 x atan(0.4) = 15.3°, inside the 22° limit.
        let corner = signals(3000.0, 0.4, 0.0);
        let mut peak = 0.0_f32;
        for _ in 0..90 {
            pool.update(&corner, DT);
            peak = peak.max(pool.tilt());
        }
        let target = POOL_TILT_GAIN * slosh_angle(0.4, 0.0);
        assert!(target < deg(POOL_MAX_TILT_DEG));
        assert!(pool.tilt() > 0.0, "the surface rises on the left in a right turn");
        assert!(peak > target * 1.05, "fluidSlosh overshoots: {peak} vs {target}");
        // A hard corner settles on the 22° limit.
        let limit = deg(POOL_MAX_TILT_DEG);
        let mut hard = LiquidSurface::new(&t);
        for _ in 0..240 {
            hard.update(&signals(3000.0, 3.0, 0.0), DT);
        }
        assert!((hard.tilt() - limit).abs() < 1e-3, "limited to 22°: {}", hard.tilt());
        let mut left = LiquidSurface::new(&t);
        for _ in 0..240 {
            left.update(&signals(3000.0, -3.0, 0.0), DT);
        }
        assert!((left.tilt() + limit).abs() < 1e-3, "and to -22° the other way");
        for _ in 0..600 {
            pool.update(&signals(900.0, 0.0, 0.0), DT);
        }
        assert!(pool.tilt().abs() < 1e-3 && !pool.animating(), "idle is still");
    }

    #[test]
    fn pool_droplets_hop_at_the_limiter_and_merge_back() {
        let t = tokens();
        let mut pool = LiquidSurface::new(&t);
        for _ in 0..30 {
            pool.update(&signals(7990.0, 0.0, 0.0), DT);
        }
        assert!(pool.droplet_count() > 0, "droplets at the limiter");
        assert!(pool.animating());
        // Off the limiter (past the hysteresis) they land and merge; nothing moves after.
        for _ in 0..180 {
            pool.update(&signals(5000.0, 0.0, 0.0), DT);
        }
        assert_eq!(pool.droplet_count(), 0);
        assert!(!pool.animating());
    }

    #[test]
    fn channel_tracks_rpm_and_holds_the_limiter_with_hysteresis() {
        let t = tokens();
        let mut ch = LiquidChannel::new(&t);
        for _ in 0..30 {
            ch.update(&signals(4000.0, 0.0, 0.0), DT);
        }
        assert!((ch.fraction() - 0.5).abs() < 0.01, "fill {}", ch.fraction());
        assert_eq!(ch.limiter(), 0.0);
        for _ in 0..30 {
            ch.update(&signals(7700.0, 0.0, 0.0), DT);
        }
        assert!(ch.limiter() > 0.95, "hot glow at the limiter: {}", ch.limiter());
        assert!(ch.droplet_count() > 0, "droplets detach at the limiter");
        // 60 rpm under the shift point: still inside the hysteresis.
        for _ in 0..10 {
            ch.update(&signals(7540.0, 0.0, 0.0), DT);
        }
        assert!(ch.limiter() > 0.95, "no flicker at the threshold");
        for _ in 0..90 {
            ch.update(&signals(3000.0, 0.0, 0.0), DT);
        }
        assert!(ch.limiter() < 0.01 && ch.droplet_count() == 0);
        for _ in 0..600 {
            ch.update(&signals(3000.0, 0.0, 0.0), DT);
        }
        assert!(!ch.animating(), "the column comes to rest");
    }

    #[test]
    fn channel_droplets_leave_only_from_the_limiter_end() {
        let t = tokens();
        let mut ch = LiquidChannel::new(&t);
        let geom = ChannelGeometry::fluid_family();
        let end = ch.angle_at(1.0);
        // A standing start straight onto the limiter: the column has to climb first.
        let sig = signals(8000.0, 0.0, 0.0);
        ch.update(&sig, DT);
        assert!(ch.limiter() > 0.0, "the hot glow starts at once");
        assert_eq!(ch.droplet_count(), 0, "no droplet from a column still climbing");
        for _ in 0..60 {
            ch.update(&sig, DT);
            for (c, _) in ch.droplets(&geom) {
                // Launched at the end of the tube, they fall inward from there (within 40°).
                let a = (c - geom.center).angle();
                let off = (wrap_from(a, end - PI) - end).abs();
                assert!(off < deg(40.0), "droplet {:.1}° off the limiter end", off.to_degrees());
            }
        }
        assert!(ch.droplet_count() > 0);
    }

    #[test]
    fn channel_meniscus_tilts_with_g_and_surges_with_the_rate() {
        let t = tokens();
        let mut ch = LiquidChannel::new(&t);
        for _ in 0..90 {
            ch.update(&signals(4000.0, 0.8, 0.0), DT);
        }
        assert!(ch.tilt() > 0.0);
        assert!(ch.tilt() <= deg(MENISCUS_MAX_TILT_DEG) * 1.5);
        // A shift drops the rpm fast: the meniscus flattens, then wobbles back.
        let mut flattest = f32::MAX;
        for _ in 0..20 {
            ch.update(&signals(2600.0, 0.8, 0.0), DT);
            flattest = flattest.min(ch.surge.value());
        }
        assert!(flattest < -0.1, "the cap flattens as the column falls: {flattest}");
        let geom = ChannelGeometry::fluid_family();
        let col = ch.column(&geom);
        assert!(col.bulge >= BULGE_RANGE.0 && col.bulge <= BULGE_RANGE.1);
        assert!(col.s_back >= 1.0, "the body hands over inside the column");
    }

    #[test]
    fn column_outline_is_closed_and_on_the_tube() {
        let t = tokens();
        let mut ch = LiquidChannel::new(&t);
        for _ in 0..30 {
            ch.update(&signals(6000.0, -0.5, 0.2), DT);
        }
        let geom = ChannelGeometry::fluid_family();
        let col = ch.column(&geom);
        let r_in = geom.radius() - geom.column - 1e-2;
        let r_out = geom.radius() + geom.column + 1e-2;
        for pt in col.outline().iter().chain(col.meniscus().iter()) {
            let d = (*pt - geom.center).length();
            assert!(d >= r_in && d <= r_out, "point off the column: {d}");
        }
        // The cap's centre sample is the reading.
        let mid = col.meniscus()[MENISCUS_SAMPLES / 2 + 1];
        let reading = Vec2::polar(geom.center, geom.radius(), ch.angle_at(ch.fraction()));
        assert!((mid - reading).length() < 0.5, "front centre at the reading");
    }

    #[test]
    fn droplet_pool_is_fixed_and_reuses_the_oldest() {
        let mut d = Droplets::new();
        for i in 0..(MAX_DROPLETS + 3) {
            d.spawn(vec2(i as f32 * 0.01, 0.0), Vec2::ZERO, 0.03, 10.0);
            d.step(0.01, Vec2::ZERO);
        }
        assert_eq!(d.len(), MAX_DROPLETS);
        d.step(0.1, vec2(0.0, 1.0));
        assert!(d.iter().all(|(p, _)| p.y > 0.0), "gravity pulls them down");
        d.merge_where(|p, _, _| p.x < 0.05);
        assert!(d.len() < MAX_DROPLETS);
        d.clear();
        assert!(d.is_empty());
        // Droplets shrink away at the end of their life (steps are capped at 0.25 s).
        d.spawn(Vec2::ZERO, Vec2::ZERO, 0.05, 1.0);
        for _ in 0..9 {
            d.step(0.1, Vec2::ZERO);
        }
        let (_, r) = d.iter().next().unwrap_or((Vec2::ZERO, 1.0));
        assert!(r < 0.05 * 0.5, "shrinking: {r}");
    }

    #[test]
    fn emission_is_from_dt_and_starts_at_once() {
        let mut d = Droplets::new();
        assert_eq!(d.due(DT, 0.16, true), 1, "first droplet leaves at limiter onset");
        let mut total = 1;
        for _ in 0..59 {
            total += d.due(DT, 0.16, true);
        }
        assert!((6..=8).contains(&total), "about 1 s / 0.16 s: {total}");
        assert_eq!(d.due(DT, 0.16, false), 0);
    }

    #[test]
    fn underlay_cells_walk_the_atlas() {
        let cell = |t: f32| atlas_cell(8, 8, 64, FLIPBOOK_FPS, t);
        let first = cell(0.0);
        assert!(first.min.x < 0.01 && first.max.x < 0.126 && first.min.y < 0.01);
        let ninth = cell(8.5 / FLIPBOOK_FPS);
        assert!(ninth.min.y > 0.12 && ninth.min.x < 0.01, "frame 8 starts row 2");
        let wrapped = cell(64.5 / FLIPBOOK_FPS);
        assert_eq!(wrapped, first, "loops after 64 frames");
        assert_eq!(
            atlas_cell(1, 1, 1, FLIPBOOK_FPS, 123.0),
            Rect::from_xywh(0.0, 0.0, 1.0, 1.0).inset(0.005)
        );
        assert_eq!(
            atlas_cell(0, 0, 0, f32::NAN, f32::INFINITY),
            atlas_cell(1, 1, 1, 0.0, 0.0),
            "degenerate input"
        );
        // Fewer frames than cells loop early.
        assert_eq!(atlas_cell(8, 8, 10, 1.0, 10.5), atlas_cell(8, 8, 10, 1.0, 0.0));
    }

    #[test]
    fn rim_light_lives_on_the_side_away_from_the_light() {
        let to_light = Vec2::from_angle(deg(-135.0));
        assert!((rim_light(-to_light, to_light) - 1.0).abs() < 1e-6, "facing away: full");
        assert!(rim_light(to_light, to_light) < 1e-6, "facing the light: none");
        let side = rim_light(Vec2::from_angle(deg(-45.0)), to_light);
        assert!((side - 0.25).abs() < 1e-5, "side-on: a quarter ({side})");
    }

    #[test]
    fn hash_is_uniformish() {
        let mean = (0..1000).map(hash01).sum::<f32>() / 1000.0;
        assert!((0.45..0.55).contains(&mean), "mean {mean}");
        assert!((0..1000).map(hash01).all(|v| (0.0..1.0).contains(&v)));
    }
}
