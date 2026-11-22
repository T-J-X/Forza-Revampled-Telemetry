//! Cluster contract v2: what every cluster implements, what it receives each frame, and where
//! it is placed on screen.
//!
//! ```text
//! host, per frame:   signals = smoother.update(&telemetry, &config, dt)
//!                    cluster.update(&FrameInput { .. })        // &mut self: springs, state
//!                    let mut p = kit.begin_frame(&layout)
//!                    cluster.draw(&mut p, &layout)              // &self: record shapes
//!                    kit.render(..)                             // one batched GPU frame
//! ```
use crate::anim::Smoothed;
use crate::images::Images;
use crate::math::{finite_or, vec2, Affine, Rect, Vec2};
use crate::mesh::Meshes;
use crate::paint::ImageId;
use crate::painter::Painter;
use crate::palette::Palette;
use crate::text::{FontId, Fonts};
use crate::tokens::Tokens;
use fh6_core::{AppConfig, ClusterTheme, FH6TelemetryData};

/// Reference height of the authoring space (4K).
pub const REF_HEIGHT: f32 = 2160.0;
/// Reference width of the authoring space (4K).
pub const REF_WIDTH: f32 = 3840.0;

/// The camera view a cluster is shown in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// Wheel-less cockpit `[3]`: the cluster sits in the steering-column gap.
    #[default]
    Cockpit,
    /// Hood `[4]`: wider, lower, frameless; scrim forced to 0%.
    Hood,
    /// Chase cameras (routing arrives in Phase 8; uses the cockpit anchor for now).
    Chase,
}

impl View {
    /// The view for a camera ring index (`3` cockpit, `4` hood, others chase).
    #[must_use]
    pub fn from_camera_index(index: i32) -> Self {
        match index {
            4 => View::Hood,
            3 => View::Cockpit,
            _ => View::Chase,
        }
    }
}

/// Where and how large a cluster is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// Anchor in target pixels.
    pub anchor: Vec2,
    /// Target pixels per local unit (user scale x target height / 2160).
    pub scale: f32,
    /// User height offset in local units (positive raises the cluster).
    pub height_offset: f32,
    /// The view.
    pub view: View,
    /// The envelope the cluster must stay inside, in local units around the anchor.
    pub envelope: Rect,
    /// Safe zone in target pixels (tokens: 40 px bottom, 160 px sides at 4K).
    pub safe: Rect,
    /// Target size in pixels.
    pub target: (u32, u32),
}

impl Layout {
    /// The layout for `view` from the config (anchors authored at 4K, user scale 0.70–1.40,
    /// height offset ±240 px from `ClusterHeightOffsetPx`).
    #[must_use]
    pub fn from_config(config: &AppConfig, tokens: &Tokens, view: View, target: (u32, u32)) -> Self {
        let (w, h) = (target.0.max(1) as f32, target.1.max(1) as f32);
        let (ax, ay, view_scale, envelope) = match view {
            View::Hood => (
                config.bonnet_anchor_x as f32,
                config.bonnet_anchor_y as f32,
                config.bonnet_scale,
                tokens.layout.hood_envelope,
            ),
            View::Cockpit | View::Chase => (
                config.cockpit_anchor_x as f32,
                config.cockpit_anchor_y as f32,
                config.cockpit_scale,
                tokens.layout.cockpit_envelope,
            ),
        };
        let user = finite_or(config.cluster_scale, 1.0).clamp(0.70, 1.40);
        let scale = finite_or(view_scale, 1.0).clamp(0.25, 4.0) * user * h / REF_HEIGHT;
        let height_offset = config
            .extra
            .get("ClusterHeightOffsetPx")
            .and_then(serde_json::Value::as_f64)
            .map_or(0.0, |v| (v as f32).clamp(-240.0, 240.0));
        let k = h / REF_HEIGHT;
        Self {
            anchor: vec2(ax * w / REF_WIDTH, ay * k),
            scale,
            height_offset,
            view,
            envelope,
            safe: Rect::from_points(vec2(160.0 * k, 0.0), vec2(w - 160.0 * k, h - 40.0 * k)),
            target,
        }
    }

    /// A layout centred at `anchor` (target px) with `scale` px per local unit (tests, tools).
    #[must_use]
    pub fn centered(anchor: Vec2, scale: f32, target: (u32, u32)) -> Self {
        let h = target.1.max(1) as f32;
        let k = h / REF_HEIGHT;
        Self {
            anchor,
            scale,
            height_offset: 0.0,
            view: View::Cockpit,
            envelope: Rect::from_xywh(-720.0, -320.0, 1440.0, 560.0),
            safe: Rect::from_points(vec2(160.0 * k, 0.0), vec2(target.0 as f32 - 160.0 * k, h - 40.0 * k)),
            target,
        }
    }

    /// Local units to target pixels.
    #[must_use]
    pub fn transform(&self) -> Affine {
        Affine::scale(self.scale).then(&Affine::translate(vec2(
            self.anchor.x,
            self.anchor.y - self.height_offset * self.scale,
        )))
    }

    /// A local point in target pixels.
    #[must_use]
    pub fn to_target(&self, p: Vec2) -> Vec2 {
        self.transform().apply(p)
    }
}

/// A gear as shown on a cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Gear {
    /// Reverse.
    Reverse,
    /// Neutral.
    #[default]
    Neutral,
    /// A forward gear (1..=10).
    Forward(u8),
}

impl Gear {
    /// From the telemetry byte (0 = reverse, 1..=10 forward, anything else neutral).
    #[must_use]
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Gear::Reverse,
            1..=10 => Gear::Forward(raw),
            _ => Gear::Neutral,
        }
    }

    /// The label: `R`, `N` or the number.
    #[must_use]
    pub fn label(self) -> &'static str {
        const N: [&str; 11] = ["N", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];
        match self {
            Gear::Reverse => "R",
            Gear::Neutral => "N",
            Gear::Forward(n) => N.get(usize::from(n)).copied().unwrap_or("N"),
        }
    }
}

/// Speed display units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpeedUnit {
    /// Miles per hour.
    #[default]
    Mph,
    /// Kilometres per hour.
    Kmh,
}

impl SpeedUnit {
    /// The unit label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            SpeedUnit::Mph => "MPH",
            SpeedUnit::Kmh => "KM/H",
        }
    }
}

/// Telemetry decoded and smoothed for drawing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Signals {
    /// True while racing (`IsRaceOn == 1`).
    pub race_on: bool,
    /// Speed in display units.
    pub speed: f32,
    /// Display unit.
    pub unit: SpeedUnit,
    /// Speed in m/s.
    pub speed_ms: f32,
    /// Engine rpm.
    pub rpm: f32,
    /// Engine rpm change rate (rpm/s), for needle feed-forward.
    pub rpm_rate: f32,
    /// Limiter rpm (`EngineMaxRpm`).
    pub max_rpm: f32,
    /// Idle rpm.
    pub idle_rpm: f32,
    /// Shift rpm (limiter x tokens shift %).
    pub shift_rpm: f32,
    /// `rpm / max_rpm`, 0..=1.
    pub rpm_fraction: f32,
    /// `rpm / shift_rpm` (shift-light stages compare against this).
    pub shift_fraction: f32,
    /// Gear.
    pub gear: Gear,
    /// Throttle 0..=1.
    pub throttle: f32,
    /// Brake 0..=1.
    pub brake: f32,
    /// Clutch 0..=1.
    pub clutch: f32,
    /// Handbrake 0..=1.
    pub handbrake: f32,
    /// Lateral G (smoothed; positive = acceleration to the right).
    pub g_lat: f32,
    /// Longitudinal G (smoothed; positive = accelerating).
    pub g_long: f32,
    /// Body slip angle in degrees (smoothed).
    pub slip_deg: f32,
    /// Yaw rate in rad/s.
    pub yaw_rate: f32,
    /// World position (metres).
    pub position: [f32; 3],
    /// Car ordinal.
    pub car_ordinal: i32,
}

/// Turns raw telemetry into [`Signals`], smoothing the noisy channels (G, slip).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignalSmoother {
    g_lat: Smoothed<f32>,
    g_long: Smoothed<f32>,
    slip: Smoothed<f32>,
    rpm_rate: Smoothed<f32>,
    last_rpm: Option<f32>,
}

impl Default for SignalSmoother {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalSmoother {
    /// Smoothing with 60 ms (G), 80 ms (slip) and 40 ms (rpm rate) time constants.
    #[must_use]
    pub fn new() -> Self {
        Self {
            g_lat: Smoothed::new(0.0, 0.06),
            g_long: Smoothed::new(0.0, 0.06),
            slip: Smoothed::new(0.0, 0.08),
            rpm_rate: Smoothed::new(0.0, 0.04),
            last_rpm: None,
        }
    }

    /// Advances by `dt` with a new telemetry sample.
    pub fn update(&mut self, data: &FH6TelemetryData, config: &AppConfig, tokens: &Tokens, dt: f32) -> Signals {
        let f = |v: f32| finite_or(v, 0.0);
        let unit = if config.speed_unit.eq_ignore_ascii_case("KMH") || config.speed_unit.eq_ignore_ascii_case("KPH") {
            SpeedUnit::Kmh
        } else {
            SpeedUnit::Mph
        };
        let speed_ms = f(data.speed_ms).max(0.0);
        let max_rpm = f(data.engine_max_rpm).max(1.0);
        let rpm = f(data.current_engine_rpm).clamp(0.0, max_rpm * 1.1);
        let rate = match self.last_rpm {
            Some(last) if dt > 1e-4 => (rpm - last) / dt,
            _ => 0.0,
        };
        self.last_rpm = Some(rpm);
        let shift_rpm = max_rpm * tokens.rev_stages.shift_pct.clamp(0.5, 1.0);
        Signals {
            race_on: data.is_race_on == 1,
            speed: match unit {
                SpeedUnit::Mph => f(data.speed_mph()),
                SpeedUnit::Kmh => f(data.speed_kmh()),
            }
            .max(0.0),
            unit,
            speed_ms,
            rpm,
            rpm_rate: self.rpm_rate.update(rate, dt),
            max_rpm,
            idle_rpm: f(data.engine_idle_rpm).max(0.0),
            shift_rpm,
            rpm_fraction: (rpm / max_rpm).clamp(0.0, 1.0),
            shift_fraction: rpm / shift_rpm.max(1.0),
            gear: Gear::from_raw(data.gear),
            throttle: data.throttle_pct(),
            brake: data.brake_pct(),
            clutch: data.clutch_pct(),
            handbrake: data.handbrake_pct(),
            g_lat: self.g_lat.update(f(data.lateral_g()), dt),
            g_long: self.g_long.update(f(data.longitudinal_g()), dt),
            slip_deg: self.slip.update(f(data.car_slip_angle_rad()).to_degrees(), dt),
            yaw_rate: f(data.angular_velocity_y),
            position: [f(data.position_x), f(data.position_y), f(data.position_z)],
            car_ordinal: data.car_ordinal,
        }
    }
}

/// The logo shown in a cluster's logo zone, with its fallbacks.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Logo {
    /// The brand logo image, when one is loaded.
    pub image: Option<ImageId>,
    /// The brand name (first fallback).
    pub brand_name: String,
    /// The user's custom name (second fallback, and shown in name zones).
    pub custom_name: String,
}

/// Fonts chosen for this frame (tokens roles, or the user's choice for `display`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontChoice {
    /// Hero numerals.
    pub display: FontId,
    /// Labels and units.
    pub text: FontId,
    /// Wordmarks and the custom name.
    pub wide: FontId,
}

/// Everything a cluster's `update` receives.
#[derive(Debug, Clone, Copy)]
pub struct FrameInput<'a> {
    /// The raw latest packet.
    pub telemetry: &'a FH6TelemetryData,
    /// Decoded and smoothed values.
    pub signals: &'a Signals,
    /// Seconds since the previous frame.
    pub dt: f32,
    /// Seconds since the cluster was created.
    pub t: f32,
    /// Live config.
    pub config: &'a AppConfig,
    /// Active palette (factory, custom or random).
    pub palette: &'a Palette,
    /// Design tokens.
    pub tokens: &'a Tokens,
    /// Fonts.
    pub fonts: FontChoice,
    /// Logo and names.
    pub logo: &'a Logo,
    /// The current view.
    pub view: View,
    /// Reduced-flash accessibility mode (strobes become a 2 Hz breath).
    pub reduced_flash: bool,
}

/// Resources a cluster may load when it is created.
#[derive(Debug)]
pub struct Setup<'a> {
    /// The device.
    pub device: &'a wgpu::Device,
    /// The queue.
    pub queue: &'a wgpu::Queue,
    /// Image store (load textures here).
    pub images: &'a mut Images,
    /// Mesh store (upload glTF cars here).
    pub meshes: &'a mut Meshes,
    /// Fonts.
    pub fonts: &'a mut Fonts,
    /// Tokens.
    pub tokens: &'a Tokens,
    /// The repository / install `assets/` folder, when known.
    pub assets: Option<&'a std::path::Path>,
}

/// A cluster: per-cluster state updated each frame, drawn with a [`Painter`].
pub trait Cluster {
    /// Advances state (springs, timers) for this frame.
    fn update(&mut self, input: &FrameInput<'_>);

    /// Records this frame's drawing. Coordinates are local (4K px around the anchor).
    fn draw(&self, p: &mut Painter<'_>, layout: &Layout);

    /// True while something still moves without new telemetry (springs settling, strobes), so
    /// the host keeps rendering between packets.
    fn animating(&self) -> bool {
        false
    }
}

/// One row of the cluster registry (the table the Phase 7 setup chat generates).
#[derive(Debug, Clone, Copy)]
pub struct ClusterEntry {
    /// Stable identifier (also the config value).
    pub theme: ClusterTheme,
    /// Display name.
    pub name: &'static str,
    /// Design family key (tokens `families`).
    pub family: &'static str,
    /// Builds the cluster, loading its resources.
    pub make: fn(&mut Setup<'_>) -> Box<dyn Cluster>,
}

/// Looks `theme` up in a registry table.
#[must_use]
pub fn find_entry(table: &'static [ClusterEntry], theme: ClusterTheme) -> Option<&'static ClusterEntry> {
    table.iter().find(|e| e.theme == theme)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_from_config_scales_from_4k() {
        let cfg = AppConfig::default();
        let t = Tokens::default();
        let l4 = Layout::from_config(&cfg, &t, View::Cockpit, (3840, 2160));
        let l1 = Layout::from_config(&cfg, &t, View::Cockpit, (1920, 1080));
        assert!((l4.anchor.x - cfg.cockpit_anchor_x as f32).abs() < 1e-3);
        assert!((l1.anchor.x * 2.0 - l4.anchor.x).abs() < 1e-3);
        assert!((l1.scale * 2.0 - l4.scale).abs() < 1e-5);
        let p = l4.to_target(Vec2::ZERO);
        assert!((p - l4.anchor).length() < 1e-3);
    }

    #[test]
    fn gears_decode() {
        assert_eq!(Gear::from_raw(0).label(), "R");
        assert_eq!(Gear::from_raw(4).label(), "4");
        assert_eq!(Gear::from_raw(10).label(), "10");
        assert_eq!(Gear::from_raw(11).label(), "N");
    }

    #[test]
    fn signals_decode_and_smooth() {
        let mut s = SignalSmoother::new();
        let data = FH6TelemetryData {
            is_race_on: 1,
            engine_max_rpm: 9000.0,
            current_engine_rpm: 4500.0,
            speed_ms: 20.0,
            gear: 3,
            accel: 255,
            ..Default::default()
        };
        let cfg = AppConfig::default();
        let sig = s.update(&data, &cfg, &Tokens::default(), 1.0 / 60.0);
        assert!(sig.race_on);
        assert!((sig.rpm_fraction - 0.5).abs() < 1e-5);
        assert_eq!(sig.gear, Gear::Forward(3));
        assert!((sig.throttle - 1.0).abs() < 1e-5);
        assert!(sig.speed > 40.0);
    }
}
