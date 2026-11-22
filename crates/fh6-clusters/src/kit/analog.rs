//! Analog sample (kit showcase, family `classicAnalog`): a GT3-style tach dial with a brushed
//! bezel lit by the virtual light, a crisp chapter ring and redline sector, DIN numerals, a
//! tapered needle on the tokens' needle spring with feed-forward, an illuminated hub, a
//! sequential shift ring and a digital gear and speed inset.
//!
//! Not the final design (Phase 7); it proves needles, ticks, conic and radial paints, tabular
//! digits and spring feel.
use crate::common::{GearGlyph, ShiftLights, SpeedReadout, VirtualLight};
use fh6_kit::cluster::Setup;
use fh6_kit::{
    deg, vec2, Align, Cap, Cluster, FontChoice, FrameInput, Glow, Gradient, LayerOptions, Layout, Painter, Palette,
    Rect, Shape, Signals, Spring, Stroke, TextStyle, TickRing, Tokens, VAlign, Vec2,
};

/// Dial centre and radius (local 4K px).
const CENTER: Vec2 = vec2(0.0, -30.0);
const RADIUS: f32 = 260.0;
/// Scale start (7:30) and sweep, degrees.
const START: f32 = 135.0;
const SWEEP: f32 = 270.0;

/// The analog sample cluster.
#[derive(Debug, Clone)]
pub struct AnalogSample {
    sig: Signals,
    palette: Palette,
    fonts: FontChoice,
    needle: Spring,
    light: VirtualLight,
    gear: GearGlyph,
    speed: SpeedReadout,
    shift: ShiftLights,
    dial_cap: f32,
    micro_cap: f32,
    label_tracking: f32,
    dial_font: fh6_kit::FontId,
}

impl AnalogSample {
    /// Builds the sample.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let t: &Tokens = setup.tokens;
        Self {
            sig: Signals::default(),
            palette: Palette::family(t, "classicAnalog"),
            fonts: FontChoice {
                display: setup.fonts.role(fh6_kit::FontRole::Display),
                text: setup.fonts.role(fh6_kit::FontRole::Text),
                wide: setup.fonts.role(fh6_kit::FontRole::Wide),
            },
            needle: Spring::new(t.springs.needle, 0.0),
            light: VirtualLight::new(t),
            gear: GearGlyph::new(t),
            speed: SpeedReadout::new(),
            shift: ShiftLights::new(11),
            dial_cap: t.type_scale.dial_numeral,
            micro_cap: t.type_scale.micro_label,
            label_tracking: t.type_scale.label_tracking,
            dial_font: setup.fonts.role(fh6_kit::FontRole::Dial),
        }
    }

    /// Full-scale rpm: the limiter rounded up to the next 1000.
    fn scale_rpm(&self) -> f32 {
        ((self.sig.max_rpm / 1000.0).ceil() * 1000.0).max(1000.0)
    }

    fn angle_of(&self, rpm: f32) -> f32 {
        deg(START + SWEEP * (rpm / self.scale_rpm()).clamp(0.0, 1.02))
    }
}

impl Cluster for AnalogSample {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt;
        self.sig = *input.signals;
        self.palette = *input.palette;
        self.fonts = input.fonts;
        self.needle.update_tracking(self.sig.rpm, self.sig.rpm_rate, dt);
        self.light.update(self.sig.g_lat, dt);
        self.gear.update(self.sig.gear, dt);
        self.speed.update(self.sig.speed);
        self.shift
            .update(&self.sig, &self.palette, input.t, dt, input.reduced_flash);
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let pal = self.palette;
        let roles = pal.roles;
        let light = self.light.angle();

        // Bezel: brushed metal as a conic of ink at low alpha, its 1 px rim lit by the light.
        let brushed = Gradient::conic(CENTER, light)
            .stop(0.0, roles.ink.with_alpha(0.42))
            .stop(0.125, roles.ink.with_alpha(0.12))
            .stop(0.25, roles.ink.with_alpha(0.28))
            .stop(0.5, roles.ink.with_alpha(0.08))
            .stop(0.75, roles.ink.with_alpha(0.22))
            .stop(1.0, roles.ink.with_alpha(0.42));
        p.fill(&Shape::ring(CENTER, RADIUS + 8.0, 16.0), brushed);
        let rim_dir = Vec2::from_angle(light);
        let rim = Gradient::linear(CENTER + rim_dir * RADIUS, CENTER - rim_dir * RADIUS)
            .stop(0.0, roles.rim_specular)
            .stop(0.6, roles.hairline)
            .stop(1.0, roles.hairline.with_alpha(0.05));
        p.stroke(&Shape::circle(CENTER, RADIUS + 16.0), rim, Stroke::rim_px(1.0));

        // Face: OLED black with a faint dome of light towards the centre.
        p.fill(
            &Shape::circle(CENTER, RADIUS),
            Gradient::radial(CENTER, RADIUS)
                .stop(0.0, roles.ink.with_alpha(0.06))
                .stop(0.7, roles.ink.with_alpha(0.02))
                .stop(1.0, roles.oled_black),
        );

        // Chapter ring: minor every 250 rpm, major every 1000; redline sector.
        let scale = self.scale_rpm();
        let majors = (scale / 1000.0) as u32;
        p.fill(
            &Shape::Ticks(
                TickRing::new(CENTER, RADIUS - 12.0, deg(START), deg(SWEEP), majors * 4 + 1)
                    .minor(14.0, 2.0)
                    .major(4, 34.0, 5.0),
            ),
            roles.ink,
        );
        let red_from = self.angle_of(self.sig.shift_rpm);
        let red_to = self.angle_of(scale);
        p.fill(
            &Shape::arc(CENTER, RADIUS - 52.0, 8.0, red_from, red_to - red_from, Cap::Butt),
            roles.warning,
        );

        // DIN numerals in thousands.
        let numeral = TextStyle::new(self.dial_font, self.dial_cap)
            .paint(roles.ink)
            .align(Align::Center, VAlign::CapMiddle);
        const DIGITS: [&str; 13] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"];
        for (k, digit) in DIGITS.iter().enumerate().take(majors as usize + 1) {
            let a = self.angle_of(k as f32 * 1000.0);
            p.text(Vec2::polar(CENTER, RADIUS - 92.0, a), digit, &numeral);
        }
        let label = TextStyle::new(self.fonts.text, self.micro_cap)
            .paint(roles.ink_label)
            .align(Align::Center, VAlign::CapMiddle)
            .tracking(self.label_tracking);
        p.text(CENTER + vec2(0.0, -110.0), "x1000 RPM", &label);

        // Digital inset in the open bottom of the scale: gear pod and speed.
        let pod = Rect::from_center_size(CENTER + vec2(0.0, 120.0), vec2(120.0, 104.0));
        p.fill(&Shape::round_rect(pod, 18.0), roles.ink_ghost);
        p.stroke(&Shape::round_rect(pod, 18.0), roles.hairline, Stroke::rim_px(1.0));
        p.layer(LayerOptions::bloom(), |p| {
            self.gear
                .draw(p, pod.center(), self.fonts.display, 72.0, pal.primary_text);
        });
        self.speed.draw(
            p,
            CENTER + vec2(0.0, 214.0),
            self.fonts.display,
            30.0,
            roles.ink,
            roles.ink_ghost,
        );
        p.text(
            CENTER + vec2(78.0, 214.0),
            self.sig.unit.label(),
            &label.align(Align::Left, VAlign::CapMiddle),
        );

        // Needle with glow and an illuminated hub (glance rank 3: it blooms).
        let a = self.angle_of(self.needle.value());
        let needle = Shape::needle(CENTER, a, RADIUS - 24.0, 44.0, 14.0, 3.0);
        p.shadow(
            &needle,
            fh6_kit::Shadow::new(vec2(4.0, 6.0), 6.0, roles.oled_black.with_alpha(0.6)),
        );
        p.layer(LayerOptions::bloom(), |p| {
            p.glow(&needle, pal.primary, Glow::new(7.0, 0.45));
            p.fill(&needle, pal.primary_text);
        });
        p.fill(
            &Shape::circle(CENTER, 30.0),
            Gradient::radial(CENTER + rim_dir * 10.0, 34.0)
                .stop(0.0, roles.ink.with_alpha(0.55))
                .stop(1.0, roles.oled_black),
        );
        p.stroke(&Shape::circle(CENTER, 30.0), roles.rim_specular, Stroke::rim_px(1.0));
        p.layer(LayerOptions::bloom(), |p| {
            p.fill(&Shape::circle(CENTER, 9.0), pal.primary_text);
        });

        // Sequential shift ring across the top of the bezel.
        self.shift
            .draw_arc(p, CENTER, RADIUS + 30.0, 10.0, deg(-90.0 - 36.0), deg(72.0), &pal);
    }

    fn animating(&self) -> bool {
        self.shift.animating() || self.gear.animating() || !self.needle.settled(self.sig.rpm, 0.5)
    }
}
