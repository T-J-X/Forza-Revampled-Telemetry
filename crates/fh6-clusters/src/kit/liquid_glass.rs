//! Liquid Glass sample (kit showcase, family `liquidGlass`): a frosted sapphire plate that lenses
//! a pool of light behind it, a luminous ribbon tach with a radial meniscus orb, a crystal gear
//! prism, tabular speed with ghost 8s and an outside-in shift strip. Spec card P8-1.
//!
//! Not the final design (Phase 7); it proves the kit's glass, bloom, gradients, text and springs.
use crate::common::{GearGlyph, ShiftLights, SpeedReadout, VirtualLight};
use fh6_kit::cluster::Setup;
use fh6_kit::{
    deg, vec2, Align, Cap, Cluster, FontChoice, FrameInput, Glass, Glow, Gradient, LayerOptions, Layout, Logo, Painter,
    Palette, Rect, Shape, Signals, Spring, Stroke, TextStyle, TickRing, Tokens, Tween, VAlign, Vec2,
};

/// Ribbon start angle (7:30) and sweep (270 degrees), spec card P8-1.
const START: f32 = 135.0;
const SWEEP: f32 = 270.0;
/// Ribbon geometry (local 4K px).
const CENTER: Vec2 = vec2(0.0, 30.0);
const RADIUS: f32 = 205.0;
const THICKNESS: f32 = 22.0;
/// The plate.
const PLATE: Rect = Rect {
    min: vec2(-520.0, -250.0),
    max: vec2(520.0, 236.0),
};

/// The Liquid Glass sample cluster.
#[derive(Debug, Clone)]
pub struct LiquidGlassSample {
    sig: Signals,
    palette: Palette,
    fonts: FontChoice,
    tokens_glass: Glass,
    rpm: Spring,
    throttle: Spring,
    brake: Spring,
    light: VirtualLight,
    gear: GearGlyph,
    speed: SpeedReadout,
    shift: ShiftLights,
    materialize: Tween,
    logo: Logo,
    micro_cap: f32,
    label_tracking: f32,
}

impl LiquidGlassSample {
    /// Builds the sample (no assets needed; the caustic texture is optional in Phase 6).
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let t: &Tokens = setup.tokens;
        let palette = Palette::family(t, "liquidGlass");
        let mut materialize = Tween::new(t.durations.materialize);
        materialize.restart();
        Self {
            sig: Signals::default(),
            palette,
            fonts: FontChoice {
                display: setup.fonts.role(fh6_kit::FontRole::Display),
                text: setup.fonts.role(fh6_kit::FontRole::Text),
                wide: setup.fonts.role(fh6_kit::FontRole::Wide),
            },
            tokens_glass: t.glass(palette.glass_tint),
            rpm: Spring::new(t.springs.ribbon, 0.0),
            throttle: Spring::new(t.springs.snap, 0.0),
            brake: Spring::new(t.springs.snap, 0.0),
            light: VirtualLight::new(t),
            gear: GearGlyph::new(t),
            speed: SpeedReadout::new(),
            shift: ShiftLights::new(15),
            materialize,
            logo: Logo::default(),
            micro_cap: t.type_scale.micro_label,
            label_tracking: t.type_scale.label_tracking,
        }
    }

    fn tip(&self) -> Vec2 {
        Vec2::polar(CENTER, RADIUS, deg(START + SWEEP * self.rpm.value().clamp(0.0, 1.0)))
    }
}

impl Cluster for LiquidGlassSample {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt;
        self.sig = *input.signals;
        self.palette = *input.palette;
        self.fonts = input.fonts;
        self.tokens_glass = input.tokens.glass(self.palette.glass_tint);
        if self.logo != *input.logo {
            self.logo.clone_from(input.logo);
        }
        let rate = self.sig.rpm_rate / self.sig.max_rpm.max(1.0);
        self.rpm.update_tracking(self.sig.rpm_fraction, rate, dt);
        self.throttle.update(self.sig.throttle, dt);
        self.brake.update(self.sig.brake, dt);
        self.light.update(self.sig.g_lat, dt);
        self.gear.update(self.sig.gear, dt);
        self.speed.update(self.sig.speed);
        self.shift
            .update(&self.sig, &self.palette, input.t, dt, input.reduced_flash);
        self.materialize.update(dt);
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let pal = self.palette;
        let roles = pal.roles;
        let frac = self.rpm.value().clamp(0.0, 1.0);
        let tip = self.tip();

        // 1. Light behind the glass: a pool of the primary colour under the ribbon tip, so the
        //    frosted plate has something to lens and the light moves with the revs.
        p.fill(
            &Shape::circle(tip, 300.0),
            Gradient::radial(tip, 300.0)
                .stop(0.0, pal.primary.with_alpha(0.55))
                .stop(0.45, pal.primary.with_alpha(0.18))
                .stop(1.0, pal.primary.with_alpha(0.0)),
        );

        // 2. The glass plate (materialises by ramping lensing and rim, never opacity).
        let m = self.materialize.progress();
        let glass = Glass {
            lens: self.tokens_glass.lens * m,
            fresnel: self.tokens_glass.fresnel * m,
            light_angle: self.light.angle(),
            refraction: fh6_kit::Refraction::World,
            ..self.tokens_glass
        };
        let plate = Shape::round_rect(PLATE, 64.0);
        p.layer(LayerOptions::glass(18.0), |p| {
            p.fill(&plate, glass);
        });

        // 3. Ribbon tach: track, hairline ticks, then the luminous fill.
        p.fill(
            &Shape::arc(CENTER, RADIUS, THICKNESS, deg(START), deg(SWEEP), Cap::Round),
            roles.ink_ghost,
        );
        let thousands = (self.sig.max_rpm / 1000.0).floor().max(1.0);
        let tick_sweep = SWEEP * (thousands * 1000.0 / self.sig.max_rpm.max(1.0)).min(1.0);
        p.fill(
            &Shape::Ticks(
                TickRing::new(CENTER, RADIUS + 34.0, deg(START), deg(tick_sweep), thousands as u32 + 1)
                    .minor(10.0, 2.0)
                    .major(5, 16.0, 3.0),
            ),
            roles.hairline,
        );
        let stage = pal.stages[0] * self.sig.shift_rpm / self.sig.max_rpm.max(1.0);
        let ribbon = Gradient::conic(CENTER, deg(START))
            .stop(0.0, pal.primary.with_alpha(0.65))
            .stop(stage * 0.75, pal.primary)
            .stop(0.72, pal.accent)
            .stop(0.75, roles.warning)
            // Beyond the sweep (only the round start cap reaches it) wrap back to the start colour.
            .stop(0.88, roles.warning)
            .stop(0.88, pal.primary.with_alpha(0.65))
            .stop(1.0, pal.primary.with_alpha(0.65));
        p.layer(LayerOptions::bloom(), |p| {
            if frac > 0.002 {
                p.fill(
                    &Shape::arc(CENTER, RADIUS, THICKNESS, deg(START), deg(SWEEP * frac), Cap::Round),
                    ribbon,
                );
            }
            // Radial meniscus orb at the leading edge (P8-1: white -> accent -> clear).
            let orb = Gradient::radial(tip, 26.0)
                .stop(0.0, roles.ink)
                .stop(0.4, pal.accent.with_alpha(200.0 / 255.0))
                .stop(1.0, pal.primary.with_alpha(0.0));
            p.fill(&Shape::circle(tip, 26.0), orb);
        });
        p.glow(&Shape::circle(tip, 10.0), pal.accent, Glow::new(18.0, 0.55));

        // 4. Crystal gear prism: an octagon with an inner facet and a top-left chamfer highlight.
        let mut oct = [Vec2::ZERO; 8];
        for (i, v) in oct.iter_mut().enumerate() {
            *v = Vec2::polar(vec2(0.0, -10.0), 128.0, deg(22.5 + 45.0 * i as f32));
        }
        p.fill(&Shape::Polygon { points: &oct }, roles.oled_black.with_alpha(0.35));
        p.stroke(
            &Shape::Polyline {
                points: &oct,
                width: 2.0,
                closed: true,
            },
            pal.accent.with_alpha(130.0 / 255.0),
            Stroke::new(2.0),
        );
        p.fill(&Shape::line(oct[4], oct[5], 3.0), roles.rim_specular);
        p.layer(LayerOptions::bloom(), |p| {
            self.gear
                .draw(p, vec2(0.0, -10.0), self.fonts.display, 150.0, roles.ink);
        });

        // 5. Speed (tabular, ghost 8s) and its unit in the ribbon's open bottom.
        self.speed.draw(
            p,
            vec2(0.0, 160.0),
            self.fonts.display,
            64.0,
            roles.ink,
            roles.ink_ghost,
        );
        let label = TextStyle::new(self.fonts.text, self.micro_cap)
            .paint(roles.ink_label)
            .align(Align::Center, VAlign::CapMiddle)
            .tracking(self.label_tracking);
        p.text(vec2(0.0, 216.0), self.sig.unit.label(), &label);

        // 6. Wings: rpm readout (left), throttle and brake bars (right).
        let value = TextStyle::new(self.fonts.display, 40.0)
            .paint(roles.ink_muted)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        p.number(vec2(-390.0, -20.0), self.sig.rpm.round() as i64, 5, &value, None);
        p.text(vec2(-390.0, 30.0), "RPM", &label);
        for (i, (level, color)) in [(self.throttle.value(), roles.go), (self.brake.value(), roles.warning)]
            .into_iter()
            .enumerate()
        {
            let x = 360.0 + i as f32 * 40.0;
            let track = Rect::from_xywh(x, -120.0, 16.0, 200.0);
            p.fill(&Shape::round_rect(track, 8.0), roles.ink_ghost);
            let h = 200.0 * level.clamp(0.0, 1.0);
            if h > 1.0 {
                p.fill(&Shape::round_rect(Rect::from_xywh(x, 80.0 - h, 16.0, h), 8.0), color);
            }
        }
        p.text(vec2(380.0, 110.0), "THR  BRK", &label);

        // 7. Shift strip above the plate (<= 42 px tall, tokens) and the logo zone.
        self.shift.draw(p, Rect::from_xywh(-300.0, -300.0, 600.0, 22.0), &pal);
        fh6_kit::logo::draw(
            p,
            Rect::from_xywh(-500.0, 140.0, 220.0, 44.0),
            &self.logo,
            &fh6_kit::logo::LogoStyle {
                tint: roles.ink_label,
                text: roles.ink_label.into(),
                ..fh6_kit::logo::LogoStyle::new(self.fonts.wide)
            },
        );
    }

    fn animating(&self) -> bool {
        self.shift.animating()
            || self.gear.animating()
            || self.materialize.running()
            || !self.rpm.settled(self.sig.rpm_fraction, 1e-4)
    }
}
