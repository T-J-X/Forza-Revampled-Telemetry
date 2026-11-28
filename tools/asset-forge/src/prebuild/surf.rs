//! Computed surface textures that replace the SDXL ones the batch 2 review failed (2026-11-28:
//! 41 of 67 items, mostly from a 2x2 grid left by the seam repaint and from wrong-looking
//! materials; TJX: "We want very clean textures and animations"). Each surface is a model of
//! its material built from periodic gradient noise ([`super::fx::fbm`]) and periodic Worley
//! cells, so every map tiles exactly, there is no AI artefact, and the finest feature is
//! kept several pixels wide (no speckle). Each gives a colour (linear) and, where the
//! material has relief, a height that the caller turns into a normal map.
use super::fx::fbm;
use crate::img::{self, Img};
use crate::procedural::Rng;
use std::f32::consts::TAU;

/// Which surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surf {
    /// Anti-glare matte display coating: soft micro-bumps, ±4% luminance.
    Antiglare,
    /// LCD backlight mura: soft cloudy unevenness on dark grey-green.
    LcdMura,
    /// A continuous glassy black ceramic coating: mottled sheen, fine orange-peel relief.
    CeramicCoating,
    /// Full-grain pebble leather (Worley crevices), near black.
    Leather,
    /// Embossed automotive leatherette: a smaller, more regular grain.
    Leatherette,
    /// Alcantara microfibre: a fine directional nap, low contrast.
    Alcantara,
    /// Acid-etched frosted glass: soft fine bumps.
    EtchedGlass,
    /// Matte black powder coat: soft orange peel.
    PowderCoat,
    /// Black urushi lacquer: deep, nearly flat, faint brush drag.
    Urushi,
    /// Still water: faint periodic ripples and capillary waves (height).
    StillWater,
    /// Washi paper: long kozo fibres over a cloudy formation.
    Washi,
    /// Wet asphalt: aggregate stones with shallow ripples over them.
    WetAsphalt,
    /// Sparse soft glints inside crystal.
    Sparkle,
    /// Forged carbon with small, swirled chips (marble).
    ForgedMarble,
    /// Forged carbon with broad, flat chips.
    ForgedChips,
    /// Black crackle paint: plates and thin cracks.
    Crackle,
    /// Granular chromatin: soft grains, grey.
    Chromatin,
    /// A cell membrane: soft granules over faint fibres, grey.
    Membrane,
    /// Antique glass: sparse seed bubbles and faint striae, grey on black.
    SeedGlass,
    /// Faint fine diagonal wear streaks, top left to bottom right, very low contrast.
    WearStreaks,
}

/// One computed surface.
pub struct SurfOut {
    /// Colour (linear, RGB).
    pub color: Img,
    /// Height 0..1 when the material has relief.
    pub height: Option<Img>,
    /// Normal strength for `img::normal_from_height` (height units per px at the master size).
    pub normal_strength: f32,
    /// Fibre direction (RG, 0.5-centred) for anisotropic sheen (forged carbon).
    pub direction: Option<Img>,
    /// Roughness (wet asphalt: water in the troughs is glossy).
    pub rough: Option<Img>,
    /// A line on the model for the manifest.
    pub model: &'static str,
}

/// Periodic Worley noise on a square grid of `p` cells (coordinates in cell units, wrapping at
/// `p`): distances to the nearest and second-nearest feature points, the nearest point's id,
/// and the offset from it. `jitter` 0..1 moves the points off the cell centres.
fn worley(x: f32, y: f32, p: i32, jitter: f32, seed: u32) -> (f32, f32, u32, [f32; 2]) {
    let (cx, cy) = (x.floor() as i32, y.floor() as i32);
    let (mut f1, mut f2, mut id, mut off) = (f32::MAX, f32::MAX, 0, [0.0; 2]);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (gx, gy) = (cx + dx, cy + dy);
            let (wx, wy) = (gx.rem_euclid(p), gy.rem_euclid(p));
            let h = hash2(wx, wy, seed);
            let jx = ((h & 0xffff) as f32 / 65535.0 - 0.5) * jitter + 0.5;
            let jy = ((h >> 16) as f32 / 65535.0 - 0.5) * jitter + 0.5;
            let (px, py) = (gx as f32 + jx, gy as f32 + jy);
            let (ex, ey) = (x - px, y - py);
            let d = (ex * ex + ey * ey).sqrt();
            if d < f1 {
                f2 = f1;
                f1 = d;
                id = hash2(wx, wy, seed ^ 0x5bd1_e995);
                off = [ex, ey];
            } else if d < f2 {
                f2 = d;
            }
        }
    }
    (f1, f2, id, off)
}

fn hash2(x: i32, y: i32, seed: u32) -> u32 {
    let mut h =
        (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1) ^ seed.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

/// A hash to 0..1.
fn unit(h: u32) -> f32 {
    (h & 0x00ff_ffff) as f32 / 16_777_215.0
}

fn smooth(e0: f32, e1: f32, x: f32) -> f32 {
    img::smoothstep(e0, e1, x)
}

/// Runs `f(u, v)` (pixel centres, 0..1) on all cores; `f` returns (colour, height, extra RG).
fn field(n: usize, f: impl Fn(f32, f32) -> ([f32; 3], f32, [f32; 2]) + Sync) -> (Img, Img, Img) {
    let mut c = Img::new(n, n, 3);
    let mut h = Img::new(n, n, 1);
    let mut d = Img::new(n, n, 3);
    let threads = std::thread::available_parallelism().map_or(4, |t| t.get()).min(n);
    let rows = n.div_ceil(threads);
    std::thread::scope(|s| {
        let cc = c.data.chunks_mut(rows * n * 3);
        let hc = h.data.chunks_mut(rows * n);
        let dc = d.data.chunks_mut(rows * n * 3);
        for (ti, ((cb, hb), db)) in cc.zip(hc).zip(dc).enumerate() {
            let f = &f;
            s.spawn(move || {
                for i in 0..hb.len() {
                    let idx = ti * rows * n + i;
                    let (x, y) = (idx % n, idx / n);
                    let (col, hv, dir) = f((x as f32 + 0.5) / n as f32, (y as f32 + 0.5) / n as f32);
                    cb[i * 3..i * 3 + 3].copy_from_slice(&col);
                    hb[i] = hv;
                    db[i * 3] = dir[0];
                    db[i * 3 + 1] = dir[1];
                    db[i * 3 + 2] = 0.5;
                }
            });
        }
    });
    (c, h, d)
}

/// Lattice period for features about `cell` px across at the master size `n` (whole cells).
fn period(n: usize, cell: f32) -> i32 {
    ((n as f32 / cell).round() as i32).max(1)
}

/// Feature sizes are given at the 4096 master; a smaller `n` (preview) scales them down.
const MASTER: f32 = 4096.0;

/// Computes `s` at `n` px square with `seed`.
#[must_use]
pub fn surface(s: Surf, n: usize, seed: u32) -> SurfOut {
    let px = MASTER / n as f32; // master px per px
    let per = |cell: f32| period(n, cell / px);
    let grey = |v: f32| [v, v, v];
    let fb = |u: f32, v: f32, cell: f32, oct: u32, sd: u32| {
        let p = per(cell);
        fbm(u * p as f32, v * p as f32, 0.0, [p, p, 1], oct, sd)
    };
    let wo = |u: f32, v: f32, cell: f32, jit: f32, sd: u32| {
        let p = per(cell);
        worley(u * p as f32, v * p as f32, p, jit, sd)
    };
    let (out, strength, model, dir): (_, f32, &'static str, bool) = match s {
        Surf::Antiglare => (
            field(n, |u, v| {
                let (f1, _, _, _) = wo(u, v, 22.0, 0.9, seed);
                let bump = 1.0 - smooth(0.0, 0.7, f1);
                let low = fb(u, v, 256.0, 2, seed + 3) * 0.5 + 0.5;
                let l = 0.5 + 0.04 * (bump * 0.7 + low * 0.3 - 0.5) * 2.0;
                (grey(l), bump * 0.8 + low * 0.2, [0.5; 2])
            }),
            1.2,
            "Worley soft bumps of 22 px cells over a 256 px low swell; luminance 0.5 +- 0.04",
            false,
        ),
        Surf::LcdMura => (
            field(n, |u, v| {
                let m = fb(u, v, 1400.0, 3, seed) * 0.5 + 0.5;
                let k = 0.85 + 0.3 * m;
                ([0.040 * k, 0.058 * k, 0.046 * k], m, [0.5; 2])
            }),
            0.0,
            "three-octave fbm, 1400 px cells: soft backlight unevenness +-15% on dark grey-green, no grain",
            false,
        ),
        Surf::CeramicCoating => (
            field(n, |u, v| {
                let mot = fb(u, v, 900.0, 3, seed) * 0.5 + 0.5;
                let peel = fb(u, v, 48.0, 2, seed + 9) * 0.5 + 0.5;
                let l = 0.018 * (0.8 + 0.4 * mot);
                (grey(l), peel * 0.7 + mot * 0.3, [0.5; 2])
            }),
            1.5,
            "continuous: 900 px mottling of the satin sheen, 48 px orange-peel relief; no joints",
            false,
        ),
        Surf::Leather | Surf::Leatherette => {
            let (cell, jit, base) = if s == Surf::Leather {
                (30.0, 0.85, 0.028)
            } else {
                (20.0, 0.55, 0.024)
            };
            (
                field(n, move |u, v| {
                    // Slight warp so the pebbles are not round.
                    let wx = fb(u, v, 160.0, 2, seed + 5) * 0.004 * px;
                    let wy = fb(u, v, 160.0, 2, seed + 6) * 0.004 * px;
                    let (f1, f2, id, _) = wo(u + wx, v + wy, cell, jit, seed);
                    let crevice = smooth(0.0, 0.18, f2 - f1);
                    let dome = 1.0 - smooth(0.0, 0.75, f1) * 0.4;
                    let h = crevice * dome;
                    let tone = 0.96 + 0.08 * unit(id);
                    let l = base * tone * (0.55 + 0.45 * crevice);
                    (grey(l), h, [0.5; 2])
                }),
                2.0,
                if s == Surf::Leather {
                    "Worley pebble grain, 30 px cells, crevices at F2-F1 < 0.18, domed pebbles, tone +-4% per pebble, near black"
                } else {
                    "Worley embossed grain, 20 px cells (more regular), near black"
                },
                false,
            )
        }
        Surf::Alcantara => (
            field(n, |u, v| {
                let p = per(18.0);
                // Nap: fine noise stretched 5:1 along x, with a slow change of brushing.
                let nap = fbm(u * (p / 5).max(1) as f32, v * p as f32, 0.0, [(p / 5).max(1), p, 1], 3, seed) * 0.5 + 0.5;
                let sheen = fb(u, v, 700.0, 2, seed + 4) * 0.5 + 0.5;
                let l = 0.030 * (0.88 + 0.12 * nap) * (0.95 + 0.1 * sheen);
                (grey(l), nap, [0.5; 2])
            }),
            0.5,
            "microfibre nap: 18 px noise stretched 5:1 along x (+-6%), 700 px sheen drift (+-5%); low contrast",
            false,
        ),
        Surf::EtchedGlass => (
            field(n, |u, v| {
                let (f1, _, _, _) = wo(u, v, 14.0, 0.9, seed);
                let bump = 1.0 - smooth(0.0, 0.75, f1);
                (grey(0.5 + 0.06 * (bump - 0.5)), bump, [0.5; 2])
            }),
            1.0,
            "uniform frosted grain: Worley soft bumps of 14 px cells, luminance 0.5 +- 0.03",
            false,
        ),
        Surf::PowderCoat => (
            field(n, |u, v| {
                let peel = fb(u, v, 40.0, 2, seed) * 0.5 + 0.5;
                let low = fb(u, v, 600.0, 2, seed + 2) * 0.5 + 0.5;
                (grey(0.022 * (0.9 + 0.2 * low)), peel, [0.5; 2])
            }),
            1.0,
            "soft orange peel: two-octave fbm of 40 px cells, 600 px tone drift; no flecks",
            false,
        ),
        Surf::Urushi => (
            field(n, |u, v| {
                let p = per(220.0);
                let drag = fbm(u * (p / 10).max(1) as f32, v * p as f32, 0.0, [(p / 10).max(1), p, 1], 3, seed) * 0.5 + 0.5;
                let depth = fb(u, v, 1200.0, 2, seed + 8) * 0.5 + 0.5;
                let k = (0.85 + 0.3 * depth) * (0.97 + 0.06 * drag);
                ([0.014 * k, 0.008 * k, 0.007 * k], drag * 0.3, [0.5; 2])
            }),
            0.2,
            "deep black with a warm undertone: 1200 px depth +-15%, brush drag stretched 10:1 at +-3%",
            false,
        ),
        Surf::StillWater => {
            let mut rng = Rng::new(u64::from(seed));
            let waves: Vec<([f32; 2], f32, f32)> = (0..9)
                .map(|i| {
                    let k = 3.0 + i as f32 * 2.0 + (rng.f32() * 3.0).floor();
                    let a = rng.f32() * TAU;
                    let kv = [(k * a.cos()).round(), (k * a.sin()).round()];
                    (kv, rng.f32() * TAU, 1.0 / (1.0 + i as f32 * 0.6))
                })
                .collect();
            (
                field(n, move |u, v| {
                    let mut h = 0.0;
                    let mut norm = 0.0;
                    for (kv, ph, a) in &waves {
                        h += a * (TAU * (kv[0] * u + kv[1] * v) + ph).sin();
                        norm += a;
                    }
                    let cap = fb(u, v, 90.0, 2, seed + 1) * 0.15;
                    let hh = (h / norm) * 0.5 + 0.5 + cap;
                    (grey(0.02), hh.clamp(0.0, 1.0), [0.5; 2])
                }),
                3.0,
                "nine sine waves with whole wave numbers (exact tile), plus 90 px capillary fbm at 15%",
                false,
            )
        }
        Surf::Washi => {
            let (c, h, d) = field(n, |u, v| {
                let form = fb(u, v, 300.0, 3, seed) * 0.5 + 0.5;
                let k = 0.95 + 0.06 * form;
                ([0.78 * k, 0.76 * k, 0.70 * k], form * 0.3, [0.5; 2])
            });
            let (c, h) = washi_fibres(c, h, n, px, seed + 77);
            ((c, h, d), 1.0, "cloudy formation (300 px fbm, +-3%) with 2400 long kozo fibres (300 to 900 px, 1.5 to 3 px wide, translucent +7%)", false)
        }
        Surf::WetAsphalt => (
            field(n, |u, v| {
                let (f1, f2, id, _) = wo(u, v, 16.0, 0.9, seed);
                let stone = smooth(0.0, 0.25, f2 - f1);
                let tone = 0.040 + 0.02 * unit(id);
                let rip = fb(u, v, 260.0, 2, seed + 4) * 0.5 + 0.5;
                let l = tone * (0.6 + 0.4 * stone);
                (grey(l), stone * 0.35 + rip * 0.65, [0.5; 2])
            }),
            1.5,
            "aggregate: Worley stones of 16 px, tones 0.040 to 0.060 (dark, low contrast); height = 35% stones + 65% shallow 260 px ripples",
            false,
        ),
        Surf::Sparkle => (
            field(n, |u, v| {
                let (f1, _, id, _) = wo(u, v, 42.0, 0.9, seed);
                let on = unit(id) < 0.35;
                let b = unit(id.rotate_left(7)).powi(3);
                let sigma = 0.07 + 0.05 * unit(id.rotate_left(13));
                let g = if on { b * (-(f1 / sigma).powi(2)).exp() } else { 0.0 };
                (grey(g), g, [0.5; 2])
            }),
            0.0,
            "sparse glints: a soft Gaussian flake in 35% of 42 px Worley cells, brightness r^3, 3 to 5 px wide",
            false,
        ),
        Surf::ForgedMarble | Surf::ForgedChips => {
            let marble = s == Surf::ForgedMarble;
            let (c, d) = forged(n, px, seed, marble);
            let h = Img::from_fn(n, n, 1, |_, _| [0.5, 0.0, 0.0, 0.0]);
            (
                (c, h, d),
                0.0,
                if marble {
                    "forged carbon: about 9000 chopped tow flakes (150 to 300 px long, 18 to 40 px wide, ends cut square) laid in random order and direction, bent along a slow periodic flow (the marble swirl), each with fine 6 px fibre striations along its length and its own tone 0.010 to 0.050; flat under resin (no relief); _direction = fibre direction for anisotropic sheen"
                } else {
                    "forged carbon: about 1100 broad chopped flakes (380 to 700 px long, 110 to 200 px wide) laid in random order and direction, each with fine 6 px fibre striations along its length and its own tone 0.010 to 0.050; flat under resin; _direction map"
                },
                true,
            )
        }
        Surf::Crackle => (
            field(n, |u, v| {
                let (f1, f2, _, _) = wo(u, v, 90.0, 0.95, seed);
                let (g1, g2, _, _) = wo(u, v, 30.0, 0.95, seed + 3);
                let crack = 1.0 - smooth(0.0, 0.035, f2 - f1);
                let fine = (1.0 - smooth(0.0, 0.05, g2 - g1)) * 0.35;
                let c = (crack + fine).min(1.0);
                let plate = 1.0 - smooth(0.2, 0.9, f1) * 0.15;
                (grey(0.02 * (1.0 - 0.7 * c)), (1.0 - c) * plate, [0.5; 2])
            }),
            2.0,
            "crackle: plates of 90 px Worley cells split by thin cracks (F2-F1 < 0.035), a fainter 30 px secondary network; flat black plates, shallow grooves",
            false,
        ),
        Surf::Chromatin => (
            field(n, |u, v| {
                let (f1, _, id, _) = wo(u, v, 26.0, 0.9, seed);
                let grain = (1.0 - smooth(0.1, 0.6, f1)) * (0.6 + 0.4 * unit(id));
                let low = fb(u, v, 400.0, 3, seed + 2) * 0.5 + 0.5;
                let l = 0.35 + 0.25 * grain * (0.6 + 0.4 * low);
                (grey(l), grain, [0.5; 2])
            }),
            0.8,
            "soft granules (26 px Worley, varied) over 400 px fbm; grey 0.35 to 0.6, no outlines",
            false,
        ),
        Surf::Membrane => (
            field(n, |u, v| {
                let p = per(60.0);
                let q = fb(u, v, 240.0, 2, seed + 1);
                let fib = fbm(u * (p / 4).max(1) as f32 + q, v * p as f32 + q, 0.0, [(p / 4).max(1), p, 1], 3, seed) * 0.5 + 0.5;
                let (f1, _, _, _) = wo(u, v, 34.0, 0.9, seed + 7);
                let gran = 1.0 - smooth(0.15, 0.6, f1);
                let l = 0.38 + 0.12 * fib + 0.1 * gran;
                (grey(l), fib * 0.5 + gran * 0.5, [0.5; 2])
            }),
            0.6,
            "faint fibres (60 px noise stretched 4:1, warped) with soft granules of 34 px; grey 0.38 to 0.6",
            false,
        ),
        Surf::WearStreaks => (
            field(n, |u, v| {
                // Rotated 45 degrees with whole periods, so it still tiles: a = along, b = across.
                let pa = 6;
                let pb = per(10.0);
                let a = (u + v) * pa as f32;
                let b = (u - v) * pb as f32;
                let st = fbm(a, b, 0.0, [pa, pb, 1], 3, seed) * 0.5 + 0.5;
                let low = fb(u, v, 800.0, 2, seed + 4) * 0.5 + 0.5;
                let l = 0.5 + 0.03 * (st - 0.5) * 2.0 * (0.6 + 0.4 * low);
                (grey(l), st, [0.5; 2])
            }),
            0.0,
            "fine wear streaks top left to bottom right: periodic fbm stretched along the diagonal (10 px across, about 680 px along, whole periods so it tiles), luminance 0.5 +- 0.03",
            false,
        ),
        Surf::SeedGlass => (
            field(n, |u, v| {
                let (f1, _, id, _) = wo(u, v, 110.0, 0.9, seed);
                let on = unit(id) < 0.4;
                let r = 0.06 + 0.12 * unit(id.rotate_left(5));
                let ring = if on { (-((f1 - r) / 0.025).powi(2)).exp() * 0.8 } else { 0.0 };
                let core = if on { (1.0 - smooth(0.0, r, f1)) * 0.12 } else { 0.0 };
                let p = per(500.0);
                let striae = fbm(u * (p / 4).max(1) as f32, v * (p * 3) as f32, 0.0, [(p / 4).max(1), p * 3, 1], 2, seed + 3) * 0.5 + 0.5;
                let l = (ring + core + 0.04 * striae).min(1.0);
                (grey(l), striae, [0.5; 2])
            }),
            0.4,
            "seed bubbles: in 40% of 110 px cells a ring of radius 7 to 20 px with a bright rim; faint wavy striae (stretched fbm, 4%)",
            false,
        ),
    };
    let (color, height, direction) = out;
    let rough = (s == Surf::WetAsphalt).then(|| {
        // Wet: the ripple troughs hold water (glossy); the stone tops stand dry and rougher.
        let mut r = height.clone();
        for v in &mut r.data {
            *v = 0.06 + 0.5 * smooth(0.45, 0.8, *v);
        }
        r
    });
    SurfOut {
        color,
        height: (strength > 0.0).then_some(height),
        normal_strength: strength,
        direction: dir.then_some(direction),
        rough,
        model,
    }
}

/// Forged carbon: chopped tow flakes painted in random order (later flakes cover earlier ones),
/// wrapping so the tile is exact. Returns the colour and the fibre direction map.
fn forged(n: usize, px: f32, seed: u32, marble: bool) -> (Img, Img) {
    let mut rng = Rng::new(u64::from(seed));
    let base = 0.012_f32;
    let mut c = Img::from_fn(n, n, 3, |_, _| [base, base, base, 1.0]);
    let mut d = Img::from_fn(n, n, 3, |_, _| [0.5, 0.5, 0.5, 1.0]);
    let (len0, len1, wid0, wid1, count) = if marble {
        (150.0, 300.0, 18.0, 40.0, 9000.0)
    } else {
        (380.0, 700.0, 110.0, 200.0, 1100.0)
    };
    // Sizes scale with the image, so the count stays: the coverage is the same at any size.
    let count = count as usize;
    let p = period(n, 900.0 / px);
    for _ in 0..count {
        let (cx, cy) = (rng.f32() * n as f32, rng.f32() * n as f32);
        // Marble: the flake follows a slow flow field, so neighbours swirl together.
        let a = if marble {
            let (u, v) = (cx / n as f32, cy / n as f32);
            fbm(u * p as f32, v * p as f32, 0.0, [p, p, 1], 2, seed + 21) * std::f32::consts::PI * 1.6
                + (rng.f32() - 0.5) * 0.5
        } else {
            rng.f32() * std::f32::consts::PI
        };
        let len = (len0 + (len1 - len0) * rng.f32()) / px;
        let wid = (wid0 + (wid1 - wid0) * rng.f32()) / px;
        let tone = 0.010 + 0.040 * rng.f32().powf(1.3);
        let phase = rng.f32() * TAU;
        let (ca, sa) = (a.cos(), a.sin());
        let reach = (len * 0.5 + wid * 0.5).ceil() as isize + 1;
        let pitch = 6.0 / px;
        let edge = 1.0_f32.max(0.8 / px);
        for yy in -reach..=reach {
            for xx in -reach..=reach {
                let (fx, fy) = (xx as f32 + 0.5 - cx.fract(), yy as f32 + 0.5 - cy.fract());
                let along = fx * ca + fy * sa;
                let across = -fx * sa + fy * ca;
                let m = (1.0 - smooth(len * 0.5 - edge, len * 0.5, along.abs()))
                    * (1.0 - smooth(wid * 0.5 - edge, wid * 0.5, across.abs()));
                if m <= 0.0 {
                    continue;
                }
                let stri = 0.5 + 0.5 * (across / pitch * TAU + phase).sin();
                let l = tone * (0.82 + 0.18 * stri);
                let (ux, uy) = (
                    (cx.floor() as isize + xx).rem_euclid(n as isize) as usize,
                    (cy.floor() as isize + yy).rem_euclid(n as isize) as usize,
                );
                for ch in 0..3 {
                    let v = c.get(ux, uy, ch);
                    c.set(ux, uy, ch, v + (l - v) * m);
                }
                if m > 0.5 {
                    d.set(ux, uy, 0, ca * 0.5 + 0.5);
                    d.set(ux, uy, 1, sa * 0.5 + 0.5);
                }
            }
        }
    }
    (c, d)
}

/// Splats long translucent fibres onto the washi formation (wrapping, so the tile stays exact).
fn washi_fibres(mut c: Img, mut h: Img, n: usize, px: f32, seed: u32) -> (Img, Img) {
    let mut rng = Rng::new(u64::from(seed));
    let count = 2400;
    for _ in 0..count {
        let (mut x, mut y) = (rng.f32() * n as f32, rng.f32() * n as f32);
        let mut a = rng.f32() * TAU;
        let len = (300.0 + 600.0 * rng.f32()) / px;
        let width = (1.5 + 1.5 * rng.f32()) / px;
        let gain = 0.05 + 0.04 * rng.f32();
        let bend = (rng.f32() - 0.5) * 0.004 * px;
        let steps = (len / (width * 0.5).max(0.5)).ceil() as usize;
        let step = len / steps as f32;
        let r = (width * 1.5).ceil() as isize;
        for _ in 0..steps {
            for dy in -r..=r {
                for dx in -r..=r {
                    let (sx, sy) = (x.floor() as isize + dx, y.floor() as isize + dy);
                    let d = (((sx as f32 + 0.5) - x).powi(2) + ((sy as f32 + 0.5) - y).powi(2)).sqrt();
                    let w = (-(d / (width * 0.5)).powi(2)).exp();
                    if w < 0.02 {
                        continue;
                    }
                    let (ux, uy) = (sx.rem_euclid(n as isize) as usize, sy.rem_euclid(n as isize) as usize);
                    // Translucent: brighten towards white by at most `gain`, never past it.
                    for ch in 0..3 {
                        let v = c.get(ux, uy, ch);
                        let t = (gain * w).min(0.12);
                        c.set(ux, uy, ch, v + (0.92 - v).max(0.0) * t);
                    }
                    let hv = h.get(ux, uy, 0);
                    h.set(ux, uy, 0, hv.max(0.3 + 0.7 * w));
                }
            }
            x += a.cos() * step;
            y += a.sin() * step;
            a += bend;
        }
    }
    (c, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Surf; 20] = [
        Surf::WearStreaks,
        Surf::Antiglare,
        Surf::LcdMura,
        Surf::CeramicCoating,
        Surf::Leather,
        Surf::Leatherette,
        Surf::Alcantara,
        Surf::EtchedGlass,
        Surf::PowderCoat,
        Surf::Urushi,
        Surf::StillWater,
        Surf::Washi,
        Surf::WetAsphalt,
        Surf::Sparkle,
        Surf::ForgedMarble,
        Surf::ForgedChips,
        Surf::Crackle,
        Surf::Chromatin,
        Surf::Membrane,
        Surf::SeedGlass,
    ];

    #[test]
    fn every_surface_tiles_with_no_seam_and_no_grid() {
        for s in ALL {
            let o = surface(s, 256, 11);
            for im in [Some(&o.color), o.height.as_ref()].into_iter().flatten() {
                let score = img::tile_seam_score(im);
                // The seam must be no worse than the step between two neighbouring inner
                // columns or rows (a seam repaint grid shows up as a step much larger).
                let (w, hh) = (im.w, im.h);
                let mut inner = 0.0_f32;
                for y in 0..hh {
                    for ch in 0..im.c.min(3) {
                        inner = inner.max((im.get(w / 2, y, ch) - im.get(w / 2 - 1, y, ch)).abs());
                    }
                }
                let mut seam = 0.0_f32;
                for y in 0..hh {
                    for ch in 0..im.c.min(3) {
                        seam = seam.max((im.get(0, y, ch) - im.get(w - 1, y, ch)).abs());
                    }
                }
                assert!(
                    seam <= inner * 1.6 + 0.02,
                    "{s:?}: seam {seam} vs inner {inner} (score {score})"
                );
            }
        }
    }

    #[test]
    fn values_stay_in_range() {
        for s in ALL {
            let o = surface(s, 128, 3);
            assert!(o.color.data.iter().all(|v| (0.0..=1.0).contains(v)), "{s:?} colour");
            if let Some(h) = &o.height {
                assert!(h.data.iter().all(|v| (0.0..=1.0).contains(v)), "{s:?} height");
            }
        }
    }
}
