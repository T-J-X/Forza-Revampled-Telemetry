//! 4K kit renders of the OSD: every tab, focus and slider states, the keyboard and the
//! remapper, plus GPU timings for `docs/perf/osd.md`. The OSD is driven through the same
//! `OsdHost` the overlay uses, with the live dyno feeding the live cluster and the carousel.
use crate::{rel, Result, DT, H, THUMB_FACTOR, W};
use fh6_clusters::registry;
use fh6_core::{AppConfig, CarDatabase, ScrimFit};
use fh6_gpu::GpuContext;
use fh6_kit::scrim::Wash;
use fh6_kit::testing::{Offscreen, RgbaImage, OFFSCREEN_FORMAT};
use fh6_kit::{FrameInput, Kit, Layout, Logo, Palette, SignalSmoother, Tokens, View};
use fh6_osd::model::{self, RowId, Tab};
use fh6_osd::nav::{NavInput, LB, RIGHT, VIEW};
use fh6_osd::{draw, ConfigOnlyFit, Frame, LiveDynoGenerator, LiveStatus, OsdHost};
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
    setup: fn(&mut OsdHost, &mut AppConfig) -> Duration,
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
        setup: |h, c| {
            focus(h, c, Tab::Cluster, RowId::Cluster);
            Duration::ZERO
        },
    },
    OsdState {
        name: "02_cluster_hood_view",
        caption: "Cluster tab · editing the Hood view (live view follows)",
        setup: |h, c| {
            focus(h, c, Tab::Cluster, RowId::EditView);
            press(h, c, 0, RIGHT, RIGHT);
            Duration::from_millis(100)
        },
    },
    OsdState {
        name: "03_colour_custom_hue",
        caption: "Colour tab · custom palette, hue slider focused",
        setup: |h, c| {
            c.color_mode = "Custom".into();
            focus(h, c, Tab::Colour, RowId::Hue);
            Duration::ZERO
        },
    },
    OsdState {
        name: "04_colour_random",
        caption: "Colour tab · random palette with re-roll and lock",
        setup: |h, c| {
            c.color_mode = "Random".into();
            c.extra.insert(model::KEY_SEED.into(), 4242.into());
            focus(h, c, Tab::Colour, RowId::Reroll);
            Duration::ZERO
        },
    },
    OsdState {
        name: "05_type_logo",
        caption: "Type & logo tab · font choice focused",
        setup: |h, c| {
            focus(h, c, Tab::Type, RowId::Font);
            Duration::ZERO
        },
    },
    OsdState {
        name: "06_fit_fov",
        caption: "Fit & camera · cockpit FOV: the echo lifts over the rendered driving position",
        setup: |h, c| {
            focus(h, c, Tab::Fit, RowId::FovCockpit);
            Duration::ZERO
        },
    },
    OsdState {
        name: "07_fit_cluster_height",
        caption: "Fit & camera · cluster height slider and default-height line",
        setup: |h, c| {
            c.cockpit_height_offset_px = 64.0;
            focus(h, c, Tab::Fit, RowId::ClusterHeight);
            Duration::ZERO
        },
    },
    OsdState {
        name: "08_fit_dash_blocker",
        caption: "Fit & camera · dash blocker rows (scrolled)",
        setup: |h, c| {
            focus(h, c, Tab::Fit, RowId::Scrim(1));
            Duration::ZERO
        },
    },
    OsdState {
        name: "09_controls",
        caption: "Controls tab · hold time slider",
        setup: |h, c| {
            focus(h, c, Tab::Controls, RowId::HoldTime);
            Duration::ZERO
        },
    },
    OsdState {
        name: "10_keyboard",
        caption: "On-screen keyboard for the custom name",
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
        setup: |h, c| {
            c.driver_seat_height_m = 0.020;
            focus(h, c, Tab::Fit, RowId::SeatHeight);
            Duration::ZERO
        },
    },
    OsdState {
        name: "13_fit_wheel_offset",
        caption: "Fit & camera · wheel offset lowered to -70 mm: the wheel lit from its rim",
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
    let cars = CarDatabase::load(assets.join("car_brands/car_brands.json"));
    let backdrop = backdrop.map(fit_backdrop);
    let mut renders = Vec::new();
    // The first timed state also brings the GPU up from idle (about a second of frames), so every
    // state is timed at the clocks it runs at while the menu is open.
    let mut warm_up = FIRST_WARM_UP;
    for state in STATES.iter().filter(|s| only.is_none_or(|o| s.name.contains(o))) {
        let mut cfg = AppConfig {
            current_camera_index: 3,
            cockpit_hud_style: "LiquidGlass".into(),
            custom_driver_name: "TJX".into(),
            ..AppConfig::default()
        };
        let mut host = OsdHost::new(tokens, Box::new(ConfigOnlyFit));
        host.set_fonts(kit.fonts().families());
        host.open(&cfg, View::Cockpit, cfg.cockpit_anchor_x as f32);
        let now = (state.setup)(&mut host, &mut cfg);
        let view = host.edit_view();
        host.prepare(kit, &ctx.device, &ctx.queue, Some(assets), &cfg);
        let theme = model::theme_for_view(&cfg, view);
        let entry = registry::resolve(theme);
        let mut cluster = {
            let mut setup = kit.setup(&ctx.device, &ctx.queue, Some(assets));
            (entry.make)(&mut setup)
        };
        let palette = Palette::from_config(tokens, &cfg, &cars, 0, entry.family);
        kit.set_palette(palette);
        let logo = Logo {
            image: None,
            brand_name: String::new(),
            custom_name: cfg.custom_driver_name.clone(),
        };
        let fonts = kit.font_choice(None);
        let mut smoother = SignalSmoother::new();
        let mut dyno = LiveDynoGenerator::new();
        dyno.phase = 2.6;
        let base = Layout::from_config(&cfg, tokens, view, (W, H));
        let mut t = 0.0;
        for i in 0..45 {
            let data = dyno.tick(DT);
            let signals = smoother.update(&data, &cfg, tokens, DT);
            let input = FrameInput {
                telemetry: &data,
                signals: &signals,
                dt: DT,
                t,
                config: &cfg,
                palette: &palette,
                tokens,
                fonts,
                logo: &logo,
                view,
                reduced_flash: model::reduced_flash(&cfg),
            };
            cluster.update(&input);
            if i == 44 {
                host.settle(&input, kit, frame, &cfg);
            } else {
                host.update(&input, kit, frame, &cfg);
            }
            t += DT;
        }
        let layout = host.live_layout(&base, frame);
        let name = model::theme_label(theme);
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
                draw::live_backing(&mut p, &layout, host.anim.open.progress(), backing);
                cluster.draw(&mut p, &layout);
                host.draw(&mut p, &cfg, frame, now, name, LiveStatus::Dyno);
            }
            kit.set_underlay(draw::underlay(host.anim.open.progress()));
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
        if let Some(bg) = &backdrop {
            over_image(&img, bg).save_png(&out.join("osd").join(format!("{}_over_game.png", state.name)))?;
        }
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
            gpu_ms,
            gpu_min_ms,
            draws,
        });
    }
    Ok(renders)
}
