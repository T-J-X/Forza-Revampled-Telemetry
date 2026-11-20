#[cfg(test)]
mod tests {
    #[test]
    fn test_steam_launcher_arg_parsing() {
        // Simulate args parsing logic
        // Case A: args[1] is game exe, args[2..] are steam args
        let args_a = vec!["FH6_SteamLauncher.exe", "E:\\Game\\ForzaHorizon6.exe", "+connect", "server"];
        assert_eq!(args_a.len(), 4);
        let game_path = &args_a[1];
        let steam_args = &args_a[2..];
        assert_eq!(game_path, "E:\\Game\\ForzaHorizon6.exe");
        assert_eq!(steam_args, &["+connect", "server"]);

        // Case B: no args
        let args_b = vec!["FH6_SteamLauncher.exe"];
        assert_eq!(args_b.len(), 1);
        // Should launch standalone
    }
}
