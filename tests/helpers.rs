//! Test helpers for common timeline creation patterns.
//!
//! This module provides utility functions to reduce boilerplate in tests.

use otio_rs::{Clip, RationalTime, TimeRange, Timeline, Track};

/// Default frame rate for tests (24 fps).
pub const DEFAULT_RATE: f64 = 24.0;

/// Create a default time range for clips (1 second at 24fps).
#[must_use]
pub fn default_range() -> TimeRange {
    TimeRange::new(
        RationalTime::new(0.0, DEFAULT_RATE),
        RationalTime::new(DEFAULT_RATE, DEFAULT_RATE), // 1 second
    )
}

/// Create a time range with a specific duration in frames.
#[must_use]
pub fn range_frames(frames: f64) -> TimeRange {
    TimeRange::new(
        RationalTime::new(0.0, DEFAULT_RATE),
        RationalTime::new(frames, DEFAULT_RATE),
    )
}

/// Create a clip with a default 1-second duration.
#[must_use]
pub fn quick_clip(name: &str) -> Clip {
    Clip::new(name, default_range())
}

/// Create a timeline with a video track containing the specified number of clips.
#[must_use]
pub fn timeline_with_clips(name: &str, track_name: &str, clip_count: usize) -> Timeline {
    let mut timeline = Timeline::new(name);
    let mut track = timeline.add_video_track(track_name);
    for i in 0..clip_count {
        let clip = Clip::new(&format!("Clip {}", i), default_range());
        track.append_clip(clip).unwrap();
    }
    drop(track);
    timeline
}

/// Create a standalone video track with the specified number of clips.
#[must_use]
pub fn track_with_clips(name: &str, clip_count: usize) -> Track {
    let mut track = Track::new_video(name);
    for i in 0..clip_count {
        let clip = Clip::new(&format!("Clip {}", i), default_range());
        track.append_clip(clip).unwrap();
    }
    track
}
