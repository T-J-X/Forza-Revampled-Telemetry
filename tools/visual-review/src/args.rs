//! A small argument parser: positionals, `--flag value` pairs and `--switch`es, in order.
use crate::error::{usage, Result};
use std::str::FromStr;

/// Flags every command accepts.
const GLOBAL: [&str; 1] = ["--straight"];

/// The arguments after the command name.
#[derive(Debug, Default)]
pub struct Args {
    pos: Vec<String>,
    flags: Vec<(String, String)>,
    switches: Vec<String>,
}

impl Args {
    /// Splits `it` into positionals, `--flag value` pairs and the switches named in `switches`.
    /// After a bare `--`, everything is positional.
    ///
    /// # Errors
    /// A usage error when a flag has no value.
    pub fn parse(mut it: impl Iterator<Item = String>, switches: &[&str]) -> Result<Self> {
        let mut args = Self::default();
        let mut flags_done = false;
        while let Some(arg) = it.next() {
            if flags_done {
                args.pos.push(arg);
            } else if arg == "--" {
                flags_done = true;
            } else if arg == "-h" {
                // The short form of help, only before a bare `--`.
                args.switches.push("--help".to_string());
            } else if arg.starts_with("--") && arg.len() > 2 {
                if switches.contains(&arg.as_str()) {
                    args.switches.push(arg);
                } else {
                    let value = it.next().ok_or_else(|| usage(format!("{arg} needs a value")))?;
                    args.flags.push((arg, value));
                }
            } else {
                args.pos.push(arg);
            }
        }
        Ok(args)
    }

    /// The positional arguments.
    pub fn pos(&self) -> &[String] {
        &self.pos
    }

    /// The last value given for `name`.
    pub fn one(&self, name: &str) -> Option<&str> {
        self.flags
            .iter()
            .rev()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Every value given for `name`, in order.
    pub fn all(&self, name: &str) -> Vec<&str> {
        self.flags
            .iter()
            .filter(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
            .collect()
    }

    /// Whether the switch `name` was given.
    pub fn has(&self, name: &str) -> bool {
        self.switches.iter().any(|s| s == name)
    }

    /// Rejects any flag or switch not in `names`.
    ///
    /// # Errors
    /// A usage error naming the unknown flag.
    pub fn known(&self, names: &[&str]) -> Result<()> {
        let unknown = self
            .flags
            .iter()
            .map(|(k, _)| k)
            .chain(self.switches.iter())
            .find(|k| !names.contains(&k.as_str()) && !GLOBAL.contains(&k.as_str()));
        match unknown {
            Some(k) => Err(usage(format!("unknown flag {k}"))),
            None => Ok(()),
        }
    }

    /// The value of `name` parsed as a number, or `default` when it is absent.
    ///
    /// # Errors
    /// A usage error when the value does not parse.
    pub fn num<T: FromStr>(&self, name: &str, default: T) -> Result<T> {
        match self.one(name) {
            None => Ok(default),
            Some(v) => v
                .trim()
                .parse()
                .map_err(|_| usage(format!("{name}: {v:?} is not a number"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(v: &[&str]) -> Args {
        Args::parse(v.iter().map(|s| (*s).to_string()), &["--fit"]).expect("parses")
    }

    #[test]
    fn splits_positionals_flags_and_switches() {
        let a = parse(&[
            "a.png",
            "--zone",
            "gear:1,2,3,4",
            "b.png",
            "--fit",
            "--zone",
            "speed:5,6,7,8",
        ]);
        assert_eq!(a.pos(), ["a.png", "b.png"]);
        assert_eq!(a.all("--zone"), ["gear:1,2,3,4", "speed:5,6,7,8"]);
        assert_eq!(a.one("--zone"), Some("speed:5,6,7,8"));
        assert!(a.has("--fit"));
        assert!(a.known(&["--zone", "--fit"]).is_ok());
        assert!(a.known(&["--zone"]).is_err());
    }

    #[test]
    fn negative_values_and_missing_values() {
        let a = parse(&["--region", "-720,-320,1440,560", "--zoom", "3"]);
        assert_eq!(a.one("--region"), Some("-720,-320,1440,560"));
        assert_eq!(a.num("--zoom", 1u32).ok(), Some(3));
        assert_eq!(a.num("--eye-m", 0.6f64).ok(), Some(0.6));
        assert!(Args::parse(["--out".to_string()].into_iter(), &[]).is_err());
        let bad = parse(&["--zoom", "three"]);
        assert!(bad.num("--zoom", 1u32).is_err());
    }
}
