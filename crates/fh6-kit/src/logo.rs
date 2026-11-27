//! The logo zone: a brand logo fitted with brand-safe padding and optical centring, falling back
//! to the brand name, then the custom name, set in the wide (wordmark) face.
//!
//! [`LogoIndex`] reads `assets/logos/logos.json` once at start-up and says which brand a car
//! shows ([`LogoIndex::car_brand`]). The host loads that brand's dial image with
//! [`LogoIndex::load_dial`] when the car changes, never per frame, and passes it in
//! [`Logo::image`]. The image store keeps every dial it loaded, so a brand seen again costs
//! nothing, and all 91 dials together stay under 200 MB of video memory.
use crate::cluster::Logo;
use crate::color::Color;
use crate::images::{ImageError, Images};
use crate::math::{vec2, Rect, Vec2};
use crate::paint::{ImageId, ImagePaint, Paint};
use crate::painter::Painter;
use crate::shape::Shape;
use crate::text::{Align, FontId, TextStyle, VAlign};
use fh6_core::CarDatabase;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

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
/// The generic emblem's key in the index. It stands for "no known brand" and is never shown as
/// a brand's logo.
pub const GENERIC_BRAND: &str = "DEFAULT";

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
/// brand-safe padding, lifted for optical centring.
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
/// case, at most [`MAX_NAME_CHARS`] characters. Writes into `buf` (no allocation).
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
    /// The index file could not be read.
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
        let Some(car) = cars.get_car_entry(&ordinal.to_string()) else {
            return CarBrand::default();
        };
        let logo = self.resolve(&car.brand_key, &car.manufacturer);
        let key = car.brand_key.trim();
        // The palette's name for a branded car, as the wordmark showed it before logos loaded.
        // The generic palette ("Standard Horizon") names a theme, not a brand.
        let palette_name = if key.is_empty() || key.eq_ignore_ascii_case(GENERIC_BRAND) {
            None
        } else {
            cars.get_brand_palette(key)
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

    /// Loads the brand's dial image into `images`. The store keeps it: a later call for the same
    /// unchanged file returns the same image without reading it again. Call it when the car
    /// changes, never per frame.
    ///
    /// # Errors
    /// [`LogoError::NoDial`] when the index names no usable dial image for the brand,
    /// [`LogoError::Image`] when the file cannot be read or decoded.
    pub fn load_dial(
        &self,
        images: &mut Images,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        brand: BrandId,
    ) -> Result<ImageId, LogoError> {
        let path = self
            .dial_path(brand)
            .ok_or_else(|| LogoError::NoDial(self.key(brand).unwrap_or("?").to_string()))?;
        Ok(images.load_file(device, queue, &path, None)?)
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

/// `dial` as a file path that stays inside the folder it is relative to, or `None`.
fn plain_relative(dial: &str) -> Option<PathBuf> {
    let path = Path::new(dial.trim());
    let inside = path
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    let names_a_file = matches!(path.components().next_back(), Some(Component::Normal(_)));
    (inside && names_a_file).then(|| path.to_path_buf())
}

/// Draws the logo zone. Returns the rectangle actually used.
pub fn draw(p: &mut Painter<'_>, zone: Rect, logo: &Logo, style: &LogoStyle) -> Rect {
    if let Some(image) = logo.image {
        if let Some(size) = p.images().size(image) {
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

    fn repo_assets() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
    }

    fn repo_index() -> LogoIndex {
        LogoIndex::load(&repo_assets()).expect("assets/logos/logos.json loads")
    }

    fn repo_cars() -> CarDatabase {
        CarDatabase::load(repo_assets().join("car_brands/car_brands.json"))
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
        assert!(!cars.cars.is_empty(), "car_brands.json loads");
        let mut generic = Vec::new();
        for (ordinal, car) in &cars.cars {
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
        let (ordinal, renault) = cars
            .cars
            .iter()
            .find(|(_, c)| c.brand_key == GENERIC_BRAND && c.manufacturer == "Renault")
            .expect("a generic-keyed Renault in car_brands.json");
        let renault_ordinal: i32 = ordinal.parse().expect("numeric ordinal");
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
    fn every_dial_exists_decodes_and_fits_video_memory() {
        let ix = repo_index();
        assert!(ix.len() > 50, "the Phase 6 set has 91 brands");
        let mut bytes = 0u64;
        for i in 0..ix.len() {
            let id = BrandId(i);
            let path = ix.dial_path(id).expect("every brand names its dial");
            let png = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let (w, h, _) = crate::images::decode_png(&png).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert!(w.max(h) <= 1024, "{} is {w}x{h}", path.display());
            // RGBA8 with a full mip chain is 4/3 of the base level.
            bytes += u64::from(w) * u64::from(h) * 4 * 4 / 3;
        }
        assert!(bytes < 256 << 20, "every dial at once is {} MB", bytes >> 20);
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
