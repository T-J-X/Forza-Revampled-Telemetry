//! Valkyrie LMP1 (`ClusterTheme::ValkyrieLmp1`), catalog row 13: Race Dash, cockpit+hood, build #16.
//!
//! **Scaffold.** Draws the shared Phase 7 under-construction frame (`common::scaffold`) until the
//! cluster chat replaces this module with the cluster described in `spec.md`
//! (`program/phases/07-cluster-factory/CLUSTER_PROMPT.md`).
use crate::common::scaffold::Scaffold;
use fh6_core::ClusterTheme;
use fh6_kit::cluster::Setup;
use fh6_kit::{Cluster, FrameInput, Layout, Painter};

/// Valkyrie LMP1 (scaffold).
#[derive(Debug, Clone)]
pub struct ValkyrieLmp1 {
    scaffold: Scaffold,
}

impl ValkyrieLmp1 {
    /// Builds the cluster (loads nothing yet).
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        Self {
            scaffold: Scaffold::new(setup, ClusterTheme::ValkyrieLmp1),
        }
    }
}

impl Cluster for ValkyrieLmp1 {
    fn update(&mut self, input: &FrameInput<'_>) {
        self.scaffold.update(input);
    }

    fn draw(&self, p: &mut Painter<'_>, layout: &Layout) {
        self.scaffold.draw(p, layout);
    }

    fn animating(&self) -> bool {
        self.scaffold.animating()
    }
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(ValkyrieLmp1::new(setup))
}
