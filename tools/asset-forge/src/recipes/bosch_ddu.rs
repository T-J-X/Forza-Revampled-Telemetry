//! BoschDdu's bespoke assets (`assets/clusters/bosch_ddu/`, ledger
//! `assets/clusters/bosch_ddu/ASSETS.md`): the physical hardware of the Bosch Motorsport DDU 10 as
//! BMW fits it to the M4 GT3 (198 × 134 × 35 mm), rendered in Blender through the hero library and
//! drawn by the cluster at full strength under everything live (screen content, lit LEDs).
//!
//! Order (ComfyUI first, Blender last):
//! - **Bead-blast macro (ComfyUI, SDXL + 4x-UltraSharp):** a seamless 4096 px macro of
//!   bead-blasted anodised aluminium, made neutral (luminance only, a fixed mean and spread)
//!   before Blender reads it. It varies the body's colour and roughness and gives it a faint
//!   bump, so the aluminium reads as bead-blasted satin and not as a flat grey.
//! - **Housing (Blender, Cycles):** one sprite of the whole unit. A dark satin gunmetal anodised
//!   body (the macro above under a thin oxide coat) with the arched top edge, large top corners
//!   and small bottom ones; one black glass face (IOR 1.5, near-black under it) set 6 px inside
//!   the outline with a rounded edge that draws a crisp thin highlight; the ten smoked lenses
//!   across the brow on the arch, five down each side (short horizontal pills, milky grey when
//!   unlit), and the ambient light sensor's dark round window in the right border. No softbox
//!   stands over the face ([`housing_rig`]), so the glass mirrors nothing: the screen stays OLED
//!   black and the face reads by its rim line and its lenses. Seen from the driver's eye: the face is registered 1:1 to the
//!   cluster's `GEOM`, and the body is drawn in an oblique projection (its depth sheared up the
//!   screen by [`SLIVER`] px), so a band of its top surface shows above the face edge and the
//!   side and bottom walls stay hidden. Rendered orthographic at 2 px per 4K px at the three key
//!   azimuths of [`SWING`] (rest, then the swing either way), then box-filtered to 1 px per 4K
//!   px: `housing_0.png` (−135°), `housing_1.png` (−160°), `housing_2.png` (−110°), straight
//!   alpha, frame x −440..440, y −320..240 (the runtime's `HOUSING_FRAME`).
//!
//! `cargo run --release -p asset-forge -- --lane cloud --masters bosch_ddu build --set bosch_ddu`
//! (hold the GPU lock of the lane you run on). The SDXL texture is cached by its fingerprint and
//! the Blender scene by its own, so a rebuild with nothing changed uses no GPU. With `--masters`
//! the Blender outputs and the 4096 px textures are copied to the Drive masters folder; the
//! recipe also writes a `*_preview.png` beside them for the tracker.
//!
//! The hero library's metal builder takes no image texture, so this recipe adds two material
//! builders to the hero bpy ([`BPY_MATERIALS`]: the textured anodised body and the black glass)
//! and renders the hero scene itself ([`render`]), with the same fingerprint cache as
//! `Scene::render`. Only the beauty passes are rendered.
use crate::ctx::Ctx;
use crate::error::{io_err, tool_err, Result};
use crate::hero::frame::Frame;
use crate::hero::geom::{Mesh, Outline};
use crate::hero::mat::Mat;
use crate::hero::part::Part;
use crate::hero::rig::{Light, Rig, SWING};
use crate::hero::texture::{self, Recipe};
use crate::hero::{Passes, Scene};
use crate::img::{self, Img};
use crate::paths::Paths;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Output folder (repo-relative).
const DIR: &str = "assets/clusters/bosch_ddu";
/// This dash's folder name inside the Drive masters folder.
const MASTERS_SUB: &str = "bosch_ddu";

/// Render scale (image px per 4K px). The runtime files are the same pictures box-filtered to
/// 1 px per 4K px, so nothing is scaled up when the cluster draws them.
const MASTER_SCALE: f32 = 2.0;
/// Cycles samples of the housing (the skill's figure for a 4K beauty).
const SAMPLES: u32 = 384;

/// The sprite's frame (local 4K px round the cockpit anchor, y down): `x0, y0, x1, y1`, as the
/// runtime's `HOUSING_FRAME` in `crates/fh6-clusters/src/clusters/bosch_ddu/hardware.rs`.
const FRAME: [f32; 4] = [-440.0, -320.0, 440.0, 240.0];

/// The housing outline, as the cluster's `GEOM` (`crates/fh6-clusters/src/clusters/bosch_ddu/mod.rs`):
/// half width, the top edge at the top corners, the bottom edge, the arch's sagitta over the top
/// edge between the top corners, and the corner radii.
const HALF_W: f32 = 405.0;
const TOP: f32 = -306.0;
const BOTTOM: f32 = 232.0;
const ARCH: f32 = 10.0;
const CORNER_TOP: f32 = 120.0;
const CORNER_BOTTOM: f32 = 44.0;
/// The screen (`x0, y0, x1, y1`): the glass over it stays essentially black.
const SCREEN: [f32; 4] = [-312.0, -204.0, 312.0, 170.0];
/// The ten brow lenses: centres x, the centre y at the middle (each follows the arch, see
/// [`top_led_centre`]) and their size.
const TOP_LED_X: [f32; 10] = [-199.0, -161.0, -95.0, -57.0, -19.0, 19.0, 57.0, 95.0, 161.0, 199.0];
const TOP_LED_Y: f32 = -284.0;
const TOP_LED: [f32; 2] = [28.0, 10.0];
/// The side lenses: five per side at ±x, from the bottom one up at the pitch, and their size.
const SIDE_LED_X: f32 = 359.0;
const SIDE_LED_Y0: f32 = -70.0;
const SIDE_LED_PITCH: f32 = 38.0;
const SIDE_LEDS: usize = 5;
const SIDE_LED: [f32; 2] = [28.0, 10.0];
/// The ambient light sensor's window in the right border: centre and radius.
const SENSOR: [f32; 2] = [359.0, 24.0];
const SENSOR_R: f32 = 8.0;

/// Local px per mm: the outline's 810 px is the DDU 10's 198 mm.
const PX_PER_MM: f32 = 2.0 * HALF_W / 198.0;
/// The body's depth (35 mm).
const DEPTH: f32 = 35.0 * PX_PER_MM;
/// How far the body's back edge shows above its front edge (px): the oblique view from the
/// driver's eye. With the arch's crest at −316 and the frame's top at −320 there are 4 px; this
/// leaves a pixel for the edge's anti-aliasing. With the front edge's [`FRONT_R`] round-over,
/// which turns up to face the eye, the top surface reads as a band of about six px.
const SLIVER: f32 = 3.0;
/// The visible anodised wall round the glass (px): the glass sits this far inside the outline.
const WALL: f32 = 6.0;
/// The body's profile across that wall: the front edge's round-over, the inner lip's round-over
/// down to the glass, and the back edge's.
const FRONT_R: f32 = 3.0;
const LIP_R: f32 = 1.0;
const BACK_R: f32 = 2.0;
/// The glass: its top below the body's front face, its thickness, the round-over of its top edge
/// (the crisp highlight) and the gap to the body's pocket wall.
const GLASS_DROP: f32 = 0.6;
const GLASS_T: f32 = 3.0;
const GLASS_EDGE: f32 = 0.8;
const GLASS_GAP: f32 = 0.05;
/// The pocket under the glass (px below the body's front face).
const POCKET: f32 = GLASS_T + 0.5;
/// How far the lenses and the sensor window stand proud of the glass's top: flush.
const FLUSH: f32 = 0.05;

// The glass's top sits under the front face but above the lip's round-over, so the lip shows
// as a step down to it; the sensor window clears the screen (checked when the crate compiles).
const _: () = {
    assert!(GLASS_DROP > 0.0 && GLASS_DROP < LIP_R);
    assert!(POCKET > GLASS_T && FRONT_R + LIP_R < WALL);
    assert!(SENSOR[0] - SENSOR_R > SCREEN[2] + 20.0);
};

/// The body's finish: dark satin gunmetal anodising. Reflectance (linear F0) and roughness of the
/// metal under the oxide, the oxide coat (weight, roughness, IOR of Al₂O₃), and what one
/// standard deviation of the neutral macro does to the colour (relative) and the roughness.
const BODY_F0: [f32; 3] = [0.16, 0.16, 0.17];
const BODY_ROUGH: f32 = 0.42;
const BODY_COAT: (f32, f32, f32) = (0.3, 0.32, 1.65);
const BODY_COLOR_SD: f32 = 0.08;
const BODY_ROUGH_SD: f32 = 0.035;
/// The faint bump of the macro (strength, distance in px).
const BODY_BUMP: (f32, f32) = (0.12, 0.5);
/// One repeat of the macro on the body (local px). The body shows only as the 6 px wall and the
/// sliver of its top, so the macro's blobs (about 200 texture px) must come out a few px across,
/// as a satin grain along the wall; at one repeat per 640 px they were 32 px blotches.
const BODY_TILE: f32 = 80.0;
/// The black glass: near-black absorbing body, its roughness (very low; just enough that the
/// softbox's reflection has soft edges) and IOR.
const GLASS_COLOR: [f32; 3] = [0.0004, 0.0004, 0.0005];
const GLASS_ROUGH: f32 = 0.07;
const GLASS_IOR: f32 = 1.5;
/// The smoked LED lenses and the sensor window (darker and glossier than the glass). Nothing is
/// mirrored in the face (see [`housing_rig`]), so an unlit lens reads by its own milky grey under
/// the key, as on the press image (about sRGB 60 to 90 there), not by a reflection.
const LENS_COLOR: [f32; 3] = [0.027, 0.027, 0.029];
const LENS_ROUGH: f32 = 0.35;
const SENSOR_COLOR: [f32; 3] = [0.0006, 0.0006, 0.0007];
const SENSOR_ROUGH: f32 = 0.02;
/// The rig's power over the housing: the static plate stays dark (the dash draws the lit parts on
/// top of it).
const HOUSING_LIGHT: f32 = 0.6;
/// The "wall" softbox that stands in for the hero rig's overhead sky: its elevation, its distance
/// and size (as shares of the frame's span) and its power before [`HOUSING_LIGHT`]. In the
/// top-down ortho view the flat glass mirrors only what is straight above each point, so a
/// softbox over the face lays a flat grey veil on the screen (0.017 with the hero sky). This one
/// stands to the side, clear of the screen at every swing azimuth: the satin body's wide lobe
/// still catches it, the glass does not, and the face stays black with crisp rim lines.
const WALL_EL: f32 = 66.0;
const WALL_DIST: f32 = 2.0;
const WALL_SIZE: f32 = 0.45;
const WALL_POWER: f32 = 1.0;
/// How far every overhead softbox's footprint stays from the screen (px), whatever the azimuth.
const SCREEN_CLEAR: f32 = 40.0;
/// The brightness limits the build checks the runtime files against (relative luminance): the
/// sprite's 95th percentile, and the screen area's 99th (the screen is OLED black: sRGB 12).
const LIMIT_P95: f32 = 0.2;
const LIMIT_SCREEN_P99: f32 = 0.004;

/// The ComfyUI macro. The prompt names only the surface finish.
const BEAD_BLAST: Recipe = Recipe {
    name: "bosch_ddu_bead_blast",
    prompt: "extreme macro photograph of bead blasted anodised aluminium, fine uniform matte satin \
             grain of tiny glass bead impact pits, a completely flat metal surface filling the whole \
             frame, top-down, even soft diffuse light, seamless tileable texture, physically based \
             material scan, neutral grey",
    negative: "machined lines, brushed streaks, scratches, dents, scuffs, screws, holes, edges, \
               corners, reflections, specular highlights, colour tint, rust, corrosion, dust, \
               fingerprints, fabric, leather pores, carbon weave, gradient",
    seed: 71_207,
};
/// Size of the macro (master and the one Blender reads).
const TEXTURE_SIZE: usize = 4096;
/// One standard deviation of the neutral macro round its mid-grey mean (sRGB-encoded units).
const TEXTURE_SPREAD: f32 = 0.08;
/// The window (radius, texture px) over which the macro's mean and spread are evened out, so the
/// smoother cross that SDXL's seam repaint leaves reads like the rest of the surface.
const TEXTURE_WINDOW: usize = 48;
/// Version of the normalisation; it is part of the neutral file's name, so a change here
/// re-renders the scene that reads it.
const TEXTURE_NORMALISE: &[u8] = b"bosch_ddu neutral bead blast v1 (local spread)";

/// bpy added to the hero script before it builds the scene: the two material kinds this recipe
/// uses that the hero cookbook has no builder for. It uses the hero script's helpers
/// (`principled`, `coords`, `M`, `vec3`, `noise`, `bump`, `setin`) and registers in `BUILDERS`.
pub const BPY_MATERIALS: &str = r#"
def mat_ddu_anodised(d):
    # Bead-blasted anodised aluminium: metal of the anodised reflectance under a thin oxide coat.
    # The neutral ComfyUI macro (mean 0.5, one deviation = 'spread') varies the colour and the
    # roughness and gives the metal and the coat a faint bump, mapped flat in object space (local
    # px), one repeat per 'tile_px'.
    m, nt, bs = principled(d['name'])
    x, y, z, r, th = coords(nt)
    c = d.get('color', (0.16, 0.16, 0.17))
    rough = float(d.get('rough', 0.42))
    setin(bs, 'Metallic', 1.0)
    tex = d.get('texture')
    if tex and os.path.isfile(tex):
        im = node(nt, 'ShaderNodeTexImage', interpolation='Cubic', extension='REPEAT')
        im.image = bpy.data.images.load(tex, check_existing=True)
        im.image.colorspace_settings.name = 'Non-Color'
        tile = float(d.get('tile_px', 640.0))
        lk(nt, vec3(nt, M(nt, 'DIVIDE', x, tile), M(nt, 'DIVIDE', y, tile), 0.0), im.inputs['Vector'])
        bw = node(nt, 'ShaderNodeRGBToBW')
        lk(nt, im.outputs['Color'], bw.inputs[0])
        h = bw.outputs[0]
        zs = M(nt, 'DIVIDE', M(nt, 'SUBTRACT', h, 0.5), max(float(d.get('spread', 0.08)), 1e-3))
    else:
        log('ddu_anodised: texture missing, procedural grain instead', tex)
        h = noise(nt, vec3(nt, x, y, z), 1.0 / 1.2, 6.0)
        zs = M(nt, 'MULTIPLY', M(nt, 'SUBTRACT', h, 0.5), 6.0)
    k = M(nt, 'MAXIMUM', M(nt, 'MULTIPLY_ADD', zs, float(d.get('color_sd', 0.08)), 1.0), 0.0)
    lk(nt, vec3(nt, M(nt, 'MULTIPLY', k, float(c[0])), M(nt, 'MULTIPLY', k, float(c[1])), M(nt, 'MULTIPLY', k, float(c[2]))), bs.inputs['Base Color'])
    rr = M(nt, 'MULTIPLY_ADD', zs, float(d.get('rough_sd', 0.035)), rough)
    lk(nt, M(nt, 'MINIMUM', M(nt, 'MAXIMUM', rr, 0.05), 1.0), bs.inputs['Roughness'])
    b = bump(nt, bs, h, float(d.get('bump', 0.12)), float(d.get('bump_px', 0.5)))
    setin(bs, 'Coat Weight', float(d.get('coat', 0.3)))
    setin(bs, 'Coat Roughness', float(d.get('coat_rough', 0.32)))
    setin(bs, 'Coat IOR', float(d.get('coat_ior', 1.65)))
    try:
        lk(nt, b.outputs['Normal'], bs.inputs['Coat Normal'])
    except Exception as e:
        log('coat normal', e)
    return m

def mat_ddu_glass(d):
    # Black glass: a smooth dielectric of the given IOR over a near-black, absorbing body.
    m, nt, bs = principled(d['name'])
    setin(bs, 'Base Color', d.get('color', (0.0015, 0.0015, 0.0018)))
    setin(bs, 'Metallic', 0.0)
    setin(bs, 'Roughness', float(d.get('rough', 0.05)))
    setin(bs, 'IOR', float(d.get('ior', 1.5)))
    return m

BUILDERS['ddu_anodised'] = mat_ddu_anodised
BUILDERS['ddu_glass'] = mat_ddu_glass
"#;

/// Where [`BPY_MATERIALS`] goes in the hero script: just before it builds the materials.
const HERO_ANCHOR: &str = "sc = reset()\nfor d in P['materials']:";

/// Builds the macro and the housing, writes the runtime files, the cluster's ledger and the
/// previews.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let scratch = ctx.scratch("bosch_ddu")?;
    let tex_dir = scratch.join("textures");
    let mut rows: Vec<String> = Vec::new();
    let mut masters: Vec<(PathBuf, String)> = Vec::new();

    // 1. ComfyUI: the bead-blast macro, made neutral so the body's brightness comes from the
    //    material and not from how dark SDXL drew the metal.
    let sdxl = texture::make_texture(&ctx.paths, &BEAD_BLAST, TEXTURE_SIZE, &tex_dir)?;
    let neutral = neutral_texture(&sdxl, &tex_dir)?;
    masters.push((sdxl.clone(), "bead-blast macro (ComfyUI, seamless 4096 px)".into()));
    masters.push((
        neutral.clone(),
        "bead-blast macro made neutral (luminance only, mid-grey mean, fixed spread): the file Blender reads".into(),
    ));
    if let Some(m) = &ctx.paths.masters {
        for (path, _) in &masters {
            copy_master(m, "comfyui", path)?;
        }
    }
    rows.push(format!(
        "| (no runtime file: rendered into the housing) | seamless bead-blast texture | {s}x{s} | master | comfyui:bosch_ddu::BEAD_BLAST (SDXL base 1.0 + refiner when installed, 1024², dpmpp_2m karras, half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed={seed} prompt=\"{prompt}\", then neutral (luminance, mean 0.5, deviation {TEXTURE_SPREAD} over a {TEXTURE_WINDOW} px window) | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the anodised body's colour and roughness variation and its faint bump (one repeat per {BODY_TILE} px). Master `{file}` |",
        s = TEXTURE_SIZE,
        seed = BEAD_BLAST.seed,
        prompt = squash(BEAD_BLAST.prompt),
        file = file_name(&neutral),
    ));

    // 2. Blender: the housing at the three key azimuths.
    let scene = housing_scene(&neutral);
    let beauty = render(&scene, &ctx.paths, &scratch.join("housing"))?;
    if beauty.len() != SWING.len() {
        return Err(tool_err(
            "bosch_ddu",
            format!("{} housing renders, expected {}", beauty.len(), SWING.len()),
        ));
    }
    let (w, h) = scene.frame.resolution();
    let k = MASTER_SCALE.round().max(1.0) as u32;
    for (i, master) in beauty.iter().enumerate() {
        let name = format!("housing_{i}.png");
        let small = box_filter(&img::load(master)?, k as usize);
        let bytes = img::save_png(&small, &out_dir.join(&name), false, true)?;
        let b = brightness(&small);
        if b.p95 > LIMIT_P95 || b.screen_p99 > LIMIT_SCREEN_P99 {
            eprintln!(
                "  bosch_ddu {name}: brighter than the static limits (p95 {:.3} > {LIMIT_P95}, or screen p99 {:.4} > {LIMIT_SCREEN_P99}); lower HOUSING_LIGHT",
                b.p95, b.screen_p99
            );
        }
        rows.push(format!(
            "| `{name}` | the DDU 10 housing: anodised body, black glass face, 10 + 5 + 5 smoked lenses, light sensor; key at {az}° (sRGB, straight alpha) | {rw}x{rh} | {kb} KB | blender:hero (bosch_ddu::housing_scene, Cycles {SAMPLES} samples, ortho at {MASTER_SCALE} px per 4K px over x {fx0}..{fx1}, y {fy0}..{fy1}, box-filtered to 1:1; the housing's rig (no softbox over the face, no HDRI) at {HOUSING_LIGHT} power; body sheared {SLIVER} px for the driver's eye) seed=0; luminance p95 {p95:.3}, screen p99 {sp:.4} | MIT (asset-forge) | drawn at full strength under the live screen and LEDs; the cluster cross-fades the azimuths with the virtual light. Master `housing/{master}` |",
            az = key_azimuth(&scene, i),
            rw = w / k,
            rh = h / k,
            kb = bytes / 1024,
            fx0 = FRAME[0],
            fx1 = FRAME[2],
            fy0 = FRAME[1],
            fy1 = FRAME[3],
            p95 = b.p95,
            sp = b.screen_p99,
            master = file_name(master),
        ));
        masters.push((master.clone(), format!("housing, key {}°", key_azimuth(&scene, i))));
    }

    // 3. Previews beside the masters.
    let previews = write_previews(ctx.paths.masters.as_deref(), &masters, &scratch.join("previews"))?;

    // 4. The cluster's own ledger.
    let mut md = String::from(
        "# BoschDdu assets\n\nBuilt by `cargo run --release -p asset-forge -- --lane cloud --masters bosch_ddu build --set bosch_ddu` (recipe `tools/asset-forge/src/recipes/bosch_ddu.rs`, hero library `tools/asset-forge/src/hero/`). The cluster draws its procedural housing when the files are missing. The housing renders cover the runtime's `HOUSING_FRAME`, x −440..440, y −320..240 in local 4K px, at 1 px per px; `housing_1` is the key at the smaller azimuth, as the runtime blends it.\n\n## Runtime files (in git)\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in &rows {
        let _ = writeln!(md, "{r}");
    }
    md.push_str("\n## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/bosch_ddu/`)\n\nCopied by `--masters bosch_ddu`: the Blender outputs into `hero_bosch_ddu_housing/`, the ComfyUI textures into `comfyui/`, and a `*_preview.png` beside them for the tracker.\n\n| Master | What it is |\n|---|---|\n");
    for (path, what) in &masters {
        let _ = writeln!(md, "| `{}` | {what} |", file_name(path));
    }
    let _ = writeln!(md, "\n{previews} preview(s) written.");
    img::write_file(&ctx.abs(&format!("{DIR}/ASSETS.md")), md.as_bytes())?;
    println!("  wrote {DIR}/ ({} runtime files + ASSETS.md)", rows.len() - 1);
    Ok(())
}

/// The housing outline (local px, y down), clockwise on screen from the arch's left end: the
/// arch, the top-right corner, the right side, the bottom corners and edge, the left side and the
/// top-left corner. No point repeats.
fn housing_outline() -> Vec<[f32; 2]> {
    const ARCH_SEGS: usize = 96;
    const TOP_SEGS: usize = 40;
    const BOTTOM_SEGS: usize = 16;
    let span = HALF_W - CORNER_TOP;
    let mut pts = Vec::new();
    for i in 0..ARCH_SEGS {
        let x = -span + 2.0 * span * i as f32 / ARCH_SEGS as f32;
        pts.push([x, TOP - ARCH + arch_drop(x)]);
    }
    let mut corner = |cx: f32, cy: f32, r: f32, from_deg: f32, segs: usize| {
        for i in 0..segs {
            let a = (from_deg + 90.0 * i as f32 / segs as f32).to_radians();
            pts.push([cx + r * a.cos(), cy + r * a.sin()]);
        }
    };
    // Each corner runs 90° clockwise on screen; the straight sides start where it ends.
    corner(span, TOP + CORNER_TOP, CORNER_TOP, -90.0, TOP_SEGS);
    let low = HALF_W - CORNER_BOTTOM;
    corner(low, BOTTOM - CORNER_BOTTOM, CORNER_BOTTOM, 0.0, BOTTOM_SEGS);
    corner(-low, BOTTOM - CORNER_BOTTOM, CORNER_BOTTOM, 90.0, BOTTOM_SEGS);
    corner(-span, TOP + CORNER_TOP, CORNER_TOP, 180.0, TOP_SEGS);
    pts
}

/// The circle the arch follows: its radius, through the crest and the arch's ends.
fn arch_radius() -> f32 {
    let span = HALF_W - CORNER_TOP;
    (span * span + ARCH * ARCH) / (2.0 * ARCH)
}

/// How far the arch drops below its crest at `x` (0 at the middle, [`ARCH`] at its ends).
fn arch_drop(x: f32) -> f32 {
    let r = arch_radius();
    r - (r * r - x * x).max(0.0).sqrt()
}

/// Brow lens `i`'s centre: on the arch, like the runtime's `top_led_rect`.
fn top_led_centre(i: usize) -> [f32; 2] {
    let x = TOP_LED_X[i];
    [x, TOP_LED_Y + arch_drop(x)]
}

/// Side lens `i`'s centre (0 at the bottom) on the left (`side` −1) or right (+1).
fn side_led_centre(side: f32, i: usize) -> [f32; 2] {
    [side * SIDE_LED_X, SIDE_LED_Y0 - SIDE_LED_PITCH * i as f32]
}

/// `pts` (a convex outline, clockwise on screen) moved `d` px inwards: each point along the
/// bisector of its two edges' inward normals, mitred so both edges move exactly `d`.
fn offset(pts: &[[f32; 2]], d: f32) -> Vec<[f32; 2]> {
    let n = pts.len();
    let normal = |a: [f32; 2], b: [f32; 2]| {
        let (tx, ty) = (b[0] - a[0], b[1] - a[1]);
        let l = (tx * tx + ty * ty).sqrt().max(1e-6);
        // Clockwise on screen with y down: the inside is to the right of the direction of travel.
        [-ty / l, tx / l]
    };
    (0..n)
        .map(|i| {
            let p = pts[i];
            let a = normal(pts[(i + n - 1) % n], p);
            let b = normal(p, pts[(i + 1) % n]);
            let (mx, my) = (a[0] + b[0], a[1] + b[1]);
            let ml = (mx * mx + my * my).sqrt().max(1e-6);
            let m = [mx / ml, my / ml];
            let k = d / (m[0] * b[0] + m[1] * b[1]).max(0.2);
            [p[0] + m[0] * k, p[1] + m[1] * k]
        })
        .collect()
}

/// The oblique view: a point `z` px behind the face plane (z < 0) moves up the screen in
/// proportion, so the body's back edge shows [`SLIVER`] px above its front edge.
fn shear(x: f32, y: f32, z: f32) -> [f32; 3] {
    [x, y + z * SLIVER / DEPTH, z]
}

/// The body's cross-section from the back edge round to the pocket under the glass: `(inset from
/// the outline, z)` in px, z = 0 at the front face. The back edge's round-over, the side wall,
/// the front edge's round-over, the flat front face, the inner lip's round-over and the pocket
/// wall; tangent-continuous, so the mesh can shade smooth.
fn body_profile() -> Vec<(f32, f32)> {
    const SEGS: usize = 8;
    let quarter = |t: usize| core::f32::consts::FRAC_PI_2 * t as f32 / SEGS as f32;
    let mut p = Vec::new();
    for t in 0..=SEGS {
        let a = quarter(t);
        p.push((BACK_R - BACK_R * a.sin(), -DEPTH + BACK_R - BACK_R * a.cos()));
    }
    for t in 0..=SEGS {
        let a = quarter(t);
        p.push((FRONT_R - FRONT_R * a.cos(), -FRONT_R + FRONT_R * a.sin()));
    }
    for t in 0..=SEGS / 2 {
        let a = 2.0 * quarter(t);
        p.push((WALL - LIP_R + LIP_R * a.sin(), -LIP_R + LIP_R * a.cos()));
    }
    p.push((WALL, -POCKET));
    p
}

/// The body as one closed mesh in local px: a ring of the offset outline per profile point,
/// sheared for the oblique view, quads between the rings, and the back and the pocket floor
/// closed.
fn body_mesh() -> Mesh {
    let outline = housing_outline();
    let prof = body_profile();
    let n = outline.len();
    let mut mesh = Mesh {
        smooth: true,
        ..Mesh::default()
    };
    for &(d, z) in &prof {
        for p in offset(&outline, d) {
            mesh.verts.push(shear(p[0], p[1], z));
        }
    }
    let at = |ring: usize, k: usize| (ring * n + k % n) as u32;
    for ring in 0..prof.len() - 1 {
        for k in 0..n {
            mesh.faces
                .push(vec![at(ring, k), at(ring, k + 1), at(ring + 1, k + 1), at(ring + 1, k)]);
            mesh.mats.push(0);
        }
    }
    mesh.faces.push((0..n).rev().map(|k| at(0, k)).collect());
    mesh.faces.push((0..n).map(|k| at(prof.len() - 1, k)).collect());
    mesh.mats.extend([0, 0]);
    mesh
}

/// A flush lens or window on the glass: a thin slab whose top stands [`FLUSH`] px proud.
fn on_glass(name: &str, outline: Outline, mat: Mat) -> Part {
    let top = -GLASS_DROP + FLUSH;
    Part::extrude(name, outline, top - 0.35, 0.35, 0.15, mat)
}

/// The housing scene over [`FRAME`], with the body textured by the neutral macro `texture`.
fn housing_scene(texture: &Path) -> Scene {
    let [x0, y0, x1, y1] = FRAME;
    let mut s = Scene::new(
        "bosch_ddu_housing",
        Frame::rect(x0, y0, x1 - x0, y1 - y0).scaled(MASTER_SCALE),
    );
    s.rig = housing_rig();
    s.samples = SAMPLES;
    s.passes = Passes {
        beauty: true,
        swing: true,
        data: false,
        ao: false,
        rmh: false,
        emission: false,
    };
    let body = Mat::new("ddu_anodised", "ddu_body")
        .color(BODY_F0)
        .rough(BODY_ROUGH)
        .set("texture", texture.to_string_lossy().to_string())
        .set("tile_px", BODY_TILE)
        .set("spread", TEXTURE_SPREAD)
        .set("color_sd", BODY_COLOR_SD)
        .set("rough_sd", BODY_ROUGH_SD)
        .set("bump", BODY_BUMP.0)
        .set("bump_px", BODY_BUMP.1)
        .set("coat", BODY_COAT.0)
        .set("coat_rough", BODY_COAT.1)
        .set("coat_ior", BODY_COAT.2);
    let glass = |name: &str, color: [f32; 3], rough: f32| {
        Mat::new("ddu_glass", name)
            .color(color)
            .rough(rough)
            .set("ior", GLASS_IOR)
    };
    s.add(Part::mesh("body", [0.0, 0.0], 0.0, body_mesh(), vec![body]));
    let face = Outline {
        points: offset(&housing_outline(), WALL + GLASS_GAP),
    };
    s.add(Part::extrude(
        "glass",
        face,
        -GLASS_T,
        GLASS_T - GLASS_DROP,
        GLASS_EDGE,
        glass("ddu_glass", GLASS_COLOR, GLASS_ROUGH),
    ));
    let lens = glass("ddu_lens", LENS_COLOR, LENS_ROUGH);
    for i in 0..TOP_LED_X.len() {
        let [cx, cy] = top_led_centre(i);
        let pill = Outline::rrect(cx, cy, TOP_LED[0], TOP_LED[1], TOP_LED[1] * 0.5, 8);
        s.add(on_glass(&format!("led_top_{i:02}"), pill, lens.clone()));
    }
    for side in [-1.0_f32, 1.0] {
        for i in 0..SIDE_LEDS {
            let [cx, cy] = side_led_centre(side, i);
            let pill = Outline::rrect(cx, cy, SIDE_LED[0], SIDE_LED[1], SIDE_LED[1] * 0.5, 8);
            let tag = if side < 0.0 { 'l' } else { 'r' };
            s.add(on_glass(&format!("led_{tag}{i}"), pill, lens.clone()));
        }
    }
    let window = Outline::rrect(SENSOR[0], SENSOR[1], 2.0 * SENSOR_R, 2.0 * SENSOR_R, SENSOR_R, 12);
    s.add(on_glass(
        "sensor",
        window,
        glass("ddu_sensor", SENSOR_COLOR, SENSOR_ROUGH),
    ));
    s
}

/// The housing's rig: the hero rig's key, second sky, rim and kick, with the "wall" softbox (see
/// [`WALL_EL`]) in place of its overhead sky and no studio HDRI, whose zenith the glass would
/// mirror. Aimed at the frame's centre; the key leads, so the swing passes turn every light.
fn housing_rig() -> Rig {
    let span = (FRAME[2] - FRAME[0]).max(FRAME[3] - FRAME[1]);
    let centre = [(FRAME[0] + FRAME[2]) * 0.5, (FRAME[1] + FRAME[3]) * 0.5];
    let mut rig = Rig::hero(span, SWING[0]).at(centre[0], centre[1], 0.0);
    for l in &mut rig.lights {
        if l.name == "sky" {
            *l = Light::soft("wall", l.az, WALL_EL, span * WALL_DIST, span * WALL_SIZE, WALL_POWER);
        }
    }
    rig.hdri = 0.0;
    rig.power(HOUSING_LIGHT)
}

/// The hero script with [`BPY_MATERIALS`] added before it builds the scene.
fn script() -> Result<String> {
    let hero = crate::hero::bpy::HERO;
    if hero.matches(HERO_ANCHOR).count() != 1 {
        return Err(tool_err(
            "bosch_ddu",
            "the hero bpy changed: the point where the recipe adds its materials is gone",
        ));
    }
    Ok(hero.replacen(HERO_ANCHOR, &format!("{BPY_MATERIALS}\n{HERO_ANCHOR}"), 1))
}

/// Renders the scene's beauty passes into `out` with the hero script plus [`BPY_MATERIALS`], as
/// `Scene::render` does: skipped when the same scene was already rendered there (fingerprint of
/// the script and the parameters, which name the texture by its own fingerprint), retried once
/// after a failure (another job may have held the card). With `--masters` a cached render is
/// copied to the masters folder too (a fresh one is copied by `blender::run`). Returns the beauty
/// files, rest key first.
fn render(scene: &Scene, paths: &Paths, out: &Path) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(out).map_err(|e| io_err(out, e))?;
    let source = script()?;
    let params = scene.to_json(out);
    let text = serde_json::to_string(&params).unwrap_or_default();
    let fp = crate::ledger::fingerprint(&[source.as_bytes(), crate::recipes::PRELUDE.as_bytes(), text.as_bytes()]);
    let fp_path = out.join(".fp");
    let n = if scene.passes.swing { 1 + scene.swing.len() } else { 1 };
    let beauty: Vec<PathBuf> = (0..n).map(|i| out.join(format!("beauty_{i}.png"))).collect();
    let fresh = std::fs::read_to_string(&fp_path).is_ok_and(|s| s.trim() == fp) && beauty.iter().all(|p| p.is_file());
    let name = format!("hero_{}", scene.name);
    if fresh {
        println!("  {name}: cached");
        if let Some(m) = &paths.masters {
            let copied = crate::lane::sink_dir(m, &name, out)?;
            println!("  masters: {copied} file(s) from {name} copied to {}", m.display());
        }
    } else {
        println!("  blender {name}");
        if let Err(first) = crate::blender::run(paths, &name, &source, &params) {
            eprintln!("  {name}: retrying once after: {first}");
            std::thread::sleep(std::time::Duration::from_secs(20));
            crate::blender::run(paths, &name, &source, &params)?;
        }
        std::fs::write(&fp_path, &fp).map_err(|e| io_err(&fp_path, e))?;
    }
    Ok(beauty)
}

/// The key azimuth (degrees) of beauty pass `i` of `scene`.
fn key_azimuth(scene: &Scene, i: usize) -> f32 {
    if i == 0 {
        SWING[0]
    } else {
        scene.swing.get(i - 1).copied().unwrap_or(SWING[0])
    }
}

/// Box-filters a master (straight alpha, sRGB-encoded) by `k`: colour averaged in linear light
/// weighted by coverage, so edges keep their colour.
fn box_filter(src: &Img, k: usize) -> Img {
    let k = k.max(1);
    let (w, h) = (src.w / k, src.h / k);
    let alpha = src.c > 3;
    Img::from_fn(w, h, 4, |x, y| {
        let mut acc = [0.0_f32; 4];
        for dy in 0..k {
            for dx in 0..k {
                let (sx, sy) = (x * k + dx, y * k + dy);
                let a = if alpha { src.get(sx, sy, 3) } else { 1.0 };
                for (ch, v) in acc.iter_mut().enumerate().take(3) {
                    *v += img::srgb_decode(src.get(sx, sy, ch.min(src.c - 1))) * a;
                }
                acc[3] += a;
            }
        }
        let inv = if acc[3] > 1e-6 { 1.0 / acc[3] } else { 0.0 };
        [
            img::srgb_encode(acc[0] * inv),
            img::srgb_encode(acc[1] * inv),
            img::srgb_encode(acc[2] * inv),
            acc[3] / (k * k) as f32,
        ]
    })
}

/// How bright a runtime file is (relative luminance, linear): the 95th percentile over the
/// sprite, and the 99th over the screen area.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Brightness {
    p95: f32,
    screen_p99: f32,
}

/// Measures `im`, a runtime file over [`FRAME`] (pixels with alpha below one half are left out).
fn brightness(im: &Img) -> Brightness {
    let px = (FRAME[2] - FRAME[0]) / im.w.max(1) as f32;
    let mut all = Vec::new();
    let mut screen = Vec::new();
    for y in 0..im.h {
        for x in 0..im.w {
            if im.c > 3 && im.get(x, y, 3) < 0.5 {
                continue;
            }
            let ch = |c: usize| img::srgb_decode(im.get(x, y, c.min(im.c - 1)));
            let l = 0.2126 * ch(0) + 0.7152 * ch(1) + 0.0722 * ch(2);
            all.push(l);
            let (lx, ly) = (FRAME[0] + (x as f32 + 0.5) * px, FRAME[1] + (y as f32 + 0.5) * px);
            if lx > SCREEN[0] && lx < SCREEN[2] && ly > SCREEN[1] && ly < SCREEN[3] {
                screen.push(l);
            }
        }
    }
    Brightness {
        p95: percentile(&mut all, 0.95),
        screen_p99: percentile(&mut screen, 0.99),
    }
}

/// The `q` quantile of `v` (0 when empty).
fn percentile(v: &mut [f32], q: f32) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f32::total_cmp);
    let i = ((v.len() - 1) as f32 * q.clamp(0.0, 1.0)).round() as usize;
    v[i.min(v.len() - 1)]
}

/// A neutral copy of the SDXL macro in `dir`: luminance only, its mean at mid grey and one
/// standard deviation at [`TEXTURE_SPREAD`] (clamped at 2.5 deviations), both evened out over
/// [`TEXTURE_WINDOW`]. It is named by a fingerprint of the source and of these settings, so the
/// Blender scene re-renders when either changes.
fn neutral_texture(src: &Path, dir: &Path) -> Result<PathBuf> {
    let bytes = std::fs::read(src).map_err(|e| io_err(src, e))?;
    let fp = crate::ledger::fingerprint(&[
        &bytes,
        TEXTURE_NORMALISE,
        &TEXTURE_SPREAD.to_le_bytes(),
        &TEXTURE_WINDOW.to_le_bytes(),
    ]);
    let out = dir.join(format!("bosch_ddu_bead_blast_neutral_{}.png", &fp[..12.min(fp.len())]));
    if out.is_file() {
        return Ok(out);
    }
    let flat = neutralise(&img::load(src)?);
    img::save_png(&flat, &out, false, true)?;
    Ok(out)
}

/// The luminance of `src` with its local mean moved to 0.5 and its local deviation to
/// [`TEXTURE_SPREAD`] (wrapping, as the texture tiles).
fn neutralise(src: &Img) -> Img {
    let (w, h) = (src.w, src.h);
    let lum: Vec<f32> = (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            let ch = |c: usize| src.get(x, y, c.min(src.c.max(1) - 1));
            img::luma(ch(0), ch(1), ch(2))
        })
        .collect();
    let mean = box_blur_wrap(&lum, w, h, TEXTURE_WINDOW);
    let dev: Vec<f32> = lum.iter().zip(&mean).map(|(l, m)| (l - m) * (l - m)).collect();
    let var = box_blur_wrap(&dev, w, h, TEXTURE_WINDOW);
    Img::from_fn(w, h, 3, |x, y| {
        let i = y * w + x;
        let sd = var[i].max(0.0).sqrt().max(1e-4);
        let z = ((lum[i] - mean[i]) / sd).clamp(-2.5, 2.5);
        let v = 0.5 + z * TEXTURE_SPREAD;
        [v, v, v, 1.0]
    })
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

/// Copies `path` into `<masters>/<sub>/`.
fn copy_master(masters: &Path, sub: &str, path: &Path) -> Result<()> {
    let dest = masters.join(sub);
    std::fs::create_dir_all(&dest).map_err(|e| io_err(&dest, e))?;
    let to = dest.join(file_name(path));
    std::fs::copy(path, &to).map_err(|e| io_err(&to, e))?;
    Ok(())
}

/// Writes a `<job>_<stem>_preview.png` (at most 1024 px wide, flattened on black) for every
/// master into `dir`, and into the masters folder (`--masters`, else this dash's Drive folder
/// when it is mounted). Returns how many were written.
fn write_previews(masters: Option<&Path>, list: &[(PathBuf, String)], dir: &Path) -> Result<usize> {
    let drive = masters
        .map(Path::to_path_buf)
        .or_else(|| crate::lane::masters_dir(MASTERS_SUB).ok());
    let mut n = 0;
    for (path, _) in list {
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
            let ch = |c: usize| small.get(x, y, c.min(small.c - 1)) * a;
            [ch(0), ch(1), ch(2), 1.0]
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

    const EPS: f32 = 1e-3;

    /// A runtime source file of the cluster.
    fn runtime(file: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/fh6-clusters/src/clusters/bosch_ddu")
            .join(file);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    fn bounds(points: impl IntoIterator<Item = [f32; 2]>) -> [f32; 4] {
        let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for [x, y] in points {
            b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
        }
        b
    }

    /// Whether `p` is inside the convex, clockwise-on-screen outline `o` by at least `margin`.
    fn inside(o: &[[f32; 2]], p: [f32; 2], margin: f32) -> bool {
        let n = o.len();
        (0..n).all(|i| {
            let (a, b) = (o[i], o[(i + 1) % n]);
            let (tx, ty) = (b[0] - a[0], b[1] - a[1]);
            let l = (tx * tx + ty * ty).sqrt();
            // Distance to the edge's line, positive inside (to the right of travel, y down).
            ((p[0] - a[0]) * -ty + (p[1] - a[1]) * tx) / l >= margin
        })
    }

    #[test]
    fn the_frame_and_the_outline_match_the_runtime_numbers() {
        let hw = runtime("hardware.rs");
        let f = |v: f32| format!("{v:.1}");
        for want in [
            format!("min: Vec2 {{ x: {}, y: {} }}", f(FRAME[0]), f(FRAME[1])),
            format!("max: Vec2 {{ x: {}, y: {} }}", f(FRAME[2]), f(FRAME[3])),
            format!("const WALL: f32 = {};", f(WALL)),
        ] {
            assert!(hw.contains(&want), "hardware.rs has `{want}`");
        }
        let geom = runtime("mod.rs");
        let top_x: Vec<String> = TOP_LED_X.iter().map(|&x| f(x)).collect();
        for want in [
            format!("half_w: {},", f(HALF_W)),
            format!("top: {},", f(TOP)),
            format!("bottom: {},", f(BOTTOM)),
            format!("arch: {},", f(ARCH)),
            format!("corner_top: {},", f(CORNER_TOP)),
            format!("corner_bottom: {},", f(CORNER_BOTTOM)),
            format!("sensor: Vec2 {{ x: {}, y: {} }},", f(SENSOR[0]), f(SENSOR[1])),
            format!("sensor_r: {},", f(SENSOR_R)),
            format!("min: Vec2 {{ x: {}, y: {} }}", f(SCREEN[0]), f(SCREEN[1])),
            format!("max: Vec2 {{ x: {}, y: {} }}", f(SCREEN[2]), f(SCREEN[3])),
            format!("top_led_x: [{}],", top_x.join(", ")),
            format!("top_led_y: {},", f(TOP_LED_Y)),
            format!("top_led: Vec2 {{ x: {}, y: {} }},", f(TOP_LED[0]), f(TOP_LED[1])),
            format!("side_led_x: {},", f(SIDE_LED_X)),
            format!("side_led_y0: {},", f(SIDE_LED_Y0)),
            format!("side_led_pitch: {},", f(SIDE_LED_PITCH)),
            format!("side_led: Vec2 {{ x: {}, y: {} }},", f(SIDE_LED[0]), f(SIDE_LED[1])),
        ] {
            assert!(geom.contains(&want), "mod.rs GEOM has `{want}`");
        }
        // The runtime files are the frame at 1 px per 4K px: 880 x 560.
        let s = housing_scene(Path::new("t.png"));
        let (w, h) = s.frame.resolution();
        let k = MASTER_SCALE as u32;
        assert_eq!((w / k, h / k), (880, 560));
        assert_eq!((w % k, h % k), (0, 0));
        assert!((s.frame.x - FRAME[0]).abs() < EPS && (s.frame.y - FRAME[1]).abs() < EPS);
        // 810 px is 198 mm.
        assert!((PX_PER_MM - 4.0909).abs() < 1e-3 && (DEPTH - 143.18).abs() < 0.01);
    }

    #[test]
    fn the_outline_has_the_arch_the_corners_and_the_edges() {
        let o = housing_outline();
        let b = bounds(o.iter().copied());
        assert!((b[0] + HALF_W).abs() < EPS && (b[2] - HALF_W).abs() < EPS, "{b:?}");
        assert!((b[1] - (TOP - ARCH)).abs() < EPS, "crest at -316: {b:?}");
        assert!((b[3] - BOTTOM).abs() < EPS, "{b:?}");
        assert!(Outline { points: o.clone() }.area() > 0.0, "clockwise on screen");
        // The arch's ends meet the top corners at y -306, its crest is at -316 (sagitta 10), and
        // every arch point lies on its circle.
        let span = HALF_W - CORNER_TOP;
        assert!((arch_drop(span) - ARCH).abs() < EPS && arch_drop(0.0).abs() < EPS);
        assert!((arch_radius() - (285.0 * 285.0 + 100.0) / 20.0).abs() < EPS);
        let centre = TOP - ARCH + arch_radius();
        for p in o.iter().filter(|p| p[0].abs() <= span && p[1] < TOP + EPS) {
            let r = (p[0] * p[0] + (p[1] - centre).powi(2)).sqrt();
            assert!((r - arch_radius()).abs() < 0.01, "{p:?}");
        }
        assert!(o.iter().any(|p| (p[0] + span).abs() < EPS && (p[1] - TOP).abs() < EPS));
        // No point repeats.
        for (i, p) in o.iter().enumerate() {
            let q = o[(i + 1) % o.len()];
            assert!((p[0] - q[0]).abs() + (p[1] - q[1]).abs() > 0.5, "{i}");
        }
    }

    #[test]
    fn the_offset_moves_every_edge_inwards_by_the_same_amount() {
        let o = housing_outline();
        let g = offset(&o, WALL);
        let b = bounds(g.iter().copied());
        let want = [-HALF_W + WALL, TOP - ARCH + WALL, HALF_W - WALL, BOTTOM - WALL];
        for (got, want) in b.iter().zip(want) {
            assert!((got - want).abs() < 0.05, "{b:?}");
        }
        // The top corners' radius shrinks by the offset.
        let span = HALF_W - CORNER_TOP;
        for p in g.iter().filter(|p| p[0] > span + 1.0 && p[1] < TOP + CORNER_TOP - 1.0) {
            let r = ((p[0] - span).powi(2) + (p[1] - TOP - CORNER_TOP).powi(2)).sqrt();
            assert!((r - (CORNER_TOP - WALL)).abs() < 0.05, "{p:?} r {r}");
        }
    }

    #[test]
    fn the_brow_lenses_follow_the_arch() {
        let span = HALF_W - CORNER_TOP;
        let r = arch_radius();
        for (i, &lx) in TOP_LED_X.iter().enumerate() {
            let [x, y] = top_led_centre(i);
            assert!((x - lx).abs() < EPS);
            // d(x) = r - sqrt(r² - x²), r = (285² + 10²) / 20.
            let d = r - (r * r - x * x).sqrt();
            assert!((y - (TOP_LED_Y + d)).abs() < EPS, "lens {i} at {y}");
            // Every lens sits the same distance under the arch.
            assert!((y - (TOP - ARCH + arch_drop(x)) - (TOP_LED_Y - (TOP - ARCH))).abs() < EPS);
            assert!(x.abs() < span);
        }
        // The middle pair sits 32 px under the crest, the outer pair a little lower.
        assert!((top_led_centre(4)[1] - (-284.0 + arch_drop(19.0))).abs() < EPS);
        assert!(top_led_centre(0)[1] > top_led_centre(4)[1] + 3.0);
        assert!((top_led_centre(0)[1] - top_led_centre(9)[1]).abs() < EPS, "symmetric");
    }

    #[test]
    fn everything_fits_inside_the_frame_and_on_the_glass() {
        // The sheared body stays inside the frame, with a pixel left for the edge.
        let m = body_mesh();
        let b = bounds(m.verts.iter().map(|v| [v[0], v[1]]));
        assert!(b[0] >= FRAME[0] + 1.0 && b[2] <= FRAME[2] - 1.0, "{b:?}");
        assert!(b[1] >= FRAME[1] + 0.5 && b[3] <= FRAME[3] - 1.0, "{b:?}");
        // Its back edge shows SLIVER px above the crest; the face edge stays on the numbers.
        assert!((b[1] - (TOP - ARCH - SLIVER)).abs() < 0.1, "{b:?}");
        let front: Vec<[f32; 2]> = m
            .verts
            .iter()
            .filter(|v| v[2].abs() < EPS)
            .map(|v| [v[0], v[1]])
            .collect();
        let fb = bounds(front);
        assert!(
            (fb[1] - (TOP - ARCH + FRONT_R)).abs() < 0.05,
            "front face starts after its round-over: {fb:?}"
        );
        // The mesh is consistent: quads plus the two caps, one slot.
        let n = housing_outline().len();
        assert_eq!(m.verts.len(), n * body_profile().len());
        assert_eq!(m.faces.len(), m.mats.len());
        assert!(m.faces.iter().flatten().all(|&v| (v as usize) < m.verts.len()));
        assert!(m.mats.iter().all(|&s| s == 0));
        // Every lens and the sensor sit on the glass, clear of its edge.
        let glass = offset(&housing_outline(), WALL + GLASS_GAP);
        for i in 0..TOP_LED_X.len() {
            let [cx, cy] = top_led_centre(i);
            for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                let p = [cx + dx * TOP_LED[0] * 0.5, cy + dy * TOP_LED[1] * 0.5];
                assert!(inside(&glass, p, 4.0), "top lens {i}");
            }
        }
        for side in [-1.0, 1.0] {
            for i in 0..SIDE_LEDS {
                let [cx, cy] = side_led_centre(side, i);
                for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                    let p = [cx + dx * SIDE_LED[0] * 0.5, cy + dy * SIDE_LED[1] * 0.5];
                    assert!(inside(&glass, p, 4.0), "side lens {side} {i}");
                    assert!(p[0].abs() > SCREEN[2] + 20.0, "clear of the screen");
                }
            }
        }
        assert!(inside(&glass, SENSOR, SENSOR_R + 4.0));
    }

    #[test]
    fn the_profile_climbs_from_the_back_round_to_the_pocket() {
        let p = body_profile();
        assert!((p[0].0 - BACK_R).abs() < EPS && (p[0].1 + DEPTH).abs() < EPS);
        assert!(p
            .iter()
            .all(|&(d, z)| (0.0..=WALL + EPS).contains(&d) && (-DEPTH - EPS..=EPS).contains(&z)));
        // The front face is flat at z 0 from the round-over to the lip, and the glass sits in the
        // pocket under it, its top below the front face.
        assert!(p.iter().any(|&(d, z)| (d - FRONT_R).abs() < EPS && z.abs() < EPS));
        assert!(p
            .iter()
            .any(|&(d, z)| (d - (WALL - LIP_R)).abs() < EPS && z.abs() < EPS));
        assert!(p.last().is_some_and(|&(d, z)| (d - WALL).abs() < EPS && z < -GLASS_T));
        // The shear moves nothing on the face plane.
        assert_eq!(shear(10.0, -20.0, 0.0), [10.0, -20.0, 0.0]);
        assert!((shear(0.0, 0.0, -DEPTH)[1] + SLIVER).abs() < EPS);
    }

    #[test]
    fn the_scene_renders_the_three_swing_azimuths_in_the_runtime_order() {
        // The runtime blends render 1 when the light swings to the smaller angle.
        assert_eq!(SWING, [-135.0, -160.0, -110.0]);
        let s = housing_scene(Path::new("t.png"));
        assert_eq!(s.swing.len() + 1, 3);
        let az: Vec<f32> = (0..3).map(|i| key_azimuth(&s, i)).collect();
        assert_eq!(az, vec![-135.0, -160.0, -110.0]);
        let rigs = s.rigs();
        for (r, want) in rigs.iter().zip(&az) {
            assert!((r.lights[0].az - want).abs() < EPS);
        }
        assert!(s.passes.beauty && s.passes.swing && !s.passes.data);
        // The parts: body, glass, 10 + 10 lenses and the sensor.
        assert_eq!(s.parts.len(), 2 + TOP_LED_X.len() + 2 * SIDE_LEDS + 1);
        let mats = s.materials();
        let names: Vec<&str> = mats.iter().map(Mat::name).collect();
        for want in ["ddu_body", "ddu_glass", "ddu_lens", "ddu_sensor"] {
            assert!(names.contains(&want), "{want} in {names:?}");
        }
        let Shape::Extrude { thick, .. } = &s.parts[1].shape else {
            panic!("the glass is an extrusion");
        };
        assert!(
            (s.parts[1].z + thick + GLASS_DROP).abs() < EPS,
            "glass top under the front face"
        );
        // The lenses stand flush on the glass.
        for p in s.parts.iter().filter(|p| p.name.starts_with("led_")) {
            assert!((p.top() - (-GLASS_DROP + FLUSH)).abs() < EPS, "{}", p.name);
        }
        // The body is textured by the file it is given.
        let body = s.materials().into_iter().find(|m| m.name() == "ddu_body");
        assert_eq!(
            body.and_then(|m| m.v.get("texture").cloned()),
            Some(serde_json::json!("t.png"))
        );
    }

    #[test]
    fn no_softbox_stands_over_the_screen_at_any_swing_azimuth() {
        // The flat glass mirrors what is straight above it, so a softbox whose footprint reaches
        // the screen veils it. Each light's footprint is taken as the circle round its softbox.
        let s = housing_scene(Path::new("t.png"));
        assert_eq!(s.rig.hdri, 0.0, "no studio HDRI: the glass would mirror its zenith");
        assert!(s.rig.lights.iter().any(|l| l.name == "wall") && s.rig.lights.iter().all(|l| l.name != "sky"));
        let rigs = s.rigs();
        assert_eq!(rigs.len(), SWING.len());
        for rig in &rigs {
            for l in &rig.lights {
                let (az, reach) = (l.az.to_radians(), l.dist * l.el.to_radians().cos());
                let c = [rig.target[0] + reach * az.cos(), rig.target[1] + reach * az.sin()];
                let near = [c[0].clamp(SCREEN[0], SCREEN[2]), c[1].clamp(SCREEN[1], SCREEN[3])];
                let gap = (c[0] - near[0]).hypot(c[1] - near[1]) - l.size[0].hypot(l.size[1]) * 0.5;
                assert!(gap >= SCREEN_CLEAR, "{} at {}°: {gap:.0} px from the screen", l.name, l.az);
            }
        }
    }

    #[test]
    fn the_recipe_adds_its_materials_to_the_hero_script_once() {
        let s = script().unwrap_or_default();
        assert_eq!(s.matches("def mat_ddu_anodised(d):").count(), 1);
        assert_eq!(s.matches(HERO_ANCHOR).count(), 1);
        let (Some(reg), Some(build), Some(builders)) = (
            s.find("BUILDERS['ddu_anodised']"),
            s.find(HERO_ANCHOR),
            s.find("BUILDERS = {"),
        ) else {
            panic!("the script has the builders, the registration and the anchor");
        };
        assert!(
            builders < reg && reg < build,
            "registered after BUILDERS and before the scene"
        );
    }

    #[test]
    fn the_box_filter_keeps_edge_colour_and_coverage() {
        // A 2x2 block: one opaque red pixel, three empty ones (black): the filtered pixel is red
        // at a quarter coverage, not dark red.
        let src = Img::from_fn(2, 2, 4, |x, y| {
            if x == 0 && y == 0 {
                [1.0, 0.0, 0.0, 1.0]
            } else {
                [0.0, 0.0, 0.0, 0.0]
            }
        });
        let out = box_filter(&src, 2);
        assert_eq!((out.w, out.h), (1, 1));
        assert!((out.get(0, 0, 0) - 1.0).abs() < 1e-4);
        assert!((out.get(0, 0, 3) - 0.25).abs() < 1e-4);
    }

    #[test]
    fn brightness_reads_the_screen_area_apart_from_the_border() {
        // A runtime-sized plate, dark glass (0.01) with a bright border outside the screen.
        let w = (FRAME[2] - FRAME[0]) as usize;
        let h = (FRAME[3] - FRAME[1]) as usize;
        let lit = img::srgb_encode(0.5);
        let dark = img::srgb_encode(0.01);
        let im = Img::from_fn(w, h, 4, |x, y| {
            let (lx, ly) = (FRAME[0] + x as f32 + 0.5, FRAME[1] + y as f32 + 0.5);
            let on = lx > SCREEN[0] && lx < SCREEN[2] && ly > SCREEN[1] && ly < SCREEN[3];
            let v = if on { dark } else { lit };
            [v, v, v, 1.0]
        });
        let b = brightness(&im);
        assert!((b.screen_p99 - 0.01).abs() < 1e-3, "{b:?}");
        assert!((b.p95 - 0.5).abs() < 1e-3, "{b:?}");
        let mut v = vec![3.0, 1.0, 2.0];
        assert!((percentile(&mut v, 0.5) - 2.0).abs() < EPS);
        assert!(percentile(&mut [], 0.5).abs() < EPS);
    }

    #[test]
    fn the_neutral_texture_has_the_fixed_mean_and_spread() {
        // A wrapping box blur keeps a constant field constant.
        let flat = box_blur_wrap(&[0.25; 12], 4, 3, 1);
        assert!(flat.iter().all(|v| (v - 0.25).abs() < 1e-6));
        // A noisy, dark, colour-tinted source comes out grey, mean 0.5, deviation the spread.
        let mut seed = 12_345_u32;
        let src = Img::from_fn(128, 128, 3, |_, _| {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let v = 0.1 + 0.05 * (seed >> 8) as f32 / (1 << 24) as f32;
            [v * 1.2, v, v * 0.8, 1.0]
        });
        let out = neutralise(&src);
        let vals: Vec<f32> = out.data.chunks(out.c).map(|p| p[0]).collect();
        assert!(out
            .data
            .chunks(out.c)
            .all(|p| (p[0] - p[1]).abs() < 1e-6 && (p[1] - p[2]).abs() < 1e-6));
        let mean = vals.iter().sum::<f32>() / vals.len() as f32;
        let sd = (vals.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / vals.len() as f32).sqrt();
        assert!((mean - 0.5).abs() < 0.01, "mean {mean}");
        assert!((sd - TEXTURE_SPREAD).abs() < 0.01, "spread {sd}");
    }

    #[test]
    fn the_texture_prompt_names_only_the_finish() {
        let p = BEAD_BLAST.prompt.to_ascii_lowercase();
        let words: Vec<&str> = p.split(|c: char| !c.is_ascii_alphanumeric()).collect();
        for bad in ["bosch", "bmw", "ddu", "display", "dash", "text", "logo"] {
            assert!(!words.contains(&bad), "{bad}");
        }
        assert!(p.contains("seamless tileable") && p.contains("top-down") && p.contains("diffuse"));
        assert!(texture::COOKBOOK.iter().all(|r| r.seed != BEAD_BLAST.seed));
    }
}
