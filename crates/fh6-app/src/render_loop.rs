//! The render thread: wakes on every packet (or each vsync while fading), draws the latest
//! packet only, and presents. No queue, no lock, no readback on this path.
use crate::latency::{caught_next_vsync, FrameStats, Percentiles, SAMPLE_CAPACITY};
use fh6_core::telemetry::FH6TelemetryData;
use fh6_core::{AppConfig, ConfigHandle, TelemetryReader, VisibilityController, VisibilityInputs};
use fh6_gpu::test_cluster::{self, StatLine, TestClusterInput};
use fh6_gpu::{FrameOutcome, GpuContext, GpuError, OverlaySurface, QuadBatch};
use fh6_input::InputShared;
use fh6_osd::LiveDynoGenerator;
use fh6_win::{ForegroundReader, OverlayHandle};
use std::io;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle, Thread};
use std::time::{Duration, Instant};

/// Reference resolution the config anchors are authored in.
const REF_W: f32 = 3840.0;
const REF_H: f32 = 2160.0;
/// Wait between checks while hidden (packets and focus changes wake the thread sooner).
const IDLE_WAIT: Duration = Duration::from_millis(50);
/// Fallback refresh interval when DWM timing is unavailable.
const FALLBACK_REFRESH: Duration = Duration::from_micros(16_667);
/// Benchmarks discard this much warm-up.
const BENCH_WARMUP: Duration = Duration::from_secs(2);

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
    /// Building the quad batch.
    pub build: Percentiles,
    /// Upload, record, submit.
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

/// `#RRGGBB` to straight sRGB-encoded RGBA, or `fallback`.
pub fn parse_hex(hex: &str, fallback: [f32; 4]) -> [f32; 4] {
    let digits = hex.trim().trim_start_matches('#');
    if digits.len() != 6 || !digits.is_ascii() {
        return fallback;
    }
    let channel = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).map(|v| f32::from(v) / 255.0);
    match (channel(0), channel(2), channel(4)) {
        (Ok(r), Ok(g), Ok(b)) => [r, g, b, 1.0],
        _ => fallback,
    }
}

/// Whether to draw the live dyno instead of the received packet. The demo, the bench and the
/// OSD live view all need a moving dash when the game is not actually driving: when the
/// stream is stale, and also when the game keeps sending `IsRaceOn == 0` packets while
/// paused (those carry frozen or zeroed values).
pub fn uses_dyno(bypass: bool, live_preview: bool, fresh: bool, race_on: bool) -> bool {
    (bypass || live_preview) && !(fresh && race_on)
}

/// The test cluster's input for one frame.
pub fn cluster_input(data: &FH6TelemetryData, cfg: &AppConfig, camera: usize, size: (u32, u32)) -> TestClusterInput {
    let (w, h) = (size.0 as f32, size.1 as f32);
    let (ax, ay, scale) = if camera == 4 {
        (cfg.bonnet_anchor_x, cfg.bonnet_anchor_y, cfg.bonnet_scale)
    } else {
        (cfg.cockpit_anchor_x, cfg.cockpit_anchor_y, cfg.cockpit_scale)
    };
    let kmh = cfg.speed_unit.eq_ignore_ascii_case("KMH") || cfg.speed_unit.eq_ignore_ascii_case("KPH");
    TestClusterInput {
        speed: if kmh { data.speed_kmh() } else { data.speed_mph() },
        gear: data.gear,
        rpm: data.current_engine_rpm,
        max_rpm: data.engine_max_rpm,
        shift_start: cfg.shift_start_pct.clamp(0.1, 1.0),
        redline: cfg.redline_pct.clamp(0.1, 1.0),
        throttle: data.throttle_pct(),
        brake: data.brake_pct(),
        anchor: [ax as f32 * w / REF_W, ay as f32 * h / REF_H],
        scale: scale * cfg.cluster_scale.clamp(0.7, 1.4) * h / REF_H,
        accent: parse_hex(&cfg.primary_hex, [0.0, 0.824, 1.0, 1.0]),
    }
}

/// The device and swapchain. `None` while they are being recreated after a loss.
struct Gpu {
    ctx: GpuContext,
    surface: OverlaySurface,
}

struct Loop {
    gpu: Option<Gpu>,
    /// Last known adapter, present mode and size, for the summary.
    adapter: String,
    present_mode: String,
    size: (u32, u32),
    batch: QuadBatch,
    stats: FrameStats,
    scratch: Vec<f32>,
    stat_lines: [StatLine; 4],
    frames: u64,
    packets: u64,
    fps_window: (Instant, u64),
    lost_since: Option<Instant>,
}

impl Loop {
    fn adopt(&mut self, gpu: Gpu) {
        self.adapter = gpu.ctx.adapter_summary().to_string();
        self.present_mode = format!("{:?}", gpu.surface.present_mode());
        self.size = gpu.surface.size();
        self.gpu = Some(gpu);
        self.lost_since = None;
    }

    /// Drops the device and swapchain so the window has no composition target left (it then
    /// shows nothing), and schedules a recreation.
    fn lose_gpu(&mut self, why: &str) {
        tracing::warn!(why, "dropping the GPU device; it is recreated in 1 s");
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
                self.adopt(Gpu { ctx, surface });
            }
            Err(err) => {
                tracing::warn!(error = %err, "recreating the GPU device failed; retrying");
                self.lost_since = Some(Instant::now());
            }
        }
    }

    /// Renders the current batch and handles loss. `None` when nothing was presented.
    fn render(&mut self, opacity: f32) -> Option<FrameOutcome> {
        let gpu = self.gpu.as_mut()?;
        let result = gpu.surface.render(&gpu.ctx, opacity, &self.batch);
        let failed = gpu.ctx.has_failed();
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
        on_bench_done,
    } = setup;
    let refresh = fh6_win::system::composition_timing()
        .map(|t| t.refresh_period)
        .unwrap_or(FALLBACK_REFRESH);
    let measure_vblank = options.stats || options.bench.is_some();
    let mut lp = Loop {
        gpu: None,
        adapter: String::new(),
        present_mode: String::new(),
        size: (1, 1),
        batch: QuadBatch::new(),
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
    };
    lp.adopt(Gpu { ctx: gpu, surface });
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

        if vis.opacity <= 0.0 {
            if !blank_presented {
                // Hide = one fully transparent frame, then idle.
                lp.batch.clear();
                blank_presented = matches!(lp.render(0.0), Some(FrameOutcome::Presented { .. }));
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
        if !(new_packet || vis.animating || synthetic) {
            continue;
        }

        let build_start = Instant::now();
        lp.batch.clear();
        let input_frame = cluster_input(data, &cfg, input.camera_index(), lp.size);
        test_cluster::build(&mut lp.batch, &input_frame);
        if options.stats {
            let line_h = (lp.size.1 as f32 / REF_H * 34.0).max(10.0);
            test_cluster::build_stats(&mut lp.batch, [line_h * 2.0, line_h * 2.0], line_h, &lp.stat_lines);
        }
        let built = build_start.elapsed();

        if let Some(FrameOutcome::Presented { timings, at }) = lp.render(vis.opacity) {
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
    fn hex_colours() {
        assert_eq!(parse_hex("#FFFFFF", [0.0; 4]), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(parse_hex("00d2ff", [0.0; 4])[0], 0.0);
        assert_eq!(parse_hex("#12", [0.5; 4]), [0.5; 4]);
        assert_eq!(parse_hex("#GGGGGG", [0.5; 4]), [0.5; 4]);
        assert_eq!(parse_hex("#ÅÅÅ", [0.5; 4]), [0.5; 4]);
    }

    #[test]
    fn dyno_replaces_paused_and_stale_packets_only_for_preview_and_demo() {
        // Real driving data always wins.
        assert!(!uses_dyno(true, true, true, true));
        // OSD live view while paused: the game keeps sending fresh IsRaceOn == 0 packets.
        assert!(uses_dyno(false, true, true, false));
        // OSD live view or demo with no stream at all.
        assert!(uses_dyno(false, true, false, false));
        assert!(uses_dyno(true, false, false, true));
        // Normal overlay never invents data; the visibility rules hide it instead.
        assert!(!uses_dyno(false, false, true, false));
        assert!(!uses_dyno(false, false, false, true));
    }

    #[test]
    fn anchors_scale_from_4k_and_follow_the_camera() {
        let cfg = AppConfig::default();
        let data = FH6TelemetryData {
            speed_ms: 10.0,
            ..FH6TelemetryData::default()
        };
        let cockpit = cluster_input(&data, &cfg, 3, (3840, 2160));
        assert_eq!(cockpit.anchor, [1920.0, 1835.0]);
        assert!((cockpit.scale - 1.45).abs() < 1e-6);
        assert!((cockpit.speed - 22.369).abs() < 0.01, "MPH by default");

        let hood = cluster_input(&data, &cfg, 4, (1920, 1080));
        assert_eq!(hood.anchor, [960.0, 880.0], "bonnet anchor at half resolution");
        assert!((hood.scale - 1.55 * 0.5).abs() < 1e-6);

        let kmh = AppConfig {
            speed_unit: "KMH".into(),
            ..AppConfig::default()
        };
        assert!((cluster_input(&data, &kmh, 3, (3840, 2160)).speed - 36.0).abs() < 0.01);
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
