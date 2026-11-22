//! `FH6_WheelDash`: wires the threads together and runs the main-thread message loop.
//!
//! Threads: main (message loop, tray, foreground hook, timers), render (GPU, wakes per packet),
//! one UDP receiver per port, input (XInput, 125 Hz), config watcher, quit-signal listener,
//! and the loopback status server.
use crate::args::AppArgs;
use crate::process_watcher::ProcessWatcher;
use crate::render_loop::{self, RenderControl, RenderOptions, RenderSetup, RenderSummary};
use crate::{perf_report, widget_server};
use fh6_core::config::resolve_config_path;
use fh6_core::{ConfigWatcher, TelemetryEngine, TelemetryPorts, TelemetryReader};
use fh6_gpu::OverlaySurface;
use fh6_input::{InputPoller, InputShared};
use fh6_win::app_window::{AppEvent, AppWindow, Flow, TrayCommand};
use fh6_win::system::{InstanceClaim, SingleInstance};
use fh6_win::{monitor, ForegroundWatcher, OverlayWindow, QuitSignal};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

const TIMER_FOREGROUND: usize = 1;
const TIMER_SLOW: usize = 2;
/// Foreground backstop poll (the WinEvent hook is the primary signal).
const FOREGROUND_POLL: Duration = Duration::from_millis(100);
const SLOW_TICK: Duration = Duration::from_secs(1);

/// Why the app could not start.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A Win32 call failed.
    #[error(transparent)]
    Win(#[from] fh6_win::WinError),
    /// The GPU layer failed.
    #[error(transparent)]
    Gpu(#[from] fh6_gpu::GpuError),
    /// A thread or file operation failed.
    #[error("{context}")]
    Io {
        /// What was being done.
        context: &'static str,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// No monitor to put the overlay on.
    #[error("no monitor found")]
    NoMonitor,
}

fn io_err(context: &'static str) -> impl FnOnce(std::io::Error) -> AppError {
    move |source| AppError::Io { context, source }
}

/// Wakes the render thread from any thread, once it exists.
#[derive(Clone, Default)]
struct Waker(Arc<OnceLock<Thread>>);

impl Waker {
    fn wake(&self) {
        if let Some(thread) = self.0.get() {
            thread.unpark();
        }
    }
    fn notify(&self) -> Arc<dyn Fn() + Send + Sync> {
        let me = self.clone();
        Arc::new(move || me.wake())
    }
}

fn start_engine(ports: TelemetryPorts, reader: TelemetryReader, waker: &Waker) -> TelemetryEngine {
    let mut engine = TelemetryEngine::with_reader(ports, reader).with_notify(waker.notify());
    if let Err(err) = engine.start() {
        tracing::error!(error = %err, "telemetry engine not started (is another app using the port?)");
    }
    engine
}

/// Runs the overlay until it is asked to quit. Returns the process exit code.
///
/// # Errors
/// [`AppError`] if the window, GPU or a required thread cannot be created.
pub fn run(args: AppArgs) -> Result<i32, AppError> {
    monitor::enable_per_monitor_dpi_awareness();
    let _instance = match SingleInstance::claim()? {
        InstanceClaim::First(guard) => guard,
        InstanceClaim::AlreadyRunning => {
            tracing::info!("FH6_WheelDash is already running");
            if args.open_widget {
                let _ = fh6_win::system::shell_open(&widget_server::url());
            }
            return Ok(0);
        }
    };

    let config_path = args.config.clone().unwrap_or_else(resolve_config_path);
    tracing::info!(path = %config_path.display(), "config");
    let waker = Waker::default();
    let watcher = ConfigWatcher::spawn(config_path.clone(), Duration::from_millis(250), waker.notify())
        .map_err(io_err("starting the config watcher"))?;
    let cfg = watcher.current();

    let reader = TelemetryReader::new();
    let engine = start_engine(cfg.telemetry_ports(), reader.clone(), &waker);
    if let Err(err) = widget_server::spawn(reader.clone()) {
        tracing::warn!(error = %err, "status server not started");
    }
    if args.open_widget {
        let _ = fh6_win::system::shell_open(&widget_server::url());
    }

    let input = Arc::new(InputShared::new(usize::try_from(cfg.current_camera_index).unwrap_or(3)));
    input.set_demo_mode(args.demo);
    let poller = InputPoller::spawn(Arc::clone(&input), watcher.handle(), reader.clone(), waker.notify())
        .map_err(io_err("starting the input thread"))?;

    let rect = monitor::primary_monitor_rect().ok_or(AppError::NoMonitor)?;
    let overlay = Rc::new(OverlayWindow::create(rect)?);
    let (gpu, surface) = OverlaySurface::create(overlay.surface_handle(), rect.width, rect.height)?;
    let foreground = ForegroundWatcher::install(&args.game_process, waker.notify())?;
    let control = Arc::new(RenderControl::new());

    let handler = Handler {
        control: Arc::clone(&control),
        input: Arc::clone(&input),
        overlay: Rc::clone(&overlay),
        foreground_reader: foreground.reader(),
        watcher_handle: watcher.handle(),
        config_path: config_path.clone(),
        reader: reader.clone(),
        engine,
        waker: waker.clone(),
        game: ProcessWatcher::new(&args.game_process),
        game_process: args.game_process.clone(),
        auto_exit: args.auto_exit,
        current_rect: rect,
    };
    let foreground = Rc::new(foreground);
    let handler = {
        let foreground = Rc::clone(&foreground);
        let mut handler = handler;
        Box::new(move |ctl: &fh6_win::app_window::WindowCtl, event: AppEvent| handler.on_event(ctl, event, &foreground))
    };
    let app_window = AppWindow::create("FH6 WheelDash: starting", handler)?;
    let close = app_window.close_handle();

    let options = RenderOptions {
        demo: args.demo,
        stats: args.stats,
        bench: args.bench_seconds.map(Duration::from_secs),
    };
    // Before the render thread: if this fails, nothing is running yet that would need stopping.
    let quit_listener = spawn_quit_listener(app_window.close_handle())?;
    let (render_join, render_thread) = render_loop::spawn(RenderSetup {
        gpu,
        surface,
        window: overlay.surface_handle(),
        telemetry: reader,
        config: watcher.handle(),
        input: Arc::clone(&input),
        foreground: foreground.reader(),
        control: Arc::clone(&control),
        options,
        resources: crate::resources::KitResources::resolve(&config_path),
        on_bench_done: Box::new(move || close.request_close()),
    })
    .map_err(io_err("starting the render thread"))?;
    let _ = waker.0.set(render_thread.clone());

    let ctl = app_window.ctl();
    ctl.set_timer(TIMER_FOREGROUND, FOREGROUND_POLL);
    ctl.set_timer(TIMER_SLOW, SLOW_TICK);
    tracing::info!(
        monitor = ?rect,
        game = %args.game_process,
        demo = args.demo,
        "FH6 WheelDash running"
    );
    let code = app_window.run();
    tracing::info!(code, "message loop ended; shutting down");

    // Shutdown order: render thread (drops the surface and its window handle) before the
    // window; the window before the hook and the shared overlay.
    control.stop();
    render_thread.unpark();
    let summary = render_join.join().ok();
    tracing::info!("render thread joined");
    quit_listener.stop();
    drop(app_window);
    drop(foreground);
    drop(poller);
    drop(overlay);
    tracing::info!("windows closed");
    if let (Some(seconds), Some(summary)) = (args.bench_seconds, summary.as_ref()) {
        write_bench(&args.bench_out, seconds, summary, &args);
    }
    Ok(code)
}

fn write_bench(path: &Path, seconds: u64, summary: &RenderSummary, args: &AppArgs) {
    let text = perf_report::markdown(summary, seconds, args.stats);
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        let _ = std::fs::create_dir_all(dir);
    }
    match std::fs::write(path, text) {
        Ok(()) => tracing::info!(path = %path.display(), "benchmark report written"),
        Err(err) => tracing::error!(path = %path.display(), error = %err, "benchmark report not written"),
    }
}

struct QuitListener {
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl QuitListener {
    fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn spawn_quit_listener(close: fh6_win::app_window::CloseHandle) -> Result<QuitListener, AppError> {
    let stop = Arc::new(AtomicBool::new(false));
    let thread = {
        let stop = Arc::clone(&stop);
        thread::Builder::new()
            .name("fh6-quit-signal".into())
            .spawn(move || {
                let signal = match QuitSignal::create() {
                    Ok(s) => s,
                    Err(err) => {
                        tracing::warn!(error = %err, "quit signal unavailable");
                        return;
                    }
                };
                while !stop.load(Ordering::Relaxed) {
                    if signal.wait(Duration::from_millis(250)) {
                        tracing::info!("quit requested by FH6_SteamLauncher");
                        close.request_close();
                        return;
                    }
                }
            })
            .map_err(io_err("starting the quit-signal thread"))?
    };
    Ok(QuitListener {
        stop,
        thread: Some(thread),
    })
}

struct Handler {
    control: Arc<RenderControl>,
    input: Arc<InputShared>,
    overlay: Rc<OverlayWindow>,
    foreground_reader: fh6_win::ForegroundReader,
    watcher_handle: fh6_core::ConfigHandle,
    config_path: PathBuf,
    reader: TelemetryReader,
    engine: TelemetryEngine,
    waker: Waker,
    game: ProcessWatcher,
    game_process: String,
    auto_exit: bool,
    current_rect: monitor::MonitorRect,
}

impl Handler {
    fn on_event(&mut self, ctl: &fh6_win::app_window::WindowCtl, event: AppEvent, fg: &ForegroundWatcher) -> Flow {
        match event {
            AppEvent::Timer(TIMER_FOREGROUND) => {
                fg.poll();
                if self.foreground_reader.is_game_foreground() {
                    self.follow_game_monitor();
                    self.overlay.reassert_topmost();
                }
                Flow::Continue
            }
            AppEvent::Timer(TIMER_SLOW) => self.slow_tick(ctl),
            AppEvent::Timer(_) => Flow::Continue,
            AppEvent::Tray(TrayCommand::ToggleOverlay) => {
                let hidden = self.control.toggle_user_hidden();
                ctl.set_overlay_checked(!hidden);
                self.waker.wake();
                Flow::Continue
            }
            AppEvent::Tray(TrayCommand::OpenConfigurator) => {
                // Until the Phase 10 web configurator: open config.json; saves hot-reload.
                if let Err(err) = fh6_win::system::shell_open(&self.config_path.to_string_lossy()) {
                    tracing::warn!(error = %err, "could not open the config file");
                }
                Flow::Continue
            }
            AppEvent::Tray(TrayCommand::OpenOsd) => {
                let open = self.input.toggle_osd();
                tracing::info!(open, "OSD toggled from the tray");
                self.waker.wake();
                Flow::Continue
            }
            AppEvent::Tray(TrayCommand::Quit) | AppEvent::CloseRequested => Flow::Quit,
        }
    }

    fn follow_game_monitor(&mut self) {
        let Some(hwnd) = self.foreground_reader.game_window() else {
            return;
        };
        let Some(rect) = monitor::monitor_rect_for_window(hwnd) else {
            return;
        };
        if rect == self.current_rect {
            return;
        }
        match self.overlay.set_bounds(rect) {
            Ok(()) => {
                tracing::info!(?rect, "overlay moved to the game's monitor");
                self.current_rect = rect;
                self.control.request_resize(rect.width, rect.height);
                self.waker.wake();
            }
            Err(err) => tracing::warn!(error = %err, "could not move the overlay"),
        }
    }

    fn slow_tick(&mut self, ctl: &fh6_win::app_window::WindowCtl) -> Flow {
        let processes = fh6_win::system::running_process_names();
        if let Some(state) = self.game.check(&processes) {
            tracing::info!(?state, game = %self.game_process, "game process");
        }
        if self.auto_exit && self.game.game_has_exited() {
            tracing::info!("game exited; --auto-exit quits");
            return Flow::Quit;
        }

        let cfg = self.watcher_handle.current();
        let wanted = cfg.telemetry_ports();
        if wanted != self.engine.ports() {
            tracing::info!(?wanted, "telemetry ports changed; restarting the engine");
            self.engine.shutdown();
            self.engine = start_engine(wanted, self.reader.clone(), &self.waker);
        } else if self.engine.bound_ports().is_empty() {
            // The port was busy at start (a previous instance still exiting, another tool):
            // keep retrying once a second.
            match self.engine.start() {
                Ok(()) => tracing::info!(ports = ?self.engine.bound_ports(), "telemetry engine started on retry"),
                Err(err) => tracing::debug!(error = %err, "telemetry port still busy"),
            }
        }

        let frame = self.reader.latest();
        let now = Instant::now();
        let status = if self.engine.bound_ports().is_empty() {
            format!("FH6 WheelDash: UDP port {} unavailable", wanted.primary)
        } else if frame.is_stale(now) {
            format!("FH6 WheelDash: waiting for telemetry on UDP {}", wanted.primary)
        } else if frame.data.is_race_on == 1 {
            "FH6 WheelDash: live".to_string()
        } else {
            "FH6 WheelDash: game paused or in menus".to_string()
        };
        ctl.set_tooltip(&status);
        Flow::Continue
    }
}
