//! Bionic Spaceframe (`ClusterTheme::BionicSpaceframe`), catalog row 39.
//! Hypercar category, Race Dash family, cockpit.
//!
//! **Scaffold.** Draws the shared Phase 7 under-construction frame (`common::scaffold`) until the
//! dash thread replaces this module with the dash described in `spec.md`
//! (`program/phases/07-cluster-factory/CLUSTER_PROMPT.md`).
use crate::common::scaffold::Scaffold;
use fh6_core::ClusterTheme;
use fh6_kit::cluster::Setup;
use fh6_kit::{Cluster, FrameInput, Layout, Painter};

/// Bionic Spaceframe (scaffold).
#[derive(Debug, Clone)]
pub struct BionicSpaceframe {
    scaffold: Scaffold,
}

impl BionicSpaceframe {
    /// Builds the cluster (loads nothing yet).
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        Self {
            scaffold: Scaffold::new(setup, ClusterTheme::BionicSpaceframe),
        }
    }
}

impl Cluster for BionicSpaceframe {
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
    Box::new(BionicSpaceframe::new(setup))
}
