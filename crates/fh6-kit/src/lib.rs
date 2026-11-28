//! fh6-kit: the GPU 2D/2.5D render kit every FH6 WheelDash cluster is built with.
//!
//! - **Scene and batching:** an immediate-mode [`Painter`] records into a per-frame arena
//!   ([`Scene`]); draws sharing a blend mode and texture become one instanced draw call.
//! - **Shapes:** analytic signed-distance primitives ([`Shape`]) filled, stroked (inner, centred,
//!   outer, or exact device-pixel rims), glowed or shadowed.
//! - **Paint:** solid, linear/radial/conic gradients, images and 9-slice, the Liquid Glass
//!   material; blend modes normal, add, screen, multiply. Linear premultiplied end to end.
//! - **Light:** per-layer bloom (dual-filter Kawase), Gaussian layer blur, glass backdrop with
//!   frost, fresnel rim, crown sheen and optional world refraction.
//! - **Text:** MTSDF atlas with kerning, tracking and tabular figures ([`Fonts`]); [`StackStr`]
//!   formats numbers on a frame path without allocating.
//! - **Motion:** closed-form springs, smoothing and easing ([`anim`]).
//! - **Effects:** GPU fluid ([`fluid`]), particles, noise, the 2.5D road camera
//!   ([`perspective`]) and a glTF mesh pass ([`mesh`]).
//! - **Contract:** the [`Cluster`] trait, [`FrameInput`] and [`Layout`]; the palette engine
//!   ([`Palette`]) and design [`Tokens`].
//!
//! The kit takes a `wgpu` device and queue, so the same code runs natively (DX12 through
//! `fh6-gpu`) and in a browser (WebGPU). See `docs/kit/README.md` for the author's guide.
#![forbid(unsafe_code)]

pub mod anim;
mod clock;
pub mod cluster;
pub mod color;
mod family;
pub mod fluid;
pub mod images;
mod kit;
pub mod logo;
pub mod math;
pub mod mesh;
pub mod noise;
pub mod optics;
mod paint;
mod painter;
pub mod palette;
pub mod particles;
pub mod perspective;
mod pool;
mod renderer;
mod scene;
pub mod scrim;
pub mod shape;
mod stack_str;
pub mod testing;
pub mod text;
pub mod tokens;

pub use anim::{Easing, Hysteresis, Smoothed, Spring, SpringSpec, Strobe, Tween};
pub use cluster::{
    Cluster, ClusterEntry, FontChoice, FrameInput, Gear, Layout, Logo, PowerUnit, Setup, SignalSmoother, Signals,
    SpeedUnit, View,
};
pub use color::Color;
pub use family::Family;
pub use images::Images;
pub use kit::{Kit, KitError};
pub use math::{deg, vec2, Affine, Rect, Vec2};
pub use paint::{
    BlendMode, Environment, Glass, Gradient, GradientKind, ImageId, ImagePaint, ImageWrap, OpticalGlass, Paint,
    PlaneWave, Refraction, Ripple, Spread, StandingWaves, Stop, LIGHT_REST_AZIMUTH, MAX_STANDING_WAVES,
};
pub use painter::Painter;
pub use palette::{Palette, PaletteSource};
pub use pool::{Pool, MAX_POOLS};
pub use renderer::{FrameStats, GpuPassKind, GpuPassTime};
pub use scene::{Backdrop, Bloom, LayerOptions, Scene};
pub use shape::{
    Axis, Cap, Dome, Glow, Outline, Piece, Radii, SegmentedBar, Shadow, Shape, Stroke, StrokeAlign, StrokeUnit,
    TickRing,
};
pub use stack_str::StackStr;
pub use text::{Align, Figures, FontId, FontRole, Fonts, TextStyle, VAlign};
pub use tokens::Tokens;
pub use wgpu;
