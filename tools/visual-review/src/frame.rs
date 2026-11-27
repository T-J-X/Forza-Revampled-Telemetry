//! Images as the OLED shows them: opaque sRGB, with transparent pixels black.
use crate::error::{image_err, Result};
use crate::geom::Rect;
use fh6_kit::color::{linear_to_srgb, srgb_to_linear};
use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::imageops::FilterType;
use image::{ExtendedColorType, ImageEncoder};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// An opaque sRGB image: a render composited over OLED black.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height * 3` sRGB bytes.
    pub rgb: Vec<u8>,
    /// The source's coverage, when it had transparent pixels.
    pub alpha: Option<Vec<u8>>,
    /// The source had straight (not premultiplied) alpha.
    pub straight: bool,
}

impl Frame {
    /// A frame filled with `c`.
    pub fn filled(width: u32, height: u32, c: [u8; 3]) -> Self {
        Self {
            width,
            height,
            rgb: c.repeat(width as usize * height as usize),
            alpha: None,
            straight: false,
        }
    }

    /// Reads a PNG or JPEG and composites it over black. `straight` says the file has straight
    /// (not premultiplied) alpha, as design-tool exports do; renders from the kit are
    /// premultiplied.
    ///
    /// # Errors
    /// An image error when the file cannot be read or decoded.
    pub fn load(path: &Path, straight: bool) -> Result<Self> {
        let img = image::open(path).map_err(|e| image_err(path, e))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(Self::from_rgba(width, height, rgba.as_raw(), straight))
    }

    /// From RGBA8, over black. Premultiplied data (the repo's convention: what the kit renders
    /// and DWM composites, where light beyond coverage may even exceed alpha) keeps its colour;
    /// straight data is multiplied by alpha.
    pub fn from_rgba(width: u32, height: u32, px: &[u8], straight: bool) -> Self {
        let mut rgb = Vec::with_capacity(px.len() / 4 * 3);
        let mut opaque = true;
        for p in px.chunks_exact(4) {
            opaque &= p[3] == 255;
            if straight {
                let a = u32::from(p[3]);
                rgb.extend(p[..3].iter().map(|&c| ((u32::from(c) * a + 127) / 255) as u8));
            } else {
                rgb.extend_from_slice(&p[..3]);
            }
        }
        let alpha = (!opaque).then(|| px.chunks_exact(4).map(|p| p[3]).collect());
        Self {
            width,
            height,
            rgb,
            alpha,
            straight,
        }
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (y as usize * self.width as usize + x as usize) * 3
    }

    /// The pixel at `(x, y)`, black outside the frame.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 3] {
        if x >= self.width || y >= self.height {
            return [0; 3];
        }
        let i = self.index(x, y);
        self.rgb.get(i..i + 3).map_or([0; 3], |p| [p[0], p[1], p[2]])
    }

    /// Sets the pixel at `(x, y)` (ignored outside the frame).
    pub fn put(&mut self, x: u32, y: u32, c: [u8; 3]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let i = self.index(x, y);
        if let Some(p) = self.rgb.get_mut(i..i + 3) {
            p.copy_from_slice(&c);
        }
    }

    /// The part inside `r`, or `None` when `r` misses the frame.
    pub fn crop(&self, r: Rect) -> Option<Self> {
        let r = r.clamp_to(self.width, self.height)?;
        let (x, y, w, h) = r.parts();
        let (wu, row_px) = (w as usize, self.width as usize);
        let mut rgb = Vec::with_capacity(wu * h as usize * 3);
        let mut alpha = self.alpha.as_ref().map(|_| Vec::with_capacity(wu * h as usize));
        for row in y..y + h {
            let s = row as usize * row_px + x as usize;
            rgb.extend_from_slice(self.rgb.get(s * 3..(s + wu) * 3)?);
            if let (Some(dst), Some(src)) = (alpha.as_mut(), self.alpha.as_ref()) {
                dst.extend_from_slice(src.get(s..s + wu)?);
            }
        }
        Some(Self {
            width: w,
            height: h,
            rgb,
            alpha,
            straight: self.straight,
        })
    }

    /// Enlarged `n` times with nearest-neighbour sampling, so pixels stay visible as pixels.
    pub fn zoom(&self, n: u32) -> Self {
        if n <= 1 {
            return self.clone();
        }
        let n_us = n as usize;
        let mut rgb = Vec::with_capacity(self.rgb.len() * n_us * n_us);
        let mut row = Vec::with_capacity(self.width as usize * n_us * 3);
        for y in 0..self.height {
            row.clear();
            for x in 0..self.width {
                let p = self.pixel(x, y);
                for _ in 0..n {
                    row.extend_from_slice(&p);
                }
            }
            for _ in 0..n {
                rgb.extend_from_slice(&row);
            }
        }
        Self {
            width: self.width * n,
            height: self.height * n,
            rgb,
            alpha: None,
            straight: false,
        }
    }

    /// Resized to `w` x `h`. Shrinking filters in linear light (Lanczos), so thin bright strokes
    /// keep the light they have on the panel; enlarging uses nearest pixels.
    pub fn resize(&self, w: u32, h: u32) -> Self {
        let (w, h) = (w.max(1), h.max(1));
        if (w, h) == (self.width, self.height) {
            return self.clone();
        }
        if w >= self.width && h >= self.height {
            let Some(buf) = image::RgbImage::from_raw(self.width, self.height, self.rgb.clone()) else {
                return self.clone();
            };
            let out = image::imageops::resize(&buf, w, h, FilterType::Nearest);
            return Self {
                width: out.width(),
                height: out.height(),
                rgb: out.into_raw(),
                alpha: None,
                straight: false,
            };
        }
        let lut = linear_lut();
        let lin: Vec<f32> = self.rgb.iter().map(|&c| lut[usize::from(c)]).collect();
        let Some(buf) = image::Rgb32FImage::from_raw(self.width, self.height, lin) else {
            return self.clone();
        };
        let out = image::imageops::resize(&buf, w, h, FilterType::Lanczos3);
        let (ow, oh) = (out.width(), out.height());
        let rgb = out
            .into_raw()
            .into_iter()
            .map(|v| (linear_to_srgb(v.clamp(0.0, 1.0)) * 255.0).round() as u8)
            .collect();
        Self {
            width: ow,
            height: oh,
            rgb,
            alpha: None,
            straight: false,
        }
    }

    /// Copies `src` with its top-left corner at `(x, y)`, clipped to this frame.
    pub fn blit(&mut self, src: &Frame, x: u32, y: u32) {
        let w = src.width.min(self.width.saturating_sub(x)) as usize * 3;
        for sy in 0..src.height.min(self.height.saturating_sub(y)) {
            let s = src.index(0, sy);
            let d = self.index(x, y + sy);
            if let (Some(from), Some(to)) = (src.rgb.get(s..s + w), self.rgb.get_mut(d..d + w)) {
                to.copy_from_slice(from);
            }
        }
    }

    /// Fills `r`, clipped to the frame, with `c`.
    pub fn fill(&mut self, r: Rect, c: [u8; 3]) {
        let Some(r) = r.clamp_to(self.width, self.height) else {
            return;
        };
        let (x, y, w, h) = r.parts();
        for yy in y..y + h {
            for xx in x..x + w {
                self.put(xx, yy, c);
            }
        }
    }

    /// This frame composited over an opaque backdrop of the same size, using the source's
    /// coverage (premultiplied over: `rgb + backdrop * (1 - alpha)`). A frame without alpha is
    /// opaque and comes back unchanged.
    pub fn over(&self, backdrop: &Frame) -> Self {
        let Some(alpha) = &self.alpha else {
            return self.clone();
        };
        let mut rgb = Vec::with_capacity(self.rgb.len());
        for ((p, b), &a) in self.rgb.chunks_exact(3).zip(backdrop.rgb.chunks_exact(3)).zip(alpha) {
            let k = 255 - u32::from(a);
            for (&c, &bc) in p.iter().zip(b) {
                rgb.push((u32::from(c) + (u32::from(bc) * k + 127) / 255).min(255) as u8);
            }
        }
        Self {
            width: self.width,
            height: self.height,
            rgb,
            alpha: None,
            straight: false,
        }
    }

    /// Flipped left to right (a mirrored copy makes your own work look unfamiliar again).
    pub fn mirrored(&self) -> Self {
        let mut out = Self::filled(self.width, self.height, [0; 3]);
        for y in 0..self.height {
            for x in 0..self.width {
                out.put(self.width - 1 - x, y, self.pixel(x, y));
            }
        }
        out
    }

    /// Every channel scaled by `k` in encoded space (dims a frame under an overlay).
    pub fn dimmed(&self, k: f32) -> Self {
        let k = k.clamp(0.0, 1.0);
        Self {
            width: self.width,
            height: self.height,
            rgb: self.rgb.iter().map(|&c| (f32::from(c) * k).round() as u8).collect(),
            alpha: None,
            straight: false,
        }
    }

    /// Writes an RGB PNG, creating its folder.
    ///
    /// # Errors
    /// An image error when the name does not end in `.png` (the tool writes PNG only) or the
    /// file cannot be written.
    pub fn save(&self, path: &Path) -> Result<()> {
        if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("png")) {
            return Err(image_err(
                path,
                "the tool writes PNG files only: give a name ending in .png",
            ));
        }
        if path.is_dir() {
            return Err(image_err(path, "is a folder: give a file name ending in .png"));
        }
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| image_err(path, e))?;
        }
        let file = File::create(path).map_err(|e| image_err(path, e))?;
        let enc = PngEncoder::new_with_quality(BufWriter::new(file), CompressionType::Fast, PngFilter::Adaptive);
        enc.write_image(&self.rgb, self.width, self.height, ExtendedColorType::Rgb8)
            .map_err(|e| image_err(path, e))
    }
}

/// sRGB bytes to linear light.
pub fn linear_lut() -> [f32; 256] {
    let mut lut = [0.0f32; 256];
    for (i, v) in lut.iter_mut().enumerate() {
        *v = srgb_to_linear(i as f32 / 255.0);
    }
    lut
}

/// WCAG relative luminance of linear RGB.
pub fn luminance(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// WCAG contrast ratio between two relative luminances.
pub fn contrast(a: f32, b: f32) -> f32 {
    let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
    (hi + 0.05) / (lo + 0.05)
}

/// Linear-light RGB, for filtering the way light adds up.
#[derive(Debug, Clone, PartialEq)]
pub struct Linear {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Row-major pixels.
    pub px: Vec<[f32; 3]>,
}

impl Linear {
    /// Decodes a frame to linear light.
    pub fn from_frame(f: &Frame) -> Self {
        let lut = linear_lut();
        let px = f
            .rgb
            .chunks_exact(3)
            .map(|p| [lut[usize::from(p[0])], lut[usize::from(p[1])], lut[usize::from(p[2])]])
            .collect();
        Self {
            width: f.width,
            height: f.height,
            px,
        }
    }

    /// Encodes back to sRGB.
    pub fn to_frame(&self) -> Frame {
        let enc = |v: f32| (linear_to_srgb(v.clamp(0.0, 1.0)) * 255.0).round() as u8;
        Frame {
            width: self.width,
            height: self.height,
            rgb: self.px.iter().flat_map(|c| [enc(c[0]), enc(c[1]), enc(c[2])]).collect(),
            alpha: None,
            straight: false,
        }
    }

    /// The pixel at `(x, y)`, black outside.
    pub fn at(&self, x: u32, y: u32) -> [f32; 3] {
        if x >= self.width || y >= self.height {
            return [0.0; 3];
        }
        self.px
            .get(y as usize * self.width as usize + x as usize)
            .copied()
            .unwrap_or([0.0; 3])
    }

    /// Averaged over `k` x `k` blocks (a box downsample in linear light).
    pub fn box_down(&self, k: u32) -> Self {
        let k = k.max(1);
        let (w, h) = (self.width / k, self.height / k);
        let n = (k * k) as f32;
        let mut px = Vec::with_capacity(w as usize * h as usize);
        for y in 0..h {
            for x in 0..w {
                let mut acc = [0.0f32; 3];
                for dy in 0..k {
                    for dx in 0..k {
                        let c = self.at(x * k + dx, y * k + dy);
                        acc[0] += c[0];
                        acc[1] += c[1];
                        acc[2] += c[2];
                    }
                }
                px.push([acc[0] / n, acc[1] / n, acc[2] / n]);
            }
        }
        Self {
            width: w,
            height: h,
            px,
        }
    }

    /// Gaussian blur with standard deviation `sigma` px: separable, edges clamped. The kernel
    /// never reaches further than the image is wide or tall; a sigma that is not a positive
    /// finite number leaves the image as it is (callers validate it first).
    pub fn blur(&self, sigma: f32) -> Self {
        if !sigma.is_finite() || sigma <= 0.0 || self.px.is_empty() {
            return self.clone();
        }
        let limit = i64::from(self.width.max(self.height));
        let r = ((f64::from(sigma) * 3.0).ceil() as i64).clamp(1, limit.max(1));
        let mut kernel: Vec<f32> = (-r..=r)
            .map(|i| (-((i * i) as f32) / (2.0 * sigma * sigma)).exp())
            .collect();
        let sum: f32 = kernel.iter().sum();
        if !sum.is_finite() || sum <= 0.0 {
            return self.clone();
        }
        kernel.iter_mut().for_each(|k| *k /= sum);
        let (w, h) = (i64::from(self.width), i64::from(self.height));
        let pass = |src: &[[f32; 3]], horizontal: bool| -> Vec<[f32; 3]> {
            let mut out = Vec::with_capacity(src.len());
            for y in 0..h {
                for x in 0..w {
                    let mut acc = [0.0f32; 3];
                    for (j, k) in kernel.iter().enumerate() {
                        let o = j as i64 - r;
                        let (sx, sy) = if horizontal {
                            ((x + o).clamp(0, w - 1), y)
                        } else {
                            (x, (y + o).clamp(0, h - 1))
                        };
                        let c = src.get((sy * w + sx) as usize).copied().unwrap_or([0.0; 3]);
                        acc[0] += c[0] * k;
                        acc[1] += c[1] * k;
                        acc[2] += c[2] * k;
                    }
                    out.push(acc);
                }
            }
            out
        };
        let px = pass(&pass(&self.px, true), false);
        Self {
            width: self.width,
            height: self.height,
            px,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premultiplied_and_straight_alpha_both_end_over_black() {
        // Premultiplied: half-covered white is (128, 128, 128, 128) and stays 128, and light
        // beyond coverage (the kit's additive bloom, rgb above alpha) is kept as it is.
        let pre = Frame::from_rgba(3, 1, &[128, 128, 128, 128, 0, 0, 0, 0, 90, 90, 90, 0], false);
        assert!(!pre.straight);
        assert_eq!(pre.rgb, [128, 128, 128, 0, 0, 0, 90, 90, 90]);
        assert_eq!(pre.alpha, Some(vec![128, 0, 0]));
        // Straight: white at alpha 128 is multiplied out to 128.
        let straight = Frame::from_rgba(1, 1, &[255, 255, 255, 128], true);
        assert!(straight.straight);
        assert_eq!(straight.rgb, [128, 128, 128]);
        // Opaque images keep no alpha.
        assert_eq!(Frame::from_rgba(1, 1, &[10, 20, 30, 255], false).alpha, None);
    }

    #[test]
    fn shrinking_keeps_the_light_of_thin_strokes_and_save_wants_png() {
        // A 1 px white line in 4 px of black, shrunk 4x, keeps a quarter of its light: about
        // sRGB 137, where averaging the encoded bytes would give 64.
        let mut f = Frame::filled(8, 8, [0; 3]);
        for y in 0..8 {
            f.put(1, y, [255, 255, 255]);
        }
        let s = f.resize(2, 2);
        assert!(s.pixel(0, 0)[0] > 110, "{:?}", s.pixel(0, 0));
        let dir = std::env::temp_dir();
        assert!(f.save(&dir.join("visual-review-test.jpg")).is_err());
        assert!(f.save(&dir).is_err());
    }

    #[test]
    fn crop_zoom_blit_and_resize_keep_pixels_where_they_belong() {
        let mut f = Frame::filled(4, 3, [0; 3]);
        f.put(2, 1, [200, 100, 50]);
        let c = f.crop(Rect::new(1, 1, 2, 2)).expect("overlaps");
        assert_eq!((c.width, c.height), (2, 2));
        assert_eq!(c.pixel(1, 0), [200, 100, 50]);
        assert!(f.crop(Rect::new(10, 10, 2, 2)).is_none());
        let z = c.zoom(3);
        assert_eq!((z.width, z.height), (6, 6));
        assert_eq!(z.pixel(3, 0), [200, 100, 50]);
        assert_eq!(z.pixel(5, 2), [200, 100, 50]);
        assert_eq!(z.pixel(2, 0), [0, 0, 0]);
        let mut big = Frame::filled(5, 5, [9, 9, 9]);
        big.blit(&c, 4, 4);
        assert_eq!(big.pixel(4, 4), [0, 0, 0]);
        assert_eq!(big.pixel(3, 3), [9, 9, 9]);
        assert_eq!(f.resize(8, 6).pixel(5, 3), [200, 100, 50]);
        assert_eq!(f.pixel(99, 0), [0, 0, 0]);
    }

    #[test]
    fn box_down_averages_light_and_blur_keeps_a_flat_field() {
        let mut lin = Linear {
            width: 4,
            height: 4,
            px: vec![[0.0; 3]; 16],
        };
        lin.px[0] = [1.0, 1.0, 1.0];
        let d = lin.box_down(2);
        assert_eq!((d.width, d.height), (2, 2));
        assert!((d.at(0, 0)[1] - 0.25).abs() < 1e-6);
        let flat = Linear {
            width: 9,
            height: 7,
            px: vec![[0.5, 0.25, 0.125]; 63],
        };
        let b = flat.blur(1.5);
        assert!(b
            .px
            .iter()
            .all(|c| (c[0] - 0.5).abs() < 1e-5 && (c[2] - 0.125).abs() < 1e-5));
        // A point spreads but keeps its energy.
        let mut point = Linear {
            width: 21,
            height: 21,
            px: vec![[0.0; 3]; 441],
        };
        point.px[220] = [1.0; 3];
        let spread: f32 = point.blur(1.5).px.iter().map(|c| c[0]).sum();
        assert!((spread - 1.0).abs() < 1e-4);
    }

    #[test]
    fn over_uses_the_coverage() {
        // Half-covered white (premultiplied 128) over a white backdrop stays white; uncovered
        // pixels show the backdrop.
        let f = Frame::from_rgba(2, 1, &[128, 128, 128, 128, 0, 0, 0, 0], false);
        let bg = Frame::filled(2, 1, [255, 255, 255]);
        let o = f.over(&bg);
        assert_eq!(o.pixel(0, 0), [255, 255, 255]);
        assert_eq!(o.pixel(1, 0), [255, 255, 255]);
        let dark = f.over(&Frame::filled(2, 1, [0, 0, 0]));
        assert_eq!(dark.pixel(0, 0), [128, 128, 128]);
        // An opaque frame is unchanged.
        let opaque = Frame::filled(1, 1, [9, 9, 9]);
        assert_eq!(opaque.over(&Frame::filled(1, 1, [200, 0, 0])).pixel(0, 0), [9, 9, 9]);
    }

    #[test]
    fn contrast_follows_wcag() {
        assert!((contrast(1.0, 0.0) - 21.0).abs() < 1e-5);
        assert!((contrast(0.0, 1.0) - 21.0).abs() < 1e-5);
        assert!((contrast(0.18, 0.18) - 1.0).abs() < 1e-6);
        let lut = linear_lut();
        assert!(lut[0] == 0.0 && (lut[255] - 1.0).abs() < 1e-6);
        assert!((luminance([1.0, 1.0, 1.0]) - 1.0).abs() < 1e-6);
    }
}
