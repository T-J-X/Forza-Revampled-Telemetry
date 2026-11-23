# TeslaAdas assets

These files are built by `cargo run --release -p asset-forge -- build --set tesla_adas` from `tools/asset-forge/src/recipes/tesla_adas.rs`. Don't edit them by hand: change the recipe and rebuild.

The ledger rows are also in `assets/ASSETS.md`. The cluster has a procedural fallback for each file, used when `setup.assets` is `None` or the file is missing.

Blender's camera is right-handed and the kit's isn't, so every Blender render through a chase camera is mirrored left-right in post. Round 5 fixed this for the pool atlases.

| File | Tool | Recipe and seed | Size | Used for | Fallback |
|---|---|---|---|---|---|
| `ego_cockpit.png` | Blender 5.3 (Cycles 128 spp, OptiX denoise, AgX −0.6 EV) | `tesla_adas::CARS`, seed 152. The Phase 6 `adas/ego_coupe.glb` with car-studio materials, lit and staged as described under Rig below. It is rendered through the cockpit chase camera (eye (0, 3.4, −6), 8° pitch) at yaw −50..50 in steps of 10. Cells are 540 × 360 at 100 px/m, and the ground centre sits at (0.5, 0.66) of each cell. | 2160 × 1080 (4 × 3 cells) | The ego car in the cockpit view. Yaw crossfades between neighbouring cells; roll is a screen rotation and dive a lift plus squash. It is tinted 12% toward the brand primary. | The glTF clay mesh (kit mesh pass) |
| `ego_hood.png` | Blender 5.3 | The same, through the hood camera (eye (0, 2.8, −6.5)). | 2160 × 1080 | The ego car in the hood view | The clay mesh |
| `ego_lamps_cockpit.png`, `ego_lamps_hood.png` | Blender 5.3 | The same shots at half size. Only the tail-lamp material is emissive; everything else is held out. Luminance becomes alpha. | 1080 × 540 | The brake lights: added in warning red × lamp level over the ego sprite | The mesh lamp quad |
| `traffic_sedan.png`, `traffic_suv.png`, `traffic_supercar.png` | Blender 5.3 | `tesla_adas::CARS`, seed 152, with each Phase 6 traffic glb in a 0.42 grey metallic paint. Rows are view elevations 2.5 / 5 / 10 / 20°; columns are headings relative to the view ray, 0..40° in steps of 10 (mirrored live for negative angles). Cells are 336 × 232 at 64 px/m, anchored at (0.5, 0.70). | 1680 × 928 | The traffic cars, picked by each car's live elevation and heading. Shadows are baked from the shadow catcher. | Clay meshes and 2D contact-shadow ellipses |
| `led_caps.png` | Blender 5.3 (Cycles 128 spp) | `tesla_adas::CAPS`, seed 153. A pill lens 6.0 × 2.2 × 0.7 with a bevelled rim, shot through an orthographic front camera. Off: clear-coated smoked acrylic. On: a back-lit white diffuser, hottest at the centre. Lit by a softbox above and in front, a cool strip below, and studio.exr. | 576 × 128 (2 cells of 288 × 128, the lens 240 × 88) | The 10 shift-light caps. Lit caps are tinted with the pod colour over the pod glow, and bloomed. | The shared vector pods (`ShiftLights::draw_at`) |
| `underglow_pool_cockpit.png` | Blender 5.3 (Cycles 192 spp, OptiX denoise) | `tesla_adas::POOL`, seed 151. The ego glb on a white ground, invisible to the camera but still blocking light, with a downward area light 1.9 × 4.0 m at 0.16 m and 520 W. Rendered through the cockpit chase camera at yaw −50..50 in steps of 10. Post: luminance becomes alpha with a 4% black point, softened 10×, gamma 0.7, feathered, and mirrored. | 2880 × 1080 (4 × 3 cells of 720 × 360) | The ego's underglow, tinted live: primary, amber in a drift, violet at the limiter. Neighbouring cells are added. | A Gaussian round a ground ellipse |
| `underglow_pool_hood.png` | Blender 5.3 | The same, through the hood camera. | 2880 × 1080 | The hood view's underglow | The same Gaussian |
| `asphalt.png` | ComfyUI (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras, 32 steps, cfg 6.5, 1536 × 640) | Seed 1507, prompt `ASPHALT_PROMPT` in the recipe. Post: the lower half of the upscale; its line-free left 46% is mirrored into a symmetric plate at 1536 × 512, and luminance becomes alpha with a far-to-near fade and side feathers. | 1536 × 512 | The road surface between the outer edges, at 6% ink, so the aggregate grain reads. | A faint gradient |

Rig for every `CARS` shot:
- **Materials:** the glb is subdivided once and smoothed. It gets clear-coated metallic paint, dark glass, rubber and smoked tail lenses.
- **Lighting:** an overhead softbox, two long strip lights, a cool rim from ahead, and the studio.exr HDRI.
- **Shadow:** a shadow catcher under the car.

The atmosphere at the horizon is procedural (a radial gradient), not an asset. The round-3 ComfyUI haze plate was dropped because it read as a separator bar.

Licences:
- The Blender renders are MIT (asset-forge).
- The SDXL output is under the CreativeML Open RAIL++-M licence; the asset-forge code that made it is MIT.
