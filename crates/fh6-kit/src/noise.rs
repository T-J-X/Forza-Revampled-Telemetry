//! Deterministic, allocation-free gradient noise: 2D/3D simplex, fractal sums and a
//! divergence-free curl field for particle and dye advection.
//!
//! The lattice hash is pure 32-bit integer arithmetic (Chris Wellons' `lowbias32`), so the
//! CPU functions here and the WGSL functions in [`WGSL_NOISE`] pick the same gradients at the
//! same lattice points: a shader and a CPU particle system sampling the same coordinates see
//! the same field (to within float rounding).
//!
//! Simplex layout follows Stefan Gustavson's reference implementation (skewed simplex grid,
//! radial kernel `(r0 - |d|^2)^4`), with an integer hash in place of the permutation table.
//! Both 2D and 3D use `r0 = 0.5`: the kernel then vanishes before reaching any simplex that
//! does not contain the corner, so value and gradient are continuous everywhere (the classic
//! `0.6` in 3D leaves small seams that would break the curl field's zero divergence).
use crate::math::{finite_or, vec2, Vec2};

/// Skew factor for 2D: `(sqrt(3) - 1) / 2`.
const F2: f32 = 0.366_025_42;
/// Unskew factor for 2D: `(3 - sqrt(3)) / 6`.
const G2: f32 = 0.211_324_87;
/// Skew factor for 3D.
const F3: f32 = 1.0 / 3.0;
/// Unskew factor for 3D.
const G3: f32 = 1.0 / 6.0;
/// Output scale that keeps 2D simplex inside `-1..=1`.
const SCALE2: f32 = 40.0;
/// Output scale that keeps 3D simplex inside `-1..=1` (measured peak of the `r0 = 0.5`
/// kernel sum is about `0.0130`; the result is also clamped).
const SCALE3: f32 = 76.0;
/// Inputs are folded into this range before the lattice cast, so huge or non-finite inputs
/// never saturate the integer cell index (the field repeats every `2 * LIMIT` units).
const LIMIT: f32 = 1.0e6;
/// Most octaves [`fbm2`] sums.
pub const MAX_OCTAVES: u32 = 12;

/// Chris Wellons' `lowbias32` integer hash (exact on every platform and in WGSL).
#[inline]
const fn lowbias32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

#[inline]
const fn hash2(i: u32, j: u32) -> u32 {
    lowbias32(i ^ lowbias32(j.wrapping_add(0x9e37_79b9)))
}

#[inline]
const fn hash3(i: u32, j: u32, k: u32) -> u32 {
    lowbias32(i ^ lowbias32(j ^ lowbias32(k.wrapping_add(0x85eb_ca6b))))
}

/// Sanitises one input coordinate: non-finite becomes 0, and very large values wrap.
#[inline]
fn coord(v: f32) -> f32 {
    let v = finite_or(v, 0.0);
    if v.abs() < LIMIT {
        v
    } else {
        v % LIMIT
    }
}

/// Floor of `v` as a lattice index, as both the float and the wrapping integer.
#[inline]
fn cell(v: f32) -> (f32, u32) {
    let f = v.floor();
    // `f` is within +-LIMIT, so the cast is exact; the u32 reinterpretation is two's complement
    // (the same as WGSL `bitcast<u32>(i32(f))`).
    (f, (f as i32) as u32)
}

/// Gustavson's 2D gradient from the low 3 bits of `h`, dotted with `(x, y)`; returns
/// `(dot, gx, gy)`.
#[inline]
fn grad2(h: u32, x: f32, y: f32) -> (f32, f32, f32) {
    let h = h & 7;
    // u is x for h < 4 else y; v the other one. Signs from bits 0 and 1; v is doubled.
    let su = if h & 1 != 0 { -1.0 } else { 1.0 };
    let sv = if h & 2 != 0 { -2.0 } else { 2.0 };
    let (gx, gy) = if h < 4 { (su, sv) } else { (sv, su) };
    (gx * x + gy * y, gx, gy)
}

/// Gustavson's 3D gradient from the low 4 bits of `h` (the 12 cube-edge directions, 4 repeated).
#[inline]
fn grad3(h: u32) -> (f32, f32, f32) {
    match h & 15 {
        0 => (1.0, 1.0, 0.0),
        1 => (-1.0, 1.0, 0.0),
        2 => (1.0, -1.0, 0.0),
        3 => (-1.0, -1.0, 0.0),
        4 => (1.0, 0.0, 1.0),
        5 => (-1.0, 0.0, 1.0),
        6 => (1.0, 0.0, -1.0),
        7 => (-1.0, 0.0, -1.0),
        8 => (0.0, 1.0, 1.0),
        9 => (0.0, -1.0, 1.0),
        10 => (0.0, 1.0, -1.0),
        11 => (0.0, -1.0, -1.0),
        12 => (1.0, 1.0, 0.0),
        13 => (-1.0, 1.0, 0.0),
        14 => (0.0, -1.0, 1.0),
        _ => (0.0, -1.0, -1.0),
    }
}

/// 2D simplex noise. Smooth, roughly isotropic, range about `-1..=1` (never outside it), one
/// feature per unit. Non-finite inputs read as 0.
#[must_use]
pub fn simplex2(x: f32, y: f32) -> f32 {
    let (x, y) = (coord(x), coord(y));
    let s = (x + y) * F2;
    let (fi, i) = cell(x + s);
    let (fj, j) = cell(y + s);
    let t = (fi + fj) * G2;
    let x0 = x - (fi - t);
    let y0 = y - (fj - t);
    let (i1, j1) = if x0 > y0 { (1u32, 0u32) } else { (0u32, 1u32) };
    let x1 = x0 - i1 as f32 + G2;
    let y1 = y0 - j1 as f32 + G2;
    let x2 = x0 - 1.0 + 2.0 * G2;
    let y2 = y0 - 1.0 + 2.0 * G2;

    let corner = |h: u32, dx: f32, dy: f32| -> f32 {
        let t = 0.5 - dx * dx - dy * dy;
        if t <= 0.0 {
            0.0
        } else {
            let t2 = t * t;
            t2 * t2 * grad2(h, dx, dy).0
        }
    };
    let n0 = corner(hash2(i, j), x0, y0);
    let n1 = corner(hash2(i.wrapping_add(i1), j.wrapping_add(j1)), x1, y1);
    let n2 = corner(hash2(i.wrapping_add(1), j.wrapping_add(1)), x2, y2);
    (SCALE2 * (n0 + n1 + n2)).clamp(-1.0, 1.0)
}

/// 3D simplex noise and its analytic gradient: `(value, [d/dx, d/dy, d/dz])`. Range about
/// `-1..=1`. Non-finite inputs read as 0.
#[must_use]
pub fn simplex3_grad(x: f32, y: f32, z: f32) -> (f32, [f32; 3]) {
    let (x, y, z) = (coord(x), coord(y), coord(z));
    let s = (x + y + z) * F3;
    let (fi, i) = cell(x + s);
    let (fj, j) = cell(y + s);
    let (fk, k) = cell(z + s);
    let t = (fi + fj + fk) * G3;
    let x0 = x - (fi - t);
    let y0 = y - (fj - t);
    let z0 = z - (fk - t);
    // Which of the six simplices: the corner offsets in rank order of x0, y0, z0.
    let (o1, o2): ([u32; 3], [u32; 3]) = if x0 >= y0 {
        if y0 >= z0 {
            ([1, 0, 0], [1, 1, 0])
        } else if x0 >= z0 {
            ([1, 0, 0], [1, 0, 1])
        } else {
            ([0, 0, 1], [1, 0, 1])
        }
    } else if y0 < z0 {
        ([0, 0, 1], [0, 1, 1])
    } else if x0 < z0 {
        ([0, 1, 0], [0, 1, 1])
    } else {
        ([0, 1, 0], [1, 1, 0])
    };
    let d = [
        [x0, y0, z0],
        [x0 - o1[0] as f32 + G3, y0 - o1[1] as f32 + G3, z0 - o1[2] as f32 + G3],
        [
            x0 - o2[0] as f32 + 2.0 * G3,
            y0 - o2[1] as f32 + 2.0 * G3,
            z0 - o2[2] as f32 + 2.0 * G3,
        ],
        [x0 - 1.0 + 3.0 * G3, y0 - 1.0 + 3.0 * G3, z0 - 1.0 + 3.0 * G3],
    ];
    let h = [
        hash3(i, j, k),
        hash3(i.wrapping_add(o1[0]), j.wrapping_add(o1[1]), k.wrapping_add(o1[2])),
        hash3(i.wrapping_add(o2[0]), j.wrapping_add(o2[1]), k.wrapping_add(o2[2])),
        hash3(i.wrapping_add(1), j.wrapping_add(1), k.wrapping_add(1)),
    ];

    let mut value = 0.0;
    let mut grad = [0.0f32; 3];
    for (dc, hc) in d.iter().zip(h) {
        let t = 0.5 - dc[0] * dc[0] - dc[1] * dc[1] - dc[2] * dc[2];
        if t > 0.0 {
            let (gx, gy, gz) = grad3(hc);
            let gd = gx * dc[0] + gy * dc[1] + gz * dc[2];
            let t2 = t * t;
            let t4 = t2 * t2;
            value += t4 * gd;
            // d/dp [t^4 (g.d)] = t^4 g - 8 t^3 (g.d) d
            let k = 8.0 * t2 * t * gd;
            grad[0] += t4 * gx - k * dc[0];
            grad[1] += t4 * gy - k * dc[1];
            grad[2] += t4 * gz - k * dc[2];
        }
    }
    (
        (SCALE3 * value).clamp(-1.0, 1.0),
        [SCALE3 * grad[0], SCALE3 * grad[1], SCALE3 * grad[2]],
    )
}

/// 3D simplex noise, range about `-1..=1`. Use the third axis as time for an evolving 2D field.
#[must_use]
pub fn simplex3(x: f32, y: f32, z: f32) -> f32 {
    simplex3_grad(x, y, z).0
}

/// Fractal Brownian motion of [`simplex2`]: `octaves` layers (clamped to `1..=MAX_OCTAVES`),
/// each `lacunarity` times the frequency and `gain` times the amplitude of the last. The sum is
/// normalised by the total amplitude, so the range stays about `-1..=1` for any settings.
/// Typical values: `octaves = 4`, `lacunarity = 2.0`, `gain = 0.5`.
#[must_use]
pub fn fbm2(x: f32, y: f32, octaves: u32, lacunarity: f32, gain: f32) -> f32 {
    let octaves = octaves.clamp(1, MAX_OCTAVES);
    let lacunarity = finite_or(lacunarity, 2.0).clamp(0.0, 16.0);
    let gain = finite_or(gain, 0.5).clamp(0.0, 4.0);
    let (mut freq, mut amp) = (1.0f32, 1.0f32);
    let (mut sum, mut norm) = (0.0f32, 0.0f32);
    for octave in 0..octaves {
        // A per-octave offset decorrelates the layers at the origin.
        let off = octave as f32 * 17.17;
        sum += amp * simplex2(x * freq + off, y * freq - off);
        norm += amp;
        freq *= lacunarity;
        amp *= gain;
    }
    if norm > 0.0 {
        (sum / norm).clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

/// Divergence-free 2D velocity field: the curl of the scalar potential
/// `psi(x, y) = simplex3(x, y, t)`, i.e. `(d psi / dy, -d psi / dx)`. Evolves smoothly with
/// `t`. Magnitudes are typically `0..~7` per unit of input; scale the input to set the eddy size
/// and multiply the output to set the speed.
#[must_use]
pub fn curl2(x: f32, y: f32, t: f32) -> Vec2 {
    let (_, g) = simplex3_grad(x, y, t);
    vec2(g[1], -g[0])
}

/// WGSL versions of [`simplex2`], [`simplex3`], [`simplex3_grad`] and [`curl2`] (prefixed
/// `fh6_`), with the same integer lattice hash as the CPU functions. Concatenate it in front of a
/// cluster shader's source:
///
/// - `fn fh6_simplex2(p: vec2<f32>) -> f32`
/// - `fn fh6_simplex3(p: vec3<f32>) -> f32`
/// - `fn fh6_simplex3_grad(p: vec3<f32>) -> vec4<f32>` (`x` = value, `yzw` = gradient)
/// - `fn fh6_curl2(p: vec2<f32>, t: f32) -> vec2<f32>`
///
/// Uses no derivatives or textures, so it is valid in any stage and in non-uniform control flow.
pub const WGSL_NOISE: &str = r"
// ---- fh6-kit noise (crates/fh6-kit/src/noise.rs) ----
fn fh6_lowbias32(v: u32) -> u32 {
    var x = v;
    x = x ^ (x >> 16u);
    x = x * 0x7feb352du;
    x = x ^ (x >> 15u);
    x = x * 0x846ca68bu;
    x = x ^ (x >> 16u);
    return x;
}

fn fh6_hash2(i: u32, j: u32) -> u32 {
    return fh6_lowbias32(i ^ fh6_lowbias32(j + 0x9e3779b9u));
}

fn fh6_hash3(i: u32, j: u32, k: u32) -> u32 {
    return fh6_lowbias32(i ^ fh6_lowbias32(j ^ fh6_lowbias32(k + 0x85ebca6bu)));
}

fn fh6_noise_coord(v: f32) -> f32 {
    // Non-finite inputs are not detectable portably in WGSL; large values wrap like the CPU.
    if (abs(v) < 1.0e6) {
        return v;
    }
    return v - trunc(v / 1.0e6) * 1.0e6;
}

fn fh6_cell(v: f32) -> u32 {
    return bitcast<u32>(i32(floor(v)));
}

fn fh6_grad2(h: u32, d: vec2<f32>) -> f32 {
    let hh = h & 7u;
    let su = select(1.0, -1.0, (hh & 1u) != 0u);
    let sv = select(2.0, -2.0, (hh & 2u) != 0u);
    let g = select(vec2<f32>(sv, su), vec2<f32>(su, sv), hh < 4u);
    return dot(g, d);
}

fn fh6_grad3(h: u32) -> vec3<f32> {
    var g = vec3<f32>(0.0, -1.0, -1.0);
    switch (h & 15u) {
        case 0u: { g = vec3<f32>(1.0, 1.0, 0.0); }
        case 1u: { g = vec3<f32>(-1.0, 1.0, 0.0); }
        case 2u: { g = vec3<f32>(1.0, -1.0, 0.0); }
        case 3u: { g = vec3<f32>(-1.0, -1.0, 0.0); }
        case 4u: { g = vec3<f32>(1.0, 0.0, 1.0); }
        case 5u: { g = vec3<f32>(-1.0, 0.0, 1.0); }
        case 6u: { g = vec3<f32>(1.0, 0.0, -1.0); }
        case 7u: { g = vec3<f32>(-1.0, 0.0, -1.0); }
        case 8u: { g = vec3<f32>(0.0, 1.0, 1.0); }
        case 9u: { g = vec3<f32>(0.0, -1.0, 1.0); }
        case 10u: { g = vec3<f32>(0.0, 1.0, -1.0); }
        case 11u: { g = vec3<f32>(0.0, -1.0, -1.0); }
        case 12u: { g = vec3<f32>(1.0, 1.0, 0.0); }
        case 13u: { g = vec3<f32>(-1.0, 1.0, 0.0); }
        case 14u: { g = vec3<f32>(0.0, -1.0, 1.0); }
        default: {}
    }
    return g;
}

fn fh6_simplex2_corner(h: u32, d: vec2<f32>) -> f32 {
    let t = 0.5 - dot(d, d);
    if (t <= 0.0) {
        return 0.0;
    }
    let t2 = t * t;
    return t2 * t2 * fh6_grad2(h, d);
}

fn fh6_simplex2(p_in: vec2<f32>) -> f32 {
    let p = vec2<f32>(fh6_noise_coord(p_in.x), fh6_noise_coord(p_in.y));
    let f2 = 0.36602542;
    let g2 = 0.21132487;
    let s = (p.x + p.y) * f2;
    let fi = floor(p.x + s);
    let fj = floor(p.y + s);
    let i = fh6_cell(p.x + s);
    let j = fh6_cell(p.y + s);
    let t = (fi + fj) * g2;
    let d0 = p - vec2<f32>(fi - t, fj - t);
    let o1 = select(vec2<u32>(0u, 1u), vec2<u32>(1u, 0u), d0.x > d0.y);
    let d1 = d0 - vec2<f32>(o1) + vec2<f32>(g2);
    let d2 = d0 - vec2<f32>(1.0) + vec2<f32>(2.0 * g2);
    let n0 = fh6_simplex2_corner(fh6_hash2(i, j), d0);
    let n1 = fh6_simplex2_corner(fh6_hash2(i + o1.x, j + o1.y), d1);
    let n2 = fh6_simplex2_corner(fh6_hash2(i + 1u, j + 1u), d2);
    return clamp(40.0 * (n0 + n1 + n2), -1.0, 1.0);
}

// Accumulates one 3D simplex corner into `acc` (x = value, yzw = gradient).
fn fh6_simplex3_corner(h: u32, d: vec3<f32>) -> vec4<f32> {
    let t = 0.5 - dot(d, d);
    if (t <= 0.0) {
        return vec4<f32>(0.0);
    }
    let g = fh6_grad3(h);
    let gd = dot(g, d);
    let t2 = t * t;
    let t4 = t2 * t2;
    return vec4<f32>(t4 * gd, t4 * g - 8.0 * t2 * t * gd * d);
}

fn fh6_simplex3_grad(p_in: vec3<f32>) -> vec4<f32> {
    let p = vec3<f32>(fh6_noise_coord(p_in.x), fh6_noise_coord(p_in.y), fh6_noise_coord(p_in.z));
    let s = (p.x + p.y + p.z) * (1.0 / 3.0);
    let g3 = 1.0 / 6.0;
    let fc = floor(p + vec3<f32>(s));
    let i = fh6_cell(p.x + s);
    let j = fh6_cell(p.y + s);
    let k = fh6_cell(p.z + s);
    let t = (fc.x + fc.y + fc.z) * g3;
    let d0 = p - (fc - vec3<f32>(t));
    var o1: vec3<u32>;
    var o2: vec3<u32>;
    if (d0.x >= d0.y) {
        if (d0.y >= d0.z) {
            o1 = vec3<u32>(1u, 0u, 0u); o2 = vec3<u32>(1u, 1u, 0u);
        } else if (d0.x >= d0.z) {
            o1 = vec3<u32>(1u, 0u, 0u); o2 = vec3<u32>(1u, 0u, 1u);
        } else {
            o1 = vec3<u32>(0u, 0u, 1u); o2 = vec3<u32>(1u, 0u, 1u);
        }
    } else if (d0.y < d0.z) {
        o1 = vec3<u32>(0u, 0u, 1u); o2 = vec3<u32>(0u, 1u, 1u);
    } else if (d0.x < d0.z) {
        o1 = vec3<u32>(0u, 1u, 0u); o2 = vec3<u32>(0u, 1u, 1u);
    } else {
        o1 = vec3<u32>(0u, 1u, 0u); o2 = vec3<u32>(1u, 1u, 0u);
    }
    let d1 = d0 - vec3<f32>(o1) + vec3<f32>(g3);
    let d2 = d0 - vec3<f32>(o2) + vec3<f32>(2.0 * g3);
    let d3 = d0 - vec3<f32>(1.0) + vec3<f32>(3.0 * g3);
    var acc = fh6_simplex3_corner(fh6_hash3(i, j, k), d0);
    acc = acc + fh6_simplex3_corner(fh6_hash3(i + o1.x, j + o1.y, k + o1.z), d1);
    acc = acc + fh6_simplex3_corner(fh6_hash3(i + o2.x, j + o2.y, k + o2.z), d2);
    acc = acc + fh6_simplex3_corner(fh6_hash3(i + 1u, j + 1u, k + 1u), d3);
    acc = acc * 76.0;
    return vec4<f32>(clamp(acc.x, -1.0, 1.0), acc.yzw);
}

fn fh6_simplex3(p: vec3<f32>) -> f32 {
    return fh6_simplex3_grad(p).x;
}

fn fh6_curl2(p: vec2<f32>, t: f32) -> vec2<f32> {
    let g = fh6_simplex3_grad(vec3<f32>(p, t));
    return vec2<f32>(g.z, -g.y);
}
// ---- end fh6-kit noise ----
";

#[cfg(test)]
mod tests {
    use super::*;

    fn samples(n: u32) -> impl Iterator<Item = (f32, f32, f32)> {
        // A cheap deterministic scatter (Weyl sequence) over a wide range, negative included.
        (0..n).map(|i| {
            let f = i as f32;
            (
                ((f * 0.618_034) % 1.0) * 200.0 - 100.0,
                ((f * 0.754_877_7) % 1.0) * 200.0 - 100.0,
                ((f * 0.569_840_3) % 1.0) * 50.0 - 25.0,
            )
        })
    }

    #[test]
    fn simplex2_range_and_spread() {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for (x, y, _) in samples(200_000) {
            let v = simplex2(x, y);
            assert!(v.is_finite());
            lo = lo.min(v);
            hi = hi.max(v);
        }
        assert!((-1.0..=-0.6).contains(&lo), "min {lo} max {hi}");
        assert!((0.6..=1.0).contains(&hi), "max {hi}");
    }

    #[test]
    fn simplex3_range_and_spread() {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for (x, y, z) in samples(200_000) {
            let v = simplex3(x, y, z);
            assert!(v.is_finite());
            lo = lo.min(v);
            hi = hi.max(v);
        }
        assert!((-1.0..=-0.6).contains(&lo), "min {lo} max {hi}");
        assert!((0.6..=1.0).contains(&hi), "max {hi}");
    }

    #[test]
    fn deterministic_and_sane_inputs() {
        for (x, y, z) in samples(1000) {
            assert_eq!(simplex2(x, y).to_bits(), simplex2(x, y).to_bits());
            assert_eq!(simplex3(x, y, z).to_bits(), simplex3(x, y, z).to_bits());
            assert_eq!(curl2(x, y, z), curl2(x, y, z));
        }
        // Lattice points are zero; non-finite and huge inputs give finite output.
        assert_eq!(simplex2(0.0, 0.0), 0.0);
        assert!(simplex2(f32::NAN, f32::INFINITY).is_finite());
        assert!(simplex3(1e30, -1e30, f32::NEG_INFINITY).is_finite());
        assert!(curl2(f32::NAN, 3.0, 1e20).is_finite());
        // Not constant.
        assert_ne!(simplex2(0.3, 0.7), simplex2(1.3, 0.7));
    }

    #[test]
    fn fbm_range_and_octave_clamp() {
        for (x, y, _) in samples(20_000) {
            let v = fbm2(x, y, 5, 2.0, 0.5);
            assert!((-1.0..=1.0).contains(&v));
        }
        assert_eq!(fbm2(0.37, 1.9, 0, 2.0, 0.5), fbm2(0.37, 1.9, 1, 2.0, 0.5));
        assert_eq!(fbm2(0.37, 1.9, 99, 2.0, 0.5), fbm2(0.37, 1.9, MAX_OCTAVES, 2.0, 0.5));
        assert!(fbm2(0.37, 1.9, 4, f32::NAN, f32::NAN).is_finite());
    }

    #[test]
    fn analytic_gradient_matches_finite_differences() {
        let e = 1e-3;
        for (x, y, z) in samples(500).map(|(x, y, z)| (x * 0.05, y * 0.05, z * 0.1)) {
            let (_, g) = simplex3_grad(x, y, z);
            let dx = (simplex3(x + e, y, z) - simplex3(x - e, y, z)) / (2.0 * e);
            let dy = (simplex3(x, y + e, z) - simplex3(x, y - e, z)) / (2.0 * e);
            let dz = (simplex3(x, y, z + e) - simplex3(x, y, z - e)) / (2.0 * e);
            let tol = 0.02 * (1.0 + g[0].abs().max(g[1].abs()).max(g[2].abs()));
            assert!((g[0] - dx).abs() < tol, "dx {} vs {dx}", g[0]);
            assert!((g[1] - dy).abs() < tol, "dy {} vs {dy}", g[1]);
            assert!((g[2] - dz).abs() < tol, "dz {} vs {dz}", g[2]);
        }
    }

    #[test]
    fn curl_is_divergence_free() {
        let e = 1e-3;
        let mut max_mag = 0.0f32;
        for (x, y, t) in samples(2000).map(|(x, y, t)| (x * 0.05, y * 0.05, t * 0.1)) {
            let c = curl2(x, y, t);
            max_mag = max_mag.max(c.length());
            let dvx_dx = (curl2(x + e, y, t).x - curl2(x - e, y, t).x) / (2.0 * e);
            let dvy_dy = (curl2(x, y + e, t).y - curl2(x, y - e, t).y) / (2.0 * e);
            let div = dvx_dx + dvy_dy;
            assert!(div.abs() < 0.05, "divergence {div} at ({x}, {y}, {t})");
        }
        assert!(max_mag > 0.5, "curl field is degenerate: max |v| {max_mag}");
    }

    #[test]
    fn wgsl_source_declares_the_entry_points() {
        for name in [
            "fn fh6_simplex2(",
            "fn fh6_simplex3(",
            "fn fh6_simplex3_grad(",
            "fn fh6_curl2(",
        ] {
            assert!(WGSL_NOISE.contains(name), "{name}");
        }
    }
}
