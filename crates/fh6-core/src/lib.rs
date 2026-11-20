//! fh6-core: Core data models, telemetry parser, config, and state machines for FH6 WheelDash.

pub mod cars;
pub mod config;
pub mod oled_care;
pub mod profiles;
pub mod rev_limiter;
pub mod shift_guard;
pub mod telemetry;
pub mod theme;
pub mod trail;
pub mod udp;

pub use cars::{BrandPalette, CarDatabase, CarEntry, CarOrdinals};
pub use config::AppConfig;
pub use config::{CameraViewMode, FontPreset};
pub use profiles::{CarProfile, CarProfileDatabase, CarProfilesFile};
pub use telemetry::FH6TelemetryData;
pub use theme::ClusterTheme;
pub use trail::WorldMapTrailBuffer;
pub use udp::TelemetryEngine;
