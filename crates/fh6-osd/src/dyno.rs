//! Dyno sweep generation and synthetic trail for OSD preview.

use fh6_core::telemetry::FH6TelemetryData;

/// Returns synthetic telemetry for the dyno sweep at phase `phase` seconds.
/// Phase is modulo 6.0, matching LiveDynoGenerator behaviour.
pub fn dyno_telemetry(phase: f32) -> FH6TelemetryData {
    const MAX_RPM: f32 = 9000.0;
    const IDLE_RPM: f32 = 850.0;

    let phase = phase % 6.0;
    let gear_idx = ((phase.floor() as i32) + 1).clamp(1, 6) as u8;
    let sub_phase = phase.fract();

    let rpm = IDLE_RPM + (MAX_RPM - IDLE_RPM) * (0.35 + 0.65 * sub_phase);
    let speed_mph = (gear_idx as f32) * 28.0 + sub_phase * 22.0;
    let speed_ms = speed_mph * 0.44704;
    let accel = if sub_phase > 0.95 { 0 } else { 255 };
    let brake = if sub_phase > 0.95 { 204 } else { 0 };
    let boost = if sub_phase > 0.3 { 16.5 * sub_phase } else { 2.0 };

    FH6TelemetryData {
        is_race_on: 0,
        engine_max_rpm: MAX_RPM,
        engine_idle_rpm: IDLE_RPM,
        current_engine_rpm: rpm,
        speed_ms,
        gear: gear_idx,
        accel,
        brake,
        boost_psi: boost,
        power_watts: 450_000.0 * (rpm / MAX_RPM),
        ..Default::default()
    }
}

/// Returns a synthetic trail with exactly 120 breadcrumb points.
/// The trail forms an S-curve racing line with z 0..≈180 m.
pub fn synthetic_trail() -> Vec<[f32; 3]> {
    let mut points = Vec::with_capacity(120);
    for i in 0..120 {
        let z = i as f32 * 1.5; // 0..178.5 m
        let x = (z * 0.03).sin() * 15.0;
        // y is constant 0 for ground plane
        points.push([x, 0.0, z]);
    }
    points
}
