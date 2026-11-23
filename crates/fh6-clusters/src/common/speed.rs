//! TabularSpeed (COMPONENTS.md §2, rank 2): 3 fixed digit cells (0.60 em), ghost `8` only in
//! the unused leading cells, a unit label, and the 0.6-unit hysteresis so the last digit never
//! flickers. The width never changes from 0 to 999.
//!
//! The face and weight are the caller's (variants `speedHero` display 600 cap 112,
//! `speedCompact` cap 84, `light` Saira 200/300 for glass and ADAS, `race` 700); the segment
//! variant lives in `common::seg7`.
use super::SpeedReadout;
use fh6_kit::{Align, Color, FontId, Painter, Palette, TextStyle, VAlign, Vec2};

/// A tabular speed field.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TabularSpeed {
    readout: SpeedReadout,
}

impl TabularSpeed {
    /// At 0.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds the speed in display units.
    pub fn update(&mut self, speed: f32) {
        self.readout.update(speed);
    }

    /// The integer shown.
    #[must_use]
    pub fn value(&self) -> i64 {
        self.readout.value()
    }

    /// Draws the 3 cells centred at `center` (cap middle), cap height `cap`, in `ink` with ghost
    /// 8s in `ghost` (`None` for no ghosts, e.g. a floating glass readout).
    pub fn draw(&self, p: &mut Painter<'_>, center: Vec2, font: FontId, cap: f32, ink: Color, ghost: Option<Color>) {
        let style = TextStyle::new(font, cap)
            .paint(ink)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        p.number(center, self.value(), 3, &style, ghost.map(Into::into));
    }

    /// Draws the unit label (`MPH` / `KM/H`) centred at `pos` (cap top) in the label ink.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_unit(
        &self,
        p: &mut Painter<'_>,
        pos: Vec2,
        label: &str,
        font: FontId,
        cap: f32,
        tracking: f32,
        palette: &Palette,
    ) {
        let style = TextStyle::new(font, cap)
            .paint(palette.roles.ink_label)
            .tracking(tracking)
            .align(Align::Center, VAlign::CapTop);
        p.text(pos, label, &style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hysteresis_and_clamp() {
        let mut s = TabularSpeed::new();
        s.update(64.2);
        assert_eq!(s.value(), 64);
        s.update(64.5);
        assert_eq!(s.value(), 64, "within 0.6 of the shown value");
        s.update(64.7);
        assert_eq!(s.value(), 65, "moved more than 0.6");
        s.update(1200.0);
        assert_eq!(s.value(), 999);
    }
}
