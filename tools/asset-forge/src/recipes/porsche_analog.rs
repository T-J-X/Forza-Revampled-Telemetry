//! PorscheAnalog's bespoke assets (`assets/clusters/porsche_analog/`, ledger
//! `assets/clusters/porsche_analog/ASSETS.md`): the physical hardware of a Porsche 992 five-tube
//! cluster, rendered in Blender through the hero library and drawn by the cluster at full
//! strength under its live vectors.
//!
//! Order (ComfyUI first, Blender last):
//! - **Dial grain (ComfyUI, SDXL + 4x-UltraSharp):** a seamless 4096 px macro of matte black
//!   powder-coat paint, made neutral (luminance only, a fixed mean and spread) before Blender
//!   reads it. It is the texture of every dial face in the Blender scenes (colour and relief), so
//!   the faces are a painted, slightly grained surface under the moving light and not a flat
//!   black fill. The dials sit in the binnacle's shade, so their scenes get the hero rig at
//!   [`CABIN`] power.
//! - **Tube hardware (Blender, Cycles):** one sprite per kind of tube, so the cluster can place,
//!   hide or swap each tube on its own: the rev counter (deep bezel with a diamond-cut inner
//!   chamfer, spun graphite-titanium top and the 11 smoked lenses of the shift ring, a raised scale ring and
//!   the darker spun inner zone, as on the 992.1 dial), the side tube (speed or G) and the outer
//!   tube; plus the satin black domed hub cap and the gear window's machined rim. Rendered
//!   orthographic at 2 px per 4K px at the virtual light's rest azimuth and its ±25° swing, then
//!   box-filtered to 1 px per 4K px.
//! - **Needles (Blender):** the rev counter's needle (white lacquer with a faceted ridge and a
//!   short tail past the hub) and the side tubes' pointer, as sprites pointing up from their
//!   pivots, rendered with the key every 45° all the way round: a needle turns under the light,
//!   so the cluster picks the render that matches its angle. It tints the lacquer with the
//!   palette.
//!
//! `cargo run --release -p asset-forge -- --masters porsche_analog build --set porsche_analog`
//! (hold the GPU lock of the lane you run on). The SDXL texture is cached by its fingerprint and
//! each Blender scene by its own, so a rebuild with nothing changed uses no GPU. With `--masters`
//! every Blender output and the 4096 px texture are copied to the Drive masters folder; this
//! recipe also writes a `*_preview.png` beside them for the tracker.
use crate::ctx::Ctx;
use crate::error::{io_err, Result};
use crate::hero::frame::Frame;
use crate::hero::geom::{self, BezelSpec, Outline};
use crate::hero::mat::{self, metal, Mat};
use crate::hero::part::Part;
use crate::hero::rig::SWING;
use crate::hero::texture::{self, Recipe};
use crate::hero::{Passes, Rendered, Scene};
use crate::img::{self, Img};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Output folder (repo-relative).
const DIR: &str = "assets/clusters/porsche_analog";
/// This dash's folder name inside the Drive masters folder.
const MASTERS_SUB: &str = "porsche_analog";

/// Render scale of every scene (image px per 4K px). The runtime files are the same pictures
/// box-filtered to 1 px per 4K px, so nothing is scaled up when the cluster draws them.
const MASTER_SCALE: f32 = 2.0;
/// Cycles samples of the tubes and of the needles.
const TUBE_SAMPLES: u32 = 256;
const SPRITE_SAMPLES: u32 = 192;
/// Empty margin round a tube in its sprite (px), for the bezel's soft edge and its fillets.
const TUBE_MARGIN: f32 = 8.0;

/// One kind of tube (local 4K px, centred on the tube), as in the cluster's `Geom`
/// (`crates/fh6-clusters/src/clusters/porsche_analog/mod.rs`, whose tests check the same numbers).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Tube {
    name: &'static str,
    /// Radius of the bezel's opening (the dial's edge).
    open: f32,
    /// Outer radius of the bezel.
    outer: f32,
    /// Bezel height above the dial, and its polished inner and outer chamfers.
    height: f32,
    chamfer_in: f32,
    chamfer_out: f32,
    /// Inner radius of the raised scale ring (0 = a flat dial).
    ring: f32,
    /// How far the scale ring stands above the dial's middle.
    ring_step: f32,
}

impl Tube {
    /// Side of the sprite's square (px).
    fn sprite_size(&self) -> f32 {
        2.0 * (self.outer + TUBE_MARGIN)
    }
}

/// The rev counter.
const TACH: Tube = Tube {
    name: "tach",
    open: 214.0,
    outer: 248.0,
    height: 24.0,
    chamfer_in: 8.0,
    chamfer_out: 5.0,
    ring: 150.0,
    ring_step: 3.0,
};
/// The side tubes (speed and G).
const SIDE: Tube = Tube {
    name: "side",
    open: 160.0,
    outer: 184.0,
    height: 18.0,
    chamfer_in: 6.0,
    chamfer_out: 4.0,
    ring: 112.0,
    ring_step: 2.0,
};
/// The outer tubes (the 992's outer dials).
const FAR: Tube = Tube {
    name: "far",
    open: 92.0,
    outer: 108.0,
    height: 14.0,
    chamfer_in: 4.0,
    chamfer_out: 3.0,
    ring: 0.0,
    ring_step: 0.0,
};
/// Every kind of tube (the tests walk them).
#[cfg(test)]
const TUBES: [Tube; 3] = [TACH, SIDE, FAR];

/// The domed hub cap over the needle's pivot (radius, height, collar height), and its finish's
/// roughness: satin, so the softboxes spread into one soft highlight.
const HUB: (f32, f32, f32) = (20.0, 10.0, 2.0);
const HUB_ROUGH: f32 = 0.3;
/// The gear window's machined rim: the porthole's radius, the rim's outer radius and its height.
const PORTHOLE: (f32, f32, f32) = (70.0, 76.0, 6.0);
/// The shift ring: 11 lenses on the rev counter's bezel top, from 205° over 130° (clockwise from
/// 3 o'clock, so ±65° round 12), at this radius and size (along the ring × across it).
const LEDS: usize = 11;
const LED_RADIUS: f32 = 231.0;
const LED_START_DEG: f32 = 205.0;
const LED_SWEEP_DEG: f32 = 130.0;
const LED_SIZE: (f32, f32) = (36.0, 12.0);
/// The bezel's spun top: dark graphite titanium, the darkest ring of each tube as on the 992.1
/// (the first builds' near-black top read as glossy plastic; the light titanium of builds 3 to 5
/// read as aftermarket chrome to three reviewers), with a finish fine enough that the key's
/// spun wedge shows on it.
const BEZEL_TOP: [f32; 3] = [0.12, 0.12, 0.13];
const BEZEL_ROUGH: f32 = 0.24;
/// The rev counter's needle: a short tail past the pivot and a tip in the ticks (from the
/// pivot), widest at the pivot.
const NEEDLE_TAIL: f32 = 26.0;
const NEEDLE_TIP: f32 = 200.0;
const NEEDLE_HALF_W: f32 = 4.5;
const NEEDLE_RIDGE: f32 = 2.6;
/// The needles are rendered with the key every 45° from the rest azimuth (−135°).
const NEEDLE_SWING: [f32; 7] = [-90.0, -45.0, 0.0, 45.0, 90.0, 135.0, 180.0];
/// The side tubes' pointer (from its pivot).
const POINTER_ROOT: f32 = 126.0;
const POINTER_TIP: f32 = 156.0;
const POINTER_HALF_W: f32 = 3.2;

// The needle's tail shows past the hub cap and its tip ends in the ticks, the window fits inside
// the inner zone, and the pointer stays on the side tube's scale ring (checked when the crate
// compiles).
const _: () = {
    assert!(NEEDLE_TAIL > HUB.0);
    assert!(NEEDLE_TIP < TACH.open);
    assert!(PORTHOLE.1 * 2.0 < TACH.open);
    assert!(POINTER_ROOT > SIDE.ring && POINTER_TIP < SIDE.open);
};

/// The ComfyUI dial texture. The prompt names only the paint: the first build's prompt named
/// an instrument dial, and SDXL drew whole gauges, numerals and all, into the "grain".
const DIAL_GRAIN: Recipe = Recipe {
    name: "porsche_analog_dial_grain",
    prompt: "extreme macro photograph of matte black powder coat paint, fine uniform orange peel \
             texture with tiny soft pigment flecks, a completely flat painted surface filling the \
             whole frame, top-down, even soft diffuse light, seamless tileable texture, physically \
             based material scan",
    negative: "dial, gauge, instrument, clock, watch, speedometer, circle, ring, numerals, numbers, \
               letters, ticks, needle, chrome, metal rim, bezel, scratches, dust, fingerprints, \
               fabric, leather pores, carbon weave, glossy reflections",
    seed: 70_701,
};
/// Size of the dial texture (master and the one Blender reads).
const GRAIN_SIZE: usize = 4096;
/// One repeat of the grain on the dials (local px): the flecks come out about a pixel across.
const GRAIN_TILE: f32 = 768.0;
/// One standard deviation of the neutral grain round its mid-grey mean (sRGB-encoded units):
/// enough relief for the light to catch as a fine matte texture, too little to read as a pattern.
const GRAIN_SPREAD: f32 = 0.08;
/// The window (radius, texture px) over which the grain's mean and spread are evened out, so the
/// smoother cross that SDXL's seam repaint leaves reads like the rest of the paint.
const GRAIN_WINDOW: usize = 48;
/// The dial paint's relief (the leather material's grain parameter).
const GRAIN_RELIEF: f32 = 3.0;
/// Version of the grain normalisation; it is part of the neutral grain's file name, so a change
/// here re-renders the scenes that read it.
const GRAIN_NORMALISE: &[u8] = b"porsche_analog neutral grain v2 (local spread)";
/// The dial paint's albedo comes from this tint over the neutral grain (whose mean is mid grey,
/// 0.214 linear): about 0.006, black paint.
const FACE_TINT: [f32; 3] = [0.03, 0.03, 0.032];
/// The dials sit in the shade of the binnacle cowl, so the tube, hub and window scenes get the
/// hero rig at this power (about −1.2 stops); the needles keep the full rig, as they are lit.
/// In the first build, at full power, the black paint of the scale ring rendered at 5 %
/// luminance (a mid-dark grey) because the hot key's off-specular lobe alone gives it about
/// 2 %; here it comes out near 2 %, while every mirror glint of the key on a chamfer still
/// clips to white.
const CABIN: f32 = 0.45;

/// Builds the texture, the tubes and the needles, writes the runtime files and the cluster's
/// ledger, and writes the previews.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let scratch = ctx.scratch("porsche_analog")?;
    let mut rows: Vec<String> = Vec::new();
    let mut masters: Vec<(PathBuf, String)> = Vec::new();

    // 1. ComfyUI: the dial grain (the Blender dial material's colour and relief), made neutral so
    //    the paint's brightness comes from the material and not from how dark SDXL drew it.
    let sdxl = texture::make_texture(&ctx.paths, &DIAL_GRAIN, GRAIN_SIZE, &scratch.join("textures"))?;
    let grain = neutral_grain(&sdxl, &scratch.join("textures"))?;
    masters.push((sdxl.clone(), "dial grain (ComfyUI, seamless 4096 px)".into()));
    masters.push((
        grain.clone(),
        "dial grain made neutral (luminance only, mid-grey mean, fixed spread): the file Blender reads".into(),
    ));
    rows.push(format!(
        "| (no runtime file: rendered into the tube sprites) | seamless dial texture | {s}x{s} | master | comfyui:porsche_analog::DIAL_GRAIN (SDXL base 1.0 + refiner when installed, 1024², dpmpp_2m karras, half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed={seed} prompt=\"{prompt}\" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the colour and relief of every dial face in the Blender scenes (one repeat per {GRAIN_TILE} px). Master `{file}` |",
        s = GRAIN_SIZE,
        seed = DIAL_GRAIN.seed,
        prompt = squash(DIAL_GRAIN.prompt),
        file = file_name(&grain),
    ));

    // 2. Blender: the three tubes, the hub cap and the window rim (three key azimuths each), and
    //    the two needles (eight azimuths each).
    let jobs: [(Scene, &str, &str); 7] = [
        (
            tube_scene(&TACH, &grain),
            "tach",
            "rev counter: graphite-titanium bezel with the shift ring's 11 smoked lenses, raised scale ring, darker spun inner zone",
        ),
        (tube_scene(&SIDE, &grain), "side", "side tube (speed and G): graphite-titanium bezel, raised scale ring, dial well"),
        (tube_scene(&FAR, &grain), "far", "outer tube (power and boost): graphite-titanium bezel and dial"),
        (hub(), "hub", "the satin black-chrome domed hub cap over the needle's pivot; 48 px square"),
        (window(), "window", "the gear window's machined graphite-titanium rim; 164 px square"),
        (
            needle(),
            "needle",
            "rev counter needle, white lacquer (tinted by the cluster), pointing up from the pivot; frame x -12..12, y -206..30 round the pivot",
        ),
        (
            pointer(),
            "pointer",
            "side-tube pointer, white lacquer (tinted), pointing up from the pivot; frame x -8..8, y -160..-122 round the pivot",
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
                "| `{name}` | {what}; key at {az}° (sRGB, straight alpha) | {rw}x{rh} | {kb} KB | blender:hero (porsche_analog::{stem}, Cycles {samples} samples, ortho at {MASTER_SCALE} px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `{stem}/{master}` |",
                az = key_azimuth(scene, i),
                rw = w / k,
                rh = h / k,
                kb = bytes / 1024,
                samples = scene.samples,
                master = file_name(beauty),
            ));
            masters.push((beauty.clone(), format!("{stem}, key {}°", key_azimuth(scene, i))));
        }
        note_data_masters(stem, &out, &mut masters);
    }

    // 3. Previews beside the masters (the masters themselves are copied by `--masters`).
    let previews = write_previews(&masters, &scratch.join("previews"))?;

    // 4. The cluster's own ledger.
    let mut md = String::from(
        "# PorscheAnalog assets\n\nBuilt by `cargo run --release -p asset-forge -- --masters porsche_analog build --set porsche_analog` (recipe `tools/asset-forge/src/recipes/porsche_analog.rs`, hero library `tools/asset-forge/src/hero/`). The cluster draws a procedural fallback for every file that is missing.\n\n## Runtime files (in git)\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in &rows {
        let _ = writeln!(md, "{r}");
    }
    md.push_str("\n## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/porsche_analog/`)\n\nCopied by `--masters porsche_analog`: the Blender outputs into one sub-folder per job (`hero_porsche_analog_<stem>`), the ComfyUI master into `comfyui/`, and a `*_preview.png` beside each for the tracker.\n\n| Master | What it is |\n|---|---|\n");
    for (path, what) in &masters {
        let _ = writeln!(md, "| `{}` | {what} |", file_name(path));
    }
    let _ = writeln!(md, "\n{previews} preview(s) written.");
    img::write_file(&ctx.abs(&format!("{DIR}/ASSETS.md")), md.as_bytes())?;
    println!("  wrote {DIR}/ ({} runtime files + ASSETS.md)", rows.len() - 1);
    Ok(())
}

/// The bezel of `t` at the origin: polished inner chamfer, spun dark top, black walls, polished
/// dark outer chamfer.
fn bezel(t: &Tube) -> Part {
    let spec = BezelSpec {
        r_in: t.open,
        r_out: t.outer,
        height: t.height,
        drop: t.height * 0.35,
        chamfer_in: t.chamfer_in,
        chamfer_out: t.chamfer_out,
        fillet: 0.8,
        step: 0.0,
    };
    Part::bezel(
        &format!("{}_bezel", t.name),
        [0.0, 0.0],
        0.0,
        spec,
        [
            mat::polished(metal::ALUMINIUM).named("pa_chamfer"),
            mat::spun(BEZEL_TOP).rough(BEZEL_ROUGH).named("pa_bezel_spun"),
            mat::matte([0.008, 0.008, 0.009]).named("pa_wall"),
            mat::polished(metal::BLACK_CHROME).named("pa_outer_chamfer"),
        ],
    )
}

/// The dial of `t` at the origin: a grained face with a raised scale ring and a spun well, or a
/// flat grained face.
fn dial(t: &Tube, grain: &Path) -> Part {
    let face = mat::leather([0.02, 0.02, 0.021], GRAIN_RELIEF)
        .named("pa_dial")
        .set("texture", grain.to_string_lossy().to_string())
        .set("tile_px", GRAIN_TILE)
        .set("tint", serde_json::json!(FACE_TINT))
        .set("sheen", 0.0)
        .rough(0.8);
    if t.ring > 0.0 {
        Part::chaptered_face(
            &format!("{}_dial", t.name),
            [0.0, 0.0],
            0.0,
            t.open,
            t.ring,
            t.ring_step,
            mat::spun([0.012, 0.012, 0.014]).rough(0.22).named("pa_well"),
            face,
        )
    } else {
        Part::face(&format!("{}_dial", t.name), [0.0, 0.0], 0.0, t.open, face)
    }
}

/// Lens `i` of the shift ring, as an outline turned along the ring.
fn led(i: usize) -> Part {
    let a = (LED_START_DEG + LED_SWEEP_DEG * i as f32 / (LEDS - 1) as f32).to_radians();
    let (cx, cy) = (LED_RADIUS * a.cos(), LED_RADIUS * a.sin());
    let local = Outline::rrect(0.0, 0.0, LED_SIZE.0, LED_SIZE.1, LED_SIZE.1 * 0.5, 6);
    // The long side runs along the ring (the tangent is the angle plus 90°).
    let (s, c) = (a + core::f32::consts::FRAC_PI_2).sin_cos();
    let points = local
        .points
        .iter()
        .map(|p| [cx + p[0] * c - p[1] * s, cy + p[0] * s + p[1] * c])
        .collect();
    Part::extrude(
        &format!("led_{i:02}"),
        Outline { points },
        TACH.height - 0.6,
        1.8,
        1.2,
        Mat::new("dielectric", "pa_led_lens")
            .color([0.006, 0.006, 0.008])
            .rough(0.04)
            .set("coat", 1.0)
            .set("coat_rough", 0.0),
    )
}

/// One tube's hardware at the origin; the rev counter also gets the shift ring's lenses.
fn tube_scene(t: &Tube, grain: &Path) -> Scene {
    let mut s = Scene::new(
        &format!("porsche_analog_{}", t.name),
        Frame::square(0.0, 0.0, t.sprite_size()).scaled(MASTER_SCALE),
    );
    s.rig = s.rig.clone().power(CABIN);
    s.samples = TUBE_SAMPLES;
    s.passes = Passes {
        beauty: true,
        swing: true,
        data: true,
        ao: true,
        rmh: false,
        emission: false,
    };
    s.add(dial(t, grain));
    s.add(bezel(t));
    if t.name == TACH.name {
        for i in 0..LEDS {
            s.add(led(i));
        }
    }
    s
}

/// White lacquer for the needles (the cluster tints it with the palette), and a dark underside.
fn lacquer() -> Vec<Mat> {
    vec![
        mat::enamel([0.86, 0.86, 0.84]).named("pa_lacquer"),
        mat::enamel([0.86, 0.86, 0.84]).named("pa_lacquer"),
        mat::matte([0.02, 0.02, 0.02]).named("pa_needle_under"),
    ]
}

/// A sprite scene over `frame` (local px round the pivot).
fn sprite(name: &str, frame: Frame) -> Scene {
    let mut s = Scene::new(name, frame.scaled(MASTER_SCALE));
    s.samples = SPRITE_SAMPLES;
    s.passes = Passes {
        beauty: true,
        swing: true,
        data: true,
        ao: false,
        rmh: false,
        emission: false,
    };
    s
}

/// The rev counter's needle, pointing up from its pivot, widest at the pivot with a short tail.
fn needle() -> Scene {
    let mut s = sprite("porsche_analog_needle", Frame::rect(-12.0, -206.0, 24.0, 236.0));
    s.swing = NEEDLE_SWING.to_vec();
    let widest = NEEDLE_TAIL / (NEEDLE_TAIL + NEEDLE_TIP);
    let blade = geom::blade(-NEEDLE_TAIL, NEEDLE_TIP, NEEDLE_HALF_W, widest, NEEDLE_RIDGE, 0.0);
    s.add(Part::mesh("needle", [0.0, 0.0], 0.0, blade, lacquer()));
    s
}

/// The domed hub cap: satin black chrome under the full rig, as the 992's black cap catches the
/// cabin's light. (A mirror polish in the shaded rig mirrored only the square sky softbox and
/// read as a dark square; satin silver read as a ball bearing.)
fn hub() -> Scene {
    let mut s = sprite("porsche_analog_hub", Frame::square(0.0, 0.0, 48.0));
    s.swing = SWING[1..].to_vec();
    let cap = mat::polished(metal::BLACK_CHROME).rough(HUB_ROUGH).named("pa_hub");
    s.add(Part::dome(
        "hub",
        [0.0, 0.0],
        0.0,
        HUB.0,
        HUB.1,
        HUB.2,
        [cap.clone(), cap],
    ));
    s
}

/// The gear window's rim: a small machined bezel round the porthole.
fn window() -> Scene {
    let mut s = sprite("porsche_analog_window", Frame::square(0.0, 0.0, 164.0));
    s.rig = s.rig.clone().power(CABIN);
    s.swing = SWING[1..].to_vec();
    let spec = BezelSpec {
        r_in: PORTHOLE.0,
        r_out: PORTHOLE.1,
        height: PORTHOLE.2,
        drop: 2.0,
        chamfer_in: 2.5,
        chamfer_out: 1.5,
        fillet: 0.5,
        step: 0.0,
    };
    s.add(Part::bezel(
        "window_rim",
        [0.0, 0.0],
        0.0,
        spec,
        [
            mat::polished(metal::ALUMINIUM).named("pa_chamfer"),
            mat::spun(BEZEL_TOP).rough(BEZEL_ROUGH).named("pa_bezel_spun"),
            mat::matte([0.008, 0.008, 0.009]).named("pa_wall"),
            mat::polished(metal::BLACK_CHROME).named("pa_outer_chamfer"),
        ],
    ));
    s
}

/// The side tubes' pointer, pointing up from its pivot.
fn pointer() -> Scene {
    let mut s = sprite("porsche_analog_pointer", Frame::rect(-8.0, -160.0, 16.0, 38.0));
    s.swing = NEEDLE_SWING.to_vec();
    let blade = geom::blade(POINTER_ROOT, POINTER_TIP, POINTER_HALF_W, 0.1, 1.6, 0.0);
    s.add(Part::mesh("pointer", [0.0, 0.0], 0.0, blade, lacquer()));
    s
}

/// The key azimuth (degrees) of beauty pass `i` of `scene`.
fn key_azimuth(scene: &Scene, i: usize) -> f32 {
    if i == 0 {
        SWING[0]
    } else {
        scene.swing.get(i - 1).copied().unwrap_or(SWING[0])
    }
}

/// Box-filters a master beauty (straight alpha, sRGB-encoded) to 1 px per 4K px and writes it
/// as an optimised 8-bit sRGB PNG. Colour is averaged in linear light weighted by coverage, so
/// edges keep their colour. Returns the file's size.
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

/// A neutral copy of the SDXL grain in `dir`: luminance only, its mean at mid grey and one
/// standard deviation at [`GRAIN_SPREAD`] (clamped at 2.5 deviations), so the dial's brightness
/// comes from the material's tint and not from how dark SDXL drew the paint. It is named by a
/// fingerprint of the source and of these settings, so the Blender scenes re-render when either
/// changes.
fn neutral_grain(src: &Path, dir: &Path) -> Result<PathBuf> {
    let bytes = std::fs::read(src).map_err(|e| io_err(src, e))?;
    let fp = crate::ledger::fingerprint(&[
        &bytes,
        GRAIN_NORMALISE,
        &GRAIN_SPREAD.to_le_bytes(),
        &GRAIN_WINDOW.to_le_bytes(),
    ]);
    let out = dir.join(format!("porsche_analog_dial_grain_neutral_{}.png", &fp[..12]));
    if out.is_file() {
        return Ok(out);
    }
    let img = img::load(src)?;
    let (w, h) = (img.w, img.h);
    let lum: Vec<f32> = (0..w * h)
        .map(|i| {
            img::luma(
                img.get(i % w, i / w, 0),
                img.get(i % w, i / w, 1),
                img.get(i % w, i / w, 2),
            )
        })
        .collect();
    // The mean and the spread over a window round each texel (wrapping, as the texture tiles), so
    // every part of the paint gets the same relief.
    let mean = box_blur_wrap(&lum, w, h, GRAIN_WINDOW);
    let dev: Vec<f32> = lum.iter().zip(&mean).map(|(l, m)| (l - m) * (l - m)).collect();
    let var = box_blur_wrap(&dev, w, h, GRAIN_WINDOW);
    let flat = Img::from_fn(w, h, 3, |x, y| {
        let i = y * w + x;
        let sd = var[i].max(0.0).sqrt().max(1e-4);
        let z = ((lum[i] - mean[i]) / sd).clamp(-2.5, 2.5);
        let v = 0.5 + z * GRAIN_SPREAD;
        [v, v, v, 1.0]
    });
    img::save_png(&flat, &out, false, true)?;
    Ok(out)
}

/// A box blur of radius `r` texels over a `w`×`h` field that wraps at its edges (running sums,
/// one pass across and one down).
fn box_blur_wrap(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    let n = (2 * r + 1) as f64;
    let pass = |src: &[f32], lines: usize, len: usize, at: &dyn Fn(usize, usize) -> usize| -> Vec<f32> {
        let mut out = vec![0.0_f32; src.len()];
        if len == 0 {
            return out;
        }
        let wrap = |k: isize| k.rem_euclid(len as isize) as usize;
        for line in 0..lines {
            let mut acc: f64 = (-(r as isize)..=r as isize)
                .map(|k| f64::from(src[at(line, wrap(k))]))
                .sum();
            for i in 0..len {
                out[at(line, i)] = (acc / n) as f32;
                acc += f64::from(src[at(line, wrap(i as isize + r as isize + 1))]);
                acc -= f64::from(src[at(line, wrap(i as isize - r as isize))]);
            }
        }
        out
    };
    let across = pass(src, h, w, &|line, i| line * w + i);
    pass(&across, w, h, &|line, i| i * w + line)
}

/// Notes the data passes of a scene (normal, height, AO) as masters.
fn note_data_masters(stem: &str, r: &Rendered, masters: &mut Vec<(PathBuf, String)>) {
    for (p, what) in [
        (&r.normal, "tangent-space normal (16-bit)"),
        (&r.height, "height (16-bit)"),
        (&r.ao, "ambient occlusion (16-bit)"),
    ] {
        if let Some(p) = p {
            masters.push((p.clone(), format!("{stem}, {what}")));
        }
    }
}

/// Writes a `<job>_<stem>_preview.png` (at most 1024 px wide, flattened on black) for every
/// master into `dir`, and into the Drive masters folder when it is mounted. Returns how many were
/// written.
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

/// A prompt on one line, with runs of spaces collapsed.
fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hero::part::Shape;

    #[test]
    fn the_wrapping_box_blur_keeps_the_total_and_wraps_round_the_edges() {
        // A constant field stays constant.
        let flat = box_blur_wrap(&[0.25; 12], 4, 3, 1);
        assert!(flat.iter().all(|v| (v - 0.25).abs() < 1e-6));
        // A spike in the corner spreads over its 3x3 neighbours, wrapping to the far edges.
        let mut spike = vec![0.0_f32; 25];
        spike[0] = 9.0;
        let out = box_blur_wrap(&spike, 5, 5, 1);
        for (x, y) in [(0, 0), (1, 0), (4, 0), (0, 1), (0, 4), (4, 4), (1, 1), (4, 1), (1, 4)] {
            assert!((out[y * 5 + x] - 1.0).abs() < 1e-5, "({x}, {y}): {}", out[y * 5 + x]);
        }
        assert!((out.iter().sum::<f32>() - 9.0).abs() < 1e-4);
    }

    #[test]
    fn every_tube_fills_its_sprite_with_a_margin() {
        for t in &TUBES {
            assert!(t.open < t.outer && t.outer + TUBE_MARGIN <= t.sprite_size() * 0.5 + 1e-3);
            assert!(t.ring < t.open, "{}: the scale ring starts inside the dial", t.name);
            assert!(t.chamfer_in + t.chamfer_out < t.outer - t.open);
        }
        // The sprites are whole 4K pixels at 1:1 after the box filter.
        for t in &TUBES {
            let (w, h) = Frame::square(0.0, 0.0, t.sprite_size())
                .scaled(MASTER_SCALE)
                .resolution();
            assert_eq!(w % 2, 0, "{}", t.name);
            assert_eq!(w, h);
        }
    }

    #[test]
    fn the_shift_ring_sits_on_the_bezel_top_with_the_middle_lens_at_twelve() {
        let top_in = TACH.open + TACH.chamfer_in;
        let top_out = TACH.outer - TACH.chamfer_out;
        assert!(LED_RADIUS - LED_SIZE.1 * 0.5 >= top_in);
        assert!(LED_RADIUS + LED_SIZE.1 * 0.5 <= top_out);
        let mid = led(LEDS / 2);
        let Shape::Extrude { outline, .. } = &mid.shape else {
            panic!("a lens is an extrusion, not {:?}", mid.shape);
        };
        let (x0, y0, x1, y1) = outline.bounds();
        let (cx, cy) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        assert!(cx.abs() < 0.5, "centre lens x {cx}");
        assert!((cy + LED_RADIUS).abs() < 0.5, "centre lens y {cy}");
        assert!(x1 - x0 > y1 - y0, "the centre lens lies along the ring");
    }

    #[test]
    fn the_rev_counter_scene_has_its_lenses_and_the_needles_turn_all_round() {
        let s = tube_scene(&TACH, Path::new("grain.png"));
        assert_eq!(s.parts.iter().filter(|p| p.name.starts_with("led_")).count(), LEDS);
        // Eight needle renders, 45° apart, starting at the rest azimuth.
        let n = needle();
        assert_eq!(n.swing.len() + 1, 8);
        for (i, az) in n.swing.iter().enumerate() {
            assert!((az - (SWING[0] + 45.0 * (i + 1) as f32)).abs() < 1e-4);
        }
        assert_eq!(hub().swing.len(), 2);
        let side = tube_scene(&SIDE, Path::new("grain.png"));
        assert_eq!(side.parts.len(), 2, "a side tube is a dial and a bezel");
        assert!(s.materials().iter().any(|m| m.name() == "pa_dial"));
    }
}
