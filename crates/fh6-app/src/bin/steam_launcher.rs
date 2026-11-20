#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Case B: double-clicked directly with no args
    if args.len() == 1 {
        // Spawn FH6_WheelDash in standalone mode
        let _ = Command::new("FH6_WheelDash.exe").spawn();
        // Try launch game if exe exists in folder
        if let Some(current_dir) = std::env::current_dir().ok() {
            let game_path = current_dir.join("ForzaHorizon6.exe");
            if game_path.exists() {
                let _ = Command::new(game_path).spawn();
            } else {
                // Launch via Steam URI
                let _ = Command::new("cmd")
                    .args(&["/C", "start", "steam://rungameid/ForzaHorizon6"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            }
        }
        return;
    }

    // Case A: launched by Steam via %command%
    // args[0] is launcher, args[1] is game exe path, args[2..] are Steam args
    let game_path = &args[1];
    let steam_args = &args[2..];

    // Spawn FH6_WheelDash with --auto-exit detached
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
    if let Some(dir) = exe_dir {
        let wheeldash = dir.join("FH6_WheelDash.exe");
        if wheeldash.exists() {
            let mut cmd = Command::new(wheeldash);
            cmd.arg("--auto-exit")
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            #[cfg(windows)]
            {
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }
            let _ = cmd.spawn();
        }
    }

    // Launch game and wait
    let mut child = Command::new(game_path)
        .args(steam_args)
        .spawn()
        .expect("Failed to launch ForzaHorizon6");

    // Wait for game exit
    let status = child.wait().expect("Failed to wait on game process");

    // Signal WheelDash to close (give it time)
    thread::sleep(Duration::from_millis(500));
    std::process::exit(status.code().unwrap_or(0));
}
