//! Offscreen rendering, PNG I/O and perceptual image comparison: golden-image tests,
//! `preview-export` and benches. Native only (readback blocks on the GPU).
#![cfg(not(target_arch = "wasm32"))]

use crate::color::{srgb_to_linear, Oklab};
use std::path::Path;
use std::time::Duration;

/// The format offscreen frames render to (same encoding as the overlay swapchain, RGBA order).
pub const OFFSCREEN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// An RGBA8 image, premultiplied, sRGB-encoded (what the overlay presents).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaImage {
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
    /// `width * height * 4` bytes.
    pub pixels: Vec<u8>,
}

/// Errors in offscreen readback and image I/O.
#[derive(Debug, thiserror::Error)]
pub enum TestingError {
    /// GPU readback failed.
    #[error("readback: {0}")]
    Readback(String),
    /// PNG I/O failed.
    #[error("png: {0}")]
    Png(String),
    /// File I/O failed.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl RgbaImage {
    /// The pixel at (x, y).
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = ((y * self.width + x) * 4) as usize;
        self.pixels.get(i..i + 4).and_then(|p| p.try_into().ok())
    }

    /// Pixels with non-zero alpha.
    #[must_use]
    pub fn covered(&self) -> usize {
        self.pixels.chunks_exact(4).filter(|p| p[3] != 0).count()
    }

    /// Box-downsampled by an integer factor (real thumbnails from a 4K render).
    #[must_use]
    pub fn downsample(&self, factor: u32) -> RgbaImage {
        let f = factor.max(1);
        let (w, h) = ((self.width / f).max(1), (self.height / f).max(1));
        let mut out = vec![0u8; (w * h * 4) as usize];
        let n = f * f;
        for y in 0..h {
            for x in 0..w {
                let mut acc = [0u32; 4];
                for dy in 0..f {
                    for dx in 0..f {
                        if let Some(p) = self.pixel(x * f + dx, y * f + dy) {
                            for c in 0..4 {
                                acc[c] += u32::from(p[c]);
                            }
                        }
                    }
                }
                let i = ((y * w + x) * 4) as usize;
                for c in 0..4 {
                    out[i + c] = (acc[c] / n) as u8;
                }
            }
        }
        RgbaImage {
            width: w,
            height: h,
            pixels: out,
        }
    }

    /// A copy of the rectangle (clamped to the image).
    #[must_use]
    pub fn crop(&self, x: u32, y: u32, w: u32, h: u32) -> RgbaImage {
        let x = x.min(self.width);
        let y = y.min(self.height);
        let w = w.min(self.width - x);
        let h = h.min(self.height - y);
        let mut pixels = Vec::with_capacity((w * h * 4) as usize);
        for row in y..y + h {
            let s = ((row * self.width + x) * 4) as usize;
            pixels.extend_from_slice(&self.pixels[s..s + (w * 4) as usize]);
        }
        RgbaImage {
            width: w,
            height: h,
            pixels,
        }
    }

    /// Composited over opaque `bg` (sRGB bytes) for viewing: premultiplied over in encoded
    /// space, exactly what DWM does with the overlay.
    #[must_use]
    pub fn over(&self, bg: [u8; 3]) -> RgbaImage {
        let mut pixels = self.pixels.clone();
        for p in pixels.chunks_exact_mut(4) {
            let a = f32::from(p[3]) / 255.0;
            for c in 0..3 {
                p[c] = (f32::from(p[c]) + f32::from(bg[c]) * (1.0 - a))
                    .round()
                    .clamp(0.0, 255.0) as u8;
            }
            p[3] = 255;
        }
        RgbaImage {
            width: self.width,
            height: self.height,
            pixels,
        }
    }

    /// Writes a PNG (RGBA8).
    ///
    /// # Errors
    /// [`TestingError`] on I/O or encoding failure.
    pub fn save_png(&self, path: &Path) -> Result<(), TestingError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let file = std::fs::File::create(path)?;
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), self.width, self.height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().map_err(|e| TestingError::Png(e.to_string()))?;
        w.write_image_data(&self.pixels)
            .map_err(|e| TestingError::Png(e.to_string()))?;
        Ok(())
    }

    /// Reads an RGBA8 PNG written by [`RgbaImage::save_png`].
    ///
    /// # Errors
    /// [`TestingError`] on I/O or decoding failure.
    pub fn load_png(path: &Path) -> Result<RgbaImage, TestingError> {
        let bytes = std::fs::read(path)?;
        let (width, height, pixels) =
            crate::images::decode_png(&bytes).map_err(|e| TestingError::Png(e.to_string()))?;
        Ok(RgbaImage { width, height, pixels })
    }
}

/// An offscreen colour target with a readback buffer.
#[derive(Debug)]
pub struct Offscreen {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    readback: wgpu::Buffer,
    width: u32,
    height: u32,
    padded_row: u32,
}

impl Offscreen {
    /// A `width` x `height` target in [`OFFSCREEN_FORMAT`].
    #[must_use]
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let (width, height) = (width.max(1), height.max(1));
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fh6-kit offscreen"),
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
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fh6-kit offscreen readback"),
            size: u64::from(padded_row) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Self {
            texture,
            view,
            readback,
            width,
            height,
            padded_row,
        }
    }

    /// The view to render into.
    #[must_use]
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Size in pixels.
    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Appends the copy to the readback buffer to `encoder`; call [`Offscreen::read`] after
    /// submitting it.
    pub fn copy_out(&self, encoder: &mut wgpu::CommandEncoder) {
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
    }

    /// Maps the readback buffer (blocks until the GPU is done) and returns the image.
    ///
    /// # Errors
    /// [`TestingError::Readback`] when mapping fails or times out.
    pub fn read(&self, device: &wgpu::Device) -> Result<RgbaImage, TestingError> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.readback.map_async(wgpu::MapMode::Read, .., move |r| {
            let _ = tx.send(r);
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(20)),
            })
            .map_err(|e| TestingError::Readback(e.to_string()))?;
        rx.recv_timeout(Duration::from_secs(20))
            .map_err(|e| TestingError::Readback(e.to_string()))?
            .map_err(|e| TestingError::Readback(e.to_string()))?;
        let row = (self.width * 4) as usize;
        let mut pixels = Vec::with_capacity(row * self.height as usize);
        {
            let mapped = self
                .readback
                .get_mapped_range(..)
                .map_err(|e| TestingError::Readback(e.to_string()))?;
            for chunk in mapped.chunks(self.padded_row as usize).take(self.height as usize) {
                pixels.extend_from_slice(&chunk[..row]);
            }
        }
        self.readback.unmap();
        Ok(RgbaImage {
            width: self.width,
            height: self.height,
            pixels,
        })
    }
}

/// Tolerance for [`compare`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tolerance {
    /// Per-pixel OKLab distance (x100, roughly a JND of 1–2) above which a pixel counts as
    /// different.
    pub delta_e: f32,
    /// Largest fraction of pixels allowed to differ.
    pub max_fraction: f32,
}

impl Default for Tolerance {
    /// 3.0 ΔE (OKLab x 100) on at most 0.5% of pixels: stable across GPUs and drivers, strict
    /// enough to catch a missing element.
    fn default() -> Self {
        Self {
            delta_e: 3.0,
            max_fraction: 0.005,
        }
    }
}

/// The result of comparing two images.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Diff {
    /// Mean ΔE over all pixels.
    pub mean: f32,
    /// Largest ΔE.
    pub max: f32,
    /// Fraction of pixels above the tolerance.
    pub fraction_over: f32,
    /// Whether the images match within the tolerance.
    pub pass: bool,
}

/// Perceptual comparison: both images are composited over black (as on the OLED), lightly
/// blurred (2x2 box, ignoring sub-pixel AA differences), converted to OKLab, and compared.
#[must_use]
pub fn compare(a: &RgbaImage, b: &RgbaImage, tol: Tolerance) -> Diff {
    if a.width != b.width || a.height != b.height {
        return Diff {
            mean: f32::INFINITY,
            max: f32::INFINITY,
            fraction_over: 1.0,
            pass: false,
        };
    }
    let lab = |img: &RgbaImage, x: u32, y: u32| -> Oklab {
        let mut acc = [0.0f32; 3];
        let mut n = 0.0;
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            if let Some(p) = img.pixel((x + dx).min(img.width - 1), (y + dy).min(img.height - 1)) {
                for c in 0..3 {
                    acc[c] += srgb_to_linear(f32::from(p[c]) / 255.0);
                }
                n += 1.0;
            }
        }
        Oklab::from_linear_rgb(acc[0] / n, acc[1] / n, acc[2] / n)
    };
    let (mut sum, mut max, mut over) = (0.0f64, 0.0f32, 0u64);
    for y in 0..a.height {
        for x in 0..a.width {
            let (p, q) = (lab(a, x, y), lab(b, x, y));
            let d = ((p.l - q.l).powi(2) + (p.a - q.a).powi(2) + (p.b - q.b).powi(2)).sqrt() * 100.0;
            sum += f64::from(d);
            max = max.max(d);
            if d > tol.delta_e {
                over += 1;
            }
        }
    }
    let n = f64::from(a.width) * f64::from(a.height);
    let fraction_over = (over as f64 / n.max(1.0)) as f32;
    Diff {
        mean: (sum / n.max(1.0)) as f32,
        max,
        fraction_over,
        pass: fraction_over <= tol.max_fraction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(w: u32, h: u32, px: [u8; 4]) -> RgbaImage {
        RgbaImage {
            width: w,
            height: h,
            pixels: px.repeat((w * h) as usize),
        }
    }

    #[test]
    fn identical_images_pass_and_different_fail() {
        let a = img(8, 8, [200, 30, 30, 255]);
        assert!(compare(&a, &a, Tolerance::default()).pass);
        let b = img(8, 8, [30, 200, 30, 255]);
        assert!(!compare(&a, &b, Tolerance::default()).pass);
        let mut c = a.clone();
        c.pixels[0] = 201;
        assert!(compare(&a, &c, Tolerance::default()).pass, "one level of noise passes");
    }

    #[test]
    fn downsample_and_over() {
        let a = img(4, 4, [255, 255, 255, 255]);
        let d = a.downsample(2);
        assert_eq!((d.width, d.height), (2, 2));
        let half = img(1, 1, [64, 64, 64, 128]).over([0, 0, 0]);
        assert_eq!(half.pixels, vec![64, 64, 64, 255]);
    }
}
