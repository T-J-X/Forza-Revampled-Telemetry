//! Clusters built on `fh6-kit` (contract v2). Phase 5 ships two showcase samples; Phase 7 adds
//! one module per cluster and a row per cluster in [`crate::registry::REGISTRY`].
pub mod analog;
pub mod liquid_glass;

pub use analog::AnalogSample;
pub use liquid_glass::LiquidGlassSample;
