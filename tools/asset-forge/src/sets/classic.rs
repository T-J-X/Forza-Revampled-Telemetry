//! Classic and motorsport materials: tileable PBR-style plates (albedo, world-lit, normal and
//! roughness/metallic/height), radial dial faces and lathed bezel rings with AO.
use crate::ctx::{Ctx, Format, Spec};
use crate::error::Result;
use crate::img::{self, Img};
use crate::ktx2w::Encoding;
use crate::ledger::fingerprint;
use crate::{blender, recipes};
use serde_json::json;

/// (name, normal strength, intended clusters)
const MATERIALS: &[(&str, f32, &str)] = &[
    (
        "brushed_titanium",
        2.0,
        "PorscheAnalog, Classic Analog plates, Race Dash bars",
    ),
    ("anodised_black", 1.0, "BoschDdu housing, Race Dash, Classic Analog"),
    ("anodised_blue", 1.0, "Race Dash accents, Classic Analog sport dials"),
    (
        "guilloche_clous",
        5.0,
        "Classic Analog dials (clous de Paris), horology concepts",
    ),
    ("guilloche_barleycorn", 4.0, "Classic Analog dials (barleycorn)"),
    (
        "guilloche_wave",
        4.0,
        "Classic Analog dials (flinqué under blue enamel)",
    ),
    ("carbon_twill", 4.0, "BoschDdu, Race Dash carbon, GT7 hood plate"),
    ("carbon_forged", 3.0, "Race Dash, hypercar concepts"),
    ("enamel_white", 1.0, "Classic Analog grand-feu dials"),
    ("alcantara", 2.0, "OSD backdrop, steering-column trim"),
    ("leather", 3.0, "OSD backdrop, classic trim"),
];

const DIALS: &[&str] = &["dial_sunburst_slate", "dial_guilloche_rose", "dial_enamel_white"];

/// Builds every classic job.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    materials(ctx)?;
    dials(ctx)?;
    bezels(ctx)?;
    Ok(())
}

fn materials(ctx: &mut Ctx) -> Result<()> {
    let names: Vec<&str> = MATERIALS.iter().map(|m| m.0).collect();
    let params = json!({ "materials": names, "size": 2048, "samples": 256, "seed": 71 });
    let body = format!("{}\n{}", recipes::classic::EXPR_HELPERS, recipes::classic::MATERIALS);
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        body.as_bytes(),
        params.to_string().as_bytes(),
        b"post-v2",
    ]);
    ctx.job("classic/materials", &fp, |ctx| {
        let out = ctx.scratch("classic_materials")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "classic_materials", &body, &p)?;
        for (name, strength, clusters) in MATERIALS {
            let gen = format!("blender:classic::MATERIALS {name} (procedural nodes, torus-periodic) seed=71");
            write_material(ctx, &out, name, *strength, clusters, &gen, true, None)?;
        }
        Ok(())
    })
}

/// Writes albedo / lit / normal / rmh for one rendered material. `disc` masks a round dial.
#[allow(clippy::too_many_arguments)]
fn write_material(
    ctx: &mut Ctx,
    out: &std::path::Path,
    name: &str,
    strength: f32,
    clusters: &'static str,
    gen: &str,
    tile: bool,
    disc: Option<f32>,
) -> Result<()> {
    let mask = |x: usize, y: usize, w: usize| -> f32 {
        match disc {
            Some(r) => {
                let c = w as f32 * 0.5;
                let d = ((x as f32 + 0.5 - c).powi(2) + (y as f32 + 0.5 - c).powi(2)).sqrt() / c;
                1.0 - img::smoothstep(r - 2.0 / c, r, d)
            }
            None => 1.0,
        }
    };
    let albedo_lin = img::load_exr(&out.join(format!("{name}_albedo.exr")))?;
    let hrm = img::load_exr(&out.join(format!("{name}_hrm.exr")))?;
    let lit = img::load(&out.join(format!("{name}_lit.png")))?;
    let w = albedo_lin.w;
    let c = if disc.is_some() { 4 } else { 3 };
    let albedo = Img::from_fn(w, albedo_lin.h, c, |x, y| {
        [
            img::srgb_encode(albedo_lin.get(x, y, 0)),
            img::srgb_encode(albedo_lin.get(x, y, 1)),
            img::srgb_encode(albedo_lin.get(x, y, 2)),
            mask(x, y, w),
        ]
    });
    let lit = Img::from_fn(lit.w, lit.h, c, |x, y| {
        [
            lit.get(x, y, 0),
            lit.get(x, y, 1),
            lit.get(x, y, 2),
            mask(x, y, w).min(lit.get(x, y, 3)),
        ]
    });
    let mut normal = img::normal_from_height(&hrm, 0, strength * w as f32 / 512.0, tile).to_channels(4);
    let rmh = Img::from_fn(w, hrm.h, 4, |x, y| {
        [hrm.get(x, y, 1), hrm.get(x, y, 2), hrm.get(x, y, 0), mask(x, y, w)]
    });
    for y in 0..normal.h {
        for x in 0..normal.w {
            normal.set(x, y, 3, mask(x, y, w));
        }
    }
    // Record ids live under the job id so `is_fresh` and stale-file cleanup find them.
    let id = if name.starts_with("dial_") {
        format!("classic/dials/{name}")
    } else {
        format!("classic/materials/{name}")
    };
    let alpha = if disc.is_some() { "straight" } else { "none" };
    let mk = |kind: &'static str| {
        let mut s = Spec::new("classic", kind, gen.to_owned(), clusters).alpha(alpha);
        if tile {
            s = s.tile();
        }
        s
    };
    let seam = if tile {
        format!("tile seam score {:.2}", img::tile_seam_score(&lit))
    } else {
        String::new()
    };
    ctx.image(
        &format!("{id}/albedo"),
        &format!("assets/classic/{name}_albedo.png"),
        &albedo,
        Format::Png8,
        mk("color").qa("flat albedo (no lighting)"),
    )?;
    ctx.image(
        &format!("{id}/lit"),
        &format!("assets/classic/{name}_lit.png"),
        &lit,
        Format::Png8,
        mk("color").qa(format!("baked studio light; {seam}")),
    )?;
    // Data maps ship smaller than the colour maps: noisy maps compress poorly in KTX2 (zstd).
    let mut normal = normal.resize(w / 2, normal.h / 2, tile);
    img::renormalize(&mut normal);
    let rmh = rmh.resize(w / 4, rmh.h / 4, tile);
    ctx.image(
        &format!("{id}/normal"),
        &format!("assets/classic/{name}_normal.ktx2"),
        &normal,
        Format::Ktx2(Encoding::Normal),
        mk("normal").qa("from height (Sobel at full size), shipped at half size"),
    )?;
    ctx.image(
        &format!("{id}/rmh"),
        &format!("assets/classic/{name}_rmh.ktx2"),
        &rmh,
        Format::Ktx2(Encoding::Linear),
        mk("data").qa("R = roughness, G = metallic, B = height; quarter size"),
    )?;
    Ok(())
}

fn dials(ctx: &mut Ctx) -> Result<()> {
    let params = json!({ "dials": DIALS, "size": 2048, "samples": 256, "seed": 72 });
    let body = format!("{}\n{}", recipes::classic::EXPR_HELPERS, recipes::classic::DIALS);
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        body.as_bytes(),
        params.to_string().as_bytes(),
        b"post-v2",
    ]);
    ctx.job("classic/dials", &fp, |ctx| {
        let out = ctx.scratch("classic_dials")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "classic_dials", &body, &p)?;
        for name in DIALS {
            let gen = format!("blender:classic::DIALS {name} (radial procedural face) seed=72");
            write_material(
                ctx,
                &out,
                name,
                3.0,
                "PorscheAnalog, Classic Analog family dial faces",
                &gen,
                false,
                Some(0.99),
            )?;
        }
        Ok(())
    })
}

fn bezels(ctx: &mut Ctx) -> Result<()> {
    let bezels = json!([
        {"name": "bezel_thin", "inner": 0.92, "outer": 1.0, "height": 0.03, "style": "plain"},
        {"name": "bezel_medium", "inner": 0.84, "outer": 1.0, "height": 0.05, "style": "plain"},
        {"name": "bezel_thick", "inner": 0.74, "outer": 1.0, "height": 0.07, "style": "plain"},
        {"name": "bezel_knurled", "inner": 0.82, "outer": 1.0, "height": 0.055, "style": "knurled", "knurls": 120},
        {"name": "bezel_fluted", "inner": 0.8, "outer": 1.0, "height": 0.06, "style": "fluted", "knurls": 90}
    ]);
    let params = json!({ "bezels": bezels, "size": 2048, "samples": 256, "seed": 73 });
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        recipes::classic::BEZELS.as_bytes(),
        params.to_string().as_bytes(),
        b"post-v1",
    ]);
    ctx.job("classic/bezels", &fp, |ctx| {
        let out = ctx.scratch("classic_bezels")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "classic_bezels", recipes::classic::BEZELS, &p)?;
        let clusters = "PorscheAnalog, Classic Analog family, BoschDdu dials";
        for b in bezels.as_array().into_iter().flatten() {
            let name = b["name"].as_str().unwrap_or_default();
            let inner = b["inner"].as_f64().unwrap_or(0.0);
            let gen = format!("blender:classic::BEZELS {name} (lathed profile, inner/outer {inner:.2}) seed=73");
            let data = img::load_exr(&out.join(format!("{name}_data.exr")))?;
            let (normal, _h) = crate::sets::glass::unpack_data(&data);
            ctx.image(
                &format!("classic/bezels/{name}_normal"),
                &format!("assets/classic/{name}_normal.ktx2"),
                &normal,
                Format::Ktx2(Encoding::Normal),
                Spec::new("classic", "normal", gen.clone(), clusters)
                    .alpha("straight")
                    .qa("A = coverage; outer radius = 98% of half-size"),
            )?;
            let ao = img::load_exr(&out.join(format!("{name}_ao.exr")))?;
            let ao1 = Img::from_fn(ao.w, ao.h, 2, |x, y| {
                let a = ao.get(x, y, 3);
                [
                    if a > 1e-4 {
                        (ao.get(x, y, 0) / a).clamp(0.0, 1.0)
                    } else {
                        1.0
                    },
                    a,
                    0.0,
                    0.0,
                ]
            });
            ctx.image(
                &format!("classic/bezels/{name}_ao"),
                &format!("assets/classic/{name}_ao.ktx2"),
                &ao1,
                Format::Ktx2(Encoding::Linear),
                Spec::new("classic", "mask", gen.clone(), clusters)
                    .alpha("straight")
                    .qa("R = ambient occlusion, A = coverage"),
            )?;
            for fin in ["steel", "blackchrome"] {
                let lit = img::load(&out.join(format!("{name}_{fin}.png")))?;
                ctx.image(
                    &format!("classic/bezels/{name}_{fin}"),
                    &format!("assets/classic/{name}_{fin}.png"),
                    &lit,
                    Format::Png8,
                    Spec::new("classic", "color", gen.clone(), clusters)
                        .alpha("straight")
                        .qa(format!("{fin}, baked studio light")),
                )?;
            }
        }
        Ok(())
    })
}
