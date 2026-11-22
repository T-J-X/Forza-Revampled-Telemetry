// fh6-kit mesh pass (glTF cars for ADAS views): depth-tested, unlit or matcap-style shading with
// a rim light, drawn into a viewport rectangle of the 2D scene. Output matches the shape pass:
// linear premultiplied colour (location 0) and emissive (location 1).

struct Draw {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    color: vec4<f32>,
    rim: vec4<f32>,
    // Light direction (world, towards the light) and shading mode (0 unlit, 1 matcap).
    light: vec4<f32>,
    // Viewport mapping: NDC of the mesh camera to NDC of the work target: scale.xy, offset.zw.
    ndc_map: vec4<f32>,
    // opacity, bloom intensity, bloom threshold, unused.
    params: vec4<f32>,
    // Camera position (world), unused.
    eye: vec4<f32>,
};

@group(0) @binding(0) var<uniform> d: Draw;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) world: vec3<f32>,
};

@vertex
fn vs_mesh(v: VsIn) -> VsOut {
    let world = d.model * vec4<f32>(v.position, 1.0);
    let clip = d.view_proj * world;
    var out: VsOut;
    // Affine map of NDC applied in clip space keeps perspective-correct interpolation.
    out.pos = vec4<f32>(clip.xy * d.ndc_map.xy + d.ndc_map.zw * clip.w, clip.z, clip.w);
    out.normal = normalize((d.model * vec4<f32>(v.normal, 0.0)).xyz);
    out.world = world.xyz;
    return out;
}

struct FsOut {
    @location(0) color: vec4<f32>,
    @location(1) emissive: vec4<f32>,
};

@fragment
fn fs_mesh(in: VsOut) -> FsOut {
    let n = normalize(in.normal);
    let view_dir = normalize(d.eye.xyz - in.world);
    var c = d.color.rgb;
    if (d.light.w > 0.5) {
        let l = normalize(d.light.xyz);
        let diffuse = max(dot(n, l), 0.0);
        let h = normalize(l + view_dir);
        let spec = pow(max(dot(n, h), 0.0), 48.0);
        c = c * (0.32 + 0.68 * diffuse) + vec3<f32>(spec * 0.35);
    }
    let rim = pow(1.0 - clamp(dot(n, view_dir), 0.0, 1.0), 3.0);
    c = c + d.rim.rgb * rim;
    let a = d.color.a * d.params.x;
    var out: FsOut;
    out.color = vec4<f32>(c * a, a);
    out.emissive = vec4<f32>(0.0);
    if (d.params.y > 0.0) {
        let peak = max(c.r, max(c.g, c.b));
        out.emissive = vec4<f32>(c * a * d.params.y * smoothstep(d.params.z - 0.7, d.params.z + 0.7, peak), 0.0);
    }
    return out;
}
