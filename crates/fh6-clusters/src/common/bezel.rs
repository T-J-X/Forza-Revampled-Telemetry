//! Bezel variants (COMPONENTS.md §9): a thin, light-driven ring around an `oledBlack` cavity,
//! never a filled plate.
//!
//! - [`BezelVariant::TitaniumRing`]: conic brushed banding with two anisotropic lobes facing the
//!   light, a top-lit 1 px outer rim and an inverted inner rim.
//! - [`BezelVariant::Chamfer45`]: a thin gunmetal frame with 45° corners; its outer bevel is lit
//!   by facing (upper-left 55%, lower-right 18%) and it carries a few clocked screws.
//! - [`BezelVariant::CarbonFrame`]: a 2x2 twill ring (procedural or an optional texture) with a
//!   diagonal sheen, Allen bolts and a 1 px rim.
//! - [`BezelVariant::Frameless`]: rims only; pair it with [`Bezel::draw_falloff`].
//!
//! Every highlight follows one light angle (`common::VirtualLight::angle()`), so the metal moves
//! with lateral G. Static surfaces stay at or below about 24% luminance (tokens `materials`).
//!
//! The metal and carbon builds sit on an opaque `oledBlack` body, so the game never shows through
//! the ring and every material level reads the same over any backdrop. Material levels are
//! linear-light coverage of `ink` over that body (the kit blends linear premultiplied).
use fh6_kit::color::srgb_to_linear;
use fh6_kit::math::{finite_or, smoothstep};
use fh6_kit::{
    deg, vec2, Color, Gradient, ImageId, ImagePaint, Paint, Painter, Palette, Rect, Shadow, Shape, Spread, Stroke,
    StrokeAlign, StrokeUnit, TickRing, Vec2,
};
use std::f32::consts::{PI, TAU};

/// `2 - sqrt(2)`: how much a 45° corner cut grows when its octagon is offset outwards by 1 px.
const CHAMFER_GROWTH: f32 = 2.0 - std::f32::consts::SQRT_2;
/// Brushed titanium: lobe peak, far lobe and trough opacity of `ink` (tokens
/// `materials.brushedTitanium`: at most 24% luminance). The trough is kept deep so the light,
/// not a grey base, makes the metal read (family rule: static surfaces at or below 20%).
const TITANIUM_PEAK: f32 = 0.24;
/// See [`TITANIUM_PEAK`].
const TITANIUM_FAR: f32 = 0.08;
/// See [`TITANIUM_PEAK`].
const TITANIUM_TROUGH: f32 = 0.012;
/// Turned bevels: outer (facing the light) and inner (inverted) lobe levels, as
/// `(peak, far, trough)`.
const BEVEL_OUTER: (f32, f32, f32) = (0.34, 0.10, 0.016);
/// See [`BEVEL_OUTER`].
const BEVEL_INNER: (f32, f32, f32) = (0.26, 0.08, 0.012);
/// Brushed banding (tokens fallback: 48 bands).
const TITANIUM_BANDS: u32 = 48;
/// Band amplitude (tokens fallback: 6% on screen). The kit blends in linear light, where 6%
/// over the dark trough would read as a knurl; 1.2% coverage stays a brushed shimmer.
const TITANIUM_BAND_ALPHA: f32 = 0.012;
/// Width of the turned bevels at each edge of a titanium ring, as a fraction of its width.
const BEVEL_PCT: f32 = 0.2;
/// Chamfer frame: bevel facing levels as seen on screen (COMPONENTS.md §9: lit 55%, shadowed
/// 18%); [`on_screen`] converts them to the kit's linear-light opacity.
pub const CHAMFER_LIT: f32 = 0.55;
/// See [`CHAMFER_LIT`].
pub const CHAMFER_SHADOW: f32 = 0.18;
/// Chamfer frame: the flat gunmetal face, lit and far side, as seen on screen (about the tokens'
/// `brushedTitanium.base` `#2A2E35` on the lit side).
const GUNMETAL_LIT: f32 = 0.22;
/// See [`GUNMETAL_LIT`].
const GUNMETAL_DARK: f32 = 0.09;
/// Share of a chamfer frame's width that is the outer 45° bevel.
const CHAMFER_BEVEL_PCT: f32 = 0.38;
/// Carbon twill pitch (tokens `materials.carbonTwill.weavePx`).
const CARBON_WEAVE: f32 = 12.0;
/// Carbon base and weave contrast (opacity of `ink`, linear light: a near-black resin with the
/// tows a few levels above it).
const CARBON_BASE: f32 = 0.009;
/// See [`CARBON_BASE`].
const CARBON_WEAVE_ALPHA: f32 = 0.02;
/// Carbon diagonal sheen (tokens `materials.carbonTwill.sheenAlpha`: 18/255 on screen over the
/// resin; linear light).
const CARBON_SHEEN: f32 = 0.025;
/// Texture tile size for an optional carbon image: four twill repeats.
const CARBON_TILE: f32 = CARBON_WEAVE * 4.0;
/// Falloff steps for a rounded-rect cavity (nested inner strokes): 12 keeps each step under
/// 10% of the display level, so the ramp reads smooth.
const FALLOFF_STEPS: u32 = 12;
/// Falloff stops for a circular cavity (plus one transparent stop; the kit allows 8).
const CIRCLE_FALLOFF_STOPS: u32 = 7;

/// Outward normals of the octagon segments from [`octagon`] (top, then clockwise).
const OCTAGON_NORMALS: [Vec2; 8] = {
    const D: f32 = std::f32::consts::FRAC_1_SQRT_2;
    [
        Vec2::new(0.0, -1.0),
        Vec2::new(D, -D),
        Vec2::new(1.0, 0.0),
        Vec2::new(D, D),
        Vec2::new(0.0, 1.0),
        Vec2::new(-D, D),
        Vec2::new(-1.0, 0.0),
        Vec2::new(-D, -D),
    ]
};

/// The cavity a bezel surrounds (local px). The bezel grows outwards from this edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BezelForm {
    /// A round dial.
    Circle {
        /// Centre.
        center: Vec2,
        /// Cavity radius.
        radius: f32,
    },
    /// A rounded rectangle (for [`BezelVariant::Chamfer45`], `radius` is the 45° corner cut).
    RoundRect {
        /// Cavity bounds.
        rect: Rect,
        /// Corner radius, or the corner cut of a chamfered frame.
        radius: f32,
    },
}

impl BezelForm {
    /// A round cavity.
    #[must_use]
    pub fn circle(center: Vec2, radius: f32) -> Self {
        Self::Circle {
            center,
            radius: finite_or(radius, 0.0).max(0.0),
        }
    }

    /// A rounded-rect cavity.
    #[must_use]
    pub fn round_rect(rect: Rect, radius: f32) -> Self {
        Self::RoundRect {
            rect,
            radius: finite_or(radius, 0.0).max(0.0),
        }
    }

    /// The centre.
    #[must_use]
    pub fn center(&self) -> Vec2 {
        match *self {
            Self::Circle { center, .. } => center,
            Self::RoundRect { rect, .. } => rect.center(),
        }
    }

    /// The same form offset outwards by `d` (negative shrinks); a rounded rect keeps a
    /// concentric corner.
    #[must_use]
    pub fn grown(&self, d: f32) -> Self {
        let d = finite_or(d, 0.0);
        match *self {
            Self::Circle { center, radius } => Self::Circle {
                center,
                radius: (radius + d).max(0.0),
            },
            Self::RoundRect { rect, radius } => Self::RoundRect {
                rect: rect.expand(d),
                radius: (radius + d).max(0.0),
            },
        }
    }

    /// The form as a kit shape.
    #[must_use]
    pub fn shape(&self) -> Shape<'static> {
        match *self {
            Self::Circle { center, radius } => Shape::circle(center, radius),
            Self::RoundRect { rect, radius } => Shape::round_rect(rect, radius),
        }
    }

    /// Bounds of the form.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        match *self {
            Self::Circle { center, radius } => Rect::from_center_size(center, Vec2::splat(radius * 2.0)),
            Self::RoundRect { rect, .. } => rect,
        }
    }
}

/// The bezel build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BezelVariant {
    /// Brushed titanium ring (Classic Analog).
    TitaniumRing,
    /// Thin gunmetal frame with 45° corners (Sci-Fi VFD, race hardware).
    Chamfer45,
    /// Carbon twill ring with Allen bolts (Race Dash).
    CarbonFrame {
        /// A tileable twill texture (for example `assets/classic/carbon_twill_lit.png`, loaded
        /// once by the cluster); `None` draws the procedural twill.
        weave: Option<ImageId>,
    },
    /// Rims only (Lucid, Modern, ADAS).
    Frameless,
}

/// A bezel: a variant and the ring width (local px).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bezel {
    /// The build.
    pub variant: BezelVariant,
    /// Ring width outwards from the cavity edge (family Classic: 20 px for a 242 px face).
    pub width: f32,
}

impl Bezel {
    /// A bezel of `variant`, `width` px wide.
    #[must_use]
    pub fn new(variant: BezelVariant, width: f32) -> Self {
        Self {
            variant,
            width: finite_or(width, 0.0).max(0.0),
        }
    }

    /// The outer edge of the bezel around `form`.
    #[must_use]
    pub fn outer(&self, form: BezelForm) -> BezelForm {
        form.grown(self.width)
    }

    /// Fills the cavity with `oledBlack` (a chamfered frame's cavity is its octagon).
    pub fn fill_cavity(&self, p: &mut Painter<'_>, form: BezelForm, palette: &Palette) {
        if self.variant == BezelVariant::Chamfer45 {
            let (rect, cut) = chamfer_frame(form);
            let inner = octagon(rect, cut);
            p.fill(&Shape::Polygon { points: &inner }, palette.roles.oled_black);
        } else {
            p.fill(&form.shape(), palette.roles.oled_black);
        }
    }

    /// Draws the bezel around `form`, lit from `light` (radians, screen convention).
    pub fn draw(&self, p: &mut Painter<'_>, form: BezelForm, palette: &Palette, light: f32) {
        let light = finite_or(light, deg(-135.0));
        // `width` is a public field: sanitise it here as `new` does.
        let this = Self::new(self.variant, self.width);
        match this.variant {
            BezelVariant::TitaniumRing => this.draw_titanium(p, form, palette, light),
            BezelVariant::Chamfer45 => this.draw_chamfer(p, form, palette, light),
            BezelVariant::CarbonFrame { weave } => this.draw_carbon(p, form, weave, palette, light),
            BezelVariant::Frameless => Self::draw_frameless(p, form, palette, light),
        }
    }

    /// Fades content into the cavity edge over `depth` px (draw after the content). This is the
    /// "content falloff" of the frameless variant; any variant may use it.
    ///
    /// The ramp is even as seen on screen (a smoothstep in display levels), reaching true black
    /// at the edge. The kit blends in linear light, so a plain linear alpha ramp would leave the
    /// content at about a third of its brightness right at the edge.
    pub fn draw_falloff(p: &mut Painter<'_>, form: BezelForm, palette: &Palette, depth: f32) {
        let black = palette.roles.oled_black;
        match form {
            BezelForm::Circle { center, radius } => {
                let d = finite_or(depth, 0.0).clamp(0.0, radius);
                if d <= 0.0 || radius <= 0.0 {
                    return;
                }
                // Transparent inside, then the ramp sampled at 6 stops toward the edge.
                let from = (radius - d) / radius;
                let mut paint = Gradient::radial(center, radius).stop(0.0, black.with_alpha(0.0));
                for i in 0..CIRCLE_FALLOFF_STOPS {
                    let s = i as f32 / (CIRCLE_FALLOFF_STOPS - 1) as f32;
                    paint = paint.stop(from + (1.0 - from) * s, black.with_alpha(falloff_alpha(1.0 - s)));
                }
                p.fill(&Shape::ring(center, radius - d * 0.5, d), paint);
            }
            BezelForm::RoundRect { .. } => {
                let d = finite_or(depth, 0.0).max(0.0);
                if d <= 0.0 {
                    return;
                }
                // Nested inner strokes, widest first: band `j` (0 at the edge) lies under the
                // strokes wider than it, so stroke `i` takes the ratio between the transmissions
                // its band edge needs.
                for i in (1..=FALLOFF_STEPS).rev() {
                    let w = d * i as f32 / FALLOFF_STEPS as f32;
                    let alpha = falloff_step_alpha(i);
                    p.stroke(&form.shape(), black.with_alpha(alpha), Stroke::inner(w));
                }
            }
        }
    }

    fn draw_titanium(&self, p: &mut Painter<'_>, form: BezelForm, palette: &Palette, light: f32) {
        let roles = palette.roles;
        let w = self.width;
        if w <= 0.0 {
            return;
        }
        let c = form.center();
        let ink = roles.ink;
        // The body: opaque black metal, then two anisotropic lobes, toward the light and
        // opposite it.
        band(p, form, 0.0, w, roles.oled_black);
        band(
            p,
            form,
            0.0,
            w,
            brushed(
                c,
                light,
                ink.with_alpha(TITANIUM_PEAK),
                ink.with_alpha(TITANIUM_FAR),
                ink.with_alpha(TITANIUM_TROUGH),
            ),
        );
        // Brushed banding: 48 radial bands whose strength follows the lobes.
        if let BezelForm::Circle { center, radius } = form {
            let pitch = TAU * (radius + w * 0.5) / TITANIUM_BANDS as f32;
            let a = TITANIUM_BAND_ALPHA;
            p.fill(
                &Shape::Ticks(
                    TickRing::new(
                        center,
                        radius + w,
                        light,
                        TAU * (TITANIUM_BANDS - 1) as f32 / TITANIUM_BANDS as f32,
                        TITANIUM_BANDS,
                    )
                    .minor(w, pitch * 0.5),
                ),
                brushed(
                    c,
                    light,
                    ink.with_alpha(a),
                    ink.with_alpha(a * 0.5),
                    ink.with_alpha(a * 0.15),
                ),
            );
        }
        // Turning marks: two concentric hairlines across the face of the ring.
        for f in [0.38, 0.62] {
            p.stroke(
                &form.grown(w * f).shape(),
                brushed(
                    c,
                    light,
                    ink.with_alpha(0.10),
                    ink.with_alpha(0.05),
                    ink.with_alpha(0.015),
                ),
                device_stroke(StrokeAlign::Center),
            );
        }
        // Turned bevels: the outer one faces the light, the inner one faces the cavity (inverted).
        let bevel = w * BEVEL_PCT;
        for (d0, d1, facing, (peak, far, trough)) in [
            (w - bevel, w, light, BEVEL_OUTER),
            (0.0, bevel, light + PI, BEVEL_INNER),
        ] {
            band(
                p,
                form,
                d0,
                d1,
                brushed(
                    c,
                    facing,
                    ink.with_alpha(peak),
                    ink.with_alpha(far),
                    ink.with_alpha(trough),
                ),
            );
        }
        // 1 px rims: top-lit outer, inverted inner.
        p.stroke(
            &form.grown(w).shape(),
            lit_rim(c, light, roles.rim_specular, roles.hairline.with_alpha(0.10)),
            Stroke::rim_px(1.0),
        );
        p.stroke(
            &form.shape(),
            lit_rim(
                c,
                light + PI,
                roles.rim_specular.with_alpha(0.7),
                roles.hairline.with_alpha(0.06),
            ),
            device_stroke(StrokeAlign::Outer),
        );
    }

    fn draw_chamfer(&self, p: &mut Painter<'_>, form: BezelForm, palette: &Palette, light: f32) {
        let roles = palette.roles;
        let w = self.width;
        if w <= 0.0 {
            return;
        }
        let (rect, cut) = chamfer_frame(form);
        let c = rect.center();
        let l = Vec2::from_angle(light);
        let inner = octagon(rect, cut);
        let mid = octagon_grown(rect, cut, w * 0.5);
        let bevel_from = octagon_grown(rect, cut, w * (1.0 - CHAMFER_BEVEL_PCT));
        let outer = octagon_grown(rect, cut, w);

        // Flat gunmetal face: one closed polyline on the mid-line over an opaque black body,
        // shaded along the light.
        let ext = rect.size().length() * 0.5 + w;
        p.fill(
            &Shape::Polyline {
                points: &mid,
                width: w,
                closed: true,
            },
            roles.oled_black,
        );
        p.fill(
            &Shape::Polyline {
                points: &mid,
                width: w,
                closed: true,
            },
            Gradient::linear(c + l * ext, c - l * ext)
                .stop(0.0, roles.ink.with_alpha(on_screen(GUNMETAL_LIT)))
                .stop(1.0, roles.ink.with_alpha(on_screen(GUNMETAL_DARK))),
        );
        // Outer 45° bevel: each of the 8 facets lit by its facing.
        for i in 0..8 {
            let j = (i + 1) % 8;
            let quad = [bevel_from[i], bevel_from[j], outer[j], outer[i]];
            p.fill(
                &Shape::Polygon { points: &quad },
                roles
                    .ink
                    .with_alpha(on_screen(chamfer_level(OCTAGON_NORMALS[i], light))),
            );
        }
        // Rationed screws: one near each end of the top and bottom sides, clocked alike.
        let screw_r = w * 0.2;
        let d = w * (1.0 - CHAMFER_BEVEL_PCT) * 0.5;
        let inset = cut + w * 1.4;
        if rect.width() > inset * 2.0 + screw_r * 4.0 {
            for (x, y) in [
                (rect.min.x + inset, rect.min.y - d),
                (rect.max.x - inset, rect.min.y - d),
                (rect.min.x + inset, rect.max.y + d),
                (rect.max.x - inset, rect.max.y + d),
            ] {
                slotted_screw(p, vec2(x, y), screw_r, palette, light);
            }
        }
        // 1 px rims: top-lit outer edge, inverted lip at the cavity.
        p.stroke(
            &Shape::Polygon { points: &outer },
            lit_rim(c, light, roles.rim_specular, roles.hairline.with_alpha(0.10)),
            Stroke::rim_px(1.0),
        );
        p.stroke(
            &Shape::Polygon { points: &inner },
            lit_rim(
                c,
                light + PI,
                roles.rim_specular.with_alpha(0.6),
                roles.hairline.with_alpha(0.05),
            ),
            device_stroke(StrokeAlign::Outer),
        );
    }

    fn draw_carbon(&self, p: &mut Painter<'_>, form: BezelForm, weave: Option<ImageId>, palette: &Palette, light: f32) {
        let roles = palette.roles;
        let w = self.width;
        if w <= 0.0 {
            return;
        }
        let c = form.center();
        let ink = roles.ink;
        let l = Vec2::from_angle(light);
        band(p, form, 0.0, w, roles.oled_black);
        band(p, form, 0.0, w, ink.with_alpha(CARBON_BASE));
        match weave {
            Some(image) => {
                let tile = Rect::from_center_size(c, Vec2::splat(CARBON_TILE));
                band(
                    p,
                    form,
                    0.0,
                    w,
                    ImagePaint::new(image, tile).repeat().tint(ink.with_alpha(0.55)),
                );
            }
            None => {
                // 2x2 twill: diagonal tows one way, the crossing tows at half contrast.
                let a = CARBON_WEAVE_ALPHA;
                for (dir, pitch, alpha) in [
                    (vec2(1.0, 1.0), CARBON_WEAVE, a),
                    (vec2(1.0, -1.0), CARBON_WEAVE * 2.0, a * 0.6),
                ] {
                    let d = dir.normalize_or_zero() * pitch;
                    band(
                        p,
                        form,
                        0.0,
                        w,
                        Gradient::linear(c, c + d)
                            .spread(Spread::Repeat)
                            .stop(0.0, ink.with_alpha(0.0))
                            .stop(0.35, ink.with_alpha(alpha))
                            .stop(0.5, ink.with_alpha(alpha))
                            .stop(0.85, ink.with_alpha(0.0))
                            .stop(1.0, ink.with_alpha(0.0)),
                    );
                }
            }
        }
        // Diagonal sheen across the light side.
        let ext = form.grown(w).bounds().size().length() * 0.5;
        band(
            p,
            form,
            0.0,
            w,
            Gradient::linear(c + l * ext, c - l * ext)
                .stop(0.0, ink.with_alpha(CARBON_SHEEN * 0.4))
                .stop(0.2, ink.with_alpha(CARBON_SHEEN))
                .stop(0.42, ink.with_alpha(0.0))
                .stop(1.0, ink.with_alpha(0.0)),
        );
        // Allen bolts on the diagonals.
        let bolt_r = w * 0.3;
        for corner in 0..4 {
            let a = deg(45.0 + 90.0 * corner as f32);
            let at = match form {
                BezelForm::Circle { center, radius } => Vec2::polar(center, radius + w * 0.5, a),
                BezelForm::RoundRect { rect, radius } => {
                    // The kit clamps a corner radius to the shorter half side; so does the bolt.
                    let half = rect.size() * 0.5;
                    let radius = radius.min(half.x).min(half.y).max(0.0);
                    let dir = Vec2::from_angle(a);
                    let corner_center =
                        c + vec2(dir.x.signum() * (half.x - radius), dir.y.signum() * (half.y - radius));
                    corner_center + dir * (radius + w * 0.5)
                }
            };
            allen_bolt(p, at, bolt_r, palette, light);
        }
        // 1 px rims.
        p.stroke(
            &form.grown(w).shape(),
            lit_rim(c, light, roles.rim_specular, roles.hairline.with_alpha(0.10)),
            Stroke::rim_px(1.0),
        );
        p.stroke(
            &form.shape(),
            lit_rim(
                c,
                light + PI,
                roles.rim_specular.with_alpha(0.5),
                roles.hairline.with_alpha(0.05),
            ),
            device_stroke(StrokeAlign::Outer),
        );
    }

    fn draw_frameless(p: &mut Painter<'_>, form: BezelForm, palette: &Palette, light: f32) {
        let roles = palette.roles;
        let c = form.center();
        // A glass edge: a lit outer rim that dissolves away from the light, and its fainter
        // inverted back face 2 px out.
        p.stroke(
            &form.shape(),
            lit_rim(
                c,
                light,
                roles.rim_specular.with_alpha(0.7),
                roles.hairline.with_alpha(0.0),
            ),
            device_stroke(StrokeAlign::Outer),
        );
        p.stroke(
            &form.grown(2.0).shape(),
            lit_rim(c, light + PI, roles.hairline, roles.hairline.with_alpha(0.0)),
            device_stroke(StrokeAlign::Outer),
        );
    }
}

/// A conic gradient around `center` for a 1 px rim: a narrow specular lobe at `peak` facing
/// `light` (tokens `light`: rim exponent 24–48) falling to `floor` on the far side.
#[must_use]
pub fn lit_rim(center: Vec2, light: f32, peak: Color, floor: Color) -> Gradient {
    let shoulder = peak.lerp(floor, 0.45);
    let flank = peak.lerp(floor, 0.8);
    Gradient::conic(center, light)
        .stop(0.0, peak)
        .stop(0.035, shoulder)
        .stop(0.12, flank)
        .stop(0.3, floor)
        .stop(0.7, floor)
        .stop(0.88, flank)
        .stop(0.965, shoulder)
        .stop(1.0, peak)
}

/// The brightness (opacity of `ink`) of a chamfer facet with outward `normal` under the light
/// at azimuth `light`: [`CHAMFER_LIT`] facing it, [`CHAMFER_SHADOW`] facing away.
#[must_use]
pub fn chamfer_level(normal: Vec2, light: f32) -> f32 {
    let facing = (normal.normalize_or_zero().dot(Vec2::from_angle(light)) + 1.0) * 0.5;
    CHAMFER_SHADOW + (CHAMFER_LIT - CHAMFER_SHADOW) * facing.clamp(0.0, 1.0).powf(1.5)
}

/// The kit's linear-light opacity of `ink` over an opaque black body that shows `level`
/// (0..=1) on screen. The kit blends linear premultiplied, so a design level of 18% is about
/// 2.7% coverage.
#[must_use]
pub fn on_screen(level: f32) -> f32 {
    srgb_to_linear(finite_or(level, 0.0).clamp(0.0, 1.0))
}

/// Black coverage of the content falloff at `x` (0 at the cavity edge, 1 at the falloff's inner
/// end): it leaves the content at a smoothstep display level, converted to the kit's linear
/// light.
fn falloff_alpha(x: f32) -> f32 {
    1.0 - falloff_transmission(x)
}

/// The linear-light share of the content that survives the falloff at `x` (see
/// [`falloff_alpha`]).
fn falloff_transmission(x: f32) -> f32 {
    srgb_to_linear(smoothstep(0.0, 1.0, finite_or(x, 1.0).clamp(0.0, 1.0)))
}

/// Opacity of the `i`-th nested falloff stroke (`1..=FALLOFF_STEPS`, `i` steps wide). Band `j`
/// (0 at the edge) lies under strokes `j + 1..=FALLOFF_STEPS`, so its transmission is the
/// product of theirs; each stroke supplies the ratio between neighbouring band targets.
fn falloff_step_alpha(i: u32) -> f32 {
    let n = FALLOFF_STEPS as f32;
    let target = |band: u32| {
        if band >= FALLOFF_STEPS {
            1.0
        } else {
            falloff_transmission((band as f32 + 0.5) / n)
        }
    };
    let inner = target(i).max(1e-6);
    (1.0 - target(i.saturating_sub(1)) / inner).clamp(0.0, 1.0)
}

/// The 8 corners of `rect` with 45° corner cuts of `cut` px, clockwise from the top-left end
/// of the top side.
#[must_use]
pub fn octagon(rect: Rect, cut: f32) -> [Vec2; 8] {
    let c = finite_or(cut, 0.0).clamp(0.0, rect.width().min(rect.height()) * 0.5);
    let (a, b) = (rect.min, rect.max);
    [
        vec2(a.x + c, a.y),
        vec2(b.x - c, a.y),
        vec2(b.x, a.y + c),
        vec2(b.x, b.y - c),
        vec2(b.x - c, b.y),
        vec2(a.x + c, b.y),
        vec2(a.x, b.y - c),
        vec2(a.x, a.y + c),
    ]
}

/// The octagon of `rect` / `cut` offset outwards by `d`, every side parallel at distance `d`.
fn octagon_grown(rect: Rect, cut: f32, d: f32) -> [Vec2; 8] {
    octagon(rect.expand(d), cut + d * CHAMFER_GROWTH)
}

/// The rect and corner cut of a chamfered frame around `form`; a circle gets the regular
/// octagon whose sides touch it.
fn chamfer_frame(form: BezelForm) -> (Rect, f32) {
    match form {
        BezelForm::Circle { center, radius } => (
            Rect::from_center_size(center, Vec2::splat(radius * 2.0)),
            radius * CHAMFER_GROWTH,
        ),
        BezelForm::RoundRect { rect, radius } => (rect, radius),
    }
}

/// A conic brushed-metal paint for a circularly turned surface: two anisotropic lobes, `peak`
/// facing `light` (about ±45°, with a bright core) and a narrower `far` lobe opposite it,
/// `trough` everywhere else. The seam sits in a trough, so the ramp is continuous.
fn brushed(center: Vec2, light: f32, peak: Color, far: Color, trough: Color) -> Gradient {
    let shoulder = peak.lerp(trough, 0.6);
    // t = 0 at a quarter turn past the light: t = 0.25 faces away from it, t = 0.75 faces it.
    Gradient::conic(center, light + PI * 0.5)
        .stop(0.17, trough)
        .stop(0.25, far)
        .stop(0.33, trough)
        .stop(0.62, trough)
        .stop(0.70, shoulder)
        .stop(0.75, peak)
        .stop(0.80, shoulder)
        .stop(0.88, trough)
}

/// A 1 device-pixel stroke with the given alignment.
fn device_stroke(align: StrokeAlign) -> Stroke {
    Stroke {
        width: 1.0,
        align,
        unit: StrokeUnit::DevicePx,
    }
}

/// Fills the band between offsets `d0` and `d1` outside `form`.
fn band(p: &mut Painter<'_>, form: BezelForm, d0: f32, d1: f32, paint: impl Into<Paint>) {
    let t = d1 - d0;
    if t <= 0.0 {
        return;
    }
    let mid = form.grown((d0 + d1) * 0.5);
    match mid {
        BezelForm::Circle { center, radius } => p.fill(&Shape::ring(center, radius, t), paint),
        BezelForm::RoundRect { .. } => p.stroke(&mid.shape(), paint, Stroke::new(t)),
    }
}

/// An opaque neutral at `level` of `ink` over `oled_black`.
fn metal(palette: &Palette, level: f32) -> Color {
    palette.roles.oled_black.lerp(palette.roles.ink, level.clamp(0.0, 1.0))
}

/// A domed fastener head lit from `light`, with a contact shadow and a 1 px rim.
fn fastener_head(p: &mut Painter<'_>, at: Vec2, r: f32, palette: &Palette, light: f32) {
    let roles = palette.roles;
    let l = Vec2::from_angle(light);
    let head = Shape::circle(at, r);
    p.shadow(
        &head,
        Shadow::new(-l * (r * 0.35), r * 0.35, roles.oled_black.with_alpha(0.7)),
    );
    p.fill(
        &head,
        Gradient::radial(at + l * (r * 0.45), r * 1.5)
            .stop(0.0, metal(palette, 0.42))
            .stop(0.5, metal(palette, 0.14))
            .stop(1.0, metal(palette, 0.05)),
    );
    p.stroke(
        &head,
        lit_rim(
            at,
            light,
            roles.rim_specular.with_alpha(0.75),
            roles.hairline.with_alpha(0.05),
        ),
        Stroke::rim_px(1.0),
    );
}

/// A slotted screw, clocked at 30°.
fn slotted_screw(p: &mut Painter<'_>, at: Vec2, r: f32, palette: &Palette, light: f32) {
    fastener_head(p, at, r, palette, light);
    let dir = Vec2::from_angle(deg(30.0));
    p.fill(
        &Shape::line(at - dir * (r * 0.72), at + dir * (r * 0.72), r * 0.3),
        palette.roles.oled_black,
    );
}

/// An Allen (hex socket) bolt.
fn allen_bolt(p: &mut Painter<'_>, at: Vec2, r: f32, palette: &Palette, light: f32) {
    fastener_head(p, at, r, palette, light);
    let mut hex = [Vec2::ZERO; 6];
    for (i, v) in hex.iter_mut().enumerate() {
        *v = Vec2::polar(at, r * 0.5, deg(15.0 + 60.0 * i as f32));
    }
    let socket = Shape::Polygon { points: &hex };
    p.fill(&socket, palette.roles.oled_black);
    p.stroke(
        &socket,
        lit_rim(
            at,
            light + PI,
            palette.roles.rim_specular.with_alpha(0.45),
            palette.roles.hairline.with_alpha(0.0),
        ),
        Stroke::rim_px(1.0),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Signed distance from `pt` to the line through `a` and `b`.
    fn line_distance(pt: Vec2, a: Vec2, b: Vec2) -> f32 {
        let d = (b - a).normalize_or_zero();
        (pt - a).cross(d).abs()
    }

    #[test]
    fn grown_octagon_sides_are_parallel_at_distance() {
        let rect = Rect::from_xywh(-300.0, -100.0, 600.0, 200.0);
        let inner = octagon(rect, 28.0);
        let outer = octagon_grown(rect, 28.0, 14.0);
        for i in 0..8 {
            let j = (i + 1) % 8;
            let da = line_distance(outer[i], inner[i], inner[j]);
            let db = line_distance(outer[j], inner[i], inner[j]);
            assert!(
                (da - 14.0).abs() < 1e-3 && (db - 14.0).abs() < 1e-3,
                "side {i}: {da} {db}"
            );
        }
    }

    #[test]
    fn circle_chamfer_is_a_regular_octagon_touching_the_circle() {
        let center = vec2(10.0, -20.0);
        let (rect, cut) = chamfer_frame(BezelForm::circle(center, 200.0));
        let o = octagon(rect, cut);
        let first = (o[1] - o[0]).length();
        for i in 0..8 {
            let j = (i + 1) % 8;
            assert!(((o[j] - o[i]).length() - first).abs() < 1e-2, "side {i} length");
            assert!(
                (line_distance(center, o[i], o[j]) - 200.0).abs() < 1e-2,
                "side {i} touches"
            );
        }
    }

    #[test]
    fn octagon_normals_point_outwards() {
        let rect = Rect::from_xywh(-50.0, -30.0, 100.0, 60.0);
        let o = octagon(rect, 10.0);
        for i in 0..8 {
            let mid = (o[i] + o[(i + 1) % 8]) * 0.5;
            assert!(mid.dot(OCTAGON_NORMALS[i]) > 0.0, "normal {i}");
            let edge = o[(i + 1) % 8] - o[i];
            assert!(edge.dot(OCTAGON_NORMALS[i]).abs() < 1e-3, "normal {i} is perpendicular");
        }
    }

    #[test]
    fn chamfer_levels_follow_the_light() {
        let light = deg(-135.0);
        let upper_left = chamfer_level(OCTAGON_NORMALS[7], light);
        let lower_right = chamfer_level(OCTAGON_NORMALS[3], light);
        assert!((upper_left - CHAMFER_LIT).abs() < 1e-4);
        assert!((lower_right - CHAMFER_SHADOW).abs() < 1e-4);
        let top = chamfer_level(OCTAGON_NORMALS[0], light);
        let right = chamfer_level(OCTAGON_NORMALS[2], light);
        assert!(upper_left > top && top > right && right > lower_right);
        // The light swings 25° with lateral G: the lit facet moves with it.
        let swung = deg(-110.0);
        assert!(chamfer_level(OCTAGON_NORMALS[0], swung) > chamfer_level(OCTAGON_NORMALS[0], light));
    }

    #[test]
    fn rim_peaks_toward_the_light() {
        let t = fh6_kit::Tokens::default();
        let pal = Palette::family(&t, "classicAnalog");
        let g = lit_rim(Vec2::ZERO, deg(-135.0), pal.roles.rim_specular, pal.roles.hairline);
        let peak = g.sample(0.0).a;
        assert!((g.sample(1.0).a - peak).abs() < 1e-6, "wraps seamlessly");
        assert!(g.sample(0.5).a < peak * 0.5, "far side is dim");
        assert!(g.sample(0.02).a > g.sample(0.1).a, "narrow lobe");
    }

    #[test]
    fn brushed_lobes_face_the_light_and_wrap_seamlessly() {
        let ink = fh6_kit::Color::WHITE;
        let (peak, far, trough) = (ink.with_alpha(0.24), ink.with_alpha(0.11), ink.with_alpha(0.035));
        let g = brushed(Vec2::ZERO, deg(-135.0), peak, far, trough);
        // t = 0.75 is the light's own azimuth, t = 0.25 the opposite one.
        assert!((g.sample(0.75).a - 0.24).abs() < 1e-6, "near lobe peaks at the light");
        assert!((g.sample(0.25).a - 0.11).abs() < 1e-6, "far lobe opposite");
        assert!((g.sample(0.5).a - 0.035).abs() < 1e-6, "trough a quarter turn away");
        assert!((g.sample(0.0).a - g.sample(1.0).a).abs() < 1e-6, "no seam");
        // Anisotropic: the near lobe is wider than the far one but both stay narrow.
        assert!(g.sample(0.75 - 0.06).a > g.sample(0.25 - 0.06).a);
        assert!(
            (g.sample(0.75 - 0.14).a - 0.035).abs() < 1e-6,
            "near lobe within about ±50°"
        );
    }

    #[test]
    fn on_screen_levels_convert_to_linear_coverage() {
        assert!((on_screen(0.55) - 0.2633).abs() < 1e-3);
        assert!((on_screen(0.18) - 0.0273).abs() < 1e-3);
        assert_eq!(on_screen(f32::NAN), 0.0);
        assert_eq!(on_screen(2.0), 1.0);
    }

    #[test]
    fn forms_grow_concentrically() {
        let c = BezelForm::circle(Vec2::ZERO, 242.0).grown(20.0);
        assert_eq!(c, BezelForm::circle(Vec2::ZERO, 262.0));
        let r = BezelForm::round_rect(Rect::from_xywh(0.0, 0.0, 100.0, 50.0), 12.0).grown(4.0);
        assert_eq!(r, BezelForm::round_rect(Rect::from_xywh(-4.0, -4.0, 108.0, 58.0), 16.0));
        let shrunk = BezelForm::round_rect(Rect::from_xywh(0.0, 0.0, 100.0, 50.0), 12.0).grown(-20.0);
        assert!(matches!(shrunk, BezelForm::RoundRect { radius, .. } if radius == 0.0));
        assert_eq!(Bezel::new(BezelVariant::TitaniumRing, f32::NAN).width, 0.0);
    }

    /// The nested strokes compose to the target ramp band by band: true black at the edge,
    /// rising monotonically, about half brightness on screen half way in, untouched inside.
    #[test]
    fn falloff_ramp_is_even_on_screen_and_black_at_the_edge() {
        let n = FALLOFF_STEPS;
        let mut prev = 0.0;
        for j in 0..n {
            let t: f32 = (j + 1..=n).map(|i| 1.0 - falloff_step_alpha(i)).product();
            let want = falloff_transmission((j as f32 + 0.5) / n as f32);
            assert!((t - want).abs() < 1e-4, "band {j}: {t} vs {want}");
            assert!(t > prev, "band {j} is brighter than the band outside it");
            prev = t;
        }
        assert!(falloff_transmission(0.5 / n as f32) < 1e-3, "near black at the edge");
        assert!((falloff_alpha(1.0)).abs() < 1e-4, "untouched at the inner end");
        assert!((falloff_alpha(0.0) - 1.0).abs() < 1e-6, "black at the edge");
        // Half way in, the display level is half (linear light about 0.21).
        assert!((falloff_transmission(0.5) - srgb_to_linear(0.5)).abs() < 1e-5);
        assert!((0..=n).all(|i| (0.0..=1.0).contains(&falloff_step_alpha(i))));
    }

    #[test]
    fn octagon_cut_is_clamped() {
        let rect = Rect::from_xywh(0.0, 0.0, 40.0, 20.0);
        let o = octagon(rect, 100.0);
        assert!(o.iter().all(|v| rect.contains(*v)));
    }
}
