//! A named event that tells a running FH6_WheelDash to quit (used by FH6_SteamLauncher).
use crate::WinError;
use std::time::Duration;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{CreateEventW, OpenEventW, SetEvent, WaitForSingleObject, EVENT_MODIFY_STATE};

/// Session-local, so it never crosses user sessions.
const EVENT_NAME: PCWSTR = w!("Local\\FH6_WheelDash_Quit");

/// The overlay's end of the quit signal. Not `Send`: create it on the thread that waits.
#[derive(Debug)]
pub struct QuitSignal {
    handle: HANDLE,
}

impl QuitSignal {
    /// Creates (or opens) the auto-reset quit event.
    ///
    /// # Errors
    /// [`WinError`] if the event cannot be created.
    pub fn create() -> Result<Self, WinError> {
        // SAFETY: default security, auto-reset, initially unsignalled, static name.
        let handle = unsafe { CreateEventW(None, false, false, EVENT_NAME) }.map_err(|e| WinError::call("CreateEventW", e))?;
        Ok(Self { handle })
    }

    /// Waits up to `timeout` for a quit request; true if one arrived.
    pub fn wait(&self, timeout: Duration) -> bool {
        let ms = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
        // SAFETY: `handle` is a live event handle owned by `self`.
        unsafe { WaitForSingleObject(self.handle, ms) == WAIT_OBJECT_0 }
    }

    /// Signals a running overlay to quit. Returns false if no overlay is running.
    pub fn signal_running_overlay() -> bool {
        // SAFETY: opening a named event with modify rights only; closed below.
        let Ok(handle) = (unsafe { OpenEventW(EVENT_MODIFY_STATE, false, EVENT_NAME) }) else {
            return false;
        };
        // SAFETY: `handle` was just opened and is closed exactly once.
        unsafe {
            let ok = SetEvent(handle).is_ok();
            let _ = CloseHandle(handle);
            ok
        }
    }
}

impl Drop for QuitSignal {
    fn drop(&mut self) {
        // SAFETY: `handle` came from CreateEventW and is closed exactly once.
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_reaches_waiter() {
        let signal = QuitSignal::create().expect("create event");
        assert!(!signal.wait(Duration::ZERO));
        assert!(QuitSignal::signal_running_overlay());
        assert!(signal.wait(Duration::from_millis(100)));
        assert!(!signal.wait(Duration::ZERO), "auto-reset");
    }
}
