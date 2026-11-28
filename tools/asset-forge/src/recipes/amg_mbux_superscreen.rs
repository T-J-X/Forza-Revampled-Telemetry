//! AmgMbuxSuperscreen's bespoke assets (`assets/clusters/amg_mbux_superscreen/`, ledger
//! `assets/clusters/amg_mbux_superscreen/ASSETS.md`). Since TJS's note of 2026-11-28 ("can be
//! outside of a slab ... what MBUX is but better") the dash has no housing: every part floats on
//! OLED black as its own rendered object.
//!
//! - **Halftone dots (ComfyUI, SDXL + 4x-UltraSharp):** a macro of a micro-perforated grille,
//!   made grey, seamless and high-passed, then used to modulate an exact hexagonal dot grid laid
//!   out in Rust (SDXL draws organic variation well but not a regular screen). It is the dotted
//!   halftone of the real W214 Sporty display, on the tunnel wall, the disc and the crescents.
//! - **Brushed metal (ComfyUI):** a macro of brushed anodised aluminium, grey and seamless; it
//!   drives the roughness of the tunnel's ring lines and the scale bands' rims.
//! - **Assembly (Blender, Cycles):** board v3 (measured from the W214 photos after TJS's note of
//!   07:41, "It looks nothing like how it should"), modelled at 1 Blender unit per 4K px: a
//!   shallow near-black funnel from the crescents' inner edge to the disc with a faint halftone
//!   towards the light, five thin satin-grey ring lines, the light at the end of the tunnel (a
//!   flat dotted disc cut at the horizon, brightest where it meets the road, one darker ring
//!   inside), a black veil below the horizon, a dark road lit from the horizon with fading edge
//!   and lane lines, the bright horizon lip and a dim cross line, and two dark plum-charcoal scale
//!   bands with a polished rim on the outer edge and a dim lip on the inner one. Lit by a dim
//!   studio HDRI and one key 38° up, rendered once per key azimuth (rest −115°, swung −65°); the
//!   cluster cross-fades the two with lateral G.
//! - **Crescents (Blender):** both crescent bodies, extruded and bevelled, a solid halftone a
//!   little brighter at the outer edge; turned into a luminance plate (white, the light in alpha)
//!   so the cluster tints it with the palette's primary (always lit, like the real display).
//! - **Lens and gauge (Blender):** the G-Force lens (a dome of dark smoked glass in a polished
//!   ring) and the boost gauge (a dark face in a chrome bezel), each per key azimuth.
//!
//! `cargo run --release -p asset-forge -- --lane cloud build --set amg_mbux_superscreen`. The SDXL
//! renders are cached in the forge scratch by their fingerprints; the Blender renders re-run on
//! every build. The files go straight to the cluster's folder with its own ledger, so the shared
//! `assets/ASSETS.md` is not touched. The cluster draws procedural parts when files are missing.
//!
//! Keep the geometry in step with `crates/fh6-clusters/src/clusters/amg_mbux_superscreen/geom.rs`.
use crate::blender;
use crate::comfy::{self, Comfy};
use crate::ctx::Ctx;
use crate::error::{io_err, tool_err, Result};
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use serde_json::{json, Map, Value};
use std::fmt::Write as _;
use std::path::Path;
use std::time::Duration;

/// Output folder (repo-relative).
const DIR: &str = "assets/clusters/amg_mbux_superscreen";
/// Render scale: image px per 4K local px.
const SCALE: f32 = 2.0;
/// Key-light azimuths (degrees, screen convention: −90 is straight up), file index order.
const LIGHTS: [f32; 2] = [-115.0, -65.0];

/// The tunnel centre (`Geom::tunnel`, local 4K px, y down).
const TUNNEL: (f32, f32) = (0.0, -36.0);
/// The crescents: outer radius, width at 9 and 3 o'clock, half span (degrees).
const CRESCENT: (f32, f32, f32) = (300.0, 80.0, 43.0);
/// The disc (the light at the end of the tunnel): centre and radius.
const DISC: (f32, f32, f32) = (0.0, -62.0, 88.0);
/// The horizon (local y): the disc is cut here and the road starts here.
const HORIZON: f32 = -58.0;
/// Ring lines: count, the gap outside the disc, the gap inside the crescents, and the ease.
const RINGS: usize = 5;
/// See [`RINGS`].
const RING_GAPS: (f32, f32) = (18.0, 16.0);
/// See [`RINGS`].
const RING_EASE: f32 = 1.15;
/// How far the disc sits behind the outermost ring line (px).
const DEPTH: f32 = 40.0;
/// The road: top y, top half width, bottom y, bottom half width.
const FLOOR: [f32; 4] = [HORIZON, 44.0, 200.0, 214.0];
/// The scale bands: inner radius, outer radius, from and to (degrees, the left band; the right
/// mirrors).
const BAND: [f32; 4] = [312.0, 392.0, 150.0, 216.0];
/// The plate rects (local px): min x, min y, max x, max y.
const ASSEMBLY_RECT: [f32; 4] = [-404.0, -280.0, 404.0, 208.0];
/// See [`ASSEMBLY_RECT`].
const CRESCENT_RECT: [f32; 4] = [-304.0, -244.0, 304.0, 172.0];
/// The widgets' radius, and the margin each plate keeps round it (lens, gauge).
const WIDGET_R: f32 = 76.0;
/// See [`WIDGET_R`].
const MARGINS: (f32, f32) = (12.0, 16.0);
/// The halftone's tile on the wall (local px): 32 dots across, a 7 px pitch.
const DOT_TILE: f32 = 224.0;
/// The brushed metal's tile (local px).
const METAL_TILE: f32 = 512.0;

/// Blender seed and Cycles samples.
const BLENDER_SEED: u64 = 1408;
/// See [`BLENDER_SEED`].
const SAMPLES: u32 = 160;
/// The crescent plate's brightest light maps to this alpha.
const PLATE_PEAK: f32 = 1.0;

/// The halftone source macro.
const DOT_PROMPT: &str = "extreme macro photograph of a micro-perforated black anodised aluminium grille, a dense regular field of tiny round holes lit softly from behind, flat even studio lighting, seamless surface texture, monochrome black and grey, ultra detailed, sharp focus";
const DOT_NEGATIVE: &str = "text, letters, numbers, watermark, logo, frame, border, vignette, perspective, people, lowres, blurry, jpeg artifacts, color, hexagon outlines, honeycomb walls, grid lines";
const DOT_SEED: u64 = 1407;
/// The brushed-metal source macro.
const METAL_PROMPT: &str = "extreme macro photograph of brushed anodised aluminium, fine straight parallel brush lines running horizontally, flat even studio lighting, seamless surface texture, monochrome grey, ultra detailed, sharp focus";
const METAL_NEGATIVE: &str = "text, letters, numbers, watermark, logo, frame, border, vignette, perspective, scratches, dents, rust, colour, lowres, blurry, jpeg artifacts";
const METAL_SEED: u64 = 1409;
/// Shipped texture size, halftone dots across and hex rows (even, so the offset rows tile).
const DOT_SIZE: usize = 1024;
/// See [`DOT_SIZE`].
const DOTS_ACROSS: usize = 32;
/// See [`DOT_SIZE`].
const DOT_ROWS: usize = 36;

/// The tunnel and the side scales. Params: `rect`, `scale`, `rings` (`[cx, cy, r]` from the disc
/// out to the crescents), `disc` (`[cx, cy, r]`), `horizon`, `depth`, `floor`, `band` (`[cx, cy,
/// r_in, r_out, from, to]`), `lights`, `samples`, `dots`, `dot_tile`, `metal` (path or ""),
/// `metal_tile`, `world`, `prefix`.
pub const ASSEMBLY_BPY: &str = r#"
sc = reset()
gpu_cycles(sc, samples=ARGS.get('samples', 128), denoise=True, bounces=8)
sc.render.film_transparent = True
view(sc, 'Standard')
S = float(ARGS['scale'])
X0, Y0, X1, Y1 = [float(v) for v in ARGS['rect']]
SEG = 384

def P(x, y, z=0.0):
    # screen px (y down) -> Blender (y up)
    return (x, -y, z)

def metal_rough(NT, base_rough, scale_px=40.0):
    # Brushed grain: the ComfyUI metal macro when given, else stretched noise.
    tc = node(NT, 'ShaderNodeTexCoord')
    mp = node(NT, 'ShaderNodeMapping')
    lk(NT, tc.outputs['Object'], mp.inputs['Vector'])
    if ARGS.get('metal'):
        t = 1.0 / float(ARGS.get('metal_tile', 512.0))
        mp.inputs['Scale'].default_value = (t, t, 1.0)
        tex = node(NT, 'ShaderNodeTexImage', interpolation='Cubic', extension='REPEAT')
        tex.image = bpy.data.images.load(ARGS['metal'], check_existing=True)
        tex.image.colorspace_settings.name = 'Non-Color'
        lk(NT, mp.outputs[0], tex.inputs['Vector'])
        src = tex.outputs['Color']
    else:
        mp.inputs['Scale'].default_value = (1.0 / scale_px, 1.0 / (scale_px * 0.04), 1.0)
        tex = node(NT, 'ShaderNodeTexNoise')
        tex.inputs['Scale'].default_value = 1.0
        tex.inputs['Detail'].default_value = 8.0
        lk(NT, mp.outputs[0], tex.inputs['Vector'])
        src = tex.outputs['Fac']
    mr = node(NT, 'ShaderNodeMapRange', clamp=True)
    mr.inputs['To Min'].default_value = base_rough * 0.7
    mr.inputs['To Max'].default_value = base_rough * 1.35
    lk(NT, src, mr.inputs['Value'])
    return mr.outputs['Result']

def metal_mat(name, base, rough, aniso=0.5):
    m, NT = new_mat(name)
    b = node(NT, 'ShaderNodeBsdfPrincipled')
    b.inputs['Base Color'].default_value = (base[0], base[1], base[2], 1.0)
    b.inputs['Metallic'].default_value = 1.0
    lk(NT, metal_rough(NT, rough), b.inputs['Roughness'])
    try:
        b.inputs['Anisotropic'].default_value = aniso
    except Exception:
        pass
    o = node(NT, 'ShaderNodeOutputMaterial')
    lk(NT, b.outputs[0], o.inputs['Surface'])
    return m

def dots_tex(NT, tile):
    tc = node(NT, 'ShaderNodeTexCoord')
    mp = node(NT, 'ShaderNodeMapping')
    t = 1.0 / float(tile)
    mp.inputs['Scale'].default_value = (t, t, 1.0)
    lk(NT, tc.outputs['Object'], mp.inputs['Vector'])
    tex = node(NT, 'ShaderNodeTexImage', interpolation='Cubic', extension='REPEAT')
    tex.image = bpy.data.images.load(ARGS['dots'], check_existing=True)
    tex.image.colorspace_settings.name = 'Non-Color'
    lk(NT, mp.outputs[0], tex.inputs['Vector'])
    return tex.outputs['Color']


RINGS = ARGS['rings']          # [cx, cy, r] the thin ring lines, disc .. crescents
dcx, dcy, dr = [float(v) for v in ARGS['disc']]
HZ = float(ARGS['horizon'])
DEPTH = float(ARGS['depth'])

def screen_y_mask(NT, y_from, y_to, lo, hi):
    # a factor that runs lo -> hi as the screen y goes from y_from to y_to (Blender Y = -y)
    geo = node(NT, 'ShaderNodeNewGeometry')
    sp = node(NT, 'ShaderNodeSeparateXYZ')
    lk(NT, geo.outputs['Position'], sp.inputs[0])
    mr = node(NT, 'ShaderNodeMapRange', clamp=True)
    mr.inputs['From Min'].default_value = -y_from
    mr.inputs['From Max'].default_value = -y_to
    mr.inputs['To Min'].default_value = lo
    mr.inputs['To Max'].default_value = hi
    lk(NT, sp.outputs['Y'], mr.inputs['Value'])
    return mr.outputs['Result'], sp

def flat(name, pts, z, mat):
    bm = bmesh.new()
    vv = [bm.verts.new(P(x, y, z)) for x, y in pts]
    bm.faces.new(vv)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        if f.normal.z < 0:
            f.normal_flip()
    ob = mesh_obj(name, bm)
    ob.data.materials.append(mat)
    return ob

# The tunnel wall: a shallow funnel from the crescents' inner edge down to the disc, glossy near
# black with a faint halftone that glows towards the light.
n = len(RINGS)
bm = bmesh.new()
loops = []
for k in range(n - 1, -1, -1):
    cx, cy, r = RINGS[k]
    z = -DEPTH * (1.0 - (k / max(n - 1, 1)))
    loops.append([bm.verts.new(P(cx + r * math.cos(2 * math.pi * i / SEG), cy + r * math.sin(2 * math.pi * i / SEG), z)) for i in range(SEG)])
inner = [bm.verts.new(P(dcx + (dr + 4) * math.cos(2 * math.pi * i / SEG), dcy + (dr + 4) * math.sin(2 * math.pi * i / SEG), -DEPTH - 2)) for i in range(SEG)]
loops.append(inner)
for a, b in zip(loops[:-1], loops[1:]):
    for k in range(SEG):
        bm.faces.new((a[k], a[(k + 1) % SEG], b[(k + 1) % SEG], b[k]))
bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
for f in bm.faces:
    if f.normal.z < -1e-4:
        f.normal_flip()
wall = mesh_obj('wall', bm)
for p in wall.data.polygons:
    p.use_smooth = True
m, NT = new_mat('wall')
bsdf = node(NT, 'ShaderNodeBsdfPrincipled')
bsdf.inputs['Base Color'].default_value = (0.003, 0.003, 0.004, 1.0)
bsdf.inputs['Roughness'].default_value = 0.55
bsdf.inputs['Specular IOR Level'].default_value = 0.08
glow, wsp = screen_y_mask(NT, dcy - RINGS[-1][2], HZ, 0.0, 0.06)
wdx = node(NT, 'ShaderNodeMath', operation='SUBTRACT'); lk(NT, wsp.outputs['X'], wdx.inputs[0]); wdx.inputs[1].default_value = dcx
wdy = node(NT, 'ShaderNodeMath', operation='ADD'); lk(NT, wsp.outputs['Y'], wdy.inputs[0]); wdy.inputs[1].default_value = dcy
wcv = node(NT, 'ShaderNodeCombineXYZ'); lk(NT, wdx.outputs[0], wcv.inputs[0]); lk(NT, wdy.outputs[0], wcv.inputs[1])
wln = node(NT, 'ShaderNodeVectorMath', operation='LENGTH'); lk(NT, wcv.outputs[0], wln.inputs[0])
wfo = node(NT, 'ShaderNodeMapRange', clamp=True)
wfo.inputs['From Min'].default_value = dr; wfo.inputs['From Max'].default_value = RINGS[-1][2]
wfo.inputs['To Min'].default_value = 1.0; wfo.inputs['To Max'].default_value = 0.0
lk(NT, wln.outputs['Value'], wfo.inputs['Value'])
wg = node(NT, 'ShaderNodeMath', operation='MULTIPLY'); lk(NT, glow, wg.inputs[0]); lk(NT, wfo.outputs['Result'], wg.inputs[1])
mul = node(NT, 'ShaderNodeMath', operation='MULTIPLY')
lk(NT, dots_tex(NT, ARGS['dot_tile'] * 0.6), mul.inputs[0])
lk(NT, wg.outputs[0], mul.inputs[1])
lk(NT, mul.outputs[0], bsdf.inputs['Emission Strength'])
bsdf.inputs['Emission Color'].default_value = (0.70, 0.78, 0.90, 1.0)
out = node(NT, 'ShaderNodeOutputMaterial')
lk(NT, bsdf.outputs[0], out.inputs['Surface'])
wall.data.materials.append(m)

# The ring lines: thin satin-grey tori lying on the wall, fainter towards the crescents.
for k, (cx, cy, r) in enumerate(RINGS):
    z = -DEPTH * (1.0 - (k / max(n - 1, 1)))
    u = k / max(n - 1, 1)
    w = 1.3 + 0.9 * u
    bpy.ops.mesh.primitive_torus_add(major_radius=r, minor_radius=w * 0.5, major_segments=SEG, minor_segments=12, location=P(cx, cy, z + w * 0.3))
    tor = bpy.context.active_object
    tor.scale = (1.0, 1.0, 0.6)
    bpy.ops.object.shade_smooth()
    g = 0.30 - 0.10 * u
    tor.data.materials.append(metal_mat('ring%d' % k, (g, g * 1.02, g * 1.08), 0.45, 0.0))

# The light at the end of the tunnel: a flat disc cut at the horizon, brightest where it meets
# the road, a fine halftone over it and one darker ring line inside.
bm = bmesh.new()
pts = []
for i in range(SEG + 1):
    a = math.pi + math.pi * i / SEG
    x, y = dcx + dr * math.cos(a), dcy + dr * math.sin(a)
    pts.append((x, min(y, HZ)))
if dcy < HZ:
    half = math.sqrt(max(dr * dr - (HZ - dcy) ** 2, 0.0))
    pts = [(-half + dcx, HZ)] + [p for p in pts if p[1] <= HZ] + [(half + dcx, HZ)]
vv = [bm.verts.new(P(x, y, -DEPTH - 3)) for x, y in pts]
bm.faces.new(vv)
bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
for f in bm.faces:
    if f.normal.z < 0:
        f.normal_flip()
disc = mesh_obj('disc', bm)
m, NT = new_mat('disc')
lit, sp = screen_y_mask(NT, dcy - dr, HZ, 0.0, 1.0)
ramp = node(NT, 'ShaderNodeValToRGB')
el = ramp.color_ramp.elements
el[0].position = 0.0; el[0].color = (0.035, 0.042, 0.055, 1.0)
el[1].position = 1.0; el[1].color = (0.92, 0.95, 1.0, 1.0)
e2 = el.new(0.6); e2.color = (0.24, 0.29, 0.36, 1.0)
lk(NT, lit, ramp.inputs['Fac'])
dx = node(NT, 'ShaderNodeMath', operation='SUBTRACT'); lk(NT, sp.outputs['X'], dx.inputs[0]); dx.inputs[1].default_value = dcx
dy = node(NT, 'ShaderNodeMath', operation='ADD'); lk(NT, sp.outputs['Y'], dy.inputs[0]); dy.inputs[1].default_value = dcy
cv = node(NT, 'ShaderNodeCombineXYZ'); lk(NT, dx.outputs[0], cv.inputs[0]); lk(NT, dy.outputs[0], cv.inputs[1])
ln = node(NT, 'ShaderNodeVectorMath', operation='LENGTH'); lk(NT, cv.outputs[0], ln.inputs[0])
ring_d = node(NT, 'ShaderNodeMath', operation='SUBTRACT'); lk(NT, ln.outputs['Value'], ring_d.inputs[0]); ring_d.inputs[1].default_value = dr * 0.64
ring_a = node(NT, 'ShaderNodeMath', operation='ABSOLUTE'); lk(NT, ring_d.outputs[0], ring_a.inputs[0])
ring_m = node(NT, 'ShaderNodeMapRange', clamp=True)
ring_m.inputs['From Min'].default_value = 0.6; ring_m.inputs['From Max'].default_value = 2.2
ring_m.inputs['To Min'].default_value = 0.35; ring_m.inputs['To Max'].default_value = 1.0
lk(NT, ring_a.outputs[0], ring_m.inputs['Value'])
dmix = node(NT, 'ShaderNodeMapRange', clamp=True)
dmix.inputs['To Min'].default_value = 0.62; dmix.inputs['To Max'].default_value = 1.0
lk(NT, dots_tex(NT, ARGS['dot_tile'] * 0.5), dmix.inputs['Value'])
m1 = node(NT, 'ShaderNodeVectorMath', operation='SCALE'); lk(NT, ramp.outputs['Color'], m1.inputs[0]); lk(NT, ring_m.outputs['Result'], m1.inputs['Scale'])
m2 = node(NT, 'ShaderNodeVectorMath', operation='SCALE'); lk(NT, m1.outputs[0], m2.inputs[0]); lk(NT, dmix.outputs['Result'], m2.inputs['Scale'])
emission_out(NT, m2.outputs[0], strength=1.0)
disc.data.materials.append(m)

# Below the horizon the tunnel sinks into shadow: a black veil over the wall and rings.
veil, VNT = new_mat('veil')
fac, _ = screen_y_mask(VNT, HZ, HZ + 120.0, 0.7, 0.92)
tr = node(VNT, 'ShaderNodeBsdfTransparent')
bk = node(VNT, 'ShaderNodeEmission'); bk.inputs['Color'].default_value = (0.0, 0.0, 0.0, 1.0)
mx = node(VNT, 'ShaderNodeMixShader')
lk(VNT, fac, mx.inputs['Fac']); lk(VNT, tr.outputs[0], mx.inputs[1]); lk(VNT, bk.outputs[0], mx.inputs[2])
vo = node(VNT, 'ShaderNodeOutputMaterial'); lk(VNT, mx.outputs[0], vo.inputs['Surface'])
vr = RINGS[-1][2] + 6.0
flat('veil', [(-vr, HZ), (vr, HZ), (vr, 240.0), (-vr, 240.0)], 2.0, veil)

# The road: a dark wedge lit from the horizon, bright edge lines that fade towards the viewer,
# two lane lines, the bright horizon lip and a dim cross line.
ft, fth, fb, fbh = [float(v) for v in ARGS['floor']]
m, NT = new_mat('road')
rl, _ = screen_y_mask(NT, ft, fb, 0.0, 1.0)
rr = node(NT, 'ShaderNodeValToRGB')
fe = rr.color_ramp.elements
fe[0].position = 0.0; fe[0].color = (0.030, 0.034, 0.040, 1.0)
fe[1].position = 1.0; fe[1].color = (0.0004, 0.0004, 0.0005, 1.0)
e = fe.new(0.28); e.color = (0.0045, 0.005, 0.006, 1.0)
lk(NT, rl, rr.inputs['Fac'])
emission_out(NT, rr.outputs['Color'], strength=1.0)
flat('road', [(-fth, ft), (fth, ft), (fbh, fb), (-fbh, fb)], 3.0, m)

def strip(name, a, b, width, z, mat):
    ax, ay = a; bx, by = b
    nx, ny = -(by - ay), (bx - ax)
    l = math.hypot(nx, ny); nx, ny = nx / l * width * 0.5, ny / l * width * 0.5
    return flat(name, [(ax + nx, ay + ny), (bx + nx, by + ny), (bx - nx, by - ny), (ax - nx, ay - ny)], z, mat)

def fading(name, top, bottom, y0, y1):
    mm, MNT = new_mat(name)
    f, _ = screen_y_mask(MNT, y0, y1, top, bottom)
    c = node(MNT, 'ShaderNodeRGB'); c.outputs[0].default_value = (0.80, 0.84, 0.90, 1.0)
    sc_ = node(MNT, 'ShaderNodeVectorMath', operation='SCALE'); lk(MNT, c.outputs[0], sc_.inputs[0]); lk(MNT, f, sc_.inputs['Scale'])
    emission_out(MNT, sc_.outputs[0], strength=1.0)
    return mm

edge_m = fading('edge', 1.0, 0.12, ft, fb)
strip('edge_l', (-fth, ft), (-fbh, fb), 2.2, 3.4, edge_m)
strip('edge_r', (fth, ft), (fbh, fb), 2.2, 3.4, edge_m)
lane_m = fading('lane', 0.35, 0.05, ft, fb)
for sx in (-1.0, 1.0):
    strip('lane', (sx * fth * 0.45, ft), (sx * fbh * 0.45, fb), 1.3, 3.3, lane_m)
cross_m = fading('cross', 0.16, 0.16, ft, fb)
strip('cross', (-fth - 70.0, 0.0), (fth + 70.0, 0.0), 1.4, 3.3, cross_m)

hz_m, HNT = new_mat('horizon')
geo = node(HNT, 'ShaderNodeNewGeometry'); hsp = node(HNT, 'ShaderNodeSeparateXYZ'); lk(HNT, geo.outputs['Position'], hsp.inputs[0])
hab = node(HNT, 'ShaderNodeMath', operation='ABSOLUTE'); lk(HNT, hsp.outputs['X'], hab.inputs[0])
hmr = node(HNT, 'ShaderNodeMapRange', clamp=True)
hlen = RINGS[-1][2] - 12.0
hmr.inputs['From Min'].default_value = 0.0; hmr.inputs['From Max'].default_value = hlen
hmr.inputs['To Min'].default_value = 1.0; hmr.inputs['To Max'].default_value = 0.0
lk(HNT, hab.outputs[0], hmr.inputs['Value'])
hpw = node(HNT, 'ShaderNodeMath', operation='POWER'); lk(HNT, hmr.outputs['Result'], hpw.inputs[0]); hpw.inputs[1].default_value = 1.6
hc = node(HNT, 'ShaderNodeRGB'); hc.outputs[0].default_value = (0.86, 0.93, 1.0, 1.0)
hs = node(HNT, 'ShaderNodeVectorMath', operation='SCALE'); lk(HNT, hc.outputs[0], hs.inputs[0]); lk(HNT, hpw.outputs[0], hs.inputs['Scale'])
emission_out(HNT, hs.outputs[0], strength=1.2)
strip('horizon', (-hlen, HZ), (hlen, HZ), 2.6, 3.6, hz_m)

# The side scales: dark plum-charcoal bands with a polished rim on the outer edge and a dim line
# on the inner edge, mirrored.
bcx, bcy, bri, bro, b0, b1 = [float(v) for v in ARGS['band']]
band_m, BNT = new_mat('band')
bb = node(BNT, 'ShaderNodeBsdfPrincipled')
bb.inputs['Base Color'].default_value = (0.0055, 0.005, 0.0075, 1.0)
bb.inputs['Roughness'].default_value = 0.6
bb.inputs['Coat Weight'].default_value = 0.0
bb.inputs['Specular IOR Level'].default_value = 0.12
bb.inputs['Coat Roughness'].default_value = 0.12
bgeo = node(BNT, 'ShaderNodeNewGeometry'); bsp = node(BNT, 'ShaderNodeSeparateXYZ'); lk(BNT, bgeo.outputs['Position'], bsp.inputs[0])
bdx = node(BNT, 'ShaderNodeMath', operation='SUBTRACT'); lk(BNT, bsp.outputs['X'], bdx.inputs[0]); bdx.inputs[1].default_value = bcx
bdy = node(BNT, 'ShaderNodeMath', operation='ADD'); lk(BNT, bsp.outputs['Y'], bdy.inputs[0]); bdy.inputs[1].default_value = bcy
bcv = node(BNT, 'ShaderNodeCombineXYZ'); lk(BNT, bdx.outputs[0], bcv.inputs[0]); lk(BNT, bdy.outputs[0], bcv.inputs[1])
bln = node(BNT, 'ShaderNodeVectorMath', operation='LENGTH'); lk(BNT, bcv.outputs[0], bln.inputs[0])
bmr = node(BNT, 'ShaderNodeMapRange', clamp=True)
bmr.inputs['From Min'].default_value = bri; bmr.inputs['From Max'].default_value = bro
bmr.inputs['To Min'].default_value = 0.0; bmr.inputs['To Max'].default_value = 0.010
lk(BNT, bln.outputs['Value'], bmr.inputs['Value'])
lk(BNT, bmr.outputs['Result'], bb.inputs['Emission Strength'])
bb.inputs['Emission Color'].default_value = (0.75, 0.70, 0.95, 1.0)
bo = node(BNT, 'ShaderNodeOutputMaterial'); lk(BNT, bb.outputs[0], bo.inputs['Surface'])
rim_m = metal_mat('rim', (0.90, 0.92, 0.95), 0.08, 0.0)
lip_m = metal_mat('lip', (0.22, 0.23, 0.26), 0.25, 0.0)
NA = 160
for side in (-1.0, 1.0):
    bm = bmesh.new()
    top, bot = [], []
    for i in range(NA + 1):
        t = i / NA
        a = math.radians(b0 + (b1 - b0) * t)
        taper = 0.72 + 0.28 * math.sin(math.pi * t) ** 0.5
        x, y = math.cos(a), math.sin(a)
        if side > 0:
            x = -x
        ro = bro
        ri = bro - (bro - bri) * taper
        top.append((bcx + x * ro, bcy + y * ro))
        bot.append((bcx + x * ri, bcy + y * ri))
    vt = [bm.verts.new(P(x, y, 4.0)) for x, y in top]
    vb = [bm.verts.new(P(x, y, 4.0)) for x, y in bot]
    for i in range(NA):
        bm.faces.new((vt[i], vt[i + 1], vb[i + 1], vb[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        if f.normal.z < 0:
            f.normal_flip()
    ext = bmesh.ops.extrude_face_region(bm, geom=list(bm.faces))
    moved = [e for e in ext['geom'] if isinstance(e, bmesh.types.BMVert)]
    bmesh.ops.translate(bm, verts=moved, vec=(0.0, 0.0, -6.0))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = mesh_obj('band', bm)
    bev = ob.modifiers.new('bev', 'BEVEL')
    bev.width = 2.0
    bev.segments = 4
    for p in ob.data.polygons:
        p.use_smooth = True
    ob.data.materials.append(band_m)
    for nm, pts_, depth_, mat_ in (('rim', top, 1.5, rim_m), ('lip', bot, 0.7, lip_m)):
        curve = bpy.data.curves.new(nm, 'CURVE')
        curve.dimensions = '3D'
        curve.bevel_depth = depth_
        curve.bevel_resolution = 6
        spl = curve.splines.new('POLY')
        spl.points.add(NA)
        for i, (x, y) in enumerate(pts_):
            spl.points[i].co = (x, -y, 4.4, 1.0)
        rob = bpy.data.objects.new(nm, curve)
        bpy.context.collection.objects.link(rob)
        rob.data.materials.append(mat_)

studio_world(sc, strength=float(ARGS.get('world', 0.35)), rotation=0.4, visible=False)
span_x, span_y = X1 - X0, Y1 - Y0
resolution(sc, round(span_x * S), round(span_y * S))
ortho_cam(sc, span_x, center=((X0 + X1) * 0.5, -(Y0 + Y1) * 0.5), z=200.0, clip_end=2000.0)
key = None
for i, az in enumerate(ARGS['lights']):
    if key is not None:
        bpy.data.objects.remove(key, do_unlink=True)
    a = math.radians(az)
    dist = 1200.0
    el = math.radians(float(ARGS.get('elevation', 38.0)))
    loc = (math.cos(a) * dist * math.cos(el), -math.sin(a) * dist * math.cos(el), dist * math.sin(el))
    key = area_light('key', loc, (0.0, -36.0, 0.0), 420.0, float(ARGS.get('energy', 4.0e7)))
    save(sc, os.path.join(OUT, '%s_%d.png' % (ARGS['prefix'], i)), depth='16', mode='RGBA')
done()
"#;

/// Both crescent bodies, emission only. Params: `rect`, `scale`, `crescent` (`[cx, cy, r, w,
/// half_deg]`), `dots`, `dot_tile`, `samples`, `file`.
pub const CRESCENT_BPY: &str = r#"
sc = reset()
gpu_cycles(sc, samples=ARGS.get('samples', 128), denoise=True, bounces=4)
sc.render.film_transparent = True
view(sc, 'Standard')
S = float(ARGS['scale'])
X0, Y0, X1, Y1 = [float(v) for v in ARGS['rect']]
ccx, ccy, cr, cw, half = [float(v) for v in ARGS['crescent']]

def P(x, y, z=0.0):
    return (x, -y, z)

# One crescent body per side: the outer arc, the tapered inner edge, extruded and bevelled so its
# edges catch a little light. Emission: a solid halftone, a little brighter at the outer (front)
# edge than at the inner edge, so the cluster can tint the plate with the
# primary and light it bottom to top with the rpm.
m, NT = new_mat('crescent')
tc = node(NT, 'ShaderNodeTexCoord')
mp = node(NT, 'ShaderNodeMapping')
t = 1.0 / float(ARGS['dot_tile'])
mp.inputs['Scale'].default_value = (t, t, 1.0)
lk(NT, tc.outputs['Object'], mp.inputs['Vector'])
tex = node(NT, 'ShaderNodeTexImage', interpolation='Cubic', extension='REPEAT')
tex.image = bpy.data.images.load(ARGS['dots'], check_existing=True)
tex.image.colorspace_settings.name = 'Non-Color'
lk(NT, mp.outputs[0], tex.inputs['Vector'])
geo = node(NT, 'ShaderNodeNewGeometry')
sp = node(NT, 'ShaderNodeSeparateXYZ')
lk(NT, geo.outputs['Position'], sp.inputs[0])
dx = node(NT, 'ShaderNodeMath', operation='SUBTRACT'); lk(NT, sp.outputs['X'], dx.inputs[0]); dx.inputs[1].default_value = ccx
dy = node(NT, 'ShaderNodeMath', operation='ADD'); lk(NT, sp.outputs['Y'], dy.inputs[0]); dy.inputs[1].default_value = ccy
cv = node(NT, 'ShaderNodeCombineXYZ'); lk(NT, dx.outputs[0], cv.inputs[0]); lk(NT, dy.outputs[0], cv.inputs[1])
ln = node(NT, 'ShaderNodeVectorMath', operation='LENGTH'); lk(NT, cv.outputs[0], ln.inputs[0])
front = node(NT, 'ShaderNodeMapRange', clamp=True)
front.inputs['From Min'].default_value = cr - cw
front.inputs['From Max'].default_value = cr
front.inputs['To Min'].default_value = 0.55
front.inputs['To Max'].default_value = 1.0
lk(NT, ln.outputs['Value'], front.inputs['Value'])
pw = node(NT, 'ShaderNodeMath', operation='POWER'); lk(NT, front.outputs['Result'], pw.inputs[0]); pw.inputs[1].default_value = 1.4
dm = node(NT, 'ShaderNodeMapRange', clamp=True)
dm.inputs['To Min'].default_value = 0.45
dm.inputs['To Max'].default_value = 1.0
lk(NT, tex.outputs['Color'], dm.inputs['Value'])
mul = node(NT, 'ShaderNodeMath', operation='MULTIPLY'); lk(NT, pw.outputs[0], mul.inputs[0]); lk(NT, dm.outputs['Result'], mul.inputs[1])
em = node(NT, 'ShaderNodeEmission'); lk(NT, mul.outputs[0], em.inputs['Strength'])
em.inputs['Color'].default_value = (1.0, 1.0, 1.0, 1.0)
gl = node(NT, 'ShaderNodeBsdfGlossy'); gl.inputs['Roughness'].default_value = 0.18
gl.inputs['Color'].default_value = (0.25, 0.25, 0.25, 1.0)
add = node(NT, 'ShaderNodeAddShader'); lk(NT, em.outputs[0], add.inputs[0]); lk(NT, gl.outputs[0], add.inputs[1])
out = node(NT, 'ShaderNodeOutputMaterial'); lk(NT, add.outputs[0], out.inputs['Surface'])

NA = 220
for side in (-1.0, 1.0):
    bm = bmesh.new()
    outer, inner = [], []
    for i in range(NA + 1):
        tt = i / NA
        if side < 0:
            a = math.radians(180.0 + half - 2.0 * half * tt)
        else:
            a = math.radians(-half + 2.0 * half * tt)
        w = cw * max(math.sin(math.pi * tt), 0.0) ** 0.38
        ox, oy = ccx + cr * math.cos(a), ccy - cr * math.sin(a)
        ix, iy = ccx + (cr - w - 0.4) * math.cos(a), ccy - (cr - w - 0.4) * math.sin(a)
        outer.append(bm.verts.new(P(ox, oy, 0.0)))
        inner.append(bm.verts.new(P(ix, iy, 0.0)))
    for i in range(NA):
        bm.faces.new((outer[i], outer[i + 1], inner[i + 1], inner[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        if f.normal.z < 0:
            f.normal_flip()
    ext = bmesh.ops.extrude_face_region(bm, geom=list(bm.faces))
    moved = [e for e in ext['geom'] if isinstance(e, bmesh.types.BMVert)]
    bmesh.ops.translate(bm, verts=moved, vec=(0.0, 0.0, -5.0))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = mesh_obj('crescent', bm)
    bev = ob.modifiers.new('bev', 'BEVEL')
    bev.width = 1.5
    bev.segments = 3
    ob.data.materials.append(m)

studio_world(sc, strength=0.6, rotation=0.4, visible=False)
span_x, span_y = X1 - X0, Y1 - Y0
resolution(sc, round(span_x * S), round(span_y * S))
ortho_cam(sc, span_x, center=((X0 + X1) * 0.5, -(Y0 + Y1) * 0.5), z=100.0, clip_end=400.0)
save(sc, os.path.join(OUT, ARGS['file']), depth='16', mode='RGBA')
done()
"#;

/// The G-Force lens (`kind` "lens") or the boost gauge (`kind` "gauge"), centred. Params: `kind`,
/// `r`, `margin`, `scale`, `lights`, `samples`, `prefix`.
pub const WIDGET_BPY: &str = r#"
sc = reset()
gpu_cycles(sc, samples=ARGS.get('samples', 128), denoise=True, bounces=10)
sc.render.film_transparent = True
view(sc, 'Standard')
S = float(ARGS['scale'])
R = float(ARGS['r'])
M = R + float(ARGS['margin'])

def chrome(name, base=(0.92, 0.93, 0.95), rough=0.06):
    m, NT = new_mat(name)
    b = node(NT, 'ShaderNodeBsdfPrincipled')
    b.inputs['Base Color'].default_value = (base[0], base[1], base[2], 1.0)
    b.inputs['Metallic'].default_value = 1.0
    b.inputs['Roughness'].default_value = rough
    o = node(NT, 'ShaderNodeOutputMaterial'); lk(NT, b.outputs[0], o.inputs['Surface'])
    return m

kind = ARGS['kind']
if kind == 'lens':
    # A domed lens of dark smoked glass (the cluster draws the etched rings, the crosshair and the
    # dot in the primary on top), seated in a thin polished ring.
    bpy.ops.mesh.primitive_uv_sphere_add(radius=R, segments=160, ring_count=80, location=(0.0, 0.0, 0.0))
    dome = bpy.context.active_object
    dome.scale = (1.0, 1.0, 0.22)
    bpy.ops.object.shade_smooth()
    m, NT = new_mat('smoked_glass')
    b = node(NT, 'ShaderNodeBsdfPrincipled')
    b.inputs['Base Color'].default_value = (0.010, 0.0015, 0.0025, 1.0)
    b.inputs['Roughness'].default_value = 0.05
    b.inputs['Specular IOR Level'].default_value = 0.3
    b.inputs['Coat Weight'].default_value = 0.6
    b.inputs['Coat Roughness'].default_value = 0.02
    b.inputs['IOR'].default_value = 1.52
    o = node(NT, 'ShaderNodeOutputMaterial'); lk(NT, b.outputs[0], o.inputs['Surface'])
    dome.data.materials.append(m)
    bpy.ops.mesh.primitive_torus_add(major_radius=R + 1.5, minor_radius=3.2, major_segments=256, minor_segments=24, location=(0.0, 0.0, 0.5))
    ring = bpy.context.active_object
    bpy.ops.object.shade_smooth()
    ring.data.materials.append(chrome('ring'))
else:
    # A dark gauge face in a broad chrome bezel with a bevelled inner lip.
    bpy.ops.mesh.primitive_cylinder_add(radius=R, depth=2.0, vertices=256, location=(0.0, 0.0, -2.0))
    face = bpy.context.active_object
    m, NT = new_mat('face')
    b = node(NT, 'ShaderNodeBsdfPrincipled')
    b.inputs['Base Color'].default_value = (0.004, 0.0045, 0.005, 1.0)
    b.inputs['Roughness'].default_value = 0.8
    b.inputs['Specular IOR Level'].default_value = 0.15
    o = node(NT, 'ShaderNodeOutputMaterial'); lk(NT, b.outputs[0], o.inputs['Surface'])
    face.data.materials.append(m)
    bpy.ops.mesh.primitive_torus_add(major_radius=R + 4.0, minor_radius=6.0, major_segments=256, minor_segments=32, location=(0.0, 0.0, 0.0))
    bez = bpy.context.active_object
    bez.scale = (1.0, 1.0, 0.7)
    bpy.ops.object.shade_smooth()
    bez.data.materials.append(chrome('bezel', rough=0.10))

studio_world(sc, strength=float(ARGS.get('world', 0.10)), rotation=0.4, visible=False)
resolution(sc, round(2 * M * S), round(2 * M * S))
ortho_cam(sc, 2 * M, center=(0.0, 0.0), z=200.0, clip_end=2000.0)
key = None
for i, az in enumerate(ARGS['lights']):
    if key is not None:
        bpy.data.objects.remove(key, do_unlink=True)
    a = math.radians(az)
    dist = 900.0
    el = math.radians(float(ARGS.get('elevation', 40.0)))
    loc = (math.cos(a) * dist * math.cos(el), -math.sin(a) * dist * math.cos(el), dist * math.sin(el))
    key = area_light('key', loc, (0.0, 0.0, 0.0), 260.0, float(ARGS.get('energy', 5.0e6)))
    save(sc, os.path.join(OUT, '%s_%d.png' % (ARGS['prefix'], i)), depth='16', mode='RGBA')
done()
"#;

/// The ring lines (`[cx, cy, r]`) from the disc out to the crescents' inner edge: the centre
/// slides from the disc to the tunnel centre and the radius grows with a slight ease, as in the
/// cluster's `Geom::ring`.
fn ring_table() -> Vec<[f32; 3]> {
    let r0 = DISC.2 + RING_GAPS.0;
    let r1 = CRESCENT.0 - CRESCENT.1 - RING_GAPS.1;
    (1..=RINGS)
        .map(|k| {
            let u = k as f32 / RINGS as f32;
            [
                DISC.0 + (TUNNEL.0 - DISC.0) * u,
                DISC.1 + (TUNNEL.1 - DISC.1) * u,
                r0 + (r1 - r0) * u.powf(RING_EASE),
            ]
        })
        .collect()
}

/// Runs one SDXL macro through `workflows/sdxl_plate.json`, cached in the scratch by its
/// fingerprint, and returns the raw image.
fn sdxl_macro(ctx: &Ctx, scratch: &Path, name: &str, prompt: &str, negative: &str, seed: u64) -> Result<Img> {
    let wf_path = ctx.paths.forge_dir().join("workflows").join("sdxl_plate.json");
    let workflow = String::from_utf8_lossy(&crate::ctx::read(&wf_path)?).to_string();
    let gen_fp = fingerprint(&[
        workflow.as_bytes(),
        prompt.as_bytes(),
        negative.as_bytes(),
        &seed.to_le_bytes(),
    ]);
    let raw_path = scratch.join(format!("{name}_raw.png"));
    let raw_fp_path = raw_path.with_extension("fp");
    let cached = raw_path.exists()
        && std::fs::read_to_string(&raw_fp_path)
            .map(|s| s == gen_fp)
            .unwrap_or(false);
    if !cached {
        println!("  comfyui {name} macro (SDXL)");
        let c = Comfy::connect(&ctx.paths)?;
        let mut params = Map::new();
        params.insert("prompt".into(), json!(prompt));
        params.insert("negative".into(), json!(negative));
        params.insert("seed".into(), json!(seed));
        params.insert("steps".into(), json!(32));
        params.insert("cfg".into(), json!(6.5));
        params.insert("width".into(), json!(1024));
        params.insert("height".into(), json!(1024));
        params.insert("prefix".into(), json!(format!("fh6_amg_mbux_{name}")));
        let wf = comfy::patch(&workflow, &params)?;
        let images = c.run(&wf, Duration::from_secs(900))?;
        let bytes = images.first().ok_or_else(|| tool_err("comfyui", "no image"))?;
        img::write_file(&raw_path, bytes)?;
        img::write_file(&raw_fp_path, gen_fp.as_bytes())?;
    }
    Ok(img::load(&raw_path)?.to_channels(3))
}

/// Copies one Blender render from the scratch into the asset folder and returns its ledger row.
fn ship(ctx: &Ctx, scratch: &Path, file: &str, generator: &str, used: &str) -> Result<String> {
    let im = img::load(&scratch.join(file))?.to_channels(4);
    let rel = format!("{DIR}/{file}");
    let bytes = img::save_png(&im, &ctx.abs(&rel), false, true)?;
    Ok(format!(
        "| `{file}` | colour, straight alpha | {}x{} | {} KB | {generator} | MIT (asset-forge) | {used} |",
        im.w,
        im.h,
        bytes / 1024
    ))
}

/// Builds the two ComfyUI textures and the Blender plates, and writes the ledger.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let mut rows = Vec::new();
    let scratch = ctx.scratch("amg_mbux_superscreen")?;
    let sdxl = "comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5)";

    // ComfyUI: the grille macro -> the halftone.
    let raw = sdxl_macro(ctx, &scratch, "grille", DOT_PROMPT, DOT_NEGATIVE, DOT_SEED)?;
    let dots = halftone_plate(&raw);
    let dots_rel = format!("{DIR}/halftone_dots.png");
    let bytes = img::save_png(&dots, &ctx.abs(&dots_rel), false, false)?;
    rows.push(format!(
        "| `halftone_dots.png` | grey mask, tileable | {s}x{s} | {} KB | {sdxl} seed={DOT_SEED} prompt=\"{DOT_PROMPT}\" → grey, seamless, high-pass; it modulates the radius of an exact hexagonal grid of {DOTS_ACROSS}×{DOT_ROWS} dots, tile seam score {:.2} | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the halftone of the tunnel wall, the disc and the crescents (Blender input) |",
        bytes / 1024,
        img::tile_seam_score(&dots),
        s = DOT_SIZE
    ));

    // ComfyUI: the brushed-metal macro -> the metal grain.
    let raw = sdxl_macro(ctx, &scratch, "metal", METAL_PROMPT, METAL_NEGATIVE, METAL_SEED)?;
    let metal = metal_plate(&raw);
    let metal_rel = format!("{DIR}/metal_brushed.png");
    let bytes = img::save_png(&metal, &ctx.abs(&metal_rel), false, false)?;
    rows.push(format!(
        "| `metal_brushed.png` | grey, tileable | {s}x{s} | {} KB | {sdxl} seed={METAL_SEED} prompt=\"{METAL_PROMPT}\" → grey, seamless, 2..98% stretched, tile seam score {:.2} | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the roughness grain of the ring lines and the scale bands' rims (Blender input) |",
        bytes / 1024,
        img::tile_seam_score(&metal),
        s = DOT_SIZE
    ));
    let dots_abs = ctx.abs(&dots_rel).to_string_lossy().to_string();
    let metal_abs = ctx.abs(&metal_rel).to_string_lossy().to_string();

    // Blender: the assembly per key light.
    let params = json!({
        "out": scratch.to_string_lossy(),
        "seed": BLENDER_SEED,
        "rect": ASSEMBLY_RECT,
        "scale": SCALE,
        "rings": ring_table(),
        "disc": [DISC.0, DISC.1, DISC.2],
        "horizon": HORIZON,
        "depth": DEPTH,
        "floor": FLOOR,
        "band": [TUNNEL.0, TUNNEL.1, BAND[0], BAND[1], BAND[2], BAND[3]],
        "lights": LIGHTS,
        "samples": SAMPLES,
        "dots": dots_abs,
        "dot_tile": DOT_TILE,
        "metal": metal_abs,
        "metal_tile": METAL_TILE,
        "world": 0.30,
        "prefix": "assembly",
    });
    println!("  blender assembly");
    blender::run(&ctx.paths, "amg_mbux_superscreen_assembly", ASSEMBLY_BPY, &params)?;
    for (i, az) in LIGHTS.iter().enumerate() {
        rows.push(ship(
            ctx,
            &scratch,
            &format!("assembly_{i}.png"),
            &format!("blender:amg_mbux_superscreen::ASSEMBLY_BPY shallow funnel {DEPTH} px deep, {RINGS} thin ring lines, dotted disc r={} cut at the horizon y={HORIZON}, black veil below it, road lit from the horizon, plum-charcoal scale bands r={}..{} with polished rims, studio 0.30 + key {az}° at 38°, scale={SCALE} samples={SAMPLES} seed={BLENDER_SEED}", DISC.2, BAND[0], BAND[1]),
            &format!("the tunnel and the side scales, x {:.0} .. {:.0}, y {:.0} .. {:.0}; cross-faded with the other light by lateral G", ASSEMBLY_RECT[0], ASSEMBLY_RECT[2], ASSEMBLY_RECT[1], ASSEMBLY_RECT[3]),
        )?);
    }

    // Blender: the crescents, as a luminance plate.
    let params = json!({
        "out": scratch.to_string_lossy(),
        "seed": BLENDER_SEED,
        "rect": CRESCENT_RECT,
        "scale": SCALE,
        "crescent": [TUNNEL.0, TUNNEL.1, CRESCENT.0, CRESCENT.1, CRESCENT.2],
        "dots": dots_abs,
        "dot_tile": DOT_TILE,
        "samples": SAMPLES,
        "file": "crescents_render.png",
    });
    println!("  blender crescents");
    blender::run(&ctx.paths, "amg_mbux_superscreen_crescents", CRESCENT_BPY, &params)?;
    let render = img::load(&scratch.join("crescents_render.png"))?.to_channels(4);
    let plate = luminance_plate(&render);
    let bytes = img::save_png(&plate, &ctx.abs(&format!("{DIR}/crescents.png")), false, true)?;
    rows.push(format!(
        "| `crescents.png` | white, light in straight alpha | {}x{} | {} KB | blender:amg_mbux_superscreen::CRESCENT_BPY two extruded, bevelled crescents r={} w={} ±{}°, solid halftone emission a little brighter at the outer edge, scale={SCALE} samples={SAMPLES} seed={BLENDER_SEED}; luminance → alpha, peak at {PLATE_PEAK} | MIT (asset-forge) | the crescents, always lit and tinted by the primary (the rpm runs up their outer edge as a drawn line); x {:.0} .. {:.0}, y {:.0} .. {:.0} |",
        plate.w,
        plate.h,
        bytes / 1024,
        CRESCENT.0,
        CRESCENT.1,
        CRESCENT.2,
        CRESCENT_RECT[0],
        CRESCENT_RECT[2],
        CRESCENT_RECT[1],
        CRESCENT_RECT[3],
    ));

    // Blender: the lens and the gauge, per key light.
    for (kind, margin, what) in [
        (
            "lens",
            MARGINS.0,
            "the G-Force lens: dark smoked glass dome, polished ring",
        ),
        ("gauge", MARGINS.1, "the boost gauge: dark face, chrome bezel"),
    ] {
        let params: Value = json!({
            "out": scratch.to_string_lossy(),
            "seed": BLENDER_SEED,
            "kind": kind,
            "r": WIDGET_R,
            "margin": margin,
            "scale": SCALE,
            "lights": LIGHTS,
            "samples": SAMPLES,
            "prefix": kind,
        });
        println!("  blender {kind}");
        blender::run(&ctx.paths, &format!("amg_mbux_superscreen_{kind}"), WIDGET_BPY, &params)?;
        for (i, az) in LIGHTS.iter().enumerate() {
            rows.push(ship(
                ctx,
                &scratch,
                &format!("{kind}_{i}.png"),
                &format!("blender:amg_mbux_superscreen::WIDGET_BPY kind={kind} r={WIDGET_R} margin={margin}, studio 0.10 + key {az}° at 40°, scale={SCALE} samples={SAMPLES} seed={BLENDER_SEED}"),
                &format!("{what}; centred on the widget, cross-faded with the other light by lateral G"),
            )?);
        }
    }

    let mut md = String::from(
        "# AmgMbuxSuperscreen assets\n\nBuilt by `cargo run --release -p asset-forge -- --lane cloud build --set amg_mbux_superscreen` (recipe `tools/asset-forge/src/recipes/amg_mbux_superscreen.rs`). The dash has no housing (TJS, 2026-11-28): every plate floats on black. Plates are rendered at 2 px per local 4K px over the rect named in their row. The cluster draws procedural parts when files are missing. No maker's mark is in any file.\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in &rows {
        let _ = writeln!(md, "{r}");
    }
    let ledger = ctx.abs(&format!("{DIR}/ASSETS.md"));
    img::write_file(&ledger, md.as_bytes())?;
    println!("  wrote {DIR}/ ({} files + ASSETS.md)", rows.len());
    Ok(())
}

/// The halftone mask: an exact hexagonal grid of round dots whose radius follows the SDXL
/// macro (grey, seamless, high-passed, 2..98% stretched), so the screen is regular but each dot
/// varies a little, like a printed halftone. Antialiased; seamless by construction.
fn halftone_plate(raw: &Img) -> Img {
    let n = DOT_SIZE;
    let small = raw.resize(n, n, false);
    let grey = Img::from_fn(n, n, 1, |x, y| {
        let v = img::luma(small.get(x, y, 0), small.get(x, y, 1), small.get(x, y, 2));
        [v, 0.0, 0.0, 0.0]
    });
    let tile = img::make_seamless(&grey, 0.2);
    let low = tile.resize(n / 16, n / 16, true).resize(n, n, true);
    let mut hp: Vec<f32> = (0..n * n).map(|i| tile.data[i] - low.data[i]).collect();
    let mut sorted = hp.clone();
    sorted.sort_by(f32::total_cmp);
    let lo = sorted[(sorted.len() as f32 * 0.02) as usize];
    let hi = sorted[(sorted.len() as f32 * 0.98) as usize];
    let span = (hi - lo).max(1e-4);
    for v in &mut hp {
        *v = ((*v - lo) / span).clamp(0.0, 1.0);
    }
    let var = Img {
        w: n,
        h: n,
        c: 1,
        data: hp,
    };
    let pitch = n as f32 / DOTS_ACROSS as f32;
    let row_h = n as f32 / DOT_ROWS as f32;
    Img::from_fn(n, n, 3, |x, y| {
        let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
        let row = (py / row_h).floor();
        let mut best = 0.0f32;
        // The nearest dot centres in this row and the two next to it (wrapping).
        for dr in [-1.0f32, 0.0, 1.0] {
            let r = row + dr;
            let cy = (r + 0.5) * row_h;
            let offset = if (r.rem_euclid(2.0)) >= 1.0 { pitch * 0.5 } else { 0.0 };
            let col = ((px - offset) / pitch).floor();
            for dc in [-1.0f32, 0.0, 1.0] {
                let cx = (col + dc + 0.5) * pitch + offset;
                let v = var.sample(cx.rem_euclid(n as f32), cy.rem_euclid(n as f32), 0, true);
                let radius = pitch * 0.5 * (0.32 + 0.4 * v);
                let d = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
                best = best.max((radius - d + 0.5).clamp(0.0, 1.0));
            }
        }
        [best, best, best, 0.0]
    })
}

/// The metal grain: the SDXL macro made grey and seamless, stretched 2..98% to full range.
fn metal_plate(raw: &Img) -> Img {
    let n = DOT_SIZE;
    let small = raw.resize(n, n, false);
    let grey = Img::from_fn(n, n, 1, |x, y| {
        let v = img::luma(small.get(x, y, 0), small.get(x, y, 1), small.get(x, y, 2));
        [v, 0.0, 0.0, 0.0]
    });
    let tile = img::make_seamless(&grey, 0.2);
    let mut sorted = tile.data.clone();
    sorted.sort_by(f32::total_cmp);
    let lo = sorted[(sorted.len() as f32 * 0.02) as usize];
    let hi = sorted[(sorted.len() as f32 * 0.98) as usize];
    let span = (hi - lo).max(1e-4);
    Img::from_fn(n, n, 3, |x, y| {
        let v = ((tile.get(x, y, 0) - lo) / span).clamp(0.0, 1.0);
        [v, v, v, 0.0]
    })
}

/// Turns the emission-only crescent render into a white plate with its light in alpha: the 99.7th
/// percentile of the light maps to [`PLATE_PEAK`], and the cluster tints it with the primary.
fn luminance_plate(render: &Img) -> Img {
    let (w, h) = (render.w, render.h);
    let light: Vec<f32> = (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            let a = render.get(x, y, 3);
            img::luma(render.get(x, y, 0), render.get(x, y, 1), render.get(x, y, 2)) * a
        })
        .collect();
    let mut sorted: Vec<f32> = light.iter().copied().filter(|v| *v > 1e-4).collect();
    sorted.sort_by(f32::total_cmp);
    let peak = sorted
        .get((sorted.len() as f32 * 0.997) as usize)
        .copied()
        .unwrap_or(1.0)
        .max(1e-4);
    Img::from_fn(w, h, 4, |x, y| {
        let a = (light[y * w + x] / peak * PLATE_PEAK).clamp(0.0, 1.0);
        [1.0, 1.0, 1.0, a]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring_lines_run_from_the_disc_to_the_crescents_and_the_bands_sit_outside() {
        let t = ring_table();
        assert_eq!(t.len(), RINGS);
        assert!(t.windows(2).all(|w| w[1][2] > w[0][2]));
        assert!(t[0][2] > DISC.2);
        // The last ring line ends on the tunnel centre, inside the crescents' inner edge.
        let last = t[RINGS - 1];
        assert!((last[0] - TUNNEL.0).abs() < 1e-3 && (last[1] - TUNNEL.1).abs() < 1e-3);
        assert!(last[2] < CRESCENT.0 - CRESCENT.1);
        // The horizon cuts the disc below its top; the bands clear the crescents and fit the plate.
        const { assert!(HORIZON > DISC.1 - DISC.2 && HORIZON < DISC.1 + DISC.2) };
        const { assert!(BAND[0] > CRESCENT.0 && BAND[1] > BAND[0]) };
        const { assert!(BAND[1] + 8.0 <= -ASSEMBLY_RECT[0]) };
        const { assert!(FLOOR[2] <= ASSEMBLY_RECT[3]) };
    }

    #[test]
    fn the_textures_tile_and_the_plate_peaks_where_asked() {
        let raw = Img::from_fn(64, 64, 3, |x, y| {
            let v = ((x * 7 + y * 13) % 17) as f32 / 17.0;
            [v, v, v, 0.0]
        });
        let dots = halftone_plate(&raw);
        assert_eq!((dots.w, dots.h), (DOT_SIZE, DOT_SIZE));
        let metal = metal_plate(&raw);
        assert_eq!((metal.w, metal.h), (DOT_SIZE, DOT_SIZE));
        assert!(metal.data.iter().all(|v| (0.0..=1.0).contains(v)));
        let render = Img::from_fn(32, 32, 4, |x, _| {
            [x as f32 / 31.0, x as f32 / 31.0, x as f32 / 31.0, 1.0]
        });
        let plate = luminance_plate(&render);
        let max = (0..32).map(|x| plate.get(x, 0, 3)).fold(0.0f32, f32::max);
        assert!((max - PLATE_PEAK).abs() < 0.02, "{max}");
    }
}
