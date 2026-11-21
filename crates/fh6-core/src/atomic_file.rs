//! Crash-safe file replacement: write a uniquely named temp file next to the target, flush it
//! to disk, then rename it over the target. A crash or power loss leaves either the old file
//! or the new one, never an empty or half-written file.
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A temp path beside `target`, unique per process and per call.
fn temp_path(target: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = target.file_name().map_or_else(|| "file".into(), |n| n.to_string_lossy().into_owned());
    target.with_file_name(format!(".{name}.{}.{n}.tmp", std::process::id()))
}

/// Replaces `target` with `bytes` atomically (UTF-8 callers pass no BOM).
///
/// # Errors
/// Returns the I/O error from creating, writing, syncing or renaming the temp file. The temp
/// file is removed on failure and `target` is left as it was.
pub fn write_atomically(target: &Path, bytes: &[u8]) -> io::Result<()> {
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
    fn temp_names_are_unique() {
        let t = Path::new("x/config.json");
        assert_ne!(temp_path(t), temp_path(t));
    }
}
