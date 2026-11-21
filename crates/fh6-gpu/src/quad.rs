//! The minimal instanced pipeline: rounded rectangles and seven-segment glyphs.
//!
//! One draw call per frame. Instances are written into a buffer allocated once at startup;
//! [`QuadBatch`] is a fixed-capacity list that never reallocates, so building a frame does no
//! heap allocation.
use bytemuck::{Pod, Zeroable};
use std::mem::size_of;

/// Most quads one frame can draw. Pushes beyond this are dropped and counted.
pub const MAX_QUADS: usize = 1024;

/// What a quad draws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuadKind {
    /// A rounded rectangle, filled or as a rim.
    RoundedRect {
        /// Corner radius in pixels.
        radius: f32,
        /// Rim width in pixels; 0 fills the rectangle.
        rim: f32,
    },
    /// A seven-segment glyph filling the rect.
    Segments {
        /// Segment thickness in pixels.
        thickness: f32,
        /// Lit segments: a=1 b=2 c=4 d=8 e=16 f=32 g=64.
        mask: u8,
    },
}

/// One instance as the shader sees it.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct QuadInstance {
    /// x, y, w, h in target pixels.
    pub rect: [f32; 4],
    /// Straight-alpha, sRGB-encoded colour.
    pub color: [f32; 4],
    /// Kind-specific parameters (see `quad.wgsl`).
    pub params: [f32; 4],
}

fn finite_or(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

impl QuadInstance {
    /// Builds an instance; NaN or infinite inputs are replaced so nothing degenerate reaches
    /// the GPU, and sizes are at least one pixel.
    pub fn new(rect: [f32; 4], color: [f32; 4], kind: QuadKind) -> Self {
        let rect = [
            finite_or(rect[0], 0.0),
            finite_or(rect[1], 0.0),
            finite_or(rect[2], 1.0).max(1.0),
            finite_or(rect[3], 1.0).max(1.0),
        ];
        let color = color.map(|c| finite_or(c, 0.0).clamp(0.0, 1.0));
        let params = match kind {
            QuadKind::RoundedRect { radius, rim } => {
                [finite_or(radius, 0.0).max(0.0), 0.0, 0.0, finite_or(rim, 0.0).max(0.0)]
            }
            QuadKind::Segments { thickness, mask } => {
                [finite_or(thickness, 1.0).max(1.0), 1.0, f32::from(mask & 0x7F), 0.0]
            }
        };
        Self { rect, color, params }
    }
}

/// Per-frame uniforms.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Globals {
    /// Target size in pixels.
    pub viewport: [f32; 2],
    /// Overall opacity (visibility fade), 0..=1.
    pub opacity: f32,
    _pad: f32,
}

impl Globals {
    /// Uniforms for a `width` x `height` target at `opacity`.
    pub fn new(width: u32, height: u32, opacity: f32) -> Self {
        Self {
            viewport: [width.max(1) as f32, height.max(1) as f32],
            opacity: finite_or(opacity, 0.0).clamp(0.0, 1.0),
            _pad: 0.0,
        }
    }
}

/// A fixed-capacity list of quads for one frame.
#[derive(Debug, Clone)]
pub struct QuadBatch {
    items: Vec<QuadInstance>,
    dropped: usize,
}

impl Default for QuadBatch {
    fn default() -> Self {
        Self::new()
    }
}

impl QuadBatch {
    /// Allocates room for [`MAX_QUADS`] once.
    pub fn new() -> Self {
        Self {
            items: Vec::with_capacity(MAX_QUADS),
            dropped: 0,
        }
    }

    /// Empties the batch, keeping its storage.
    pub fn clear(&mut self) {
        self.items.clear();
        self.dropped = 0;
    }

    /// Adds a quad; beyond [`MAX_QUADS`] it is dropped (and counted) instead of reallocating.
    pub fn push(&mut self, quad: QuadInstance) {
        if self.items.len() < MAX_QUADS {
            self.items.push(quad);
        } else {
            self.dropped += 1;
        }
    }

    /// The quads in draw order.
    pub fn as_slice(&self) -> &[QuadInstance] {
        &self.items
    }

    /// Number of quads.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Quads dropped this frame because the batch was full.
    pub fn dropped(&self) -> usize {
        self.dropped
    }
}

/// The pipeline plus its uniform and instance buffers.
#[derive(Debug)]
pub struct QuadRenderer {
    pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    instances: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    count: u32,
    /// Last uniforms uploaded; they rarely change, so most frames skip the upload.
    uploaded_globals: Option<Globals>,
    format: wgpu::TextureFormat,
}

impl QuadRenderer {
    /// Builds the pipeline for render targets of `format`.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fh6 quad shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/quad.wgsl").into()),
        });
        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fh6 quad globals layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(size_of::<Globals>() as u64),
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fh6 quad layout"),
            bind_group_layouts: &[Some(&bind_layout)],
            immediate_size: 0,
        });
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] = [
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
        ];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fh6 quad pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<QuadInstance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &ATTRIBUTES,
                })],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fh6 quad globals"),
            size: size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fh6 quad instances"),
            size: (size_of::<QuadInstance>() * MAX_QUADS) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fh6 quad globals"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            }],
        });
        Self {
            pipeline,
            globals,
            instances,
            bind_group,
            count: 0,
            uploaded_globals: None,
            format,
        }
    }

    /// The target format this pipeline was built for.
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Uploads this frame's uniforms and quads.
    pub fn prepare(&mut self, queue: &wgpu::Queue, globals: &Globals, batch: &QuadBatch) {
        if self.uploaded_globals.as_ref() != Some(globals) {
            queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(globals));
            self.uploaded_globals = Some(*globals);
        }
        let quads = batch.as_slice();
        if !quads.is_empty() {
            queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(quads));
        }
        self.count = u32::try_from(quads.len()).unwrap_or(0);
    }

    /// Records one render pass into `view`, cleared to transparent `(0,0,0,0)` first.
    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fh6 quad pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..4, 0..self.count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_layout_is_48_bytes() {
        assert_eq!(size_of::<QuadInstance>(), 48);
        assert_eq!(size_of::<Globals>(), 16);
    }

    #[test]
    fn nan_inputs_are_sanitised() {
        let q = QuadInstance::new(
            [f32::NAN, 5.0, f32::INFINITY, -3.0],
            [f32::NAN, 2.0, 0.5, 1.0],
            QuadKind::RoundedRect {
                radius: f32::NAN,
                rim: -1.0,
            },
        );
        assert!(q.rect.iter().chain(&q.color).chain(&q.params).all(|v| v.is_finite()));
        assert_eq!(q.rect[2], 1.0, "width at least 1 px");
        assert_eq!(q.rect[3], 1.0, "height at least 1 px");
        assert_eq!(q.color[1], 1.0, "colour clamped");
    }

    #[test]
    fn batch_never_grows_past_capacity() {
        let mut batch = QuadBatch::new();
        let cap = batch.items.capacity();
        let q = QuadInstance::new([0.0; 4], [1.0; 4], QuadKind::RoundedRect { radius: 0.0, rim: 0.0 });
        for _ in 0..MAX_QUADS + 10 {
            batch.push(q);
        }
        assert_eq!(batch.len(), MAX_QUADS);
        assert_eq!(batch.dropped(), 10);
        assert_eq!(batch.items.capacity(), cap, "no reallocation");
    }
}
