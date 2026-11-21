// fh6-gpu instanced quad shader: rounded rectangles and seven-segment glyphs.
//
// Colour contract (ADR 0001): instance colours are straight-alpha, sRGB-encoded RGBA (the
// values you would write as a hex code). The fragment output is premultiplied in that
// encoded space, E(c)*a, which is what DWM composites for an 8-bit premultiplied swapchain.

struct Globals {
    // Target size in pixels.
    viewport: vec2<f32>,
    // Overall opacity from the visibility controller (0..1).
    opacity: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct InstanceIn {
    // x, y, w, h in target pixels (top-left origin).
    @location(0) rect: vec4<f32>,
    // Straight-alpha sRGB-encoded colour.
    @location(1) color: vec4<f32>,
    // x: corner radius (kind 0) or segment thickness in px (kind 1)
    // y: kind (0 = rounded rect, 1 = seven-segment glyph)
    // z: segment bit mask (kind 1): a=1 b=2 c=4 d=8 e=16 f=32 g=64
    // w: rim width in px (kind 0, 0 = filled)
    @location(2) params: vec4<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    // Pixel position relative to the rect's top-left corner.
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) size: vec2<f32>,
    @location(2) @interpolate(flat) color: vec4<f32>,
    @location(3) @interpolate(flat) params: vec4<f32>,
};

// One pixel of padding around each rect so the anti-aliased edge is not clipped.
const PAD: f32 = 1.0;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, inst: InstanceIn) -> VsOut {
    // Triangle strip corners: (0,0) (1,0) (0,1) (1,1).
    let corner = vec2<f32>(f32(vi & 1u), f32((vi >> 1u) & 1u));
    let size = max(inst.rect.zw, vec2<f32>(1.0, 1.0));
    let local = corner * (size + 2.0 * PAD) - PAD;
    let pixel = inst.rect.xy + local;
    let ndc = vec2<f32>(pixel.x / globals.viewport.x * 2.0 - 1.0, 1.0 - pixel.y / globals.viewport.y * 2.0);
    var out: VsOut;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.local = local;
    out.size = size;
    out.color = inst.color;
    out.params = inst.params;
    return out;
}

// Signed distance to a rounded box centred at the origin.
fn sd_round_box(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let r = min(radius, min(half_size.x, half_size.y));
    let q = abs(p) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

// Signed distance to a classic hexagonal segment: half length `l`, half thickness `t`.
fn sd_segment(p: vec2<f32>, l: f32, t: f32, vertical: bool) -> f32 {
    var q = abs(p);
    if (vertical) {
        q = q.yx;
    }
    return max(q.y - t, q.x + q.y - l);
}

fn seven_segment(local: vec2<f32>, size: vec2<f32>, thickness: f32, mask: u32) -> f32 {
    let w = size.x;
    let h = size.y;
    let t = clamp(thickness, 1.0, min(w, h) * 0.45);
    let ht = t * 0.5;
    let gap = max(t * 0.12, 0.75);
    // Horizontal segments run between the vertical centre lines.
    let hl = (w - t) * 0.5 - gap + ht;
    // Vertical segments span half the height each.
    let vl = (h * 0.5 - ht) * 0.5 - gap + ht;
    let cx = w * 0.5;
    let upper = (ht + h * 0.5) * 0.5;
    let lower = (h * 0.5 + h - ht) * 0.5;
    var d = 1e6;
    if ((mask & 1u) != 0u) { d = min(d, sd_segment(local - vec2<f32>(cx, ht), hl, ht, false)); }
    if ((mask & 2u) != 0u) { d = min(d, sd_segment(local - vec2<f32>(w - ht, upper), vl, ht, true)); }
    if ((mask & 4u) != 0u) { d = min(d, sd_segment(local - vec2<f32>(w - ht, lower), vl, ht, true)); }
    if ((mask & 8u) != 0u) { d = min(d, sd_segment(local - vec2<f32>(cx, h - ht), hl, ht, false)); }
    if ((mask & 16u) != 0u) { d = min(d, sd_segment(local - vec2<f32>(ht, lower), vl, ht, true)); }
    if ((mask & 32u) != 0u) { d = min(d, sd_segment(local - vec2<f32>(ht, upper), vl, ht, true)); }
    if ((mask & 64u) != 0u) { d = min(d, sd_segment(local - vec2<f32>(cx, h * 0.5), hl, ht, false)); }
    return d;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let kind = u32(in.params.y + 0.5);
    var d: f32;
    if (kind == 1u) {
        d = seven_segment(in.local, in.size, in.params.x, u32(in.params.z + 0.5));
    } else {
        let half_size = in.size * 0.5;
        d = sd_round_box(in.local - half_size, half_size, in.params.x);
        let rim = in.params.w;
        if (rim > 0.0) {
            d = abs(d + rim * 0.5) - rim * 0.5;
        }
    }
    // One-pixel anti-aliased edge (distances are in pixels).
    let coverage = clamp(0.5 - d, 0.0, 1.0);
    let a = in.color.a * coverage * globals.opacity;
    return vec4<f32>(in.color.rgb * a, a);
}
