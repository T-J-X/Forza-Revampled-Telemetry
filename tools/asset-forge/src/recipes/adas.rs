//! ADAS cars: low-poly stylised bodies lofted from parametric cross-sections, with wheels,
//! exported as binary glTF in the kit's convention (Y up, nose towards +Z, centred, ground at
//! y = 0) plus a rim-lit chase-view sprite for the sprite fallback.

/// Params: `cars: [{name, L, W, H, ride, wr, wb, track, hood, cabin_front, cabin_rear,
/// roof_peak, deck, paint}]`, `sprite: {w, h, samples}`. Writes `<name>.glb`, `<name>.json`
/// (triangle count) and `<name>_sprite.png`.
pub const CARS: &str = r#"
def smooth(e0, e1, x):
    t = min(1.0, max(0.0, (x - e0) / (e1 - e0)))
    return t * t * (3.0 - 2.0 * t)

def make_mats(c):
    out = []
    mats = [
        ('paint', tuple(c.get('paint', (0.75, 0.77, 0.8))), 0.25, 1.0),
        ('glass', (0.01, 0.012, 0.015), 0.05, 0.0),
        ('tyre', (0.02, 0.02, 0.02), 0.8, 0.0),
        ('tail', (0.9, 0.02, 0.02), 0.3, 0.0),
        ('head', (0.95, 0.95, 1.0), 0.3, 0.0),
    ]
    for (nm, colr, rough, metal) in mats:
        m = bpy.data.materials.new(c['name'] + '_' + nm)
        try:
            m.use_nodes = True
            p = m.node_tree.nodes.get('Principled BSDF')
            p.inputs['Base Color'].default_value = (colr[0], colr[1], colr[2], 1.0)
            p.inputs['Roughness'].default_value = rough
            p.inputs['Metallic'].default_value = metal
            if nm in ('tail', 'head'):
                p.inputs['Emission Color'].default_value = (colr[0], colr[1], colr[2], 1.0)
                p.inputs['Emission Strength'].default_value = 2.0
        except Exception as e:
            log('material', nm, e)
        m.diffuse_color = (colr[0], colr[1], colr[2], 1.0)
        out.append(m)
    return out

def build_car(c):
    mats = make_mats(c)
    L, W, H = c['L'], c['W'], c['H']
    hl, hw = L * 0.5, W * 0.5
    ride, wr, wb = c['ride'], c['wr'], c['wb']
    wheels_x = (wb * 0.5 + c.get('wb_shift', 0.0), -wb * 0.5 + c.get('wb_shift', 0.0))
    belt = c['belt'] * H
    hood = c['hood'] * H
    deck = c['deck'] * H
    xa, xc = c['cabin_front'] * hl, c['cabin_rear'] * hl
    roof = H
    nx = 46
    ring = []
    bm = bmesh.new()
    rings = []
    for i in range(nx + 1):
        t = i / nx
        x = hl - L * t
        # plan taper: superellipse nose/tail
        u = abs(x) / hl
        taper = (max(0.0, 1.0 - u ** c.get('plan_n', 6.0))) ** (1.0 / c.get('plan_n', 6.0))
        w = hw * max(0.18, taper)
        # bottom with wheel arches
        z0 = ride
        for xw in wheels_x:
            d = abs(x - xw)
            ra = wr * 1.18
            if d < ra:
                z0 = max(z0, ride + (math.sqrt(ra * ra - d * d) - (ra - wr * 1.05)) * 0.95)
        nose_drop = smooth(hl * 0.62, hl, x) * 1.0 + smooth(-hl * 0.7, -hl, x) * 0.55
        zb_front = hood + (belt - hood) * smooth(hl * 0.75, xa, x) if x > xa else belt
        zb_rear = deck + (belt - deck) * smooth(-hl * 0.85, xc, x) if x < xc else belt
        zb = min(zb_front, zb_rear)
        zb = zb - (zb - ride) * nose_drop * 0.55
        # greenhouse height
        if xc < x < xa:
            # s runs from the windscreen base (0) to the end of the backlight (1): the windscreen
            # rises over [0, peak], the roof holds, the backlight falls over the last `tail`.
            s = (xa - x) / (xa - xc)
            peak = c.get('roof_peak', 0.45)
            tail = c.get('tail', 0.3)
            g = smooth(0.0, peak, s) * (1.0 - smooth(1.0 - tail, 1.0, s))
        else:
            g = 0.0
        zr = zb + (roof - zb) * g + 0.025
        zb = min(zb, zr - 0.02)
        z0 = min(z0, zb - 0.05)
        pts = [
            (0.0, z0 - 0.01),
            (w * 0.86, z0),
            (w * 0.98, z0 + (zb - z0) * 0.22),
            (w, z0 + (zb - z0) * 0.6),
            (w * 0.985, zb - 0.02),
            (w * 0.93, zb),
            (w * (0.93 - 0.11 * g), zb + (zr - zb) * 0.35),
            (w * (0.93 - 0.2 * g), zb + (zr - zb) * 0.8),
            (w * (0.86 - 0.26 * g), zr - 0.004),
            (w * 0.4 * (1.0 - 0.2 * g), zr),
            (0.0, zr + 0.004),
        ]
        full = pts + [(-y, z) for (y, z) in reversed(pts[1:-1])]
        rings.append([bm.verts.new((x, y, z)) for (y, z) in full])
    n = len(rings[0])
    faces = []
    for i in range(nx):
        for j in range(n):
            a, b = rings[i][j], rings[i][(j + 1) % n]
            cc, d = rings[i + 1][(j + 1) % n], rings[i + 1][j]
            f = bm.faces.new((a, b, cc, d))
            faces.append((f, i, j))
    # caps
    fcap = bm.faces.new(list(reversed(rings[0])))
    rcap = bm.faces.new(rings[-1])
    bm.normal_update()
    # outward normals
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    bm.faces.ensure_lookup_table()
    # material ids: 0 body, 1 glass, 2 tyre, 3 tail light, 4 head light
    glass_rows = set(range(6, 9)) | set(range(n - 9 + 1, n - 6 + 1))
    for (f, i, j) in faces:
        x = hl - L * (i + 0.5) / nx
        if xc + 0.05 < x < xa - 0.02 and (j in (6, 7, n - 7, n - 8)):
            f.material_index = 1
        elif i <= 1 and j in (4, 5, n - 5, n - 6):
            f.material_index = 4
        elif i >= nx - 2 and j in (4, 5, n - 5, n - 6):
            f.material_index = 3
    for f in bm.faces:
        f.smooth = True
    body = mesh_obj(c['name'], bm)
    for m in mats:
        body.data.materials.append(m)
    # wheels
    parts = [body]
    for xw in wheels_x:
        for side in (-1.0, 1.0):
            bpy.ops.mesh.primitive_cylinder_add(vertices=18, radius=wr, depth=wr * 0.62, location=(xw, side * (hw - wr * 0.34), wr), rotation=(math.radians(90.0), 0.0, 0.0))
            wh = bpy.context.active_object
            for m in mats:
                wh.data.materials.append(m)
            wh.data.polygons.foreach_set('material_index', [2] * len(wh.data.polygons))
            parts.append(wh)
    bpy.ops.object.select_all(action='DESELECT')
    for p in parts:
        p.select_set(True)
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.join()
    car = bpy.context.active_object
    car.name = c['name']
    # nose to glTF +Z: build frame +X forward -> Blender -Y
    car.rotation_euler = (0.0, 0.0, math.radians(-90.0))
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)
    try:
        bpy.ops.object.shade_smooth_by_angle(angle=math.radians(38.0))
    except Exception:
        bpy.ops.object.shade_auto_smooth(angle=math.radians(38.0))
    return car

for c in ARGS['cars']:
    sc = reset()
    car = build_car(c)
    me = car.data
    me.calc_loop_triangles()
    tris = len(me.loop_triangles)
    bpy.ops.object.select_all(action='DESELECT')
    car.select_set(True)
    glb = os.path.join(OUT, c['name'] + '.glb')
    bpy.ops.export_scene.gltf(filepath=glb, export_format='GLB', use_selection=True, export_apply=True,
                              export_yup=True, export_normals=True, export_texcoords=False, export_materials='EXPORT',
                              export_animations=False, export_skins=False, export_morph=False)
    dims = [round(v, 3) for v in car.dimensions]
    with open(os.path.join(OUT, c['name'] + '.json'), 'w', encoding='utf-8') as fh:
        json.dump({'tris': tris, 'dims': dims}, fh)
    log('car', c['name'], 'tris', tris, 'dims', dims)
    # rim-lit chase-view sprite
    s = ARGS['sprite']
    gpu_cycles(sc, samples=s.get('samples', 128), denoise=True)
    view(sc, 'AgX', look='AgX - Punchy')
    resolution(sc, s['w'], s['h'])
    studio_world(sc, strength=0.35, hdri='studio.exr', rotation=0.4, visible=False)
    area_light('rim_l', (-3.5, -3.0, 2.0), (0, 0, 0.6), 2.5, 1400.0, color=(0.55, 0.85, 1.0))
    area_light('rim_r', (3.5, -3.0, 2.0), (0, 0, 0.6), 2.5, 1400.0, color=(0.55, 0.85, 1.0))
    area_light('top', (0.0, 1.5, 5.0), (0, 0, 0), 3.0, 500.0)
    persp_cam(sc, (0.0, 7.6, 3.4), (0.0, 0.0, 0.55), lens=55.0)
    save(sc, os.path.join(OUT, c['name'] + '_sprite.png'), depth='16', mode='RGBA')
done()
"#;
