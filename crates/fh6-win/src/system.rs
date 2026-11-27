//! Small system services: the single-instance mutex, process enumeration, opening files or
//! URLs with the shell, DWM composition timing and the render thread's scheduling class.
use crate::WinError;
use std::time::Duration;
use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND};
use windows::Win32::Graphics::Dwm::{DwmGetCompositionTimingInfo, DWM_TIMING_INFO};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows::Win32::System::Threading::{
    AvRevertMmThreadCharacteristics, AvSetMmThreadCharacteristicsW, CreateMutexW, GetCurrentThread,
    SetThreadInformation, SetThreadPriority, ThreadPowerThrottling, THREAD_POWER_THROTTLING_CURRENT_VERSION,
    THREAD_POWER_THROTTLING_EXECUTION_SPEED, THREAD_POWER_THROTTLING_STATE, THREAD_PRIORITY_ABOVE_NORMAL,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// Holds the named mutex that makes FH6_WheelDash single-instance.
#[derive(Debug)]
pub struct SingleInstance {
    handle: HANDLE,
}

/// Outcome of claiming the single-instance mutex.
#[derive(Debug)]
pub enum InstanceClaim {
    /// This is the only instance; keep the guard alive for the process lifetime.
    First(SingleInstance),
    /// Another instance is already running.
    AlreadyRunning,
}

impl SingleInstance {
    /// Claims `Local\FH6_WheelDash_SingleInstance`.
    ///
    /// # Errors
    /// [`WinError`] if the mutex cannot be created at all.
    pub fn claim() -> Result<InstanceClaim, WinError> {
        // SAFETY: default security, not initially owned, static name.
        let handle = unsafe { CreateMutexW(None, false, w!("Local\\FH6_WheelDash_SingleInstance")) }
            .map_err(|e| WinError::call("CreateMutexW", e))?;
        // SAFETY: read immediately after the call whose status it reports.
        let existed = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        if existed {
            // SAFETY: our handle to the existing mutex; closed once.
            unsafe {
                let _ = CloseHandle(handle);
            }
            return Ok(InstanceClaim::AlreadyRunning);
        }
        Ok(InstanceClaim::First(Self { handle }))
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        // SAFETY: the handle came from CreateMutexW and is closed exactly once.
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

/// Lower-cased executable names of every running process.
pub fn running_process_names() -> Vec<String> {
    // SAFETY: a process snapshot that is closed below; `entry` has its dwSize set as required.
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return Vec::new();
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: u32::try_from(std::mem::size_of::<PROCESSENTRY32W>()).unwrap_or(0),
            ..PROCESSENTRY32W::default()
        };
        let mut names = Vec::new();
        let mut ok = Process32FirstW(snapshot, &mut entry).is_ok();
        while ok {
            let len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
            if !name.is_empty() {
                names.push(name.to_ascii_lowercase());
            }
            ok = Process32NextW(snapshot, &mut entry).is_ok();
        }
        let _ = CloseHandle(snapshot);
        names
    }
}

/// True if a process with this executable name (case-insensitive) is running.
pub fn is_process_running(exe_name: &str) -> bool {
    let wanted = exe_name.to_ascii_lowercase();
    running_process_names().contains(&wanted)
}

/// Opens a file, folder or URL with its default handler.
///
/// # Errors
/// [`WinError`] if the shell reports failure (a return value of 32 or less).
pub fn shell_open(target: &str) -> Result<(), WinError> {
    let target = HSTRING::from(target);
    // SAFETY: all strings are valid NUL-terminated wide strings that outlive the call.
    let result = unsafe {
        ShellExecuteW(
            None::<HWND>,
            w!("open"),
            &target,
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize > 32 {
        Ok(())
    } else {
        Err(WinError::last("ShellExecuteW"))
    }
}

/// DWM composition timing: how often the desktop refreshes and when the next vblank is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompositionTiming {
    /// One refresh interval.
    pub refresh_period: Duration,
    /// Time from now until the next vblank.
    pub until_next_vblank: Duration,
}

/// Reads DWM's composition timing, or `None` if DWM does not report it.
pub fn composition_timing() -> Option<CompositionTiming> {
    let mut info = DWM_TIMING_INFO {
        cbSize: u32::try_from(std::mem::size_of::<DWM_TIMING_INFO>()).unwrap_or(0),
        ..DWM_TIMING_INFO::default()
    };
    let (mut now, mut freq) = (0i64, 0i64);
    // SAFETY: `info` is sized correctly; a null HWND asks for the desktop timing (required on
    // Windows 8.1 and later); the QPC out-params are valid locals.
    unsafe {
        DwmGetCompositionTimingInfo(HWND::default(), &mut info).ok()?;
        QueryPerformanceCounter(&mut now).ok()?;
        QueryPerformanceFrequency(&mut freq).ok()?;
    }
    let period = info.qpcRefreshPeriod;
    if period == 0 || freq <= 0 {
        return None;
    }
    let now = u64::try_from(now).ok()?;
    // DWM may report the upcoming vblank (in the future) or a past one.
    let until_next = if info.qpcVBlank > now {
        (info.qpcVBlank - now) % period
    } else {
        period - (now - info.qpcVBlank) % period
    };
    let to_duration = |ticks: u64| Duration::from_secs_f64(ticks as f64 / freq as f64);
    Some(CompositionTiming {
        refresh_period: to_duration(period),
        until_next_vblank: to_duration(until_next),
    })
}

/// How the calling thread was raised by [`raise_frame_thread`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameThreadClass {
    /// Registered with the Multimedia Class Scheduler Service as a "Games" task.
    Mmcss,
    /// MMCSS refused (service off, or no task slot); the thread runs at above-normal priority.
    AboveNormal,
    /// Neither worked; the thread keeps its normal priority.
    Normal,
}

/// Keeps the calling thread in its raised scheduling class; dropping it reverts the class.
/// It belongs to the thread that made it (it is neither `Send` nor `Sync`).
#[derive(Debug)]
pub struct FrameThreadGuard {
    mmcss: Option<HANDLE>,
    class: FrameThreadClass,
    full_speed: bool,
    /// The MMCSS handle is per thread: the guard must be dropped where it was made.
    _not_send: std::marker::PhantomData<*const ()>,
}

impl FrameThreadGuard {
    /// The class the thread runs in.
    #[must_use]
    pub fn class(&self) -> FrameThreadClass {
        self.class
    }

    /// True when the thread is opted out of Windows power throttling (EcoQoS), so it is not
    /// moved to efficiency cores or a low clock while another window (the game) has the focus.
    #[must_use]
    pub fn full_speed(&self) -> bool {
        self.full_speed
    }
}

impl Drop for FrameThreadGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.mmcss.take() {
            // SAFETY: `handle` came from `AvSetMmThreadCharacteristicsW` on this thread (the guard
            // is not `Send`), and it is reverted once (`take`).
            if let Err(err) = unsafe { AvRevertMmThreadCharacteristics(handle) } {
                tracing::debug!(error = %err.message(), "MMCSS revert failed");
            }
        }
    }
}

/// Raises the calling thread for frame work that must not be preempted or slowed mid-frame:
/// - an MMCSS "Games" task (the class Windows gives game render threads), or above-normal
///   priority when MMCSS is unavailable;
/// - out of power throttling. The overlay never has the focus (the game does), so Windows treats
///   it as background work and may run it on an efficiency core at a low clock (EcoQoS).
///
/// The frame thread works for well under 1 ms per refresh and then waits, so it takes no time
/// from the game; it only stops the frame from being parked or slowed between its wake and its
/// `present`.
#[must_use = "dropping the guard reverts the thread's class at once"]
pub fn raise_frame_thread() -> FrameThreadGuard {
    let full_speed = opt_out_of_power_throttling();
    let mut index = 0u32;
    // SAFETY: the task name is a static wide string; `index` is a valid out-param (0 asks for a new
    // task). The handle is reverted by the guard on this thread.
    match unsafe { AvSetMmThreadCharacteristicsW(w!("Games"), &mut index) } {
        Ok(handle) if !handle.is_invalid() => {
            return FrameThreadGuard {
                mmcss: Some(handle),
                class: FrameThreadClass::Mmcss,
                full_speed,
                _not_send: std::marker::PhantomData,
            }
        }
        Ok(_) => tracing::debug!("MMCSS returned no handle"),
        Err(err) => tracing::debug!(error = %err.message(), "MMCSS refused the frame thread"),
    }
    // SAFETY: `GetCurrentThread` is a pseudo-handle that is always valid for the calling thread.
    let class = match unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_ABOVE_NORMAL) } {
        Ok(()) => FrameThreadClass::AboveNormal,
        Err(_) => FrameThreadClass::Normal,
    };
    FrameThreadGuard {
        mmcss: None,
        class,
        full_speed,
        _not_send: std::marker::PhantomData,
    }
}

/// Turns execution-speed throttling off for the calling thread (control the policy, state off:
/// always full speed). False when Windows refuses (before Windows 11 the thread class is unknown).
fn opt_out_of_power_throttling() -> bool {
    let state = THREAD_POWER_THROTTLING_STATE {
        Version: THREAD_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: THREAD_POWER_THROTTLING_EXECUTION_SPEED,
        StateMask: 0,
    };
    let size = u32::try_from(std::mem::size_of::<THREAD_POWER_THROTTLING_STATE>()).unwrap_or(0);
    // SAFETY: `state` is a valid THREAD_POWER_THROTTLING_STATE that outlives the call, `size` is
    // its size, and the pseudo-handle names the calling thread.
    let result = unsafe {
        SetThreadInformation(
            GetCurrentThread(),
            ThreadPowerThrottling,
            std::ptr::from_ref(&state).cast(),
            size,
        )
    };
    if let Err(err) = &result {
        tracing::debug!(error = %err.message(), "power throttling opt-out refused");
    }
    result.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_test_process_is_listed() {
        let names = running_process_names();
        assert!(!names.is_empty());
        assert!(names.iter().all(|n| *n == n.to_ascii_lowercase()));
        assert!(is_process_running("explorer.exe") || is_process_running("svchost.exe"));
        assert!(!is_process_running("definitely-not-a-real-process-fh6.exe"));
    }

    #[test]
    fn the_frame_thread_is_raised_and_reverted_on_its_own_thread() {
        let class = std::thread::spawn(|| {
            let guard = raise_frame_thread();
            let class = (guard.class(), guard.full_speed());
            drop(guard);
            class
        })
        .join()
        .expect("the test thread ran");
        // MMCSS runs on every desktop Windows; a stripped-down one still gets the priority bump.
        assert_ne!(class.0, FrameThreadClass::Normal);
        // Windows 10 1709 and later accept the throttling opt-out.
        assert!(class.1, "the frame thread is not throttled");
    }

    #[test]
    fn dwm_reports_a_sane_refresh() {
        let timing = composition_timing().expect("DWM is always on in Windows 10/11");
        let hz = 1.0 / timing.refresh_period.as_secs_f64();
        assert!((23.0..=500.0).contains(&hz), "refresh {hz} Hz");
        assert!(timing.until_next_vblank <= timing.refresh_period);
    }
}
