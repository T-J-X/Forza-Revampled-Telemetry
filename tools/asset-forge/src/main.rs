//! asset-forge: rebuilds FH6 WheelDash's textures, flipbooks, models, logos and fonts into
//! `assets/` from Blender recipes, ComfyUI workflows, exact-maths generators and pinned sources.
//!
//! ```text
//! cargo run --release -p asset-forge -- doctor
//! cargo run --release -p asset-forge -- build [--set glass|fluid|scifi|classic|adas|logos|fonts|comfy|gt7_hood_pro] [--force] [--only <job>]
//! cargo run --release -p asset-forge -- sheet      # rewrite assets/contact-sheet.html and ASSETS.md
//! ```
//! `build` is idempotent: a job is skipped when its fingerprint (recipe source, parameters,
//! seed) matches `assets/forge.lock.json` and its files exist.

mod blender;
mod comfy;
mod ctx;
mod error;
// The hero library is shared: each cluster recipe uses the parts it needs, so not every builder
// has a caller inside this binary at any one time.
#[allow(dead_code)]
mod hero;
mod img;
mod ktx2w;
mod ledger;
mod net;
mod paths;
mod procedural;
mod recipes;
mod sets;
mod sheet;
mod svg;

use error::{ForgeError, Result};
use std::process::ExitCode;

const SETS: &[&str] = &[
    "glass",
    "fluid",
    "scifi",
    "classic",
    "adas",
    "fonts",
    "logos",
    "comfy",
    "gt7_hood_pro",
    "retro_cyber",
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("asset-forge: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<()> {
    let paths = paths::Paths::discover();
    match args.first().map(String::as_str) {
        Some("doctor") => doctor(&paths),
        Some("build") => {
            let mut set: Option<String> = None;
            let mut force = false;
            let mut only = None;
            let mut it = args.iter().skip(1);
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--set" => set = it.next().cloned(),
                    "--force" => force = true,
                    "--only" => only = it.next().cloned(),
                    other => return Err(ForgeError::Usage(format!("unknown build flag {other}"))),
                }
            }
            if let Some(s) = &set {
                if !SETS.contains(&s.as_str()) {
                    return Err(ForgeError::Usage(format!("--set must be one of {}", SETS.join("|"))));
                }
            }
            let mut ctx = ctx::Ctx::new(paths, force, only)?;
            for s in SETS {
                if set.as_deref().is_some_and(|want| want != *s) {
                    continue;
                }
                println!("[{s}]");
                sets::build(&mut ctx, s)?;
            }
            ctx.ledger.save()?;
            sheet::write(&ctx.paths, &ctx.ledger)?;
            println!("ledger and contact sheet updated");
            Ok(())
        }
        Some("hero-demo") => hero::demo::run(&paths, args.iter().any(|a| a == "--preview")),
        Some("hero-detail") => {
            let (Some(beauty), Some(out)) = (args.get(1), args.get(2)) else {
                return Err(ForgeError::Usage(
                    "asset-forge hero-detail <beauty.png> <out.png> [--depth depth.png] [--denoise 0.25] [--seed N] [--prompt TEXT]".into(),
                ));
            };
            let mut depth = None;
            let mut denoise = 0.25_f32;
            let mut seed = 71_003_u64;
            let mut prompt = hero::texture::DETAIL_PROMPT.to_owned();
            let mut it = args.iter().skip(3);
            while let Some(a) = it.next() {
                match (a.as_str(), it.next()) {
                    ("--depth", Some(v)) => depth = Some(std::path::PathBuf::from(v)),
                    ("--denoise", Some(v)) => denoise = v.parse().unwrap_or(denoise),
                    ("--seed", Some(v)) => seed = v.parse().unwrap_or(seed),
                    ("--prompt", Some(v)) => prompt.clone_from(v),
                    (other, _) => return Err(ForgeError::Usage(format!("unknown hero-detail flag {other}"))),
                }
            }
            let p = hero::texture::detail_pass(
                &paths,
                std::path::Path::new(beauty),
                depth.as_deref(),
                &prompt,
                denoise,
                seed,
                std::path::Path::new(out),
            )?;
            println!("wrote {}", p.display());
            Ok(())
        }
        Some("hero-versus") => {
            let (Some(board), Some(after)) = (args.get(1), args.get(2)) else {
                return Err(ForgeError::Usage("asset-forge hero-versus <board.png> <after.png> [out.png]".into()));
            };
            let out = args.get(3).map_or_else(
                || paths.root.join("docs/handover-shots/asset-pipeline-v2-vs-penpot.png"),
                std::path::PathBuf::from,
            );
            hero::demo::versus(std::path::Path::new(board), std::path::Path::new(after), &out).map(|p| println!("wrote {}", p.display()))
        }
        Some("sheet") => {
            let mut ledger = ledger::Ledger::load(&paths.assets())?;
            ledger.save()?;
            sheet::write(&paths, &ledger)
        }
        _ => Err(ForgeError::Usage(
            "asset-forge doctor | build [--set glass|fluid|scifi|classic|adas|logos|fonts|comfy|gt7_hood_pro] [--force] [--only <job>] | sheet | hero-demo [--preview] | hero-versus <board.png> <after.png> [out.png]".into(),
        )),
    }
}

/// Checks Blender, ComfyUI, the models and the GPU; fails when a required piece is missing.
fn doctor(paths: &paths::Paths) -> Result<()> {
    let mut ok = true;
    let mut check = |name: &str, pass: bool, detail: String| {
        println!("{} {name}: {detail}", if pass { "✓" } else { "✗" });
        ok &= pass;
    };
    check(
        "repo root",
        paths.root.join("assets").is_dir(),
        paths.root.display().to_string(),
    );
    match blender::version(&paths.blender) {
        Ok(v) => check(
            "blender",
            v.starts_with("Blender 5."),
            format!("{v} ({})", paths.blender.display()),
        ),
        Err(e) => check("blender", false, e.to_string()),
    }
    let py = paths.comfy_dir.join("python_embeded").join("python.exe");
    check("comfyui install", py.is_file(), paths.comfy_dir.display().to_string());
    let models = paths.comfy_models();
    let ckpt = models.join("checkpoints").join(paths::SDXL_CKPT);
    let size = std::fs::metadata(&ckpt).map(|m| m.len()).unwrap_or(0);
    check(
        "sdxl base 1.0",
        size == paths::SDXL_BYTES,
        format!("{} ({size} bytes)", ckpt.display()),
    );
    let up = models.join("upscale_models").join(paths::UPSCALER);
    let size = std::fs::metadata(&up).map(|m| m.len()).unwrap_or(0);
    check(
        "4x-UltraSharp",
        size == paths::UPSCALER_BYTES,
        format!("{} ({size} bytes)", up.display()),
    );
    match comfy::Comfy::connect(paths) {
        Ok(c) => {
            let stats = c.stats().unwrap_or_default();
            let dev = stats["devices"][0]["name"].as_str().unwrap_or("?").to_owned();
            let vram = stats["devices"][0]["vram_total"].as_u64().unwrap_or(0) / 1_048_576;
            check(
                "comfyui api",
                true,
                format!("{} up; {dev}, {vram} MB VRAM", paths.comfy_url),
            );
            // Optional models (asset pipeline v2): reported, never required.
            let m = hero::texture::Models::probe(&c);
            let show = |o: &Option<String>| o.clone().unwrap_or_else(|| "not installed (optional)".into());
            println!("· sdxl refiner: {}", show(&m.refiner));
            println!("· controlnet depth: {}", show(&m.depth));
            println!("· controlnet canny: {}", show(&m.canny));
        }
        Err(e) => check("comfyui api", false, e.to_string()),
    }
    let wf = paths.forge_dir().join("workflows").join("sdxl_plate.json");
    check("workflow sdxl_plate.json", wf.is_file(), wf.display().to_string());
    if ok {
        println!("doctor: all checks passed");
        Ok(())
    } else {
        Err(error::tool_err("doctor", "one or more checks failed"))
    }
}
