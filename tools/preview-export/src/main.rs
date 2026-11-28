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
//! `--car ORDINAL` picks the car whose brand logo every logo zone shows, loaded from
//! `assets/logos/` in one call (`LogoIndex::load_dial`; the app loads the same dial on a worker
//! thread) (default 3667, the 2021 Porsche 911 GT3; 0 for no car,
//! which shows the custom name).
mod cluster;
mod fit;
mod gallery;
mod osd;
mod states;

use fh6_clusters::registry::REGISTRY;
use fh6_core::{AppConfig, CarDatabase};
use fh6_gpu::GpuContext;
use fh6_kit::logo::{BrandId, LogoIndex};
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
/// The car whose logo the previews show unless `--car` says otherwise: the 2021 Porsche 911 GT3.
const DEFAULT_CAR: i32 = 3667;
/// The driver's custom name in every preview config: what a dash's greeting says, and what the
/// logo zone falls back to.
pub(crate) const DRIVER_NAME: &str = "TJX";

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct Args {
    fit: bool,
    frames: u32,
    out: Option<PathBuf>,
    cluster: Option<String>,
    backdrop: Option<PathBuf>,
    car: i32,
}

/// Reads the arguments after the program's name.
fn parse_args(mut it: impl Iterator<Item = String>) -> Result<Args> {
    let mut args = Args {
        fit: false,
        frames: 30,
        out: None,
        cluster: None,
        backdrop: None,
        car: DEFAULT_CAR,
    };
    while let Some(a) = it.next() {
        match a.as_str() {
            "--fit" => args.fit = true,
            "--frames" => args.frames = it.next().ok_or("--frames needs a value")?.parse()?,
            "--out" => args.out = Some(PathBuf::from(it.next().ok_or("--out needs a value")?)),
            "--cluster" => args.cluster = Some(it.next().ok_or("--cluster needs a name")?),
            "--backdrop" => args.backdrop = Some(PathBuf::from(it.next().ok_or("--backdrop needs a value")?)),
            "--car" => {
                let value = it.next().ok_or("--car needs an ordinal")?;
                args.car = value.parse().map_err(|e| format!("--car {value:?}: {e}"))?;
            }
            "-h" | "--help" => {
                println!(
                    "preview-export [--fit] [--cluster NAME] [--frames N] [--out DIR] [--backdrop PNG] [--car ORDINAL]"
                );
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
            let mut cause = e.source();
            while let Some(c) = cause {
                eprintln!("  caused by: {c}");
                cause = c.source();
            }
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
    let args = parse_args(std::env::args().skip(1))?;
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
        custom_driver_name: DRIVER_NAME.to_string(),
        ..Default::default()
    };
    // The whole run draws the OSD states, which need the car database; `--cluster` and `--fit`
    // use it only to name the car's brand.
    let whole_run = args.cluster.is_none() && !args.fit;
    let cars = if whole_run || args.car != 0 {
        load_cars(&assets, whole_run)?
    } else {
        None
    };
    let logo = car_logo(&ctx, &mut kit, &assets, cars.as_ref(), args.car);
    if let Some(name) = args.cluster.as_deref() {
        return cluster::run(&ctx, &mut kit, &tokens, &root, &out, name, args.frames, timing, &logo);
    }
    if args.fit {
        fit::run(&ctx, &mut kit, &tokens, &root, &out, args.frames, &logo)?;
        println!("Fit review written to {}", out.join("fit").display());
        return Ok(());
    }
    let target = Offscreen::new(&ctx.device, W, H);
    let layout = Layout::from_config(&cfg, &tokens, View::Cockpit, (W, H));
    let fonts = kit.font_choice(None);
    let mut renders = Vec::new();

    for entry in REGISTRY {
        let name = entry.theme.to_string();
        let palette = Palette::family(&tokens, entry.family.key());
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
    let cars = cars.as_ref().ok_or("the car database was not read")?;
    let osd_renders = osd::render_all(
        &ctx,
        &mut kit,
        &tokens,
        &assets,
        cars,
        &out,
        timing,
        backdrop.as_ref(),
        &logo,
    )?;
    fit::scrims(&ctx, &mut kit, &tokens, &root, &out)?;
    gallery::write_index(&out, &renders, &osd_renders, ctx.adapter_summary().to_string().as_str())?;
    verify(&out, &renders, &osd_renders)?;
    println!("Export complete: {} kit renders in {}", renders.len(), out.display());
    Ok(())
}

/// The car database (`assets/car_brands/car_brands.json`), read once for the run. When the run
/// `needs` it (the whole run does, for the OSD states), a file that cannot be read ends the run
/// here, before any render. Otherwise the run carries on without it (`None`, with a note on
/// stderr), and the logo zone shows the custom name.
fn load_cars(assets: &Path, needs: bool) -> Result<Option<CarDatabase>> {
    match CarDatabase::load(assets.join("car_brands/car_brands.json")) {
        Ok(cars) => Ok(Some(cars)),
        Err(e) if needs => Err(e.into()),
        Err(e) => {
            eprintln!("cars: {}; the logo zone shows the custom name", with_cause(&e));
            Ok(None)
        }
    }
}

/// `e` and what caused it, on one line.
fn with_cause(e: &dyn Error) -> String {
    e.source()
        .map_or_else(|| e.to_string(), |cause| format!("{e}: {cause}"))
}

/// What `--car` asks the logo zone to show, before any image is loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CarChoice {
    /// The car, as the "logo:" line names it: `car 3667 (2021 Porsche 911 GT3)`.
    car: String,
    /// The brand whose dial to load, when the car has one.
    dial: Option<BrandId>,
    /// The name the wordmark falls back to (empty: the custom name shows).
    brand_name: String,
    /// Why there is no dial to load, when there is none, as it follows the car in the "logo:"
    /// line.
    why_not: &'static str,
}

/// Picks `car`'s brand as the app does ([`LogoIndex::car_brand`]). `cars` and `index` are `None`
/// when their file could not be read.
fn choose(cars: Option<&CarDatabase>, index: Option<&LogoIndex>, car: i32) -> CarChoice {
    let Some(cars) = cars else {
        return CarChoice {
            car: format!("car {car}"),
            dial: None,
            brand_name: String::new(),
            why_not: ": car_brands.json was not read",
        };
    };
    let Some(model) = cars.car(car).map(|c| c.model_name.as_str()) else {
        return CarChoice {
            car: format!("car {car}"),
            dial: None,
            brand_name: String::new(),
            why_not: " is not in car_brands.json",
        };
    };
    // Without the index a car still has its brand palette's name.
    let no_index = LogoIndex::default();
    let brand = index.unwrap_or(&no_index).car_brand(cars, car);
    CarChoice {
        car: format!("car {car} ({model})"),
        dial: brand.logo,
        brand_name: brand.name,
        why_not: if index.is_some() {
            " has no brand logo"
        } else {
            ": logos.json was not read"
        },
    }
}

/// The "logo:" line for a car that shows a name instead of a logo: why ([`CarChoice::why_not`]),
/// and the wordmark the zone draws for `names` ([`fh6_kit::logo::wordmark`]: the brand's name,
/// else the custom name).
fn no_logo_line(car: &str, why: &str, names: &Logo) -> String {
    let mut buf = [0u8; 64];
    let shown = fh6_kit::logo::wordmark(names, &mut buf);
    format!("logo: {car}{why}; the wordmark shows {shown:?}")
}

/// The logo zone for `car`, as the app fills it: the brand's dial image (loaded into `kit` once,
/// before any cluster is built), the brand name it falls back to, and the custom name. Every
/// case prints one "logo:" line on stdout. A logo index that cannot be read, or a dial that
/// cannot be loaded, leaves the names, with the reason on stderr; so does a car database that
/// could not be read (`cars` is `None`, and [`load_cars`] has said why).
fn car_logo(ctx: &GpuContext, kit: &mut Kit, assets: &Path, cars: Option<&CarDatabase>, car: i32) -> Logo {
    let custom_name = DRIVER_NAME.to_string();
    if car == 0 {
        println!("logo: no car, the custom name {custom_name:?}");
        return Logo {
            custom_name,
            ..Logo::default()
        };
    }
    let index = match LogoIndex::load(assets) {
        Ok(index) => Some(index),
        Err(e) => {
            eprintln!("logos: {e}; the logo zone shows names");
            None
        }
    };
    let choice = choose(cars, index.as_ref(), car);
    let loaded = index.as_ref().zip(choice.dial).map(|(index, id)| {
        let path = index.dial_path(id).unwrap_or_default();
        (
            index.load_dial(kit.images_mut(), &ctx.device, &ctx.queue, id),
            rel(assets, &path),
        )
    });
    let (image, why_not) = match loaded {
        Some((Ok(image), path)) => {
            println!("logo: {}, {path}", choice.car);
            (Some(image), None)
        }
        Some((Err(e), _)) => {
            eprintln!("logo of {}: {e}", choice.car);
            (None, Some(": its dial did not load"))
        }
        None => (None, Some(choice.why_not)),
    };
    let logo = Logo {
        image,
        brand_name: choice.brand_name,
        custom_name,
        loading: false,
    };
    if let Some(why) = why_not {
        println!("{}", no_logo_line(&choice.car, why, &logo));
    }
    logo
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Result<Args> {
        parse_args(list.iter().map(|a| (*a).to_string()))
    }

    #[test]
    fn car_takes_an_ordinal_and_names_a_bad_one() {
        assert_eq!(args(&[]).expect("no arguments").car, DEFAULT_CAR);
        assert_eq!(args(&["--car", "0"]).expect("--car 0").car, 0);
        assert_eq!(args(&["--fit", "--car", "3595"]).expect("--car 3595").car, 3595);
        let bad = args(&["--car", "abc"]).expect_err("not a number").to_string();
        assert!(bad.starts_with("--car \"abc\": "), "{bad}");
        let none = args(&["--cluster", "PorscheAnalog", "--car"]).expect_err("no value");
        assert_eq!(none.to_string(), "--car needs an ordinal");
    }

    /// The shipped car database and logo index, and the assets folder.
    fn shipped() -> (CarDatabase, LogoIndex, PathBuf) {
        let assets = workspace_root().join("assets");
        let cars = CarDatabase::load(assets.join("car_brands/car_brands.json")).expect("car_brands.json");
        let index = LogoIndex::load(&assets).expect("logos.json");
        (cars, index, assets)
    }

    #[test]
    fn a_car_with_a_brand_logo_gets_its_brands_dial() {
        let (cars, index, assets) = shipped();
        for (car, label, key, name, file) in [
            (
                3667,
                "car 3667 (2021 Porsche 911 GT3)",
                "PORSCHE",
                "Porsche",
                "logos/porsche_dial.png",
            ),
            (
                4122,
                "car 4122 (2025 BMW M4 CS)",
                "BMW_M",
                "BMW M Motorsport",
                "logos/bmw_m_dial.png",
            ),
        ] {
            let choice = choose(Some(&cars), Some(&index), car);
            assert_eq!((choice.car.as_str(), choice.brand_name.as_str()), (label, name));
            let dial = choice.dial.expect("the brand has a dial");
            assert_eq!(index.key(dial), Some(key));
            assert_eq!(rel(&assets, &index.dial_path(dial).expect("its file")), file);
        }
    }

    /// The "logo:" line of a choice that loads no dial.
    fn line(choice: &CarChoice, why: &str) -> String {
        let names = Logo {
            image: None,
            brand_name: choice.brand_name.clone(),
            custom_name: DRIVER_NAME.to_string(),
            loading: false,
        };
        no_logo_line(&choice.car, why, &names)
    }

    #[test]
    fn a_car_without_a_logo_says_why_and_which_name_shows() {
        let (cars, index, _) = shipped();
        // A traffic car is in the database under the generic brand: the custom name shows.
        let truck = choose(Some(&cars), Some(&index), 2713);
        assert_eq!(truck.dial, None);
        assert_eq!(
            line(&truck, truck.why_not),
            "logo: car 2713 (2014 Box Truck (Traffic)) has no brand logo; the wordmark shows \"TJX\""
        );
        let unknown = choose(Some(&cars), Some(&index), 999_999);
        assert_eq!(unknown.dial, None);
        assert_eq!(
            line(&unknown, unknown.why_not),
            "logo: car 999999 is not in car_brands.json; the wordmark shows \"TJX\""
        );
        // A database that was not read knows no car, whatever the index says.
        let no_cars = choose(None, Some(&index), DEFAULT_CAR);
        assert_eq!(no_cars.dial, None);
        assert_eq!(
            line(&no_cars, no_cars.why_not),
            "logo: car 3667: car_brands.json was not read; the wordmark shows \"TJX\""
        );
        // Without the index the car keeps its brand palette's name.
        let no_index = choose(Some(&cars), None, DEFAULT_CAR);
        assert_eq!(no_index.dial, None);
        assert_eq!(
            line(&no_index, no_index.why_not),
            "logo: car 3667 (2021 Porsche 911 GT3): logos.json was not read; the wordmark shows \"PORSCHE\""
        );
        // A dial that fails to load leaves the brand's name, cut as the wordmark cuts it.
        let m4 = choose(Some(&cars), Some(&index), 4122);
        assert_eq!(
            line(&m4, ": its dial did not load"),
            "logo: car 4122 (2025 BMW M4 CS): its dial did not load; the wordmark shows \"BMW M\""
        );
    }

    #[test]
    fn a_car_database_error_is_printed_with_its_cause() {
        let missing = workspace_root().join("assets/car_brands/no_such_file.json");
        let error = CarDatabase::load(&missing).expect_err("no such file");
        let text = with_cause(&error);
        assert!(text.starts_with("reading "), "{text}");
        assert!(
            text.contains("no_such_file.json: "),
            "the cause follows the path: {text}"
        );
    }
}
