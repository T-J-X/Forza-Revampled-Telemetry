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
const K_DOME: u32 = 12u;
const K_BLOB: u32 = 13u;

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
const P_OPTIC: u32 = 6u;

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
@group(0) @binding(10) var samp_aniso: sampler;
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

// Superellipse dome (OEM scrim). p0 = centre, semi-axes; p1 = exponents top/bottom, feather
// top/side; p2 = feather bottom, void half width, void top, void feather. First-order signed
// distance r - 1 / |grad r| (mirrors fh6_core::fit::ScrimFit::dome_distance).
fn sd_dome_core(p: vec2<f32>, p0: vec4<f32>, p1: vec4<f32>) -> f32 {
    let c = p0.xy;
    let ab = p0.zw;
    let n = select(p1.y, p1.x, p.y < c.y);
    let uv = max(abs(p - c) / ab, vec2<f32>(1e-6));
    let s = pow(uv.x, n) + pow(uv.y, n);
    let r = pow(s, 1.0 / n);
    let k = pow(r, 1.0 - n);
    let grad = vec2<f32>(k * pow(uv.x, n - 1.0) / ab.x, k * pow(uv.y, n - 1.0) / ab.y);
    return (r - 1.0) / max(length(grad), 1e-6);
}

fn dome_feather(p: vec2<f32>, p0: vec4<f32>, p1: vec4<f32>, p2: vec4<f32>) -> f32 {
    let q = (p - p0.xy) / p0.zw;
    let len = max(length(q), 1e-6);
    let side_w = abs(q.x / len);
    let vertical = select(p2.x, p1.z, q.y < 0.0);
    return p1.w * side_w + vertical * (1.0 - side_w);
}

fn sd_dome_void(p: vec2<f32>, p0: vec4<f32>, p2: vec4<f32>) -> f32 {
    let hw = p2.y;
    if (hw <= 0.0) {
        return 1e9;
    }
    let dx = p.x - p0.x;
    let cap_y = p2.z + hw;
    if (p.y < cap_y) {
        return length(vec2<f32>(dx, p.y - cap_y)) - hw;
    }
    return abs(dx) - hw;
}

fn median3(v: vec3<f32>) -> f32 {
    return max(min(v.x, v.y), min(max(v.x, v.y), v.z));
}

// ---------------------------------------------------------------- liquid groups (K_BLOB)
//
// A blob's pieces live in `points`, six records each (shape.rs `encode_piece`): (kind, radius),
// three outline vectors, (thickness, bevel), (base, 0).

fn sd_piece(p: vec2<f32>, i: u32) -> f32 {
    let head = points[i];
    let v1 = points[i + 1u];
    let v2 = points[i + 2u];
    let v3 = points[i + 3u];
    let k = u32(head.x + 0.5);
    if (k == 0u) {
        return length(p - v1) - head.y;
    }
    if (k == 1u) {
        return sd_round_box4(p - v1, v2, vec4<f32>(head.y));
    }
    if (k == 2u) {
        return sd_uneven_capsule(p, v1, v2, v3.x, v3.y);
    }
    return sd_arc(p, v1, v2.x, v2.y, v3.x, v3.y, true);
}

// The merged field of a blob at p: (distance, thickness, bevel, base). The distance is the
// smooth union min(a, b) - k ln(1 + exp(-|a - b| / k)) of the pieces (optics.md section 2); the
// cross-section is blended with the same weights, so it changes smoothly where pieces bridge.
fn blob_field(p: vec2<f32>, first: u32, count: u32, k: f32) -> vec4<f32> {
    var acc = vec4<f32>(1e9, 0.0, 1.0, 0.0);
    for (var j = 0u; j < count; j = j + 1u) {
        let i = first + j * 6u;
        let d = sd_piece(p, i);
        let sec = vec3<f32>(points[i + 4u], points[i + 5u].x);
        if (j == 0u) {
            acc = vec4<f32>(d, sec);
        } else if (k <= 1e-3) {
            if (d < acc.x) {
                acc = vec4<f32>(d, sec);
            }
        } else {
            let e = exp(-abs(d - acc.x) / k);
            let nearer = 1.0 / (1.0 + e);
            let w_new = select(1.0 - nearer, nearer, d < acc.x);
            acc = vec4<f32>(min(d, acc.x) - k * log(1.0 + e), mix(acc.yzw, sec, w_new));
        }
    }
    return acc;
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

// A glass instance's own bounds (local, padded for anti-aliasing). Its optics come from these,
// never from the quad, which the painter trims to a clip. Glass paints have no solid colour, so the
// painter stores the bounds in the instance's colour lane.
fn glass_frame(in: VsOut) -> vec4<f32> {
    return in.color;
}

fn glass_delta(in: VsOut, p: vec2<f32>, d: f32, frag: vec2<f32>) -> vec4<f32> {
    let pa = paints[in.ids.y];
    let fr = glass_frame(in);
    let px = in.xlate.z;
    let wp = frag + g.work_origin;
    let c_local = (fr.xy + fr.zw) * 0.5;
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
        var blurred = sharp;
        if (g.backdrop_lod >= 0.0) {
            blurred = textureSampleLevel(backdrop_blur, samp, uv, g.backdrop_lod);
        }
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
    let h = max(fr.w - fr.y, 1e-3);
    let ty = clamp((p.y - fr.y) / h, 0.0, 1.0);
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
    let top = fr.y + pad;
    let hw = max((fr.z - fr.x) * 0.5 - pad, 1e-3);
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
        let cuv = (p - fr.xy) / max(fr.zw - fr.xy, vec2<f32>(1e-3));
        let caus = textureSampleLevel(image, samp_rep, cuv, 0.0) * pa.g2.z;
        col = vec4<f32>(col.rgb + caus.rgb * (vec3<f32>(1.0) - min(col.rgb, vec3<f32>(1.0))), col.a);
    }
    return col - here;
}

// ---------------------------------------------------------------- optical glass
//
// Every step is an equation of docs/specs/optics.md; crates/fh6-kit/src/optics.rs is the CPU
// reference with the unit tests, and tests/glass_gpu.rs compares this shader with it.

// Distance of the kinds whose outline is a pure function of p (glass needs it at five points).
fn sd_basic(in: VsOut, kind: u32, flags: u32, p: vec2<f32>) -> f32 {
    if (kind == K_RECT) {
        return sd_round_box4(p - in.p0.xy, in.p0.zw, in.p1);
    }
    if (kind == K_CIRCLE) {
        return length(p - in.p0.xy) - in.p0.z;
    }
    if (kind == K_ARC) {
        return sd_arc(p, in.p0.xy, in.p0.z, in.p0.w, in.p1.x, in.p1.y, (flags & F_ROUND_CAP) != 0u);
    }
    if (kind == K_LINE) {
        if ((flags & F_ROUND_CAP) != 0u) {
            return sd_uneven_capsule(p, in.p0.xy, in.p0.zw, in.p1.x, in.p1.y);
        }
        return sd_trapezoid(p, in.p0.xy, in.p0.zw, in.p1.x, in.p1.y);
    }
    if (kind == K_DOME) {
        return max(sd_dome_core(p, in.p0, in.p1), -sd_dome_void(p, in.p0, in.p2));
    }
    if (kind == K_POLYGON) {
        return sd_polygon(p, u32(in.p0.x), u32(in.p0.y));
    }
    if (kind == K_POLYLINE) {
        return sd_polyline(p, u32(in.p0.x), u32(in.p0.y), (flags & F_CLOSED) != 0u, in.p1.x);
    }
    let fr = glass_frame(in);
    return sd_box(p - (fr.xy + fr.zw) * 0.5, (fr.zw - fr.xy) * 0.5);
}

// (distance, thickness, bevel, base) of a glass shape at p: a blob's pieces carry their own
// cross-sections, every other shape takes the paint's.
fn glass_field(in: VsOut, kind: u32, flags: u32, p: vec2<f32>, section: vec3<f32>) -> vec4<f32> {
    if (kind == K_BLOB) {
        return blob_field(p, u32(in.p0.x + 0.5), u32(in.p0.y + 0.5), in.p0.z);
    }
    return vec4<f32>(sd_basic(in, kind, flags, p), section);
}

// h^k of the profile h = H (1 - (1 - u)^k)^(1/k), u = clamp(-d / w, 0, 1) (section 2). h has an
// infinite slope at the rim; h^k is smooth there, so its finite differences are accurate.
fn height_pow(f: vec4<f32>, k: f32, grown: f32) -> f32 {
    let u = clamp(-f.x / max(f.z, 1e-3), 0.0, 1.0);
    return pow(max(f.y * grown, 0.0), k) * (1.0 - pow(1.0 - u, k));
}

// Exact Fresnel reflectance of a dielectric for unpolarised light, per channel (section 3).
fn fresnel3(cos_i: f32, eta: vec3<f32>) -> vec3<f32> {
    let ci = clamp(cos_i, 0.0, 1.0);
    let ct = sqrt(max(vec3<f32>(1.0) - vec3<f32>(1.0 - ci * ci) / (eta * eta), vec3<f32>(0.0)));
    let r_par = (eta * ci - ct) / (eta * ci + ct);
    let r_perp = (vec3<f32>(ci) - eta * ct) / (vec3<f32>(ci) + eta * ct);
    return 0.5 * (r_par * r_par + r_perp * r_perp);
}

// A soft-edged rectangular light centred on direction c, with tangent axes tx and ty. `half`
// is its half size in tangent units (the tangents of its half angles).
fn softbox(e: vec3<f32>, c: vec3<f32>, tx: vec3<f32>, ty: vec3<f32>, half: vec2<f32>, soft: f32) -> f32 {
    let cz = dot(e, c);
    if (cz <= 0.02) {
        return 0.0;
    }
    let q = vec2<f32>(dot(e, tx), dot(e, ty)) / cz;
    return 1.0 - smoothstep(-soft, soft, sd_box(q, half));
}

// The analytic studio, in the environment frame (x right, y down, z at the viewer):
// - a horizon ring: a band of light from the plane of the screen to 9 deg above it, all the
//   way round, brightest towards the key (azimuth -135 deg) and, cooler, opposite it. A rounded
//   edge mirrors it as one thin line along the outline: the white line of dark-field glass
//   photography;
// - a key strip up and to the left (azimuth -135 deg, 40 deg above the screen, 74 x 9 deg);
// - a long thin strip opposite and lower (azimuth 45 deg, 25 deg up, 105 x 7 deg);
// - a faint sky from above.
// `spread` is the width of one pixel's footprint on the environment (radians): the lights'
// edges are softened by it, which is the pixel filter applied to the reflection.
fn studio(e: vec3<f32>, spread: f32) -> vec3<f32> {
    let soft = 0.5 * spread;
    let key = softbox(
        e,
        vec3<f32>(-0.5417, -0.5417, 0.6428),
        vec3<f32>(0.7071, -0.7071, 0.0),
        vec3<f32>(0.4545, 0.4545, 0.7660),
        vec2<f32>(0.75, 0.08),
        0.02 + soft,
    );
    let strip = softbox(
        e,
        vec3<f32>(0.6409, 0.6409, 0.4226),
        vec3<f32>(-0.7071, 0.7071, 0.0),
        vec3<f32>(-0.2988, -0.2988, 0.9063),
        vec2<f32>(1.30, 0.06),
        0.02 + soft,
    );
    let sky = 0.10 * smoothstep(0.0, 1.0, -e.y) * smoothstep(-0.1, 0.4, e.z);
    let band = smoothstep(-0.01 - soft, 0.01 + soft, e.z)
        * (1.0 - smoothstep(0.1464 - soft, 0.1664 + soft, e.z));
    let side = e.xy / max(length(e.xy), 1e-4);
    let to_key = pow(max(dot(side, vec2<f32>(-0.7071, -0.7071)), 0.0), 6.0);
    let to_far = pow(max(dot(side, vec2<f32>(0.7071, 0.7071)), 0.0), 6.0);
    let ring = band * (vec3<f32>(0.9) + vec3<f32>(5.0) * to_key + vec3<f32>(1.98, 2.2, 2.46) * to_far);
    return vec3<f32>(8.0) * key + vec3<f32>(4.2, 4.7, 5.2) * strip + vec3<f32>(sky) + ring;
}

// Octahedral layout of the sphere: the hemisphere facing the viewer fills the inner diamond.
fn oct_uv(d: vec3<f32>) -> vec2<f32> {
    let n = d / max(abs(d.x) + abs(d.y) + abs(d.z), 1e-6);
    var o = n.xy;
    if (n.z < 0.0) {
        let s = vec2<f32>(select(-1.0, 1.0, n.x >= 0.0), select(-1.0, 1.0, n.y >= 0.0));
        o = (vec2<f32>(1.0) - abs(n.yx)) * s;
    }
    return o * 0.5 + vec2<f32>(0.5);
}

fn world_uv(world_px: vec2<f32>) -> vec2<f32> {
    return clamp((world_px - g.world_origin) / g.world_size, vec2<f32>(0.0), vec2<f32>(1.0));
}

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
    let x = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(1.055 * pow(x, vec3<f32>(1.0 / 2.4)) - 0.055, x * 12.92, x <= vec3<f32>(0.0031308));
}

fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
    let x = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(pow((x + 0.055) / 1.055, vec3<f32>(2.4)), x / 12.92, x <= vec3<f32>(0.04045));
}

fn enc1(c: f32) -> f32 {
    let x = clamp(c, 0.0, 1.0);
    return select(1.055 * pow(x, 1.0 / 2.4) - 0.055, x * 12.92, x <= 0.0031308);
}

fn dec1(c: f32) -> f32 {
    let x = clamp(c, 0.0, 1.0);
    return select(pow((x + 0.055) / 1.055, 2.4), x / 12.92, x <= 0.04045);
}

// The linear light one channel of a scene value shows over black once resolved (mirrors
// fs_resolve in post.wgsl: colour up to the alpha is encoded as coverage, light beyond it as
// emission). With no coverage or full coverage the two encodings cancel.
fn shown1(c: f32, alpha: f32) -> f32 {
    let a = clamp(alpha, 0.0, 1.0);
    if (a < 1e-4 || a > 0.9999) {
        return clamp(c, 0.0, 1.0);
    }
    let covered = min(c, a);
    return dec1(enc1(max(c - covered, 0.0)) * (1.0 - a) + enc1(covered / a) * a);
}

fn shown(s: vec4<f32>) -> vec3<f32> {
    return vec3<f32>(shown1(s.r, s.a), shown1(s.g, s.a), shown1(s.b, s.a));
}

// The scene value that shows linear light `l` over black at alpha `a` (the inverse of `shown`).
fn unshown(l: vec3<f32>, a: f32) -> vec3<f32> {
    if (a > 0.9999) {
        return clamp(l, vec3<f32>(0.0), vec3<f32>(1.0));
    }
    let t = srgb_encode(l);
    let cover = t <= vec3<f32>(a);
    let dec = srgb_decode(select((t - vec3<f32>(a)) / max(1.0 - a, 1e-4), t / max(a, 1e-4), cover));
    return select(vec3<f32>(a) + dec, dec * a, cover);
}

fn optic_glass(in: VsOut, p: vec2<f32>, frag: vec2<f32>, kind: u32, flags: u32) -> vec4<f32> {
    let pa = paints[in.ids.y];
    let px = max(in.xlate.z, 1e-6);
    let wp = frag + g.work_origin;
    let section = pa.g1.xyz;
    let k = clamp(pa.g1.w, 1.0, 8.0);
    let grown = pa.colors[5].x;

    // Section 2: the surface height and its normal, from the outline's distance field.
    let one_px = 1.0 / px;
    let ex = vec2<f32>(one_px, 0.0);
    let ey = vec2<f32>(0.0, one_px);
    let f0 = glass_field(in, kind, flags, p, section);
    // Further in than the rounded edge the surface is level, so its neighbours are not needed.
    var fxp = f0;
    var fxm = f0;
    var fyp = f0;
    var fym = f0;
    if (-f0.x < f0.z + 2.0 * one_px) {
        fxp = glass_field(in, kind, flags, p + ex, section);
        fxm = glass_field(in, kind, flags, p - ex, section);
        fyp = glass_field(in, kind, flags, p + ey, section);
        fym = glass_field(in, kind, flags, p - ey, section);
    }
    let hk = height_pow(f0, k, grown);
    let gq = vec2<f32>(
        height_pow(fxp, k, grown) - height_pow(fxm, k, grown),
        height_pow(fyp, k, grown) - height_pow(fym, k, grown),
    ) / (2.0 * one_px);
    let h = pow(hk, 1.0 / k);
    // n is proportional to (-grad h, 1) = (-grad(h^k), k h^(k-1)); the second form has no
    // division, so it reaches the horizontal rim normal without a singularity.
    let rot = in.xform / px;
    var slope = vec2<f32>(rot.x * gq.x + rot.z * gq.y, rot.y * gq.x + rot.w * gq.y);
    let up = k * pow(max(h, 1e-5), k - 1.0);
    // Section 2, surface waves: a ring packet eta(r) = A e^(-x^2) cos(2 pi x), x = (r - R) / L,
    // adds its own slope. With q = h^k the normal is (-grad q - U grad eta, U), U = k h^(k-1).
    var waved = false;
    let wave_a = pa.colors[2].w * grown;
    let wave_l = pa.colors[3].w;
    if (abs(wave_a) > 1e-4 && wave_l > 1e-3) {
        let rel = p - pa.colors[5].yz;
        let rr = max(length(rel), 1e-4);
        let x = (rr - pa.colors[5].w) / wave_l;
        if (abs(x) < 3.0) {
            let d_eta = wave_a * exp(-x * x) * (-2.0 * x * cos(TAU * x) - TAU * sin(TAU * x)) / wave_l;
            let ge = rel / rr * d_eta;
            slope = slope + up * vec2<f32>(rot.x * ge.x + rot.z * ge.y, rot.y * ge.x + rot.w * ge.y);
            waved = true;
        }
    }
    var n = vec3<f32>(0.0, 0.0, 1.0);
    let n_len = length(vec3<f32>(slope, up));
    if (n_len > 1e-12) {
        n = vec3<f32>(-slope, up) / n_len;
    }

    // Section 7, pixel footprint: how far the normal turns across one device pixel. Along the slope it turns
    // at the profile's curvature, d(theta)/ds = (G' U - G U') / (G^2 + U^2) with
    // G = |grad(h^k)| and U = k h^(k-1); across the slope it turns with the outline, whose
    // curvature is the Laplacian of the distance. dn_s and dn_c are those two changes of n.
    let big_g = length(gq);
    var dn_s = vec3<f32>(0.0);
    var dn_c = vec3<f32>(0.0);
    if (big_g > 1e-9 && n_len > 1e-12) {
        let w = max(f0.z, 1e-3);
        let tt = clamp(1.0 + f0.x / w, 1e-4, 1.0);
        let g_s = -pow(max(f0.y * grown, 0.0), k) * k * (k - 1.0) * pow(tt, k - 2.0) / (w * w);
        let u_s = (k - 1.0) * big_g / max(h, 1e-5);
        let theta_s = (g_s * up - big_g * u_s) / (n_len * n_len) * one_px;
        let kappa = (fxp.x + fxm.x + fyp.x + fym.x - 4.0 * f0.x) / one_px;
        let sin_t = big_g / n_len;
        let downhill = -vec2<f32>(rot.x * gq.x + rot.z * gq.y, rot.y * gq.x + rot.w * gq.y) / big_g;
        dn_s = theta_s * vec3<f32>(downhill * n.z, -sin_t);
        dn_c = kappa * sin_t * vec3<f32>(-downhill.y, downhill.x, 0.0);
    }

    // Sections 3 to 6: Fresnel, Snell and Beer-Lambert, one index per channel.
    let eta = pa.g0.xyz;
    let ci = clamp(n.z, 0.0, 1.0);
    let fres = fresnel3(ci, eta);
    let ct = sqrt(max(vec3<f32>(1.0) - vec3<f32>(1.0 - ci * ci) / (eta * eta), vec3<f32>(0.0)));
    let bend = vec3<f32>(ci) / eta - ct;
    let wz = -vec3<f32>(1.0) / eta + bend * n.z;
    let depth = h + f0.w * grown;
    let path = vec3<f32>(depth) / max(-wz, vec3<f32>(1e-3));
    let reach = bend * path * px;
    let keep = exp(-pa.g2.xyz * path);

    // What lies under the piece, seen along the refracted ray of each channel, as the linear
    // light it shows on screen. `veil` is how much of the game the kit's own layers cover there.
    let frost = pa.g0.w;
    var under = vec3<f32>(0.0);
    var veil = 0.0;
    var here = vec4<f32>(0.0);
    if (g.backdrop_valid > 0.5) {
        let ur = backdrop_uv(wp + n.xy * reach.x);
        let ug = backdrop_uv(wp + n.xy * reach.y);
        let ub = backdrop_uv(wp + n.xy * reach.z);
        var sg = textureSampleLevel(backdrop, samp, ug, 0.0);
        var sr = sg;
        var sb = sg;
        if (big_g > 1e-9 || waved) {
            // Only a sloped surface sends the three colours to different places.
            sr = textureSampleLevel(backdrop, samp, ur, 0.0);
            sb = textureSampleLevel(backdrop, samp, ub, 0.0);
        }
        if (frost > 0.001 && g.backdrop_lod >= 0.0) {
            sr = mix(sr, textureSampleLevel(backdrop_blur, samp, ur, g.backdrop_lod), frost);
            sg = mix(sg, textureSampleLevel(backdrop_blur, samp, ug, g.backdrop_lod), frost);
            sb = mix(sb, textureSampleLevel(backdrop_blur, samp, ub, g.backdrop_lod), frost);
        }
        under = vec3<f32>(shown1(sr.r, sr.a), shown1(sg.g, sg.a), shown1(sb.b, sb.a));
        veil = clamp(sg.a, 0.0, 1.0);
        here = textureSampleLevel(backdrop, samp, backdrop_uv(wp), 0.0);
    }
    if (pa.colors[0].w > 0.5 && g.world_valid > 0.5) {
        // The captured game frame, bent like everything else. The compositor blends in encoded
        // space, so the layers above it pass the decoded value of their encoded transmission.
        let lod = frost * 3.0;
        let world = vec3<f32>(
            textureSampleLevel(world_tex, samp, world_uv(wp + n.xy * reach.x), lod).r,
            textureSampleLevel(world_tex, samp, world_uv(wp + n.xy * reach.y), lod).g,
            textureSampleLevel(world_tex, samp, world_uv(wp + n.xy * reach.z), lod).b,
        );
        under = under + world * srgb_decode(vec3<f32>(1.0 - veil));
        veil = 1.0;
    }

    // Section 7: the mirrored environment. Below the horizon the surface mirrors the plane it
    // sits on, so a rim picks up the light of its neighbours.
    let r = vec3<f32>(2.0 * n.z * n.xy, 2.0 * n.z * n.z - 1.0);
    let e = vec3<f32>(dot(pa.colors[2].xyz, r), dot(pa.colors[3].xyz, r), dot(pa.colors[4].xyz, r));
    // Section 7, pixel footprint: its extent on the environment, from r = 2 (n.v) n - v.
    let dr_s = vec3<f32>(2.0 * (dn_s.z * n.xy + n.z * dn_s.xy), 4.0 * n.z * dn_s.z);
    let dr_c = vec3<f32>(2.0 * (dn_c.z * n.xy + n.z * dn_c.xy), 4.0 * n.z * dn_c.z);
    var de_s = vec3<f32>(dot(pa.colors[2].xyz, dr_s), dot(pa.colors[3].xyz, dr_s), dot(pa.colors[4].xyz, dr_s));
    var de_c = vec3<f32>(dot(pa.colors[2].xyz, dr_c), dot(pa.colors[3].xyz, dr_c), dot(pa.colors[4].xyz, dr_c));
    de_s = de_s * min(1.0, 0.5 / max(length(de_s), 1e-6));
    de_c = de_c * min(1.0, 0.5 / max(length(de_c), 1e-6));
    let env_kind = pa.g3.w;
    var env = pa.colors[1].xyz;
    if (env_kind < 0.5) {
        env = env * studio(e, max(length(de_s), length(de_c)));
    } else if (env_kind > 1.5) {
        let uv_e = oct_uv(e);
        env = env * textureSampleGrad(image, samp_aniso, uv_e, oct_uv(e + de_s) - uv_e, oct_uv(e + de_c) - uv_e).rgb;
    }
    if (abs(env_kind - 1.0) > 0.5) {
        let below = smoothstep(0.0, 0.05, -r.z);
        var ground = vec3<f32>(0.0);
        if (below > 0.0 && g.backdrop_valid > 0.5) {
            let far = depth / max(-r.z, 0.25);
            ground = shown(textureSampleLevel(backdrop, samp, backdrop_uv(wp + r.xy * far * px), 0.0));
        }
        env = mix(env, ground, below);
    }

    // Section 7: the key light's GGX lobe, D G F / (4 (n.l)(n.v)) times E (n.l).
    let l = pa.g3.xyz;
    let nl = dot(n, l);
    var spec = vec3<f32>(0.0);
    if (nl > 1e-3 && ci > 1e-3) {
        let hv = normalize(l + vec3<f32>(0.0, 0.0, 1.0001));
        let nh = max(dot(n, hv), 0.0);
        // Section 7, pixel footprint: the lobe is widened by the spread of normals inside the
        // pixel, alpha^2 + min(2 sigma^2 (|dn/dx|^2 + |dn/dy|^2), 0.18) with sigma^2 = 1 / (2 pi).
        let a2 = min(pa.g2.w * pa.g2.w + min(0.3183099 * (dot(dn_s, dn_s) + dot(dn_c, dn_c)), 0.18), 1.0);
        let dd = nh * nh * (a2 - 1.0) + 1.0;
        let lam_l = 0.5 * (sqrt(1.0 + a2 * (1.0 - nl * nl) / (nl * nl)) - 1.0);
        let lam_v = 0.5 * (sqrt(1.0 + a2 * (1.0 - ci * ci) / (ci * ci)) - 1.0);
        let dg = a2 / (PI * dd * dd) / (1.0 + lam_l + lam_v);
        spec = fresnel3(hv.z, eta) * pa.colors[0].xyz * (dg / (4.0 * ci));
    }

    // Section 8: reflected plus transmitted, in linear light. The piece hides what it reflects
    // or absorbs: the compositor blends in encoded space, so the game behind is attenuated by
    // the encoded value of the glass's linear transmission.
    let passed = (vec3<f32>(1.0) - fres) * keep;
    let light = fres * env + spec + passed * under;
    let through = srgb_encode(vec3<f32>((passed.x + passed.y + passed.z) / 3.0)).x;
    let alpha = 1.0 - through * (1.0 - veil);
    return vec4<f32>(unshown(light, alpha), alpha) - here;
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
    } else if (kind == K_DOME) {
        d = max(sd_dome_core(p, in.p0, in.p1), -sd_dome_void(p, in.p0, in.p2));
    } else if (kind == K_BLOB) {
        d = blob_field(p, u32(in.p0.x + 0.5), u32(in.p0.y + 0.5), in.p0.z).x;
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
    if (mode == M_FILL && kind == K_DOME) {
        // Feathered dome: opaque core, smoothstep fall-off across the feather (at least one
        // device pixel), cut by the void, with +-0.5/255 triangular dither against banding.
        let fw = max(dome_feather(p, in.p0, in.p1, in.p2), 1.0 / px);
        let core = 1.0 - smoothstep(0.0, fw, sd_dome_core(p, in.p0, in.p1));
        let vf = max(in.p2.w, 1.0 / px);
        let open = smoothstep(-0.5 * vf, 0.5 * vf, sd_dome_void(p, in.p0, in.p2));
        let wpd = in.pos.xy + g.work_origin;
        let dither = (hash2(wpd) + hash2(wpd + 37.0) - 1.0) / 255.0;
        cov = core * open;
        if (cov > 0.0 && cov < 1.0) {
            cov = clamp(cov + dither, 0.0, 1.0);
        }
    } else if (mode == M_FILL) {
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
    if (in.ids.y != 0u && paints[in.ids.y].head.x == P_OPTIC) {
        out.color = optic_glass(in, p, frag, kind, flags) * cov;
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
