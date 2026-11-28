//! Decodes `tests/golden/packets/*.bin` and compares each with its `.json` (or writes the
//! `.json` when it does not exist yet). Never rewrite an existing golden to make it pass.
use fh6_core::telemetry::decode;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn telemetry_to_json(data: &fh6_core::telemetry::FH6TelemetryData) -> serde_json::Value {
    serde_json::json!({
        "IsRaceOn": data.is_race_on,
        "TimestampMS": data.timestamp_ms,
        "EngineMaxRpm": data.engine_max_rpm,
        "EngineIdleRpm": data.engine_idle_rpm,
        "CurrentEngineRpm": data.current_engine_rpm,
        "Acceleration": [data.acceleration_x, data.acceleration_y, data.acceleration_z],
        "Velocity": [data.velocity_x, data.velocity_y, data.velocity_z],
        "CarOrdinal": data.car_ordinal,
        "DrivetrainType": data.drivetrain_type,
        "SpeedMS": data.speed_ms,
        "SpeedMPH": data.speed_mph(),
        "SpeedKMH": data.speed_kmh(),
        "PowerWatts": data.power_watts,
        "PowerHP": data.power_watts / 745.7,
        "BoostPSI": data.boost_psi,
        "Accel": data.accel,
        "Brake": data.brake,
        "Clutch": data.clutch,
        "Handbrake": data.hand_brake,
        "Gear": data.gear,
        "Steer": data.steer,
    })
}

/// Whether a golden's text holds the same values as `decoded`. The values are compared, not the
/// text, so line endings (git may check the goldens out with CRLF on Windows) and key order (it
/// depends on whether `serde_json`'s `preserve_order` is on in the build) do not matter.
///
/// `decoded` goes through its own text first: an f32 is written in its shortest form, which reads
/// back as a different f64 than the f32's own value.
fn matches(expected: &str, decoded: &Value) -> bool {
    let as_written = serde_json::to_string(decoded).and_then(|text| serde_json::from_str::<Value>(&text));
    match (serde_json::from_str::<Value>(expected), as_written) {
        (Ok(expected), Ok(decoded)) => expected == decoded,
        _ => false,
    }
}

/// `json` as a new golden is written: pretty, with its keys in alphabetical order like the
/// existing goldens.
fn golden_text(json: &Value) -> serde_json::Result<String> {
    match json.as_object() {
        Some(map) => serde_json::to_string_pretty(&map.iter().collect::<BTreeMap<_, _>>()),
        None => serde_json::to_string_pretty(json),
    }
}

/// Returns `Ok(true)` when the packet matches (or a new golden was written).
fn process_file(bin_path: &Path) -> Result<bool, String> {
    let bytes = fs::read(bin_path).map_err(|e| format!("read error: {e}"))?;
    let data = decode(&bytes).map_err(|e| format!("decode error: {e}"))?;
    let json = telemetry_to_json(&data);
    let out_path = bin_path.with_extension("json");
    if out_path.exists() {
        let expected = fs::read_to_string(&out_path).map_err(|e| format!("read expected: {e}"))?;
        if matches(&expected, &json) {
            println!("OK {}", bin_path.display());
            Ok(true)
        } else {
            println!("MISMATCH {}", bin_path.display());
            Ok(false)
        }
    } else {
        let pretty = golden_text(&json).map_err(|e| format!("json error: {e}"))?;
        fs::write(&out_path, pretty).map_err(|e| format!("write error: {e}"))?;
        println!("WROTE {}", out_path.display());
        Ok(true)
    }
}

fn main() -> ExitCode {
    let Some(arg) = std::env::args().nth(1) else {
        eprintln!("Usage: parity-dump <path-or-dir>");
        return ExitCode::FAILURE;
    };
    let path = Path::new(&arg);
    let files: Vec<_> = if path.is_file() {
        vec![path.to_path_buf()]
    } else if path.is_dir() {
        match fs::read_dir(path) {
            Ok(entries) => {
                let mut bins: Vec<_> = entries
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("bin"))
                    .collect();
                bins.sort();
                bins
            }
            Err(e) => {
                eprintln!("Error reading {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
        }
    } else {
        eprintln!("Path not found: {}", path.display());
        return ExitCode::FAILURE;
    };
    let mut ok = 0usize;
    for file in &files {
        match process_file(file) {
            Ok(true) => ok += 1,
            Ok(false) => {}
            Err(e) => eprintln!("Error {}: {e}", file.display()),
        }
    }
    println!("{ok}/{} OK", files.len());
    if ok == files.len() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_endings_and_key_order_do_not_matter_but_values_do() {
        let decoded = serde_json::json!({"b": 0.5, "a": [1, 2]});
        assert!(matches("{\r\n  \"a\": [1, 2],\r\n  \"b\": 0.5\r\n}\r\n", &decoded));
        assert!(matches("{\"b\": 0.5, \"a\": [1, 2]}", &decoded));
        assert!(!matches("{\"a\": [1, 2], \"b\": 0.25}", &decoded));
        assert!(!matches("{\"a\": [1, 2]}", &decoded));
        assert!(!matches("not json", &decoded));
    }

    #[test]
    fn a_new_golden_lists_its_keys_in_alphabetical_order() {
        let text = golden_text(&serde_json::json!({"IsRaceOn": 1, "Accel": 0})).expect("serialises");
        assert!(text.find("Accel") < text.find("IsRaceOn"), "{text}");
    }

    #[test]
    fn every_shipped_golden_matches_its_packet() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/packets");
        let mut checked = 0;
        for entry in fs::read_dir(&dir).expect("the golden packets folder") {
            let path = entry.expect("an entry").path();
            if path.extension().and_then(|e| e.to_str()) == Some("bin") {
                // Never let the test write a golden: each packet must already have one.
                assert!(
                    path.with_extension("json").is_file(),
                    "{} has no golden",
                    path.display()
                );
                assert_eq!(process_file(&path), Ok(true), "{}", path.display());
                checked += 1;
            }
        }
        assert_eq!(checked, 20);
    }
}
