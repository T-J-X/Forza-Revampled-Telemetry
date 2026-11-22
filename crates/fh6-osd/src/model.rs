//! The OSD's rows: tabs, what each row edits in `AppConfig`, how it is shown and adjusted.
//!
//! Every edit writes straight into the working config (live-applied); the host wraps it in a
//! new `Arc` and queues the save. Row lists are fixed arrays and values format into a stack
//! [`Line`], so nothing here allocates per frame.
use crate::fit::{CameraApply, CockpitFit};
use crate::palette_edit::{ColourTarget, Hsv, PRESETS};
use fh6_core::{AppConfig, ClusterTheme};
use fh6_kit::View;
use serde_json::Value;
use std::fmt::{self, Write as _};

// ----- config keys kept in `extra` -------------------------------------------------------

/// Cluster height offset in 4K px (±240), read by `Layout::from_config`.
pub const KEY_HEIGHT: &str = "ClusterHeightOffsetPx";
/// Gentle limiter pulse instead of the 10 Hz strobe.
pub const KEY_REDUCED_FLASH: &str = "ReducedFlash";
/// Optional world refraction behind glass.
pub const KEY_REFRACTION: &str = "GlassRefraction";
/// Random palette seed; present = the same palette on every car.
pub const KEY_SEED: &str = "PaletteSeed";
/// Display font family for the clusters (absent = the pack's default).
pub const KEY_FONT: &str = "ClusterFont";
/// Logo zone on or off (absent = on).
pub const KEY_LOGO: &str = "ShowLogo";
/// The stats overlay (absent = off; `--stats` also turns it on).
pub const KEY_STATS: &str = "ShowStats";
/// Shift point as a fraction of the redline (absent = the tokens' value).
pub const KEY_SHIFT: &str = "ShiftPointPct";
/// Cluster for the chase cameras (absent = the cockpit cluster).
pub const KEY_CHASE: &str = "ChaseHudStyle";

/// The shift point shown when `ShiftPointPct` is absent (tokens `revStages.shiftPct`).
pub const DEFAULT_SHIFT_PCT: f64 = 0.97;

fn extra_bool(cfg: &AppConfig, key: &str, default: bool) -> bool {
    cfg.extra.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn extra_f64(cfg: &AppConfig, key: &str, default: f64) -> f64 {
    cfg.extra.get(key).and_then(Value::as_f64).unwrap_or(default)
}

fn extra_str<'a>(cfg: &'a AppConfig, key: &str) -> Option<&'a str> {
    cfg.extra
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

fn set_extra(cfg: &mut AppConfig, key: &str, value: Value) {
    cfg.extra.insert(key.to_string(), value);
}

/// `ShowLogo` (default on).
#[must_use]
pub fn show_logo(cfg: &AppConfig) -> bool {
    extra_bool(cfg, KEY_LOGO, true)
}

/// `ShowStats` (default off).
#[must_use]
pub fn show_stats(cfg: &AppConfig) -> bool {
    extra_bool(cfg, KEY_STATS, false)
}

/// `ReducedFlash` (default off).
#[must_use]
pub fn reduced_flash(cfg: &AppConfig) -> bool {
    extra_bool(cfg, KEY_REDUCED_FLASH, false)
}

/// `ClusterFont`, when set.
#[must_use]
pub fn cluster_font(cfg: &AppConfig) -> Option<&str> {
    extra_str(cfg, KEY_FONT)
}

/// `ShiftPointPct` (0.80..1.00), when set.
#[must_use]
pub fn shift_point(cfg: &AppConfig) -> Option<f32> {
    cfg.extra
        .get(KEY_SHIFT)
        .and_then(Value::as_f64)
        .map(|v| (v as f32).clamp(0.80, 1.0))
}

// ----- a stack string ----------------------------------------------------------------------

/// A fixed-capacity UTF-8 line (values, labels): formatting never allocates. Text past the
/// capacity is dropped at a character boundary.
#[derive(Clone, Copy)]
pub struct Line {
    buf: [u8; 96],
    len: u8,
}

impl Line {
    /// An empty line.
    #[must_use]
    pub const fn new() -> Self {
        Self { buf: [0; 96], len: 0 }
    }

    /// The text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..usize::from(self.len)]).unwrap_or("")
    }

    /// Empties the line.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl Default for Line {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Write for Line {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            let mut tmp = [0u8; 4];
            let enc = ch.encode_utf8(&mut tmp).as_bytes();
            let start = usize::from(self.len);
            if start + enc.len() > self.buf.len() {
                break;
            }
            self.buf[start..start + enc.len()].copy_from_slice(enc);
            self.len += enc.len() as u8;
        }
        Ok(())
    }
}

// ----- tabs and rows -----------------------------------------------------------------------

/// The five tabs (LB / RB).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    /// Cluster per view, units, shift point, limiter.
    #[default]
    Cluster,
    /// Factory, custom or random palette.
    Colour,
    /// Font, logo, custom name.
    Type,
    /// FOV, seat, wheel, cluster height and scale, dash blocker.
    Fit,
    /// OSD button, hold time, auto-pause, stats.
    Controls,
}

impl Tab {
    /// Tabs in order.
    pub const ALL: [Tab; 5] = [Tab::Cluster, Tab::Colour, Tab::Type, Tab::Fit, Tab::Controls];

    /// The tab label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Tab::Cluster => "Cluster",
            Tab::Colour => "Colour",
            Tab::Type => "Type & logo",
            Tab::Fit => "Fit & camera",
            Tab::Controls => "Controls",
        }
    }

    /// Position in [`Tab::ALL`].
    #[must_use]
    pub fn index(self) -> usize {
        self as usize
    }

    /// The tab `steps` along, wrapping.
    #[must_use]
    pub fn step(self, steps: i32) -> Tab {
        Tab::ALL[(self.index() as i32 + steps).rem_euclid(Tab::ALL.len() as i32) as usize]
    }
}

/// One row of a tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowId {
    /// Which view the Cluster rows edit (cockpit, hood, chase).
    EditView,
    /// The cluster carousel for the edited view.
    Cluster,
    /// mph / km/h.
    SpeedUnit,
    /// kW / hp.
    PowerUnit,
    /// Shift point (% of redline).
    ShiftPoint,
    /// 10 Hz strobe or gentle pulse.
    LimiterStyle,
    /// Factory, custom, random.
    ColourMode,
    /// The factory palette's four colours (info).
    FactorySwatches,
    /// Which custom colour the HSV rows edit.
    ColourRole,
    /// Hue of the edited colour.
    Hue,
    /// Saturation of the edited colour.
    Saturation,
    /// Brightness of the edited colour.
    Brightness,
    /// A curated custom palette.
    Preset,
    /// New random palette.
    Reroll,
    /// Keep the random palette on every car.
    PaletteLock,
    /// World refraction behind glass.
    GlassRefraction,
    /// Display font.
    Font,
    /// Logo zone.
    ShowLogo,
    /// Custom name (opens the keyboard).
    CustomName,
    /// Cockpit FOV.
    FovCockpit,
    /// Hood FOV.
    FovHood,
    /// Chase FOV.
    FovChase,
    /// Driver seat height.
    SeatHeight,
    /// Steering wheel offset.
    WheelOffset,
    /// Cluster height offset.
    ClusterHeight,
    /// Cluster scale.
    ClusterScale,
    /// A dash-blocker (scrim) row from the cockpit-fit backend.
    Scrim(u8),
    /// Re-align the camera ring (when the backend offers it).
    Resync,
    /// The OSD toggle button (remapper).
    OsdButton,
    /// OSD hold time.
    HoldTime,
    /// Pause the game when the OSD opens mid-race.
    AutoPause,
    /// The stats overlay.
    Stats,
}

/// A slider's range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderSpec {
    /// Smallest value.
    pub min: f32,
    /// Largest value.
    pub max: f32,
    /// One D-pad step.
    pub step: f32,
    /// Decimals shown.
    pub decimals: u8,
    /// Suffix (`"°"`, `" mm"`).
    pub unit: &'static str,
}

impl SliderSpec {
    const fn new(min: f32, max: f32, step: f32, decimals: u8, unit: &'static str) -> Self {
        Self {
            min,
            max,
            step,
            decimals,
            unit,
        }
    }

    /// `v` as a 0..1 position along the track.
    #[must_use]
    pub fn fraction(&self, v: f32) -> f32 {
        ((v - self.min) / (self.max - self.min).max(1e-6)).clamp(0.0, 1.0)
    }

    /// `v` clamped and snapped to the step grid.
    #[must_use]
    pub fn snap(&self, v: f32) -> f32 {
        let v = if v.is_finite() { v } else { self.min };
        let steps = ((v - self.min) / self.step.max(1e-6)).round();
        (self.min + steps * self.step).clamp(self.min, self.max)
    }
}

/// How a row behaves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RowKind {
    /// Left / right cycles the options (fires on press only).
    Choice,
    /// A choice shown as live thumbnails.
    Carousel,
    /// Left / right moves along a range (repeats with acceleration).
    Slider(SliderSpec),
    /// Left / right flips it.
    Toggle,
    /// Right runs it.
    Action,
    /// Shows information; not adjustable.
    Info,
}

const FOV: SliderSpec = SliderSpec::new(40.0, 110.0, 1.0, 0, "°");
const SHIFT: SliderSpec = SliderSpec::new(80.0, 100.0, 1.0, 0, "%");
const HUE: SliderSpec = SliderSpec::new(0.0, 358.0, 2.0, 0, "°");
const SAT: SliderSpec = SliderSpec::new(0.0, 100.0, 2.0, 0, "%");
const VAL: SliderSpec = SliderSpec::new(10.0, 100.0, 2.0, 0, "%");
const SEAT: SliderSpec = SliderSpec::new(-100.0, 50.0, 1.0, 0, " mm");
const WHEEL: SliderSpec = SliderSpec::new(-80.0, 20.0, 1.0, 0, " mm");
const HEIGHT: SliderSpec = SliderSpec::new(-240.0, 240.0, 4.0, 0, " px");
const SCALE: SliderSpec = SliderSpec::new(70.0, 140.0, 1.0, 0, "%");
const HOLD: SliderSpec = SliderSpec::new(200.0, 1000.0, 50.0, 0, " ms");

/// Most rows a tab can have.
pub const MAX_ROWS: usize = 24;

/// A tab's rows (fixed capacity).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RowList {
    ids: [RowId; MAX_ROWS],
    len: usize,
}

impl RowList {
    fn new() -> Self {
        Self {
            ids: [RowId::EditView; MAX_ROWS],
            len: 0,
        }
    }

    fn push(&mut self, id: RowId) {
        if self.len < MAX_ROWS {
            self.ids[self.len] = id;
            self.len += 1;
        }
    }

    /// The rows.
    #[must_use]
    pub fn as_slice(&self) -> &[RowId] {
        &self.ids[..self.len]
    }

    /// Number of rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// True when the tab has no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The row at `i`.
    #[must_use]
    pub fn get(&self, i: usize) -> Option<RowId> {
        self.as_slice().get(i).copied()
    }
}

/// The colour modes, in row order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColourMode {
    /// The car's brand palette.
    Factory,
    /// The four hex colours.
    Custom,
    /// A seeded random palette.
    Random,
}

impl ColourMode {
    /// The mode `cfg` is in.
    #[must_use]
    pub fn of(cfg: &AppConfig) -> Self {
        let m = cfg.color_mode.trim();
        if m.eq_ignore_ascii_case("custom") {
            ColourMode::Custom
        } else if m.eq_ignore_ascii_case("random") {
            ColourMode::Random
        } else {
            ColourMode::Factory
        }
    }

    fn label(self) -> &'static str {
        match self {
            ColourMode::Factory => "Factory",
            ColourMode::Custom => "Custom",
            ColourMode::Random => "Random",
        }
    }

    fn value(self) -> &'static str {
        match self {
            ColourMode::Factory => "AutoBrand",
            ColourMode::Custom => "Custom",
            ColourMode::Random => "Random",
        }
    }

    fn step(self, d: i32) -> Self {
        const ALL: [ColourMode; 3] = [ColourMode::Factory, ColourMode::Custom, ColourMode::Random];
        let i = ALL.iter().position(|m| *m == self).unwrap_or(0) as i32;
        ALL[(i + d).rem_euclid(3) as usize]
    }
}

/// Edit state that lives with the OSD, not in the config.
#[derive(Debug, Clone, PartialEq)]
pub struct EditState {
    /// The view the Cluster rows edit (and the live view shows).
    pub view: View,
    /// The custom colour the HSV rows edit.
    pub colour_target: ColourTarget,
    /// HSV of the four custom colours (kept so a grey keeps its hue while editing).
    pub hsv: [Hsv; 4],
    /// Last applied preset.
    pub preset: usize,
    /// Font families on offer (the host fills this from the loaded faces).
    pub fonts: Vec<String>,
    /// The last camera change's outcome.
    pub camera: CameraApply,
}

impl Default for EditState {
    fn default() -> Self {
        Self {
            view: View::Cockpit,
            colour_target: ColourTarget::Primary,
            hsv: [Hsv::default(); 4],
            preset: 0,
            fonts: Vec::new(),
            camera: CameraApply::NextLaunch,
        }
    }
}

fn hex_field(cfg: &AppConfig, t: ColourTarget) -> &str {
    match t {
        ColourTarget::Primary => &cfg.primary_hex,
        ColourTarget::Secondary => &cfg.secondary_hex,
        ColourTarget::Accent => &cfg.accent_hex,
        ColourTarget::Shift => &cfg.shift_flash_hex,
    }
}

fn hex_field_mut(cfg: &mut AppConfig, t: ColourTarget) -> &mut String {
    match t {
        ColourTarget::Primary => &mut cfg.primary_hex,
        ColourTarget::Secondary => &mut cfg.secondary_hex,
        ColourTarget::Accent => &mut cfg.accent_hex,
        ColourTarget::Shift => &mut cfg.shift_flash_hex,
    }
}

impl EditState {
    /// Loads the HSV cache from the config's hex colours (Sapphire for any invalid one).
    pub fn load_colours(&mut self, cfg: &AppConfig) {
        for (i, t) in ColourTarget::ALL.into_iter().enumerate() {
            self.hsv[i] = Hsv::from_hex(hex_field(cfg, t))
                .or_else(|| Hsv::from_hex(PRESETS[0].hex[i]))
                .unwrap_or_default();
        }
    }

    fn target_index(&self) -> usize {
        ColourTarget::ALL
            .iter()
            .position(|t| *t == self.colour_target)
            .unwrap_or(0)
    }
}

/// Everything a row reads.
#[derive(Clone, Copy)]
pub struct Ctx<'a> {
    /// The working config.
    pub cfg: &'a AppConfig,
    /// OSD edit state.
    pub edit: &'a EditState,
    /// The cockpit-fit backend.
    pub fit: &'a dyn CockpitFit,
}

/// The rows of `tab` for the current config.
#[must_use]
pub fn rows(tab: Tab, ctx: Ctx<'_>) -> RowList {
    let mut r = RowList::new();
    match tab {
        Tab::Cluster => {
            for id in [
                RowId::EditView,
                RowId::Cluster,
                RowId::SpeedUnit,
                RowId::PowerUnit,
                RowId::ShiftPoint,
                RowId::LimiterStyle,
            ] {
                r.push(id);
            }
        }
        Tab::Colour => {
            r.push(RowId::ColourMode);
            match ColourMode::of(ctx.cfg) {
                ColourMode::Factory => r.push(RowId::FactorySwatches),
                ColourMode::Custom => {
                    for id in [
                        RowId::ColourRole,
                        RowId::Hue,
                        RowId::Saturation,
                        RowId::Brightness,
                        RowId::Preset,
                    ] {
                        r.push(id);
                    }
                }
                ColourMode::Random => {
                    r.push(RowId::Reroll);
                    r.push(RowId::PaletteLock);
                }
            }
            r.push(RowId::GlassRefraction);
        }
        Tab::Type => {
            r.push(RowId::Font);
            r.push(RowId::ShowLogo);
            r.push(RowId::CustomName);
        }
        Tab::Fit => {
            for id in [
                RowId::FovCockpit,
                RowId::FovHood,
                RowId::FovChase,
                RowId::SeatHeight,
                RowId::WheelOffset,
                RowId::ClusterHeight,
                RowId::ClusterScale,
            ] {
                r.push(id);
            }
            for i in 0..ctx.fit.scrim_rows().len().min(MAX_ROWS - 8) {
                r.push(RowId::Scrim(i as u8));
            }
            if ctx.fit.can_resync() {
                r.push(RowId::Resync);
            }
        }
        Tab::Controls => {
            for id in [RowId::OsdButton, RowId::HoldTime, RowId::AutoPause, RowId::Stats] {
                r.push(id);
            }
        }
    }
    r
}

/// How `row` behaves.
#[must_use]
pub fn kind(row: RowId, fit: &dyn CockpitFit) -> RowKind {
    match row {
        RowId::EditView
        | RowId::SpeedUnit
        | RowId::PowerUnit
        | RowId::LimiterStyle
        | RowId::ColourMode
        | RowId::ColourRole
        | RowId::Preset
        | RowId::Font => RowKind::Choice,
        RowId::Cluster => RowKind::Carousel,
        RowId::ShiftPoint => RowKind::Slider(SHIFT),
        RowId::Hue => RowKind::Slider(HUE),
        RowId::Saturation => RowKind::Slider(SAT),
        RowId::Brightness => RowKind::Slider(VAL),
        RowId::FovCockpit | RowId::FovHood | RowId::FovChase => RowKind::Slider(FOV),
        RowId::SeatHeight => RowKind::Slider(SEAT),
        RowId::WheelOffset => RowKind::Slider(WHEEL),
        RowId::ClusterHeight => RowKind::Slider(HEIGHT),
        RowId::ClusterScale => RowKind::Slider(SCALE),
        RowId::HoldTime => RowKind::Slider(HOLD),
        RowId::Scrim(i) => match fit.scrim_rows().get(usize::from(i)) {
            Some(f) => RowKind::Slider(SliderSpec::new(f.min, f.max, f.step, f.decimals, f.unit)),
            None => RowKind::Info,
        },
        RowId::PaletteLock | RowId::GlassRefraction | RowId::ShowLogo | RowId::AutoPause | RowId::Stats => {
            RowKind::Toggle
        }
        RowId::Reroll | RowId::CustomName | RowId::Resync | RowId::OsdButton => RowKind::Action,
        RowId::FactorySwatches => RowKind::Info,
    }
}

/// The row label (sentence case).
#[must_use]
pub fn label(row: RowId, fit: &dyn CockpitFit) -> &'static str {
    match row {
        RowId::EditView => "Edit view",
        RowId::Cluster => "Cluster",
        RowId::SpeedUnit => "Speed units",
        RowId::PowerUnit => "Power units",
        RowId::ShiftPoint => "Shift point",
        RowId::LimiterStyle => "Limiter light",
        RowId::ColourMode => "Colour mode",
        RowId::FactorySwatches => "Brand colours",
        RowId::ColourRole => "Colour",
        RowId::Hue => "Hue",
        RowId::Saturation => "Saturation",
        RowId::Brightness => "Brightness",
        RowId::Preset => "Preset",
        RowId::Reroll => "Re-roll palette",
        RowId::PaletteLock => "Same palette on every car",
        RowId::GlassRefraction => "Glass refraction",
        RowId::Font => "Font",
        RowId::ShowLogo => "Logo",
        RowId::CustomName => "Custom name",
        RowId::FovCockpit => "Cockpit FOV",
        RowId::FovHood => "Hood FOV",
        RowId::FovChase => "Chase FOV",
        RowId::SeatHeight => "Seat height",
        RowId::WheelOffset => "Wheel offset",
        RowId::ClusterHeight => "Cluster height",
        RowId::ClusterScale => "Cluster scale",
        RowId::Scrim(i) => fit.scrim_rows().get(usize::from(i)).map_or("Dash blocker", |f| f.label),
        RowId::Resync => "Resync camera",
        RowId::OsdButton => "OSD button",
        RowId::HoldTime => "Hold time",
        RowId::AutoPause => "Pause game when open",
        RowId::Stats => "Stats overlay",
    }
}

/// The section heading a row starts, if any (caps, drawn above it).
#[must_use]
pub fn section(row: RowId) -> Option<&'static str> {
    Some(match row {
        RowId::EditView => "CLUSTER",
        RowId::SpeedUnit => "READOUTS",
        RowId::ColourMode => "PALETTE",
        RowId::GlassRefraction => "MATERIAL",
        RowId::Font => "TYPE",
        RowId::FovCockpit => "FIELD OF VIEW",
        RowId::SeatHeight => "DRIVING POSITION",
        RowId::ClusterHeight => "CLUSTER",
        RowId::Scrim(0) => "DASH BLOCKER",
        RowId::OsdButton => "OSD",
        RowId::Stats => "OVERLAY",
        _ => return None,
    })
}

/// True for rows that change the game's camera files.
#[must_use]
pub fn is_camera_row(row: RowId) -> bool {
    matches!(
        row,
        RowId::FovCockpit | RowId::FovHood | RowId::FovChase | RowId::SeatHeight | RowId::WheelOffset
    )
}

/// A one-line explanation shown under the focused row.
#[must_use]
pub fn help(row: RowId, edit: &EditState) -> Option<&'static str> {
    if is_camera_row(row) {
        return Some(edit.camera.notice());
    }
    Some(match row {
        RowId::EditView => "The live view follows the view you edit",
        RowId::Cluster => "Left and right browse all 29 clusters",
        RowId::ShiftPoint => "Where the shift lights reach the last stage",
        RowId::LimiterStyle => "Gentle pulse avoids the 10 Hz flash",
        RowId::FactorySwatches => "From the car's brand colours",
        RowId::Reroll => "Right rolls a new palette and keeps it",
        RowId::PaletteLock => "Off: a new palette for every car",
        RowId::GlassRefraction => "Lets the game show through glass clusters",
        RowId::CustomName => "Right opens the keyboard",
        RowId::ClusterHeight | RowId::ClusterScale => "Applies now",
        RowId::OsdButton => "Right, then hold the new button for 1 second",
        RowId::AutoPause => "Sends Esc when the OSD opens mid-race",
        _ => return None,
    })
}

// ----- clusters per view ---------------------------------------------------------------------

/// The theme `cfg` selects for `view`: hood uses `BonnetHudStyle`; chase uses `ChaseHudStyle`
/// when set, else the cockpit cluster; cockpit uses `CockpitHudStyle`, then `HudStyle`.
#[must_use]
pub fn theme_for_view(cfg: &AppConfig, view: View) -> ClusterTheme {
    let cockpit = || {
        if !cfg.cockpit_hud_style.trim().is_empty() {
            cfg.cockpit_hud_style.as_str()
        } else if !cfg.hud_style.trim().is_empty() {
            cfg.hud_style.as_str()
        } else {
            "LiquidGlass"
        }
    };
    let style = match view {
        View::Hood if !cfg.bonnet_hud_style.trim().is_empty() => cfg.bonnet_hud_style.as_str(),
        View::Hood => "Gt7HoodPro",
        View::Chase => extra_str(cfg, KEY_CHASE).unwrap_or_else(cockpit),
        View::Cockpit => cockpit(),
    };
    fh6_render::dispatch::style_to_theme(style)
}

/// Writes `theme` as the cluster for `view` (string and typed fields).
pub fn set_theme_for_view(cfg: &mut AppConfig, view: View, theme: ClusterTheme) {
    let name = theme.to_string();
    match view {
        View::Cockpit => {
            cfg.cockpit_hud_style = name;
            cfg.cockpit_hud_style_typed = theme;
        }
        View::Hood => {
            cfg.bonnet_hud_style = name;
            cfg.bonnet_hud_style_typed = theme;
        }
        View::Chase => {
            set_extra(cfg, KEY_CHASE, Value::String(name));
            cfg.chase_hud_style_typed = theme;
        }
    }
}

/// The theme `steps` along [`ClusterTheme::all`], wrapping.
#[must_use]
pub fn theme_step(theme: ClusterTheme, steps: i32) -> ClusterTheme {
    let all = ClusterTheme::all();
    let i = all.iter().position(|t| *t == theme).unwrap_or(0) as i32;
    all[(i + steps).rem_euclid(all.len() as i32) as usize]
}

/// The display name of a cluster.
#[must_use]
pub fn theme_label(theme: ClusterTheme) -> &'static str {
    match theme {
        ClusterTheme::BoschDdu => "Bosch DDU",
        ClusterTheme::DriftGlass => "Drift Glass",
        ClusterTheme::PorscheAnalog => "Porsche Analog",
        ClusterTheme::LiquidGlass => "Liquid Glass",
        ClusterTheme::WindshieldHud => "Windshield HUD",
        ClusterTheme::TeslaAdas => "Tesla ADAS",
        ClusterTheme::RetroCyber => "Retro Cyber",
        ClusterTheme::Gt7HoodPro => "GT7 Hood Pro",
        ClusterTheme::AmgMbuxSuperscreen => "AMG MBUX",
        ClusterTheme::LfaV10Nurburgring => "LFA V10",
        ClusterTheme::PaganiHorologerie => "Pagani Horologerie",
        ClusterTheme::F35HelmetHmd => "F-35 Helmet",
        ClusterTheme::BugattiTourbillon => "Bugatti Tourbillon",
        ClusterTheme::EvaBioSync => "EVA Bio-Sync",
        ClusterTheme::KoenigseggGyro => "Koenigsegg Gyro",
        ClusterTheme::ValkyrieLmp1 => "Valkyrie LMP1",
        ClusterTheme::SynthwaveOutrun => "Synthwave Outrun",
        ClusterTheme::ChronoMultiverse => "Chrono Multiverse",
        ClusterTheme::ChronoEventHorizon => "Event Horizon",
        ClusterTheme::NeuralCyberSynapse => "Neural Synapse",
        ClusterTheme::Gt7ModularTripod => "GT7 Tripod",
        ClusterTheme::ArNavWayfinder => "AR Wayfinder",
        ClusterTheme::SolarisWarpCore => "Solaris Warp Core",
        ClusterTheme::VisionSpatialGlass => "Vision Spatial",
        ClusterTheme::GlyphDotMatrix => "Glyph Dot Matrix",
        ClusterTheme::AeroPlasmaBlade => "Aero Plasma Blade",
        ClusterTheme::CymaticResonanceField => "Cymatic Field",
        ClusterTheme::KineticLiquidMetal => "Kinetic Liquid Metal",
        ClusterTheme::ShibuyaChromaticPrisma => "Shibuya Prisma",
    }
}

/// The label of a view.
#[must_use]
pub fn view_label(view: View) -> &'static str {
    match view {
        View::Cockpit => "Cockpit",
        View::Hood => "Hood",
        View::Chase => "Chase",
    }
}

fn view_step(view: View, d: i32) -> View {
    const ALL: [View; 3] = [View::Cockpit, View::Hood, View::Chase];
    let i = ALL.iter().position(|v| *v == view).unwrap_or(0) as i32;
    ALL[(i + d).rem_euclid(3) as usize]
}

// ----- values ----------------------------------------------------------------------------------

fn kmh(cfg: &AppConfig) -> bool {
    cfg.speed_unit.eq_ignore_ascii_case("KMH") || cfg.speed_unit.eq_ignore_ascii_case("KPH")
}

fn hp(cfg: &AppConfig) -> bool {
    cfg.power_unit.eq_ignore_ascii_case("HP")
}

/// The numeric value of a slider row.
#[must_use]
pub fn slider_value(row: RowId, ctx: Ctx<'_>) -> f32 {
    let cfg = ctx.cfg;
    let hsv = ctx.edit.hsv[ctx.edit.target_index()];
    match row {
        RowId::ShiftPoint => (extra_f64(cfg, KEY_SHIFT, DEFAULT_SHIFT_PCT) * 100.0) as f32,
        RowId::Hue => hsv.h,
        RowId::Saturation => hsv.s * 100.0,
        RowId::Brightness => hsv.v * 100.0,
        RowId::FovCockpit => cfg.standard_cockpit_fov,
        RowId::FovHood => cfg.bonnet_fov,
        RowId::FovChase => cfg.chase_near_fov,
        RowId::SeatHeight => cfg.driver_seat_height_m * 1000.0,
        RowId::WheelOffset => cfg.steering_wheel_offset_y_m * 1000.0,
        RowId::ClusterHeight => extra_f64(cfg, KEY_HEIGHT, 0.0) as f32,
        RowId::ClusterScale => cfg.cluster_scale * 100.0,
        RowId::HoldTime => cfg.osd_hold_ms as f32,
        RowId::Scrim(i) => ctx.fit.scrim_rows().get(usize::from(i)).map_or(0.0, |f| (f.get)(cfg)),
        _ => 0.0,
    }
}

/// Writes the value of a slider row (already snapped).
fn set_slider(row: RowId, v: f32, cfg: &mut AppConfig, edit: &mut EditState, fit: &dyn CockpitFit) {
    let ti = edit.target_index();
    match row {
        RowId::ShiftPoint => set_extra(
            cfg,
            KEY_SHIFT,
            Value::from(f64::from((v / 100.0 * 1000.0).round() / 1000.0)),
        ),
        RowId::Hue | RowId::Saturation | RowId::Brightness => {
            let hsv = &mut edit.hsv[ti];
            match row {
                RowId::Hue => hsv.h = v,
                RowId::Saturation => hsv.s = v / 100.0,
                _ => hsv.v = v / 100.0,
            }
            *hex_field_mut(cfg, edit.colour_target) = hsv.to_hex();
        }
        RowId::FovCockpit => {
            cfg.standard_cockpit_fov = v;
            cfg.wheel_less_cockpit_fov = v;
        }
        RowId::FovHood => cfg.bonnet_fov = v,
        RowId::FovChase => {
            let delta = cfg.chase_far_fov - cfg.chase_near_fov;
            cfg.chase_near_fov = v;
            cfg.chase_far_fov = (v + delta).clamp(30.0, 120.0);
        }
        RowId::SeatHeight => cfg.driver_seat_height_m = v / 1000.0,
        RowId::WheelOffset => cfg.steering_wheel_offset_y_m = v / 1000.0,
        RowId::ClusterHeight => set_extra(cfg, KEY_HEIGHT, Value::from(v.round() as i64)),
        RowId::ClusterScale => cfg.cluster_scale = (v / 100.0 * 100.0).round() / 100.0,
        RowId::HoldTime => cfg.osd_hold_ms = (v.round() as u32).clamp(200, 1000),
        RowId::Scrim(i) => {
            if let Some(f) = fit.scrim_rows().get(usize::from(i)) {
                (f.set)(cfg, v);
            }
        }
        _ => {}
    }
}

/// Formats `row`'s value into `out`.
pub fn format_value(row: RowId, ctx: Ctx<'_>, out: &mut Line) {
    out.clear();
    let cfg = ctx.cfg;
    let edit = ctx.edit;
    let _ = match kind(row, ctx.fit) {
        RowKind::Slider(spec) => {
            let v = slider_value(row, ctx);
            let signed = matches!(row, RowId::SeatHeight | RowId::WheelOffset | RowId::ClusterHeight) && v > 0.0;
            write!(
                out,
                "{}{:.*}{}",
                if signed { "+" } else { "" },
                usize::from(spec.decimals),
                v,
                spec.unit
            )
        }
        RowKind::Toggle => out.write_str(if toggle_value(row, cfg) { "On" } else { "Off" }),
        _ => match row {
            RowId::EditView => out.write_str(view_label(edit.view)),
            RowId::Cluster => out.write_str(theme_label(theme_for_view(cfg, edit.view))),
            RowId::SpeedUnit => out.write_str(if kmh(cfg) { "km/h" } else { "mph" }),
            RowId::PowerUnit => out.write_str(if hp(cfg) { "hp" } else { "kW" }),
            RowId::LimiterStyle => out.write_str(if reduced_flash(cfg) {
                "Gentle pulse"
            } else {
                "10 Hz strobe"
            }),
            RowId::ColourMode => out.write_str(ColourMode::of(cfg).label()),
            RowId::ColourRole => out.write_str(edit.colour_target.label()),
            RowId::Preset => out.write_str(PRESETS[edit.preset % PRESETS.len()].name),
            RowId::Font => out.write_str(cluster_font(cfg).unwrap_or("Pack default")),
            RowId::CustomName => {
                let n = cfg.custom_driver_name.trim();
                out.write_str(if n.is_empty() { "None" } else { n })
            }
            RowId::Reroll => out.write_str("Roll"),
            RowId::Resync => out.write_str("Run"),
            RowId::OsdButton => {
                crate::remap::mask_label(cfg.osd_toggle_mask, out);
                Ok(())
            }
            RowId::FactorySwatches => out.write_str("Auto"),
            _ => Ok(()),
        },
    };
}

/// The on/off state of a toggle row.
#[must_use]
pub fn toggle_value(row: RowId, cfg: &AppConfig) -> bool {
    match row {
        RowId::PaletteLock => cfg.extra.get(KEY_SEED).is_some_and(|v| v.as_u64().is_some()),
        RowId::GlassRefraction => extra_bool(cfg, KEY_REFRACTION, false),
        RowId::ShowLogo => show_logo(cfg),
        RowId::AutoPause => cfg.osd_auto_pause_game,
        RowId::Stats => show_stats(cfg),
        _ => false,
    }
}

// ----- adjusting --------------------------------------------------------------------------------

/// What an adjustment did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Effect {
    /// The config changed (save it, rebuild what depends on it).
    pub changed: bool,
    /// A camera value changed (the backend was told).
    pub camera: bool,
    /// Open the keyboard.
    pub keyboard: bool,
    /// Start the remapper.
    pub remap: bool,
    /// The tab's row list may have changed.
    pub rows: bool,
}

impl Effect {
    const CHANGED: Effect = Effect {
        changed: true,
        camera: false,
        keyboard: false,
        remap: false,
        rows: false,
    };
}

fn new_seed(old: u64) -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    // splitmix64
    let mut z = old ^ nanos.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) & 0x001F_FFFF_FFFF_FFFF
}

/// Adjusts `row` by `dir` (−1 left, +1 right) times `mult` steps (sliders only).
pub fn adjust(
    row: RowId,
    dir: i32,
    mult: i32,
    cfg: &mut AppConfig,
    edit: &mut EditState,
    fit: &mut dyn CockpitFit,
) -> Effect {
    if dir == 0 {
        return Effect::default();
    }
    let d = dir.signum();
    match kind(row, fit) {
        RowKind::Slider(spec) => {
            let before = slider_value(row, Ctx { cfg, edit, fit });
            let after = spec.snap(before + spec.step * (d * mult.max(1)) as f32);
            if (after - before).abs() < 1e-4 {
                return Effect::default();
            }
            set_slider(row, after, cfg, edit, fit);
            let mut e = Effect::CHANGED;
            if is_camera_row(row) {
                edit.camera = fit.camera_changed(cfg);
                e.camera = true;
            }
            e
        }
        RowKind::Toggle => {
            let on = !toggle_value(row, cfg);
            match row {
                RowId::PaletteLock => {
                    if on {
                        set_extra(cfg, KEY_SEED, Value::from(new_seed(0)));
                    } else {
                        cfg.extra.remove(KEY_SEED);
                    }
                }
                RowId::GlassRefraction => set_extra(cfg, KEY_REFRACTION, Value::Bool(on)),
                RowId::ShowLogo => set_extra(cfg, KEY_LOGO, Value::Bool(on)),
                RowId::AutoPause => cfg.osd_auto_pause_game = on,
                RowId::Stats => set_extra(cfg, KEY_STATS, Value::Bool(on)),
                _ => return Effect::default(),
            }
            Effect::CHANGED
        }
        RowKind::Action => match row {
            _ if d < 0 => Effect::default(),
            RowId::CustomName => Effect {
                keyboard: true,
                ..Effect::default()
            },
            RowId::OsdButton => Effect {
                remap: true,
                ..Effect::default()
            },
            RowId::Reroll => {
                let old = cfg.extra.get(KEY_SEED).and_then(Value::as_u64).unwrap_or(1);
                set_extra(cfg, KEY_SEED, Value::from(new_seed(old)));
                Effect::CHANGED
            }
            RowId::Resync => {
                fit.resync();
                Effect::default()
            }
            _ => Effect::default(),
        },
        RowKind::Choice | RowKind::Carousel => adjust_choice(row, d, cfg, edit),
        RowKind::Info => Effect::default(),
    }
}

fn adjust_choice(row: RowId, d: i32, cfg: &mut AppConfig, edit: &mut EditState) -> Effect {
    match row {
        RowId::EditView => {
            edit.view = view_step(edit.view, d);
            // Not a config change, but the live view follows it.
            Effect {
                rows: true,
                ..Effect::default()
            }
        }
        RowId::Cluster => {
            let next = theme_step(theme_for_view(cfg, edit.view), d);
            set_theme_for_view(cfg, edit.view, next);
            Effect::CHANGED
        }
        RowId::SpeedUnit => {
            cfg.speed_unit = if kmh(cfg) { "MPH" } else { "KMH" }.to_string();
            Effect::CHANGED
        }
        RowId::PowerUnit => {
            cfg.power_unit = if hp(cfg) { "kW" } else { "HP" }.to_string();
            Effect::CHANGED
        }
        RowId::LimiterStyle => {
            let gentle = !reduced_flash(cfg);
            set_extra(cfg, KEY_REDUCED_FLASH, Value::Bool(gentle));
            Effect::CHANGED
        }
        RowId::ColourMode => {
            let next = ColourMode::of(cfg).step(d);
            cfg.color_mode = next.value().to_string();
            if next == ColourMode::Custom {
                // Make the four hex values coherent with what the editor shows.
                for (i, t) in ColourTarget::ALL.into_iter().enumerate() {
                    *hex_field_mut(cfg, t) = edit.hsv[i].to_hex();
                }
            }
            Effect {
                changed: true,
                rows: true,
                ..Effect::default()
            }
        }
        RowId::ColourRole => {
            edit.colour_target = edit.colour_target.step(d);
            Effect::default()
        }
        RowId::Preset => {
            edit.preset = (edit.preset as i32 + d).rem_euclid(PRESETS.len() as i32) as usize;
            let preset = PRESETS[edit.preset];
            for (i, t) in ColourTarget::ALL.into_iter().enumerate() {
                *hex_field_mut(cfg, t) = preset.hex[i].to_string();
                if let Some(hsv) = Hsv::from_hex(preset.hex[i]) {
                    edit.hsv[i] = hsv;
                }
            }
            Effect::CHANGED
        }
        RowId::Font => {
            let n = edit.fonts.len() as i32 + 1;
            let cur = cluster_font(cfg)
                .and_then(|f| edit.fonts.iter().position(|x| x.eq_ignore_ascii_case(f)))
                .map_or(0, |i| i as i32 + 1);
            let next = (cur + d).rem_euclid(n);
            if next == 0 {
                cfg.extra.remove(KEY_FONT);
            } else if let Some(name) = edit.fonts.get((next - 1) as usize) {
                set_extra(cfg, KEY_FONT, Value::String(name.clone()));
            }
            Effect::CHANGED
        }
        _ => Effect::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fit::ConfigOnlyFit;

    fn ctx<'a>(cfg: &'a AppConfig, edit: &'a EditState) -> Ctx<'a> {
        Ctx {
            cfg,
            edit,
            fit: &ConfigOnlyFit,
        }
    }

    #[test]
    fn every_tab_has_rows_and_labels() {
        let cfg = AppConfig::default();
        let edit = EditState::default();
        for tab in Tab::ALL {
            let r = rows(tab, ctx(&cfg, &edit));
            assert!(!r.is_empty(), "{tab:?}");
            for row in r.as_slice() {
                assert!(!label(*row, &ConfigOnlyFit).is_empty());
                let mut line = Line::new();
                format_value(*row, ctx(&cfg, &edit), &mut line);
            }
        }
        assert_eq!(Tab::Cluster.step(-1), Tab::Controls);
    }

    #[test]
    fn sliders_clamp_and_step() {
        let mut cfg = AppConfig::default();
        let mut edit = EditState::default();
        let mut fit = ConfigOnlyFit;
        cfg.standard_cockpit_fov = 109.0;
        let e = adjust(RowId::FovCockpit, 1, 10, &mut cfg, &mut edit, &mut fit);
        assert!(e.changed && e.camera);
        assert_eq!(cfg.standard_cockpit_fov, 110.0);
        assert_eq!(cfg.wheel_less_cockpit_fov, 110.0);
        let e = adjust(RowId::FovCockpit, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert!(!e.changed, "already at max");
        adjust(RowId::ClusterHeight, -1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.extra.get(KEY_HEIGHT).and_then(Value::as_i64), Some(-4));
        let mut line = Line::new();
        format_value(RowId::ClusterHeight, ctx(&cfg, &edit), &mut line);
        assert_eq!(line.as_str(), "-4 px");
    }

    #[test]
    fn cluster_carousel_writes_the_edited_view() {
        let mut cfg = AppConfig::default();
        let mut edit = EditState::default();
        let mut fit = ConfigOnlyFit;
        cfg.cockpit_hud_style = "LiquidGlass".into();
        adjust(RowId::Cluster, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.cockpit_hud_style, "WindshieldHud");
        assert_eq!(cfg.cockpit_hud_style_typed, ClusterTheme::WindshieldHud);
        edit.view = View::Chase;
        assert_eq!(
            theme_for_view(&cfg, View::Chase),
            ClusterTheme::WindshieldHud,
            "chase follows cockpit"
        );
        adjust(RowId::Cluster, -1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(theme_for_view(&cfg, View::Chase), ClusterTheme::LiquidGlass);
        assert_eq!(theme_for_view(&cfg, View::Cockpit), ClusterTheme::WindshieldHud);
        // Every theme round-trips through its config name.
        for t in ClusterTheme::all() {
            set_theme_for_view(&mut cfg, View::Hood, *t);
            assert_eq!(theme_for_view(&cfg, View::Hood), *t);
        }
    }

    #[test]
    fn colour_modes_and_custom_edits() {
        let mut cfg = AppConfig::default();
        let mut edit = EditState::default();
        let mut fit = ConfigOnlyFit;
        edit.load_colours(&cfg);
        cfg.color_mode = "AutoBrand".into();
        adjust(RowId::ColourMode, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.color_mode, "Custom");
        assert!(rows(Tab::Colour, ctx(&cfg, &edit)).as_slice().contains(&RowId::Hue));
        adjust(RowId::Preset, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.primary_hex, PRESETS[1].hex[0]);
        adjust(RowId::Hue, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_ne!(cfg.primary_hex, PRESETS[1].hex[0]);
        adjust(RowId::ColourMode, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.color_mode, "Random");
        adjust(RowId::Reroll, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert!(toggle_value(RowId::PaletteLock, &cfg));
        adjust(RowId::PaletteLock, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert!(cfg.extra.get(KEY_SEED).is_none());
    }

    #[test]
    fn toggles_and_actions() {
        let mut cfg = AppConfig::default();
        let mut edit = EditState::default();
        let mut fit = ConfigOnlyFit;
        assert!(show_logo(&cfg));
        adjust(RowId::ShowLogo, -1, 1, &mut cfg, &mut edit, &mut fit);
        assert!(!show_logo(&cfg));
        assert!(adjust(RowId::CustomName, 1, 1, &mut cfg, &mut edit, &mut fit).keyboard);
        assert!(!adjust(RowId::CustomName, -1, 1, &mut cfg, &mut edit, &mut fit).keyboard);
        assert!(adjust(RowId::OsdButton, 1, 1, &mut cfg, &mut edit, &mut fit).remap);
        edit.fonts = vec!["Saira Condensed".into(), "Michroma".into()];
        adjust(RowId::Font, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cluster_font(&cfg), Some("Saira Condensed"));
        adjust(RowId::Font, -1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cluster_font(&cfg), None);
    }

    #[test]
    fn line_truncates_at_a_char_boundary() {
        let mut line = Line::new();
        for _ in 0..40 {
            let _ = line.write_str("死");
        }
        assert_eq!(line.as_str().chars().count(), 32);
    }
}
