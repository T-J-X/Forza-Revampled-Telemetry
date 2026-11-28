//! McLaren Slim Display (`ClusterTheme::MclarenSlimDisplay`), catalog row 70.
//! Supercar category, Modern Digital 2027 family, cockpit and hood.
//!
//! **Scaffold.** Draws the shared Phase 7 under-construction frame (`common::scaffold`) until the
//! dash thread replaces this module with the dash described in `spec.md`
//! (`program/phases/07-cluster-factory/CLUSTER_PROMPT.md`).
use crate::common::scaffold::Scaffold;
use fh6_core::ClusterTheme;
use fh6_kit::cluster::Setup;
use fh6_kit::{Cluster, FrameInput, Layout, Painter};

/// McLaren Slim Display (scaffold).
#[derive(Debug, Clone)]
pub struct MclarenSlimDisplay {
    scaffold: Scaffold,
}

impl MclarenSlimDisplay {
    /// Builds the cluster (loads nothing yet).
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        Self {
            scaffold: Scaffold::new(setup, ClusterTheme::MclarenSlimDisplay),
        }
    }
}

impl Cluster for MclarenSlimDisplay {
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
    Box::new(MclarenSlimDisplay::new(setup))
}
