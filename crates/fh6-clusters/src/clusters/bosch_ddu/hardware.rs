//! The rendered housing: Blender sprites from `assets/clusters/bosch_ddu/` (recipe
//! `tools/asset-forge/src/recipes/bosch_ddu.rs`), rendered at three key-light azimuths and
//! cross-faded by the virtual light, and the procedural fallback drawn when a file is missing.
//!
//! [`HOUSING_FRAME`] is the sprites' frame in local 4K px round the cockpit anchor (1 image px per
//! local px after the recipe's box filter); keep it in step with the recipe.
use super::Geom;
use fh6_kit::cluster::Setup;
use fh6_kit::{
    vec2, Color, Gradient, ImageId, ImagePaint, Paint, Painter, Palette, Radii, Rect, Shadow, Shape, Stroke, Vec2,
};

/// Key azimuths of the housing renders: rest, then the virtual light's swing either way.
const AZIMUTHS: usize = 3;
/// The rest azimuth of the light and its swing (degrees, screen convention): the values
/// `common::VirtualLight` moves between (a test holds the two together, at the swing's ends and
/// half way).
const REST_DEG: f32 = -135.0;
const SWING_DEG: f32 = 25.0;
/// A swing render under this weight is not drawn (under one 8-bit step).
const MIN_WEIGHT: f32 = 0.003;

/// The housing sprite's frame (local px): the housing (x ±405, y −316..232) with room for its
/// side walls, inside the cockpit envelope (y −320..240). The renders hold no shadow: the body
/// is opaque to its antialiased edge, so the cross-fade (source-over, exact where both renders
/// are opaque) never thickens it, and [`contact_shadow`] draws the shadow under either path.
pub(super) const HOUSING_FRAME: Rect = Rect {
    min: Vec2 { x: -440.0, y: -320.0 },
    max: Vec2 { x: 440.0, y: 240.0 },
};

/// Linear levels of the procedural fallback (over OLED black): the anodised side wall at its lit
/// top and its shaded foot, the black glass face's sheen under the key light, and the dark glass
/// of the light sensor's window.
const WALL_TOP: f32 = 0.075;
const WALL_FOOT: f32 = 0.022;
const FACE_SHEEN: f32 = 0.035;
const SENSOR_GLASS: f32 = 0.006;
/// The sheen is a broad pool on the brow from a circle of this radius whose lowest point stays
/// this far above the screen, so the screen is true black whatever the light does.
const SHEEN_R: f32 = 600.0;
const SHEEN_CLEAR: f32 = 4.0;
/// Width of the anodised wall seen round the black glass face (px).
const WALL: f32 = 6.0;
/// The arch's fill and its rim run this far (px) under the top edge they bridge, so no seam
/// shows where the arch meets the body. A translucent paint must not be laid under the arch
/// again in that row.
const ARCH_LAP: f32 = 1.0;
/// Between the top corners the glass edge's highlight starts this far (px) under the face's top
/// edge, clear of its 1 px stroke there, where the arch's own rim runs instead.
const TOP_EDGE_CLEAR: f32 = WALL;

/// The loaded housing renders (`None` when a file is missing).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Sprites {
    housing: [Option<ImageId>; AZIMUTHS],
}

/// Loads one file under `assets/` (path parts joined), when the assets folder is known and
/// holds it.
fn load_asset(setup: &mut Setup<'_>, parts: &[&str]) -> Option<ImageId> {
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
            *slot = load_asset(setup, &["clusters", "bosch_ddu", &name]);
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
                if weight > MIN_WEIGHT {
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
    let d = ((angle.to_degrees() - REST_DEG) / SWING_DEG).clamp(-1.0, 1.0);
    [1.0, (-d).max(0.0), d.max(0.0)]
}

/// The body's corner radii, `inset` px inside the outline.
fn corner_radii(g: &Geom, inset: f32) -> Radii {
    Radii {
        tl: g.corner_top - inset,
        tr: g.corner_top - inset,
        br: g.corner_bottom - inset,
        bl: g.corner_bottom - inset,
    }
}

/// The contact shadow under the housing, drawn before the rendered housing or the fallback.
pub(super) fn contact_shadow(p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
    p.shadow(
        &Shape::round_rect(g.body(), corner_radii(g, 0.0)),
        Shadow::new(vec2(0.0, 10.0), 18.0, pal.roles.oled_black.with_alpha(0.9)),
    );
}

/// The procedural housing: an anodised body with a black glass face edge to edge, its arched
/// top, large top corners and small bottom ones, lit from the key light's side (`swing` −1..1).
pub(super) fn fallback(p: &mut Painter<'_>, g: &Geom, pal: &Palette, swing: f32) {
    let body = g.body();
    let radii = corner_radii(g, 0.0);
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
    let face_radii = corner_radii(g, WALL);
    let black = pal.roles.oled_black;
    p.fill(&Shape::round_rect(face, face_radii), black);
    arch(p, g, WALL, black.into());
    // The key light's sheen on the glass brow, leaning to its side. It is the lower edge of a
    // large circle that ends above the screen, so no sheen reaches the screen (true black
    // there) and no edge shows where it ends.
    let sheen = Color::linear(FACE_SHEEN, FACE_SHEEN, FACE_SHEEN * 1.05, 1.0);
    let lean = swing * 120.0;
    let lowest = g.screen.min.y - SHEEN_CLEAR;
    let gloss = Gradient::radial(vec2(lean - 140.0, lowest - SHEEN_R), SHEEN_R)
        .stop(0.0, sheen)
        .stop(0.84, sheen)
        .stop(0.93, sheen.with_alpha(0.4))
        .stop(1.0, sheen.with_alpha(0.0));
    // The gloss is translucent, so the face's part of it stops where the arch's begins: between
    // the corners it starts under the row the arch's fill laps over. Laid twice, that row shows
    // as a faint line across the brow.
    let span = g.arch_span();
    let outline = Shape::round_rect(face, face_radii);
    for part in [
        Rect::from_points(vec2(face.min.x, face.min.y), vec2(-span, lowest)),
        Rect::from_points(vec2(span, face.min.y), vec2(face.max.x, lowest)),
        Rect::from_points(vec2(-span, face.min.y + ARCH_LAP), vec2(span, lowest)),
    ] {
        p.scoped(|p| {
            p.clip(part, 0.0);
            p.fill(&outline, gloss);
        });
    }
    arch(p, g, WALL, gloss.into());
    // The glass edge's highlight, crisp and thin: the face's outline without the flat top edge
    // between the corners, where the arch's own edge runs instead.
    let rim = pal.roles.rim_specular.fade(0.14);
    for part in [
        Rect::from_points(vec2(face.min.x, face.min.y), vec2(-span, face.max.y)),
        Rect::from_points(vec2(span, face.min.y), vec2(face.max.x, face.max.y)),
        Rect::from_points(vec2(-span, face.min.y + TOP_EDGE_CLEAR), vec2(span, face.max.y)),
    ] {
        p.scoped(|p| {
            p.clip(part, 0.0);
            p.stroke(&outline, rim, Stroke::inner(1.0));
        });
    }
    // The arch's rim: its clip ends [`ARCH_LAP`] px under the arch's ends, where the 1 px inner
    // stroke still runs: a clip that ended at the ends would cut the rim away over its last 15 px
    // each side.
    let (clip, circle) = arch_parts(g, WALL);
    p.scoped(|p| {
        p.clip(clip, 0.0);
        p.stroke(&circle, rim, Stroke::inner(1.0));
    });
    // The ambient light sensor in the right border.
    p.fill(
        &Shape::circle(g.sensor, g.sensor_r),
        Color::linear(SENSOR_GLASS, SENSOR_GLASS, SENSOR_GLASS * 1.17, 1.0),
    );
    p.stroke(
        &Shape::circle(g.sensor, g.sensor_r),
        pal.roles.rim_specular.fade(0.18),
        Stroke::outer(1.0),
    );
}

/// The arch `inset` px inside the outline: the clip that keeps it between the top corners (from
/// a pixel over its crest to [`ARCH_LAP`] px under its ends) and the circle through its crest and
/// its ends (chord 2 × span, sagitta `arch`).
fn arch_parts(g: &Geom, inset: f32) -> (Rect, Shape<'static>) {
    let (span, r) = (g.arch_span(), g.arch_radius());
    let crest = g.body().min.y - g.arch + inset;
    (
        Rect::from_xywh(-span, crest - 1.0, span * 2.0, g.arch + 1.0 + ARCH_LAP),
        Shape::circle(vec2(0.0, crest + r), r),
    )
}

/// The shallow arch over the top edge between the top corners, `inset` px inside the outline.
fn arch(p: &mut Painter<'_>, g: &Geom, inset: f32, paint: Paint) {
    let (clip, circle) = arch_parts(g, inset);
    p.scoped(|p| {
        p.clip(clip, 0.0);
        p.fill(&circle, paint);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::VirtualLight;
    use fh6_kit::Tokens;

    #[test]
    fn the_renders_follow_the_virtual_light() {
        // The rest render alone at rest, a full swing render at 1.5 g either way and a half one
        // at 0.75 g, where nothing clamps: REST_DEG and SWING_DEG are the virtual light's own
        // values (a smaller or larger swing fails the half way point).
        let t = Tokens::default();
        let mut light = VirtualLight::new(&t);
        assert_eq!(light_weights(light.angle()), [1.0, 0.0, 0.0]);
        for (g_lat, side, weight) in [(1.5, 2, 1.0), (-1.5, 1, 1.0), (0.75, 2, 0.5), (-0.75, 1, 0.5)] {
            for _ in 0..600 {
                light.update(g_lat, 1.0 / 60.0);
            }
            let w = light_weights(light.angle());
            assert!(
                (w[side] - weight).abs() < 0.01 && w[3 - side] == 0.0,
                "{g_lat} g: {w:?}"
            );
        }
    }
}
