//! fh6-win: the overlay window, monitor and focus tracking, the quit signal, and the game's
//! camera file patcher. All Win32 `unsafe` lives here (and in fh6-input / fh6-gpu).

#[cfg(windows)]
pub mod app_window;
pub mod camera_patcher;
#[cfg(windows)]
pub mod capture;
pub mod car_brands;
#[cfg(windows)]
pub mod foreground;
#[cfg(windows)]
pub mod monitor;
#[cfg(windows)]
pub mod overlay;
#[cfg(windows)]
pub mod quit_signal;
#[cfg(windows)]
pub mod system;

#[cfg(windows)]
pub use foreground::{ForegroundReader, ForegroundWatcher};
#[cfg(windows)]
pub use monitor::MonitorRect;
#[cfg(windows)]
pub use overlay::{OverlayHandle, OverlayWindow};
#[cfg(windows)]
pub use quit_signal::QuitSignal;

/// A failed Win32 call.
#[derive(Debug, thiserror::Error)]
#[error("{call} failed: {message}")]
pub struct WinError {
    call: &'static str,
    message: String,
}

impl WinError {
    /// Wraps an error returned by a `windows` crate call.
    #[cfg(windows)]
    pub(crate) fn call(call: &'static str, err: windows::core::Error) -> Self {
        Self {
            call,
            message: err.message(),
        }
    }

    /// Captures `GetLastError` after a call that signals failure by its return value.
    #[cfg(windows)]
    pub(crate) fn last(call: &'static str) -> Self {
        Self::call(call, windows::core::Error::from_thread())
    }

    /// The Win32 function that failed.
    pub fn function(&self) -> &'static str {
        self.call
    }
}
