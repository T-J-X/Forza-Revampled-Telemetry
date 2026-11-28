//! The rendered plates: Blender renders from `assets/clusters/amg_mbux_superscreen/` (recipe
//! `tools/asset-forge/src/recipes/amg_mbux_superscreen.rs`) and the procedural fallbacks drawn
//! when a file is missing.
//!
//! - `housing_0.png`, `housing_1.png`: the display unit (piano-black bezel, the cover glass's
//!   edge and the black screen), with the key light at rest and swung; cross-faded by lateral G.
//!   They cover [`Geom::housing`] 1:1.
//! - `tunnel.png`: the receding rings and the floor of the tunnel in luminance, tinted by the
//!   palette's primary at draw time, so a custom palette recolours it. It covers
//!   [`Geom::tunnel_rect`] 1:1.
use super::geom::{Geom, RINGS};
use super::RING_LEVEL;
use fh6_kit::cluster::Setup;
use fh6_kit::{vec2, Gradient, ImageId, ImagePaint, Painter, Palette, Shape, Stroke};

/// Light azimuths of the housing renders: rest, then the light swung by lateral G.
pub(super) const HOUSING_AZIMUTHS: usize = 2;
/// The asset folder of this dash.
const FOLDER: &str = "amg_mbux_superscreen";

/// Every loaded plate (`None` when its file is missing).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Plates {
    housing: [Option<ImageId>; HOUSING_AZIMUTHS],
    tunnel: Option<ImageId>,
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

impl Plates {
    /// Loads every plate once.
    pub fn load(setup: &mut Setup<'_>) -> Self {
        Self {
            housing: [load(setup, "housing_0.png"), load(setup, "housing_1.png")],
            tunnel: load(setup, "tunnel.png"),
        }
    }

    /// The display unit: the rendered housing, cross-faded to the swung light by `swing`
    /// (0..1), or the procedural one.
    pub fn draw_housing(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, swing: f32) {
        let rect = g.housing;
        if let Some(base) = self.housing[0] {
            p.fill(&Shape::rect(rect), ImagePaint::new(base, rect));
            if let Some(swung) = self.housing[1] {
                let w = swing.clamp(0.0, 1.0);
                if w > 0.003 {
                    p.scoped(|p| {
                        p.set_opacity(w);
                        p.fill(&Shape::rect(rect), ImagePaint::new(swung, rect));
                    });
                }
            }
            return;
        }
        fallback_housing(p, g, pal);
    }

    /// The tunnel's receding rings and floor: the rendered plate tinted by the primary, or the
    /// procedural one.
    pub fn draw_tunnel(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        if let Some(image) = self.tunnel {
            let rect = g.tunnel_rect();
            p.fill(&Shape::rect(rect), ImagePaint::new(image, rect).tint(pal.primary));
        } else {
            fallback_rings(p, g, pal);
        }
        draw_floor(p, g, pal);
    }
}

/// A procedural display unit: an opaque piano-black bezel lit from above, the black glass inside
/// it and the cover glass's thin bright edge.
fn fallback_housing(p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
    let ink = pal.roles.ink;
    let h = g.housing;
    // Opaque, so the game's own dash never shows through the unit.
    p.fill(&Shape::round_rect(h, g.housing_r), pal.roles.oled_black);
    p.fill(
        &Shape::round_rect(h, g.housing_r),
        Gradient::linear(vec2(0.0, h.min.y), vec2(0.0, h.max.y))
            .stop(0.0, ink.intensity(0.050))
            .stop(0.12, ink.intensity(0.018))
            .stop(1.0, ink.intensity(0.006)),
    );
    p.fill(&Shape::round_rect(g.screen, g.screen_r), pal.roles.oled_black);
    // The cover glass's edge catches the light along the top.
    p.stroke(
        &Shape::round_rect(h.inset(1.0), g.housing_r - 1.0),
        Gradient::linear(vec2(0.0, h.min.y), vec2(0.0, h.max.y))
            .stop(0.0, ink.intensity(0.16))
            .stop(0.5, ink.intensity(0.05))
            .stop(1.0, ink.intensity(0.03)),
        Stroke::rim_px(1.5),
    );
    p.stroke(
        &Shape::round_rect(g.screen, g.screen_r),
        ink.intensity(0.035),
        Stroke::rim_px(1.0),
    );
}

/// The procedural rings: [`RINGS`] receding rings dimming towards the disc.
fn fallback_rings(p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
    for k in 0..RINGS {
        let (c, r) = g.ring(k);
        let depth = k as f32 / (RINGS as f32 - 1.0).max(1.0);
        let level = RING_LEVEL * (1.0 - 0.55 * depth);
        let width = 3.0 - 1.5 * depth;
        p.fill(&Shape::ring(c, r, width), pal.primary.intensity(level));
    }
}

/// The floor, over the rings (rendered or procedural): a dark wedge with its edges.
fn draw_floor(p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
    // The floor: a dark wedge that hides the rings behind it, lit faintly along its top edge,
    // with red edges running down from the disc's side.
    let f = g.floor();
    p.fill(&Shape::Polygon { points: &f }, pal.roles.oled_black);
    p.fill(
        &Shape::Polygon { points: &f },
        Gradient::linear(vec2(0.0, g.floor_top), vec2(0.0, g.floor_bottom))
            .stop(0.0, pal.primary.intensity(0.07))
            .stop(0.35, pal.primary.intensity(0.015))
            .stop(1.0, pal.primary.intensity(0.0)),
    );
    let edge = pal.primary.intensity(RING_LEVEL * 1.6);
    p.fill(&Shape::line(f[0], f[3], 2.0), edge);
    p.fill(&Shape::line(f[1], f[2], 2.0), edge);
    p.fill(&Shape::line(f[0], f[1], 2.0), edge.intensity(0.8));
}

/// True when every plate loaded (tests and the handover's asset check).
#[cfg(test)]
#[allow(dead_code)]
pub(super) fn all_loaded(plates: &Plates) -> bool {
    plates.housing.iter().all(Option::is_some) && plates.tunnel.is_some()
}

/// The procedural fallbacks never draw transparent black or a non-finite light.
#[cfg(test)]
pub(super) fn fallback_levels() -> [f32; RINGS] {
    let mut out = [0.0; RINGS];
    for (k, level) in out.iter_mut().enumerate() {
        let depth = k as f32 / (RINGS as f32 - 1.0).max(1.0);
        *level = RING_LEVEL * (1.0 - 0.55 * depth);
    }
    out
}
