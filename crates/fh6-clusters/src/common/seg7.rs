//! Seg7Digit: slanted VFD seven-segment digits drawn as SDF segments (COMPONENTS.md §15,
//! `docs/design/families/sci-fi.md`).
//!
//! Every segment is a chamfered (pointed hexagon) polygon evaluated by the kit as an exact
//! signed distance, so it stays crisp at any scale and glows from the same shape. The geometry
//! follows the spec: an 8° slant, a stroke of 10.5% of the digit height and gaps of 18% of the
//! stroke between segment tips. Unlit segments are always drawn at 7% of the phosphor (the ghost
//! `8`); lit segments are the phosphor fill with the tokens' `glow.phosphor` lobes in a bloom
//! layer and a hot core mixed 45% toward the white ink.
//!
//! - [`Seg7Geometry`]: cell size, stroke, gap, slant and the segment polygons.
//! - [`Seg7Digit`]: one animated cell (instant for live numbers, an 80 ms sequence for a gear).
//! - [`Seg7Number`]: `N` fixed cells, right-aligned, ghost 8s in the leading cells, an optional
//!   decimal point or colon; digits come from arithmetic, nothing allocates.
//! - [`draw_speed`]: the `segment` variant of TabularSpeed (COMPONENTS.md §2), 3 cells.
//! - [`draw_scanlines`]: the VFD cavity's 2 px scanlines, clipped to a rectangle.
use fh6_kit::color::{linear_to_srgb, srgb_to_linear};
use fh6_kit::shape::{seven_seg_mask, SEVEN_SEG_DIGITS};
use fh6_kit::tokens::GlowLevel;
use fh6_kit::{
    deg, vec2, Affine, Color, Glow, Gradient, LayerOptions, Painter, Palette, Rect, Shape, Spread, Tokens, Vec2,
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
/// 7%, the hero frames and tokens `vfdGhostUnlit`).
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

/// Width of the hot core as a fraction of the stroke.
const CORE_WIDTH: f32 = 0.4;
/// Soft edge of the hot core as a fraction of the stroke.
const CORE_SOFTNESS: f32 = 0.1;
/// Digit height at which the `glow.phosphor` lobes apply unscaled (sci-fi.md: the ghost-888
/// speed has a cap of 120); larger digits get a proportionally wider halo.
const GLOW_REFERENCE_HEIGHT: f32 = 120.0;
/// Extra space a colon adds between its two cells, as a fraction of the stroke.
const COLON_EXTRA: f32 = 1.4;
/// Vertical offset of the colon dots from the digit middle, as a fraction of the height.
const COLON_DOT_V: f32 = 0.2;
/// Number of drawable elements in a cell: segments a–g and the decimal point.
const ELEMENTS: usize = 8;

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

/// Whether lit segments write to the emissive (bloom) buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Emission {
    /// Glance ranks 1–3 (gear, speed, tach readouts): the `glow.phosphor` lobes in a bloom layer.
    #[default]
    Bloom,
    /// Ranks 4–5 (mini readouts, data tiles): lit segments with their core, no glow, no bloom.
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
        let half = self.stroke * 0.5;
        self.hex(center, index, half, self.tip_pull())
    }

    /// The hot core of segment `index`: the same hexagon inset to [`CORE_WIDTH`] of the stroke.
    fn segment_core(&self, center: Vec2, index: usize) -> [Vec2; 6] {
        let half = self.stroke * 0.5;
        let core = half * CORE_WIDTH;
        // A parallel chamfer inset by (half - core) moves the tip by that distance times sqrt(2).
        self.hex(
            center,
            index,
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

    /// Glow sigma multiplier: the token lobes at the reference height, proportional above it.
    fn glow_scale(&self) -> f32 {
        (self.height / GLOW_REFERENCE_HEIGHT).clamp(0.5, 2.5)
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

    /// The centre line of segment `index` in cell coordinates (corner to corner).
    fn axis(&self, index: usize) -> (Vec2, Vec2) {
        let half = self.stroke * 0.5;
        let x = self.width * 0.5 - half;
        let y = self.height * 0.5 - half;
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

    fn hex(&self, center: Vec2, index: usize, half: f32, pull: f32) -> [Vec2; 6] {
        let (a, b) = self.axis(index);
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
/// [`fh6_kit::shape::seven_seg_mask`]) plus [`POINT`] for the decimal point. The point's ghost is
/// not drawn by a lone digit; [`Seg7Number`] draws it on the cell that carries the point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seg7Digit {
    mask: u8,
    rising: u8,
    clock: f32,
    levels: [f32; ELEMENTS],
    reveal: Reveal,
    fade_seconds: f32,
    phosphor: Phosphor,
}

impl Seg7Digit {
    /// A blank digit (ghost `8` only) in the primary phosphor; old segments of a
    /// [`Reveal::Sequence`] digit fade over `tokens.durations.micro`.
    #[must_use]
    pub fn new(tokens: &Tokens, reveal: Reveal) -> Self {
        let fade = tokens.durations.micro;
        Self {
            mask: 0,
            rising: 0,
            clock: 0.0,
            levels: [0.0; ELEMENTS],
            reveal,
            fade_seconds: if fade.is_finite() && fade > 0.0 { fade } else { 0.05 },
            phosphor: Phosphor::Primary,
        }
    }

    /// The same digit in another phosphor.
    #[must_use]
    pub fn with_phosphor(mut self, phosphor: Phosphor) -> Self {
        self.phosphor = phosphor;
        self
    }

    /// Switches the phosphor (the gear bay turns [`Phosphor::Accent`] on the limiter).
    pub fn set_phosphor(&mut self, phosphor: Phosphor) {
        self.phosphor = phosphor;
    }

    /// The phosphor in use.
    #[must_use]
    pub fn phosphor(&self) -> Phosphor {
        self.phosphor
    }

    /// Advances to `mask` (segments a–g in bits 0–6, [`POINT`] in bit 7) by `dt` seconds.
    pub fn update(&mut self, mask: u8, dt: f32) {
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
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
            let bit = 1u8 << i;
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
        self.update(seven_seg_mask(ch), dt);
    }

    /// The target mask.
    #[must_use]
    pub fn mask(&self) -> u8 {
        self.mask
    }

    /// The phosphor level (0..=1) of element `index` (0..7 = a..g, 7 = the point).
    #[must_use]
    pub fn level(&self, index: usize) -> f32 {
        self.levels.get(index).copied().unwrap_or(0.0)
    }

    /// True while a reveal sequence or a fade still runs without new input.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.levels.iter().enumerate().any(|(i, &level)| {
            if self.mask & (1u8 << i) != 0 {
                level < 1.0
            } else {
                level > 0.0
            }
        })
    }

    /// Draws the digit centred at `center`: ghost segments, then the lit phosphor.
    pub fn draw(&self, p: &mut Painter<'_>, center: Vec2, geom: &Seg7Geometry, palette: &Palette, emission: Emission) {
        let inks = Inks::new(self.phosphor.color(palette), palette);
        ghost_cell(p, geom, center, &self.levels, inks.ghost, false);
        emit(p, emission, |p| {
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

    /// The same readout in another phosphor.
    #[must_use]
    pub fn with_phosphor(mut self, phosphor: Phosphor) -> Self {
        self.phosphor = phosphor;
        self
    }

    /// Switches the phosphor.
    pub fn set_phosphor(&mut self, phosphor: Phosphor) {
        self.phosphor = phosphor;
    }

    /// Advances to `value` (clamped to what `N` cells can show) by `dt` seconds.
    pub fn update(&mut self, value: i64, dt: f32) {
        let (masks, shown) = number_masks::<N>(value, self.leading, self.separator);
        self.value = shown;
        for (cell, mask) in self.cells.iter_mut().zip(masks) {
            cell.update(mask, dt);
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
        let inks = Inks::new(self.phosphor.color(palette), palette);
        let point_cell = self.separator.point_cell(N);
        for (i, cell) in self.cells.iter().enumerate() {
            let c = self.cell_center(center, geom, i);
            ghost_cell(p, geom, c, &cell.levels, inks.ghost, point_cell == Some(i));
        }
        let colon = self.separator.colon_after(N).map(|after| {
            let left = self.cell_center(center, geom, after - 1);
            vec2(
                left.x + (geom.width + geom.spacing + geom.colon_extra()) * 0.5,
                center.y,
            )
        });
        emit(p, emission, |p| {
            for (i, cell) in self.cells.iter().enumerate() {
                let c = self.cell_center(center, geom, i);
                lit_cell(p, geom, c, &cell.levels, &inks, emission);
            }
            if let Some(c) = colon {
                let lobes = glow_lobes(p, geom, emission);
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
    let inks = Inks::new(Phosphor::Primary.color(palette), palette);
    let levels = masks.map(mask_levels);
    for (i, cell) in levels.iter().enumerate() {
        let c = vec2(center.x + cell_offset(3, &geom, Separator::Plain, i), center.y);
        ghost_cell(p, &geom, c, cell, inks.ghost, false);
    }
    emit(p, Emission::Bloom, |p| {
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
        if mask & (1u8 << i) != 0 {
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

/// The three colours of one phosphor.
#[derive(Debug, Clone, Copy)]
struct Inks {
    lit: Color,
    core: Color,
    ghost: Color,
}

impl Inks {
    fn new(phosphor: Color, palette: &Palette) -> Self {
        Self {
            lit: phosphor,
            core: phosphor.lerp(palette.roles.ink, CORE_WHITE),
            ghost: ghost_of(phosphor),
        }
    }
}

/// The ghost colour: [`GHOST_LEVEL`] of the phosphor's displayed (sRGB) brightness over black.
/// The kit blends in linear light, so the coverage is converted from display space.
fn ghost_of(phosphor: Color) -> Color {
    let (r, g, b) = phosphor.straight_rgb();
    let peak = r.max(g).max(b);
    if peak.is_nan() || peak <= 1e-6 {
        return phosphor.with_alpha(0.0);
    }
    let shown = srgb_to_linear(GHOST_LEVEL * linear_to_srgb(peak.min(1.0)));
    phosphor.with_alpha((shown / peak).clamp(0.0, 1.0))
}

/// Runs `f` in a bloom layer for [`Emission::Bloom`], in the current layer otherwise.
fn emit<'f>(p: &mut Painter<'f>, emission: Emission, f: impl FnOnce(&mut Painter<'f>)) {
    match emission {
        Emission::Bloom => p.layer(LayerOptions::bloom(), f),
        Emission::Matte => f(p),
    }
}

/// The phosphor glow lobes and their scale, or `None` for [`Emission::Matte`].
fn glow_lobes(p: &Painter<'_>, geom: &Seg7Geometry, emission: Emission) -> Option<(GlowLevel, f32)> {
    match emission {
        Emission::Bloom => Some((p.tokens().glow.phosphor, geom.glow_scale())),
        Emission::Matte => None,
    }
}

/// Ghost segments (and the point's ghost when `point`) that are not fully lit.
fn ghost_cell(
    p: &mut Painter<'_>,
    geom: &Seg7Geometry,
    center: Vec2,
    levels: &[f32; ELEMENTS],
    ghost: Color,
    point: bool,
) {
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
    if point && levels[7] < 1.0 {
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
    let lobes = glow_lobes(p, geom, emission);
    let softness = geom.stroke * CORE_SOFTNESS;
    for (i, &level) in levels.iter().take(7).enumerate() {
        if level > 0.0 {
            let outer = geom.segment(center, i);
            let core = geom.segment_core(center, i);
            light(p, &outer, &core, level, inks, lobes, softness);
        }
    }
    if levels[7] > 0.0 {
        let outer = geom.point(center);
        let core = geom.square(center, geom.point_uv(), geom.stroke * 0.5 * CORE_WIDTH);
        light(p, &outer, &core, levels[7], inks, lobes, softness);
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
    }

    #[test]
    fn core_sits_inside_its_segment() {
        let g = Seg7Geometry {
            slant: 0.0,
            ..Seg7Geometry::new(150.0)
        };
        for i in 0..7 {
            let outer = g.segment(Vec2::ZERO, i);
            let core = g.segment_core(Vec2::ZERO, i);
            let lo = outer.iter().fold(vec2(f32::INFINITY, f32::INFINITY), |m, p| m.min(*p));
            let hi = outer
                .iter()
                .fold(vec2(f32::NEG_INFINITY, f32::NEG_INFINITY), |m, p| m.max(*p));
            for p in core {
                assert!(p.x > lo.x && p.x < hi.x && p.y > lo.y && p.y < hi.y, "segment {i}");
            }
        }
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
        assert_eq!(n.cells()[0].mask(), digit('7'));
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
        let inks = Inks::new(pal.primary, &pal);
        // The ghost shows 7% of the phosphor's displayed brightness over black.
        let (r, g, b) = pal.primary.straight_rgb();
        let peak = r.max(g).max(b);
        let shown = linear_to_srgb(inks.ghost.a * peak);
        assert!(
            (shown - GHOST_LEVEL * linear_to_srgb(peak)).abs() < 1e-3,
            "ghost {shown}"
        );
        // The core is brighter than the phosphor and not pure white.
        assert!(inks.core.luminance() > pal.primary.luminance());
        assert!(inks.core.luminance() < pal.roles.ink.luminance());
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
