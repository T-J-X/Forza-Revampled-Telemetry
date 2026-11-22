//! Text: multi-channel signed distance field (MTSDF) glyph atlas, kerning, tracking and tabular
//! figures.
//!
//! Glyphs are generated with `fdsm` (a pure-Rust msdfgen) when a font loads, for the preload
//! charset, and on demand afterwards (a glyph first used mid-frame appears on the next frame).
//! Kerning pairs are measured once per face with `rustybuzz` (GPOS and `kern`), so laying out a
//! string at draw time is table lookups only: no shaping and no allocation.
//!
//! Sizes are **cap heights** in local units (tokens `type`): a user font replacing the display
//! face keeps the same optical size.
use crate::color::Color;
use crate::math::{finite_or, vec2, Rect, Vec2};
use crate::paint::Paint;
use fdsm::bezier::scanline::FillRule;
use fdsm::bezier::{Point, Segment};
use fdsm::correct_error::{correct_error_mtsdf, ErrorCorrectionConfig};
use fdsm::generate::generate_mtsdf;
use fdsm::render::correct_sign_mtsdf;
use fdsm::shape::{Contour, Shape};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use ttf_parser::{GlyphId, OutlineBuilder, Tag};

/// Atlas pixels per em.
pub const ATLAS_EM: f32 = 64.0;
/// Distance range in atlas pixels (the MSDF "pxRange").
pub const PX_RANGE: f32 = 8.0;
/// Atlas width in texels.
pub const ATLAS_WIDTH: u32 = 2048;
/// Largest atlas height in texels (it starts at 1024 and doubles).
pub const ATLAS_MAX_HEIGHT: u32 = 4096;
/// Tabular digit cell, in em (tokens `type.tabular`: 0.60 em).
pub const TABULAR_CELL_EM: f32 = 0.60;
/// Characters generated when a font loads.
pub const PRELOAD: &str =
    " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~°·–—±×";
/// Characters whose pairs are kerned.
const KERN_SET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,-/:'";

/// The bundled fallback face (Noto Sans Regular, OFL), used until the Phase 6 font pack lands.
static NOTO_SANS: &[u8] = include_bytes!("../../fh6-osd/assets/fonts/NotoSans-Regular.ttf");

/// A loaded font face (a family at one weight and style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontId(pub(crate) u16);

impl FontId {
    /// The embedded fallback face (Noto Sans Regular).
    pub const FALLBACK: FontId = FontId(0);
}

/// Token font roles (tokens `type.roles`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontRole {
    /// Hero gear, speed, primary metrics (motorsport default: Saira Condensed).
    Display,
    /// Labels, units, micro-labels (Titillium Web).
    Text,
    /// Floating light numerals (Saira light).
    Light,
    /// Analog dial numerals (Barlow Semi Condensed).
    Dial,
    /// Wordmarks, the custom name (Michroma).
    Wide,
    /// Sci-Fi labels (Chakra Petch).
    Tech,
    /// Fluid / generative numerals (Exo 2).
    Fluid,
    /// HUD numerics (B612 Mono).
    Aviation,
    /// Race Dash gear and values.
    Race,
}

impl FontRole {
    /// All roles.
    pub const ALL: [FontRole; 9] = [
        FontRole::Display,
        FontRole::Text,
        FontRole::Light,
        FontRole::Dial,
        FontRole::Wide,
        FontRole::Tech,
        FontRole::Fluid,
        FontRole::Aviation,
        FontRole::Race,
    ];

    fn index(self) -> usize {
        self as usize
    }

    /// The role for a tokens key (`display`, `text`, ...).
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            "display" => FontRole::Display,
            "text" => FontRole::Text,
            "light" => FontRole::Light,
            "dial" => FontRole::Dial,
            "wide" => FontRole::Wide,
            "tech" => FontRole::Tech,
            "fluid" => FontRole::Fluid,
            "aviation" => FontRole::Aviation,
            "race" => FontRole::Race,
            _ => return None,
        })
    }
}

/// Horizontal alignment of a text run relative to its anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    /// Anchor at the left edge.
    Left,
    /// Anchor at the centre.
    #[default]
    Center,
    /// Anchor at the right edge.
    Right,
}

/// Vertical alignment of a text run relative to its anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VAlign {
    /// Anchor on the baseline.
    Baseline,
    /// Anchor at the middle of the cap height (centres numerals optically).
    #[default]
    CapMiddle,
    /// Anchor at the top of the capitals.
    CapTop,
}

/// Digit widths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Figures {
    /// The font's own advances (with kerning).
    #[default]
    Proportional,
    /// Fixed 0.60 em cells for `0-9`, so values never shift as they change.
    Tabular,
}

/// How to set a run of text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    /// The face.
    pub font: FontId,
    /// Cap height in local units (tokens scale: gearHero 200, speedHero 112, microLabel 16 ...).
    pub cap: f32,
    /// Fill.
    pub paint: Paint,
    /// Horizontal alignment.
    pub align: Align,
    /// Vertical alignment.
    pub valign: VAlign,
    /// Extra spacing between letters, in em (tokens: micro-labels +0.16).
    pub tracking: f32,
    /// Digit widths.
    pub figures: Figures,
    /// Synthetic weight offset, -1 (thinner) to 1 (bolder). Prefer a real weight face.
    pub weight: f32,
    /// Synthetic slant (horizontal shear), 0 = upright, about 0.2 = italic.
    pub slant: f32,
}

impl TextStyle {
    /// Centred white text of cap height `cap` in `font`.
    #[must_use]
    pub fn new(font: FontId, cap: f32) -> Self {
        Self {
            font,
            cap,
            paint: Paint::Solid(Color::WHITE),
            align: Align::Center,
            valign: VAlign::CapMiddle,
            tracking: 0.0,
            figures: Figures::Proportional,
            weight: 0.0,
            slant: 0.0,
        }
    }

    /// With a paint.
    #[must_use]
    pub fn paint(mut self, paint: impl Into<Paint>) -> Self {
        self.paint = paint.into();
        self
    }

    /// With an alignment.
    #[must_use]
    pub fn align(mut self, align: Align, valign: VAlign) -> Self {
        self.align = align;
        self.valign = valign;
        self
    }

    /// With tabular figures.
    #[must_use]
    pub fn tabular(mut self) -> Self {
        self.figures = Figures::Tabular;
        self
    }

    /// With tracking in em.
    #[must_use]
    pub fn tracking(mut self, em: f32) -> Self {
        self.tracking = em;
        self
    }
}

/// The measured size of a run, in local units.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TextMetrics {
    /// Advance width including tracking.
    pub width: f32,
    /// Cap height.
    pub cap: f32,
    /// Em size.
    pub em: f32,
}

/// What a font is, for lookup.
#[derive(Debug, Clone, PartialEq)]
pub struct FontDesc {
    /// Family name, e.g. "Saira Condensed".
    pub family: String,
    /// CSS weight, 100..=900.
    pub weight: u16,
    /// Italic face.
    pub italic: bool,
}

/// Errors loading fonts.
#[derive(Debug, thiserror::Error)]
pub enum FontError {
    /// The bytes are not a font ttf-parser can read.
    #[error("not a readable font: {0}")]
    Parse(String),
    /// Too many faces are loaded.
    #[error("too many font faces (limit {0})")]
    TooMany(usize),
    /// Reading a font file failed.
    #[error("reading {path}: {source}")]
    Io {
        /// The file.
        path: String,
        /// The cause.
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, Copy)]
struct Glyph {
    /// Texel rectangle in the atlas `[x0, y0, x1, y1]`.
    texels: [u32; 4],
    /// Plane rectangle relative to the pen, in em, y down: `[left, top, right, bottom]`.
    plane: [f32; 4],
    /// Advance in em.
    advance: f32,
    /// Glyph id (for kerning).
    gid: u16,
    /// Whether the glyph has an outline (space has none).
    visible: bool,
}

#[derive(Debug)]
struct Face {
    desc: FontDesc,
    data: Cow<'static, [u8]>,
    index: u32,
    /// `wght` value for variable fonts.
    wght: Option<f32>,
    units_per_em: f32,
    /// Cap height / em, clamped to 0.60..=0.78 (tokens: user-font rule).
    cap_ratio: f32,
    glyphs: HashMap<char, Glyph>,
    kerning: HashMap<(u16, u16), f32>,
}

/// One glyph quad, ready for the painter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GlyphQuad {
    /// Quad in local units.
    pub quad: Rect,
    /// Normalised atlas UV `[u0, v0, u1, v1]`.
    pub uv: [f32; 4],
    /// Local units per atlas texel.
    pub local_per_texel: f32,
    /// Distance range in atlas texels.
    pub px_range: f32,
}

/// A prepared text run: font chosen, width measured.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Run {
    face: u16,
    em: f32,
    width: f32,
    cap: f32,
    align: Align,
    valign: VAlign,
}

impl Run {
    /// The pen origin (left end of the baseline) for an anchor at `pos`.
    pub fn origin(&self, pos: Vec2) -> Vec2 {
        let x = match self.align {
            Align::Left => pos.x,
            Align::Center => pos.x - self.width * 0.5,
            Align::Right => pos.x - self.width,
        };
        let y = match self.valign {
            VAlign::Baseline => pos.y,
            VAlign::CapMiddle => pos.y + self.cap * 0.5,
            VAlign::CapTop => pos.y + self.cap,
        };
        vec2(x, y)
    }

    /// Cap box of the run whose pen origin is `origin`.
    pub fn bounds(&self, origin: Vec2) -> Rect {
        Rect::from_points(
            vec2(origin.x, origin.y - self.cap),
            vec2(origin.x + self.width, origin.y),
        )
    }

    /// Index of the primary face.
    pub fn font_index(&self) -> u16 {
        self.face
    }
}

/// A rectangle of atlas texels that changed and must be uploaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// The MTSDF atlas: RGBA8, rows of `ATLAS_WIDTH` texels.
#[derive(Debug)]
pub(crate) struct Atlas {
    pub pixels: Vec<u8>,
    pub height: u32,
    shelf_x: u32,
    shelf_y: u32,
    shelf_h: u32,
    pub dirty: Vec<DirtyRect>,
    /// Bumped when the atlas grows (the GPU texture must be recreated).
    pub generation: u32,
}

impl Atlas {
    fn new() -> Self {
        let height = 1024;
        Self {
            pixels: vec![0; (ATLAS_WIDTH * height * 4) as usize],
            height,
            shelf_x: 0,
            shelf_y: 0,
            shelf_h: 0,
            dirty: Vec::with_capacity(256),
            generation: 0,
        }
    }

    /// Reserves a `w x h` cell (1 texel of padding), growing the atlas when needed.
    fn allocate(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        let (pw, ph) = (w + 1, h + 1);
        if pw > ATLAS_WIDTH {
            return None;
        }
        if self.shelf_x + pw > ATLAS_WIDTH {
            self.shelf_y += self.shelf_h;
            self.shelf_x = 0;
            self.shelf_h = 0;
        }
        while self.shelf_y + ph > self.height {
            if self.height >= ATLAS_MAX_HEIGHT {
                return None;
            }
            self.height *= 2;
            self.pixels.resize((ATLAS_WIDTH * self.height * 4) as usize, 0);
            self.generation += 1;
        }
        let at = (self.shelf_x, self.shelf_y);
        self.shelf_x += pw;
        self.shelf_h = self.shelf_h.max(ph);
        Some(at)
    }

    fn blit(&mut self, x: u32, y: u32, w: u32, h: u32, rgba: &[u8]) {
        for row in 0..h {
            let src = (row * w * 4) as usize;
            let dst = (((y + row) * ATLAS_WIDTH + x) * 4) as usize;
            if let (Some(s), Some(d)) = (
                rgba.get(src..src + (w * 4) as usize),
                self.pixels.get_mut(dst..dst + (w * 4) as usize),
            ) {
                d.copy_from_slice(s);
            }
        }
        self.dirty.push(DirtyRect { x, y, w, h });
    }
}

/// Every loaded face, the role table and the shared glyph atlas.
#[derive(Debug)]
pub struct Fonts {
    faces: Vec<Face>,
    roles: [FontId; 9],
    pub(crate) atlas: Atlas,
}

/// Most faces that can be loaded.
pub const MAX_FACES: usize = 64;

impl Fonts {
    /// The embedded Noto Sans fallback, assigned to every role.
    ///
    /// # Errors
    /// [`FontError::Parse`] if the embedded font cannot be read (a build defect).
    pub fn new() -> Result<Self, FontError> {
        let mut fonts = Self {
            faces: Vec::new(),
            roles: [FontId::FALLBACK; 9],
            atlas: Atlas::new(),
        };
        fonts.add_face(
            Cow::Borrowed(NOTO_SANS),
            0,
            None,
            FontDesc {
                family: "Noto Sans".into(),
                weight: 400,
                italic: false,
            },
        )?;
        Ok(fonts)
    }

    /// Loads a font from bytes. For a variable font with a `wght` axis, pass `weight` to pick an
    /// instance (load it several times for several weights).
    ///
    /// # Errors
    /// [`FontError`] when the bytes are not a font or too many faces are loaded.
    pub fn load(&mut self, bytes: Vec<u8>, weight: Option<u16>) -> Result<FontId, FontError> {
        let face = ttf_parser::Face::parse(&bytes, 0).map_err(|e| FontError::Parse(e.to_string()))?;
        let family = face
            .names()
            .into_iter()
            .filter(|n| {
                n.name_id == ttf_parser::name_id::TYPOGRAPHIC_FAMILY || n.name_id == ttf_parser::name_id::FAMILY
            })
            .find_map(|n| n.to_string())
            .unwrap_or_else(|| "Unknown".into());
        let italic = face.is_italic();
        let natural = face.weight().to_number();
        let wght = if face.is_variable() {
            weight.map(f32::from)
        } else {
            None
        };

        self.add_face(
            Cow::Owned(bytes),
            0,
            wght,
            FontDesc {
                family,
                weight: weight.filter(|_| wght.is_some()).unwrap_or(natural),
                italic,
            },
        )
    }

    /// Loads every `.ttf` / `.otf` in `dir` (Phase 6 pack, or the user `fonts/` folder beside the
    /// exe). Unreadable files are skipped with a warning. Returns the faces loaded.
    ///
    /// # Errors
    /// [`FontError::Io`] when the directory cannot be listed.
    pub fn load_dir(&mut self, dir: &Path) -> Result<Vec<FontId>, FontError> {
        let entries = std::fs::read_dir(dir).map_err(|source| FontError::Io {
            path: dir.display().to_string(),
            source,
        })?;
        let mut paths: Vec<_> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("ttf") || e.eq_ignore_ascii_case("otf"))
            })
            .collect();
        paths.sort();
        let mut loaded = Vec::new();
        for path in paths {
            match std::fs::read(&path) {
                Ok(bytes) => match self.load(bytes, None) {
                    Ok(id) => loaded.push(id),
                    Err(err) => tracing::warn!(path = %path.display(), error = %err, "skipping font"),
                },
                Err(err) => tracing::warn!(path = %path.display(), error = %err, "skipping font"),
            }
        }
        Ok(loaded)
    }

    fn add_face(
        &mut self,
        data: Cow<'static, [u8]>,
        index: u32,
        wght: Option<f32>,
        desc: FontDesc,
    ) -> Result<FontId, FontError> {
        if self.faces.len() >= MAX_FACES {
            return Err(FontError::TooMany(MAX_FACES));
        }
        let mut face = ttf_parser::Face::parse(&data, index).map_err(|e| FontError::Parse(e.to_string()))?;
        if let Some(w) = wght {
            face.set_variation(Tag::from_bytes(b"wght"), w);
        }
        let upm = face.units_per_em().max(1) as f32;
        let cap = face
            .capital_height()
            .filter(|c| *c > 0)
            .map(f32::from)
            .or_else(|| {
                face.glyph_index('H')
                    .and_then(|g| face.glyph_bounding_box(g))
                    .map(|b| f32::from(b.y_max))
            })
            .unwrap_or(upm * 0.7);
        let cap_ratio = (cap / upm).clamp(0.60, 0.78);
        let kerning = measure_kerning(&data, index, wght);

        let id = FontId(self.faces.len() as u16);
        self.faces.push(Face {
            desc,
            data,
            index,
            wght,
            units_per_em: upm,
            cap_ratio,
            glyphs: HashMap::with_capacity(160),
            kerning,
        });
        self.generate(id.0, PRELOAD.chars());
        Ok(id)
    }

    /// The face assigned to `role`.
    #[must_use]
    pub fn role(&self, role: FontRole) -> FontId {
        self.roles[role.index()]
    }

    /// Assigns a face to a role (bind from tokens, or a user font choice).
    pub fn set_role(&mut self, role: FontRole, font: FontId) {
        if usize::from(font.0) < self.faces.len() {
            self.roles[role.index()] = font;
        }
    }

    /// Assigns each token role (`type.fonts`) to its family when that family is loaded, at the
    /// role's working weight. Roles whose family is missing keep their current face (the
    /// embedded Noto until the Phase 6 pack is installed). Returns how many roles were bound.
    pub fn bind_tokens(&mut self, fonts: &[crate::tokens::FontToken]) -> usize {
        let mut bound = 0;
        for token in fonts {
            let Some(role) = FontRole::from_key(&token.role) else {
                continue;
            };
            let weight = match role {
                FontRole::Display | FontRole::Race => 600,
                FontRole::Light => 300,
                FontRole::Dial | FontRole::Tech | FontRole::Fluid => 500,
                FontRole::Text | FontRole::Wide | FontRole::Aviation => 400,
            };
            if let Some(id) = self.find(&token.family, weight, false) {
                self.set_role(role, id);
                bound += 1;
            }
        }
        bound
    }

    /// The face of `family` nearest to `weight`, preferring the requested style.
    #[must_use]
    pub fn find(&self, family: &str, weight: u16, italic: bool) -> Option<FontId> {
        self.faces
            .iter()
            .enumerate()
            .filter(|(_, f)| f.desc.family.eq_ignore_ascii_case(family))
            .min_by_key(|(_, f)| {
                let style_penalty = if f.desc.italic == italic { 0 } else { 10_000 };
                style_penalty + i32::from(f.desc.weight).abs_diff(i32::from(weight))
            })
            .map(|(i, _)| FontId(i as u16))
    }

    /// The description of a face.
    #[must_use]
    pub fn desc(&self, font: FontId) -> Option<&FontDesc> {
        self.faces.get(usize::from(font.0)).map(|f| &f.desc)
    }

    /// The distinct family names of the loaded faces, in load order (the OSD's font list).
    #[must_use]
    pub fn families(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for f in &self.faces {
            if !out.iter().any(|n| n.eq_ignore_ascii_case(&f.desc.family)) {
                out.push(f.desc.family.clone());
            }
        }
        out
    }

    /// Number of loaded faces.
    #[must_use]
    pub fn len(&self) -> usize {
        self.faces.len()
    }

    /// True when no face is loaded (never, after [`Fonts::new`]).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }

    /// Cap height / em of a face.
    #[must_use]
    pub fn cap_ratio(&self, font: FontId) -> f32 {
        self.faces.get(usize::from(font.0)).map_or(0.7, |f| f.cap_ratio)
    }

    /// Width of one tabular digit cell for `style`, in local units.
    #[must_use]
    pub fn tabular_cell(&self, style: &TextStyle) -> f32 {
        let em = finite_or(style.cap, 0.0).max(0.0) / self.cap_ratio(style.font);
        em * (TABULAR_CELL_EM + style.tracking)
    }

    /// Measures `text` in `style`.
    #[must_use]
    pub fn measure(&self, text: &str, style: &TextStyle) -> TextMetrics {
        self.prepare_run(text, style)
            .map_or_else(TextMetrics::default, |r| TextMetrics {
                width: r.width,
                cap: r.cap,
                em: r.em,
            })
    }

    fn face_for(&self, font: FontId) -> u16 {
        if usize::from(font.0) < self.faces.len() {
            font.0
        } else {
            0
        }
    }

    /// The glyph for `ch` in `face`, falling back to the embedded face.
    fn glyph(&self, face: u16, ch: char) -> Option<(u16, &Glyph)> {
        let f = self.faces.get(usize::from(face))?;
        if let Some(g) = f.glyphs.get(&ch) {
            return Some((face, g));
        }
        if face != 0 {
            if let Some(g) = self.faces.first().and_then(|f| f.glyphs.get(&ch)) {
                return Some((0, g));
            }
        }
        None
    }

    fn advance_of(&self, face: u16, prev: Option<u16>, ch: char, g: &Glyph, em: f32, style: &TextStyle) -> (f32, f32) {
        let tabular = style.figures == Figures::Tabular && ch.is_ascii_digit();
        if tabular {
            let cell = TABULAR_CELL_EM * em;
            let offset = (cell - g.advance * em) * 0.5;
            return (cell, offset);
        }
        let kern = prev
            .and_then(|p| {
                self.faces
                    .get(usize::from(face))
                    .and_then(|f| f.kerning.get(&(p, g.gid)))
            })
            .copied()
            .unwrap_or(0.0);
        (g.advance * em, kern * em)
    }

    pub(crate) fn prepare_run(&self, text: &str, style: &TextStyle) -> Option<Run> {
        let cap = finite_or(style.cap, 0.0);
        if cap <= 0.0 || text.is_empty() {
            return None;
        }
        let face = self.face_for(style.font);
        let em = cap / self.faces.get(usize::from(face))?.cap_ratio;
        let tracking = finite_or(style.tracking, 0.0) * em;
        let mut width = 0.0;
        let mut prev: Option<(u16, u16)> = None;
        let mut n = 0usize;
        for ch in text.chars() {
            let Some((gf, g)) = self.glyph(face, ch) else {
                // Not generated yet: reserve a nominal advance so layout is stable.
                width += 0.5 * em + tracking;
                prev = None;
                n += 1;
                continue;
            };
            let p = prev.filter(|(pf, _)| *pf == gf).map(|(_, gid)| gid);
            let (adv, kern) = self.advance_of(gf, p, ch, g, em, style);
            let kern = if style.figures == Figures::Tabular && ch.is_ascii_digit() {
                0.0
            } else {
                kern
            };
            width += adv + kern + tracking;
            prev = Some((gf, g.gid));
            n += 1;
        }
        if n > 0 {
            width -= tracking;
        }
        Some(Run {
            face,
            em,
            width: width.max(0.0),
            cap,
            align: style.align,
            valign: style.valign,
        })
    }

    /// Emits a quad per visible glyph of `text` (pen origin `origin`), or `Err(ch)` for glyphs
    /// not in the atlas yet.
    pub(crate) fn layout(
        &self,
        text: &str,
        style: &TextStyle,
        run: &Run,
        origin: Vec2,
        mut emit: impl FnMut(Result<GlyphQuad, char>),
    ) {
        let em = run.em;
        let tracking = finite_or(style.tracking, 0.0) * em;
        let local_per_texel = em / ATLAS_EM;
        let (aw, ah) = (ATLAS_WIDTH as f32, self.atlas.height as f32);
        let mut pen = origin.x;
        let mut prev: Option<(u16, u16)> = None;
        for ch in text.chars() {
            let Some((gf, g)) = self.glyph(run.face, ch) else {
                if !ch.is_whitespace() {
                    emit(Err(ch));
                }
                pen += 0.5 * em + tracking;
                prev = None;
                continue;
            };
            let tabular = style.figures == Figures::Tabular && ch.is_ascii_digit();
            let p = prev.filter(|(pf, _)| *pf == gf).map(|(_, gid)| gid);
            let (adv, offset) = self.advance_of(gf, p, ch, g, em, style);
            let x = pen + offset;
            if g.visible {
                let [l, t, r, b] = g.plane;
                let [x0, y0, x1, y1] = g.texels;
                emit(Ok(GlyphQuad {
                    quad: Rect::from_points(vec2(x + l * em, origin.y + t * em), vec2(x + r * em, origin.y + b * em)),
                    uv: [x0 as f32 / aw, y0 as f32 / ah, x1 as f32 / aw, y1 as f32 / ah],
                    local_per_texel,
                    px_range: PX_RANGE,
                }));
            }
            pen = if tabular {
                pen + adv + tracking
            } else {
                x + adv + tracking
            };
            prev = Some((gf, g.gid));
        }
    }

    /// Generates glyphs for `(face, char)` pairs that are missing (called after a frame).
    /// Returns how many were added.
    pub(crate) fn generate_missing(&mut self, missing: &[(u16, char)]) -> usize {
        let mut added = 0;
        for &(face, ch) in missing {
            let target = if self
                .faces
                .get(usize::from(face))
                .and_then(|f| ttf_parser::Face::parse(&f.data, f.index).ok())
                .and_then(|f| f.glyph_index(ch))
                .is_some()
            {
                face
            } else {
                0
            };
            let before = self.faces.get(usize::from(target)).map_or(0, |f| f.glyphs.len());
            self.generate(target, std::iter::once(ch));
            let after = self.faces.get(usize::from(target)).map_or(0, |f| f.glyphs.len());
            if after > before {
                added += 1;
            } else if let Some(f) = self.faces.get_mut(usize::from(target)) {
                // Not in any face: remember an empty glyph so it is not retried every frame.
                f.glyphs.insert(
                    ch,
                    Glyph {
                        texels: [0; 4],
                        plane: [0.0; 4],
                        advance: 0.5,
                        gid: 0,
                        visible: false,
                    },
                );
            }
        }
        added
    }

    /// Rasterises the MTSDF of every char in `chars` for face `face` and packs it.
    fn generate(&mut self, face: u16, chars: impl Iterator<Item = char>) {
        let Some(f) = self.faces.get(usize::from(face)) else {
            return;
        };
        let todo: Vec<char> = chars.filter(|c| !f.glyphs.contains_key(c)).collect();
        if todo.is_empty() {
            return;
        }
        let rendered = render_glyphs(&f.data, f.index, f.wght, f.units_per_em, &todo);
        for r in rendered {
            let mut glyph = Glyph {
                texels: [0; 4],
                plane: r.plane,
                advance: r.advance,
                gid: r.gid,
                visible: r.w > 0 && r.h > 0,
            };
            if glyph.visible {
                match self.atlas.allocate(r.w, r.h) {
                    Some((x, y)) => {
                        self.atlas.blit(x, y, r.w, r.h, &r.rgba);
                        glyph.texels = [x, y, x + r.w, y + r.h];
                    }
                    None => {
                        tracing::warn!("glyph atlas full; {:?} not added", r.ch);
                        continue;
                    }
                }
            }
            if let Some(f) = self.faces.get_mut(usize::from(face)) {
                f.glyphs.insert(r.ch, glyph);
            }
        }
    }
}

struct Rendered {
    ch: char,
    gid: u16,
    w: u32,
    h: u32,
    rgba: Vec<u8>,
    plane: [f32; 4],
    advance: f32,
}

/// Renders glyphs (in parallel on native targets).
fn render_glyphs(data: &[u8], index: u32, wght: Option<f32>, upm: f32, chars: &[char]) -> Vec<Rendered> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).clamp(1, 8);
        if chars.len() > 8 && threads > 1 {
            let chunk = chars.len().div_ceil(threads);
            return std::thread::scope(|s| {
                let handles: Vec<_> = chars
                    .chunks(chunk)
                    .map(|part| s.spawn(move || render_chunk(data, index, wght, upm, part)))
                    .collect();
                handles.into_iter().flat_map(|h| h.join().unwrap_or_default()).collect()
            });
        }
    }
    render_chunk(data, index, wght, upm, chars)
}

fn render_chunk(data: &[u8], index: u32, wght: Option<f32>, upm: f32, chars: &[char]) -> Vec<Rendered> {
    let Ok(mut face) = ttf_parser::Face::parse(data, index) else {
        return Vec::new();
    };
    if let Some(w) = wght {
        face.set_variation(Tag::from_bytes(b"wght"), w);
    }
    chars.iter().filter_map(|&ch| render_one(&face, upm, ch)).collect()
}

fn render_one(face: &ttf_parser::Face<'_>, upm: f32, ch: char) -> Option<Rendered> {
    let gid = face.glyph_index(ch)?;
    let advance = f32::from(face.glyph_hor_advance(gid).unwrap_or(0)) / upm;
    let empty = Rendered {
        ch,
        gid: gid.0,
        w: 0,
        h: 0,
        rgba: Vec::new(),
        plane: [0.0; 4],
        advance,
    };
    let Some(bbox) = face.glyph_bounding_box(gid) else {
        return Some(empty);
    };
    let scale = f64::from(ATLAS_EM / upm);
    let range = f64::from(PX_RANGE);
    let x_min = f64::from(bbox.x_min);
    let y_max = f64::from(bbox.y_max);
    let w = ((f64::from(bbox.x_max) - x_min) * scale + 2.0 * range).ceil().max(1.0) as u32;
    let h = ((y_max - f64::from(bbox.y_min)) * scale + 2.0 * range).ceil().max(1.0) as u32;
    let mut builder = ShapeBuilder {
        shape: Shape::default(),
        contour: Contour::default(),
        start: Point::new(0.0, 0.0),
        last: Point::new(0.0, 0.0),
        map: move |x: f32, y: f32| {
            Point::new(
                (f64::from(x) - x_min) * scale + range,
                (y_max - f64::from(y)) * scale + range,
            )
        },
    };
    face.outline_glyph(gid, &mut builder)?;
    builder.finish_contour();
    let shape = builder.shape;
    if shape.contours.is_empty() {
        return Some(empty);
    }
    let colored = Shape::edge_coloring_simple(shape, 0.03, 0x5EED_F46B);
    let prepared = colored.prepare();
    let mut img = image::Rgba32FImage::new(w, h);
    generate_mtsdf(&prepared, range, &mut img);
    correct_error_mtsdf(&mut img, &colored, &prepared, range, &ErrorCorrectionConfig::default());
    correct_sign_mtsdf(&mut img, &prepared, FillRule::Nonzero);
    let rgba: Vec<u8> = img
        .into_raw()
        .into_iter()
        .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect();
    // Plane bounds in em, y down from the baseline, matching the texel rectangle exactly.
    let em_per_px = 1.0 / f64::from(ATLAS_EM);
    let left = (x_min * scale - range) * em_per_px;
    let top = -(y_max * scale + range) * em_per_px;
    let plane = [
        left as f32,
        top as f32,
        (left + f64::from(w) * em_per_px) as f32,
        (top + f64::from(h) * em_per_px) as f32,
    ];
    Some(Rendered {
        ch,
        gid: gid.0,
        w,
        h,
        rgba,
        plane,
        advance,
    })
}

struct ShapeBuilder<F: Fn(f32, f32) -> Point> {
    shape: Shape<Contour>,
    contour: Contour,
    start: Point,
    last: Point,
    map: F,
}

impl<F: Fn(f32, f32) -> Point> ShapeBuilder<F> {
    fn finish_contour(&mut self) {
        if self.contour.segments.is_empty() {
            return;
        }
        if (self.last - self.start).norm() > 1e-9 {
            self.contour.segments.push(Segment::line(self.last, self.start));
        }
        self.shape.contours.push(std::mem::take(&mut self.contour));
    }
}

impl<F: Fn(f32, f32) -> Point> OutlineBuilder for ShapeBuilder<F> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish_contour();
        let p = (self.map)(x, y);
        self.start = p;
        self.last = p;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = (self.map)(x, y);
        if (p - self.last).norm() > 1e-9 {
            self.contour.segments.push(Segment::line(self.last, p));
        }
        self.last = p;
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let c = (self.map)(x1, y1);
        let p = (self.map)(x, y);
        self.contour.segments.push(Segment::quad(self.last, c, p));
        self.last = p;
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let c1 = (self.map)(x1, y1);
        let c2 = (self.map)(x2, y2);
        let p = (self.map)(x, y);
        self.contour.segments.push(Segment::cubic(self.last, c1, c2, p));
        self.last = p;
    }
    fn close(&mut self) {
        self.finish_contour();
    }
}

/// Measures kerning for every pair of [`KERN_SET`] with rustybuzz: GPOS pair adjustments and
/// the legacy `kern` table. Values are in em; zero pairs are not stored.
fn measure_kerning(data: &[u8], index: u32, wght: Option<f32>) -> HashMap<(u16, u16), f32> {
    let mut out = HashMap::new();
    let Some(mut face) = rustybuzz::Face::from_slice(data, index) else {
        return out;
    };
    if let Some(w) = wght {
        face.set_variations(&[rustybuzz::Variation {
            tag: Tag::from_bytes(b"wght"),
            value: w,
        }]);
    }
    let upm = face.units_per_em().max(1) as f32;
    let chars: Vec<char> = KERN_SET.chars().collect();
    let gids: Vec<Option<GlyphId>> = chars.iter().map(|c| face.glyph_index(*c)).collect();
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    let mut pair = [0u8; 8];
    for (i, a) in chars.iter().enumerate() {
        let Some(ga) = gids[i] else { continue };
        let nominal = f32::from(face.glyph_hor_advance(ga).unwrap_or(0));
        for (j, b) in chars.iter().enumerate() {
            let Some(gb) = gids[j] else { continue };
            let la = a.encode_utf8(&mut pair).len();
            let lb = b.encode_utf8(&mut pair[la..]).len();
            let Ok(text) = std::str::from_utf8(&pair[..la + lb]) else {
                continue;
            };
            buffer.push_str(text);
            let glyphs = rustybuzz::shape(&face, &[], buffer);
            let positions = glyphs.glyph_positions();
            let infos = glyphs.glyph_infos();
            if positions.len() == 2 && infos.first().map(|g| g.glyph_id as u16) == Some(ga.0) {
                let delta = positions[0].x_advance as f32 - nominal + positions[1].x_offset as f32;
                if delta.abs() > 0.5 {
                    out.insert((ga.0, gb.0), delta / upm);
                }
            }
            buffer = glyphs.clear();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fonts() -> Fonts {
        Fonts::new().expect("embedded font loads")
    }

    #[test]
    fn preload_generates_digits_and_letters() {
        let f = fonts();
        for ch in "0123456789AVWa".chars() {
            assert!(f.glyph(0, ch).is_some(), "{ch} generated");
        }
        assert!(f.atlas.dirty.len() > 50, "atlas has uploads pending");
    }

    #[test]
    fn families_lists_each_family_once() {
        let f = fonts();
        let fams = f.families();
        assert!(!fams.is_empty());
        let first = &fams[0];
        assert_eq!(fams.iter().filter(|n| n.eq_ignore_ascii_case(first)).count(), 1);
        assert!(f.find(first, 400, false).is_some());
    }

    #[test]
    fn tabular_digits_have_fixed_width() {
        let f = fonts();
        let style = TextStyle::new(FontId::FALLBACK, 100.0).tabular();
        let w1 = f.measure("111", &style).width;
        let w8 = f.measure("888", &style).width;
        assert!((w1 - w8).abs() < 1e-3, "{w1} vs {w8}");
        let cell = f.tabular_cell(&style);
        assert!((w8 - cell * 3.0).abs() < 1e-3);
        let prop = TextStyle::new(FontId::FALLBACK, 100.0);
        assert!(f.measure("III", &prop).width < f.measure("WWW", &prop).width);
    }

    #[test]
    fn kerning_tightens_av() {
        let f = fonts();
        let style = TextStyle::new(FontId::FALLBACK, 100.0);
        let av = f.measure("AV", &style).width;
        let a = f.measure("A", &style).width;
        let v = f.measure("V", &style).width;
        assert!(av < a + v - 1.0, "AV kerned: {av} vs {}", a + v);
    }

    #[test]
    fn layout_alignment_and_missing_glyphs() {
        let f = fonts();
        let style = TextStyle::new(FontId::FALLBACK, 50.0);
        let run = f.prepare_run("Hi", &style).expect("run");
        let o = run.origin(vec2(100.0, 100.0));
        assert!((o.x + run.width * 0.5 - 100.0).abs() < 1e-3, "centred");
        assert!((o.y - 125.0).abs() < 1e-3, "cap middle");
        let mut quads = 0;
        let mut missing = Vec::new();
        f.layout("Hi死", &style, &run, o, |g| match g {
            Ok(_) => quads += 1,
            Err(c) => missing.push(c),
        });
        assert_eq!(quads, 2);
        assert_eq!(missing, vec!['死']);
    }

    #[test]
    fn missing_glyphs_are_generated_or_marked() {
        let mut f = fonts();
        f.generate_missing(&[(0, 'é'), (0, '死')]);
        assert!(f.glyph(0, 'é').is_some_and(|(_, g)| g.visible));
        assert!(
            f.glyph(0, '死').is_some_and(|(_, g)| !g.visible),
            "unsupported char remembered as blank"
        );
    }

    #[test]
    fn mtsdf_inside_is_bright() {
        let f = fonts();
        let (_, g) = f.glyph(0, 'I').expect("I");
        let [x0, y0, x1, y1] = g.texels;
        let (cx, cy) = ((x0 + x1) / 2, (y0 + y1) / 2);
        let i = ((cy * ATLAS_WIDTH + cx) * 4) as usize;
        let px = &f.atlas.pixels[i..i + 4];
        assert!(px[3] > 128, "true SDF positive inside the stem, got {px:?}");
        let corner = ((y0 * ATLAS_WIDTH + x0) * 4) as usize;
        assert!(f.atlas.pixels[corner + 3] < 128, "outside is negative");
    }
}
