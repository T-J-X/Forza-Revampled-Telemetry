//! The mesh pass: glTF (`.glb`) models for ADAS cars, drawn depth-tested inside a 2D viewport
//! rectangle with unlit or matcap-style shading and a rim light. Until Phase 6 ships the glTF
//! cars, [`MeshData::car_proxy`] builds a procedural low-poly car.
use crate::color::Color;
use crate::math::{vec3, Mat4, Rect, Vec3};
use crate::scene::MAX_MESHES;
use bytemuck::{Pod, Zeroable};
use std::mem::size_of;

/// A handle to a mesh uploaded with [`Meshes`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshId(pub(crate) u32);

/// How a mesh is shaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeshShading {
    /// Flat colour plus rim light.
    Unlit,
    /// Diffuse + specular from one light plus rim light (matcap look).
    #[default]
    Matcap,
}

/// One mesh draw, recorded with [`crate::Painter::mesh`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshDraw {
    /// The mesh.
    pub mesh: MeshId,
    /// Model transform (world space, metres; +Y up, +Z forward).
    pub model: Mat4,
    /// Camera view-projection (see [`crate::perspective::Camera3`]).
    pub view_proj: Mat4,
    /// Camera position, for rim and specular.
    pub eye: Vec3,
    /// The local rectangle the camera's image maps to.
    pub viewport: Rect,
    /// Base colour (premultiplied, e.g. the palette primary for the ego car).
    pub color: Color,
    /// Rim light colour (additive).
    pub rim: Color,
    /// Direction towards the light (world).
    pub light: Vec3,
    /// Shading model.
    pub shading: MeshShading,
}

/// A recorded mesh draw with its target-space viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MeshCmd {
    pub draw: MeshDraw,
    pub viewport: Rect,
    pub opacity: f32,
    pub bloom: u32,
}

/// CPU-side triangle mesh.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MeshData {
    /// Positions.
    pub positions: Vec<[f32; 3]>,
    /// Normals (same length as positions).
    pub normals: Vec<[f32; 3]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

/// Errors loading meshes.
#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    /// The glTF could not be parsed.
    #[error("glTF: {0}")]
    Gltf(String),
    /// The model has no triangles.
    #[error("the model has no triangle geometry")]
    Empty,
}

impl MeshData {
    /// Parses a binary glTF (`.glb`), flattening every node of the default scene (with its
    /// transform) into one mesh. Missing normals are computed per face.
    ///
    /// # Errors
    /// [`MeshError`] when the file is not a usable glTF.
    pub fn from_glb(bytes: &[u8]) -> Result<Self, MeshError> {
        let gltf = gltf::Gltf::from_slice(bytes).map_err(|e| MeshError::Gltf(e.to_string()))?;
        let blob = gltf.blob.as_deref();
        let mut out = MeshData::default();
        let scene = gltf
            .default_scene()
            .or_else(|| gltf.scenes().next())
            .ok_or(MeshError::Empty)?;
        let mut stack: Vec<(gltf::Node<'_>, Mat4)> = scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
        while let Some((node, parent)) = stack.pop() {
            let local = Mat4 {
                cols: node.transform().matrix(),
            };
            let world = parent.mul(&local);
            if let Some(mesh) = node.mesh() {
                for prim in mesh.primitives() {
                    if prim.mode() != gltf::mesh::Mode::Triangles {
                        continue;
                    }
                    let reader = prim.reader(|buf| match buf.source() {
                        gltf::buffer::Source::Bin => blob,
                        gltf::buffer::Source::Uri(_) => None,
                    });
                    let Some(pos) = reader.read_positions() else { continue };
                    let base = out.positions.len() as u32;
                    let positions: Vec<[f32; 3]> = pos.collect();
                    let normals: Option<Vec<[f32; 3]>> = reader.read_normals().map(Iterator::collect);
                    let indices: Vec<u32> = match reader.read_indices() {
                        Some(i) => i.into_u32().collect(),
                        None => (0..positions.len() as u32).collect(),
                    };
                    for (i, p) in positions.iter().enumerate() {
                        let w = world.project(vec3(p[0], p[1], p[2]));
                        out.positions.push([w[0], w[1], w[2]]);
                        let n = normals
                            .as_ref()
                            .and_then(|ns| ns.get(i))
                            .copied()
                            .unwrap_or([0.0, 1.0, 0.0]);
                        let wn = transform_normal(&world, n);
                        out.normals.push(wn);
                    }
                    out.indices.extend(indices.iter().map(|i| base + i));
                    if normals.is_none() {
                        out.recompute_normals_from(base as usize);
                    }
                }
            }
            for child in node.children() {
                stack.push((child, world));
            }
        }
        if out.indices.is_empty() {
            return Err(MeshError::Empty);
        }
        Ok(out)
    }

    /// A procedural low-poly car: a bevelled body box with a tapered cabin, `length` x `width`
    /// x `height` metres, centred on the origin at ground level, nose towards +Z.
    #[must_use]
    pub fn car_proxy(length: f32, width: f32, height: f32) -> Self {
        let mut m = MeshData::default();
        let hl = length * 0.5;
        let hw = width * 0.5;
        let body_h = height * 0.55;
        // Body: a box with chamfered nose and tail.
        m.add_prism(
            &[
                (-hl, 0.12),
                (-hl, body_h * 0.85),
                (-hl * 0.85, body_h),
                (hl * 0.80, body_h * 0.92),
                (hl, body_h * 0.55),
                (hl, 0.12),
            ],
            hw,
        );
        // Cabin: tapered greenhouse.
        m.add_prism(
            &[
                (-hl * 0.55, body_h),
                (-hl * 0.35, height),
                (hl * 0.15, height),
                (hl * 0.45, body_h),
            ],
            hw * 0.82,
        );
        m
    }

    /// Adds a prism extruded along X from a side profile (z, y) polygon (convex, CCW).
    fn add_prism(&mut self, profile: &[(f32, f32)], half_width: f32) {
        let n = profile.len();
        for side in [-1.0f32, 1.0] {
            let x = side * half_width;
            let base = self.positions.len() as u32;
            for &(z, y) in profile {
                self.positions.push([x, y, z]);
                self.normals.push([side, 0.0, 0.0]);
            }
            for i in 1..n as u32 - 1 {
                if side > 0.0 {
                    self.indices.extend([base, base + i, base + i + 1]);
                } else {
                    self.indices.extend([base, base + i + 1, base + i]);
                }
            }
        }
        for i in 0..n {
            let (z0, y0) = profile[i];
            let (z1, y1) = profile[(i + 1) % n];
            let edge = vec3(0.0, y1 - y0, z1 - z0);
            let normal = vec3(1.0, 0.0, 0.0).cross(edge).normalize_or_zero();
            let base = self.positions.len() as u32;
            for (x, z, y) in [
                (-half_width, z0, y0),
                (half_width, z0, y0),
                (half_width, z1, y1),
                (-half_width, z1, y1),
            ] {
                self.positions.push([x, y, z]);
                self.normals.push([normal.x, normal.y, normal.z]);
            }
            self.indices
                .extend([base, base + 2, base + 1, base, base + 3, base + 2]);
        }
    }

    fn recompute_normals_from(&mut self, first_vertex: usize) {
        for n in self.normals.iter_mut().skip(first_vertex) {
            *n = [0.0; 3];
        }
        for tri in self.indices.chunks_exact(3) {
            let [a, b, c] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
            if a < first_vertex || b < first_vertex || c < first_vertex {
                continue;
            }
            let p = |i: usize| vec3(self.positions[i][0], self.positions[i][1], self.positions[i][2]);
            let fnrm = (p(b) - p(a)).cross(p(c) - p(a));
            for i in [a, b, c] {
                self.normals[i][0] += fnrm.x;
                self.normals[i][1] += fnrm.y;
                self.normals[i][2] += fnrm.z;
            }
        }
        for n in self.normals.iter_mut().skip(first_vertex) {
            let v = vec3(n[0], n[1], n[2]).normalize_or_zero();
            *n = [v.x, v.y, v.z];
        }
    }
}

fn transform_normal(m: &Mat4, n: [f32; 3]) -> [f32; 3] {
    let c = &m.cols;
    let v = vec3(
        c[0][0] * n[0] + c[1][0] * n[1] + c[2][0] * n[2],
        c[0][1] * n[0] + c[1][1] * n[1] + c[2][1] * n[2],
        c[0][2] * n[0] + c[1][2] * n[1] + c[2][2] * n[2],
    )
    .normalize_or_zero();
    [v.x, v.y, v.z]
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

#[derive(Debug)]
struct GpuMesh {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    count: u32,
}

/// The per-draw uniform (`mesh.wgsl` `Draw`), padded to 256 bytes for dynamic offsets.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(crate) struct DrawUniform {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    color: [f32; 4],
    rim: [f32; 4],
    light: [f32; 4],
    ndc_map: [f32; 4],
    params: [f32; 4],
    eye: [f32; 4],
    _pad: [f32; 8],
}

pub(crate) const DRAW_STRIDE: u64 = 256;

/// Uploaded meshes and the mesh pipeline.
#[derive(Debug)]
pub struct Meshes {
    meshes: Vec<GpuMesh>,
    pub(crate) pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
    staged: Vec<DrawUniform>,
}

impl Meshes {
    pub(crate) fn new(device: &wgpu::Device, depth_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fh6-kit mesh"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mesh.wgsl").into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fh6-kit mesh layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(size_of::<DrawUniform>() as u64),
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fh6-kit mesh pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        const ATTRS: [wgpu::VertexAttribute; 2] = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 12,
                shader_location: 1,
            },
        ];
        let blend = Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING);
        let target = |b| {
            Some(wgpu::ColorTargetState {
                format: crate::renderer::SCENE_FORMAT,
                blend: b,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fh6-kit mesh pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_mesh"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &ATTRS,
                })],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_mesh"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[target(blend), target(Some(crate::renderer::ADD_BLEND))],
            }),
            multiview_mask: None,
            cache: None,
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fh6-kit mesh uniforms"),
            size: DRAW_STRIDE * MAX_MESHES as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fh6-kit mesh bind group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniforms,
                    offset: 0,
                    size: wgpu::BufferSize::new(size_of::<DrawUniform>() as u64),
                }),
            }],
        });
        Self {
            meshes: Vec::new(),
            pipeline,
            uniforms,
            bind_group,
            staged: Vec::with_capacity(MAX_MESHES),
        }
    }

    /// Uploads a mesh.
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &MeshData) -> MeshId {
        let vertices: Vec<Vertex> = data
            .positions
            .iter()
            .zip(&data.normals)
            .map(|(p, n)| Vertex {
                position: *p,
                normal: *n,
            })
            .collect();
        let vb = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fh6-kit mesh vertices"),
            size: (vertices.len().max(1) * size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vb, 0, bytemuck::cast_slice(&vertices));
        let ib = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fh6-kit mesh indices"),
            size: (data.indices.len().max(1) * 4) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&ib, 0, bytemuck::cast_slice(&data.indices));
        self.meshes.push(GpuMesh {
            vertices: vb,
            indices: ib,
            count: data.indices.len() as u32,
        });
        MeshId((self.meshes.len() - 1) as u32)
    }

    /// Builds this frame's draw uniforms (one write) for `cmds`, mapping each viewport into the
    /// work rectangle.
    pub(crate) fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        cmds: &[MeshCmd],
        work_origin: [f32; 2],
        work_size: [f32; 2],
    ) {
        self.staged.clear();
        for c in cmds.iter().take(MAX_MESHES) {
            let v = c.viewport;
            let (ws_x, ws_y) = (work_size[0].max(1.0), work_size[1].max(1.0));
            let sx = v.width() / ws_x;
            let sy = v.height() / ws_y;
            let ox = (v.min.x + v.width() * 0.5 - work_origin[0]) / ws_x * 2.0 - 1.0;
            let oy = 1.0 - (v.min.y + v.height() * 0.5 - work_origin[1]) / ws_y * 2.0;
            let bloom = if c.bloom == 0 {
                [0.0, 0.0]
            } else {
                let lo = half_to_f32((c.bloom & 0xFFFF) as u16);
                let hi = half_to_f32((c.bloom >> 16) as u16);
                [lo, hi]
            };
            let (r, g, b) = c.draw.color.straight_rgb();
            let (rr, rg, rb) = c.draw.rim.straight_rgb();
            self.staged.push(DrawUniform {
                view_proj: c.draw.view_proj.cols,
                model: c.draw.model.cols,
                color: [r, g, b, c.draw.color.a],
                rim: [rr * c.draw.rim.a, rg * c.draw.rim.a, rb * c.draw.rim.a, 0.0],
                light: [
                    c.draw.light.x,
                    c.draw.light.y,
                    c.draw.light.z,
                    match c.draw.shading {
                        MeshShading::Unlit => 0.0,
                        MeshShading::Matcap => 1.0,
                    },
                ],
                ndc_map: [sx, sy, ox, oy],
                params: [c.opacity, bloom[0], bloom[1], 0.0],
                eye: [c.draw.eye.x, c.draw.eye.y, c.draw.eye.z, 1.0],
                _pad: [0.0; 8],
            });
        }
        if !self.staged.is_empty() {
            queue.write_buffer(&self.uniforms, 0, bytemuck::cast_slice(&self.staged));
        }
    }

    /// Records draw `index` (already prepared) into `pass`.
    pub(crate) fn record(&self, pass: &mut wgpu::RenderPass<'_>, index: u32, mesh: MeshId) {
        let Some(m) = self.meshes.get(mesh.0 as usize) else {
            return;
        };
        if m.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[(u64::from(index) * DRAW_STRIDE) as u32]);
        pass.set_vertex_buffer(0, m.vertices.slice(..));
        pass.set_index_buffer(m.indices.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..m.count, 0, 0..1);
    }
}

/// Half-float bits to `f32` (normal and subnormal values; inf/NaN become 0).
pub(crate) fn half_to_f32(h: u16) -> f32 {
    let exp = (h >> 10) & 0x1F;
    let mant = f32::from(h & 0x3FF);
    let v = match exp {
        0 => mant * 2f32.powi(-24),
        31 => 0.0,
        e => (1.0 + mant / 1024.0) * 2f32.powi(i32::from(e) - 15),
    };
    if h & 0x8000 != 0 {
        -v
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_is_256_bytes() {
        assert_eq!(size_of::<DrawUniform>() as u64, DRAW_STRIDE);
    }

    #[test]
    fn car_proxy_is_closed_geometry() {
        let m = MeshData::car_proxy(4.6, 1.9, 1.3);
        assert_eq!(m.positions.len(), m.normals.len());
        assert!(m.indices.len() % 3 == 0 && !m.indices.is_empty());
        assert!(m.indices.iter().all(|i| (*i as usize) < m.positions.len()));
        let max_y = m.positions.iter().map(|p| p[1]).fold(0.0f32, f32::max);
        assert!((max_y - 1.3).abs() < 1e-4);
    }

    #[test]
    fn half_round_trip() {
        for v in [0.0f32, 0.5, 1.0, 0.7, 2.0] {
            let h = crate::scene::f16_bits(v);
            assert!((half_to_f32(h) - v).abs() < 1e-3, "{v}");
        }
    }

    #[test]
    fn glb_errors_are_reported() {
        assert!(MeshData::from_glb(b"not a glb").is_err());
    }
}
