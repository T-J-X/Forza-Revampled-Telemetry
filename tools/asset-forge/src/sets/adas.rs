//! ADAS set: low-poly glTF cars (Blender loft, ≤ 5k triangles, clean smooth-by-angle normals)
//! with rim-lit chase-view sprites, and the lane / road decal sheet (vector SVG made here).
use crate::ctx::{Ctx, Format, Spec};
use crate::error::{ForgeError, Result};
use crate::img;
use crate::ledger::fingerprint;
use crate::{blender, recipes, svg};
use serde_json::json;
use std::fmt::Write as _;

const CLUSTERS: &str = "TeslaAdas, ADAS HUD family, WindshieldHud";
/// Triangle budget per car (MASTER_PROMPT).
pub const MAX_TRIS: u64 = 5000;

/// Builds every ADAS job.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    cars(ctx)?;
    decals(ctx)?;
    Ok(())
}

fn cars(ctx: &mut Ctx) -> Result<()> {
    let cars = json!([
        {"name": "ego_coupe", "L": 4.55, "W": 1.92, "H": 1.30, "ride": 0.12, "wr": 0.345, "wb": 2.72, "belt": 0.66, "hood": 0.56, "deck": 0.62, "cabin_front": 0.22, "cabin_rear": -0.62, "roof_peak": 0.42, "tail": 0.45, "plan_n": 5.0, "paint": [0.80, 0.82, 0.86]},
        {"name": "sport_sedan", "L": 4.85, "W": 1.87, "H": 1.44, "ride": 0.13, "wr": 0.335, "wb": 2.90, "belt": 0.64, "hood": 0.55, "deck": 0.62, "cabin_front": 0.26, "cabin_rear": -0.56, "roof_peak": 0.36, "tail": 0.3, "plan_n": 5.0, "paint": [0.36, 0.38, 0.42]},
        {"name": "suv", "L": 4.85, "W": 1.96, "H": 1.74, "ride": 0.21, "wr": 0.385, "wb": 2.92, "belt": 0.62, "hood": 0.56, "deck": 0.6, "cabin_front": 0.3, "cabin_rear": -0.82, "roof_peak": 0.22, "tail": 0.12, "plan_n": 4.0, "paint": [0.30, 0.32, 0.35]},
        {"name": "supercar", "L": 4.62, "W": 2.02, "H": 1.16, "ride": 0.10, "wr": 0.35, "wb": 2.68, "wb_shift": -0.05, "belt": 0.6, "hood": 0.46, "deck": 0.66, "cabin_front": 0.36, "cabin_rear": -0.34, "roof_peak": 0.45, "tail": 0.5, "plan_n": 4.0, "paint": [0.42, 0.40, 0.38]},
        {"name": "hypercar", "L": 4.78, "W": 2.06, "H": 1.10, "ride": 0.09, "wr": 0.36, "wb": 2.76, "wb_shift": -0.08, "belt": 0.58, "hood": 0.42, "deck": 0.7, "cabin_front": 0.42, "cabin_rear": -0.22, "roof_peak": 0.5, "tail": 0.5, "plan_n": 3.5, "paint": [0.33, 0.35, 0.38]}
    ]);
    let params = json!({ "cars": cars, "sprite": {"w": 1024, "h": 768, "samples": 160}, "seed": 91 });
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        recipes::adas::CARS.as_bytes(),
        params.to_string().as_bytes(),
        b"post-v1",
    ]);
    ctx.job("adas/cars", &fp, |ctx| {
        let out = ctx.scratch("adas_cars")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "adas_cars", recipes::adas::CARS, &p)?;
        for c in cars.as_array().into_iter().flatten() {
            let name = c["name"].as_str().unwrap_or_default();
            let meta_path = out.join(format!("{name}.json"));
            let meta: serde_json::Value = serde_json::from_slice(&crate::ctx::read(&meta_path)?).map_err(|source| ForgeError::Json {
                context: meta_path.display().to_string(),
                source,
            })?;
            let tris = meta["tris"].as_u64().unwrap_or(u64::MAX);
            if tris > MAX_TRIS {
                return Err(ForgeError::Check { asset: name.to_owned(), message: format!("{tris} triangles > {MAX_TRIS}") });
            }
            let glb_rel = format!("assets/adas/{name}.glb");
            std::fs::create_dir_all(ctx.abs("assets/adas")).map_err(|e| crate::error::io_err(&ctx.abs("assets/adas"), e))?;
            std::fs::copy(out.join(format!("{name}.glb")), ctx.abs(&glb_rel)).map_err(|e| crate::error::io_err(&ctx.abs(&glb_rel), e))?;
            let sprite = img::load(&out.join(format!("{name}_sprite.png")))?;
            let gen = format!(
                "blender:adas::CARS {name} (lofted parametric sections L {} W {} H {} m, smooth by angle 38°) seed=91",
                c["L"], c["W"], c["H"]
            );
            ctx.file(
                &format!("adas/cars/{name}_glb"),
                &glb_rel,
                format!("{tris} tris"),
                Spec::new("adas", "mesh", gen.clone(), CLUSTERS).space("n/a").qa(format!(
                    "{tris} ≤ {MAX_TRIS} tris; Y up, nose +Z, ground y=0; dims {}; materials paint/glass/tyre/tail/head",
                    meta["dims"]
                )),
                Some(&sprite),
            )?;
            ctx.image(
                &format!("adas/cars/{name}_sprite"),
                &format!("assets/adas/{name}_sprite.png"),
                &sprite,
                Format::Png8,
                Spec::new("adas", "color", gen, CLUSTERS).alpha("straight").qa("rim-lit chase view, sprite fallback"),
            )?;
        }
        Ok(())
    })
}

/// The 4x4 road decal sheet (4096 units; white/neutral, tint in the shader).
pub fn decal_svg() -> String {
    let mut s = String::new();
    s.push_str(r##"<svg xmlns="http://www.w3.org/2000/svg" width="4096" height="4096" viewBox="0 0 4096 4096">"##);
    s.push_str(r##"<defs>
<linearGradient id="path" x1="0" x2="1" y1="0" y2="0"><stop offset="0" stop-color="#FFFFFF" stop-opacity="0"/><stop offset="0.18" stop-color="#FFFFFF" stop-opacity="0.35"/><stop offset="0.5" stop-color="#FFFFFF" stop-opacity="0.9"/><stop offset="0.82" stop-color="#FFFFFF" stop-opacity="0.35"/><stop offset="1" stop-color="#FFFFFF" stop-opacity="0"/></linearGradient>
<linearGradient id="fade" x1="0" x2="0" y1="0" y2="1"><stop offset="0" stop-color="#FFFFFF" stop-opacity="0"/><stop offset="0.3" stop-color="#FFFFFF" stop-opacity="1"/><stop offset="1" stop-color="#FFFFFF" stop-opacity="1"/></linearGradient>
<linearGradient id="edge" x1="0" x2="1" y1="0" y2="0"><stop offset="0" stop-color="#FFFFFF" stop-opacity="0"/><stop offset="0.8" stop-color="#FFFFFF" stop-opacity="0.5"/><stop offset="0.9" stop-color="#FFFFFF" stop-opacity="1"/><stop offset="1" stop-color="#FFFFFF" stop-opacity="0"/></linearGradient>
<radialGradient id="glow" cx="0.5" cy="0.5" r="0.5"><stop offset="0" stop-color="#FFFFFF" stop-opacity="0.85"/><stop offset="0.55" stop-color="#FFFFFF" stop-opacity="0.3"/><stop offset="1" stop-color="#FFFFFF" stop-opacity="0"/></radialGradient>
</defs>"##);
    let cell = |i: usize| ((i % 4) as f32 * 1024.0, (i / 4) as f32 * 1024.0);
    // 0: dashed lane line (3 m dash / 9 m gap scaled), vertical strip.
    let (x, y) = cell(0);
    for k in 0..4 {
        let _ = write!(
            s,
            r##"<rect x="{}" y="{}" width="48" height="160" rx="10" fill="#FFFFFF"/>"##,
            x + 488.0,
            y + 32.0 + k as f32 * 256.0
        );
    }
    // 1: solid line.
    let (x, y) = cell(1);
    let _ = write!(
        s,
        r##"<rect x="{}" y="{y}" width="48" height="1024" fill="#FFFFFF"/>"##,
        x + 488.0
    );
    // 2: double solid.
    let (x, y) = cell(2);
    let _ = write!(
        s,
        r##"<rect x="{}" y="{y}" width="36" height="1024" fill="#FFFFFF"/><rect x="{}" y="{y}" width="36" height="1024" fill="#FFFFFF"/>"##,
        x + 450.0,
        x + 538.0
    );
    // 3: dash + solid.
    let (x, y) = cell(3);
    let _ = write!(
        s,
        r##"<rect x="{}" y="{y}" width="36" height="1024" fill="#FFFFFF"/>"##,
        x + 538.0
    );
    for k in 0..4 {
        let _ = write!(
            s,
            r##"<rect x="{}" y="{}" width="36" height="160" rx="8" fill="#FFFFFF"/>"##,
            x + 450.0,
            y + 32.0 + k as f32 * 256.0
        );
    }
    // 4: chevrons.
    let (x, y) = cell(4);
    for k in 0..3 {
        let yy = y + 180.0 + k as f32 * 250.0;
        let _ = write!(
            s,
            r##"<path d="M{} {} L{} {yy} L{} {} L{} {} L{} {} L{} {} Z" fill="#FFFFFF"/>"##,
            x + 232.0,
            yy + 200.0,
            x + 512.0,
            x + 792.0,
            yy + 200.0,
            x + 792.0,
            yy + 280.0,
            x + 512.0,
            yy + 80.0,
            x + 232.0,
            yy + 280.0
        );
    }
    // 5: arrow straight.
    let (x, y) = cell(5);
    let _ = write!(
        s,
        r##"<path d="M{} {} V{} H{} L{} {} L{} {} H{} V{} Z" fill="#FFFFFF"/>"##,
        x + 462.0,
        y + 960.0,
        y + 420.0,
        x + 340.0,
        x + 512.0,
        y + 80.0,
        x + 684.0,
        y + 420.0,
        x + 562.0,
        y + 960.0
    );
    // 6: arrow left turn.
    let (x, y) = cell(6);
    let _ = write!(
        s,
        r##"<path d="M{} {} V{} Q{} {} {} {} H{} V{} L{} {} L{} {} V{} H{} Q{} {} {} {} V{} Z" fill="#FFFFFF"/>"##,
        x + 612.0,
        y + 960.0,
        y + 520.0,
        x + 612.0,
        y + 380.0,
        x + 472.0,
        y + 380.0,
        x + 400.0,
        y + 470.0,
        x + 140.0,
        y + 330.0,
        x + 400.0,
        y + 190.0,
        y + 280.0,
        x + 472.0,
        x + 712.0,
        y + 280.0,
        x + 712.0,
        y + 520.0,
        y + 960.0
    );
    // 7: arrow right turn (mirror of 6).
    let (x, y) = cell(7);
    let _ = write!(
        s,
        r##"<g transform="translate({} 0) scale(-1 1)"><path d="M{} {} V{} Q{} {} {} {} H{} V{} L{} {} L{} {} V{} H{} Q{} {} {} {} V{} Z" fill="#FFFFFF"/></g>"##,
        2.0 * x + 1024.0,
        x + 612.0,
        y + 960.0,
        y + 520.0,
        x + 612.0,
        y + 380.0,
        x + 472.0,
        y + 380.0,
        x + 400.0,
        y + 470.0,
        x + 140.0,
        y + 330.0,
        x + 400.0,
        y + 190.0,
        y + 280.0,
        x + 472.0,
        x + 712.0,
        y + 280.0,
        x + 712.0,
        y + 520.0,
        y + 960.0
    );
    // 8: crosswalk bars.
    let (x, y) = cell(8);
    for k in 0..6 {
        let _ = write!(
            s,
            r##"<rect x="{}" y="{}" width="96" height="784" fill="#FFFFFF"/>"##,
            x + 92.0 + k as f32 * 150.0,
            y + 120.0
        );
    }
    // 9: stop line.
    let (x, y) = cell(9);
    let _ = write!(
        s,
        r##"<rect x="{x}" y="{}" width="1024" height="120" fill="#FFFFFF"/>"##,
        y + 452.0
    );
    // 10: two-tone path ribbon (Tesla style), vertical, fades in with distance.
    let (x, y) = cell(10);
    let _ = write!(
        s,
        r##"<rect x="{}" y="{y}" width="624" height="1024" fill="url(#path)" mask="url(#m10)"/><mask id="m10"><rect x="{}" y="{y}" width="624" height="1024" fill="url(#fade)"/></mask>"##,
        x + 200.0,
        x + 200.0
    );
    // 11: road edge glow (gradient towards the edge).
    let (x, y) = cell(11);
    let _ = write!(
        s,
        r##"<rect x="{x}" y="{y}" width="1024" height="1024" fill="url(#edge)"/>"##
    );
    // 12: ego underglow ellipse.
    let (x, y) = cell(12);
    let _ = write!(
        s,
        r##"<ellipse cx="{}" cy="{}" rx="300" ry="470" fill="url(#glow)"/>"##,
        x + 512.0,
        y + 512.0
    );
    // 13: blind-spot proximity arcs (three bands, cyan → amber → crimson by tint).
    let (x, y) = cell(13);
    for (k, r) in [260.0f32, 360.0, 460.0].iter().enumerate() {
        let a0 = 200f32.to_radians();
        let a1 = 340f32.to_radians();
        let (cx, cy) = (x + 512.0, y + 620.0);
        let _ = write!(
            s,
            r##"<path d="M{:.1} {:.1} A{r} {r} 0 0 1 {:.1} {:.1}" stroke="#FFFFFF" stroke-width="{}" stroke-linecap="round" fill="none" opacity="{:.2}"/>"##,
            cx + r * a0.cos(),
            cy + r * a0.sin(),
            cx + r * a1.cos(),
            cy + r * a1.sin(),
            46.0 - k as f32 * 10.0,
            1.0 - k as f32 * 0.28
        );
    }
    // 14: lane fill gradient (between lines).
    let (x, y) = cell(14);
    let _ = write!(
        s,
        r##"<rect x="{x}" y="{y}" width="1024" height="1024" fill="url(#path)" opacity="0.35"/>"##
    );
    // 15: lead-car lock box (FSD).
    let (x, y) = cell(15);
    let (cx, cy, h, l) = (x + 512.0, y + 512.0, 330.0f32, 110.0f32);
    for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let (px, py) = (cx + sx * h, cy + sy * h);
        let _ = write!(
            s,
            r##"<path d="M{px} {} V{py} H{}" stroke="#FFFFFF" stroke-width="22" fill="none" stroke-linecap="round"/>"##,
            py - sy * l,
            px - sx * l
        );
    }
    s.push_str("</svg>\n");
    s
}

fn decals(ctx: &mut Ctx) -> Result<()> {
    let text = decal_svg();
    let fp = fingerprint(&[b"decals-v1", text.as_bytes()]);
    ctx.job("adas/decals", &fp, |ctx| {
        let rel_svg = "assets/adas/road_decals_4x4.svg";
        img::write_file(&ctx.abs(rel_svg), text.as_bytes())?;
        let opt = svg::options(&[]);
        let tree = svg::parse(text.as_bytes(), &opt, rel_svg)?;
        let png = svg::render_longest(&tree, 4096, rel_svg)?;
        let gen = "rust:sets::adas::decal_svg (dashed, solid, double, dash-solid, chevrons, 3 arrows, crosswalk, stop line, two-tone path ribbon, edge glow, underglow, blind-spot arcs, lane fill, lock box)";
        ctx.file("adas/decals/svg", rel_svg, "4096x4096 units".into(), Spec::new("adas", "vector", gen, CLUSTERS).alpha("straight"), Some(&png))?;
        ctx.image(
            "adas/decals/png",
            "assets/adas/road_decals_4x4.png",
            &png,
            Format::Png8,
            Spec::new("adas", "color", gen, CLUSTERS).alpha("straight").qa("4x4 cells of 1024 px, white; tint per semantic colour"),
        )
    })
}
