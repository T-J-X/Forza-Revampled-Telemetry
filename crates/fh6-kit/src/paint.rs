//! Paints: what fills a shape. Solid, linear/radial/conic gradients (up to 8 stops), images and
//! the Liquid Glass material. Paint coordinates are in the shape's local space, so a gradient
//! moves, scales and rotates with the shape that uses it.
use crate::color::Color;
use crate::math::{finite_or, Rect, Vec2};

/// Most stops a gradient carries. Extra stops are ignored.
pub const MAX_STOPS: usize = 8;

/// How a draw combines with what is already in its layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BlendMode {
    /// Premultiplied source-over.
    #[default]
    Normal,
    /// Additive light: `dst + src`.
    Add,
    /// Screen: `src + dst * (1 - src)`.
    Screen,
    /// Multiply: darkens what is underneath by the source colour.
    Multiply,
}

impl BlendMode {
    /// All modes, in pipeline order.
    pub const ALL: [BlendMode; 4] = [
        BlendMode::Normal,
        BlendMode::Add,
        BlendMode::Screen,
        BlendMode::Multiply,
    ];

    /// Index into [`BlendMode::ALL`].
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            BlendMode::Normal => 0,
            BlendMode::Add => 1,
            BlendMode::Screen => 2,
            BlendMode::Multiply => 3,
        }
    }
}

/// What happens outside a gradient's 0..1 range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Spread {
    /// Clamp to the end stops.
    #[default]
    Pad,
    /// Repeat the ramp.
    Repeat,
    /// Mirror the ramp back and forth.
    Reflect,
}

/// One gradient stop.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Stop {
    /// Position along the ramp, 0..=1.
    pub offset: f32,
    /// Colour at that position.
    pub color: Color,
}

/// The geometry of a gradient, in the shape's local coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientKind {
    /// Along the line from `start` (t = 0) to `end` (t = 1).
    Linear {
        /// Where t = 0.
        start: Vec2,
        /// Where t = 1.
        end: Vec2,
    },
    /// Outwards from `center` (t = 0) to `radius` (t = 1).
    Radial {
        /// Centre.
        center: Vec2,
        /// Radius where t = 1.
        radius: f32,
    },
    /// Around `center`, clockwise from `start_angle` (t = 0) through one full turn.
    Conic {
        /// Centre.
        center: Vec2,
        /// Angle (radians, clockwise from 3 o'clock) where t = 0.
        start_angle: f32,
    },
}

/// A colour ramp with up to [`MAX_STOPS`] stops. Interpolated in linear premultiplied space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gradient {
    /// Geometry.
    pub kind: GradientKind,
    /// Behaviour outside 0..1.
    pub spread: Spread,
    stops: [Stop; MAX_STOPS],
    count: u8,
}

impl Gradient {
    /// A gradient with no stops yet (add them with [`Gradient::stop`]).
    #[must_use]
    pub fn new(kind: GradientKind) -> Self {
        Self {
            kind,
            spread: Spread::Pad,
            stops: [Stop::default(); MAX_STOPS],
            count: 0,
        }
    }

    /// A linear gradient from `start` to `end`.
    #[must_use]
    pub fn linear(start: Vec2, end: Vec2) -> Self {
        Self::new(GradientKind::Linear { start, end })
    }

    /// A radial gradient.
    #[must_use]
    pub fn radial(center: Vec2, radius: f32) -> Self {
        Self::new(GradientKind::Radial { center, radius })
    }

    /// A conic (sweep) gradient.
    #[must_use]
    pub fn conic(center: Vec2, start_angle: f32) -> Self {
        Self::new(GradientKind::Conic { center, start_angle })
    }

    /// Adds a stop. Offsets are clamped to 0..=1 and kept non-decreasing; stops past
    /// [`MAX_STOPS`] are ignored.
    #[must_use]
    pub fn stop(mut self, offset: f32, color: Color) -> Self {
        let i = usize::from(self.count);
        if i >= MAX_STOPS {
            return self;
        }
        let mut offset = finite_or(offset, 0.0).clamp(0.0, 1.0);
        if i > 0 {
            offset = offset.max(self.stops[i - 1].offset);
        }
        self.stops[i] = Stop { offset, color };
        self.count += 1;
        self
    }

    /// Sets the spread mode.
    #[must_use]
    pub fn spread(mut self, spread: Spread) -> Self {
        self.spread = spread;
        self
    }

    /// The stops in use.
    #[must_use]
    pub fn stops(&self) -> &[Stop] {
        &self.stops[..usize::from(self.count)]
    }

    /// The colour at ramp position `t` (CPU reference of the shader's evaluation).
    #[must_use]
    pub fn sample(&self, t: f32) -> Color {
        let stops = self.stops();
        let Some(first) = stops.first() else {
            return Color::TRANSPARENT;
        };
        let t = match self.spread {
            Spread::Pad => t.clamp(0.0, 1.0),
            Spread::Repeat => t.rem_euclid(1.0),
            Spread::Reflect => {
                let m = t.rem_euclid(2.0);
                if m > 1.0 {
                    2.0 - m
                } else {
                    m
                }
            }
        };
        if t <= first.offset {
            return first.color;
        }
        for w in stops.windows(2) {
            let (a, b) = (w[0], w[1]);
            if t <= b.offset {
                let span = b.offset - a.offset;
                let k = if span > 1e-6 { (t - a.offset) / span } else { 1.0 };
                return a.color.lerp(b.color, k);
            }
        }
        stops[stops.len() - 1].color
    }
}

/// A handle to an image loaded into the kit's [`crate::Images`] store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageId(pub(crate) u32);

impl ImageId {
    /// The raw slot index.
    #[must_use]
    pub fn index(self) -> u32 {
        self.0
    }
}

/// How an image maps outside its rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageWrap {
    /// Clamp to the edge texels.
    #[default]
    Clamp,
    /// Tile.
    Repeat,
}

/// An image mapped onto local coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImagePaint {
    /// The image.
    pub image: ImageId,
    /// The local rectangle the `uv` sub-rectangle maps to.
    pub rect: Rect,
    /// Source sub-rectangle in 0..1 texture coordinates.
    pub uv: Rect,
    /// Multiplied with the texel (premultiplied). White = unchanged.
    pub tint: Color,
    /// Edge behaviour.
    pub wrap: ImageWrap,
}

impl ImagePaint {
    /// The whole image stretched over `rect`.
    #[must_use]
    pub fn new(image: ImageId, rect: Rect) -> Self {
        Self {
            image,
            rect,
            uv: Rect::from_xywh(0.0, 0.0, 1.0, 1.0),
            tint: Color::WHITE,
            wrap: ImageWrap::Clamp,
        }
    }

    /// With a tint (also how to fade an image: `Color::WHITE.fade(0.5)`).
    #[must_use]
    pub fn tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }

    /// Tiled.
    #[must_use]
    pub fn repeat(mut self) -> Self {
        self.wrap = ImageWrap::Repeat;
        self
    }
}

/// Where glass takes the picture it refracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Refraction {
    /// Only the kit's own layers drawn before the glass layer (cheap, always available).
    #[default]
    OwnLayers,
    /// The captured game frame behind the overlay (`refraction_source`), plus our own layers.
    /// Falls back to [`Refraction::OwnLayers`] when no source is attached.
    World,
}

/// The Liquid Glass material (tokens `glass`, spec card P8-1).
///
/// Draw glass **first** in a layer whose [`crate::LayerOptions::backdrop`] is set: it samples
/// the scene as it was when that layer began.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glass {
    /// Tint (premultiplied). Its alpha is the glass tint alpha (tokens: 28/44/64 of 255).
    pub tint: Color,
    /// Frost: 0 = clear, 1 = fully frosted (uses the layer's blurred backdrop).
    pub frost: f32,
    /// Frost noise jitter in local units (tokens: 1.4 px).
    pub frost_noise: f32,
    /// Self-lensing displacement at the edge, in local units (tokens: 24–40 px band).
    pub lens: f32,
    /// Width of the lensing band, in local units.
    pub lens_band: f32,
    /// Fresnel rim strength (0..1); index of refraction is 1.52 (R0 = 0.0426).
    pub fresnel: f32,
    /// The 1 px specular rim, top colour.
    pub rim_top: Color,
    /// The rim colour at 40% height.
    pub rim_mid: Color,
    /// The rim colour at the bottom.
    pub rim_bottom: Color,
    /// The inner rim (1 px, inset by `inner_rim_inset` px).
    pub inner_rim: Color,
    /// Inner rim inset in device pixels (tokens: 2 px).
    pub inner_rim_inset: f32,
    /// Crown sheen peak opacity (tokens: alpha 76 of 255).
    pub sheen: f32,
    /// Crown sheen height as a fraction of the shape height (tokens: 18%).
    pub sheen_height: f32,
    /// Crown sheen sag at the sides, in local units (tokens: 44 px).
    pub sheen_sag: f32,
    /// Light azimuth in radians (tokens `light`: 135° + 25° × lateral G).
    pub light_angle: f32,
    /// Optional caustic texture, screen-blended (tokens: 0.32).
    pub caustic: Option<ImageId>,
    /// Caustic strength.
    pub caustic_strength: f32,
    /// What the glass refracts.
    pub refraction: Refraction,
}

impl Default for Glass {
    /// The tokens' regular Liquid Glass (tint alpha 44, rim `#FFFFFF 92% → #38BDF8 59% →
    /// #0284C7 27%`, inner rim alpha 40, sheen 76).
    fn default() -> Self {
        Self {
            tint: Color::rgb_u32(0x0A1220).with_alpha8(44),
            frost: 0.6,
            frost_noise: 1.4,
            lens: 10.0,
            lens_band: 32.0,
            fresnel: 0.55,
            rim_top: Color::WHITE.with_alpha(0.92),
            rim_mid: Color::rgb_u32(0x38BDF8).with_alpha(0.59),
            rim_bottom: Color::rgb_u32(0x0284C7).with_alpha(0.27),
            inner_rim: Color::WHITE.with_alpha8(40),
            inner_rim_inset: 2.0,
            sheen: 76.0 / 255.0,
            sheen_height: 0.18,
            sheen_sag: 44.0,
            light_angle: crate::math::deg(-135.0),
            caustic: None,
            caustic_strength: 0.32,
            refraction: Refraction::OwnLayers,
        }
    }
}

impl Glass {
    /// Glass tinted with `color` (the palette's `glassTint` role) at `alpha` (0..=1).
    #[must_use]
    pub fn tinted(color: Color, alpha: f32) -> Self {
        Self {
            tint: color.with_alpha(alpha),
            ..Self::default()
        }
    }
}

/// What fills a shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paint {
    /// A single colour.
    Solid(Color),
    /// A colour ramp.
    Gradient(Gradient),
    /// A texture.
    Image(ImagePaint),
    /// The Liquid Glass material.
    Glass(Glass),
}

impl From<Color> for Paint {
    fn from(c: Color) -> Self {
        Paint::Solid(c)
    }
}

impl From<Gradient> for Paint {
    fn from(g: Gradient) -> Self {
        Paint::Gradient(g)
    }
}

impl From<ImagePaint> for Paint {
    fn from(i: ImagePaint) -> Self {
        Paint::Image(i)
    }
}

impl From<Glass> for Paint {
    fn from(g: Glass) -> Self {
        Paint::Glass(g)
    }
}

impl Paint {
    /// The image this paint samples, if any (it decides the draw's texture binding).
    #[must_use]
    pub fn image(&self) -> Option<ImageId> {
        match self {
            Paint::Image(i) => Some(i.image),
            Paint::Glass(g) => g.caustic,
            Paint::Solid(_) | Paint::Gradient(_) => None,
        }
    }

    /// The paint with its opacity multiplied by `k`.
    #[must_use]
    pub fn fade(self, k: f32) -> Self {
        match self {
            Paint::Solid(c) => Paint::Solid(c.fade(k)),
            Paint::Gradient(mut g) => {
                let n = usize::from(g.count);
                for s in &mut g.stops[..n] {
                    s.color = s.color.fade(k);
                }
                Paint::Gradient(g)
            }
            Paint::Image(mut i) => {
                i.tint = i.tint.fade(k);
                Paint::Image(i)
            }
            Paint::Glass(g) => Paint::Glass(g),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vec2;

    #[test]
    fn gradient_sampling() {
        let g = Gradient::linear(vec2(0.0, 0.0), vec2(1.0, 0.0))
            .stop(0.0, Color::BLACK)
            .stop(1.0, Color::WHITE);
        assert_eq!(g.sample(-1.0), Color::BLACK);
        assert_eq!(g.sample(2.0), Color::WHITE);
        assert!((g.sample(0.5).r - 0.5).abs() < 1e-6);
        let r = g.spread(Spread::Reflect);
        assert!((r.sample(1.25).r - 0.75).abs() < 1e-5);
    }

    #[test]
    fn stops_are_capped_and_monotonic() {
        let mut g = Gradient::linear(Vec2::ZERO, Vec2::ONE);
        for i in 0..12 {
            g = g.stop(1.0 - i as f32 * 0.1, Color::WHITE);
        }
        assert_eq!(g.stops().len(), MAX_STOPS);
        assert!(g.stops().windows(2).all(|w| w[0].offset <= w[1].offset));
    }

    #[test]
    fn hard_stops_give_discrete_bands() {
        let g = Gradient::linear(Vec2::ZERO, vec2(1.0, 0.0))
            .stop(0.0, Color::BLACK)
            .stop(0.5, Color::BLACK)
            .stop(0.5, Color::WHITE)
            .stop(1.0, Color::WHITE);
        assert_eq!(g.sample(0.49), Color::BLACK);
        assert_eq!(g.sample(0.51), Color::WHITE);
    }
}
