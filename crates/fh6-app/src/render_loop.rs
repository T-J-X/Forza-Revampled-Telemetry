//! The render thread: wakes on every packet (or each refresh while something animates), lets
//! the active cluster update and draw with the render kit, and presents. No queue, no lock, no
//! readback on this path.
use crate::latency::{caught_next_vsync, FrameStats, Percentiles, SAMPLE_CAPACITY};
use crate::refraction::{RefractionSource, RETRY};
use crate::resources::KitResources;
use fh6_core::oled_care::{OledCare, OledCareEngine, OledCareInputs};
use fh6_core::{
    AppConfig, CarProfilesFile, ClusterTheme, ConfigHandle, FH6TelemetryData, ProfilesHandle, ScrimFit,
    TelemetryReader, VisibilityController, VisibilityInputs,
};
use fh6_gpu::seven_seg::StackStr;
use fh6_gpu::{FrameOutcome, GpuContext, GpuError, OverlaySurface};
use fh6_input::InputShared;
use fh6_kit::color::srgb_to_linear;
use fh6_kit::scrim::{self, Wash};
use fh6_kit::{
    vec2, Affine, Align, BlendMode, Cluster, Color, FontId, FrameInput, FrameStats as KitFrameStats, Kit, Layout, Logo,
    Painter, Palette, Rect, Shape, SignalSmoother, TextStyle, VAlign, View,
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
/// The hide frame: fully transparent.
const HIDDEN: Fade = Fade {
    opacity: 0.0,
    care: OledCare {
        offset_px: (0, 0),
        dim: 1.0,
    },
};

/// Cross-thread controls for the render thread.
#[derive(Debug, Default)]
pub struct RenderControl {
    stop: AtomicBool,
    user_hidden: AtomicBool,
    pending_size: AtomicU64,
    failed: AtomicBool,
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

    /// True once the render thread has panicked. Nothing can draw the dash, or hide it, after
    /// that, so the app has to quit.
    pub fn render_failed(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
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
    /// CPU time per presented frame: the whole iteration from the wake to the end of the frame,
    /// less the latency wait.
    pub cpu_frame: Percentiles,
    /// CPU time per presented frame that the OLED idle dim turned down.
    pub cpu_frame_dimmed: Percentiles,
    /// Blocked on the swapchain latency wait.
    pub acquire_wait: Percentiles,
    /// Config, layout, signals and the cluster's `update`.
    pub build: Percentiles,
    /// Scrim, wash, the cluster's `draw`, the dim and the OSD, recorded into the kit's arena.
    pub draw: Percentiles,
    /// Kit render (pack, upload, record) and submit.
    pub encode_submit: Percentiles,
    /// Of `encode_submit`: the swapchain view, the encoder, the kit's render and `finish`.
    pub record: Percentiles,
    /// Of `record`: the kit's own render (pack, the upload copy, pass recording).
    pub kit: Percentiles,
    /// The loop's checks between the wake and the build (config, telemetry, OSD edges,
    /// visibility, OLED care).
    pub before_build: Percentiles,
    /// After `present` returned: the kit's staging hand-back and the failure checks.
    pub after_present: Percentiles,
    /// Of `encode_submit`: `Queue::submit`.
    pub submit: Percentiles,
    /// The `present` call.
    pub present: Percentiles,
    /// Render passes the kit recorded, over every presented frame.
    pub passes: u64,
    /// Draw calls the kit recorded, over every presented frame.
    pub draw_calls: u64,
    /// New packets shown at the first vblank after arrival.
    pub vsync_hits: u64,
    /// New packets checked for that.
    pub vsync_checked: u64,
    /// Display refresh interval reported by DWM.
    pub refresh: Duration,
}

/// Starts the render thread. Returns its join handle and its `Thread` (for unparking).
///
/// If the thread panics, [`RenderControl::render_failed`] turns true and `on_panic` runs once
/// on the dying thread, so it must not block (the app passes a close request). A panic in the
/// loop is reported as its unwind starts, before the loop lets go of the swapchain.
///
/// # Errors
/// Returns the I/O error if the thread cannot be spawned.
pub fn spawn(
    setup: RenderSetup,
    on_panic: impl FnOnce() + Send + 'static,
) -> io::Result<(JoinHandle<RenderSummary>, Thread)> {
    let watch = PanicWatch {
        control: Arc::clone(&setup.control),
        on_panic: Some(Box::new(on_panic)),
    };
    let handle = thread::Builder::new()
        .name("fh6-render".into())
        .spawn(move || run(setup, watch))?;
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
        // OSD edits swap the base on every step; only a new car or profiles file is news.
        let car_changed = self.current.is_none()
            || self.ordinal != ordinal
            || !self.profiles.as_ref().is_some_and(|p| Arc::ptr_eq(p, profiles));
        let effective = match profiles.effective_config(base, ordinal) {
            Some(cfg) => {
                if car_changed {
                    tracing::info!(car_ordinal = ordinal, "car profile applied");
                }
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

/// The dyno's next frame, for the car last seen racing. The dyno knows revs and gears but no
/// car, and everything that reads the drawn packet (the car's profile, its brand palette and
/// logo, the cluster) has to see the same car as while racing.
fn dyno_frame(dyno: &mut LiveDynoGenerator, dt: f32, last_raced_car: i32) -> FH6TelemetryData {
    FH6TelemetryData {
        car_ordinal: last_raced_car,
        ..dyno.tick(dt)
    }
}

/// `layout` moved by the OLED orbit `offset` (whole target pixels). A move that would push the
/// cluster's envelope out of the layout's safe zone is cut back, so the clamp of
/// [`Layout::from_config`] still holds.
fn orbited(mut layout: Layout, offset: (i32, i32)) -> Layout {
    let on_screen = layout.transform().apply_rect(&layout.envelope);
    let safe = layout.safe;
    // `room_back` and `room_on` are how far the envelope can move towards the near and the far
    // side of the safe zone. Rounding them inwards keeps the move a whole number of pixels.
    let fit = |shift: i32, room_back: f32, room_on: f32| {
        (shift as f32).clamp(room_back.min(0.0).ceil(), room_on.max(0.0).floor())
    };
    layout.anchor.x += fit(offset.0, safe.min.x - on_screen.min.x, safe.max.x - on_screen.max.x);
    layout.anchor.y += fit(offset.1, safe.min.y - on_screen.min.y, safe.max.y - on_screen.max.y);
    layout
}

/// How a frame is shown.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Fade {
    /// The visibility fade of the whole frame, 0..=1.
    opacity: f32,
    /// The OLED care for the frame. The layout already carries the orbit; the dim and the stats
    /// lines take it from here.
    care: OledCare,
}

/// Everything the dash lights, in cluster units: the cluster's envelope with `glow_reach` of
/// room for glow that spills over it and, when a scrim is drawn, its dome (the wash runs along
/// the dome's top edge).
fn lit_area(layout: &Layout, glow_reach: f32, scrim: Option<&ScrimFit>) -> Rect {
    // A broken token must not turn the area inside out or make it infinite.
    let reach = if glow_reach.is_finite() {
        glow_reach.max(0.0)
    } else {
        0.0
    };
    let cluster = layout.envelope.expand(reach);
    scrim.map_or(cluster, |fit| cluster.union(&scrim::dome(fit).bounds()))
}

/// Whether `dim` (an [`OledCare::dim`](fh6_core::oled_care::OledCare::dim)) dims a frame at
/// all. 1 does not, and neither does a value outside 0..=1.
fn is_dimmed(dim: f32) -> bool {
    (0.0..1.0).contains(&dim)
}

/// The OLED idle dim. Turns down the light of everything recorded so far inside `area` (cluster
/// units) and leaves its coverage alone: black stays black and an opaque plate stays opaque, so
/// neither the scrim nor a cluster's backing lets the game through.
///
/// `dim` is on the scale of an opacity, 0..=1: white ends up as bright as white at opacity
/// `dim`, and every colour loses the same share of its light. At 1, or with a value outside
/// that range, nothing is dimmed.
///
/// It cannot reach bloom, which the kit adds after every draw. The render loop therefore draws
/// a dimmed frame without bloom.
pub fn dim_dash(p: &mut Painter<'_>, area: Rect, dim: f32) {
    if !is_dimmed(dim) {
        return;
    }
    // An opacity scales the encoded value on screen (ADR 0001); the same step in light is its
    // sRGB decode.
    let light = srgb_to_linear(dim);
    p.scoped(|p| {
        // Multiply by black at alpha `1 - light`: the colour underneath is scaled by `light` and
        // its alpha is kept.
        p.set_blend(BlendMode::Multiply);
        p.fill(&Shape::rect(area), Color::BLACK.with_alpha(1.0 - light));
    });
}

/// Where and how strongly the stats lines are drawn: they ride the orbit and dim with the dash,
/// so they cannot burn in either. Returns the shift (target pixels) and the opacity.
fn stats_look(care: OledCare) -> (Affine, f32) {
    let (dx, dy) = care.offset_px;
    let opacity = if is_dimmed(care.dim) { care.dim } else { 1.0 };
    (Affine::translate(vec2(dx as f32, dy as f32)), opacity)
}

/// Tells the rest of the app when the render thread panics: it reports when it is dropped during
/// an unwind, so whoever holds it decides how early that is.
struct PanicWatch {
    control: Arc<RenderControl>,
    on_panic: Option<Box<dyn FnOnce() + Send>>,
}

impl Drop for PanicWatch {
    fn drop(&mut self) {
        if !thread::panicking() {
            return;
        }
        self.control.failed.store(true, Ordering::Relaxed);
        if let Some(report) = self.on_panic.take() {
            report();
        }
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
            View::Cockpit => cfg.cockpit_anchor_x,
            View::Chase => cfg.chase_anchor_x,
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
        let cluster = {
            let mut setup = kit.setup(&gpu.ctx.device, &gpu.ctx.queue, self.resources.assets.as_deref());
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
        host.prepare(
            kit,
            &gpu.ctx.device,
            &gpu.ctx.queue,
            self.resources.assets.as_deref(),
            cfg,
        );
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

    /// Records and presents one frame at `fade`. With `draw` false, presents a fully transparent
    /// frame (hide). `None` when nothing was presented; otherwise the outcome, the time spent
    /// drawing into the kit's arena (before `render_with`) and the kit's own render time.
    ///
    /// The OLED idle dim turns down the light of the wash and of the cluster ([`dim_dash`]). The
    /// scrim is black, so it stays as it is: a faded scrim would uncover the game's own gauges
    /// behind an idle dash.
    fn render(
        &mut self,
        fade: Fade,
        layout: &Layout,
        draw: bool,
        stats_on: bool,
        scrim: Option<(ScrimFit, Wash)>,
        osd: Option<&OsdFrame<'_>>,
    ) -> Option<(FrameOutcome, Duration, KitFrameStats)> {
        let draw_start = Instant::now();
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
            return result
                .ok()
                .map(|outcome| (outcome, Duration::ZERO, KitFrameStats::default()));
        };
        // The OSD dims the game through the kit's underlay (free in the resolve); a hide frame
        // and the normal dash have none.
        kit.set_underlay(match osd {
            Some(o) if draw => fh6_osd::draw::underlay(o.host.anim.open.progress()),
            _ => 0.0,
        });
        // No draw can dim bloom (see `dim_dash`), so a dimmed frame has none.
        kit.set_bloom(!is_dimmed(fade.care.dim));
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
                // The OSD keeps the dash awake, so the dim only ever meets the plain dash.
                let lit = lit_area(
                    layout,
                    p.tokens().glow.max_bloom_radius,
                    scrim.as_ref().map(|(fit, _)| fit),
                );
                dim_dash(&mut p, lit, fade.care.dim);
                if let Some(o) = osd {
                    let name = self.active.as_ref().map_or("", |a| model::theme_label(a.theme));
                    o.host.draw(&mut p, o.cfg, o.frame, o.now, name, o.status);
                }
                if stats_on {
                    draw_stats(&mut p, &self.stat_lines, self.size, fade.care);
                }
            }
        }
        let drawn = draw_start.elapsed();
        let mut kit_stats = KitFrameStats::default();
        let result = surface.render_with(ctx, |ctx, encoder, view, format, size| {
            kit_stats = kit.render(&ctx.device, &ctx.queue, encoder, view, format, size, fade.opacity);
        });
        // `render_with` has submitted the frame (or dropped it): the kit may map its upload
        // staging buffer again.
        kit.frame_submitted();
        let failed = ctx.has_failed();
        match result {
            Ok(outcome) if !failed => Some((outcome, drawn, kit_stats)),
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
fn draw_stats(p: &mut Painter<'_>, lines: &[StatLine; 4], size: (u32, u32), care: OledCare) {
    let k = size.1 as f32 / REF_H;
    let cap = 26.0 * k;
    let font = p.fonts().role(fh6_kit::FontRole::Display);
    let ink = p.palette().roles.ink;
    let style = TextStyle::new(font, cap)
        .paint(ink)
        .align(Align::Left, VAlign::CapTop)
        .tabular();
    let (shift, opacity) = stats_look(care);
    p.save();
    p.set_transform(shift);
    p.set_opacity(opacity);
    for (i, line) in lines.iter().enumerate() {
        let mut text = StackStr::<32>::new();
        let _ = write!(text, "{} {:.*}", line.label, line.decimals, line.value);
        p.text(vec2(64.0 * k, 64.0 * k + i as f32 * cap * 1.8), text.as_str(), &style);
    }
    p.restore();
}

fn run(setup: RenderSetup, watch: PanicWatch) -> RenderSummary {
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
    // Frame work runs from wake to `present` without being parked behind the game's threads or
    // slowed as background work (known defect 10: "CPU per frame" is wall-clock time here).
    let priority = fh6_win::system::raise_frame_thread();
    tracing::info!(
        class = ?priority.class(),
        full_speed = priority.full_speed(),
        performance_cores = priority.performance_cores(),
        "render thread scheduling class"
    );
    let refresh = fh6_win::system::composition_timing()
        .map(|t| t.refresh_period)
        .unwrap_or(FALLBACK_REFRESH);
    let measure_vblank = options.stats || options.bench.is_some();
    let palette = Palette::family(&resources.tokens, "modernDigital");
    // Declared before `lp`: locals are dropped last-declared first, so when a panic unwinds this
    // function the swapchain in `lp` goes (and the window shows nothing) before the OSD's saver
    // waits for its writer thread.
    let mut osd = Osd {
        host: OsdHost::new(&resources.tokens, osd_setup.fit),
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
    if options.open_osd {
        input.set_osd_open(true);
    }
    let mut visibility = VisibilityController::default();
    let mut oled = OledCareEngine::new();
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
    // The car last seen in a racing packet: the dyno's frames (the OSD live view while paused)
    // name it, so they keep its profile and its colours.
    let mut last_car_ordinal = 0i32;

    tracing::info!(?refresh, "render loop started");
    // The last local, so it is the first one dropped: a panic in the loop is reported before
    // anything else here is torn down.
    let _watch = watch;
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
            oled.rest(dt);
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
        let mut osd_open = input.osd_open();
        if osd_open && !osd.was_open && control.user_hidden() && !bypass {
            // The overlay is hidden from the tray: an OSD nobody can see must not pause the game.
            input.set_osd_open(false);
            osd_open = false;
        }
        if fresh && race_on && frame.data.car_ordinal != 0 {
            last_car_ordinal = frame.data.car_ordinal;
        }
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
            synth_data = dyno_frame(&mut dyno, dt.as_secs_f32(), last_car_ordinal);
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
            oled.rest(dt);
            let layout = Layout::from_config(&cfg, &lp.resources.tokens, camera_view, lp.size);
            if osd_open {
                // Hidden (the game lost focus): input meant for another window is dropped.
                let _ = input.take_edges();
            }
            if !blank_presented {
                // Hide = one fully transparent frame, then idle.
                blank_presented = matches!(
                    lp.render(HIDDEN, &layout, false, false, None, None),
                    Some((FrameOutcome::Presented { .. }, _, _))
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
        // OLED care runs on the loop's clock on every iteration, also when no new frame is drawn.
        let care = oled.update(OledCareInputs::from_frame(&cfg, data, input.buttons(), osd_open), dt);
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
        // The whole dash (scrim, wash and cluster) rides the pixel orbit.
        let layout = orbited(layout, care.offset_px);
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
        let fade = Fade {
            opacity: vis.opacity,
            care,
        };
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
        if let Some((FrameOutcome::Presented { timings, at }, drawn, kit_stats)) =
            lp.render(fade, &layout, true, stats_on, scrim, osd_draw.as_ref())
        {
            lp.frames += 1;
            // The whole iteration on this thread, from the wake to the end of `render` (the
            // checks before the build, the build, the draw (known defect 17), the kit's render,
            // submit, present and the staging hand-back). Only the latency wait is left out (it
            // is not CPU work).
            let frame_end = Instant::now();
            let cpu = frame_end
                .saturating_duration_since(wake)
                .saturating_sub(timings.acquire_wait);
            lp.stats
                .before_build
                .record(build_start.saturating_duration_since(wake));
            lp.stats.after_present.record(frame_end.saturating_duration_since(at));
            lp.stats.cpu_frame.record(cpu);
            if is_dimmed(care.dim) {
                lp.stats.cpu_frame_dimmed.record(cpu);
            }
            lp.stats.acquire_wait.record(timings.acquire_wait);
            lp.stats.build.record(built);
            lp.stats.draw.record(drawn);
            lp.stats.encode_submit.record(timings.encode_submit);
            lp.stats.record.record(timings.record);
            lp.stats.kit.record(kit_stats.cpu);
            lp.stats.passes += u64::from(kit_stats.passes);
            lp.stats.draw_calls += u64::from(kit_stats.draw_calls);
            lp.stats.submit.record(timings.submit);
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
        cpu_frame_dimmed: lp.stats.cpu_frame_dimmed.percentiles(&mut lp.scratch),
        acquire_wait: lp.stats.acquire_wait.percentiles(&mut lp.scratch),
        build: lp.stats.build.percentiles(&mut lp.scratch),
        draw: lp.stats.draw.percentiles(&mut lp.scratch),
        encode_submit: lp.stats.encode_submit.percentiles(&mut lp.scratch),
        record: lp.stats.record.percentiles(&mut lp.scratch),
        kit: lp.stats.kit.percentiles(&mut lp.scratch),
        before_build: lp.stats.before_build.percentiles(&mut lp.scratch),
        after_present: lp.stats.after_present.percentiles(&mut lp.scratch),
        submit: lp.stats.submit.percentiles(&mut lp.scratch),
        present: lp.stats.present.percentiles(&mut lp.scratch),
        passes: lp.stats.passes,
        draw_calls: lp.stats.draw_calls,
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
    fn dyno_frames_name_the_last_raced_car() {
        let (mut dyno, mut plain) = (LiveDynoGenerator::new(), LiveDynoGenerator::new());
        let frame = dyno_frame(&mut dyno, 0.016, 3595);
        assert_eq!(frame.car_ordinal, 3595, "the OSD live view over a paused game");
        let unnamed = plain.tick(0.016);
        assert_eq!(unnamed.car_ordinal, 0, "the dyno itself names no car");
        assert_eq!(
            FH6TelemetryData {
                car_ordinal: 0,
                ..frame
            },
            unnamed,
            "everything else is the dyno's frame"
        );
    }

    #[test]
    fn orbit_moves_the_dash_by_whole_pixels() {
        let base = Layout::from_config(
            &AppConfig::default(),
            &fh6_kit::Tokens::default(),
            View::Cockpit,
            (3840, 2160),
        );
        let moved = orbited(base, (2, -1));
        assert_eq!(moved.anchor, vec2(1922.0, 1834.0));
        assert_eq!(
            Layout {
                anchor: base.anchor,
                ..moved
            },
            base,
            "only the anchor moves"
        );
        assert_eq!(orbited(base, (0, 0)), base);
    }

    #[test]
    fn orbit_never_pushes_the_dash_out_of_its_safe_zone() {
        // At scale 1 the envelope spans x -720..720 and y -320..240 around the anchor, and the
        // safe zone of a 3840 x 2160 target is x 160..3680, y 0..2120.
        // Each row: anchor, orbit offset, where the anchor ends up.
        for (anchor, offset, want) in [
            // Flush with the top: it can move down, not up.
            ((1920.0, 320.0), (0, -2), (1920.0, 320.0)),
            ((1920.0, 320.0), (0, 2), (1920.0, 322.0)),
            // One pixel from the right side: one pixel of room.
            ((2959.0, 1000.0), (2, 0), (2960.0, 1000.0)),
            ((2959.0, 1000.0), (-2, 0), (2957.0, 1000.0)),
            // 1.4 px from the bottom: the move stays a whole pixel.
            ((1920.0, 1878.6), (0, 2), (1920.0, 1879.6)),
            // Already past the left side: it may only move back in.
            ((700.0, 1000.0), (-2, 0), (700.0, 1000.0)),
            ((700.0, 1000.0), (2, 0), (702.0, 1000.0)),
        ] {
            let layout = Layout::centered(vec2(anchor.0, anchor.1), 1.0, (3840, 2160));
            let got = orbited(layout, offset).anchor;
            assert!(
                (got - vec2(want.0, want.1)).length() < 1e-3,
                "anchor {anchor:?}, offset {offset:?}: ended at {got:?}"
            );
        }
    }

    #[test]
    fn a_broken_glow_reach_leaves_the_cluster_itself_dimmed() {
        let layout = Layout::centered(vec2(1920.0, 1835.0), 1.0, (3840, 2160));
        for reach in [f32::NAN, f32::INFINITY, -50.0, -5000.0] {
            assert_eq!(lit_area(&layout, reach, None), layout.envelope, "glow reach {reach}");
        }
    }

    #[test]
    fn the_stats_lines_ride_the_orbit_and_dim_with_the_dash() {
        let (shift, opacity) = stats_look(OledCare {
            offset_px: (2, -1),
            dim: 0.35,
        });
        assert_eq!(shift.apply(vec2(64.0, 64.0)), vec2(66.0, 63.0));
        assert_eq!(opacity, 0.35);
        for dim in [1.0, f32::NAN, 7.0] {
            let (_, opacity) = stats_look(OledCare { offset_px: (0, 0), dim });
            assert_eq!(opacity, 1.0, "dim {dim}");
        }
    }

    #[test]
    fn the_idle_dim_covers_the_cluster_its_glow_and_the_dome() {
        let cfg = AppConfig::default();
        let layout = Layout::centered(vec2(1920.0, 1835.0), 1.0, (3840, 2160));
        // The envelope spans x -720..720 and y -320..240; glow reaches 96 further.
        let cluster = Rect::from_points(vec2(-816.0, -416.0), vec2(816.0, 336.0));
        assert_eq!(lit_area(&layout, 96.0, None), cluster, "hood and chase: no scrim");
        // The default dome: centre (0, -75), semi-axes 640 x 470, widest feather 140.
        let fit = ScrimFit::from_config(&cfg, View::Cockpit.fit_view());
        let lit = lit_area(&layout, 96.0, Some(&fit));
        assert_eq!(lit, Rect::from_points(vec2(-816.0, -685.0), vec2(816.0, 535.0)));
        // The wash is the lip along the top of the dome: from its apex at y -545 down to y -404,
        // never wider than the dome.
        for lip in [vec2(0.0, -545.0), vec2(-640.0, -404.0), vec2(640.0, -404.0)] {
            assert!(lit.contains(lip), "the wash at {lip:?} is dimmed with the dash");
        }
    }

    /// A stand-in for the render thread: `body` holds the watch as the render loop does.
    /// Returns how the thread ended, its control block and how often the panic was reported.
    fn watched<T: Send + 'static>(
        body: impl FnOnce(PanicWatch) -> T + Send + 'static,
    ) -> (thread::Result<T>, Arc<RenderControl>, u64) {
        let control = Arc::new(RenderControl::new());
        let reports = Arc::new(AtomicU64::new(0));
        let watch = PanicWatch {
            control: Arc::clone(&control),
            on_panic: Some(Box::new({
                let reports = Arc::clone(&reports);
                move || {
                    reports.fetch_add(1, Ordering::Relaxed);
                }
            })),
        };
        let joined = thread::spawn(move || body(watch)).join();
        (joined, control, reports.load(Ordering::Relaxed))
    }

    #[test]
    fn a_render_thread_panic_is_reported_once() {
        let (joined, control, reports) = watched(|_watch| -> u32 { panic!("render thread test panic") });
        assert!(joined.is_err(), "the panic still reaches the join");
        assert!(control.render_failed());
        assert_eq!(reports, 1);
    }

    #[test]
    fn a_panic_is_reported_before_the_loop_lets_go_of_what_it_owns() {
        /// Notes, when it is dropped, whether the failure had already been reported.
        struct Owned(Arc<RenderControl>, Arc<AtomicBool>);
        impl Drop for Owned {
            fn drop(&mut self) {
                self.1.store(self.0.render_failed(), Ordering::Relaxed);
            }
        }
        let reported_first = Arc::new(AtomicBool::new(false));
        let (joined, _, reports) = watched({
            let reported_first = Arc::clone(&reported_first);
            move |watch| -> u32 {
                let _owned = Owned(Arc::clone(&watch.control), reported_first);
                // Declared last, as in `run`, so the unwind drops it first.
                let _watch = watch;
                panic!("render thread test panic")
            }
        });
        assert!(joined.is_err());
        assert_eq!(reports, 1);
        assert!(reported_first.load(Ordering::Relaxed));
    }

    #[test]
    fn a_clean_render_thread_exit_is_not_a_failure() {
        let (joined, control, reports) = watched(|_watch| 7u32);
        assert_eq!(joined.ok(), Some(7));
        assert!(!control.render_failed());
        assert_eq!(reports, 0);
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
