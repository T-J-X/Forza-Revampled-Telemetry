//! Registry of 18 cluster renderers (stubs).
use crate::renderer::StubRenderer;
use crate::ClusterRenderer;
use fh6_core::ClusterTheme;

pub struct Registry {
    renderers: Vec<Box<dyn ClusterRenderer>>,
}

impl Registry {
    pub fn new() -> Self {
        let renderers: Vec<Box<dyn ClusterRenderer>> = vec![
            Box::new(StubRenderer::new(ClusterTheme::BoschDdu, "BoschDdu")),
            Box::new(StubRenderer::new(ClusterTheme::DriftGlass, "DriftGlass")),
            Box::new(StubRenderer::new(ClusterTheme::PorscheAnalog, "PorscheAnalog")),
            Box::new(StubRenderer::new(ClusterTheme::LiquidGlass, "LiquidGlass")),
            Box::new(StubRenderer::new(ClusterTheme::WindshieldHud, "WindshieldHud")),
            Box::new(StubRenderer::new(ClusterTheme::TeslaAdas, "TeslaAdas")),
            Box::new(StubRenderer::new(ClusterTheme::RetroCyber, "RetroCyber")),
            Box::new(StubRenderer::new(ClusterTheme::Gt7HoodPro, "Gt7HoodPro")),
            Box::new(StubRenderer::new(
                ClusterTheme::AmgMbuxSuperscreen,
                "AmgMbuxSuperscreen",
            )),
            Box::new(StubRenderer::new(ClusterTheme::LfaV10Nurburgring, "LfaV10Nurburgring")),
            Box::new(StubRenderer::new(ClusterTheme::PaganiHorologerie, "PaganiHorologerie")),
            Box::new(StubRenderer::new(ClusterTheme::F35HelmetHmd, "F35HelmetHmd")),
            Box::new(StubRenderer::new(ClusterTheme::BugattiTourbillon, "BugattiTourbillon")),
            Box::new(StubRenderer::new(ClusterTheme::EvaBioSync, "EvaBioSync")),
            Box::new(StubRenderer::new(ClusterTheme::KoenigseggGyro, "KoenigseggGyro")),
            Box::new(StubRenderer::new(ClusterTheme::ValkyrieLmp1, "ValkyrieLmp1")),
            Box::new(StubRenderer::new(ClusterTheme::SynthwaveOutrun, "SynthwaveOutrun")),
            Box::new(StubRenderer::new(ClusterTheme::ChronoMultiverse, "ChronoMultiverse")),
        ];
        Self { renderers }
    }

    pub fn get(&self, theme: ClusterTheme) -> Option<&dyn ClusterRenderer> {
        self.renderers.iter().find(|r| r.theme() == theme).map(|b| b.as_ref())
    }

    pub fn len(&self) -> usize {
        self.renderers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.renderers.is_empty()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_core::ClusterTheme;

    #[test]
    fn registry_has_18() {
        let reg = Registry::new();
        assert_eq!(reg.len(), 18);
        assert!(!reg.is_empty());
    }

    #[test]
    fn registry_lookup() {
        let reg = Registry::new();
        let r = reg.get(ClusterTheme::LiquidGlass).expect("present");
        assert_eq!(r.theme(), ClusterTheme::LiquidGlass);
    }
}
