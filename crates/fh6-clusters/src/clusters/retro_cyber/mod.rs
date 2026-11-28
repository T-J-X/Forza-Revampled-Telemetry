//! Retro Cyber (`ClusterTheme::RetroCyber`), catalog row 19: the Sci-Fi family's pattern-setter
//! (VFD dialect), cockpit + hood, build #6.
//!
//! A 2027 production revival of the **Toyota Supra A70 "Panoramic Digital Meter"** (1986–1993),
//! the full-width vacuum-fluorescent panel Toyota's catalogue sold with "large VFD tubes in five
//! colours" and a "horizontal tachometer" (`RESEARCH.md`). One long smoked lens in a slim
//! graphite surround; behind it four separate VFD windows on true black, in the A70's order:
//!
//! 1. **Pedal window** (the A70's fuel window): two columns of thick horizontal segments,
//!    throttle and brake.
//! 2. **Centre window:** the horizontal tach (one upright bar per 250 rpm, the bar tops on a curve
//!    that rises to the right, numerals on the same curve, red numerals in the red zone under a
//!    bracket), the seven-segment speed below it with ghost 8s, lateral G where the A70 shows its
//!    trip, and the gear (white phosphor) under the red zone.
//! 3. **Slip window** (the A70's temperature bar): body slip angle, slanted top.
//! 4. **Gauge window** (the A70's oil and turbo slants): engine power and boost.
//!
//! The VFD layers: phosphor (unlit elements are opaque 7% ghosts of their own phosphor), the
//! control-grid mesh and the filaments multiplied over the windows (so they modulate lit phosphor
//! only), the 2 px scanlines, then the lens as optical glass (`fh6_kit::OpticalGlass`,
//! `docs/specs/optics.md`): clear over black, its rims bending the windows' edges, its highlight
//! following the virtual light.
//!
//! Ranks:
//! - 1: the gear (cockpit cap 190) and the shift bracket over the approach to the shift point
//!   (pods fill left to right with the bars under them; at the shift point every pod takes the
//!   shift colour and strobes on the strip's own clock; the gear turns accent, steady).
//! - 2: the speed (cap 128, ghost 8s, 0.6 hysteresis).
//! - 3: the tach bars.
//! - 4: pedal columns, slip bar, power and boost slants, the G readout (5 Hz with hysteresis).
//! - 5: tach numerals, printed labels, scale figures, lens and surround.
//!
//! Signature: the **sector boot**. On the first observed `IsRaceOn` 0 → 1 after launch the
//! filaments warm for 150 ms, then the panel lights left to right in 8 sectors of 40 ms as an
//! all-segments self-test that settles to ghost (segments switch, they never fade in, as a VFD).
//! Gear and speed are live from frame 1.
//!
//! Telemetry: rpm (bars, bracket, red zone from the shift point), gear, speed, throttle, brake,
//! `boost_psi`, `power_watts` in the user's power unit, smoothed lateral G (readout, light) and
//! body slip from velocity (`Signals::slip_deg`). Forza sends no fuel, temperatures, oil pressure
//! or trip distance, so those A70 windows carry these verified fields in the same shapes.
//!
//! ## Zone table (local 4K px around the anchor)
//!
//! | Zone | Element | Cockpit | Hood |
//! |---|---|---|---|
//! | Z5 | lens (surround 12 / 10 px outside) | (−700, −304)..(700, 210), r 36 | (−866, −208)..(866, 150), r 30 |
//! | Z4 | pedal window | (−680, −150)..(−548, 160) | (−850, −120)..(−720, 106) |
//! | Z3 | centre window | (−528, −294)..(318, 160) | (−700, −200)..(420, 106) |
//! | Z3 | tach scale | x −490..290, baseline −96 → −130, bars 12 → 84 | x −660..380, baseline −60 → −84, bars 10 → 54 |
//! | Z1 | shift bracket | 22 px above the numerals | 16 px above the numerals |
//! | Z2 | speed | (−150, 40), cap 128 | (−330, 20), cap 96 |
//! | Z1 | gear | (175, 20), cap 190 | (250, 26), cap 150 |
//! | Z4 | G readout | (−420, 70), cap 44 | (−590, 40), cap 36 |
//! | Z4 | slip window | (338, −150)..(418, 160) | (440, −120)..(520, 106) |
//! | Z4 | gauge window | (436, −150)..(690, 160) | (540, −120)..(850, 106) |
//! | Z5 | logo zone | (−490, −233)..(−310, −197) | (−660, −150)..(−510, −120) |
//! | Z5 | printed labels | cap 16 at y 176 | cap 14 at y 120 |
use crate::common::bezel::{lit_rim, on_screen};
use crate::common::seg7::{
    draw_scanlines, Emission, Phosphor, Reveal, Seg7Digit, Seg7Geometry, Seg7Number, Segments, Separator, DIAGONAL,
};
use crate::common::shift::LimiterCue;
use crate::common::{ShiftLights, SpeedReadout, VirtualLight};
use fh6_kit::cluster::Setup;
use fh6_kit::color::{linear_to_srgb, srgb_to_linear};
use fh6_kit::logo::LogoStyle;
use fh6_kit::{
    vec2, Align, BlendMode, Cluster, Color, FontId, FontRole, FrameInput, Gear, Glow, Gradient, Hysteresis, ImageId,
    ImagePaint, ImageWrap, LayerOptions, Layout, Logo, OpticalGlass, Paint, Painter, Palette, Piece, PowerUnit, Rect,
    Shape, Signals, Smoothed, TextStyle, Tokens, VAlign, Vec2, View,
};

/// Tach bars per 1,000 rpm (one per 250 rpm).
const BARS_PER_K: usize = 4;
/// The most thousands a scale shows (a 12,000 rpm limiter).
const MAX_THOUSANDS: usize = 12;
/// The fewest thousands a scale shows.
const MIN_THOUSANDS: usize = 4;
/// Bar width as a share of the bar pitch.
const BAR_FILL: f32 = 0.56;
/// Shift-bracket pods.
const PODS: usize = 10;
/// Gap between bracket pods as a share of the pod pitch.
const POD_GAP: f32 = 0.22;
/// Pedal segments per column.
const PEDAL_ROWS: usize = 10;
/// Slip-bar segments.
const SLIP_STEPS: usize = 12;
/// Body slip that fills the slip bar (degrees).
const SLIP_FULL_DEG: f32 = 48.0;
/// Segments per slanted gauge.
const SLANT_STEPS: usize = 12;
/// Slant of the power and boost gauges (degrees from upright, leaning right).
const SLANT_DEG: f32 = 12.0;
/// The boost gauge's range (psi): the A70's turbo gauge also reads vacuum below 0.
const BOOST_MIN_PSI: f32 = -10.0;
/// See [`BOOST_MIN_PSI`].
const BOOST_MAX_PSI: f32 = 20.0;
/// The smallest power scale (in the user's unit), so idle never fills the gauge.
const POWER_MIN_SCALE: f32 = 200.0;
/// The power scale grows in steps of this (user's unit).
const POWER_STEP: f32 = 100.0;
/// The power scale only re-ranges with the throttle below this, never mid-pull.
const POWER_RERANGE_THROTTLE: f32 = 0.1;
/// Watts per mechanical horsepower.
const WATTS_PER_HP: f32 = 745.7;
/// Small readouts refresh at 5 Hz, like a car display.
const READOUT_PERIOD_S: f32 = 0.2;
/// Boot: filament warm-up (s).
const FILAMENT_WARM_S: f32 = 0.15;
/// Boot: ignition sectors, left to right.
const SECTORS: usize = 8;
/// Boot: each sector's self-test (s).
const SECTOR_S: f32 = 0.04;
/// Phosphor release: lit segments fall to ghost over this time (s).
const PHOSPHOR_DECAY_S: f32 = 0.05;
/// The whole boot (warm-up, 8 sectors and the last sector's decay).
const BOOT_S: f32 = FILAMENT_WARM_S + SECTORS as f32 * SECTOR_S + PHOSPHOR_DECAY_S;
/// Boot filament luminance as seen on screen.
const FILAMENT_WARM_LEVEL: f32 = 0.15;
/// Neutral shows in ink at 72%.
const NEUTRAL_LEVEL: f32 = 0.72;
/// Parallax of the mesh and filament layers at full felt G (the phosphor is layer 0).
const PARALLAX: [f32; 2] = [1.0, 3.0];
/// Felt G that reaches full parallax.
const PARALLAX_G: f32 = 1.5;
/// Unlit VFD elements show this share of their phosphor's displayed brightness.
const GHOST_LEVEL: f32 = 0.07;
/// The bracket's halo (a local glow, not bloom, so it stays inside the window).
const POD_GLOW: Glow = Glow {
    sigma: 3.5,
    intensity: 0.45,
    fill_inside: false,
};
/// The tach bars' halo.
const BAR_GLOW: Glow = Glow {
    sigma: 2.5,
    intensity: 0.3,
    fill_inside: false,
};
/// Rank-4 lit phosphor as seen on screen.
const RANK4_LEVEL: f32 = 0.62;
/// Static print (labels, numerals, figures) as seen on screen: at most 20% relative luminance.
const PRINT_LEVEL: f32 = 0.46;
/// Red-zone numerals as seen on screen (the warning role, dimmed like print).
const RED_PRINT_LEVEL: f32 = 0.72;
/// The unlit bracket rail (the printed red line under the pods) as seen on screen.
const RAIL_LEVEL: f32 = 0.18;
/// VFD tube-package rims (the glass envelope seen edge-on) as seen on screen.
const ENVELOPE_LEVEL: f32 = 0.09;
/// The ComfyUI phosphor plate's tile (local px).
const PLATE_TILE: f32 = 80.0;
/// Gain on lit phosphor under the grain plate.
const PLATE_GAIN: f32 = 1.1;
/// Hex-mesh tile size (local px).
const MESH_TILE: f32 = 144.0;
/// Mesh strength (multiply lerp from identity).
const MESH_STRENGTH: f32 = 0.42;
/// Filament darkening (multiply).
const FILAMENT_DARK: f32 = 0.55;
/// Filament heights as shares of each window's height.
const FILAMENTS: [f32; 3] = [0.2, 0.5, 0.8];
/// Surround face levels on the lit and far side, as seen on screen (satin graphite).
const SURROUND_LIT: f32 = 0.2;
/// See [`SURROUND_LIT`].
const SURROUND_DARK: f32 = 0.07;
/// The lens mirrors the studio at this share of the tokens' environment gain: the car's cabin
/// is dark, and a flat face mirroring the full studio greys the VFD windows' black (rejection
/// pattern e). The key light's highlight and the rims keep their full strength.
const LENS_ENV: f32 = 0.04;
/// Margin round the Blender surround render (local px; the recipe's `MARGIN`).
const SURROUND_MARGIN: f32 = 4.0;
/// The lens is polished: a broad satin lobe lit the whole flat face grey; a polished face keeps
/// the key light's highlight to the rims, whose normals turn towards it.
const LENS_ROUGHNESS: f32 = 0.04;
/// The lens's cross-section: thickness, rounded edge, glass below the rim (local px).
const LENS_SECTION: (f32, f32, f32) = (7.0, 10.0, 5.0);

/// Geometry of one view (local 4K px around the anchor).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
    lens: Rect,
    lens_r: f32,
    surround: f32,
    pedal: Rect,
    centre: Rect,
    slip: Rect,
    gauges: Rect,
    /// Tach x span; `y` of the baseline at the left end.
    tach_x: (f32, f32),
    tach_base: f32,
    /// How far the baseline rises from the left end to the right end.
    tach_rise: f32,
    /// Bar heights at the left and right ends.
    bar_h: (f32, f32),
    numeral_cap: f32,
    pod_h: f32,
    speed: Vec2,
    speed_cap: f32,
    gear: Vec2,
    gear_cap: f32,
    readout: Vec2,
    readout_cap: f32,
    label_y: f32,
    label_cap: f32,
    figure_cap: f32,
    /// The logo zone: printed under the tach legend at the window's top left.
    logo: Rect,
    /// Inset of the bar gauges inside their windows (top, bottom).
    bar_inset: (f32, f32),
}

const fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect {
        min: vec2(x0, y0),
        max: vec2(x1, y1),
    }
}

const COCKPIT: Geom = Geom {
    lens: rect(-700.0, -304.0, 700.0, 210.0),
    lens_r: 36.0,
    surround: 12.0,
    pedal: rect(-680.0, -150.0, -548.0, 160.0),
    centre: rect(-528.0, -294.0, 318.0, 160.0),
    slip: rect(338.0, -150.0, 418.0, 160.0),
    gauges: rect(436.0, -150.0, 690.0, 160.0),
    tach_x: (-490.0, 290.0),
    tach_base: -96.0,
    tach_rise: 34.0,
    bar_h: (12.0, 84.0),
    numeral_cap: 26.0,
    pod_h: 22.0,
    speed: vec2(-150.0, 40.0),
    speed_cap: 128.0,
    gear: vec2(175.0, 20.0),
    gear_cap: 190.0,
    readout: vec2(-420.0, 70.0),
    readout_cap: 44.0,
    label_y: 176.0,
    label_cap: 16.0,
    figure_cap: 15.0,
    logo: rect(-490.0, -233.0, -310.0, -197.0),
    bar_inset: (30.0, 10.0),
};

/// Hood: the scrim is 0%, so the panel widens and lowers, closer to the A70's own 6:1.
const HOOD: Geom = Geom {
    lens: rect(-866.0, -208.0, 866.0, 150.0),
    lens_r: 30.0,
    surround: 10.0,
    pedal: rect(-850.0, -120.0, -720.0, 106.0),
    centre: rect(-700.0, -200.0, 420.0, 106.0),
    slip: rect(440.0, -120.0, 520.0, 106.0),
    gauges: rect(540.0, -120.0, 850.0, 106.0),
    tach_x: (-660.0, 380.0),
    tach_base: -60.0,
    tach_rise: 24.0,
    bar_h: (10.0, 54.0),
    numeral_cap: 22.0,
    pod_h: 16.0,
    speed: vec2(-330.0, 20.0),
    speed_cap: 96.0,
    gear: vec2(250.0, 26.0),
    gear_cap: 150.0,
    readout: vec2(-590.0, 40.0),
    readout_cap: 36.0,
    label_y: 120.0,
    label_cap: 14.0,
    figure_cap: 13.0,
    logo: rect(-660.0, -150.0, -510.0, -120.0),
    bar_inset: (24.0, 8.0),
};

fn geom(view: View) -> &'static Geom {
    match view {
        View::Hood => &HOOD,
        View::Cockpit | View::Chase => &COCKPIT,
    }
}

/// The tach scale for a car: thousands shown and where the red numerals start.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Scale {
    /// The top numeral (thousands).
    top: usize,
    /// The first red numeral (thousands).
    red_from: usize,
    /// Where the bracket starts and ends (0..1 along the scale).
    bracket: (f32, f32),
}

impl Scale {
    /// The scale for a limiter of `max_rpm`, a shift point of `shift_rpm` and a first shift
    /// stage at `stage0` of the shift point: the first whole thousand at least 250 rpm above the
    /// limiter, red from the thousand nearest the shift point.
    fn new(max_rpm: f32, shift_rpm: f32, stage0: f32) -> Self {
        let max_rpm = if max_rpm.is_finite() { max_rpm.max(1.0) } else { 8000.0 };
        let shift_rpm = if shift_rpm.is_finite() {
            shift_rpm.clamp(1.0, max_rpm)
        } else {
            max_rpm
        };
        let top = (((max_rpm + 250.0) / 1000.0).ceil() as usize).clamp(MIN_THOUSANDS, MAX_THOUSANDS);
        let span = top as f32 * 1000.0;
        let red_from = ((shift_rpm / 1000.0).round() as usize).clamp(1, top);
        let stage0 = if stage0.is_finite() {
            stage0.clamp(0.3, 0.99)
        } else {
            0.75
        };
        let a = (stage0 * shift_rpm / span).clamp(0.0, 1.0);
        let b = (shift_rpm / span).clamp(a, 1.0);
        Self {
            top,
            red_from,
            bracket: (a, b),
        }
    }

    fn bars(&self) -> usize {
        self.top * BARS_PER_K
    }

    /// Bars lit at `rpm`.
    fn lit(&self, rpm: f32) -> usize {
        let rpm = if rpm.is_finite() { rpm.max(0.0) } else { 0.0 };
        ((rpm / (1000.0 / BARS_PER_K as f32)).round() as usize).min(self.bars())
    }
}

impl Geom {
    fn x_at(&self, u: f32) -> f32 {
        self.tach_x.0 + (self.tach_x.1 - self.tach_x.0) * u
    }

    /// The baseline at `u` (0 left .. 1 right).
    fn base_at(&self, u: f32) -> f32 {
        self.tach_base - self.tach_rise * u * u
    }

    /// The bar tops at `u`.
    fn top_at(&self, u: f32) -> f32 {
        self.base_at(u) - (self.bar_h.0 + (self.bar_h.1 - self.bar_h.0) * u.max(0.0).powf(1.5))
    }

    /// The numerals' centre line at `u`.
    fn numeral_at(&self, u: f32) -> f32 {
        self.top_at(u) - self.numeral_cap * 0.4 - self.numeral_cap * 0.5
    }

    /// The bracket's lower edge at `u`.
    fn pod_base_at(&self, u: f32) -> f32 {
        self.numeral_at(u) - self.numeral_cap * 0.5 - self.numeral_cap * 0.35
    }

    /// Bar `i` of `n`: its top edge follows the curve.
    fn bar(&self, i: usize, n: usize) -> [Vec2; 4] {
        let n = n.max(1) as f32;
        let pitch = (self.tach_x.1 - self.tach_x.0) / n;
        let w = pitch * BAR_FILL;
        // Bar i covers (i, i + 1] quarter-thousands; it stands at the top of its step.
        let u = (i as f32 + 1.0) / n;
        let x1 = self.x_at(u);
        let x0 = x1 - w;
        let (u0, u1) = ((x0 - self.tach_x.0) / (pitch * n), u);
        [
            vec2(x0, self.base_at(u0)),
            vec2(x1, self.base_at(u1)),
            vec2(x1, self.top_at(u1)),
            vec2(x0, self.top_at(u0)),
        ]
    }

    /// Bracket pod `i` of [`PODS`] between `from` and `to` along the scale.
    fn pod(&self, i: usize, from: f32, to: f32) -> [Vec2; 4] {
        let pitch = (to - from) / PODS as f32;
        let u0 = from + pitch * (i as f32 + POD_GAP * 0.5);
        let u1 = from + pitch * (i as f32 + 1.0 - POD_GAP * 0.5);
        let (b0, b1) = (self.pod_base_at(u0), self.pod_base_at(u1));
        [
            vec2(self.x_at(u0), b0),
            vec2(self.x_at(u1), b1),
            vec2(self.x_at(u1), b1 - self.pod_h),
            vec2(self.x_at(u0), b0 - self.pod_h),
        ]
    }

    /// The bar area of a gauge window (between the figure row and the bottom inset).
    fn bar_area(&self, window: Rect) -> Rect {
        rect(
            window.min.x,
            window.min.y + self.bar_inset.0,
            window.max.x,
            window.max.y - self.bar_inset.1,
        )
    }

    /// Pedal segment `row` (0 at the bottom) of column `col` (0 throttle, 1 brake).
    fn pedal_cell(&self, col: usize, row: usize) -> Rect {
        let a = self.bar_area(self.pedal);
        let margin = a.width() * 0.1;
        let gap = a.width() * 0.1;
        let w = (a.width() - margin * 2.0 - gap) * 0.5;
        let x = a.min.x + margin + col as f32 * (w + gap);
        let pitch = a.height() / PEDAL_ROWS as f32;
        let y = a.max.y - pitch * (row as f32 + 1.0);
        rect(x, y + pitch * 0.18, x + w, y + pitch * 0.82)
    }

    /// Slip segment `j` (0 at the bottom); the top one is cut on a slant like the A70's
    /// temperature bar.
    fn slip_cell(&self, j: usize) -> [Vec2; 4] {
        let a = self.bar_area(self.slip);
        let w = a.width() * 0.56;
        let x0 = a.center().x - w * 0.5;
        let pitch = a.height() / SLIP_STEPS as f32;
        let y1 = a.max.y - pitch * j as f32 - pitch * 0.15;
        let y0 = a.max.y - pitch * (j as f32 + 1.0) + pitch * 0.15;
        if j + 1 == SLIP_STEPS {
            [
                vec2(x0, y1),
                vec2(x0 + w, y1),
                vec2(x0 + w, y0 + pitch * 0.35),
                vec2(x0, y0),
            ]
        } else {
            [vec2(x0, y1), vec2(x0 + w, y1), vec2(x0 + w, y0), vec2(x0, y0)]
        }
    }

    /// The two slanted gauges' footprints (power left, boost right): bottom-left x and width.
    fn slant_base(&self, k: usize) -> (f32, f32) {
        let a = self.bar_area(self.gauges);
        let lean = a.height() * SLANT_DEG.to_radians().tan();
        let w = (a.width() - lean) * 0.2;
        let gap = (a.width() - lean - w * 2.0) / 3.0;
        (a.min.x + gap + k as f32 * (w + gap), w)
    }

    /// Segment `j` (0 at the bottom) of slanted gauge `k`: a parallelogram leaning right.
    fn slant_cell(&self, k: usize, j: usize) -> [Vec2; 4] {
        let a = self.bar_area(self.gauges);
        let tan = SLANT_DEG.to_radians().tan();
        let (x0, w) = self.slant_base(k);
        let pitch = a.height() / SLANT_STEPS as f32;
        let y1 = a.max.y - pitch * j as f32 - pitch * 0.15;
        let y0 = a.max.y - pitch * (j as f32 + 1.0) + pitch * 0.15;
        let s1 = (a.max.y - y1) * tan;
        let s0 = (a.max.y - y0) * tan;
        [
            vec2(x0 + s1, y1),
            vec2(x0 + w + s1, y1),
            vec2(x0 + w + s0, y0),
            vec2(x0 + s0, y0),
        ]
    }

    fn windows(&self) -> [Rect; 4] {
        [self.pedal, self.centre, self.slip, self.gauges]
    }

    /// Boot sector `k` (left to right) of the lens.
    fn sector(&self, k: usize) -> Rect {
        let b = self.lens;
        let w = b.width() / SECTORS as f32;
        rect(b.min.x + w * k as f32, b.min.y, b.min.x + w * (k + 1) as f32, b.max.y)
    }
}

/// The opaque 7% ghost of a phosphor (sRGB level, so it reads the same on any black).
fn ghost(phosphor: Color) -> Color {
    let (r, g, b) = phosphor.straight_rgb();
    let dim = |c: f32| srgb_to_linear(GHOST_LEVEL * linear_to_srgb(c.clamp(0.0, 1.0)));
    Color::linear(dim(r), dim(g), dim(b), 1.0)
}

/// `c` dimmed to `level` of its displayed brightness.
fn dimmed(c: Color, level: f32) -> Color {
    c.intensity(srgb_to_linear(level))
}

/// The phosphor paint for colour `c`: the ComfyUI grain plate tinted by `c` when present, else
/// the flat colour.
fn phosphor_paint(plate: Option<ImageId>, c: Color) -> Paint {
    match plate {
        Some(img) => {
            let tile = Rect::from_xywh(0.0, 0.0, PLATE_TILE, PLATE_TILE);
            ImagePaint::new(img, tile).repeat().tint(c).into()
        }
        None => c.into(),
    }
}

/// The boot clock: started by the first observed `IsRaceOn` 0 → 1 after launch (a cluster made
/// while racing never saw the 0, so it does not boot).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Boot {
    prev: Option<bool>,
    done: bool,
    t: Option<f32>,
}

impl Boot {
    /// Advances with this frame's `IsRaceOn`.
    fn observe(&mut self, race_on: bool, dt: f32) {
        if let Some(t) = self.t.as_mut() {
            *t += dt;
            if *t > BOOT_S || !race_on {
                self.t = None;
            }
        }
        if self.prev == Some(false) && race_on && !self.done {
            self.done = true;
            self.t = Some(0.0);
        }
        self.prev = Some(race_on);
    }

    fn running(&self) -> bool {
        self.t.is_some()
    }

    fn t(&self) -> Option<f32> {
        self.t
    }
}

/// When sector `k` ignites (s into the boot).
fn sector_start(k: usize) -> f32 {
    FILAMENT_WARM_S + k as f32 * SECTOR_S
}

/// The self-test level of sector `k` at boot time `t`: full for its 40 ms, then the phosphor
/// decay to ghost.
fn sector_level(t: f32, k: usize) -> f32 {
    let s = sector_start(k);
    if t < s {
        0.0
    } else if t < s + SECTOR_S {
        1.0
    } else {
        (1.0 - (t - s - SECTOR_S) / PHOSPHOR_DECAY_S).clamp(0.0, 1.0)
    }
}

/// Felt G: accelerating right pushes you left, accelerating forward pushes you back (down).
fn felt_g(sig: &Signals) -> Vec2 {
    let v = vec2(-sig.g_lat, sig.g_long);
    if v.x.is_finite() && v.y.is_finite() {
        v
    } else {
        Vec2::ZERO
    }
}

/// Engine power in the user's unit.
fn power_in(watts: f32, unit: PowerUnit) -> f32 {
    let w = if watts.is_finite() { watts.max(0.0) } else { 0.0 };
    match unit {
        PowerUnit::Hp => w / WATTS_PER_HP,
        PowerUnit::Kw => w / 1000.0,
    }
}

/// Segments lit for `value` on a gauge of `steps` from `lo` to `hi`.
fn steps_lit(value: f32, lo: f32, hi: f32, steps: usize) -> usize {
    let v = if value.is_finite() { value } else { lo };
    let k = ((v - lo) / (hi - lo).max(1e-3)).clamp(0.0, 1.0);
    (k * steps as f32).round() as usize
}

/// A readout that takes a new value at 5 Hz with hysteresis.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SlowReadout {
    shown: Hysteresis,
    value: i64,
    timer: f32,
}

impl SlowReadout {
    fn new(threshold: f32) -> Self {
        Self {
            shown: Hysteresis::new(0.0, threshold),
            value: 0,
            timer: 0.0,
        }
    }

    /// Feeds `v`; the first frame snaps (the cluster is rebuilt on a view change).
    fn update(&mut self, v: f32, dt: f32, first: bool) {
        self.timer -= dt;
        if first || self.timer <= 0.0 {
            self.value = self.shown.update(v);
            self.timer = READOUT_PERIOD_S;
        }
    }
}

/// The faces this cluster sets (resolved once in `new`; a user display font replaces them).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    label: FontId,
    wide: FontId,
    role_display: FontId,
}

/// The maker whose wordmark (`assets/logos/toyota_dial.png`) the panel carries when the car is
/// one of theirs and the host sends no logo image (compared in `update`, never in `draw`).
const MAKER: &str = "Toyota";

/// The Retro Cyber cluster.
#[derive(Debug, Clone)]
pub struct RetroCyber {
    faces: Faces,
    sig: Signals,
    palette: Palette,
    view: View,
    first: bool,
    light: VirtualLight,
    glass: OpticalGlass,
    parallax: Smoothed<Vec2>,
    shift: ShiftLights,
    cue: LimiterCue,
    scale: Scale,
    bars_lit: usize,
    gear: Seg7Digit,
    speed_value: SpeedReadout,
    speed: Seg7Number<3>,
    g_value: SlowReadout,
    g_readout: Seg7Number<2>,
    test_speed: Seg7Number<3>,
    test_gear: Seg7Digit,
    test_readout: Seg7Number<2>,
    pedals: [usize; 2],
    slip: usize,
    power: usize,
    power_scale: f32,
    boost: usize,
    boot: Boot,
    mesh: Option<ImageId>,
    phosphor: Option<ImageId>,
    /// The Blender surround per view (cockpit, hood), two key lights each.
    surround: [Option<[ImageId; 2]>; 2],
    /// The maker's wordmark, and whether this car is the maker's.
    maker: Option<ImageId>,
    maker_car: bool,
    /// The host's logo (copied only when it changes).
    logo: Logo,
}

impl RetroCyber {
    /// Builds the cluster: faces, the seven-segment cells, the lens glass and the optional
    /// phosphor grain and hex mesh.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let t: &Tokens = setup.tokens;
        let palette = Palette::family(t, "sciFi");
        let text = setup.fonts.role(FontRole::Text);
        let faces = Faces {
            label: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            wide: setup.fonts.role(FontRole::Wide),
            role_display: setup.fonts.role(FontRole::Display),
        };
        let mut load = |rel: &str| -> Option<ImageId> {
            let path = setup.assets?.join(rel);
            path.is_file()
                .then(|| setup.images.load_file(setup.device, setup.queue, &path, None).ok())
                .flatten()
        };
        let mesh = load("scifi/vfd_hex_mesh.png");
        let phosphor = load("clusters/retro_cyber/phosphor_grain.png");
        let mut pair = |view: &str| -> Option<[ImageId; 2]> {
            let a = load(&format!("clusters/retro_cyber/surround_{view}_l160.png"))?;
            let b = load(&format!("clusters/retro_cyber/surround_{view}_l110.png"))?;
            Some([a, b])
        };
        let surround = [pair("cockpit"), pair("hood")];
        let maker = load("logos/toyota_dial.png");
        let gear = Seg7Digit::new(t, Reveal::Instant)
            .with_segments(Segments::Diagonal)
            .with_phosphor(Phosphor::Ink)
            .with_ghost(Phosphor::Primary);
        let mut test_gear = gear;
        test_gear.update(0x7F | DIAGONAL, 0.0);
        let mut test_speed = Seg7Number::<3>::new(t, Reveal::Instant);
        test_speed.update(888, 0.0);
        let g_readout = Seg7Number::<2>::new(t, Reveal::Instant)
            .with_separator(Separator::Decimal { places: 1 })
            .with_phosphor(Phosphor::Secondary)
            .with_ghost(Phosphor::Secondary);
        let mut test_readout = g_readout;
        test_readout.update(88, 0.0);
        let scale = Scale::new(8000.0, 7760.0, palette.stages[0]);
        Self {
            faces,
            sig: Signals::default(),
            palette,
            view: View::Cockpit,
            first: true,
            light: VirtualLight::new(t),
            glass: t.optical_glass(&palette),
            parallax: Smoothed::new(Vec2::ZERO, 0.08),
            shift: ShiftLights::new(PODS as u32),
            cue: LimiterCue::new(),
            scale,
            bars_lit: 0,
            gear,
            speed_value: SpeedReadout::new(),
            speed: Seg7Number::new(t, Reveal::Instant),
            g_value: SlowReadout::new(1.0),
            g_readout,
            test_speed,
            test_gear,
            test_readout,
            pedals: [0; 2],
            slip: 0,
            power: 0,
            power_scale: POWER_MIN_SCALE,
            boost: 0,
            boot: Boot::default(),
            mesh,
            phosphor,
            surround,
            maker,
            maker_car: false,
            logo: Logo::default(),
        }
    }

    /// The parallax offset of VFD layer `layer` (0 mesh, 1 filaments).
    fn offset(&self, layer: usize) -> Vec2 {
        self.parallax.value() * PARALLAX[layer]
    }

    /// The bracket pod colours this frame: `None` when unlit or strobed off. Pods fill left to
    /// right over the bars that light them; at the shift point every pod takes the shift colour
    /// and follows the strip's strobe.
    fn pod_color(&self, i: usize, pal: &Palette) -> Option<Color> {
        if self.shift.over_shift() {
            // `ShiftLights` owns the strobe clock: its outermost pod is lit exactly in the on
            // phase (and always, as a breath, in reduced-flash mode).
            return self.shift.color_of(0, pal);
        }
        let level = self.shift.level();
        let rank = (i as f32 + 1.0) / PODS as f32;
        (level > 0.0 && level >= rank - 1e-4).then(|| {
            let start = pal.stages[0];
            pal.rev_stage(start + rank * (1.0 - start))
        })
    }
}

impl Cluster for RetroCyber {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = if input.dt.is_finite() { input.dt.max(0.0) } else { 0.0 };
        let sig = *input.signals;
        let first = self.first;
        self.first = false;
        self.sig = sig;
        self.palette = *input.palette;
        self.view = input.view;
        if input.fonts.display != self.faces.role_display {
            self.faces.label = input.fonts.display;
            self.faces.role_display = input.fonts.display;
        }
        self.faces.wide = input.fonts.wide;
        if self.logo != *input.logo {
            self.logo.clone_from(input.logo);
            self.maker_car = self.logo.brand_name.trim().eq_ignore_ascii_case(MAKER);
        }
        self.boot.observe(input.telemetry.is_race_on != 0, dt);

        // Light, lens and parallax follow felt G.
        self.light.update(sig.g_lat, dt);
        let op = &input.tokens.optics;
        let mut glass = input.tokens.optical_glass(&self.palette);
        glass.aim(self.light.angle(), op.key_elevation, Vec2::ZERO);
        glass.env_gain = glass.env_gain.map(|e| e * LENS_ENV);
        glass.roughness = glass.roughness.min(LENS_ROUGHNESS);
        self.glass = glass;
        let felt = felt_g(&sig) * (1.0 / PARALLAX_G);
        let len = felt.length();
        let dir = if len > 1.0 { felt * (1.0 / len) } else { felt };
        if first {
            self.parallax = Smoothed::new(dir, 0.08);
        } else {
            self.parallax.update(dir, dt);
        }

        // Tach scale, bars, bracket and the limiter cue.
        self.scale = Scale::new(sig.max_rpm, sig.shift_rpm, self.palette.stages[0]);
        self.bars_lit = self.scale.lit(sig.rpm);
        self.shift.update(&sig, &self.palette, input.t, dt, input.reduced_flash);
        self.cue.update(self.shift.over_shift(), 0.12, dt);

        // Gear: white phosphor, accent while the limiter cue holds, warning in reverse, neutral
        // at 72%. Segments switch on the frame the gear changes (a VFD has no morph).
        self.gear.set_phosphor(match sig.gear {
            Gear::Reverse => Phosphor::Warning,
            Gear::Neutral | Gear::Forward(_) if self.cue.level() > 0.5 => Phosphor::Accent,
            Gear::Neutral | Gear::Forward(_) => Phosphor::Ink,
        });
        self.gear.set_brightness(match sig.gear {
            Gear::Neutral => NEUTRAL_LEVEL,
            Gear::Reverse | Gear::Forward(_) => 1.0,
        });
        self.gear.update_gear(sig.gear, dt);

        // Speed (0.6 hysteresis), instant segments.
        self.speed_value.update(sig.speed);
        self.speed.update(self.speed_value.value(), dt);

        // Lateral G in tenths, 5 Hz with a one-tenth hysteresis.
        let g = if sig.g_lat.is_finite() { sig.g_lat.abs() } else { 0.0 };
        self.g_value.update((g * 10.0).min(99.0), dt, first);
        self.g_readout.update(self.g_value.value, dt);

        // Bar gauges switch in whole segments, unsmoothed (a VFD has no needle).
        self.pedals = [
            steps_lit(sig.throttle, 0.0, 1.0, PEDAL_ROWS),
            steps_lit(sig.brake, 0.0, 1.0, PEDAL_ROWS),
        ];
        self.slip = steps_lit(sig.slip_deg.abs(), 0.0, SLIP_FULL_DEG, SLIP_STEPS);
        let power = power_in(input.telemetry.power_watts, sig.power_unit);
        let wanted = ((power / POWER_STEP).ceil() * POWER_STEP).max(POWER_MIN_SCALE);
        if first || (wanted > self.power_scale && sig.throttle < POWER_RERANGE_THROTTLE) {
            self.power_scale = wanted;
        }
        self.power = steps_lit(power, 0.0, self.power_scale, SLANT_STEPS);
        self.boost = steps_lit(input.telemetry.boost_psi, BOOST_MIN_PSI, BOOST_MAX_PSI, SLANT_STEPS);
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = geom(self.view);
        let pal = self.palette;
        let boot_t = self.boot.t();

        // 1. The surround and the opaque black panel behind the lens.
        self.draw_surround(p, g, &pal);

        // 2. Phosphor: everything except the live gear and speed.
        self.draw_windows(p, g, &pal);
        self.draw_tach(p, g, &pal, false);
        self.draw_bracket(p, g, &pal, false);
        self.draw_pedals(p, g, &pal, false);
        self.draw_slip(p, g, &pal, false);
        self.draw_slants(p, g, &pal, false);
        self.g_readout
            .draw(p, g.readout, &Seg7Geometry::new(g.readout_cap), &pal, Emission::Matte);
        self.draw_print(p, g, &pal);

        // 3. Boot: sectors not yet ignited are cold glass (black).
        if let Some(t) = boot_t {
            for k in 0..SECTORS {
                if t < sector_start(k) {
                    p.fill(&Shape::rect(g.sector(k)), pal.roles.oled_black);
                }
            }
        }

        // 4. Live from frame 1: the gear (rank 1) and the speed (rank 2).
        let emission = if self.cue.level() > 0.5 {
            Emission::Soft
        } else {
            Emission::Matte
        };
        self.gear
            .draw(p, g.gear, &Seg7Geometry::new(g.gear_cap), &pal, emission);
        self.speed
            .draw(p, g.speed, &Seg7Geometry::new(g.speed_cap), &pal, Emission::Bloom);

        // 5. Boot self-test: every segment of the igniting sectors, then the phosphor decay.
        if let Some(t) = boot_t {
            for k in 0..SECTORS {
                let level = sector_level(t, k);
                if level > 0.0 {
                    p.scoped(|p| {
                        p.clip(g.sector(k), 0.0);
                        p.set_opacity(level);
                        self.draw_test(p, g, &pal);
                    });
                }
            }
        }

        // 6. Control-grid mesh and filaments, multiplied (they only modulate lit phosphor), then
        //    the scanlines.
        let windows = g.windows();
        p.scoped(|p| {
            p.set_blend(BlendMode::Multiply);
            p.scoped(|p| {
                p.translate(self.offset(0));
                for w in windows {
                    self.draw_mesh(p, w, g.lens.min);
                }
            });
            p.translate(self.offset(1));
            let px = p.device_px();
            for w in windows {
                for f in FILAMENTS {
                    let y = w.min.y + w.height() * f;
                    p.fill(
                        &Shape::rect(rect(w.min.x + 4.0, y - px * 0.5, w.max.x - 4.0, y + px * 0.5)),
                        pal.roles.oled_black.with_alpha(FILAMENT_DARK),
                    );
                }
            }
        });
        for w in windows {
            draw_scanlines(p, w, &pal);
        }
        if let Some(t) = boot_t {
            if t < FILAMENT_WARM_S {
                p.scoped(|p| {
                    p.set_blend(BlendMode::Add);
                    p.translate(self.offset(1));
                    let px = p.device_px();
                    let warm = pal.roles.ink.with_alpha(srgb_to_linear(FILAMENT_WARM_LEVEL));
                    for w in windows {
                        for f in FILAMENTS {
                            let y = w.min.y + w.height() * f;
                            p.fill(
                                &Shape::rect(rect(w.min.x + 4.0, y - px * 0.5, w.max.x - 4.0, y + px * 0.5)),
                                warm,
                            );
                        }
                    }
                });
            }
        }

        // 7. The lens: one piece of optical glass over the whole panel.
        let (thick, bevel, base) = LENS_SECTION;
        let lens = [Piece::round_rect(g.lens, g.lens_r).section(thick, bevel, base)];
        let glass = self.glass;
        p.layer(LayerOptions::glass(0.0), |p| {
            p.fill(
                &Shape::Blob {
                    pieces: &lens,
                    smooth: 0.0,
                },
                glass,
            );
        });
    }

    fn animating(&self) -> bool {
        self.boot.running()
            || self.shift.animating()
            || self.cue.animating()
            || self.gear.animating()
            || self.speed.animating()
            || self.g_readout.animating()
    }
}

impl RetroCyber {
    /// The slim satin-graphite surround: an opaque black body under the lens, then the Blender
    /// render (two key lights blended by lateral G), or without it the face shaded along the
    /// light, a light-driven 1 px outer rim and a polished inner chamfer at the lens.
    fn draw_surround(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let ink = roles.ink;
        let outer = g.lens.expand(g.surround);
        let r_out = g.lens_r + g.surround;
        p.fill(&Shape::round_rect(outer, r_out), roles.oled_black);
        let c = g.lens.center();
        let light = self.light.angle();
        let art = match self.view {
            View::Hood => self.surround[1],
            View::Cockpit | View::Chase => self.surround[0],
        };
        if let Some(lights) = art {
            // The two key lights (−160° → −110°) blended by the virtual light's azimuth; the
            // render's centre is transparent, so the lens body stays black.
            let image_rect = outer.expand(SURROUND_MARGIN);
            let lo = -160f32.to_radians();
            let k = ((light - lo) / 50f32.to_radians()).clamp(0.0, 1.0);
            for (img, share, blend) in [(lights[0], 1.0 - k, BlendMode::Normal), (lights[1], k, BlendMode::Add)] {
                if share > 0.001 {
                    p.scoped(|p| {
                        p.set_blend(blend);
                        let paint = ImagePaint::new(img, image_rect).tint(Color::WHITE.with_alpha(share));
                        p.fill(&Shape::round_rect(outer.expand(1.0), r_out + 1.0), paint);
                    });
                }
            }
            p.fill(&Shape::round_rect(g.lens, g.lens_r), roles.oled_black);
            return;
        }
        let l = Vec2::from_angle(light);
        let ext = outer.size().length() * 0.5;
        let face = Gradient::linear(c + l * ext, c - l * ext)
            .stop(0.0, ink.with_alpha(on_screen(SURROUND_LIT)))
            .stop(1.0, ink.with_alpha(on_screen(SURROUND_DARK)));
        // The face is the ring between the outer outline and the lens: the lens body is drawn
        // black over it.
        p.fill(&Shape::round_rect(outer, r_out), face);
        p.fill(&Shape::round_rect(g.lens, g.lens_r), roles.oled_black);
        let px = p.device_px();
        let rim_out = lit_rim(
            c,
            light,
            roles.rim_specular.with_alpha(0.5),
            roles.hairline.with_alpha(0.05),
        );
        p.stroke(
            &Shape::round_rect(outer.inset(px * 0.5), r_out - px * 0.5),
            rim_out,
            fh6_kit::Stroke::new(px),
        );
        let rim_in = lit_rim(
            c,
            light + core::f32::consts::PI,
            roles.rim_specular.with_alpha(0.4),
            roles.hairline.with_alpha(0.04),
        );
        p.stroke(
            &Shape::round_rect(g.lens.expand(px * 1.5), g.lens_r + px * 1.5),
            rim_in,
            fh6_kit::Stroke::new(px * 2.0),
        );
    }

    /// The four VFD tube packages: black, with the glass envelope's edge seen as a faint rim.
    fn draw_windows(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let rim = dimmed(pal.roles.ink, ENVELOPE_LEVEL);
        let px = p.device_px();
        for w in g.windows() {
            p.stroke(&Shape::round_rect(w, 10.0), rim, fh6_kit::Stroke::new(px));
        }
    }

    /// The horizontal tach: ghost bars, then halos, then lit bodies (one draw call for the plate
    /// fills).
    fn draw_tach(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        let n = self.scale.bars();
        let lit = if all { n } else { self.bars_lit };
        let c = pal.primary;
        let off = ghost(c);
        for i in lit..n {
            let pts = g.bar(i, n);
            p.fill(&Shape::Polygon { points: &pts }, off);
        }
        for i in 0..lit {
            let pts = g.bar(i, n);
            p.glow(&Shape::Polygon { points: &pts }, c, BAR_GLOW);
        }
        for i in 0..lit {
            let pts = g.bar(i, n);
            p.fill(
                &Shape::Polygon { points: &pts },
                phosphor_paint(self.phosphor, c.intensity(PLATE_GAIN)),
            );
        }
    }

    /// The shift bracket: the printed red rail with its turned-down ends, ghost pods, then the
    /// lit pods with their halos.
    fn draw_bracket(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        let (a, b) = self.scale.bracket;
        let rail = dimmed(pal.roles.warning, RAIL_LEVEL);
        let px = p.device_px();
        let (xa, xb) = (g.x_at(a), g.x_at(b));
        let (ya, yb) = (g.pod_base_at(a) - g.pod_h - 5.0, g.pod_base_at(b) - g.pod_h - 5.0);
        p.fill(&Shape::line(vec2(xa, ya), vec2(xb, yb), px * 2.0), rail);
        for (x, y) in [(xa, ya), (xb, yb)] {
            p.fill(&Shape::line(vec2(x, y), vec2(x, y + g.pod_h * 0.6), px * 2.0), rail);
        }
        let stage = |i: usize| {
            let start = pal.stages[0];
            pal.rev_stage(start + (i as f32 + 1.0) / PODS as f32 * (1.0 - start))
        };
        let colour = |i: usize| if all { Some(stage(i)) } else { self.pod_color(i, pal) };
        for i in 0..PODS {
            if colour(i).is_none() {
                let pts = g.pod(i, a, b);
                p.fill(&Shape::Polygon { points: &pts }, ghost(stage(i)));
            }
        }
        for i in 0..PODS {
            if let Some(c) = colour(i) {
                let pts = g.pod(i, a, b);
                p.glow(&Shape::Polygon { points: &pts }, c, POD_GLOW);
            }
        }
        for i in 0..PODS {
            if let Some(c) = colour(i) {
                let pts = g.pod(i, a, b);
                p.fill(
                    &Shape::Polygon { points: &pts },
                    phosphor_paint(self.phosphor, c.intensity(PLATE_GAIN)),
                );
            }
        }
    }

    /// Throttle and brake columns (rank 4: matte, no glow).
    fn draw_pedals(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        let colors = [pal.roles.go, pal.roles.warning];
        for pass_lit in [false, true] {
            for (col, color) in colors.into_iter().enumerate() {
                let lit = if all { PEDAL_ROWS } else { self.pedals[col] };
                for row in 0..PEDAL_ROWS {
                    if (row < lit) == pass_lit {
                        let paint = if pass_lit {
                            phosphor_paint(self.phosphor, dimmed(color, RANK4_LEVEL))
                        } else {
                            ghost(color).into()
                        };
                        p.fill(&Shape::rect(g.pedal_cell(col, row)), paint);
                    }
                }
            }
        }
    }

    /// The slip bar (rank 4).
    fn draw_slip(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        let color = pal.secondary;
        let lit = if all { SLIP_STEPS } else { self.slip };
        for pass_lit in [false, true] {
            for j in 0..SLIP_STEPS {
                if (j < lit) == pass_lit {
                    let pts = g.slip_cell(j);
                    let paint = if pass_lit {
                        phosphor_paint(self.phosphor, dimmed(color, RANK4_LEVEL))
                    } else {
                        ghost(color).into()
                    };
                    p.fill(&Shape::Polygon { points: &pts }, paint);
                }
            }
        }
    }

    /// Power and boost slants (rank 4).
    fn draw_slants(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        let color = pal.secondary;
        for pass_lit in [false, true] {
            for (k, &v) in [self.power, self.boost].iter().enumerate() {
                let lit = if all { SLANT_STEPS } else { v };
                for j in 0..SLANT_STEPS {
                    if (j < lit) == pass_lit {
                        let pts = g.slant_cell(k, j);
                        let paint = if pass_lit {
                            phosphor_paint(self.phosphor, dimmed(color, RANK4_LEVEL))
                        } else {
                            ghost(color).into()
                        };
                        p.fill(&Shape::Polygon { points: &pts }, paint);
                    }
                }
            }
        }
    }

    /// Rank-5 print: tach numerals on the curve (red in the red zone), the tach legend, scale
    /// figures, the speed unit and the labels printed along the lens's lower band.
    fn draw_print(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let ink = dimmed(pal.roles.ink_label, PRINT_LEVEL);
        let red = dimmed(pal.roles.warning, RED_PRINT_LEVEL);
        let numeral = TextStyle::new(self.faces.label, g.numeral_cap)
            .paint(ink)
            .align(Align::Center, VAlign::CapMiddle);
        let top = self.scale.top;
        for k in 0..=top {
            let u = k as f32 / top as f32;
            let style = if k >= self.scale.red_from && k > 0 {
                numeral.paint(red)
            } else {
                numeral
            };
            let cells = if k >= 10 { 2 } else { 1 };
            p.number(vec2(g.x_at(u), g.numeral_at(u)), k as i64, cells, &style, None);
        }
        let small = TextStyle::new(self.faces.label, g.figure_cap)
            .paint(ink)
            .tracking(0.08)
            .align(Align::Left, VAlign::CapTop);
        let legend_y = g.centre.min.y + g.figure_cap * 1.4;
        p.text(vec2(g.tach_x.0, legend_y), "TACH \u{d7}1000r/min", &small);

        self.draw_logo(p, g, ink);

        // The speed unit, right of the digits at their foot.
        let sg = Seg7Geometry::new(g.speed_cap);
        let unit = TextStyle::new(self.faces.label, g.figure_cap * 1.3)
            .paint(ink)
            .tracking(0.06)
            .align(Align::Left, VAlign::Baseline);
        let speed_right = g.speed.x + sg.advance() * 1.5 + 6.0;
        p.text(
            vec2(speed_right, g.speed.y + g.speed_cap * 0.5),
            self.sig.unit.label(),
            &unit,
        );
        let rg = Seg7Geometry::new(g.readout_cap);
        p.text(
            vec2(g.readout.x + rg.advance() + 6.0, g.readout.y + g.readout_cap * 0.5),
            "G",
            &unit,
        );

        // Scale figures over the bar gauges.
        let figure = TextStyle::new(self.faces.label, g.figure_cap)
            .paint(ink)
            .align(Align::Center, VAlign::CapTop);
        let fy = g.slip.min.y + g.figure_cap * 0.5;
        p.text(vec2(g.slip.center().x, fy), "45\u{b0}", &figure);
        let a = g.bar_area(g.gauges);
        let lean = a.height() * SLANT_DEG.to_radians().tan();
        let power_unit = self.sig.power_unit.label();
        let figure_r = TextStyle {
            align: Align::Right,
            ..figure
        };
        let unit_l = TextStyle {
            align: Align::Left,
            ..figure
        };
        for (k, value, unit) in [
            (0usize, self.power_scale as i64, power_unit),
            (1, BOOST_MAX_PSI as i64, "psi"),
        ] {
            let (x0, w) = g.slant_base(k);
            let x = x0 + lean + w * 0.5;
            p.number(vec2(x + 2.0, fy), value, 3, &figure_r, None);
            p.text(vec2(x + 6.0, fy), unit, &unit_l);
        }

        // Labels along the lens's lower band.
        let label = TextStyle::new(self.faces.label, g.label_cap)
            .paint(ink)
            .tracking(0.08)
            .align(Align::Center, VAlign::CapTop);
        let y = g.label_y;
        let tight = label.tracking(0.04);
        for col in 0..2 {
            let x = g.pedal_cell(col, 0).center().x;
            p.text(vec2(x, y), ["THR", "BRK"][col], &tight);
        }
        p.text(vec2(g.readout.x, y), "G-LOAD", &label);
        p.text(vec2(g.speed.x, y), "SPEED", &label);
        p.text(vec2(g.gear.x, y), "GEAR", &label);
        p.text(vec2(g.slip.center().x, y), "SLIP", &label);
        // The gauge window's two names sit at its ends, under their bars' feet, so they never
        // run into each other.
        let left = TextStyle {
            align: Align::Left,
            ..label
        };
        let right = TextStyle {
            align: Align::Right,
            ..label
        };
        p.text(vec2(g.gauges.min.x + 6.0, y), "POWER", &left);
        p.text(vec2(g.gauges.max.x - 6.0, y), "BOOST", &right);
    }

    /// The logo zone, printed like the labels: the host's logo image, else the maker's wordmark
    /// on the maker's own cars, else the brand or custom name; nothing when the logo is off.
    fn draw_logo(&self, p: &mut Painter<'_>, g: &Geom, ink: Color) {
        let off = self.logo.image.is_none()
            && self.logo.brand_name.trim().is_empty()
            && self.logo.custom_name.trim().is_empty();
        if off {
            return;
        }
        let maker = if self.maker_car { self.maker } else { None };
        let style = LogoStyle {
            tint: ink,
            text: ink.into(),
            cap: g.logo.height() * 0.5,
            ..LogoStyle::new(self.faces.wide)
        };
        match self.logo.image.or(maker) {
            Some(image) => {
                let only = Logo {
                    image: Some(image),
                    ..Logo::default()
                };
                fh6_kit::logo::draw(p, g.logo, &only, &style);
            }
            None => {
                fh6_kit::logo::draw(p, g.logo, &self.logo, &style);
            }
        }
    }

    /// The boot self-test: every segment lit.
    fn draw_test(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        self.draw_tach(p, g, pal, true);
        self.draw_bracket(p, g, pal, true);
        self.draw_pedals(p, g, pal, true);
        self.draw_slip(p, g, pal, true);
        self.draw_slants(p, g, pal, true);
        self.test_readout
            .draw(p, g.readout, &Seg7Geometry::new(g.readout_cap), pal, Emission::Matte);
        self.test_gear
            .draw(p, g.gear, &Seg7Geometry::new(g.gear_cap), pal, Emission::Matte);
        self.test_speed
            .draw(p, g.speed, &Seg7Geometry::new(g.speed_cap), pal, Emission::Matte);
    }

    /// The control-grid mesh over `zone` (multiply, so it only modulates lit phosphor), tiled
    /// from `origin` so every window shares one grid: the Phase 6 hex tile, or a procedural
    /// triangular lattice of three hairline families.
    fn draw_mesh(&self, p: &mut Painter<'_>, zone: Rect, origin: Vec2) {
        match self.mesh {
            Some(img) => {
                let tile = Rect::from_xywh(origin.x, origin.y, MESH_TILE, MESH_TILE);
                let mut paint = ImagePaint::new(img, tile).tint(Color::WHITE.with_alpha(MESH_STRENGTH));
                paint.wrap = ImageWrap::Repeat;
                p.fill(&Shape::rect(zone), paint);
            }
            None => {
                let dark = Color::BLACK.with_alpha(MESH_STRENGTH * 0.6);
                let clear = dark.with_alpha(0.0);
                for angle in [0.0_f32, 60.0, 120.0] {
                    let n = Vec2::from_angle(angle.to_radians() + core::f32::consts::FRAC_PI_2);
                    let paint = Gradient::linear(origin, origin + n * 6.0)
                        .stop(0.0, dark)
                        .stop(1.0 / 6.0, dark)
                        .stop(1.0 / 6.0, clear)
                        .stop(1.0, clear)
                        .spread(fh6_kit::Spread::Repeat);
                    p.fill(&Shape::rect(zone), paint);
                }
            }
        }
    }
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(RetroCyber::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inside(outer: Rect, inner: Rect) -> bool {
        outer.contains(inner.min) && outer.contains(inner.max)
    }

    fn overlaps(a: Rect, b: Rect) -> bool {
        a.min.x < b.max.x && b.min.x < a.max.x && a.min.y < b.max.y && b.min.y < a.max.y
    }

    #[test]
    fn layout_stays_inside_the_view_envelopes() {
        let t = Tokens::default();
        for (g, env) in [(&COCKPIT, t.layout.cockpit_envelope), (&HOOD, t.layout.hood_envelope)] {
            assert!(
                inside(env, g.lens.expand(g.surround)),
                "surround {:?} in {env:?}",
                g.lens
            );
            let w = g.windows();
            for (i, a) in w.iter().enumerate() {
                assert!(inside(g.lens.inset(8.0), *a), "{a:?} inside the lens");
                for b in &w[i + 1..] {
                    assert!(!overlaps(a.expand(8.0), *b), "windows {a:?} and {b:?} stay apart");
                }
            }
            // The A70 order, left to right.
            assert!(g.pedal.max.x < g.centre.min.x && g.centre.max.x < g.slip.min.x);
            assert!(g.slip.max.x < g.gauges.min.x);
            // Labels sit on the lens's lower band, under every window.
            for a in w {
                assert!(a.max.y < g.label_y && g.label_y + g.label_cap < g.lens.max.y);
            }
        }
    }

    #[test]
    fn tach_rises_to_the_right_and_stays_in_the_centre_window() {
        for g in [&COCKPIT, &HOOD] {
            let n = MAX_THOUSANDS * BARS_PER_K;
            let first = g.bar(0, n);
            let last = g.bar(n - 1, n);
            let h = |b: [Vec2; 4]| b[0].y - b[3].y;
            assert!(h(last) >= 3.0 * h(first), "bars grow to the right");
            assert!(last[0].y < first[0].y, "the baseline rises");
            // The bracket clears the window's top, and the strobing strip is at most 42 px.
            for u in [0.0f32, 0.5, 1.0] {
                assert!(
                    g.pod_base_at(u) - g.pod_h - 5.0 > g.centre.min.y + 2.0,
                    "bracket and rail at {u}"
                );
            }
            assert!(g.pod_h <= 42.0);
            // The numerals sit above the bar tops and below the bracket.
            for u in [0.0f32, 0.5, 1.0] {
                assert!(g.numeral_at(u) + g.numeral_cap * 0.5 < g.top_at(u));
                assert!(g.numeral_at(u) - g.numeral_cap * 0.5 > g.pod_base_at(u));
            }
        }
    }

    #[test]
    fn gear_speed_and_readout_clear_the_tach_and_each_other() {
        for g in [&COCKPIT, &HOOD] {
            let gb = Seg7Geometry::new(g.gear_cap).bounds(g.gear);
            let sg = Seg7Geometry::new(g.speed_cap);
            let sb = Rect::from_center_size(g.speed, vec2(sg.advance() * 3.0, g.speed_cap));
            let rg = Seg7Geometry::new(g.readout_cap);
            let rb = Rect::from_center_size(g.readout, vec2(rg.advance() * 2.0, g.readout_cap));
            for b in [gb, sb, rb] {
                assert!(inside(g.centre.inset(4.0), b), "{b:?} in the centre window");
                // Clear of the tach baseline above it.
                let u0 = ((b.min.x - g.tach_x.0) / (g.tach_x.1 - g.tach_x.0)).clamp(0.0, 1.0);
                let u1 = ((b.max.x - g.tach_x.0) / (g.tach_x.1 - g.tach_x.0)).clamp(0.0, 1.0);
                assert!(
                    b.min.y > g.base_at(u0).max(g.base_at(u1)) + 20.0,
                    "{b:?} under the bars"
                );
            }
            assert!(
                !overlaps(gb, sb.expand(60.0)),
                "gear and speed apart (unit label between)"
            );
            assert!(!overlaps(rb.expand(30.0), sb));
            // Rank: the gear is the largest figure, then the speed.
            assert!(g.gear_cap > g.speed_cap && g.speed_cap > 2.5 * g.numeral_cap);
            assert!(g.gear_cap >= 150.0, "tokens: hero gear cap");
        }
    }

    #[test]
    fn the_logo_zone_clears_the_legend_and_the_numerals() {
        for g in [&COCKPIT, &HOOD] {
            assert!(inside(g.centre.inset(4.0), g.logo));
            let legend_bottom = g.centre.min.y + g.figure_cap * 2.4;
            assert!(g.logo.min.y > legend_bottom + 4.0, "under the legend");
            let u = (g.logo.max.x - g.tach_x.0) / (g.tach_x.1 - g.tach_x.0);
            assert!(
                g.logo.max.y + 12.0 < g.numeral_at(u) - g.numeral_cap * 0.5,
                "over the numerals"
            );
        }
    }

    #[test]
    fn gauges_stay_in_their_windows() {
        for g in [&COCKPIT, &HOOD] {
            for col in 0..2 {
                for row in 0..PEDAL_ROWS {
                    assert!(inside(g.pedal.inset(2.0), g.pedal_cell(col, row)));
                }
            }
            for j in 0..SLIP_STEPS {
                for pt in g.slip_cell(j) {
                    assert!(g.slip.inset(2.0).contains(pt));
                }
            }
            for k in 0..2 {
                for j in 0..SLANT_STEPS {
                    for pt in g.slant_cell(k, j) {
                        assert!(g.gauges.inset(2.0).contains(pt), "slant {k} cell {j} {pt:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn scale_for_common_limiters() {
        let s = Scale::new(7000.0, 6790.0, 0.75);
        assert_eq!((s.top, s.red_from), (8, 7));
        assert_eq!(s.bars(), 32);
        assert_eq!(s.lit(0.0), 0);
        assert_eq!(s.lit(3000.0), 12);
        assert_eq!(s.lit(1e9), 32);
        assert_eq!(s.lit(f32::NAN), 0);
        assert!(s.bracket.0 < s.bracket.1 && s.bracket.1 <= 1.0);
        let hi = Scale::new(9000.0, 8730.0, 0.75);
        assert_eq!((hi.top, hi.red_from), (10, 9));
        let silly = Scale::new(f32::NAN, f32::NAN, f32::NAN);
        assert!(silly.top >= MIN_THOUSANDS && silly.top <= MAX_THOUSANDS);
        assert_eq!(Scale::new(1e9, 1e9, 0.75).top, MAX_THOUSANDS);
    }

    #[test]
    fn gauge_mapping() {
        assert_eq!(steps_lit(0.5, 0.0, 1.0, 10), 5);
        assert_eq!(steps_lit(-3.0, 0.0, 1.0, 10), 0);
        assert_eq!(steps_lit(f32::NAN, 0.0, 1.0, 10), 0);
        assert_eq!(steps_lit(0.0, BOOST_MIN_PSI, BOOST_MAX_PSI, SLANT_STEPS), 4);
        assert!((power_in(745.7, PowerUnit::Hp) - 1.0).abs() < 1e-4);
        assert!((power_in(1000.0, PowerUnit::Kw) - 1.0).abs() < 1e-4);
        assert_eq!(power_in(f32::NAN, PowerUnit::Kw), 0.0);
    }

    #[test]
    fn slow_readout_refreshes_at_5_hz_and_snaps_first() {
        let mut r = SlowReadout::new(1.0);
        r.update(7.0, 1.0 / 60.0, true);
        assert_eq!(r.value, 7);
        r.update(12.0, 1.0 / 60.0, false);
        assert_eq!(r.value, 7, "held until the next 5 Hz tick");
        for _ in 0..12 {
            r.update(12.0, 1.0 / 60.0, false);
        }
        assert_eq!(r.value, 12);
    }

    #[test]
    fn boot_runs_only_on_the_first_observed_race_start() {
        let mut b = Boot::default();
        b.observe(true, 0.016);
        assert!(!b.running());
        b.observe(false, 0.016);
        b.observe(true, 0.016);
        assert!(b.running(), "0 -> 1 boots");
        for _ in 0..40 {
            b.observe(true, 1.0 / 60.0);
        }
        assert!(!b.running(), "the boot ends within {BOOT_S} s");
        b.observe(false, 0.016);
        b.observe(true, 0.016);
        assert!(!b.running(), "only the first race start after launch boots");
        const _: () = assert!(BOOT_S <= 1.0);
    }

    #[test]
    fn sectors_ignite_left_to_right_and_settle() {
        assert_eq!(sector_level(0.1, 0), 0.0, "filaments warm first");
        assert_eq!(sector_level(0.16, 0), 1.0);
        assert_eq!(sector_level(0.16, 1), 0.0);
        assert!(sector_level(BOOT_S, 7) < 1e-4, "settled to ghost");
    }

    #[test]
    fn ghosts_are_opaque_and_dim() {
        let c = Color::linear(0.0, 1.0, 0.8, 1.0);
        let gh = ghost(c);
        assert_eq!(gh.a, 1.0);
        assert!((linear_to_srgb(gh.g) - GHOST_LEVEL).abs() < 1e-3);
    }

    #[test]
    fn print_stays_under_the_static_cap() {
        // V11: static parts at most 20% relative luminance (white ink at PRINT_LEVEL).
        assert!(srgb_to_linear(PRINT_LEVEL) <= 0.2);
    }
}
