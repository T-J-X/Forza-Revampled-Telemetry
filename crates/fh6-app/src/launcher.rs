//! What `FH6_SteamLauncher` does with its command line.
//!
//! Steam runs the launch option `"<install dir>\FH6_SteamLauncher.exe" %command%`, so the
//! launcher receives the game's own command line after its name: the game executable first,
//! then the game's arguments.
use std::ffi::OsString;
use std::path::PathBuf;

/// The launcher's plan for this run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchPlan {
    /// Started by Steam: start WheelDash, then the game with its arguments, wait for it.
    WrapGame {
        /// The game executable from `%command%`.
        game: PathBuf,
        /// The game's arguments, passed through untouched.
        game_args: Vec<OsString>,
    },
    /// Double-clicked with no arguments: start WheelDash on its own and explain the setup.
    Standalone,
}

/// Builds the plan from the arguments after the launcher's own name.
pub fn plan(args: impl IntoIterator<Item = OsString>) -> LaunchPlan {
    let mut args = args.into_iter();
    match args.next() {
        Some(game) => LaunchPlan::WrapGame {
            game: PathBuf::from(game),
            game_args: args.collect(),
        },
        None => LaunchPlan::Standalone,
    }
}

/// The Steam launch option text for an install folder.
pub fn steam_launch_option(install_dir: &std::path::Path) -> String {
    format!("\"{}\" %command%", install_dir.join("FH6_SteamLauncher.exe").display())
}
