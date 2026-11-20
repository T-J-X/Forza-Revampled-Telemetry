#[cfg(windows)]
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, CloseHandle};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::PCWSTR;

const MUTEX_NAME: &str = "FH6_WheelDash_SingleInstance";

#[derive(Debug)]
pub struct SingleInstanceGuard {
    #[cfg(windows)]
    handle: Option<windows::Win32::Foundation::HANDLE>,
}

impl SingleInstanceGuard {
    pub fn new() -> Result<Self, SingleInstanceError> {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            let name_w: Vec<u16> = MUTEX_NAME.encode_utf16().chain(std::iter::once(0)).collect();
            let name_pcw = PCWSTR::from_raw(name_w.as_ptr());
            unsafe {
                let h = CreateMutexW(None, false, name_pcw).map_err(|_| SingleInstanceError::CreateFailed)?;
                if h.0.is_null() {
                    return Err(SingleInstanceError::CreateFailed);
                }
                let last_err = GetLastError();
                if last_err == ERROR_ALREADY_EXISTS {
                    CloseHandle(h);
                    return Err(SingleInstanceError::AlreadyRunning);
                }
                Ok(Self { handle: Some(h) })
            }
        }
        #[cfg(not(windows))]
        {
            // On non-Windows, assume single instance for tests
            Ok(Self {})
        }
    }

    pub fn is_running(&self) -> bool {
        false
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        if let Some(h) = self.handle.take() {
            unsafe { CloseHandle(h); }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SingleInstanceError {
    AlreadyRunning,
    CreateFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_new_succeeds_on_non_windows() {
        let g = SingleInstanceGuard::new();
        assert!(g.is_ok());
    }

    #[test]
    fn error_variants_are_distinct() {
        assert_ne!(SingleInstanceError::AlreadyRunning, SingleInstanceError::CreateFailed);
    }
}
