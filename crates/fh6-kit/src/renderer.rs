//! The GPU side of the kit: pipelines, work targets and the frame graph.
//!
//! A frame is: one upload (every instance, paint, clip, point and uniform packed into a single
//! buffer write), then per layer segment an optional backdrop snapshot and blur, the instanced
//! shape pass (scene + emissive targets, depth for meshes), an optional layer blur and
//! composite, then bloom (dual-filter Kawase, one compute pass) and the resolve into the caller's target.
//!
//! All work targets are sized to the **work rectangle** (the union of what was drawn, plus the
//! bloom margin), not the screen, so a cluster costs what it covers.
use crate::clock::Stamp;
use crate::images::Images;
use crate::math::Rect;
use crate::mesh::Meshes;
use crate::paint::BlendMode;
use crate::scene::{
    DrawCmd, GpuClip, GpuPaint, Instance, Scene, MAX_CLIPS, MAX_INSTANCES, MAX_PAINTS, MAX_POINTS, MAX_SEGMENTS,
};
use crate::text::{Fonts, ATLAS_WIDTH};
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::mem::size_of;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Format of the linear scene, emissive and layer targets.
pub(crate) const SCENE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
/// Depth format of the mesh pass.
pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// Additive blending (emissive target, glass deltas, `BlendMode::Add`).
pub(crate) const ADD_BLEND: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    },
};

/// Pyramid levels for blur and bloom (1/2 .. 1/32 resolution).
const PYRAMID_LEVELS: u32 = 5;
/// Texels past a bloom dispatch's viewport that it writes as zero: as far as a bilinear tap
/// clamped to `src_max` reaches past the viewport, with the pyramid's rounding (see `bloom`).
const BLOOM_MARGIN: u32 = 2;
/// Uniform slot stride (WebGPU's minimum dynamic offset alignment).
const SLOT: u64 = 256;
/// Most post passes per frame.
const MAX_POST: usize = 96;
/// Backdrop frost below this sigma (device px) skips the blurred copy: glass in that layer is clear.
const MIN_FROST_PX: f32 = 0.25;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
struct Globals {
    target_size: [f32; 2],
    work_origin: [f32; 2],
    work_size: [f32; 2],
    cap_size: [f32; 2],
    time: f32,
    backdrop_lod: f32,
    backdrop_valid: f32,
    world_valid: f32,
    world_origin: [f32; 2],
    world_size: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
struct PostParams {
    src_texel: [f32; 2],
    src_max: [f32; 2],
    dst_texel: [f32; 2],
    dir: [f32; 2],
    sigma: f32,
    intensity: f32,
    src_lod: f32,
    base_lod: f32,
    opacity: f32,
    bloom_on: f32,
    origin: [f32; 2],
}

/// Byte offsets of this frame's regions inside the upload buffer.
#[derive(Debug, Clone, Copy, Default)]
struct Offsets {
    globals: u64,
    post: u64,
    instances: u64,
    paints: u64,
    clips: u64,
    points: u64,
}

fn align(v: u64) -> u64 {
    v.div_ceil(SLOT) * SLOT
}

const GLOBALS_CAP: u64 = SLOT * MAX_SEGMENTS as u64;
const POST_CAP: u64 = SLOT * MAX_POST as u64;
const INSTANCES_CAP: u64 = (size_of::<Instance>() * MAX_INSTANCES) as u64;
const PAINTS_CAP: u64 = (size_of::<GpuPaint>() * MAX_PAINTS) as u64;
const CLIPS_CAP: u64 = (size_of::<GpuClip>() * MAX_CLIPS) as u64;
const POINTS_CAP: u64 = (8 * MAX_POINTS) as u64;
/// Upload buffer size: every region at capacity, aligned (the worst-case offsets plus sizes).
const UPLOAD_SIZE: u64 = GLOBALS_CAP + POST_CAP + INSTANCES_CAP + PAINTS_CAP + CLIPS_CAP + POINTS_CAP + 8 * SLOT;

/// Host-visible staging buffers for the frame upload, reused frame after frame. `Queue::write_buffer`
/// creates (and later frees) a staging buffer on every call and submits it in a command list of its
/// own; a ring slot is mapped once, written, unmapped, copied into the upload buffer at the start of
/// the frame's own command buffer, and mapped again after the frame is submitted.
const STAGING_SLOTS: usize = 3;

/// A slot is mapped and free to write.
const SLOT_READY: u8 = 0;
/// A slot was written and unmapped for this frame; it waits for [`Renderer::frame_submitted`].
const SLOT_WRITTEN: u8 = 1;
/// A slot's `map_async` is pending (its frame is on the GPU).
const SLOT_MAPPING: u8 = 2;
/// A slot's map failed (device lost); it is never used again.
const SLOT_LOST: u8 = 3;

#[derive(Debug)]
struct StagingSlot {
    buffer: wgpu::Buffer,
    /// One of the `SLOT_*` states. The map callback sets it from the thread that polls the device.
    state: Arc<AtomicU8>,
}

impl StagingSlot {
    fn new(device: &wgpu::Device) -> Self {
        Self {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("fh6-kit upload staging"),
                size: UPLOAD_SIZE,
                usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: true,
            }),
            state: Arc::new(AtomicU8::new(SLOT_READY)),
        }
    }

    fn state(&self) -> u8 {
        self.state.load(Ordering::Acquire)
    }
}

/// The bytes a frame copies from its staging slot: everything before the post parameters, and the
/// post parameters at capacity (they are pushed while passes are recorded, after the copy).
fn staged_len(post_offset: u64) -> u64 {
    (post_offset + POST_CAP).min(UPLOAD_SIZE)
}

/// A texture with views of each mip and of the whole chain.
#[derive(Debug)]
struct Tex {
    _texture: wgpu::Texture,
    texture_ref: wgpu::Texture,
    full: wgpu::TextureView,
    mips: Vec<wgpu::TextureView>,
    size: (u32, u32),
}

impl Tex {
    fn new(
        device: &wgpu::Device,
        label: &str,
        size: (u32, u32),
        format: wgpu::TextureFormat,
        mips: u32,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        let full = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mips = (0..mips)
            .map(|m| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    base_mip_level: m,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();
        Self {
            texture_ref: texture.clone(),
            _texture: texture,
            full,
            mips,
            size,
        }
    }

    fn mip_size(&self, m: u32) -> (u32, u32) {
        ((self.size.0 >> m).max(1), (self.size.1 >> m).max(1))
    }
}

/// Views a post pass can sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum V {
    Scene,
    Emissive,
    Layer,
    Backdrop,
    Pyr(u32),
    Tmp(u32),
    BPyr(u32),
}

#[derive(Debug)]
struct Targets {
    cap: (u32, u32),
    scene: Tex,
    emissive: Tex,
    layer: Tex,
    backdrop: Tex,
    depth: Tex,
    pyr: Tex,
    tmp: Tex,
    bpyr: Tex,
}

impl Targets {
    fn new(device: &wgpu::Device, cap: (u32, u32)) -> Self {
        use wgpu::TextureUsages as U;
        let rt = U::RENDER_ATTACHMENT | U::TEXTURE_BINDING;
        let half = ((cap.0 / 2).max(1), (cap.1 / 2).max(1));
        Self {
            cap,
            scene: Tex::new(device, "fh6-kit scene", cap, SCENE_FORMAT, 1, rt | U::COPY_SRC),
            emissive: Tex::new(device, "fh6-kit emissive", cap, SCENE_FORMAT, 1, rt),
            layer: Tex::new(device, "fh6-kit layer", cap, SCENE_FORMAT, 1, rt),
            backdrop: Tex::new(
                device,
                "fh6-kit backdrop",
                cap,
                SCENE_FORMAT,
                1,
                // RENDER_ATTACHMENT: cleared by a pass when a glass layer is drawn first.
                rt | U::COPY_DST,
            ),
            depth: Tex::new(device, "fh6-kit depth", cap, DEPTH_FORMAT, 1, U::RENDER_ATTACHMENT),
            // STORAGE_BINDING: the bloom chain writes these from compute (`shaders/bloom.wgsl`).
            pyr: Tex::new(
                device,
                "fh6-kit pyramid",
                half,
                SCENE_FORMAT,
                PYRAMID_LEVELS,
                rt | U::STORAGE_BINDING,
            ),
            tmp: Tex::new(
                device,
                "fh6-kit pyramid tmp",
                half,
                SCENE_FORMAT,
                PYRAMID_LEVELS,
                rt | U::STORAGE_BINDING,
            ),
            bpyr: Tex::new(
                device,
                "fh6-kit backdrop pyramid",
                half,
                SCENE_FORMAT,
                PYRAMID_LEVELS,
                rt,
            ),
        }
    }

    fn view(&self, v: V) -> &wgpu::TextureView {
        fn mip(t: &Tex, m: u32) -> &wgpu::TextureView {
            t.mips.get(m as usize).unwrap_or(&t.full)
        }
        match v {
            V::Scene => &self.scene.full,
            V::Emissive => &self.emissive.full,
            V::Layer => &self.layer.full,
            V::Backdrop => &self.backdrop.full,
            V::Pyr(m) => mip(&self.pyr, m),
            V::Tmp(m) => mip(&self.tmp, m),
            V::BPyr(m) => mip(&self.bpyr, m),
        }
    }

    /// Size of the texture behind `v`.
    fn size(&self, v: V) -> (u32, u32) {
        match v {
            V::Scene | V::Emissive | V::Layer | V::Backdrop => self.cap,
            V::Pyr(m) => self.pyr.mip_size(m),
            V::Tmp(m) => self.tmp.mip_size(m),
            V::BPyr(m) => self.bpyr.mip_size(m),
        }
    }
}

/// Timings and counts of the last rendered frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FrameStats {
    /// CPU time spent in [`crate::Kit::render`] (packing, uploading, recording).
    pub cpu: Duration,
    /// Shape instances drawn.
    pub instances: u32,
    /// Draw calls issued (shapes + meshes + post passes), counting each bloom dispatch as one.
    pub draw_calls: u32,
    /// Render and compute passes recorded (bloom is one compute pass).
    pub passes: u32,
    /// Work rectangle area in pixels.
    pub work_pixels: u64,
    /// Items dropped because a capacity was reached.
    pub dropped: u32,
}

/// One pass of the last frame on the GPU, from the pass timeline ([`crate::Kit::set_gpu_timeline`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpuPassTime {
    /// What ran: `"shapes"`, `"glass shapes"` (a segment that began with a backdrop copy) or
    /// `"resolve"`.
    pub name: &'static str,
    /// The layer segment a shapes pass drew (0 for the resolve).
    pub segment: u32,
    /// When the pass began, from the start of the frame's first pass.
    pub start: Duration,
    /// When the pass ended, from the start of the frame's first pass. The gap to the next pass's
    /// start holds what ran between them: barriers, a backdrop copy, blur or bloom passes.
    pub end: Duration,
}

/// Timestamps one frame can write: the frame's begin and end, then a begin and an end for every
/// shapes pass after the first and for the resolve when the pass timeline is on.
const TIMER_QUERIES: u32 = 2 + 2 * (MAX_SEGMENTS as u32 + 1);

#[derive(Debug)]
struct GpuTimer {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    readback: wgpu::Buffer,
    period_ns: f32,
}

/// A pass of the pass timeline: its name, segment and timestamp indices.
#[derive(Debug, Clone, Copy)]
struct TimelineMark {
    name: &'static str,
    segment: u32,
    begin: u32,
    end: u32,
}

#[derive(Debug)]
struct World {
    view: wgpu::TextureView,
    rect: Rect,
}

/// The renderer. Owned by [`crate::Kit`].
#[derive(Debug)]
pub(crate) struct Renderer {
    shape_bgl: wgpu::BindGroupLayout,
    shape_pipelines: [wgpu::RenderPipeline; 4],
    post_bgl: wgpu::BindGroupLayout,
    post_layout: wgpu::PipelineLayout,
    post_module: wgpu::ShaderModule,
    down: wgpu::RenderPipeline,
    blur: wgpu::RenderPipeline,
    /// The bloom chain as compute (`shaders/bloom.wgsl`): one pass, one dispatch per level.
    bloom_bgl: wgpu::BindGroupLayout,
    bloom_down: wgpu::ComputePipeline,
    bloom_up: wgpu::ComputePipeline,
    composite: wgpu::RenderPipeline,
    resolve: Vec<(wgpu::TextureFormat, wgpu::RenderPipeline)>,
    upload: wgpu::Buffer,
    staging: Vec<u8>,
    /// Reused staging buffers for `upload` (see [`STAGING_SLOTS`]).
    staging_ring: [StagingSlot; STAGING_SLOTS],
    sampler: wgpu::Sampler,
    sampler_rep: wgpu::Sampler,
    sampler_aniso: wgpu::Sampler,
    atlas: Tex,
    atlas_generation: u32,
    world_dummy: Tex,
    world: Option<World>,
    targets: Option<Targets>,
    shape_group: Option<wgpu::BindGroup>,
    post_groups: HashMap<(V, V), wgpu::BindGroup>,
    /// Bloom dispatch groups by (source, base, destination).
    bloom_groups: HashMap<(V, V, V), wgpu::BindGroup>,
    pub(crate) meshes: Meshes,
    timer: Option<GpuTimer>,
    timing_enabled: bool,
    /// Every pass is timed, not only the whole frame (benches).
    timeline_enabled: bool,
    /// The passes the last frame timed, and how many timestamps it wrote.
    timeline: Vec<TimelineMark>,
    queries_used: u32,
    bloom_enabled: bool,
    /// Premultiplied black floor under the whole target (0 = none), see `Kit::set_underlay`.
    underlay: f32,
    globals: Vec<Globals>,
    post: Vec<PostParams>,
    started: Stamp,
}

fn blend_state(mode: BlendMode) -> wgpu::BlendState {
    use wgpu::{BlendComponent as C, BlendFactor as F, BlendOperation as O};
    let comp = |src, dst| C {
        src_factor: src,
        dst_factor: dst,
        operation: O::Add,
    };
    match mode {
        BlendMode::Normal => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
        BlendMode::Add => ADD_BLEND,
        BlendMode::Screen => wgpu::BlendState {
            color: comp(F::One, F::OneMinusSrc),
            alpha: comp(F::One, F::OneMinusSrcAlpha),
        },
        BlendMode::Multiply => wgpu::BlendState {
            color: comp(F::Dst, F::OneMinusSrcAlpha),
            alpha: comp(F::Zero, F::One),
        },
    }
}

impl Renderer {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue, images: &Images) -> Self {
        let shape_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fh6-kit shape"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shape.wgsl").into()),
        });
        let post_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fh6-kit post"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/post.wgsl").into()),
        });
        let tex_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let storage_entry = |binding, size: u64| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: true,
                min_binding_size: wgpu::BufferSize::new(size),
            },
            count: None,
        };
        let sampler_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let shape_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fh6-kit shape layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(size_of::<Globals>() as u64),
                    },
                    count: None,
                },
                storage_entry(1, size_of::<GpuPaint>() as u64),
                storage_entry(2, size_of::<GpuClip>() as u64),
                storage_entry(3, 8),
                tex_entry(4),
                tex_entry(5),
                tex_entry(6),
                tex_entry(7),
                sampler_entry(8),
                sampler_entry(9),
                sampler_entry(10),
            ],
        });
        let shape_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fh6-kit shape pipeline layout"),
            bind_group_layouts: &[Some(&shape_bgl), Some(images.layout())],
            immediate_size: 0,
        });
        const ATTRS: [wgpu::VertexAttribute; 9] = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 16,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 2,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 48,
                shader_location: 3,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 64,
                shader_location: 4,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 80,
                shader_location: 5,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 96,
                shader_location: 6,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 112,
                shader_location: 7,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32x4,
                offset: 128,
                shader_location: 8,
            },
        ];
        let make_shape = |mode: BlendMode| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("fh6-kit shape pipeline"),
                layout: Some(&shape_layout),
                vertex: wgpu::VertexState {
                    module: &shape_module,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &ATTRS,
                    })],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Always),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shape_module,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: SCENE_FORMAT,
                            blend: Some(blend_state(mode)),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        Some(wgpu::ColorTargetState {
                            format: SCENE_FORMAT,
                            blend: Some(ADD_BLEND),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let shape_pipelines = BlendMode::ALL.map(make_shape);

        let post_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fh6-kit post layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(size_of::<PostParams>() as u64),
                    },
                    count: None,
                },
                tex_entry(1),
                tex_entry(2),
                sampler_entry(3),
            ],
        });
        let post_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fh6-kit post pipeline layout"),
            bind_group_layouts: &[Some(&post_bgl)],
            immediate_size: 0,
        });
        let post_pipe =
            |entry: &str, format, blend| make_post(device, &post_layout, &post_module, entry, format, blend);
        let down = post_pipe("fs_down", SCENE_FORMAT, None);
        let (bloom_bgl, bloom_down, bloom_up) = make_bloom(device);
        let blur = post_pipe("fs_blur", SCENE_FORMAT, None);
        let composite = post_pipe(
            "fs_composite",
            SCENE_FORMAT,
            Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        );

        let upload = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fh6-kit frame upload"),
            size: UPLOAD_SIZE,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fh6-kit linear clamp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let sampler_rep = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fh6-kit linear repeat"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        // Environment maps are minified very unevenly across a bevel (`shape.wgsl`, section 7).
        let sampler_aniso = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fh6-kit anisotropic clamp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        });
        let atlas = Tex::new(
            device,
            "fh6-kit glyph atlas",
            (ATLAS_WIDTH, 1024),
            wgpu::TextureFormat::Rgba8Unorm,
            1,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let world_dummy = Tex::new(
            device,
            "fh6-kit world placeholder",
            (1, 1),
            wgpu::TextureFormat::Rgba8UnormSrgb,
            1,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &world_dummy.texture_ref,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0, 0, 0, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let timer = device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
            .then(|| GpuTimer {
                set: device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("fh6-kit gpu timer"),
                    ty: wgpu::QueryType::Timestamp,
                    count: TIMER_QUERIES,
                }),
                resolve: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("fh6-kit timer resolve"),
                    size: u64::from(TIMER_QUERIES) * 8,
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }),
                readback: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("fh6-kit timer readback"),
                    size: u64::from(TIMER_QUERIES) * 8,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                period_ns: queue.get_timestamp_period(),
            });
        Self {
            shape_bgl,
            shape_pipelines,
            post_bgl,
            post_layout,
            post_module,
            down,
            blur,
            bloom_bgl,
            bloom_down,
            bloom_up,
            composite,
            resolve: Vec::with_capacity(2),
            upload,
            staging: Vec::with_capacity(UPLOAD_SIZE as usize),
            staging_ring: std::array::from_fn(|_| StagingSlot::new(device)),
            sampler,
            sampler_rep,
            sampler_aniso,
            atlas,
            atlas_generation: 0,
            world_dummy,
            world: None,
            targets: None,
            shape_group: None,
            post_groups: HashMap::with_capacity(64),
            bloom_groups: HashMap::with_capacity(2 * PYRAMID_LEVELS as usize),
            meshes: Meshes::new(device, DEPTH_FORMAT),
            timer,
            timing_enabled: false,
            timeline_enabled: false,
            timeline: Vec::with_capacity(MAX_SEGMENTS + 1),
            queries_used: 2,
            bloom_enabled: true,
            underlay: 0.0,
            globals: Vec::with_capacity(MAX_SEGMENTS),
            post: Vec::with_capacity(MAX_POST),
            started: Stamp::now(),
        }
    }

    /// Turns GPU timestamp queries on (when the device supports them).
    /// Turns the bloom passes on or off (a quality setting; layers still record emissive).
    /// Sets the black underlay alpha (0..1) applied by the clear and the resolve.
    pub(crate) fn set_underlay(&mut self, alpha: f32) {
        self.underlay = if alpha.is_finite() { alpha.clamp(0.0, 1.0) } else { 0.0 };
    }

    pub(crate) fn set_bloom(&mut self, on: bool) {
        self.bloom_enabled = on;
    }

    pub(crate) fn set_timing(&mut self, on: bool) -> bool {
        self.timing_enabled = on && self.timer.is_some();
        self.timeline_enabled &= self.timing_enabled;
        self.timing_enabled
    }

    /// Times every pass, not only the whole frame (turns frame timing on with it).
    pub(crate) fn set_timeline(&mut self, on: bool) -> bool {
        let timing = self.set_timing(on || self.timing_enabled);
        self.timeline_enabled = on && timing;
        self.timeline_enabled
    }

    /// Sets (or clears) the refraction source: a view of the captured game frame and the target
    /// rectangle it covers.
    pub(crate) fn set_world(&mut self, world: Option<(wgpu::TextureView, Rect)>) {
        self.world = world.map(|(view, rect)| World { view, rect });
        self.shape_group = None;
    }

    /// Tells the renderer that the command buffer of its last [`Renderer::render`] was submitted
    /// (or dropped), so the staging slot it used can be mapped again. Mapping a buffer that a
    /// recorded but unsubmitted command buffer reads would fail that submit, so slots are only
    /// recycled through this call; without it the renderer falls back to `Queue::write_buffer`
    /// once every slot has been used.
    pub(crate) fn frame_submitted(&mut self) {
        for slot in &self.staging_ring {
            if slot.state() != SLOT_WRITTEN {
                continue;
            }
            slot.state.store(SLOT_MAPPING, Ordering::Release);
            let state = Arc::clone(&slot.state);
            slot.buffer.slice(..).map_async(wgpu::MapMode::Write, move |result| {
                let next = if result.is_ok() { SLOT_READY } else { SLOT_LOST };
                state.store(next, Ordering::Release);
            });
        }
    }

    pub(crate) fn has_world(&self) -> bool {
        self.world.is_some()
    }

    fn ensure_resolve(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) -> usize {
        if let Some(i) = self.resolve.iter().position(|(f, _)| *f == format) {
            return i;
        }
        let p = make_post(device, &self.post_layout, &self.post_module, "fs_resolve", format, None);
        self.resolve.push((format, p));
        self.resolve.len() - 1
    }

    fn ensure_targets(&mut self, device: &wgpu::Device, need: (u32, u32), max: (u32, u32)) {
        let grow = |v: u32, m: u32| (v.div_ceil(256) * 256).min(m.max(v)).max(64);
        let ok = self
            .targets
            .as_ref()
            .is_some_and(|t| t.cap.0 >= need.0 && t.cap.1 >= need.1);
        if ok {
            return;
        }
        let old = self.targets.as_ref().map_or((0, 0), |t| t.cap);
        let cap = (grow(need.0.max(old.0), max.0), grow(need.1.max(old.1), max.1));
        tracing::debug!(?cap, "fh6-kit work targets (re)allocated");
        self.targets = Some(Targets::new(device, cap));
        self.shape_group = None;
        self.post_groups.clear();
        self.bloom_groups.clear();
    }

    fn upload_atlas(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, fonts: &mut Fonts) {
        let atlas = &mut fonts.atlas;
        if atlas.generation != self.atlas_generation || self.atlas.size.1 != atlas.height {
            self.atlas = Tex::new(
                device,
                "fh6-kit glyph atlas",
                (ATLAS_WIDTH, atlas.height),
                wgpu::TextureFormat::Rgba8Unorm,
                1,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            );
            self.atlas_generation = atlas.generation;
            self.shape_group = None;
            atlas.dirty.clear();
            atlas.dirty.push(crate::text::DirtyRect {
                x: 0,
                y: 0,
                w: ATLAS_WIDTH,
                h: atlas.height,
            });
        }
        for r in atlas.dirty.drain(..) {
            if r.w == 0 || r.h == 0 {
                continue;
            }
            let offset = u64::from((r.y * ATLAS_WIDTH + r.x) * 4);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas.texture_ref,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: r.x, y: r.y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                &atlas.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset,
                    bytes_per_row: Some(ATLAS_WIDTH * 4),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: r.w,
                    height: r.h,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    fn shape_group(&mut self, device: &wgpu::Device) -> Option<&wgpu::BindGroup> {
        if self.shape_group.is_none() {
            let t = self.targets.as_ref()?;
            let world = self.world.as_ref().map_or(&self.world_dummy.full, |w| &w.view);
            let buf = |size: u64| {
                wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &self.upload,
                    offset: 0,
                    size: wgpu::BufferSize::new(size),
                })
            };
            self.shape_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fh6-kit shape group"),
                layout: &self.shape_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buf(size_of::<Globals>() as u64),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: buf(PAINTS_CAP),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: buf(CLIPS_CAP),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: buf(POINTS_CAP),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&self.atlas.full),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(&t.backdrop.full),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(&t.bpyr.full),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::TextureView(world),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::Sampler(&self.sampler_rep),
                    },
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: wgpu::BindingResource::Sampler(&self.sampler_aniso),
                    },
                ],
            }));
        }
        self.shape_group.as_ref()
    }

    fn post_group(&mut self, device: &wgpu::Device, src: V, base: V) {
        if self.post_groups.contains_key(&(src, base)) {
            return;
        }
        let Some(t) = self.targets.as_ref() else { return };
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fh6-kit post group"),
            layout: &self.post_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.upload,
                        offset: 0,
                        size: wgpu::BufferSize::new(size_of::<PostParams>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(t.view(src)),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(t.view(base)),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        self.post_groups.insert((src, base), group);
    }

    /// Pushes one post pass's parameters and returns its dynamic offset.
    fn push_post(&mut self, post_base: u64, params: PostParams) -> Option<u32> {
        if self.post.len() >= MAX_POST {
            tracing::warn!("fh6-kit post pass budget exceeded; pass skipped");
            return None;
        }
        self.post.push(params);
        u32::try_from(post_base + (self.post.len() as u64 - 1) * SLOT).ok()
    }

    /// Records one full-viewport post pass from `src` (+ `base`) into the target view `dst`.
    #[allow(clippy::too_many_arguments)]
    fn post_pass(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        stats: &mut FrameStats,
        pipe: PostPipe,
        src: V,
        base: V,
        dst: V,
        viewport: (u32, u32),
        clear: bool,
        params: PostParams,
        post_base: u64,
    ) {
        let Some(offset) = self.push_post(post_base, params) else {
            return;
        };
        self.post_group(device, src, base);
        let (Some(t), Some(group)) = (self.targets.as_ref(), self.post_groups.get(&(src, base))) else {
            return;
        };
        let pipeline = match pipe {
            PostPipe::Down => &self.down,
            PostPipe::Blur => &self.blur,
            PostPipe::Composite => &self.composite,
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fh6-kit post"),
            color_attachments: &[Some(color_attachment(t.view(dst), clear))],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let (vw, vh) = viewport;
        pass.set_viewport(0.0, 0.0, vw.max(1) as f32, vh.max(1) as f32, 0.0, 1.0);
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, group, &[offset]);
        pass.draw(0..3, 0..1);
        stats.passes += 1;
        stats.draw_calls += 1;
    }

    fn texel(&self, v: V) -> [f32; 2] {
        let s = self.targets.as_ref().map_or((1, 1), |t| t.size(v));
        [1.0 / s.0.max(1) as f32, 1.0 / s.1.max(1) as f32]
    }

    /// Downsamples `src` (full resolution) to pyramid `level` of `chain`, then Gaussian-blurs it
    /// there with `sigma_px` (in full-resolution pixels). The result is in `chain` mip
    /// `level - 1`.
    #[allow(clippy::too_many_arguments)]
    fn blur_chain(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        stats: &mut FrameStats,
        src: V,
        chain: Chain,
        level: u32,
        sigma_px: f32,
        work: (u32, u32),
        uv_max: [f32; 2],
        post_base: u64,
    ) {
        let at = |m: u32| match chain {
            Chain::Pyr => V::Pyr(m),
            Chain::Backdrop => V::BPyr(m),
        };
        for m in 0..level {
            let from = if m == 0 { src } else { at(m - 1) };
            let to = at(m);
            let vp = (work.0.div_ceil(2 << m), work.1.div_ceil(2 << m));
            let params = PostParams {
                src_texel: self.texel(from),
                src_max: uv_max,
                dst_texel: self.texel(to),
                ..zero_params()
            };
            self.post_pass(
                device,
                encoder,
                stats,
                PostPipe::Down,
                from,
                from,
                to,
                vp,
                true,
                params,
                post_base,
            );
        }
        let m = level - 1;
        let vp = (work.0.div_ceil(2 << m), work.1.div_ceil(2 << m));
        let sigma = (sigma_px / (1u32 << level) as f32).max(0.3);
        for (from, to, dir) in [(at(m), V::Tmp(m), [1.0, 0.0]), (V::Tmp(m), at(m), [0.0, 1.0])] {
            let params = PostParams {
                src_texel: self.texel(from),
                src_max: uv_max,
                dst_texel: self.texel(to),
                dir,
                sigma,
                ..zero_params()
            };
            self.post_pass(
                device,
                encoder,
                stats,
                PostPipe::Blur,
                from,
                from,
                to,
                vp,
                true,
                params,
                post_base,
            );
        }
    }

    /// The bind group of one bloom dispatch: `src` and `base` sampled, `dst` written.
    fn bloom_group(&mut self, device: &wgpu::Device, src: V, base: V, dst: V) {
        if self.bloom_groups.contains_key(&(src, base, dst)) {
            return;
        }
        let Some(t) = self.targets.as_ref() else { return };
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fh6-kit bloom group"),
            layout: &self.bloom_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.upload,
                        offset: 0,
                        size: wgpu::BufferSize::new(size_of::<PostParams>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(t.view(src)),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(t.view(base)),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(t.view(dst)),
                },
            ],
        });
        self.bloom_groups.insert((src, base, dst), group);
    }

    /// Dual-filter bloom of the emissive target: result in `Tmp(0)` (half resolution). One
    /// compute pass: `PYRAMID_LEVELS` down dispatches, then `PYRAMID_LEVELS - 1` up dispatches,
    /// each the same maths as the former render pass of that level (`shaders/bloom.wgsl`).
    fn bloom(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        stats: &mut FrameStats,
        work: (u32, u32),
        uv_max: [f32; 2],
        post_base: u64,
    ) {
        const STEPS: usize = 2 * PYRAMID_LEVELS as usize - 1;
        /// One dispatch: up (or down), its parameter offset, (src, base, dst) and the texels it
        /// covers (the viewport and its zeroed margin).
        type Step = (bool, u32, (V, V, V), (u32, u32));
        let mut steps: [Option<Step>; STEPS] = [None; STEPS];
        let downs = (0..PYRAMID_LEVELS).map(|m| {
            let from = if m == 0 { V::Emissive } else { V::Pyr(m - 1) };
            (false, m, (from, from, V::Pyr(m)))
        });
        let ups = (0..PYRAMID_LEVELS - 1).rev().map(|m| {
            let from = if m == PYRAMID_LEVELS - 2 {
                V::Pyr(m + 1)
            } else {
                V::Tmp(m + 1)
            };
            (true, m, (from, V::Pyr(m), V::Tmp(m)))
        });
        for (slot, (up, m, views)) in steps.iter_mut().zip(downs.chain(ups)) {
            let vp = (work.0.div_ceil(2 << m), work.1.div_ceil(2 << m));
            let params = PostParams {
                src_texel: self.texel(views.0),
                src_max: uv_max,
                dst_texel: self.texel(views.2),
                // The bloom shaders read `origin` as the viewport, in destination texels.
                origin: [vp.0 as f32, vp.1 as f32],
                ..zero_params()
            };
            let Some(offset) = self.push_post(post_base, params) else {
                return;
            };
            self.bloom_group(device, views.0, views.1, views.2);
            // The render passes cleared their whole target; the dispatch zeroes a margin of
            // `BLOOM_MARGIN` texels round the viewport instead. Bilinear taps clamped to `src_max`
            // reach at most that far past it, so stale texels of an earlier, larger work
            // rectangle never bleed in.
            let size = self.targets.as_ref().map_or(vp, |t| t.size(views.2));
            let grid = ((vp.0 + BLOOM_MARGIN).min(size.0), (vp.1 + BLOOM_MARGIN).min(size.1));
            *slot = Some((up, offset, views, grid));
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("fh6-kit bloom"),
            timestamp_writes: None,
        });
        stats.passes += 1;
        for (up, offset, views, grid) in steps.into_iter().flatten() {
            let Some(group) = self.bloom_groups.get(&views) else {
                continue;
            };
            pass.set_pipeline(if up { &self.bloom_up } else { &self.bloom_down });
            pass.set_bind_group(0, group, &[offset]);
            pass.dispatch_workgroups(grid.0.max(1).div_ceil(8), grid.1.max(1).div_ceil(8), 1);
            stats.draw_calls += 1;
        }
    }

    /// Renders `scene` into `output` (cleared to transparent first, the cluster resolved inside
    /// its work rectangle). The caller submits `encoder`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        scene: &Scene,
        images: &Images,
        fonts: &mut Fonts,
        output: &wgpu::TextureView,
        output_format: wgpu::TextureFormat,
        target_size: (u32, u32),
        opacity: f32,
        bloom_margin_px: f32,
    ) -> FrameStats {
        let t0 = Stamp::now();
        let bloom_on = scene.bloom_used && self.bloom_enabled;
        let mut stats = FrameStats {
            instances: scene.instances.len() as u32,
            dropped: scene.dropped,
            ..Default::default()
        };
        self.upload_atlas(device, queue, fonts);
        let resolve_index = self.ensure_resolve(device, output_format);
        let (tw, th) = (target_size.0.max(1), target_size.1.max(1));
        let timer_on = self.timing_enabled && self.timer.is_some();
        let timeline_on = timer_on && self.timeline_enabled;
        self.timeline.clear();
        // Indices 0 and 1 are the frame's begin and end; the timeline's own start at 2.
        let mut next_query = 2_u32;

        // Work rectangle: what was drawn (+ bloom margin), snapped to 16 px, inside the target.
        let mut bounds = scene.bounds;
        if bloom_on {
            bounds = bounds.expand(bloom_margin_px);
        }
        let b = bounds.intersect(&Rect::from_xywh(0.0, 0.0, tw as f32, th as f32));
        let work = if b.is_empty() || scene.draws.is_empty() || opacity <= 0.0 {
            None
        } else {
            let x0 = ((b.min.x / 16.0).floor() * 16.0).max(0.0) as u32;
            let y0 = ((b.min.y / 16.0).floor() * 16.0).max(0.0) as u32;
            let x1 = (((b.max.x / 16.0).ceil() * 16.0) as u32).min(tw);
            let y1 = (((b.max.y / 16.0).ceil() * 16.0) as u32).min(th);
            (x1 > x0 && y1 > y0).then_some((x0, y0, x1 - x0, y1 - y0))
        };
        let Some((wx, wy, ww, wh)) = work else {
            let ts = self
                .timer
                .as_ref()
                .filter(|_| timer_on)
                .map(|t| wgpu::RenderPassTimestampWrites {
                    query_set: &t.set,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: Some(1),
                });
            drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fh6-kit clear"),
                color_attachments: &[Some(output_attachment(output, self.underlay * opacity.clamp(0.0, 1.0)))],
                depth_stencil_attachment: None,
                timestamp_writes: ts,
                occlusion_query_set: None,
                multiview_mask: None,
            }));
            self.queries_used = 2;
            self.finish_timer(encoder);
            stats.passes = 1;
            stats.cpu = t0.elapsed();
            return stats;
        };
        stats.work_pixels = u64::from(ww) * u64::from(wh);
        self.ensure_targets(device, (ww, wh), (tw, th));
        let Some(cap) = self.targets.as_ref().map(|t| t.cap) else {
            return stats;
        };
        let uv_max = [(ww as f32 - 0.5) / cap.0 as f32, (wh as f32 - 0.5) / cap.1 as f32];

        // ----- pack every data region into one staging buffer -------------------------------
        let world_rect = self
            .world
            .as_ref()
            .map_or(Rect::from_xywh(0.0, 0.0, 1.0, 1.0), |w| w.rect);
        let base_globals = Globals {
            target_size: [tw as f32, th as f32],
            work_origin: [wx as f32, wy as f32],
            work_size: [ww as f32, wh as f32],
            cap_size: [cap.0 as f32, cap.1 as f32],
            time: self.started.elapsed().as_secs_f32(),
            backdrop_lod: 0.0,
            backdrop_valid: 0.0,
            world_valid: if self.world.is_some() { 1.0 } else { 0.0 },
            world_origin: [world_rect.min.x, world_rect.min.y],
            world_size: [world_rect.width().max(1.0), world_rect.height().max(1.0)],
        };
        self.globals.clear();
        self.post.clear();
        for seg in &scene.segments {
            let mut g = base_globals;
            if let Some(s) = seg.backdrop_px {
                g.backdrop_valid = 1.0;
                // A layer with no frost keeps only the sharp snapshot (lod -1 = no blurred copy).
                g.backdrop_lod = if s >= MIN_FROST_PX {
                    (blur_level(s) - 1) as f32
                } else {
                    -1.0
                };
            }
            self.globals.push(g);
        }
        self.staging.clear();
        let mut offs = Offsets::default();
        for g in &self.globals {
            let at = self.staging.len();
            self.staging.extend_from_slice(bytemuck::bytes_of(g));
            self.staging.resize(at + SLOT as usize, 0);
        }
        offs.instances = push_region(&mut self.staging, bytemuck::cast_slice(&scene.instances));
        offs.paints = push_region(&mut self.staging, bytemuck::cast_slice(&scene.paints));
        offs.clips = push_region(&mut self.staging, bytemuck::cast_slice(&scene.clips));
        offs.points = push_region(&mut self.staging, bytemuck::cast_slice(&scene.points));
        if scene.points.is_empty() {
            // The point list binding needs at least one element.
            self.staging.extend_from_slice(&[0u8; 8]);
        }
        offs.post = align(self.staging.len() as u64);
        if !scene.meshes.is_empty() {
            self.meshes
                .prepare(queue, &scene.meshes, [wx as f32, wy as f32], [ww as f32, wh as f32]);
        }

        // The upload's copy goes first in the command buffer, before any pass reads it; the bytes
        // land in the mapped slot at the end of the frame, before the buffer is submitted.
        let staged = self.staging_ring.iter().position(|slot| slot.state() == SLOT_READY);
        if let Some(i) = staged {
            encoder.copy_buffer_to_buffer(&self.staging_ring[i].buffer, 0, &self.upload, 0, staged_len(offs.post));
        }

        // ----- record -----------------------------------------------------------------------
        let mut scene_cleared = false;
        let mut first_pass = true;
        for (si, seg) in scene.segments.iter().enumerate() {
            if let Some(sigma) = seg.backdrop_px {
                let level = blur_level(sigma);
                if scene_cleared {
                    if let Some(t) = self.targets.as_ref() {
                        encoder.copy_texture_to_texture(
                            texel_copy(&t.scene.texture_ref),
                            texel_copy(&t.backdrop.texture_ref),
                            wgpu::Extent3d {
                                width: ww,
                                height: wh,
                                depth_or_array_layers: 1,
                            },
                        );
                    }
                    if sigma >= MIN_FROST_PX {
                        self.blur_chain(
                            device,
                            encoder,
                            &mut stats,
                            V::Backdrop,
                            Chain::Backdrop,
                            level,
                            sigma,
                            (ww, wh),
                            uv_max,
                            offs.post,
                        );
                    }
                } else if let Some(t) = self.targets.as_ref() {
                    // Nothing is drawn yet, so the backdrop is empty. Cleared with render passes:
                    // `clear_texture` needs the optional CLEAR_TEXTURE feature (absent on WebGPU),
                    // which made a cluster whose first draw is its glass fail validation.
                    let frosted = if sigma >= MIN_FROST_PX {
                        t.bpyr.mips.as_slice()
                    } else {
                        &[]
                    };
                    for view in std::iter::once(&t.backdrop.full).chain(frosted) {
                        drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("fh6-kit clear backdrop"),
                            color_attachments: &[Some(color_attachment(view, true))],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        }));
                        stats.passes += 1;
                    }
                }
            }
            let into_layer = seg.blur_px > 0.0;
            let Some(group) = self.shape_group(device).cloned() else {
                break;
            };
            let Some(t) = self.targets.as_ref() else { break };
            let (begin, end) = if timeline_on {
                let begin = if first_pass { 0 } else { take_query(&mut next_query) };
                let end = take_query(&mut next_query);
                self.timeline.push(TimelineMark {
                    name: if seg.backdrop_px.is_some() {
                        "glass shapes"
                    } else {
                        "shapes"
                    },
                    segment: si as u32,
                    begin,
                    end,
                });
                (Some(begin), Some(end))
            } else {
                (first_pass.then_some(0), None)
            };
            let ts = self
                .timer
                .as_ref()
                .filter(|_| timer_on && (first_pass || timeline_on))
                .map(|tm| wgpu::RenderPassTimestampWrites {
                    query_set: &tm.set,
                    beginning_of_pass_write_index: begin,
                    end_of_pass_write_index: end,
                });
            first_pass = false;
            let dyn_offsets = [
                (offs.globals + si as u64 * SLOT) as u32,
                offs.paints as u32,
                offs.clips as u32,
                offs.points as u32,
            ];
            {
                let color_view = if into_layer { &t.layer.full } else { &t.scene.full };
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("fh6-kit shapes"),
                    color_attachments: &[
                        Some(color_attachment(color_view, into_layer || !scene_cleared)),
                        Some(color_attachment(&t.emissive.full, !scene_cleared)),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &t.depth.full,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Discard,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: ts,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                stats.passes += 1;
                pass.set_viewport(0.0, 0.0, ww as f32, wh as f32, 0.0, 1.0);
                pass.set_vertex_buffer(0, self.upload.slice(offs.instances..));
                let mut bound_blend: Option<BlendMode> = None;
                let mut bound_image: Option<Option<crate::paint::ImageId>> = None;
                let draws = scene
                    .draws
                    .get(seg.draws.start as usize..seg.draws.end as usize)
                    .unwrap_or(&[]);
                for cmd in draws {
                    match cmd {
                        DrawCmd::Shapes {
                            instances,
                            blend,
                            image,
                        } => {
                            if bound_blend != Some(*blend) {
                                if bound_blend.is_none() {
                                    pass.set_bind_group(0, &group, &dyn_offsets);
                                }
                                pass.set_pipeline(&self.shape_pipelines[blend.index()]);
                                bound_blend = Some(*blend);
                            }
                            if bound_image != Some(*image) {
                                if let Some(img) = images.bind_group(*image) {
                                    pass.set_bind_group(1, img, &[]);
                                }
                                bound_image = Some(*image);
                            }
                            pass.draw(0..4, instances.clone());
                            stats.draw_calls += 1;
                        }
                        DrawCmd::Mesh(i) => {
                            let Some(m) = scene.meshes.get(*i as usize) else {
                                continue;
                            };
                            let v = m.viewport;
                            let x0 = (v.min.x - wx as f32).floor().clamp(0.0, ww as f32) as u32;
                            let y0 = (v.min.y - wy as f32).floor().clamp(0.0, wh as f32) as u32;
                            let x1 = (v.max.x - wx as f32).ceil().clamp(0.0, ww as f32) as u32;
                            let y1 = (v.max.y - wy as f32).ceil().clamp(0.0, wh as f32) as u32;
                            if x1 <= x0 || y1 <= y0 {
                                continue;
                            }
                            pass.set_scissor_rect(x0, y0, x1 - x0, y1 - y0);
                            self.meshes.record(&mut pass, *i, m.draw.mesh);
                            pass.set_scissor_rect(0, 0, ww, wh);
                            // The mesh pass bound its own vertex buffer to slot 0;
                            // restore the shape instances for later draws in this segment.
                            pass.set_vertex_buffer(0, self.upload.slice(offs.instances..));
                            // The mesh pipeline replaced group 0 and the pipeline.
                            bound_blend = None;
                            bound_image = None;
                            stats.draw_calls += 1;
                        }
                    }
                }
            }
            scene_cleared = true;
            if into_layer {
                let level = blur_level(seg.blur_px);
                self.blur_chain(
                    device,
                    encoder,
                    &mut stats,
                    V::Layer,
                    Chain::Pyr,
                    level,
                    seg.blur_px,
                    (ww, wh),
                    uv_max,
                    offs.post,
                );
                let src = V::Pyr(level - 1);
                let params = PostParams {
                    src_texel: self.texel(src),
                    src_max: uv_max,
                    dst_texel: [1.0 / cap.0 as f32, 1.0 / cap.1 as f32],
                    opacity: 1.0,
                    ..zero_params()
                };
                self.post_pass(
                    device,
                    encoder,
                    &mut stats,
                    PostPipe::Composite,
                    src,
                    src,
                    V::Scene,
                    (ww, wh),
                    false,
                    params,
                    offs.post,
                );
            }
        }
        if bloom_on {
            self.bloom(device, encoder, &mut stats, (ww, wh), uv_max, offs.post);
        }

        // ----- resolve into the caller's target ---------------------------------------------
        let base = if bloom_on { V::Tmp(0) } else { V::Scene };
        let params = PostParams {
            src_texel: [1.0 / cap.0 as f32, 1.0 / cap.1 as f32],
            src_max: uv_max,
            dst_texel: [1.0 / cap.0 as f32, 1.0 / cap.1 as f32],
            // The resolve shader reads `dir` as the bloom sampling limit.
            dir: uv_max,
            intensity: 1.0 / PYRAMID_LEVELS as f32,
            opacity: opacity.clamp(0.0, 1.0),
            bloom_on: if bloom_on { 1.0 } else { 0.0 },
            origin: [wx as f32, wy as f32],
            // The resolve reads `sigma` as the underlay alpha.
            sigma: self.underlay,
            ..zero_params()
        };
        if let Some(offset) = self.push_post(offs.post, params) {
            self.post_group(device, V::Scene, base);
            if let Some(group) = self.post_groups.get(&(V::Scene, base)) {
                let begin = timeline_on.then(|| {
                    let begin = take_query(&mut next_query);
                    self.timeline.push(TimelineMark {
                        name: "resolve",
                        segment: 0,
                        begin,
                        end: 1,
                    });
                    begin
                });
                let ts = self
                    .timer
                    .as_ref()
                    .filter(|_| timer_on)
                    .map(|tm| wgpu::RenderPassTimestampWrites {
                        query_set: &tm.set,
                        beginning_of_pass_write_index: begin,
                        end_of_pass_write_index: Some(1),
                    });
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("fh6-kit resolve"),
                    color_attachments: &[Some(output_attachment(output, self.underlay * opacity.clamp(0.0, 1.0)))],
                    depth_stencil_attachment: None,
                    timestamp_writes: ts,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_viewport(wx as f32, wy as f32, ww as f32, wh as f32, 0.0, 1.0);
                if let Some((_, p)) = self.resolve.get(resolve_index) {
                    pass.set_pipeline(p);
                }
                pass.set_bind_group(0, group, &[offset]);
                pass.draw(0..3, 0..1);
                stats.passes += 1;
                stats.draw_calls += 1;
            }
        }
        self.queries_used = next_query;
        self.finish_timer(encoder);

        // ----- the single upload ------------------------------------------------------------
        self.staging.resize(offs.post as usize, 0);
        for p in &self.post {
            let at = self.staging.len();
            self.staging.extend_from_slice(bytemuck::bytes_of(p));
            self.staging.resize(at + SLOT as usize, 0);
        }
        match staged.and_then(|i| self.staging_ring.get(i)) {
            Some(slot) => {
                // The post parameters are capped at MAX_POST, so the bytes always fit the copy.
                let len = self.staging.len().min(staged_len(offs.post) as usize);
                let written = match slot.buffer.slice(..).get_mapped_range_mut() {
                    Ok(mut mapped) => {
                        mapped.slice(..len).copy_from_slice(&self.staging[..len]);
                        true
                    }
                    Err(err) => {
                        // A ready slot is mapped, so this does not happen. The copy is already
                        // recorded and the buffer is not mapped, so the submit stays valid; this
                        // one frame shows the slot's previous bytes, and the slot is retired.
                        tracing::warn!(error = %err, "fh6-kit upload staging slot was not mapped; slot retired");
                        false
                    }
                };
                if written {
                    slot.buffer.unmap();
                    slot.state.store(SLOT_WRITTEN, Ordering::Release);
                } else {
                    slot.state.store(SLOT_LOST, Ordering::Release);
                }
            }
            None => queue.write_buffer(&self.upload, 0, &self.staging),
        }
        stats.cpu = t0.elapsed();
        stats
    }

    fn finish_timer(&self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(t) = self.timer.as_ref().filter(|_| self.timing_enabled) {
            let used = self.queries_used.clamp(2, TIMER_QUERIES);
            encoder.resolve_query_set(&t.set, 0..used, &t.resolve, 0);
            encoder.copy_buffer_to_buffer(&t.resolve, 0, &t.readback, 0, u64::from(used) * 8);
        }
    }

    /// Reads the GPU time of the last submitted frame (blocks; benches and tests only).
    pub(crate) fn read_gpu_time(&self, device: &wgpu::Device) -> Option<Duration> {
        self.read_timestamps(device, |ticks, to_time| {
            let (a, b) = (*ticks.first()?, *ticks.get(1)?);
            Some(to_time(b.saturating_sub(a)))
        })
    }

    /// Reads every pass of the last submitted frame when the pass timeline is on (blocks;
    /// benches only). Times run from the start of the frame's first pass.
    pub(crate) fn read_gpu_timeline(&self, device: &wgpu::Device) -> Option<Vec<GpuPassTime>> {
        if !self.timeline_enabled {
            return None;
        }
        self.read_timestamps(device, |ticks, to_time| {
            let origin = *ticks.first()?;
            let at = |i: u32| ticks.get(i as usize).map(|t| to_time(t.saturating_sub(origin)));
            self.timeline
                .iter()
                .map(|m| {
                    Some(GpuPassTime {
                        name: m.name,
                        segment: m.segment,
                        start: at(m.begin)?,
                        end: at(m.end)?,
                    })
                })
                .collect()
        })
    }

    /// Maps the timestamp readback of the last submitted frame and hands `f` its raw ticks and a
    /// tick-to-time conversion.
    fn read_timestamps<T>(
        &self,
        device: &wgpu::Device,
        f: impl FnOnce(&[u64], &dyn Fn(u64) -> Duration) -> Option<T>,
    ) -> Option<T> {
        let t = self.timer.as_ref().filter(|_| self.timing_enabled)?;
        let (tx, rx) = std::sync::mpsc::channel();
        t.readback.map_async(wgpu::MapMode::Read, .., move |r| {
            let _ = tx.send(r);
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(5)),
            })
            .ok()?;
        rx.recv_timeout(Duration::from_secs(5)).ok()?.ok()?;
        let used = self.queries_used.clamp(2, TIMER_QUERIES) as usize;
        let ticks: Vec<u64> = {
            let view = t.readback.get_mapped_range(..).ok()?;
            view.get(..used * 8)?
                .chunks_exact(8)
                .map(|c| c.try_into().map_or(0, u64::from_le_bytes))
                .collect()
        };
        t.readback.unmap();
        let period = f64::from(t.period_ns);
        let to_time = |ticks: u64| Duration::from_nanos((ticks as f64 * period) as u64);
        f(&ticks, &to_time)
    }
}

#[derive(Debug, Clone, Copy)]
enum Chain {
    Pyr,
    Backdrop,
}

#[derive(Debug, Clone, Copy)]
enum PostPipe {
    Down,
    Blur,
    Composite,
}

/// The next free timestamp index, which it then takes.
fn take_query(next: &mut u32) -> u32 {
    let index = *next;
    *next += 1;
    index
}

fn zero_params() -> PostParams {
    PostParams::zeroed()
}

/// Appends `bytes` at the next 256-byte boundary; returns its offset.
fn push_region(staging: &mut Vec<u8>, bytes: &[u8]) -> u64 {
    let at = align(staging.len() as u64);
    staging.resize(at as usize, 0);
    staging.extend_from_slice(bytes);
    at
}

fn texel_copy(texture: &wgpu::Texture) -> wgpu::TexelCopyTextureInfo<'_> {
    wgpu::TexelCopyTextureInfo {
        texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
    }
}

/// Pyramid level (1 = half resolution) at which a Gaussian of `sigma_px` is blurred with a
/// sigma of at most ~2 texels.
fn blur_level(sigma_px: f32) -> u32 {
    let s = sigma_px.max(0.5);
    ((s / 2.0).log2().ceil().max(1.0) as u32).min(PYRAMID_LEVELS)
}

/// The caller's target, cleared to the premultiplied black underlay (`alpha` 0 = transparent).
fn output_attachment(view: &wgpu::TextureView, alpha: f32) -> wgpu::RenderPassColorAttachment<'_> {
    wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: f64::from(alpha),
            }),
            store: wgpu::StoreOp::Store,
        },
    }
}

fn color_attachment(view: &wgpu::TextureView, clear: bool) -> wgpu::RenderPassColorAttachment<'_> {
    wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: if clear {
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
            } else {
                wgpu::LoadOp::Load
            },
            store: wgpu::StoreOp::Store,
        },
    }
}

/// The bloom chain's layout and its two compute pipelines (`shaders/bloom.wgsl`).
fn make_bloom(device: &wgpu::Device) -> (wgpu::BindGroupLayout, wgpu::ComputePipeline, wgpu::ComputePipeline) {
    let sampled = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("fh6-kit bloom layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(size_of::<PostParams>() as u64),
                },
                count: None,
            },
            sampled(1),
            sampled(2),
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: SCENE_FORMAT,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("fh6-kit bloom pipeline layout"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fh6-kit bloom shaders"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/bloom.wgsl").into()),
    });
    let pipe = |entry: &str| {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(entry),
            layout: Some(&layout),
            module: &module,
            entry_point: Some(entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    };
    let down = pipe("cs_down");
    let up = pipe("cs_up");
    (bgl, down, up)
}

fn make_post(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    entry: &str,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(entry),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("vs_full"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_fit_slots() {
        assert!(size_of::<Globals>() as u64 <= SLOT);
        assert!(size_of::<PostParams>() as u64 <= SLOT);
        assert_eq!(size_of::<Globals>() % 16, 0);
        assert_eq!(size_of::<PostParams>() % 16, 0);
    }

    #[test]
    fn blur_levels() {
        assert_eq!(blur_level(1.0), 1);
        assert_eq!(blur_level(4.0), 1);
        assert_eq!(blur_level(8.0), 2);
        assert_eq!(blur_level(30.0), 4);
        assert_eq!(blur_level(1000.0), PYRAMID_LEVELS);
    }

    #[test]
    fn a_staged_copy_covers_every_post_slot_and_stays_inside_the_upload() {
        // The copy is recorded before the post parameters are known, so it covers all of them.
        let post = align(GLOBALS_CAP + INSTANCES_CAP);
        assert_eq!(staged_len(post), post + POST_CAP);
        assert!(post + POST_CAP <= UPLOAD_SIZE);
        // Never past the end, and a multiple of the 4-byte copy alignment.
        assert_eq!(staged_len(UPLOAD_SIZE), UPLOAD_SIZE);
        assert_eq!(staged_len(post) % wgpu::COPY_BUFFER_ALIGNMENT, 0);
        assert_eq!(UPLOAD_SIZE % wgpu::COPY_BUFFER_ALIGNMENT, 0);
    }

    #[test]
    fn upload_holds_worst_case() {
        let worst = GLOBALS_CAP + INSTANCES_CAP + PAINTS_CAP + CLIPS_CAP + POINTS_CAP + 5 * SLOT + POST_CAP;
        assert!(worst <= UPLOAD_SIZE);
    }
}
