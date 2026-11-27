//! Serde support for settings stored as a name and parsed with [`FromStr`]: the cluster styles
//! and the unit and colour-mode enums of `config.json`.
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::{self, Deserializer, Unexpected, Visitor};

/// Deserializes a string with `T`'s [`FromStr`]. `expected` describes the setting for the error
/// message (`"a speed unit (MPH or KMH)"`). A name `T` rejects is an `invalid_value` error.
pub(crate) fn deserialize<'de, D, T>(deserializer: D, expected: &'static str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
{
    deserializer.deserialize_str(NameVisitor {
        expected,
        target: PhantomData,
    })
}

struct NameVisitor<T> {
    expected: &'static str,
    target: PhantomData<T>,
}

impl<T: FromStr> Visitor<'_> for NameVisitor<T> {
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.expected)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<T, E> {
        v.parse().map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
    }
}

/// Implements `Serialize` (as `self.name()`) and `Deserialize` (with [`FromStr`], see
/// [`deserialize`]) for a setting enum, so the name table in `name()` and `from_str` is the only
/// spelling of each option.
macro_rules! serde_by_name {
    ($ty:ty, $expected:literal) => {
        impl serde::Serialize for $ty {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.name())
            }
        }

        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                $crate::serde_str::deserialize(deserializer, $expected)
            }
        }
    };
}

pub(crate) use serde_by_name;
