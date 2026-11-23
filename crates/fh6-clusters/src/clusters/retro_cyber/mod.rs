//! Retro Cyber (`ClusterTheme::RetroCyber`), catalog row 19: the Sci-Fi family's pattern-setter
//! (VFD dialect), cockpit + hood, build #6.
//!
//! A vacuum-fluorescent tube in a machined housing: a 10 px gunmetal chamfer45 ring (light-driven
//! facets) around an opaque `#000000` cavity. Inside, four VFD layers, each with its own parallax
//! under felt G (0 / 1 / 3 / 4 px):
//! 1. **Phosphor** (0 px): procedural seven-segment digits, pods, chevrons and dots. Unlit
//!    elements show as opaque 7% ghosts of their own phosphor (the physical glass of a VFD).
//! 2. **Control-grid mesh** (+1 px): a fine hex mesh multiplied over the cavity, so it only
//!    modulates lit phosphor (off pixels stay black).
//! 3. **Filaments** (+3 px): five 1 px wires, multiplied, seen as dark lines across lit segments.
//! 4. **Front glass** (+4 px): never a tint; only a light-driven crescent and a 1 px rim.
//!
//! The 2 px scanlines multiply emission over the cavity only.
//!
//! Ranks:
//! - 1: the octagon-bay seven-segment gear (cap 220, white phosphor with a 45% white core, the
//!   `N` diagonal on its glass) under the 38 px slanted 15-pod shift strip.
//! - 2: the seven-segment speed (cap 112, ghost 8s, 0.6 hysteresis).
//! - 3: the 24-chevron rising staircase (14 primary, 6 secondary, 4 accent), steady at the limiter.
//! - 4: unglowing THR / BRK / BST columns (go / warning / secondary), a 13 × 5 dot G-scope with a
//!   500 ms trail, 6-step drift columns on the side the tail steps out and a DRIFT readout.
//! - 5: SLIP / LOCK / SPIN annunciators (lit only when active), scale numerals, labels, bezel.
//!
//! Signature moments:
//! - **Filament ignition boot:** on the first observed `IsRaceOn` 0 → 1 after launch the five
//!   filaments warm as 1 px lines at 15% luminance for 150 ms, then the cavity ignites in 8
//!   left-to-right sectors of 40 ms as an all-segments self-test that settles to ghost with the
//!   50 ms phosphor decay (≈ 0.52 s in all). Gear and speed are live from frame 1.
//! - **Gear change:** the new digit re-strikes segment by segment over 80 ms (`Reveal::Sequence`)
//!   while a 1-frame 60% rim flash laps the octagon's 8 facets in 50 ms.
//! - **Limiter:** only the 38 px strip strobes (10 Hz, reduced-flash breath); the octagon rim holds
//!   steady accent; after 0.5 s on the limiter the gear pulses at 2 Hz.
//!
//! Telemetry: rpm against the shift point (strip, chevrons), gear, speed, throttle, brake, boost,
//! smoothed G (scope, light azimuth, parallax) and body slip from velocity (drift columns,
//! readout, SLIP). LOCK and SPIN are **inferred** (the packet has no ABS / TC flags and no tyre
//! slip ratio): LOCK = hard brake without the matching deceleration, SPIN = throttle with the body
//! sliding. Nothing is synthesised.
//!
//! ## Zone table (local 4K px around the anchor; mirrors `mockups/RetroCyber/index.html`)
//!
//! | Zone | Element | Cockpit | Hood |
//! |---|---|---|---|
//! | Z5 | cavity (bezel 10 px outside it) | 1380 × 500 at (0, −40), cut 58 | 1680 × 336 at (0, −20), cut 48 |
//! | Z1 | 15-pod strip | x ±600, y −264..−226 | x ±600, y −174..−144 |
//! | Z3 | 24 chevrons | x ±600, y −212..−150 | x ±600, y −134..−92 |
//! | Z1 | octagon gear bay | 300 × 280 at (0, 46), cut 72, gear cap 220 | 220 × 194 at (0, 39), cut 52, cap 150 |
//! | Z4 | drift columns | x ±196, 22 × 200 | x ±150, 18 × 160 |
//! | Z2 | speed (3 cells) | (−470, −10), cap 112 | (−590, 40), cap 100 |
//! | Z4 | G-scope 13 × 5 | (−470, 140), pitch 16 × 13 | (−290, 50), pitch 14 × 12 |
//! | Z4 | THR / BRK / BST | x 290..470, y −70..140 | x 250..450, y −50..106 |
//! | Z5 | SLIP / LOCK / SPIN | x 520..650, y −70..70 | x 490..670, y −56..76 |
//! | Z4 | DRIFT readout | (585, 118), cap 40 | (760, 40), cap 36 |
use crate::common::bezel::{chamfer_level, lit_rim, octagon, on_screen};
use crate::common::seg7::{
    draw_scanlines, Emission, Phosphor, Reveal, Seg7Digit, Seg7Geometry, Seg7Number, Segments, DIAGONAL,
};
use crate::common::shift::LimiterCue;
use crate::common::tach::TachChevron;
use crate::common::{ShiftLights, SpeedReadout, VirtualLight};
use core::f32::consts::TAU;
use fh6_kit::cluster::Setup;
use fh6_kit::color::{linear_to_srgb, srgb_to_linear};
use fh6_kit::shape::SEVEN_SEG_DIGITS;
use fh6_kit::{
    vec2, Align, BlendMode, Cluster, Color, FontId, FontRole, FrameInput, Gear, Glow, Gradient, ImageId, ImagePaint,
    ImageWrap, Layout, Paint, Painter, Palette, Rect, Shape, Signals, Smoothed, Stroke, TextStyle, Tokens, VAlign,
    Vec2, View,
};

/// Shift-strip pods (catalog brief).
const PODS: u32 = 15;
/// Pod slant, matching the seven-segment digits (degrees).
const POD_SLANT_DEG: f32 = 8.0;
/// Boot: filament warm-up (s).
const FILAMENT_WARM_S: f32 = 0.15;
/// Boot: ignition sectors, left to right.
const SECTORS: usize = 8;
/// Boot: each sector's self-test (s).
const SECTOR_S: f32 = 0.04;
/// Phosphor release: lit segments fall to ghost over this time (s).
const PHOSPHOR_DECAY_S: f32 = 0.05;
/// The whole boot (warm-up, 8 sectors and the last sector's decay).
const BOOT_S: f32 = FILAMENT_WARM_S + SECTORS as f32 * SECTOR_S + PHOSPHOR_DECAY_S;
/// Boot filament luminance as seen on screen.
const FILAMENT_WARM_LEVEL: f32 = 0.15;
/// Gear change: the rim flash laps the 8 facets in this time (s).
const RIM_LAP_S: f32 = 0.05;
/// Each facet flashes for one 60 Hz frame.
const RIM_FLASH_S: f32 = 1.0 / 60.0;
/// Rim flash strength (share of ink).
const RIM_FLASH: f32 = 0.6;
/// Gear pulse starts after this long on the limiter (s), at 2 Hz.
const PULSE_AFTER_S: f32 = 0.5;
/// Gear pulse frequency (Hz).
const PULSE_HZ: f32 = 2.0;
/// Neutral shows in ink at 72%.
const NEUTRAL_LEVEL: f32 = 0.72;
/// Steps per pedal / boost column.
const COLUMN_STEPS: usize = 16;
/// Boost that fills the BST column (psi).
const BOOST_FULL_PSI: f32 = 25.0;
/// Drift column thresholds (body slip, degrees), bottom to top.
const DRIFT_STEPS: [f32; 6] = [5.0, 10.0, 15.0, 20.0, 30.0, 45.0];
/// The DRIFT readout and SLIP lamp light from this body slip (degrees).
const SLIP_LAMP_DEG: f32 = 8.0;
/// SPIN (inferred): throttle above this with the body sliding past [`SPIN_SLIP_DEG`].
const SPIN_THROTTLE: f32 = 0.6;
/// See [`SPIN_THROTTLE`].
const SPIN_SLIP_DEG: f32 = 12.0;
/// LOCK (inferred): brake above this while the car decelerates less than
/// [`LOCK_DECEL_PER_BRAKE`] g per unit of brake, above [`LOCK_MIN_SPEED`] m/s.
const LOCK_BRAKE: f32 = 0.6;
/// See [`LOCK_BRAKE`].
const LOCK_DECEL_PER_BRAKE: f32 = 0.35;
/// See [`LOCK_BRAKE`].
const LOCK_MIN_SPEED: f32 = 5.0;
/// Annunciators hold for this long after their condition clears (no flicker).
const LAMP_HOLD_S: f32 = 0.15;
/// G-scope grid.
const SCOPE_COLS: usize = 13;
/// See [`SCOPE_COLS`].
const SCOPE_ROWS: usize = 5;
/// Felt G per scope column / row.
const SCOPE_G: Vec2 = Vec2::new(0.25, 0.5);
/// The scope trail fades over this time (s).
const SCOPE_TRAIL_S: f32 = 0.5;
/// Parallax of the four VFD layers at full felt G (phosphor, mesh, filaments, glass).
const PARALLAX: [f32; 4] = [0.0, 1.0, 3.0, 4.0];
/// Felt G that reaches full parallax.
const PARALLAX_G: f32 = 1.5;
/// Unlit VFD elements show this share of their phosphor's displayed brightness.
const GHOST_LEVEL: f32 = 0.07;
/// Housing ring width (outside the cavity).
const RING: f32 = 10.0;
/// Share of the ring that is the outer 45° bevel.
const BEVEL_PCT: f32 = 0.38;
/// Gunmetal face on the lit and far side, as seen on screen (`brushedTitanium.base`).
const GUNMETAL_LIT: f32 = 0.22;
/// See [`GUNMETAL_LIT`].
const GUNMETAL_DARK: f32 = 0.09;
/// The strip (rank 1) and chevron (rank 3) halos. They are local glows, not bloom: the display
/// bloom has a fixed radius in pixels and cannot be masked to the cavity, so even at 18% of the
/// tokens' intensity it lit the bezel and the black outside it. A 3 px glow stays inside.
const STRIP_GLOW: Glow = Glow {
    sigma: 3.5,
    intensity: 0.45,
    fill_inside: false,
};
/// See [`STRIP_GLOW`].
const CHEVRON_GLOW: Glow = Glow {
    sigma: 2.5,
    intensity: 0.3,
    fill_inside: false,
};
/// The gear's emission off the limiter: the single soft lobe, so rank 1 glows a little more
/// than the speed; the limiter keeps it (the strip strobes, the gear does not).
const GEAR_EMISSION: Emission = Emission::Soft;
/// Rank-4 lit phosphor (columns, drift columns) as seen on screen.
const RANK4_LEVEL: f32 = 0.6;
/// Labels as seen on screen, relative to the label ink.
const LABEL_LEVEL: f32 = 0.7;
/// Bay rim: 2 px at 35% ink, steady accent at the limiter.
const BAY_RIM: f32 = 0.35;
/// See [`BAY_RIM`].
const BAY_RIM_PX: f32 = 2.0;
/// Margin round the Blender housing render (local px; as the recipe's `MARGIN`).
const HOUSING_MARGIN: f32 = 4.0;
/// The ComfyUI phosphor plate's tile (local px): grains of about 2–3 px, under the 6 px mesh.
const PLATE_TILE: f32 = 80.0;
/// Gain on lit phosphor under the grain plate: the plate's mean modulation is about 0.94 of
/// display level, so its highlights may clip to keep a lit block as bright as flat paint.
const PLATE_GAIN: f32 = 1.1;
/// Annunciator legend cap height as a share of the lamp height (36–39 px lamps: 19–21 px caps).
const LAMP_CAP: f32 = 0.54;
/// Hex-mesh tile size (local px): the Phase 6 tile (432 px, about 24 cells across) at a 6 px cell.
const MESH_TILE: f32 = 144.0;
/// Mesh strength (multiply lerp from identity).
const MESH_STRENGTH: f32 = 0.42;
/// Filament darkening (multiply).
const FILAMENT_DARK: f32 = 0.55;
/// Front-glass rim and crescent peaks (linear coverage of ink).
const GLASS_RIM: f32 = 0.22;
/// See [`GLASS_RIM`].
const GLASS_CRESCENT: f32 = 0.035;

/// Geometry of one view (local 4K px around the anchor).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
    cavity: Rect,
    cut: f32,
    strip: Rect,
    pod_gap: f32,
    chevrons: Rect,
    scale_cap: f32,
    bay: Rect,
    bay_cut: f32,
    gear_cap: f32,
    drift_x: f32,
    drift_size: Vec2,
    speed: Vec2,
    speed_cap: f32,
    scope: Vec2,
    scope_pitch: Vec2,
    columns: Rect,
    lamps: Rect,
    readout: Vec2,
    readout_cap: f32,
    filaments: [f32; 5],
    label_cap: f32,
}

const fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect {
        min: vec2(x0, y0),
        max: vec2(x1, y1),
    }
}

const COCKPIT: Geom = Geom {
    cavity: rect(-690.0, -290.0, 690.0, 210.0),
    cut: 58.0,
    strip: rect(-600.0, -264.0, 600.0, -226.0),
    pod_gap: 12.0,
    chevrons: rect(-600.0, -212.0, 600.0, -150.0),
    scale_cap: 14.0,
    bay: rect(-150.0, -94.0, 150.0, 186.0),
    bay_cut: 72.0,
    gear_cap: 220.0,
    drift_x: 196.0,
    drift_size: vec2(22.0, 200.0),
    speed: vec2(-470.0, -10.0),
    speed_cap: 112.0,
    scope: vec2(-470.0, 140.0),
    scope_pitch: vec2(16.0, 13.0),
    columns: rect(290.0, -70.0, 470.0, 140.0),
    lamps: rect(520.0, -70.0, 650.0, 70.0),
    readout: vec2(585.0, 118.0),
    readout_cap: 40.0,
    filaments: [-251.0, -181.0, -30.0, 60.0, 150.0],
    label_cap: 15.0,
};

/// Hood: the scrim is 0%, so the tube widens and lowers (1680 × 336). The same grammar as the
/// cockpit: strip, then the full-width staircase, then speed and scope left of a smaller bay and
/// the columns, lamps and readout right of it.
const HOOD: Geom = Geom {
    cavity: rect(-840.0, -188.0, 840.0, 148.0),
    cut: 48.0,
    strip: rect(-600.0, -174.0, 600.0, -144.0),
    pod_gap: 12.0,
    chevrons: rect(-600.0, -134.0, 600.0, -92.0),
    scale_cap: 12.0,
    bay: rect(-110.0, -58.0, 110.0, 136.0),
    bay_cut: 52.0,
    gear_cap: 150.0,
    drift_x: 150.0,
    drift_size: vec2(18.0, 160.0),
    speed: vec2(-590.0, 40.0),
    speed_cap: 100.0,
    scope: vec2(-290.0, 50.0),
    scope_pitch: vec2(14.0, 12.0),
    columns: rect(250.0, -50.0, 450.0, 106.0),
    lamps: rect(490.0, -56.0, 670.0, 76.0),
    readout: vec2(760.0, 40.0),
    readout_cap: 36.0,
    filaments: [-159.0, -113.0, -20.0, 50.0, 115.0],
    label_cap: 14.0,
};

fn geom(view: View) -> &'static Geom {
    match view {
        View::Hood => &HOOD,
        View::Cockpit | View::Chase => &COCKPIT,
    }
}

impl Geom {
    /// The central band of the cavity: its full height between the corner cuts (the mesh,
    /// scanlines and boot sectors cover it; nothing is drawn in the corner triangles).
    fn band(&self) -> Rect {
        rect(
            self.cavity.min.x + self.cut,
            self.cavity.min.y,
            self.cavity.max.x - self.cut,
            self.cavity.max.y,
        )
    }

    /// The zones that hold phosphor (the mesh and scanlines only matter over lit phosphor, so
    /// they cover these, not the whole cavity): strip, chevrons, speed, bay with its drift
    /// columns, and the right block (columns, lamps, readout).
    fn zones(&self) -> [Rect; 5] {
        let sg = Seg7Geometry::new(self.speed_cap);
        let speed = Rect::from_center_size(self.speed, vec2(sg.advance() * 3.0 + 12.0, self.speed_cap + 12.0));
        let c = self.bay.center();
        let bay = Rect::from_points(
            vec2(c.x - self.drift_x - self.drift_size.x, self.bay.min.y),
            vec2(c.x + self.drift_x + self.drift_size.x, self.bay.max.y),
        );
        let rg = Seg7Geometry::new(self.readout_cap);
        let right = Rect::from_points(
            vec2(self.columns.min.x, self.columns.min.y.min(self.lamps.min.y)),
            vec2(
                self.lamps.max.x.max(self.readout.x + rg.advance() * 1.5),
                self.columns.max.y.max(self.readout.y + self.readout_cap * 0.5 + 4.0),
            ),
        );
        [self.strip.expand(4.0), self.chevrons.expand(4.0), speed, bay, right]
    }

    /// Boot sector `k` (left to right) of the band.
    fn sector(&self, k: usize) -> Rect {
        let b = self.band();
        let w = b.width() / SECTORS as f32;
        rect(b.min.x + w * k as f32, b.min.y, b.min.x + w * (k + 1) as f32, b.max.y)
    }

    /// Pod `i` of the strip: a parallelogram leaning like the digits.
    fn pod(&self, i: usize) -> [Vec2; 4] {
        let n = PODS as f32;
        let w = (self.strip.width() - self.pod_gap * (n - 1.0)) / n;
        let h = self.strip.height();
        let lean = h * POD_SLANT_DEG.to_radians().tan();
        let x = self.strip.min.x + i as f32 * (w + self.pod_gap);
        let (top, bottom) = (self.strip.min.y, self.strip.max.y);
        [
            vec2(x - lean * 0.5, bottom),
            vec2(x + w - lean * 0.5, bottom),
            vec2(x + w + lean * 0.5, top),
            vec2(x + lean * 0.5, top),
        ]
    }

    /// Cell `j` (0 at the bottom) of the drift column on `side` (-1 left, +1 right).
    fn drift_cell(&self, side: f32, j: usize) -> Rect {
        let c = self.bay.center();
        let n = DRIFT_STEPS.len() as f32;
        let pitch = self.drift_size.y / n;
        let bottom = c.y + self.drift_size.y * 0.5;
        let x = c.x + side * self.drift_x;
        Rect::from_center_size(
            vec2(x, bottom - pitch * (j as f32 + 0.5)),
            vec2(self.drift_size.x, pitch * 0.72),
        )
    }

    /// Column `c` (THR, BRK, BST) and its step `j` (0 at the bottom).
    fn column_cell(&self, c: usize, j: usize) -> Rect {
        let w = self.columns.width() / 3.0;
        let x = self.columns.min.x + w * (c as f32 + 0.5);
        let pitch = self.columns.height() / COLUMN_STEPS as f32;
        let y = self.columns.max.y - pitch * (j as f32 + 0.5);
        Rect::from_center_size(vec2(x, y), vec2(w * 0.62, pitch * 0.62))
    }

    /// Annunciator `i` (SLIP, LOCK, SPIN), stacked.
    fn lamp(&self, i: usize) -> Rect {
        let h = (self.lamps.height() - 2.0 * 12.0) / 3.0;
        let y = self.lamps.min.y + i as f32 * (h + 12.0);
        rect(self.lamps.min.x, y, self.lamps.max.x, y + h)
    }

    /// The scope dot at `(col, row)`.
    fn scope_dot(&self, col: usize, row: usize) -> Vec2 {
        let half = vec2((SCOPE_COLS - 1) as f32 * 0.5, (SCOPE_ROWS - 1) as f32 * 0.5);
        self.scope
            + vec2(
                (col as f32 - half.x) * self.scope_pitch.x,
                (row as f32 - half.y) * self.scope_pitch.y,
            )
    }
}

/// The opaque 7% ghost of a phosphor (sRGB level, so it reads the same on any black).
fn ghost(phosphor: Color) -> Color {
    let (r, g, b) = phosphor.straight_rgb();
    let dim = |c: f32| srgb_to_linear(GHOST_LEVEL * linear_to_srgb(c.clamp(0.0, 1.0)));
    Color::linear(dim(r), dim(g), dim(b), 1.0)
}

/// The VFD stage colour of pod `i` (pods fill outside-in onto the centre pod): the outer 8 in
/// `primary`, the next 4 in `secondary`, the centre 3 in `accent` (the family's cyan → yellow →
/// magenta; at the shift point every pod takes `shift`).
fn pod_stage(i: usize, palette: &Palette) -> Color {
    let half = (PODS / 2) as usize;
    let from_centre = i.abs_diff(half);
    if from_centre <= 1 {
        palette.accent
    } else if from_centre <= 3 {
        palette.secondary
    } else {
        palette.primary
    }
}

/// The octagon of `r` / `cut` offset outwards by `d`, every side parallel at distance `d`.
fn octagon_grown(r: Rect, cut: f32, d: f32) -> [Vec2; 8] {
    octagon(r.expand(d), cut + d * (2.0 - core::f32::consts::SQRT_2))
}

/// The chamfer45 housing (COMPONENTS.md §9, the `common::bezel` Chamfer45 look) drawn facet by
/// facet: an opaque `oledBlack` body (a rect plus two side trapezoids that overlap it by 2 px,
/// so there is no anti-aliased seam), then per facet the flat gunmetal face shaded along the
/// light, the outer 45° bevel lit by its facing, and the 1 px rims (top-lit outer edge, inverted
/// lip at the cavity). Each draw covers only its own facet: the shared component's two closed
/// polylines cover the whole tube and cost 0.33 ms at 4K.
fn draw_housing(p: &mut Painter<'_>, g: &Geom, pal: &Palette, light: f32, art: Option<[ImageId; 2]>) {
    let roles = pal.roles;
    let ink = roles.ink;
    let (r, cut) = (g.cavity, g.cut);
    let outer = octagon_grown(r, cut, RING);
    let body = r.expand(RING);
    let cut_o = cut + RING * (2.0 - core::f32::consts::SQRT_2);
    p.fill(
        &Shape::rect(rect(body.min.x + cut_o, body.min.y, body.max.x - cut_o, body.max.y)),
        roles.oled_black,
    );
    for (edge, inner) in [
        (body.min.x, body.min.x + cut_o + 2.0),
        (body.max.x, body.max.x - cut_o - 2.0),
    ] {
        let side = [
            vec2(inner, body.min.y),
            vec2(inner, body.max.y),
            vec2(edge, body.max.y - cut_o),
            vec2(edge, body.min.y + cut_o),
        ];
        p.fill(&Shape::Polygon { points: &side }, roles.oled_black);
    }
    if let Some(lights) = art {
        draw_housing_art(p, g, pal, light, lights);
        return;
    }
    let inner = octagon(r, cut);
    let bevel_from = octagon_grown(r, cut, RING * (1.0 - BEVEL_PCT));
    let c = r.center();
    let l = Vec2::from_angle(light);
    let ext = r.size().length() * 0.5 + RING;
    let face = Gradient::linear(c + l * ext, c - l * ext)
        .stop(0.0, ink.with_alpha(on_screen(GUNMETAL_LIT)))
        .stop(1.0, ink.with_alpha(on_screen(GUNMETAL_DARK)));
    let rim_out = lit_rim(
        c,
        light,
        roles.rim_specular.with_alpha(0.5),
        roles.hairline.with_alpha(0.06),
    );
    let rim_in = lit_rim(
        c,
        light + core::f32::consts::PI,
        roles.rim_specular.with_alpha(0.35),
        roles.hairline.with_alpha(0.05),
    );
    let px = p.device_px();
    for k in 0..8 {
        let j = (k + 1) % 8;
        let d = (outer[j] - outer[k]).normalize_or_zero();
        let normal = vec2(d.y, -d.x);
        let flat = [inner[k], inner[j], bevel_from[j], bevel_from[k]];
        p.fill(&Shape::Polygon { points: &flat }, face);
        let bevel = [bevel_from[k], bevel_from[j], outer[j], outer[k]];
        p.fill(
            &Shape::Polygon { points: &bevel },
            ink.with_alpha(on_screen(chamfer_level(normal, light))),
        );
        p.fill(&Shape::line(outer[k], outer[j], px), rim_out);
        p.fill(&Shape::line(inner[k], inner[j], px), rim_in);
    }
    // Rationed greeble: one slotted screw near each end of the top and bottom faces.
    let screw_r = RING * 0.2;
    let dy = RING * (1.0 - BEVEL_PCT) * 0.5;
    let inset = cut + RING * 1.4;
    let slot = Vec2::from_angle(30f32.to_radians()) * (screw_r * 0.72);
    for (x, y) in [
        (r.min.x + inset, r.min.y - dy),
        (r.max.x - inset, r.min.y - dy),
        (r.min.x + inset, r.max.y + dy),
        (r.max.x - inset, r.max.y + dy),
    ] {
        let at = vec2(x, y);
        p.fill(&Shape::circle(at, screw_r), ink.with_alpha(on_screen(0.30)));
        p.fill(&Shape::line(at - slot, at + slot, screw_r * 0.3), roles.oled_black);
    }
}

/// The Blender housing (`assets/clusters/retro_cyber/housing_<view>_l160|l110.png`): the two key
/// lights blended by the virtual light's azimuth (−160° → −110°), each drawn only over the ring
/// band facet by facet, then the light-driven 1 px outer rim on top for a crisp edge.
fn draw_housing_art(p: &mut Painter<'_>, g: &Geom, pal: &Palette, light: f32, art: [ImageId; 2]) {
    let (r, cut) = (g.cavity, g.cut);
    let image_rect = r.expand(RING + HOUSING_MARGIN);
    let lo = -160f32.to_radians();
    let k = ((light - lo) / 50f32.to_radians()).clamp(0.0, 1.0);
    let inner = octagon_grown(r, cut, -1.0);
    let outer = octagon_grown(r, cut, RING + 1.0);
    for (img, share, blend) in [(art[0], 1.0 - k, BlendMode::Normal), (art[1], k, BlendMode::Add)] {
        if share <= 0.001 {
            continue;
        }
        p.scoped(|p| {
            p.set_blend(blend);
            let paint = ImagePaint::new(img, image_rect).tint(Color::WHITE.with_alpha(share));
            for i in 0..8 {
                let j = (i + 1) % 8;
                let band = [inner[i], inner[j], outer[j], outer[i]];
                p.fill(&Shape::Polygon { points: &band }, paint);
            }
        });
    }
    let roles = pal.roles;
    let rim = lit_rim(
        r.center(),
        light,
        roles.rim_specular.with_alpha(0.4),
        roles.hairline.with_alpha(0.0),
    );
    let edge = octagon_grown(r, cut, RING);
    let px = p.device_px();
    for i in 0..8 {
        p.fill(&Shape::line(edge[i], edge[(i + 1) % 8], px), rim);
    }
}

/// The phosphor paint for colour `c`: the ComfyUI grain plate tinted by `c` when present, else
/// the flat colour.
fn phosphor_paint(plate: Option<ImageId>, c: Color) -> Paint {
    match plate {
        Some(img) => {
            let tile = Rect::from_xywh(0.0, 0.0, PLATE_TILE, PLATE_TILE);
            ImagePaint::new(img, tile).repeat().tint(c).into()
        }
        None => c.into(),
    }
}

/// One lit phosphor block (a pod or a chevron) with an optional tight glow. With the ComfyUI
/// plate it is the phosphor grain tinted by its colour; without it, flat emission falling to 85%
/// at its lower edge. No white band, which reads as a lit plastic key rather than phosphor.
fn phosphor_block(p: &mut Painter<'_>, pts: &[Vec2], c: Color, glow: Option<Glow>, plate: Option<ImageId>) {
    let shape = Shape::Polygon { points: pts };
    if let Some(gl) = glow {
        p.glow(&shape, c, gl);
    }
    if plate.is_some() {
        p.fill(&shape, phosphor_paint(plate, c));
        return;
    }
    let (top, bottom) = pts
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(t, b), v| (t.min(v.y), b.max(v.y)));
    let x = pts.first().map_or(0.0, |v| v.x);
    p.fill(
        &shape,
        Gradient::linear(vec2(x, bottom), vec2(x, top))
            .stop(0.0, c.intensity(srgb_to_linear(0.85)))
            .stop(0.35, c)
            .stop(1.0, c),
    );
}

/// Rank-4 phosphor: the colour at [`RANK4_LEVEL`] of its displayed brightness, no glow.
fn rank4(c: Color) -> Color {
    c.intensity(srgb_to_linear(RANK4_LEVEL))
}

/// The boot clock: started by the first observed `IsRaceOn` 0 → 1 after launch (a cluster made
/// while racing never saw the 0, so it does not boot).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Boot {
    prev: Option<bool>,
    done: bool,
    t: Option<f32>,
}

impl Boot {
    /// Advances with this frame's `IsRaceOn`.
    fn observe(&mut self, race_on: bool, dt: f32) {
        if let Some(t) = self.t.as_mut() {
            *t += dt;
            if *t > BOOT_S || !race_on {
                self.t = None;
            }
        }
        if self.prev == Some(false) && race_on && !self.done {
            self.done = true;
            self.t = Some(0.0);
        }
        self.prev = Some(race_on);
    }

    fn running(&self) -> bool {
        self.t.is_some()
    }

    /// Seconds into the boot, if it runs.
    fn t(&self) -> Option<f32> {
        self.t
    }
}

/// When sector `k` ignites (s into the boot).
fn sector_start(k: usize) -> f32 {
    FILAMENT_WARM_S + k as f32 * SECTOR_S
}

/// The self-test level of sector `k` at boot time `t`: full for its 40 ms, then the phosphor
/// decay to ghost.
fn sector_level(t: f32, k: usize) -> f32 {
    let s = sector_start(k);
    if t < s {
        0.0
    } else if t < s + SECTOR_S {
        1.0
    } else {
        (1.0 - (t - s - SECTOR_S) / PHOSPHOR_DECAY_S).clamp(0.0, 1.0)
    }
}

/// Drift steps lit for a body slip of `deg` (absolute).
fn drift_steps(deg: f32) -> usize {
    let a = if deg.is_finite() { deg.abs() } else { 0.0 };
    DRIFT_STEPS.iter().filter(|&&t| a >= t).count()
}

/// The scope cell for felt G `felt` (x right, y down).
fn scope_cell(felt: Vec2) -> (usize, usize) {
    let f = |v: f32, g: f32, half: usize| {
        let v = if v.is_finite() { v } else { 0.0 };
        (half as f32 + v / g).round().clamp(0.0, (2 * half) as f32) as usize
    };
    (
        f(felt.x, SCOPE_G.x, SCOPE_COLS / 2),
        f(felt.y, SCOPE_G.y, SCOPE_ROWS / 2),
    )
}

/// Felt G: accelerating right pushes you left, accelerating forward pushes you back (down).
fn felt_g(sig: &Signals) -> Vec2 {
    let v = vec2(-sig.g_lat, sig.g_long);
    if v.x.is_finite() && v.y.is_finite() {
        v
    } else {
        Vec2::ZERO
    }
}

/// An annunciator that holds briefly after its condition clears.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Lamp {
    hold: f32,
}

impl Lamp {
    fn update(&mut self, on: bool, dt: f32) {
        self.hold = if on { LAMP_HOLD_S } else { (self.hold - dt).max(0.0) };
    }

    fn lit(&self) -> bool {
        self.hold > 0.0
    }
}

/// The faces this cluster sets (resolved once; a user display font replaces the labels, the VFD
/// digits stay procedural).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    label: FontId,
    role_display: FontId,
}

/// The Retro Cyber cluster.
#[derive(Debug, Clone)]
pub struct RetroCyber {
    faces: Faces,
    sig: Signals,
    palette: Palette,
    view: View,
    t: f32,
    light: VirtualLight,
    parallax: Smoothed<Vec2>,
    shift: ShiftLights,
    cue: LimiterCue,
    chevrons: TachChevron,
    gear: Seg7Digit,
    last_gear: Gear,
    /// Seconds since the last gear change (negative: no rim flash running).
    flash_t: f32,
    speed_value: SpeedReadout,
    speed: Seg7Number<3>,
    readout: [Seg7Digit; 2],
    test_speed: Seg7Number<3>,
    test_gear: Seg7Digit,
    test_readout: Seg7Digit,
    scope: [f32; SCOPE_COLS * SCOPE_ROWS],
    columns: [f32; 3],
    drift: usize,
    drift_side: f32,
    lamps: [Lamp; 3],
    boot: Boot,
    mesh: Option<ImageId>,
    phosphor: Option<ImageId>,
    /// The Blender housing per view (cockpit, hood), two key lights each.
    housing: [Option<[ImageId; 2]>; 2],
}

impl RetroCyber {
    /// Builds the cluster: faces, the seven-segment cells and the optional Phase 6 hex mesh.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let t: &Tokens = setup.tokens;
        let palette = Palette::family(t, "sciFi");
        let tech = setup.fonts.role(FontRole::Tech);
        let faces = Faces {
            label: setup.fonts.find("Chakra Petch", 600, false).unwrap_or(tech),
            role_display: setup.fonts.role(FontRole::Display),
        };
        let mut load = |rel: &str| -> Option<ImageId> {
            let path = setup.assets?.join(rel);
            path.is_file()
                .then(|| setup.images.load_file(setup.device, setup.queue, &path, None).ok())
                .flatten()
        };
        let mesh = load("scifi/vfd_hex_mesh.png");
        let phosphor = load("clusters/retro_cyber/phosphor_grain.png");
        let mut pair = |view: &str| -> Option<[ImageId; 2]> {
            let a = load(&format!("clusters/retro_cyber/housing_{view}_l160.png"))?;
            let b = load(&format!("clusters/retro_cyber/housing_{view}_l110.png"))?;
            Some([a, b])
        };
        let housing = [pair("cockpit"), pair("hood")];
        let gear = Seg7Digit::new(t, Reveal::Sequence)
            .with_segments(Segments::Diagonal)
            .with_phosphor(Phosphor::Ink)
            .with_ghost(Phosphor::Primary);
        let mut test_gear = Seg7Digit::new(t, Reveal::Instant)
            .with_segments(Segments::Diagonal)
            .with_phosphor(Phosphor::Ink)
            .with_ghost(Phosphor::Primary);
        test_gear.update(0x7F | DIAGONAL, 0.0);
        let mut test_speed = Seg7Number::<3>::new(t, Reveal::Instant);
        test_speed.update(888, 0.0);
        let digit = Seg7Digit::new(t, Reveal::Instant).with_phosphor(Phosphor::Secondary);
        let mut test_readout = digit;
        test_readout.update(0x7F, 0.0);
        Self {
            faces,
            sig: Signals::default(),
            palette,
            view: View::Cockpit,
            t: 0.0,
            light: VirtualLight::new(t),
            parallax: Smoothed::new(Vec2::ZERO, 0.08),
            shift: ShiftLights::new(PODS),
            cue: LimiterCue::new(),
            chevrons: TachChevron::new(t),
            gear,
            last_gear: Gear::Neutral,
            flash_t: -1.0,
            speed_value: SpeedReadout::new(),
            speed: Seg7Number::new(t, Reveal::Instant),
            readout: [digit; 2],
            test_speed,
            test_gear,
            test_readout,
            scope: [0.0; SCOPE_COLS * SCOPE_ROWS],
            columns: [0.0; 3],
            drift: 0,
            drift_side: 1.0,
            lamps: [Lamp::default(); 3],
            boot: Boot::default(),
            mesh,
            phosphor,
            housing,
        }
    }

    /// The Blender housing for `view`, if built.
    fn housing(&self, view: View) -> Option<[ImageId; 2]> {
        match view {
            View::Hood => self.housing[1],
            View::Cockpit | View::Chase => self.housing[0],
        }
    }

    /// The parallax offset of VFD layer `layer` (0 phosphor .. 3 glass).
    fn offset(&self, layer: usize) -> Vec2 {
        self.parallax.value() * PARALLAX[layer]
    }
}

impl Cluster for RetroCyber {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = if input.dt.is_finite() { input.dt.max(0.0) } else { 0.0 };
        let sig = *input.signals;
        self.sig = sig;
        self.palette = *input.palette;
        self.view = input.view;
        self.t = input.t;
        if input.fonts.display != self.faces.role_display {
            self.faces.label = input.fonts.display;
            self.faces.role_display = input.fonts.display;
        }
        self.boot.observe(input.telemetry.is_race_on != 0, dt);

        // Light and parallax follow felt G.
        self.light.update(sig.g_lat, dt);
        let felt = felt_g(&sig);
        let dir = felt * (1.0 / PARALLAX_G);
        let len = dir.length();
        let dir = if len > 1.0 { dir * (1.0 / len) } else { dir };
        self.parallax.update(dir, dt);

        // Strip, limiter cue and chevrons.
        self.shift.update(&sig, &self.palette, input.t, dt, input.reduced_flash);
        self.cue.update(self.shift.over_shift(), 0.12, dt);
        self.chevrons.update(&sig, dt);

        // Gear: re-strike, phosphor, neutral at 72%, the 2 Hz pulse after 0.5 s on the limiter.
        if sig.gear != self.last_gear {
            self.last_gear = sig.gear;
            self.flash_t = 0.0;
        } else if self.flash_t >= 0.0 {
            self.flash_t += dt;
            if self.flash_t > RIM_LAP_S + RIM_FLASH_S {
                self.flash_t = -1.0;
            }
        }
        self.gear.set_phosphor(match sig.gear {
            Gear::Reverse => Phosphor::Warning,
            Gear::Neutral | Gear::Forward(_) => Phosphor::Ink,
        });
        let pulse = if self.cue.held() > PULSE_AFTER_S && !input.reduced_flash {
            0.82 + 0.18 * (0.5 + 0.5 * (TAU * PULSE_HZ * (self.cue.held() - PULSE_AFTER_S)).cos())
        } else {
            1.0
        };
        self.gear.set_brightness(match sig.gear {
            Gear::Neutral => NEUTRAL_LEVEL,
            Gear::Reverse | Gear::Forward(_) => pulse,
        });
        self.gear.update_gear(sig.gear, dt);

        // Speed (0.6 hysteresis), instant segments.
        self.speed_value.update(sig.speed);
        self.speed.update(self.speed_value.value(), dt);

        // Drift: body slip from velocity (verified fields); the tail steps out on the slip side.
        let slip = if sig.slip_deg.is_finite() { sig.slip_deg } else { 0.0 };
        self.drift = drift_steps(slip);
        if slip.abs() >= DRIFT_STEPS[0] {
            self.drift_side = slip.signum();
        }
        let shown = if slip.abs() >= SLIP_LAMP_DEG {
            slip.abs().round().min(99.0) as usize
        } else {
            0
        };
        let masks = if shown == 0 {
            [0u16, 0u16]
        } else if shown < 10 {
            [0, u16::from(SEVEN_SEG_DIGITS[shown])]
        } else {
            [
                u16::from(SEVEN_SEG_DIGITS[shown / 10]),
                u16::from(SEVEN_SEG_DIGITS[shown % 10]),
            ]
        };
        for (cell, mask) in self.readout.iter_mut().zip(masks) {
            cell.update(mask, dt);
        }

        // Annunciators (SLIP from body slip; LOCK and SPIN inferred).
        let decel = -sig.g_long;
        let lock = sig.brake > LOCK_BRAKE && sig.speed_ms > LOCK_MIN_SPEED && decel < LOCK_DECEL_PER_BRAKE * sig.brake;
        let spin = sig.throttle > SPIN_THROTTLE && slip.abs() > SPIN_SLIP_DEG;
        for (lamp, on) in self.lamps.iter_mut().zip([slip.abs() >= SLIP_LAMP_DEG, lock, spin]) {
            lamp.update(on, dt);
        }

        // Columns light in steps with no smoothing (a VFD has no needle).
        let boost = input.telemetry.boost_psi;
        let boost = if boost.is_finite() { boost / BOOST_FULL_PSI } else { 0.0 };
        self.columns = [sig.throttle, sig.brake, boost].map(|v| if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 });

        // G-scope: the current cell lights, the trail fades over 500 ms.
        let decay = dt / SCOPE_TRAIL_S;
        for l in &mut self.scope {
            *l = (*l - decay).max(0.0);
        }
        let (col, row) = scope_cell(felt);
        self.scope[row * SCOPE_COLS + col] = 1.0;
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = geom(self.view);
        let pal = self.palette;
        let boot_t = self.boot.t();

        // 1. The housing: the opaque body and the chamfer45 ring lit by the virtual light.
        draw_housing(p, g, &pal, self.light.angle(), self.housing(self.view));

        // 2. Phosphor (parallax 0): everything except the live gear and speed.
        p.scoped(|p| {
            p.translate(self.offset(0));
            self.draw_strip(p, g, &pal, false);
            self.draw_chevrons(p, g, &pal, false);
            self.chevrons
                .draw_scale(p, g.chevrons, self.faces.label, g.scale_cap, &pal);
            self.draw_bay(p, g, &pal);
            self.draw_drift(p, g, &pal, false);
            self.draw_columns(p, g, &pal, false);
            self.draw_scope(p, g, &pal, false);
            self.draw_lamps(p, g, &pal, false);
            self.draw_readout(p, g, &pal, false);
            self.draw_labels(p, g, &pal);
        });

        // 3. Boot: sectors not yet ignited are cold glass (black), filaments warming.
        if let Some(t) = boot_t {
            for k in 0..SECTORS {
                if t < sector_start(k) {
                    p.fill(&Shape::rect(g.sector(k)), pal.roles.oled_black);
                }
            }
        }

        // 4. Live from frame 1: the gear (rank 1) and the speed (rank 2).
        p.scoped(|p| {
            p.translate(self.offset(0));
            let held = self.cue.level() > 0.5;
            let emission = if held { Emission::Soft } else { GEAR_EMISSION };
            let gg = Seg7Geometry::new(g.gear_cap);
            self.gear.draw(p, g.bay.center(), &gg, &pal, emission);
            let sg = Seg7Geometry::new(g.speed_cap);
            self.speed.draw(p, g.speed, &sg, &pal, Emission::Bloom);
        });

        // 5. Boot self-test: every segment of the igniting sectors, then the phosphor decay.
        if let Some(t) = boot_t {
            for k in 0..SECTORS {
                let level = sector_level(t, k);
                if level > 0.0 {
                    p.scoped(|p| {
                        p.clip(g.sector(k), 0.0);
                        p.set_opacity(level);
                        p.translate(self.offset(0));
                        self.draw_test(p, g, &pal);
                    });
                }
            }
        }

        // 6. Control-grid mesh (+1 px) and filaments (+3 px), multiplied: they only modulate lit
        //    phosphor. Then the scanlines over the cavity band.
        let band = g.band();
        let zones = g.zones();
        p.scoped(|p| {
            p.set_blend(BlendMode::Multiply);
            p.scoped(|p| {
                p.translate(self.offset(1));
                for z in zones {
                    self.draw_mesh(p, z, band.min);
                }
            });
            p.translate(self.offset(2));
            let px = p.device_px();
            for y in g.filaments {
                p.fill(
                    &Shape::rect(rect(band.min.x, y - px * 0.5, band.max.x, y + px * 0.5)),
                    pal.roles.oled_black.with_alpha(FILAMENT_DARK),
                );
            }
        });
        for z in zones {
            draw_scanlines(p, z, &pal);
        }
        if let Some(t) = boot_t {
            if t < FILAMENT_WARM_S {
                p.scoped(|p| {
                    p.set_blend(BlendMode::Add);
                    p.translate(self.offset(2));
                    let px = p.device_px();
                    let warm = pal.roles.ink.with_alpha(srgb_to_linear(FILAMENT_WARM_LEVEL));
                    for y in g.filaments {
                        p.fill(
                            &Shape::rect(rect(band.min.x, y - px * 0.5, band.max.x, y + px * 0.5)),
                            warm,
                        );
                    }
                });
            }
        }

        // 7. Front glass (+4 px): a light-driven 1 px rim and a faint crescent, never a tint.
        p.scoped(|p| {
            p.translate(self.offset(3));
            // Facet by facet, so each draw only covers its own edge.
            let outer = octagon(g.cavity.inset(3.0), (g.cut - 1.8).max(0.0));
            let inner = octagon(g.cavity.inset(15.0), (g.cut - 9.0).max(0.0));
            let c = g.cavity.center();
            let light = self.light.angle();
            let ink = pal.roles.ink;
            let crescent = lit_rim(c, light, ink.with_alpha(GLASS_CRESCENT), ink.with_alpha(0.0));
            let rim = lit_rim(c, light, ink.with_alpha(GLASS_RIM), ink.with_alpha(0.0));
            let px = p.device_px();
            for k in 0..8 {
                let j = (k + 1) % 8;
                let quad = [outer[k], outer[j], inner[j], inner[k]];
                p.fill(&Shape::Polygon { points: &quad }, crescent);
                p.fill(&Shape::line(outer[k], outer[j], px), rim);
            }
        });

        // 8. Gear change: a 1-frame rim flash laps the octagon's 8 facets in 50 ms.
        if self.flash_t >= 0.0 {
            p.scoped(|p| {
                p.translate(self.offset(0));
                p.set_blend(BlendMode::Add);
                let pts = octagon(g.bay, g.bay_cut);
                for k in 0..8 {
                    let on = k as f32 * RIM_LAP_S / 8.0;
                    if (on..on + RIM_FLASH_S).contains(&self.flash_t) {
                        let (a, b) = (pts[k], pts[(k + 1) % 8]);
                        p.fill(&Shape::line(a, b, 2.0), pal.roles.ink.with_alpha(RIM_FLASH));
                    }
                }
            });
        }
    }

    fn animating(&self) -> bool {
        self.boot.running()
            || self.flash_t >= 0.0
            || self.shift.animating()
            || self.cue.animating()
            || self.chevrons.animating()
            || self.gear.animating()
            || self.speed.animating()
            || self.readout.iter().any(Seg7Digit::animating)
            || self.scope.iter().any(|&l| l > 0.0 && l < 1.0)
            || self.lamps.iter().any(|l| l.lit())
            || self.cue.held() > PULSE_AFTER_S
    }
}

impl RetroCyber {
    /// The 15-pod strip: ghosts in each pod's stage colour, lit pods in one bloom layer (or every
    /// pod lit for the boot self-test).
    fn draw_strip(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        // `ShiftLights` decides which pods are lit (outside-in, 0 ms attack, the 10 Hz strobe
        // and the reduced-flash breath); the VFD stages colour them.
        let over = self.shift.over_shift() && !all;
        let lit = |i: usize| -> Option<Color> {
            if all {
                return Some(pod_stage(i, pal));
            }
            let c = self.shift.color_of(i, pal)?;
            Some(if over { c } else { pod_stage(i, pal) })
        };
        for i in 0..PODS as usize {
            if lit(i).is_none() {
                let pts = g.pod(i);
                p.fill(&Shape::Polygon { points: &pts }, ghost(pod_stage(i, pal)));
            }
        }
        // Halos first, then every lit body, so the plate fills share one draw call.
        for i in 0..PODS as usize {
            if let Some(c) = lit(i) {
                let pts = g.pod(i);
                p.glow(&Shape::Polygon { points: &pts }, c, STRIP_GLOW);
            }
        }
        for i in 0..PODS as usize {
            if let Some(c) = lit(i) {
                let pts = g.pod(i);
                phosphor_block(p, &pts, c.intensity(PLATE_GAIN), None, self.phosphor);
            }
        }
    }

    /// The 24 chevrons: opaque ghosts, lit steps in phosphor with a short bloom (rank 3: under
    /// the strip's). Steady at the limiter: the strip owns it.
    fn draw_chevrons(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        let n = self.chevrons.count();
        let lit = if all { n } else { self.chevrons.lit() };
        for i in lit..n {
            let (pts, len) = self.chevrons.segment(i, g.chevrons);
            p.fill(
                &Shape::Polygon { points: &pts[..len] },
                ghost(self.chevrons.segment_color(i, pal)),
            );
        }
        if lit == 0 {
            return;
        }
        for i in 0..lit {
            let (pts, len) = self.chevrons.segment(i, g.chevrons);
            let c = self.chevrons.segment_color(i, pal);
            p.glow(&Shape::Polygon { points: &pts[..len] }, c, CHEVRON_GLOW);
        }
        for i in 0..lit {
            let (pts, len) = self.chevrons.segment(i, g.chevrons);
            let c = self.chevrons.segment_color(i, pal).intensity(PLATE_GAIN);
            phosphor_block(p, &pts[..len], c, None, self.phosphor);
        }
    }

    /// The octagon gear bay: an inner lip and a 1 px rim, steady accent at the limiter.
    fn draw_bay(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let pts = octagon(g.bay, g.bay_cut);
        let k = self.cue.level();
        let rim = pal.roles.ink.with_alpha(on_screen(BAY_RIM)).lerp(pal.accent_text, k);
        p.stroke(&Shape::Polygon { points: &pts }, rim, Stroke::new(BAY_RIM_PX));
        let lip = octagon(g.bay.inset(6.0), (g.bay_cut - 6.0 * 0.586).max(0.0));
        p.stroke(
            &Shape::Polygon { points: &lip },
            ghost(pal.primary).lerp(ghost(pal.accent), k),
            Stroke::rim_px(1.0),
        );
    }

    /// The drift columns flanking the bay: ghosts on both sides, the tail side lit in secondary.
    fn draw_drift(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        let lit = pal.secondary;
        let off = ghost(lit);
        // Ghosts first, then the lit steps (the plate fills share one draw call).
        for pass_lit in [false, true] {
            for side in [-1.0, 1.0] {
                for j in 0..DRIFT_STEPS.len() {
                    let on = all || (side == self.drift_side && j < self.drift);
                    if on == pass_lit {
                        let paint = if on {
                            phosphor_paint(self.phosphor, rank4(lit))
                        } else {
                            off.into()
                        };
                        p.fill(&Shape::rect(g.drift_cell(side, j)), paint);
                    }
                }
            }
        }
    }

    /// THR / BRK / BST columns (rank 4: matte, no glow).
    fn draw_columns(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        let roles = pal.roles;
        let colors = [roles.go, roles.warning, pal.secondary];
        // Ghosts first, then the lit steps (the plate fills share one draw call).
        for pass_lit in [false, true] {
            for (c, (&v, color)) in self.columns.iter().zip(colors).enumerate() {
                let lit = if all {
                    COLUMN_STEPS
                } else {
                    (v * COLUMN_STEPS as f32).round() as usize
                };
                for j in 0..COLUMN_STEPS {
                    if (j < lit) == pass_lit {
                        let paint = if pass_lit {
                            phosphor_paint(self.phosphor, rank4(color))
                        } else {
                            ghost(color).into()
                        };
                        p.fill(&Shape::rect(g.column_cell(c, j)), paint);
                    }
                }
            }
        }
    }

    /// The 13 × 5 dot G-scope with its trail; the centre dot is a brighter reference.
    fn draw_scope(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        let phosphor = pal.primary_text;
        let off = ghost(phosphor);
        let r = g.scope_pitch.y * 0.32;
        for row in 0..SCOPE_ROWS {
            for col in 0..SCOPE_COLS {
                let c = g.scope_dot(col, row);
                let level = if all { 1.0 } else { self.scope[row * SCOPE_COLS + col] };
                let centre = col == SCOPE_COLS / 2 && row == SCOPE_ROWS / 2;
                let base = if centre { off.lerp(phosphor, 0.12) } else { off };
                let color = if level > 0.0 {
                    base.lerp(phosphor, level * level)
                } else {
                    base
                };
                p.fill(&Shape::circle(c, r), color);
            }
        }
    }

    /// SLIP / LOCK / SPIN: legends in their colour when active, ghost glass otherwise.
    fn draw_lamps(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        let roles = pal.roles;
        let names = ["SLIP", "LOCK", "SPIN"];
        let colors = [roles.caution, roles.warning, roles.caution];
        for (i, ((lamp, name), color)) in self.lamps.iter().zip(names).zip(colors).enumerate() {
            let r = g.lamp(i);
            let on = all || lamp.lit();
            let ink = if on { color } else { ghost(color) };
            let pts = octagon(r, r.height() * 0.3);
            p.stroke(&Shape::Polygon { points: &pts }, ink, Stroke::rim_px(1.0));
            let style = TextStyle::new(self.faces.label, r.height() * LAMP_CAP)
                .paint(ink)
                .tracking(0.12)
                .align(Align::Center, VAlign::CapMiddle);
            p.text(r.center(), name, &style);
        }
    }

    /// The DRIFT readout: two matte cells in secondary.
    fn draw_readout(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, all: bool) {
        let geom = Seg7Geometry::new(g.readout_cap);
        for (i, cell) in self.readout.iter().enumerate() {
            let c = vec2(g.readout.x + (i as f32 - 0.5) * geom.advance(), g.readout.y);
            let d = if all { &self.test_readout } else { cell };
            d.draw(p, c, &geom, pal, Emission::Matte);
        }
    }

    /// Rank-5 labels: unit, column legends, DRIFT, G.
    fn draw_labels(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let style = TextStyle::new(self.faces.label, g.label_cap)
            .paint(pal.roles.ink_label.intensity(srgb_to_linear(LABEL_LEVEL)))
            .tracking(0.16)
            .align(Align::Center, VAlign::CapTop);
        let below_speed = g.speed.y + g.speed_cap * 0.5 + 14.0;
        p.text(vec2(g.speed.x, below_speed), self.sig.unit.label(), &style);
        let col_y = g.columns.max.y + 12.0;
        for (c, name) in ["THR", "BRK", "BST"].into_iter().enumerate() {
            let x = g.column_cell(c, 0).center().x;
            p.text(vec2(x, col_y), name, &style);
        }
        let rg = Seg7Geometry::new(g.readout_cap);
        p.text(
            vec2(g.readout.x, g.readout.y + g.readout_cap * 0.5 + 12.0),
            "DRIFT",
            &style,
        );
        let deg = TextStyle {
            align: Align::Left,
            ..style
        };
        p.text(
            vec2(g.readout.x + rg.advance() + 4.0, g.readout.y - g.readout_cap * 0.5),
            "°",
            &deg,
        );
        let left = TextStyle {
            align: Align::Right,
            valign: VAlign::CapMiddle,
            ..style
        };
        let scope_left = g.scope_dot(0, SCOPE_ROWS / 2).x - g.scope_pitch.x;
        p.text(vec2(scope_left, g.scope.y), "G", &left);
    }

    /// The boot self-test: every segment lit.
    fn draw_test(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        self.draw_strip(p, g, pal, true);
        self.draw_chevrons(p, g, pal, true);
        let gg = Seg7Geometry::new(g.gear_cap);
        self.test_gear.draw(p, g.bay.center(), &gg, pal, Emission::Matte);
        let sg = Seg7Geometry::new(g.speed_cap);
        self.test_speed.draw(p, g.speed, &sg, pal, Emission::Matte);
        self.draw_drift(p, g, pal, true);
        self.draw_columns(p, g, pal, true);
        self.draw_scope(p, g, pal, true);
        self.draw_lamps(p, g, pal, true);
        self.draw_readout(p, g, pal, true);
    }

    /// The control-grid mesh over `zone` (multiply, so it only modulates lit phosphor), tiled
    /// from `origin` so every zone shares one grid: the Phase 6 hex tile, or a procedural
    /// triangular lattice of three hairline families.
    fn draw_mesh(&self, p: &mut Painter<'_>, zone: Rect, origin: Vec2) {
        let strength = MESH_STRENGTH;
        let band = zone;
        match self.mesh {
            Some(img) => {
                let tile = Rect::from_xywh(origin.x, origin.y, MESH_TILE, MESH_TILE);
                let mut paint = ImagePaint::new(img, tile).tint(Color::WHITE.with_alpha(strength));
                paint.wrap = ImageWrap::Repeat;
                p.fill(&Shape::rect(band), paint);
            }
            None => {
                // Three line families at 0°, 60° and 120°, 6 px apart, 1 px wires.
                let dark = Color::BLACK.with_alpha(strength * 0.6);
                let clear = dark.with_alpha(0.0);
                for angle in [0.0_f32, 60.0, 120.0] {
                    let n = Vec2::from_angle(angle.to_radians() + core::f32::consts::FRAC_PI_2);
                    let o = origin;
                    let paint = Gradient::linear(o, o + n * 6.0)
                        .stop(0.0, dark)
                        .stop(1.0 / 6.0, dark)
                        .stop(1.0 / 6.0, clear)
                        .stop(1.0, clear)
                        .spread(fh6_kit::Spread::Repeat);
                    p.fill(&Shape::rect(band), paint);
                }
            }
        }
    }
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(RetroCyber::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inside(outer: Rect, inner: Rect) -> bool {
        outer.contains(inner.min) && outer.contains(inner.max)
    }

    /// Inside the octagon of `r` with corner cut `cut`, with `margin` px to spare.
    fn in_octagon(r: Rect, cut: f32, p: Vec2, margin: f32) -> bool {
        let c = r.center();
        let (hx, hy) = (r.width() * 0.5, r.height() * 0.5);
        let (dx, dy) = ((p.x - c.x).abs(), (p.y - c.y).abs());
        dx <= hx - margin && dy <= hy - margin && (dx + dy) <= hx + hy - cut - margin * core::f32::consts::SQRT_2
    }

    #[test]
    fn layout_stays_inside_the_view_envelopes() {
        let t = Tokens::default();
        for (g, env) in [(&COCKPIT, t.layout.cockpit_envelope), (&HOOD, t.layout.hood_envelope)] {
            assert!(inside(env, g.cavity.expand(10.0)), "tube {:?} in {env:?}", g.cavity);
            for r in [g.strip, g.chevrons, g.bay, g.columns, g.lamps] {
                assert!(inside(g.cavity.inset(8.0), r), "{r:?} inside the cavity");
                for corner in [r.min, r.max, vec2(r.min.x, r.max.y), vec2(r.max.x, r.min.y)] {
                    assert!(in_octagon(g.cavity, g.cut, corner, 4.0), "{corner:?} clear of the cut");
                }
            }
            assert!(g.strip.height() <= 42.0, "the strobing strip is <= 42 px tall");
            // The scale numerals sit between the chevrons and the bay.
            assert!(g.chevrons.max.y + g.scale_cap * 1.9 < g.bay.min.y || g.chevrons.max.x < g.bay.min.x);
            // The scope, speed and readout stay in the cavity.
            for pt in [
                g.scope_dot(0, 0),
                g.scope_dot(SCOPE_COLS - 1, SCOPE_ROWS - 1),
                g.speed + vec2(0.0, g.speed_cap * 0.5 + 40.0),
                g.readout + vec2(0.0, g.readout_cap * 0.5 + 30.0),
            ] {
                assert!(in_octagon(g.cavity, g.cut, pt, 4.0), "{pt:?} in the cavity");
            }
        }
    }

    #[test]
    fn the_gear_fits_its_octagon_bay() {
        for g in [&COCKPIT, &HOOD] {
            let geo = Seg7Geometry::new(g.gear_cap);
            // Every corner of every slanted segment and the N diagonal keeps 6 px from the rim.
            let c = g.bay.center();
            let diagonal = geo.diagonal(c);
            for i in 0..7 {
                for pt in geo.segment(c, i).iter().chain(diagonal.iter()) {
                    assert!(in_octagon(g.bay, g.bay_cut, *pt, 6.0), "{pt:?} inside the bay");
                }
            }
            // Drift columns clear the bay.
            let left = g.drift_cell(-1.0, 0);
            assert!(left.max.x < g.bay.min.x - 16.0);
        }
    }

    #[test]
    fn the_speed_clears_the_drift_column_and_the_scope() {
        for g in [&COCKPIT, &HOOD] {
            let sg = Seg7Geometry::new(g.speed_cap);
            let right = g.speed.x + sg.advance() * 1.5;
            assert!(right < g.drift_cell(-1.0, 0).min.x - 24.0);
            // The scope (with its "G" label, about 2 pitches to its left) sits clear of the speed
            // and of the drift column.
            let scope_left = g.scope_dot(0, 0).x - g.scope_pitch.x * 2.5;
            assert!(right + 24.0 < scope_left || g.speed.y + g.speed_cap * 0.5 < g.scope_dot(0, 0).y);
            assert!(g.scope_dot(SCOPE_COLS - 1, 0).x + 16.0 < g.drift_cell(-1.0, 0).min.x);
        }
    }

    #[test]
    fn boot_runs_only_on_the_first_observed_race_start() {
        let mut b = Boot::default();
        // Made while racing: no boot.
        b.observe(true, 0.016);
        assert!(!b.running());
        b.observe(false, 0.016);
        b.observe(true, 0.016);
        assert!(b.running(), "0 -> 1 boots");
        for _ in 0..40 {
            b.observe(true, 1.0 / 60.0);
        }
        assert!(!b.running(), "the boot ends within {BOOT_S} s");
        b.observe(false, 0.016);
        b.observe(true, 0.016);
        assert!(!b.running(), "only the first race start after launch boots");
        const _: () = assert!(BOOT_S <= 1.0);
    }

    #[test]
    fn sectors_ignite_left_to_right_and_settle() {
        assert_eq!(sector_level(0.1, 0), 0.0, "filaments warm first");
        assert_eq!(sector_level(0.16, 0), 1.0);
        assert_eq!(sector_level(0.16, 1), 0.0);
        assert_eq!(sector_level(sector_start(7) + 0.01, 7), 1.0);
        assert!(sector_level(BOOT_S, 7) < 1e-4, "settled to ghost");
        for k in 1..SECTORS {
            assert!((sector_start(k) - sector_start(k - 1) - SECTOR_S).abs() < 1e-6);
        }
    }

    #[test]
    fn drift_and_scope_mapping() {
        assert_eq!(drift_steps(3.0), 0);
        assert_eq!(drift_steps(-27.0), 4);
        assert_eq!(drift_steps(60.0), 6);
        assert_eq!(drift_steps(f32::NAN), 0);
        assert_eq!(scope_cell(Vec2::ZERO), (6, 2));
        assert_eq!(scope_cell(vec2(0.96, -0.58)), (10, 1));
        assert_eq!(scope_cell(vec2(9.0, 9.0)), (12, 4));
        assert_eq!(scope_cell(vec2(f32::NAN, 0.0)), (6, 2));
    }

    #[test]
    fn ghosts_are_opaque_and_dim() {
        let c = Color::linear(0.0, 1.0, 0.8, 1.0);
        let gh = ghost(c);
        assert_eq!(gh.a, 1.0);
        assert!((linear_to_srgb(gh.g) - GHOST_LEVEL).abs() < 1e-3);
    }
}
