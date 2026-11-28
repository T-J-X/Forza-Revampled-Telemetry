// fh6-kit pool pass: soft black pools written into the scene before its first layer
// (`Kit::set_pools`, `pool.rs`). The pass draws one full-viewport triangle per pool, scissored to
// that pool's bounds. Every fragment works out all the pools together, so where two pools
// overlap either draw writes the same value: the pass needs no blending and reads nothing.

struct Pool {
    // Centre and half size (ellipse radii or the rectangle's half extents), in target px.
    shape: vec4<f32>,
    // Corner radius, sigma (target px), core alpha, kind (1 ellipse, 2 rounded rectangle).
    style: vec4<f32>,
};

struct Pools {
    // Work origin in target px.
    origin: vec2<f32>,
    count: u32,
    pad: u32,
    pools: array<Pool, 6>,
};

@group(0) @binding(0) var<uniform> pl: Pools;

@vertex
fn vs_full(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

// The same approximation as shape.wgsl (Abramowitz-Stegun 7.1.26 style, max error ~5e-4).
fn erf_approx(x: f32) -> f32 {
    let s = sign(x);
    let a = abs(x);
    let t = 1.0 / (1.0 + 0.3275911 * a);
    let y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * exp(-a * a);
    return s * y;
}

fn sd_round_box(p: vec2<f32>, h: vec2<f32>, radius: f32) -> f32 {
    let r = min(radius, min(h.x, h.y));
    let q = abs(p) - h + vec2<f32>(r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

// One pool's alpha at `p` (target px). The ellipse is the painter's radial gradient with stops
// at 0 (the core alpha), 0.55 (three quarters of it) and 1 (nothing); the rounded rectangle is
// the painter's shadow (`M_SHADOW` in shape.wgsl) with no offset or spread.
fn pool_alpha(pool: Pool, p: vec2<f32>) -> f32 {
    let q = p - pool.shape.xy;
    let alpha = pool.style.z;
    if (pool.style.w < 1.5) {
        let t = length(q / max(pool.shape.zw, vec2<f32>(1e-6)));
        if (t >= 1.0) {
            return 0.0;
        }
        if (t <= 0.55) {
            return alpha * (1.0 - 0.25 * t / 0.55);
        }
        return alpha * 0.75 * (1.0 - t) / 0.45;
    }
    let d = sd_round_box(q, pool.shape.zw, pool.style.x);
    return alpha * 0.5 * (1.0 - erf_approx(d / (max(pool.style.y, 1e-4) * 1.41421356)));
}

@fragment
fn fs_pools(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let p = pos.xy + pl.origin;
    // Each pool lies over the ones before it: what the stack lets through is the product.
    var keep = 1.0;
    for (var i = 0u; i < min(pl.count, 6u); i = i + 1u) {
        keep = keep * (1.0 - clamp(pool_alpha(pl.pools[i], p), 0.0, 1.0));
    }
    // Premultiplied black: no colour, only alpha.
    return vec4<f32>(0.0, 0.0, 0.0, 1.0 - keep);
}
