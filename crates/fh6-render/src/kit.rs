pub struct DesignKit;

impl DesignKit {
    pub const GRID_UNIT: f32 = 8.0;
    pub const CAP_HEIGHT_RATIO: f32 = 0.72;
    pub const TABULAR_PITCH_FACTOR: f32 = 0.62;
    pub const MICRO_LABEL_TRACKING_PX: f32 = 2.2;
    pub const MICRO_LABEL_ALPHA: u8 = 145;

    pub fn snap8(v: f32, scale: f32) -> f32 {
        let unit = Self::GRID_UNIT * if scale > 0.01 { scale } else { 1.0 };
        (v / unit).round() * unit
    }
}

pub mod colors {
    pub const REV_CYAN: u32 = 0x00E5FF;
    pub const REV_GREEN: u32 = 0x00E676;
    pub const REV_AMBER: u32 = 0xFFB300;
    pub const REV_CRIMSON: u32 = 0xFF1744;
    pub const THROTTLE_GREEN: u32 = 0x10B981;
    pub const BRAKE_CRIMSON: u32 = 0xEF4444;
    pub const CLUTCH_CYAN: u32 = 0x38BDF8;
    pub const ABS_YELLOW: u32 = 0xFACC15;
    pub const TEXT_PRIMARY: u32 = 0xF8FAFC;
    pub const TEXT_MUTED: u32 = 0x8E9BAE;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snap8_grid() {
        let v = DesignKit::snap8(13.0, 1.0);
        assert_eq!(v, 16.0);
    }
}
