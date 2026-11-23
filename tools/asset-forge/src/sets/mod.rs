//! The asset sets. Each `build` runs its jobs through [`crate::ctx::Ctx::job`], so unchanged
//! jobs are skipped.
use crate::ctx::Ctx;
use crate::error::Result;

pub mod adas;
pub mod classic;
pub mod comfyset;
pub mod fluid;
pub mod fonts;
pub mod glass;
pub mod logos;
pub mod scifi;

/// Builds one named set.
pub fn build(ctx: &mut Ctx, set: &str) -> Result<()> {
    match set {
        "glass" => glass::build(ctx),
        "fluid" => fluid::build(ctx),
        "scifi" => scifi::build(ctx),
        "classic" => classic::build(ctx),
        "adas" => adas::build(ctx),
        "logos" => logos::build(ctx),
        "fonts" => fonts::build(ctx),
        "comfy" => comfyset::build(ctx),
        "gt7_hood_pro" => crate::recipes::gt7_hood_pro::build(ctx),
        "retro_cyber" => crate::recipes::retro_cyber::build(ctx),
        _ => Ok(()),
    }
}
