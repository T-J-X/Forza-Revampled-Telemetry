//! Anti-Orb Contoured Binnacle Ambient Wash.
//! Draws a Bezier-cowl-clipped
//! inverse-square radial glow tinted by the active cluster accent colour.
//! Uses premultiplied RGBA8 buffers for correct tiny-skia blending.

use tiny_skia::PixmapMut;
use crate::binnacle_center;

/// Maps normalized cowl feather [0.05..1.0] to feather sigma in pixels.
/// Linear mapping: 0.05 -> 6 px, 1.0 -> 64 px at scale 1.0.
pub fn feather_sigma_px(norm: f32, scale: f32) -> f32 {
    let norm_clamped = norm.clamp(0.05, 1.0);
    let min_px = 6.0;
    let max_px = 64.0;
    let t = (norm_clamped - 0.05) / (1.0 - 0.05);
    let px = min_px + (max_px - min_px) * t;
    px * scale
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirtyKey {
    accent_r: u8,
    accent_g: u8,
    accent_b: u8,
    intensity_bits: u32,
    feather_bits:   u32,
    scale_bits:     u32,
    canvas_w: u32,
    canvas_h: u32,
}

pub struct BinnacleAmbientWash {
    dirty_key: Option<DirtyKey>,
    roi_buf: Vec<u8>, // Premultiplied RGBA8 flattened, binnacle bounding rect only
    roi_x: u32,
    roi_y: u32,
    roi_w: u32,
    roi_h: u32,
}

impl BinnacleAmbientWash {
    pub fn new() -> Self {
        Self {
            dirty_key: None,
            roi_buf: Vec::new(),
            roi_x: 0, roi_y: 0, roi_w: 0, roi_h: 0,
        }
    }

    pub fn render(
        &mut self,
        pixmap: &mut PixmapMut,
        canvas_w: u32,
        canvas_h: u32,
        accent: (u8, u8, u8),
        intensity: f32,
        feather_sigma: f32,
        scale: f32,
    ) {
        let key = DirtyKey {
            accent_r: accent.0,
            accent_g: accent.1,
            accent_b: accent.2,
            intensity_bits: intensity.to_bits(),
            feather_bits:   feather_sigma.to_bits(),
            scale_bits:     scale.to_bits(),
            canvas_w,
            canvas_h,
        };

        if self.dirty_key.as_ref() != Some(&key) {
            self.recompute(canvas_w, canvas_h, accent, intensity, feather_sigma, scale);
            self.dirty_key = Some(key);
        }

        self.blit(pixmap, canvas_w);
    }

    fn recompute(
        &mut self,
        canvas_w: u32,
        canvas_h: u32,
        accent: (u8, u8, u8),
        intensity: f32,
        feather_sigma: f32,
        scale: f32,
    ) {
        let (cx, cy) = binnacle_center(canvas_w, canvas_h);

        let margin = (feather_sigma * 3.0).ceil() as u32 + 8;
        let half_w = (600.0 * scale) as u32 + margin;
        let half_h = (280.0 * scale) as u32 + margin;
        let roi_x = (cx as u32).saturating_sub(half_w);
        let roi_y = (cy as u32).saturating_sub(half_h);
        let roi_w = (half_w * 2).min(canvas_w - roi_x);
        let roi_h = (half_h * 2).min(canvas_h - roi_y);
        self.roi_x = roi_x;
        self.roi_y = roi_y;
        self.roi_w = roi_w;
        self.roi_h = roi_h;

        let y_apex = cy - 220.0 * scale;
        let r_arch = 520.0 * scale;
        let h_arch = 110.0 * scale;

        let neck_top = cy + 60.0 * scale;
        let neck_hw  = 55.0  * scale;

        let i_glow = intensity * 1.0_f32;
        let k      = 1.0 / (380.0 * scale * 380.0 * scale);
        let two_sigma_sq = 2.0 * feather_sigma * feather_sigma;

        let buf_len = (roi_w * roi_h * 4) as usize;
        self.roi_buf.resize(buf_len, 0u8);

        for py in 0..roi_h {
            for px in 0..roi_w {
                let wx = (roi_x + px) as f32;
                let wy = (roi_y + py) as f32;

                let dx_arch = wx - cx;
                let y_cowl = y_apex + (dx_arch * dx_arch / (r_arch * r_arch)) * h_arch;
                let d_cowl = wy - y_cowl;

                let base = ((py * roi_w + px) * 4) as usize;

                if d_cowl < -feather_sigma || (wy > neck_top && (wx - cx).abs() < neck_hw) {
                    self.roi_buf[base]     = 0;
                    self.roi_buf[base + 1] = 0;
                    self.roi_buf[base + 2] = 0;
                    self.roi_buf[base + 3] = 0;
                    continue;
                }

                let ddx = wx - cx;
                let ddy = wy - cy;
                let radial = i_glow / (1.0 + k * (ddx * ddx + 1.65 * ddy * ddy));
                let gauss = if d_cowl >= 0.0 {
                    1.0_f32
                } else {
                    (-(d_cowl * d_cowl) / two_sigma_sq).exp()
                };

                let l = (radial * gauss).clamp(0.0, 1.0);
                let alpha_f = l * 35.0;

                if alpha_f < 2.0 {
                    self.roi_buf[base]     = 0;
                    self.roi_buf[base + 1] = 0;
                    self.roi_buf[base + 2] = 0;
                    self.roi_buf[base + 3] = 0;
                } else {
                    let alpha = alpha_f as u8;
                    // Premultiplied alpha calculation
                    let r = ((accent.0 as u32 * alpha as u32) / 255) as u8;
                    let g = ((accent.1 as u32 * alpha as u32) / 255) as u8;
                    let b = ((accent.2 as u32 * alpha as u32) / 255) as u8;

                    self.roi_buf[base]     = r;
                    self.roi_buf[base + 1] = g;
                    self.roi_buf[base + 2] = b;
                    self.roi_buf[base + 3] = alpha;
                }
            }
        }
    }

    fn blit(&self, pixmap: &mut PixmapMut, canvas_w: u32) {
        let stride = canvas_w as usize * 4;
        let dst = pixmap.data_mut();

        for py in 0..self.roi_h {
            let src_row_start = (py * self.roi_w * 4) as usize;
            let dst_row_start = ((self.roi_y + py) as usize) * stride
                + (self.roi_x as usize) * 4;

            for px in 0..self.roi_w {
                let s = src_row_start + (px as usize) * 4;
                let d = dst_row_start + (px as usize) * 4;
                if d + 3 >= dst.len() { break; }

                let src_a = self.roi_buf[s + 3] as u32;
                if src_a == 0 { continue; }

                let inv_src_a = 255 - src_a;
                // premultiplied source-over
                dst[d]   = (self.roi_buf[s]   as u32 + (dst[d]   as u32 * inv_src_a) / 255) as u8;
                dst[d+1] = (self.roi_buf[s+1] as u32 + (dst[d+1] as u32 * inv_src_a) / 255) as u8;
                dst[d+2] = (self.roi_buf[s+2] as u32 + (dst[d+2] as u32 * inv_src_a) / 255) as u8;
                let dst_a = dst[d + 3] as u32;
                let out_a = src_a + dst_a * inv_src_a / 255;
                dst[d+3] = out_a as u8;
            }
        }
    }
}

impl Default for BinnacleAmbientWash {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Pixmap;

    #[test]
    fn wash_cavity_pixel_has_valid_premultiplied_alpha() {
        let mut wash = BinnacleAmbientWash::new();
        let mut pix = Pixmap::new(3840, 2160).unwrap();
        wash.render(&mut pix.as_mut(), 3840, 2160, (56, 189, 248), 1.0, 12.0, 2.0);
        
        // Sample inside the binnacle cavity (cx, cy)
        let cx = 1920;
        let cy = (2160.0 * 0.845) as usize;
        let idx = (cy * 3840 + cx) * 4;
        let a = pix.data()[idx + 3];
        let r = pix.data()[idx];
        
        assert!(a >= 12 && a <= 35, "cavity alpha must be within 12..=35, got {}", a);
        assert!(r <= a, "premultiplied R channel ({}) must be <= alpha ({})", r, a);
    }

    #[test]
    fn wash_above_cowl_is_pure_black() {
        let mut wash = BinnacleAmbientWash::new();
        let mut pix = Pixmap::new(3840, 2160).unwrap();
        wash.render(&mut pix.as_mut(), 3840, 2160, (56, 189, 248), 1.0, 12.0, 2.0);
        
        // Top-left corner (well above cowl dome)
        let idx = (10 * 3840 + 10) * 4;
        assert_eq!(pix.data()[idx + 3], 0, "pixel above cowl must have alpha=0");
    }

    #[test]
    fn steering_neck_cutout_is_pure_black() {
        let mut wash = BinnacleAmbientWash::new();
        let mut pix = Pixmap::new(3840, 2160).unwrap();
        wash.render(&mut pix.as_mut(), 3840, 2160, (56, 189, 248), 1.0, 12.0, 2.0);
        
        // Directly below binnacle center in steering neck cutout
        let cx = 1920;
        let ny = (2160.0 * 0.845 + 150.0) as usize;
        let idx = (ny * 3840 + cx) * 4;
        assert_eq!(pix.data()[idx + 3], 0, "steering neck cutout pixel must have alpha=0");
    }

    #[test]
    fn dirty_key_skips_recompute_on_same_params() {
        let mut wash = BinnacleAmbientWash::new();
        let mut pix = Pixmap::new(3840, 2160).unwrap();
        wash.render(&mut pix.as_mut(), 3840, 2160, (255, 60, 0), 0.8, 12.0, 2.0);
        let buf_ptr_1 = wash.roi_buf.as_ptr();
        wash.render(&mut pix.as_mut(), 3840, 2160, (255, 60, 0), 0.8, 12.0, 2.0);
        let buf_ptr_2 = wash.roi_buf.as_ptr();
        
        assert_eq!(buf_ptr_1, buf_ptr_2, "dirty-key must prevent unnecessary recompute buffer reallocation");
    }
}
