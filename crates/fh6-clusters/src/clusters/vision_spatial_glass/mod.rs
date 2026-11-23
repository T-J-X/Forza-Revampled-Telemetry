//! Vision Spatial Glass (`ClusterTheme::VisionSpatialGlass`), catalog row 2: Liquid Glass, cockpit, build #8.
//!
//! **Scaffold.** Draws the shared Phase 7 under-construction frame (`common::scaffold`) until the
//! cluster chat replaces this module with the cluster described in `spec.md`
//! (`program/phases/07-cluster-factory/CLUSTER_PROMPT.md`).
use crate::common::scaffold::Scaffold;
use fh6_core::ClusterTheme;
use fh6_kit::cluster::Setup;
use fh6_kit::{Cluster, FrameInput, Layout, Painter};

/// Vision Spatial Glass (scaffold).
#[derive(Debug, Clone)]
pub struct VisionSpatialGlass {
    scaffold: Scaffold,
}

impl VisionSpatialGlass {
    /// Builds the cluster (loads nothing yet).
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        Self {
            scaffold: Scaffold::new(setup, ClusterTheme::VisionSpatialGlass),
        }
    }
}

impl Cluster for VisionSpatialGlass {
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
    Box::new(VisionSpatialGlass::new(setup))
}
