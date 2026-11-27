//! A small string on the stack, for formatting numbers on a frame path without allocating.
use std::fmt;

/// A string of at most `N` bytes on the stack. Write into it with `write!`; a write that would
/// not fit fails with [`fmt::Error`] and leaves the text written so far.
#[derive(Clone, Copy)]
pub struct StackStr<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Default for StackStr<N> {
    fn default() -> Self {
        Self { buf: [0; N], len: 0 }
    }
}

impl<const N: usize> StackStr<N> {
    /// An empty string.
    pub fn new() -> Self {
        Self::default()
    }

    /// The text written so far (always valid UTF-8, since only whole `&str`s are copied in).
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    /// Clears the string.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl<const N: usize> fmt::Write for StackStr<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        if end > N {
            return Err(fmt::Error);
        }
        self.buf[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

impl<const N: usize> fmt::Debug for StackStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    #[test]
    fn stack_str_formats_without_heap() {
        let mut s = StackStr::<16>::new();
        write!(s, "{:5.2}", 1.23456).expect("fits");
        assert_eq!(s.as_str(), " 1.23");
        let mut tiny = StackStr::<2>::new();
        assert!(write!(tiny, "123").is_err(), "overflow is an error, not a panic");
    }

    #[test]
    fn a_failed_write_keeps_the_text_before_it_and_clear_empties() {
        let mut s = StackStr::<4>::new();
        write!(s, "ab").expect("fits");
        assert!(write!(s, "cde").is_err(), "5 bytes do not fit in 4");
        assert_eq!(s.as_str(), "ab");
        s.clear();
        assert_eq!(s.as_str(), "");
        write!(s, "wxyz").expect("exactly fits");
        assert_eq!(s.as_str(), "wxyz");
    }
}
