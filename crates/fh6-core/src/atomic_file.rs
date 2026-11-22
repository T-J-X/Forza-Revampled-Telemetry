//! Crash-safe file replacement: write a uniquely named temp file next to the target, flush it
//! to disk, then rename it over the target. A crash or power loss leaves either the old file
//! or the new one, never an empty or half-written file. Temp files a crashed writer left
//! behind are removed by the next save of the same target.
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// A temp file from another process this old is an orphan (no save takes anywhere near this).
const STALE_TEMP_AGE: Duration = Duration::from_secs(60);

/// A temp path beside `target`, unique per process and per call.
fn temp_path(target: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = target
        .file_name()
        .map_or_else(|| "file".into(), |n| n.to_string_lossy().into_owned());
    target.with_file_name(format!(".{name}.{}.{n}.tmp", std::process::id()))
}

/// Best-effort removal of `.{name}.<pid>.<n>.tmp` files beside `target` that another process
/// left behind (killed or crashed mid-save) and that are older than [`STALE_TEMP_AGE`].
fn sweep_stale_temps(target: &Path) {
    let Some(name) = target.file_name() else { return };
    let dir = match target.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir,
        _ => Path::new("."),
    };
    let prefix = format!(".{}.", name.to_string_lossy());
    let own = format!("{prefix}{}.", std::process::id());
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.filter_map(Result::ok) {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if !file_name.starts_with(&prefix) || !file_name.ends_with(".tmp") || file_name.starts_with(&own) {
            continue;
        }
        let stale = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age >= STALE_TEMP_AGE);
        if stale {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Replaces `target` with `bytes` atomically (UTF-8 callers pass no BOM).
///
/// # Errors
/// Returns the I/O error from creating, writing, syncing or renaming the temp file. The temp
/// file is removed on failure and `target` is left as it was.
pub fn write_atomically(target: &Path, bytes: &[u8]) -> io::Result<()> {
    sweep_stale_temps(target);
    let tmp = temp_path(target);
    let result = (|| {
        let mut file = File::create(&tmp)?;
        file.write_all(bytes)?;
        // Flush the data before the rename: NTFS journals the rename, not the contents.
        file.sync_all()?;
        drop(file);
        // `rename` replaces the destination (MoveFileEx with REPLACE_EXISTING on Windows).
        fs::rename(&tmp, target)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_contents_and_leaves_no_temp_file() {
        let dir = std::env::temp_dir().join(format!("fh6_atomic_{}", std::process::id()));
        fs::create_dir_all(&dir).expect("create dir");
        let target = dir.join("data.json");
        fs::write(&target, "old").expect("seed");
        write_atomically(&target, b"new").expect("write");
        assert_eq!(fs::read_to_string(&target).expect("read"), "new");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .expect("list")
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp files left: {leftovers:?}");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn orphaned_temp_files_are_swept() {
        let dir = std::env::temp_dir().join(format!("fh6_atomic_sweep_{}", std::process::id()));
        fs::create_dir_all(&dir).expect("create dir");
        let target = dir.join("config.json");
        let orphan = dir.join(".config.json.1.0.tmp");
        let in_progress = dir.join(".config.json.2.0.tmp");
        let unrelated = dir.join(".other.json.1.0.tmp");
        for path in [&orphan, &in_progress, &unrelated] {
            fs::write(path, "x").expect("seed temp");
        }
        let old = std::time::SystemTime::now() - Duration::from_secs(600);
        for path in [&orphan, &unrelated] {
            File::options()
                .write(true)
                .open(path)
                .and_then(|f| f.set_modified(old))
                .expect("age the file");
        }
        write_atomically(&target, b"{}").expect("write");
        assert!(!orphan.exists(), "an old temp of this target is removed");
        assert!(in_progress.exists(), "a recent temp may belong to a live writer");
        assert!(unrelated.exists(), "other targets' temps are not touched");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn temp_names_are_unique() {
        let t = Path::new("x/config.json");
        assert_ne!(temp_path(t), temp_path(t));
    }
}
