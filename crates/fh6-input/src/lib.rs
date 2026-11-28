//! fh6-input: XInput polling with strict manual-transmission button locks. Only [`xinput`]
//! uses `unsafe`; the workspace lint table denies it everywhere else in the crate.

pub mod guide;
pub mod hold_state;
pub mod poller;
pub mod script;
pub mod tracker;
#[allow(
    unsafe_code,
    reason = "XInput loading and calls: every `unsafe` block names its invariant in a `// SAFETY:` comment"
)]
pub mod xinput;

pub use hold_state::{HoldConfig, HoldToggle};
pub use poller::{InputPoller, InputShared};
pub use script::{PadScript, ScriptError, ScriptedPad};
pub use tracker::{InputControllerTracker, InputEvents, PadSource, PollContext, ResyncConfig, MASK_MANUAL_LOCK};
pub use xinput::{XInput, XInputError};
