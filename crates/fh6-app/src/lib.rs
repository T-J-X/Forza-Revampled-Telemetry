//! fh6-app: the `FH6_WheelDash` overlay and the `FH6_SteamLauncher` wrapper.
//!
//! All Win32 and GPU `unsafe` lives in fh6-win, fh6-input and fh6-gpu; this crate wires
//! their safe APIs together.

#[cfg(windows)]
pub mod app;
pub mod args;
pub mod camera_sync;
pub mod latency;
pub mod launcher;
pub mod logging;
pub mod osd_fit;
pub mod perf_report;
pub mod process_watcher;
#[cfg(windows)]
pub mod refraction;
pub mod render_loop;
pub mod resources;
pub mod widget_server;
