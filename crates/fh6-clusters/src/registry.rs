//! The cluster registry (contract v2): one row per cluster, keyed by [`ClusterTheme`].
//!
//! This table is the single place a cluster registers itself. The Phase 7 setup chat generates
//! it (one row per cluster module); until a theme has a kit implementation, [`resolve`] falls
//! back to the family's sample so every config value still shows a working cluster.
use crate::kit::{AnalogSample, LiquidGlassSample};
use fh6_core::ClusterTheme;
use fh6_kit::cluster::{find_entry, ClusterEntry, Setup};
use fh6_kit::Cluster;

fn make_liquid_glass(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(LiquidGlassSample::new(setup))
}

fn make_analog(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(AnalogSample::new(setup))
}

/// Every cluster with a kit implementation.
pub static REGISTRY: &[ClusterEntry] = &[
    ClusterEntry {
        theme: ClusterTheme::LiquidGlass,
        name: "Liquid Glass (kit sample)",
        family: "liquidGlass",
        make: make_liquid_glass,
    },
    ClusterEntry {
        theme: ClusterTheme::PorscheAnalog,
        name: "Analog (kit sample)",
        family: "classicAnalog",
        make: make_analog,
    },
];

/// The registry row for `theme`, if it has a kit implementation.
#[must_use]
pub fn entry(theme: ClusterTheme) -> Option<&'static ClusterEntry> {
    find_entry(REGISTRY, theme)
}

/// The row to use for `theme`: its own, else the sample of its family, else the first row.
#[must_use]
pub fn resolve(theme: ClusterTheme) -> &'static ClusterEntry {
    if let Some(e) = entry(theme) {
        return e;
    }
    let analog_family = matches!(
        theme,
        ClusterTheme::PorscheAnalog
            | ClusterTheme::LfaV10Nurburgring
            | ClusterTheme::PaganiHorologerie
            | ClusterTheme::BugattiTourbillon
            | ClusterTheme::KoenigseggGyro
            | ClusterTheme::ChronoMultiverse
    );
    let fallback = if analog_family {
        ClusterTheme::PorscheAnalog
    } else {
        ClusterTheme::LiquidGlass
    };
    entry(fallback).unwrap_or(&REGISTRY[0])
}

/// Creates the cluster for `theme` (see [`resolve`]).
pub fn create(theme: ClusterTheme, setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    (resolve(theme).make)(setup)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_resolves() {
        for theme in ClusterTheme::all() {
            let e = resolve(*theme);
            assert!(REGISTRY.iter().any(|r| r.theme == e.theme), "{theme}");
        }
        assert_eq!(
            resolve(ClusterTheme::PaganiHorologerie).theme,
            ClusterTheme::PorscheAnalog
        );
        assert_eq!(resolve(ClusterTheme::LiquidGlass).theme, ClusterTheme::LiquidGlass);
    }

    #[test]
    fn themes_are_unique() {
        for (i, a) in REGISTRY.iter().enumerate() {
            assert!(
                REGISTRY[i + 1..].iter().all(|b| b.theme != a.theme),
                "{} twice",
                a.theme
            );
        }
    }
}
