//! Telemetry data structures and the 324-byte little-endian Forza UDP packet decoder.
//! Field offsets are fixed by the game's packet format; check any change against
//! `tests/golden/packets` (97% rule).
//!
//! `decode` reads every field of the official FH6 Data Out list (support.forza.net article
//! 51744149102611), at the offsets its field order gives: the 88 fields fill bytes 0 to 322. The
//! page states 324 bytes but its fields add up to 323, so byte 323 is read by nothing. The golden
//! packets cover only the fields `mock-udp` writes; the rest wait on real-game check C.
//!
//! [`decode`] is the one place hostile floats are handled: every float it returns is finite and
//! within `±`[`MAX_ABS`], so nothing downstream (the smoother, the dashes, the OSD) needs its own
//! NaN guard for packet data. Zero and tiny values pass through as sent (a corrupt packet becomes
//! zeros, not a rejected packet), so a division by a packet value still needs its own guard:
//! `max_rpm` can be 0 and `idle_rpm` can exceed it.

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize)]
pub struct FH6TelemetryData {
    pub is_race_on: i32,
    pub timestamp_ms: u32,
    pub engine_max_rpm: f32,
    pub engine_idle_rpm: f32,
    pub current_engine_rpm: f32,
    pub acceleration_x: f32,
    pub acceleration_y: f32,
    pub acceleration_z: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub angular_velocity_y: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub tire_slip_angle_fl: f32,
    pub tire_slip_angle_fr: f32,
    pub tire_slip_angle_rl: f32,
    pub tire_slip_angle_rr: f32,
    pub normalized_suspension_travel_fl: f32,
    pub normalized_suspension_travel_fr: f32,
    pub normalized_suspension_travel_rl: f32,
    pub normalized_suspension_travel_rr: f32,
    pub suspension_travel_meters_fl: f32,
    pub suspension_travel_meters_fr: f32,
    pub suspension_travel_meters_rl: f32,
    pub suspension_travel_meters_rr: f32,
    pub car_ordinal: i32,
    pub drivetrain_type: i32,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub speed_ms: f32,
    pub power_watts: f32,
    pub boost_psi: f32,
    pub accel: u8,
    pub brake: u8,
    pub clutch: u8,
    pub hand_brake: u8,
    pub gear: u8,
    pub steer: i8,
    // The fields below complete the official FH6 Data Out list (support.forza.net article
    // 51744149102611). The 20 golden packets hold 0 in all of them, so their offsets follow the
    // documented field order only; real-game check C confirms them on a capture. Units the page
    // does not state are not assumed.
    /// `AngularVelocityX` (44), about the car's x axis.
    pub angular_velocity_x: f32,
    /// `AngularVelocityZ` (52), about the car's z axis.
    pub angular_velocity_z: f32,
    /// `TireSlipRatio*` (84-96).
    pub tire_slip_ratio_fl: f32,
    pub tire_slip_ratio_fr: f32,
    pub tire_slip_ratio_rl: f32,
    pub tire_slip_ratio_rr: f32,
    /// `WheelRotationSpeed*` (100-112).
    pub wheel_rotation_speed_fl: f32,
    pub wheel_rotation_speed_fr: f32,
    pub wheel_rotation_speed_rl: f32,
    pub wheel_rotation_speed_rr: f32,
    /// `WheelOnRumbleStrip*` (116-128): 1 on a rumble strip, 0 off.
    pub wheel_on_rumble_strip_fl: i32,
    pub wheel_on_rumble_strip_fr: i32,
    pub wheel_on_rumble_strip_rl: i32,
    pub wheel_on_rumble_strip_rr: i32,
    /// `WheelInPuddle*` (132-144): 1 in a puddle, 0 out. A flag, not a depth.
    pub wheel_in_puddle_fl: i32,
    pub wheel_in_puddle_fr: i32,
    pub wheel_in_puddle_rl: i32,
    pub wheel_in_puddle_rr: i32,
    /// `SurfaceRumble*` (148-160): the force-feedback rumble of the surface under each wheel.
    pub surface_rumble_fl: f32,
    pub surface_rumble_fr: f32,
    pub surface_rumble_rl: f32,
    pub surface_rumble_rr: f32,
    /// `TireCombinedSlip*` (180-192).
    pub tire_combined_slip_fl: f32,
    pub tire_combined_slip_fr: f32,
    pub tire_combined_slip_rl: f32,
    pub tire_combined_slip_rr: f32,
    /// `CarClass` (216): a class index. Its mapping to the class letters is unverified.
    pub car_class: i32,
    /// `CarPerformanceIndex` (220): the PI number.
    pub car_performance_index: i32,
    /// `NumCylinders` (228).
    pub num_cylinders: i32,
    /// `CarGroup` (232), Horizon only. The page does not list the values.
    pub car_group: u32,
    /// `SmashableVelDiff` (236), Horizon only. Its meaning and unit are unverified.
    pub smashable_vel_diff: f32,
    /// `SmashableMass` (240), Horizon only. Its meaning and unit are unverified.
    pub smashable_mass: f32,
    /// `Torque` (264). The page states no unit (Forza Motorsport sends N·m).
    pub torque: f32,
    /// `TireTemp*` (268-280). The page states no unit (Forza titles have sent Fahrenheit).
    pub tire_temp_fl: f32,
    pub tire_temp_fr: f32,
    pub tire_temp_rl: f32,
    pub tire_temp_rr: f32,
    /// `Fuel` (288).
    pub fuel: f32,
    /// `DistanceTraveled` (292).
    pub distance_traveled: f32,
    /// `BestLap` (296).
    pub best_lap: f32,
    /// `LastLap` (300).
    pub last_lap: f32,
    /// `CurrentLap` (304).
    pub current_lap: f32,
    /// `CurrentRaceTime` (308).
    pub current_race_time: f32,
    /// `LapNumber` (312).
    pub lap_number: u16,
    /// `RacePosition` (314).
    pub race_position: u8,
    /// `NormalizedDrivingLine` (321).
    pub normalized_driving_line: i8,
    /// `NormalizedAIBrakeDifference` (322).
    pub normalized_ai_brake_difference: i8,
}

impl FH6TelemetryData {
    pub fn speed_mph(&self) -> f32 {
        self.speed_ms * 2.2369363_f32
    }
    pub fn speed_kmh(&self) -> f32 {
        self.speed_ms * 3.6
    }
    pub fn lateral_g(&self) -> f32 {
        self.acceleration_x / 9.80665
    }
    pub fn longitudinal_g(&self) -> f32 {
        self.acceleration_z / 9.80665
    }
    pub fn throttle_pct(&self) -> f32 {
        self.accel as f32 / 255.0
    }
    pub fn brake_pct(&self) -> f32 {
        self.brake as f32 / 255.0
    }
    pub fn clutch_pct(&self) -> f32 {
        self.clutch as f32 / 255.0
    }
    pub fn handbrake_pct(&self) -> f32 {
        self.hand_brake as f32 / 255.0
    }
    pub fn yaw_curvature(&self) -> f32 {
        let v = self.speed_ms.max(2.0);
        self.angular_velocity_y / v
    }
    pub fn car_slip_angle_rad(&self) -> f32 {
        if self.speed_ms <= 1.0 {
            return 0.0;
        }
        let raw = (self.velocity_x).atan2(self.velocity_z);
        if self.speed_ms < 3.0 {
            raw * (self.speed_ms - 1.0) / 2.0
        } else {
            raw
        }
    }
    pub fn car_slip_angle_deg(&self) -> f32 {
        self.car_slip_angle_rad() * 180.0 / std::f32::consts::PI
    }
    pub fn rear_tire_slip_angle_rad(&self) -> f32 {
        (self.tire_slip_angle_rl + self.tire_slip_angle_rr) * 0.5
    }
    /// How many wheels the game flags as in a puddle (0 to 4). Any non-zero flag counts.
    pub fn wheels_in_puddle(&self) -> u8 {
        [
            self.wheel_in_puddle_fl,
            self.wheel_in_puddle_fr,
            self.wheel_in_puddle_rl,
            self.wheel_in_puddle_rr,
        ]
        .into_iter()
        .map(|f| u8::from(f != 0))
        .sum()
    }
    /// How many wheels the game flags as on a rumble strip (0 to 4). Any non-zero flag counts.
    pub fn wheels_on_rumble_strip(&self) -> u8 {
        [
            self.wheel_on_rumble_strip_fl,
            self.wheel_on_rumble_strip_fr,
            self.wheel_on_rumble_strip_rl,
            self.wheel_on_rumble_strip_rr,
        ]
        .into_iter()
        .map(|f| u8::from(f != 0))
        .sum()
    }
    pub fn gear_string(&self) -> String {
        if self.gear == 0 {
            "R".to_string()
        } else if self.gear > 10 {
            "N".to_string()
        } else {
            self.gear.to_string()
        }
    }
}

/// Length of a Forza "Data Out" packet in bytes.
pub const PACKET_LEN: usize = 324;

/// Why a datagram could not be decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TelemetryError {
    /// The datagram is not exactly [`PACKET_LEN`] bytes.
    #[error("invalid telemetry packet length {0} (expected 324)")]
    InvalidLength(usize),
}

/// The largest magnitude a decoded float keeps. Real readings sit far below it (rpm about 1e4,
/// power about 2e6 W, world position about 1e4 m), so anything larger is a corrupt packet. The
/// bound keeps a dash's products finite: `MAX_ABS` cubed is about 1e21, far inside `f32`.
pub const MAX_ABS: f32 = 1.0e7;

/// A packet float: NaN and the infinities become 0.0, and the rest are clamped to
/// `±`[`MAX_ABS`]. Values inside the bound come through bit for bit.
fn le_f32(b: &[u8; PACKET_LEN], off: usize) -> f32 {
    let v = f32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]]);
    if v.is_finite() {
        v.clamp(-MAX_ABS, MAX_ABS)
    } else {
        0.0
    }
}

fn le_i32(b: &[u8; PACKET_LEN], off: usize) -> i32 {
    i32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

fn le_u32(b: &[u8; PACKET_LEN], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

fn le_u16(b: &[u8; PACKET_LEN], off: usize) -> u16 {
    u16::from_le_bytes([b[off], b[off + 1]])
}

/// Decodes one 324-byte little-endian packet. Every float in the result is finite with
/// `|v| <= `[`MAX_ABS`] (see [`le_f32`]); `CurrentEngineRpm` is also clamped to at least 0.
/// Zeros, subnormals and inconsistent readings (idle above max) come through unchanged.
///
/// # Errors
/// [`TelemetryError::InvalidLength`] if `buf` is not exactly 324 bytes.
pub fn decode(buf: &[u8]) -> Result<FH6TelemetryData, TelemetryError> {
    let b: &[u8; PACKET_LEN] = buf.try_into().map_err(|_| TelemetryError::InvalidLength(buf.len()))?;
    // Fixed little-endian offsets of the 324-byte packet (golden-tested).
    Ok(FH6TelemetryData {
        is_race_on: le_i32(b, 0),
        timestamp_ms: le_u32(b, 4),
        engine_max_rpm: le_f32(b, 8),
        engine_idle_rpm: le_f32(b, 12),
        current_engine_rpm: le_f32(b, 16).max(0.0),
        acceleration_x: le_f32(b, 20),
        acceleration_y: le_f32(b, 24),
        acceleration_z: le_f32(b, 28),
        velocity_x: le_f32(b, 32),
        velocity_y: le_f32(b, 36),
        velocity_z: le_f32(b, 40),
        angular_velocity_y: le_f32(b, 48),
        yaw: le_f32(b, 56),
        pitch: le_f32(b, 60),
        roll: le_f32(b, 64),
        normalized_suspension_travel_fl: le_f32(b, 68),
        normalized_suspension_travel_fr: le_f32(b, 72),
        normalized_suspension_travel_rl: le_f32(b, 76),
        normalized_suspension_travel_rr: le_f32(b, 80),
        tire_slip_angle_fl: le_f32(b, 164),
        tire_slip_angle_fr: le_f32(b, 168),
        tire_slip_angle_rl: le_f32(b, 172),
        tire_slip_angle_rr: le_f32(b, 176),
        suspension_travel_meters_fl: le_f32(b, 196),
        suspension_travel_meters_fr: le_f32(b, 200),
        suspension_travel_meters_rl: le_f32(b, 204),
        suspension_travel_meters_rr: le_f32(b, 208),
        car_ordinal: le_i32(b, 212),
        drivetrain_type: le_i32(b, 224),
        position_x: le_f32(b, 244),
        position_y: le_f32(b, 248),
        position_z: le_f32(b, 252),
        speed_ms: le_f32(b, 256),
        power_watts: le_f32(b, 260),
        boost_psi: le_f32(b, 284),
        accel: b[315],
        brake: b[316],
        clutch: b[317],
        hand_brake: b[318],
        gear: b[319],
        steer: i8::from_le_bytes([b[320]]),
        angular_velocity_x: le_f32(b, 44),
        angular_velocity_z: le_f32(b, 52),
        tire_slip_ratio_fl: le_f32(b, 84),
        tire_slip_ratio_fr: le_f32(b, 88),
        tire_slip_ratio_rl: le_f32(b, 92),
        tire_slip_ratio_rr: le_f32(b, 96),
        wheel_rotation_speed_fl: le_f32(b, 100),
        wheel_rotation_speed_fr: le_f32(b, 104),
        wheel_rotation_speed_rl: le_f32(b, 108),
        wheel_rotation_speed_rr: le_f32(b, 112),
        wheel_on_rumble_strip_fl: le_i32(b, 116),
        wheel_on_rumble_strip_fr: le_i32(b, 120),
        wheel_on_rumble_strip_rl: le_i32(b, 124),
        wheel_on_rumble_strip_rr: le_i32(b, 128),
        wheel_in_puddle_fl: le_i32(b, 132),
        wheel_in_puddle_fr: le_i32(b, 136),
        wheel_in_puddle_rl: le_i32(b, 140),
        wheel_in_puddle_rr: le_i32(b, 144),
        surface_rumble_fl: le_f32(b, 148),
        surface_rumble_fr: le_f32(b, 152),
        surface_rumble_rl: le_f32(b, 156),
        surface_rumble_rr: le_f32(b, 160),
        tire_combined_slip_fl: le_f32(b, 180),
        tire_combined_slip_fr: le_f32(b, 184),
        tire_combined_slip_rl: le_f32(b, 188),
        tire_combined_slip_rr: le_f32(b, 192),
        car_class: le_i32(b, 216),
        car_performance_index: le_i32(b, 220),
        num_cylinders: le_i32(b, 228),
        car_group: le_u32(b, 232),
        smashable_vel_diff: le_f32(b, 236),
        smashable_mass: le_f32(b, 240),
        torque: le_f32(b, 264),
        tire_temp_fl: le_f32(b, 268),
        tire_temp_fr: le_f32(b, 272),
        tire_temp_rl: le_f32(b, 276),
        tire_temp_rr: le_f32(b, 280),
        fuel: le_f32(b, 288),
        distance_traveled: le_f32(b, 292),
        best_lap: le_f32(b, 296),
        last_lap: le_f32(b, 300),
        current_lap: le_f32(b, 304),
        current_race_time: le_f32(b, 308),
        lap_number: le_u16(b, 312),
        race_position: b[314],
        normalized_driving_line: i8::from_le_bytes([b[321]]),
        normalized_ai_brake_difference: i8::from_le_bytes([b[322]]),
    })
}

/// Encodes `t` as the game would send it: the inverse of [`decode`] for every packet whose
/// floats are within `±`[`MAX_ABS`] and whose rpm is not negative. Byte 323, which `decode`
/// does not read, is zero. For tools and tests (`mock-udp`-style packets, the robustness gate's byte sweeps).
#[must_use]
pub fn encode(t: &FH6TelemetryData) -> [u8; PACKET_LEN] {
    let mut b = [0u8; PACKET_LEN];
    let mut put = |off: usize, bytes: [u8; 4]| b[off..off + 4].copy_from_slice(&bytes);
    put(0, t.is_race_on.to_le_bytes());
    put(4, t.timestamp_ms.to_le_bytes());
    for (off, v) in [
        (8, t.engine_max_rpm),
        (12, t.engine_idle_rpm),
        (16, t.current_engine_rpm),
        (20, t.acceleration_x),
        (24, t.acceleration_y),
        (28, t.acceleration_z),
        (32, t.velocity_x),
        (36, t.velocity_y),
        (40, t.velocity_z),
        (48, t.angular_velocity_y),
        (56, t.yaw),
        (60, t.pitch),
        (64, t.roll),
        (68, t.normalized_suspension_travel_fl),
        (72, t.normalized_suspension_travel_fr),
        (76, t.normalized_suspension_travel_rl),
        (80, t.normalized_suspension_travel_rr),
        (164, t.tire_slip_angle_fl),
        (168, t.tire_slip_angle_fr),
        (172, t.tire_slip_angle_rl),
        (176, t.tire_slip_angle_rr),
        (196, t.suspension_travel_meters_fl),
        (200, t.suspension_travel_meters_fr),
        (204, t.suspension_travel_meters_rl),
        (208, t.suspension_travel_meters_rr),
        (244, t.position_x),
        (248, t.position_y),
        (252, t.position_z),
        (256, t.speed_ms),
        (260, t.power_watts),
        (284, t.boost_psi),
        (44, t.angular_velocity_x),
        (52, t.angular_velocity_z),
        (84, t.tire_slip_ratio_fl),
        (88, t.tire_slip_ratio_fr),
        (92, t.tire_slip_ratio_rl),
        (96, t.tire_slip_ratio_rr),
        (100, t.wheel_rotation_speed_fl),
        (104, t.wheel_rotation_speed_fr),
        (108, t.wheel_rotation_speed_rl),
        (112, t.wheel_rotation_speed_rr),
        (148, t.surface_rumble_fl),
        (152, t.surface_rumble_fr),
        (156, t.surface_rumble_rl),
        (160, t.surface_rumble_rr),
        (180, t.tire_combined_slip_fl),
        (184, t.tire_combined_slip_fr),
        (188, t.tire_combined_slip_rl),
        (192, t.tire_combined_slip_rr),
        (236, t.smashable_vel_diff),
        (240, t.smashable_mass),
        (264, t.torque),
        (268, t.tire_temp_fl),
        (272, t.tire_temp_fr),
        (276, t.tire_temp_rl),
        (280, t.tire_temp_rr),
        (288, t.fuel),
        (292, t.distance_traveled),
        (296, t.best_lap),
        (300, t.last_lap),
        (304, t.current_lap),
        (308, t.current_race_time),
    ] {
        put(off, v.to_le_bytes());
    }
    put(212, t.car_ordinal.to_le_bytes());
    put(224, t.drivetrain_type.to_le_bytes());
    for (off, v) in [
        (116, t.wheel_on_rumble_strip_fl),
        (120, t.wheel_on_rumble_strip_fr),
        (124, t.wheel_on_rumble_strip_rl),
        (128, t.wheel_on_rumble_strip_rr),
        (132, t.wheel_in_puddle_fl),
        (136, t.wheel_in_puddle_fr),
        (140, t.wheel_in_puddle_rl),
        (144, t.wheel_in_puddle_rr),
        (216, t.car_class),
        (220, t.car_performance_index),
        (228, t.num_cylinders),
    ] {
        put(off, v.to_le_bytes());
    }
    put(232, t.car_group.to_le_bytes());
    b[312..314].copy_from_slice(&t.lap_number.to_le_bytes());
    b[314] = t.race_position;
    b[315] = t.accel;
    b[316] = t.brake;
    b[317] = t.clutch;
    b[318] = t.hand_brake;
    b[319] = t.gear;
    b[320] = t.steer.to_le_bytes()[0];
    b[321] = t.normalized_driving_line.to_le_bytes()[0];
    b[322] = t.normalized_ai_brake_difference.to_le_bytes()[0];
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_invalid_length() {
        let buf = vec![0u8; 100];
        assert!(matches!(decode(&buf), Err(TelemetryError::InvalidLength(100))));
    }

    #[test]
    fn decode_zero_packet_fields() {
        let buf = vec![0u8; PACKET_LEN];
        let data = decode(&buf).expect("324 zero bytes decode");
        assert_eq!(data.is_race_on, 0);
        assert_eq!(data.timestamp_ms, 0);
        assert_eq!(data.speed_ms, 0.0);
        assert_eq!(data.accel, 0);
        assert_eq!(data.steer, 0);
    }

    #[test]
    fn decode_reads_signed_steer_and_negative_rpm_clamped() {
        let mut buf = [0u8; PACKET_LEN];
        buf[16..20].copy_from_slice(&(-50.0f32).to_le_bytes());
        buf[320] = 0x80;
        let data = decode(&buf).expect("decode");
        assert_eq!(data.current_engine_rpm, 0.0);
        assert_eq!(data.steer, -128);
    }

    /// Every float field, named so a new field fails to compile here until it is listed.
    fn floats(t: &FH6TelemetryData) -> Vec<(&'static str, f32)> {
        let FH6TelemetryData {
            is_race_on: _,
            timestamp_ms: _,
            engine_max_rpm,
            engine_idle_rpm,
            current_engine_rpm,
            acceleration_x,
            acceleration_y,
            acceleration_z,
            velocity_x,
            velocity_y,
            velocity_z,
            angular_velocity_y,
            yaw,
            pitch,
            roll,
            tire_slip_angle_fl,
            tire_slip_angle_fr,
            tire_slip_angle_rl,
            tire_slip_angle_rr,
            normalized_suspension_travel_fl,
            normalized_suspension_travel_fr,
            normalized_suspension_travel_rl,
            normalized_suspension_travel_rr,
            suspension_travel_meters_fl,
            suspension_travel_meters_fr,
            suspension_travel_meters_rl,
            suspension_travel_meters_rr,
            car_ordinal: _,
            drivetrain_type: _,
            position_x,
            position_y,
            position_z,
            speed_ms,
            power_watts,
            boost_psi,
            accel: _,
            brake: _,
            clutch: _,
            hand_brake: _,
            gear: _,
            steer: _,
            angular_velocity_x,
            angular_velocity_z,
            tire_slip_ratio_fl,
            tire_slip_ratio_fr,
            tire_slip_ratio_rl,
            tire_slip_ratio_rr,
            wheel_rotation_speed_fl,
            wheel_rotation_speed_fr,
            wheel_rotation_speed_rl,
            wheel_rotation_speed_rr,
            wheel_on_rumble_strip_fl: _,
            wheel_on_rumble_strip_fr: _,
            wheel_on_rumble_strip_rl: _,
            wheel_on_rumble_strip_rr: _,
            wheel_in_puddle_fl: _,
            wheel_in_puddle_fr: _,
            wheel_in_puddle_rl: _,
            wheel_in_puddle_rr: _,
            surface_rumble_fl,
            surface_rumble_fr,
            surface_rumble_rl,
            surface_rumble_rr,
            tire_combined_slip_fl,
            tire_combined_slip_fr,
            tire_combined_slip_rl,
            tire_combined_slip_rr,
            car_class: _,
            car_performance_index: _,
            num_cylinders: _,
            car_group: _,
            smashable_vel_diff,
            smashable_mass,
            torque,
            tire_temp_fl,
            tire_temp_fr,
            tire_temp_rl,
            tire_temp_rr,
            fuel,
            distance_traveled,
            best_lap,
            last_lap,
            current_lap,
            current_race_time,
            lap_number: _,
            race_position: _,
            normalized_driving_line: _,
            normalized_ai_brake_difference: _,
        } = *t;
        vec![
            ("engine_max_rpm", engine_max_rpm),
            ("engine_idle_rpm", engine_idle_rpm),
            ("current_engine_rpm", current_engine_rpm),
            ("acceleration_x", acceleration_x),
            ("acceleration_y", acceleration_y),
            ("acceleration_z", acceleration_z),
            ("velocity_x", velocity_x),
            ("velocity_y", velocity_y),
            ("velocity_z", velocity_z),
            ("angular_velocity_y", angular_velocity_y),
            ("yaw", yaw),
            ("pitch", pitch),
            ("roll", roll),
            ("tire_slip_angle_fl", tire_slip_angle_fl),
            ("tire_slip_angle_fr", tire_slip_angle_fr),
            ("tire_slip_angle_rl", tire_slip_angle_rl),
            ("tire_slip_angle_rr", tire_slip_angle_rr),
            ("normalized_suspension_travel_fl", normalized_suspension_travel_fl),
            ("normalized_suspension_travel_fr", normalized_suspension_travel_fr),
            ("normalized_suspension_travel_rl", normalized_suspension_travel_rl),
            ("normalized_suspension_travel_rr", normalized_suspension_travel_rr),
            ("suspension_travel_meters_fl", suspension_travel_meters_fl),
            ("suspension_travel_meters_fr", suspension_travel_meters_fr),
            ("suspension_travel_meters_rl", suspension_travel_meters_rl),
            ("suspension_travel_meters_rr", suspension_travel_meters_rr),
            ("position_x", position_x),
            ("position_y", position_y),
            ("position_z", position_z),
            ("speed_ms", speed_ms),
            ("power_watts", power_watts),
            ("boost_psi", boost_psi),
            ("angular_velocity_x", angular_velocity_x),
            ("angular_velocity_z", angular_velocity_z),
            ("tire_slip_ratio_fl", tire_slip_ratio_fl),
            ("tire_slip_ratio_fr", tire_slip_ratio_fr),
            ("tire_slip_ratio_rl", tire_slip_ratio_rl),
            ("tire_slip_ratio_rr", tire_slip_ratio_rr),
            ("wheel_rotation_speed_fl", wheel_rotation_speed_fl),
            ("wheel_rotation_speed_fr", wheel_rotation_speed_fr),
            ("wheel_rotation_speed_rl", wheel_rotation_speed_rl),
            ("wheel_rotation_speed_rr", wheel_rotation_speed_rr),
            ("surface_rumble_fl", surface_rumble_fl),
            ("surface_rumble_fr", surface_rumble_fr),
            ("surface_rumble_rl", surface_rumble_rl),
            ("surface_rumble_rr", surface_rumble_rr),
            ("tire_combined_slip_fl", tire_combined_slip_fl),
            ("tire_combined_slip_fr", tire_combined_slip_fr),
            ("tire_combined_slip_rl", tire_combined_slip_rl),
            ("tire_combined_slip_rr", tire_combined_slip_rr),
            ("smashable_vel_diff", smashable_vel_diff),
            ("smashable_mass", smashable_mass),
            ("torque", torque),
            ("tire_temp_fl", tire_temp_fl),
            ("tire_temp_fr", tire_temp_fr),
            ("tire_temp_rl", tire_temp_rl),
            ("tire_temp_rr", tire_temp_rr),
            ("fuel", fuel),
            ("distance_traveled", distance_traveled),
            ("best_lap", best_lap),
            ("last_lap", last_lap),
            ("current_lap", current_lap),
            ("current_race_time", current_race_time),
        ]
    }

    /// Gate G5 at the decoder: whatever the bytes, every float comes out finite and bounded.
    #[test]
    fn hostile_bytes_decode_to_finite_bounded_floats() {
        let words = [
            f32::NAN,
            -f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::MAX,
            f32::MIN,
            f32::MIN_POSITIVE,
            f32::from_bits(1),
            MAX_ABS * 2.0,
        ];
        let mut packets: Vec<Vec<u8>> = words
            .iter()
            .map(|w| w.to_le_bytes().iter().copied().cycle().take(PACKET_LEN).collect())
            .collect();
        packets.extend([0x00u8, 0xFF, 0x7F, 0x80].map(|b| vec![b; PACKET_LEN]));
        let mut x: u64 = 0x2545_F491_4F6C_DD1D;
        for _ in 0..64 {
            packets.push(
                (0..PACKET_LEN)
                    .map(|_| {
                        x ^= x << 13;
                        x ^= x >> 7;
                        x ^= x << 17;
                        x.to_le_bytes()[7]
                    })
                    .collect(),
            );
        }
        for (i, bytes) in packets.iter().enumerate() {
            let data = decode(bytes).expect("324 bytes decode");
            for (name, v) in floats(&data) {
                assert!(v.is_finite() && v.abs() <= MAX_ABS, "packet {i}: {name} = {v}");
            }
            assert!(data.current_engine_rpm >= 0.0, "packet {i}: negative rpm");
        }
    }

    #[test]
    fn non_finite_floats_decode_as_zero_and_huge_ones_clamp() {
        let mut buf = [0u8; PACKET_LEN];
        buf[256..260].copy_from_slice(&f32::NAN.to_le_bytes()); // speed
        buf[260..264].copy_from_slice(&f32::INFINITY.to_le_bytes()); // power
        buf[284..288].copy_from_slice(&f32::NEG_INFINITY.to_le_bytes()); // boost
        buf[244..248].copy_from_slice(&1.0e30f32.to_le_bytes()); // position x
        buf[248..252].copy_from_slice(&(-1.0e30f32).to_le_bytes()); // position y
        let data = decode(&buf).expect("decode");
        assert_eq!(data.speed_ms, 0.0);
        assert_eq!(data.power_watts, 0.0);
        assert_eq!(data.boost_psi, 0.0);
        assert_eq!(data.position_x, MAX_ABS);
        assert_eq!(data.position_y, -MAX_ABS);
    }

    /// Byte offsets of the 62 floats `decode` reads, in the order [`floats`] lists them.
    const FLOAT_OFFSETS: [usize; 62] = [
        8, 12, 16, 20, 24, 28, 32, 36, 40, 48, 56, 60, 64, 68, 72, 76, 80, 164, 168, 172, 176, 196, 200, 204, 208, 244,
        248, 252, 256, 260, 284, 44, 52, 84, 88, 92, 96, 100, 104, 108, 112, 148, 152, 156, 160, 180, 184, 188, 192,
        236, 240, 264, 268, 272, 276, 280, 288, 292, 296, 300, 304, 308,
    ];

    /// Byte offsets of the 4-byte integers `decode` reads.
    const INT_OFFSETS: [usize; 16] = [
        0, 4, 116, 120, 124, 128, 132, 136, 140, 144, 212, 216, 220, 224, 228, 232,
    ];

    /// A packet with every field `decode` reads set to its own value and every other byte zero.
    fn distinct_bytes() -> [u8; PACKET_LEN] {
        let mut b = [0u8; PACKET_LEN];
        for (i, off) in FLOAT_OFFSETS.into_iter().enumerate() {
            b[off..off + 4].copy_from_slice(&(100.0 + i as f32 * 1.25).to_le_bytes());
        }
        b[0..4].copy_from_slice(&1i32.to_le_bytes());
        b[4..8].copy_from_slice(&123_456_789u32.to_le_bytes());
        b[212..216].copy_from_slice(&(-2_200i32).to_le_bytes());
        b[224..228].copy_from_slice(&2i32.to_le_bytes());
        for (i, off) in [116, 120, 124, 128, 132, 136, 140, 144].into_iter().enumerate() {
            b[off..off + 4].copy_from_slice(&(i as i32 + 1).to_le_bytes());
        }
        b[216..220].copy_from_slice(&5i32.to_le_bytes());
        b[220..224].copy_from_slice(&897i32.to_le_bytes());
        b[228..232].copy_from_slice(&8i32.to_le_bytes());
        b[232..236].copy_from_slice(&0x8000_0011u32.to_le_bytes());
        b[312..314].copy_from_slice(&0x0302u16.to_le_bytes());
        b[314] = 6;
        b[315..323].copy_from_slice(&[201, 202, 203, 204, 7, (-77i8).to_le_bytes()[0], 127, 0x81]);
        b
    }

    /// Every byte but 323 belongs to exactly one decoded field: the documented list is complete.
    #[test]
    fn decoded_fields_tile_bytes_0_to_322() {
        let mut owner = [0u8; PACKET_LEN];
        let fields = FLOAT_OFFSETS
            .into_iter()
            .chain(INT_OFFSETS)
            .map(|off| off..off + 4)
            .chain(std::iter::once(312..314)) // LapNumber (U16)
            .chain((314..323).map(|off| off..off + 1)); // RacePosition to NormalizedAIBrakeDifference
        for range in fields {
            for o in &mut owner[range] {
                *o += 1;
            }
        }
        let wrong: Vec<usize> = (0..PACKET_LEN - 1).filter(|&i| owner[i] != 1).collect();
        assert!(wrong.is_empty(), "bytes read by no field or by two: {wrong:?}");
        assert_eq!(owner[PACKET_LEN - 1], 0);
    }

    /// Crafted packets for the fields the 20 synthetic goldens leave at 0 (they hold 0 there).
    #[test]
    fn decodes_the_documented_fields_the_goldens_leave_at_zero() {
        let mut b = [0u8; PACKET_LEN];
        b[44..48].copy_from_slice(&0.25f32.to_le_bytes());
        b[52..56].copy_from_slice(&(-0.5f32).to_le_bytes());
        b[84..88].copy_from_slice(&1.5f32.to_le_bytes());
        b[100..104].copy_from_slice(&88.0f32.to_le_bytes());
        b[116..120].copy_from_slice(&1i32.to_le_bytes()); // rumble strip FL
        b[136..140].copy_from_slice(&1i32.to_le_bytes()); // puddle FR
        b[144..148].copy_from_slice(&1i32.to_le_bytes()); // puddle RR
        b[148..152].copy_from_slice(&0.75f32.to_le_bytes());
        b[192..196].copy_from_slice(&2.0f32.to_le_bytes());
        b[216..220].copy_from_slice(&4i32.to_le_bytes()); // CarClass
        b[220..224].copy_from_slice(&800i32.to_le_bytes()); // CarPerformanceIndex
        b[228..232].copy_from_slice(&12i32.to_le_bytes()); // NumCylinders
        b[232..236].copy_from_slice(&3u32.to_le_bytes()); // CarGroup
        b[236..240].copy_from_slice(&4.5f32.to_le_bytes());
        b[240..244].copy_from_slice(&120.0f32.to_le_bytes());
        b[264..268].copy_from_slice(&720.0f32.to_le_bytes()); // Torque
        b[280..284].copy_from_slice(&185.0f32.to_le_bytes()); // TireTemp RR
        b[288..292].copy_from_slice(&0.6f32.to_le_bytes());
        b[292..296].copy_from_slice(&15_000.0f32.to_le_bytes());
        b[296..300].copy_from_slice(&92.5f32.to_le_bytes());
        b[300..304].copy_from_slice(&93.25f32.to_le_bytes());
        b[304..308].copy_from_slice(&12.0f32.to_le_bytes());
        b[308..312].copy_from_slice(&197.75f32.to_le_bytes());
        b[312..314].copy_from_slice(&3u16.to_le_bytes());
        b[314] = 2;
        b[321] = (-64i8).to_le_bytes()[0];
        b[322] = 100;
        let t = decode(&b).expect("decode");
        assert_eq!((t.angular_velocity_x, t.angular_velocity_z), (0.25, -0.5));
        assert_eq!((t.tire_slip_ratio_fl, t.wheel_rotation_speed_fl), (1.5, 88.0));
        assert_eq!((t.surface_rumble_fl, t.tire_combined_slip_rr), (0.75, 2.0));
        assert_eq!(t.wheel_on_rumble_strip_fl, 1);
        assert_eq!(t.wheels_on_rumble_strip(), 1);
        assert_eq!(
            [
                t.wheel_in_puddle_fl,
                t.wheel_in_puddle_fr,
                t.wheel_in_puddle_rl,
                t.wheel_in_puddle_rr
            ],
            [0, 1, 0, 1]
        );
        assert_eq!(t.wheels_in_puddle(), 2);
        assert_eq!((t.car_class, t.car_performance_index), (4, 800));
        assert_eq!((t.num_cylinders, t.car_group), (12, 3));
        assert_eq!((t.smashable_vel_diff, t.smashable_mass), (4.5, 120.0));
        assert_eq!((t.torque, t.tire_temp_rr, t.fuel), (720.0, 185.0, 0.6));
        assert_eq!(t.distance_traveled, 15_000.0);
        assert_eq!((t.best_lap, t.last_lap, t.current_lap), (92.5, 93.25, 12.0));
        assert_eq!(t.current_race_time, 197.75);
        assert_eq!((t.lap_number, t.race_position), (3, 2));
        assert_eq!(
            (t.normalized_driving_line, t.normalized_ai_brake_difference),
            (-64, 100)
        );
        // The fields beside them still read as before.
        assert_eq!((t.car_ordinal, t.drivetrain_type, t.boost_psi, t.accel), (0, 0, 0.0, 0));
    }

    /// A flag of any non-zero value counts as set; the game documents 1.
    #[test]
    fn wheel_counts_treat_any_non_zero_flag_as_set() {
        let t = FH6TelemetryData {
            wheel_in_puddle_fl: -1,
            wheel_in_puddle_fr: 7,
            wheel_in_puddle_rl: 1,
            wheel_in_puddle_rr: 1,
            ..FH6TelemetryData::default()
        };
        assert_eq!(t.wheels_in_puddle(), 4);
        assert_eq!(t.wheels_on_rumble_strip(), 0);
    }

    #[test]
    fn encode_is_the_inverse_of_decode() {
        let bytes = distinct_bytes();
        let t = decode(&bytes).expect("decode");
        assert_eq!(floats(&t).len(), FLOAT_OFFSETS.len());
        assert_eq!(encode(&t), bytes, "a field encoded at another field's offset");
        assert_eq!(decode(&encode(&t)), Ok(t));
        assert_eq!(encode(&FH6TelemetryData::default()), [0u8; PACKET_LEN]);
    }

    #[test]
    fn in_range_floats_decode_bit_for_bit_at_every_offset() {
        for v in [
            -MAX_ABS,
            -1.0e-30,
            -0.0,
            f32::from_bits(1),
            0.1,
            8_700.0,
            1.5e6,
            MAX_ABS,
        ] {
            let mut bytes = distinct_bytes();
            for off in FLOAT_OFFSETS {
                if off == 16 && v.is_sign_negative() {
                    continue; // rpm is clamped to at least 0
                }
                bytes[off..off + 4].copy_from_slice(&v.to_le_bytes());
            }
            let back = decode(&bytes).expect("decode");
            assert_eq!(encode(&back), bytes, "{v:e}");
        }
    }
}
