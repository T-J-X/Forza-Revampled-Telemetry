# HANDOVER: Phase 6 · Assets, logos & fonts

**Chat / branch:** `claude/objective-euclid-13487b`  **Date:** 2026-11-22  **Result:** ✅ complete (known issues listed below)

## Exit criteria
| Criterion (copied from the MASTER_PROMPT) | Met? | Evidence |
|---|---|---|
| `tools/asset-forge` builds, `-- doctor` passes, `-- build` rebuilds at least one asset of every set | ✅ | `cargo build --release -p asset-forge` clean; `doctor` output and a forced rebuild of one job per set under Verification output |
| `git ls-files` lists no `.py`, `.ps1`, `.sh`, `.js`, `.ts` or `.c` files | ✅ | Check under Verification output (empty). bpy lives in `tools/asset-forge/src/recipes/*.rs` as `const &str` |
| Liquid Glass set | ✅ | `assets/glass/`: 4 plate shapes (rrect, pill, squircle, lens circle) with 4K 16-bit normal masters, KTX2 normals, refraction offsets and fresnel masks; lens caustics (dispersive); tileable caustic still and 64-frame loop; frost micro-normal; blue noise; sheen arch; droplet / meniscus / capsule liquid normals; 2 SDXL caustic plates |
| Fluid flipbooks plus flow and normal maps | ✅ | `assets/fluid/`: mercury blob, ferrofluid, mercury pool, each 64 frames (8×8 atlas of 512 px cells) + packed normal/flow KTX2 (256 px cells); loop seam scores 1.1–1.5; 5 matcaps; 4 Chladni plates; curl-noise flow; 6 SDXL plates |
| Sci-fi set | ✅ | `assets/scifi/`: holo floor grid, 32-frame warp tunnel, reticle sheet (SVG + PNG), HUD masks (square / dot / hatch grids, 2/3 px scanlines, VFD hex mesh, phosphor triad), 5 SDXL plates |
| Classic materials | ✅ | `assets/classic/`: 11 materials (brushed titanium, guilloché barleycorn / clous de Paris / wave, carbon twill 2×2, forged carbon, anodised aluminium black and blue, white enamel, Alcantara, leather) as albedo + lit + normal + roughness/metal/height; 3 dial faces (white enamel, rose guilloché, slate sunburst); 5 bezel profiles with normal, AO and 2 lit finishes |
| ADAS glTF cars | ✅ | `assets/adas/`: ego coupé, sport sedan, SUV, supercar, hypercar `.glb` (2,148 tris each, ≤ 5k) + chase sprites; road/lane decal sheet (SVG + PNG) |
| Logos for 100% of brands, with `logos.json` | ✅ | `assets/logos/`: 91 entries covering every `BrandPalettes` key and every `Cars` make (plus `DEFAULT`), each SVG + 4096 full + 4096 mono + 1024 dial; `logos.json` with aliases; `NOTICE.md` |
| Font pack with `fonts.json` | ✅ | `assets/fonts/`: 17 OFL families from google/fonts at commit `06bfe0d6`, each with `OFL.txt`; `fonts.json` (files, weights, axes, roles, default flag, `tnum`, monospaced digits, cap ratio) |
| `ASSETS.md` with a licence for every file | ✅ | `assets/ASSETS.md` (generated): one row per asset-forge file, plus a table for the files from the initial import |
| Contact sheet reviewed; critique notes and fixes in the handover | ✅ | `assets/contact-sheet.html`; see Critique below |
| Total repo growth reported | ✅ | See Repo growth |

## What was built
- `tools/asset-forge/` (Rust binary, in the workspace): `doctor`, `build [--set] [--force] [--only]`, `sheet`.
  - `src/recipes/*.rs`: Blender 5.3 bpy recipes as Rust strings (glass plates and liquid, fluid flipbooks and matcaps, classic materials / dials / bezels, holo grid and warp tunnel, ADAS cars), sharing one `PRELUDE` of helpers.
  - `src/sets/*.rs`: one module per set; each job is fingerprinted (FNV-1a of recipe + params + post version) in `assets/forge.lock.json`, so a second `build` does nothing.
  - `src/comfy.rs` + `workflows/sdxl_plate.json`: ComfyUI over HTTP (`/prompt`, `/history`, `/view`), SDXL base 1.0 + 4x-UltraSharp; the raw render is cached by its own fingerprint so post-processing changes never re-run SDXL.
  - `src/procedural.rs`, `src/img.rs`, `src/ktx2w.rs`, `src/svg.rs`: maths generators, seam metrics, a KTX2 writer (RGBA8 UNORM/SRGB, full mips, zstd), resvg rendering.
  - `src/ledger.rs`, `src/sheet.rs`: `assets/assets.json`, `assets/ASSETS.md`, `assets/contact-sheet.html`, thumbnails in `assets/_thumbs/`, and a sweep that deletes files no job owns any more.
  - `data/logos.json`: per-brand source (Commons URL, licence, attribution) or remade wordmark spec, aliases, mono mode and crop overrides.
- `assets/{glass,fluid,scifi,classic,adas,logos,fonts}/`, `assets/_thumbs/`, `assets/assets.json`, `assets/ASSETS.md`, `assets/contact-sheet.html`, `assets/forge.lock.json`.
- `.gitignore`: Blender caches, model weights, `assets/_masters/` (4K ComfyUI masters).
- `docs/PROGRESS.md`: phase table and decision log (every new dependency and the format calls).

## Verification output
```
$ asset-forge doctor            (paths redacted)
✓ repo root: <worktree>
✓ blender: Blender 5.3.0 (hash 02f0899d78e3 built 2026-11-10 01:33:43) (%USERPROFILE%\Documents\Dash_Tools\blender-5.3.0-windows-x64\blender.exe)
✓ comfyui install: %USERPROFILE%\Documents\Dash_Tools\ComfyUI_windows_portable
✓ sdxl base 1.0: ...\models\checkpoints\sd_xl_base_1.0.safetensors (6938078334 bytes)
✓ 4x-UltraSharp: ...\models\upscale_models\4x-UltraSharp.pth (66961958 bytes)
✓ comfyui api: http://127.0.0.1:8188 up; cuda:0 NVIDIA GeForce RTX 3070 Ti Laptop GPU : cudaMallocAsync, 8191 MB VRAM
✓ workflow sdxl_plate.json: <worktree>\tools\asset-forge\workflows\sdxl_plate.json
doctor: all checks passed

$ asset-forge build --force --only <one job per set>
  ✓ glass/sheen (1.3 s)
  ✓ fluid/maths (6.5 s)
  ✓ scifi/masks (1.5 s)
  ✓ classic/bezels (105.2 s)
  ✓ adas/decals (3.5 s)
  ✓ fonts/pack (18.8 s)
  ✓ logos/default (12.0 s)
  ✓ comfy/synth_sun (31.6 s)

$ asset-forge build             (second run, after the merge of main)
126 jobs (fresh), 0 rebuilt; ledger and contact sheet updated

$ git ls-files | grep -cE '\.(py|ps1|sh|js|ts|c)$'
0

$ cargo fmt --all -- --check    -> clean
$ cargo check --workspace       -> Finished (1 warning from main: web-preview manifest, unused dependency fh6-clusters)
$ cargo clippy --workspace --all-targets -- -D warnings -> Finished (same manifest warning only)
$ cargo test --workspace        -> every suite "test result: ok" (asset-forge 12 passed; fh6-clusters 77; golden images ok), 0 failed
$ cargo run -p parity-dump -- tests/golden/packets -> 20/20 OK

$ throwaway test (not committed): fh6_kit::mesh::MeshData::from_glb on assets/adas/*.glb
ego_coupe: 1740 verts, 6444 indices   sport_sedan: 1784 / 6444   suv: 1745 / 6444
supercar: 1702 / 6444                 hypercar: 1745 / 6444      -> 1 passed
```

## Visuals
- `assets/contact-sheet.html`: every asset grouped by set on OLED black, alpha on a checkerboard, with kind, colour space, alpha mode, size, generator, licence and QA. Open it in the built-in browser.
- Thumbnails: `assets/_thumbs/<set>__<job>__<name>.png`.

## Critique (design:design-critique on the glass, fluid and sci-fi sets) and fixes
Framework: first impression on OLED black, usefulness as a cluster layer, hierarchy (does the asset carry one idea), consistency with the tokens (cyan `#38BDF8` glass rim, violet shift, true black), and seams / loops.

| Set / asset | Finding | Severity | Fix |
|---|---|---|---|
| Glass · lens caustic | A flat grey ring: no optical character, read as a stroke, not a lens | 🟡 | Traced per RGB channel with IOR 1.47 / 1.50 / 1.535 and a cool gain bias, so the focus splits into spectral fringes; added a squircle caustic |
| Glass · lens caustic (rrect) | Square tone steps where the straight edge meets the corner arc | 🟢 | Physical (a rounded rectangle's curvature jumps there). Kept, and added `caustic_lens_squircle.png` (continuous curvature, no step) as the clean choice |
| Glass · plates | Normals, refraction and fresnel clean; 16-bit masters smooth on the bevel | ✅ | — |
| Fluid · mercury blob | Black ball with white splotches: the chrome reflected a black lower hemisphere and high-frequency ripples broke the reflection apart | 🔴 | Procedural studio world with a floor and a low bounce card; second noise layer halved (0.08 → 0.035) |
| Fluid · ferrofluid | Spikes appeared only in some frames (4D Voronoi slice) | 🔴 | Static 3D Voronoi spike lattice (like the stable Rosensweig pattern) whose height breathes with looped noise |
| Fluid · ferrofluid | Grid of stubs on the side wall; lattice too regular | 🟡 | Spikes masked to the upward crown (normal z 0.5–0.8 smoothstep); Voronoi randomness 0.25 → 0.45 |
| Fluid · mercury pool | Flat grey (top-down mirror saw only a uniform softbox), then camouflage noise with dense strip lights | 🔴 | Zenith of soft strip lights plus a cross bar (gradient edges, 22% base); ripple amplitude reduced (0.09 → 0.055). Now reads as soft pale silver; still bright (see Known issues) |
| Fluid · matcaps | Lower half pure black | 🟡 | Same studio floor / bounce fix |
| Sci-fi · warp tunnel | First pass: dim scratchy starbursts; second: monochrome steel blue | 🔴 | Streak density 140, tighter smoothstep, emission 10, hue mask made bimodal so streaks read cyan or violet |
| Sci-fi · reticles, masks, holo grid | Crisp, on-token, true black | ✅ | — |
| ComfyUI · tiled plates (`holo_interference`, `caustic_water`) | Hard horizontal seam at mid-height | 🔴 | `make_seamless` rewritten as two separable passes (the 2D pass leaked the shifted copy's wrap row into the side bands) |
| ComfyUI · `chladni_sand` | Zebra stripes with a broken centre: SDXL can't do symmetry | 🟡 | Four-fold mirror of the top-left quadrant (square Chladni plates are symmetric) |
| ComfyUI · plasma, nebula, ferrofluid macro, liquid chrome, accretion disk, energy core, warp streaks, synth sun | Strong, on-family, true black floors | ✅ | — |
| Logos · dial variants | BMW and Cadillac lost detail; McLaren vanished; Alpine and Abarth carried sub-lines; remade wordmarks with wide tracking were clipped | 🔴 | Per-brand mono modes (silhouette / knockout / ink), crops (Alpine: the A; Abarth: the scorpion), wordmark viewBox fitted to the real glyph bounds; Rivian's black wordmark dropped out in knockout → silhouette mode |

## My calls
- **KTX2 RGBA8 UNORM (zstd, full mips) for every data map; 16-bit PNG only for the four glass normal masters.** The kit's PNG path decodes to 8-bit sRGB, which would gamma-bend normals.
- **Flipbook cells 512 px, not 1024.** An 8×8 atlas of 1024 px cells is 256 MB of VRAM per flipbook uncompressed; 512 holds up at cluster scale. Data maps are packed into one KTX2 at 256 px cells.
- **ComfyUI plates commit at 2048; 4K masters stay in gitignored `assets/_masters/`.** Repo budget; the plates sit behind glass and blur.
- **Lens caustics traced in Rust over the Blender plate normals,** replacing Cycles MNEE (near-black after 7 minutes, and not reproducible).
- **Logos:** Commons SVG first (57 brands); Ferrari, Lamborghini and Porsche use their official wordmarks because the emblems are not on Commons; 34 brands are remade wordmarks in OFL faces, outlined to paths; `DEFAULT` is a neutral steering-wheel mark.
- **Classic normals at 1024 and roughness/metal/height at 512** (albedo and lit at 2048): noisy 2K normal KTX2s were 16 MB each.
- **Fonts: 17 families:** the Phase 3 picks (Saira Condensed, Titillium Web, Barlow Semi Condensed, Michroma, Chakra Petch, Exo 2, B612 Mono) plus Saira, Saira Semi Condensed, Barlow Condensed, Orbitron, Rajdhani, Oxanium, Teko, Russo One, Share Tech Mono and Noto Sans (fallback). Upright styles only; italics skipped for size.
- **`oxipng` accepted although it builds C (`libdeflater`):** it is a dev tool, never linked into the exe; logged in the decision log.

## Known issues / debt
- **Mercury pool beauty atlas** is pale and low in contrast (AgX on a bright zenith). Its main runtime use is the packed normal/flow map; tint or contrast-grade the beauty in the shader, or lower the zenith strength in `recipes/fluid.rs` `studio(zenith=True)` and rebuild `--only mercury_pool`.
- **Rounded-rect lens caustic** keeps a tone step at the corner arcs (physical, see Critique). Use the squircle caustic where it shows.
- **Formula Drift mono / dial** keep only the red "DRIFT" badge; the black "FORMULA" drops out in knockout mode. Acceptable at dial size.
- **Bentley's Commons file is tagged CC0**, which looks doubtful; the NOTICE covers it as a trademark either way.
- **Subaru** uses the 2003–2019 Commons logo (the current one is not on Commons).
- **Reference photos** in `assets/reference/` are listed in `ASSETS.md` as reference-only (not shipped).
- **v1 textures** (`assets/*.png`, `assets/textures/4k/`) are still in the repo because `fh6-clusters` (`tesla_adas.rs`) loads `adas_ego_coupe.png`; Phase 7 can delete them once clusters use the new sets.
- No CC0 fill-ins were needed (item 9 of the brief).

## Repo growth
**`assets/` grows from 5.8 MiB to 328.4 MiB (+322.6 MiB, 1,151 new files). That is under the ~400 MB target, so nothing is left out of git except the 4K ComfyUI masters (230 MiB, gitignored in `assets/_masters/`, rebuilt by `build --set comfy`).** The pipeline source (`tools/asset-forge`) is about 330 KB.

| Folder | Size | Files |
|---|---|---|
| `assets/classic/` | 110.0 MiB | 76 |
| `assets/glass/` | 74.9 MiB | 35 |
| `assets/fluid/` | 64.5 MiB | 22 |
| `assets/scifi/` | 35.1 MiB | 16 |
| `assets/_thumbs/` | 13.7 MiB | 542 |
| `assets/logos/` | 12.7 MiB | 366 |
| `assets/fonts/` | 9.7 MiB | 82 |
| `assets/adas/` | 1.0 MiB | 12 |
| ledger, lock, contact sheet | 1.0 MiB | 4 |

The largest single files are `glass/lens_circle_normal.png` (11.3 MB, 16-bit 4K master), `fluid/mercury_pool_normal_flow_8x8.ktx2` (9.1 MB), `scifi/warp_tunnel_8x4.png` (9.1 MB), `glass/plate_squircle_normal.png` (7.9 MB) and `glass/caustic_loop_8x8.png` (6.3 MB). If the repo needs to shrink later, the first candidates are the four 16-bit PNG glass masters (29 MB together; the runtime uses the KTX2 copies) and the classic `*_lit.png` previews (31 MB; the kit lights albedo + normal itself). Both rebuild by script.

## What the next phases must know
### Phase 5 / kit (`fh6-kit`, merged)
- `Images::load_file` decodes PNG as 8-bit **sRGB** and premultiplies. That is right for colour plates and logos, wrong for data: load normals, flow, refraction, fresnel and height from the **KTX2** copies (`R8G8B8A8_UNORM`, stored mips used as-is). The 16-bit PNG normals are masters only; a linear 16-bit PNG path would be needed to use them directly.
- KTX2 data maps are not colour: sample them in your own shader, do not paint them as premultiplied images. Normals are `xy*0.5+0.5` in RG; what B and A hold is stated per file in the ledger's QA column (for example refraction maps: RG = offset, B = thickness, A = coverage).
- **Packed fluid map** `assets/fluid/<name>_normal_flow_8x8.ktx2`: same 8×8 layout as the beauty atlas but 256 px cells; RG = camera-space normal.xy (z = sqrt(1 − x² − y²)), BA = divergence-free flow along the displacement contours, all 0.5-centred; coverage comes from the beauty atlas alpha. Frame `f` is cell `(f % 8, f / 8)` from the top-left; 64 frames at 30 fps loop exactly.
- `additive` alpha in the ledger means RGB is light (caustics, warp, SDXL plates): composite with additive blend, alpha unused.
- Cars: `.glb`, Y up, nose towards +Z, ground at y = 0, materials `paint` / `glass` / `tyre` / `tail` / `head` (tint `paint` with the brand colour); `MeshData::from_glb` loads them.
- Fonts: `assets/fonts/<family>/` + `fonts.json` (roles match the tokens' font roles).

### Phase 7 (cluster factory)
- Pick assets from `assets/assets.json` (`clusters` field per record) or the contact sheet; every record says colour space, alpha, tile and flipbook layout.
- Liquid Glass: plate normal + refraction + fresnel per shape, `caustic_lens_*` additive over the lens, `sheen_arch.png` for the crown, `frost_micro_normal.ktx2` + `blue_noise_128.png` for frost, `caustic_loop_8x8.png` for moving light.
- Fluid / generative: the three flipbooks + packed maps, matcaps for lit liquid-metal shading of SDF shapes, `curl_noise_flow.ktx2` to advect the kit's GPU fluid, Chladni plates for resonance concepts.
- Sci-fi / retro: masks are tileable (scanlines 2/3 px, VFD hex, phosphor triad); reticles come as SVG (vector, any size) and PNG.
- Classic / motorsport: materials tile; use `lit` for flat renders and normal + rmh for the kit's lighting; bezels are radial profiles with AO.
- Logos: resolve a car by `BrandKey` (or `Manufacturer` through `aliases`) in `assets/logos/logos.json`; `dial` is the 1024 px white variant tuned for dark dials (kit tint: mono white at alpha 215).
- **Make more assets:** add a job to `tools/asset-forge/src/sets/<set>.rs` (Blender recipe, ComfyUI plate entry or Rust generator) and run `cargo run --release -p asset-forge -- build --set <set>`. Ledger row, thumbnail and contact-sheet card follow automatically. See `tools/asset-forge/README.md`.
