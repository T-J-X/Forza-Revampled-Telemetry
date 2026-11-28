//! The logo zone: a brand logo fitted with brand-safe padding and optical centring, falling back
//! to the brand name, then the custom name, set in the wide (wordmark) face.
//!
//! [`LogoIndex`] reads `assets/logos/logos.json` once at start-up and says which brand a car
//! shows ([`LogoIndex::car_brand`]). A host keeps a [`CarLogo`] and tells it when the car
//! changes (never per frame). Its [`DialLoader`] decodes that brand's dial on a worker thread
//! ([`LogoIndex::decode_dial`] needs no device), uploads it over the frames that follow, and the
//! [`Logo`] the clusters draw then carries it in [`Logo::image`]. Tools decode and upload at once
//! with [`LogoIndex::load_dial`]. Only the levels from [`DIAL_TEXTURE_SIDE`] down are uploaded,
//! so a brand costs at most 1.33 MiB of video memory and all 91 dials together 47 MiB.
use crate::clock::Stamp;
use crate::cluster::Logo;
use crate::color::Color;
use crate::images::{self, ImageError, Images, MipChain, MipUpload};
use crate::math::{vec2, Rect, Vec2};
use crate::paint::{ImageId, ImagePaint, Paint};
use crate::painter::Painter;
use crate::shape::Shape;
use crate::text::{Align, FontId, TextStyle, VAlign};
use fh6_core::CarDatabase;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

/// Brand-safe padding as a fraction of the zone's shorter side.
pub const SAFE_PADDING: f32 = 0.12;
/// Optical centring: marks sit this fraction of the zone height above the geometric centre.
pub const OPTICAL_LIFT: f32 = 0.02;
/// Longest wordmark fallback, in characters (tokens `layout.logoZone`).
pub const MAX_NAME_CHARS: usize = 14;
/// The logo folder inside `assets/`.
pub const LOGO_DIR: &str = "logos";
/// The logo index inside [`LOGO_DIR`].
pub const INDEX_FILE: &str = "logos.json";
/// The generic emblem's key in the index: the car list's own key for it. It stands for "no
/// known brand" and is never shown as a brand's logo.
pub const GENERIC_BRAND: &str = fh6_core::cars::DEFAULT_BRAND_KEY;
/// The longest side a dial file may have. The shipped dials are 1024 px; a bigger file (one of
/// the 4096 px full logos named by mistake) is refused from its header, before decoding.
pub const MAX_DIAL_SIDE: u32 = 1024;
/// The longest side of the largest dial level that is uploaded. The widest logo zone (Liquid
/// Glass's 240 x 48, less its padding) draws a wide wordmark dial at most 320 px across at user
/// scale 1.4 on a 4K target, and a square dial at most 51 px, so the 1024 px level is never
/// sampled: the kit picks the level from the drawn size, and the levels kept are the same texels
/// as in the full chain. A brand then costs at most 1.33 MiB of video memory instead of 5.33.
pub const DIAL_TEXTURE_SIDE: u32 = 512;

/// How to draw the logo zone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogoStyle {
    /// Image tint (tokens: dial variant is mono white at alpha 215).
    pub tint: Color,
    /// Wordmark paint.
    pub text: Paint,
    /// Wordmark face (the `wide` role).
    pub font: FontId,
    /// Wordmark cap height in local units (tokens: 18).
    pub cap: f32,
    /// Wordmark tracking in em (tokens: +30%).
    pub tracking: f32,
}

impl LogoStyle {
    /// The tokens' wordmark style in `font`, white.
    #[must_use]
    pub fn new(font: FontId) -> Self {
        Self {
            tint: Color::WHITE,
            text: Paint::Solid(Color::WHITE),
            font,
            cap: 18.0,
            tracking: 0.30,
        }
    }
}

/// The rectangle an image of `image_size` occupies inside `zone`: aspect kept, inside the
/// brand-safe padding, then lifted 2% of the zone's height for optical centring (so a
/// height-limited image's top edge sits that much into the top padding).
#[must_use]
pub fn fit(zone: Rect, image_size: (u32, u32)) -> Rect {
    let pad = zone.width().min(zone.height()) * SAFE_PADDING;
    let inner = zone.inset(pad);
    let (iw, ih) = (image_size.0.max(1) as f32, image_size.1.max(1) as f32);
    if inner.is_empty() {
        return Rect::from_center_size(zone.center(), Vec2::ZERO);
    }
    let k = (inner.width() / iw).min(inner.height() / ih);
    let size = vec2(iw * k, ih * k);
    let center = inner.center() - vec2(0.0, zone.height() * OPTICAL_LIFT);
    Rect::from_center_size(center, size)
}

/// The wordmark text for a logo without an image: the brand name, else the custom name, upper
/// case, cut to [`MAX_NAME_CHARS`] characters before upper-casing (a letter such as "ß" can
/// grow to two). Writes into `buf` (no allocation).
#[must_use]
pub fn wordmark<'b>(logo: &Logo, buf: &'b mut [u8; 64]) -> &'b str {
    let name = if logo.brand_name.trim().is_empty() {
        logo.custom_name.trim()
    } else {
        logo.brand_name.trim()
    };
    // Too long: cut at the last word boundary inside the limit ("BMW M Motorsport" -> "BMW M").
    let name = if name.chars().count() > MAX_NAME_CHARS {
        let cut: usize = name.char_indices().nth(MAX_NAME_CHARS).map_or(name.len(), |(i, _)| i);
        let head = &name[..cut];
        head.rfind(char::is_whitespace).map_or(head, |i| head[..i].trim_end())
    } else {
        name
    };
    let mut len = 0;
    for ch in name.chars().take(MAX_NAME_CHARS) {
        for up in ch.to_uppercase() {
            let mut tmp = [0u8; 4];
            let s = up.encode_utf8(&mut tmp);
            if len + s.len() > buf.len() {
                break;
            }
            buf[len..len + s.len()].copy_from_slice(s.as_bytes());
            len += s.len();
        }
    }
    std::str::from_utf8(&buf[..len]).unwrap_or("")
}

/// Errors reading the logo index or loading a brand's logo.
#[derive(Debug, thiserror::Error)]
pub enum LogoError {
    /// The index file or a dial image could not be read.
    #[error("reading {path}: {source}")]
    Io {
        /// The file.
        path: String,
        /// The cause.
        source: std::io::Error,
    },
    /// The index file is not JSON shaped like a logo index.
    #[error("parsing the logo index: {0}")]
    Json(#[from] serde_json::Error),
    /// The index names no usable dial image for the brand (its key, or `?` for an id from
    /// another index).
    #[error("brand {0} has no usable dial image")]
    NoDial(String),
    /// The dial image is larger than [`MAX_DIAL_SIDE`] on a side.
    #[error("{path} is {width}x{height}; a dial may be at most {MAX_DIAL_SIDE} px a side")]
    TooLarge {
        /// The file.
        path: String,
        /// Its width in pixels.
        width: u32,
        /// Its height in pixels.
        height: u32,
    },
    /// The dial image could not be read or decoded.
    #[error(transparent)]
    Image(#[from] ImageError),
}

/// A brand in a [`LogoIndex`]. It only means something to the index that returned it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BrandId(usize);

/// One brand of the index.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Brand {
    /// The index key (`PORSCHE`), as `BrandKey` in `car_brands.json`.
    key: String,
    /// The brand as people write it ("Porsche").
    name: String,
    /// The dial variant, relative to the logo folder. `None` when the entry names none, or names
    /// a path that leaves the folder.
    dial: Option<PathBuf>,
}

/// `logos.json` as read; every other field of an entry (`full`, `mono`, `aspect`, ...) is
/// ignored. Maps iterate sorted by key, which keeps the index deterministic.
#[derive(Debug, Deserialize)]
struct IndexFile {
    #[serde(default)]
    logos: BTreeMap<String, IndexEntry>,
    #[serde(default)]
    aliases: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct IndexEntry {
    #[serde(default)]
    name: String,
    #[serde(default)]
    dial: String,
}

/// The brand logos listed in `assets/logos/logos.json`: which brand a car shows and where that
/// brand's dial image is. Read once at start-up; lookups run only when the car changes.
#[derive(Debug, Clone, Default)]
pub struct LogoIndex {
    /// The logo folder the dial paths are relative to.
    dir: PathBuf,
    /// Sorted by key; a [`BrandId`] indexes it.
    brands: Vec<Brand>,
    /// Make and key spellings, sorted, each with its brand.
    aliases: Vec<(String, BrandId)>,
}

impl LogoIndex {
    /// Reads `<assets>/logos/logos.json`.
    ///
    /// # Errors
    /// [`LogoError::Io`] when the file cannot be read, [`LogoError::Json`] when it is not a logo
    /// index. See [`LogoIndex::from_json`] for the entries left out.
    pub fn load(assets: &Path) -> Result<Self, LogoError> {
        let dir = assets.join(LOGO_DIR);
        let path = dir.join(INDEX_FILE);
        let text = std::fs::read_to_string(&path).map_err(|source| LogoError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_json(&text, &dir)
    }

    /// Builds an index from the JSON text of `logos.json`, with dial paths relative to `dir` (a
    /// leading BOM is tolerated). A dial path that is absolute or climbs out of `dir` is dropped
    /// (the brand keeps its name), and an alias to a brand the index does not list is left out;
    /// both with a warning.
    ///
    /// # Errors
    /// [`LogoError::Json`] when the text is not a logo index.
    pub fn from_json(text: &str, dir: &Path) -> Result<Self, LogoError> {
        let file: IndexFile = serde_json::from_str(text.trim_start_matches('\u{feff}'))?;
        let brands: Vec<Brand> = file
            .logos
            .into_iter()
            .map(|(key, entry)| {
                let dial = plain_relative(&entry.dial);
                if dial.is_none() && !entry.dial.trim().is_empty() {
                    tracing::warn!(brand = %key, dial = %entry.dial, "logo dial path leaves the logo folder; ignored");
                }
                Brand {
                    key,
                    name: entry.name.trim().to_string(),
                    dial,
                }
            })
            .collect();
        let mut aliases = Vec::with_capacity(file.aliases.len());
        for (spelling, key) in file.aliases {
            match brands.binary_search_by(|b| b.key.as_str().cmp(key.as_str())) {
                Ok(i) => aliases.push((spelling, BrandId(i))),
                Err(_) => tracing::warn!(alias = %spelling, brand = %key, "logo alias to an unlisted brand; ignored"),
            }
        }
        Ok(Self {
            dir: dir.to_path_buf(),
            brands,
            aliases,
        })
    }

    /// Number of brands, the generic emblem included.
    #[must_use]
    pub fn len(&self) -> usize {
        self.brands.len()
    }

    /// True when the index lists no brand (no `logos.json`).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.brands.is_empty()
    }

    /// The brand a car shows: its `BrandKey` when the index lists it, else its make through the
    /// aliases. A spelling matches exactly first, else when it differs only in ASCII case (the
    /// first such alias in byte order, so `"nissan"` finds `NISSAN` before `Nissan`). `None` for
    /// the generic emblem and for brands the index does not know.
    #[must_use]
    pub fn resolve(&self, brand_key: &str, make: &str) -> Option<BrandId> {
        let key = brand_key.trim();
        if !key.is_empty() && !key.eq_ignore_ascii_case(GENERIC_BRAND) {
            let by_key = self.by_key(key).or_else(|| self.by_alias(key));
            if let Some(id) = by_key.filter(|&id| !self.is_generic(id)) {
                return Some(id);
            }
        }
        let make = make.trim();
        if make.is_empty() {
            return None;
        }
        // An exact alias to the generic emblem means "no brand": no case-folded second try.
        self.by_alias(make).filter(|&id| !self.is_generic(id))
    }

    /// What car `ordinal` of `cars` (`car_brands.json`) shows in the logo zone. Allocates the
    /// name: call it when the car changes, never per frame.
    #[must_use]
    pub fn car_brand(&self, cars: &CarDatabase, ordinal: i32) -> CarBrand {
        let Some(car) = cars.car(ordinal) else {
            return CarBrand::default();
        };
        let logo = self.resolve(&car.brand_key, &car.manufacturer);
        let key = car.brand_key.trim();
        // The palette's name for a branded car, as the wordmark showed it before logos loaded.
        // The generic palette ("Standard Horizon") names a theme, not a brand.
        let palette_name = if key.is_empty() || key.eq_ignore_ascii_case(GENERIC_BRAND) {
            None
        } else {
            cars.brand_palette(key)
                .map(|p| p.display_name.trim())
                .filter(|n| !n.is_empty())
        };
        let name = palette_name
            .or_else(|| logo.and_then(|id| self.name(id)))
            .unwrap_or_default()
            .to_string();
        CarBrand { logo, name }
    }

    /// The brand's index key (`"PORSCHE"`).
    #[must_use]
    pub fn key(&self, id: BrandId) -> Option<&str> {
        self.brands.get(id.0).map(|b| b.key.as_str())
    }

    /// The brand as people write it (`"Porsche"`).
    #[must_use]
    pub fn name(&self, id: BrandId) -> Option<&str> {
        self.brands.get(id.0).map(|b| b.name.as_str())
    }

    /// The brand's dial image: 1024 px, white with alpha, tuned for small sizes on dark dials.
    #[must_use]
    pub fn dial_path(&self, id: BrandId) -> Option<PathBuf> {
        let dial = self.brands.get(id.0)?.dial.as_ref()?;
        Some(self.dir.join(dial))
    }

    /// Reads and decodes the brand's dial into a premultiplied mip chain whose largest level is
    /// at most [`DIAL_TEXTURE_SIDE`] px a side. It needs no device, so a host runs it on a worker
    /// thread and uploads the chain on the render thread ([`DialLoader`]). The file's header is
    /// checked first: a dial over [`MAX_DIAL_SIDE`] on either side is refused without decoding.
    ///
    /// # Errors
    /// [`LogoError::NoDial`] when the index names no usable dial image for the brand,
    /// [`LogoError::Io`] when the file cannot be read, [`LogoError::TooLarge`] when it is too
    /// big, and [`LogoError::Image`] when it cannot be decoded.
    pub fn decode_dial(&self, brand: BrandId) -> Result<MipChain, LogoError> {
        let path = self
            .dial_path(brand)
            .ok_or_else(|| LogoError::NoDial(self.key(brand).unwrap_or("?").to_string()))?;
        let bytes = std::fs::read(&path).map_err(|source| LogoError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let (width, height) = images::png_size(&bytes)?;
        if width.max(height) > MAX_DIAL_SIDE {
            return Err(LogoError::TooLarge {
                path: path.display().to_string(),
                width,
                height,
            });
        }
        Ok(images::decode_png_mips(&bytes, DIAL_TEXTURE_SIDE)?)
    }

    /// Decodes the brand's dial ([`LogoIndex::decode_dial`]) and uploads it into `images` on
    /// this thread: for tools such as `preview-export`. The live app decodes on a worker thread
    /// instead, and keeps one image per brand itself; this uploads a new texture on every call.
    ///
    /// # Errors
    /// The errors of [`LogoIndex::decode_dial`], and [`LogoError::Image`] when the image store
    /// refuses the texture.
    pub fn load_dial(
        &self,
        images: &mut Images,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        brand: BrandId,
    ) -> Result<ImageId, LogoError> {
        let chain = self.decode_dial(brand)?;
        Ok(images.upload_mips(device, queue, &chain, self.key(brand).unwrap_or("brand dial"))?)
    }

    fn by_key(&self, key: &str) -> Option<BrandId> {
        self.brands
            .binary_search_by(|b| b.key.as_str().cmp(key))
            .ok()
            .or_else(|| self.brands.iter().position(|b| b.key.eq_ignore_ascii_case(key)))
            .map(BrandId)
    }

    fn by_alias(&self, spelling: &str) -> Option<BrandId> {
        match self.aliases.binary_search_by(|(s, _)| s.as_str().cmp(spelling)) {
            Ok(i) => self.aliases.get(i).map(|&(_, id)| id),
            Err(_) => self
                .aliases
                .iter()
                .find(|(s, _)| s.eq_ignore_ascii_case(spelling))
                .map(|&(_, id)| id),
        }
    }

    /// The generic emblem, or an id this index never returned.
    fn is_generic(&self, id: BrandId) -> bool {
        self.brands
            .get(id.0)
            .is_none_or(|b| b.key.eq_ignore_ascii_case(GENERIC_BRAND))
    }
}

/// What a car shows in the logo zone.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CarBrand {
    /// The brand's logo; `None` for unknown and generic cars.
    pub logo: Option<BrandId>,
    /// The name the wordmark falls back to without an image: the brand palette's name for a
    /// branded car ("Porsche"), else the logo index's name for its make, else empty (the custom
    /// name shows).
    pub name: String,
}

/// What the logo zone can show for a brand right now (see [`DialLoader::dial`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dial {
    /// The brand's dial, in the kit's image store.
    Ready(ImageId),
    /// No dial: the car has no brand logo, or it could not be loaded. Show the brand's name.
    Missing,
    /// It is being decoded or uploaded. Show nothing until [`DialLoader::poll`] hands it over.
    Loading,
}

/// A brand's dial in the current kit. A brand not asked for since the kit started has none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    /// Sent to the worker, and not in the image store yet.
    Asked,
    /// Could not be read, decoded or uploaded (warned about once).
    Failed,
    /// In the kit's image store.
    Loaded(ImageId),
}

/// A dial the worker decoded, and how long that took.
#[derive(Debug)]
struct Decoded {
    brand: BrandId,
    result: Result<MipChain, LogoError>,
    ms: f64,
}

/// A decoded dial on its way into the image store, a step or more per frame.
#[derive(Debug)]
struct Uploading {
    brand: BrandId,
    chain: MipChain,
    /// The texture the chain is going into, from the upload's first step to its last. It lives
    /// here so that an upload which ends or is dropped can leave no texture behind.
    texture: Option<MipUpload>,
    decode_ms: f64,
    /// The render thread's time on it so far, the longest of its steps, and the frames it took.
    upload_ms: f64,
    worst_step_ms: f64,
    frames: u32,
}

/// The `fh6-logos` thread: decodes the dials it is sent, in order, and sends each back.
#[derive(Debug)]
struct Worker {
    /// `None` once the thread is gone (or never started).
    jobs: Option<Sender<BrandId>>,
    done: Receiver<Decoded>,
    in_flight: usize,
}

impl Worker {
    /// Starts the thread with its own copy of the index. The handle is not kept: the thread ends
    /// by itself once `jobs` is dropped and its current decode is done.
    fn start(index: LogoIndex) -> Self {
        let (jobs, inbox) = mpsc::channel::<BrandId>();
        let (outbox, done) = mpsc::channel::<Decoded>();
        let spawned = std::thread::Builder::new().name("fh6-logos".into()).spawn(move || {
            for brand in inbox {
                let started = Stamp::now();
                let result = index.decode_dial(brand);
                let ms = started.elapsed().as_secs_f64() * 1000.0;
                if outbox.send(Decoded { brand, result, ms }).is_err() {
                    break;
                }
            }
        });
        let jobs = match spawned {
            Ok(_detached) => Some(jobs),
            Err(err) => {
                tracing::warn!(error = %err, "the brand logo thread did not start; logo zones show the brand name");
                None
            }
        };
        Self {
            jobs,
            done,
            in_flight: 0,
        }
    }

    /// Queues `brand`; false when the thread is not running.
    fn request(&mut self, brand: BrandId) -> bool {
        let sent = self.jobs.as_ref().is_some_and(|jobs| jobs.send(brand).is_ok());
        if sent {
            self.in_flight += 1;
        } else {
            self.jobs = None;
        }
        sent
    }

    /// The next finished decode, without blocking.
    fn try_next(&mut self) -> Result<Decoded, TryRecvError> {
        let next = self.done.try_recv();
        if next.is_ok() {
            self.in_flight = self.in_flight.saturating_sub(1);
        }
        next
    }
}

/// How long one [`DialLoader::poll`] keeps uploading before it leaves the rest to the next
/// frame, in milliseconds. A poll always takes one step, so a frame spends at most this plus one
/// step on a dial (the app's whole frame has 1 ms). The step that finishes a dial ends the poll
/// too: that frame also writes the dial's log line and frees its texels, and starts no other
/// dial. On the PC's DX12 card a step costs about 0.1 ms (0.14 ms at worst), which is more than
/// this, so there a frame takes exactly one step.
pub const STEP_BUDGET_MS: f64 = 0.05;

/// Each brand's dial in one kit, loaded the first time a car of that brand appears and kept for
/// the kit's life. The file is read and decoded on the `fh6-logos` worker thread
/// ([`LogoIndex::decode_dial`]). The render thread only uploads the decoded levels, at most
/// 1.33 MiB, and spreads that over the frames that follow: each [`DialLoader::poll`] creates the
/// texture or copies slices of at most [`images::UPLOAD_STEP_BYTES`] until [`STEP_BUDGET_MS`]
/// has gone, one dial at a time. So a car change never stalls a frame: the logo zone stays empty
/// while the dial decodes (10 to 20 ms on the PC) and uploads (11 to 19 steps for the pack's
/// dials; at the PC's one step a frame that is 0.09 to 0.16 s at 120 frames a second, and twice
/// that at 60), then shows it. A brand whose dial cannot be loaded shows its name, and is never
/// read again on that kit.
///
/// Where threads cannot start (`wasm32`), every brand shows its name.
#[derive(Debug)]
pub struct DialLoader {
    index: LogoIndex,
    /// The brands asked for on this kit (at most one entry each).
    slots: Vec<(BrandId, Slot)>,
    worker: Worker,
    /// The brand the host is waiting for: the last [`DialLoader::dial`] that returned
    /// [`Dial::Loading`].
    waiting: Option<BrandId>,
    /// The decoded dial being uploaded, with the texture it is going into.
    uploading: Option<Uploading>,
}

impl DialLoader {
    /// Starts the worker thread with its own copy of `index`.
    #[must_use]
    pub fn start(index: &LogoIndex) -> Self {
        Self::with_worker(index, Worker::start(index.clone()))
    }

    fn with_worker(index: &LogoIndex, worker: Worker) -> Self {
        Self {
            index: index.clone(),
            slots: Vec::with_capacity(index.len()),
            worker,
            waiting: None,
            uploading: None,
        }
    }

    /// A new kit (after a device loss) has none of the dials: forget them all, and the one
    /// being uploaded. A decode that is still running lands on the new kit if its brand is asked
    /// for again there, and is dropped if not.
    ///
    /// Call it as soon as the old device is lost, too: the dial being uploaded holds a texture
    /// of that device, and a texture keeps its device alive.
    pub fn new_kit(&mut self) {
        self.slots.clear();
        self.waiting = None;
        self.uploading = None;
    }

    /// The dial for a car of `brand` ([`LogoIndex::car_brand`]): ready, missing, or loading. The
    /// first time a brand appears on this kit, its decode is queued and this says
    /// [`Dial::Loading`]; [`DialLoader::poll`] then reports when it is ready. Call it when the
    /// car changes, never per frame.
    pub fn dial(&mut self, brand: Option<BrandId>) -> Dial {
        self.waiting = None;
        let Some(brand) = brand else {
            return Dial::Missing;
        };
        match slot_of(&self.slots, brand) {
            Some(Slot::Loaded(image)) => Dial::Ready(image),
            Some(Slot::Failed) => Dial::Missing,
            Some(Slot::Asked) => {
                self.waiting = Some(brand);
                Dial::Loading
            }
            None if self.worker.request(brand) => {
                set_slot(&mut self.slots, brand, Slot::Asked);
                self.waiting = Some(brand);
                Dial::Loading
            }
            None => {
                tracing::warn!(
                    brand = self.index.name(brand).unwrap_or_default(),
                    "the brand logo thread is not running; the logo zone shows the brand name"
                );
                set_slot(&mut self.slots, brand, Slot::Failed);
                Dial::Missing
            }
        }
    }

    /// Whether a dial is being decoded or uploaded. A host calls [`DialLoader::poll`] only then,
    /// so a frame without one costs a single comparison.
    #[must_use]
    pub fn busy(&self) -> bool {
        self.worker.in_flight > 0 || self.uploading.is_some()
    }

    /// One frame's share of the uploads, on this thread and without blocking: it takes the next
    /// dial the worker has decoded and copies it into `images` a step at a time until
    /// [`STEP_BUDGET_MS`] has gone or the dial is in, and leaves the rest to the next call. Returns what the brand
    /// the host is waiting for now shows, once that is settled: its dial, or [`Dial::Missing`]
    /// when it failed. `None` while that brand is not done, and when none is awaited.
    pub fn poll(&mut self, images: &mut Images, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Dial> {
        let started = Stamp::now();
        self.poll_timed(
            STEP_BUDGET_MS,
            || started.elapsed().as_secs_f64() * 1000.0,
            |chain, texture, _, label| images.upload_step(device, queue, texture, chain, label),
        )
    }

    /// [`DialLoader::poll`] with its clock and its upload step as closures (the unit tests fake
    /// both). `since_start_ms` is the time the poll has run. The step is given the chain, the
    /// texture it is filling, the chain's brand and the texture's label, and returns the image
    /// once its last slice is in. `budget_ms` is [`STEP_BUDGET_MS`]: at 0 every call takes one
    /// step.
    fn poll_timed(
        &mut self,
        budget_ms: f64,
        mut since_start_ms: impl FnMut() -> f64,
        mut step: impl FnMut(&MipChain, &mut Option<MipUpload>, BrandId, &str) -> Result<Option<ImageId>, ImageError>,
    ) -> Option<Dial> {
        let mut awaited = None;
        // An upload counts each frame it takes a step in, once.
        let mut new_frame = true;
        loop {
            if self.uploading.is_none() {
                if !self.next_decoded(&mut awaited) {
                    break;
                }
                new_frame = true;
            }
            let Some(up) = self.uploading.as_mut() else {
                break;
            };
            let before = since_start_ms();
            let label = self.index.key(up.brand).unwrap_or("brand dial");
            let stepped = step(&up.chain, &mut up.texture, up.brand, label);
            let now = since_start_ms();
            up.upload_ms += now - before;
            up.worst_step_ms = up.worst_step_ms.max(now - before);
            up.frames += u32::from(new_frame);
            new_frame = false;
            let name = self.index.name(up.brand).unwrap_or_default();
            let slot = match stepped {
                Ok(None) => None,
                Ok(Some(image)) => {
                    tracing::info!(
                        brand = name,
                        decode_ms = up.decode_ms,
                        upload_ms = up.upload_ms,
                        worst_step_ms = up.worst_step_ms,
                        frames = up.frames,
                        kib = up.chain.byte_len() / 1024,
                        "brand logo ready"
                    );
                    Some(Slot::Loaded(image))
                }
                Err(err) => {
                    tracing::warn!(brand = name, error = %err, "brand logo not loaded; the logo zone shows the brand name");
                    Some(Slot::Failed)
                }
            };
            if let Some(slot) = slot {
                let brand = up.brand;
                self.uploading = None;
                set_slot(&mut self.slots, brand, slot);
                if self.waiting == Some(brand) {
                    self.waiting = None;
                    awaited = Some(match slot {
                        Slot::Loaded(image) => Dial::Ready(image),
                        Slot::Failed | Slot::Asked => Dial::Missing,
                    });
                }
                // This frame has also logged the dial and freed its texels (after `now` was
                // read): it starts no other dial, whatever time it had left.
                break;
            }
            if now >= budget_ms {
                break;
            }
        }
        awaited
    }

    /// Takes finished decodes from the worker until one is to be uploaded (it becomes
    /// `uploading`: true) or there are no more (false). A decode that failed fails its brand,
    /// and `awaited` says so when the host is waiting for it. One whose brand is not waiting for
    /// a dial on this kit is dropped: it was sent before a device loss, or sent twice.
    fn next_decoded(&mut self, awaited: &mut Option<Dial>) -> bool {
        loop {
            let done = match self.worker.try_next() {
                Ok(done) => done,
                Err(TryRecvError::Empty) => return false,
                Err(TryRecvError::Disconnected) => {
                    // The thread died (a panic in the decoder): nothing more will arrive.
                    tracing::error!("the brand logo thread stopped; logo zones show the brand name");
                    self.worker.jobs = None;
                    self.worker.in_flight = 0;
                    for (_, slot) in self.slots.iter_mut().filter(|(_, slot)| *slot == Slot::Asked) {
                        *slot = Slot::Failed;
                    }
                    if self.waiting.take().is_some() {
                        *awaited = Some(Dial::Missing);
                    }
                    return false;
                }
            };
            let brand = done.brand;
            if slot_of(&self.slots, brand) != Some(Slot::Asked) {
                continue;
            }
            match done.result {
                Ok(chain) => {
                    self.uploading = Some(Uploading {
                        brand,
                        chain,
                        texture: None,
                        decode_ms: done.ms,
                        upload_ms: 0.0,
                        worst_step_ms: 0.0,
                        frames: 0,
                    });
                    return true;
                }
                Err(err) => {
                    tracing::warn!(
                        brand = self.index.name(brand).unwrap_or_default(),
                        error = %err,
                        "brand logo not loaded; the logo zone shows the brand name"
                    );
                    set_slot(&mut self.slots, brand, Slot::Failed);
                    if self.waiting == Some(brand) {
                        self.waiting = None;
                        *awaited = Some(Dial::Missing);
                    }
                }
            }
        }
    }
}

/// `brand`'s slot, if it was asked for on this kit.
fn slot_of(slots: &[(BrandId, Slot)], brand: BrandId) -> Option<Slot> {
    slots.iter().find(|(b, _)| *b == brand).map(|&(_, slot)| slot)
}

/// Sets `brand`'s slot (one entry per brand).
fn set_slot(slots: &mut Vec<(BrandId, Slot)>, brand: BrandId, slot: Slot) {
    match slots.iter_mut().find(|(b, _)| *b == brand) {
        Some(entry) => entry.1 = slot,
        None => slots.push((brand, slot)),
    }
}

/// How many drawn frames a car waits for its dial before it shows its brand's name in the
/// meantime: 10 s at 60 frames a second, 5 s at 120. A dial takes 0.1 to 0.3 s, so this is for
/// a read that hangs (a file that is only in cloud storage). The dial still replaces the name
/// when it arrives.
pub const NAME_AFTER_FRAMES: u32 = 600;

/// The current car's logo, kept by a host across frames: the [`Logo`] its clusters draw. It owns
/// the [`DialLoader`], so the host holds no image id of its own and makes three calls:
/// [`CarLogo::show`] when the car or a setting changes, [`CarLogo::poll`] on each drawn frame
/// while [`CarLogo::busy`], and [`CarLogo::new_kit`] when the device is lost and when its
/// replacement's kit is made.
#[derive(Debug)]
pub struct CarLogo {
    dials: DialLoader,
    /// What the clusters get. It has the car's names from the moment the car shows, and is
    /// marked [`Logo::loading`] while the dial is on its way, so the zone does not show the
    /// name first; only after [`NAME_AFTER_FRAMES`] of a read that hangs does the name show.
    shown: Logo,
    /// Whether the shown car's dial is still on its way.
    awaiting: bool,
    /// Drawn frames spent waiting for it.
    waited: u32,
}

impl CarLogo {
    /// Starts the dial loader ([`DialLoader::start`]), with nothing to show yet.
    #[must_use]
    pub fn start(index: &LogoIndex) -> Self {
        Self::with_loader(DialLoader::start(index))
    }

    fn with_loader(dials: DialLoader) -> Self {
        Self {
            dials,
            shown: Logo::default(),
            awaiting: false,
            waited: 0,
        }
    }

    /// What the logo zone shows now.
    #[must_use]
    pub fn logo(&self) -> &Logo {
        &self.shown
    }

    /// Chooses what the car shows: its brand's dial ([`LogoIndex::car_brand`]), or the names
    /// when it has none. `None` is a logo zone that is switched off: nothing shows and nothing
    /// loads. Call it when the car or a setting changes, and after [`CarLogo::new_kit`]; never
    /// per frame. A car of the brand that is already awaited keeps the frames it has waited, so
    /// a setting changed while a read hangs neither restarts the wait nor takes the name away.
    pub fn show(&mut self, brand: Option<CarBrand>, custom_name: &str) {
        let awaited = if self.awaiting { self.dials.waiting } else { None };
        self.awaiting = false;
        let Some(brand) = brand else {
            self.waited = 0;
            self.dials.waiting = None;
            self.shown = Logo::default();
            return;
        };
        let dial = self.dials.dial(brand.logo);
        if dial != Dial::Loading || awaited != brand.logo {
            self.waited = 0;
        }
        self.awaiting = dial == Dial::Loading;
        self.shown = Logo {
            image: match dial {
                Dial::Ready(image) => Some(image),
                Dial::Missing | Dial::Loading => None,
            },
            brand_name: brand.name,
            custom_name: custom_name.to_string(),
            loading: self.awaiting && self.waited < NAME_AFTER_FRAMES,
        };
    }

    /// Whether a dial is on its way ([`DialLoader::busy`]). A host calls [`CarLogo::poll`] only
    /// then, so a frame without one costs a single comparison.
    #[must_use]
    pub fn busy(&self) -> bool {
        self.dials.busy()
    }

    /// One frame's share of loading ([`DialLoader::poll`]). The car's dial shows once it is in.
    pub fn poll(&mut self, images: &mut Images, device: &wgpu::Device, queue: &wgpu::Queue) {
        let dial = self.dials.poll(images, device, queue);
        self.landed(dial);
    }

    /// A new kit (after a device loss) has none of the images: nothing shows until the host
    /// calls [`CarLogo::show`] again. A host calls this twice: when the device is lost, because a
    /// dial that is half uploaded holds a texture of that device and would keep it alive
    /// ([`DialLoader::new_kit`]), and again when the new kit is made.
    pub fn new_kit(&mut self) {
        self.dials.new_kit();
        self.shown = Logo::default();
        self.awaiting = false;
        self.waited = 0;
    }

    /// What one poll found for the car that is waiting: its dial or its failure, or nothing yet.
    fn landed(&mut self, dial: Option<Dial>) {
        if !self.awaiting {
            return;
        }
        match dial {
            Some(dial) => {
                self.awaiting = false;
                self.shown.loading = false;
                if let Dial::Ready(image) = dial {
                    self.shown.image = Some(image);
                }
            }
            None => {
                self.waited = self.waited.saturating_add(1);
                if self.waited == NAME_AFTER_FRAMES {
                    self.shown.loading = false;
                }
            }
        }
    }
}

/// `dial` as a file path that stays inside the folder it is relative to, or `None`.
fn plain_relative(dial: &str) -> Option<PathBuf> {
    let path = Path::new(dial.trim());
    let inside = path
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    let names_a_file = matches!(path.components().next_back(), Some(Component::Normal(_)));
    (inside && names_a_file).then(|| path.to_path_buf())
}

/// Draws the logo zone: the image, else the wordmark ([`wordmark`]), and nothing while the
/// image is on its way ([`Logo::loading`]). Returns the rectangle actually used.
pub fn draw(p: &mut Painter<'_>, zone: Rect, logo: &Logo, style: &LogoStyle) -> Rect {
    if logo.loading {
        // The image is on its way: nothing yet, not even the name.
        return Rect::from_center_size(zone.center(), Vec2::ZERO);
    }
    if let Some(image) = logo.image {
        // The picture's size, not the texture's: a dial uploaded from 512 px down keeps its shape.
        if let Some(size) = p.images().source_size(image) {
            let r = fit(zone, size);
            p.fill(&Shape::rect(r), ImagePaint::new(image, r).tint(style.tint));
            return r;
        }
    }
    let mut buf = [0u8; 64];
    let text = wordmark(logo, &mut buf);
    if text.is_empty() {
        return Rect::from_center_size(zone.center(), Vec2::ZERO);
    }
    let mut ts = TextStyle::new(style.font, style.cap)
        .paint(style.text)
        .align(Align::Center, VAlign::CapMiddle)
        .tracking(style.tracking);
    // Shrink to fit the zone's inner width (labels shrink to 85% then elide; wordmarks just fit).
    let inner = zone.inset(zone.width().min(zone.height()) * SAFE_PADDING);
    let w = p.measure(text, &ts).width;
    if w > inner.width() && w > 0.0 {
        ts.cap *= inner.width() / w;
    }
    p.text(zone.center(), text, &ts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::time::{Duration, Instant};

    fn repo_assets() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
    }

    fn repo_index() -> LogoIndex {
        LogoIndex::load(&repo_assets()).expect("assets/logos/logos.json loads")
    }

    fn repo_cars() -> CarDatabase {
        CarDatabase::load(repo_assets().join("car_brands/car_brands.json")).expect("car_brands.json loads")
    }

    /// A small index for the lookup rules.
    fn toy_index() -> LogoIndex {
        let json = r#"{
            "logos": {
                "DEFAULT": { "name": "Generic", "dial": "default_dial.png" },
                "NISSAN": { "name": "Nissan", "dial": "nissan_dial.png" },
                "NISSAN_NISMO": { "name": "Nismo", "dial": "nismo_dial.png" },
                "BOXCO": { "name": "Boxco", "dial": "boxco_dial.png" },
                "ESCAPE": { "name": "Escape", "dial": "../outside.png" },
                "ROOTED": { "name": "Rooted", "dial": "/abs/rooted.png" },
                "NESTED": { "name": "Nested", "dial": "./sub/nested_dial.png" }
            },
            "aliases": {
                "Nissan": "NISSAN_NISMO",
                "NISSAN": "NISSAN",
                "Box": "DEFAULT",
                "BOX": "BOXCO",
                "Generic": "DEFAULT",
                "Ghost": "NOT_LISTED"
            }
        }"#;
        LogoIndex::from_json(json, Path::new("logos")).expect("toy index parses")
    }

    /// `car_brand` against a small car list, beside the shipped one: keys as the car list may
    /// write them, a key the index lacks with a make it knows, the generic key, and cars or
    /// lists that are not there.
    #[test]
    fn a_car_brand_follows_its_key_then_its_make_then_names_nothing() {
        let cars = CarDatabase::from_json_str(
            r##"{
                "BrandPalettes": {
                    "NISSAN": {"DisplayName": "Nissan Motor", "PrimaryHex": "#C3002F"},
                    "DEFAULT": {"DisplayName": "Standard Horizon", "PrimaryHex": "#00D2FF"}
                },
                "Cars": {
                    " 1 ": {"BrandKey": " nissan ", "Manufacturer": "Nissan"},
                    "2": {"BrandKey": "BOX_TRUCKS", "Manufacturer": "BOX"},
                    "3": {"BrandKey": "DEFAULT", "Manufacturer": "Nissan"},
                    "4": {"BrandKey": "DEFAULT", "Manufacturer": "Generic"}
                }
            }"##,
        )
        .expect("the small car list parses");
        let ix = toy_index();
        let shows = |ordinal| {
            let brand = ix.car_brand(&cars, ordinal);
            (key_of(&ix, brand.logo).map(str::to_string), brand.name)
        };
        let some = |key: &str, name: &str| (Some(key.to_string()), name.to_string());
        // A lower-case, padded key under a padded ordinal: the key's logo, the palette's name.
        assert_eq!(shows(1), some("NISSAN", "Nissan Motor"));
        // A key the index lacks: the make's alias, and the index's name (no palette has it).
        assert_eq!(shows(2), some("BOXCO", "Boxco"));
        // The generic key: the make's alias, and not the generic palette's theme name.
        assert_eq!(shows(3), some("NISSAN_NISMO", "Nismo"));
        // A make that is an alias of the generic emblem: no logo and no name.
        assert_eq!(shows(4), (None, String::new()));
        // A car the list lacks, and a list that did not load.
        assert_eq!(ix.car_brand(&cars, 999_999), CarBrand::default());
        assert_eq!(ix.car_brand(&CarDatabase::default(), 1), CarBrand::default());
        // An index that did not load: no logo, and the palette still names the brand.
        let none = LogoIndex::default().car_brand(&cars, 1);
        assert_eq!((none.logo, none.name.as_str()), (None, "Nissan Motor"));
    }

    fn key_of(index: &LogoIndex, id: Option<BrandId>) -> Option<&str> {
        id.and_then(|id| index.key(id))
    }

    #[test]
    fn lookups_follow_the_documented_order() {
        let ix = toy_index();
        assert_eq!(ix.len(), 7);
        // The key wins over the make.
        assert_eq!(key_of(&ix, ix.resolve("NISSAN", "Nissan")), Some("NISSAN"));
        // An exact alias beats a case-folded one; case folding takes the first in byte order.
        assert_eq!(key_of(&ix, ix.resolve("", "Nissan")), Some("NISSAN_NISMO"));
        assert_eq!(key_of(&ix, ix.resolve("", "NISSAN")), Some("NISSAN"));
        assert_eq!(key_of(&ix, ix.resolve("", "nissan")), Some("NISSAN"));
        // Keys match whatever their case, and spaces around either input are ignored.
        assert_eq!(key_of(&ix, ix.resolve(" nissan_nismo ", "")), Some("NISSAN_NISMO"));
        // A generic key falls through to the make.
        assert_eq!(key_of(&ix, ix.resolve("DEFAULT", "NISSAN")), Some("NISSAN"));
        assert_eq!(key_of(&ix, ix.resolve("Generic", "BOX")), Some("BOXCO"));
        // An unknown key falls through to the make too.
        assert_eq!(key_of(&ix, ix.resolve("NOT_A_BRAND", "Nissan")), Some("NISSAN_NISMO"));
    }

    #[test]
    fn the_generic_emblem_is_never_a_brand() {
        let ix = toy_index();
        assert_eq!(ix.resolve("DEFAULT", ""), None);
        assert_eq!(ix.resolve("default", "Generic"), None);
        // An exact alias to the generic emblem wins over a case-folded alias to a real brand.
        assert_eq!(ix.resolve("", "Box"), None);
        assert_eq!(ix.resolve("", ""), None);
        assert_eq!(ix.resolve("", "Unheard Of"), None);
        // An alias to a brand the index does not list was left out.
        assert_eq!(ix.resolve("", "Ghost"), None);
    }

    #[test]
    fn dial_paths_stay_inside_the_logo_folder() {
        let ix = toy_index();
        let id = |key: &str| ix.resolve(key, "").expect("listed brand");
        assert_eq!(
            ix.dial_path(id("NISSAN")),
            Some(Path::new("logos").join("nissan_dial.png"))
        );
        assert_eq!(
            ix.dial_path(id("NESTED")),
            Some(Path::new("logos").join("./sub/nested_dial.png"))
        );
        // Paths that leave the folder are dropped, but the brand keeps resolving (its name still
        // shows as the wordmark).
        assert_eq!(ix.dial_path(id("ESCAPE")), None);
        assert_eq!(ix.dial_path(id("ROOTED")), None);
        assert_eq!(ix.name(id("ESCAPE")), Some("Escape"));
        for bad in ["", "  ", ".", "./", "..", "a/../../b.png", "/x.png"] {
            assert_eq!(plain_relative(bad), None, "{bad:?}");
        }
        assert_eq!(plain_relative(" a.png "), Some(PathBuf::from("a.png")));
        // An id from another index names nothing here.
        assert_eq!(ix.dial_path(BrandId(usize::MAX)), None);
        assert_eq!(ix.key(BrandId(usize::MAX)), None);
    }

    #[test]
    fn a_broken_index_is_an_error() {
        assert!(matches!(
            LogoIndex::from_json("{ not json", Path::new("logos")),
            Err(LogoError::Json(_))
        ));
        assert!(matches!(
            LogoIndex::load(Path::new("definitely/not/an/assets/folder")),
            Err(LogoError::Io { .. })
        ));
        // A BOM and missing sections are fine: an empty index.
        let empty = LogoIndex::from_json("\u{feff}{}", Path::new("logos")).expect("empty index");
        assert!(empty.is_empty());
        assert_eq!(empty.resolve("PORSCHE", "Porsche"), None);
        assert_eq!(empty.car_brand(&repo_cars(), 3667).logo, None);
    }

    #[test]
    fn every_car_in_the_database_resolves() {
        let ix = repo_index();
        let cars = repo_cars();
        assert!(!cars.is_empty(), "car_brands.json loads");
        let mut generic = Vec::new();
        for (ordinal, car) in cars.cars() {
            match ix.resolve(&car.brand_key, &car.manufacturer) {
                // A branded car shows its own brand, never another.
                Some(id) if !car.brand_key.eq_ignore_ascii_case(GENERIC_BRAND) => {
                    assert_eq!(ix.key(id), Some(car.brand_key.as_str()), "car {ordinal}");
                }
                Some(_) => {}
                None => {
                    assert!(
                        car.brand_key.eq_ignore_ascii_case(GENERIC_BRAND),
                        "car {ordinal} has brand {:?} but no logo",
                        car.brand_key
                    );
                    generic.push(car.manufacturer.as_str());
                }
            }
        }
        // Only the makes the index itself calls generic (traffic and placeholder cars) show none.
        for make in &generic {
            let alias = ix.aliases.iter().find(|(s, _)| s == make).map(|&(_, id)| id);
            assert!(
                alias.is_some_and(|id| ix.is_generic(id)),
                "make {make:?} has no logo and no generic alias"
            );
        }
        assert!(generic.len() < 10, "{} cars without a logo: {generic:?}", generic.len());
    }

    #[test]
    fn a_car_shows_its_brand_and_name() {
        let ix = repo_index();
        let cars = repo_cars();
        // 2021 Porsche 911 GT3: by its key, named as its palette names it.
        let gt3 = ix.car_brand(&cars, 3667);
        assert_eq!(key_of(&ix, gt3.logo), Some("PORSCHE"));
        assert_eq!(gt3.name, "Porsche");
        // A car whose key is generic: by its make, named as the logo index names it.
        let (renault_ordinal, renault) = cars
            .cars()
            .find(|(_, c)| c.brand_key == GENERIC_BRAND && c.manufacturer == "Renault")
            .expect("a generic-keyed Renault in car_brands.json");
        let shown = ix.car_brand(&cars, renault_ordinal);
        assert_eq!(key_of(&ix, shown.logo), Some("RENAULT"), "{}", renault.model_name);
        assert_eq!(shown.name, "Renault");
        // A traffic car: no logo, and no "Standard Horizon" wordmark either.
        let bus = ix.car_brand(&cars, 2714);
        assert_eq!(bus, CarBrand::default());
        // No car, or one the database does not list.
        assert_eq!(ix.car_brand(&cars, 0), CarBrand::default());
        assert_eq!(ix.car_brand(&cars, -1), CarBrand::default());
    }

    #[test]
    fn every_dial_decodes_into_the_capped_chain_and_fits_video_memory() {
        let ix = repo_index();
        assert!(ix.len() > 50, "the Phase 6 set has 91 brands");
        let mut bytes = 0usize;
        for i in 0..ix.len() {
            let id = BrandId(i);
            let chain = ix.decode_dial(id).unwrap_or_else(|e| panic!("{:?}: {e}", ix.key(id)));
            let (w, h) = chain.size();
            assert!(w.max(h) <= DIAL_TEXTURE_SIDE, "{:?} uploads {w}x{h}", ix.key(id));
            assert_eq!(
                chain.len() as u32,
                w.max(h).ilog2() + 1,
                "{:?}: every level down to 1 x 1",
                ix.key(id)
            );
            bytes += chain.byte_len();
        }
        // 47.4 MiB for the 91 dials of the Phase 6 set; their 1024 px levels would add 142 MiB.
        assert!(bytes < 64 << 20, "every dial at once is {} MiB", bytes >> 20);
    }

    /// A folder of its own under the system temp folder, removed when dropped.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("fh6_logo_{name}_{}", std::process::id()));
            std::fs::create_dir_all(&dir).expect("temp folder");
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A white RGBA PNG.
    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut bytes, width, height);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            let mut w = enc.write_header().expect("header");
            w.write_image_data(&vec![255; (width * height * 4) as usize])
                .expect("data");
        }
        bytes
    }

    fn write_png(path: &Path, width: u32, height: u32) {
        std::fs::write(path, png_bytes(width, height)).expect("write the PNG");
    }

    #[test]
    fn an_oversized_dial_is_refused_from_its_header() {
        let tmp = TempDir::new("oversized");
        let over = MAX_DIAL_SIDE + 1;
        write_png(&tmp.0.join("wide_dial.png"), over, 1);
        write_png(&tmp.0.join("tall_dial.png"), 1, over);
        // Too big by its header, and cut off 7 bytes into its image data: it cannot be decoded,
        // so only a check that reads the header alone can say it is too large.
        let mut cut = png_bytes(over, over);
        cut.truncate(48);
        std::fs::write(tmp.0.join("cut_dial.png"), &cut).expect("write the cut PNG");
        assert!(images::decode_png(&cut).is_err(), "the cut file does not decode");
        write_png(&tmp.0.join("ok_dial.png"), MAX_DIAL_SIDE, 2);
        let ix = LogoIndex::from_json(
            r#"{"logos":{
                "WIDE":{"name":"Wide","dial":"wide_dial.png"},
                "TALL":{"name":"Tall","dial":"tall_dial.png"},
                "CUT":{"name":"Cut","dial":"cut_dial.png"},
                "OK":{"name":"Ok","dial":"ok_dial.png"}}}"#,
            &tmp.0,
        )
        .expect("index");
        for (key, size) in [("WIDE", (over, 1)), ("TALL", (1, over)), ("CUT", (over, over))] {
            match ix.decode_dial(ix.by_key(key).expect(key)) {
                Err(LogoError::TooLarge { width, height, .. }) => assert_eq!((width, height), size, "{key}"),
                other => panic!("{key}: expected TooLarge, got {other:?}"),
            }
        }
        // At the limit it decodes, and only the levels from 512 px down are kept.
        let chain = ix
            .decode_dial(ix.by_key("OK").expect("OK"))
            .expect("a 1024 px dial decodes");
        assert_eq!(chain.size(), (DIAL_TEXTURE_SIDE, 1));
        assert_eq!(chain.len(), 10, "512, 256, ... 1");
    }

    #[test]
    fn a_broken_dial_is_an_image_error_and_its_car_shows_the_name() {
        let tmp = TempDir::new("broken");
        // A 64 x 64 PNG cut off just after its header: small enough to pass the size check, and
        // impossible to decode.
        let mut cut = png_bytes(64, 64);
        cut.truncate(40);
        std::fs::write(tmp.0.join("broken_dial.png"), &cut).expect("write the cut PNG");
        let ix = LogoIndex::from_json(
            r#"{"logos":{"BROKEN":{"name":"Broken","dial":"broken_dial.png"}}}"#,
            &tmp.0,
        )
        .expect("index");
        let broken = ix.by_key("BROKEN").expect("BROKEN");
        assert!(
            matches!(ix.decode_dial(broken), Err(LogoError::Image(_))),
            "a decode error, not an I/O or size error"
        );
        // Through the worker, the loader fails the brand and uploads nothing.
        let mut loader = DialLoader::start(&ix);
        assert_eq!(loader.dial(Some(broken)), Dial::Loading);
        let mut uploads = Vec::new();
        let shown = settle(&mut loader, &mut uploads, |b| Ok(image_of(b)));
        assert_eq!(shown, Some(Dial::Missing));
        assert!(uploads.is_empty(), "nothing to upload");
        assert_eq!(loader.dial(Some(broken)), Dial::Missing, "and it is never read again");
    }

    #[test]
    fn a_missing_dial_file_is_an_io_error() {
        let tmp = TempDir::new("missing");
        let ix = LogoIndex::from_json(r#"{"logos":{"GONE":{"name":"Gone","dial":"gone_dial.png"}}}"#, &tmp.0)
            .expect("index");
        let gone = ix.by_key("GONE").expect("GONE");
        assert!(matches!(ix.decode_dial(gone), Err(LogoError::Io { .. })));
    }

    /// Runs frames with a fake upload that is done in one step (recording each brand it
    /// uploads) until the loader is idle; returns what the awaited brand ended up showing, if it
    /// arrived.
    fn settle(
        loader: &mut DialLoader,
        uploads: &mut Vec<BrandId>,
        result: impl Fn(BrandId) -> Result<ImageId, ImageError>,
    ) -> Option<Dial> {
        let until = Instant::now() + Duration::from_secs(30);
        let mut awaited = None;
        while loader.busy() && Instant::now() < until {
            let found = loader.poll_with(0.0, |_, brand, _| {
                uploads.push(brand);
                result(brand).map(Some)
            });
            awaited = awaited.or(found);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(!loader.busy(), "the worker finished within 30 s");
        awaited
    }

    impl DialLoader {
        /// One frame's poll with a fake upload step, on a clock that stands still: with a budget
        /// of 0 it takes one step, and with no limit every step there is.
        fn poll_with(
            &mut self,
            budget_ms: f64,
            mut step: impl FnMut(&MipChain, BrandId, &str) -> Result<Option<ImageId>, ImageError>,
        ) -> Option<Dial> {
            self.poll_timed(budget_ms, || 0.0, |chain, _, brand, label| step(chain, brand, label))
        }
    }

    /// The image id the fakes give a brand's dial.
    fn image_of(brand: BrandId) -> ImageId {
        ImageId(brand.0 as u32)
    }

    /// A fake upload that takes `steps` calls for each dial, as the real one takes a call for
    /// the texture and one for each slice. It records every call.
    struct SteppedUpload {
        steps: usize,
        calls: Vec<BrandId>,
    }

    impl SteppedUpload {
        fn new(steps: usize) -> Self {
            Self {
                steps,
                calls: Vec::new(),
            }
        }

        fn step(&mut self, brand: BrandId) -> Result<Option<ImageId>, ImageError> {
            self.calls.push(brand);
            let taken = self.calls.iter().filter(|b| **b == brand).count();
            Ok((taken % self.steps == 0).then(|| image_of(brand)))
        }
    }

    /// Runs frames of one step each until the fake has been called `calls` times.
    fn frames_until_calls(loader: &mut DialLoader, fake: &mut SteppedUpload, calls: usize) {
        let until = Instant::now() + Duration::from_secs(30);
        while fake.calls.len() < calls && Instant::now() < until {
            let _ = loader.poll_with(0.0, |_, brand, _| fake.step(brand));
            if fake.calls.len() < calls {
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        assert_eq!(fake.calls.len(), calls, "the decode landed within 30 s");
    }

    #[test]
    fn a_brand_is_decoded_once_then_ready_for_every_car_of_it() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut loader = DialLoader::start(&ix);
        let porsche = ix.car_brand(&cars, 3667).logo;
        assert!(porsche.is_some());
        assert_eq!(loader.dial(porsche), Dial::Loading, "the 911 GT3's dial is decoding");
        assert_eq!(
            loader.dial(ix.car_brand(&cars, 3123).logo),
            Dial::Loading,
            "a 992 meanwhile"
        );
        assert_eq!(loader.worker.in_flight, 1, "one decode for the brand");
        let mut uploads = Vec::new();
        let shown = settle(&mut loader, &mut uploads, |_| Ok(ImageId(7)));
        assert_eq!(shown, Some(Dial::Ready(ImageId(7))), "the awaited car gets its dial");
        assert_eq!(uploads, [porsche.unwrap_or(BrandId(usize::MAX))], "uploaded once");
        assert_eq!(loader.dial(porsche), Dial::Ready(ImageId(7)), "ready at once next time");
        assert!(!loader.busy(), "a loaded brand asks for nothing more");
    }

    #[test]
    fn a_car_without_a_brand_logo_shows_its_name() {
        let mut loader = DialLoader::start(&repo_index());
        assert_eq!(loader.dial(None), Dial::Missing);
        assert!(!loader.busy());
    }

    #[test]
    fn a_dial_uploads_a_step_per_frame_and_is_ready_when_the_last_is_in() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut loader = DialLoader::start(&ix);
        let porsche = ix.car_brand(&cars, 3667).logo;
        assert_eq!(loader.dial(porsche), Dial::Loading);
        let mut fake = SteppedUpload::new(5);
        // Frames go by until the decode lands; the frame that finds it takes the first step.
        frames_until_calls(&mut loader, &mut fake, 1);
        for frame in 2..5 {
            assert!(loader.busy(), "still uploading before frame {frame}");
            assert_eq!(loader.dial(porsche), Dial::Loading, "nothing to draw yet");
            assert_eq!(loader.poll_with(0.0, |_, brand, _| fake.step(brand)), None);
            assert_eq!(fake.calls.len(), frame, "one step in a frame with no time left");
        }
        let image = porsche.map(image_of).expect("the GT3 has a brand logo");
        assert_eq!(
            loader.poll_with(0.0, |_, brand, _| fake.step(brand)),
            Some(Dial::Ready(image)),
            "the frame of the last step hands it over"
        );
        assert_eq!(fake.calls.len(), 5);
        assert!(!loader.busy());
        assert_eq!(loader.dial(porsche), Dial::Ready(image));
    }

    #[test]
    fn a_frame_with_time_left_takes_more_steps() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut loader = DialLoader::start(&ix);
        let porsche = ix.car_brand(&cars, 3667).logo;
        assert_eq!(loader.dial(porsche), Dial::Loading);
        let mut fake = SteppedUpload::new(5);
        let until = Instant::now() + Duration::from_secs(30);
        let mut shown = None;
        while fake.calls.is_empty() && Instant::now() < until {
            shown = loader.poll_with(f64::INFINITY, |_, brand, _| fake.step(brand));
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(fake.calls.len(), 5, "every step in the frame the decode landed in");
        assert_eq!(shown, porsche.map(|brand| Dial::Ready(image_of(brand))));
        assert!(!loader.busy());
    }

    #[test]
    fn dials_upload_one_at_a_time_and_a_car_is_only_handed_its_own() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut loader = DialLoader::start(&ix);
        // A Porsche, a BMW M and a Ferrari, one after the other before any dial is in.
        let brands = [3667, 4122, 3595].map(|ordinal| ix.car_brand(&cars, ordinal).logo.expect("a branded car"));
        assert!(brands[0] != brands[1] && brands[1] != brands[2] && brands[0] != brands[2]);
        for brand in brands {
            assert_eq!(loader.dial(Some(brand)), Dial::Loading);
        }
        let mut fake = SteppedUpload::new(3);
        let mut handed = Vec::new();
        let until = Instant::now() + Duration::from_secs(30);
        while loader.busy() && Instant::now() < until {
            let before = fake.calls.len();
            if let Some(dial) = loader.poll_with(0.0, |_, brand, _| fake.step(brand)) {
                handed.push((fake.calls.len(), dial));
            }
            assert!(
                fake.calls.len() <= before + 1,
                "at most one step in a frame with no time left"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(!loader.busy(), "all three within 30 s");
        let in_turn: Vec<BrandId> = brands.iter().flat_map(|&brand| [brand; 3]).collect();
        assert_eq!(fake.calls, in_turn, "each dial is finished before the next is begun");
        assert_eq!(
            handed,
            [(9, Dial::Ready(image_of(brands[2])))],
            "the car that is waiting (the Ferrari) gets its own dial, and no other car's"
        );
        for brand in brands {
            assert_eq!(loader.dial(Some(brand)), Dial::Ready(image_of(brand)));
        }
    }

    #[test]
    fn dials_decoded_before_the_first_frame_still_upload_one_at_a_time() {
        let (ix, cars) = (repo_index(), repo_cars());
        let (mut loader, inbox, outbox) = held_loader(&ix);
        let brands = [3667, 4122, 3595].map(|ordinal| ix.car_brand(&cars, ordinal).logo.expect("a branded car"));
        for brand in brands {
            assert_eq!(loader.dial(Some(brand)), Dial::Loading);
        }
        // All three decodes are in before the next drawn frame, as after a hitch in the game.
        for brand in inbox.try_iter() {
            let decoded = Decoded {
                brand,
                result: ix.decode_dial(brand),
                ms: 0.0,
            };
            outbox.send(decoded).expect("the loader is listening");
        }
        let mut fake = SteppedUpload::new(3);
        for frame in 1..=9 {
            assert!(loader.busy(), "still uploading before frame {frame}");
            let _ = loader.poll_with(0.0, |_, brand, _| fake.step(brand));
            assert_eq!(fake.calls.len(), frame, "one step in a frame with no time left");
        }
        let in_turn: Vec<BrandId> = brands.iter().flat_map(|&brand| [brand; 3]).collect();
        assert_eq!(fake.calls, in_turn, "each dial is finished before the next is begun");
        assert!(!loader.busy());
        for brand in brands {
            assert_eq!(loader.dial(Some(brand)), Dial::Ready(image_of(brand)));
        }
    }

    #[test]
    fn the_budget_is_for_the_whole_frame_not_for_each_step() {
        let (ix, cars) = (repo_index(), repo_cars());
        let (mut loader, inbox, outbox) = held_loader(&ix);
        let porsche = ix.car_brand(&cars, 3667).logo;
        assert_eq!(loader.dial(porsche), Dial::Loading);
        let brand = inbox.try_recv().expect("the decode was asked for");
        let decoded = Decoded {
            brand,
            result: ix.decode_dial(brand),
            ms: 0.0,
        };
        outbox.send(decoded).expect("the loader is listening");
        let mut fake = SteppedUpload::new(40);
        // A clock that only the steps move. They take 1, 3 and 1 ms in turn, and a frame has
        // 5 ms: no step is as long as that, and the third ends on it.
        let clock = Cell::new(0.0);
        let mut frame = |loader: &mut DialLoader| {
            clock.set(0.0);
            let shown = loader.poll_timed(
                5.0,
                || clock.get(),
                |_, _, brand, _| {
                    clock.set(clock.get() + [1.0, 3.0, 1.0][fake.calls.len() % 3]);
                    fake.step(brand)
                },
            );
            (shown, fake.calls.len())
        };
        assert_eq!(frame(&mut loader), (None, 3), "the third step ends the frame's 5 ms");
        assert!(loader.busy(), "the rest waits for the next frame");
        assert_eq!(frame(&mut loader), (None, 6), "and so does the next frame's third");
        // What the "brand logo ready" log line will say: two frames, 10 ms, 3 ms at worst.
        let up = loader.uploading.as_ref().expect("the dial is still uploading");
        assert_eq!(up.frames, 2);
        assert!((up.upload_ms - 10.0).abs() < 1e-9, "{} ms", up.upload_ms);
        assert!((up.worst_step_ms - 3.0).abs() < 1e-9, "{} ms", up.worst_step_ms);
    }

    #[test]
    fn the_frame_that_finishes_a_dial_starts_no_other() {
        let (ix, cars) = (repo_index(), repo_cars());
        let (mut loader, inbox, outbox) = held_loader(&ix);
        let brands = [3667, 4122].map(|ordinal| ix.car_brand(&cars, ordinal).logo.expect("a branded car"));
        for brand in brands {
            assert_eq!(loader.dial(Some(brand)), Dial::Loading);
        }
        for brand in inbox.try_iter() {
            let decoded = Decoded {
                brand,
                result: ix.decode_dial(brand),
                ms: 0.0,
            };
            outbox.send(decoded).expect("the loader is listening");
        }
        let mut fake = SteppedUpload::new(3);
        // Frames with all the time there is: each still ends with the dial it finishes.
        let shown = loader.poll_with(f64::INFINITY, |_, brand, _| fake.step(brand));
        assert_eq!(shown, None, "the Porsche's dial is not the one awaited");
        assert_eq!(fake.calls, [brands[0]; 3], "nothing of the BMW M's dial yet");
        let shown = loader.poll_with(f64::INFINITY, |_, brand, _| fake.step(brand));
        assert_eq!(shown, Some(Dial::Ready(image_of(brands[1]))));
        assert_eq!(fake.calls.len(), 6);
        assert!(!loader.busy());
    }

    #[test]
    fn a_dial_that_fails_shows_the_name_and_is_never_read_again() {
        let tmp = TempDir::new("loader_missing");
        let ix = LogoIndex::from_json(r#"{"logos":{"GONE":{"name":"Gone","dial":"gone_dial.png"}}}"#, &tmp.0)
            .expect("index");
        let gone = ix.by_key("GONE");
        let mut loader = DialLoader::start(&ix);
        assert_eq!(loader.dial(gone), Dial::Loading);
        let mut uploads = Vec::new();
        assert_eq!(
            settle(&mut loader, &mut uploads, |_| Ok(ImageId(1))),
            Some(Dial::Missing)
        );
        assert!(uploads.is_empty(), "nothing to upload");
        assert_eq!(loader.dial(gone), Dial::Missing, "known to fail");
        assert!(!loader.busy(), "and not read again");
    }

    #[test]
    fn a_dial_the_image_store_refuses_counts_as_failed() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut loader = DialLoader::start(&ix);
        let bmw = ix.car_brand(&cars, 4122).logo;
        assert_eq!(loader.dial(bmw), Dial::Loading);
        let mut uploads = Vec::new();
        let refuse = |_| {
            Err(ImageError::Size {
                width: 512,
                height: 512,
                max: 0,
            })
        };
        assert_eq!(settle(&mut loader, &mut uploads, refuse), Some(Dial::Missing));
        assert_eq!(uploads.len(), 1);
        assert_eq!(loader.dial(bmw), Dial::Missing);
        assert!(!loader.busy());
    }

    #[test]
    fn a_new_kit_loads_again_and_an_old_decode_lands_only_once() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut loader = DialLoader::start(&ix);
        let porsche = ix.car_brand(&cars, 3667).logo;
        assert_eq!(loader.dial(porsche), Dial::Loading);
        loader.new_kit();
        assert_eq!(loader.dial(porsche), Dial::Loading, "the new kit has no dials");
        assert_eq!(loader.worker.in_flight, 2, "asked again");
        let mut uploads = Vec::new();
        assert_eq!(
            settle(&mut loader, &mut uploads, |_| Ok(ImageId(3))),
            Some(Dial::Ready(ImageId(3)))
        );
        assert_eq!(uploads.len(), 1, "the second decode of the brand is dropped");
        assert_eq!(loader.dial(porsche), Dial::Ready(ImageId(3)));
    }

    #[test]
    fn a_new_kit_forgets_the_dials_of_the_old_one() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut loader = DialLoader::start(&ix);
        let porsche = ix.car_brand(&cars, 3667).logo;
        assert_eq!(loader.dial(porsche), Dial::Loading);
        let mut uploads = Vec::new();
        assert_eq!(
            settle(&mut loader, &mut uploads, |_| Ok(ImageId(7))),
            Some(Dial::Ready(ImageId(7)))
        );
        loader.new_kit();
        assert_eq!(
            loader.dial(porsche),
            Dial::Loading,
            "image 7 was the old kit's: the brand loads again"
        );
        assert_eq!(
            settle(&mut loader, &mut uploads, |_| Ok(ImageId(8))),
            Some(Dial::Ready(ImageId(8)))
        );
        assert_eq!(loader.dial(porsche), Dial::Ready(ImageId(8)));
        assert_eq!(uploads.len(), 2, "once into each kit");
    }

    #[test]
    fn a_decode_nobody_asked_for_on_the_new_kit_is_dropped() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut loader = DialLoader::start(&ix);
        let porsche = ix.car_brand(&cars, 3667).logo;
        assert_eq!(loader.dial(porsche), Dial::Loading);
        loader.new_kit();
        let mut uploads = Vec::new();
        assert_eq!(settle(&mut loader, &mut uploads, |_| Ok(ImageId(3))), None);
        assert!(
            uploads.is_empty(),
            "no car on this kit showed the brand: nothing is uploaded"
        );
        assert_eq!(loader.dial(porsche), Dial::Loading, "a car that does show it loads it");
        assert_eq!(
            settle(&mut loader, &mut uploads, |_| Ok(ImageId(4))),
            Some(Dial::Ready(ImageId(4)))
        );
        assert_eq!(uploads.len(), 1);
    }

    #[test]
    fn a_new_kit_drops_the_dial_being_uploaded() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut loader = DialLoader::start(&ix);
        let porsche = ix.car_brand(&cars, 3667).logo;
        assert_eq!(loader.dial(porsche), Dial::Loading);
        let mut fake = SteppedUpload::new(4);
        frames_until_calls(&mut loader, &mut fake, 1);
        assert!(loader.busy(), "one step of four is in");
        loader.new_kit();
        assert!(!loader.busy(), "the half-made texture went with the old kit");
        assert_eq!(loader.poll_with(0.0, |_, brand, _| fake.step(brand)), None);
        assert_eq!(fake.calls.len(), 1, "and nothing more is copied into it");
        // On the new kit the brand starts again from its first step.
        let mut new_kit = SteppedUpload::new(4);
        assert_eq!(loader.dial(porsche), Dial::Loading);
        frames_until_calls(&mut loader, &mut new_kit, 3);
        assert!(loader.busy(), "three steps of four");
        frames_until_calls(&mut loader, &mut new_kit, 4);
        assert_eq!(
            loader.dial(porsche),
            Dial::Ready(porsche.map(image_of).expect("a brand"))
        );
        assert!(!loader.busy());
    }

    #[test]
    fn a_dial_that_arrives_after_the_car_changed_is_kept_but_not_handed_over() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut loader = DialLoader::start(&ix);
        let porsche = ix.car_brand(&cars, 3667).logo;
        assert_eq!(loader.dial(porsche), Dial::Loading);
        assert_eq!(loader.dial(None), Dial::Missing, "the next car has no brand logo");
        let mut uploads = Vec::new();
        assert_eq!(
            settle(&mut loader, &mut uploads, |_| Ok(ImageId(5))),
            None,
            "nobody waits for it"
        );
        assert_eq!(loader.dial(porsche), Dial::Ready(ImageId(5)), "but it is in the kit");
    }

    #[test]
    fn a_stopped_worker_fails_what_it_was_asked_and_everything_after() {
        let (ix, cars) = (repo_index(), repo_cars());
        let (jobs, _inbox) = mpsc::channel();
        let (outbox, done) = mpsc::channel::<Decoded>();
        drop(outbox);
        let mut loader = DialLoader::with_worker(
            &ix,
            Worker {
                jobs: Some(jobs),
                done,
                in_flight: 0,
            },
        );
        let porsche = ix.car_brand(&cars, 3667).logo;
        assert_eq!(loader.dial(porsche), Dial::Loading);
        let mut uploads = Vec::new();
        assert_eq!(
            settle(&mut loader, &mut uploads, |_| Ok(ImageId(9))),
            Some(Dial::Missing)
        );
        assert_eq!(loader.dial(porsche), Dial::Missing);
        assert_eq!(
            loader.dial(ix.car_brand(&cars, 4122).logo),
            Dial::Missing,
            "no thread to ask"
        );
        assert!(!loader.busy());
    }

    /// One drawn frame of `logo` while it is busy, with a fake upload that is done in one step.
    fn frame(logo: &mut CarLogo, image: impl Fn(BrandId) -> ImageId) {
        let dial = logo.dials.poll_with(0.0, |_, brand, _| Ok(Some(image(brand))));
        logo.landed(dial);
    }

    /// Frames until nothing is on its way, checking `each` after every one.
    fn frames_until_idle(logo: &mut CarLogo, image: impl Fn(BrandId) -> ImageId, each: impl Fn(&Logo)) {
        let until = Instant::now() + Duration::from_secs(30);
        while logo.busy() && Instant::now() < until {
            frame(logo, &image);
            each(logo.logo());
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(!logo.busy(), "the worker finished within 30 s");
    }

    fn named(brand: &str, custom: &str) -> Logo {
        Logo {
            image: None,
            brand_name: brand.into(),
            custom_name: custom.into(),
            loading: false,
        }
    }

    /// The names of a car whose dial is on its way: the zone draws nothing yet.
    fn loading(brand: &str, custom: &str) -> Logo {
        Logo {
            loading: true,
            ..named(brand, custom)
        }
    }

    /// A loader whose worker does not answer by itself: the test holds its ends, the brands the
    /// loader asks for and the sender of the decodes.
    fn held_loader(ix: &LogoIndex) -> (DialLoader, Receiver<BrandId>, Sender<Decoded>) {
        let (jobs, inbox) = mpsc::channel();
        let (outbox, done) = mpsc::channel::<Decoded>();
        let worker = Worker {
            jobs: Some(jobs),
            done,
            in_flight: 0,
        };
        (DialLoader::with_worker(ix, worker), inbox, outbox)
    }

    #[test]
    fn a_car_shows_nothing_while_its_dial_loads_then_the_dial() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut logo = CarLogo::start(&ix);
        assert_eq!(*logo.logo(), Logo::default());
        let gt3 = ix.car_brand(&cars, 3667);
        let dial = gt3.logo.map(image_of);
        assert!(dial.is_some());
        logo.show(Some(gt3), "TJX");
        assert!(logo.busy());
        assert_eq!(
            *logo.logo(),
            loading("Porsche", "TJX"),
            "the names, marked loading so the zone stays empty, not the name"
        );
        frames_until_idle(&mut logo, image_of, |shown| {
            assert!(
                *shown == loading("Porsche", "TJX") || shown.image == dial,
                "nothing, then the dial: {shown:?}"
            );
        });
        assert_eq!(
            *logo.logo(),
            Logo {
                image: dial,
                ..named("Porsche", "TJX")
            }
        );
        // The next car of the brand has it at once.
        logo.show(Some(ix.car_brand(&cars, 3123)), "TJX");
        assert_eq!(logo.logo().image, dial);
        assert!(!logo.busy());
    }

    #[test]
    fn a_car_without_a_brand_logo_shows_the_names_at_once() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut logo = CarLogo::start(&ix);
        // A traffic car: no logo and no brand name, so the custom name shows.
        logo.show(Some(ix.car_brand(&cars, 2714)), "TJX");
        assert_eq!(*logo.logo(), named("", "TJX"));
        assert!(!logo.busy());
    }

    #[test]
    fn the_next_car_never_shows_the_last_cars_dial() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut logo = CarLogo::start(&ix);
        let (gt3, sf90) = (ix.car_brand(&cars, 3667), ix.car_brand(&cars, 3595));
        let (porsche, ferrari) = (gt3.logo.map(image_of), sf90.logo.map(image_of));
        assert!(porsche.is_some() && ferrari.is_some() && porsche != ferrari);
        logo.show(Some(gt3.clone()), "TJX");
        logo.show(Some(sf90), "TJX");
        // The Porsche dial lands first, while the Ferrari is the car on screen.
        frames_until_idle(&mut logo, image_of, |shown| {
            assert!(
                *shown == loading("Ferrari", "TJX") || shown.image == ferrari,
                "the Ferrari shows nothing or its own dial: {shown:?}"
            );
        });
        assert_eq!(
            *logo.logo(),
            Logo {
                image: ferrari,
                ..named("Ferrari", "TJX")
            }
        );
        // The Porsche dial was kept for the next Porsche.
        logo.show(Some(gt3), "TJX");
        assert_eq!(logo.logo().image, porsche);
        assert!(!logo.busy());
    }

    #[test]
    fn a_logo_zone_switched_off_while_its_dial_loads_stays_empty() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut logo = CarLogo::start(&ix);
        let gt3 = ix.car_brand(&cars, 3667);
        logo.show(Some(gt3.clone()), "TJX");
        logo.show(None, "TJX");
        assert_eq!(*logo.logo(), Logo::default());
        frames_until_idle(&mut logo, image_of, |shown| {
            assert_eq!(*shown, Logo::default(), "the dial lands, and the zone stays off");
        });
        // Switched on again, the dial is there.
        let dial = gt3.logo.map(image_of);
        logo.show(Some(gt3), "TJX");
        assert_eq!(logo.logo().image, dial);
    }

    #[test]
    fn a_logo_zone_switched_off_while_a_read_hangs_stays_empty() {
        let (ix, cars) = (repo_index(), repo_cars());
        let (loader, _inbox, _outbox) = held_loader(&ix);
        let mut logo = CarLogo::with_loader(loader);
        logo.show(Some(ix.car_brand(&cars, 3667)), "TJX");
        frame(&mut logo, image_of);
        logo.show(None, "TJX");
        // The read never comes back: the name that would have shown meanwhile must not.
        for _ in 0..=NAME_AFTER_FRAMES {
            frame(&mut logo, image_of);
            assert_eq!(*logo.logo(), Logo::default(), "the zone stays off");
        }
    }

    #[test]
    fn a_new_kit_shows_nothing_until_the_dial_is_in_it() {
        let (ix, cars) = (repo_index(), repo_cars());
        let mut logo = CarLogo::start(&ix);
        let gt3 = ix.car_brand(&cars, 3667);
        let old = gt3.logo.map(image_of);
        logo.show(Some(gt3.clone()), "TJX");
        frames_until_idle(&mut logo, image_of, |_| {});
        assert_eq!(logo.logo().image, old);
        logo.new_kit();
        assert_eq!(*logo.logo(), Logo::default(), "no image of the old kit is left to draw");
        logo.show(Some(gt3), "TJX");
        assert_eq!(*logo.logo(), loading("Porsche", "TJX"), "the brand loads again");
        let in_new_kit = |brand: BrandId| ImageId(brand.0 as u32 + 1000);
        frames_until_idle(&mut logo, in_new_kit, |shown| {
            assert!(shown.image != old, "never the old kit's image");
        });
        assert_eq!(logo.logo().image, old.map(|id| ImageId(id.0 + 1000)));
    }

    #[test]
    fn a_dial_that_does_not_arrive_shows_the_name_meanwhile_and_the_dial_if_it_comes() {
        let (ix, cars) = (repo_index(), repo_cars());
        let (loader, inbox, outbox) = held_loader(&ix);
        let mut logo = CarLogo::with_loader(loader);
        logo.show(Some(ix.car_brand(&cars, 3667)), "TJX");
        for _ in 1..NAME_AFTER_FRAMES {
            frame(&mut logo, image_of);
        }
        assert_eq!(*logo.logo(), loading("Porsche", "TJX"), "still empty a frame before");
        frame(&mut logo, image_of);
        assert_eq!(*logo.logo(), named("Porsche", "TJX"), "the name, while the read hangs");
        frame(&mut logo, image_of);
        assert_eq!(*logo.logo(), named("Porsche", "TJX"));
        assert!(logo.busy(), "and it still waits");
        // The read comes back after all.
        let brand = inbox.try_recv().expect("the decode was asked for");
        let decoded = Decoded {
            brand,
            result: ix.decode_dial(brand),
            ms: 0.0,
        };
        outbox.send(decoded).expect("the loader is listening");
        frame(&mut logo, image_of);
        assert_eq!(
            *logo.logo(),
            Logo {
                image: Some(image_of(brand)),
                ..named("Porsche", "TJX")
            }
        );
        assert!(!logo.busy());
    }

    #[test]
    fn showing_the_same_car_again_keeps_the_frames_it_has_waited() {
        let (ix, cars) = (repo_index(), repo_cars());
        let (loader, _inbox, _outbox) = held_loader(&ix);
        let mut logo = CarLogo::with_loader(loader);
        let gt3 = ix.car_brand(&cars, 3667);
        logo.show(Some(gt3.clone()), "TJX");
        for _ in 1..NAME_AFTER_FRAMES {
            frame(&mut logo, image_of);
        }
        // A setting changes a frame before the name is due: the host shows the same car again.
        logo.show(Some(gt3.clone()), "TJX");
        assert_eq!(*logo.logo(), loading("Porsche", "TJX"), "still empty");
        frame(&mut logo, image_of);
        assert_eq!(*logo.logo(), named("Porsche", "TJX"), "the wait did not start again");
        // With the name up, another change (here the custom name) keeps it up.
        logo.show(Some(gt3.clone()), "ACE");
        assert_eq!(
            *logo.logo(),
            named("Porsche", "ACE"),
            "the name stays while the read hangs"
        );
        // A car of another brand starts its own wait, and so does the first car after it.
        logo.show(Some(ix.car_brand(&cars, 3595)), "ACE");
        assert_eq!(*logo.logo(), loading("Ferrari", "ACE"));
        logo.show(Some(gt3), "ACE");
        assert_eq!(
            *logo.logo(),
            loading("Porsche", "ACE"),
            "a car change starts the wait again"
        );
        for _ in 1..NAME_AFTER_FRAMES {
            frame(&mut logo, image_of);
        }
        assert_eq!(*logo.logo(), loading("Porsche", "ACE"));
        frame(&mut logo, image_of);
        assert_eq!(*logo.logo(), named("Porsche", "ACE"));
    }

    #[test]
    fn fit_keeps_aspect_and_padding() {
        let zone = Rect::from_xywh(0.0, 0.0, 200.0, 100.0);
        let r = fit(zone, (400, 100));
        assert!((r.width() / r.height() - 4.0).abs() < 1e-4);
        let pad = 100.0 * SAFE_PADDING;
        assert!(r.min.x >= pad - 1e-3 && r.max.x <= 200.0 - pad + 1e-3);
        assert!(r.center().y < zone.center().y, "optically lifted");
    }

    #[test]
    fn wordmark_fallbacks() {
        let mut buf = [0u8; 64];
        let logo = Logo {
            image: None,
            brand_name: "Porsche".into(),
            custom_name: "TJX".into(),
            loading: false,
        };
        assert_eq!(wordmark(&logo, &mut buf), "PORSCHE");
        let logo = Logo {
            brand_name: "BMW M Motorsport".into(),
            ..Logo::default()
        };
        assert_eq!(wordmark(&logo, &mut buf), "BMW M");
        let logo = Logo {
            brand_name: String::new(),
            custom_name: "a very long custom name here".into(),
            ..Logo::default()
        };
        assert!(wordmark(&logo, &mut buf).chars().count() <= MAX_NAME_CHARS);
    }
}
