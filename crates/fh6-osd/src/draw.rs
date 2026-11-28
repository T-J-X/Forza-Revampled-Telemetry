//! Draws the OSD with the render kit: the dimmed backdrop, the live-zone header, the focus echo
//! (lifted over the rendered driving position on the camera rows, see [`crate::drive`]), and
//! the Liquid Glass control panel (tabs, rows, carousel thumbnails, keyboard and remap cards,
//! hint bar). Authored in 4K units (3840×2160, scaled by `height / 2160`); the panel stays
//! right-aligned on wider screens. Allocation-free.
use crate::anim::{AnimTargets, OsdAnim};
use crate::drive::{self, DriveImages};
use crate::fit::CockpitFit;
use crate::keyboard::{self, Key};
use crate::model::{self, EditState, Line, RowId, RowKind, RowList, Tab, MAX_ROWS};
use crate::nav::{Mode, OsdController};
use crate::remap;
use fh6_core::{AppConfig, ClusterTheme, ScrimFit};
use fh6_kit::optics::absorption_coefficient;
use fh6_kit::scrim::{self, Wash};
use fh6_kit::{
    vec2, Affine, Align, Cluster, Color, Environment, FontId, FontRole, Fonts, Glow, Gradient, ImageId, LayerOptions,
    Layout, OpticalGlass, Painter, Palette, Piece, Pool, Rect, Refraction, Setup, Shape, Stroke, TextStyle, VAlign,
    Vec2, View,
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
/// How far the focus echo rises on the camera rows (4K units), clearing the way for the
/// driving position under it.
const ECHO_LIFT: f32 = 300.0;
/// The driving position's top edge (4K units); it is centred on the live zone.
const DRIVE_TOP: f32 = 850.0;
/// The black pool under the driving position (radii, 4K units): the renders are black
/// upholstery and carbon, and read as a product shot on black.
const DRIVE_POOL: Vec2 = vec2(620.0, 390.0);
/// The seat height the game's camera files ship with (mm, `AppConfig::default`), where the
/// default eye's ring sits.
const DEFAULT_SEAT_MM: f32 = -45.0;
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
    /// The driving position's rendered layers (see [`drive::load`]); `None` draws its line
    /// drawing.
    pub drive: Option<DriveImages>,
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
    let half = widths[ti] * 0.5 + TAB_FILL_PAD;
    let key = key_rect(ctl);
    let camera = ctl.mode() == Mode::Browse && rows.get(fi).is_some_and(model::is_camera_row);
    AnimTargets {
        glide: frame.live_centre_x(),
        focus_top: g.top,
        focus_bot: g.top + g.h,
        tab_left: centres[ti] - half,
        tab_right: centres[ti] + half,
        key: [key.min.x, key.min.y, key.max.x, key.max.y],
        scroll,
        echo_lift: if camera { 1.0 } else { 0.0 },
        calm: cfg.reduced_flash,
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

/// The soft black pools under the OSD, at most [`Pools::MAX`]: behind the live cluster, the
/// focus echo, the driving position and the control column. They keep the menu dark over a
/// bright game frame and are invisible on OLED black.
///
/// The host passes them to `Kit::set_pools` every frame, as it passes [`underlay`] to
/// `Kit::set_underlay`. Painted as shapes they cost about 0.4 ms of GPU time in a 4K frame (a
/// soft shape blends into two HDR targets for every pixel it covers); the kit writes them in a
/// pass that blends nothing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pools {
    list: [Pool; Pools::MAX],
    len: usize,
}

impl Pools {
    /// Most pools one OSD frame has.
    pub const MAX: usize = 4;

    /// No pools: what a host sets while the OSD is closed.
    pub const NONE: Pools = Pools {
        list: [Pool::Ellipse {
            center: Vec2::ZERO,
            radii: Vec2::ZERO,
            alpha: 0.0,
        }; Pools::MAX],
        len: 0,
    };

    /// The pools, in the order they lie over each other.
    #[must_use]
    pub fn as_slice(&self) -> &[Pool] {
        self.list.get(..self.len).unwrap_or(&[])
    }

    fn push(&mut self, pool: Pool) {
        if let Some(slot) = self.list.get_mut(self.len) {
            *slot = pool;
            self.len += 1;
        }
    }
}

/// This frame's pools, in target pixels. `live_pool` is false while the real dash blocker
/// stands under the live cluster ([`live_backing`]): the blocker is the backing then.
#[must_use]
pub fn pools(anim: &OsdAnim, frame: Frame, layout: &Layout, live_pool: bool) -> Pools {
    let open = anim.open.progress().clamp(0.0, 1.0);
    // The live-zone overlays and the panel fade in on this ease ([`ui`]), and their pools with
    // them.
    let eased = 1.0 - (1.0 - open) * (1.0 - open);
    let k = frame.k;
    let mut pools = Pools::NONE;
    if live_pool {
        // Behind the live cluster, so the game's own lit gauges do not compete with it.
        let s = layout.scale;
        pools.push(Pool::Ellipse {
            center: layout.to_target(vec2(0.0, -10.0)),
            radii: vec2(820.0 * s, 400.0 * s),
            alpha: LIVE_SCRIM * open,
        });
    }
    // Behind the echo, so 110 px type holds over a bright sky. It rises with the echo, and a
    // second one comes up under the driving position.
    let cx = frame.live_centre_x();
    let lift = anim.echo_lift.value();
    pools.push(Pool::Ellipse {
        center: vec2(cx, 880.0 - ECHO_LIFT * lift) * k,
        radii: vec2(700.0, 330.0) * k,
        alpha: ECHO_SCRIM * eased,
    });
    let shown = lift.clamp(0.0, 1.0);
    if shown > 0.001 {
        pools.push(Pool::Ellipse {
            center: vec2(cx, DRIVE_TOP + drive::VIEW.y * 0.5) * k,
            radii: DRIVE_POOL * k,
            alpha: ECHO_SCRIM * shown * eased,
        });
    }
    // Behind the control column, which has no panel. It slides in with the column.
    let slide = vec2(frame.ox + (1.0 - eased) * PANEL_SLIDE, 0.0);
    pools.push(Pool::RoundRect {
        rect: Rect::from_points(
            (vec2(PANEL.min.x + 60.0, PANEL.min.y + 40.0) + slide) * k,
            (vec2(PANEL.max.x - 60.0, PANEL.max.y - 40.0) + slide) * k,
        ),
        radius: PANEL_R * k,
        sigma: POOL_SIGMA * k,
        alpha: POOL_ALPHA * eased,
    });
    pools
}

/// What the host draws under the live cluster: the real dash blocker (Phase 8 dome and wash, in
/// the layout's local units) while a blocker row has focus, so its shape and fit can be judged
/// against the cluster. Otherwise nothing: the soft pool that keeps the cluster legible is one
/// of [`pools`].
pub fn live_backing(p: &mut Painter<'_>, blocker: Option<(&ScrimFit, Wash)>) {
    if let Some((fit, wash)) = blocker {
        scrim::draw_scrim(p, fit);
        scrim::draw_wash(p, fit, wash);
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

/// Draws the live-zone overlays and the panel. Call after the live cluster. The black pools
/// under them are not drawn here: see [`pools`].
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
    p.translate(vec2(inp.frame.ox + (1.0 - eased) * PANEL_SLIDE, 0.0));
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
    let row = inp.ctl.focused_row(inp.cfg, inp.fit);
    // The echo rises on the camera rows and the driving position fades in under it; both ride
    // the same spring, so the view also fades out after the focus leaves those rows.
    let lift = inp.anim.echo_lift.value();
    driving_position(p, inp, ink, row, cx, lift.clamp(0.0, 1.0));
    let Some(row) = row else {
        return;
    };
    let up = ECHO_LIFT * lift;
    let ctx = inp.ctl.ctx(inp.cfg, inp.fit);
    p.text(
        vec2(cx, 740.0 - up),
        model::label(row, inp.fit),
        &centered(ink.section()),
    );
    let mut value = Line::new();
    model::format_value(row, ctx, &mut value);
    p.text(
        vec2(cx, 860.0 - up),
        value.as_str(),
        &ink.style(ink.display, 110.0, ink.ink(), Align::Center),
    );
    if let RowKind::Slider(spec) = model::kind(row, inp.fit) {
        let f = spec.fraction(model::slider_value(row, ctx));
        let track = Rect::from_xywh(cx - 360.0, 966.0 - up, 720.0, 8.0);
        p.fill(&Shape::round_rect(track, 4.0), ink.ghost());
        p.fill(
            &Shape::round_rect(Rect::from_xywh(track.min.x, track.min.y, 720.0 * f, 8.0), 4.0),
            ink.acc,
        );
    }
    if let Some(help) = model::help(row, &inp.ctl.edit) {
        p.text(
            vec2(cx, 1030.0 - up),
            help,
            &ink.style(ink.text, HINT_CAP, ink.muted(), Align::Center),
        );
    }
    if row == RowId::ClusterHeight {
        default_height_line(p, inp, ink);
    }
}

/// The driving position under the lifted echo (see [`crate::drive`]) at opacity `shown`: the
/// seat and the eye at the seat height, the wheel at its offset, and the cockpit FOV's fan. The
/// part the focused row moves is the one in the accent; on the hood and chase FOV rows nothing
/// in the view is theirs, so it stays in secondary ink as context.
fn driving_position(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink, row: Option<RowId>, cx: f32, shown: f32) {
    if shown <= 0.001 {
        return;
    }
    let cfg = inp.cfg;
    let focus = match row {
        Some(RowId::FovCockpit) => drive::Focus::Fov,
        Some(RowId::SeatHeight) => drive::Focus::Seat,
        Some(RowId::WheelOffset) => drive::Focus::Wheel,
        _ => drive::Focus::None,
    };
    let state = drive::DriveState {
        seat_mm: cfg.driver_seat_height_m * 1000.0,
        wheel_mm: cfg.steering_wheel_offset_y_m * 1000.0,
        default_seat_mm: DEFAULT_SEAT_MM,
        fov_deg: cfg.wheel_less_cockpit_fov,
        focus,
        energy: inp.anim.energy.value(),
    };
    let look = drive::Look {
        accent: ink.acc,
        muted: ink.muted(),
        ghost: ink.ghost(),
        text: ink.text,
    };
    let origin = vec2(cx - drive::VIEW.x * 0.5, DRIVE_TOP);
    drive::draw(p, origin, inp.drive, &state, &look, shown);
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
/// How far right of its place the panel starts while the OSD opens (4K units).
const PANEL_SLIDE: f32 = 64.0;
/// Radiance white in `studio_env.png` (`assets/clusters/liquid_glass/ASSETS.md`), how far the
/// studio leans to the car's primary colour, and the key light's share, all as the Liquid
/// Glass cluster sets them, so the two read as one material.
const ENV_WHITE: f32 = 16.0;
const AMBIENT_TINT: f32 = 0.4;
const KEY_SHARE: f32 = 0.5;
/// How far the key light leans (radians) when focus is at the top or bottom of the column.
const PARALLAX_TILT: f32 = 0.07;

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
fn osd_glass(p: &Painter<'_>, env: Option<ImageId>, materialize: f32, lean_y: f32) -> OpticalGlass {
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
    // Parallax: the key light leans with the focus's height in the column, so its highlights
    // travel round the pieces as focus moves. The environment turns but does not lean, so its
    // dark cone stays on the view axis and no studio light reaches the text (as in the cluster).
    let azimuth = fh6_kit::LIGHT_REST_AZIMUTH;
    let elevation = op.key_elevation;
    let mut level = glass;
    level.aim(azimuth, elevation, Vec2::ZERO);
    glass.aim(azimuth, elevation, vec2(PARALLAX_TILT * lean_y.clamp(-1.0, 1.0), 0.0));
    glass.env_rotation = level.env_rotation;
    glass.materialize = materialize;
    glass
}

/// The inner light's strength at mid height and at the floor, and how much the energise light
/// adds on top (a share of it) while a value changes.
const INNER_MID: f32 = 0.05;
const INNER_FLOOR: f32 = 0.20;
const ENERGY_GAIN: f32 = 0.5;

/// Corner radius of a keyboard key (its glass cap when focused).
const KEY_R: f32 = 18.0;

/// The light a focused piece gives from within: a glow rising from its floor, under the glass,
/// so the rounded edge bends a band of it into the rim (Liquid Glass "illuminates from within").
/// `energy` (0..1) raises it while a value is being changed.
fn inner_light(p: &mut Painter<'_>, rect: Rect, radius: f32, ink: &Ink, energy: f32) {
    let lift = 1.0 + ENERGY_GAIN * energy.clamp(0.0, 1.0);
    p.fill(
        &Shape::round_rect(rect, radius),
        Gradient::linear(vec2(0.0, rect.min.y), vec2(0.0, rect.max.y))
            .stop(0.0, ink.acc.with_alpha(0.0))
            .stop(0.55, ink.acc.with_alpha(INNER_MID * lift))
            .stop(1.0, ink.acc.with_alpha(INNER_FLOOR * lift)),
    );
}

/// The on-screen keyboard's focused key, in panel units (its glass cap's target).
fn key_rect(ctl: &OsdController) -> Rect {
    let kb = ctl.keyboard();
    let (first, span) = keyboard::key_span(kb.row, kb.col);
    let x0 = CX0 + ((CX1 - CX0) - (10.0 * KEY_W + 9.0 * KEY_GAP)) * 0.5;
    let y0 = ROWS_Y0 + KEYS_TOP;
    Rect::from_xywh(
        x0 + f32::from(first) * (KEY_W + KEY_GAP),
        y0 + f32::from(kb.row) * (KEY_H + KEY_GAP),
        f32::from(span) * KEY_W + f32::from(span.saturating_sub(1)) * KEY_GAP,
        KEY_H,
    )
}

/// The keyboard's glass key cap where its springs have it (never thinner than 40 px).
fn key_cap(inp: &DrawInput<'_>) -> Rect {
    let [l, t, r, b] = inp.anim.key.map(|s| s.value());
    let (x0, x1) = (l.min(r), l.max(r).max(l.min(r) + 40.0));
    let (y0, y1) = (t.min(b), t.max(b).max(t.min(b) + 40.0));
    Rect::from_points(vec2(x0, y0), vec2(x1, y1))
}

/// Keyboard key size, gap and the keys' top below `ROWS_Y0`.
const KEY_W: f32 = 104.0;
const KEY_H: f32 = 100.0;
const KEY_GAP: f32 = 12.0;
const KEYS_TOP: f32 = 290.0;

/// The focused row's platter, in panel units (content scrolled into place).
fn focus_rect(inp: &DrawInput<'_>) -> Rect {
    let y = ROWS_Y0 - inp.anim.scroll.value();
    let top = inp.anim.focus_top.value();
    let bottom = inp.anim.focus_bot.value().max(top + 40.0);
    Rect::from_points(vec2(GLASS_X0, y + top), vec2(GLASS_X1, y + bottom))
}

fn panel(p: &mut Painter<'_>, inp: &DrawInput<'_>, ink: &Ink) {
    // No panel: a feathered black pool ([`pools`]) keeps the column dark over a bright frame,
    // and on OLED black it is invisible. The rows are light on black; the glass is the
    // navigation: the tab bar, the hint bar and the focus platter.

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
    let left = inp.anim.tab_left.value();
    let right = inp.anim.tab_right.value().max(left + TAB_FILL_H);
    let fill = Rect::from_points(
        vec2(left, TABS_Y - TAB_FILL_H * 0.5),
        vec2(right, TABS_Y + TAB_FILL_H * 0.5),
    );
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
    let focus_y = match inp.ctl.mode() {
        Mode::Keyboard => key_cap(inp).center().y,
        _ => focus_rect(inp).center().y,
    };
    let lean = (focus_y - (ROWS_Y0 + ROWS_Y1) * 0.5) / ((ROWS_Y1 - ROWS_Y0) * 0.5);
    let glass = osd_glass(p, inp.env, grown, lean);
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
        Mode::Keyboard => Some(Piece::round_rect(key_cap(inp), KEY_R).section(KEY_RISE, KEY_BEVEL, GLASS_BASE)),
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

    inner_light(p, focus_rect(inp), PLATTER_R, ink, inp.anim.energy.value());

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

    // The focused key lights from within under its glass cap, which glides between keys.
    inner_light(p, key_cap(inp), KEY_R, ink, inp.anim.energy.value());
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
            if !focused {
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

    #[test]
    fn the_echo_lifts_over_the_driving_position_on_the_camera_rows_only() {
        let cfg = AppConfig::default();
        assert!(
            (DEFAULT_SEAT_MM - cfg.driver_seat_height_m * 1000.0).abs() < 1e-3,
            "the default eye's ring follows AppConfig"
        );
        let fit = crate::fit::ConfigOnlyFit;
        let fonts = Fonts::new().expect("embedded font loads");
        let frame = Frame::new((3840, 2160));
        let mut ctl = OsdController::new();
        ctl.open(&cfg, View::Cockpit);
        for (row, lifted) in [
            (RowId::FovCockpit, true),
            (RowId::FovHood, true),
            (RowId::FovChase, true),
            (RowId::SeatHeight, true),
            (RowId::WheelOffset, true),
            (RowId::ClusterHeight, false),
        ] {
            assert!(ctl.focus_row(Tab::Fit, row, &cfg, &fit), "{row:?} is on the Fit tab");
            let lift = targets(&ctl, &cfg, &fit, &fonts, frame).echo_lift;
            assert_eq!(lift, if lifted { 1.0 } else { 0.0 }, "{row:?}");
        }
        // The lifted echo's help line (cap middle, plus half a cap and a descender) stays clear
        // of the view, and the view clears the live cluster's envelope at the default anchor.
        let help_bottom = 1030.0 - ECHO_LIFT + HINT_CAP;
        assert!(
            help_bottom < DRIVE_TOP - 80.0,
            "drive.rs keeps the fan's tip within 80 px of the top"
        );
        // The seat at its lowest (-100 mm) drops about 0.55 px per mm below the view.
        let view_bottom = DRIVE_TOP + drive::VIEW.y + 100.0 * 0.55;
        assert!(view_bottom < cfg.cockpit_anchor_y as f32 - 320.0);
    }

    /// The motion of an OSD that is fully open, with the echo lifted by `echo_lift` (0 or 1).
    fn open_anim(echo_lift: f32) -> OsdAnim {
        let mut anim = OsdAnim::new(&fh6_kit::Tokens::default());
        anim.reset(1920.0);
        anim.settle(&AnimTargets {
            echo_lift,
            ..AnimTargets::default()
        });
        anim
    }

    #[test]
    fn the_pools_lie_under_the_cluster_the_echo_the_driving_position_and_the_column() {
        let frame = Frame::new((3840, 2160));
        let layout = Layout::centered(vec2(1150.0, 1835.0), 1.0, (3840, 2160));
        let cx = frame.live_centre_x();
        let alpha = |pools: &Pools, at: Vec2| Pool::combined_alpha(pools.as_slice(), at);

        // A plain row: the cluster's pool, the echo's and the column's.
        let rest = pools(&open_anim(0.0), frame, &layout, true);
        assert_eq!(rest.as_slice().len(), 3);
        assert!(
            (alpha(&rest, vec2(1150.0, 1825.0)) - LIVE_SCRIM).abs() < 0.01,
            "behind the cluster"
        );
        assert!(
            (alpha(&rest, vec2(cx, 880.0)) - ECHO_SCRIM).abs() < 0.01,
            "behind the echo"
        );
        // The column's pool is at its core down the middle of the rows. The labels start and the
        // values end inside its feather, where more than half of it is left.
        let (mid_x, mid_y) = ((TX + VX) * 0.5, (ROWS_Y0 + ROWS_Y1) * 0.5);
        assert!(
            (alpha(&rest, vec2(mid_x, mid_y)) - POOL_ALPHA).abs() < 0.01,
            "behind the rows"
        );
        assert!(
            alpha(&rest, vec2(TX, mid_y)) > POOL_ALPHA * 0.5,
            "behind the labels' edge"
        );
        assert!(
            alpha(&rest, vec2(VX, mid_y)) > POOL_ALPHA * 0.5,
            "behind the values' edge"
        );
        assert!(
            alpha(&rest, vec2(mid_x, HINT_Y)) > POOL_ALPHA * 0.85,
            "behind the hint bar"
        );
        assert_eq!(alpha(&rest, vec2(160.0, 196.0)), 0.0, "the header has no pool");

        // A camera row: the echo's pool rises with it and one comes up under the driving position.
        let lifted = pools(&open_anim(1.0), frame, &layout, true);
        assert_eq!(lifted.as_slice().len(), Pools::MAX);
        assert!((alpha(&lifted, vec2(cx, 880.0 - ECHO_LIFT)) - ECHO_SCRIM).abs() < 0.01);
        let view_centre = vec2(cx, DRIVE_TOP + drive::VIEW.y * 0.5);
        assert!(
            alpha(&lifted, view_centre) >= ECHO_SCRIM - 0.01,
            "under the driving position"
        );

        // While the real dash blocker stands under the cluster, the cluster has no pool.
        let blocked = pools(&open_anim(0.0), frame, &layout, false);
        assert_eq!(blocked.as_slice().len(), 2);
        assert!(alpha(&blocked, vec2(1150.0, 1825.0)) < 0.01);

        // On the first frame of the open the pools show nothing yet, and `NONE` has none at all.
        let mut opening = OsdAnim::new(&fh6_kit::Tokens::default());
        opening.reset(1920.0);
        let first = pools(&opening, frame, &layout, true);
        assert!(first.as_slice().iter().all(|pool| !pool.is_visible()));
        assert!(Pools::NONE.as_slice().is_empty());
    }

    #[test]
    fn the_pools_follow_the_frame_like_the_panel_they_lie_under() {
        let anim = open_anim(0.0);
        let column = |frame: Frame| {
            let layout = Layout::centered(vec2(0.0, 0.0), frame.k, frame.size);
            match pools(&anim, frame, &layout, false).as_slice().last().copied() {
                Some(Pool::RoundRect { rect, sigma, .. }) => (rect, sigma),
                other => panic!("the column's pool is the last one: {other:?}"),
            }
        };
        let (at_4k, sigma_4k) = column(Frame::new((3840, 2160)));
        assert_eq!(at_4k.min, vec2(PANEL.min.x + 60.0, PANEL.min.y + 40.0));
        assert_eq!(at_4k.max, vec2(PANEL.max.x - 60.0, PANEL.max.y - 40.0));
        assert_eq!(sigma_4k, POOL_SIGMA);
        // Half the size at 1080p, sigma included.
        let (at_1080, sigma_1080) = column(Frame::new((1920, 1080)));
        assert!((at_1080.min - at_4k.min * 0.5).length() < 1e-3);
        assert!((at_1080.max - at_4k.max * 0.5).length() < 1e-3);
        assert!((sigma_1080 - POOL_SIGMA * 0.5).abs() < 1e-4);
        // On a 32:9 screen the panel sits right, and its pool with it.
        let wide = Frame::new((5120, 1440));
        let (at_wide, _) = column(wide);
        assert!((at_wide.min.x - (PANEL.min.x + 60.0 + wide.ox) * wide.k).abs() < 1e-2);
    }
}
