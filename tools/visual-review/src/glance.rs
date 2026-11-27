//! `glance`: the tokens' blur gate (RESEARCH principle 4). The full frame is reduced to 960x540
//! in linear light and blurred at sigma 1.5, which is roughly what reaches a driver's eye at a
//! glance; each zone's contrast is then measured on that image. A heavier "squint" blur shows
//! what still stands out when detail is gone (the hierarchy check).
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::error::{usage, Result};
use crate::frame::{contrast, luminance, Linear};
use crate::geom::Rect;
use crate::overlay::{Map, Mark, Painter, MAGENTA, YELLOW};
use crate::tokens::Tokens;
use std::path::Path;

/// The squint view's blur (sigma at 960x540, about 16 px at 4K): my call, a viewing aid, not a
/// gate.
const SQUINT_SIGMA: f32 = 4.0;
/// The blur sigmas the command accepts (px at 960x540).
const SIGMA_RANGE: (f32, f32) = (0.1, 20.0);
/// WCAG 1.4.3 for normal text: the default for zones other than the gear and the speed.
const TEXT_MIN: f32 = 4.5;
/// The share of bright pixels below which a zone box holds almost no glyph.
const SHARE_LOW: f32 = 0.02;
/// The share above which a zone box holds too little background round the glyph.
const SHARE_HIGH: f32 = 0.85;

/// A measured zone: a name, a box in frame pixels and the contrast it needs.
#[derive(Debug, Clone, PartialEq)]
pub struct Zone {
    /// `gear`, `speed` or any other name.
    pub name: String,
    /// The box round the glyph, in full-frame pixels.
    pub rect: Rect,
    /// The contrast it needs.
    pub min: f32,
}

impl Zone {
    /// Parses `NAME:X,Y,W,H[:MIN]`. Without `MIN`, `gear` and `speed` take the blur-gate tokens
    /// and every other zone needs 4.5:1.
    ///
    /// # Errors
    /// A usage error for a malformed zone.
    pub fn parse(s: &str, t: &Tokens) -> Result<Self> {
        let mut parts = s.splitn(3, ':');
        let name = parts.next().unwrap_or("").trim();
        let rect = parts
            .next()
            .ok_or_else(|| usage(format!("--zone {s:?}: expected NAME:X,Y,W,H[:MIN]")))?;
        if name.is_empty() {
            return Err(usage(format!("--zone {s:?}: the zone needs a name")));
        }
        let rect = Rect::parse(rect)?;
        let min = match parts.next() {
            Some(m) => m
                .trim()
                .parse::<f32>()
                .ok()
                .filter(|v| *v >= 1.0)
                .ok_or_else(|| usage(format!("--zone {s:?}: MIN must be a ratio of at least 1")))?,
            None if name.eq_ignore_ascii_case("gear") => t.gear_min,
            None if name.eq_ignore_ascii_case("speed") => t.speed_min,
            None => TEXT_MIN,
        };
        Ok(Self {
            name: name.to_string(),
            rect,
            min,
        })
    }
}

/// A zone's numbers on the glance image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measure {
    /// Relative luminance of the brightest 5%.
    pub bright: f32,
    /// Relative luminance of the darkest 5%.
    pub dark: f32,
    /// WCAG contrast between them.
    pub ratio: f32,
    /// Share of the box brighter than the midpoint (how much of it is glyph).
    pub share: f32,
}

/// The `q`-quantile of `v` (reorders `v`).
fn quantile(v: &mut [f32], q: f32) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let i = (((v.len() - 1) as f32) * q.clamp(0.0, 1.0)).round() as usize;
    let (_, x, _) = v.select_nth_unstable_by(i, f32::total_cmp);
    *x
}

/// Measures the box `r` (glance pixels) of `img`.
pub fn measure(img: &Linear, r: Rect) -> Option<Measure> {
    let r = r.clamp_to(img.width, img.height)?;
    let (x, y, w, h) = r.parts();
    let mut ys = Vec::with_capacity(w as usize * h as usize);
    for yy in y..y + h {
        for xx in x..x + w {
            ys.push(luminance(img.at(xx, yy)));
        }
    }
    let bright = quantile(&mut ys, 0.95);
    let dark = quantile(&mut ys, 0.05);
    let mid = (bright + dark) * 0.5;
    let share = ys.iter().filter(|&&v| v > mid).count() as f32 / ys.len().max(1) as f32;
    Some(Measure {
        bright,
        dark,
        ratio: contrast(bright, dark),
        share,
    })
}

/// `glance`: writes the glance, squint and zone images and prints each zone's contrast.
///
/// # Errors
/// Usage and image errors.
pub fn run(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&["--zone", "--anchor", "--out", "--sigma", "--squint-sigma", "--over"])?;
    let [input] = args.pos() else {
        return Err(usage(
            "glance needs one render: a full frame, a <state>_crop.png or a sequence frame",
        ));
    };
    let path = Path::new(input);
    let t = &ctx.tokens;
    let sigma = args.num("--sigma", t.glance_sigma)?;
    let squint_sigma = args.num("--squint-sigma", SQUINT_SIGMA)?;
    for (flag, s) in [("--sigma", sigma), ("--squint-sigma", squint_sigma)] {
        if !s.is_finite() || !(SIGMA_RANGE.0..=SIGMA_RANGE.1).contains(&s) {
            return Err(usage(format!(
                "{flag} must be {} to {} px",
                SIGMA_RANGE.0, SIGMA_RANGE.1
            )));
        }
    }
    let loaded = ctx.load(path)?;
    // A crop or sequence frame goes back into a black 4K frame at its place, so the gate sees
    // the same frame size and the same coordinates as a full render.
    let mut frame = match ctx.origin(&loaded) {
        Some((0, 0)) => loaded,
        Some((ox, oy)) => {
            let mut full = crate::frame::Frame::filled(t.canvas.0, t.canvas.1, [0; 3]);
            full.blit(&loaded, u32::try_from(ox).unwrap_or(0), u32::try_from(oy).unwrap_or(0));
            if let Some(a) = &loaded.alpha {
                let mut alpha = vec![0u8; t.canvas.0 as usize * t.canvas.1 as usize];
                for y in 0..loaded.height as usize {
                    let src = a.get(y * loaded.width as usize..(y + 1) * loaded.width as usize);
                    let d = (y + oy.max(0) as usize) * t.canvas.0 as usize + ox.max(0) as usize;
                    if let (Some(src), Some(dst)) = (src, alpha.get_mut(d..d + loaded.width as usize)) {
                        dst.copy_from_slice(src);
                    }
                }
                full.alpha = Some(alpha);
            }
            full
        }
        None => {
            return Err(usage(format!(
                "glance needs a full {}x{} render, a <state>_crop.png or a sequence frame; {input} is {}x{}",
                t.canvas.0, t.canvas.1, loaded.width, loaded.height
            )))
        }
    };
    if let Some(bg) = cli::backdrop(args, ctx, frame.width, frame.height)? {
        if frame.alpha.is_none() {
            return Err(usage(format!(
                "{input} has no transparency (preview-export saves its renders already over black), so it cannot be put over a backdrop; --over needs a render that keeps its alpha"
            )));
        }
        frame = frame.over(&bg);
        out!("composited over {}", args.one("--over").unwrap_or(""));
    }
    let (gw, gh) = t.glance_size;
    let k = t.canvas.0 / gw.max(1);
    if k == 0 || t.canvas.0 != gw * k || t.canvas.1 != gh * k {
        return Err(usage(format!(
            "the blur gate size {gw}x{gh} is not an integer fraction of {}x{}",
            t.canvas.0, t.canvas.1
        )));
    }
    if (sigma - t.glance_sigma).abs() > f32::EPSILON {
        out!(
            "note: --sigma {sigma} is not the tokens' blur gate ({}); the 7:1 and 4.5:1 minimums are defined at the gate, so these verdicts are for this view only",
            t.glance_sigma
        );
    }
    let (ax, ay) = cli::anchor(args, ctx)?;
    let zones = args
        .all("--zone")
        .into_iter()
        .map(|z| {
            Zone::parse(z, t).map(|mut zone| {
                zone.rect = zone.rect.offset(ax, ay);
                zone
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let small = Linear::from_frame(&frame).box_down(k);
    let glance = small.blur(sigma);
    let squint = small.blur(squint_sigma);
    let dir = cli::out_dir(args, ctx, path);
    let stem = match args.one("--over") {
        Some(o) if o.starts_with('#') => format!("{}_over_{}", cli::stem(path), o.trim_start_matches('#')),
        Some(o) => format!("{}_over_{}", cli::stem(path), cli::stem(Path::new(o))),
        None => cli::stem(path),
    };
    let glance_png = dir.join(format!("{stem}_glance.png"));
    glance.to_frame().save(&glance_png)?;
    let squint_png = dir.join(format!("{stem}_squint.png"));
    squint.to_frame().save(&squint_png)?;
    out!("glance view ({gw}x{gh}, blur sigma {sigma}): {}", glance_png.display());
    out!("squint view (blur sigma {squint_sigma}): {}", squint_png.display());
    if zones.is_empty() {
        out!("no --zone given: pass the gear and speed boxes (NAME:X,Y,W,H) to measure their contrast");
        return Ok(());
    }

    let inv = 1.0 / f64::from(k);
    let mut marks = Vec::with_capacity(zones.len());
    for z in &zones {
        let small_rect = z.rect.scaled(inv);
        let Some(m) = measure(&glance, small_rect) else {
            out!("zone {} at {} is outside the frame: no", z.name, z.rect);
            continue;
        };
        let pass = m.ratio >= z.min;
        let verdict = if pass { "yes" } else { "no" };
        out!(
            "zone {} at {} (glance {small_rect}): bright {:.3}, dark {:.4}, contrast {:.1}:1, needs {:.1}:1: {verdict}",
            z.name,
            z.rect,
            m.bright,
            m.dark,
            m.ratio,
            z.min
        );
        if m.share < SHARE_LOW || m.share > SHARE_HIGH {
            out!(
                "  check the box: {:.0}% of it is glyph; draw it round the glyph with background on every side",
                m.share * 100.0
            );
        }
        marks.push(Mark::Box {
            rect: small_rect,
            label: format!("{} {:.1}:1 {verdict}", z.name, m.ratio),
            color: if pass { YELLOW } else { MAGENTA },
        });
    }
    let zoned = Painter::new(&ctx.root).annotated(&glance.to_frame(), &marks, Map::IDENTITY)?;
    let zoned_png = dir.join(format!("{stem}_glance_zones.png"));
    zoned.save(&zoned_png)?;
    out!("zones drawn on the glance view: {}", zoned_png.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Frame;

    fn tokens() -> Tokens {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        Tokens::load(&root).expect("tokens")
    }

    #[test]
    fn zones_parse_with_token_defaults() {
        let t = tokens();
        let g = Zone::parse("gear:1800,1600,240,260", &t).expect("parses");
        assert_eq!(g.rect, Rect::new(1800, 1600, 240, 260));
        assert!((g.min - 7.0).abs() < 1e-6);
        assert!((Zone::parse("speed:1,2,3,4", &t).expect("parses").min - 4.5).abs() < 1e-6);
        assert!((Zone::parse("tach:1,2,3,4:3", &t).expect("parses").min - 3.0).abs() < 1e-6);
        for bad in ["gear", ":1,2,3,4", "gear:1,2,3", "gear:1,2,3,4:0.5", "gear:1,2,3,4:x"] {
            assert!(Zone::parse(bad, &t).is_err(), "{bad:?} should be rejected");
        }
    }

    /// A white bar `bw` x `bh` (4K px) at the centre of a black 4K frame, at grey level `g`.
    fn bar(bw: u32, bh: u32, g: u8) -> Frame {
        let mut f = Frame::filled(3840, 2160, [0; 3]);
        f.fill(
            Rect::new(
                1920 - i64::from(bw / 2),
                1080 - i64::from(bh / 2),
                i64::from(bw),
                i64::from(bh),
            ),
            [g, g, g],
        );
        f
    }

    #[test]
    fn a_bold_white_glyph_passes_and_a_hairline_or_dim_one_fails() {
        let t = tokens();
        let zone = Rect::new(1820, 980, 200, 200).scaled(0.25);
        let measure_bar = |f: &Frame| {
            let g = Linear::from_frame(f).box_down(4).blur(t.glance_sigma);
            measure(&g, zone).expect("inside")
        };
        // A 40 px stroke survives the blur at full contrast.
        let bold = measure_bar(&bar(40, 160, 255));
        assert!(bold.ratio > 15.0, "bold {:?}", bold);
        // A 1 px hairline nearly vanishes at a glance.
        let hair = measure_bar(&bar(1, 160, 255));
        assert!(hair.ratio < t.gear_min, "hairline {:?}", hair);
        // Dim grey on black fails the gear's 7:1 even when bold.
        let dim = measure_bar(&bar(40, 160, 70));
        assert!(dim.ratio < t.gear_min, "dim {:?}", dim);
    }

    #[test]
    fn quantiles_pick_the_tails() {
        let mut v: Vec<f32> = (0..=100).map(|i| i as f32).collect();
        assert!((quantile(&mut v, 0.95) - 95.0).abs() < 1e-6);
        assert!((quantile(&mut v, 0.05) - 5.0).abs() < 1e-6);
        assert_eq!(quantile(&mut [], 0.5), 0.0);
    }
}
