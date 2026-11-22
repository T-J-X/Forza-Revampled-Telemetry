//! The build context: paths, the ledger, idempotent jobs and the one call every set uses to
//! write an asset (file + thumbnail + ledger row).
use crate::error::{io_err, Result};
use crate::img::{self, Img};
use crate::ktx2w::{self, Encoding};
use crate::ledger::{AssetRecord, Flipbook, Ledger};
use crate::paths::Paths;
use std::path::{Path, PathBuf};

/// Longest side of contact-sheet thumbnails.
pub const THUMB: usize = 384;

/// Shared state for one `build` run.
pub struct Ctx {
    /// Tool and repo paths.
    pub paths: Paths,
    /// The ledger being rebuilt.
    pub ledger: Ledger,
    /// Rebuild even when fingerprints match.
    pub force: bool,
    /// Only run jobs whose id contains this text.
    pub only: Option<String>,
    agent: Option<ureq::Agent>,
}

/// How an image is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// 8-bit PNG (sRGB-tagged when the spec says `srgb`).
    Png8,
    /// 16-bit PNG.
    Png16,
    /// KTX2 RGBA8 with mips and zstd.
    Ktx2(Encoding),
}

/// The ledger fields that describe an asset.
#[derive(Debug, Clone)]
pub struct Spec {
    /// Set name.
    pub set: &'static str,
    /// Kind (color, normal, data, mask, flipbook, ...).
    pub kind: &'static str,
    /// `srgb`, `linear` or `n/a`.
    pub colorspace: &'static str,
    /// `none`, `straight`, `premultiplied` or `additive`.
    pub alpha: &'static str,
    /// Tiles seamlessly.
    pub tileable: bool,
    /// Flipbook layout.
    pub flipbook: Option<Flipbook>,
    /// Generator text.
    pub generator: String,
    /// Licence.
    pub licence: String,
    /// Intended clusters.
    pub clusters: &'static str,
    /// QA notes.
    pub qa: String,
}

impl Spec {
    /// A spec for a forge-made asset (MIT).
    pub fn new(set: &'static str, kind: &'static str, generator: impl Into<String>, clusters: &'static str) -> Self {
        let linear = matches!(kind, "normal" | "data" | "mask" | "flow" | "offset");
        Self {
            set,
            kind,
            colorspace: if linear { "linear" } else { "srgb" },
            alpha: "none",
            tileable: false,
            flipbook: None,
            generator: generator.into(),
            licence: "MIT (asset-forge)".to_owned(),
            clusters,
            qa: String::new(),
        }
    }

    /// Sets the alpha mode.
    pub fn alpha(mut self, a: &'static str) -> Self {
        self.alpha = a;
        self
    }

    /// Marks it tileable.
    pub fn tile(mut self) -> Self {
        self.tileable = true;
        self
    }

    /// Sets the colour space.
    pub fn space(mut self, s: &'static str) -> Self {
        self.colorspace = s;
        self
    }

    /// Adds a QA note.
    pub fn qa(mut self, q: impl Into<String>) -> Self {
        let q = q.into();
        if !self.qa.is_empty() {
            self.qa.push_str("; ");
        }
        self.qa.push_str(&q);
        self
    }

    /// Sets the flipbook layout.
    pub fn flip(mut self, f: Flipbook) -> Self {
        self.flipbook = Some(f);
        self
    }

    /// Sets the licence.
    pub fn licence(mut self, l: impl Into<String>) -> Self {
        self.licence = l.into();
        self
    }
}

impl Ctx {
    /// A context over the repo's current ledger.
    pub fn new(paths: Paths, force: bool, only: Option<String>) -> Result<Self> {
        let ledger = Ledger::load(&paths.assets())?;
        Ok(Self {
            paths,
            ledger,
            force,
            only,
            agent: None,
        })
    }

    /// The shared HTTP agent.
    pub fn agent(&mut self) -> ureq::Agent {
        self.agent.get_or_insert_with(crate::net::agent).clone()
    }

    /// Runs `body` unless the job is fresh. Its old records are dropped first; the fingerprint is
    /// stored only when it succeeds.
    pub fn job(&mut self, job: &str, fingerprint: &str, body: impl FnOnce(&mut Ctx) -> Result<()>) -> Result<()> {
        if let Some(only) = &self.only {
            if !job.contains(only.as_str()) {
                return Ok(());
            }
        }
        if !self.force && self.ledger.is_fresh(&self.paths.root, job, fingerprint) {
            println!("  = {job} (fresh)");
            return Ok(());
        }
        println!("  > {job}");
        let started = std::time::Instant::now();
        let old = self.ledger.job_files(job);
        self.ledger.clear_job(job);
        body(self)?;
        // Files the previous run wrote that this run no longer produces are removed.
        let now: std::collections::BTreeSet<String> = self.ledger.job_files(job).into_iter().collect();
        for f in old.into_iter().filter(|f| !now.contains(f)) {
            let _ = std::fs::remove_file(self.paths.root.join(&f));
        }
        self.ledger.mark(job, fingerprint);
        self.ledger.save()?;
        println!("  ✓ {job} ({:.1} s)", started.elapsed().as_secs_f32());
        Ok(())
    }

    /// Absolute path of a repo-relative path.
    pub fn abs(&self, rel: &str) -> PathBuf {
        self.paths.root.join(rel)
    }

    /// A scratch directory for a job (outside git).
    pub fn scratch(&self, name: &str) -> Result<PathBuf> {
        let d = self.paths.scratch.join(name);
        std::fs::create_dir_all(&d).map_err(|e| io_err(&d, e))?;
        Ok(d)
    }

    fn thumb_path(&self, id: &str) -> String {
        format!("assets/_thumbs/{}.png", id.replace('/', "__"))
    }

    /// Writes `img` to `rel` in `format`, a thumbnail and a ledger row with id `id`.
    pub fn image(&mut self, id: &str, rel: &str, img: &Img, format: Format, spec: Spec) -> Result<()> {
        let path = self.abs(rel);
        let srgb = spec.colorspace == "srgb";
        let bytes = match format {
            Format::Png8 => img::save_png(img, &path, false, srgb)?,
            Format::Png16 => img::save_png(img, &path, true, false)?,
            Format::Ktx2(enc) => {
                let data = ktx2w::encode(img, enc, spec.alpha == "premultiplied", true);
                img::write_file(&path, &data)?
            }
        };
        let thumb_rel = self.thumb_path(id);
        img::save_thumb(img, &self.abs(&thumb_rel), THUMB)?;
        let dims = format!("{}x{}", img.w, img.h);
        self.record(id, rel, bytes, dims, spec, thumb_rel);
        Ok(())
    }

    /// Records a file that was written some other way (SVG, glTF, font, licence).
    pub fn file(&mut self, id: &str, rel: &str, dims: String, spec: Spec, thumb: Option<&Img>) -> Result<()> {
        let path = self.abs(rel);
        let bytes = std::fs::metadata(&path).map_err(|e| io_err(&path, e))?.len();
        let thumb_rel = match thumb {
            Some(t) => {
                let tr = self.thumb_path(id);
                img::save_thumb(t, &self.abs(&tr), THUMB)?;
                tr
            }
            None => String::new(),
        };
        self.record(id, rel, bytes, dims, spec, thumb_rel);
        Ok(())
    }

    fn record(&mut self, id: &str, rel: &str, bytes: u64, dims: String, spec: Spec, thumb: String) {
        self.ledger.put(AssetRecord {
            id: id.to_owned(),
            path: rel.to_owned(),
            set: spec.set.to_owned(),
            kind: spec.kind.to_owned(),
            colorspace: spec.colorspace.to_owned(),
            alpha: spec.alpha.to_owned(),
            tileable: spec.tileable,
            flipbook: spec.flipbook,
            dims,
            bytes,
            generator: spec.generator,
            licence: spec.licence,
            clusters: spec.clusters.to_owned(),
            qa: spec.qa,
            thumb,
        });
    }
}

/// Reads a file to bytes with a path-carrying error.
pub fn read(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| io_err(path, e))
}
