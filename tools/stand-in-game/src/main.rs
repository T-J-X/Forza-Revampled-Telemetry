//! Stands in for the game during manual overlay tests.
//!
//! ```text
//! cargo run -p stand-in-game -- [--color 101418] [--title "Stand-in game"]
//! FH6_WheelDash --game-process fh6-stand-in.exe
//! ```
//! Fills the primary monitor with a solid colour and takes the focus; Esc closes it. Copy the
//! exe under another name to get a second, non-game window for the focus-loss check.
use std::process::ExitCode;

fn parse_rgb(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 || !hex.is_ascii() {
        return None;
    }
    let c = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some([c(0)?, c(2)?, c(4)?])
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let value = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
    let rgb = value("--color")
        .and_then(|v| parse_rgb(v))
        .unwrap_or([0x10, 0x14, 0x18]);
    let title = value("--title").map_or("Stand-in game", String::as_str);
    fh6_win::monitor::enable_per_monitor_dpi_awareness();
    let Some(rect) = fh6_win::monitor::primary_monitor_rect() else {
        eprintln!("no primary monitor");
        return ExitCode::FAILURE;
    };
    match fh6_win::stand_in::run_stand_in_window(rect, rgb, title) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("stand-in window failed: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_parse() {
        assert_eq!(parse_rgb("#101418"), Some([0x10, 0x14, 0x18]));
        assert_eq!(parse_rgb("zz0000"), None);
    }
}
