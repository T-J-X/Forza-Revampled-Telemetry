//! The zone table: every part of GT7's Bonnet View band as data, in local 4K px round the hood
//! anchor (1920, 1760), y down, angles clockwise from 3 o'clock.
//!
//! The recipe (`tools/asset-forge/src/recipes/gt7_hood_pro.rs`) renders the static parts from
//! the same numbers into one plate registered to [`Geom::plate`], so a change here is a change
//! there too (and a new asset build). The cockpit view draws the same band scaled to fit the
//! cockpit envelope ([`fit`]).
use fh6_kit::{vec2, Rect, Vec2};

/// The band, as data. Points are centres unless a name says otherwise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Geom {
    /// The hardware plate's frame (the Blender sprite at 1 px per local px): x, y, w, h.
    pub plate: [f32; 4],
    /// The LED field of the rev bar (x from, x to) and its LEDs (top, bottom).
    pub rev_x: [f32; 2],
    pub tick_y: [f32; 2],
    /// LED count, LED width and corner radius.
    pub ticks: usize,
    pub tick_w: f32,
    pub tick_r: f32,
    /// The rev bar's black-glass light bar: centre, size, corner radius.
    pub bar_c: Vec2,
    pub bar_size: Vec2,
    pub bar_r: f32,
    /// The polished plinth shows this far round the light bar.
    pub plinth: f32,
    /// The steering dot: its height, radius and travel either side of the centre line.
    pub dot_y: f32,
    pub dot_r: f32,
    pub dot_travel: f32,
    /// The speed: right edge, cap top, cap height, digit cells.
    pub speed_right: f32,
    pub speed_top: f32,
    pub speed_cap: f32,
    pub speed_cells: u8,
    /// The speed unit under the digits: cap top and cap height.
    pub unit_top: f32,
    pub unit_cap: f32,
    /// The divider between speed and gear: x, y, w, h.
    pub divider: [f32; 4],
    /// The gear: left edge, cap top, cap height, and the widest it may set (a two-digit gear
    /// is set smaller to fit).
    pub gear_left: f32,
    pub gear_top: f32,
    pub gear_cap: f32,
    pub gear_max_w: f32,
    /// The suggested-gear tile: centre, base size, face size, face radius, digit cap, and its
    /// sprite's frame (x, y, w, h).
    pub sug_c: Vec2,
    pub sug_outer: Vec2,
    pub sug_face: Vec2,
    pub sug_r: f32,
    pub sug_cap: f32,
    pub sug_frame: [f32; 4],
    /// The pedal slots (brake left, throttle right): centre x of each, centre y, base size,
    /// slot size, and the fill's inset inside the slot.
    pub slots_x: [f32; 2],
    pub slot_cy: f32,
    pub slot_outer: Vec2,
    pub slot_inner: Vec2,
    pub fill_inset: f32,
    /// The tyre diagram: the car chip and the four tyre lenses (front left, front right, rear
    /// left, rear right).
    pub car_c: Vec2,
    pub car_size: Vec2,
    pub pills: [Vec2; 4],
    pub pill_size: Vec2,
    /// The boost gauge: centre, scale radius, scale band width, sweep (degrees) and the bar
    /// values at its two ends (x100 kPa, as GT7's boost gauge).
    pub boost_c: Vec2,
    pub boost_r: f32,
    pub boost_band: f32,
    pub boost_sweep: [f32; 2],
    pub boost_bar: [f32; 2],
    /// The boost fill's track inside the scale: radius and width, and the width of the plate's
    /// black-glass groove it runs in.
    pub boost_track: [f32; 2],
    pub boost_groove: f32,
    /// The boost label: cap top and cap height.
    pub boost_label_top: f32,
    pub boost_label_cap: f32,
    /// The trip tile: centre, base size, face size and digit cap.
    pub trip_c: Vec2,
    pub trip_outer: Vec2,
    pub trip_face: Vec2,
    pub trip_cap: f32,
    /// The telltales (slide left, limiter right): centres, lamp face radius and ring radius.
    pub lamps: [Vec2; 2],
    pub lamp_face: f32,
    pub lamp_ring: f32,
    /// The logo zone: x, y, w, h.
    pub logo: [f32; 4],
}

/// The hood band.
pub(super) const HOOD: Geom = Geom {
    plate: [-880.0, -100.0, 1760.0, 240.0],
    rev_x: [-380.0, 380.0],
    tick_y: [-60.0, -34.0],
    ticks: 95,
    tick_w: 5.0,
    tick_r: 1.5,
    bar_c: vec2(0.0, -47.0),
    bar_size: vec2(788.0, 46.0),
    bar_r: 13.0,
    plinth: 8.0,
    dot_y: -94.0,
    dot_r: 8.0,
    dot_travel: 340.0,
    speed_right: -26.0,
    speed_top: 8.0,
    speed_cap: 58.0,
    speed_cells: 3,
    unit_top: 82.0,
    unit_cap: 22.0,
    divider: [-1.0, 8.0, 2.0, 90.0],
    gear_left: 30.0,
    gear_top: 8.0,
    gear_cap: 88.0,
    gear_max_w: 120.0,
    sug_c: vec2(211.0, 49.0),
    sug_outer: vec2(94.0, 86.0),
    sug_face: vec2(88.0, 80.0),
    sug_r: 11.0,
    sug_cap: 52.0,
    sug_frame: [156.0, -2.0, 110.0, 102.0],
    slots_x: [-442.0, 442.0],
    slot_cy: 13.0,
    slot_outer: vec2(24.0, 158.0),
    slot_inner: vec2(16.0, 150.0),
    fill_inset: 3.0,
    car_c: vec2(-795.0, 18.0),
    car_size: vec2(70.0, 160.0),
    pills: [
        vec2(-848.0, -38.0),
        vec2(-742.0, -38.0),
        vec2(-848.0, 74.0),
        vec2(-742.0, 74.0),
    ],
    pill_size: vec2(22.0, 58.0),
    boost_c: vec2(-600.0, 6.0),
    boost_r: 60.0,
    boost_band: 5.0,
    boost_sweep: [160.0, 380.0],
    boost_bar: [-1.0, 2.0],
    boost_track: [51.0, 6.0],
    boost_groove: 9.0,
    boost_label_top: 34.0,
    boost_label_cap: 22.0,
    trip_c: vec2(-600.0, 108.0),
    trip_outer: vec2(208.0, 44.0),
    trip_face: vec2(200.0, 36.0),
    trip_cap: 22.0,
    lamps: [vec2(-497.0, -36.0), vec2(497.0, -36.0)],
    lamp_face: 24.0,
    lamp_ring: 29.0,
    logo: [540.0, -40.0, 320.0, 100.0],
};

/// A rect from `[x, y, w, h]`.
pub(super) fn rect(r: [f32; 4]) -> Rect {
    Rect::from_xywh(r[0], r[1], r[2], r[3])
}

impl Geom {
    /// The plate's frame.
    pub fn plate_rect(&self) -> Rect {
        rect(self.plate)
    }

    /// Distance from one LED's centre to the next.
    pub fn tick_pitch(&self) -> f32 {
        (self.rev_x[1] - self.rev_x[0]) / self.ticks.max(1) as f32
    }

    /// LED `i` (0 at the left).
    pub fn tick_rect(&self, i: usize) -> Rect {
        let pitch = self.tick_pitch();
        let cx = self.rev_x[0] + pitch * (i as f32 + 0.5);
        Rect::from_xywh(
            cx - self.tick_w * 0.5,
            self.tick_y[0],
            self.tick_w,
            self.tick_y[1] - self.tick_y[0],
        )
    }

    /// The light bar.
    pub fn bar_rect(&self) -> Rect {
        Rect::from_center_size(self.bar_c, self.bar_size)
    }

    /// The light bar with its plinth.
    pub fn plinth_rect(&self) -> Rect {
        self.bar_rect().expand(self.plinth)
    }

    /// The speed field: `cells` digit cells `cell` wide, right-aligned on the divider side.
    pub fn speed_field(&self, cell: f32) -> Rect {
        let w = cell * f32::from(self.speed_cells);
        Rect::from_xywh(self.speed_right - w, self.speed_top, w, self.speed_cap)
    }

    /// The suggested-gear tile's face.
    pub fn sug_face_rect(&self) -> Rect {
        Rect::from_center_size(self.sug_c, self.sug_face)
    }

    /// The suggested-gear tile's base (black chrome round the face).
    pub fn sug_outer_rect(&self) -> Rect {
        Rect::from_center_size(self.sug_c, self.sug_outer)
    }

    /// Pedal slot `i` (0 brake, 1 throttle): its base, or the slot itself when `inner`.
    pub fn slot_rect(&self, i: usize, inner: bool) -> Rect {
        let x = self.slots_x[i.min(1)];
        let size = if inner { self.slot_inner } else { self.slot_outer };
        Rect::from_center_size(vec2(x, self.slot_cy), size)
    }

    /// The fill of pedal slot `i` at `level` (0..=1), rising from the slot's floor.
    pub fn fill_rect(&self, i: usize, level: f32) -> Rect {
        let slot = self.slot_rect(i, true).inset(self.fill_inset);
        let level = if level.is_finite() { level.clamp(0.0, 1.0) } else { 0.0 };
        let h = slot.height() * level;
        Rect::from_xywh(slot.min.x, slot.max.y - h, slot.width(), h)
    }

    /// Tyre lens `i`.
    pub fn pill_rect(&self, i: usize) -> Rect {
        Rect::from_center_size(self.pills[i.min(3)], self.pill_size)
    }

    /// The car chip.
    pub fn car_rect(&self) -> Rect {
        Rect::from_center_size(self.car_c, self.car_size)
    }

    /// The boost gauge's angle (radians) for `bar` (x100 kPa), clamped to the scale.
    pub fn boost_angle(&self, bar: f32) -> f32 {
        let [b0, b1] = self.boost_bar;
        let t = if bar.is_finite() {
            ((bar - b0) / (b1 - b0)).clamp(0.0, 1.0)
        } else {
            ((0.0 - b0) / (b1 - b0)).clamp(0.0, 1.0)
        };
        (self.boost_sweep[0] + t * (self.boost_sweep[1] - self.boost_sweep[0])).to_radians()
    }

    /// The trip tile's face.
    pub fn trip_face_rect(&self) -> Rect {
        Rect::from_center_size(self.trip_c, self.trip_face)
    }

    /// The trip tile's base.
    pub fn trip_outer_rect(&self) -> Rect {
        Rect::from_center_size(self.trip_c, self.trip_outer)
    }

    /// Everything the band draws, for the cockpit fit.
    pub fn bounds(&self) -> Rect {
        let dot = Rect::from_xywh(
            -self.dot_travel - self.dot_r,
            self.dot_y - self.dot_r,
            2.0 * (self.dot_travel + self.dot_r),
            2.0 * self.dot_r,
        );
        self.plate_rect().union(&dot).union(&rect(self.logo))
    }
}

/// How the band is placed in `envelope`: unscaled when it fits (the hood view), otherwise scaled
/// down to fit and centred (the cockpit and chase views). Returns the scale and the offset.
pub(super) fn fit(g: &Geom, envelope: Rect) -> (f32, Vec2) {
    let band = g.bounds();
    let inside = envelope.contains(band.min) && envelope.contains(band.max);
    if inside {
        return (1.0, Vec2::ZERO);
    }
    let k = (envelope.width() / band.width().max(1.0)).min(envelope.height() / band.height().max(1.0));
    if !k.is_finite() || k <= 0.0 {
        return (1.0, Vec2::ZERO);
    }
    let k = k.min(1.0);
    (k, envelope.center() - band.center() * k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_kit::Tokens;

    /// The boost gauge's outer bounds.
    fn boost_rect(g: &Geom) -> Rect {
        Rect::from_center_size(g.boost_c, Vec2::splat(2.0 * (g.boost_r + g.boost_band)))
    }

    /// Telltale `i`'s ring bounds.
    fn lamp_rect(g: &Geom, i: usize) -> Rect {
        Rect::from_center_size(g.lamps[i.min(1)], Vec2::splat(2.0 * g.lamp_ring))
    }

    /// Every zone the band draws, named.
    fn zones(g: &Geom) -> Vec<(&'static str, Rect)> {
        let mut z = vec![
            ("plinth", g.plinth_rect()),
            ("speed", g.speed_field(0.96 * g.speed_cap / 0.75)),
            ("unit", Rect::from_xywh(-200.0, g.unit_top, 120.0, g.unit_cap)),
            ("divider", rect(g.divider)),
            (
                "gear",
                Rect::from_xywh(g.gear_left, g.gear_top, g.gear_max_w, g.gear_cap),
            ),
            ("tile", g.sug_outer_rect()),
            ("car", g.car_rect()),
            ("boost", boost_rect(g)),
            ("trip", g.trip_outer_rect()),
            ("logo", rect(g.logo)),
        ];
        for i in 0..2 {
            z.push(("slot", g.slot_rect(i, false)));
            z.push(("lamp", lamp_rect(g, i)));
        }
        for i in 0..4 {
            z.push(("tyre", g.pill_rect(i)));
        }
        z
    }

    #[test]
    fn every_zone_stays_inside_the_hood_envelope() {
        let env = Tokens::default().layout.hood_envelope;
        let g = &HOOD;
        for (name, r) in zones(g) {
            assert!(
                env.contains(r.min) && env.contains(r.max),
                "{name} {r:?} outside {env:?}"
            );
        }
        let b = g.bounds();
        assert!(env.contains(b.min) && env.contains(b.max), "band {b:?}");
        assert_eq!(fit(g, env), (1.0, Vec2::ZERO), "the hood view draws the band unscaled");
    }

    #[test]
    fn the_band_scales_down_into_the_cockpit_envelope() {
        let env = Tokens::default().layout.cockpit_envelope;
        let (k, off) = fit(&HOOD, env);
        assert!(k > 0.5 && k < 1.0, "scale {k}");
        let b = HOOD.bounds();
        let (a, z) = (b.min * k + off, b.max * k + off);
        assert!(env.contains(a) && env.contains(z), "{a:?}..{z:?} outside {env:?}");
    }

    #[test]
    fn the_zones_keep_clear_of_each_other() {
        let g = &HOOD;
        let z = zones(g);
        for (i, (na, a)) in z.iter().enumerate() {
            for (nb, b) in z.iter().skip(i + 1) {
                if ["car", "tyre"].contains(na) && ["car", "tyre"].contains(nb) {
                    continue; // the tyre lenses sit at the car chip's corners by design
                }
                let gap_x = (b.min.x - a.max.x).max(a.min.x - b.max.x);
                let gap_y = (b.min.y - a.max.y).max(a.min.y - b.max.y);
                assert!(
                    gap_x.max(gap_y) >= 6.0,
                    "{na} {a:?} and {nb} {b:?} are {gap_x} / {gap_y} apart"
                );
            }
        }
    }

    #[test]
    fn the_leds_fill_the_field_inside_the_light_bar() {
        let g = &HOOD;
        let bar = g.bar_rect();
        assert!(g.tick_pitch() >= g.tick_w + 2.0, "each LED keeps a gap to the next");
        for i in [0, g.ticks / 2, g.ticks - 1] {
            let t = g.tick_rect(i);
            assert!(bar.contains(t.min) && bar.contains(t.max), "LED {i} {t:?}");
        }
        // The strobing strip stays within the tokens' 42 px.
        assert!(g.tick_y[1] - g.tick_y[0] <= 42.0);
        // GT7's dense bar: close to a hundred LEDs.
        assert!((90..=100).contains(&g.ticks));
    }

    #[test]
    fn readouts_are_big_enough_to_read() {
        let g = &HOOD;
        // ISO 15008 at 0.9 m: dynamic readouts at least 17.7 px of cap; the hood gear at least
        // 36 px; labels and units at least 22 px (the pilot's lesson).
        for cap in [g.speed_cap, g.trip_cap, g.sug_cap] {
            assert!(cap >= 17.7, "{cap}");
        }
        assert!(g.gear_cap >= 36.0 && g.sug_cap >= 36.0);
        assert!(g.unit_cap >= 22.0 && g.boost_label_cap >= 22.0);
        // The gear is the largest glyph (rank 1).
        assert!(g.gear_cap > g.speed_cap && g.gear_cap > g.sug_cap);
    }

    #[test]
    fn the_sprites_cover_their_parts() {
        let g = &HOOD;
        let plate = g.plate_rect();
        let tile = rect(g.sug_frame);
        assert!(tile.contains(g.sug_outer_rect().min) && tile.contains(g.sug_outer_rect().max));
        for r in [
            g.plinth_rect(),
            g.car_rect(),
            boost_rect(g),
            g.trip_outer_rect(),
            g.slot_rect(0, false),
            g.slot_rect(1, false),
            lamp_rect(g, 0),
            lamp_rect(g, 1),
            g.pill_rect(0),
            g.pill_rect(3),
        ] {
            assert!(
                plate.contains(r.min) && plate.contains(r.max),
                "{r:?} outside the plate"
            );
        }
    }

    #[test]
    fn the_boost_scale_runs_round_the_top() {
        let g = &HOOD;
        // Below the scale and above it clamp to its ends; zero sits on the vacuum side.
        assert!((g.boost_angle(-5.0) - g.boost_sweep[0].to_radians()).abs() < 1e-5);
        assert!((g.boost_angle(9.0) - g.boost_sweep[1].to_radians()).abs() < 1e-5);
        let zero = g.boost_angle(0.0).to_degrees();
        assert!(zero > 180.0 && zero < 270.0, "zero at {zero}°");
        assert!(g.boost_angle(f32::NAN).is_finite());
        // The fill runs inside its groove, and the groove keeps clear of the polished scale.
        assert!(g.boost_track[1] + 2.0 <= g.boost_groove);
        assert!(g.boost_track[0] + g.boost_groove * 0.5 + 1.5 <= g.boost_r - g.boost_band * 0.5);
    }

    /// The asset recipe, which renders the plate and the tile from its own copy of these numbers.
    const RECIPE: &str = include_str!("../../../../../tools/asset-forge/src/recipes/gt7_hood_pro.rs");

    /// The numbers of the recipe's `const NAME` (everything after its `=`, up to its `;`).
    fn recipe(name: &str) -> Vec<f32> {
        let key = format!("const {name}:");
        let start = RECIPE.find(&key).unwrap_or_else(|| panic!("the recipe has no {name}"));
        let rest = &RECIPE[start..];
        let eq = rest.find('=').expect("a const has a value");
        let end = eq + rest[eq..].find(';').expect("a const ends");
        let mut out = Vec::new();
        let mut num = String::new();
        for ch in rest[eq + 1..end].chars().chain([' ']) {
            if ch.is_ascii_digit() || ch == '.' || (ch == '-' && num.is_empty()) {
                num.push(ch);
            } else {
                if !num.is_empty() && num != "-" {
                    out.push(num.parse().unwrap_or_else(|_| panic!("{name}: {num}")));
                }
                num.clear();
            }
        }
        out
    }

    fn flat(points: &[Vec2]) -> Vec<f32> {
        points.iter().flat_map(|p| [p.x, p.y]).collect()
    }

    #[test]
    fn the_recipe_renders_the_same_band() {
        let g = &HOOD;
        let pairs: [(&str, Vec<f32>); 27] = [
            ("PLATE", g.plate.to_vec()),
            ("REV_X", g.rev_x.to_vec()),
            ("TICK_Y", g.tick_y.to_vec()),
            ("TICKS", vec![g.ticks as f32]),
            ("TICK_W", vec![g.tick_w]),
            ("BAR", vec![g.bar_c.y, g.bar_size.x, g.bar_size.y, g.bar_r]),
            ("PLINTH", vec![g.plinth]),
            ("CAR_C", flat(&[g.car_c])),
            ("CAR_WH", flat(&[g.car_size])),
            ("PILL_WH", flat(&[g.pill_size])),
            ("PILLS", flat(&g.pills)),
            ("BOOST_C", flat(&[g.boost_c])),
            ("BOOST_R", vec![g.boost_r]),
            ("BOOST_BAND", vec![g.boost_band]),
            ("BOOST_SWEEP", g.boost_sweep.to_vec()),
            ("BOOST_GROOVE", vec![g.boost_track[0], g.boost_groove]),
            ("SLOTS_X", g.slots_x.to_vec()),
            ("SLOT_CY", vec![g.slot_cy]),
            ("SLOT_OUTER", flat(&[g.slot_outer])),
            ("SLOT_INNER", flat(&[g.slot_inner])),
            ("TELLTALES", flat(&g.lamps)),
            ("TELL_R", vec![g.lamp_face, g.lamp_ring]),
            ("TRIP_C", flat(&[g.trip_c])),
            ("TRIP_OUTER", flat(&[g.trip_outer])),
            ("TRIP_FACE", flat(&[g.trip_face])),
            ("SUG_C", flat(&[g.sug_c])),
            ("SUG_FRAME", g.sug_frame.to_vec()),
        ];
        for (name, want) in pairs {
            assert_eq!(recipe(name), want, "{name}");
        }
        assert_eq!(recipe("SUG_OUTER"), flat(&[g.sug_outer]));
        assert_eq!(recipe("SUG_FACE"), flat(&[g.sug_face]));
        assert!(g.bar_c.x.abs() < 1e-6, "the recipe centres the light bar on x 0");
        // The tile's enamel is the warning role, so the sprite and the fallback agree.
        let warning = Tokens::default().roles.warning;
        let red = recipe("WARNING_RED");
        for (got, want) in red.iter().zip([warning.r, warning.g, warning.b]) {
            assert!((got - want).abs() < 5e-4, "{red:?} against {warning:?}");
        }
    }

    #[test]
    fn pedal_fills_rise_from_the_slot_floor() {
        let g = &HOOD;
        let empty = g.fill_rect(0, 0.0);
        let full = g.fill_rect(0, 1.0);
        assert!(empty.height().abs() < 1e-4);
        assert!((empty.max.y - full.max.y).abs() < 1e-4, "both sit on the floor");
        assert!(g.slot_rect(0, true).contains(full.min) && g.slot_rect(0, true).contains(full.max));
        assert_eq!(g.fill_rect(1, f32::NAN).height(), 0.0);
    }
}
