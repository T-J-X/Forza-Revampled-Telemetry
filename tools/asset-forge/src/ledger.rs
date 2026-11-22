//! The asset ledger: one record per file, written to `assets/assets.json` (machine-readable, for
//! the kit and the contact sheet) and `assets/ASSETS.md` (the human ledger), plus
//! `assets/forge.lock.json` (job fingerprints that make `build` idempotent).
use crate::error::{io_err, ForgeError, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One shipped file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetRecord {
    /// Stable id, e.g. `glass/plate_rrect_normal`.
    pub id: String,
    /// Path relative to the repo root.
    pub path: String,
    /// Set: glass, fluid, scifi, classic, adas, logos, fonts.
    pub set: String,
    /// What it is: color, normal, data, mask, flipbook, mesh, logo, font, vector, licence.
    pub kind: String,
    /// `srgb`, `linear` or `n/a`.
    pub colorspace: String,
    /// `none`, `straight`, `premultiplied` or `additive` (RGB is light; alpha unused).
    pub alpha: String,
    /// Tiles seamlessly in both directions.
    pub tileable: bool,
    /// Flipbook layout, when it is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flipbook: Option<Flipbook>,
    /// Pixel size (images) or triangle count (meshes), as text.
    pub dims: String,
    /// File size in bytes.
    pub bytes: u64,
    /// How it was made: `blender:<recipe> seed=<n>`, `comfyui:<workflow> seed=<n>`,
    /// `rust:<module> seed=<n>` or the source URL.
    pub generator: String,
    /// Licence (SPDX or a short phrase).
    pub licence: String,
    /// Clusters / families it is meant for.
    pub clusters: String,
    /// QA results (seam scores, triangle budget), free text.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub qa: String,
    /// Small preview for the contact sheet, relative to the repo root.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub thumb: String,
}

/// Atlas layout of a flipbook.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Flipbook {
    /// Columns.
    pub cols: u32,
    /// Rows.
    pub rows: u32,
    /// Frames used (row-major from the top-left).
    pub frames: u32,
    /// Suggested playback rate.
    pub fps: u32,
    /// Cell size in pixels.
    pub cell: u32,
}

/// All records plus job fingerprints.
#[derive(Debug, Default)]
pub struct Ledger {
    records: BTreeMap<String, AssetRecord>,
    lock: BTreeMap<String, String>,
    assets_dir: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct AssetsFile {
    version: u32,
    doc: String,
    assets: Vec<AssetRecord>,
}

impl Ledger {
    /// Loads the existing ledger and lock (missing files start empty).
    pub fn load(assets_dir: &Path) -> Result<Self> {
        let mut ledger = Self {
            assets_dir: assets_dir.to_path_buf(),
            ..Self::default()
        };
        let json = assets_dir.join("assets.json");
        if json.is_file() {
            let text = std::fs::read_to_string(&json).map_err(|e| io_err(&json, e))?;
            let file: AssetsFile = serde_json::from_str(&text).map_err(|source| ForgeError::Json {
                context: json.display().to_string(),
                source,
            })?;
            for r in file.assets {
                ledger.records.insert(r.id.clone(), r);
            }
        }
        let lock = assets_dir.join("forge.lock.json");
        if lock.is_file() {
            let text = std::fs::read_to_string(&lock).map_err(|e| io_err(&lock, e))?;
            ledger.lock = serde_json::from_str(&text).map_err(|source| ForgeError::Json {
                context: lock.display().to_string(),
                source,
            })?;
        }
        Ok(ledger)
    }

    /// True when job `job` already ran with this fingerprint and all its records' files exist.
    pub fn is_fresh(&self, root: &Path, job: &str, fingerprint: &str) -> bool {
        if self.lock.get(job).map(String::as_str) != Some(fingerprint) {
            return false;
        }
        let prefix = format!("{job}/");
        let mut any = false;
        for r in self
            .records
            .values()
            .filter(|r| r.id.starts_with(&prefix) || r.id == job)
        {
            any = true;
            if !root.join(&r.path).is_file() {
                return false;
            }
        }
        any
    }

    /// Paths (files and thumbnails) recorded for `job`.
    pub fn job_files(&self, job: &str) -> Vec<String> {
        let prefix = format!("{job}/");
        self.records
            .values()
            .filter(|r| r.id.starts_with(&prefix) || r.id == job)
            .flat_map(|r| [r.path.clone(), r.thumb.clone()])
            .filter(|p| !p.is_empty())
            .collect()
    }

    /// Drops every record of `job` (before it is rebuilt).
    pub fn clear_job(&mut self, job: &str) {
        let prefix = format!("{job}/");
        self.records.retain(|id, _| !(id.starts_with(&prefix) || id == job));
    }

    /// Records the job's fingerprint after it succeeded.
    pub fn mark(&mut self, job: &str, fingerprint: &str) {
        self.lock.insert(job.to_owned(), fingerprint.to_owned());
    }

    /// Adds or replaces a record.
    pub fn put(&mut self, record: AssetRecord) {
        self.records.insert(record.id.clone(), record);
    }

    /// Every record, sorted by id.
    pub fn records(&self) -> impl Iterator<Item = &AssetRecord> {
        self.records.values()
    }

    /// Writes `assets.json`, `forge.lock.json` and `ASSETS.md` (UTF-8, no BOM).
    pub fn save(&mut self) -> Result<()> {
        // Records that belong to no job in the lock (ids renamed by a recipe change) are orphans.
        let jobs: Vec<String> = self.lock.keys().cloned().collect();
        self.records.retain(|id, _| {
            jobs.iter()
                .any(|j| id == j || id.strip_prefix(j.as_str()).is_some_and(|t| t.starts_with('/')))
        });
        self.sweep()?;
        let file = AssetsFile {
            version: 1,
            doc: "Generated by tools/asset-forge. One record per shipped asset; see ASSETS.md for the human ledger."
                .to_owned(),
            assets: self.records.values().cloned().collect(),
        };
        let json = serde_json::to_string_pretty(&file).map_err(|source| ForgeError::Json {
            context: "assets.json".to_owned(),
            source,
        })?;
        write(&self.assets_dir.join("assets.json"), &(json + "\n"))?;
        let lock = serde_json::to_string_pretty(&self.lock).map_err(|source| ForgeError::Json {
            context: "forge.lock.json".to_owned(),
            source,
        })?;
        write(&self.assets_dir.join("forge.lock.json"), &(lock + "\n"))?;
        write(&self.assets_dir.join("ASSETS.md"), &self.markdown())?;
        Ok(())
    }

    /// Deletes files the ledger no longer knows: thumbnails without a record, and files inside the
    /// folders asset-forge owns (left behind by renamed or removed jobs).
    fn sweep(&self) -> Result<()> {
        let known: std::collections::HashSet<&str> = self
            .records
            .values()
            .flat_map(|r| [r.path.as_str(), r.thumb.as_str()])
            .collect();
        let mut dirs = vec![self.assets_dir.join("_thumbs")];
        dirs.extend(FORGE_DIRS.iter().map(|d| self.assets_dir.join(d)));
        for dir in dirs {
            for file in walk(&dir) {
                let rel = self.repo_rel(&file);
                if !known.contains(rel.as_str()) {
                    std::fs::remove_file(&file).map_err(|e| io_err(&file, e))?;
                }
            }
        }
        Ok(())
    }

    /// `assets/<...>` with forward slashes, as records store paths.
    fn repo_rel(&self, file: &Path) -> String {
        let tail = file.strip_prefix(&self.assets_dir).unwrap_or(file);
        let parts: Vec<String> = tail
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        format!("assets/{}", parts.join("/"))
    }

    /// Files in `assets/` that asset-forge did not make (the v1 import, reference photos, brand
    /// data), so the ledger still names a licence for every file.
    fn other_files(&self) -> Vec<(String, u64)> {
        let mut out = Vec::new();
        for file in walk(&self.assets_dir) {
            let rel = self.repo_rel(&file);
            let top = rel
                .trim_start_matches("assets/")
                .split('/')
                .next()
                .unwrap_or_default()
                .to_owned();
            let generated = ["assets.json", "ASSETS.md", "contact-sheet.html", "forge.lock.json"];
            if FORGE_DIRS.contains(&top.as_str())
                || top == "_thumbs"
                || top == "_masters"
                || generated.contains(&top.as_str())
            {
                continue;
            }
            let bytes = std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0);
            out.push((rel, bytes));
        }
        out.sort();
        out
    }

    fn markdown(&self) -> String {
        let mut by_set: BTreeMap<&str, Vec<&AssetRecord>> = BTreeMap::new();
        for r in self.records.values() {
            by_set.entry(r.set.as_str()).or_default().push(r);
        }
        let total: u64 = self.records.values().map(|r| r.bytes).sum();
        let mut s = String::new();
        s.push_str("# Asset ledger\n\n");
        s.push_str(
            "Generated by `cargo run -p asset-forge -- build` (do not edit by hand; edit the recipe and rebuild). ",
        );
        s.push_str("Every row names its generator (script and seed, workflow and seed, or source URL), its licence, size and the clusters it is for. ");
        s.push_str("Machine-readable copy: [`assets.json`](assets.json). Review: [`contact-sheet.html`](contact-sheet.html).\n\n");
        s.push_str(&format!(
            "**{} files, {:.1} MB.** Colour space and alpha are recorded per file: `linear` maps must be sampled without sRGB decoding; ",
            self.records.len(),
            total as f64 / 1_048_576.0
        ));
        s.push_str("`straight` alpha is premultiplied by the loader, `premultiplied` is stored that way, `additive` means RGB is emitted light.\n\n");
        s.push_str("Licences: assets made by asset-forge are MIT like the repo; fonts are SIL OFL 1.1 (each folder keeps its `OFL.txt`); ");
        s.push_str("logos are trademarks of their owners, used only to identify the car (see [`logos/NOTICE.md`](logos/NOTICE.md)); CC0 fill-ins name their source.\n\n");
        for (set, rows) in by_set {
            let bytes: u64 = rows.iter().map(|r| r.bytes).sum();
            s.push_str(&format!(
                "## {set} ({} files, {:.1} MB)\n\n",
                rows.len(),
                bytes as f64 / 1_048_576.0
            ));
            s.push_str("| Path | Kind | Space / alpha | Dims | Size | Generator | Licence | Clusters | QA |\n");
            s.push_str("|---|---|---|---|---|---|---|---|---|\n");
            for r in rows {
                let mut kind = r.kind.clone();
                if r.tileable {
                    kind.push_str(", tileable");
                }
                if let Some(f) = r.flipbook {
                    kind.push_str(&format!(
                        ", flipbook {}x{} ({} f @ {} fps, {} px)",
                        f.cols, f.rows, f.frames, f.fps, f.cell
                    ));
                }
                s.push_str(&format!(
                    "| `{}` | {} | {} / {} | {} | {} | {} | {} | {} | {} |\n",
                    r.path,
                    kind,
                    r.colorspace,
                    r.alpha,
                    r.dims,
                    human(r.bytes),
                    r.generator.replace('|', "/"),
                    r.licence,
                    r.clusters,
                    r.qa.replace('|', "/")
                ));
            }
            s.push('\n');
        }
        let others = self.other_files();
        if !others.is_empty() {
            s.push_str(&format!(
                "## Files not made by asset-forge ({} files)\n\n",
                others.len()
            ));
            s.push_str("| Path | Size | Origin | Licence |\n|---|---|---|---|\n");
            for (rel, bytes) in others {
                let (origin, licence) = other_origin(&rel);
                s.push_str(&format!("| `{rel}` | {} | {origin} | {licence} |\n", human(bytes)));
            }
            s.push('\n');
        }
        s
    }
}

/// Folders under `assets/` that asset-forge owns completely (swept of unknown files on save).
const FORGE_DIRS: [&str; 7] = ["glass", "fluid", "scifi", "classic", "adas", "fonts", "logos"];

/// Origin and licence of a file asset-forge did not make.
fn other_origin(rel: &str) -> (&'static str, &'static str) {
    if rel.starts_with("assets/reference/") {
        (
            "Reference photo from the initial import; design reference only, never loaded or shipped",
            "Reference use only (not redistributed in builds)",
        )
    } else if rel.starts_with("assets/car_brands/") {
        ("Project data (brand palettes and car list)", "MIT (repo)")
    } else {
        (
            "v1 project texture from the initial import, superseded by the asset-forge sets",
            "MIT (repo)",
        )
    }
}

/// Every file under `dir`, recursively (empty when it does not exist).
fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(p);
            }
        }
    }
    out
}

/// `1.2 MB` / `340 KB`.
pub fn human(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{} KB", bytes.div_ceil(1024))
    }
}

fn write(path: &Path, text: &str) -> Result<()> {
    std::fs::write(path, text.as_bytes()).map_err(|e| io_err(path, e))
}

/// FNV-1a 64 of several parts, as hex: the fingerprint of a job (recipe source, parameters, seed).
pub fn fingerprint(parts: &[&[u8]]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in parts {
        for b in p.iter().chain(&[0xff]) {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_changes_with_any_part() {
        let a = fingerprint(&[b"recipe", b"seed=1"]);
        let b = fingerprint(&[b"recipe", b"seed=2"]);
        assert_ne!(a, b);
        assert_eq!(a, fingerprint(&[b"recipe", b"seed=1"]));
    }
}
