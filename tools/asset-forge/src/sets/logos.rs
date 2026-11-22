//! Logos: every brand in `assets/car_brands/car_brands.json` (palette keys plus the makes in
//! `Cars`). Sources are pinned in `tools/asset-forge/data/logos.json`: official SVGs from
//! Wikimedia Commons, or a remade wordmark set in an OFL face from the font pack and outlined.
//! Each brand ships the SVG, a 4096 px full-colour PNG, a 4096 px monochrome-white PNG and a
//! 1024 px "dial" PNG tuned for small sizes on dark dials, plus `assets/logos/logos.json`.
use crate::ctx::{Ctx, Format, Spec};
use crate::error::{ForgeError, Result};
use crate::img::{self, smoothstep, Img};
use crate::ledger::fingerprint;
use crate::{net, svg};
use serde_json::{json, Map, Value};
use std::path::PathBuf;

const CLUSTERS: &str = "every cluster's logo zone (fh6-kit logo.rs), OSD, configurator";

/// Builds the logo set: one job per brand (so a tweak rebuilds one logo), then the index job
/// that writes `logos.json` and `NOTICE.md`.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let src_path = ctx.paths.forge_dir().join("data").join("logos.json");
    let src = crate::ctx::read(&src_path)?;
    let fonts_json = std::fs::read(ctx.abs("assets/fonts/fonts.json")).unwrap_or_default();
    let doc: Value = serde_json::from_slice(&src).map_err(|source| ForgeError::Json { context: src_path.display().to_string(), source })?;
    let fonts: Value = serde_json::from_slice(&fonts_json).unwrap_or(Value::Null);
    let font_dirs: Vec<PathBuf> = std::fs::read_dir(ctx.abs("assets/fonts"))
        .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_dir()).collect())
        .unwrap_or_default();
    let dir_refs: Vec<&std::path::Path> = font_dirs.iter().map(PathBuf::as_path).collect();
    let opt = svg::options(&dir_refs);
    let meta_dir = ctx.scratch("logos_meta")?;
    let previous: Value = std::fs::read(ctx.abs("assets/logos/logos.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(Value::Null);
    let mut all_fp: Vec<String> = Vec::new();
    let mut failures = Vec::new();
    let logos: Vec<Value> = doc["logos"].as_array().cloned().unwrap_or_default();
    for l in &logos {
        let key = l["key"].as_str().unwrap_or_default().to_owned();
        let slug = key.to_lowercase();
        let uses_fonts: &[u8] = if l["source"].as_str() == Some("commons") { b"" } else { &fonts_json };
        let fp = fingerprint(&[b"logo-v2", l.to_string().as_bytes(), include_bytes!("logos.rs"), uses_fonts]);
        all_fp.push(fp.clone());
        let meta_path = meta_dir.join(format!("{slug}.json"));
        let res = ctx.job(&format!("logos/{slug}"), &fp, |ctx| {
            let agent = ctx.agent();
            let entry = one(ctx, &agent, &opt, &fonts, l)?;
            img::write_file(&meta_path, entry.to_string().as_bytes())?;
            Ok(())
        });
        if let Err(e) = res {
            failures.push(format!("{key}: {e}"));
        }
    }
    if !failures.is_empty() {
        return Err(ForgeError::Check { asset: "logos".into(), message: failures.join("\n") });
    }
    let refs: Vec<&[u8]> = all_fp.iter().map(String::as_bytes).collect();
    let index_fp = fingerprint(&refs);
    ctx.job("logos/index", &index_fp, |ctx| {
        let mut out_map = Map::new();
        let mut aliases = Map::new();
        for l in &logos {
            let key = l["key"].as_str().unwrap_or_default().to_owned();
            let slug = key.to_lowercase();
            // This run's metadata, or the committed entry for logos that were fresh.
            let entry = std::fs::read(meta_dir.join(format!("{slug}.json")))
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                .or_else(|| previous["logos"].get(&key).cloned())
                .ok_or_else(|| ForgeError::Check { asset: key.clone(), message: "no metadata; rebuild with --force".into() })?;
            for a in l["aliases"].as_array().into_iter().flatten() {
                if let Some(a) = a.as_str() {
                    aliases.insert(a.to_owned(), json!(key));
                }
            }
            aliases.insert(key.clone(), json!(key));
            out_map.insert(key, entry);
        }
        let total = out_map.len();
        let logos_json = json!({
            "version": 1,
            "doc": "Brand logos for FH6 WheelDash, keyed like assets/car_brands/car_brands.json (BrandPalettes keys plus makes from Cars). Paths are relative to assets/logos/. full = official colours, mono = white with alpha, dial = 1024 px white tuned for small sizes on dark dials. Resolve a car by BrandKey, or for DEFAULT cars by Manufacturer through `aliases`. Trademarks belong to their owners; see NOTICE.md.",
            "count": total,
            "logos": out_map,
            "aliases": aliases,
        });
        let text = serde_json::to_string_pretty(&logos_json).map_err(|source| ForgeError::Json { context: "logos.json".into(), source })? + "\n";
        img::write_file(&ctx.abs("assets/logos/logos.json"), text.as_bytes())?;
        ctx.file("logos/index/logos_json", "assets/logos/logos.json", format!("{total} brands"), Spec::new("logos", "data", "asset-forge logos set", CLUSTERS).space("n/a"), None)?;
        img::write_file(&ctx.abs("assets/logos/NOTICE.md"), NOTICE.as_bytes())?;
        ctx.file("logos/index/notice", "assets/logos/NOTICE.md", "text".into(), Spec::new("logos", "licence", "asset-forge logos set", CLUSTERS).space("n/a"), None)?;
        Ok(())
    })
}

const NOTICE: &str = "# Logo notice\n\nThe car logos and brand names in this folder are trademarks of their respective owners. FH6 WheelDash uses them only to identify the car being driven, in the way a car's own instrument cluster shows its maker's badge. No endorsement by, or affiliation with, any manufacturer is implied.\n\nSources are recorded per logo in `logos.json` and `../ASSETS.md`: official artwork from Wikimedia Commons (mostly public domain as simple text logos; one CC BY-SA 4.0 file is credited there), or a wordmark remade by `tools/asset-forge` in an SIL OFL font where no usable vector artwork exists. Remade files are marked `\"remade\": true`.\n\nIf you own one of these marks and want it changed or removed, please open an issue on the repository.\n";

fn one(ctx: &mut Ctx, agent: &ureq::Agent, opt: &resvg::usvg::Options<'_>, fonts: &Value, l: &Value) -> Result<Value> {
    let key = l["key"].as_str().unwrap_or_default();
    let name = l["name"].as_str().unwrap_or(key);
    let slug = key.to_lowercase();
    let svg_rel = format!("assets/logos/svg/{slug}.svg");
    let source = l["source"].as_str().unwrap_or("remade");
    let (svg_text, generator, licence, remade) = match source {
        "commons" => {
            let url = l["url"].as_str().unwrap_or_default();
            let bytes = net::get_bytes(agent, url)?;
            std::thread::sleep(std::time::Duration::from_millis(250));
            let attribution = l["attribution"].as_str().unwrap_or_default();
            let lic = l["licence"].as_str().unwrap_or("see source");
            let lic_text = if attribution.is_empty() || lic == "Public domain" { lic.to_owned() } else { format!("{lic} ({attribution})") };
            (String::from_utf8_lossy(&bytes).to_string(), format!("{} — {}", l["page"].as_str().unwrap_or(url), l["file"].as_str().unwrap_or_default()), lic_text, false)
        }
        _ => {
            let svg = if l["generic"].as_bool() == Some(true) { generic_svg() } else { wordmark_svg(l, fonts) };
            let tree = svg::parse(svg.as_bytes(), opt, key)?;
            let outlined = svg::outlined(&tree);
            let font = l["wordmark"]["font"].as_str().unwrap_or("geometric");
            (outlined, format!("rust:sets::logos remade ({font}, outlined)"), l["licence"].as_str().unwrap_or("MIT (asset-forge)").to_owned(), true)
        }
    };
    img::write_file(&ctx.abs(&svg_rel), svg_text.as_bytes())?;
    let tree = svg::parse(svg_text.as_bytes(), opt, &svg_rel)?;
    let mut full = trim(&svg::render_longest(&tree, 4096, &svg_rel)?, 4096);
    // Optional crop [x0, y0, x1, y1] as fractions of the trimmed art (drops sub-lines such as a
    // team name), then trim and re-fit again.
    if let Some(c) = l["crop"].as_array() {
        let f = |i: usize| c.get(i).and_then(Value::as_f64).unwrap_or(if i < 2 { 0.0 } else { 1.0 }) as f32;
        let (x0, y0) = ((f(0) * full.w as f32) as usize, (f(1) * full.h as f32) as usize);
        let (x1, y1) = ((f(2) * full.w as f32) as usize, (f(3) * full.h as f32) as usize);
        full = trim(&full.crop(x0, y0, x1.saturating_sub(x0).max(1), y1.saturating_sub(y0).max(1)), 4096);
    }
    let mode = match l["mono"].as_str() {
        Some("silhouette") => Mono::Silhouette,
        Some("knockout") => Mono::Knockout,
        Some("ink") => Mono::Ink,
        _ => classify(&full),
    };
    let mono = to_mono(&full, mode);
    let dial = dial_variant(&mono, mode);
    let base = format!("assets/logos/{slug}");
    let spec = |kind: &'static str, q: &str| Spec::new("logos", kind, generator.clone(), CLUSTERS).alpha("straight").licence(licence.clone()).qa(q.to_owned());
    ctx.file(&format!("logos/{slug}/svg"), &svg_rel, "vector".into(), spec("vector", if remade { "remade wordmark" } else { "official SVG, unmodified" }).space("srgb"), Some(&full))?;
    ctx.image(&format!("logos/{slug}/full"), &format!("{base}_full.png"), &full, Format::Png8, spec("logo", "full colour, 4096 px longest side"))?;
    ctx.image(&format!("logos/{slug}/mono"), &format!("{base}_mono.png"), &mono, Format::Png8, spec("logo", &format!("white, {mode:?}")))?;
    ctx.image(&format!("logos/{slug}/dial"), &format!("{base}_dial.png"), &dial, Format::Png8, spec("logo", "white, 1024 px, strokes thickened for small sizes"))?;
    Ok(json!({
        "name": name,
        "kind": l["kind"],
        "svg": format!("svg/{slug}.svg"),
        "full": format!("{slug}_full.png"),
        "mono": format!("{slug}_mono.png"),
        "dial": format!("{slug}_dial.png"),
        "aspect": (full.w as f32 / full.h as f32 * 1000.0).round() / 1000.0,
        "mono_mode": format!("{mode:?}").to_lowercase(),
        "remade": remade,
        "source": if remade { Value::String("asset-forge".into()) } else { l["page"].clone() },
        "licence": licence,
    }))
}

fn wordmark_svg(l: &Value, fonts: &Value) -> String {
    let w = &l["wordmark"];
    let text = w["text"].as_str().unwrap_or("BRAND");
    let font_id = w["font"].as_str().unwrap_or("michroma");
    let family = fonts["families"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|f| f["id"].as_str() == Some(font_id))
        .and_then(|f| f["family"].as_str())
        .unwrap_or("Michroma")
        .to_owned();
    let weight = w["weight"].as_u64().unwrap_or(400);
    let tracking = w["tracking"].as_f64().unwrap_or(0.12);
    let colour = w["colour"].as_str().unwrap_or("#E8ECF2");
    let size = 200.0;
    let width = (text.chars().count() as f64 * size * (0.78 + tracking)).ceil() + 80.0;
    let esc = text.replace('&', "&amp;").replace('<', "&lt;");
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="300" viewBox="0 0 {width} 300"><text x="{}" y="222" text-anchor="middle" font-family="{family}" font-weight="{weight}" font-size="{size}" letter-spacing="{}" fill="{colour}">{esc}</text></svg>"##,
        width / 2.0,
        tracking * size
    )
}

/// A neutral steering-wheel mark for traffic vehicles and unknown makes.
fn generic_svg() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="1000" viewBox="0 0 1000 1000"><g fill="none" stroke="#E8ECF2" stroke-linecap="round"><circle cx="500" cy="500" r="420" stroke-width="70"/><circle cx="500" cy="540" r="110" stroke-width="60"/><path d="M110 470 Q500 380 890 470" stroke-width="70"/><path d="M500 650 V915" stroke-width="80"/></g></svg>"##.to_owned()
}

/// Crops transparent margins and re-fits so the longest side is `longest`.
fn trim(img: &Img, longest: usize) -> Img {
    let (mut x0, mut y0, mut x1, mut y1) = (img.w, img.h, 0, 0);
    for y in 0..img.h {
        for x in 0..img.w {
            if img.get(x, y, 3) > 0.004 {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if x1 < x0 || y1 < y0 {
        return img.clone();
    }
    let c = img.crop(x0, y0, x1 - x0 + 1, y1 - y0 + 1);
    let k = longest as f32 / c.w.max(c.h) as f32;
    if (k - 1.0).abs() < 0.02 {
        return c;
    }
    c.resize(((c.w as f32 * k).round() as usize).max(1), ((c.h as f32 * k).round() as usize).max(1), false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mono {
    /// All ink becomes white.
    Silhouette,
    /// Light parts white, dark parts transparent (badges with internal detail).
    Knockout,
    /// Dark parts white, light parts transparent (black-and-white crests, grey-ringed roundels).
    Ink,
}

/// Silhouette for single-ink or text-on-transparent logos; knockout for filled badges whose dark
/// areas sit inside lighter ink (so a dark dial shows through as the badge's dark parts).
fn classify(full: &Img) -> Mono {
    let small = full.resize((full.w / 4).max(1), (full.h / 4).max(1), false);
    let mut ls = Vec::new();
    for y in 0..small.h {
        for x in 0..small.w {
            if small.get(x, y, 3) > 0.6 {
                ls.push(img::luma(small.get(x, y, 0), small.get(x, y, 1), small.get(x, y, 2)));
            }
        }
    }
    if ls.len() < 16 {
        return Mono::Silhouette;
    }
    let mut sorted = ls.clone();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let p5 = sorted[sorted.len() / 20];
    let p95 = sorted[sorted.len() * 19 / 20];
    if p95 - p5 < 0.25 {
        return Mono::Silhouette;
    }
    let mid = (p5 + p95) * 0.5;
    let (mut dark, mut edge) = (0usize, 0usize);
    for y in 0..small.h {
        for x in 0..small.w {
            if small.get(x, y, 3) <= 0.6 {
                continue;
            }
            if img::luma(small.get(x, y, 0), small.get(x, y, 1), small.get(x, y, 2)) >= mid {
                continue;
            }
            dark += 1;
            let near_clear = (-2isize..=2).any(|dy| (-2isize..=2).any(|dx| small.get_clamp(x as isize + dx, y as isize + dy, 3) < 0.1));
            if near_clear {
                edge += 1;
            }
        }
    }
    if dark == 0 || edge as f32 / dark as f32 > 0.5 {
        Mono::Silhouette
    } else {
        Mono::Knockout
    }
}

fn to_mono(full: &Img, mode: Mono) -> Img {
    let mut lo = 1.0f32;
    let mut hi = 0.0f32;
    if mode != Mono::Silhouette {
        let mut ls: Vec<f32> = (0..full.h)
            .flat_map(|y| (0..full.w).map(move |x| (x, y)))
            .filter(|&(x, y)| full.get(x, y, 3) > 0.6)
            .map(|(x, y)| img::srgb_encode(img::luma(full.get(x, y, 0), full.get(x, y, 1), full.get(x, y, 2))))
            .collect();
        ls.sort_by(|a, b| a.total_cmp(b));
        if !ls.is_empty() {
            lo = ls[ls.len() / 20];
            hi = ls[ls.len() * 19 / 20];
        }
    }
    Img::from_fn(full.w, full.h, 4, |x, y| {
        let a = full.get(x, y, 3);
        let k = match mode {
            Mono::Silhouette => 1.0,
            Mono::Knockout => {
                let l = img::srgb_encode(img::luma(full.get(x, y, 0), full.get(x, y, 1), full.get(x, y, 2)));
                smoothstep(lo, hi.max(lo + 0.05), l)
            }
            Mono::Ink => {
                let l = img::srgb_encode(img::luma(full.get(x, y, 0), full.get(x, y, 1), full.get(x, y, 2)));
                (1.0 - smoothstep(lo, hi.max(lo + 0.05), l)).sqrt()
            }
        };
        [1.0, 1.0, 1.0, a * k]
    })
}

/// 1024 px, with strokes thickened (alpha dilation) so thin details survive small sizes.
fn dial_variant(mono: &Img, mode: Mono) -> Img {
    let k = 960.0 / mono.w.max(mono.h) as f32;
    let small = mono.resize(((mono.w as f32 * k).round() as usize).max(1), ((mono.h as f32 * k).round() as usize).max(1), false);
    let r: isize = if mode == Mono::Silhouette { 3 } else { 1 };
    let (w, h) = (small.w + 64, small.h + 64);
    let mut padded = Img::new(w, h, 4);
    padded.paste(&small, 32, 32);
    let horiz = Img::from_fn(w, h, 1, |x, y| {
        let m = (-r..=r).map(|d| padded.get_clamp(x as isize + d, y as isize, 3)).fold(0.0f32, f32::max);
        [m, 0.0, 0.0, 0.0]
    });
    Img::from_fn(w, h, 4, |x, y| {
        let m = (-r..=r).map(|d| horiz.get_clamp(x as isize, y as isize + d, 0)).fold(0.0f32, f32::max);
        // Firm the edge a little so the mark reads crisply on a dark dial.
        [1.0, 1.0, 1.0, smoothstep(0.08, 0.85, m)]
    })
}
