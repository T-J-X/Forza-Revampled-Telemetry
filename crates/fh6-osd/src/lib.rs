//! fh6-osd: the controller-first Liquid Glass OSD.
//!
//! A full-screen settings menu drawn with the render kit: the live cluster (the real selected
//! cluster, driven by the live dyno while the game is paused) beside a glass control panel
//! with five tabs. Every change is applied to the live view at once and saved to
//! `config.json` in the background. Navigation uses only the D-pad, LB/RB, View and Menu:
//! A, B, X and Y belong to the manual gearbox and are stripped before any input is read.
//!
//! - [`model`]: tabs, rows and how each one edits `AppConfig`.
//! - [`nav`]: the input state machine (browse, keyboard, remap) with hold-to-repeat.
//! - [`draw`] and [`host`]: kit drawing and the render-loop glue.
//! - [`pause`]: auto-pause (ESC on open mid-race) behind a [`pause::KeyboardInjector`].
//! - [`saver`]: debounced background saves.
//! - [`fit`]: the seam to the cockpit-fit backend (camera files, scrim rows).

pub mod anim;
pub mod draw;
pub mod dyno;
pub mod fit;
pub mod host;
pub mod keyboard;
pub mod model;
pub mod nav;
pub mod palette_edit;
pub mod pause;
pub mod remap;
pub mod saver;

pub use draw::{Frame, LiveStatus};
pub use dyno::LiveDynoGenerator;
pub use fit::{CameraApply, CockpitFit, ConfigOnlyFit, FitRow};
pub use host::OsdHost;
pub use model::{theme_for_view, Tab};
pub use nav::{Mode, NavInput, NavOutput, OsdController};
pub use pause::{AutoPause, KeyboardInjector, MockInjector, NullInjector, PauseInputs, VK_ESCAPE};
pub use saver::ConfigSaver;
