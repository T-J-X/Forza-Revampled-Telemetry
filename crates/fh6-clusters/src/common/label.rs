//! MicroLabel (COMPONENTS.md §7, rank 5) and LogoZone (§6, rank 5).
//!
//! MicroLabel: the `text` face (Titillium Web 600/700) in caps at `type.scale.microLabel` (cap 16)
//! or `nano` (12, the floor), +16–30% tracking, in `ink_label`; never glows. "Shy" state labels
//! (TC, ABS, LIM) show their colour only while active.
//!
//! LogoZone: crest ≤ 96 px or wordmark ≤ 220 × 44 px; the dial variant is mono white at alpha 215
//! with a 1 px engraved shadow; the fallback is the brand, then the custom name in the `wide`
//! face (Michroma) caps, ≤ 14 characters (the kit's `logo::draw`).
use fh6_kit::logo::{self, LogoStyle};
use fh6_kit::{Align, Color, FontId, Logo, Painter, Palette, Rect, TextStyle, Tokens, VAlign, Vec2};

/// The micro-label size (tokens `type.scale`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelSize {
    /// Cap 16.
    #[default]
    Micro,
    /// Cap 12 (the floor).
    Nano,
}

/// A micro-label style: `font` (the `text` role), size, tracking from the tokens, label ink,
/// aligned as asked.
#[must_use]
pub fn micro_label(
    tokens: &Tokens,
    font: FontId,
    size: LabelSize,
    palette: &Palette,
    align: Align,
    valign: VAlign,
) -> TextStyle {
    let cap = match size {
        LabelSize::Micro => tokens.type_scale.micro_label,
        LabelSize::Nano => tokens.type_scale.micro_label * 0.75,
    };
    TextStyle::new(font, cap)
        .paint(palette.roles.ink_label)
        .tracking(tokens.type_scale.label_tracking)
        .align(align, valign)
}

/// The colour of a shy state label: `active_color` while `active`, else the quiet label ink.
#[must_use]
pub fn shy(active: bool, active_color: Color, palette: &Palette) -> Color {
    if active {
        active_color
    } else {
        palette.roles.ink_label.fade(0.55)
    }
}

/// Where a logo sits and how it looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogoVariant {
    /// Full colour (or the brand name) on glass or black.
    #[default]
    Full,
    /// The dial variant: mono white at alpha 215 with a 1 px engraved shadow.
    Dial,
}

/// Draws the logo zone in `zone` (≤ 220 × 44 for a wordmark, ≤ 96 px for a crest): the car's
/// logo, else the brand name, else the custom name in the `wide` face.
pub fn logo_zone(p: &mut Painter<'_>, zone: Rect, logo: &Logo, wide: FontId, variant: LogoVariant, palette: &Palette) {
    let ink = match variant {
        LogoVariant::Full => palette.roles.ink_label,
        LogoVariant::Dial => palette.roles.ink.with_alpha(215.0 / 255.0),
    };
    if variant == LogoVariant::Dial {
        // Engraved: a 1 px dark copy below-right first.
        let shadow = LogoStyle {
            text: palette.roles.oled_black.with_alpha(0.8).into(),
            ..LogoStyle::new(wide)
        };
        let off = Rect::from_points(zone.min + Vec2::new(1.0, 1.0), zone.max + Vec2::new(1.0, 1.0));
        logo::draw(p, off, logo, &shadow);
    }
    let style = LogoStyle {
        text: ink.into(),
        ..LogoStyle::new(wide)
    };
    logo::draw(p, zone, logo, &style);
}

/// Draws a label/value pair right-aligned at `right` (value) with the label to its left, e.g.
/// `BOOST  21.7`. The value is an integer of `cells` tabular cells; no allocation.
#[allow(clippy::too_many_arguments)]
pub fn metric(
    p: &mut Painter<'_>,
    right: Vec2,
    label: &str,
    value: i64,
    cells: u8,
    value_style: &TextStyle,
    label_style: &TextStyle,
    gap: f32,
) {
    let style = TextStyle {
        align: Align::Right,
        valign: VAlign::CapMiddle,
        ..*value_style
    };
    let r = p.number(right, value, cells, &style, None);
    let ls = TextStyle {
        align: Align::Right,
        valign: VAlign::CapMiddle,
        ..*label_style
    };
    p.text(Vec2::new(r.min.x - gap, right.y), label, &ls);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_kit::{Palette, Tokens};

    #[test]
    fn shy_labels_are_quiet_until_active() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "raceDash");
        assert_eq!(shy(true, pal.roles.caution, &pal), pal.roles.caution);
        assert!(shy(false, pal.roles.caution, &pal).a < pal.roles.ink_label.a);
    }

    #[test]
    fn nano_is_the_floor() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "raceDash");
        let s = micro_label(&t, FontId::FALLBACK, LabelSize::Nano, &pal, Align::Left, VAlign::CapTop);
        assert!(s.cap >= 12.0 - 1e-3, "{}", s.cap);
    }
}
