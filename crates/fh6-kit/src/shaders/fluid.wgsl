// fh6-kit GPU fluid (crates/fh6-kit/src/fluid.rs): Jos Stam's stable fluids on a small grid,
// compute shaders only, WebGPU core features only.
//
// Every pass shares one bind group layout:
//   0 params (uniform), 1 linear-clamp sampler, 2 src_a, 3 src_b (sampled rgba16float),
//   4 dst (rgba16float storage, write-only).
// The host ping-pongs textures so no pass reads the texture it writes.
//
// Units: positions in grid cells (cell centres at i + 0.5), velocity in cells per second, y down.
// Walls are free-slip: the normal velocity reflects at the border and pressure has a zero
// normal gradient. Dye is linear premultiplied RGBA: alpha = density in 0..1.

struct Splat {
    // xy = centre in cells, z = Gaussian radius in cells, w unused.
    pos_radius: vec4<f32>,
    // xy = velocity added at the centre (cells / s), zw unused.
    force: vec4<f32>,
    // Linear premultiplied dye added at the centre.
    color: vec4<f32>,
};

struct Params {
    size: vec2<f32>,
    inv_size: vec2<f32>,
    dt: f32,
    velocity_dissipation: f32,
    dye_dissipation: f32,
    vorticity: f32,
    pressure_decay: f32,
    splat_count: u32,
    _pad0: f32,
    _pad1: f32,
    splats: array<Splat, 16>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var src_a: texture_2d<f32>;
@group(0) @binding(3) var src_b: texture_2d<f32>;
@group(0) @binding(4) var dst: texture_storage_2d<rgba16float, write>;

fn grid() -> vec2<i32> {
    return vec2<i32>(textureDimensions(dst));
}

fn inside(gid: vec3<u32>) -> bool {
    let d = textureDimensions(dst);
    return gid.x < d.x && gid.y < d.y;
}

fn load_a(c: vec2<i32>) -> vec4<f32> {
    return textureLoad(src_a, clamp(c, vec2<i32>(0), grid() - vec2<i32>(1)), 0);
}

fn load_b(c: vec2<i32>) -> vec4<f32> {
    return textureLoad(src_b, clamp(c, vec2<i32>(0), grid() - vec2<i32>(1)), 0);
}

// Gaussian weight of splat `i` at cell centre `p`.
fn splat_weight(i: u32, p: vec2<f32>) -> f32 {
    let s = params.splats[i];
    let d = p - s.pos_radius.xy;
    let r = max(s.pos_radius.z, 0.5);
    return exp(-dot(d, d) / (r * r));
}

// Keeps dye a valid premultiplied colour with alpha in 0..1.
fn clamp_dye(c_in: vec4<f32>) -> vec4<f32> {
    var c = max(c_in, vec4<f32>(0.0));
    if (c.a > 1.0) {
        c = c / c.a;
    }
    return c;
}

@compute @workgroup_size(8, 8)
fn splat_velocity(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (!inside(gid)) {
        return;
    }
    let c = vec2<i32>(gid.xy);
    let p = vec2<f32>(gid.xy) + vec2<f32>(0.5);
    var v = load_a(c).xy;
    let n = min(params.splat_count, 16u);
    for (var i = 0u; i < n; i = i + 1u) {
        v = v + params.splats[i].force.xy * splat_weight(i, p);
    }
    textureStore(dst, c, vec4<f32>(v, 0.0, 1.0));
}

@compute @workgroup_size(8, 8)
fn splat_dye(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (!inside(gid)) {
        return;
    }
    let c = vec2<i32>(gid.xy);
    let p = vec2<f32>(gid.xy) + vec2<f32>(0.5);
    var d = load_a(c);
    let n = min(params.splat_count, 16u);
    for (var i = 0u; i < n; i = i + 1u) {
        d = d + params.splats[i].color * splat_weight(i, p);
    }
    textureStore(dst, c, clamp_dye(d));
}

// src_a = velocity -> dst.x = curl (d vy / dx - d vx / dy).
@compute @workgroup_size(8, 8)
fn curl(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (!inside(gid)) {
        return;
    }
    let c = vec2<i32>(gid.xy);
    let l = load_a(c - vec2<i32>(1, 0)).y;
    let r = load_a(c + vec2<i32>(1, 0)).y;
    let t = load_a(c - vec2<i32>(0, 1)).x;
    let b = load_a(c + vec2<i32>(0, 1)).x;
    let w = 0.5 * ((r - l) - (b - t));
    textureStore(dst, c, vec4<f32>(w, 0.0, 0.0, 1.0));
}

// src_a = velocity, src_b = curl -> velocity with vorticity confinement applied.
@compute @workgroup_size(8, 8)
fn vorticity(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (!inside(gid)) {
        return;
    }
    let c = vec2<i32>(gid.xy);
    let l = abs(load_b(c - vec2<i32>(1, 0)).x);
    let r = abs(load_b(c + vec2<i32>(1, 0)).x);
    let t = abs(load_b(c - vec2<i32>(0, 1)).x);
    let b = abs(load_b(c + vec2<i32>(0, 1)).x);
    let w = load_b(c).x;
    let g = 0.5 * vec2<f32>(r - l, b - t);
    let n = g / (length(g) + 1.0e-5);
    let force = params.vorticity * vec2<f32>(n.y * w, -n.x * w);
    let v = load_a(c).xy + force * params.dt;
    textureStore(dst, c, vec4<f32>(v, 0.0, 1.0));
}

// src_a = velocity -> dst.x = divergence (free-slip walls).
@compute @workgroup_size(8, 8)
fn divergence(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (!inside(gid)) {
        return;
    }
    let c = vec2<i32>(gid.xy);
    let g = grid();
    let v = load_a(c).xy;
    var l = load_a(c - vec2<i32>(1, 0)).x;
    var r = load_a(c + vec2<i32>(1, 0)).x;
    var t = load_a(c - vec2<i32>(0, 1)).y;
    var b = load_a(c + vec2<i32>(0, 1)).y;
    if (c.x == 0) {
        l = -v.x;
    }
    if (c.x == g.x - 1) {
        r = -v.x;
    }
    if (c.y == 0) {
        t = -v.y;
    }
    if (c.y == g.y - 1) {
        b = -v.y;
    }
    let d = 0.5 * ((r - l) + (b - t));
    textureStore(dst, c, vec4<f32>(d, 0.0, 0.0, 1.0));
}

// src_a = last step's pressure -> scaled warm start for the Jacobi solve.
@compute @workgroup_size(8, 8)
fn pressure_decay(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (!inside(gid)) {
        return;
    }
    let c = vec2<i32>(gid.xy);
    let p = load_a(c).x * params.pressure_decay;
    textureStore(dst, c, vec4<f32>(p, 0.0, 0.0, 1.0));
}

// src_a = pressure, src_b = divergence -> one Jacobi iteration of laplace(p) = div.
@compute @workgroup_size(8, 8)
fn jacobi(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (!inside(gid)) {
        return;
    }
    let c = vec2<i32>(gid.xy);
    let l = load_a(c - vec2<i32>(1, 0)).x;
    let r = load_a(c + vec2<i32>(1, 0)).x;
    let t = load_a(c - vec2<i32>(0, 1)).x;
    let b = load_a(c + vec2<i32>(0, 1)).x;
    let d = load_b(c).x;
    let p = (l + r + t + b - d) * 0.25;
    textureStore(dst, c, vec4<f32>(p, 0.0, 0.0, 1.0));
}

// src_a = velocity, src_b = pressure -> divergence-free velocity.
@compute @workgroup_size(8, 8)
fn gradient_subtract(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (!inside(gid)) {
        return;
    }
    let c = vec2<i32>(gid.xy);
    let l = load_b(c - vec2<i32>(1, 0)).x;
    let r = load_b(c + vec2<i32>(1, 0)).x;
    let t = load_b(c - vec2<i32>(0, 1)).x;
    let b = load_b(c + vec2<i32>(0, 1)).x;
    let v = load_a(c).xy - 0.5 * vec2<f32>(r - l, b - t);
    textureStore(dst, c, vec4<f32>(v, 0.0, 1.0));
}

// Semi-Lagrangian back-trace: the value at cell centre p comes from p - dt * v(p).
fn back_trace(c: vec2<i32>) -> vec2<f32> {
    let p = vec2<f32>(c) + vec2<f32>(0.5);
    let v = load_b(c).xy;
    return (p - params.dt * v) * params.inv_size;
}

// src_a = velocity (field), src_b = velocity (carrier) -> advected velocity.
@compute @workgroup_size(8, 8)
fn advect_velocity(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (!inside(gid)) {
        return;
    }
    let c = vec2<i32>(gid.xy);
    let s = textureSampleLevel(src_a, samp, back_trace(c), 0.0).xy;
    let v = s / (1.0 + params.velocity_dissipation * params.dt);
    textureStore(dst, c, vec4<f32>(v, 0.0, 1.0));
}

// src_a = dye (field), src_b = velocity (carrier) -> advected dye.
@compute @workgroup_size(8, 8)
fn advect_dye(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (!inside(gid)) {
        return;
    }
    let c = vec2<i32>(gid.xy);
    let s = textureSampleLevel(src_a, samp, back_trace(c), 0.0);
    let d = s / (1.0 + params.dye_dissipation * params.dt);
    textureStore(dst, c, clamp_dye(d));
}
