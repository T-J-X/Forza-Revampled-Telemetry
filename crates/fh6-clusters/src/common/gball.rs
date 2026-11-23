//! GBall (COMPONENTS.md §11, tokens `components.gBall`): a friction-circle G meter.
//!
//! Rings at 0.5 / 1.0 / 1.5 g (1.0 g brighter, 1.5 g dashed with short arcs), a hairline
//! crosshair, the ball showing the force the driver feels (`-acceleration`, a ball in a bowl:
//! acceleration to the right pushes it left, braking pushes it forward/up) on the tokens `gBall`
//! spring, a 500 ms trail of fading dots in a fixed ring buffer, and a total-g readout. Rank 4:
//! no glow, no bloom.
use crate::common::race::draw_fixed;
use fh6_kit::math::finite_or;
use fh6_kit::{
    vec2, Align, Cap, FontId, Gradient, Hysteresis, Painter, Palette, Shape, Signals, Spring, Stroke, TextStyle, Tokens,
    VAlign, Vec2,
};
use std::f32::consts::TAU;

/// Ring radii in g (tokens `components.gBall.ringsG`).
pub const RINGS_G: [f32; 3] = [0.5, 1.0, 1.5];
/// Trail length in seconds (tokens `components.gBall.trailMs`).
pub const TRAIL_SECONDS: f32 = 0.5;
/// Trail dots kept in the ring buffer (one every 25 ms).
pub const TRAIL_DOTS: usize = 20;
/// Time between trail samples.
const TRAIL_STEP: f32 = TRAIL_SECONDS / TRAIL_DOTS as f32;
/// The ball may run 10% past the outer ring before it is held at the rim.
const MAX_G: f32 = 1.65;
/// Dashes in the 1.5 g ring.
const DASHES: usize = 36;
/// Fraction of each dash period that is drawn.
const DASH_DUTY: f32 = 0.42;
/// Ball radius as a fraction of the outer ring radius.
const BALL_PCT: f32 = 0.075;
/// Ring and crosshair line width in local units (one device pixel at 4K, scale 1.0).
const LINE_W: f32 = 2.0;
/// Readout precision (hundredths of a g).
const READOUT_DECIMALS: u8 = 2;

/// The felt force for lateral and longitudinal G, in g, screen convention (x right, y down):
/// `(-g_lat, g_long)`, so acceleration to the right pushes the ball left and braking
/// (`g_long < 0`) pushes it up. Clamped to just past the outer ring; non-finite input is 0.
#[must_use]
pub fn felt_force(g_lat: f32, g_long: f32) -> Vec2 {
    let v = vec2(-finite_or(g_lat, 0.0), finite_or(g_long, 0.0));
    let len = v.length();
    if len > MAX_G {
        v * (MAX_G / len)
    } else {
        v
    }
}

/// The G-ball state: ball springs, the trail ring buffer and the readout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GBall {
    x: Spring,
    y: Spring,
    target: Vec2,
    trail: [Vec2; TRAIL_DOTS],
    head: usize,
    len: usize,
    since_sample: f32,
    readout: Hysteresis,
    total: i64,
}

impl GBall {
    /// A ball at rest in the centre, on the tokens' `gBall` spring.
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            x: Spring::new(tokens.springs.g_ball, 0.0),
            y: Spring::new(tokens.springs.g_ball, 0.0),
            target: Vec2::ZERO,
            trail: [Vec2::ZERO; TRAIL_DOTS],
            head: 0,
            len: 0,
            since_sample: 0.0,
            readout: Hysteresis::new(0.0, 0.6),
            total: 0,
        }
    }

    /// Advances with this frame's smoothed G (`Signals::g_lat`, `Signals::g_long`).
    pub fn update(&mut self, sig: &Signals, dt: f32) {
        self.update_g(sig.g_lat, sig.g_long, dt);
    }

    /// Advances with lateral G (positive = accelerating to the right) and longitudinal G
    /// (positive = accelerating forward).
    pub fn update_g(&mut self, g_lat: f32, g_long: f32, dt: f32) {
        let dt = finite_or(dt, 0.0).max(0.0);
        self.target = felt_force(g_lat, g_long);
        self.x.update(self.target.x, dt);
        self.y.update(self.target.y, dt);
        let total = vec2(finite_or(g_lat, 0.0), finite_or(g_long, 0.0)).length();
        self.total = self.readout.update(total * 100.0);

        self.since_sample += dt;
        let ball = self.ball();
        let mut pushed = 0;
        while self.since_sample >= TRAIL_STEP && pushed < TRAIL_DOTS {
            self.head = (self.head + 1) % TRAIL_DOTS;
            self.trail[self.head] = ball;
            self.len = (self.len + 1).min(TRAIL_DOTS);
            self.since_sample -= TRAIL_STEP;
            pushed += 1;
        }
        self.since_sample = self.since_sample.min(TRAIL_STEP);
    }

    /// The ball position in g (screen convention: x right, y down).
    #[must_use]
    pub fn ball(&self) -> Vec2 {
        vec2(self.x.value(), self.y.value())
    }

    /// The total G shown by the readout, in hundredths of a g.
    #[must_use]
    pub fn total_centi_g(&self) -> i64 {
        self.total
    }

    /// True while the ball moves or the trail has not yet collapsed onto it.
    #[must_use]
    pub fn animating(&self) -> bool {
        const EPS: f32 = 1e-3;
        if !self.x.settled(self.target.x, EPS) || !self.y.settled(self.target.y, EPS) {
            return true;
        }
        let ball = self.ball();
        self.trail_iter().any(|(p, _)| (p - ball).length() > EPS)
    }

    /// Trail sample `k` (0 = newest) and its age in seconds.
    fn sample(&self, k: usize) -> (Vec2, f32) {
        let idx = (self.head + TRAIL_DOTS - k % TRAIL_DOTS) % TRAIL_DOTS;
        (self.trail[idx], k as f32 * TRAIL_STEP + self.since_sample)
    }

    /// Trail samples newest first, with their age in seconds.
    fn trail_iter(&self) -> impl Iterator<Item = (Vec2, f32)> + '_ {
        (0..self.len).map(move |k| self.sample(k))
    }

    /// Draws the meter and the readout: the outer (1.5 g) ring has radius `radius` around
    /// `center`; the readout sits under it (allow `radius * 0.1 + 2.2 x metricSecondary`).
    pub fn draw(
        &self,
        p: &mut Painter<'_>,
        center: Vec2,
        radius: f32,
        palette: &Palette,
        value_font: FontId,
        label_font: FontId,
    ) {
        self.draw_meter(p, center, radius, palette);
        let cap = p.tokens().type_scale.metric_secondary;
        let baseline = center.y + radius + radius * 0.1 + cap * 1.2;
        self.draw_readout(p, vec2(center.x, baseline), palette, value_font, label_font);
    }

    /// Draws rings, crosshair, trail and ball (no readout).
    pub fn draw_meter(&self, p: &mut Painter<'_>, center: Vec2, radius: f32, palette: &Palette) {
        let roles = palette.roles;
        let per_g = radius / RINGS_G[2];

        // Crosshair: hairline, running just past the outer ring like a reticle.
        let reach = radius * 1.06;
        p.fill(
            &Shape::line(center - vec2(reach, 0.0), center + vec2(reach, 0.0), LINE_W),
            roles.hairline,
        );
        p.fill(
            &Shape::line(center - vec2(0.0, reach), center + vec2(0.0, reach), LINE_W),
            roles.hairline,
        );

        // 0.5 g: hairline; 1.0 g: the reference ring, brighter; 1.5 g: dashed.
        p.fill(&Shape::ring(center, RINGS_G[0] * per_g, LINE_W), roles.hairline);
        p.fill(&Shape::ring(center, RINGS_G[1] * per_g, LINE_W), roles.ink_label);
        let step = TAU / DASHES as f32;
        let dash = step * DASH_DUTY;
        for i in 0..DASHES {
            // Dashes centred on the axes so the crosshair passes through a dash, not a gap.
            let a = step * i as f32 - dash * 0.5;
            p.fill(
                &Shape::arc(center, radius, LINE_W, a, dash, Cap::Butt),
                roles.hairline,
            );
        }

        // Trail: dots fade and shrink with age, drawn oldest first.
        let ball_r = radius * BALL_PCT;
        for k in (0..self.len).rev() {
            let (at, age) = self.sample(k);
            let life = (1.0 - age / TRAIL_SECONDS).clamp(0.0, 1.0);
            if life <= 0.0 {
                continue;
            }
            let pos = center + at * per_g;
            p.fill(
                &Shape::circle(pos, ball_r * (0.22 + 0.38 * life)),
                palette.primary.fade(0.55 * life * life),
            );
        }

        // Ball: spherical shading from the virtual light (high, left), a 1 px top-lit rim.
        let pos = center + self.ball() * per_g;
        let ball = Shape::circle(pos, ball_r);
        let hi = pos + vec2(-0.38, -0.42) * ball_r;
        p.fill(
            &ball,
            Gradient::radial(hi, ball_r * 1.7)
                .stop(0.0, palette.primary.lerp(roles.ink, 0.62))
                .stop(0.45, palette.primary)
                .stop(1.0, palette.primary.intensity(0.55)),
        );
        p.stroke(
            &ball,
            Gradient::linear(pos - vec2(0.0, ball_r), pos + vec2(0.0, ball_r))
                .stop(0.0, roles.rim_specular)
                .stop(0.6, roles.rim_specular.fade(0.0)),
            Stroke::rim_px(1.0),
        );
    }

    /// Draws the total-g readout (`1.02 G`) with its baseline at `pos`, centred on the point.
    pub fn draw_readout(
        &self,
        p: &mut Painter<'_>,
        pos: Vec2,
        palette: &Palette,
        value_font: FontId,
        label_font: FontId,
    ) {
        let ts = p.tokens().type_scale;
        let value = TextStyle::new(value_font, ts.metric_secondary)
            .paint(palette.roles.ink_muted)
            .align(Align::Right, VAlign::Baseline)
            .tabular();
        let unit = TextStyle::new(label_font, ts.micro_label)
            .paint(palette.roles.ink_label)
            .tracking(ts.label_tracking)
            .align(Align::Left, VAlign::Baseline);
        let gap = ts.micro_label * 0.35;
        // The value's right edge sits right of centre so "0.00 G" reads centred as a unit.
        let split = pos.x + ts.metric_secondary * 0.45;
        draw_fixed(p, vec2(split, pos.y), self.total, READOUT_DECIMALS, &value);
        p.text(vec2(split + gap, pos.y), "G", &unit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn ball_shows_the_felt_force() {
        // Accelerating to the right pushes the ball left; braking pushes it up (forward).
        let f = felt_force(0.8, -0.5);
        assert!(f.x < -0.79 && f.y < -0.49, "{f:?}");
        // Accelerating forward pushes the ball down (back).
        assert!(felt_force(0.0, 0.4).y > 0.39);
        // Held just past the outer ring.
        assert!((felt_force(3.0, 4.0).length() - MAX_G).abs() < 1e-4);
        assert_eq!(felt_force(f32::NAN, f32::INFINITY), Vec2::ZERO);
    }

    #[test]
    fn ball_follows_the_gball_spring_and_settles() {
        let t = Tokens::default();
        let mut g = GBall::new(&t);
        g.update_g(-0.96, -0.58, DT);
        let first = g.ball();
        assert!(first.x > 0.0 && first.x < 0.96, "springs, not a jump: {first:?}");
        for _ in 0..60 {
            g.update_g(-0.96, -0.58, DT);
        }
        let b = g.ball();
        assert!((b.x - 0.96).abs() < 0.01 && (b.y + 0.58).abs() < 0.01, "{b:?}");
        assert_eq!(g.total_centi_g(), 112);
        assert!(!g.animating(), "trail collapsed after 1 s of constant G");
    }

    #[test]
    fn trail_covers_500_ms_at_any_frame_rate() {
        let t = Tokens::default();
        for hz in [60.0f32, 120.0, 144.0] {
            let mut g = GBall::new(&t);
            for _ in 0..(hz as usize) {
                g.update_g(0.5, 0.0, 1.0 / hz);
            }
            assert_eq!(g.len, TRAIL_DOTS, "{hz} Hz");
            let oldest = g.trail_iter().last().map_or(0.0, |(_, age)| age);
            assert!(oldest <= TRAIL_SECONDS + 1e-3, "{hz} Hz: oldest {oldest}");
        }
        // A huge frame never loops more than the buffer.
        let mut g = GBall::new(&t);
        g.update_g(0.5, 0.0, 10.0);
        assert_eq!(g.len, TRAIL_DOTS);
    }

    #[test]
    fn trail_moves_while_the_ball_moves() {
        let t = Tokens::default();
        let mut g = GBall::new(&t);
        for i in 0..30 {
            g.update_g((i as f32 * 0.2).sin(), 0.0, DT);
        }
        assert!(g.animating());
        let spread = g.trail_iter().map(|(p, _)| p.x).fold(f32::NEG_INFINITY, f32::max)
            - g.trail_iter().map(|(p, _)| p.x).fold(f32::INFINITY, f32::min);
        assert!(spread > 0.1, "trail spread {spread}");
    }
}
