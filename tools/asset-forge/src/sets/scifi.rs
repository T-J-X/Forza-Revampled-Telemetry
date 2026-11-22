//! Sci-fi and out-of-this-world set: holographic grids (Blender perspective plate plus exact
//! tileable masks), a looping warp tunnel (Blender), HUD reticle sheets (vector SVG made here,
//! rasterised with resvg), scanline, VFD mesh and phosphor masks.
use crate::ctx::{Ctx, Format, Spec};
use crate::error::Result;
use crate::img::{self, smoothstep, Img};
use crate::ledger::{fingerprint, Flipbook};
use crate::procedural as pr;
use crate::{blender, recipes, svg};
use serde_json::json;
use std::fmt::Write as _;

const CLUSTERS: &str = "sci-fi family (RetroCyber, SynthwaveOutrun, FUI concepts), WindshieldHud, TeslaAdas";

/// Builds every sci-fi job.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    holo_grid(ctx)?;
    warp(ctx)?;
    masks(ctx)?;
    reticles(ctx)?;
    Ok(())
}

fn holo_grid(ctx: &mut Ctx) -> Result<()> {
    let params = json!({ "w": 4096, "h": 2048, "samples": 160, "seed": 81 });
    let body = format!("{}\n{}", recipes::classic::EXPR_HELPERS, recipes::scifi::HOLO_GRID);
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        body.as_bytes(),
        params.to_string().as_bytes(),
        b"post-v1",
    ]);
    ctx.job("scifi/holo_grid", &fp, |ctx| {
        let out = ctx.scratch("scifi_holo")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "scifi_holo", &body, &p)?;
        let g = img::load(&out.join("holo_grid.png"))?.to_channels(3);
        ctx.image(
            "scifi/holo_grid/plate",
            "assets/scifi/holo_floor_grid.png",
            &g,
            Format::Png8,
            Spec::new(
                "scifi",
                "color",
                "blender:scifi::HOLO_GRID (emissive grid, ray-length fade, 24 mm lens) seed=81",
                CLUSTERS,
            )
            .alpha("additive"),
        )
    })
}

fn warp(ctx: &mut Ctx) -> Result<()> {
    let params = json!({ "frames": 32, "cell": 512, "samples": 64, "seed": 82 });
    let body = format!("{}\n{}", recipes::classic::EXPR_HELPERS, recipes::scifi::WARP_TUNNEL);
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        body.as_bytes(),
        params.to_string().as_bytes(),
        b"post-v1",
    ]);
    ctx.job("scifi/warp", &fp, |ctx| {
        let out = ctx.scratch("scifi_warp")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "scifi_warp", &body, &p)?;
        let frames: Vec<Img> = (0..32)
            .map(|f| img::load(&out.join(format!("warp_{f:04}.png"))).map(|i| i.to_channels(3)))
            .collect::<Result<_>>()?;
        let score = img::loop_seam_score(&frames);
        ctx.image(
            "scifi/warp/tunnel",
            "assets/scifi/warp_tunnel_8x4.png",
            &img::atlas(&frames, 8),
            Format::Png8,
            Spec::new(
                "scifi",
                "flipbook",
                "blender:scifi::WARP_TUNNEL (emissive tube, torus-periodic streaks scrolled one period) seed=82",
                CLUSTERS,
            )
            .alpha("additive")
            .flip(Flipbook {
                cols: 8,
                rows: 4,
                frames: 32,
                fps: 30,
                cell: 512,
            })
            .qa(format!("loop seam score {score:.2}")),
        )
    })
}

/// Anti-aliased line coverage for distance `d` px from a line of half-width `hw`.
fn cov(d: f32, hw: f32) -> f32 {
    1.0 - smoothstep(hw - 0.75, hw + 0.75, d)
}

fn masks(ctx: &mut Ctx) -> Result<()> {
    let fp = fingerprint(&[
        b"scifi-masks-v1",
        include_bytes!("scifi.rs"),
        include_bytes!("../procedural.rs"),
    ]);
    ctx.job("scifi/masks", &fp, |ctx| {
        let gen_grid = "rust:sets::scifi::masks (exact tileable lattice, analytic AA)";
        // Square grid: minor every 64 px, major every 256 px.
        let grid = Img::from_fn(2048, 2048, 1, |x, y| {
            let line = |p: f32, period: f32| {
                let m = (p + 0.5).rem_euclid(period);
                m.min(period - m)
            };
            let (fx, fy) = (x as f32, y as f32);
            let minor = cov(line(fx, 64.0).min(line(fy, 64.0)), 1.0) * 0.45;
            let major = cov(line(fx, 256.0).min(line(fy, 256.0)), 1.6);
            [minor.max(major), 0.0, 0.0, 0.0]
        });
        ctx.image(
            "scifi/masks/grid_square",
            "assets/scifi/grid_square_2k.png",
            &grid,
            Format::Png8,
            Spec::new("scifi", "mask", gen_grid, CLUSTERS)
                .alpha("additive")
                .tile()
                .space("linear")
                .qa(format!("tile seam score {:.2}", img::tile_seam_score(&grid))),
        )?;
        // Dot matrix (Oblivion-style), 32 px pitch.
        let dots = Img::from_fn(2048, 2048, 1, |x, y| {
            let c = |p: usize| ((p % 32) as f32 + 0.5 - 16.0).abs();
            let d = (c(x).powi(2) + c(y).powi(2)).sqrt();
            let big = (x / 32) % 8 == 0 && (y / 32) % 8 == 0;
            [cov(d, if big { 3.2 } else { 1.8 }), 0.0, 0.0, 0.0]
        });
        ctx.image(
            "scifi/masks/grid_dots",
            "assets/scifi/grid_dots_2k.png",
            &dots,
            Format::Png8,
            Spec::new("scifi", "mask", gen_grid, CLUSTERS)
                .alpha("additive")
                .tile()
                .space("linear"),
        )?;
        // 45° crosshatch, 64 px pitch along the diagonal (2048 / 64 integer so it tiles).
        let hatch = Img::from_fn(2048, 2048, 1, |x, y| {
            let a = ((x + y) % 64) as f32;
            let b = ((x + 2048 - y) % 64) as f32;
            let d1 = a.min(64.0 - a) / std::f32::consts::SQRT_2;
            let d2 = b.min(64.0 - b) / std::f32::consts::SQRT_2;
            [cov(d1.min(d2), 0.9) * 0.6, 0.0, 0.0, 0.0]
        });
        ctx.image(
            "scifi/masks/grid_hatch",
            "assets/scifi/grid_hatch_2k.png",
            &hatch,
            Format::Png8,
            Spec::new("scifi", "mask", gen_grid, CLUSTERS)
                .alpha("additive")
                .tile()
                .space("linear"),
        )?;
        let scan = pr::scanlines(2, 64);
        ctx.image(
            "scifi/masks/scanline_2px",
            "assets/scifi/scanline_2px.png",
            &scan,
            Format::Png8,
            Spec::new(
                "scifi",
                "mask",
                "rust:procedural::scanlines (2 px pitch)",
                "RetroCyber VFD cavity, sci-fi family",
            )
            .alpha("none")
            .tile()
            .space("linear")
            .qa("multiply; repeat"),
        )?;
        let scan3 = pr::scanlines(3, 96);
        ctx.image(
            "scifi/masks/scanline_3px",
            "assets/scifi/scanline_3px.png",
            &scan3,
            Format::Png8,
            Spec::new(
                "scifi",
                "mask",
                "rust:procedural::scanlines (3 px pitch)",
                "sci-fi family",
            )
            .alpha("none")
            .tile()
            .space("linear")
            .qa("multiply; repeat"),
        )?;
        let mesh = pr::vfd_mesh(16, 9.0);
        ctx.image(
            "scifi/masks/vfd_mesh",
            "assets/scifi/vfd_hex_mesh.png",
            &mesh,
            Format::Png8,
            Spec::new(
                "scifi",
                "mask",
                "rust:procedural::vfd_mesh (flat-top hex wire, 9 px)",
                "RetroCyber, VFD members of the sci-fi family",
            )
            .alpha("none")
            .tile()
            .space("linear")
            .qa(format!("multiply; tile seam score {:.2}", img::tile_seam_score(&mesh))),
        )?;
        let triad = pr::phosphor_triad(3);
        ctx.image(
            "scifi/masks/phosphor_triad",
            "assets/scifi/phosphor_triad.png",
            &triad,
            Format::Png8,
            Spec::new(
                "scifi",
                "mask",
                "rust:procedural::phosphor_triad (aperture-grille RGB stripes, 3 px)",
                "SynthwaveOutrun CRT look",
            )
            .alpha("none")
            .tile()
            .space("linear")
            .qa("multiply; repeat"),
        )?;
        Ok(())
    })
}

/// The 16-cell HUD reticle sheet as SVG (4096 units, white strokes on transparent).
pub fn reticle_svg() -> String {
    let mut s = String::new();
    s.push_str(r##"<svg xmlns="http://www.w3.org/2000/svg" width="4096" height="4096" viewBox="0 0 4096 4096" fill="none" stroke="#FFFFFF" stroke-linecap="round" stroke-linejoin="round">"##);
    let cell = |i: usize| ((i % 4) as f32 * 1024.0 + 512.0, (i / 4) as f32 * 1024.0 + 512.0);
    let pol = |cx: f32, cy: f32, r: f32, deg: f32| {
        let a = deg.to_radians();
        (cx + r * a.cos(), cy - r * a.sin())
    };
    let arc = |cx: f32, cy: f32, r: f32, a0: f32, a1: f32| {
        let (x0, y0) = pol(cx, cy, r, a0);
        let (x1, y1) = pol(cx, cy, r, a1);
        let large = i32::from((a1 - a0).abs() > 180.0);
        format!("M{x0:.1} {y0:.1} A{r} {r} 0 {large} 0 {x1:.1} {y1:.1}")
    };
    // 0: fighter gear reticle — ring with 10° ticks and a broken cross.
    let (cx, cy) = cell(0);
    let _ = write!(s, r##"<circle cx="{cx}" cy="{cy}" r="380" stroke-width="10"/>"##);
    for k in 0..36 {
        let len = if k % 9 == 0 {
            70.0
        } else if k % 3 == 0 {
            44.0
        } else {
            24.0
        };
        let (x0, y0) = pol(cx, cy, 380.0, k as f32 * 10.0);
        let (x1, y1) = pol(cx, cy, 380.0 - len, k as f32 * 10.0);
        let _ = write!(
            s,
            r##"<line x1="{x0:.1}" y1="{y0:.1}" x2="{x1:.1}" y2="{y1:.1}" stroke-width="8"/>"##
        );
    }
    let _ = write!(
        s,
        r##"<path d="M{} {cy} H{} M{} {cy} H{} M{cx} {} V{} M{cx} {} V{}" stroke-width="10"/>"##,
        cx - 220.0,
        cx - 70.0,
        cx + 70.0,
        cx + 220.0,
        cy - 220.0,
        cy - 70.0,
        cy + 70.0,
        cy + 220.0
    );
    // 1: FSD lead-car lock box — corner brackets.
    let (cx, cy) = cell(1);
    let (h, l) = (330.0, 110.0);
    for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let (x, y) = (cx + sx * h, cy + sy * h);
        let _ = write!(
            s,
            r##"<path d="M{x} {} V{y} H{}" stroke-width="16"/>"##,
            y - sy * l,
            x - sx * l
        );
    }
    // 2: chevron stack.
    let (cx, cy) = cell(2);
    for k in 0..4 {
        let y = cy - 240.0 + k as f32 * 150.0;
        let _ = write!(
            s,
            r##"<path d="M{} {} L{cx} {y} L{} {}" stroke-width="22" opacity="{:.2}"/>"##,
            cx - 220.0,
            y + 110.0,
            cx + 220.0,
            y + 110.0,
            1.0 - k as f32 * 0.2
        );
    }
    // 3: range rings with ticks.
    let (cx, cy) = cell(3);
    for r in [140.0, 260.0, 380.0] {
        let _ = write!(
            s,
            r##"<circle cx="{cx}" cy="{cy}" r="{r}" stroke-width="6" stroke-dasharray="{} {}"/>"##,
            r * 0.12,
            r * 0.06
        );
    }
    // 4: pitch ladder segment.
    let (cx, cy) = cell(4);
    for (k, deg) in [-10, -5, 0, 5, 10].iter().enumerate() {
        let y = cy + 300.0 - k as f32 * 150.0;
        let dash = if *deg < 0 { r##" stroke-dasharray="40 26""## } else { "" };
        let w = if *deg == 0 { 420.0 } else { 300.0 };
        let _ = write!(
            s,
            r##"<path d="M{} {y} H{} M{} {y} H{}" stroke-width="10"{dash}/>"##,
            cx - w,
            cx - 90.0,
            cx + 90.0,
            cx + w
        );
    }
    // 5: waterline (boresight W).
    let (cx, cy) = cell(5);
    let _ = write!(
        s,
        r##"<path d="M{} {cy} H{} L{} {} L{cx} {} L{} {} L{} {cy} H{}" stroke-width="14"/>"##,
        cx - 360.0,
        cx - 160.0,
        cx - 80.0,
        cy + 90.0,
        cy,
        cx + 80.0,
        cy + 90.0,
        cx + 160.0,
        cx + 360.0
    );
    // 6: hex target.
    let (cx, cy) = cell(6);
    for r in [360.0, 200.0] {
        let pts: Vec<String> = (0..6)
            .map(|k| {
                let (x, y) = pol(cx, cy, r, 30.0 + k as f32 * 60.0);
                format!("{x:.1},{y:.1}")
            })
            .collect();
        let _ = write!(s, r##"<polygon points="{}" stroke-width="10"/>"##, pts.join(" "));
    }
    let _ = write!(
        s,
        r##"<circle cx="{cx}" cy="{cy}" r="16" fill="#FFFFFF" stroke="none"/>"##
    );
    // 7: 24-segment 270° arc gauge.
    let (cx, cy) = cell(7);
    for k in 0..24 {
        let a0 = 225.0 - k as f32 * 11.25 - 1.6;
        let a1 = a0 - 7.6;
        let _ = write!(
            s,
            r##"<path d="{}" stroke-width="44" stroke-linecap="butt"/>"##,
            arc(cx, cy, 360.0, a1, a0)
        );
    }
    // 8: crosshair with dots.
    let (cx, cy) = cell(8);
    let _ = write!(
        s,
        r##"<path d="M{} {cy} H{} M{cx} {} V{}" stroke-width="6"/>"##,
        cx - 400.0,
        cx + 400.0,
        cy - 400.0,
        cy + 400.0
    );
    for k in 1..=7 {
        let d = k as f32 * 55.0;
        for (x, y) in [(cx + d, cy), (cx - d, cy), (cx, cy + d), (cx, cy - d)] {
            let _ = write!(s, r##"<circle cx="{x}" cy="{y}" r="9" fill="#FFFFFF" stroke="none"/>"##);
        }
    }
    // 9: radar sweep sector.
    let (cx, cy) = cell(9);
    let _ = write!(
        s,
        r##"<path d="M{cx} {cy} L{:.1} {:.1} {} Z" stroke-width="8"/>"##,
        pol(cx, cy, 400.0, 60.0).0,
        pol(cx, cy, 400.0, 60.0).1,
        arc(cx, cy, 400.0, 60.0, 120.0).replacen('M', "L", 1)
    );
    let _ = write!(
        s,
        r##"<path d="{}" stroke-width="5" opacity="0.6"/><path d="{}" stroke-width="5" opacity="0.6"/>"##,
        arc(cx, cy, 260.0, 60.0, 120.0),
        arc(cx, cy, 130.0, 60.0, 120.0)
    );
    // 10: diamond lock.
    let (cx, cy) = cell(10);
    let _ = write!(
        s,
        r##"<path d="M{cx} {} L{} {cy} L{cx} {} L{} {cy} Z" stroke-width="14"/>"##,
        cy - 330.0,
        cx + 330.0,
        cy + 330.0,
        cx - 330.0
    );
    let _ = write!(
        s,
        r##"<path d="M{cx} {} L{} {cy} L{cx} {} L{} {cy} Z" stroke-width="6" opacity="0.6"/>"##,
        cy - 190.0,
        cx + 190.0,
        cy + 190.0,
        cx - 190.0
    );
    // 11: bracket pair.
    let (cx, cy) = cell(11);
    let _ = write!(
        s,
        r##"<path d="M{} {} H{} V{} H{} M{} {} H{} V{} H{}" stroke-width="18"/>"##,
        cx - 240.0,
        cy - 300.0,
        cx - 340.0,
        cy + 300.0,
        cx - 240.0,
        cx + 240.0,
        cy - 300.0,
        cx + 340.0,
        cy + 300.0,
        cx + 240.0
    );
    // 12: compass tape.
    let (cx, cy) = cell(12);
    let _ = write!(
        s,
        r##"<path d="M{} {} H{}" stroke-width="6"/>"##,
        cx - 460.0,
        cy + 60.0,
        cx + 460.0
    );
    for k in -18..=18 {
        let x = cx + k as f32 * 25.0;
        let len = if k % 6 == 0 {
            120.0
        } else if k % 2 == 0 {
            70.0
        } else {
            40.0
        };
        let _ = write!(
            s,
            r##"<path d="M{x} {} V{}" stroke-width="6"/>"##,
            cy + 60.0,
            cy + 60.0 - len
        );
    }
    let _ = write!(
        s,
        r##"<path d="M{} {} L{cx} {} L{} {} Z" fill="#FFFFFF" stroke="none"/>"##,
        cx - 34.0,
        cy + 160.0,
        cy + 100.0,
        cx + 34.0,
        cy + 160.0
    );
    // 13: triangle carets.
    let (cx, cy) = cell(13);
    for (k, x) in [-240.0f32, 0.0, 240.0].iter().enumerate() {
        let fill = if k == 1 { "#FFFFFF" } else { "none" };
        let _ = write!(
            s,
            r##"<path d="M{} {} L{} {} L{} {} Z" stroke-width="12" fill="{fill}"/>"##,
            cx + x - 80.0,
            cy - 60.0,
            cx + x + 80.0,
            cy - 60.0,
            cx + x,
            cy + 80.0
        );
    }
    // 14: octagonal gear bay (RetroCyber [ 4 ]).
    let (cx, cy) = cell(14);
    let pts: Vec<String> = (0..8)
        .map(|k| {
            let (x, y) = pol(cx, cy, 400.0, 22.5 + k as f32 * 45.0);
            format!("{x:.1},{y:.1}")
        })
        .collect();
    let _ = write!(s, r##"<polygon points="{}" stroke-width="14"/>"##, pts.join(" "));
    let pts: Vec<String> = (0..8)
        .map(|k| {
            let (x, y) = pol(cx, cy, 340.0, 22.5 + k as f32 * 45.0);
            format!("{x:.1},{y:.1}")
        })
        .collect();
    let _ = write!(
        s,
        r##"<polygon points="{}" stroke-width="5" opacity="0.55"/>"##,
        pts.join(" ")
    );
    // 15: dot ring (60 dots, quarter marks large).
    let (cx, cy) = cell(15);
    for k in 0..60 {
        let (x, y) = pol(cx, cy, 380.0, k as f32 * 6.0);
        let r = if k % 15 == 0 {
            22.0
        } else if k % 5 == 0 {
            14.0
        } else {
            8.0
        };
        let _ = write!(
            s,
            r##"<circle cx="{x:.1}" cy="{y:.1}" r="{r}" fill="#FFFFFF" stroke="none"/>"##
        );
    }
    s.push_str("</svg>\n");
    s
}

fn reticles(ctx: &mut Ctx) -> Result<()> {
    let text = reticle_svg();
    let fp = fingerprint(&[b"reticles-v1", text.as_bytes()]);
    ctx.job("scifi/reticles", &fp, |ctx| {
        let rel_svg = "assets/scifi/hud_reticles_4x4.svg";
        img::write_file(&ctx.abs(rel_svg), text.as_bytes())?;
        let opt = svg::options(&[]);
        let tree = svg::parse(text.as_bytes(), &opt, rel_svg)?;
        let png = svg::render_longest(&tree, 4096, rel_svg)?;
        let gen = "rust:sets::scifi::reticle_svg (16 vector HUD symbols: gear reticle, FSD lock box, chevrons, range rings, pitch ladder, waterline, hex target, 24-seg arc, crosshair, radar sector, diamond lock, brackets, compass tape, carets, octagon gear bay, dot ring)";
        ctx.file("scifi/reticles/svg", rel_svg, "4096x4096 units".into(), Spec::new("scifi", "vector", gen, CLUSTERS).alpha("straight").space("srgb"), Some(&png))?;
        ctx.image(
            "scifi/reticles/png",
            "assets/scifi/hud_reticles_4x4.png",
            &png,
            Format::Png8,
            Spec::new("scifi", "color", gen, CLUSTERS).alpha("straight").qa("4x4 cells of 1024 px; white, tint in the shader"),
        )
    })
}
