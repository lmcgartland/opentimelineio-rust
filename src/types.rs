//! Re-exports and type aliases for convenience.

/// A specialized Result type for OTIO operations.
pub type Result<T> = std::result::Result<T, crate::OtioError>;
