//! Abstract fluid recipes: liquid-metal and ferrofluid flipbooks and studio matcaps.
//!
//! Loops are exact: the surface displacement samples 4D noise along a closed circle in noise
//! space, `(p*s + R(cos θ, sin θ, 0), W = R sin θ)` with θ = 2π·frame/frames, so frame N equals
//! frame 0 and every step moves the same distance.

/// Helpers shared by the fluid recipes: a Geometry Nodes displacement tree driven by looped
/// 4D noise (and optional Voronoi spikes), a view-space data material, and studio lighting.
pub const FLUID_HELPERS: &str = r#"
def gn_displace(ob, layers, spikes=None, axis='NORMAL'):
    # layers: [(scale, amp)], spikes: (scale, amp, radius) Voronoi cones. Returns the node handles.
    mod = ob.modifiers.new('disp', 'NODES')
    ng = bpy.data.node_groups.new('disp', 'GeometryNodeTree')
    ng.interface.new_socket('Geometry', in_out='INPUT', socket_type='NodeSocketGeometry')
    ng.interface.new_socket('Geometry', in_out='OUTPUT', socket_type='NodeSocketGeometry')
    mod.node_group = ng
    nt = ng
    gi = node(nt, 'NodeGroupInput')
    go = node(nt, 'NodeGroupOutput')
    pos = node(nt, 'GeometryNodeInputPosition')
    nrm = node(nt, 'GeometryNodeInputNormal')
    handles = []
    total = None
    for (scale, amp) in layers:
        sc_ = node(nt, 'ShaderNodeVectorMath', operation='SCALE')
        lk(nt, pos.outputs[0], sc_.inputs[0])
        sc_.inputs['Scale'].default_value = scale
        add = node(nt, 'ShaderNodeVectorMath', operation='ADD')
        lk(nt, sc_.outputs[0], add.inputs[0])
        nz = node(nt, 'ShaderNodeTexNoise', noise_dimensions='4D')
        nz.inputs['Scale'].default_value = 1.0
        nz.inputs['Detail'].default_value = 2.0
        nz.inputs['Roughness'].default_value = 0.45
        lk(nt, add.outputs[0], nz.inputs['Vector'])
        d = math_node(nt, 'SUBTRACT', nz.outputs['Fac'], vb=0.5)
        d = math_node(nt, 'MULTIPLY', d, vb=amp)
        total = d if total is None else math_node(nt, 'ADD', total, d)
        handles.append((add, nz))
    spike_h = None
    if spikes:
        # A static spike lattice (3D Voronoi on the rest position, like the stable Rosensweig
        # pattern) whose height breathes with looped 4D noise, on upward-facing surface only.
        (s_scale, s_amp, s_rad) = spikes
        vo = node(nt, 'ShaderNodeTexVoronoi', voronoi_dimensions='3D', feature='F1', distance='EUCLIDEAN')
        vo.inputs['Scale'].default_value = s_scale
        vo.inputs['Randomness'].default_value = 0.45
        lk(nt, pos.outputs[0], vo.inputs['Vector'])
        t = math_node(nt, 'DIVIDE', vo.outputs['Distance'], vb=s_rad)
        t = math_node(nt, 'MINIMUM', t, vb=1.0)
        t = math_node(nt, 'SUBTRACT', None, t, va=1.0)
        t = math_node(nt, 'POWER', t, vb=2.4)
        sc_ = node(nt, 'ShaderNodeVectorMath', operation='SCALE')
        lk(nt, pos.outputs[0], sc_.inputs[0])
        sc_.inputs['Scale'].default_value = 1.6
        add = node(nt, 'ShaderNodeVectorMath', operation='ADD')
        lk(nt, sc_.outputs[0], add.inputs[0])
        br = node(nt, 'ShaderNodeTexNoise', noise_dimensions='4D')
        br.inputs['Scale'].default_value = 1.0
        br.inputs['Detail'].default_value = 1.0
        lk(nt, add.outputs[0], br.inputs['Vector'])
        breathe = math_node(nt, 'MULTIPLY_ADD', br.outputs['Fac'])
        breathe.node.inputs[1].default_value = 1.6
        breathe.node.inputs[2].default_value = -0.35
        breathe = math_node(nt, 'MAXIMUM', breathe, vb=0.15)
        sz = node(nt, 'ShaderNodeSeparateXYZ')
        lk(nt, nrm.outputs[0], sz.inputs[0])
        # Spikes stand only on the upward-facing crown (normal z above ~0.55), never on the wall.
        up = node(nt, 'ShaderNodeMapRange', interpolation_type='SMOOTHSTEP', clamp=True)
        lk(nt, sz.outputs['Z'], up.inputs['Value'])
        up.inputs['From Min'].default_value = 0.5
        up.inputs['From Max'].default_value = 0.8
        up = up.outputs['Result']
        t = math_node(nt, 'MULTIPLY', t, breathe)
        t = math_node(nt, 'MULTIPLY', t, up)
        spike_h = math_node(nt, 'MULTIPLY', t, vb=s_amp)
        total = spike_h if total is None else math_node(nt, 'ADD', total, spike_h)
        handles.append((add, br))
    store = node(nt, 'GeometryNodeStoreNamedAttribute', data_type='FLOAT', domain='POINT')
    store.inputs['Name'].default_value = 'disp'
    setp = node(nt, 'GeometryNodeSetPosition')
    off = node(nt, 'ShaderNodeVectorMath', operation='SCALE')
    if axis == 'Z':
        zc = node(nt, 'ShaderNodeCombineXYZ')
        zc.inputs[2].default_value = 1.0
        lk(nt, zc.outputs[0], off.inputs[0])
    else:
        lk(nt, nrm.outputs[0], off.inputs[0])
    lk(nt, total, off.inputs['Scale'])
    lk(nt, gi.outputs[0], setp.inputs['Geometry'])
    lk(nt, off.outputs[0], setp.inputs['Offset'])
    lk(nt, setp.outputs[0], store.inputs['Geometry'])
    lk(nt, total, store.inputs['Value'])
    lk(nt, store.outputs[0], go.inputs[0])
    return handles

def set_loop(handles, theta, radius):
    for (add, tex) in handles:
        add.inputs[1].default_value = (radius * math.cos(theta), radius * math.sin(theta), 0.0)
        tex.inputs['W'].default_value = radius * math.sin(theta)

def view_data_material(dmin, dmax):
    # Emission = (camera-space N.xy *0.5+0.5, normalised displacement attribute).
    m, nt = new_mat('forge_view_data')
    geo = node(nt, 'ShaderNodeNewGeometry')
    vt = node(nt, 'ShaderNodeVectorTransform', vector_type='NORMAL', convert_from='WORLD', convert_to='CAMERA')
    lk(nt, geo.outputs['Normal'], vt.inputs[0])
    sn = node(nt, 'ShaderNodeSeparateXYZ')
    lk(nt, vt.outputs[0], sn.inputs[0])
    nx = math_node(nt, 'MULTIPLY_ADD', sn.outputs['X'])
    nx.node.inputs[1].default_value = 0.5
    nx.node.inputs[2].default_value = 0.5
    ny = math_node(nt, 'MULTIPLY_ADD', sn.outputs['Y'])
    ny.node.inputs[1].default_value = 0.5
    ny.node.inputs[2].default_value = 0.5
    at = node(nt, 'ShaderNodeAttribute', attribute_name='disp', attribute_type='GEOMETRY')
    mr = node(nt, 'ShaderNodeMapRange', clamp=True)
    lk(nt, at.outputs['Fac'], mr.inputs['Value'])
    mr.inputs['From Min'].default_value = dmin
    mr.inputs['From Max'].default_value = dmax
    c = node(nt, 'ShaderNodeCombineXYZ')
    lk(nt, nx, c.inputs[0])
    lk(nt, ny, c.inputs[1])
    lk(nt, mr.outputs['Result'], c.inputs[2])
    emission_out(nt, c.outputs[0])
    return m

def metal_material(name, color, roughness, coat=0.0, metallic=1.0):
    m, nt = new_mat(name)
    p = node(nt, 'ShaderNodeBsdfPrincipled')
    p.inputs['Base Color'].default_value = (color[0], color[1], color[2], 1.0)
    p.inputs['Metallic'].default_value = metallic
    p.inputs['Roughness'].default_value = roughness
    try:
        p.inputs['Coat Weight'].default_value = coat
        p.inputs['Coat Roughness'].default_value = 0.03
    except Exception:
        pass
    o = node(nt, 'ShaderNodeOutputMaterial')
    lk(nt, p.outputs[0], o.inputs['Surface'])
    return m

def studio(sc, strength=1.0, key=600.0, zenith=False):
    # A procedural photo studio at infinity for chrome to reflect: a big overhead softbox, a bright
    # horizon band, two vertical strip boxes, a cool rim and a black floor. Plus area lights for
    # crisp speculars. Mirror metal is only as good as what it reflects.
    w = bpy.data.worlds.new('forge_studio')
    sc.world = w
    try:
        w.use_nodes = True
    except Exception:
        pass
    nt = w.node_tree
    nt.nodes.clear()
    out = node(nt, 'ShaderNodeOutputWorld')
    bg = node(nt, 'ShaderNodeBackground')
    tc = node(nt, 'ShaderNodeTexCoord')
    sp = node(nt, 'ShaderNodeSeparateXYZ')
    lk(nt, tc.outputs['Generated'], sp.inputs[0])
    def band(sock, a, b, c, d):
        n = node(nt, 'ShaderNodeMapRange', interpolation_type='SMOOTHSTEP', clamp=True)
        lk(nt, sock, n.inputs['Value'])
        n.inputs['From Min'].default_value = a
        n.inputs['From Max'].default_value = b
        m = node(nt, 'ShaderNodeMapRange', interpolation_type='SMOOTHSTEP', clamp=True)
        lk(nt, sock, m.inputs['Value'])
        m.inputs['From Min'].default_value = c
        m.inputs['From Max'].default_value = d
        return math_node(nt, 'SUBTRACT', n.outputs['Result'], m.outputs['Result'])
    top = band(sp.outputs['Z'], 0.78, 0.9, 2.0, 3.0) if not zenith else band(sp.outputs['Z'], 0.5, 0.6, 2.0, 3.0)
    horizon = band(sp.outputs['Z'], -0.02, 0.06, 0.16, 0.3)
    strip_a = band(sp.outputs['X'], 0.86, 0.9, 0.97, 0.99)
    strip_b = band(sp.outputs['X'], -0.99, -0.97, -0.9, -0.86)
    rim = band(sp.outputs['Y'], 0.9, 0.95, 2.0, 3.0)
    upper = band(sp.outputs['Z'], 0.05, 0.25, 2.0, 3.0)
    # A dim floor and a low bounce card, so the underside of chrome reads as a gradient, not a hole.
    floor = band(sp.outputs['Z'], -3.0, -2.0, -0.08, -0.01)
    bounce = band(sp.outputs['Z'], -0.4, -0.28, -0.14, -0.05)
    if zenith:
        # A top-down mirror only sees the sky near the zenith: break the softbox into a ceiling of
        # strip lights and a cross bar so the ripples have edges to bend.
        sx = math_node(nt, 'FRACT', math_node(nt, 'MULTIPLY', sp.outputs['X'], vb=1.0))
        sx = math_node(nt, 'ABSOLUTE', math_node(nt, 'SUBTRACT', sx, vb=0.5))
        sy = math_node(nt, 'ABSOLUTE', sp.outputs['Y'])
        stripes = node(nt, 'ShaderNodeMapRange', interpolation_type='SMOOTHSTEP', clamp=True)
        lk(nt, sx, stripes.inputs['Value'])
        stripes.inputs['From Min'].default_value = 0.04
        stripes.inputs['From Max'].default_value = 0.42
        bar = node(nt, 'ShaderNodeMapRange', interpolation_type='SMOOTHSTEP', clamp=True)
        lk(nt, sy, bar.inputs['Value'])
        bar.inputs['From Min'].default_value = 0.3
        bar.inputs['From Max'].default_value = 0.02
        lit = math_node(nt, 'MAXIMUM', math_node(nt, 'SUBTRACT', None, stripes.outputs['Result'], va=1.0), bar.outputs['Result'])
        top = math_node(nt, 'MULTIPLY', top, math_node(nt, 'ADD', math_node(nt, 'MULTIPLY', lit, vb=0.78), None, vb=0.22))
    L = math_node(nt, 'MULTIPLY', top, vb=3.2)
    L = math_node(nt, 'ADD', L, math_node(nt, 'MULTIPLY', horizon, vb=1.6))
    strips = math_node(nt, 'MULTIPLY', math_node(nt, 'ADD', strip_a, strip_b), upper)
    L = math_node(nt, 'ADD', L, math_node(nt, 'MULTIPLY', strips, vb=2.6))
    L = math_node(nt, 'ADD', L, math_node(nt, 'MULTIPLY', rim, vb=1.2))
    L = math_node(nt, 'ADD', L, math_node(nt, 'MULTIPLY', floor, vb=0.1))
    L = math_node(nt, 'ADD', L, math_node(nt, 'MULTIPLY', bounce, vb=0.55))
    L = math_node(nt, 'ADD', L, None, vb=0.015)
    L = math_node(nt, 'MULTIPLY', L, vb=strength)
    lk(nt, L, bg.inputs['Strength'])
    bg.inputs['Color'].default_value = (1.0, 1.0, 1.0, 1.0)
    lk(nt, bg.outputs[0], out.inputs['Surface'])
    try:
        w.cycles_visibility.camera = False
    except Exception:
        pass
    area_light('key', (-3.0, -2.0, 4.5), (0, 0, 0), 2.5, key)
    area_light('rim', (0.5, 4.0, 2.5), (0, 0, 0), 1.5, key * 0.6, color=(0.85, 0.92, 1.0))
"#;

/// A looping flipbook. Params: `style` (`mercury_blob`, `ferrofluid`, `mercury_pool`),
/// `frames`, `cell`, `samples`, `seed`. Writes `beauty_####.png` and `data_####.exr`.
pub const FLIPBOOK: &str = r#"
style = ARGS['style']
frames = int(ARGS['frames'])
cell = int(ARGS['cell'])
sc = reset()
dev = gpu_cycles(sc, samples=ARGS.get('samples', 128), denoise=True, bounces=10)
resolution(sc, cell, cell)
view(sc, 'AgX', look='AgX - Punchy')
studio(sc, zenith=(style == 'mercury_pool'))
if style == 'mercury_pool':
    bpy.ops.mesh.primitive_grid_add(x_subdivisions=600, y_subdivisions=600, size=2.0)
    ob = bpy.context.active_object
    bpy.ops.object.shade_smooth()
    handles = gn_displace(ob, [(2.2, 0.055), (6.0, 0.012)], axis='Z')
    ob.data.materials.append(metal_material('mercury', (0.80, 0.81, 0.83), 0.035))
    ortho_cam(sc, 2.0)
    dmin, dmax = -0.07, 0.07
    radius = 0.9
elif style == 'ferrofluid':
    bpy.ops.mesh.primitive_uv_sphere_add(segments=512, ring_count=256, radius=1.0)
    ob = bpy.context.active_object
    ob.scale = (1.0, 1.0, 0.62)
    bpy.ops.object.transform_apply(scale=True)
    bpy.ops.object.shade_smooth()
    handles = gn_displace(ob, [(1.1, 0.12)], spikes=(7.0, 0.30, 0.55))
    ob.data.materials.append(metal_material('ferro', (0.012, 0.012, 0.014), 0.16, coat=1.0, metallic=0.25))
    persp_cam(sc, (0.0, -4.6, 2.6), (0.0, 0.0, 0.05), lens=62.0)
    dmin, dmax = -0.06, 0.4
    radius = 0.7
else:
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=7, radius=1.0)
    ob = bpy.context.active_object
    bpy.ops.object.shade_smooth()
    handles = gn_displace(ob, [(0.9, 0.5), (2.4, 0.035)])
    ob.data.materials.append(metal_material('chrome', (0.93, 0.93, 0.95), 0.03))
    persp_cam(sc, (0.0, -5.4, 0.9), (0.0, 0.0, 0.0), lens=58.0)
    dmin, dmax = -0.33, 0.33
    radius = 0.75
dm = view_data_material(dmin, dmax)
for f in range(frames):
    theta = 2.0 * math.pi * f / frames
    set_loop(handles, theta, radius)
    sc.frame_set(1)
    gpu_cycles(sc, samples=ARGS.get('samples', 128), denoise=True, bounces=10)
    sc.render.film_transparent = True
    save(sc, os.path.join(OUT, 'beauty_%04d.png' % f), depth='16')
    render_data(sc, os.path.join(OUT, 'data_%04d.exr' % f), dm, samples=8)
done()
"#;

/// Matcaps: spheres in the same studio light, seen head-on with an orthographic camera.
/// Params: `items: [{name, color, roughness, metallic, coat}]`, `size`.
pub const MATCAPS: &str = r#"
for it in ARGS['items']:
    sc = reset()
    gpu_cycles(sc, samples=ARGS.get('samples', 256), denoise=True)
    resolution(sc, ARGS['size'], ARGS['size'])
    view(sc, 'AgX', look='AgX - Punchy')
    studio(sc)
    bpy.ops.mesh.primitive_uv_sphere_add(segments=256, ring_count=128, radius=1.0)
    ob = bpy.context.active_object
    bpy.ops.object.shade_smooth()
    ob.data.materials.append(metal_material(it['name'], it['color'], it['roughness'], it.get('coat', 0.0), it.get('metallic', 1.0)))
    cd = bpy.data.cameras.new('cam')
    cd.type = 'ORTHO'
    cd.ortho_scale = 2.0
    cam = bpy.data.objects.new('cam', cd)
    cam.location = (0.0, -10.0, 0.0)
    cam.rotation_euler = (math.radians(90.0), 0.0, 0.0)
    link(cam)
    sc.camera = cam
    save(sc, os.path.join(OUT, it['name'] + '.png'), depth='16')
done()
"#;
