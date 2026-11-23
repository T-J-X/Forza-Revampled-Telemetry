//! The hero scene builder in bpy, kept as a Rust string constant (the repo holds no `.py`).
//! It runs after [`crate::recipes::PRELUDE`] and reads the scene JSON written by
//! [`super::Scene::render`]: camera, rigs (one per key azimuth), materials, parts and passes.
//!
//! Passes, all registered 1:1 to the frame:
//! - `beauty_<i>.png`: 16-bit straight RGBA, Standard view (exact colours, no AgX tone curve),
//!   transparent film with transparent glass, one per rig (rest key, then the G-swing keys).
//! - `data.exr`: world normal XY (×0.5+0.5) and height (from the height range) as emission.
//! - `ao.exr`, `rmh.exr` (roughness, metallic, height) and `emission.png` (lights off: only
//!   lume, LEDs and emitters, for an Add layer).
pub const HERO: &str = r#"
P = ARGS
MATS = {}

def M(nt, op, a, b=None, c=None):
    n = node(nt, 'ShaderNodeMath', operation=op)
    for i, v in enumerate((a, b, c)):
        if v is None:
            continue
        if isinstance(v, (int, float)):
            n.inputs[i].default_value = float(v)
        else:
            lk(nt, v, n.inputs[i])
    return n.outputs[0]

def tri(nt, x):
    # Triangle wave 0..1..0 with period 1.
    f = M(nt, 'FRACT', x)
    return M(nt, 'MULTIPLY', M(nt, 'ABSOLUTE', M(nt, 'SUBTRACT', f, 0.5)), 2.0)

def coords(nt):
    # Object-space position in px (parts are built round their own origin).
    tc = node(nt, 'ShaderNodeTexCoord')
    sp = node(nt, 'ShaderNodeSeparateXYZ')
    lk(nt, tc.outputs['Object'], sp.inputs[0])
    x = M(nt, 'MULTIPLY', sp.outputs['X'], 100.0)
    y = M(nt, 'MULTIPLY', sp.outputs['Y'], 100.0)
    z = M(nt, 'MULTIPLY', sp.outputs['Z'], 100.0)
    r = M(nt, 'SQRT', M(nt, 'ADD', M(nt, 'MULTIPLY', x, x), M(nt, 'MULTIPLY', y, y)))
    th = M(nt, 'ARCTAN2', y, x)
    return x, y, z, r, th

def vec3(nt, a, b, c):
    n = node(nt, 'ShaderNodeCombineXYZ')
    for i, v in enumerate((a, b, c)):
        if isinstance(v, (int, float)):
            n.inputs[i].default_value = float(v)
        else:
            lk(nt, v, n.inputs[i])
    return n.outputs[0]

def bump(nt, bs, height, strength, distance_px):
    b = node(nt, 'ShaderNodeBump')
    b.inputs['Strength'].default_value = strength
    b.inputs['Distance'].default_value = distance_px * 0.01
    lk(nt, height, b.inputs['Height'])
    lk(nt, b.outputs['Normal'], bs.inputs['Normal'])
    return b

def noise(nt, vec, scale, detail=4.0):
    n = node(nt, 'ShaderNodeTexNoise')
    n.inputs['Scale'].default_value = scale
    n.inputs['Detail'].default_value = detail
    lk(nt, vec, n.inputs['Vector'])
    return n.outputs[0]

def principled(name):
    m, nt = new_mat(name)
    bs = node(nt, 'ShaderNodeBsdfPrincipled')
    out = node(nt, 'ShaderNodeOutputMaterial')
    lk(nt, bs.outputs[0], out.inputs['Surface'])
    return m, nt, bs

def setin(bs, name, v):
    try:
        s = bs.inputs[name]
    except Exception:
        log('no input', name)
        return
    if isinstance(v, (list, tuple)) and len(v) == 3 and len(s.default_value) == 4:
        v = (v[0], v[1], v[2], 1.0)
    s.default_value = v

def finish_rough(nt, bs, base, var, vec, scale):
    # Roughness with a little variation so big faces never read as CG-flat.
    n = noise(nt, vec, scale, 6.0)
    r = M(nt, 'MULTIPLY_ADD', M(nt, 'SUBTRACT', n, 0.5), var, base)
    lk(nt, M(nt, 'MAXIMUM', r, 0.01), bs.inputs['Roughness'])

def mat_metal(d):
    m, nt, bs = principled(d['name'])
    setin(bs, 'Base Color', d.get('color', (0.9, 0.9, 0.9)))
    setin(bs, 'Metallic', d.get('metallic', 1.0))
    rough = d.get('rough', 0.2)
    setin(bs, 'Roughness', rough)
    x, y, z, r, th = coords(nt)
    fin = d.get('finish', 'polished')
    grain = max(d.get('grain', 1.0), 0.05)
    aniso = d.get('aniso', 0.0)
    if fin == 'linear':
        h = noise(nt, vec3(nt, M(nt, 'MULTIPLY', x, 0.004), y, 0.0), 1.0 / grain, 8.0)
        tang = vec3(nt, 1.0, 0.0, 0.0)
    elif fin == 'sunray':
        # Grain runs along the radius: streak coordinate = angle × reference radius.
        s = M(nt, 'MULTIPLY', th, 200.0)
        h = noise(nt, vec3(nt, s, M(nt, 'MULTIPLY', r, 0.004), 0.0), 1.0 / grain, 8.0)
        tang = vec3(nt, x, y, 0.0)
    elif fin == 'spun':
        h = noise(nt, vec3(nt, r, M(nt, 'MULTIPLY', th, 0.6), 0.0), 1.0 / grain, 8.0)
        tang = vec3(nt, M(nt, 'MULTIPLY', y, -1.0), x, 0.0)
    else:
        h = noise(nt, vec3(nt, x, y, z), 1.0 / grain, 6.0)
        tang = None
    if tang is not None and aniso > 0:
        setin(bs, 'Anisotropic', aniso)
        nm = node(nt, 'ShaderNodeVectorMath', operation='NORMALIZE')
        lk(nt, tang, nm.inputs[0])
        lk(nt, nm.outputs[0], bs.inputs['Tangent'])
    if fin == 'polished':
        finish_rough(nt, bs, rough, 0.02, vec3(nt, x, y, z), 0.02)
    else:
        lk(nt, M(nt, 'MULTIPLY_ADD', M(nt, 'SUBTRACT', h, 0.5), 0.12, rough), bs.inputs['Roughness'])
    hgt = h
    strength = 0.0 if fin == 'polished' else (0.25 if fin == 'bead' else 0.12)
    kn = d.get('knurl', 0.0)
    if kn > 0:
        s = M(nt, 'MULTIPLY', th, M(nt, 'DIVIDE', r, kn))
        rr = M(nt, 'DIVIDE', r, kn)
        if d.get('knurl_kind') == 'reeded':
            hgt = tri(nt, s)
        else:
            hgt = M(nt, 'MINIMUM', tri(nt, M(nt, 'ADD', s, rr)), tri(nt, M(nt, 'SUBTRACT', s, rr)))
        bump(nt, bs, hgt, 1.0, kn * 0.35)
    elif strength > 0:
        bump(nt, bs, hgt, strength, grain * 0.5)
    if d.get('coat', 0.0) > 0:
        setin(bs, 'Coat Weight', d['coat'])
        setin(bs, 'Coat Roughness', d.get('coat_rough', 0.05))
        if 'coat_tint' in d:
            setin(bs, 'Coat Tint', d['coat_tint'])
    return m

def mat_guilloche(d):
    m, nt, bs = principled(d['name'])
    setin(bs, 'Base Color', d.get('color', (0.8, 0.8, 0.8)))
    setin(bs, 'Metallic', d.get('metallic', 1.0))
    setin(bs, 'Roughness', d.get('rough', 0.2))
    x, y, z, r, th = coords(nt)
    p = max(d.get('pitch', 8.0), 0.5)
    pat = d.get('pattern', 'clous')
    if pat == 'rose':
        w = M(nt, 'ADD', M(nt, 'DIVIDE', r, p / 6.2831853), M(nt, 'MULTIPLY', M(nt, 'SINE', M(nt, 'MULTIPLY', th, 12.0)), 2.5))
        h = M(nt, 'MULTIPLY_ADD', M(nt, 'SINE', w), 0.5, 0.5)
    elif pat == 'wave':
        w = M(nt, 'ADD', M(nt, 'DIVIDE', y, p / 6.2831853), M(nt, 'MULTIPLY', M(nt, 'SINE', M(nt, 'DIVIDE', x, p)), 1.8))
        h = M(nt, 'MULTIPLY_ADD', M(nt, 'SINE', w), 0.5, 0.5)
    else:
        u = M(nt, 'DIVIDE', M(nt, 'ADD', x, y), p * 1.41421356)
        v = M(nt, 'DIVIDE', M(nt, 'SUBTRACT', x, y), p * 1.41421356)
        h = M(nt, 'MINIMUM', tri(nt, u), tri(nt, v))
    bump(nt, bs, h, 1.0, p * 0.2 * d.get('depth', 1.0))
    return m

def mat_carbon(d):
    m, nt, bs = principled(d['name'])
    setin(bs, 'Base Color', d.get('color', (0.02, 0.02, 0.02)))
    setin(bs, 'Metallic', 0.6)
    setin(bs, 'Roughness', d.get('rough', 0.3))
    x, y, z, r, th = coords(nt)
    t = max(d.get('tow', 8.0), 1.0)
    u = M(nt, 'FLOOR', M(nt, 'DIVIDE', x, t))
    v = M(nt, 'FLOOR', M(nt, 'DIVIDE', y, t))
    o = M(nt, 'LESS_THAN', M(nt, 'FLOORED_MODULO', M(nt, 'ADD', u, v), 4.0), 2.0)
    # Horizontal tows (o = 1) bulge across y, vertical tows across x.
    hy = M(nt, 'SINE', M(nt, 'MULTIPLY', M(nt, 'FRACT', M(nt, 'DIVIDE', y, t)), 3.14159265))
    hx = M(nt, 'SINE', M(nt, 'MULTIPLY', M(nt, 'FRACT', M(nt, 'DIVIDE', x, t)), 3.14159265))
    h = M(nt, 'ADD', M(nt, 'MULTIPLY', o, hy), M(nt, 'MULTIPLY', M(nt, 'SUBTRACT', 1.0, o), hx))
    bump(nt, bs, h, 0.6, t * 0.15)
    setin(bs, 'Anisotropic', 0.8)
    lk(nt, vec3(nt, o, M(nt, 'SUBTRACT', 1.0, o), 0.0), bs.inputs['Tangent'])
    setin(bs, 'Coat Weight', 1.0)
    setin(bs, 'Coat Roughness', 0.02)
    return m

def mat_dielectric(d):
    m, nt, bs = principled(d['name'])
    setin(bs, 'Base Color', d.get('color', (0.5, 0.5, 0.5)))
    setin(bs, 'Metallic', 0.0)
    setin(bs, 'Roughness', d.get('rough', 0.5))
    if d.get('coat', 0.0) > 0:
        setin(bs, 'Coat Weight', d['coat'])
        setin(bs, 'Coat Roughness', d.get('coat_rough', 0.03))
    if d.get('sss', 0.0) > 0:
        setin(bs, 'Subsurface Weight', d['sss'])
        setin(bs, 'Subsurface Radius', (0.02, 0.02, 0.02))
    if d.get('emit_strength', 0.0) > 0:
        setin(bs, 'Emission Color', d.get('emit', d.get('color')))
        setin(bs, 'Emission Strength', d['emit_strength'])
    return m

def mat_glass(d):
    m, nt, bs = principled(d['name'])
    setin(bs, 'Base Color', d.get('color', (1, 1, 1)))
    setin(bs, 'Metallic', 0.0)
    setin(bs, 'Roughness', d.get('rough', 0.0))
    setin(bs, 'IOR', d.get('ior', 1.5))
    setin(bs, 'Transmission Weight', 1.0)
    if d.get('film_nm', 0.0) > 0:
        setin(bs, 'Thin Film Thickness', d['film_nm'])
        setin(bs, 'Thin Film IOR', d.get('film_ior', 1.38))
    return m

def mat_led(d):
    m, nt, bs = principled(d['name'])
    c = d.get('color', (1, 0, 0))
    setin(bs, 'Base Color', [0.35 + 0.65 * v for v in c])
    setin(bs, 'Roughness', d.get('rough', 0.04))
    setin(bs, 'IOR', d.get('ior', 1.49))
    setin(bs, 'Transmission Weight', 0.85)
    setin(bs, 'Emission Color', d.get('emit', c))
    setin(bs, 'Emission Strength', d.get('emit_strength', 0.0))
    return m

def mat_leather(d):
    m, nt, bs = principled(d['name'])
    x, y, z, r, th = coords(nt)
    g = max(d.get('grain', 3.0), 0.3)
    tex = d.get('texture')
    if tex and os.path.isfile(tex):
        im = node(nt, 'ShaderNodeTexImage')
        im.image = bpy.data.images.load(tex, check_existing=True)
        im.extension = 'REPEAT'
        tile = d.get('tile_px', 512.0)
        lk(nt, vec3(nt, M(nt, 'DIVIDE', x, tile), M(nt, 'DIVIDE', y, tile), 0.0), im.inputs['Vector'])
        tint = node(nt, 'ShaderNodeMix', data_type='RGBA', blend_type='MULTIPLY')
        tint.inputs[0].default_value = 1.0
        lk(nt, im.outputs['Color'], tint.inputs[6])
        tint.inputs[7].default_value = tuple(list(d.get('tint', (1.0, 1.0, 1.0))) + [1.0])
        lk(nt, tint.outputs[2], bs.inputs['Base Color'])
        bw = node(nt, 'ShaderNodeRGBToBW')
        lk(nt, im.outputs['Color'], bw.inputs[0])
        bump(nt, bs, bw.outputs[0], 0.6, g * 0.4)
    else:
        vo = node(nt, 'ShaderNodeTexVoronoi', feature='DISTANCE_TO_EDGE')
        vo.inputs['Scale'].default_value = 1.0 / g
        lk(nt, vec3(nt, x, y, z), vo.inputs['Vector'])
        crease = node(nt, 'ShaderNodeMapRange', clamp=True)
        lk(nt, vo.outputs['Distance'], crease.inputs['Value'])
        crease.inputs['From Min'].default_value = 0.0
        crease.inputs['From Max'].default_value = 0.18
        n = noise(nt, vec3(nt, x, y, z), 0.02, 3.0)
        h = M(nt, 'ADD', crease.outputs['Result'], M(nt, 'MULTIPLY', n, 0.3))
        bump(nt, bs, h, 0.7, g * 0.35)
        c = d.get('color', (0.03, 0.025, 0.02))
        mix = node(nt, 'ShaderNodeMix', data_type='RGBA', blend_type='MIX')
        lk(nt, M(nt, 'MULTIPLY_ADD', n, 0.6, M(nt, 'MULTIPLY', crease.outputs['Result'], 0.4)), mix.inputs[0])
        mix.inputs[6].default_value = (c[0] * 0.55, c[1] * 0.55, c[2] * 0.55, 1.0)
        mix.inputs[7].default_value = (c[0] * 1.25, c[1] * 1.25, c[2] * 1.25, 1.0)
        lk(nt, mix.outputs[2], bs.inputs['Base Color'])
        rr = M(nt, 'MULTIPLY_ADD', crease.outputs['Result'], -0.18, d.get('rough', 0.5) + 0.1)
        lk(nt, rr, bs.inputs['Roughness'])
    setin(bs, 'Sheen Weight', d.get('sheen', 0.35))
    setin(bs, 'Sheen Roughness', 0.45)
    if tex and os.path.isfile(tex):
        setin(bs, 'Roughness', d.get('rough', 0.5))
    return m

def mat_alcantara(d):
    m, nt, bs = principled(d['name'])
    x, y, z, r, th = coords(nt)
    c = d.get('color', (0.03, 0.03, 0.03))
    n = noise(nt, vec3(nt, x, y, z), 0.6, 10.0)
    mix = node(nt, 'ShaderNodeMix', data_type='RGBA', blend_type='MIX')
    lk(nt, n, mix.inputs[0])
    mix.inputs[6].default_value = (c[0] * 0.7, c[1] * 0.7, c[2] * 0.7, 1.0)
    mix.inputs[7].default_value = (c[0] * 1.3, c[1] * 1.3, c[2] * 1.3, 1.0)
    lk(nt, mix.outputs[2], bs.inputs['Base Color'])
    setin(bs, 'Roughness', d.get('rough', 0.92))
    setin(bs, 'Sheen Weight', d.get('sheen', 1.0))
    setin(bs, 'Sheen Roughness', 0.35)
    bump(nt, bs, n, 0.15, 0.6)
    return m

def mat_emission(d):
    m, nt = new_mat(d['name'])
    em = node(nt, 'ShaderNodeEmission')
    c = d.get('color', (1, 1, 1))
    em.inputs['Color'].default_value = (c[0], c[1], c[2], 1.0)
    em.inputs['Strength'].default_value = d.get('emit_strength', 1.0)
    out = node(nt, 'ShaderNodeOutputMaterial')
    lk(nt, em.outputs[0], out.inputs['Surface'])
    return m

BUILDERS = {
    'metal': mat_metal, 'guilloche': mat_guilloche, 'carbon': mat_carbon,
    'dielectric': mat_dielectric, 'glass': mat_glass, 'led': mat_led,
    'leather': mat_leather, 'alcantara': mat_alcantara, 'emission': mat_emission,
}

def make_mat(d):
    m = BUILDERS.get(d.get('kind'), mat_dielectric)(d)
    m['forge_rough'] = float(d.get('rough', 0.5))
    m['forge_metal'] = 1.0 if d.get('kind') in ('metal', 'guilloche') else (0.6 if d.get('kind') == 'carbon' else 0.0)
    return m

def assign(ob, names):
    for n in names:
        ob.data.materials.append(MATS.get(n))

def build_lathe(p):
    prof = p['profile']
    segs = int(p.get('segs', 256))
    bm = bmesh.new()
    rings = []
    for (r, z, mi, sharp) in prof:
        if r < 1e-6:
            rings.append([bm.verts.new((0.0, 0.0, z))])
        else:
            rings.append([bm.verts.new((r * math.cos(2 * math.pi * k / segs), r * math.sin(2 * math.pi * k / segs), z)) for k in range(segs)])
    for i in range(len(rings) - 1):
        a, b = rings[i], rings[i + 1]
        mi = int(prof[i][2])
        for k in range(segs):
            k2 = (k + 1) % segs
            if len(a) == 1 and len(b) == 1:
                continue
            if len(a) == 1:
                vs = (a[0], b[k], b[k2])
            elif len(b) == 1:
                vs = (a[k], a[k2], b[0])
            else:
                vs = (a[k], a[k2], b[k2], b[k])
            try:
                f = bm.faces.new(vs)
                f.material_index = mi
                f.smooth = True
            except ValueError:
                pass
    bm.edges.ensure_lookup_table()
    for i, (r, z, mi, sharp) in enumerate(prof):
        if sharp and len(rings[i]) > 1 and 0 < i < len(prof) - 1:
            ring = set(rings[i])
            for e in bm.edges:
                if e.verts[0] in ring and e.verts[1] in ring:
                    e.smooth = False
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-7)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = mesh_obj(p['name'], bm)
    return ob

def build_extrude(p):
    pts = p['outline']
    cx = sum(q[0] for q in pts) / len(pts)
    cy = sum(q[1] for q in pts) / len(pts)
    rel = [(q[0] - cx, q[1] - cy) for q in pts]
    ob = plate_mesh(p['name'], rel, p['thick'], p['bevel'], segs=10, profile=0.5)
    ob.location.x += cx
    ob.location.y += cy
    return ob

def build_mesh(p):
    bm = bmesh.new()
    vs = [bm.verts.new(tuple(v)) for v in p['verts']]
    fm = p.get('fmats', [])
    for i, f in enumerate(p['faces']):
        try:
            face = bm.faces.new([vs[j] for j in f])
        except ValueError:
            continue
        face.material_index = int(fm[i]) if i < len(fm) else 0
        face.smooth = bool(p.get('smooth', False))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    for f in bm.faces:
        if f.material_index == 1 and f.normal.z < 0:
            f.normal_flip()
    return mesh_obj(p['name'], bm)

def build_part(p):
    t = p['type']
    ob = build_lathe(p) if t == 'lathe' else (build_extrude(p) if t == 'extrude' else build_mesh(p))
    ob.location = (ob.location.x + p['loc'][0], ob.location.y + p['loc'][1], ob.location.z + p['loc'][2])
    ob.rotation_euler = (0.0, 0.0, p.get('rot', 0.0))
    assign(ob, p['mats'])
    return ob

def camera(sc, f):
    resolution(sc, f['res'][0], f['res'][1])
    c = f['centre']
    if f['type'] == 'ORTHO':
        return ortho_cam(sc, f['ortho_scale'], center=(c[0], c[1]), z=40.0, clip_end=100.0)
    t = math.radians(f.get('tilt', 0.0))
    d = f['distance']
    return persp_cam(sc, (c[0], c[1] - math.sin(t) * d, math.cos(t) * d), (c[0], c[1], 0.0), lens=f['lens'])

LIGHTS = []

def rig(sc, r):
    for ob in LIGHTS:
        bpy.data.objects.remove(ob, do_unlink=True)
    LIGHTS.clear()
    for l in r['lights']:
        ob = area_light(l['name'], l['loc'], l['target'], l['size'][0], l['energy'], tuple(l['color']), size_y=l['size'][1])
        try:
            ob.visible_camera = False
        except Exception:
            pass
        LIGHTS.append(ob)
    studio_world(sc, r.get('hdri', 0.0), 'studio.exr', r.get('hdri_rot', 0.0), visible=False)
    sc.view_settings.exposure = r.get('exposure', 0.0)

def swap_rmh(objs, hmin, hmax):
    cache = {}
    saved = []
    for ob in objs:
        if ob.type != 'MESH':
            continue
        saved.append((ob, [s.material for s in ob.material_slots]))
        for s in ob.material_slots:
            src = s.material
            key = src.name if src else '-'
            if key not in cache:
                m, nt = new_mat('rmh_' + key)
                geo = node(nt, 'ShaderNodeNewGeometry')
                sp = node(nt, 'ShaderNodeSeparateXYZ')
                lk(nt, geo.outputs['Position'], sp.inputs[0])
                hz = node(nt, 'ShaderNodeMapRange', clamp=True)
                lk(nt, sp.outputs['Z'], hz.inputs['Value'])
                hz.inputs['From Min'].default_value = hmin
                hz.inputs['From Max'].default_value = hmax
                rough = src.get('forge_rough', 0.5) if src else 0.5
                metal = src.get('forge_metal', 0.0) if src else 0.0
                emission_out(nt, vec3(nt, rough, metal, hz.outputs['Result']))
                cache[key] = m
            s.material = cache[key]
    return saved

def restore(saved):
    for ob, mats in saved:
        for s, m in zip(ob.material_slots, mats):
            s.material = m

sc = reset()
for d in P['materials']:
    MATS[d['name']] = make_mat(d)
objs = [build_part(p) for p in P['parts']]
camera(sc, P['frame'])
dev = gpu_cycles(sc, samples=int(P.get('samples', 256)), denoise=True, bounces=16)
try:
    sc.cycles.film_transparent_glass = True
    sc.cycles.film_transparent_roughness = 0.1
except Exception as e:
    log('film glass', e)
sc.cycles.sample_clamp_indirect = 20.0
view(sc, 'Standard', None, 0.0)
log('device', dev, 'objects', len(objs))
passes = P['passes']
for i, r in enumerate(P['rigs']):
    rig(sc, r)
    if passes.get('beauty', True):
        save(sc, os.path.join(OUT, 'beauty_%d.png' % i), depth='16')
    if i == 0 and not passes.get('swing', True):
        break
rig(sc, P['rigs'][0])
hmin, hmax = P['height_range']
if passes.get('emission'):
    for ob in LIGHTS:
        ob.hide_render = True
    sc.world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0.0
    save(sc, os.path.join(OUT, 'emission.png'), depth='16')
    for ob in LIGHTS:
        ob.hide_render = False
if passes.get('data'):
    render_data(sc, os.path.join(OUT, 'data.exr'), data_material(hmin, hmax), samples=16)
if passes.get('ao'):
    render_data(sc, os.path.join(OUT, 'ao.exr'), ao_material(distance=P.get('ao_distance', 0.3)), samples=64)
if passes.get('rmh'):
    saved = swap_rmh(objs, hmin, hmax)
    gpu_cycles(sc, samples=16, denoise=False, bounces=2)
    sc.render.filter_size = 1.0
    save(sc, os.path.join(OUT, 'rmh.exr'), exr=True)
    restore(saved)
done()
"#;
