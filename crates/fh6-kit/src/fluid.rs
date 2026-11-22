//! GPU 2D stable fluids (Jos Stam, "Stable Fluids", 1999) for liquid-metal, plasma and smoke
//! looks: compute shaders only, so it runs on DX12 and in a browser (WebGPU core).
//!
//! Each [`Fluid::step`] records one compute pass of small dispatches:
//!
//! 1. velocity splats (skipped when none are queued),
//! 2. vorticity confinement (curl, then the confinement force; skipped at strength 0),
//! 3. projection: divergence, a warm-started pressure ([`FluidConfig::pressure_decay`]), then
//!    [`FluidConfig::pressure_iterations`] Jacobi iterations and the gradient subtraction,
//! 4. semi-Lagrangian advection of velocity, with dissipation,
//! 5. dye splats and dye advection, with dissipation.
//!
//! Every field is an `rgba16float` texture written as a write-only storage texture and read
//! with `textureLoad` / `textureSampleLevel` (no implicit derivatives), ping-ponged so no
//! dispatch reads what it writes. All bind groups are built once in [`Fluid::new`]; a step makes
//! one uniform write (parameters plus up to [`MAX_SPLATS`] splats) and no heap allocation of its
//! own.
//!
//! The dye always ends a step in the same texture, so [`Fluid::dye_view`] is stable and can be
//! registered once as an image. It holds **linear premultiplied** RGBA (alpha = density in
//! `0..=1`), the kit's image convention, so it can be drawn and filtered directly.
//!
//! # Using it from a cluster
//!
//! ```ignore
//! // In the cluster's constructor (`crate::cluster::Setup` gives device, queue, images):
//! let fluid = Fluid::new(setup.device, 256, 96, FluidConfig::default());
//! let dye = setup.images.register_view(setup.device, fluid.dye_view(), fluid.size());
//!
//! // Each frame, before drawing (Cluster::update has no encoder, so use step_submit):
//! fluid.splat(vec2(0.1, 0.5), vec2(0.8, 0.0), accent, 0.06);
//! fluid.step_submit(device, queue, dt);
//!
//! // In draw:
//! p.fill(&Shape::rect(zone), ImagePaint::new(dye, zone));
//! ```
//!
//! [`Fluid::step_submit`] creates a command encoder and submits it on its own: one extra small
//! queue submission per frame (tens of microseconds of CPU on DX12, plus the command-buffer
//! bookkeeping wgpu allocates internally per encoder). The GPU work is about 30 tiny dispatches
//! (`8x8` workgroups) at the default 20 Jacobi iterations; at 256x128 that is well under 0.1 ms
//! on any discrete GPU. When a host already has an encoder, call [`Fluid::step`] instead and the
//! work rides in that frame's submission. Submissions execute in order, so a step submitted
//! before the frame's render submission is visible to that frame.
use crate::color::Color;
use crate::math::{finite_or, Vec2};
use bytemuck::{Pod, Zeroable};

/// Most splats one step applies; further [`Fluid::splat`] calls in the same step are dropped.
pub const MAX_SPLATS: usize = 16;
/// Smallest grid side.
pub const MIN_SIZE: u32 = 8;
/// Largest grid side.
pub const MAX_SIZE: u32 = 512;
/// Most Jacobi iterations per step.
pub const MAX_PRESSURE_ITERATIONS: u32 = 200;
/// Longest `dt` a step integrates (seconds); longer frames are clamped.
pub const MAX_DT: f32 = 1.0 / 15.0;
/// Workgroup side in `fluid.wgsl`.
const WORKGROUP: u32 = 8;
/// The field format (storage-writable and filterable in WebGPU core).
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Solver settings. Out-of-range values are clamped by [`FluidConfig::sanitized`] (applied
/// automatically by [`Fluid`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FluidConfig {
    /// Jacobi iterations per step, `1..=MAX_PRESSURE_ITERATIONS` (more = less compressible).
    pub pressure_iterations: u32,
    /// Velocity decay rate in 1/s, `0..=50` (each step divides by `1 + rate * dt`).
    pub velocity_dissipation: f32,
    /// Dye decay rate in 1/s, `0..=50`.
    pub dye_dissipation: f32,
    /// Vorticity confinement strength, `0..=200` (0 = off; 10..40 gives curly smoke).
    pub vorticity: f32,
    /// Fraction of last step's pressure kept as the solver's starting guess, `0..=1`.
    pub pressure_decay: f32,
}

impl Default for FluidConfig {
    fn default() -> Self {
        Self {
            pressure_iterations: 20,
            velocity_dissipation: 0.2,
            dye_dissipation: 1.0,
            vorticity: 0.0,
            pressure_decay: 0.8,
        }
    }
}

impl FluidConfig {
    /// The config with every field finite and inside its documented range.
    #[must_use]
    pub fn sanitized(self) -> Self {
        let d = Self::default();
        Self {
            pressure_iterations: self.pressure_iterations.clamp(1, MAX_PRESSURE_ITERATIONS),
            velocity_dissipation: finite_or(self.velocity_dissipation, d.velocity_dissipation).clamp(0.0, 50.0),
            dye_dissipation: finite_or(self.dye_dissipation, d.dye_dissipation).clamp(0.0, 50.0),
            vorticity: finite_or(self.vorticity, 0.0).clamp(0.0, 200.0),
            pressure_decay: finite_or(self.pressure_decay, d.pressure_decay).clamp(0.0, 1.0),
        }
    }
}

/// One splat as the shader reads it (`Splat` in `fluid.wgsl`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
struct GpuSplat {
    /// Centre in cells (xy), radius in cells (z).
    pos_radius: [f32; 4],
    /// Velocity impulse in cells per second (xy).
    force: [f32; 4],
    /// Linear premultiplied dye.
    color: [f32; 4],
}

/// The uniform block (`Params` in `fluid.wgsl`): 48 bytes of header, then the splats.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
struct Params {
    size: [f32; 2],
    inv_size: [f32; 2],
    dt: f32,
    velocity_dissipation: f32,
    dye_dissipation: f32,
    vorticity: f32,
    pressure_decay: f32,
    splat_count: u32,
    _pad: [f32; 2],
    splats: [GpuSplat; MAX_SPLATS],
}

impl Params {
    fn new(width: u32, height: u32) -> Self {
        let (w, h) = (width as f32, height as f32);
        Self {
            size: [w, h],
            inv_size: [1.0 / w, 1.0 / h],
            ..Self::zeroed()
        }
    }

    /// Queues a splat given in UV space; returns `false` when the queue is full.
    fn push_splat(&mut self, pos_uv: Vec2, force: Vec2, color: Color, radius: f32) -> bool {
        let i = self.splat_count as usize;
        let Some(slot) = self.splats.get_mut(i) else {
            return false;
        };
        let [w, h] = self.size;
        let pos = pos_uv.sanitized();
        let force = force.sanitized();
        let radius_cells = (finite_or(radius, 0.0).max(0.0) * w.min(h)).clamp(0.5, w.max(h));
        let c = |v: f32| finite_or(v, 0.0).max(0.0);
        *slot = GpuSplat {
            pos_radius: [pos.x * w, pos.y * h, radius_cells, 0.0],
            force: [force.x * w, force.y * h, 0.0, 0.0],
            color: [c(color.r), c(color.g), c(color.b), c(color.a)],
        };
        self.splat_count += 1;
        true
    }
}

/// The size of the uniform block in bytes.
const PARAMS_SIZE: u64 = std::mem::size_of::<Params>() as u64;

#[derive(Debug)]
struct Pipelines {
    splat_velocity: wgpu::ComputePipeline,
    splat_dye: wgpu::ComputePipeline,
    curl: wgpu::ComputePipeline,
    vorticity: wgpu::ComputePipeline,
    divergence: wgpu::ComputePipeline,
    pressure_decay: wgpu::ComputePipeline,
    jacobi: wgpu::ComputePipeline,
    gradient_subtract: wgpu::ComputePipeline,
    advect_velocity: wgpu::ComputePipeline,
    advect_dye: wgpu::ComputePipeline,
}

/// Every bind group a step uses, built once. Index `[i]` = the current velocity (or pressure)
/// texture; the pass writes the other one.
#[derive(Debug)]
struct BindGroups {
    /// A = vel[i] -> vel[1 - i].
    splat_velocity: [wgpu::BindGroup; 2],
    /// A = vel[i] -> curl.
    curl: [wgpu::BindGroup; 2],
    /// A = vel[i], B = curl -> vel[1 - i].
    vorticity: [wgpu::BindGroup; 2],
    /// A = vel[i] -> div.
    divergence: [wgpu::BindGroup; 2],
    /// A = p[j], B = div -> p[1 - j] (decay and Jacobi).
    pressure: [wgpu::BindGroup; 2],
    /// `[i][j]`: A = vel[i], B = p[j] -> vel[1 - i].
    gradient: [[wgpu::BindGroup; 2]; 2],
    /// A = B = vel[i] -> vel[1 - i].
    advect_velocity: [wgpu::BindGroup; 2],
    /// A = dye0 -> dye1.
    splat_dye: wgpu::BindGroup,
    /// A = dye1, B = vel[i] -> dye0.
    advect_dye: [wgpu::BindGroup; 2],
}

/// A GPU stable-fluids simulation with a dye field the kit can draw as an image.
#[derive(Debug)]
pub struct Fluid {
    size: (u32, u32),
    config: FluidConfig,
    params: Params,
    uniform: wgpu::Buffer,
    pipelines: Pipelines,
    groups: BindGroups,
    /// Current velocity texture index.
    vel: usize,
    /// Current pressure texture index.
    pressure: usize,
    dye_texture: wgpu::Texture,
    dye_view: wgpu::TextureView,
    /// Keeps the internal fields alive alongside the bind groups that reference them.
    _fields: Vec<wgpu::Texture>,
}

fn field(device: &wgpu::Device, label: &str, size: (u32, u32), extra: wgpu::TextureUsages) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | extra,
        view_formats: &[],
    })
}

fn view(t: &wgpu::Texture) -> wgpu::TextureView {
    t.create_view(&wgpu::TextureViewDescriptor::default())
}

fn layout_entry(binding: u32, ty: wgpu::BindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty,
        count: None,
    }
}

fn sampled() -> wgpu::BindingType {
    wgpu::BindingType::Texture {
        sample_type: wgpu::TextureSampleType::Float { filterable: true },
        view_dimension: wgpu::TextureViewDimension::D2,
        multisampled: false,
    }
}

impl Fluid {
    /// A fluid on a `width` x `height` grid (each clamped to `MIN_SIZE..=MAX_SIZE` and the
    /// device limit), starting at rest with no dye. Allocates every texture, pipeline and bind
    /// group up front.
    #[must_use]
    pub fn new(device: &wgpu::Device, width: u32, height: u32, config: FluidConfig) -> Self {
        let max = MAX_SIZE.min(device.limits().max_texture_dimension_2d).max(MIN_SIZE);
        let size = (width.clamp(MIN_SIZE, max), height.clamp(MIN_SIZE, max));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fh6-kit fluid"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/fluid.wgsl").into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fh6-kit fluid layout"),
            entries: &[
                layout_entry(
                    0,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(PARAMS_SIZE),
                    },
                ),
                layout_entry(1, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)),
                layout_entry(2, sampled()),
                layout_entry(3, sampled()),
                layout_entry(
                    4,
                    wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: FORMAT,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fh6-kit fluid pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some(entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let pipelines = Pipelines {
            splat_velocity: pipeline("splat_velocity"),
            splat_dye: pipeline("splat_dye"),
            curl: pipeline("curl"),
            vorticity: pipeline("vorticity"),
            divergence: pipeline("divergence"),
            pressure_decay: pipeline("pressure_decay"),
            jacobi: pipeline("jacobi"),
            gradient_subtract: pipeline("gradient_subtract"),
            advect_velocity: pipeline("advect_velocity"),
            advect_dye: pipeline("advect_dye"),
        };

        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fh6-kit fluid params"),
            size: PARAMS_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fh6-kit fluid linear clamp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let none = wgpu::TextureUsages::empty();
        let vel_t = [
            field(device, "fh6-kit fluid velocity 0", size, none),
            field(device, "fh6-kit fluid velocity 1", size, none),
        ];
        let p_t = [
            field(device, "fh6-kit fluid pressure 0", size, none),
            field(device, "fh6-kit fluid pressure 1", size, none),
        ];
        let div_t = field(device, "fh6-kit fluid divergence", size, none);
        let curl_t = field(device, "fh6-kit fluid curl", size, none);
        let dye0 = field(device, "fh6-kit fluid dye 0", size, wgpu::TextureUsages::COPY_SRC);
        let dye1 = field(device, "fh6-kit fluid dye 1", size, wgpu::TextureUsages::COPY_SRC);

        let vel = [view(&vel_t[0]), view(&vel_t[1])];
        let p = [view(&p_t[0]), view(&p_t[1])];
        let div = view(&div_t);
        let curl = view(&curl_t);
        let dye_view = view(&dye0);
        let dye1_view = view(&dye1);

        let group = |a: &wgpu::TextureView, b: &wgpu::TextureView, dst: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fh6-kit fluid"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(a),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(b),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(dst),
                    },
                ],
            })
        };
        // Unused `B` slots get a texture the pass does not write (no usage conflicts).
        let groups = BindGroups {
            splat_velocity: [0, 1].map(|i| group(&vel[i], &div, &vel[1 - i])),
            curl: [0, 1].map(|i| group(&vel[i], &div, &curl)),
            vorticity: [0, 1].map(|i| group(&vel[i], &curl, &vel[1 - i])),
            divergence: [0, 1].map(|i| group(&vel[i], &curl, &div)),
            pressure: [0, 1].map(|j| group(&p[j], &div, &p[1 - j])),
            gradient: [0, 1].map(|i| [0, 1].map(|j| group(&vel[i], &p[j], &vel[1 - i]))),
            advect_velocity: [0, 1].map(|i| group(&vel[i], &vel[i], &vel[1 - i])),
            splat_dye: group(&dye_view, &div, &dye1_view),
            advect_dye: [0, 1].map(|i| group(&dye1_view, &vel[i], &dye_view)),
        };

        let [v0, v1] = vel_t;
        let [p0, p1] = p_t;
        Self {
            size,
            config: config.sanitized(),
            params: Params::new(size.0, size.1),
            uniform,
            pipelines,
            groups,
            vel: 0,
            pressure: 0,
            dye_texture: dye0,
            dye_view,
            _fields: vec![v0, v1, p0, p1, div_t, curl_t, dye1],
        }
    }

    /// Grid size in cells (also the dye texture size).
    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// The solver settings in use.
    #[must_use]
    pub fn config(&self) -> FluidConfig {
        self.config
    }

    /// Changes the solver settings (clamped) from the next step on.
    pub fn set_config(&mut self, config: FluidConfig) {
        self.config = config.sanitized();
    }

    /// The dye: `rgba16float`, linear premultiplied, alpha = density. The same view after every
    /// step, so register it once with `Images::register_view`.
    #[must_use]
    pub fn dye_view(&self) -> &wgpu::TextureView {
        &self.dye_view
    }

    /// The dye texture behind [`Fluid::dye_view`] (`COPY_SRC`, for readback and captures).
    #[must_use]
    pub fn dye_texture(&self) -> &wgpu::Texture {
        &self.dye_texture
    }

    /// Splats queued for the next step.
    #[must_use]
    pub fn pending_splats(&self) -> usize {
        self.params.splat_count as usize
    }

    /// Queues a splat for the next step: at `pos_uv` (`0..1`, y down) add the velocity `force`
    /// (UV units per second, e.g. `(0.5, 0)` crosses half the field per second) and the dye
    /// `color` (linear premultiplied), both with a Gaussian falloff of `radius` (a fraction of
    /// the grid's shorter side). Returns `false`, dropping the splat, when [`MAX_SPLATS`] are
    /// already queued.
    pub fn splat(&mut self, pos_uv: Vec2, force: Vec2, color: Color, radius: f32) -> bool {
        self.params.push_splat(pos_uv, force, color, radius)
    }

    /// Records one simulation step of `dt` seconds (clamped to `0..=MAX_DT`) into `encoder`,
    /// applying and then clearing the queued splats. One `queue.write_buffer` of the uniform
    /// block; no heap allocation of its own.
    pub fn step(&mut self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, dt: f32) {
        let cfg = self.config;
        let params = &mut self.params;
        params.dt = finite_or(dt, 0.0).clamp(0.0, MAX_DT);
        params.velocity_dissipation = cfg.velocity_dissipation;
        params.dye_dissipation = cfg.dye_dissipation;
        params.vorticity = cfg.vorticity;
        params.pressure_decay = cfg.pressure_decay;
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&self.params));
        let splats = self.params.splat_count > 0;
        self.params.splat_count = 0;

        let wg = (self.size.0.div_ceil(WORKGROUP), self.size.1.div_ceil(WORKGROUP));
        let pl = &self.pipelines;
        let g = &self.groups;
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("fh6-kit fluid step"),
            timestamp_writes: None,
        });
        let mut run = |pipeline: &wgpu::ComputePipeline, group: &wgpu::BindGroup| {
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.dispatch_workgroups(wg.0, wg.1, 1);
        };

        let (mut v, mut p) = (self.vel, self.pressure);
        if splats {
            run(&pl.splat_velocity, &g.splat_velocity[v]);
            v = 1 - v;
        }
        if cfg.vorticity > 0.0 {
            run(&pl.curl, &g.curl[v]);
            run(&pl.vorticity, &g.vorticity[v]);
            v = 1 - v;
        }
        run(&pl.divergence, &g.divergence[v]);
        run(&pl.pressure_decay, &g.pressure[p]);
        p = 1 - p;
        for _ in 0..cfg.pressure_iterations {
            run(&pl.jacobi, &g.pressure[p]);
            p = 1 - p;
        }
        run(&pl.gradient_subtract, &g.gradient[v][p]);
        v = 1 - v;
        run(&pl.advect_velocity, &g.advect_velocity[v]);
        v = 1 - v;
        // Always both dye passes, so the dye ends in the same (registered) texture every step.
        run(&pl.splat_dye, &g.splat_dye);
        run(&pl.advect_dye, &g.advect_dye[v]);

        self.vel = v;
        self.pressure = p;
    }

    /// [`Fluid::step`] in its own command encoder, submitted immediately: for hosts (like
    /// `Cluster::update`) that have no encoder. Costs one extra small queue submission per call;
    /// see the module docs.
    pub fn step_submit(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, dt: f32) -> wgpu::SubmissionIndex {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fh6-kit fluid step"),
        });
        self.step(queue, &mut encoder, dt);
        queue.submit(Some(encoder.finish()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vec2;
    use std::mem::{align_of, offset_of, size_of};

    #[test]
    fn uniform_layout_matches_wgsl() {
        assert_eq!(size_of::<GpuSplat>(), 48);
        assert_eq!(offset_of!(Params, size), 0);
        assert_eq!(offset_of!(Params, inv_size), 8);
        assert_eq!(offset_of!(Params, dt), 16);
        assert_eq!(offset_of!(Params, velocity_dissipation), 20);
        assert_eq!(offset_of!(Params, dye_dissipation), 24);
        assert_eq!(offset_of!(Params, vorticity), 28);
        assert_eq!(offset_of!(Params, pressure_decay), 32);
        assert_eq!(offset_of!(Params, splat_count), 36);
        assert_eq!(offset_of!(Params, splats), 48);
        assert_eq!(size_of::<Params>(), 48 + 48 * MAX_SPLATS);
        assert_eq!(PARAMS_SIZE % 16, 0, "uniform blocks are 16-byte multiples");
        assert!(align_of::<Params>() <= 16);
    }

    #[test]
    fn config_is_clamped() {
        let c = FluidConfig {
            pressure_iterations: 0,
            velocity_dissipation: f32::NAN,
            dye_dissipation: -3.0,
            vorticity: 1e9,
            pressure_decay: 2.0,
        }
        .sanitized();
        assert_eq!(c.pressure_iterations, 1);
        assert_eq!(c.velocity_dissipation, FluidConfig::default().velocity_dissipation);
        assert_eq!(c.dye_dissipation, 0.0);
        assert_eq!(c.vorticity, 200.0);
        assert_eq!(c.pressure_decay, 1.0);
        let c = FluidConfig {
            pressure_iterations: 10_000,
            vorticity: f32::INFINITY,
            ..FluidConfig::default()
        }
        .sanitized();
        assert_eq!(c.pressure_iterations, MAX_PRESSURE_ITERATIONS);
        assert_eq!(c.vorticity, 0.0);
        assert_eq!(FluidConfig::default().sanitized(), FluidConfig::default());
    }

    #[test]
    fn splat_queue_is_bounded() {
        let mut p = Params::new(128, 64);
        for i in 0..MAX_SPLATS {
            assert!(
                p.push_splat(vec2(0.5, 0.5), vec2(1.0, 0.0), Color::WHITE, 0.1),
                "splat {i}"
            );
        }
        assert_eq!(p.splat_count as usize, MAX_SPLATS);
        assert!(!p.push_splat(vec2(0.5, 0.5), vec2(1.0, 0.0), Color::WHITE, 0.1));
        assert_eq!(p.splat_count as usize, MAX_SPLATS);
    }

    #[test]
    fn splat_converts_uv_to_cells() {
        let mut p = Params::new(128, 64);
        assert!(p.push_splat(vec2(0.25, 0.5), vec2(0.5, -1.0), Color::linear(1.0, 0.5, 0.0, 0.5), 0.1));
        let s = p.splats[0];
        assert_eq!(s.pos_radius[0], 32.0);
        assert_eq!(s.pos_radius[1], 32.0);
        assert!(
            (s.pos_radius[2] - 6.4).abs() < 1e-5,
            "radius is a fraction of the short side"
        );
        assert_eq!(s.force[0], 64.0);
        assert_eq!(s.force[1], -64.0);
        assert_eq!(s.color, [0.5, 0.25, 0.0, 0.5]);
        // Hostile input stays finite, radius at least half a cell.
        assert!(p.push_splat(vec2(f32::NAN, 0.0), vec2(f32::INFINITY, 0.0), Color::WHITE, f32::NAN));
        let s = p.splats[1];
        assert!(s
            .pos_radius
            .iter()
            .chain(&s.force)
            .chain(&s.color)
            .all(|v| v.is_finite()));
        assert_eq!(s.pos_radius[2], 0.5);
    }
}
