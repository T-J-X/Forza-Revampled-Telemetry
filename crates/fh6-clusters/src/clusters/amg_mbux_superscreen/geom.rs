//! The zone table of `AmgMbuxSuperscreen` as data (local 4K px round the cockpit anchor, y down).
//!
//! It follows Penpot board v4 (page "07 · AmgMbuxSuperscreen", boards "v4 · <state>"): the W214
//! Sporty display's crescents, side scales, widgets and selector stack round TJS's sky tunnel (a
//! dusk sky inside the tunnel, and twelve rings that light from the far end towards the driver as
//! the rev counter). There is no housing: every part floats on OLED black inside the cockpit
//! envelope (−720, −320, 1440 × 560) over a soft charcoal stage with no edge. Keep the plate geometry in step with
//! `tools/asset-forge/src/recipes/amg_mbux_superscreen.rs`.
use fh6_kit::{deg, vec2, Rect, Vec2};

/// Crescent outline samples per edge (the outer arc and the tapered inner edge).
pub(super) const CRESCENT_SAMPLES: usize = 33;
/// The crescents' taper: the width follows sin(πt) to this power (flat in the middle, pointed
/// tips, as on the real display).
pub(super) const CRESCENT_TAPER: f32 = 0.38;
/// The tunnel's rings (the rev counter): the ribs on the Blender plate, lit by the cluster.
pub(super) const RINGS: usize = 12;
/// The nearest rings, which light red (the shift zone) and strobe on the limiter.
pub(super) const RED_RINGS: usize = 3;
/// The rings' perspective ease: the near ones spread further apart.
pub(super) const RING_EASE: f32 = 1.7;
/// Points of the sky window's outline (the tunnel's circle above the horizon).
pub(super) const SKY_SAMPLES: usize = 49;
/// Ticks on a side scale (major every `SCALE_MAJOR_EVERY`).
pub(super) const SCALE_TICKS: usize = 21;
/// See [`SCALE_TICKS`].
pub(super) const SCALE_MAJOR_EVERY: usize = 5;

/// Geometry of the cockpit view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Geom {
    /// The soft charcoal stage behind everything: centre and radii (an ellipse with no edge).
    pub stage: Vec2,
    /// See [`Geom::stage`].
    pub stage_r: Vec2,
    /// Centre of the tunnel rev counter (the crescents' circle).
    pub tunnel: Vec2,
    /// Outer radius of the two red crescents.
    pub crescent_r: f32,
    /// Crescent thickness at its middle (9 and 3 o'clock); it tapers to 0 at the tips.
    pub crescent_w: f32,
    /// Half the crescent's span round 9 (and 3) o'clock (radians).
    pub crescent_half: f32,
    /// The far ring (by the horizon): centre and radius.
    pub ring_far: Vec2,
    /// See [`Geom::ring_far`].
    pub ring_far_r: f32,
    /// The gap the near ring leaves inside the crescents.
    pub ring_inset: f32,
    /// The horizon (y): the sky ends and the road starts here.
    pub horizon: f32,
    /// The rings fade over this height above the horizon, to this share of their light at the
    /// top (the sky shows through them).
    pub ceiling_fade: f32,
    /// See [`Geom::ceiling_fade`].
    pub ceiling_floor: f32,
    /// The rect one turn of the sky plate covers (it wraps left to right; its bottom edge is the
    /// horizon).
    pub sky: Rect,
    /// The road: a trapezoid with these y values and half widths.
    pub floor_top_half: f32,
    /// See [`Geom::floor_top_half`].
    pub floor_bottom: f32,
    /// See [`Geom::floor_top_half`].
    pub floor_bottom_half: f32,
    /// Speed (cap middle) and its cap height.
    pub speed: Vec2,
    /// See [`Geom::speed`].
    pub speed_cap: f32,
    /// The speed's unit, left-aligned to the right of three digits (cap middle of its start).
    pub speed_unit: Vec2,
    /// The thin speed line under the speed: its y and half length.
    pub line_y: f32,
    /// See [`Geom::line_y`].
    pub line_half: f32,
    /// The rpm readout on the road (cap middle) and its cap height; its unit below.
    pub rpm: Vec2,
    /// See [`Geom::rpm`].
    pub rpm_cap: f32,
    /// See [`Geom::rpm`].
    pub rpm_unit: Vec2,
    /// The side scales (torque left, power right) round the tunnel centre: the band's inner and
    /// outer radius, and the left band's lower and upper angle; the right band mirrors them.
    pub band_in: f32,
    /// See [`Geom::band_in`].
    pub band_out: f32,
    /// See [`Geom::band_in`].
    pub scale_from: f32,
    /// See [`Geom::band_in`].
    pub scale_to: f32,
    /// The share of the band the scale uses (its zero and its top), inside the band's ends.
    pub scale_span: Vec2,
    /// Radius of the scale numerals' centres (on the band, inside the ticks), and their cap.
    pub scale_numeral_r: f32,
    /// See [`Geom::scale_numeral_r`].
    pub scale_cap: f32,
    /// Live torque (left) and power (right) values (cap middle), their cap, and their units.
    pub torque_value: Vec2,
    /// See [`Geom::torque_value`].
    pub power_value: Vec2,
    /// See [`Geom::torque_value`].
    pub value_cap: f32,
    /// See [`Geom::torque_value`].
    pub value_unit_dy: f32,
    /// G-force widget (left) and engine-data widget (right): centre and radius.
    pub g_widget: Vec2,
    /// See [`Geom::g_widget`].
    pub boost_widget: Vec2,
    /// See [`Geom::g_widget`].
    pub widget_r: f32,
    /// How far a widget's title sits above its ring (to the cap middle).
    pub title_dy: f32,
    /// The engaged gear (cap middle) and its cap; the selector stack's column (x) of the
    /// neighbours and the `M`, its cap and its row pitch.
    pub gear: Vec2,
    /// See [`Geom::gear`].
    pub gear_cap: f32,
    /// See [`Geom::gear`]: the centre of the stack's middle row (the `M`).
    pub stack: Vec2,
    /// See [`Geom::gear`].
    pub stack_cap: f32,
    /// See [`Geom::gear`].
    pub stack_pitch: f32,
    /// The logo zone (bottom left).
    pub logo: Rect,
    /// Cap height of titles, and of units and small labels.
    pub label_cap: f32,
    /// See [`Geom::label_cap`].
    pub unit_cap: f32,
    /// The plates' rects (the Blender renders cover them 1:1): the assembly (tunnel, road and
    /// side scales) and the crescents.
    pub assembly_rect: Rect,
    /// See [`Geom::assembly_rect`].
    pub crescent_rect: Rect,
    /// The margin the lens and the gauge plates keep round their widget.
    pub lens_margin: f32,
    /// See [`Geom::lens_margin`].
    pub gauge_margin: f32,
}

impl Geom {
    /// The cockpit table. The dash draws only in the cockpit view (catalog row 4).
    pub(super) const COCKPIT: Geom = Geom {
        stage: Vec2 { x: 0.0, y: -36.0 },
        stage_r: Vec2 { x: 760.0, y: 300.0 },
        tunnel: Vec2 { x: 0.0, y: -36.0 },
        crescent_r: 300.0,
        crescent_w: 84.0,
        crescent_half: 0.837_758, // 48°
        ring_far: Vec2 { x: 0.0, y: -60.0 },
        ring_far_r: 64.0,
        ring_inset: 14.0,
        horizon: -58.0,
        ceiling_fade: 90.0,
        ceiling_floor: 0.08,
        sky: Rect {
            min: Vec2 { x: -512.0, y: -282.0 },
            max: Vec2 { x: 512.0, y: -58.0 },
        },
        floor_top_half: 44.0,
        floor_bottom: 200.0,
        floor_bottom_half: 214.0,
        speed: Vec2 { x: 0.0, y: -190.0 },
        speed_cap: 58.0,
        speed_unit: Vec2 { x: 90.0, y: -172.0 },
        line_y: -153.0,
        line_half: 150.0,
        rpm: Vec2 { x: 0.0, y: 56.0 },
        rpm_cap: 40.0,
        rpm_unit: Vec2 { x: 0.0, y: 92.0 },
        band_in: 312.0,
        band_out: 392.0,
        scale_from: 2.617_994, // 150°
        scale_to: 3.769_911,   // 216°
        scale_span: Vec2 { x: 0.08, y: 0.92 },
        scale_numeral_r: 344.0,
        scale_cap: 25.0,
        torque_value: Vec2 { x: -400.0, y: 161.0 },
        power_value: Vec2 { x: 400.0, y: 161.0 },
        value_cap: 30.0,
        value_unit_dy: 28.0,
        g_widget: Vec2 { x: -500.0, y: -52.0 },
        boost_widget: Vec2 { x: 500.0, y: -52.0 },
        widget_r: 76.0,
        title_dy: 32.0,
        gear: Vec2 { x: 660.0, y: 170.0 },
        gear_cap: 84.0,
        stack: Vec2 { x: 584.0, y: 170.0 },
        stack_cap: 18.0,
        stack_pitch: 28.0,
        logo: Rect {
            min: Vec2 { x: -686.0, y: 164.0 },
            max: Vec2 { x: -506.0, y: 208.0 },
        },
        label_cap: 20.0,
        unit_cap: 18.0,
        assembly_rect: Rect {
            min: Vec2 { x: -404.0, y: -280.0 },
            max: Vec2 { x: 404.0, y: 208.0 },
        },
        crescent_rect: Rect {
            min: Vec2 { x: -304.0, y: -262.0 },
            max: Vec2 { x: 304.0, y: 190.0 },
        },
        lens_margin: 12.0,
        gauge_margin: 16.0,
    };

    /// Angle (screen convention) of the left crescent at fill `t` (0 = bottom tip, 1 = top tip).
    pub(super) fn crescent_angle(&self, t: f32) -> f32 {
        deg(180.0) - self.crescent_half + 2.0 * self.crescent_half * t
    }

    /// Thickness of the crescent at fill `t`: 0 at the tips, flat in the middle.
    pub(super) fn crescent_width(&self, t: f32) -> f32 {
        let s = (core::f32::consts::PI * t.clamp(0.0, 1.0)).sin().max(0.0);
        self.crescent_w * s.powf(CRESCENT_TAPER)
    }

    /// The point on the left crescent's outer edge (`inset` 0) or inner edge (`inset` 1) at
    /// fill `t`; `side` −1 is the left crescent and +1 its mirror on the right.
    pub(super) fn crescent_point(&self, t: f32, inset: f32, side: f32) -> Vec2 {
        let a = self.crescent_angle(t);
        let r = self.crescent_r - self.crescent_width(t) * inset;
        let p = vec2(a.cos(), a.sin()) * r;
        vec2(if side < 0.0 { p.x } else { -p.x }, p.y) + self.tunnel
    }

    /// The angle of a side scale at `t` (0 = the band's lower end, 1 = its upper end) on `side`.
    pub(super) fn band_angle(&self, t: f32, side: f32) -> f32 {
        let a = self.scale_from + (self.scale_to - self.scale_from) * t.clamp(0.0, 1.0);
        if side < 0.0 {
            a
        } else {
            deg(180.0) - a
        }
    }

    /// The band position of a scale value `v` (0 = zero, 1 = full).
    pub(super) fn scale_t(&self, v: f32) -> f32 {
        self.scale_span.x + (self.scale_span.y - self.scale_span.x) * v.clamp(0.0, 1.0)
    }

    /// A point on a side scale at value `v` (0 = zero, 1 = full), radius `r`.
    pub(super) fn scale_point(&self, v: f32, r: f32, side: f32) -> Vec2 {
        let a = self.band_angle(self.scale_t(v), side);
        self.tunnel + vec2(a.cos(), a.sin()) * r
    }

    /// Centre and radius of ring `k` (0 = the far one, by the horizon; [`RINGS`] − 1 the near
    /// one, just inside the crescents): the centre slides from the far ring's to the tunnel
    /// centre and the radius grows with the perspective ease, as in the recipe's `ring_table`.
    pub(super) fn ring(&self, k: usize) -> (Vec2, f32) {
        let e = ((k as f32 + 1.0) / RINGS as f32).powf(RING_EASE);
        let r1 = self.crescent_r - self.crescent_w - self.ring_inset;
        (
            self.ring_far + (self.tunnel - self.ring_far) * e,
            self.ring_far_r + (r1 - self.ring_far_r) * e,
        )
    }

    /// The point the sky and the horizon lean about with lateral G.
    pub(super) fn horizon_pivot(&self) -> Vec2 {
        vec2(self.tunnel.x, self.horizon)
    }

    /// The sky window: the near ring's circle above the horizon, in the frame leaned by `tilt`
    /// about [`Geom::horizon_pivot`] (the circle stays put on screen; the horizon leans), into
    /// `out`. Returns the number of points (0 when the horizon misses the circle).
    pub(super) fn sky_window(&self, tilt: f32, out: &mut [Vec2; SKY_SAMPLES]) -> usize {
        let (c, r) = self.ring(RINGS - 1);
        let pivot = self.horizon_pivot();
        let c = pivot + (c - pivot).rotate(-tilt);
        let dy = self.horizon - c.y;
        if dy.abs() >= r {
            return 0;
        }
        let hx = (r * r - dy * dy).sqrt();
        let a0 = dy.atan2(-hx);
        let mut a1 = dy.atan2(hx);
        // Run over the top (screen up is −90°).
        if a1 < a0 {
            a1 += core::f32::consts::TAU;
        }
        let n = SKY_SAMPLES;
        for (i, p) in out.iter_mut().enumerate() {
            let a = a0 + (a1 - a0) * i as f32 / (n - 1) as f32;
            *p = c + vec2(a.cos(), a.sin()) * r;
        }
        n
    }

    /// The four corners of the road (clockwise from top left).
    pub(super) fn floor(&self) -> [Vec2; 4] {
        [
            vec2(-self.floor_top_half, self.horizon),
            vec2(self.floor_top_half, self.horizon),
            vec2(self.floor_bottom_half, self.floor_bottom),
            vec2(-self.floor_bottom_half, self.floor_bottom),
        ]
    }

    /// The rect a widget's plate covers.
    pub(super) fn widget_rect(&self, center: Vec2, margin: f32) -> Rect {
        Rect::from_center_size(center, Vec2::splat((self.widget_r + margin) * 2.0))
    }

    /// The centre of row `row` of the selector stack (−1 the gear above, 0 the `M`, +1 the gear
    /// below).
    pub(super) fn stack_row(&self, row: f32) -> Vec2 {
        self.stack + vec2(0.0, row * self.stack_pitch)
    }
}
