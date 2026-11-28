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
//!   drives the roughness of the tunnel's metal ridges and the scale bands.
//! - **Assembly (Blender, Cycles):** the tunnel and the side scales, modelled at 1 Blender unit
//!   per 4K px: a glossy near-black funnel lofted through the ring table with the halftone
//!   glowing towards the disc, six grey brushed-metal ridges, a shallow dotted disc lit at its
//!   rim, a dark road floor with a lit lip and metal edges, and two tapered dark-chrome scale
//!   bands with polished rims. Lit by a dim studio HDRI and one key 38° up, rendered once per key
//!   azimuth (rest −115°, swung −65°); the cluster cross-fades the two with lateral G.
//! - **Crescents (Blender):** both crescent bodies, extruded and bevelled, the halftone glowing
//!   brightest at the outer edge; turned into a luminance plate (white, the light in alpha) so the
//!   cluster tints it with the palette's primary and lights it bottom to top with the rpm.
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
const CRESCENT: (f32, f32, f32) = (278.0, 66.0, 56.0);
/// The disc: centre and radius.
const DISC: (f32, f32, f32) = (0.0, -62.0, 92.0);
/// The funnel's lip radius (round [`TUNNEL`]); the wall runs from it to the disc.
const LIP_R: f32 = 212.0;
/// Metal ridges: count, outermost and innermost radius, outermost and innermost width.
const RIDGES: usize = 6;
/// See [`RIDGES`].
const RIDGE_R: (f32, f32) = (200.0, 100.0);
/// See [`RIDGES`].
const RIDGE_W: (f32, f32) = (4.5, 2.5);
/// Wall loops from the lip to the disc, and how deep each sits behind the one before it.
const WALL_LOOPS: usize = 8;
/// See [`WALL_LOOPS`].
const RING_DEPTH: f32 = 46.0;
/// The floor: top y, top half width, bottom y, bottom half width.
const FLOOR: [f32; 4] = [34.0, 64.0, 208.0, 176.0];
/// The scale bands: inner radius, width, from and to (degrees, the left band; the right mirrors).
const BAND: [f32; 4] = [362.0, 34.0, 138.0, 218.0];
/// The plate rects (local px): min x, min y, max x, max y.
const ASSEMBLY_RECT: [f32; 4] = [-404.0, -300.0, 404.0, 236.0];
/// See [`ASSEMBLY_RECT`].
const CRESCENT_RECT: [f32; 4] = [-282.0, -272.0, 282.0, 200.0];
/// The widgets' radius, and the margin each plate keeps round it (lens, gauge).
const WIDGET_R: f32 = 84.0;
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

/// The tunnel and the side scales. Params: `rect`, `scale`, `rings` (`[cx, cy, r, z]` from the
/// lip to the disc), `ridges` (`[cx, cy, r, z, w]`), `disc` (`[cx, cy, r]`), `floor`, `band`
/// (`[cx, cy, r, w, from, to]`), `lights`, `samples`, `dots`, `dot_tile`, `metal` (path or ""),
/// `metal_tile`, `prefix`.
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

RINGS = ARGS['rings']          # [cx, cy, r, z] lip .. disc
RIDGES = ARGS['ridges']        # [cx, cy, r, z, w] the metal ridges
dcx, dcy, dr = [float(v) for v in ARGS['disc']]

# The tunnel wall: a loft through the ring table, glossy near-black, the halftone glowing faintly
# towards the disc.
bm = bmesh.new()
loops = []
for cx, cy, r, z in RINGS:
    loops.append([bm.verts.new(P(cx + r * math.cos(2 * math.pi * i / SEG), cy + r * math.sin(2 * math.pi * i / SEG), z)) for i in range(SEG)])
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
zmin = min(rg[3] for rg in RINGS)
m, NT = new_mat('wall')
bsdf = node(NT, 'ShaderNodeBsdfPrincipled')
bsdf.inputs['Base Color'].default_value = (0.010, 0.010, 0.012, 1.0)
bsdf.inputs['Roughness'].default_value = 0.22
bsdf.inputs['Coat Weight'].default_value = 0.5
bsdf.inputs['Coat Roughness'].default_value = 0.05
geo = node(NT, 'ShaderNodeNewGeometry')
sp = node(NT, 'ShaderNodeSeparateXYZ')
lk(NT, geo.outputs['Position'], sp.inputs[0])
depth = node(NT, 'ShaderNodeMapRange', clamp=True)
depth.inputs['From Min'].default_value = zmin * 0.35
depth.inputs['From Max'].default_value = zmin
depth.inputs['To Min'].default_value = 0.0
depth.inputs['To Max'].default_value = 0.22
lk(NT, sp.outputs['Z'], depth.inputs['Value'])
mul = node(NT, 'ShaderNodeMath', operation='MULTIPLY')
lk(NT, dots_tex(NT, ARGS['dot_tile']), mul.inputs[0])
lk(NT, depth.outputs['Result'], mul.inputs[1])
lk(NT, mul.outputs[0], bsdf.inputs['Emission Strength'])
bsdf.inputs['Emission Color'].default_value = (0.85, 0.88, 0.92, 1.0)
out = node(NT, 'ShaderNodeOutputMaterial')
lk(NT, bsdf.outputs[0], out.inputs['Surface'])
wall.data.materials.append(m)

# The metal ridges: flattened tori standing out of the wall, grey brushed metal, dimmer in depth.
ridge_mats = []
for i, (cx, cy, r, z, w) in enumerate(RIDGES):
    bpy.ops.mesh.primitive_torus_add(major_radius=r, minor_radius=w * 0.5, major_segments=SEG, minor_segments=24, location=P(cx, cy, z + w * 0.15))
    tor = bpy.context.active_object
    tor.scale = (1.0, 1.0, 0.55)
    bpy.ops.object.shade_smooth()
    u = i / max(len(RIDGES) - 1, 1)
    g = 0.42 - 0.22 * u
    tor.data.materials.append(metal_mat('ridge%d' % i, (g, g * 1.02, g * 1.06), 0.26 + 0.1 * u))

# The disc: a shallow dome at the bottom of the tube; dark core, lit rim, brightest at the top,
# a fine halftone over all of it.
bpy.ops.mesh.primitive_uv_sphere_add(radius=dr, segments=128, ring_count=64, location=P(dcx, dcy, zmin - dr * 0.8))
disc = bpy.context.active_object
disc.scale = (1.0, 1.0, 0.18)
bpy.ops.object.shade_smooth()
m, NT = new_mat('disc')
geo = node(NT, 'ShaderNodeNewGeometry')
sp = node(NT, 'ShaderNodeSeparateXYZ')
lk(NT, geo.outputs['Position'], sp.inputs[0])
dx = node(NT, 'ShaderNodeMath', operation='SUBTRACT'); lk(NT, sp.outputs['X'], dx.inputs[0]); dx.inputs[1].default_value = dcx
dy = node(NT, 'ShaderNodeMath', operation='ADD'); lk(NT, sp.outputs['Y'], dy.inputs[0]); dy.inputs[1].default_value = dcy
cv = node(NT, 'ShaderNodeCombineXYZ'); lk(NT, dx.outputs[0], cv.inputs[0]); lk(NT, dy.outputs[0], cv.inputs[1])
ln = node(NT, 'ShaderNodeVectorMath', operation='LENGTH'); lk(NT, cv.outputs[0], ln.inputs[0])
rad = node(NT, 'ShaderNodeMapRange', clamp=True)
rad.inputs['From Min'].default_value = 0.0; rad.inputs['From Max'].default_value = dr
lk(NT, ln.outputs['Value'], rad.inputs['Value'])
ramp = node(NT, 'ShaderNodeValToRGB')
el = ramp.color_ramp.elements
el[0].position = 0.0; el[0].color = (0.05, 0.05, 0.055, 1.0)
el[1].position = 1.0; el[1].color = (0.75, 0.78, 0.82, 1.0)
e2 = el.new(0.72); e2.color = (0.16, 0.17, 0.18, 1.0)
lk(NT, rad.outputs['Result'], ramp.inputs['Fac'])
top = node(NT, 'ShaderNodeMapRange', clamp=True)
top.inputs['From Min'].default_value = -dcy - dr; top.inputs['From Max'].default_value = -dcy + dr
top.inputs['To Min'].default_value = 0.7; top.inputs['To Max'].default_value = 1.3
lk(NT, sp.outputs['Y'], top.inputs['Value'])
dmix = node(NT, 'ShaderNodeMapRange', clamp=True)
dmix.inputs['To Min'].default_value = 0.45; dmix.inputs['To Max'].default_value = 1.0
lk(NT, dots_tex(NT, ARGS['dot_tile'] * 0.55), dmix.inputs['Value'])
m1 = node(NT, 'ShaderNodeVectorMath', operation='SCALE'); lk(NT, ramp.outputs['Color'], m1.inputs[0]); lk(NT, top.outputs['Result'], m1.inputs['Scale'])
m2 = node(NT, 'ShaderNodeVectorMath', operation='SCALE'); lk(NT, m1.outputs[0], m2.inputs[0]); lk(NT, dmix.outputs['Result'], m2.inputs['Scale'])
emission_out(NT, m2.outputs[0], strength=1.0)
disc.data.materials.append(m)

# The floor: a dark wedge in front of the tube, a lit lip along its top and thin metal edges.
ft, fth, fb, fbh = [float(v) for v in ARGS['floor']]
bm = bmesh.new()
vs = [bm.verts.new(P(-fth, ft, 3.0)), bm.verts.new(P(fth, ft, 3.0)), bm.verts.new(P(fbh, fb, 3.0)), bm.verts.new(P(-fbh, fb, 3.0))]
bm.faces.new(vs)
bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
for f in bm.faces:
    if f.normal.z < 0:
        f.normal_flip()
floor = mesh_obj('floor', bm)
m, NT = new_mat('floor')
b = node(NT, 'ShaderNodeBsdfPrincipled')
b.inputs['Roughness'].default_value = 0.6
b.inputs['Specular IOR Level'].default_value = 0.03
geo = node(NT, 'ShaderNodeNewGeometry'); sp = node(NT, 'ShaderNodeSeparateXYZ'); lk(NT, geo.outputs['Position'], sp.inputs[0])
fr = node(NT, 'ShaderNodeMapRange', clamp=True)
fr.inputs['From Min'].default_value = -ft; fr.inputs['From Max'].default_value = -fb
lk(NT, sp.outputs['Y'], fr.inputs['Value'])
framp = node(NT, 'ShaderNodeValToRGB')
fe = framp.color_ramp.elements
fe[0].position = 0.0; fe[0].color = (0.010, 0.010, 0.011, 1.0)
fe[1].position = 1.0; fe[1].color = (0.004, 0.004, 0.004, 1.0)
e = fe.new(0.3); e.color = (0.004, 0.004, 0.0045, 1.0)
lk(NT, fr.outputs['Result'], framp.inputs['Fac'])
lk(NT, framp.outputs['Color'], b.inputs['Base Color'])
o = node(NT, 'ShaderNodeOutputMaterial'); lk(NT, b.outputs[0], o.inputs['Surface'])
floor.data.materials.append(m)

def strip(name, a, b, width, z, mat):
    ax, ay = a; bx, by = b
    nx, ny = -(by - ay), (bx - ax)
    l = math.hypot(nx, ny); nx, ny = nx / l * width * 0.5, ny / l * width * 0.5
    bm = bmesh.new()
    vv = [bm.verts.new(P(ax + nx, ay + ny, z)), bm.verts.new(P(bx + nx, by + ny, z)), bm.verts.new(P(bx - nx, by - ny, z)), bm.verts.new(P(ax - nx, ay - ny, z))]
    bm.faces.new(vv)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        if f.normal.z < 0:
            f.normal_flip()
    ob = mesh_obj(name, bm)
    ob.data.materials.append(mat)
    return ob

lip, LNT = new_mat('lip')
lrgb = node(LNT, 'ShaderNodeRGB'); lrgb.outputs[0].default_value = (0.9, 0.93, 0.96, 1.0)
emission_out(LNT, lrgb.outputs[0], strength=1.1)
strip('lip', (-fth, ft), (fth, ft), 2.5, 3.4, lip)
edge = metal_mat('edge', (0.7, 0.72, 0.75), 0.18, 0.0)
strip('edge_l', (-fth, ft), (-fbh, fb), 2.2, 3.4, edge)
strip('edge_r', (fth, ft), (fbh, fb), 2.2, 3.4, edge)
lane = metal_mat('lane', (0.35, 0.36, 0.38), 0.3, 0.0)
for sx in (-1.0, 1.0):
    strip('lane', (sx * fth * 0.42, ft + 6.0), (sx * fbh * 0.42, fb), 1.4, 3.3, lane)

# The side scales: tapered dark-chrome bands with a polished rim, mirrored.
bcx, bcy, br, bw, b0, b1 = [float(v) for v in ARGS['band']]
band_m = metal_mat('band', (0.20, 0.22, 0.25), 0.20, 0.6)
rim_m = metal_mat('rim', (0.92, 0.93, 0.95), 0.07, 0.0)
NA = 160
for side in (-1.0, 1.0):
    bm = bmesh.new()
    top, bot = [], []
    for i in range(NA + 1):
        t = i / NA
        a = math.radians(b0 + (b1 - b0) * t)
        taper = 0.55 + 0.45 * math.sin(math.pi * t) ** 0.5
        w = bw * taper
        x, y = math.cos(a), math.sin(a)
        if side > 0:
            x = -x
        ro, ri = br + w, br
        for lst, rr in ((top, ro), (bot, ri)):
            lst.append((bcx + x * rr, bcy + y * rr))
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
    # the rim along the outer edge
    curve = bpy.data.curves.new('rim', 'CURVE')
    curve.dimensions = '3D'
    curve.bevel_depth = 1.6
    curve.bevel_resolution = 6
    spl = curve.splines.new('POLY')
    spl.points.add(NA)
    for i, (x, y) in enumerate(top):
        spl.points[i].co = (x, -y, 4.6, 1.0)
    rob = bpy.data.objects.new('rim', curve)
    bpy.context.collection.objects.link(rob)
    rob.data.materials.append(rim_m)

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
# edges catch a little light. Emission only: the halftone glows brightest at the outer (front)
# edge and fades to a quarter at the inner edge, so the cluster can tint the plate with the
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
front.inputs['To Min'].default_value = 0.08
front.inputs['To Max'].default_value = 1.0
lk(NT, ln.outputs['Value'], front.inputs['Value'])
pw = node(NT, 'ShaderNodeMath', operation='POWER'); lk(NT, front.outputs['Result'], pw.inputs[0]); pw.inputs[1].default_value = 2.6
dm = node(NT, 'ShaderNodeMapRange', clamp=True)
dm.inputs['To Min'].default_value = 0.72
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
        w = cw * max(math.sin(math.pi * tt), 0.0) ** 0.7
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

/// The wall loops (`[cx, cy, r, z]`) from the lip to the disc: the centre slides to the disc and
/// the radius shrinks with a slight ease, as in the cluster's `Geom::ring`.
fn wall_table() -> Vec<[f32; 4]> {
    (0..WALL_LOOPS)
        .map(|k| {
            let t = k as f32 / (WALL_LOOPS as f32 - 1.0);
            let r = LIP_R + (DISC.2 * 1.04 - LIP_R) * t.powf(0.85);
            [
                TUNNEL.0 + (DISC.0 - TUNNEL.0) * t,
                TUNNEL.1 + (DISC.1 - TUNNEL.1) * t,
                r,
                -RING_DEPTH * k as f32,
            ]
        })
        .collect()
}

/// The metal ridges (`[cx, cy, r, z, w]`), each at the depth the wall has at its radius.
fn ridge_table() -> Vec<[f32; 5]> {
    let wall = wall_table();
    (0..RIDGES)
        .map(|k| {
            let u = k as f32 / (RIDGES as f32 - 1.0);
            let r = RIDGE_R.0 + (RIDGE_R.1 - RIDGE_R.0) * u;
            let z = wall
                .windows(2)
                .find(|w| w[0][2] >= r && r >= w[1][2])
                .map(|w| {
                    let f = (w[0][2] - r) / (w[0][2] - w[1][2]).max(1e-3);
                    w[0][3] + (w[1][3] - w[0][3]) * f
                })
                .unwrap_or(0.0);
            [
                TUNNEL.0 + (DISC.0 - TUNNEL.0) * u,
                TUNNEL.1 + (DISC.1 - TUNNEL.1) * u,
                r,
                z,
                RIDGE_W.0 + (RIDGE_W.1 - RIDGE_W.0) * u,
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
        "| `metal_brushed.png` | grey, tileable | {s}x{s} | {} KB | {sdxl} seed={METAL_SEED} prompt=\"{METAL_PROMPT}\" → grey, seamless, 2..98% stretched, tile seam score {:.2} | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the roughness grain of the metal ridges and scale bands (Blender input) |",
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
        "rings": wall_table(),
        "ridges": ridge_table(),
        "disc": [DISC.0, DISC.1, DISC.2],
        "floor": FLOOR,
        "band": [TUNNEL.0, TUNNEL.1, BAND[0], BAND[1], BAND[2], BAND[3]],
        "lights": LIGHTS,
        "samples": SAMPLES,
        "dots": dots_abs,
        "dot_tile": DOT_TILE,
        "metal": metal_abs,
        "metal_tile": METAL_TILE,
        "prefix": "assembly",
    });
    println!("  blender assembly");
    blender::run(&ctx.paths, "amg_mbux_superscreen_assembly", ASSEMBLY_BPY, &params)?;
    for (i, az) in LIGHTS.iter().enumerate() {
        rows.push(ship(
            ctx,
            &scratch,
            &format!("assembly_{i}.png"),
            &format!("blender:amg_mbux_superscreen::ASSEMBLY_BPY funnel through {WALL_LOOPS} loops {RING_DEPTH} px apart, {RIDGES} brushed-metal ridges, dotted disc, road floor, dark-chrome scale bands r={} w={}, studio 0.35 + key {az}° at 38°, scale={SCALE} samples={SAMPLES} seed={BLENDER_SEED}", BAND[0], BAND[1]),
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
        "| `crescents.png` | white, light in straight alpha | {}x{} | {} KB | blender:amg_mbux_superscreen::CRESCENT_BPY two extruded, bevelled crescents r={} w={} ±{}°, halftone emission brightest at the outer edge, scale={SCALE} samples={SAMPLES} seed={BLENDER_SEED}; luminance → alpha, peak at {PLATE_PEAK} | MIT (asset-forge) | the crescents, tinted by the primary and lit bottom to top with the rpm; x {:.0} .. {:.0}, y {:.0} .. {:.0} |",
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
    fn the_wall_runs_from_the_lip_to_the_disc_and_the_ridges_sit_on_it() {
        let t = wall_table();
        assert_eq!(t.len(), WALL_LOOPS);
        assert!((t[0][2] - LIP_R).abs() < 1e-3);
        assert!(t.windows(2).all(|w| w[1][2] < w[0][2] && w[1][3] < w[0][3]));
        let r = ridge_table();
        assert_eq!(r.len(), RIDGES);
        assert!(r.iter().all(|k| k[3] < 0.0 && k[2] < LIP_R && k[2] > DISC.2));
        // The ridges stay inside the crescents' inner edge and the bands outside them.
        assert!(r[0][2] + r[0][4] < CRESCENT.0 - CRESCENT.1);
        const { assert!(BAND[0] > CRESCENT.0 + 40.0) };
        // The bands fit their plate.
        const { assert!(BAND[0] + BAND[1] <= -ASSEMBLY_RECT[0]) };
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
