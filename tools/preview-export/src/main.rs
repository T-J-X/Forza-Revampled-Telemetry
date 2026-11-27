//! Headless 4K preview exporter on `fh6-kit`.
//!
//! For every cluster in `fh6_clusters::registry::REGISTRY` and each of the four review states it
//! warms the cluster up through `update` with a real `dt` (springs settle, strobes run), renders
//! one 3840x2160 frame on the GPU exactly as the overlay would, and writes:
//! - `mockups/kit/<Cluster>/<state>.png`: the 4K frame over OLED black;
//! - `mockups/_thumbs/kit/<Cluster>/<state>.png`: a 960x540 box-downsample of that frame;
//! - `mockups/index.html`: a script-free gallery with badges computed from the renders (GPU time,
//!   draw calls, colour budget shares);
//! - `mockups/osd/`: the kit OSD (every tab, focus, slider, keyboard and remap states) with
//!   GPU timings; `--backdrop PNG` also writes each state composited over a game frame;
//! - the OEM scrim at 0/50/100% on the kit (`fit::scrims`).
//!
//! `--fit` renders only the Phase 8 cockpit fit review into `mockups/fit/` (see `fit.rs`).
//!
//! `--cluster <Name>` renders only one catalog cluster's review set into `mockups/<Theme>/`
//! (4 states per view at 4K, crops, in-cockpit composites, the 60-frame sequence; `cluster.rs`).
//!
//! `--frames N` sets the warm-up length (default 30 frames at 60 Hz).
mod cluster;
mod fit;
mod gallery;
mod osd;
mod states;

use fh6_clusters::registry::REGISTRY;
use fh6_core::AppConfig;
use fh6_gpu::GpuContext;
use fh6_kit::testing::{Offscreen, RgbaImage, OFFSCREEN_FORMAT};
use fh6_kit::{FrameInput, Kit, Layout, Logo, Palette, SignalSmoother, Tokens, View};
use gallery::{Badges, Render};
use states::STATES;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const W: u32 = 3840;
const H: u32 = 2160;
const THUMB_FACTOR: u32 = 4;
const DT: f32 = 1.0 / 60.0;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct Args {
    fit: bool,
    frames: u32,
    out: Option<PathBuf>,
    cluster: Option<String>,
    backdrop: Option<PathBuf>,
}

fn parse_args() -> Result<Args> {
    let mut args = Args {
        fit: false,
        frames: 30,
        out: None,
        cluster: None,
        backdrop: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--fit" => args.fit = true,
            "--frames" => args.frames = it.next().ok_or("--frames needs a value")?.parse()?,
            "--out" => args.out = Some(PathBuf::from(it.next().ok_or("--out needs a value")?)),
            "--cluster" => args.cluster = Some(it.next().ok_or("--cluster needs a name")?),
            "--backdrop" => args.backdrop = Some(PathBuf::from(it.next().ok_or("--backdrop needs a value")?)),
            "-h" | "--help" => {
                println!("preview-export [--fit] [--cluster NAME] [--frames N] [--out DIR] [--backdrop PNG]");
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument {other:?}").into()),
        }
    }
    Ok(args)
}

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("preview-export failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    // GPU validation errors and font warnings arrive as tracing events: show them.
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_writer(std::io::stderr)
        .try_init();
    let args = parse_args()?;
    let root = workspace_root();
    let out = args.out.clone().unwrap_or_else(|| root.join("mockups"));
    fs::create_dir_all(&out)?;

    let tokens = Tokens::load(&root.join("data/design_tokens.json")).unwrap_or_else(|e| {
        eprintln!("tokens: {e}; using built-in defaults");
        Tokens::default()
    });
    let ctx = GpuContext::headless()?;
    println!("GPU: {}", ctx.adapter_summary());
    let mut kit = Kit::new(&ctx.device, &ctx.queue, tokens.clone())?;
    let timing = kit.set_gpu_timing(true);
    let assets = root.join("assets");
    if assets.join("fonts").is_dir() {
        let faces = kit.fonts_mut().load_dir(&assets.join("fonts"))?;
        println!(
            "fonts: {} faces from assets/fonts, {} roles bound",
            faces.len(),
            kit.bind_font_roles()
        );
    }
    let cfg = AppConfig {
        current_camera_index: 3,
        custom_driver_name: "TJX".to_string(),
        ..Default::default()
    };
    let logo = Logo {
        image: None,
        brand_name: String::new(),
        custom_name: cfg.custom_driver_name.clone(),
    };
    if let Some(name) = args.cluster.as_deref() {
        return cluster::run(&ctx, &mut kit, &tokens, &root, &out, name, args.frames, timing);
    }
    if args.fit {
        fit::run(&ctx, &mut kit, &tokens, &root, &out, args.frames)?;
        println!("Fit review written to {}", out.join("fit").display());
        return Ok(());
    }
    let target = Offscreen::new(&ctx.device, W, H);
    let layout = Layout::from_config(&cfg, &tokens, View::Cockpit, (W, H));
    let fonts = kit.font_choice(None);
    let mut renders = Vec::new();

    for entry in REGISTRY {
        let name = entry.theme.to_string();
        let palette = Palette::family(&tokens, entry.family);
        for state in &STATES {
            let started = Instant::now();
            let mut cluster = {
                let mut setup = kit.setup(&ctx.device, &ctx.queue, Some(&assets));
                (entry.make)(&mut setup)
            };
            kit.set_palette(palette);
            let mut smoother = SignalSmoother::new();
            let data = state.telemetry();
            let mut t = 0.0;
            for _ in 0..=args.frames {
                let signals = smoother.update(&data, &cfg, &tokens, DT);
                cluster.update(&FrameInput {
                    telemetry: &data,
                    signals: &signals,
                    dt: DT,
                    t,
                    config: &cfg,
                    palette: &palette,
                    tokens: &tokens,
                    fonts,
                    logo: &logo,
                    view: View::Cockpit,
                    reduced_flash: false,
                });
                t += DT;
            }
            // Repeat the frame until every glyph it uses is in the atlas (16 are added per frame).
            let mut img = RgbaImage {
                width: 0,
                height: 0,
                pixels: Vec::new(),
            };
            let mut stats = fh6_kit::FrameStats::default();
            let mut gpu_ms = None;
            let mut missing = 1;
            for _ in 0..16 {
                if missing == 0 {
                    break;
                }
                {
                    let mut p = kit.begin_frame(&layout);
                    cluster.draw(&mut p, &layout);
                }
                missing = kit.scene().missing_glyph_count();
                let mut encoder = ctx.device.create_command_encoder(&Default::default());
                stats = kit.render(
                    &ctx.device,
                    &ctx.queue,
                    &mut encoder,
                    target.view(),
                    OFFSCREEN_FORMAT,
                    (W, H),
                    1.0,
                );
                target.copy_out(&mut encoder);
                ctx.queue.submit(Some(encoder.finish()));
                img = target.read(&ctx.device)?;
                if timing {
                    gpu_ms = kit.gpu_time(&ctx.device).map(|d| d.as_secs_f64() * 1000.0);
                }
            }
            if ctx.has_failed() {
                return Err(format!("wgpu reported an error rendering {name} {}", state.name).into());
            }
            let full = out.join("kit").join(&name).join(format!("{}.png", state.name));
            let thumb = out.join("_thumbs/kit").join(&name).join(format!("{}.png", state.name));
            let shown = img.over([0, 0, 0]);
            shown.save_png(&full)?;
            shown.downsample(THUMB_FACTOR).save_png(&thumb)?;
            // A 1:1 crop around the cluster for close review.
            let (cx, cy) = (layout.anchor.x as u32, layout.anchor.y as u32);
            let crop = shown.crop(cx.saturating_sub(800), cy.saturating_sub(450), 1600, 760);
            crop.save_png(&out.join("kit").join(&name).join(format!("{}_crop.png", state.name)))?;
            let badges = Badges::compute(&img, &palette, &tokens, &stats, gpu_ms);
            println!(
                "{name:<22} {:<18} {:>6.3} ms GPU  {:>3} draws  {:>4} instances  ({:.1} s)",
                state.name,
                gpu_ms.unwrap_or(f64::NAN),
                stats.draw_calls,
                stats.instances,
                started.elapsed().as_secs_f32()
            );
            renders.push(Render {
                cluster: name.clone(),
                display: entry.name.to_string(),
                state: state.name.to_string(),
                full: rel(&out, &full),
                thumb: rel(&out, &thumb),
                badges,
            });
        }
    }

    let backdrop = match &args.backdrop {
        Some(p) => {
            let (width, height, pixels) = fh6_kit::images::decode_png(&fs::read(p)?)?;
            Some(RgbaImage { width, height, pixels })
        }
        None => None,
    };
    let osd_renders = osd::render_all(&ctx, &mut kit, &tokens, &assets, &out, timing, backdrop.as_ref())?;
    fit::scrims(&ctx, &mut kit, &tokens, &root, &out)?;
    gallery::write_index(&out, &renders, &osd_renders, ctx.adapter_summary().to_string().as_str())?;
    verify(&out, &renders, &osd_renders)?;
    println!("Export complete: {} kit renders in {}", renders.len(), out.display());
    Ok(())
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root).unwrap_or(p).to_string_lossy().replace('\\', "/")
}

fn verify(out: &Path, renders: &[Render], osd: &[osd::OsdRender]) -> Result<()> {
    for r in osd {
        for f in [&r.full, &r.thumb] {
            if !out.join(f).is_file() {
                return Err(format!("missing {f}").into());
            }
        }
    }
    for r in renders {
        for f in [&r.full, &r.thumb] {
            let p = out.join(f);
            if !p.is_file() {
                return Err(format!("missing {}", p.display()).into());
            }
        }
        if r.badges.covered == 0 {
            return Err(format!("{} {} rendered nothing", r.cluster, r.state).into());
        }
    }
    for f in [
        "Scrim/OemScrim_0pct.png",
        "Scrim/OemScrim_50pct.png",
        "Scrim/OemScrim_100pct.png",
        "index.html",
    ] {
        if !out.join(f).is_file() {
            return Err(format!("missing {f}").into());
        }
    }
    Ok(())
}
