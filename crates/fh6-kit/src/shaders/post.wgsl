// fh6-kit post passes: dual-filter (Kawase) down/upsampling for bloom, separable Gaussian blur,
// layer compositing and the final resolve to the presentation format.
//
// Every pass draws one full-viewport triangle. Sources are sampled by normalised coordinates of
// their own texture; `src_max` clamps sampling to the valid (work) region so texels outside it,
// left over from earlier frames, never bleed in.

struct Params {
    // 1 / source texture size (of the sampled mip).
    src_texel: vec2<f32>,
    // Largest valid source UV (work region).
    src_max: vec2<f32>,
    // 1 / destination texture size (of the written mip).
    dst_texel: vec2<f32>,
    // Blur direction in source texels (x or y), or (scale, 0) for upsampling.
    dir: vec2<f32>,
    // Gaussian sigma in source texels; bloom intensity; source mip; base mip.
    sigma: f32,
    intensity: f32,
    src_lod: f32,
    base_lod: f32,
    // Resolve: overall opacity, work origin in target px, bloom enabled (and `sigma` is the
    // black underlay alpha).
    opacity: f32,
    bloom_on: f32,
    origin: vec2<f32>,
};

@group(0) @binding(0) var<uniform> pp: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var base: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;

@vertex
fn vs_full(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

fn tap(uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(src, samp, clamp(uv, vec2<f32>(0.0), pp.src_max), pp.src_lod);
}

// Dual-filter downsample: centre plus four diagonal half-texel taps.
@fragment
fn fs_down(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy * pp.dst_texel;
    let o = pp.src_texel;
    var c = tap(uv) * 4.0;
    c = c + tap(uv + vec2<f32>(-o.x, -o.y));
    c = c + tap(uv + vec2<f32>(o.x, -o.y));
    c = c + tap(uv + vec2<f32>(-o.x, o.y));
    c = c + tap(uv + vec2<f32>(o.x, o.y));
    return c * 0.125;
}

// Dual-filter upsample (8-tap tent) of `src`, plus `base` (the matching down level).
@fragment
fn fs_up(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy * pp.dst_texel;
    let o = pp.src_texel;
    var c = tap(uv + vec2<f32>(-2.0 * o.x, 0.0));
    c = c + tap(uv + vec2<f32>(2.0 * o.x, 0.0));
    c = c + tap(uv + vec2<f32>(0.0, -2.0 * o.y));
    c = c + tap(uv + vec2<f32>(0.0, 2.0 * o.y));
    c = c + tap(uv + vec2<f32>(-o.x, -o.y)) * 2.0;
    c = c + tap(uv + vec2<f32>(o.x, -o.y)) * 2.0;
    c = c + tap(uv + vec2<f32>(-o.x, o.y)) * 2.0;
    c = c + tap(uv + vec2<f32>(o.x, o.y)) * 2.0;
    let b = textureSampleLevel(base, samp, clamp(uv, vec2<f32>(0.0), pp.src_max), pp.base_lod);
    return c / 12.0 + b;
}

// Separable Gaussian, 9 bilinear taps covering +-~4 sigma for sigma <= 3 texels.
@fragment
fn fs_blur(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy * pp.dst_texel;
    let sigma = max(pp.sigma, 0.3);
    var sum = vec4<f32>(0.0);
    var wsum = 0.0;
    for (var i = -8; i <= 8; i = i + 1) {
        let x = f32(i) * 0.75;
        let w = exp(-0.5 * (x / sigma) * (x / sigma));
        sum = sum + tap(uv + pp.dir * pp.src_texel * x) * w;
        wsum = wsum + w;
    }
    return sum / wsum;
}

// Composites a (blurred, possibly lower resolution) layer over the scene: premultiplied over,
// done by the pipeline blend state.
@fragment
fn fs_composite(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy * pp.dst_texel;
    return tap(uv) * pp.opacity;
}

fn encode(c: vec3<f32>) -> vec3<f32> {
    let x = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    let lo = x * 12.92;
    let hi = 1.055 * pow(x, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, x <= vec3<f32>(0.0031308));
}

// Resolve the linear premultiplied scene plus bloom into the swapchain's encoding: E(c) * a for
// covered colour (what DWM blends for 8-bit premultiplied content, ADR 0001), and light beyond
// coverage (bloom, glass rims) as additive emission.
@fragment
fn fs_resolve(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let work_px = pos.xy - pp.origin;
    let uv = work_px * pp.src_texel;
    let s = textureSampleLevel(src, samp, clamp(uv, vec2<f32>(0.0), pp.src_max), 0.0);
    var light = s.rgb;
    if (pp.bloom_on > 0.5) {
        let buv = work_px * pp.dst_texel;
        light = light + textureSampleLevel(base, samp, clamp(buv, vec2<f32>(0.0), pp.dir), 0.0).rgb * pp.intensity;
    }
    let a = clamp(s.a, 0.0, 1.0);
    let covered = min(light, vec3<f32>(a));
    let excess = max(light - covered, vec3<f32>(0.0));
    var rgb = vec3<f32>(0.0);
    if (a > 1e-4) {
        rgb = encode(covered / a) * a;
    }
    rgb = rgb + encode(excess) * (1.0 - a);
    let out = vec4<f32>(min(rgb, vec3<f32>(1.0)), a) * pp.opacity;
    // Underlay (`sigma` in the resolve): premultiplied black under everything drawn, so a
    // full-screen dim costs nothing in the shape pass. Black adds no colour, only alpha.
    let u = clamp(pp.sigma, 0.0, 1.0) * pp.opacity;
    return vec4<f32>(out.rgb, out.a + u * (1.0 - out.a));
}
