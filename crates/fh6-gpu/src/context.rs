//! Instance, adapter, device and queue.
use crate::block_on::block_on;
use crate::GpuError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// What the chosen adapter is, for logs and perf reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterSummary {
    /// Adapter name, e.g. "NVIDIA GeForce RTX 3070 Ti Laptop GPU".
    pub name: String,
    /// Backend, e.g. "Dx12".
    pub backend: String,
    /// Device type, e.g. "DiscreteGpu" or "Cpu" for WARP.
    pub device_type: String,
    /// Driver description.
    pub driver: String,
}

impl std::fmt::Display for AdapterSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}, {}, {})",
            self.name, self.backend, self.device_type, self.driver
        )
    }
}

/// The GPU device the overlay renders with.
#[derive(Debug)]
pub struct GpuContext {
    /// The wgpu instance (DX12, DirectComposition swapchains).
    pub instance: wgpu::Instance,
    /// The chosen adapter.
    pub adapter: wgpu::Adapter,
    /// The logical device.
    pub device: wgpu::Device,
    /// Its queue.
    pub queue: wgpu::Queue,
    summary: AdapterSummary,
    /// Set by wgpu's uncaptured-error and device-lost callbacks (which would otherwise panic).
    failed: Arc<AtomicBool>,
}

/// The instance every surface and device is made from: DX12 only, with swapchains created on a
/// DirectComposition visual (the only DX12 route with per-pixel alpha, see ADR 0001).
pub(crate) fn new_instance() -> wgpu::Instance {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    desc.backends = wgpu::Backends::DX12;
    desc.backend_options.dx12.presentation_system = wgpu::Dx12SwapchainKind::DxgiFromVisual;
    desc.backend_options.dx12.latency_waitable_object = wgpu::Dx12UseFrameLatencyWaitableObject::Wait;
    wgpu::Instance::new(desc)
}

impl GpuContext {
    /// A device without any window: hardware adapter first, then WARP.
    ///
    /// # Errors
    /// [`GpuError::NoAdapter`] if neither exists, [`GpuError::Device`] if device creation fails.
    pub fn headless() -> Result<Self, GpuError> {
        let instance = new_instance();
        Self::from_instance(instance, None)
    }

    /// A device able to present to `surface` (made from the same instance).
    pub(crate) fn from_instance(
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, GpuError> {
        let hardware = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: surface,
            ..Default::default()
        }));
        let adapter = match hardware {
            Ok(adapter) => adapter,
            Err(err) => {
                tracing::warn!(error = %err, "no hardware adapter, trying WARP");
                block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    force_fallback_adapter: true,
                    compatible_surface: surface,
                    ..Default::default()
                }))?
            }
        };
        let info = adapter.get_info();
        let summary = AdapterSummary {
            name: info.name.clone(),
            backend: format!("{:?}", info.backend),
            device_type: format!("{:?}", info.device_type),
            driver: format!("{} {}", info.driver, info.driver_info).trim().to_string(),
        };
        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("fh6-gpu device"),
            // Optional extras: GPU timestamps (benches) and BC textures (KTX2 assets), when present.
            required_features: adapter.features()
                & (wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TEXTURE_COMPRESSION_BC),
            required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        }))?;
        let failed = Arc::new(AtomicBool::new(false));
        {
            let failed = Arc::clone(&failed);
            device.on_uncaptured_error(Arc::new(move |err| {
                tracing::error!(error = %err, "GPU error");
                failed.store(true, Ordering::Relaxed);
            }));
        }
        {
            let failed = Arc::clone(&failed);
            device.set_device_lost_callback(move |reason, message| {
                if reason == wgpu::DeviceLostReason::Destroyed {
                    tracing::debug!(%message, "GPU device destroyed");
                } else {
                    tracing::error!(?reason, %message, "GPU device lost");
                    failed.store(true, Ordering::Relaxed);
                }
            });
        }
        tracing::info!(adapter = %summary, "GPU device ready");
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            summary,
            failed,
        })
    }

    /// The adapter in use.
    pub fn adapter_summary(&self) -> &AdapterSummary {
        &self.summary
    }

    /// True once wgpu reported an uncaptured error or lost the device. The caller should drop
    /// this context (and its surfaces) and create a new one.
    pub fn has_failed(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }
}
