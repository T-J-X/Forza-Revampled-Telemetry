//! The app's hidden message window: the tray icon and menu, timers, and the message loop.
//!
//! The window is never shown. It owns the `Shell_NotifyIconW` icon (re-added when Explorer
//! restarts), dispatches timers and tray commands to one handler closure, and runs the
//! thread's message loop (which also delivers the foreground WinEvent hook).
use crate::WinError;
use std::cell::{Cell, RefCell};
use std::sync::OnceLock;
use std::time::Duration;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow, DispatchMessageW,
    GetCursorPos, GetMessageW, GetWindowLongPtrW, LoadIconW, PostMessageW, PostQuitMessage, RegisterWindowMessageW,
    SetForegroundWindow, SetTimer, SetWindowLongPtrW, TrackPopupMenu, TranslateMessage, GWLP_USERDATA, IDI_APPLICATION,
    MF_CHECKED, MF_SEPARATOR, MF_STRING, MSG, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, WINDOW_EX_STYLE, WM_APP,
    WM_CLOSE, WM_CONTEXTMENU, WM_DESTROY, WM_LBUTTONDBLCLK, WM_NCDESTROY, WM_NULL, WM_RBUTTONUP, WM_TIMER, WNDCLASSEXW,
    WS_OVERLAPPED,
};

const CLASS_NAME: PCWSTR = w!("FH6WheelDashApp");
const WM_TRAY: u32 = WM_APP + 1;
const TRAY_ID: u32 = 1;

/// A tray menu choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    /// Show or hide the overlay.
    ToggleOverlay,
    /// Open the configurator.
    OpenConfigurator,
    /// Open or close the OSD.
    OpenOsd,
    /// Quit the app.
    Quit,
}

impl TrayCommand {
    const ALL: [TrayCommand; 4] = [Self::ToggleOverlay, Self::OpenConfigurator, Self::OpenOsd, Self::Quit];

    fn id(self) -> usize {
        match self {
            Self::ToggleOverlay => 1,
            Self::OpenConfigurator => 2,
            Self::OpenOsd => 3,
            Self::Quit => 4,
        }
    }

    fn from_id(id: usize) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.id() == id)
    }
}

/// Something the handler should react to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    /// A timer set with [`WindowCtl::set_timer`] fired.
    Timer(usize),
    /// A tray menu item (or a double-click on the icon, which toggles the overlay).
    Tray(TrayCommand),
    /// Something asked the app to close (the Steam launcher, `CloseHandle`).
    CloseRequested,
}

/// What to do after handling an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    /// Keep running.
    Continue,
    /// Destroy the window and end the message loop.
    Quit,
}

/// The event handler. Runs on the message-loop thread.
pub type AppHandler = Box<dyn for<'w> FnMut(&WindowCtl<'w>, AppEvent) -> Flow>;

struct WindowState {
    handler: RefCell<AppHandler>,
    tooltip: Cell<[u16; 128]>,
    overlay_checked: Cell<bool>,
    taskbar_created: u32,
    destroyed: Cell<bool>,
}

fn encode_tip(text: &str) -> [u16; 128] {
    let mut tip = [0u16; 128];
    for (slot, unit) in tip.iter_mut().take(127).zip(text.encode_utf16()) {
        *slot = unit;
    }
    tip
}

fn notify_icon(hwnd: HWND, tip: [u16; 128], message: windows::Win32::UI::Shell::NOTIFY_ICON_MESSAGE) -> bool {
    // SAFETY: IDI_APPLICATION is a predefined system icon; no module handle is needed.
    let icon = unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap_or_default();
    let data = NOTIFYICONDATAW {
        cbSize: u32::try_from(std::mem::size_of::<NOTIFYICONDATAW>()).unwrap_or(0),
        hWnd: hwnd,
        uID: TRAY_ID,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAY,
        hIcon: icon,
        szTip: tip,
        ..NOTIFYICONDATAW::default()
    };
    // SAFETY: `data` is fully initialised with the correct size and a live window handle.
    unsafe { Shell_NotifyIconW(message, &data) }.as_bool()
}

fn remove_icon(hwnd: HWND) {
    let data = NOTIFYICONDATAW {
        cbSize: u32::try_from(std::mem::size_of::<NOTIFYICONDATAW>()).unwrap_or(0),
        hWnd: hwnd,
        uID: TRAY_ID,
        ..NOTIFYICONDATAW::default()
    };
    // SAFETY: deleting by (hWnd, uID); harmless if the icon is already gone.
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &data);
    }
}

/// Main-thread control over the app window, passed to the handler. It borrows the window's
/// state, so it cannot outlive the [`AppWindow`] that owns it.
#[derive(Clone, Copy)]
pub struct WindowCtl<'a> {
    hwnd: HWND,
    state: &'a WindowState,
}

impl std::fmt::Debug for WindowCtl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowCtl")
            .field("hwnd", &self.hwnd)
            .finish_non_exhaustive()
    }
}

impl WindowCtl<'_> {
    fn state(&self) -> Option<&WindowState> {
        Some(self.state)
    }

    /// Starts (or restarts) timer `id` with the given period.
    pub fn set_timer(&self, id: usize, period: Duration) {
        let ms = u32::try_from(period.as_millis()).unwrap_or(u32::MAX).max(1);
        // SAFETY: a window-owned timer on a live window; WM_TIMER goes to its wndproc.
        unsafe {
            SetTimer(Some(self.hwnd), id, ms, None);
        }
    }

    /// Updates the tray tooltip (truncated to 127 UTF-16 units).
    pub fn set_tooltip(&self, text: &str) {
        let tip = encode_tip(text);
        if let Some(state) = self.state() {
            state.tooltip.set(tip);
        }
        notify_icon(self.hwnd, tip, NIM_MODIFY);
    }

    /// Sets the check mark on the "Overlay visible" menu item.
    pub fn set_overlay_checked(&self, checked: bool) {
        if let Some(state) = self.state() {
            state.overlay_checked.set(checked);
        }
    }

    /// Destroys the window, which ends the message loop.
    pub fn quit(&self) {
        // SAFETY: called on the window's own thread (WindowCtl is not Send).
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/// Posts a close request to the app window from any thread.
#[derive(Debug, Clone, Copy)]
pub struct CloseHandle {
    hwnd: isize,
}

impl CloseHandle {
    /// Asks the app to close (delivered as [`AppEvent::CloseRequested`]).
    pub fn request_close(&self) {
        // SAFETY: PostMessageW is thread-safe; a stale HWND just makes the post fail.
        unsafe {
            let _ = PostMessageW(
                Some(HWND(self.hwnd as *mut core::ffi::c_void)),
                WM_CLOSE,
                WPARAM(0),
                LPARAM(0),
            );
        }
    }
}

fn show_tray_menu(hwnd: HWND, state: &WindowState) -> Option<TrayCommand> {
    // SAFETY: the menu is created, shown and destroyed here on the window's thread; the
    // labels are static strings.
    unsafe {
        let menu = CreatePopupMenu().ok()?;
        let check = if state.overlay_checked.get() {
            MF_CHECKED
        } else {
            MF_STRING
        };
        let _ = AppendMenuW(
            menu,
            MF_STRING | check,
            TrayCommand::ToggleOverlay.id(),
            w!("Overlay visible"),
        );
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            TrayCommand::OpenConfigurator.id(),
            w!("Open configurator"),
        );
        let _ = AppendMenuW(menu, MF_STRING, TrayCommand::OpenOsd.id(), w!("Open OSD"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
        let _ = AppendMenuW(menu, MF_STRING, TrayCommand::Quit.id(), w!("Quit FH6 WheelDash"));
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        // Required so the menu closes when the user clicks elsewhere.
        let _ = SetForegroundWindow(hwnd);
        let picked = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY,
            pt.x,
            pt.y,
            None,
            hwnd,
            None,
        );
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(menu);
        TrayCommand::from_id(usize::try_from(picked.0).unwrap_or(0))
    }
}

fn dispatch(hwnd: HWND, state: &WindowState, event: AppEvent) {
    let ctl = WindowCtl { hwnd, state };
    // A re-entrant message (for example during a modal loop) is skipped rather than aliased.
    let flow = match state.handler.try_borrow_mut() {
        Ok(mut handler) => handler(&ctl, event),
        Err(_) => Flow::Continue,
    };
    if flow == Flow::Quit && !state.destroyed.get() {
        ctl.quit();
    }
}

unsafe extern "system" fn app_wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: see WindowCtl::state; the pointer is 0 until AppWindow::create installs it and
    // is cleared on WM_NCDESTROY.
    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const WindowState;
    // SAFETY: a non-null pointer refers to the live WindowState owned by AppWindow.
    let Some(state) = (unsafe { ptr.as_ref() }) else {
        // SAFETY: default processing for messages before the state is installed.
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    };
    match msg {
        WM_TIMER => dispatch(hwnd, state, AppEvent::Timer(wparam.0)),
        WM_TRAY => match u32::try_from(lparam.0 & 0xFFFF).unwrap_or(0) {
            WM_RBUTTONUP | WM_CONTEXTMENU => {
                if let Some(cmd) = show_tray_menu(hwnd, state) {
                    dispatch(hwnd, state, AppEvent::Tray(cmd));
                }
            }
            WM_LBUTTONDBLCLK => dispatch(hwnd, state, AppEvent::Tray(TrayCommand::ToggleOverlay)),
            _ => {}
        },
        WM_CLOSE => {
            dispatch(hwnd, state, AppEvent::CloseRequested);
            if !state.destroyed.get() {
                WindowCtl { hwnd, state }.quit();
            }
        }
        WM_DESTROY => {
            state.destroyed.set(true);
            remove_icon(hwnd);
            // SAFETY: ends this thread's message loop.
            unsafe { PostQuitMessage(0) };
        }
        WM_NCDESTROY => {
            // SAFETY: detach the state so no later message can reach it.
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };
        }
        m if m == state.taskbar_created && m != 0 => {
            // Explorer restarted: the tray icon must be added again.
            notify_icon(hwnd, state.tooltip.get(), NIM_ADD);
        }
        // SAFETY: default processing for everything else.
        _ => return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
    LRESULT(0)
}

fn register_class() -> Result<(), WinError> {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    // SAFETY: this executable's module handle, valid for the process lifetime.
    let instance = unsafe { GetModuleHandleW(None) }.unwrap_or_default();
    let class = WNDCLASSEXW {
        cbSize: u32::try_from(std::mem::size_of::<WNDCLASSEXW>()).unwrap_or(80),
        lpfnWndProc: Some(app_wndproc),
        hInstance: instance.into(),
        lpszClassName: CLASS_NAME,
        ..WNDCLASSEXW::default()
    };
    // SAFETY: `class` is fully initialised with a valid window procedure and static strings.
    unsafe { crate::register_class_once(&REGISTERED, &class) }
}

/// The hidden app window. Lives on the main thread; not `Send`.
pub struct AppWindow {
    hwnd: HWND,
    state: *mut WindowState,
}

impl std::fmt::Debug for AppWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppWindow")
            .field("hwnd", &self.hwnd)
            .finish_non_exhaustive()
    }
}

impl AppWindow {
    /// Creates the window and its tray icon with `tooltip`; events go to `handler`.
    ///
    /// # Errors
    /// [`WinError`] if the class or window cannot be created. A tray icon that cannot be
    /// added (no Explorer shell) is logged, not fatal.
    pub fn create(tooltip: &str, handler: AppHandler) -> Result<Self, WinError> {
        register_class()?;
        // SAFETY: registered class, static strings; a hidden top-level window (not message-only,
        // so it receives the TaskbarCreated broadcast).
        let hwnd = unsafe {
            let instance = GetModuleHandleW(None).unwrap_or_default();
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                CLASS_NAME,
                w!("FH6 WheelDash"),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(instance.into()),
                None,
            )
        }
        .map_err(|e| WinError::call("CreateWindowExW", e))?;
        // SAFETY: static NUL-terminated name.
        let taskbar_created = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
        let tip = encode_tip(tooltip);
        let state = Box::into_raw(Box::new(WindowState {
            handler: RefCell::new(handler),
            tooltip: Cell::new(tip),
            overlay_checked: Cell::new(true),
            taskbar_created,
            destroyed: Cell::new(false),
        }));
        // SAFETY: `state` is a valid heap pointer owned by the returned AppWindow and freed in
        // its Drop after the window is destroyed.
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize) };
        if !notify_icon(hwnd, tip, NIM_ADD) {
            tracing::warn!("tray icon could not be added");
        }
        Ok(Self { hwnd, state })
    }

    /// Control handle for the main thread.
    pub fn ctl(&self) -> WindowCtl<'_> {
        // SAFETY: `state` is the Box pointer installed in `create`; it is freed only in Drop,
        // and the returned WindowCtl borrows `self`, so it cannot outlive it.
        let state = unsafe { &*self.state };
        WindowCtl { hwnd: self.hwnd, state }
    }

    /// A `Send` handle other threads use to ask the app to close.
    pub fn close_handle(&self) -> CloseHandle {
        CloseHandle {
            hwnd: self.hwnd.0 as isize,
        }
    }

    /// Runs the message loop until the window is destroyed. Returns the exit code.
    pub fn run(&self) -> i32 {
        let mut msg = MSG::default();
        loop {
            // SAFETY: standard message loop on the thread that owns the window.
            let got = unsafe { GetMessageW(&mut msg, None, 0, 0) };
            match got.0 {
                0 => return i32::try_from(msg.wParam.0).unwrap_or(0),
                -1 => return 1,
                _ => {
                    // SAFETY: `msg` was filled by GetMessageW.
                    unsafe {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }
        }
    }
}

impl Drop for AppWindow {
    fn drop(&mut self) {
        // SAFETY: `state` is the pointer installed in `create`; the window is destroyed (and
        // its user data cleared on WM_NCDESTROY) before the state is freed, so no message
        // can observe a dangling pointer.
        unsafe {
            if let Some(state) = self.state.as_ref() {
                if !state.destroyed.get() {
                    let _ = DestroyWindow(self.hwnd);
                }
            }
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
            drop(Box::from_raw(self.state));
        }
    }
}
