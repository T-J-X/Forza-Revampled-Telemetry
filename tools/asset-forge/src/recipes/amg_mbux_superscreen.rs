//! AmgMbuxSuperscreen's bespoke assets (`assets/clusters/amg_mbux_superscreen/`, ledger
//! `assets/clusters/amg_mbux_superscreen/ASSETS.md`):
//!
//! - **Halftone dots (ComfyUI, SDXL + 4x-UltraSharp):** a macro of a micro-perforated grille,
//!   made grey, seamless and high-passed, then used to modulate an exact hexagonal dot grid laid
//!   out in Rust (SDXL draws organic variation well but not a regular screen). The result is the
//!   dotted halftone the real W214 Sporty display prints on its tunnel's inner rings and along the
//!   inside of its crescents; the Blender tunnel uses it as an emission mask.
//! - **Tunnel (Blender, Cycles):** the receding tube modelled in 3D at 1 Blender unit per 4K px.
//!   It is a lofted funnel from the crescents' circle down to the vanishing disc through the six
//!   rings of the cluster's `Geom::ring`, with a glossy black-red wall, a thin emissive line at
//!   each ring, a brighter short arc of the third ring at 9 and 3 o'clock (as on the real
//!   display), and the halftone on the wall and inside the crescents. It is lit only by its own
//!   emission, so the depth comes from the glossy wall reflecting the ring lines. Rendered top-down
//!   and orthographic at 2 px per 4K px, then turned into a luminance plate (white, the light in
//!   alpha), so the cluster tints it with the palette's primary.
//! - **Housing (Blender, Cycles):** the 12.3-inch display unit: a piano-black body with a rounded
//!   edge, a cover glass over the whole face with its polished edge, and the screen's matte black
//!   under the glass. Lit by a dim studio HDRI and a key 35° above the plane, rendered once per
//!   key azimuth (rest −120°, swung −60°); the cluster cross-fades the two with lateral G.
//!
//! `cargo run --release -p asset-forge -- build --set amg_mbux_superscreen`. The SDXL render is
//! cached in the forge scratch by its fingerprint; the Blender renders re-run on every build. The
//! files go straight to the cluster's folder with its own ledger, so the shared `assets/ASSETS.md`
//! is not touched. The cluster draws its procedural housing and tunnel when the files are missing.
//!
//! Keep the rects and the ring table in step with
//! `crates/fh6-clusters/src/clusters/amg_mbux_superscreen/geom.rs`.
use crate::blender;
use crate::comfy::{self, Comfy};
use crate::ctx::Ctx;
use crate::error::{io_err, tool_err, Result};
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use serde_json::{json, Map};
use std::fmt::Write as _;
use std::time::Duration;

/// Output folder (repo-relative).
const DIR: &str = "assets/clusters/amg_mbux_superscreen";
/// Render scale: image px per 4K local px.
const SCALE: f32 = 2.0;

/// The housing rect and corner radius, and the screen rect and radius (`Geom::housing`,
/// `Geom::screen`; local 4K px, y down): min x, min y, max x, max y.
const HOUSING: [f32; 4] = [-720.0, -314.0, 720.0, 236.0];
/// See [`HOUSING`].
const HOUSING_R: f32 = 30.0;
/// See [`HOUSING`].
const SCREEN: [f32; 4] = [-706.0, -300.0, 706.0, 222.0];
/// See [`HOUSING`].
const SCREEN_R: f32 = 18.0;
/// Key-light azimuths of the housing renders (degrees, screen convention), file index order.
const LIGHTS: [f32; 2] = [-120.0, -60.0];

/// The tunnel (`Geom`): its centre, the crescents' radius, width and half span, the disc.
const TUNNEL: (f32, f32) = (0.0, -36.0);
/// See [`TUNNEL`].
const CRESCENT_R: f32 = 278.0;
/// See [`TUNNEL`].
const CRESCENT_W: f32 = 40.0;
/// See [`TUNNEL`] (degrees).
const CRESCENT_HALF_DEG: f32 = 52.0;
/// See [`TUNNEL`].
const DISC: (f32, f32) = (0.0, -62.0);
/// See [`TUNNEL`].
const DISC_R: f32 = 92.0;
/// Receding rings (`geom::RINGS`).
const RINGS: usize = 6;
/// How deep each ring sits behind the one before it (Blender units = 4K px).
const RING_DEPTH: f32 = 46.0;
/// The halftone's tile on the wall (local px): 32 dots across, a 7 px pitch.
const DOT_TILE: f32 = 224.0;

/// Blender seed and Cycles samples.
const BLENDER_SEED: u64 = 1408;
/// See [`BLENDER_SEED`].
const SAMPLES: u32 = 160;
/// The plate's brightest ring line maps to this light (share of the primary), matching the
/// procedural rings' level (the cluster's `RING_LEVEL`, 0.16) with a little headroom.
const PLATE_PEAK: f32 = 0.24;

/// The halftone source macro.
const DOT_PROMPT: &str = "extreme macro photograph of a micro-perforated black anodised aluminium grille, a dense regular field of tiny round holes lit softly from behind, flat even studio lighting, seamless surface texture, monochrome black and grey, ultra detailed, sharp focus";
const DOT_NEGATIVE: &str = "text, letters, numbers, watermark, logo, frame, border, vignette, perspective, people, lowres, blurry, jpeg artifacts, color, hexagon outlines, honeycomb walls, grid lines";
const DOT_SEED: u64 = 1407;
/// Shipped halftone size, dots across and hex rows (even, so the offset rows tile).
const DOT_SIZE: usize = 1024;
/// See [`DOT_SIZE`].
const DOTS_ACROSS: usize = 32;
/// See [`DOT_SIZE`].
const DOT_ROWS: usize = 36;

/// The tunnel: a lofted funnel through the rings, ring lines, the halftone on the wall and inside
/// the crescents, lit by emission only. Params: `rings` (list of `[cx, cy, r, z]`, screen coords),
/// `rect` (`[x0, y0, x1, y1]`), `scale`, `dots` (path), `dot_tile`, `crescent` (`[cx, cy, r, w,
/// half_deg]`), `samples`, `file`.
pub const TUNNEL_BPY: &str = r#"
sc = reset()
gpu_cycles(sc, samples=ARGS.get('samples', 160), denoise=True, bounces=6)
sc.render.film_transparent = True
view(sc, 'Standard')
S = float(ARGS['scale'])
X0, Y0, X1, Y1 = [float(v) for v in ARGS['rect']]
RINGS = ARGS['rings']
SEG = 256

def ring_pts(cx, cy, r, z):
    return [(cx + r * math.cos(2.0 * math.pi * i / SEG), -(cy + r * math.sin(2.0 * math.pi * i / SEG)), z) for i in range(SEG)]

# The wall: one loft from each ring to the next (screen y down -> Blender y up).
bm = bmesh.new()
loops = [[bm.verts.new(p) for p in ring_pts(*rg)] for rg in RINGS]
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

def dots_tex(NT):
    tc = node(NT, 'ShaderNodeTexCoord')
    mp = node(NT, 'ShaderNodeMapping')
    t = 1.0 / float(ARGS['dot_tile'])
    mp.inputs['Scale'].default_value = (t, t, 1.0)
    lk(NT, tc.outputs['Object'], mp.inputs['Vector'])
    tex = node(NT, 'ShaderNodeTexImage', interpolation='Cubic', extension='REPEAT')
    tex.image = bpy.data.images.load(ARGS['dots'], check_existing=True)
    tex.image.colorspace_settings.name = 'Non-Color'
    lk(NT, mp.outputs[0], tex.inputs['Vector'])
    return tex

# Wall: glossy black-red dielectric plus the halftone glowing at mid depth.
zmin = min(rg[3] for rg in RINGS)
m, NT = new_mat('wall')
bsdf = node(NT, 'ShaderNodeBsdfPrincipled')
bsdf.inputs['Base Color'].default_value = (0.012, 0.0, 0.001, 1.0)
bsdf.inputs['Roughness'].default_value = 0.16
bsdf.inputs['Coat Weight'].default_value = 0.6
bsdf.inputs['Coat Roughness'].default_value = 0.04
tex = dots_tex(NT)
geo = node(NT, 'ShaderNodeNewGeometry')
sp = node(NT, 'ShaderNodeSeparateXYZ')
lk(NT, geo.outputs['Position'], sp.inputs[0])
depth = node(NT, 'ShaderNodeMapRange', clamp=True)
depth.inputs['From Min'].default_value = 0.0
depth.inputs['From Max'].default_value = zmin
lk(NT, sp.outputs['Z'], depth.inputs['Value'])
ramp = node(NT, 'ShaderNodeValToRGB')
el = ramp.color_ramp.elements
el[0].position = 0.0
el[0].color = (0.05, 0.05, 0.05, 1.0)
el[1].position = 1.0
el[1].color = (0.12, 0.12, 0.12, 1.0)
mid = el.new(0.42)
mid.color = (0.55, 0.55, 0.55, 1.0)
lk(NT, depth.outputs['Result'], ramp.inputs['Fac'])
mul = node(NT, 'ShaderNodeMath', operation='MULTIPLY')
lk(NT, tex.outputs['Color'], mul.inputs[0])
lk(NT, ramp.outputs['Color'], mul.inputs[1])
lk(NT, mul.outputs[0], bsdf.inputs['Emission Strength'])
bsdf.inputs['Emission Color'].default_value = (1.0, 1.0, 1.0, 1.0)
out = node(NT, 'ShaderNodeOutputMaterial')
lk(NT, bsdf.outputs[0], out.inputs['Surface'])
wall.data.materials.append(m)

# Ring lines: thin emissive tori, dimmer with depth.
n = len(RINGS)
for i, (cx, cy, r, z) in enumerate(RINGS):
    if i == 0 or i == n - 1:
        continue
    bpy.ops.mesh.primitive_torus_add(major_radius=r, minor_radius=1.6, major_segments=256, minor_segments=12, location=(cx, -cy, z + 0.4))
    tor = bpy.context.active_object
    lm, LNT = new_mat('line%d' % i)
    emission_out(LNT, node(LNT, 'ShaderNodeRGB').outputs[0], strength=6.0 * (1.0 - 0.6 * i / n))
    LNT.nodes['RGB'].outputs[0].default_value = (1.0, 1.0, 1.0, 1.0)
    tor.data.materials.append(lm)

# The halftone inside each crescent: an annular sector fading inwards from the crescent's inner
# edge, at the front (z 0.5).
ccx, ccy, cr, cw, half = [float(v) for v in ARGS['crescent']]
r_out, r_in = cr - cw - 2.0, cr - cw - 52.0
hm, HNT = new_mat('halftone')
htex = dots_tex(HNT)
hgeo = node(HNT, 'ShaderNodeNewGeometry')
hsp = node(HNT, 'ShaderNodeSeparateXYZ')
lk(HNT, hgeo.outputs['Position'], hsp.inputs[0])
dx = node(HNT, 'ShaderNodeMath', operation='SUBTRACT')
lk(HNT, hsp.outputs['X'], dx.inputs[0])
dx.inputs[1].default_value = ccx
dy = node(HNT, 'ShaderNodeMath', operation='ADD')
lk(HNT, hsp.outputs['Y'], dy.inputs[0])
dy.inputs[1].default_value = ccy
cxyz = node(HNT, 'ShaderNodeCombineXYZ')
lk(HNT, dx.outputs[0], cxyz.inputs[0])
lk(HNT, dy.outputs[0], cxyz.inputs[1])
vlen = node(HNT, 'ShaderNodeVectorMath', operation='LENGTH')
lk(HNT, cxyz.outputs[0], vlen.inputs[0])
fall = node(HNT, 'ShaderNodeMapRange', clamp=True)
fall.inputs['From Min'].default_value = r_in
fall.inputs['From Max'].default_value = r_out
fall.inputs['To Min'].default_value = 0.0
fall.inputs['To Max'].default_value = 0.8
lk(HNT, vlen.outputs['Value'], fall.inputs['Value'])
hmul = node(HNT, 'ShaderNodeMath', operation='MULTIPLY')
lk(HNT, htex.outputs['Color'], hmul.inputs[0])
lk(HNT, fall.outputs['Result'], hmul.inputs[1])
emission_out(HNT, htex.outputs['Color'], strength=1.0)
em = [nd for nd in HNT.nodes if nd.bl_idname == 'ShaderNodeEmission'][0]
lk(HNT, hmul.outputs[0], em.inputs['Strength'])
for side in (-1.0, 1.0):
    bm = bmesh.new()
    NA = 96
    inner, outer = [], []
    for i in range(NA + 1):
        # Fill t from the bottom tip to the top tip; the sector narrows with the crescent's taper.
        t = i / NA
        a = math.radians(180.0 - half + 2.0 * half * t)
        taper = max(math.sin(math.pi * t), 0.0) ** 0.7
        ro = cr - cw * taper - 2.0
        ri = ro - 50.0 * taper - 0.5
        x, y = math.cos(a), math.sin(a)
        if side > 0:
            x = -x
        outer.append(bm.verts.new((ccx + x * ro, -(ccy + y * ro), 0.5)))
        inner.append(bm.verts.new((ccx + x * ri, -(ccy + y * ri), 0.5)))
    for i in range(NA):
        bm.faces.new((outer[i], outer[i + 1], inner[i + 1], inner[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        if f.normal.z < -1e-4:
            f.normal_flip()
    sector = mesh_obj('sector', bm)
    sector.data.materials.append(hm)

# The inner red arcs: on the real display a short, brighter arc of the third ring stands at 9 and
# 3 o'clock round the disc, echoing the crescents.
icx, icy, ir, iz = RINGS[3]
am, ANT = new_mat('inner_arc')
emission_out(ANT, node(ANT, 'ShaderNodeRGB').outputs[0], strength=4.0)
ANT.nodes['RGB'].outputs[0].default_value = (1.0, 1.0, 1.0, 1.0)
for side in (-1.0, 1.0):
    bm = bmesh.new()
    NA = 64
    inner, outer = [], []
    for i in range(NA + 1):
        t = i / NA
        a = math.radians(180.0 - 38.0 + 76.0 * t)
        wdt = 7.0 * max(math.sin(math.pi * t), 0.0) ** 0.7 + 0.5
        x, y = math.cos(a), math.sin(a)
        if side > 0:
            x = -x
        outer.append(bm.verts.new((icx + x * (ir + 1.0), -(icy + y * (ir + 1.0)), iz + 0.6)))
        inner.append(bm.verts.new((icx + x * (ir - wdt), -(icy + y * (ir - wdt)), iz + 0.6)))
    for i in range(NA):
        bm.faces.new((outer[i], outer[i + 1], inner[i + 1], inner[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        if f.normal.z < -1e-4:
            f.normal_flip()
    arc = mesh_obj('inner_arc', bm)
    arc.data.materials.append(am)

w = bpy.data.worlds.new('black')
sc.world = w
w.use_nodes = True
w.node_tree.nodes['Background'].inputs['Strength'].default_value = 0.0
span_x, span_y = X1 - X0, Y1 - Y0
resolution(sc, round(span_x * S), round(span_y * S))
ortho_cam(sc, span_x, center=((X0 + X1) * 0.5, -(Y0 + Y1) * 0.5), z=60.0, clip_end=800.0)
save(sc, os.path.join(OUT, ARGS['file']), depth='16', mode='RGBA')
done()
"#;

/// The housing: the display unit's body, its cover glass and the screen under it. Params:
/// `housing` (`[x0, y0, x1, y1, r]`), `screen` (same), `scale`, `lights` (azimuths), `samples`,
/// `prefix`.
pub const HOUSING_BPY: &str = r#"
sc = reset()
gpu_cycles(sc, samples=ARGS.get('samples', 160), denoise=True, bounces=8)
sc.render.film_transparent = True
view(sc, 'Standard')
S = float(ARGS['scale'])
hx0, hy0, hx1, hy1, hr = [float(v) for v in ARGS['housing']]
sx0, sy0, sx1, sy1, sr = [float(v) for v in ARGS['screen']]
hw, hh = hx1 - hx0, hy1 - hy0
hcx, hcy = (hx0 + hx1) * 0.5, -(hy0 + hy1) * 0.5
scx, scy = (sx0 + sx1) * 0.5, -(sy0 + sy1) * 0.5

# Body: a piano-black slab with a rounded edge.
body = plate_mesh('body', outline('rrect', hw, hh, hr), 10.0, 3.5)
body.location = (hcx, hcy, -10.0)
bm_, BNT = new_mat('piano')
bb = node(BNT, 'ShaderNodeBsdfPrincipled')
bb.inputs['Base Color'].default_value = (0.004, 0.004, 0.005, 1.0)
bb.inputs['Roughness'].default_value = 0.12
bb.inputs['Coat Weight'].default_value = 1.0
bb.inputs['Coat Roughness'].default_value = 0.03
bo = node(BNT, 'ShaderNodeOutputMaterial')
lk(BNT, bb.outputs[0], bo.inputs['Surface'])
body.data.materials.append(bm_)

# The screen: matte LCD black, just under the glass.
bpy.ops.mesh.primitive_plane_add(size=1.0, location=(scx, scy, 0.15))
scr = bpy.context.active_object
scr.scale = (sx1 - sx0, sy1 - sy0, 1.0)
lm, LNT = new_mat('lcd')
lb = node(LNT, 'ShaderNodeBsdfPrincipled')
lb.inputs['Base Color'].default_value = (0.0, 0.0, 0.0, 1.0)
lb.inputs['Roughness'].default_value = 0.55
lo = node(LNT, 'ShaderNodeOutputMaterial')
lk(LNT, lb.outputs[0], lo.inputs['Surface'])
scr.data.materials.append(lm)

# Cover glass over the whole face, with a polished rounded edge.
glass = plate_mesh('glass', outline('rrect', hw - 3.0, hh - 3.0, hr - 1.5), 1.6, 1.2)
glass.location = (hcx, hcy, 0.2)
gm, GNT = new_mat('glass')
gb = node(GNT, 'ShaderNodeBsdfPrincipled')
gb.inputs['Base Color'].default_value = (1.0, 1.0, 1.0, 1.0)
gb.inputs['Roughness'].default_value = 0.02
gb.inputs['IOR'].default_value = 1.52
gb.inputs['Transmission Weight'].default_value = 1.0
go = node(GNT, 'ShaderNodeOutputMaterial')
lk(GNT, gb.outputs[0], go.inputs['Surface'])
glass.data.materials.append(gm)

studio_world(sc, strength=0.12, rotation=0.4, visible=False)
resolution(sc, round(hw * S), round(hh * S))
ortho_cam(sc, hw, center=(hcx, hcy), z=80.0, clip_end=300.0)
key = None
for i, az in enumerate(ARGS['lights']):
    if key is not None:
        bpy.data.objects.remove(key, do_unlink=True)
    a = math.radians(az)
    dx, dy = math.cos(a), -math.sin(a)
    dist = hw
    el = math.radians(35.0)
    loc = (dx * dist * math.cos(el), dy * dist * math.cos(el), dist * math.sin(el))
    key = area_light('key', loc, (0.0, 0.0, 0.0), dist * 0.25, 6.0e6)
    save(sc, os.path.join(OUT, '%s_%d.png' % (ARGS['prefix'], i)), depth='8', mode='RGBA')
done()
"#;

/// The six rings of `Geom::ring`, with the funnel's lip (t = 0) and the disc (t = 1) at the ends:
/// `[cx, cy, r, z]` in screen coords, z receding into the screen.
fn ring_table() -> Vec<[f32; 4]> {
    (0..=RINGS + 1)
        .map(|k| {
            let t = k as f32 / (RINGS as f32 + 1.0);
            let r = CRESCENT_R * 0.88 + (DISC_R * 1.05 - CRESCENT_R * 0.88) * t.powf(0.85);
            let cx = TUNNEL.0 + (DISC.0 - TUNNEL.0) * t;
            let cy = TUNNEL.1 + (DISC.1 - TUNNEL.1) * t;
            [cx, cy, r, -RING_DEPTH * k as f32]
        })
        .collect()
}

/// The tunnel plate's rect (`Geom::tunnel_rect`): the crescents' circle plus 6 px, clipped to the
/// screen.
fn tunnel_rect() -> [f32; 4] {
    let r = CRESCENT_R + 6.0;
    [
        (TUNNEL.0 - r).max(SCREEN[0]),
        (TUNNEL.1 - r).max(SCREEN[1]),
        (TUNNEL.0 + r).min(SCREEN[2]),
        (TUNNEL.1 + r).min(SCREEN[3]),
    ]
}

/// Builds the halftone, the tunnel plate and the housing renders, and writes the ledger.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let mut rows = Vec::new();
    let scratch = ctx.scratch("amg_mbux_superscreen")?;

    // ComfyUI: the perforated-grille macro (cached by fingerprint).
    let wf_path = ctx.paths.forge_dir().join("workflows").join("sdxl_plate.json");
    let workflow = String::from_utf8_lossy(&crate::ctx::read(&wf_path)?).to_string();
    let gen_fp = fingerprint(&[
        workflow.as_bytes(),
        DOT_PROMPT.as_bytes(),
        DOT_NEGATIVE.as_bytes(),
        &DOT_SEED.to_le_bytes(),
    ]);
    let raw_path = scratch.join("grille_raw.png");
    let raw_fp_path = raw_path.with_extension("fp");
    let cached = raw_path.exists()
        && std::fs::read_to_string(&raw_fp_path)
            .map(|s| s == gen_fp)
            .unwrap_or(false);
    if !cached {
        println!("  comfyui grille macro (SDXL)");
        let c = Comfy::connect(&ctx.paths)?;
        let mut params = Map::new();
        params.insert("prompt".into(), json!(DOT_PROMPT));
        params.insert("negative".into(), json!(DOT_NEGATIVE));
        params.insert("seed".into(), json!(DOT_SEED));
        params.insert("steps".into(), json!(32));
        params.insert("cfg".into(), json!(6.5));
        params.insert("width".into(), json!(1024));
        params.insert("height".into(), json!(1024));
        params.insert("prefix".into(), json!("fh6_amg_mbux_grille"));
        let wf = comfy::patch(&workflow, &params)?;
        let images = c.run(&wf, Duration::from_secs(900))?;
        let bytes = images.first().ok_or_else(|| tool_err("comfyui", "no image"))?;
        img::write_file(&raw_path, bytes)?;
        img::write_file(&raw_fp_path, gen_fp.as_bytes())?;
    }
    let raw = img::load(&raw_path)?.to_channels(3);
    let dots = halftone_plate(&raw);
    let dots_rel = format!("{DIR}/halftone_dots.png");
    let bytes = img::save_png(&dots, &ctx.abs(&dots_rel), false, false)?;
    rows.push(format!(
        "| `halftone_dots.png` | grey mask, tileable | {s}x{s} | {} KB | comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5) seed={DOT_SEED} prompt=\"{DOT_PROMPT}\" → grey, seamless, high-pass; it modulates the radius of an exact hexagonal grid of {DOTS_ACROSS}×{DOT_ROWS} dots, tile seam score {:.2} | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the halftone on the Blender tunnel's wall and inside the crescents |",
        bytes / 1024,
        img::tile_seam_score(&dots),
        s = DOT_SIZE
    ));

    // Blender: the tunnel, turned into a luminance plate.
    let rect = tunnel_rect();
    let rings = ring_table();
    let params = json!({
        "out": scratch.to_string_lossy(),
        "seed": BLENDER_SEED,
        "rings": rings,
        "rect": rect,
        "scale": SCALE,
        "dots": ctx.abs(&dots_rel).to_string_lossy(),
        "dot_tile": DOT_TILE,
        "crescent": [TUNNEL.0, TUNNEL.1, CRESCENT_R, CRESCENT_W, CRESCENT_HALF_DEG],
        "samples": SAMPLES,
        "file": "tunnel_render.png",
    });
    println!("  blender tunnel");
    blender::run(&ctx.paths, "amg_mbux_superscreen_tunnel", TUNNEL_BPY, &params)?;
    let render = img::load(&scratch.join("tunnel_render.png"))?.to_channels(4);
    let plate = luminance_plate(&render);
    let rel = format!("{DIR}/tunnel.png");
    let bytes = img::save_png(&plate, &ctx.abs(&rel), false, true)?;
    rows.push(format!(
        "| `tunnel.png` | white, light in straight alpha | {}x{} | {} KB | blender:amg_mbux_superscreen::TUNNEL_BPY funnel through {RINGS} rings {RING_DEPTH} px apart, glossy wall, emissive ring lines, halftone from `halftone_dots.png` tiled at {DOT_TILE} px, emission only, scale={SCALE} samples={SAMPLES} seed={BLENDER_SEED}; luminance → alpha, peak at {PLATE_PEAK} | MIT (asset-forge) | the tunnel's rings and halftone, tinted by the palette's primary; covers x {:.0} .. {:.0}, y {:.0} .. {:.0} |",
        plate.w,
        plate.h,
        bytes / 1024,
        rect[0],
        rect[2],
        rect[1],
        rect[3],
    ));

    // Blender: the housing per key light.
    let params = json!({
        "out": scratch.to_string_lossy(),
        "seed": BLENDER_SEED,
        "housing": [HOUSING[0], HOUSING[1], HOUSING[2], HOUSING[3], HOUSING_R],
        "screen": [SCREEN[0], SCREEN[1], SCREEN[2], SCREEN[3], SCREEN_R],
        "scale": SCALE,
        "lights": LIGHTS,
        "samples": SAMPLES,
        "prefix": "housing",
    });
    println!("  blender housing");
    blender::run(&ctx.paths, "amg_mbux_superscreen_housing", HOUSING_BPY, &params)?;
    for (i, az) in LIGHTS.iter().enumerate() {
        let file = format!("housing_{i}.png");
        let im = img::load(&scratch.join(&file))?.to_channels(4);
        let rel = format!("{DIR}/{file}");
        let bytes = img::save_png(&im, &ctx.abs(&rel), false, true)?;
        rows.push(format!(
            "| `{file}` | colour, straight alpha | {}x{} | {} KB | blender:amg_mbux_superscreen::HOUSING_BPY piano-black body r={HOUSING_R}, cover glass IOR 1.52, matte LCD black, studio 0.12 + key {az}° at 35°, scale={SCALE} samples={SAMPLES} seed={BLENDER_SEED} | MIT (asset-forge) | the display unit; cross-faded with the other light by lateral G |",
            im.w,
            im.h,
            bytes / 1024,
        ));
    }

    let mut md = String::from(
        "# AmgMbuxSuperscreen assets\n\nBuilt by `cargo run --release -p asset-forge -- build --set amg_mbux_superscreen` (recipe `tools/asset-forge/src/recipes/amg_mbux_superscreen.rs`). The cluster draws its procedural housing and tunnel when the files are missing. The housing renders cover x ±720, y −314 .. 236 in local 4K px at 2 px per px; the tunnel plate covers the rect named in its row. No maker's mark is in any file.\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in rows {
        let _ = writeln!(md, "{r}");
    }
    let ledger = ctx.abs(&format!("{DIR}/ASSETS.md"));
    img::write_file(&ledger, md.as_bytes())?;
    println!("  wrote {DIR}/ (4 files + ASSETS.md)");
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

/// Turns the emission-only tunnel render into a white plate with its light in alpha: the 99.7th
/// percentile of the light maps to [`PLATE_PEAK`], so the plate sits at the procedural rings'
/// level and the cluster tints it with the primary.
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
    fn the_ring_table_runs_from_the_lip_to_the_disc() {
        let t = ring_table();
        assert_eq!(t.len(), RINGS + 2);
        assert!((t[0][2] - CRESCENT_R * 0.88).abs() < 1e-3);
        assert!((t[RINGS + 1][2] - DISC_R * 1.05).abs() < 1e-3);
        assert!(t.windows(2).all(|w| w[1][2] < w[0][2] && w[1][3] < w[0][3]));
        assert_eq!(tunnel_rect(), [-284.0, -300.0, 284.0, 222.0]);
    }

    #[test]
    fn the_halftone_tiles_and_the_plate_peaks_where_asked() {
        let raw = Img::from_fn(64, 64, 3, |x, y| {
            let v = ((x * 7 + y * 13) % 17) as f32 / 17.0;
            [v, v, v, 0.0]
        });
        let dots = halftone_plate(&raw);
        assert_eq!((dots.w, dots.h), (DOT_SIZE, DOT_SIZE));
        let render = Img::from_fn(32, 32, 4, |x, _| {
            [x as f32 / 31.0, x as f32 / 31.0, x as f32 / 31.0, 1.0]
        });
        let plate = luminance_plate(&render);
        let max = (0..32).map(|x| plate.get(x, 0, 3)).fold(0.0f32, f32::max);
        assert!((max - PLATE_PEAK).abs() < 0.02, "{max}");
    }
}
