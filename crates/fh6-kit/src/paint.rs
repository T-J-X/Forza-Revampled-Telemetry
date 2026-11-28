//! Paints: what fills a shape. Solid, linear/radial/conic gradients (up to 8 stops), images,
//! optical glass and the legacy slab glass. Paint coordinates are in the shape's local space, so
//! a gradient moves, scales and rotates with the shape that uses it.
use crate::color::Color;
use crate::math::{finite_or, vec3, Rect, Vec2, Vec3};
use crate::optics::{Conductor, Dispersion, Profile};

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

/// What optical glass reflects (`docs/specs/optics.md` §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Environment {
    /// The kit's analytic studio: a key softbox, a strip light opposite it and a faint sky.
    /// Always available, so it is also the fallback when a map is missing.
    #[default]
    Studio,
    /// The same radiance from every direction (the conservation test, flat ambient light).
    Uniform,
    /// An environment map in octahedral layout (z towards the viewer in the centre).
    Map(ImageId),
}

/// A ring wave on the surface of optical glass: the ripple a liquid makes
/// (`docs/specs/optics.md` §2). At distance `r` from `center` the surface rises by
/// `amplitude · e^(−x²) · cos(2πx)` with `x = (r − radius) / wavelength`: a packet about three
/// wavelengths wide that sits on the circle of the given `radius`. Animate `radius` to send it
/// outwards and `amplitude` to let it die away; an amplitude of 0 is a still surface.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ripple {
    /// Where the wave started (local units).
    pub center: Vec2,
    /// Radius of the ring the packet sits on.
    pub radius: f32,
    /// Peak height of the wave.
    pub amplitude: f32,
    /// Distance from crest to crest.
    pub wavelength: f32,
}

/// Most plane waves a [`StandingWaves`] surface sums.
pub const MAX_STANDING_WAVES: usize = 6;

/// One plane wave of a [`StandingWaves`] surface: `A cos(k e·(p − c) + φ)`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PlaneWave {
    /// Direction of the wave vector `e` (normalised when the paint is encoded).
    pub direction: Vec2,
    /// Its amplitude `A` (local units of height).
    pub amplitude: f32,
    /// Its phase `φ` (radians).
    pub phase: f32,
}

/// A surface of standing plane waves on optical glass or liquid metal (`docs/specs/optics.md`
/// §2, standing waves): `η(p) = w(r) Σᵢ Aᵢ cos(k eᵢ·(p − c) + φᵢ)`, up to
/// [`MAX_STANDING_WAVES`] waves sharing one wavenumber `k`. `w` eases the height to 0 inside
/// `still_radius` of the centre `c` (a pinned post) over a band `still_width` wide. Two waves at
/// 90° make squares, three at 120° hexagons, six at 30° a twelve-fold quasi-pattern: the patterns
/// of a Faraday wave. Set `wavenumber`, the amplitudes and the phases every frame; the normal is
/// the exact gradient, one sine and cosine per wave per pixel. No waves (the default) is a still
/// surface.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StandingWaves {
    /// The centre `c` the phases and the still zone are measured from (local units).
    pub center: Vec2,
    /// The shared wavenumber `k = 2π / λ` (radians per local unit).
    pub wavenumber: f32,
    /// Radius of the still zone around the centre.
    pub still_radius: f32,
    /// Width of the band over which the height eases in from the still zone.
    pub still_width: f32,
    waves: [PlaneWave; MAX_STANDING_WAVES],
    count: u8,
}

impl StandingWaves {
    /// No waves yet, centred on `center` with wavenumber `wavenumber`.
    #[must_use]
    pub fn new(center: Vec2, wavenumber: f32) -> Self {
        Self {
            center,
            wavenumber,
            ..Self::default()
        }
    }

    /// `count` waves of equal `amplitude` and phase 0 whose directions are spread evenly over a
    /// half turn from `turn` (radians): 2 give squares, 3 hexagons, 6 a twelve-fold pattern.
    /// `count` is capped at [`MAX_STANDING_WAVES`].
    #[must_use]
    pub fn symmetric(center: Vec2, wavenumber: f32, count: usize, amplitude: f32, turn: f32) -> Self {
        let mut s = Self::new(center, wavenumber);
        let count = count.min(MAX_STANDING_WAVES);
        for i in 0..count {
            let (sin, cos) = (turn + core::f32::consts::PI * i as f32 / count as f32).sin_cos();
            s.push(PlaneWave {
                direction: Vec2::new(cos, sin),
                amplitude,
                phase: 0.0,
            });
        }
        s
    }

    /// With a still zone of `radius` that eases in over `width`.
    #[must_use]
    pub fn still(mut self, radius: f32, width: f32) -> Self {
        self.still_radius = radius;
        self.still_width = width;
        self
    }

    /// Adds a wave. Returns `false` (and adds nothing) when the surface already has
    /// [`MAX_STANDING_WAVES`].
    pub fn push(&mut self, wave: PlaneWave) -> bool {
        let i = usize::from(self.count);
        if i >= MAX_STANDING_WAVES {
            return false;
        }
        self.waves[i] = wave;
        self.count += 1;
        true
    }

    /// Removes every wave.
    pub fn clear(&mut self) {
        self.count = 0;
        self.waves = [PlaneWave::default(); MAX_STANDING_WAVES];
    }

    /// The waves.
    #[must_use]
    pub fn waves(&self) -> &[PlaneWave] {
        &self.waves[..usize::from(self.count)]
    }

    /// The waves, to change their amplitudes and phases in place.
    pub fn waves_mut(&mut self) -> &mut [PlaneWave] {
        &mut self.waves[..usize::from(self.count)]
    }

    /// Whether the surface moves at all: at least one wave with a non-zero amplitude and a
    /// finite wavenumber. (A wavenumber of 0 still makes a surface: a constant height that the
    /// still zone eases in.)
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.wavenumber.is_finite()
            && self
                .waves()
                .iter()
                .any(|w| w.amplitude != 0.0 && w.amplitude.is_finite())
    }

    /// The height at `p` and its gradient at full materialise (the CPU mirror of the shader,
    /// [`crate::optics::standing_waves`]).
    #[must_use]
    pub fn sample(&self, p: Vec2) -> crate::optics::WaveSample {
        crate::optics::standing_waves(self, p)
    }
}

/// Screen azimuth of the key light at rest: up and to the left (tokens `light.azimuthDeg`).
pub const LIGHT_REST_AZIMUTH: f32 = -2.356_194_5;

/// Physically based glass: every equation is in `docs/specs/optics.md`.
///
/// The surface is a height profile over the shape's outline. Per pixel the shader derives the
/// normal from it, reflects the [`Environment`] and the key light by the exact Fresnel
/// reflectance, refracts what is under the piece by Snell's law with one index per channel,
/// and absorbs along the path (Beer-Lambert).
///
/// Draw it in a layer made with [`crate::LayerOptions::glass`]: it bends the scene as it was
/// when that layer began. A [`crate::Shape::Blob`] takes the cross-section of each of its
/// pieces; any other shape takes [`OpticalGlass::profile`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpticalGlass {
    /// Refractive index for the red, green and blue channels
    /// ([`Dispersion::primaries`]).
    pub ior: [f32; 3],
    /// The cross-section of shapes other than a blob. Its `exponent` applies to blobs too.
    pub profile: Profile,
    /// Absorption coefficient per channel, per local unit of path
    /// ([`crate::optics::absorption_coefficient`]).
    pub absorption: [f32; 3],
    /// Frost of the surface for transmitted light: 0 = polished, 1 = fully frosted (uses the
    /// layer's blurred backdrop).
    pub frost: f32,
    /// Unit direction towards the key light (screen x and y, z towards the viewer).
    pub light_dir: Vec3,
    /// Irradiance of the key light per channel (linear; above 1 is a bright source).
    pub light: [f32; 3],
    /// GGX roughness of the key light's highlight (0.03 polished .. 0.3 satin).
    pub roughness: f32,
    /// What the surface mirrors.
    pub environment: Environment,
    /// Scale of the environment's radiance per channel.
    pub env_gain: [f32; 3],
    /// Rows of the rotation that takes a screen direction into the environment's frame.
    pub env_rotation: [Vec3; 3],
    /// How far the piece has materialised: scales thickness and base, so lensing and
    /// highlights grow in (never opacity).
    pub materialize: f32,
    /// What the glass bends.
    pub refraction: Refraction,
    /// A liquid ripple on the surface (still by default).
    pub ripple: Ripple,
    /// Standing waves on the surface (none by default). They add to the ripple.
    pub standing: StandingWaves,
    /// `Some` makes the piece a metal (`docs/specs/optics.md` §3, conductors): it reflects by the
    /// exact Fresnel of the complex index, transmits nothing and is opaque. `ior`,
    /// `absorption`, `frost` and `refraction` are then unused. `None` (the default) is glass.
    pub conductor: Option<Conductor>,
}

impl OpticalGlass {
    /// Clear, polished glass of the given material under the studio environment, lit from the
    /// rest direction at 40° elevation, with a flat cross-section (set [`OpticalGlass::profile`]
    /// or draw a [`crate::Shape::Blob`]).
    #[must_use]
    pub fn new(material: &Dispersion) -> Self {
        let mut glass = Self {
            ior: material.primaries(),
            profile: Profile {
                thickness: 0.0,
                bevel: 1.0,
                base: 0.0,
                exponent: 2.0,
            },
            absorption: [0.0; 3],
            frost: 0.0,
            light_dir: vec3(0.0, 0.0, 1.0),
            light: [1.0; 3],
            roughness: 0.06,
            environment: Environment::Studio,
            env_gain: [1.0; 3],
            env_rotation: [vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0)],
            materialize: 1.0,
            refraction: Refraction::OwnLayers,
            ripple: Ripple::default(),
            standing: StandingWaves::default(),
            conductor: None,
        };
        glass.aim(LIGHT_REST_AZIMUTH, crate::math::deg(40.0), Vec2::ZERO);
        glass
    }

    /// A polished liquid metal of the given index (for example [`Conductor::mercury`]) under
    /// the studio environment, lit like [`OpticalGlass::new`], with a flat cross-section. Give
    /// it [`OpticalGlass::standing`] waves or a [`OpticalGlass::ripple`] to move its surface.
    #[must_use]
    pub fn metal(conductor: Conductor) -> Self {
        let mut metal = Self::new(&Dispersion::CROWN_BK7);
        metal.conductor = Some(conductor);
        metal.roughness = 0.03;
        metal
    }

    /// Sets the cross-section used by shapes other than a blob.
    #[must_use]
    pub fn with_profile(mut self, profile: Profile) -> Self {
        self.profile = profile;
        self
    }

    /// Tints the glass: `transmittance` is the linear colour that survives `depth` local units
    /// of glass. Thicker parts are tinted more (Beer-Lambert).
    #[must_use]
    pub fn tinted(mut self, transmittance: [f32; 3], depth: f32) -> Self {
        self.absorption = transmittance.map(|t| crate::optics::absorption_coefficient(t, depth));
        self
    }

    /// Points the light rig: the key light comes from screen `azimuth` (radians, clockwise from
    /// 3 o'clock) at `elevation` above the screen plane, and the environment turns with it.
    /// `tilt` leans the whole rig about the screen's x and y axes (radians), which sweeps
    /// reflections across flat faces.
    pub fn aim(&mut self, azimuth: f32, elevation: f32, tilt: Vec2) {
        let (sa, ca) = (azimuth - LIGHT_REST_AZIMUTH).sin_cos();
        let (sx, cx) = tilt.x.sin_cos();
        let (sy, cy) = tilt.y.sin_cos();
        // world = Rz(turn) · Ry(tilt.y) · Rx(tilt.x) · env, as columns.
        let rx = [vec3(1.0, 0.0, 0.0), vec3(0.0, cx, sx), vec3(0.0, -sx, cx)];
        let ry = [vec3(cy, 0.0, -sy), vec3(0.0, 1.0, 0.0), vec3(sy, 0.0, cy)];
        let rz = [vec3(ca, sa, 0.0), vec3(-sa, ca, 0.0), vec3(0.0, 0.0, 1.0)];
        let mul = |m: &[Vec3; 3], v: Vec3| m[0] * v.x + m[1] * v.y + m[2] * v.z;
        let col = |v: Vec3| mul(&rz, mul(&ry, mul(&rx, v)));
        let cols = [
            col(vec3(1.0, 0.0, 0.0)),
            col(vec3(0.0, 1.0, 0.0)),
            col(vec3(0.0, 0.0, 1.0)),
        ];
        // The inverse of a rotation is its transpose: its rows are these columns.
        self.env_rotation = cols;
        let (se, ce) = elevation.sin_cos();
        let (sr, cr) = LIGHT_REST_AZIMUTH.sin_cos();
        let rest = vec3(ce * cr, ce * sr, se);
        self.light_dir = (cols[0] * rest.x + cols[1] * rest.y + cols[2] * rest.z).normalize_or_zero();
    }
}

/// The legacy slab glass (tokens `glass`): a tinted panel with a painted rim, sheen and a fixed
/// edge warp. It is not physically based and stays only until its remaining users are rebuilt
/// on [`OpticalGlass`]; new work uses that.
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
    /// The legacy slab glass.
    Glass(Glass),
    /// Physically based glass.
    OpticalGlass(OpticalGlass),
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

impl From<OpticalGlass> for Paint {
    fn from(g: OpticalGlass) -> Self {
        Paint::OpticalGlass(g)
    }
}

impl Paint {
    /// The image this paint samples, if any (it decides the draw's texture binding).
    #[must_use]
    pub fn image(&self) -> Option<ImageId> {
        match self {
            Paint::Image(i) => Some(i.image),
            Paint::Glass(g) => g.caustic,
            Paint::OpticalGlass(g) => match g.environment {
                Environment::Map(image) => Some(image),
                Environment::Studio | Environment::Uniform => None,
            },
            Paint::Solid(_) | Paint::Gradient(_) => None,
        }
    }

    /// True for the glass materials, which write the change they make to the scene.
    #[must_use]
    pub(crate) fn is_glass(&self) -> bool {
        matches!(self, Paint::Glass(_) | Paint::OpticalGlass(_))
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
            Paint::OpticalGlass(g) => Paint::OpticalGlass(g),
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
