//! Draws the OSD with the render kit: the dimmed backdrop, the live-zone header, focus echo
//! and fit schematic, and the Liquid Glass control panel (tabs, rows, carousel thumbnails,
//! keyboard and remap cards, hint bar). Authored in 4K units (3840×2160, scaled by
//! `height / 2160`); the panel stays right-aligned on wider screens. Allocation-free.
use crate::anim::{AnimTargets, OsdAnim};
use crate::fit::CockpitFit;
use crate::keyboard::{self, Key};
use crate::model::{self, EditState, Line, RowId, RowKind, RowList, Tab, MAX_ROWS};
use crate::nav::{Mode, OsdController};
use crate::remap;
use fh6_core::{AppConfig, ClusterTheme, ScrimFit};
use fh6_kit::optics::absorption_coefficient;
use fh6_kit::scrim::{self, Wash};
use fh6_kit::{
    deg, vec2, Affine, Align, Cluster, Color, Environment, FontId, FontRole, Fonts, Glow, Gradient, ImageId,
    LayerOptions, Layout, OpticalGlass, Painter, Palette, Piece, Rect, Refraction, Setup, Shadow, Shape, Stroke,
    TextStyle, VAlign, Vec2, View,
};
use std::fmt::Write as _;
use std::time::Duration;

/// Reference size the OSD is authored in.
pub const REF_H: f32 = 2160.0;
/// Panel bounds (4K, before the right-align offset).
pub const PANEL: Rect = Rect {
    min: vec2(2380.0, 140.0),
    max: vec2(3700.0, 2020.0),
};
const PANEL_R: f32 = 56.0;
const CX0: f32 = 2460.0;
const CX1: f32 = 3620.0;
/// The one left text edge in the panel.
const TX: f32 = CX0 + 28.0;
/// The one right edge of every value, slider track and toggle.
const VX: f32 = CX1 - 68.0;
const TABS_Y: f32 = 276.0;
const ROWS_Y0: f32 = 424.0;
const ROWS_Y1: f32 = 1736.0;
const HINT_Y: f32 = 1846.0;
const NOTE_Y: f32 = 1972.0;
const LABEL_CAP: f32 = 30.0;
const VALUE_CAP: f32 = 30.0;
const HELP_CAP: f32 = 26.0;
const SECTION_CAP: f32 = 24.0;
const SECTION_GAP: f32 = 76.0;
const TAB_CAP: f32 = 30.0;
/// Smallest tab cap when the labels have to shrink to fit.
const TAB_MIN_CAP: f32 = 26.0;
/// Hint and status text.
const HINT_CAP: f32 = 26.0;
/// Black pool behind the focus echo.
const ECHO_SCRIM: f32 = 0.45;
/// Black pool behind the live cluster.
const LIVE_SCRIM: f32 = 0.35;
/// Cluster envelope width the thumbnails are fitted to (tokens cockpit envelope).
const ENVELOPE_W: f32 = 1440.0;

/// Screen geometry for a target size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// Target pixels per 4K unit.
    pub k: f32,
    /// Extra width (4K units) on screens wider than 16:9; the panel shifts right by it.
    pub ox: f32,
    /// Target size.
    pub size: (u32, u32),
}

impl Frame {
    /// The frame for `size`.
    #[must_use]
    pub fn new(size: (u32, u32)) -> Self {
        let k = size.1.max(1) as f32 / REF_H;
        let ox = (size.0.max(1) as f32 / k - 3840.0).max(0.0);
        Self { k, ox, size }
    }

    /// Centre of the live zone (left of the panel), 4K units.
    #[must_use]
    pub fn live_centre_x(&self) -> f32 {
        (PANEL.min.x + self.ox) * 0.5
    }
}

/// Where the live cluster's data comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LiveStatus {
    /// The game is paused or not sending: the live dyno drives the cluster.
    #[default]
    Dyno,
    /// Racing telemetry is flowing.
    Telemetry,
}

/// One carousel cell.
#[derive(Clone, Copy)]
pub struct Thumb<'a> {
    /// The cluster's theme.
    pub theme: ClusterTheme,
    /// The cluster, already updated this frame.
    pub cluster: &'a dyn Cluster,
}

/// Everything the OSD draw reads.
#[derive(Clone, Copy)]
pub struct DrawInput<'a> {
    /// Navigation state.
    pub ctl: &'a OsdController,
    /// The working config.
    pub cfg: &'a AppConfig,
    /// The cockpit-fit backend.
    pub fit: &'a dyn CockpitFit,
    /// Motion.
    pub anim: &'a OsdAnim,
    /// Screen geometry.
    pub frame: Frame,
    /// Monotonic time (notices, capture progress).
    pub now: Duration,
    /// The live cluster's display name.
    pub live_name: &'static str,
    /// Where the live data comes from.
    pub status: LiveStatus,
    /// Carousel cells, in order −2..=+2 around the selected cluster (may be empty).
    pub thumbs: &'a [Option<Thumb<'a>>; 5],
    /// The studio environment the glass mirrors (see [`load_environment`]).
    pub env: Option<ImageId>,
}

/// A row's place in the scrolling content (relative to its top).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RowGeom {
    /// Top of the row.
    pub top: f32,
    /// Height.
    pub h: f32,
    /// A section heading sits above the row.
    pub section: bool,
}

/// A row's height; a focused row with a help line grows so the 26 px help clears the focus rim.
fn row_height(kind: RowKind, help_shown: bool) -> f32 {
    match (kind, help_shown) {
        (RowKind::Slider(_), false) => 164.0,
        (RowKind::Slider(_), true) => 208.0,
        (RowKind::Carousel, _) => 340.0,
        (RowKind::Info, false) => 124.0,
        (_, false) => 116.0,
        (_, true) => 148.0,
    }
}

/// The layout of `rows` (row `focus` focused) and the content height.
#[must_use]
pub fn geometry(rows: &RowList, fit: &dyn CockpitFit, edit: &EditState, focus: usize) -> ([RowGeom; MAX_ROWS], f32) {
    let mut out = [RowGeom::default(); MAX_ROWS];
    let mut y = 0.0;
    for (i, row) in rows.as_slice().iter().enumerate() {
        let section = model::section(*row).is_some();
        if section {
            y += SECTION_GAP;
        }
        let help_shown = i == focus && model::help(*row, edit).is_some();
        let h = row_height(model::kind(*row, fit), help_shown);
        out[i] = RowGeom { top: y, h, section };
        y += h;
    }
    (out, y)
}

/// Tab label centres and widths, and the cap they are set at.
#[must_use]
pub fn tab_slots(fonts: &Fonts) -> ([f32; 5], [f32; 5], f32) {
    let font = fonts.role(FontRole::Text);
    let mut widths = [0.0f32; 5];
    let mut cap = TAB_CAP;
    for _ in 0..3 {
        let style = TextStyle::new(font, cap);
        for (i, tab) in Tab::ALL.into_iter().enumerate() {
            widths[i] = fonts.measure(tab.label(), &style).width;
        }
        let total: f32 = widths.iter().sum();
        let avail = (CX1 - 28.0) - TX - 4.0 * 28.0;
        if total <= avail || total <= 0.0 {
            break;
        }
        cap = (cap * avail / total).max(TAB_MIN_CAP);
    }
    let total: f32 = widths.iter().sum();
    let gap = (((CX1 - 28.0) - TX - total) / 4.0).max(0.0);
    let mut centres = [0.0f32; 5];
    let mut x = TX;
    for i in 0..5 {
        centres[i] = x + widths[i] * 0.5;
        x += widths[i] + gap;
    }
    (centres, widths, cap)
}

/// Where the animations should head for the current state.
#[must_use]
pub fn targets(ctl: &OsdController, cfg: &AppConfig, fit: &dyn CockpitFit, fonts: &Fonts, frame: Frame) -> AnimTargets {
    let rows = model::rows(ctl.tab(), ctl.ctx(cfg, fit));
    let fi = ctl.focus_index().min(rows.len().saturating_sub(1));
    let (geo, total) = geometry(&rows, fit, &ctl.edit, fi);
    let g = geo[fi];
    let visible = ROWS_Y1 - ROWS_Y0;
    let max_scroll = (total - visible + 24.0).max(0.0);
    let scroll = (g.top + g.h * 0.5 - visible * 0.5).clamp(0.0, max_scroll);
    let (centres, widths, _) = tab_slots(fonts);
    let ti = ctl.tab().index();
    AnimTargets {
        glide: frame.live_centre_x(),
        focus_y: g.top,
        focus_h: g.h,
        underline_x: centres[ti],
        underline_w: widths[ti] + 24.0,
        scroll,
    }
}

#[derive(Clone, Copy)]
struct Ink {
    pal: Palette,
    /// OSD accent (fills, rings): the palette accent unless it sits near the warning red.
    acc: Color,
    /// The same accent lifted for text.
    acc_text: Color,
    text: FontId,
    display: FontId,
}

impl Ink {
    fn ink(&self) -> Color {
        self.pal.roles.ink
    }
    fn muted(&self) -> Color {
        self.pal.roles.ink_muted
    }
    fn ghost(&self) -> Color {
        self.pal.roles.ink_ghost
    }
    fn hair(&self) -> Color {
        self.pal.roles.hairline
    }
    fn accent(&self) -> Color {
        self.acc_text
    }
    fn style(&self, font: FontId, cap: f32, paint: Color, align: Align) -> TextStyle {
        TextStyle::new(font, cap).paint(paint).align(align, VAlign::CapMiddle)
    }
    fn section(&self) -> TextStyle {
        TextStyle::new(self.text, SECTION_CAP)
            .paint(self.muted())
            .align(Align::Left, VAlign::CapMiddle)
    }
}

/// How dark the game is dimmed behind the OSD. The Phase 3 direction pairs about 45% black with
/// a σ 22 blur; the overlay cannot blur a game it does not capture, so it dims harder instead.
pub const DIM: f32 = 0.62;

/// The black underlay alpha that dims the game behind the OSD at `open` progress. Hosts pass
/// it to `Kit::set_underlay`, which applies it in the resolve: a full-screen shape would cost
/// about 0.9 ms of 4K blending, the underlay nothing measurable.
#[must_use]
pub fn underlay(open: f32) -> f32 {
    DIM * open.clamp(0.0, 1.0)
}

/// A soft black ellipse (centre and radii in the current local units), `alpha` at the core.
fn feathered_ellipse(p: &mut Painter<'_>, centre: Vec2, radii: Vec2, alpha: f32) {
    p.save();
    let world = p.current_transform();
    p.set_transform(
        Affine::scale_xy(radii.x, radii.y)
            .then(&Affine::translate(centre))
            .then(&world),
    );
    p.fill(
        &Shape::circle(Vec2::ZERO, 1.0),
        Gradient::radial(Vec2::ZERO, 1.0)
            .stop(0.0, Color::BLACK.with_alpha(alpha))
            .stop(0.55, Color::BLACK.with_alpha(alpha * 0.75))
            .stop(1.0, Color::BLACK.with_alpha(0.0)),
    );
    p.restore();
}

/// A soft black pool behind the live cluster (in target pixels from `layout`), so the game's
/// own lit gauges do not compete with it. Hosts draw it before the live cluster.
pub fn live_scrim(p: &mut Painter<'_>, layout: &Layout, open: f32) {
    p.save();
    p.set_transform(Affine::IDENTITY);
    let centre = layout.to_target(vec2(0.0, -10.0));
    let k = layout.scale;
    feathered_ellipse(p, centre, vec2(820.0 * k, 400.0 * k), LIVE_SCRIM * open.clamp(0.0, 1.0));
    p.restore();
}

/// What sits under the live cluster: the real dash blocker (Phase 8 dome and wash, in the
/// layout's local units) while a blocker row has focus, so its shape and fit can be judged
/// against the cluster; otherwise the soft pool that keeps the cluster legible.
pub fn live_backing(p: &mut Painter<'_>, layout: &Layout, open: f32, blocker: Option<(&ScrimFit, Wash)>) {
    match blocker {
        Some((fit, wash)) => {
            scrim::draw_scrim(p, fit);
            scrim::draw_wash(p, fit, wash);
        }
        None => live_scrim(p, layout, open),
    }
}

/// The live layout: the cluster at its real height and scale, gliding to the live zone.
#[must_use]
pub fn live_layout(base: &Layout, anim: &OsdAnim, frame: Frame) -> Layout {
    Layout {
        anchor: vec2(anim.glide.value() * frame.k, base.anchor.y),
        ..*base
    }
}

/// Draws the live-zone overlays and the panel. Call after the live cluster.
pub fn ui(p: &mut Painter<'_>, inp: &DrawInput<'_>) {
    let pal = *p.palette();
    let (acc, acc_text) = osd_accent(&pal);
    let ink = Ink {
        pal,
        acc,
        acc_text,
        text: p.fonts().role(FontRole::Text),
        display: p.fonts().role(FontRole::Display),
    };
    let open = inp.anim.open.progress();
    let eased = 1.0 - (1.0 - open) * (1.0 - open);
    p.save();
    p.set_transform(Affine::scale(inp.frame.k));
    p.set_opacity(eased);
    live_zone(p, inp, &ink);
    p.translate(vec2(inp.frame.ox + (1.0 - eased) * 64.0, 0.0));
    panel(p, inp, &ink);
    p.restore();
}

// ----- live zone -----------------------------------------------------------------------------

fn live_zone(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink) {
    let x = 160.0;
    p.text(vec2(x, 196.0), "Live view", &ink.section());
    p.text(
        vec2(x, 270.0),
        inp.live_name,
        &ink.style(ink.display, 54.0, ink.ink(), Align::Left),
    );
    let (dot, status) = match inp.status {
        LiveStatus::Dyno => (ink.accent(), "Game paused · live dyno"),
        LiveStatus::Telemetry => (ink.pal.roles.go, "Live telemetry"),
    };
    p.fill(&Shape::circle(vec2(x + 8.0, 342.0), 8.0), dot);
    let mut line = Line::new();
    let _ = write!(line, "{status} · {} view", model::view_label(inp.ctl.edit.view));
    p.text(
        vec2(x + 32.0, 342.0),
        line.as_str(),
        &ink.style(ink.text, HINT_CAP, ink.muted(), Align::Left),
    );

    let cx = inp.frame.live_centre_x();
    // A feathered black pool behind the echo, so 110 px type holds over a bright sky.
    feathered_ellipse(p, vec2(cx, 880.0), vec2(700.0, 330.0), ECHO_SCRIM);
    match inp.ctl.mode() {
        Mode::Browse => focus_echo(p, inp, ink, cx),
        Mode::Keyboard => {
            p.text(vec2(cx, 740.0), "Custom name", &centered(ink.section()));
            let name = inp.ctl.keyboard().text.as_str();
            p.text(
                vec2(cx, 860.0),
                if name.is_empty() { " " } else { name },
                &ink.style(ink.display, 96.0, ink.ink(), Align::Center),
            );
        }
        Mode::Remap => {
            p.text(vec2(cx, 740.0), "New OSD button", &centered(ink.section()));
            let mut l = Line::new();
            let cand = inp.ctl.remap().candidate();
            if cand == 0 {
                let _ = l.write_str("Hold a button");
            } else {
                remap::mask_label(cand, &mut l);
            }
            p.text(
                vec2(cx, 860.0),
                l.as_str(),
                &ink.style(
                    ink.display,
                    96.0,
                    if cand == 0 { ink.muted() } else { ink.ink() },
                    Align::Center,
                ),
            );
        }
    }
}

fn centered(mut s: TextStyle) -> TextStyle {
    s.align = Align::Center;
    s
}

/// A heading in sentence case ("FIELD OF VIEW" becomes "Field of view"): Liquid Glass lists no
/// longer set section headers in capitals. A single short word ("OSD") is an acronym and stays.
fn sentence_case(text: &str, out: &mut Line) {
    out.clear();
    if !text.contains(' ') && text.chars().count() <= 3 {
        let _ = out.write_str(text);
        return;
    }
    for (i, ch) in text.chars().enumerate() {
        let _ = out.write_char(if i == 0 {
            ch.to_ascii_uppercase()
        } else {
            ch.to_ascii_lowercase()
        });
    }
}

fn focus_echo(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink, cx: f32) {
    let Some(row) = inp.ctl.focused_row(inp.cfg, inp.fit) else {
        return;
    };
    let ctx = inp.ctl.ctx(inp.cfg, inp.fit);
    p.text(vec2(cx, 740.0), model::label(row, inp.fit), &centered(ink.section()));
    let mut value = Line::new();
    model::format_value(row, ctx, &mut value);
    p.text(
        vec2(cx, 860.0),
        value.as_str(),
        &ink.style(ink.display, 110.0, ink.ink(), Align::Center),
    );
    if let RowKind::Slider(spec) = model::kind(row, inp.fit) {
        let f = spec.fraction(model::slider_value(row, ctx));
        let track = Rect::from_xywh(cx - 360.0, 966.0, 720.0, 8.0);
        p.fill(&Shape::round_rect(track, 4.0), ink.ghost());
        p.fill(
            &Shape::round_rect(Rect::from_xywh(track.min.x, track.min.y, 720.0 * f, 8.0), 4.0),
            ink.acc,
        );
    }
    if let Some(help) = model::help(row, &inp.ctl.edit) {
        p.text(
            vec2(cx, 1030.0),
            help,
            &ink.style(ink.text, HINT_CAP, ink.muted(), Align::Center),
        );
    }
    if model::is_camera_row(row) {
        camera_schematic(p, inp, ink, row, cx);
    }
    if row == RowId::ClusterHeight {
        default_height_line(p, inp, ink);
    }
}

/// A side view of the driving position: eye point (seat height), FOV wedge, wheel.
fn camera_schematic(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink, row: RowId, cx: f32) {
    let cfg = inp.cfg;
    let fov = match row {
        RowId::FovHood => cfg.bonnet_fov,
        RowId::FovChase => cfg.chase_near_fov,
        _ => cfg.wheel_less_cockpit_fov,
    };
    // Everything stays inside this box, between the focus echo and the cluster.
    let bx = Rect::from_xywh(cx - 420.0, 1080.0, 840.0, 280.0);
    p.save();
    p.clip(bx, 24.0);
    p.fill(&Shape::round_rect(bx, 24.0), Color::BLACK.with_alpha(0.35));
    let origin = vec2(bx.min.x + 90.0, bx.min.y + 130.0);
    // 1.6 units per mm of seat travel: the whole −100..+50 mm range fits the box.
    let seat_y = |mm: f32| origin.y - mm * 1.6;
    let ground = bx.max.y - 26.0;
    p.fill(
        &Shape::line(vec2(bx.min.x + 24.0, ground), vec2(bx.max.x - 24.0, ground), 2.0),
        ink.hair(),
    );
    let eye = vec2(origin.x, seat_y(cfg.driver_seat_height_m * 1000.0));
    let half = deg(fov.clamp(20.0, 140.0) * 0.5);
    let reach = 1400.0;
    let wedge = [
        eye,
        eye + vec2(half.cos(), -half.sin()) * reach,
        eye + vec2(half.cos(), half.sin()) * reach,
    ];
    p.fill(&Shape::Polygon { points: &wedge }, ink.acc.with_alpha(0.12));
    p.fill(&Shape::line(wedge[0], wedge[1], 2.0), ink.acc.with_alpha(0.6));
    p.fill(&Shape::line(wedge[0], wedge[2], 2.0), ink.acc.with_alpha(0.6));
    // Default eye height (ghost ring) and the wheel rim.
    p.stroke(
        &Shape::circle(vec2(origin.x, seat_y(-45.0)), 12.0),
        ink.muted(),
        Stroke::new(2.0),
    );
    let wheel = vec2(
        origin.x + 300.0,
        origin.y + 40.0 - cfg.steering_wheel_offset_y_m * 1000.0 * 1.6,
    );
    p.fill(&Shape::ring(wheel, 56.0, 9.0), ink.muted());
    p.fill(&Shape::circle(eye, 14.0), ink.acc);
    p.glow(&Shape::circle(eye, 14.0), ink.acc, Glow::new(16.0, 0.5));
    p.restore();
    let mut line = Line::new();
    let _ = write!(
        line,
        "Eye {:+.0} mm  ·  FOV {:.0}°  ·  Wheel {:+.0} mm",
        cfg.driver_seat_height_m * 1000.0,
        fov,
        cfg.steering_wheel_offset_y_m * 1000.0
    );
    p.text(
        vec2(cx, bx.max.y + 40.0),
        line.as_str(),
        &ink.style(ink.text, 24.0, ink.muted(), Align::Center),
    );
}

/// A dashed line where the cluster sits at height offset 0, across the live zone.
fn default_height_line(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink) {
    // The edited view's anchor (Phase 8 per-view placement); a car profile may move it, and the
    // line follows what the live view really uses.
    let y = inp.ctl.edit.live_anchor_y.unwrap_or(match inp.ctl.edit.view {
        View::Hood => inp.cfg.bonnet_anchor_y as f32,
        View::Chase => inp.cfg.chase_anchor_y as f32,
        View::Cockpit => inp.cfg.cockpit_anchor_y as f32,
    });
    let x1 = PANEL.min.x + inp.frame.ox - 120.0;
    let mut x = 200.0;
    while x < x1 {
        p.fill(&Shape::line(vec2(x, y), vec2((x + 28.0).min(x1), y), 3.0), ink.muted());
        x += 48.0;
    }
    p.text(vec2(200.0, y - 36.0), "Default height", &ink.section());
}

// ----- panel --------------------------------------------------------------------------------

/// Left and right edges of the glass pieces: wider than the text column, so every label sits
/// under a level face (`TX` is `GLASS_X0 + 52`, past the 36 px rounded edge).
const GLASS_X0: f32 = CX0 - 24.0;
const GLASS_X1: f32 = CX1 + 24.0;
/// Corner radius of the focus platter.
const PLATTER_R: f32 = 44.0;
/// The glass cross-section: rise of the level face, width of the rounded edge, glass below.
const GLASS_RISE: f32 = 14.0;
const GLASS_BEVEL: f32 = 36.0;
const GLASS_BASE: f32 = 4.0;
/// Small pieces (a key cap, the notice pill) have a shallower cross-section.
const KEY_RISE: f32 = 10.0;
const KEY_BEVEL: f32 = 20.0;
/// The tab bar capsule and the hint capsule.
const BAR_H: f32 = 96.0;
const HINT_H: f32 = 88.0;
/// The notice pill above the tab bar: centre and height.
const NOTICE_Y: f32 = 140.0;
const NOTICE_H: f32 = 60.0;
/// The selected tab's fill, under the tab bar's glass: padding round the label and height.
const TAB_FILL_PAD: f32 = 30.0;
const TAB_FILL_H: f32 = 68.0;
/// Black pool behind the control column: core opacity and softness.
const POOL_ALPHA: f32 = 0.78;
const POOL_SIGMA: f32 = 90.0;
/// Radiance white in `studio_env.png` (`assets/clusters/liquid_glass/ASSETS.md`), how far the
/// studio leans to the car's primary colour, and the key light's share, all as the Liquid
/// Glass cluster sets them, so the two read as one material.
const ENV_WHITE: f32 = 16.0;
const AMBIENT_TINT: f32 = 0.4;
const KEY_SHARE: f32 = 0.5;

/// Loads the studio environment the Liquid Glass cluster's glass mirrors. The image cache
/// keys files by path, so when that cluster is live both share one texture.
pub fn load_environment(setup: &mut Setup<'_>) -> Option<ImageId> {
    let path = setup
        .assets?
        .join("clusters")
        .join("liquid_glass")
        .join("studio_env.png");
    if !path.is_file() {
        return None;
    }
    match setup.images.load_file(setup.device, setup.queue, &path, None) {
        Ok(image) => Some(image),
        Err(error) => {
            // The OSD still draws: its glass mirrors the kit's analytic studio instead.
            tracing::warn!(%error, "the OSD's studio environment did not load");
            None
        }
    }
}

/// The OSD's optical glass: clear sapphire under the Liquid Glass cluster's studio, leaning to
/// the palette's primary colour like the cluster's, with no tint of its own (it holds text).
fn osd_glass(p: &Painter<'_>, env: Option<ImageId>, materialize: f32) -> OpticalGlass {
    let t = p.tokens();
    let op = &t.optics;
    let mut glass = t.optical_glass(p.palette());
    glass.absorption = [absorption_coefficient(op.clarity, op.tint_depth); 3];
    if let Some(env) = env {
        glass.environment = Environment::Map(env);
        glass.env_gain = glass.env_gain.map(|g| g * ENV_WHITE);
    }
    let hue = p.palette().primary;
    let peak = hue.r.max(hue.g).max(hue.b).max(1e-4);
    let lean = [hue.r, hue.g, hue.b].map(|c| 1.0 - AMBIENT_TINT * (1.0 - c / peak));
    for ((env, light), lean) in glass.env_gain.iter_mut().zip(glass.light.iter_mut()).zip(lean) {
        *env *= lean;
        *light *= lean * KEY_SHARE;
    }
    glass.refraction = Refraction::World;
    glass.materialize = materialize;
    glass
}

/// Corner radius of a keyboard key (its glass cap when focused).
const KEY_R: f32 = 18.0;

/// The light a focused piece gives from within: a glow rising from its floor, under the glass,
/// so the rounded edge bends a band of it into the rim (Liquid Glass "illuminates from within").
fn inner_light(p: &mut Painter<'_>, rect: Rect, radius: f32, ink: &Ink) {
    p.fill(
        &Shape::round_rect(rect, radius),
        Gradient::linear(vec2(0.0, rect.min.y), vec2(0.0, rect.max.y))
            .stop(0.0, ink.acc.with_alpha(0.0))
            .stop(0.55, ink.acc.with_alpha(0.05))
            .stop(1.0, ink.acc.with_alpha(0.20)),
    );
}

/// The focused key of the on-screen keyboard, in panel units.
fn focused_key_rect(inp: &DrawInput<'_>) -> Option<Rect> {
    let kb = inp.ctl.keyboard();
    let (first, span) = keyboard::key_span(kb.row, kb.col);
    let x0 = CX0 + ((CX1 - CX0) - (10.0 * KEY_W + 9.0 * KEY_GAP)) * 0.5;
    let y0 = ROWS_Y0 + KEYS_TOP;
    Some(Rect::from_xywh(
        x0 + f32::from(first) * (KEY_W + KEY_GAP),
        y0 + f32::from(kb.row) * (KEY_H + KEY_GAP),
        f32::from(span) * KEY_W + f32::from(span.saturating_sub(1)) * KEY_GAP,
        KEY_H,
    ))
}

/// Keyboard key size, gap and the keys' top below `ROWS_Y0`.
const KEY_W: f32 = 104.0;
const KEY_H: f32 = 100.0;
const KEY_GAP: f32 = 12.0;
const KEYS_TOP: f32 = 290.0;

/// The focused row's platter, in panel units (content scrolled into place).
fn focus_rect(inp: &DrawInput<'_>) -> Rect {
    let fy = ROWS_Y0 + inp.anim.focus_y.value() - inp.anim.scroll.value();
    Rect::from_points(
        vec2(GLASS_X0, fy),
        vec2(GLASS_X1, fy + inp.anim.focus_h.value().max(40.0)),
    )
}

fn panel(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink) {
    // No panel: a feathered black pool keeps the column dark over a bright frame, and on OLED
    // black it is invisible. The rows are light on black; the glass is the navigation: the tab
    // bar, the hint bar and the focus platter.
    let col = Rect::from_points(
        vec2(PANEL.min.x + 60.0, PANEL.min.y + 40.0),
        vec2(PANEL.max.x - 60.0, PANEL.max.y - 40.0),
    );
    p.shadow(
        &Shape::round_rect(col, PANEL_R),
        Shadow {
            offset: Vec2::ZERO,
            sigma: POOL_SIGMA,
            spread: 0.0,
            color: Color::BLACK.with_alpha(POOL_ALPHA),
        },
    );

    // Header and tabs. While the keyboard or remapper has LB/RB, the tabs dim.
    // A notice ("Saved", "Restored") floats above the tab bar in its own glass pill.
    let notice = inp.ctl.notice(inp.now).map(|n| {
        let style = ink.style(ink.text, HINT_CAP, ink.ink(), Align::Center);
        let w = p.measure(n, &style).width + 72.0;
        let pill = Rect::from_center_size(vec2((CX0 + CX1) * 0.5, NOTICE_Y), vec2(w, NOTICE_H));
        p.fill(&Shape::round_rect(pill, NOTICE_H * 0.5), ink.acc.with_alpha(0.18));
        p.text(pill.center(), n, &style);
        pill
    });
    let modal = inp.ctl.mode() != Mode::Browse;
    let tab_k = if modal { 0.4 } else { 1.0 };
    // The selected tab: a soft accent fill under the bar's glass, sliding between tabs.
    let ux = inp.anim.underline_x.value();
    let uw = (inp.anim.underline_w.value() - 24.0).max(0.0) + 2.0 * TAB_FILL_PAD;
    let fill = Rect::from_center_size(vec2(ux, TABS_Y), vec2(uw.max(TAB_FILL_H), TAB_FILL_H));
    p.fill(
        &Shape::round_rect(fill, TAB_FILL_H * 0.5),
        ink.acc.with_alpha(0.22 * tab_k),
    );
    let (centres, _, cap) = tab_slots(p.fonts());
    for (i, tab) in Tab::ALL.into_iter().enumerate() {
        let active = tab == inp.ctl.tab();
        let paint = if active { ink.ink() } else { ink.muted() };
        p.text(
            vec2(centres[i], TABS_Y),
            tab.label(),
            &ink.style(ink.text, cap, paint.fade(tab_k), Align::Center),
        );
    }

    match inp.ctl.mode() {
        Mode::Browse => rows(p, inp, ink),
        Mode::Keyboard => keyboard_card(p, inp, ink),
        Mode::Remap => remap_card(p, inp, ink),
    }
    hint_bar(p, inp, ink);

    // The glass, in one layer over everything it bends.
    let open = inp.anim.open.progress();
    let grown = 1.0 - (1.0 - open) * (1.0 - open);
    let glass = osd_glass(p, inp.env, grown);
    let bar = Rect::from_points(
        vec2(GLASS_X0, TABS_Y - BAR_H * 0.5),
        vec2(GLASS_X1, TABS_Y + BAR_H * 0.5),
    );
    let hint = Rect::from_points(
        vec2(GLASS_X0, HINT_Y - HINT_H * 0.5),
        vec2(GLASS_X1, HINT_Y + HINT_H * 0.5),
    );
    let focus = match inp.ctl.mode() {
        Mode::Browse => {
            Some(Piece::round_rect(focus_rect(inp), PLATTER_R).section(GLASS_RISE, GLASS_BEVEL, GLASS_BASE))
        }
        Mode::Keyboard => {
            focused_key_rect(inp).map(|r| Piece::round_rect(r, KEY_R).section(KEY_RISE, KEY_BEVEL, GLASS_BASE))
        }
        Mode::Remap => None,
    };
    let pieces = [
        Some(Piece::pill(bar).section(GLASS_RISE, GLASS_BEVEL, GLASS_BASE)),
        Some(Piece::pill(hint).section(GLASS_RISE, GLASS_BEVEL, GLASS_BASE)),
        focus,
        notice.map(|r| Piece::pill(r).section(KEY_RISE, KEY_BEVEL, GLASS_BASE)),
    ];
    p.layer(LayerOptions::glass(0.0), |p| {
        for piece in pieces.iter().flatten() {
            p.fill(
                &Shape::Blob {
                    pieces: std::slice::from_ref(piece),
                    smooth: 0.0,
                },
                glass,
            );
        }
    });
}

/// How much of a row (or its section heading) shows inside the viewport, eased: rows fade at
/// the viewport edges instead of being cut through a slider.
fn edge_fade(top: f32, bottom: f32) -> f32 {
    let h = (bottom - top).max(1.0);
    let shown = (bottom.min(ROWS_Y1) - top.max(ROWS_Y0 - 12.0)).max(0.0) / h;
    fh6_kit::math::smoothstep(0.55, 0.98, shown)
}

fn rows(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink) {
    let fit = inp.fit;
    let ctx = inp.ctl.ctx(inp.cfg, fit);
    let list = model::rows(inp.ctl.tab(), ctx);
    let fi = inp.ctl.focus_index().min(list.len().saturating_sub(1));
    let (geo, total) = geometry(&list, fit, &inp.ctl.edit, fi);
    let scroll = inp.anim.scroll.value();
    let visible = ROWS_Y1 - ROWS_Y0;

    p.save();
    p.clip(
        Rect::from_points(vec2(PANEL.min.x, ROWS_Y0 - 12.0), vec2(PANEL.max.x, ROWS_Y1)),
        0.0,
    );
    p.set_opacity(inp.anim.tab_fade.progress());

    inner_light(p, focus_rect(inp), PLATTER_R, ink);

    for (i, row) in list.as_slice().iter().enumerate() {
        let g = geo[i];
        let top = ROWS_Y0 + g.top - scroll;
        if top + g.h < ROWS_Y0 - SECTION_GAP || top > ROWS_Y1 {
            continue;
        }
        if let Some(sec) = model::section(*row) {
            let k = edge_fade(top - SECTION_GAP + 20.0, top - 14.0);
            if k > 0.01 {
                p.save();
                p.set_opacity(k);
                let mut title = Line::new();
                sentence_case(sec, &mut title);
                p.text(vec2(TX, top - 30.0), title.as_str(), &ink.section());
                p.restore();
            }
        }
        let k = edge_fade(top, top + g.h);
        if k > 0.01 {
            p.save();
            p.set_opacity(k);
            draw_row(p, inp, ink, *row, top, i == fi);
            p.restore();
        }
    }

    if total > visible {
        let bar_x = CX1 + 36.0;
        p.fill(
            &Shape::line(vec2(bar_x, ROWS_Y0), vec2(bar_x, ROWS_Y1), 3.0),
            ink.hair(),
        );
        let h = visible * visible / total;
        let y = ROWS_Y0 + (visible - h) * (scroll / (total - visible).max(1.0)).clamp(0.0, 1.0);
        p.fill(
            &Shape::round_rect(Rect::from_xywh(bar_x - 3.0, y, 6.0, h), 3.0),
            ink.muted(),
        );
    }
    p.restore();
}

/// The hue track: the colour wheel at full saturation, so the position reads from the couch.
fn hue_track(rect: Rect) -> Gradient {
    let mut g = Gradient::linear(vec2(rect.min.x, 0.0), vec2(rect.max.x, 0.0));
    for i in 0..=6u8 {
        let t = f32::from(i) / 6.0;
        let [r, gg, b] = crate::palette_edit::Hsv {
            h: t * 358.0,
            s: 1.0,
            v: 1.0,
        }
        .to_rgb();
        g = g.stop(t, Color::srgb8(r, gg, b, 255));
    }
    g
}

fn draw_row(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink, row: RowId, top: f32, focused: bool) {
    let ctx = inp.ctl.ctx(inp.cfg, inp.fit);
    let kind = model::kind(row, inp.fit);
    let ly = top + 46.0;
    let label_paint = if focused {
        ink.ink()
    } else {
        ink.pal.roles.ink.with_alpha(0.86)
    };
    p.text(
        vec2(TX, ly),
        model::label(row, inp.fit),
        &ink.style(ink.text, LABEL_CAP, label_paint, Align::Left),
    );
    let mut value = Line::new();
    model::format_value(row, ctx, &mut value);
    let value_paint = if focused { ink.accent() } else { ink.muted() };
    let vstyle = ink.style(ink.text, VALUE_CAP, value_paint, Align::Right);
    // Every value, track and toggle ends at VX; chevrons sit in the gutter to its right.
    let chevron_tip = CX1 - 28.0;
    match kind {
        RowKind::Slider(spec) => {
            p.text(vec2(VX, ly), value.as_str(), &vstyle);
            let f = spec.fraction(model::slider_value(row, ctx));
            let w = VX - TX;
            let ty = top + 104.0;
            let track = Rect::from_xywh(TX, ty - 5.0, w, 10.0);
            if row == RowId::Hue {
                p.fill(&Shape::round_rect(track, 5.0), hue_track(track));
            } else {
                p.fill(&Shape::round_rect(track, 5.0), ink.ghost());
                p.fill(
                    &Shape::round_rect(Rect::from_xywh(TX, ty - 5.0, w * f, 10.0), 5.0),
                    ink.acc,
                );
            }
            let thumb = vec2(TX + w * f, ty);
            if focused {
                p.glow(&Shape::circle(thumb, 22.0), ink.acc, Glow::new(14.0, 0.5));
            }
            p.fill(&Shape::circle(thumb, 22.0), ink.ink());
            if focused {
                if let Some(help) = model::help(row, &inp.ctl.edit) {
                    p.text(
                        vec2(TX, top + 150.0),
                        help,
                        &ink.style(ink.text, HELP_CAP, ink.muted(), Align::Left),
                    );
                }
            }
            return;
        }
        RowKind::Toggle => {
            let on = model::toggle_value(row, inp.cfg);
            let pill = Rect::from_xywh(VX - 96.0, ly - 24.0, 96.0, 48.0);
            p.fill(&Shape::round_rect(pill, 24.0), if on { ink.acc } else { ink.ghost() });
            let knob = vec2(if on { pill.max.x - 24.0 } else { pill.min.x + 24.0 }, ly);
            p.fill(&Shape::circle(knob, 18.0), if on { ink.ink() } else { ink.muted() });
            p.text(
                vec2(pill.min.x - 24.0, ly),
                value.as_str(),
                &ink.style(ink.text, VALUE_CAP, value_paint, Align::Right),
            );
        }
        RowKind::Choice | RowKind::Carousel => {
            // Both chevrons always show (left and right adjust this row); accent when focused.
            let r = p.text(vec2(VX, ly), value.as_str(), &vstyle);
            let chev = if focused { ink.accent() } else { ink.ghost() };
            chevron(p, vec2(chevron_tip, ly), 1.0, chev);
            chevron(p, vec2(r.min.x - 30.0, ly), -1.0, chev);
        }
        RowKind::Action => {
            p.text(vec2(VX, ly), value.as_str(), &vstyle);
            chevron(p, vec2(chevron_tip, ly), 1.0, value_paint);
        }
        RowKind::Info => {
            let pal = ink.pal;
            for (i, c) in [pal.shift, pal.accent, pal.secondary, pal.primary]
                .into_iter()
                .enumerate()
            {
                let centre = vec2(VX - 22.0 - i as f32 * 60.0, ly);
                p.fill(&Shape::circle(centre, 22.0), c);
                p.stroke(&Shape::circle(centre, 22.0), ink.hair(), Stroke::rim_px(1.0));
            }
        }
    }
    if kind == RowKind::Carousel {
        carousel(p, inp, ink, top);
    } else if focused {
        if let Some(help) = model::help(row, &inp.ctl.edit) {
            p.text(
                vec2(TX, top + 96.0),
                help,
                &ink.style(ink.text, HELP_CAP, ink.muted(), Align::Left),
            );
        }
    }
}

fn chevron(p: &mut Painter<'_>, tip: Vec2, dir: f32, paint: Color) {
    let pts = [tip, tip + vec2(-14.0 * dir, -12.0), tip + vec2(-14.0 * dir, 12.0)];
    p.fill(&Shape::Polygon { points: &pts }, paint);
}

/// Live thumbnails of the neighbouring clusters (−2..=+2), the selected one larger.
fn carousel(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink, top: f32) {
    const BIG: Vec2 = vec2(330.0, 176.0);
    const SMALL: Vec2 = vec2(192.0, 112.0);
    const GAP: f32 = 16.0;
    let cy = top + 196.0;
    let mid = (CX0 + CX1) * 0.5;
    let offsets = [
        -(BIG.x * 0.5 + GAP + SMALL.x * 1.5 + GAP),
        -(BIG.x * 0.5 + GAP + SMALL.x * 0.5),
        0.0,
        BIG.x * 0.5 + GAP + SMALL.x * 0.5,
        BIG.x * 0.5 + GAP + SMALL.x * 1.5 + GAP,
    ];
    for (i, cell) in inp.thumbs.iter().enumerate() {
        let size = if i == 2 { BIG } else { SMALL };
        let centre = vec2(mid + offsets[i], cy);
        let rect = Rect::from_center_size(centre, size);
        p.fill(&Shape::round_rect(rect, 18.0), Color::BLACK.with_alpha(0.62));
        if let Some(t) = cell {
            p.save();
            p.clip(rect, 18.0);
            let world = p.current_transform();
            let s = world.scale_factor() * size.x / ENVELOPE_W;
            let c = world.apply(centre + vec2(0.0, size.y * 0.06));
            // Thumbnails skip their glass backdrops: each would copy and blur the whole frame.
            p.set_backdrops(false);
            p.set_transform(Affine::scale(s).then(&Affine::translate(c)));
            let layout = Layout::centered(c, s, inp.frame.size);
            t.cluster.draw(p, &layout);
            p.restore();
        }
        if i == 2 {
            // Selection, not focus: a plain white rim (the row ring is the only accent glow).
            p.stroke(
                &Shape::round_rect(rect, 18.0),
                ink.ink().with_alpha(0.78),
                Stroke::inner(2.0),
            );
        } else {
            p.stroke(&Shape::round_rect(rect, 18.0), ink.hair(), Stroke::rim_px(1.0));
        }
    }
    let theme = model::theme_for_view(inp.cfg, inp.ctl.edit.view);
    let all = ClusterTheme::all();
    let idx = all.iter().position(|t| *t == theme).unwrap_or(0) + 1;
    let mut l = Line::new();
    let _ = write!(l, "{idx} / {}", all.len());
    p.text(
        vec2(mid, cy + BIG.y * 0.5 + 34.0),
        l.as_str(),
        &ink.style(ink.text, 24.0, ink.muted(), Align::Center),
    );
}

fn keyboard_card(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink) {
    let kb = inp.ctl.keyboard();
    p.text(
        vec2(TX, ROWS_Y0 + 40.0),
        "Custom name",
        &ink.style(ink.text, 34.0, ink.ink(), Align::Left),
    );
    let field = Rect::from_xywh(CX0, ROWS_Y0 + 92.0, CX1 - CX0, 128.0);
    p.stroke(&Shape::round_rect(field, 26.0), ink.accent(), Stroke::inner(2.0));
    let text_style = ink.style(ink.display, 58.0, ink.ink(), Align::Left);
    let r = p.text(
        vec2(field.min.x + 36.0, field.center().y),
        kb.text.as_str(),
        &text_style,
    );
    if (inp.anim.t * 1.6).fract() < 0.55 {
        let x = if kb.text.is_empty() {
            field.min.x + 38.0
        } else {
            r.max.x + 8.0
        };
        p.fill(
            &Shape::rect(Rect::from_xywh(x, field.center().y - 34.0, 5.0, 68.0)),
            ink.acc,
        );
    }
    let mut count = Line::new();
    let _ = write!(count, "{} / {}", kb.text.chars().count(), keyboard::MAX_NAME_CHARS);
    p.text(
        vec2(field.max.x - 32.0, field.center().y),
        count.as_str(),
        &ink.style(ink.text, 24.0, ink.muted(), Align::Right),
    );

    let (key_w, key_h, gap) = (KEY_W, KEY_H, KEY_GAP);
    let x0 = CX0 + ((CX1 - CX0) - (10.0 * key_w + 9.0 * gap)) * 0.5;
    let y0 = ROWS_Y0 + KEYS_TOP;
    let mut buf = [0u8; 4];
    for row in 0..keyboard::ROWS {
        let mut col = 0u8;
        while col < keyboard::COLS {
            let (first, span) = keyboard::key_span(row, col);
            let key = keyboard::key_at(row, first, kb.caps);
            let rect = Rect::from_xywh(
                x0 + f32::from(first) * (key_w + gap),
                y0 + f32::from(row) * (key_h + gap),
                f32::from(span) * key_w + f32::from(span - 1) * gap,
                key_h,
            );
            let focused = kb.row == row && keyboard::key_span(row, kb.col).0 == first;
            let shape = Shape::round_rect(rect, KEY_R);
            if focused {
                // The same focus as a row: the key lights from within under its glass cap.
                inner_light(p, rect, KEY_R, ink);
            } else {
                p.stroke(&shape, ink.hair(), Stroke::rim_px(1.0));
            }
            let (label, style) = match key {
                Key::Char(c) => (
                    &*c.encode_utf8(&mut buf),
                    ink.style(ink.display, if focused { 46.0 } else { 40.0 }, ink.ink(), Align::Center),
                ),
                Key::Case if kb.caps => ("CAPS", ink.style(ink.text, 24.0, ink.accent(), Align::Center)),
                other => (other.label(), ink.style(ink.text, 26.0, ink.ink(), Align::Center)),
            };
            p.text(rect.center(), label, &style);
            col = first + span;
        }
    }
}

fn remap_card(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink) {
    let cap = inp.ctl.remap();
    let mid = (CX0 + CX1) * 0.5;
    p.text(
        vec2(TX, ROWS_Y0 + 44.0),
        "Hold the new OSD button",
        &ink.style(ink.text, 36.0, ink.ink(), Align::Left),
    );
    p.text(
        vec2(TX, ROWS_Y0 + 104.0),
        "Hold it for 1 second. A, B, X, Y and the D-pad are ignored.",
        &ink.style(ink.text, HINT_CAP, ink.muted(), Align::Left),
    );
    let card = Rect::from_xywh(CX0, ROWS_Y0 + 170.0, CX1 - CX0, 420.0);
    p.stroke(&Shape::round_rect(card, 32.0), ink.hair(), Stroke::rim_px(1.0));
    let cand = cap.candidate();
    let mut l = Line::new();
    if cand == 0 {
        let _ = l.write_str("Waiting for a button");
    } else {
        remap::mask_label(cand, &mut l);
    }
    p.text(
        vec2(mid, card.min.y + 140.0),
        l.as_str(),
        &ink.style(
            ink.display,
            76.0,
            if cand == 0 { ink.muted() } else { ink.ink() },
            Align::Center,
        ),
    );
    let prog = cap.progress(inp.now);
    let bar = Rect::from_xywh(mid - 400.0, card.min.y + 250.0, 800.0, 14.0);
    p.fill(&Shape::round_rect(bar, 7.0), ink.ghost());
    p.text(vec2(bar.min.x, bar.min.y - 30.0), "Hold", &ink.section());
    if prog > 0.0 {
        p.layer(LayerOptions::bloom(), |p| {
            p.fill(
                &Shape::round_rect(Rect::from_xywh(bar.min.x, bar.min.y, 800.0 * prog, 14.0), 7.0),
                ink.acc,
            );
        });
    }
    if let Some(msg) = cap.message {
        p.text(
            vec2(mid, card.min.y + 330.0),
            msg,
            &ink.style(ink.text, 26.0, ink.pal.roles.caution, Align::Center),
        );
    }
    let mut cur = Line::new();
    let _ = cur.write_str("Current: ");
    remap::mask_label(inp.cfg.osd_toggle_mask, &mut cur);
    let _ = write!(cur, "   ·   {:.0} s left", cap.remaining(inp.now).ceil());
    p.text(
        vec2(TX, card.max.y + 60.0),
        cur.as_str(),
        &ink.style(ink.text, HINT_CAP, ink.muted(), Align::Left),
    );
}

fn hint_bar(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink) {
    let text_style = ink.style(ink.text, HINT_CAP, ink.muted(), Align::Left);
    // Keycap chips only name buttons the menu really uses; the remapper has none to name.
    let hints: &[(&str, &str)] = match inp.ctl.mode() {
        Mode::Browse => &[("D-PAD", "Move · adjust"), ("LB RB", "Tabs"), ("VIEW", "Back")],
        Mode::Keyboard => &[("RB", "Type"), ("LB", "Delete"), ("MENU", "Done"), ("VIEW", "Cancel")],
        Mode::Remap => {
            p.text(
                vec2(TX, HINT_Y),
                "Hold any allowed button for 1 s · cancels after 10 s",
                &text_style,
            );
            &[]
        }
    };
    let chip_style = ink.style(ink.text, 22.0, ink.ink(), Align::Center).tracking(0.12);
    let mut x = TX;
    for (chip, text) in hints {
        let w = p.measure(chip, &chip_style).width + 32.0;
        let rect = Rect::from_xywh(x, HINT_Y - 26.0, w, 52.0);
        p.stroke(&Shape::round_rect(rect, 12.0), ink.muted(), Stroke::inner(2.0));
        p.text(rect.center(), chip, &chip_style);
        let r = p.text(vec2(rect.max.x + 14.0, HINT_Y), text, &text_style);
        x = r.max.x + 40.0;
    }
    p.text(
        vec2(TX, NOTE_Y),
        "A, B, X and Y stay with your gearbox: the menu never uses them.",
        &ink.style(ink.text, 22.0, ink.muted(), Align::Left),
    );
}

/// The OSD accent: the palette accent, else its primary, else sapphire, the first that sits
/// at least 25° of hue from the warning red and has colour (a red focus ring reads as an
/// error). Returns the fill colour and its text-safe variant.
fn osd_accent(pal: &Palette) -> (Color, Color) {
    let warn = pal.roles.warning.to_oklch().h;
    let ok = |c: Color| {
        let o = c.to_oklch();
        o.c > 0.04 && fh6_kit::color::Oklch::hue_distance(o.h, warn) >= 25.0
    };
    if ok(pal.accent) {
        (pal.accent, pal.accent_text)
    } else if ok(pal.primary) {
        (pal.primary, pal.primary_text)
    } else {
        let sapphire = Color::rgb_u32(0x38BDF8);
        (sapphire, sapphire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accent_avoids_the_warning_red() {
        let t = fh6_kit::Tokens::default();
        let red = Palette::custom(
            &t,
            Color::rgb_u32(0xFF1744),
            Color::rgb_u32(0xFF3030),
            Color::WHITE,
            None,
        );
        let (acc, _) = osd_accent(&red);
        let h = acc.to_oklch().h;
        assert!(fh6_kit::color::Oklch::hue_distance(h, red.roles.warning.to_oklch().h) >= 25.0);
        let blue = Palette::custom(
            &t,
            Color::rgb_u32(0x38BDF8),
            Color::rgb_u32(0x7DD3FC),
            Color::WHITE,
            None,
        );
        assert_eq!(osd_accent(&blue).0, blue.accent);
    }

    #[test]
    fn geometry_stacks_rows_and_sections() {
        let cfg = AppConfig::default();
        let ctl = OsdController::new();
        let fit = crate::fit::ConfigOnlyFit;
        let rows = model::rows(Tab::Fit, ctl.ctx(&cfg, &fit));
        let (geo, total) = geometry(&rows, &fit, &ctl.edit, 0);
        assert!(geo[0].section && geo[0].top > 0.0);
        for w in geo[..rows.len()].windows(2) {
            assert!(w[1].top >= w[0].top + w[0].h);
        }
        assert!(total > ROWS_Y1 - ROWS_Y0, "the Fit tab scrolls");
        assert_eq!(Frame::new((3840, 2160)).ox, 0.0);
        assert!(
            (Frame::new((5120, 1440)).ox - 3840.0).abs() < 1.0,
            "32:9 shifts the panel right"
        );
    }
}
