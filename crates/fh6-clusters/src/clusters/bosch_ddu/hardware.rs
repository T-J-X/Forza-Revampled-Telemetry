//! The rendered housing: Blender sprites from `assets/clusters/bosch_ddu/` (recipe
//! `tools/asset-forge/src/recipes/bosch_ddu.rs`), rendered at three key-light azimuths and
//! cross-faded by the virtual light, and the procedural fallback drawn when a file is missing.
//!
//! [`HOUSING_FRAME`] is the sprites' frame in local 4K px round the cockpit anchor (1 image px per
//! local px after the recipe's box filter); keep it in step with the recipe.
use super::Geom;
use fh6_kit::cluster::Setup;
use fh6_kit::{vec2, Color, Gradient, ImageId, ImagePaint, Painter, Palette, Radii, Rect, Shadow, Shape, Stroke, Vec2};

/// Key azimuths of the housing renders: rest, then the virtual light's swing either way.
pub(super) const AZIMUTHS: usize = 3;
/// The rest azimuth of the light and its swing (degrees, screen convention).
const REST_DEG: f32 = -135.0;
const SWING_DEG: f32 = 25.0;

/// The housing sprite's frame (local px): the housing (x ±405, y −316..232) with room for its
/// side walls and a little of its contact shadow, inside the cockpit envelope (y −320..240).
pub(super) const HOUSING_FRAME: Rect = Rect {
    min: Vec2 { x: -440.0, y: -320.0 },
    max: Vec2 { x: 440.0, y: 240.0 },
};

/// Linear levels of the procedural fallback (over OLED black): the anodised side wall at its lit
/// top and its shaded foot, and the black glass face's sheen under the key light.
const WALL_TOP: f32 = 0.075;
const WALL_FOOT: f32 = 0.022;
const FACE_SHEEN: f32 = 0.035;
/// Width of the anodised wall seen round the black glass face (px).
const WALL: f32 = 6.0;

/// The loaded housing renders (`None` when a file is missing).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Sprites {
    housing: [Option<ImageId>; AZIMUTHS],
}

/// Loads one file under `assets/` (path parts joined), when the assets folder is known and
/// holds it.
fn load(setup: &mut Setup<'_>, parts: &[&str]) -> Option<ImageId> {
    let mut path = setup.assets?.to_path_buf();
    for part in parts {
        path.push(part);
    }
    if !path.is_file() {
        return None;
    }
    match setup.images.load_file(setup.device, setup.queue, &path, None) {
        Ok(image) => Some(image),
        Err(error) => {
            // The dash still draws: the housing has a procedural fallback.
            tracing::warn!(asset = %path.display(), %error, "a BoschDdu asset did not load");
            None
        }
    }
}

impl Sprites {
    /// Loads every render once.
    pub fn load(setup: &mut Setup<'_>) -> Self {
        let mut housing = [None; AZIMUTHS];
        for (k, slot) in housing.iter_mut().enumerate() {
            let name = format!("housing_{k}.png");
            *slot = load(setup, &["clusters", "bosch_ddu", &name]);
        }
        Self { housing }
    }

    /// True when the housing render loaded (its lenses are then rendered hardware).
    pub fn loaded(&self) -> bool {
        self.housing[0].is_some()
    }

    /// Draws the housing, cross-faded with the light weights `w`. False when the rest render is
    /// missing (draw the fallback).
    pub fn draw(&self, p: &mut Painter<'_>, w: [f32; AZIMUTHS]) -> bool {
        let Some(base) = self.housing[0] else {
            return false;
        };
        let rect = HOUSING_FRAME;
        p.fill(&Shape::rect(rect), ImagePaint::new(base, rect));
        for (image, &weight) in self.housing.iter().zip(&w).skip(1) {
            if let Some(image) = image {
                if weight > 0.003 {
                    p.scoped(|p| {
                        p.set_opacity(weight.min(1.0));
                        p.fill(&Shape::rect(rect), ImagePaint::new(*image, rect));
                    });
                }
            }
        }
        true
    }
}

/// The weights of the three renders for the virtual light at `angle` (radians): the rest render
/// at full strength and the swing render on the light's side on top at the swing's fraction.
pub(super) fn light_weights(angle: f32) -> [f32; AZIMUTHS] {
    let d = (angle.to_degrees() - REST_DEG) / SWING_DEG;
    if !d.is_finite() {
        return [1.0, 0.0, 0.0];
    }
    let d = d.clamp(-1.0, 1.0);
    [1.0, (-d).max(0.0), d.max(0.0)]
}

/// The procedural housing: an anodised body with a black glass face edge to edge, its arched
/// top, large top corners and small bottom ones, lit from the key light's side (`swing` −1..1).
pub(super) fn fallback(p: &mut Painter<'_>, g: &Geom, pal: &Palette, swing: f32) {
    let body = g.body();
    let radii = Radii {
        tl: g.corner_top,
        tr: g.corner_top,
        br: g.corner_bottom,
        bl: g.corner_bottom,
    };
    // The contact shadow under the housing.
    p.shadow(
        &Shape::round_rect(body, radii),
        Shadow::new(vec2(0.0, 10.0), 18.0, pal.roles.oled_black.with_alpha(0.9)),
    );
    let wall = Gradient::linear(vec2(0.0, body.min.y - g.arch), vec2(0.0, body.max.y))
        .stop(0.0, Color::linear(WALL_TOP, WALL_TOP, WALL_TOP * 1.04, 1.0))
        .stop(
            0.35,
            Color::linear(WALL_TOP * 0.55, WALL_TOP * 0.55, WALL_TOP * 0.58, 1.0),
        )
        .stop(1.0, Color::linear(WALL_FOOT, WALL_FOOT, WALL_FOOT, 1.0));
    p.fill(&Shape::round_rect(body, radii), wall);
    arch(p, g, 0.0, wall.into());

    // The black glass face, inset by the visible wall.
    let face = Rect::from_points(body.min + Vec2::splat(WALL), body.max - Vec2::splat(WALL));
    let face_radii = Radii {
        tl: g.corner_top - WALL,
        tr: g.corner_top - WALL,
        br: g.corner_bottom - WALL,
        bl: g.corner_bottom - WALL,
    };
    let black = pal.roles.oled_black;
    p.fill(&Shape::round_rect(face, face_radii), black);
    arch(p, g, WALL, black.into());
    // The key light's sheen on the glass brow, leaning to its side.
    let sheen = Color::linear(FACE_SHEEN, FACE_SHEEN, FACE_SHEEN * 1.05, 1.0);
    let lean = swing * 120.0;
    let gloss = Gradient::radial(vec2(lean - 140.0, face.min.y - 40.0), 420.0)
        .stop(0.0, sheen)
        .stop(0.55, sheen.with_alpha(0.35))
        .stop(1.0, sheen.with_alpha(0.0));
    p.fill(&Shape::round_rect(face, face_radii), gloss);
    arch(p, g, WALL, gloss.into());
    // The glass edge's highlight, crisp and thin.
    p.stroke(
        &Shape::round_rect(face, face_radii),
        pal.roles.rim_specular.fade(0.14),
        Stroke::inner(1.0),
    );
    // The ambient light sensor in the right border.
    p.fill(
        &Shape::circle(g.sensor, g.sensor_r),
        Color::linear(0.006, 0.006, 0.007, 1.0),
    );
    p.stroke(
        &Shape::circle(g.sensor, g.sensor_r),
        pal.roles.rim_specular.fade(0.18),
        Stroke::outer(1.0),
    );
}

/// The shallow arch over the top edge between the top corners, `inset` px inside the outline.
fn arch(p: &mut Painter<'_>, g: &Geom, inset: f32, paint: fh6_kit::Paint) {
    let body = g.body();
    let span = body.width() * 0.5 - g.corner_top;
    // The circle through the arch's crest and its ends (chord 2 × span, sagitta `arch`).
    let r = (span * span + g.arch * g.arch) / (2.0 * g.arch);
    let crest = body.min.y - g.arch + inset;
    p.scoped(|p| {
        p.clip(Rect::from_xywh(-span, crest - 1.0, span * 2.0, g.arch + 2.0), 0.0);
        p.fill(&Shape::circle(vec2(0.0, crest + r), r), paint);
    });
}
