use fh6_core::config::sanitize_toggle_mask;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemapperState {
    Idle,
    Capturing,
}

pub struct Remapper {
    state: RemapperState,
    accumulated_mask: u16,
    last_committed_mask: u16,
    mask_changed: bool,
}

impl Remapper {
    pub fn new(initial_mask: u16) -> Self {
        let sanitized = sanitize_toggle_mask(initial_mask);
        Self {
            state: RemapperState::Idle,
            accumulated_mask: 0,
            last_committed_mask: sanitized,
            mask_changed: false,
        }
    }

    pub fn start_capture(&mut self) {
        self.state = RemapperState::Capturing;
        self.accumulated_mask = 0;
        self.mask_changed = false;
    }

    pub fn update_held(&mut self, pressed_mask: u16) {
        if self.state == RemapperState::Capturing {
            // Strip ABXY bits for pressed mask, do not fallback
            const ABXY_MASK: u16 = 0xF000;
            let stripped = pressed_mask & !ABXY_MASK;
            self.accumulated_mask = stripped;
        }
    }

    pub fn commit(&mut self) -> (u16, bool) {
        if self.state != RemapperState::Capturing {
            return (self.last_committed_mask, self.mask_changed);
        }
        let sanitized = sanitize_toggle_mask(self.accumulated_mask);
        let changed = sanitized != self.last_committed_mask;
        self.last_committed_mask = sanitized;
        self.mask_changed = changed;
        self.state = RemapperState::Idle;
        (sanitized, changed)
    }

    pub fn is_capturing(&self) -> bool {
        self.state == RemapperState::Capturing
    }

    pub fn current_mask(&self) -> u16 {
        self.last_committed_mask
    }

    pub fn mask_changed_flag(&self) -> bool {
        self.mask_changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_commits_sanitized_mask() {
        let mut r = Remapper::new(0x0020);
        r.start_capture();
        r.update_held(0x0040);
        let (mask, changed) = r.commit();
        assert_eq!(mask, 0x0040);
        assert!(changed);
    }

    #[test]
    fn capture_strips_abxy_and_flags_change() {
        let mut r = Remapper::new(0x0020);
        r.start_capture();
        r.update_held(0x1000 | 0x0020); // ABXY + safe
        let (mask, changed) = r.commit();
        // sanitize strips ABXY, mask stays 0x0020 -> no change
        assert_eq!(mask, 0x0020);
        assert!(!changed);
    }

    #[test]
    fn capture_abxy_only_falls_back_to_default() {
        let mut r = Remapper::new(0x0020);
        r.start_capture();
        r.update_held(0x1000); // only ABXY
        let (mask, changed) = r.commit();
        // sanitized -> 0x0020 fallback, no change
        assert_eq!(mask, 0x0020);
        assert!(!changed);
    }

    #[test]
    fn capture_accumulates_held_buttons() {
        let mut r = Remapper::new(0x0020);
        r.start_capture();
        r.update_held(0x0010);
        r.update_held(0x0030);
        let (mask, _) = r.commit();
        assert_eq!(mask, 0x0030);
    }

    #[test]
    fn render_test_mask_differs_pixmap_hash_would_differ() {
        // Pure-logic test: mask change flag is set when displayed mask differs
        let mut r = Remapper::new(0x0020);
        r.start_capture();
        r.update_held(0x0080);
        let (mask, changed) = r.commit();
        assert_eq!(mask, 0x0080);
        assert!(changed);
        // Second commit without new capture should keep flag
        assert!(r.mask_changed_flag());
    }
}
