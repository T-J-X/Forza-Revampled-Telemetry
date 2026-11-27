//! fh6-clusters: the cluster catalog ([`catalog::CATALOG`]), the cluster registry (contract v2,
//! [`registry::REGISTRY`]), one module per catalog cluster ([`clusters`]), shared components
//! ([`common`]), the review states and motion sequence ([`review`]) and the two Phase 5 kit
//! samples ([`kit`]). Every cluster draws with `fh6-kit`.
//!
//! Phase 7 rule: a cluster chat edits only `src/clusters/<snake_name>/` (and its own assets and
//! goldens). The catalog, registry, theme enum, golden harness and `common/` are shared and are
//! changed only by the conductor.

pub mod catalog;
pub mod clusters;
pub mod common;
pub mod kit;
pub mod registry;
pub mod review;
