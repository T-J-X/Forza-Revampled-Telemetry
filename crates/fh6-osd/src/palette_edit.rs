//! The custom palette editor's colour maths (HSV over sRGB hex) and its presets.
use std::fmt::Write as _;

/// Which of the four palette roles the custom editor changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColourTarget {
    /// Hero accents, tach fill, needle.
    #[default]
    Primary,
    /// Trim and secondary arcs.
    Secondary,
    /// Leading edges and the active segment.
    Accent,
    /// Limiter strobe and the last shift stage.
    Shift,
}

impl ColourTarget {
    /// All roles in editor order.
    pub const ALL: [ColourTarget; 4] = [Self::Primary, Self::Secondary, Self::Accent, Self::Shift];

    /// The label shown in the OSD.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Accent => "Accent",
            Self::Shift => "Shift light",
        }
    }

    /// The role `steps` places further on, wrapping.
    #[must_use]
    pub fn step(self, steps: i32) -> Self {
        let i = Self::ALL.iter().position(|t| *t == self).unwrap_or(0) as i32;
        Self::ALL[(i + steps).rem_euclid(4) as usize]
    }
}

/// A colour as hue (degrees, 0..360), saturation and value (0..1), in sRGB.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Hsv {
    /// Hue in degrees.
    pub h: f32,
    /// Saturation 0..1.
    pub s: f32,
    /// Value 0..1.
    pub v: f32,
}

impl Hsv {
    /// Parses `#RRGGBB` (or `RRGGBB`); `None` when it is not a 6-digit hex colour.
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<Self> {
        let [r, g, b] = parse_hex(hex)?;
        Some(Self::from_rgb(r, g, b))
    }

    /// HSV from 8-bit sRGB.
    #[must_use]
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let (r, g, b) = (f32::from(r) / 255.0, f32::from(g) / 255.0, f32::from(b) / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let h = if d <= f32::EPSILON {
            0.0
        } else if (max - r).abs() <= f32::EPSILON {
            60.0 * ((g - b) / d).rem_euclid(6.0)
        } else if (max - g).abs() <= f32::EPSILON {
            60.0 * ((b - r) / d + 2.0)
        } else {
            60.0 * ((r - g) / d + 4.0)
        };
        let s = if max <= f32::EPSILON { 0.0 } else { d / max };
        Self { h, s, v: max }
    }

    /// 8-bit sRGB from HSV (inputs are clamped and wrapped).
    #[must_use]
    pub fn to_rgb(self) -> [u8; 3] {
        let h = self.h.rem_euclid(360.0);
        let s = self.s.clamp(0.0, 1.0);
        let v = self.v.clamp(0.0, 1.0);
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
        let m = v - c;
        let (r, g, b) = match (h / 60.0) as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let q = |f: f32| ((f + m) * 255.0).round().clamp(0.0, 255.0) as u8;
        [q(r), q(g), q(b)]
    }

    /// `#RRGGBB`.
    #[must_use]
    pub fn to_hex(self) -> String {
        let [r, g, b] = self.to_rgb();
        let mut s = String::with_capacity(7);
        let _ = write!(s, "#{r:02X}{g:02X}{b:02X}");
        s
    }
}

/// Parses `#RRGGBB` / `RRGGBB` into bytes.
#[must_use]
pub fn parse_hex(hex: &str) -> Option<[u8; 3]> {
    let h = hex.trim().trim_start_matches('#');
    if h.len() != 6 || !h.is_ascii() {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

/// A named custom palette: primary, secondary, accent, shift light.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preset {
    /// Name shown in the OSD.
    pub name: &'static str,
    /// `#RRGGBB` for primary, secondary, accent and shift light.
    pub hex: [&'static str; 4],
}

/// Curated presets. Every primary sits at least 25 degrees from the warning red, and every
/// shift light reads against black (OLED first).
pub const PRESETS: &[Preset] = &[
    Preset {
        name: "Sapphire",
        hex: ["#38BDF8", "#1E3A8A", "#7DD3FC", "#FFFFFF"],
    },
    Preset {
        name: "Papaya",
        hex: ["#FF8000", "#47C7FC", "#FFB366", "#7C5CFF"],
    },
    Preset {
        name: "Verde",
        hex: ["#22C55E", "#14532D", "#86EFAC", "#FFFFFF"],
    },
    Preset {
        name: "Ultraviolet",
        hex: ["#8B5CF6", "#312E81", "#C4B5FD", "#00E5FF"],
    },
    Preset {
        name: "Ice",
        hex: ["#E2E8F0", "#64748B", "#38BDF8", "#00E5FF"],
    },
    Preset {
        name: "Gold leaf",
        hex: ["#F5C542", "#7C5A10", "#FDE68A", "#FFFFFF"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_through_hsv() {
        for hex in ["#38BDF8", "#FF8000", "#22C55E", "#000000", "#FFFFFF", "#7C5CFF"] {
            let hsv = Hsv::from_hex(hex).expect("valid hex");
            assert_eq!(hsv.to_hex(), hex);
        }
    }

    #[test]
    fn bad_hex_is_rejected() {
        assert_eq!(parse_hex("#12345"), None);
        assert_eq!(parse_hex("#GG0000"), None);
        assert_eq!(parse_hex("38BDF8"), Some([0x38, 0xBD, 0xF8]));
    }

    #[test]
    fn presets_are_valid_hex() {
        for p in PRESETS {
            for h in p.hex {
                assert!(parse_hex(h).is_some(), "{} {h}", p.name);
            }
        }
    }

    #[test]
    fn targets_wrap() {
        assert_eq!(ColourTarget::Primary.step(-1), ColourTarget::Shift);
        assert_eq!(ColourTarget::Shift.step(1), ColourTarget::Primary);
    }
}
