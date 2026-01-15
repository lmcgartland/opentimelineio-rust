//! Re-exports and type aliases for convenience.

/// A specialized Result type for OTIO operations.
pub type Result<T> = std::result::Result<T, crate::OtioError>;

/// The kind of a track (video or audio).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackKind {
    /// A video track.
    Video,
    /// An audio track.
    Audio,
}
