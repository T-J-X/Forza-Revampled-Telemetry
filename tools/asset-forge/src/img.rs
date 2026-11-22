//! A small float image type and the operations the pipeline needs: PNG/EXR in, PNG 8/16-bit
//! and KTX2 out, resampling, seamless tiling, normals and flow from height, atlases, mips and
//! the seam metrics QA uses.
//!
//! Values are stored as they will be encoded (0..1). Colour images hold sRGB-encoded values;
//! data images (normals, flow, masks) hold linear values. Nothing converts implicitly.
use crate::error::{image_err, io_err, Result};
use std::path::Path;

/// A float image with 1..=4 interleaved channels.
#[derive(Debug, Clone, PartialEq)]
pub struct Img {
    /// Width in pixels.
    pub w: usize,
    /// Height in pixels.
    pub h: usize,
    /// Channels per pixel (1..=4).
    pub c: usize,
    /// Row-major interleaved values.
    pub data: Vec<f32>,
}

impl Img {
    /// A black (all-zero) image.
    pub fn new(w: usize, h: usize, c: usize) -> Self {
        Self {
            w,
            h,
            c,
            data: vec![0.0; w * h * c],
        }
    }

    /// An image filled from `f(x, y) -> [values; c]`.
    pub fn from_fn(w: usize, h: usize, c: usize, mut f: impl FnMut(usize, usize) -> [f32; 4]) -> Self {
        let mut img = Self::new(w, h, c);
        for y in 0..h {
            for x in 0..w {
                let v = f(x, y);
                let i = (y * w + x) * c;
                img.data[i..i + c].copy_from_slice(&v[..c]);
            }
        }
        img
    }

    /// Value of channel `ch` at (x, y).
    #[inline]
    pub fn get(&self, x: usize, y: usize, ch: usize) -> f32 {
        self.data[(y * self.w + x) * self.c + ch]
    }

    /// Sets channel `ch` at (x, y).
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, ch: usize, v: f32) {
        self.data[(y * self.w + x) * self.c + ch] = v;
    }

    /// Value with wrap-around addressing.
    #[inline]
    pub fn get_wrap(&self, x: isize, y: isize, ch: usize) -> f32 {
        let xx = x.rem_euclid(self.w as isize) as usize;
        let yy = y.rem_euclid(self.h as isize) as usize;
        self.get(xx, yy, ch)
    }

    /// Value with clamped addressing.
    #[inline]
    pub fn get_clamp(&self, x: isize, y: isize, ch: usize) -> f32 {
        let xx = x.clamp(0, self.w as isize - 1) as usize;
        let yy = y.clamp(0, self.h as isize - 1) as usize;
        self.get(xx, yy, ch)
    }

    /// Bilinear sample at pixel-space (x, y) (pixel centres at +0.5), wrapping or clamping.
    pub fn sample(&self, x: f32, y: f32, ch: usize, wrap: bool) -> f32 {
        let fx = x - 0.5;
        let fy = y - 0.5;
        let x0 = fx.floor();
        let y0 = fy.floor();
        let tx = fx - x0;
        let ty = fy - y0;
        let (x0, y0) = (x0 as isize, y0 as isize);
        let g = |xx: isize, yy: isize| {
            if wrap {
                self.get_wrap(xx, yy, ch)
            } else {
                self.get_clamp(xx, yy, ch)
            }
        };
        let a = g(x0, y0) * (1.0 - tx) + g(x0 + 1, y0) * tx;
        let b = g(x0, y0 + 1) * (1.0 - tx) + g(x0 + 1, y0 + 1) * tx;
        a * (1.0 - ty) + b * ty
    }

    /// Converts to `c` channels: grey expands to RGB, missing alpha is 1, extra channels drop.
    pub fn to_channels(&self, c: usize) -> Img {
        Img::from_fn(self.w, self.h, c, |x, y| {
            let px = |ch: usize| self.get(x, y, ch);
            match self.c {
                1 => [px(0), px(0), px(0), 1.0],
                2 => [px(0), px(0), px(0), px(1)],
                3 => [px(0), px(1), px(2), 1.0],
                _ => [px(0), px(1), px(2), px(3)],
            }
        })
    }

    /// Resizes with a separable filter: box-averaging when shrinking, bilinear when growing.
    pub fn resize(&self, nw: usize, nh: usize, wrap: bool) -> Img {
        if nw == self.w && nh == self.h {
            return self.clone();
        }
        let sx = self.w as f32 / nw as f32;
        let sy = self.h as f32 / nh as f32;
        let kx = sx.max(1.0).ceil() as usize;
        let ky = sy.max(1.0).ceil() as usize;
        Img::from_fn(nw, nh, self.c, |x, y| {
            let mut out = [0.0f32; 4];
            let mut n = 0.0;
            for j in 0..ky {
                for i in 0..kx {
                    // Subsamples spread evenly over the source footprint of this output pixel.
                    let px = x as f32 * sx + (i as f32 + 0.5) * sx / kx as f32;
                    let py = y as f32 * sy + (j as f32 + 0.5) * sy / ky as f32;
                    for (ch, o) in out.iter_mut().enumerate().take(self.c) {
                        *o += self.sample(px, py, ch, wrap);
                    }
                    n += 1.0;
                }
            }
            for o in &mut out {
                *o /= n;
            }
            out
        })
    }

    /// Halves the size with a 2x2 box filter (one mip step). Odd sizes clamp.
    pub fn half(&self) -> Img {
        let nw = (self.w / 2).max(1);
        let nh = (self.h / 2).max(1);
        Img::from_fn(nw, nh, self.c, |x, y| {
            let mut out = [0.0f32; 4];
            for (ch, o) in out.iter_mut().enumerate().take(self.c) {
                let x0 = (2 * x) as isize;
                let y0 = (2 * y) as isize;
                *o = (self.get_clamp(x0, y0, ch)
                    + self.get_clamp(x0 + 1, y0, ch)
                    + self.get_clamp(x0, y0 + 1, ch)
                    + self.get_clamp(x0 + 1, y0 + 1, ch))
                    * 0.25;
            }
            out
        })
    }

    /// Crops the rectangle (x, y, w, h).
    pub fn crop(&self, x0: usize, y0: usize, w: usize, h: usize) -> Img {
        Img::from_fn(w, h, self.c, |x, y| {
            let mut out = [0.0f32; 4];
            for (ch, o) in out.iter_mut().enumerate().take(self.c) {
                *o = self.get_clamp((x0 + x) as isize, (y0 + y) as isize, ch);
            }
            out
        })
    }

    /// Pastes `src` (same channel count) with its top-left at (x0, y0).
    pub fn paste(&mut self, src: &Img, x0: usize, y0: usize) {
        for y in 0..src.h.min(self.h.saturating_sub(y0)) {
            for x in 0..src.w.min(self.w.saturating_sub(x0)) {
                for ch in 0..self.c.min(src.c) {
                    self.set(x0 + x, y0 + y, ch, src.get(x, y, ch));
                }
            }
        }
    }
}

// ---------------------------------------------------------------- colour helpers

/// sRGB encode of a linear value.
pub fn srgb_encode(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// sRGB decode to linear.
pub fn srgb_decode(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Rec. 709 luminance of sRGB-encoded RGB (decoded, weighted, re-encoded as a linear value).
pub fn luma(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * srgb_decode(r) + 0.7152 * srgb_decode(g) + 0.0722 * srgb_decode(b)
}

/// Smoothstep.
pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ---------------------------------------------------------------- IO

/// Loads any PNG or EXR into 0..1 floats (PNG values stay encoded; EXR is linear float).
pub fn load(path: &Path) -> Result<Img> {
    let dynimg = image::open(path).map_err(|e| image_err(path, e))?;
    let rgba = dynimg.to_rgba32f();
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    Ok(Img {
        w,
        h,
        c: 4,
        data: rgba.into_raw(),
    })
}

/// Loads an EXR and un-premultiplies nothing (Cycles writes straight colour with alpha coverage
/// for emission passes over a transparent film; values at partially covered edges are kept).
pub fn load_exr(path: &Path) -> Result<Img> {
    load(path)
}

fn png_color(c: usize) -> png::ColorType {
    match c {
        1 => png::ColorType::Grayscale,
        2 => png::ColorType::GrayscaleAlpha,
        3 => png::ColorType::Rgb,
        _ => png::ColorType::Rgba,
    }
}

/// Encodes `img` as PNG bytes (8 or 16 bit), optionally with an sRGB chunk.
pub fn encode_png(img: &Img, sixteen: bool, srgb: bool) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, img.w as u32, img.h as u32);
        enc.set_color(png_color(img.c));
        enc.set_depth(if sixteen {
            png::BitDepth::Sixteen
        } else {
            png::BitDepth::Eight
        });
        enc.set_compression(png::Compression::Fast);
        if srgb {
            enc.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        }
        let mut writer = enc.write_header().map_err(|e| image_err(Path::new("<png>"), e))?;
        let bytes: Vec<u8> = if sixteen {
            img.data
                .iter()
                .flat_map(|v| ((v.clamp(0.0, 1.0) * 65535.0).round() as u16).to_be_bytes())
                .collect()
        } else {
            img.data
                .iter()
                .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
                .collect()
        };
        writer
            .write_image_data(&bytes)
            .map_err(|e| image_err(Path::new("<png>"), e))?;
    }
    Ok(out)
}

/// Losslessly recompresses PNG bytes with oxipng (preset 2, metadata kept safe).
pub fn optimize_png(bytes: &[u8]) -> Vec<u8> {
    let mut opts = oxipng::Options::from_preset(2);
    opts.strip = oxipng::StripChunks::Safe;
    oxipng::optimize_from_memory(bytes, &opts).unwrap_or_else(|_| bytes.to_vec())
}

/// Writes `img` as an optimised PNG. `srgb` tags colour images.
pub fn save_png(img: &Img, path: &Path, sixteen: bool, srgb: bool) -> Result<u64> {
    let raw = encode_png(img, sixteen, srgb)?;
    let opt = optimize_png(&raw);
    write_file(path, &opt)
}

/// Writes bytes, creating parent folders. Returns the size.
pub fn write_file(path: &Path, bytes: &[u8]) -> Result<u64> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
    }
    std::fs::write(path, bytes).map_err(|e| io_err(path, e))?;
    Ok(bytes.len() as u64)
}

/// Saves a small preview (longest side `max`) as an 8-bit PNG for the contact sheet. Data maps
/// are shown as stored.
pub fn save_thumb(img: &Img, path: &Path, max: usize) -> Result<u64> {
    let k = (max as f32 / img.w.max(img.h) as f32).min(1.0);
    let tw = ((img.w as f32 * k).round() as usize).max(1);
    let th = ((img.h as f32 * k).round() as usize).max(1);
    let t = img.to_channels(4).resize(tw, th, false);
    save_png(&t, path, false, true)
}

// ---------------------------------------------------------------- texture maths

/// Makes an image tile seamlessly: blends it with its half-offset copy using a weight that is 0
/// at the original borders and 1 inside, so the borders come from the copy's interior.
/// `band` is the blend band as a fraction of the size (0.1..0.5).
pub fn make_seamless(img: &Img, band: f32) -> Img {
    // Two separable passes. Horizontal: near the left/right edges, cross-fade to the copy shifted
    // by half the width, whose own discontinuity sits in the middle column where its weight is 0.
    // Then the same vertically on that result, which keeps every row seamless. (A single 2D pass
    // leaks the shifted copy's wrap row into the side bands as a seam at mid-height.)
    let (w, h) = (img.w, img.h);
    let bw = (w as f32 * band).max(1.0);
    let bh = (h as f32 * band).max(1.0);
    let pass_x = Img::from_fn(w, h, img.c, |x, y| {
        let d = (x as f32 + 0.5).min(w as f32 - x as f32 - 0.5);
        let wgt = smoothstep(0.0, bw, d);
        let sx = (x + w / 2) % w;
        let mut out = [0.0f32; 4];
        for (ch, o) in out.iter_mut().enumerate().take(img.c) {
            *o = img.get(x, y, ch) * wgt + img.get(sx, y, ch) * (1.0 - wgt);
        }
        out
    });
    Img::from_fn(w, h, img.c, |x, y| {
        let d = (y as f32 + 0.5).min(h as f32 - y as f32 - 0.5);
        let wgt = smoothstep(0.0, bh, d);
        let sy = (y + h / 2) % h;
        let mut out = [0.0f32; 4];
        for (ch, o) in out.iter_mut().enumerate().take(img.c) {
            *o = pass_x.get(x, y, ch) * wgt + pass_x.get(x, sy, ch) * (1.0 - wgt);
        }
        out
    })
}

/// Four-fold mirror symmetry from the top-left quadrant (square Chladni plates are symmetric).
pub fn mirror4(img: &Img) -> Img {
    let (w, h) = (img.w, img.h);
    Img::from_fn(w, h, img.c, |x, y| {
        let sx = if x < w / 2 { x } else { w - 1 - x };
        let sy = if y < h / 2 { y } else { h - 1 - y };
        let mut out = [0.0f32; 4];
        for (ch, o) in out.iter_mut().enumerate().take(img.c) {
            *o = img.get(sx, sy, ch);
        }
        out
    })
}

/// Tile seam score: mean |difference| across the wrap edges divided by the mean |difference|
/// between neighbouring interior pixels. About 1 means the seam is as smooth as the texture.
pub fn tile_seam_score(img: &Img) -> f32 {
    let mut edge = 0.0f64;
    let mut ne = 0.0f64;
    for y in 0..img.h {
        for ch in 0..img.c.min(3) {
            edge += f64::from((img.get(0, y, ch) - img.get(img.w - 1, y, ch)).abs());
            ne += 1.0;
        }
    }
    for x in 0..img.w {
        for ch in 0..img.c.min(3) {
            edge += f64::from((img.get(x, 0, ch) - img.get(x, img.h - 1, ch)).abs());
            ne += 1.0;
        }
    }
    let mut inner = 0.0f64;
    let mut ni = 0.0f64;
    let step = (img.h / 64).max(1);
    for y in (0..img.h).step_by(step) {
        for x in 1..img.w {
            for ch in 0..img.c.min(3) {
                inner += f64::from((img.get(x, y, ch) - img.get(x - 1, y, ch)).abs());
                ni += 1.0;
            }
        }
    }
    let e = edge / ne.max(1.0);
    let i = inner / ni.max(1.0);
    (e / i.max(1e-6)) as f32
}

/// Loop seam score for a frame sequence: |last - first| divided by the mean |frame(i+1) - frame(i)|.
/// About 1 means the wrap is as smooth as any other step.
pub fn loop_seam_score(frames: &[Img]) -> f32 {
    if frames.len() < 3 {
        return f32::INFINITY;
    }
    let diff = |a: &Img, b: &Img| -> f64 {
        let n = a.data.len().min(b.data.len()).max(1);
        a.data
            .iter()
            .zip(&b.data)
            .map(|(x, y)| f64::from((x - y).abs()))
            .sum::<f64>()
            / n as f64
    };
    let mut sum = 0.0;
    for i in 0..frames.len() - 1 {
        sum += diff(&frames[i], &frames[i + 1]);
    }
    let mean = sum / (frames.len() - 1) as f64;
    let wrap = diff(&frames[frames.len() - 1], &frames[0]);
    (wrap / mean.max(1e-9)) as f32
}

/// Tangent-space normal map (RGB, +Y up in the image = green up) from a height map in channel
/// `ch`, where `strength` is height units per pixel. `wrap` uses tileable differences.
pub fn normal_from_height(hgt: &Img, ch: usize, strength: f32, wrap: bool) -> Img {
    Img::from_fn(hgt.w, hgt.h, 3, |x, y| {
        let (xi, yi) = (x as isize, y as isize);
        let g = |dx: isize, dy: isize| {
            if wrap {
                hgt.get_wrap(xi + dx, yi + dy, ch)
            } else {
                hgt.get_clamp(xi + dx, yi + dy, ch)
            }
        };
        // Sobel: image y grows downwards, normal map green points up.
        let gx = (g(1, -1) + 2.0 * g(1, 0) + g(1, 1)) - (g(-1, -1) + 2.0 * g(-1, 0) + g(-1, 1));
        let gy = (g(-1, 1) + 2.0 * g(0, 1) + g(1, 1)) - (g(-1, -1) + 2.0 * g(0, -1) + g(1, -1));
        let nx = -gx * strength / 8.0;
        let ny = gy * strength / 8.0;
        let len = (nx * nx + ny * ny + 1.0).sqrt();
        [nx / len * 0.5 + 0.5, ny / len * 0.5 + 0.5, 1.0 / len * 0.5 + 0.5, 1.0]
    })
}

/// Divergence-free flow map from a height map: the flow runs along the height contours
/// (the 2D curl of the height). RG = direction (0.5-centred), B = normalised speed.
pub fn flow_from_height(hgt: &Img, ch: usize, wrap: bool) -> Img {
    let mut grads = Vec::with_capacity(hgt.w * hgt.h);
    let mut max_mag = 1e-6f32;
    for y in 0..hgt.h {
        for x in 0..hgt.w {
            let (xi, yi) = (x as isize, y as isize);
            let g = |dx: isize, dy: isize| {
                if wrap {
                    hgt.get_wrap(xi + dx, yi + dy, ch)
                } else {
                    hgt.get_clamp(xi + dx, yi + dy, ch)
                }
            };
            let gx = (g(1, 0) - g(-1, 0)) * 0.5;
            let gy = (g(0, 1) - g(0, -1)) * 0.5;
            // curl: (dh/dy, -dh/dx), image y down -> flow y up for the shader.
            let fx = gy;
            let fy = gx;
            let m = (fx * fx + fy * fy).sqrt();
            max_mag = max_mag.max(m);
            grads.push((fx, fy, m));
        }
    }
    // Normalise speed against the 99th-percentile-ish maximum so a few spikes don't flatten it.
    let mut mags: Vec<f32> = grads.iter().map(|g| g.2).collect();
    mags.sort_by(|a, b| a.total_cmp(b));
    let p99 = mags
        .get(mags.len().saturating_mul(99) / 100)
        .copied()
        .unwrap_or(max_mag)
        .max(1e-6);
    Img::from_fn(hgt.w, hgt.h, 3, |x, y| {
        let (fx, fy, m) = grads[y * hgt.w + x];
        let s = (m / p99).min(1.0);
        let (dx, dy) = if m > 1e-9 { (fx / m, fy / m) } else { (0.0, 0.0) };
        [dx * s * 0.5 + 0.5, dy * s * 0.5 + 0.5, s, 1.0]
    })
}

/// Packs equally sized frames row-major into a `cols`-wide atlas.
pub fn atlas(frames: &[Img], cols: usize) -> Img {
    let (fw, fh, c) = frames.first().map_or((1, 1, 4), |f| (f.w, f.h, f.c));
    let rows = frames.len().div_ceil(cols.max(1));
    let mut out = Img::new(fw * cols, fh * rows, c);
    for (i, f) in frames.iter().enumerate() {
        out.paste(f, (i % cols) * fw, (i / cols) * fh);
    }
    out
}

/// Black-point lift removal for OLED plates: values at or below `b` become true 0.
pub fn crush_blacks(img: &mut Img, b: f32) {
    let c = img.c;
    for (i, v) in img.data.iter_mut().enumerate() {
        if c == 4 && i % 4 == 3 {
            continue;
        }
        *v = ((*v - b) / (1.0 - b)).max(0.0);
    }
}

/// Renormalises 0.5-centred normals stored in RGB (after filtering).
pub fn renormalize(img: &mut Img) {
    let c = img.c;
    for px in img.data.chunks_exact_mut(c) {
        let x = px[0] * 2.0 - 1.0;
        let y = px[1] * 2.0 - 1.0;
        let z = px[2] * 2.0 - 1.0;
        let l = (x * x + y * y + z * z).sqrt().max(1e-6);
        px[0] = x / l * 0.5 + 0.5;
        px[1] = y / l * 0.5 + 0.5;
        px[2] = z / l * 0.5 + 0.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seamless_wraps_cleanly() {
        let img = Img::from_fn(64, 64, 1, |x, _| [x as f32 / 63.0, 0.0, 0.0, 0.0]);
        assert!(tile_seam_score(&img) > 20.0, "a ramp has a hard seam");
        let s = make_seamless(&img, 0.25);
        assert!(tile_seam_score(&s) < 3.0, "seam score {}", tile_seam_score(&s));
    }

    #[test]
    fn loop_score_of_a_periodic_sequence_is_about_one() {
        let frames: Vec<Img> = (0..16)
            .map(|i| {
                let t = i as f32 / 16.0 * std::f32::consts::TAU;
                Img::from_fn(8, 8, 1, |x, _| {
                    [((x as f32) * 0.3 + t).sin() * 0.5 + 0.5, 0.0, 0.0, 0.0]
                })
            })
            .collect();
        let s = loop_seam_score(&frames);
        assert!((s - 1.0).abs() < 0.2, "loop score {s}");
    }

    #[test]
    fn flat_height_gives_flat_normal() {
        let h = Img::new(8, 8, 1);
        let n = normal_from_height(&h, 0, 4.0, true);
        assert!((n.get(3, 3, 2) - 1.0).abs() < 1e-6);
        assert!((n.get(3, 3, 0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn png16_round_trips() {
        let img = Img::from_fn(4, 3, 4, |x, y| [x as f32 / 3.0, y as f32 / 2.0, 0.25, 1.0]);
        let bytes = encode_png(&img, true, false).unwrap_or_default();
        let back = image::load_from_memory(&bytes).map(|d| d.to_rgba32f());
        let back = back.unwrap_or_else(|_| image::Rgba32FImage::new(1, 1));
        assert_eq!(back.width(), 4);
        assert!((back.get_pixel(3, 2)[0] - 1.0).abs() < 1e-4);
    }
}
