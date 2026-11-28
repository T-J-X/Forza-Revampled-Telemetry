//! `asset-forge prebuild <snake_name>...`: the layout-independent 4K assets of a dash, made
//! before its thread starts, so the thread finds them ready after its Penpot board.
//!
//! A dash thread draws its Penpot board first, so a pre-build can only make what does not
//! depend on the layout (TJX, 2026-11-28: "look at the whole catalog ... all assets that will be
//! needed in the cloud for blender and comfyui"):
//!
//! - **Textures (ComfyUI, SDXL + refiner + 4x-UltraSharp):** seamless 4096² masters of the
//!   organic surfaces a brief names (forged carbon, leather, alcantara, enamel, paper ...), made
//!   by [`texture::make_texture`] (half-offset seam repaint, `make_seamless` guard).
//! - **Swatches (Blender, Cycles):** 4096² tiles or discs of the procedural materials a brief
//!   names (sunray and guilloché dials, brushed and anodised alloy, twill carbon ...), rendered
//!   top-down and lit evenly (the rig is four times the tile's span away), with their normal and
//!   rough/metal/height passes.
//! - **Sprites (Blender, Cycles):** hardware whose size the brief already states (needles, hub
//!   caps, LED caps, encoder rings, screws), square ortho frames at 2 px per 4K px, pivot at the
//!   centre, pointing up, rendered at the rest key and the two G-swing keys.
//!
//! What depends on the layout (housings, yokes, binnacles, full plates) is listed in the dash's
//! `deferred` notes and stays with its thread. Psychedelic and Abstract figures are drawn by the
//! dash's own maths at run time and are never pre-built (CATALOG.md: noise never draws the
//! figure).
//!
//! Where things go, per dash:
//! - 4K masters: `<Drive masters>\<snake>\prebuild\{textures,swatches,sprites}\` plus the
//!   contact sheet and a copy of the manifest. Blender's raw job folders also land in
//!   `<Drive masters>\<snake>\hero_prebuild_*` through the `--masters` sink.
//! - Repo: `assets/clusters/<snake>/prebuild/manifest.json` and `README.md` (every file, its
//!   generator, seed, prompt, size, fingerprint, licence and use) and the 1:1 runtime sprites in
//!   `sprites/`. Textures and swatches are committed as manifest rows only: their runtime size
//!   comes from the layout, so the dash's own recipe downsizes the master it picks.
//!
//! The shared ledger (`assets/ASSETS.md` and friends) is never touched.
pub mod kits;
pub mod liquid;

use crate::error::{io_err, tool_err, ForgeError, Result};
use crate::hero::frame::Frame;
use crate::hero::geom::Outline;
use crate::hero::mat::Mat;
use crate::hero::part::Part;
use crate::hero::rig::Rig;
use crate::hero::{texture, Passes, Scene};
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use crate::paths::Paths;
use serde_json::{json, Value};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Master size of textures and swatches (px).
pub const MASTER: usize = 4096;
/// Swatch frame in local px (rendered at 2 px per px, so the master is 4096²).
const SWATCH_SPAN: f32 = 2048.0;
/// Sprite render scale: image px per 4K local px (the runtime copy is box-filtered to 1:1).
const SPRITE_SCALE: f32 = 2.0;
/// Cycles samples for swatches and sprites.
const SAMPLES: u32 = 256;
/// Licence of SDXL outputs.
const SDXL_LICENCE: &str = "CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge)";
/// Licence of Blender renders.
const BLENDER_LICENCE: &str = "MIT (asset-forge)";

/// A ComfyUI texture.
#[derive(Debug, Clone)]
pub struct Tex {
    /// Prompt, negative, seed and file stem.
    pub recipe: texture::Recipe,
    /// What the dash uses it for.
    pub use_: &'static str,
}

/// A Blender material swatch.
#[derive(Debug, Clone)]
pub struct Swatch {
    /// File stem.
    pub name: &'static str,
    /// The material.
    pub mat: Mat,
    /// A disc (radial patterns: sunray, spun, rose) instead of a seamless tile.
    pub disc: bool,
    /// What the dash uses it for.
    pub use_: &'static str,
}

/// A Blender hardware sprite.
#[derive(Debug, Clone)]
pub struct Sprite {
    /// File stem.
    pub name: &'static str,
    /// Square frame side in 4K local px.
    pub size: f32,
    /// The parts, centred on (0, 0), pointing up.
    pub parts: Vec<Part>,
    /// Also render the two G-swing keys.
    pub swing: bool,
    /// What the dash uses it for.
    pub use_: &'static str,
}

/// What a computed asset is (exact maths on the CPU, [`liquid`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Calc {
    /// The liquid-metal studio as an octahedral environment map.
    StudioEnv,
    /// The studio's 256 x 1 env band through the key.
    EnvBand,
    /// A smooth metal sphere in the studio.
    Matcap(liquid::Metal),
    /// The metal's Fresnel reflectance over cos θ.
    Fresnel(liquid::Metal),
    /// A lossless film of this index on the metal: thickness x cos θ.
    Film(f32, liquid::Metal),
}

/// A computed asset.
#[derive(Debug, Clone)]
pub struct Computed {
    /// File stem.
    pub name: &'static str,
    /// What to compute.
    pub calc: Calc,
    /// What the dash uses it for.
    pub use_: &'static str,
}

/// Everything pre-built for one dash.
#[derive(Debug, Clone, Default)]
pub struct Kit {
    /// The dash's key (theme name).
    pub key: &'static str,
    /// Build number in CATALOG.md when the kit was written.
    pub build: u32,
    /// ComfyUI textures.
    pub textures: Vec<Tex>,
    /// Blender swatches.
    pub swatches: Vec<Swatch>,
    /// Blender sprites.
    pub sprites: Vec<Sprite>,
    /// Computed maps (CPU, exact).
    pub computed: Vec<Computed>,
    /// Layout-dependent assets left to the dash thread.
    pub deferred: Vec<&'static str>,
    /// A line for the README.
    pub notes: &'static str,
}

/// One manifest row.
struct Row {
    name: String,
    kind: &'static str,
    master: Option<String>,
    runtime: Vec<String>,
    dims: String,
    fp: String,
    generator: String,
    licence: &'static str,
    use_: String,
}

/// `prebuild --plan | --list | <snake>... | --all [--preview] [--blender-only]` (`--blender-only` skips the
/// ComfyUI textures, for a CPU check of the Blender scenes without a ComfyUI server).
pub fn cli(paths: &Paths, args: &[String]) -> Result<()> {
    let preview = args.iter().any(|a| a == "--preview");
    let blender_only = args.iter().any(|a| a == "--blender-only");
    let names: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| !a.starts_with("--"))
        .collect();
    if args.iter().any(|a| a == "--plan") {
        print!("{}", plan());
        return Ok(());
    }
    if args.iter().any(|a| a == "--list") {
        for snake in kits::ORDER {
            if let Some(k) = kits::kit(snake) {
                println!(
                    "#{:<3} {snake:<28} {} textures, {} swatches, {} sprites, {} computed",
                    k.build,
                    k.textures.len(),
                    k.swatches.len(),
                    k.sprites.len(),
                    k.computed.len()
                );
            }
        }
        return Ok(());
    }
    let list: Vec<&str> = if args.iter().any(|a| a == "--all") {
        kits::ORDER.to_vec()
    } else {
        names
    };
    if list.is_empty() {
        return Err(ForgeError::Usage(
            "asset-forge prebuild --plan | --list | <snake_name>... | --all [--preview] [--blender-only]".into(),
        ));
    }
    for snake in &list {
        if kits::kit(snake).is_none() {
            return Err(ForgeError::Usage(format!(
                "no pre-build kit for {snake}; `asset-forge prebuild --list` shows them"
            )));
        }
    }
    for snake in list {
        println!("[prebuild {snake}]");
        run(paths, snake, preview, blender_only)?;
    }
    Ok(())
}

/// Builds one dash's kit.
pub fn run(paths: &Paths, snake: &str, preview: bool, blender_only: bool) -> Result<()> {
    let kit = kits::kit(snake).ok_or_else(|| tool_err("prebuild", format!("no kit for {snake}")))?;
    // Each dash's masters go to its own Drive folder, whatever --masters said. A full run stops
    // before any GPU work when the Drive folder is not mounted; a preview never writes there,
    // nor into the repo (its files stay in the scratch folder).
    let mut paths = paths.clone();
    paths.masters = if preview {
        None
    } else {
        Some(crate::lane::masters_dir(snake)?)
    };
    let masters = paths.masters.as_ref().map(|d| d.join("prebuild"));
    let base = paths
        .scratch
        .join(if preview { "prebuild_preview" } else { "prebuild" });
    let scratch = base.join(snake);
    let repo_rel = format!("assets/clusters/{snake}/prebuild");
    let repo = if preview {
        scratch.join("repo")
    } else {
        paths.root.join(&repo_rel)
    };
    // Textures are cached across dashes: a recipe shared by several kits runs SDXL once.
    let shared_textures = base.join("_shared").join("textures");
    for d in [&repo, &scratch] {
        std::fs::create_dir_all(d).map_err(|e| io_err(d, e))?;
    }
    let (scale, samples, tex_size) = if preview {
        (0.25, 48, 1024)
    } else {
        (1.0, SAMPLES, MASTER)
    };
    let mut rows = Vec::new();
    let mut thumbs: Vec<Img> = Vec::new();
    let mut made: Vec<(&str, PathBuf)> = Vec::new();

    for t in kit.textures.iter().filter(|_| !blender_only) {
        println!("  texture {}", t.recipe.name);
        let path = texture::make_texture(&paths, &t.recipe, tex_size, &shared_textures)?;
        let im = img::load(&path)?.to_channels(3);
        let master = sink(masters.as_deref(), "textures", &path)?;
        made.push((t.recipe.name, path.clone()));
        thumbs.push(thumb(&im));
        rows.push(Row {
            name: t.recipe.name.into(),
            kind: "texture (ComfyUI, seamless, sRGB)",
            master,
            runtime: Vec::new(),
            dims: format!("{}x{}", im.w, im.h),
            fp: file_fp(&path)?,
            generator: format!(
                "comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed={} prompt=\"{}\" extra negative=\"{}\", tile seam score {:.4}",
                t.recipe.seed,
                t.recipe.prompt,
                t.recipe.negative,
                img::tile_seam_score(&im)
            ),
            licence: SDXL_LICENCE,
            use_: t.use_.into(),
        });
    }

    for s in &kit.swatches {
        println!("  swatch {}", s.name);
        // A regular pattern (twill, clous) gets a span of a whole, even number of repeats, so
        // the tile repeats exactly and the seam blend never doubles the pattern.
        let span = snapped_span(&s.mat);
        let half = span * 0.5;
        let fr = Frame::rect(-half, -half, span, span).scaled(MASTER as f32 / span * scale);
        let mut sc = Scene::new(&format!("prebuild_{snake}_{}", s.name), fr);
        // Near-even but asymmetric light (`Rig::swatch`): every light twelve spans away, the key
        // off the diagonal so weave and grain read (make_seamless takes care of the edges).
        sc.rig = Rig::swatch(span);
        sc.swing.clear();
        sc.samples = samples;
        sc.passes = Passes {
            beauty: true,
            swing: false,
            data: true,
            ao: false,
            rmh: true,
            emission: false,
        };
        // A swatch may name one of the kit's textures (`texture_ref`): Blender gets its path.
        let mut m = s.mat.clone();
        if let Some(r) = m.v.get("texture_ref").and_then(Value::as_str).map(str::to_owned) {
            let p = made
                .iter()
                .find(|(n, _)| *n == r)
                .map(|(_, p)| p.to_string_lossy().to_string())
                .ok_or_else(|| tool_err("prebuild", format!("{} names texture {r}, not in the kit", s.name)))?;
            m = m.set("texture", p);
        }
        if s.disc {
            sc.add(Part::face("disc", [0.0, 0.0], 0.0, half * 0.98, m));
        } else {
            let side = span * 1.2;
            sc.add(Part::extrude(
                "tile",
                Outline::rrect(0.0, 0.0, side, side, 1.0, 4),
                -4.0,
                4.0,
                0.0,
                m,
            ));
        }
        let out = sc.render(&paths, &scratch.join("swatches").join(s.name))?;
        let mut beauty = img::load(&out.beauty[0])?;
        if !s.disc {
            beauty = img::make_seamless(&beauty.to_channels(3), 0.04);
        }
        let file = scratch.join("swatches").join(format!("{}.png", s.name));
        img::save_png(&beauty, &file, true, true)?;
        let master = sink(masters.as_deref(), "swatches", &file)?;
        for (suffix, p) in [("normal", &out.normal), ("rmh", &out.rmh)] {
            if let Some(p) = p {
                let named = scratch.join("swatches").join(format!("{}_{suffix}.png", s.name));
                if s.disc {
                    std::fs::copy(p, &named).map_err(|e| io_err(&named, e))?;
                } else {
                    // The data passes tile like the beauty (linear data, no sRGB tag).
                    let mut d = img::make_seamless(&img::load(p)?.to_channels(3), 0.04);
                    if suffix == "normal" {
                        renormalize_normal(&mut d);
                    }
                    img::save_png(&d, &named, true, false)?;
                }
                sink(masters.as_deref(), "swatches", &named)?;
            }
        }
        thumbs.push(thumb(&beauty));
        rows.push(Row {
            name: s.name.into(),
            kind: if s.disc {
                "swatch disc (Blender, 16-bit sRGB, straight alpha) + _normal + _rmh"
            } else {
                "swatch tile (Blender, seamless, 16-bit sRGB) + _normal + _rmh"
            },
            master,
            runtime: Vec::new(),
            dims: format!("{}x{}", beauty.w, beauty.h),
            fp: file_fp(&file)?,
            generator: format!(
                "blender:hero prebuild swatch, Cycles {samples} samples, ortho top-down {span} local px to {MASTER} px, hero rig at 16x span key -135°, Standard view; material {}",
                serde_json::to_string(&s.mat.to_json()).unwrap_or_default()
            ),
            licence: BLENDER_LICENCE,
            use_: s.use_.into(),
        });
    }

    for sp in &kit.sprites {
        println!("  sprite {}", sp.name);
        let fr = Frame::square(0.0, 0.0, sp.size).scaled(SPRITE_SCALE * scale);
        let mut sc = Scene::new(&format!("prebuild_{snake}_{}", sp.name), fr);
        sc.samples = samples;
        if !sp.swing {
            sc.swing.clear();
        }
        sc.passes = Passes {
            beauty: true,
            swing: sp.swing,
            data: false,
            ao: false,
            rmh: false,
            emission: true,
        };
        for p in &sp.parts {
            sc.add(p.clone());
        }
        let out = sc.render(&paths, &scratch.join("sprites").join(sp.name))?;
        let mut runtime = Vec::new();
        let mut master = None;
        let mut first: Option<Img> = None;
        let mut fp_all = Vec::new();
        for (i, b) in out.beauty.iter().enumerate() {
            let named = scratch.join("sprites").join(format!("{}_{i}.png", sp.name));
            std::fs::copy(b, &named).map_err(|e| io_err(&named, e))?;
            let m = sink(masters.as_deref(), "sprites", &named)?;
            if i == 0 {
                master = m;
            }
            let big = img::load(&named)?;
            // The runtime copy is exactly 1:1 with the frame (a preview resamples).
            let k = (SPRITE_SCALE * scale).round().max(1.0) as usize;
            let side = sp.size.round().max(1.0) as usize;
            let small = if preview {
                big.resize(side, side, false)
            } else {
                img::box_down(&big, k)
            };
            let rel = format!("{repo_rel}/sprites/{}_{i}.png", sp.name);
            let abs = repo.join("sprites").join(format!("{}_{i}.png", sp.name));
            if let Some(parent) = abs.parent() {
                std::fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
            }
            img::save_png(&small, &abs, false, true)?;
            fp_all.push(file_fp(&abs)?);
            runtime.push(rel);
            if first.is_none() {
                first = Some(big);
            }
        }
        if let Some(em) = &out.emission {
            let named = scratch.join("sprites").join(format!("{}_emission.png", sp.name));
            std::fs::copy(em, &named).map_err(|e| io_err(&named, e))?;
            sink(masters.as_deref(), "sprites", &named)?;
        }
        let big = first.ok_or_else(|| tool_err("prebuild", format!("{} rendered no beauty", sp.name)))?;
        thumbs.push(thumb(&big));
        rows.push(Row {
            name: sp.name.into(),
            kind: "sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°)",
            master,
            runtime,
            dims: format!("{s}x{s} at 1:1 ({m}x{m} master)", s = sp.size.round(), m = big.w),
            fp: fp_all.join(","),
            generator: format!(
                "blender:hero prebuild sprite, Cycles {samples} samples, ortho square {} local px at {SPRITE_SCALE} px/px, hero rig, box-filtered to 1:1; parts {}",
                sp.size,
                sp.parts.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")
            ),
            licence: BLENDER_LICENCE,
            use_: sp.use_.into(),
        });
    }

    for c in &kit.computed {
        println!("  computed {}", c.name);
        let (row, th) = computed(c, preview, &scratch, masters.as_deref(), &repo, &repo_rel)?;
        thumbs.push(th);
        rows.push(row);
    }

    // Contact sheet: every item at 512 px on black, four across.
    if !thumbs.is_empty() {
        let sheet = img::atlas(&thumbs, 4.min(thumbs.len()));
        let file = scratch.join(format!("{snake}_prebuild_sheet.png"));
        img::save_png(&sheet, &file, false, true)?;
        if let Some(m) = &masters {
            copy_into(&file, m)?;
        }
        println!("  sheet {}", file.display());
    }

    let manifest = manifest(&kit, snake, &rows, preview);
    let text = serde_json::to_string_pretty(&manifest).unwrap_or_default();
    let mpath = repo.join("manifest.json");
    img::write_file(&mpath, text.as_bytes())?;
    let readme = readme(&kit, snake, &rows);
    img::write_file(&repo.join("README.md"), readme.as_bytes())?;
    if let Some(m) = &masters {
        copy_into(&mpath, m)?;
    }
    println!("  wrote {} ({} items)", repo.display(), rows.len());
    Ok(())
}

/// Makes one computed asset: the master (16-bit linear) to the scratch and the Drive masters, the
/// runtime copy into the repo. Returns its manifest row and a thumbnail.
fn computed(
    c: &Computed,
    preview: bool,
    scratch: &Path,
    masters: Option<&Path>,
    repo: &Path,
    repo_rel: &str,
) -> Result<(Row, Img)> {
    use liquid::Metal;
    let div = if preview { 4 } else { 1 };
    let src = |m: Metal| m.source();
    // (master, runtime, runtime is sRGB 8-bit, kind, generator)
    let (master, runtime, srgb8, kind, generator): (Img, Img, bool, &str, String) = match c.calc {
        Calc::StudioEnv => {
            let m = liquid::studio_env(MASTER / div);
            let r = liquid::encode_srgb(&m.resize(2048 / div, 2048 / div, false));
            (m, r, true, "environment, octahedral (z at the viewer in the centre, the kit's `oct_uv`); runtime sRGB 8-bit, master 16-bit linear; white = radiance 16",
             "computed: prebuild::liquid::studio (analytic emitters, 2x2 supersampled): key strip az -175..-95 el 36..44 radiance 10; softbox arc az -180..-30 el 18..30 radiance 5.4 fading over its last 25 deg; cool stripe az 15..75 el 3..9 (0.8, 2.0, 3.4); horizon ring el 0..5 radiance 0.5; behind the screen sky 0.06..0.18 over floor 0.012 with a 1 deg horizon; black within 45 deg of the viewer".into())
        }
        Calc::EnvBand => {
            let m = liquid::env_band(256);
            (m.clone(), m, false, "env band LUT 256x1, 16-bit linear (white = radiance 16): x = angle -180..180 deg round the vertical great circle through the key (az -135), 0 at the key-side horizon, 0.75 at the viewer",
             "computed: prebuild::liquid::env_band, 8x supersampled slice of the studio".into())
        }
        Calc::Matcap(metal) => {
            let m = liquid::matcap(metal, MASTER / div);
            let r = liquid::encode_srgb(&m.resize(1024 / div, 1024 / div, false));
            (m, r, true, "matcap (view-space normal u, v = -1..1, y down), straight alpha = disc coverage; runtime sRGB 8-bit RGBA, master 16-bit linear; white = radiance 16",
             format!("computed: prebuild::liquid::matcap, perfectly smooth sphere, mirror direction into the studio x exact spectral Fresnel ({}), 4x4 supersampled", src(metal)))
        }
        Calc::Fresnel(metal) => {
            let m = liquid::fresnel_lut(metal);
            (m.clone(), m, false, "Fresnel LUT 256x1, 16-bit linear sRGB reflectance, x = cos theta 0..1",
             format!("computed: prebuild::liquid::conductor_reflectance, exact unpolarised, integrated 380..780 nm with the CIE 1931 fit (Wyman 2013), flat spectrum = white; {}", src(metal)))
        }
        Calc::Film(n, metal) => {
            let m = liquid::film_lut(n, metal, 512 / div, 128 / div);
            (m.clone(), m, false, "thin-film LUT, 16-bit linear sRGB reflectance: x = film thickness 0..1200 nm (column 0 = bare metal), y = cos theta 1 (top) .. 0",
             format!("computed: prebuild::liquid::film_reflectance, Airy sum of optics.md 9 with a complex-index substrate, film n = {n}, s and p averaged, integrated 380..780 nm (CIE 1931 fit, flat = white); {}", src(metal)))
        }
    };
    let dir = scratch.join("computed");
    std::fs::create_dir_all(&dir).map_err(|e| io_err(&dir, e))?;
    let mfile = dir.join(format!("{}_master.png", c.name));
    img::save_png(&master, &mfile, true, false)?;
    let msink = sink(masters, "computed", &mfile)?;
    let rel = format!("{repo_rel}/computed/{}.png", c.name);
    let abs = repo.join("computed").join(format!("{}.png", c.name));
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
    }
    if srgb8 {
        img::save_png(&runtime, &abs, false, true)?;
    } else {
        img::save_png(&runtime, &abs, true, false)?;
    }
    // Thumbnail: sRGB-encoded so the sheet shows what the eye sees; LUTs stretched to square.
    let th = liquid::encode_srgb(&master.to_channels(4).resize(512, 512, false));
    let row = Row {
        name: c.name.into(),
        kind,
        master: msink,
        runtime: vec![rel],
        dims: format!("{}x{} runtime ({}x{} master)", runtime.w, runtime.h, master.w, master.h),
        fp: file_fp(&abs)?,
        generator,
        licence: "MIT (asset-forge); optical constants CC0 (refractiveindex.info)",
        use_: c.use_.into(),
    };
    Ok((row, thumb(&th)))
}

/// The kits as Markdown, one section per dash in build order (`prebuild --plan`).
#[must_use]
pub fn plan() -> String {
    let mut md = String::new();
    for snake in kits::ORDER {
        let Some(k) = kits::kit(snake) else { continue };
        let _ = writeln!(md, "### #{} `{}` (`{snake}`)\n", k.build, k.key);
        for t in &k.textures {
            let _ = writeln!(
                md,
                "- Texture `{}` (ComfyUI, seed {}): {}",
                t.recipe.name, t.recipe.seed, t.use_
            );
        }
        for s in &k.swatches {
            let shape = if s.disc { "disc" } else { "tile" };
            let _ = writeln!(md, "- Swatch `{}` (Blender {shape}): {}", s.name, s.use_);
        }
        for s in &k.sprites {
            let _ = writeln!(md, "- Sprite `{}` (Blender, {} px): {}", s.name, s.size, s.use_);
        }
        for c in &k.computed {
            let _ = writeln!(md, "- Computed `{}` (CPU, exact): {}", c.name, c.use_);
        }
        for d in &k.deferred {
            let _ = writeln!(md, "- Left to the thread: {d}");
        }
        md.push('\n');
    }
    md
}

/// The swatch span in local px: [`SWATCH_SPAN`] rounded to an even number of the material's
/// repeats (a 2x2 twill repeats every four tows; guilloché every `pitch`), or as it is for
/// materials without a regular pattern.
fn snapped_span(m: &Mat) -> f32 {
    let get = |k: &str| m.v.get(k).and_then(Value::as_f64).map(|v| v as f32);
    let kind = m.v.get("kind").and_then(Value::as_str).unwrap_or_default();
    let period = match kind {
        "carbon" => get("tow").map(|t| t * 4.0),
        _ => get("pitch").or_else(|| get("knurl")),
    };
    match period {
        Some(p) if p > 0.5 => {
            let n = (SWATCH_SPAN / p / 2.0).round().max(1.0) * 2.0;
            n * p
        }
        _ => SWATCH_SPAN,
    }
}

/// Re-normalises an 8/16-bit tangent normal map (0..1 encoded) after blending.
fn renormalize_normal(d: &mut Img) {
    let c = d.c;
    for px in d.data.chunks_mut(c) {
        let (x, y, z) = (px[0] * 2.0 - 1.0, px[1] * 2.0 - 1.0, px[2] * 2.0 - 1.0);
        let l = (x * x + y * y + z * z).sqrt();
        if l > 1e-6 {
            px[0] = x / l * 0.5 + 0.5;
            px[1] = y / l * 0.5 + 0.5;
            px[2] = z / l * 0.5 + 0.5;
        }
    }
}

/// Copies `file` into `<masters>/prebuild/<sub>/` and returns its path relative to the Drive
/// masters folder, or `None` when no masters folder is mounted.
fn sink(masters: Option<&Path>, sub: &str, file: &Path) -> Result<Option<String>> {
    let Some(m) = masters else { return Ok(None) };
    let dest = m.join(sub);
    copy_into(file, &dest)?;
    let name = file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let dash = m
        .parent()
        .and_then(Path::file_name)
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(Some(format!("{dash}/prebuild/{sub}/{name}")))
}

fn copy_into(file: &Path, dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(dir).map_err(|e| io_err(dir, e))?;
    let to = dir.join(file.file_name().unwrap_or_default());
    std::fs::copy(file, &to).map_err(|e| io_err(&to, e))?;
    Ok(to)
}

fn file_fp(path: &Path) -> Result<String> {
    Ok(fingerprint(&[&crate::ctx::read(path)?]))
}

/// A 512² thumbnail flattened on OLED black.
fn thumb(src: &Img) -> Img {
    let s = src.to_channels(4).resize(512, 512, false);
    Img::from_fn(512, 512, 3, |x, y| {
        let a = s.get(x, y, 3);
        [s.get(x, y, 0) * a, s.get(x, y, 1) * a, s.get(x, y, 2) * a, 1.0]
    })
}

fn manifest(kit: &Kit, snake: &str, rows: &[Row], preview: bool) -> Value {
    json!({
        "schema": 1,
        "dash": kit.key,
        "snake": snake,
        "build": kit.build,
        "preview": preview,
        "made_by": "asset-forge prebuild (tools/asset-forge/src/prebuild/)",
        "drive_folder": format!("{}/{snake}/prebuild", crate::lane::MASTERS_FOLDER),
        "items": rows.iter().map(|r| json!({
            "name": r.name, "kind": r.kind, "master": r.master, "runtime": r.runtime,
            "dims": r.dims, "fingerprint_fnv64": r.fp, "generator": r.generator,
            "licence": r.licence, "use": r.use_,
        })).collect::<Vec<_>>(),
        "deferred": kit.deferred,
        "notes": kit.notes,
    })
}

fn readme(kit: &Kit, snake: &str, rows: &[Row]) -> String {
    let mut md = format!(
        "# {} pre-built assets\n\nMade before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild {snake}` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `{}/{snake}/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.\n\n{}\n\n| Name | Kind | Size | Master (Drive) | Use | Generator |\n|---|---|---|---|---|---|\n",
        kit.key,
        crate::lane::MASTERS_FOLDER,
        kit.notes
    );
    for r in rows {
        let _ = writeln!(
            md,
            "| `{}` | {} | {} | {} | {} | {} |",
            r.name,
            r.kind,
            r.dims,
            r.master.as_deref().unwrap_or("(not copied)"),
            r.use_,
            r.generator.replace('|', "/")
        );
    }
    if !kit.deferred.is_empty() {
        md.push_str("\n## Left to the dash thread (needs the Penpot layout)\n\n");
        for d in &kit.deferred {
            let _ = writeln!(md, "- {d}");
        }
    }
    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hero::mat::{self, metal};

    #[test]
    fn regular_patterns_get_an_even_number_of_whole_repeats() {
        for m in [
            mat::carbon(10.0),
            mat::carbon(12.0),
            mat::guilloche(metal::STEEL, "clous", 12.0),
        ] {
            let span = snapped_span(&m);
            let kind = m.v.get("kind").and_then(Value::as_str).unwrap_or_default().to_owned();
            let p = if kind == "carbon" {
                m.v.get("tow").and_then(Value::as_f64).unwrap_or(0.0) as f32 * 4.0
            } else {
                12.0
            };
            let n = span / p;
            assert!((n - n.round()).abs() < 1e-4, "{kind}: {n} repeats");
            assert_eq!(n.round() as i64 % 2, 0, "{kind}: odd repeat count {n}");
            assert!(
                (span - SWATCH_SPAN).abs() <= p,
                "{kind}: span {span} strays from {SWATCH_SPAN}"
            );
        }
        assert_eq!(snapped_span(&mat::brushed(metal::STEEL)), SWATCH_SPAN);
    }

    #[test]
    fn blended_normals_are_unit_length_again() {
        let mut d = Img::from_fn(2, 1, 3, |x, _| {
            if x == 0 {
                [0.75, 0.5, 0.75, 1.0]
            } else {
                [0.5, 0.5, 1.0, 1.0]
            }
        });
        renormalize_normal(&mut d);
        for x in 0..2 {
            let v: Vec<f32> = (0..3).map(|c| d.get(x, 0, c) * 2.0 - 1.0).collect();
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            assert!((l - 1.0).abs() < 1e-5, "length {l}");
        }
    }

    #[test]
    fn every_kit_in_the_order_exists_and_names_are_unique() {
        for snake in kits::ORDER {
            let k = kits::kit(snake).unwrap_or_default();
            assert!(!k.key.is_empty(), "{snake} has no kit");
            let mut names: Vec<&str> = k.textures.iter().map(|t| t.recipe.name).collect();
            names.extend(k.swatches.iter().map(|s| s.name));
            names.extend(k.sprites.iter().map(|s| s.name));
            names.extend(k.computed.iter().map(|c| c.name));
            let n = names.len();
            names.sort_unstable();
            names.dedup();
            assert_eq!(names.len(), n, "{snake} repeats a file name");
            assert!(n > 0, "{snake} has an empty kit");
        }
    }
}
