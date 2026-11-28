//! What the four other tubes show: speed, G, power or boost (each tube's content is an option of
//! the dash), and the driver greeting in the right tube while the car is parked.
//!
//! The side tubes (left and right) show a full dial; the outer tubes show a gauge on the crescent
//! that is not tucked behind the side tube.
use super::hardware::{self, Kind};
use super::{Content, Geom, PorscheAnalog, NUMERALS};
use fh6_kit::{
    vec2, Align, Cap, Color, Glow, Gradient, Painter, Palette, Shape, SpeedUnit, Stroke, TextStyle, VAlign, Vec2,
};

/// Speed scales: top and numeral step (mph: 0 to 220 every 20; km/h: 0 to 360 every 40), with a
/// minor tick at every half step.
const MPH_SCALE: (f32, f32) = (220.0, 20.0);
const KMH_SCALE: (f32, f32) = (360.0, 40.0);
/// Speed numerals, so drawing never formats a string.
const MPH_NUMERALS: [&str; 12] = [
    "0", "20", "40", "60", "80", "100", "120", "140", "160", "180", "200", "220",
];
const KMH_NUMERALS: [&str; 10] = ["0", "40", "80", "120", "160", "200", "240", "280", "320", "360"];
/// A side tube's dial sweep: 270° from 7:30.
const DIAL_START_DEG: f32 = 135.0;
const DIAL_SWEEP_DEG: f32 = 270.0;
/// The speed scale's ticks (length, width): at every numeral, and halfway between.
const SPEED_MAJOR: (f32, f32) = (12.0, 3.5);
const SPEED_MINOR: (f32, f32) = (7.0, 2.0);
/// The outer tubes read on their visible crescent: from 7 o'clock over 9 to 11 on the left, from
/// 5 o'clock over 3 to 1 on the right (degrees, clockwise from 3 o'clock).
const FAR_LEFT_ARC: (f32, f32) = (120.0, 240.0);
const FAR_RIGHT_ARC: (f32, f32) = (60.0, -60.0);
/// The G plot's rings (g) and the radius of its outer ring (px).
const G_RINGS: [f32; 3] = [0.5, 1.0, 1.5];
const G_REACH: f32 = 120.0;
/// The G tube's pedal labels: distance from the centre and angle (degrees; BRK at this angle,
/// THR mirrored), inside the top ends of their arcs.
const G_LABEL: (f32, f32) = (102.0, 236.0);
/// The G magnitude: its drop below the centre and its cap (between the 1 g and 1.5 g rings).
const G_VALUE: (f32, f32) = (98.0, 26.0);
/// Power and boost scales: the top grows with what the car has shown, never below these, in
/// whole steps of these, in the user's unit (hp and psi with mph, kW and bar with km/h).
const POWER_FLOOR_HP: f32 = 300.0;
const POWER_STEP_HP: f32 = 100.0;
const POWER_FLOOR_KW: f32 = 200.0;
const POWER_STEP_KW: f32 = 50.0;
const BOOST_FLOOR_PSI: f32 = 20.0;
const BOOST_STEP_PSI: f32 = 5.0;
const BOOST_FLOOR_BAR: f32 = 1.5;
const BOOST_STEP_BAR: f32 = 0.5;
const BOOST_VACUUM_PSI: f32 = -15.0;
/// The G tube's pedal arcs: their width and their ink level (rank 4: under the numerals, but
/// still 3:1 in the glance view).
const PEDAL_ARC: (f32, f32) = (6.0, 0.62);
/// The greeting's name: its cap, the cap it drops to when the name is long, and the widest it
/// may be (px), inside the side tube's opening.
const GREET_CAP: (f32, f32) = (52.0, 34.0);
const GREET_WIDTH: f32 = 250.0;
pub(super) const HP_TO_KW: f32 = 0.745_7;
pub(super) const PSI_TO_BAR: f32 = 0.068_947_6;
/// The outer tubes' value: its centre's distance from the tube's centre towards the visible
/// crescent (px), and its cap.
const FAR_VALUE: (f32, f32) = (26.0, 26.0);
/// Where an outer tube's value may sit, across the tube towards its visible crescent (px): the
/// side tube in front covers the tube past −32, and the long tick at 3 or 9 o'clock starts at
/// 70. A value wider than this span is drawn smaller.
const FAR_VALUE_SPAN: (f32, f32) = (-28.0, 64.0);
/// A true minus sign, so a vacuum never reads as a tick.
const MINUS: &str = "\u{2212}";
/// The outer gauges' end values: how far inside the arc they sit (px) and their cap. Each sits
/// past its end of the arc by half its own width and [`FAR_END_CLEAR`], so it never meets the
/// end tick or the pointer resting at that end.
const FAR_END: (f32, f32) = (8.0, 13.0);
const FAR_END_CLEAR: f32 = 6.0;
/// The decimal point's share of a tabular cell.
const POINT_CELL: f32 = 0.35;

/// What an outer gauge shows: the pointer's fraction, the value, its decimals and unit, and the
/// scale's end values with their decimals.
struct FarGauge<'a> {
    frac: f32,
    value: f32,
    decimals: u32,
    unit: &'a str,
    ends: ([f32; 2], u32),
}

/// A tube slot: the side tubes are 1 (left) and 2 (right), the outer tubes 0 and 3.
fn is_side(slot: usize) -> bool {
    slot == 1 || slot == 2
}

impl PorscheAnalog {
    /// Draws tube `slot` (0 far left .. 3 far right): its hardware and its content.
    pub(super) fn draw_tube(&self, p: &mut Painter<'_>, g: &Geom, slot: usize, pal: &Palette, w: [f32; 3]) {
        let Some(&content) = self.opts.tubes.get(slot) else {
            return;
        };
        if content == Content::Off {
            return;
        }
        let Some(&c) = g.tubes.get(slot) else {
            return;
        };
        let side = is_side(slot);
        if side {
            // A soft contact shadow where the side tube's bezel overhangs the outer tube.
            p.shadow(
                &Shape::circle(c, g.side_outer - 2.0),
                fh6_kit::Shadow::new(vec2(0.0, 5.0), 10.0, pal.roles.oled_black.with_alpha(0.75)),
            );
        }
        let drawn = if side {
            self.sprites.draw(p, Kind::Side, c, w)
        } else {
            self.sprites.draw(p, Kind::Far, c, w)
        };
        if !drawn {
            let (open, outer, ring) = if side {
                (g.side_open, g.side_outer, g.side_scale_in)
            } else {
                (g.far_open, g.far_outer, 0.0)
            };
            hardware::fallback_tube(p, c, open, outer, ring, pal);
        }
        // The printed scales and the live data are always at full strength (only the glass
        // materialises); the greeting and the live content take turns in the right tube.
        let greet = if slot == 2 {
            self.greet.value().clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (content_alpha, greet_alpha) = turns(greet);
        if content_alpha > 0.001 {
            p.scoped(|p| {
                p.set_opacity(content_alpha);
                if side {
                    self.draw_side_content(p, g, c, content, pal);
                } else {
                    let sign = if slot == 0 { -1.0 } else { 1.0 };
                    self.draw_far_content(p, g, c, sign, content, pal);
                }
            });
        }
        if greet_alpha > 0.001 {
            p.scoped(|p| {
                p.set_opacity(greet_alpha);
                self.draw_greeting(p, g, c, pal);
            });
        }
    }

    /// The speed scale for the user's unit: top, numeral step and the numerals.
    fn speed_scale(&self) -> (f32, f32, &'static [&'static str]) {
        match self.sig.unit {
            SpeedUnit::Mph => (MPH_SCALE.0, MPH_SCALE.1, &MPH_NUMERALS),
            SpeedUnit::Kmh => (KMH_SCALE.0, KMH_SCALE.1, &KMH_NUMERALS),
        }
    }

    /// Power in the user's unit (hp with mph, kW with km/h) as the digits show it, its label,
    /// and the gauge fraction (from the smoothed value, so the pointer moves smoothly).
    fn power_reading(&self) -> (f32, &'static str, f32) {
        let hp = self.power_hp.value().max(0.0);
        let top = self.power_top.max(1.0);
        let frac = (hp / top).clamp(0.0, 1.0);
        let digits = self.power_digits.value().max(0.0);
        match self.sig.unit {
            SpeedUnit::Mph => (digits, "HP", frac),
            SpeedUnit::Kmh => (digits * HP_TO_KW, "kW", frac),
        }
    }

    /// Boost in the user's unit (psi with mph, bar with km/h) as the digits show it, its label,
    /// and the gauge fraction (from full vacuum to the top).
    fn boost_reading(&self) -> (f32, &'static str, f32) {
        let psi = self.boost_psi.value();
        let top = self.boost_top.max(BOOST_STEP_BAR / PSI_TO_BAR);
        let frac = ((psi - BOOST_VACUUM_PSI) / (top - BOOST_VACUUM_PSI)).clamp(0.0, 1.0);
        let digits = self.boost_digits.value();
        match self.sig.unit {
            SpeedUnit::Mph => (digits, "PSI", frac),
            SpeedUnit::Kmh => (digits * PSI_TO_BAR, "BAR", frac),
        }
    }

    /// The G magnitude (smoothed signals) and the gauge fraction. The digits show it through
    /// their own readout (`g_digits`).
    pub(super) fn g_reading(&self) -> (f32, f32) {
        let mag = (self.sig.g_lat * self.sig.g_lat + self.sig.g_long * self.sig.g_long).sqrt();
        let mag = if mag.is_finite() { mag } else { 0.0 };
        (mag, (mag / G_RINGS[2]).clamp(0.0, 1.0))
    }

    /// A side tube's full dial.
    fn draw_side_content(&self, p: &mut Painter<'_>, g: &Geom, c: Vec2, content: Content, pal: &Palette) {
        match content {
            Content::Speed => self.draw_speed_side(p, g, c, pal),
            Content::GForce => self.draw_g_side(p, g, c, pal),
            Content::Power => {
                let (v, unit, frac) = self.power_reading();
                self.draw_gauge_side(p, g, c, frac, v, 0, unit, pal);
            }
            Content::Boost => {
                let (v, unit, frac) = self.boost_reading();
                self.draw_gauge_side(p, g, c, frac, v, 1, unit, pal);
            }
            Content::Off => {}
        }
    }

    /// An outer tube's crescent gauge: the pointer from the smoothed value, the digits from the
    /// same readouts the side tubes use, so no digit flickers.
    fn draw_far_content(&self, p: &mut Painter<'_>, g: &Geom, c: Vec2, sign: f32, content: Content, pal: &Palette) {
        let (frac, value, decimals, unit) = match content {
            Content::Speed => {
                let (top, _, _) = self.speed_scale();
                let pointer = self.speed_needle.value().max(0.0);
                let digits = self.speed.value().clamp(0, 999) as f32;
                ((pointer / top).clamp(0.0, 1.0), digits, 0, self.sig.unit.label())
            }
            Content::GForce => {
                let (_, frac) = self.g_reading();
                (frac, self.g_digits.value(), 2, "G")
            }
            Content::Power => {
                let (v, unit, frac) = self.power_reading();
                (frac, v, 0, unit)
            }
            Content::Boost => {
                let (v, unit, frac) = self.boost_reading();
                (frac, v, 1, unit)
            }
            Content::Off => return,
        };
        let gauge = FarGauge {
            frac,
            value,
            decimals,
            unit,
            ends: self.far_ends(content),
        };
        self.draw_gauge_far(p, g, c, sign, &gauge, pal);
    }

    /// The values at the two ends of an outer gauge's scale, in the user's unit, and their
    /// decimals.
    fn far_ends(&self, content: Content) -> ([f32; 2], u32) {
        let kmh = self.sig.unit == SpeedUnit::Kmh;
        match content {
            Content::Speed => ([0.0, self.speed_scale().0], 0),
            Content::GForce => ([0.0, G_RINGS[2]], 1),
            Content::Power => {
                let top = self.power_top;
                ([0.0, if kmh { top * HP_TO_KW } else { top }], 0)
            }
            Content::Boost => {
                let top = self.boost_top;
                if kmh {
                    ([BOOST_VACUUM_PSI * PSI_TO_BAR, top * PSI_TO_BAR], 1)
                } else {
                    ([BOOST_VACUUM_PSI, top], 0)
                }
            }
            Content::Off => ([0.0, 0.0], 0),
        }
    }

    /// Speed: a 270° dial with numerals every step, the rendered pointer, the digits and the unit.
    fn draw_speed_side(&self, p: &mut Painter<'_>, g: &Geom, c: Vec2, pal: &Palette) {
        let (top, step, numerals) = self.speed_scale();
        let ink = self.numeral_ink(pal);
        let angle = |v: f32| (DIAL_START_DEG + DIAL_SWEEP_DEG * (v / top).clamp(0.0, 1.04)).to_radians();
        let minors = (top / (step * 0.5)).round() as u32;
        for k in 0..=minors {
            let a = angle(k as f32 * step * 0.5);
            let ((len, width), level) = if k % 2 == 0 {
                (SPEED_MAJOR, super::LEVEL_NUMERAL)
            } else {
                (SPEED_MINOR, super::LEVEL_MINOR)
            };
            tick(p, c, g.side_tick_out, len, width, a, pal.roles.ink.with_alpha(level));
        }
        let style = TextStyle::new(self.faces.dial, g.side_num_cap)
            .paint(ink.with_alpha(0.78))
            .align(Align::Center, VAlign::CapMiddle);
        // Each numeral tucked inside the bars by its own width, so none touches a tick.
        let inner = g.side_tick_out - SPEED_MAJOR.0;
        for (i, label) in numerals.iter().enumerate() {
            let a = angle(i as f32 * step);
            let at = super::tucked(p, c, inner, a, label, &style);
            p.text(at, label, &style);
        }
        self.draw_pointer(p, c, angle(self.speed_needle.value()), g.pointer, pal);
        // The digits: tabular, centred by their count (the field never jitters within a count).
        let value = self.speed.value().clamp(0, 999);
        let cells = digits(value);
        let digits_style = TextStyle::new(self.faces.speed, g.speed_cap)
            .paint(pal.roles.ink.with_alpha(0.94))
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        p.number(c + vec2(0.0, -6.0), value, cells, &digits_style, None);
        let unit = TextStyle::new(self.faces.label, self.unit_cap * 0.8)
            .paint(pal.roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapTop);
        p.text(c + vec2(0.0, g.speed_cap * 0.5 + 18.0), self.sig.unit.label(), &unit);
    }

    /// G: the felt-force dot over rings at 0.5, 1 and 1.5 g, throttle and brake arcs on the
    /// edge, and the magnitude.
    fn draw_g_side(&self, p: &mut Painter<'_>, g: &Geom, c: Vec2, pal: &Palette) {
        let roles = pal.roles;
        for (i, ring) in G_RINGS.iter().enumerate() {
            let alpha = if i == 1 { 0.24 } else { 0.13 };
            p.stroke(
                &Shape::circle(c, G_REACH * ring / G_RINGS[2]),
                roles.ink.with_alpha(alpha),
                Stroke::rim_px(1.5),
            );
        }
        for a in [0.0_f32, 90.0, 180.0, 270.0] {
            tick(
                p,
                c,
                G_REACH + 8.0,
                16.0,
                2.0,
                a.to_radians(),
                roles.ink.with_alpha(0.3),
            );
        }
        // Brake up the left edge and throttle up the right, as the pedals sit.
        let r = g.side_tick_out - 8.0;
        let brk_start = 135f32.to_radians();
        let thr_start = 45f32.to_radians();
        let quarter = 90f32.to_radians();
        let (width, level) = PEDAL_ARC;
        let track = roles.ink.with_alpha(0.10);
        p.fill(&Shape::arc(c, r, width, brk_start, quarter, Cap::Round), track);
        p.fill(&Shape::arc(c, r, width, thr_start, -quarter, Cap::Round), track);
        let thr = clamp01(self.sig.throttle);
        let brk = clamp01(self.sig.brake);
        // In the dial's own ink, told apart by side and label: green and red stay the shift
        // lenses' own colours.
        let lit = roles.ink.with_alpha(level);
        if brk > 0.01 {
            p.fill(&Shape::arc(c, r, width, brk_start, quarter * brk, Cap::Round), lit);
        }
        if thr > 0.01 {
            p.fill(&Shape::arc(c, r, width, thr_start, -quarter * thr, Cap::Round), lit);
        }
        let labels = TextStyle::new(self.faces.label, self.micro_cap * 0.8)
            .paint(roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapMiddle);
        // The labels sit inside the arcs' top ends, between the outer two rings, where nothing
        // else is written.
        p.text(Vec2::polar(c, G_LABEL.0, G_LABEL.1.to_radians()), "BRK", &labels);
        p.text(
            Vec2::polar(c, G_LABEL.0, (180.0 - G_LABEL.1).to_radians()),
            "THR",
            &labels,
        );
        // The dot: where the force is felt (rank 4: a small glow, never bloom).
        let dot = c + vec2(self.g[0].value(), self.g[1].value()) * G_REACH;
        p.glow(&Shape::circle(dot, 7.0), pal.primary, Glow::new(5.0, 0.35));
        p.fill(&Shape::circle(dot, 7.0), pal.primary_text);
        // The magnitude between the outer two rings at the bottom, with its unit after it: the
        // digits' face at full ink, so a small reading survives a glance.
        let value = TextStyle::new(self.faces.speed, G_VALUE.1)
            .paint(roles.ink)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        let unit = TextStyle::new(self.faces.label, self.micro_cap * 0.8)
            .paint(roles.ink_label)
            .align(Align::Left, VAlign::CapMiddle);
        let at = c + vec2(-6.0, G_VALUE.0);
        let field = fixed(p, at, self.g_digits.value(), 2, &value);
        p.text(vec2(field.max.x + 5.0, at.y), "G", &unit);
    }

    /// Power or boost in a side tube: a 270° scale, the rendered pointer and the value.
    #[allow(clippy::too_many_arguments)]
    fn draw_gauge_side(
        &self,
        p: &mut Painter<'_>,
        g: &Geom,
        c: Vec2,
        frac: f32,
        value: f32,
        decimals: u32,
        unit: &str,
        pal: &Palette,
    ) {
        let angle = |f: f32| (DIAL_START_DEG + DIAL_SWEEP_DEG * clamp01(f)).to_radians();
        for k in 0..=20u32 {
            let (len, width, level) = if k % 5 == 0 {
                (14.0, 3.5, super::LEVEL_NUMERAL)
            } else {
                (8.0, 2.0, super::LEVEL_MINOR)
            };
            tick(
                p,
                c,
                g.side_tick_out,
                len,
                width,
                angle(k as f32 / 20.0),
                pal.roles.ink.with_alpha(level),
            );
        }
        self.draw_pointer(p, c, angle(frac), g.pointer, pal);
        let style = TextStyle::new(self.faces.speed, g.speed_cap * 0.7)
            .paint(pal.roles.ink.with_alpha(0.94))
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        fixed(p, c + vec2(0.0, -4.0), value, decimals, &style);
        let label = TextStyle::new(self.faces.label, self.unit_cap * 0.8)
            .paint(pal.roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapTop);
        p.text(c + vec2(0.0, g.speed_cap * 0.35 + 18.0), unit, &label);
    }

    /// An outer tube's gauge on its visible crescent: the arc, five ticks, the scale's end values
    /// just past its ends, the pointer, and the value beside it.
    fn draw_gauge_far(&self, p: &mut Painter<'_>, g: &Geom, c: Vec2, sign: f32, gauge: &FarGauge<'_>, pal: &Palette) {
        let (a0, a1) = if sign < 0.0 { FAR_LEFT_ARC } else { FAR_RIGHT_ARC };
        let (a0, a1) = (a0.to_radians(), a1.to_radians());
        let angle = |f: f32| a0 + (a1 - a0) * clamp01(f);
        p.fill(
            &Shape::arc(c, g.far_arc_r, 2.0, a0, a1 - a0, Cap::Butt),
            pal.roles.ink.with_alpha(super::LEVEL_MINOR),
        );
        for k in 0..=4u32 {
            let len = if k % 2 == 0 { 10.0 } else { 6.0 };
            tick(
                p,
                c,
                g.far_arc_r + 4.0,
                len,
                2.5,
                angle(k as f32 / 4.0),
                pal.roles.ink.with_alpha(super::LEVEL_NUMERAL),
            );
        }
        // The scale's two end values, past the arc's ends by half their width and a clear gap,
        // where neither the end tick nor the pointer at rest there can touch them.
        let ends = TextStyle::new(self.faces.dial, FAR_END.1)
            .paint(self.numeral_ink(pal).with_alpha(super::LEVEL_HALF))
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        let end_cell = p.fonts().tabular_cell(&ends);
        let r_end = g.far_arc_r - FAR_END.0;
        let outward = (a1 - a0).signum();
        let (values, places) = gauge.ends;
        for (value, end, away) in [(values[0], a0, -outward), (values[1], a1, outward)] {
            let past = end_past(Digits::of(value, places).width(end_cell), r_end);
            fixed(p, Vec2::polar(c, r_end, end + away * past), value, places, &ends);
        }
        self.draw_pointer(p, c, angle(gauge.frac), g.far_pointer, pal);
        // The value on the clear face between the pointer's root and the side tube's bezel,
        // inside the visible span (smaller when it is wider than the span).
        let style = TextStyle::new(self.faces.speed, FAR_VALUE.1)
            .paint(pal.roles.ink)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        let width = Digits::of(gauge.value, gauge.decimals).width(p.fonts().tabular_cell(&style));
        let (x, fit) = far_value_fit(width);
        let style = TextStyle {
            cap: style.cap * fit,
            ..style
        };
        let at = c + vec2(sign * x, -6.0);
        fixed(p, at, gauge.value, gauge.decimals, &style);
        let label = TextStyle::new(self.faces.label, self.nano_cap)
            .paint(pal.roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapTop);
        p.text(at + vec2(0.0, FAR_VALUE.1 * 0.5 + 8.0), gauge.unit, &label);
    }

    /// A pointer from `blade.0` to `blade.1` out from `c` at `angle`: the rendered lacquer sprite
    /// tinted with the palette, or a vector blade.
    fn draw_pointer(&self, p: &mut Painter<'_>, c: Vec2, angle: f32, blade: (f32, f32), pal: &Palette) {
        let tint = self.needle_ink(pal);
        let dir = Vec2::polar(Vec2::ZERO, 1.0, angle);
        let shape = Shape::Line {
            a: c + dir * blade.0,
            b: c + dir * blade.1,
            width_a: 6.0,
            width_b: 1.5,
            cap: Cap::Round,
        };
        let away = Vec2::polar(Vec2::ZERO, 4.0, self.light.angle() + core::f32::consts::PI);
        p.shadow(
            &shape,
            fh6_kit::Shadow::new(away, 2.5, pal.roles.oled_black.with_alpha(0.7)),
        );
        if !self
            .sprites
            .draw_pointer(p, c, angle, self.light.angle(), blade.0, blade.1, tint)
        {
            p.fill(&shape, tint);
        }
    }

    /// "HELLO," and the driver's name (the config's driver name, which shows whether or not the
    /// logo does, else the brand, else "DRIVER 1"), as the 992 greets its driver profile. A long
    /// name shrinks to fit and never leaves the tube's opening. The name stays a step under the
    /// gear's ink, so a parked car's gear still leads.
    fn draw_greeting(&self, p: &mut Painter<'_>, g: &Geom, c: Vec2, pal: &Palette) {
        let hello = TextStyle::new(self.faces.label, self.micro_cap * 1.1)
            .paint(pal.roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapMiddle);
        p.text(c + vec2(0.0, -40.0), "HELLO,", &hello);
        let name = greeting_name(&self.driver, &self.logo);
        let style = TextStyle::new(self.faces.speed, GREET_CAP.0)
            .paint(pal.roles.ink.with_alpha(0.8))
            .align(Align::Center, VAlign::CapMiddle);
        let wide = p.measure(name, &style).width;
        let style = if wide <= GREET_WIDTH {
            style
        } else {
            // The smaller cap, then down in proportion until it fits.
            let at_small = wide * GREET_CAP.1 / GREET_CAP.0;
            let cap = if at_small <= GREET_WIDTH {
                GREET_CAP.1
            } else {
                GREET_CAP.1 * GREET_WIDTH / at_small.max(1.0)
            };
            TextStyle { cap, ..style }
        };
        p.scoped(|p| {
            p.clip(
                fh6_kit::Rect::from_center_size(c, Vec2::splat(g.side_open * 2.0)),
                g.side_open,
            );
            p.text(c + vec2(0.0, 16.0), name, &style);
        });
        p.fill(
            &Shape::Line {
                a: c + vec2(-60.0, 64.0),
                b: c + vec2(60.0, 64.0),
                width_a: 1.5,
                width_b: 1.5,
                cap: Cap::Butt,
            },
            Gradient::linear(c + vec2(-60.0, 64.0), c + vec2(60.0, 64.0))
                .stop(0.0, pal.primary.with_alpha(0.0))
                .stop(0.5, pal.primary)
                .stop(1.0, pal.primary.with_alpha(0.0)),
        );
    }
}

/// How much of the live content and of the greeting show at greeting level `greet` (0 driving,
/// 1 parked). They take turns: one fades out over the first half of the change and the other in
/// over the second, so they never show at once.
fn turns(greet: f32) -> (f32, f32) {
    let g = clamp01(greet);
    (((1.0 - g) * 2.0 - 1.0).clamp(0.0, 1.0), (g * 2.0 - 1.0).clamp(0.0, 1.0))
}

/// A radial tick ending at `r_out`, `len` long.
fn tick(p: &mut Painter<'_>, c: Vec2, r_out: f32, len: f32, width: f32, angle: f32, color: Color) {
    p.fill(
        &Shape::Line {
            a: Vec2::polar(c, r_out - len, angle),
            b: Vec2::polar(c, r_out, angle),
            width_a: width.max(1.0),
            width_b: width.max(1.0),
            cap: Cap::Butt,
        },
        color,
    );
}

/// `v` clamped to 0..=1 (0 for NaN).
fn clamp01(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// The smallest multiple of `step` at or above `v`.
fn nice(v: f32, step: f32) -> f32 {
    let s = step.max(1e-3);
    ((v / s).ceil() * s).max(s)
}

/// The printed top of the power scale (hp) for a peak of `peak` hp: a whole step of the user's
/// unit (100 hp, or 50 kW with km/h), never below the floor.
pub(super) fn power_top(peak: f32, unit: SpeedUnit) -> f32 {
    let peak = if peak.is_finite() { peak.max(0.0) } else { 0.0 };
    match unit {
        SpeedUnit::Mph => nice(peak.max(POWER_FLOOR_HP), POWER_STEP_HP),
        SpeedUnit::Kmh => nice((peak * HP_TO_KW).max(POWER_FLOOR_KW), POWER_STEP_KW) / HP_TO_KW,
    }
}

/// The printed top of the boost scale (psi) for a peak of `peak` psi: a whole step of the
/// user's unit (5 psi, or 0.5 bar with km/h), never below the floor.
pub(super) fn boost_top(peak: f32, unit: SpeedUnit) -> f32 {
    let peak = if peak.is_finite() { peak } else { 0.0 };
    match unit {
        SpeedUnit::Mph => nice(peak.max(BOOST_FLOOR_PSI), BOOST_STEP_PSI),
        SpeedUnit::Kmh => nice((peak * PSI_TO_BAR).max(BOOST_FLOOR_BAR), BOOST_STEP_BAR) / PSI_TO_BAR,
    }
}

/// Digits in a non-negative integer (1 for 0).
fn digits(v: i64) -> u8 {
    let mut n = v.unsigned_abs();
    let mut d = 1u8;
    while n >= 10 {
        n /= 10;
        d += 1;
    }
    d
}

/// A number laid out as tabular digits: its sign, whole and fractional parts, and the cells they
/// take (at most three places; values clamped to −999..=9999, NaN shown as 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Digits {
    negative: bool,
    whole: u64,
    frac: u64,
    whole_digits: u32,
    decimals: u32,
}

impl Digits {
    fn of(value: f32, decimals: u32) -> Self {
        let decimals = decimals.min(3);
        let v = if value.is_finite() {
            value.clamp(-999.0, 9999.0)
        } else {
            0.0
        };
        let scale = 10u64.pow(decimals);
        let n = (v * scale as f32).round() as i64;
        let whole = n.unsigned_abs() / scale;
        Self {
            negative: n < 0,
            whole,
            frac: n.unsigned_abs() % scale,
            whole_digits: u32::from(digits(whole as i64)),
            decimals,
        }
    }

    /// The field's width for tabular cells `cell` px wide (the point takes [`POINT_CELL`]).
    fn width(&self, cell: f32) -> f32 {
        let cells = (self.whole_digits + self.decimals + u32::from(self.negative)) as f32;
        let point = if self.decimals > 0 { POINT_CELL } else { 0.0 };
        cell * (cells + point)
    }

    /// Digit `i` of the whole part, counting from the left.
    fn whole_digit(&self, i: u32) -> usize {
        ((self.whole / 10u64.pow(self.whole_digits - 1 - i)) % 10) as usize
    }

    /// Digit `i` of the fractional part, counting from the left.
    fn frac_digit(&self, i: u32) -> usize {
        ((self.frac / 10u64.pow(self.decimals - 1 - i)) % 10) as usize
    }
}

/// How far past its end of the arc (radians) an end value `width` px wide sits at radius `r`, so
/// its near edge clears the end by [`FAR_END_CLEAR`].
fn end_past(width: f32, r: f32) -> f32 {
    (width * 0.5 + FAR_END_CLEAR) / r.max(1.0)
}

/// Where an outer tube's value `width` px wide sits (its centre across the tube, towards the
/// visible crescent) and the share of its cap it keeps, inside [`FAR_VALUE_SPAN`].
fn far_value_fit(width: f32) -> (f32, f32) {
    let (lo, hi) = FAR_VALUE_SPAN;
    let span = hi - lo;
    if width <= span {
        (FAR_VALUE.0.clamp(lo + width * 0.5, hi - width * 0.5), 1.0)
    } else {
        ((lo + hi) * 0.5, span / width.max(1.0))
    }
}

/// Draws `value` with `decimals` places (tabular, digit by digit, no allocation), centred at
/// `centre`. Returns the drawn field.
fn fixed(p: &mut Painter<'_>, centre: Vec2, value: f32, decimals: u32, style: &TextStyle) -> fh6_kit::Rect {
    let d = Digits::of(value, decimals);
    let cell = p.fonts().tabular_cell(style);
    let width = d.width(cell);
    let mut x = centre.x - width * 0.5;
    let one = TextStyle {
        align: Align::Center,
        ..*style
    };
    let mut put = |p: &mut Painter<'_>, s: &str, w: f32| {
        p.text(vec2(x + w * 0.5, centre.y), s, &one);
        x += w;
    };
    if d.negative {
        put(p, MINUS, cell);
    }
    for i in 0..d.whole_digits {
        put(p, NUMERALS[d.whole_digit(i)], cell);
    }
    if d.decimals > 0 {
        put(p, ".", cell * POINT_CELL);
        for i in 0..d.decimals {
            put(p, NUMERALS[d.frac_digit(i)], cell);
        }
    }
    fh6_kit::Rect::from_xywh(centre.x - width * 0.5, centre.y - style.cap * 0.5, width, style.cap)
}

/// The name the greeting shows: the config's driver name, else the car's brand, else
/// "DRIVER 1". The brand is there while its dial is still loading ([`fh6_kit::Logo::loading`]), so a
/// parked car greets its brand from the first frame and never "DRIVER 1" first.
pub(super) fn greeting_name<'a>(driver: &'a str, logo: &'a fh6_kit::Logo) -> &'a str {
    if !driver.trim().is_empty() {
        driver.trim()
    } else if !logo.brand_name.trim().is_empty() {
        logo.brand_name.trim()
    } else {
        "DRIVER 1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_split_signs_places_and_rounding() {
        let d = Digits::of(-15.0, 0);
        assert!(d.negative && d.whole == 15 && d.whole_digits == 2 && d.decimals == 0);
        assert_eq!((d.whole_digit(0), d.whole_digit(1)), (1, 5));
        // A tiny negative rounds to an unsigned zero; half a step rounds away from zero.
        assert!(!Digits::of(-0.04, 1).negative);
        let d = Digits::of(-0.05, 1);
        assert!(d.negative && d.whole == 0 && d.frac == 1);
        let d = Digits::of(12.345, 2);
        assert_eq!((d.whole, d.frac, d.frac_digit(0), d.frac_digit(1)), (12, 35, 3, 5));
        // At most three places, clamped values, NaN as zero.
        assert_eq!(Digits::of(1.0, 7).decimals, 3);
        assert_eq!(Digits::of(1.0e6, 0).whole, 9999);
        assert_eq!(Digits::of(f32::NAN, 1), Digits::of(0.0, 1));
        // Width: three cells for "−15", two cells and a point for "5.6".
        assert!((Digits::of(-15.0, 0).width(10.0) - 30.0).abs() < 1e-4);
        assert!((Digits::of(5.6, 1).width(10.0) - (20.0 + 10.0 * POINT_CELL)).abs() < 1e-4);
    }

    #[test]
    fn end_values_clear_the_end_ticks_and_the_resting_pointer() {
        // "−15", "300" and "−1.0" at the end values' cap (a cell of about 11 px): the near edge
        // of each stays FAR_END_CLEAR past the end of the arc, where the tick and the pointer at
        // rest stand.
        let r = 76.0 - FAR_END.0;
        let cell = 11.14;
        for (v, places) in [(-15.0, 0), (300.0, 0), (-1.03, 1), (1500.0, 0)] {
            let w = Digits::of(v, places).width(cell);
            let past = end_past(w, r);
            let near_edge = past * r - w * 0.5;
            assert!((near_edge - FAR_END_CLEAR).abs() < 1e-3, "{v}: near edge {near_edge}");
        }
    }

    #[test]
    fn outer_values_stay_inside_their_visible_span() {
        let (lo, hi) = FAR_VALUE_SPAN;
        for width in [20.0, 60.0, 75.0, 92.0, 97.0, 140.0] {
            let (x, fit) = far_value_fit(width);
            let w = width * fit;
            assert!(
                x - w * 0.5 >= lo - 1e-3 && x + w * 0.5 <= hi + 1e-3,
                "{width}: {x} {fit}"
            );
            assert!(fit <= 1.0);
        }
        // A narrow value keeps its place and size.
        assert_eq!(far_value_fit(40.0), (FAR_VALUE.0, 1.0));
    }

    #[test]
    fn nice_rounds_up_to_the_step() {
        assert_eq!(nice(310.0, 100.0), 400.0);
        assert_eq!(nice(300.0, 100.0), 300.0);
        assert_eq!(nice(0.0, 5.0), 5.0);
        assert_eq!(nice(21.0, 5.0), 25.0);
    }

    #[test]
    fn the_power_and_boost_tops_end_on_a_round_step_of_the_users_unit() {
        let close = |a: f32, b: f32| (a - b).abs() < 1e-3;
        // mph: hp and psi.
        assert_eq!(power_top(0.0, SpeedUnit::Mph), 300.0);
        assert_eq!(power_top(391.0, SpeedUnit::Mph), 400.0);
        assert_eq!(boost_top(5.6, SpeedUnit::Mph), 20.0);
        assert_eq!(boost_top(21.0, SpeedUnit::Mph), 25.0);
        // km/h: the printed ends are whole 50 kW and 0.5 bar steps.
        assert!(close(power_top(0.0, SpeedUnit::Kmh) * HP_TO_KW, 200.0));
        assert!(close(power_top(391.0, SpeedUnit::Kmh) * HP_TO_KW, 300.0));
        assert!(close(boost_top(5.6, SpeedUnit::Kmh) * PSI_TO_BAR, 1.5));
        assert!(close(boost_top(30.0, SpeedUnit::Kmh) * PSI_TO_BAR, 2.5));
        // Bad input falls back to the floors.
        assert_eq!(power_top(f32::NAN, SpeedUnit::Mph), 300.0);
        assert_eq!(boost_top(f32::INFINITY, SpeedUnit::Mph), 20.0);
    }

    #[test]
    fn the_greeting_and_the_live_content_never_show_at_once() {
        for k in 0..=100 {
            let (content, greeting) = turns(k as f32 / 100.0);
            assert!(content == 0.0 || greeting == 0.0, "both show at {k}%");
        }
        assert_eq!(turns(0.0), (1.0, 0.0));
        assert_eq!(turns(1.0), (0.0, 1.0));
        assert_eq!(turns(f32::NAN), (1.0, 0.0));
    }

    #[test]
    fn digits_counts() {
        assert_eq!(digits(0), 1);
        assert_eq!(digits(9), 1);
        assert_eq!(digits(118), 3);
        assert_eq!(digits(1000), 4);
    }

    #[test]
    fn the_speed_numerals_match_their_scales() {
        assert_eq!(MPH_NUMERALS.len() as f32, MPH_SCALE.0 / MPH_SCALE.1 + 1.0);
        assert_eq!(KMH_NUMERALS.len() as f32, KMH_SCALE.0 / KMH_SCALE.1 + 1.0);
        assert_eq!(MPH_NUMERALS.last(), Some(&"220"));
        assert_eq!(KMH_NUMERALS.last(), Some(&"360"));
    }

    #[test]
    fn the_far_gauges_stay_on_the_visible_crescent() {
        // Left: 7 to 11 o'clock (all on the outer, left half); right mirrored.
        for (a0, a1, outward) in [
            (FAR_LEFT_ARC.0, FAR_LEFT_ARC.1, -1.0),
            (FAR_RIGHT_ARC.0, FAR_RIGHT_ARC.1, 1.0),
        ] {
            for k in 0..=8 {
                let a = (a0 + (a1 - a0) * k as f32 / 8.0).to_radians();
                assert!(a.cos() * outward > 0.4, "{a0}..{a1} at {k}: cos {}", a.cos());
            }
        }
    }
}
