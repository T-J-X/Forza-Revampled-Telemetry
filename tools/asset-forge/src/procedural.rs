//! Exact-maths textures made in Rust: periodic noise, a caustic ray tracer, blue noise,
//! Chladni plates, curl noise, the sheen arch and the VFD / scanline masks. These are the maps
//! where an equation beats a render.
use crate::img::{smoothstep, Img};
use std::f32::consts::{PI, TAU};

/// A small deterministic RNG (SplitMix64).
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    /// Seeds it.
    pub fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    /// Next u64.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

fn hash2(x: i64, y: i64, seed: u64) -> u64 {
    let mut h = seed ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    h = h.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    h ^ (h >> 33)
}

/// Periodic gradient noise at (x, y) in lattice units with integer `period`, roughly -1..1.
pub fn pnoise(x: f32, y: f32, period: i64, seed: u64) -> f32 {
    let xi = x.floor();
    let yi = y.floor();
    let (fx, fy) = (x - xi, y - yi);
    let (xi, yi) = (xi as i64, yi as i64);
    let grad = |ix: i64, iy: i64, dx: f32, dy: f32| {
        let h = hash2(ix.rem_euclid(period), iy.rem_euclid(period), seed);
        let a = (h >> 40) as f32 / (1u64 << 24) as f32 * TAU;
        a.cos() * dx + a.sin() * dy
    };
    let fade = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let (u, v) = (fade(fx), fade(fy));
    let n00 = grad(xi, yi, fx, fy);
    let n10 = grad(xi + 1, yi, fx - 1.0, fy);
    let n01 = grad(xi, yi + 1, fx, fy - 1.0);
    let n11 = grad(xi + 1, yi + 1, fx - 1.0, fy - 1.0);
    let a = n00 + (n10 - n00) * u;
    let b = n01 + (n11 - n01) * u;
    (a + (b - a) * v) * 1.414
}

/// Tileable fBm over the unit square: `base` lattice cells per tile, `octaves` doublings.
pub fn fbm(u: f32, v: f32, base: i64, octaves: u32, gain: f32, seed: u64) -> f32 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut norm = 0.0;
    let mut p = base;
    for o in 0..octaves {
        sum += amp * pnoise(u * p as f32, v * p as f32, p, seed.wrapping_add(u64::from(o) * 7919));
        norm += amp;
        amp *= gain;
        p *= 2;
    }
    sum / norm
}

/// One wave of a periodic water surface.
#[derive(Debug, Clone, Copy)]
struct Wave {
    kx: f32,
    ky: f32,
    amp: f32,
    phase: f32,
    cycles: f32,
}

fn waves(seed: u64, count: usize, kmin: i32, kmax: i32) -> Vec<Wave> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::with_capacity(count);
    while out.len() < count {
        let kx = (rng.f32() * (2 * kmax + 1) as f32) as i32 - kmax;
        let ky = (rng.f32() * (2 * kmax + 1) as f32) as i32 - kmax;
        let k = ((kx * kx + ky * ky) as f32).sqrt();
        if k < kmin as f32 || k > kmax as f32 {
            continue;
        }
        // Integer temporal cycles keep the loop exact; faster for shorter waves (dispersion-like).
        let cycles = (k.sqrt()).round().max(1.0) * if rng.f32() < 0.5 { 1.0 } else { -1.0 };
        out.push(Wave {
            kx: kx as f32,
            ky: ky as f32,
            amp: 1.0 / k.powf(1.6),
            phase: rng.f32() * TAU,
            cycles,
        });
    }
    out
}

/// Caustic light under a periodic wavy surface at loop time `t` (0..1), traced with
/// `oversample`² parallel rays per pixel and splatted with wrap-around, so the result tiles and
/// loops exactly. Returns linear relative irradiance (mean 1).
pub fn caustic(size: usize, t: f32, seed: u64, focus: f32, oversample: usize) -> Img {
    let ws = waves(seed, 28, 2, 9);
    let n = size as f32;
    let mut acc = vec![0.0f32; size * size];
    let step = 1.0 / (n * oversample as f32);
    let total = size * oversample;
    let norm: f32 = ws.iter().map(|w| w.amp * (w.kx * w.kx + w.ky * w.ky).sqrt()).sum();
    for j in 0..total {
        for i in 0..total {
            let u = (i as f32 + 0.5) * step;
            let v = (j as f32 + 0.5) * step;
            let (mut gx, mut gy) = (0.0f32, 0.0f32);
            for w in &ws {
                let arg = TAU * (w.kx * u + w.ky * v) + w.phase + TAU * w.cycles * t;
                let c = arg.cos() * w.amp * TAU;
                gx += c * w.kx;
                gy += c * w.ky;
            }
            // Small-angle refraction: the ray shifts against the slope.
            let x = (u - focus * gx / (norm * TAU)) * n;
            let y = (v - focus * gy / (norm * TAU)) * n;
            let x0 = (x - 0.5).floor();
            let y0 = (y - 0.5).floor();
            let tx = x - 0.5 - x0;
            let ty = y - 0.5 - y0;
            let (x0, y0) = (x0 as i64, y0 as i64);
            let s = size as i64;
            let mut add = |xx: i64, yy: i64, wgt: f32| {
                let idx = (yy.rem_euclid(s) * s + xx.rem_euclid(s)) as usize;
                acc[idx] += wgt;
            };
            add(x0, y0, (1.0 - tx) * (1.0 - ty));
            add(x0 + 1, y0, tx * (1.0 - ty));
            add(x0, y0 + 1, (1.0 - tx) * ty);
            add(x0 + 1, y0 + 1, tx * ty);
        }
    }
    let per = (oversample * oversample) as f32;
    Img {
        w: size,
        h: size,
        c: 1,
        data: acc.into_iter().map(|a| a / per).collect(),
    }
}

/// Maps relative irradiance to a display value: dark water, bright filaments, black floor.
pub fn caustic_tone(irr: &Img, white: f32) -> Img {
    let mut out = irr.clone();
    for v in &mut out.data {
        let x = (*v / white).max(0.0);
        *v = (1.0 - (-1.6 * x * x).exp()).clamp(0.0, 1.0);
    }
    out
}

/// Separable Gaussian blur with wrap or clamp.
pub fn blur(img: &Img, sigma: f32, wrap: bool) -> Img {
    if sigma <= 0.0 {
        return img.clone();
    }
    let r = (sigma * 3.0).ceil() as isize;
    let k: Vec<f32> = (-r..=r).map(|i| (-(i * i) as f32 / (2.0 * sigma * sigma)).exp()).collect();
    let ks: f32 = k.iter().sum();
    let pass = |src: &Img, horizontal: bool| {
        Img::from_fn(src.w, src.h, src.c, |x, y| {
            let mut out = [0.0f32; 4];
            for (ti, kw) in k.iter().enumerate() {
                let d = ti as isize - r;
                let (sx, sy) = if horizontal {
                    (x as isize + d, y as isize)
                } else {
                    (x as isize, y as isize + d)
                };
                for (ch, o) in out.iter_mut().enumerate().take(src.c) {
                    let v = if wrap {
                        src.get_wrap(sx, sy, ch)
                    } else {
                        src.get_clamp(sx, sy, ch)
                    };
                    *o += v * kw / ks;
                }
            }
            out
        })
    };
    pass(&pass(img, true), false)
}

/// Void-and-cluster blue noise (Ulichney 1993): a `size`² rank map normalised to 0..1.
pub fn blue_noise(size: usize, seed: u64) -> Img {
    let n = size * size;
    let sigma = 1.9f32;
    let r = 6isize;
    let mut kern = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            kern.push((dx, dy, (-((dx * dx + dy * dy) as f32) / (2.0 * sigma * sigma)).exp()));
        }
    }
    let s = size as isize;
    let idx = |x: isize, y: isize| (y.rem_euclid(s) * s + x.rem_euclid(s)) as usize;
    let splat = |energy: &mut [f32], p: usize, sign: f32| {
        let (px, py) = ((p % size) as isize, (p / size) as isize);
        for &(dx, dy, w) in &kern {
            energy[idx(px + dx, py + dy)] += sign * w;
        }
    };
    let mut rng = Rng::new(seed);
    let mut on = vec![false; n];
    let mut energy = vec![0.0f32; n];
    let ones = n / 10;
    let mut placed = 0;
    while placed < ones {
        let p = (rng.next_u64() % n as u64) as usize;
        if !on[p] {
            on[p] = true;
            splat(&mut energy, p, 1.0);
            placed += 1;
        }
    }
    let argmax_on = |on: &[bool], e: &[f32]| (0..n).filter(|&i| on[i]).max_by(|&a, &b| e[a].total_cmp(&e[b]));
    let argmin_off = |on: &[bool], e: &[f32]| (0..n).filter(|&i| !on[i]).min_by(|&a, &b| e[a].total_cmp(&e[b]));
    // Phase 0: relax the initial pattern until the tightest cluster is the largest void.
    for _ in 0..n {
        let (Some(c), _) = (argmax_on(&on, &energy), ()) else { break };
        on[c] = false;
        splat(&mut energy, c, -1.0);
        let Some(v) = argmin_off(&on, &energy) else { break };
        on[v] = true;
        splat(&mut energy, v, 1.0);
        if v == c {
            break;
        }
    }
    let mut rank = vec![0usize; n];
    let proto_on = on.clone();
    let proto_e = energy.clone();
    // Phase 1: remove clusters, ranking downwards.
    let mut count = ones;
    while count > 0 {
        let Some(c) = argmax_on(&on, &energy) else { break };
        on[c] = false;
        splat(&mut energy, c, -1.0);
        count -= 1;
        rank[c] = count;
    }
    // Phases 2 and 3: fill voids, ranking upwards.
    on = proto_on;
    energy = proto_e;
    let mut count = ones;
    while count < n {
        let Some(v) = argmin_off(&on, &energy) else { break };
        on[v] = true;
        splat(&mut energy, v, 1.0);
        rank[v] = count;
        count += 1;
    }
    Img {
        w: size,
        h: size,
        c: 1,
        data: rank.into_iter().map(|r| (r as f32 + 0.5) / n as f32).collect(),
    }
}

/// A Chladni nodal-line plate for modes (n, m) on a square plate: the sand collects where
/// cos(nπx)cos(mπy) − cos(mπx)cos(nπy) = 0. Returns brightness (sand) 0..1.
pub fn chladni(size: usize, n: f32, m: f32, line: f32) -> Img {
    Img::from_fn(size, size, 1, |x, y| {
        let u = (x as f32 + 0.5) / size as f32;
        let v = (y as f32 + 0.5) / size as f32;
        let f = (n * PI * u).cos() * (m * PI * v).cos() - (m * PI * u).cos() * (n * PI * v).cos();
        let d = f.abs();
        let sand = 1.0 - smoothstep(0.0, line, d);
        let grain = 1.0 - smoothstep(line, line * 4.0, d);
        [sand * 0.85 + grain * 0.15, 0.0, 0.0, 0.0]
    })
}

/// Tileable curl-noise flow: RG = direction (0.5-centred), B = speed.
pub fn curl_noise(size: usize, base: i64, seed: u64) -> Img {
    let pot = Img::from_fn(size, size, 1, |x, y| {
        let u = (x as f32 + 0.5) / size as f32;
        let v = (y as f32 + 0.5) / size as f32;
        [fbm(u, v, base, 4, 0.5, seed), 0.0, 0.0, 0.0]
    });
    crate::img::flow_from_height(&pot, 0, true)
}

/// The arched crown sheen: white with straight alpha along an elliptical arch at the top.
pub fn sheen_arch(w: usize, h: usize) -> Img {
    Img::from_fn(w, h, 4, |x, y| {
        let u = (x as f32 + 0.5) / w as f32 - 0.5;
        let v = (y as f32 + 0.5) / h as f32;
        // Ellipse whose top edge is the arch; distance below the edge sets the falloff.
        let ex = u / 0.62;
        let arch = 0.06 + 0.5 * ex * ex;
        let d = v - arch;
        let band = if d < 0.0 {
            smoothstep(-0.035, 0.0, d)
        } else {
            (-d / 0.30).exp()
        };
        let side = 1.0 - smoothstep(0.30, 0.62, u.abs());
        let crown = 0.75 + 0.25 * (1.0 - smoothstep(0.0, 0.35, u.abs()));
        let a = (band * side * crown).clamp(0.0, 1.0);
        [1.0, 1.0, 1.0, a]
    })
}

/// Horizontal scanlines with a `period` px pitch (1 px dark line per period), as a mask.
pub fn scanlines(period: usize, height: usize) -> Img {
    Img::from_fn(8, height, 1, |_, y| {
        let p = y % period;
        let v = if p + 1 == period {
            0.18
        } else if p == 0 {
            0.85
        } else {
            1.0
        };
        [v, 0.0, 0.0, 0.0]
    })
}

/// A VFD mesh: the fine flat-top hexagonal wire in front of a vacuum-fluorescent display.
/// `pairs` column pairs across with hex size `s` px; the height is the nearest whole number of
/// rows, so it tiles to within half a pixel.
pub fn vfd_mesh(pairs: usize, s: f32) -> Img {
    let w = (3.0 * s * pairs as f32).round() as usize;
    let rows = (w as f32 / (3f32.sqrt() * s)).round().max(1.0);
    let h = (rows * 3f32.sqrt() * s).round() as usize;
    Img::from_fn(w, h, 1, |x, y| {
        // Hex lattice distance via axial coordinates; wire = near cell edges.
        let px = (x as f32 + 0.5) / s;
        let py = (y as f32 + 0.5) / s;
        let q = px * 2.0 / 3.0;
        let r = -px / 3.0 + 3f32.sqrt() / 3.0 * py;
        let (rq, rr, rs) = cube_round(q, r, -q - r);
        let dq = q - rq;
        let dr = r - rr;
        let ds = (-q - r) - rs;
        let edge = dq.abs().max(dr.abs()).max(ds.abs());
        let wire = smoothstep(0.38, 0.48, edge);
        [1.0 - 0.75 * wire, 0.0, 0.0, 0.0]
    })
}

fn cube_round(q: f32, r: f32, s: f32) -> (f32, f32, f32) {
    let (mut rq, mut rr, mut rs) = (q.round(), r.round(), s.round());
    let (dq, dr, ds) = ((rq - q).abs(), (rr - r).abs(), (rs - s).abs());
    if dq > dr && dq > ds {
        rq = -rr - rs;
    } else if dr > ds {
        rr = -rq - rs;
    } else {
        rs = -rq - rr;
    }
    (rq, rr, rs)
}

/// RGB phosphor-stripe triad mask (aperture grille), tileable, `px` px per stripe.
pub fn phosphor_triad(px: usize) -> Img {
    let w = px * 3;
    Img::from_fn(w * 4, w * 4, 3, |x, y| {
        let col = (x / px) % 3;
        let inx = (x % px) as f32 + 0.5;
        let s = smoothstep(0.0, 0.35 * px as f32, inx.min(px as f32 - inx));
        let gap = if y % (w * 2) == 0 { 0.6 } else { 1.0 };
        let mut c = [0.08, 0.08, 0.08, 1.0];
        c[col] = 0.08 + 0.92 * s * gap;
        c
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::img::{loop_seam_score, tile_seam_score};

    #[test]
    fn fbm_tiles() {
        let img = Img::from_fn(128, 128, 1, |x, y| {
            [fbm(x as f32 / 128.0, y as f32 / 128.0, 4, 4, 0.5, 3) * 0.5 + 0.5, 0.0, 0.0, 0.0]
        });
        assert!(tile_seam_score(&img) < 2.0, "{}", tile_seam_score(&img));
    }

    #[test]
    fn caustics_tile_and_loop() {
        let frames: Vec<Img> = (0..12).map(|i| caustic(64, i as f32 / 12.0, 5, 0.25, 2)).collect();
        assert!(tile_seam_score(&frames[0]) < 2.5, "tile {}", tile_seam_score(&frames[0]));
        let l = loop_seam_score(&frames);
        assert!(l < 1.6, "loop {l}");
        let mean: f32 = frames[0].data.iter().sum::<f32>() / frames[0].data.len() as f32;
        assert!((mean - 1.0).abs() < 0.01, "energy conserved: {mean}");
    }

    #[test]
    fn blue_noise_is_a_permutation() {
        let b = blue_noise(16, 1);
        let mut ranks: Vec<usize> = b.data.iter().map(|v| (v * 256.0) as usize).collect();
        ranks.sort_unstable();
        ranks.dedup();
        assert_eq!(ranks.len(), 256);
    }
}
