//! fh6-gpu: the GPU foundation of FH6 WheelDash (ADR 0001).
//!
//! - [`GpuContext`]: wgpu instance, adapter, device and queue on DX12, with the DirectComposition
//!   swapchain mode that gives premultiplied per-pixel alpha.
//! - [`OverlaySurface`]: the overlay window's swapchain (premultiplied, Mailbox or Fifo, frame
//!   latency 1), resize-safe; [`OverlaySurface::render_with`] presents what the render kit
//!   records.
//!
//! Offscreen targets and readback for tests, benches and `preview-export` are the kit's
//! (`fh6_kit::testing`). `wgpu` is re-exported so dependants name the exact version this crate
//! uses.

mod block_on;
mod context;
mod surface;

pub use context::{AdapterSummary, GpuContext};
pub use surface::{FrameOutcome, FrameTimings, OverlaySurface};
pub use wgpu;

/// Everything that can go wrong in the GPU layer.
#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    /// No DX12 adapter (hardware or WARP) is available.
    #[error("no compatible GPU adapter: {0}")]
    NoAdapter(#[from] wgpu::RequestAdapterError),
    /// The adapter refused to create a device.
    #[error("creating the GPU device")]
    Device(#[from] wgpu::RequestDeviceError),
    /// The window surface could not be created.
    #[error("creating the window surface")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    /// The surface cannot present with premultiplied alpha, so the overlay would be opaque.
    #[error("the surface does not support premultiplied alpha (supported: {0:?})")]
    NoPremultipliedAlpha(Vec<wgpu::CompositeAlphaMode>),
    /// The surface offers no usable format.
    #[error("the surface offers no texture format")]
    NoSurfaceFormat,
    /// The surface was lost and must be recreated.
    #[error("the surface was lost")]
    SurfaceLost,
    /// A validation error while acquiring a frame.
    #[error("surface validation error")]
    SurfaceValidation,
    /// wgpu reported an uncaptured error (validation, a failed surface configure) or lost the
    /// device. Drop the context and its surfaces and create new ones.
    #[error("the GPU reported an error or lost the device")]
    DeviceFailed,
}
