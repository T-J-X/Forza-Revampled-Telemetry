# Pre-built 4K assets

TJX, 2026-11-28: "look at the whole catalog, set a task to work on all assets that will be needed in the cloud for blender and comfyui models and then penpot when it has a chance ... needs to be high 4k quality". Thread "Pre-build 4K assets for the catalog" went through the 64 briefs whose dash had not started (all but LiquidGlass, PorscheAnalog and the six dashes with running threads) and listed what each needs.

## What can be made before a dash starts

A dash draws its Penpot board before any code (CLUSTER_PROMPT.md), so only what does not depend on the layout is pre-built:

- **ComfyUI textures:** seamless 4096² masters of the organic surfaces a brief names. SDXL never makes metals, twill, guilloché or anything exact.
- **Blender swatches:** 4096² tiles (seamless) or discs (radial patterns) of the procedural materials a brief names, with normal and rough/metal/height passes.
- **Blender sprites:** hardware whose size the brief states (rings, caps, LED and lamp lenses, crystals), at the rest key and the two G-swing keys, 2 px per 4K px for the master and box-filtered to 1:1 in git.
- **Computed maps (CPU, exact):** for the liquid-metal dashes (`OilSlickChrome`, `FaradayMandala`, added 08:20 when TJX split KineticLiquidMetal's three directions into separate dashes): the liquid-metal studio as an octahedral environment (4096² master) and a 256×1 env band, matcaps of chrome and liquid mercury, Fresnel tables, and thin-film tables of an n = 1.47 oil film on each metal by thickness and angle. They come from measured optical constants (Cr: Johnson & Christy 1974; liquid Hg: Inagaki et al. 1981; refractiveindex.info, CC0), the exact complex Fresnel and Airy sums of `docs/specs/optics.md` §3 and §9, and the analytic studio, so Cycles or SDXL would only add noise (`tools/asset-forge/src/prebuild/liquid.rs`, tested against the optics.md values). No GPU is needed.

Not pre-built: housings, yokes and binnacles (their shape is the layout), Psychedelic and Abstract figures (the dash's maths draws them; noise never draws the figure), clear optical glass (computed by the kit, `docs/specs/optics.md`), and the AMG MBUX parts, which wait for each dash's step-1 frame grabs (no housing slab; the display floats on black).

## How a dash thread uses it

- Run: `gpu-lock.exe run --lane cloud --thread "<thread>" --job prebuild -- cargo run --release -p asset-forge -- --lane cloud prebuild <snake>` (code in `tools/asset-forge/src/prebuild/`). `prebuild --plan` prints the list below from the code; `--list` gives counts.
- Masters: `G:\My Drive\FH6 WheelDash v2 - 4K Masters\<snake>\prebuild\` (`textures`, `swatches`, `sprites`, the contact sheet and the manifest).
- Repo: `assets/clusters/<snake>/prebuild/manifest.json` and `README.md` list every file with its generator, seed, prompt, licence and use; `sprites/` holds the 1:1 runtime sprites and `computed/` the runtime copies of the computed maps (env 2048² and matcaps 1024² in sRGB 8-bit, look-up tables 16-bit linear). Textures and swatches have no runtime copy: the dash's own recipe downsizes the master to the size its layout needs.
- A ring width marked "proposed" is a guess where the brief gives only the diameter: step 1 confirms it, and the thread changes the number in `kits.rs` and re-runs if it differs.

## Dashes with nothing to pre-build

VisionSpatialGlass, DriftGlass, VernierLoupe, GlyphDotMatrix, WindshieldHud, F35HelmetHmd, ArNavWayfinder, HarmonographDrive, DichroicLouvre, GeodesicWell, Lightpainter, AeroPlasmaBlade, RackFocus, LumenEtch, TreadSeismograph and FibreOpticCorona are procedural only by their briefs. ChronoMultiverse, AuroraVeil, IsochromaticLoadPath, AeroSmokeTunnel and AbyssalShearBloom draw their figure from the model. CarplayIsland, HopfFibration, VisionPetals, LiquidMorph, KaleidoscopeMirror and GoldenAngle reuse the existing `liquid_glass` studio plate (seed 77203). AmgMbuxSupersport, AmgMbuxTrackPace and AmgMbuxClassic wait for their frame grabs. Gt7CockpitPro shares Gt7HoodPro's parts, and Gt7ModularTripod (translucent panels, thin lines) has no hardware. SynthwaveOutrun's chrome bevel follows its font atlas. VisionPetals' petal lens, LiquidMorph's drops and SapphireMystery's cabochon are optical glass, which the kit computes from the outline. CymaticResonanceField's rim, SolarisWarpCore's coil hoops and ArcLadderCascade's insulators are optional profile strips, and the brief's analytic bevel is the default.

## The kits

### #9 `ValkyrieLmp1` (`valkyrie_lmp1`)

- Texture `forged_carbon_marble` (ComfyUI, seed 90002): the yoke's forged-carbon marble (brief: albedo + normal, seamless): base colour, and a normal from its luminance
- Left to the thread: the 1440x420 forged-carbon yoke with angled shoulders: shape needs the Penpot board
- Left to the thread: the knurled anodised encoder rings: the knurl only reads on a lathe ring, and the brief gives no ring diameter (render them as bezel sprites with a knurled outer chamfer once the layout sets the size)

### #12 `SkylineR34Mfd` (`skyline_r34_mfd`)

- Texture `lcd_backlight_mura` (ComfyUI, seed 120001): modulation layer for the gear and speed LCD windows and the MFD tiles; the pixel grid stays procedural
- Sprite `tach_ring_440` (Blender, 456 px): the 440 px tach ring: matte period black plastic with a polished inner chamfer (14 px width proposed; step 1 checks the R34 ring)
- Left to the thread: the MFD tile housings and the dial-face print: layout and text

### #16 `AmgOneF1` (`amg_one_f1`)

- Texture `antiglare_matte_grain` (ComfyUI, seed 160001): labelled micro-grain over the screen face (luminance modulation of 4% at most); the pixel grid stays procedural
- Swatch `carbon_twill_rim` (Blender tile): surface of the 780 px carbon rim arc (2x2 twill, 12 px tows)
- Left to the thread: the housing, the rim arc and the F1 shift-light caps: need the Penpot layout and the cap pitch

### #19 `ChronoEventHorizon` (`chrono_event_horizon`)

- Texture `accretion_shear_filaments` (ComfyUI, seed 190001): greyscale shear texture wrapped polar onto the 1100x190 disk and tinted by the dash; the lensing itself is the dash's deflection table, never this texture

### #20 `Gt3RsApogee` (`gt3_rs_apogee`)

- Texture `mli_foil_crinkle_height` (ComfyUI, seed 200001): height only: the dash's gold-foil dial backing shades it with metal GOLD (SDXL never draws the metal)
- Texture `heat_shield_tile_coating` (ComfyUI, seed 200004): surface of the side parts; the tile grid itself is geometry (seed 200004: 200002 drew a 2x2 tile grid into the texture)
- Texture `star_field_tile` (ComfyUI, seed 200003): environment for reflections in the titanium and gold (a tile, not a figure)
- Sprite `rev_counter_ring_500` (Blender, 516 px): the 500 px machined titanium rev-counter ring, no hub (22 px width proposed)
- Sprite `docking_light_off` (Blender, 38 px): the 12 docking lights on the 760x30 strip: 30 px caps, unlit and lit
- Sprite `docking_light_on` (Blender, 38 px): the 12 docking lights on the 760x30 strip: 30 px caps, unlit and lit
- Left to the thread: the attitude ball and its gimbal cage: markings settle in step 1 against the reference ball

### #25 `BugattiTourbillon` (`bugatti_tourbillon`)

- Swatch `brushed_steel_linear` (Blender tile): the bridges' base albedo and roughness (20% luminance at most in the kit)
- Sprite `jewel_lens_30` (Blender, 42 px): the 11 jewels on the 30x740 strip and the jewel bearings: clear sapphire in a polished steel setting, tinted at run time
- Left to the thread: Côtes de Genève stripes: the hero library has no stripe pattern yet (add a guilloché 'stripes' mode first)
- Left to the thread: the 210 px tourbillon cage glTF: part of the dash's 15k-triangle budget and its motion

### #26 `ShibuyaChromaticPrisma` (`shibuya_chromatic_prisma`)

- Texture `wet_asphalt_ripple` (ComfyUI, seed 260001): greyscale height for the mirror-ripple normal map

### #30 `GtrNismoMfd` (`gtr_nismo_mfd`)

- Swatch `carbon_twill_surround` (Blender tile): the meter surround (2x2 twill)
- Sprite `tach_ring_460` (Blender, 476 px): the 460 px chronograph-style tach ring: machined alloy top, polished chamfers (20 px width proposed; step 1 checks the R35 ring)

### #31 `SimInputTrace` (`sim_input_trace`)

- Texture `antiglare_matte_grain` (ComfyUI, seed 160001): labelled micro-grain over the screen face (luminance modulation of 4% at most); the pixel grid stays procedural
- Swatch `dark_anodised_carrier` (Blender tile): the LED carrier and the trace window's frame edge
- Sprite `shift_led_32_off` (Blender, 40 px): 16 LEDs on 780x32: 32 px caps, tinted go / warning / shift by the kit
- Sprite `shift_led_32_on` (Blender, 40 px): 16 LEDs on 780x32: 32 px caps, tinted go / warning / shift by the kit

### #33 `LorenzButterfly` (`lorenz_butterfly`)

- Texture `pad_rim_powder_coat` (ComfyUI, seed 330001): labelled grain on the rims of the true-black pads (the gear pad and the speed pad); the attractor is the dash's RK4

### #34 `PaganiHorologerie` (`pagani_horologerie`)

- Swatch `clous_de_paris_p12` (Blender tile): the dial faces, raked by the G light (14 alpha at most)
- Swatch `rose_guilloche_p24` (Blender disc): the secondary guilloché zone (the brief's barleycorn, as the library's rose pattern)
- Swatch `black_anodised_alu` (Blender tile): the tonneau case chamfer
- Sprite `polished_steel_matcap` (Blender, 1012 px): a 1000 px polished steel hemisphere under the hero rig: the matcap for the hands and screws

### #37 `NeuralCyberSynapse` (`neural_cyber_synapse`)

- Texture `soma_membrane_grain` (ComfyUI, seed 370001): optional modulation inside the 260 px soma (a gradient is the fallback); the nerve front is the dash's Nagumo model

### #42 `SuminagashiMarble` (`suminagashi_marble`)

- Texture `urushi_lacquer` (ComfyUI, seed 420001): the tray rim's lacquer
- Texture `still_water_sheen` (ComfyUI, seed 420002): greyscale height for the water's faint sheen normal; the marbling is the dash's area-preserving maps
- Texture `washi_paper_fibre` (ComfyUI, seed 420003): labelled fibre texture under the labels
- Left to the thread: the gear stone (200 px) and the side stones (260x130): pebble geometry is modelled with the layout

### #43 `T50Floodlit` (`t50_floodlit`)

- Texture `leather_binnacle_black` (ComfyUI, seed 430001): the binnacle surround (leather(...).set("texture")) for edges and reflections
- Swatch `spun_aluminium_chapter` (Blender disc): the chapter ring's fine circular brushing
- Swatch `carbon_twill_pane_frame` (Blender tile): frames of the two 300x220 panes
- Sprite `dial_ring_480` (Blender, 496 px): the 480 px thin polished bezel over a satin spun chapter blank, no numerals (the scale follows the car's max rpm)
- Sprite `centre_cap_200` (Blender, 212 px): the 200 px machined cap over the hub

### #47 `LfaV10Nurburgring` (`lfa_v10_nurburgring`)

- Swatch `spun_titanium` (Blender disc): the ring's circular grain and its tangential anisotropy
- Sprite `titanium_ring_480` (Blender, 496 px): the floating 480 px titanium ring, 24 px wide (r_in 216), no face or pivot

### #48 `HakosukaS20` (`hakosuka_s20`)

- Texture `crackle_black_paint` (ComfyUI, seed 480001): bare black panel finish (the same seed in HakosukaS20 and ZNismoSport, so both get the same texture)
- Texture `worn_steering_wood` (ComfyUI, seed 480002): wood in the environment the bright rings reflect
- Sprite `tube_ring_420` (Blender, 436 px): the two big dials: a thin bright chrome ring on a deep matte tube (8 px width proposed)
- Sprite `small_dial_ring_150` (Blender, 166 px): the three 150 px small dials (5 px width proposed); render straight, the 12° turn is in the kit
- Sprite `jewel_lamp_28_off` (Blender, 36 px): the 10 lamps on 740x28: 28 px jewel lenses
- Sprite `jewel_lamp_28_on` (Blender, 36 px): the 10 lamps on 740x28: 28 px jewel lenses

### #49 `SimPitWall` (`sim_pit_wall`)

- Texture `antiglare_matte_grain` (ComfyUI, seed 160001): labelled micro-grain over the screen face (luminance modulation of 4% at most); the pixel grid stays procedural
- Swatch `carbon_twill_carrier` (Blender tile): the LED carrier surface
- Sprite `shift_led_32_off` (Blender, 40 px): 14 LEDs on 760x32: 32 px caps (the same design as SimInputTrace's)
- Sprite `shift_led_32_on` (Blender, 40 px): 14 LEDs on 760x32: 32 px caps (the same design as SimInputTrace's)

### #51 `KaleidoscopeMirror` (`kaleidoscope_mirror`)

- Texture `chip_glass_inclusions` (ComfyUI, seed 510001): labelled grain map inside the eight chips; the mirror copies are the dash's dihedral group
- Sprite `eyepiece_ring_380` (Blender, 396 px): the knurled black-chrome eyepiece ring round the 380 px figure (10 px width and 4 px knurl pitch proposed)

### #52 `KoenigseggGyro` (`koenigsegg_gyro`)

- Swatch `carbon_twill_12` (Blender tile): the rotating ring band
- Swatch `brushed_titanium` (Blender tile): the inner rim
- Sprite `instrument_ring_540` (Blender, 556 px): the 540 px instrument ring: carbon band, brushed titanium rim, polished chamfers (30 px width proposed); the kit turns it with steering

### #53 `ApertureNine` (`aperture_nine`)

- Swatch `brushed_black_titanium_blade` (Blender tile): the iris blades' anisotropic lobe (18% luminance at most); the iris geometry is the dash's nine-disc maths

### #56 `RevueltoCorsa` (`revuelto_corsa`)

- Texture `forged_composite_large_chips` (ComfyUI, seed 560001): the hexagon pod body
- Left to the thread: the hex-mesh grille: the hero library has no hexagon perforation yet
- Left to the thread: the 300 px hexagon pod: the library has no hexagon outline yet, and the anodise tint is not in the brief

### #57 `ZNismoSport` (`z_nismo_sport`)

- Texture `crackle_black_paint` (ComfyUI, seed 480001): bare black panel finish (the same seed in HakosukaS20 and ZNismoSport, so both get the same texture)
- Texture `dash_top_leatherette` (ComfyUI, seed 570002): the dash top the pods stand on (stitching is geometry)
- Sprite `hooded_pod_170` (Blender, 186 px): the 170 px hooded pods: matte crackle-black tube with a polished inner ring (12 px width proposed); render straight, the turn is in the kit
- Sprite `pod_crystal_170` (Blender, 180 px): the 170 px pod crystal (reflections only, drawn over the needle)

### #60 `GoldenAngle` (`golden_angle`)

- Texture `bead_inner_sparkle` (ComfyUI, seed 600001): labelled sparkle grain inside the bead; the seed spiral is the dash's Vogel model
- Sprite `seed_bead_26` (Blender, 38 px): the 26 px domed crystal bead; one sprite serves every seed

### #61 `RattrapanteSector` (`rattrapante_sector`)

- Swatch `spun_titanium_arc_rail` (Blender disc): the arc rail's tangential grain

### #64 `EvaBioSync` (`eva_bio_sync`)

- Texture `cell_nucleus_grain` (ComfyUI, seed 640001): optional modulation of the lit-cell gradient

### #65 `MclarenSlimDisplay` (`mclaren_slim_display`)

- Texture `antiglare_matte_grain` (ComfyUI, seed 160001): labelled micro-grain over the screen face (luminance modulation of 4% at most); the pixel grid stays procedural
- Texture `alcantara_black` (ComfyUI, seed 650002): the dash top the slab folds into (alcantara(...).set("texture"))

### #68 `BionicSpaceframe` (`bionic_spaceframe`)

- Swatch `brushed_titanium_strut` (Blender tile): the struts' surface (base 12% luminance at most); the truss is the dash's Michell geometry

### #70 `StarSailOrigami` (`star_sail_origami`)

- Swatch `brushed_film_hairline` (Blender tile): optional facet sheen (use its normal); the fold is the dash's Miura-ori maths

### #72 `HyperbolicTiles` (`hyperbolic_tiles`)

- Texture `etched_glass_grain` (ComfyUI, seed 720001): labelled sparkle map on the two disc faces; the tiling is the dash's Poincaré maths
- Sprite `disc_rim_ring_400` (Blender, 416 px): the 400 px thin machined rim (8 px width proposed); one render, mirrored for both discs

### #73 `OilSlickChrome` (`oil_slick_chrome`)

- Computed `liquid_metal_studio_env` (CPU, exact): what the metal mirrors: the octahedral environment for the kit's reflection lookup (gain 16); turn it about the view axis with the G-light
- Computed `liquid_metal_env_band` (CPU, exact): the 256x1 HDR env band for 2D SDF metal shading (the brief's env-band LUT): index by the mirror direction's angle in the key plane
- Computed `chrome_matcap` (CPU, exact): the chrome ribbon and any bare chrome: one lookup by view-space normal, Fresnel and studio included
- Computed `mercury_matcap` (CPU, exact): the liquid-metal blob where the film has drained to nothing
- Computed `chrome_fresnel` (CPU, exact): bare chrome reflectance over cos theta, for shading with the environment map instead of the matcap
- Computed `mercury_fresnel` (CPU, exact): bare liquid-metal reflectance over cos theta
- Computed `oil_on_mercury_film` (CPU, exact): the skin's colour: reflectance of an n = 1.47 oil film on liquid metal by thickness (Jeffreys drainage, 900 to 100 nm) and cos theta; multiply the mirrored studio by it
- Computed `oil_on_chrome_film` (CPU, exact): the same film on solid chrome, for the ribbon where oil runs onto it
- Left to the thread: the metaball skin and its thickness field: computed live from the brief's equations (Jeffreys h = sqrt(mu s / (rho g t)), skin at sum q^2 / |x - c|^2 = 1), never a texture
- Left to the thread: a polished chrome swatch: a smooth mirror has no surface texture to bake, so chrome_matcap and chrome_fresnel replace it

### #74 `FaradayMandala` (`faraday_mandala`)

- Computed `liquid_metal_studio_env` (CPU, exact): what the metal mirrors: the octahedral environment for the kit's reflection lookup (gain 16); turn it about the view axis with the G-light
- Computed `liquid_metal_env_band` (CPU, exact): the 256x1 HDR env band for 2D SDF metal shading (the brief's env-band LUT): index by the mirror direction's angle in the key plane
- Computed `mercury_matcap` (CPU, exact): the liquid-metal disc: shade the wave field's normal with one lookup
- Computed `mercury_fresnel` (CPU, exact): liquid-metal reflectance over cos theta, for shading with the environment map
- Computed `chrome_matcap` (CPU, exact): the rim ring and the gear's still centre node if they are solid chrome
- Left to the thread: the Faraday wave field and its normals: the brief's exact zeta = A sqrt(eps) sum cos(k e_i x + phi_i) is cheaper live than any flipbook, and its amplitude follows the revs
- Left to the thread: the rpm rim ring's hardware: its diameter comes from the layout

