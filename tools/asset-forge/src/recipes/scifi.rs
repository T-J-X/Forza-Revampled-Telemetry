//! Sci-fi recipes: a perspective holographic floor grid and a looping warp tunnel. Both are
//! emission-only on black (additive assets). Use with `classic::EXPR_HELPERS` prepended.

/// Holographic floor grid seen low over the horizon. Params: `w`, `h`, `samples`.
pub const HOLO_GRID: &str = r#"
sc = reset()
gpu_cycles(sc, samples=ARGS.get('samples', 160), denoise=True, bounces=2)
sc.render.film_transparent = False
view(sc, 'Standard')
resolution(sc, ARGS['w'], ARGS['h'])
bpy.ops.mesh.primitive_plane_add(size=400.0)
plane = bpy.context.active_object
m, NT = new_mat('holo')
tc = node(NT, 'ShaderNodeTexCoord')
sp = node(NT, 'ShaderNodeSeparateXYZ')
lk(NT, tc.outputs['Object'], sp.inputs[0])
x = E(sp.outputs['X'])
y = E(sp.outputs['Y'])
def lines(px, py, width):
    dx = abs_(fract_(px) - 0.5)
    dy = abs_(fract_(py) - 0.5)
    return max_(sstep(0.5 - width, 0.5, dx), sstep(0.5 - width, 0.5, dy))
minor = lines(x, y, 0.035)
major = lines(x / 5.0, y / 5.0, 0.012)
lp = node(NT, 'ShaderNodeLightPath')
dist = E(lp.outputs['Ray Length'])
fade = 1.0 - sstep(4.0, 60.0, dist)
glow = (0.35 * minor + 1.0 * major) * fade
em = node(NT, 'ShaderNodeEmission')
em.inputs['Color'].default_value = (0.15, 0.85, 1.0, 1.0)
lk(NT, (glow * 6.0).s, em.inputs['Strength'])
out = node(NT, 'ShaderNodeOutputMaterial')
lk(NT, em.outputs[0], out.inputs['Surface'])
plane.data.materials.append(m)
w = bpy.data.worlds.new('w')
sc.world = w
try:
    w.use_nodes = True
    w.node_tree.nodes['Background'].inputs['Color'].default_value = (0, 0, 0, 1)
except Exception:
    pass
persp_cam(sc, (0.0, -6.0, 1.5), (0.0, 30.0, -0.6), lens=24.0)
save(sc, os.path.join(OUT, 'holo_grid.png'), depth='16', mode='RGB')
done()
"#;

/// Warp tunnel flipbook: the camera looks down an emissive tube whose streak texture is
/// periodic along the tube (torus-mapped noise) and scrolls by exactly one period over the loop.
/// Params: `frames`, `cell`, `samples`.
pub const WARP_TUNNEL: &str = r#"
frames = int(ARGS['frames'])
sc = reset()
gpu_cycles(sc, samples=ARGS.get('samples', 96), denoise=True, bounces=2)
sc.render.film_transparent = False
view(sc, 'AgX', look='AgX - Punchy')
resolution(sc, ARGS['cell'], ARGS['cell'])
bpy.ops.mesh.primitive_cylinder_add(vertices=256, radius=1.0, depth=80.0, end_fill_type='NOTHING', rotation=(math.radians(90.0), 0.0, 0.0), location=(0.0, 40.0, 0.0))
tube = bpy.context.active_object
bpy.ops.object.shade_smooth()
m, NT = new_mat('warp')
tc = node(NT, 'ShaderNodeTexCoord')
sp = node(NT, 'ShaderNodeSeparateXYZ')
lk(NT, tc.outputs['Object'], sp.inputs[0])
ox = E(sp.outputs['X'])
oz = E(sp.outputs['Y'])
oy = E(sp.outputs['Z'])
at = node(NT, 'ShaderNodeMath', operation='ARCTAN2')
lk(NT, oz.s, at.inputs[0])
lk(NT, ox.s, at.inputs[1])
ang = E(at.outputs[0])
PERIOD = 20.0
shift = node(NT, 'ShaderNodeValue')
shift.outputs[0].default_value = 0.0
u = ang / TAU + 0.5
v = (oy + E(shift.outputs[0])) / PERIOD
streak = tnoise(u, v, 140, 3, 3.0, 0.55)
hue = tnoise(u, v, 6, 2, 2.0, 0.5, 9.0)
s = sstep(0.5, 0.78, streak)
lp = node(NT, 'ShaderNodeLightPath')
dist = E(lp.outputs['Ray Length'])
fade = (1.0 - sstep(2.0, 70.0, dist)) * sstep(0.5, 6.0, dist)
mix = node(NT, 'ShaderNodeMix', data_type='RGBA')
mix.inputs[6].default_value = (0.1, 0.8, 1.0, 1.0)
mix.inputs[7].default_value = (0.75, 0.25, 1.0, 1.0)
lk(NT, hue.s, mix.inputs[0])
em = node(NT, 'ShaderNodeEmission')
lk(NT, mix.outputs[2], em.inputs['Color'])
lk(NT, (s * fade * 16.0).s, em.inputs['Strength'])
out = node(NT, 'ShaderNodeOutputMaterial')
lk(NT, em.outputs[0], out.inputs['Surface'])
tube.data.materials.append(m)
w = bpy.data.worlds.new('w')
sc.world = w
persp_cam(sc, (0.0, 0.0, 0.0), (0.0, 10.0, 0.0), lens=16.0)
for f in range(frames):
    shift.outputs[0].default_value = -PERIOD * f / frames
    save(sc, os.path.join(OUT, 'warp_%04d.png' % f), depth='16', mode='RGB')
done()
"#;
