use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };
    pub const MOTORSPORT_GREEN: Self = Self {
        r: 0,
        g: 230,
        b: 118,
        a: 255,
    };
    pub const ICE_CYAN: Self = Self {
        r: 0,
        g: 229,
        b: 255,
        a: 255,
    };
    pub const HIGH_VIS_AMBER: Self = Self {
        r: 255,
        g: 179,
        b: 0,
        a: 255,
    };
    pub const CRIMSON_SHIFT: Self = Self {
        r: 255,
        g: 23,
        b: 68,
        a: 255,
    };
    pub const STROBE_CRIMSON: Self = Self {
        r: 255,
        g: 0,
        b: 85,
        a: 255,
    };
    pub const STROBE_WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct RevLimiterState {
    pub stage: u8,
    pub is_strobe_active: bool,
    pub strobe_phase: bool,
    pub active_stage_color: Rgba,
    pub strobe_color: Rgba,
    pub show_shift_halo: bool,
}

pub struct RevLimiterSystem;

impl RevLimiterSystem {
    pub fn evaluate(
        normalized_rpm: f32,
        shift_start_pct: f32,
        redline_pct: f32,
        brand_accent: Option<Rgba>,
    ) -> RevLimiterState {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let strobe_phase = (now_ms % 83) < 42;
        let strobe_color = if strobe_phase {
            Rgba::STROBE_CRIMSON
        } else {
            Rgba::STROBE_WHITE
        };

        let effective_redline = if redline_pct > 0.01 { redline_pct } else { 0.94 };

        let mut state = RevLimiterState {
            stage: 1,
            is_strobe_active: false,
            strobe_phase,
            active_stage_color: Rgba::TRANSPARENT,
            strobe_color,
            show_shift_halo: false,
        };

        if normalized_rpm >= effective_redline || normalized_rpm >= 0.94 {
            state.stage = 5;
            state.is_strobe_active = true;
            state.active_stage_color = state.strobe_color;
            state.show_shift_halo = true;
        } else if normalized_rpm >= 0.90 {
            state.stage = 4;
            state.is_strobe_active = false;
            state.active_stage_color = Rgba::CRIMSON_SHIFT;
            state.show_shift_halo = true;
        } else if normalized_rpm >= 0.82 {
            state.stage = 3;
            state.is_strobe_active = false;
            state.active_stage_color = Rgba::HIGH_VIS_AMBER;
            state.show_shift_halo = false;
        } else if normalized_rpm >= 0.72 || (shift_start_pct > 0.01 && normalized_rpm >= shift_start_pct) {
            state.stage = 2;
            state.is_strobe_active = false;
            state.active_stage_color = Rgba::MOTORSPORT_GREEN;
            state.show_shift_halo = false;
        } else {
            state.stage = 1;
            state.is_strobe_active = false;
            state.active_stage_color = brand_accent.unwrap_or(Rgba::TRANSPARENT);
            state.show_shift_halo = false;
        }

        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_shift_start_072_redline_094() {
        let brand = Some(Rgba {
            r: 10,
            g: 20,
            b: 30,
            a: 255,
        });
        let cases = [
            (0.50, 1, false, false),
            (0.72, 2, false, false),
            (0.82, 3, false, false),
            (0.90, 4, false, true),
            (0.94, 5, true, true),
        ];
        for (rpm, stage, strobe, halo) in cases {
            let s = RevLimiterSystem::evaluate(rpm, 0.72, 0.94, brand);
            assert_eq!(s.stage, stage, "rpm {rpm}");
            assert_eq!(s.is_strobe_active, strobe, "rpm {rpm}");
            assert_eq!(s.show_shift_halo, halo, "rpm {rpm}");
        }
    }

    #[test]
    fn custom_shift_start() {
        // shift_start_pct is OR with 0.72, so 0.72 threshold still applies
        let s = RevLimiterSystem::evaluate(0.75, 0.75, 0.94, None);
        assert_eq!(s.stage, 2);
        let s2 = RevLimiterSystem::evaluate(0.74, 0.75, 0.94, None);
        // 0.74 >= 0.72 so stage 2 despite shift_start 0.75
        assert_eq!(s2.stage, 2);
        // With a lower custom start, it still triggers at 0.72
        let s3 = RevLimiterSystem::evaluate(0.71, 0.65, 0.94, None);
        assert_eq!(s3.stage, 2);
    }

    #[test]
    fn redline_override() {
        let s = RevLimiterSystem::evaluate(0.93, 0.72, 0.90, None);
        // effective redline 0.90, so 0.93 >=0.90 => stage 4? Actually 0.93>=0.94? No. >=0.90 => stage4
        // But condition also checks normalized >= effective_redline => 0.93>=0.90 => stage5? Wait code checks >=effective_redline OR >=0.94. So 0.93>=0.90 true => stage5
        assert_eq!(s.stage, 5);
    }

    #[test]
    fn brand_accent_used_at_stage1() {
        let brand = Rgba {
            r: 1,
            g: 2,
            b: 3,
            a: 255,
        };
        let s = RevLimiterSystem::evaluate(0.5, 0.72, 0.94, Some(brand));
        assert_eq!(s.active_stage_color, brand);
    }
}
