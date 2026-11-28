# Optics: the equations behind glass, light and spectra

**Status:** in force from 2026-11-27. It replaces card P8-1 of [phase-8-materials-and-shaders.md](phase-8-materials-and-shaders.md) and corrects the optics in card P8-2 and in `docs/design/RESEARCH.md` (see [§11](#11-what-this-corrects)).

**Rule:** every glass, reflective or spectral effect in a cluster, the OSD or the scrim uses the equations here. A cluster that needs an effect this document does not have adds it here first, with its source and a unit test, and only then uses it.

**Where the code is:**
- `crates/fh6-kit/src/optics.rs` is the CPU reference: one pure function per equation, each with a unit test against a known value ([§13](#13-tests)).
- `crates/fh6-kit/src/shaders/shape.wgsl` (`optic_glass`) evaluates the same equations per pixel, for glass and for metal (`OpticalGlass::metal`).
- `crates/fh6-kit/tests/glass_gpu.rs` compares rendered pixels with the CPU reference and runs the conservation checks on the GPU.

---

## 0. Conventions

- The screen is the xy plane, y down. z points at the viewer. Lengths are local units (4K pixels at user scale 1).
- The viewer is orthographic: every view ray runs along −z, so the view vector is $\mathbf v = (0, 0, 1)$ everywhere.
- Colour is linear light. A formula with a subscript $c$ is evaluated once per channel, R, G and B.
- $\eta$ is the index of the medium a ray enters divided by the index of the medium it leaves. Air is 1 (its real index, 1.0003, is ignored).

## 1. Materials

Each material has one dispersion formula. Its index at any wavelength, its Abbe number and its normal reflectance all follow from that formula; none is typed in separately.

Sellmeier form, $\lambda$ in µm:

$$n^2(\lambda) - 1 = \sum_{i=1}^{3} \frac{B_i \lambda^2}{\lambda^2 - C_i}$$

| Material (`Dispersion::…`) | $B_1, B_2, B_3$ | $C_1, C_2, C_3$ (µm²) | $n_d$ | $V_d$ | $R_0$ | Source |
|---|---|---|---|---|---|---|
| Synthetic sapphire, ordinary ray (`SAPPHIRE`) | 1.4313493, 0.65054713, 5.3414021 | 0.0726631², 0.1193242², 18.028251² | 1.7682 | 72.3 | 0.0770 | [S1] |
| Schott N-BK7 crown glass (`CROWN_BK7`) | 1.03961212, 0.231792344, 1.01046945 | 0.00600069867, 0.0200179144, 103.560653 | 1.5168 | 64.17 | 0.0422 | [S2] |
| Fused silica (`FUSED_SILICA`) | 0.6961663, 0.4079426, 0.8974794 | 0.0684043², 0.1162414², 9.896161² | 1.4585 | 67.8 | 0.0348 | [S3] |

- $n_d$ is the index at the helium d line, 587.56 nm.
- The Abbe number is $V_d = (n_d - 1)/(n_F - n_C)$, with F = 486.13 nm and C = 656.27 nm [S4].
- $R_0 = \left(\frac{n - 1}{n + 1}\right)^2$ is the reflectance at normal incidence ([§3](#3-fresnel-reflectance)).
- Sapphire is birefringent by 0.008 [S5]. The kit uses the ordinary ray only.
- **The Liquid Glass family is sapphire.** The other two are available to any cluster.

A material known only by $n_d$ and $V_d$ uses the two-term Cauchy form $n(\lambda) = A + B/\lambda^2$ [S6], fitted through both numbers (`Dispersion::from_abbe`):

$$B = \frac{n_d - 1}{V_d\,(\lambda_F^{-2} - \lambda_C^{-2})}, \qquad A = n_d - \frac{B}{\lambda_d^2}$$

Cauchy holds for normal dispersion in the visible range only [S6]. For N-BK7 it stays within 0.001 of the Sellmeier index from 440 to 660 nm (tested).

**Borosilicate 3.3** (the tubes and the ampoule of `KineticLiquidMetal`, a lab instrument's glass) is known by its two catalogue lines only, so it uses the Cauchy fit above through `Dispersion::from_abbe(1.4714, 66.0)`. SCHOTT gives BOROFLOAT 33 an index $n_d$ = 1.47140 and a principal dispersion $n_F - n_C$ = 71.4 × 10⁻⁴, so $V_d$ = 0.47140 / 0.00714 = 66.0, and a density of 2.23 g cm⁻³ [S29]. Its $R_0$ is 3.6%.

**Water** (the rippled surface that the liquid-light caustic of `tools/asset-forge` is traced through) is used at one index only, $n = 1.333$: the tabulated value at 550 and 575 nm for liquid water at 25 °C [S23]. The same table gives 1.339 at 400 nm and 1.331 at 700 nm; [§5](#5-dispersion) says why one index is enough there.

## 2. The surface comes from the outline

A glass piece is described by its outline, a signed distance field $d(x, y)$ that is negative inside. Its top surface is a height above the rim, a function of the distance inwards $s = -d$:

$$u = \mathrm{clamp}(s / w,\ 0,\ 1), \qquad h(s) = H\,\bigl(1 - (1 - u)^k\bigr)^{1/k}$$

| Symbol | `Profile` field | Meaning |
|---|---|---|
| $H$ | `thickness` | Rise of the top surface above the rim |
| $w$ | `bevel` | Width of the rounded edge, measured inwards from the outline |
| $b$ | `base` | Glass below the rim level, down to the plane the piece sits on |
| $k$ | `exponent` | Shape of the edge: 1 is a chamfer, 2 a circular arc, larger a squarer shoulder |

With $k = 2$ and $H = w = R$ on a circle of radius $R$, the piece is a hemisphere (tested). With $w$ smaller than the piece, it has a flat top with a rounded edge. The edge is a quadrant of a superellipse, the profile family of superquadrics [S24].

The unit surface normal follows from the slope of $h$ and the gradient of the outline:

$$\mathbf n \propto \Bigl(H (1 - u)^{k-1}\,\nabla d,\ \ w\,\bigl(1 - (1 - u)^k\bigr)^{1 - 1/k}\Bigr)$$

It is horizontal at the rim, vertical on the flat top, and turns continuously between. Nothing else in this document paints an edge: every edge effect comes from this normal.

**Liquid merging.** Pieces in one group share one outline, the smooth union of their distances (the exponential smooth minimum of distance fields [S25]):

$$\mathrm{smin}_k(a, b) = \min(a, b) - k \ln\bigl(1 + e^{-|a - b|/k}\bigr) = -k \ln\bigl(e^{-a/k} + e^{-b/k}\bigr)$$

It is associative and commutative, so the result does not depend on the order of the pieces. It is smooth to every order, so normals and highlights have no crease where two pieces join. It never differs from the plain union by more than $k \ln N$ for $N$ pieces. $k$ is a length: pieces closer than a few $k$ start to bridge.

Two outlines a gap $g$ apart first touch when $\mathrm{smin}_k$ reaches zero midway between them, at $k = g / (2 \ln 2) = 0.72\,g$. So raising $k$ makes two pieces reach for each other and fuse through a neck, with neither piece moving.

**Surface waves.** A liquid surface carries waves. A ripple is a ring packet added to the height, at distance $r$ from where it started:

$$\eta(r) = A\,e^{-x^2}\cos(2\pi x), \qquad x = \frac{r - R}{\lambda}, \qquad \frac{d\eta}{dr} = \frac{A\,e^{-x^2}}{\lambda}\,\bigl(-2x\cos 2\pi x - 2\pi \sin 2\pi x\bigr)$$

with amplitude $A$, wavelength $\lambda$ and ring radius $R$ (animated outwards). The packet is a Gaussian-windowed cosine, a Gabor function [S26]. It is a shape chosen to look like the ring of waves that a drop sends across a liquid surface (the Cauchy–Poisson problem [S27]), not a solution of the capillary-wave equation, so its height and wavelength are design values held to the slope limit below. The surface is $h + \eta$, so the normal becomes $\mathbf n \propto (-\nabla h - \nabla\eta,\ 1)$ and everything after it (reflection, refraction, the highlight) follows without a special case. The steepest slope is about $2\pi A/\lambda$. Through a depth $D = H + b$ of glass a slope $\theta$ moves what lies under it by $D\tan(\theta - \theta_t)$ with $\sin\theta_t = \sin\theta / n$, so a wave over a numeral is kept under 5° (tested on the CPU and against the shader).

**Standing waves.** A liquid shaken up and down at frequency $f$ breaks into Faraday waves: a standing pattern at $f/2$ whose surface is a sum of plane waves sharing one wavenumber [S33]. The kit adds that sum to the height (`StandingWaves`, beside the ripple), with up to six waves:

$$\zeta(\mathbf p) = \sum_{i=1}^{N \le 6} A_i \cos\bigl(k\,\mathbf e_i \cdot (\mathbf p - \mathbf c) + \varphi_i\bigr), \qquad \nabla\zeta = -\sum_i k A_i\,\mathbf e_i \sin\bigl(k\,\mathbf e_i \cdot (\mathbf p - \mathbf c) + \varphi_i\bigr)$$

with unit directions $\mathbf e_i$, amplitudes $A_i$, phases $\varphi_i$, one wavenumber $k = 2\pi/\lambda$ and a centre $\mathbf c$. The gradient is exact, one sine and cosine per wave per pixel, so no texture or flipbook is involved and $k$, $A_i$ and $\varphi_i$ may change every frame. Two waves 90° apart give squares, three 120° apart hexagons, and six 30° apart a twelve-fold quasi-pattern: the patterns Faraday waves select [S33]. Each is unchanged by a turn of 90°, 60° or 30° about $\mathbf c$ (tested). A post pinned at the centre holds the surface still around it, so the height is eased in from a still radius $R_s$ over a band $W$:

$$\eta = w(r)\,\zeta, \qquad w = s^2(3 - 2s), \quad s = \mathrm{clamp}\Bigl(\frac{r - R_s}{W}, 0, 1\Bigr), \qquad \nabla\eta = w\,\nabla\zeta + \zeta\,w'(r)\,\hat{\mathbf r}$$

$w$ is $C^1$, so neither the height nor the normal has a crease at the band's edges. The normal follows as for the ripple, $\mathbf n \propto (-\nabla h - \nabla\eta,\ 1)$. The waves also turn the normal inside a pixel, by the second derivatives $-k^2 \sum_i A_i \cos(\ldots)\,\mathbf e_i \mathbf e_i^{\mathsf T}$ (the ease is taken as constant across a pixel); the shader adds that change to the pixel footprint of [§7](#7-specular-light), so a short wavelength on a mirror widens its highlights instead of breaking them into flickering dots. The amplitude, like the ripple's, scales with materialise. Its steepest slope is at most $k \sum_i |A_i|$ outside the still band and $\sum_i |A_i|\,(k + 1.5/W)$ inside it ($w'$ peaks at $1.5/W$); over a numeral seen through glass, the same 5° limit as the ripple applies. The shader takes $|k| \le 64$ and $|A_i| \le 10^4$, so the second derivatives stay finite.

**Materialise.** A piece appears by scaling $H$ and $b$ from 0 to their values. Lensing and highlights grow in; opacity is never ramped.

## 3. Fresnel reflectance

How much of the light a surface reflects, for unpolarised light meeting a dielectric at angle $\theta_i$ to the normal [S7]:

$$\cos\theta_t = \sqrt{1 - \frac{\sin^2\theta_i}{\eta^2}}, \quad r_\parallel = \frac{\eta\cos\theta_i - \cos\theta_t}{\eta\cos\theta_i + \cos\theta_t}, \quad r_\perp = \frac{\cos\theta_i - \eta\cos\theta_t}{\cos\theta_i + \eta\cos\theta_t}, \quad F = \tfrac12\bigl(r_\parallel^2 + r_\perp^2\bigr)$$

- For the view ray, $\cos\theta_i = \mathbf n \cdot \mathbf v = n_z$, so $F$ is evaluated per pixel from the normal of [§2](#2-the-surface-comes-from-the-outline).
- When the square root's argument is negative the ray is totally reflected and $F = 1$.
- Sapphire reflects 7.7% face-on, the same at 45° to within a point, and everything at grazing incidence. That rise is what draws the rim of a piece.

**Schlick's approximation** [S8], $F \approx R_0 + (1 - R_0)(1 - \cos\theta_i)^5$, is allowed only where the exact form is too costly. For the three materials above it is within 0.04 of the exact value, worst near 85° (tested). The glass shader uses the exact form.

### Conductors: mercury

A metal has a complex index $\tilde n = n + ik$, the square root of its permittivity $\varepsilon = \varepsilon_1 + i\varepsilon_2$. With $|\varepsilon| = \sqrt{\varepsilon_1^2 + \varepsilon_2^2}$ [S7]:

$$n = \sqrt{\tfrac12\bigl(|\varepsilon| + \varepsilon_1\bigr)}, \qquad k = \sqrt{\tfrac12\bigl(|\varepsilon| - \varepsilon_1\bigr)}$$

The Fresnel equations of the dielectric case hold with $\eta$ replaced by $\tilde n$ (complex arithmetic, unpolarised average of $|r_\parallel|^2$ and $|r_\perp|^2$); face-on they reduce to

$$R_0 = \frac{(n - 1)^2 + k^2}{(n + 1)^2 + k^2}$$

- Liquid mercury at 2.27 eV (546 nm, the green mercury line) has $\varepsilon = -18.46 + 13.50\,i$ [S30], so $n$ = 1.485, $k$ = 4.546 and $R_0$ = 0.779. The same paper's Drude fit gives 0.765 ± 0.008 there.
- A conductor's reflectance dips only a little near its principal angle (no Brewster zero; mercury stays above 0.7) and reaches 1 at grazing incidence. A liquid-metal dash weights the studio its mercury mirrors by this curve (78% head-on, 100% at the edge), so the metal is a mirror of a measured strength, never a flat grey.
- **Per channel.** Like a dielectric ([§5](#5-dispersion)), a metal is evaluated at the three channel wavelengths, 611, 549 and 464 nm, with $n$ and $k$ interpolated linearly in measured tables: liquid mercury [S31] and chromium [S32] (`optics::Metal`). Mercury there is $(n, k)$ = (1.86, 5.09), (1.55, 4.64), (1.13, 3.98) and reflects 0.781, 0.779 and 0.779 face-on: a neutral white, because $n$ and $k$ rise together with wavelength. Chromium reflects 0.55 to 0.56.
- **The shader's form.** The shader evaluates the same reflectance without complex numbers [S7]: with $t = n^2 - k^2 - \sin^2\theta$, $a^2 + b^2 = \sqrt{t^2 + 4n^2k^2}$ and $a = \sqrt{(a^2 + b^2 + t)/2}$,

$$R_s = \frac{a^2 + b^2 - 2a\cos\theta + \cos^2\theta}{a^2 + b^2 + 2a\cos\theta + \cos^2\theta}, \qquad R_p = R_s\,\frac{(a^2 + b^2)\cos^2\theta - 2a\cos\theta\sin^2\theta + \sin^4\theta}{(a^2 + b^2)\cos^2\theta + 2a\cos\theta\sin^2\theta + \sin^4\theta}, \qquad F = \tfrac12(R_s + R_p)$$

  It agrees with the complex form within 1e-4 at every angle for $n$ from 0.05 to 3.2 (tested).
- **What a metal pixel shows.** A metal transmits nothing; what it does not reflect it absorbs within nanometres. So [§8](#8-what-one-pixel-of-glass-shows) loses its transmitted term, $L_c = F_c\,E_c(\mathbf r) + S_c$, with the conductor's $F$ in the key light's lobe too, and the piece is opaque ($a = 1$): it shows black where it mirrors black, so off pixels stay off. Refraction, absorption and frost do not apply.
- Code and tests: `crates/fh6-kit/src/optics.rs` (`Conductor`, `Metal`, `index_from_permittivity`, `fresnel_conductor`, `fresnel_conductor_real`); `OpticalGlass::metal` draws it.

## 4. Refraction through a piece

Snell's law, $\sin\theta_i = \eta \sin\theta_t$, in vector form [S7]. With $\mathbf v$ pointing away from the surface:

$$\boldsymbol\omega_t = -\frac{\mathbf v}{\eta} + \Bigl(\frac{\cos\theta_i}{\eta} - \cos\theta_t\Bigr)\mathbf n$$

**The contact model.** A piece sits in optical contact with the plane that carries what it shows (its instrument light, the layers drawn before it, the captured game frame). The plane is $b$ below the rim. The refracted view ray starts on the top surface at height $h$ and meets the plane after travelling

$$\ell = \frac{h + b}{-\omega_{t,z}}, \qquad \boldsymbol\Delta = \ell\,(\omega_{t,x},\ \omega_{t,y})$$

The pixel shows the plane at $\mathbf p + \boldsymbol\Delta$. $\ell$ is the path length that absorption uses ([§6](#6-absorption)).

What follows from this, with no further tuning:
- Under the flat top the ray is not bent: $\boldsymbol\Delta = 0$. Numerals placed there stay undistorted (tested).
- On the bevel the ray bends inwards, so the edge shows content from further inside the piece. At the rim the displacement is $b\sqrt{\eta^2 - 1}$: 1.46 $b$ for sapphire (tested).
- A hemisphere magnifies what is under its centre by its index, 1.77× for sapphire (tested). This is the known behaviour of a hemispherical contact magnifier.
- Radiance is not scaled by the magnification. Along a refracted path $L / n^2$ is conserved [S9], and the content plane is inside the glass on both ends of the comparison, so a magnified numeral is larger, not brighter.

## 5. Dispersion

Refraction is evaluated once per channel, each with its own index:

$$\eta_c = n(\lambda_c), \qquad \lambda_R = 611\ \text{nm}, \quad \lambda_G = 549\ \text{nm}, \quad \lambda_B = 464\ \text{nm}$$

These are the dominant wavelengths of the sRGB primaries (derived in a test from [S10] and [S11]). The light being bent comes from a display, where each channel is a narrow band around its primary, so one index per channel is the dispersion of an RGB image; no fringe colours are painted.

- For sapphire $\eta_R, \eta_G, \eta_B$ = 1.7669, 1.7705, 1.7778, a spread of 0.011.
- **Real dispersion is small.** Through a 40 px deep sapphire edge the blue and red images separate by well under a pixel. It becomes visible only through thick pieces and prisms. The earlier fixed 1.5 px red and blue split was not physical.
- Fresnel uses the same per-channel index, so reflections are about 2% stronger in blue than in red.

**The liquid-light caustic is traced at one wavelength.** Its tile is a single channel that the dash tints with the palette, so a spectral trace would have no colour to keep. Water's index falls by 0.008 from 400 to 700 nm [S23], which moves each caustic line by $\Delta n / (n - 1) = 2.4\%$ of its offset from the undeflected ray: at most about 1.5 px for a 64 px cell on screen, a softening and never a fringe. The floor under the water sits at the ripples' mean focal depth, $f = n / \bigl((n - 1)\,\kappa\bigr)$ for a surface of curvature $\kappa$: the paraxial focus of one refracting surface [S28].

**Broadband light** (the coating colour in [§9](#9-thin-film-coating)) is integrated over wavelength and converted to linear sRGB:

$$(X, Y, Z) = \int L(\lambda)\,(\bar x, \bar y, \bar z)(\lambda)\,d\lambda, \qquad \begin{pmatrix} R \\ G \\ B \end{pmatrix} = \begin{pmatrix} 3.2406 & -1.5372 & -0.4986 \\ -0.9689 & 1.8758 & 0.0415 \\ 0.0557 & -0.2040 & 1.0570 \end{pmatrix} \begin{pmatrix} X \\ Y \\ Z \end{pmatrix}$$

The colour matching functions are the multi-lobe fit of [S10]; the matrix is IEC 61966-2-1 [S11]. `spectrum_to_linear_srgb` balances the result so a flat spectrum is display white.

## 6. Absorption

Glass darkens and colours light in proportion to the distance travelled in it, the Beer-Lambert law [S12]:

$$T_c(\ell) = e^{-\sigma_c\,\ell}$$

A tint is given as the internal transmittance $T_{\mathrm{ref},c}$ at a reference depth $d_{\mathrm{ref}}$, which sets $\sigma_c = -\ln T_{\mathrm{ref},c} / d_{\mathrm{ref}}$. So a piece twice as thick transmits the square (tested), an oblique ray through the bevel is tinted more than a ray through the flat top, and a thin piece is nearly clear. Tint is never a flat alpha.

## 7. Specular light

**Environment.** The surface mirrors its surroundings along

$$\mathbf r = -\mathbf v + 2\,(\mathbf n \cdot \mathbf v)\,\mathbf n$$

and reflects the environment radiance $E_c(\mathbf r)$ weighted by $F_c$. The environment is a studio map when present and an analytic studio otherwise. It turns about the view axis with the virtual light, so every reflection moves together.

**The studio.** A surface whose normal is tilted by $\theta$ from the view axis mirrors the direction at elevation $90° - 2\theta$ above the plane of the screen. Clear glass on a dark ground shows its shape only where its edges mirror a light, which studio photographers call dark-field lighting: light lines against a dark background [S21]. The analytic studio is built for that. The rendered studio map (the Liquid Glass recipe in `tools/asset-forge`) has the same lights, plus a cyclorama that carries the ComfyUI studio panorama at radiance 0.3 (`assets/clusters/liquid_glass/ASSETS.md`):

| Light | Where (azimuth clockwise from 3 o'clock, elevation above the screen) | Radiance | What it draws on a rounded edge |
|---|---|---|---|
| Horizon ring | all the way round, elevation 0° to 9° | 0.9, plus 5.0 $\cos^6$ towards azimuth −135° and (1.98, 2.2, 2.46) $\cos^6$ towards 45° | one thin line along the whole outline, where $\theta$ is between 40.5° and 45° |
| Key strip | azimuth −135°, elevation 40°, 74° × 9° | 8 | a second, shorter line further in ($\theta \approx 25°$), up and to the left |
| Far strip | azimuth 45°, elevation 25°, 105° × 7° | (4.2, 4.7, 5.2) | a thin cool line down and to the right ($\theta \approx 32°$) |
| Sky | above, soft | up to 0.1 | a faint lift on edges that face up |

A face tilted $\theta$ mirrors the direction $2\theta$ off the view axis. The rendered map is black within 44° of the axis (fading in by 45°, short of the key's nearest edge at 45.4°), so every face tilted less than 22° mirrors black and the numerals under it keep their contrast; the analytic studio has only its faint sky there (radiance at most 0.1). Every light is a long strip: a short card shows as a card wherever a ripple holds a rim's tilt still for a few frames.

**The plane under the piece.** A mirrored ray with $r_z < 0$ meets the screen itself, a distance $(h + b)\,\lVert\mathbf r_{xy}\rVert / (-r_z)$ away, and the surface mirrors what is drawn there. The studio behind the plane of the screen is never seen.

**Pixel footprint.** A rounded edge turns through many degrees inside a few pixels, so a pixel mirrors a patch of the environment, not a point. Along the slope the normal's tilt is $\theta = \arctan(G/U)$ with $G = \lVert\nabla h^k\rVert$ and $U = k\,h^{k-1}$ (§2), which turns at

$$\frac{d\theta}{ds} = \frac{G'\,U - G\,U'}{G^2 + U^2}, \qquad G' = -\frac{H^k\,k\,(k-1)\,(1-u)^{k-2}}{w^2}, \qquad U' = \frac{(k-1)\,G}{h}$$

(for a hemisphere of radius $R$ this is $-1/(R\cos\theta)$; tested). Across the slope it turns by $\kappa \sin\theta$, where $\kappa = \nabla^2 d$ is the curvature of the outline's level line. Those two changes of $\mathbf n$ per pixel give the change of $\mathbf r$, and the environment is filtered over that patch: the map is sampled anisotropically with those two gradients, and the edges of the analytic lights are softened by half the patch width. The key light's lobe is filtered the same way, by widening its roughness with the spread of normals inside the pixel [S22]:

$$\alpha'^2 = \alpha^2 + \min\!\Bigl(2\sigma^2\bigl(\lVert\partial_x \mathbf n\rVert^2 + \lVert\partial_y \mathbf n\rVert^2\bigr),\ \kappa_c\Bigr), \qquad \sigma^2 = \frac{1}{2\pi}, \quad \kappa_c = 0.18$$

$\sigma^2$ is the variance of a Gaussian pixel filter of peak 1, and the clamp bounds the error of the derivative estimate (tested). Without this a thin highlight on a tight edge breaks into flickering dots.

**Key light.** The one virtual light of a cluster is a directional light of irradiance $E_L$ along $\mathbf l$. Its highlight is a GGX microfacet lobe [S9][S13], with half vector $\mathbf h = (\mathbf l + \mathbf v)/\lVert \mathbf l + \mathbf v \rVert$ and roughness $\alpha$:

$$D(\mathbf h) = \frac{\alpha^2}{\pi\bigl(\cos^2\theta_h\,(\alpha^2 - 1) + 1\bigr)^2}, \qquad \Lambda(\theta) = \frac{\sqrt{1 + \alpha^2\tan^2\theta} - 1}{2}, \qquad G = \frac{1}{1 + \Lambda(\theta_l) + \Lambda(\theta_v)}$$

$$S_c = \frac{D(\mathbf h)\,G\,F_c(\mathbf v \cdot \mathbf h)}{4\,(\mathbf n \cdot \mathbf l)(\mathbf n \cdot \mathbf v)}\;E_{L,c}\,(\mathbf n \cdot \mathbf l)$$

$D$ integrates to 1 over projected solid angle, and with $F = 1$ the lobe never reflects more than arrives (both tested). The light's direction comes from the tokens (`light.azimuthDeg`, `light.elevationDeg`). The code (`common::VirtualLight`) sets the azimuth to $-135° + 25° \cdot \mathrm{clamp}(g_{lat} / 1.5,\ -1,\ 1)$, clockwise on the screen from 3 o'clock, so the light starts at the upper left and swings by up to 25° at 1.5 g of lateral acceleration. The whole rig also leans by `glass.optics.tiltDegPerG` (6°) per g of acceleration about the two screen axes (`OpticalGlass::aim`), so every piece's highlights travel together. A cluster may keep the environment level and lean only the key light (the Liquid Glass pilot does): the environment's dark cone then stays on the view axis at any G.

## 8. What one pixel of glass shows

$$L_c = \underbrace{F_c\,E_c(\mathbf r) + S_c}_{\text{reflected}} \;+\; \underbrace{(1 - F_c)\,T_c(\ell_c)\,B_c(\mathbf p + \boldsymbol\Delta_c)}_{\text{transmitted}}$$

$B$ is what lies under the piece: the kit's layers drawn before the glass, over the captured game frame when `GlassRefraction` is on.

**Energy conservation.** A fraction $F$ is reflected and $1 - F$ transmitted, and they sum to 1 at every angle (tested against the Fresnel transmission coefficients, which are computed separately). Absorption only removes light. So with a uniform environment and a uniform backdrop of the same radiance and no absorption, a piece of glass is invisible. The GPU test renders exactly that and requires every pixel to equal the backdrop.

**Display light.** The overlay is composited over the game in sRGB-encoded values, not in linear light: the kit's resolve writes colour up to the alpha as coverage, $E(c/a)\,a$, and light beyond it as emission ($E$ is the sRGB encoding). So the equation above is evaluated on displayed linear light. What lies under the piece is first converted to the light it will show, the physics is done in linear light, and the result is converted back to the scene value that shows it at the piece's alpha. Without that step a 7.7% reflection would be written as 7.7% of the encoded range and show as 0.7% (the GPU Fresnel test caught exactly this).

**Alpha.** Where there is no captured frame to bend, the game shows through unbent and the piece hides the fraction it reflects or absorbs. Because the compositor multiplies encoded values, the game behind is attenuated by the encoded value of the glass's linear transmission:

$$a = 1 - E\bigl((1 - \bar F)\,\bar T\bigr)\,(1 - a_B)$$

with $a_B$ the alpha of the kit's own layers under the piece and the bars the mean over the three channels. Glass never fills its footprint with an opaque colour, so off pixels stay off.

## 9. Thin-film coating

For a cluster that wants a coated crystal. The Liquid Glass family is uncoated, because a coating removes the reflections that make it read as glass.

A film of index $n_1$ and thickness $t$ on a substrate $n_2$, seen from a medium $n_0$. The two interface reflections interfere with phase $\beta$ [S14][S15]:

$$\beta = \frac{2\pi\,n_1\,t\,\cos\theta_1}{\lambda}, \qquad R = \frac{r_{01}^2 + r_{12}^2 + 2\,r_{01} r_{12} \cos 2\beta}{1 + r_{01}^2 r_{12}^2 + 2\,r_{01} r_{12} \cos 2\beta}$$

$r_{01}$ and $r_{12}$ are the Fresnel amplitude coefficients of the two interfaces, evaluated for each polarisation and averaged.

Known values (all tested):
- No film gives the bare interface, 4.3% for crown glass.
- A quarter-wave film, $t = \lambda_0 / (4 n_1)$, of the ideal index $n_1 = \sqrt{n_0 n_2}$ reflects nothing at $\lambda_0$ [S14].
- Magnesium fluoride ($n_1$ = 1.38) on crown glass reflects 1.26% at its design wavelength, the "about 1%" of [S14].
- A slab in air matches the transfer-matrix closed form [S15].

**The colour of a coated sapphire.** One quarter-wave layer of magnesium fluoride tuned to 550 nm suppresses green most and leaves 0.1% in green, 0.4% in red and 1.2% in blue. Under white light that is a faint violet, hue `#9E51FF`, about a tenth as bright as the uncoated reflection. This replaces the guessed rim tint `#7B6CFF`.

## 10. Prism

A prism of apex angle $A$ turns a ray that meets its first face at $\theta_1$ by

$$\delta = \theta_1 + \theta_2 - A, \qquad \sin\theta_1 = n \sin\theta_1', \quad \theta_2' = A - \theta_1', \quad \sin\theta_2 = n \sin\theta_2'$$

(Snell's law at both faces.) It is least for the symmetric passage, where $n = \sin\frac{A + \delta}{2} / \sin\frac{A}{2}$: 37.18° for $A$ = 60°, $n$ = 1.5 (tested). Because $n$ depends on wavelength, so does $\delta$; that difference is the spectrum a prism throws. A cluster that splits colour by yaw or slip turns the prism (changes $\theta_1$) and evaluates $\delta(\lambda_c)$; it does not offset the channels by a made-up amount.

## 11. What this corrects

| Topic | Where | Before | Now |
|---|---|---|---|
| Index of refraction | Card P8-1 §1; `Glass::fresnel` comment; `glass_delta` | "$n$ = 1.52", called sapphire | Sapphire is 1.768 ($R_0$ = 0.0770). 1.52 is a crown glass ($R_0$ = 0.0426). One formula per material ([§1](#1-materials)) |
| Fresnel | Card P8-1 §1; `RESEARCH.md` "fresnel (r/R)^4 to 40/255" | Schlick quoted, then drawn as fixed white-to-cyan gradient stops; a ramp over edge distance | Exact Fresnel per pixel from the surface normal ([§3](#3-fresnel-reflectance)) |
| Refraction | `RESEARCH.md` "refraction uv·(1 − 0.03r²)"; `Glass::lens` | A fixed radial warp from the shape's centre | Snell's law through the thickness profile ([§4](#4-refraction-through-a-piece)) |
| Dispersion | `families/liquid-glass.md` "1.5 px chromatic split"; card P8-2 §8 | A fixed pixel offset, or one proportional to yaw | One index per channel from the dispersion formula ([§5](#5-dispersion)); prisms by [§10](#10-prism) |
| Tint | Tokens `glass.tintAlpha` 28 / 44 / 64 | A flat alpha | Beer-Lambert over the path length ([§6](#6-absorption)) |
| Specular | Card P8-1 §2 crown sheen; rim gradient | Painted gradients, not tied to the light | Environment reflection and a GGX lobe from the virtual light ([§7](#7-specular-light)) |
| Energy | Nowhere | Reflection plus transmission could exceed 1 | Conserved, and tested on the CPU and the GPU ([§8](#8-what-one-pixel-of-glass-shows)) |
| Coating tint | `RESEARCH.md` "AR rim tint `#7B6CFF`" | A guess | Thin-film interference; a quarter-wave MgF₂ coat on sapphire reflects a faint `#9E51FF` ([§9](#9-thin-film-coating)) |
| Caustic and prism geometry | Card P8-1 §4, §5 | Octagon strokes and radial gradients | Not optics. They are layout, and belong to the cluster's design |

Card P8-1's values for stroke colours, gradient stops and sizes are withdrawn with it.

## 12. Corrected sci-fi equations

These belong to clusters that are built later. The equations are fixed here so those clusters start from tested maths (`optics.rs` has the functions).

### 12.1 Relativistic Doppler and beaming (`ChronoEventHorizon`, card P8-2 §2)

The card said the approaching side of the disk is "blueshifted and brightened" and gave no law. The law [S16]:

$$\mathcal D = \frac{1}{\gamma\,(1 - \beta\cos\theta)}, \qquad \gamma = \frac{1}{\sqrt{1 - \beta^2}}, \qquad \nu_{\text{obs}} = \mathcal D\,\nu_{\text{emit}}$$

$\beta$ is the speed over $c$ and $\theta$ the angle between the motion and the line of sight. $I_\nu / \nu^3$ is invariant, so a source with spectrum $S \propto \nu^{\alpha}$ appears brighter at a fixed frequency by

$$\frac{S_{\text{obs}}}{S_{\text{emit}}} = \mathcal D^{\,3 - \alpha}$$

Known values (tested): at $\beta$ = 0.5, $\mathcal D = \sqrt3$ approaching, $1/\sqrt3$ receding and $1/\gamma$ = 0.866 across the line of sight. With a flat spectrum the approaching side is 27 times brighter than the receding side.

Disk radii for a non-rotating black hole of mass $M$ [S17]: the photon sphere is at $3GM/c^2$ and the innermost stable circular orbit, the disk's inner edge, at $6GM/c^2$.

The card's "Einstein ring" formula $r(\theta) = R\,(1 + 0.18\,G_{\text{lat}}\cos(\theta - \beta) + 0.12\sin 2\theta)$ is not gravitational lensing. It is an art-directed wobble and must be labelled as that, or dropped.

### 12.2 Chladni figures (`CymaticResonanceField`, card P8-2 §6)

The card set the mode numbers to real values, `n = rpm / 1200` and `m = speed / 40`. Standing waves on a plate exist only for integer modes, each at its own frequency.

- **Square plate fixed at its centre** [S18], integers $m \ne n$: sand collects where $\cos\frac{n\pi x}{L}\cos\frac{m\pi y}{L} - \cos\frac{m\pi x}{L}\cos\frac{n\pi y}{L} = 0$. For $m = n$ the expression is zero everywhere and gives no figure (tested).
- **Which mode sounds.** A simply supported rectangular plate $a \times b$ has modes $\sin\frac{m\pi x}{a}\sin\frac{n\pi y}{b}$ at $\omega_{mn} = \pi^2\bigl(\frac{m^2}{a^2} + \frac{n^2}{b^2}\bigr)\sqrt{D / 2\rho h}$ [S19]. On a square plate the frequencies run as $m^2 + n^2$: mode (2,1) is 2.5 times mode (1,1) (tested).
- **Round plate** [S20]: Chladni's law, $f = C\,(m + 2n)^p$ with $m$ nodal diameters, $n$ nodal circles and $p \approx 2$ for a flat plate. Rings and lobes belong to round plates, where the shapes are Bessel functions; a square plate has no "2, 4 and 8-lobe rings".

So the cluster drives the plate at the engine's firing frequency, picks the integer mode whose $\omega_{mn}$ is nearest, and cross-fades to the next mode as the frequency passes between them.

## 13. Tests

All in `crates/fh6-kit/src/optics.rs` unless a file is named.

| Equation | Test | Known value it checks |
|---|---|---|
| §1 materials | `sapphire_is_not_crown_glass` | $n_d$, $V_d$, $R_0$ of all three materials against their catalogues |
| §1 Cauchy | `cauchy_from_abbe_reproduces_the_catalogue_lines` | $n_d$ and $V_d$ exactly; within 0.001 of Sellmeier |
| §2 profile | `hemisphere_profile` | A radius-50 sphere: height 40 and normal (0.6, 0, 0.8) at 20 in |
| §2 smooth union | `shape.rs`: `smooth_union_properties`, `two_outlines_touch_at_the_stated_blend_length` | Order independence, never above the plain union, bounded by $k \ln N$; two fronts meet at $k = g / (2 \ln 2)$ |
| §2 surface waves | `a_ripple_is_a_packet_with_the_stated_slope`; `tests/glass_gpu.rs`: `a_ripple_bends_only_where_it_stands` | The slope is the derivative of the height; still three wavelengths away; the shader's displaced ramp against Snell's law within 0.012 |
| §2 standing waves | `symmetric_waves_keep_their_rotational_symmetry`, `standing_wave_gradient_is_the_derivative_of_the_height`, `the_still_zone_pins_the_centre`, `zero_amplitude_is_a_still_surface`; `tests/glass_gpu.rs`: `standing_waves_bend_by_their_exact_slope`, `still_standing_waves_are_the_still_surface` | 2, 3 and 6 waves unchanged by 90°, 60° and 30° turns; gradient and second derivatives against central differences within 1e-4; zero height and slope inside the still radius, a $C^1$ ease; the shader's displaced ramp against Snell's law within 0.012; zero amplitude or a whole-piece still zone writes the flat piece's exact pixels, glass and metal |
| §3 Fresnel | `fresnel_known_values`, `fresnel_matches_the_sine_tangent_form` | 4% at $n$ = 1.5; Brewster's angle; total internal reflection; the sine-tangent form |
| §3 Schlick | `schlick_stays_within_its_stated_error` | Within 0.04 of exact |
| §3 conductors | `mercury_from_its_permittivity`, `conductor_fresnel_limits_and_its_real_form`, `metal_tables_interpolate_and_clamp`; `tests/glass_gpu.rs`: `a_metal_reflects_by_its_complex_fresnel` | $n$ 1.485, $k$ 4.546 from $\varepsilon$; $R_0$ 0.779 from $\varepsilon$ and from the table; above 0.7 at every angle; $k = 0$ is the dielectric; 1 at grazing; the real form; the shader per channel within 0.01 on a dome, for mercury and for a metal whose three channels differ, opaque, nothing under it showing |
| §4 Snell | `refraction_obeys_snell` | $\eta\sin\theta_t = \sin\theta_i$; reversibility; the critical angle |
| §4 contact model | `a_hemisphere_on_a_page_magnifies_by_its_index`, `the_flat_top_shows_what_is_under_it_unbent` | 1.77× magnification; zero displacement under the flat top; $b\sqrt{\eta^2 - 1}$ at the rim |
| §5 dispersion | `index_falls_with_wavelength_in_the_visible`, `channel_wavelengths_are_the_srgb_dominant_wavelengths` | Blue > green > red; 611, 549 and 464 nm within 2 nm |
| §5 colour | `colour_matching_fit_and_matrix` | Equal-energy white has $X = Y = Z$; D65 maps to (1, 1, 1) |
| §6 absorption | `beer_lambert` | 0.5 at the reference depth, 0.25 at twice it |
| §7 GGX | `ggx_is_normalised`, `ggx_lobe_never_reflects_more_than_arrives` | $\int D\cos\theta\,d\omega = 1$; albedo ≤ 1 |
| §7 pixel footprint | `the_normal_turns_at_the_profile_curvature`, `the_footprint_filter_widens_and_clamps` | $-1/(Rs	heta)$ on a sphere and the finite difference of the normal on four profiles; $sigma^2 = 1/2pi$, the clamp at 0.18 |
| §7 key light on the GPU | `tests/glass_gpu.rs`: `highlights_follow_the_light` | The highlight sits where the half vector meets the normal and moves with the light |
| §8 conservation | `reflection_plus_transmission_is_one`; `tests/glass_gpu.rs` | $R + T = 1$ at every angle; invisible glass in a uniform field |
| §9 thin film | `thin_film_known_values`, `a_quarter_wave_coating_reflects_purple_not_a_guessed_hex` | 0, 1.26% and the bare interface; the slab closed form; green is the minimum |
| §10 prism | `prism_minimum_deviation` | 37.18° for 60° and $n$ = 1.5 |
| §12.1 Doppler | `doppler_beaming_known_values` | $\sqrt3$, $1/\sqrt3$, 0.866; the 27:1 ratio |
| §12.2 plates | `plate_modes` | 2.5 and 4 times the fundamental; no figure for $m = n$ |

## 14. Sources

Read on 2026-11-27. Where a page's text extraction dropped Greek letters or fraction bars, the equation was restored to its standard form and then pinned by the test in §13.

- **[S1]** I. H. Malitson and M. J. Dodge, "Refractive index and birefringence of synthetic sapphire", J. Opt. Soc. Am. 62, 1405 (1972); formula from M. J. Dodge, Handbook of Laser Science and Technology vol. IV (CRC Press, 1986), p. 30. Via https://refractiveindex.info/?shelf=main&book=Al2O3&page=Malitson-o
- **[S2]** SCHOTT optical glass catalogue, N-BK7 (Zemax catalogue 2017-01-20b). Via https://refractiveindex.info/?shelf=specs&book=SCHOTT-optical&page=N-BK7
- **[S3]** I. H. Malitson, "Interspecimen comparison of the refractive index of fused silica", J. Opt. Soc. Am. 55, 1205–1208 (1965). Via https://refractiveindex.info/?shelf=glass&book=fused_silica&page=Malitson
- **[S4]** "Abbe number", https://en.wikipedia.org/wiki/Abbe_number (definition and the d, F, C wavelengths).
- **[S5]** "Sapphire", https://en.wikipedia.org/wiki/Sapphire (optical properties: $n_\omega$ 1.768–1.772, $n_\varepsilon$ 1.760–1.763, birefringence 0.008, Abbe number 72.2).
- **[S6]** "Cauchy's equation", https://en.wikipedia.org/wiki/Cauchy%27s_equation
- **[S7]** M. Pharr, W. Jakob and G. Humphreys, Physically Based Rendering, 4th ed., §9.3 "Specular reflection and transmission" (eqs. 9.1 to 9.7). https://pbr-book.org/4ed/Reflection_Models/Specular_Reflection_and_Transmission
- **[S8]** C. Schlick, "An inexpensive BRDF model for physically-based rendering", Computer Graphics Forum 13(3), 233–246 (1994).
- **[S9]** B. Walter, S. R. Marschner, H. Li and K. E. Torrance, "Microfacet models for refraction through rough surfaces", Eurographics Symposium on Rendering 2007 (eqs. 16, 20 to 22, 33, 34, 40; the invariance of radiance over $\eta^2$ is in §4.2). https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf
- **[S10]** C. Wyman, P.-P. Sloan and P. Shirley, "Simple analytic approximations to the CIE XYZ color matching functions", Journal of Computer Graphics Techniques 2(2), 2013 (table 1, listing 1). https://jcgt.org/published/0002/02/01/
- **[S11]** "sRGB", https://en.wikipedia.org/wiki/SRGB (IEC 61966-2-1 matrix, primaries and D65 white).
- **[S12]** Physically Based Rendering, 4th ed., §11.2 "Transmittance" (eqs. 11.5 to 11.7).
- **[S13]** Physically Based Rendering, 4th ed., §9.6 "Roughness using microfacet theory" (eqs. 9.15, 9.16, 9.19 to 9.22, 9.33).
- **[S14]** "Anti-reflective coating", https://en.wikipedia.org/wiki/Anti-reflective_coating (quarter-wave condition, ideal index, magnesium fluoride on crown glass); "Thin-film interference", https://en.wikipedia.org/wiki/Thin-film_interference (path difference $2 n d \cos\theta$ and the half-wave rule).
- **[S15]** "Transfer-matrix method (optics)", https://en.wikipedia.org/wiki/Transfer-matrix_method_(optics) (reflection of a slab in air), which cites Born and Wolf, Principles of Optics. The Airy form in §9 is the sum of that geometric series for one film; the tests tie it to the page's closed form.
- **[S16]** "Relativistic beaming", https://en.wikipedia.org/wiki/Relativistic_beaming (Doppler factor; $S_o = S_e\,\mathcal D^{3-\alpha}$, cited there to Peacock 2012); "Relativistic Doppler effect", https://en.wikipedia.org/wiki/Relativistic_Doppler_effect
- **[S17]** "Photon sphere", https://en.wikipedia.org/wiki/Photon_sphere; "Innermost stable circular orbit", https://en.wikipedia.org/wiki/Innermost_stable_circular_orbit
- **[S18]** P. Bourke, "Chladni plate interference surfaces", http://paulbourke.net/geometry/chladni/
- **[S19]** "Vibration of plates", https://en.wikipedia.org/wiki/Vibration_of_plates (Kirchhoff-Love plate; simply supported rectangular plate).
- **[S20]** "Chladni's law", https://en.wikipedia.org/wiki/Chladni%27s_law (cites Rossing and Fletcher, Principles of Vibration and Sound, 2004).
- **[S21]** F. Hunter, S. Biver and P. Fuqua, *Light: Science & Magic*, chapter 7, "The Case of the Disappearing Glass" (bright-field and dark-field lighting: glass is defined by dark lines on a light ground or light lines on a dark ground). Contents via https://www.oreilly.com/library/view/light-science-and/9780240808192/13_Chapter7.xhtml
- **[S22]** Y. Tokuyoshi and A. S. Kaplanyan, "Stable Geometric Specular Antialiasing with Projected-Space NDF Filtering", Journal of Computer Graphics Techniques 10(2), 2021, https://www.jcgt.org/published/0010/02/02/ (listing 5: isotropic filtering from the normal's screen derivatives; appendix B: $sigma^2 = 1/2pi$; equation 10: the clamp 0.18, from A. S. Kaplanyan, S. Hill, A. Patney and A. Lefohn, "Filtering Distributions of Normals for Shading Antialiasing", HPG 2016).
- **[S23]** G. M. Hale and M. R. Querry, "Optical constants of water in the 200-nm to 200-µm wavelength region", Appl. Opt. 12, 555-563 (1973), https://doi.org/10.1364/AO.12.000555. Table for liquid water at 25 °C via https://refractiveindex.info/?shelf=main&book=H2O&page=Hale (the record `database/data/main/H2O/nk/Hale.yml`): n = 1.339 at 0.400 µm, 1.337 at 0.450, 1.333 at 0.550 and 0.575, 1.332 at 0.600, 1.331 at 0.700.
- **[S24]** A. H. Barr, "Superquadrics and angle-preserving transformations", IEEE Computer Graphics and Applications 1(1), 11-23 (1981), https://doi.org/10.1109/MCG.1981.1673799.
- **[S25]** I. Quilez, "Smooth minimum", https://iquilezles.org/articles/smin/ (the exponential smooth minimum, $-k \log_2(2^{-a/k} + 2^{-b/k})$; the form above uses the natural logarithm, which only rescales $k$).
- **[S26]** D. Gabor, "Theory of communication. Part 1: The analysis of information", Journal of the Institution of Electrical Engineers, Part III, 93(26), 429-441 (1946), https://doi.org/10.1049/ji-3-2.1946.0074 (harmonic oscillations modulated by a Gaussian "probability pulse").
- **[S27]** H. Lamb, *Hydrodynamics*, 6th edition, Cambridge University Press (1932), chapter IX, "Surface waves" (pp. 363-475), which treats the waves that spread from a local disturbance (the Cauchy-Poisson problem).
- **[S28]** E. Hecht, *Optics*, 5th edition, Pearson (2017), chapter 5, refraction at a spherical surface: a surface of radius $R$ from air into a medium of index $n$ brings parallel light to a focus $nR/(n - 1)$ behind it.

- **[S29]** SCHOTT, "BOROFLOAT® technical details", optical data (reference values): refraction index $n_d$ (587.6 nm) 1.47140, dispersion $n_F - n_C$ 71.4 × 10⁻⁴; density 2.23 g/cm³ at 25 °C. https://www.schott.com/en-gb/products/borofloat/technical-details (read 2026-11-28).
- **[S30]** H. Arakawa, T. Inagaki and M. W. Williams, "Optical properties of metals by spectroscopic ellipsometry", Surface Science 96, 248 (1980), quoting the liquid-mercury permittivity of R. S. Smith and R. R. Stromberg, J. Opt. Soc. Am. 56, 1539 (1966). https://www.osti.gov/servlets/purl/6018359 (read 2026-11-28).

- **[S31]** T. Inagaki, E. T. Arakawa and M. W. Williams, "Optical properties of liquid mercury", Physical Review B 23, 5246 (1981), https://doi.org/10.1103/PhysRevB.23.5246; the $n$, $k$ table as published in the refractiveindex.info database (CC0), https://refractiveindex.info/?shelf=main&book=Hg&page=Inagaki (read 2026-11-28).
- **[S32]** P. B. Johnson and R. W. Christy, "Optical constants of transition metals: Ti, V, Cr, Mn, Fe, Co, Ni, and Pd", Physical Review B 9, 5056 (1974), https://doi.org/10.1103/PhysRevB.9.5056; the $n$, $k$ table as published in the refractiveindex.info database (CC0), https://refractiveindex.info/?shelf=main&book=Cr&page=Johnson (read 2026-11-28).
- **[S33]** M. Faraday, "On a peculiar class of acoustical figures; and on certain forms assumed by groups of particles upon vibrating elastic surfaces", Philosophical Transactions of the Royal Society 121, 299-340 (1831), https://doi.org/10.1098/rstl.1831.0018 (the waves at half the driving frequency); W. S. Edwards and S. Fauve, "Patterns and quasi-patterns in the Faraday experiment", Journal of Fluid Mechanics 278, 123-148 (1994), https://doi.org/10.1017/S0022112094003642 (squares, hexagons and the twelve-fold quasi-pattern as sums of standing plane waves of one wavenumber).

[S24] to [S28] are cited for the form of a function, not for a number. Their bibliographic records were checked on 2026-11-27 (for [S27], its chapter and pages, not its text).
