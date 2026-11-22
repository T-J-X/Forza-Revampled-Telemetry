//! Liquid Glass set: plate normals (Blender geometry), refraction offsets and fresnel rims
//! (derived in Rust), droplets and meniscus (Blender metaballs), the Cycles caustic lens,
//! tileable looping caustics (Rust ray tracer), frosted micro-noise, blue-noise dither and the
//! arched sheen.
use crate::ctx::{Ctx, Format, Spec};
use crate::error::Result;
use crate::img::{self, Img};
use crate::ktx2w::Encoding;
use crate::ledger::{fingerprint, Flipbook};
use crate::procedural as pr;
use crate::{blender, recipes};
use serde_json::json;

const CLUSTERS: &str = "LiquidGlass, OSD glass, Liquid Glass family";

/// Builds every glass job.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    plates(ctx)?;
    liquid(ctx)?;
    caustic_lens(ctx)?;
    caustic_tiles(ctx)?;
    frost_and_dither(ctx)?;
    sheen(ctx)?;
    Ok(())
}

/// Unpacks a data-pass EXR: returns (normal RGB + coverage A, height, coverage).
pub fn unpack_data(exr: &Img) -> (Img, Img) {
    let mut normal = Img::new(exr.w, exr.h, 4);
    let mut height = Img::new(exr.w, exr.h, 1);
    for y in 0..exr.h {
        for x in 0..exr.w {
            let a = exr.get(x, y, 3).clamp(0.0, 1.0);
            let (nx, ny, hz) = if a > 1e-4 {
                (
                    (exr.get(x, y, 0) / a) * 2.0 - 1.0,
                    (exr.get(x, y, 1) / a) * 2.0 - 1.0,
                    exr.get(x, y, 2) / a,
                )
            } else {
                (0.0, 0.0, 0.0)
            };
            // Blend partially covered edge pixels towards flat so the silhouette stays clean.
            let nx = nx * a;
            let ny = ny * a;
            let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
            normal.set(x, y, 0, nx * 0.5 + 0.5);
            normal.set(x, y, 1, ny * 0.5 + 0.5);
            normal.set(x, y, 2, nz * 0.5 + 0.5);
            normal.set(x, y, 3, a);
            height.set(x, y, 0, hz.clamp(0.0, 1.0) * a);
        }
    }
    (normal, height)
}

/// Refraction offsets for a view straight down through glass of index `ior`: RG = offset
/// (0.5-centred, ±`max_px` at scale 1), B = thickness 0..1, A = coverage.
fn refraction(normal: &Img, height: &Img, ior: f32, thick_px: f32) -> (Img, f32) {
    let eta = 1.0 / ior;
    let mut offs = Vec::with_capacity(normal.w * normal.h);
    let mut max_px = 1e-3f32;
    for y in 0..normal.h {
        for x in 0..normal.w {
            let n = [
                normal.get(x, y, 0) * 2.0 - 1.0,
                normal.get(x, y, 1) * 2.0 - 1.0,
                normal.get(x, y, 2) * 2.0 - 1.0,
            ];
            // incident (0,0,-1); refract(I, N, eta)
            let cosi = n[2];
            let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
            let t = if k < 0.0 {
                [0.0, 0.0, -1.0]
            } else {
                let a = eta * cosi - k.sqrt();
                [a * n[0], a * n[1], -eta + a * n[2]]
            };
            let depth = (0.25 + height.get(x, y, 0)) * thick_px;
            let tz = t[2].abs().max(0.2);
            let ox = t[0] / tz * depth;
            let oy = t[1] / tz * depth;
            max_px = max_px.max(ox.abs()).max(oy.abs());
            offs.push((ox, oy));
        }
    }
    let img = Img::from_fn(normal.w, normal.h, 4, |x, y| {
        let (ox, oy) = offs[y * normal.w + x];
        [
            ox / max_px * 0.5 + 0.5,
            oy / max_px * 0.5 + 0.5,
            height.get(x, y, 0),
            normal.get(x, y, 3),
        ]
    });
    (img, max_px)
}

/// R = Schlick fresnel (F0 0.04), G = rim band (where the bevel turns), A = coverage.
fn fresnel(normal: &Img) -> Img {
    Img::from_fn(normal.w, normal.h, 4, |x, y| {
        let nz = (normal.get(x, y, 2) * 2.0 - 1.0).clamp(0.0, 1.0);
        let f = 0.04 + 0.96 * (1.0 - nz).powi(5);
        let rim = img::smoothstep(0.004, 0.35, 1.0 - nz);
        [f, rim, 0.0, normal.get(x, y, 3)]
    })
}

struct Shape {
    name: &'static str,
    kind: &'static str,
    px: (usize, usize),
    r: f32,
    bevel: f32,
    thick: f32,
    profile: f32,
    note: &'static str,
}

const SHAPES: &[Shape] = &[
    Shape { name: "plate_rrect", kind: "rrect", px: (4096, 2048), r: 0.48, bevel: 0.16, thick: 0.2, profile: 0.62, note: "rounded rect 2:1, corner 480 px, bevel 160 px" },
    Shape { name: "plate_pill", kind: "pill", px: (4096, 1024), r: 0.0, bevel: 0.14, thick: 0.18, profile: 0.6, note: "pill 4:1, bevel 140 px" },
    Shape { name: "plate_squircle", kind: "squircle", px: (4096, 4096), r: 0.0, bevel: 0.24, thick: 0.28, profile: 0.6, note: "superellipse n=5 (continuous corners), bevel 240 px" },
    Shape { name: "lens_circle", kind: "circle", px: (4096, 4096), r: 0.0, bevel: 0.34, thick: 0.4, profile: 0.55, note: "round dial lens, bevel 340 px" },
];

fn plates(ctx: &mut Ctx) -> Result<()> {
    let margin = 1.02f32;
    let shapes: Vec<serde_json::Value> = SHAPES
        .iter()
        .map(|s| {
            let w = s.px.0 as f32 / 1000.0 / margin;
            let h = s.px.1 as f32 / 1000.0 / margin;
            json!({"name": s.name, "kind": s.kind, "w": w, "h": h, "r": s.r, "bevel": s.bevel,
                   "thick": s.thick, "profile": s.profile, "n": 5.0, "px_w": s.px.0, "px_h": s.px.1, "margin": margin})
        })
        .collect();
    let params = json!({ "shapes": shapes, "seed": 0 });
    let fp = fingerprint(&[recipes::PRELUDE.as_bytes(), recipes::glass::PLATES.as_bytes(), params.to_string().as_bytes(), b"post-v3"]);
    ctx.job("glass/plates", &fp, |ctx| {
        let out = ctx.scratch("glass_plates")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "glass_plates", recipes::glass::PLATES, &p)?;
        for s in SHAPES {
            let exr = img::load_exr(&out.join(format!("{}.exr", s.name)))?;
            let (normal, height) = unpack_data(&exr);
            let gen = format!("blender:glass::PLATES ({}) seed=0", s.note);
            let base = format!("assets/glass/{}", s.name);
            ctx.image(
                &format!("glass/plates/{}_normal16", s.name),
                &format!("{base}_normal.png"),
                &normal,
                Format::Png16,
                Spec::new("glass", "normal", gen.clone(), CLUSTERS).alpha("straight").qa("A = coverage; 16-bit master"),
            )?;
            ctx.image(
                &format!("glass/plates/{}_normal", s.name),
                &format!("{base}_normal.ktx2"),
                &normal,
                Format::Ktx2(Encoding::Normal),
                Spec::new("glass", "normal", gen.clone(), CLUSTERS).alpha("straight").qa("RGBA8 UNORM + mips; A = coverage"),
            )?;
            let thick_px = s.thick * 1000.0;
            // Offsets and fresnel are smooth: ship them at half size.
            let (hw, hh) = (normal.w / 2, normal.h / 2);
            let mut half_n = normal.resize(hw, hh, false);
            img::renormalize(&mut half_n);
            let half_h = height.resize(hw, hh, false);
            let (refr, max_px) = refraction(&half_n, &half_h, 1.5, thick_px / 2.0);
            let max_px = max_px * 2.0;
            ctx.image(
                &format!("glass/plates/{}_refraction", s.name),
                &format!("{base}_refraction.ktx2"),
                &refr,
                Format::Ktx2(Encoding::Linear),
                Spec::new("glass", "offset", gen.clone(), CLUSTERS)
                    .alpha("straight")
                    .qa(format!("IOR 1.5; RG = offset 0.5-centred, ±{max_px:.0} px at 4K scale 1; B = thickness; A = coverage")),
            )?;
            ctx.image(
                &format!("glass/plates/{}_fresnel", s.name),
                &format!("{base}_fresnel.ktx2"),
                &fresnel(&half_n),
                Format::Ktx2(Encoding::Linear),
                Spec::new("glass", "mask", gen, CLUSTERS).alpha("straight").qa("R = Schlick F (F0 0.04), G = rim band, A = coverage"),
            )?;
        }
        Ok(())
    })
}

fn liquid(ctx: &mut Ctx) -> Result<()> {
    let params = json!({ "seed": 11, "drops": 150 });
    let fp = fingerprint(&[recipes::PRELUDE.as_bytes(), recipes::glass::LIQUID.as_bytes(), params.to_string().as_bytes(), b"post-v2"]);
    ctx.job("glass/liquid", &fp, |ctx| {
        let out = ctx.scratch("glass_liquid")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "glass_liquid", recipes::glass::LIQUID, &p)?;
        let items: [(&str, bool, &str); 4] = [
            ("droplet_single", false, "single droplet sprite, contact-angle rim"),
            ("droplets_tile", true, "seeded droplet scatter, wrapped across edges"),
            ("meniscus_ring", false, "liquid in a ring channel climbing both walls (radial meniscus)"),
            ("liquid_capsule", false, "capsule blob for ribbon ends"),
        ];
        for (name, tile, note) in items {
            let exr = img::load_exr(&out.join(format!("{name}.exr")))?;
            let (normal, height) = unpack_data(&exr);
            let gen = format!("blender:glass::LIQUID ({note}) seed=11");
            let mut spec = Spec::new("glass", "normal", gen.clone(), "LiquidGlass ribbon tach, meniscus orb, fluid family").alpha("straight");
            if tile {
                spec = spec.tile().qa(format!("tile seam score {:.2}", img::tile_seam_score(&normal)));
            }
            ctx.image(&format!("glass/liquid/{name}_normal"), &format!("assets/glass/{name}_normal.ktx2"), &normal, Format::Ktx2(Encoding::Normal), spec.qa("A = coverage"))?;
            let mut hspec = Spec::new("glass", "data", gen, "LiquidGlass, fluid family").alpha("none");
            if tile {
                hspec = hspec.tile();
            }
            ctx.image(&format!("glass/liquid/{name}_height"), &format!("assets/glass/{name}_height.ktx2"), &height, Format::Ktx2(Encoding::Linear), hspec.qa("R = height 0..1"))?;
        }
        Ok(())
    })
}

/// Light under a glass plate: parallel rays refracted by the plate's normal map land `depth_px`
/// below; returns relative irradiance (1 = unobstructed light).
fn lens_caustic(normal: &Img, ior: f32, depth_px: f32, oversample: usize) -> Img {
    let (w, h) = (normal.w, normal.h);
    let mut acc = vec![0.0f32; w * h];
    let eta = 1.0 / ior;
    let step = 1.0 / oversample as f32;
    for j in 0..h * oversample {
        for i in 0..w * oversample {
            let x = (i as f32 + 0.5) * step;
            let y = (j as f32 + 0.5) * step;
            let nx = normal.sample(x, y, 0, false) * 2.0 - 1.0;
            let ny = normal.sample(x, y, 1, false) * 2.0 - 1.0;
            let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
            let k = 1.0 - eta * eta * (1.0 - nz * nz);
            let (tx, ty, tz) = if k < 0.0 {
                (0.0, 0.0, -1.0)
            } else {
                let a = eta * nz - k.sqrt();
                (a * nx, a * ny, -eta + a * nz)
            };
            // Image y grows downwards; normal-map green points up.
            let px = x + tx / tz.abs().max(0.2) * depth_px;
            let py = y - ty / tz.abs().max(0.2) * depth_px;
            let x0 = (px - 0.5).floor();
            let y0 = (py - 0.5).floor();
            let fx = px - 0.5 - x0;
            let fy = py - 0.5 - y0;
            let (x0, y0) = (x0 as isize, y0 as isize);
            for (dx, dy, wgt) in [(0, 0, (1.0 - fx) * (1.0 - fy)), (1, 0, fx * (1.0 - fy)), (0, 1, (1.0 - fx) * fy), (1, 1, fx * fy)] {
                let (xx, yy) = (x0 + dx, y0 + dy);
                if xx >= 0 && yy >= 0 && (xx as usize) < w && (yy as usize) < h {
                    acc[yy as usize * w + xx as usize] += wgt;
                }
            }
        }
    }
    let per = (oversample * oversample) as f32;
    Img { w, h, c: 1, data: acc.into_iter().map(|a| a / per).collect() }
}

fn caustic_lens(ctx: &mut Ctx) -> Result<()> {
    let sources = [("plate_rrect", 2048usize, 1024usize), ("lens_circle", 2048, 2048)];
    let mut parts: Vec<Vec<u8>> = vec![b"lens-caustic-v3".to_vec()];
    for (name, _, _) in sources {
        let p = ctx.abs(&format!("assets/glass/{name}_normal.png"));
        parts.push(std::fs::metadata(&p).map(|m| m.len().to_le_bytes().to_vec()).unwrap_or_default());
    }
    let refs: Vec<&[u8]> = parts.iter().map(Vec::as_slice).collect();
    let fp = fingerprint(&refs);
    ctx.job("glass/caustic_lens", &fp, |ctx| {
        for (name, w, h) in sources {
            let src = img::load(&ctx.abs(&format!("assets/glass/{name}_normal.png")))?.resize(w, h, false);
            let irr = lens_caustic(&src, 1.5, w as f32 * 0.09, 3);
            let irr = pr::blur(&irr, 2.2, false);
            // Only light above the unobstructed level is kept: OLED black elsewhere.
            let tone = Img::from_fn(w, h, 1, |x, y| {
                let v = (irr.get(x, y, 0) - 1.0).max(0.0);
                [img::srgb_encode(1.0 - (-1.4 * v).exp()), 0.0, 0.0, 0.0]
            });
            let out = name.replace("plate_", "caustic_lens_").replace("lens_circle", "caustic_lens_circle");
            ctx.image(
                &format!("glass/caustic_lens/{out}"),
                &format!("assets/glass/{out}.png"),
                &tone,
                Format::Png8,
                Spec::new(
                    "glass",
                    "color",
                    format!("rust:sets::glass::lens_caustic (rays refracted through the Blender {name} normals, IOR 1.5, receiver 9% of width below)"),
                    CLUSTERS,
                )
                .alpha("additive")
                .qa("light focused above the unobstructed level; everything else #000"),
            )?;
        }
        Ok(())
    })
}

fn caustic_tiles(ctx: &mut Ctx) -> Result<()> {
    let fp = fingerprint(&[b"caustic-v2", include_bytes!("../procedural.rs")]);
    ctx.job("glass/caustics", &fp, |ctx| {
        let gen = "rust:procedural::caustic (periodic wave surface, refracted parallel rays, wrap splat) seed=21";
        let still = pr::caustic(2048, 0.0, 21, 0.22, 3);
        let still = pr::blur(&still, 0.8, true);
        let tone = pr::caustic_tone(&still, 2.4);
        let enc = Img::from_fn(tone.w, tone.h, 1, |x, y| [img::srgb_encode(tone.get(x, y, 0)), 0.0, 0.0, 0.0]);
        ctx.image(
            "glass/caustics/caustic_tile",
            "assets/glass/caustic_tile_2k.png",
            &enc,
            Format::Png8,
            Spec::new("glass", "color", gen, CLUSTERS).alpha("additive").tile().qa(format!("tile seam score {:.2}", img::tile_seam_score(&enc))),
        )?;
        let frames: Vec<Img> = (0..64)
            .map(|i| {
                let f = pr::caustic(512, i as f32 / 64.0, 21, 0.22, 3);
                let f = pr::blur(&f, 0.6, true);
                let t = pr::caustic_tone(&f, 2.4);
                Img::from_fn(t.w, t.h, 1, |x, y| [img::srgb_encode(t.get(x, y, 0)), 0.0, 0.0, 0.0])
            })
            .collect();
        let loop_score = img::loop_seam_score(&frames);
        let tile_score = img::tile_seam_score(&frames[17]);
        let atlas = img::atlas(&frames, 8);
        ctx.image(
            "glass/caustics/caustic_loop",
            "assets/glass/caustic_loop_8x8.png",
            &atlas,
            Format::Png8,
            Spec::new("glass", "flipbook", gen, CLUSTERS)
                .alpha("additive")
                .tile()
                .flip(Flipbook { cols: 8, rows: 8, frames: 64, fps: 24, cell: 512 })
                .qa(format!("loop seam score {loop_score:.2}, cell tile seam score {tile_score:.2}")),
        )?;
        Ok(())
    })
}

fn frost_and_dither(ctx: &mut Ctx) -> Result<()> {
    let fp = fingerprint(&[b"frost-v1", include_bytes!("../procedural.rs")]);
    ctx.job("glass/frost", &fp, |ctx| {
        let gen = "rust:procedural::fbm (tileable gradient noise, 5 octaves) seed=31";
        let h = Img::from_fn(1024, 1024, 1, |x, y| {
            let u = (x as f32 + 0.5) / 1024.0;
            let v = (y as f32 + 0.5) / 1024.0;
            let coarse = pr::fbm(u, v, 16, 3, 0.5, 31);
            let fine = pr::fbm(u, v, 128, 3, 0.55, 32);
            [0.5 + 0.18 * coarse + 0.32 * fine, 0.0, 0.0, 0.0]
        });
        let n = img::normal_from_height(&h, 0, 3.0, true);
        ctx.image(
            "glass/frost/frost_normal",
            "assets/glass/frost_micro_normal.ktx2",
            &n,
            Format::Ktx2(Encoding::Normal),
            Spec::new("glass", "normal", gen, CLUSTERS).tile().qa(format!("tile seam score {:.2}", img::tile_seam_score(&n))),
        )?;
        ctx.image(
            "glass/frost/frost_height",
            "assets/glass/frost_micro_height.png",
            &h,
            Format::Png8,
            Spec::new("glass", "data", gen, CLUSTERS).tile().space("linear").qa("8-bit PNG, treat as linear"),
        )?;
        let bn = pr::blue_noise(128, 41);
        ctx.image(
            "glass/frost/blue_noise",
            "assets/glass/blue_noise_128.png",
            &bn,
            Format::Png8,
            Spec::new("glass", "data", "rust:procedural::blue_noise (void-and-cluster, sigma 1.9) seed=41", "every gradient (dither), glass blur")
                .tile()
                .space("linear")
                .qa("rank map; sample with point filtering"),
        )?;
        Ok(())
    })
}

fn sheen(ctx: &mut Ctx) -> Result<()> {
    let fp = fingerprint(&[b"sheen-v1", include_bytes!("../procedural.rs")]);
    ctx.job("glass/sheen", &fp, |ctx| {
        let s = pr::sheen_arch(4096, 1024);
        ctx.image(
            "glass/sheen/sheen_arch",
            "assets/glass/sheen_arch.png",
            &s,
            Format::Png8,
            Spec::new("glass", "color", "rust:procedural::sheen_arch (elliptical crown arch)", CLUSTERS).alpha("straight").qa("white; alpha carries the sheen"),
        )
    })
}
