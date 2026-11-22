//! The controller thread: polls XInput at 125 Hz and publishes results through atomics.
use crate::hold_state::HoldConfig;
use crate::tracker::{InputControllerTracker, PollContext, ResyncConfig, CAMERA_RING_LEN};
use crate::xinput::XInput;
use fh6_core::{ConfigHandle, TelemetryReader};
use std::io;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Poll period: 125 Hz, above the 60 Hz minimum and fast enough that a 150 ms debounce or a
/// 400 ms hold is resolved within 8 ms.
pub const POLL_PERIOD: Duration = Duration::from_millis(8);

/// Called when the controller changed something the renderer shows.
pub type InputNotify = Arc<dyn Fn() + Send + Sync>;

/// State shared between the input thread, the render thread and the tray.
#[derive(Debug)]
pub struct InputShared {
    camera_index: AtomicU8,
    osd_open: AtomicBool,
    demo_mode: AtomicBool,
    pad_connected: AtomicBool,
    guide_available: AtomicBool,
    buttons: AtomicU16,
}

impl InputShared {
    /// Starts on `camera_index` (clamped into the six-position ring).
    pub fn new(camera_index: usize) -> Self {
        let index = u8::try_from(camera_index % CAMERA_RING_LEN).unwrap_or(3);
        Self {
            camera_index: AtomicU8::new(index),
            osd_open: AtomicBool::new(false),
            demo_mode: AtomicBool::new(false),
            pad_connected: AtomicBool::new(false),
            guide_available: AtomicBool::new(false),
            buttons: AtomicU16::new(0),
        }
    }

    /// The app's model of the game's camera (0..6; 3 = cockpit, 4 = hood).
    pub fn camera_index(&self) -> usize {
        usize::from(self.camera_index.load(Ordering::Relaxed))
    }

    /// Sets the camera index (for example from config).
    pub fn set_camera_index(&self, index: usize) {
        if let Ok(i) = u8::try_from(index % CAMERA_RING_LEN) {
            self.camera_index.store(i, Ordering::Relaxed);
        }
    }

    /// Resyncs the camera ring to `ResyncCameraIndex` (the OSD item; the LB + RB hold does the
    /// same from the controller).
    pub fn resync_camera(&self, cfg: &fh6_core::AppConfig) {
        let index = ResyncConfig::from_config(cfg).camera_index;
        self.set_camera_index(index);
        tracing::info!(camera = index, "camera resynced");
    }

    /// Whether the OSD is open.
    pub fn osd_open(&self) -> bool {
        self.osd_open.load(Ordering::Relaxed)
    }

    /// Opens or closes the OSD (tray menu, hold toggle).
    pub fn set_osd_open(&self, open: bool) {
        self.osd_open.store(open, Ordering::Relaxed);
    }

    /// Flips the OSD state and returns the new one.
    pub fn toggle_osd(&self) -> bool {
        !self.osd_open.fetch_xor(true, Ordering::Relaxed)
    }

    /// Demo mode lets RB cycle the camera without telemetry.
    pub fn set_demo_mode(&self, demo: bool) {
        self.demo_mode.store(demo, Ordering::Relaxed);
    }

    /// Whether demo mode is on.
    pub fn demo_mode(&self) -> bool {
        self.demo_mode.load(Ordering::Relaxed)
    }

    /// True if a controller answered the last poll.
    pub fn pad_connected(&self) -> bool {
        self.pad_connected.load(Ordering::Relaxed)
    }

    /// True when the loaded XInput reports the Guide button.
    pub fn guide_available(&self) -> bool {
        self.guide_available.load(Ordering::Relaxed)
    }

    /// The last A/B/X/Y-stripped buttons.
    pub fn buttons(&self) -> u16 {
        self.buttons.load(Ordering::Relaxed)
    }
}

/// Owns the input thread; dropping it stops the thread.
#[derive(Debug)]
pub struct InputPoller {
    running: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl InputPoller {
    /// Starts polling. If XInput cannot be loaded the thread logs it and exits; the overlay
    /// keeps working without a controller.
    ///
    /// # Errors
    /// Returns the I/O error if the thread cannot be spawned.
    pub fn spawn(
        shared: Arc<InputShared>,
        config: ConfigHandle,
        telemetry: TelemetryReader,
        notify: InputNotify,
    ) -> io::Result<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let thread = {
            let running = Arc::clone(&running);
            thread::Builder::new()
                .name("fh6-input".into())
                .spawn(move || run(&shared, &config, &telemetry, &*notify, &running))?
        };
        Ok(Self {
            running,
            thread: Some(thread),
        })
    }
}

impl Drop for InputPoller {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.thread().unpark();
            let _ = thread.join();
        }
    }
}

fn run(
    shared: &InputShared,
    config: &ConfigHandle,
    telemetry: &TelemetryReader,
    notify: &dyn Fn(),
    running: &AtomicBool,
) {
    let xinput = match XInput::load() {
        Ok(x) => x,
        Err(err) => {
            tracing::warn!(error = %err, "controller input disabled");
            return;
        }
    };
    shared.guide_available.store(xinput.has_guide(), Ordering::Relaxed);
    let mut hold = HoldConfig::from_config(&config.current());
    let mut tracker = InputControllerTracker::new(xinput, hold);
    let mut resync = ResyncConfig::from_config(&config.current());
    tracker.set_resync_config(resync);
    let epoch = Instant::now();

    while running.load(Ordering::Relaxed) {
        let now = Instant::now();
        let cfg = config.current();
        let new_hold = HoldConfig::from_config(&cfg);
        if new_hold != hold {
            hold = new_hold;
            tracker.set_hold_config(hold);
        }
        let new_resync = ResyncConfig::from_config(&cfg);
        if new_resync != resync {
            resync = new_resync;
            tracker.set_resync_config(resync);
        }
        let frame = telemetry.latest();
        let ctx = PollContext {
            now_ms: u64::try_from(now.duration_since(epoch).as_millis()).unwrap_or(u64::MAX),
            driving_active: frame.is_driving_active(now),
            ms_since_resume: frame.time_since_driving_resumed_ms(now),
            // Only the --demo flag lifts the camera-ring race guard; config.json's DemoMode
            // never does, so RB in a menu cannot move the camera index.
            demo_mode: shared.demo_mode(),
            settings_focused: false,
        };
        tracker.is_osd_open = shared.osd_open();
        let mut camera = shared.camera_index();
        let events = tracker.poll(&mut camera, &ctx);

        shared.pad_connected.store(tracker.is_connected(), Ordering::Relaxed);
        shared.buttons.store(tracker.buttons(), Ordering::Relaxed);
        if events.camera_changed {
            shared.set_camera_index(camera);
            tracing::debug!(camera, "camera ring advanced");
        }
        if events.camera_resynced {
            shared.set_camera_index(camera);
            tracing::info!(camera, "camera resynced by holding LB + RB");
        }
        if events.osd_toggled {
            // Flip the shared state itself, so a tray toggle since this poll began is not lost.
            let open = shared.toggle_osd();
            tracker.is_osd_open = open;
            tracing::info!(open, "OSD toggled by hold");
        }
        if events.guide_pressed && cfg.guide_button_opens_widget {
            tracing::info!("Guide pressed");
        }
        if events.any() {
            notify();
        }
        thread::park_timeout(POLL_PERIOD.saturating_sub(now.elapsed()));
    }
}
