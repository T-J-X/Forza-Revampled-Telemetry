//! Errors of the review tool.
use std::fmt::Display;
use std::path::Path;

/// Everything that can go wrong in a command.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Bad arguments: the message says what was expected.
    #[error("{0}")]
    Usage(String),
    /// An image could not be read or written.
    #[error("{path}: {msg}")]
    Image {
        /// The file.
        path: String,
        /// What failed.
        msg: String,
    },
    /// File I/O on a named file or folder.
    #[error("{path}: {msg}")]
    File {
        /// The file or folder.
        path: String,
        /// What failed.
        msg: String,
    },
    /// `data/design_tokens.json` is missing or lacks a value.
    #[error("{path}: {msg}")]
    Tokens {
        /// The tokens file.
        path: String,
        /// What is missing.
        msg: String,
    },
    /// The SVG overlay failed to parse.
    #[error("overlay: {0}")]
    Overlay(String),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// A usage error.
pub fn usage(msg: impl Into<String>) -> Error {
    Error::Usage(msg.into())
}

/// An image error for `path`.
pub fn image_err(path: &Path, msg: impl Display) -> Error {
    Error::Image {
        path: path.display().to_string(),
        msg: msg.to_string(),
    }
}

/// A file or folder error for `path`.
pub fn file_err(path: &Path, msg: impl Display) -> Error {
    Error::File {
        path: path.display().to_string(),
        msg: msg.to_string(),
    }
}
