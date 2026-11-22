//! Liquid Glass recipes: bevelled plate geometry and liquid droplets / meniscus. (Lens caustics
//! are traced in Rust over these plates; see `sets::glass`.)

/// Plates: for each shape, a bevelled slab rendered top-down as a normal + height data pass.
/// Params: `shapes: [{name, kind, w, h, r, bevel, thick, profile, px_w, px_h, n}]` in metres
/// where 1 unit = 1000 px.
pub const PLATES: &str = r#"
for s in ARGS['shapes']:
    sc = reset()
    pts = outline(s['kind'], s['w'], s['h'], s.get('r', 0.0), s.get('n', 5.0), segs=128)
    ob = plate_mesh('plate', pts, s['thick'], s['bevel'], segs=48, profile=s.get('profile', 0.5))
    resolution(sc, s['px_w'], s['px_h'])
    ortho_cam(sc, s['w'] * s.get('margin', 1.0))
    render_data(sc, os.path.join(OUT, s['name'] + '.exr'), data_material(0.0, s['thick']), samples=16)
done()
"#;

/// Liquid: a single droplet, a seamless droplet scatter tile, a meniscus ring channel and a
/// liquid capsule end. Droplets are metaballs clipped at the glass plane (camera clip at z=0),
/// so the contact line keeps a physical contact angle.
pub const LIQUID: &str = r#"
def mball(name, res=0.01):
    mb = bpy.data.metaballs.new(name)
    mb.resolution = res
    mb.render_resolution = res
    mb.threshold = 0.6
    ob = bpy.data.objects.new(name, mb)
    link(ob)
    return mb, ob

def render_top(sc, name, width, px_w, px_h, hmax, center=(0.0, 0.0)):
    resolution(sc, px_w, px_h)
    ortho_cam(sc, width, center=center, z=20.0, clip_end=20.0)
    render_data(sc, os.path.join(OUT, name + '.exr'), data_material(0.0, hmax), samples=16)

# 1. single droplet (sprite): a blob of three merged balls, slightly asymmetric.
sc = reset()
mb, ob = mball('drop', 0.006)
for (x, y, z, r) in ((0.0, 0.0, -0.18, 0.62), (0.07, 0.05, -0.2, 0.42), (-0.06, -0.04, -0.21, 0.38)):
    e = mb.elements.new()
    e.co = (x, y, z)
    e.radius = r
render_top(sc, 'droplet_single', 1.4, 1024, 1024, 0.5)

# 2. droplet scatter tile: seeded balls wrapped across the edges so the tile is seamless.
sc = reset()
W = 2.0
mb, ob = mball('drops', 0.006)
rnd = random.Random(SEED + 7)
balls = []
for i in range(ARGS.get('drops', 150)):
    r = 0.012 + 0.11 * (rnd.random() ** 3.2)
    balls.append((rnd.random() * W, rnd.random() * W, r))
for (x, y, r) in balls:
    for ox in (-W, 0.0, W):
        for oy in (-W, 0.0, W):
            px, py = x + ox, y + oy
            if -0.3 < px < W + 0.3 and -0.3 < py < W + 0.3:
                e = mb.elements.new()
                e.co = (px, py, -0.35 * r)
                e.radius = r * 1.55
render_top(sc, 'droplets_tile', W, 2048, 2048, 0.12, center=(W * 0.5, W * 0.5))

# 3. meniscus ring: liquid in a circular channel climbing both walls (lathed height profile).
sc = reset()
rin, rout, h0, amp, lam = 0.62, 0.92, 0.02, 0.05, 0.035
rings, segs = 160, 512
bm = bmesh.new()
grid = []
for i in range(rings + 1):
    r = rin + (rout - rin) * i / rings
    z = h0 + amp * (math.exp(-(r - rin) / lam) + math.exp(-(rout - r) / lam))
    row = []
    for j in range(segs):
        a = 2.0 * math.pi * j / segs
        row.append(bm.verts.new((r * math.cos(a), r * math.sin(a), z)))
    grid.append(row)
for i in range(rings):
    for j in range(segs):
        a, b = grid[i][j], grid[i][(j + 1) % segs]
        c, d = grid[i + 1][(j + 1) % segs], grid[i + 1][j]
        f = bm.faces.new((a, b, c, d))
        f.smooth = True
bm.normal_update()
for f in bm.faces:
    if f.normal.z < 0:
        f.normal_flip()
ob = mesh_obj('meniscus', bm)
render_top(sc, 'meniscus_ring', 2.0, 2048, 2048, h0 + 2 * amp)

# 4. liquid capsule (ribbon end / pill blob) from capsule metaballs.
sc = reset()
mb, ob = mball('capsule', 0.008)
e = mb.elements.new()
e.type = 'CAPSULE'
e.co = (0.0, 0.0, -0.08)
e.radius = 0.2
e.size_x = 1.55
render_top(sc, 'liquid_capsule', 4.0, 2048, 512, 0.16)
done()
"#;
