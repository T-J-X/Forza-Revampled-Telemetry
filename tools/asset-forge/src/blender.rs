//! Runs a Blender recipe headless: the shared bpy prelude plus the recipe's body (both Rust
//! string constants) are written to a scratch `.py` outside git, parameters go in a JSON file,
//! and `blender.exe -b --factory-startup -P <script> -- <params.json>` runs it. The script must
//! print `FORGE-OK` as its last act.
//!
//! On the cloud lane the same script runs on the pod instead ([`crate::cloud::remote`]) and
//! its output folder is copied back, so the caller sees no difference.
use crate::error::{io_err, tool_err, Result};
use crate::lane::Lane;
use crate::paths::Paths;
use std::path::Path;
use std::process::Command;

/// Runs `body` with `params` on the run's GPU lane. Returns Blender's stdout for logging.
pub fn run(paths: &Paths, name: &str, body: &str, params: &serde_json::Value) -> Result<String> {
    let source = format!("{}\n{}\n", crate::recipes::PRELUDE, body);
    let stdout = match paths.lane {
        Lane::Local => run_local(paths, name, &source, params)?,
        Lane::Cloud => crate::cloud::remote::run_blender(paths, name, &source, params)?,
    };
    if let (Some(masters), Some(out)) = (&paths.masters, params.get("out").and_then(|o| o.as_str())) {
        let copied = crate::lane::sink_dir(masters, name, Path::new(out))?;
        println!(
            "  masters: {copied} file(s) from {name} copied to {}",
            masters.display()
        );
    }
    Ok(stdout)
}

/// Runs the script with this PC's Blender.
fn run_local(paths: &Paths, name: &str, source: &str, params: &serde_json::Value) -> Result<String> {
    if !paths.blender.is_file() {
        return Err(tool_err("blender", format!("not found at {}", paths.blender.display())));
    }
    let dir = paths.scratch.join("blender");
    std::fs::create_dir_all(&dir).map_err(|e| io_err(&dir, e))?;
    let script = dir.join(format!("{name}.py"));
    let args = dir.join(format!("{name}.json"));
    let log = dir.join(format!("{name}.log"));
    std::fs::write(&script, source).map_err(|e| io_err(&script, e))?;
    let text = serde_json::to_string_pretty(params).unwrap_or_default();
    std::fs::write(&args, text).map_err(|e| io_err(&args, e))?;
    let out = Command::new(&paths.blender)
        .arg("-b")
        .arg("--factory-startup")
        .arg("--python-exit-code")
        .arg("3")
        .arg("-P")
        .arg(&script)
        .arg("--")
        .arg(&args)
        .output()
        .map_err(|e| tool_err("blender", e.to_string()))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::write(&log, format!("{stdout}\n--- stderr ---\n{stderr}"));
    if !out.status.success() || !stdout.contains("FORGE-OK") {
        let tail: String = stdout
            .lines()
            .chain(stderr.lines())
            .filter(|l| !l.trim().is_empty())
            .rev()
            .take(25)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(tool_err(
            "blender",
            format!("recipe {name} failed ({}); log {}\n{tail}", out.status, log.display()),
        ));
    }
    Ok(stdout)
}

/// `blender --version` first line, for `doctor`.
pub fn version(blender: &Path) -> Result<String> {
    let out = Command::new(blender)
        .arg("-b")
        .arg("--factory-startup")
        .arg("--version")
        .output()
        .map_err(|e| tool_err("blender", e.to_string()))?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    text.lines()
        .find(|l| l.starts_with("Blender "))
        .map(str::to_owned)
        .ok_or_else(|| tool_err("blender", "no version line"))
}
