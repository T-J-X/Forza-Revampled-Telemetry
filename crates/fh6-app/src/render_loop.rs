//! The render thread: wakes on every packet (or each refresh while something animates), lets
//! the active cluster update and draw with the render kit, and presents. No queue, no lock, no
//! readback on this path.
use crate::latency::{caught_next_vsync, FrameStats, Percentiles, SAMPLE_CAPACITY};
use crate::refraction::{RefractionSource, RETRY};
use crate::resources::KitResources;
use fh6_core::{AppConfig, ClusterTheme, ConfigHandle, TelemetryReader, VisibilityController, VisibilityInputs};
use fh6_gpu::seven_seg::StackStr;
use fh6_gpu::{FrameOutcome, GpuContext, GpuError, OverlaySurface};
use fh6_input::InputShared;
use fh6_kit::{
    vec2, Affine, Align, Cluster, FrameInput, Kit, Layout, Logo, Painter, Palette, SignalSmoother, TextStyle, VAlign,
    View,
};
use fh6_osd::LiveDynoGenerator;
use fh6_win::{ForegroundReader, OverlayHandle};
use std::fmt::Write as _;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle, Thread};
use std::time::{Duration, Instant};

/// Reference height the config anchors are authored in.
const REF_H: f32 = 2160.0;
/// Wait between checks while hidden (packets and focus changes wake the thread sooner).
const IDLE_WAIT: Duration = Duration::from_millis(50);
/// Fallback refresh interval when DWM timing is unavailable.
const FALLBACK_REFRESH: Duration = Duration::from_micros(16_667);
/// Benchmarks discard this much warm-up.
const BENCH_WARMUP: Duration = Duration::from_secs(2);
/// Longest `dt` a cluster sees (after a hide, a stall or a debugger pause).
const MAX_DT: f32 = 0.1;

/// Cross-thread controls for the render thread.
#[derive(Debug, Default)]
pub struct RenderControl {
    stop: AtomicBool,
    user_hidden: AtomicBool,
    pending_size: AtomicU64,
}

impl RenderControl {
    /// A running, visible control block.
    pub fn new() -> Self {
        Self::default()
    }

    /// Asks the render thread to finish.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    fn stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    /// Flips the user's show/hide choice and returns the new "hidden" state.
    pub fn toggle_user_hidden(&self) -> bool {
        !self.user_hidden.fetch_xor(true, Ordering::Relaxed)
    }

    /// Whether the user hid the overlay.
    pub fn user_hidden(&self) -> bool {
        self.user_hidden.load(Ordering::Relaxed)
    }

    /// Asks for a swapchain resize on the next frame.
    pub fn request_resize(&self, width: u32, height: u32) {
        let packed = (u64::from(width.max(1)) << 32) | u64::from(height.max(1));
        self.pending_size.store(packed, Ordering::Relaxed);
    }

    fn take_resize(&self) -> Option<(u32, u32)> {
        match self.pending_size.swap(0, Ordering::Relaxed) {
            0 => None,
            packed => Some(((packed >> 32) as u32, (packed & 0xFFFF_FFFF) as u32)),
        }
    }
}

/// Behaviour switches from the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RenderOptions {
    /// Ignore the visibility rules; synthesise telemetry when none arrives.
    pub demo: bool,
    /// Draw the stats lines.
    pub stats: bool,
    /// Run this long, then stop and report.
    pub bench: Option<Duration>,
}

/// Everything the render thread owns or reads.
pub struct RenderSetup {
    /// The GPU device.
    pub gpu: GpuContext,
    /// The overlay swapchain.
    pub surface: OverlaySurface,
    /// Keeps the window alive and lets a lost surface be recreated.
    pub window: OverlayHandle,
    /// Latest telemetry.
    pub telemetry: TelemetryReader,
    /// Live config.
    pub config: ConfigHandle,
    /// Controller state (camera ring, OSD).
    pub input: Arc<InputShared>,
    /// Whether the game has the foreground.
    pub foreground: ForegroundReader,
    /// Stop, hide, resize requests.
    pub control: Arc<RenderControl>,
    /// Command-line switches.
    pub options: RenderOptions,
    /// Tokens, brand palettes, asset folders.
    pub resources: KitResources,
    /// The game executable (for the optional glass refraction capture).
    pub game_process: String,
    /// Run once when a benchmark finishes (asks the app to quit).
    pub on_bench_done: Box<dyn FnOnce() + Send>,
}

/// What the render thread measured, returned when it ends.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderSummary {
    /// Adapter description.
    pub adapter: String,
    /// Present mode used.
    pub present_mode: String,
    /// Swapchain size at the end.
    pub size: (u32, u32),
    /// Measured time span (after warm-up for benchmarks).
    pub measured: Duration,
    /// Frames presented in that span.
    pub frames: u64,
    /// New packets shown in that span.
    pub packets: u64,
    /// Packet receive to `present` return.
    pub receive_to_present: Percentiles,
    /// Packet receive to the first vblank after `present`.
    pub receive_to_vblank: Percentiles,
    /// CPU time per presented frame.
    pub cpu_frame: Percentiles,
    /// Blocked on the swapchain latency wait.
    pub acquire_wait: Percentiles,
    /// Cluster update and draw recording.
    pub build: Percentiles,
    /// Kit render (pack, upload, record) and submit.
    pub encode_submit: Percentiles,
    /// The `present` call.
    pub present: Percentiles,
    /// New packets shown at the first vblank after arrival.
    pub vsync_hits: u64,
    /// New packets checked for that.
    pub vsync_checked: u64,
    /// Display refresh interval reported by DWM.
    pub refresh: Duration,
}

/// Starts the render thread. Returns its join handle and its `Thread` (for unparking).
///
/// # Errors
/// Returns the I/O error if the thread cannot be spawned.
pub fn spawn(setup: RenderSetup) -> io::Result<(JoinHandle<RenderSummary>, Thread)> {
    let handle = thread::Builder::new()
        .name("fh6-render".into())
        .spawn(move || run(setup))?;
    let thread = handle.thread().clone();
    Ok((handle, thread))
}

/// The theme `cfg` selects for `view` (hood uses `BonnetHudStyle`, the rest
/// `CockpitHudStyle`, then `HudStyle`).
pub fn theme_for(cfg: &AppConfig, view: View) -> ClusterTheme {
    let style = match view {
        View::Hood => cfg.bonnet_hud_style.as_str(),
        View::Cockpit | View::Chase => {
            if !cfg.cockpit_hud_style.trim().is_empty() {
                cfg.cockpit_hud_style.as_str()
            } else if !cfg.hud_style.trim().is_empty() {
                cfg.hud_style.as_str()
            } else {
                "LiquidGlass"
            }
        }
    };
    fh6_render::dispatch::style_to_theme(style)
}

/// Whether to draw the live dyno instead of the received packet. The demo, the bench and the
/// OSD live view all need a moving dash when the game is not actually driving: when the
/// stream is stale, and also when the game keeps sending `IsRaceOn == 0` packets while
/// paused (those carry frozen or zeroed values).
pub fn uses_dyno(bypass: bool, live_preview: bool, fresh: bool, race_on: bool) -> bool {
    (bypass || live_preview) && !(fresh && race_on)
}

/// One `--stats` line.
#[derive(Debug, Clone, Copy, PartialEq)]
struct StatLine {
    label: &'static str,
    value: f32,
    decimals: usize,
}

/// The device, swapchain and kit. `None` while they are being recreated after a loss.
struct Gpu {
    ctx: GpuContext,
    surface: OverlaySurface,
    kit: Option<Kit>,
}

/// The cluster on screen and what it was built for.
struct Active {
    theme: ClusterTheme,
    view: View,
    family: &'static str,
    cluster: Box<dyn Cluster>,
    born: Instant,
}

struct Loop {
    gpu: Option<Gpu>,
    resources: KitResources,
    /// Last known adapter, present mode and size, for the summary.
    adapter: String,
    present_mode: String,
    size: (u32, u32),
    stats: FrameStats,
    scratch: Vec<f32>,
    stat_lines: [StatLine; 4],
    frames: u64,
    packets: u64,
    fps_window: (Instant, u64),
    lost_since: Option<Instant>,
    active: Option<Active>,
    smoother: SignalSmoother,
    /// Theme cache: the config it came from (pointer identity) and the view.
    theme_cache: Option<(*const AppConfig, View, ClusterTheme)>,
    /// Palette cache: config identity, car ordinal and family.
    palette_cache: Option<(*const AppConfig, i32, &'static str)>,
    palette: Palette,
    logo: Logo,
    last_update: Instant,
    refraction: Option<RefractionSource>,
    refraction_retry: Instant,
}

fn new_kit(ctx: &GpuContext, resources: &KitResources) -> Option<Kit> {
    match Kit::new(&ctx.device, &ctx.queue, resources.tokens.clone()) {
        Ok(mut kit) => {
            for dir in resources
                .assets
                .iter()
                .map(|a| a.join("fonts"))
                .chain(resources.user_fonts.clone())
            {
                if dir.is_dir() {
                    match kit.fonts_mut().load_dir(&dir) {
                        Ok(faces) => tracing::info!(dir = %dir.display(), faces = faces.len(), "fonts loaded"),
                        Err(err) => tracing::warn!(error = %err, "fonts not loaded"),
                    }
                }
            }
            let bound = kit.bind_font_roles();
            tracing::info!(bound, "font roles bound to the tokens' families");
            Some(kit)
        }
        Err(err) => {
            tracing::error!(error = %err, "render kit could not start");
            None
        }
    }
}

impl Loop {
    fn adopt(&mut self, ctx: GpuContext, surface: OverlaySurface) {
        self.adapter = ctx.adapter_summary().to_string();
        self.present_mode = format!("{:?}", surface.present_mode());
        self.size = surface.size();
        let kit = new_kit(&ctx, &self.resources);
        // Clusters and the refraction texture belong to the old device: rebuild them.
        self.active = None;
        self.refraction = None;
        self.gpu = Some(Gpu { ctx, surface, kit });
        self.lost_since = None;
    }

    /// Drops the device and swapchain so the window has no composition target left (it then
    /// shows nothing), and schedules a recreation.
    fn lose_gpu(&mut self, why: &str) {
        tracing::warn!(why, "dropping the GPU device; it is recreated in 1 s");
        self.active = None;
        self.refraction = None;
        self.gpu = None;
        self.lost_since = Some(Instant::now());
    }

    /// Recreates the device after a loss, at most once a second.
    fn try_recreate(&mut self, window: &OverlayHandle) {
        let Some(since) = self.lost_since else { return };
        if since.elapsed() < Duration::from_secs(1) {
            return;
        }
        match OverlaySurface::create(window.clone(), self.size.0, self.size.1) {
            Ok((ctx, surface)) => {
                tracing::info!("GPU device and overlay surface recreated");
                self.adopt(ctx, surface);
            }
            Err(err) => {
                tracing::warn!(error = %err, "recreating the GPU device failed; retrying");
                self.lost_since = Some(Instant::now());
            }
        }
    }

    /// Makes sure the active cluster matches the config and view.
    fn ensure_cluster(&mut self, cfg: &Arc<AppConfig>, view: View) {
        let ptr = Arc::as_ptr(cfg);
        let theme = match self.theme_cache {
            Some((p, v, t)) if p == ptr && v == view => t,
            _ => {
                let t = theme_for(cfg, view);
                self.theme_cache = Some((ptr, view, t));
                t
            }
        };
        if self.active.as_ref().is_some_and(|a| a.theme == theme && a.view == view) {
            return;
        }
        let Some(gpu) = self.gpu.as_mut() else { return };
        let Some(kit) = gpu.kit.as_mut() else { return };
        let entry = fh6_clusters::registry::resolve(theme);
        let assets = self.resources.assets.clone();
        let cluster = {
            let mut setup = kit.setup(&gpu.ctx.device, &gpu.ctx.queue, assets.as_deref());
            (entry.make)(&mut setup)
        };
        tracing::info!(%theme, cluster = entry.name, ?view, "cluster selected");
        self.active = Some(Active {
            theme,
            view,
            family: entry.family,
            cluster,
            born: Instant::now(),
        });
        self.palette_cache = None;
    }

    /// Refreshes the palette and logo when the config, car or family changed.
    fn ensure_palette(&mut self, cfg: &Arc<AppConfig>, car_ordinal: i32) {
        let Some(family) = self.active.as_ref().map(|a| a.family) else {
            return;
        };
        let key = (Arc::as_ptr(cfg), car_ordinal, family);
        if self.palette_cache == Some(key) {
            return;
        }
        let tokens = &self.resources.tokens;
        self.palette = Palette::from_config(tokens, cfg, &self.resources.cars, car_ordinal, family);
        let brand = self
            .resources
            .cars
            .get_brand_palette_for_ordinal(&car_ordinal.to_string())
            .map(|b| b.display_name.clone())
            .unwrap_or_default();
        self.logo = Logo {
            image: None,
            brand_name: brand,
            custom_name: cfg.custom_driver_name.clone(),
        };
        if let Some(kit) = self.gpu.as_mut().and_then(|g| g.kit.as_mut()) {
            kit.set_palette(self.palette);
        }
        self.palette_cache = Some(key);
    }

    /// Starts, polls or stops the optional refraction capture (`"GlassRefraction": true`).
    fn update_refraction(&mut self, cfg: &AppConfig, layout: &Layout, game_process: &str) {
        let on = cfg
            .extra
            .get("GlassRefraction")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let Some(gpu) = self.gpu.as_mut() else { return };
        let Some(kit) = gpu.kit.as_mut() else { return };
        if !on {
            if self.refraction.take().is_some() {
                kit.set_refraction_source(None);
                tracing::info!("glass refraction capture stopped");
            }
            return;
        }
        if self.refraction.is_none() {
            let now = Instant::now();
            if now < self.refraction_retry {
                return;
            }
            self.refraction_retry = now + RETRY;
            let (w, h) = (self.size.0 as f32, self.size.1 as f32);
            let rect = layout
                .transform()
                .apply_rect(&layout.envelope.expand(48.0))
                .intersect(&fh6_kit::Rect::from_xywh(0.0, 0.0, w, h));
            self.refraction = RefractionSource::start(game_process, rect);
        }
        if let Some(src) = self.refraction.as_mut() {
            match src.poll(&gpu.ctx.device, &gpu.ctx.queue) {
                Ok(Some(view)) => kit.set_refraction_source(Some((view, src.rect()))),
                Ok(None) => {}
                Err(err) => {
                    tracing::warn!(error = %err, "glass refraction capture stopped");
                    self.refraction = None;
                    kit.set_refraction_source(None);
                }
            }
        }
    }

    /// Records and presents one frame at `opacity`. With `draw` false, presents a fully
    /// transparent frame (hide). `None` when nothing was presented.
    fn render(&mut self, opacity: f32, layout: &Layout, draw: bool, stats_on: bool) -> Option<FrameOutcome> {
        let gpu = self.gpu.as_mut()?;
        let Gpu { ctx, surface, kit } = gpu;
        let Some(kit) = kit.as_mut() else {
            // No kit: present an empty frame so nothing stale stays on screen.
            let result = surface.render_with(ctx, |_, enc, view, _, _| {
                drop(enc.begin_render_pass(&fh6_kit::wgpu::RenderPassDescriptor {
                    label: Some("fh6 clear"),
                    color_attachments: &[Some(fh6_kit::wgpu::RenderPassColorAttachment {
                        view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: fh6_kit::wgpu::Operations {
                            load: fh6_kit::wgpu::LoadOp::Clear(fh6_kit::wgpu::Color::TRANSPARENT),
                            store: fh6_kit::wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                }));
            });
            return result.ok();
        };
        {
            let mut p = kit.begin_frame(layout);
            if draw {
                if let Some(active) = self.active.as_ref() {
                    active.cluster.draw(&mut p, layout);
                }
                if stats_on {
                    draw_stats(&mut p, &self.stat_lines, self.size);
                }
            }
        }
        let result = surface.render_with(ctx, |ctx, encoder, view, format, size| {
            kit.render(&ctx.device, &ctx.queue, encoder, view, format, size, opacity);
        });
        let failed = ctx.has_failed();
        match result {
            Ok(outcome) if !failed => Some(outcome),
            Ok(_) => {
                self.lose_gpu("wgpu reported an error");
                None
            }
            Err(GpuError::SurfaceLost) => {
                self.lose_gpu("surface lost");
                None
            }
            Err(err) => {
                tracing::warn!(error = %err, "frame not presented");
                if failed {
                    self.lose_gpu("wgpu reported an error");
                }
                None
            }
        }
    }
}

/// The `--stats` lines in the top-left corner, in target pixels.
fn draw_stats(p: &mut Painter<'_>, lines: &[StatLine; 4], size: (u32, u32)) {
    let k = size.1 as f32 / REF_H;
    let cap = 26.0 * k;
    let font = p.fonts().role(fh6_kit::FontRole::Display);
    let ink = p.palette().roles.ink;
    let style = TextStyle::new(font, cap)
        .paint(ink)
        .align(Align::Left, VAlign::CapTop)
        .tabular();
    p.save();
    p.set_transform(Affine::IDENTITY);
    for (i, line) in lines.iter().enumerate() {
        let mut text = StackStr::<32>::new();
        let _ = write!(text, "{} {:.*}", line.label, line.decimals, line.value);
        p.text(vec2(64.0 * k, 64.0 * k + i as f32 * cap * 1.8), text.as_str(), &style);
    }
    p.restore();
}

fn run(setup: RenderSetup) -> RenderSummary {
    let RenderSetup {
        gpu,
        surface,
        window,
        telemetry,
        config,
        input,
        foreground,
        control,
        options,
        resources,
        game_process,
        on_bench_done,
    } = setup;
    let refresh = fh6_win::system::composition_timing()
        .map(|t| t.refresh_period)
        .unwrap_or(FALLBACK_REFRESH);
    let measure_vblank = options.stats || options.bench.is_some();
    let palette = Palette::family(&resources.tokens, "modernDigital");
    let mut lp = Loop {
        gpu: None,
        resources,
        adapter: String::new(),
        present_mode: String::new(),
        size: (1, 1),
        stats: FrameStats::default(),
        scratch: Vec::with_capacity(SAMPLE_CAPACITY),
        stat_lines: [
            StatLine {
                label: "P50",
                value: 0.0,
                decimals: 2,
            },
            StatLine {
                label: "P99",
                value: 0.0,
                decimals: 2,
            },
            StatLine {
                label: "CPU",
                value: 0.0,
                decimals: 3,
            },
            StatLine {
                label: "FPS",
                value: 0.0,
                decimals: 0,
            },
        ],
        frames: 0,
        packets: 0,
        fps_window: (Instant::now(), 0),
        lost_since: None,
        active: None,
        smoother: SignalSmoother::new(),
        theme_cache: None,
        palette_cache: None,
        palette,
        logo: Logo::default(),
        last_update: Instant::now(),
        refraction: None,
        refraction_retry: Instant::now(),
    };
    lp.adopt(gpu, surface);
    let mut visibility = VisibilityController::default();
    let mut dyno = LiveDynoGenerator::new();
    let mut last_seq = 0u64;
    let mut last_tick = Instant::now();
    let mut blank_presented = false;
    let mut keep_ticking = false;
    let start = Instant::now();
    let mut measured_from = start;
    let mut warmed_up = options.bench.is_none();
    let mut on_bench_done = Some(on_bench_done);

    tracing::info!(?refresh, "render loop started");
    while !control.stopped() {
        if let Some(limit) = options.bench {
            if !warmed_up && start.elapsed() >= BENCH_WARMUP {
                warmed_up = true;
                measured_from = Instant::now();
                lp.stats.clear();
                lp.frames = 0;
                lp.packets = 0;
            }
            if start.elapsed() >= limit + BENCH_WARMUP {
                if let Some(done) = on_bench_done.take() {
                    done();
                }
                break;
            }
        }

        let waiting_for_gpu = lp.lost_since.is_some();
        thread::park_timeout(if keep_ticking && !waiting_for_gpu {
            refresh
        } else {
            IDLE_WAIT
        });
        let wake = Instant::now();
        let dt = wake - last_tick;
        last_tick = wake;

        if let Some((w, h)) = control.take_resize() {
            lp.size = (w.max(1), h.max(1));
            if let Some(gpu) = lp.gpu.as_mut() {
                gpu.surface.resize(&gpu.ctx, w, h);
            }
        }
        // A loss can happen while the dash is hidden and nothing renders: notice it here (one
        // atomic load) so the device is rebuilt before the dash is needed again.
        if lp.gpu.as_ref().is_some_and(|gpu| gpu.ctx.has_failed()) {
            lp.lose_gpu("wgpu reported an error");
        }
        lp.try_recreate(&window);
        if lp.gpu.is_none() {
            // Nothing is on screen while the device is gone: the window has no content.
            blank_presented = true;
            keep_ticking = false;
            continue;
        }

        let cfg = config.current();
        let frame = telemetry.latest();
        let fresh = !frame.is_stale(wake);
        let bypass = options.demo || options.bench.is_some();
        let live_preview = input.osd_open();
        let race_on = frame.data.is_race_on == 1;
        let synthetic = uses_dyno(bypass, live_preview, fresh, race_on);
        let synth_data;
        let data = if synthetic {
            synth_data = dyno.tick(dt.as_secs_f32());
            &synth_data
        } else {
            &frame.data
        };
        let vis = visibility.update(
            VisibilityInputs {
                race_on,
                stream_fresh: fresh,
                game_foreground: foreground.is_game_foreground(),
                user_hidden: control.user_hidden(),
                live_preview,
                bypass,
            },
            dt,
        );
        // Only a packet that is actually drawn counts toward the latency stats.
        let new_packet = !synthetic && fresh && frame.seq != last_seq;
        last_seq = frame.seq;
        let view = View::from_camera_index(i32::try_from(input.camera_index()).unwrap_or(3));
        let layout = Layout::from_config(&cfg, &lp.resources.tokens, view, lp.size);

        if vis.opacity <= 0.0 {
            if !blank_presented {
                // Hide = one fully transparent frame, then idle.
                blank_presented = matches!(
                    lp.render(0.0, &layout, false, false),
                    Some(FrameOutcome::Presented { .. })
                );
                tracing::debug!(reason = ?vis.hidden_because, blank_presented, "overlay hidden");
            }
            // A skipped blank frame (timeout, occluded, reconfigured) is retried on the next
            // refresh, so the last visible frame never lingers; once it is on screen, idle.
            keep_ticking = !blank_presented;
            continue;
        }
        blank_presented = false;
        // While visible, wake every refresh even without packets so a stale stream is hidden
        // within one frame of the 250 ms trigger (packets and focus changes also unpark).
        keep_ticking = true;
        let cluster_moving = lp.active.as_ref().is_some_and(|a| a.cluster.animating());
        if !(new_packet || vis.animating || synthetic || cluster_moving || lp.active.is_none()) {
            continue;
        }

        let build_start = Instant::now();
        lp.ensure_cluster(&cfg, view);
        lp.ensure_palette(&cfg, data.car_ordinal);
        lp.update_refraction(&cfg, &layout, &game_process);
        let update_dt = (build_start - lp.last_update).as_secs_f32().min(MAX_DT);
        lp.last_update = build_start;
        let signals = lp.smoother.update(data, &cfg, &lp.resources.tokens, update_dt);
        let fonts = lp
            .gpu
            .as_ref()
            .and_then(|g| g.kit.as_ref())
            .map(|k| k.font_choice(None));
        if let (Some(active), Some(fonts)) = (lp.active.as_mut(), fonts) {
            let reduced_flash = cfg
                .extra
                .get("ReducedFlash")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            active.cluster.update(&FrameInput {
                telemetry: data,
                signals: &signals,
                dt: update_dt,
                t: (build_start - active.born).as_secs_f32(),
                config: &cfg,
                palette: &lp.palette,
                tokens: &lp.resources.tokens,
                fonts,
                logo: &lp.logo,
                view,
                reduced_flash,
            });
        }
        let built = build_start.elapsed();

        if let Some(FrameOutcome::Presented { timings, at }) = lp.render(vis.opacity, &layout, true, options.stats) {
            lp.frames += 1;
            lp.stats.cpu_frame.record(built + timings.cpu());
            lp.stats.acquire_wait.record(timings.acquire_wait);
            lp.stats.build.record(built);
            lp.stats.encode_submit.record(timings.encode_submit);
            lp.stats.present.record(timings.present);
            if new_packet {
                lp.packets += 1;
                if let Some(received) = frame.received_at {
                    let to_present = at.saturating_duration_since(received);
                    lp.stats.receive_to_present.record(to_present);
                    if measure_vblank {
                        if let Some(timing) = fh6_win::system::composition_timing() {
                            lp.stats.receive_to_vblank.record(to_present + timing.until_next_vblank);
                            lp.stats.vsync_checked += 1;
                            if caught_next_vsync(to_present, timing.until_next_vblank, timing.refresh_period) {
                                lp.stats.vsync_hits += 1;
                            }
                        }
                    }
                }
            }
            if options.stats && lp.frames.is_multiple_of(30) {
                refresh_stat_lines(&mut lp);
            }
        }
    }

    let measured = measured_from.elapsed();
    let summary = RenderSummary {
        adapter: lp.adapter.clone(),
        present_mode: lp.present_mode.clone(),
        size: lp.size,
        measured,
        frames: lp.frames,
        packets: lp.packets,
        receive_to_present: lp.stats.receive_to_present.percentiles(&mut lp.scratch),
        receive_to_vblank: lp.stats.receive_to_vblank.percentiles(&mut lp.scratch),
        cpu_frame: lp.stats.cpu_frame.percentiles(&mut lp.scratch),
        acquire_wait: lp.stats.acquire_wait.percentiles(&mut lp.scratch),
        build: lp.stats.build.percentiles(&mut lp.scratch),
        encode_submit: lp.stats.encode_submit.percentiles(&mut lp.scratch),
        present: lp.stats.present.percentiles(&mut lp.scratch),
        vsync_hits: lp.stats.vsync_hits,
        vsync_checked: lp.stats.vsync_checked,
        refresh,
    };
    tracing::info!(
        frames = summary.frames,
        packets = summary.packets,
        "render loop stopped"
    );
    summary
}

fn refresh_stat_lines(lp: &mut Loop) {
    let r2p = lp.stats.receive_to_present.percentiles(&mut lp.scratch);
    let cpu = lp.stats.cpu_frame.percentiles(&mut lp.scratch);
    lp.stat_lines[0].value = r2p.p50;
    lp.stat_lines[1].value = r2p.p99;
    lp.stat_lines[2].value = cpu.p99;
    let (since, frames_then) = lp.fps_window;
    let elapsed = since.elapsed().as_secs_f32();
    if elapsed >= 1.0 {
        lp.stat_lines[3].value = (lp.frames - frames_then) as f32 / elapsed;
        lp.fps_window = (Instant::now(), lp.frames);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_follows_the_view() {
        let cfg = AppConfig {
            cockpit_hud_style: "LiquidGlass".into(),
            bonnet_hud_style: "Gt7HoodPro".into(),
            ..AppConfig::default()
        };
        assert_eq!(theme_for(&cfg, View::Cockpit), ClusterTheme::LiquidGlass);
        assert_eq!(theme_for(&cfg, View::Hood), ClusterTheme::Gt7HoodPro);
        let empty = AppConfig {
            cockpit_hud_style: String::new(),
            hud_style: String::new(),
            ..AppConfig::default()
        };
        assert_eq!(theme_for(&empty, View::Cockpit), ClusterTheme::LiquidGlass);
    }

    #[test]
    fn layouts_scale_from_4k_and_follow_the_camera() {
        let cfg = AppConfig::default();
        let tokens = fh6_kit::Tokens::default();
        let cockpit = Layout::from_config(&cfg, &tokens, View::from_camera_index(3), (3840, 2160));
        assert_eq!(cockpit.anchor, vec2(1920.0, 1835.0));
        let hood = Layout::from_config(&cfg, &tokens, View::from_camera_index(4), (1920, 1080));
        assert_eq!(hood.anchor, vec2(960.0, 880.0), "bonnet anchor at half resolution");
        assert!((hood.scale - cockpit.scale * 0.5).abs() < 1e-6);
    }

    #[test]
    fn dyno_replaces_paused_and_stale_packets_only_for_preview_and_demo() {
        assert!(!uses_dyno(true, true, true, true));
        assert!(uses_dyno(false, true, true, false));
        assert!(uses_dyno(false, true, false, false));
        assert!(uses_dyno(true, false, false, true));
        assert!(!uses_dyno(false, false, true, false));
        assert!(!uses_dyno(false, false, false, true));
    }

    #[test]
    fn resize_requests_are_taken_once() {
        let control = RenderControl::new();
        assert_eq!(control.take_resize(), None);
        control.request_resize(3840, 2160);
        assert_eq!(control.take_resize(), Some((3840, 2160)));
        assert_eq!(control.take_resize(), None);
        assert!(control.toggle_user_hidden());
        assert!(!control.toggle_user_hidden());
    }
}
