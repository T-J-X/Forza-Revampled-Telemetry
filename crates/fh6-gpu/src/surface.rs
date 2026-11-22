//! The overlay window's swapchain.
use crate::context::{new_instance, GpuContext};
use crate::quad::{Globals, QuadBatch, QuadRenderer};
use crate::GpuError;
use std::time::{Duration, Instant};

/// Where the time of one presented frame went.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameTimings {
    /// Blocked in `get_current_texture` (the frame-latency waitable object). Not CPU work.
    pub acquire_wait: Duration,
    /// Uploading buffers, recording and submitting.
    pub encode_submit: Duration,
    /// The `present` call itself.
    pub present: Duration,
}

impl FrameTimings {
    /// CPU time spent on the frame, excluding the latency wait.
    pub fn cpu(&self) -> Duration {
        self.encode_submit + self.present
    }
}

/// What happened to a frame request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameOutcome {
    /// Presented; `at` is when `present` returned.
    Presented {
        /// Time breakdown.
        timings: FrameTimings,
        /// When `present` returned.
        at: Instant,
    },
    /// Nothing presented this time (timeout, occluded, or the surface was reconfigured).
    Skipped,
}

/// A premultiplied-alpha swapchain on the overlay window.
#[derive(Debug)]
pub struct OverlaySurface {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    renderer: QuadRenderer,
}

impl OverlaySurface {
    /// Creates the device and the window's swapchain together, so the adapter is one that can
    /// present to this window. `target` keeps the window handle alive for the surface's life.
    ///
    /// # Errors
    /// [`GpuError`] if the surface, adapter or device cannot be created, or if the surface
    /// cannot present with premultiplied alpha.
    pub fn create(
        target: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Result<(GpuContext, Self), GpuError> {
        let instance = new_instance();
        let surface = instance.create_surface(target)?;
        let ctx = GpuContext::from_instance(instance, Some(&surface))?;
        let caps = surface.get_capabilities(&ctx.adapter);
        if !caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
            return Err(GpuError::NoPremultipliedAlpha(caps.alpha_modes));
        }
        // Prefer an 8-bit UNORM format: the shader writes encoded premultiplied values.
        let format = [wgpu::TextureFormat::Bgra8Unorm, wgpu::TextureFormat::Rgba8Unorm]
            .into_iter()
            .find(|f| caps.formats.contains(f))
            .or_else(|| caps.formats.iter().copied().find(|f| !f.is_srgb()))
            .or_else(|| caps.formats.first().copied())
            .ok_or(GpuError::NoSurfaceFormat)?;
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: width.max(1),
            height: height.max(1),
            present_mode,
            desired_maximum_frame_latency: 1,
            alpha_mode: wgpu::CompositeAlphaMode::PreMultiplied,
            view_formats: Vec::new(),
        };
        surface.configure(&ctx.device, &config);
        let renderer = QuadRenderer::new(&ctx.device, format);
        tracing::info!(?format, ?present_mode, width, height, "overlay surface configured");
        Ok((
            ctx,
            Self {
                surface,
                config,
                renderer,
            },
        ))
    }

    /// Current swapchain size.
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// The present mode in use (Mailbox or Fifo).
    pub fn present_mode(&self) -> wgpu::PresentMode {
        self.config.present_mode
    }

    /// The swapchain format.
    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Resizes the swapchain (zero sizes are clamped to 1). A no-op if the size is unchanged.
    pub fn resize(&mut self, ctx: &GpuContext, width: u32, height: u32) {
        let (width, height) = (width.max(1), height.max(1));
        if (width, height) == (self.config.width, self.config.height) {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&ctx.device, &self.config);
        tracing::info!(width, height, "overlay surface resized");
    }

    /// Acquires the next swapchain image, lets `record` encode the frame into it (the render
    /// kit's path), submits and presents. `record` receives the context, an encoder, the image's
    /// view, the swapchain format and size.
    ///
    /// # Errors
    /// [`GpuError::SurfaceLost`] when the surface must be recreated, and
    /// [`GpuError::SurfaceValidation`] on a validation error.
    pub fn render_with(
        &mut self,
        ctx: &GpuContext,
        record: impl FnOnce(&GpuContext, &mut wgpu::CommandEncoder, &wgpu::TextureView, wgpu::TextureFormat, (u32, u32)),
    ) -> Result<FrameOutcome, GpuError> {
        let t0 = Instant::now();
        let acquired = self.surface.get_current_texture();
        let t1 = Instant::now();
        let (frame, suboptimal) = match acquired {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(FrameOutcome::Skipped)
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&ctx.device, &self.config);
                return Ok(FrameOutcome::Skipped);
            }
            wgpu::CurrentSurfaceTexture::Lost => return Err(GpuError::SurfaceLost),
            wgpu::CurrentSurfaceTexture::Validation => return Err(GpuError::SurfaceValidation),
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fh6 overlay frame"),
        });
        record(
            ctx,
            &mut encoder,
            &view,
            self.config.format,
            (self.config.width, self.config.height),
        );
        ctx.queue.submit(Some(encoder.finish()));
        let t2 = Instant::now();
        ctx.queue.present(frame);
        let t3 = Instant::now();
        if suboptimal {
            self.surface.configure(&ctx.device, &self.config);
        }
        Ok(FrameOutcome::Presented {
            timings: FrameTimings {
                acquire_wait: t1 - t0,
                encode_submit: t2 - t1,
                present: t3 - t2,
            },
            at: t3,
        })
    }

    /// Draws `batch` at `opacity` and presents it.
    ///
    /// # Errors
    /// [`GpuError::SurfaceLost`] when the surface must be recreated, and
    /// [`GpuError::SurfaceValidation`] on a validation error.
    pub fn render(&mut self, ctx: &GpuContext, opacity: f32, batch: &QuadBatch) -> Result<FrameOutcome, GpuError> {
        let t0 = Instant::now();
        let acquired = self.surface.get_current_texture();
        let t1 = Instant::now();
        let (frame, suboptimal) = match acquired {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(FrameOutcome::Skipped)
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&ctx.device, &self.config);
                return Ok(FrameOutcome::Skipped);
            }
            wgpu::CurrentSurfaceTexture::Lost => return Err(GpuError::SurfaceLost),
            wgpu::CurrentSurfaceTexture::Validation => return Err(GpuError::SurfaceValidation),
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let globals = Globals::new(self.config.width, self.config.height, opacity);
        self.renderer.prepare(&ctx.queue, &globals, batch);
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fh6 overlay frame"),
        });
        self.renderer.record(&mut encoder, &view);
        ctx.queue.submit(Some(encoder.finish()));
        let t2 = Instant::now();
        ctx.queue.present(frame);
        let t3 = Instant::now();
        if suboptimal {
            self.surface.configure(&ctx.device, &self.config);
        }
        Ok(FrameOutcome::Presented {
            timings: FrameTimings {
                acquire_wait: t1 - t0,
                encode_submit: t2 - t1,
                present: t3 - t2,
            },
            at: t3,
        })
    }
}
