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
pub mod keyboard;
#[cfg(windows)]
pub mod monitor;
#[cfg(windows)]
pub mod overlay;
#[cfg(windows)]
pub mod quit_signal;
#[cfg(windows)]
pub mod stand_in;
#[cfg(windows)]
pub mod system;
#[cfg(windows)]
pub mod wgc;

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

/// Registers a window class once per process. A failed registration is not cached, and a
/// class that already exists (another thread registered it first) counts as success.
/// Returns `true` only when this call registered `class`, so the caller knows whether the
/// resources it put in `class` (a background brush) are now owned by the class.
///
/// # Safety
/// `class` must be fully initialised: a valid window procedure and `'static` strings.
#[cfg(windows)]
pub(crate) unsafe fn register_class_once(
    done: &std::sync::OnceLock<()>,
    class: &windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW,
) -> Result<bool, WinError> {
    if done.get().is_some() {
        return Ok(false);
    }
    // SAFETY: the caller guarantees `class` is valid for registration.
    let atom = unsafe { windows::Win32::UI::WindowsAndMessaging::RegisterClassExW(class) };
    let registered = if atom == 0 {
        let err = windows::core::Error::from_thread();
        if err.code() != windows::Win32::Foundation::ERROR_CLASS_ALREADY_EXISTS.to_hresult() {
            return Err(WinError::call("RegisterClassExW", err));
        }
        false
    } else {
        true
    };
    let _ = done.set(());
    Ok(registered)
}
