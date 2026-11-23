//! The material cookbook as data. Each builder returns a [`Mat`]: a `kind` plus parameters that
//! [`super::bpy::HERO`]'s `make_mat` turns into a Principled node tree. Patterns (brushing,
//! sunray, guilloché, knurling, carbon weave, leather grain) are procedural in object space, so
//! they line up with the part's centre and need no UVs. Lengths are local px.
//!
//! Colours are **linear** RGB. Metals use measured F0 reflectance (the `metal` module) rather
//! than eyeballed greys; everything else is albedo.
use serde_json::{json, Map, Value};

/// Measured metal reflectance at normal incidence (linear RGB, from published n/k data).
pub mod metal {
    /// Aluminium.
    pub const ALUMINIUM: [f32; 3] = [0.913, 0.922, 0.924];
    /// Titanium.
    pub const TITANIUM: [f32; 3] = [0.542, 0.497, 0.449];
    /// Chromium (chrome plating).
    pub const CHROME: [f32; 3] = [0.549, 0.556, 0.554];
    /// Stainless steel.
    pub const STEEL: [f32; 3] = [0.672, 0.637, 0.585];
    /// Nickel.
    pub const NICKEL: [f32; 3] = [0.660, 0.609, 0.526];
    /// Gold.
    pub const GOLD: [f32; 3] = [1.000, 0.766, 0.336];
    /// Copper.
    pub const COPPER: [f32; 3] = [0.955, 0.638, 0.538];
    /// Black chrome / smoked PVD.
    pub const BLACK_CHROME: [f32; 3] = [0.10, 0.10, 0.11];
    /// Gunmetal PVD.
    pub const GUNMETAL: [f32; 3] = [0.20, 0.20, 0.22];
}

/// A material description for Blender.
#[derive(Debug, Clone, PartialEq)]
pub struct Mat {
    /// Kind plus parameters (`kind`, `name`, …).
    pub v: Map<String, Value>,
}

impl Mat {
    /// A material of `kind` named `name`.
    #[must_use]
    pub fn new(kind: &str, name: &str) -> Self {
        let mut v = Map::new();
        v.insert("kind".into(), json!(kind));
        v.insert("name".into(), json!(name));
        Self { v }
    }

    /// Sets a parameter.
    #[must_use]
    pub fn set(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.v.insert(key.into(), value.into());
        self
    }

    /// Base colour (linear RGB).
    #[must_use]
    pub fn color(self, rgb: [f32; 3]) -> Self {
        self.set("color", json!(rgb))
    }

    /// Roughness.
    #[must_use]
    pub fn rough(self, r: f32) -> Self {
        self.set("rough", json!(r))
    }

    /// Renames (one cookbook material used twice with different parameters).
    #[must_use]
    pub fn named(self, name: &str) -> Self {
        self.set("name", json!(name))
    }

    /// The material's name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.v.get("name").and_then(Value::as_str).unwrap_or("mat")
    }

    /// JSON for Blender.
    #[must_use]
    pub fn to_json(&self) -> Value {
        Value::Object(self.v.clone())
    }
}

/// Bead-blasted / machined aluminium: bright, slightly rough, fine isotropic grain.
#[must_use]
pub fn machined_alu() -> Mat {
    Mat::new("metal", "machined_alu")
        .color(metal::ALUMINIUM)
        .rough(0.28)
        .set("finish", "bead")
        .set("grain", 1.2)
}

/// Anodised aluminium: metal under a tinted dielectric coat (the dye layer).
#[must_use]
pub fn anodised(tint: [f32; 3]) -> Mat {
    Mat::new("metal", "anodised")
        .color(metal::ALUMINIUM)
        .rough(0.32)
        .set("finish", "bead")
        .set("grain", 1.0)
        .set("coat", 1.0)
        .set("coat_rough", 0.18)
        .set("coat_tint", json!(tint))
}

/// Linear brushed metal (grain along x).
#[must_use]
pub fn brushed(base: [f32; 3]) -> Mat {
    Mat::new("metal", "brushed")
        .color(base)
        .rough(0.3)
        .set("finish", "linear")
        .set("aniso", 0.85)
        .set("grain", 0.8)
}

/// Sunray (sunburst) dial: grain runs radially from the centre, so a bright wedge pair turns
/// with the light. Full strength: this is the dial, not an overlay.
#[must_use]
pub fn sunray(base: [f32; 3]) -> Mat {
    Mat::new("metal", "sunray")
        .color(base)
        .rough(0.17)
        .set("finish", "sunray")
        .set("aniso", 0.95)
        .set("grain", 0.7)
}

/// Spun / turned (concentric grain, like a lathe-turned bezel top or a CD).
#[must_use]
pub fn spun(base: [f32; 3]) -> Mat {
    Mat::new("metal", "spun")
        .color(base)
        .rough(0.26)
        .set("finish", "spun")
        .set("aniso", 0.9)
        .set("grain", 0.8)
}

/// Polished metal (mirror chamfers, needles, hubs).
#[must_use]
pub fn polished(base: [f32; 3]) -> Mat {
    Mat::new("metal", "polished")
        .color(base)
        .rough(0.05)
        .set("finish", "polished")
}

/// Knurled metal: a diamond (or `reeded`) bump pattern of `pitch` px round the part's axis.
#[must_use]
pub fn knurled(base: [f32; 3], pitch: f32, reeded: bool) -> Mat {
    Mat::new("metal", "knurled")
        .color(base)
        .rough(0.18)
        .set("finish", "polished")
        .set("knurl", pitch)
        .set("knurl_kind", if reeded { "reeded" } else { "diamond" })
}

/// Guilloché: `clous` (Clous de Paris pyramids), `rose` (rosette waves) or `wave` (barleycorn
/// waves), pitch in px.
#[must_use]
pub fn guilloche(base: [f32; 3], pattern: &str, pitch: f32) -> Mat {
    Mat::new("guilloche", "guilloche")
        .color(base)
        .rough(0.2)
        .set("pattern", pattern)
        .set("pitch", pitch)
        .set("depth", 1.0)
}

/// 2×2 twill carbon fibre with a deep clear coat; `tow` = tow width in px.
#[must_use]
pub fn carbon(tow: f32) -> Mat {
    Mat::new("carbon", "carbon")
        .color([0.018, 0.018, 0.02])
        .rough(0.32)
        .set("tow", tow)
}

/// Piano black: black dielectric under a mirror clear coat.
#[must_use]
pub fn piano_black() -> Mat {
    Mat::new("dielectric", "piano_black")
        .color([0.004, 0.004, 0.005])
        .rough(0.08)
        .set("coat", 1.0)
        .set("coat_rough", 0.0)
}

/// Sapphire crystal: IOR 1.77 with a 120 nm MgF₂-style anti-reflective film (the faint
/// blue-violet sheen on a watch crystal).
#[must_use]
pub fn sapphire() -> Mat {
    Mat::new("glass", "sapphire")
        .color([0.98, 0.99, 1.0])
        .rough(0.0)
        .set("ior", 1.77)
        .set("film_nm", 120.0)
        .set("film_ior", 1.38)
}

/// Plain glass (IOR 1.5), optionally tinted.
#[must_use]
pub fn glass(tint: [f32; 3]) -> Mat {
    Mat::new("glass", "glass").color(tint).rough(0.0).set("ior", 1.5)
}

/// Fired enamel: saturated colour, glassy coat, a little subsurface depth.
#[must_use]
pub fn enamel(color: [f32; 3]) -> Mat {
    Mat::new("dielectric", "enamel")
        .color(color)
        .rough(0.3)
        .set("coat", 1.0)
        .set("coat_rough", 0.04)
        .set("sss", 0.15)
}

/// Lume (Super-LumiNova style): matte paint that glows `glow` (0 = daylight look).
#[must_use]
pub fn lume(color: [f32; 3], glow: f32) -> Mat {
    Mat::new("dielectric", "lume")
        .color(color)
        .rough(0.55)
        .set("emit", json!(color))
        .set("emit_strength", glow)
}

/// An LED cap: clear PMMA lens over a coloured die; `glow` lights it.
#[must_use]
pub fn led_cap(color: [f32; 3], glow: f32) -> Mat {
    Mat::new("led", "led_cap")
        .color(color)
        .rough(0.04)
        .set("ior", 1.49)
        .set("emit", json!(color))
        .set("emit_strength", glow)
}

/// Leather with a Voronoi pebble grain of `grain` px. Give it `texture` (an image path, e.g. a
/// ComfyUI leather plate) to replace the procedural colour and relief.
#[must_use]
pub fn leather(color: [f32; 3], grain: f32) -> Mat {
    Mat::new("leather", "leather")
        .color(color)
        .rough(0.52)
        .set("grain", grain)
        .set("sheen", 0.35)
}

/// Alcantara / suede: very rough, strong sheen, fibre noise.
#[must_use]
pub fn alcantara(color: [f32; 3]) -> Mat {
    Mat::new("alcantara", "alcantara")
        .color(color)
        .rough(0.92)
        .set("sheen", 1.0)
}

/// Matte rubber or plastic.
#[must_use]
pub fn matte(color: [f32; 3]) -> Mat {
    Mat::new("dielectric", "matte").color(color).rough(0.75)
}

/// Pad-printed dial ink (numerals, ticks on a physical dial).
#[must_use]
pub fn ink(color: [f32; 3]) -> Mat {
    Mat::new("dielectric", "ink").color(color).rough(0.42)
}

/// Pure emission (no shading), e.g. a backlit graphic.
#[must_use]
pub fn emissive(color: [f32; 3], strength: f32) -> Mat {
    Mat::new("emission", "emissive")
        .color(color)
        .set("emit_strength", strength)
}

/// Every cookbook material with a sensible default, for the swatch sheet and tests.
#[must_use]
pub fn cookbook() -> Vec<Mat> {
    vec![
        machined_alu(),
        anodised([0.9, 0.12, 0.05]).named("anodised_red"),
        brushed(metal::TITANIUM).named("brushed_titanium"),
        sunray([0.05, 0.055, 0.06]).named("sunray_slate"),
        spun(metal::STEEL).named("spun_steel"),
        polished(metal::CHROME).named("polished_chrome"),
        knurled(metal::TITANIUM, 6.0, false).named("knurled_titanium"),
        guilloche([0.78, 0.76, 0.72], "clous", 10.0).named("guilloche_clous"),
        guilloche([0.20, 0.30, 0.55], "rose", 8.0).named("guilloche_rose"),
        carbon(8.0),
        piano_black(),
        sapphire(),
        glass([1.0, 1.0, 1.0]),
        enamel([0.03, 0.08, 0.25]).named("enamel_blue"),
        lume([0.62, 0.9, 0.55], 1.5).named("lume_green"),
        led_cap([1.0, 0.08, 0.02], 6.0).named("led_red"),
        leather([0.022, 0.018, 0.016], 3.0).named("leather_black"),
        leather([0.32, 0.12, 0.05], 3.0).named("leather_cognac"),
        alcantara([0.03, 0.03, 0.032]).named("alcantara_black"),
        matte([0.02, 0.02, 0.02]).named("matte_black"),
        ink([0.85, 0.85, 0.82]).named("ink_white"),
        polished(metal::BLACK_CHROME).named("black_chrome"),
        brushed(metal::GUNMETAL).named("gunmetal"),
        polished(metal::GOLD).named("polished_gold"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookbook_names_are_unique_and_kinds_known() {
        let book = cookbook();
        let mut names: Vec<&str> = book.iter().map(Mat::name).collect();
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate names");
        let kinds = [
            "metal",
            "guilloche",
            "carbon",
            "dielectric",
            "glass",
            "led",
            "leather",
            "alcantara",
            "emission",
        ];
        for m in &book {
            let k = m.v.get("kind").and_then(Value::as_str).unwrap_or("");
            assert!(kinds.contains(&k), "{} has unknown kind {k}", m.name());
        }
    }

    #[test]
    fn metals_use_measured_reflectance() {
        let m = polished(metal::GOLD);
        let c = m.to_json()["color"].as_array().cloned().unwrap_or_default();
        assert!((c[1].as_f64().unwrap_or(0.0) - 0.766).abs() < 1e-6);
    }
}
