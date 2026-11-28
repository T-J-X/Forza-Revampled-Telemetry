//! A monotonic stopwatch that is safe on every target: `std::time::Instant` natively, and a
//! zero clock on `wasm32-unknown-unknown`, where `Instant::now` panics. Statistics use it, and
//! so does the frame budget of a brand dial's upload (`logo::DialLoader::poll`), which has
//! nothing to upload on `wasm32` (the decoding thread cannot start there). Animation time always
//! comes from the host's `dt`.
use std::time::Duration;

/// A point in time to measure from.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Stamp {
    #[cfg(not(target_arch = "wasm32"))]
    at: std::time::Instant,
}

impl Stamp {
    /// Now.
    pub(crate) fn now() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            at: std::time::Instant::now(),
        }
    }

    /// Time since this stamp (zero on wasm32).
    pub(crate) fn elapsed(&self) -> Duration {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.at.elapsed()
        }
        #[cfg(target_arch = "wasm32")]
        {
            Duration::ZERO
        }
    }
}
