//! The seven design families a cluster belongs to. Each one names a tokens entry
//! (`families.<key>`) that holds its default palette and type.

/// The seven design families (`docs/design/families/`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// Liquid Glass (Design Council 3A).
    LiquidGlass,
    /// Modern Digital 2027 (Design Council 3B).
    ModernDigital,
    /// Classic Analog (Design Council 3B).
    ClassicAnalog,
    /// Race Dash (Design Council 3B).
    RaceDash,
    /// ADAS & HUD (Design Council 3D).
    AdasHud,
    /// Sci-Fi (Design Council 3C).
    SciFi,
    /// Fluid / Generative (Design Council 3C).
    FluidGenerative,
}

impl Family {
    /// The tokens key (`families.<key>`, `Palette::family`).
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Family::LiquidGlass => "liquidGlass",
            Family::ModernDigital => "modernDigital",
            Family::ClassicAnalog => "classicAnalog",
            Family::RaceDash => "raceDash",
            Family::AdasHud => "adasHud",
            Family::SciFi => "sciFi",
            Family::FluidGenerative => "fluidGenerative",
        }
    }

    /// The human name.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Family::LiquidGlass => "Liquid Glass",
            Family::ModernDigital => "Modern Digital 2027",
            Family::ClassicAnalog => "Classic Analog",
            Family::RaceDash => "Race Dash",
            Family::AdasHud => "ADAS & HUD",
            Family::SciFi => "Sci-Fi",
            Family::FluidGenerative => "Fluid / Generative",
        }
    }

    /// The family spec, relative to the repo root.
    #[must_use]
    pub const fn spec(self) -> &'static str {
        match self {
            Family::LiquidGlass => "docs/design/families/liquid-glass.md",
            Family::ModernDigital => "docs/design/families/modern-digital.md",
            Family::ClassicAnalog => "docs/design/families/classic-analog.md",
            Family::RaceDash => "docs/design/families/race-dash.md",
            Family::AdasHud => "docs/design/families/adas-hud.md",
            Family::SciFi => "docs/design/families/sci-fi.md",
            Family::FluidGenerative => "docs/design/families/fluid-generative.md",
        }
    }

    /// The family's hero frames, relative to the repo root.
    #[must_use]
    pub const fn heroes(self) -> &'static str {
        match self {
            Family::LiquidGlass => "docs/design/heroes/liquid-glass/",
            Family::ModernDigital => "docs/design/heroes/modern-digital/",
            Family::ClassicAnalog => "docs/design/heroes/classic-analog/",
            Family::RaceDash => "docs/design/heroes/race-dash/",
            Family::AdasHud => "docs/design/heroes/adas-hud/",
            Family::SciFi => "docs/design/heroes/sci-fi/",
            Family::FluidGenerative => "docs/design/heroes/fluid-generative/",
        }
    }
}
