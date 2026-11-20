use std::time::Instant;

/// OLED care engine: sub-pixel orbit and idle dimmer.
///
/// Protects the LG C1 panel from burn-in by orbiting the overlay a few pixels
/// and dimming it when the car and the controller are idle.
#[derive(Debug)]
pub struct OledCareEngine {
    start_time: Instant,
    last_input_time: Instant,
    current_dim_alpha: f32,
}

impl OledCareEngine {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_input_time: now,
            current_dim_alpha: 1.0,
        }
    }

    /// Returns pixel offset for Lissajous orbit when enabled.
    /// X period 180s, Y period 240s, amplitude ±2px.
    pub fn get_lissajous_pixel_offset(&self, enabled: bool) -> (i32, i32) {
        if !enabled {
            return (0, 0);
        }
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let dx = (2.0 * (2.0 * std::f64::consts::PI * elapsed / 180.0).sin()).round() as i32;
        let dy = (2.0 * (2.0 * std::f64::consts::PI * elapsed / 240.0).cos()).round() as i32;
        (dx, dy)
    }

    /// Updates the idle-dimmer alpha from speed, RPM and input activity.
    pub fn update_dimmer(
        &mut self,
        enabled: bool,
        speed_ms: f32,
        current_rpm: f32,
        idle_rpm: f32,
        controller_buttons: u16,
        is_osd_open: bool,
        min_alpha: f32,
    ) -> f32 {
        let now = Instant::now();
        if !enabled || is_osd_open {
            self.last_input_time = now;
            self.current_dim_alpha = 1.0;
            return 1.0;
        }

        let has_button_input = controller_buttons != 0;
        let rev_threshold = idle_rpm + 150.0;
        let rev_check = if rev_threshold > 1100.0 { rev_threshold } else { 1100.0 };
        let is_moving_or_revving = speed_ms >= 0.2 || current_rpm > rev_check;

        if has_button_input || is_moving_or_revving {
            self.last_input_time = now;
            self.current_dim_alpha = 1.0;
            return 1.0;
        }

        let idle_seconds = now.duration_since(self.last_input_time).as_secs_f64() as f32;
        if idle_seconds > 30.0 {
            if self.current_dim_alpha > min_alpha {
                self.current_dim_alpha = (self.current_dim_alpha - 0.015).max(min_alpha);
            }
        } else {
            self.current_dim_alpha = 1.0;
        }
        self.current_dim_alpha
    }

    pub fn report_activity(&mut self) {
        let now = Instant::now();
        self.last_input_time = now;
        self.current_dim_alpha = 1.0;
    }

    #[cfg(test)]
    fn set_times_for_test(&mut self, start: Instant, last: Instant, alpha: f32) {
        self.start_time = start;
        self.last_input_time = last;
        self.current_dim_alpha = alpha;
    }
}

impl Default for OledCareEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn orbit_bounds() {
        let engine = OledCareEngine::new();
        for _ in 0..1000 {
            let (dx, dy) = engine.get_lissajous_pixel_offset(true);
            assert!((-2..=2).contains(&dx));
            assert!((-2..=2).contains(&dy));
        }
        let (dx, dy) = engine.get_lissajous_pixel_offset(false);
        assert_eq!((dx, dy), (0, 0));
    }

    #[test]
    fn dimmer_floor_respected() {
        let mut engine = OledCareEngine::new();
        let min_alpha = 0.35;
        let start = Instant::now() - Duration::from_secs(1000);
        let last = Instant::now() - Duration::from_secs(100);
        engine.set_times_for_test(start, last, 1.0);
        for _ in 0..200 {
            let a = engine.update_dimmer(true, 0.0, 800.0, 800.0, 0, false, min_alpha);
            assert!(a >= min_alpha - 1e-6);
            assert!(a <= 1.0);
        }
    }

    #[test]
    fn dimmer_wakes_on_input() {
        let mut engine = OledCareEngine::new();
        let start = Instant::now();
        let last = Instant::now() - Duration::from_secs(100);
        engine.set_times_for_test(start, last, 0.5);
        let a = engine.update_dimmer(true, 0.0, 800.0, 800.0, 0x0010, false, 0.35);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn dimmer_wakes_on_movement() {
        let mut engine = OledCareEngine::new();
        let start = Instant::now();
        let last = Instant::now() - Duration::from_secs(100);
        engine.set_times_for_test(start, last, 0.5);
        let a = engine.update_dimmer(true, 0.5, 800.0, 800.0, 0, false, 0.35);
        assert_eq!(a, 1.0);
    }
}
