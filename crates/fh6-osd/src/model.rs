//! The OSD's rows: tabs, what each row edits in `AppConfig`, how it is shown and adjusted.
//!
//! Every edit writes straight into the working config (live-applied); the host wraps it in a
//! new `Arc` and queues the save. Row lists are fixed arrays and values format into a stack
//! [`Line`], so nothing here allocates per frame.
use crate::fit::{CameraApply, CockpitFit};
use crate::palette_edit::{ColourTarget, Hsv, PRESETS};
use fh6_core::{AppConfig, ClusterTheme, ColorMode, PowerUnit, SpeedUnit};
use fh6_kit::View;
use std::fmt::{self, Write as _};

/// The shift point shown when `ShiftPointPct` is absent (tokens `revStages.shiftPct`).
pub const DEFAULT_SHIFT_PCT: f32 = 0.97;

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
    /// Write the camera values to the game's camera files (`CameraPatchAuto`).
    CameraPatch,
    /// Cluster height offset for the edited view (`CockpitHeightOffsetPx` and siblings).
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

/// The colour modes the OSD offers, in row order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColourMode {
    /// The car's brand palette ([`ColorMode::AutoBrand`]; a config in [`ColorMode::Brand`] shows
    /// as Factory too).
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
        match cfg.color_mode {
            ColorMode::Custom => ColourMode::Custom,
            ColorMode::Random => ColourMode::Random,
            ColorMode::AutoBrand | ColorMode::Brand => ColourMode::Factory,
        }
    }

    fn label(self) -> &'static str {
        match self {
            ColourMode::Factory => "Factory",
            ColourMode::Custom => "Custom",
            ColourMode::Random => "Random",
        }
    }

    fn value(self) -> ColorMode {
        match self {
            ColourMode::Factory => ColorMode::AutoBrand,
            ColourMode::Custom => ColorMode::Custom,
            ColourMode::Random => ColorMode::Random,
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
    /// The current car's profile (`car_profiles.json`) overrides the focused row: the live view
    /// shows the profile's value, the row the global one (the host sets this every frame).
    pub profile_override: bool,
    /// The anchor height the live view really uses for the edited view (a car profile may move
    /// it); `None` = the config's own anchor.
    pub live_anchor_y: Option<f32>,
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
            profile_override: false,
            live_anchor_y: None,
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
                RowId::CameraPatch,
                RowId::ClusterHeight,
                RowId::ClusterScale,
            ] {
                r.push(id);
            }
            // Keep one slot for the resync action.
            let room = MAX_ROWS.saturating_sub(r.len() + 1);
            for i in 0..ctx.fit.scrim_rows().len().min(room) {
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
        RowId::PaletteLock
        | RowId::GlassRefraction
        | RowId::ShowLogo
        | RowId::CameraPatch
        | RowId::AutoPause
        | RowId::Stats => RowKind::Toggle,
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
        RowId::CameraPatch => "Camera patch",
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

/// Shown under a row whose value the current car's profile overrides.
pub const PROFILE_OVERRIDE_NOTE: &str = "This car's profile overrides this setting";

/// A one-line explanation shown under the focused row.
#[must_use]
pub fn help(row: RowId, edit: &EditState) -> Option<&'static str> {
    if edit.profile_override {
        return Some(PROFILE_OVERRIDE_NOTE);
    }
    if is_camera_row(row) {
        return Some(edit.camera.notice());
    }
    Some(match row {
        RowId::EditView => "The live view follows the view you edit",
        RowId::Cluster => "Left and right browse every cluster",
        RowId::ShiftPoint => "Where the shift lights reach the last stage",
        RowId::LimiterStyle => "Gentle pulse avoids the 10 Hz flash",
        RowId::FactorySwatches => "From the car's brand colours",
        RowId::Reroll => "Right rolls a new palette and keeps it",
        RowId::PaletteLock => "Off: a new palette for every car",
        RowId::GlassRefraction => "Lets the game show through glass clusters",
        RowId::CustomName => "Right opens the keyboard",
        RowId::CameraPatch => "Writes FOV and seat to the game's camera files",
        RowId::ClusterHeight => "Applies now, to the view you edit",
        RowId::ClusterScale => "Applies now",
        RowId::OsdButton => "Right, then hold the new button for 1 second",
        RowId::AutoPause => "Sends Esc when the OSD opens mid-race",
        _ => return None,
    })
}

// ----- clusters per view ---------------------------------------------------------------------

/// The theme `cfg` selects for `view`: `CockpitHudStyle`, `BonnetHudStyle`, or for the chase
/// cameras `ChaseHudStyle` when set, else the cockpit cluster ([`AppConfig::theme`]).
#[must_use]
pub fn theme_for_view(cfg: &AppConfig, view: View) -> ClusterTheme {
    cfg.theme(view.fit_view())
}

/// Writes `theme` as the cluster for `view`.
pub fn set_theme_for_view(cfg: &mut AppConfig, view: View, theme: ClusterTheme) {
    cfg.set_theme(view.fit_view(), theme);
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
        ClusterTheme::LumenEtch => "Lumen Etch",
        ClusterTheme::VernierLoupe => "Vernier Loupe",
        ClusterTheme::DichroicLouvre => "Dichroic Louvre",
        ClusterTheme::FibreOpticCorona => "Fibre-Optic Corona",
        ClusterTheme::ApertureNine => "Aperture Nine",
        ClusterTheme::AeroSmokeTunnel => "Aero Smoke Tunnel",
        ClusterTheme::RattrapanteSector => "Rattrapante Sector",
        ClusterTheme::SapphireMystery => "Sapphire Mystery",
        ClusterTheme::RackFocus => "Rack Focus",
        ClusterTheme::BionicSpaceframe => "Bionic Spaceframe",
        ClusterTheme::GeodesicWell => "Geodesic Well",
        ClusterTheme::Lightpainter => "Lightpainter",
        ClusterTheme::TreadSeismograph => "Tread Seismograph",
        ClusterTheme::StarSailOrigami => "Star Sail Origami",
        ClusterTheme::ArcLadderCascade => "Arc Ladder Cascade",
        ClusterTheme::AuroraVeil => "Aurora Veil",
        ClusterTheme::IsochromaticLoadPath => "Isochromatic Load Path",
        ClusterTheme::AbyssalShearBloom => "Abyssal Shear Bloom",
        ClusterTheme::HarmonographDrive => "Harmonograph Drive",
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

/// The numeric value of a slider row.
#[must_use]
pub fn slider_value(row: RowId, ctx: Ctx<'_>) -> f32 {
    let cfg = ctx.cfg;
    let hsv = ctx.edit.hsv[ctx.edit.target_index()];
    match row {
        RowId::ShiftPoint => cfg.shift_point_pct.unwrap_or(DEFAULT_SHIFT_PCT) * 100.0,
        RowId::Hue => hsv.h,
        RowId::Saturation => hsv.s * 100.0,
        RowId::Brightness => hsv.v * 100.0,
        RowId::FovCockpit => cfg.wheel_less_cockpit_fov,
        RowId::FovHood => cfg.bonnet_fov,
        RowId::FovChase => cfg.chase_near_fov,
        RowId::SeatHeight => cfg.driver_seat_height_m * 1000.0,
        RowId::WheelOffset => cfg.steering_wheel_offset_y_m * 1000.0,
        RowId::ClusterHeight => match ctx.edit.view {
            View::Cockpit => cfg.cockpit_height_offset_px,
            View::Hood => cfg.bonnet_height_offset_px,
            View::Chase => cfg.chase_height_offset_px,
        },
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
        RowId::ShiftPoint => cfg.shift_point_pct = Some((v / 100.0 * 1000.0).round() / 1000.0),
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
            // The patcher clamps every FOV to 40–110 when it writes the camera file.
            cfg.chase_far_fov = (v + delta).clamp(40.0, 110.0);
        }
        RowId::SeatHeight => cfg.driver_seat_height_m = v / 1000.0,
        RowId::WheelOffset => cfg.steering_wheel_offset_y_m = v / 1000.0,
        RowId::ClusterHeight => {
            let v = v.round();
            match edit.view {
                View::Cockpit => cfg.cockpit_height_offset_px = v,
                View::Hood => cfg.bonnet_height_offset_px = v,
                View::Chase => cfg.chase_height_offset_px = v,
            }
        }
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
            RowId::SpeedUnit => out.write_str(match cfg.speed_unit {
                SpeedUnit::Mph => "mph",
                SpeedUnit::Kmh => "km/h",
            }),
            RowId::PowerUnit => out.write_str(match cfg.power_unit {
                PowerUnit::Hp => "hp",
                PowerUnit::Kw => "kW",
            }),
            RowId::LimiterStyle => out.write_str(if cfg.reduced_flash {
                "Gentle pulse"
            } else {
                "10 Hz strobe"
            }),
            RowId::ColourMode => out.write_str(ColourMode::of(cfg).label()),
            RowId::ColourRole => out.write_str(edit.colour_target.label()),
            RowId::Preset => out.write_str(PRESETS[edit.preset % PRESETS.len()].name),
            RowId::Font => out.write_str(cfg.cluster_font.as_deref().unwrap_or("Pack default")),
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
        RowId::PaletteLock => cfg.palette_seed.is_some(),
        RowId::GlassRefraction => cfg.glass_refraction,
        RowId::ShowLogo => cfg.show_logo,
        RowId::CameraPatch => cfg.camera_patch_auto,
        RowId::AutoPause => cfg.osd_auto_pause_game,
        RowId::Stats => cfg.show_stats,
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
                RowId::PaletteLock => cfg.palette_seed = on.then(|| new_seed(0)),
                RowId::GlassRefraction => cfg.glass_refraction = on,
                RowId::ShowLogo => cfg.show_logo = on,
                RowId::CameraPatch => {
                    cfg.camera_patch_auto = on;
                    // The camera rows' notice follows the patch state.
                    edit.camera = fit.camera_changed(cfg);
                }
                RowId::AutoPause => cfg.osd_auto_pause_game = on,
                RowId::Stats => cfg.show_stats = on,
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
                cfg.palette_seed = Some(new_seed(cfg.palette_seed.unwrap_or(1)));
                Effect::CHANGED
            }
            RowId::Resync => {
                fit.resync(cfg);
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
            cfg.speed_unit = match cfg.speed_unit {
                SpeedUnit::Mph => SpeedUnit::Kmh,
                SpeedUnit::Kmh => SpeedUnit::Mph,
            };
            Effect::CHANGED
        }
        RowId::PowerUnit => {
            cfg.power_unit = match cfg.power_unit {
                PowerUnit::Hp => PowerUnit::Kw,
                PowerUnit::Kw => PowerUnit::Hp,
            };
            Effect::CHANGED
        }
        RowId::LimiterStyle => {
            cfg.reduced_flash = !cfg.reduced_flash;
            Effect::CHANGED
        }
        RowId::ColourMode => {
            let next = ColourMode::of(cfg).step(d);
            cfg.color_mode = next.value();
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
            let cur = cfg
                .cluster_font
                .as_deref()
                .and_then(|f| edit.fonts.iter().position(|x| x.eq_ignore_ascii_case(f)))
                .map_or(0, |i| i as i32 + 1);
            let next = (cur + d).rem_euclid(n);
            if next == 0 {
                cfg.cluster_font = None;
            } else if let Some(name) = edit.fonts.get((next - 1) as usize) {
                cfg.cluster_font = Some(name.clone());
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
        cfg.wheel_less_cockpit_fov = 109.0;
        let e = adjust(RowId::FovCockpit, 1, 10, &mut cfg, &mut edit, &mut fit);
        assert!(e.changed && e.camera);
        assert_eq!(cfg.standard_cockpit_fov, 110.0);
        assert_eq!(cfg.wheel_less_cockpit_fov, 110.0);
        let e = adjust(RowId::FovCockpit, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert!(!e.changed, "already at max");
        adjust(RowId::ClusterHeight, -1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.cockpit_height_offset_px, -4.0, "edits the edited view (cockpit)");
        assert_eq!(cfg.bonnet_height_offset_px, 0.0);
        let mut line = Line::new();
        format_value(RowId::ClusterHeight, ctx(&cfg, &edit), &mut line);
        assert_eq!(line.as_str(), "-4 px");
    }

    #[test]
    fn cluster_carousel_writes_the_edited_view() {
        let mut cfg = AppConfig::default();
        let mut edit = EditState::default();
        let mut fit = ConfigOnlyFit;
        cfg.cockpit_hud_style = ClusterTheme::LiquidGlass;
        adjust(RowId::Cluster, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.cockpit_hud_style, ClusterTheme::WindshieldHud);
        edit.view = View::Chase;
        assert_eq!(
            theme_for_view(&cfg, View::Chase),
            ClusterTheme::WindshieldHud,
            "chase follows cockpit"
        );
        adjust(RowId::Cluster, -1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(theme_for_view(&cfg, View::Chase), ClusterTheme::LiquidGlass);
        assert_eq!(theme_for_view(&cfg, View::Cockpit), ClusterTheme::WindshieldHud);
        // Every theme can be set and read back.
        for t in ClusterTheme::all() {
            set_theme_for_view(&mut cfg, View::Hood, *t);
            assert_eq!(theme_for_view(&cfg, View::Hood), *t);
        }
    }

    /// A backend that offers the resync action and records the camera it was asked for.
    struct ResyncFit(Option<i32>);

    impl CockpitFit for ResyncFit {
        fn camera_changed(&mut self, cfg: &AppConfig) -> CameraApply {
            if cfg.camera_patch_auto {
                CameraApply::NextLaunch
            } else {
                CameraApply::PatchOff
            }
        }
        fn can_resync(&self) -> bool {
            true
        }
        fn resync(&mut self, cfg: &AppConfig) {
            self.0 = Some(cfg.resync_camera_index);
        }
    }

    #[test]
    fn fit_tab_binds_the_cockpit_fit_keys() {
        let mut cfg = AppConfig::default();
        let mut edit = EditState::default();
        let mut fit = ResyncFit(None);

        // Cluster height is per view: it edits the view being edited and leaves the others.
        edit.view = View::Hood;
        adjust(RowId::ClusterHeight, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.bonnet_height_offset_px, 4.0);
        assert_eq!(cfg.cockpit_height_offset_px, 0.0);
        edit.view = View::Chase;
        adjust(RowId::ClusterHeight, -1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.chase_height_offset_px, -4.0);
        assert!(
            !cfg.extra.contains_key("ClusterHeightOffsetPx"),
            "the old free-form key is gone"
        );

        // The chase cluster goes to the typed key the render loop reads.
        set_theme_for_view(&mut cfg, View::Chase, ClusterTheme::RetroCyber);
        assert_eq!(cfg.chase_hud_style, Some(ClusterTheme::RetroCyber));
        assert!(!cfg.extra.contains_key("ChaseHudStyle"));

        // The camera patch switch drives the notice under the camera rows.
        assert!(!toggle_value(RowId::CameraPatch, &cfg), "off by default");
        adjust(RowId::FovCockpit, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(help(RowId::FovCockpit, &edit), Some(CameraApply::PatchOff.notice()));
        assert!(adjust(RowId::CameraPatch, 1, 1, &mut cfg, &mut edit, &mut fit).changed);
        assert!(cfg.camera_patch_auto);
        assert_eq!(help(RowId::SeatHeight, &edit), Some(CameraApply::NextLaunch.notice()));

        // Every dash-blocker row and the resync action fit on the tab.
        let r = rows(
            Tab::Fit,
            Ctx {
                cfg: &cfg,
                edit: &edit,
                fit: &fit,
            },
        );
        let scrim = r.as_slice().iter().filter(|r| matches!(r, RowId::Scrim(_))).count();
        assert_eq!(scrim, crate::fit::SCRIM_ROWS.len());
        assert_eq!(r.as_slice().last(), Some(&RowId::Resync));
        cfg.resync_camera_index = 4;
        adjust(RowId::Resync, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(fit.0, Some(4));
        adjust(RowId::Resync, -1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(fit.0, Some(4), "left does nothing on an action row");

        // A blocker row writes its Phase 8 key inside its range.
        cfg.oem_dash_feather_px = 240.0;
        let feather = crate::fit::SCRIM_ROWS
            .iter()
            .position(|f| f.label == "Edge feather")
            .map(|i| RowId::Scrim(i as u8));
        let feather = feather.expect("feather row");
        assert!(
            !adjust(feather, 1, 10, &mut cfg, &mut edit, &mut fit).changed,
            "already at max"
        );
        adjust(feather, -1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.oem_dash_feather_px, 236.0);
    }

    #[test]
    fn colour_modes_and_custom_edits() {
        let mut cfg = AppConfig::default();
        let mut edit = EditState::default();
        let mut fit = ConfigOnlyFit;
        edit.load_colours(&cfg);
        cfg.color_mode = ColorMode::AutoBrand;
        adjust(RowId::ColourMode, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.color_mode, ColorMode::Custom);
        assert!(rows(Tab::Colour, ctx(&cfg, &edit)).as_slice().contains(&RowId::Hue));
        adjust(RowId::Preset, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.primary_hex, PRESETS[1].hex[0]);
        adjust(RowId::Hue, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_ne!(cfg.primary_hex, PRESETS[1].hex[0]);
        adjust(RowId::ColourMode, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.color_mode, ColorMode::Random);
        adjust(RowId::Reroll, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert!(toggle_value(RowId::PaletteLock, &cfg));
        adjust(RowId::PaletteLock, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.palette_seed, None);
        // A config in Brand mode shows as Factory, and stepping from it leaves Brand.
        cfg.color_mode = ColorMode::Brand;
        assert_eq!(ColourMode::of(&cfg), ColourMode::Factory);
        adjust(RowId::ColourMode, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.color_mode, ColorMode::Custom);
    }

    #[test]
    fn toggles_and_actions() {
        let mut cfg = AppConfig::default();
        let mut edit = EditState::default();
        let mut fit = ConfigOnlyFit;
        assert!(cfg.show_logo);
        adjust(RowId::ShowLogo, -1, 1, &mut cfg, &mut edit, &mut fit);
        assert!(!cfg.show_logo);
        assert!(adjust(RowId::CustomName, 1, 1, &mut cfg, &mut edit, &mut fit).keyboard);
        assert!(!adjust(RowId::CustomName, -1, 1, &mut cfg, &mut edit, &mut fit).keyboard);
        assert!(adjust(RowId::OsdButton, 1, 1, &mut cfg, &mut edit, &mut fit).remap);
        edit.fonts = vec!["Saira Condensed".into(), "Michroma".into()];
        adjust(RowId::Font, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.cluster_font.as_deref(), Some("Saira Condensed"));
        adjust(RowId::Font, -1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.cluster_font, None);
    }

    #[test]
    fn units_shift_point_and_limiter_rows_edit_typed_fields() {
        let mut cfg = AppConfig::default();
        let mut edit = EditState::default();
        let mut fit = ConfigOnlyFit;
        let mut line = Line::new();
        adjust(RowId::SpeedUnit, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.speed_unit, SpeedUnit::Kmh);
        format_value(RowId::SpeedUnit, ctx(&cfg, &edit), &mut line);
        assert_eq!(line.as_str(), "km/h");
        assert_eq!(cfg.power_unit, PowerUnit::Hp, "the power unit is its own setting");
        adjust(RowId::PowerUnit, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert_eq!(cfg.power_unit, PowerUnit::Kw);
        adjust(RowId::LimiterStyle, 1, 1, &mut cfg, &mut edit, &mut fit);
        assert!(cfg.reduced_flash);

        assert_eq!(cfg.shift_point_pct, None);
        assert!((slider_value(RowId::ShiftPoint, ctx(&cfg, &edit)) - 97.0).abs() < 1e-3);
        adjust(RowId::ShiftPoint, -1, 1, &mut cfg, &mut edit, &mut fit);
        let set = cfg.shift_point_pct.expect("the slider sets the shift point");
        assert!((0.80..=1.0).contains(&set), "{set}");
        assert!(
            cfg.extra.is_empty(),
            "nothing lands in the untyped keys: {:?}",
            cfg.extra
        );
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
