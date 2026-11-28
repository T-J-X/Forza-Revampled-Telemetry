//! Tesla ADAS (`ClusterTheme::TeslaAdas`), catalog row 15: the ADAS & HUD family's pattern-setter
//! (cockpit + hood, build #5).
//!
//! It recreates the Model S and Model X (2021+) driver display: Tesla's driving visualisation in
//! the centre, the Track Mode G-meter card on the left, and the gear and speed stacked on the
//! right, frameless on true black. On TJX's direction it is Japan-inspired, as a Japan-market car
//! would show it on the Bayshore Route at night: Japanese road paint, a kei box in the traffic,
//! and labels set like Japanese guide signs. `RESEARCH.md` holds the sources and every deliberate
//! change.
//!
//! Zone table (local 4K px around the view's anchor; [`COCKPIT`] / [`HOOD`]):
//!
//! | Rank | Element | Cockpit | Hood |
//! |---|---|---|---|
//! | 1 | shift line: 16 pills, outside-in onto the centre pair | 52 × 12, pitch 60, y −300 | 62 × 10, pitch 70, y −204 |
//! | 1 | gear (Noto Sans 600) | cap 160 at (490, −170) | cap 150 at (470, −110) |
//! | 2 | speed (Noto Sans 500), unit under it | cap 96 at (490, 14) | cap 84 at (470, 46) |
//! | 3 | power line: centre-zero, power to the right, negative power to the left in green | ±216 at y 226 | ±200 at y 170 |
//! | 3 | rpm and power figures under their two-language labels | x −560 and −420, y 164 to 214 | x −530 and −410, y 114 to 158 |
//! | 4 | road scene ([`scene`]) | 720 × 492 from (−360, −252) | 660 × 362 from (−330, −182) |
//! | 4 | G-meter with its peak history | r 96 at (−490, −22) | r 70 at (−470, −26) |
//! | 5 | logo zone, card label, R · N · D row | left column top; right column foot | the same |
//!
//! The shift line is the only part that strobes (10 Hz at the shift point, 12 px tall). Below
//! the first shift stage its pills fill dimly with the revs; the stage colours then replace them
//! from the outside in. The glow under the ego car follows the same stages and holds the shift
//! colour, steady, at the shift point.
//!
//! Telemetry (verified fields only): rpm, idle and limiter → shift line, rev fill, rpm figure;
//! gear → gear and the R · N · D row; speed → figure, road scroll, parked view; brake and
//! longitudinal acceleration → brake light, slow-down dashes, stop bar and 止まれ; lateral
//! acceleration → the path's bend and the G-meter; velocity (body slip) → the ego car's yaw;
//! power → power line and figure. The traffic is a seeded ambient simulation.
mod scene;

use crate::common::gball::{GBall, RINGS_G};
use crate::common::glass_plate::legibility_halo;
use crate::common::hero_gear::HeroGear;
use crate::common::label::{logo_zone, LogoVariant};
use crate::common::shift::{draw_pod, LimiterCue, PodStyle};
use crate::common::speed::TabularSpeed;
use crate::common::ShiftLights;
use fh6_kit::cluster::Setup;
use fh6_kit::math::{finite_or, smoothstep};
use fh6_kit::{
    vec2, Align, Cap, Cluster, FontId, FontRole, FrameInput, Gear, Hysteresis, Layout, Logo, Painter, Palette,
    PowerUnit, Rect, Shape, Spring, TextStyle, Tokens, VAlign, Vec2, View,
};
use scene::{Scene, SceneGeom, SceneInput};
use std::f32::consts::TAU;

/// Standard gravity (m/s²).
const G: f32 = 9.80665;
/// Pills in the shift line.
const SEGMENTS: u32 = 16;
/// How bright a pill is while it only shows the revs (below the first shift stage): an opaque
/// grey of this share of the ink's light, so it reads the same over the scrim and on the hood
/// and stays 3:1 against black.
const REV_LEVEL: f32 = 0.24;
/// The power line reads full at the engine's highest power seen so far, and never at less than
/// this (W), so a small engine's first metres do not fill the line.
const POWER_FLOOR_W: f32 = 110_000.0;
/// The Japanese label face (`asset-forge` set `tesla_adas`, job `tesla_adas/font`).
const JP_FAMILY: &str = "Noto Sans JP";
const JP_FONT: &str = "clusters/tesla_adas/NotoSansJP-Medium-subset.ttf";
/// Every Japanese character the dash draws, rasterised when it is built.
const JP_TEXT: &str = "トラックモード回転数出力止まれ";
/// Label sizes (cap heights): the Japanese line, the Latin line under it (the tokens' floor).
const JP_CAP: f32 = 16.0;
const LATIN_CAP: f32 = 12.0;
/// Directions the G-meter's peak history keeps.
const PEAK_BINS: usize = 24;

/// Geometry of one view (local 4K px around the anchor).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
    shift_y: f32,
    shift_pitch: f32,
    pill: Vec2,
    scene: SceneGeom,
    power_y: f32,
    power_half: f32,
    power_px: f32,
    gear: Vec2,
    gear_cap: f32,
    speed: Vec2,
    speed_cap: f32,
    /// The unit label's cap top and cap height.
    unit_y: f32,
    unit_cap: f32,
    /// The N of the R · N · D row, the letters' spacing and cap height.
    prnd: Vec2,
    prnd_pitch: f32,
    prnd_cap: f32,
    logo: Rect,
    /// The Track Mode label's Japanese line, and the drop to its Latin line.
    card: Vec2,
    card_stack: f32,
    meter: Vec2,
    meter_r: f32,
    /// The two status cells' centres, and the y of their Japanese, Latin and value lines.
    status_x: [f32; 2],
    status_y: [f32; 3],
    status_cap: f32,
    /// Legibility halos (the hood view has no scrim).
    halos: [Option<Rect>; 4],
}

impl Geom {
    /// Pill `i`'s rectangle.
    fn pill_rect(&self, i: u32) -> Rect {
        let x = (i as f32 - (SEGMENTS - 1) as f32 * 0.5) * self.shift_pitch;
        Rect::from_center_size(vec2(x, self.shift_y), self.pill)
    }
}

const COCKPIT: Geom = Geom {
    shift_y: -300.0,
    shift_pitch: 60.0,
    pill: vec2(52.0, 12.0),
    scene: SceneGeom {
        viewport: Rect {
            min: vec2(-360.0, -252.0),
            max: vec2(360.0, 240.0),
        },
        focal: 700.0,
        horizon_y: -226.0,
        surface: 0.022,
        parked: Rect {
            min: vec2(-290.0, -160.0),
            max: vec2(290.0, 170.0),
        },
    },
    power_y: 226.0,
    power_half: 216.0,
    power_px: 4.0,
    gear: vec2(490.0, -170.0),
    gear_cap: 160.0,
    speed: vec2(490.0, 14.0),
    speed_cap: 96.0,
    unit_y: 80.0,
    unit_cap: 16.0,
    prnd: vec2(490.0, 196.0),
    prnd_pitch: 50.0,
    prnd_cap: 22.0,
    logo: Rect {
        min: vec2(-600.0, -254.0),
        max: vec2(-380.0, -210.0),
    },
    card: vec2(-490.0, -174.0),
    card_stack: 24.0,
    meter: vec2(-490.0, -22.0),
    meter_r: 96.0,
    status_x: [-560.0, -420.0],
    status_y: [164.0, 187.0, 214.0],
    status_cap: 22.0,
    halos: [None; 4],
};

/// Hood: the scrim is 0%, so every group sits on a legibility halo.
const HOOD: Geom = Geom {
    shift_y: -204.0,
    shift_pitch: 70.0,
    pill: vec2(62.0, 10.0),
    scene: SceneGeom {
        viewport: Rect {
            min: vec2(-330.0, -182.0),
            max: vec2(330.0, 180.0),
        },
        focal: 520.0,
        horizon_y: -160.0,
        surface: 0.0,
        parked: Rect {
            min: vec2(-220.0, -110.0),
            max: vec2(220.0, 140.0),
        },
    },
    power_y: 170.0,
    power_half: 200.0,
    power_px: 3.0,
    gear: vec2(470.0, -110.0),
    gear_cap: 150.0,
    speed: vec2(470.0, 46.0),
    speed_cap: 84.0,
    unit_y: 100.0,
    unit_cap: 14.0,
    prnd: vec2(470.0, 152.0),
    prnd_pitch: 46.0,
    prnd_cap: 20.0,
    logo: Rect {
        min: vec2(-570.0, -192.0),
        max: vec2(-370.0, -152.0),
    },
    card: vec2(-470.0, -136.0),
    card_stack: 21.0,
    meter: vec2(-470.0, -26.0),
    meter_r: 70.0,
    status_x: [-530.0, -410.0],
    status_y: [114.0, 135.0, 161.0],
    status_cap: 24.0,
    halos: [
        Some(Rect {
            min: vec2(-560.0, -212.0),
            max: vec2(560.0, -196.0),
        }),
        Some(Rect {
            min: vec2(-610.0, -196.0),
            max: vec2(-340.0, 170.0),
        }),
        Some(Rect {
            min: vec2(-320.0, -176.0),
            max: vec2(320.0, 176.0),
        }),
        Some(Rect {
            min: vec2(350.0, -190.0),
            max: vec2(600.0, 166.0),
        }),
    ],
};

fn geom(view: View) -> &'static Geom {
    match view {
        View::Hood => &HOOD,
        View::Cockpit | View::Chase => &COCKPIT,
    }
}

/// The faces this cluster sets (resolved once; a user display font replaces the numerals).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    gear: FontId,
    figure: FontId,
    label: FontId,
    /// The Japanese label face, when its file is installed.
    kanji: Option<FontId>,
    wide: FontId,
    role_display: FontId,
}

/// Finds the Japanese face, loading it the first time (the cluster is rebuilt on every view
/// change and must not load a second copy).
fn japanese_face(setup: &mut Setup<'_>) -> Option<FontId> {
    let font = match setup.fonts.find(JP_FAMILY, 500, false) {
        Some(font) => font,
        None => {
            let bytes = std::fs::read(setup.assets?.join(JP_FONT)).ok()?;
            setup.fonts.load(bytes, None).ok()?
        }
    };
    setup.fonts.preload_text(font, JP_TEXT);
    Some(font)
}

/// The cells a figure needs, so a centred figure sits on its label's axis whatever its length.
fn cells_of(value: i64) -> u8 {
    match value.unsigned_abs() {
        0..=9 => 1,
        10..=99 => 2,
        100..=999 => 3,
        1000..=9999 => 4,
        _ => 5,
    }
}

/// The lighting order of pill `i`: 0 for the outermost pair, 7 for the centre pair.
fn pair_of(i: u32) -> f32 {
    let half = (SEGMENTS - 1) as f32 * 0.5;
    half - (i as f32 - half).abs()
}

/// How much of pill `i` the rev fill lights for `progress` (0 at idle, 1 at the first shift
/// stage): the pairs fill one after another from the outside in.
fn rev_fill(i: u32, progress: f32) -> f32 {
    let pairs = (SEGMENTS / 2) as f32;
    let j = pair_of(i);
    smoothstep(j / pairs, (j + 1.0) / pairs, progress)
}

/// The revs' share of the way from idle to the first shift stage.
fn rev_progress(rpm: f32, idle: f32, first_stage_rpm: f32) -> f32 {
    let span = first_stage_rpm - idle;
    if span.is_finite() && span >= 1.0 && rpm.is_finite() {
        ((rpm - idle) / span).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// The G-meter's history: the largest felt force seen in each direction, as Tesla's Track Mode
/// shades it. It is kept until the engine changes.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PeakTrace {
    peak: [f32; PEAK_BINS],
}

impl PeakTrace {
    const fn new() -> Self {
        Self { peak: [0.0; PEAK_BINS] }
    }

    /// Records the felt force `g` (in g, screen axes). The neighbouring directions rise with it
    /// so one hard stop draws a lobe, not a spike.
    fn record(&mut self, g: Vec2) {
        let m = g.x.hypot(g.y).min(RINGS_G[2]);
        if !m.is_finite() || m < 0.02 {
            return;
        }
        let at = g.y.atan2(g.x).rem_euclid(TAU) / TAU * PEAK_BINS as f32;
        let bin = (at.round() as usize) % PEAK_BINS;
        for (step, share) in [(0usize, 1.0f32), (1, 0.86), (2, 0.6)] {
            for b in [(bin + step) % PEAK_BINS, (bin + PEAK_BINS - step) % PEAK_BINS] {
                self.peak[b] = self.peak[b].max(m * share);
            }
        }
    }

    /// The history's outline around `center` at `per_g` px per g. Returns false when it is empty.
    fn outline(&self, center: Vec2, per_g: f32, out: &mut [Vec2; PEAK_BINS]) -> bool {
        for (i, (slot, peak)) in out.iter_mut().zip(self.peak).enumerate() {
            let a = i as f32 / PEAK_BINS as f32 * TAU;
            *slot = center + vec2(a.cos(), a.sin()) * (peak * per_g);
        }
        self.peak.iter().any(|p| *p > 0.05)
    }
}

/// The Tesla ADAS cluster.
#[derive(Debug, Clone)]
pub struct TeslaAdas {
    faces: Faces,
    palette: Palette,
    view: View,
    logo: Logo,
    scene: Scene,
    shift: ShiftLights,
    cue: LimiterCue,
    gear: HeroGear,
    speed: TabularSpeed,
    ball: GBall,
    peaks: PeakTrace,
    /// The revs' share of the way to the first shift stage.
    rev: f32,
    /// Power as a share of the highest power seen (-1..1), and that highest power (W).
    power: Spring,
    power_target: f32,
    power_top: f32,
    power_shown: Hysteresis,
    power_value: i64,
    power_unit: &'static str,
    rpm: i64,
    /// The engine's limiter and idle rpm: a change means another car, so the peaks start again.
    engine: (f32, f32),
    unit: &'static str,
    tracking: f32,
    started: bool,
}

impl TeslaAdas {
    /// Builds the cluster: faces, the Japanese label face and the car sprites.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let t: &Tokens = setup.tokens;
        let display = setup.fonts.role(FontRole::Display);
        let text = setup.fonts.role(FontRole::Text);
        let kanji = japanese_face(setup);
        let faces = Faces {
            gear: setup.fonts.find("Noto Sans", 600, false).unwrap_or(display),
            figure: setup.fonts.find("Noto Sans", 500, false).unwrap_or(display),
            label: setup.fonts.find("Noto Sans", 500, false).unwrap_or(text),
            kanji,
            wide: setup.fonts.role(FontRole::Wide),
            role_display: display,
        };
        let sprites = scene::load_sprites(setup);
        Self {
            faces,
            palette: Palette::family(t, "adasHud"),
            view: View::Cockpit,
            logo: Logo::default(),
            scene: Scene::new(sprites),
            shift: ShiftLights::new(SEGMENTS),
            cue: LimiterCue::new(),
            gear: HeroGear::new(t),
            speed: TabularSpeed::new(),
            ball: GBall::new(t),
            peaks: PeakTrace::new(),
            rev: 0.0,
            power: Spring::new(t.springs.fast, 0.0),
            power_target: 0.0,
            power_top: POWER_FLOOR_W,
            power_shown: Hysteresis::new(0.0, 1.0),
            power_value: 0,
            power_unit: PowerUnit::Hp.label(),
            rpm: 0,
            engine: (0.0, 0.0),
            unit: "MPH",
            tracking: t.type_scale.label_tracking,
            started: false,
        }
    }
}

impl Cluster for TeslaAdas {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = finite_or(input.dt, 0.0).max(0.0);
        let sig = *input.signals;
        self.palette = *input.palette;
        self.view = input.view;
        if self.logo != *input.logo {
            self.logo.clone_from(input.logo);
        }
        if input.fonts.display != self.faces.role_display {
            self.faces.gear = input.fonts.display;
            self.faces.figure = input.fonts.display;
            self.faces.role_display = input.fonts.display;
        }

        self.shift.update(&sig, &self.palette, input.t, dt, input.reduced_flash);
        self.cue.update(self.shift.over_shift(), 0.25, dt);
        self.rev = rev_progress(sig.rpm, sig.idle_rpm, self.palette.stages[0] * sig.shift_rpm);
        self.rpm = if sig.rpm.is_finite() {
            ((sig.rpm / 50.0).round() * 50.0).clamp(0.0, 99_950.0) as i64
        } else {
            0
        };
        self.speed.update(sig.speed);
        self.unit = sig.unit.label();

        // Another engine: the power scale and the G history belong to the last car.
        let engine = (finite_or(sig.max_rpm, 0.0), finite_or(sig.idle_rpm, 0.0));
        if engine != self.engine {
            self.engine = engine;
            self.power_top = POWER_FLOOR_W;
            self.peaks = PeakTrace::new();
        }
        let watts = finite_or(input.telemetry.power_watts, 0.0);
        self.power_top = self.power_top.max(watts.abs());
        self.power_target = (watts / self.power_top).clamp(-1.0, 1.0);
        self.power_unit = sig.power_unit.label();
        let shown = match sig.power_unit {
            PowerUnit::Hp => watts / 745.7,
            PowerUnit::Kw => watts / 1000.0,
        };
        self.power_value = self.power_shown.update(shown.clamp(0.0, 9999.0));

        self.ball.update(&sig, dt);
        self.peaks.record(self.ball.ball());

        if self.started {
            self.gear.update(sig.gear, self.cue.held(), dt);
            self.power.update(self.power_target, dt);
        } else {
            // The cluster is rebuilt on every view change: start on the reading, not from zero.
            self.gear.snap(sig.gear);
            self.power.reset(self.power_target);
            self.started = true;
        }

        self.scene.update(
            &SceneInput {
                speed_ms: sig.speed_ms,
                a_lat: sig.g_lat * G,
                a_long: sig.g_long * G,
                slip_deg: sig.slip_deg,
                brake: sig.brake,
                shift_level: self.shift.level(),
                over_shift: self.shift.over_shift(),
            },
            dt,
        );
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = geom(self.view);
        let pal = self.palette;
        for halo in g.halos.into_iter().flatten() {
            legibility_halo(p, halo, 0.5, &pal);
        }
        // Rank 4 first: the scene sits behind everything else.
        self.scene.draw(p, &g.scene, &pal, self.faces.kanji);
        self.draw_power(p, g, &pal);
        self.draw_shift(p, g, &pal);
        self.draw_card(p, g, &pal);
        self.draw_status(p, g, &pal);
        self.draw_drive(p, g, &pal);
    }

    fn animating(&self) -> bool {
        self.shift.animating()
            || self.cue.animating()
            || self.gear.animating()
            || self.ball.animating()
            || self.scene.animating()
            || !self.power.settled(self.power_target, 1e-3)
    }
}

impl TeslaAdas {
    /// The shift line. Unlit pills carry the rev fill; the shared `ShiftLights` decides the
    /// stage colours and the strobe, and while it strobes the dark phase is dark.
    fn draw_shift(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let style = PodStyle {
            glow_sigma: 5.0,
            glow: 0.45,
            unlit_alpha: 10.0 / 255.0,
            unlit_rim: 0.05,
            ..PodStyle::default()
        };
        for i in 0..SEGMENTS {
            let rect = g.pill_rect(i);
            let color = self.shift.color_of(i as usize, pal);
            draw_pod(p, rect, color, &style, pal);
            if color.is_none() && !self.shift.over_shift() {
                let fill = rev_fill(i, self.rev);
                if fill > 0.01 {
                    p.fill(
                        &Shape::round_rect(rect, rect.height() * 0.5),
                        pal.roles.ink.intensity(REV_LEVEL).fade(fill),
                    );
                }
            }
        }
    }

    /// Tesla's power meter: a hairline with a centre mark, filled to the right with power and
    /// to the left, in green, with negative power.
    fn draw_power(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let (y, half, w) = (g.power_y, g.power_half, g.power_px);
        p.fill(&Shape::line(vec2(-half, y), vec2(half, y), 1.0), roles.hairline);
        p.fill(
            &Shape::line(vec2(0.0, y - w * 1.5), vec2(0.0, y + w * 1.5), 2.0),
            roles.ink_label,
        );
        let v = self.power.value().clamp(-1.0, 1.0);
        if v.abs() > 0.004 {
            let color = if v > 0.0 { roles.ink } else { roles.go };
            p.fill(
                &Shape::Line {
                    a: vec2(0.0, y),
                    b: vec2(half * v, y),
                    width_a: w,
                    width_b: w,
                    cap: Cap::Round,
                },
                color,
            );
        }
    }

    /// A two-language label, set like a Japanese guide sign. Without the Japanese face it is
    /// the Latin line alone, at the Japanese line's size.
    fn sign(&self, p: &mut Painter<'_>, pos: Vec2, japanese: &str, latin: &str, stack: f32, pal: &Palette) {
        let ink = pal.roles.ink_label;
        let latin_style = |cap: f32| TextStyle::new(self.faces.label, cap).paint(ink).tracking(self.tracking);
        let Some(kanji) = self.faces.kanji else {
            p.text(pos, latin, &latin_style(JP_CAP).align(Align::Center, VAlign::CapMiddle));
            return;
        };
        let jp_style = TextStyle::new(kanji, JP_CAP).paint(ink).tracking(0.06);
        p.text(pos, japanese, &jp_style.align(Align::Center, VAlign::CapMiddle));
        p.text(
            pos + vec2(0.0, stack),
            latin,
            &latin_style(LATIN_CAP).align(Align::Center, VAlign::CapMiddle),
        );
    }

    /// The left card: the logo zone, the Track Mode label and the G-meter over its peak history.
    fn draw_card(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        logo_zone(p, g.logo, &self.logo, self.faces.wide, LogoVariant::Full, pal);
        self.sign(p, g.card, "トラックモード", "TRACK MODE", g.card_stack, pal);
        let mut outline = [Vec2::ZERO; PEAK_BINS];
        if self.peaks.outline(g.meter, g.meter_r / RINGS_G[2], &mut outline) {
            p.fill(&Shape::Polygon { points: &outline }, pal.primary.with_alpha(0.16));
        }
        self.ball.draw_meter(p, g.meter, g.meter_r, pal);
        let cap = p.tokens().type_scale.metric_secondary;
        let baseline = GBall::readout_baseline(g.meter, g.meter_r, cap);
        // The shared readout sets its value in the muted ink. Here it is a figure the driver
        // reads at a glance, so it gets the full ink.
        let mut figures = *pal;
        figures.roles.ink_muted = pal.roles.ink;
        self.ball
            .draw_readout(p, baseline, &figures, self.faces.figure, self.faces.label);
    }

    /// The status row: rpm and power, each under its two-language label.
    fn draw_status(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let value = TextStyle::new(self.faces.figure, g.status_cap)
            .paint(pal.roles.ink)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        let [y_label, y_latin, y_value] = g.status_y;
        let cells = [("回転数", "RPM", self.rpm), ("出力", self.power_unit, self.power_value)];
        for ((japanese, latin, number), x) in cells.into_iter().zip(g.status_x) {
            self.sign(p, vec2(x, y_label), japanese, latin, y_latin - y_label, pal);
            p.number(vec2(x, y_value), number, cells_of(number), &value, None);
        }
    }

    /// The right column: gear, speed with its unit, and Tesla's R · N · D row with the current
    /// letter lit.
    fn draw_drive(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        // Neutral is as bright as a gear: at a standstill the gear still leads the speed.
        let gear_ink = match self.gear.gear() {
            Gear::Neutral => roles.ink,
            _ => self.gear.color(pal),
        };
        self.gear.draw_with(p, g.gear, self.faces.gear, g.gear_cap, gear_ink);
        // The speed is centred under the gear, as the real display centres it, with no ghost 8s
        // (the real display has none). The unit is centred under it.
        let digits = TextStyle::new(self.faces.figure, g.speed_cap)
            .paint(roles.ink)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        let shown = self.speed.value();
        p.number(g.speed, shown, cells_of(shown), &digits, None);
        let unit = TextStyle::new(self.faces.label, g.unit_cap)
            .paint(roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapTop);
        p.text(vec2(g.speed.x, g.unit_y), self.unit, &unit);
        let current = match self.gear.gear() {
            Gear::Reverse => 0,
            Gear::Neutral => 1,
            Gear::Forward(_) => 2,
        };
        for (i, letter) in ["R", "N", "D"].into_iter().enumerate() {
            let ink = if i == current {
                roles.ink
            } else {
                roles.ink_label.fade(0.8)
            };
            let style = TextStyle::new(self.faces.label, g.prnd_cap)
                .paint(ink)
                .align(Align::Center, VAlign::CapMiddle);
            p.text(g.prnd + vec2((i as f32 - 1.0) * g.prnd_pitch, 0.0), letter, &style);
        }
    }
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(TeslaAdas::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_stays_inside_the_view_envelopes_and_meets_the_zone_minimums() {
        let t = Tokens::default();
        for (g, env) in [(&COCKPIT, t.layout.cockpit_envelope), (&HOOD, t.layout.hood_envelope)] {
            let inside = |r: Rect| env.contains(r.min) && env.contains(r.max);
            assert!(inside(g.scene.viewport), "scene {:?} in {env:?}", g.scene.viewport);
            assert!(inside(g.logo), "logo zone");
            for i in 0..SEGMENTS {
                assert!(inside(g.pill_rect(i)), "pill {i}");
            }
            // Zone minimums (tokens): the strobing strip at most 42 px tall and at least 720 px
            // wide, the gear at least 150 px, the speed at least 84 px and within 460 px of it.
            let span = g.pill_rect(SEGMENTS - 1).max.x - g.pill_rect(0).min.x;
            assert!(g.pill.y <= 42.0 && span >= 720.0, "shift line {span} x {}", g.pill.y);
            assert!(g.gear_cap >= 150.0 && g.speed_cap >= 84.0);
            let apart = g.gear - g.speed;
            assert!(apart.x.hypot(apart.y) <= 460.0, "the speed sits by the gear");
            // The shift line clears the gear, the gear clears the speed, and the R N D row sits
            // under the unit, all inside the envelope.
            let gear_top = g.gear.y - g.gear_cap * 0.5;
            assert!(
                gear_top - (g.shift_y + g.pill.y * 0.5) >= 8.0,
                "gear under the shift line"
            );
            assert!((g.speed.y - g.speed_cap * 0.5) - (g.gear.y + g.gear_cap * 0.5) >= 30.0);
            assert!(g.unit_y >= g.speed.y + g.speed_cap * 0.5 + 6.0);
            assert!(g.prnd.y - g.prnd_cap * 0.5 >= g.unit_y + g.unit_cap + 12.0);
            assert!(g.prnd.y + g.prnd_cap * 0.5 <= env.max.y);
            // The columns stay clear of the scene's undimmed middle, and the power line lies
            // inside the band, under the ego car.
            let clear = g.scene.viewport.width() * 0.5 * 0.64;
            assert!(g.gear.x - g.gear_cap * 0.5 > clear);
            assert!(g.meter.x + g.meter_r * 1.18 < -clear);
            assert!(g.power_half < g.scene.viewport.width() * 0.5);
            assert!(g.power_y + g.power_px <= g.scene.viewport.max.y);
            // The left column, top to bottom: logo, label, meter (a ball at the limit reaches
            // 1.18 r), the G readout, then the status rows down to the envelope.
            assert!(g.card.y - JP_CAP * 0.5 >= g.logo.max.y + 8.0);
            let label_foot = g.card.y + g.card_stack + LATIN_CAP * 0.5;
            assert!(
                g.meter.y - g.meter_r * 1.06 >= label_foot + 8.0,
                "meter under the label"
            );
            let readout = GBall::readout_baseline(g.meter, g.meter_r, t.type_scale.metric_secondary).y;
            assert!(
                g.status_y[0] - JP_CAP * 0.5 >= readout + 8.0,
                "status under the readout"
            );
            assert!(g.status_y[2] + g.status_cap * 0.5 <= env.max.y);
            assert!(g.status_y[1] - g.status_y[0] >= (JP_CAP + LATIN_CAP) * 0.5 + 4.0);
            assert!(g.status_x[0] - 60.0 >= env.min.x);
        }
        assert!(LATIN_CAP >= t.type_scale.micro_label * 0.75, "no label under the floor");
    }

    #[test]
    fn the_hood_view_has_halos_and_the_cockpit_has_none() {
        assert!(COCKPIT.halos.iter().all(Option::is_none), "the scrim is dark");
        assert!(HOOD.halos.iter().all(Option::is_some), "the hood has no scrim");
    }

    #[test]
    fn the_rev_fill_runs_outside_in_and_is_dark_at_idle() {
        for i in 0..SEGMENTS {
            assert_eq!(rev_fill(i, 0.0), 0.0, "pill {i} at idle");
            assert_eq!(rev_fill(i, 1.0), 1.0, "pill {i} at the first stage");
            assert_eq!(rev_fill(i, 0.4), rev_fill(SEGMENTS - 1 - i, 0.4), "mirrored");
        }
        assert_eq!(pair_of(0), 0.0);
        assert_eq!(pair_of(7), 7.0);
        assert_eq!(pair_of(8), 7.0);
        assert!(rev_fill(0, 0.2) > rev_fill(3, 0.2), "the outer pair leads");
        assert_eq!(rev_fill(7, 0.5), 0.0, "the centre pair is last");
    }

    #[test]
    fn rev_progress_needs_a_real_shift_stage() {
        assert_eq!(rev_progress(900.0, 900.0, 6000.0), 0.0);
        assert_eq!(rev_progress(6000.0, 900.0, 6000.0), 1.0);
        assert_eq!(rev_progress(9000.0, 900.0, 6000.0), 1.0);
        assert_eq!(rev_progress(3000.0, 900.0, 0.0), 0.0, "no shift point: no fill");
        assert_eq!(rev_progress(f32::NAN, 900.0, 6000.0), 0.0);
    }

    #[test]
    fn the_peak_history_keeps_the_largest_force_per_direction() {
        let mut peaks = PeakTrace::new();
        let mut out = [Vec2::ZERO; PEAK_BINS];
        assert!(!peaks.outline(Vec2::ZERO, 100.0, &mut out), "empty at first");
        peaks.record(vec2(1.0, 0.0));
        peaks.record(vec2(0.4, 0.0));
        assert!(peaks.outline(Vec2::ZERO, 100.0, &mut out));
        assert!((out[0].x - 100.0).abs() < 1e-3, "1 g to the right: {:?}", out[0]);
        assert!(out[1].x > 60.0 && out[PEAK_BINS - 1].x > 60.0, "a lobe, not a spike");
        assert_eq!(out[PEAK_BINS / 2], Vec2::ZERO, "nothing to the left yet");
        peaks.record(vec2(f32::NAN, 0.0));
        peaks.record(vec2(9.0, 0.0));
        assert!(peaks.peak[0] <= RINGS_G[2], "held at the outer ring");
    }

    #[test]
    fn every_japanese_character_drawn_is_preloaded() {
        for text in ["トラックモード", "回転数", "出力", "止まれ"] {
            for ch in text.chars() {
                assert!(JP_TEXT.contains(ch), "{ch} is drawn but not preloaded");
            }
        }
    }
}
