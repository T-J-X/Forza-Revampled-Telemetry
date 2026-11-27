//! The per-frame scene: a fixed-capacity arena of GPU instances, paints, clips, point lists,
//! draw commands and layer segments. It is cleared, never freed, between frames, so recording a
//! warm frame performs no heap allocation.
use crate::math::{finite_or, Rect};
use crate::mesh::MeshCmd;
use crate::paint::{BlendMode, ImageId};
use bytemuck::{Pod, Zeroable};
use std::ops::Range;

/// Most shape instances one frame can hold; more are dropped and counted.
pub const MAX_INSTANCES: usize = 16_384;
/// Most distinct paints per frame (index 0 is reserved for "solid instance colour").
pub const MAX_PAINTS: usize = 2_048;
/// Most clip rectangles per frame (index 0 is reserved for "no clip").
pub const MAX_CLIPS: usize = 256;
/// Most polyline / polygon points per frame.
pub const MAX_POINTS: usize = 16_384;
/// Most mesh draws per frame.
pub const MAX_MESHES: usize = 64;
/// Most layer segments per frame.
pub const MAX_SEGMENTS: usize = 64;

/// One shape instance as the GPU sees it (`shape.wgsl` `Instance`). 144 bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub(crate) struct Instance {
    /// Local-to-target linear part `[a, b, c, d]`.
    pub xform: [f32; 4],
    /// Translation `[tx, ty]`, device pixels per local unit, opacity.
    pub xlate: [f32; 4],
    /// Local quad `[min.x, min.y, max.x, max.y]`, padded for AA and effects.
    pub bbox: [f32; 4],
    /// Shape parameters.
    pub p0: [f32; 4],
    /// Shape parameters.
    pub p1: [f32; 4],
    /// Shape parameters.
    pub p2: [f32; 4],
    /// Mode parameters: stroke (width, align), glow (sigma, intensity), shadow (dx, dy, sigma, spread).
    pub style: [f32; 4],
    /// Solid colour (linear premultiplied) when `paint == 0`.
    pub color: [f32; 4],
    /// `kind | mode << 8 | flags << 16`, paint index, clip index, packed bloom (intensity, threshold).
    pub ids: [u32; 4],
}

/// A paint as the GPU sees it (`shape.wgsl` `Paint`). 240 bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuPaint {
    /// kind, stop count, spread / wrap, has image.
    pub head: [u32; 4],
    /// Geometry.
    pub g0: [f32; 4],
    /// Geometry / material parameters.
    pub g1: [f32; 4],
    /// Material parameters.
    pub g2: [f32; 4],
    /// Material parameters.
    pub g3: [f32; 4],
    /// Stop colours (or material colours), linear premultiplied.
    pub colors: [[f32; 4]; 8],
    /// Stop offsets, 4 per vector.
    pub offsets: [[f32; 4]; 2],
}

/// An axis-aligned rounded clip in target pixels (`shape.wgsl` `Clip`). 32 bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuClip {
    /// `[min.x, min.y, max.x, max.y]` in target pixels.
    pub rect: [f32; 4],
    /// Corner radius in pixels, then padding.
    pub params: [f32; 4],
}

/// Bloom contribution of a layer (tokens `glow.bloomChain`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bloom {
    /// Luminance above which pixels bloom (tokens: 0.7). 0 makes everything in the layer glow.
    pub threshold: f32,
    /// Strength of the emissive write (tokens: 0.8).
    pub intensity: f32,
}

impl Default for Bloom {
    fn default() -> Self {
        Self {
            threshold: 0.7,
            intensity: 0.8,
        }
    }
}

/// A snapshot of everything drawn so far, sampled by glass in the layer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Backdrop {
    /// Blur sigma of the frosted copy, in local units at the layer's transform.
    pub frost_sigma: f32,
}

impl Default for Backdrop {
    fn default() -> Self {
        Self { frost_sigma: 12.0 }
    }
}

/// Options for [`crate::Painter::layer`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerOptions {
    /// Writes the layer's light into the bloom buffer. `None` = no bloom ("bloom is earned").
    pub bloom: Option<Bloom>,
    /// Gaussian-blurs the whole layer before compositing (sigma in local units).
    pub blur: Option<f32>,
    /// Snapshots the scene at the start of the layer so glass can refract it.
    pub backdrop: Option<Backdrop>,
    /// Layer opacity (multiplies each draw).
    pub opacity: f32,
}

impl Default for LayerOptions {
    fn default() -> Self {
        Self {
            bloom: None,
            blur: None,
            backdrop: None,
            opacity: 1.0,
        }
    }
}

impl LayerOptions {
    /// A layer whose light blooms with the token defaults.
    #[must_use]
    pub fn bloom() -> Self {
        Self {
            bloom: Some(Bloom::default()),
            ..Self::default()
        }
    }

    /// A glass layer: snapshots the backdrop with the given frost sigma.
    #[must_use]
    pub fn glass(frost_sigma: f32) -> Self {
        Self {
            backdrop: Some(Backdrop { frost_sigma }),
            ..Self::default()
        }
    }

    /// A blurred UI layer.
    #[must_use]
    pub fn blurred(sigma: f32) -> Self {
        Self {
            blur: Some(sigma),
            ..Self::default()
        }
    }
}

/// One GPU draw: a run of instances sharing blend mode and image.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DrawCmd {
    /// Instanced shapes.
    Shapes {
        instances: Range<u32>,
        blend: BlendMode,
        image: Option<ImageId>,
    },
    /// One mesh draw (index into `Scene::meshes`).
    Mesh(u32),
}

/// A run of draws with the same layer options.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Segment {
    /// Draw commands.
    pub draws: Range<u32>,
    /// Blur sigma in device pixels (0 = none).
    pub blur_px: f32,
    /// Backdrop frost sigma in device pixels, when the segment snapshots the backdrop.
    pub backdrop_px: Option<f32>,
}

/// Packs bloom (intensity, threshold) into one `u32` as two f16 halves.
pub(crate) fn pack_bloom(bloom: Option<Bloom>) -> u32 {
    match bloom {
        None => 0,
        Some(b) => {
            let i = f16_bits(finite_or(b.intensity, 0.0).clamp(0.0, 64.0));
            let t = f16_bits(finite_or(b.threshold, 0.0).clamp(0.0, 64.0));
            u32::from(i) | (u32::from(t) << 16)
        }
    }
}

/// IEEE half-precision bits of a non-negative finite `f32` (round to nearest, no NaN input).
pub(crate) fn f16_bits(v: f32) -> u16 {
    let bits = v.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = ((bits >> 23) & 0xFF) as i32;
    let mant = bits & 0x7F_FFFF;
    if exp == 0 {
        return sign;
    }
    let e = exp - 127 + 15;
    if e >= 31 {
        return sign | 0x7C00;
    }
    if e <= 0 {
        if e < -10 {
            return sign;
        }
        let m = (mant | 0x80_0000) >> (1 - e);
        return sign | ((m + 0x1000) >> 13) as u16;
    }
    let half = sign as u32 | ((e as u32) << 10) | (mant >> 13);
    // Round to nearest.
    (half + ((mant >> 12) & 1)) as u16
}

/// The frame arena.
#[derive(Debug)]
pub struct Scene {
    pub(crate) instances: Vec<Instance>,
    pub(crate) paints: Vec<GpuPaint>,
    pub(crate) clips: Vec<GpuClip>,
    pub(crate) points: Vec<[f32; 2]>,
    pub(crate) draws: Vec<DrawCmd>,
    pub(crate) segments: Vec<Segment>,
    pub(crate) meshes: Vec<MeshCmd>,
    /// Union of all draws in target pixels.
    pub(crate) bounds: Rect,
    /// Items dropped this frame because a fixed capacity was reached.
    pub(crate) dropped: u32,
    /// Glyphs asked for that the atlas does not hold yet: `(font index, char)`.
    pub(crate) missing_glyphs: Vec<(u16, char)>,
    /// Largest bloom radius used, in device pixels (sizes the work rectangle margin).
    pub(crate) bloom_used: bool,
}

/// Most missing glyphs remembered per frame.
pub(crate) const MAX_MISSING: usize = 64;

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    /// Allocates every list at its fixed capacity once.
    #[must_use]
    pub fn new() -> Self {
        let mut s = Self {
            instances: Vec::with_capacity(MAX_INSTANCES),
            paints: Vec::with_capacity(MAX_PAINTS),
            clips: Vec::with_capacity(MAX_CLIPS),
            points: Vec::with_capacity(MAX_POINTS),
            draws: Vec::with_capacity(MAX_INSTANCES.min(4096)),
            segments: Vec::with_capacity(MAX_SEGMENTS),
            meshes: Vec::with_capacity(MAX_MESHES),
            bounds: Rect::EMPTY,
            dropped: 0,
            missing_glyphs: Vec::with_capacity(MAX_MISSING),
            bloom_used: false,
        };
        s.clear();
        s
    }

    /// Empties the arena, keeping every allocation.
    pub fn clear(&mut self) {
        self.instances.clear();
        self.paints.clear();
        self.paints.push(GpuPaint::zeroed());
        self.clips.clear();
        self.clips.push(GpuClip {
            rect: [-1e9, -1e9, 1e9, 1e9],
            params: [0.0; 4],
        });
        self.points.clear();
        self.draws.clear();
        self.segments.clear();
        self.meshes.clear();
        self.bounds = Rect::EMPTY;
        self.dropped = 0;
        self.missing_glyphs.clear();
        self.bloom_used = false;
    }

    /// Number of shape instances recorded.
    #[must_use]
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// Number of GPU draw calls the scene will issue for shapes and meshes.
    #[must_use]
    pub fn draw_count(&self) -> usize {
        self.draws.len()
    }

    /// Number of layer segments.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Glyphs drawn this frame that are not in the atlas yet. The renderer generates up to 16 per
    /// frame; offline renders (goldens, `preview-export`) repeat a frame until this is 0.
    #[must_use]
    pub fn missing_glyph_count(&self) -> usize {
        self.missing_glyphs.len()
    }

    /// Items dropped because a fixed capacity was reached (0 in a healthy frame).
    #[must_use]
    pub fn dropped(&self) -> u32 {
        self.dropped
    }

    /// Union of everything drawn, in target pixels (empty when nothing was drawn).
    #[must_use]
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// The `(capacity)` of every list, to prove in tests that warm frames do not grow them.
    #[must_use]
    pub fn capacities(&self) -> [usize; 9] {
        [
            self.instances.capacity(),
            self.paints.capacity(),
            self.clips.capacity(),
            self.points.capacity(),
            self.draws.capacity(),
            self.segments.capacity(),
            self.meshes.capacity(),
            self.missing_glyphs.capacity(),
            0,
        ]
    }

    /// A hash of everything the GPU will draw this frame: every instance, paint, clip, point,
    /// draw run, layer segment and mesh draw, bit for bit. Two frames with the same fingerprint
    /// draw the same picture, so a test can tell whether an input reached the frame without a
    /// GPU readback. It is stable within one build, not across builds.
    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        use std::hash::{DefaultHasher, Hasher};
        let mut h = DefaultHasher::new();
        self.for_each_float(|v| h.write_u32(v.to_bits()));
        for i in &self.instances {
            h.write(bytemuck::bytes_of(&i.ids));
        }
        for p in &self.paints {
            h.write(bytemuck::bytes_of(&p.head));
        }
        for d in &self.draws {
            match d {
                DrawCmd::Shapes {
                    instances,
                    blend,
                    image,
                } => {
                    h.write_u8(0);
                    h.write_u32(instances.start);
                    h.write_u32(instances.end);
                    std::hash::Hash::hash(blend, &mut h);
                    std::hash::Hash::hash(image, &mut h);
                }
                DrawCmd::Mesh(index) => {
                    h.write_u8(1);
                    h.write_u32(*index);
                }
            }
        }
        for s in &self.segments {
            h.write_u32(s.draws.start);
            h.write_u32(s.draws.end);
            h.write_u8(u8::from(s.backdrop_px.is_some()));
        }
        for m in &self.meshes {
            h.write_u32(m.draw.mesh.0);
            h.write_u8(m.draw.shading as u8);
            h.write_u32(m.bloom);
        }
        h.write_u32(self.dropped);
        h.finish()
    }

    /// How many floats the GPU would read this frame are NaN or infinite (0 in a healthy frame).
    /// The robustness gate (G5) drives every dash with hostile telemetry and asserts this is 0.
    #[must_use]
    pub fn non_finite_count(&self) -> usize {
        let mut n = 0;
        self.for_each_float(|v| n += usize::from(!v.is_finite()));
        n
    }

    /// Calls `f` with every float the GPU reads: instance, paint, clip and point data, layer
    /// blur and frost sigmas, and the mesh draws.
    fn for_each_float(&self, mut f: impl FnMut(f32)) {
        let mut all = |vs: &[f32]| vs.iter().copied().for_each(&mut f);
        for i in &self.instances {
            for v in [i.xform, i.xlate, i.bbox, i.p0, i.p1, i.p2, i.style, i.color] {
                all(&v);
            }
        }
        for p in &self.paints {
            for v in [p.g0, p.g1, p.g2, p.g3] {
                all(&v);
            }
            all(bytemuck::cast_slice(&p.colors));
            all(bytemuck::cast_slice(&p.offsets));
        }
        for c in &self.clips {
            all(&c.rect);
            all(&c.params);
        }
        all(bytemuck::cast_slice(&self.points));
        for s in &self.segments {
            all(&[s.blur_px, s.backdrop_px.unwrap_or(0.0)]);
        }
        for m in &self.meshes {
            let d = &m.draw;
            all(bytemuck::cast_slice(&d.model.cols));
            all(bytemuck::cast_slice(&d.view_proj.cols));
            all(&[d.eye.x, d.eye.y, d.eye.z, d.light.x, d.light.y, d.light.z]);
            for r in [d.viewport, m.viewport] {
                all(&[r.min.x, r.min.y, r.max.x, r.max.y]);
            }
            for c in [d.color, d.rim] {
                all(&[c.r, c.g, c.b, c.a]);
            }
            all(&[m.opacity]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn gpu_layouts() {
        assert_eq!(size_of::<Instance>(), 144);
        assert_eq!(size_of::<GpuPaint>(), 240);
        assert_eq!(size_of::<GpuClip>(), 32);
    }

    fn instance(x: f32) -> Instance {
        Instance {
            xform: [1.0, 0.0, 0.0, 1.0],
            xlate: [x, 0.0, 1.0, 1.0],
            ..Instance::zeroed()
        }
    }

    #[test]
    fn fingerprint_follows_every_drawn_value() {
        let mut a = Scene::new();
        let mut b = Scene::new();
        assert_eq!(a.fingerprint(), b.fingerprint());
        a.instances.push(instance(10.0));
        b.instances.push(instance(10.0));
        assert_eq!(a.fingerprint(), b.fingerprint());
        b.instances[0].xlate[0] = 10.5;
        assert_ne!(a.fingerprint(), b.fingerprint(), "a moved instance");
        b.instances[0] = instance(10.0);
        b.instances[0].ids[1] = 3;
        assert_ne!(a.fingerprint(), b.fingerprint(), "another paint index");
        b.instances[0] = instance(10.0);
        b.draws.push(DrawCmd::Shapes {
            instances: 0..1,
            blend: BlendMode::Add,
            image: None,
        });
        assert_ne!(a.fingerprint(), b.fingerprint(), "a draw run");
        a.draws.push(DrawCmd::Shapes {
            instances: 0..1,
            blend: BlendMode::Normal,
            image: None,
        });
        assert_ne!(a.fingerprint(), b.fingerprint(), "another blend mode");
        a.clear();
        b.clear();
        assert_eq!(a.fingerprint(), b.fingerprint(), "clear resets everything");
    }

    #[test]
    fn non_finite_count_sees_every_float_list() {
        let mut s = Scene::new();
        assert_eq!(s.non_finite_count(), 0, "a cleared scene is finite");
        s.instances.push(instance(f32::NAN));
        assert_eq!(s.non_finite_count(), 1);
        s.instances[0].color[3] = f32::INFINITY;
        assert_eq!(s.non_finite_count(), 2);
        s.paints[0].colors[7][0] = f32::NEG_INFINITY;
        s.paints[0].offsets[1][3] = f32::NAN;
        assert_eq!(s.non_finite_count(), 4);
        s.clips[0].params[0] = f32::NAN;
        s.points.push([f32::NAN, 0.0]);
        s.segments.push(Segment {
            draws: 0..0,
            blur_px: f32::NAN,
            backdrop_px: Some(f32::INFINITY),
        });
        assert_eq!(s.non_finite_count(), 8);
        // Integer fields are not floats: an id whose bits read as NaN does not count.
        s.instances[0].ids = [u32::MAX; 4];
        assert_eq!(s.non_finite_count(), 8);
    }

    #[test]
    fn half_floats() {
        assert_eq!(f16_bits(1.0), 0x3C00);
        assert_eq!(f16_bits(0.5), 0x3800);
        assert_eq!(f16_bits(0.0), 0);
        assert_eq!(f16_bits(65504.0), 0x7BFF);
        assert_eq!(f16_bits(1e6), 0x7C00);
        assert_eq!(pack_bloom(None), 0);
        assert_eq!(
            pack_bloom(Some(Bloom {
                intensity: 1.0,
                threshold: 0.5
            })),
            0x3800_3C00
        );
    }
}
