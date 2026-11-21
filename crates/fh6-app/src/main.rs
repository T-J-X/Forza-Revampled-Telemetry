#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! FH6_WheelDash: the overlay executable.

use fh6_app::args::{self, Command, USAGE};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = match args::parse(std::env::args().skip(1)) {
        Ok(Command::Run(args)) => args,
        Ok(Command::Help) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(err) => {
            eprintln!("{err}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    fh6_app::logging::init(args.log.as_deref());
    run(args)
}

#[cfg(windows)]
fn run(args: fh6_app::args::AppArgs) -> ExitCode {
    match fh6_app::app::run(args) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(err) => {
            tracing::error!(error = %err, "FH6_WheelDash could not start");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn run(_args: fh6_app::args::AppArgs) -> ExitCode {
    eprintln!("FH6_WheelDash runs on Windows only");
    ExitCode::FAILURE
}
