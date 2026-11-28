//! The driving position beside the camera rows of the Fit & camera tab: the seat, wheel and
//! dash that asset-forge's `osd` recipe renders (`assets/osd/`, ledger `assets/osd/ASSETS.md`),
//! moved live (the seat and the eye with the seat height, the wheel with its offset), with the
//! eye, the default eye and the FOV fan drawn on top in the same projection. The part whose row
//! has focus lights from its rim in the accent. Without the files a line drawing of the same
//! scene stands in. Allocation-free.
use fh6_kit::{
    vec2, Align, BlendMode, Color, FontId, Gradient, ImageId, ImagePaint, Painter, Rect, Setup, Shape, Stroke,
    TextStyle, VAlign, Vec2,
};

/// The view as `tools/asset-forge/src/recipes/osd.rs` renders it (that recipe's tests check the
/// same numbers): 4K px per metre, the camera's yaw and pitch (degrees, orthographic) and its
/// window on the image plane (m: left, bottom, right, top).
const PX_PER_M: f32 = 550.0;
const YAW_DEG: f32 = 24.0;
const PITCH_DEG: f32 = 8.0;
const FRAME: [f32; 4] = [-0.37, -0.11, 1.37, 0.93];
/// The view's size (4K px).
pub const VIEW: Vec2 = vec2(957.0, 572.0);
/// The driver's eye at seat height 0 (m from the seat's H-point).
const EYE: [f32; 3] = [-0.02, 0.0, 0.66];
/// Each layer's place in the view (4K px from its top-left: x, y, width, height).
const DASH_RECT: [f32; 4] = [552.0, 152.0, 388.0, 338.0];
const WHEEL_RECT: [f32; 4] = [376.0, 194.0, 276.0, 240.0];
const SEAT_RECT: [f32; 4] = [0.0, 0.0, 492.0, 572.0];

/// The line drawing that stands in for the renders: the seat's centre line, the wheel (hub,
/// column angle, rim radius) and the dash's profile at its cut, as in the recipe.
const SEAT_PATH: [[f32; 2]; 10] = [
    [0.44, 0.06],
    [0.30, 0.0],
    [0.14, -0.035],
    [0.0, -0.05],
    [-0.10, 0.03],
    [-0.16, 0.20],
    [-0.22, 0.42],
    [-0.27, 0.62],
    [-0.30, 0.78],
    [-0.31, 0.88],
];
const HUB: [f32; 3] = [0.50, 0.0, 0.40];
const COLUMN_DEG: f32 = 22.0;
const RIM_R: f32 = 0.18;
const DASH: [[f32; 2]; 16] = [
    [0.82, 0.14],
    [0.80, 0.24],
    [0.785, 0.40],
    [0.775, 0.50],
    [0.745, 0.575],
    [0.755, 0.605],
    [0.85, 0.635],
    [0.96, 0.625],
    [1.04, 0.578],
    [1.14, 0.558],
    [1.24, 0.548],
    [1.27, 0.52],
    [1.25, 0.44],
    [1.12, 0.30],
    [1.00, 0.16],
    [0.88, 0.12],
];
const DASH_CUT: f32 = -0.06;

/// The FOV fan: radius (4K px, short enough to stay under the focus echo at the widest FOV and
/// the highest seat), points on its arc and its fill at the eye.
const FAN_R: f32 = 240.0;
const FAN_POINTS: usize = 17;
const FAN_FILL: f32 = 0.22;
/// The rim light on the focused part, and how much more while its value changes.
const RIM_LIGHT: f32 = 0.7;
const RIM_ENERGY: f32 = 0.3;
/// The eye's dot and the default eye's ring (radius, 4K px). The ring is wider than the dot's
/// dark collar, so at the default height the dot sits inside it.
const EYE_R: f32 = 16.0;
/// The rim light on a part the focused row does not move (a share of secondary ink).
const QUIET_RIM: f32 = 0.3;
/// The fan's edge stroke, doubled while the FOV row has focus, and how much of the edge's
/// strength is left at the arc.
const FAN_EDGE: f32 = 2.5;
const FAN_EDGE_END: f32 = 0.6;
const DEFAULT_R: f32 = 24.0;
/// The default ring's stroke (4K px).
const RING_W: f32 = 4.0;
/// The legend: its type size and its right edge and middle in the view (under the dash, clear
/// of the seat at every height and the wheel at every offset).
const LEGEND_CAP: f32 = 26.0;
const LEGEND_AT: Vec2 = vec2(940.0, 536.0);
// Checked when the crate compiles: the legend sits under the dash and inside the view.
const _: () = {
    assert!(LEGEND_AT.y - LEGEND_CAP > DASH_RECT[1] + DASH_RECT[3]);
    assert!(LEGEND_AT.y + LEGEND_CAP < VIEW.y && LEGEND_AT.x < VIEW.x);
};

/// The rendered layers, loaded once (see [`load`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriveImages {
    /// The dash, fixed.
    pub dash: ImageId,
    /// The wheel and its column.
    pub wheel: ImageId,
    /// The wheel lit by the rim light alone.
    pub wheel_rim: ImageId,
    /// The seat.
    pub seat: ImageId,
    /// The seat lit by the rim light alone.
    pub seat_rim: ImageId,
}

/// Loads the five layers from `assets/osd/`; `None` (and the line drawing) unless all load.
pub fn load(setup: &mut Setup<'_>) -> Option<DriveImages> {
    let dir = setup.assets?.join("osd");
    let mut one = |name: &str| -> Option<ImageId> {
        let path = dir.join(name);
        if !path.is_file() {
            tracing::debug!(path = %path.display(), "no driving-position layer: the OSD draws the line drawing");
            return None;
        }
        match setup.images.load_file(setup.device, setup.queue, &path, None) {
            Ok(image) => Some(image),
            Err(error) => {
                tracing::warn!(%error, file = name, "a driving-position layer did not load: the OSD draws the line drawing");
                None
            }
        }
    };
    Some(DriveImages {
        dash: one("drive_dash.png")?,
        wheel: one("drive_wheel.png")?,
        wheel_rim: one("drive_wheel_rim.png")?,
        seat: one("drive_seat.png")?,
        seat_rim: one("drive_seat_rim.png")?,
    })
}

/// Which part the focused row moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// A FOV row: the fan is the accent.
    Fov,
    /// Seat height: the seat lights from its rim.
    Seat,
    /// Wheel offset: the wheel lights from its rim.
    Wheel,
    /// Nothing in the view is the focused row's: another camera's FOV (the fan then shows the
    /// cockpit's, in secondary ink), or the view is fading out after the camera rows.
    None,
}

/// What the view shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriveState {
    /// Seat height (mm, up is positive).
    pub seat_mm: f32,
    /// Wheel offset (mm, up is positive).
    pub wheel_mm: f32,
    /// The default seat height (mm), where the default eye's ring sits.
    pub default_seat_mm: f32,
    /// The FOV the fan shows (degrees).
    pub fov_deg: f32,
    /// The part the focused row moves.
    pub focus: Focus,
    /// The energise light (0 at rest, 1 while a value changes).
    pub energy: f32,
}

/// The colours and face the view draws with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    /// The OSD's accent.
    pub accent: Color,
    /// Secondary ink.
    pub muted: Color,
    /// The faintest ink (the line drawing).
    pub ghost: Color,
    /// The text face.
    pub text: FontId,
}

/// A point (m, round the H-point) in the view (4K px from its top-left, y down).
fn view_px(p: [f32; 3]) -> Vec2 {
    let (sy, cy) = YAW_DEG.to_radians().sin_cos();
    let (sp, cp) = PITCH_DEG.to_radians().sin_cos();
    let x = p[0] * cy + p[1] * sy;
    let y = -p[0] * sy * sp + p[1] * cy * sp + p[2] * cp;
    vec2((x - FRAME[0]) * PX_PER_M, (FRAME[3] - y) * PX_PER_M)
}

/// A direction or length in the car (m) as a vector in the view (4K px, y down).
fn view_vec(v: [f32; 3]) -> Vec2 {
    view_px(v) - view_px([0.0; 3])
}

/// How far up the view (4K px) a part moves when it rises `mm`.
fn rise_px(mm: f32) -> f32 {
    if mm.is_finite() {
        mm * 0.001 * PITCH_DEG.to_radians().cos() * PX_PER_M
    } else {
        0.0
    }
}

/// Where the eye is drawn at `seat_mm` (4K px from the view's top-left).
#[must_use]
pub fn eye_at(seat_mm: f32) -> Vec2 {
    view_px(EYE) - vec2(0.0, rise_px(seat_mm))
}

/// How far below the view's top (4K px) the view reaches with the seat at `seat_mm`: the view's
/// height, plus however far a lowered seat hangs below it.
#[must_use]
pub fn bottom_at(seat_mm: f32) -> f32 {
    VIEW.y + (-rise_px(seat_mm)).max(0.0)
}

/// Draws the view with its top-left at `origin` (4K units) at opacity `alpha`.
pub fn draw(p: &mut Painter<'_>, origin: Vec2, images: Option<DriveImages>, s: &DriveState, look: &Look, alpha: f32) {
    let alpha = alpha.clamp(0.0, 1.0);
    if alpha <= 0.001 {
        return;
    }
    let seat_up = rise_px(s.seat_mm);
    let wheel_up = rise_px(s.wheel_mm);
    match images {
        Some(img) => {
            layer(p, origin, DASH_RECT, 0.0, img.dash, Color::WHITE.fade(alpha));
            layer(p, origin, WHEEL_RECT, wheel_up, img.wheel, Color::WHITE.fade(alpha));
            layer(p, origin, SEAT_RECT, seat_up, img.seat, Color::WHITE.fade(alpha));
            // The focused part lights from its rim: the rim-only render, tinted, added on top.
            let lit = look
                .accent
                .fade((RIM_LIGHT + RIM_ENERGY * s.energy.clamp(0.0, 1.0)) * alpha);
            // Both parts keep a quiet rim at all times, so their outlines hold over black.
            let quiet = look.muted.fade(QUIET_RIM * alpha);
            let (seat_rim, wheel_rim) = match s.focus {
                Focus::Seat => (lit, quiet),
                Focus::Wheel => (quiet, lit),
                Focus::Fov | Focus::None => (quiet, quiet),
            };
            p.save();
            p.set_blend(BlendMode::Add);
            layer(p, origin, SEAT_RECT, seat_up, img.seat_rim, seat_rim);
            layer(p, origin, WHEEL_RECT, wheel_up, img.wheel_rim, wheel_rim);
            p.restore();
        }
        None => sketch(p, origin, seat_up, wheel_up, s.focus, look, alpha),
    }
    // The fan fills only while the FOV row has focus; otherwise its two edges say enough.
    let (fan_paint, fill) = if s.focus == Focus::Fov {
        (look.accent, true)
    } else {
        (look.muted, false)
    };
    let eye = origin + eye_at(s.seat_mm);
    fan(p, eye, s.fov_deg, fan_paint, fill, alpha);
    p.stroke(
        &Shape::circle(origin + eye_at(s.default_seat_mm), DEFAULT_R),
        look.muted.fade(alpha),
        Stroke::new(RING_W),
    );
    // A dark collar keeps the dot apart from the render behind it.
    p.fill(&Shape::circle(eye, EYE_R + 3.0), Color::BLACK.fade(0.7 * alpha));
    p.fill(&Shape::circle(eye, EYE_R), look.accent.fade(alpha));
    legend(p, origin + LEGEND_AT, look, alpha);
}

/// One layer at its place in the view, `up` 4K px higher, multiplied by `tint`.
fn layer(p: &mut Painter<'_>, origin: Vec2, rect: [f32; 4], up: f32, image: ImageId, tint: Color) {
    let r = Rect::from_xywh(
        origin.x + rect[0],
        origin.y + rect[1] - up,
        rect[2].max(1.0),
        rect[3].max(1.0),
    );
    p.fill(&Shape::rect(r), ImagePaint::new(image, r).tint(tint));
}

/// The FOV fan from `eye`: the field of view in the car's centre plane, fading out from the eye.
fn fan(p: &mut Painter<'_>, eye: Vec2, fov_deg: f32, paint: Color, fill: bool, alpha: f32) {
    let half = (fov_deg.clamp(20.0, 140.0) * 0.5).to_radians();
    let reach = FAN_R / PX_PER_M;
    let mut pts = [eye; FAN_POINTS + 1];
    for (i, pt) in pts[1..].iter_mut().enumerate() {
        let th = -half + 2.0 * half * i as f32 / (FAN_POINTS - 1) as f32;
        *pt = eye + view_vec([th.cos() * reach, 0.0, th.sin() * reach]);
    }
    if fill {
        p.fill(
            &Shape::Polygon { points: &pts },
            Gradient::radial(eye, FAN_R)
                .stop(0.0, paint.fade(FAN_FILL * alpha))
                .stop(0.45, paint.fade(FAN_FILL * 0.3 * alpha))
                .stop(1.0, Color::TRANSPARENT),
        );
    }
    let width = if fill { 2.0 * FAN_EDGE } else { FAN_EDGE };
    let edge = paint.fade(0.85 * alpha);
    for end in [pts[1], pts[FAN_POINTS]] {
        p.fill(
            &Shape::line(eye, end, width),
            Gradient::linear(eye, end)
                .stop(0.0, edge)
                .stop(1.0, edge.fade(FAN_EDGE_END)),
        );
    }
    // The arc closes the fan at the edges' end strength: one band, so no joint doubles up.
    let mut band = [eye; 2 * FAN_POINTS];
    for (i, pt) in pts[1..].iter().enumerate() {
        let out = (*pt - eye).normalize_or_zero() * (width * 0.5);
        band[i] = *pt + out;
        band[2 * FAN_POINTS - 1 - i] = *pt - out;
    }
    p.fill(&Shape::Polygon { points: &band }, edge.fade(FAN_EDGE_END));
}

/// "● Your eye  ○ Default", right-aligned at `right_mid` (its right edge and cap middle). The
/// marks are the view's dot and ring at the legend's type size.
fn legend(p: &mut Painter<'_>, right_mid: Vec2, look: &Look, alpha: f32) {
    let style = TextStyle::new(look.text, LEGEND_CAP)
        .paint(look.muted.fade(alpha))
        .align(Align::Left, VAlign::CapMiddle);
    let (eye_label, default_label) = ("Your eye", "Default");
    let gap = 16.0;
    let r = LEGEND_CAP * 0.45;
    let w_default = p.measure(default_label, &style).width;
    let w_eye = p.measure(eye_label, &style).width;
    let mut x = right_mid.x - w_default;
    p.text(vec2(x, right_mid.y), default_label, &style);
    x -= gap + 2.0 * r;
    p.stroke(
        &Shape::circle(vec2(x + r, right_mid.y), r - 1.25),
        look.muted.fade(alpha),
        Stroke::new(2.5),
    );
    x -= 2.5 * gap + w_eye;
    p.text(vec2(x, right_mid.y), eye_label, &style);
    x -= gap + 2.0 * r;
    p.fill(
        &Shape::circle(vec2(x + r, right_mid.y), r - 2.0),
        look.accent.fade(alpha),
    );
}

/// The line drawing used when the renders are missing: the seat's centre line, the wheel's rim
/// and column, and the dash's outline, in the renders' projection.
fn sketch(p: &mut Painter<'_>, origin: Vec2, seat_up: f32, wheel_up: f32, focus: Focus, look: &Look, alpha: f32) {
    let pick = |f: Focus| (if focus == f { look.accent } else { look.ghost }).fade(alpha);
    let mut seat = [Vec2::ZERO; SEAT_PATH.len()];
    for (pt, &[x, z]) in seat.iter_mut().zip(SEAT_PATH.iter()) {
        *pt = origin + view_px([x, 0.0, z]) - vec2(0.0, seat_up);
    }
    p.fill(
        &Shape::Polyline {
            points: &seat,
            width: 18.0,
            closed: false,
        },
        pick(Focus::Seat),
    );
    let c = COLUMN_DEG.to_radians();
    let (up, along) = ([c.sin(), 0.0, c.cos()], [-c.cos(), 0.0, c.sin()]);
    let mut rim = [Vec2::ZERO; 32];
    for (i, pt) in rim.iter_mut().enumerate() {
        let th = std::f32::consts::TAU * i as f32 / 32.0;
        let (a, b) = (RIM_R * th.cos(), RIM_R * th.sin());
        let q = [HUB[0] + up[0] * b, HUB[1] + a, HUB[2] + up[2] * b];
        *pt = origin + view_px(q) - vec2(0.0, wheel_up);
    }
    p.fill(
        &Shape::Polyline {
            points: &rim,
            width: 8.0,
            closed: true,
        },
        pick(Focus::Wheel),
    );
    let hub = origin + view_px(HUB) - vec2(0.0, wheel_up);
    let reach = 0.3;
    let column_end = hub + view_vec([-along[0] * reach, 0.0, -along[2] * reach]);
    p.fill(&Shape::line(hub, column_end, 10.0), pick(Focus::Wheel));
    let mut dash = [Vec2::ZERO; DASH.len()];
    for (pt, &[x, z]) in dash.iter_mut().zip(DASH.iter()) {
        *pt = origin + view_px([x, DASH_CUT, z]);
    }
    p.fill(
        &Shape::Polyline {
            points: &dash,
            width: 3.0,
            closed: true,
        },
        look.ghost.fade(alpha),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_view_matches_the_recipe() {
        // tools/asset-forge/src/recipes/osd.rs checks the same values.
        let size = vec2((FRAME[2] - FRAME[0]) * PX_PER_M, (FRAME[3] - FRAME[1]) * PX_PER_M);
        assert!((size - VIEW).length() < 1e-3, "{size:?}");
        let eye = view_px(EYE);
        assert!(
            (eye.x - 193.45).abs() < 0.02 && (eye.y - 151.41).abs() < 0.02,
            "eye at {eye:?}"
        );
        assert_eq!(DASH_RECT, [552.0, 152.0, 388.0, 338.0]);
        assert_eq!(WHEEL_RECT, [376.0, 194.0, 276.0, 240.0]);
        assert_eq!(SEAT_RECT, [0.0, 0.0, 492.0, 572.0]);
        for r in [DASH_RECT, WHEEL_RECT, SEAT_RECT] {
            assert!(r[0] + r[2] <= VIEW.x && r[1] + r[3] <= VIEW.y);
        }
        // The line drawing's geometry, each number weighted by its place: the recipe's test
        // pins the same sum, so a reshaped seat or dash fails on the side that did not follow.
        let tail = [COLUMN_DEG, RIM_R, DASH_CUT];
        let numbers = SEAT_PATH.iter().chain(&DASH).flatten().chain(&HUB).chain(&tail);
        let sum: f32 = numbers.zip(1u8..).map(|(v, place)| v * f32::from(place)).sum();
        assert!((sum - 2145.40).abs() < 0.01, "the sketch geometry moved: {sum}");
    }

    #[test]
    fn the_seat_and_eye_move_together_and_up_is_up() {
        assert!(rise_px(50.0) > 0.0 && rise_px(-100.0) < 0.0);
        // 0.55 px per mm, foreshortened by the 8° pitch.
        assert!((rise_px(100.0) - 54.47).abs() < 0.01, "{}", rise_px(100.0));
        assert!((eye_at(-45.0).y - eye_at(0.0).y - rise_px(45.0)).abs() < 1e-4);
        assert_eq!(rise_px(f32::NAN), 0.0, "a bad value cannot throw the view away");
    }

    #[test]
    fn the_fan_stays_under_the_echo_at_the_widest_fov_and_highest_seat() {
        // The FOV rows reach the top of FOV_RANGE (110°) and the seat the top of its row's
        // range (the OSD clamps what it draws to it). Even then the fan's tip ends less than
        // 80 px above the view's top edge: the OSD puts the lifted echo's help line 120 px above
        // that edge (draw.rs, ECHO_LIFT and DRIVE_TOP), so the two never touch.
        let eye = eye_at(crate::model::SEAT.max);
        let half = (fh6_core::config::FOV_RANGE.1 * 0.5).to_radians();
        let top = eye + view_vec([half.cos() * FAN_R / PX_PER_M, 0.0, half.sin() * FAN_R / PX_PER_M]);
        assert!(top.y > -80.0, "fan top {} px above the view", -top.y);
        // The legend sits under the wheel at the bottom of its row's range; the const block
        // under the legend's constants checks it against the dash and the view's bottom edge.
        let lowest = rise_px(crate::model::WHEEL.min);
        assert!(LEGEND_AT.y - LEGEND_CAP > WHEEL_RECT[1] + WHEEL_RECT[3] - lowest);
    }
}
