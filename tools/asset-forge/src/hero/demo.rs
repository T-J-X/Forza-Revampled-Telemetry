//! `asset-forge hero-demo`: the asset pipeline v2 proof. Rebuilds PorscheAnalog's hardware with
//! the library (machined titanium bezels with knurled chamfers, full-strength sunray faces,
//! sapphire hub lens, faceted needle with lume, piano-black visor with LED jewels, a stitched
//! leather hood textured by ComfyUI), composites the live vectors on top exactly as the kit
//! would, and writes the review images to `docs/handover-shots/asset-pipeline-v2-*.png`.
//!
//! It never touches `assets/` or the ledger: renders go to `target/asset-forge/hero-demo/`.
//! The PorscheAnalog cluster module is not edited; its chat adopts the library.
use super::frame::{self, Frame};
use super::geom::{self, BezelSpec, Outline};
use super::mat::{self, metal, Mat};
use super::part::{self, Part};
use super::{texture, Passes, Scene};
use crate::error::{io_err, Result};
use crate::img::{self, Img};
use crate::paths::Paths;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Tach centre and radius (PorscheAnalog zone table).
const TACH: ([f32; 2], f32) = ([0.0, -14.0], 226.0);
/// Satellites: speed (left) and G / pedals (right).
const SATS: [([f32; 2], f32); 2] = [([-416.0, 24.0], 176.0), ([416.0, 24.0], 176.0)];

/// Runs the demo. `preview` halves the resolution and the samples for quick iteration.
pub fn run(paths: &Paths, preview: bool) -> Result<()> {
    let root = paths.scratch.join("hero-demo");
    let shots = paths.root.join("docs").join("handover-shots");
    std::fs::create_dir_all(&shots).map_err(|e| io_err(&shots, e))?;
    let scale = if preview { 0.5 } else { 1.0 };
    let samples = if preview { 64 } else { 384 };

    // 1. ComfyUI: the leather grain for the hood (falls back to procedural grain).
    let leather_tex = match texture::recipe("leather_black_pebble") {
        Some(r) => match texture::make_texture(paths, &r, 2048, &root.join("textures")) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("  leather texture skipped (procedural grain instead): {e}");
                None
            }
        },
        None => None,
    };

    // 2. Blender: the hardware plate, the needle sprite and the glass layer.
    let mut hw = hardware(leather_tex.as_deref(), scale, samples);
    hw.passes = Passes::ALL;
    let hw_out = hw.render(paths, &root.join("hardware"))?;
    println!(
        "  hardware plate: {} passes in {}",
        hw_out.beauty.len(),
        hw_out.dir.display()
    );
    let mut nd = needle(scale, samples);
    nd.passes = Passes {
        beauty: true,
        swing: true,
        data: true,
        ao: false,
        rmh: false,
        emission: false,
    };
    let nd_out = nd.render(paths, &root.join("needle"))?;
    let mut gl = glass(scale, samples);
    gl.passes = Passes {
        beauty: true,
        swing: true,
        data: false,
        ao: false,
        rmh: false,
        emission: false,
    };
    let gl_out = gl.render(paths, &root.join("glass"))?;

    // 3. Composite the way the kit draws it: plate, live vectors, needle, glass.
    let plate = img::load(&hw_out.beauty[0])?;
    let mut frame_img = Img::new(plate.w, plate.h, 4);
    fill(&mut frame_img, [0.0, 0.0, 0.0, 1.0]);
    over(&mut frame_img, &plate, 0, 0);
    let fr = hw.frame;
    let k = plate.w as f32 / fr.w;
    let to_px = |p: [f32; 2]| [(p[0] - fr.x) * k, (p[1] - fr.y) * k];
    let fonts = paths.root.join("assets").join("fonts");
    let vectors = live_vectors(fr, k, &fonts)?;
    over(&mut frame_img, &vectors, 0, 0);
    let needle_img = img::load(&nd_out.beauty[0])?;
    let rpm_angle = 135.0 + 270.0 * 6200.0 / 9000.0;
    let rot = rpm_angle - 270.0;
    let c = to_px(TACH.0);
    shadow(&mut frame_img, &needle_img, c, rot, 7.0 * k, 0.55);
    stamp_rotated(&mut frame_img, &needle_img, c, rot);
    let glass_img = img::load(&gl_out.beauty[0])?;
    over(&mut frame_img, &glass_img, 0, 0);
    let after = shots.join("asset-pipeline-v2-after.png");
    img::save_png(&frame_img.to_channels(3), &after, false, true)?;
    println!("  wrote {}", after.display());

    // 4. The pass sheet (beauty rest / swing, normal, AO, rmh, depth) and the swing strip.
    let mut tiles: Vec<(String, Img)> = Vec::new();
    for (i, b) in hw_out.beauty.iter().enumerate() {
        let az = if i == 0 { -135.0 } else { hw.swing[i - 1] };
        tiles.push((format!("beauty key {az:.0}°"), flatten(&img::load(b)?)));
    }
    for (label, p) in [
        ("normal", &hw_out.normal),
        ("ambient occlusion", &hw_out.ao),
        ("rough/metal/height", &hw_out.rmh),
        ("ControlNet depth", &hw_out.depth),
    ] {
        if let Some(p) = p {
            tiles.push((label.into(), flatten(&img::load(p)?)));
        }
    }
    let sheet = contact(&tiles, 2, 960)?;
    let passes = shots.join("asset-pipeline-v2-passes.png");
    img::save_png(&sheet, &passes, false, true)?;
    println!("  wrote {}", passes.display());

    // 5. The material cookbook, rendered (validates every builder in Cycles).
    let mut sw = swatches(scale, samples.min(192));
    sw.passes = Passes::BEAUTY;
    let sw_out = sw.render(paths, &root.join("swatches"))?;
    let sw_img = flatten(&img::load(&sw_out.beauty[0])?);
    let sw_lab = label_swatches(&sw_img, &fonts)?;
    let mats = shots.join("asset-pipeline-v2-materials.png");
    img::save_png(&sw_lab, &mats, false, true)?;
    println!("  wrote {}", mats.display());
    Ok(())
}

fn ti_set() -> [Mat; 4] {
    [
        mat::polished(metal::TITANIUM).named("ti_polished"),
        mat::spun(metal::TITANIUM).named("ti_spun"),
        mat::brushed(metal::GUNMETAL).named("ti_wall"),
        mat::knurled(metal::TITANIUM, 5.0, false).named("ti_knurl"),
    ]
}

/// The static hardware: hood, bezels, faces, hub lens well, bearing ring, visor and jewels.
#[must_use]
pub fn hardware(leather_tex: Option<&Path>, scale: f32, samples: u32) -> Scene {
    let fr = Frame::envelope(&frame::COCKPIT).scaled(scale);
    let mut s = Scene::new("porsche_hw", fr);
    s.samples = samples;
    // Leather hood: smooth union of the three gauge pods, pillow edge, saddle stitching.
    let pods = |grow: f32| {
        let mut v = vec![(TACH.0[0], TACH.0[1], TACH.1 + 18.0 + grow)];
        for (c, r) in SATS {
            v.push((c[0], c[1], r + 14.0 + grow));
        }
        v
    };
    let hood = Outline::circle_union(&pods(46.0), 80.0, (0.0, 0.0), 1440);
    let mut leather = mat::leather([0.03, 0.026, 0.024], 2.6);
    if let Some(t) = leather_tex {
        leather = leather
            .set("texture", t.to_string_lossy().to_string())
            .set("tile_px", 150.0)
            .set("tint", serde_json::json!([0.55, 0.52, 0.5]));
    }
    s.add(Part::extrude("hood", hood, -44.0, 30.0, 14.0, leather));
    let seam = Outline::circle_union(&pods(30.0), 80.0, (0.0, 0.0), 1440);
    let thread = mat::matte([0.5, 0.08, 0.04]).named("thread").rough(0.6);
    s.add(Part::mesh(
        "stitches",
        [0.0, 0.0],
        -14.0,
        part::stitches(&seam, 11.0, 7.0, 2.2, 1.6),
        vec![thread],
    ));
    // Tach.
    let face = mat::sunray([0.14, 0.145, 0.15]).named("sunray_anthracite");
    s.add(Part::face("tach_face", TACH.0, 0.0, TACH.1 + 2.0, face.clone()));
    s.add(Part::bezel(
        "tach_bezel",
        TACH.0,
        0.0,
        BezelSpec::ring(TACH.1, 18.0),
        ti_set(),
    ));
    s.add(Part::face("hub_well", TACH.0, 0.6, 96.0, mat::piano_black()));
    s.add(Part::ring(
        "hub_bearing",
        TACH.0,
        0.6,
        96.0,
        105.0,
        4.0,
        mat::polished(metal::STEEL).named("steel_polished"),
    ));
    // Satellites.
    for (i, (c, r)) in SATS.iter().enumerate() {
        s.add(Part::face(&format!("sat{i}_face"), *c, 0.0, r + 2.0, face.clone()));
        s.add(Part::bezel(
            &format!("sat{i}_bezel"),
            *c,
            0.0,
            BezelSpec::ring(*r, 14.0),
            ti_set(),
        ));
    }
    // G-ball well on the right satellite.
    s.add(Part::face("g_well", SATS[1].0, 0.6, 76.0, mat::piano_black()));
    s.add(Part::ring(
        "g_ring",
        SATS[1].0,
        0.6,
        76.0,
        81.0,
        3.0,
        mat::polished(metal::STEEL).named("steel_polished"),
    ));
    // Visor with 15 LED jewels (unlit; the kit lights them).
    s.add(Part::extrude(
        "visor",
        Outline::rrect(0.0, -300.0, 780.0, 44.0, 22.0, 12),
        -12.0,
        12.0,
        5.0,
        mat::piano_black(),
    ));
    for j in 0..15 {
        let x = -364.0 + 52.0 * j as f32;
        let col = if j < 5 {
            [0.05, 0.9, 0.12]
        } else if j < 10 {
            [1.0, 0.6, 0.02]
        } else {
            [1.0, 0.04, 0.02]
        };
        let cap = mat::led_cap(col, 0.0).named(&format!("led_{}", j / 5));
        s.add(Part::ring(
            &format!("jewel{j}_bezel"),
            [x, -300.0],
            0.0,
            13.0,
            16.0,
            3.0,
            mat::polished(metal::TITANIUM).named("ti_polished"),
        ));
        s.add(Part::dome(
            &format!("jewel{j}"),
            [x, -300.0],
            0.0,
            13.0,
            6.0,
            1.0,
            [cap, mat::piano_black()],
        ));
    }
    s
}

/// The needle sprite: pointing up, pivot at the image centre, 448 px square.
#[must_use]
pub fn needle(scale: f32, samples: u32) -> Scene {
    let mut s = Scene::new("porsche_needle", Frame::square(0.0, 0.0, 448.0).scaled(scale));
    s.samples = samples;
    let facets = mat::polished(metal::STEEL).named("needle_steel").rough(0.32);
    let lume = mat::lume([0.95, 0.93, 0.86], 0.15).named("needle_lume");
    let under = mat::matte([0.02, 0.02, 0.02]).named("needle_under");
    s.add(Part::mesh(
        "blade",
        [0.0, 0.0],
        6.0,
        geom::blade(104.0, 210.0, 9.0, 0.22, 4.0, 10.0),
        vec![facets.clone(), lume, under.clone()],
    ));
    s.add(Part::mesh(
        "counterweight",
        [0.0, 0.0],
        6.0,
        geom::blade(-104.0, -152.0, 10.0, 0.55, 3.5, 0.0),
        vec![facets.clone(), facets, under],
    ));
    s
}

/// The glass layer: the tach crystal and the hub lens, reflections only (transparent film).
#[must_use]
pub fn glass(scale: f32, samples: u32) -> Scene {
    let mut s = Scene::new("porsche_glass", Frame::envelope(&frame::COCKPIT).scaled(scale));
    s.samples = samples;
    s.add(Part::crystal("hub_lens", TACH.0, 4.0, 96.0, 5.0, 3.0, mat::sapphire()));
    s.add(Part::crystal(
        "tach_crystal",
        TACH.0,
        12.0,
        TACH.1,
        4.0,
        2.0,
        mat::sapphire(),
    ));
    for (i, (c, r)) in SATS.iter().enumerate() {
        s.add(Part::crystal(
            &format!("sat{i}_crystal"),
            *c,
            10.0,
            *r,
            4.0,
            1.5,
            mat::sapphire(),
        ));
    }
    s
}

/// The cookbook swatch board: one machined puck per material, 6 × 4.
#[must_use]
pub fn swatches(scale: f32, samples: u32) -> Scene {
    let book = mat::cookbook();
    let (cols, pitch) = (6usize, 240.0f32);
    let rows = book.len().div_ceil(cols);
    let fr = Frame::rect(0.0, 0.0, cols as f32 * pitch, rows as f32 * pitch).scaled(scale);
    let mut s = Scene::new("cookbook", fr);
    s.samples = samples;
    s.swing.clear();
    let puck = geom::Profile::new()
        .sharp(0.0, 14.0, 0)
        .round(80.0, 14.0, 0, 2.0)
        .round(92.0, 4.0, 0, 2.0)
        .sharp(92.0, -4.0, 0)
        .build();
    let card = mat::ink([0.8, 0.8, 0.78]).named("swatch_card");
    for (i, m) in book.into_iter().enumerate() {
        let c = [pitch * ((i % cols) as f32 + 0.5), pitch * ((i / cols) as f32 + 0.45)];
        let glassy = m.v.get("kind").and_then(serde_json::Value::as_str) == Some("glass");
        if glassy {
            // Glass needs something to refract: a printed card with a stripe.
            s.add(Part::face(&format!("card{i}"), c, -6.0, 70.0, card.clone()));
            s.add(Part::ring(
                &format!("cardring{i}"),
                c,
                -6.0,
                30.0,
                44.0,
                1.0,
                mat::matte([0.02, 0.02, 0.02]).named("swatch_black"),
            ));
        }
        s.add(Part::lathe(&format!("puck{i}"), c, 0.0, puck.clone(), vec![m]));
    }
    s
}

fn flatten(src: &Img) -> Img {
    let s = src.to_channels(4);
    Img::from_fn(s.w, s.h, 3, |x, y| {
        let a = s.get(x, y, 3);
        [s.get(x, y, 0) * a, s.get(x, y, 1) * a, s.get(x, y, 2) * a, 1.0]
    })
}

fn fill(dst: &mut Img, v: [f32; 4]) {
    for px in dst.data.chunks_exact_mut(dst.c) {
        px.copy_from_slice(&v[..px.len()]);
    }
}

/// Straight-alpha "over" in encoded sRGB (what the kit's Normal blend does on premultiplied).
fn over(dst: &mut Img, src: &Img, ox: usize, oy: usize) {
    let s = src.to_channels(4);
    for y in 0..s.h {
        for x in 0..s.w {
            let (dx, dy) = (x + ox, y + oy);
            if dx >= dst.w || dy >= dst.h {
                continue;
            }
            let a = s.get(x, y, 3);
            if a <= 0.0 {
                continue;
            }
            for ch in 0..3 {
                let v = s.get(x, y, ch) * a + dst.get(dx, dy, ch) * (1.0 - a);
                dst.set(dx, dy, ch, v);
            }
            if dst.c == 4 {
                let da = dst.get(dx, dy, 3);
                dst.set(dx, dy, 3, a + da * (1.0 - a));
            }
        }
    }
}

/// Draws `sprite` (pivot at its centre) rotated `deg` clockwise with its centre at `c`.
fn stamp_rotated(dst: &mut Img, sprite: &Img, c: [f32; 2], deg: f32) {
    let s = sprite.to_channels(4);
    let (sin, cos) = deg.to_radians().sin_cos();
    let r = (s.w.max(s.h) as f32) * 0.75;
    let (x0, x1) = ((c[0] - r).max(0.0) as usize, ((c[0] + r) as usize).min(dst.w));
    let (y0, y1) = ((c[1] - r).max(0.0) as usize, ((c[1] + r) as usize).min(dst.h));
    for y in y0..y1 {
        for x in x0..x1 {
            let (dx, dy) = (x as f32 + 0.5 - c[0], y as f32 + 0.5 - c[1]);
            // Inverse rotation into sprite space.
            let sx = dx * cos + dy * sin + s.w as f32 * 0.5;
            let sy = -dx * sin + dy * cos + s.h as f32 * 0.5;
            if sx < 0.0 || sy < 0.0 || sx >= s.w as f32 || sy >= s.h as f32 {
                continue;
            }
            let a = s.sample(sx, sy, 3, false);
            if a <= 0.0 {
                continue;
            }
            for ch in 0..3 {
                let v = s.sample(sx, sy, ch, false) * a + dst.get(x, y, ch) * (1.0 - a);
                dst.set(x, y, ch, v);
            }
        }
    }
}

/// A soft contact shadow of the sprite, offset away from the upper-left key.
fn shadow(dst: &mut Img, sprite: &Img, c: [f32; 2], deg: f32, offset: f32, strength: f32) {
    let s = sprite.to_channels(4);
    let mut a = Img::from_fn(s.w, s.h, 1, |x, y| [s.get(x, y, 3), 0.0, 0.0, 0.0]);
    for _ in 0..3 {
        a = box_blur(&a, (offset * 0.6).max(1.0) as usize);
    }
    let dark = Img::from_fn(s.w, s.h, 4, |x, y| [0.0, 0.0, 0.0, a.get(x, y, 0) * strength]);
    stamp_rotated(dst, &dark, [c[0] + offset * 0.7, c[1] + offset * 0.7], deg);
}

fn box_blur(src: &Img, r: usize) -> Img {
    let pass = |img: &Img, horiz: bool| {
        Img::from_fn(img.w, img.h, 1, |x, y| {
            let mut sum = 0.0;
            let mut n = 0.0;
            for d in 0..=(2 * r) {
                let o = d as isize - r as isize;
                let v = if horiz {
                    img.get_clamp(x as isize + o, y as isize, 0)
                } else {
                    img.get_clamp(x as isize, y as isize + o, 0)
                };
                sum += v;
                n += 1.0;
            }
            [sum / n, 0.0, 0.0, 0.0]
        })
    };
    pass(&pass(src, true), false)
}

/// The live layer PorscheAnalog keeps in vectors: ticks, numerals, redline, gear, speed,
/// G-ball and pedal arcs. Drawn here only to show the plate under real content.
fn live_vectors(fr: Frame, k: f32, fonts: &Path) -> Result<Img> {
    let (w, h) = ((fr.w * k).round() as u32, (fr.h * k).round() as u32);
    let mut svg = String::new();
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="{} {} {} {}">"#,
        fr.w, fr.h, fr.x, fr.y, fr.w, fr.h
    );
    let (tc, _) = TACH;
    let pt = |c: [f32; 2], r: f32, deg: f32| {
        let a = deg.to_radians();
        (c[0] + r * a.cos(), c[1] + r * a.sin())
    };
    // Tach ticks every 250 rpm, majors every 1000, red from 7500.
    for i in 0..=36 {
        let rpm = i as f32 * 250.0;
        let deg = 135.0 + 270.0 * rpm / 9000.0;
        let major = i % 4 == 0;
        let (r0, r1, sw) = if major {
            (186.0, 220.0, 5.0)
        } else {
            (204.0, 220.0, 2.4)
        };
        let col = if rpm >= 7500.0 { "#ff2a1a" } else { "#f4f1ea" };
        let (x0, y0) = pt(tc, r0, deg);
        let (x1, y1) = pt(tc, r1, deg);
        let _ = write!(
            svg,
            r#"<line x1="{x0:.2}" y1="{y0:.2}" x2="{x1:.2}" y2="{y1:.2}" stroke="{col}" stroke-width="{sw}" stroke-linecap="butt"/>"#
        );
        if major {
            let (nx, ny) = pt(tc, 158.0, deg);
            let _ = write!(
                svg,
                r##"<text x="{nx:.1}" y="{:.1}" font-family="Titillium Web" font-weight="600" font-size="34" fill="#f4f1ea" text-anchor="middle">{}</text>"##,
                ny + 12.0,
                i / 4
            );
        }
    }
    let (rx0, ry0) = pt(tc, 224.0, 135.0 + 270.0 * 7500.0 / 9000.0);
    let (rx1, ry1) = pt(tc, 224.0, 405.0);
    let _ = write!(
        svg,
        r##"<path d="M{rx0:.2},{ry0:.2} A224,224 0 0,1 {rx1:.2},{ry1:.2}" fill="none" stroke="#ff2a1a" stroke-width="5"/>"##
    );
    let _ = write!(
        svg,
        r##"<text x="{}" y="{}" font-family="Titillium Web" font-weight="700" font-size="120" fill="#ffffff" text-anchor="middle">3</text>"##,
        tc[0],
        tc[1] + 42.0
    );
    let _ = write!(
        svg,
        r##"<text x="{}" y="{}" font-family="Titillium Web" font-weight="600" font-size="22" fill="#b8b4ac" text-anchor="middle" letter-spacing="3">x1000 RPM</text>"##,
        tc[0],
        tc[1] + 140.0
    );
    // Speed satellite.
    let (sc, _) = SATS[0];
    for i in 0..=40 {
        let deg = 135.0 + 270.0 * i as f32 / 40.0;
        let major = i % 5 == 0;
        let (r0, r1, sw) = if major {
            (146.0, 168.0, 4.0)
        } else {
            (158.0, 168.0, 2.0)
        };
        let (x0, y0) = pt(sc, r0, deg);
        let (x1, y1) = pt(sc, r1, deg);
        let _ = write!(
            svg,
            r##"<line x1="{x0:.2}" y1="{y0:.2}" x2="{x1:.2}" y2="{y1:.2}" stroke="#f4f1ea" stroke-width="{sw}"/>"##
        );
    }
    let _ = write!(
        svg,
        r##"<text x="{}" y="{}" font-family="Titillium Web" font-weight="700" font-size="104" fill="#ffffff" text-anchor="middle">74</text>"##,
        sc[0],
        sc[1] + 34.0
    );
    let _ = write!(
        svg,
        r##"<text x="{}" y="{}" font-family="Titillium Web" font-weight="600" font-size="24" fill="#b8b4ac" text-anchor="middle" letter-spacing="4">MPH</text>"##,
        sc[0],
        sc[1] + 82.0
    );
    // G / pedal satellite.
    let (gc, _) = SATS[1];
    let _ = write!(
        svg,
        r##"<circle cx="{}" cy="{}" r="38" fill="none" stroke="#6f6b64" stroke-width="2"/><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#6f6b64" stroke-width="2"/><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#6f6b64" stroke-width="2"/>"##,
        gc[0],
        gc[1],
        gc[0] - 72.0,
        gc[1],
        gc[0] + 72.0,
        gc[1],
        gc[0],
        gc[1] - 72.0,
        gc[0],
        gc[1] + 72.0
    );
    let _ = write!(
        svg,
        r##"<circle cx="{}" cy="{}" r="13" fill="#ff7a12"/>"##,
        gc[0] + 26.0,
        gc[1] - 18.0
    );
    let arc = |r: f32, a0: f32, a1: f32| {
        let (x0, y0) = pt(gc, r, a0);
        let (x1, y1) = pt(gc, r, a1);
        format!("M{x0:.2},{y0:.2} A{r},{r} 0 0,1 {x1:.2},{y1:.2}")
    };
    let _ = write!(
        svg,
        r##"<path d="{}" fill="none" stroke="#2a2a2a" stroke-width="12" stroke-linecap="round"/><path d="{}" fill="none" stroke="#35d07f" stroke-width="12" stroke-linecap="round"/>"##,
        arc(140.0, 200.0, 340.0),
        arc(140.0, 200.0, 300.0)
    );
    let _ = write!(
        svg,
        r##"<path d="{}" fill="none" stroke="#2a2a2a" stroke-width="12" stroke-linecap="round"/><path d="{}" fill="none" stroke="#ff3b2f" stroke-width="12" stroke-linecap="round"/>"##,
        arc(140.0, 20.0, 160.0),
        arc(140.0, 20.0, 52.0)
    );
    svg.push_str("</svg>");
    let opt = crate::svg::options(&[fonts]);
    let tree = crate::svg::parse(svg.as_bytes(), &opt, "hero-demo-vectors")?;
    crate::svg::render_into(&tree, w, h, k, "hero-demo-vectors")
}

fn label_swatches(src: &Img, fonts: &Path) -> Result<Img> {
    let book = mat::cookbook();
    let cols = 6usize;
    let pitch = src.w as f32 / cols as f32;
    let mut svg = String::new();
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#,
        src.w, src.h
    );
    for (i, m) in book.iter().enumerate() {
        let x = pitch * ((i % cols) as f32 + 0.5);
        let y = pitch * ((i / cols) as f32 + 0.93);
        let _ = write!(
            svg,
            r##"<text x="{x:.1}" y="{y:.1}" font-family="Titillium Web" font-size="{:.1}" fill="#c8c4bc" text-anchor="middle">{}</text>"##,
            pitch * 0.075,
            m.name()
        );
    }
    svg.push_str("</svg>");
    let opt = crate::svg::options(&[fonts]);
    let tree = crate::svg::parse(svg.as_bytes(), &opt, "swatch-labels")?;
    let labels = crate::svg::render_into(&tree, src.w as u32, src.h as u32, 1.0, "swatch-labels")?;
    let mut out = src.to_channels(3);
    over(&mut out, &labels, 0, 0);
    Ok(out)
}

/// A labelled grid of images, each scaled to `tile_w` px wide (rows as tall as their tallest).
fn contact(tiles: &[(String, Img)], cols: usize, tile_w: usize) -> Result<Img> {
    let cols = cols.max(1);
    let label_h = 44;
    let hs: Vec<usize> = tiles
        .iter()
        .map(|(_, t)| (t.h as f32 * tile_w as f32 / t.w.max(1) as f32).round().max(1.0) as usize)
        .collect();
    let rows = tiles.len().div_ceil(cols);
    let row_h: Vec<usize> = (0..rows)
        .map(|r| hs.iter().skip(r * cols).take(cols).copied().max().unwrap_or(1))
        .collect();
    let w = cols * tile_w + (cols + 1) * 16;
    let h = row_h.iter().map(|rh| rh + label_h + 16).sum::<usize>() + 16;
    let mut out = Img::new(w, h, 3);
    fill(&mut out, [0.06, 0.06, 0.065, 1.0]);
    let mut svg = String::new();
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">"#
    );
    for (i, (label, t)) in tiles.iter().enumerate() {
        let row = i / cols;
        let cx = 16 + (i % cols) * (tile_w + 16);
        let cy = 16 + row_h.iter().take(row).map(|rh| rh + label_h + 16).sum::<usize>();
        let r = t.resize(tile_w, hs[i], false);
        out.paste(&r.to_channels(3), cx, cy + label_h);
        let _ = write!(
            svg,
            r##"<text x="{}" y="{}" font-family="Titillium Web" font-weight="600" font-size="26" fill="#e8e4dc">{label}</text>"##,
            cx,
            cy + 32
        );
    }
    svg.push_str("</svg>");
    let opt = crate::svg::options(&[]);
    let tree = crate::svg::parse(svg.as_bytes(), &opt, "contact")?;
    let labels = crate::svg::render_into(&tree, w as u32, h as u32, 1.0, "contact")?;
    over(&mut out, &labels, 0, 0);
    Ok(out)
}

/// The side-by-side for the conductor: the Penpot board above the render, both at the same
/// width. A full 3840x2160 board export is cropped to the cockpit envelope (the render's frame).
pub fn versus(board: &Path, after: &Path, out: &Path) -> Result<PathBuf> {
    let mut a = img::load(board)?.to_channels(3);
    if a.w == 3840 && a.h == 2160 {
        let (an, [ex, ey, ew, eh]) = (frame::COCKPIT.screen, frame::COCKPIT.envelope);
        a = a.crop((an[0] + ex) as usize, (an[1] + ey) as usize, ew as usize, eh as usize);
    }
    let b = img::load(after)?.to_channels(3);
    let w = b.w;
    let ah = (a.h as f32 * w as f32 / a.w as f32).round() as usize;
    let a = a.resize(w, ah, false);
    let tiles = vec![
        ("Penpot board · 07 PorscheAnalog (layout and intent)".to_string(), a),
        (
            "asset pipeline v2 · Blender + ComfyUI hardware, live vectors on top".to_string(),
            b,
        ),
    ];
    let sheet = contact(&tiles, 1, w)?;
    img::save_png(&sheet, out, false, true)?;
    Ok(out.to_path_buf())
}
