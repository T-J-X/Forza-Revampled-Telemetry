//! ValkyrieLmp1's bespoke assets (`assets/clusters/valkyrie_lmp1/`, ledger
//! `assets/clusters/valkyrie_lmp1/ASSETS.md`):
//!
//! - **Forged carbon (ComfyUI, SDXL + 4x-UltraSharp, seamless 4096²):** the macro of resin-cast
//!   chopped carbon that the pre-build made for this dash (`prebuild/kits.rs`, texture
//!   `forged_carbon_marble`, seed 90001). The recipe asks for the same texture through the same
//!   cache, so on the machine that ran the pre-build it is not generated again. It is the surface
//!   of the Blender wheel face (colour, fibre sparkle and bump under the clear coat).
//! - **Wheel face (Blender, Cycles, the hero library):** the Valkyrie's carbon steering-wheel face
//!   as the cluster's hardware: the angled-shoulder yoke in forged carbon, the OLED window's
//!   cover glass, the anodised partition of the eight centre shift lights and the two shoulder
//!   pods, ten unlit LED lens caps, and the two shoulder encoders (a diamond-knurled anodised ring
//!   round a machined cap, in a piano-black well). Rendered orthographic, 1:1 at the 4K frame,
//!   over each view's envelope, at the rest key and the two G-swing keys; the cluster blends them
//!   with lateral G.
//!
//! `cargo run --release -p asset-forge -- --lane cloud build --set valkyrie_lmp1`. The cluster
//! draws its procedural wheel face when the files are missing.
use crate::ctx::Ctx;
use crate::error::{io_err, Result};
use crate::hero::frame::{self, Frame};
use crate::hero::geom::Outline;
use crate::hero::mat::{self, metal};
use crate::hero::part::Part;
use crate::hero::{texture, Passes, Scene};
use crate::img;
use std::fmt::Write as _;

/// Output folder (repo-relative).
const DIR: &str = "assets/clusters/valkyrie_lmp1";
/// Cycles samples.
const SAMPLES: u32 = 256;
/// The forged-carbon texture, exactly as the pre-build kit defines it (same prompt and seed, so
/// the same cached master).
const FORGED: texture::Recipe = texture::Recipe {
    name: "forged_carbon_marble",
    prompt: "macro photograph of forged carbon composite, marbled chopped carbon fibre flakes of mixed size swirled under clear glossy resin, black and charcoal with fine silvery fibre glints, flat top-down view, even diffuse light, seamless tileable texture, material scan",
    negative: "woven, twill, checker pattern, grid, objects, lighting falloff, perspective, seams, blur, cartoon",
    seed: 90_001,
};
/// Texture master size, and one tile on the wheel face (local px at cockpit scale).
const TEX_SIZE: usize = 4096;
/// See [`TEX_SIZE`].
const TILE_PX: f32 = 900.0;

/// The wheel face in local 4K px round the cockpit anchor (y down), as the cluster's `Geom`.
mod g {
    /// Yoke: top edge half width and y, the shoulder line, the outer half width, the lower
    /// corner y, the bottom half width and y.
    pub const YOKE: [f32; 7] = [560.0, -300.0, -196.0, 700.0, 120.0, 520.0, 212.0];
    /// Corner fillet of the yoke outline.
    pub const YOKE_FILLET: f32 = 26.0;
    /// The OLED window (x0, y0, x1, y1) and its corner radius.
    pub const WINDOW: [f32; 4] = [-440.0, -214.0, 440.0, 192.0];
    /// See [`WINDOW`].
    pub const WINDOW_R: f32 = 28.0;
    /// The centre partition (x0, y0, x1, y1).
    pub const PARTITION: [f32; 4] = [-408.0, -286.0, 408.0, -240.0];
    /// Pod lens size, pod row y, the eight centre pods' x and the shoulder pods' x.
    pub const POD: [f32; 2] = [76.0, 30.0];
    /// See [`POD`].
    pub const POD_Y: f32 = -263.0;
    /// See [`POD`].
    pub const SINGLE_X: f32 = 560.0;
    /// Encoders: centres, the knurled ring's inner and outer radius, the well's outer radius.
    pub const ENC: [[f32; 2]; 2] = [[-604.0, 4.0], [604.0, 4.0]];
    /// See [`ENC`].
    pub const KNURL: [f32; 2] = [59.0, 73.0];
    /// See [`ENC`].
    pub const WELL_R: f32 = 92.0;

    /// The eight centre pods' x.
    pub fn pods_x() -> [f32; 8] {
        core::array::from_fn(|i| -350.0 + 100.0 * i as f32)
    }
}

/// Scale of the hood layout relative to the cockpit one (the cluster's `HOOD_SCALE`).
const HOOD_SCALE: f32 = 0.7;

/// The yoke outline (absolute local px, y down), corners filleted, scaled by `s`.
fn yoke_outline(s: f32) -> Outline {
    let [ht, top, sh, h, low, hb, bot] = g::YOKE;
    let corners = [
        [-ht, top],
        [ht, top],
        [h, sh],
        [h, low],
        [hb, bot],
        [-hb, bot],
        [-h, low],
        [-h, sh],
    ];
    let n = corners.len();
    let mut points = Vec::new();
    for i in 0..n {
        let p = corners[i];
        let a = corners[(i + n - 1) % n];
        let b = corners[(i + 1) % n];
        let da = norm([a[0] - p[0], a[1] - p[1]]);
        let db = norm([b[0] - p[0], b[1] - p[1]]);
        let r = g::YOKE_FILLET;
        // A quadratic fillet from r along the incoming edge to r along the outgoing one.
        let p0 = [p[0] + da[0] * r, p[1] + da[1] * r];
        let p2 = [p[0] + db[0] * r, p[1] + db[1] * r];
        for k in 0..=8 {
            let t = k as f32 / 8.0;
            let u = 1.0 - t;
            let q = [
                u * u * p0[0] + 2.0 * u * t * p[0] + t * t * p2[0],
                u * u * p0[1] + 2.0 * u * t * p[1] + t * t * p2[1],
            ];
            points.push([q[0] * s, q[1] * s]);
        }
    }
    // Outline is CCW in a y-down frame when listed clockwise on screen; reverse to match the
    // library's convention (`Outline::rrect` runs with increasing angle, y down).
    points.reverse();
    Outline { points }
}

fn norm(v: [f32; 2]) -> [f32; 2] {
    let l = (v[0] * v[0] + v[1] * v[1]).sqrt().max(1e-6);
    [v[0] / l, v[1] / l]
}

/// The wheel face scene for one view.
fn wheel_face(name: &str, fr: Frame, s: f32, forged: Option<&str>) -> Scene {
    let mut sc = Scene::new(name, fr);
    sc.samples = SAMPLES;
    sc.passes = Passes {
        beauty: true,
        swing: true,
        data: false,
        ao: false,
        rmh: false,
        emission: false,
    };
    let carbon = match forged {
        Some(t) => mat::forged(t, TILE_PX * s),
        None => mat::carbon(10.0 * s).named("forged_carbon"),
    };
    // Yoke.
    sc.add(Part::extrude(
        "yoke",
        yoke_outline(s),
        -22.0 * s,
        22.0 * s,
        7.0 * s,
        carbon,
    ));
    // OLED cover glass, set into the face (the cluster keeps its inside black).
    let [x0, y0, x1, y1] = g::WINDOW;
    sc.add(Part::extrude(
        "window_bezel",
        Outline::rrect(
            0.0,
            (y0 + y1) * 0.5 * s,
            (x1 - x0 + 16.0) * s,
            (y1 - y0 + 16.0) * s,
            (g::WINDOW_R + 8.0) * s,
            10,
        ),
        -6.0 * s,
        6.0 * s,
        3.0 * s,
        mat::anodised([0.05, 0.05, 0.055]).named("bezel_black"),
    ));
    sc.add(Part::extrude(
        "window_glass",
        Outline::rrect(
            0.0,
            (y0 + y1) * 0.5 * s,
            (x1 - x0) * s,
            (y1 - y0) * s,
            g::WINDOW_R * s,
            10,
        ),
        -1.0 * s,
        1.5 * s,
        1.0 * s,
        mat::piano_black().named("cover_glass"),
    ));
    // Shift-light partition and the shoulder pods' housings.
    let [px0, py0, px1, py1] = g::PARTITION;
    let anod = mat::anodised([0.08, 0.085, 0.09]).named("partition");
    sc.add(Part::extrude(
        "partition",
        Outline::rrect(
            0.0,
            (py0 + py1) * 0.5 * s,
            (px1 - px0) * s,
            (py1 - py0) * s,
            14.0 * s,
            8,
        ),
        0.0,
        7.0 * s,
        3.0 * s,
        anod.clone(),
    ));
    let [pw, ph] = g::POD;
    for sx in [-g::SINGLE_X, g::SINGLE_X] {
        sc.add(Part::extrude(
            &format!("pod_housing_{}", if sx < 0.0 { "l" } else { "r" }),
            Outline::rrect(sx * s, g::POD_Y * s, 92.0 * s, 46.0 * s, 14.0 * s, 8),
            0.0,
            7.0 * s,
            3.0 * s,
            anod.clone(),
        ));
    }
    // Unlit LED lens caps (the kit lights them).
    let cap = mat::led_cap([0.0, 0.25, 0.4], 0.0).named("led_cap");
    let mut pods: Vec<f32> = g::pods_x().to_vec();
    pods.push(-g::SINGLE_X);
    pods.push(g::SINGLE_X);
    for (i, x) in pods.iter().enumerate() {
        sc.add(Part::extrude(
            &format!("led{i}"),
            Outline::rrect(x * s, g::POD_Y * s, pw * s, ph * s, 7.0 * s, 6),
            7.0 * s,
            2.5 * s,
            1.2 * s,
            cap.clone(),
        ));
    }
    // Encoders: a piano-black well, the knurled anodised ring, a machined cap.
    for (i, c) in g::ENC.iter().enumerate() {
        let at = [c[0] * s, c[1] * s];
        sc.add(Part::face(
            &format!("enc{i}_well"),
            at,
            0.5 * s,
            g::WELL_R * s,
            mat::piano_black(),
        ));
        sc.add(Part::ring(
            &format!("enc{i}_knurl"),
            at,
            0.0,
            g::KNURL[0] * s,
            g::KNURL[1] * s,
            12.0 * s,
            mat::knurled(metal::ALUMINIUM, 5.0 * s, false)
                .named("knurl_anodised")
                .set("coat", 1.0)
                .set("coat_rough", 0.18)
                .set("coat_tint", serde_json::json!([0.30, 0.02, 0.02])),
        ));
        sc.add(Part::face(
            &format!("enc{i}_cap"),
            at,
            4.0 * s,
            (g::KNURL[0] - 2.0) * s,
            mat::spun(metal::GUNMETAL).named("cap_spun"),
        ));
    }
    sc
}

/// Builds the texture (cached) and the two views' wheel faces, and writes the ledger.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let mut rows = Vec::new();
    // The pre-build's cache folder for this dash, so its texture is reused.
    let tex_dir = ctx
        .paths
        .scratch
        .join("prebuild")
        .join("valkyrie_lmp1")
        .join("textures");
    let forged = match texture::make_texture(&ctx.paths, &FORGED, TEX_SIZE, &tex_dir) {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("  forged carbon texture skipped (procedural twill instead): {e}");
            None
        }
    };
    if let Some(p) = &forged {
        // The runtime copy: the master downsized to 1024² for the cluster's procedural fallback
        // is not shipped; the texture lives inside the Blender render. Record the master.
        rows.push(format!(
            "| `forged_carbon_marble.png` (master only, `{}`) | colour, seamless | {TEX_SIZE}x{TEX_SIZE} | master | comfyui:hero::texture (SDXL base 1.0 + refiner + 4x-UltraSharp, seamless) seed={} prompt=\"{}\" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the forged carbon of the wheel face renders below, tiled at {TILE_PX} px |",
            p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
            FORGED.seed,
            FORGED.prompt,
        ));
    }
    let tex = forged.as_ref().map(|p| p.to_string_lossy().to_string());
    let scratch = ctx.scratch("valkyrie_lmp1")?;
    for (view, anchor, s) in [("cockpit", &frame::COCKPIT, 1.0), ("hood", &frame::HOOD, HOOD_SCALE)] {
        let fr = Frame::envelope(anchor);
        let sc = wheel_face(&format!("valkyrie_{view}"), fr, s, tex.as_deref());
        println!("  blender wheel face ({view})");
        let out = sc.render(&ctx.paths, &scratch.join(view))?;
        let keys: Vec<f32> = core::iter::once(crate::hero::rig::SWING[0])
            .chain(sc.swing.iter().copied())
            .collect();
        for (i, path) in out.beauty.iter().enumerate() {
            let im = img::load(path)?.to_channels(4);
            let file = format!("wheel_{view}_k{i}.png");
            let rel = format!("{DIR}/{file}");
            let bytes = img::save_png(&im, &ctx.abs(&rel), false, true)?;
            rows.push(format!(
                "| `{file}` | colour, straight alpha | {}x{} | {} KB | blender:hero (Cycles {SAMPLES} samples, ortho 1:1 over the {view} envelope) wheel face: forged-carbon yoke, cover glass, anodised partition, 10 LED caps, 2 knurled encoders; key azimuth {:.0}° | MIT (asset-forge) | the {view} wheel face; blended with the other keys by lateral G |",
                im.w,
                im.h,
                bytes / 1024,
                keys.get(i).copied().unwrap_or(0.0),
            ));
        }
    }
    let mut md = String::from(
        "# ValkyrieLmp1 assets\n\nBuilt by `cargo run --release -p asset-forge -- --lane cloud build --set valkyrie_lmp1` (recipe `tools/asset-forge/src/recipes/valkyrie_lmp1.rs`). Each render covers its view's whole envelope at 1:1 (cockpit 1440x560 from (-720, -320); hood 1760x400 from (-880, -220), local 4K px). The cluster draws its procedural wheel face when the files are missing.\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in rows {
        let _ = writeln!(md, "{r}");
    }
    let ledger = ctx.abs(&format!("{DIR}/ASSETS.md"));
    img::write_file(&ledger, md.as_bytes())?;
    println!("  wrote {DIR}/ (wheel faces + ASSETS.md)");
    Ok(())
}
