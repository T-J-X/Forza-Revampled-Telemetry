//! Font pack: OFL motorsport faces from github.com/google/fonts, pinned to one commit, with each
//! family's `OFL.txt`, and `assets/fonts/fonts.json` describing files, weights, variable axes,
//! roles, defaults, cap ratios and tabular-figure support (read from the font tables).
use crate::ctx::{Ctx, Spec};
use crate::error::{io_err, tool_err, ForgeError, Result};
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use crate::{net, svg};
use serde::Serialize;

/// (google/fonts dir, family, token roles, is default for a role)
const FAMILIES: &[(&str, &str, &[&str])] = &[
    ("sairacondensed", "Saira Condensed", &["display", "race"]),
    ("titilliumweb", "Titillium Web", &["text"]),
    ("saira", "Saira", &["light"]),
    ("barlowsemicondensed", "Barlow Semi Condensed", &["dial"]),
    ("michroma", "Michroma", &["wide"]),
    ("chakrapetch", "Chakra Petch", &["tech"]),
    ("exo2", "Exo 2", &["fluid"]),
    ("b612mono", "B612 Mono", &["aviation"]),
    ("notosans", "Noto Sans", &["fallback"]),
    ("sairasemicondensed", "Saira Semi Condensed", &[]),
    ("barlowcondensed", "Barlow Condensed", &[]),
    ("orbitron", "Orbitron", &[]),
    ("rajdhani", "Rajdhani", &[]),
    ("oxanium", "Oxanium", &[]),
    ("teko", "Teko", &[]),
    ("sharetechmono", "Share Tech Mono", &[]),
    ("russoone", "Russo One", &[]),
];

#[derive(Serialize)]
struct Axis {
    tag: String,
    min: f32,
    default: f32,
    max: f32,
}

#[derive(Serialize)]
struct FontFile {
    file: String,
    weight: u16,
    style: &'static str,
    variable: bool,
    axes: Vec<Axis>,
}

#[derive(Serialize)]
struct Tnum {
    /// The GSUB table offers the `tnum` feature.
    feature: bool,
    /// Digits 0-9 already share one advance width by default.
    digits_monospaced: bool,
}

#[derive(Serialize)]
struct Family {
    id: String,
    family: String,
    dir: String,
    files: Vec<FontFile>,
    roles: Vec<String>,
    default: bool,
    cap_ratio: f32,
    tnum: Tnum,
    licence: &'static str,
    licence_file: String,
    source: String,
}

/// Builds the font pack.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let list: Vec<&str> = FAMILIES.iter().map(|f| f.0).collect();
    let fp = fingerprint(&[b"fonts-v1", list.join(",").as_bytes(), include_bytes!("fonts.rs")]);
    ctx.job("fonts/pack", &fp, |ctx| {
        let agent = ctx.agent();
        let head = net::get_json(&agent, "https://api.github.com/repos/google/fonts/commits/main")?;
        let sha = head["sha"].as_str().ok_or_else(|| tool_err("github", "no commit sha"))?.to_owned();
        println!("    google/fonts @ {sha}");
        let mut families = Vec::new();
        for (dir, family, roles) in FAMILIES {
            let listing = net::get_json(&agent, &format!("https://api.github.com/repos/google/fonts/contents/ofl/{dir}?ref={sha}"))?;
            let entries = listing.as_array().ok_or_else(|| tool_err("github", format!("no listing for {dir}")))?;
            let id = family.to_lowercase().replace(' ', "-");
            let rel_dir = format!("assets/fonts/{id}");
            let abs_dir = ctx.abs(&rel_dir);
            std::fs::create_dir_all(&abs_dir).map_err(|e| io_err(&abs_dir, e))?;
            let source = format!("https://github.com/google/fonts/tree/{sha}/ofl/{dir}");
            let mut files = Vec::new();
            let mut cap_ratio = 0.7;
            let mut tnum = Tnum { feature: false, digits_monospaced: false };
            let mut licence_file = String::new();
            let mut preview: Option<(String, String, String, Vec<u8>)> = None;
            for e in entries {
                let name = e["name"].as_str().unwrap_or_default();
                let want_font = name.ends_with(".ttf") && !name.contains("Italic");
                let want_licence = name == "OFL.txt";
                if !(want_font || want_licence) {
                    continue;
                }
                let url = format!("https://raw.githubusercontent.com/google/fonts/{sha}/ofl/{dir}/{}", net::encode(name).replace("%5B", "[").replace("%5D", "]").replace("%2C", ","));
                let bytes = net::get_bytes(&agent, &url)?;
                let rel = format!("{rel_dir}/{name}");
                img::write_file(&ctx.abs(&rel), &bytes)?;
                if want_licence {
                    licence_file = rel.clone();
                    ctx.file(&format!("fonts/pack/{id}/licence"), &rel, "text".into(), font_spec(family, &source, "licence"), None)?;
                    continue;
                }
                let face = ttf_parser::Face::parse(&bytes, 0).map_err(|e| ForgeError::Image { path: rel.clone(), message: e.to_string() })?;
                let axes: Vec<Axis> = face
                    .variation_axes()
                    .into_iter()
                    .map(|a| Axis { tag: a.tag.to_string(), min: a.min_value, default: a.def_value, max: a.max_value })
                    .collect();
                let upem = f32::from(face.units_per_em());
                if let Some(cap) = face.capital_height() {
                    cap_ratio = f32::from(cap) / upem;
                }
                tnum = Tnum { feature: has_tnum(&face), digits_monospaced: digits_monospaced(&face) };
                files.push(FontFile {
                    file: name.to_owned(),
                    weight: face.weight().to_number(),
                    style: if face.is_italic() { "italic" } else { "normal" },
                    variable: !axes.is_empty(),
                    axes,
                });
                let file_id = format!("fonts/pack/{id}/{}", name.trim_end_matches(".ttf"));
                let dims = format!("{} glyphs", face.number_of_glyphs());
                let is_regular = name.contains("Regular") || name.contains('[') || preview.is_none();
                if is_regular {
                    preview = Some((file_id.clone(), rel.clone(), dims.clone(), bytes.clone()));
                }
                ctx.file(&file_id, &rel, dims, font_spec(family, &source, "font"), None)?;
            }
            if files.is_empty() {
                return Err(tool_err("fonts", format!("no upright TTF in ofl/{dir}")));
            }
            files.sort_by_key(|f| f.weight);
            // Specimen thumbnail for the contact sheet, rendered from the font itself.
            if let Some((file_id, rel, dims, bytes)) = preview {
                let thumb = specimen(family, &bytes)?;
                ctx.file(&file_id, &rel, dims, font_spec(family, &source, "font"), Some(&thumb))?;
            }
            let is_default = roles.contains(&"display") || roles.contains(&"text");
            families.push(Family {
                id,
                family: (*family).to_owned(),
                dir: rel_dir.trim_start_matches("assets/").to_owned(),
                files,
                roles: roles.iter().map(|r| (*r).to_owned()).collect(),
                default: is_default,
                cap_ratio: (cap_ratio * 1000.0).round() / 1000.0,
                tnum,
                licence: "OFL-1.1",
                licence_file: licence_file.trim_start_matches("assets/").to_owned(),
                source,
            });
        }
        let doc = serde_json::json!({
            "version": 1,
            "doc": "Font pack for FH6 WheelDash (asset-forge). Paths are relative to assets/. Roles match data/design_tokens.json type.fonts. Digits are always drawn with a fixed advance by fh6-kit; tnum tells whether the face can do it natively. Users may add their own fonts.",
            "source_commit": format!("google/fonts@{sha}"),
            "defaults": { "display": "saira-condensed", "text": "titillium-web", "race": "saira-condensed", "light": "saira", "dial": "barlow-semi-condensed", "wide": "michroma", "tech": "chakra-petch", "fluid": "exo-2", "aviation": "b612-mono", "fallback": "noto-sans" },
            "families": families,
        });
        let text = serde_json::to_string_pretty(&doc).map_err(|source| ForgeError::Json { context: "fonts.json".into(), source })? + "\n";
        let rel = "assets/fonts/fonts.json";
        img::write_file(&ctx.abs(rel), text.as_bytes())?;
        ctx.file("fonts/pack/fonts_json", rel, format!("{} families", FAMILIES.len()), font_spec("all", "asset-forge", "data").licence("MIT (asset-forge)"), None)?;
        Ok(())
    })
}

fn font_spec(family: &str, source: &str, kind: &'static str) -> Spec {
    let clusters: &'static str = "all clusters (type roles in data/design_tokens.json), OSD, configurator";
    Spec::new("fonts", kind, format!("{family}: {source}"), clusters)
        .space("n/a")
        .licence("OFL-1.1")
}

fn has_tnum(face: &ttf_parser::Face<'_>) -> bool {
    let tag = ttf_parser::Tag::from_bytes(b"tnum");
    face.tables()
        .gsub
        .is_some_and(|g| g.features.into_iter().any(|f| f.tag == tag))
}

fn digits_monospaced(face: &ttf_parser::Face<'_>) -> bool {
    let adv: Vec<u16> = ('0'..='9')
        .filter_map(|c| face.glyph_index(c).and_then(|g| face.glyph_hor_advance(g)))
        .collect();
    adv.len() == 10 && adv.iter().all(|a| *a == adv[0])
}

/// A specimen image: the family name and "4 128 km/h" drawn with the font through resvg.
fn specimen(family: &str, bytes: &[u8]) -> Result<Img> {
    let mut opt = svg::options(&[]);
    std::sync::Arc::make_mut(&mut opt.fontdb).load_font_data(bytes.to_vec());
    let text = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="400"><rect width="1200" height="400" fill="#000000"/><text x="40" y="170" font-family="{family}" font-size="150" fill="#FFFFFF">4 128 km/h</text><text x="44" y="320" font-family="{family}" font-size="72" fill="#9AA4B2">{family}</text></svg>"##
    );
    let tree = svg::parse(text.as_bytes(), &opt, family)?;
    svg::render_longest(&tree, 1200, family)
}
