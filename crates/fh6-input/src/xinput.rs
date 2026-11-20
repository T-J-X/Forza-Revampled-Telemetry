//! Dynamic XInput loading: `xinput1_4.dll`, falling back to `xinput9_1_0.dll`.
//!
//! `XInputGetState` is resolved by name. `XInputGetStateEx` (ordinal 100, exported only by
//! `xinput1_4.dll`) is resolved when present; it is the only call that reports the Guide
//! button (`0x0400`). Both DLLs are loaded from System32 only, so a planted copy in the
//! working directory is never picked up.

/// Guide button bit, reported only by `XInputGetStateEx`.
pub const XINPUT_GAMEPAD_GUIDE: u16 = 0x0400;

/// Win32 `XINPUT_GAMEPAD`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(non_snake_case, reason = "mirrors the Win32 struct field names")]
pub struct XInputGamepad {
    pub wButtons: u16,
    pub bLeftTrigger: u8,
    pub bRightTrigger: u8,
    pub sThumbLX: i16,
    pub sThumbLY: i16,
    pub sThumbRX: i16,
    pub sThumbRY: i16,
}

/// Win32 `XINPUT_STATE`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(non_snake_case, reason = "mirrors the Win32 struct field names")]
pub struct XInputState {
    pub dwPacketNumber: u32,
    pub Gamepad: XInputGamepad,
}

/// The buffer handed to both entry points. Ordinal 100 is documented by the community (SDL's
/// `XINPUT_GAMEPAD_EX`) as writing 4 reserved bytes after the gamepad, so the buffer is padded.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct XInputStateEx {
    state: XInputState,
    reserved: u32,
}

/// Why XInput could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum XInputError {
    /// Neither DLL could be loaded from System32.
    #[error("neither xinput1_4.dll nor xinput9_1_0.dll could be loaded")]
    NotFound,
    /// The DLL loaded but does not export `XInputGetState`.
    #[error("{dll} has no XInputGetState export")]
    MissingExport {
        /// The DLL that was loaded.
        dll: &'static str,
    },
    /// XInput only exists on Windows.
    #[error("XInput is only available on Windows")]
    Unsupported,
}

#[cfg(windows)]
mod imp {
    use super::{XInputError, XInputState, XInputStateEx};
    use windows::core::{s, PCSTR, PCWSTR};
    use windows::Win32::Foundation::{FreeLibrary, HMODULE};
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32};

    type GetStateFn = unsafe extern "system" fn(u32, *mut XInputStateEx) -> u32;

    const ERROR_SUCCESS: u32 = 0;
    const GET_STATE_EX_ORDINAL: usize = 100;

    /// A loaded XInput DLL. Not `Send`: create it on the thread that polls.
    pub struct XInput {
        module: HMODULE,
        dll: &'static str,
        get_state: GetStateFn,
        get_state_ex: Option<GetStateFn>,
    }

    impl std::fmt::Debug for XInput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("XInput")
                .field("dll", &self.dll)
                .field("guide", &self.get_state_ex.is_some())
                .finish()
        }
    }

    impl XInput {
        /// Loads `xinput1_4.dll` (falling back to `xinput9_1_0.dll`) and resolves its exports.
        ///
        /// # Errors
        /// [`XInputError::NotFound`] if neither DLL loads, [`XInputError::MissingExport`] if
        /// `XInputGetState` is missing.
        pub fn load() -> Result<Self, XInputError> {
            const CANDIDATES: [(&str, PCWSTR); 2] = [
                ("xinput1_4.dll", windows::core::w!("xinput1_4.dll")),
                ("xinput9_1_0.dll", windows::core::w!("xinput9_1_0.dll")),
            ];
            for (dll, wide) in CANDIDATES {
                // SAFETY: `wide` is a static NUL-terminated UTF-16 literal; the search is
                // restricted to System32, and the returned module is freed in Drop.
                let Ok(module) = (unsafe { LoadLibraryExW(wide, None, LOAD_LIBRARY_SEARCH_SYSTEM32) }) else {
                    continue;
                };
                // SAFETY: `module` was just loaded and is valid; `s!` yields a static C string.
                let get_state = unsafe { GetProcAddress(module, s!("XInputGetState")) };
                let Some(get_state) = get_state else {
                    // SAFETY: `module` came from LoadLibraryExW above and is not used again.
                    unsafe {
                        let _ = FreeLibrary(module);
                    }
                    return Err(XInputError::MissingExport { dll });
                };
                // GetProcAddress takes an ordinal as a PCSTR whose high bits are zero.
                let ordinal = PCSTR(GET_STATE_EX_ORDINAL as *const u8);
                // SAFETY: `module` is valid; an ordinal below 0x10000 is a documented form of
                // `lpProcName`.
                let get_state_ex = unsafe { GetProcAddress(module, ordinal) };
                // SAFETY: both exports have the signature
                // `DWORD WINAPI XInputGetState(DWORD dwUserIndex, XINPUT_STATE* pState)`;
                // `XInputStateEx` begins with an `XINPUT_STATE` and is larger than it, so
                // passing it is sound for both entry points.
                let (get_state, get_state_ex) = unsafe {
                    (
                        std::mem::transmute::<unsafe extern "system" fn() -> isize, GetStateFn>(get_state),
                        get_state_ex.map(|f| std::mem::transmute::<unsafe extern "system" fn() -> isize, GetStateFn>(f)),
                    )
                };
                tracing::info!(dll, guide = get_state_ex.is_some(), "XInput loaded");
                return Ok(Self {
                    module,
                    dll,
                    get_state,
                    get_state_ex,
                });
            }
            Err(XInputError::NotFound)
        }

        /// The DLL that was loaded.
        pub fn dll_name(&self) -> &'static str {
            self.dll
        }

        /// True when `XInputGetStateEx` is available, so the Guide button is reported.
        pub fn has_guide(&self) -> bool {
            self.get_state_ex.is_some()
        }

        /// Polls one controller slot (0..=3). `None` if nothing is connected there.
        pub fn state(&self, slot: u32) -> Option<XInputState> {
            let mut buf = XInputStateEx::default();
            let f = self.get_state_ex.unwrap_or(self.get_state);
            // SAFETY: `f` is a live export of the still-loaded module (freed only in Drop) and
            // `buf` is a valid, writable, correctly aligned XINPUT_STATE-prefixed buffer.
            let rc = unsafe { f(slot, &mut buf) };
            (rc == ERROR_SUCCESS).then_some(buf.state)
        }
    }

    impl Drop for XInput {
        fn drop(&mut self) {
            // SAFETY: `module` came from LoadLibraryExW in `load` and no export is used after
            // this point, because `self` is being destroyed.
            unsafe {
                let _ = FreeLibrary(self.module);
            }
        }
    }
}

#[cfg(windows)]
pub use imp::XInput;

/// Stand-in so the crate builds off Windows; loading always fails.
#[cfg(not(windows))]
#[derive(Debug)]
pub struct XInput;

#[cfg(not(windows))]
impl XInput {
    /// Always fails off Windows.
    ///
    /// # Errors
    /// Always [`XInputError::Unsupported`].
    pub fn load() -> Result<Self, XInputError> {
        Err(XInputError::Unsupported)
    }
    /// Never reports a controller.
    pub fn state(&self, _slot: u32) -> Option<XInputState> {
        None
    }
    /// No Guide support.
    pub fn has_guide(&self) -> bool {
        false
    }
    /// No DLL.
    pub fn dll_name(&self) -> &'static str {
        ""
    }
}

impl crate::tracker::PadSource for XInput {
    fn poll_slot(&mut self, slot: usize) -> Option<XInputState> {
        u32::try_from(slot).ok().and_then(|s| self.state(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_layout_matches_win32() {
        assert_eq!(std::mem::size_of::<XInputGamepad>(), 12);
        assert_eq!(std::mem::size_of::<XInputState>(), 16);
        assert_eq!(std::mem::size_of::<XInputStateEx>(), 20);
    }

    #[cfg(windows)]
    #[test]
    fn loads_from_system32() {
        let xinput = XInput::load().expect("xinput1_4.dll ships with Windows 10 and 11");
        assert_eq!(xinput.dll_name(), "xinput1_4.dll");
        // Polling an empty slot must not crash; a pad may or may not be connected.
        let _ = xinput.state(3);
    }
}
