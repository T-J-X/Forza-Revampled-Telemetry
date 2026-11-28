//! Gt7HoodPro's bespoke assets (`assets/clusters/gt7_hood_pro/`, ledger
//! `assets/clusters/gt7_hood_pro/ASSETS.md`): the physical hardware of Gran Turismo 7's Bonnet
//! View meter built as a 2027 production part, rendered in Blender through the hero library and
//! drawn by the cluster at full strength under its live vectors (the lit LED ticks, the digits,
//! the tyre loads, the pedal fills and the boost arc).
//!
//! Order (ComfyUI first, Blender last):
//! - **Forged carbon (ComfyUI, SDXL + 4x-UltraSharp):** a seamless 4096 px macro of forged
//!   carbon. It is the colour and relief of the tyre diagram's car chip in the Blender scene, so
//!   GT7's grey car icon becomes a sculpted carbon car seen from above.
//! - **The hardware plate (Blender, Cycles):** every static part of the band in one plate
//!   registered to the hood envelope: the rev bar's pillowed black-glass light bar on its
//!   polished plinth with 95 domed smoked LED lenses, the steering rail, the carbon car chip with
//!   its canopy and four tyre lenses, the boost gauge's gunmetal arc with its marks and the
//!   black-glass groove of its fill, the two pedal slots, the two domed telltale lamps in their
//!   rings and the black-glass trip window. Lit by the band rig ([`band_rig`]).
//!   Rendered orthographic at 2 px per 4K px at the virtual light's rest azimuth and its ±25°
//!   swing, then box-filtered to 1 px per 4K px.
//! - **The suggested-gear tile (Blender):** the red enamel tile the cluster shows at the shift
//!   point (the palette's `warning` red), as its own sprite (three azimuths) because it comes and
//!   goes.
//!
//! The numbers below mirror the cluster's `Geom` (`crates/fh6-clusters/src/clusters/gt7_hood_pro/
//! geom.rs`); a test there reads this file and fails when the two disagree.
//!
//! `cargo run --release -p asset-forge -- --masters gt7_hood_pro build --set gt7_hood_pro` (hold
//! the GPU lock of the lane you run on). The SDXL texture is cached by its fingerprint and each
//! Blender scene by its own, so a rebuild with nothing changed uses no GPU.
use crate::ctx::Ctx;
use crate::error::{io_err, Result};
use crate::hero::frame::Frame;
use crate::hero::geom::Outline;
use crate::hero::mat::{self, metal, Mat};
use crate::hero::part::Part;
use crate::hero::rig::{Light, Rig, SWING};
use crate::hero::texture::{self, Recipe};
use crate::hero::{Passes, Scene};
use crate::img::{self, Img};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Output folder (repo-relative).
const DIR: &str = "assets/clusters/gt7_hood_pro";
/// This dash's folder name inside the Drive masters folder.
const MASTERS_SUB: &str = "gt7_hood_pro";

/// Render scale of every scene (image px per 4K px); the runtime files are box-filtered to 1:1.
const MASTER_SCALE: f32 = 2.0;
/// Cycles samples.
const SAMPLES: u32 = 192;
/// The band sits outdoors over the bonnet but every static part must stay dim (the review's
/// static-parts cap), so the rig runs at this power: chamfers still clip to white glints.
const RIG_POWER: f32 = 0.55;
/// Seen from straight above, a flat polished face mirrors only the zenith, so the hero rig's
/// overhead sky turned every flat black-glass face one even grey that read as matte (the first
/// build). The sky is turned down to this share of the hero rig's, the studio world with it,
/// and a window softbox on the key side, low enough to stay off the zenith, lights the curved
/// edges instead: the pillowed bar, the lenses, the domes and the half-round slots carry one
/// bright band each on the key side, and the flat faces stay deep black glass.
const SKY_SHARE: f32 = 0.4;
const HDRI_SHARE: f32 = 0.5;
/// The window: elevation (degrees), distance and size (shares of the span), and power.
const WINDOW: (f32, f32, f32, f32) = (52.0, 2.0, 1.0, 0.5);
/// The window on the key diagonal lit only the bar's rounded ends (the second build). A long
/// strip straight up the screen (45° round from the key, so 12 o'clock at rest) lights the long
/// top edge of every pillowed part, and a weaker one below lights the bottom edge: azimuth
/// offset from the key, elevation, distance, length and width (shares of the span), power.
const TOP_STRIP: (f32, f32, f32, f32, f32, f32) = (45.0, 55.0, 2.0, 1.4, 0.08, 0.25);
const BOTTOM_STRIP: (f32, f32, f32, f32, f32, f32) = (225.0, 55.0, 2.0, 1.4, 0.08, 0.08);

/// The plate's frame (local px round the hood anchor): the band's static parts plus a margin.
const PLATE: [f32; 4] = [-880.0, -100.0, 1760.0, 240.0];

/// The rev bar: the LED field (x0, x1, y0, y1), the tick count and tick width, and the light
/// bar round it (centre y, width, height, corner radius).
const REV_X: [f32; 2] = [-380.0, 380.0];
const TICK_Y: [f32; 2] = [-60.0, -34.0];
const TICKS: usize = 95;
const TICK_W: f32 = 5.0;
/// Each lens's corner radius: a pill, as the lit LED the cluster draws over it.
const TICK_R: f32 = 2.5;
const BAR: (f32, f32, f32, f32) = (-47.0, 788.0, 46.0, 13.0);
/// The polished plinth under the light bar shows this far round it.
const PLINTH: f32 = 8.0;
/// Heights (px above the plate's floor) and rounded edges: the plinth's polished chamfer, the
/// light bar pillowed down to a flat centre just taller than the LED field, and each lens a
/// small domed cell, so every unlit LED catches the light.
const PLINTH_H: f32 = 4.0;
const PLINTH_BEVEL: f32 = 3.5;
const BAR_H: f32 = 10.0;
const BAR_BEVEL: f32 = 9.0;
const LENS_H: f32 = 2.0;
const LENS_BEVEL: f32 = 2.0;
/// The steering rail the dot rides on: half length, centre y, height (the cluster's
/// `Geom::rail`), with a notch at the centre and a stop at each end.
const RAIL: [f32; 3] = [350.0, -94.0, 6.0];

/// The tyre diagram: the car chip's centre and size, the canopy, and the four tyre lenses.
const CAR_C: [f32; 2] = [-795.0, 18.0];
const CAR_WH: [f32; 2] = [70.0, 160.0];
const CANOPY: (f32, f32, f32) = (12.0, 24.0, 42.0); // centre y offset, half width, half length
const PILL_WH: [f32; 2] = [22.0, 58.0];
const PILLS: [[f32; 2]; 4] = [[-848.0, -38.0], [-742.0, -38.0], [-848.0, 74.0], [-742.0, 74.0]];

/// The boost gauge: centre, scale radius, band width, and its sweep (degrees clockwise from 3
/// o'clock, y down: the arc opens downwards like GT7's small gauge).
const BOOST_C: [f32; 2] = [-600.0, 6.0];
const BOOST_R: f32 = 60.0;
const BOOST_BAND: f32 = 5.0;
const BOOST_SWEEP: [f32; 2] = [160.0, 380.0];
/// The black-glass groove the live boost fill runs in: radius and width.
const BOOST_GROOVE: [f32; 2] = [51.0, 9.0];

/// The pedal slots: centre x of each, centre y, outer size, slot size.
const SLOTS_X: [f32; 2] = [-442.0, 442.0];
const SLOT_CY: f32 = 13.0;
const SLOT_OUTER: [f32; 2] = [24.0, 158.0];
const SLOT_INNER: [f32; 2] = [16.0, 150.0];

/// The telltales (slide left, limiter right): centres, and the ring's radii (the lamp's face
/// inside it).
const TELLTALES: [[f32; 2]; 2] = [[-497.0, -36.0], [497.0, -36.0]];
const TELL_R: [f32; 2] = [24.0, 29.0];

/// The trip tile: centre, outer size, face size.
const TRIP_C: [f32; 2] = [-600.0, 108.0];
const TRIP_OUTER: [f32; 2] = [208.0, 44.0];
const TRIP_FACE: [f32; 2] = [200.0, 36.0];

/// The suggested-gear tile: centre, outer size, face size.
const SUG_C: [f32; 2] = [211.0, 49.0];
const SUG_OUTER: [f32; 2] = [94.0, 86.0];
const SUG_FACE: [f32; 2] = [88.0, 80.0];
/// The tile face's corner radius (its base's is 3 px more).
const SUG_R: f32 = 11.0;
/// Its sprite's frame (local px).
const SUG_FRAME: [f32; 4] = [156.0, -2.0, 110.0, 102.0];
/// The tile's enamel base colour: the palette's fixed `warning` role (#FF2D3A) in linear light,
/// so the rendered tile and the vector fallback share a hue. The render's light and clear coat
/// shade it, so its pixels are near that red, not equal to it (the code audit, RC-3).
const WARNING_RED: [f32; 3] = [1.0, 0.0262, 0.0423];

// Every part fits its frame, the LED field fits inside the bar, and the tiles do not overlap
// (checked when the crate compiles).
const _: () = {
    assert!(REV_X[0] - PLINTH > PLATE[0] && REV_X[1] + PLINTH < PLATE[0] + PLATE[2]);
    assert!(BAR.1 > REV_X[1] - REV_X[0] && BAR.2 > TICK_Y[1] - TICK_Y[0]);
    assert!(CAR_C[0] - CAR_WH[0] > PLATE[0]);
    assert!(TRIP_C[1] + TRIP_OUTER[1] * 0.5 < PLATE[1] + PLATE[3]);
    assert!(SUG_C[0] - SUG_OUTER[0] * 0.5 > SUG_FRAME[0]);
    assert!(SUG_C[0] + SUG_OUTER[0] * 0.5 < SUG_FRAME[0] + SUG_FRAME[2]);
};

/// The ComfyUI car-chip texture. The prompt names only the material, so SDXL draws no objects.
const CARBON: Recipe = Recipe {
    name: "gt7_hood_pro_forged_carbon",
    prompt: "macro photograph of forged carbon fibre composite, large marbled chopped carbon \
             flakes in graphite grey and black under clear resin, glossy, flat top-down, filling \
             the whole frame, even soft studio light, seamless tileable texture, physically based \
             material scan",
    negative: "woven pattern, twill, weave, grid, car, wheel, logo, text, letters, numbers, \
               frame, border, vignette, scratches, dust, fingerprints, colour, rainbow",
    seed: 73_101,
};
/// Size of the texture (master and the one Blender reads).
const CARBON_SIZE: usize = 4096;
/// One repeat of the carbon on the chip (local px): flakes about 6 to 20 px across.
const CARBON_TILE: f32 = 320.0;
/// Tint over the texture (linear): a graphite car, GT7's grey icon in carbon.
const CARBON_TINT: [f32; 3] = [0.42, 0.43, 0.46];

/// Builds the texture, the plate and the tile, writes the runtime files and the ledger.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let scratch = ctx.scratch("gt7_hood_pro")?;
    let mut rows: Vec<String> = Vec::new();
    let mut masters: Vec<(PathBuf, String)> = Vec::new();

    // 1. ComfyUI: the forged carbon of the car chip.
    let carbon = texture::make_texture(&ctx.paths, &CARBON, CARBON_SIZE, &scratch.join("textures"))?;
    masters.push((
        carbon.clone(),
        "forged carbon (ComfyUI, seamless 4096 px): the car chip's colour and relief".into(),
    ));
    rows.push(format!(
        "| (no runtime file: rendered into `plate_*.png`) | seamless forged-carbon texture | {s}x{s} | master | comfyui:gt7_hood_pro::CARBON (SDXL base 1.0 + refiner when installed, 1024², dpmpp_2m karras, half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed={seed} prompt=\"{prompt}\" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the colour and relief of the tyre diagram's car chip (one repeat per {CARBON_TILE} px). Master `{file}` |",
        s = CARBON_SIZE,
        seed = CARBON.seed,
        prompt = squash(CARBON.prompt),
        file = file_name(&carbon),
    ));

    // 2. Blender: the hardware plate and the suggested-gear tile, three key azimuths each.
    let jobs: [(Scene, &str, &str); 2] = [
        (
            plate(&carbon),
            "plate",
            "the band's static hardware: a pillowed black-glass light bar on a polished black-chrome plinth with 95 domed smoked LED lenses, the steering rail with its notch and stops, a forged-carbon car chip with a black-glass canopy and four pillowed black-glass tyre lenses, a polished gunmetal boost arc with four engraved marks and its black-glass groove, two half-round black-glass pedal slots, two domed telltale lamps in black-chrome rings and the black-glass trip window; covers x -880..880, y -100..140",
        ),
        (
            suggest(),
            "suggest",
            "the suggested-gear tile: red enamel on a black-chrome base; covers x 156..266, y -2..100",
        ),
    ];
    for (scene, stem, what) in &jobs {
        let out = scene.render(&ctx.paths, &scratch.join(stem))?;
        let (w, h) = scene.frame.resolution();
        let k = MASTER_SCALE.round().max(1.0) as u32;
        for (i, beauty) in out.beauty.iter().enumerate() {
            let name = format!("{stem}_{i}.png");
            let bytes = runtime_png(beauty, &out_dir.join(&name))?;
            rows.push(format!(
                "| `{name}` | {what}; key at {az}° (sRGB, straight alpha) | {rw}x{rh} | {kb} KB | blender:hero (gt7_hood_pro::{stem}, Cycles {samples} samples, ortho at {MASTER_SCALE} px per 4K px, box-filtered to 1:1; the band rig: hero rig with the sky at {SKY_SHARE} and the world at {HDRI_SHARE}, a window softbox and two strips, at power {RIG_POWER}) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light. Master `{stem}/{master}` |",
                az = key_azimuth(scene, i),
                rw = w / k,
                rh = h / k,
                kb = bytes / 1024,
                samples = scene.samples,
                master = file_name(beauty),
            ));
            masters.push((beauty.clone(), format!("{stem}, key {}°", key_azimuth(scene, i))));
        }
    }

    // 3. Previews beside the masters (the masters themselves are copied by `--masters`).
    let previews = write_previews(&masters, &scratch.join("previews"))?;

    // 4. The cluster's own ledger.
    let mut md = String::from(
        "# Gt7HoodPro assets\n\nBuilt by `cargo run --release -p asset-forge -- --masters gt7_hood_pro build --set gt7_hood_pro` (recipe `tools/asset-forge/src/recipes/gt7_hood_pro.rs`, hero library `tools/asset-forge/src/hero/`). The cluster draws a procedural fallback for every file that is missing.\n\n## Runtime files (in git)\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in &rows {
        let _ = writeln!(md, "{r}");
    }
    md.push_str("\n## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/gt7_hood_pro/`)\n\nCopied by `--masters gt7_hood_pro`: the Blender outputs into one sub-folder per job (`hero_gt7_hood_pro_<stem>`), the ComfyUI master into `comfyui/`, and a `*_preview.png` beside each.\n\n| Master | What it is |\n|---|---|\n");
    for (path, what) in &masters {
        let _ = writeln!(md, "| `{}` | {what} |", file_name(path));
    }
    let _ = writeln!(md, "\n{previews} preview(s) written.");
    img::write_file(&ctx.abs(&format!("{DIR}/ASSETS.md")), md.as_bytes())?;
    println!("  wrote {DIR}/ ({} runtime files + ASSETS.md)", rows.len() - 1);
    Ok(())
}

/// Polished black chrome: the plinths, rims and bases (dark, so only the chamfers glint).
fn black_chrome() -> Mat {
    mat::polished(metal::BLACK_CHROME).named("gt_black_chrome")
}

/// The light bar and the slots: black glass under a mirror coat.
fn black_glass() -> Mat {
    mat::piano_black().named("gt_black_glass")
}

/// A smoked lens over an unlit LED: a touch lighter than the black glass round it, so each lens
/// reads as its own cell, with a mirror coat for the key's glint.
fn smoked_lens() -> Mat {
    Mat::new("dielectric", "gt_smoked_lens")
        .color([0.016, 0.016, 0.018])
        .rough(0.05)
        .set("coat", 1.0)
        .set("coat_rough", 0.0)
}

/// The band's rig: the hero rig with its overhead sky and world turned down and the window
/// softbox added on the key side (see [`SKY_SHARE`]), at [`RIG_POWER`].
fn band_rig(rest: &Rig) -> Rig {
    let mut r = rest.clone();
    let span = r.lights.first().map_or(1000.0, |l| l.dist / 2.2);
    let key_az = r.lights.first().map_or(SWING[0], |l| l.az);
    for l in &mut r.lights {
        if l.name.starts_with("sky") {
            l.power *= SKY_SHARE;
        }
    }
    r.hdri *= HDRI_SHARE;
    let (el, dist, size, power) = WINDOW;
    r.lights
        .push(Light::soft("window", key_az, el, span * dist, span * size, power).tint([0.96, 0.98, 1.0]));
    for (name, (daz, el, dist, len, wid, power)) in [("top_strip", TOP_STRIP), ("bottom_strip", BOTTOM_STRIP)] {
        r.lights.push(
            Light::strip(name, key_az + daz, el, span * dist, span * len, span * wid, power).tint([0.96, 0.98, 1.0]),
        );
    }
    r.power(RIG_POWER)
}

/// The car seen from above, inside the chip's `half` size round `c` (front up): a narrower
/// nose, mirrors, and hips over the rear wheels.
fn car_outline(c: [f32; 2], half: [f32; 2], segs: usize) -> Outline {
    let base = superellipse([0.0, 0.0], half, 3.2, segs);
    let bump = |t: f32, at: f32, w: f32| (-((t - at) / w).powi(2)).exp();
    let points = base
        .points
        .iter()
        .map(|p| {
            let t = p[1] / half[1].max(1.0); // -1 at the nose, 1 at the tail
            let k = 0.8 + 0.1 * (t + 1.0) * 0.5 + 0.06 * bump(t, -0.32, 0.08) + 0.12 * bump(t, 0.5, 0.22);
            [c[0] + p[0] * k.min(1.0), c[1] + p[1]]
        })
        .collect();
    Outline { points }
}

/// A radial mark at `deg` (clockwise from 3 o'clock, y down) from radius `r` out by `len`,
/// `w` wide, round `c`.
fn radial_mark(c: [f32; 2], deg: f32, r: f32, len: f32, w: f32) -> Outline {
    let (s, co) = deg.to_radians().sin_cos();
    let (n, t) = ([co, s], [-s, co]);
    let at = |rad: f32, side: f32| {
        [
            c[0] + n[0] * rad + t[0] * side * w * 0.5,
            c[1] + n[1] * rad + t[1] * side * w * 0.5,
        ]
    };
    Outline {
        points: vec![at(r, -1.0), at(r + len, -1.0), at(r + len, 1.0), at(r, 1.0)],
    }
}

/// A rounded rectangle extruded `thick` px from `z`.
fn slab(name: &str, c: [f32; 2], wh: [f32; 2], r: f32, z: f32, thick: f32, bevel: f32, m: Mat) -> Part {
    Part::extrude(name, Outline::rrect(c[0], c[1], wh[0], wh[1], r, 8), z, thick, bevel, m)
}

/// A rounded superellipse (`n` 2 = ellipse, larger = boxier), centred on `c`.
fn superellipse(c: [f32; 2], half: [f32; 2], n: f32, segs: usize) -> Outline {
    let e = 2.0 / n;
    let points = (0..segs)
        .map(|i| {
            let t = core::f32::consts::TAU * i as f32 / segs as f32;
            let (s, co) = t.sin_cos();
            [
                c[0] + half[0] * co.signum() * co.abs().powf(e),
                c[1] + half[1] * s.signum() * s.abs().powf(e),
            ]
        })
        .collect();
    Outline { points }
}

/// A band along a circular arc (`a0` to `a1` degrees, clockwise from 3 o'clock, y down) with
/// round ends.
fn arc_band(c: [f32; 2], r: f32, width: f32, a0: f32, a1: f32, segs: usize) -> Outline {
    let (ro, ri) = (r + width * 0.5, r - width * 0.5);
    let at = |rad: f32, deg: f32| {
        let (s, co) = deg.to_radians().sin_cos();
        [c[0] + rad * co, c[1] + rad * s]
    };
    let mut points = Vec::with_capacity(2 * segs + 16);
    for i in 0..=segs {
        points.push(at(ro, a0 + (a1 - a0) * i as f32 / segs as f32));
    }
    // Round end at a1: half a circle from the outer edge to the inner.
    let end = at(r, a1);
    for j in 1..8 {
        let phi = (a1 + 180.0 * j as f32 / 8.0).to_radians();
        points.push([end[0] + width * 0.5 * phi.cos(), end[1] + width * 0.5 * phi.sin()]);
    }
    for i in (0..=segs).rev() {
        points.push(at(ri, a0 + (a1 - a0) * i as f32 / segs as f32));
    }
    let start = at(r, a0);
    for j in 1..8 {
        let phi = (a0 + 180.0 + 180.0 * j as f32 / 8.0).to_radians();
        points.push([start[0] + width * 0.5 * phi.cos(), start[1] + width * 0.5 * phi.sin()]);
    }
    Outline { points }
}

/// The band's static hardware over [`PLATE`].
fn plate(carbon: &Path) -> Scene {
    let [x, y, w, h] = PLATE;
    let mut s = Scene::new("gt7_hood_pro_plate", Frame::rect(x, y, w, h).scaled(MASTER_SCALE));
    s.rig = band_rig(&s.rig);
    s.samples = SAMPLES;
    s.passes = Passes {
        beauty: true,
        swing: true,
        data: false,
        ao: false,
        rmh: false,
        emission: false,
    };

    // The steering rail: a black-glass track with a polished notch at the centre and a stop at
    // each end of the dot's travel.
    let [rail_half, rail_y, rail_h] = RAIL;
    s.add(slab(
        "steer_rail",
        [0.0, rail_y],
        [2.0 * rail_half, rail_h],
        rail_h * 0.5,
        0.0,
        3.0,
        2.5,
        black_glass(),
    ));
    s.add(slab(
        "steer_notch",
        [0.0, rail_y],
        [3.0, 10.0],
        1.5,
        0.0,
        4.0,
        1.4,
        black_chrome(),
    ));
    for (i, sx) in [-1.0f32, 1.0].into_iter().enumerate() {
        s.add(slab(
            &format!("steer_stop_{i}"),
            [sx * (rail_half - 3.0), rail_y],
            [6.0, 10.0],
            3.0,
            0.0,
            4.0,
            2.5,
            black_chrome(),
        ));
    }

    // The rev bar: polished plinth, black-glass bar pillowed to a flat centre, 95 domed smoked
    // lenses.
    let bar_c = [0.0, BAR.0];
    s.add(slab(
        "rev_plinth",
        bar_c,
        [BAR.1 + 2.0 * PLINTH, BAR.2 + 2.0 * PLINTH],
        BAR.3 + PLINTH,
        0.0,
        PLINTH_H,
        PLINTH_BEVEL,
        black_chrome(),
    ));
    s.add(slab(
        "rev_bar",
        bar_c,
        [BAR.1, BAR.2],
        BAR.3,
        0.0,
        BAR_H,
        BAR_BEVEL,
        black_glass(),
    ));
    let pitch = (REV_X[1] - REV_X[0]) / TICKS as f32;
    let tick_cy = (TICK_Y[0] + TICK_Y[1]) * 0.5;
    for i in 0..TICKS {
        let cx = REV_X[0] + pitch * (i as f32 + 0.5);
        s.add(slab(
            &format!("lens_{i:02}"),
            [cx, tick_cy],
            [TICK_W, TICK_Y[1] - TICK_Y[0]],
            TICK_R,
            BAR_H,
            LENS_H,
            LENS_BEVEL,
            smoked_lens(),
        ));
    }

    // The tyre diagram: a carbon car seen from above, its canopy, and four tyre lenses.
    let car = mat::leather([0.03, 0.03, 0.032], 2.0)
        .named("gt_forged_carbon")
        .set("texture", carbon.to_string_lossy().to_string())
        .set("tile_px", CARBON_TILE)
        .set("tint", serde_json::json!(CARBON_TINT))
        .set("sheen", 0.0)
        .rough(0.28);
    s.add(Part::extrude(
        "car_body",
        car_outline(CAR_C, [CAR_WH[0] * 0.5, CAR_WH[1] * 0.5], 128),
        0.0,
        7.0,
        5.0,
        car,
    ));
    s.add(Part::extrude(
        "car_canopy",
        car_outline([CAR_C[0], CAR_C[1] + CANOPY.0], [CANOPY.1, CANOPY.2], 96),
        7.0,
        3.0,
        2.8,
        black_glass(),
    ));
    for (i, p) in PILLS.iter().enumerate() {
        s.add(slab(
            &format!("tyre_{i}"),
            *p,
            PILL_WH,
            8.0,
            0.0,
            5.0,
            4.5,
            black_glass(),
        ));
    }

    // The boost gauge's polished arc, and inside it the black-glass groove the fill runs in.
    s.add(Part::extrude(
        "boost_arc",
        arc_band(BOOST_C, BOOST_R, BOOST_BAND, BOOST_SWEEP[0], BOOST_SWEEP[1], 72),
        0.0,
        3.0,
        2.5,
        mat::polished(metal::GUNMETAL).named("gt_polished_gunmetal"),
    ));
    for (i, bar) in BOOST_MARKS.iter().enumerate() {
        let t = (bar - BOOST_BAR[0]) / (BOOST_BAR[1] - BOOST_BAR[0]);
        let deg = BOOST_SWEEP[0] + t * (BOOST_SWEEP[1] - BOOST_SWEEP[0]);
        let [r, len, w] = BOOST_MARK;
        s.add(Part::extrude(
            &format!("boost_mark_{i}"),
            radial_mark(BOOST_C, deg, r, len, w),
            0.0,
            2.0,
            0.8,
            black_chrome(),
        ));
    }
    s.add(Part::extrude(
        "boost_groove",
        arc_band(
            BOOST_C,
            BOOST_GROOVE[0],
            BOOST_GROOVE[1],
            BOOST_SWEEP[0],
            BOOST_SWEEP[1],
            72,
        ),
        0.0,
        2.0,
        1.0,
        black_glass(),
    ));

    // The pedal slots.
    for (i, sx) in SLOTS_X.iter().enumerate() {
        let c = [*sx, SLOT_CY];
        s.add(slab(
            &format!("slot_base_{i}"),
            c,
            SLOT_OUTER,
            6.0,
            0.0,
            PLINTH_H,
            PLINTH_BEVEL,
            black_chrome(),
        ));
        s.add(slab(
            &format!("slot_{i}"),
            c,
            SLOT_INNER,
            SLOT_INNER[0] * 0.5,
            0.0,
            7.0,
            6.0,
            black_glass(),
        ));
    }

    // The telltale rings round their black-glass lamps.
    for (i, c) in TELLTALES.iter().enumerate() {
        s.add(Part::ring(
            &format!("tell_ring_{i}"),
            *c,
            0.0,
            TELL_R[0],
            TELL_R[1],
            4.0,
            black_chrome(),
        ));
        s.add(Part::dome(
            &format!("tell_lamp_{i}"),
            *c,
            0.0,
            TELL_R[0],
            6.0,
            1.0,
            [black_glass(), black_chrome()],
        ));
    }

    // The trip tile.
    s.add(slab(
        "trip_base",
        TRIP_C,
        TRIP_OUTER,
        12.0,
        0.0,
        PLINTH_H,
        PLINTH_BEVEL,
        black_chrome(),
    ));
    s.add(slab("trip_face", TRIP_C, TRIP_FACE, 9.0, 0.0, 5.0, 4.0, black_glass()));
    s
}

/// The boost scale's major marks (bar), engraved outside its arc: inner radius, length, width.
const BOOST_MARKS: [f32; 4] = [-1.0, 0.0, 1.0, 2.0];
const BOOST_BAR: [f32; 2] = [-1.0, 2.0];
const BOOST_MARK: [f32; 3] = [65.0, 5.0, 2.0];

/// The suggested-gear tile over [`SUG_FRAME`].
fn suggest() -> Scene {
    let [x, y, w, h] = SUG_FRAME;
    let mut s = Scene::new("gt7_hood_pro_suggest", Frame::rect(x, y, w, h).scaled(MASTER_SCALE));
    s.rig = band_rig(&s.rig);
    s.samples = SAMPLES;
    s.passes = Passes {
        beauty: true,
        swing: true,
        data: false,
        ao: false,
        rmh: false,
        emission: false,
    };
    s.add(slab(
        "sug_base",
        SUG_C,
        SUG_OUTER,
        SUG_R + 3.0,
        0.0,
        PLINTH_H,
        PLINTH_BEVEL,
        black_chrome(),
    ));
    s.add(slab(
        "sug_tile",
        SUG_C,
        SUG_FACE,
        SUG_R,
        0.0,
        8.0,
        6.0,
        mat::enamel(WARNING_RED).named("gt_red_enamel"),
    ));
    s
}

/// The key azimuth of beauty `i` of `scene`.
fn key_azimuth(scene: &Scene, i: usize) -> f32 {
    if i == 0 {
        SWING[0]
    } else {
        scene.swing.get(i - 1).copied().unwrap_or(SWING[0])
    }
}

/// Box-filters a master (straight alpha) to 1 px per 4K px, averaging in linear light weighted
/// by alpha, and saves it; returns the byte count.
fn runtime_png(master: &Path, out: &Path) -> Result<u64> {
    let src = img::load(master)?;
    let k = MASTER_SCALE.round().max(1.0) as usize;
    let (w, h) = (src.w / k, src.h / k);
    let small = Img::from_fn(w, h, 4, |x, y| {
        let mut acc = [0.0_f32; 4];
        for dy in 0..k {
            for dx in 0..k {
                let (sx, sy) = (x * k + dx, y * k + dy);
                let a = src.get(sx, sy, 3);
                for (ch, v) in acc.iter_mut().enumerate().take(3) {
                    *v += img::srgb_decode(src.get(sx, sy, ch)) * a;
                }
                acc[3] += a;
            }
        }
        let a = acc[3] / (k * k) as f32;
        let inv = if acc[3] > 1e-6 { 1.0 / acc[3] } else { 0.0 };
        [
            img::srgb_encode(acc[0] * inv),
            img::srgb_encode(acc[1] * inv),
            img::srgb_encode(acc[2] * inv),
            a,
        ]
    });
    img::save_png(&small, out, false, true)
}

/// Writes a 1024 px preview of every master (flattened over black) and copies it to the Drive
/// masters folder when that is set up; returns how many were written.
fn write_previews(masters: &[(PathBuf, String)], dir: &Path) -> Result<usize> {
    let drive = crate::lane::masters_dir(MASTERS_SUB).ok();
    let mut n = 0;
    for (path, _) in masters {
        let Ok(src) = img::load(path) else {
            continue;
        };
        let scale = (1024.0 / src.w.max(1) as f32).min(1.0);
        let (w, h) = (
            ((src.w as f32 * scale).round() as usize).max(1),
            ((src.h as f32 * scale).round() as usize).max(1),
        );
        let small = src.resize(w, h, false);
        let flat = Img::from_fn(w, h, 3, |x, y| {
            let a = if small.c > 3 { small.get(x, y, 3) } else { 1.0 };
            [
                small.get(x, y, 0) * a,
                small.get(x, y, 1) * a,
                small.get(x, y, 2) * a,
                1.0,
            ]
        });
        let stem = path
            .file_stem()
            .map_or_else(|| "master".into(), |s| s.to_string_lossy().to_string());
        let job = path
            .parent()
            .and_then(Path::file_name)
            .map_or_else(String::new, |s| format!("{}_", s.to_string_lossy()));
        let name = format!("{job}{stem}_preview.png");
        img::save_png(&flat, &dir.join(&name), false, true)?;
        if let Some(d) = &drive {
            if let Err(e) = std::fs::copy(dir.join(&name), d.join(&name)) {
                eprintln!("  preview {name} not copied to Drive: {e}");
            }
        }
        n += 1;
    }
    Ok(n)
}

/// A path's file name.
fn file_name(p: &Path) -> String {
    p.file_name()
        .map_or_else(String::new, |s| s.to_string_lossy().to_string())
}

/// Collapses runs of whitespace (prompts in the ledger).
fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_led_lenses_fill_the_field_and_stay_inside_the_bar() {
        let s = plate(Path::new("carbon.png"));
        let lenses: Vec<&Part> = s.parts.iter().filter(|p| p.name.starts_with("lens_")).collect();
        assert_eq!(lenses.len(), TICKS);
        let pitch = (REV_X[1] - REV_X[0]) / TICKS as f32;
        assert!(pitch > TICK_W + 2.0, "each lens keeps a gap to the next");
        assert!(REV_X[1] - REV_X[0] < BAR.1 && TICK_Y[1] - TICK_Y[0] < BAR.2);
    }

    #[test]
    fn the_arc_band_is_closed_round_both_ends_and_stays_on_its_radius() {
        let o = arc_band(BOOST_C, BOOST_R, BOOST_BAND, BOOST_SWEEP[0], BOOST_SWEEP[1], 72);
        for p in &o.points {
            let d = ((p[0] - BOOST_C[0]).powi(2) + (p[1] - BOOST_C[1]).powi(2)).sqrt();
            assert!((BOOST_R - BOOST_BAND..=BOOST_R + BOOST_BAND).contains(&d), "{d}");
        }
        assert!(o.points.len() > 2 * 72);
    }

    #[test]
    fn the_car_outline_stays_in_the_chip_with_a_narrower_nose() {
        let half = [CAR_WH[0] * 0.5, CAR_WH[1] * 0.5];
        let o = car_outline(CAR_C, half, 128);
        let width_at = |front: bool| {
            o.points
                .iter()
                .filter(|p| ((p[1] - CAR_C[1]) / half[1] - if front { -0.75 } else { 0.5 }).abs() < 0.08)
                .map(|p| (p[0] - CAR_C[0]).abs())
                .fold(0.0, f32::max)
        };
        assert!(
            width_at(true) < width_at(false) * 0.95,
            "nose {} hips {}",
            width_at(true),
            width_at(false)
        );
        for p in &o.points {
            assert!((p[0] - CAR_C[0]).abs() <= half[0] + 1e-3 && (p[1] - CAR_C[1]).abs() <= half[1] + 1e-3);
        }
    }

    #[test]
    fn the_rig_keeps_the_zenith_dark_and_lights_the_edges() {
        let s = plate(Path::new("carbon.png"));
        for r in s.rigs() {
            let window = r.lights.iter().find(|l| l.name == "window").expect("a window");
            // The window's angular half size stays off the zenith.
            let half = (window.size[0] * 0.5 / window.dist).atan().to_degrees();
            assert!(window.el + half < 80.0, "window reaches {}°", window.el + half);
            assert!(
                (window.az - r.lights[0].az).abs() < 1e-3,
                "the window turns with the key"
            );
        }
        let lenses = s.parts.iter().filter(|p| p.name.starts_with("lens_")).count();
        assert_eq!(lenses, TICKS);
        // The pillowed bar keeps a flat centre for the LED field.
        assert!(BAR.2 - 2.0 * BAR_BEVEL >= TICK_Y[1] - TICK_Y[0]);
        const { assert!(BAR_BEVEL <= BAR_H && LENS_BEVEL <= LENS_H) };
        // Every boost mark stays clear of the arc.
        const { assert!(BOOST_MARK[0] >= BOOST_R + BOOST_BAND * 0.5 + 1.5) };
    }

    #[test]
    fn the_car_outline_spans_the_chip() {
        let o = superellipse(CAR_C, [CAR_WH[0] * 0.5, CAR_WH[1] * 0.5], 3.2, 96);
        let (x0, x1) = o
            .points
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p[0]), b.max(p[0])));
        let (y0, y1) = o
            .points
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p[1]), b.max(p[1])));
        assert!((x1 - x0 - CAR_WH[0]).abs() < 0.5 && (y1 - y0 - CAR_WH[1]).abs() < 0.5);
    }

    #[test]
    fn both_scenes_render_every_swing_azimuth() {
        for s in [plate(Path::new("carbon.png")), suggest()] {
            assert_eq!(s.rigs().len(), SWING.len());
            assert!(s.passes.beauty && s.passes.swing);
        }
    }
}
