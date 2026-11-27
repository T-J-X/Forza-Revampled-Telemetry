//! The image store: PNG, KTX2 and SVG (through resvg) textures, plus GPU views registered by
//! other passes (the fluid dye, the refraction capture). Images load once; every texture has a
//! mip chain so logos stay clean when drawn small.
use crate::color::{linear_to_srgb, srgb_to_linear};
use crate::paint::ImageId;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Errors loading images.
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    /// Reading the file failed.
    #[error("reading {path}: {source}")]
    Io {
        /// The file.
        path: String,
        /// The cause.
        source: std::io::Error,
    },
    /// The PNG could not be decoded.
    #[error("decoding PNG: {0}")]
    Png(String),
    /// The KTX2 container could not be read or uses an unsupported format.
    #[error("KTX2: {0}")]
    Ktx2(String),
    /// The SVG could not be parsed or rendered.
    #[error("SVG: {0}")]
    Svg(String),
    /// The file extension is not png, ktx2 or svg.
    #[error("unsupported image type {0:?}")]
    Unsupported(String),
    /// The image is empty or larger than the device allows.
    #[error("image size {width}x{height} is not supported (max {max})")]
    Size {
        /// Width.
        width: u32,
        /// Height.
        height: u32,
        /// Device limit.
        max: u32,
    },
}

#[derive(Debug)]
struct Entry {
    _texture: Option<wgpu::Texture>,
    bind_group: wgpu::BindGroup,
    size: (u32, u32),
    mips: u32,
    /// Texels are straight alpha (premultiply in the shader).
    straight: bool,
}

/// A file that [`Images::load_file`] uploaded: its path, the SVG size it was rasterised at, and
/// its length and modification time, so a changed file is loaded again.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FileKey {
    path: PathBuf,
    svg_size: Option<(u32, u32)>,
    len: u64,
    modified: Option<SystemTime>,
}

/// Every image the kit can draw. Slot 0 is a 1x1 white texel used by draws without an image.
#[derive(Debug)]
pub struct Images {
    entries: Vec<Entry>,
    /// The files already uploaded. The app builds a cluster again whenever the view or theme
    /// changes, and the cluster loads its files in `new()`: this hands it the textures it already
    /// has instead of decoding and uploading them again (and keeping both copies).
    files: Vec<(FileKey, ImageId)>,
    layout: wgpu::BindGroupLayout,
    max_dim: u32,
    bc: bool,
}

/// Most images that can be loaded.
pub const MAX_IMAGES: usize = 4096;

impl Images {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fh6-kit image layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let mut images = Self {
            entries: Vec::with_capacity(64),
            files: Vec::new(),
            layout,
            max_dim: device.limits().max_texture_dimension_2d,
            bc: device.features().contains(wgpu::Features::TEXTURE_COMPRESSION_BC),
        };
        images.upload_rgba8(device, queue, 1, 1, &[255, 255, 255, 255], "white");
        images
    }

    pub(crate) fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    /// The bind group of `id`, falling back to the white texel for unknown ids.
    pub(crate) fn bind_group(&self, id: Option<ImageId>) -> Option<&wgpu::BindGroup> {
        let i = id.map_or(0, |i| i.0 as usize);
        self.entries
            .get(i)
            .or_else(|| self.entries.first())
            .map(|e| &e.bind_group)
    }

    /// Size in texels.
    #[must_use]
    pub fn size(&self, id: ImageId) -> Option<(u32, u32)> {
        self.entries.get(id.0 as usize).map(|e| e.size)
    }

    /// Number of mip levels.
    #[must_use]
    pub fn mip_count(&self, id: ImageId) -> u32 {
        self.entries.get(id.0 as usize).map_or(1, |e| e.mips)
    }

    pub(crate) fn is_straight(&self, id: ImageId) -> bool {
        self.entries.get(id.0 as usize).is_some_and(|e| e.straight)
    }

    /// Number of images (including the white texel).
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Never true (the white texel is always present).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Loads a `.png`, `.ktx2` or `.svg` file. SVGs are rasterised at `svg_size` (or their own
    /// size when `None`). A file loaded before, unchanged (same length and modification time)
    /// and at the same SVG size, returns the same image without reading it again.
    ///
    /// # Errors
    /// [`ImageError`] when the file cannot be read or decoded.
    pub fn load_file(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &Path,
        svg_size: Option<(u32, u32)>,
    ) -> Result<ImageId, ImageError> {
        let meta = std::fs::metadata(path).ok();
        let key = FileKey {
            path: path.to_path_buf(),
            svg_size,
            len: meta.as_ref().map_or(0, std::fs::Metadata::len),
            modified: meta.and_then(|m| m.modified().ok()),
        };
        if let Some(&(_, id)) = self.files.iter().find(|(k, _)| *k == key) {
            return Ok(id);
        }
        let bytes = std::fs::read(path).map_err(|source| ImageError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        let label = path.file_name().and_then(|n| n.to_str()).unwrap_or("image");
        let id = match ext.as_str() {
            "png" => self.load_png(device, queue, &bytes, label),
            "ktx2" => self.load_ktx2(device, queue, &bytes, label),
            "svg" => self.load_svg(device, queue, &bytes, svg_size, label),
            other => Err(ImageError::Unsupported(other.to_string())),
        }?;
        self.files.push((key, id));
        Ok(id)
    }

    /// Decodes a PNG (any colour type) and uploads it premultiplied with mips.
    ///
    /// # Errors
    /// [`ImageError::Png`] when decoding fails.
    pub fn load_png(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<ImageId, ImageError> {
        let (w, h, rgba) = decode_png(bytes)?;
        self.check_size(w, h)?;
        Ok(self.upload_rgba8(device, queue, w, h, &rgba, label))
    }

    /// Rasterises an SVG with resvg at `size` (its own size when `None`), fitted with its
    /// aspect ratio kept, and uploads it.
    ///
    /// # Errors
    /// [`ImageError::Svg`] when the SVG cannot be parsed.
    pub fn load_svg(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        size: Option<(u32, u32)>,
        label: &str,
    ) -> Result<ImageId, ImageError> {
        let (w, h, rgba) = rasterize_svg(bytes, size)?;
        self.check_size(w, h)?;
        Ok(self.upload_premultiplied_srgb8(device, queue, w, h, &rgba, label))
    }

    /// Loads a KTX2 texture: `R8G8B8A8_SRGB` / `_UNORM`, the HDR formats `R16G16B16A16_SFLOAT`
    /// and `E5B9G9R9_UFLOAT_PACK32` (environment maps), or BC7 when the device supports BC
    /// compression; each optionally zstd-supercompressed. Stored mips are used as-is.
    ///
    /// # Errors
    /// [`ImageError::Ktx2`] for malformed files and unsupported formats.
    pub fn load_ktx2(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<ImageId, ImageError> {
        let reader = ktx2::Reader::new(bytes).map_err(|e| ImageError::Ktx2(e.to_string()))?;
        let header = reader.header();
        let (w, h) = (header.pixel_width, header.pixel_height.max(1));
        self.check_size(w, h)?;
        if header.layer_count > 1 || header.face_count != 1 || header.pixel_depth > 1 {
            return Err(ImageError::Ktx2("only single 2D images are supported".into()));
        }
        let fmt = header
            .format
            .ok_or_else(|| ImageError::Ktx2("Basis Universal is not supported".into()))?;
        let premultiplied = reader.is_alpha_premultiplied().unwrap_or(false);
        let (format, block, bytes_per_block) = if fmt == ktx2::Format::R8G8B8A8_SRGB {
            (wgpu::TextureFormat::Rgba8UnormSrgb, 1, 4)
        } else if fmt == ktx2::Format::R8G8B8A8_UNORM {
            (wgpu::TextureFormat::Rgba8Unorm, 1, 4)
        } else if fmt == ktx2::Format::BC7_SRGB_BLOCK && self.bc {
            (wgpu::TextureFormat::Bc7RgbaUnormSrgb, 4, 16)
        } else if fmt == ktx2::Format::BC7_UNORM_BLOCK && self.bc {
            (wgpu::TextureFormat::Bc7RgbaUnorm, 4, 16)
        } else if fmt == ktx2::Format::R16G16B16A16_SFLOAT {
            // Linear HDR radiance (environment maps): filterable everywhere, no feature needed.
            (wgpu::TextureFormat::Rgba16Float, 1, 8)
        } else if fmt == ktx2::Format::E5B9G9R9_UFLOAT_PACK32 {
            (wgpu::TextureFormat::Rgb9e5Ufloat, 1, 4)
        } else {
            return Err(ImageError::Ktx2(format!("unsupported format {fmt:?}")));
        };
        let zstd = match header.supercompression_scheme {
            None => false,
            Some(s) if s == ktx2::SupercompressionScheme::Zstandard => true,
            Some(s) => return Err(ImageError::Ktx2(format!("unsupported supercompression {s:?}"))),
        };
        let mips = header.level_count.max(1);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: w.div_ceil(block) * block,
                height: h.div_ceil(block) * block,
                depth_or_array_layers: 1,
            },
            mip_level_count: mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for (level, data) in reader.levels().enumerate() {
            let raw: Vec<u8> = if zstd {
                let mut out = Vec::with_capacity(usize::try_from(data.uncompressed_byte_length).unwrap_or(0));
                ruzstd::decoding::StreamingDecoder::new(data.data)
                    .map_err(|e| ImageError::Ktx2(e.to_string()))?
                    .read_to_end(&mut out)
                    .map_err(|e| ImageError::Ktx2(e.to_string()))?;
                out
            } else {
                data.data.to_vec()
            };
            let lw = (w >> level).max(1);
            let lh = (h >> level).max(1);
            let blocks_w = lw.div_ceil(block);
            let blocks_h = lh.div_ceil(block);
            let needed = (blocks_w * blocks_h * bytes_per_block) as usize;
            if raw.len() < needed {
                return Err(ImageError::Ktx2(format!("level {level} is truncated")));
            }
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: level as u32,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &raw[..needed],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(blocks_w * bytes_per_block),
                    rows_per_image: Some(blocks_h),
                },
                wgpu::Extent3d {
                    width: blocks_w * block,
                    height: blocks_h * block,
                    depth_or_array_layers: 1,
                },
            );
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Ok(self.push(device, Some(texture), &view, (w, h), mips, !premultiplied))
    }

    /// Uploads straight-alpha sRGB RGBA8 pixels (premultiplied in linear light, with mips).
    pub fn upload_rgba8(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        rgba: &[u8],
        label: &str,
    ) -> ImageId {
        let premul = premultiply_srgb8(rgba);
        self.upload_premultiplied_srgb8(device, queue, width, height, &premul, label)
    }

    /// Uploads premultiplied sRGB-encoded RGBA8 pixels (tiny-skia's layout) with mips.
    pub fn upload_premultiplied_srgb8(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        premul: &[u8],
        label: &str,
    ) -> ImageId {
        let (width, height) = (width.max(1), height.max(1));
        let mips = width.max(height).ilog2() + 1;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mut level = premul.to_vec();
        let (mut w, mut h) = (width, height);
        for mip in 0..mips {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: mip,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &level,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(w * 4),
                    rows_per_image: Some(h),
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
            if mip + 1 < mips {
                let (nw, nh, next) = downsample_srgb8(&level, w, h);
                level = next;
                w = nw;
                h = nh;
            }
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.push(device, Some(texture), &view, (width, height), mips, false)
    }

    /// Registers a view owned elsewhere (linear premultiplied, e.g. the fluid dye) so paints
    /// can sample it. Call [`Images::replace_view`] if the view is recreated.
    pub fn register_view(&mut self, device: &wgpu::Device, view: &wgpu::TextureView, size: (u32, u32)) -> ImageId {
        self.push(device, None, view, size, 1, false)
    }

    /// Points an existing image slot at a new view.
    pub fn replace_view(&mut self, device: &wgpu::Device, id: ImageId, view: &wgpu::TextureView, size: (u32, u32)) {
        let bind_group = self.make_bind_group(device, view);
        if let Some(e) = self.entries.get_mut(id.0 as usize) {
            e.bind_group = bind_group;
            e.size = size;
            e.mips = 1;
            e._texture = None;
        }
    }

    fn check_size(&self, w: u32, h: u32) -> Result<(), ImageError> {
        if w == 0 || h == 0 || w > self.max_dim || h > self.max_dim || self.entries.len() >= MAX_IMAGES {
            return Err(ImageError::Size {
                width: w,
                height: h,
                max: self.max_dim,
            });
        }
        Ok(())
    }

    fn make_bind_group(&self, device: &wgpu::Device, view: &wgpu::TextureView) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fh6-kit image"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            }],
        })
    }

    fn push(
        &mut self,
        device: &wgpu::Device,
        texture: Option<wgpu::Texture>,
        view: &wgpu::TextureView,
        size: (u32, u32),
        mips: u32,
        straight: bool,
    ) -> ImageId {
        let bind_group = self.make_bind_group(device, view);
        self.entries.push(Entry {
            _texture: texture,
            bind_group,
            size,
            mips,
            straight,
        });
        ImageId((self.entries.len() - 1) as u32)
    }
}

/// Decodes any PNG into straight-alpha sRGB RGBA8.
///
/// # Errors
/// [`ImageError::Png`] on malformed input.
pub fn decode_png(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), ImageError> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|e| ImageError::Png(e.to_string()))?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| ImageError::Png(e.to_string()))?;
    let (w, h) = (info.width, info.height);
    let px = (w * h) as usize;
    let src = &buf[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => src.to_vec(),
        png::ColorType::Rgb => src.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => src.chunks_exact(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        png::ColorType::Grayscale => src.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return Err(ImageError::Png("palette not expanded".into())),
    };
    if rgba.len() != px * 4 {
        return Err(ImageError::Png("unexpected buffer size".into()));
    }
    Ok((w, h, rgba))
}

/// Rasterises an SVG to premultiplied sRGB RGBA8 at `size` (aspect kept, centred).
///
/// # Errors
/// [`ImageError::Svg`] when parsing fails or the size is zero.
pub fn rasterize_svg(bytes: &[u8], size: Option<(u32, u32)>) -> Result<(u32, u32, Vec<u8>), ImageError> {
    use resvg::{tiny_skia, usvg};
    let tree = usvg::Tree::from_data(bytes, &usvg::Options::default()).map_err(|e| ImageError::Svg(e.to_string()))?;
    let natural = tree.size();
    let (w, h) = size.unwrap_or((natural.width().ceil() as u32, natural.height().ceil() as u32));
    let mut pixmap =
        tiny_skia::Pixmap::new(w.max(1), h.max(1)).ok_or_else(|| ImageError::Svg(format!("bad size {w}x{h}")))?;
    let k = (w as f32 / natural.width()).min(h as f32 / natural.height());
    let dx = (w as f32 - natural.width() * k) * 0.5;
    let dy = (h as f32 - natural.height() * k) * 0.5;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(k, k).post_translate(dx, dy),
        &mut pixmap.as_mut(),
    );
    Ok((pixmap.width(), pixmap.height(), pixmap.take()))
}

/// The linear value of each 8-bit sRGB code.
fn srgb8_to_linear_table() -> [f32; 256] {
    let mut table = [0.0_f32; 256];
    for (code, v) in table.iter_mut().enumerate() {
        *v = srgb_to_linear(code as f32 / 255.0);
    }
    table
}

/// Straight sRGB RGBA8 to premultiplied (in linear light), re-encoded as sRGB RGBA8. Opaque
/// and empty pixels need no arithmetic, which is most of a 4K plate.
#[must_use]
pub fn premultiply_srgb8(rgba: &[u8]) -> Vec<u8> {
    let linear = srgb8_to_linear_table();
    let mut out = Vec::with_capacity(rgba.len());
    for p in rgba.chunks_exact(4) {
        match p[3] {
            255 => out.extend_from_slice(p),
            0 => out.extend_from_slice(&[0, 0, 0, 0]),
            alpha => {
                let a = f32::from(alpha) / 255.0;
                let enc = |c: u8| (linear_to_srgb(linear[usize::from(c)] * a) * 255.0).round() as u8;
                out.extend_from_slice(&[enc(p[0]), enc(p[1]), enc(p[2]), alpha]);
            }
        }
    }
    out
}

/// Halves a premultiplied sRGB RGBA8 image with a 2x2 box filter in linear light.
fn downsample_srgb8(src: &[u8], w: u32, h: u32) -> (u32, u32, Vec<u8>) {
    let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
    let linear = srgb8_to_linear_table();
    let mut out = vec![0u8; (nw * nh * 4) as usize];
    let index = |x: u32, y: u32| ((y.min(h - 1) * w + x.min(w - 1)) * 4) as usize;
    for y in 0..nh {
        for x in 0..nw {
            let (sx, sy) = (x * 2, y * 2);
            let taps = [
                index(sx, sy),
                index(sx + 1, sy),
                index(sx, sy + 1),
                index(sx + 1, sy + 1),
            ];
            let o = ((y * nw + x) * 4) as usize;
            for c in 0..4 {
                let codes = taps.map(|i| src[i + c]);
                out[o + c] = if codes[0] == codes[1] && codes[0] == codes[2] && codes[0] == codes[3] {
                    // Four equal texels average to themselves (flat areas need no arithmetic).
                    codes[0]
                } else {
                    let value = |code: u8| {
                        if c == 3 {
                            f32::from(code) / 255.0
                        } else {
                            linear[usize::from(code)]
                        }
                    };
                    let v = (value(codes[0]) + value(codes[1]) + value(codes[2]) + value(codes[3])) * 0.25;
                    let v = if c == 3 { v } else { linear_to_srgb(v) };
                    (v.clamp(0.0, 1.0) * 255.0).round() as u8
                };
            }
        }
    }
    (nw, nh, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premultiply_keeps_opaque_and_zeroes_transparent() {
        let out = premultiply_srgb8(&[200, 100, 50, 255, 200, 100, 50, 0]);
        assert_eq!(&out[..4], &[200, 100, 50, 255]);
        assert_eq!(&out[4..], &[0, 0, 0, 0]);
    }

    #[test]
    fn downsample_halves() {
        let src = vec![255u8; 4 * 4 * 4];
        let (w, h, out) = downsample_srgb8(&src, 4, 4);
        assert_eq!((w, h), (2, 2));
        assert!(out.iter().all(|v| *v == 255));
    }

    /// The arithmetic the fast paths skip: every pixel through decode, scale and encode.
    fn premultiply_slow(rgba: &[u8]) -> Vec<u8> {
        rgba.chunks_exact(4)
            .flat_map(|p| {
                let a = f32::from(p[3]) / 255.0;
                let enc = |c: u8| (linear_to_srgb(srgb_to_linear(f32::from(c) / 255.0) * a) * 255.0).round() as u8;
                [enc(p[0]), enc(p[1]), enc(p[2]), p[3]]
            })
            .collect()
    }

    fn downsample_slow(src: &[u8], w: u32, h: u32) -> Vec<u8> {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut out = vec![0u8; (nw * nh * 4) as usize];
        let at = |x: u32, y: u32, c: usize| -> f32 {
            let v = f32::from(src[((y.min(h - 1) * w + x.min(w - 1)) * 4) as usize + c]) / 255.0;
            if c == 3 {
                v
            } else {
                srgb_to_linear(v)
            }
        };
        for y in 0..nh {
            for x in 0..nw {
                for c in 0..4 {
                    let (sx, sy) = (x * 2, y * 2);
                    let v = (at(sx, sy, c) + at(sx + 1, sy, c) + at(sx, sy + 1, c) + at(sx + 1, sy + 1, c)) * 0.25;
                    let v = if c == 3 { v } else { linear_to_srgb(v) };
                    out[((y * nw + x) * 4) as usize + c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        }
        out
    }

    #[test]
    fn the_fast_paths_change_no_pixel() {
        // Every code survives decode and encode, which is what the opaque and flat paths rely on.
        for code in 0..=255u8 {
            let back = (linear_to_srgb(srgb_to_linear(f32::from(code) / 255.0)) * 255.0).round() as u8;
            assert_eq!(back, code);
        }
        // Pseudo-random texels with opaque, empty, partial and flat runs (odd sizes too).
        let mut state = 0x2545_F491_4F6C_DD1D_u64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 32) as u8
        };
        for (w, h) in [(8u32, 8u32), (7, 5), (1, 4), (16, 2)] {
            let mut px = Vec::with_capacity((w * h * 4) as usize);
            for i in 0..w * h {
                let alpha = match i % 5 {
                    0 => 255,
                    1 => 0,
                    _ => next(),
                };
                let grey = next();
                if i % 3 == 0 {
                    px.extend_from_slice(&[grey, grey, grey, alpha]);
                } else {
                    px.extend_from_slice(&[next(), next(), 200, alpha]);
                }
            }
            let premul = premultiply_srgb8(&px);
            assert_eq!(premul, premultiply_slow(&px), "premultiply {w}x{h}");
            let (nw, nh, mip) = downsample_srgb8(&premul, w, h);
            assert_eq!((nw, nh), ((w / 2).max(1), (h / 2).max(1)));
            assert_eq!(mip, downsample_slow(&premul, w, h), "mip of {w}x{h}");
        }
    }

    #[test]
    fn svg_rasterises_with_aspect() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="20" height="10" fill="#ff0000"/></svg>"##;
        let (w, h, px) = rasterize_svg(svg, Some((40, 40))).expect("svg");
        assert_eq!((w, h), (40, 40));
        let row = (20 * 40 * 4) as usize;
        assert_eq!(&px[row..row + 4], &[255, 0, 0, 255], "centre band is red");
        assert_eq!(px[3], 0, "letterbox top is transparent");
    }

    #[test]
    fn png_round_trip() {
        let mut bytes = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut bytes, 2, 1);
            enc.set_color(png::ColorType::Rgb);
            enc.set_depth(png::BitDepth::Eight);
            let mut w = enc.write_header().expect("header");
            w.write_image_data(&[1, 2, 3, 4, 5, 6]).expect("data");
        }
        let (w, h, rgba) = decode_png(&bytes).expect("decode");
        assert_eq!((w, h), (2, 1));
        assert_eq!(rgba, vec![1, 2, 3, 255, 4, 5, 6, 255]);
    }
}
