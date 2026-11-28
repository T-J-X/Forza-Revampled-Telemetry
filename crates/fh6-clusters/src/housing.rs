//! A dash's housing: the slab, bezel or wheel face round its display, which the driver can turn
//! on or off per dash from the OSD (TJX, 2026-11-28: "have an option to turn the slab on and off
//! in the osd").
//!
//! The choice is stored in the config key `Housing`, a JSON object from dash name to on or off
//! (`"Housing": {"ValkyrieLmp1": false}`); a dash with no entry uses its default below. A dash
//! that draws a housing:
//!
//! 1. adds its default to [`default_for`], one match arm (`Some(true)` for on, `Some(false)` for
//!    off); the OSD shows its "Housing" row only for dashes listed there;
//! 2. reads [`shown`] in its `update` (`housing::shown(input.config, ClusterTheme::Mine)`), so a
//!    change applies on the next frame, and draws the housing only when it is true; with the
//!    housing off, everything else floats on OLED black;
//! 3. gives each translucent part that sits on the housing its opacity with
//!    [`crate::common::bezel::level_alpha`]`(level, shown)`, not `bezel::on_screen(level)`:
//!    `on_screen` is right only over an opaque body, and over the overlay's transparent pixels
//!    it shows about 2.5 times too dim.
//!
//! [`shown`] is a map lookup by name with no allocation, cheap enough for every frame.

use fh6_core::config::AppConfig;
use fh6_core::theme::ClusterTheme;

/// Whether `theme` has a housing, and if so whether it is on until the driver chooses: `None`
/// for a dash without one (the OSD hides its row).
#[must_use]
pub const fn default_for(theme: ClusterTheme) -> Option<bool> {
    match theme {
        // Every AMG MBUX dash floats on black by default (TJX, 2026-11-28: "it can be outside of
        // a slab ... this applies to all mbux"); the slab is the driver's option.
        ClusterTheme::AmgMbuxSuperscreen
        | ClusterTheme::AmgMbuxSupersport
        | ClusterTheme::AmgMbuxTrackPace
        | ClusterTheme::AmgMbuxClassic => Some(false),
        // The carbon wheel face round the screen.
        ClusterTheme::ValkyrieLmp1 => Some(true),
        _ => None,
    }
}

/// True when `theme` has a housing.
#[must_use]
pub const fn has_housing(theme: ClusterTheme) -> bool {
    default_for(theme).is_some()
}

/// Whether `theme` draws its housing under `cfg`: the driver's choice, else the dash's default;
/// always false for a dash without one.
#[must_use]
pub fn shown(cfg: &AppConfig, theme: ClusterTheme) -> bool {
    default_for(theme).is_some_and(|default| cfg.housing(theme).unwrap_or(default))
}

/// Turns `theme`'s housing on or off for `cfg`. Does nothing for a dash without one.
pub fn set(cfg: &mut AppConfig, theme: ClusterTheme, on: bool) {
    if has_housing(theme) {
        cfg.set_housing(theme, on);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mbux_dashes_default_off_and_the_choice_wins() {
        let mut cfg = AppConfig::default();
        for t in [
            ClusterTheme::AmgMbuxSuperscreen,
            ClusterTheme::AmgMbuxSupersport,
            ClusterTheme::AmgMbuxTrackPace,
            ClusterTheme::AmgMbuxClassic,
        ] {
            assert!(has_housing(t));
            assert!(!shown(&cfg, t), "{t:?} floats on black by default");
        }
        set(&mut cfg, ClusterTheme::AmgMbuxSuperscreen, true);
        assert!(shown(&cfg, ClusterTheme::AmgMbuxSuperscreen));
        assert!(!shown(&cfg, ClusterTheme::AmgMbuxClassic), "the choice is per dash");
    }

    #[test]
    fn a_default_on_housing_turns_off_and_a_dash_without_one_never_shows() {
        let mut cfg = AppConfig::default();
        assert!(shown(&cfg, ClusterTheme::ValkyrieLmp1));
        set(&mut cfg, ClusterTheme::ValkyrieLmp1, false);
        assert!(!shown(&cfg, ClusterTheme::ValkyrieLmp1));

        assert!(!has_housing(ClusterTheme::BoschDdu));
        set(&mut cfg, ClusterTheme::BoschDdu, true);
        assert!(!shown(&cfg, ClusterTheme::BoschDdu), "no housing to show");
        assert_eq!(cfg.housing(ClusterTheme::BoschDdu), None, "nothing is stored for it");
    }
}
