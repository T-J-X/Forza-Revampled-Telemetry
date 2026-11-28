//! Telemetry data structures and the 324-byte little-endian Forza UDP packet decoder.
//! Field offsets are fixed by the game's packet format; check any change against
//! `tests/golden/packets` (97% rule).
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
    })
}

/// Encodes `t` as the game would send it: the inverse of [`decode`] for every packet whose
/// floats are within `±`[`MAX_ABS`] and whose rpm is not negative. Bytes `decode` does not read
/// are zero. For tools and tests (`mock-udp`-style packets, the robustness gate's byte sweeps).
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
    ] {
        put(off, v.to_le_bytes());
    }
    put(212, t.car_ordinal.to_le_bytes());
    put(224, t.drivetrain_type.to_le_bytes());
    b[315] = t.accel;
    b[316] = t.brake;
    b[317] = t.clutch;
    b[318] = t.hand_brake;
    b[319] = t.gear;
    b[320] = t.steer.to_le_bytes()[0];
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

    /// Byte offsets of the 31 floats `decode` reads.
    const FLOAT_OFFSETS: [usize; 31] = [
        8, 12, 16, 20, 24, 28, 32, 36, 40, 48, 56, 60, 64, 68, 72, 76, 80, 164, 168, 172, 176, 196, 200, 204, 208, 244,
        248, 252, 256, 260, 284,
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
        b[315..321].copy_from_slice(&[201, 202, 203, 204, 7, (-77i8).to_le_bytes()[0]]);
        b
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
