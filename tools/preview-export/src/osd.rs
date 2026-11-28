//! 4K kit renders of the OSD: every tab, focus and slider states, the keyboard and the
//! remapper, plus GPU timings for `docs/perf/osd.md`, the motion sequence
//! (`osd/sequence/frame_NNN.png`, for `visual-review flash` and `motion`) and the OSD's product
//! page (`osd/index.html`, no script). The OSD is driven through the same `OsdHost` the overlay
//! uses, with the live dyno feeding the live cluster and the carousel.
use crate::cluster::{inline_html, PAGE_CSS};
use crate::{rel, Result, DT, H, THUMB_FACTOR, W};
use fh6_clusters::registry;
use fh6_core::{AppConfig, CarDatabase, ClusterTheme, ColorMode, ScrimFit};
use fh6_gpu::GpuContext;
use fh6_kit::scrim::Wash;
use fh6_kit::testing::{Offscreen, RgbaImage, OFFSCREEN_FORMAT};
use fh6_kit::{Cluster, FontChoice, FrameInput, Kit, Layout, Logo, Palette, SignalSmoother, Tokens, View};
use fh6_osd::model::{self, RowId, Tab};
use fh6_osd::nav::{NavInput, DOWN, LB, RB, RIGHT, VIEW};
use fh6_osd::{draw, ConfigOnlyFit, Frame, LiveDynoGenerator, LiveStatus, OsdHost};
use std::fmt::Write as _;
use std::path::Path;
use std::time::Duration;

/// Frames rendered back to back before a state's timed frames, so the GPU is at full clocks.
const WARM_UP: u32 = 100;
/// The same for the first timed state, which starts from an idle GPU.
const FIRST_WARM_UP: u32 = 600;
/// Frames timed one by one after the warm-up (their median and minimum are reported).
const TIMED_FRAMES: u32 = 20;

/// One OSD review state.
struct OsdState {
    name: &'static str,
    caption: &'static str,
    /// The state shows the driving position; the product page features a 1:1 crop of it.
    drive: bool,
    setup: fn(&mut OsdHost, &mut AppConfig) -> Duration,
}

/// The 1:1 crop of the live zone the product page shows for the driving-position states (4K
/// px: x, y, width, height): the lifted echo and the view under it, at 16:9.
const DRIVE_CROP: [u32; 4] = [630, 400, 1120, 1060];
/// The OSD's GPU budget for a whole frame at 4K (ms).
const BUDGET_MS: f64 = 1.5;

/// The motion sequence at 60 fps, as `(frame, buttons held from that frame on)`: the menu opens
/// on the cockpit FOV row, right is held for a second (hold-to-repeat, the WCAG flash test's
/// span), released, focus moves down a row, and RB changes the tab.
const SEQUENCE: [(u32, u16); 7] = [(0, 0), (24, RIGHT), (84, 0), (108, DOWN), (112, 0), (144, RB), (148, 0)];
/// Frames in the sequence (3 s).
const SEQUENCE_FRAMES: u32 = 180;
/// The sequence's frames are the 4K frame at half size (`visual-review flash --scale 2`).
const SEQUENCE_FACTOR: u32 = 2;
/// The product page's reel plays every `REEL_STEP`th frame as a thumbnail, at half speed.
const REEL_STEP: u32 = 2;

/// The buttons held at `frame` of the sequence.
fn sequence_held(frame: u32) -> u16 {
    SEQUENCE
        .iter()
        .rev()
        .find(|(at, _)| *at <= frame)
        .map_or(0, |(_, held)| *held)
}

fn focus(host: &mut OsdHost, cfg: &AppConfig, tab: Tab, row: RowId) {
    if !host.focus_row(tab, row, cfg) {
        eprintln!("OSD preview: {row:?} is not on {tab:?} with this config; focusing the first row");
    }
}

fn press(host: &mut OsdHost, cfg: &mut AppConfig, t_ms: u64, held: u16, pressed: u16) {
    let input = NavInput {
        held,
        pressed,
        released: 0,
        now: Duration::from_millis(t_ms),
        toggle_mask: cfg.osd_toggle_mask,
        hold_ms: cfg.osd_hold_ms,
    };
    host.step(&input, cfg);
}

const STATES: &[OsdState] = &[
    OsdState {
        name: "01_cluster_carousel",
        caption: "Cluster tab · carousel focused (live thumbnails)",
        drive: false,
        setup: |h, c| {
            focus(h, c, Tab::Cluster, RowId::Cluster);
            Duration::ZERO
        },
    },
    OsdState {
        name: "02_cluster_hood_view",
        caption: "Cluster tab · editing the Hood view (live view follows)",
        drive: false,
        setup: |h, c| {
            focus(h, c, Tab::Cluster, RowId::EditView);
            press(h, c, 0, RIGHT, RIGHT);
            Duration::from_millis(100)
        },
    },
    OsdState {
        name: "03_colour_custom_hue",
        caption: "Colour tab · custom palette, hue slider focused",
        drive: false,
        setup: |h, c| {
            c.color_mode = ColorMode::Custom;
            focus(h, c, Tab::Colour, RowId::Hue);
            Duration::ZERO
        },
    },
    OsdState {
        name: "04_colour_random",
        caption: "Colour tab · random palette with re-roll and lock",
        drive: false,
        setup: |h, c| {
            c.color_mode = ColorMode::Random;
            c.palette_seed = Some(4242);
            focus(h, c, Tab::Colour, RowId::Reroll);
            Duration::ZERO
        },
    },
    OsdState {
        name: "05_type_logo",
        caption: "Type & logo tab · font choice focused",
        drive: false,
        setup: |h, c| {
            focus(h, c, Tab::Type, RowId::Font);
            Duration::ZERO
        },
    },
    OsdState {
        name: "06_fit_fov",
        caption: "Fit & camera · cockpit FOV: the echo lifts over the rendered driving position",
        drive: true,
        setup: |h, c| {
            focus(h, c, Tab::Fit, RowId::FovCockpit);
            Duration::ZERO
        },
    },
    OsdState {
        name: "07_fit_cluster_height",
        caption: "Fit & camera · cluster height slider and default-height line",
        drive: false,
        setup: |h, c| {
            c.cockpit_height_offset_px = 64.0;
            focus(h, c, Tab::Fit, RowId::ClusterHeight);
            Duration::ZERO
        },
    },
    OsdState {
        name: "08_fit_dash_blocker",
        caption: "Fit & camera · dash blocker rows (scrolled)",
        drive: false,
        setup: |h, c| {
            focus(h, c, Tab::Fit, RowId::Scrim(1));
            Duration::ZERO
        },
    },
    OsdState {
        name: "09_controls",
        caption: "Controls tab · hold time slider",
        drive: false,
        setup: |h, c| {
            focus(h, c, Tab::Controls, RowId::HoldTime);
            Duration::ZERO
        },
    },
    OsdState {
        name: "10_keyboard",
        caption: "On-screen keyboard for the custom name",
        drive: false,
        setup: |h, c| {
            focus(h, c, Tab::Type, RowId::CustomName);
            h.ctl.start_keyboard(c);
            press(h, c, 0, 0, LB);
            press(h, c, 20, RIGHT, RIGHT);
            Duration::from_millis(100)
        },
    },
    OsdState {
        name: "11_remap",
        caption: "OSD button remapper mid-capture (View + L3 held)",
        drive: false,
        setup: |h, c| {
            focus(h, c, Tab::Controls, RowId::OsdButton);
            h.ctl.start_remap(Duration::ZERO);
            press(h, c, 0, VIEW | 0x0040, VIEW | 0x0040);
            press(h, c, 600, VIEW | 0x0040, 0);
            Duration::from_millis(600)
        },
    },
    OsdState {
        name: "12_fit_seat_height",
        caption: "Fit & camera · seat height raised 20 mm: the seat lit from its rim, the eye off its default ring",
        drive: true,
        setup: |h, c| {
            c.driver_seat_height_m = 0.020;
            focus(h, c, Tab::Fit, RowId::SeatHeight);
            Duration::ZERO
        },
    },
    OsdState {
        name: "13_fit_wheel_offset",
        caption: "Fit & camera · wheel offset lowered to -70 mm: the wheel lit from its rim",
        drive: true,
        setup: |h, c| {
            c.steering_wheel_offset_y_m = -0.070;
            focus(h, c, Tab::Fit, RowId::WheelOffset);
            Duration::ZERO
        },
    },
];

/// One OSD render for the gallery and the perf report.
pub struct OsdRender {
    /// State file stem.
    pub name: &'static str,
    /// Gallery caption.
    pub caption: &'static str,
    /// Full render (relative to the output folder).
    pub full: String,
    /// Thumbnail.
    pub thumb: String,
    /// The frame over the game capture and its thumbnail (with `--backdrop`).
    pub over: Option<(String, String)>,
    /// The 1:1 crop of the driving position (the states that show it).
    pub drive: Option<String>,
    /// Median GPU time of the whole OSD frame (live cluster included), ms.
    pub gpu_ms: Option<f64>,
    /// Fastest of the timed frames (closest to an uncontended GPU), ms.
    pub gpu_min_ms: Option<f64>,
    /// Draw calls.
    pub draws: u32,
}

/// Resizes `bg` to the render size (bilinear) for the over-the-game composites.
fn fit_backdrop(bg: &RgbaImage) -> RgbaImage {
    let mut out = vec![0u8; (W * H * 4) as usize];
    let (sw, sh) = (bg.width.max(1) as f32, bg.height.max(1) as f32);
    for y in 0..H {
        let fy = ((y as f32 + 0.5) * sh / H as f32 - 0.5).clamp(0.0, sh - 1.0);
        let (y0, ty) = (fy.floor() as u32, fy.fract());
        let y1 = (y0 + 1).min(bg.height - 1);
        for x in 0..W {
            let fx = ((x as f32 + 0.5) * sw / W as f32 - 0.5).clamp(0.0, sw - 1.0);
            let (x0, tx) = (fx.floor() as u32, fx.fract());
            let x1 = (x0 + 1).min(bg.width - 1);
            let px = |xx, yy| bg.pixel(xx, yy).unwrap_or([0, 0, 0, 255]);
            let (a, b, c, d) = (px(x0, y0), px(x1, y0), px(x0, y1), px(x1, y1));
            let i = ((y * W + x) * 4) as usize;
            for ch in 0..3 {
                let top = f32::from(a[ch]) * (1.0 - tx) + f32::from(b[ch]) * tx;
                let bot = f32::from(c[ch]) * (1.0 - tx) + f32::from(d[ch]) * tx;
                out[i + ch] = (top * (1.0 - ty) + bot * ty).round() as u8;
            }
            out[i + 3] = 255;
        }
    }
    RgbaImage {
        width: W,
        height: H,
        pixels: out,
    }
}

/// The overlay frame composited over an opaque backdrop (premultiplied over).
fn over_image(fg: &RgbaImage, bg: &RgbaImage) -> RgbaImage {
    let mut pixels = bg.pixels.clone();
    for (o, f) in pixels.chunks_exact_mut(4).zip(fg.pixels.chunks_exact(4)) {
        let a = f32::from(f[3]) / 255.0;
        for c in 0..3 {
            o[c] = (f32::from(f[c]) + f32::from(o[c]) * (1.0 - a))
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
    RgbaImage {
        width: fg.width,
        height: fg.height,
        pixels,
    }
}

/// The live cluster under the OSD and what drives it, as the overlay's render loop holds them.
struct Live {
    cluster: Box<dyn Cluster>,
    palette: Palette,
    logo: Logo,
    fonts: FontChoice,
    smoother: SignalSmoother,
    dyno: LiveDynoGenerator,
    view: View,
    /// The cluster's name, for the live zone's header.
    name: &'static str,
    t: f32,
}

impl Live {
    /// The cluster `cfg` selects for `view`, with its palette set on the kit.
    fn new(
        ctx: &GpuContext,
        kit: &mut Kit,
        tokens: &Tokens,
        assets: &Path,
        cars: &CarDatabase,
        cfg: &AppConfig,
        view: View,
    ) -> Self {
        let theme = model::theme_for_view(cfg, view);
        let entry = registry::resolve(theme);
        let cluster = {
            let mut setup = kit.setup(&ctx.device, &ctx.queue, Some(assets));
            (entry.make)(&mut setup)
        };
        let palette = Palette::from_config(tokens, cfg, cars, 0, entry.family.key());
        kit.set_palette(palette);
        let mut dyno = LiveDynoGenerator::new();
        dyno.phase = 2.6;
        Self {
            cluster,
            palette,
            logo: Logo {
                image: None,
                brand_name: String::new(),
                custom_name: cfg.custom_driver_name.clone(),
            },
            fonts: kit.font_choice(None),
            smoother: SignalSmoother::new(),
            dyno,
            view,
            name: model::theme_label(theme),
            t: 0.0,
        }
    }

    /// One frame of the dyno and the live cluster's `update`, then `then` with the frame the
    /// cluster got (the OSD's own `update` or `settle`).
    fn tick(&mut self, tokens: &Tokens, cfg: &AppConfig, then: impl FnOnce(&FrameInput<'_>)) {
        let data = self.dyno.tick(DT);
        let signals = self.smoother.update(&data, cfg, tokens, DT);
        let input = FrameInput {
            telemetry: &data,
            signals: &signals,
            dt: DT,
            t: self.t,
            config: cfg,
            palette: &self.palette,
            tokens,
            fonts: self.fonts,
            logo: &self.logo,
            view: self.view,
            reduced_flash: cfg.reduced_flash,
        };
        self.cluster.update(&input);
        then(&input);
        self.t += DT;
    }
}

/// The config every OSD render starts from.
fn review_config() -> AppConfig {
    AppConfig {
        current_camera_index: 3,
        cockpit_hud_style: ClusterTheme::LiquidGlass,
        custom_driver_name: "TJX".into(),
        ..AppConfig::default()
    }
}

/// Renders every OSD state into `out/osd/` (with `only`, the states whose name contains it).
#[allow(clippy::too_many_arguments)]
pub fn render_all(
    ctx: &GpuContext,
    kit: &mut Kit,
    tokens: &Tokens,
    assets: &Path,
    out: &Path,
    timing: bool,
    backdrop: Option<&RgbaImage>,
    only: Option<&str>,
) -> Result<Vec<OsdRender>> {
    let target = Offscreen::new(&ctx.device, W, H);
    let frame = Frame::new((W, H));
    let cars = CarDatabase::load(assets.join("car_brands/car_brands.json"))?;
    let backdrop = backdrop.map(fit_backdrop);
    let mut renders = Vec::new();
    // The first timed state also brings the GPU up from idle (about a second of frames), so every
    // state is timed at the clocks it runs at while the menu is open.
    let mut warm_up = FIRST_WARM_UP;
    for state in STATES.iter().filter(|s| only.is_none_or(|o| s.name.contains(o))) {
        let mut cfg = review_config();
        let mut host = OsdHost::new(tokens, Box::new(ConfigOnlyFit));
        host.set_fonts(kit.fonts().families());
        host.open(&cfg, View::Cockpit, cfg.cockpit_anchor_x as f32);
        let now = (state.setup)(&mut host, &mut cfg);
        let view = host.edit_view();
        host.prepare(kit, &ctx.device, &ctx.queue, Some(assets), &cfg);
        let mut live = Live::new(ctx, kit, tokens, assets, &cars, &cfg, view);
        let palette = live.palette;
        let base = Layout::from_config(&cfg, tokens, view, (W, H));
        for i in 0..45 {
            live.tick(tokens, &cfg, |input| {
                if i == 44 {
                    host.settle(input, kit, frame, &cfg);
                } else {
                    host.update(input, kit, frame, &cfg);
                }
            });
        }
        let layout = host.live_layout(&base, frame);
        let name = live.name;
        // As in the overlay: the real dash blocker under the live cluster while a blocker row
        // has focus (Hood and chase resolve to opacity 0, so the soft pool stays).
        let blocker = Some(ScrimFit::from_config(&cfg, view.fit_view()))
            .filter(|fit| host.blocker_focused(&cfg) && fit.visible())
            .map(|fit| (fit, Wash::from_config(&cfg, palette.primary)));
        let mut img = RgbaImage {
            width: 0,
            height: 0,
            pixels: Vec::new(),
        };
        let mut gpu = Vec::new();
        let mut draws = 0;
        let mut passes = 0;
        let mut segments = 0;
        // Frame 1 queues missing glyphs; later frames are steady state and timed.
        // Timing: `warm_up` frames back to back (the GPU clocks up), then 20 timed frames.
        let frames = if timing { warm_up + TIMED_FRAMES } else { 2 };
        for f in 0..frames {
            {
                let mut p = kit.begin_frame(&layout);
                let backing = blocker.as_ref().map(|(fit, wash)| (fit, *wash));
                draw::live_backing(&mut p, backing);
                live.cluster.draw(&mut p, &layout);
                host.draw(&mut p, &cfg, frame, now, name, LiveStatus::Dyno);
            }
            kit.set_underlay(draw::underlay(host.anim.open.progress()));
            kit.set_pools(host.pools(&layout, frame, blocker.is_none()).as_slice());
            let mut encoder = ctx.device.create_command_encoder(&Default::default());
            let stats = kit.render(
                &ctx.device,
                &ctx.queue,
                &mut encoder,
                target.view(),
                OFFSCREEN_FORMAT,
                (W, H),
                1.0,
            );
            draws = stats.draw_calls;
            passes = stats.passes;
            segments = kit.scene().segment_count();
            let last = f + 1 == frames;
            if last {
                target.copy_out(&mut encoder);
            }
            ctx.queue.submit(Some(encoder.finish()));
            if last {
                img = target.read(&ctx.device)?;
            }
            if timing && f >= warm_up {
                if let Some(d) = kit.gpu_time(&ctx.device) {
                    gpu.push(d.as_secs_f64() * 1000.0);
                }
            }
        }
        if ctx.has_failed() {
            return Err(format!("wgpu reported an error rendering OSD {}", state.name).into());
        }
        warm_up = WARM_UP;
        gpu.sort_by(f64::total_cmp);
        let gpu_ms = gpu.get(gpu.len() / 2).copied();
        let gpu_min_ms = gpu.first().copied();
        let full = out.join("osd").join(format!("{}.png", state.name));
        let thumb = out.join("_thumbs/osd").join(format!("{}.png", state.name));
        let shown = img.over([0, 0, 0]);
        shown.save_png(&full)?;
        shown.downsample(THUMB_FACTOR).save_png(&thumb)?;
        let over_game = backdrop.as_ref().map(|bg| over_image(&img, bg));
        let over = match &over_game {
            Some(o) => {
                let full = out.join("osd").join(format!("{}_over_game.png", state.name));
                let thumb = out.join("_thumbs/osd").join(format!("{}_over_game.png", state.name));
                o.save_png(&full)?;
                o.downsample(THUMB_FACTOR).save_png(&thumb)?;
                Some((rel(out, &full), rel(out, &thumb)))
            }
            None => None,
        };
        let drive = if state.drive {
            let [x, y, w, h] = DRIVE_CROP;
            let path = out.join("osd").join(format!("{}_drive.png", state.name));
            over_game.as_ref().unwrap_or(&shown).crop(x, y, w, h).save_png(&path)?;
            Some(rel(out, &path))
        } else {
            None
        };
        println!(
            "OSD {:<24} {:>6.3} ms GPU median  {:>6.3} min  {:>3} draws  {:>2} passes  {:>2} segments",
            state.name,
            gpu_ms.unwrap_or(f64::NAN),
            gpu_min_ms.unwrap_or(f64::NAN),
            draws,
            passes,
            segments
        );
        renders.push(OsdRender {
            name: state.name,
            caption: state.caption,
            full: rel(out, &full),
            thumb: rel(out, &thumb),
            over,
            drive,
            gpu_ms,
            gpu_min_ms,
            draws,
        });
    }
    clear_backing(kit);
    Ok(renders)
}

/// Takes the OSD's dim and pools off the kit, so what is rendered next (the scrim sheets, a
/// cluster) does not get them.
fn clear_backing(kit: &mut Kit) {
    kit.set_underlay(0.0);
    kit.set_pools(&[]);
}

/// Renders the motion sequence ([`SEQUENCE`]) over OLED black into `out/osd/sequence/`, every
/// frame at half size, and returns the product page's reel: every [`REEL_STEP`]th frame as a
/// thumbnail (paths relative to `out`). The OSD is stepped as the overlay's render loop steps
/// it: input, `prepare`, the live cluster's `update`, the OSD's `update`, then the draw.
pub fn render_sequence(
    ctx: &GpuContext,
    kit: &mut Kit,
    tokens: &Tokens,
    assets: &Path,
    out: &Path,
) -> Result<Vec<String>> {
    let target = Offscreen::new(&ctx.device, W, H);
    let frame = Frame::new((W, H));
    let cars = CarDatabase::load(assets.join("car_brands/car_brands.json"))?;
    let mut cfg = review_config();
    let view = View::Cockpit;
    let mut live = Live::new(ctx, kit, tokens, assets, &cars, &cfg, view);
    // The dash is already running when the menu opens.
    for _ in 0..45 {
        live.tick(tokens, &cfg, |_| {});
    }
    let mut host = OsdHost::new(tokens, Box::new(ConfigOnlyFit));
    host.set_fonts(kit.fonts().families());
    host.open(&cfg, view, cfg.cockpit_anchor_x as f32);
    focus(&mut host, &cfg, Tab::Fit, RowId::FovCockpit);
    let base = Layout::from_config(&cfg, tokens, view, (W, H));
    let dir = out.join("osd").join("sequence");
    let thumbs = out.join("_thumbs/osd/sequence");
    std::fs::create_dir_all(&dir)?;
    std::fs::create_dir_all(&thumbs)?;
    let mut reel = Vec::new();
    let mut before = 0;
    for f in 0..SEQUENCE_FRAMES {
        let held = sequence_held(f);
        let now = Duration::from_secs_f32(f as f32 * DT);
        let input = NavInput {
            held,
            pressed: held & !before,
            released: before & !held,
            now,
            toggle_mask: cfg.osd_toggle_mask,
            hold_ms: cfg.osd_hold_ms,
        };
        before = held;
        host.step(&input, &mut cfg);
        host.prepare(kit, &ctx.device, &ctx.queue, Some(assets), &cfg);
        live.tick(tokens, &cfg, |input| host.update(input, kit, frame, &cfg));
        let layout = host.live_layout(&base, frame);
        {
            let mut p = kit.begin_frame(&layout);
            live.cluster.draw(&mut p, &layout);
            host.draw(&mut p, &cfg, frame, now, live.name, LiveStatus::Dyno);
        }
        kit.set_underlay(draw::underlay(host.anim.open.progress()));
        kit.set_pools(host.pools(&layout, frame, true).as_slice());
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        kit.render(
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
        let shown = target.read(&ctx.device)?.over([0, 0, 0]);
        let file = format!("frame_{f:03}.png");
        shown.downsample(SEQUENCE_FACTOR).save_png(&dir.join(&file))?;
        if f % REEL_STEP == 0 {
            let thumb = thumbs.join(&file);
            shown.downsample(THUMB_FACTOR).save_png(&thumb)?;
            reel.push(rel(out, &thumb));
        }
    }
    clear_backing(kit);
    if ctx.has_failed() {
        return Err("wgpu reported an error rendering the OSD sequence".into());
    }
    println!(
        "OSD sequence: {SEQUENCE_FRAMES} frames at 60 fps written to {}",
        dir.display()
    );
    Ok(reel)
}

/// A path relative to the export folder, as `osd/index.html` links it.
fn from_page(p: &str) -> String {
    p.strip_prefix("osd/").map_or_else(|| format!("../{p}"), str::to_owned)
}

/// `s` with its first letter in upper case (a state's caption after its tab, as a card title).
fn capitalised(s: &str) -> String {
    let mut chars = s.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// Writes `osd/index.html`: the OSD's product page (no script). A hero with the OSD over the
/// game, the motion sequence as a looping reel (`reel`, from [`render_sequence`]), the driving
/// position the camera rows show, every state as a card, and the GPU table.
pub fn write_page(out: &Path, renders: &[OsdRender], reel: &[String], adapter: &str) -> std::io::Result<()> {
    let ms = |v: Option<f64>| v.map_or_else(|| "n/a".to_string(), |v| format!("{v:.3} ms"));
    let worst = renders.iter().filter_map(|r| r.gpu_ms).reduce(f64::max);
    let shown = |r: &OsdRender| match &r.over {
        Some((full, thumb)) => (from_page(full), from_page(thumb)),
        None => (from_page(&r.full), from_page(&r.thumb)),
    };
    let hero = renders.first().map(shown).unwrap_or_default();
    let mut h = String::new();
    let _ = write!(
        h,
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>Liquid Glass OSD · FH6 WheelDash</title><style>{PAGE_CSS}\
         .grid3{{display:grid;grid-template-columns:repeat(3,1fr);gap:20px}}\
         @media (max-width:1200px){{.grid3{{grid-template-columns:1fr}}}}\
         .reel.osd img{{animation-name:osdreel;animation-duration:{reel_s}s;\
         animation-delay:calc(var(--i)*{reel_s}s/{reel_n})}}\
         @keyframes osdreel{{0%{{opacity:1}}{reel_pct:.4}%{{opacity:0}}100%{{opacity:0}}}}</style></head><body>\
         <header class=\"bar\"><span class=\"mark\">FH6 WHEELDASH</span>\
         <span class=\"crumb\">Settings menu</span><span class=\"pill\">Liquid Glass</span></header><main>\
         <section class=\"hero\"><div><p class=\"eyebrow\">Settings menu · controller first</p>\
         <h1>Liquid Glass OSD</h1><p class=\"lede\">Hold View to open it over the paused game: the live \
         cluster on the left, five tabs of clear synthetic sapphire on the right, and every change lands on the \
         live view at once. It reads the D-pad, LB, RB, View and Menu only; A, B, X and Y stay with your \
         gearbox.</p><dl class=\"facts\">\
         <div><dt>GPU at 4K</dt><dd>{worst}</dd><dd class=\"sub\">slowest of {n} states · budget {BUDGET_MS} ms</dd></div>\
         <div><dt>Tabs</dt><dd>5</dd></div><div><dt>Opens with</dt><dd>View, held</dd></div></dl></div>\
         <a class=\"stage\" href=\"{full}\"><img src=\"{thumb}\" alt=\"The OSD over the paused game\"></a></section>",
        worst = ms(worst),
        n = renders.len(),
        full = hero.0,
        thumb = hero.1,
        // Half speed: each reel frame stands for `REEL_STEP` frames of 1/60 s.
        reel_s = reel.len() as f32 * REEL_STEP as f32 * DT * 2.0,
        reel_n = reel.len().max(1),
        reel_pct = 100.0 / reel.len().max(1) as f32,
    );
    if !reel.is_empty() {
        h.push_str(
            "<section><h2>In motion</h2><p class=\"note\">At half speed: the menu opens on the cockpit FOV row, \
             right is held for a second, focus moves down a row, and RB changes the tab. The glass forms by \
             thickness, the platter stretches toward the next row and settles, and nothing flashes.</p>\
             <div class=\"reel osd\">",
        );
        for (i, frame) in reel.iter().enumerate() {
            let _ = write!(
                h,
                "<img style=\"--i:{i}\" src=\"{src}\" alt=\"{alt}\">",
                src = from_page(frame),
                alt = if i == 0 { "the OSD's motion sequence" } else { "" },
            );
        }
        h.push_str("</div></section>");
    }
    let drives: Vec<&OsdRender> = renders.iter().filter(|r| r.drive.is_some()).collect();
    if !drives.is_empty() {
        h.push_str(
            "<section><h2>Driving position</h2><p class=\"note\">On the camera rows the focus echo lifts and \
             the driving position comes up under it: Blender renders of a carbon bucket seat, the wheel and the \
             dash, at your seat height and wheel offset, with your eye, the default eye and the cockpit FOV&#8217;s \
             fan. The part the focused row moves lights along its edges in the accent. These crops are 1:1.</p>\
             <div class=\"grid3\">",
        );
        for r in drives {
            let _ = write!(
                h,
                "<figure><a href=\"{full}\"><img src=\"{crop}\" alt=\"{cap}\"></a>\
                 <figcaption><b>{cap}</b></figcaption></figure>",
                full = shown(r).0,
                crop = r.drive.as_deref().map(from_page).unwrap_or_default(),
                cap = inline_html(&capitalised(r.caption.split(" · ").nth(1).unwrap_or(r.caption))),
            );
        }
        h.push_str("</div></section>");
    }
    h.push_str("<section><h2>States</h2><div class=\"grid\">");
    for r in renders {
        let (full, thumb) = shown(r);
        let _ = write!(
            h,
            "<figure><a href=\"{full}\"><img src=\"{thumb}\" alt=\"{cap}\"></a>\
             <figcaption><b>{cap}</b><span>{gpu}</span></figcaption></figure>",
            cap = inline_html(r.caption),
            gpu = ms(r.gpu_ms),
        );
    }
    h.push_str(
        "</div></section><section><h2>Review data</h2><details><summary>GPU per state (whole frame, live \
         cluster included)</summary><table><tr><th>State</th><th>GPU median</th><th>GPU min</th><th>Draws</th></tr>",
    );
    for r in renders {
        let ok = r
            .gpu_ms
            .map(|g| if g < BUDGET_MS { "ok" } else { "bad" })
            .unwrap_or_default();
        let _ = write!(
            h,
            "<tr><td>{name}</td><td class=\"{ok}\">{gpu}</td><td>{min}</td><td>{draws}</td></tr>",
            name = r.name,
            gpu = ms(r.gpu_ms),
            min = ms(r.gpu_min_ms),
            draws = r.draws,
        );
    }
    let _ = writeln!(
        h,
        "</table></details><p class=\"note\" style=\"margin-top:18px\">Spec: <code>docs/design/OSD.md</code>. \
         Rendered on {adapter} at 3840 × 2160; GPU times count only from a real GPU.</p></section></main></body></html>",
        adapter = inline_html(adapter),
    );
    let page = out.join("osd").join("index.html");
    std::fs::write(&page, h).map_err(|e| std::io::Error::new(e.kind(), format!("{}: {e}", page.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_links_files_from_its_own_folder() {
        assert_eq!(from_page("osd/06_fit_fov.png"), "06_fit_fov.png");
        assert_eq!(from_page("_thumbs/osd/06_fit_fov.png"), "../_thumbs/osd/06_fit_fov.png");
    }

    #[test]
    fn a_card_title_starts_with_a_capital() {
        assert_eq!(
            capitalised("cockpit FOV: the echo lifts"),
            "Cockpit FOV: the echo lifts"
        );
        assert_eq!(capitalised("éclat"), "Éclat");
        assert_eq!(capitalised(""), "");
    }

    #[test]
    fn the_sequence_holds_right_for_a_full_second_and_ends_released() {
        assert_eq!(sequence_held(0), 0);
        let held = (0..SEQUENCE_FRAMES).filter(|f| sequence_held(*f) == RIGHT).count();
        assert_eq!(held, 60, "WCAG's count test needs one second of the held state");
        assert_eq!(sequence_held(110), DOWN);
        assert_eq!(sequence_held(146), RB);
        assert_eq!(sequence_held(SEQUENCE_FRAMES - 1), 0);
        assert!(SEQUENCE.windows(2).all(|w| w[0].0 < w[1].0), "steps are in frame order");
    }

    #[test]
    fn the_drive_states_are_the_camera_rows_that_show_it() {
        let drive: Vec<&str> = STATES.iter().filter(|s| s.drive).map(|s| s.name).collect();
        assert_eq!(drive, ["06_fit_fov", "12_fit_seat_height", "13_fit_wheel_offset"]);
        let [x, y, w, h] = DRIVE_CROP;
        assert!(x + w <= W && y + h <= H);
    }
}
