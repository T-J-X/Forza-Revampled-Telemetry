//! Components shared by the kit clusters (tokens `components`, `docs/design/COMPONENTS.md`).
//!
//! Each component keeps its own animation state (`update`) and draws with the kit (`draw`), so a
//! cluster composes them in a few lines and every family gets the same token behaviour.
use fh6_kit::{
    vec2, Align, Color, FontId, Gear, LayerOptions, Painter, Palette, Rect, Shape, Signals, Spring, SpringSpec, Strobe,
    TextStyle, Tokens, VAlign,
};

/// Sequential shift lights (tokens `colorRoles.revStages`, `motion.shiftLights`,
/// `motion.limiterStrobe`): fill outside-in onto the centre pod, 0 ms attack, <= 50 ms release,
/// unlit pods neutral, the limiter strobes 10 Hz on the strip only.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShiftLights {
    /// Number of pods (odd, so there is a centre pod).
    pub count: u32,
    level: f32,
    over_shift: bool,
    strobe_on: bool,
    breath: f32,
}

/// Release time constant of the shift lights (tokens: release <= 50 ms).
const RELEASE_TAU: f32 = 0.025;

impl ShiftLights {
    /// A strip of `count` pods.
    #[must_use]
    pub fn new(count: u32) -> Self {
        Self {
            count: count.max(3) | 1,
            level: 0.0,
            over_shift: false,
            strobe_on: false,
            breath: 1.0,
        }
    }

    /// Advances with this frame's signals.
    pub fn update(&mut self, sig: &Signals, palette: &Palette, t: f32, dt: f32, reduced_flash: bool) {
        let start = palette.stages[0];
        let target = ((sig.shift_fraction - start) / (1.0 - start).max(1e-3)).clamp(0.0, 1.0);
        if target >= self.level {
            self.level = target;
        } else {
            let k = 1.0 - (-dt.max(0.0) / RELEASE_TAU).exp();
            self.level += (target - self.level) * k;
        }
        self.over_shift = sig.shift_fraction >= 1.0;
        self.strobe_on = Strobe::LIMITER.on(t);
        self.breath = Strobe::breath(t);
        if reduced_flash {
            self.strobe_on = true;
        } else {
            self.breath = 1.0;
        }
    }

    /// True while the limiter strobe runs (the host keeps rendering between packets).
    #[must_use]
    pub fn animating(&self) -> bool {
        self.over_shift || (self.level > 0.0 && self.level < 1.0)
    }

    /// The colour of pod `i`, or `None` when it is unlit (or strobed off).
    fn pod_color(&self, i: usize, palette: &Palette) -> Option<Color> {
        let half = (self.count / 2) as f32;
        let k = (i as f32 - half).abs();
        // 0 for the outermost pods, 1 for the centre pod: pods light outside-in.
        let rank = 1.0 - k / half.max(1.0);
        let lit = self.level > 0.0 && self.level >= rank - 1e-4;
        if !lit {
            return None;
        }
        if self.over_shift {
            return self.strobe_on.then(|| palette.shift.fade(self.breath));
        }
        let start = palette.stages[0];
        Some(palette.rev_stage(start + rank * (1.0 - start)))
    }

    /// Draws the strip in `rect` (pods are pills; lit pods bloom).
    pub fn draw(&self, p: &mut Painter<'_>, rect: Rect, palette: &Palette) {
        let n = self.count as usize;
        let gap = rect.height() * 0.45;
        let w = (rect.width() - gap * (n as f32 - 1.0)) / n as f32;
        let r = rect.height() * 0.5;
        for i in 0..n {
            let pod = Rect::from_xywh(rect.min.x + i as f32 * (w + gap), rect.min.y, w, rect.height());
            match self.pod_color(i, palette) {
                Some(color) => p.layer(LayerOptions::bloom(), |p| p.fill(&Shape::round_rect(pod, r), color)),
                None => p.fill(&Shape::round_rect(pod, r), palette.roles.ink_ghost),
            }
        }
    }

    /// Draws the pods as a sequential shift ring: arc segments around `center` from `start`
    /// over `sweep` (radians, clockwise).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_arc(
        &self,
        p: &mut Painter<'_>,
        center: fh6_kit::Vec2,
        radius: f32,
        thickness: f32,
        start: f32,
        sweep: f32,
        palette: &Palette,
    ) {
        let n = self.count as usize;
        let step = sweep / n as f32;
        let seg = step * 0.72;
        for i in 0..n {
            let a = start + step * i as f32 + (step - seg) * 0.5;
            let shape = Shape::arc(center, radius, thickness, a, seg, fh6_kit::Cap::Butt);
            match self.pod_color(i, palette) {
                Some(color) => p.layer(LayerOptions::bloom(), |p| p.fill(&shape, color)),
                None => p.fill(&shape, palette.roles.ink_ghost),
            }
        }
    }
}

/// The hero gear glyph with the tokens' gear-change motion: the new glyph enters from 1.12 with
/// a 24 px slide (up for an upshift) on the `fast` spring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GearGlyph {
    gear: Gear,
    slide: Spring,
}

impl GearGlyph {
    /// A glyph for the `fast` spring of `tokens`.
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            gear: Gear::Neutral,
            slide: Spring::new(tokens.springs.fast, 0.0),
        }
    }

    fn rank(g: Gear) -> i32 {
        match g {
            Gear::Reverse => -1,
            Gear::Neutral => 0,
            Gear::Forward(n) => i32::from(n),
        }
    }

    /// Advances; starts the entry motion when the gear changes.
    pub fn update(&mut self, gear: Gear, dt: f32) {
        if gear != self.gear {
            let up = Self::rank(gear) > Self::rank(self.gear);
            self.slide.reset(if up { 1.0 } else { -1.0 });
            self.gear = gear;
        }
        self.slide.update(0.0, dt);
    }

    /// True while the entry motion runs.
    #[must_use]
    pub fn animating(&self) -> bool {
        !self.slide.settled(0.0, 1e-3)
    }

    /// Draws the gear centred at `center` with cap height `cap`.
    pub fn draw(&self, p: &mut Painter<'_>, center: fh6_kit::Vec2, font: FontId, cap: f32, color: Color) {
        let s = self.slide.value();
        let scale = 1.0 + 0.12 * s.abs();
        let pos = vec2(center.x, center.y + 24.0 * s);
        p.scoped(|p| {
            p.translate(pos);
            p.scale(scale);
            let style = TextStyle::new(font, cap)
                .paint(color)
                .align(Align::Center, VAlign::CapMiddle);
            p.text(vec2(0.0, 0.0), self.gear.label(), &style);
        });
    }
}

/// A tabular speed readout: fixed cells, ghost 8s in empty cells (tokens `type.tabular`), and
/// the 0.6 hysteresis so the last digit never flickers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeedReadout {
    shown: fh6_kit::Hysteresis,
    value: i64,
}

impl Default for SpeedReadout {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeedReadout {
    /// Starts at 0.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shown: fh6_kit::Hysteresis::new(0.0, 0.6),
            value: 0,
        }
    }

    /// Feeds the speed in display units.
    pub fn update(&mut self, speed: f32) {
        self.value = self.shown.update(speed).clamp(0, 999);
    }

    /// The integer shown.
    #[must_use]
    pub fn value(&self) -> i64 {
        self.value
    }

    /// Draws 3 cells centred at `center`.
    pub fn draw(&self, p: &mut Painter<'_>, center: fh6_kit::Vec2, font: FontId, cap: f32, ink: Color, ghost: Color) {
        let style = TextStyle::new(font, cap)
            .paint(ink)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        p.number(center, self.value, 3, &style, Some(ghost.into()));
    }
}

/// The virtual light (tokens `light`): azimuth `135° + 25° x clamp(gLat / 1.5)` on the `light`
/// spring. Glass rims and specular highlights read it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualLight {
    azimuth: Spring,
}

impl VirtualLight {
    /// At rest (lateral G = 0).
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            azimuth: Spring::new(SpringSpec { ..tokens.springs.light }, fh6_kit::deg(-135.0)),
        }
    }

    /// Advances with the smoothed lateral G.
    pub fn update(&mut self, g_lat: f32, dt: f32) {
        let target = fh6_kit::deg(-135.0 + 25.0 * (g_lat / 1.5).clamp(-1.0, 1.0));
        self.azimuth.update(target, dt);
    }

    /// The light direction angle (radians, screen convention), for `Glass::light_angle`.
    #[must_use]
    pub fn angle(&self) -> f32 {
        self.azimuth.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_kit::Tokens;

    fn signals(shift_fraction: f32) -> Signals {
        Signals {
            shift_fraction,
            ..Signals::default()
        }
    }

    #[test]
    fn shift_lights_attack_instantly_and_release_fast() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "raceDash");
        let mut s = ShiftLights::new(15);
        s.update(&signals(0.95), &pal, 0.0, 1.0 / 120.0, false);
        let lit = s.level;
        assert!(lit > 0.7, "same-frame attack: {lit}");
        // 50 ms later at idle, the strip is (almost) dark.
        for _ in 0..6 {
            s.update(&signals(0.2), &pal, 0.0, 1.0 / 120.0, false);
        }
        assert!(s.level < 0.15 * lit, "release within 50 ms: {}", s.level);
    }

    #[test]
    fn gear_glyph_slides_up_on_upshift() {
        let t = Tokens::default();
        let mut g = GearGlyph::new(&t);
        g.update(Gear::Forward(3), 0.0);
        assert!(g.slide.value() > 0.9);
        for _ in 0..60 {
            g.update(Gear::Forward(3), 1.0 / 60.0);
        }
        assert!(!g.animating());
        g.update(Gear::Forward(2), 0.0);
        assert!(g.slide.value() < -0.9, "downshift enters from below");
    }

    #[test]
    fn speed_hysteresis() {
        let mut s = SpeedReadout::new();
        s.update(117.6);
        assert_eq!(s.value(), 118);
        s.update(118.3);
        assert_eq!(s.value(), 118);
        s.update(1500.0);
        assert_eq!(s.value(), 999);
    }
}
