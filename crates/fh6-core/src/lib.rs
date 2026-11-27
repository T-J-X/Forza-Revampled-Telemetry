//! fh6-core: Core data models, telemetry parser, config, and state machines for FH6 WheelDash.

pub mod atomic_file;
pub mod camera_status;
pub mod cars;
pub mod config;
pub mod config_watch;
pub mod fit;
pub mod oled_care;
pub mod profiles;
mod serde_str;
pub mod telemetry;
pub mod theme;
pub mod trail;
pub mod udp;
pub mod visibility;

pub use camera_status::{CameraPatchHandle, CameraPatchState, CameraPatchStatus, CameraRequest};
pub use cars::{BrandPalette, CarDatabase, CarDatabaseError, CarEntry};
pub use config::{AppConfig, ColorMode, ParseSettingError, PowerUnit, SafeZone, SpeedUnit, TelemetryPorts};
pub use config_watch::{ConfigHandle, ConfigWatcher};
pub use fit::{FitView, ScrimFit, ViewPlacement};
pub use profiles::{CarProfile, CarProfilesFile, ProfilesHandle};
pub use telemetry::FH6TelemetryData;
pub use theme::{ClusterTheme, ParseClusterThemeError};
pub use trail::WorldMapTrailBuffer;
pub use udp::{TelemetryEngine, TelemetryFrame, TelemetryReader};
pub use visibility::{HideReason, Visibility, VisibilityController, VisibilityInputs};
