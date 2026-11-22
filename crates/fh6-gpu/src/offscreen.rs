//! Rendering into a texture and reading it back, for tests, benches and `preview-export`.
//!
//! The readback is a deliberate CPU round-trip and is never used by the overlay's frame loop.
use crate::context::GpuContext;
use crate::quad::{Globals, QuadBatch, QuadRenderer};
use crate::GpuError;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// An RGBA8 image with premultiplied alpha, rows tightly packed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height * 4` bytes, R G B A per pixel, premultiplied.
    pub pixels: Vec<u8>,
}

impl RgbaImage {
    /// The RGBA bytes of pixel (x, y), or `None` outside the image.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = ((y as usize) * (self.width as usize) + x as usize) * 4;
        self.pixels.get(i..i + 4).and_then(|p| p.try_into().ok())
    }

    /// Number of pixels with non-zero alpha.
    pub fn covered_pixels(&self) -> usize {
        self.pixels.chunks_exact(4).filter(|p| p[3] != 0).count()
    }
}

/// An offscreen colour target with a matching readback buffer.
#[derive(Debug)]
pub struct OffscreenTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    readback: wgpu::Buffer,
    renderer: QuadRenderer,
    width: u32,
    height: u32,
    padded_row: u32,
}

/// Format of offscreen targets: same encoding as the overlay swapchain, RGBA byte order.
pub const OFFSCREEN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

impl OffscreenTarget {
    /// A `width` x `height` target (each at least 1).
    pub fn new(ctx: &GpuContext, width: u32, height: u32) -> Self {
        let (width, height) = (width.max(1), height.max(1));
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fh6 offscreen target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_row = (width * 4).div_ceil(align) * align;
        let readback = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fh6 offscreen readback"),
            size: u64::from(padded_row) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let renderer = QuadRenderer::new(&ctx.device, OFFSCREEN_FORMAT);
        Self {
            texture,
            view,
            readback,
            renderer,
            width,
            height,
            padded_row,
        }
    }

    /// Target size.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Renders `batch` and waits for the GPU. Returns (CPU encode+submit time, submit-to-idle
    /// time), the latter an upper bound on GPU time for the frame. Check
    /// [`GpuContext::has_failed`] afterwards: a rejected submit returns almost at once.
    pub fn render(&mut self, ctx: &GpuContext, opacity: f32, batch: &QuadBatch) -> (Duration, Duration) {
        let t0 = Instant::now();
        self.renderer
            .prepare(&ctx.queue, &Globals::new(self.width, self.height, opacity), batch);
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fh6 offscreen frame"),
        });
        self.renderer.record(&mut encoder, &self.view);
        let index = ctx.queue.submit(Some(encoder.finish()));
        let t1 = Instant::now();
        let _ = ctx.device.poll(wgpu::PollType::Wait {
            submission_index: Some(index),
            timeout: Some(Duration::from_secs(5)),
        });
        (t1 - t0, t1.elapsed())
    }

    /// Renders `batch` and reads the result back.
    ///
    /// # Errors
    /// [`GpuError::Readback`] if mapping the readback buffer fails, and
    /// [`GpuError::DeviceFailed`] if wgpu reported an error (the image would be meaningless).
    pub fn render_to_image(
        &mut self,
        ctx: &GpuContext,
        opacity: f32,
        batch: &QuadBatch,
    ) -> Result<RgbaImage, GpuError> {
        self.renderer
            .prepare(&ctx.queue, &Globals::new(self.width, self.height, opacity), batch);
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fh6 offscreen readback frame"),
        });
        self.renderer.record(&mut encoder, &self.view);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        let index = ctx.queue.submit(Some(encoder.finish()));

        let (tx, rx) = mpsc::channel();
        self.readback.map_async(wgpu::MapMode::Read, .., move |result| {
            let _ = tx.send(result);
        });
        ctx.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(index),
                timeout: Some(Duration::from_secs(10)),
            })
            .map_err(|e| GpuError::Readback(e.to_string()))?;
        rx.recv_timeout(Duration::from_secs(10))
            .map_err(|e| GpuError::Readback(e.to_string()))?
            .map_err(|e| GpuError::Readback(e.to_string()))?;

        let row = (self.width * 4) as usize;
        let mut pixels = Vec::with_capacity(row * self.height as usize);
        {
            let mapped = self
                .readback
                .get_mapped_range(..)
                .map_err(|e| GpuError::Readback(e.to_string()))?;
            for chunk in mapped.chunks(self.padded_row as usize).take(self.height as usize) {
                pixels.extend_from_slice(&chunk[..row]);
            }
        }
        self.readback.unmap();
        if ctx.has_failed() {
            return Err(GpuError::DeviceFailed);
        }
        Ok(RgbaImage {
            width: self.width,
            height: self.height,
            pixels,
        })
    }
}
