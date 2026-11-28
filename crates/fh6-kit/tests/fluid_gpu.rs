//! GPU tests for the fluid solver and the WGSL noise functions on a headless DX12 device.
#![cfg(not(target_arch = "wasm32"))]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test helpers: a GPU setup or readback that fails should stop the test with its message"
)]

use fh6_gpu::GpuContext;
use fh6_kit::fluid::{Fluid, FluidConfig};
use fh6_kit::{noise, vec2, wgpu, Color};
use std::sync::mpsc;
use std::time::Duration;

/// Maps `buffer`, waits for the GPU, and returns a copy of its bytes.
fn read_buffer(ctx: &GpuContext, buffer: &wgpu::Buffer) -> Vec<u8> {
    let (tx, rx) = mpsc::channel();
    buffer.map_async(wgpu::MapMode::Read, .., move |r| {
        let _ = tx.send(r);
    });
    ctx.device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(10)),
        })
        .expect("poll");
    rx.recv_timeout(Duration::from_secs(10))
        .expect("map callback")
        .expect("map ok");
    let bytes = buffer.get_mapped_range(..).expect("mapped range").to_vec();
    buffer.unmap();
    bytes
}

fn f16_to_f32(bits: u16) -> f32 {
    let sign = if bits & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exp = i32::from((bits >> 10) & 0x1f);
    let mant = f32::from(bits & 0x3ff);
    match exp {
        0 => sign * mant * 2f32.powi(-24),
        31 => {
            if mant == 0.0 {
                sign * f32::INFINITY
            } else {
                f32::NAN
            }
        }
        _ => sign * (1.0 + mant / 1024.0) * 2f32.powi(exp - 15),
    }
}

/// Reads the dye back as RGBA f32 texels, row-major.
fn read_dye(ctx: &GpuContext, fluid: &Fluid) -> Vec<[f32; 4]> {
    let (w, h) = fluid.size();
    let row = w * 8;
    let padded = row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("dye readback"),
        size: u64::from(padded * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("dye copy"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: fluid.dye_texture(),
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    ctx.queue.submit(Some(encoder.finish()));
    let bytes = read_buffer(ctx, &buffer);
    let mut out = Vec::with_capacity((w * h) as usize);
    for y in 0..h as usize {
        let line = &bytes[y * padded as usize..][..row as usize];
        for px in line.chunks_exact(8) {
            let c = |i: usize| f16_to_f32(u16::from_le_bytes([px[i], px[i + 1]]));
            out.push([c(0), c(2), c(4), c(6)]);
        }
    }
    out
}

#[test]
fn fluid_splat_spreads_dye_without_validation_errors() {
    let ctx = GpuContext::headless().expect("headless GPU");
    let config = FluidConfig {
        vorticity: 20.0,
        dye_dissipation: 0.0,
        ..FluidConfig::default()
    };
    let mut fluid = Fluid::new(&ctx.device, 128, 64, config);
    assert_eq!(fluid.size(), (128, 64));
    let view_before = format!("{:?}", fluid.dye_view());

    // A rightward jet of orange dye from the left third.
    let orange = Color::linear(1.0, 0.4, 0.05, 1.0);
    for _ in 0..12 {
        assert!(fluid.splat(vec2(0.3, 0.5), vec2(1.5, 0.0), orange, 0.04));
        fluid.step_submit(&ctx.device, &ctx.queue, 1.0 / 60.0);
    }
    for _ in 0..12 {
        fluid.step_submit(&ctx.device, &ctx.queue, 1.0 / 60.0);
    }
    assert_eq!(fluid.pending_splats(), 0);
    assert_eq!(
        format!("{:?}", fluid.dye_view()),
        view_before,
        "dye view must be stable"
    );

    let dye = read_dye(&ctx, &fluid);
    let (w, h) = fluid.size();
    let at = |x: u32, y: u32| dye[(y * w + x) as usize];
    assert!(dye.iter().flatten().all(|v| v.is_finite()), "non-finite dye");
    assert!(
        dye.iter().all(|p| p[3] <= 1.0 + 1e-3 && p[0] <= p[3] * 1.0 + 1e-2),
        "dye must stay premultiplied with alpha in 0..1"
    );

    let splat_px = (w * 3 / 10, h / 2);
    let covered = dye.iter().filter(|p| p[3] > 0.05).count();
    let splat_radius_cells = 0.04 * h as f32;
    let one_splat_area = std::f32::consts::PI * splat_radius_cells * splat_radius_cells;
    assert!(at(splat_px.0, splat_px.1)[3] > 0.0 || covered > 0, "no dye at all");
    assert!(
        covered as f32 > 10.0 * one_splat_area,
        "dye covers {covered} cells, a static splat would cover ~{one_splat_area:.0}"
    );
    // The jet carried dye downstream (right) of the splat point, and not to the far left.
    let downstream: f32 = (splat_px.0 + 6..splat_px.0 + 30).map(|x| at(x, h / 2)[3]).sum();
    let far_left: f32 = (0..4).map(|x| at(x, h / 2)[3]).sum();
    eprintln!("dye: {covered} cells covered (one splat ~{one_splat_area:.0}), downstream {downstream:.2}, far left {far_left:.3}");
    assert!(downstream > 1.0, "downstream dye {downstream}");
    assert!(
        downstream > far_left * 4.0,
        "downstream {downstream} vs far left {far_left}"
    );
    // Colour is preserved (premultiplied orange: r > g > b).
    let c = at(splat_px.0 + 10, h / 2);
    assert!(c[0] > c[1] && c[1] > c[2], "colour {c:?}");

    assert!(!ctx.has_failed(), "wgpu reported a validation error or device loss");
}

#[test]
fn wgsl_noise_matches_cpu() {
    const N: u32 = 256;
    let ctx = GpuContext::headless().expect("headless GPU");
    let source = format!(
        "{}\n{}",
        noise::WGSL_NOISE,
        r"
@group(0) @binding(0) var<storage, read_write> out: array<vec4<f32>>;

fn sample_point(i: u32) -> vec3<f32> {
    let f = f32(i);
    return vec3<f32>(fract(f * 0.618034) * 40.0 - 20.0, fract(f * 0.7548777) * 40.0 - 20.0, fract(f * 0.5698403) * 10.0);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= arrayLength(&out)) {
        return;
    }
    let p = sample_point(i);
    let c = fh6_curl2(p.xy, p.z);
    out[i] = vec4<f32>(fh6_simplex2(p.xy), fh6_simplex3(p), c.x, c.y);
}
"
    );
    let module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("noise parity"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("noise parity"),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let size = u64::from(N) * 16;
    let storage = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("noise out"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("noise readback"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("noise parity"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage.as_entire_binding(),
        }],
    });
    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("noise parity"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("noise parity"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(N.div_ceil(64), 1, 1);
    }
    encoder.copy_buffer_to_buffer(&storage, 0, &readback, 0, size);
    ctx.queue.submit(Some(encoder.finish()));
    let bytes = read_buffer(&ctx, &readback);
    let gpu: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();

    let mut worst = 0.0f32;
    for i in 0..N {
        let f = i as f32;
        let (x, y, z) = (
            (f * 0.618_034).fract() * 40.0 - 20.0,
            (f * 0.754_877_7).fract() * 40.0 - 20.0,
            (f * 0.569_840_3).fract() * 10.0,
        );
        let c = noise::curl2(x, y, z);
        let cpu = [noise::simplex2(x, y), noise::simplex3(x, y, z), c.x, c.y];
        let g = &gpu[(i * 4) as usize..][..4];
        for k in 0..4 {
            let err = (cpu[k] - g[k]).abs() / (1.0 + cpu[k].abs());
            worst = worst.max(err);
            assert!(err < 1e-3, "sample {i} component {k}: cpu {} gpu {}", cpu[k], g[k]);
        }
    }
    eprintln!("noise CPU/GPU worst relative error {worst:e}");
    assert!(!ctx.has_failed(), "wgpu reported a validation error");
}
