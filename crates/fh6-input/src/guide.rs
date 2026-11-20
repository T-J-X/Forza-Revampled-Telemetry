use crate::xinput::{XInputDynamic, XInputState};

pub struct GuideEdgeDetector {
    prev_pressed: bool,
    enabled: bool,
}

impl GuideEdgeDetector {
    pub fn new(enabled: bool) -> Self {
        Self {
            prev_pressed: false,
            enabled,
        }
    }

    pub fn update(&mut self, xinput: &XInputDynamic, state: &XInputState) -> Option<()> {
        if !self.enabled {
            self.prev_pressed = false;
            return None;
        }
        let pressed = xinput.guide_button_pressed(state);
        let edge = pressed && !self.prev_pressed;
        self.prev_pressed = pressed;
        if edge {
            Some(())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xinput::{XInputDynamic, XInputGamepad, XInputState};

    #[test]
    fn guide_edge_triggers_widget_once() {
        let xinput = XInputDynamic {
            is_loaded: true,
            active_dll: "xinput1_4.dll".to_string(),
            guide_detection_enabled: true,
            #[cfg(windows)]
            xinput_get_state_ex: None,
        };
        let mut det = GuideEdgeDetector::new(true);
        let state_release = XInputState { Gamepad: XInputGamepad { wButtons: 0, ..Default::default() } };
        let state_press = XInputState { Gamepad: XInputGamepad { wButtons: 0x0400, ..Default::default() } };
        assert_eq!(det.update(&xinput, &state_release), None);
        assert_eq!(det.update(&xinput, &state_press), Some(()));
        // hold
        assert_eq!(det.update(&xinput, &state_press), None);
        // release
        assert_eq!(det.update(&xinput, &state_release), None);
        // press again
        assert_eq!(det.update(&xinput, &state_press), Some(()));
    }
}
