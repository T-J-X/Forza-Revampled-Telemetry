//! The zone table of `AmgMbuxSuperscreen` as data (local 4K px round the cockpit anchor, y down).
//!
//! The dash is one physical screen, the W214 driver display (12.3 inches, 2.67:1), so its zones
//! sit inside the screen's black glass and the screen sits inside the cockpit envelope
//! (−720, −320, 1440 × 560). First build, made before its Penpot board; the rebuild follows the board.
use fh6_kit::{deg, vec2, Rect, Vec2};

/// Crescent outline samples per edge (the outer arc and the tapered inner edge).
pub(super) const CRESCENT_SAMPLES: usize = 25;
/// Shift segments on each side of the centre along the glass's top edge (they fill outside-in).
pub(super) const SHIFT_PER_SIDE: usize = 7;
/// Receding rings of the tunnel (drawn by the Blender plate, or by the fallback).
pub(super) const RINGS: usize = 6;
/// Ticks on a side scale (major every `SCALE_MAJOR_EVERY`).
pub(super) const SCALE_TICKS: usize = 21;
/// See [`SCALE_TICKS`].
pub(super) const SCALE_MAJOR_EVERY: usize = 5;

/// Geometry of the cockpit view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Geom {
    /// The display unit with its bezel (the housing plate covers exactly this rect).
    pub housing: Rect,
    /// Corner radius of the housing.
    pub housing_r: f32,
    /// The black glass of the screen, inside the bezel.
    pub screen: Rect,
    /// Corner radius of the screen glass.
    pub screen_r: f32,
    /// Centre of the tunnel rev counter (the outer crescents' circle).
    pub tunnel: Vec2,
    /// Outer radius of the two red crescents.
    pub crescent_r: f32,
    /// Crescent thickness at its middle (9 and 3 o'clock); it tapers to 0 at the tips.
    pub crescent_w: f32,
    /// Half the crescent's span round 9 (and 3) o'clock (radians).
    pub crescent_half: f32,
    /// The vanishing disc that holds the gear: centre and radius.
    pub disc: Vec2,
    /// See [`Geom::disc`].
    pub disc_r: f32,
    /// Gear cap height.
    pub gear_cap: f32,
    /// Speed (cap middle) and its cap height.
    pub speed: Vec2,
    /// See [`Geom::speed`].
    pub speed_cap: f32,
    /// Unit under the speed (cap middle).
    pub speed_unit: Vec2,
    /// Shift segments: inner and outer x (|x|), centre y, height, gap.
    pub shift_inner: f32,
    /// See [`Geom::shift_inner`].
    pub shift_outer: f32,
    /// See [`Geom::shift_inner`].
    pub shift_y: f32,
    /// See [`Geom::shift_inner`].
    pub shift_h: f32,
    /// See [`Geom::shift_inner`].
    pub shift_gap: f32,
    /// The tunnel floor: a trapezoid with these y values and half widths.
    pub floor_top: f32,
    /// See [`Geom::floor_top`].
    pub floor_bottom: f32,
    /// See [`Geom::floor_top`].
    pub floor_top_half: f32,
    /// See [`Geom::floor_top`].
    pub floor_bottom_half: f32,
    /// The rpm readout in the floor (cap middle) and its cap height; its unit below.
    pub rpm: Vec2,
    /// See [`Geom::rpm`].
    pub rpm_cap: f32,
    /// See [`Geom::rpm`].
    pub rpm_unit: Vec2,
    /// The side scales (torque left, power right) round the tunnel centre: radius, the left
    /// scale's start (its zero, low) and end angles; the right scale mirrors them.
    pub scale_r: f32,
    /// See [`Geom::scale_r`].
    pub scale_from: f32,
    /// See [`Geom::scale_r`].
    pub scale_to: f32,
    /// Radius of the scale numerals' centres (inside the ticks, between them and the crescents,
    /// as on the real display).
    pub scale_numeral_r: f32,
    /// Live torque readout (right-aligned value end) and power readout (left-aligned start).
    pub torque_value: Vec2,
    /// See [`Geom::torque_value`].
    pub power_value: Vec2,
    /// G-force widget (left) and engine-data widget (right): centre and radius.
    pub g_widget: Vec2,
    /// See [`Geom::g_widget`].
    pub boost_widget: Vec2,
    /// See [`Geom::g_widget`].
    pub widget_r: f32,
    /// The logo zone (bottom left) and the transmission box (bottom right).
    pub logo: Rect,
    /// See [`Geom::logo`].
    pub transmission: Rect,
    /// Smallest cap height of any label.
    pub label_cap: f32,
}

impl Geom {
    /// The cockpit table. The dash draws only in the cockpit view (catalog row 4).
    pub(super) const COCKPIT: Geom = Geom {
        housing: Rect {
            min: Vec2 { x: -720.0, y: -314.0 },
            max: Vec2 { x: 720.0, y: 236.0 },
        },
        housing_r: 30.0,
        screen: Rect {
            min: Vec2 { x: -706.0, y: -300.0 },
            max: Vec2 { x: 706.0, y: 222.0 },
        },
        screen_r: 18.0,
        tunnel: Vec2 { x: 0.0, y: -36.0 },
        crescent_r: 278.0,
        crescent_w: 40.0,
        crescent_half: 0.907_571_2, // 52°
        disc: Vec2 { x: 0.0, y: -62.0 },
        disc_r: 92.0,
        gear_cap: 120.0,
        speed: Vec2 { x: 0.0, y: -230.0 },
        speed_cap: 60.0,
        speed_unit: Vec2 { x: 0.0, y: -180.0 },
        shift_inner: 96.0,
        shift_outer: 680.0,
        shift_y: -286.0,
        shift_h: 10.0,
        shift_gap: 8.0,
        floor_top: 92.0,
        floor_bottom: 196.0,
        floor_top_half: 128.0,
        floor_bottom_half: 214.0,
        rpm: Vec2 { x: 0.0, y: 132.0 },
        rpm_cap: 44.0,
        rpm_unit: Vec2 { x: 0.0, y: 176.0 },
        scale_r: 360.0,
        scale_from: 2.408_554_4, // 138°
        scale_to: 3.804_818,     // 218°
        scale_numeral_r: 314.0,
        torque_value: Vec2 { x: -430.0, y: 150.0 },
        power_value: Vec2 { x: 430.0, y: 150.0 },
        g_widget: Vec2 { x: -566.0, y: -40.0 },
        boost_widget: Vec2 { x: 566.0, y: -40.0 },
        widget_r: 84.0,
        logo: Rect {
            min: Vec2 { x: -686.0, y: 156.0 },
            max: Vec2 { x: -486.0, y: 204.0 },
        },
        transmission: Rect {
            min: Vec2 { x: 590.0, y: 156.0 },
            max: Vec2 { x: 686.0, y: 204.0 },
        },
        label_cap: 22.0,
    };

    /// Angle (screen convention) of the left crescent at fill `t` (0 = bottom tip, 1 = top tip).
    pub(super) fn crescent_angle(&self, t: f32) -> f32 {
        deg(180.0) - self.crescent_half + 2.0 * self.crescent_half * t
    }

    /// Thickness of the crescent at fill `t`: a sine taper, 0 at the tips.
    pub(super) fn crescent_width(&self, t: f32) -> f32 {
        let s = (core::f32::consts::PI * t.clamp(0.0, 1.0)).sin().max(0.0);
        self.crescent_w * s.powf(0.7)
    }

    /// The point on the left crescent's outer edge (`inset` 0) or inner edge (`inset` 1) at
    /// fill `t`; `side` −1 is the left crescent and +1 its mirror on the right.
    pub(super) fn crescent_point(&self, t: f32, inset: f32, side: f32) -> Vec2 {
        let a = self.crescent_angle(t);
        let r = self.crescent_r - self.crescent_width(t) * inset;
        let p = vec2(a.cos(), a.sin()) * r;
        vec2(if side < 0.0 { p.x } else { -p.x }, p.y) + self.tunnel
    }

    /// A point on a side scale at `t` (0 = its zero, low; 1 = its top), radius `r`.
    pub(super) fn scale_point(&self, t: f32, r: f32, side: f32) -> Vec2 {
        let a = self.scale_from + (self.scale_to - self.scale_from) * t.clamp(0.0, 1.0);
        let p = vec2(a.cos(), a.sin()) * r;
        vec2(if side < 0.0 { p.x } else { -p.x }, p.y) + self.tunnel
    }

    /// The shift segment `i` (0 = outermost) on `side` (−1 left, +1 right).
    pub(super) fn shift_segment(&self, i: usize, side: f32) -> Rect {
        let n = SHIFT_PER_SIDE as f32;
        let span = self.shift_outer - self.shift_inner;
        let w = (span - self.shift_gap * (n - 1.0)) / n;
        let outer = self.shift_outer - i as f32 * (w + self.shift_gap);
        let (x0, x1) = if side < 0.0 {
            (-outer, -outer + w)
        } else {
            (outer - w, outer)
        };
        Rect::from_points(
            vec2(x0, self.shift_y - self.shift_h * 0.5),
            vec2(x1, self.shift_y + self.shift_h * 0.5),
        )
    }

    /// Radius and centre of receding ring `k` (0 = the outermost, under the crescents).
    pub(super) fn ring(&self, k: usize) -> (Vec2, f32) {
        let t = (k as f32 + 1.0) / (RINGS as f32 + 1.0);
        let r = self.crescent_r * 0.88 + (self.disc_r * 1.05 - self.crescent_r * 0.88) * t.powf(0.85);
        let c = self.tunnel + (self.disc - self.tunnel) * t;
        (c, r)
    }

    /// The four corners of the tunnel floor (clockwise from top left).
    pub(super) fn floor(&self) -> [Vec2; 4] {
        [
            vec2(-self.floor_top_half, self.floor_top),
            vec2(self.floor_top_half, self.floor_top),
            vec2(self.floor_bottom_half, self.floor_bottom),
            vec2(-self.floor_bottom_half, self.floor_bottom),
        ]
    }

    /// The tunnel plate's rect (the Blender render covers it 1:1).
    pub(super) fn tunnel_rect(&self) -> Rect {
        let r = self.crescent_r + 6.0;
        Rect::from_points(
            vec2(self.tunnel.x - r, self.tunnel.y - r).max(self.screen.min),
            vec2(self.tunnel.x + r, self.tunnel.y + r).min(self.screen.max),
        )
    }
}
