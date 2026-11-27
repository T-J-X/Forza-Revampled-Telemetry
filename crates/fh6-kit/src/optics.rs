//! Physical optics: the CPU reference for every equation in `docs/specs/optics.md`.
//!
//! The glass shader (`shaders/shape.wgsl`, `optic_glass`) evaluates the same equations per pixel.
//! The unit tests here pin each equation to a known value, and `tests/glass_gpu.rs` compares
//! rendered pixels with [`view_through`].
//!
//! Conventions: the screen is the xy plane (y down) and z points at the viewer. The viewer is
//! orthographic, so every view ray runs along -z. Directions are unit vectors. `eta` is always
//! the index of the medium being entered divided by the index of the medium being left.
use crate::math::{vec2, vec3, Vec2, Vec3};
use core::f32::consts::PI;

// ------------------------------------------------------------------------------ dispersion

/// Fraunhofer d line (helium, 587.56 nm): where `n_d` is quoted.
pub const LINE_D_NM: f64 = 587.56;
/// Fraunhofer F line (hydrogen, 486.13 nm).
pub const LINE_F_NM: f64 = 486.13;
/// Fraunhofer C line (hydrogen, 656.27 nm).
pub const LINE_C_NM: f64 = 656.27;

/// The wavelengths (nm) at which the red, green and blue channels are refracted.
///
/// They are the dominant wavelengths of the sRGB primaries (a test derives them from the CIE
/// fit). Light from a display primary is narrow-band around this wavelength, so refracting each
/// channel with its own index is the dispersion of an RGB image (optics.md §5).
pub const PRIMARY_NM: [f64; 3] = [611.0, 549.0, 464.0];

/// How a material's refractive index depends on wavelength.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dispersion {
    /// Three-term Sellmeier: `n² − 1 = Σ Bᵢ λ² / (λ² − Cᵢ)`, with λ in µm and `Cᵢ` in µm².
    Sellmeier {
        /// Oscillator strengths `B₁..B₃`.
        b: [f64; 3],
        /// Resonance wavelengths squared `C₁..C₃`, µm².
        c: [f64; 3],
    },
    /// Two-term Cauchy: `n = A + B / λ²`, with λ in µm. Valid for normal dispersion in the
    /// visible range only.
    Cauchy {
        /// Constant term.
        a: f64,
        /// Dispersion term, µm².
        b: f64,
    },
}

impl Dispersion {
    /// Synthetic sapphire (α-Al₂O₃), ordinary ray, 20 °C (Malitson and Dodge 1972).
    /// `n_d` = 1.7682, `V_d` = 72.3. The 0.008 birefringence is ignored.
    pub const SAPPHIRE: Self = Self::Sellmeier {
        b: [1.431_349_3, 0.650_547_13, 5.341_402_1],
        c: [
            0.072_663_1 * 0.072_663_1,
            0.119_324_2 * 0.119_324_2,
            18.028_251 * 18.028_251,
        ],
    };

    /// Schott N-BK7 borosilicate crown glass (Schott catalogue). `n_d` = 1.5168, `V_d` = 64.17.
    pub const CROWN_BK7: Self = Self::Sellmeier {
        b: [1.039_612_12, 0.231_792_344, 1.010_469_45],
        c: [0.006_000_698_67, 0.020_017_914_4, 103.560_653],
    };

    /// Fused silica, 20 °C (Malitson 1965). `n_d` = 1.4585, `V_d` = 67.8.
    pub const FUSED_SILICA: Self = Self::Sellmeier {
        b: [0.696_166_3, 0.407_942_6, 0.897_479_4],
        c: [
            0.068_404_3 * 0.068_404_3,
            0.116_241_4 * 0.116_241_4,
            9.896_161 * 9.896_161,
        ],
    };

    /// A Cauchy model through `n_d` with the dispersion of Abbe number `v_d`, for materials
    /// specified by those two numbers only: `B = (n_d − 1) / (V_d (λ_F⁻² − λ_C⁻²))`.
    #[must_use]
    pub fn from_abbe(n_d: f64, v_d: f64) -> Self {
        let inv2 = |nm: f64| 1.0e6 / (nm * nm);
        let b = if v_d.is_finite() && v_d > 0.0 {
            (n_d - 1.0) / (v_d * (inv2(LINE_F_NM) - inv2(LINE_C_NM)))
        } else {
            0.0
        };
        Self::Cauchy {
            a: n_d - b * inv2(LINE_D_NM),
            b,
        }
    }

    /// The refractive index at `wavelength_nm` (clamped to 300..2000 nm; never below 1).
    #[must_use]
    pub fn index(&self, wavelength_nm: f64) -> f64 {
        let nm = if wavelength_nm.is_finite() {
            wavelength_nm.clamp(300.0, 2000.0)
        } else {
            LINE_D_NM
        };
        let l2 = (nm / 1000.0) * (nm / 1000.0);
        match *self {
            Self::Sellmeier { b, c } => {
                let mut n2 = 1.0;
                for (bi, ci) in b.iter().zip(c.iter()) {
                    let den = l2 - ci;
                    if den.abs() > 1e-9 {
                        n2 += bi * l2 / den;
                    }
                }
                n2.max(1.0).sqrt()
            }
            Self::Cauchy { a, b } => (a + b / l2).max(1.0),
        }
    }

    /// The index at the d line.
    #[must_use]
    pub fn n_d(&self) -> f64 {
        self.index(LINE_D_NM)
    }

    /// The Abbe number `V_d = (n_d − 1) / (n_F − n_C)`.
    #[must_use]
    pub fn abbe(&self) -> f64 {
        let spread = self.index(LINE_F_NM) - self.index(LINE_C_NM);
        if spread.abs() > 1e-12 {
            (self.n_d() - 1.0) / spread
        } else {
            f64::INFINITY
        }
    }

    /// The indices for the red, green and blue channels (at [`PRIMARY_NM`]).
    #[must_use]
    pub fn primaries(&self) -> [f32; 3] {
        PRIMARY_NM.map(|nm| self.index(nm) as f32)
    }
}

// ------------------------------------------------------------------------------- interface

/// Reflectance at normal incidence, `R₀ = ((η − 1) / (η + 1))²`.
#[must_use]
pub fn normal_reflectance(eta: f32) -> f32 {
    let r = (eta - 1.0) / (eta + 1.0);
    r * r
}

/// The exact Fresnel reflectance of a dielectric interface for unpolarised light.
///
/// `cos_i` is the cosine between the surface normal and the incident direction. A negative
/// `cos_i` means the ray arrives from inside, and the interface is flipped. Returns 1 under
/// total internal reflection.
#[must_use]
pub fn fresnel_dielectric(cos_i: f32, eta: f32) -> f32 {
    if !(eta.is_finite() && eta > 0.0 && cos_i.is_finite()) {
        return 1.0;
    }
    let (cos_i, eta) = if cos_i < 0.0 {
        ((-cos_i).min(1.0), 1.0 / eta)
    } else {
        (cos_i.min(1.0), eta)
    };
    let sin2_t = (1.0 - cos_i * cos_i) / (eta * eta);
    if sin2_t >= 1.0 {
        return 1.0;
    }
    let cos_t = (1.0 - sin2_t).sqrt();
    let r_par = (eta * cos_i - cos_t) / (eta * cos_i + cos_t);
    let r_perp = (cos_i - eta * cos_t) / (cos_i + eta * cos_t);
    0.5 * (r_par * r_par + r_perp * r_perp)
}

/// Schlick's approximation, `R₀ + (1 − R₀)(1 − cos θ)⁵`. Cheaper than
/// [`fresnel_dielectric`] and within 0.04 of it for the indices in this file (tested; the
/// worst case is near 85°); the glass shader uses the exact form.
#[must_use]
pub fn fresnel_schlick(cos_i: f32, r0: f32) -> f32 {
    let m = (1.0 - cos_i).clamp(0.0, 1.0);
    let m2 = m * m;
    r0 + (1.0 - r0) * m2 * m2 * m
}

/// The mirror direction of `wo` about the normal `n`: `−wo + 2 (n·wo) n`.
#[must_use]
pub fn reflect(wo: Vec3, n: Vec3) -> Vec3 {
    n * (2.0 * wo.dot(n)) - wo
}

/// Snell's law in vector form. `wi` points away from the surface on the incident side, `n` is
/// the surface normal and `eta` is the index entered over the index left, relative to `n`.
/// Returns the transmitted direction, or `None` under total internal reflection.
#[must_use]
pub fn refract(wi: Vec3, n: Vec3, eta: f32) -> Option<Vec3> {
    if !(eta.is_finite() && eta > 0.0) {
        return None;
    }
    let mut cos_i = n.dot(wi);
    let mut eta = eta;
    let mut n = n;
    if cos_i < 0.0 {
        eta = 1.0 / eta;
        cos_i = -cos_i;
        n = n * -1.0;
    }
    let sin2_i = (1.0 - cos_i * cos_i).max(0.0);
    let sin2_t = sin2_i / (eta * eta);
    if sin2_t >= 1.0 {
        return None;
    }
    let cos_t = (1.0 - sin2_t).sqrt();
    Some(wi * (-1.0 / eta) + n * (cos_i / eta - cos_t))
}

// ------------------------------------------------------------------------------ absorption

/// The absorption coefficient that gives internal `transmittance` (0..1] after `depth` of
/// glass: `σ = −ln T / d` (per unit of `depth`).
#[must_use]
pub fn absorption_coefficient(transmittance: f32, depth: f32) -> f32 {
    if !(depth.is_finite() && depth > 0.0) {
        return 0.0;
    }
    -transmittance.clamp(1e-6, 1.0).ln() / depth
}

/// Beer-Lambert internal transmittance over a path of length `path`: `T = e^(−σ ℓ)`.
#[must_use]
pub fn transmittance(sigma: f32, path: f32) -> f32 {
    (-sigma.max(0.0) * path.max(0.0)).exp()
}

// -------------------------------------------------------------------------------- specular

/// The GGX (Trowbridge-Reitz) distribution of microfacet normals,
/// `D = α² / (π (cos²θ_h (α² − 1) + 1)²)`.
#[must_use]
pub fn ggx_d(cos_h: f32, alpha: f32) -> f32 {
    let a2 = alpha.max(1e-4) * alpha.max(1e-4);
    let c = cos_h.clamp(0.0, 1.0);
    let den = c * c * (a2 - 1.0) + 1.0;
    a2 / (PI * den * den)
}

/// Smith's auxiliary function for GGX, `Λ = (√(1 + α² tan²θ) − 1) / 2`.
#[must_use]
pub fn smith_lambda(cos: f32, alpha: f32) -> f32 {
    let c = cos.abs().clamp(1e-4, 1.0);
    let tan2 = (1.0 - c * c) / (c * c);
    0.5 * ((1.0 + alpha * alpha * tan2).sqrt() - 1.0)
}

/// The height-correlated Smith masking-shadowing term, `G = 1 / (1 + Λ(l) + Λ(v))`.
#[must_use]
pub fn smith_g(cos_l: f32, cos_v: f32, alpha: f32) -> f32 {
    1.0 / (1.0 + smith_lambda(cos_l, alpha) + smith_lambda(cos_v, alpha))
}

/// The Torrance-Sparrow specular term without Fresnel, `D G / (4 cos θ_l cos θ_v)`. Multiply
/// by the Fresnel reflectance at the half vector and by the light's irradiance times
/// `cos θ_l`.
#[must_use]
pub fn ggx_specular(cos_h: f32, cos_l: f32, cos_v: f32, alpha: f32) -> f32 {
    if cos_l <= 0.0 || cos_v <= 0.0 {
        return 0.0;
    }
    ggx_d(cos_h, alpha) * smith_g(cos_l, cos_v, alpha) / (4.0 * cos_l * cos_v)
}

// ---------------------------------------------------------------------------------- colour

/// The CIE 1931 2° colour matching functions `(x̄, ȳ, z̄)` at `wavelength_nm`, from the
/// multi-lobe Gaussian fit of Wyman, Sloan and Shirley (2013).
#[must_use]
pub fn cie_xyz(wavelength_nm: f32) -> [f32; 3] {
    let lobe = |mu: f32, below: f32, above: f32| {
        let t = (wavelength_nm - mu) * if wavelength_nm < mu { below } else { above };
        (-0.5 * t * t).exp()
    };
    [
        0.362 * lobe(442.0, 0.0624, 0.0374) + 1.056 * lobe(599.8, 0.0264, 0.0323) - 0.065 * lobe(501.1, 0.0490, 0.0382),
        0.821 * lobe(568.8, 0.0213, 0.0247) + 0.286 * lobe(530.9, 0.0613, 0.0322),
        1.217 * lobe(437.0, 0.0845, 0.0278) + 0.681 * lobe(459.0, 0.0385, 0.0725),
    ]
}

/// CIE XYZ (D65 white at Y = 1) to linear sRGB (IEC 61966-2-1).
#[must_use]
pub fn xyz_to_linear_srgb(xyz: [f32; 3]) -> [f32; 3] {
    let [x, y, z] = xyz;
    [
        3.2406 * x - 1.5372 * y - 0.4986 * z,
        -0.9689 * x + 1.8758 * y + 0.0415 * z,
        0.0557 * x - 0.2040 * y + 1.0570 * z,
    ]
}

/// The linear sRGB colour of a reflectance or transmittance spectrum `f(λ nm)` under white
/// light, balanced so that a flat spectrum of 1 is display white `(1, 1, 1)`.
#[must_use]
pub fn spectrum_to_linear_srgb(f: impl Fn(f32) -> f32) -> [f32; 3] {
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
    let xyz = [
        sum[0] / white[0] * D65[0],
        sum[1] / white[1] * D65[1],
        sum[2] / white[2] * D65[2],
    ];
    xyz_to_linear_srgb(xyz)
}

// ------------------------------------------------------------------------------- thin film

/// Reflectance of a single thin film (index `n1`, `thickness_nm`) on a substrate (`n2`) seen
/// from a medium (`n0`), for unpolarised light of `wavelength_nm` arriving with cosine `cos0`
/// to the normal. Airy summation of the two interface reflections with phase
/// `β = 2π n₁ d cos θ₁ / λ`.
#[must_use]
pub fn thin_film_reflectance(n0: f32, n1: f32, n2: f32, thickness_nm: f32, wavelength_nm: f32, cos0: f32) -> f32 {
    let cos0 = cos0.clamp(0.0, 1.0);
    let sin2_0 = 1.0 - cos0 * cos0;
    let cos_in = |n: f32| {
        let s2 = (n0 / n) * (n0 / n) * sin2_0;
        (s2 < 1.0).then(|| (1.0 - s2).sqrt())
    };
    let (Some(cos1), Some(cos2)) = (cos_in(n1), cos_in(n2)) else {
        return 1.0;
    };
    let beta2 = 4.0 * PI * n1 * thickness_nm.max(0.0) * cos1 / wavelength_nm.max(1.0);
    let cos_2b = beta2.cos();
    let airy = |r01: f32, r12: f32| {
        let p = r01 * r12;
        (r01 * r01 + r12 * r12 + 2.0 * p * cos_2b) / (1.0 + p * p + 2.0 * p * cos_2b)
    };
    let s = airy(
        (n0 * cos0 - n1 * cos1) / (n0 * cos0 + n1 * cos1),
        (n1 * cos1 - n2 * cos2) / (n1 * cos1 + n2 * cos2),
    );
    let p = airy(
        (n1 * cos0 - n0 * cos1) / (n1 * cos0 + n0 * cos1),
        (n2 * cos1 - n1 * cos2) / (n2 * cos1 + n1 * cos2),
    );
    0.5 * (s + p)
}

// ----------------------------------------------------------------------------------- prism

/// The angle (radians) by which a prism of apex angle `apex` and index `n` turns a ray that
/// meets its first face at `incidence` to the normal: `δ = θ₁ + θ₂ − A`, with Snell's law at
/// both faces. `None` when the ray is totally reflected at the second face.
#[must_use]
pub fn prism_deviation(n: f32, apex: f32, incidence: f32) -> Option<f32> {
    let inside_1 = (incidence.sin() / n).clamp(-1.0, 1.0).asin();
    let inside_2 = apex - inside_1;
    let s = n * inside_2.sin();
    (s.abs() < 1.0).then(|| incidence + s.asin() - apex)
}

// ------------------------------------------------------------------------------ glass body

/// The cross-section of a glass piece, as a height above the content plane's rim level.
///
/// With `s` the distance inwards from the outline, the top surface rises over the bevel as a
/// superellipse, `h(s) = H (1 − (1 − s/w)^k)^(1/k)`, and is flat at `H` beyond it. `k = 2` is a
/// circular arc; with `H = w` equal to the radius of a circular outline the piece is a
/// hemisphere. Under the bevel sits `base` more glass, down to the plane that carries what the
/// piece shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Profile {
    /// Rise of the top surface above the rim, `H`.
    pub thickness: f32,
    /// Width of the bevel measured inwards from the outline, `w`.
    pub bevel: f32,
    /// Glass between the rim level and the content plane, `b`.
    pub base: f32,
    /// Superellipse exponent `k` of the bevel (1 = chamfer, 2 = circular arc, larger = squarer
    /// shoulder).
    pub exponent: f32,
}

impl Profile {
    fn unit(&self, inset: f32) -> (f32, f32) {
        let u = (inset / self.bevel.max(1e-6)).clamp(0.0, 1.0);
        (u, self.exponent.clamp(1.0, 8.0))
    }

    /// Height of the top surface at `inset` from the outline.
    #[must_use]
    pub fn height(&self, inset: f32) -> f32 {
        let (u, k) = self.unit(inset);
        self.thickness.max(0.0) * (1.0 - (1.0 - u).powf(k)).max(0.0).powf(1.0 / k)
    }

    /// The unit surface normal at `inset`, where `outward` is the unit gradient of the
    /// outline's signed distance (pointing out of the piece).
    #[must_use]
    pub fn normal(&self, inset: f32, outward: Vec2) -> Vec3 {
        let (u, k) = self.unit(inset);
        if u >= 1.0 {
            return vec3(0.0, 0.0, 1.0);
        }
        let rise = self.thickness.max(0.0) * (1.0 - u).powf(k - 1.0);
        let run = self.bevel.max(1e-6) * (1.0 - (1.0 - u).powf(k)).max(0.0).powf(1.0 - 1.0 / k);
        let n = vec3(outward.x * rise, outward.y * rise, run).normalize_or_zero();
        if n == Vec3::default() {
            vec3(0.0, 0.0, 1.0)
        } else {
            n
        }
    }

    /// How fast the surface normal turns per unit of inset along the slope, in radians per unit
    /// (`docs/specs/optics.md` §7, pixel footprint). With `G = |∇(h^k)|` and `U = k h^(k−1)` the
    /// normal's tilt is `atan(G / U)`, so `dθ/ds = (G′U − G U′) / (G² + U²)`. It is negative on
    /// the bevel (the normal rights itself inwards), zero on the flat top and unbounded at the
    /// rim.
    #[must_use]
    pub fn turn_rate(&self, inset: f32) -> f32 {
        let (u, k) = self.unit(inset);
        if u >= 1.0 {
            return 0.0;
        }
        let w = self.bevel.max(1e-6);
        let hk = self.thickness.max(0.0).powf(k);
        let t = (1.0 - u).max(1e-4);
        let g = hk * k * t.powf(k - 1.0) / w;
        let g_s = -hk * k * (k - 1.0) * t.powf(k - 2.0) / (w * w);
        let h = self.height(inset).max(1e-5);
        let up = k * h.powf(k - 1.0);
        let u_s = (k - 1.0) * g / h;
        (g_s * up - g * u_s) / (g * g + up * up).max(1e-12)
    }
}

/// Height of a ring wave packet on a glass surface at distance `r` from its centre
/// (`docs/specs/optics.md` §2): `A e^(−x²) cos(2πx)` with `x = (r − R) / λ`.
#[must_use]
pub fn ripple_height(amplitude: f32, wavelength: f32, radius: f32, r: f32) -> f32 {
    let x = (r - radius) / wavelength.max(1e-6);
    amplitude * (-x * x).exp() * (2.0 * PI * x).cos()
}

/// Radial slope of that packet, `dη/dr`: what tilts the surface normal.
#[must_use]
pub fn ripple_slope(amplitude: f32, wavelength: f32, radius: f32, r: f32) -> f32 {
    let l = wavelength.max(1e-6);
    let x = (r - radius) / l;
    amplitude * (-x * x).exp() * (-2.0 * x * (2.0 * PI * x).cos() - 2.0 * PI * (2.0 * PI * x).sin()) / l
}

/// Variance of the pixel filter (a Gaussian of peak 1: 1 / 2π) and the clamp of the footprint
/// filter (`docs/specs/optics.md` §7, [S22]).
pub const FOOTPRINT_SIGMA2: f32 = 0.159_154_94;
/// See [`FOOTPRINT_SIGMA2`].
pub const FOOTPRINT_KAPPA: f32 = 0.18;

/// The squared GGX roughness of a highlight filtered over one pixel, across which the surface
/// normal changes by `dn_dx` and `dn_dy`: `α² + min(2σ²(|∂n/∂x|² + |∂n/∂y|²), κ)` [S22]. A
/// surface that curves inside a pixel shows a wider, dimmer highlight instead of a flickering
/// one.
#[must_use]
pub fn footprint_roughness_sq(alpha: f32, dn_dx: Vec3, dn_dy: Vec3) -> f32 {
    let spread = 2.0 * FOOTPRINT_SIGMA2 * (dn_dx.dot(dn_dx) + dn_dy.dot(dn_dy));
    (alpha * alpha + spread.min(FOOTPRINT_KAPPA)).clamp(0.0, 1.0)
}

/// What the viewer sees through one point of a glass piece.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sight {
    /// The surface normal.
    pub normal: Vec3,
    /// Fresnel reflectance of the surface for the view ray.
    pub reflectance: f32,
    /// Where the refracted ray meets the content plane, relative to the surface point.
    pub offset: Vec2,
    /// Length of the ray's path inside the glass.
    pub path: f32,
}

/// Traces the view ray through the top surface of a piece to its content plane: the normal
/// from the [`Profile`], the exact Fresnel reflectance, Snell refraction and the path length
/// that Beer-Lambert absorption uses. `eta` is the glass index (the viewer is in air).
#[must_use]
pub fn view_through(profile: &Profile, inset: f32, outward: Vec2, eta: f32) -> Sight {
    let normal = profile.normal(inset, outward);
    let depth = profile.height(inset) + profile.base.max(0.0);
    let reflectance = fresnel_dielectric(normal.z, eta);
    match refract(vec3(0.0, 0.0, 1.0), normal, eta) {
        Some(wt) if wt.z < -1e-6 => {
            let t = depth / -wt.z;
            Sight {
                normal,
                reflectance,
                offset: vec2(wt.x, wt.y) * t,
                path: t,
            }
        }
        _ => Sight {
            normal,
            reflectance: 1.0,
            offset: Vec2::ZERO,
            path: depth,
        },
    }
}

// ------------------------------------------------------------- relativistic and plate waves

/// The relativistic Doppler factor `D = 1 / (γ (1 − β cos θ))` of a source moving at speed
/// `beta` (in units of c) at angle θ to the line of sight (`cos_theta` = 1 is head-on
/// approach). Observed frequency is `D` times the emitted frequency.
#[must_use]
pub fn doppler_factor(beta: f32, cos_theta: f32) -> f32 {
    let beta = beta.clamp(0.0, 0.999_999);
    let gamma = 1.0 / (1.0 - beta * beta).sqrt();
    1.0 / (gamma * (1.0 - beta * cos_theta.clamp(-1.0, 1.0)))
}

/// Relativistic beaming of a source whose spectrum is `S ∝ ν^α`: the observed brightness at a
/// fixed frequency is `D^(3 − α)` times the emitted one.
#[must_use]
pub fn beaming_gain(doppler: f32, spectral_index: f32) -> f32 {
    doppler.max(0.0).powf(3.0 - spectral_index)
}

/// Angular frequency of mode `(m, n)` of a simply supported rectangular plate `a × b`:
/// `ω = π² (m²/a² + n²/b²) √(D / 2ρh)`, where `stiffness` is `√(D / 2ρh)`.
#[must_use]
pub fn plate_mode_frequency(m: u32, n: u32, a: f32, b: f32, stiffness: f32) -> f32 {
    let (m, n) = (m as f32, n as f32);
    PI * PI * (m * m / (a * a) + n * n / (b * b)) * stiffness
}

/// The standing-wave amplitude of a square plate of side `l` fixed at its centre, for integer
/// modes `m ≠ n`: `cos(nπx/l) cos(mπy/l) − cos(mπx/l) cos(nπy/l)`. Sand collects where it is
/// zero. For `m = n` it is zero everywhere, so that pair gives no figure.
#[must_use]
pub fn chladni_square(x: f32, y: f32, m: u32, n: u32, l: f32) -> f32 {
    let k = PI / l.max(1e-6);
    let (m, n) = (m as f32, n as f32);
    (n * k * x).cos() * (m * k * y).cos() - (m * k * x).cos() * (n * k * y).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(got: f32, want: f32, tol: f32, what: &str) {
        assert!((got - want).abs() <= tol, "{what}: got {got}, want {want} ± {tol}");
    }

    fn close64(got: f64, want: f64, tol: f64, what: &str) {
        assert!((got - want).abs() <= tol, "{what}: got {got}, want {want} ± {tol}");
    }

    #[test]
    fn sapphire_is_not_crown_glass() {
        // Card P8-1 called n = 1.52 "sapphire". Sapphire is 1.768; 1.52 is a crown glass.
        close64(Dispersion::SAPPHIRE.n_d(), 1.768_17, 5e-5, "sapphire n_d");
        close64(Dispersion::SAPPHIRE.abbe(), 72.3, 0.2, "sapphire V_d");
        close64(Dispersion::CROWN_BK7.n_d(), 1.5168, 5e-5, "N-BK7 n_d");
        close64(Dispersion::CROWN_BK7.abbe(), 64.17, 0.02, "N-BK7 V_d");
        close64(Dispersion::FUSED_SILICA.n_d(), 1.458_46, 5e-5, "fused silica n_d");
        close64(Dispersion::FUSED_SILICA.abbe(), 67.82, 0.05, "fused silica V_d");
        close(
            normal_reflectance(Dispersion::SAPPHIRE.n_d() as f32),
            0.0770,
            2e-4,
            "sapphire R0",
        );
        close(
            normal_reflectance(Dispersion::CROWN_BK7.n_d() as f32),
            0.0422,
            2e-4,
            "N-BK7 R0",
        );
        close(
            normal_reflectance(1.52),
            0.0426,
            1e-4,
            "the old card's R0 was for n = 1.52",
        );
    }

    #[test]
    fn index_falls_with_wavelength_in_the_visible() {
        for d in [Dispersion::SAPPHIRE, Dispersion::CROWN_BK7, Dispersion::FUSED_SILICA] {
            let [r, g, b] = d.primaries();
            assert!(b > g && g > r, "{d:?}: blue {b} > green {g} > red {r}");
        }
        let [r, _, b] = Dispersion::SAPPHIRE.primaries();
        close(
            b - r,
            0.0109,
            5e-4,
            "sapphire index spread between the blue and red channels",
        );
    }

    #[test]
    fn cauchy_from_abbe_reproduces_the_catalogue_lines() {
        let c = Dispersion::from_abbe(1.5168, 64.17);
        close64(c.n_d(), 1.5168, 1e-9, "n_d");
        close64(c.abbe(), 64.17, 1e-6, "V_d");
        for nm in [440.0, 464.0, 549.0, 611.0, 660.0] {
            close64(
                c.index(nm),
                Dispersion::CROWN_BK7.index(nm),
                1e-3,
                "Cauchy vs Sellmeier",
            );
        }
    }

    #[test]
    fn fresnel_known_values() {
        close(fresnel_dielectric(1.0, 1.5), 0.04, 1e-6, "n = 1.5, normal incidence");
        // At Brewster's angle the parallel part vanishes and R_s = ((n² − 1) / (n² + 1))².
        let brewster = 1.5_f32.atan().cos();
        close(
            fresnel_dielectric(brewster, 1.5),
            0.5 * (1.25_f32 / 3.25).powi(2),
            1e-6,
            "Brewster",
        );
        close(fresnel_dielectric(0.0, 1.5), 1.0, 1e-6, "grazing");
        // From inside, past the critical angle asin(1 / 1.5) = 41.8°.
        close(
            fresnel_dielectric(-(45.0_f32.to_radians().cos()), 1.5),
            1.0,
            0.0,
            "total internal reflection",
        );
        close(fresnel_dielectric(-1.0, 1.5), 0.04, 1e-6, "inside, normal incidence");
    }

    #[test]
    fn fresnel_matches_the_sine_tangent_form() {
        for eta in [1.458_f32, 1.5168, 1.7682] {
            for deg in [5.0_f32, 20.0, 45.0, 60.0, 75.0, 85.0] {
                let i = deg.to_radians();
                let t = (i.sin() / eta).asin();
                let rs = ((i - t).sin() / (i + t).sin()).powi(2);
                let rp = ((i - t).tan() / (i + t).tan()).powi(2);
                close(
                    fresnel_dielectric(i.cos(), eta),
                    0.5 * (rs + rp),
                    2e-6,
                    "sine-tangent form",
                );
            }
        }
    }

    #[test]
    fn reflection_plus_transmission_is_one() {
        // Transmittance from the amplitude coefficients, independent of the reflectance code.
        for eta in [1.33_f32, 1.5168, 1.7682, 2.4] {
            for step in 0..90 {
                let i = (step as f32).to_radians();
                let (ci, ct) = (i.cos(), (1.0 - (i.sin() / eta).powi(2)).sqrt());
                let ts = 2.0 * ci / (ci + eta * ct);
                let tp = 2.0 * ci / (eta * ci + ct);
                let t = 0.5 * (eta * ct / ci) * (ts * ts + tp * tp);
                let r = fresnel_dielectric(ci, eta);
                close(r + t, 1.0, 2e-5, "R + T");
                assert!((0.0..=1.0).contains(&r), "R in range: {r}");
            }
        }
    }

    #[test]
    fn schlick_stays_within_its_stated_error() {
        for eta in [1.458_f32, 1.5168, 1.7682] {
            let r0 = normal_reflectance(eta);
            let mut worst = 0.0_f32;
            for step in 0..=900 {
                let c = (step as f32 * 0.1).to_radians().cos();
                worst = worst.max((fresnel_schlick(c, r0) - fresnel_dielectric(c, eta)).abs());
            }
            assert!(worst < 0.04, "eta {eta}: Schlick is off by {worst}");
            assert!(
                worst > 0.02,
                "eta {eta}: the stated error is no longer the real one ({worst})"
            );
        }
    }

    #[test]
    fn refraction_obeys_snell() {
        let n = vec3(0.0, 0.0, 1.0);
        for eta in [1.5168_f32, 1.7682] {
            for deg in [0.0_f32, 10.0, 30.0, 60.0, 89.0] {
                let i = deg.to_radians();
                let wi = vec3(i.sin(), 0.0, i.cos());
                let wt = refract(wi, n, eta).expect("air to glass always transmits");
                close(wt.length(), 1.0, 1e-5, "unit direction");
                close(eta * wt.x.abs(), i.sin(), 1e-5, "eta sin(t) = sin(i)");
                assert!(wt.z < 0.0 && wt.x <= 0.0, "continues to the far side: {wt:?}");
                // Reversibility: light coming back along the refracted path leaves along `wi`.
                let back = refract(wt, n, eta).expect("reverse path");
                close((back - wi).length(), 0.0, 1e-4, "reversible");
            }
        }
        let inside = vec3(60.0_f32.to_radians().sin(), 0.0, -(60.0_f32.to_radians().cos()));
        assert!(
            refract(inside, n, 1.5).is_none(),
            "60° inside n = 1.5 is past the 41.8° critical angle"
        );
    }

    #[test]
    fn mirror_reflection() {
        let wo = vec3(0.6, 0.0, 0.8);
        let r = reflect(wo, vec3(0.0, 0.0, 1.0));
        close((r - vec3(-0.6, 0.0, 0.8)).length(), 0.0, 1e-6, "mirror");
    }

    #[test]
    fn beer_lambert() {
        let sigma = absorption_coefficient(0.5, 10.0);
        close(transmittance(sigma, 0.0), 1.0, 0.0, "no path");
        close(transmittance(sigma, 10.0), 0.5, 1e-6, "reference depth");
        close(transmittance(sigma, 20.0), 0.25, 1e-6, "double the depth squares it");
        close(transmittance(sigma, 5.0), 0.5_f32.sqrt(), 1e-6, "half the depth");
        close(
            transmittance(sigma, 3.0) * transmittance(sigma, 4.0),
            transmittance(sigma, 7.0),
            1e-6,
            "paths multiply",
        );
    }

    #[test]
    fn a_ripple_is_a_packet_with_the_stated_slope() {
        let (a, l, big_r) = (3.0, 24.0, 90.0);
        close(ripple_height(a, l, big_r, big_r), a, 1e-6, "the crest sits on the ring");
        close(ripple_slope(a, l, big_r, big_r), 0.0, 1e-6, "a crest is level");
        // The slope is the derivative of the height.
        for r in [60.0_f32, 78.0, 85.0, 93.0, 101.5, 120.0] {
            let numeric = (ripple_height(a, l, big_r, r + 0.01) - ripple_height(a, l, big_r, r - 0.01)) / 0.02;
            close(ripple_slope(a, l, big_r, r), numeric, 2e-3, "slope");
        }
        // Three wavelengths away the surface is still (the shader stops there).
        assert!(ripple_height(a, l, big_r, big_r + 3.0 * l).abs() < 1e-3 * a);
        assert!(ripple_slope(a, l, big_r, big_r - 3.0 * l).abs() < 1e-3);
        // The steepest slope is about 2π A / λ, which bounds how far the normal tilts.
        let steepest = (0..2000)
            .map(|i| ripple_slope(a, l, big_r, 30.0 + i as f32 * 0.06).abs())
            .fold(0.0, f32::max);
        assert!(
            steepest <= 2.0 * PI * a / l && steepest > 0.8 * 2.0 * PI * a / l,
            "{steepest}"
        );
    }

    #[test]
    fn the_normal_turns_at_the_profile_curvature() {
        // A radius-50 sphere: the tilt is asin(1 - s / 50), so it turns at -1 / (50 cos θ).
        let p = Profile {
            thickness: 50.0,
            bevel: 50.0,
            base: 0.0,
            exponent: 2.0,
        };
        close(p.turn_rate(20.0), -1.0 / 40.0, 1e-4, "hemisphere, 20 in");
        close(p.turn_rate(50.0), 0.0, 1e-6, "pole and flat top");
        assert!(p.turn_rate(0.01) < -1.0, "unbounded at the rim");
        // Any profile: the analytic rate equals the finite difference of the normal's tilt.
        for (thickness, bevel, exponent) in [(20.0, 26.0, 2.0), (30.0, 46.0, 2.0), (12.0, 40.0, 3.0), (9.0, 9.0, 2.5)] {
            let p = Profile {
                thickness,
                bevel,
                base: 0.0,
                exponent,
            };
            for frac in [0.2_f32, 0.5, 0.8] {
                let s = bevel * frac;
                let tilt = |x: f32| {
                    let n = p.normal(x, vec2(1.0, 0.0));
                    n.x.atan2(n.z)
                };
                let numeric = (tilt(s + 0.01) - tilt(s - 0.01)) / 0.02;
                close(p.turn_rate(s), numeric, 2e-3 * numeric.abs().max(0.01), "turn rate");
            }
        }
    }

    #[test]
    fn the_footprint_filter_widens_and_clamps() {
        let zero = vec3(0.0, 0.0, 0.0);
        close(
            FOOTPRINT_SIGMA2,
            1.0 / (2.0 * PI),
            1e-7,
            "a Gaussian pixel filter of peak 1",
        );
        // A level surface keeps its roughness.
        close(footprint_roughness_sq(0.07, zero, zero), 0.0049, 1e-7, "flat");
        // The normal turns 0.1 rad across the pixel: α² + 2 σ² 0.01.
        close(
            footprint_roughness_sq(0.07, vec3(0.1, 0.0, 0.0), zero),
            0.0049 + 0.01 / PI,
            1e-6,
            "0.1 rad per pixel",
        );
        // A rim that turns a radian per pixel is clamped at κ, and nothing passes 1.
        close(
            footprint_roughness_sq(0.07, vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0)),
            0.0049 + 0.18,
            1e-6,
            "clamp",
        );
        close(
            footprint_roughness_sq(1.0, vec3(1.0, 0.0, 0.0), zero),
            1.0,
            1e-7,
            "never above 1",
        );
    }

    #[test]
    fn ggx_is_normalised() {
        for alpha in [0.05_f32, 0.1, 0.3, 0.8] {
            // ∫ D cosθ dω over the hemisphere = 1.
            let steps = 200_000;
            let mut sum = 0.0_f64;
            for i in 0..steps {
                let th = (i as f64 + 0.5) / steps as f64 * std::f64::consts::FRAC_PI_2;
                let d = f64::from(ggx_d(th.cos() as f32, alpha));
                sum += d * th.cos() * th.sin() * std::f64::consts::TAU * std::f64::consts::FRAC_PI_2 / steps as f64;
            }
            close(sum as f32, 1.0, 5e-3, "GGX normalisation");
            close(
                ggx_d(1.0, alpha),
                1.0 / (PI * alpha * alpha),
                1e-2 / (alpha * alpha),
                "peak",
            );
        }
        close(smith_lambda(1.0, 0.3), 0.0, 1e-6, "no masking at normal incidence");
        close(smith_g(1.0, 1.0, 0.3), 1.0, 1e-6, "G = 1 at normal incidence");
        assert!(
            smith_g(0.1, 0.1, 0.5) < smith_g(0.9, 0.9, 0.5),
            "grazing directions are masked more"
        );
    }

    #[test]
    fn ggx_lobe_never_reflects_more_than_arrives() {
        // Directional albedo with F = 1: ∫ f cosθ_l dω_l <= 1 for every view angle.
        for alpha in [0.1_f32, 0.4] {
            for view_deg in [0.0_f32, 40.0, 75.0] {
                let v = vec3(view_deg.to_radians().sin(), 0.0, view_deg.to_radians().cos());
                let (nt, np) = (400, 400);
                let mut sum = 0.0_f64;
                for it in 0..nt {
                    let th = (it as f32 + 0.5) / nt as f32 * PI * 0.5;
                    for ip in 0..np {
                        let ph = (ip as f32 + 0.5) / np as f32 * 2.0 * PI;
                        let l = vec3(th.sin() * ph.cos(), th.sin() * ph.sin(), th.cos());
                        let h = (l + v).normalize_or_zero();
                        let f = ggx_specular(h.z, l.z, v.z, alpha);
                        sum += f64::from(f * l.z * th.sin()) * f64::from(PI * 0.5 / nt as f32 * 2.0 * PI / np as f32);
                    }
                }
                assert!(sum <= 1.01, "alpha {alpha}, view {view_deg}°: albedo {sum}");
                assert!(
                    sum > 0.3,
                    "alpha {alpha}, view {view_deg}°: albedo {sum} is suspiciously low"
                );
            }
        }
    }

    #[test]
    fn colour_matching_fit_and_matrix() {
        close(cie_xyz(555.0)[1], 1.0, 0.01, "photopic peak");
        let mut sums = [0.0_f32; 3];
        for nm in 360..=830 {
            let v = cie_xyz(nm as f32);
            for c in 0..3 {
                sums[c] += v[c];
            }
        }
        // Equal-energy white has chromaticity (1/3, 1/3): the three integrals are equal.
        close(sums[0] / sums[1], 1.0, 0.005, "X / Y of equal-energy white");
        close(sums[2] / sums[1], 1.0, 0.005, "Z / Y of equal-energy white");
        let white = xyz_to_linear_srgb([0.9505, 1.0, 1.0890]);
        for (c, v) in white.iter().enumerate() {
            close(*v, 1.0, 2e-3, &format!("D65 white, channel {c}"));
        }
        let flat = spectrum_to_linear_srgb(|_| 1.0);
        for v in flat {
            close(v, 1.0, 2e-3, "a flat spectrum is display white");
        }
    }

    #[test]
    fn channel_wavelengths_are_the_srgb_dominant_wavelengths() {
        // The monochromatic light whose chromaticity lies on the line from D65 through each
        // primary. Searched inside the band of each primary.
        let white = (0.3127_f32, 0.3290_f32);
        let primaries = [(0.64_f32, 0.33_f32), (0.30, 0.60), (0.15, 0.06)];
        let bands = [(580.0_f32, 650.0_f32), (500.0, 570.0), (440.0, 490.0)];
        for c in 0..3 {
            let side = |nm: f32| {
                let [x, y, z] = cie_xyz(nm);
                let (cx, cy) = (x / (x + y + z), y / (x + y + z));
                (primaries[c].0 - white.0) * (cy - white.1) - (primaries[c].1 - white.1) * (cx - white.0)
            };
            let mut nm = bands[c].0;
            let mut found = None;
            while nm < bands[c].1 {
                if side(nm) * side(nm + 0.1) <= 0.0 {
                    found = Some(nm);
                    break;
                }
                nm += 0.1;
            }
            let dominant = found.expect("the primary's hue line crosses the spectral locus");
            close(
                dominant,
                PRIMARY_NM[c] as f32,
                2.0,
                &format!("dominant wavelength of primary {c}"),
            );
        }
    }

    #[test]
    fn thin_film_known_values() {
        let (air, mgf2, crown) = (1.0_f32, 1.38_f32, 1.52_f32);
        let quarter = |n1: f32| 550.0 / (4.0 * n1);
        // No film: the bare interface (about 4% for crown glass).
        close(
            thin_film_reflectance(air, mgf2, crown, 0.0, 550.0, 1.0),
            normal_reflectance(crown),
            1e-6,
            "zero thickness",
        );
        // The ideal quarter-wave film has index √(n0 n2) and reflects nothing at its design wavelength.
        let ideal = (air * crown).sqrt();
        close(
            thin_film_reflectance(air, ideal, crown, quarter(ideal), 550.0, 1.0),
            0.0,
            1e-6,
            "ideal quarter-wave film",
        );
        // MgF2 on crown: ((n0 n2 − n1²) / (n0 n2 + n1²))² = 1.26%, the "about 1%" of the literature.
        close(
            thin_film_reflectance(air, mgf2, crown, quarter(mgf2), 550.0, 1.0),
            0.0126,
            1e-4,
            "MgF2 quarter-wave on crown",
        );
        // A free-standing slab in air against the transfer-matrix closed form.
        let (n, d, nm) = (1.5_f32, 430.0_f32, 610.0_f32);
        let kd = 2.0 * PI * n * d / nm;
        let want = (n - 1.0 / n).powi(2) * kd.sin().powi(2)
            / ((n + 1.0 / n).powi(2) * kd.sin().powi(2) + 4.0 * kd.cos().powi(2));
        close(
            thin_film_reflectance(1.0, n, 1.0, d, nm, 1.0),
            want,
            1e-5,
            "slab in air",
        );
    }

    #[test]
    fn a_quarter_wave_coating_reflects_purple_not_a_guessed_hex() {
        // Single-layer MgF2 on sapphire, tuned to 550 nm: green is suppressed most, so the
        // residual reflection is magenta to purple. This replaces the guessed #7B6CFF.
        let d = 550.0 / (4.0 * 1.38);
        let [r, g, b] = spectrum_to_linear_srgb(|nm| {
            thin_film_reflectance(1.0, 1.38, Dispersion::SAPPHIRE.index(f64::from(nm)) as f32, d, nm, 1.0)
        });
        assert!(g < r && g < b, "green is the minimum: {r} {g} {b}");
        let bare = normal_reflectance(Dispersion::SAPPHIRE.n_d() as f32);
        assert!(
            r < bare && g < bare && b < bare,
            "the coating lowers every channel below {bare}"
        );
    }

    #[test]
    fn prism_minimum_deviation() {
        // Symmetric passage: n = sin((A + δ) / 2) / sin(A / 2). A = 60°, n = 1.5 gives δ = 37.18°.
        let apex = 60.0_f32.to_radians();
        let incidence = (1.5 * (apex * 0.5).sin()).asin();
        let dev = prism_deviation(1.5, apex, incidence).expect("transmits");
        close(dev.to_degrees(), 37.18, 0.01, "minimum deviation");
        for other in [40.0_f32, 60.0] {
            let d = prism_deviation(1.5, apex, other.to_radians()).expect("transmits");
            assert!(
                d > dev,
                "any other incidence deviates more: {other}° gives {}",
                d.to_degrees()
            );
        }
        // Blue is turned more than red.
        let [r, _, b] = Dispersion::CROWN_BK7.primaries();
        let dr = prism_deviation(r, apex, incidence).expect("red");
        let db = prism_deviation(b, apex, incidence).expect("blue");
        assert!(db > dr, "blue {db} > red {dr}");
        assert!(
            prism_deviation(1.5, 100.0_f32.to_radians(), 0.0).is_none(),
            "total internal reflection"
        );
    }

    #[test]
    fn hemisphere_profile() {
        let p = Profile {
            thickness: 50.0,
            bevel: 50.0,
            base: 0.0,
            exponent: 2.0,
        };
        close(p.height(0.0), 0.0, 1e-4, "rim");
        close(p.height(50.0), 50.0, 1e-4, "pole");
        close(p.height(80.0), 50.0, 1e-4, "flat beyond the bevel");
        // 20 in from the rim of a radius-50 sphere: r = 30, so h = 40 and the normal is (0.6, 0, 0.8).
        close(p.height(20.0), 40.0, 1e-3, "height on the sphere");
        let n = p.normal(20.0, vec2(1.0, 0.0));
        close((n - vec3(0.6, 0.0, 0.8)).length(), 0.0, 1e-4, "sphere normal");
        let rim = p.normal(0.0, vec2(0.0, -1.0));
        close(
            (rim - vec3(0.0, -1.0, 0.0)).length(),
            0.0,
            1e-4,
            "the rim normal is horizontal",
        );
        close(
            p.normal(60.0, vec2(1.0, 0.0)).z,
            1.0,
            0.0,
            "the flat top faces the viewer",
        );
    }

    #[test]
    fn a_hemisphere_on_a_page_magnifies_by_its_index() {
        // A classic check: a hemispherical contact magnifier of index n magnifies by n near
        // its centre.
        let radius = 200.0_f32;
        let p = Profile {
            thickness: radius,
            bevel: radius,
            base: 0.0,
            exponent: 2.0,
        };
        let eta = Dispersion::SAPPHIRE.n_d() as f32;
        let r = 4.0_f32; // 4 px from the centre, looking along +x from the centre
        let s = view_through(&p, radius - r, vec2(1.0, 0.0), eta);
        let seen = r + s.offset.x;
        close(r / seen, eta, 0.02, "paraxial magnification");
        close(s.reflectance, normal_reflectance(eta), 1e-3, "near-normal reflectance");
        close(s.path, radius, 0.5, "path through the pole");
    }

    #[test]
    fn the_flat_top_shows_what_is_under_it_unbent() {
        let p = Profile {
            thickness: 12.0,
            bevel: 16.0,
            base: 6.0,
            exponent: 2.0,
        };
        let eta = Dispersion::SAPPHIRE.n_d() as f32;
        let top = view_through(&p, 40.0, vec2(1.0, 0.0), eta);
        close(top.offset.length(), 0.0, 0.0, "no displacement");
        close(top.path, 18.0, 1e-5, "thickness + base");
        close(top.reflectance, 0.0770, 2e-4, "R0");
        // On the bevel the ray bends inwards (against the outward normal) and the path is longer than the depth.
        let bevel = view_through(&p, 4.0, vec2(1.0, 0.0), eta);
        assert!(bevel.offset.x < 0.0, "bends towards the inside: {:?}", bevel.offset);
        assert!(bevel.path > p.height(4.0) + p.base, "oblique path is longer");
        assert!(bevel.reflectance > top.reflectance, "the bevel reflects more");
        // At the very rim the ray leaves the normal at tan = √(η² − 1): the offset is base × that.
        let rim = view_through(&p, 0.0, vec2(1.0, 0.0), eta);
        close(-rim.offset.x, 6.0 * (eta * eta - 1.0).sqrt(), 0.05, "rim displacement");
    }

    #[test]
    fn doppler_beaming_known_values() {
        close(doppler_factor(0.0, 1.0), 1.0, 1e-6, "at rest");
        close(doppler_factor(0.5, 1.0), 3.0_f32.sqrt(), 1e-5, "approaching at 0.5 c");
        close(
            doppler_factor(0.5, -1.0),
            1.0 / 3.0_f32.sqrt(),
            1e-5,
            "receding at 0.5 c",
        );
        close(doppler_factor(0.5, 0.0), 0.75_f32.sqrt(), 1e-5, "transverse: 1 / gamma");
        // A flat spectrum (alpha = 0) is boosted by D³: 27 times brighter approaching than receding at 0.5 c.
        let ratio = beaming_gain(doppler_factor(0.5, 1.0), 0.0) / beaming_gain(doppler_factor(0.5, -1.0), 0.0);
        close(ratio, 27.0, 1e-2, "approaching over receding");
    }

    #[test]
    fn plate_modes() {
        let f = |m, n| plate_mode_frequency(m, n, 1.0, 1.0, 1.0);
        close(f(2, 1) / f(1, 1), 2.5, 1e-5, "(2,1) over (1,1) on a square plate");
        close(f(2, 2) / f(1, 1), 4.0, 1e-5, "(2,2) over (1,1)");
        for (x, y) in [(0.1_f32, 0.7_f32), (0.33, 0.9), (0.5, 0.2)] {
            close(chladni_square(x, y, 3, 3, 1.0), 0.0, 1e-6, "m = n gives no figure");
            close(
                chladni_square(x, y, 1, 4, 1.0),
                -chladni_square(y, x, 1, 4, 1.0),
                1e-5,
                "antisymmetric about the diagonal",
            );
        }
        close(
            chladni_square(0.3, 0.3, 2, 5, 1.0),
            0.0,
            1e-6,
            "the diagonal is a nodal line",
        );
    }
}
