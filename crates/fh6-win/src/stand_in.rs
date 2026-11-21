//! A plain, solid-colour, focusable window that stands in for the game during manual tests:
//! it gives the overlay a neutral background for screenshots and a foreground process for the
//! focus rule (`FH6_WheelDash --game-process fh6-stand-in.exe`).
use crate::monitor::MonitorRect;
use crate::WinError;
use std::sync::OnceLock;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateSolidBrush, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, PostQuitMessage, RegisterClassExW,
    SetForegroundWindow, TranslateMessage, MSG, WINDOW_EX_STYLE, WM_DESTROY, WM_KEYDOWN, WNDCLASSEXW, WS_POPUP,
    WS_VISIBLE,
};

const CLASS_NAME: PCWSTR = windows::core::w!("FH6WheelDashStandIn");

unsafe extern "system" fn stand_in_wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_KEYDOWN if wparam.0 == usize::from(VK_ESCAPE.0) => {
            // SAFETY: destroying our own window on its thread.
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            // SAFETY: ends this thread's message loop.
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        // SAFETY: default processing.
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// Shows a borderless window filling `rect` with colour `rgb`, titled `title`, and runs its
/// message loop until it is closed (Esc, or WM_CLOSE from outside).
///
/// # Errors
/// [`WinError`] if the window cannot be created.
pub fn run_stand_in_window(rect: MonitorRect, rgb: [u8; 3], title: &str) -> Result<(), WinError> {
    static ATOM: OnceLock<u16> = OnceLock::new();
    let colour = COLORREF(u32::from(rgb[0]) | (u32::from(rgb[1]) << 8) | (u32::from(rgb[2]) << 16));
    let atom = *ATOM.get_or_init(|| {
        // SAFETY: a solid brush owned by the window class for the process lifetime; the module
        // handle is this executable's.
        unsafe {
            let brush: HBRUSH = CreateSolidBrush(colour);
            let class = WNDCLASSEXW {
                cbSize: u32::try_from(std::mem::size_of::<WNDCLASSEXW>()).unwrap_or(80),
                lpfnWndProc: Some(stand_in_wndproc),
                hInstance: GetModuleHandleW(None).unwrap_or_default().into(),
                hbrBackground: brush,
                lpszClassName: CLASS_NAME,
                ..WNDCLASSEXW::default()
            };
            RegisterClassExW(&class)
        }
    });
    if atom == 0 {
        return Err(WinError::last("RegisterClassExW"));
    }
    let title = HSTRING::from(title);
    // SAFETY: registered class; `title` outlives the call.
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            CLASS_NAME,
            &title,
            WS_POPUP | WS_VISIBLE,
            rect.x,
            rect.y,
            i32::try_from(rect.width).unwrap_or(i32::MAX),
            i32::try_from(rect.height).unwrap_or(i32::MAX),
            None,
            None,
            Some(GetModuleHandleW(None).unwrap_or_default().into()),
            None,
        )
    }
    .map_err(|e| WinError::call("CreateWindowExW", e))?;
    // SAFETY: our own new window; Windows may refuse, in which case it just flashes.
    unsafe {
        let _ = SetForegroundWindow(hwnd);
    }
    let mut msg = MSG::default();
    // SAFETY: standard message loop on the window's thread.
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
