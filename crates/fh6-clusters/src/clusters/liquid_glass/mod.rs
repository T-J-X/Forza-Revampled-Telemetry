//! Liquid Glass (`ClusterTheme::LiquidGlass`), catalog row 1: the Liquid Glass family's
//! pattern-setter and the default cockpit theme (cockpit + hood, build #1, the Phase 7 pilot).
//!
//! One monolithic sapphire capsule (a 1240 × 300 squircle, n 4) at the regular glass tint, with
//! no wallpaper: only the 1 px light-driven rim, the fresnel band and the crown sheen define it.
//! - Rank 1: 16 glass pills in a strip under the crown, and the gear (Saira Condensed 700, cap
//!   200) under a 230 px domed lens with a dimming pad.
//! - Rank 2: tabular speed on the left (Saira 200, cap 112, ghost 8s).
//! - Rank 3: a light pipe 14 px inside the rim from lower-left over the crown to lower-right,
//!   6 → 14 px wide, ending in a meniscus bead that swells with throttle.
//! - Rank 4: a pendant glass drop under the centre holding the caustic G spot (felt force on the
//!   G-ball spring; body slip defocuses it from a 6 px point into a 40 px ring caustic).
//! - Power and boost float on the right in Saira 300.
//!
//! Signature moments:
//! - **Upshift light-run:** a glint lights the new gear glyph, climbs the dome rim, splits at
//!   the crown and races both ways round the capsule rim at 4000 px/s, meeting in the pendant
//!   drop after about 340 ms, which flashes. The bead slides back on the ribbon spring meanwhile.
//! - **Limiter:** only the pill strip strobes (10 Hz, `common::ShiftLights`). The one steady cue
//!   is the hot meniscus: the pipe warms to the shift colour, the bead core turns white with a
//!   warning halo and the glass rim brightens (no bloom on the rim).
//! - **Light:** lateral G swings the rim crescent, crown sheen and bead specular up to 25°.
//! - **Show:** a 200 ms lensing and rim ramp; hiding is the host's 1 frame.
//!
//! Brief: `spec.md`. Telemetry is native: rpm (current, idle, limiter), the tokens' shift point,
//! gear, speed, throttle, body slip (smoothed, from velocity), acceleration, power, boost and the
//! distance integrated from speed (the caustic phase). Nothing is synthesised.
use crate::common::glass_plate::{dimming_pad, legibility_halo, squircle, GlassPlate, GlassVariant};
use crate::common::hero_gear::HeroGear;
use crate::common::label::{logo_zone, LogoVariant};
use crate::common::ribbon::{PathRibbon, RibbonStyle};
use crate::common::shift::{LimiterCue, PodShape, PodStyle};
use crate::common::speed::TabularSpeed;
use crate::common::ShiftLights;
use core::f32::consts::TAU;
use fh6_kit::cluster::Setup;
use fh6_kit::ImageId;
use fh6_kit::{
    deg, vec2, Align, BlendMode, Cap, Cluster, Color, FontId, FontRole, FrameInput, Gradient, ImagePaint, LayerOptions,
    Layout, Logo, Painter, Palette, Rect, Shape, Signals, Smoothed, Spring, Stroke, TextStyle, Tokens, VAlign, Vec2,
    View,
};

/// Comet speed of the upshift light-run (px/s).
const RUN_SPEED: f32 = 4000.0;
/// The glint on the glyph and the climb up the dome rim (s).
const GLINT_S: f32 = 0.06;
/// How long the pendant drop flashes when the two comets meet (s).
const MEET_S: f32 = 0.16;
/// The 16 pills (catalog brief).
const PILLS: u32 = 16;
/// Capsule exponent (squircle). 5 keeps the top and bottom flat over the middle and the ends
/// round at 4:1 and 6.5:1 (4 pinched the hood capsule's ends into points).
const N: f32 = 5.0;
/// Inset of the light pipe's centre line from the capsule rim (px).
const PIPE_INSET: f32 = 14.0;

/// Geometry of one view (local 4K px around the anchor).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
    center: Vec2,
    size: Vec2,
    /// Half the span of the pill centres (x).
    pill_span: f32,
    pill_size: Vec2,
    /// The pills follow the crown: each centre sits this far below the pipe's centre line.
    pill_drop: f32,
    dome: Vec2,
    dome_r: f32,
    gear_cap: f32,
    speed: Vec2,
    speed_cap: f32,
    metrics_x: f32,
    metrics_y: [f32; 2],
    metric_cap: f32,
    drop: Vec2,
    drop_r: f32,
    /// Meniscus bead radius at the shift point (it shrinks towards idle).
    bead_r: f32,
    /// The logo zone (none in the hood view: nothing to mirror it on the left).
    logo: Option<Rect>,
    halo: bool,
}

const COCKPIT: Geom = Geom {
    center: vec2(0.0, -30.0),
    size: vec2(1240.0, 300.0),
    pill_span: 470.0,
    pill_size: vec2(44.0, 18.0),
    pill_drop: 34.0,
    dome: vec2(0.0, -2.0),
    dome_r: 104.0,
    gear_cap: 168.0,
    speed: vec2(-372.0, 4.0),
    speed_cap: 112.0,
    metrics_x: 524.0,
    metrics_y: [-38.0, 38.0],
    metric_cap: 44.0,
    drop: vec2(0.0, 126.0),
    drop_r: 44.0,
    bead_r: 13.0,
    logo: Some(Rect {
        min: vec2(250.0, 84.0),
        max: vec2(470.0, 104.0),
    }),
    halo: false,
};

/// Hood: the scrim is 0%, so the capsule stretches to 1560 × 240 with a legibility halo; speed
/// and metrics mirror at ±470 px.
const HOOD: Geom = Geom {
    center: vec2(0.0, -30.0),
    size: vec2(1560.0, 240.0),
    pill_span: 540.0,
    pill_size: vec2(40.0, 14.0),
    pill_drop: 32.0,
    dome: vec2(0.0, -4.0),
    dome_r: 78.0,
    gear_cap: 124.0,
    speed: vec2(-470.0, -8.0),
    speed_cap: 96.0,
    metrics_x: 620.0,
    metrics_y: [-34.0, 26.0],
    metric_cap: 38.0,
    drop: vec2(0.0, 96.0),
    drop_r: 38.0,
    bead_r: 11.0,
    logo: None,
    halo: true,
};

impl Geom {
    /// The y of the light pipe's centre line on the crown at `x` (the squircle inset by
    /// [`PIPE_INSET`]).
    fn crown_y(&self, x: f32) -> f32 {
        let a = self.size.x * 0.5 - PIPE_INSET;
        let b = self.size.y * 0.5 - PIPE_INSET;
        let u = ((x - self.center.x).abs() / a).min(1.0).powf(N);
        self.center.y - b * (1.0 - u).max(0.0).powf(1.0 / N)
    }

    /// The pill centres: evenly spaced in x, on an offset curve of the crown.
    fn pill_centers(&self) -> [Vec2; PILLS as usize] {
        let mut out = [Vec2::ZERO; PILLS as usize];
        let step = 2.0 * self.pill_span / (PILLS - 1) as f32;
        for (i, c) in out.iter_mut().enumerate() {
            let x = self.center.x - self.pill_span + step * i as f32;
            *c = vec2(x, self.crown_y(x) + self.pill_drop);
        }
        out
    }
}

fn geom(view: View) -> &'static Geom {
    match view {
        View::Hood => &HOOD,
        View::Cockpit | View::Chase => &COCKPIT,
    }
}

/// The faces this cluster sets (resolved once; a user display font replaces gear and speed).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    gear: FontId,
    speed: FontId,
    metric: FontId,
    label: FontId,
    wide: FontId,
    /// The display role at creation, to notice a user font override.
    role_display: FontId,
}

/// One view's paths: the light pipe and the two light-run halves.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Paths {
    pipe: PathRibbon,
    run_left: PathRibbon,
    run_right: PathRibbon,
}

impl Paths {
    fn new(tokens: &Tokens, g: &Geom) -> Self {
        let pipe = PathRibbon::squircle_rim(
            tokens,
            g.center,
            g.size,
            N,
            PIPE_INSET,
            deg(150.0),
            deg(390.0),
            // Rank 3 and subordinate to the pills: thin, dim and never in the bloom.
            RibbonStyle {
                width_start: 4.0,
                width_end: 8.0,
                gain: 0.6,
                ..RibbonStyle::default()
            },
        );
        let rim = RibbonStyle {
            track_width: 0.0,
            ..RibbonStyle::default()
        };
        // From the crown (12 o'clock) down each side to the bottom centre (the pendant).
        let run_left = PathRibbon::squircle_rim(tokens, g.center, g.size, N, 1.5, deg(270.0), deg(90.0), rim);
        let run_right = PathRibbon::squircle_rim(tokens, g.center, g.size, N, 1.5, deg(270.0), deg(450.0), rim);
        Self {
            pipe,
            run_left,
            run_right,
        }
    }
}

/// The Liquid Glass cluster.
#[derive(Debug, Clone)]
pub struct LiquidGlass {
    faces: Faces,
    sig: Signals,
    palette: Palette,
    view: View,
    plate: GlassPlate,
    cockpit: Paths,
    hood: Paths,
    gear: HeroGear,
    speed: TabularSpeed,
    shift: ShiftLights,
    cue: LimiterCue,
    throttle: Spring,
    spot: [Spring; 2],
    spot_target: [f32; 2],
    defocus: Smoothed<f32>,
    power_hp: Smoothed<f32>,
    boost: Smoothed<f32>,
    distance: f32,
    /// Seconds since the last upshift light-run started (negative: none running).
    run_t: f32,
    logo: Logo,
    caustic: Option<ImageId>,
    micro_cap: f32,
    tracking: f32,
}

impl LiquidGlass {
    /// Builds the cluster: faces, paths and the optional Phase 6 ring caustic.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let t: &Tokens = setup.tokens;
        let palette = Palette::family(t, "liquidGlass");
        let display = setup.fonts.role(FontRole::Display);
        let text = setup.fonts.role(FontRole::Text);
        let faces = Faces {
            gear: setup.fonts.find("Saira Condensed", 700, false).unwrap_or(display),
            speed: setup.fonts.find("Saira", 200, false).unwrap_or(display),
            metric: setup.fonts.find("Saira", 400, false).unwrap_or(display),
            label: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            wide: setup.fonts.role(FontRole::Wide),
            role_display: display,
        };
        let caustic = setup.assets.and_then(|a| {
            let path = a.join("glass/caustic_lens_circle.png");
            path.is_file()
                .then(|| setup.images.load_file(setup.device, setup.queue, &path, None).ok())
                .flatten()
        });
        Self {
            faces,
            sig: Signals::default(),
            palette,
            view: View::Cockpit,
            plate: GlassPlate::new(t, &palette, GlassVariant::Regular),
            cockpit: Paths::new(t, &COCKPIT),
            hood: Paths::new(t, &HOOD),
            gear: HeroGear::new(t),
            speed: TabularSpeed::new(),
            shift: ShiftLights::new(PILLS),
            cue: LimiterCue::new(),
            throttle: Spring::new(t.springs.snap, 0.0),
            spot: [Spring::new(t.springs.g_ball, 0.0), Spring::new(t.springs.g_ball, 0.0)],
            spot_target: [0.0; 2],
            defocus: Smoothed::new(0.0, 0.12),
            power_hp: Smoothed::new(0.0, 0.10),
            boost: Smoothed::new(0.0, 0.10),
            distance: 0.0,
            run_t: -1.0,
            logo: Logo::default(),
            caustic,
            micro_cap: t.type_scale.micro_label,
            tracking: t.type_scale.label_tracking,
        }
    }

    fn paths(&self) -> &Paths {
        match self.view {
            View::Hood => &self.hood,
            View::Cockpit | View::Chase => &self.cockpit,
        }
    }

    fn paths_mut(&mut self) -> &mut Paths {
        match self.view {
            View::Hood => &mut self.hood,
            View::Cockpit | View::Chase => &mut self.cockpit,
        }
    }

    /// Seconds the comets need to reach the pendant in the current view.
    fn run_duration(&self) -> f32 {
        let p = self.paths();
        GLINT_S + p.run_left.length().max(p.run_right.length()) / RUN_SPEED
    }

    /// The rpm fraction the pipe shows: idle → limiter mapped onto the pipe.
    fn pipe_target(sig: &Signals) -> f32 {
        let span = (sig.max_rpm - sig.idle_rpm).max(1.0);
        ((sig.rpm - sig.idle_rpm) / span).clamp(0.0, 1.0)
    }
}

impl Cluster for LiquidGlass {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt.max(0.0);
        let sig = *input.signals;
        self.sig = sig;
        self.palette = *input.palette;
        if input.view != self.view {
            self.view = input.view;
            let f = Self::pipe_target(&sig);
            self.paths_mut().pipe.snap(f);
        }
        // A user display font (FontChoice.display differs from the role) replaces the numerals.
        if input.fonts.display != self.faces.role_display {
            self.faces.gear = input.fonts.display;
            self.faces.speed = input.fonts.display;
            self.faces.metric = input.fonts.display;
            self.faces.role_display = input.fonts.display;
        }
        if self.logo != *input.logo {
            self.logo.clone_from(input.logo);
        }

        self.plate.update(sig.g_lat, dt, input.tokens, &self.palette);
        let span = (sig.max_rpm - sig.idle_rpm).max(1.0);
        let rate = sig.rpm_rate / span;
        let target = Self::pipe_target(&sig);
        self.paths_mut().pipe.update(target, rate, dt);
        self.shift.update(&sig, &self.palette, input.t, dt, input.reduced_flash);
        self.cue.update(self.shift.over_shift(), 0.25, dt);
        self.gear.update(sig.gear, self.cue.held(), dt);
        if self.gear.since_change() == 0.0 && self.gear.was_upshift() {
            self.run_t = 0.0;
        } else if self.run_t >= 0.0 {
            self.run_t += dt;
            if self.run_t > self.run_duration() + MEET_S {
                self.run_t = -1.0;
            }
        }
        self.speed.update(sig.speed);
        self.throttle.update(sig.throttle, dt);
        // Felt force: accelerating right pushes the spot left; accelerating forward pushes it
        // back (down on screen), braking forward (up).
        // 1.5 g reaches ±26 px in the cockpit drop.
        let reach = geom(self.view).drop_r * 0.59;
        self.spot_target = [
            (-sig.g_lat / 1.5).clamp(-1.0, 1.0) * reach,
            (sig.g_long / 1.5).clamp(-1.0, 1.0) * reach * 0.6,
        ];
        self.spot[0].update(self.spot_target[0], dt);
        self.spot[1].update(self.spot_target[1], dt);
        // Grip = focus: body slip from 3° defocuses to a full ring by 18°.
        let slip = ((sig.slip_deg.abs() - 3.0) / 15.0).clamp(0.0, 1.0);
        self.defocus.update(slip * slip * (3.0 - 2.0 * slip), dt);
        let power = input.telemetry.power_watts;
        let boost = input.telemetry.boost_psi;
        self.power_hp.update(
            if power.is_finite() {
                (power / 745.7).max(0.0)
            } else {
                0.0
            },
            dt,
        );
        self.boost.update(if boost.is_finite() { boost } else { 0.0 }, dt);
        if sig.speed_ms.is_finite() {
            self.distance += sig.speed_ms * dt;
        }
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = geom(self.view);
        let pal = self.palette;
        let roles = pal.roles;
        let paths = self.paths();
        let heat = self.cue.level();
        let capsule = squircle(g.center, g.size, N);

        // 0. Hood: a legibility halo over a bright bonnet (alpha <= 0.55).
        if g.halo {
            legibility_halo(p, Rect::from_center_size(g.center, g.size), 0.5, &pal);
        }

        // 1. The sapphire capsule and the pendant drop (one glass layer). The drop hangs under
        //    the flat bottom of the capsule; its top half is hidden inside the capsule.
        // The limiter brightens the rim only a little (the pills own the limiter). The crown
        // sheen is a thin specular band, not a gel-button gloss; the fresnel edge carries the
        // thickness.
        let mut glass = self.plate.material(heat * 0.4);
        glass.sheen *= 0.45;
        glass.sheen_height = 0.08;
        glass.fresnel = (glass.fresnel * 1.6).min(1.0);
        p.layer(LayerOptions::glass(18.0), |p| {
            p.fill(&capsule, glass);
            p.scoped(|p| {
                let bottom = g.center.y + g.size.y * 0.5;
                p.clip(
                    Rect::from_points(
                        vec2(g.drop.x - g.drop_r - 4.0, bottom - 1.0),
                        g.drop + vec2(g.drop_r + 4.0, g.drop_r + 4.0),
                    ),
                    0.0,
                );
                p.fill(&Shape::circle(g.drop, g.drop_r), glass);
            });
        });
        // Hood: over a bright bonnet the body needs a neutral backing, darkest in the centre
        // (inset so the 1 px rim and the fresnel band stay lit).
        if g.halo {
            let inner = squircle(g.center, g.size - Vec2::splat(12.0), N);
            p.fill(
                &inner,
                Gradient::radial(g.center, g.size.x * 0.5)
                    .stop(0.0, roles.oled_black.with_alpha(0.55))
                    .stop(1.0, roles.oled_black.with_alpha(0.25)),
            );
        }

        // 2. The dome lens over the gear: a dimming pad (the lens interior is the darkest area
        //    of the capsule), a 1 px rim lit from the virtual light, and one tapered specular
        //    crescent towards the light with a weaker counter-crescent opposite.
        dimming_pad(p, g.dome, g.dome_r * 0.98, 1.0, &pal);
        let light = self.plate.light_dir();
        let m = self.plate.materialized();
        let lit_rim = Gradient::linear(g.dome + light * g.dome_r, g.dome - light * g.dome_r)
            .stop(0.0, roles.rim_specular.with_alpha(0.55 * m))
            .stop(0.45, pal.primary.with_alpha(0.22 * m))
            .stop(1.0, pal.primary.with_alpha(0.06 * m));
        p.stroke(&Shape::circle(g.dome, g.dome_r), lit_rim, Stroke::rim_px(1.0));
        let la = self.plate.light_angle();
        for (mid, alpha) in [(la, 0.30), (la + deg(180.0), 0.30 * 0.35)] {
            let half = deg(27.5);
            let cs = mid - half - 0.01;
            let paint = Gradient::conic(g.dome, cs)
                .stop(0.0, roles.rim_specular.with_alpha(0.0))
                .stop((half + 0.01) / TAU, roles.rim_specular.with_alpha(alpha * m))
                .stop((2.0 * half + 0.01) / TAU, roles.rim_specular.with_alpha(0.0));
            p.fill(
                &Shape::arc(g.dome, g.dome_r - 6.0, 3.0, mid - half, 2.0 * half, Cap::Butt),
                paint,
            );
        }

        // 3. Light pipe (rank 3, outside the bloom, in the primary hue at any rpm; only the
        //    redline share is warm) and the speed's own quiet pad.
        paths.pipe.draw_track(p, &pal);
        let frac = paths.pipe.fraction();
        paths.pipe.draw_fill(p, frac, &pal, 0.0);
        dimming_pad(p, g.speed, g.speed_cap * 1.5, 0.6, &pal);

        // 4. Rank 1: the gear, clipped to its lens (the entry overshoot never reaches the rim).
        //    It blooms only on the limiter, softly.
        p.scoped(|p| {
            p.clip(
                Rect::from_center_size(g.dome, Vec2::splat(g.dome_r * 2.0 - 4.0)),
                g.dome_r - 2.0,
            );
            self.gear.draw(p, g.dome, self.faces.gear, g.gear_cap, &pal);
            if heat > 0.01 {
                p.layer(LayerOptions::bloom(), |p| {
                    let c = self.gear.color(&pal).fade(0.35 * heat);
                    self.gear.draw_with(p, g.dome, self.faces.gear, g.gear_cap, c);
                });
            }
        });

        // The meniscus bead: its size and glow follow the orb intensity law
        // (0.35 + 0.65 × rpm / shift rpm), so at idle it is a small dim point inside the glass.
        let intensity = 0.35 + 0.65 * (self.sig.rpm / self.sig.shift_rpm.max(1.0)).clamp(0.0, 1.0);
        let bead = g.bead_r * (0.25 + 0.75 * intensity);
        let core = bead * (0.55 + 0.25 * self.throttle.value().clamp(0.0, 1.0));
        paths
            .pipe
            .draw_bead(p, frac.max(0.004), bead, core, light, &pal, intensity, heat);

        // The 16 pills on an offset curve of the crown, clear of the bead.
        let centers = g.pill_centers();
        self.shift.draw_at(
            p,
            &centers,
            g.pill_size,
            &pal,
            &PodStyle {
                glow_sigma: 6.0,
                ..PodStyle::default().with_shape(PodShape::Pill)
            },
        );

        // 5. Speed (rank 2): floating light digits (no ghost 8s in this family), the unit on
        //    the numerals' baseline to the right of the last cell.
        let speed_style = TextStyle::new(self.faces.speed, g.speed_cap)
            .paint(roles.ink)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        let cells = p.number(g.speed, self.speed.value(), 3, &speed_style, None);
        let baseline = g.speed.y + g.speed_cap * 0.5;
        let unit_style = TextStyle::new(self.faces.label, self.micro_cap)
            .paint(roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Left, VAlign::Baseline);
        p.text(vec2(cells.max.x + 14.0, baseline), self.sig.unit.label(), &unit_style);

        // 6. Power and boost on the right flank: label, value and unit share one baseline per
        //    row; values at 80% ink (rank 4 by size).
        let value = TextStyle::new(self.faces.metric, g.metric_cap)
            .paint(roles.ink.fade(0.8))
            .align(Align::Right, VAlign::Baseline)
            .tabular();
        let unit = TextStyle::new(self.faces.label, self.micro_cap * 0.85)
            .paint(roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Left, VAlign::Baseline);
        let label = TextStyle {
            align: Align::Right,
            ..unit
        };
        let right = g.metrics_x - 52.0;
        let rows = [g.metrics_y[0] + g.metric_cap * 0.5, g.metrics_y[1] + g.metric_cap * 0.5];
        let hp = self.power_hp.value().round().clamp(0.0, 9999.0) as i64;
        p.number(vec2(right, rows[0]), hp, 4, &value, None);
        p.text(vec2(right + 10.0, rows[0]), "HP", &unit);
        let cell = g.metric_cap / 0.7 * 0.6;
        let label_x = right - cell * 4.0 - 18.0;
        p.text(vec2(label_x, rows[0]), "POWER", &label);
        // Boost with one decimal: integer cells, a narrow point, then the tenth (no allocation).
        let b = self.boost.value().clamp(-99.9, 99.9);
        let tenths = (b * 10.0).round() as i64;
        let (whole, frac10) = (tenths.abs() / 10, tenths.abs() % 10);
        let tenth_r = p.number(vec2(right, rows[1]), frac10, 1, &value, None);
        let dot = TextStyle {
            align: Align::Center,
            ..value
        };
        p.text(vec2(tenth_r.min.x - cell * 0.15, rows[1]), ".", &dot);
        let whole_r = p.number(vec2(tenth_r.min.x - cell * 0.3, rows[1]), whole, 2, &value, None);
        if tenths < 0 {
            let digits = if whole >= 10 { 2.0 } else { 1.0 };
            p.text(vec2(whole_r.max.x - cell * (digits + 0.3), rows[1]), "-", &dot);
        }
        p.text(vec2(right + 10.0, rows[1]), "PSI", &unit);
        p.text(vec2(label_x, rows[1]), "BOOST", &label);

        // 7. Logo / custom-name zone on the lower rim (rank 5; cockpit only).
        if let Some(zone) = g.logo {
            logo_zone(p, zone, &self.logo, self.faces.wide, LogoVariant::Full, &pal);
        }

        // 8. The pendant drop's caustic G spot (additive light inside the drop).
        self.draw_spot(p, g, &pal);

        // 9. The upshift light-run.
        if self.run_t >= 0.0 {
            self.draw_run(p, g, &pal);
        }
    }

    fn animating(&self) -> bool {
        self.plate.animating()
            || self.paths().pipe.animating()
            || self.shift.animating()
            || self.gear.animating()
            || self.cue.animating()
            || self.run_t >= 0.0
            || !self.spot[0].settled(self.spot_target[0], 0.05)
            || !self.spot[1].settled(self.spot_target[1], 0.05)
    }
}

impl LiquidGlass {
    /// The caustic spot: a sharp point when the tyres grip, a dispersive ring when sliding.
    fn draw_spot(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let d = self.defocus.value().clamp(0.0, 1.0);
        let bottom = g.center.y + g.size.y * 0.5;
        // The visible part of the drop hangs from the capsule bottom to the drop's lowest point.
        let hang = g.drop.y + g.drop_r - bottom;
        let c = vec2(
            g.drop.x + self.spot[0].value(),
            bottom + hang * 0.52 + self.spot[1].value(),
        );
        p.scoped(|p| {
            p.clip(
                Rect::from_points(
                    vec2(g.drop.x - g.drop_r, bottom + 1.0),
                    g.drop + vec2(g.drop_r, g.drop_r),
                ),
                0.0,
            );
            // The 0.5 g reference ring (a scale for the spot, 10% ink).
            let ring_c = vec2(g.drop.x, bottom + hang * 0.52);
            let half_g = g.drop_r * 0.59 / 3.0;
            p.stroke(
                &Shape::circle(ring_c, half_g),
                roles.ink.with_alpha(0.10),
                Stroke::rim_px(1.0),
            );
            p.set_blend(BlendMode::Add);
            // Focused point (rank 4: at most 60% ink, no bloom).
            let r_point = 3.0 + 5.0 * d;
            p.fill(
                &Shape::circle(c, r_point * 2.4),
                Gradient::radial(c, r_point * 2.4)
                    .stop(0.0, roles.ink.with_alpha(0.6 * (1.0 - d)))
                    .stop(0.3, pal.primary.with_alpha(0.3 * (1.0 - d)))
                    .stop(1.0, pal.primary.with_alpha(0.0)),
            );
            // Defocused ring caustic (Phase 6 dispersive ring if present, else procedural).
            if d > 0.01 {
                // 6 px point -> 40 px ring (diameters), kept inside the hanging part.
                let ring_r = (3.0 + 17.0 * d).min(hang * 0.42);
                match self.caustic {
                    Some(img) => {
                        let rect = Rect::from_center_size(c, vec2(ring_r * 2.0, ring_r * 2.0));
                        p.fill(
                            &Shape::circle(c, ring_r),
                            ImagePaint::new(img, rect).tint(Color::WHITE.with_alpha(0.65 * d)),
                        );
                    }
                    None => {
                        p.fill(
                            &Shape::ring(c, ring_r * 0.86, ring_r * 0.24),
                            Gradient::radial(c, ring_r)
                                .stop(0.70, pal.primary.with_alpha(0.0))
                                .stop(0.86, roles.ink.with_alpha(0.55 * d))
                                .stop(1.0, pal.primary.with_alpha(0.0)),
                        );
                    }
                }
            }
            // Slow caustic shimmer driven by distance travelled (still at idle).
            let phase = self.distance * 0.004;
            let sc = vec2(g.drop.x, bottom + hang * 0.5) + vec2(phase.cos(), phase.sin() * 0.4) * (g.drop_r * 0.4);
            p.fill(
                &Shape::circle(sc, hang * 0.4),
                Gradient::radial(sc, hang * 0.4)
                    .stop(0.0, pal.primary.with_alpha(0.05))
                    .stop(1.0, pal.primary.with_alpha(0.0)),
            );
        });
    }

    /// The upshift light-run: glint on the glyph, a short streak climbing each side of the dome
    /// rim, then two specular streaks running round the capsule rim that meet in the pendant
    /// drop. Additive light on the 1 px rim, not bloom balls; dimmed while it crosses the crown
    /// so it never competes with the bead.
    fn draw_run(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let paths = self.paths();
        let t = self.run_t;
        p.scoped(|p| {
            p.set_blend(BlendMode::Add);
            if t < GLINT_S {
                let k = t / GLINT_S;
                // Glint on the glyph: a fading additive copy, clipped to the lens.
                p.scoped(|p| {
                    p.clip(
                        Rect::from_center_size(g.dome, Vec2::splat(g.dome_r * 2.0 - 4.0)),
                        g.dome_r - 2.0,
                    );
                    self.gear.draw_with(
                        p,
                        g.dome,
                        self.faces.gear,
                        g.gear_cap,
                        roles.ink.with_alpha(0.3 * (1.0 - k)),
                    );
                });
                // Climb: a 40° streak up each side of the lens rim, bottom to top.
                let head = deg(180.0) * k;
                let len = deg(40.0).min(head.max(deg(4.0)));
                let from = head - len;
                for (start, sweep) in [(deg(90.0) + from, len), (deg(90.0) - from, -len)] {
                    p.fill(
                        &Shape::arc(g.dome, g.dome_r, 2.0, start, sweep, Cap::Butt),
                        roles.ink.with_alpha(0.7),
                    );
                }
            }
            const TAIL_PX: f32 = 160.0;
            const TAIL_STEPS: usize = 16;
            let s = (t - GLINT_S).max(0.0) * RUN_SPEED;
            for run in [&paths.run_left, &paths.run_right] {
                let len = run.length().max(1.0);
                let head = (s / len).min(1.0);
                if t >= GLINT_S && head < 1.0 {
                    // 30% while the streak crosses the crown run, full on the flanks.
                    let k = (head / 0.3).clamp(0.0, 1.0);
                    let fade = 0.3 + 0.7 * k * k * (3.0 - 2.0 * k);
                    // A continuous tapered streak: butt-capped segments, so nothing overlaps.
                    let mut prev = run.point_at(head);
                    for i in 1..=TAIL_STEPS {
                        let back = (head - i as f32 * (TAIL_PX / TAIL_STEPS as f32) / len).max(0.0);
                        let q = run.point_at(back);
                        let a0 = 1.0 - (i - 1) as f32 / TAIL_STEPS as f32;
                        let a1 = 1.0 - i as f32 / TAIL_STEPS as f32;
                        p.fill(
                            &Shape::Line {
                                a: prev,
                                b: q,
                                width_a: 0.5 + 2.0 * a0,
                                width_b: 0.5 + 2.0 * a1,
                                cap: Cap::Butt,
                            },
                            roles.ink.lerp(pal.primary, 1.0 - a0).with_alpha(0.8 * a0 * a0 * fade),
                        );
                        prev = q;
                        if back <= 0.0 {
                            break;
                        }
                    }
                }
            }
            // The meeting: the pendant drop flashes softly (rank 4: no bloom).
            let arrive = self.run_duration();
            if t >= arrive {
                let k = 1.0 - ((t - arrive) / MEET_S).clamp(0.0, 1.0);
                let c = g.drop + vec2(0.0, 6.0);
                p.fill(
                    &Shape::circle(c, g.drop_r * 0.8),
                    Gradient::radial(c, g.drop_r * 0.8)
                        .stop(0.0, roles.ink.with_alpha(0.5 * k))
                        .stop(0.5, pal.primary.with_alpha(0.25 * k))
                        .stop(1.0, pal.primary.with_alpha(0.0)),
                );
            }
        });
    }
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(LiquidGlass::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::glass_plate::squircle_point;

    #[test]
    fn layout_stays_inside_the_view_envelopes() {
        let t = Tokens::default();
        for (g, env) in [(&COCKPIT, t.layout.cockpit_envelope), (&HOOD, t.layout.hood_envelope)] {
            let cap = Rect::from_center_size(g.center, g.size);
            assert!(
                env.contains(cap.min) && env.contains(cap.max),
                "capsule {cap:?} in {env:?}"
            );
            assert!(g.drop.y + g.drop_r <= env.max.y, "pendant drop inside the envelope");
            // Every pill clears the full-size bead on the pipe by >= 4 px, and the dome by 12 px.
            let dome_top = g.dome.y - g.dome_r;
            for c in g.pill_centers() {
                let top = c.y - g.pill_size.y * 0.5;
                let clear = top - g.crown_y(c.x);
                assert!(
                    clear >= g.bead_r + 4.0,
                    "pill at x {} only {clear} px below the pipe",
                    c.x
                );
                if (c.x - g.dome.x).abs() < g.dome_r + g.pill_size.x * 0.5 {
                    assert!(
                        c.y + g.pill_size.y * 0.5 <= dome_top - 12.0,
                        "pill at x {} on the dome",
                        c.x
                    );
                }
            }
            assert!(g.pill_size.y <= 42.0, "the strobing strip is <= 42 px tall");
            // The lens keeps >= 16 px from the capsule's bottom rim.
            let bottom = g.center.y + g.size.y * 0.5;
            assert!(bottom - (g.dome.y + g.dome_r) >= 16.0, "lens clear of the rim");
        }
    }

    #[test]
    fn the_pipe_follows_idle_to_limiter() {
        let sig = Signals {
            rpm: 900.0,
            idle_rpm: 900.0,
            max_rpm: 8700.0,
            ..Signals::default()
        };
        assert_eq!(LiquidGlass::pipe_target(&sig), 0.0);
        let top = Signals { rpm: 8700.0, ..sig };
        assert_eq!(LiquidGlass::pipe_target(&top), 1.0);
    }

    #[test]
    fn the_light_run_reaches_the_pendant_in_about_340_ms() {
        let t = Tokens::default();
        let p = Paths::new(&t, &COCKPIT);
        let d = GLINT_S + p.run_left.length().max(p.run_right.length()) / RUN_SPEED;
        assert!((0.30..0.42).contains(&d), "{d}");
        // The two halves are mirror images.
        assert!((p.run_left.length() - p.run_right.length()).abs() < 1.0);
        let bottom = squircle_point(COCKPIT.center, COCKPIT.size, N, deg(90.0), 1.5);
        assert!((p.run_left.point_at(1.0) - bottom).length() < 2.0);
    }
}
