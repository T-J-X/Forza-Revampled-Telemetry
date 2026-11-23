//! TeslaAdas bespoke assets (`assets/clusters/tesla_adas/`), set `tesla_adas`:
//!
//! - **Underglow light-pool atlases** (Blender, Cycles): the Phase 6 ego coupe on a white ground
//!   with a downward area light under its floor, rendered through the cluster's own chase cameras
//!   (cockpit and hood: the same eye, pitch and focal length as `tesla_adas::COCKPIT` / `HOOD`) at
//!   11 yaw angles from −50° to +50°. The car is invisible to the camera but still blocks light,
//!   so each pool carries the real tyre and sill shadows in true perspective for that yaw. Post:
//!   luminance → alpha over white with a hard black point, softened, mirrored; 4 × 3 atlas cells.
//! - **Studio-lit car sprites** (Blender, Cycles): the Phase 6 glTF cars with car-studio materials
//!   and lighting. The ego through each chase camera at the same 11 yaws (plus a tail-lamp
//!   emission mask); each traffic body over a 4 elevation × 5 relative-heading grid.
//! - **LED caps** (Blender, Cycles): the shift-light pill lens, off (smoked acrylic) and on (a
//!   back-lit diffuser, tinted live).
//! - **Asphalt plate** (ComfyUI: SDXL base 1.0 + 4x-UltraSharp): a dark road surface receding to a
//!   central vanishing point, used as the road fill between the edge lines. Post: the part below
//!   the horizon, luminance → alpha over white, a near-to-far fade baked in.
//!
//! Each keeps a procedural fallback in the cluster (`setup.assets` may be `None`).
use crate::comfy::{self, Comfy};
use crate::ctx::{Ctx, Format, Spec};
use crate::error::{tool_err, Result};
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use crate::{blender, recipes};
use serde_json::{json, Map};
use std::time::Duration;

const CLUSTERS: &str = "TeslaAdas";
const SET: &str = "tesla_adas";

/// One pool cell's size in local 4K px (rendered at [`POOL_SCALE`]x).
pub const POOL_LOCAL: [usize; 2] = [480, 240];
/// Render scale of a pool cell.
const POOL_SCALE: f32 = 1.5;
/// The yaw angles of the atlas cells (degrees, positive = nose to the right), row-major in a
/// [`POOL_COLS`]-wide grid.
pub const POOL_YAWS: [f32; 11] = [-50.0, -40.0, -30.0, -20.0, -10.0, 0.0, 10.0, 20.0, 30.0, 40.0, 50.0];
/// Atlas columns (3 rows hold the 11 cells).
pub const POOL_COLS: usize = 4;

/// bpy: per view and yaw, the ego (yawed about its centre, light included) on a white ground lit
/// only by an area light under its floor, seen from a chase camera that reproduces
/// `fh6_kit::perspective::Camera25` (eye, pitch, focal px), shifted so the image centre is the
/// ego's ground centre. Writes `pool_<view>_<i>.exr` (scene-linear).
pub const POOL: &str = r#"
import math
P = ARGS
for v in P['views']:
    for i, yaw_deg in enumerate(P['yaws']):
        sc = reset()
        gpu_cycles(sc, samples=int(P['samples']), denoise=True, bounces=6)
        sc.render.film_transparent = True
        w = bpy.data.worlds.new('void')
        sc.world = w
        w.use_nodes = True
        bg = w.node_tree.nodes.get('Background')
        if bg is not None:
            bg.inputs['Strength'].default_value = 0.0
        ego_z = float(P['ego_z'])
        yaw = math.radians(float(yaw_deg))
        # glTF +Z (nose) imports as Blender -Y; kit depth z is Blender -y; a positive kit heading
        # turns the nose towards +x, a positive rotation about Blender +Z.
        bpy.ops.import_scene.gltf(filepath=P['glb'])
        for o in list(sc.objects):
            if o.parent is None and o.type in ('MESH', 'EMPTY'):
                o.location.y -= ego_z
                o.rotation_euler.z += yaw
        for o in sc.objects:
            if o.type == 'MESH':
                o.visible_camera = False
                o.visible_glossy = False
        bpy.ops.mesh.primitive_plane_add(size=80.0, location=(0.0, -ego_z, 0.0))
        ground = bpy.context.active_object
        m, nt = new_mat('ground')
        diffuse = nt.nodes.new('ShaderNodeBsdfDiffuse')
        diffuse.inputs['Color'].default_value = (0.8, 0.8, 0.8, 1.0)
        mout = nt.nodes.new('ShaderNodeOutputMaterial')
        nt.links.new(diffuse.outputs['BSDF'], mout.inputs['Surface'])
        ground.data.materials.append(m)
        ld = bpy.data.lights.new('underglow', 'AREA')
        ld.shape = 'RECTANGLE'
        ld.size = float(P['light_w'])
        ld.size_y = float(P['light_l'])
        ld.energy = float(P['energy'])
        lo = bpy.data.objects.new('underglow', ld)
        lo.location = (0.0, -ego_z, float(P['light_h']))
        lo.rotation_euler = (0.0, 0.0, yaw)
        try:
            lo.visible_camera = False
        except Exception:
            pass
        link(lo)
        ex, ey, ez = v['eye']
        pitch = math.radians(float(v['pitch_deg']))
        f = float(v['focal'])
        lw, lh = P['local']
        k = float(P['scale'])
        rx, ry = int(lw * k), int(lh * k)
        resolution(sc, rx, ry)
        view(sc, 'Standard')
        dy, dz = -ey, ego_z - ez
        s, c = math.sin(pitch), math.cos(pitch)
        zc = dz * c - dy * s
        yc = dy * c + dz * s
        below = -f * yc / zc
        cd = bpy.data.cameras.new('chase')
        cd.sensor_fit = 'HORIZONTAL'
        cd.sensor_width = 36.0
        cd.lens = f * k * 36.0 / rx
        cd.shift_x = 0.0
        cd.shift_y = -(below * k) / max(rx, ry)
        cd.clip_start = 0.05
        cd.clip_end = 400.0
        cam = bpy.data.objects.new('chase', cd)
        cam.location = (ex, -ez, ey)
        cam.rotation_euler = (math.radians(90.0) - pitch, 0.0, math.pi)
        link(cam)
        sc.camera = cam
        save(sc, os.path.join(OUT, 'pool_%s_%d.exr' % (v['name'], i)), exr=True)
done()
"#;

/// bpy: one studio-lit render per shot. The Phase 6 glTF car (subdivided once, smoothed) with
/// car-studio materials on its five slots (clear-coated metallic paint, dark glass, rubber,
/// smoked tail lamps, headlamps), posed by heading and dive, on a shadow catcher, under an
/// overhead softbox, two long strip lights (the paint's highlight lines), a cool rim light from
/// ahead and Blender's studio HDRI for reflections. The camera reproduces
/// `fh6_kit::perspective::Camera25` (pitched, no yaw) with the car's ground centre straight ahead
/// at depth `d` and `h` below the eye; its focal gives `s` px per metre at that point and its
/// lens shift puts that point at `(0.5, ay)` of the image. A `lamp_pass` shot renders only the
/// tail lamps (emission, everything else held out) for the live brake light. Writes PNG
/// (AgX, straight alpha). Blender's camera is right-handed and the kit's is not: the images are
/// mirrored left-right in post.
pub const CARS: &str = r#"

import math
P = ARGS

def principled(name, base, metallic=0.0, rough=0.5, coat=0.0, coat_rough=0.03, emit=None, emit_strength=0.0):
    m, nt = new_mat(name)
    b = nt.nodes.new('ShaderNodeBsdfPrincipled')
    b.inputs['Base Color'].default_value = (base[0], base[1], base[2], 1.0)
    b.inputs['Metallic'].default_value = metallic
    b.inputs['Roughness'].default_value = rough
    try:
        b.inputs['Coat Weight'].default_value = coat
        b.inputs['Coat Roughness'].default_value = coat_rough
    except Exception:
        pass
    if emit is not None:
        b.inputs['Emission Color'].default_value = (emit[0], emit[1], emit[2], 1.0)
        b.inputs['Emission Strength'].default_value = emit_strength
    o = nt.nodes.new('ShaderNodeOutputMaterial')
    nt.links.new(b.outputs['BSDF'], o.inputs['Surface'])
    return m

def holdout(name):
    m, nt = new_mat(name)
    h = nt.nodes.new('ShaderNodeHoldout')
    o = nt.nodes.new('ShaderNodeOutputMaterial')
    nt.links.new(h.outputs[0], o.inputs['Surface'])
    return m

def emissive(name, strength):
    m, nt = new_mat(name)
    e = nt.nodes.new('ShaderNodeEmission')
    e.inputs['Color'].default_value = (1.0, 1.0, 1.0, 1.0)
    e.inputs['Strength'].default_value = strength
    o = nt.nodes.new('ShaderNodeOutputMaterial')
    nt.links.new(e.outputs[0], o.inputs['Surface'])
    return m

def kit_to_blender(x, y, z):
    # glTF / kit (x, y up, z forward) -> Blender (x, -z, y), as the glTF importer does.
    return (x, -z, y)

for shot in P['shots']:
    sc = reset()
    gpu_cycles(sc, samples=int(P['samples']), denoise=True, bounces=8)
    sc.render.film_transparent = True
    lamp_pass = bool(shot.get('lamp_pass', False))
    if lamp_pass:
        w = bpy.data.worlds.new('void')
        sc.world = w
        w.use_nodes = True
        bg = w.node_tree.nodes.get('Background')
        if bg is not None:
            bg.inputs['Strength'].default_value = 0.0
    else:
        studio_world(sc, strength=float(P['world']), hdri='studio.exr', rotation=0.0, visible=False)
    bpy.ops.import_scene.gltf(filepath=shot['glb'])
    paint = shot['paint']
    mats = {
        'paint': principled('paint', paint, metallic=float(P['metallic']), rough=float(P['rough']), coat=1.0, coat_rough=0.03),
        'glass': principled('glass', (0.012, 0.014, 0.018), metallic=0.0, rough=0.04, coat=1.0, coat_rough=0.0),
        'tyre': principled('tyre', (0.012, 0.012, 0.013), rough=0.85),
        'tail': principled('tail', (0.10, 0.004, 0.006), rough=0.12, coat=1.0),
        'head': principled('head', (0.75, 0.78, 0.82), metallic=0.3, rough=0.08, emit=(0.85, 0.9, 1.0), emit_strength=1.5),
    }
    if lamp_pass:
        hold = holdout('hold')
        mats = {k: hold for k in mats}
        mats['tail'] = emissive('tail_lit', 6.0)
    for o in list(sc.objects):
        if o.type != 'MESH':
            continue
        for i, slot in enumerate(o.material_slots):
            name = slot.material.name if slot.material else ''
            for key, m in mats.items():
                if name.endswith('_' + key):
                    o.material_slots[i].material = m
        for poly in o.data.polygons:
            poly.use_smooth = True
        try:
            bpy.context.view_layer.objects.active = o
            o.select_set(True)
            bpy.ops.object.shade_auto_smooth(angle=math.radians(38.0))
            if int(P.get('subsurf', 0)) > 0:
                md = o.modifiers.new('subsurf', 'SUBSURF')
                md.levels = int(P['subsurf'])
                md.render_levels = int(P['subsurf'])
        except Exception as e:
            log('auto smooth', e)
    # The car's pose: dive about kit x (nose down positive) then heading about kit y (nose to +x).
    yaw = math.radians(float(shot['yaw_deg']))
    dive = math.radians(float(shot.get('dive_deg', 0.0)))
    for o in list(sc.objects):
        if o.parent is None and o.type in ('MESH', 'EMPTY'):
            o.rotation_mode = 'XYZ'
            o.rotation_euler = (dive, 0.0, yaw)
            o.location = (0.0, 0.0, float(shot.get('lift', 0.0)))
    if not lamp_pass:
        bpy.ops.mesh.primitive_plane_add(size=60.0, location=(0.0, 0.0, 0.0))
        ground = bpy.context.active_object
        ground.is_shadow_catcher = True
        # Studio lights fixed to the world (the car turns under them): an overhead softbox, two
        # long strips for the paint's highlight lines, and a cool rim from ahead.
        area_light('softbox', (0.0, 0.6, 5.0), (0.0, 0.0, 0.0), 5.0, float(P['key']), size_y=3.0)
        for sx in (-1.0, 1.0):
            area_light('strip', (sx * 3.6, 1.0, 2.6), (0.0, 0.0, 0.6), 0.35, float(P['strip']), size_y=7.0)
        area_light('rim', (0.0, -7.0, 2.2), (0.0, 0.0, 0.7), 4.0, float(P['rim']), color=(0.80, 0.90, 1.0), size_y=1.2)
    # Camera: the cluster's chase-camera orientation (pitched down, no yaw), the car's ground
    # centre straight ahead at depth D and height h below the eye; focal chosen for S px per metre
    # at that point, lens shift putting it at the anchor (0.5, ay).
    h = float(shot['h'])
    D = float(shot['d'])
    pitch = math.radians(float(shot['pitch_deg']))
    rx, ry = int(shot['rx']), int(shot['ry'])
    s, c = math.sin(pitch), math.cos(pitch)
    zc = D * c + h * s
    yc = -h * c + D * s
    f = float(shot['s']) * zc
    below = -f * yc / zc
    resolution(sc, rx, ry)
    view(sc, 'AgX', exposure=float(P['exposure']))
    cd = bpy.data.cameras.new('chase')
    cd.sensor_fit = 'HORIZONTAL'
    cd.sensor_width = 36.0
    cd.lens = f * 36.0 / rx
    cd.shift_x = 0.0
    cd.shift_y = ((float(shot['ay']) - 0.5) * ry - below) / max(rx, ry)
    cd.clip_start = 0.05
    cd.clip_end = 400.0
    cam = bpy.data.objects.new('chase', cd)
    cam.location = kit_to_blender(0.0, h, -D)
    cam.rotation_euler = (math.radians(90.0) - pitch, 0.0, math.pi)
    link(cam)
    sc.camera = cam
    save(sc, os.path.join(OUT, shot['file']), exr=False, depth='8')
done()
"#;

/// bpy: the shift-light LED cap, off and on. A pill lens (the PRELUDE plate mesh, bevelled rim)
/// in clear-coated smoked acrylic, or as a diffuser lit from behind (white emission, hottest at the
/// centre), under a softbox above-front (the highlight along the upper bevel) and a cool strip
/// below, seen straight on through an orthographic camera. Writes `cap_off.png` / `cap_on.png`.
pub const CAPS: &str = r#"
import math
P = ARGS
for state in ('off', 'on'):
    sc = reset()
    gpu_cycles(sc, samples=int(P['samples']), denoise=True, bounces=10)
    sc.render.film_transparent = True
    studio_world(sc, strength=float(P['world']), hdri='studio.exr', rotation=0.6, visible=False)
    w, h, d = P['lens']
    lens = plate_mesh('lens', outline('pill', w, h, segs=24), d, d * 0.85, segs=14, profile=0.6)
    m, nt = new_mat('lens')
    b = nt.nodes.new('ShaderNodeBsdfPrincipled')
    b.inputs['Roughness'].default_value = 0.06
    try:
        b.inputs['Coat Weight'].default_value = 1.0
        b.inputs['Coat Roughness'].default_value = 0.02
    except Exception:
        pass
    o = nt.nodes.new('ShaderNodeOutputMaterial')
    if state == 'off':
        b.inputs['Base Color'].default_value = (0.010, 0.011, 0.013, 1.0)
        nt.links.new(b.outputs['BSDF'], o.inputs['Surface'])
    else:
        # A diffuser lit from behind: white emission, hottest in the middle of the lens.
        b.inputs['Base Color'].default_value = (0.9, 0.9, 0.9, 1.0)
        tc = nt.nodes.new('ShaderNodeTexCoord')
        grad = nt.nodes.new('ShaderNodeTexGradient')
        grad.gradient_type = 'SPHERICAL'
        mp = nt.nodes.new('ShaderNodeMapping')
        mp.inputs['Scale'].default_value = (1.6 / w, 1.6 / h, 1.0)
        nt.links.new(tc.outputs['Object'], mp.inputs['Vector'])
        nt.links.new(mp.outputs['Vector'], grad.inputs['Vector'])
        ramp = nt.nodes.new('ShaderNodeMath')
        ramp.operation = 'MULTIPLY_ADD'
        ramp.inputs[1].default_value = float(P['hot'])
        ramp.inputs[2].default_value = float(P['emit'])
        nt.links.new(grad.outputs['Fac'], ramp.inputs[0])
        b.inputs['Emission Color'].default_value = (1.0, 1.0, 1.0, 1.0)
        nt.links.new(ramp.outputs[0], b.inputs['Emission Strength'])
        nt.links.new(b.outputs['BSDF'], o.inputs['Surface'])
    lens.data.materials.append(m)
    # Studio: a wide softbox above and in front (the highlight along the upper bevel), a low
    # cool strip below (the lower rim).
    area_light('softbox', (0.0, -d * 6.0, d * 9.0), (0.0, h * 0.2, 0.0), w * 1.4, float(P['key']), size_y=h * 1.2)
    area_light('under', (0.0, h * 3.0, -d * 2.0 + d * 4.0), (0.0, 0.0, 0.0), w * 1.2, float(P['rim']), color=(0.8, 0.9, 1.0), size_y=h * 0.3)
    rx, ry = P['cell']
    resolution(sc, rx, ry)
    # The lit diffuser in Standard (linear to 1.0), so its radial falloff survives: AgX would
    # compress the hot centre and the rim into one flat tone.
    view(sc, 'AgX' if state == 'off' else 'Standard', exposure=float(P['exposure']))
    cd = bpy.data.cameras.new('front')
    cd.type = 'ORTHO'
    cd.ortho_scale = float(P['span'])
    cd.sensor_fit = 'HORIZONTAL'
    cam = bpy.data.objects.new('front', cd)
    cam.location = (0.0, 0.0, 10.0)
    link(cam)
    sc.camera = cam
    save(sc, os.path.join(OUT, 'cap_%s.png' % state), exr=False, depth='8')
done()
"#;

const ASPHALT_PROMPT: &str = "photograph of an empty dark asphalt road at night seen from a low camera behind a car, the road surface fills the frame and recedes straight to a vanishing point at the top centre, fine aggregate texture, faint cold blue sheen, moody, minimal";
const ASPHALT_NEGATIVE: &str = "lane markings, white lines, yellow lines, arrows, text, letters, watermark, logo, car, vehicle, people, buildings, trees, street lights, sky, sun, moon, lowres, blurry, jpeg artifacts, oversaturated";
const ASPHALT_SEED: u64 = 1507;
/// The shipped plate size (the cluster maps it over the road from the horizon to the band's
/// bottom).
const ASPHALT_SIZE: [usize; 2] = [1536, 512];

/// Ego sprite cell (render px) and its scale (px per metre at the ground centre).
pub const EGO_CELL: [usize; 2] = [540, 360];
/// Ego sprite scale (render px per metre at the car's ground centre).
pub const EGO_S: f32 = 100.0;
/// Where the car's ground centre sits in a cell (fractions of width and height).
pub const EGO_ANCHOR: [f32; 2] = [0.5, 0.66];
/// Traffic sprite cell (render px).
pub const TRAFFIC_CELL: [usize; 2] = [336, 232];
/// Traffic sprite scale (render px per metre).
pub const TRAFFIC_S: f32 = 64.0;
/// Traffic anchor.
pub const TRAFFIC_ANCHOR: [f32; 2] = [0.5, 0.70];
/// Traffic view elevations (degrees below the horizon to the car's ground centre), atlas rows.
pub const TRAFFIC_ELEVATIONS: [f32; 4] = [2.5, 5.0, 10.0, 20.0];
/// Traffic headings relative to the view ray (degrees, nose to the right), atlas columns; the
/// cluster mirrors a cell for a negative angle.
pub const TRAFFIC_HEADINGS: [f32; 5] = [0.0, 10.0, 20.0, 30.0, 40.0];
/// The traffic bodies: `(name, glb)`.
pub const TRAFFIC_MODELS: [(&str, &str); 3] = [
    ("sedan", "sport_sedan.glb"),
    ("suv", "suv.glb"),
    ("supercar", "supercar.glb"),
];
/// The cluster cameras: `(view, eye [x, y, z], pitch degrees)` (`tesla_adas::COCKPIT` / `HOOD`).
const VIEWS: [(&str, [f32; 3]); 2] = [("cockpit", [0.0, 3.4, -6.0]), ("hood", [0.0, 2.8, -6.5])];
const PITCH_DEG: f32 = 8.0;
const EGO_Z: f32 = 3.2;
/// The traffic rig's eye height (the cockpit's; the elevation bins carry the geometry).
const TRAFFIC_EYE_H: f32 = 3.4;
const EGO_PAINT: [f32; 3] = [0.60, 0.62, 0.66];
const TRAFFIC_PAINT: [f32; 3] = [0.42, 0.44, 0.47];
const CAR_SEED: u64 = 152;

/// Builds the set.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    pools(ctx)?;
    cars(ctx)?;
    caps(ctx)?;
    asphalt(ctx)
}

/// LED cap cell (render px): the lens fills [`CAP_LENS_PX`] of it, centred.
pub const CAP_CELL: [usize; 2] = [288, 128];
/// The lens inside a cap cell (render px).
pub const CAP_LENS_PX: [usize; 2] = [240, 88];

/// The shift-light LED caps: a 2-cell atlas, off (left) and on (right).
fn caps(ctx: &mut Ctx) -> Result<()> {
    let params = json!({
        "seed": 153, "samples": 128, "world": 0.5, "lens": [6.0, 2.2, 0.7], "hot": 0.7, "emit": 0.45,
        "key": 60.0, "rim": 15.0, "exposure": 0.0, "cell": CAP_CELL, "span": 7.2
    });
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        CAPS.as_bytes(),
        params.to_string().as_bytes(),
        b"led-caps-v2",
    ]);
    ctx.job("tesla_adas/led_caps", &fp, |ctx| {
        let out = ctx.scratch("tesla_adas_caps")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "tesla_adas_caps", CAPS, &p)?;
        let mut atlas = Img::new(CAP_CELL[0] * 2, CAP_CELL[1], 4);
        for (i, state) in ["off", "on"].iter().enumerate() {
            // Orthographic and symmetric: no mirror needed.
            let cell = img::load(&out.join(format!("cap_{state}.png")))?;
            atlas.paste(&cell, i * CAP_CELL[0], 0);
        }
        ctx.image(
            "tesla_adas/led_caps",
            "assets/clusters/tesla_adas/led_caps.png",
            &atlas,
            Format::Png8,
            Spec::new(
                SET,
                "sprite",
                "blender:tesla_adas::CAPS (Cycles 128 spp, OptiX denoise, AgX; pill lens 6.0 x 2.2 x 0.7 with a bevelled rim: clear-coated smoked acrylic off, a back-lit white diffuser on; softbox above-front + cool strip below + studio.exr) seed=153",
                CLUSTERS,
            )
            .alpha("straight")
            .qa(format!(
                "2 cells of {} x {} (off, on); the lens fills the centred {} x {} px of each; the on cell is white, tinted live with the pod colour",
                CAP_CELL[0], CAP_CELL[1], CAP_LENS_PX[0], CAP_LENS_PX[1]
            )),
        )
    })
}

/// Mirrors left-right (Blender's right-handed camera vs the kit's).
fn flip_x(src: &Img) -> Img {
    Img::from_fn(src.w, src.h, src.c, |x, y| {
        let mut px = [0.0; 4];
        for (ch, v) in px.iter_mut().enumerate().take(src.c) {
            *v = src.get(src.w - 1 - x, y, ch);
        }
        px
    })
}

/// One CARS shot.
#[allow(clippy::too_many_arguments)]
fn shot(
    glb: &str,
    paint: [f32; 3],
    yaw: f32,
    h: f32,
    d: f32,
    s: f32,
    cell: [usize; 2],
    ay: f32,
    file: &str,
    lamp: bool,
) -> serde_json::Value {
    json!({
        "glb": glb, "paint": paint, "yaw_deg": yaw, "h": h, "d": d, "pitch_deg": PITCH_DEG,
        "s": s, "rx": cell[0], "ry": cell[1], "ay": ay, "file": file, "lamp_pass": lamp
    })
}

/// The studio-lit car sprites: the ego per cluster camera at the pool yaws (with a tail-lamp
/// emission mask at half size), and each traffic body over a grid of view elevations and
/// relative headings.
fn cars(ctx: &mut Ctx) -> Result<()> {
    let ego_glb = ctx.abs("assets/adas/ego_coupe.glb");
    let mut shots = Vec::new();
    let half = [EGO_CELL[0] / 2, EGO_CELL[1] / 2];
    for (view, eye) in VIEWS {
        for (i, yaw) in POOL_YAWS.iter().enumerate() {
            let (g, h, d) = (ego_glb.to_string_lossy(), eye[1], EGO_Z - eye[2]);
            shots.push(shot(
                &g,
                EGO_PAINT,
                *yaw,
                h,
                d,
                EGO_S,
                EGO_CELL,
                EGO_ANCHOR[1],
                &format!("ego_{view}_{i}.png"),
                false,
            ));
            shots.push(shot(
                &g,
                EGO_PAINT,
                *yaw,
                h,
                d,
                EGO_S * 0.5,
                half,
                EGO_ANCHOR[1],
                &format!("lamp_{view}_{i}.png"),
                true,
            ));
        }
    }
    let mut glb_bytes = crate::ctx::read(&ego_glb)?;
    for (name, file) in TRAFFIC_MODELS {
        let glb = ctx.abs(&format!("assets/adas/{file}"));
        glb_bytes.extend(crate::ctx::read(&glb)?);
        for (r, el) in TRAFFIC_ELEVATIONS.iter().enumerate() {
            let d = TRAFFIC_EYE_H / el.to_radians().tan();
            for (c, yaw) in TRAFFIC_HEADINGS.iter().enumerate() {
                shots.push(shot(
                    &glb.to_string_lossy(),
                    TRAFFIC_PAINT,
                    *yaw,
                    TRAFFIC_EYE_H,
                    d,
                    TRAFFIC_S,
                    TRAFFIC_CELL,
                    TRAFFIC_ANCHOR[1],
                    &format!("{name}_{r}_{c}.png"),
                    false,
                ));
            }
        }
    }
    let params = json!({
        "samples": 128, "world": 0.6, "metallic": 0.75, "rough": 0.22, "key": 300.0, "strip": 260.0,
        "rim": 220.0, "exposure": -0.6, "subsurf": 1, "seed": CAR_SEED, "shots": shots
    });
    // The fingerprint leaves out the absolute paths (the glb bytes stand in for them).
    let mut stable = params.clone();
    if let Some(list) = stable["shots"].as_array_mut() {
        for s in list {
            s["glb"] = json!("");
        }
    }
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        CARS.as_bytes(),
        stable.to_string().as_bytes(),
        &glb_bytes,
        b"car-sprites-v1",
    ]);
    ctx.job("tesla_adas/car_sprites", &fp, |ctx| {
        let out = ctx.scratch("tesla_adas_cars")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "tesla_adas_cars", CARS, &p)?;
        let gen = "blender:tesla_adas::CARS (Cycles 128 spp, OptiX denoise, AgX -0.6 EV; subdivided Phase 6 glTF, clear-coated metallic paint, overhead softbox + two strip lights + cool rim + studio.exr HDRI, shadow catcher) seed=152";
        let rows = POOL_YAWS.len().div_ceil(POOL_COLS);
        for (view, _) in VIEWS {
            let mut atlas = Img::new(EGO_CELL[0] * POOL_COLS, EGO_CELL[1] * rows, 4);
            let mut lamps = Img::new(half[0] * POOL_COLS, half[1] * rows, 4);
            for i in 0..POOL_YAWS.len() {
                let (cx, cy) = (i % POOL_COLS, i / POOL_COLS);
                let body = flip_x(&img::load(&out.join(format!("ego_{view}_{i}.png")))?);
                atlas.paste(&body, cx * EGO_CELL[0], cy * EGO_CELL[1]);
                let lamp = flip_x(&img::load(&out.join(format!("lamp_{view}_{i}.png")))?);
                lamps.paste(&to_light_mask(&lamp, 1, 0.0, 1.0, |_, _| 1.0), cx * half[0], cy * half[1]);
            }
            ctx.image(
                &format!("tesla_adas/car_sprites/ego_{view}"),
                &format!("assets/clusters/tesla_adas/ego_{view}.png"),
                &atlas,
                Format::Png8,
                Spec::new(SET, "sprite", format!("{gen} ego_coupe.glb through the {view} chase camera, yaw -50..50 step 10"), CLUSTERS)
                    .alpha("straight")
                    .qa(format!(
                        "{POOL_COLS} x {rows} atlas of {} x {} cells (yaw -50..50 step 10, row-major); {EGO_S} px per metre at the ego's ground centre, which sits at ({}, {}) of each cell; mirrored left-right to the kit's handedness",
                        EGO_CELL[0], EGO_CELL[1], EGO_ANCHOR[0], EGO_ANCHOR[1]
                    )),
            )?;
            ctx.image(
                &format!("tesla_adas/car_sprites/lamps_{view}"),
                &format!("assets/clusters/tesla_adas/ego_lamps_{view}.png"),
                &lamps,
                Format::Png8,
                Spec::new(SET, "mask", format!("{gen} tail-lamp emission pass, {view}"), CLUSTERS)
                    .alpha("straight")
                    .qa(format!("the ego atlas's layout at half size ({} x {} cells); white RGB, alpha = the visible tail-lamp lenses", half[0], half[1])),
            )?;
        }
        for (name, file) in TRAFFIC_MODELS {
            let cols = TRAFFIC_HEADINGS.len();
            let mut atlas = Img::new(TRAFFIC_CELL[0] * cols, TRAFFIC_CELL[1] * TRAFFIC_ELEVATIONS.len(), 4);
            for r in 0..TRAFFIC_ELEVATIONS.len() {
                for c in 0..cols {
                    let cell = flip_x(&img::load(&out.join(format!("{name}_{r}_{c}.png")))?);
                    atlas.paste(&cell, c * TRAFFIC_CELL[0], r * TRAFFIC_CELL[1]);
                }
            }
            ctx.image(
                &format!("tesla_adas/car_sprites/{name}"),
                &format!("assets/clusters/tesla_adas/traffic_{name}.png"),
                &atlas,
                Format::Png8,
                Spec::new(SET, "sprite", format!("{gen} {file}, elevation x relative heading grid"), CLUSTERS)
                    .alpha("straight")
                    .qa(format!(
                        "{cols} x {} atlas of {} x {} cells: rows = view elevation {TRAFFIC_ELEVATIONS:?} deg, columns = heading relative to the view ray {TRAFFIC_HEADINGS:?} deg; {TRAFFIC_S} px per metre at the ground centre, anchored at ({}, {}); mirrored to the kit's handedness",
                        TRAFFIC_ELEVATIONS.len(), TRAFFIC_CELL[0], TRAFFIC_CELL[1], TRAFFIC_ANCHOR[0], TRAFFIC_ANCHOR[1]
                    )),
            )?;
        }
        Ok(())
    })
}

/// Luminance → alpha over white: softened by a down-and-up resample (`soften` = the reduction
/// factor; 1 keeps it sharp), normalised to the 99.7th percentile, everything at or below
/// `black` (after normalising) set to exactly 0, shaped by `gamma` (< 1 lifts a light's falloff
/// tail), and multiplied by `shape(u, v)` (feathers and fades; `u`, `v` in 0..1).
fn to_light_mask(src: &Img, soften: usize, black: f32, gamma: f32, shape: impl Fn(f32, f32) -> f32) -> Img {
    let mut lum = Img::from_fn(src.w, src.h, 1, |x, y| {
        let a = if src.c >= 4 {
            src.get(x, y, 3).clamp(0.0, 1.0)
        } else {
            1.0
        };
        [
            img::luma(src.get(x, y, 0), src.get(x, y, 1), src.get(x, y, 2)) * a,
            0.0,
            0.0,
            0.0,
        ]
    });
    if soften > 1 {
        let small = lum.resize((src.w / soften).max(2), (src.h / soften).max(2), false);
        lum = small.resize(src.w, src.h, false);
    }
    let mut sorted = lum.data.clone();
    sorted.sort_by(f32::total_cmp);
    let peak = sorted[((sorted.len() as f32 * 0.997) as usize).min(sorted.len() - 1)].max(1e-6);
    Img::from_fn(src.w, src.h, 4, |x, y| {
        let u = x as f32 / (src.w - 1).max(1) as f32;
        let v = y as f32 / (src.h - 1).max(1) as f32;
        let l = (lum.get(x, y, 0) / peak).clamp(0.0, 1.0);
        let l = ((l - black) / (1.0 - black)).max(0.0);
        [1.0, 1.0, 1.0, l.powf(gamma) * shape(u, v)]
    })
}

/// Feathered edges: `fx` / `fy` of the width / height fade in from each side.
fn feather(fx: f32, fy: f32) -> impl Fn(f32, f32) -> f32 {
    move |u, v| {
        img::smoothstep(0.0, fx, u)
            * img::smoothstep(0.0, fx, 1.0 - u)
            * img::smoothstep(0.0, fy, v)
            * img::smoothstep(0.0, fy, 1.0 - v)
    }
}

fn pools(ctx: &mut Ctx) -> Result<()> {
    let glb = ctx.abs("assets/adas/ego_coupe.glb");
    let params = json!({
        "views": [
            {"name": "cockpit", "eye": [0.0, 3.4, -6.0], "pitch_deg": 8.0, "focal": 440.0},
            {"name": "hood", "eye": [0.0, 2.8, -6.5], "pitch_deg": 8.0, "focal": 400.0}
        ],
        "yaws": POOL_YAWS,
        "ego_z": 3.2,
        "local": POOL_LOCAL,
        "scale": POOL_SCALE,
        "light_w": 1.9,
        "light_l": 4.0,
        "light_h": 0.16,
        "energy": 520.0,
        "samples": 192,
        "seed": 151
    });
    let glb_bytes = crate::ctx::read(&glb)?;
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        POOL.as_bytes(),
        params.to_string().as_bytes(),
        &glb_bytes,
        b"pool-atlas-v2-mirrored",
    ]);
    ctx.job("tesla_adas/underglow_pools", &fp, |ctx| {
        let out = ctx.scratch("tesla_adas_pools")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        p["glb"] = json!(glb.to_string_lossy());
        blender::run(&ctx.paths, "tesla_adas_pools", POOL, &p)?;
        let (cw, ch) = (
            (POOL_LOCAL[0] as f32 * POOL_SCALE) as usize,
            (POOL_LOCAL[1] as f32 * POOL_SCALE) as usize,
        );
        let rows = POOL_YAWS.len().div_ceil(POOL_COLS);
        for view in ["cockpit", "hood"] {
            let mut atlas = Img::new(cw * POOL_COLS, ch * rows, 4);
            for i in 0..POOL_YAWS.len() {
                let exr = img::load_exr(&out.join(format!("pool_{view}_{i}.exr")))?;
                let cell = flip_x(&to_light_mask(&exr, 10, 0.04, 0.7, feather(0.12, 0.16)));
                atlas.paste(&cell, (i % POOL_COLS) * cw, (i / POOL_COLS) * ch);
            }
            let gen = format!(
                "blender:tesla_adas::POOL {view} chase camera at yaw -50..50 step 10 (Cycles 192 spp, OptiX denoise; ego_coupe.glb camera-invisible, area light 1.9 x 4.0 m at 0.16 m, 520 W) seed=151"
            );
            ctx.image(
                &format!("tesla_adas/underglow_pools/{view}"),
                &format!("assets/clusters/tesla_adas/underglow_pool_{view}.png"),
                &atlas,
                Format::Png8,
                Spec::new(SET, "mask", gen, CLUSTERS)
                    .alpha("straight")
                    .space("linear")
                    .qa(format!(
                        "{POOL_COLS} x {rows} atlas of {cw} x {ch} cells (yaw -50..50 step 10, row-major); white RGB, alpha = ground light: softened 10x, 99.7th percentile = 1, black point 4% (true 0 below), gamma 0.7, edges feathered; each cell {} x {} local px centred on the ego's ground centre",
                        POOL_LOCAL[0], POOL_LOCAL[1]
                    )),
            )?;
        }
        Ok(())
    })
}

fn asphalt(ctx: &mut Ctx) -> Result<()> {
    let wf_path = ctx.paths.forge_dir().join("workflows").join("sdxl_plate.json");
    let workflow = String::from_utf8_lossy(&crate::ctx::read(&wf_path)?).to_string();
    let gen_fp = fingerprint(&[
        workflow.as_bytes(),
        ASPHALT_PROMPT.as_bytes(),
        ASPHALT_NEGATIVE.as_bytes(),
        &ASPHALT_SEED.to_le_bytes(),
        b"1536x640",
    ]);
    let fp = fingerprint(&[gen_fp.as_bytes(), b"asphalt-post-v3"]);
    ctx.job("tesla_adas/asphalt", &fp, |ctx| {
        let raw_path = ctx.scratch("comfy")?.join("tesla_adas_asphalt.png");
        let raw_fp_path = raw_path.with_extension("fp");
        let cached = raw_path.exists() && std::fs::read_to_string(&raw_fp_path).map(|s| s == gen_fp).unwrap_or(false);
        if !cached {
            let c = Comfy::connect(&ctx.paths)?;
            let mut params = Map::new();
            params.insert("prompt".into(), json!(ASPHALT_PROMPT));
            params.insert("negative".into(), json!(ASPHALT_NEGATIVE));
            params.insert("seed".into(), json!(ASPHALT_SEED));
            params.insert("steps".into(), json!(32));
            params.insert("cfg".into(), json!(6.5));
            params.insert("width".into(), json!(1536));
            params.insert("height".into(), json!(640));
            params.insert("prefix".into(), json!("fh6_tesla_adas_asphalt"));
            let wf = comfy::patch(&workflow, &params)?;
            let images = c.run(&wf, Duration::from_secs(900))?;
            let bytes = images.first().ok_or_else(|| tool_err("comfyui", "no image"))?;
            img::write_file(&raw_path, bytes)?;
            img::write_file(&raw_fp_path, gen_fp.as_bytes())?;
        }
        // 6144 x 2560 from the upscaler: the road below the horizon (at about 47%). SDXL painted a
        // centre line despite the negative prompt, so keep the line-free left 46% and mirror it
        // into a symmetric plate (the cluster draws its own lane lines).
        let raw = img::load(&raw_path)?;
        let top = raw.h * 50 / 100;
        let half = raw.w * 46 / 100;
        let road = raw.crop(0, top, half, raw.h - top);
        let sym = Img::from_fn(half * 2, road.h, road.c, |x, y| {
            let sx = if x < half { x } else { 2 * half - 1 - x };
            let mut px = [0.0; 4];
            for (ch, v) in px.iter_mut().enumerate().take(road.c) {
                *v = road.get(sx, y, ch);
            }
            px
        });
        let small = sym.resize(ASPHALT_SIZE[0], ASPHALT_SIZE[1], false);
        // Texture only, the lighting comes from the cluster: near (bottom) full, far (top) 0.
        let mask = to_light_mask(&small, 1, 0.0, 1.0, |u, v| {
            img::smoothstep(0.0, 0.85, v) * (1.0 - img::smoothstep(0.80, 1.0, v)) * img::smoothstep(0.0, 0.08, u) * img::smoothstep(0.0, 0.08, 1.0 - u)
        });
        ctx.image(
            "tesla_adas/asphalt",
            "assets/clusters/tesla_adas/asphalt.png",
            &mask,
            Format::Png8,
            Spec::new(
                SET,
                "mask",
                format!(
                    "comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5, 1536 x 640) seed={ASPHALT_SEED} prompt=\"{ASPHALT_PROMPT}\""
                ),
                CLUSTERS,
            )
            .alpha("straight")
            .space("linear")
            .licence("CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge)")
            .qa("the line-free left 46% of the lower half of the 6144 x 2560 upscale, mirrored into a symmetric plate → 1536 x 512; luminance → alpha over white (tinted live to near-black), a far-to-near fade (0 at the top), a fade-out over the bottom 20% (the band edge) and side feathers baked in"),
        )
    })
}
