//! asset-forge: rebuilds FH6 WheelDash's textures, flipbooks, models, logos and fonts into
//! `assets/` from Blender recipes, ComfyUI workflows, exact-maths generators and pinned sources.
//!
//! ```text
//! cargo run --release -p asset-forge -- doctor
//! cargo run --release -p asset-forge -- build [--set glass|fluid|scifi|classic|adas|logos|fonts|comfy|<cluster set>] [--force] [--only <job>]
//! cargo run --release -p asset-forge -- sheet      # rewrite assets/contact-sheet.html and ASSETS.md
//! ```
//! `build` is idempotent: a job is skipped when its fingerprint (recipe source, parameters,
//! seed) matches `assets/forge.lock.json` and its files exist.
//!
//! GPU jobs run on one of two lanes. `--lane cloud` (or `FH6_GPU_LANE=cloud`) before any command
//! sends its ComfyUI and Blender work to the rented pod; `cloud` manages that pod. `--masters
//! <snake_name>` also copies the 4K outputs to the dash's Google Drive masters folder.
//!
//! ```text
//! cargo run --release -p asset-forge -- cloud setup|up|down|status|spend|doctor
//! cargo run --release -p asset-forge -- --lane cloud --masters <snake_name> build --set <name>
//! cargo run --release -p asset-forge -- [--lane cloud] selftest
//! ```

mod blender;
mod cloud;
mod comfy;
mod ctx;
mod error;
// The hero library is shared: each cluster recipe uses the parts it needs, so not every builder
// has a caller inside this binary at any one time.
#[allow(dead_code)]
mod hero;
mod img;
mod ktx2w;
mod lane;
mod ledger;
mod net;
mod paths;
mod prebuild;
mod procedural;
mod recipes;
mod selftest;
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
    "tesla_adas",
    "gt7_hood_pro",
    "retro_cyber",
    "bosch_ddu",
    "liquid_glass",
    "porsche_analog",
    "osd",
    "vision_spatial_glass",
    "amg_mbux_superscreen",
    "drift_glass",
    "glyph_dot_matrix",
    "pagani_horologerie",
    "bugatti_tourbillon",
    "lfa_v10_nurburgring",
    "koenigsegg_gyro",
    "valkyrie_lmp1",
    "gt7_modular_tripod",
    "windshield_hud",
    "ar_nav_wayfinder",
    "f35_helmet_hmd",
    "synthwave_outrun",
    "eva_bio_sync",
    "chrono_event_horizon",
    "neural_cyber_synapse",
    "solaris_warp_core",
    "aero_plasma_blade",
    "shibuya_chromatic_prisma",
    "kinetic_liquid_metal",
    "cymatic_resonance_field",
    "chrono_multiverse",
    "lumen_etch",
    "vernier_loupe",
    "dichroic_louvre",
    "fibre_optic_corona",
    "aperture_nine",
    "aero_smoke_tunnel",
    "rattrapante_sector",
    "sapphire_mystery",
    "rack_focus",
    "bionic_spaceframe",
    "geodesic_well",
    "lightpainter",
    "tread_seismograph",
    "star_sail_origami",
    "arc_ladder_cascade",
    "aurora_veil",
    "isochromatic_load_path",
    "abyssal_shear_bloom",
    "harmonograph_drive",
    "amg_mbux_supersport",
    "skyline_r34_mfd",
    "vision_petals",
    "amg_one_f1",
    "carplay_island",
    "gt3_rs_apogee",
    "gt7_cockpit_pro",
    "liquid_morph",
    "amg_mbux_track_pace",
    "gtr_nismo_mfd",
    "sim_input_trace",
    "lorenz_butterfly",
    "amg_mbux_classic",
    "suminagashi_marble",
    "t50_floodlit",
    "hakosuka_s20",
    "sim_pit_wall",
    "kaleidoscope_mirror",
    "revuelto_corsa",
    "z_nismo_sport",
    "golden_angle",
    "mclaren_slim_display",
    "hopf_fibration",
    "hyperbolic_tiles",
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

/// Takes `--lane <local|cloud>` and `--masters <snake_name>` out of the arguments (they may
/// stand anywhere) and applies them, or `FH6_GPU_LANE` and `FH6_MASTERS` when they are absent.
fn globals(paths: &mut paths::Paths, args: &[String]) -> Result<Vec<String>> {
    globals_from(
        paths,
        args,
        std::env::var("FH6_GPU_LANE").ok(),
        std::env::var("FH6_MASTERS").ok(),
    )
}

/// [`globals`] with the two environment variables passed in, so tests do not depend on the
/// environment they run in.
fn globals_from(
    paths: &mut paths::Paths,
    args: &[String],
    env_lane: Option<String>,
    env_masters: Option<String>,
) -> Result<Vec<String>> {
    let mut lane = env_lane;
    let mut masters = env_masters.filter(|m| !m.trim().is_empty());
    let mut rest = Vec::with_capacity(args.len());
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            flag @ ("--lane" | "--masters") => {
                let value = it
                    .next()
                    .cloned()
                    .ok_or_else(|| ForgeError::Usage(format!("{flag} needs a value")))?;
                if flag == "--lane" {
                    lane = Some(value);
                } else {
                    masters = Some(value);
                }
            }
            _ => rest.push(arg.clone()),
        }
    }
    if let Some(lane) = lane {
        paths.lane = lane::Lane::parse(&lane)?;
    }
    if let Some(name) = masters {
        paths.masters = Some(lane::masters_dir(name.trim())?);
    }
    Ok(rest)
}

fn run(args: &[String]) -> Result<()> {
    let mut paths = paths::Paths::discover();
    let args = globals(&mut paths, args)?;
    let result = dispatch(&paths, &args);
    // Closes the forward to the pod, if this run opened one. The pod's idle clock starts here.
    cloud::close();
    result
}

fn dispatch(paths: &paths::Paths, args: &[String]) -> Result<()> {
    let paths = paths.clone();
    match args.first().map(String::as_str) {
        Some("doctor") if paths.lane == lane::Lane::Cloud => cloud::cli(&paths, &["doctor".to_owned()]),
        Some("doctor") => doctor(&paths),
        Some("cloud") => cloud::cli(&paths, &args[1..]),
        Some("selftest") => selftest::run(&paths),
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
        Some("prebuild") => prebuild::cli(&paths, &args[1..]),
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
            "asset-forge [--lane local|cloud] [--masters <snake_name>] doctor | build [--set glass|fluid|scifi|classic|adas|logos|fonts|comfy|<cluster set>] [--force] [--only <job>] | sheet | selftest | prebuild --list|<snake_name>...|--all [--preview] | hero-demo [--preview] | hero-versus <board.png> <after.png> [out.png] | cloud setup|up|down|status|spend|doctor".into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The code re-cut registered a set for every dash, so a dash thread never edits the three
    /// lists: each module directory of `fh6-clusters` has its set here, and every set has its arm in
    /// `sets::build`.
    #[test]
    fn every_dash_module_has_a_set() {
        let clusters = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/fh6-clusters/src/clusters");
        let mut seen = 0;
        for dir in std::fs::read_dir(&clusters).expect("the clusters folder").flatten() {
            if !dir.path().is_dir() {
                continue;
            }
            let name = dir.file_name().to_string_lossy().into_owned();
            assert!(SETS.contains(&name.as_str()), "{name} has no asset-forge set");
            seen += 1;
        }
        assert_eq!(seen, 72);
        // `sets::build` ignores an unknown name, so a set without its arm would build nothing.
        let arms = include_str!("sets/mod.rs");
        for set in SETS {
            assert!(
                arms.contains(&format!("\"{set}\" =>")),
                "{set} has no arm in sets::build"
            );
        }
    }

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| (*a).to_owned()).collect()
    }

    fn parse(list: &[&str]) -> (Result<Vec<String>>, paths::Paths) {
        let mut paths = paths::Paths::discover();
        let rest = globals_from(&mut paths, &args(list), None, None);
        (rest, paths)
    }

    #[test]
    fn the_lane_flag_is_taken_out_wherever_it_stands() {
        let (rest, paths) = parse(&["build", "--lane", "cloud", "--set", "glass"]);
        assert_eq!(rest.unwrap(), args(&["build", "--set", "glass"]));
        assert_eq!(paths.lane, lane::Lane::Cloud);

        let (rest, paths) = parse(&["--lane", "local", "doctor"]);
        assert_eq!(rest.unwrap(), args(&["doctor"]));
        assert_eq!(paths.lane, lane::Lane::Local);

        // Without the flag, the environment decides, and the flag wins over it.
        let mut paths = paths::Paths::discover();
        globals_from(&mut paths, &args(&["doctor"]), Some("cloud".into()), None).unwrap();
        assert_eq!(paths.lane, lane::Lane::Cloud);
        let mut paths = paths::Paths::discover();
        globals_from(
            &mut paths,
            &args(&["--lane", "local", "doctor"]),
            Some("cloud".into()),
            None,
        )
        .unwrap();
        assert_eq!(paths.lane, lane::Lane::Local);
    }

    #[test]
    fn a_wrong_or_missing_lane_or_masters_name_is_refused() {
        assert!(parse(&["--lane", "moon", "doctor"]).0.is_err());
        assert!(parse(&["doctor", "--lane"]).0.is_err());
        assert!(parse(&["doctor", "--masters"]).0.is_err());
        assert!(parse(&["--masters", "Not A Dash", "doctor"]).0.is_err());
    }
}
