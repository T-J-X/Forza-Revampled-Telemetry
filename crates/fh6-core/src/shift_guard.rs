/// Manual shift guardian with money-shift detection.
///
/// Learns per-gear ratios while driving and flags a downshift that would
/// over-rev the engine.
#[derive(Debug, Default)]
pub struct ManualShiftGuardian {
    gear_ratios: [f32; 11],
    sample_counts: [u32; 11],
    pub is_money_shift_hazard: bool,
    pub projected_downshift_rpm: f32,
    pub rev_match_target_rpm: f32,
    pub max_safe_downshift_speed_mph: f32,
}

impl ManualShiftGuardian {
    pub fn new() -> Self {
        let mut ratios = [0.0f32; 11];
        ratios[1] = 175.0;
        ratios[2] = 115.0;
        ratios[3] = 82.0;
        ratios[4] = 62.0;
        ratios[5] = 49.0;
        ratios[6] = 40.0;
        ratios[7] = 34.0;
        ratios[8] = 29.0;
        ratios[9] = 25.0;
        ratios[10] = 22.0;
        Self {
            gear_ratios: ratios,
            sample_counts: [0; 11],
            is_money_shift_hazard: false,
            projected_downshift_rpm: 0.0,
            rev_match_target_rpm: 0.0,
            max_safe_downshift_speed_mph: 0.0,
        }
    }

    pub fn update(
        &mut self,
        current_gear: i32,
        speed_ms: f32,
        current_rpm: f32,
        max_rpm: f32,
        _idle_rpm: f32,
        accel: u8,
        _brake: u8,
    ) {
        if max_rpm <= 500.0 {
            self.is_money_shift_hazard = false;
            self.projected_downshift_rpm = 0.0;
            self.rev_match_target_rpm = 0.0;
            self.max_safe_downshift_speed_mph = 0.0;
            return;
        }

        if (1..=10).contains(&current_gear) {
            let g = current_gear as usize;
            if speed_ms > 5.0 && current_rpm > 1500.0 && accel > 30 {
                let observed_ratio = current_rpm / speed_ms;
                if observed_ratio > 10.0 && observed_ratio < 400.0 {
                    if self.sample_counts[g] == 0 {
                        self.gear_ratios[g] = observed_ratio;
                    } else {
                        self.gear_ratios[g] = self.gear_ratios[g] * 0.95 + observed_ratio * 0.05;
                    }
                    if self.sample_counts[g] < 1000 {
                        self.sample_counts[g] += 1;
                    }
                }
            }
        }

        if (2..=10).contains(&current_gear) {
            let current_idx = current_gear as usize;
            let lower_idx = (current_gear - 1) as usize;
            let mut lower_ratio = self.gear_ratios[lower_idx];
            if lower_ratio <= 0.01 {
                lower_ratio = self.gear_ratios[current_idx] * 1.32;
            }

            self.projected_downshift_rpm = speed_ms * lower_ratio;
            self.rev_match_target_rpm = self.projected_downshift_rpm;
            self.max_safe_downshift_speed_mph = (max_rpm * 0.98 / lower_ratio) * 2.23694;

            self.is_money_shift_hazard = self.projected_downshift_rpm >= max_rpm * 0.98;
        } else {
            self.is_money_shift_hazard = false;
            self.projected_downshift_rpm = 0.0;
            self.rev_match_target_rpm = 0.0;
            self.max_safe_downshift_speed_mph = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_off_when_max_rpm_low() {
        let mut g = ManualShiftGuardian::new();
        g.update(3, 10.0, 2000.0, 400.0, 800.0, 50, 0);
        assert!(!g.is_money_shift_hazard);
        assert_eq!(g.projected_downshift_rpm, 0.0);
    }

    #[test]
    fn guard_detects_hazard() {
        let mut g = ManualShiftGuardian::new();
        // Use gear 3, speed high enough to project >0.98 max rpm
        // With default ratios: gear3=82, gear2=115
        // projected = speed *115
        // Need projected >=0.98*max_rpm
        // Choose speed=80, max_rpm=8000 => projected=9200 > 7840
        g.update(3, 80.0, 3000.0, 8000.0, 800.0, 50, 0);
        assert!(g.is_money_shift_hazard);
        assert!(g.projected_downshift_rpm > 0.0);
    }

    #[test]
    fn guard_no_hazard_low_speed() {
        let mut g = ManualShiftGuardian::new();
        g.update(5, 5.0, 2000.0, 8000.0, 800.0, 50, 0);
        assert!(!g.is_money_shift_hazard);
    }

    #[test]
    fn learns_ratio() {
        let mut g = ManualShiftGuardian::new();
        // Simulate learning
        g.update(4, 20.0, 2000.0, 8000.0, 800.0, 50, 0);
        // sample count should increase
        assert!(g.sample_counts[4] > 0);
    }
}
