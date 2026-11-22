//! Design tokens from `data/figma_tokens.json`, bound for the kit.
//!
//! The loader reads schema v3 keys (`colorRoles`, `paletteModes`, `motion`, `glow`, `glass`,
//! `type`, `layout`) when present and falls back, key by key, to the v3 values for anything
//! missing, so a v2 file still loads. No cluster hardcodes a colour, size or timing that has a
//! token: it reads it from here or from the [`crate::Palette`].
use crate::anim::{Easing, SpringSpec};
use crate::color::Color;
use crate::math::{vec2, Rect, Vec2};
use crate::paint::Glass;
use serde_json::Value;
use std::path::Path;

/// Errors loading tokens.
#[derive(Debug, thiserror::Error)]
pub enum TokensError {
    /// The file could not be read.
    #[error("reading {path}: {source}")]
    Io {
        /// The file.
        path: String,
        /// The cause.
        source: std::io::Error,
    },
    /// The file is not valid JSON.
    #[error("parsing tokens JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// Fixed and derived colour roles (tokens `colorRoles.roles`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Roles {
    /// Redline sector, brake, alarms. Never replaced by a brand colour.
    pub warning: Color,
    /// Caution stage, assists, slip.
    pub caution: Color,
    /// First shift stage, throttle.
    pub go: Color,
    /// Hero digits and primary text.
    pub ink: Color,
    /// Units, secondary values (alpha 158).
    pub ink_muted: Color,
    /// Micro-labels (alpha 120).
    pub ink_label: Color,
    /// Ghost 8s, unlit segments and LEDs (alpha 16).
    pub ink_ghost: Color,
    /// 1 px tracks and dividers (alpha 46).
    pub hairline: Color,
    /// Top of every 1 px specular rim (alpha 216).
    pub rim_specular: Color,
    /// Off pixels, VFD cavities, the scrim core.
    pub oled_black: Color,
}

impl Default for Roles {
    fn default() -> Self {
        let ink = Color::rgb_u32(0xF4F7FB);
        Self {
            warning: Color::rgb_u32(0xFF2D3A),
            caution: Color::rgb_u32(0xFFB020),
            go: Color::rgb_u32(0x2BE37A),
            ink,
            ink_muted: ink.with_alpha8(158),
            ink_label: ink.with_alpha8(120),
            ink_ghost: ink.with_alpha8(16),
            hairline: Color::WHITE.with_alpha8(46),
            rim_specular: Color::WHITE.with_alpha8(216),
            oled_black: Color::BLACK,
        }
    }
}

/// Named springs (tokens `motion.springs`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Springs {
    /// Live needle (with feed-forward).
    pub needle: SpringSpec,
    /// Bar and ribbon fills (with feed-forward).
    pub ribbon: SpringSpec,
    /// Small state snaps.
    pub snap: SpringSpec,
    /// Incoming gear glyph.
    pub fast: SpringSpec,
    /// Panels.
    pub default: SpringSpec,
    /// Slow panels.
    pub slow: SpringSpec,
    /// Expressive moves.
    pub expressive: SpringSpec,
    /// Virtual light azimuth.
    pub light: SpringSpec,
    /// G-ball.
    pub g_ball: SpringSpec,
    /// Liquid surface tilt (deliberately under-damped).
    pub fluid_slosh: SpringSpec,
}

impl Default for Springs {
    fn default() -> Self {
        Self {
            needle: SpringSpec::new(0.070, 0.75),
            ribbon: SpringSpec::new(0.070, 0.90),
            snap: SpringSpec::new(0.102, 1.0),
            fast: SpringSpec::new(0.168, 0.9),
            default: SpringSpec::new(0.237, 0.9),
            slow: SpringSpec::new(0.363, 0.9),
            expressive: SpringSpec::new(0.322, 0.8),
            light: SpringSpec::new(0.150, 1.0),
            g_ball: SpringSpec::new(0.168, 0.9),
            fluid_slosh: SpringSpec::new(0.62, 0.35),
        }
    }
}

/// Baked easing curves (tokens `motion.easing`).
#[derive(Debug, Clone, PartialEq)]
pub struct Easings {
    /// M3 emphasized.
    pub emphasized: Easing,
    /// M3 emphasized decelerate (entries).
    pub emphasized_decelerate: Easing,
    /// M3 standard accelerate (exits).
    pub standard_accelerate: Easing,
}

impl Default for Easings {
    fn default() -> Self {
        Self {
            emphasized: Easing::cubic_bezier(0.2, 0.0, 0.0, 1.0),
            emphasized_decelerate: Easing::cubic_bezier(0.05, 0.7, 0.1, 1.0),
            standard_accelerate: Easing::cubic_bezier(0.3, 0.0, 1.0, 1.0),
        }
    }
}

/// Named durations in seconds (tokens `motion.durations`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Durations {
    /// 50 ms.
    pub micro: f32,
    /// 100 ms.
    pub short: f32,
    /// 150 ms.
    pub crossfade: f32,
    /// 200 ms (glass materialise, show).
    pub materialize: f32,
    /// 300 ms.
    pub medium: f32,
    /// 500 ms.
    pub long: f32,
}

impl Default for Durations {
    fn default() -> Self {
        Self {
            micro: 0.05,
            short: 0.10,
            crossfade: 0.15,
            materialize: 0.20,
            medium: 0.30,
            long: 0.50,
        }
    }
}

/// One Gaussian lobe of a glow level.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GlowLobe {
    /// Sigma in 4K px.
    pub sigma: f32,
    /// Intensity.
    pub intensity: f32,
}

/// A glow level: up to three lobes (tokens `glow.<level>.lobes`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GlowLevel {
    lobes: [GlowLobe; 3],
    count: usize,
}

impl GlowLevel {
    fn from_lobes(lobes: &[(f32, f32)]) -> Self {
        let mut l = Self::default();
        for (i, (sigma, intensity)) in lobes.iter().take(3).enumerate() {
            l.lobes[i] = GlowLobe {
                sigma: *sigma,
                intensity: *intensity,
            };
            l.count = i + 1;
        }
        l
    }

    /// The lobes, tight core first.
    #[must_use]
    pub fn lobes(&self) -> &[GlowLobe] {
        &self.lobes[..self.count]
    }
}

/// Glow levels and the bloom chain (tokens `glow`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlowTokens {
    /// Soft.
    pub soft: GlowLevel,
    /// Medium (two lobes).
    pub medium: GlowLevel,
    /// Hot (three lobes).
    pub hot: GlowLevel,
    /// VFD phosphor.
    pub phosphor: GlowLevel,
    /// Bloom threshold.
    pub bloom_threshold: f32,
    /// Bloom intensity.
    pub bloom_intensity: f32,
    /// Largest bloom radius in 4K px (budget).
    pub max_bloom_radius: f32,
}

impl Default for GlowTokens {
    fn default() -> Self {
        Self {
            soft: GlowLevel::from_lobes(&[(6.0, 0.35)]),
            medium: GlowLevel::from_lobes(&[(5.0, 0.45), (18.0, 0.3)]),
            hot: GlowLevel::from_lobes(&[(4.0, 0.6), (16.0, 0.45), (48.0, 0.25)]),
            phosphor: GlowLevel::from_lobes(&[(2.0, 0.7), (10.0, 0.4)]),
            bloom_threshold: 0.7,
            bloom_intensity: 0.8,
            max_bloom_radius: 96.0,
        }
    }
}

/// The Liquid Glass tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassTokens {
    /// Tint alpha: minimum, default, maximum (0..=255).
    pub tint_alpha: [u8; 3],
    /// Frost noise scale in px.
    pub frost_noise_px: f32,
    /// Sheen peak alpha (0..=255).
    pub sheen_alpha: u8,
    /// Sheen height fraction.
    pub sheen_height: f32,
    /// Sheen sag in px.
    pub sheen_sag: f32,
    /// Caustic strength.
    pub caustic: f32,
    /// Inner rim alpha (0..=255).
    pub inner_rim_alpha: u8,
}

impl Default for GlassTokens {
    fn default() -> Self {
        Self {
            tint_alpha: [28, 44, 64],
            frost_noise_px: 1.4,
            sheen_alpha: 76,
            sheen_height: 0.18,
            sheen_sag: 44.0,
            caustic: 0.32,
            inner_rim_alpha: 40,
        }
    }
}

/// The type scale: cap heights in 4K px at scale 1.0 (tokens `type.scale`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeScale {
    /// Hero gear.
    pub gear_hero: f32,
    /// Compact gear.
    pub gear_compact: f32,
    /// Hero speed.
    pub speed_hero: f32,
    /// Compact speed.
    pub speed_compact: f32,
    /// Primary metric.
    pub metric_primary: f32,
    /// Secondary metric.
    pub metric_secondary: f32,
    /// Dial numeral.
    pub dial_numeral: f32,
    /// Unit.
    pub unit: f32,
    /// Micro-label.
    pub micro_label: f32,
    /// Smallest allowed.
    pub nano: f32,
    /// Micro-label tracking in em.
    pub label_tracking: f32,
}

impl Default for TypeScale {
    fn default() -> Self {
        Self {
            gear_hero: 200.0,
            gear_compact: 150.0,
            speed_hero: 112.0,
            speed_compact: 84.0,
            metric_primary: 44.0,
            metric_secondary: 30.0,
            dial_numeral: 34.0,
            unit: 20.0,
            micro_label: 16.0,
            nano: 12.0,
            label_tracking: 0.16,
        }
    }
}

/// Rev-stage thresholds as fractions of the shift rpm (tokens `colorRoles.revStages`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RevStages {
    /// Stage 1 (go) start.
    pub stage1: f32,
    /// Stage 2 (caution) start.
    pub stage2: f32,
    /// Stage 3 (warning) start.
    pub stage3: f32,
    /// Shift rpm as a fraction of the limiter (default 0.97).
    pub shift_pct: f32,
}

impl Default for RevStages {
    fn default() -> Self {
        Self {
            stage1: 0.75,
            stage2: 0.85,
            stage3: 0.93,
            shift_pct: 0.97,
        }
    }
}

/// View anchors and envelopes in 4K px (tokens `layout`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutTokens {
    /// Cockpit `[3]` anchor.
    pub cockpit_anchor: Vec2,
    /// Cockpit envelope relative to the anchor.
    pub cockpit_envelope: Rect,
    /// Hood `[4]` anchor.
    pub hood_anchor: Vec2,
    /// Hood envelope.
    pub hood_envelope: Rect,
    /// Raised HUD anchor.
    pub raised_anchor: Vec2,
}

impl Default for LayoutTokens {
    fn default() -> Self {
        Self {
            cockpit_anchor: vec2(1920.0, 1835.0),
            cockpit_envelope: Rect::from_xywh(-720.0, -320.0, 1440.0, 560.0),
            hood_anchor: vec2(1920.0, 1760.0),
            hood_envelope: Rect::from_xywh(-880.0, -220.0, 1760.0, 400.0),
            raised_anchor: vec2(1920.0, 1595.0),
        }
    }
}

/// A family default palette (tokens `paletteModes.familyDefaults`).
#[derive(Debug, Clone, PartialEq)]
pub struct FamilyPalette {
    /// Family key, e.g. "liquidGlass".
    pub family: String,
    /// Primary, secondary, accent, shift.
    pub colors: [Color; 4],
}

/// The bound token set.
#[derive(Debug, Clone, PartialEq)]
pub struct Tokens {
    /// Colour roles.
    pub roles: Roles,
    /// Rev stages.
    pub rev_stages: RevStages,
    /// Springs.
    pub springs: Springs,
    /// Easings.
    pub easing: Easings,
    /// Durations.
    pub durations: Durations,
    /// Glow and bloom.
    pub glow: GlowTokens,
    /// Glass.
    pub glass: GlassTokens,
    /// Type scale.
    pub type_scale: TypeScale,
    /// Layout.
    pub layout: LayoutTokens,
    /// Family default palettes.
    pub family_defaults: Vec<FamilyPalette>,
    /// Schema version string from `meta.version`.
    pub version: String,
}

impl Default for Tokens {
    fn default() -> Self {
        let fam = |name: &str, p: u32, s: u32, a: u32, sh: u32| FamilyPalette {
            family: name.into(),
            colors: [
                Color::rgb_u32(p),
                Color::rgb_u32(s),
                Color::rgb_u32(a),
                Color::rgb_u32(sh),
            ],
        };
        Self {
            roles: Roles::default(),
            rev_stages: RevStages::default(),
            springs: Springs::default(),
            easing: Easings::default(),
            durations: Durations::default(),
            glow: GlowTokens::default(),
            glass: GlassTokens::default(),
            type_scale: TypeScale::default(),
            layout: LayoutTokens::default(),
            family_defaults: vec![
                fam("classicAnalog", 0xFF7A1A, 0xC9CED6, 0xFFC21A, 0xFFFFFF),
                fam("modernDigital", 0x00D2FF, 0x8A96A8, 0xFFFFFF, 0x978DFC),
                fam("liquidGlass", 0x38BDF8, 0xC7E9FF, 0xFFFFFF, 0x978DFC),
                fam("sciFi", 0x00F5D4, 0xFEE440, 0xFF2E97, 0xFF2E97),
                fam("fluidGenerative", 0x9AD8FF, 0xB8C2CF, 0xFFFFFF, 0xFF5A1F),
                fam("adasHud", 0x3B82F6, 0x94A3B8, 0x38BDF8, 0x978DFC),
                fam("raceDash", 0x00B4FF, 0x9AA3AE, 0xFFD60A, 0xB27CFF),
            ],
            version: "3 (built-in defaults)".into(),
        }
    }
}

fn get<'a>(v: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(v, |cur, key| cur.get(key))
}

fn num(v: &Value, path: &[&str]) -> Option<f32> {
    get(v, path).and_then(Value::as_f64).map(|x| x as f32)
}

fn color_role(v: &Value, name: &str) -> Option<Color> {
    let role = get(v, &["colorRoles", "roles", name])?;
    let hex = role.get("hex")?.as_str()?;
    let c = Color::from_hex(hex).ok()?;
    Some(match role.get("alpha").and_then(Value::as_u64) {
        Some(a) => c.with_alpha8(u8::try_from(a).unwrap_or(255)),
        None => c,
    })
}

impl Tokens {
    /// Loads and binds `path`.
    ///
    /// # Errors
    /// [`TokensError`] when the file cannot be read or is not JSON. Missing keys are not errors.
    pub fn load(path: &Path) -> Result<Self, TokensError> {
        let text = std::fs::read_to_string(path).map_err(|source| TokensError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_json(&text)
    }

    /// Binds tokens from JSON text (a leading BOM is tolerated).
    ///
    /// # Errors
    /// [`TokensError::Json`] when the text is not JSON.
    pub fn from_json(text: &str) -> Result<Self, TokensError> {
        let v: Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))?;
        Ok(Self::from_value(&v))
    }

    /// Binds tokens from a parsed document; every missing key keeps its default.
    #[must_use]
    pub fn from_value(v: &Value) -> Self {
        let mut t = Self::default();
        if let Some(ver) = get(v, &["meta", "version"]).and_then(Value::as_str) {
            t.version = ver.to_string();
        }
        let r = &mut t.roles;
        for (name, slot) in [
            ("warning", &mut r.warning),
            ("caution", &mut r.caution),
            ("go", &mut r.go),
            ("ink", &mut r.ink),
            ("inkMuted", &mut r.ink_muted),
            ("inkLabel", &mut r.ink_label),
            ("inkGhost", &mut r.ink_ghost),
            ("hairline", &mut r.hairline),
            ("rimSpecular", &mut r.rim_specular),
            ("oledBlack", &mut r.oled_black),
        ] {
            if let Some(c) = color_role(v, name) {
                *slot = c;
            }
        }
        let rs = &mut t.rev_stages;
        for (key, slot) in [
            ("stage1", &mut rs.stage1),
            ("stage2", &mut rs.stage2),
            ("stage3", &mut rs.stage3),
        ] {
            if let Some(x) = num(v, &["colorRoles", "revStages", key, "from"]) {
                *slot = x;
            }
        }
        if let Some(x) = num(v, &["colorRoles", "revStages", "shiftPct"]) {
            rs.shift_pct = x;
        }
        let sp = &mut t.springs;
        for (key, slot) in [
            ("needle", &mut sp.needle),
            ("ribbon", &mut sp.ribbon),
            ("snap", &mut sp.snap),
            ("fast", &mut sp.fast),
            ("default", &mut sp.default),
            ("slow", &mut sp.slow),
            ("expressive", &mut sp.expressive),
            ("light", &mut sp.light),
            ("gBall", &mut sp.g_ball),
            ("fluidSlosh", &mut sp.fluid_slosh),
        ] {
            if let (Some(period), Some(zeta)) = (
                num(v, &["motion", "springs", key, "T"]),
                num(v, &["motion", "springs", key, "zeta"]),
            ) {
                *slot = SpringSpec::new(period, zeta);
            }
        }
        for (key, slot) in [
            ("emphasized", &mut t.easing.emphasized),
            ("emphasizedDecelerate", &mut t.easing.emphasized_decelerate),
            ("standardAccelerate", &mut t.easing.standard_accelerate),
        ] {
            if let Some(e) = get(v, &["motion", "easing", key])
                .and_then(Value::as_str)
                .and_then(Easing::parse)
            {
                *slot = e;
            }
        }
        let d = &mut t.durations;
        for (key, slot) in [
            ("micro", &mut d.micro),
            ("short", &mut d.short),
            ("crossfade", &mut d.crossfade),
            ("materialize", &mut d.materialize),
            ("medium", &mut d.medium),
            ("long", &mut d.long),
        ] {
            if let Some(ms) = num(v, &["motion", "durations", key]) {
                *slot = ms / 1000.0;
            }
        }
        let g = &mut t.glow;
        for (key, slot) in [
            ("soft", &mut g.soft),
            ("medium", &mut g.medium),
            ("hot", &mut g.hot),
            ("phosphor", &mut g.phosphor),
        ] {
            if let Some(lobes) = get(v, &["glow", key, "lobes"]).and_then(Value::as_array) {
                let pairs: Vec<(f32, f32)> = lobes
                    .iter()
                    .filter_map(|l| Some((num(l, &["sigma"])?, num(l, &["intensity"])?)))
                    .collect();
                if !pairs.is_empty() {
                    *slot = GlowLevel::from_lobes(&pairs);
                }
            }
        }
        if let Some(x) = num(v, &["glow", "bloomChain", "threshold"]) {
            g.bloom_threshold = x;
        }
        if let Some(x) = num(v, &["glow", "bloomChain", "intensity"]) {
            g.bloom_intensity = x;
        }
        if let Some(x) = num(v, &["glow", "maxBloomRadiusPx"]) {
            g.max_bloom_radius = x;
        }
        let gl = &mut t.glass;
        for (i, key) in ["min", "default", "max"].iter().enumerate() {
            if let Some(a) = num(v, &["glass", "tintAlpha", key]) {
                gl.tint_alpha[i] = a.clamp(0.0, 255.0) as u8;
            }
        }
        if let Some(x) = num(v, &["glass", "frost", "noiseScalePx"]) {
            gl.frost_noise_px = x;
        }
        let ts = &mut t.type_scale;
        for (key, slot) in [
            ("gearHero", &mut ts.gear_hero),
            ("gearCompact", &mut ts.gear_compact),
            ("speedHero", &mut ts.speed_hero),
            ("speedCompact", &mut ts.speed_compact),
            ("metricPrimary", &mut ts.metric_primary),
            ("metricSecondary", &mut ts.metric_secondary),
            ("dialNumeral", &mut ts.dial_numeral),
            ("unit", &mut ts.unit),
            ("microLabel", &mut ts.micro_label),
            ("nano", &mut ts.nano),
        ] {
            if let Some(x) = num(v, &["type", "scale", key]).or_else(|| num(v, &["type", "scale", key, "cap"])) {
                *slot = x;
            }
        }
        if let Some(fams) = get(v, &["paletteModes", "familyDefaults"]).and_then(Value::as_object) {
            let parsed: Vec<FamilyPalette> = fams
                .iter()
                .filter_map(|(name, p)| {
                    let c = |k: &str| p.get(k).and_then(Value::as_str).and_then(|h| Color::from_hex(h).ok());
                    Some(FamilyPalette {
                        family: name.clone(),
                        colors: [c("primary")?, c("secondary")?, c("accent")?, c("shift")?],
                    })
                })
                .collect();
            if !parsed.is_empty() {
                t.family_defaults = parsed;
            }
        }
        t
    }

    /// The Liquid Glass material for `tint` at the default tint alpha.
    #[must_use]
    pub fn glass(&self, tint: Color) -> Glass {
        Glass {
            tint: tint.with_alpha8(self.glass.tint_alpha[1]),
            frost_noise: self.glass.frost_noise_px,
            sheen: f32::from(self.glass.sheen_alpha) / 255.0,
            sheen_height: self.glass.sheen_height,
            sheen_sag: self.glass.sheen_sag,
            caustic_strength: self.glass.caustic,
            inner_rim: Color::WHITE.with_alpha8(self.glass.inner_rim_alpha),
            ..Glass::default()
        }
    }

    /// The family default palette for `family` (e.g. "liquidGlass").
    #[must_use]
    pub fn family(&self, family: &str) -> Option<&FamilyPalette> {
        self.family_defaults
            .iter()
            .find(|f| f.family.eq_ignore_ascii_case(family))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_file_falls_back_to_defaults() {
        let t = Tokens::from_json(r#"{"meta":{"version":"2.0.0"},"typography":{}}"#).expect("json");
        assert_eq!(t.version, "2.0.0");
        assert_eq!(t.roles, Roles::default());
        assert_eq!(t.springs, Springs::default());
    }

    #[test]
    fn v3_keys_override() {
        let json = r##"{
          "colorRoles": {"roles": {"warning": {"hex": "#FF0000"}, "inkMuted": {"hex": "#FFFFFF", "alpha": 100}},
                          "revStages": {"stage1": {"from": 0.7}}},
          "motion": {"springs": {"needle": {"T": 0.05, "zeta": 0.8}},
                     "easing": {"emphasized": "cubic-bezier(0, 0, 1, 1)"},
                     "durations": {"micro": 40}},
          "glow": {"hot": {"lobes": [{"sigma": 3, "intensity": 1}]}, "bloomChain": {"threshold": 0.5}},
          "paletteModes": {"familyDefaults": {"x": {"primary": "#010203", "secondary": "#000000", "accent": "#000000", "shift": "#FFFFFF"}}}
        }"##;
        let t = Tokens::from_json(json).expect("json");
        assert_eq!(t.roles.warning.to_string(), "#FF0000");
        assert_eq!(t.roles.ink_muted.to_srgb8()[3], 100);
        assert!((t.rev_stages.stage1 - 0.7).abs() < 1e-6);
        assert_eq!(t.springs.needle, SpringSpec::new(0.05, 0.8));
        assert!((t.easing.emphasized.eval(0.3) - 0.3).abs() < 0.02);
        assert!((t.durations.micro - 0.04).abs() < 1e-6);
        assert_eq!(t.glow.hot.lobes().len(), 1);
        assert!((t.glow.bloom_threshold - 0.5).abs() < 1e-6);
        assert_eq!(t.family("x").map(|f| f.colors[0].to_string()), Some("#010203".into()));
    }

    #[test]
    fn repo_tokens_file_loads() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/figma_tokens.json");
        let t = Tokens::load(&path).expect("data/figma_tokens.json loads");
        assert!(!t.version.is_empty());
        assert!(t.family_defaults.len() >= 7);
    }
}
