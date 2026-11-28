//! The golden packets: every `tests/golden/packets/*.bin` decodes to exactly the values in the
//! `.json` beside it, floats included, and every key of every `.json` is checked. The packets are
//! synthetic (`mock-udp`), so they prove the decoder against the fields `mock-udp` writes, not
//! against the game (AGENTS.md directive 1).
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test: a missing or malformed golden should stop the test with its message"
)]

use fh6_core::telemetry::{decode, FH6TelemetryData, PACKET_LEN};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// At least this many packets are checked in (the 20 synthetic goldens).
const MIN_PACKETS: usize = 20;

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/packets")
}

/// A decoded value as the golden `.json` stores it.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Field {
    Int(i64),
    Float(f32),
    Vec3([f32; 3]),
}

/// Every key `parity-dump` writes, with the decoded value it must hold.
fn expected_fields(d: &FH6TelemetryData) -> Vec<(&'static str, Field)> {
    use Field::{Float, Int, Vec3};
    vec![
        ("IsRaceOn", Int(d.is_race_on.into())),
        ("TimestampMS", Int(d.timestamp_ms.into())),
        ("EngineMaxRpm", Float(d.engine_max_rpm)),
        ("EngineIdleRpm", Float(d.engine_idle_rpm)),
        ("CurrentEngineRpm", Float(d.current_engine_rpm)),
        (
            "Acceleration",
            Vec3([d.acceleration_x, d.acceleration_y, d.acceleration_z]),
        ),
        ("Velocity", Vec3([d.velocity_x, d.velocity_y, d.velocity_z])),
        ("CarOrdinal", Int(d.car_ordinal.into())),
        ("DrivetrainType", Int(d.drivetrain_type.into())),
        ("SpeedMS", Float(d.speed_ms)),
        ("SpeedMPH", Float(d.speed_mph())),
        ("SpeedKMH", Float(d.speed_kmh())),
        ("PowerWatts", Float(d.power_watts)),
        ("PowerHP", Float(d.power_watts / 745.7)),
        ("BoostPSI", Float(d.boost_psi)),
        ("Accel", Int(d.accel.into())),
        ("Brake", Int(d.brake.into())),
        ("Clutch", Int(d.clutch.into())),
        ("Handbrake", Int(d.hand_brake.into())),
        ("Gear", Int(d.gear.into())),
        ("Steer", Int(d.steer.into())),
    ]
}

/// A JSON number as the `f32` it was written from (`serde_json` writes an `f32` with the
/// shortest digits that read back as that `f32`).
fn json_f32(v: &Value) -> Option<f32> {
    v.as_f64().map(|x| x as f32)
}

/// The mismatch between `want` and the golden's `got`, if any. Floats compare exactly.
fn mismatch(want: Field, got: &Value) -> Option<String> {
    let ok = match want {
        Field::Int(w) => got.as_i64() == Some(w),
        Field::Float(w) => json_f32(got).is_some_and(|g| g.to_bits() == w.to_bits()),
        Field::Vec3(w) => got.as_array().is_some_and(|a| {
            a.len() == 3
                && a.iter()
                    .zip(w)
                    .all(|(g, w)| json_f32(g).is_some_and(|g| g.to_bits() == w.to_bits()))
        }),
    };
    (!ok).then(|| format!("decoded {want:?}, golden {got}"))
}

fn files_with(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .expect("tests/golden/packets")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some(ext))
        .collect();
    files.sort();
    files
}

#[test]
fn every_golden_packet_matches_its_json_in_every_field() {
    let dir = golden_dir();
    let bins = files_with(&dir, "bin");
    assert!(
        bins.len() >= MIN_PACKETS,
        "expected at least {MIN_PACKETS} golden packets in {}, found {}",
        dir.display(),
        bins.len()
    );
    let mut failures = Vec::new();
    for bin in &bins {
        let name = bin
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let bytes = fs::read(bin).expect("read packet");
        assert_eq!(bytes.len(), PACKET_LEN, "{name} is not a {PACKET_LEN}-byte packet");
        let data = decode(&bytes).expect("decode packet");
        let json_path = bin.with_extension("json");
        let Ok(text) = fs::read_to_string(&json_path) else {
            failures.push(format!("{name}: no golden .json"));
            continue;
        };
        let golden: Value = serde_json::from_str(&text).expect("golden .json parses");
        let Some(object) = golden.as_object() else {
            failures.push(format!("{name}: the golden is not a JSON object"));
            continue;
        };
        let fields = expected_fields(&data);
        let want: BTreeSet<&str> = fields.iter().map(|(k, _)| *k).collect();
        let have: BTreeSet<&str> = object.keys().map(String::as_str).collect();
        for key in have.difference(&want) {
            failures.push(format!("{name}: golden key {key} is not checked"));
        }
        for (key, value) in fields {
            match object.get(key) {
                None => failures.push(format!("{name}: golden has no {key}")),
                Some(got) => {
                    if let Some(m) = mismatch(value, got) {
                        failures.push(format!("{name}: {key}: {m}"));
                    }
                }
            }
        }
    }
    for json in files_with(&dir, "json") {
        if !json.with_extension("bin").exists() {
            failures.push(format!("{}: golden with no packet", json.display()));
        }
    }
    assert!(failures.is_empty(), "golden telemetry:\n{}", failures.join("\n"));
}

/// The comparison itself: a float one bit off, a wrong int and a short vector all fail.
#[test]
fn the_comparison_is_exact() {
    let x = 8_000.0f32;
    assert!(mismatch(Field::Float(x), &serde_json::json!(x)).is_none());
    assert!(mismatch(Field::Float(x), &serde_json::json!(f32::from_bits(x.to_bits() + 1))).is_some());
    assert!(mismatch(Field::Float(0.1), &serde_json::json!(0.1f32)).is_none());
    assert!(mismatch(Field::Int(11), &serde_json::json!(12)).is_some());
    assert!(mismatch(Field::Vec3([1.0, 2.0, 3.0]), &serde_json::json!([1.0, 2.0])).is_some());
    assert!(mismatch(Field::Vec3([1.0, 2.0, 3.0]), &serde_json::json!([1.0, 2.0, 3.0])).is_none());
    assert!(mismatch(Field::Float(1.0), &serde_json::json!("1.0")).is_some());
}
