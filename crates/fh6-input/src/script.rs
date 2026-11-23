//! A scripted controller for manual runs and demos without a pad: a timeline of button masks
//! played through the real tracker (hold toggle, edge latches, A/B/X/Y stripping).
//!
//! Script text: one step per line, `<ms> <mask>` (mask in hex, `0x` optional), meaning "from
//! `ms` after start, hold `mask`". `#` starts a comment. The last step holds until the end.
use crate::tracker::PadSource;
use crate::xinput::{XInputGamepad, XInputState};
use std::time::{Duration, Instant};

/// A parsed button timeline.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PadScript {
    steps: Vec<(Duration, u16)>,
}

/// Why a script line could not be read.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("pad script line {line}: {reason}")]
pub struct ScriptError {
    /// 1-based line number.
    pub line: usize,
    /// What was wrong.
    pub reason: &'static str,
}

impl PadScript {
    /// A script from `(ms, mask)` steps (sorted by time).
    #[must_use]
    pub fn from_steps(steps: &[(u64, u16)]) -> Self {
        let mut steps: Vec<(Duration, u16)> = steps.iter().map(|&(ms, m)| (Duration::from_millis(ms), m)).collect();
        steps.sort_by_key(|s| s.0);
        Self { steps }
    }

    /// Parses script text.
    ///
    /// # Errors
    /// Returns the first malformed line.
    pub fn parse(text: &str) -> Result<Self, ScriptError> {
        let mut steps = Vec::new();
        for (i, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split_whitespace();
            let (Some(ms), Some(mask)) = (parts.next(), parts.next()) else {
                return Err(ScriptError {
                    line: i + 1,
                    reason: "expected `<ms> <mask>`",
                });
            };
            let ms: u64 = ms.parse().map_err(|_| ScriptError {
                line: i + 1,
                reason: "time is not a whole number of ms",
            })?;
            let hex = mask.trim_start_matches("0x").trim_start_matches("0X");
            let mask = u16::from_str_radix(hex, 16).map_err(|_| ScriptError {
                line: i + 1,
                reason: "mask is not 16-bit hex",
            })?;
            steps.push((ms, mask));
        }
        Ok(Self::from_steps(&steps))
    }

    /// The mask held at `t`.
    #[must_use]
    pub fn mask_at(&self, t: Duration) -> u16 {
        self.steps.iter().rev().find(|(at, _)| *at <= t).map_or(0, |(_, m)| *m)
    }

    /// When the last step starts.
    #[must_use]
    pub fn length(&self) -> Duration {
        self.steps.last().map_or(Duration::ZERO, |s| s.0)
    }
}

/// A [`PadSource`] playing a [`PadScript`] on slot 0 from the moment it is created.
#[derive(Debug)]
pub struct ScriptedPad {
    script: PadScript,
    start: Instant,
    packet: u32,
}

impl ScriptedPad {
    /// Starts playing `script` now.
    #[must_use]
    pub fn new(script: PadScript) -> Self {
        Self {
            script,
            start: Instant::now(),
            packet: 0,
        }
    }
}

impl PadSource for ScriptedPad {
    fn poll_slot(&mut self, slot: usize) -> Option<XInputState> {
        if slot != 0 {
            return None;
        }
        self.packet = self.packet.wrapping_add(1);
        Some(XInputState {
            dwPacketNumber: self.packet,
            Gamepad: XInputGamepad {
                wButtons: self.script.mask_at(self.start.elapsed()),
                ..XInputGamepad::default()
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_plays() {
        let s = PadScript::parse("# open\n0 0\n500 0020\n1000 0x0000 # release\n").expect("valid");
        assert_eq!(s.mask_at(Duration::from_millis(100)), 0);
        assert_eq!(s.mask_at(Duration::from_millis(700)), 0x0020);
        assert_eq!(s.mask_at(Duration::from_millis(5000)), 0);
        assert_eq!(s.length(), Duration::from_millis(1000));
        assert_eq!(
            PadScript::parse("100").unwrap_err(),
            ScriptError {
                line: 1,
                reason: "expected `<ms> <mask>`"
            }
        );
        assert!(PadScript::parse("x 1").is_err());
        assert!(PadScript::parse("1 zz").is_err());
    }

    #[test]
    fn only_slot_zero_answers() {
        let mut pad = ScriptedPad::new(PadScript::from_steps(&[(0, 0x0001)]));
        assert_eq!(pad.poll_slot(0).map(|s| s.Gamepad.wButtons), Some(0x0001));
        assert!(pad.poll_slot(1).is_none());
    }
}
