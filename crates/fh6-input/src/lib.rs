//! fh6-input: XInput polling with strict manual-transmission button locks.

pub mod tracker;
pub mod xinput;
pub mod hold_state;
pub mod guide;

pub use tracker::InputControllerTracker;
pub use tracker::PadSource;
pub use tracker::MASK_MANUAL_LOCK;
