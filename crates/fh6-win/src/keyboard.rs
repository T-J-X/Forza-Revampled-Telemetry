//! Synthetic key taps through `SendInput` (the OSD's auto-pause ESC).
//!
//! Keys are sent as scan codes (`KEYEVENTF_SCANCODE`), which games reading raw or DirectInput
//! keyboard state see like a real key. The press and release go in one `SendInput` call so
//! nothing can interleave between them.
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC, VIRTUAL_KEY,
};

/// Sends key taps to the foreground window.
#[derive(Debug, Default, Clone, Copy)]
pub struct SendInputKeyboard;

impl SendInputKeyboard {
    /// A keyboard injector.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Taps virtual key `vk` (down, then up). Returns false when the key has no scan code or
    /// the system blocked the input (for example UIPI against an elevated foreground window).
    pub fn tap_key(&mut self, vk: u16) -> bool {
        // SAFETY: MapVirtualKeyW only reads its two integer arguments.
        let scan = unsafe { MapVirtualKeyW(u32::from(vk), MAPVK_VK_TO_VSC) };
        let Ok(scan) = u16::try_from(scan) else {
            return false;
        };
        if scan == 0 {
            return false;
        }
        let key = |flags: KEYBD_EVENT_FLAGS| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: scan,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let inputs = [
            key(KEYEVENTF_SCANCODE),
            key(KEYBD_EVENT_FLAGS(KEYEVENTF_SCANCODE.0 | KEYEVENTF_KEYUP.0)),
        ];
        let size = i32::try_from(std::mem::size_of::<INPUT>()).unwrap_or(i32::MAX);
        // SAFETY: `inputs` is a live slice of fully initialised keyboard INPUT structs and
        // `size` is the size of one INPUT, as SendInput requires; the call does not retain
        // the pointer.
        let sent = unsafe { SendInput(&inputs, size) };
        let ok = sent as usize == inputs.len();
        if !ok {
            tracing::warn!(vk, sent, "SendInput did not inject the key");
        }
        ok
    }
}
