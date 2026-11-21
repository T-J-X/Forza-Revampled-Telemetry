//! Game process lifetime tracking for `--auto-exit` (pure logic; enumeration is in fh6-win).

/// A change in whether the game is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// The game is not running (or has exited).
    Stopped,
    /// The game is running.
    Started,
}

/// Reports start/stop transitions of one process name.
#[derive(Debug, Clone)]
pub struct ProcessWatcher {
    target_name: String,
    last_state: GameState,
    seen_running: bool,
}

impl ProcessWatcher {
    /// Watches for `target_name` (case-insensitive).
    pub fn new(target_name: &str) -> Self {
        Self {
            target_name: target_name.to_ascii_lowercase(),
            last_state: GameState::Stopped,
            seen_running: false,
        }
    }

    /// Feeds the current process list (any case); returns a transition, if any.
    pub fn check<S: AsRef<str>>(&mut self, processes: &[S]) -> Option<GameState> {
        let found = processes.iter().any(|p| p.as_ref().eq_ignore_ascii_case(&self.target_name));
        let current = if found { GameState::Started } else { GameState::Stopped };
        self.seen_running |= found;
        let changed = current != self.last_state;
        self.last_state = current;
        changed.then_some(current)
    }

    /// True once the game has been seen and is gone again: time for `--auto-exit` to quit.
    pub fn game_has_exited(&self) -> bool {
        self.seen_running && self.last_state == GameState::Stopped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions_are_reported_once() {
        let mut w = ProcessWatcher::new("ForzaHorizon6.exe");
        let none: [&str; 0] = [];
        assert_eq!(w.check(&none), None);
        assert_eq!(w.check(&["forzahorizon6.exe"]), Some(GameState::Started));
        assert_eq!(w.check(&["ForzaHorizon6.EXE"]), None);
        assert_eq!(w.check(&none), Some(GameState::Stopped));
    }

    #[test]
    fn auto_exit_waits_until_the_game_was_seen() {
        let mut w = ProcessWatcher::new("ForzaHorizon6.exe");
        let none: [&str; 0] = [];
        w.check(&none);
        assert!(!w.game_has_exited(), "never started: keep waiting (no 3.5 s quit)");
        w.check(&["forzahorizon6.exe"]);
        assert!(!w.game_has_exited());
        w.check(&none);
        assert!(w.game_has_exited());
    }
}
