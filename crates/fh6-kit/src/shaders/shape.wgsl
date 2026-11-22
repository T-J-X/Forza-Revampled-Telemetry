// fh6-kit shape pass: every 2D primitive, MSDF text, images, 9-slice and the glass material in
// one instanced pipeline. Output is linear, premultiplied RGBA into an Rgba16Float scene target
// (location 0) plus an emissive target that feeds bloom (location 1).
//
// Distances are signed (negative inside) and in LOCAL units; `xlate.z` converts them to device
// pixels, so every edge gets exactly one pixel of analytic anti-aliasing at any scale. No
// derivatives are used, so the shader is valid in non-uniform control flow (WebGPU rules).

const PI: f32 = 3.14159265;
const TAU: f32 = 6.28318531;

// Shape kinds (shape.rs `kind`).
const K_RECT: u32 = 0u;
const K_CIRCLE: u32 = 1u;
const K_ARC: u32 = 2u;
const K_LINE: u32 = 3u;
const K_POLYLINE: u32 = 4u;
const K_POLYGON: u32 = 5u;
const K_TICKS: u32 = 6u;
const K_CHEVRON: u32 = 7u;
const K_BAR: u32 = 8u;
const K_GLYPH: u32 = 9u;
const K_SEVENSEG: u32 = 10u;
const K_NINESLICE: u32 = 11u;

// Coverage modes (shape.rs `mode`).
const M_FILL: u32 = 0u;
const M_STROKE: u32 = 1u;
const M_GLOW: u32 = 2u;
const M_SHADOW: u32 = 3u;

// Flags (shape.rs `flag`).
const F_STROKE_PX: u32 = 1u;
const F_GLOW_FILL: u32 = 2u;
const F_VERTICAL: u32 = 4u;
const F_CLOSED: u32 = 8u;
const F_ROUND_CAP: u32 = 16u;

// Paint kinds (painter.rs `paint_kind`).
const P_SOLID: u32 = 0u;
const P_LINEAR: u32 = 1u;
const P_RADIAL: u32 = 2u;
const P_CONIC: u32 = 3u;
const P_IMAGE: u32 = 4u;
const P_GLASS: u32 = 5u;

struct Globals {
    target_size: vec2<f32>,
    work_origin: vec2<f32>,
    work_size: vec2<f32>,
    cap_size: vec2<f32>,
    time: f32,
    backdrop_lod: f32,
    backdrop_valid: f32,
    world_valid: f32,
    world_origin: vec2<f32>,
    world_size: vec2<f32>,
};

struct Paint {
    head: vec4<u32>,
    g0: vec4<f32>,
    g1: vec4<f32>,
    g2: vec4<f32>,
    g3: vec4<f32>,
    colors: array<vec4<f32>, 8>,
    offsets: array<vec4<f32>, 2>,
};

struct Clip {
    rect: vec4<f32>,
    params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(1) var<storage, read> paints: array<Paint>;
@group(0) @binding(2) var<storage, read> clips: array<Clip>;
@group(0) @binding(3) var<storage, read> points: array<vec2<f32>>;
@group(0) @binding(4) var atlas: texture_2d<f32>;
@group(0) @binding(5) var backdrop: texture_2d<f32>;
@group(0) @binding(6) var backdrop_blur: texture_2d<f32>;
@group(0) @binding(7) var world_tex: texture_2d<f32>;
@group(0) @binding(8) var samp: sampler;
@group(0) @binding(9) var samp_rep: sampler;
@group(1) @binding(0) var image: texture_2d<f32>;

struct InstanceIn {
    @location(0) xform: vec4<f32>,
    @location(1) xlate: vec4<f32>,
    @location(2) bbox: vec4<f32>,
    @location(3) p0: vec4<f32>,
    @location(4) p1: vec4<f32>,
    @location(5) p2: vec4<f32>,
    @location(6) style: vec4<f32>,
    @location(7) color: vec4<f32>,
    @location(8) ids: vec4<u32>,
};

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) xform: vec4<f32>,
    @location(2) @interpolate(flat) xlate: vec4<f32>,
    @location(3) @interpolate(flat) bbox: vec4<f32>,
    @location(4) @interpolate(flat) p0: vec4<f32>,
    @location(5) @interpolate(flat) p1: vec4<f32>,
    @location(6) @interpolate(flat) p2: vec4<f32>,
    @location(7) @interpolate(flat) style: vec4<f32>,
    @location(8) @interpolate(flat) color: vec4<f32>,
    @location(9) @interpolate(flat) ids: vec4<u32>,
};

struct FsOut {
    @location(0) color: vec4<f32>,
    @location(1) emissive: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, inst: InstanceIn) -> VsOut {
    let corner = vec2<f32>(f32(vi & 1u), f32((vi >> 1u) & 1u));
    let local = mix(inst.bbox.xy, inst.bbox.zw, corner);
    let world = vec2<f32>(
        inst.xform.x * local.x + inst.xform.z * local.y + inst.xlate.x,
        inst.xform.y * local.x + inst.xform.w * local.y + inst.xlate.y,
    );
    let w = world - g.work_origin;
    let ndc = vec2<f32>(w.x / g.work_size.x * 2.0 - 1.0, 1.0 - w.y / g.work_size.y * 2.0);
    var out: VsOut;
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.local = local;
    out.xform = inst.xform;
    out.xlate = inst.xlate;
    out.bbox = inst.bbox;
    out.p0 = inst.p0;
    out.p1 = inst.p1;
    out.p2 = inst.p2;
    out.style = inst.style;
    out.color = inst.color;
    out.ids = inst.ids;
    return out;
}

// ---------------------------------------------------------------- signed distances

fn sd_box(p: vec2<f32>, h: vec2<f32>) -> f32 {
    let q = abs(p) - h;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0);
}

// Rounded box, per-corner radii r = (tl, tr, br, bl), y down.
fn sd_round_box4(p: vec2<f32>, h: vec2<f32>, r4: vec4<f32>) -> f32 {
    var r = select(select(r4.x, r4.w, p.y > 0.0), select(r4.y, r4.z, p.y > 0.0), p.x > 0.0);
    r = min(r, min(h.x, h.y));
    let q = abs(p) - h + vec2<f32>(r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-12), 0.0, 1.0);
    return length(pa - ba * h);
}

// Round-capped tapered segment (iq's round cone in 2D).
fn sd_uneven_capsule(p: vec2<f32>, pa: vec2<f32>, pb: vec2<f32>, ra: f32, rb: f32) -> f32 {
    let ba = pb - pa;
    let l2 = max(dot(ba, ba), 1e-12);
    let rr = ra - rb;
    let a2 = l2 - rr * rr;
    let il2 = 1.0 / l2;
    let pp = p - pa;
    let y = dot(pp, ba);
    let z = y - l2;
    let xv = pp * l2 - ba * y;
    let x2 = dot(xv, xv);
    let y2 = y * y * l2;
    let z2 = z * z * l2;
    let k = sign(rr) * rr * rr * x2;
    if (sign(z) * a2 * z2 > k) {
        return sqrt(x2 + z2) * il2 - rb;
    }
    if (sign(y) * a2 * y2 < k) {
        return sqrt(x2 + y2) * il2 - ra;
    }
    return (sqrt(x2 * a2 * il2) + y * rr) * il2 - ra;
}

// Butt-capped tapered segment (iq's trapezoid; ra, rb are half widths).
fn sd_trapezoid(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, ra: f32, rb: f32) -> f32 {
    let rba = rb - ra;
    let baba = max(dot(b - a, b - a), 1e-12);
    let papa = dot(p - a, p - a);
    let paba = dot(p - a, b - a) / baba;
    let x = sqrt(max(papa - paba * paba * baba, 0.0));
    let cax = max(0.0, x - select(rb, ra, paba < 0.5));
    let cay = abs(paba - 0.5) - 0.5;
    let k = rba * rba + baba;
    let f = clamp((rba * (x - ra) + paba * baba) / k, 0.0, 1.0);
    let cbx = x - ra - f * rba;
    let cby = paba - f;
    let s = select(1.0, -1.0, cbx < 0.0 && cay < 0.0);
    return s * sqrt(min(cax * cax + cay * cay * baba, cbx * cbx + cby * cby * baba));
}

fn wrap_angle(a: f32) -> f32 {
    return a - TAU * floor(a / TAU);
}

fn sd_arc(p: vec2<f32>, c: vec2<f32>, radius: f32, hw: f32, start: f32, sweep: f32, round_cap: bool) -> f32 {
    let q = p - c;
    let ring = abs(length(q) - radius) - hw;
    if (sweep >= TAU - 1e-4) {
        return ring;
    }
    let rel = wrap_angle(atan2(q.y, q.x) - start);
    let e0 = c + radius * vec2<f32>(cos(start), sin(start));
    let a1 = start + sweep;
    let e1 = c + radius * vec2<f32>(cos(a1), sin(a1));
    // Angular distance to each end, for choosing the nearer one.
    let d_start = min(rel, TAU - rel);
    let d_end = min(abs(rel - sweep), TAU - abs(rel - sweep));
    let use_end = d_end < d_start;
    let in_sweep = rel <= sweep;
    if (round_cap) {
        if (in_sweep) {
            return ring;
        }
        return min(length(p - e0), length(p - e1)) - hw;
    }
    // Butt caps: the cap plane of the nearer end, tangent pointing away from the sweep.
    var e = e0;
    var t = -vec2<f32>(-sin(start), cos(start));
    var n = vec2<f32>(cos(start), sin(start));
    if (use_end) {
        e = e1;
        t = vec2<f32>(-sin(a1), cos(a1));
        n = vec2<f32>(cos(a1), sin(a1));
    }
    let u = dot(p - e, n);
    let v = dot(p - e, t);
    if (in_sweep) {
        return max(ring, v);
    }
    let du = abs(u) - hw;
    if (du <= 0.0 && v <= 0.0) {
        return max(du, v);
    }
    return length(max(vec2<f32>(du, v), vec2<f32>(0.0)));
}

fn sd_ticks(p: vec2<f32>, p0: vec4<f32>, p1: vec4<f32>, p2: vec4<f32>) -> f32 {
    let c = p0.xy;
    let r_out = p0.z;
    let start = p1.x;
    let step = p1.y;
    let count = max(p1.z, 1.0);
    let q = p - c;
    var i = 0.0;
    if (abs(step) > 1e-6) {
        var rel = atan2(q.y, q.x) - start;
        rel = rel - TAU * floor((rel + 0.5 * step) / TAU);
        i = clamp(round(rel / step), 0.0, count - 1.0);
    }
    var len = p0.w;
    var hw = p1.w;
    let every = p2.x;
    if (every >= 1.0 && abs(i - every * round(i / every)) < 0.5) {
        len = p2.y;
        hw = p2.z;
    }
    let a = start + i * step;
    let ca = cos(a);
    let sa = sin(a);
    let r = vec2<f32>(q.x * ca + q.y * sa, -q.x * sa + q.y * ca);
    return sd_box(r - vec2<f32>(r_out - len * 0.5, 0.0), vec2<f32>(len * 0.5, hw));
}

fn sd_chevron(p: vec2<f32>, p0: vec4<f32>, p1: vec4<f32>) -> f32 {
    let a = p1.y;
    let q0 = p - p0.xy;
    let ca = cos(a);
    let sa = sin(a);
    let q = vec2<f32>(q0.x * ca + q0.y * sa, abs(-q0.x * sa + q0.y * ca));
    let t = p1.x;
    return sd_segment(q, vec2<f32>(-p0.z + t, p0.w - t), vec2<f32>(p0.z - t, 0.0)) - t;
}

// Returns (distance, lit fraction, cell centre x, cell centre y).
fn sd_bar(p: vec2<f32>, p0: vec4<f32>, p1: vec4<f32>, vertical: bool) -> vec4<f32> {
    let o = p0.xy;
    let s = p0.zw;
    let n = max(p1.x, 1.0);
    let gap = p1.y;
    let lit = p1.z;
    let r = p1.w;
    var along = p.x - o.x;
    var len = s.x;
    if (vertical) {
        along = o.y + s.y - p.y;
        len = s.y;
    }
    let cell = max((len - gap * (n - 1.0)) / n, 1e-3);
    let pitch = cell + gap;
    let base = clamp(floor(along / pitch), 0.0, n - 1.0);
    var best = 1e9;
    var best_i = base;
    for (var k = 0; k < 2; k = k + 1) {
        let i = clamp(base + f32(k), 0.0, n - 1.0);
        var center: vec2<f32>;
        var half: vec2<f32>;
        if (vertical) {
            center = vec2<f32>(o.x + s.x * 0.5, o.y + s.y - (i * pitch + cell * 0.5));
            half = vec2<f32>(s.x * 0.5, cell * 0.5);
        } else {
            center = vec2<f32>(o.x + i * pitch + cell * 0.5, o.y + s.y * 0.5);
            half = vec2<f32>(cell * 0.5, s.y * 0.5);
        }
        let d = sd_round_box4(p - center, half, vec4<f32>(r));
        if (d < best) {
            best = d;
            best_i = i;
        }
    }
    var cc: vec2<f32>;
    if (vertical) {
        cc = vec2<f32>(o.x + s.x * 0.5, o.y + s.y - (best_i * pitch + cell * 0.5));
    } else {
        cc = vec2<f32>(o.x + best_i * pitch + cell * 0.5, o.y + s.y * 0.5);
    }
    return vec4<f32>(best, clamp(lit - best_i, 0.0, 1.0), cc);
}

fn sd_hex_segment(p: vec2<f32>, l: f32, t: f32, vertical: bool) -> f32 {
    var q = abs(p);
    if (vertical) {
        q = q.yx;
    }
    return max(q.y - t, q.x + q.y - l);
}

fn sd_seven(local: vec2<f32>, size: vec2<f32>, thickness: f32, mask: u32) -> f32 {
    let w = size.x;
    let h = size.y;
    let t = clamp(thickness, 0.5, min(w, h) * 0.45);
    let ht = t * 0.5;
    let gap = max(t * 0.12, 0.4);
    let hl = (w - t) * 0.5 - gap + ht;
    let vl = (h * 0.5 - ht) * 0.5 - gap + ht;
    let cx = w * 0.5;
    let upper = (ht + h * 0.5) * 0.5;
    let lower = (h * 0.5 + h - ht) * 0.5;
    var d = 1e6;
    if ((mask & 1u) != 0u) { d = min(d, sd_hex_segment(local - vec2<f32>(cx, ht), hl, ht, false)); }
    if ((mask & 2u) != 0u) { d = min(d, sd_hex_segment(local - vec2<f32>(w - ht, upper), vl, ht, true)); }
    if ((mask & 4u) != 0u) { d = min(d, sd_hex_segment(local - vec2<f32>(w - ht, lower), vl, ht, true)); }
    if ((mask & 8u) != 0u) { d = min(d, sd_hex_segment(local - vec2<f32>(cx, h - ht), hl, ht, false)); }
    if ((mask & 16u) != 0u) { d = min(d, sd_hex_segment(local - vec2<f32>(ht, lower), vl, ht, true)); }
    if ((mask & 32u) != 0u) { d = min(d, sd_hex_segment(local - vec2<f32>(ht, upper), vl, ht, true)); }
    if ((mask & 64u) != 0u) { d = min(d, sd_hex_segment(local - vec2<f32>(cx, h * 0.5), hl, ht, false)); }
    return d;
}

fn sd_polyline(p: vec2<f32>, first: u32, count: u32, closed: bool, hw: f32) -> f32 {
    var d = 1e9;
    let segs = select(count - 1u, count, closed);
    for (var i = 0u; i < segs; i = i + 1u) {
        let a = points[first + i];
        let b = points[first + (i + 1u) % count];
        d = min(d, sd_segment(p, a, b));
    }
    return d - hw;
}

fn sd_polygon(p: vec2<f32>, first: u32, count: u32) -> f32 {
    var d = dot(p - points[first], p - points[first]);
    var s = 1.0;
    var j = count - 1u;
    for (var i = 0u; i < count; i = i + 1u) {
        let vi = points[first + i];
        let vj = points[first + j];
        let e = vj - vi;
        let w = p - vi;
        let b = w - e * clamp(dot(w, e) / max(dot(e, e), 1e-12), 0.0, 1.0);
        d = min(d, dot(b, b));
        let c1 = p.y >= vi.y;
        let c2 = p.y < vj.y;
        let c3 = e.x * w.y > e.y * w.x;
        if ((c1 && c2 && c3) || (!c1 && !c2 && !c3)) {
            s = -s;
        }
        j = i;
    }
    return s * sqrt(d);
}

fn median3(v: vec3<f32>) -> f32 {
    return max(min(v.x, v.y), min(max(v.x, v.y), v.z));
}

// ---------------------------------------------------------------- paint

fn ramp(pa: Paint, t_in: f32) -> vec4<f32> {
    let n = pa.head.y;
    if (n == 0u) {
        return vec4<f32>(0.0);
    }
    var t = t_in;
    let spread = pa.head.z;
    if (spread == 1u) {
        t = fract(t);
    } else if (spread == 2u) {
        let m = t - 2.0 * floor(t * 0.5);
        t = select(m, 2.0 - m, m > 1.0);
    } else {
        t = clamp(t, 0.0, 1.0);
    }
    var c = pa.colors[0];
    var prev_o = pa.offsets[0].x;
    if (t <= prev_o) {
        return c;
    }
    for (var i = 1u; i < n; i = i + 1u) {
        let o = pa.offsets[i / 4u][i % 4u];
        let next = pa.colors[i];
        if (t <= o) {
            let span = o - prev_o;
            let k = select(1.0, (t - prev_o) / span, span > 1e-6);
            return mix(c, next, k);
        }
        c = next;
        prev_o = o;
    }
    return c;
}

fn sample_image_paint(pa: Paint, p: vec2<f32>, px: f32) -> vec4<f32> {
    let rect_min = pa.g0.xy;
    let rect_size = max(pa.g0.zw, vec2<f32>(1e-6));
    var rel = (p - rect_min) / rect_size;
    if (pa.head.z == 1u) {
        rel = fract(rel);
    }
    let uv = pa.g1.xy + rel * pa.g1.zw;
    let tex = pa.g2.xy;
    let texels_per_px = (abs(pa.g1.zw) * tex / rect_size) / max(px, 1e-6);
    let lod = clamp(log2(max(max(texels_per_px.x, texels_per_px.y), 1e-6)), 0.0, max(pa.g2.z - 1.0, 0.0));
    var c: vec4<f32>;
    if (pa.head.z == 1u) {
        c = textureSampleLevel(image, samp_rep, uv, lod);
    } else {
        c = textureSampleLevel(image, samp, uv, lod);
    }
    if (pa.g2.w > 0.5) {
        c = vec4<f32>(c.rgb * c.a, c.a);
    }
    return c * pa.colors[0];
}

fn eval_paint(idx: u32, solid: vec4<f32>, p: vec2<f32>, px: f32) -> vec4<f32> {
    if (idx == 0u) {
        return solid;
    }
    let pa = paints[idx];
    let k = pa.head.x;
    if (k == P_SOLID) {
        return pa.colors[0];
    }
    if (k == P_LINEAR) {
        let a = pa.g0.xy;
        let b = pa.g0.zw;
        let ab = b - a;
        return ramp(pa, dot(p - a, ab) / max(dot(ab, ab), 1e-12));
    }
    if (k == P_RADIAL) {
        return ramp(pa, length(p - pa.g0.xy) / pa.g0.z);
    }
    if (k == P_CONIC) {
        let q = p - pa.g0.xy;
        return ramp(pa, wrap_angle(atan2(q.y, q.x) - pa.g0.z) / TAU);
    }
    if (k == P_IMAGE) {
        return sample_image_paint(pa, p, px);
    }
    return vec4<f32>(0.0);
}

// ---------------------------------------------------------------- glass

fn hash2(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(0.1031, 0.1030));
    let r = q + dot(q, q.yx + 33.33);
    return fract((r.x + r.y) * r.x);
}

fn backdrop_uv(world_px: vec2<f32>) -> vec2<f32> {
    return clamp((world_px - g.work_origin) / g.cap_size, vec2<f32>(0.0), (g.work_size - 0.5) / g.cap_size);
}

fn glass_delta(in: VsOut, p: vec2<f32>, d: f32, frag: vec2<f32>) -> vec4<f32> {
    let pa = paints[in.ids.y];
    let px = in.xlate.z;
    let wp = frag + g.work_origin;
    let c_local = (in.bbox.xy + in.bbox.zw) * 0.5;
    let wc = vec2<f32>(
        in.xform.x * c_local.x + in.xform.z * c_local.y + in.xlate.x,
        in.xform.y * c_local.x + in.xform.w * c_local.y + in.xlate.y,
    );
    let dir = normalize(wp - wc + vec2<f32>(1e-4, 0.0));
    let e_px = max(-d * px, 0.0);
    let frost = pa.g0.x;
    let band = pa.g0.w * px;
    let lens = pa.g0.z * px;
    let k = 1.0 - clamp(e_px / band, 0.0, 1.0);
    let jitter = (vec2<f32>(hash2(wp), hash2(wp + 17.0)) - 0.5) * pa.g0.y * px * frost;
    let sample_px = wp - dir * lens * k * k + jitter;

    var under = vec4<f32>(0.0);
    var here = vec4<f32>(0.0);
    if (g.backdrop_valid > 0.5) {
        let uv = backdrop_uv(sample_px);
        let sharp = textureSampleLevel(backdrop, samp, uv, 0.0);
        let blurred = textureSampleLevel(backdrop_blur, samp, uv, g.backdrop_lod);
        under = mix(sharp, blurred, frost);
        here = textureSampleLevel(backdrop, samp, backdrop_uv(wp), 0.0);
    }
    if (pa.g2.w > 0.5 && g.world_valid > 0.5) {
        let wuv = clamp((sample_px - g.world_origin) / g.world_size, vec2<f32>(0.0), vec2<f32>(1.0));
        let wcol = textureSampleLevel(world_tex, samp, wuv, frost * 3.0);
        under = under + vec4<f32>(wcol.rgb, 1.0) * (1.0 - under.a);
    }
    let tint = pa.colors[0];
    var col = tint + under * (1.0 - tint.a);

    // Fresnel (IOR 1.52, R0 = 0.0426): grazing at the edge.
    let cos_t = clamp(e_px / max(band * 0.5, 1.0), 0.0, 1.0);
    let fres = (0.0426 + 0.9574 * pow(1.0 - cos_t, 5.0)) * pa.g1.x;
    let light = vec2<f32>(cos(pa.g2.y), sin(pa.g2.y));
    let facing = 0.55 + 0.45 * dot(dir, light);
    col = col + vec4<f32>(vec3<f32>(fres * 0.35 * facing), fres * 0.35 * facing);

    // 1 px specular rim, coloured top to bottom.
    let h = max(in.bbox.w - in.bbox.y, 1e-3);
    let ty = clamp((p.y - in.bbox.y) / h, 0.0, 1.0);
    var rim_col = mix(pa.colors[1], pa.colors[2], clamp(ty / 0.4, 0.0, 1.0));
    if (ty > 0.4) {
        rim_col = mix(pa.colors[2], pa.colors[3], (ty - 0.4) / 0.6);
    }
    let rim = clamp(1.0 - abs(e_px - 0.5), 0.0, 1.0) * facing;
    col = col + rim_col * rim;
    let inset = pa.g1.y;
    let inner = clamp(1.0 - abs(e_px - inset - 0.5), 0.0, 1.0);
    col = col + pa.colors[4] * inner;

    // Arched crown sheen under the top edge.
    let pad = 1.5 / px;
    let top = in.bbox.y + pad;
    let hw = max((in.bbox.z - in.bbox.x) * 0.5 - pad, 1e-3);
    let cx = c_local.x + cos(pa.g2.y + 2.356) * hw * 0.15;
    let crest = top + pa.g2.x * ((p.x - cx) / hw) * ((p.x - cx) / hw);
    let sh_h = max((h - 2.0 * pad) * pa.g1.w, 1e-3);
    let st = (p.y - crest) / sh_h;
    var sheen = 0.0;
    if (st >= 0.0 && st <= 1.0) {
        sheen = select(mix(0.342, 0.0, (st - 0.5) * 2.0), mix(1.0, 0.342, st * 2.0), st < 0.5);
    }
    sheen = sheen * pa.g1.z * smoothstep(0.0, 2.0, e_px);
    col = col + vec4<f32>(vec3<f32>(sheen), sheen);

    // Caustic texture, screen-blended.
    if (pa.head.w == 1u) {
        let cuv = (p - in.bbox.xy) / max(in.bbox.zw - in.bbox.xy, vec2<f32>(1e-3));
        let caus = textureSampleLevel(image, samp_rep, cuv, 0.0) * pa.g2.z;
        col = vec4<f32>(col.rgb + caus.rgb * (vec3<f32>(1.0) - min(col.rgb, vec3<f32>(1.0))), col.a);
    }
    return col - here;
}

// ---------------------------------------------------------------- main

fn erf_approx(x: f32) -> f32 {
    // Abramowitz-Stegun 7.1.26 style, max error ~5e-4.
    let s = sign(x);
    let a = abs(x);
    let t = 1.0 / (1.0 + 0.3275911 * a);
    let y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * exp(-a * a);
    return s * y;
}

@fragment
fn fs_main(in: VsOut) -> FsOut {
    let kind = in.ids.x & 0xffu;
    let mode = (in.ids.x >> 8u) & 0xffu;
    let flags = in.ids.x >> 16u;
    let px = max(in.xlate.z, 1e-6);
    var p = in.local;
    if (mode == M_SHADOW) {
        p = p - in.style.xy;
    }

    var d = 1e9;
    var lit = 1.0;
    var paint_pos = in.local;
    var nine_uv = vec2<f32>(-1.0);
    if (kind == K_RECT) {
        d = sd_round_box4(p - in.p0.xy, in.p0.zw, in.p1);
    } else if (kind == K_CIRCLE) {
        d = length(p - in.p0.xy) - in.p0.z;
    } else if (kind == K_ARC) {
        d = sd_arc(p, in.p0.xy, in.p0.z, in.p0.w, in.p1.x, in.p1.y, (flags & F_ROUND_CAP) != 0u);
    } else if (kind == K_LINE) {
        if ((flags & F_ROUND_CAP) != 0u) {
            d = sd_uneven_capsule(p, in.p0.xy, in.p0.zw, in.p1.x, in.p1.y);
        } else {
            d = sd_trapezoid(p, in.p0.xy, in.p0.zw, in.p1.x, in.p1.y);
        }
    } else if (kind == K_POLYLINE) {
        d = sd_polyline(p, u32(in.p0.x), u32(in.p0.y), (flags & F_CLOSED) != 0u, in.p1.x);
    } else if (kind == K_POLYGON) {
        d = sd_polygon(p, u32(in.p0.x), u32(in.p0.y));
    } else if (kind == K_TICKS) {
        d = sd_ticks(p, in.p0, in.p1, in.p2);
    } else if (kind == K_CHEVRON) {
        d = sd_chevron(p, in.p0, in.p1);
    } else if (kind == K_BAR) {
        let b = sd_bar(p, in.p0, in.p1, (flags & F_VERTICAL) != 0u);
        d = b.x;
        lit = b.y;
        paint_pos = b.zw;
    } else if (kind == K_SEVENSEG) {
        d = sd_seven(p - in.p0.xy, in.p0.zw, in.p1.x, u32(in.p1.y + 0.5));
    } else if (kind == K_GLYPH) {
        let r = in.p2;
        let t = (p - r.xy) / max(r.zw - r.xy, vec2<f32>(1e-6));
        let uv = mix(in.p0.xy, in.p0.zw, t);
        let s = textureSampleLevel(atlas, samp, uv, 0.0);
        let inside = all(t >= vec2<f32>(0.0)) && all(t <= vec2<f32>(1.0));
        let dist_texels = select(-0.5 * in.p1.y, (median3(s.rgb) - 0.5) * in.p1.y, inside);
        d = -dist_texels * in.p1.x - in.p1.z * 2.0 * in.p1.x;
    } else if (kind == K_NINESLICE) {
        let o = in.p0.xy;
        let sz = in.p0.zw;
        let l = p - o;
        d = sd_box(l - sz * 0.5, sz * 0.5);
        let ins = in.p1;
        let uvi = in.p2;
        var u: f32;
        if (l.x < ins.x) {
            u = l.x / max(ins.x, 1e-6) * uvi.x;
        } else if (l.x > sz.x - ins.z) {
            u = 1.0 - uvi.z + (l.x - (sz.x - ins.z)) / max(ins.z, 1e-6) * uvi.z;
        } else {
            u = uvi.x + (l.x - ins.x) / max(sz.x - ins.x - ins.z, 1e-6) * (1.0 - uvi.x - uvi.z);
        }
        var v: f32;
        if (l.y < ins.y) {
            v = l.y / max(ins.y, 1e-6) * uvi.y;
        } else if (l.y > sz.y - ins.w) {
            v = 1.0 - uvi.w + (l.y - (sz.y - ins.w)) / max(ins.w, 1e-6) * uvi.w;
        } else {
            v = uvi.y + (l.y - ins.y) / max(sz.y - ins.y - ins.w, 1e-6) * (1.0 - uvi.y - uvi.w);
        }
        nine_uv = clamp(vec2<f32>(u, v), vec2<f32>(0.0), vec2<f32>(1.0));
    }

    // Coverage from distance.
    var cov = 0.0;
    if (mode == M_FILL) {
        cov = clamp(0.5 - d * px, 0.0, 1.0);
    } else if (mode == M_STROKE) {
        var w = in.style.x;
        if ((flags & F_STROKE_PX) != 0u) {
            w = w / px;
        }
        let wpx = w * px;
        let we = max(wpx, 1.0) / px;
        let dd = abs(d - in.style.y * we * 0.5) - we * 0.5;
        cov = clamp(0.5 - dd * px, 0.0, 1.0) * min(wpx, 1.0);
    } else if (mode == M_GLOW) {
        let sigma = max(in.style.x, 1e-4);
        let outside = exp(-0.5 * (max(d, 0.0) / sigma) * (max(d, 0.0) / sigma));
        if ((flags & F_GLOW_FILL) != 0u) {
            cov = in.style.y * outside;
        } else {
            cov = in.style.y * outside * clamp(0.5 + d * px, 0.0, 1.0);
        }
    } else {
        let sigma = max(in.style.z, 1e-4);
        cov = 0.5 * (1.0 - erf_approx((d - in.style.w) / (sigma * 1.41421356)));
    }

    // Clip (target pixels).
    let frag = in.pos.xy;
    let wp = frag + g.work_origin;
    let clip_idx = in.ids.z;
    if (clip_idx != 0u) {
        let c = clips[clip_idx];
        let center = (c.rect.xy + c.rect.zw) * 0.5;
        let half = max((c.rect.zw - c.rect.xy) * 0.5, vec2<f32>(0.0));
        let dc = sd_round_box4(wp - center, half, vec4<f32>(c.params.x));
        cov = cov * clamp(0.5 - dc, 0.0, 1.0);
    }
    cov = cov * in.xlate.w;

    var out: FsOut;
    out.emissive = vec4<f32>(0.0);
    if (cov <= 0.0) {
        out.color = vec4<f32>(0.0);
        return out;
    }

    // Glass writes the change it makes to the scene (blended additively).
    if (in.ids.y != 0u && paints[in.ids.y].head.x == P_GLASS) {
        out.color = glass_delta(in, p, d, frag) * cov;
        return out;
    }

    var color: vec4<f32>;
    if (kind == K_NINESLICE) {
        let pa = paints[in.ids.y];
        let uv = pa.g1.xy + nine_uv * pa.g1.zw;
        var c = textureSampleLevel(image, samp, uv, 0.0);
        if (pa.g2.w > 0.5) {
            c = vec4<f32>(c.rgb * c.a, c.a);
        }
        color = c * pa.colors[0];
    } else {
        color = eval_paint(in.ids.y, in.color, paint_pos, px);
    }
    if (kind == K_BAR) {
        color = mix(in.p2, color, lit);
    }
    color = color * cov;
    out.color = color;

    // Emissive write for bloom layers: intensity and threshold packed as two halves.
    if (in.ids.w != 0u) {
        let bloom = unpack2x16float(in.ids.w);
        let straight = color.rgb / max(color.a, 1e-4);
        let peak = max(straight.r, max(straight.g, straight.b));
        let k = smoothstep(bloom.y - 0.7, bloom.y + 0.7, peak);
        out.emissive = vec4<f32>(color.rgb * bloom.x * k, 0.0);
    }
    return out;
}
