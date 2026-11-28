//! The pre-build kits, one per dash not yet started, in CATALOG.md build order (2026-11-28).
//!
//! Drawn up from each brief's "Phase 6 assets" line and its hardware description. Rules held to:
//! SDXL only for organic surfaces (metals, twill, guilloché and anything exact are Blender);
//! nothing that depends on the layout (it waits for the dash's Penpot board); no Psychedelic or
//! Abstract figure (the dash's maths draws it); clear optical glass is computed in the kit
//! (`docs/specs/optics.md`), so lens normal and thickness bakes are not pre-built. Where a brief
//! gives a ring's diameter but not its width, the width here is a proposal, said so in `use_`;
//! the dash thread re-renders with its own number if step 1 finds another. Colours are linear.
//!
//! Dashes with nothing layout-independent to make have no kit: VisionSpatialGlass,
//! AmgMbuxSupersport, DriftGlass, Gt7CockpitPro, ChronoMultiverse, BugattiTourbillon's cage,
//! WindshieldHud, F35HelmetHmd, AmgMbuxTrackPace, AuroraVeil, ArNavWayfinder, AmgMbuxClassic,
//! SynthwaveOutrun, Gt7ModularTripod, HarmonographDrive, DichroicLouvre, GeodesicWell,
//! IsochromaticLoadPath, Lightpainter, AeroPlasmaBlade, RackFocus, AeroSmokeTunnel, LumenEtch,
//! TreadSeismograph, AbyssalShearBloom, HopfFibration, FibreOpticCorona, CarplayIsland,
//! VernierLoupe (procedural only, or waiting for step-1 frame grabs on the AMG dashes).
use super::liquid::Metal;
use super::{Calc, Computed, Kit, Sprite, Swatch, Tex};
use crate::hero::geom::BezelSpec;
use crate::hero::mat::{self, metal, Mat};
use crate::hero::part::Part;
use crate::hero::texture::Recipe;

/// Dashes with a kit, in build order.
pub const ORDER: &[&str] = &[
    "kinetic_liquid_metal",
    "valkyrie_lmp1",
    "skyline_r34_mfd",
    "amg_one_f1",
    "chrono_event_horizon",
    "gt3_rs_apogee",
    "bugatti_tourbillon",
    "shibuya_chromatic_prisma",
    "gtr_nismo_mfd",
    "sim_input_trace",
    "lorenz_butterfly",
    "pagani_horologerie",
    "neural_cyber_synapse",
    "suminagashi_marble",
    "t50_floodlit",
    "lfa_v10_nurburgring",
    "hakosuka_s20",
    "sim_pit_wall",
    "kaleidoscope_mirror",
    "koenigsegg_gyro",
    "aperture_nine",
    "revuelto_corsa",
    "z_nismo_sport",
    "golden_angle",
    "rattrapante_sector",
    "eva_bio_sync",
    "mclaren_slim_display",
    "bionic_spaceframe",
    "star_sail_origami",
    "hyperbolic_tiles",
    "faraday_mandala",
];

/// A computed map.
const fn calc(name: &'static str, calc: Calc, use_: &'static str) -> Computed {
    Computed { name, calc, use_ }
}

/// The liquid-metal studio, shared by the liquid-metal dashes.
fn studio_pair() -> [Computed; 2] {
    [
        calc(
            "liquid_metal_studio_env",
            Calc::StudioEnv,
            "what the metal mirrors: the octahedral environment for the kit's reflection lookup (gain 16); turn it about the view axis with the G-light",
        ),
        calc(
            "liquid_metal_env_band",
            Calc::EnvBand,
            "the 256x1 HDR env band for 2D SDF metal shading (the brief's env-band LUT): index by the mirror direction's angle in the key plane",
        ),
    ]
}

const fn tex(name: &'static str, prompt: &'static str, negative: &'static str, seed: u64, use_: &'static str) -> Tex {
    Tex {
        recipe: Recipe {
            name,
            prompt,
            negative,
            seed,
        },
        use_,
    }
}

fn tile(name: &'static str, mat: Mat, use_: &'static str) -> Swatch {
    Swatch {
        name,
        mat: mat.named(name),
        disc: false,
        use_,
    }
}

fn disc(name: &'static str, mat: Mat, use_: &'static str) -> Swatch {
    Swatch {
        name,
        mat: mat.named(name),
        disc: true,
        use_,
    }
}

/// A machined ring of outer diameter `d` and radial `width`, with an optional dark well inside.
fn ring(name: &'static str, d: f32, width: f32, mats: [Mat; 4], well: Option<Mat>, use_: &'static str) -> Sprite {
    let r_out = d * 0.5;
    let r_in = r_out - width;
    let mut parts = Vec::new();
    if let Some(w) = well {
        parts.push(Part::face("well", [0.0, 0.0], 0.0, r_in + 1.0, w));
    }
    parts.push(Part::bezel("ring", [0.0, 0.0], 0.0, BezelSpec::ring(r_in, width), mats));
    Sprite {
        name,
        size: (d + 16.0).ceil(),
        parts,
        swing: true,
        use_,
    }
}

/// Four slots of one metal: polished chamfer, `top` finish, dark walls, polished outer chamfer.
fn metal_set(base: [f32; 3], top: Mat, tag: &str) -> [Mat; 4] {
    [
        mat::polished(base).named(&format!("{tag}_chamfer")),
        top.named(&format!("{tag}_top")),
        mat::brushed(metal::GUNMETAL).named(&format!("{tag}_wall")),
        mat::polished(base).named(&format!("{tag}_outer")),
    ]
}

/// An LED or lamp cap of diameter `d` in a piano-black collar: `_off` (smoked, unlit) and `_on`
/// (white diffuser lit at full strength; the kit tints it).
fn led_pair(name_off: &'static str, name_on: &'static str, d: f32, use_: &'static str) -> [Sprite; 2] {
    let r = d * 0.5 - 2.0;
    lamp_pair(name_off, name_on, d, use_, &|glow: f32, n: &str| {
        Part::dome(
            "cap",
            [0.0, 0.0],
            0.0,
            r,
            r * 0.42,
            0.0,
            [
                mat::led_cap([1.0, 1.0, 1.0], glow).set("radius", r).named(n),
                mat::piano_black(),
            ],
        )
    })
}

/// A faceted jewel lamp of diameter `d` (the period warning lamps): a pale crystal jewel the kit
/// tints, `_off` and `_on`, in the same black-chrome collar as [`led_pair`].
fn jewel_pair(name_off: &'static str, name_on: &'static str, d: f32, use_: &'static str) -> [Sprite; 2] {
    let r = d * 0.5 - 2.0;
    lamp_pair(name_off, name_on, d, use_, &|glow: f32, n: &str| {
        Part::jewel(
            "jewel",
            [0.0, 0.0],
            0.0,
            r,
            r * 0.38,
            r * 0.12,
            8,
            [
                mat::jewel([0.62, 0.6, 0.56], glow, r).named(n),
                mat::polished(metal::BLACK_CHROME).named("jewel_girdle"),
            ],
        )
    })
}

/// The off and lit sprites of one lamp in a black-chrome collar; `cap(glow, material_name)`
/// builds the lens.
fn lamp_pair(
    name_off: &'static str,
    name_on: &'static str,
    d: f32,
    use_: &'static str,
    cap: &dyn Fn(f32, &str) -> Part,
) -> [Sprite; 2] {
    let r = d * 0.5 - 2.0;
    let bezel = || {
        Part::ring(
            "collar",
            [0.0, 0.0],
            0.0,
            r,
            r + 2.0,
            2.0,
            mat::polished(metal::BLACK_CHROME).named("led_collar"),
        )
    };
    [
        Sprite {
            name: name_off,
            size: d + 8.0,
            parts: vec![bezel(), cap(0.0, "led_off")],
            swing: true,
            use_,
        },
        Sprite {
            name: name_on,
            size: d + 8.0,
            parts: vec![bezel(), cap(LED_ON_GLOW, "led_on")],
            swing: false,
            use_,
        },
    ]
}

/// Emission strength of a lit LED or lamp cap at the rim; the centre runs 2.2 times hotter
/// (`glow_strength` in the bpy builder). The cap is an opaque diffuser now, so the glow covers
/// its face (batch 2: a transmissive lens rendered transparent and only its rim showed).
const LED_ON_GLOW: f32 = 3.0;

/// ValkyrieLmp1's shoulder encoder (the dash thread's spec, 2026-11-28): a diamond-knurled ring
/// from r59 to r73 round a turned cap of r53, in a piano-black well of r92, all in dark red
/// anodise (sRGB 58, 30, 33). One sprite; the dash mirrors it for the two rings at (±604, 4).
fn valkyrie_encoder() -> Sprite {
    // sRGB 58, 30, 33 in linear light.
    let red = [0.0423, 0.0130, 0.0152];
    let anod = |m: Mat, n: &str| m.set("metallic", 0.5).set("coat", 1.0).set("coat_rough", 0.18).named(n);
    Sprite {
        name: "encoder_ring_knurled_184",
        size: 200.0,
        parts: vec![
            Part::face("well", [0.0, 0.0], 0.0, 92.0, mat::piano_black().named("encoder_well")),
            Part::ring(
                "knurl",
                [0.0, 0.0],
                0.0,
                59.0,
                73.0,
                7.0,
                anod(mat::knurled(red, 3.0, false), "encoder_knurl"),
            ),
            Part::dome(
                "cap",
                [0.0, 0.0],
                0.0,
                53.0,
                6.0,
                2.0,
                [anod(mat::spun(red), "encoder_cap"), anod(mat::polished(red), "encoder_cap_edge")],
            ),
        ],
        swing: true,
        use_: "the two shoulder encoders at (±604, 4) in 4K px (0.7 scale in the hood view): one sprite, mirrored; a 3 px diamond knurl on the ring's face",
    }
}

/// A domed cap of diameter `d` and height `h`.
fn cap(name: &'static str, d: f32, h: f32, mats: [Mat; 2], use_: &'static str) -> Sprite {
    Sprite {
        name,
        size: d + 12.0,
        parts: vec![Part::dome("cap", [0.0, 0.0], 0.0, d * 0.5, h, h * 0.15, mats)],
        swing: true,
        use_,
    }
}

const ANTIGLARE: Tex = tex(
    "antiglare_matte_grain",
    "macro photograph of the anti-glare matte coating of a switched-off black display screen, fine random etched micro-grain, uniform soft frosted surface, dark charcoal grey, flat top-down view, even diffuse light, seamless tileable texture, material scan",
    "pixel grid, subpixels, moire, objects, lighting falloff, perspective, seams, blur, cartoon",
    160_001,
    "labelled micro-grain over the screen face (luminance modulation of 4% at most); the pixel grid stays procedural",
);

const CRACKLE: Tex = tex(
    "crackle_black_paint",
    "macro photograph of matte black wrinkle-finish crackle paint, fine irregular crinkle relief, near black, flat top-down view, even diffuse light, seamless tileable texture, material scan",
    "gloss, rust, scratches, chips, objects, lighting falloff, perspective, seams, blur, cartoon",
    480_001,
    "bare black panel finish (the same seed in HakosukaS20 and ZNismoSport, so both get the same texture)",
);

/// The kit of one dash.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn kit(snake: &str) -> Option<Kit> {
    let k = match snake {
        "kinetic_liquid_metal" => Kit {
            key: "KineticLiquidMetal",
            build: 7,
            computed: [
                studio_pair().to_vec(),
                vec![
                    calc("chrome_matcap", Calc::Matcap(Metal::Chrome), "the chrome ribbon and any bare chrome: one lookup by view-space normal, Fresnel and studio included"),
                    calc("mercury_matcap", Calc::Matcap(Metal::Mercury), "the liquid-metal blob where the film has drained to nothing"),
                    calc("chrome_fresnel", Calc::Fresnel(Metal::Chrome), "bare chrome reflectance over cos theta, for shading with the environment map instead of the matcap"),
                    calc("mercury_fresnel", Calc::Fresnel(Metal::Mercury), "bare liquid-metal reflectance over cos theta"),
                    calc("oil_on_mercury_film", Calc::Film(1.47, Metal::Mercury), "the skin's colour: reflectance of an n = 1.47 oil film on liquid metal by thickness (Jeffreys drainage, 900 to 100 nm) and cos theta; multiply the mirrored studio by it"),
                    calc("oil_on_chrome_film", Calc::Film(1.47, Metal::Chrome), "the same film on solid chrome, for the ribbon where oil runs onto it"),
                ],
            ]
            .concat(),
            deferred: vec![
                "the metaball skin and its thickness field: computed live from the brief's equations (Jeffreys h = sqrt(mu s / (rho g t)), skin at sum q^2 / |x - c|^2 = 1), never a texture",
                "a polished chrome swatch: a smooth mirror has no surface texture to bake, so chrome_matcap and chrome_fresnel replace it",
            ],
            notes: "Everything here is computed on the CPU from measured optical constants (Cr: Johnson & Christy 1974; liquid Hg: Inagaki et al. 1981, the measured stand-in for the brief's liquid metal) and the analytic studio, so it is exact, has no seams or noise, and needs no GPU.",
            ..Kit::default()
        },
        "faraday_mandala" => Kit {
            key: "FaradayMandala",
            build: 74,
            computed: [
                studio_pair().to_vec(),
                vec![
                    calc("mercury_matcap", Calc::Matcap(Metal::Mercury), "the liquid-metal disc: shade the wave field's normal with one lookup"),
                    calc("mercury_fresnel", Calc::Fresnel(Metal::Mercury), "liquid-metal reflectance over cos theta, for shading with the environment map"),
                    calc("chrome_matcap", Calc::Matcap(Metal::Chrome), "the rim ring and the gear's still centre node if they are solid chrome"),
                ],
            ]
            .concat(),
            deferred: vec![
                "the Faraday wave field and its normals: the brief's exact zeta = A sqrt(eps) sum cos(k e_i x + phi_i) is cheaper live than any flipbook, and its amplitude follows the revs",
                "the rpm rim ring's hardware: its diameter comes from the layout",
            ],
            notes: "The brief's metal is galinstan (rho 6440, sigma 0.534); no measured galinstan constants are in the refractiveindex.info database, so liquid mercury (Inagaki et al. 1981), the other room-temperature liquid metal and close in reflectance, stands in. All maps are computed on the CPU, exact and seamless.",
            ..Kit::default()
        },
        "valkyrie_lmp1" => Kit {
            key: "ValkyrieLmp1",
            build: 9,
            textures: vec![tex(
                "forged_carbon_marble",
                "macro photograph of a flat polished forged carbon fibre composite panel, smooth glossy clear resin surface, thin flattened black and dark grey carbon fibre strands in a swirling marbled pattern, satin sheen on the fibres, flat top-down view, even diffuse light, seamless tileable texture, material scan",
                "woven, twill, checker pattern, grid, rocks, stones, coal, crumpled, rough relief, cracks, beige, brown, white specks, objects, lighting falloff, perspective, seams, blur, cartoon",
                90_002,
                "the yoke's forged-carbon marble (brief: albedo + normal, seamless): base colour, and a normal from its luminance",
            )],
            sprites: vec![valkyrie_encoder()],
            deferred: vec!["the 1440x420 forged-carbon yoke with angled shoulders: shape needs the Penpot board"],
            notes: "Forged carbon is SDXL (chopped-fibre marble is organic); the knurl is Blender, on the ring itself. Seed 90002 replaced 90001, which drew rough coal-like chunks with beige specks, not flat resin.",
            ..Kit::default()
        },
        "skyline_r34_mfd" => Kit {
            key: "SkylineR34Mfd",
            build: 12,
            textures: vec![tex(
                "lcd_backlight_mura",
                "macro photograph of a dim late-1990s liquid crystal display panel, soft cloudy backlight mura and faint uneven brightness, very fine diffuser grain, dark grey-green, flat top-down view, even diffuse light, seamless tileable texture",
                "text, pixel grid, numerals, segments, objects, lighting falloff, perspective, seams, blur, cartoon",
                120_001,
                "modulation layer for the gear and speed LCD windows and the MFD tiles; the pixel grid stays procedural",
            )],
            sprites: vec![ring(
                "tach_ring_440",
                440.0,
                14.0,
                [
                    mat::polished(metal::CHROME).named("r34_chamfer"),
                    mat::matte([0.012, 0.012, 0.013]).named("r34_plastic").rough(0.55),
                    mat::matte([0.006, 0.006, 0.007]).named("r34_wall"),
                    mat::matte([0.012, 0.012, 0.013]).named("r34_outer").rough(0.55),
                ],
                None,
                "the 440 px tach ring: matte period black plastic with a polished inner chamfer (14 px width proposed; step 1 checks the R34 ring)",
            )],
            deferred: vec!["the MFD tile housings and the dial-face print: layout and text"],
            notes: "Period LCD and plastics.",
            ..Kit::default()
        },
        "amg_one_f1" => Kit {
            key: "AmgOneF1",
            build: 16,
            textures: vec![ANTIGLARE],
            swatches: vec![tile("carbon_twill_rim", mat::carbon(12.0), "surface of the 780 px carbon rim arc (2x2 twill, 12 px tows)")],
            deferred: vec!["the housing, the rim arc and the F1 shift-light caps: need the Penpot layout and the cap pitch"],
            notes: "Twill carbon is Blender, never SDXL.",
            ..Kit::default()
        },
        "chrono_event_horizon" => Kit {
            key: "ChronoEventHorizon",
            build: 19,
            textures: vec![tex(
                "accretion_shear_filaments",
                "macro photograph of fine sheared luminous smoke filaments all streaming in one horizontal direction, turbulent wisps of varied brightness on black, monochrome grey, flat top-down view, even diffuse light, seamless tileable texture",
                "vortex, spiral, galaxy, stars, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                190_001,
                "greyscale shear texture wrapped polar onto the 1100x190 disk and tinted by the dash; the lensing itself is the dash's deflection table, never this texture",
            )],
            notes: "Only the disk's material grain; the black-hole optics are the dash's maths.",
            ..Kit::default()
        },
        "gt3_rs_apogee" => {
            let ti = metal_set(metal::TITANIUM, mat::spun(metal::TITANIUM), "apogee");
            let mut s = vec![ring(
                "rev_counter_ring_500",
                500.0,
                22.0,
                ti,
                None,
                "the 500 px machined titanium rev-counter ring, no hub (22 px width proposed)",
            )];
            s.extend(led_pair("docking_light_off", "docking_light_on", 30.0, "the 12 docking lights on the 760x30 strip: 30 px caps, unlit and lit"));
            Kit {
                key: "Gt3RsApogee",
                build: 20,
                textures: vec![
                    tex(
                        "mli_foil_crinkle_height",
                        "macro photograph of crumpled thin insulation foil, dense random crinkles and soft folds, uniform mid grey, relief only, flat top-down view, even diffuse light, seamless tileable texture",
                        "gold colour, metallic reflections, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                        200_001,
                        "height only: the dash's gold-foil dial backing shades it with metal GOLD (SDXL never draws the metal)",
                    ),
                    tex(
                        "heat_shield_tile_coating",
                        "macro photograph of a continuous black glassy ceramic coating, fine pitting and subtle mottled matte-to-satin variation across one uninterrupted surface, charcoal black, flat top-down view, even diffuse light, seamless tileable texture, material scan",
                        "tiles, squares, panels, gaps, grid, grout lines, borders, edges, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                        200_004,
                        "surface of the side parts; the tile grid itself is geometry (seed 200004: 200002 drew a 2x2 tile grid into the texture)",
                    ),
                    tex(
                        "star_field_tile",
                        "photograph of a faint star field in deep black space, sparse dim stars of varied brightness, no nebula, seamless tileable texture",
                        "planets, nebula, galaxy, spacecraft, text, objects, perspective, seams, blur, cartoon",
                        200_003,
                        "environment for reflections in the titanium and gold (a tile, not a figure)",
                    ),
                ],
                sprites: s,
                deferred: vec!["the attitude ball and its gimbal cage: markings settle in step 1 against the reference ball"],
                notes: "Space-instrument materials for an out-of-this-world GT3 RS dash.",
                ..Kit::default()
            }
        }
        "bugatti_tourbillon" => Kit {
            key: "BugattiTourbillon",
            build: 25,
            swatches: vec![tile("brushed_steel_linear", mat::brushed(metal::STEEL), "the bridges' base albedo and roughness (20% luminance at most in the kit)")],
            sprites: vec![cap(
                "jewel_lens_30",
                30.0,
                9.0,
                [mat::jewel([0.42, 0.012, 0.03], 0.0, 15.0).named("jewel_ruby"), mat::polished(metal::STEEL).named("jewel_setting")],
                "the 11 jewels on the 30x740 strip and the jewel bearings: a faceted ruby jewel (opaque body, glassy coat) in a polished steel setting, tinted at run time",
            )],
            deferred: vec![
                "Côtes de Genève stripes: the hero library has no stripe pattern yet (add a guilloché 'stripes' mode first)",
                "the 210 px tourbillon cage glTF: part of the dash's 15k-triangle budget and its motion",
            ],
            notes: "Horology steel; the stripe pattern waits for a library addition.",
            ..Kit::default()
        },
        "shibuya_chromatic_prisma" => Kit {
            key: "ShibuyaChromaticPrisma",
            build: 26,
            textures: vec![tex(
                "wet_asphalt_ripple",
                "macro photograph of wet dark asphalt road surface under a thin film of rain water, fine aggregate grain and shallow puddle ripples, greyscale, flat top-down view, even diffuse light, seamless tileable texture",
                "road markings, reflections of objects, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                260_001,
                "greyscale height for the mirror-ripple normal map",
            )],
            notes: "The prism dispersion is the dash's optics; this is only the wet street grain.",
            ..Kit::default()
        },
        "gtr_nismo_mfd" => Kit {
            key: "GtrNismoMfd",
            build: 30,
            swatches: vec![tile("carbon_twill_surround", mat::carbon(10.0), "the meter surround (2x2 twill)")],
            sprites: vec![ring(
                "tach_ring_460",
                460.0,
                20.0,
                metal_set(metal::ALUMINIUM, mat::machined_alu(), "gtr"),
                None,
                "the 460 px chronograph-style tach ring: machined alloy top, polished chamfers (20 px width proposed; step 1 checks the R35 ring)",
            )],
            notes: "Twill carbon is Blender, not SDXL.",
            ..Kit::default()
        },
        "sim_input_trace" => Kit {
            key: "SimInputTrace",
            build: 31,
            textures: vec![ANTIGLARE],
            swatches: vec![tile(
                "dark_anodised_carrier",
                mat::anodised([0.0232, 0.0252, 0.0296]),
                "the LED carrier and the trace window's frame edge",
            )],
            sprites: led_pair("shift_led_32_off", "shift_led_32_on", 32.0, "16 LEDs on 780x32: 32 px caps, tinted go / warning / shift by the kit").to_vec(),
            ..Kit::default()
        },
        "lorenz_butterfly" => Kit {
            key: "LorenzButterfly",
            build: 33,
            textures: vec![tex(
                "pad_rim_powder_coat",
                "macro photograph of matte black powder-coat paint, very fine uniform grain with tiny soft flecks, near black, flat top-down view, even diffuse light, seamless tileable texture",
                "orange peel bumps, scratches, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                330_001,
                "labelled grain on the rims of the true-black pads (the gear pad and the speed pad); the attractor is the dash's RK4",
            )],
            ..Kit::default()
        },
        "pagani_horologerie" => Kit {
            key: "PaganiHorologerie",
            build: 34,
            swatches: vec![
                tile("clous_de_paris_p12", mat::guilloche(metal::STEEL, "clous", 12.0), "the dial faces, raked by the G light (14 alpha at most)"),
                disc("rose_guilloche_p24", mat::guilloche(metal::STEEL, "rose", 24.0), "the secondary guilloché zone (the brief's barleycorn, as the library's rose pattern)"),
                tile("black_anodised_alu", mat::anodised([0.007, 0.007, 0.008]), "the tonneau case chamfer"),
            ],
            sprites: vec![cap(
                "polished_steel_matcap",
                1000.0,
                500.0,
                [mat::polished(metal::STEEL).set("world", 1.0).named("matcap_steel"), mat::polished(metal::STEEL).named("matcap_collar")],
                "a 1000 px polished steel hemisphere under a bright studio world: the matcap for the hands and screws",
            )],
            ..Kit::default()
        },
        "neural_cyber_synapse" => Kit {
            key: "NeuralCyberSynapse",
            build: 37,
            textures: vec![tex(
                "soma_membrane_grain",
                "macro photograph of a translucent biological cell membrane under a microscope, soft cytoplasm granules and faint fibrous texture, monochrome grey, flat top-down view, even diffuse light, seamless tileable texture",
                "cell outlines, nucleus, stain colours, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                370_001,
                "optional modulation inside the 260 px soma (a gradient is the fallback); the nerve front is the dash's Nagumo model",
            )],
            ..Kit::default()
        },
        "suminagashi_marble" => Kit {
            key: "SuminagashiMarble",
            build: 42,
            textures: vec![
                tex(
                    "urushi_lacquer",
                    "macro photograph of polished black urushi lacquer, deep glossy black with very faint brush strokes and fine fibrous undercoat grain, flat top-down view, even diffuse light, seamless tileable texture",
                    "gold leaf, maki-e, decoration, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                    420_001,
                    "the tray rim's lacquer",
                ),
                tex(
                    "still_water_sheen",
                    "macro photograph of the still surface of dark water, extremely faint shallow ripples and micro capillary waves, monochrome grey, flat top-down view, even diffuse light, seamless tileable texture",
                    "reflections, ink, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                    420_002,
                    "greyscale height for the water's faint sheen normal; the marbling is the dash's area-preserving maps",
                ),
                tex(
                    "washi_paper_fibre",
                    "macro photograph of handmade white washi paper, long translucent kozo fibres in a thin sheet, flat top-down view, even diffuse light, seamless tileable texture",
                    "writing, ink, folds, stains, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                    420_003,
                    "labelled fibre texture under the labels",
                ),
            ],
            deferred: vec!["the gear stone (200 px) and the side stones (260x130): pebble geometry is modelled with the layout"],
            ..Kit::default()
        },
        "t50_floodlit" => Kit {
            key: "T50Floodlit",
            build: 43,
            textures: vec![tex(
                "leather_binnacle_black",
                "macro photograph of black full-grain nappa leather, fine natural pebble grain, soft satin sheen, flat top-down view, even diffuse light, seamless tileable texture, material scan",
                "stitching, perforation, plastic, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                430_001,
                "the binnacle surround (leather(...).set(\"texture\")) for edges and reflections",
            )],
            swatches: vec![
                disc("spun_aluminium_chapter", mat::spun(metal::ALUMINIUM), "the chapter ring's fine circular brushing"),
                tile("carbon_twill_pane_frame", mat::carbon(10.0), "frames of the two 300x220 panes"),
            ],
            sprites: vec![
                ring(
                    "dial_ring_480",
                    480.0,
                    12.0,
                    [
                        mat::polished(metal::ALUMINIUM).named("t50_chamfer"),
                        mat::polished(metal::ALUMINIUM).named("t50_top"),
                        mat::brushed(metal::GUNMETAL).named("t50_wall"),
                        mat::polished(metal::ALUMINIUM).named("t50_outer"),
                    ],
                    Some(mat::spun(metal::ALUMINIUM).named("t50_chapter")),
                    "the 480 px thin polished bezel over a satin spun chapter blank, no numerals (the scale follows the car's max rpm)",
                ),
                cap(
                    "centre_cap_200",
                    200.0,
                    26.0,
                    [mat::spun(metal::ALUMINIUM).named("t50_cap"), mat::polished(metal::ALUMINIUM).named("t50_cap_chamfer")],
                    "the 200 px spun-turned cap over the hub",
                ),
            ],
            notes: "The needle's floodlit shadow is the dash's projection maths; these are the hardware it falls on.",
            ..Kit::default()
        },
        "lfa_v10_nurburgring" => Kit {
            key: "LfaV10Nurburgring",
            build: 47,
            swatches: vec![disc("spun_titanium", mat::spun(metal::TITANIUM), "the ring's circular grain and its tangential anisotropy")],
            sprites: vec![ring(
                "titanium_ring_480",
                480.0,
                24.0,
                metal_set(metal::TITANIUM, mat::spun(metal::TITANIUM), "lfa"),
                None,
                "the floating 480 px titanium ring, 24 px wide (r_in 216), no face or pivot",
            )],
            ..Kit::default()
        },
        "hakosuka_s20" => Kit {
            key: "HakosukaS20",
            build: 48,
            textures: vec![
                CRACKLE,
                tex(
                    "worn_steering_wood",
                    "macro photograph of worn varnished dark wood from a vintage steering wheel rim, fine straight grain, soft wear, flat top-down view, even diffuse light, seamless tileable texture",
                    "knots, cracks, stitching, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                    480_002,
                    "wood in the environment the bright rings reflect",
                ),
            ],
            sprites: {
                let chrome = || {
                    [
                        mat::polished(metal::CHROME).named("s20_chamfer"),
                        mat::polished(metal::CHROME).named("s20_top"),
                        mat::matte([0.01, 0.01, 0.01]).named("s20_wall"),
                        mat::polished(metal::CHROME).named("s20_outer"),
                    ]
                };
                let mut v = vec![
                    ring("tube_ring_420", 420.0, 8.0, chrome(), Some(mat::matte([0.006, 0.006, 0.006]).named("s20_tube")), "the two big dials: a thin bright chrome ring on a deep matte tube (8 px width proposed)"),
                    ring("small_dial_ring_150", 150.0, 5.0, chrome(), None, "the three 150 px small dials (5 px width proposed); render straight, the 12° turn is in the kit"),
                ];
                v.extend(jewel_pair("jewel_lamp_28_off", "jewel_lamp_28_on", 28.0, "the 10 lamps on 740x28: 28 px jewel lenses"));
                v
            },
            ..Kit::default()
        },
        "sim_pit_wall" => Kit {
            key: "SimPitWall",
            build: 49,
            textures: vec![ANTIGLARE],
            swatches: vec![tile("carbon_twill_carrier", mat::carbon(10.0), "the LED carrier surface")],
            sprites: led_pair("shift_led_32_off", "shift_led_32_on", 32.0, "14 LEDs on 760x32: 32 px caps (the same design as SimInputTrace's)").to_vec(),
            ..Kit::default()
        },
        "kaleidoscope_mirror" => Kit {
            key: "KaleidoscopeMirror",
            build: 51,
            textures: vec![tex(
                "chip_glass_inclusions",
                "macro photograph of clear antique glass with tiny seed bubbles and faint wavy striae, monochrome grey on black, flat top-down view, even diffuse light, seamless tileable texture",
                "cracks, chips, colour, edges, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                510_001,
                "labelled grain map inside the eight chips; the mirror copies are the dash's dihedral group",
            )],
            sprites: vec![ring(
                "eyepiece_ring_380",
                380.0,
                10.0,
                [
                    mat::polished(metal::BLACK_CHROME).named("kal_chamfer"),
                    mat::knurled(metal::BLACK_CHROME, 4.0, false).named("kal_knurl"),
                    mat::brushed(metal::GUNMETAL).named("kal_wall"),
                    mat::knurled(metal::BLACK_CHROME, 4.0, false).named("kal_outer"),
                ],
                None,
                "the knurled black-chrome eyepiece ring round the 380 px figure (10 px width and 4 px knurl pitch proposed)",
            )],
            ..Kit::default()
        },
        "koenigsegg_gyro" => Kit {
            key: "KoenigseggGyro",
            build: 52,
            swatches: vec![
                tile("carbon_twill_12", mat::carbon(12.0), "the rotating ring band"),
                tile("brushed_titanium", mat::brushed(metal::TITANIUM), "the inner rim"),
            ],
            sprites: vec![ring(
                "instrument_ring_540",
                540.0,
                30.0,
                [
                    mat::polished(metal::TITANIUM).named("kgy_chamfer"),
                    mat::carbon(12.0).named("kgy_carbon"),
                    mat::brushed(metal::TITANIUM).named("kgy_wall"),
                    mat::polished(metal::TITANIUM).named("kgy_outer"),
                ],
                None,
                "the 540 px instrument ring: carbon band, brushed titanium rim, polished chamfers (30 px width proposed); the kit turns it with steering",
            )],
            ..Kit::default()
        },
        "aperture_nine" => Kit {
            key: "ApertureNine",
            build: 53,
            swatches: vec![tile(
                "brushed_black_titanium_blade",
                mat::brushed([0.05, 0.05, 0.055]).set("coat", 1.0).set("coat_rough", 0.2),
                "the iris blades' anisotropic lobe (18% luminance at most); the iris geometry is the dash's nine-disc maths",
            )],
            ..Kit::default()
        },
        "revuelto_corsa" => Kit {
            key: "RevueltoCorsa",
            build: 56,
            textures: vec![tex(
                "forged_composite_large_chips",
                "macro photograph of forged carbon composite with large flat chopped carbon flakes, broad overlapping chips under glossy resin, black and graphite grey with soft fibre sheen, flat top-down view, even diffuse light, seamless tileable texture, material scan",
                "woven, twill, grid, checker pattern, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                560_001,
                "the hexagon pod body",
            )],
            deferred: vec![
                "the hex-mesh grille: the hero library has no hexagon perforation yet",
                "the 300 px hexagon pod: the library has no hexagon outline yet, and the anodise tint is not in the brief",
            ],
            ..Kit::default()
        },
        "z_nismo_sport" => Kit {
            key: "ZNismoSport",
            build: 57,
            textures: vec![
                CRACKLE,
                tex(
                    "dash_top_leatherette",
                    "macro photograph of black automotive leatherette dash cover, fine embossed grain, soft matte sheen, flat top-down view, even diffuse light, seamless tileable texture, material scan",
                    "stitching, seams, perforation, text, objects, lighting falloff, perspective, blur, cartoon",
                    570_002,
                    "the dash top the pods stand on (stitching is geometry)",
                ),
            ],
            sprites: vec![
                ring(
                    "hooded_pod_170",
                    170.0,
                    12.0,
                    [
                        mat::polished(metal::CHROME).named("z_chamfer"),
                        mat::matte([0.012, 0.012, 0.012]).named("z_crackle").rough(0.7),
                        mat::matte([0.006, 0.006, 0.006]).named("z_wall"),
                        mat::matte([0.012, 0.012, 0.012]).named("z_outer").rough(0.7),
                    ],
                    Some(mat::matte([0.004, 0.004, 0.004]).named("z_tube")),
                    "the 170 px hooded pods: matte crackle-black tube with a polished inner ring (12 px width proposed); render straight, the turn is in the kit",
                ),
                Sprite {
                    name: "pod_crystal_170",
                    size: 180.0,
                    parts: vec![Part::crystal("crystal", [0.0, 0.0], 12.0, 79.0, 4.0, 2.0, mat::glass([1.0, 1.0, 1.0]))],
                    swing: true,
                    use_: "the 170 px pod crystal (reflections only, drawn over the needle)",
                },
            ],
            ..Kit::default()
        },
        "golden_angle" => Kit {
            key: "GoldenAngle",
            build: 60,
            textures: vec![tex(
                "bead_inner_sparkle",
                "macro photograph of tiny sparkling inclusions inside clear crystal, sparse bright glints and fine internal facets on black, monochrome, flat top-down view, even diffuse light, seamless tileable texture",
                "gemstones, shapes, colour, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                600_001,
                "labelled sparkle grain inside the bead; the seed spiral is the dash's Vogel model",
            )],
            sprites: vec![cap(
                "seed_bead_26",
                26.0,
                9.0,
                [mat::jewel([0.6, 0.42, 0.16], 0.0, 13.0).named("bead_crystal"), mat::piano_black().named("bead_seat")],
                "the 26 px domed crystal bead; one sprite serves every seed",
            )],
            ..Kit::default()
        },
        "rattrapante_sector" => Kit {
            key: "RattrapanteSector",
            build: 61,
            swatches: vec![disc("spun_titanium_arc_rail", mat::spun(metal::TITANIUM), "the arc rail's tangential grain")],
            ..Kit::default()
        },
        "eva_bio_sync" => Kit {
            key: "EvaBioSync",
            build: 64,
            textures: vec![tex(
                "cell_nucleus_grain",
                "macro photograph of translucent cell nucleus material under a microscope, soft granular chromatin texture, monochrome grey, flat top-down view, even diffuse light, seamless tileable texture",
                "cell outlines, hexagons, stain colours, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                640_001,
                "optional modulation of the lit-cell gradient",
            )],
            ..Kit::default()
        },
        "mclaren_slim_display" => Kit {
            key: "MclarenSlimDisplay",
            build: 65,
            textures: vec![
                ANTIGLARE,
                tex(
                    "alcantara_black",
                    "macro photograph of black alcantara microfibre suede, soft dense nap with faint directional sheen, flat top-down view, even diffuse light, seamless tileable texture, material scan",
                    "leather pores, stitching, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                    650_002,
                    "the dash top the slab folds into (alcantara(...).set(\"texture\"))",
                ),
            ],
            ..Kit::default()
        },
        "bionic_spaceframe" => Kit {
            key: "BionicSpaceframe",
            build: 68,
            swatches: vec![tile("brushed_titanium_strut", mat::brushed(metal::TITANIUM), "the struts' surface (base 12% luminance at most); the truss is the dash's Michell geometry")],
            ..Kit::default()
        },
        "star_sail_origami" => Kit {
            key: "StarSailOrigami",
            build: 70,
            swatches: vec![tile("brushed_film_hairline", mat::brushed(metal::ALUMINIUM), "optional facet sheen (use its normal); the fold is the dash's Miura-ori maths")],
            ..Kit::default()
        },
        "hyperbolic_tiles" => Kit {
            key: "HyperbolicTiles",
            build: 72,
            textures: vec![tex(
                "etched_glass_grain",
                "macro photograph of finely acid-etched frosted glass, uniform fine matte grain, monochrome grey, flat top-down view, even diffuse light, seamless tileable texture",
                "scratches, fingerprints, patterns, text, objects, lighting falloff, perspective, seams, blur, cartoon",
                720_001,
                "labelled sparkle map on the two disc faces; the tiling is the dash's Poincaré maths",
            )],
            sprites: vec![ring(
                "disc_rim_ring_400",
                400.0,
                8.0,
                metal_set(metal::ALUMINIUM, mat::machined_alu(), "hyp"),
                None,
                "the 400 px thin machined rim (8 px width proposed); one render, mirrored for both discs",
            )],
            ..Kit::default()
        },
        _ => return None,
    };
    Some(k)
}
