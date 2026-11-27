//! The palette engine: factory (brand colours by car ordinal), custom (user hex values) and
//! random (seeded, harmonious, OLED-safe), mapped onto the token colour roles.
//!
//! Clusters never use a hex value: they read `palette.primary`, `palette.roles.warning`, and so
//! on. Text and thin lines use the text-safe variants (`*_text`), lifted to >= 7:1 on black.
use crate::color::{Color, Oklab, Oklch};
use crate::tokens::{Roles, Tokens};
use fh6_core::cars::DEFAULT_BRAND_KEY;
use fh6_core::{AppConfig, BrandPalette, CarDatabase, ColorMode};

/// Where the palette came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteSource {
    /// Brand colours of the current car.
    Factory,
    /// User-chosen colours.
    Custom,
    /// A seeded harmonious roll.
    Random(u64),
    /// The cluster family's default palette.
    Family,
}

/// The four brand roles plus the fixed and derived roles, ready to draw with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// Hero accents, tach fill, needle, ego-car tint.
    pub primary: Color,
    /// Trim, logo-zone ring, secondary arcs.
    pub secondary: Color,
    /// Leading edges, meniscus orb, active segment.
    pub accent: Color,
    /// Limiter strobe and the final shift stage.
    pub shift: Color,
    /// `primary` lifted for text and thin lines on black.
    pub primary_text: Color,
    /// `accent` lifted for text and thin lines on black.
    pub accent_text: Color,
    /// `mix(primary, #0A1220, 0.72)`, opaque (apply the glass tint alpha when used).
    pub glass_tint: Color,
    /// Fixed roles from the tokens.
    pub roles: Roles,
    /// Rev-stage starts as fractions of the shift rpm: go, caution, warning.
    pub stages: [f32; 3],
    /// Where the palette came from.
    pub source: PaletteSource,
}

impl Palette {
    /// A palette from the four brand roles.
    #[must_use]
    pub fn new(tokens: &Tokens, colors: [Color; 4], source: PaletteSource) -> Self {
        let [primary, secondary, accent, shift] = colors.map(|c| c.with_alpha(1.0));
        let glass_tint = primary.lerp(Color::rgb_u32(0x0A1220), 0.72).with_alpha(1.0);
        Self {
            primary,
            secondary,
            accent,
            shift,
            primary_text: primary.text_safe(),
            accent_text: accent.text_safe(),
            glass_tint,
            roles: tokens.roles,
            stages: [
                tokens.rev_stages.stage1,
                tokens.rev_stages.stage2,
                tokens.rev_stages.stage3,
            ],
            source,
        }
    }

    /// The family default palette (tokens `paletteModes.familyDefaults`), or the modern digital
    /// one for an unknown family.
    #[must_use]
    pub fn family(tokens: &Tokens, family: &str) -> Self {
        let colors = tokens
            .family(family)
            .or_else(|| tokens.family("modernDigital"))
            .map_or([Color::WHITE; 4], |f| f.colors);
        Self::new(tokens, colors, PaletteSource::Family)
    }

    /// A factory palette from a brand entry of `car_brands.json`. Unparseable hex values fall
    /// back to the family default role by role.
    #[must_use]
    pub fn factory(tokens: &Tokens, brand: &BrandPalette, family: &str) -> Self {
        let base = Self::family(tokens, family);
        let pick = |hex: &str, fallback: Color| Color::from_hex_or(hex, fallback);
        Self::new(
            tokens,
            [
                pick(&brand.primary_hex, base.primary),
                pick(&brand.secondary_hex, base.secondary),
                pick(&brand.accent_hex, base.accent),
                pick(&brand.shift_flash_hex, base.shift),
            ],
            PaletteSource::Factory,
        )
    }

    /// The factory palette for car `ordinal`: its brand, then `DEFAULT`, then the family.
    #[must_use]
    pub fn for_car(tokens: &Tokens, cars: &CarDatabase, ordinal: i32, family: &str) -> Self {
        cars.car_palette(ordinal)
            .or_else(|| cars.brand_palette(DEFAULT_BRAND_KEY))
            .map_or_else(|| Self::family(tokens, family), |b| Self::factory(tokens, b, family))
    }

    /// A custom palette. Without a secondary, it is the primary rotated -30 degrees at
    /// lightness - 0.12 (tokens `paletteModes.custom`).
    #[must_use]
    pub fn custom(tokens: &Tokens, primary: Color, accent: Color, shift: Color, secondary: Option<Color>) -> Self {
        let secondary = secondary.unwrap_or_else(|| {
            let p = primary.to_oklch();
            Color::from_oklch(Oklch::new(p.l - 0.12, p.c, p.h - 30.0))
        });
        Self::new(tokens, [primary, secondary, accent, shift], PaletteSource::Custom)
    }

    /// A seeded random palette in OKLCH (tokens `paletteModes.random`): primary L 0.66..0.80,
    /// C 0.14..0.24, never within 25 degrees of the warning hue; accent at +30/-30/+150/180
    /// degrees; secondary the low-chroma complement; shift the most distinct of white, accent,
    /// `#7C5CFF` and `#00E5FF` from the warning colour, at least 60 degrees from the primary.
    #[must_use]
    pub fn random(tokens: &Tokens, seed: u64) -> Self {
        let mut rng = SplitMix64(seed);
        let warning_hue = tokens.roles.warning.to_oklch().h;
        let mut hue = rng.range(0.0, 360.0);
        for _ in 0..64 {
            if Oklch::hue_distance(hue, warning_hue) >= 25.0 {
                break;
            }
            hue = rng.range(0.0, 360.0);
        }
        if Oklch::hue_distance(hue, warning_hue) < 25.0 {
            hue = (warning_hue + 180.0).rem_euclid(360.0);
        }
        let primary = Color::from_oklch(Oklch::new(rng.range(0.66, 0.80), rng.range(0.14, 0.24), hue));
        let offsets = [30.0, -30.0, 150.0, 180.0];
        let accent_h = hue + offsets[(rng.next() % 4) as usize];
        let accent = Color::from_oklch(Oklch::new(rng.range(0.78, 0.90), rng.range(0.12, 0.20), accent_h));
        let secondary = Color::from_oklch(Oklch::new(rng.range(0.55, 0.70), rng.range(0.04, 0.10), hue + 180.0));
        let candidates = [Color::WHITE, accent, Color::rgb_u32(0x7C5CFF), Color::rgb_u32(0x00E5FF)];
        let warn_lab = lab(tokens.roles.warning);
        let shift = candidates
            .into_iter()
            .filter(|c| {
                let lc = c.to_oklch();
                lc.c < 0.03 || Oklch::hue_distance(lc.h, hue) >= 60.0
            })
            .max_by(|a, b| distance(lab(*a), warn_lab).total_cmp(&distance(lab(*b), warn_lab)))
            .unwrap_or(Color::WHITE);
        Self::new(tokens, [primary, secondary, accent, shift], PaletteSource::Random(seed))
    }

    /// The palette `config` asks for: [`ColorMode::Custom`] uses the hex values,
    /// [`ColorMode::Random`] a seed (`PaletteSeed`, else the car ordinal), [`ColorMode::Brand`]
    /// the palette of `CustomBrandKey`, and [`ColorMode::AutoBrand`] (or `Brand` with the
    /// `DEFAULT` key) the factory palette for `car_ordinal`.
    #[must_use]
    pub fn from_config(
        tokens: &Tokens,
        config: &AppConfig,
        cars: &CarDatabase,
        car_ordinal: i32,
        family: &str,
    ) -> Self {
        match config.color_mode {
            ColorMode::Custom => {
                let base = Self::family(tokens, family);
                let primary = Color::from_hex_or(&config.primary_hex, base.primary);
                let accent = Color::from_hex_or(&config.accent_hex, base.accent);
                let shift = Color::from_hex_or(&config.shift_flash_hex, base.shift);
                let secondary = Color::from_hex(&config.secondary_hex).ok();
                Self::custom(tokens, primary, accent, shift, secondary)
            }
            ColorMode::Random => {
                let seed = config.palette_seed.unwrap_or(car_ordinal.unsigned_abs().into());
                Self::random(tokens, seed)
            }
            ColorMode::Brand if !config.custom_brand_key.trim().eq_ignore_ascii_case(DEFAULT_BRAND_KEY) => cars
                .brand_palette(&config.custom_brand_key)
                .map_or_else(|| Self::family(tokens, family), |b| Self::factory(tokens, b, family)),
            ColorMode::Brand | ColorMode::AutoBrand => Self::for_car(tokens, cars, car_ordinal, family),
        }
    }

    /// The colour of a lit shift light at `fraction` of the shift rpm (tokens rev stages:
    /// go, caution, warning, then the palette's shift colour at 1.0).
    #[must_use]
    pub fn rev_stage(&self, fraction: f32) -> Color {
        if fraction >= 1.0 {
            self.shift
        } else if fraction >= self.stages[2] {
            self.roles.warning
        } else if fraction >= self.stages[1] {
            self.roles.caution
        } else {
            self.roles.go
        }
    }
}

fn lab(c: Color) -> Oklab {
    let (r, g, b) = c.straight_rgb();
    Oklab::from_linear_rgb(r, g, b)
}

fn distance(a: Oklab, b: Oklab) -> f32 {
    ((a.l - b.l).powi(2) + (a.a - b.a).powi(2) + (a.b - b.b).powi(2)).sqrt()
}

/// SplitMix64: a tiny deterministic generator for palette rolls.
#[derive(Debug, Clone, Copy)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        let u = (self.next() >> 40) as f32 / (1u64 << 24) as f32;
        lo + (hi - lo) * u
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens() -> Tokens {
        Tokens::default()
    }

    #[test]
    fn random_is_seeded_and_harmonious() {
        let t = tokens();
        let a = Palette::random(&t, 42);
        let b = Palette::random(&t, 42);
        assert_eq!(a, b, "same seed, same palette");
        let warn = t.roles.warning.to_oklch().h;
        for seed in 0..500 {
            let p = Palette::random(&t, seed);
            let pc = p.primary.to_oklch();
            assert!(
                Oklch::hue_distance(pc.h, warn) >= 24.5,
                "seed {seed}: primary too close to warning"
            );
            assert!(
                (0.6..=0.85).contains(&pc.l),
                "seed {seed}: OLED-safe lightness {}",
                pc.l
            );
            let sc = p.shift.to_oklch();
            assert!(
                sc.c < 0.03 || Oklch::hue_distance(sc.h, pc.h) >= 59.0,
                "seed {seed}: shift distinct"
            );
            assert!(p.primary_text.contrast_on_black() >= 7.0, "seed {seed}");
        }
    }

    #[test]
    fn factory_parses_brand_and_falls_back() {
        let t = tokens();
        let brand = BrandPalette {
            display_name: "Test".into(),
            primary_hex: "#D40000".into(),
            secondary_hex: "not a colour".into(),
            accent_hex: "#FF2800".into(),
            shift_flash_hex: "#FFFFFF".into(),
        };
        let p = Palette::factory(&t, &brand, "liquidGlass");
        assert_eq!(p.primary.to_string(), "#D40000");
        assert_eq!(p.secondary, Palette::family(&t, "liquidGlass").secondary);
        assert_eq!(p.source, PaletteSource::Factory);
    }

    #[test]
    fn custom_derives_secondary() {
        let t = tokens();
        let p = Palette::custom(&t, Color::rgb_u32(0x38BDF8), Color::WHITE, Color::WHITE, None);
        let (pl, sl) = (p.primary.to_oklch(), p.secondary.to_oklch());
        assert!((pl.l - sl.l - 0.12).abs() < 0.03);
        assert!((Oklch::hue_distance(pl.h, sl.h) - 30.0).abs() < 3.0);
    }

    #[test]
    fn rev_stages_follow_tokens() {
        let p = Palette::family(&tokens(), "raceDash");
        assert_eq!(p.rev_stage(0.8), p.roles.go);
        assert_eq!(p.rev_stage(0.9), p.roles.caution);
        assert_eq!(p.rev_stage(0.95), p.roles.warning);
        assert_eq!(p.rev_stage(1.0), p.shift);
    }

    #[test]
    fn repo_car_brands_load_by_ordinal() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/car_brands/car_brands.json");
        let cars = CarDatabase::load(&path).expect("assets/car_brands/car_brands.json loads");
        let t = tokens();
        // 3595 is in the shipped database; 1 is not, and falls back to the DEFAULT brand.
        assert!(cars.car(3595).is_some() && cars.car(1).is_none());
        for ordinal in [3595, 1] {
            let p = Palette::for_car(&t, &cars, ordinal, "modernDigital");
            assert_eq!(p.source, PaletteSource::Factory, "car {ordinal}");
        }
    }
}
