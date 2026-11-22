//! Classic and motorsport material recipes: tileable procedural materials (titanium, anodising,
//! guilloché, carbon, enamel, Alcantara, leather), radial dial faces and lathed bezel rings.
//!
//! Tileability: every pattern is periodic on the unit UV square. Lattice patterns use integer
//! frequencies; noise is sampled on a 4D torus `(cos 2πu, sin 2πu, cos 2πv, sin 2πv)·r`, so the
//! left edge equals the right edge exactly. The lit render uses only the world (light at
//! infinity) with an orthographic camera, so the shading depends only on the local surface and
//! the lit image tiles too.

/// Node-expression helpers: an `E` wrapper turns `+ - * / **` into Math nodes so patterns read
/// like formulas, plus torus noise and torus Voronoi.
pub const EXPR_HELPERS: &str = r#"
NT = None
TAU = 2.0 * math.pi

class E:
    def __init__(self, s):
        self.s = s
    def _b(self, op, o, rev=False):
        n = node(NT, 'ShaderNodeMath', operation=op)
        a, b = (o, self) if rev else (self, o)
        for i, x in enumerate((a, b)):
            if isinstance(x, E):
                lk(NT, x.s, n.inputs[i])
            else:
                n.inputs[i].default_value = float(x)
        return E(n.outputs[0])
    def __add__(self, o): return self._b('ADD', o)
    def __radd__(self, o): return self._b('ADD', o, True)
    def __sub__(self, o): return self._b('SUBTRACT', o)
    def __rsub__(self, o): return self._b('SUBTRACT', o, True)
    def __mul__(self, o): return self._b('MULTIPLY', o)
    def __rmul__(self, o): return self._b('MULTIPLY', o, True)
    def __truediv__(self, o): return self._b('DIVIDE', o)
    def __rtruediv__(self, o): return self._b('DIVIDE', o, True)
    def __pow__(self, o): return self._b('POWER', o)

def un(op, x):
    n = node(NT, 'ShaderNodeMath', operation=op)
    lk(NT, x.s, n.inputs[0])
    return E(n.outputs[0])

def sin_(x): return un('SINE', x)
def cos_(x): return un('COSINE', x)
def abs_(x): return un('ABSOLUTE', x)
def fract_(x): return un('FRACT', x)
def floor_(x): return un('FLOOR', x)
def max_(a, b): return a._b('MAXIMUM', b)
def min_(a, b): return a._b('MINIMUM', b)
def mod_(a, b): return a._b('FLOORED_MODULO', b)
def lt_(a, b): return a._b('LESS_THAN', b)
def clamp01(x):
    n = node(NT, 'ShaderNodeClamp')
    lk(NT, x.s, n.inputs['Value'])
    return E(n.outputs[0])
def sstep(e0, e1, x):
    n = node(NT, 'ShaderNodeMapRange', interpolation_type='SMOOTHSTEP', clamp=True)
    lk(NT, x.s, n.inputs['Value'])
    n.inputs['From Min'].default_value = e0
    n.inputs['From Max'].default_value = e1
    return E(n.outputs['Result'])

def uv():
    tc = node(NT, 'ShaderNodeTexCoord')
    s = node(NT, 'ShaderNodeSeparateXYZ')
    lk(NT, tc.outputs['UV'], s.inputs[0])
    return E(s.outputs['X']), E(s.outputs['Y'])

def torus(u, v, su, sv, off=0.0):
    a = u * TAU
    b = v * TAU
    ku, kv = su / TAU, sv / TAU
    c = node(NT, 'ShaderNodeCombineXYZ')
    lk(NT, (cos_(a) * ku + off).s, c.inputs[0])
    lk(NT, (sin_(a) * ku).s, c.inputs[1])
    lk(NT, (cos_(b) * kv).s, c.inputs[2])
    return c.outputs[0], (sin_(b) * kv + off * 0.37)

def tnoise(u, v, su, sv, detail=3.0, rough=0.5, off=0.0):
    vec, w = torus(u, v, su, sv, off)
    n = node(NT, 'ShaderNodeTexNoise', noise_dimensions='4D')
    n.inputs['Scale'].default_value = 1.0
    n.inputs['Detail'].default_value = detail
    n.inputs['Roughness'].default_value = rough
    lk(NT, vec, n.inputs['Vector'])
    lk(NT, w.s, n.inputs['W'])
    return E(n.outputs['Fac'])

def tvoronoi(u, v, cells, feature='F1', off=0.0, randomness=1.0):
    vec, w = torus(u, v, cells, cells, off)
    n = node(NT, 'ShaderNodeTexVoronoi', voronoi_dimensions='4D', feature=feature)
    n.inputs['Scale'].default_value = 1.0
    try:
        n.inputs['Randomness'].default_value = randomness
    except Exception:
        pass
    lk(NT, vec, n.inputs['Vector'])
    lk(NT, w.s, n.inputs['W'])
    return n

def E_const(x):
    n = node(NT, 'ShaderNodeValue')
    n.outputs[0].default_value = float(x)
    return E(n.outputs[0])

def col(cr, cg, cb, k):
    k = k if isinstance(k, E) else E_const(k)
    # colour (constants) scaled by expression k
    return (k * cr, k * cg, k * cb)

def lit_world(sc, strength=1.0):
    # Light at infinity only (so ortho top-down renders tile): a bright overhead softbox with a
    # soft edge, a dimmer ring of light lower down and a dark floor. Normal detail tilts the
    # reflection across the softbox edge, which is what makes brushing and guilloche read.
    global NT
    w = bpy.data.worlds.new('forge_lit')
    sc.world = w
    try:
        w.use_nodes = True
    except Exception:
        pass
    NT = w.node_tree
    NT.nodes.clear()
    out = node(NT, 'ShaderNodeOutputWorld')
    bg = node(NT, 'ShaderNodeBackground')
    tc = node(NT, 'ShaderNodeTexCoord')
    sp = node(NT, 'ShaderNodeSeparateXYZ')
    lk(NT, tc.outputs['Generated'], sp.inputs[0])
    dz = E(sp.outputs['Z'])
    dx = E(sp.outputs['X'])
    box = sstep(0.80, 0.95, dz) * (0.8 + 0.2 * sstep(-0.3, 0.3, dx))
    ring = sstep(0.25, 0.5, dz) * (1.0 - sstep(0.62, 0.8, dz))
    L = 0.05 + 2.4 * box + 0.55 * ring
    lk(NT, (L * strength).s, bg.inputs['Strength'])
    bg.inputs['Color'].default_value = (1.0, 1.0, 1.0, 1.0)
    lk(NT, bg.outputs[0], out.inputs['Surface'])
    return w

def finish(mode, albedo, height, rough, metal, bump_dist=0.002, clearcoat=0.0):
    out = node(NT, 'ShaderNodeOutputMaterial')
    if mode == 'albedo':
        c = node(NT, 'ShaderNodeCombineXYZ')
        for i in range(3):
            lk(NT, albedo[i].s, c.inputs[i])
        emission_out(NT, c.outputs[0]).id_data
        NT.nodes.remove(out)
        return
    if mode == 'hrm':
        c = node(NT, 'ShaderNodeCombineXYZ')
        lk(NT, height.s, c.inputs[0])
        lk(NT, rough.s, c.inputs[1])
        lk(NT, metal.s, c.inputs[2])
        emission_out(NT, c.outputs[0])
        NT.nodes.remove(out)
        return
    p = node(NT, 'ShaderNodeBsdfPrincipled')
    cc = node(NT, 'ShaderNodeCombineColor')
    for i in range(3):
        lk(NT, albedo[i].s, cc.inputs[i])
    lk(NT, cc.outputs[0], p.inputs['Base Color'])
    lk(NT, rough.s, p.inputs['Roughness'])
    lk(NT, metal.s, p.inputs['Metallic'])
    bump = node(NT, 'ShaderNodeBump')
    bump.inputs['Distance'].default_value = bump_dist
    lk(NT, height.s, bump.inputs['Height'])
    lk(NT, bump.outputs['Normal'], p.inputs['Normal'])
    try:
        p.inputs['Coat Weight'].default_value = clearcoat
        p.inputs['Coat Roughness'].default_value = 0.04
    except Exception:
        pass
    lk(NT, p.outputs[0], out.inputs['Surface'])
"#;

/// Tileable material plates. Params: `materials: [names]`, `size`, `samples`, `seed`.
/// Writes `<name>_albedo.exr`, `<name>_hrm.exr` (height, roughness, metallic) and
/// `<name>_lit.png` (world-lit, tileable).
pub const MATERIALS: &str = r#"
def pattern(name, mode):
    global NT
    m, NT = new_mat(name + '_' + mode)
    u, v = uv()
    if name == 'brushed_titanium':
        h = 0.5 + 0.7 * (tnoise(u, v, 5, 900, 2.0, 0.5) - 0.5) + 0.3 * (tnoise(u, v, 2, 70, 3.0, 0.5, 3.0) - 0.5)
        k = 0.9 + 0.2 * h
        finish(mode, col(0.50, 0.51, 0.53, k), h, 0.26 + 0.12 * (h - 0.5), E_const(1.0), 0.0008)
    elif name == 'anodised_black':
        h = 0.5 + 0.5 * (tnoise(u, v, 220, 220, 3.0, 0.6) - 0.5)
        finish(mode, col(0.022, 0.022, 0.025, 0.9 + 0.2 * h), h, 0.36 + 0.06 * (h - 0.5), E_const(1.0), 0.0004)
    elif name == 'anodised_blue':
        h = 0.5 + 0.5 * (tnoise(u, v, 220, 220, 3.0, 0.6, 5.0) - 0.5)
        finish(mode, col(0.03, 0.13, 0.42, 0.9 + 0.2 * h), h, 0.33 + 0.06 * (h - 0.5), E_const(1.0), 0.0004)
    elif name == 'guilloche_clous':
        N = 40.0
        a = (u + v) * N
        b = (u - v) * N
        fa = abs_(fract_(a) - 0.5)
        fb = abs_(fract_(b) - 0.5)
        h = 1.0 - 2.0 * max_(fa, fb) + 0.03 * sin_(a * (TAU * 9.0))
        finish(mode, col(0.74, 0.74, 0.76, 0.95 + 0.05 * h), h, E_const(0.17), E_const(1.0), 0.004)
    elif name == 'guilloche_barleycorn':
        ph = u * 30.0 + 0.22 * abs_(sin_(v * (math.pi * 60.0)))
        h = 0.5 + 0.5 * cos_(ph * TAU) * (0.75 + 0.25 * cos_(v * (TAU * 60.0)))
        finish(mode, col(0.72, 0.72, 0.74, 0.95 + 0.05 * h), h, E_const(0.16), E_const(1.0), 0.003)
    elif name == 'guilloche_wave':
        ph = v * 56.0 + 1.1 * sin_(u * (TAU * 3.0)) + 0.35 * sin_(u * (TAU * 7.0))
        h = 0.5 + 0.5 * sin_(ph * TAU)
        k = 0.7 + 0.3 * h
        finish(mode, col(0.035, 0.09, 0.30, k), h, E_const(0.1), E_const(0.55), 0.003, clearcoat=1.0)
    elif name == 'carbon_twill':
        N = 24.0
        i = floor_(u * N)
        j = floor_(v * N)
        fu = fract_(u * N)
        fv = fract_(v * N)
        horiz = lt_(mod_(i + j, E_const(4.0)), E_const(2.0))
        hh = sin_(fv * math.pi) ** 0.5
        hv = sin_(fu * math.pi) ** 0.5
        sh = tnoise(u, v, 6, 1400, 2.0, 0.5)
        sv = tnoise(u, v, 1400, 6, 2.0, 0.5, 2.0)
        tow = horiz * hh + (1.0 - horiz) * hv
        streak = horiz * sh + (1.0 - horiz) * sv
        h = tow * (0.85 + 0.3 * (streak - 0.5))
        k = 0.6 + 0.8 * streak
        finish(mode, col(0.02, 0.02, 0.022, k), h, 0.2 + 0.08 * (streak - 0.5), E_const(0.0), 0.003, clearcoat=1.0)
    elif name == 'carbon_forged':
        vo = tvoronoi(u, v, 14, 'F1')
        cell = node(NT, 'ShaderNodeSeparateColor')
        lk(NT, vo.outputs['Color'], cell.inputs[0])
        r = E(cell.outputs[0])
        ed = tvoronoi(u, v, 14, 'DISTANCE_TO_EDGE')
        edge = sstep(0.0, 0.06, E(ed.outputs['Distance']))
        fib = tnoise(u, v, 600, 600, 2.0, 0.5, 4.0)
        h = 0.6 * edge + 0.25 * fib + 0.15 * r
        k = 0.5 + 1.4 * r * (0.8 + 0.4 * fib)
        finish(mode, col(0.022, 0.022, 0.024, k), h, 0.18 + 0.2 * r, E_const(0.0), 0.002, clearcoat=1.0)
    elif name == 'enamel_white':
        h = 0.5 + 0.35 * (tnoise(u, v, 48, 48, 5.0, 0.55) - 0.5)
        finish(mode, col(0.86, 0.85, 0.81, 0.98 + 0.04 * h), h, E_const(0.07), E_const(0.0), 0.0006, clearcoat=1.0)
    elif name == 'alcantara':
        h = 0.5 + 0.55 * (tnoise(u, v, 520, 520, 2.0, 0.6) - 0.5) + 0.25 * (tnoise(u, v, 26, 26, 4.0, 0.5, 6.0) - 0.5)
        finish(mode, col(0.036, 0.036, 0.038, 0.75 + 0.5 * h), h, E_const(0.93), E_const(0.0), 0.0012)
    elif name == 'leather':
        ed = tvoronoi(u, v, 70, 'DISTANCE_TO_EDGE', 1.0)
        crease = sstep(0.0, 0.09, E(ed.outputs['Distance']))
        n2 = tnoise(u, v, 300, 300, 3.0, 0.55, 7.0)
        h = 0.75 * crease + 0.25 * n2
        finish(mode, col(0.032, 0.028, 0.025, 0.75 + 0.45 * h), h, 0.5 + 0.12 * (1.0 - crease), E_const(0.0), 0.0015, clearcoat=0.15)
    else:
        raise RuntimeError('unknown material ' + name)
    return m

size = int(ARGS.get('size', 2048))
for name in ARGS['materials']:
    for mode in ('albedo', 'hrm', 'lit'):
        sc = reset()
        resolution(sc, size, size)
        bpy.ops.mesh.primitive_plane_add(size=1.0)
        plane = bpy.context.active_object
        plane.data.materials.append(pattern(name, mode))
        ortho_cam(sc, 1.0)
        if mode == 'lit':
            gpu_cycles(sc, samples=ARGS.get('samples', 256), denoise=True)
            sc.render.film_transparent = False
            view(sc, 'AgX', look='AgX - Punchy')
            lit_world(sc)
            save(sc, os.path.join(OUT, name + '_lit.png'), depth='16', mode='RGB')
        else:
            gpu_cycles(sc, samples=16, denoise=False, bounces=2)
            sc.render.filter_size = 1.0
            save(sc, os.path.join(OUT, name + '_' + mode + '.exr'), exr=True)
done()
"#;

/// Radial dial faces (not tileable): sunburst, rose-engine guilloché, grand-feu enamel.
/// Params: `dials: [names]`, `size`, `samples`. Writes albedo / hrm EXRs and a lit PNG.
pub const DIALS: &str = r#"
def dial(name, mode):
    global NT
    m, NT = new_mat(name + '_' + mode)
    u, v = uv()
    x = u - 0.5
    y = v - 0.5
    r = (x * x + y * y) ** 0.5
    at = node(NT, 'ShaderNodeMath', operation='ARCTAN2')
    lk(NT, y.s, at.inputs[0])
    lk(NT, x.s, at.inputs[1])
    ang = E(at.outputs[0])
    disc = 1.0 - sstep(0.495, 0.5, r)
    if name == 'dial_sunburst_slate':
        # Fine radial brushing: noise that varies only with angle (sampled on a circle) plus grain.
        c = node(NT, 'ShaderNodeCombineXYZ')
        lk(NT, (cos_(ang) * 380.0).s, c.inputs[0])
        lk(NT, (sin_(ang) * 380.0).s, c.inputs[1])
        nz = node(NT, 'ShaderNodeTexNoise')
        nz.inputs['Scale'].default_value = 1.0
        nz.inputs['Detail'].default_value = 2.0
        lk(NT, c.outputs[0], nz.inputs['Vector'])
        s = E(nz.outputs['Fac'])
        h = (0.5 + 0.8 * (s - 0.5)) * disc
        finish(mode, col(0.11, 0.13, 0.16, 0.85 + 0.3 * s), h, 0.22 + 0.1 * (s - 0.5), E_const(1.0), 0.0006)
    elif name == 'dial_guilloche_rose':
        # Rose-engine: concentric rings modulated by a 24-petal angular wave.
        ph = r * 90.0 + 0.9 * sin_(ang * 24.0) * (0.3 + r)
        h = (0.5 + 0.5 * sin_(ph * TAU)) * disc
        finish(mode, col(0.70, 0.70, 0.72, 0.9 + 0.1 * h), h, E_const(0.15), E_const(1.0), 0.0025)
    elif name == 'dial_enamel_white':
        nz = node(NT, 'ShaderNodeTexNoise')
        nz.inputs['Scale'].default_value = 60.0
        nz.inputs['Detail'].default_value = 5.0
        n = E(nz.outputs['Fac'])
        dome = 1.0 - r * r * 1.6
        h = (0.4 * dome + 0.08 * (n - 0.5)) * disc
        finish(mode, col(0.88, 0.87, 0.83, 1.0), h, E_const(0.06), E_const(0.0), 0.002, clearcoat=1.0)
    else:
        raise RuntimeError('unknown dial ' + name)
    return m

size = int(ARGS.get('size', 2048))
for name in ARGS['dials']:
    for mode in ('albedo', 'hrm', 'lit'):
        sc = reset()
        resolution(sc, size, size)
        bpy.ops.mesh.primitive_plane_add(size=1.0)
        plane = bpy.context.active_object
        plane.data.materials.append(dial(name, mode))
        ortho_cam(sc, 1.0)
        if mode == 'lit':
            gpu_cycles(sc, samples=ARGS.get('samples', 256), denoise=True)
            sc.render.film_transparent = True
            view(sc, 'AgX', look='AgX - Punchy')
            studio_world(sc, strength=0.7, hdri='studio.exr', rotation=0.9, visible=False)
            area_light('key', (-1.2, 1.4, 1.6), (0, 0, 0), 1.2, 160.0)
            area_light('fill', (1.6, -0.8, 1.2), (0, 0, 0), 1.6, 60.0, color=(0.85, 0.9, 1.0))
            save(sc, os.path.join(OUT, name + '_lit.png'), depth='16', mode='RGBA')
        else:
            gpu_cycles(sc, samples=16, denoise=False, bounces=2)
            sc.render.filter_size = 1.0
            save(sc, os.path.join(OUT, name + '_' + mode + '.exr'), exr=True)
done()
"#;

/// Bezel rings lathed from a profile: data pass (normal + height), AO pass and lit renders in
/// polished steel and black chrome. Params: `bezels: [{name, inner, outer, height, style,
/// knurls}]`, `size`, `samples`.
pub const BEZELS: &str = r#"
def ring_mesh(inner, outer, height, style, knurls, segs=720, prof=64):
    bm = bmesh.new()
    rows = []
    for i in range(prof + 1):
        t = i / prof
        r = inner + (outer - inner) * t
        # profile: inner chamfer, crowned top, outer step down
        z = height * (0.55 + 0.45 * math.sin(math.pi * min(1.0, t * 1.08)) ** 0.6)
        if t < 0.08:
            z = height * (0.2 + 0.8 * (t / 0.08) ** 0.7) * 0.75
        if t > 0.86:
            z = height * 0.55 * (1.0 - (t - 0.86) / 0.14) ** 0.5 + height * 0.05
        row = []
        for j in range(segs):
            a = 2.0 * math.pi * j / segs
            zz = z
            if style == 'knurled' and 0.25 < t < 0.8:
                zz += height * 0.06 * (0.5 + 0.5 * math.cos(a * knurls)) ** 3
            if style == 'fluted' and t > 0.62:
                zz += height * 0.08 * abs(math.sin(a * knurls * 0.5))
            row.append(bm.verts.new((r * math.cos(a), r * math.sin(a), zz)))
        rows.append(row)
    for i in range(prof):
        for j in range(segs):
            f = bm.faces.new((rows[i][j], rows[i][(j + 1) % segs], rows[i + 1][(j + 1) % segs], rows[i + 1][j]))
            f.smooth = True
    bm.normal_update()
    for f in bm.faces:
        if f.normal.z < 0:
            f.normal_flip()
    return mesh_obj('bezel', bm)

size = int(ARGS.get('size', 2048))
for b in ARGS['bezels']:
    sc = reset()
    resolution(sc, size, size)
    ob = ring_mesh(b['inner'], b['outer'], b['height'], b['style'], b.get('knurls', 0))
    ortho_cam(sc, 2.04)
    render_data(sc, os.path.join(OUT, b['name'] + '_data.exr'), data_material(0.0, b['height'] * 1.1), samples=16)
    bpy.ops.mesh.primitive_plane_add(size=6.0, location=(0, 0, 0.0))
    floor = bpy.context.active_object
    floor.visible_camera = False
    gpu_cycles(sc, samples=64, denoise=True, bounces=4)
    sc.render.filter_size = 1.0
    vl = sc.view_layers[0]
    vl.material_override = ao_material(distance=b['height'] * 1.5, samples=24)
    save(sc, os.path.join(OUT, b['name'] + '_ao.exr'), exr=True)
    vl.material_override = None
    bpy.data.objects.remove(floor)
    for (finish_name, colr, rough) in (('steel', (0.9, 0.9, 0.92), 0.08), ('blackchrome', (0.05, 0.05, 0.055), 0.06)):
        m, nt = new_mat(finish_name)
        p = node(nt, 'ShaderNodeBsdfPrincipled')
        p.inputs['Base Color'].default_value = (colr[0], colr[1], colr[2], 1.0)
        p.inputs['Metallic'].default_value = 1.0
        p.inputs['Roughness'].default_value = rough
        try:
            p.inputs['Anisotropic'].default_value = 0.6
        except Exception:
            pass
        o = node(nt, 'ShaderNodeOutputMaterial')
        lk(nt, p.outputs[0], o.inputs['Surface'])
        ob.data.materials.clear()
        ob.data.materials.append(m)
        gpu_cycles(sc, samples=ARGS.get('samples', 256), denoise=True)
        view(sc, 'AgX', look='AgX - Punchy')
        studio_world(sc, strength=0.9, hdri='studio.exr', rotation=0.9, visible=False)
        if not any(o.type == 'LIGHT' for o in sc.objects):
            area_light('key', (-2.0, 2.4, 2.6), (0, 0, 0), 1.5, 260.0)
            area_light('strip', (2.6, -1.0, 1.0), (0, 0, 0), 0.25, 160.0, size_y=3.0)
        save(sc, os.path.join(OUT, b['name'] + '_' + finish_name + '.png'), depth='16', mode='RGBA')
done()
"#;
