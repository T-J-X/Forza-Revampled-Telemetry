#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! FH6_SteamLauncher: the Steam launch option wrapper.
//!
//! Set the game's Steam launch option to `"<install dir>\FH6_SteamLauncher.exe" %command%`.
//! The launcher starts FH6_WheelDash (`--auto-exit`), then the game with its own arguments,
//! waits for the game to exit (including a relaunched copy of itself), then tells
//! FH6_WheelDash to quit.

use fh6_app::launcher::{self, LaunchPlan};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// `CREATE_NO_WINDOW`: no console flashes up for the overlay.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(Path::to_path_buf)
}

fn start_wheeldash(dir: &Path, auto_exit: bool) {
    let exe = dir.join("FH6_WheelDash.exe");
    if !exe.is_file() {
        tracing::warn!(path = %exe.display(), "FH6_WheelDash.exe not found next to the launcher");
        return;
    }
    let mut cmd = Command::new(&exe);
    if auto_exit {
        cmd.arg("--auto-exit");
    }
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    match cmd.spawn() {
        Ok(child) => tracing::info!(pid = child.id(), "FH6_WheelDash started"),
        Err(err) => tracing::error!(error = %err, "FH6_WheelDash did not start"),
    }
}

#[cfg(windows)]
fn game_still_running(game: &Path) -> bool {
    game.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(fh6_win::system::is_process_running)
}

#[cfg(not(windows))]
fn game_still_running(_game: &Path) -> bool {
    false
}

#[cfg(windows)]
fn signal_overlay_quit() {
    if fh6_win::QuitSignal::signal_running_overlay() {
        tracing::info!("asked FH6_WheelDash to quit");
    }
}

#[cfg(not(windows))]
fn signal_overlay_quit() {}

fn main() -> ExitCode {
    fh6_app::logging::init(None);
    let Some(dir) = exe_dir() else {
        tracing::error!("cannot locate the launcher's folder");
        return ExitCode::FAILURE;
    };
    match launcher::plan(std::env::args_os().skip(1)) {
        LaunchPlan::Standalone => {
            start_wheeldash(&dir, false);
            tracing::info!(
                option = %launcher::steam_launch_option(&dir),
                "started without a game; set this as the game's Steam launch option"
            );
            ExitCode::SUCCESS
        }
        LaunchPlan::WrapGame { game, game_args } => {
            start_wheeldash(&dir, true);
            let status = match Command::new(&game).args(&game_args).spawn() {
                Ok(mut child) => child.wait(),
                Err(err) => {
                    tracing::error!(game = %game.display(), error = %err, "the game did not start");
                    signal_overlay_quit();
                    return ExitCode::FAILURE;
                }
            };
            // Some launchers re-exec the game; wait until no copy of it is left.
            while game_still_running(&game) {
                std::thread::sleep(Duration::from_secs(1));
            }
            signal_overlay_quit();
            match status {
                // Crash codes such as 0xC0000005 are negative or above 255: report failure,
                // never success.
                Ok(status) => status
                    .code()
                    .and_then(|c| u8::try_from(c).ok())
                    .map_or(ExitCode::FAILURE, ExitCode::from),
                Err(err) => {
                    tracing::error!(error = %err, "waiting for the game failed");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
