use fh6_core::telemetry::decode;
use std::fs;
use std::path::Path;

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

fn process_file(bin_path: &Path) -> Result<(), String> {
    let bytes = fs::read(bin_path).map_err(|e| format!("read error: {}", e))?;
    let data = decode(&bytes).map_err(|e| format!("decode error: {:?}", e))?;
    let json = telemetry_to_json(&data);
    let out_path = bin_path.with_extension("json");
    let pretty = serde_json::to_string_pretty(&json).map_err(|e| format!("json error: {}", e))?;
    if out_path.exists() {
        let expected = fs::read_to_string(&out_path).map_err(|e| format!("read expected: {}", e))?;
        if expected.trim() == pretty.trim() {
            println!("OK {}", bin_path.display());
        } else {
            println!("MISMATCH {}", bin_path.display());
        }
    } else {
        fs::write(&out_path, pretty).map_err(|e| format!("write error: {}", e))?;
        println!("WROTE {}", out_path.display());
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: parity-dump <path-or-dir>");
        std::process::exit(1);
    }
    let path = Path::new(&args[1]);
    if path.is_file() {
        if let Err(e) = process_file(path) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    } else if path.is_dir() {
        for entry in fs::read_dir(path).unwrap_or_else(|_| panic!("read dir")) {
            let entry = entry.unwrap();
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("bin") {
                if let Err(e) = process_file(&p) {
                    eprintln!("Error {}: {}", p.display(), e);
                }
            }
        }
    } else {
        eprintln!("Path not found");
        std::process::exit(1);
    }
}
