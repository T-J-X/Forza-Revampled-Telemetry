//! Errors for every asset-forge step.

/// Anything that can stop a build step.
#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    /// File system error, with the path that failed.
    #[error("{path}: {source}")]
    Io {
        /// The file or directory.
        path: String,
        /// The cause.
        source: std::io::Error,
    },
    /// An image could not be decoded or encoded.
    #[error("image {path}: {message}")]
    Image {
        /// The file.
        path: String,
        /// What went wrong.
        message: String,
    },
    /// JSON could not be parsed or written.
    #[error("json {context}: {source}")]
    Json {
        /// What was being read.
        context: String,
        /// The cause.
        source: serde_json::Error,
    },
    /// An HTTP request failed.
    #[error("http {url}: {message}")]
    Http {
        /// The request URL.
        url: String,
        /// What went wrong.
        message: String,
    },
    /// An external tool (Blender, ComfyUI) failed or is missing.
    #[error("{tool}: {message}")]
    Tool {
        /// The tool.
        tool: String,
        /// What went wrong.
        message: String,
    },
    /// A produced asset failed a quality check (seam, size, triangle budget).
    #[error("check failed for {asset}: {message}")]
    Check {
        /// The asset id.
        asset: String,
        /// What failed.
        message: String,
    },
    /// Bad command-line usage.
    #[error("usage: {0}")]
    Usage(String),
}

/// Shorthand result.
pub type Result<T> = std::result::Result<T, ForgeError>;

/// Wraps an `std::io::Error` with its path.
pub fn io_err(path: &std::path::Path, source: std::io::Error) -> ForgeError {
    ForgeError::Io {
        path: path.display().to_string(),
        source,
    }
}

/// Builds a tool error.
pub fn tool_err(tool: &str, message: impl Into<String>) -> ForgeError {
    ForgeError::Tool {
        tool: tool.to_owned(),
        message: message.into(),
    }
}

/// Builds an image error.
pub fn image_err(path: &std::path::Path, message: impl std::fmt::Display) -> ForgeError {
    ForgeError::Image {
        path: path.display().to_string(),
        message: message.to_string(),
    }
}
