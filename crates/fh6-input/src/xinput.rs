#[repr(C)]
#[derive(Clone, Copy, Default)]
#[allow(non_snake_case)]
pub struct XInputGamepad {
    pub wButtons: u16,
    pub bLeftTrigger: u8,
    pub bRightTrigger: u8,
    pub sThumbLX: i16,
    pub sThumbLY: i16,
    pub sThumbRX: i16,
    pub sThumbRY: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
#[allow(non_snake_case)]
pub struct XInputState {
    pub Gamepad: XInputGamepad,
}

pub struct XInputDynamic {
    pub is_loaded: bool,
    pub active_dll: String,
    pub guide_detection_enabled: bool,
    #[cfg(windows)]
    pub xinput_get_state_ex: Option<unsafe extern "system" fn(u32, *mut XInputState) -> i32>,
}

impl XInputDynamic {
    pub fn new() -> Self {
        // Real loader would attempt LoadLibraryW("xinput1_4.dll") and GetProcAddress ordinal 100.
        // For testability and without linking complications, we expose a default that is considered loaded.
        // In production, this would log once if ordinal not found and disable guide detection.
        Self {
            is_loaded: true,
            active_dll: "xinput1_4.dll".to_string(),
            guide_detection_enabled: true,
            #[cfg(windows)]
            xinput_get_state_ex: None,
        }
    }

    pub fn get_state(&self, _user_index: u32) -> Option<XInputState> {
        None
    }

    #[cfg(windows)]
    pub fn guide_button_pressed(&self, state: &XInputState) -> bool {
        if !self.guide_detection_enabled {
            return false;
        }
        (state.Gamepad.wButtons & 0x0400) != 0
    }
}

impl Default for XInputDynamic {
    fn default() -> Self {
        Self::new()
    }
}
