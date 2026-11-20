//! Telemetry data structures and the 324-byte little-endian Forza UDP packet decoder.
//! Field offsets are fixed by the game's packet format; check any change against
//! `tests/golden/packets` (97% rule).

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
    pub fn normalized_rpm(&self) -> f32 {
        if self.engine_max_rpm <= 500.0 {
            (self.power_watts / 1_400_000.0).clamp(0.0, 1.0)
        } else {
            let max = self.engine_max_rpm.max(1.0);
            (self.current_engine_rpm / max).clamp(0.0, 1.2)
        }
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

#[derive(Debug, Default)]
pub struct TelemetryEngine {
    pub latest: FH6TelemetryData,
}

#[derive(Debug)]
pub enum TelemetryError {
    InvalidLength(usize),
    Io(std::io::Error),
}

impl From<std::io::Error> for TelemetryError {
    fn from(e: std::io::Error) -> Self {
        TelemetryError::Io(e)
    }
}

pub fn decode(buf: &[u8]) -> Result<FH6TelemetryData, TelemetryError> {
    if buf.len() != 324 {
        return Err(TelemetryError::InvalidLength(buf.len()));
    }
    // Fixed little-endian offsets of the 324-byte packet.
    let is_race_on = i32::from_le_bytes(buf[0..4].try_into().unwrap());
    let timestamp_ms = u32::from_le_bytes(buf[4..8].try_into().unwrap());
    let engine_max_rpm = f32::from_le_bytes(buf[8..12].try_into().unwrap());
    let engine_idle_rpm = f32::from_le_bytes(buf[12..16].try_into().unwrap());
    let current_engine_rpm_raw = f32::from_le_bytes(buf[16..20].try_into().unwrap());
    let current_engine_rpm = current_engine_rpm_raw.max(0.0);
    let acceleration_x = f32::from_le_bytes(buf[20..24].try_into().unwrap());
    let acceleration_y = f32::from_le_bytes(buf[24..28].try_into().unwrap());
    let acceleration_z = f32::from_le_bytes(buf[28..32].try_into().unwrap());
    let velocity_x = f32::from_le_bytes(buf[32..36].try_into().unwrap());
    let velocity_y = f32::from_le_bytes(buf[36..40].try_into().unwrap());
    let velocity_z = f32::from_le_bytes(buf[40..44].try_into().unwrap());
    // Use slices for all fields per offset table
    let angular_velocity_y = f32::from_le_bytes(buf[48..52].try_into().unwrap());
    let _ = f32::from_le_bytes(buf[52..56].try_into().unwrap()); // unused
    let yaw = f32::from_le_bytes(buf[56..60].try_into().unwrap());
    let pitch = f32::from_le_bytes(buf[60..64].try_into().unwrap());
    let roll = f32::from_le_bytes(buf[64..68].try_into().unwrap());
    let normalized_suspension_travel_fl = f32::from_le_bytes(buf[68..72].try_into().unwrap());
    let normalized_suspension_travel_fr = f32::from_le_bytes(buf[72..76].try_into().unwrap());
    let normalized_suspension_travel_rl = f32::from_le_bytes(buf[76..80].try_into().unwrap());
    let normalized_suspension_travel_rr = f32::from_le_bytes(buf[80..84].try_into().unwrap());
    // skip to 164
    let tire_slip_angle_fl = f32::from_le_bytes(buf[164..168].try_into().unwrap());
    let tire_slip_angle_fr = f32::from_le_bytes(buf[168..172].try_into().unwrap());
    let tire_slip_angle_rl = f32::from_le_bytes(buf[172..176].try_into().unwrap());
    let tire_slip_angle_rr = f32::from_le_bytes(buf[176..180].try_into().unwrap());
    let suspension_travel_meters_fl = f32::from_le_bytes(buf[196..200].try_into().unwrap());
    let suspension_travel_meters_fr = f32::from_le_bytes(buf[200..204].try_into().unwrap());
    let suspension_travel_meters_rl = f32::from_le_bytes(buf[204..208].try_into().unwrap());
    let suspension_travel_meters_rr = f32::from_le_bytes(buf[208..212].try_into().unwrap());
    let car_ordinal = i32::from_le_bytes(buf[212..216].try_into().unwrap());
    let _ = i32::from_le_bytes(buf[216..220].try_into().unwrap()); // unused
    let drivetrain_type = i32::from_le_bytes(buf[224..228].try_into().unwrap());
    // skip
    let position_x = f32::from_le_bytes(buf[244..248].try_into().unwrap());
    let position_y = f32::from_le_bytes(buf[248..252].try_into().unwrap());
    let position_z = f32::from_le_bytes(buf[252..256].try_into().unwrap());
    let speed_ms = f32::from_le_bytes(buf[256..260].try_into().unwrap());
    let power_watts = f32::from_le_bytes(buf[260..264].try_into().unwrap());
    let _ = f32::from_le_bytes(buf[264..268].try_into().unwrap()); // unused
    let boost_psi = f32::from_le_bytes(buf[284..288].try_into().unwrap());
    let accel = buf[315];
    let brake = buf[316];
    let clutch = buf[317];
    let hand_brake = buf[318];
    let gear = buf[319];
    let steer = buf[320] as i8;

    Ok(FH6TelemetryData {
        is_race_on,
        timestamp_ms,
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
        car_ordinal,
        drivetrain_type,
        position_x,
        position_y,
        position_z,
        speed_ms,
        power_watts,
        boost_psi,
        accel,
        brake,
        clutch,
        hand_brake,
        gear,
        steer,
    })
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
        let buf = vec![0u8; 324];
        let data = decode(&buf).unwrap();
        assert_eq!(data.is_race_on, 0);
        assert_eq!(data.timestamp_ms, 0);
        assert_eq!(data.speed_ms, 0.0);
        assert_eq!(data.accel, 0);
        assert_eq!(data.steer, 0);
    }
}
