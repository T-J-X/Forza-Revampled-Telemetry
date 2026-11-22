//! The logo zone: a brand logo fitted with brand-safe padding and optical centring, falling back
//! to the brand name, then the custom name, set in the wide (wordmark) face.
use crate::cluster::Logo;
use crate::color::Color;
use crate::math::{vec2, Rect, Vec2};
use crate::paint::{ImagePaint, Paint};
use crate::painter::Painter;
use crate::shape::Shape;
use crate::text::{Align, FontId, TextStyle, VAlign};

/// Brand-safe padding as a fraction of the zone's shorter side.
pub const SAFE_PADDING: f32 = 0.12;
/// Optical centring: marks sit this fraction of the zone height above the geometric centre.
pub const OPTICAL_LIFT: f32 = 0.02;
/// Longest wordmark fallback, in characters (tokens `layout.logoZone`).
pub const MAX_NAME_CHARS: usize = 14;

/// How to draw the logo zone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogoStyle {
    /// Image tint (tokens: dial variant is mono white at alpha 215).
    pub tint: Color,
    /// Wordmark paint.
    pub text: Paint,
    /// Wordmark face (the `wide` role).
    pub font: FontId,
    /// Wordmark cap height in local units (tokens: 18).
    pub cap: f32,
    /// Wordmark tracking in em (tokens: +30%).
    pub tracking: f32,
}

impl LogoStyle {
    /// The tokens' wordmark style in `font`, white.
    #[must_use]
    pub fn new(font: FontId) -> Self {
        Self {
            tint: Color::WHITE,
            text: Paint::Solid(Color::WHITE),
            font,
            cap: 18.0,
            tracking: 0.30,
        }
    }
}

/// The rectangle an image of `image_size` occupies inside `zone`: aspect kept, inside the
/// brand-safe padding, lifted for optical centring.
#[must_use]
pub fn fit(zone: Rect, image_size: (u32, u32)) -> Rect {
    let pad = zone.width().min(zone.height()) * SAFE_PADDING;
    let inner = zone.inset(pad);
    let (iw, ih) = (image_size.0.max(1) as f32, image_size.1.max(1) as f32);
    if inner.is_empty() {
        return Rect::from_center_size(zone.center(), Vec2::ZERO);
    }
    let k = (inner.width() / iw).min(inner.height() / ih);
    let size = vec2(iw * k, ih * k);
    let center = inner.center() - vec2(0.0, zone.height() * OPTICAL_LIFT);
    Rect::from_center_size(center, size)
}

/// The wordmark text for a logo without an image: the brand name, else the custom name, upper
/// case, at most [`MAX_NAME_CHARS`] characters. Writes into `buf` (no allocation).
#[must_use]
pub fn wordmark<'b>(logo: &Logo, buf: &'b mut [u8; 64]) -> &'b str {
    let name = if logo.brand_name.trim().is_empty() {
        logo.custom_name.trim()
    } else {
        logo.brand_name.trim()
    };
    let mut len = 0;
    for ch in name.chars().take(MAX_NAME_CHARS) {
        for up in ch.to_uppercase() {
            let mut tmp = [0u8; 4];
            let s = up.encode_utf8(&mut tmp);
            if len + s.len() > buf.len() {
                break;
            }
            buf[len..len + s.len()].copy_from_slice(s.as_bytes());
            len += s.len();
        }
    }
    std::str::from_utf8(&buf[..len]).unwrap_or("")
}

/// Draws the logo zone. Returns the rectangle actually used.
pub fn draw(p: &mut Painter<'_>, zone: Rect, logo: &Logo, style: &LogoStyle) -> Rect {
    if let Some(image) = logo.image {
        if let Some(size) = p.images().size(image) {
            let r = fit(zone, size);
            p.fill(&Shape::rect(r), ImagePaint::new(image, r).tint(style.tint));
            return r;
        }
    }
    let mut buf = [0u8; 64];
    let text = wordmark(logo, &mut buf);
    if text.is_empty() {
        return Rect::from_center_size(zone.center(), Vec2::ZERO);
    }
    let mut ts = TextStyle::new(style.font, style.cap)
        .paint(style.text)
        .align(Align::Center, VAlign::CapMiddle)
        .tracking(style.tracking);
    // Shrink to fit the zone's inner width (labels shrink to 85% then elide; wordmarks just fit).
    let inner = zone.inset(zone.width().min(zone.height()) * SAFE_PADDING);
    let w = p.measure(text, &ts).width;
    if w > inner.width() && w > 0.0 {
        ts.cap *= inner.width() / w;
    }
    p.text(zone.center(), text, &ts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_keeps_aspect_and_padding() {
        let zone = Rect::from_xywh(0.0, 0.0, 200.0, 100.0);
        let r = fit(zone, (400, 100));
        assert!((r.width() / r.height() - 4.0).abs() < 1e-4);
        let pad = 100.0 * SAFE_PADDING;
        assert!(r.min.x >= pad - 1e-3 && r.max.x <= 200.0 - pad + 1e-3);
        assert!(r.center().y < zone.center().y, "optically lifted");
    }

    #[test]
    fn wordmark_fallbacks() {
        let mut buf = [0u8; 64];
        let logo = Logo {
            image: None,
            brand_name: "Porsche".into(),
            custom_name: "TJX".into(),
        };
        assert_eq!(wordmark(&logo, &mut buf), "PORSCHE");
        let logo = Logo {
            brand_name: String::new(),
            custom_name: "a very long custom name here".into(),
            ..Logo::default()
        };
        assert_eq!(wordmark(&logo, &mut buf).chars().count(), MAX_NAME_CHARS);
    }
}
