//! The Phase 7 scaffold every catalog cluster starts from: a quiet "under construction" frame
//! drawn with the kit, so all 73 dashes are selectable, render in `preview-export` and run live
//! before their cluster chat builds them.
//!
//! It is still a working dash (live gear, tabular speed, rev bar and outside-in shift lights from
//! the telemetry), never an opaque fill: corner brackets mark the view envelope, a hairline rim
//! traces it, and the catalog name, family, catalog row and category are set in the label inks.
//! It names no build number: the build order lives in CATALOG.md and changes there.
use crate::catalog::{self, CatalogEntry, Kind};
use crate::common::{GearGlyph, ShiftLights, SpeedReadout};
use fh6_core::ClusterTheme;
use fh6_kit::cluster::Setup;
use fh6_kit::{
    vec2, Align, FontChoice, FontRole, FrameInput, Gradient, LayerOptions, Layout, Painter, Palette, Rect, Shape,
    Signals, Spring, Stroke, TextStyle, VAlign, Vec2,
};

/// Length of the corner-bracket arms (local px).
const BRACKET: f32 = 56.0;
/// Bracket stroke width (local px).
const BRACKET_W: f32 = 2.0;

/// The scaffold cluster state.
#[derive(Debug, Clone)]
pub struct Scaffold {
    theme: ClusterTheme,
    title: String,
    family_label: String,
    slot_label: String,
    sig: Signals,
    palette: Palette,
    fonts: FontChoice,
    rpm: Spring,
    gear: GearGlyph,
    speed: SpeedReadout,
    shift: ShiftLights,
    micro_cap: f32,
    tracking: f32,
}

impl Scaffold {
    /// The scaffold for `theme`, labelled from its catalog row.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>, theme: ClusterTheme) -> Self {
        let t = setup.tokens;
        let entry: Option<&CatalogEntry> = catalog::entry(theme);
        let (title, family_label, slot_label, family) = match entry {
            Some(e) => (
                e.display.to_uppercase(),
                format!(
                    "{}  ·  {}",
                    e.family.label().to_uppercase(),
                    if e.pattern_setter {
                        "PATTERN-SETTER"
                    } else if e.kind == Kind::New {
                        "NEW DASH"
                    } else {
                        "REBUILD"
                    }
                ),
                format!(
                    "ROW {}  ·  {}  ·  {}",
                    e.row,
                    e.category.label().to_uppercase(),
                    e.views.label().to_uppercase()
                ),
                e.family.key(),
            ),
            None => (theme.name().to_uppercase(), String::new(), String::new(), "liquidGlass"),
        };
        Self {
            theme,
            title,
            family_label,
            slot_label,
            sig: Signals::default(),
            palette: Palette::family(t, family),
            fonts: FontChoice {
                display: setup.fonts.role(FontRole::Display),
                text: setup.fonts.role(FontRole::Text),
                wide: setup.fonts.role(FontRole::Wide),
            },
            rpm: Spring::new(t.springs.ribbon, 0.0),
            gear: GearGlyph::new(t),
            speed: SpeedReadout::new(),
            shift: ShiftLights::new(15),
            micro_cap: t.type_scale.micro_label,
            tracking: t.type_scale.label_tracking,
        }
    }

    /// The theme this scaffold stands in for.
    #[must_use]
    pub fn theme(&self) -> ClusterTheme {
        self.theme
    }

    /// Advances the live parts.
    pub fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt;
        self.sig = *input.signals;
        self.palette = *input.palette;
        self.fonts = input.fonts;
        let rate = self.sig.rpm_rate / self.sig.max_rpm.max(1.0);
        self.rpm.update_tracking(self.sig.rpm_fraction, rate, dt);
        self.gear.update(self.sig.gear, dt);
        self.speed.update(self.sig.speed);
        self.shift
            .update(&self.sig, &self.palette, input.t, dt, input.reduced_flash);
    }

    /// True while a spring or the limiter strobe is still moving.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.shift.animating() || self.gear.animating() || !self.rpm.settled(self.sig.rpm_fraction, 1e-4)
    }

    /// Draws the scaffold inside the view envelope.
    pub fn draw(&self, p: &mut Painter<'_>, layout: &Layout) {
        let pal = self.palette;
        let roles = pal.roles;
        let env = layout.envelope;
        let (w, h) = (env.width(), env.height());
        let mid = env.center();

        // 1. The envelope: four corner brackets and a hairline rim (no fill).
        p.stroke(
            &Shape::round_rect(env, 28.0),
            roles.hairline.fade(0.45),
            Stroke::rim_px(1.0),
        );
        for (corner, dir) in [
            (env.min, vec2(1.0, 1.0)),
            (vec2(env.max.x, env.min.y), vec2(-1.0, 1.0)),
            (vec2(env.min.x, env.max.y), vec2(1.0, -1.0)),
            (env.max, vec2(-1.0, -1.0)),
        ] {
            let c = corner + dir * 6.0;
            p.fill(
                &Shape::line(c, c + vec2(dir.x * BRACKET, 0.0), BRACKET_W),
                roles.ink_label,
            );
            p.fill(
                &Shape::line(c, c + vec2(0.0, dir.y * BRACKET), BRACKET_W),
                roles.ink_label,
            );
        }

        // 2. Shift lights across the top (<= 42 px, the only strobe).
        let strip_w = (w * 0.5).min(720.0);
        self.shift.draw(
            p,
            Rect::from_xywh(mid.x - strip_w * 0.5, env.min.y + 34.0, strip_w, 18.0),
            &pal,
        );

        // 3. Name, family, catalog row and category.
        let title = TextStyle::new(self.fonts.wide, 26.0)
            .paint(roles.ink_muted)
            .tracking(0.28)
            .align(Align::Center, VAlign::CapMiddle);
        p.text(vec2(mid.x, env.min.y + 104.0), &self.title, &title);
        let label = TextStyle::new(self.fonts.text, self.micro_cap)
            .paint(roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapMiddle);
        p.text(
            vec2(mid.x, env.min.y + 142.0),
            "UNDER CONSTRUCTION  ·  PHASE 7 CLUSTER FACTORY",
            &label,
        );

        // 4. Live core: gear (rank 1), speed (rank 2), rev bar (rank 3).
        let core_y = env.min.y + h * 0.60;
        let gear_cap = (h * 0.27).clamp(90.0, 150.0);
        p.layer(LayerOptions::bloom(), |p| {
            let gear_color = match self.sig.gear {
                fh6_kit::Gear::Reverse => roles.warning,
                fh6_kit::Gear::Neutral => roles.ink.fade(0.72),
                fh6_kit::Gear::Forward(_) => roles.ink,
            };
            self.gear
                .draw(p, vec2(mid.x, core_y), self.fonts.display, gear_cap, gear_color);
        });
        let speed_cap = gear_cap * 0.56;
        let speed_x = mid.x - (w * 0.22).min(300.0);
        self.speed.draw(
            p,
            vec2(speed_x, core_y),
            self.fonts.display,
            speed_cap,
            roles.ink,
            roles.ink_ghost,
        );
        p.text(
            vec2(speed_x, core_y + speed_cap * 0.5 + 28.0),
            self.sig.unit.label(),
            &label,
        );
        let rpm_x = mid.x + (w * 0.22).min(300.0);
        let value = TextStyle::new(self.fonts.display, speed_cap * 0.5)
            .paint(roles.ink_muted)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        p.number(vec2(rpm_x, core_y), self.sig.rpm.round() as i64, 5, &value, None);
        p.text(vec2(rpm_x, core_y + speed_cap * 0.5 + 28.0), "RPM", &label);

        let bar = Rect::from_xywh(mid.x - strip_w * 0.5, env.max.y - 64.0, strip_w, 8.0);
        p.fill(&Shape::round_rect(bar, 4.0), roles.ink_ghost);
        let frac = self.rpm.value().clamp(0.0, 1.0);
        if frac > 0.002 {
            let lit = Rect::from_xywh(bar.min.x, bar.min.y, bar.width() * frac, bar.height());
            let fill = Gradient::linear(bar.at(0.0, 0.5), bar.at(1.0, 0.5))
                .stop(0.0, pal.primary.with_alpha(0.7))
                .stop(pal.stages[0], pal.primary)
                .stop(pal.stages[1], pal.rev_stage(pal.stages[1]))
                .stop(1.0, roles.warning);
            p.layer(LayerOptions::bloom(), |p| p.fill(&Shape::round_rect(lit, 4.0), fill));
        }

        // 5. Footer labels.
        let left = TextStyle::new(self.fonts.text, self.micro_cap)
            .paint(roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Left, VAlign::CapMiddle);
        let right = TextStyle {
            align: Align::Right,
            ..left
        };
        let foot = env.max.y - 30.0;
        p.text(vec2(env.min.x + 30.0, foot), &self.family_label, &left);
        p.text(vec2(env.max.x - 30.0, foot), &self.slot_label, &right);
        // Centre mark: the anchor, so fit reviews can see where (0, 0) lands.
        let c = Vec2::ZERO;
        p.fill(
            &Shape::line(c - vec2(10.0, 0.0), c + vec2(10.0, 0.0), 1.0),
            roles.hairline,
        );
        p.fill(
            &Shape::line(c - vec2(0.0, 10.0), c + vec2(0.0, 10.0), 1.0),
            roles.hairline,
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::CATALOG;

    #[test]
    fn every_catalog_row_has_a_family_palette_key() {
        let tokens = fh6_kit::Tokens::default();
        for e in &CATALOG {
            assert!(tokens.family(e.family.key()).is_some(), "{}", e.family.key());
        }
    }
}
