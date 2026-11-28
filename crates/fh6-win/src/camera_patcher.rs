//! The game's camera files: `mediapc\Camera\CameraSettings.ini` (per-view FOVs, the wheel
//! offset, hood offsets) and `CameraPhysics.xml` (the DriverCam fixed offset, i.e. the seat).
//!
//! Safety rules, in order:
//! 1. Nothing is written while the game runs: [`apply`] returns [`ApplyOutcome::Queued`] and the
//!    caller retries when the game has exited (the restart-needed status).
//! 2. Before the first change the originals are copied to
//!    `%LOCALAPPDATA%\FH6WheelDash\backups\<YYYYMMDD-HHMMSS>\` and verified; later changes keep
//!    that first backup. [`restore`] puts it back in one call.
//! 3. Edits are surgical: only the values of known keys/attributes change. Line endings (CRLF),
//!    the trailing newline, key spelling and every other byte stay as they were. Files are
//!    written UTF-8 without BOM, atomically (temp file + rename).
//! 4. A file that lacks a required key or the DriverCam offset layer is not touched at all.
#![forbid(unsafe_code)]
use crate::car_brands::{BindCarBrandError, BindOutcome, CarBinding};
use crate::steam::camera_dir;
use fh6_core::atomic_file::write_atomically;
use fh6_core::config::{AppConfig, FOV_RANGE};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The settings file name.
pub const SETTINGS_FILE: &str = "CameraSettings.ini";
/// The physics file name.
pub const PHYSICS_FILE: &str = "CameraPhysics.xml";

/// Why a patch, backup or restore did not happen. Files are never left half-written.
#[derive(Debug, thiserror::Error)]
pub enum CameraPatchError {
    /// Reading or writing a file failed.
    #[error("{path}: {source}")]
    Io {
        /// The file.
        path: PathBuf,
        /// The error.
        source: io::Error,
    },
    /// A key the patch must write is missing from `CameraSettings.ini`.
    #[error("CameraSettings.ini has no `{0}` key")]
    MissingKey(&'static str),
    /// `CameraPhysics.xml` has no DriverCam block with a fixed CarSpaceXYZOffset layer.
    #[error("CameraPhysics.xml has no DriverCam ConstantOne/None/CarSpaceXYZOffset layer")]
    MissingSeatLayer,
    /// The seat layer lacks a `Param.x`, `Param.y` or `Param.z` attribute.
    #[error("the DriverCam offset layer has no `{0}` attribute")]
    MissingAttribute(&'static str),
    /// There is no backup to restore.
    #[error("no backup of the original camera files")]
    NoBackup,
    /// The backup copy did not match the original byte for byte.
    #[error("backup verification failed for {0}")]
    BackupMismatch(PathBuf),
    /// The game is running; nothing was written.
    #[error("the game is running; close it first")]
    GameRunning,
    /// `%LOCALAPPDATA%` is not set, so there is nowhere to keep backups.
    #[error("LOCALAPPDATA is not set; no backup folder")]
    NoBackupRoot,
}

fn io_at(path: &Path) -> impl FnOnce(io::Error) -> CameraPatchError + '_ {
    move |source| CameraPatchError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// The values written to the game files, already clamped.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraPatch {
    /// `CockpitCam\FOV`: the wheel-less cockpit view `[3]` (`WheelLessCockpitFOV`).
    pub cockpit_fov: f32,
    /// `Driver\FOV`: the standard driver view (`StandardCockpitFOV`).
    pub driver_fov: f32,
    /// `HoodCam\FOV` (`BonnetFOV`).
    pub hood_fov: f32,
    /// `FollowLowCam\FOV` (`ChaseNearFOV`).
    pub chase_near_fov: f32,
    /// `FollowHighCam\FOV` (`ChaseFarFOV`).
    pub chase_far_fov: f32,
    /// `BumperHighCam\FOV` (`BumperFOV`).
    pub bumper_fov: f32,
    /// `Driver\DefaultOffsetFromSteeringWheelY` in m (`SteeringWheelOffsetYM`).
    pub wheel_offset_y: f32,
    /// DriverCam fixed `CarSpaceXYZOffset` `Param.x/y/z` in m: x and z from `CockpitOffsetX/Z`,
    /// y (the seat height) from `DriverSeatHeightM`.
    pub seat: [f32; 3],
    /// `HoodCam\CamYOffset`, `CamZOffset`, `CamPitchOffset`.
    pub hood: [f32; 3],
}

fn clampf(v: f32, fallback: f32, lo: f32, hi: f32) -> f32 {
    if v.is_finite() {
        v.clamp(lo, hi)
    } else {
        fallback
    }
}

impl CameraPatch {
    /// The patch for `cfg`.
    #[must_use]
    pub fn from_config(cfg: &AppConfig) -> Self {
        let fov = |v: f32, d: f32| clampf(v, d, FOV_RANGE.0, FOV_RANGE.1);
        Self {
            cockpit_fov: fov(cfg.wheel_less_cockpit_fov, 80.0),
            driver_fov: fov(cfg.standard_cockpit_fov, 80.0),
            hood_fov: fov(cfg.bonnet_fov, 78.0),
            chase_near_fov: fov(cfg.chase_near_fov, 62.0),
            chase_far_fov: fov(cfg.chase_far_fov, 68.0),
            bumper_fov: fov(cfg.bumper_fov, 78.0),
            wheel_offset_y: clampf(cfg.steering_wheel_offset_y_m, -0.035, -0.3, 0.3),
            seat: [
                clampf(cfg.cockpit_offset_x, 0.0, -0.5, 0.5),
                clampf(cfg.driver_seat_height_m, -0.045, -0.3, 0.3),
                clampf(cfg.cockpit_offset_z, -0.5, -1.5, 1.0),
            ],
            hood: [
                clampf(cfg.hood_offset_y, 0.0, -1.0, 1.0),
                clampf(cfg.hood_offset_z, 0.0, -1.0, 1.0),
                clampf(cfg.hood_pitch_offset, 0.04, -0.5, 0.5),
            ],
        }
    }

    /// The `CameraSettings.ini` keys and values, with whether the key must exist.
    fn settings(&self) -> [(&'static str, f32, bool); 10] {
        [
            (r"CockpitCam\FOV", self.cockpit_fov, true),
            (r"Driver\FOV", self.driver_fov, true),
            (r"Driver\DefaultOffsetFromSteeringWheelY", self.wheel_offset_y, true),
            (r"HoodCam\FOV", self.hood_fov, false),
            (r"FollowLowCam\FOV", self.chase_near_fov, false),
            (r"FollowHighCam\FOV", self.chase_far_fov, false),
            (r"BumperHighCam\FOV", self.bumper_fov, false),
            (r"HoodCam\CamYOffset", self.hood[0], false),
            (r"HoodCam\CamZOffset", self.hood[1], false),
            (r"HoodCam\CamPitchOffset", self.hood[2], false),
        ]
    }
}

/// Formats `v` with the old value's decimals, or more when `v` needs them (up to 6).
fn fmt_like(old: &str, v: f32) -> String {
    let old_dec = old.split_once('.').map_or(0, |(_, f)| f.len()).min(6);
    let mut need = 0;
    while need < 6 {
        let k = 10f64.powi(need);
        if ((f64::from(v) * k).round() / k - f64::from(v)).abs() < 1e-6 {
            break;
        }
        need += 1;
    }
    let s = format!("{:.*}", old_dec.max(need as usize), v);
    match s.strip_prefix('-') {
        Some(rest) if rest.chars().all(|c| c == '0' || c == '.') => rest.to_string(),
        _ => s,
    }
}

fn strip_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

/// Splits a line (with its ending) into content and ending (`\r\n`, `\n` or nothing).
fn split_ending(line: &str) -> (&str, &str) {
    if let Some(c) = line.strip_suffix("\r\n") {
        (c, "\r\n")
    } else if let Some(c) = line.strip_suffix('\n') {
        (c, "\n")
    } else {
        (line, "")
    }
}

/// Patches `CameraSettings.ini` text: the value of each known key changes, nothing else.
///
/// # Errors
/// [`CameraPatchError::MissingKey`] when `CockpitCam\FOV`, `Driver\FOV` or
/// `Driver\DefaultOffsetFromSteeringWheelY` is absent.
pub fn patch_settings_text(text: &str, patch: &CameraPatch) -> Result<String, CameraPatchError> {
    let text = strip_bom(text);
    let keys = patch.settings();
    let mut found = [false; 10];
    let mut out = String::with_capacity(text.len() + 64);
    for line in text.split_inclusive('\n') {
        let (content, ending) = split_ending(line);
        let mut parts = content.splitn(2, ' ');
        let key = parts.next().unwrap_or("");
        let value = parts.next();
        let hit = keys.iter().position(|(k, _, _)| k.eq_ignore_ascii_case(key)).zip(value);
        match hit {
            Some((i, old)) => {
                found[i] = true;
                let (_, v, _) = keys[i];
                let trimmed = old.trim();
                let lead = &old[..old.len() - old.trim_start().len()];
                let trail = &old[old.trim_end().len()..];
                out.push_str(key);
                out.push(' ');
                out.push_str(lead);
                out.push_str(&fmt_like(trimmed, v));
                out.push_str(trail);
                out.push_str(ending);
            }
            None => out.push_str(line),
        }
    }
    for (i, (k, _, required)) in keys.iter().enumerate() {
        if *required && !found[i] {
            return Err(CameraPatchError::MissingKey(k));
        }
    }
    Ok(out)
}

/// Replaces the value of `attr="..."` in `line`; `None` when the attribute is absent.
fn set_attr(line: &str, attr: &str, value: &str) -> Option<String> {
    let needle = format!(" {attr}=\"");
    let start = line.find(&needle)? + needle.len();
    let end = start + line[start..].find('"')?;
    let mut out = String::with_capacity(line.len() + 8);
    out.push_str(&line[..start]);
    out.push_str(value);
    out.push_str(&line[end..]);
    Some(out)
}

/// Whether a `<Layer .../>` line is a fixed offset in car space.
fn is_fixed_car_offset(line: &str) -> bool {
    line.contains("<Layer")
        && line.contains("InputType=\"ConstantOne\"")
        && line.contains("NoiseType=\"None\"")
        && line.contains("OutputType=\"CarSpaceXYZOffset\"")
}

/// Patches `CameraPhysics.xml` text: `Param.x/y/z` of the last fixed CarSpaceXYZOffset layer in
/// the `<DriverCam>` block (the seat), nothing else. The hood block is never touched.
///
/// # Errors
/// [`CameraPatchError::MissingSeatLayer`] or [`CameraPatchError::MissingAttribute`].
pub fn patch_physics_text(text: &str, patch: &CameraPatch) -> Result<String, CameraPatchError> {
    let text = strip_bom(text);
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut in_driver = false;
    let mut target = None;
    for (i, line) in lines.iter().enumerate() {
        if line.contains("<DriverCam") {
            in_driver = true;
        } else if line.contains("</DriverCam>") {
            in_driver = false;
        } else if in_driver && is_fixed_car_offset(line) {
            target = Some(i);
        }
    }
    let i = target.ok_or(CameraPatchError::MissingSeatLayer)?;
    let (content, ending) = split_ending(lines[i]);
    let mut l = content.to_string();
    for (attr, v) in [
        ("Param.x", patch.seat[0]),
        ("Param.y", patch.seat[1]),
        ("Param.z", patch.seat[2]),
    ] {
        l = set_attr(&l, attr, &format!("{v:.6}")).ok_or(CameraPatchError::MissingAttribute(attr))?;
    }
    let mut out = String::with_capacity(text.len() + 16);
    for (j, line) in lines.iter().enumerate() {
        if j == i {
            out.push_str(&l);
            out.push_str(ending);
        } else {
            out.push_str(line);
        }
    }
    Ok(out)
}

/// The two camera files of an install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraFiles {
    /// `CameraSettings.ini`.
    pub settings: PathBuf,
    /// `CameraPhysics.xml`.
    pub physics: PathBuf,
}

impl CameraFiles {
    /// The files under `install\mediapc\Camera`.
    #[must_use]
    pub fn for_install(install: &Path) -> Self {
        Self::in_dir(&camera_dir(install))
    }

    /// The files directly in `dir` (tests and copies).
    #[must_use]
    pub fn in_dir(dir: &Path) -> Self {
        Self {
            settings: dir.join(SETTINGS_FILE),
            physics: dir.join(PHYSICS_FILE),
        }
    }

    fn read(&self) -> Result<(String, String), CameraPatchError> {
        let s = fs::read_to_string(&self.settings).map_err(io_at(&self.settings))?;
        let p = fs::read_to_string(&self.physics).map_err(io_at(&self.physics))?;
        Ok((s, p))
    }
}

/// `%LOCALAPPDATA%\FH6WheelDash\backups`.
#[must_use]
pub fn default_backup_root() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(base).join("FH6WheelDash").join("backups"))
}

/// `YYYYMMDD-HHMMSS` (UTC) for a time.
#[must_use]
pub fn timestamp(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let rem = secs % 86_400;
    // Civil-from-days (H. Hinnant), proleptic Gregorian.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}{m:02}{d:02}-{:02}{:02}{:02}",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// The first (original) backup under `root`: the oldest timestamped folder holding both files.
#[must_use]
pub fn original_backup(root: &Path) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.join(SETTINGS_FILE).is_file() && p.join(PHYSICS_FILE).is_file())
        .collect();
    dirs.sort();
    dirs.into_iter().next()
}

/// Copies the originals to a new timestamped folder under `root` unless a backup exists, and
/// verifies the copy. Returns the original backup folder.
///
/// # Errors
/// I/O errors, or [`CameraPatchError::BackupMismatch`] when the copy differs.
pub fn ensure_backup(files: &CameraFiles, root: &Path) -> Result<PathBuf, CameraPatchError> {
    if let Some(existing) = original_backup(root) {
        return Ok(existing);
    }
    let stamp = timestamp(SystemTime::now());
    let mut dir = root.join(&stamp);
    let mut n = 1;
    while dir.exists() {
        dir = root.join(format!("{stamp}-{n}"));
        n += 1;
    }
    fs::create_dir_all(&dir).map_err(io_at(&dir))?;
    for (src, name) in [(&files.settings, SETTINGS_FILE), (&files.physics, PHYSICS_FILE)] {
        let dst = dir.join(name);
        let bytes = fs::read(src).map_err(io_at(src))?;
        fs::write(&dst, &bytes).map_err(io_at(&dst))?;
        if fs::read(&dst).map_err(io_at(&dst))? != bytes {
            return Err(CameraPatchError::BackupMismatch(dst));
        }
    }
    tracing::info!(dir = %dir.display(), "original camera files backed up");
    Ok(dir)
}

/// What [`apply`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// The files already match; nothing written, no backup needed.
    UpToDate,
    /// Written. The original backup folder.
    Applied(PathBuf),
    /// The game is running; nothing written. Apply again after it exits.
    Queued,
}

/// Brings the camera files in line with `patch`. Backs up the originals first.
///
/// # Errors
/// [`CameraPatchError`]; on any error before the writes nothing changed, and each write is
/// atomic.
pub fn apply(
    files: &CameraFiles,
    patch: &CameraPatch,
    backup_root: &Path,
    game_running: bool,
) -> Result<ApplyOutcome, CameraPatchError> {
    let (settings, physics) = files.read()?;
    let new_settings = patch_settings_text(&settings, patch)?;
    let new_physics = patch_physics_text(&physics, patch)?;
    let settings_changed = new_settings != settings;
    let physics_changed = new_physics != physics;
    if !settings_changed && !physics_changed {
        return Ok(ApplyOutcome::UpToDate);
    }
    if game_running {
        return Ok(ApplyOutcome::Queued);
    }
    let backup = ensure_backup(files, backup_root)?;
    if settings_changed {
        write_atomically(&files.settings, new_settings.as_bytes()).map_err(io_at(&files.settings))?;
    }
    if physics_changed {
        write_atomically(&files.physics, new_physics.as_bytes()).map_err(io_at(&files.physics))?;
    }
    tracing::info!(settings_changed, physics_changed, "camera files written");
    Ok(ApplyOutcome::Applied(backup))
}

/// Restores the original files from the first backup.
///
/// # Errors
/// [`CameraPatchError::GameRunning`], [`CameraPatchError::NoBackup`] or I/O errors.
pub fn restore(files: &CameraFiles, backup_root: &Path, game_running: bool) -> Result<PathBuf, CameraPatchError> {
    if game_running {
        return Err(CameraPatchError::GameRunning);
    }
    let dir = original_backup(backup_root).ok_or(CameraPatchError::NoBackup)?;
    for (dst, name) in [(&files.settings, SETTINGS_FILE), (&files.physics, PHYSICS_FILE)] {
        let src = dir.join(name);
        let bytes = fs::read(&src).map_err(io_at(&src))?;
        write_atomically(dst, &bytes).map_err(io_at(dst))?;
    }
    tracing::info!(dir = %dir.display(), "original camera files restored");
    Ok(dir)
}

/// Binds a car ordinal to a brand in `car_brands.json`. See [`crate::car_brands`].
///
/// # Errors
/// See [`BindCarBrandError`]; the file is untouched on error.
pub fn bind_car_brand(path: impl AsRef<Path>, car: CarBinding<'_>) -> Result<BindOutcome, BindCarBrandError> {
    crate::car_brands::bind_car_brand(path.as_ref(), car)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A fresh folder holding copies of the repo's `data/` camera files (never the game's).
    struct Sandbox {
        root: PathBuf,
    }

    impl Sandbox {
        fn new(name: &str) -> Self {
            static N: AtomicU32 = AtomicU32::new(0);
            let root = std::env::temp_dir().join(format!(
                "fh6_campatch_{name}_{}_{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&root);
            let cam = camera_dir(&root.join("game"));
            fs::create_dir_all(&cam).unwrap();
            let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
            fs::copy(data.join(SETTINGS_FILE), cam.join(SETTINGS_FILE)).unwrap();
            fs::copy(data.join(PHYSICS_FILE), cam.join(PHYSICS_FILE)).unwrap();
            Self { root }
        }
        fn files(&self) -> CameraFiles {
            CameraFiles::for_install(&self.root.join("game"))
        }
        fn backups(&self) -> PathBuf {
            self.root.join("backups")
        }
    }

    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn cfg() -> AppConfig {
        AppConfig {
            wheel_less_cockpit_fov: 95.0,
            standard_cockpit_fov: 72.5,
            driver_seat_height_m: -0.06,
            steering_wheel_offset_y_m: -0.05,
            ..AppConfig::default()
        }
    }

    fn line_of<'a>(text: &'a str, key: &str) -> &'a str {
        text.lines().find(|l| l.starts_with(key)).unwrap_or("")
    }

    #[test]
    fn writes_the_real_cockpit_fov_key_and_keeps_every_other_byte() {
        let sb = Sandbox::new("fov");
        let files = sb.files();
        let before = fs::read_to_string(&files.settings).unwrap();
        let out = apply(&files, &CameraPatch::from_config(&cfg()), &sb.backups(), false).unwrap();
        assert!(matches!(out, ApplyOutcome::Applied(_)));
        let after = fs::read_to_string(&files.settings).unwrap();
        assert_eq!(line_of(&after, r"CockpitCam\FOV "), r"CockpitCam\FOV 95");
        assert_eq!(line_of(&after, r"Driver\FOV "), r"Driver\FOV 72.5");
        assert_eq!(line_of(&after, r"CockpitCam\MinFOV"), r"CockpitCam\MinFOV 40");
        assert!(after.ends_with("\r\n"), "trailing CRLF kept");
        assert!(!after.starts_with('\u{feff}'), "no BOM");
        assert_eq!(after.matches("\r\n").count(), before.matches("\r\n").count());
        assert_eq!(after.lines().count(), before.lines().count());
        let changed: Vec<_> = before.lines().zip(after.lines()).filter(|(a, b)| a != b).collect();
        assert_eq!(
            changed.len(),
            4,
            "cockpit FOV, driver FOV, wheel Y, hood FOV: {changed:?}"
        );
    }

    #[test]
    fn writes_the_seat_param_y_in_drivercam_only() {
        let sb = Sandbox::new("seat");
        let files = sb.files();
        let before = fs::read_to_string(&files.physics).unwrap();
        apply(&files, &CameraPatch::from_config(&cfg()), &sb.backups(), false).unwrap();
        let after = fs::read_to_string(&files.physics).unwrap();
        let changed: Vec<(usize, &str)> = before
            .lines()
            .zip(after.lines())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (_, b))| (i, b))
            .collect();
        assert_eq!(changed.len(), 1, "one layer line changes");
        let (i, line) = changed[0];
        let driver = before.lines().position(|l| l.contains("<DriverCam")).unwrap();
        let driver_end = before.lines().position(|l| l.contains("</DriverCam>")).unwrap();
        assert!(i > driver && i < driver_end, "inside DriverCam");
        assert!(line.contains("OutputType=\"CarSpaceXYZOffset\""));
        assert!(line.contains("Param.y=\"-0.060000\""));
        assert!(line.contains("Param.z=\"-0.500000\""), "z from CockpitOffsetZ");
        assert_eq!(after.len(), before.len(), "same width numbers, nothing else touched");
    }

    #[test]
    fn writes_the_wheel_offset() {
        let sb = Sandbox::new("wheel");
        let files = sb.files();
        apply(&files, &CameraPatch::from_config(&cfg()), &sb.backups(), false).unwrap();
        let after = fs::read_to_string(&files.settings).unwrap();
        assert_eq!(
            line_of(&after, r"Driver\DefaultOffsetFromSteeringWheelY"),
            r"Driver\DefaultOffsetFromSteeringWheelY -0.050"
        );
        assert_eq!(
            line_of(&after, r"Driver\DefaultOffsetFromSteeringWheelZ"),
            r"Driver\DefaultOffsetFromSteeringWheelZ 0.02"
        );
    }

    #[test]
    fn backs_up_once_and_restores_in_one_call() {
        let sb = Sandbox::new("backup");
        let files = sb.files();
        let orig_s = fs::read(&files.settings).unwrap();
        let orig_p = fs::read(&files.physics).unwrap();
        let ApplyOutcome::Applied(first) =
            apply(&files, &CameraPatch::from_config(&cfg()), &sb.backups(), false).unwrap()
        else {
            panic!("expected a write");
        };
        assert_eq!(fs::read(first.join(SETTINGS_FILE)).unwrap(), orig_s);
        assert_eq!(fs::read(first.join(PHYSICS_FILE)).unwrap(), orig_p);
        let name = first.file_name().and_then(|n| n.to_str()).unwrap_or("");
        assert_eq!(name.len(), 15, "YYYYMMDD-HHMMSS: {name}");
        // A second change keeps the first (original) backup.
        let more = AppConfig {
            wheel_less_cockpit_fov: 100.0,
            ..cfg()
        };
        let ApplyOutcome::Applied(second) =
            apply(&files, &CameraPatch::from_config(&more), &sb.backups(), false).unwrap()
        else {
            panic!("expected a write");
        };
        assert_eq!(second, first);
        assert_eq!(fs::read_dir(sb.backups()).unwrap().count(), 1);
        assert_eq!(restore(&files, &sb.backups(), false).unwrap(), first);
        assert_eq!(fs::read(&files.settings).unwrap(), orig_s);
        assert_eq!(fs::read(&files.physics).unwrap(), orig_p);
    }

    #[test]
    fn nothing_is_written_while_the_game_runs() {
        let sb = Sandbox::new("running");
        let files = sb.files();
        let orig = fs::read(&files.settings).unwrap();
        let out = apply(&files, &CameraPatch::from_config(&cfg()), &sb.backups(), true).unwrap();
        assert_eq!(out, ApplyOutcome::Queued);
        assert_eq!(fs::read(&files.settings).unwrap(), orig);
        assert!(!sb.backups().exists(), "no backup either");
        assert!(matches!(
            restore(&files, &sb.backups(), true),
            Err(CameraPatchError::GameRunning)
        ));
        assert!(matches!(
            restore(&files, &sb.backups(), false),
            Err(CameraPatchError::NoBackup)
        ));
    }

    #[test]
    fn defaults_match_the_shipped_files_so_nothing_is_written() {
        let sb = Sandbox::new("same");
        let patch = CameraPatch::from_config(&AppConfig {
            bonnet_fov: 78.0,
            ..AppConfig::default()
        });
        assert_eq!(
            apply(&sb.files(), &patch, &sb.backups(), false).unwrap(),
            ApplyOutcome::UpToDate
        );
        assert!(!sb.backups().exists());
    }

    #[test]
    fn missing_keys_or_layers_leave_files_untouched() {
        let patch = CameraPatch::from_config(&cfg());
        assert!(matches!(
            patch_settings_text("Driver\\FOV 80\r\n", &patch),
            Err(CameraPatchError::MissingKey(k)) if k == r"CockpitCam\FOV"
        ));
        assert!(matches!(
            patch_physics_text("<Hood>\r\n<Layer InputType=\"ConstantOne\" NoiseType=\"None\" OutputType=\"CarSpaceXYZOffset\" Param.y=\"0\"/>\r\n</Hood>\r\n", &patch),
            Err(CameraPatchError::MissingSeatLayer)
        ));
        let sb = Sandbox::new("broken");
        let files = sb.files();
        fs::write(&files.physics, "<CameraPhysics/>\r\n").unwrap();
        let orig = fs::read(&files.settings).unwrap();
        assert!(apply(&files, &patch, &sb.backups(), false).is_err());
        assert_eq!(fs::read(&files.settings).unwrap(), orig, "settings untouched too");
    }

    #[test]
    fn values_are_clamped_and_formatted_like_the_file() {
        let wild = AppConfig {
            wheel_less_cockpit_fov: 300.0,
            standard_cockpit_fov: f32::NAN,
            driver_seat_height_m: -9.0,
            ..AppConfig::default()
        };
        let p = CameraPatch::from_config(&wild);
        assert_eq!(p.cockpit_fov, 110.0);
        assert_eq!(p.driver_fov, 80.0);
        assert_eq!(p.seat[1], -0.3);
        assert_eq!(fmt_like("80", 95.0), "95");
        assert_eq!(fmt_like("62.0", 70.5), "70.5");
        assert_eq!(fmt_like("-0.035", -0.04), "-0.040");
        assert_eq!(fmt_like("0", -0.0), "0");
        assert_eq!(fmt_like("0", 0.25), "0.25");
    }

    #[test]
    fn timestamps_are_civil_utc() {
        assert_eq!(timestamp(UNIX_EPOCH), "19700101-000000");
        assert_eq!(
            timestamp(UNIX_EPOCH + std::time::Duration::from_secs(951_782_400 + 3661)),
            "20000229-010101"
        );
    }
}
