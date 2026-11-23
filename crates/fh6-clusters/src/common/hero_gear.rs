//! HeroGear (COMPONENTS.md §1, rank 1): the gear glyph with its state colours and the tokens'
//! gear-change motion, plus the timing a family needs for its own gear-change flourish.
//!
//! - `1..10` in `ink`, `N` in `ink` at 72%, `R` in `warning`; never hidden.
//! - Change: the new glyph enters from 1.12 with a 24 px slide (up for an upshift) on the
//!   `fast` spring, with a 1-frame 60% rim flash ([`HeroGear::flash`]).
//! - Limiter: an optional 2 Hz pulse after 0.5 s on the limiter ([`HeroGear::pulse`]).
use super::GearGlyph;
use fh6_kit::{Color, FontId, Gear, Painter, Palette, Tokens, Vec2};

/// The hero gear state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeroGear {
    glyph: GearGlyph,
    gear: Gear,
    since_change: f32,
    upshift: bool,
    flash: f32,
    limiter_held: f32,
    pulse_t: f32,
}

fn rank(g: Gear) -> i32 {
    match g {
        Gear::Reverse => -1,
        Gear::Neutral => 0,
        Gear::Forward(n) => i32::from(n),
    }
}

impl HeroGear {
    /// In neutral, settled.
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            glyph: GearGlyph::new(tokens),
            gear: Gear::Neutral,
            since_change: 10.0,
            upshift: false,
            flash: 0.0,
            limiter_held: 0.0,
            pulse_t: 0.0,
        }
    }

    /// Advances with the gear and how long the limiter has been held (seconds, 0 when off; see
    /// `common::shift::LimiterCue::held`).
    pub fn update(&mut self, gear: Gear, limiter_held: f32, dt: f32) {
        let dt = dt.max(0.0);
        self.flash = 0.0;
        if gear != self.gear {
            self.upshift = rank(gear) > rank(self.gear);
            self.gear = gear;
            self.since_change = 0.0;
            self.flash = 0.6;
        } else {
            self.since_change += dt;
        }
        self.glyph.update(gear, dt);
        self.limiter_held = limiter_held.max(0.0);
        self.pulse_t = if self.limiter_held > 0.5 { self.pulse_t + dt } else { 0.0 };
    }

    /// The gear shown.
    #[must_use]
    pub fn gear(&self) -> Gear {
        self.gear
    }

    /// Seconds since the last gear change (large when none happened yet).
    #[must_use]
    pub fn since_change(&self) -> f32 {
        self.since_change
    }

    /// True if the last change was an upshift.
    #[must_use]
    pub fn was_upshift(&self) -> bool {
        self.upshift
    }

    /// The 1-frame rim flash (0.6 on the frame of a change, else 0).
    #[must_use]
    pub fn flash(&self) -> f32 {
        self.flash
    }

    /// The optional 2 Hz limiter pulse, 0..1 (0 until the limiter has been held for 0.5 s).
    #[must_use]
    pub fn pulse(&self) -> f32 {
        if self.pulse_t <= 0.0 {
            return 0.0;
        }
        0.5 - 0.5 * (self.pulse_t * std::f32::consts::TAU * 2.0).cos()
    }

    /// The glyph colour for the current gear.
    #[must_use]
    pub fn color(&self, palette: &Palette) -> Color {
        match self.gear {
            Gear::Reverse => palette.roles.warning,
            Gear::Neutral => palette.roles.ink.fade(0.72),
            Gear::Forward(_) => palette.roles.ink,
        }
    }

    /// True while the entry motion or the pulse runs.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.glyph.animating() || self.pulse_t > 0.0
    }

    /// Draws the glyph centred at `center` (cap middle) with cap height `cap` in its state
    /// colour. Put it in a bloom layer (rank 1).
    pub fn draw(&self, p: &mut Painter<'_>, center: Vec2, font: FontId, cap: f32, palette: &Palette) {
        self.glyph.draw(p, center, font, cap, self.color(palette));
    }

    /// Draws the glyph in an explicit colour (for keylines, engraved or chromatic copies).
    pub fn draw_with(&self, p: &mut Painter<'_>, center: Vec2, font: FontId, cap: f32, color: Color) {
        self.glyph.draw(p, center, font, cap, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_kit::{Palette, Tokens};

    #[test]
    fn change_flashes_one_frame_and_knows_its_direction() {
        let t = Tokens::default();
        let mut g = HeroGear::new(&t);
        g.update(Gear::Forward(3), 0.0, 1.0 / 60.0);
        assert!(g.was_upshift() && g.flash() > 0.5 && g.since_change() == 0.0);
        g.update(Gear::Forward(3), 0.0, 1.0 / 60.0);
        assert_eq!(g.flash(), 0.0);
        g.update(Gear::Forward(2), 0.0, 1.0 / 60.0);
        assert!(!g.was_upshift());
    }

    #[test]
    fn state_colours_and_pulse() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "liquidGlass");
        let mut g = HeroGear::new(&t);
        g.update(Gear::Reverse, 0.0, 0.016);
        assert_eq!(g.color(&pal), pal.roles.warning);
        g.update(Gear::Forward(5), 0.2, 0.016);
        assert_eq!(g.pulse(), 0.0, "no pulse before 0.5 s on the limiter");
        for _ in 0..20 {
            g.update(Gear::Forward(5), 0.8, 0.0125);
        }
        assert!(g.pulse() > 0.0);
    }
}
