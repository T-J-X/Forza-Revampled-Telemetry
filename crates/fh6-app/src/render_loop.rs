//! The render thread: wakes on every packet (or each refresh while something animates), lets
//! the active cluster update and draw with the render kit, and presents. No queue, no lock, no
//! readback on this path.
use crate::latency::{caught_next_vsync, FrameStats, Percentiles, SAMPLE_CAPACITY};
use crate::refraction::{RefractionSource, RETRY};
use crate::resources::KitResources;
use fh6_core::{
    AppConfig, CarProfilesFile, ClusterTheme, ConfigHandle, ProfilesHandle, ScrimFit, TelemetryReader,
    VisibilityController, VisibilityInputs,
};
use fh6_gpu::seven_seg::StackStr;
use fh6_gpu::{FrameOutcome, GpuContext, GpuError, OverlaySurface};
use fh6_input::InputShared;
use fh6_kit::scrim::{self, Wash};
use fh6_kit::{
    vec2, Affine, Align, Cluster, FontId, FrameInput, Kit, Layout, Logo, Painter, Palette, SignalSmoother, TextStyle,
    VAlign, View,
};
use fh6_osd::nav::NavInput;
use fh6_osd::{
    model, AutoPause, CockpitFit, ConfigSaver, Frame, KeyboardInjector, LiveDynoGenerator, LiveStatus, OsdHost,
    PauseInputs,
};
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
/// After the OSD closes, keep drawing its working config until the saved file has been
/// reloaded (so nothing flickers back), for at most this long.
const FOLLOW_SAVE: Duration = Duration::from_secs(2);

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
    /// Open the OSD at start (`--open-osd`).
    pub open_osd: bool,
}

/// What the OSD needs from the app.
pub struct OsdSetup {
    /// Saves OSD edits to `config.json` (none: edits stay in memory).
    pub saver: Option<ConfigSaver>,
    /// Sends the auto-pause ESC.
    pub injector: Box<dyn KeyboardInjector>,
    /// Camera files and scrim rows (Phase 8 backend, or config-only).
    pub fit: Box<dyn CockpitFit>,
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
    /// Live per-car profiles (`car_profiles.json`), applied over the config by car ordinal.
    pub profiles: ProfilesHandle,
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
    /// The OSD's saver, key injector and cockpit-fit backend.
    pub osd: OsdSetup,
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

/// The config with the current car's profile applied (`car_profiles.json`, by car ordinal).
/// It is rebuilt only when the base config, the profiles or the car change: one allocation per
/// change, none per frame. Without a profile (or with `PerCarProfileEnabled` off) it is the base
/// config itself.
#[derive(Debug, Default)]
pub struct CarConfig {
    base: Option<Arc<AppConfig>>,
    profiles: Option<Arc<CarProfilesFile>>,
    ordinal: i32,
    current: Option<Arc<AppConfig>>,
}

impl CarConfig {
    /// The effective config for `ordinal`.
    pub fn resolve(&mut self, base: &Arc<AppConfig>, profiles: &Arc<CarProfilesFile>, ordinal: i32) -> Arc<AppConfig> {
        let same = self.base.as_ref().is_some_and(|b| Arc::ptr_eq(b, base))
            && self.profiles.as_ref().is_some_and(|p| Arc::ptr_eq(p, profiles))
            && self.ordinal == ordinal;
        if let (true, Some(current)) = (same, self.current.as_ref()) {
            return Arc::clone(current);
        }
        let effective = match profiles.effective_config(base, ordinal) {
            Some(cfg) => {
                tracing::info!(car_ordinal = ordinal, "car profile applied");
                Arc::new(cfg)
            }
            None => Arc::clone(base),
        };
        self.base = Some(Arc::clone(base));
        self.profiles = Some(Arc::clone(profiles));
        self.ordinal = ordinal;
        self.current = Some(Arc::clone(&effective));
        effective
    }
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
    /// `ClusterFont` resolved against the loaded faces.
    font_override: Option<FontId>,
}

/// The OSD as the render thread runs it: the host, the working config being edited, the
/// auto-pause state and the saver.
struct Osd {
    host: OsdHost,
    /// The working copy edits land in.
    edit: AppConfig,
    /// The working config the frame draws with (a new `Arc` per edit, so caches refresh).
    working: Option<Arc<AppConfig>>,
    was_open: bool,
    /// After closing: draw `working` until the saved file is reloaded or this passes.
    follow_until: Option<Instant>,
    pause: AutoPause,
    injector: Box<dyn KeyboardInjector>,
    saver: Option<ConfigSaver>,
    epoch: Instant,
    fonts_listed: bool,
}

impl Osd {
    /// The open edge: snapshot the config, discard the opening press, pause mid-race.
    fn opened(&mut self, cfg: &Arc<AppConfig>, view: View, input: &InputShared, pause: PauseInputs) {
        let _ = input.take_edges();
        self.edit = (**cfg).clone();
        self.working = Some(Arc::clone(cfg));
        self.follow_until = None;
        let anchor_x = match view {
            View::Hood => cfg.bonnet_anchor_x,
            View::Cockpit | View::Chase => cfg.cockpit_anchor_x,
        } as f32;
        self.host.open(cfg, view, anchor_x);
        if self.pause.on_open(&mut *self.injector, pause) {
            tracing::info!("OSD opened mid-race: ESC sent to pause the game");
        }
        tracing::info!(?view, "OSD opened");
    }

    /// The close edge: flush the save, schedule the resume, lift the settings focus.
    fn closed(&mut self, now: Instant, input: &InputShared) {
        input.set_settings_focused(false);
        if let Some(saver) = &self.saver {
            saver.flush();
        }
        self.pause.on_close(now - self.epoch);
        self.follow_until = Some(now + FOLLOW_SAVE);
        tracing::info!("OSD closed");
    }

    /// The config to draw this frame: the working copy while open (and briefly after), else
    /// the live one.
    fn config_for_frame(&mut self, live: &Arc<AppConfig>, open: bool, now: Instant) -> Arc<AppConfig> {
        if open {
            if let Some(w) = &self.working {
                return Arc::clone(w);
            }
        }
        if let (Some(until), Some(w)) = (self.follow_until, &self.working) {
            if now < until && **w != **live {
                return Arc::clone(w);
            }
            self.follow_until = None;
            self.working = None;
        }
        Arc::clone(live)
    }

    /// One frame of controller input while open. Returns true when the OSD asked to close.
    fn navigate(&mut self, input: &InputShared, now: Instant) -> bool {
        let (pressed, released) = input.take_edges();
        let nav = NavInput {
            held: input.buttons(),
            pressed,
            released,
            now: now - self.epoch,
            toggle_mask: self.edit.osd_toggle_mask,
            hold_ms: self.edit.osd_hold_ms,
        };
        let out = self.host.step(&nav, &mut self.edit);
        input.set_settings_focused(out.settings_focus);
        if out.changed {
            let snapshot = Arc::new(self.edit.clone());
            if let Some(saver) = &self.saver {
                saver.save(Arc::clone(&snapshot));
            }
            self.working = Some(snapshot);
        }
        out.close
    }
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
                let t = fh6_osd::theme_for_view(cfg, view);
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
        // `ShowLogo: false` empties the logo zone (no image, wordmark or name).
        self.logo = if model::show_logo(cfg) {
            Logo {
                image: None,
                brand_name: brand,
                custom_name: cfg.custom_driver_name.clone(),
            }
        } else {
            Logo::default()
        };
        if let Some(kit) = self.gpu.as_mut().and_then(|g| g.kit.as_mut()) {
            kit.set_palette(self.palette);
            self.font_override = model::cluster_font(cfg).and_then(|family| kit.fonts().find(family, 600, false));
            // A face from the pack or the user folder is rasterised on first use: do it now, so
            // the first frame in the new font has every digit and letter.
            if let Some(font) = self.font_override {
                kit.fonts_mut().ensure_preloaded(font);
            }
        }
        self.palette_cache = Some(key);
    }

    /// Creates the OSD carousel's thumbnail clusters when the selection moved.
    fn prepare_osd(&mut self, host: &mut OsdHost, cfg: &AppConfig) {
        let Some(gpu) = self.gpu.as_mut() else { return };
        let Some(kit) = gpu.kit.as_mut() else { return };
        let assets = self.resources.assets.clone();
        host.prepare(kit, &gpu.ctx.device, &gpu.ctx.queue, assets.as_deref(), cfg);
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
    fn render(
        &mut self,
        opacity: f32,
        layout: &Layout,
        draw: bool,
        stats_on: bool,
        scrim: Option<(ScrimFit, Wash)>,
        osd: Option<&OsdFrame<'_>>,
    ) -> Option<FrameOutcome> {
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
        // The OSD dims the game through the kit's underlay (free in the resolve); a hide frame
        // and the normal dash have none.
        kit.set_underlay(match osd {
            Some(o) if draw => fh6_osd::draw::underlay(o.host.anim.open.progress()),
            _ => 0.0,
        });
        {
            let mut p = kit.begin_frame(layout);
            if draw {
                if let Some(o) = osd {
                    // The cluster glides to the OSD's live zone. Under it: the real dash blocker
                    // while a blocker row has focus (it follows the cluster), else a soft pool.
                    let blocker = if o.blocker {
                        scrim.as_ref().map(|(fit, wash)| (fit, *wash))
                    } else {
                        None
                    };
                    fh6_osd::draw::live_backing(&mut p, layout, o.host.anim.open.progress(), blocker);
                } else if let Some((fit, wash)) = scrim {
                    // Game frame, scrim, wash, cluster (SCRIM_FIT §3).
                    scrim::draw_scrim(&mut p, &fit);
                    scrim::draw_wash(&mut p, &fit, wash);
                }
                if let Some(active) = self.active.as_ref() {
                    active.cluster.draw(&mut p, layout);
                }
                if let Some(o) = osd {
                    let name = self.active.as_ref().map_or("", |a| model::theme_label(a.theme));
                    o.host.draw(&mut p, o.cfg, o.frame, o.now, name, o.status);
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

/// What one OSD frame draws with.
struct OsdFrame<'a> {
    host: &'a OsdHost,
    cfg: &'a AppConfig,
    frame: Frame,
    now: Duration,
    status: LiveStatus,
    /// A dash-blocker row has focus: draw the real blocker under the live cluster.
    blocker: bool,
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
        profiles,
        input,
        foreground,
        control,
        options,
        resources,
        game_process,
        on_bench_done,
        osd: osd_setup,
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
        font_override: None,
    };
    lp.adopt(gpu, surface);
    let mut osd = Osd {
        host: OsdHost::new(&lp.resources.tokens, osd_setup.fit),
        edit: AppConfig::default(),
        working: None,
        was_open: false,
        follow_until: None,
        pause: AutoPause::new(),
        injector: osd_setup.injector,
        saver: osd_setup.saver,
        epoch: Instant::now(),
        fonts_listed: false,
    };
    if options.open_osd {
        input.set_osd_open(true);
    }
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
    let mut car_config = CarConfig::default();

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

        let live_cfg = config.current();
        let frame = telemetry.latest();
        let fresh = !frame.is_stale(wake);
        let bypass = options.demo || options.bench.is_some();
        let race_on = frame.data.is_race_on == 1;
        let game_foreground = foreground.is_game_foreground();
        let camera_view = View::from_camera_index(i32::try_from(input.camera_index()).unwrap_or(3));

        // OSD edges: open snapshots the config (and pauses mid-race); close flushes the save.
        let osd_open = input.osd_open();
        let pause_inputs = PauseInputs {
            racing: fresh && race_on,
            game_foreground,
            auto_pause: live_cfg.osd_auto_pause_game,
        };
        if osd_open && !osd.was_open {
            osd.opened(&live_cfg, camera_view, &input, pause_inputs);
        } else if !osd_open && osd.was_open {
            osd.closed(wake, &input);
        }
        osd.was_open = osd_open;
        if osd.pause.poll(&mut *osd.injector, wake - osd.epoch, pause_inputs) {
            tracing::info!("OSD closed: ESC sent to resume the game");
        }
        if !osd.fonts_listed {
            if let Some(kit) = lp.gpu.as_ref().and_then(|g| g.kit.as_ref()) {
                osd.host.set_fonts(kit.fonts().families());
                osd.fonts_listed = true;
            }
        }
        // The OSD's working config (while open, and until the saved file reloads) is the base;
        // the car's profile overlays it.
        let base_cfg = osd.config_for_frame(&live_cfg, osd_open, wake);
        let live_preview = osd_open;
        let synthetic = uses_dyno(bypass, live_preview, fresh, race_on);
        let synth_data;
        let data = if synthetic {
            synth_data = dyno.tick(dt.as_secs_f32());
            &synth_data
        } else {
            &frame.data
        };
        // Per-car overrides switch with the car ordinal (cached; no allocation per frame).
        let cfg = car_config.resolve(&base_cfg, &profiles.current(), data.car_ordinal);
        let vis = visibility.update(
            VisibilityInputs {
                race_on,
                stream_fresh: fresh,
                game_foreground,
                user_hidden: control.user_hidden(),
                live_preview,
                bypass,
            },
            dt,
        );
        // Only a packet that is actually drawn counts toward the latency stats.
        let new_packet = !synthetic && fresh && frame.seq != last_seq;
        last_seq = frame.seq;
        let osd_frame = Frame::new(lp.size);

        // Chase views show the floating HUD only when `ChaseHudEnabled` is on; the OSD's live
        // view always shows.
        let view_shown = osd_open || camera_view != View::Chase || cfg.chase_hud_enabled;
        if vis.opacity <= 0.0 || !view_shown {
            let layout = Layout::from_config(&cfg, &lp.resources.tokens, camera_view, lp.size);
            if osd_open {
                // Hidden (the game lost focus): input meant for another window is dropped.
                let _ = input.take_edges();
            }
            if !blank_presented {
                // Hide = one fully transparent frame, then idle.
                blank_presented = matches!(
                    lp.render(0.0, &layout, false, false, None, None),
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
        if !(new_packet || vis.animating || synthetic || cluster_moving || osd_open || lp.active.is_none()) {
            continue;
        }

        let build_start = Instant::now();
        // Controller input lands in the working config; this very frame shows it.
        let mut cfg = cfg;
        // The OSD rows show and edit the global config; the live view shows what this car really
        // gets (its profile on top), and a row the profile overrides says so.
        let mut rows_cfg = Arc::clone(&cfg);
        if osd_open {
            if osd.navigate(&input, build_start) {
                input.set_osd_open(false);
            }
            rows_cfg = osd.config_for_frame(&live_cfg, true, build_start);
            cfg = car_config.resolve(&rows_cfg, &profiles.current(), data.car_ordinal);
            osd.host.note_profile(&rows_cfg, &cfg);
            lp.prepare_osd(&mut osd.host, &rows_cfg);
        }
        // The OSD's live view shows the view being edited, gliding into the live zone.
        let view = if osd_open { osd.host.edit_view() } else { camera_view };
        let base_layout = Layout::from_config(&cfg, &lp.resources.tokens, view, lp.size);
        let layout = if osd_open {
            osd.host.live_layout(&base_layout, osd_frame)
        } else {
            base_layout
        };
        lp.ensure_cluster(&cfg, view);
        lp.ensure_palette(&cfg, data.car_ordinal);
        lp.update_refraction(&cfg, &layout, &game_process);
        let update_dt = (build_start - lp.last_update).as_secs_f32().min(MAX_DT);
        lp.last_update = build_start;
        let signals = lp.smoother.update(data, &cfg, &lp.resources.tokens, update_dt);
        let font_override = lp.font_override;
        let fonts = lp
            .gpu
            .as_ref()
            .and_then(|g| g.kit.as_ref())
            .map(|k| k.font_choice(font_override));
        if let (Some(active), Some(fonts)) = (lp.active.as_mut(), fonts) {
            let input_frame = FrameInput {
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
                reduced_flash: model::reduced_flash(&cfg),
            };
            active.cluster.update(&input_frame);
            if osd_open {
                if let Some(kit) = lp.gpu.as_ref().and_then(|g| g.kit.as_ref()) {
                    osd.host.update(&input_frame, kit, osd_frame, &rows_cfg);
                }
            }
        }
        // Cockpit: scrim + wash under the cluster. Hood and chase: scrim 0 (`ScrimFit` gives 0).
        let scrim_fit = ScrimFit::from_config(&cfg, view.fit_view());
        let scrim = scrim_fit
            .visible()
            .then(|| (scrim_fit, Wash::from_config(&cfg, lp.palette.primary)));
        let built = build_start.elapsed();

        let stats_on = options.stats || model::show_stats(&cfg);
        let osd_draw = osd_open.then(|| OsdFrame {
            host: &osd.host,
            cfg: &rows_cfg,
            frame: osd_frame,
            now: build_start - osd.epoch,
            status: if synthetic {
                LiveStatus::Dyno
            } else {
                LiveStatus::Telemetry
            },
            blocker: osd.host.blocker_focused(&rows_cfg),
        });
        if let Some(FrameOutcome::Presented { timings, at }) =
            lp.render(vis.opacity, &layout, true, stats_on, scrim, osd_draw.as_ref())
        {
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
            if stats_on && lp.frames.is_multiple_of(30) {
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
        let theme_for = fh6_osd::theme_for_view;
        let mut cfg = AppConfig {
            cockpit_hud_style: "LiquidGlass".into(),
            bonnet_hud_style: "Gt7HoodPro".into(),
            ..AppConfig::default()
        };
        assert_eq!(theme_for(&cfg, View::Cockpit), ClusterTheme::LiquidGlass);
        assert_eq!(theme_for(&cfg, View::Hood), ClusterTheme::Gt7HoodPro);
        assert_eq!(
            theme_for(&cfg, View::Chase),
            ClusterTheme::LiquidGlass,
            "an empty ChaseHudStyle follows the cockpit"
        );
        cfg.chase_hud_style = "RetroCyber".into();
        assert_eq!(theme_for(&cfg, View::Chase), ClusterTheme::RetroCyber);
        let empty = AppConfig {
            cockpit_hud_style: String::new(),
            hud_style: String::new(),
            ..AppConfig::default()
        };
        assert_eq!(theme_for(&empty, View::Cockpit), ClusterTheme::LiquidGlass);
        // The OSD writes the chase cluster to the typed Phase 8 key.
        fh6_osd::model::set_theme_for_view(&mut cfg, View::Chase, ClusterTheme::Gt7HoodPro);
        assert_eq!(cfg.chase_hud_style, "Gt7HoodPro");
        assert_eq!(theme_for(&cfg, View::Chase), ClusterTheme::Gt7HoodPro);
    }

    #[test]
    fn car_config_switches_with_the_ordinal_and_caches() {
        let base = Arc::new(AppConfig {
            per_car_profile_enabled: true,
            oem_dash_opaque_pct: 80,
            ..AppConfig::default()
        });
        let profiles = Arc::new(
            CarProfilesFile::from_json_str(
                r#"{"Version":"2.0.0","Profiles":{"3595":{"CarOrdinal":3595,"CockpitAnchorY":1820,"OemDashOpaquePct":95}}}"#,
            )
            .unwrap(),
        );
        let mut cc = CarConfig::default();
        let m4 = cc.resolve(&base, &profiles, 4122);
        assert!(Arc::ptr_eq(&m4, &base), "no profile: the base config itself");
        let sf90 = cc.resolve(&base, &profiles, 3595);
        assert_eq!(sf90.cockpit_anchor_y, 1820);
        assert_eq!(sf90.oem_dash_opaque_pct, 95);
        assert_eq!(ScrimFit::from_config(&sf90, View::Cockpit.fit_view()).opacity, 0.95);
        let again = cc.resolve(&base, &profiles, 3595);
        assert!(Arc::ptr_eq(&sf90, &again), "cached while nothing changes");
        let back = cc.resolve(&base, &profiles, 4122);
        assert_eq!(back.cockpit_anchor_y, 1835);
        let off = Arc::new(AppConfig {
            per_car_profile_enabled: false,
            ..(*base).clone()
        });
        assert_eq!(cc.resolve(&off, &profiles, 3595).cockpit_anchor_y, 1835, "profiles off");
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
