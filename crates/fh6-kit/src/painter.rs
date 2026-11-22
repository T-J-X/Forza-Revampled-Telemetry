//! The immediate-mode [`Painter`]: what a cluster's `draw` calls.
//!
//! Every call appends to the frame's [`Scene`] arena (no heap allocation once warm). Draws that
//! share a blend mode and texture are merged into one instanced GPU draw; a typical cluster is a
//! handful of draw calls whatever its shape count.
use crate::color::Color;
use crate::images::Images;
use crate::math::{finite_or, vec2, Affine, Rect, Vec2};
use crate::mesh::{MeshCmd, MeshDraw};
use crate::paint::{BlendMode, GradientKind, ImageId, ImageWrap, Paint, Refraction, Spread};
use crate::palette::Palette;
use crate::scene::{
    pack_bloom, Bloom, DrawCmd, GpuClip, GpuPaint, Instance, LayerOptions, Scene, Segment, MAX_CLIPS, MAX_INSTANCES,
    MAX_MESHES, MAX_MISSING, MAX_PAINTS, MAX_POINTS, MAX_SEGMENTS,
};
use crate::shape::{encode_simple, flag, kind, mode, Encoded, Glow, Shadow, Shape, Stroke, StrokeAlign, StrokeUnit};
use crate::text::{Fonts, TextMetrics, TextStyle};
use crate::tokens::Tokens;

/// Most nested `save` levels.
const MAX_DEPTH: usize = 32;

/// Paint kinds (must match `shape.wgsl`).
mod paint_kind {
    pub const SOLID: u32 = 0;
    pub const LINEAR: u32 = 1;
    pub const RADIAL: u32 = 2;
    pub const CONIC: u32 = 3;
    pub const IMAGE: u32 = 4;
    pub const GLASS: u32 = 5;
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SegmentKey {
    blur_px: f32,
    backdrop_px: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
struct State {
    xf: Affine,
    opacity: f32,
    blend: BlendMode,
    clip: u32,
    clip_rect: Option<Rect>,
    bloom: Option<Bloom>,
    segment: SegmentKey,
}

/// Records one frame of drawing. Created by [`crate::Kit::begin_frame`].
///
/// Coordinates are **local**: by default 4K reference pixels around the cluster anchor at user
/// scale 1.0 (the host sets the base transform from the [`crate::Layout`]).
#[derive(Debug)]
pub struct Painter<'f> {
    scene: &'f mut Scene,
    fonts: &'f Fonts,
    images: &'f Images,
    palette: &'f Palette,
    tokens: &'f Tokens,
    state: State,
    stack: [Option<State>; MAX_DEPTH],
    depth: usize,
    segment_open: bool,
    last_paint: Option<(Paint, u32)>,
}

impl<'f> Painter<'f> {
    pub(crate) fn new(
        scene: &'f mut Scene,
        fonts: &'f Fonts,
        images: &'f Images,
        palette: &'f Palette,
        tokens: &'f Tokens,
        base: Affine,
    ) -> Self {
        Self {
            scene,
            fonts,
            images,
            palette,
            tokens,
            state: State {
                xf: base,
                opacity: 1.0,
                blend: BlendMode::Normal,
                clip: 0,
                clip_rect: None,
                bloom: None,
                segment: SegmentKey {
                    blur_px: 0.0,
                    backdrop_px: None,
                },
            },
            stack: [None; MAX_DEPTH],
            depth: 0,
            segment_open: false,
            last_paint: None,
        }
    }

    /// The active palette (factory, custom or random), mapped onto token roles.
    #[must_use]
    pub fn palette(&self) -> &Palette {
        self.palette
    }

    /// The design tokens.
    #[must_use]
    pub fn tokens(&self) -> &Tokens {
        self.tokens
    }

    /// The loaded fonts (for measuring text).
    #[must_use]
    pub fn fonts(&self) -> &Fonts {
        self.fonts
    }

    /// The loaded images (for their sizes).
    #[must_use]
    pub fn images(&self) -> &Images {
        self.images
    }

    // ----- state -------------------------------------------------------------------------

    /// Pushes the current transform, opacity, blend mode, clip and layer onto the stack.
    pub fn save(&mut self) {
        if self.depth < MAX_DEPTH {
            self.stack[self.depth] = Some(self.state);
        } else {
            tracing::warn!("painter save stack overflow; restore will be ignored");
        }
        self.depth += 1;
    }

    /// Pops the state pushed by the matching [`Painter::save`].
    pub fn restore(&mut self) {
        if self.depth == 0 {
            return;
        }
        self.depth -= 1;
        if let Some(saved) = self.stack.get(self.depth).copied().flatten() {
            // Leaving a glass layer does not need a new pass: the backdrop was snapshotted when the
            // layer began, so later draws continue in the same segment. A blur change does.
            let blur_changed = saved.segment.blur_px != self.state.segment.blur_px;
            if blur_changed || saved.segment.backdrop_px.is_some() && saved.segment != self.state.segment {
                self.segment_open = false;
            }
            self.state = saved;
        }
    }

    /// Runs `f` between [`Painter::save`] and [`Painter::restore`].
    pub fn scoped<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.save();
        let r = f(self);
        self.restore();
        r
    }

    /// Applies `t` in local space (it runs before the current transform).
    pub fn transform(&mut self, t: &Affine) {
        if t.is_finite() {
            self.state.xf = t.then(&self.state.xf);
        }
    }

    /// Moves the local origin.
    pub fn translate(&mut self, d: Vec2) {
        self.transform(&Affine::translate(d));
    }

    /// Scales local space about the origin.
    pub fn scale(&mut self, s: f32) {
        self.transform(&Affine::scale(s));
    }

    /// Rotates local space about `center`.
    pub fn rotate_about(&mut self, angle: f32, center: Vec2) {
        self.transform(&Affine::rotate_about(angle, center));
    }

    /// Replaces the local-to-target transform (e.g. `Affine::IDENTITY` to draw in target
    /// pixels for a HUD readout). Restored by the matching [`Painter::restore`].
    pub fn set_transform(&mut self, xf: Affine) {
        if xf.is_finite() {
            self.state.xf = xf;
        }
    }

    /// The current local-to-target transform.
    #[must_use]
    pub fn current_transform(&self) -> Affine {
        self.state.xf
    }

    /// Local units per device pixel at the current transform (1 px rims, hairlines).
    #[must_use]
    pub fn device_px(&self) -> f32 {
        let s = self.state.xf.scale_factor();
        if s > 1e-6 {
            1.0 / s
        } else {
            1.0
        }
    }

    /// Multiplies the opacity of everything drawn until the matching restore.
    pub fn set_opacity(&mut self, opacity: f32) {
        self.state.opacity *= finite_or(opacity, 0.0).clamp(0.0, 1.0);
    }

    /// Sets the blend mode until the matching restore.
    pub fn set_blend(&mut self, blend: BlendMode) {
        self.state.blend = blend;
    }

    /// Clips further drawing to `rect` (local) with rounded corners. The clip is the
    /// axis-aligned target-space bounds of the transformed rectangle, intersected with any
    /// current clip.
    pub fn clip(&mut self, rect: Rect, radius: f32) {
        let world = self.state.xf.apply_rect(&rect);
        let world = match self.state.clip_rect {
            Some(c) => world.intersect(&c),
            None => world,
        };
        if self.scene.clips.len() >= MAX_CLIPS {
            self.scene.dropped += 1;
            return;
        }
        let r = finite_or(radius, 0.0).max(0.0) * self.state.xf.scale_factor();
        self.scene.clips.push(GpuClip {
            rect: world.to_array(),
            params: [r, 0.0, 0.0, 0.0],
        });
        self.state.clip = (self.scene.clips.len() - 1) as u32;
        self.state.clip_rect = Some(world);
    }

    /// Runs `f` in a new layer: bloom, blur, glass backdrop and opacity apply to everything it
    /// draws. Layers nest; inner options replace outer ones for their draws.
    pub fn layer<R>(&mut self, opts: LayerOptions, f: impl FnOnce(&mut Self) -> R) -> R {
        self.save();
        let s = self.state.xf.scale_factor();
        let key = SegmentKey {
            blur_px: opts.blur.map_or(0.0, |b| finite_or(b, 0.0).max(0.0) * s),
            backdrop_px: opts.backdrop.map(|b| finite_or(b.frost_sigma, 0.0).max(0.0) * s),
        };
        if key != self.state.segment || key.backdrop_px.is_some() {
            self.state.segment = key;
            self.segment_open = false;
        }
        self.state.bloom = opts.bloom;
        self.set_opacity(opts.opacity);
        let r = f(self);
        self.restore();
        r
    }

    // ----- drawing -----------------------------------------------------------------------

    /// Fills `shape` with `paint`.
    pub fn fill(&mut self, shape: &Shape<'_>, paint: impl Into<Paint>) {
        self.draw(shape, &paint.into(), mode::FILL, [0.0; 4], 0, 0.0);
    }

    /// Strokes the outline of `shape`.
    pub fn stroke(&mut self, shape: &Shape<'_>, paint: impl Into<Paint>, stroke: Stroke) {
        let w = finite_or(stroke.width, 0.0).max(0.0);
        let align = match stroke.align {
            StrokeAlign::Center => 0.0,
            StrokeAlign::Inner => -1.0,
            StrokeAlign::Outer => 1.0,
        };
        let (flags, w_local) = match stroke.unit {
            StrokeUnit::Local => (0, w),
            StrokeUnit::DevicePx => (flag::STROKE_PX, w * self.device_px()),
        };
        let outward = match stroke.align {
            StrokeAlign::Center => w_local * 0.5,
            StrokeAlign::Inner => 0.0,
            StrokeAlign::Outer => w_local,
        };
        self.draw(shape, &paint.into(), mode::STROKE, [w, align, 0.0, 0.0], flags, outward);
    }

    /// Draws a soft outer glow of `shape` (Gaussian falloff of the distance).
    pub fn glow(&mut self, shape: &Shape<'_>, paint: impl Into<Paint>, glow: Glow) {
        let sigma = finite_or(glow.sigma, 0.0).max(1e-3);
        let flags = if glow.fill_inside { flag::GLOW_FILL } else { 0 };
        self.draw(
            shape,
            &paint.into(),
            mode::GLOW,
            [sigma, finite_or(glow.intensity, 0.0).max(0.0), 0.0, 0.0],
            flags,
            sigma * 3.0,
        );
    }

    /// Draws a blurred, offset shadow of `shape`. Draw it before the shape itself.
    pub fn shadow(&mut self, shape: &Shape<'_>, shadow: Shadow) {
        let sigma = finite_or(shadow.sigma, 0.0).max(1e-3);
        let off = shadow.offset.sanitized();
        let spread = finite_or(shadow.spread, 0.0);
        let pad = sigma * 3.0 + spread.max(0.0) + off.x.abs().max(off.y.abs());
        self.draw(
            shape,
            &Paint::Solid(shadow.color),
            mode::SHADOW,
            [off.x, off.y, sigma, spread],
            0,
            pad,
        );
    }

    /// Draws `image` stretched over `rect`.
    pub fn image(&mut self, rect: Rect, image: ImageId) {
        self.fill(&Shape::rect(rect), crate::paint::ImagePaint::new(image, rect));
    }

    /// Draws `image` as a 9-slice: corners keep their size (`insets` in local units: left, top,
    /// right, bottom), edges stretch in one direction, the centre in both. `uv_insets` are the
    /// matching source insets as fractions of the image.
    pub fn nine_slice(&mut self, rect: Rect, image: ImageId, insets: [f32; 4], uv_insets: [f32; 4], tint: Color) {
        let paint = Paint::Image(crate::paint::ImagePaint::new(image, rect).tint(tint));
        let enc = Encoded {
            kind: kind::NINESLICE,
            flags: 0,
            p0: [rect.min.x, rect.min.y, rect.width(), rect.height()],
            p1: insets.map(|v| finite_or(v, 0.0).max(0.0)),
            p2: uv_insets.map(|v| finite_or(v, 0.0).clamp(0.0, 0.5)),
        };
        self.push(enc, rect, &paint, mode::FILL, [0.0; 4]);
    }

    /// Records a mesh draw (glTF car) at the current layer. The mesh pass is depth-tested and
    /// unlit / matcap with a rim light.
    pub fn mesh(&mut self, draw: &MeshDraw) {
        if self.scene.meshes.len() >= MAX_MESHES {
            self.scene.dropped += 1;
            return;
        }
        if !self.ensure_segment() {
            return;
        }
        let viewport = self.state.xf.apply_rect(&draw.viewport);
        self.scene.bounds = self.scene.bounds.union(&viewport);
        self.scene.meshes.push(MeshCmd {
            draw: *draw,
            viewport,
            opacity: self.state.opacity,
            bloom: pack_bloom(self.state.bloom),
        });
        let idx = (self.scene.meshes.len() - 1) as u32;
        self.scene.draws.push(DrawCmd::Mesh(idx));
        if let Some(seg) = self.scene.segments.last_mut() {
            seg.draws.end = self.scene.draws.len() as u32;
        }
    }

    /// Draws `text` with its pen origin at `pos` (see [`TextStyle`] for alignment). Returns the
    /// bounds of the set text in local units. Glyphs missing from the atlas are generated after
    /// the frame and appear on the next one.
    pub fn text(&mut self, pos: Vec2, text: &str, style: &TextStyle) -> Rect {
        let fonts = self.fonts;
        let Some(run) = fonts.prepare_run(text, style) else {
            return Rect::from_points(pos, pos);
        };
        let origin = run.origin(pos);
        let paint = style.paint;
        let mut xf_saved = None;
        if style.slant.abs() > 1e-4 {
            xf_saved = Some(self.state.xf);
            let skew = Affine::translate(-origin)
                .then(&Affine::skew_x(-style.slant))
                .then(&Affine::translate(origin));
            self.state.xf = skew.then(&self.state.xf);
        }
        let font_index = run.font_index();
        fonts.layout(text, style, &run, origin, |g| match g {
            Ok(g) => {
                let enc = Encoded {
                    kind: kind::GLYPH,
                    flags: 0,
                    p0: g.uv,
                    p1: [
                        g.local_per_texel,
                        g.px_range,
                        finite_or(style.weight, 0.0).clamp(-1.0, 1.0),
                        0.0,
                    ],
                    p2: g.quad.to_array(),
                };
                self.push(enc, g.quad, &paint, mode::FILL, [0.0; 4]);
            }
            Err(ch) => {
                if self.scene.missing_glyphs.len() < MAX_MISSING
                    && !self.scene.missing_glyphs.contains(&(font_index, ch))
                {
                    self.scene.missing_glyphs.push((font_index, ch));
                }
            }
        });
        if let Some(xf) = xf_saved {
            self.state.xf = xf;
        }
        run.bounds(origin)
    }

    /// Measures `text` without drawing it.
    #[must_use]
    pub fn measure(&self, text: &str, style: &TextStyle) -> TextMetrics {
        self.fonts.measure(text, style)
    }

    /// Draws an integer in `cells` fixed-width digit cells (tabular, tokens `type`): cells
    /// without a digit show a ghost `8` in `ghost` when given, so the field width never changes.
    /// `pos` is the field's anchor per `style.align`. Allocation-free.
    pub fn number(&mut self, pos: Vec2, value: i64, cells: u8, style: &TextStyle, ghost: Option<Paint>) -> Rect {
        let mut buf = [0u8; 24];
        let text = format_int(value, &mut buf);
        let cells = usize::from(cells.clamp(1, 20));
        let mut tab = *style;
        tab.figures = crate::text::Figures::Tabular;
        let cell_w = self.fonts.tabular_cell(&tab);
        let field_w = cell_w * cells as f32;
        let left = match style.align {
            crate::text::Align::Left => pos.x,
            crate::text::Align::Center => pos.x - field_w * 0.5,
            crate::text::Align::Right => pos.x - field_w,
        };
        let shown = &text[text.len().saturating_sub(cells)..];
        let empty = cells - shown.len();
        let mut left_style = tab;
        left_style.align = crate::text::Align::Left;
        if let Some(ghost_paint) = ghost {
            let mut g = left_style;
            g.paint = ghost_paint;
            let eights = [b'8'; 20];
            if let Ok(s) = std::str::from_utf8(&eights[..empty]) {
                if !s.is_empty() {
                    self.text(vec2(left, pos.y), s, &g);
                }
            }
        }
        let r = self.text(vec2(left + cell_w * empty as f32, pos.y), shown, &left_style);
        Rect::from_points(vec2(left, r.min.y), vec2(left + field_w, r.max.y))
    }

    // ----- internals ---------------------------------------------------------------------

    fn draw(&mut self, shape: &Shape<'_>, paint: &Paint, coverage_mode: u32, style: [f32; 4], flags: u32, pad: f32) {
        let enc = match *shape {
            Shape::Polyline { points, width, closed } => self.encode_points(points, kind::POLYLINE, width, closed),
            Shape::Polygon { points } => self.encode_points(points, kind::POLYGON, 0.0, true),
            _ => encode_simple(shape),
        };
        let Some(mut enc) = enc else { return };
        enc.flags |= flags;
        let bbox = shape.bounds().expand(pad.max(0.0));
        self.push(enc, bbox, paint, coverage_mode, style);
    }

    fn encode_points(&mut self, points: &[Vec2], k: u32, width: f32, closed: bool) -> Option<Encoded> {
        let min = if k == kind::POLYGON { 3 } else { 2 };
        if points.len() < min {
            return None;
        }
        if self.scene.points.len() + points.len() > MAX_POINTS {
            self.scene.dropped += 1;
            return None;
        }
        let first = self.scene.points.len();
        self.scene
            .points
            .extend(points.iter().map(|p| [finite_or(p.x, 0.0), finite_or(p.y, 0.0)]));
        Some(Encoded {
            kind: k,
            flags: if closed { flag::CLOSED } else { 0 },
            p0: [first as f32, points.len() as f32, 0.0, 0.0],
            p1: [finite_or(width, 0.0).max(0.0) * 0.5, 0.0, 0.0, 0.0],
            p2: [0.0; 4],
        })
    }

    /// Opens a segment for the current layer options if none is open. False when full.
    fn ensure_segment(&mut self) -> bool {
        if self.segment_open {
            return true;
        }
        if self.scene.segments.len() >= MAX_SEGMENTS {
            // Keep drawing into the last segment rather than dropping content.
            self.segment_open = !self.scene.segments.is_empty();
            if !self.segment_open {
                self.scene.dropped += 1;
            }
            return self.segment_open;
        }
        let at = self.scene.draws.len() as u32;
        self.scene.segments.push(Segment {
            draws: at..at,
            blur_px: self.state.segment.blur_px,
            backdrop_px: self.state.segment.backdrop_px,
        });
        self.segment_open = true;
        true
    }

    fn push(&mut self, enc: Encoded, bbox: Rect, paint: &Paint, coverage_mode: u32, style: [f32; 4]) {
        if self.scene.instances.len() >= MAX_INSTANCES {
            self.scene.dropped += 1;
            return;
        }
        let xf = self.state.xf;
        let px_scale = xf.scale_factor();
        if px_scale <= 1e-6 || bbox.is_empty() && enc.kind != kind::GLYPH {
            return;
        }
        if !self.ensure_segment() {
            return;
        }
        let Some((paint_index, color)) = self.encode_paint(paint) else {
            self.scene.dropped += 1;
            return;
        };
        let is_glass = matches!(paint, Paint::Glass(_));
        let blend = if is_glass { BlendMode::Add } else { self.state.blend };
        let image = paint.image();

        // Anti-aliasing margin: 1.5 device pixels.
        let mut bbox = bbox.expand(1.5 / px_scale);
        // Trim the quad to the clip when the transform is axis-aligned: pixels outside the clip
        // are discarded anyway, so a large clipped shape (a dome lip, a masked sweep) costs only
        // the clipped area.
        // Glass derives its optics from the full bounds, so it is never trimmed.
        if let (Some(clip), false) = (self.state.clip_rect, is_glass) {
            let [a, b, c, d, tx, ty] = xf.m;
            if b == 0.0 && c == 0.0 && a > 1e-6 && d > 1e-6 {
                let local = Rect::from_points(
                    vec2((clip.min.x - tx) / a, (clip.min.y - ty) / d),
                    vec2((clip.max.x - tx) / a, (clip.max.y - ty) / d),
                );
                bbox = bbox.intersect(&local);
                if bbox.is_empty() && enc.kind != kind::GLYPH {
                    return;
                }
            }
        }
        let world = xf.apply_rect(&bbox);
        let blur_margin = self.state.segment.blur_px * 3.0;
        self.scene.bounds = self.scene.bounds.union(&world.expand(blur_margin));
        let bloom = if is_glass { 0 } else { pack_bloom(self.state.bloom) };
        if bloom != 0 {
            self.scene.bloom_used = true;
        }
        let [a, b, c, d, tx, ty] = xf.m;
        self.scene.instances.push(Instance {
            xform: [a, b, c, d],
            xlate: [tx, ty, px_scale, self.state.opacity],
            bbox: bbox.to_array(),
            p0: enc.p0,
            p1: enc.p1,
            p2: enc.p2,
            style,
            color: color.to_array(),
            ids: [
                enc.kind | (coverage_mode << 8) | (enc.flags << 16),
                paint_index,
                self.state.clip,
                bloom,
            ],
        });
        let index = (self.scene.instances.len() - 1) as u32;

        let extend = matches!(
            self.scene.draws.last(),
            Some(DrawCmd::Shapes { instances, blend: b, image: i }) if *b == blend && *i == image && instances.end == index
        ) && self
            .scene
            .segments
            .last()
            .is_some_and(|s| s.draws.end == self.scene.draws.len() as u32 && s.draws.start < s.draws.end);
        if extend {
            if let Some(DrawCmd::Shapes { instances, .. }) = self.scene.draws.last_mut() {
                instances.end = index + 1;
            }
        } else {
            self.scene.draws.push(DrawCmd::Shapes {
                instances: index..index + 1,
                blend,
                image,
            });
        }
        if let Some(seg) = self.scene.segments.last_mut() {
            seg.draws.end = self.scene.draws.len() as u32;
        }
    }

    /// Returns `(paint index, solid colour)`. Solid paints live in the instance (index 0).
    fn encode_paint(&mut self, paint: &Paint) -> Option<(u32, Color)> {
        if let Paint::Solid(c) = paint {
            return Some((0, *c));
        }
        if let Some((last, idx)) = self.last_paint {
            if last == *paint {
                return Some((idx, Color::WHITE));
            }
        }
        if self.scene.paints.len() >= MAX_PAINTS {
            return None;
        }
        let gpu = gpu_paint(paint, self.images);
        self.scene.paints.push(gpu);
        let idx = (self.scene.paints.len() - 1) as u32;
        self.last_paint = Some((*paint, idx));
        Some((idx, Color::WHITE))
    }
}

/// Converts a paint to its GPU record.
fn gpu_paint(paint: &Paint, images: &Images) -> GpuPaint {
    let mut g: GpuPaint = bytemuck::Zeroable::zeroed();
    match paint {
        Paint::Solid(c) => {
            g.head = [paint_kind::SOLID, 1, 0, 0];
            g.colors[0] = c.to_array();
        }
        Paint::Gradient(grad) => {
            let (kind, g0) = match grad.kind {
                GradientKind::Linear { start, end } => (paint_kind::LINEAR, [start.x, start.y, end.x, end.y]),
                GradientKind::Radial { center, radius } => (
                    paint_kind::RADIAL,
                    [center.x, center.y, finite_or(radius, 1.0).max(1e-3), 0.0],
                ),
                GradientKind::Conic { center, start_angle } => (
                    paint_kind::CONIC,
                    [center.x, center.y, finite_or(start_angle, 0.0), 0.0],
                ),
            };
            let spread = match grad.spread {
                Spread::Pad => 0,
                Spread::Repeat => 1,
                Spread::Reflect => 2,
            };
            let stops = grad.stops();
            g.head = [kind, stops.len() as u32, spread, 0];
            g.g0 = g0.map(|v| finite_or(v, 0.0));
            for (i, s) in stops.iter().enumerate() {
                g.colors[i] = s.color.to_array();
                g.offsets[i / 4][i % 4] = s.offset;
            }
        }
        Paint::Image(img) => {
            let wrap = match img.wrap {
                ImageWrap::Clamp => 0,
                ImageWrap::Repeat => 1,
            };
            let (w, h) = images.size(img.image).unwrap_or((1, 1));
            g.head = [paint_kind::IMAGE, 0, wrap, 1];
            g.g0 = [img.rect.min.x, img.rect.min.y, img.rect.width(), img.rect.height()];
            g.g1 = [img.uv.min.x, img.uv.min.y, img.uv.width(), img.uv.height()];
            let straight = if images.is_straight(img.image) { 1.0 } else { 0.0 };
            g.g2 = [w as f32, h as f32, images.mip_count(img.image) as f32, straight];
            g.colors[0] = img.tint.to_array();
        }
        Paint::Glass(gl) => {
            let (w, h) = gl.caustic.and_then(|c| images.size(c)).unwrap_or((1, 1));
            g.head = [paint_kind::GLASS, 0, 0, u32::from(gl.caustic.is_some())];
            g.g0 = [
                finite_or(gl.frost, 0.0).clamp(0.0, 1.0),
                finite_or(gl.frost_noise, 0.0).max(0.0),
                finite_or(gl.lens, 0.0),
                finite_or(gl.lens_band, 1.0).max(1e-3),
            ];
            g.g1 = [
                finite_or(gl.fresnel, 0.0).max(0.0),
                finite_or(gl.inner_rim_inset, 0.0).max(0.0),
                finite_or(gl.sheen, 0.0).max(0.0),
                finite_or(gl.sheen_height, 0.0).clamp(0.0, 1.0),
            ];
            g.g2 = [
                finite_or(gl.sheen_sag, 0.0),
                finite_or(gl.light_angle, 0.0),
                finite_or(gl.caustic_strength, 0.0).max(0.0),
                match gl.refraction {
                    Refraction::OwnLayers => 0.0,
                    Refraction::World => 1.0,
                },
            ];
            g.g3 = [w as f32, h as f32, 0.0, 0.0];
            g.colors[0] = gl.tint.to_array();
            g.colors[1] = gl.rim_top.to_array();
            g.colors[2] = gl.rim_mid.to_array();
            g.colors[3] = gl.rim_bottom.to_array();
            g.colors[4] = gl.inner_rim.to_array();
        }
    }
    g
}

/// Formats `v` into `buf` without allocating; returns the digits (with a leading `-`).
fn format_int(v: i64, buf: &mut [u8; 24]) -> &str {
    let neg = v < 0;
    let mut n = v.unsigned_abs();
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    if neg {
        i -= 1;
        buf[i] = b'-';
    }
    std::str::from_utf8(&buf[i..]).unwrap_or("0")
}

#[cfg(test)]
mod tests {
    use super::format_int;

    #[test]
    fn integers_format_without_allocation() {
        let mut b = [0u8; 24];
        assert_eq!(format_int(0, &mut b), "0");
        assert_eq!(format_int(118, &mut b), "118");
        assert_eq!(format_int(-42, &mut b), "-42");
        assert_eq!(format_int(i64::MIN, &mut b), "-9223372036854775808");
    }
}
