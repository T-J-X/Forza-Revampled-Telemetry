//! Colour: linear-light, premultiplied RGBA end to end, with sRGB and OKLab/OKLCH conversions.
//!
//! Hex values in tokens and `car_brands.json` are sRGB with straight alpha. They are converted
//! once, at the edge, into [`Color`] (linear, premultiplied). Everything the GPU blends is in
//! that space; the final resolve pass encodes to sRGB for the swapchain (ADR 0001).
use crate::math::{finite_or, lerp};
use std::fmt;
use std::str::FromStr;

/// sRGB transfer function, encoded to linear.
#[inline]
#[must_use]
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB transfer function, linear to encoded.
#[inline]
#[must_use]
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// A linear-light, premultiplied RGBA colour.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    /// Red, premultiplied, linear.
    pub r: f32,
    /// Green, premultiplied, linear.
    pub g: f32,
    /// Blue, premultiplied, linear.
    pub b: f32,
    /// Coverage / opacity.
    pub a: f32,
}

/// An error parsing a hex colour.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid hex colour {0:?} (expected #RRGGBB or #RRGGBBAA)")]
pub struct ParseColorError(pub String);

impl Color {
    /// Fully transparent.
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    /// Opaque OLED black (`#000000`).
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    /// Opaque white.
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// From linear straight-alpha components.
    #[inline]
    #[must_use]
    pub fn linear(r: f32, g: f32, b: f32, a: f32) -> Self {
        let a = finite_or(a, 0.0).clamp(0.0, 1.0);
        Self {
            r: finite_or(r, 0.0).max(0.0) * a,
            g: finite_or(g, 0.0).max(0.0) * a,
            b: finite_or(b, 0.0).max(0.0) * a,
            a,
        }
    }

    /// From sRGB-encoded straight-alpha components in `0..=1`.
    #[inline]
    #[must_use]
    pub fn srgb(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::linear(srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b), a)
    }

    /// From 8-bit sRGB straight-alpha components.
    #[inline]
    #[must_use]
    pub fn srgb8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::srgb(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            f32::from(a) / 255.0,
        )
    }

    /// From a `0xRRGGBB` sRGB value, opaque.
    #[inline]
    #[must_use]
    pub fn rgb_u32(rgb: u32) -> Self {
        let [_, r, g, b] = rgb.to_be_bytes();
        Self::srgb8(r, g, b, 255)
    }

    /// Parses `#RRGGBB`, `#RRGGBBAA`, `RRGGBB` or `#RGB`.
    ///
    /// # Errors
    /// [`ParseColorError`] when the text is not one of those forms.
    pub fn from_hex(hex: &str) -> Result<Self, ParseColorError> {
        let digits = hex.trim().trim_start_matches('#');
        let bad = || ParseColorError(hex.to_string());
        if !digits.is_ascii() {
            return Err(bad());
        }
        let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).map_err(|_| bad());
        let nib = |i: usize| {
            u8::from_str_radix(&digits[i..=i], 16)
                .map(|v| v * 17)
                .map_err(|_| bad())
        };
        match digits.len() {
            3 => Ok(Self::srgb8(nib(0)?, nib(1)?, nib(2)?, 255)),
            6 => Ok(Self::srgb8(byte(0)?, byte(2)?, byte(4)?, 255)),
            8 => Ok(Self::srgb8(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
            _ => Err(bad()),
        }
    }

    /// Parses `hex`, or returns `fallback` (for config values a user may have mistyped).
    #[must_use]
    pub fn from_hex_or(hex: &str, fallback: Color) -> Self {
        Self::from_hex(hex).unwrap_or(fallback)
    }

    /// The same colour at opacity `alpha` (0..=1, replacing the current opacity).
    #[inline]
    #[must_use]
    pub fn with_alpha(self, alpha: f32) -> Self {
        let (r, g, b) = self.straight_rgb();
        Self::linear(r, g, b, alpha)
    }

    /// The same colour at an 8-bit alpha (tokens use 0..=255).
    #[inline]
    #[must_use]
    pub fn with_alpha8(self, alpha: u8) -> Self {
        self.with_alpha(f32::from(alpha) / 255.0)
    }

    /// Opacity multiplied by `k` (premultiplied scale).
    #[inline]
    #[must_use]
    pub fn fade(self, k: f32) -> Self {
        let k = finite_or(k, 0.0).clamp(0.0, 1.0);
        Self {
            r: self.r * k,
            g: self.g * k,
            b: self.b * k,
            a: self.a * k,
        }
    }

    /// Light intensity multiplied by `k` without changing coverage (for emissive boosts).
    #[inline]
    #[must_use]
    pub fn intensity(self, k: f32) -> Self {
        let k = finite_or(k, 0.0).max(0.0);
        Self {
            r: self.r * k,
            g: self.g * k,
            b: self.b * k,
            a: self.a,
        }
    }

    /// Linear interpolation in premultiplied linear space.
    #[inline]
    #[must_use]
    pub fn lerp(self, o: Color, t: f32) -> Self {
        Self {
            r: lerp(self.r, o.r, t),
            g: lerp(self.g, o.g, t),
            b: lerp(self.b, o.b, t),
            a: lerp(self.a, o.a, t),
        }
    }

    /// Straight (un-premultiplied) linear RGB.
    #[inline]
    #[must_use]
    pub fn straight_rgb(self) -> (f32, f32, f32) {
        if self.a > 1e-6 {
            (self.r / self.a, self.g / self.a, self.b / self.a)
        } else {
            (0.0, 0.0, 0.0)
        }
    }

    /// Relative luminance of the straight colour (Rec. 709 weights, linear).
    #[inline]
    #[must_use]
    pub fn luminance(self) -> f32 {
        let (r, g, b) = self.straight_rgb();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// 8-bit sRGB straight RGBA.
    #[must_use]
    pub fn to_srgb8(self) -> [u8; 4] {
        let (r, g, b) = self.straight_rgb();
        let enc = |v: f32| (linear_to_srgb(v.clamp(0.0, 1.0)) * 255.0).round() as u8;
        [enc(r), enc(g), enc(b), (self.a.clamp(0.0, 1.0) * 255.0).round() as u8]
    }

    /// `[r, g, b, a]` as the GPU sees it.
    #[inline]
    #[must_use]
    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// The colour in OKLCH (straight; alpha dropped).
    #[must_use]
    pub fn to_oklch(self) -> Oklch {
        let (r, g, b) = self.straight_rgb();
        Oklab::from_linear_rgb(r, g, b).to_oklch()
    }

    /// An opaque colour from OKLCH, gamut-clipped by reducing chroma.
    #[must_use]
    pub fn from_oklch(c: Oklch) -> Self {
        let (r, g, b) = c.to_linear_rgb_clipped();
        Self::linear(r, g, b, 1.0)
    }

    /// Contrast ratio (WCAG) of this colour drawn opaque on black.
    #[must_use]
    pub fn contrast_on_black(self) -> f32 {
        (self.luminance() + 0.05) / 0.05
    }

    /// The text-safe lift (tokens `derivedTextSafe`): raises OKLCH lightness to at least 0.70
    /// and caps chroma at 0.26, so thin lines and digits keep >= 7:1 on OLED black.
    #[must_use]
    pub fn text_safe(self) -> Self {
        let mut c = self.to_oklch();
        c.l = c.l.max(0.70);
        c.c = c.c.min(0.26);
        Self::from_oklch(c).with_alpha(self.a)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b, a] = self.to_srgb8();
        if a == 255 {
            write!(f, "#{r:02X}{g:02X}{b:02X}")
        } else {
            write!(f, "#{r:02X}{g:02X}{b:02X}{a:02X}")
        }
    }
}

impl FromStr for Color {
    type Err = ParseColorError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

/// A colour in OKLab (Björn Ottosson, 2020).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Oklab {
    /// Lightness 0..1.
    pub l: f32,
    /// Green-red axis.
    pub a: f32,
    /// Blue-yellow axis.
    pub b: f32,
}

/// A colour in OKLCH: lightness, chroma, hue in degrees.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Oklch {
    /// Lightness 0..1.
    pub l: f32,
    /// Chroma (0 = grey; sRGB tops out near 0.32).
    pub c: f32,
    /// Hue in degrees, 0..360.
    pub h: f32,
}

impl Oklab {
    /// From straight linear sRGB.
    #[must_use]
    pub fn from_linear_rgb(r: f32, g: f32, b: f32) -> Self {
        let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
        let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
        let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;
        let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());
        Self {
            l: 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
            a: 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
            b: 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
        }
    }

    /// To straight linear sRGB (may be out of gamut).
    #[must_use]
    pub fn to_linear_rgb(self) -> (f32, f32, f32) {
        let l = self.l + 0.396_337_78 * self.a + 0.215_803_76 * self.b;
        let m = self.l - 0.105_561_346 * self.a - 0.063_854_17 * self.b;
        let s = self.l - 0.089_484_18 * self.a - 1.291_485_5 * self.b;
        let (l, m, s) = (l * l * l, m * m * m, s * s * s);
        (
            4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
            -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
            -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
        )
    }

    /// To polar form.
    #[must_use]
    pub fn to_oklch(self) -> Oklch {
        let c = (self.a * self.a + self.b * self.b).sqrt();
        let h = self.b.atan2(self.a).to_degrees().rem_euclid(360.0);
        Oklch { l: self.l, c, h }
    }
}

impl Oklch {
    /// A colour from lightness, chroma and hue (degrees).
    #[must_use]
    pub fn new(l: f32, c: f32, h: f32) -> Self {
        Self {
            l: finite_or(l, 0.0).clamp(0.0, 1.0),
            c: finite_or(c, 0.0).max(0.0),
            h: finite_or(h, 0.0).rem_euclid(360.0),
        }
    }

    /// To OKLab.
    #[must_use]
    pub fn to_oklab(self) -> Oklab {
        let (s, c) = self.h.to_radians().sin_cos();
        Oklab {
            l: self.l,
            a: self.c * c,
            b: self.c * s,
        }
    }

    /// Hue rotated by `degrees`.
    #[must_use]
    pub fn rotate_hue(self, degrees: f32) -> Self {
        Self::new(self.l, self.c, self.h + degrees)
    }

    /// Straight linear sRGB, reducing chroma (binary search) until it fits the gamut.
    #[must_use]
    pub fn to_linear_rgb_clipped(self) -> (f32, f32, f32) {
        let in_gamut = |(r, g, b): (f32, f32, f32)| {
            let ok = |v: f32| (-1e-4..=1.0 + 1e-4).contains(&v);
            ok(r) && ok(g) && ok(b)
        };
        let direct = self.to_oklab().to_linear_rgb();
        if in_gamut(direct) {
            return clamp3(direct);
        }
        let (mut lo, mut hi) = (0.0f32, self.c);
        for _ in 0..24 {
            let mid = (lo + hi) * 0.5;
            if in_gamut(Oklch { c: mid, ..self }.to_oklab().to_linear_rgb()) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        clamp3(Oklch { c: lo, ..self }.to_oklab().to_linear_rgb())
    }

    /// Shortest angular distance between two hues, in degrees (0..=180).
    #[must_use]
    pub fn hue_distance(a: f32, b: f32) -> f32 {
        let d = (a - b).rem_euclid(360.0);
        d.min(360.0 - d)
    }
}

fn clamp3((r, g, b): (f32, f32, f32)) -> (f32, f32, f32) {
    (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip() {
        for hex in ["#FF2D3A", "#000000", "#FFFFFF", "#38BDF8", "#7C5CFF80"] {
            let c = Color::from_hex(hex).expect("valid");
            assert_eq!(c.to_string(), hex, "{hex}");
        }
        assert!(Color::from_hex("#12345").is_err());
        assert!(Color::from_hex("zzzzzz").is_err());
        assert_eq!(Color::from_hex("#fff"), Ok(Color::WHITE));
    }

    #[test]
    fn premultiplied_alpha() {
        let c = Color::srgb8(255, 255, 255, 128);
        assert!((c.r - c.a).abs() < 1e-6, "white premultiplied: rgb == a");
        let faded = Color::WHITE.fade(0.5);
        assert!((faded.r - 0.5).abs() < 1e-6 && (faded.a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn oklab_round_trip_and_known_value() {
        let c = Color::from_hex("#38BDF8").expect("valid");
        let back = Color::from_oklch(c.to_oklch());
        assert_eq!(back.to_srgb8(), c.to_srgb8());
        let white = Color::WHITE.to_oklch();
        assert!((white.l - 1.0).abs() < 1e-3 && white.c < 1e-3);
    }

    #[test]
    fn text_safe_lifts_dark_brand_blue_to_7_to_1() {
        let bmw = Color::from_hex("#0066B1").expect("valid");
        assert!(bmw.contrast_on_black() < 7.0);
        let lifted = bmw.text_safe();
        assert!(lifted.contrast_on_black() >= 7.0, "got {}", lifted.contrast_on_black());
        assert!(
            Oklch::hue_distance(lifted.to_oklch().h, bmw.to_oklch().h) < 6.0,
            "hue kept"
        );
    }
}
