//! The optional glass refraction source: a Windows.Graphics.Capture of the game window around
//! the cluster, uploaded as a texture that `Refraction::World` glass samples.
//!
//! Off by default (`"GlassRefraction": true` in `config.json` turns it on). Capture is
//! pipelined (the image is one frame behind) and downscaled 2x; its cost is reported in
//! `docs/perf/render-kit.md`.
use fh6_kit::wgpu;
use fh6_kit::Rect;
use fh6_win::wgc::{self, CaptureFrame, CaptureRegion, WindowCapture};
use std::time::{Duration, Instant};

/// Wait between attempts to start the capture when it is unavailable (game not running).
pub const RETRY: Duration = Duration::from_secs(5);

/// A running capture and its GPU texture.
pub struct RefractionSource {
    capture: WindowCapture,
    frame: CaptureFrame,
    texture: Option<(wgpu::Texture, wgpu::TextureView, (u32, u32))>,
    rect: Rect,
    /// CPU time of the last delivered frame (copy, map, convert) plus its upload.
    pub last_cost: Duration,
}

impl RefractionSource {
    /// Starts capturing `rect` (target pixels, which are the game's client pixels for a
    /// borderless game on the overlay's monitor) of the window of `game_process`.
    pub fn start(game_process: &str, rect: Rect) -> Option<Self> {
        if !wgc::is_supported() {
            tracing::warn!("Windows.Graphics.Capture is not supported; glass refraction stays off");
            return None;
        }
        let hwnd = wgc::find_process_window(game_process)?;
        let region = CaptureRegion {
            x: rect.min.x.max(0.0) as i32,
            y: rect.min.y.max(0.0) as i32,
            width: rect.width().max(1.0) as u32,
            height: rect.height().max(1.0) as u32,
        };
        let max = (region.width.div_ceil(2), region.height.div_ceil(2));
        match WindowCapture::start(hwnd, region, max) {
            Ok(capture) => {
                tracing::info!(?region, "glass refraction capture started");
                Some(Self {
                    capture,
                    frame: CaptureFrame::with_capacity(region.width, region.height),
                    texture: None,
                    rect,
                    last_cost: Duration::ZERO,
                })
            }
            Err(err) => {
                tracing::warn!(error = %err, "glass refraction capture failed to start");
                None
            }
        }
    }

    /// The target rectangle captured.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Polls for a new frame and uploads it. Returns the view to bind when the texture was
    /// (re)created, so the caller attaches it to the kit.
    ///
    /// # Errors
    /// Returns the capture error; the caller drops the source and retries later.
    pub fn poll(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Option<wgpu::TextureView>, wgc::WgcError> {
        let t0 = Instant::now();
        if !self.capture.latest(&mut self.frame)? {
            return Ok(None);
        }
        let size = (self.frame.width.max(1), self.frame.height.max(1));
        let mut new_view = None;
        if self.texture.as_ref().is_none_or(|(_, _, s)| *s != size) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fh6 refraction source"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            new_view = Some(view.clone());
            self.texture = Some((texture, view, size));
        }
        if let Some((texture, _, _)) = self.texture.as_ref() {
            let bytes = (size.0 * size.1 * 4) as usize;
            if let Some(pixels) = self.frame.pixels.get(..bytes) {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    pixels,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(size.0 * 4),
                        rows_per_image: Some(size.1),
                    },
                    wgpu::Extent3d {
                        width: size.0,
                        height: size.1,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
        self.last_cost = t0.elapsed();
        Ok(new_view)
    }
}
