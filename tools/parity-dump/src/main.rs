//! Decodes `tests/golden/packets/*.bin` and compares each with its `.json`. A missing `.json`
//! fails, as does a `.json` with no `.bin`, so a deleted golden cannot certify itself.
//! `--write-missing` writes the missing goldens for a new packet; never rewrite an existing one
//! to make it pass.
//!
//! Usage: `parity-dump [--write-missing] <packet.bin | directory>`
use fh6_core::telemetry::decode;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The golden's fields, keyed in sorted order. A `BTreeMap` keeps that order whatever
/// `serde_json` features the build unifies (`fh6-win` turns on `preserve_order`, which would
/// otherwise write the keys in insertion order and mismatch every golden in a workspace build).
fn telemetry_to_json(data: &fh6_core::telemetry::FH6TelemetryData) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("IsRaceOn", json!(data.is_race_on)),
        ("TimestampMS", json!(data.timestamp_ms)),
        ("EngineMaxRpm", json!(data.engine_max_rpm)),
        ("EngineIdleRpm", json!(data.engine_idle_rpm)),
        ("CurrentEngineRpm", json!(data.current_engine_rpm)),
        (
            "Acceleration",
            json!([data.acceleration_x, data.acceleration_y, data.acceleration_z]),
        ),
        ("Velocity", json!([data.velocity_x, data.velocity_y, data.velocity_z])),
        ("CarOrdinal", json!(data.car_ordinal)),
        ("DrivetrainType", json!(data.drivetrain_type)),
        ("SpeedMS", json!(data.speed_ms)),
        ("SpeedMPH", json!(data.speed_mph())),
        ("SpeedKMH", json!(data.speed_kmh())),
        ("PowerWatts", json!(data.power_watts)),
        ("PowerHP", json!(data.power_watts / 745.7)),
        ("BoostPSI", json!(data.boost_psi)),
        ("Accel", json!(data.accel)),
        ("Brake", json!(data.brake)),
        ("Clutch", json!(data.clutch)),
        ("Handbrake", json!(data.hand_brake)),
        ("Gear", json!(data.gear)),
        ("Steer", json!(data.steer)),
    ])
}

/// Git may check the goldens out with CRLF on Windows; compare line content only.
fn normalise(text: &str) -> String {
    text.replace("\r\n", "\n").trim().to_string()
}

/// What checking one packet found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    /// The decoded packet matches its golden.
    Match,
    /// The decoded packet differs from its golden.
    Mismatch,
    /// The packet has no golden.
    Missing,
    /// The packet had no golden and `--write-missing` wrote one.
    Wrote,
}

impl Outcome {
    fn passed(self) -> bool {
        matches!(self, Self::Match | Self::Wrote)
    }

    fn label(self) -> &'static str {
        match self {
            Self::Match => "OK",
            Self::Mismatch => "MISMATCH",
            Self::Missing => "MISSING",
            Self::Wrote => "WROTE",
        }
    }
}

/// Checks `bin_path` against the `.json` beside it, writing the `.json` only when it is missing
/// and `write_missing` is set. The write uses `create_new`, so a golden that appears meanwhile
/// is compared, never overwritten.
fn process_file(bin_path: &Path, write_missing: bool) -> Result<Outcome, String> {
    let bytes = fs::read(bin_path).map_err(|e| format!("read error: {e}"))?;
    let data = decode(&bytes).map_err(|e| format!("decode error: {e}"))?;
    let pretty = serde_json::to_string_pretty(&telemetry_to_json(&data)).map_err(|e| format!("json error: {e}"))?;
    let out_path = bin_path.with_extension("json");
    if write_missing {
        match fs::OpenOptions::new().write(true).create_new(true).open(&out_path) {
            Ok(mut file) => {
                file.write_all(pretty.as_bytes())
                    .map_err(|e| format!("write error: {e}"))?;
                return Ok(Outcome::Wrote);
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
            Err(e) => return Err(format!("write error: {e}")),
        }
    }
    match fs::read_to_string(&out_path) {
        Ok(expected) if normalise(&expected) == normalise(&pretty) => Ok(Outcome::Match),
        Ok(_) => Ok(Outcome::Mismatch),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Outcome::Missing),
        Err(e) => Err(format!("read expected: {e}")),
    }
}

/// Files in `dir` with extension `ext`, sorted.
fn files_with(dir: &Path, ext: &str) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("reading {}: {e}", dir.display()))?;
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some(ext))
        .collect();
    files.sort();
    Ok(files)
}

/// Goldens in `dir` whose packet is gone.
fn orphans(dir: &Path) -> Result<Vec<PathBuf>, String> {
    Ok(files_with(dir, "json")?
        .into_iter()
        .filter(|j| !j.with_extension("bin").exists())
        .collect())
}

/// Checks `path` (one packet or a directory of them); true when everything passed.
fn run(path: &Path, write_missing: bool) -> Result<bool, String> {
    let (bins, orphaned) = if path.is_file() {
        (vec![path.to_path_buf()], Vec::new())
    } else if path.is_dir() {
        (files_with(path, "bin")?, orphans(path)?)
    } else {
        return Err(format!("path not found: {}", path.display()));
    };
    if bins.is_empty() {
        return Err(format!("no .bin packets in {}", path.display()));
    }
    let mut passed = 0usize;
    for file in &bins {
        match process_file(file, write_missing) {
            Ok(outcome) => {
                println!("{} {}", outcome.label(), file.display());
                passed += usize::from(outcome.passed());
            }
            Err(e) => println!("ERROR {}: {e}", file.display()),
        }
    }
    for json in &orphaned {
        println!("ORPHAN {} (no .bin beside it)", json.display());
    }
    println!("{passed}/{} OK, {} orphaned goldens", bins.len(), orphaned.len());
    Ok(passed == bins.len() && orphaned.is_empty())
}

fn main() -> ExitCode {
    let mut write_missing = false;
    let mut target = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--write-missing" => write_missing = true,
            _ if target.is_none() && !arg.starts_with("--") => target = Some(arg),
            _ => {
                eprintln!("unexpected argument {arg}");
                target = None;
                break;
            }
        }
    }
    let Some(target) = target else {
        eprintln!("Usage: parity-dump [--write-missing] <packet.bin | directory>");
        return ExitCode::FAILURE;
    };
    match run(Path::new(&target), write_missing) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_core::telemetry::PACKET_LEN;

    /// A fresh, empty directory under the system temp folder.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("parity-dump-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn packet(dir: &Path, stem: &str, rpm: f32) -> PathBuf {
        let mut bytes = vec![0u8; PACKET_LEN];
        bytes[16..20].copy_from_slice(&rpm.to_le_bytes());
        let path = dir.join(format!("{stem}.bin"));
        fs::write(&path, bytes).unwrap();
        path
    }

    /// The output's keys are sorted, as in the goldens, with or without `preserve_order`.
    #[test]
    fn keys_are_written_sorted() {
        let data = decode(&[0u8; PACKET_LEN]).unwrap();
        let pretty = serde_json::to_string_pretty(&telemetry_to_json(&data)).unwrap();
        let keys: Vec<&str> = pretty
            .lines()
            .filter_map(|l| l.strip_prefix("  \"").and_then(|l| l.split('"').next()))
            .collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        assert_eq!(keys.len(), 21);
        assert_eq!(keys, sorted);
    }

    #[test]
    fn line_endings_do_not_matter() {
        assert_eq!(normalise("{\r\n  \"a\": 1\r\n}\r\n"), normalise("{\n  \"a\": 1\n}"));
        assert_ne!(normalise("{\"a\": 1}"), normalise("{\"a\": 2}"));
    }

    #[test]
    fn a_missing_golden_fails_unless_asked_to_write_it() {
        let dir = scratch("missing");
        let bin = packet(&dir, "01", 900.0);
        assert_eq!(process_file(&bin, false).unwrap(), Outcome::Missing);
        assert!(!bin.with_extension("json").exists(), "a plain run writes nothing");
        assert!(!run(&dir, false).unwrap());
        assert_eq!(process_file(&bin, true).unwrap(), Outcome::Wrote);
        assert_eq!(process_file(&bin, false).unwrap(), Outcome::Match);
        assert!(run(&dir, false).unwrap());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_changed_packet_mismatches_and_is_never_rewritten() {
        let dir = scratch("mismatch");
        let bin = packet(&dir, "01", 900.0);
        process_file(&bin, true).unwrap();
        let golden = fs::read_to_string(bin.with_extension("json")).unwrap();
        packet(&dir, "01", 5_200.0);
        assert_eq!(process_file(&bin, true).unwrap(), Outcome::Mismatch);
        assert_eq!(fs::read_to_string(bin.with_extension("json")).unwrap(), golden);
        assert!(!run(&dir, true).unwrap());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_golden_without_its_packet_fails() {
        let dir = scratch("orphan");
        let bin = packet(&dir, "01", 900.0);
        process_file(&bin, true).unwrap();
        fs::copy(bin.with_extension("json"), dir.join("02.json")).unwrap();
        assert_eq!(orphans(&dir).unwrap(), vec![dir.join("02.json")]);
        assert!(!run(&dir, false).unwrap());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_empty_directory_is_an_error() {
        let dir = scratch("empty");
        assert!(run(&dir, false).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    /// The checked-in goldens all pass, with nothing written.
    #[test]
    fn the_repo_goldens_pass() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/packets");
        assert!(run(&dir, false).unwrap());
    }
}
