//! Bonnet Line (`ClusterTheme::Gt7HoodPro`), catalog row 3: the Modern Digital 2027 family's
//! pattern-setter and the default hood theme (hood, build #2).
//!
//! Gran Turismo 7's Bonnet View meter (`RESEARCH.md`): one low band across the bottom of the
//! screen, floating over the bonnet with no panel behind it, rebuilt as 2027 production
//! hardware. The static parts are one Blender plate (`hardware`, recipe
//! `tools/asset-forge/src/recipes/gt7_hood_pro.rs`): a black-glass light bar on a polished plinth
//! with 95 smoked LED lenses, a forged-carbon car chip (the ComfyUI texture) with four tyre
//! lenses, the boost gauge's polished scale, the pedal slots, the telltale rings and the trip
//! tile. The live data is drawn on top of it. Board: Penpot page `07 · Gt7HoodPro`.
//!
//! Zone table (local 4K px round the hood anchor; the numbers are [`geom::HOOD`]):
//!
//! | Zone | Element | Place | Rank |
//! |---|---|---|---|
//! | Rev bar | 95 LEDs in a black-glass light bar, lit from the left with the revs | x −380..380, LEDs y −60..−34, bar 788 × 46 at y −47 | 1 (shift light) and 3 (tach) |
//! | Steering dot | a dot that slides with the steering | y −94, r 8, ±340 | 4 |
//! | Speed · gear | the tabular speed right-aligned to a thin divider on the centre line, the gear right of it | speed cap 58 ending at x −26, divider x 0, gear cap 88 from x 30 | 2 · 1 |
//! | Unit | `mph` / `km/h` under the speed | cap 22 at y 82 | 5 |
//! | Suggested gear | a red key tile naming the next gear at the shift point | 94 × 86 at (211, 49), digit cap 52 | 1 |
//! | Pedals | brake (left) and throttle (right) fills in black-glass slots | x ∓442, 16 × 150 | 4 |
//! | Tyre diagram | carbon car chip, four tyre lenses lit by tyre load | chip (−795, 18) 70 × 160 | 4 |
//! | Boost gauge | polished scale, a fill from 0 (−1..2 bar) and its label | (−600, 6), r 60 | 4 |
//! | Trip tile | distance since the view was entered, GT7's odometer box | (−600, 108), 200 × 36, cap 22 | 4 |
//! | Telltales | slide (left) and limiter (right) lamps | (∓497, −36), r 24 | 4 |
//! | Logo zone | the car's logo or the custom name | x 540..860, y −40..60 | 5 |
//!
//! The cockpit and chase views draw the same band scaled to fit the cockpit envelope.
//!
//! Brightness follows the rank: the gear and the speed are the full ink, the lit LEDs burn at
//! 78% with the head LED at full, the side groups sit below them, and the parts that never
//! change (labels, the divider, the logo) are held at the tokens' cap for static parts
//! ([`STATIC_LUMA`]), dimmed by their light and never by their coverage. Over a bright bonnet or
//! sky every glyph, the divider and the logo carry a black keyline, as GT7's digits carry a dark
//! edge; nothing sits on a box. The live fills run in the plate's black-glass channels.
//!
//! Signature moments:
//! - **Shift point and limiter (GT7's rev alert):** from the warning stage the whole lit run takes
//!   the `shift` colour at once; at the shift point the run strobes (10 Hz on the strip's own
//!   clock, so it starts lit, with a fifth of the light kept in the dark phase) and the red
//!   suggested-gear tile pops up naming the next gear. The tile itself never strobes.
//! - **Gear change, the hand-over:** when the driver takes the suggested gear, the tile brightens
//!   for one frame and folds shut towards the gear in 100 ms, while the new gear rises into
//!   place with the tokens' gear motion (`common::hero_gear`). The tile stays away until the
//!   revs have fallen below the shift point.
//! - **Light:** lateral G swings the virtual light, and the plate's three renders cross-fade with
//!   it, so the chamfers and lenses catch the light as the car turns.
//! - **Reduced flash:** the strobe becomes the 2 Hz breath.
//!
//! Telemetry is native: rpm (current, limiter), the tokens' shift point, gear, speed, throttle,
//! brake, steer, acceleration (tyre loads by weight transfer, and the light), body slip from
//! velocity (the slide lamp) and boost. The trip integrates the speed. Tyre heat, wear, fuel,
//! assists and the driving-line suggestion are not in the verified telemetry, so they are left
//! out or replaced by the nearest verified quantity (`RESEARCH.md`, "Deliberate changes").
mod geom;
mod hardware;

use crate::common::gball::felt_force;
use crate::common::hero_gear::HeroGear;
use crate::common::speed::TabularSpeed;
use crate::common::VirtualLight;
use core::f32::consts::TAU;
use fh6_kit::cluster::Setup;
use fh6_kit::logo::{self, LogoStyle};
use fh6_kit::{
    vec2, Align, BlendMode, Bloom, Cap, Cluster, Color, FontId, FontRole, FrameInput, Gear, Gradient, LayerOptions,
    Layout, Logo, Paint, Painter, Palette, Rect, Shape, Signals, Smoothed, SpeedUnit, Spring, Strobe, Stroke,
    TextStyle, Tokens, VAlign, Vec2,
};
use geom::{rect, Geom, HOOD};
use hardware::Sprites;

/// Michroma's digits are 0.95 to 1.0 em wide and the kit's tabular cell is 0.60 em, so the
/// speed and trip cells add this much tracking (em) to hold them.
const MICHROMA_TRACKING: f32 = 0.36;
/// The lit LEDs' share of the light below the head LED, so the bar never outshines the gear.
const TICK_BODY: f32 = 0.78;
/// Release time constant of the LED run (tokens `motion.shiftLights`: release within 50 ms).
const TICK_RELEASE_TAU: f32 = 0.025;
/// Width of the LED run's backlash (rpm): wider than the engine's bounce on the limiter (about
/// 50 rpm from one frame to the next), so the head LED holds still there; a pull is drawn at
/// most 30 rpm behind, a third of one LED at the review car's span.
const REV_BAND: f32 = 60.0;
/// Off-hysteresis of the shift point and of the alert (tokens' shift hysteresis, rpm, as
/// PorscheAnalog): an rpm hovering on a threshold neither flashes the tile nor restarts the
/// strobe's clock.
const SHIFT_HYSTERESIS_RPM: f32 = 120.0;
/// Below this limiter (rpm) the telemetry has not named the car's limiter (a zero packet gives
/// 1 rpm): the bar stays dark and no alert runs.
const MIN_LIMITER_RPM: f32 = 1000.0;
/// The kit's tabular cell (em): a user face's tracking is its widest digit less this.
const TABULAR_CELL_EM: f32 = 0.6;
/// Keeps a strobe phase edge that falls on a frame from rounding either way (the pilot's).
const STROBE_LEAD: f32 = 1e-4;
/// Share of the shift colour's light the LEDs keep in the strobe's dark phase.
const STROBE_FLOOR: f32 = 0.2;
/// Luminance cap of the parts that never change (labels, divider, scale mark, logo).
const STATIC_LUMA: f32 = 0.19;
/// The alert run's bloom: the shift colour is under the tokens' bloom threshold and would not
/// bloom at all, so the alert run glowed less than the white run it replaces. The white run
/// keeps the tokens' bloom.
const ALERT_BLOOM: Bloom = Bloom {
    threshold: 0.2,
    intensity: 0.8,
};
/// An alert LED's hot core: its inset inside the LED (x, y) and its share of the ink. The core
/// makes the alert run brighter than the white run it replaces, as an LED driven harder is.
const CORE_INSET: [f32; 2] = [1.5, 4.0];
const CORE_LIGHT: f32 = 0.9;
/// The alert zone: the unlit lenses from where the run takes the shift colour are tinted the
/// warning red at this share of its light (a static part, under the tokens' cap). The zone
/// follows the driver's shift point, so it is drawn live, not baked into the plate.
const ZONE_LIGHT: f32 = 0.22;
/// The thousand-rpm marks: their width, and the limiter range they are drawn for.
const MARK_W: f32 = 2.0;
const MARK_RPM: [f32; 2] = [1000.0, 20_000.0];
/// The pedal fills' share of the ink (under the speed, so a full bar never outlasts it).
const PEDAL_LIGHT: f32 = 0.7;
/// The boost fill's share of the ink (under the lit run's, rank 4 below rank 3).
const BOOST_LIGHT: f32 = 0.7;
/// The trip digits' share of the ink (GT7's odometer is light grey, under the speed).
const TRIP_LIGHT: f32 = 0.72;
/// Tyre load per g of felt force on each axis, round the rest load of a half.
const LOAD_PER_G: f32 = 0.35;
/// A tyre lens's light at no load and at full load (below the speed's full ink: rank 4), and
/// the share of it left at the lens's ends: the light sits inside the lens, hot in the middle.
const PILL_LIGHT: [f32; 2] = [0.1, 0.5];
const PILL_EDGE: f32 = 0.2;
/// The lit tyre fill's inset inside its lens.
const PILL_INSET: f32 = 3.0;
/// Pounds per square inch to bar (x100 kPa).
const PSI_TO_BAR: f32 = 0.068_947_6;
/// The steering byte at full lock.
const STEER_FULL: f32 = 127.0;
/// Time constant of the steering dot (s): it rides out one packet's jitter.
const STEER_TAU: f32 = 0.03;
/// The slide lamp lights at this body slip (degrees) from 8 m/s (29 km/h), and goes out below
/// the lower slip or under three quarters of that speed.
const SLIDE_ON_DEG: f32 = 8.0;
const SLIDE_OFF_DEG: f32 = 5.0;
const SLIDE_MIN_MS: f32 = 8.0;
/// The limiter lamp lights at this share of the limiter and goes out below the lower one.
const LIM_ON: f32 = 0.985;
const LIM_OFF: f32 = 0.975;
/// A lit lamp's glow behind its icon (alpha at the centre).
const LAMP_GLOW: f32 = 0.32;
/// The highest gear the telemetry can name.
const TOP_GEAR: u8 = 10;
/// The suggested-gear tile: how long its hand-over runs after the driver takes the gear, how
/// fast it fades when the revs drop without a shift (time constant: five of them, 50 ms, take it
/// out), and how much larger it pops up.
const FOLD_S: f32 = 0.1;
const FADE_TAU: f32 = 0.01;
const POP: f32 = 0.08;
/// The hand-over: the tile slides this far towards the gear and loses this share of its size
/// while it fades, whole, with its digit.
const HAND_SLIDE: f32 = 90.0;
const HAND_SHRINK: f32 = 0.45;
/// The tile's digit keeps this far from the face's sides.
const TILE_MARGIN: f32 = 10.0;
/// The tile's face brightens on the frame the driver takes the gear (a click of light, kept
/// below the gear's own light so the new gear leads).
const FLASH_ALPHA: f32 = 0.4;
/// Hood keylines (tokens `type.keyline`: a black ring round the strokes, never a box, alpha
/// 230): the ring's width per cap height, its narrowest and widest, and its coverage.
const KEYLINE_PER_CAP: f32 = 0.1;
const KEYLINE_MIN: f32 = 3.0;
const KEYLINE_MAX: f32 = 8.0;
const KEYLINE_ALPHA: f32 = 0.9;
/// The rings of a keyline: copies, radius in keyline widths and coverage, the outermost first.
/// The gear and the speed add a wider, soft ring, so their edge holds over a sunlit road.
const KEYLINE_HARD: [(usize, f32, f32); 1] = [(8, 1.0, KEYLINE_ALPHA)];
const KEYLINE_SOFT: [(usize, f32, f32); 2] = [(12, 1.7, 0.25), (8, 1.0, KEYLINE_ALPHA)];
/// The speed's synthetic weight: Michroma has one weight, and its thin strokes faded before
/// the side groups in the squint view.
const SPEED_WEIGHT: f32 = 0.3;
/// Labels' tracking (em) and the logo zone's wordmark cap and tracking. A wordmark that would
/// shrink under the label floor to fit is set in the label face at that floor instead.
const LABEL_TRACKING: f32 = 0.08;
const LOGO_CAP: f32 = 28.0;
const LOGO_TRACKING: f32 = 0.3;
const LOGO_MIN_CAP: f32 = 22.0;
/// Metres per mile and per kilometre.
const M_PER_MILE: f64 = 1609.344;
const M_PER_KM: f64 = 1000.0;
/// The trip tile shows three whole digits, tenths and the unit.
const TRIP_MAX_TENTHS: i64 = 9_999;
/// The trip tile: the text's inset from the face's right edge, the gap between the number and
/// its unit, and the decimal point's size.
const TRIP_PAD: f32 = 12.0;
const TRIP_GAP: f32 = 8.0;
const TRIP_POINT: f32 = 5.0;

/// The faces, resolved once in `new()`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    /// Speed and trip digits (Michroma: GT7's squared digits, with a plain zero).
    speed: FontId,
    /// Gear and the tile's digit (Orbitron 600).
    gear: FontId,
    /// Units and labels (Titillium Web 600).
    label: FontId,
    /// The logo zone's wordmark.
    wide: FontId,
    /// Extra tabular tracking (em) for the speed face; `None` for a user face, whose tracking
    /// is measured from its digits when it is drawn.
    tracking: Option<f32>,
}

/// The rev bar's state: the LED run, GT7's alert and the strobe's clock.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct RevBar {
    /// The rpm the run shows: the raw rpm through a [`REV_BAND`] backlash.
    rpm: f32,
    /// LEDs lit (fractional while a release runs) and where the run is going.
    lit: f32,
    target: f32,
    /// From the warning stage the whole run takes the shift colour.
    alert: bool,
    /// At the shift point the run strobes.
    over: bool,
    /// Seconds since the shift point (wrapped at 1 s, a whole number of strobe and breath
    /// periods); `None` below it.
    since: Option<f32>,
    /// The first LED of the alert zone (the LED the head is on when the run takes the shift
    /// colour); `None` while the limiter is unknown.
    zone: Option<usize>,
}

impl RevBar {
    /// Advances one frame. The frame that reaches the shift point reads 0 on the strip's clock,
    /// so the strobe starts on its lit phase whatever the app's clock reads.
    fn update(&mut self, sig: &Signals, palette: &Palette, ticks: usize, dt: f32, fresh: bool) {
        self.rpm = if fresh {
            finite(sig.rpm)
        } else {
            backlash(self.rpm, sig.rpm, REV_BAND)
        };
        let known = finite(sig.max_rpm) >= MIN_LIMITER_RPM;
        self.target = if known {
            fraction(self.rpm, sig.max_rpm) * ticks as f32
        } else {
            0.0
        };
        if fresh || self.target >= self.lit {
            self.lit = self.target;
        } else {
            let k = 1.0 - (-dt / TICK_RELEASE_TAU).exp();
            self.lit += (self.target - self.lit) * k;
        }
        let shift = finite(sig.shift_fraction);
        // Same-frame attack; a crossing back releases only 120 rpm below the threshold.
        let h = if known && !fresh {
            SHIFT_HYSTERESIS_RPM / finite(sig.shift_rpm).max(MIN_LIMITER_RPM)
        } else {
            0.0
        };
        self.over = known && shift >= if self.over { 1.0 - h } else { 1.0 };
        let stage = palette.stages[2];
        self.alert = known && (self.over || shift >= if self.alert { stage - h } else { stage });
        self.zone = known.then(|| alert_zone(stage, sig.shift_rpm, sig.max_rpm, ticks));
        self.since = match (self.over, self.since) {
            (false, _) => None,
            (true, None) => Some(0.0),
            (true, Some(t)) => Some((t + dt).rem_euclid(1.0)),
        };
    }

    /// LEDs lit this frame.
    fn count(&self, ticks: usize) -> usize {
        let n = (self.lit + 0.5).floor();
        if n.is_finite() && n > 0.0 {
            (n as usize).min(ticks)
        } else {
            0
        }
    }

    /// The time the strobe and the breath read.
    fn clock(&self) -> f32 {
        self.since.map_or(0.0, |t| t + STROBE_LEAD)
    }

    /// The strobe's share of the light this frame (1 below the shift point).
    fn level(&self, calm: bool) -> f32 {
        if !self.over {
            1.0
        } else if calm {
            Strobe::breath(self.clock()).max(STROBE_FLOOR)
        } else if Strobe::LIMITER.on(self.clock()) {
            1.0
        } else {
            STROBE_FLOOR
        }
    }

    /// The colours of the lit run's body and its head LED.
    fn colors(&self, pal: &Palette, calm: bool) -> (Color, Color) {
        if self.over {
            let c = pal.shift.intensity(self.level(calm));
            return (c, c);
        }
        let base = if self.alert { pal.shift } else { pal.roles.ink };
        (base.intensity(TICK_BODY), base)
    }

    /// The hot core of every lit LED while the alert runs (`None` below it).
    fn core(&self, pal: &Palette, calm: bool) -> Option<Color> {
        (self.alert || self.over).then(|| pal.roles.ink.intensity(CORE_LIGHT * self.level(calm)))
    }

    fn animating(&self) -> bool {
        self.over || (self.lit - self.target).abs() > 0.01
    }
}

/// The suggested-gear tile: GT7's red gear box, here naming the next gear at the shift point.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Tile {
    /// The gear the tile names (`None` while it is hidden).
    digit: Option<u8>,
    /// How much of the tile shows (0..=1).
    level: f32,
    /// The pop as it appears: 1 falling to 0 on the `snap` spring (the tile starts 8% larger).
    pop: Spring,
    /// The hand-over after the driver takes the gear (0..1), `None` when it is not running.
    fold: Option<f32>,
    /// True on the frame the hand-over starts: the face brightens.
    flash: bool,
    /// The tile may show: false from a gear change until the revs fall below the shift point.
    armed: bool,
}

impl Tile {
    fn new(tokens: &Tokens) -> Self {
        Self {
            digit: None,
            level: 0.0,
            pop: Spring::new(tokens.springs.snap, 0.0),
            fold: None,
            flash: false,
            armed: true,
        }
    }

    /// Advances one frame: `changed` is true on the frame the gear changes, `over` while the
    /// revs are past the shift point.
    fn update(&mut self, gear: Gear, changed: bool, over: bool, dt: f32) {
        self.flash = false;
        if changed {
            let took = self.level > 0.5 && matches!((gear, self.digit), (Gear::Forward(n), Some(d)) if n == d);
            self.armed = false;
            if took {
                self.fold = Some(0.0);
                self.flash = true;
            }
        }
        if !over {
            self.armed = true;
        }
        let target = match gear {
            Gear::Forward(n) if over && self.armed && n < TOP_GEAR => Some(n + 1),
            _ => None,
        };
        if let Some(d) = target {
            if self.digit.is_none() || self.level < 0.5 {
                self.pop.reset(1.0);
            }
            self.digit = Some(d);
            self.level = 1.0;
            self.fold = None;
            self.flash = false;
        } else if let Some(f) = self.fold {
            // The flash frame shows the tile where it stood; the hand-over runs from the next.
            let f = if self.flash { f } else { f + dt / FOLD_S };
            if f >= 1.0 {
                self.fold = None;
                self.level = 0.0;
                self.digit = None;
            } else {
                self.fold = Some(f);
            }
        } else if self.level > 0.0 {
            let k = 1.0 - (-dt / FADE_TAU).exp();
            self.level -= self.level * k;
            if self.level < 0.01 {
                self.level = 0.0;
                self.digit = None;
            }
        }
        self.pop.update(0.0, dt);
    }

    fn shown(&self) -> bool {
        self.level > 0.0 || self.fold.is_some()
    }

    fn animating(&self) -> bool {
        self.fold.is_some() || (self.level > 0.0 && self.level < 1.0) || !self.pop.settled(0.0, 1e-3)
    }
}

/// The Bonnet Line cluster (`Gt7HoodPro`).
#[derive(Debug, Clone)]
pub struct Gt7HoodPro {
    faces: Faces,
    /// The faces resolved in `new()`, brought back when the user clears their display font.
    own: Faces,
    /// The display role at creation, and the display font last seen, to notice a user font.
    role_display: FontId,
    seen_display: FontId,
    sprites: Sprites,
    sig: Signals,
    calm: bool,
    /// True until the first frame: a cluster built mid-race shows what it finds, with no gear
    /// change, pop or sweep played.
    fresh: bool,
    logo: Logo,
    light: VirtualLight,
    rev: RevBar,
    gear: HeroGear,
    speed: TabularSpeed,
    /// The steering dot (−1 full left .. 1 full right) and its target.
    steer: Smoothed<f32>,
    steer_target: f32,
    /// The boost fill (bar) on the tokens' needle spring, and its target.
    boost: Spring,
    boost_target: f32,
    /// Metres driven since this view was last entered (while racing): the app builds the
    /// cluster again on a camera or theme change, so the trip starts again then.
    trip_m: f64,
    /// The telltales.
    slide: bool,
    limiter: bool,
    tile: Tile,
}

impl Gt7HoodPro {
    /// Builds the cluster: faces, springs and the rendered hardware (procedural without it).
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let sprites = Sprites::load(setup);
        let t: &Tokens = setup.tokens;
        let display = setup.fonts.role(FontRole::Display);
        let text = setup.fonts.role(FontRole::Text);
        let wide = setup.fonts.role(FontRole::Wide);
        let (speed, tracking) = match setup.fonts.find("Michroma", 400, false) {
            Some(face) => (face, Some(MICHROMA_TRACKING)),
            None => (wide, None),
        };
        let faces = Faces {
            speed,
            gear: setup.fonts.find("Orbitron", 600, false).unwrap_or(display),
            label: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            wide,
            tracking,
        };
        Self {
            faces,
            own: faces,
            role_display: display,
            seen_display: display,
            sprites,
            sig: Signals::default(),
            calm: false,
            fresh: true,
            logo: Logo::default(),
            light: VirtualLight::new(t),
            rev: RevBar::default(),
            gear: HeroGear::new(t),
            speed: TabularSpeed::new(),
            steer: Smoothed::new(0.0, STEER_TAU),
            steer_target: 0.0,
            boost: Spring::new(t.springs.needle, 0.0),
            boost_target: 0.0,
            trip_m: 0.0,
            slide: false,
            limiter: false,
            tile: Tile::new(t),
        }
    }
}

impl Cluster for Gt7HoodPro {
    fn update(&mut self, input: &FrameInput<'_>) {
        // A frame time that is not a number, or a long stall, never reaches a clock.
        let dt = if input.dt.is_finite() {
            input.dt.clamp(0.0, 0.25)
        } else {
            0.0
        };
        let sig = *input.signals;
        self.sig = sig;
        self.calm = input.reduced_flash;
        // A user display font (FontChoice.display differs from the role) replaces the numerals;
        // back on the role, the dash's own faces return.
        let display = input.fonts.display;
        if display != self.seen_display {
            self.seen_display = display;
            self.faces = if display == self.role_display {
                self.own
            } else {
                Faces {
                    speed: display,
                    gear: display,
                    tracking: None,
                    ..self.own
                }
            };
        }
        if self.logo != *input.logo {
            self.logo.clone_from(input.logo);
        }
        let g = &HOOD;
        let fresh = self.fresh;

        self.rev.update(&sig, input.palette, g.ticks, dt, fresh);
        if fresh {
            self.gear.snap(sig.gear);
        }
        self.gear.update(sig.gear, 0.0, dt);
        let changed = !fresh && self.gear.since_change() == 0.0;
        self.tile.update(sig.gear, changed, self.rev.over, dt);
        if fresh {
            // A tile found on the first frame is shown at rest, with no pop.
            self.tile.pop.reset(0.0);
        }
        self.speed.update(sig.speed);

        self.steer_target = (f32::from(input.telemetry.steer) / STEER_FULL).clamp(-1.0, 1.0);
        let bar = finite(input.telemetry.boost_psi) * PSI_TO_BAR;
        self.boost_target = bar.clamp(g.boost_bar[0], g.boost_bar[1]);
        if fresh {
            self.steer.reset(self.steer_target);
            self.boost.reset(self.boost_target);
        } else {
            self.steer.update(self.steer_target, dt);
            self.boost.update(self.boost_target, dt);
        }

        if sig.race_on {
            self.trip_m += f64::from(finite(sig.speed_ms).max(0.0)) * f64::from(dt);
        }

        let slip = finite(sig.slip_deg).abs();
        let moving = finite(sig.speed_ms);
        // Reversing in a straight line travels at about 180° of slip: not a slide.
        let forward = sig.gear != Gear::Reverse;
        self.slide = forward
            && if self.slide {
                slip > SLIDE_OFF_DEG && moving > SLIDE_MIN_MS * 0.75
            } else {
                slip >= SLIDE_ON_DEG && moving >= SLIDE_MIN_MS
            };
        let at = if finite(sig.max_rpm) >= MIN_LIMITER_RPM {
            fraction(self.rev.rpm, sig.max_rpm)
        } else {
            0.0
        };
        self.limiter = if self.limiter { at > LIM_OFF } else { at >= LIM_ON };

        self.light.update(sig.g_lat, dt);
        self.fresh = false;
    }

    fn draw(&self, p: &mut Painter<'_>, layout: &Layout) {
        let g = &HOOD;
        let pal = *p.palette();
        let (k, off) = geom::fit(g, layout.envelope);
        p.scoped(|p| {
            p.translate(off);
            p.scale(k);
            let w = hardware::light_weights(self.light.angle());
            if !self.sprites.draw_plate(p, g, w) {
                hardware::fallback_plate(p, g, &pal);
            }
            self.draw_zone(p, g, &pal);
            self.draw_rev(p, g, &pal);
            self.draw_marks(p, g, &pal);
            self.draw_steer(p, g, &pal);
            self.draw_pedals(p, g, &pal);
            self.draw_tyres(p, g, &pal);
            self.draw_boost(p, g, &pal);
            self.draw_trip(p, g, &pal);
            self.draw_lamps(p, g, &pal);
            self.draw_readout(p, g, &pal);
            self.draw_tile(p, g, &pal, w);
            self.draw_logo(p, g, &pal);
        });
    }

    fn animating(&self) -> bool {
        self.rev.animating()
            || self.gear.animating()
            || self.tile.animating()
            || !self.boost.settled(self.boost_target, 1e-3)
            || (self.steer.value() - self.steer_target).abs() > 1e-3
    }
}

impl Gt7HoodPro {
    /// The tabular tracking (em) of the speed face at `cap`: the dash's own for Michroma, or
    /// for a user face the widest digit's advance less the kit's cell, so its digits never
    /// overlap.
    fn tracking(&self, p: &Painter<'_>, cap: f32) -> f32 {
        if let Some(t) = self.faces.tracking {
            return t;
        }
        let style = TextStyle::new(self.faces.speed, cap);
        let em = cap / p.fonts().cap_ratio(self.faces.speed).max(0.1);
        let widest = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
            .iter()
            .map(|d| p.measure(d, &style).width)
            .filter(|w| w.is_finite())
            .fold(0.0, f32::max);
        (widest / em.max(1e-3) - TABULAR_CELL_EM).max(0.0)
    }

    /// The lit LEDs over the plate's lenses (the unlit lenses are the plate's).
    fn draw_rev(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let n = self.rev.count(g.ticks);
        if n == 0 {
            return;
        }
        let (body, head) = self.rev.colors(pal, self.calm);
        let core = self.rev.core(pal, self.calm);
        let opts = LayerOptions {
            bloom: Some(if core.is_some() { ALERT_BLOOM } else { Bloom::default() }),
            ..LayerOptions::default()
        };
        p.layer(opts, |p| {
            for i in 0..n {
                let r = g.tick_rect(i);
                let color = if i + 1 == n { head } else { body };
                p.fill(&Shape::round_rect(r, g.tick_r), color);
                if let Some(core) = core {
                    let hot = Rect::from_points(
                        r.min + vec2(CORE_INSET[0], CORE_INSET[1]),
                        r.max - vec2(CORE_INSET[0], CORE_INSET[1]),
                    );
                    p.fill(&Shape::round_rect(hot, 1.0), core);
                }
            }
        });
    }

    /// The alert zone's unlit lenses, tinted red, so the bar shows where the run will change
    /// colour before it gets there.
    fn draw_zone(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let Some(from) = self.rev.zone else { return };
        let red = pal.roles.warning.intensity(ZONE_LIGHT);
        for i in from.max(self.rev.count(g.ticks))..g.ticks {
            p.fill(&Shape::round_rect(g.tick_rect(i), g.tick_r), red);
        }
    }

    /// The thousand-rpm marks under the bar, placed by the car's limiter: they make the bar a
    /// scale. A limiter the telemetry has not named yet draws none.
    fn draw_marks(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let max = finite(self.sig.max_rpm);
        if !(MARK_RPM[0]..=MARK_RPM[1]).contains(&max) {
            return;
        }
        let ink = static_ink(pal.roles.ink);
        let black = pal.roles.oled_black.with_alpha(KEYLINE_ALPHA);
        let [top, bottom] = g.mark_y;
        let mut rpm = MARK_RPM[0];
        while rpm <= max {
            let x = g.rev_x_at(rpm, max);
            let mark = Rect::from_xywh(x - MARK_W * 0.5, top, MARK_W, bottom - top);
            p.fill(&Shape::round_rect(mark.expand(2.0), 2.0), black);
            p.fill(&Shape::rect(mark), ink);
            rpm += MARK_RPM[0];
        }
    }

    /// The steering dot over the rev bar's centre, with a black keyline.
    fn draw_steer(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let c = vec2(dot_x(g, self.steer.value()), g.dot_y);
        p.fill(
            &Shape::circle(c, g.dot_r + 2.5),
            pal.roles.oled_black.with_alpha(KEYLINE_ALPHA),
        );
        p.fill(&Shape::circle(c, g.dot_r), pal.primary_text);
    }

    /// The brake and throttle fills, rising in their slots, and their labels.
    fn draw_pedals(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let ink = pal.roles.ink.intensity(PEDAL_LIGHT);
        for (i, level) in [self.sig.brake, self.sig.throttle].into_iter().enumerate() {
            let r = g.fill_rect(i, level);
            if r.height() >= 1.0 {
                p.fill(&Shape::round_rect(r, 2.0), ink);
            }
        }
        let style = TextStyle::new(self.faces.label, g.slot_label_cap)
            .paint(static_ink(pal.roles.ink))
            .tracking(LABEL_TRACKING)
            .align(Align::Center, VAlign::CapTop);
        for (i, label) in ["BRK", "THR"].into_iter().enumerate() {
            let pos = vec2(g.slots_x[i], g.slot_label_top);
            keyline_text(p, pos, label, &style, pal.roles.oled_black, &KEYLINE_HARD);
            p.text(pos, label, &style);
        }
    }

    /// The tyre lenses lit by tyre load: the felt force moves weight to the outside of a turn
    /// and to the front under braking.
    fn draw_tyres(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        p.scoped(|p| {
            // Screened over the plate, so the lens keeps its own highlights over the light.
            p.set_blend(BlendMode::Screen);
            for i in 0..4 {
                let level = PILL_LIGHT[0] + (PILL_LIGHT[1] - PILL_LIGHT[0]) * tyre_load(g, i, &self.sig);
                let r = g.pill_rect(i).inset(PILL_INSET);
                let ink = pal.roles.ink.intensity(level);
                let light = Gradient::radial(r.center(), r.height() * 0.5)
                    .stop(0.0, ink)
                    .stop(1.0, ink.intensity(PILL_EDGE));
                p.fill(&Shape::round_rect(r, r.width() * 0.4), light);
            }
        });
    }

    /// The boost fill from zero along the plate's groove, the zero mark and the label.
    fn draw_boost(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let [track_r, track_w] = g.boost_track;
        let zero = g.boost_angle(0.0);
        let sweep = g.boost_angle(self.boost.value()) - zero;
        if sweep.abs() > 0.004 {
            p.fill(
                &Shape::arc(g.boost_c, track_r, track_w, zero, sweep, Cap::Round),
                pal.roles.ink.intensity(BOOST_LIGHT),
            );
        }
        // The zero mark across the groove, over the fill.
        let ink = static_ink(pal.roles.ink);
        let (s, c) = zero.sin_cos();
        let dir = vec2(c, s);
        let half = g.boost_groove * 0.5 - 1.0;
        p.fill(
            &Shape::line(
                g.boost_c + dir * (track_r - half),
                g.boost_c + dir * (track_r + half),
                2.0,
            ),
            ink,
        );
        let style = TextStyle::new(self.faces.label, g.boost_label_cap)
            .paint(ink)
            .tracking(LABEL_TRACKING)
            .align(Align::Center, VAlign::CapTop);
        let pos = vec2(g.boost_c.x, g.boost_label_top);
        keyline_text(p, pos, "BOOST", &style, pal.roles.oled_black, &KEYLINE_HARD);
        p.text(pos, "BOOST", &style);
    }

    /// The trip in the user's unit, as GT7's odometer box: whole units, a point, tenths and the
    /// unit, set from the right so the digits never move as the number grows.
    fn draw_trip(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let (per_unit, unit) = match self.sig.unit {
            SpeedUnit::Mph => (M_PER_MILE, "mi"),
            SpeedUnit::Kmh => (M_PER_KM, "km"),
        };
        let (whole, tenth) = trip_parts(self.trip_m / per_unit);
        let y = g.trip_c.y;
        let unit_style = TextStyle::new(self.faces.label, g.trip_cap)
            .paint(static_ink(pal.roles.ink))
            .tracking(LABEL_TRACKING)
            .align(Align::Right, VAlign::CapMiddle);
        let right = g.trip_face_rect().max.x - TRIP_PAD;
        p.text(vec2(right, y), unit, &unit_style);
        let unit_w = finite(p.measure(unit, &unit_style).width);

        let ink = pal.roles.ink.intensity(TRIP_LIGHT);
        let style = TextStyle::new(self.faces.speed, g.trip_cap)
            .paint(ink)
            .tabular()
            .tracking(self.tracking(p, g.trip_cap))
            .align(Align::Right, VAlign::CapMiddle);
        let cell = p.fonts().tabular_cell(&style);
        let bias = tabular_bias(p, &style);
        let tenth_right = right - unit_w - TRIP_GAP;
        p.number(vec2(tenth_right + bias, y), tenth, 1, &style, None);
        // The point sits on the baseline between the two fields, a narrow cell of its own.
        let point_x = tenth_right - cell - TRIP_POINT;
        let point = Rect::from_xywh(point_x, y + g.trip_cap * 0.5 - TRIP_POINT, TRIP_POINT, TRIP_POINT);
        p.fill(&Shape::round_rect(point, 1.0), ink);
        let cells = if whole >= 100 {
            3
        } else if whole >= 10 {
            2
        } else {
            1
        };
        p.number(vec2(point_x - TRIP_POINT * 0.4 + bias, y), whole, cells, &style, None);
    }

    /// The telltales: grey when off (GT7), the slide lamp amber and the limiter lamp red when on.
    fn draw_lamps(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let off = static_ink(pal.roles.ink);
        for (i, c) in g.lamps.into_iter().enumerate() {
            let (on, color) = if i == 0 {
                (self.slide, pal.roles.caution)
            } else {
                (self.limiter, pal.roles.warning)
            };
            if on {
                p.fill(
                    &Shape::circle(c, g.lamp_face),
                    Gradient::radial(c, g.lamp_face)
                        .stop(0.0, color.with_alpha(LAMP_GLOW))
                        .stop(1.0, color.with_alpha(0.0)),
                );
                p.stroke(&Shape::circle(c, g.lamp_face), color, Stroke::rim_px(1.5));
            }
            let ink = if on { color } else { off };
            if i == 0 {
                hardware::slide_icon(p, c, ink);
            } else {
                hardware::limiter_icon(p, c, ink);
            }
        }
    }

    /// The speed, its unit, the divider and the gear, each glyph with a black keyline.
    fn draw_readout(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let black = pal.roles.oled_black;
        let mut speed = TextStyle::new(self.faces.speed, g.speed_cap)
            .paint(pal.roles.ink)
            .tabular()
            .tracking(self.tracking(p, g.speed_cap))
            .align(Align::Right, VAlign::CapTop);
        speed.weight = SPEED_WEIGHT;
        let bias = tabular_bias(p, &speed);
        let pos = vec2(g.speed_right + bias, g.speed_top);
        let value = self.speed.value().clamp(0, 999);
        keyline_number(p, pos, value, g.speed_cells, &speed, black, &KEYLINE_SOFT);
        p.number(pos, value, g.speed_cells, &speed, None);

        let ink = static_ink(pal.roles.ink);
        let unit = match self.sig.unit {
            SpeedUnit::Mph => "mph",
            SpeedUnit::Kmh => "km/h",
        };
        // Right-aligned with the digits, so it sits under the speed whether that is 7 or 207.
        let unit_style = TextStyle::new(self.faces.label, g.unit_cap)
            .paint(ink)
            .tracking(LABEL_TRACKING)
            .align(Align::Right, VAlign::CapTop);
        let unit_pos = vec2(g.speed_right - bias, g.unit_top);
        keyline_text(p, unit_pos, unit, &unit_style, black, &KEYLINE_HARD);
        p.text(unit_pos, unit, &unit_style);
        let divider = rect(g.divider);
        p.fill(
            &Shape::round_rect(divider.expand(KEYLINE_MIN), KEYLINE_MIN),
            black.with_alpha(KEYLINE_ALPHA),
        );
        p.fill(&Shape::rect(divider), ink);

        // The gear: a two-digit gear is set smaller to keep clear of the tile. Neutral is the
        // full ink like a forward gear (rank 1 in every state); reverse keeps its warning colour.
        let label = self.gear.gear().label();
        let color = match self.gear.gear() {
            Gear::Neutral => pal.roles.ink,
            _ => self.gear.color(pal),
        };
        let mut style = TextStyle::new(self.faces.gear, g.gear_cap)
            .paint(color)
            .align(Align::Left, VAlign::CapTop);
        let width = p.measure(label, &style).width;
        if width > g.gear_max_w && width.is_finite() {
            style.cap = g.gear_cap * g.gear_max_w / width;
        }
        let width = width.min(g.gear_max_w);
        let (dy, scale) = self.gear.entry();
        let center = vec2(g.gear_left + width * 0.5, g.gear_top + g.gear_cap * 0.5);
        let at = vec2(-width * 0.5, -style.cap * 0.5);
        p.scoped(|p| {
            p.translate(center + vec2(0.0, dy));
            p.scale(scale);
            keyline_text(p, at, label, &style, black, &KEYLINE_SOFT);
            p.text(at, label, &style);
        });
    }

    /// The suggested-gear tile: it pops up at the shift point with an up chevron over the next
    /// gear. When the driver takes that gear the face brightens for a frame, then the whole tile,
    /// digit and all, slides towards the gear, shrinking and fading (the hand-over). When the
    /// revs drop without a shift it fades where it stands.
    fn draw_tile(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, w: [f32; 3]) {
        let t = &self.tile;
        if !t.shown() {
            return;
        }
        let hand = t.fold.map_or(0.0, |f| {
            let f = f.clamp(0.0, 1.0);
            f * f * (3.0 - 2.0 * f)
        });
        let opacity = t.level.clamp(0.0, 1.0) * (1.0 - hand) * (1.0 - hand);
        if opacity <= 0.004 {
            return;
        }
        let pop = 1.0 + POP * finite(t.pop.value()).clamp(0.0, 1.0);
        p.scoped(|p| {
            p.set_opacity(opacity);
            p.translate(g.sug_c + vec2(-HAND_SLIDE * hand, 0.0));
            p.scale(pop * (1.0 - HAND_SHRINK * hand));
            p.translate(-g.sug_c);
            p.layer(LayerOptions::bloom(), |p| {
                if !self.sprites.draw_tile(p, g, w) {
                    hardware::fallback_tile(p, g, pal);
                }
                if t.flash {
                    p.fill(
                        &Shape::round_rect(g.sug_face_rect(), g.sug_r),
                        pal.roles.ink.with_alpha(FLASH_ALPHA),
                    );
                }
                let Some(d) = t.digit else { return };
                let [apex, drop, half, stroke] = g.sug_chevron;
                let x = g.sug_c.x;
                let arms = [vec2(x - half, apex + drop), vec2(x, apex), vec2(x + half, apex + drop)];
                p.fill(
                    &Shape::Polyline {
                        points: &arms,
                        width: stroke,
                        closed: false,
                    },
                    pal.roles.ink,
                );
                // A proportional run, set smaller when "10" would overflow the face.
                let label = Gear::Forward(d).label();
                let mut style = TextStyle::new(self.faces.gear, g.sug_cap)
                    .paint(pal.roles.ink)
                    .align(Align::Center, VAlign::CapMiddle);
                let width = p.measure(label, &style).width;
                let room = g.sug_face.x - 2.0 * TILE_MARGIN;
                if width > room && width.is_finite() {
                    style.cap = g.sug_cap * room / width;
                }
                p.text(vec2(x, g.sug_digit_y), label, &style);
            });
        });
    }

    /// The car's logo, or the custom name, at the band's right end (static, so it is dimmed),
    /// with a keyline like the glyphs.
    fn draw_logo(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let zone = rect(g.logo);
        // A long wordmark in the wide face would shrink below the label floor: set it in the
        // label face at the floor instead (the logo then fits it to the zone as before).
        let (mut font, mut cap, mut tracking) = (self.faces.wide, LOGO_CAP, LOGO_TRACKING);
        if self.logo.image.is_none() {
            let mut buf = [0u8; 64];
            let name = logo::wordmark(&self.logo, &mut buf);
            let inner = zone.width() - 2.0 * zone.width().min(zone.height()) * logo::SAFE_PADDING;
            let wide = TextStyle::new(font, cap).tracking(tracking);
            let w = p.measure(name, &wide).width;
            if w.is_finite() && w > 0.0 && cap * inner / w < LOGO_MIN_CAP {
                (font, cap, tracking) = (self.faces.label, LOGO_MIN_CAP, LABEL_TRACKING);
            }
        }
        let style = |c: Color| LogoStyle {
            tint: c,
            text: Paint::Solid(c),
            font,
            cap,
            tracking,
        };
        let key = style(pal.roles.oled_black.with_alpha(KEYLINE_ALPHA));
        let r = keyline_width(cap);
        for i in 0..8 {
            logo::draw(p, zone.translate(keyline_offset(i, 8, r)), &self.logo, &key);
        }
        logo::draw(p, zone, &self.logo, &style(static_ink(pal.roles.ink)));
    }
}

/// `v`, or 0 when it is not a number.
fn finite(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

/// The first LED of the alert zone: the LED the run's head is on when the revs reach the alert
/// `stage` of the shift point.
fn alert_zone(stage: f32, shift_rpm: f32, max_rpm: f32, ticks: usize) -> usize {
    let onset = stage * finite(shift_rpm) / finite(max_rpm).max(1.0);
    let lit = (onset * ticks as f32 + 0.5).floor();
    if lit.is_finite() && lit >= 1.0 {
        (lit as usize - 1).min(ticks)
    } else {
        0
    }
}

/// The steering dot's x for `steer` (−1 full left .. 1 full right).
fn dot_x(g: &Geom, steer: f32) -> f32 {
    finite(steer).clamp(-1.0, 1.0) * g.dot_travel
}

/// `rpm` as a share of the limiter (0..=1).
fn fraction(rpm: f32, max_rpm: f32) -> f32 {
    let f = rpm / max_rpm.max(1.0);
    if f.is_finite() {
        f.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// `held` moved just far enough to stay within `band / 2` of `raw`.
fn backlash(held: f32, raw: f32, band: f32) -> f32 {
    if !raw.is_finite() {
        return finite(held);
    }
    if !held.is_finite() {
        return raw;
    }
    held.clamp(raw - band * 0.5, raw + band * 0.5)
}

/// The load on tyre `i` (0..=1, a half at rest) from the felt force of the smoothed G.
fn tyre_load(g: &Geom, i: usize, sig: &Signals) -> f32 {
    let felt = felt_force(sig.g_lat, sig.g_long);
    let pill = g.pills[i.min(3)];
    let side = if pill.x < g.car_c.x { -1.0 } else { 1.0 };
    let end = if pill.y < g.car_c.y { -1.0 } else { 1.0 };
    let load = 0.5 + LOAD_PER_G * (felt.x * side + felt.y * end);
    if load.is_finite() {
        load.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

/// `ink` dimmed so that it shows at [`STATIC_LUMA`] on black: by its light, keeping its coverage
/// (the overlay blends coverage in encoded space, so a faded ink shows far darker than its
/// value; `docs/kit/README.md` §13).
fn static_ink(ink: Color) -> Color {
    let seen = ink.luminance() * ink.a.clamp(0.0, 1.0);
    if seen > STATIC_LUMA {
        ink.intensity(STATIC_LUMA / seen)
    } else {
        ink
    }
}

/// How far right a tabular run in `style` must move so its digits sit centred on their cells:
/// the kit centres each digit in a 0.60 em cell and adds the tracking after it.
fn tabular_bias(p: &Painter<'_>, style: &TextStyle) -> f32 {
    let em = style.cap / p.fonts().cap_ratio(style.font).max(0.1);
    0.5 * style.tracking * em
}

/// The trip's whole units and its tenths digit for `units` driven, held at the tile's limit.
fn trip_parts(units: f64) -> (i64, i64) {
    let tenths = (units * 10.0).floor();
    let tenths = if tenths.is_finite() {
        (tenths as i64).clamp(0, TRIP_MAX_TENTHS)
    } else {
        0
    };
    (tenths / 10, tenths % 10)
}

/// The keyline's width for a cap height.
fn keyline_width(cap: f32) -> f32 {
    (cap * KEYLINE_PER_CAP).clamp(KEYLINE_MIN, KEYLINE_MAX)
}

/// Offset `i` of a keyline ring's `n` copies.
fn keyline_offset(i: usize, n: usize, r: f32) -> Vec2 {
    Vec2::polar(Vec2::ZERO, r, i as f32 * TAU / n.max(1) as f32)
}

/// Hood keyline: black copies of a text run in `rings` round `pos`, so it reads over a bright
/// road or sky. The width follows the cap height.
fn keyline_text(
    p: &mut Painter<'_>,
    pos: Vec2,
    text: &str,
    style: &TextStyle,
    black: Color,
    rings: &[(usize, f32, f32)],
) {
    let width = keyline_width(style.cap);
    for &(n, k, alpha) in rings {
        let key = style.paint(black.with_alpha(alpha));
        for i in 0..n {
            p.text(pos + keyline_offset(i, n, width * k), text, &key);
        }
    }
}

/// [`keyline_text`] for a tabular number.
fn keyline_number(
    p: &mut Painter<'_>,
    pos: Vec2,
    value: i64,
    cells: u8,
    style: &TextStyle,
    black: Color,
    rings: &[(usize, f32, f32)],
) {
    let width = keyline_width(style.cap);
    for &(n, k, alpha) in rings {
        let key = style.paint(black.with_alpha(alpha));
        for i in 0..n {
            p.number(pos + keyline_offset(i, n, width * k), value, cells, &key, None);
        }
    }
}

/// Builds the cluster (the registry's constructor).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(Gt7HoodPro::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signals(rpm: f32) -> Signals {
        Signals {
            rpm,
            max_rpm: 8700.0,
            idle_rpm: 900.0,
            shift_rpm: 8439.0,
            shift_fraction: rpm / 8439.0,
            rpm_fraction: rpm / 8700.0,
            gear: Gear::Forward(3),
            ..Signals::default()
        }
    }

    fn palette() -> Palette {
        Palette::family(&Tokens::default(), "modernDigital")
    }

    #[test]
    fn the_led_run_follows_the_revs_and_holds_still_on_the_limiter() {
        let pal = palette();
        let mut rev = RevBar::default();
        rev.update(&signals(900.0), &pal, 95, 1.0 / 60.0, true);
        assert_eq!(rev.count(95), 10, "idle lights about a tenth of the bar");
        // Same-frame attack, drawn half the backlash behind the revs.
        rev.update(&signals(5200.0), &pal, 95, 1.0 / 60.0, false);
        let pulled = ((5200.0 - REV_BAND * 0.5) / 8700.0 * 95.0 + 0.5).floor() as usize;
        assert_eq!(rev.count(95), pulled);
        // The engine bounces 50 rpm on the limiter: the head LED stays where it is.
        rev.update(&signals(8690.0), &pal, 95, 1.0 / 60.0, false);
        let held = rev.count(95);
        for k in 0..12 {
            let rpm = if k % 2 == 0 { 8640.0 } else { 8690.0 };
            rev.update(&signals(rpm), &pal, 95, 1.0 / 60.0, false);
            assert_eq!(rev.count(95), held, "frame {k}");
        }
        // A lift releases within 50 ms (two time constants: seven eighths of the drop).
        let from = rev.lit;
        for _ in 0..3 {
            rev.update(&signals(2000.0), &pal, 95, 1.0 / 60.0, false);
        }
        assert!(
            rev.lit - rev.target <= 0.14 * (from - rev.target),
            "{} of {from}",
            rev.lit
        );
    }

    #[test]
    fn the_alert_colours_the_whole_run_and_the_strobe_starts_lit() {
        let pal = palette();
        let mut rev = RevBar::default();
        rev.update(&signals(6000.0), &pal, 95, 1.0 / 60.0, true);
        let (body, head) = rev.colors(&pal, false);
        assert_eq!(head, pal.roles.ink);
        assert_eq!(body, pal.roles.ink.intensity(TICK_BODY));
        // The warning stage: the whole run takes the shift colour at once, steady.
        rev.update(&signals(8000.0), &pal, 95, 1.0 / 60.0, false);
        assert!(rev.alert && !rev.over);
        assert_eq!(rev.colors(&pal, false).1, pal.shift);
        // The shift point: the strobe starts on its lit phase, three frames on, three off at 60 fps.
        let mut lit = Vec::new();
        for _ in 0..12 {
            rev.update(&signals(8690.0), &pal, 95, 1.0 / 60.0, false);
            lit.push(rev.colors(&pal, false).1 == pal.shift);
        }
        assert_eq!(
            lit,
            [true, true, true, false, false, false, true, true, true, false, false, false]
        );
        // The dark phase keeps a fifth of the light, so a still never reads as idle.
        rev.since = Some(0.06);
        assert_eq!(rev.colors(&pal, false).1, pal.shift.intensity(STROBE_FLOOR));
        // Reduced flash: a breath, never below the floor.
        assert!(rev.colors(&pal, true).1.luminance() >= pal.shift.intensity(STROBE_FLOOR).luminance() * 0.99);
    }

    #[test]
    fn the_tile_names_the_next_gear_at_the_shift_point_only() {
        let t = Tokens::default();
        let mut tile = Tile::new(&t);
        tile.update(Gear::Forward(3), false, false, 1.0 / 60.0);
        assert!(!tile.shown());
        tile.update(Gear::Forward(3), false, true, 1.0 / 60.0);
        assert_eq!(tile.digit, Some(4));
        assert!(tile.shown() && tile.level == 1.0, "no delay at the shift point");
        // Neutral, reverse and the top gear name nothing.
        for gear in [Gear::Neutral, Gear::Reverse, Gear::Forward(TOP_GEAR)] {
            let mut other = Tile::new(&t);
            other.update(gear, false, true, 1.0 / 60.0);
            assert_eq!(other.digit, None, "{gear:?}");
        }
        // A lift without a shift: gone within 50 ms.
        for _ in 0..3 {
            tile.update(Gear::Forward(3), false, false, 1.0 / 60.0);
        }
        assert!(!tile.shown(), "level {}", tile.level);
    }

    #[test]
    fn taking_the_suggested_gear_folds_the_tile_and_keeps_it_away() {
        let t = Tokens::default();
        let mut tile = Tile::new(&t);
        tile.update(Gear::Forward(3), false, true, 1.0 / 60.0);
        // The upshift lands while the revs are still past the shift point (the review sequence's
        // frame 42): the tile flashes, folds and does not come back naming 5.
        tile.update(Gear::Forward(4), true, true, 1.0 / 60.0);
        assert!(tile.flash && tile.fold == Some(0.0));
        assert_eq!(tile.digit, Some(4), "the tile keeps its digit through the hand-over");
        let mut frames = 1;
        while tile.shown() {
            assert_eq!(tile.digit, Some(4), "frame {frames}: never a blank tile");
            tile.update(Gear::Forward(4), false, true, 1.0 / 60.0);
            assert!(!tile.flash, "frame {frames}");
            frames += 1;
            assert!(frames < 12, "the hand-over ends");
        }
        assert!(tile.digit.is_none());
        assert!((6..=8).contains(&frames), "hands over in about 100 ms: {frames} frames");
        // Once the revs have dropped below the shift point it may show again.
        tile.update(Gear::Forward(4), false, false, 1.0 / 60.0);
        tile.update(Gear::Forward(4), false, true, 1.0 / 60.0);
        assert_eq!(tile.digit, Some(5));
    }

    #[test]
    fn tyre_load_moves_out_of_the_turn_and_forward_under_braking() {
        let g = &HOOD;
        let rest = Signals::default();
        for i in 0..4 {
            assert!((tyre_load(g, i, &rest) - 0.5).abs() < 1e-6);
        }
        // Turning right (acceleration to the right): the left tyres take the load.
        let right = Signals {
            g_lat: 1.0,
            ..Signals::default()
        };
        assert!(tyre_load(g, 0, &right) > 0.7 && tyre_load(g, 1, &right) < 0.3);
        // Braking: the front tyres take it.
        let braking = Signals {
            g_long: -1.0,
            ..Signals::default()
        };
        assert!(tyre_load(g, 0, &braking) > 0.7 && tyre_load(g, 2, &braking) < 0.3);
        let wild = Signals {
            g_lat: f32::NAN,
            g_long: f32::INFINITY,
            ..Signals::default()
        };
        assert!(tyre_load(g, 3, &wild).is_finite());
    }

    #[test]
    fn the_shift_point_holds_through_a_dip_and_an_unknown_limiter_stays_dark() {
        let pal = palette();
        let mut rev = RevBar::default();
        rev.update(&signals(8000.0), &pal, 95, 1.0 / 60.0, true);
        rev.update(&signals(8500.0), &pal, 95, 1.0 / 60.0, false);
        assert!(rev.over && rev.since == Some(0.0));
        // The revs hover 60 rpm under the shift point: the strobe's clock keeps running.
        for _ in 0..4 {
            rev.update(&signals(8380.0), &pal, 95, 1.0 / 60.0, false);
            assert!(rev.over);
        }
        assert!(rev.since.is_some_and(|t| t > 0.06));
        // Well under it, the shift point and then the alert let go.
        rev.update(&signals(8200.0), &pal, 95, 1.0 / 60.0, false);
        assert!(!rev.over && rev.alert);
        rev.update(&signals(6000.0), &pal, 95, 1.0 / 60.0, false);
        assert!(!rev.alert);
        // A zero packet's limiter (1 rpm): no run, no alert, no strobe.
        let zero = Signals {
            rpm: 1.1,
            max_rpm: 1.0,
            shift_rpm: 0.97,
            shift_fraction: 1.1,
            ..Signals::default()
        };
        let mut dark = RevBar::default();
        dark.update(&zero, &pal, 95, 1.0 / 60.0, true);
        dark.update(&zero, &pal, 95, 1.0 / 60.0, false);
        assert!(!dark.over && !dark.alert && dark.count(95) == 0 && dark.since.is_none());
    }

    #[test]
    fn the_alert_zone_follows_the_shift_point() {
        let pal = palette();
        let t = Tokens::default();
        // The tokens' shift point (0.97 of the limiter): the zone starts at LED 85.
        let shift = 8700.0 * t.rev_stages.shift_pct;
        assert_eq!(alert_zone(pal.stages[2], shift, 8700.0, 95), 85);
        // A driver who shifts earlier moves it left; an unknown limiter has none.
        assert!(alert_zone(pal.stages[2], 7800.0, 8700.0, 95) < 85);
        assert_eq!(alert_zone(pal.stages[2], f32::NAN, 8700.0, 95), 0);
        let mut rev = RevBar::default();
        rev.update(&signals(3000.0), &pal, 95, 1.0 / 60.0, true);
        assert!(rev.zone.is_some());
        let zero = Signals {
            max_rpm: 1.0,
            ..Signals::default()
        };
        rev.update(&zero, &pal, 95, 1.0 / 60.0, false);
        assert_eq!(rev.zone, None);
        // The zone is a static part: under the cap.
        assert!(pal.roles.warning.intensity(ZONE_LIGHT).luminance() < STATIC_LUMA);
    }

    #[test]
    fn the_steering_dot_spans_its_travel() {
        let g = &HOOD;
        assert_eq!(dot_x(g, 1.0), g.dot_travel);
        assert_eq!(dot_x(g, -128.0 / STEER_FULL), -g.dot_travel);
        assert_eq!(dot_x(g, 0.0), 0.0);
        assert_eq!(dot_x(g, f32::NAN), 0.0);
    }

    #[test]
    fn the_side_groups_burn_below_the_lit_run() {
        for k in [PEDAL_LIGHT, BOOST_LIGHT, TRIP_LIGHT, PILL_LIGHT[1]] {
            assert!(k < TICK_BODY, "{k}");
        }
    }

    #[test]
    fn the_trip_counts_tenths_and_holds_at_its_limit() {
        assert_eq!(trip_parts(0.0), (0, 0));
        assert_eq!(trip_parts(7.79), (7, 7));
        assert_eq!(trip_parts(123.45), (123, 4));
        assert_eq!(trip_parts(-5.0), (0, 0));
        assert_eq!(trip_parts(1e12), (999, 9));
        assert_eq!(trip_parts(f64::NAN), (0, 0));
    }

    #[test]
    fn the_alert_run_is_brighter_than_the_white_run() {
        let pal = palette();
        let mut rev = RevBar::default();
        rev.update(&signals(6000.0), &pal, 95, 1.0 / 60.0, true);
        assert_eq!(rev.core(&pal, false), None, "no core below the alert");
        rev.update(&signals(8000.0), &pal, 95, 1.0 / 60.0, false);
        let core = rev.core(&pal, false).expect("the alert run has a hot core");
        assert!(core.luminance() > pal.roles.ink.intensity(TICK_BODY).luminance());
        // On the strobe the core follows the run into its dark phase.
        rev.update(&signals(8690.0), &pal, 95, 1.0 / 60.0, false);
        rev.since = Some(0.06);
        let dark = rev.core(&pal, false).expect("a core on the strobe");
        assert!(dark.luminance() < core.luminance() * 0.3);
    }

    #[test]
    fn backlash_and_fraction_survive_bad_numbers() {
        assert_eq!(backlash(100.0, f32::NAN, 60.0), 100.0);
        assert_eq!(backlash(f32::NAN, 500.0, 60.0), 500.0);
        assert_eq!(backlash(100.0, 200.0, 60.0), 170.0);
        assert_eq!(fraction(5000.0, 0.0), 1.0);
        assert_eq!(fraction(f32::NAN, 8700.0), 0.0);
        assert_eq!(static_ink(Color::BLACK), Color::BLACK);
        assert!(static_ink(Color::WHITE).luminance() <= STATIC_LUMA + 1e-4);
    }
}
