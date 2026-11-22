//! The on-screen keyboard for the custom name: a D-pad grid. RB types the focused key, LB
//! deletes, Menu is done and View cancels. A, B, X and Y are never used.

/// Longest custom name, in characters.
pub const MAX_NAME_CHARS: usize = 14;

/// Character rows (upper case; lower case when Caps is off).
pub const KEY_ROWS: [&str; 4] = ["1234567890", "QWERTYUIOP", "ASDFGHJKL-", "ZXCVBNM_.!"];
/// Columns in every row.
pub const COLS: u8 = 10;
/// Rows including the action row.
pub const ROWS: u8 = 5;

/// One key of the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Types a character.
    Char(char),
    /// Toggles upper and lower case.
    Case,
    /// Types a space.
    Space,
    /// Deletes the last character.
    Delete,
    /// Accepts the name.
    Done,
}

impl Key {
    /// The cap label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Key::Char(_) => "",
            Key::Case => "Caps",
            Key::Space => "Space",
            Key::Delete => "Delete",
            Key::Done => "Done",
        }
    }
}

/// The key at `row`, `col` (the action row spans: Caps 2, Space 4, Delete 2, Done 2).
#[must_use]
pub fn key_at(row: u8, col: u8, caps: bool) -> Key {
    let col = col.min(COLS - 1);
    if let Some(r) = KEY_ROWS.get(usize::from(row)) {
        let c = r.chars().nth(usize::from(col)).unwrap_or(' ');
        return Key::Char(if caps { c } else { c.to_ascii_lowercase() });
    }
    match col {
        0..=1 => Key::Case,
        2..=5 => Key::Space,
        6..=7 => Key::Delete,
        _ => Key::Done,
    }
}

/// The span of columns a key covers in the action row (`(first, width)`), or one column.
#[must_use]
pub fn key_span(row: u8, col: u8) -> (u8, u8) {
    if usize::from(row) < KEY_ROWS.len() {
        return (col, 1);
    }
    match col {
        0..=1 => (0, 2),
        2..=5 => (2, 4),
        6..=7 => (6, 2),
        _ => (8, 2),
    }
}

/// What a key press did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyOutcome {
    /// The text or the grid changed.
    Edited,
    /// Nothing happened (full, empty).
    Ignored,
    /// Done was pressed.
    Done,
}

/// The keyboard's state while it is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keyboard {
    /// The name being edited.
    pub text: String,
    /// Focused row.
    pub row: u8,
    /// Focused column.
    pub col: u8,
    /// Upper case.
    pub caps: bool,
}

impl Default for Keyboard {
    fn default() -> Self {
        Self {
            text: String::with_capacity(64),
            row: 1,
            col: 0,
            caps: true,
        }
    }
}

impl Keyboard {
    /// Opens on `text` (truncated to [`MAX_NAME_CHARS`]).
    pub fn open(&mut self, text: &str) {
        self.text.clear();
        self.text.extend(text.chars().take(MAX_NAME_CHARS));
        self.row = 1;
        self.col = 0;
        self.caps = true;
    }

    /// The focused key.
    #[must_use]
    pub fn focused(&self) -> Key {
        key_at(self.row, self.col, self.caps)
    }

    /// Moves the focus; rows and columns wrap. In the action row a move skips a whole key.
    pub fn move_focus(&mut self, dx: i32, dy: i32) {
        if dy != 0 {
            self.row = (i32::from(self.row) + dy).rem_euclid(i32::from(ROWS)) as u8;
        }
        if dx != 0 {
            let (first, width) = key_span(self.row, self.col);
            let next = if dx > 0 {
                i32::from(first) + i32::from(width)
            } else {
                i32::from(first) - 1
            };
            let col = next.rem_euclid(i32::from(COLS)) as u8;
            self.col = key_span(self.row, col).0;
        }
    }

    /// Presses the focused key.
    pub fn press(&mut self) -> KeyOutcome {
        match self.focused() {
            Key::Char(c) => self.push(c),
            Key::Space => self.push(' '),
            Key::Case => {
                self.caps = !self.caps;
                KeyOutcome::Edited
            }
            Key::Delete => self.backspace(),
            Key::Done => KeyOutcome::Done,
        }
    }

    /// Deletes the last character.
    pub fn backspace(&mut self) -> KeyOutcome {
        if self.text.pop().is_some() {
            KeyOutcome::Edited
        } else {
            KeyOutcome::Ignored
        }
    }

    fn push(&mut self, c: char) -> KeyOutcome {
        if self.text.chars().count() >= MAX_NAME_CHARS {
            return KeyOutcome::Ignored;
        }
        self.text.push(c);
        KeyOutcome::Edited
    }

    /// The name to save: trimmed.
    #[must_use]
    pub fn result(&self) -> &str {
        self.text.trim()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_deletes_and_caps() {
        let mut kb = Keyboard::default();
        kb.open("");
        assert_eq!(kb.focused(), Key::Char('Q'));
        assert_eq!(kb.press(), KeyOutcome::Edited);
        kb.move_focus(1, 0);
        kb.press();
        assert_eq!(kb.text, "QW");
        kb.row = 4;
        kb.col = 0;
        kb.press();
        assert!(!kb.caps);
        kb.row = 1;
        kb.col = 2;
        kb.press();
        assert_eq!(kb.text, "QWe");
        kb.backspace();
        assert_eq!(kb.text, "QW");
    }

    #[test]
    fn limits_length_and_keeps_unicode_names() {
        let mut kb = Keyboard::default();
        kb.open("ØG『死』ŦJ乂");
        assert_eq!(kb.text.chars().count(), 8);
        for _ in 0..20 {
            kb.press();
        }
        assert_eq!(kb.text.chars().count(), MAX_NAME_CHARS);
        assert_eq!(kb.press(), KeyOutcome::Ignored);
    }

    #[test]
    fn action_row_moves_by_key() {
        let mut kb = Keyboard {
            row: 4,
            col: 0,
            ..Keyboard::default()
        };
        kb.move_focus(1, 0);
        assert_eq!(kb.focused(), Key::Space);
        kb.move_focus(1, 0);
        assert_eq!(kb.focused(), Key::Delete);
        kb.move_focus(1, 0);
        assert_eq!(kb.focused(), Key::Done);
        assert_eq!(kb.press(), KeyOutcome::Done);
        kb.move_focus(1, 0);
        assert_eq!(kb.focused(), Key::Case, "wraps");
        kb.move_focus(-1, 0);
        assert_eq!(kb.focused(), Key::Done);
        kb.move_focus(0, 1);
        assert_eq!(kb.row, 0, "rows wrap");
    }
}
