//! Is the game the foreground window? Event-driven, with a polling backstop.
//!
//! `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` reports every focus change immediately (the hook
//! runs on the installing thread's message loop). [`ForegroundWatcher::poll`] repeats the check
//! from a 10 Hz timer in case an event is missed. Process names are cached per window, so a
//! check is normally one `GetForegroundWindow` plus one `GetWindowThreadProcessId`.
use crate::WinError;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT,
};

/// Called when the game gains or loses the foreground.
pub type ForegroundNotify = Arc<dyn Fn() + Send + Sync>;

/// Lock-free view of the foreground state, for the render thread.
#[derive(Debug, Clone)]
pub struct ForegroundReader {
    shared: Arc<Shared>,
}

impl ForegroundReader {
    /// True while a window of the game process has the foreground.
    pub fn is_game_foreground(&self) -> bool {
        self.shared.game_foreground.load(Ordering::Relaxed)
    }

    /// The game's foreground window (raw `HWND`), if it has been seen.
    pub fn game_window(&self) -> Option<isize> {
        match self.shared.game_hwnd.load(Ordering::Relaxed) {
            0 => None,
            h => Some(h),
        }
    }
}

#[derive(Debug, Default)]
struct Cache {
    hwnd: isize,
    pid: u32,
    is_game: bool,
}

struct Shared {
    target_exe: String,
    game_foreground: AtomicBool,
    game_hwnd: AtomicIsize,
    cache: Mutex<Cache>,
    notify: ForegroundNotify,
}

impl std::fmt::Debug for Shared {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shared")
            .field("target_exe", &self.target_exe)
            .field("game_foreground", &self.game_foreground)
            .finish_non_exhaustive()
    }
}

/// The hook callback has no user-data pointer, so the watcher state is process-global.
static ACTIVE: OnceLock<Mutex<Option<Arc<Shared>>>> = OnceLock::new();

fn active() -> Option<Arc<Shared>> {
    ACTIVE.get()?.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

/// The file name of the process that owns `pid`, lower-cased.
pub(crate) fn process_image_name(pid: u32) -> Option<String> {
    // SAFETY: OpenProcess with query-only rights; the handle is closed below.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut buf = [0u16; 1024];
    let mut len = u32::try_from(buf.len()).unwrap_or(1024);
    // SAFETY: `buf` is writable for `len` UTF-16 units and `process` is a valid handle.
    let ok = unsafe { QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len) };
    // SAFETY: `process` came from OpenProcess and is closed exactly once.
    unsafe {
        let _ = CloseHandle(process);
    }
    ok.ok()?;
    let path = String::from_utf16_lossy(buf.get(..len as usize)?);
    let name = path.rsplit(['\\', '/']).next().unwrap_or(&path);
    Some(name.to_ascii_lowercase())
}

fn refresh(shared: &Shared) {
    // SAFETY: no preconditions.
    let hwnd = unsafe { GetForegroundWindow() };
    let raw = hwnd.0 as isize;
    let is_game = if raw == 0 {
        false
    } else {
        let mut cache = shared.cache.lock().unwrap_or_else(PoisonError::into_inner);
        let mut pid = 0u32;
        // SAFETY: `pid` is a valid out pointer; a stale HWND just yields 0.
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if cache.hwnd != raw || cache.pid != pid {
            let is_game = pid != 0 && process_image_name(pid).is_some_and(|n| n == shared.target_exe);
            *cache = Cache {
                hwnd: raw,
                pid,
                is_game,
            };
        }
        cache.is_game
    };
    if is_game {
        shared.game_hwnd.store(raw, Ordering::Relaxed);
    }
    let was = shared.game_foreground.swap(is_game, Ordering::Relaxed);
    if was != is_game {
        tracing::debug!(game_foreground = is_game, "foreground changed");
        (shared.notify)();
    }
}

unsafe extern "system" fn on_foreground_event(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if let Some(shared) = active() {
        refresh(&shared);
    }
}

/// Installs the foreground hook. Create it on the thread that runs the message loop; it is
/// not `Send`, and dropping it removes the hook.
#[derive(Debug)]
pub struct ForegroundWatcher {
    hook: HWINEVENTHOOK,
    shared: Arc<Shared>,
}

impl ForegroundWatcher {
    /// Watches for windows of the process named `target_exe` (for example `ForzaHorizon6.exe`,
    /// compared case-insensitively). `notify` runs when the game gains or loses the foreground.
    ///
    /// # Errors
    /// [`WinError`] if the hook cannot be installed.
    pub fn install(target_exe: &str, notify: ForegroundNotify) -> Result<Self, WinError> {
        let shared = Arc::new(Shared {
            target_exe: target_exe.to_ascii_lowercase(),
            game_foreground: AtomicBool::new(false),
            game_hwnd: AtomicIsize::new(0),
            cache: Mutex::new(Cache::default()),
            notify,
        });
        *ACTIVE
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(Arc::clone(&shared));
        // SAFETY: an out-of-context hook with a 'static callback; it is delivered through this
        // thread's message loop and removed in Drop.
        let hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(on_foreground_event),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        };
        if hook.is_invalid() {
            return Err(WinError::last("SetWinEventHook"));
        }
        refresh(&shared);
        Ok(Self { hook, shared })
    }

    /// Re-checks the foreground window (the 10 Hz backstop).
    pub fn poll(&self) {
        refresh(&self.shared);
    }

    /// A lock-free reader for other threads.
    pub fn reader(&self) -> ForegroundReader {
        ForegroundReader {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl Drop for ForegroundWatcher {
    fn drop(&mut self) {
        // SAFETY: `hook` came from SetWinEventHook on this thread and is removed once.
        unsafe {
            let _ = UnhookWinEvent(self.hook);
        }
        if let Some(slot) = ACTIVE.get() {
            let mut slot = slot.lock().unwrap_or_else(PoisonError::into_inner);
            if slot.as_ref().is_some_and(|s| Arc::ptr_eq(s, &self.shared)) {
                *slot = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_process_name_is_found() {
        let name = process_image_name(std::process::id()).expect("can query our own process");
        assert!(name.ends_with(".exe"), "{name}");
        assert_eq!(name, name.to_ascii_lowercase());
    }
}
