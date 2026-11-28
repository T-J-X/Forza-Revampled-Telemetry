//! Computed effect textures: clean, seamless and grain-free by construction (TJX, 2026-11-28:
//! "We want very clean textures and animations"). Everything is built from periodic gradient
//! noise whose lattice wraps at the image size, so a band tiles exactly along its axis and a
//! flipbook loops exactly in time, and the finest octave is kept well above the pixel size, so
//! there is no speckle. A regular pattern (the perforated panel) is exact geometry.
use crate::img::{self, Img};
use std::f32::consts::TAU;

// ------------------------------------------------------------------------- periodic noise

/// A hash of three lattice coordinates (already wrapped) and a seed.
fn hash(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ (z as u32).wrapping_mul(0xcb1a_b31f)
        ^ seed.wrapping_mul(0x1656_67b1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^ (h >> 15)
}

/// A unit gradient for a lattice point.
fn grad(h: u32) -> [f32; 3] {
    // Spherical Fibonacci-like spread from the hash bits.
    let u = (h & 0xffff) as f32 / 65535.0;
    let v = (h >> 16) as f32 / 65535.0;
    let z = u * 2.0 - 1.0;
    let r = (1.0 - z * z).max(0.0).sqrt();
    let a = v * TAU;
    [r * a.cos(), r * a.sin(), z]
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// 3D gradient noise, periodic with integer periods `p` in each axis (lattice units), about
/// −1..1.
#[must_use]
pub fn noise3(x: f32, y: f32, z: f32, p: [i32; 3], seed: u32) -> f32 {
    let (xf, yf, zf) = (x.floor(), y.floor(), z.floor());
    let (xi, yi, zi) = (xf as i32, yf as i32, zf as i32);
    let (tx, ty, tz) = (x - xf, y - yf, z - zf);
    let w = |v: i32, n: i32| v.rem_euclid(n.max(1));
    let corner = |dx: i32, dy: i32, dz: i32| {
        let g = grad(hash(w(xi + dx, p[0]), w(yi + dy, p[1]), w(zi + dz, p[2]), seed));
        g[0] * (tx - dx as f32) + g[1] * (ty - dy as f32) + g[2] * (tz - dz as f32)
    };
    let (u, v, s) = (fade(tx), fade(ty), fade(tz));
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x00 = lerp(corner(0, 0, 0), corner(1, 0, 0), u);
    let x10 = lerp(corner(0, 1, 0), corner(1, 1, 0), u);
    let x01 = lerp(corner(0, 0, 1), corner(1, 0, 1), u);
    let x11 = lerp(corner(0, 1, 1), corner(1, 1, 1), u);
    lerp(lerp(x00, x10, v), lerp(x01, x11, v), s) * 1.15
}

/// Fractal sum of `oct` octaves of [`noise3`]; each octave doubles the frequency (and the
/// periods, so it still wraps) and halves the amplitude. Normalised to about −1..1.
#[must_use]
pub fn fbm(x: f32, y: f32, z: f32, p: [i32; 3], oct: u32, seed: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut f) = (0.0, 1.0, 0.0, 1.0);
    for o in 0..oct {
        let m = f as i32;
        sum += amp
            * noise3(
                x * f,
                y * f,
                z * f,
                [p[0] * m, p[1] * m, p[2] * m],
                seed.wrapping_add(o * 101),
            );
        norm += amp;
        amp *= 0.5;
        f *= 2.0;
    }
    sum / norm
}

fn smooth(e0: f32, e1: f32, x: f32) -> f32 {
    img::smoothstep(e0, e1, x)
}

/// Runs `f(x, y)` for every pixel on all cores.
fn par(w: usize, h: usize, c: usize, f: impl Fn(usize, usize) -> [f32; 4] + Sync) -> Img {
    let mut out = Img::new(w, h, c);
    let threads = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(h.max(1));
    let rows = h.div_ceil(threads);
    std::thread::scope(|s| {
        for (ti, chunk) in out.data.chunks_mut(rows * w * c).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (i, px) in chunk.chunks_mut(c).enumerate() {
                    let idx = ti * rows * w + i;
                    let v = f(idx % w, idx / w);
                    px.copy_from_slice(&v[..c]);
                }
            });
        }
    });
    out
}

// ------------------------------------------------------------------------------- the fire

/// Lattice cells along a band's long axis (the band repeats after this many).
const BAND_CELLS: i32 = 8;

/// Flame luminance at band coordinates `u` (0..1 along, wraps) and `v` (0..1 across), time
/// phase `t` (lattice units in the periodic time axis): a soft bright core along the middle,
/// bent by a slow warp, with wisps that thin out towards the edges.
fn flame(u: f32, v: f32, t: f32, tp: i32, aspect: f32, seed: u32) -> f32 {
    let cells = BAND_CELLS as f32;
    let (x, y) = (u * cells, v * cells / aspect);
    let py = (cells / aspect).round().max(1.0) as i32;
    let p = [BAND_CELLS, py.max(1), tp];
    // Slow warp: the core weaves gently.
    let warp = fbm(x * 0.5, 0.0, t, [BAND_CELLS / 2, 1, tp], 2, seed) * 0.08;
    let d = (v - 0.5 - warp).abs();
    // Domain-warped turbulence, three octaves above a big base: wisps, never speckle.
    let qx = fbm(x, y, t, p, 3, seed + 7);
    let qy = fbm(x + 3.1, y + 1.7, t, p, 3, seed + 13);
    let turb = fbm(x + qx * 0.9, y + qy * 0.9, t, p, 4, seed + 29) * 0.5 + 0.5;
    let core = (-(d / 0.07).powi(2)).exp();
    let body = (-(d / 0.17).powi(2)).exp();
    let wisps = smooth(0.42, 0.78, turb) * (-(d / 0.30).powi(2)).exp();
    (core * 0.95 + body * 0.55 * (0.55 + 0.45 * turb) + wisps * 0.45).clamp(0.0, 1.0)
}

/// The fire-jet band: `w` × `h`, seamless along x, luminance in RGB (grey), alpha 1.
#[must_use]
pub fn fire_band(w: usize, h: usize, seed: u32) -> Img {
    let aspect = w as f32 / h as f32;
    par(w, h, 3, |x, y| {
        let mut acc = 0.0;
        for (dx, dy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
            let u = (x as f32 + dx) / w as f32;
            let v = (y as f32 + dy) / h as f32;
            acc += flame(u, v, 0.0, 1, aspect, seed) * 0.25;
        }
        [acc, acc, acc, 1.0]
    })
}

/// The flow map for the fire band: the jet runs along +x, bent by the curl of a slow field so the
/// wisps curl as they scroll. RG = direction (0.5-centred, x right, y down), B = speed 0..1.
#[must_use]
pub fn fire_flow(w: usize, h: usize, seed: u32) -> Img {
    let aspect = w as f32 / h as f32;
    let cells = BAND_CELLS as f32;
    let py = ((cells / aspect).round() as i32).max(1);
    let p = [BAND_CELLS, py, 1];
    par(w, h, 3, |x, y| {
        let u = (x as f32 + 0.5) / w as f32 * cells;
        let v = (y as f32 + 0.5) / h as f32 * cells / aspect;
        let e = 0.01;
        let n = |a: f32, b: f32| fbm(a, b, 0.0, p, 2, seed + 41);
        // Curl of a scalar field: (∂n/∂y, −∂n/∂x).
        let cx = (n(u, v + e) - n(u, v - e)) / (2.0 * e);
        let cy = -(n(u + e, v) - n(u - e, v)) / (2.0 * e);
        let (fx, fy) = (1.0 + 0.12 * cx, 0.12 * cy);
        let l = (fx * fx + fy * fy).sqrt().max(1e-6);
        let across = ((y as f32 + 0.5) / h as f32 - 0.5).abs();
        let speed = (1.0 - across * 1.4).clamp(0.2, 1.0);
        [fx / l * 0.5 + 0.5, fy / l * 0.5 + 0.5, speed, 1.0]
    })
}

/// Frames in the flame-front loop.
pub const FRONT_FRAMES: usize = 16;

/// One frame of the burning front: a vertical band `w` × `h` whose right half is the burnt-through
/// side and whose left edge licks forward in tongues, rising as the loop runs. Luminance in RGB,
/// alpha = coverage. Frame `i` of [`FRONT_FRAMES`] loops exactly (periodic time lattice, and the
/// rise is a whole lattice period per loop).
#[must_use]
pub fn flame_front(w: usize, h: usize, i: usize, seed: u32) -> Img {
    let t = i as f32 / FRONT_FRAMES as f32;
    let cells_y = 4.0_f32;
    let cells_x = cells_y * w as f32 / h as f32;
    let tp = 2; // the time lattice wraps after 2 cells per loop
    let p = [((cells_x.round()) as i32).max(1), cells_y as i32, tp];
    par(w, h, 4, |x, y| {
        let mut lum = 0.0;
        let mut a = 0.0;
        for (dx, dy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
            let u = (x as f32 + dx) / w as f32;
            // Rising: one whole lattice period in y per loop.
            let v = ((y as f32 + dy) / h as f32 + t).rem_euclid(1.0);
            let (nx, ny, nz) = (u * cells_x, v * cells_y, t * tp as f32);
            let q = fbm(nx, ny, nz, p, 3, seed + 3);
            let tongue = fbm(nx * 0.5 + q * 0.6, ny + q * 0.6, nz, p, 4, seed + 17);
            // Front position across the band (0 left .. 1 right), pushed left by the tongues.
            let front = 0.55 - 0.28 * (tongue * 0.5 + 0.5);
            let dist = u - front;
            let edge = (-(dist / 0.045).powi(2)).exp();
            let behind = smooth(-0.02, 0.18, dist) * (0.55 + 0.25 * q);
            let cover = smooth(-0.06, 0.02, dist);
            lum += ((edge + behind).clamp(0.0, 1.0)) * 0.25;
            a += cover * 0.25;
        }
        [lum, lum, lum, a]
    })
}

/// The horizon haze: a very soft, seamless-along-x band of fog and faint cloud, luminance in
/// RGB, fading out smoothly at the top and bottom edges.
#[must_use]
pub fn haze_band(w: usize, h: usize, seed: u32) -> Img {
    let aspect = w as f32 / h as f32;
    let cells = 6.0_f32;
    let py = ((cells / aspect).round() as i32).max(1);
    let p = [cells as i32, py, 1];
    par(w, h, 3, |x, y| {
        let u = (x as f32 + 0.5) / w as f32;
        let v = (y as f32 + 0.5) / h as f32;
        let (nx, ny) = (u * cells, v * cells / aspect);
        let q = fbm(nx, ny, 0.0, p, 2, seed + 5);
        let n = fbm(nx + q * 0.5, ny * 1.6 + q * 0.5, 0.0, p, 4, seed + 11) * 0.5 + 0.5;
        // Fog densest just below the horizon line at 60% height, clouds faint above it.
        let fog = (-((v - 0.62) / 0.16).powi(2)).exp();
        let cloud = smooth(0.5, 0.85, n) * (-((v - 0.35) / 0.18).powi(2)).exp() * 0.35;
        let env = smooth(0.0, 0.12, v) * smooth(0.0, 0.12, 1.0 - v);
        let l = ((fog * (0.6 + 0.4 * n) + cloud) * env).clamp(0.0, 1.0);
        [l, l, l, 1.0]
    })
}

// --------------------------------------------------------------------- perforated panel

/// A perforated panel tile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Perf {
    /// Hole pitch (px at the master size); the master side must be a whole number of pitches.
    pub pitch: f32,
    /// Hole diameter (px).
    pub hole: f32,
    /// Chamfer width round each hole (px).
    pub chamfer: f32,
    /// Linear albedo of the panel.
    pub albedo: [f32; 3],
    /// Roughness of the panel face.
    pub rough: f32,
}

/// Distance from the pixel centre to the nearest hole centre on a square grid of `pitch`.
fn hole_dist(x: f32, y: f32, pitch: f32) -> (f32, f32, f32) {
    let gx = x - ((x / pitch).floor() + 0.5) * pitch;
    let gy = y - ((y / pitch).floor() + 0.5) * pitch;
    ((gx * gx + gy * gy).sqrt(), gx, gy)
}

/// The panel's height at a point: 1 on the face, falling over the chamfer into the hole (0).
fn perf_height(p: &Perf, x: f32, y: f32) -> f32 {
    let (d, _, _) = hole_dist(x, y, p.pitch);
    let r = p.hole * 0.5;
    smooth(r, r + p.chamfer, d)
}

/// The perforated panel: (albedo sRGB-ready linear, normal, roughness), each `n` px square,
/// 4x4 supersampled. Seamless when `n` is a whole number of pitches.
#[must_use]
pub fn perforated(p: &Perf, n: usize) -> (Img, Img, Img) {
    let ss = 4;
    let albedo = par(n, n, 3, |x, y| {
        let mut face = 0.0;
        for sy in 0..ss {
            for sx in 0..ss {
                let px = x as f32 + (sx as f32 + 0.5) / ss as f32;
                let py = y as f32 + (sy as f32 + 0.5) / ss as f32;
                face += perf_height(p, px, py) / (ss * ss) as f32;
            }
        }
        // Holes read as near-black voids (the dark behind the panel).
        let k = 0.08 + 0.92 * face;
        [p.albedo[0] * k, p.albedo[1] * k, p.albedo[2] * k, 1.0]
    });
    let normal = par(n, n, 3, |x, y| {
        let (mut ax, mut ay) = (0.0, 0.0);
        let e = 0.25;
        for sy in 0..ss {
            for sx in 0..ss {
                let px = x as f32 + (sx as f32 + 0.5) / ss as f32;
                let py = y as f32 + (sy as f32 + 0.5) / ss as f32;
                // Height in px: the chamfer drops `chamfer` px over `chamfer` px (45°).
                let hx = (perf_height(p, px + e, py) - perf_height(p, px - e, py)) / (2.0 * e) * p.chamfer;
                let hy = (perf_height(p, px, py + e) - perf_height(p, px, py - e)) / (2.0 * e) * p.chamfer;
                ax += hx / (ss * ss) as f32;
                ay += hy / (ss * ss) as f32;
            }
        }
        // Tangent space, green up (image y grows downwards), as `img::normal_from_height`.
        let (nx, ny) = (-ax, ay);
        let l = (nx * nx + ny * ny + 1.0).sqrt();
        [nx / l * 0.5 + 0.5, ny / l * 0.5 + 0.5, 1.0 / l * 0.5 + 0.5, 1.0]
    });
    let rough = par(n, n, 3, |x, y| {
        let h = perf_height(p, x as f32 + 0.5, y as f32 + 0.5);
        let r = p.rough + (0.6 - p.rough) * (1.0 - h);
        [r, r, r, 1.0]
    });
    (albedo, normal, rough)
}

// ---------------------------------------------------------------------------------- rain

/// Rain streaks in the atlas.
pub const STREAKS: usize = 12;

/// Rain streaks: [`STREAKS`] motion-blurred drops side by side, each in a `cw` × `ch` cell (alpha
/// on black, white RGB): 1 to 3 px wide and 64 to 256 px long at 4K (`scale` px per 4K px),
/// a soft Gaussian across, a tapered head and tail along.
#[must_use]
pub fn rain_streaks(scale: f32, seed: u32) -> Img {
    let (cw, ch) = ((16.0 * scale) as usize, (256.0 * scale) as usize);
    let mut rng = crate::procedural::Rng::new(u64::from(seed));
    let specs: Vec<(f32, f32, f32)> = (0..STREAKS)
        .map(|i| {
            let w = 1.0 + 2.0 * (i as f32 / (STREAKS - 1) as f32);
            let len = 64.0 + 192.0 * rng.f32();
            (w * scale, len * scale, 0.55 + 0.45 * rng.f32())
        })
        .collect();
    let mut out = Img::new(cw * STREAKS, ch, 4);
    for (i, (w, len, peak)) in specs.iter().enumerate() {
        let cx = cw as f32 * 0.5;
        let top = (ch as f32 - len) * 0.5;
        for y in 0..ch {
            for x in 0..cw {
                let mut a = 0.0;
                for (dx, dy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                    let (px, py) = (x as f32 + dx, y as f32 + dy);
                    let t = (py - top) / len;
                    if !(0.0..=1.0).contains(&t) {
                        continue;
                    }
                    // Brightest near the head (bottom), fading up the tail; soft ends.
                    let along = smooth(0.0, 0.25, t) * (1.0 - smooth(0.9, 1.0, t)) * (0.35 + 0.65 * t);
                    let sigma = w * 0.5;
                    let across = (-((px - cx) / sigma).powi(2) * 0.5).exp();
                    a += along * across * peak * 0.25;
                }
                let ox = i * cw + x;
                out.set(ox, y, 0, 1.0);
                out.set(ox, y, 1, 1.0);
                out.set(ox, y, 2, 1.0);
                out.set(ox, y, 3, a.clamp(0.0, 1.0));
            }
        }
    }
    out
}

/// Beads in the bead sheet (4 x 4).
pub const BEADS: usize = 16;

/// Water beads: [`BEADS`] drops of 8 to 24 px (4K px; `scale` px per 4K px) in a 4 x 4 sheet of
/// 32 px cells, each a refracting dome: a dark refraction rim where Fresnel turns the view to
/// grazing (water, n = 1.333), a sharp specular highlight towards the upper-left key, and a soft
/// caustic crescent opposite. RGB = light (linear), alpha = coverage and body.
#[must_use]
pub fn water_beads(scale: f32, seed: u32) -> Img {
    let cell = (32.0 * scale) as usize;
    let mut rng = crate::procedural::Rng::new(u64::from(seed));
    let mut out = Img::new(cell * 4, cell * 4, 4);
    let key = {
        let v = [-0.45_f32, -0.55, 0.70];
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / l, v[1] / l, v[2] / l]
    };
    for b in 0..BEADS {
        let d = (8.0 + 16.0 * (b as f32 / (BEADS - 1) as f32)) * scale;
        let r = d * 0.5;
        let squash = 0.9 + 0.1 * rng.f32();
        let (bx, by) = ((b % 4) * cell, (b / 4) * cell);
        let c = cell as f32 * 0.5;
        for y in 0..cell {
            for x in 0..cell {
                let (mut lum, mut a) = (0.0, 0.0);
                for sy in 0..4 {
                    for sx in 0..4 {
                        let u = (x as f32 + (sx as f32 + 0.5) / 4.0 - c) / r;
                        let v = (y as f32 + (sy as f32 + 0.5) / 4.0 - c) / (r * squash);
                        let rr = u * u + v * v;
                        if rr >= 1.0 {
                            continue;
                        }
                        let z = (1.0 - rr).sqrt() * 0.6; // a flattened dome
                        let ln = (u * u + v * v + z * z).sqrt();
                        let n = [u / ln, v / ln, z / ln];
                        let cos = n[2];
                        // Fresnel of water at the view angle.
                        let f = 0.02 + 0.98 * (1.0 - cos).powi(5);
                        // Mirror of the key: r = 2 (n.v) n - v, v = (0, 0, 1).
                        let rv = [2.0 * cos * n[0], 2.0 * cos * n[1], 2.0 * cos * n[2] - 1.0];
                        let spec = (rv[0] * key[0] + rv[1] * key[1] + rv[2] * key[2]).max(0.0).powf(90.0);
                        let caustic = (-((u - 0.35).powi(2) + (v - 0.4).powi(2)) / 0.08).exp() * 0.25;
                        lum += (spec * 1.6 + caustic + f * 0.5) / 16.0;
                        a += (0.18 + 0.6 * f + spec).min(1.0) / 16.0;
                    }
                }
                let (ox, oy) = (bx + x, by + y);
                let l = if a > 0.0 { (lum / a).min(1.0) } else { 0.0 };
                out.set(ox, oy, 0, l);
                out.set(ox, oy, 1, l);
                out.set(ox, oy, 2, l);
                out.set(ox, oy, 3, a.clamp(0.0, 1.0));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------- fuse sparks

/// The burning fuse line: `w` × `h`, seamless along x. A jagged bright core (its centre line
/// jitters by up to 5% of the height on a periodic noise, its brightness flickers along x) inside
/// a soft glow, with brighter ember knots. Luminance in RGB, alpha = coverage.
#[must_use]
pub fn fuse_line(w: usize, h: usize, seed: u32) -> Img {
    let cells = 64; // jitter cells along the strip
    par(w, h, 4, |x, y| {
        let (mut lum, mut a) = (0.0, 0.0);
        for (dx, dy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
            let u = (x as f32 + dx) / w as f32;
            let v = (y as f32 + dy) / h as f32;
            let jag = fbm(u * cells as f32, 0.5, 0.0, [cells, 1, 1], 3, seed) * 0.05;
            let d = (v - 0.5 - jag).abs() * h as f32; // px from the core line
            let flick = fbm(u * 16.0, 1.5, 0.0, [16, 1, 1], 3, seed + 5) * 0.5 + 0.5;
            let knot = smooth(0.62, 0.9, fbm(u * 24.0, 2.5, 0.0, [24, 1, 1], 2, seed + 9) * 0.5 + 0.5);
            let core = (-(d / (1.6 + 1.2 * knot)).powi(2)).exp() * (0.7 + 0.3 * flick);
            let glow = (-(d / (9.0 + 8.0 * knot)).powi(2)).exp() * (0.25 + 0.35 * flick + 0.3 * knot);
            let l = (core + glow).min(1.0);
            lum += l * 0.25;
            a += (core + glow * 0.9).min(1.0) * 0.25;
        }
        let k = if a > 0.0 { (lum / a).min(1.0) } else { 0.0 };
        [k, k, k, a]
    })
}

/// Spark sprites in the sheet (8 x 2).
pub const SPARKS: usize = 16;

/// Sparks spitting off the line: [`SPARKS`] sprites of `cell` px, each 1 to 4 ballistic streaks
/// thrown up and out from a point near the bottom centre (gravity bends them back), thin Gaussian
/// trails that taper towards the tail, and a hot head. Luminance in RGB, alpha = coverage.
#[must_use]
pub fn sparks(cell: usize, seed: u32) -> Img {
    let mut rng = crate::procedural::Rng::new(u64::from(seed));
    let mut out = Img::new(cell * 8, cell * 2, 4);
    let s = cell as f32 / 64.0;
    for i in 0..SPARKS {
        let (ox, oy) = ((i % 8) * cell, (i / 8) * cell);
        let n = 1 + (rng.f32() * 4.0) as usize;
        let mut acc = vec![0.0_f32; cell * cell];
        for _ in 0..n {
            let ang = (-90.0 + (rng.f32() - 0.5) * 110.0).to_radians();
            let speed = (40.0 + 30.0 * rng.f32()) * s;
            let g = 60.0 * s;
            let len_t = 0.45 + 0.4 * rng.f32();
            let (sx, sy) = (cell as f32 * 0.5, cell as f32 * 0.85);
            let steps = 160;
            for k in 0..steps {
                let t = len_t * k as f32 / (steps - 1) as f32;
                let px = sx + speed * ang.cos() * t;
                let py = sy + speed * ang.sin() * t + 0.5 * g * t * t;
                let f = k as f32 / (steps - 1) as f32; // 0 tail .. 1 head
                let bright = 0.15 + 0.85 * f * f;
                let sigma = (0.6 + 0.6 * f) * s;
                let r = (sigma * 3.0).ceil() as isize;
                for yy in -r..=r {
                    for xx in -r..=r {
                        let (qx, qy) = (px.floor() as isize + xx, py.floor() as isize + yy);
                        if qx < 1 || qy < 1 || qx >= cell as isize - 1 || qy >= cell as isize - 1 {
                            continue;
                        }
                        let d2 = ((qx as f32 + 0.5) - px).powi(2) + ((qy as f32 + 0.5) - py).powi(2);
                        let wgt = (-d2 / (2.0 * sigma * sigma)).exp() * bright / steps as f32 * 18.0;
                        let idx = qy as usize * cell + qx as usize;
                        acc[idx] = (acc[idx] + wgt).min(1.0);
                    }
                }
            }
        }
        for y in 0..cell {
            for x in 0..cell {
                let a = acc[y * cell + x];
                out.set(ox + x, oy + y, 0, 1.0);
                out.set(ox + x, oy + y, 1, 1.0);
                out.set(ox + x, oy + y, 2, 1.0);
                out.set(ox + x, oy + y, 3, a);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_wraps_at_its_period() {
        let p = [5, 3, 2];
        for &(x, y, z) in &[(0.3_f32, 0.7_f32, 0.1_f32), (2.2, 1.9, 1.4), (4.9, 0.05, 0.5)] {
            let a = fbm(x, y, z, p, 4, 9);
            assert!((a - fbm(x + 5.0, y, z, p, 4, 9)).abs() < 1e-5);
            assert!((a - fbm(x, y + 3.0, z, p, 4, 9)).abs() < 1e-5);
            assert!((a - fbm(x, y, z + 2.0, p, 4, 9)).abs() < 1e-5);
        }
    }

    #[test]
    fn the_bands_tile_along_their_length() {
        for im in [fire_band(256, 64, 1), haze_band(256, 64, 2)] {
            let w = im.w;
            let mut worst: f32 = 0.0;
            let mut inner: f32 = 0.0;
            for y in 0..im.h {
                worst = worst.max((im.get(0, y, 0) - im.get(w - 1, y, 0)).abs());
                inner = inner.max((im.get(w / 2, y, 0) - im.get(w / 2 - 1, y, 0)).abs());
            }
            assert!(worst <= inner * 1.5 + 0.02, "seam {worst} vs neighbour step {inner}");
        }
    }

    #[test]
    fn the_flame_front_loops() {
        let a = flame_front(32, 128, 0, 3);
        let b = flame_front(32, 128, FRONT_FRAMES, 3);
        let d = a
            .data
            .iter()
            .zip(&b.data)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f32::max);
        assert!(d < 1e-4, "frame 16 differs from frame 0 by {d}");
    }

    #[test]
    fn the_fire_has_no_speckle() {
        // Clean: neighbouring pixels never jump by more than a few percent.
        let im = fire_band(512, 128, 1);
        let mut worst: f32 = 0.0;
        for y in 0..im.h {
            for x in 1..im.w {
                worst = worst.max((im.get(x, y, 0) - im.get(x - 1, y, 0)).abs());
            }
        }
        assert!(worst < 0.08, "step {worst}");
    }

    #[test]
    fn rain_and_beads_stay_inside_their_cells() {
        let r = rain_streaks(1.0, 5);
        for i in 0..STREAKS {
            for y in 0..r.h {
                assert!(r.get(i * 16, y, 3) < 0.01 && r.get(i * 16 + 15, y, 3) < 0.05);
            }
        }
        let b = water_beads(1.0, 5);
        for y in 0..b.h {
            assert!(b.get(0, y, 3) == 0.0);
        }
        assert!(b.data.iter().all(|v| (0.0..=1.0).contains(v)));
    }

    #[test]
    fn the_fuse_line_tiles_and_sparks_stay_in_their_cells() {
        let f = fuse_line(512, 64, 4);
        let mut seam: f32 = 0.0;
        let mut inner: f32 = 0.0;
        for y in 0..f.h {
            seam = seam.max((f.get(0, y, 3) - f.get(511, y, 3)).abs());
            for x in 1..512 {
                inner = inner.max((f.get(x, y, 3) - f.get(x - 1, y, 3)).abs());
            }
        }
        assert!(seam <= inner * 1.6 + 0.02, "{seam} {inner}");
        let sp = sparks(64, 4);
        for y in 0..sp.h {
            for k in 0..8 {
                assert!(sp.get(k * 64, y, 3) == 0.0);
            }
        }
    }

    #[test]
    fn the_perforation_tiles_and_its_normals_are_unit() {
        let p = Perf {
            pitch: 32.0,
            hole: 12.0,
            chamfer: 3.0,
            albedo: [0.01, 0.02, 0.06],
            rough: 0.18,
        };
        let (a, n, _) = perforated(&p, 128);
        for y in 0..128 {
            assert!((a.get(0, y, 2) - a.get(127, y, 2)).abs() < 1e-6 || y % 32 != 0);
        }
        for px in n.data.chunks(3) {
            let v: Vec<f32> = px.iter().map(|c| c * 2.0 - 1.0).collect();
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            assert!((l - 1.0).abs() < 1e-3);
        }
    }
}
