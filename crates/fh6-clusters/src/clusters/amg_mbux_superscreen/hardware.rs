//! The rendered plates: Blender renders from `assets/clusters/amg_mbux_superscreen/` (recipe
//! `tools/asset-forge/src/recipes/amg_mbux_superscreen.rs`, board v4) and the procedural
//! fallbacks drawn when a file is missing.
//!
//! - `sky.png`: the SDXL dusk sky seen through the tunnel. It wraps left to right; one turn
//!   covers [`Geom::sky`], and the cluster scrolls it with the speed and leans it with lateral G.
//! - `assembly_0.png`, `assembly_1.png`: the twelve ring ribs (nearly clear above the horizon),
//!   the road and the two side-scale bands, with the key light at rest and swung; cross-faded by
//!   lateral G. They cover [`Geom::assembly_rect`] 1:1.
//! - `crescents.png`: both crescents' halftone as light in alpha (white), tinted by the
//!   palette's primary at draw time, so a custom palette recolours it. It covers
//!   [`Geom::crescent_rect`] 1:1.
//! - `lens_0.png`, `lens_1.png` and `gauge_0.png`, `gauge_1.png`: the G-Force lens (dark smoked
//!   glass in a polished ring) and the boost gauge (a dark face in a chrome bezel), per light.
use super::geom::{Geom, CRESCENT_SAMPLES, RINGS};
use fh6_kit::cluster::Setup;
use fh6_kit::{deg, vec2, Cap, Color, Gradient, ImageId, ImagePaint, Painter, Palette, Rect, Shape, Vec2};

/// Light azimuths of the lit renders: rest, then the light swung by lateral G.
pub(super) const AZIMUTHS: usize = 2;
/// The asset folder of this dash.
const FOLDER: &str = "amg_mbux_superscreen";
/// The procedural fallbacks' colours (sRGB 8-bit, from boards v3 and v4).
const BAND_BODY: [u8; 3] = [36, 35, 44];
const BAND_RIM: [u8; 3] = [201, 208, 217];
const BAND_LIP: [u8; 3] = [74, 80, 91];
const RING_LINE: [u8; 3] = [58, 64, 73];
const SKY_TOP: [u8; 3] = [10, 14, 26];
const SKY_MID: [u8; 3] = [34, 42, 62];
const SKY_LOW: [u8; 3] = [150, 150, 160];
const ROAD_TOP: [u8; 3] = [38, 42, 48];
const ROAD_EDGE: [u8; 3] = [213, 220, 229];

/// Every loaded plate (`None` when its file is missing).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Plates {
    sky: Option<ImageId>,
    assembly: [Option<ImageId>; AZIMUTHS],
    crescents: Option<ImageId>,
    lens: [Option<ImageId>; AZIMUTHS],
    gauge: [Option<ImageId>; AZIMUTHS],
}

/// Loads one file of this dash's asset folder, when the assets folder is known and holds it.
fn load(setup: &mut Setup<'_>, name: &str) -> Option<ImageId> {
    let path = setup.assets?.join("clusters").join(FOLDER).join(name);
    if !path.is_file() {
        return None;
    }
    match setup.images.load_file(setup.device, setup.queue, &path, None) {
        Ok(image) => Some(image),
        Err(error) => {
            // The dash still draws: every plate has a procedural fallback.
            tracing::warn!(asset = %path.display(), %error, "an AmgMbuxSuperscreen asset did not load");
            None
        }
    }
}

fn rgb(c: [u8; 3]) -> Color {
    Color::srgb8(c[0], c[1], c[2], 255)
}

/// Draws a lit plate, cross-faded to the swung light by `swing` (0..1).
fn lit_plate(p: &mut Painter<'_>, pair: [Option<ImageId>; AZIMUTHS], rect: Rect, swing: f32) -> bool {
    let Some(base) = pair[0] else {
        return false;
    };
    p.fill(&Shape::rect(rect), ImagePaint::new(base, rect));
    if let Some(swung) = pair[1] {
        let w = swing.clamp(0.0, 1.0);
        if w > 0.003 {
            p.scoped(|p| {
                p.set_opacity(w);
                p.fill(&Shape::rect(rect), ImagePaint::new(swung, rect));
            });
        }
    }
    true
}

impl Plates {
    /// Loads every plate once.
    pub fn load(setup: &mut Setup<'_>) -> Self {
        Self {
            sky: load(setup, "sky.png"),
            assembly: [load(setup, "assembly_0.png"), load(setup, "assembly_1.png")],
            crescents: load(setup, "crescents.png"),
            lens: [load(setup, "lens_0.png"), load(setup, "lens_1.png")],
            gauge: [load(setup, "gauge_0.png"), load(setup, "gauge_1.png")],
        }
    }

    /// The sky inside the sky window (the caller has leaned the frame), scrolled left by
    /// `scroll` local px: the rendered plate, wrapping, or a procedural dusk gradient.
    pub fn draw_sky(&self, p: &mut Painter<'_>, g: &Geom, window: &[Vec2], scroll: f32) {
        let shape = Shape::Polygon { points: window };
        if let Some(image) = self.sky {
            let w = g.sky.width();
            let x = -scroll.rem_euclid(w.max(1.0));
            let rect = g.sky.translate(vec2(x, 0.0));
            p.fill(&shape, ImagePaint::new(image, rect).repeat());
            return;
        }
        p.fill(
            &shape,
            Gradient::linear(vec2(0.0, g.sky.min.y), vec2(0.0, g.horizon))
                .stop(0.0, rgb(SKY_TOP))
                .stop(0.7, rgb(SKY_MID))
                .stop(1.0, rgb(SKY_LOW)),
        );
    }

    /// The ring ribs, the road and the side-scale bands: the rendered pair, or the procedural
    /// one.
    pub fn draw_assembly(&self, p: &mut Painter<'_>, g: &Geom, swing: f32) {
        if !lit_plate(p, self.assembly, g.assembly_rect, swing) {
            fallback_bands(p, g);
            fallback_tunnel(p, g);
        }
    }

    /// The road again, over the lit rings (they light the walls, never the road): the plate's
    /// own road pixels, or the procedural road.
    pub fn draw_road(&self, p: &mut Painter<'_>, g: &Geom) {
        let f = g.floor();
        if let Some(base) = self.assembly[0] {
            p.fill(&Shape::Polygon { points: &f }, ImagePaint::new(base, g.assembly_rect));
            return;
        }
        fallback_road(p, g);
    }

    /// The crescents at `heat` of their light (dim at low revs, full near the shift point): the
    /// rendered halftone tinted by the primary, or a procedural body with the same light
    /// (brighter at the outer edge).
    pub fn draw_crescents(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, heat: f32) {
        let red = pal.primary.intensity(heat.clamp(0.0, 1.0));
        if let Some(image) = self.crescents {
            let rect = g.crescent_rect;
            p.fill(&Shape::rect(rect), ImagePaint::new(image, rect).tint(red));
            return;
        }
        let inner = (g.crescent_r - g.crescent_w) / g.crescent_r;
        let paint = Gradient::radial(g.tunnel, g.crescent_r)
            .stop(inner, red.intensity(0.45))
            .stop(1.0 - (1.0 - inner) * 0.2, red.intensity(0.82))
            .stop(1.0, red);
        for side in [-1.0, 1.0] {
            let mut pts = [Vec2::ZERO; CRESCENT_SAMPLES * 2];
            let n = crescent_polygon(g, 0.0, 1.0, side, &mut pts);
            p.fill(&Shape::Polygon { points: &pts[..n] }, paint);
        }
    }

    /// The G-Force lens body: the rendered pair, or a dark red disc.
    pub fn draw_lens(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, swing: f32) {
        let rect = g.widget_rect(g.g_widget, g.lens_margin);
        if !lit_plate(p, self.lens, rect, swing) {
            p.fill(
                &Shape::circle(g.g_widget, g.widget_r),
                Gradient::radial(g.g_widget, g.widget_r)
                    .stop(0.0, pal.primary.intensity(0.06))
                    .stop(0.7, pal.primary.intensity(0.16))
                    .stop(1.0, pal.primary.intensity(0.42)),
            );
        }
    }

    /// The boost gauge body: the rendered pair, or a dark face with a grey bezel.
    pub fn draw_gauge(&self, p: &mut Painter<'_>, g: &Geom, swing: f32) {
        let rect = g.widget_rect(g.boost_widget, g.gauge_margin);
        if !lit_plate(p, self.gauge, rect, swing) {
            let c = g.boost_widget;
            p.fill(
                &Shape::circle(c, g.widget_r),
                Gradient::radial(c - vec2(0.0, g.widget_r * 0.2), g.widget_r * 1.2)
                    .stop(0.0, Color::srgb8(25, 27, 32, 255))
                    .stop(1.0, Color::srgb8(10, 11, 13, 255)),
            );
            p.fill(
                &Shape::ring(c, g.widget_r, 2.5),
                rgb(RING_LINE).lerp(rgb(BAND_RIM), 0.6),
            );
        }
    }
}

/// The procedural side-scale bands: a dark plum-charcoal band, a silver rim on its outer edge
/// and a dim lip on its inner edge.
fn fallback_bands(p: &mut Painter<'_>, g: &Geom) {
    let w = g.band_out - g.band_in;
    let mid = (g.band_in + g.band_out) * 0.5;
    for side in [-1.0, 1.0] {
        let (start, sweep) = if side < 0.0 {
            (g.scale_from, g.scale_to - g.scale_from)
        } else {
            (deg(180.0) - g.scale_from, -(g.scale_to - g.scale_from))
        };
        p.fill(&Shape::arc(g.tunnel, mid, w, start, sweep, Cap::Butt), rgb(BAND_BODY));
        p.fill(
            &Shape::arc(g.tunnel, g.band_out, 2.6, start, sweep, Cap::Butt),
            rgb(BAND_RIM).intensity(0.9),
        );
        p.fill(
            &Shape::arc(g.tunnel, g.band_in, 1.4, start, sweep, Cap::Butt),
            rgb(BAND_LIP),
        );
    }
}

/// The procedural tunnel: the ring ribs (faint above the horizon, where the sky shows through
/// them) and the road.
fn fallback_tunnel(p: &mut Painter<'_>, g: &Geom) {
    let top = g.horizon - g.ceiling_fade;
    for k in 0..RINGS {
        let (c, r) = g.ring(k);
        let u = (k as f32 + 1.0) / RINGS as f32;
        let line = rgb(RING_LINE).intensity(0.9 - 0.3 * u);
        p.fill(
            &Shape::ring(c, r, 1.3 + 2.2 * u),
            Gradient::linear(vec2(0.0, top), vec2(0.0, g.horizon))
                .stop(0.0, line.fade(g.ceiling_floor))
                .stop(1.0, line),
        );
    }
    fallback_road(p, g);
}

/// The procedural road: lit from the horizon, with edge lines that fade towards the viewer.
fn fallback_road(p: &mut Painter<'_>, g: &Geom) {
    let f = g.floor();
    p.fill(
        &Shape::Polygon { points: &f },
        Gradient::linear(vec2(0.0, g.horizon), vec2(0.0, g.floor_bottom))
            .stop(0.0, rgb(ROAD_TOP))
            .stop(0.25, Color::srgb8(18, 20, 24, 255))
            .stop(1.0, Color::srgb8(5, 5, 6, 255)),
    );
    let edge = Gradient::linear(vec2(0.0, g.horizon), vec2(0.0, g.floor_bottom))
        .stop(0.0, rgb(ROAD_EDGE))
        .stop(1.0, rgb(ROAD_EDGE).intensity(0.2));
    p.fill(&Shape::line(f[0], f[3], 2.0), edge);
    p.fill(&Shape::line(f[1], f[2], 2.0), edge);
}

/// The outline of the left (`side` −1) or right crescent between fills `t0` and `t1`, at
/// `w_scale` of its width from the outer edge, into `out`; returns the number of points (0 for an
/// empty span).
pub(super) fn crescent_band(
    g: &Geom,
    t0: f32,
    t1: f32,
    w_scale: f32,
    side: f32,
    out: &mut [Vec2; CRESCENT_SAMPLES * 2],
) -> usize {
    let (t0, t1) = (t0.clamp(0.0, 1.0), t1.clamp(0.0, 1.0));
    if t1 - t0 < 1e-3 {
        return 0;
    }
    let n = CRESCENT_SAMPLES;
    for i in 0..n {
        let t = t0 + (t1 - t0) * i as f32 / (n - 1) as f32;
        out[i] = g.crescent_point(t, 0.0, side);
        out[2 * n - 1 - i] = g.crescent_point(t, w_scale, side);
    }
    2 * n
}

/// The whole outline of a crescent between fills `t0` and `t1`.
pub(super) fn crescent_polygon(g: &Geom, t0: f32, t1: f32, side: f32, out: &mut [Vec2; CRESCENT_SAMPLES * 2]) -> usize {
    crescent_band(g, t0, t1, 1.0, side, out)
}

/// True when every plate loaded (tests and the handover's asset check).
#[cfg(test)]
#[allow(dead_code)]
pub(super) fn all_loaded(plates: &Plates) -> bool {
    plates.sky.is_some()
        && plates.assembly.iter().all(Option::is_some)
        && plates.crescents.is_some()
        && plates.lens.iter().all(Option::is_some)
        && plates.gauge.iter().all(Option::is_some)
}
