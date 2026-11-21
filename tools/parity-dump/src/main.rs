//! Decodes `tests/golden/packets/*.bin` and compares each with its `.json` (or writes the
//! `.json` when it does not exist yet). Never rewrite an existing golden to make it pass.
use fh6_core::telemetry::decode;
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

/// Git may check the goldens out with CRLF on Windows; compare line content only.
fn normalise(text: &str) -> String {
    text.replace("\r\n", "\n").trim().to_string()
}

/// Returns `Ok(true)` when the packet matches (or a new golden was written).
fn process_file(bin_path: &Path) -> Result<bool, String> {
    let bytes = fs::read(bin_path).map_err(|e| format!("read error: {e}"))?;
    let data = decode(&bytes).map_err(|e| format!("decode error: {e}"))?;
    let json = telemetry_to_json(&data);
    let out_path = bin_path.with_extension("json");
    let pretty = serde_json::to_string_pretty(&json).map_err(|e| format!("json error: {e}"))?;
    if out_path.exists() {
        let expected = fs::read_to_string(&out_path).map_err(|e| format!("read expected: {e}"))?;
        if normalise(&expected) == normalise(&pretty) {
            println!("OK {}", bin_path.display());
            Ok(true)
        } else {
            println!("MISMATCH {}", bin_path.display());
            Ok(false)
        }
    } else {
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
    fn line_endings_do_not_matter() {
        assert_eq!(normalise("{\r\n  \"a\": 1\r\n}\r\n"), normalise("{\n  \"a\": 1\n}"));
        assert_ne!(normalise("{\"a\": 1}"), normalise("{\"a\": 2}"));
    }
}
