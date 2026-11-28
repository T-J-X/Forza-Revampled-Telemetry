//! The rendered hardware: the Blender plate and the suggested-gear tile from
//! `assets/clusters/gt7_hood_pro/` (recipe `tools/asset-forge/src/recipes/gt7_hood_pro.rs`),
//! each rendered at the virtual light's rest azimuth and its ±25° swing and cross-faded by the
//! light, and the procedural fallbacks drawn when a file is missing.
use super::geom::{rect, Geom};
use fh6_kit::cluster::Setup;
use fh6_kit::{vec2, BlendMode, Cap, Color, ImageId, ImagePaint, Painter, Palette, Rect, Shape, Stroke, Vec2};

/// Key azimuths of each sprite: rest, then the light's swing to each side.
const AZIMUTHS: usize = 3;
/// The light's rest azimuth and its swing (degrees, screen convention).
const REST_DEG: f32 = -135.0;
const SWING_DEG: f32 = 25.0;
/// How dark the fallback's hardware is: the share of the ink's light in its faces.
const FALLBACK_FACE: f32 = 0.045;
/// The fallback's rims: the share of the ink's light.
const FALLBACK_RIM: f32 = 0.16;
/// The fallback's unlit LED lenses.
const FALLBACK_LENS: f32 = 0.07;

/// The loaded sprites (`None` where a file is missing).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Sprites {
    plate: [Option<ImageId>; AZIMUTHS],
    tile: [Option<ImageId>; AZIMUTHS],
}

/// Loads `assets/clusters/gt7_hood_pro/<name>` when the assets folder is known and holds it.
fn load(setup: &mut Setup<'_>, name: &str) -> Option<ImageId> {
    let path = setup.assets?.join("clusters").join("gt7_hood_pro").join(name);
    if !path.is_file() {
        return None;
    }
    match setup.images.load_file(setup.device, setup.queue, &path, None) {
        Ok(image) => Some(image),
        Err(error) => {
            // The dash still draws: every sprite has a procedural fallback.
            tracing::warn!(asset = %path.display(), %error, "a Gt7HoodPro asset did not load");
            None
        }
    }
}

impl Sprites {
    /// Loads every sprite once.
    pub fn load(setup: &mut Setup<'_>) -> Self {
        let mut s = Self::default();
        for k in 0..AZIMUTHS {
            s.plate[k] = load(setup, &format!("plate_{k}.png"));
            s.tile[k] = load(setup, &format!("suggest_{k}.png"));
        }
        s
    }

    /// Draws the plate over its frame, cross-faded with the light weights `w`. False when its
    /// rest render is missing (draw [`fallback_plate`]).
    pub fn draw_plate(&self, p: &mut Painter<'_>, g: &Geom, w: [f32; AZIMUTHS]) -> bool {
        blend(p, &self.plate, g.plate_rect(), &w)
    }

    /// Draws the suggested-gear tile over its frame. False when its rest render is missing.
    pub fn draw_tile(&self, p: &mut Painter<'_>, g: &Geom, w: [f32; AZIMUTHS]) -> bool {
        blend(p, &self.tile, rect(g.sug_frame), &w)
    }
}

/// Draws `set[0]` over `rect` and the swing renders at their weights, so the weights sum to one
/// coverage: the rest render at `1 - swing` and each swing render added at its own weight. A plain
/// cross-fade over the rest render counted every anti-aliased edge twice during a swing, so the
/// rims thickened and the tile's bloom doubled (the code audit, RL-1 and RL-2). Nothing of the
/// dash is drawn under the plate or the tile, so adding their light is exact.
fn blend(p: &mut Painter<'_>, set: &[Option<ImageId>; AZIMUTHS], rect: Rect, w: &[f32; AZIMUTHS]) -> bool {
    let Some(base) = set[0] else {
        return false;
    };
    let mut swing = 0.0;
    for (image, &weight) in set.iter().zip(w).skip(1) {
        if image.is_some() && weight > 0.003 {
            swing += weight.clamp(0.0, 1.0);
        }
    }
    let swing = swing.min(1.0);
    if swing < 1.0 {
        p.scoped(|p| {
            p.set_opacity(1.0 - swing);
            p.fill(&Shape::rect(rect), ImagePaint::new(base, rect));
        });
    }
    for (image, &weight) in set.iter().zip(w).skip(1) {
        if let Some(image) = image {
            if weight > 0.003 {
                p.scoped(|p| {
                    p.set_blend(BlendMode::Add);
                    p.set_opacity(weight.min(1.0));
                    p.fill(&Shape::rect(rect), ImagePaint::new(*image, rect));
                });
            }
        }
    }
    true
}

/// Weights of the renders (rest, −25°, +25°) for the virtual light's `angle` (radians): a swing
/// render in proportion to the swing. The rest entry is always 1 and only marks the slot;
/// [`blend`] draws the rest render at 1 minus the swing weights.
pub(super) fn light_weights(angle: f32) -> [f32; AZIMUTHS] {
    let d = (angle.to_degrees() - REST_DEG) / SWING_DEG;
    if !d.is_finite() {
        return [1.0, 0.0, 0.0];
    }
    let d = d.clamp(-1.0, 1.0);
    [1.0, (-d).max(0.0), d.max(0.0)]
}

/// A face colour of the fallback hardware: the ink at `k` of its light, opaque.
fn hw(pal: &Palette, k: f32) -> Color {
    pal.roles.ink.intensity(k).with_alpha(1.0)
}

/// The procedural plate: every static part as dark hardware with a thin rim, so the dash reads
/// without its assets.
pub(super) fn fallback_plate(p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
    let face = hw(pal, FALLBACK_FACE);
    let rim = pal.roles.ink.intensity(FALLBACK_RIM);
    let core = pal.roles.oled_black.with_alpha(0.94);
    let rim_px = Stroke::rim_px(1.0);

    // The rev bar: a plinth, the black light bar and its unlit lenses.
    let plinth = g.plinth_rect();
    p.fill(&Shape::round_rect(plinth, g.bar_r + g.plinth), core);
    p.stroke(&Shape::round_rect(plinth, g.bar_r + g.plinth), rim, rim_px);
    p.fill(&Shape::round_rect(g.bar_rect(), g.bar_r), face);
    let lens = hw(pal, FALLBACK_LENS);
    for i in 0..g.ticks {
        p.fill(&Shape::round_rect(g.tick_rect(i), g.tick_r), lens);
    }

    // The steering rail the dot rides on.
    let rail = g.rail_rect();
    p.fill(&Shape::round_rect(rail, rail.height() * 0.5), face);
    p.stroke(&Shape::round_rect(rail, rail.height() * 0.5), rim, rim_px);

    // The tyre diagram: the car chip and its four tyre lenses.
    let car = g.car_rect();
    p.fill(
        &Shape::round_rect(car, car.width() * 0.32),
        hw(pal, FALLBACK_FACE * 1.4),
    );
    p.stroke(&Shape::round_rect(car, car.width() * 0.32), rim, rim_px);
    for i in 0..4 {
        let r = g.pill_rect(i);
        p.fill(&Shape::round_rect(r, r.width() * 0.36), core);
        p.stroke(&Shape::round_rect(r, r.width() * 0.36), rim, rim_px);
    }

    // The boost gauge's scale and the black-glass groove its fill runs in.
    let a0 = g.boost_sweep[0].to_radians();
    let sweep = (g.boost_sweep[1] - g.boost_sweep[0]).to_radians();
    p.fill(
        &Shape::arc(g.boost_c, g.boost_r, g.boost_band, a0, sweep, Cap::Round),
        hw(pal, FALLBACK_RIM),
    );
    p.fill(
        &Shape::arc(g.boost_c, g.boost_track[0], g.boost_groove, a0, sweep, Cap::Round),
        core,
    );

    // The pedal slots.
    for i in 0..2 {
        let outer = g.slot_outer_rect(i);
        p.fill(&Shape::round_rect(outer, 6.0), core);
        p.stroke(&Shape::round_rect(outer, 6.0), rim, rim_px);
        p.fill(&Shape::round_rect(g.slot_rect(i), 4.0), face);
    }

    // The telltale rings round their lamps.
    for c in g.lamps {
        p.fill(&Shape::circle(c, g.lamp_ring), core);
        p.stroke(&Shape::circle(c, g.lamp_ring - 2.0), rim, Stroke::new(3.0));
    }

    // The trip tile.
    let trip = g.trip_outer_rect();
    p.fill(&Shape::round_rect(trip, 12.0), core);
    p.stroke(&Shape::round_rect(trip, 12.0), rim, rim_px);
    p.fill(&Shape::round_rect(g.trip_face_rect(), 9.0), face);
}

/// The procedural suggested-gear tile: a red face on a black base.
pub(super) fn fallback_tile(p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
    let outer = g.sug_outer_rect();
    p.fill(
        &Shape::round_rect(outer, g.sug_r + 3.0),
        pal.roles.oled_black.with_alpha(0.94),
    );
    p.fill(&Shape::round_rect(g.sug_face_rect(), g.sug_r), pal.roles.warning);
    // A soft top light, so the fallback still reads as a raised key.
    let face = g.sug_face_rect();
    let top = Rect::from_xywh(face.min.x + 6.0, face.min.y + 3.0, face.width() - 12.0, 3.0);
    p.fill(&Shape::round_rect(top, 1.5), pal.roles.ink.intensity(0.22));
}

/// The centre of the lamp icon's skid lines and car, relative to the lamp's centre.
const SKID_LINES: [[Vec2; 5]; 2] = [
    [
        vec2(-7.0, 3.0),
        vec2(-10.0, 6.0),
        vec2(-5.0, 9.5),
        vec2(-10.0, 13.0),
        vec2(-7.0, 16.0),
    ],
    [
        vec2(7.0, 3.0),
        vec2(4.0, 6.0),
        vec2(9.0, 9.5),
        vec2(4.0, 13.0),
        vec2(7.0, 16.0),
    ],
];

/// The slide telltale's icon (ISO 2575's stability-control symbol: a car over two skid lines)
/// centred on `c`, in `color`.
pub(super) fn slide_icon(p: &mut Painter<'_>, c: Vec2, color: Color) {
    let body = Rect::from_xywh(c.x - 11.0, c.y - 15.0, 22.0, 12.0);
    p.fill(&Shape::round_rect(body, 4.0), color);
    for wheel_x in [-8.5_f32, 8.5] {
        p.fill(
            &Shape::round_rect(Rect::from_xywh(c.x + wheel_x - 2.5, c.y - 5.0, 5.0, 4.0), 1.0),
            color,
        );
    }
    let mut pts = [Vec2::ZERO; 5];
    for line in SKID_LINES {
        for (dst, src) in pts.iter_mut().zip(line) {
            *dst = c + src;
        }
        p.fill(
            &Shape::Polyline {
                points: &pts,
                width: 2.5,
                closed: false,
            },
            color,
        );
    }
}

/// The limiter telltale's icon (a rev gauge with its needle at the end of the scale) centred on
/// `c`, in `color`.
pub(super) fn limiter_icon(p: &mut Painter<'_>, c: Vec2, color: Color) {
    let hub = c + vec2(0.0, 3.0);
    let (start, sweep) = (150.0_f32.to_radians(), 240.0_f32.to_radians());
    p.fill(&Shape::arc(hub, 13.0, 3.0, start, sweep, Cap::Round), color);
    let tip = Vec2::polar(hub, 10.0, start + sweep);
    p.fill(&Shape::line(hub, tip, 3.0), color);
    p.fill(&Shape::circle(hub, 3.0), color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_weights_follow_the_swing() {
        assert_eq!(light_weights(REST_DEG.to_radians()), [1.0, 0.0, 0.0]);
        let left = light_weights((REST_DEG - SWING_DEG).to_radians());
        assert!((left[1] - 1.0).abs() < 1e-5 && left[2] == 0.0);
        let half = light_weights((REST_DEG + SWING_DEG * 0.5).to_radians());
        assert!((half[2] - 0.5).abs() < 1e-4 && half[1] == 0.0);
        assert_eq!(light_weights(f32::NAN), [1.0, 0.0, 0.0]);
        // Past the swing the weights hold at its end.
        assert_eq!(
            light_weights((REST_DEG + 3.0 * SWING_DEG).to_radians()),
            [1.0, 0.0, 1.0]
        );
    }

    #[test]
    fn the_icons_fit_their_lamps() {
        let g = &super::super::geom::HOOD;
        for line in SKID_LINES {
            for q in line {
                assert!(q.length() + 1.25 < g.lamp_face, "skid line point {q:?}");
            }
        }
        // The car body's far corner and the gauge's outer edge stay inside the lamp face.
        assert!(vec2(11.0, 15.0).length() < g.lamp_face);
        assert!(13.0 + 1.5 + 3.0 < g.lamp_face);
    }
}
