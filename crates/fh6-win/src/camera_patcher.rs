use crate::car_brands::{BindCarBrandError, BindOutcome, CarBinding};
use fh6_core::config::AppConfig;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct CameraPatcher;

impl CameraPatcher {
    pub fn is_forza_running() -> bool {
        false
    }

    pub fn patch_camera_ini<P: AsRef<Path>>(path: P, cfg: &AppConfig) -> io::Result<()> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let mut lines = Vec::new();
        for line in content.lines() {
            let lower = line.to_ascii_lowercase();
            let new_line = if lower.starts_with("driver\\fov ") {
                format!("Driver\\FOV {}", fmt_fov_int(cfg.standard_cockpit_fov))
            } else if lower.starts_with("hoodcam\\fov ") {
                format!("HoodCam\\FOV {}", fmt_fov_int(cfg.bonnet_fov))
            } else if lower.starts_with("followlowcam\\fov ") {
                format!("FollowLowCam\\FOV {}", fmt_fov_one(cfg.chase_near_fov))
            } else if lower.starts_with("followhighcam\\fov ") {
                format!("FollowHighCam\\FOV {}", fmt_fov_one(cfg.chase_far_fov))
            } else if lower.starts_with("bumperhighcam\\fov ") {
                format!("BumperHighCam\\FOV {}", fmt_fov_int(cfg.bumper_fov))
            } else if lower.starts_with("driver\\defaultoffsetfromsteeringwheelx ") {
                format!(
                    "Driver\\DefaultOffsetFromSteeringWheelX {}",
                    fmt_offset(cfg.cockpit_offset_x)
                )
            } else if lower.starts_with("driver\\defaultoffsetfromsteeringwheely ") {
                format!(
                    "Driver\\DefaultOffsetFromSteeringWheelY {}",
                    fmt_offset(cfg.cockpit_offset_y)
                )
            } else if lower.starts_with("driver\\defaultoffsetfromsteeringwheelz ") {
                format!(
                    "Driver\\DefaultOffsetFromSteeringWheelZ {}",
                    fmt_offset(cfg.cockpit_offset_z)
                )
            } else if lower.starts_with("hoodcam\\camyoffset ") {
                format!("HoodCam\\CamYOffset {}", fmt_offset(cfg.hood_offset_y))
            } else if lower.starts_with("hoodcam\\camzoffset ") {
                format!("HoodCam\\CamZOffset {}", fmt_offset(cfg.hood_offset_z))
            } else if lower.starts_with("hoodcam\\campitchoffset ") {
                format!("HoodCam\\CamPitchOffset {}", fmt_offset(cfg.hood_pitch_offset))
            } else {
                line.to_string()
            };
            lines.push(new_line);
        }
        let out = lines.join("\n");
        let mut f = fs::File::create(path)?;
        f.write_all(out.as_bytes())
    }

    pub fn patch_camera_physics<P: AsRef<Path>>(path: P, cfg: &AppConfig) -> io::Result<()> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut in_driver = false;
        let mut in_hood = false;
        let mut driver_layers = Vec::new();
        let mut hood_layers = Vec::new();
        let mut driver_start = None;
        let mut hood_start = None;
        for (i, line) in lines.iter().enumerate() {
            if line.contains("<DriverCam") {
                in_driver = true;
                in_hood = false;
                driver_start = Some(i);
                driver_layers.clear();
            }
            if line.contains("</DriverCam>") {
                in_driver = false;
            }
            if line.contains("<Hood") && !line.contains("</Hood") {
                in_hood = true;
                in_driver = false;
                hood_start = Some(i);
                hood_layers.clear();
            }
            if line.contains("</Hood>") {
                in_hood = false;
            }
            if line.contains("<Layer") && (in_driver || in_hood) {
                let is_match = line.contains("InputType=\"ConstantOne\"")
                    && line.contains("NoiseType=\"None\"")
                    && (line.contains("OutputType=\"CarSpaceXYZOffset\"")
                        || line.contains("OutputType=\"CameraSpaceXYZOffset\""));
                if is_match {
                    if in_driver {
                        driver_layers.push(i);
                    } else if in_hood {
                        hood_layers.push(i);
                    }
                }
            }
        }
        if let Some(&i) = driver_layers.last() {
            let mut l = lines[i].clone();
            l = replace_attr(&l, "OutputType", "CarSpaceXYZOffset");
            l = replace_attr(&l, "Param.x", &fmt_offset6(cfg.cockpit_offset_x));
            l = replace_attr(&l, "Param.y", &fmt_offset6(cfg.cockpit_offset_y));
            l = replace_attr(&l, "Param.z", &fmt_offset6(cfg.cockpit_offset_z));
            lines[i] = l;
        }
        if let Some(&i) = hood_layers.last() {
            let mut l = lines[i].clone();
            l = replace_attr(&l, "OutputType", "CarSpaceXYZOffset");
            l = replace_attr(&l, "Param.y", &fmt_offset6(cfg.hood_offset_y));
            l = replace_attr(&l, "Param.z", &fmt_offset6(cfg.hood_offset_z));
            lines[i] = l;
        }
        if let Some(i) = driver_start {
            let c = count_layers(&lines, i, "DriverCam");
            lines[i] = replace_attr(&lines[i], "NumLayers", &c.to_string());
        }
        if let Some(i) = hood_start {
            let c = count_layers(&lines, i, "Hood");
            lines[i] = replace_attr(&lines[i], "NumLayers", &c.to_string());
        }
        let out = lines.join("\n");
        let mut f = fs::File::create(path)?;
        f.write_all(out.as_bytes())
    }

    pub fn patch_user_config<P: AsRef<Path>>(path: P, cfg: &AppConfig) -> io::Result<()> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let tags = [
            ("DriverNoWheelFOV", cfg.wheel_less_cockpit_fov),
            ("DLC1DriverNoWheelFOV", cfg.wheel_less_cockpit_fov),
            ("DriverFOV", cfg.standard_cockpit_fov),
            ("DLC1DriverFOV", cfg.standard_cockpit_fov),
            ("HoodFOV", cfg.bonnet_fov),
            ("DLC1HoodFOV", cfg.bonnet_fov),
            ("FollowLowFOV", cfg.chase_near_fov),
            ("DLC1FollowLowFOV", cfg.chase_near_fov),
            ("FollowHighFOV", cfg.chase_far_fov),
            ("DLC1FollowHighFOV", cfg.chase_far_fov),
            ("BumperHighFOV", cfg.bumper_fov),
            ("DLC1BumperHighFOV", cfg.bumper_fov),
        ];
        for (tag, v) in tags {
            for line in &mut lines {
                if line.contains(&format!("<{tag}")) {
                    *line = replace_attr(line, "value", &fmt_offset6(v));
                }
            }
        }
        let out = lines.join("\n");
        let mut f = fs::File::create(path)?;
        f.write_all(out.as_bytes())
    }

    /// Binds a car ordinal to a brand in `car_brands.json`. See [`crate::car_brands`].
    ///
    /// # Errors
    /// See [`BindCarBrandError`]; the file is untouched on error.
    pub fn bind_car_brand(path: impl AsRef<Path>, car: CarBinding<'_>) -> Result<BindOutcome, BindCarBrandError> {
        crate::car_brands::bind_car_brand(path.as_ref(), car)
    }
}

fn fmt_fov_int(v: f32) -> String {
    format!("{:.0}", v)
}
fn fmt_fov_one(v: f32) -> String {
    format!("{:.1}", v)
}
fn fmt_offset(v: f32) -> String {
    format!("{:.3}", v)
}
fn fmt_offset6(v: f32) -> String {
    format!("{:.6}", v)
}

fn replace_attr(line: &str, attr: &str, value: &str) -> String {
    if let Some(start) = line.find(&format!("{attr}=\"")) {
        let after = start + attr.len() + 2;
        if let Some(end) = line[after..].find('"') {
            let mut out = String::new();
            out.push_str(&line[..after]);
            out.push_str(value);
            out.push_str(&line[after + end..]);
            return out;
        }
    }
    if let Some(pos) = line.rfind('>') {
        let mut out = line[..pos].to_string();
        out.push_str(&format!(" {attr}=\"{value}\""));
        out.push('>');
        return out;
    }
    line.to_string()
}

fn count_layers(lines: &[String], start_idx: usize, node: &str) -> usize {
    let end = format!("</{node}>");
    let mut c = 0;
    for l in lines.iter().skip(start_idx + 1) {
        if l.contains(&end) {
            break;
        }
        if l.contains("<Layer") {
            c += 1;
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    fn cfg() -> AppConfig {
        AppConfig {
            standard_cockpit_fov: 62.0,
            bonnet_fov: 55.0,
            chase_near_fov: 70.5,
            chase_far_fov: 85.2,
            bumper_fov: 50.0,
            cockpit_offset_x: 1.234,
            cockpit_offset_y: -0.567,
            cockpit_offset_z: 0.891,
            hood_offset_y: 0.123,
            hood_offset_z: -0.456,
            hood_pitch_offset: 2.345,
            wheel_less_cockpit_fov: 60.0,
            ..Default::default()
        }
    }
    fn tmp_path(name: &str) -> std::path::PathBuf {
        let mut p = env::temp_dir();
        p.push(format!("fh6_test_{name}_{}.tmp", std::process::id()));
        p
    }
    #[test]
    fn ini() {
        let p = tmp_path("ini");
        std::fs::write(
            &p,
            "Driver\\FOV 45\nHoodCam\\FOV 45\nDriver\\DefaultOffsetFromSteeringWheelX 0\n",
        )
        .unwrap();
        CameraPatcher::patch_camera_ini(&p, &cfg()).unwrap();
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains("Driver\\FOV 62"));
        assert!(s.contains("HoodCam\\FOV 55"));
        std::fs::remove_file(p).ok();
    }
    #[test]
    fn physics() {
        let p = tmp_path("phys");
        std::fs::write(&p, "<DriverCam NumLayers=\"1\">\n<Layer InputType=\"ConstantOne\" OutputType=\"CameraSpaceXYZOffset\" NoiseType=\"None\"/>\n</DriverCam>").unwrap();
        CameraPatcher::patch_camera_physics(&p, &cfg()).unwrap();
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains("Param.x=\"1.234000\""));
        std::fs::remove_file(p).ok();
    }
}
