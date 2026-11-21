//! Rising-edge detection for the Guide button.

/// Reports a Guide press once per press.
#[derive(Debug, Clone, Default)]
pub struct GuideEdgeDetector {
    prev_pressed: bool,
    enabled: bool,
}

impl GuideEdgeDetector {
    /// A detector; when `enabled` is false it never fires.
    pub fn new(enabled: bool) -> Self {
        Self {
            prev_pressed: false,
            enabled,
        }
    }

    /// Feeds the current Guide state; returns true on the press edge.
    pub fn update(&mut self, pressed: bool) -> bool {
        if !self.enabled {
            self.prev_pressed = false;
            return false;
        }
        let edge = pressed && !self.prev_pressed;
        self.prev_pressed = pressed;
        edge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guide_edge_triggers_once_per_press() {
        let mut det = GuideEdgeDetector::new(true);
        assert!(!det.update(false));
        assert!(det.update(true));
        assert!(!det.update(true), "held");
        assert!(!det.update(false), "released");
        assert!(det.update(true), "pressed again");
    }

    #[test]
    fn disabled_never_fires() {
        let mut det = GuideEdgeDetector::new(false);
        assert!(!det.update(true));
    }
}
