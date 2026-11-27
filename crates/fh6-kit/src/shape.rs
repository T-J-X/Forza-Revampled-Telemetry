//! Shapes: analytic signed-distance primitives, anti-aliased at any scale.
//!
//! Every shape is described in the painter's current local coordinates (4K reference pixels
//! around the cluster anchor, by default). The GPU evaluates the exact distance per pixel, so
//! edges stay crisp at any user scale, and the same shape can be filled, stroked (inner, centred
//! or outer), glowed or shadowed.
use crate::color::Color;
use crate::math::{finite_or, vec2, Rect, Vec2};
use std::f32::consts::TAU;

/// Corner radii of a rounded rectangle, clockwise from top-left.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Radii {
    /// Top-left.
    pub tl: f32,
    /// Top-right.
    pub tr: f32,
    /// Bottom-right.
    pub br: f32,
    /// Bottom-left.
    pub bl: f32,
}

impl Radii {
    /// The same radius on every corner.
    #[must_use]
    pub const fn all(r: f32) -> Self {
        Self {
            tl: r,
            tr: r,
            br: r,
            bl: r,
        }
    }

    /// Per-corner radii, clockwise from top-left.
    #[must_use]
    pub const fn new(tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        Self { tl, tr, br, bl }
    }

    fn to_array(self) -> [f32; 4] {
        [self.tl, self.tr, self.br, self.bl].map(|r| finite_or(r, 0.0).max(0.0))
    }
}

impl From<f32> for Radii {
    fn from(r: f32) -> Self {
        Self::all(r)
    }
}

/// End caps of open strokes and arcs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cap {
    /// Flat, ending exactly at the end point / angle.
    #[default]
    Butt,
    /// Semicircular.
    Round,
}

/// Bar orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Axis {
    /// Fills left to right.
    #[default]
    Horizontal,
    /// Fills bottom to top.
    Vertical,
}

/// A ring of evenly spaced radial ticks (dial chapter ring).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickRing {
    /// Dial centre.
    pub center: Vec2,
    /// Outer end of the ticks.
    pub radius: f32,
    /// Start angle of the first tick (radians, clockwise from 3 o'clock).
    pub start: f32,
    /// Angle from the first tick to the last.
    pub sweep: f32,
    /// Number of ticks (at least 2 for a sweep, 1 draws a single tick).
    pub count: u32,
    /// Minor tick length (inwards from `radius`).
    pub length: f32,
    /// Minor tick width.
    pub width: f32,
    /// Every `major_every`-th tick (from the first) is a major tick; 0 = none.
    pub major_every: u32,
    /// Major tick length.
    pub major_length: f32,
    /// Major tick width.
    pub major_width: f32,
}

impl TickRing {
    /// Ticks from `start` over `sweep` with token default sizes (minor 2 x 14, major 5 x 34).
    #[must_use]
    pub fn new(center: Vec2, radius: f32, start: f32, sweep: f32, count: u32) -> Self {
        Self {
            center,
            radius,
            start,
            sweep,
            count,
            length: 14.0,
            width: 2.0,
            major_every: 0,
            major_length: 34.0,
            major_width: 5.0,
        }
    }

    /// Marks every `n`-th tick as major, with its length and width.
    #[must_use]
    pub fn major(mut self, every: u32, length: f32, width: f32) -> Self {
        self.major_every = every;
        self.major_length = length;
        self.major_width = width;
        self
    }

    /// Sets the minor tick length and width.
    #[must_use]
    pub fn minor(mut self, length: f32, width: f32) -> Self {
        self.length = length;
        self.width = width;
        self
    }
}

/// A row of segments (LED strip, shift lights, ribbon bar) lit up to a fractional count.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentedBar {
    /// The whole bar.
    pub rect: Rect,
    /// Number of segments.
    pub count: u32,
    /// Gap between segments.
    pub gap: f32,
    /// Lit amount in segments (`3.5` lights three and half of the fourth).
    pub lit: f32,
    /// Corner radius of each segment.
    pub radius: f32,
    /// Colour of unlit segments (tokens: neutral white at alpha 22 or less).
    pub unlit: Color,
    /// Fill direction.
    pub axis: Axis,
}

/// A superellipse dome `|x/a|^n + |y/b|^n = 1` (separate exponents for the upper and lower
/// halves) whose filled coverage falls off smoothly across a direction-dependent feather, with
/// an optional round-topped vertical void cut out below `void_top`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dome {
    /// Centre.
    pub center: Vec2,
    /// Semi-axes (half width, half height) of the opaque core.
    pub semi_axes: Vec2,
    /// Exponent of the upper half (2 = ellipse, 4 = squircle).
    pub exponent_top: f32,
    /// Exponent of the lower half.
    pub exponent_bottom: f32,
    /// Feather above the dome.
    pub feather_top: f32,
    /// Feather at the sides.
    pub feather_side: f32,
    /// Feather below.
    pub feather_bottom: f32,
    /// Half width of the void (0 = none), centred on `center.x`.
    pub void_half_width: f32,
    /// Top of the void (y, local units).
    pub void_top: f32,
    /// Softness of the void edge.
    pub void_feather: f32,
}

impl Dome {
    /// The bounds including the widest feather.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        let f = self
            .feather_top
            .max(self.feather_side)
            .max(self.feather_bottom)
            .max(0.0);
        Rect::from_center_size(self.center, self.semi_axes * 2.0).expand(f)
    }
}

/// The outline of one [`Piece`] of a [`Shape::Blob`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outline {
    /// A disc.
    Circle {
        /// Centre.
        center: Vec2,
        /// Radius.
        radius: f32,
    },
    /// A rounded rectangle (a pill when `radius` is half its shorter side).
    RoundRect {
        /// Bounds.
        rect: Rect,
        /// Corner radius.
        radius: f32,
    },
    /// A round-ended segment whose radius may change along it (a capsule, a teardrop).
    Capsule {
        /// Start.
        a: Vec2,
        /// End.
        b: Vec2,
        /// Radius at `a`.
        radius_a: f32,
        /// Radius at `b`.
        radius_b: f32,
    },
    /// A round-ended band along a circular arc (a curved rod).
    Arc {
        /// Centre of the arc.
        center: Vec2,
        /// Radius of the band's centre line.
        radius: f32,
        /// Half the band's width.
        half_width: f32,
        /// Start angle (radians, clockwise from 3 o'clock).
        start: f32,
        /// Sweep (radians; negative sweeps run anticlockwise).
        sweep: f32,
    },
}

impl Outline {
    /// Signed distance from `p` to the outline (negative inside). Mirrors `shape.wgsl`.
    #[must_use]
    pub fn distance(&self, p: Vec2) -> f32 {
        match *self {
            Outline::Circle { center, radius } => (p - center).length() - radius,
            Outline::RoundRect { rect, radius } => {
                let half = rect.size() * 0.5;
                let r = radius.clamp(0.0, half.x.min(half.y).max(0.0));
                let q = p - rect.center();
                let q = vec2(q.x.abs() - half.x + r, q.y.abs() - half.y + r);
                q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0) - r
            }
            Outline::Capsule {
                a,
                b,
                radius_a,
                radius_b,
            } => sd_uneven_capsule(p, a, b, radius_a, radius_b),
            Outline::Arc {
                center,
                radius,
                half_width,
                start,
                sweep,
            } => {
                let (start, sweep) = if sweep < 0.0 {
                    (start + sweep, -sweep)
                } else {
                    (start, sweep)
                };
                let q = p - center;
                let ring = (q.length() - radius).abs() - half_width;
                if sweep >= TAU - 1e-4 {
                    return ring;
                }
                let rel = (q.y.atan2(q.x) - start).rem_euclid(TAU);
                if rel <= sweep {
                    return ring;
                }
                let e0 = Vec2::polar(center, radius, start);
                let e1 = Vec2::polar(center, radius, start + sweep);
                (p - e0).length().min((p - e1).length()) - half_width
            }
        }
    }

    /// The largest distance from the outline to its inside (the radius of the biggest disc
    /// that fits). A bevel this wide curves the whole top surface.
    #[must_use]
    pub fn inradius(&self) -> f32 {
        match *self {
            Outline::Circle { radius, .. } => radius,
            Outline::RoundRect { rect, .. } => rect.width().min(rect.height()) * 0.5,
            Outline::Capsule { radius_a, radius_b, .. } => radius_a.max(radius_b),
            Outline::Arc { half_width, .. } => half_width,
        }
    }

    /// Local-space bounds.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        match *self {
            Outline::Circle { center, radius } => Rect::from_center_size(center, Vec2::splat(radius * 2.0)),
            Outline::RoundRect { rect, .. } => rect,
            Outline::Capsule {
                a,
                b,
                radius_a,
                radius_b,
            } => Rect::from_points(a, b).expand(radius_a.max(radius_b)),
            Outline::Arc {
                center,
                radius,
                half_width,
                ..
            } => Rect::from_center_size(center, Vec2::splat((radius + half_width) * 2.0)),
        }
    }
}

/// Round-capped tapered segment (mirrors `sd_uneven_capsule` in `shape.wgsl`).
fn sd_uneven_capsule(p: Vec2, pa: Vec2, pb: Vec2, ra: f32, rb: f32) -> f32 {
    let ba = pb - pa;
    let l2 = ba.dot(ba).max(1e-12);
    let rr = ra - rb;
    let a2 = l2 - rr * rr;
    let il2 = 1.0 / l2;
    let pp = p - pa;
    let y = pp.dot(ba);
    let z = y - l2;
    let xv = pp * l2 - ba * y;
    let x2 = xv.dot(xv);
    let y2 = y * y * l2;
    let z2 = z * z * l2;
    let sign = |v: f32| {
        if v > 0.0 {
            1.0
        } else if v < 0.0 {
            -1.0
        } else {
            0.0
        }
    };
    let k = sign(rr) * rr * rr * x2;
    if sign(z) * a2 * z2 > k {
        return (x2 + z2).sqrt() * il2 - rb;
    }
    if sign(y) * a2 * y2 < k {
        return (x2 + y2).sqrt() * il2 - ra;
    }
    ((x2 * a2 * il2).sqrt() + y * rr) * il2 - ra
}

/// One piece of a liquid group: an outline and its glass cross-section
/// (`docs/specs/optics.md` §2). The cross-section only matters to the optical glass paint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    /// The outline.
    pub outline: Outline,
    /// Rise of the top surface above the rim.
    pub thickness: f32,
    /// Width of the rounded edge, measured inwards from the outline.
    pub bevel: f32,
    /// Glass below the rim level, down to the plane the piece sits on.
    pub base: f32,
}

impl Piece {
    /// A piece with a flat cross-section (give it one with [`Piece::section`] or
    /// [`Piece::dome`]).
    #[must_use]
    pub fn new(outline: Outline) -> Self {
        Self {
            outline,
            thickness: 0.0,
            bevel: 1.0,
            base: 0.0,
        }
    }

    /// A disc.
    #[must_use]
    pub fn circle(center: Vec2, radius: f32) -> Self {
        Self::new(Outline::Circle { center, radius })
    }

    /// A rounded rectangle.
    #[must_use]
    pub fn round_rect(rect: Rect, radius: f32) -> Self {
        Self::new(Outline::RoundRect { rect, radius })
    }

    /// A pill: a rounded rectangle whose ends are half circles.
    #[must_use]
    pub fn pill(rect: Rect) -> Self {
        Self::round_rect(rect, rect.width().min(rect.height()) * 0.5)
    }

    /// A round-ended segment of constant radius.
    #[must_use]
    pub fn capsule(a: Vec2, b: Vec2, radius: f32) -> Self {
        Self::new(Outline::Capsule {
            a,
            b,
            radius_a: radius,
            radius_b: radius,
        })
    }

    /// A round-ended band of full `width` along an arc.
    #[must_use]
    pub fn arc(center: Vec2, radius: f32, width: f32, start: f32, sweep: f32) -> Self {
        Self::new(Outline::Arc {
            center,
            radius,
            half_width: width * 0.5,
            start,
            sweep,
        })
    }

    /// Sets the cross-section: a top surface `thickness` above the rim, reached over a rounded
    /// edge `bevel` wide, with `base` more glass below the rim.
    #[must_use]
    pub fn section(mut self, thickness: f32, bevel: f32, base: f32) -> Self {
        self.thickness = thickness;
        self.bevel = bevel;
        self.base = base;
        self
    }

    /// Makes the piece a dome: the whole top surface curves, rising `height` at the middle
    /// (the bevel is the outline's [`Outline::inradius`]). A disc with `height` equal to its
    /// radius is a hemisphere; a band is a half rod.
    #[must_use]
    pub fn dome(mut self, height: f32, base: f32) -> Self {
        self.thickness = height;
        self.bevel = self.outline.inradius().max(1e-3);
        self.base = base;
        self
    }
}

/// The smooth union of two signed distances with blend length `k`:
/// `min(a, b) − k ln(1 + e^(−|a − b| / k))` (`docs/specs/optics.md` §2). Associative,
/// commutative and smooth to every order; `k <= 0` is the plain union.
#[must_use]
pub fn smooth_union(a: f32, b: f32, k: f32) -> f32 {
    if k > 1e-3 {
        a.min(b) - k * (-(a - b).abs() / k).exp().ln_1p()
    } else {
        a.min(b)
    }
}

/// Signed distance from `p` to the smooth union of `pieces` (what [`Shape::Blob`] draws).
#[must_use]
pub fn blob_distance(pieces: &[Piece], smooth: f32, p: Vec2) -> f32 {
    pieces
        .iter()
        .map(|piece| piece.outline.distance(p))
        .reduce(|acc, d| smooth_union(acc, d, smooth))
        .unwrap_or(f32::MAX)
}

/// Most pieces one [`Shape::Blob`] can hold; more are ignored.
pub const MAX_BLOB_PIECES: usize = 64;

/// A shape to fill, stroke, glow or shadow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape<'a> {
    /// A rectangle with per-corner radii.
    Rect {
        /// Bounds.
        rect: Rect,
        /// Corner radii.
        radii: Radii,
    },
    /// A circle.
    Circle {
        /// Centre.
        center: Vec2,
        /// Radius.
        radius: f32,
    },
    /// A ring segment (arc) of the given thickness, centred on `radius`. A full ring has
    /// `sweep = TAU`.
    Arc {
        /// Centre.
        center: Vec2,
        /// Radius of the arc's centre line.
        radius: f32,
        /// Full thickness.
        thickness: f32,
        /// Start angle (radians, clockwise from 3 o'clock).
        start: f32,
        /// Sweep (radians, clockwise; negative sweeps run anticlockwise).
        sweep: f32,
        /// End caps.
        cap: Cap,
    },
    /// A tapered line from `a` (width `width_a`) to `b` (width `width_b`): needles, capsules.
    Line {
        /// Start.
        a: Vec2,
        /// End.
        b: Vec2,
        /// Full width at `a`.
        width_a: f32,
        /// Full width at `b`.
        width_b: f32,
        /// End caps.
        cap: Cap,
    },
    /// An open or closed polyline stroke of constant width (round joins).
    Polyline {
        /// Points (at least 2).
        points: &'a [Vec2],
        /// Full stroke width.
        width: f32,
        /// Joins the last point back to the first.
        closed: bool,
    },
    /// A filled polygon (even-odd rule).
    Polygon {
        /// Vertices (at least 3).
        points: &'a [Vec2],
    },
    /// A ring of ticks.
    Ticks(TickRing),
    /// A chevron (`>`) centred at `center`, pointing along `angle`.
    Chevron {
        /// Centre of the chevron's bounding box.
        center: Vec2,
        /// Full width (along the pointing direction) and height.
        size: Vec2,
        /// Stroke thickness of the two arms.
        thickness: f32,
        /// Pointing direction (radians, clockwise from 3 o'clock).
        angle: f32,
    },
    /// A segmented bar.
    Bar(SegmentedBar),
    /// A feathered superellipse dome with an optional vertical void (the OEM dash scrim).
    /// Filled, it is opaque in the core and falls off across the feather (dithered); stroked,
    /// glowed or shadowed, it uses the signed distance to the core edge.
    Dome(Dome),
    /// A seven-segment digit in `rect`. Mask bits: a=1 b=2 c=4 d=8 e=16 f=32 g=64.
    SevenSeg {
        /// Cell.
        rect: Rect,
        /// Segment thickness.
        thickness: f32,
        /// Lit segments.
        mask: u8,
    },
    /// Pieces merged like liquid: the smooth union of their outlines (see [`smooth_union`]).
    /// Pieces further apart than a few `smooth` stay separate; closer, they bridge and fuse.
    Blob {
        /// The pieces (at most [`MAX_BLOB_PIECES`]).
        pieces: &'a [Piece],
        /// Blend length in local units (0 = plain union).
        smooth: f32,
    },
}

impl<'a> Shape<'a> {
    /// A sharp-cornered rectangle.
    #[must_use]
    pub fn rect(rect: Rect) -> Self {
        Shape::Rect {
            rect,
            radii: Radii::default(),
        }
    }

    /// A rounded rectangle.
    #[must_use]
    pub fn round_rect(rect: Rect, radii: impl Into<Radii>) -> Self {
        Shape::Rect {
            rect,
            radii: radii.into(),
        }
    }

    /// A circle.
    #[must_use]
    pub fn circle(center: Vec2, radius: f32) -> Self {
        Shape::Circle { center, radius }
    }

    /// A full ring.
    #[must_use]
    pub fn ring(center: Vec2, radius: f32, thickness: f32) -> Self {
        Shape::Arc {
            center,
            radius,
            thickness,
            start: 0.0,
            sweep: TAU,
            cap: Cap::Butt,
        }
    }

    /// An arc.
    #[must_use]
    pub fn arc(center: Vec2, radius: f32, thickness: f32, start: f32, sweep: f32, cap: Cap) -> Self {
        Shape::Arc {
            center,
            radius,
            thickness,
            start,
            sweep,
            cap,
        }
    }

    /// A round-capped line of constant width.
    #[must_use]
    pub fn line(a: Vec2, b: Vec2, width: f32) -> Self {
        Shape::Line {
            a,
            b,
            width_a: width,
            width_b: width,
            cap: Cap::Round,
        }
    }

    /// A tapered needle from `pivot` along `angle`: `tail` behind the pivot, `length` in front,
    /// `base_width` at the pivot end and `tip_width` at the tip (tokens: 14 to 3 px).
    #[must_use]
    pub fn needle(pivot: Vec2, angle: f32, length: f32, tail: f32, base_width: f32, tip_width: f32) -> Self {
        let dir = Vec2::from_angle(angle);
        Shape::Line {
            a: pivot - dir * tail,
            b: pivot + dir * length,
            width_a: base_width,
            width_b: tip_width,
            cap: Cap::Round,
        }
    }

    /// Local-space bounds of the shape's geometry (before strokes and effects).
    #[must_use]
    pub fn bounds(&self) -> Rect {
        match *self {
            Shape::Rect { rect, .. } => rect,
            Shape::Circle { center, radius } => Rect::from_center_size(center, Vec2::splat(radius * 2.0)),
            Shape::Arc {
                center,
                radius,
                thickness,
                ..
            } => Rect::from_center_size(center, Vec2::splat((radius + thickness * 0.5) * 2.0)),
            Shape::Line {
                a, b, width_a, width_b, ..
            } => Rect::from_points(a, b).expand(width_a.max(width_b) * 0.5),
            Shape::Polyline { points, width, .. } => points_bounds(points).expand(width * 0.5),
            Shape::Polygon { points } => points_bounds(points),
            Shape::Ticks(t) => {
                Rect::from_center_size(t.center, Vec2::splat(t.radius * 2.0 + t.major_width.max(t.width)))
            }
            Shape::Chevron {
                center,
                size,
                thickness,
                ..
            } => Rect::from_center_size(center, Vec2::splat(size.x.max(size.y) + thickness)),
            Shape::Bar(b) => b.rect,
            Shape::Dome(d) => d.bounds(),
            Shape::SevenSeg { rect, .. } => rect,
            Shape::Blob { pieces, smooth } => {
                let n = pieces.len().min(MAX_BLOB_PIECES);
                // A smooth union grows the plain union by at most k ln N.
                let grow = finite_or(smooth, 0.0).max(0.0) * (n.max(1) as f32).ln();
                pieces[..n]
                    .iter()
                    .fold(Rect::EMPTY, |r, piece| r.union(&piece.outline.bounds()))
                    .expand(grow)
            }
        }
    }
}

fn points_bounds(points: &[Vec2]) -> Rect {
    points
        .iter()
        .fold(Rect::EMPTY, |r, p| r.union(&Rect::from_points(*p, *p)))
}

/// How a shape's distance becomes coverage.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum StrokeAlign {
    /// Centred on the edge.
    #[default]
    Center,
    /// Entirely inside the shape (rims that never grow the silhouette).
    Inner,
    /// Entirely outside the shape.
    Outer,
}

/// Stroke width units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrokeUnit {
    /// Local units: scales with the cluster.
    #[default]
    Local,
    /// Device pixels: exactly this many screen pixels at any scale (1 px specular rims).
    DevicePx,
}

/// A stroke style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    /// Width.
    pub width: f32,
    /// Placement relative to the edge.
    pub align: StrokeAlign,
    /// Units of `width`.
    pub unit: StrokeUnit,
}

impl Stroke {
    /// A centred stroke in local units.
    #[must_use]
    pub fn new(width: f32) -> Self {
        Self {
            width,
            align: StrokeAlign::Center,
            unit: StrokeUnit::Local,
        }
    }

    /// An inner stroke in local units.
    #[must_use]
    pub fn inner(width: f32) -> Self {
        Self {
            align: StrokeAlign::Inner,
            ..Self::new(width)
        }
    }

    /// An outer stroke in local units.
    #[must_use]
    pub fn outer(width: f32) -> Self {
        Self {
            align: StrokeAlign::Outer,
            ..Self::new(width)
        }
    }

    /// An inner rim exactly `px` device pixels wide (tokens: specular rims are 1 px).
    #[must_use]
    pub fn rim_px(px: f32) -> Self {
        Self {
            width: px,
            align: StrokeAlign::Inner,
            unit: StrokeUnit::DevicePx,
        }
    }
}

/// An outer glow (soft light around the silhouette).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glow {
    /// Gaussian sigma in local units (tokens `glow` levels).
    pub sigma: f32,
    /// Peak intensity multiplier.
    pub intensity: f32,
    /// Also light the interior (a filled glow) instead of only the outside.
    pub fill_inside: bool,
}

impl Glow {
    /// An outside-only glow.
    #[must_use]
    pub fn new(sigma: f32, intensity: f32) -> Self {
        Self {
            sigma,
            intensity,
            fill_inside: false,
        }
    }
}

/// A drop shadow (or inner light pad when the colour is light).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    /// Offset in local units.
    pub offset: Vec2,
    /// Gaussian blur sigma in local units.
    pub sigma: f32,
    /// Grows (positive) or shrinks the silhouette before blurring.
    pub spread: f32,
    /// Shadow colour.
    pub color: Color,
}

impl Shadow {
    /// A shadow below-right by `offset`.
    #[must_use]
    pub fn new(offset: Vec2, sigma: f32, color: Color) -> Self {
        Self {
            offset,
            sigma,
            spread: 0.0,
            color,
        }
    }
}

/// Per-instance shape kinds (must match `shape.wgsl`).
pub(crate) mod kind {
    pub const RECT: u32 = 0;
    pub const CIRCLE: u32 = 1;
    pub const ARC: u32 = 2;
    pub const LINE: u32 = 3;
    pub const POLYLINE: u32 = 4;
    pub const POLYGON: u32 = 5;
    pub const TICKS: u32 = 6;
    pub const CHEVRON: u32 = 7;
    pub const BAR: u32 = 8;
    pub const GLYPH: u32 = 9;
    pub const SEVENSEG: u32 = 10;
    pub const NINESLICE: u32 = 11;
    pub const DOME: u32 = 12;
    pub const BLOB: u32 = 13;
}

/// Records per [`Piece`] in the point list: outline kind and radius, three outline vectors,
/// then the cross-section (must match `sd_piece` and `blob_field` in `shape.wgsl`).
pub(crate) const PIECE_RECORDS: usize = 6;

/// Encodes one piece into its point-list records.
pub(crate) fn encode_piece(piece: &Piece) -> [[f32; 2]; PIECE_RECORDS] {
    let f = |v: f32| finite_or(v, 0.0);
    let pos = |v: f32| finite_or(v, 0.0).max(0.0);
    let [head, v1, v2, v3] = match piece.outline {
        Outline::Circle { center, radius } => [[0.0, pos(radius)], [f(center.x), f(center.y)], [0.0; 2], [0.0; 2]],
        Outline::RoundRect { rect, radius } => {
            let c = rect.center();
            let h = rect.size() * 0.5;
            [
                [1.0, pos(radius).min(pos(h.x).min(pos(h.y)))],
                [f(c.x), f(c.y)],
                [pos(h.x), pos(h.y)],
                [0.0; 2],
            ]
        }
        Outline::Capsule {
            a,
            b,
            radius_a,
            radius_b,
        } => [
            [2.0, 0.0],
            [f(a.x), f(a.y)],
            [f(b.x), f(b.y)],
            [pos(radius_a), pos(radius_b)],
        ],
        Outline::Arc {
            center,
            radius,
            half_width,
            start,
            sweep,
        } => {
            let (start, sweep) = if sweep < 0.0 {
                (start + sweep, -sweep)
            } else {
                (start, sweep)
            };
            [
                [3.0, 0.0],
                [f(center.x), f(center.y)],
                [pos(radius), pos(half_width)],
                [f(start), f(sweep).min(TAU)],
            ]
        }
    };
    [
        head,
        v1,
        v2,
        v3,
        [pos(piece.thickness), pos(piece.bevel).max(1e-3)],
        [pos(piece.base), 0.0],
    ]
}

/// Coverage modes (must match `shape.wgsl`).
pub(crate) mod mode {
    pub const FILL: u32 = 0;
    pub const STROKE: u32 = 1;
    pub const GLOW: u32 = 2;
    pub const SHADOW: u32 = 3;
}

/// Flag bits (must match `shape.wgsl`).
pub(crate) mod flag {
    /// Stroke width is in device pixels.
    pub const STROKE_PX: u32 = 1;
    /// Glow also covers the interior.
    pub const GLOW_FILL: u32 = 2;
    /// Bar is vertical.
    pub const VERTICAL: u32 = 4;
    /// Closed polyline.
    pub const CLOSED: u32 = 8;
    /// Round caps.
    pub const ROUND_CAP: u32 = 16;
}

/// Shape geometry encoded for the GPU: kind, three parameter vectors, flags.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Encoded {
    pub kind: u32,
    pub flags: u32,
    pub p0: [f32; 4],
    pub p1: [f32; 4],
    pub p2: [f32; 4],
}

/// Encodes everything except point-list shapes, which the painter encodes with their
/// storage-buffer offsets.
pub(crate) fn encode_simple(shape: &Shape<'_>) -> Option<Encoded> {
    let f = |v: f32| finite_or(v, 0.0);
    let e = match *shape {
        Shape::Rect { rect, radii } => {
            let c = rect.center();
            let h = rect.size() * 0.5;
            let mut r = radii.to_array();
            let max_r = h.x.min(h.y).max(0.0);
            for v in &mut r {
                *v = v.min(max_r);
            }
            Encoded {
                kind: kind::RECT,
                p0: [f(c.x), f(c.y), f(h.x).max(0.0), f(h.y).max(0.0)],
                p1: r,
                ..Default::default()
            }
        }
        Shape::Circle { center, radius } => Encoded {
            kind: kind::CIRCLE,
            p0: [f(center.x), f(center.y), f(radius).max(0.0), 0.0],
            ..Default::default()
        },
        Shape::Arc {
            center,
            radius,
            thickness,
            start,
            sweep,
            cap,
        } => {
            // Negative sweeps are normalised to a positive sweep from the other end.
            let (start, sweep) = if sweep < 0.0 {
                (start + sweep, -sweep)
            } else {
                (start, sweep)
            };
            Encoded {
                kind: kind::ARC,
                flags: if cap == Cap::Round { flag::ROUND_CAP } else { 0 },
                p0: [
                    f(center.x),
                    f(center.y),
                    f(radius).max(0.0),
                    f(thickness).max(0.0) * 0.5,
                ],
                p1: [f(start), f(sweep).min(TAU), 0.0, 0.0],
                ..Default::default()
            }
        }
        Shape::Line {
            a,
            b,
            width_a,
            width_b,
            cap,
        } => Encoded {
            kind: kind::LINE,
            flags: if cap == Cap::Round { flag::ROUND_CAP } else { 0 },
            p0: [f(a.x), f(a.y), f(b.x), f(b.y)],
            p1: [f(width_a).max(0.0) * 0.5, f(width_b).max(0.0) * 0.5, 0.0, 0.0],
            ..Default::default()
        },
        Shape::Ticks(t) => {
            let count = t.count.max(1);
            let step = if count > 1 { t.sweep / (count - 1) as f32 } else { 0.0 };
            Encoded {
                kind: kind::TICKS,
                p0: [f(t.center.x), f(t.center.y), f(t.radius), f(t.length).max(0.0)],
                p1: [f(t.start), f(step), count as f32, f(t.width).max(0.0) * 0.5],
                p2: [
                    t.major_every as f32,
                    f(t.major_length).max(0.0),
                    f(t.major_width).max(0.0) * 0.5,
                    0.0,
                ],
                ..Default::default()
            }
        }
        Shape::Chevron {
            center,
            size,
            thickness,
            angle,
        } => Encoded {
            kind: kind::CHEVRON,
            p0: [f(center.x), f(center.y), f(size.x) * 0.5, f(size.y) * 0.5],
            p1: [f(thickness).max(0.0) * 0.5, f(angle), 0.0, 0.0],
            ..Default::default()
        },
        Shape::Bar(b) => Encoded {
            kind: kind::BAR,
            flags: if b.axis == Axis::Vertical { flag::VERTICAL } else { 0 },
            p0: [f(b.rect.min.x), f(b.rect.min.y), f(b.rect.width()), f(b.rect.height())],
            p1: [b.count.max(1) as f32, f(b.gap).max(0.0), f(b.lit), f(b.radius).max(0.0)],
            p2: b.unlit.to_array(),
        },
        Shape::SevenSeg { rect, thickness, mask } => Encoded {
            kind: kind::SEVENSEG,
            p0: [f(rect.min.x), f(rect.min.y), f(rect.width()), f(rect.height())],
            p1: [f(thickness).max(0.5), f32::from(mask & 0x7F), 0.0, 0.0],
            ..Default::default()
        },
        Shape::Dome(d) => Encoded {
            kind: kind::DOME,
            p0: [
                f(d.center.x),
                f(d.center.y),
                f(d.semi_axes.x).max(1.0),
                f(d.semi_axes.y).max(1.0),
            ],
            p1: [
                f(d.exponent_top).clamp(1.5, 8.0),
                f(d.exponent_bottom).clamp(1.5, 8.0),
                f(d.feather_top).max(0.0),
                f(d.feather_side).max(0.0),
            ],
            p2: [
                f(d.feather_bottom).max(0.0),
                f(d.void_half_width).max(0.0),
                f(d.void_top),
                f(d.void_feather).max(0.0),
            ],
            ..Default::default()
        },
        Shape::Polyline { .. } | Shape::Polygon { .. } | Shape::Blob { .. } => return None,
    };
    Some(e)
}

/// Seven-segment masks for `0..=9`, then `-`, blank. Bits: a=1 b=2 c=4 d=8 e=16 f=32 g=64.
pub const SEVEN_SEG_DIGITS: [u8; 10] = [0x3F, 0x06, 0x5B, 0x4F, 0x66, 0x6D, 0x7D, 0x07, 0x7F, 0x6F];

/// The seven-segment mask for a character (`0-9`, `-`, `r`, `n`, `N`, `P`, `E`, space).
#[must_use]
pub fn seven_seg_mask(ch: char) -> u8 {
    match ch {
        '0'..='9' => SEVEN_SEG_DIGITS[(ch as u8 - b'0') as usize],
        '-' => 0x40,
        'r' | 'R' => 0x50,
        'n' => 0x54,
        'N' => 0x37,
        'P' => 0x73,
        'E' => 0x79,
        _ => 0,
    }
}

/// A sample point on a circle at a gauge fraction: angle `start + sweep * t`.
#[must_use]
pub fn gauge_point(center: Vec2, radius: f32, start: f32, sweep: f32, t: f32) -> Vec2 {
    Vec2::polar(center, radius, start + sweep * t)
}

/// Returns `(x, y)` as a [`Vec2`]; convenience for layout tables.
#[must_use]
pub fn pt(x: f32, y: f32) -> Vec2 {
    vec2(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_radii_are_clamped_to_half_size() {
        let e = encode_simple(&Shape::round_rect(Rect::from_xywh(0.0, 0.0, 10.0, 4.0), 100.0)).expect("simple");
        assert!(e.p1.iter().all(|r| *r <= 2.0));
    }

    #[test]
    fn negative_sweep_is_normalised() {
        let e = encode_simple(&Shape::arc(Vec2::ZERO, 10.0, 2.0, 1.0, -0.5, Cap::Butt)).expect("simple");
        assert!((e.p1[0] - 0.5).abs() < 1e-6 && (e.p1[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn seven_seg_masks() {
        assert_eq!(seven_seg_mask('8'), 0x7F);
        assert_eq!(seven_seg_mask('1'), 0x06);
        assert_eq!(seven_seg_mask(' '), 0);
    }

    #[test]
    fn bounds_cover_geometry() {
        let b = Shape::needle(Vec2::ZERO, 0.0, 100.0, 10.0, 14.0, 3.0).bounds();
        assert!(b.contains(vec2(100.0, 0.0)) && b.contains(vec2(-10.0, 0.0)));
    }

    #[test]
    fn smooth_union_properties() {
        let k = 12.0;
        for (a, b, c) in [(-30.0_f32, 5.0_f32, 40.0_f32), (3.0, 3.0, 3.0), (-8.0, -9.0, 20.0)] {
            let ab_c = smooth_union(smooth_union(a, b, k), c, k);
            let a_bc = smooth_union(a, smooth_union(b, c, k), k);
            let c_ba = smooth_union(c, smooth_union(b, a, k), k);
            assert!(
                (ab_c - a_bc).abs() < 1e-3 && (ab_c - c_ba).abs() < 1e-3,
                "order independent: {ab_c} {a_bc} {c_ba}"
            );
            let plain = a.min(b).min(c);
            assert!(ab_c <= plain + 1e-5, "never outside the plain union");
            assert!(plain - ab_c <= k * 3.0_f32.ln() + 1e-3, "at most k ln N inside it");
        }
        assert!(
            (smooth_union(-5.0, 400.0, k) + 5.0).abs() < 1e-4,
            "a far piece changes nothing"
        );
        assert_eq!(smooth_union(2.0, 7.0, 0.0), 2.0, "k = 0 is the plain union");
        // Two equal distances fuse by k ln 2.
        assert!((smooth_union(0.0, 0.0, k) + k * 2.0_f32.ln()).abs() < 1e-4);
    }

    #[test]
    fn outlines_measure_distance() {
        let c = Outline::Circle {
            center: vec2(10.0, 0.0),
            radius: 5.0,
        };
        assert!((c.distance(vec2(10.0, 0.0)) + 5.0).abs() < 1e-5 && (c.distance(vec2(20.0, 0.0)) - 5.0).abs() < 1e-5);
        let pill = Piece::pill(Rect::from_xywh(-20.0, -5.0, 40.0, 10.0));
        assert!(
            (pill.outline.distance(vec2(0.0, 0.0)) + 5.0).abs() < 1e-5,
            "pill centre is its inradius deep"
        );
        assert!(
            (pill.outline.distance(vec2(20.0, 0.0))).abs() < 1e-5,
            "the end of the pill is on the outline"
        );
        assert!((pill.outline.inradius() - 5.0).abs() < 1e-6);
        let cap = Piece::capsule(vec2(0.0, 0.0), vec2(30.0, 0.0), 4.0);
        assert!((cap.outline.distance(vec2(15.0, 4.0))).abs() < 1e-4, "capsule side");
        assert!((cap.outline.distance(vec2(34.0, 0.0))).abs() < 1e-4, "capsule end");
        let arc = Piece::arc(Vec2::ZERO, 100.0, 12.0, 0.0, core::f32::consts::FRAC_PI_2);
        assert!(
            (arc.outline.distance(vec2(100.0, 0.0)) + 6.0).abs() < 1e-4,
            "arc centre line"
        );
        assert!(
            (arc.outline.distance(vec2(-100.0, 0.0)) - 135.4).abs() < 0.1,
            "far side of the ring is outside"
        );
        // A negative sweep is the same band from the other end.
        let back = Piece::arc(
            Vec2::ZERO,
            100.0,
            12.0,
            core::f32::consts::FRAC_PI_2,
            -core::f32::consts::FRAC_PI_2,
        );
        let p = vec2(70.0, 75.0);
        assert!((arc.outline.distance(p) - back.outline.distance(p)).abs() < 1e-4);
    }

    #[test]
    fn a_blob_bridges_close_pieces_and_bounds_them() {
        let pieces = [
            Piece::circle(vec2(-30.0, 0.0), 20.0),
            Piece::circle(vec2(30.0, 0.0), 20.0),
        ];
        // Midway between the discs: 10 outside each. The smooth union pulls it inwards.
        let apart = blob_distance(&pieces, 0.0, Vec2::ZERO);
        let fused = blob_distance(&pieces, 16.0, Vec2::ZERO);
        assert!(
            (apart - 10.0).abs() < 1e-5 && fused < 0.0,
            "apart {apart}, fused {fused}"
        );
        let shape = Shape::Blob {
            pieces: &pieces,
            smooth: 16.0,
        };
        let b = shape.bounds();
        assert!(b.contains(vec2(-50.0, 0.0)) && b.contains(vec2(50.0, 20.0)));
        let enc = encode_piece(&pieces[0].dome(20.0, 4.0));
        assert_eq!(enc[0], [0.0, 20.0]);
        assert_eq!(enc[4], [20.0, 20.0], "a dome's bevel is its inradius");
        assert_eq!(enc[5], [4.0, 0.0]);
    }
}
