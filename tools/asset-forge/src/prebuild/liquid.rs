//! Computed assets for the liquid-metal dashes (`KineticLiquidMetal`, the oil-slick chrome blob since 08:50, and `FaradayMandala`): a studio
//! environment, metal matcaps, Fresnel and thin-film look-up tables. Each one is exact maths, so
//! it is made here on the CPU instead of being rendered or generated: a mirror of emissive panels
//! on black is the panels' own radiance, and a measured metal's reflectance is its Fresnel sum,
//! so Cycles or SDXL would only add noise.
//!
//! - **Studio** ([`studio`]): the "liquid-metal studio" in the kit frame of `docs/specs/optics.md`
//!   §7 (z towards the viewer, y down, azimuth clockwise from 3 o'clock, elevation above the
//!   screen). A softbox arc and a key strip upper left, a cool stripe low on the right, a horizon
//!   ring, and behind the screen a dim sky above a darker floor with a sharp horizon between, which
//!   draws the chrome horizon line on every silhouette. Nothing within 45° of the viewer, so a face
//!   tilted less than 22° mirrors black (OLED: only highlights exist).
//! - **Metals** ([`Metal`]): spectral optical constants from the refractiveindex.info database
//!   (CC0): chromium, Johnson and Christy, Phys. Rev. B 9, 5056 (1974); liquid mercury, Inagaki,
//!   Arakawa and Williams, Phys. Rev. B 23, 5246 (1981).
//! - **Fresnel** ([`conductor_reflectance`]): exact unpolarised reflectance of a complex index,
//!   integrated over wavelength with the CIE 1931 fit of `fh6_kit::optics::cie_xyz` and balanced so
//!   a flat spectrum is white (`spectrum_to_linear_srgb`, copied here because asset-forge does not
//!   link the kit).
//! - **Thin film** ([`film_reflectance`]): the Airy sum of `optics.md` §9 with the metal as a
//!   complex-index substrate (CATALOG.md, the oil-slick maths row).
use crate::img::{self, Img};
use std::f32::consts::PI;

/// Radiance that encodes as white in the environment and the matcaps (the Liquid Glass
/// convention, `recipes/liquid_glass.rs` `ENV_RANGE`).
pub const ENV_RANGE: f32 = 16.0;

// --------------------------------------------------------------------------------- complex

#[derive(Debug, Clone, Copy, PartialEq)]
struct C {
    re: f32,
    im: f32,
}

impl C {
    const fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }
    const fn real(re: f32) -> Self {
        Self { re, im: 0.0 }
    }
    fn add(self, o: Self) -> Self {
        Self::new(self.re + o.re, self.im + o.im)
    }
    fn sub(self, o: Self) -> Self {
        Self::new(self.re - o.re, self.im - o.im)
    }
    fn mul(self, o: Self) -> Self {
        Self::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
    fn div(self, o: Self) -> Self {
        let d = o.re * o.re + o.im * o.im;
        Self::new(
            (self.re * o.re + self.im * o.im) / d,
            (self.im * o.re - self.re * o.im) / d,
        )
    }
    fn norm2(self) -> f32 {
        self.re * self.re + self.im * self.im
    }
    /// Principal square root (real part ≥ 0).
    fn sqrt(self) -> Self {
        let r = self.norm2().sqrt();
        let re = ((r + self.re) * 0.5).max(0.0).sqrt();
        let im = ((r - self.re) * 0.5).max(0.0).sqrt().copysign(self.im);
        Self::new(re, im)
    }
    fn cis(phase: f32) -> Self {
        Self::new(phase.cos(), phase.sin())
    }
}

// ---------------------------------------------------------------------------------- metals

/// A material with a spectral complex index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metal {
    /// Polished chromium (Johnson and Christy 1974).
    Chrome,
    /// Liquid mercury (Inagaki et al. 1981), the measured stand-in for a liquid metal.
    Mercury,
}

/// (λ µm, n, k), Johnson and Christy 1974, 368..756 nm.
#[allow(clippy::approx_constant)] // 3.140 is a measured k, not π
const CR: &[(f32, f32, f32)] = &[
    (0.3680, 1.870, 2.690),
    (0.3810, 1.920, 2.740),
    (0.3970, 2.000, 2.830),
    (0.4130, 2.080, 2.930),
    (0.4310, 2.190, 3.040),
    (0.4510, 2.330, 3.140),
    (0.4710, 2.510, 3.240),
    (0.4960, 2.750, 3.300),
    (0.5210, 2.940, 3.330),
    (0.5490, 3.180, 3.330),
    (0.5820, 3.220, 3.300),
    (0.6170, 3.170, 3.300),
    (0.6590, 3.090, 3.340),
    (0.7040, 3.050, 3.390),
    (0.7560, 3.080, 3.420),
];

/// (λ µm, n, k), Inagaki, Arakawa and Williams 1981, liquid mercury at room temperature.
const HG: &[(f32, f32, f32)] = &[
    (0.3647, 0.713, 3.074),
    (0.3875, 0.798, 3.294),
    (0.4133, 0.898, 3.538),
    (0.4428, 1.027, 3.802),
    (0.4769, 1.186, 4.090),
    (0.5166, 1.383, 4.406),
    (0.5636, 1.621, 4.750),
    (0.6199, 1.910, 5.150),
    (0.6888, 2.284, 5.582),
    (0.7749, 2.746, 6.054),
];

impl Metal {
    /// The source of the constants.
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            Self::Chrome => "Cr, Johnson & Christy, Phys. Rev. B 9, 5056 (1974), via refractiveindex.info (CC0)",
            Self::Mercury => {
                "liquid Hg, Inagaki, Arakawa & Williams, Phys. Rev. B 23, 5246 (1981), via refractiveindex.info (CC0)"
            }
        }
    }

    /// Complex index n + ik at `nm`, linear between table rows, clamped at the ends.
    fn index(self, nm: f32) -> C {
        let t = match self {
            Self::Chrome => CR,
            Self::Mercury => HG,
        };
        let um = nm / 1000.0;
        if um <= t[0].0 {
            return C::new(t[0].1, t[0].2);
        }
        for w in t.windows(2) {
            let (a, b) = (w[0], w[1]);
            if um <= b.0 {
                let f = (um - a.0) / (b.0 - a.0);
                return C::new(a.1 + (b.1 - a.1) * f, a.2 + (b.2 - a.2) * f);
            }
        }
        let l = t[t.len() - 1];
        C::new(l.1, l.2)
    }
}

// ----------------------------------------------------------------------------------- colour

/// CIE 1931 2° matching functions, the multi-lobe fit of Wyman, Sloan and Shirley (2013), as
/// `fh6_kit::optics::cie_xyz`.
fn cie_xyz(nm: f32) -> [f32; 3] {
    let lobe = |mu: f32, below: f32, above: f32| {
        let t = (nm - mu) * if nm < mu { below } else { above };
        (-0.5 * t * t).exp()
    };
    [
        0.362 * lobe(442.0, 0.0624, 0.0374) + 1.056 * lobe(599.8, 0.0264, 0.0323) - 0.065 * lobe(501.1, 0.0490, 0.0382),
        0.821 * lobe(568.8, 0.0213, 0.0247) + 0.286 * lobe(530.9, 0.0613, 0.0322),
        1.217 * lobe(437.0, 0.0845, 0.0278) + 0.681 * lobe(459.0, 0.0385, 0.0725),
    ]
}

/// The linear sRGB colour of a reflectance spectrum under white light, balanced so a flat
/// spectrum of 1 is (1, 1, 1): `fh6_kit::optics::spectrum_to_linear_srgb`.
fn spectrum_rgb(f: impl Fn(f32) -> f32) -> [f32; 3] {
    const D65: [f32; 3] = [0.9505, 1.0, 1.0890];
    let mut sum = [0.0_f32; 3];
    let mut white = [0.0_f32; 3];
    let mut nm = 380.0_f32;
    while nm <= 780.0 {
        let cmf = cie_xyz(nm);
        let v = f(nm);
        for c in 0..3 {
            sum[c] += cmf[c] * v;
            white[c] += cmf[c];
        }
        nm += 5.0;
    }
    let [x, y, z] = [
        sum[0] / white[0] * D65[0],
        sum[1] / white[1] * D65[1],
        sum[2] / white[2] * D65[2],
    ];
    [
        3.2406 * x - 1.5372 * y - 0.4986 * z,
        -0.9689 * x + 1.8758 * y + 0.0415 * z,
        0.0557 * x - 0.2040 * y + 1.0570 * z,
    ]
}

// ---------------------------------------------------------------------------------- fresnel

/// Cosine of the transmitted angle in a medium of complex index `n` for light arriving from
/// air at `cos0`.
fn cos_in(n: C, cos0: f32) -> C {
    let s2 = C::real(1.0 - cos0 * cos0);
    C::real(1.0).sub(s2.div(n.mul(n))).sqrt()
}

/// s and p amplitude coefficients of the interface a → b (indices and cosines).
fn amp(na: C, ca: C, nb: C, cb: C) -> (C, C) {
    let s = na.mul(ca).sub(nb.mul(cb)).div(na.mul(ca).add(nb.mul(cb)));
    let p = nb.mul(ca).sub(na.mul(cb)).div(nb.mul(ca).add(na.mul(cb)));
    (s, p)
}

/// Unpolarised reflectance of bare `metal` from air at `cos0` and wavelength `nm`.
#[must_use]
pub fn conductor_reflectance(metal: Metal, nm: f32, cos0: f32) -> f32 {
    let cos0 = cos0.clamp(1e-4, 1.0);
    let n = metal.index(nm);
    let (s, p) = amp(C::real(1.0), C::real(cos0), n, cos_in(n, cos0));
    0.5 * (s.norm2() + p.norm2())
}

/// Unpolarised reflectance of a lossless film (index `film`, `thick_nm`) on `metal`, from air at
/// `cos0`: the Airy sum r = (r01 + r12 e^{2iβ}) ÷ (1 + r01 r12 e^{2iβ}), β = 2π n₁ d cos θ₁ ÷ λ,
/// per polarisation, |r|² averaged.
#[must_use]
pub fn film_reflectance(film: f32, metal: Metal, thick_nm: f32, nm: f32, cos0: f32) -> f32 {
    film_on(film, metal.index(nm), thick_nm, nm, cos0)
}

fn film_on(film: f32, sub: C, thick_nm: f32, nm: f32, cos0: f32) -> f32 {
    let cos0 = cos0.clamp(1e-4, 1.0);
    let (n0, n1) = (C::real(1.0), C::real(film));
    let (c0, c1, c2) = (C::real(cos0), cos_in(n1, cos0), cos_in(sub, cos0));
    let (s01, p01) = amp(n0, c0, n1, c1);
    let (s12, p12) = amp(n1, c1, sub, c2);
    let phase = C::cis(4.0 * PI * film * thick_nm.max(0.0) * c1.re / nm);
    let airy = |a: C, b: C| {
        let bp = b.mul(phase);
        a.add(bp).div(C::real(1.0).add(a.mul(bp))).norm2()
    };
    0.5 * (airy(s01, s12) + airy(p01, p12))
}

/// Linear sRGB reflectance of bare `metal` at `cos0` under white light.
#[must_use]
pub fn metal_rgb(metal: Metal, cos0: f32) -> [f32; 3] {
    spectrum_rgb(|nm| conductor_reflectance(metal, nm, cos0))
}

// ----------------------------------------------------------------------------------- studio

fn smooth(e0: f32, e1: f32, x: f32) -> f32 {
    img::smoothstep(e0, e1, x)
}

/// A soft box in (azimuth, elevation), degrees, with edges feathered over `f`°.
fn boxed(az: f32, el: f32, az0: f32, az1: f32, el0: f32, el1: f32, f: f32) -> f32 {
    let wrap = |a: f32| (a + 540.0).rem_euclid(360.0) - 180.0;
    let mid = (az0 + az1) * 0.5;
    let half = (az1 - az0) * 0.5;
    let da = wrap(az - mid).abs();
    let a = 1.0 - smooth(half - f, half, da);
    let e = smooth(el0 - f, el0, el) * (1.0 - smooth(el1, el1 + f, el));
    a * e
}

/// Linear radiance of the liquid-metal studio along the unit kit-frame direction `d`.
#[must_use]
pub fn studio(d: [f32; 3]) -> [f32; 3] {
    let el = d[2].clamp(-1.0, 1.0).asin().to_degrees();
    let az = d[1].atan2(d[0]).to_degrees();
    let mut c = [0.0_f32; 3];
    let mut add = |w: f32, rgb: [f32; 3]| {
        for i in 0..3 {
            c[i] += w * rgb[i];
        }
    };
    // In front of the screen: lights only, nothing within 45° of the viewer.
    if el > -2.0 {
        let open = 1.0 - smooth(44.0, 45.0, el);
        // Key strip, upper left: 80° along the azimuth, 8° tall.
        add(boxed(az, el, -175.0, -95.0, 36.0, 44.0, 0.6) * open, [10.0, 10.0, 10.0]);
        // Softbox arc under it, 140° of azimuth, falling off over its last 25° each end.
        let arc_end = {
            let wrap = |a: f32| (a + 540.0).rem_euclid(360.0) - 180.0;
            let da = wrap(az + 105.0).abs();
            1.0 - smooth(45.0, 70.0, da)
        };
        add(
            boxed(az, el, -180.0, -30.0, 18.0, 30.0, 0.8) * arc_end * open,
            [5.2, 5.4, 5.6],
        );
        // Cool stripe low on the right (lower right on screen).
        add(boxed(az, el, 15.0, 75.0, 3.0, 9.0, 0.5), [0.8, 2.0, 3.4]);
        // Horizon ring all round.
        add(smooth(-1.0, 0.0, el) * (1.0 - smooth(4.0, 5.0, el)), [0.5, 0.5, 0.5]);
    }
    // Behind the screen: a dim sky above (screen up, y < 0) over a dark floor, split sharply.
    if el < 0.0 {
        let behind = smooth(0.0, -1.0, el);
        let up = (-d[1]).clamp(-1.0, 1.0).asin().to_degrees();
        let sky = smooth(-0.5, 0.5, up);
        let sky_rad = 0.06 + 0.12 * (up.max(0.0) / 90.0).sqrt();
        let floor_rad = 0.012;
        let v = sky * sky_rad + (1.0 - sky) * floor_rad;
        add(behind, [v * 0.95, v * 0.98, v]);
    }
    c
}

/// The direction of octahedral coordinates (u, v) in 0..1: the layout `shape.wgsl` `oct_uv`
/// reads (as `recipes/liquid_glass.rs` `oct_dir`).
#[must_use]
pub fn oct_dir(u: f32, v: f32) -> [f32; 3] {
    let (ox, oy) = (u * 2.0 - 1.0, v * 2.0 - 1.0);
    let z = 1.0 - ox.abs() - oy.abs();
    let (x, y) = if z >= 0.0 {
        (ox, oy)
    } else {
        ((1.0 - oy.abs()).copysign(ox), (1.0 - ox.abs()).copysign(oy))
    };
    let len = (x * x + y * y + z * z).sqrt().max(1e-9);
    [x / len, y / len, z / len]
}

/// Runs `f(x, y)` for every pixel of a `w` × `h` image on all cores.
fn par_image(w: usize, h: usize, c: usize, f: impl Fn(usize, usize) -> [f32; 4] + Sync) -> Img {
    let mut out = Img::new(w, h, c);
    let threads = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(h.max(1));
    let rows_per = h.div_ceil(threads);
    std::thread::scope(|s| {
        for (ti, chunk) in out.data.chunks_mut(rows_per * w * c).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (i, px) in chunk.chunks_mut(c).enumerate() {
                    let idx = ti * rows_per * w + i;
                    let v = f(idx % w, idx / w);
                    px.copy_from_slice(&v[..c]);
                }
            });
        }
    });
    out
}

/// 2 x 2 supersample offsets.
const SS: [(f32, f32); 4] = [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)];

/// The studio as an octahedral map, linear radiance ÷ [`ENV_RANGE`] clamped to 0..1.
#[must_use]
pub fn studio_env(n: usize) -> Img {
    par_image(n, n, 3, |x, y| {
        let mut acc = [0.0_f32; 3];
        for (dx, dy) in SS {
            let c = studio(oct_dir((x as f32 + dx) / n as f32, (y as f32 + dy) / n as f32));
            for i in 0..3 {
                acc[i] += c[i] * 0.25;
            }
        }
        [
            (acc[0] / ENV_RANGE).clamp(0.0, 1.0),
            (acc[1] / ENV_RANGE).clamp(0.0, 1.0),
            (acc[2] / ENV_RANGE).clamp(0.0, 1.0),
            1.0,
        ]
    })
}

/// The env band: the studio along the vertical great circle through the key (azimuth −135°),
/// x = 0..1 for the angle −180°..180° from the key-side horizon up through the viewer (90°) to
/// the far horizon (180°) and round behind the screen; 8x supersampled. Linear ÷ [`ENV_RANGE`].
#[must_use]
pub fn env_band(w: usize) -> Img {
    let a = (-135.0_f32).to_radians();
    let (ax, ay) = (a.cos(), a.sin());
    Img::from_fn(w, 1, 3, |x, _| {
        let mut acc = [0.0_f32; 3];
        for s in 0..8 {
            let t = (x as f32 + (s as f32 + 0.5) / 8.0) / w as f32;
            let phi = (t * 360.0 - 180.0).to_radians();
            let c = studio([phi.cos() * ax, phi.cos() * ay, phi.sin()]);
            for i in 0..3 {
                acc[i] += c[i] / 8.0;
            }
        }
        [
            (acc[0] / ENV_RANGE).clamp(0.0, 1.0),
            (acc[1] / ENV_RANGE).clamp(0.0, 1.0),
            (acc[2] / ENV_RANGE).clamp(0.0, 1.0),
            1.0,
        ]
    })
}

// ---------------------------------------------------------------------------------- matcaps

/// Fresnel table over cos θ (256 steps, 0..1) in linear sRGB.
fn fresnel_table(metal: Metal) -> Vec<[f32; 3]> {
    (0..256).map(|i| metal_rgb(metal, i as f32 / 255.0)).collect()
}

/// A Fresnel look-up table: x = cos θ 0..1, linear sRGB reflectance (rows 1).
#[must_use]
pub fn fresnel_lut(metal: Metal) -> Img {
    let t = fresnel_table(metal);
    Img::from_fn(256, 1, 3, |x, _| {
        let c = t[x];
        [c[0].clamp(0.0, 1.0), c[1].clamp(0.0, 1.0), c[2].clamp(0.0, 1.0), 1.0]
    })
}

/// A matcap of a perfectly smooth `metal` sphere in the studio, `n` px square: pixel (x, y) is
/// the view-space normal (u, v, √(1 − u² − v²)) with u, v = −1..1 (y down), lit by the mirror
/// direction r = 2 (n·v) n − v weighted by the metal's Fresnel at n·v. RGB = linear radiance ÷
/// [`ENV_RANGE`]; alpha = disc coverage. 4x4 supersampled (the rim changes fast).
#[must_use]
pub fn matcap(metal: Metal, n: usize) -> Img {
    let table = fresnel_table(metal);
    par_image(n, n, 4, |x, y| {
        let mut acc = [0.0_f32; 3];
        let mut cover = 0.0_f32;
        for sy in 0..4 {
            for sx in 0..4 {
                let u = ((x as f32 + (sx as f32 + 0.5) / 4.0) / n as f32) * 2.0 - 1.0;
                let v = ((y as f32 + (sy as f32 + 0.5) / 4.0) / n as f32) * 2.0 - 1.0;
                let rr = u * u + v * v;
                if rr >= 1.0 {
                    continue;
                }
                cover += 1.0 / 16.0;
                let z = (1.0 - rr).sqrt();
                // r = 2 (n·v) n − v with v = (0, 0, 1).
                let r = [2.0 * z * u, 2.0 * z * v, 2.0 * z * z - 1.0];
                let e = studio(r);
                let f = table[(z * 255.0).round() as usize];
                for i in 0..3 {
                    acc[i] += e[i] * f[i].max(0.0) / 16.0;
                }
            }
        }
        // Premultiplied while summing; store straight colour with coverage alpha.
        let k = if cover > 0.0 { 1.0 / cover } else { 0.0 };
        [
            (acc[0] * k / ENV_RANGE).clamp(0.0, 1.0),
            (acc[1] * k / ENV_RANGE).clamp(0.0, 1.0),
            (acc[2] * k / ENV_RANGE).clamp(0.0, 1.0),
            cover,
        ]
    })
}

// -------------------------------------------------------------------------------- thin film

/// Thickest film in the thin-film tables (nm): The oil-slick blob's drainage runs 900 to 100 nm.
pub const FILM_MAX_NM: f32 = 1200.0;

/// A thin-film table: x = thickness 0..[`FILM_MAX_NM`] nm (`w` steps, first column bare metal),
/// y = cos θ from 1 (top row, face-on) to 0 (`h` steps); linear sRGB reflectance under white
/// light of a lossless film of index `film` on `metal`, clamped to 0..1.
#[must_use]
pub fn film_lut(film: f32, metal: Metal, w: usize, h: usize) -> Img {
    par_image(w, h, 3, |x, y| {
        let d = x as f32 / (w - 1).max(1) as f32 * FILM_MAX_NM;
        let cos0 = 1.0 - y as f32 / (h - 1).max(1) as f32;
        let c = spectrum_rgb(|nm| film_reflectance(film, metal, d, nm, cos0));
        [c[0].clamp(0.0, 1.0), c[1].clamp(0.0, 1.0), c[2].clamp(0.0, 1.0), 1.0]
    })
}

/// Encodes linear 0..1 values as sRGB (the 8-bit runtime copies of the env and matcaps).
#[must_use]
pub fn encode_srgb(src: &Img) -> Img {
    let mut out = src.clone();
    let c = out.c;
    for px in out.data.chunks_mut(c) {
        for v in px.iter_mut().take(3) {
            *v = img::srgb_encode(*v);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_index_reduces_to_the_dielectric_cases() {
        // Bare glass 1.5 face-on: 4%.
        let r = film_on(1.38, C::real(1.5), 0.0, 550.0, 1.0);
        assert!((r - 0.04).abs() < 1e-4, "{r}");
        // Quarter-wave MgF2 on crown glass reflects 1.26% at its design wavelength (optics.md §9).
        let t = 550.0 / (4.0 * 1.38);
        let r = film_on(1.38, C::real(1.52), t, 550.0, 1.0);
        assert!((r - 0.0126).abs() < 2e-4, "{r}");
        // Ideal index n1 = √(n0 n2): no reflection at λ0.
        let n1 = 1.52_f32.sqrt();
        let r = film_on(n1, C::real(1.52), 550.0 / (4.0 * n1), 550.0, 1.0);
        assert!(r < 1e-6, "{r}");
    }

    #[test]
    fn a_film_of_zero_thickness_is_the_bare_metal() {
        for m in [Metal::Chrome, Metal::Mercury] {
            for cos0 in [1.0, 0.7, 0.2] {
                for nm in [450.0, 550.0, 650.0] {
                    let a = film_reflectance(1.47, m, 0.0, nm, cos0);
                    let b = conductor_reflectance(m, nm, cos0);
                    assert!((a - b).abs() < 1e-5, "{m:?} {cos0} {nm}: {a} vs {b}");
                }
            }
        }
    }

    #[test]
    fn metals_reflect_what_their_measurements_say() {
        // Face-on R = ((n−1)² + k²) ÷ ((n+1)² + k²); mercury about 0.78 at 550 nm, chromium about
        // 0.55, and every metal goes to 1 at grazing incidence.
        let r = |n: f32, k: f32| ((n - 1.0).powi(2) + k * k) / ((n + 1.0).powi(2) + k * k);
        let hg = conductor_reflectance(Metal::Mercury, 549.0, 1.0);
        let want = r(1.5507, 4.6337);
        assert!((hg - want).abs() < 0.005 && (0.74..0.80).contains(&hg), "Hg {hg}");
        let cr = conductor_reflectance(Metal::Chrome, 549.0, 1.0);
        assert!((cr - r(3.18, 3.33)).abs() < 1e-4 && (0.5..0.6).contains(&cr), "Cr {cr}");
        for m in [Metal::Chrome, Metal::Mercury] {
            assert!(conductor_reflectance(m, 550.0, 1e-4) > 0.99);
        }
    }

    #[test]
    fn a_flat_spectrum_is_white() {
        let c = spectrum_rgb(|_| 1.0);
        for v in c {
            assert!((v - 1.0).abs() < 0.01, "{c:?}");
        }
    }

    #[test]
    fn the_studio_is_black_round_the_viewer_and_lit_where_it_says() {
        assert_eq!(studio([0.0, 0.0, 1.0]), [0.0; 3]);
        let dir = |az: f32, el: f32| {
            let (a, e) = (az.to_radians(), el.to_radians());
            [e.cos() * a.cos(), e.cos() * a.sin(), e.sin()]
        };
        assert!(studio(dir(-135.0, 40.0))[0] >= 10.0);
        assert!(studio(dir(-105.0, 24.0))[0] >= 5.0);
        assert!(studio(dir(45.0, 6.0))[2] >= 3.4);
        // Behind the screen the sky sits above a darker floor.
        let sky = studio([0.0, -0.7, -0.7])[0];
        let floor = studio([0.0, 0.7, -0.7])[0];
        assert!(sky > 4.0 * floor, "{sky} {floor}");
    }

    #[test]
    fn a_mirror_sphere_shows_the_metal_times_the_studio() {
        // The matcap's centre faces the viewer and mirrors the black clear cone; its key-facing
        // point (tilt 25° towards azimuth −135°) mirrors the key strip at elevation 40°.
        let m = matcap(Metal::Mercury, 256);
        assert!(m.get(128, 128, 0) < 1e-6);
        let tilt = 25.0_f32.to_radians();
        let a = (-135.0_f32).to_radians();
        let (u, v) = (tilt.sin() * a.cos(), tilt.sin() * a.sin());
        let (x, y) = (((u + 1.0) * 0.5 * 256.0) as usize, ((v + 1.0) * 0.5 * 256.0) as usize);
        let want = 10.0 * metal_rgb(Metal::Mercury, tilt.cos())[1] / ENV_RANGE;
        assert!((m.get(x, y, 1) - want).abs() < 0.05, "{} vs {want}", m.get(x, y, 1));
        assert!(m.get(0, 0, 3) == 0.0 && m.get(128, 128, 3) == 1.0);
    }

    #[test]
    fn oil_on_chrome_shows_interference_colour() {
        // A 300 nm oil film on chrome: its colour must differ visibly from bare chrome.
        let bare = spectrum_rgb(|nm| conductor_reflectance(Metal::Chrome, nm, 1.0));
        let oil = spectrum_rgb(|nm| film_reflectance(1.47, Metal::Chrome, 300.0, nm, 1.0));
        let spread =
            |c: [f32; 3]| c.iter().copied().fold(f32::MIN, f32::max) - c.iter().copied().fold(f32::MAX, f32::min);
        assert!(spread(oil) > spread(bare) + 0.05, "bare {bare:?} oil {oil:?}");
    }
}
