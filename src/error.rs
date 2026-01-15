//! Error types for `OpenTimelineIO` operations.

use thiserror::Error;

/// The error type for `OpenTimelineIO` operations.
#[derive(Debug, Error)]
pub enum Error {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A JSON parsing error occurred.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// An invalid time range was specified.
    #[error("Invalid time range: {0}")]
    InvalidTimeRange(String),

    /// A referenced item was not found.
    #[error("Not found: {0}")]
    NotFound(String),
}

/// A specialized Result type for `OpenTimelineIO` operations.
pub type Result<T> = std::result::Result<T, Error>;
