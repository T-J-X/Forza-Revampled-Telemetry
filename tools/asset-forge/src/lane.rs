//! GPU lanes. A lane is one GPU that runs one job at a time, with its own lock file next to the
//! tools (`SKILLS.md`, section 5). `local` is this PC's card; `cloud` is the rented pod driven by
//! [`crate::cloud`]. This module also finds the Google Drive masters folder and copies a job's
//! 4K outputs into it when a run asks for that with `--masters`.
use crate::error::{io_err, tool_err, ForgeError, Result};
use std::path::{Path, PathBuf};

/// The Google Drive folder that holds every dash's 4K masters.
pub const MASTERS_FOLDER: &str = "FH6 WheelDash v2 - 4K Masters";

/// ComfyUI outputs at least this long on one side count as masters (the 4x upscales).
pub const MASTER_MIN_SIDE: u32 = 2048;

/// Which GPU a run uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lane {
    /// The card in this PC: the ComfyUI portable install and `blender.exe`.
    #[default]
    Local,
    /// The rented pod, reached over SSH.
    Cloud,
}

impl Lane {
    /// Parses `local` or `cloud` (an empty string means local).
    pub fn parse(text: &str) -> Result<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "" | "local" => Ok(Self::Local),
            "cloud" => Ok(Self::Cloud),
            other => Err(ForgeError::Usage(format!(
                "the lane must be local or cloud, not {other}"
            ))),
        }
    }

    /// The lane's name, as typed after `--lane`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Cloud => "cloud",
        }
    }

    /// The lane's lock file name inside the tools folder.
    #[must_use]
    pub fn lock_name(self) -> &'static str {
        match self {
            Self::Local => "gpu.lock",
            Self::Cloud => "gpu-cloud.lock",
        }
    }

    /// The lane's lock file.
    #[must_use]
    pub fn lock_path(self, tools: &Path) -> PathBuf {
        tools.join(self.lock_name())
    }
}

/// A lane's lock file as the threads write it: `thread:`, `started:` and `job:` lines.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Lock {
    /// The thread that holds the lane.
    pub thread: String,
    /// When it took the lane (UTC, as written).
    pub started: String,
    /// What it is running.
    pub job: String,
}

impl Lock {
    /// Reads the three fields; unknown lines are ignored and missing ones stay empty.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut lock = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let value = value.trim().to_owned();
            match key.trim().to_ascii_lowercase().as_str() {
                "thread" => lock.thread = value,
                "started" => lock.started = value,
                "job" => lock.job = value,
                _ => {}
            }
        }
        lock
    }

    /// The lane's lock, or `None` when nobody holds it.
    #[must_use]
    pub fn read(tools: &Path, lane: Lane) -> Option<Self> {
        std::fs::read_to_string(lane.lock_path(tools))
            .ok()
            .map(|t| Self::parse(&t))
    }

    /// The lane's lock, or an error that says how to take it.
    pub fn require(tools: &Path, lane: Lane) -> Result<Self> {
        Self::read(tools, lane).ok_or_else(|| {
            tool_err(
                lane.name(),
                format!(
                    "the {} lane is not locked. Take {} first (SKILLS.md, section 5), and release it when the job ends",
                    lane.name(),
                    lane.lock_path(tools).display()
                ),
            )
        })
    }
}

/// True for a dash's `snake_name` or `shared`: lowercase letters, digits and underscores.
#[must_use]
pub fn valid_masters_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// Finds the masters folder: `FH6_MASTERS_DIR`, or `<drive>:\My Drive\<MASTERS_FOLDER>` on the
/// first drive letter that has it (the Google Drive desktop app picks the letter).
#[must_use]
pub fn masters_root() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("FH6_MASTERS_DIR").map(PathBuf::from) {
        return dir.is_dir().then_some(dir);
    }
    (b'D'..=b'Z')
        .map(|letter| PathBuf::from(format!("{}:\\My Drive\\{MASTERS_FOLDER}", letter as char)))
        .find(|dir| dir.is_dir())
}

/// The masters folder of one dash (`name` is its `snake_name`, or `shared`). The folder is
/// created inside the Drive folder; the Drive folder itself must already be mounted.
pub fn masters_dir(name: &str) -> Result<PathBuf> {
    if !valid_masters_name(name) {
        return Err(ForgeError::Usage(format!(
            "--masters takes a dash's snake_name or shared, not {name}"
        )));
    }
    let root = masters_root().ok_or_else(|| {
        tool_err(
            "masters",
            format!(
                "the Drive folder {MASTERS_FOLDER} is not mounted (set FH6_MASTERS_DIR, or start the Google Drive app)"
            ),
        )
    })?;
    let dir = root.join(name);
    std::fs::create_dir_all(&dir).map_err(|e| io_err(&dir, e))?;
    Ok(dir)
}

/// Keeps letters, digits, `.`, `-` and `_`; everything else becomes `_`. Used for folder and
/// file names that go to the pod or to the masters folder.
#[must_use]
pub fn safe_name(text: &str) -> String {
    let cleaned: String = text
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.trim_matches(|c| c == '.' || c == '_').is_empty() {
        "job".to_owned()
    } else {
        cleaned
    }
}

/// Width and height from a PNG's header, or `None` when the bytes are not a PNG.
#[must_use]
pub fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || !bytes.starts_with(SIGNATURE) || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let be = |at: usize| u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    Some((be(16), be(20)))
}

/// Copies every file in `out` (not its sub-folders) into `<masters>/<job>/`. Returns how many
/// files were copied.
pub fn sink_dir(masters: &Path, job: &str, out: &Path) -> Result<usize> {
    let dest = masters.join(safe_name(job));
    let entries = std::fs::read_dir(out).map_err(|e| io_err(out, e))?;
    let mut copied = 0;
    for entry in entries {
        let path = entry.map_err(|e| io_err(out, e))?.path();
        // The fingerprint stamps (`.fp`) are bookkeeping, not outputs.
        if !path.is_file() || path.file_name().is_some_and(|n| n.to_string_lossy().ends_with(".fp")) {
            continue;
        }
        let Some(name) = path.file_name() else { continue };
        std::fs::create_dir_all(&dest).map_err(|e| io_err(&dest, e))?;
        let to = dest.join(name);
        std::fs::copy(&path, &to).map_err(|e| io_err(&to, e))?;
        copied += 1;
    }
    Ok(copied)
}

/// Writes ComfyUI output images of master size into `<masters>/comfyui/`. `names` are the file
/// names ComfyUI gave them. Returns how many were written.
pub fn sink_images(masters: &Path, names: &[String], images: &[Vec<u8>]) -> Result<usize> {
    let dest = masters.join("comfyui");
    let mut written = 0;
    for (name, bytes) in names.iter().zip(images) {
        let big = png_size(bytes).is_some_and(|(w, h)| w.max(h) >= MASTER_MIN_SIDE);
        if !big {
            continue;
        }
        std::fs::create_dir_all(&dest).map_err(|e| io_err(&dest, e))?;
        let to = dest.join(safe_name(name));
        std::fs::write(&to, bytes).map_err(|e| io_err(&to, e))?;
        written += 1;
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_parses_both_names_and_rejects_others() {
        assert_eq!(Lane::parse("cloud").ok(), Some(Lane::Cloud));
        assert_eq!(Lane::parse(" Local ").ok(), Some(Lane::Local));
        assert_eq!(Lane::parse("").ok(), Some(Lane::Local));
        assert!(Lane::parse("moon").is_err());
    }

    #[test]
    fn each_lane_has_its_own_lock_file() {
        assert_eq!(Lane::Local.lock_name(), "gpu.lock");
        assert_eq!(Lane::Cloud.lock_name(), "gpu-cloud.lock");
        assert_ne!(
            Lane::Local.lock_path(Path::new("t")),
            Lane::Cloud.lock_path(Path::new("t"))
        );
    }

    #[test]
    fn lock_reads_the_three_lines_the_threads_write() {
        let lock = Lock::parse("thread: Set up cloud GPU lane\nstarted: 2026-11-27T14:00:00Z\njob: cloud selftest\n");
        assert_eq!(lock.thread, "Set up cloud GPU lane");
        assert_eq!(lock.started, "2026-11-27T14:00:00Z");
        assert_eq!(lock.job, "cloud selftest");
        assert_eq!(Lock::parse("nonsense"), Lock::default());
    }

    #[test]
    fn a_missing_lock_is_an_error_that_names_the_file() {
        let dir = std::env::temp_dir().join("forge_lane_no_lock");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::remove_file(Lane::Cloud.lock_path(&dir));
        let err = Lock::require(&dir, Lane::Cloud).map(|_| ()).unwrap_err().to_string();
        assert!(err.contains("gpu-cloud.lock"), "{err}");
    }

    #[test]
    fn masters_names_are_snake_case_only() {
        assert!(valid_masters_name("liquid_glass"));
        assert!(valid_masters_name("shared"));
        assert!(!valid_masters_name("Liquid Glass"));
        assert!(!valid_masters_name("..\\elsewhere"));
        assert!(!valid_masters_name(""));
    }

    #[test]
    fn safe_name_keeps_plain_names_and_replaces_the_rest() {
        assert_eq!(safe_name("hero_bezel-01.png"), "hero_bezel-01.png");
        assert_eq!(safe_name("a b/c:d"), "a_b_c_d");
        assert_eq!(safe_name(".."), "job");
    }

    #[test]
    fn png_size_reads_the_header() {
        let mut png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR".to_vec();
        png.extend_from_slice(&4096u32.to_be_bytes());
        png.extend_from_slice(&2160u32.to_be_bytes());
        assert_eq!(png_size(&png), Some((4096, 2160)));
        assert_eq!(png_size(b"not a png"), None);
    }

    #[test]
    fn only_master_sized_images_reach_the_masters_folder() {
        let dir = std::env::temp_dir().join("forge_lane_sink_images");
        let _ = std::fs::remove_dir_all(&dir);
        let header = |w: u32, h: u32| {
            let mut png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR".to_vec();
            png.extend_from_slice(&w.to_be_bytes());
            png.extend_from_slice(&h.to_be_bytes());
            png
        };
        let names = vec!["base_00001_.png".to_owned(), "upscaled_00001_.png".to_owned()];
        let images = vec![header(1024, 1024), header(4096, 4096)];
        assert_eq!(sink_images(&dir, &names, &images).ok(), Some(1));
        assert!(dir.join("comfyui").join("upscaled_00001_.png").is_file());
        assert!(!dir.join("comfyui").join("base_00001_.png").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_job_folder_is_copied_without_its_stamps() {
        let root = std::env::temp_dir().join("forge_lane_sink_dir");
        let _ = std::fs::remove_dir_all(&root);
        let out = root.join("out");
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(out.join("beauty_0.png"), b"x").unwrap();
        std::fs::write(out.join("data.exr"), b"y").unwrap();
        std::fs::write(out.join(".fp"), b"stamp").unwrap();
        let masters = root.join("masters");
        assert_eq!(sink_dir(&masters, "hero bezel", &out).ok(), Some(2));
        assert!(masters.join("hero_bezel").join("beauty_0.png").is_file());
        assert!(!masters.join("hero_bezel").join(".fp").exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
