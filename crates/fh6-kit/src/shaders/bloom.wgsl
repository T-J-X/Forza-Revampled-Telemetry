// fh6-kit bloom: the dual-filter (Kawase) down and up chain as compute shaders, so the whole chain
// is one compute pass of nine dispatches instead of nine render passes. A render pass costs the
// host far more CPU to record and encode than a dispatch does (known defect 10).
//
// The maths is that of the former bloom render passes, tap for tap (cs_down is fs_down of
// post.wgsl; cs_up was its fs_up): an invocation is the pixel whose centre a fragment would see
// (id + 0.5), it samples with the same sampler, clamp and level, and stores what the fragment
// would return. The dispatch also covers a margin of two texels past the viewport and stores zero
// there, as the render passes' clear did: a bilinear tap clamped to `src_max` reaches into that
// margin, and without the zeroes it would read what an earlier, larger work rectangle left.

struct Params {
    src_texel: vec2<f32>,
    src_max: vec2<f32>,
    dst_texel: vec2<f32>,
    dir: vec2<f32>,
    sigma: f32,
    intensity: f32,
    src_lod: f32,
    base_lod: f32,
    opacity: f32,
    bloom_on: f32,
    // Bloom dispatches: the viewport size in destination texels (the render passes' viewport).
    origin: vec2<f32>,
};

@group(0) @binding(0) var<uniform> pp: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var base: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;
@group(0) @binding(4) var dst: texture_storage_2d<rgba16float, write>;

fn tap(uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(src, samp, clamp(uv, vec2<f32>(0.0), pp.src_max), pp.src_lod);
}

// True inside the viewport; the margin past it is stored as zero.
fn inside(id: vec3<u32>) -> bool {
    return f32(id.x) < pp.origin.x && f32(id.y) < pp.origin.y;
}

// Dual-filter downsample: centre plus four diagonal half-texel taps (fs_down).
@compute @workgroup_size(8, 8, 1)
fn cs_down(@builtin(global_invocation_id) id: vec3<u32>) {
    if (!inside(id)) {
        textureStore(dst, vec2<i32>(id.xy), vec4<f32>(0.0));
        return;
    }
    let uv = (vec2<f32>(id.xy) + vec2<f32>(0.5)) * pp.dst_texel;
    let o = pp.src_texel;
    var c = tap(uv) * 4.0;
    c = c + tap(uv + vec2<f32>(-o.x, -o.y));
    c = c + tap(uv + vec2<f32>(o.x, -o.y));
    c = c + tap(uv + vec2<f32>(-o.x, o.y));
    c = c + tap(uv + vec2<f32>(o.x, o.y));
    textureStore(dst, vec2<i32>(id.xy), c * 0.125);
}

// Dual-filter upsample (8-tap tent) of `src`, plus `base`, the matching down level.
@compute @workgroup_size(8, 8, 1)
fn cs_up(@builtin(global_invocation_id) id: vec3<u32>) {
    if (!inside(id)) {
        textureStore(dst, vec2<i32>(id.xy), vec4<f32>(0.0));
        return;
    }
    let uv = (vec2<f32>(id.xy) + vec2<f32>(0.5)) * pp.dst_texel;
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
    textureStore(dst, vec2<i32>(id.xy), c / 12.0 + b);
}
