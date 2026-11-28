//! TeslaAdas bespoke assets (`assets/clusters/tesla_adas/`), set `tesla_adas`:
//!
//! - **The car** (Blender, Cycles). No model is imported: [`CAR_LIB`] lofts a generic electric
//!   fastback from a table of stations (one row per cross-section, rear to nose), subdivides it,
//!   cuts the wheel arches and adds wheels, a light bar, running lights and mirrors, with its
//!   panel gaps (boot lid, bonnet, doors, bumper line) cut into the paint as shut lines. The same
//!   table, scaled, gives the two fastback traffic bodies; the third is a kei box from a table of
//!   its own. Every car wears a Japanese number plate (white, or yellow on the kei).
//! - **The ego atlas**: the car through the dash's own chase camera ([`CAM_H`], [`CAM_D`],
//!   [`PITCH_DEG`]: the same eye and pitch as `tesla_adas::scene`) at 11 yaws, in three passes:
//!   the studio-lit body, the light pool of a panel under its floor (the glow, tinted live) and
//!   the light of its brake lamps (added live while braking).
//! - **The parked car**: the same car in a three-quarter front view, as Tesla's parked layout
//!   draws it.
//! - **Traffic**: each body over 4 view elevations x 5 headings.
//! - **The Japanese label face** ([`FONT_FILE`]): Noto Sans JP Medium (OFL), vendored as a subset
//!   (kana, 233 kanji, Basic Latin). The job checks the file and enters it in the ledger; how it was
//!   made is in the dash's `ASSETS.md`.
//! - **Tunnel lights plate** (ComfyUI: SDXL base 1.0 + 4x-UltraSharp): the ceiling lamps of a
//!   Japanese expressway tunnel. Every lit car shot hangs it over the road as a light card, so the
//!   glass roof and the paint carry the lamp rows. It is built first; the car jobs' fingerprints
//!   include its bytes.
//!
//! Each keeps a procedural fallback in the cluster (`setup.assets` may be `None`).
use crate::comfy::{self, Comfy};
use crate::ctx::{Ctx, Format, Spec};
use crate::error::{io_err, tool_err, Result};
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use crate::{blender, recipes};
use serde_json::{json, Map, Value};
use std::time::Duration;

const CLUSTERS: &str = "TeslaAdas";
const SET: &str = "tesla_adas";

/// bpy: the car builder. `build_car(kind, mats)` returns the root of one car standing on z = 0
/// with its nose towards Blender -Y (the kit's +z); `car_mats` makes its materials.
pub const CAR_LIB: &str = r#"
# ---- car_lib: a lofted generic EV fastback, built from a station table (no imported model) ----
# Blender frame: +X right, nose towards -Y, +Z up, ground at z = 0, the car centred on x = y = 0.
# The kit's frame is (x, y up, z forward) = Blender (x, z, -y).

# One row per station, rear to nose: u (metres from the tail), top, belt, half-width at the
# shoulder, half-width of the top face, bottom of the side.
SEDAN = [
    (0.00, 0.930, 0.890, 0.800, 0.660, 0.360),
    (0.12, 0.955, 0.915, 0.900, 0.760, 0.300),
    (0.30, 0.975, 0.935, 0.948, 0.800, 0.255),
    (0.55, 1.000, 0.948, 0.970, 0.790, 0.215),
    (0.85, 1.050, 0.960, 0.980, 0.735, 0.200),
    (1.20, 1.155, 0.970, 0.982, 0.670, 0.195),
    (1.60, 1.290, 0.976, 0.976, 0.615, 0.195),
    (2.00, 1.375, 0.976, 0.969, 0.592, 0.195),
    (2.10, 1.388, 0.976, 0.968, 0.588, 0.195),
    (2.50, 1.405, 0.972, 0.965, 0.585, 0.195),
    (2.90, 1.372, 0.965, 0.965, 0.593, 0.195),
    (3.00, 1.352, 0.963, 0.965, 0.600, 0.195),
    (3.25, 1.240, 0.957, 0.968, 0.640, 0.195),
    (3.55, 1.092, 0.950, 0.972, 0.715, 0.195),
    (3.82, 0.990, 0.942, 0.978, 0.800, 0.200),
    (4.10, 0.945, 0.905, 0.976, 0.825, 0.205),
    (4.38, 0.900, 0.860, 0.958, 0.815, 0.220),
    (4.60, 0.845, 0.805, 0.915, 0.775, 0.245),
    (4.76, 0.785, 0.745, 0.845, 0.700, 0.275),
    (4.85, 0.725, 0.690, 0.740, 0.600, 0.310),
]

# The kei box: Japan's light-car class is capped at 3.40 m long, 1.48 m wide and 2.00 m high, so
# its shape is a tall one-box with a stub bonnet, an upright screen and a vertical tail.
KEI = [
    (0.00, 1.700, 1.020, 0.700, 0.600, 0.330),
    (0.06, 1.745, 1.030, 0.725, 0.640, 0.280),
    (0.20, 1.770, 1.035, 0.738, 0.655, 0.240),
    (0.60, 1.780, 1.035, 0.740, 0.660, 0.210),
    (1.20, 1.782, 1.030, 0.740, 0.660, 0.200),
    (1.90, 1.778, 1.020, 0.740, 0.658, 0.200),
    (2.35, 1.760, 1.010, 0.740, 0.650, 0.200),
    (2.45, 1.735, 1.005, 0.740, 0.648, 0.200),
    (2.70, 1.400, 0.990, 0.738, 0.660, 0.200),
    (2.92, 1.060, 0.975, 0.735, 0.690, 0.205),
    (3.10, 0.985, 0.940, 0.730, 0.690, 0.210),
    (3.25, 0.930, 0.890, 0.715, 0.670, 0.225),
    (3.35, 0.840, 0.800, 0.680, 0.620, 0.260),
    (3.40, 0.740, 0.700, 0.620, 0.540, 0.310),
]

# Where the glass starts and ends along the car (u, metres from the tail, before scaling).
GLASS_REAR = (0.85, 2.00)     # the rear window, on the top faces
GLASS_ROOF = (2.10, 2.90)     # the roof panel; the gaps either side of it are painted cross-bars
GLASS_FRONT = (3.00, 3.82)    # the windscreen
GLASS_SIDE = (1.20, 3.55)     # the side windows; the faces either side of it are the pillars
AXLES = (0.98, 3.90)          # rear and front axle
RINGS = 12

# Per-kind changes: overall scales, a roof stretch and a ducktail kick on the sedan table, or a
# table of its own with its own glass, axles, mirror station and tail.
# Shut lines (u, metres from the tail): the panel gaps across the top (boot lid, bonnet) and
# down the sides (the doors' edges), and the bumper line across the tail.
SEAMS_SEDAN = {'top': (0.80, 3.90), 'side': (1.28, 2.45, 3.52), 'bumper': 0.205}
SEAMS_KEI = {'top': (2.98,), 'side': (0.46, 1.52, 2.58), 'bumper': 0.215}
FASTBACK = {'table': SEDAN, 'glass': (GLASS_REAR, GLASS_ROOF, GLASS_FRONT), 'side': GLASS_SIDE, 'axles': AXLES,
            'mirror_u': 3.50, 'duck': 0.0, 'box': False, 'plate': (0.92, 0.92, 0.90), 'seams': SEAMS_SEDAN}
KINDS = {
    'ego':   dict(FASTBACK, len=1.00, wid=1.00, hgt=1.00, cabin=0.00, tail=0.00, wheel_r=0.355, duck=0.060),
    'sedan': dict(FASTBACK, len=0.98, wid=0.97, hgt=1.02, cabin=0.02, tail=0.00, wheel_r=0.345),
    'sport': dict(FASTBACK, len=0.94, wid=1.03, hgt=0.89, cabin=-0.03, tail=-0.02, wheel_r=0.350, duck=0.095),
    'kei':   {'table': KEI, 'glass': ((2.43, 2.92),), 'side': (0.34, 2.62), 'axles': (0.43, 2.95), 'mirror_u': 2.62,
              'duck': 0.0, 'box': True, 'plate': (0.95, 0.74, 0.03), 'seams': SEAMS_KEI,
              'len': 1.00, 'wid': 1.00, 'hgt': 1.00, 'cabin': 0.00, 'tail': 0.00, 'wheel_r': 0.280},
}



def car_table(kind):
    k = KINDS[kind]
    rows = []
    for (u, top, belt, hw, hwr, bot) in k['table']:
        # Lift the roof by the cabin term in the middle of the car and the tail term at the back;
        # the ducktail kicks the trailing edge of the deck up.
        mid = max(0.0, 1.0 - abs(u - 2.45) / 1.6)
        back = max(0.0, 1.0 - u / 1.9)
        t = top + k['cabin'] * mid + k['tail'] * back + k['duck'] * max(0.0, 1.0 - u / 0.32)
        b = belt + 0.35 * k['tail'] * back
        rows.append((u * k['len'], bot + (t - bot) * k['hgt'], bot + (b - bot) * k['hgt'], hw * k['wid'], hwr * k['wid'], bot))
    return rows


def ring(row):
    (u, top, belt, hw, hwr, bot) = row
    crown = max(top - belt, 0.02)
    return [
        (0.0, bot + 0.015),
        (hw * 0.80, bot),
        (hw * 0.955, bot + 0.06),
        (hw * 0.985, bot + 0.15),
        (hw * 1.000, bot + 0.50 * (belt - bot)),
        (hw * 0.992, belt - 0.075),
        (hw * 0.945, belt),
        (hw * 0.945 + (hwr - hw * 0.945) * 0.55, belt + crown * 0.56),
        (hwr, top - min(0.05, crown * 0.45)),
        (hwr * 0.86, top - min(0.028, crown * 0.26)),
        (hwr * 0.45, top - min(0.008, crown * 0.08)),
        (0.0, top),
    ]


def between(u, span, scale):
    return span[0] * scale <= u <= span[1] * scale


def build_body(kind, mats):
    k = KINDS[kind]
    rows = car_table(kind)
    L = rows[-1][0]
    bm = bmesh.new()
    crease = bm.edges.layers.float.new('crease_edge')
    grid = []
    for row in rows:
        y = L * 0.5 - row[0]
        grid.append([bm.verts.new((x, y, z)) for (x, z) in ring(row)])
    # End stations: two smaller copies pulled in, so the bumpers round off, then a small cap.
    for end, idx in ((0, 0), (1, len(rows) - 1)):
        row = rows[idx]
        cz = 0.5 * (row[1] + row[5])
        for (dy, kx, kz) in ((0.040, 0.90, 0.86), (0.055, 0.55, 0.52)):
            y = L * 0.5 - row[0] + (dy if end == 0 else -dy)
            pts = [(x * kx, cz + (z - cz) * kz) for (x, z) in ring(row)]
            vs = [bm.verts.new((x, y, z)) for (x, z) in pts]
            if end == 0:
                grid.insert(0, vs)
            else:
                grid.append(vs)
    us = [rows[0][0]] * 2 + [r[0] for r in rows] + [rows[-1][0]] * 2
    slot = {'paint': 0, 'glass': 1, 'trim': 2}
    n = len(grid)
    for i in range(n - 1):
        um = 0.5 * (us[i] + us[i + 1])
        for j in range(RINGS - 1):
            a, b, c, d = grid[i][j], grid[i][j + 1], grid[i + 1][j + 1], grid[i + 1][j]
            f = bm.faces.new((a, b, c, d))
            mat = 'paint'
            if j <= 2 and 2 <= i < n - 3:
                mat = 'trim'
            elif j in (6, 7) and between(um, k['side'], k['len']):
                mat = 'glass'
            elif j >= 9 and any(between(um, span, k['len']) for span in k['glass']):
                mat = 'glass'
            f.material_index = slot[mat]
            f.smooth = True
    # Caps (half an n-gon each; the mirror closes them).
    f = bm.faces.new(list(reversed(grid[0])))
    f.material_index = slot['paint']
    f.smooth = True
    f = bm.faces.new(grid[-1])
    f.material_index = slot['paint']
    f.smooth = True
    bm.normal_update()
    # Feature lines: the rocker and the belt hold a soft crease; the glass edge a firmer one.
    for i in range(n - 1):
        for j, w in ((3, 0.60), (6, 0.50)):
            e = bm.edges.get((grid[i][j], grid[i + 1][j]))
            if e is not None:
                e[crease] = w
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    ob = mesh_obj('body_' + kind, bm)
    for key in ('paint', 'glass', 'trim'):
        ob.data.materials.append(mats[key])
    md = ob.modifiers.new('mirror', 'MIRROR')
    md.use_axis = (True, False, False)
    md.use_clip = True
    md.merge_threshold = 0.002
    md = ob.modifiers.new('subsurf', 'SUBSURF')
    md.levels = 3
    md.render_levels = 3
    return ob, rows, L


def wheel(name, r, width, mats):
    """A tyre with rounded shoulders and an aero cover, its axis along X, centred at the origin."""
    parts = []
    bm = bmesh.new()
    segs = 48
    prof = [
        (r * 0.66, -width * 0.50), (r * 0.90, -width * 0.50), (r * 0.985, -width * 0.40), (r, -width * 0.22),
        (r, width * 0.22), (r * 0.985, width * 0.40), (r * 0.90, width * 0.50), (r * 0.66, width * 0.50),
    ]
    loops = []
    for (pr, px) in prof:
        loops.append([bm.verts.new((px, pr * math.cos(2 * math.pi * s / segs), pr * math.sin(2 * math.pi * s / segs))) for s in range(segs)])
    for a in range(len(loops) - 1):
        for s in range(segs):
            f = bm.faces.new((loops[a][s], loops[a][(s + 1) % segs], loops[a + 1][(s + 1) % segs], loops[a + 1][s]))
            f.smooth = True
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    tyre = mesh_obj(name + '_tyre', bm)
    tyre.data.materials.append(mats['tyre'])
    parts.append(tyre)
    # The cover: a shallow cone of rings on the outer face (+X), with five slots.
    bm = bmesh.new()
    rr = [(0.0, 0.030), (r * 0.16, 0.032), (r * 0.20, 0.018), (r * 0.62, 0.004), (r * 0.67, -0.012), (r * 0.675, -0.06)]
    loops = []
    for (pr, off) in rr:
        if pr == 0.0:
            loops.append([bm.verts.new((width * 0.5 - 0.035 + off, 0.0, 0.0))])
        else:
            loops.append([bm.verts.new((width * 0.5 - 0.035 + off, pr * math.cos(2 * math.pi * s / segs), pr * math.sin(2 * math.pi * s / segs))) for s in range(segs)])
    for s in range(segs):
        f = bm.faces.new((loops[0][0], loops[1][s], loops[1][(s + 1) % segs]))
        f.smooth = True
    for a in range(1, len(loops) - 1):
        for s in range(segs):
            f = bm.faces.new((loops[a][s], loops[a + 1][s], loops[a + 1][(s + 1) % segs], loops[a][(s + 1) % segs]))
            f.smooth = True
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    cover = mesh_obj(name + '_cover', bm)
    cover.data.materials.append(mats['rim'])
    parts.append(cover)
    for sidx in range(5):
        a = 2 * math.pi * sidx / 5.0
        bpy.ops.mesh.primitive_cube_add(size=1.0)
        slot_ob = bpy.context.active_object
        slot_ob.scale = (0.012, r * 0.085, r * 0.19)
        rad = r * 0.43
        slot_ob.location = (width * 0.5 - 0.022, rad * math.sin(a), rad * math.cos(a))
        slot_ob.rotation_euler = (-a, 0.0, 0.0)
        slot_ob.data.materials.append(mats['trim'])
        parts.append(slot_ob)
    root = bpy.data.objects.new(name, None)
    link(root)
    for o in parts:
        o.parent = root
    return root


def strip_on(body, name, y_from, x_half, z, height, mat, toward=1.0, segs=40, offset=0.004, rows=1):
    """A thin strip projected onto the body along Y from outside (toward = +1: onto the tail from
    behind; -1: onto the nose from ahead), then given 8 mm of thickness. `rows` is the number of
    quads up the strip: a tall strip needs several to follow a curved panel."""
    bm = bmesh.new()
    rows_v = []
    for r in range(rows + 1):
        zi = z - height * 0.5 + height * r / rows
        rows_v.append([bm.verts.new((-x_half + 2.0 * x_half * s / segs, y_from, zi)) for s in range(segs + 1)])
    for r in range(rows):
        for s in range(segs):
            f = bm.faces.new((rows_v[r][s], rows_v[r][s + 1], rows_v[r + 1][s + 1], rows_v[r + 1][s]))
            f.smooth = True
    ob = mesh_obj(name, bm)
    ob.data.materials.append(mat)
    md = ob.modifiers.new('wrap', 'SHRINKWRAP')
    md.target = body
    md.wrap_method = 'PROJECT'
    md.use_project_x = False
    md.use_project_y = True
    md.use_project_z = False
    md.use_negative_direction = toward > 0.0
    md.use_positive_direction = toward < 0.0
    md.offset = offset
    md = ob.modifiers.new('thick', 'SOLIDIFY')
    md.thickness = 0.008
    md.offset = 1.0
    return ob


SEAM_HALF = 0.0050     # half the gap's width (m): about a pixel at the dash's scale
SEAM_SOFT = 0.0030     # its soft shoulder (m)


def _m(nt, op, a, b=None):
    """A math node on `a` (a socket or a number) and `b`."""
    n = nt.nodes.new('ShaderNodeMath')
    n.operation = op
    for i, v in enumerate((a, b)):
        if v is None:
            continue
        if isinstance(v, (int, float)):
            n.inputs[i].default_value = v
        else:
            nt.links.new(v, n.inputs[i])
    return n.outputs[0]


def _band(nt, coord, centre, half, soft):
    """1 on the line `coord == centre`, falling to 0 over `soft` past `half`."""
    d = _m(nt, 'ABSOLUTE', _m(nt, 'SUBTRACT', coord, centre))
    return _m(nt, 'SUBTRACT', 1.0, _m(nt, 'MINIMUM', 1.0, _m(nt, 'DIVIDE', _m(nt, 'MAXIMUM', _m(nt, 'SUBTRACT', d, half), 0.0), soft)))


def _between(nt, coord, lo, hi):
    return _m(nt, 'MULTIPLY', _m(nt, 'GREATER_THAN', coord, lo), _m(nt, 'LESS_THAN', coord, hi))


def shut_lines(body, kind, rows, L, paint):
    """Gives the body a copy of the paint with its panel gaps cut in: the gap is dark and has no
    clear coat, so it reads as a shut line, not a painted stripe."""
    k = KINDS[kind]
    seams = k['seams']
    m = paint.copy()
    m.name = 'paint_' + kind
    nt = m.node_tree
    b = next(n for n in nt.nodes if n.type == 'BSDF_PRINCIPLED')
    tc = nt.nodes.new('ShaderNodeTexCoord')
    xyz = nt.nodes.new('ShaderNodeSeparateXYZ')
    nt.links.new(tc.outputs['Object'], xyz.inputs[0])
    y, z = xyz.outputs['Y'], xyz.outputs['Z']
    belt = max(r[2] for r in rows)
    bot = min(r[5] for r in rows)
    y_of = lambda u: L * 0.5 - u * k['len']
    line = None
    def add(v):
        nonlocal line
        line = v if line is None else _m(nt, 'MAXIMUM', line, v)
    for u in seams['top']:
        add(_m(nt, 'MULTIPLY', _band(nt, y, y_of(u), SEAM_HALF, SEAM_SOFT), _m(nt, 'GREATER_THAN', z, belt - 0.03)))
    for u in seams['side']:
        add(_m(nt, 'MULTIPLY', _band(nt, y, y_of(u), SEAM_HALF, SEAM_SOFT), _between(nt, z, bot + 0.07, belt - 0.015)))
    # The bumper line across the tail, on the tail's last 0.30 m only.
    tail = rows[0]
    zb = tail[5] + seams['bumper']
    add(_m(nt, 'MULTIPLY', _band(nt, z, zb, SEAM_HALF, SEAM_SOFT), _m(nt, 'GREATER_THAN', y, L * 0.5 - 0.30)))
    base = b.inputs['Base Color'].default_value
    mix = nt.nodes.new('ShaderNodeMix')
    mix.data_type = 'RGBA'
    mix.inputs['A'].default_value = base
    mix.inputs['B'].default_value = (0.006, 0.006, 0.007, 1.0)
    nt.links.new(_m(nt, 'MULTIPLY', line, 0.92), mix.inputs['Factor'])
    nt.links.new(mix.outputs['Result'], b.inputs['Base Color'])
    nt.links.new(_m(nt, 'SUBTRACT', 1.0, line), b.inputs['Coat Weight'])
    nt.links.new(_m(nt, 'ADD', b.inputs['Roughness'].default_value, _m(nt, 'MULTIPLY', line, 0.5)), b.inputs['Roughness'])
    body.data.materials[0] = m


def build_car(kind, mats):
    """Builds one car and returns its root empty. `mats` holds paint, glass, trim, tyre, rim,
    tail, head and well."""
    k = KINDS[kind]
    body, rows, L = build_body(kind, mats)
    shut_lines(body, kind, rows, L, mats['paint'])
    root = bpy.data.objects.new('car_' + kind, None)
    link(root)
    body.parent = root
    r = k['wheel_r']
    width = 0.265
    hw_max = max(row[3] for row in rows)
    # Wheel arches: cut after the subdivision, lined in the dark well material.
    for ai, au in enumerate(k['axles']):
        y = L * 0.5 - au * k['len']
        for sx in (-1.0, 1.0):
            bpy.ops.mesh.primitive_cylinder_add(vertices=64, radius=r + 0.055, depth=0.62, location=(sx * (hw_max - 0.10), y, r), rotation=(0.0, math.radians(90.0), 0.0))
            cut = bpy.context.active_object
            cut.data.materials.append(mats['well'])
            cut.display_type = 'WIRE'
            cut.hide_render = True
            cut.parent = root
            md = body.modifiers.new('arch', 'BOOLEAN')
            md.operation = 'DIFFERENCE'
            md.object = cut
            try:
                md.solver = 'EXACT'
                md.material_mode = 'TRANSFER'
            except Exception as e:
                log('boolean', e)
            w = wheel('wheel_%d_%d' % (ai, int(sx)), r, width, mats)
            w.location = (sx * (hw_max - 0.035 - width * 0.5), y, r)
            if sx < 0.0:
                w.rotation_euler = (0.0, 0.0, math.pi)
            w.parent = root
    # Lamps: the fastbacks carry one light bar across the tail; the kei box carries two upright
    # lamps at its corners and its rear window on the tail. Two thin running lights on the nose.
    tail_u = rows[1]
    back = L * 0.5 + 0.6
    if k['box']:
        for sx in (-1.0, 1.0):
            lamp = strip_on(body, 'tail_%d' % int(sx), back, 0.045, 1.130, 0.400, mats['tail'], toward=1.0, segs=6, rows=6)
            lamp.location.x = sx * hw_max * 0.855
            lamp.parent = root
        glass = strip_on(body, 'tail_glass', back, hw_max * 0.70, 1.420, 0.430, mats['glass'], toward=1.0, segs=24, rows=8)
        glass.parent = root
        plate_z = 0.640
    else:
        tail = strip_on(body, 'tail_bar', back, hw_max * 0.86, tail_u[2] - 0.075, 0.052, mats['tail'], toward=1.0)
        tail.parent = root
        plate_z = tail_u[2] - 0.255
    for sx in (-1.0, 1.0):
        head = strip_on(body, 'head_%d' % int(sx), -L * 0.5 - 0.6, 0.20 if not k['box'] else 0.13, rows[-1][2] - 0.035, 0.034 if not k['box'] else 0.06, mats['head'], toward=-1.0, segs=12)
        head.location.x = sx * hw_max * (0.52 if not k['box'] else 0.62)
        head.parent = root
    # The Japanese number plate: 330 x 165 mm, white for ordinary cars and yellow for the kei
    # class, with the two dark green rows of a real plate as plain blocks (no readable number).
    plate = strip_on(body, 'plate', back, 0.165, plate_z, 0.165, mats['plate'], toward=1.0, segs=6, rows=3)
    plate.parent = root
    for (px, pz, pw, ph) in ((0.0, 0.048, 0.150, 0.030), (-0.066, -0.026, 0.110, 0.072), (0.066, -0.026, 0.110, 0.072)):
        mark = strip_on(body, 'plate_mark', back, pw * 0.5, plate_z + pz, ph, mats['plate_ink'], toward=1.0, segs=4, offset=0.013)
        mark.location.x = px
        mark.parent = root
    # The dark diffuser under the tail and the intake under the nose.
    rear = strip_on(body, 'diffuser', back, hw_max * 0.62, rows[0][5] + 0.150, 0.10, mats['trim'], toward=1.0)
    rear.parent = root
    front = strip_on(body, 'intake', -L * 0.5 - 0.6, hw_max * 0.50, rows[-1][5] + 0.140, 0.09, mats['trim'], toward=-1.0)
    front.parent = root
    # Door mirrors.
    cowl = min(rows, key=lambda row: abs(row[0] - k['mirror_u'] * k['len']))
    for sx in (-1.0, 1.0):
        bpy.ops.mesh.primitive_uv_sphere_add(segments=24, ring_count=12, radius=1.0)
        mir = bpy.context.active_object
        mir.scale = (0.075, 0.060, 0.042)
        mir.location = (sx * (cowl[3] + 0.020), L * 0.5 - cowl[0], cowl[2] + 0.045)
        for p in mir.data.polygons:
            p.use_smooth = True
        mir.data.materials.append(mats['paint'])
        mir.parent = root
    return root


def bsdf(name, base, metallic=0.0, rough=0.5, coat=0.0, coat_rough=0.03, emit=None, emit_strength=0.0, ior=None, transmission=0.0):
    m, nt = new_mat(name)
    b = nt.nodes.new('ShaderNodeBsdfPrincipled')
    b.inputs['Base Color'].default_value = (base[0], base[1], base[2], 1.0)
    b.inputs['Metallic'].default_value = metallic
    b.inputs['Roughness'].default_value = rough
    try:
        b.inputs['Coat Weight'].default_value = coat
        b.inputs['Coat Roughness'].default_value = coat_rough
        if ior is not None:
            b.inputs['IOR'].default_value = ior
        b.inputs['Transmission Weight'].default_value = transmission
    except Exception as e:
        log('bsdf', e)
    if emit is not None:
        b.inputs['Emission Color'].default_value = (emit[0], emit[1], emit[2], 1.0)
        b.inputs['Emission Strength'].default_value = emit_strength
    o = nt.nodes.new('ShaderNodeOutputMaterial')
    nt.links.new(b.outputs['BSDF'], o.inputs['Surface'])
    return m


def car_mats(paint, metallic, rough, tail_emit, head_emit, plate=(0.92, 0.92, 0.90)):
    """The car's materials. `tail_emit` and `head_emit` are the lamps' emission strengths and
    `plate` is the number plate's colour."""
    return {
        'plate': bsdf('plate', plate, rough=0.45),
        'plate_ink': bsdf('plate_ink', (0.010, 0.085, 0.045), rough=0.5),
        'paint': bsdf('paint', paint, metallic=metallic, rough=rough, coat=1.0, coat_rough=0.025),
        'glass': bsdf('glass', (0.010, 0.012, 0.016), rough=0.035, coat=1.0, coat_rough=0.0),
        'trim': bsdf('trim', (0.012, 0.012, 0.013), rough=0.42),
        'well': bsdf('well', (0.004, 0.004, 0.004), rough=0.9),
        'tyre': bsdf('tyre', (0.014, 0.014, 0.015), rough=0.82),
        'rim': bsdf('rim', (0.10, 0.105, 0.11), metallic=0.85, rough=0.32, coat=0.4),
        'tail': bsdf('tail', (0.16, 0.004, 0.006), rough=0.10, coat=1.0, emit=(1.0, 0.03, 0.02), emit_strength=tail_emit),
        'head': bsdf('head', (0.8, 0.82, 0.86), rough=0.08, coat=1.0, emit=(0.86, 0.92, 1.0), emit_strength=head_emit),
    }
"#;

/// bpy: one render per shot. `pass` is `beauty` (studio-lit, AgX, straight alpha), `parked` (the
/// same with the parked lighting and the running lights on), `glow` (only a light panel under the
/// floor, on a pale ground; scene-linear EXR) or `brake` (only the tail bar's emission; EXR).
/// The chase camera reproduces `fh6_kit::perspective::Camera25` (pitched, no yaw). Blender's
/// camera is right-handed and the kit's is not: chase renders are mirrored left-right in post.
pub const SHOTS: &str = r#"
# ---- shots: every TeslaAdas car render (beauty, glow, brake, parked) ----
P = ARGS


def chase_cam(sc, shot):
    """The cluster's chase camera (`fh6_kit::perspective::Camera25`, pitched, no yaw): the car's
    ground centre straight ahead at depth `d` and `h` below the eye; the focal gives `s` px per
    metre at that point and the lens shift puts it at (0.5, ay) of the image."""
    h, D = float(shot['h']), float(shot['d'])
    pitch = math.radians(float(shot['pitch_deg']))
    rx, ry = int(shot['rx']), int(shot['ry'])
    s, c = math.sin(pitch), math.cos(pitch)
    zc = D * c + h * s
    yc = -h * c + D * s
    f = float(shot['s']) * zc
    below = -f * yc / zc
    resolution(sc, rx, ry)
    cd = bpy.data.cameras.new('chase')
    cd.sensor_fit = 'HORIZONTAL'
    cd.sensor_width = 36.0
    cd.lens = f * 36.0 / rx
    cd.shift_x = 0.0
    cd.shift_y = ((float(shot['ay']) - 0.5) * ry - below) / max(rx, ry)
    cd.clip_start = 0.05
    cd.clip_end = 400.0
    cam = bpy.data.objects.new('chase', cd)
    cam.location = (0.0, D, h)
    cam.rotation_euler = (math.radians(90.0) - pitch, 0.0, math.pi)
    link(cam)
    sc.camera = cam


def night_card(path, gain):
    """The Wangan's lamps over the car: the night plate as a light card above the road, its long
    side along the road. The camera does not see it; the paint and the glass roof reflect it, and
    its light tints the body. Without the file the car is lit by the studio alone."""
    if not path or not os.path.isfile(path):
        log('night plate missing, studio light only:', path)
        return
    bpy.ops.mesh.primitive_plane_add(size=1.0, location=(0.0, -14.0, 6.0))
    card = bpy.context.active_object
    card.rotation_euler = (0.0, 0.0, math.radians(90.0))
    card.scale = (64.0, 64.0 / 2.4, 1.0)
    m, nt = new_mat('night')
    tex = nt.nodes.new('ShaderNodeTexImage')
    tex.image = bpy.data.images.load(path, check_existing=True)
    tex.extension = 'EXTEND'
    e = nt.nodes.new('ShaderNodeEmission')
    e.inputs['Strength'].default_value = gain
    o = nt.nodes.new('ShaderNodeOutputMaterial')
    nt.links.new(tex.outputs['Color'], e.inputs['Color'])
    nt.links.new(e.outputs['Emission'], o.inputs['Surface'])
    card.data.materials.append(m)
    card.visible_camera = False


def black_world(sc):
    w = bpy.data.worlds.new('void')
    sc.world = w
    w.use_nodes = True
    bg = w.node_tree.nodes.get('Background')
    if bg is not None:
        bg.inputs['Strength'].default_value = 0.0


def ground_plane(grey):
    bpy.ops.mesh.primitive_plane_add(size=80.0, location=(0.0, 0.0, 0.0))
    ground = bpy.context.active_object
    m, nt = new_mat('ground')
    d = nt.nodes.new('ShaderNodeBsdfDiffuse')
    d.inputs['Color'].default_value = (grey, grey, grey, 1.0)
    o = nt.nodes.new('ShaderNodeOutputMaterial')
    nt.links.new(d.outputs['BSDF'], o.inputs['Surface'])
    ground.data.materials.append(m)
    return ground


for shot in P['shots']:
    sc = reset()
    kind = shot['kind']
    mode = shot['pass']
    gpu_cycles(sc, samples=int(shot['samples']), denoise=True, bounces=8)
    sc.render.film_transparent = True
    lit = mode in ('beauty', 'parked')
    tail = float(P['tail_running']) if lit else (float(P['tail_brake']) if mode == 'brake' else 0.0)
    head = float(P['head_parked']) if mode == 'parked' else (float(P['head_running']) if mode == 'beauty' else 0.0)
    mats = car_mats(shot['paint'], float(P['metallic']), float(P['rough']), tail, head, plate=KINDS[kind]['plate'])
    root = build_car(kind, mats)
    root.rotation_euler = (0.0, 0.0, math.radians(float(shot.get('yaw_deg', 0.0))))
    if lit:
        studio_world(sc, strength=float(P['world']), hdri='studio.exr', rotation=0.0, visible=False)
        # The road under the car shades its underside and shows in the paint, but the camera does
        # not see it. A shadow catcher would bake black-with-alpha round the car, and the overlay's
        # 8-bit premultiplied output shows that as a lighter box over the faint road surface.
        ground_plane(0.06).visible_camera = False
        night_card(P.get('night'), float(P.get('night_gain', 0.0)))
        # Studio lights fixed to the world (the car turns under them). Driving: an overhead softbox,
        # two long strips for the highlight lines along the shoulders and a cool rim from ahead.
        # Parked: no softbox (it would fill the windscreen with one white patch); a long strip
        # over the centre line and a strip each side draw the roof and the shoulders instead.
        if mode == 'parked':
            area_light('spine', (0.0, 0.0, 4.6), (0.0, 0.0, 0.0), 0.45, float(P['key']) * 0.9, size_y=8.0)
            for sx, gain in ((-1.0, 1.25), (1.0, 0.55)):
                area_light('strip', (sx * 3.6, -0.4, 2.3), (0.0, 0.0, 0.55), 0.40, float(P['strip']) * gain, size_y=8.0)
            area_light('nose', (0.0, -6.5, 1.6), (0.0, -2.0, 0.6), 3.2, float(P['rim']) * 0.5, size_y=0.5)
            area_light('rim', (0.0, 7.5, 2.6), (0.0, 0.0, 0.8), 4.0, float(P['rim']), color=(0.80, 0.90, 1.0), size_y=1.0)
        else:
            area_light('softbox', (0.0, 1.2, 5.2), (0.0, 0.0, 0.0), 4.6, float(P['key']), size_y=2.6)
            for sx in (-1.0, 1.0):
                area_light('strip', (sx * 3.4, 0.6, 2.7), (0.0, 0.0, 0.6), 0.32, float(P['strip']), size_y=7.5)
            area_light('rim', (0.0, -7.5, 2.4), (0.0, 0.0, 0.7), 4.0, float(P['rim']), color=(0.80, 0.90, 1.0), size_y=1.0)
        view(sc, 'AgX', exposure=float(P['exposure']))
    else:
        black_world(sc)
        view(sc, 'Standard')
    if mode == 'glow':
        # Only a panel of light under the floor: the pool on the road and its bounce on the sills.
        ground_plane(0.8)
        ld = bpy.data.lights.new('underglow', 'AREA')
        ld.shape = 'RECTANGLE'
        ld.size = float(P['glow_w'])
        ld.size_y = float(P['glow_l'])
        ld.energy = float(P['glow_energy'])
        lo = bpy.data.objects.new('underglow', ld)
        lo.location = (0.0, 0.0, float(P['glow_h']))
        lo.rotation_euler = root.rotation_euler
        try:
            lo.visible_camera = False
        except Exception as e:
            log('light visibility', e)
        link(lo)
    elif mode == 'brake':
        ground_plane(0.5)
    if shot['cam'] == 'chase':
        chase_cam(sc, shot)
    else:
        resolution(sc, int(shot['rx']), int(shot['ry']))
        persp_cam(sc, tuple(shot['loc']), tuple(shot['target']), lens=float(shot['lens']))
    exr = mode in ('glow', 'brake')
    save(sc, os.path.join(OUT, shot['file']), exr=exr, depth='8')
done()
"#;

/// The chase camera's eye height above the road (metres).
pub const CAM_H: f32 = 3.2;
/// Its distance behind the ego's ground centre (metres).
pub const CAM_D: f32 = 7.2;
/// Its downward pitch (degrees).
pub const PITCH_DEG: f32 = 16.0;
/// The yaw of each ego cell (degrees, positive = nose to the right), row-major in an
/// [`EGO_COLS`]-wide grid.
pub const EGO_YAWS: [f32; 11] = [-40.0, -32.0, -24.0, -16.0, -8.0, 0.0, 8.0, 16.0, 24.0, 32.0, 40.0];
/// Ego atlas columns (3 rows hold the 11 cells).
pub const EGO_COLS: usize = 4;
/// Ego cell (render px).
pub const EGO_CELL: [usize; 2] = [1024, 640];
/// Ego scale (render px per metre at the car's ground centre).
pub const EGO_S: f32 = 176.0;
/// Where the car's ground centre sits in a cell (fractions of width and height).
pub const EGO_ANCHOR: [f32; 2] = [0.5, 0.46];
/// Traffic cell (render px).
pub const TRAFFIC_CELL: [usize; 2] = [336, 232];
/// Traffic scale (render px per metre).
pub const TRAFFIC_S: f32 = 64.0;
/// Traffic anchor.
pub const TRAFFIC_ANCHOR: [f32; 2] = [0.5, 0.70];
/// Traffic view elevations (degrees below the horizon to the car's ground centre), atlas rows.
pub const TRAFFIC_ELEVATIONS: [f32; 4] = [3.0, 6.0, 10.0, 16.0];
/// Traffic headings relative to the view ray (degrees, nose to the right), atlas columns; the
/// cluster mirrors a cell for a negative angle.
pub const TRAFFIC_HEADINGS: [f32; 5] = [0.0, 10.0, 20.0, 30.0, 40.0];
/// The traffic bodies ([`CAR_LIB`]'s `KINDS`).
pub const TRAFFIC_KINDS: [&str; 3] = ["sedan", "kei", "sport"];
/// The parked image (render px).
pub const PARKED_SIZE: [usize; 2] = [2048, 1152];
/// The ego's pearl white and the traffic's grey (linear RGB).
const EGO_PAINT: [f32; 3] = [0.80, 0.81, 0.82];
const TRAFFIC_PAINT: [f32; 3] = [0.42, 0.44, 0.47];
const CAR_SEED: u64 = 152;

/// The tunnel lights plate, relative to the repository root.
pub const NIGHT_FILE: &str = "assets/clusters/tesla_adas/tunnel_lights.png";
const NIGHT_PROMPT: &str = "long exposure photograph looking straight up at the ceiling of a Japanese expressway tunnel at night, two parallel horizontal rows of glowing orange sodium ceiling lamps running across the whole frame from left to right, a few cold white lights, dark concrete ceiling, soft glow, light haze, flat frontal view, symmetrical";
const NIGHT_NEGATIVE: &str = "vanishing point, perspective, road, cars, vehicle, people, text, letters, signs, watermark, logo, daylight, sky, lowres, blurry, jpeg artifacts";
const NIGHT_SEED: u64 = 6006;
/// The shipped plate: the long side lies along the road.
const NIGHT_SIZE: [usize; 2] = [4096, 1706];
/// The card's emission strength in the car shots.
const NIGHT_GAIN: f32 = 2.5;

/// Builds the set.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    font(ctx)?;
    night(ctx)?;
    ego(ctx)?;
    parked(ctx)?;
    traffic(ctx)
}

/// The Japanese label face, relative to the repository root.
pub const FONT_FILE: &str = "assets/clusters/tesla_adas/NotoSansJP-Medium-subset.ttf";
const FONT_LICENCE: &str = "assets/clusters/tesla_adas/NotoSansJP-OFL.txt";
/// The font pack's own google/fonts commit, so the Japanese face matches the pack's Noto Sans.
const FONT_SOURCE: &str =
    "https://github.com/google/fonts/tree/06bfe0d62a98375cb09c942fa18e68bd32cff057/ofl/notosansjp";
/// [`fingerprint`] of the vendored file.
const FONT_FINGERPRINT: &str = "b66f47a060dd3699";

/// Checks the vendored Japanese face and enters it and its licence in the ledger.
fn font(ctx: &mut Ctx) -> Result<()> {
    let path = ctx.abs(FONT_FILE);
    let bytes = std::fs::read(&path).map_err(|e| io_err(&path, e))?;
    let fp = fingerprint(&[&bytes]);
    if fp != FONT_FINGERPRINT {
        return Err(tool_err(
            "tesla_adas/font",
            format!("{FONT_FILE} is not the vendored subset ({fp}, expected {FONT_FINGERPRINT})"),
        ));
    }
    ctx.job("tesla_adas/font", &fp, |ctx| {
        let made = format!("vendored: NotoSansJP[wght].ttf from {FONT_SOURCE}, instanced at wght 500 with overlaps removed, then subset (fontTools 4.69.1; the commands are in ASSETS.md)");
        ctx.file("tesla_adas/font/noto_sans_jp_medium", FONT_FILE, "700 glyphs".into(), Spec::new(SET, "font", made, CLUSTERS).space("n/a").licence("OFL-1.1"), None)?;
        ctx.file("tesla_adas/font/licence", FONT_LICENCE, "text".into(), Spec::new(SET, "licence", format!("{FONT_SOURCE}/OFL.txt"), CLUSTERS).space("n/a").licence("OFL-1.1"), None)
    })
}

/// What every car shot shares: the studio (world, softbox, strips, rim, exposure), the paint's
/// metallic and roughness, the lamps' emission per pass and the glow panel.
fn studio() -> Value {
    json!({
        "seed": CAR_SEED, "world": 0.30, "metallic": 0.25, "rough": 0.30, "key": 520.0, "strip": 560.0,
        "rim": 320.0, "exposure": -0.7, "tail_running": 1.4, "tail_brake": 14.0, "head_running": 0.0,
        "head_parked": 6.0, "glow_w": 1.9, "glow_l": 4.6, "glow_h": 0.14, "glow_energy": 520.0,
        "night_gain": NIGHT_GAIN
    })
}

/// The Blender source of a car job.
fn script() -> String {
    format!("{CAR_LIB}\n{SHOTS}")
}

/// One chase-camera shot of `kind` at `yaw` degrees, `s` px per metre in a `cell`, its ground
/// centre `ay` down the cell, from `h` above and `d` behind.
#[allow(clippy::too_many_arguments)]
fn chase(
    kind: &str,
    pass: &str,
    paint: [f32; 3],
    yaw: f32,
    h: f32,
    d: f32,
    s: f32,
    cell: [usize; 2],
    ay: f32,
    samples: u32,
    file: &str,
) -> Value {
    json!({
        "kind": kind, "pass": pass, "paint": paint, "yaw_deg": yaw, "h": h, "d": d, "pitch_deg": PITCH_DEG,
        "s": s, "rx": cell[0], "ry": cell[1], "ay": ay, "cam": "chase", "samples": samples, "file": file
    })
}

/// The job's fingerprint: the shared prelude, both scripts, the parameters, the tunnel lights
/// plate as it is on disk and a version tag.
fn car_fingerprint(ctx: &Ctx, params: &Value, tag: &[u8]) -> String {
    let plate = std::fs::read(ctx.abs(NIGHT_FILE)).unwrap_or_default();
    fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        CAR_LIB.as_bytes(),
        SHOTS.as_bytes(),
        params.to_string().as_bytes(),
        &plate,
        tag,
    ])
}

/// A job's parameters with the paths only this machine knows: the scratch folder and the plate.
fn with_paths(ctx: &Ctx, params: &Value, out: &std::path::Path) -> Value {
    let mut p = params.clone();
    p["out"] = json!(out.to_string_lossy());
    p["night"] = json!(ctx.abs(NIGHT_FILE).to_string_lossy());
    p
}

/// The ego atlas and its two light masks.
fn ego(ctx: &mut Ctx) -> Result<()> {
    let half = [EGO_CELL[0] / 2, EGO_CELL[1] / 2];
    let mut shots = Vec::new();
    for (i, yaw) in EGO_YAWS.iter().enumerate() {
        let shot = |pass: &str, s: f32, cell: [usize; 2], samples: u32, ext: &str| {
            let file = format!("{pass}_{i}.{ext}");
            chase(
                "ego",
                pass,
                EGO_PAINT,
                *yaw,
                CAM_H,
                CAM_D,
                s,
                cell,
                EGO_ANCHOR[1],
                samples,
                &file,
            )
        };
        shots.push(shot("beauty", EGO_S, EGO_CELL, 128, "png"));
        shots.push(shot("glow", EGO_S * 0.5, half, 96, "exr"));
        shots.push(shot("brake", EGO_S * 0.5, half, 64, "exr"));
    }
    let mut params = studio();
    params["shots"] = json!(shots);
    let fp = car_fingerprint(ctx, &params, b"ego-v2");
    ctx.job("tesla_adas/ego", &fp, |ctx| {
        let out = ctx.scratch("tesla_adas_ego")?;
        let p = with_paths(ctx, &params, &out);
        blender::run(&ctx.paths, "tesla_adas_ego", &script(), &p)?;
        let rows = EGO_YAWS.len().div_ceil(EGO_COLS);
        let mut atlas = Img::new(EGO_CELL[0] * EGO_COLS, EGO_CELL[1] * rows, 4);
        let mut glow = Img::new(half[0] * EGO_COLS, half[1] * rows, 4);
        let mut brake = Img::new(half[0] * EGO_COLS, half[1] * rows, 4);
        // One exposure for every yaw (the straight-ahead cell's), so the cells crossfade evenly.
        let straight = EGO_YAWS.len() / 2;
        let glow_peak = percentile(&img::load_exr(&out.join(format!("glow_{straight}.exr")))?, 0.997);
        let brake_peak = percentile(&img::load_exr(&out.join(format!("brake_{straight}.exr")))?, 0.9995);
        for i in 0..EGO_YAWS.len() {
            let (cx, cy) = (i % EGO_COLS, i / EGO_COLS);
            let body = flip_x(&img::load(&out.join(format!("beauty_{i}.png")))?);
            atlas.paste(&body, cx * EGO_CELL[0], cy * EGO_CELL[1]);
            let pool = img::load_exr(&out.join(format!("glow_{i}.exr")))?;
            let pool = flip_x(&light_mask(&pool, glow_peak, 4, 0.015, 0.75, feather(0.08, 0.08)));
            glow.paste(&pool, cx * half[0], cy * half[1]);
            let lamps = img::load_exr(&out.join(format!("brake_{i}.exr")))?;
            let lamps = flip_x(&light_mask(&lamps, brake_peak, 1, 0.004, 0.8, feather(0.04, 0.04)));
            brake.paste(&lamps, cx * half[0], cy * half[1]);
        }
        let gen = format!(
            "blender:tesla_adas::CAR_LIB + SHOTS (Cycles, OptiX denoise; the lofted fastback through the chase camera {CAM_H} m up, {CAM_D} m behind, {PITCH_DEG} deg down, yaw -40..40 step 8) seed={CAR_SEED}"
        );
        let layout = format!(
            "{EGO_COLS} x {rows} atlas (yaw -40..40 step 8, row-major); the car's ground centre sits at ({}, {}) of each cell; mirrored left-right for the kit",
            EGO_ANCHOR[0], EGO_ANCHOR[1]
        );
        ctx.image(
            "tesla_adas/ego/body",
            "assets/clusters/tesla_adas/ego.png",
            &atlas,
            Format::Png8,
            Spec::new(SET, "sprite", format!("{gen}; beauty pass, 128 spp, AgX -0.7 EV, pearl white, glass roof, light bar at running level"), CLUSTERS)
                .alpha("straight")
                .qa(format!("{} x {} cells at {EGO_S} px per metre; {layout}", EGO_CELL[0], EGO_CELL[1])),
        )?;
        ctx.image(
            "tesla_adas/ego/glow",
            "assets/clusters/tesla_adas/ego_glow.png",
            &glow,
            Format::Png8,
            Spec::new(SET, "mask", format!("{gen}; glow pass, 96 spp: a 1.9 x 4.6 m light panel 0.14 m under the floor, the car opaque, a pale ground"), CLUSTERS)
                .alpha("straight")
                .space("linear")
                .qa(format!("{} x {} cells; white RGB, alpha = light (softened 4x, one exposure for all yaws, black point 1.5%, gamma 0.75, feathered); tinted live and added over the car; {layout}", half[0], half[1])),
        )?;
        ctx.image(
            "tesla_adas/ego/brake",
            "assets/clusters/tesla_adas/ego_brake.png",
            &brake,
            Format::Png8,
            Spec::new(SET, "mask", format!("{gen}; brake pass, 64 spp: only the tail bar's emission, its spill on the bumper and the road"), CLUSTERS)
                .alpha("straight")
                .space("linear")
                .qa(format!("{} x {} cells; white RGB, alpha = light (one exposure for all yaws, gamma 0.8); tinted warning red and added live while braking; {layout}", half[0], half[1])),
        )
    })
}

/// The parked car: a three-quarter front view under the parked lighting.
fn parked(ctx: &mut Ctx) -> Result<()> {
    let mut params = studio();
    params["shots"] = json!([{
        "kind": "ego", "pass": "parked", "paint": EGO_PAINT, "yaw_deg": 0.0, "cam": "look",
        "rx": PARKED_SIZE[0], "ry": PARKED_SIZE[1], "loc": [-3.6, -6.9, 2.6], "target": [0.15, 0.2, 0.50],
        "lens": 64.0, "samples": 256, "file": "parked.png"
    }]);
    let fp = car_fingerprint(ctx, &params, b"parked-v2");
    ctx.job("tesla_adas/parked", &fp, |ctx| {
        let out = ctx.scratch("tesla_adas_parked")?;
        let p = with_paths(ctx, &params, &out);
        blender::run(&ctx.paths, "tesla_adas_parked", &script(), &p)?;
        let image = img::load(&out.join("parked.png"))?;
        ctx.image(
            "tesla_adas/parked",
            "assets/clusters/tesla_adas/ego_parked.png",
            &image,
            Format::Png8,
            Spec::new(
                SET,
                "sprite",
                format!("blender:tesla_adas::CAR_LIB + SHOTS (Cycles 256 spp, OptiX denoise, AgX -0.7 EV; the lofted fastback from the front left, 64 mm, a strip light over the centre line and one each side, running lights on) seed={CAR_SEED}"),
                CLUSTERS,
            )
            .alpha("straight")
            .qa("the parked layout's car image; drawn centred in the scene while the car stands still in neutral"),
        )
    })
}

/// The traffic atlases: each body over a grid of view elevations and relative headings.
fn traffic(ctx: &mut Ctx) -> Result<()> {
    let mut shots = Vec::new();
    for kind in TRAFFIC_KINDS {
        for (r, el) in TRAFFIC_ELEVATIONS.iter().enumerate() {
            let d = CAM_H / el.to_radians().tan();
            for (c, yaw) in TRAFFIC_HEADINGS.iter().enumerate() {
                let file = format!("{kind}_{r}_{c}.png");
                shots.push(chase(
                    kind,
                    "beauty",
                    TRAFFIC_PAINT,
                    *yaw,
                    CAM_H,
                    d,
                    TRAFFIC_S,
                    TRAFFIC_CELL,
                    TRAFFIC_ANCHOR[1],
                    96,
                    &file,
                ));
            }
        }
    }
    let mut params = studio();
    params["shots"] = json!(shots);
    let fp = car_fingerprint(ctx, &params, b"traffic-v2");
    ctx.job("tesla_adas/traffic", &fp, |ctx| {
        let out = ctx.scratch("tesla_adas_traffic")?;
        let p = with_paths(ctx, &params, &out);
        blender::run(&ctx.paths, "tesla_adas_traffic", &script(), &p)?;
        let cols = TRAFFIC_HEADINGS.len();
        for kind in TRAFFIC_KINDS {
            let mut atlas = Img::new(TRAFFIC_CELL[0] * cols, TRAFFIC_CELL[1] * TRAFFIC_ELEVATIONS.len(), 4);
            for r in 0..TRAFFIC_ELEVATIONS.len() {
                for c in 0..cols {
                    let cell = flip_x(&img::load(&out.join(format!("{kind}_{r}_{c}.png")))?);
                    atlas.paste(&cell, c * TRAFFIC_CELL[0], r * TRAFFIC_CELL[1]);
                }
            }
            ctx.image(
                &format!("tesla_adas/traffic/{kind}"),
                &format!("assets/clusters/tesla_adas/traffic_{kind}.png"),
                &atlas,
                Format::Png8,
                Spec::new(
                    SET,
                    "sprite",
                    format!("blender:tesla_adas::CAR_LIB + SHOTS (Cycles 96 spp, OptiX denoise, AgX -0.7 EV; the `{kind}` body in grey paint) seed={CAR_SEED}"),
                    CLUSTERS,
                )
                .alpha("straight")
                .qa(format!(
                    "{cols} x {} atlas of {} x {} cells: rows = view elevation {TRAFFIC_ELEVATIONS:?} deg, columns = heading relative to the view ray {TRAFFIC_HEADINGS:?} deg; {TRAFFIC_S} px per metre; the ground centre at ({}, {}) of each cell; mirrored left-right for the kit",
                    TRAFFIC_ELEVATIONS.len(), TRAFFIC_CELL[0], TRAFFIC_CELL[1], TRAFFIC_ANCHOR[0], TRAFFIC_ANCHOR[1]
                )),
            )?;
        }
        Ok(())
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

/// The luminance of `src` (times its alpha when it has one).
fn luminance(src: &Img) -> Img {
    Img::from_fn(src.w, src.h, 1, |x, y| {
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
    })
}

/// The luminance at quantile `q` (0..1) of `src`, never below a tiny floor.
fn percentile(src: &Img, q: f32) -> f32 {
    let mut sorted = luminance(src).data;
    sorted.sort_by(f32::total_cmp);
    let last = sorted.len().saturating_sub(1);
    let at = ((sorted.len() as f32 * q.clamp(0.0, 1.0)) as usize).min(last);
    sorted.get(at).copied().unwrap_or(0.0).max(1e-6)
}

/// Luminance → alpha over white: softened by a down-and-up resample (`soften` = the reduction
/// factor; 1 keeps it sharp), divided by `peak`, everything at or below `black` set to exactly 0,
/// shaped by `gamma` (< 1 lifts a light's falloff tail) and multiplied by `shape(u, v)` (feathers
/// and fades; `u`, `v` in 0..1).
fn light_mask(src: &Img, peak: f32, soften: usize, black: f32, gamma: f32, shape: impl Fn(f32, f32) -> f32) -> Img {
    let mut lum = luminance(src);
    if soften > 1 {
        let small = lum.resize((src.w / soften).max(2), (src.h / soften).max(2), false);
        lum = small.resize(src.w, src.h, false);
    }
    let peak = peak.max(1e-6);
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

/// The tunnel lights plate: SDXL's ceiling lamps, reduced to their light (the concrete between
/// the lamps goes to black, so the card adds lamps and no grey haze).
fn night(ctx: &mut Ctx) -> Result<()> {
    let wf_path = ctx.paths.forge_dir().join("workflows").join("sdxl_plate.json");
    let workflow = String::from_utf8_lossy(&crate::ctx::read(&wf_path)?).to_string();
    let gen_fp = fingerprint(&[
        workflow.as_bytes(),
        NIGHT_PROMPT.as_bytes(),
        NIGHT_NEGATIVE.as_bytes(),
        &NIGHT_SEED.to_le_bytes(),
        b"1536x640",
    ]);
    let fp = fingerprint(&[gen_fp.as_bytes(), b"night-post-v1"]);
    ctx.job("tesla_adas/night", &fp, |ctx| {
        let raw_path = ctx.scratch("comfy")?.join("tesla_adas_night.png");
        let raw_fp_path = raw_path.with_extension("fp");
        let cached = raw_path.exists() && std::fs::read_to_string(&raw_fp_path).map(|s| s == gen_fp).unwrap_or(false);
        if !cached {
            let c = Comfy::connect(&ctx.paths)?;
            let mut params = Map::new();
            params.insert("prompt".into(), json!(NIGHT_PROMPT));
            params.insert("negative".into(), json!(NIGHT_NEGATIVE));
            params.insert("seed".into(), json!(NIGHT_SEED));
            params.insert("steps".into(), json!(32));
            params.insert("cfg".into(), json!(6.5));
            params.insert("width".into(), json!(1536));
            params.insert("height".into(), json!(640));
            params.insert("prefix".into(), json!("fh6_tesla_adas_night"));
            let wf = comfy::patch(&workflow, &params)?;
            let images = c.run(&wf, Duration::from_secs(900))?;
            let bytes = images.first().ok_or_else(|| tool_err("comfyui", "no image"))?;
            img::write_file(&raw_path, bytes)?;
            img::write_file(&raw_fp_path, gen_fp.as_bytes())?;
        }
        // 6144 x 2560 from the upscaler. Keep the light: the median luminance (the ceiling) goes
        // to black and the lamps' peak to full, each pixel keeping its colour.
        let raw = img::load(&raw_path)?;
        let small = raw.resize(NIGHT_SIZE[0], NIGHT_SIZE[1], false);
        let (black, peak) = (percentile(&small, 0.5), percentile(&small, 0.999));
        let plate = lights_only(&small, black, peak);
        ctx.image(
            "tesla_adas/night",
            NIGHT_FILE,
            &plate,
            Format::Png8,
            Spec::new(
                SET,
                "texture",
                format!(
                    "comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5, 1536 x 640) seed={NIGHT_SEED} prompt=\"{NIGHT_PROMPT}\""
                ),
                CLUSTERS,
            )
            .licence("CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge)")
            .qa(format!(
                "the 6144 x 2560 upscale at {} x {}, its median luminance taken to black and its 99.9th percentile to full; a build input: every lit car shot hangs it 6 m over the road as a light card, long side along the road, emission x {NIGHT_GAIN}",
                NIGHT_SIZE[0], NIGHT_SIZE[1]
            )),
        )
    })
}

/// The light in `src`: luminance `black` and below goes to zero and `peak` and above to full,
/// each pixel keeping its hue (its brightest channel carries the level).
fn lights_only(src: &Img, black: f32, peak: f32) -> Img {
    let span = (peak - black).max(1e-6);
    Img::from_fn(src.w, src.h, 3, |x, y| {
        let rgb = [
            src.get(x, y, 0),
            src.get(x, y, 1.min(src.c - 1)),
            src.get(x, y, 2.min(src.c - 1)),
        ];
        let level = ((img::luma(rgb[0], rgb[1], rgb[2]) - black) / span).clamp(0.0, 1.0);
        let k = img::srgb_encode(level) / rgb[0].max(rgb[1]).max(rgb[2]).max(1e-6);
        [rgb[0] * k, rgb[1] * k, rgb[2] * k, 1.0]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ego_cells_cover_the_yaws_and_the_camera_sees_the_car() {
        assert_eq!(EGO_YAWS.len().div_ceil(EGO_COLS), 3);
        assert!(EGO_YAWS.windows(2).all(|w| (w[1] - w[0] - 8.0).abs() < 1e-6));
        // The car is 4.85 m long: at 40 degrees of yaw its plan diagonal still fits the cell.
        let (l, w) = (4.85f32, 1.96f32);
        let half = 0.5 * (l * 40f32.to_radians().sin() + w * 40f32.to_radians().cos());
        assert!(half * EGO_S * 1.1 < EGO_CELL[0] as f32 * 0.5, "{half}");
    }

    #[test]
    fn the_night_plate_keeps_lamps_and_drops_the_ceiling() {
        let px = [
            [0.10, 0.10, 0.10],
            [0.05, 0.05, 0.05],
            [0.80, 0.40, 0.10],
            [0.40, 0.20, 0.05],
        ];
        let src = Img::from_fn(4, 1, 3, |x, _| [px[x][0], px[x][1], px[x][2], 0.0]);
        let (black, peak) = (img::luma(0.10, 0.10, 0.10), img::luma(0.80, 0.40, 0.10));
        let out = lights_only(&src, black, peak);
        assert!(
            out.get(0, 0, 0).abs() < 1e-6 && out.get(1, 0, 1).abs() < 1e-6,
            "the ceiling is black"
        );
        assert!((out.get(2, 0, 0) - 1.0).abs() < 1e-5, "a lamp at the peak is full");
        assert!(
            (out.get(2, 0, 0) / out.get(2, 0, 2) - 8.0).abs() < 1e-3,
            "a lamp keeps its hue"
        );
        let dim = out.get(3, 0, 0);
        assert!(dim > 0.1 && dim < 0.9, "a dimmer lamp stays dimmer: {dim}");
    }

    #[test]
    fn a_light_mask_is_white_with_the_light_in_alpha() {
        let src = Img::from_fn(8, 4, 3, |x, _| {
            let v = x as f32 / 7.0;
            [v, v, v, 0.0]
        });
        let peak = percentile(&src, 1.0);
        assert!((peak - 1.0).abs() < 1e-5);
        let mask = light_mask(&src, peak, 1, 0.0, 1.0, |_, _| 1.0);
        assert_eq!(mask.c, 4);
        assert!((mask.get(7, 0, 3) - 1.0).abs() < 1e-5);
        assert!(mask.get(0, 0, 3).abs() < 1e-6);
        assert!((mask.get(3, 2, 0) - 1.0).abs() < 1e-6);
    }
}
