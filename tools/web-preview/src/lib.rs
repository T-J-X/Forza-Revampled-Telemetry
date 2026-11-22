//! web-preview: a kit cluster rendered in the browser with WebGPU, driven by a synthetic
//! drivetrain. Hold Space for throttle, press 1–9 for gears. `?cluster=PorscheAnalog` picks the
//! cluster. Phase 10's configurator builds on this page.
//!
//! Build and serve with `cargo run -p web-preview --bin web-preview-build` and
//! `cargo run -p web-preview --bin web-preview-serve`, then open <http://127.0.0.1:5310/>.
pub mod sim;

#[cfg(target_arch = "wasm32")]
mod web;
