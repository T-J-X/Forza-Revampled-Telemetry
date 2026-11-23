//! SignedBar (COMPONENTS.md §13, tokens `components.signedBar`): an F1 MGU-K style bar above
//! the gear.
//!
//! It grows from the centre: right for acceleration / deploy in `go`, left for braking in
//! `warning`. Its height is 8–12% of the gear cap, it follows the tokens `fast` spring, and it
//! fades out near zero so a coasting car shows nothing but a ghost track. Rank 4: no glow.
use fh6_kit::math::{finite_or, smoothstep};
use fh6_kit::{vec2, Gradient, Painter, Palette, Radii, Rect, Shape, Signals, Spring, Stroke, Tokens, Vec2};

/// Height range as a fraction of the gear cap (tokens `components.signedBar.heightPctOfGear`).
pub const HEIGHT_PCT_RANGE: (f32, f32) = (0.08, 0.12);
/// Default height as a fraction of the gear cap.
pub const DEFAULT_HEIGHT_PCT: f32 = 0.10;
/// Below this magnitude the fill is invisible.
const FADE_START: f32 = 0.02;
/// From this magnitude the fill is fully opaque.
const FADE_FULL: f32 = 0.10;

/// What the bar shows, as a signed fraction of full scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignedSource {
    /// Longitudinal G (verified acceleration channel), full scale at `full_scale_g`.
    LongitudinalG {
        /// The G that fills one half of the bar.
        full_scale_g: f32,
    },
    /// Throttle minus brake (verified pedal bytes): deploy right, harvest left.
    Pedals,
}

impl SignedSource {
    /// The signed fraction (-1..=1) this source reads from `sig`.
    #[must_use]
    pub fn sample(&self, sig: &Signals) -> f32 {
        let v = match *self {
            SignedSource::LongitudinalG { full_scale_g } => sig.g_long / full_scale_g.abs().max(0.05),
            SignedSource::Pedals => sig.throttle - sig.brake,
        };
        finite_or(v, 0.0).clamp(-1.0, 1.0)
    }
}

/// The signed bar state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignedBar {
    source: SignedSource,
    spring: Spring,
    target: f32,
    height_pct: f32,
}

impl SignedBar {
    /// A bar at zero reading `source`, on the tokens' `fast` spring.
    #[must_use]
    pub fn new(tokens: &Tokens, source: SignedSource) -> Self {
        Self {
            source,
            spring: Spring::new(tokens.springs.fast, 0.0),
            target: 0.0,
            height_pct: DEFAULT_HEIGHT_PCT,
        }
    }

    /// With a height as a fraction of the gear cap, clamped to [`HEIGHT_PCT_RANGE`].
    #[must_use]
    pub fn with_height_pct(mut self, pct: f32) -> Self {
        self.height_pct = finite_or(pct, DEFAULT_HEIGHT_PCT).clamp(HEIGHT_PCT_RANGE.0, HEIGHT_PCT_RANGE.1);
        self
    }

    /// Advances with this frame's signals through the bar's source.
    pub fn update(&mut self, sig: &Signals, dt: f32) {
        let v = self.source.sample(sig);
        self.update_value(v, dt);
    }

    /// Advances towards a caller-computed signed fraction (-1..=1).
    pub fn update_value(&mut self, value: f32, dt: f32) {
        self.target = finite_or(value, 0.0).clamp(-1.0, 1.0);
        self.spring.update(self.target, dt);
    }

    /// The sprung value, -1..=1.
    #[must_use]
    pub fn value(&self) -> f32 {
        self.spring.value().clamp(-1.0, 1.0)
    }

    /// The fill opacity: 0 near zero, 1 from a tenth of full scale.
    #[must_use]
    pub fn opacity(&self) -> f32 {
        smoothstep(FADE_START, FADE_FULL, self.value().abs())
    }

    /// The bar height for a gear of cap height `gear_cap`.
    #[must_use]
    pub fn height(&self, gear_cap: f32) -> f32 {
        gear_cap * self.height_pct
    }

    /// True while the spring moves.
    #[must_use]
    pub fn animating(&self) -> bool {
        !self.spring.settled(self.target, 1e-3)
    }

    /// Draws the bar centred on `center` (place it above the gear), `half_width` to each side,
    /// sized from the gear cap `gear_cap`.
    pub fn draw(&self, p: &mut Painter<'_>, center: Vec2, half_width: f32, gear_cap: f32, palette: &Palette) {
        let roles = palette.roles;
        let h = self.height(gear_cap);
        let r = h * 0.28;
        let track = Rect::from_xywh(center.x - half_width, center.y - h * 0.5, half_width * 2.0, h);
        // Unlit track: a ghost lens that fades towards both ends, and the zero notch.
        p.fill(
            &Shape::round_rect(track, r),
            Gradient::linear(track.at(0.0, 0.5), track.at(1.0, 0.5))
                .stop(0.0, roles.ink_ghost.fade(0.3))
                .stop(0.5, roles.ink_ghost)
                .stop(1.0, roles.ink_ghost.fade(0.3)),
        );
        let notch_w = 2.0 * p.device_px();
        p.fill(
            &Shape::rect(Rect::from_center_size(center, vec2(notch_w, h * 1.7))),
            roles.hairline,
        );

        let a = self.opacity();
        if a <= 0.0 {
            return;
        }
        let v = self.value();
        let len = v.abs() * half_width;
        let (color, fill, radii, tip_x) = if v >= 0.0 {
            (
                roles.go,
                Rect::from_xywh(center.x, track.min.y, len, h),
                Radii::new(0.0, r, r, 0.0),
                center.x + len,
            )
        } else {
            (
                roles.warning,
                Rect::from_xywh(center.x - len, track.min.y, len, h),
                Radii::new(r, 0.0, 0.0, r),
                center.x - len,
            )
        };
        let shape = Shape::round_rect(fill, radii);
        // Intensity grows from the centre to the tip: the bar reads as energy flowing outwards.
        p.fill(
            &shape,
            Gradient::linear(vec2(center.x, center.y), vec2(tip_x, center.y))
                .stop(0.0, color.fade(0.38 * a))
                .stop(0.7, color.fade(0.85 * a))
                .stop(1.0, color.fade(a)),
        );
        // Top-lit 1 px rim.
        p.stroke(
            &shape,
            Gradient::linear(fill.at(0.5, 0.0), fill.at(0.5, 1.0))
                .stop(0.0, roles.rim_specular.fade(0.55 * a))
                .stop(0.6, roles.rim_specular.fade(0.0)),
            Stroke::rim_px(1.0),
        );
        // Leading edge: a hot line at the tip.
        let edge_w = (h * 0.16).max(notch_w).min(len);
        let edge_x = if v >= 0.0 { tip_x - edge_w } else { tip_x };
        p.fill(
            &Shape::round_rect(Rect::from_xywh(edge_x, track.min.y, edge_w, h), edge_w * 0.5),
            color.lerp(roles.ink, 0.55).fade(a),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn settle(bar: &mut SignedBar, sig: &Signals) {
        for _ in 0..90 {
            bar.update(sig, DT);
        }
    }

    #[test]
    fn longitudinal_g_sign_and_scale() {
        let t = Tokens::default();
        let mut bar = SignedBar::new(&t, SignedSource::LongitudinalG { full_scale_g: 1.0 });
        let braking = Signals {
            g_long: -0.58,
            ..Signals::default()
        };
        settle(&mut bar, &braking);
        assert!((bar.value() + 0.58).abs() < 0.01, "{}", bar.value());
        assert!(!bar.animating());
        let launch = Signals {
            g_long: 2.4,
            ..Signals::default()
        };
        settle(&mut bar, &launch);
        assert!((bar.value() - 1.0).abs() < 1e-3, "clamped to full scale");
    }

    #[test]
    fn pedals_source() {
        let sig = Signals {
            throttle: 0.3,
            brake: 0.62,
            ..Signals::default()
        };
        assert!((SignedSource::Pedals.sample(&sig) + 0.32).abs() < 1e-5);
    }

    #[test]
    fn follows_the_fast_spring_not_a_jump() {
        let t = Tokens::default();
        let mut bar = SignedBar::new(&t, SignedSource::Pedals);
        bar.update_value(1.0, DT);
        assert!(bar.value() > 0.0 && bar.value() < 0.5, "{}", bar.value());
        assert!(bar.animating());
    }

    #[test]
    fn fades_out_near_zero() {
        let t = Tokens::default();
        let mut bar = SignedBar::new(&t, SignedSource::Pedals);
        bar.update_value(0.01, 1.0);
        assert_eq!(bar.opacity(), 0.0);
        bar.update_value(0.05, 1.0);
        let mid = bar.opacity();
        assert!(mid > 0.0 && mid < 1.0, "{mid}");
        bar.update_value(-0.4, 1.0);
        assert_eq!(bar.opacity(), 1.0);
    }

    #[test]
    fn height_stays_within_the_token_range() {
        let t = Tokens::default();
        let bar = SignedBar::new(&t, SignedSource::Pedals);
        assert!((bar.height(236.0) - 23.6).abs() < 1e-3);
        assert!((bar.with_height_pct(0.5).height(200.0) - 24.0).abs() < 1e-3);
        assert!((bar.with_height_pct(0.0).height(200.0) - 16.0).abs() < 1e-3);
    }
}
