//! The overlay window: transparent, click-through, topmost, never activated (ADR 0001).
//!
//! The window has no GDI surface (`WS_EX_NOREDIRECTIONBITMAP`); its only content is the
//! DirectComposition visual that wgpu creates for it. `WS_EX_LAYERED | WS_EX_TRANSPARENT`
//! make every mouse event fall through to the game, and `WS_EX_NOACTIVATE` keeps focus there.
use crate::monitor::MonitorRect;
use crate::WinError;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle,
};
use std::num::NonZeroIsize;
use std::sync::{Arc, OnceLock};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, PostQuitMessage, SetLayeredWindowAttributes, SetWindowPos,
    ShowWindow, HTTRANSPARENT, HWND_TOPMOST, LWA_ALPHA, MA_NOACTIVATE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SW_SHOWNOACTIVATE, WM_CLOSE, WM_MOUSEACTIVATE, WM_NCHITTEST, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

const CLASS_NAME: PCWSTR = w!("FH6WheelDashOverlay");

unsafe extern "system" fn overlay_wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        // Belt and braces with WS_EX_TRANSPARENT: never claim a hit test.
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        // Only the app may destroy this window (after the GPU surface is gone). A WM_CLOSE from
        // outside (taskkill) becomes a clean app quit instead of destroying it under a live
        // swapchain: WM_QUIT ends the message loop and the normal shutdown order runs.
        WM_CLOSE => {
            // SAFETY: posts WM_QUIT to this (the window's own) thread's queue.
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        // SAFETY: forwarding the unmodified message to the default procedure.
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn register_class() -> Result<(), WinError> {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    // SAFETY: GetModuleHandleW(None) returns this executable's module, valid for the
    // process lifetime.
    let instance = unsafe { GetModuleHandleW(None) }.unwrap_or_default();
    let class = WNDCLASSEXW {
        cbSize: u32::try_from(std::mem::size_of::<WNDCLASSEXW>()).unwrap_or(80),
        lpfnWndProc: Some(overlay_wndproc),
        hInstance: instance.into(),
        lpszClassName: CLASS_NAME,
        ..WNDCLASSEXW::default()
    };
    // SAFETY: `class` is fully initialised with a valid window procedure and static strings.
    unsafe { crate::register_class_once(&REGISTERED, &class) }
}

/// Keeps the window alive for surfaces: see [`OverlayHandle`].
#[derive(Debug)]
struct HandleToken {
    hwnd: NonZeroIsize,
}

/// A `Send + Sync` handle to the overlay window, for creating the GPU surface on the render
/// thread. While any handle exists, dropping the [`OverlayWindow`] leaves the `HWND` alive
/// (it is reclaimed at process exit) instead of destroying it, so a surface can never outlive
/// its window. Drop every handle (join the render thread) before dropping the window.
#[derive(Debug, Clone)]
pub struct OverlayHandle {
    token: Arc<HandleToken>,
}

impl HasWindowHandle for OverlayHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let raw = RawWindowHandle::Win32(Win32WindowHandle::new(self.token.hwnd));
        // SAFETY: the HWND is a live window: OverlayWindow never destroys it while this
        // handle (a clone of its token) exists, and the borrow is tied to `&self`.
        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}

impl HasDisplayHandle for OverlayHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::windows())
    }
}

/// The overlay window. Owned by the thread that created it (it runs that thread's message
/// loop); it is not `Send`.
#[derive(Debug)]
pub struct OverlayWindow {
    hwnd: HWND,
    token: Arc<HandleToken>,
}

impl OverlayWindow {
    /// Creates the window over `rect` and shows it without activating it.
    ///
    /// # Errors
    /// [`WinError`] if the class cannot be registered or the window cannot be created.
    pub fn create(rect: MonitorRect) -> Result<Self, WinError> {
        register_class()?;
        let ex_style = WS_EX_NOREDIRECTIONBITMAP
            | WS_EX_LAYERED
            | WS_EX_TRANSPARENT
            | WS_EX_TOPMOST
            | WS_EX_TOOLWINDOW
            | WS_EX_NOACTIVATE;
        let width = i32::try_from(rect.width).unwrap_or(i32::MAX);
        let height = i32::try_from(rect.height).unwrap_or(i32::MAX);
        // SAFETY: the class is registered above; the title is a static string; no parent,
        // menu or creation parameter is passed.
        let hwnd = unsafe {
            let instance = GetModuleHandleW(None).unwrap_or_default();
            CreateWindowExW(
                ex_style,
                CLASS_NAME,
                w!("FH6 WheelDash"),
                WS_POPUP,
                rect.x,
                rect.y,
                width,
                height,
                None,
                None,
                Some(instance.into()),
                None,
            )
        }
        .map_err(|e| WinError::call("CreateWindowExW", e))?;
        let Some(raw) = NonZeroIsize::new(hwnd.0 as isize) else {
            return Err(WinError::last("CreateWindowExW"));
        };
        let window = Self {
            hwnd,
            token: Arc::new(HandleToken { hwnd: raw }),
        };
        // A layered window is invisible until its attributes are set; 255 = fully opaque
        // layer, with per-pixel alpha coming from the composition swapchain.
        // SAFETY: `hwnd` was just created on this thread.
        unsafe { SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA) }
            .map_err(|e| WinError::call("SetLayeredWindowAttributes", e))?;
        // SAFETY: valid window; SW_SHOWNOACTIVATE never takes focus from the game.
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        Ok(window)
    }

    /// The raw `HWND` value, for creating the GPU surface.
    pub fn hwnd_raw(&self) -> isize {
        self.hwnd.0 as isize
    }

    /// A `Send + Sync` handle for creating the GPU surface. See [`OverlayHandle`].
    pub fn surface_handle(&self) -> OverlayHandle {
        OverlayHandle {
            token: Arc::clone(&self.token),
        }
    }

    /// Moves and resizes the window to `rect` (another monitor, a resolution change).
    ///
    /// # Errors
    /// [`WinError`] if `SetWindowPos` fails.
    pub fn set_bounds(&self, rect: MonitorRect) -> Result<(), WinError> {
        // SAFETY: valid window owned by this thread.
        unsafe {
            SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                rect.x,
                rect.y,
                i32::try_from(rect.width).unwrap_or(i32::MAX),
                i32::try_from(rect.height).unwrap_or(i32::MAX),
                SWP_NOACTIVATE,
            )
        }
        .map_err(|e| WinError::call("SetWindowPos", e))
    }

    /// Puts the window back on top of the topmost band (after the game raised itself).
    pub fn reassert_topmost(&self) {
        // SAFETY: valid window owned by this thread; position and size are unchanged.
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}

impl Drop for OverlayWindow {
    fn drop(&mut self) {
        if Arc::strong_count(&self.token) > 1 {
            // A surface may still present to this HWND; leaving it alive until process exit
            // is safe, destroying it is not.
            tracing::warn!("overlay window dropped while a surface handle is alive; not destroying it");
            return;
        }
        // SAFETY: the window was created by this object on this thread (OverlayWindow is
        // not Send), no surface handle remains, and it is destroyed exactly once.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}
