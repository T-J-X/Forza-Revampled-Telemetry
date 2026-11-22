//! Blender recipes. Blender only runs Python, so each recipe is a bpy script kept as a Rust
//! string constant (the repo holds no `.py` files). `blender::run` prepends [`PRELUDE`] and
//! passes the parameters as a JSON file. Every script is deterministic for a given seed.

pub mod adas;
pub mod classic;
pub mod fluid;
pub mod glass;
pub mod scifi;

/// Shared helpers: argument loading, scene reset, Cycles on the GPU, cameras, data passes
/// (normal / height / AO written as emission into 32-bit EXR), studio lighting and saving.
pub const PRELUDE: &str = r#"
import bpy, bmesh, sys, json, math, os, random
from mathutils import Vector, Matrix, Euler

ARGS = json.load(open(sys.argv[sys.argv.index('--') + 1], encoding='utf-8'))
OUT = ARGS['out']
os.makedirs(OUT, exist_ok=True)
SEED = int(ARGS.get('seed', 0))
random.seed(SEED)

def log(*a):
    print('[forge]', *a, flush=True)

def reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    sc.frame_set(1)
    return sc

def gpu_cycles(sc, samples=64, denoise=True, bounces=12):
    sc.render.engine = 'CYCLES'
    prefs = bpy.context.preferences.addons['cycles'].preferences
    dev = 'CPU'
    for t in ('OPTIX', 'CUDA'):
        try:
            prefs.compute_device_type = t
            prefs.get_devices()
            ok = False
            for d in prefs.devices:
                d.use = (d.type == t)
                ok = ok or d.use
            if ok:
                dev = 'GPU'
                break
        except Exception as e:
            log('device', t, e)
    sc.cycles.device = dev
    sc.cycles.samples = samples
    sc.cycles.use_adaptive_sampling = False
    sc.cycles.seed = SEED
    sc.cycles.use_denoising = denoise
    if denoise:
        try:
            sc.cycles.denoiser = 'OPTIX' if dev == 'GPU' else 'OPENIMAGEDENOISE'
        except Exception:
            pass
    sc.cycles.max_bounces = bounces
    sc.cycles.transmission_bounces = bounces
    sc.cycles.glossy_bounces = min(bounces, 8)
    sc.cycles.caustics_reflective = True
    sc.cycles.caustics_refractive = True
    sc.render.film_transparent = True
    sc.render.filter_size = 1.2
    return dev

def resolution(sc, w, h):
    sc.render.resolution_x = int(w)
    sc.render.resolution_y = int(h)
    sc.render.resolution_percentage = 100

def view(sc, name='AgX', look=None, exposure=0.0):
    for n in (name, 'Standard'):
        try:
            sc.view_settings.view_transform = n
            break
        except Exception:
            pass
    try:
        sc.view_settings.look = look if look else 'None'
    except Exception:
        pass
    sc.view_settings.exposure = exposure
    sc.view_settings.gamma = 1.0

def link(ob):
    bpy.context.scene.collection.objects.link(ob)
    return ob

def ortho_cam(sc, width, center=(0.0, 0.0), z=20.0, clip_end=100.0):
    cd = bpy.data.cameras.new('cam')
    cd.type = 'ORTHO'
    cd.ortho_scale = width
    cd.sensor_fit = 'HORIZONTAL'
    cd.clip_start = 0.001
    cd.clip_end = clip_end
    cam = bpy.data.objects.new('cam', cd)
    cam.location = (center[0], center[1], z)
    cam.rotation_euler = (0.0, 0.0, 0.0)
    link(cam)
    sc.camera = cam
    return cam

def persp_cam(sc, loc, target, lens=50.0, clip_end=500.0):
    cd = bpy.data.cameras.new('cam')
    cd.lens = lens
    cd.sensor_fit = 'HORIZONTAL'
    cd.clip_start = 0.01
    cd.clip_end = clip_end
    cam = bpy.data.objects.new('cam', cd)
    cam.location = loc
    d = Vector(target) - Vector(loc)
    cam.rotation_euler = d.to_track_quat('-Z', 'Y').to_euler()
    link(cam)
    sc.camera = cam
    return cam

def mesh_obj(name, bm):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    link(ob)
    return ob

def new_mat(name):
    m = bpy.data.materials.new(name)
    try:
        m.use_nodes = True
    except Exception:
        pass
    nt = m.node_tree
    nt.nodes.clear()
    return m, nt

def node(nt, kind, **kw):
    n = nt.nodes.new(kind)
    for k, v in kw.items():
        setattr(n, k, v)
    return n

def lk(nt, a, b):
    nt.links.new(a, b)

def math_node(nt, op, a=None, b=None, va=0.0, vb=0.0):
    n = node(nt, 'ShaderNodeMath', operation=op)
    if a is not None:
        lk(nt, a, n.inputs[0])
    else:
        n.inputs[0].default_value = va
    if b is not None:
        lk(nt, b, n.inputs[1])
    else:
        n.inputs[1].default_value = vb
    return n.outputs[0]

def emission_out(nt, color_socket, strength=1.0):
    em = node(nt, 'ShaderNodeEmission')
    em.inputs['Strength'].default_value = strength
    lk(nt, color_socket, em.inputs['Color'])
    out = node(nt, 'ShaderNodeOutputMaterial')
    lk(nt, em.outputs[0], out.inputs['Surface'])
    return em

def data_material(hmin=0.0, hmax=1.0):
    # Emission = (Nx*0.5+0.5, Ny*0.5+0.5, (z-hmin)/(hmax-hmin)); top-down ortho => world N = tangent N.
    m, nt = new_mat('forge_data')
    geo = node(nt, 'ShaderNodeNewGeometry')
    sn = node(nt, 'ShaderNodeSeparateXYZ')
    lk(nt, geo.outputs['Normal'], sn.inputs[0])
    sp = node(nt, 'ShaderNodeSeparateXYZ')
    lk(nt, geo.outputs['Position'], sp.inputs[0])
    nx = math_node(nt, 'MULTIPLY_ADD', sn.outputs['X'])
    nx.node.inputs[1].default_value = 0.5
    nx.node.inputs[2].default_value = 0.5
    ny = math_node(nt, 'MULTIPLY_ADD', sn.outputs['Y'])
    ny.node.inputs[1].default_value = 0.5
    ny.node.inputs[2].default_value = 0.5
    hz = node(nt, 'ShaderNodeMapRange', clamp=True)
    lk(nt, sp.outputs['Z'], hz.inputs['Value'])
    hz.inputs['From Min'].default_value = hmin
    hz.inputs['From Max'].default_value = hmax
    c = node(nt, 'ShaderNodeCombineXYZ')
    lk(nt, nx, c.inputs[0])
    lk(nt, ny, c.inputs[1])
    lk(nt, hz.outputs['Result'], c.inputs[2])
    emission_out(nt, c.outputs[0])
    return m

def ao_material(distance=0.2, samples=16):
    m, nt = new_mat('forge_ao')
    ao = node(nt, 'ShaderNodeAmbientOcclusion', samples=samples, only_local=False)
    ao.inputs['Distance'].default_value = distance
    ao.inputs['Color'].default_value = (1, 1, 1, 1)
    emission_out(nt, ao.outputs['Color'])
    return m

def render_data(sc, path, mat, samples=16):
    # Exact data pass: no denoise, no view transform (EXR is scene-linear).
    gpu_cycles(sc, samples=samples, denoise=False, bounces=4)
    sc.render.filter_size = 1.0
    vl = sc.view_layers[0]
    vl.material_override = mat
    save(sc, path, exr=True)
    vl.material_override = None

def save(sc, path, exr=False, depth='16', mode='RGBA'):
    s = sc.render.image_settings
    if exr:
        s.file_format = 'OPEN_EXR'
        s.color_depth = '32'
        try:
            s.exr_codec = 'ZIP'
        except Exception:
            pass
    else:
        s.file_format = 'PNG'
        s.color_depth = depth
        s.compression = 15
    s.color_mode = mode
    sc.render.filepath = path
    bpy.ops.render.render(write_still=True)
    log('wrote', path)

def studio_world(sc, strength=1.0, hdri='studio.exr', rotation=0.0, visible=False):
    w = bpy.data.worlds.new('forge_world')
    sc.world = w
    try:
        w.use_nodes = True
    except Exception:
        pass
    nt = w.node_tree
    nt.nodes.clear()
    out = node(nt, 'ShaderNodeOutputWorld')
    bg = node(nt, 'ShaderNodeBackground')
    bg.inputs['Strength'].default_value = strength
    lk(nt, bg.outputs[0], out.inputs['Surface'])
    folder = bpy.utils.system_resource('DATAFILES', path='studiolights/world')
    path = os.path.join(folder, hdri) if folder else ''
    if path and os.path.isfile(path):
        env = node(nt, 'ShaderNodeTexEnvironment')
        env.image = bpy.data.images.load(path, check_existing=True)
        tc = node(nt, 'ShaderNodeTexCoord')
        mp = node(nt, 'ShaderNodeMapping')
        mp.inputs['Rotation'].default_value = (0.0, 0.0, rotation)
        lk(nt, tc.outputs['Generated'], mp.inputs['Vector'])
        lk(nt, mp.outputs[0], env.inputs['Vector'])
        lk(nt, env.outputs['Color'], bg.inputs['Color'])
        log('hdri', path)
    else:
        bg.inputs['Color'].default_value = (0.05, 0.05, 0.06, 1.0)
        log('hdri missing, flat world')
    try:
        w.cycles_visibility.camera = visible
    except Exception:
        pass
    return w

def area_light(name, loc, target, size, energy, color=(1.0, 1.0, 1.0), size_y=None):
    ld = bpy.data.lights.new(name, 'AREA')
    ld.energy = energy
    ld.color = color
    ld.shape = 'RECTANGLE'
    ld.size = size
    ld.size_y = size_y if size_y else size
    ob = bpy.data.objects.new(name, ld)
    ob.location = loc
    ob.rotation_euler = (Vector(target) - Vector(loc)).to_track_quat('-Z', 'Y').to_euler()
    link(ob)
    return ob

def outline(kind, w, h, r=0.0, n=5.0, segs=96):
    # Closed CCW outline of a rounded rect, pill, superellipse squircle or ellipse, centred at 0.
    pts = []
    if kind == 'pill':
        kind = 'rrect'
        r = min(w, h) * 0.5
    if kind == 'rrect':
        r = min(r, w * 0.5, h * 0.5)
        cx, cy = w * 0.5 - r, h * 0.5 - r
        for (sx, sy, a0) in ((1, 1, 0.0), (-1, 1, 90.0), (-1, -1, 180.0), (1, -1, 270.0)):
            for i in range(segs + 1):
                a = math.radians(a0 + 90.0 * i / segs)
                pts.append((sx * cx + r * math.cos(a), sy * cy + r * math.sin(a)))
    elif kind == 'squircle':
        m = segs * 4
        for i in range(m):
            t = 2.0 * math.pi * i / m
            c, s = math.cos(t), math.sin(t)
            x = math.copysign(abs(c) ** (2.0 / n), c) * w * 0.5
            y = math.copysign(abs(s) ** (2.0 / n), s) * h * 0.5
            pts.append((x, y))
    else:
        m = segs * 4
        for i in range(m):
            t = 2.0 * math.pi * i / m
            pts.append((math.cos(t) * w * 0.5, math.sin(t) * h * 0.5))
    clean = []
    for p in pts:
        if not clean or (abs(p[0] - clean[-1][0]) + abs(p[1] - clean[-1][1])) > 1e-7:
            clean.append(p)
    if len(clean) > 2 and abs(clean[0][0] - clean[-1][0]) + abs(clean[0][1] - clean[-1][1]) < 1e-7:
        clean.pop()
    return clean

def plate_mesh(name, pts, thick, bevel, segs=32, profile=0.5):
    # A slab from the outline with its top edge bevelled: flat top, smooth rounded rim.
    bm = bmesh.new()
    verts = [bm.verts.new((x, y, 0.0)) for (x, y) in pts]
    face = bm.faces.new(verts)
    bm.normal_update()
    if face.normal.z < 0:
        face.normal_flip()
    res = bmesh.ops.extrude_face_region(bm, geom=[face])
    top_verts = [g for g in res['geom'] if isinstance(g, bmesh.types.BMVert)]
    bmesh.ops.translate(bm, vec=(0.0, 0.0, thick), verts=top_verts)
    tops = [g for g in res['geom'] if isinstance(g, bmesh.types.BMFace)]
    bmesh.ops.delete(bm, geom=[face], context='FACES_ONLY')
    if bevel > 0:
        edges = list(tops[0].edges)
        bmesh.ops.bevel(bm, geom=edges, offset=bevel, offset_type='OFFSET', segments=segs,
                        profile=profile, affect='EDGES', clamp_overlap=True)
    bm.normal_update()
    big = max(bm.faces, key=lambda f: f.calc_area())
    for f in bm.faces:
        f.smooth = f is not big
    return mesh_obj(name, bm)

def done():
    print('FORGE-OK', flush=True)
"#;
