# 💎 MASTER PHASE 8 SPECIFICATION: ZERO-SLOP SCI-FI KINEMATICS, APPLE OPTICAL GLASS SHADERS & 4K IN-COCKPIT COMPOSITOR

---

## 🔍 FORENSIC AUDIT: THE SHORTCUTS & CHEATS TO PURGE

A comprehensive scan of `rust\crates\fh6-clusters\src\clusters\*.rs` and `fh6-osd\src\render.rs` uncovered the exact lazy patterns that made clusters feel "flat and lifeless":

| File | Line | The Cheat Found in Code | Phase 8 Mandate |
|---|---|---|---|
| `neural_cyber_synapse.rs` | 51 | `draw_text(..., &"7".to_string(), ...)` | **HARDCODED STRING "7" FOR GEAR!** Never changes regardless of telemetry! Replace with dynamic `data.gear_string()` + synaptic pulse ring. |
| `f35_helmet_hmd.rs` | 35 | `let _roll = sanitize(data.roll, ...);` | **IGNORED CAR ROLL!** Discarded roll angle to avoid rotating the pitch ladder. Must apply 2D rotation matrix so horizon tilts with car. |
| `kinetic_liquid_metal.rs` | 76 | `let slip = 0.1; // placeholder` | **HARDCODED SLIP 0.1!** Replace with dynamic `data.rear_tire_slip_angle_rad()`. |
| `glyph_dot_matrix.rs` | 157 | `let torque = hp * 0.5; // placeholder` | **PLACEHOLDER TORQUE!** Replace with real torque calculation `(hp * 5252.0 / rpm)`. |
| `gt7_hood_pro.rs` | 135 | `// Left wing tire slip placeholders` | **PLACEHOLDER TIRE SLIP!** Wire real 4-corner slip angles (`tire_slip_angle_fl..rr`). |
| `cymatic_resonance_field.rs`| 29 | `// Placeholder for computation` | **FAKE CHLADNI WAVE!** Replace with authentic 2D Chladni nodal standing wave equations. |
| `porsche_analog.rs` | 24 | `// 992 GT3 Weissach dial placeholder` | **38-LINE FAKE RECTANGLE!** Zero needle, zero dial ticks, zero carbon wings. |
| `drift_glass.rs` | 16 | 34 lines of flat boxes | **ZERO DRIFT TELEMETRY!** Zero slip arc, zero G-force ball. |
| `solaris_warp_core.rs` | 31 | `fill_rect_xywh` 2x2 dot loops | Flat horizontal rectangles instead of pulsating toroidal plasma compression rings. |
| `chrono_event_horizon.rs` | 32 | Loops of 2x2 pixel dots | Flat dots instead of continuous Kerr black hole relativistic accretion disk spirals. |
| `fh6-osd/src/render.rs` | 400 | `render_dyno_cluster` | **HARDCODED GENERIC SHIFT BAR & SPEEDO BOX!** Ignores user's selected cluster completely! |
| **All 29 Clusters** | 8–10 | `pixmap.fill_rect(bounds, &opaque_bg)` | **SOLID OPAQUE SCREEN FILL!** Blackout curtain over the car interior; destroys OLED `#000000`. |

---

## 🚨 THE 5 IRON ANTI-CHEAT RULES FOR PHASE 8

1. **`BANNED: pixmap.fill_rect(bounds, &opaque_color)`**:
   - Canvas background is strictly pure transparent OLED black `RGBA(0, 0, 0, 0)`.
   - Never draw a full-screen dark rectangle. Unlit pixels must be physically off on OLED.
2. **`BANNED: Any cluster under 120 lines or with hardcoded telemetry/gear`**:
   - Zero `"placeholder"`, zero hardcoded `"7"`, zero ignored `_roll/_pitch`.
   - Every gauge must respond continuously to live RPM, speed, boost, steering, and multi-axis G-forces.
3. **`BANNED: Static OSD Dyno Stage`**:
   - `fh6-osd::render::OsdRenderer::render_dyno_cluster` must dynamically render the active cluster selected in Tab 0 (`config.cockpit_hud_style`) using `fh6_clusters::render_cluster`.
4. **`MANDATORY: 4K Asset Blitting via AssetCache`**:
   - Wire `fh6-render::assets::AssetCache` to decode PNGs once at startup.
   - Textures from `assets/` (`bosch_ddu_bezel.png`, `porsche_tach_dial.png`, `liquid_glass_caustic_lens.png`, `adas_ego_coupe.png`, `cyber_vfd_bezel.png`, `gt7_stadium_glass.png`) must be blitted with bilinear filtering and screen/source-over blending.
5. **`MANDATORY: 4K In-Cockpit Photorealistic Compositor`**:
   - Composite all 29 clusters onto `assets/reference/cockpit_with_wheel.jpg` with OEM Scrim and Bézier-cowl ambient wash.

---

## 🔬 CARD P8-1: withdrawn, replaced by `optics.md`

This card is replaced by [optics.md](optics.md) (2026-11-27). Its glass optics were not physical: an index of 1.52 was called sapphire (sapphire is 1.768), the Fresnel term was quoted and then painted as fixed gradient stops, and the sheen, rim and orb were gradients with no link to a light or a surface. Every glass, reflective and spectral effect now uses the equations in `optics.md`, which the kit implements (`fh6_kit::optics`, `fh6_kit::OpticalGlass`) and tests. The card's stroke colours, gradient stops and sizes are withdrawn with it.

The optics of card P8-2 below are corrected in `optics.md` §10 (prism dispersion, for item 8) and §12 (relativistic Doppler beaming, for item 2, and Chladni figures, for item 6). Where this file and `optics.md` disagree, `optics.md` is right.

---

## ⚡ CARD P8-2: SCI-FI & ABSTRACT CLUSTERS REACTIVE KINEMATICS

### 1. `SolarisWarpCore` — Toroidal Plasma Compression Chamber
- **Pulse Phase**: $\phi = (t_{\text{ms}} \cdot 0.006 \cdot (1.0 + 3.0 \cdot \text{norm\_rpm}) + V_{\text{ms}} \cdot 0.1) \pmod{2\pi}$.
- **Concentric Toroidal Rings**: 8 nested Bézier ellipses whose radii pulse outward at phase $\phi$, with color transitioning from containment amber (`#F59E0B`) to plasma cyan (`#06B6D4`) to redline crimson (`#EF4444`).
- **Dilithium Boost Expansion**: Core chamber expands with boost pressure: $R_{\text{core}} = R_0 \cdot (1.0 + 0.04 \cdot P_{\text{boost}})$.
- **Magnetic Flux Deflection**: Central plasma beam deflects toward corner apex under lateral acceleration: $\Delta x = \frac{G_{\text{lat}}}{2.0} \cdot 45.0 \cdot \text{scale}$.

### 2. `ChronoEventHorizon` — Relativistic Kerr Accretion Disk
- **Gravitational Lensing Einstein Ring**: Warped by lateral G-force and vehicle slip angle:
  $$r(\theta) = R_{\text{einstein}} \cdot \left(1.0 + 0.18 \cdot G_{\text{lat}} \cdot \cos(\theta - \beta) + 0.12 \cdot \sin(2\theta)\right)$$
- **Relativistic Accretion Disk Spiral**: Continuous logarithmic spiral path $r(\theta) = a \cdot e^{b\theta}$ rotating with angular speed $\omega = \text{rpm} \cdot 0.002$.
- **Doppler Relativistic Beaming**: The approaching side (calculated from yaw rate $\omega_y$) is blueshifted (`#38BDF8`) and brightened; the receding side is redshifted (`#EF4444`) and dimmed.
- **Singularity Core**: True black `#000000` Kerr shadow with luminous photon ring flare at redline.

### 3. `NeuralCyberSynapse` — Action Potential EEG Waveform
- **Action Potential Spike**: Continuous cubic Bézier waveform connecting 32 axonal synaptic nodes.
- **Amplitude Modulation**: $A = (12.0 + 35.0 \cdot \text{throttle\_pct}) \cdot \text{scale}$.
- **Spike Propagation Velocity**: Waveform phase advances with vehicle velocity: $\Delta\phi = V_{\text{ms}} \cdot 0.15$.
- **Dynamic Synaptic Gear**: Fix the hardcoded `"7"`! Display live `data.gear_string()` enclosed in a synaptic dendritic node ring.
- **Axon Strobe**: High-RPM synaptic overload triggers branching dendritic lightning arcs across the display.

### 4. `F35HelmetHmd` — Collimated Articulated Horizon & Flight Vector
- **Pitch Ladder with Real Roll Rotation Matrix**: Apply 2D rotational transform by $-\text{roll}$:
  $$\begin{pmatrix} x' \\ y' \end{pmatrix} = \begin{pmatrix} \cos(-\text{roll}) & -\sin(-\text{roll}) \\ \sin(-\text{roll}) & \cos(-\text{roll}) \end{pmatrix} \begin{pmatrix} x \\ y - \text{pitch} \cdot K \end{pmatrix}$$
- **Flight Path Marker (FPM / Velocity Vector "Bird")**: Real-time position driven by vehicle slip angle: $(cx + \beta_{\text{deg}} \cdot 1.8 \cdot \text{scale}, cy)$.
- **Scrolling Airspeed & Altitude Tapes**: Vertical ladder tapes with tick intervals and moving numeric windows.
- **Target Vanishing Boresight**: High-vis collimated green (`#00FF66`) reticle with weapon lock bracket on acceleration.

### 5. `EvaBioSync` — MAGI Supercomputer & AT-Field Octagonal Shockwave
- **Magi Consensus Matrix**: 3 status hexagonal nodes: MELCHIOR (RPM), BALTHASAR (Speed/Gear), CASPER (Tire grip).
- **Harmonic Sync Percentage Waveform**: Sine wave coherence $\Delta\phi$ harmonizes as RPM increases toward redline: $\text{Sync} = 84.0 + 15.0 \cdot \text{norm\_rpm} - 10.0 \cdot \text{slip}$.
- **AT-Field Barrier Pulse**: Under braking ($G_{\text{long}} < -0.2G$), renders expanding concentric orange octagonal field barriers with glowing border strobes.

### 6. `CymaticResonanceField` — Chladni 2D Nodal Standing Wave Patterns
- **Modal Resonant Frequencies**: Harmonic modes $n = (\text{rpm} / 1200.0).clamp(1.0, 6.0)$, $m = (V_{\text{kmh}} / 40.0).clamp(1.0, 6.0)$.
- **Standing Wave Equation**: Nodal points where $\cos\left(\frac{n\pi x}{L}\right) \cos\left(\frac{m\pi y}{L}\right) - \cos\left(\frac{m\pi x}{L}\right) \cos\left(\frac{n\pi y}{L}\right) = 0$.
- Morphs dynamically between 2-lobe, 4-lobe, and 8-lobe standing wave rings as engine passes through harmonic RPM nodes.

### 7. `KineticLiquidMetal` — Hydrodynamic Mercury Meniscus Slosh
- **Fluid Sloshing Angle**: $\theta_{\text{slosh}} = \arctan\left(\frac{G_{\text{lat}}}{1.0 + |G_{\text{long}}|}\right)$.
- **Surface Wave Harmonics**: Viscous fluid meniscus ripples with engine vibration harmonics ($\omega = \text{rpm} \cdot 0.08$).
- **Dynamic Fluid Level**: Liquid volume rises along the gauge arc proportional to RPM, with fluid drop-shadows and metallic specular highlights.

### 8. `ShibuyaChromaticPrisma` — RGB Spectral Aberration
- **Chromatic Dispersion Displacement**: $\Delta_{\text{RGB}} = (\omega_{\text{yaw}} \cdot 6.0 + \beta_{\text{slip}} \cdot 18.0) \cdot \text{scale}$.
- Separate Red, Green, and Blue sub-pixel vector passes offset dynamically during high-speed cornering and drifting.

---

## 🏎️ CARD P8-3: MOTORSPORT REBUILDS (`porsche_analog.rs`, `drift_glass.rs`, `bosch_ddu.rs`)

1. **`PorscheAnalog` (Complete 992 GT3 Weissach Rebuild)**:
   - Blits `assets/porsche_tach_dial.png` (959 KB) centered at $(cx, cy)$.
   - Major ticks (1000 RPM) and minor ticks (250 RPM) from 0 to 9000 RPM.
   - Symmetrical curved Weissach carbon wings with diagonal woven carbon-fiber vector hatches.
   - Mechanical illuminated sweep needle with aluminum center cap and drop-shadow.
   - Left wing digital speed + ghost "888"; right wing gear + oil temp.
2. **`DriftGlass` (Complete GT Drift Rebuild)**:
   - Swept frameless smoked chevron glass plate.
   - Live Counter-Steer Slip Angle Arc: $\beta = \arctan2(V_x, V_z) \cdot \frac{180}{\pi}$.
   - 2D G-Force Friction Circle (Inertia Ball) responding to lateral and longitudinal acceleration ($1.5G$ limit ring).
3. **`BoschDdu`**:
   - Blits `assets/bosch_ddu_bezel.png` (688 KB) with woven carbon fiber weave and steel Allen bolts.
   - 16-LED CARTEK sequential shift lights + dual telemetry side pods.

---

## 📸 CARD P8-4: 4K IN-COCKPIT PHOTOREALISTIC MASTER COMPOSITOR

1. **Asset Source**: `assets/reference/cockpit_with_wheel.jpg` (Forza GT-R cockpit).
2. **Seating Kinematics**:
   - Lowered bucket-seat steering hub anchor: $(X, Y) = (1920, 1860)$, $\text{scale} = 1.96$.
3. **Compositing Pipeline**:
   - Layer 0: 4K in-car cockpit photo (`assets/reference/cockpit_with_wheel.jpg`).
   - Layer 1: Recessed OEM Dash Blackout Scrim (`render_oem_scrim`, 85% opacity over factory dials).
   - Layer 2: `BinnacleAmbientWash` (anti-orb Bézier cowl wash tinted to active cluster signature color).
   - Layer 3: The 4K textured cluster gauge with transparent OLED background.
   - Layer 4: Visor lip shading / steering hub rim shadow.
4. **Master Poster Generation**:
   - Output `02_Final_4K_Mockups/preview_cockpit_all_29_clusters_night_glow.png` (5×6 4K contact sheet of all 29 clusters living in the car).
   - Update `index.html` with a live toggle: `[ In-Cockpit Mode ]` vs `[ OLED Cutout Mode ]`.
