//! Monitor geometry and per-monitor DPI awareness.
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};

/// A monitor's rectangle in physical (unscaled) desktop pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorRect {
    /// Left edge in virtual-desktop pixels.
    pub x: i32,
    /// Top edge in virtual-desktop pixels.
    pub y: i32,
    /// Width in pixels (at least 1).
    pub width: u32,
    /// Height in pixels (at least 1).
    pub height: u32,
}

/// Makes the process per-monitor-v2 DPI aware, so window sizes and positions are physical
/// pixels (3840x2160 on the 4K TV at 200% scaling). Call before creating any window.
///
/// Returns false if the awareness was already set (for example by a manifest), which is fine.
pub fn enable_per_monitor_dpi_awareness() -> bool {
    // SAFETY: plain Win32 call with a predefined pseudo-handle constant.
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) }.is_ok()
}

fn rect_of(monitor: HMONITOR) -> Option<MonitorRect> {
    let mut info = MONITORINFO {
        cbSize: u32::try_from(std::mem::size_of::<MONITORINFO>()).unwrap_or(40),
        ..MONITORINFO::default()
    };
    // SAFETY: `info` is a correctly sized, writable MONITORINFO; an invalid HMONITOR makes the
    // call fail rather than misbehave.
    let ok = unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool();
    if !ok {
        return None;
    }
    let r = info.rcMonitor;
    Some(MonitorRect {
        x: r.left,
        y: r.top,
        width: u32::try_from(r.right - r.left).unwrap_or(1).max(1),
        height: u32::try_from(r.bottom - r.top).unwrap_or(1).max(1),
    })
}

/// The primary monitor's rectangle.
pub fn primary_monitor_rect() -> Option<MonitorRect> {
    // SAFETY: MonitorFromPoint has no preconditions; DEFAULTTOPRIMARY always returns a monitor.
    let monitor = unsafe { MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) };
    rect_of(monitor)
}

/// The rectangle of the monitor showing most of the window `hwnd` (a raw `HWND` value).
pub fn monitor_rect_for_window(hwnd: isize) -> Option<MonitorRect> {
    if hwnd == 0 {
        return None;
    }
    // SAFETY: MonitorFromWindow accepts any HWND value; a stale one yields the nearest monitor.
    let monitor = unsafe { MonitorFromWindow(HWND(hwnd as *mut core::ffi::c_void), MONITOR_DEFAULTTONEAREST) };
    rect_of(monitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_monitor_has_size() {
        let r = primary_monitor_rect().expect("a desktop session has a primary monitor");
        assert!(r.width >= 640 && r.height >= 480, "{r:?}");
    }
}
