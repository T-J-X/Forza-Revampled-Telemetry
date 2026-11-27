//! `asset-forge selftest`: the same small jobs on whichever GPU lane the run uses, timed, so
//! the local card and the pod can be compared like for like and a new pod can be proved to
//! work end to end. Three jobs:
//! - an SDXL 1024² texture with the 4x upscale to 4096², run twice (the first run also loads
//!   the checkpoint);
//! - SDXL base and refiner with the canny ControlNet over that texture;
//! - a 3840x2160 Cycles render of a glass plate at 128 samples.
//!
//! Outputs go to `target/asset-forge/selftest/<lane>/`. Hold the lane's lock while it runs.
use crate::comfy::Comfy;
use crate::error::{io_err, tool_err, Result};
use crate::hero::sdxl::{self, Control, Job};
use crate::hero::texture::{Models, NEGATIVE};
use crate::paths::{Paths, UPSCALER};
use serde_json::json;
use std::path::Path;
use std::time::{Duration, Instant};

/// The texture prompt (one of the organic surfaces the dashes use).
const PROMPT: &str = "macro photograph of black full-grain automotive nappa leather, fine pebble grain, \
    soft even diffuse studio light, flat top-down, seamless tileable texture, 8k material scan";
/// The texture seeds for the first and the second run.
const SEEDS: [u64; 2] = [90_001, 90_002];
/// The render size.
const RENDER: [u32; 2] = [3840, 2160];
/// The render's sample count.
const SAMPLES: u32 = 128;

/// A glass plate with a bevelled rim over a dark metal backing, under the studio HDRI and two
/// area lights. It prints which Cycles device it rendered on.
const GLASS: &str = r#"
sc = reset()
dev = gpu_cycles(sc, samples=int(ARGS['samples']), denoise=True, bounces=12)
prefs = bpy.context.preferences.addons['cycles'].preferences
log('cycles', prefs.compute_device_type, dev, '+'.join(d.name for d in prefs.devices if d.use))
resolution(sc, ARGS['w'], ARGS['h'])
view(sc, 'Standard')
studio_world(sc, strength=1.0)

plate = plate_mesh('plate', outline('squircle', 6.0, 3.2, n=4.5), 0.35, 0.12, segs=24)
m, nt = new_mat('glass')
bsdf = node(nt, 'ShaderNodeBsdfGlass')
bsdf.inputs['IOR'].default_value = 1.52
bsdf.inputs['Roughness'].default_value = 0.02
lk(nt, bsdf.outputs[0], node(nt, 'ShaderNodeOutputMaterial').inputs['Surface'])
plate.data.materials.append(m)

back = plate_mesh('back', outline('rrect', 7.0, 4.0, r=0.4), 0.05, 0.0)
back.location.z = -0.3
m2, nt2 = new_mat('back')
metal = node(nt2, 'ShaderNodeBsdfPrincipled')
metal.inputs['Base Color'].default_value = (0.02, 0.05, 0.09, 1.0)
metal.inputs['Metallic'].default_value = 1.0
metal.inputs['Roughness'].default_value = 0.35
lk(nt2, metal.outputs[0], node(nt2, 'ShaderNodeOutputMaterial').inputs['Surface'])
back.data.materials.append(m2)

area_light('key', (3.0, -4.0, 6.0), (0.0, 0.0, 0.0), 4.0, 900.0)
area_light('rim', (-4.0, 3.0, 3.0), (0.0, 0.0, 0.0), 2.0, 400.0, color=(0.6, 0.8, 1.0))
persp_cam(sc, (0.0, -6.5, 5.5), (0.0, 0.0, 0.0), lens=50.0)
save(sc, os.path.join(OUT, 'selftest_glass.png'), depth='16', mode='RGBA')
done()
"#;

fn seconds(span: Duration) -> String {
    format!("{:6.1} s", span.as_secs_f32())
}

fn save(dir: &Path, name: &str, bytes: &[u8]) -> Result<String> {
    let path = dir.join(name);
    std::fs::write(&path, bytes).map_err(|e| io_err(&path, e))?;
    Ok(crate::lane::png_size(bytes).map_or_else(|| "?".to_owned(), |(w, h)| format!("{w}x{h}")))
}

/// The line Blender's script printed about its render device, such as `OPTIX GPU <names>`.
fn cycles_device(log: &str) -> &str {
    log.lines()
        .find_map(|l| l.trim().strip_prefix("[forge] cycles "))
        .unwrap_or("unknown")
}

/// How long Blender had been running when it saved the picture. Blender 5.3 starts each of its
/// own log lines with that clock, as in `00:15.149  render  | Saved: '...'`. The time covers
/// Blender's start, building the scene, the render and the save, but no file transfer.
fn saved_at(log: &str) -> &str {
    log.lines()
        .rev()
        .find(|l| l.contains("| Saved:"))
        .and_then(|l| l.split_whitespace().next())
        .unwrap_or("?")
}

/// Runs the three jobs on the run's lane and prints the timings.
pub fn run(paths: &Paths) -> Result<()> {
    let lane = paths.lane.name();
    let dir = paths.scratch.join("selftest").join(lane);
    std::fs::create_dir_all(&dir).map_err(|e| io_err(&dir, e))?;
    println!("selftest on the {lane} lane");

    let started = Instant::now();
    let comfy = Comfy::connect(paths)?;
    let ready = started.elapsed();
    let stats = comfy.stats().unwrap_or_default();
    let device = stats["devices"][0]["name"].as_str().unwrap_or("?").to_owned();
    let vram = stats["devices"][0]["vram_total"].as_u64().unwrap_or(0) / 1_048_576;
    println!(
        "  comfyui   {device}, {vram} MB VRAM (ready after {})",
        seconds(ready).trim()
    );

    let models = Models::probe(&comfy);
    for (run, seed) in SEEDS.iter().enumerate() {
        let mut job = Job::new(PROMPT, NEGATIVE, *seed);
        job.steps = 32;
        job.upscale = Some(UPSCALER.into());
        job.prefix = format!("forge_selftest_{run}");
        let t = Instant::now();
        let images = comfy.run_retry(&sdxl::build(&job), Duration::from_secs(900))?;
        let took = t.elapsed();
        let first = images.first().ok_or_else(|| tool_err("selftest", "no texture"))?;
        let size = save(&dir, &format!("texture_{run}.png"), first)?;
        let label = if run == 0 {
            "first run, loads the model"
        } else {
            "second run"
        };
        println!(
            "  texture   SDXL 1024² 32 steps + 4x upscale → {size}  {}  ({label})",
            seconds(took)
        );
    }

    match (models.refiner.clone(), models.canny.clone()) {
        (Some(refiner), Some(canny)) => {
            // The control image is the texture at SDXL's own size, so the upload stays small.
            let small = crate::img::load(&dir.join("texture_1.png"))?
                .to_channels(3)
                .resize(1024, 1024, false);
            let image = comfy.upload_image(
                "forge_selftest_control.png",
                &crate::img::encode_png(&small, false, true)?,
            )?;
            let mut job = Job::new(PROMPT, NEGATIVE, SEEDS[0]);
            job.refiner = Some((refiner, 0.8));
            job.control = Some(Control {
                model: canny,
                image,
                canny: true,
                strength: 0.6,
                end: 0.8,
            });
            job.prefix = "forge_selftest_control".into();
            let t = Instant::now();
            let images = comfy.run_retry(&sdxl::build(&job), Duration::from_secs(900))?;
            let took = t.elapsed();
            let first = images
                .first()
                .ok_or_else(|| tool_err("selftest", "no controlled image"))?;
            let size = save(&dir, "control.png", first)?;
            println!(
                "  control   SDXL base + refiner + canny ControlNet → {size}  {}",
                seconds(took)
            );
        }
        _ => println!("  control   skipped: the refiner or the canny ControlNet is not installed"),
    }
    comfy.free();

    let out = dir.join("glass");
    let params = json!({
        "out": out.to_string_lossy(),
        "seed": 7,
        "w": RENDER[0],
        "h": RENDER[1],
        "samples": SAMPLES,
    });
    let t = Instant::now();
    let log = crate::blender::run(paths, "selftest_glass", GLASS, &params)?;
    let took = t.elapsed();
    let picture = out.join("selftest_glass.png");
    if !picture.is_file() {
        return Err(tool_err("selftest", format!("{} was not written", picture.display())));
    }
    println!(
        "  blender   Cycles {} {}x{} {SAMPLES} samples  {}  (Blender itself ran {} to the save; the rest is file transfer)",
        cycles_device(&log),
        RENDER[0],
        RENDER[1],
        seconds(took),
        saved_at(&log)
    );
    println!("  outputs   {}", dir.display());
    println!("selftest: passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_device_line_is_read_from_the_log() {
        let log =
            "Blender 5.3.0\n[forge] hdri /x/studio.exr\n[forge] cycles OPTIX GPU NVIDIA GeForce RTX 4090\nFORGE-OK\n";
        assert_eq!(cycles_device(log), "OPTIX GPU NVIDIA GeForce RTX 4090");
        assert_eq!(cycles_device("nothing"), "unknown");
    }

    #[test]
    fn blenders_clock_at_the_save_is_read_from_its_log() {
        // Lines from the first render on the pod, 2026-11-27.
        let log =
            "00:05.746  cycles           | WARNING HIPEW initialization failed: Error opening HIP dynamic library\n\
                   [forge] cycles OPTIX GPU NVIDIA GeForce RTX 4090\n\
                   Blender 5.3.0 (hash 02f0899d78e3 built 2026-11-10 01:31:18)\n\
                   00:15.149  render           | Saved: '/workspace/jobs/selftest_glass/out/selftest_glass.png'\n\
                   [forge] wrote /workspace/jobs/selftest_glass/out/selftest_glass.png\nFORGE-OK\n";
        assert_eq!(saved_at(log), "00:15.149");
        assert_eq!(cycles_device(log), "OPTIX GPU NVIDIA GeForce RTX 4090");
        assert_eq!(saved_at("no save line"), "?");
    }

    #[test]
    fn the_glass_recipe_ends_properly_and_reports_its_device() {
        assert!(GLASS.contains("log('cycles'"));
        assert!(GLASS.trim_end().ends_with("done()"));
        assert!(GLASS.contains("selftest_glass.png"));
    }
}
