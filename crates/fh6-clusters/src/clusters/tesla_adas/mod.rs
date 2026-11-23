//! Tesla ADAS (`ClusterTheme::TeslaAdas`), catalog row 15: the ADAS & HUD family's pattern-setter
//! (cockpit + hood, build #5).
//!
//! A clay-and-light diorama floating frameless in the column gap. Zone table (local 4K px around
//! the anchor; [`COCKPIT`] / [`HOOD`]):
//!
//! | Zone | Element | Cockpit | Hood |
//! |---|---|---|---|
//! | Z1 | 10 pill pods, outside-in onto the centre pair | 60 × 22 at y −296, pitch 82 | 52 × 18 at y −208, pitch 70 |
//! | Z3 | 6 px (hood 3 px) rev ribbon, primary → accent, 12 px white comet, redline share | ±640 at y −270 | ±760 at y −189 |
//! | Z1 | gear (Saira Condensed 800) in a 1 px vertical pill, n+1 above / n−1 below as ghosts | cap 170 at (0, −144), pill ±72 × −258..−30 | cap 96 at (0, −100), pill ±52 × −176..−24 |
//! | Z2 | speed (Saira Light, no ghost 8s), unit below | cap 112 at (−380, −150) | cap 84 at (−470, −100) |
//! | Z5 | rpm and the shy SLIP / LIM chips | (380, −178) cap 44, chips y −112 | (470, −122) cap 36, chips y −70 |
//! | Z4 | chase-cam road scene ([`scene`]): glTF clay cars, carpet, lanes, arcs | 1440 × 260 from (−720, −20), 8° pitch | 1760 × 180 from (−880, 0), 8° pitch |
//!
//! Signature moments:
//! - **Brake dive:** in the frame the brake passes 5% the ego twin's tail bar lights warning red,
//!   its nose dips by the measured deceleration (4°/g, clamped 4°, fast spring) and a 2 px
//!   caution stop line drops across the lanes at `v² / (2 · max(−a_long, 2 m/s²))`, with its
//!   distance in Titillium cap 16 (hidden beyond 120 m). The carpet beyond it falls to 35%.
//! - **Upshift:** one 250 ms accent wake climbs the carpet (alpha only, no bloom) while the gear
//!   pill rolls 24 px.
//! - **Limiter:** only the pod strip strobes (10 Hz); the underglow holds a steady limiter violet
//!   and the gear pulses at 2 Hz after 0.5 s.
//! - **Drift:** the ego yaws into the bend by the slip angle, the carpet and underglow turn amber
//!   and the SLIP chip lights.
//!
//! Telemetry (verified fields only): rpm, idle, limiter and the tokens' shift point → pods and
//! ribbon; gear → pill; speed → readout, dash scroll, carpet; brake and longitudinal acceleration
//! → lamps, dive, stop line; lateral acceleration → path curvature (`a_lat / v²`), roll and the
//! light; body slip (from velocity) → ego yaw, drift and SLIP; car ordinal → the factory palette
//! (brand-tinted ego). The traffic is a seeded ambient simulation with no numbers on it.
//!
//! Brief: `spec.md`; departures are in `HANDOVER.md` ("My calls").
mod scene;

use crate::common::adas_road::EYE;
use crate::common::glass_plate::legibility_halo;
use crate::common::hero_gear::HeroGear;
use crate::common::label::shy;
use crate::common::shift::{LimiterCue, PodShape, PodStyle};
use crate::common::speed::TabularSpeed;
use crate::common::ShiftLights;
use fh6_kit::cluster::Setup;
use fh6_kit::math::{smoothstep, vec3};
use fh6_kit::{
    vec2, Align, Cap, Cluster, Color, FontId, FontRole, FrameInput, Gear, Glow, Gradient, ImageId, ImagePaint,
    LayerOptions, Layout, Painter, Palette, Rect, Shape, Spring, Stroke, TextStyle, Tokens, VAlign, Vec2, View,
};
use scene::{Scene, SceneGeom, SceneInput};

/// Standard gravity (m/s^2).
const G: f32 = 9.80665;
/// LED cap atlas cell and the lens inside it (render px; the recipe's `CAP_CELL` / `CAP_LENS_PX`).
const CAP_CELL: [f32; 2] = [288.0, 128.0];
const CAP_LENS: [f32; 2] = [240.0, 88.0];
/// LED cap atlas cell and the lens inside it (render px; the recipe's `CAP_CELL` / `CAP_LENS_PX`).const CAP_CELL: [f32; 2] = [288.0, 128.0];const CAP_LENS: [f32; 2] = [240.0, 88.0];
/// The pod count (catalog brief).
const PODS: u32 = 10;
/// Width of the comet at the head of the rev ribbon (px).
const COMET_PX: f32 = 12.0;

/// Geometry of one view (local 4K px around the anchor).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
    pods_y: f32,
    pod_size: Vec2,
    pod_pitch: f32,
    ribbon_y: f32,
    ribbon_half: f32,
    ribbon_px: f32,
    gear: Vec2,
    gear_cap: f32,
    pill: Rect,
    /// Vertical distance from the gear's centre to each neighbour ghost.
    ghost_dy: f32,
    ghost_cap: f32,
    /// How far the pill's contents roll on a gear change.
    roll_px: f32,
    speed: Vec2,
    speed_cap: f32,
    unit_y: f32,
    rpm: Vec2,
    rpm_cap: f32,
    chips_y: f32,
    chip_size: Vec2,
    chip_x: [f32; 2],
    /// The hood's legibility halo behind the top zone (none in the cockpit: the scrim is dark).
    halo: Option<Rect>,
    scene: SceneGeom,
}

impl Geom {
    /// The pod centres, evenly spaced about x = 0.
    fn pod_centers(&self) -> [Vec2; PODS as usize] {
        let mut out = [Vec2::ZERO; PODS as usize];
        let first = -self.pod_pitch * (PODS - 1) as f32 * 0.5;
        for (i, c) in out.iter_mut().enumerate() {
            *c = vec2(first + self.pod_pitch * i as f32, self.pods_y);
        }
        out
    }
}

const COCKPIT: Geom = Geom {
    pods_y: -296.0,
    pod_size: vec2(60.0, 22.0),
    pod_pitch: 82.0,
    ribbon_y: -270.0,
    ribbon_half: 640.0,
    ribbon_px: 6.0,
    gear: vec2(0.0, -144.0),
    gear_cap: 170.0,
    pill: Rect {
        min: vec2(-72.0, -258.0),
        max: vec2(72.0, -30.0),
    },
    ghost_dy: 100.0,
    ghost_cap: 18.0,
    roll_px: 24.0,
    speed: vec2(-380.0, -150.0),
    speed_cap: 112.0,
    unit_y: -78.0,
    rpm: vec2(380.0, -178.0),
    rpm_cap: 44.0,
    chips_y: -112.0,
    chip_size: vec2(84.0, 28.0),
    chip_x: [334.0, 426.0],
    halo: None,
    scene: SceneGeom {
        viewport: Rect {
            min: vec2(-720.0, -20.0),
            max: vec2(720.0, 240.0),
        },
        eye: EYE,
        pitch_deg: 8.0,
        // About the tokens' 33° field (`components.adasRoad`), the horizon 22 px below the band's
        // top so the far road and the lead car sit clear of the gear pill.
        focal: 440.0,
        horizon_frac: 0.085,
        pool: 0,
    },
};

/// Hood [4]: the scrim is 0%, so the top zone sits on a legibility halo and the scene widens to
/// ±880 × 180 px.
const HOOD: Geom = Geom {
    pods_y: -208.0,
    pod_size: vec2(52.0, 18.0),
    pod_pitch: 70.0,
    ribbon_y: -189.0,
    ribbon_half: 760.0,
    ribbon_px: 3.0,
    gear: vec2(0.0, -100.0),
    gear_cap: 96.0,
    pill: Rect {
        min: vec2(-52.0, -176.0),
        max: vec2(52.0, -24.0),
    },
    ghost_dy: 64.0,
    ghost_cap: 12.0,
    roll_px: 16.0,
    speed: vec2(-470.0, -100.0),
    speed_cap: 84.0,
    unit_y: -44.0,
    rpm: vec2(470.0, -122.0),
    rpm_cap: 36.0,
    chips_y: -56.0,
    chip_size: vec2(72.0, 24.0),
    chip_x: [430.0, 510.0],
    halo: Some(Rect {
        min: vec2(-640.0, -216.0),
        max: vec2(640.0, -16.0),
    }),
    scene: SceneGeom {
        viewport: Rect {
            min: vec2(-880.0, 0.0),
            max: vec2(880.0, 180.0),
        },
        eye: vec3(0.0, 2.8, -6.5),
        pitch_deg: 8.0,
        focal: 400.0,
        horizon_frac: 0.02,
        pool: 1,
    },
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
    speed: FontId,
    metric: FontId,
    label: FontId,
    role_display: FontId,
}

/// The neighbour gears shown as ghosts above (n+1) and below (n−1) the current one.
fn neighbours(gear: Gear) -> (Option<Gear>, Option<Gear>) {
    match gear {
        Gear::Reverse => (Some(Gear::Neutral), None),
        Gear::Neutral => (Some(Gear::Forward(1)), Some(Gear::Reverse)),
        Gear::Forward(n) => {
            let up = (n < 10).then_some(Gear::Forward(n + 1));
            let down = if n <= 1 { Gear::Neutral } else { Gear::Forward(n - 1) };
            (up, Some(down))
        }
    }
}

/// The Tesla ADAS cluster.
#[derive(Debug, Clone)]
pub struct TeslaAdas {
    faces: Faces,
    palette: Palette,
    view: View,
    scene: Scene,
    shift: ShiftLights,
    cue: LimiterCue,
    gear: HeroGear,
    roll: Spring,
    rev: Spring,
    rev_target: f32,
    red_from: f32,
    caution_from: f32,
    /// A copy of the shared gear glyph's entry spring (same spec, same steps), so the 1.12 entry
    /// scale can be cancelled inside the pill.
    entry: Spring,
    speed: TabularSpeed,
    rpm: f32,
    unit: &'static str,
    micro_cap: f32,
    tracking: f32,
    /// The Blender LED caps (off, on; `assets/clusters/tesla_adas/led_caps.png`), or `None` for
    /// the shared vector pods.
    caps: Option<ImageId>,
}

impl TeslaAdas {
    /// Builds the cluster: faces and the four car meshes (Phase 6 glTF, or the kit's proxy).
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let t: &Tokens = setup.tokens;
        let display = setup.fonts.role(FontRole::Display);
        let text = setup.fonts.role(FontRole::Text);
        let faces = Faces {
            gear: setup.fonts.find("Saira Condensed", 800, false).unwrap_or(display),
            speed: setup.fonts.find("Saira", 300, false).unwrap_or(display),
            metric: setup.fonts.find("Saira", 400, false).unwrap_or(display),
            label: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            role_display: display,
        };
        let springs = t.springs;
        let micro_cap = t.type_scale.micro_label;
        let tracking = t.type_scale.label_tracking;
        let assets = scene::load_assets(setup);
        let caps = setup
            .assets
            .map(|a| a.join("clusters/tesla_adas/led_caps.png"))
            .and_then(|path| {
                path.is_file()
                    .then(|| setup.images.load_file(setup.device, setup.queue, &path, None).ok())
                    .flatten()
            });
        Self {
            faces,
            palette: Palette::family(t, "adasHud"),
            view: View::Cockpit,
            scene: Scene::new(t, assets),
            shift: ShiftLights::new(PODS),
            cue: LimiterCue::new(),
            gear: HeroGear::new(t),
            roll: Spring::new(springs.snap, 0.0),
            rev: Spring::new(springs.snap, 0.0),
            rev_target: 0.0,
            red_from: 1.0,
            caution_from: 1.0,
            entry: Spring::new(springs.fast, 0.0),
            speed: TabularSpeed::new(),
            rpm: 0.0,
            unit: "MPH",
            micro_cap,
            tracking,
            caps,
        }
    }

    /// Fraction of the ribbon for `rpm` (idle → limiter).
    fn rev_fraction(rpm: f32, idle: f32, max: f32) -> f32 {
        let span = (max - idle).max(1.0);
        ((rpm - idle) / span).clamp(0.0, 1.0)
    }
}

impl Cluster for TeslaAdas {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt.max(0.0);
        let sig = *input.signals;
        self.palette = *input.palette;
        self.view = input.view;
        if input.fonts.display != self.faces.role_display {
            self.faces.gear = input.fonts.display;
            self.faces.speed = input.fonts.display;
            self.faces.metric = input.fonts.display;
            self.faces.role_display = input.fonts.display;
        }

        self.shift.update(&sig, &self.palette, input.t, dt, input.reduced_flash);
        self.cue.update(self.shift.over_shift(), 0.25, dt);
        self.gear.update(sig.gear, self.cue.held(), dt);
        if self.gear.since_change() == 0.0 {
            let g = geom(self.view);
            // n+1 sits above: an upshift rolls the new gear down from above, a downshift up.
            let from = if self.gear.was_upshift() { -g.roll_px } else { g.roll_px };
            self.roll.reset(from);
            if self.gear.was_upshift() {
                self.scene.wake();
            }
        }
        self.roll.update(0.0, dt);
        // Mirror `GearGlyph`'s entry: reset on the change (1 up, -1 down), then one step.
        if self.gear.since_change() == 0.0 {
            self.entry.reset(if self.gear.was_upshift() { 1.0 } else { -1.0 });
        }
        self.entry.update(0.0, dt);

        self.rev_target = Self::rev_fraction(sig.rpm, sig.idle_rpm, sig.max_rpm);
        self.rev.update(self.rev_target, dt);
        // The ribbon warms with the pods: caution from the first amber pod, red from the first
        // red pod (the palette's rev stages are fractions of the shift rpm).
        let stage_x = |stage: f32| {
            if sig.shift_rpm >= 1.0 {
                Self::rev_fraction(stage * sig.shift_rpm, sig.idle_rpm, sig.max_rpm)
            } else {
                1.0
            }
        };
        self.caution_from = stage_x(self.palette.stages[1]);
        self.red_from = stage_x(self.palette.stages[2]);
        self.speed.update(sig.speed);
        self.rpm = if sig.rpm.is_finite() { sig.rpm.max(0.0) } else { 0.0 };
        self.unit = sig.unit.label();

        self.scene.update(
            &SceneInput {
                speed_ms: sig.speed_ms,
                a_lat: sig.g_lat * G,
                a_long: sig.g_long * G,
                slip_deg: sig.slip_deg,
                brake: sig.brake,
                reverse: sig.gear == Gear::Reverse,
                limiter: self.cue.level(),
            },
            dt,
        );
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = geom(self.view);
        let pal = self.palette;

        if let Some(r) = g.halo {
            legibility_halo(p, r, 0.5, &pal);
        }
        // Z4 first: the scene is rank 4 and sits behind everything else.
        self.scene.draw(p, &g.scene, &pal, self.faces.label);
        self.draw_ribbon(p, g, &pal);
        self.draw_pods(p, g, &pal);
        self.draw_gear(p, g, &pal);
        self.draw_readouts(p, g, &pal);
        // Clay meshes only for cars without a sprite (missing assets): they depth-test only against
        // each other, so they come last, over the 2D scene.
        self.scene.draw_meshes(p, &g.scene, &pal);
    }

    fn animating(&self) -> bool {
        self.shift.animating()
            || self.cue.animating()
            || self.gear.animating()
            || self.scene.animating()
            || !self.roll.settled(0.0, 0.05)
            || !self.entry.settled(0.0, 1e-3)
            || !self.rev.settled(self.rev_target, 1e-3)
    }
}

impl TeslaAdas {
    /// The shift lights: the Blender LED caps (a smoked acrylic lens, or the back-lit diffuser
    /// tinted with the pod's colour over its glow, bloomed), or the shared vector pods without
    /// the asset. The shared `ShiftLights` decides each pod's colour and the strobe.
    fn draw_pods(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let Some(caps) = self.caps else {
            self.shift.draw_at(
                p,
                &g.pod_centers(),
                g.pod_size,
                pal,
                &PodStyle {
                    glow_sigma: 6.0,
                    unlit_alpha: 1.0 / 255.0,
                    unlit_rim: 0.02,
                    ..PodStyle::default().with_shape(PodShape::Pill)
                },
            );
            return;
        };
        // The lens fills CAP_LENS of each cell; the cell grows by the same ratio round the pod.
        let cell = vec2(
            g.pod_size.x * CAP_CELL[0] / CAP_LENS[0],
            g.pod_size.y * CAP_CELL[1] / CAP_LENS[1],
        );
        let uv = |i: f32| Rect::from_xywh(0.5 * i, 0.0, 0.5, 1.0);
        for (i, c) in g.pod_centers().iter().enumerate() {
            let pod = Rect::from_center_size(*c, g.pod_size);
            let rect = Rect::from_center_size(*c, cell);
            let lens = |p: &mut Painter<'_>, lit: bool, tint: Color| {
                let paint = ImagePaint {
                    uv: uv(f32::from(u8::from(lit))),
                    ..ImagePaint::new(caps, rect).tint(tint)
                };
                p.fill(&Shape::rect(rect), paint);
            };
            match self.shift.color_of(i, pal) {
                Some(color) => {
                    let shape = Shape::round_rect(pod, pod.height() * 0.5);
                    p.glow(&shape, color, Glow::new(6.0, 0.55));
                    p.layer(LayerOptions::bloom(), |p| lens(p, true, color));
                }
                None => lens(p, false, Color::WHITE),
            }
        }
    }

    /// The rev ribbon (rank 3, outside the bloom): a hairline track, the redline share in a dim
    /// warning, the fill from primary to accent and a 12 px white comet at its head.
    fn draw_ribbon(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let (y, half, w) = (g.ribbon_y, g.ribbon_half, g.ribbon_px);
        let x_at = |f: f32| -half + 2.0 * half * f.clamp(0.0, 1.0);
        let line = |a: f32, b: f32, width: f32| Shape::Line {
            a: vec2(a, y),
            b: vec2(b, y),
            width_a: width,
            width_b: width,
            cap: Cap::Butt,
        };
        p.fill(&line(-half, half, 1.0), pal.roles.hairline);
        let red = x_at(self.red_from);
        if red < half - 1.0 {
            p.fill(&line(red, half, 2.0), pal.roles.warning.intensity(0.45));
        }
        let head = x_at(self.rev.value());
        if head > -half + 1.0 {
            let fill = Gradient::linear(vec2(-half, y), vec2(half, y))
                .stop(0.0, pal.primary.intensity(0.55))
                .stop(1.0, pal.accent);
            let caution = x_at(self.caution_from);
            p.fill(&line(-half, head.min(caution), w), fill);
            // Past the first amber pod the fill turns caution, past the first red pod warning,
            // so the ribbon tells the same story as the pods.
            if head > caution {
                p.fill(&line(caution, head.min(red), w), pal.roles.caution);
            }
            if head > red {
                p.fill(&line(red, head, w), pal.roles.warning);
            }
            let tail = (head - COMET_PX).max(-half);
            let ink = pal.roles.ink;
            p.fill(
                &line(tail, head, w),
                Gradient::linear(vec2(tail, y), vec2(head, y))
                    .stop(0.0, ink.with_alpha(0.0))
                    .stop(1.0, ink),
            );
            p.glow(&Shape::circle(vec2(head, y), w * 0.5), ink, Glow::new(4.0, 0.45));
        }
    }

    /// The gear in its 1 px vertical pill, with the neighbours as ghosts; the contents roll on a
    /// change and the glyph pulses at 2 Hz on a held limiter.
    fn draw_gear(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let radius = g.pill.width() * 0.5;
        let pill = Shape::round_rect(g.pill, radius);
        // A quiet pad so the glyph reads over the scene's far road, and the lit 1 px rim
        // (brighter at the top, towards the light).
        p.fill(
            &pill,
            Gradient::linear(g.pill.min, vec2(g.pill.min.x, g.pill.max.y))
                .stop(0.0, roles.oled_black.with_alpha(0.0))
                .stop(0.25, roles.oled_black.with_alpha(0.85))
                .stop(1.0, roles.oled_black.with_alpha(0.85)),
        );
        p.stroke(
            &pill,
            Gradient::linear(g.pill.min, vec2(g.pill.min.x, g.pill.max.y))
                .stop(0.0, roles.rim_specular.with_alpha(0.42))
                .stop(0.5, roles.ink.with_alpha(0.10))
                .stop(1.0, roles.ink.with_alpha(0.22)),
            Stroke::rim_px(1.0),
        );
        let roll = self.roll.value();
        let (up, down) = neighbours(self.gear.gear());
        p.scoped(|p| {
            // The glyph keeps the shared 24 px roll but not its 1.12 entry scale, so it stays
            // inside a straight-edged window 10 px inside the pill.
            p.clip(g.pill.inset(10.0), 0.0);
            let s = self.entry.value();
            p.translate(g.gear);
            p.scale(1.0 / (1.0 + 0.12 * s.abs()));
            p.translate(-g.gear);
            self.gear.draw(p, g.gear, self.faces.gear, g.gear_cap, pal);
            let pulse = self.gear.pulse();
            if pulse > 0.01 {
                p.layer(LayerOptions::bloom(), |p| {
                    let c = self.gear.color(pal).fade(0.35 * pulse);
                    self.gear.draw_with(p, g.gear, self.faces.gear, g.gear_cap, c);
                });
            }
        });
        p.scoped(|p| {
            p.clip(g.pill.inset(4.0), radius - 4.0);
            // The neighbours wait until the glyph has settled, then fade in (never under it).
            let ghost = TextStyle::new(self.faces.label, g.ghost_cap)
                .paint(
                    roles
                        .ink_label
                        .fade(0.6 * smoothstep(0.18, 0.32, self.gear.since_change())),
                )
                .align(Align::Center, VAlign::CapMiddle);
            if let Some(n) = up {
                p.text(vec2(g.gear.x, g.gear.y - g.ghost_dy + roll), n.label(), &ghost);
            }
            if let Some(n) = down {
                p.text(vec2(g.gear.x, g.gear.y + g.ghost_dy + roll), n.label(), &ghost);
            }
        });
    }

    /// Speed (rank 2) on the left; rpm and the shy chips on the right.
    fn draw_readouts(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;

        self.speed
            .draw(p, g.speed, self.faces.speed, g.speed_cap, roles.ink, None);
        // The unit sits under the digits' right edge (the units cell is always lit, so it never
        // drifts away from a short figure such as "0").
        let digits = TextStyle::new(self.faces.speed, g.speed_cap).tabular();
        let right = g.speed.x + 1.5 * p.fonts().tabular_cell(&digits);
        let unit = TextStyle::new(self.faces.label, self.micro_cap)
            .paint(roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Right, VAlign::CapTop);
        p.text(vec2(right, g.unit_y), self.unit, &unit);

        // Rpm in 50s (a steady figure), 80% ink, its label under it.
        let rpm = ((self.rpm / 50.0).round() * 50.0).clamp(0.0, 99_999.0) as i64;
        let cells = if rpm >= 10_000 { 5 } else { 4 };
        let value = TextStyle::new(self.faces.metric, g.rpm_cap)
            .paint(roles.ink.fade(0.8))
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        p.number(g.rpm, rpm, cells, &value, None);
        let label = TextStyle::new(self.faces.label, self.micro_cap)
            .paint(roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapTop);
        p.text(vec2(g.rpm.x, g.rpm.y + g.rpm_cap * 0.5 + 10.0), "RPM", &label);

        // Shy chips: dark until active (SLIP: a latched drift; LIM: at the shift point).
        let chips = [
            ("SLIP", self.scene.drifting(), roles.caution),
            ("LIM", self.shift.over_shift(), pal.shift),
        ];
        for ((text, on, color), x) in chips.into_iter().zip(g.chip_x) {
            let rect = Rect::from_center_size(vec2(x, g.chips_y), g.chip_size);
            let shape = Shape::round_rect(rect, g.chip_size.y * 0.5);
            if on {
                p.fill(&shape, color.intensity(0.10));
            }
            let rim = if on { color } else { roles.ink.with_alpha(0.06) };
            p.stroke(&shape, rim, Stroke::rim_px(1.0));
            let style = TextStyle::new(self.faces.label, g.chip_size.y * 0.42)
                .paint(shy(on, color, pal))
                .tracking(self.tracking)
                .align(Align::Center, VAlign::CapMiddle);
            p.text(rect.center(), text, &style);
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
    fn layout_stays_inside_the_view_envelopes() {
        let t = Tokens::default();
        for (g, env) in [(&COCKPIT, t.layout.cockpit_envelope), (&HOOD, t.layout.hood_envelope)] {
            let inside = |r: Rect| env.contains(r.min) && env.contains(r.max);
            assert!(inside(g.scene.viewport), "scene {:?} in {env:?}", g.scene.viewport);
            assert!(inside(g.pill), "pill in the envelope");
            for c in g.pod_centers() {
                assert!(inside(Rect::from_center_size(c, g.pod_size)), "pod at {c:?}");
            }
            assert!(g.pod_size.y <= 42.0, "the strobing strip is <= 42 px tall");
            let ribbon = Rect::from_points(
                vec2(-g.ribbon_half, g.ribbon_y - 2.0),
                vec2(g.ribbon_half, g.ribbon_y + 2.0),
            );
            assert!(inside(ribbon), "ribbon in the envelope");
            // Pods clear the ribbon, the ribbon clears the pill, and the pill holds the glyph and
            // both ghosts (with the roll) inside its straight part and end caps.
            assert!(g.ribbon_y - g.ribbon_px * 0.5 - (g.pods_y + g.pod_size.y * 0.5) >= 8.0);
            assert!(g.pill.min.y - (g.ribbon_y + g.ribbon_px * 0.5) >= 8.0);
            let glyph_half = g.gear_cap * 0.5;
            for dir in [-1.0f32, 1.0] {
                let ghost = g.gear.y + dir * g.ghost_dy;
                let gap = (ghost - g.gear.y).abs() - glyph_half - g.ghost_cap * 0.5;
                assert!(gap >= 4.0, "ghost {gap} px from the glyph");
                assert!(
                    ghost - g.ghost_cap * 0.5 > g.pill.min.y && ghost + g.ghost_cap * 0.5 < g.pill.max.y,
                    "ghost inside the pill"
                );
            }
            assert!(g.roll_px <= 24.0, "the pill rolls 24 px at most");
            // The readouts clear the pill and stay above the scene's road (they may overlap its
            // dissolving far end only).
            assert!(g.speed.x + g.speed_cap * 1.0 < g.pill.min.x);
            assert!(g.chip_x[0] - g.chip_size.x * 0.5 > g.pill.max.x);
        }
    }

    #[test]
    fn neighbours_follow_the_gearbox() {
        assert_eq!(neighbours(Gear::Neutral), (Some(Gear::Forward(1)), Some(Gear::Reverse)));
        assert_eq!(
            neighbours(Gear::Forward(1)),
            (Some(Gear::Forward(2)), Some(Gear::Neutral))
        );
        assert_eq!(neighbours(Gear::Forward(10)), (None, Some(Gear::Forward(9))));
        assert_eq!(neighbours(Gear::Reverse), (Some(Gear::Neutral), None));
    }

    #[test]
    fn the_ribbon_runs_idle_to_limiter() {
        assert_eq!(TeslaAdas::rev_fraction(900.0, 900.0, 8700.0), 0.0);
        assert_eq!(TeslaAdas::rev_fraction(8700.0, 900.0, 8700.0), 1.0);
        assert_eq!(TeslaAdas::rev_fraction(f32::NAN.max(0.0), 900.0, 8700.0), 0.0);
    }
}
