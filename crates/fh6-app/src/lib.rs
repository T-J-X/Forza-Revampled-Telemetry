//! fh6-app: the `FH6_WheelDash` overlay and the `FH6_SteamLauncher` wrapper.
//!
//! All `unsafe` (Win32 and XInput) lives in fh6-win and fh6-input; this crate wires their safe
//! APIs together and forbids `unsafe` itself.
#![forbid(unsafe_code)]

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
