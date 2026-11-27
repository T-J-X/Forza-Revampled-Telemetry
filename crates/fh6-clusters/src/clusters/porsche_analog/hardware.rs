//! The rendered hardware: Blender sprites from `assets/clusters/porsche_analog/` (recipe
//! `tools/asset-forge/src/recipes/porsche_analog.rs`), each rendered at several key-light
//! azimuths and cross-faded by the virtual light, and the procedural fallbacks drawn when a file
//! is missing.
//!
//! The rectangles below are the sprites' frames in local 4K px (1 image px per local px after the
//! recipe's box filter), round each part's anchor; keep them in step with the recipe.
use core::f32::consts::FRAC_PI_2;
use fh6_kit::cluster::Setup;
use fh6_kit::{vec2, Color, Gradient, ImageId, ImagePaint, Painter, Palette, Rect, Shape, Stroke, Vec2};

/// Key azimuths of the tube, hub and window renders (degrees, screen convention): rest, then the
/// virtual light's ±25° swing.
const TUBE_AZIMUTHS: usize = 3;
/// The needles are rendered every 45° all the way round (from −135°), because they turn under
/// the light: which facet catches it depends on the needle's angle.
const NEEDLE_AZIMUTHS: usize = 8;
const NEEDLE_STEP_DEG: f32 = 45.0;
/// The rest azimuth of the light and its swing (degrees).
const REST_DEG: f32 = -135.0;
const SWING_DEG: f32 = 25.0;

/// Sprite frames (local px round the part's anchor).
const TACH_SIDE: f32 = 512.0;
const SIDE_SIDE: f32 = 384.0;
const FAR_SIDE: f32 = 232.0;
const HUB_SIDE: f32 = 48.0;
/// The gear window's round rim (outer radius 76, plus a margin).
const WINDOW_SIDE: f32 = 164.0;
/// The needle sprite points up from its pivot: x −12..12, y −206..30 (tail 26, tip 200).
const NEEDLE_FRAME: (f32, f32, f32, f32) = (-12.0, -206.0, 24.0, 236.0);
/// The pointer sprite points up from its pivot: x −8..8, y −160..−122; its blade runs from 126
/// to 156.
const POINTER_FRAME: (f32, f32, f32, f32) = (-8.0, -160.0, 16.0, 38.0);
const POINTER_BLADE: (f32, f32) = (126.0, 156.0);

/// A static part.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Tach,
    Side,
    Far,
    Hub,
}

/// Every loaded sprite (`None` when its file is missing).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Sprites {
    tach: [Option<ImageId>; TUBE_AZIMUTHS],
    side: [Option<ImageId>; TUBE_AZIMUTHS],
    far: [Option<ImageId>; TUBE_AZIMUTHS],
    hub: [Option<ImageId>; TUBE_AZIMUTHS],
    window: [Option<ImageId>; TUBE_AZIMUTHS],
    needle: [Option<ImageId>; NEEDLE_AZIMUTHS],
    pointer: [Option<ImageId>; NEEDLE_AZIMUTHS],
}

/// Loads one file under `assets/` (path parts joined), when the assets folder is known and
/// holds it.
pub(super) fn load(setup: &mut Setup<'_>, parts: &[&str]) -> Option<ImageId> {
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
            // The dash still draws: every sprite has a procedural fallback.
            tracing::warn!(asset = %path.display(), %error, "a PorscheAnalog asset did not load");
            None
        }
    }
}

impl Sprites {
    /// Loads every sprite once.
    pub fn load(setup: &mut Setup<'_>) -> Self {
        fn set<const N: usize>(setup: &mut Setup<'_>, stem: &str) -> [Option<ImageId>; N] {
            let mut out = [None; N];
            for (k, slot) in out.iter_mut().enumerate() {
                let name = format!("{stem}_{k}.png");
                *slot = load(setup, &["clusters", "porsche_analog", &name]);
            }
            out
        }
        Self {
            tach: set(setup, "tach"),
            side: set(setup, "side"),
            far: set(setup, "far"),
            hub: set(setup, "hub"),
            window: set(setup, "window"),
            needle: set(setup, "needle"),
            pointer: set(setup, "pointer"),
        }
    }

    fn tube_set(&self, kind: Kind) -> (&[Option<ImageId>; TUBE_AZIMUTHS], f32) {
        match kind {
            Kind::Tach => (&self.tach, TACH_SIDE),
            Kind::Side => (&self.side, SIDE_SIDE),
            Kind::Far => (&self.far, FAR_SIDE),
            Kind::Hub => (&self.hub, HUB_SIDE),
        }
    }

    /// True when the rest render of `kind` loaded.
    pub fn has(&self, kind: Kind) -> bool {
        self.tube_set(kind).0[0].is_some()
    }

    /// Draws a static part centred at `c`, cross-faded with the light weights `w`. False when
    /// its file is missing (draw the fallback).
    pub fn draw(&self, p: &mut Painter<'_>, kind: Kind, c: Vec2, w: [f32; 3]) -> bool {
        let (set, side) = self.tube_set(kind);
        blend(p, set, Rect::from_center_size(c, Vec2::splat(side)), Color::WHITE, &w)
    }

    /// Draws the gear window's rendered rim centred at `c`.
    pub fn draw_window(&self, p: &mut Painter<'_>, c: Vec2, w: [f32; 3]) -> bool {
        blend(
            p,
            &self.window,
            Rect::from_center_size(c, Vec2::splat(WINDOW_SIDE)),
            Color::WHITE,
            &w,
        )
    }

    /// Draws the rev counter's needle about `pivot` at `angle`, tinted, lit for `light` (the
    /// virtual light's angle). False when its files are missing.
    pub fn draw_needle(&self, p: &mut Painter<'_>, pivot: Vec2, angle: f32, light: f32, tint: Color) -> bool {
        let (x, y, fw, fh) = NEEDLE_FRAME;
        let rect = Rect::from_xywh(pivot.x + x, pivot.y + y, fw, fh);
        turned(p, &self.needle, pivot, angle, light, rect, tint)
    }

    /// Draws a pointer about `pivot` at `angle`, its blade running from `root` to `tip` (the
    /// sprite is scaled down for the small tubes, never up). False when its files are missing.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_pointer(
        &self,
        p: &mut Painter<'_>,
        pivot: Vec2,
        angle: f32,
        light: f32,
        root: f32,
        tip: f32,
        tint: Color,
    ) -> bool {
        let rect = pointer_rect(pivot, root, tip);
        turned(p, &self.pointer, pivot, angle, light, rect, tint)
    }
}

/// The pointer sprite's rectangle (pointing up from `pivot`) that puts its blade on `root..tip`
/// from the pivot: the sprite is scaled down for a shorter blade, never up.
fn pointer_rect(pivot: Vec2, root: f32, tip: f32) -> Rect {
    let (x, y, fw, fh) = POINTER_FRAME;
    let k = ((tip - root) / (POINTER_BLADE.1 - POINTER_BLADE.0)).clamp(0.05, 1.0);
    // The frame's distances from the pivot, mapped so the blade lands on root..tip.
    let near = root - (POINTER_BLADE.0 + y + fh) * k;
    let far = root + (-y - POINTER_BLADE.0) * k;
    Rect::from_xywh(pivot.x + x * k, pivot.y - far, fw * k, far - near)
}

/// Draws `set[0]` over `rect`, then the swing renders on top at their weights.
fn blend(p: &mut Painter<'_>, set: &[Option<ImageId>], rect: Rect, tint: Color, w: &[f32]) -> bool {
    let Some(Some(base)) = set.first() else {
        return false;
    };
    p.fill(&Shape::rect(rect), ImagePaint::new(*base, rect).tint(tint));
    for (image, &weight) in set.iter().zip(w).skip(1) {
        if let Some(image) = image {
            if weight > 0.003 {
                p.scoped(|p| {
                    p.set_opacity(weight.min(1.0));
                    p.fill(&Shape::rect(rect), ImagePaint::new(*image, rect).tint(tint));
                });
            }
        }
    }
    true
}

/// Draws a sprite that points up from `pivot`, turned to `angle`, picking and cross-fading the
/// two renders whose key azimuth (in the sprite's own frame) is nearest the light's.
fn turned(
    p: &mut Painter<'_>,
    set: &[Option<ImageId>; NEEDLE_AZIMUTHS],
    pivot: Vec2,
    angle: f32,
    light: f32,
    rect: Rect,
    tint: Color,
) -> bool {
    if set.iter().any(Option::is_none) {
        return false;
    }
    let turn = angle + FRAC_PI_2;
    let w = needle_weights(light - turn);
    p.scoped(|p| {
        p.rotate_about(turn, pivot);
        let mut first = true;
        for (k, image) in set.iter().enumerate() {
            let (Some(image), weight) = (image, w[k]) else {
                continue;
            };
            if weight <= 0.003 {
                continue;
            }
            p.scoped(|p| {
                if !first {
                    p.set_opacity(weight.min(1.0));
                }
                p.fill(&Shape::rect(rect), ImagePaint::new(*image, rect).tint(tint));
            });
            first = false;
        }
    });
    true
}

/// Weights of the tube renders (rest, −25°, +25°) for the virtual light's `angle` (radians):
/// the rest render always at 1, a swing render on top in proportion to the swing.
pub(super) fn light_weights(angle: f32) -> [f32; 3] {
    let d = (angle.to_degrees() - REST_DEG) / SWING_DEG;
    if !d.is_finite() {
        return [1.0, 0.0, 0.0];
    }
    let d = d.clamp(-1.0, 1.0);
    [1.0, (-d).max(0.0), d.max(0.0)]
}

/// Weights of the eight needle renders for the light at `relative` radians in the needle's own
/// frame: the lower of the two neighbours at full weight, the upper on top at the fraction
/// between them (so the sum on screen is their linear blend).
pub(super) fn needle_weights(relative: f32) -> [f32; NEEDLE_AZIMUTHS] {
    let mut w = [0.0; NEEDLE_AZIMUTHS];
    let rel = if relative.is_finite() {
        relative
    } else {
        REST_DEG.to_radians()
    };
    let steps = ((rel.to_degrees() - REST_DEG) / NEEDLE_STEP_DEG).rem_euclid(NEEDLE_AZIMUTHS as f32);
    let lo = (steps.floor() as usize) % NEEDLE_AZIMUTHS;
    let hi = (lo + 1) % NEEDLE_AZIMUTHS;
    let f = steps - steps.floor();
    w[lo] = 1.0;
    w[hi] = f;
    // Draw order is by index, so when the upper neighbour comes first it must carry the base.
    if hi < lo {
        w[hi] = 1.0;
        w[lo] = 1.0 - f;
    }
    w
}

/// A procedural tube: a bezel lit from the upper left with a bright inner chamfer, a black dial
/// and the raised scale ring's edge.
pub(super) fn fallback_tube(p: &mut Painter<'_>, c: Vec2, open: f32, outer: f32, scale_in: f32, pal: &Palette) {
    let ink = pal.roles.ink;
    let mid = (open + outer) * 0.5;
    // Opaque, so a tube tucked behind is hidden where this one covers it.
    p.fill(&Shape::circle(c, outer), pal.roles.oled_black);
    p.fill(
        &Shape::ring(c, mid, (outer - open).max(1.0)),
        Gradient::linear(c + vec2(-outer, -outer), c + vec2(outer, outer))
            .stop(0.0, ink.with_alpha(0.22))
            .stop(0.5, ink.with_alpha(0.07))
            .stop(1.0, ink.with_alpha(0.02)),
    );
    p.fill(&Shape::circle(c, open), pal.roles.oled_black);
    p.stroke(&Shape::circle(c, open + 1.0), ink.with_alpha(0.6), Stroke::rim_px(1.5));
    p.stroke(
        &Shape::circle(c, outer - 0.5),
        ink.with_alpha(0.18),
        Stroke::rim_px(1.0),
    );
    if scale_in > 0.0 {
        p.stroke(&Shape::circle(c, scale_in), ink.with_alpha(0.10), Stroke::rim_px(1.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::TAU;

    #[test]
    fn tube_weights_follow_the_swing() {
        assert_eq!(light_weights(REST_DEG.to_radians()), [1.0, 0.0, 0.0]);
        let left = light_weights((REST_DEG - SWING_DEG).to_radians());
        assert!((left[1] - 1.0).abs() < 1e-5 && left[2] == 0.0);
        let half = light_weights((REST_DEG + SWING_DEG * 0.5).to_radians());
        assert!((half[2] - 0.5).abs() < 1e-4 && half[1] == 0.0);
        assert_eq!(light_weights(f32::NAN), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn needle_weights_pick_the_two_nearest_renders() {
        // Exactly on render 0 (the rest azimuth in the needle's frame).
        let w = needle_weights(REST_DEG.to_radians());
        assert!((w[0] - 1.0).abs() < 1e-5 && w[1] < 1e-5);
        // Halfway between renders 2 and 3.
        let w = needle_weights((REST_DEG + NEEDLE_STEP_DEG * 2.5).to_radians());
        assert!((w[2] - 1.0).abs() < 1e-5 && (w[3] - 0.5).abs() < 1e-4);
        // Between the last render and the first: the base goes to the first in draw order.
        let w = needle_weights((REST_DEG + NEEDLE_STEP_DEG * 7.25).to_radians());
        assert!((w[0] - 1.0).abs() < 1e-5 && (w[7] - 0.75).abs() < 1e-4);
        // A full turn later, the same weights.
        let a = needle_weights(1.0);
        let b = needle_weights(1.0 + TAU);
        for k in 0..NEEDLE_AZIMUTHS {
            assert!((a[k] - b[k]).abs() < 1e-4);
        }
    }

    #[test]
    fn the_scaled_pointer_lands_its_blade_on_root_and_tip() {
        // In the sprite's own scale the frame is 122..160 from the pivot, the blade 126..156.
        let (_, y, _, fh) = POINTER_FRAME;
        assert!((-(y + fh) - 122.0).abs() < 1e-5 && (-y - 160.0).abs() < 1e-5);
        let geom = &super::super::GEOM;
        let pivot = vec2(10.0, 20.0);
        for (root, tip) in [geom.pointer, geom.far_pointer] {
            let r = pointer_rect(pivot, root, tip);
            // The blade's place inside the frame, scaled with it, lands on root..tip.
            let k = r.height() / fh;
            let blade_root = (pivot.y - r.max.y) + (POINTER_BLADE.0 + y + fh) * k;
            let blade_tip = (pivot.y - r.min.y) - (-y - POINTER_BLADE.1) * k;
            assert!((blade_root - root).abs() < 1e-3, "root {blade_root} for {root}");
            assert!((blade_tip - tip).abs() < 1e-3, "tip {blade_tip} for {tip}");
            // Centred on the pivot's line, and never scaled up.
            assert!((r.center().x - pivot.x).abs() < 1e-3 && k <= 1.0 + 1e-6);
        }
    }

    #[test]
    fn the_sprite_frames_match_the_zone_table() {
        let g = &super::super::GEOM;
        // Each tube sprite is the part's outer radius and 8 px of shadow round it.
        assert_eq!(TACH_SIDE, 2.0 * (g.tach_outer + 8.0));
        assert_eq!(SIDE_SIDE, 2.0 * (g.side_outer + 8.0));
        assert_eq!(FAR_SIDE, 2.0 * (g.far_outer + 8.0));
        assert_eq!(WINDOW_SIDE, 2.0 * (g.porthole_rim + 6.0));
        assert!(HUB_SIDE >= 2.0 * g.hub_r);
        // The needle sprite: its tip 6 px short of the frame's top, its tail 4 px short of the
        // bottom, as rendered.
        let (_, y, _, fh) = NEEDLE_FRAME;
        assert_eq!(-y, g.needle_tip + 6.0);
        assert_eq!(y + fh, g.needle_tail + 4.0);
        // The side tubes use the pointer's own blade; the outer tubes scale it down.
        assert_eq!(POINTER_BLADE, g.pointer);
        assert!(g.far_pointer.1 - g.far_pointer.0 <= POINTER_BLADE.1 - POINTER_BLADE.0);
    }
}
