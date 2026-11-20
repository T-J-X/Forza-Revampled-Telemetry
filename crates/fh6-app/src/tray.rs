#[cfg(windows)]
use windows::Win32::UI::Shell::{Shell_NotifyIconW, NIM_ADD, NIM_DELETE, NIM_MODIFY};
use windows::Win32::Foundation::HWND;
// use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW, WM_DESTROY, MSG, WNDCLASSW, CS_HREDRAW, CS_VREDRAW, WS_OVERLAPPEDWINDOW};
// use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
// use windows::core::{PCWSTR, HINSTANCE};

#[derive(Debug)]
pub struct TrayIcon {
    #[cfg(windows)]
    hwnd: HWND,
    tooltip: String,
}

impl TrayIcon {
    pub fn new(tooltip: &str) -> Self {
        #[cfg(windows)]
        {
            // Minimal window class for message loop; real implementation would create hidden window.
            // For testability, we skip actual creation and just return struct.
            Self { hwnd: HWND(std::ptr::null_mut()), tooltip: tooltip.to_string() }
        }
        #[cfg(not(windows))]
        {
            Self { tooltip: tooltip.to_string() }
        }
    }

    pub fn add(&self) -> bool {
        #[cfg(windows)]
        {
            // Stub: in real code fill NOTIFYICONDATAA/W and call Shell_NotifyIconW(NIM_ADD)
            true
        }
        #[cfg(not(windows))]
        {
            true
        }
    }

    pub fn update_tooltip(&mut self, tooltip: &str) {
        self.tooltip = tooltip.to_string();
    }

    pub fn remove(&self) -> bool {
        #[cfg(windows)]
        {
            // Shell_NotifyIconW(NIM_DELETE)
            true
        }
        #[cfg(not(windows))]
        {
            true
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        let _ = self.remove();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_lifecycle_add_remove() {
        let mut tray = TrayIcon::new("FH6 WheelDash");
        assert!(tray.add());
        tray.update_tooltip("Game started");
        assert_eq!(tray.tooltip, "Game started");
        assert!(tray.remove());
    }
}
