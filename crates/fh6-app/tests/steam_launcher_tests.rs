//! The Steam launcher's command-line handling.
use fh6_app::launcher::{plan, steam_launch_option, LaunchPlan};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

fn os(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

#[test]
fn steam_command_wraps_the_game_with_its_arguments() {
    // Steam replaces %command% with the game executable and its own arguments.
    let got = plan(os(&[
        r"E:\SteamLibrary\steamapps\common\ForzaHorizon6\ForzaHorizon6.exe",
        "+connect",
        "server",
    ]));
    assert_eq!(
        got,
        LaunchPlan::WrapGame {
            game: PathBuf::from(r"E:\SteamLibrary\steamapps\common\ForzaHorizon6\ForzaHorizon6.exe"),
            game_args: os(&["+connect", "server"]),
        }
    );
}

#[test]
fn game_arguments_with_spaces_pass_through_untouched() {
    let got = plan(os(&[
        r"C:\Games\Forza Horizon 6\ForzaHorizon6.exe",
        "-name",
        "two words",
    ]));
    let LaunchPlan::WrapGame { game, game_args } = got else {
        panic!("expected WrapGame");
    };
    assert_eq!(game, PathBuf::from(r"C:\Games\Forza Horizon 6\ForzaHorizon6.exe"));
    assert_eq!(game_args, os(&["-name", "two words"]));
}

#[test]
fn no_arguments_means_standalone() {
    assert_eq!(plan(Vec::<OsString>::new()), LaunchPlan::Standalone);
}

#[test]
fn launch_option_text_quotes_the_path() {
    let text = steam_launch_option(Path::new(r"E:\Tools\WheelDash"));
    assert_eq!(text, r#""E:\Tools\WheelDash\FH6_SteamLauncher.exe" %command%"#);
}
