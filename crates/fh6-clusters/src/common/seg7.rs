//! Seg7Digit: slanted VFD seven-segment digits drawn as SDF segments (COMPONENTS.md §15,
//! `docs/design/families/sci-fi.md`).
//!
//! Every segment is a chamfered (pointed hexagon) polygon evaluated by the kit as an exact
//! signed distance, so it stays crisp at any scale and glows from the same shape. The geometry
//! follows the spec: an 8° slant, a stroke of 10.5% of the digit height and gaps of 18% of the
//! stroke between segment tips. Unlit segments are always drawn at 7% of the phosphor (the ghost
//! `8`); lit segments are the phosphor fill with the tokens' `glow.phosphor` lobes in a bloom
//! layer and a hot core mixed 45% toward the white ink ([`Emission`] picks the softer
//! `glow.soft` lobe for a hero gear on the limiter, or no glow for ranks 4–5).
//!
//! Ghost segments are opaque: each channel shows 7% of the phosphor's displayed brightness on any
//! black, whether the digits sit in an opaque VFD cavity (`materials.vfdCavity`, the family's
//! normal case) or straight on the transparent overlay. Like a real VFD, an unlit segment is a
//! physical part of the glass, so it also hides whatever is behind it.
//!
//! - [`Seg7Geometry`]: cell size, stroke, gap, slant and the segment polygons.
//! - [`Seg7Digit`]: one animated cell (instant for live numbers, an 80 ms sequence for a gear),
//!   on a seven-segment glass or the gear bay's glass with the `N` diagonal ([`Segments`],
//!   [`gear_mask`]).
//! - [`Seg7Number`]: `N` fixed cells, right-aligned, ghost 8s in the leading cells, an optional
//!   decimal point or colon; digits come from arithmetic, nothing allocates.
//! - [`draw_speed`]: the `segment` variant of TabularSpeed (COMPONENTS.md §2), 3 cells.
//! - [`draw_scanlines`]: the VFD cavity's 2 px scanlines, clipped to a rectangle.
use fh6_kit::color::{linear_to_srgb, srgb_to_linear};
use fh6_kit::shape::{seven_seg_mask, SEVEN_SEG_DIGITS};
use fh6_kit::tokens::GlowLevel;
use fh6_kit::{
    deg, vec2, Affine, Bloom, Color, Gear, Glow, Gradient, LayerOptions, Painter, Palette, Rect, Shape, Spread, Tokens,
    Vec2,
};

/// Digit slant in degrees (COMPONENTS.md §15); the top of the digit leans right.
pub const SLANT_DEG: f32 = 8.0;
/// Segment stroke as a fraction of the digit height (COMPONENTS.md §15: 10.5%).
pub const STROKE_RATIO: f32 = 0.105;
/// Gap between neighbouring segment tips as a fraction of the stroke (COMPONENTS.md §15: 18%).
pub const GAP_RATIO: f32 = 0.18;
/// Digit width as a fraction of its height (tokens RetroCyber `digitWidth` 42 / `digitHeight` 72).
pub const WIDTH_RATIO: f32 = 42.0 / 72.0;
/// Space between neighbouring cells as a fraction of the digit height; it leaves room for a
/// decimal point between two cells.
pub const SPACING_RATIO: f32 = 0.2;
/// Unlit segments show this fraction of the phosphor's displayed brightness (COMPONENTS.md §15:
/// 7%; the hero frames show the same dim phosphor).
pub const GHOST_LEVEL: f32 = 0.07;
/// The lit core is the phosphor mixed this far toward the white ink (COMPONENTS.md §15: 45%).
pub const CORE_WHITE: f32 = 0.45;
/// Time over which a new digit's segments light in sequence (sci-fi.md, gear change: 80 ms).
pub const SEQUENCE_SECONDS: f32 = 0.08;
/// Scanline thickness in 4K px (tokens `materials.vfdCavity.scanlinePx`).
pub const SCANLINE_PX: f32 = 2.0;
/// Scanline opacity out of 255 (tokens `materials.vfdCavity.scanlineAlpha`).
pub const SCANLINE_ALPHA: u8 = 34;
/// The decimal point's bit in a cell mask (bits 0–6 are segments a–g, as in the kit's masks).
pub const POINT: u8 = 0x80;
/// The minus sign's cell mask (segment g).
pub const MINUS: u8 = 0x40;
/// The N diagonal's bit in a [`Seg7Digit`] mask (cells with [`Segments::Diagonal`] only).
pub const DIAGONAL: u16 = 0x100;
/// The hero frames' neutral: both verticals and the diagonal (`b c e f` + [`DIAGONAL`]).
pub const NEUTRAL_DIAGONAL: u16 = 0x36 | DIAGONAL;

/// Width of the hot core as a fraction of the stroke.
const CORE_WIDTH: f32 = 0.4;
/// Soft edge of the hot core as a fraction of the stroke.
const CORE_SOFTNESS: f32 = 0.1;
/// Digit height at which the token glow lobes apply at their authored sigma. The lobes scale
/// linearly with the digit, so a readout looks the same at every size. Every segment glows on
/// its own and neighbouring halos add up inside a digit's counters: at this reference the
/// `glow.phosphor` halo (σ 10) is under half the counter, and at the RetroCyber speed cap of 120
/// (σ 5) the counters stay black and the segments crisp, as in the Sci-Fi hero frames.
const GLOW_REFERENCE_HEIGHT: f32 = 240.0;
/// Smallest and largest glow scale: tiny readouts keep a visible halo, huge ones stay inside the
/// tokens' bloom radius budget.
const GLOW_SCALE_RANGE: (f32, f32) = (0.25, 2.0);
/// Digit height up to which a lit phosphor writes the full [`PHOSPHOR_BLOOM`].
const BLOOM_REFERENCE_HEIGHT: f32 = 120.0;
/// Share of the tokens' bloom intensity a lit phosphor writes. VFD light is a tight core with a
/// short halo (tokens `glow.phosphor`): the lobes carry the halo, the display bloom only a trace,
/// so the true-black cavity around the digits stays black rather than fogged. Taller digits write
/// proportionally less (see [`Seg7Geometry::bloom_scale`]).
const PHOSPHOR_BLOOM: f32 = 0.03;
/// Extra space a colon adds between its two cells, as a fraction of the stroke.
const COLON_EXTRA: f32 = 1.4;
/// Vertical offset of the colon dots from the digit middle, as a fraction of the height.
const COLON_DOT_V: f32 = 0.2;
/// Number of drawable elements in a cell: segments a–g, the decimal point and the N diagonal.
const ELEMENTS: usize = 9;
/// Element index of the decimal point.
const POINT_INDEX: usize = 7;
/// Element index of the N diagonal.
const DIAGONAL_INDEX: usize = 8;

/// The segments a cell's glass carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Segments {
    /// Seven segments (numbers).
    #[default]
    Seven,
    /// Seven segments plus the N diagonal from the top-left to the bottom-right corner: the Sci-Fi
    /// gear bay, whose neutral is a real `N` and whose ghost shows the diagonal behind every gear.
    Diagonal,
}

impl Segments {
    /// The mask bits this glass can light.
    fn allowed(self) -> u16 {
        match self {
            Segments::Seven => 0xFF,
            Segments::Diagonal => 0xFF | DIAGONAL,
        }
    }
}

/// The mask for `gear` on a cell with `segments`: `r` for reverse, `N` for neutral (the diagonal
/// `N` on [`Segments::Diagonal`], the seven-segment `N` otherwise) and the digit for 1–9. One cell
/// has no room for two digits, so gears above 9 show their last digit with the decimal point
/// lit (10th gear reads `0.`); a cluster that wants `10` draws a two-cell [`Seg7Number`].
#[must_use]
pub fn gear_mask(gear: Gear, segments: Segments) -> u16 {
    match gear {
        Gear::Reverse => u16::from(seven_seg_mask('r')),
        Gear::Neutral => match segments {
            Segments::Diagonal => NEUTRAL_DIAGONAL,
            Segments::Seven => u16::from(seven_seg_mask('N')),
        },
        Gear::Forward(0) => u16::from(MINUS),
        Gear::Forward(n) if n <= 9 => u16::from(SEVEN_SEG_DIGITS[usize::from(n)]),
        Gear::Forward(n) => u16::from(SEVEN_SEG_DIGITS[usize::from(n % 10)] | POINT),
    }
}

/// Which palette role a digit's phosphor takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phosphor {
    /// `primary` (phosphor cyan in the Sci-Fi family), text-safe: speed and live numbers.
    #[default]
    Primary,
    /// `secondary` (caution yellow in Sci-Fi), text-safe.
    Secondary,
    /// `accent` (magenta in Sci-Fi), text-safe: the limiter cue.
    Accent,
    /// `shift`: the limiter colour, as is.
    Shift,
    /// The white `ink`: a hero gear.
    Ink,
    /// The fixed `warning` role: reverse gear.
    Warning,
}

impl Phosphor {
    /// The lit colour of this phosphor in `palette`.
    #[must_use]
    pub fn color(self, palette: &Palette) -> Color {
        match self {
            Phosphor::Primary => palette.primary_text,
            Phosphor::Secondary => palette.secondary.text_safe(),
            Phosphor::Accent => palette.accent_text,
            Phosphor::Shift => palette.shift,
            Phosphor::Ink => palette.roles.ink,
            Phosphor::Warning => palette.roles.warning,
        }
    }
}

/// Whether lit segments glow and write to the emissive (bloom) buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Emission {
    /// Glance ranks 2–3 phosphor readouts (speed, tach numbers): the tight `glow.phosphor` lobes
    /// in a bloom layer.
    #[default]
    Bloom,
    /// The hero gear while the limiter is active (COMPONENTS.md §1: "soft glow only while the
    /// limiter is active"): the single `glow.soft` lobe in a bloom layer, so a large white glyph
    /// keeps its edges and the ghosts around it stay readable.
    Soft,
    /// The `glow.phosphor` lobes of [`Emission::Bloom`] as local glows in the current layer, with
    /// no bloom: the halo stays inside the cavity and the frame needs no bloom pass, which grows
    /// the whole work rectangle by the bloom margin and adds the bloom pyramid (about 0.09 ms at
    /// 4K for a speed readout). For a dash whose GPU budget has no room for bloom.
    Halo,
    /// The single `glow.soft` lobe of [`Emission::Soft`], local, with no bloom (the hero gear on
    /// the limiter of a dash that uses [`Emission::Halo`]).
    SoftHalo,
    /// Ranks 4–5 (mini readouts, data tiles) and the hero gear off the limiter: lit segments with
    /// their core, no glow, no bloom.
    Matte,
}

/// How a cell's segments follow a new value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Reveal {
    /// Segments switch on the frame the value changes (live numbers: speed, rpm).
    #[default]
    Instant,
    /// The new digit re-strikes: its segments light one after another over
    /// [`SEQUENCE_SECONDS`] (clockwise from the top, the middle last) while the old digit's
    /// segments fade over `motion.durations.micro` until their turn (a gear change).
    Sequence,
}

/// What fills the leading cells of a [`Seg7Number`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Leading {
    /// Unlit cells, so only the ghost `8` shows (TabularSpeed's ghost 8s).
    #[default]
    Ghost,
    /// Lit zeros (counters, lap numbers).
    Zeros,
}

/// Punctuation inside a [`Seg7Number`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Separator {
    /// Digits only.
    #[default]
    Plain,
    /// A fixed decimal point: the value is in units of `10^-places` (`125` with 1 place shows
    /// `12.5`); the cell before the point always shows a digit.
    Decimal {
        /// Digits after the point (at most `N - 1`).
        places: u8,
    },
    /// A steady colon after the first `after` cells (`1:42`); it widens the block slightly.
    Colon {
        /// Number of cells left of the colon (1..N).
        after: u8,
    },
}

impl Separator {
    /// Digits after the decimal point, clamped so at least one digit stands before it.
    fn places(self, cells: usize) -> usize {
        match self {
            Separator::Decimal { places } => usize::from(places).min(cells.saturating_sub(1)),
            Separator::Plain | Separator::Colon { .. } => 0,
        }
    }

    /// The cell index right of the colon, when the colon fits between two cells.
    fn colon_after(self, cells: usize) -> Option<usize> {
        match self {
            Separator::Colon { after } if (1..cells).contains(&usize::from(after)) => Some(usize::from(after)),
            Separator::Plain | Separator::Decimal { .. } | Separator::Colon { .. } => None,
        }
    }

    /// The cell that carries the decimal point, if any.
    fn point_cell(self, cells: usize) -> Option<usize> {
        let places = self.places(cells);
        (places > 0).then(|| cells - 1 - places)
    }
}

/// Geometry of one slanted VFD digit cell, in local px (4K reference pixels).
///
/// Cell coordinates `(u, v)` are centred on the digit; the slant maps them to
/// `(x + u - v * tan(slant), y + v)`, so the digit leans right about its middle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seg7Geometry {
    /// Digit height: top edge of segment `a` to bottom edge of segment `d`.
    pub height: f32,
    /// Digit width before the slant.
    pub width: f32,
    /// Segment stroke.
    pub stroke: f32,
    /// Gap between neighbouring segment tips, measured across the chamfers.
    pub gap: f32,
    /// Slant in radians (positive leans the top right).
    pub slant: f32,
    /// Space between neighbouring cells.
    pub spacing: f32,
}

impl Seg7Geometry {
    /// The spec geometry for digits `height` px tall: width 58%, stroke 10.5%, gap 18% of the
    /// stroke, 8° slant, spacing 20% of the height.
    #[must_use]
    pub fn new(height: f32) -> Self {
        let height = if height.is_finite() { height.max(1.0) } else { 1.0 };
        let stroke = height * STROKE_RATIO;
        Self {
            height,
            width: height * WIDTH_RATIO,
            stroke,
            gap: stroke * GAP_RATIO,
            slant: deg(SLANT_DEG),
            spacing: height * SPACING_RATIO,
        }
    }

    /// The same geometry with the digit width set to `ratio` of its height (clamped to a
    /// width that still leaves the middle segment longer than its chamfers).
    #[must_use]
    pub fn with_width_ratio(mut self, ratio: f32) -> Self {
        let ratio = if ratio.is_finite() { ratio } else { WIDTH_RATIO };
        self.width = self.height * ratio.clamp(0.4, 1.0);
        self
    }

    /// Distance from one cell centre to the next.
    #[must_use]
    pub fn advance(&self) -> f32 {
        self.width + self.spacing
    }

    /// Horizontal shift per unit of `v` (the tangent of the slant).
    #[must_use]
    pub fn shear(&self) -> f32 {
        self.slant.tan()
    }

    /// Maps a cell coordinate `uv` of the digit centred at `center` through the slant.
    #[must_use]
    pub fn map(&self, center: Vec2, uv: Vec2) -> Vec2 {
        vec2(center.x + uv.x - uv.y * self.shear(), center.y + uv.y)
    }

    /// The bounds of a digit centred at `center`, slant overhang and decimal point included.
    #[must_use]
    pub fn bounds(&self, center: Vec2) -> Rect {
        let lean = self.height * 0.5 * self.shear().abs();
        let half_w = self.width * 0.5 + self.spacing * 0.5 + lean;
        Rect::from_center_size(center, vec2(half_w * 2.0, self.height))
    }

    /// Segment `index` (0..7 = a..g, wrapping) of the digit centred at `center`: a chamfered
    /// hexagon, slanted, with its tips pulled back so neighbours keep the gap.
    #[must_use]
    pub fn segment(&self, center: Vec2, index: usize) -> [Vec2; 6] {
        let (a, b) = self.axis(index);
        self.hex(center, a, b, self.stroke * 0.5, self.tip_pull())
    }

    /// The N diagonal ([`Segments::Diagonal`]) of the digit centred at `center`: a chamfered bar
    /// from the top-left corner to the bottom-right corner, joining the tops of `f` and the
    /// bottom of `c` like the hero frames' neutral.
    #[must_use]
    pub fn diagonal(&self, center: Vec2) -> [Vec2; 6] {
        let (a, b) = self.diagonal_axis();
        self.hex(center, a, b, self.stroke * 0.5, self.tip_pull())
    }

    /// The outline of element `index` (0..7 = a..g, [`DIAGONAL_INDEX`] = the diagonal).
    fn element(&self, center: Vec2, index: usize) -> [Vec2; 6] {
        if index == DIAGONAL_INDEX {
            self.diagonal(center)
        } else {
            self.segment(center, index)
        }
    }

    /// The hot core of element `index`: the same hexagon inset to [`CORE_WIDTH`] of the stroke.
    fn element_core(&self, center: Vec2, index: usize) -> [Vec2; 6] {
        let (a, b) = if index == DIAGONAL_INDEX {
            self.diagonal_axis()
        } else {
            self.axis(index)
        };
        let half = self.stroke * 0.5;
        let core = half * CORE_WIDTH;
        // A parallel chamfer inset by (half - core) moves the tip by that distance times sqrt(2).
        self.hex(
            center,
            a,
            b,
            core,
            self.tip_pull() + (half - core) * std::f32::consts::SQRT_2,
        )
    }

    /// The decimal point of the digit centred at `center`: a slanted square of the stroke's
    /// size, low in the spacing right of the digit.
    #[must_use]
    pub fn point(&self, center: Vec2) -> [Vec2; 4] {
        self.square(center, self.point_uv(), self.stroke * 0.5)
    }

    /// The colon's two dots around `center` (the middle of the space between two cells).
    #[must_use]
    pub fn colon(&self, center: Vec2) -> [[Vec2; 4]; 2] {
        let v = self.height * COLON_DOT_V;
        let half = self.stroke * 0.5;
        [
            self.square(center, vec2(0.0, -v), half),
            self.square(center, vec2(0.0, v), half),
        ]
    }

    /// Extra width a colon adds between its two cells.
    #[must_use]
    pub fn colon_extra(&self) -> f32 {
        self.stroke * COLON_EXTRA
    }

    /// Glow sigma multiplier: the token lobes at [`GLOW_REFERENCE_HEIGHT`], proportional to the
    /// height (within [`GLOW_SCALE_RANGE`]) so the halo keeps its size relative to the stroke.
    fn glow_scale(&self) -> f32 {
        let (min, max) = GLOW_SCALE_RANGE;
        (self.height / GLOW_REFERENCE_HEIGHT).clamp(min, max)
    }

    /// Bloom write multiplier: 1 up to [`BLOOM_REFERENCE_HEIGHT`], then falling with the height.
    /// The bloom pyramid has a fixed radius in pixels, so the veil a digit casts grows with its
    /// lit area; without this a 300 px gear at the limiter fogs its whole bay and drowns its own
    /// ghost segments.
    fn bloom_scale(&self) -> f32 {
        (BLOOM_REFERENCE_HEIGHT / self.height).min(1.0)
    }

    /// How far each tip is pulled back along its axis: two 45° chamfers pulled back by `s` sit
    /// `s * sqrt(2)` apart, so `s = gap / sqrt(2)`.
    fn tip_pull(&self) -> f32 {
        self.gap * std::f32::consts::FRAC_1_SQRT_2
    }

    fn point_uv(&self) -> Vec2 {
        vec2(
            self.width * 0.5 + self.spacing * 0.5,
            self.height * 0.5 - self.stroke * 0.5,
        )
    }

    /// The stroke-centre corners: half the width and half the height, each less half a stroke.
    fn corners(&self) -> (f32, f32) {
        let half = self.stroke * 0.5;
        (self.width * 0.5 - half, self.height * 0.5 - half)
    }

    /// The centre line of the N diagonal in cell coordinates (top-left to bottom-right corner).
    fn diagonal_axis(&self) -> (Vec2, Vec2) {
        let (x, y) = self.corners();
        (vec2(-x, -y), vec2(x, y))
    }

    /// The centre line of segment `index` in cell coordinates (corner to corner).
    fn axis(&self, index: usize) -> (Vec2, Vec2) {
        let (x, y) = self.corners();
        let (tl, tr) = (vec2(-x, -y), vec2(x, -y));
        let (ml, mr) = (vec2(-x, 0.0), vec2(x, 0.0));
        let (bl, br) = (vec2(-x, y), vec2(x, y));
        match index % 7 {
            0 => (tl, tr),
            1 => (tr, mr),
            2 => (mr, br),
            3 => (br, bl),
            4 => (bl, ml),
            5 => (ml, tl),
            _ => (ml, mr),
        }
    }

    /// A pointed hexagon of half-width `half` along `a -> b`, its tips pulled back by `pull`,
    /// slanted and placed at `center`.
    fn hex(&self, center: Vec2, a: Vec2, b: Vec2, half: f32, pull: f32) -> [Vec2; 6] {
        let d = (b - a).normalize_or_zero();
        let n = d.perp();
        [
            a + d * pull,
            a + d * (pull + half) + n * half,
            b - d * (pull + half) + n * half,
            b - d * pull,
            b - d * (pull + half) - n * half,
            a + d * (pull + half) - n * half,
        ]
        .map(|uv| self.map(center, uv))
    }

    fn square(&self, center: Vec2, uv: Vec2, half: f32) -> [Vec2; 4] {
        [
            vec2(uv.x - half, uv.y - half),
            vec2(uv.x + half, uv.y - half),
            vec2(uv.x + half, uv.y + half),
            vec2(uv.x - half, uv.y + half),
        ]
        .map(|p| self.map(center, p))
    }
}

/// One VFD digit cell with per-segment phosphor levels.
///
/// The mask uses the kit's bits (`a=1 b=2 c=4 d=8 e=16 f=32 g=64`, see
/// [`fh6_kit::shape::seven_seg_mask`]) plus [`POINT`] for the decimal point and, on a
/// [`Segments::Diagonal`] cell, [`DIAGONAL`]. The point's ghost is not drawn by a lone digit;
/// [`Seg7Number`] draws it on the cell that carries the point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seg7Digit {
    mask: u16,
    rising: u16,
    clock: f32,
    levels: [f32; ELEMENTS],
    reveal: Reveal,
    fade_seconds: f32,
    segments: Segments,
    phosphor: Phosphor,
    ghost: Phosphor,
    brightness: f32,
}

impl Seg7Digit {
    /// A blank seven-segment digit (ghost `8` only) in the primary phosphor; old segments of a
    /// [`Reveal::Sequence`] digit fade over `tokens.durations.micro`.
    #[must_use]
    pub fn new(tokens: &Tokens, reveal: Reveal) -> Self {
        Self {
            mask: 0,
            rising: 0,
            clock: 0.0,
            levels: [0.0; ELEMENTS],
            reveal,
            fade_seconds: positive_or(tokens.durations.micro, 0.05),
            segments: Segments::Seven,
            phosphor: Phosphor::Primary,
            ghost: Phosphor::Primary,
            brightness: 1.0,
        }
    }

    /// The same digit on another glass ([`Segments::Diagonal`] for the Sci-Fi gear bay).
    #[must_use]
    pub fn with_segments(mut self, segments: Segments) -> Self {
        self.segments = segments;
        self
    }

    /// The segments this digit's glass carries.
    #[must_use]
    pub fn segments(&self) -> Segments {
        self.segments
    }

    /// The same digit with `phosphor` for both its lit and its ghost segments (the cell's area
    /// of the VFD glass).
    #[must_use]
    pub fn with_phosphor(mut self, phosphor: Phosphor) -> Self {
        self.phosphor = phosphor;
        self.ghost = phosphor;
        self
    }

    /// The same digit with its ghost segments in `ghost` (a white [`Phosphor::Ink`] hero gear
    /// sits on the panel's [`Phosphor::Primary`] ghosts, as in the Sci-Fi hero frames).
    #[must_use]
    pub fn with_ghost(mut self, ghost: Phosphor) -> Self {
        self.ghost = ghost;
        self
    }

    /// Switches the lit phosphor and keeps the ghost (reverse turns [`Phosphor::Warning`] on the
    /// same glass).
    pub fn set_phosphor(&mut self, phosphor: Phosphor) {
        self.phosphor = phosphor;
    }

    /// The lit phosphor.
    #[must_use]
    pub fn phosphor(&self) -> Phosphor {
        self.phosphor
    }

    /// The ghost phosphor.
    #[must_use]
    pub fn ghost(&self) -> Phosphor {
        self.ghost
    }

    /// Scales the lit segments' light (0..=1; HeroGear shows `N` at 0.72). The light dims at full
    /// coverage, so overlapping elements (the `N` diagonal over `f` and `c`) stay seamless; ghosts
    /// stay at [`GHOST_LEVEL`]. A non-finite value resets to 1.
    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = if brightness.is_finite() {
            brightness.clamp(0.0, 1.0)
        } else {
            1.0
        };
    }

    /// The lit segments' light scale.
    #[must_use]
    pub fn brightness(&self) -> f32 {
        self.brightness
    }

    /// Advances to `mask` (segments a–g in bits 0–6, [`POINT`] in bit 7, [`DIAGONAL`] in bit 8)
    /// by `dt` seconds. Bits the cell's [`Segments`] cannot show are dropped.
    pub fn update(&mut self, mask: u16, dt: f32) {
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        let mask = mask & self.segments.allowed();
        if mask != self.mask {
            // A sequence re-strikes the whole new digit: segments it shares with the old one
            // fade with the rest until their turn comes.
            self.rising = match self.reveal {
                Reveal::Sequence => mask,
                Reveal::Instant => 0,
            };
            self.mask = mask;
            self.clock = 0.0;
        } else {
            self.clock += dt;
        }
        let rising = self.rising;
        let steps = rising.count_ones().saturating_sub(1).max(1) as f32;
        let decay = dt / self.fade_seconds;
        for (i, level) in self.levels.iter_mut().enumerate() {
            let bit = 1u16 << i;
            let on = if mask & bit == 0 {
                false
            } else if rising & bit == 0 {
                true
            } else {
                let rank = (rising & (bit - 1)).count_ones() as f32;
                self.clock + 1e-6 >= SEQUENCE_SECONDS * rank / steps
            };
            *level = match (on, self.reveal) {
                (true, _) => 1.0,
                (false, Reveal::Instant) => 0.0,
                (false, Reveal::Sequence) => (*level - decay).max(0.0),
            };
        }
    }

    /// Advances to the character `ch` (`0-9`, `-`, `r`, `n`, `N`, `P`, `E`, space).
    pub fn update_char(&mut self, ch: char, dt: f32) {
        self.update(u16::from(seven_seg_mask(ch)), dt);
    }

    /// Advances to `gear` (see [`gear_mask`]; the diagonal `N` on a [`Segments::Diagonal`] cell).
    pub fn update_gear(&mut self, gear: Gear, dt: f32) {
        self.update(gear_mask(gear, self.segments), dt);
    }

    /// The target mask.
    #[must_use]
    pub fn mask(&self) -> u16 {
        self.mask
    }

    /// The phosphor level (0..=1) of element `index` (0..7 = a..g, 7 = the point, 8 = the
    /// diagonal).
    #[must_use]
    pub fn level(&self, index: usize) -> f32 {
        self.levels.get(index).copied().unwrap_or(0.0)
    }

    /// True while a reveal sequence or a fade still runs without new input.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.levels.iter().enumerate().any(|(i, &level)| {
            if self.mask & (1u16 << i) != 0 {
                level < 1.0
            } else {
                level > 0.0
            }
        })
    }

    /// Draws the digit centred at `center`: ghost segments, then the lit phosphor.
    pub fn draw(&self, p: &mut Painter<'_>, center: Vec2, geom: &Seg7Geometry, palette: &Palette, emission: Emission) {
        let inks = Inks::new(self.phosphor, self.ghost, palette, self.brightness);
        let glass = Glass {
            point: false,
            diagonal: self.segments == Segments::Diagonal,
        };
        ghost_cell(p, geom, center, &self.levels, inks.ghost, glass);
        emit(p, geom, emission, |p| {
            lit_cell(p, geom, center, &self.levels, &inks, emission)
        });
    }
}

/// `N` fixed VFD cells showing an integer right-aligned: the width never changes, unused leading
/// cells show the ghost `8` (or zeros), with an optional decimal point or colon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seg7Number<const N: usize> {
    cells: [Seg7Digit; N],
    leading: Leading,
    separator: Separator,
    phosphor: Phosphor,
    ghost: Phosphor,
    value: i64,
}

impl<const N: usize> Seg7Number<N> {
    /// `N` blank cells in the primary phosphor, ghost leading cells, no separator.
    #[must_use]
    pub fn new(tokens: &Tokens, reveal: Reveal) -> Self {
        Self {
            cells: [Seg7Digit::new(tokens, reveal); N],
            leading: Leading::Ghost,
            separator: Separator::Plain,
            phosphor: Phosphor::Primary,
            ghost: Phosphor::Primary,
            value: 0,
        }
    }

    /// The same readout with other leading cells.
    #[must_use]
    pub fn with_leading(mut self, leading: Leading) -> Self {
        self.leading = leading;
        self
    }

    /// The same readout with a decimal point or colon.
    #[must_use]
    pub fn with_separator(mut self, separator: Separator) -> Self {
        self.separator = separator;
        self
    }

    /// The same readout with `phosphor` for both its lit and its ghost segments.
    #[must_use]
    pub fn with_phosphor(mut self, phosphor: Phosphor) -> Self {
        self.phosphor = phosphor;
        self.ghost = phosphor;
        self
    }

    /// The same readout with its ghost segments in `ghost`.
    #[must_use]
    pub fn with_ghost(mut self, ghost: Phosphor) -> Self {
        self.ghost = ghost;
        self
    }

    /// Switches the lit phosphor and keeps the ghost (the rpm readout turns
    /// [`Phosphor::Accent`] on the limiter).
    pub fn set_phosphor(&mut self, phosphor: Phosphor) {
        self.phosphor = phosphor;
    }

    /// The lit phosphor.
    #[must_use]
    pub fn phosphor(&self) -> Phosphor {
        self.phosphor
    }

    /// Advances to `value` (clamped to what `N` cells can show) by `dt` seconds.
    pub fn update(&mut self, value: i64, dt: f32) {
        let (masks, shown) = number_masks::<N>(value, self.leading, self.separator);
        self.value = shown;
        for (cell, mask) in self.cells.iter_mut().zip(masks) {
            cell.update(u16::from(mask), dt);
        }
    }

    /// The value shown after clamping.
    #[must_use]
    pub fn value(&self) -> i64 {
        self.value
    }

    /// The cells, left to right.
    #[must_use]
    pub fn cells(&self) -> &[Seg7Digit; N] {
        &self.cells
    }

    /// True while any cell still animates.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.cells.iter().any(Seg7Digit::animating)
    }

    /// Width of the block (cells, spacing and colon) in local px.
    #[must_use]
    pub fn width(&self, geom: &Seg7Geometry) -> f32 {
        block_width(N, geom, self.separator)
    }

    /// The centre of cell `index` for a block centred at `center`.
    #[must_use]
    pub fn cell_center(&self, center: Vec2, geom: &Seg7Geometry, index: usize) -> Vec2 {
        vec2(center.x + cell_offset(N, geom, self.separator, index), center.y)
    }

    /// Draws the block centred at `center`: all ghosts first, then the lit phosphor in one
    /// emissive group.
    pub fn draw(&self, p: &mut Painter<'_>, center: Vec2, geom: &Seg7Geometry, palette: &Palette, emission: Emission) {
        let inks = Inks::new(self.phosphor, self.ghost, palette, 1.0);
        let point_cell = self.separator.point_cell(N);
        for (i, cell) in self.cells.iter().enumerate() {
            let c = self.cell_center(center, geom, i);
            let glass = Glass {
                point: point_cell == Some(i),
                diagonal: false,
            };
            ghost_cell(p, geom, c, &cell.levels, inks.ghost, glass);
        }
        let colon = self.separator.colon_after(N).map(|after| {
            let left = self.cell_center(center, geom, after - 1);
            vec2(
                left.x + (geom.width + geom.spacing + geom.colon_extra()) * 0.5,
                center.y,
            )
        });
        emit(p, geom, emission, |p| {
            for (i, cell) in self.cells.iter().enumerate() {
                let c = self.cell_center(center, geom, i);
                lit_cell(p, geom, c, &cell.levels, &inks, emission);
            }
            if let Some(c) = colon {
                let lobes = glow_lobes(p.tokens(), geom, emission);
                let core_half = geom.stroke * 0.5 * CORE_WIDTH;
                for (dot, v) in geom.colon(c).iter().zip([-1.0, 1.0]) {
                    let core = geom.square(c, vec2(0.0, v * geom.height * COLON_DOT_V), core_half);
                    light(p, dot, &core, 1.0, &inks, lobes, geom.stroke * CORE_SOFTNESS);
                }
            }
        });
    }
}

/// The cell masks for `value` right-aligned in `N` cells, and the value they show.
///
/// Values above `10^N - 1` show all nines and values below `-(10^(N-1) - 1)` clamp there; a
/// negative value takes a minus in the cell left of its first digit (cell 0 with
/// [`Leading::Zeros`]), and shows dashes in every cell when there is no room for the minus.
/// With [`Separator::Decimal`] the cell before the point always shows a digit and carries
/// [`POINT`].
#[must_use]
pub fn number_masks<const N: usize>(value: i64, leading: Leading, separator: Separator) -> ([u8; N], i64) {
    let mut masks = [0u8; N];
    if N == 0 {
        return (masks, 0);
    }
    let places = separator.places(N);
    let min_digits = places + 1;
    let max = pow10(N).saturating_sub(1);
    let min = -pow10(N - 1).saturating_sub(1);
    let shown = value.clamp(min, max);
    let mut magnitude = shown.unsigned_abs();
    let mut first = N;
    for (pos, k) in (0..N).rev().enumerate() {
        if magnitude > 0 || pos < min_digits {
            masks[k] = SEVEN_SEG_DIGITS[(magnitude % 10) as usize];
            magnitude /= 10;
            first = k;
        } else if leading == Leading::Zeros {
            masks[k] = SEVEN_SEG_DIGITS[0];
        }
    }
    if shown < 0 {
        let slot = match leading {
            Leading::Ghost => first.checked_sub(1),
            Leading::Zeros => (first > 0).then_some(0),
        };
        match slot {
            Some(s) => masks[s] = MINUS,
            None => return ([MINUS; N], shown),
        }
    }
    if let Some(cell) = separator.point_cell(N) {
        masks[cell] |= POINT;
    }
    (masks, shown)
}

/// Draws a speed as the `segment` TabularSpeed variant (COMPONENTS.md §2): 3 fixed VFD cells of
/// `cell_height` centred at `center`, ghost 8s in the unused leading cells, the primary phosphor
/// with its glow (speed is glance rank 2). Pass the hysteresis-filtered integer (for example
/// [`crate::common::SpeedReadout::value`]); it clamps to `0..=999`.
pub fn draw_speed(p: &mut Painter<'_>, center: Vec2, cell_height: f32, value: i64, palette: &Palette) {
    let geom = Seg7Geometry::new(cell_height);
    let (masks, _) = number_masks::<3>(value.max(0), Leading::Ghost, Separator::Plain);
    let inks = Inks::new(Phosphor::Primary, Phosphor::Primary, palette, 1.0);
    let levels = masks.map(mask_levels);
    for (i, cell) in levels.iter().enumerate() {
        let c = vec2(center.x + cell_offset(3, &geom, Separator::Plain, i), center.y);
        ghost_cell(p, &geom, c, cell, inks.ghost, Glass::default());
    }
    emit(p, &geom, Emission::Bloom, |p| {
        for (i, cell) in levels.iter().enumerate() {
            let c = vec2(center.x + cell_offset(3, &geom, Separator::Plain, i), center.y);
            lit_cell(p, &geom, c, cell, &inks, Emission::Bloom);
        }
    });
}

/// Draws the VFD cavity's horizontal scanlines over `rect` (local px): [`SCANLINE_PX`] lines of
/// `oledBlack` at [`SCANLINE_ALPHA`], one line per two line heights, snapped to device pixels so
/// they never shimmer. They only darken what is lit underneath (off pixels stay black), so
/// draw them last, over the cavity only. One instance, whatever the size.
pub fn draw_scanlines(p: &mut Painter<'_>, rect: Rect, palette: &Palette) {
    let xf = p.current_transform();
    let target = xf.apply_rect(&rect);
    if target.is_empty() || !target.min.is_finite() || !target.max.is_finite() {
        return;
    }
    let (period, line) = scanline_pitch(xf.scale_factor());
    let snapped = Rect::from_points(
        vec2(target.min.x.floor(), target.min.y.floor()),
        vec2(target.max.x.ceil(), target.max.y.ceil()),
    );
    let dark = palette.roles.oled_black.with_alpha8(SCANLINE_ALPHA);
    let clear = dark.with_alpha(0.0);
    let split = line / period;
    let paint = Gradient::linear(vec2(0.0, snapped.min.y), vec2(0.0, snapped.min.y + period))
        .stop(0.0, dark)
        .stop(split, dark)
        .stop(split, clear)
        .stop(1.0, clear)
        .spread(Spread::Repeat);
    p.scoped(|p| {
        p.set_transform(Affine::IDENTITY);
        p.fill(&Shape::rect(snapped), paint);
    });
}

/// Scanline period and line thickness in whole device pixels for `scale` device px per 4K px.
fn scanline_pitch(scale: f32) -> (f32, f32) {
    let scale = if scale.is_finite() { scale.max(0.0) } else { 1.0 };
    let period = (2.0 * SCANLINE_PX * scale).round().max(2.0);
    let line = (SCANLINE_PX * scale).round().clamp(1.0, period - 1.0);
    (period, line)
}

/// `10^n`, saturating at `i64::MAX`.
fn pow10(n: usize) -> i64 {
    (0..n).fold(1i64, |v, _| v.saturating_mul(10))
}

/// Full-on / off levels for a mask (no animation).
fn mask_levels(mask: u8) -> [f32; ELEMENTS] {
    let mut levels = [0.0; ELEMENTS];
    for (i, level) in levels.iter_mut().enumerate() {
        if u16::from(mask) & (1u16 << i) != 0 {
            *level = 1.0;
        }
    }
    levels
}

fn block_width(cells: usize, geom: &Seg7Geometry, separator: Separator) -> f32 {
    if cells == 0 {
        return 0.0;
    }
    let colon = if separator.colon_after(cells).is_some() {
        geom.colon_extra()
    } else {
        0.0
    };
    cells as f32 * geom.width + (cells - 1) as f32 * geom.spacing + colon
}

/// Horizontal offset of cell `index` from the block centre.
fn cell_offset(cells: usize, geom: &Seg7Geometry, separator: Separator, index: usize) -> f32 {
    let colon = match separator.colon_after(cells) {
        Some(after) if index >= after => geom.colon_extra(),
        _ => 0.0,
    };
    -block_width(cells, geom, separator) * 0.5 + geom.width * 0.5 + index as f32 * geom.advance() + colon
}

/// `v` when it is finite and positive, else `fallback`.
fn positive_or(v: f32, fallback: f32) -> f32 {
    if v.is_finite() && v > 0.0 {
        v
    } else {
        fallback
    }
}

/// The three colours of one cell.
#[derive(Debug, Clone, Copy)]
struct Inks {
    lit: Color,
    core: Color,
    ghost: Color,
}

impl Inks {
    /// Lit and core from `phosphor` at `brightness`, the ghost from `ghost`.
    fn new(phosphor: Phosphor, ghost: Phosphor, palette: &Palette, brightness: f32) -> Self {
        let lit = phosphor.color(palette);
        Self {
            lit: lit.intensity(brightness),
            core: lit.lerp(palette.roles.ink, CORE_WHITE).intensity(brightness),
            ghost: ghost_of(ghost.color(palette)),
        }
    }
}

/// The ghost colour: opaque, each channel at [`GHOST_LEVEL`] of the phosphor's displayed (sRGB)
/// value. Being opaque, it shows the same 7% over a VFD cavity and over the transparent overlay
/// (the resolve encodes covered light per pixel, so a translucent ghost would read 7% on one and
/// far brighter on the other).
fn ghost_of(phosphor: Color) -> Color {
    let (r, g, b) = phosphor.straight_rgb();
    let dim = |c: f32| srgb_to_linear(GHOST_LEVEL * linear_to_srgb(c.clamp(0.0, 1.0)));
    Color::linear(dim(r), dim(g), dim(b), 1.0)
}

/// The bloom a phosphor layer of digits shaped by `geom` writes: the tokens' threshold, and the
/// tokens' intensity scaled by [`PHOSPHOR_BLOOM`] and the digit size.
fn phosphor_bloom(tokens: &Tokens, geom: &Seg7Geometry) -> Bloom {
    Bloom {
        threshold: tokens.glow.bloom_threshold,
        intensity: tokens.glow.bloom_intensity * PHOSPHOR_BLOOM * geom.bloom_scale(),
    }
}

/// Runs `f` in a phosphor bloom layer for [`Emission::Bloom`] and [`Emission::Soft`], in the
/// current layer for the others.
fn emit<'f>(p: &mut Painter<'f>, geom: &Seg7Geometry, emission: Emission, f: impl FnOnce(&mut Painter<'f>)) {
    match emission {
        Emission::Bloom | Emission::Soft => {
            let options = LayerOptions {
                bloom: Some(phosphor_bloom(p.tokens(), geom)),
                ..LayerOptions::default()
            };
            p.layer(options, f);
        }
        Emission::Halo | Emission::SoftHalo | Emission::Matte => f(p),
    }
}

/// The glow lobes for `emission` and their scale, or `None` for [`Emission::Matte`].
fn glow_lobes(tokens: &Tokens, geom: &Seg7Geometry, emission: Emission) -> Option<(GlowLevel, f32)> {
    let glow = &tokens.glow;
    match emission {
        Emission::Bloom | Emission::Halo => Some((glow.phosphor, geom.glow_scale())),
        Emission::Soft | Emission::SoftHalo => Some((glow.soft, geom.glow_scale())),
        Emission::Matte => None,
    }
}

/// The optional elements a cell's glass shows as ghosts besides segments a–g.
#[derive(Debug, Clone, Copy, Default)]
struct Glass {
    /// The decimal point (the cell that carries a [`Separator::Decimal`] point).
    point: bool,
    /// The N diagonal ([`Segments::Diagonal`]).
    diagonal: bool,
}

/// Ghost elements of the cell's glass that are not fully lit.
fn ghost_cell(
    p: &mut Painter<'_>,
    geom: &Seg7Geometry,
    center: Vec2,
    levels: &[f32; ELEMENTS],
    ghost: Color,
    glass: Glass,
) {
    if glass.diagonal && levels[DIAGONAL_INDEX] < 1.0 {
        p.fill(
            &Shape::Polygon {
                points: &geom.diagonal(center),
            },
            ghost,
        );
    }
    for (i, &level) in levels.iter().take(7).enumerate() {
        if level < 1.0 {
            p.fill(
                &Shape::Polygon {
                    points: &geom.segment(center, i),
                },
                ghost,
            );
        }
    }
    if glass.point && levels[POINT_INDEX] < 1.0 {
        p.fill(
            &Shape::Polygon {
                points: &geom.point(center),
            },
            ghost,
        );
    }
}

/// The lit elements of one cell at their levels.
fn lit_cell(
    p: &mut Painter<'_>,
    geom: &Seg7Geometry,
    center: Vec2,
    levels: &[f32; ELEMENTS],
    inks: &Inks,
    emission: Emission,
) {
    let lobes = glow_lobes(p.tokens(), geom, emission);
    let softness = geom.stroke * CORE_SOFTNESS;
    for (i, &level) in levels.iter().enumerate() {
        if level <= 0.0 || i == POINT_INDEX {
            continue;
        }
        let outer = geom.element(center, i);
        let core = geom.element_core(center, i);
        light(p, &outer, &core, level, inks, lobes, softness);
    }
    let point = levels[POINT_INDEX];
    if point > 0.0 {
        let outer = geom.point(center);
        let core = geom.square(center, geom.point_uv(), geom.stroke * 0.5 * CORE_WIDTH);
        light(p, &outer, &core, point, inks, lobes, softness);
    }
}

/// One lit element: the phosphor halo (token lobes), the phosphor body and the soft hot core.
fn light(
    p: &mut Painter<'_>,
    outer: &[Vec2],
    core: &[Vec2],
    level: f32,
    inks: &Inks,
    lobes: Option<(GlowLevel, f32)>,
    softness: f32,
) {
    let body = Shape::Polygon { points: outer };
    let lit = inks.lit.fade(level);
    if let Some((lobes, scale)) = lobes {
        for lobe in lobes.lobes() {
            p.glow(&body, lit, Glow::new(lobe.sigma * scale, lobe.intensity));
        }
    }
    p.fill(&body, lit);
    p.glow(
        &Shape::Polygon { points: core },
        inks.core.fade(level),
        Glow {
            sigma: softness.max(0.5),
            intensity: 1.0,
            fill_inside: true,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn digit(ch: char) -> u8 {
        seven_seg_mask(ch)
    }

    #[test]
    fn masks_right_align_with_ghost_leading_cells() {
        assert_eq!(
            number_masks::<3>(88, Leading::Ghost, Separator::Plain),
            ([0, 0x7F, 0x7F], 88)
        );
        assert_eq!(
            number_masks::<3>(0, Leading::Ghost, Separator::Plain),
            ([0, 0, digit('0')], 0)
        );
        assert_eq!(
            number_masks::<3>(152, Leading::Ghost, Separator::Plain),
            ([digit('1'), digit('5'), digit('2')], 152)
        );
        assert_eq!(
            number_masks::<4>(7450, Leading::Ghost, Separator::Plain).0,
            [digit('7'), digit('4'), digit('5'), digit('0')]
        );
    }

    #[test]
    fn masks_clamp_and_pad() {
        assert_eq!(
            number_masks::<3>(1500, Leading::Ghost, Separator::Plain),
            ([0x6F; 3], 999)
        );
        assert_eq!(
            number_masks::<3>(88, Leading::Zeros, Separator::Plain).0,
            [digit('0'), 0x7F, 0x7F]
        );
        assert_eq!(
            number_masks::<3>(-5, Leading::Ghost, Separator::Plain).0,
            [0, MINUS, digit('5')]
        );
        assert_eq!(
            number_masks::<3>(-5, Leading::Zeros, Separator::Plain).0,
            [MINUS, digit('0'), digit('5')]
        );
        // -999 clamps to -99 (the minus needs a cell).
        assert_eq!(
            number_masks::<3>(-999, Leading::Ghost, Separator::Plain),
            ([MINUS, digit('9'), digit('9')], -99)
        );
        // One cell cannot show a minus at all: it clamps to 0.
        assert_eq!(
            number_masks::<1>(-7, Leading::Ghost, Separator::Plain),
            ([digit('0')], 0)
        );
        assert_eq!(number_masks::<0>(42, Leading::Ghost, Separator::Plain).1, 0);
    }

    #[test]
    fn decimal_point_keeps_a_digit_before_it() {
        let (m, _) = number_masks::<3>(5, Leading::Ghost, Separator::Decimal { places: 1 });
        assert_eq!(m, [0, digit('0') | POINT, digit('5')]);
        let (m, _) = number_masks::<3>(147, Leading::Ghost, Separator::Decimal { places: 1 });
        assert_eq!(m, [digit('1'), digit('4') | POINT, digit('7')]);
        // Too many places clamp to N - 1.
        let (m, _) = number_masks::<2>(7, Leading::Ghost, Separator::Decimal { places: 5 });
        assert_eq!(m, [digit('0') | POINT, digit('7')]);
        // No room for the minus: dashes.
        let (m, _) = number_masks::<2>(-7, Leading::Ghost, Separator::Decimal { places: 1 });
        assert_eq!(m, [MINUS, MINUS]);
    }

    #[test]
    fn geometry_follows_the_spec() {
        let g = Seg7Geometry::new(200.0);
        assert!((g.stroke - 21.0).abs() < 1e-4, "stroke 10.5%: {}", g.stroke);
        assert!((g.gap - 21.0 * 0.18).abs() < 1e-4, "gap 18% of stroke: {}", g.gap);
        // Unslanted, the chamfers of a (top) and b (upper right) sit exactly `gap` apart.
        let flat = Seg7Geometry { slant: 0.0, ..g };
        let a = flat.segment(Vec2::ZERO, 0);
        let b = flat.segment(Vec2::ZERO, 1);
        // a's right tip and b's top tip lie on parallel 45° chamfer lines x + y = c.
        let ca = a[3].x + a[3].y;
        let cb = b[0].x + b[0].y;
        let dist = (cb - ca).abs() / std::f32::consts::SQRT_2;
        assert!((dist - flat.gap).abs() < 1e-3, "chamfer gap {dist} vs {}", flat.gap);
        // Segment a's thickness is the stroke and its top edge is the digit top.
        let top = a.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let bottom = a.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        assert!((bottom - top - g.stroke).abs() < 1e-3);
        assert!((top + g.height * 0.5).abs() < 1e-3);
    }

    #[test]
    fn slant_leans_the_top_right_and_stays_in_bounds() {
        let g = Seg7Geometry::new(120.0);
        let center = vec2(300.0, -50.0);
        let b = g.segment(center, 1);
        let c = g.segment(center, 2);
        let top_b = b[0];
        let bottom_c = c[3];
        let expected = g.height * 0.5 * deg(SLANT_DEG).tan();
        assert!(top_b.x > bottom_c.x, "b's top tip is right of c's bottom tip");
        assert!(
            ((top_b.x - bottom_c.x) - 2.0 * (expected - (g.stroke * 0.5 + g.tip_pull()) * deg(8.0).tan())).abs() < 1e-2
        );
        let bounds = g.bounds(center);
        for i in 0..7 {
            for pt in g.segment(center, i) {
                assert!(
                    bounds.expand(1e-3).contains(pt),
                    "segment {i} point {pt:?} outside {bounds:?}"
                );
            }
        }
        for pt in g.point(center) {
            assert!(bounds.expand(1e-3).contains(pt));
        }
        for pt in g.diagonal(center) {
            assert!(bounds.expand(1e-3).contains(pt), "diagonal {pt:?} outside {bounds:?}");
        }
    }

    /// True if `p` lies strictly inside the convex polygon `poly` (either winding).
    fn inside_convex(poly: &[Vec2], p: Vec2) -> bool {
        let sides = poly
            .iter()
            .zip(poly.iter().cycle().skip(1))
            .map(|(a, b)| (*b - *a).cross(p - *a));
        let (mut pos, mut neg) = (false, false);
        for s in sides {
            pos |= s > 0.0;
            neg |= s < 0.0;
        }
        !(pos && neg)
    }

    #[test]
    fn core_sits_inside_its_element() {
        let g = Seg7Geometry::new(150.0);
        let center = vec2(40.0, -20.0);
        for i in (0..7).chain([DIAGONAL_INDEX]) {
            let outer = g.element(center, i);
            for p in g.element_core(center, i) {
                assert!(inside_convex(&outer, p), "element {i}: core point {p:?}");
            }
        }
    }

    #[test]
    fn diagonal_runs_corner_to_corner_through_the_middle() {
        let g = Seg7Geometry {
            slant: 0.0,
            ..Seg7Geometry::new(200.0)
        };
        let d = g.diagonal(Vec2::ZERO);
        let (top, bottom) = (d[0], d[3]);
        assert!(top.x < 0.0 && top.y < 0.0, "starts top-left: {top:?}");
        assert!(bottom.x > 0.0 && bottom.y > 0.0, "ends bottom-right: {bottom:?}");
        assert!(inside_convex(&d, Vec2::ZERO), "crosses the cell middle");
        // Its stroke matches the segments'.
        let across = (d[1] - d[5]).length();
        assert!((across - g.stroke).abs() < 1e-3, "diagonal stroke {across}");
    }

    #[test]
    fn gear_masks_cover_every_gear() {
        assert_eq!(gear_mask(Gear::Reverse, Segments::Seven), u16::from(digit('r')));
        assert_eq!(gear_mask(Gear::Neutral, Segments::Seven), u16::from(digit('N')));
        assert_eq!(gear_mask(Gear::Neutral, Segments::Diagonal), NEUTRAL_DIAGONAL);
        for n in 1..=9u8 {
            let ch = char::from(b'0' + n);
            assert_eq!(gear_mask(Gear::Forward(n), Segments::Diagonal), u16::from(digit(ch)));
        }
        assert_eq!(
            gear_mask(Gear::Forward(10), Segments::Seven),
            u16::from(digit('0') | POINT),
            "10th gear reads 0."
        );
        assert_eq!(gear_mask(Gear::Forward(0), Segments::Seven), u16::from(MINUS));
    }

    #[test]
    fn diagonal_lights_only_on_a_diagonal_glass() {
        let t = Tokens::default();
        let mut plain = Seg7Digit::new(&t, Reveal::Instant);
        plain.update(NEUTRAL_DIAGONAL, DT);
        assert_eq!(
            plain.mask(),
            NEUTRAL_DIAGONAL & !DIAGONAL,
            "a seven-segment cell drops it"
        );
        assert_eq!(plain.level(DIAGONAL_INDEX), 0.0);
        let mut bay = Seg7Digit::new(&t, Reveal::Sequence).with_segments(Segments::Diagonal);
        assert_eq!(bay.segments(), Segments::Diagonal);
        bay.update_gear(Gear::Neutral, DT);
        assert_eq!(bay.mask(), NEUTRAL_DIAGONAL);
        assert_eq!(bay.level(1), 1.0, "b strikes first");
        assert_eq!(bay.level(DIAGONAL_INDEX), 0.0, "the diagonal strikes last");
        for _ in 0..6 {
            bay.update_gear(Gear::Neutral, DT);
        }
        assert_eq!(bay.level(DIAGONAL_INDEX), 1.0);
        assert!(!bay.animating());
    }

    #[test]
    fn instant_digits_switch_on_the_same_frame() {
        let t = Tokens::default();
        let mut d = Seg7Digit::new(&t, Reveal::Instant);
        d.update_char('8', DT);
        assert!((0..7).all(|i| d.level(i) == 1.0));
        assert!(!d.animating());
        d.update_char('1', DT);
        assert_eq!(d.level(0), 0.0, "segment a off at once");
        assert_eq!(d.level(1), 1.0);
        assert!(!d.animating());
    }

    #[test]
    fn sequence_lights_segments_in_order_over_80_ms() {
        let t = Tokens::default();
        let mut d = Seg7Digit::new(&t, Reveal::Sequence);
        d.update_char('8', DT);
        assert_eq!(d.level(0), 1.0, "segment a lights at once");
        assert_eq!(d.level(6), 0.0, "segment g lights last");
        assert!(d.animating());
        let mut elapsed = 0.0;
        while d.animating() && elapsed < 1.0 {
            d.update_char('8', DT);
            elapsed += DT;
        }
        assert!((0..7).all(|i| d.level(i) == 1.0));
        assert!(elapsed <= SEQUENCE_SECONDS + DT + 1e-4, "sequence took {elapsed} s");
        assert!(elapsed >= SEQUENCE_SECONDS - DT, "sequence took {elapsed} s");
    }

    #[test]
    fn sequence_fades_old_segments_within_micro() {
        let t = Tokens::default();
        let mut d = Seg7Digit::new(&t, Reveal::Sequence);
        for _ in 0..12 {
            d.update_char('8', DT);
        }
        d.update_char('1', DT);
        let a = d.level(0);
        assert!(a > 0.0 && a < 1.0, "segment a fades: {a}");
        for _ in 0..3 {
            d.update_char('1', DT);
        }
        assert_eq!(d.level(0), 0.0, "gone within durations.micro");
        assert_eq!(d.level(1), 1.0, "the new digit's first segment re-strikes at once");
        let c = d.level(2);
        assert!(c < 1.0, "the shared segment c waits for its turn: {c}");
        for _ in 0..3 {
            d.update_char('1', DT);
        }
        assert_eq!(d.level(2), 1.0, "and is lit by the end of the sequence");
        assert!(!d.animating());
    }

    #[test]
    fn number_animates_until_settled() {
        let t = Tokens::default();
        let mut n = Seg7Number::<4>::new(&t, Reveal::Sequence);
        n.update(7450, DT);
        assert_eq!(n.value(), 7450);
        assert!(n.animating());
        for _ in 0..10 {
            n.update(7450, DT);
        }
        assert!(!n.animating());
        assert_eq!(n.cells()[0].mask(), u16::from(digit('7')));
    }

    #[test]
    fn block_width_is_fixed_and_cells_are_centred() {
        let t = Tokens::default();
        let g = Seg7Geometry::new(120.0);
        let mut n = Seg7Number::<3>::new(&t, Reveal::Instant);
        let w0 = n.width(&g);
        n.update(999, DT);
        assert_eq!(n.width(&g), w0, "width never changes with the value");
        let first = n.cell_center(Vec2::ZERO, &g, 0).x;
        let last = n.cell_center(Vec2::ZERO, &g, 2).x;
        assert!((first + last).abs() < 1e-3, "symmetric about the centre");
        assert!((last - first - 2.0 * g.advance()).abs() < 1e-3);
        let colon = Seg7Number::<4>::new(&t, Reveal::Instant).with_separator(Separator::Colon { after: 1 });
        assert!((colon.width(&g) - block_width(4, &g, Separator::Plain) - g.colon_extra()).abs() < 1e-3);
    }

    #[test]
    fn colours_come_from_the_palette() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "sciFi");
        assert_eq!(Phosphor::Primary.color(&pal), pal.primary_text);
        assert_eq!(Phosphor::Ink.color(&pal), pal.roles.ink);
        let inks = Inks::new(Phosphor::Primary, Phosphor::Primary, &pal, 1.0);
        // The ghost is opaque and each channel shows 7% of the phosphor's displayed value.
        assert_eq!(inks.ghost.a, 1.0);
        let lit = pal.primary_text.to_srgb8();
        let ghost = inks.ghost.to_srgb8();
        for ch in 0..3 {
            let want = GHOST_LEVEL * f32::from(lit[ch]);
            assert!(
                (f32::from(ghost[ch]) - want).abs() <= 1.0,
                "channel {ch}: ghost {ghost:?} vs 7% of {lit:?}"
            );
        }
        // The ghost keeps the phosphor's hue: the brightest channel stays the brightest.
        let max_ch = |c: [u8; 4]| (0..3).max_by_key(|&i| c[i]).unwrap_or(0);
        assert_eq!(max_ch(ghost), max_ch(lit));
        // The core is brighter than the phosphor and not pure white.
        assert!(inks.core.luminance() > pal.primary_text.luminance());
        assert!(inks.core.luminance() < pal.roles.ink.luminance());
        // A non-finite colour cannot poison the ghost.
        let bad = ghost_of(Color {
            r: f32::NAN,
            g: 0.5,
            b: f32::INFINITY,
            a: 1.0,
        });
        assert!(bad.r.is_finite() && bad.g.is_finite() && bad.b.is_finite());
    }

    #[test]
    fn ghost_phosphor_is_the_glass_and_survives_a_lit_switch() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "sciFi");
        let mut gear = Seg7Digit::new(&t, Reveal::Sequence)
            .with_phosphor(Phosphor::Ink)
            .with_ghost(Phosphor::Primary);
        assert_eq!(gear.ghost(), Phosphor::Primary);
        gear.set_phosphor(Phosphor::Warning);
        assert_eq!(gear.phosphor(), Phosphor::Warning);
        assert_eq!(gear.ghost(), Phosphor::Primary, "reverse keeps the panel's ghost");
        let boost = Seg7Number::<3>::new(&t, Reveal::Instant).with_phosphor(Phosphor::Secondary);
        assert_eq!(boost.phosphor(), Phosphor::Secondary);
        let inks = Inks::new(Phosphor::Ink, Phosphor::Primary, &pal, 1.0);
        let (r, g, _) = inks.ghost.straight_rgb();
        assert!(g > r, "a white gear's ghost is the cyan panel phosphor");
    }

    #[test]
    fn brightness_dims_lit_segments_only() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "sciFi");
        let mut d = Seg7Digit::new(&t, Reveal::Instant);
        assert_eq!(d.brightness(), 1.0);
        d.set_brightness(0.72);
        assert!((d.brightness() - 0.72).abs() < 1e-6);
        d.set_brightness(3.0);
        assert_eq!(d.brightness(), 1.0);
        d.set_brightness(f32::NAN);
        assert_eq!(d.brightness(), 1.0);
        let full = Inks::new(Phosphor::Ink, Phosphor::Primary, &pal, 1.0);
        let dim = Inks::new(Phosphor::Ink, Phosphor::Primary, &pal, 0.72);
        // Light scales, coverage stays full (overlaps stay seamless).
        assert!((dim.lit.g - 0.72 * full.lit.g).abs() < 1e-5);
        assert!((dim.core.r - 0.72 * full.core.r).abs() < 1e-5);
        assert_eq!(dim.lit.a, full.lit.a);
        assert_eq!(dim.core.a, full.core.a);
        assert_eq!(dim.ghost, full.ghost, "ghosts never dim");
    }

    #[test]
    fn glow_scales_with_the_digit_and_bloom_thins_on_large_digits() {
        let speed = Seg7Geometry::new(120.0);
        let gear = Seg7Geometry::new(240.0);
        // The halo keeps its size relative to the stroke.
        assert!((gear.glow_scale() / speed.glow_scale() - 2.0).abs() < 1e-5);
        assert!(
            (gear.glow_scale() - 1.0).abs() < 1e-6,
            "tokens unscaled at the reference"
        );
        // At the speed cap the phosphor halo (σ 10 -> 5) stays well inside the counter.
        let counter = speed.width - 2.0 * speed.stroke;
        assert!(
            10.0 * speed.glow_scale() * 3.0 < counter,
            "3σ inside the {counter} px counter"
        );
        assert_eq!(Seg7Geometry::new(10.0).glow_scale(), GLOW_SCALE_RANGE.0);
        assert_eq!(Seg7Geometry::new(5000.0).glow_scale(), GLOW_SCALE_RANGE.1);
        assert_eq!(speed.bloom_scale(), 1.0);
        assert!((Seg7Geometry::new(300.0).bloom_scale() - 0.4).abs() < 1e-6);
    }

    #[test]
    fn halo_modes_draw_the_bloom_modes_lobes() {
        let t = Tokens::default();
        let geom = Seg7Geometry::new(128.0);
        let lobes = |e| glow_lobes(&t, &geom, e);
        assert_eq!(lobes(Emission::Halo), lobes(Emission::Bloom));
        assert_eq!(lobes(Emission::SoftHalo), lobes(Emission::Soft));
        assert!(lobes(Emission::Halo).is_some() && lobes(Emission::SoftHalo).is_some());
        assert_eq!(lobes(Emission::Matte), None);
    }

    #[test]
    fn scanlines_snap_to_device_pixels() {
        assert_eq!(scanline_pitch(1.0), (4.0, 2.0), "4K: 2 px lines every 4 px");
        assert_eq!(scanline_pitch(0.5), (2.0, 1.0), "1080p: 1 px lines every 2 px");
        let (period, line) = scanline_pitch(0.1);
        assert!(period >= 2.0 && line >= 1.0 && line < period);
        let (period, line) = scanline_pitch(f32::NAN);
        assert!(period >= 2.0 && line >= 1.0 && line < period);
    }
}
