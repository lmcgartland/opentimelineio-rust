//! # `otio-rs`
//!
//! Rust bindings to [OpenTimelineIO](https://opentimeline.io/) - an open-source
//! API and interchange format for editorial timeline information.
//!
//! ## Example
//!
//! ```no_run
//! use otio_rs::{Timeline, Track, Clip, RationalTime, TimeRange};
//!
//! let mut timeline = Timeline::new("My Timeline");
//! timeline.set_global_start_time(RationalTime::new(0.0, 24.0)).unwrap();
//!
//! let mut video_track = timeline.add_video_track("V1");
//!
//! let source_range = TimeRange::new(
//!     RationalTime::new(0.0, 24.0),
//!     RationalTime::new(48.0, 24.0), // 2 seconds at 24fps
//! );
//! let clip = Clip::new("My Clip", source_range);
//! video_track.append_clip(clip).unwrap();
//!
//! timeline.write_to_file(std::path::Path::new("output.otio")).unwrap();
//! ```

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// FFI wrappers use unwrap for CString which can only panic on interior nulls
#![allow(clippy::missing_panics_doc)]

mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod traits;
pub use traits::HasMetadata;

mod types;
pub use types::*;

mod iterators;
pub use iterators::{
    ClipRef, Composable, GapRef, StackChildIter, StackRef, TrackChildIter, TrackRef,
};

mod builders;
pub use builders::{ClipBuilder, ExternalReferenceBuilder, TimelineBuilder};

use std::ffi::{CStr, CString};
use std::path::Path;

/// Error type for OTIO operations.
#[derive(Debug)]
pub struct OtioError {
    pub code: i32,
    pub message: String,
}

impl std::fmt::Display for OtioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OTIO error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for OtioError {}

impl From<ffi::OtioError> for OtioError {
    fn from(e: ffi::OtioError) -> Self {
        let message = unsafe {
            CStr::from_ptr(e.message.as_ptr())
                .to_string_lossy()
                .into_owned()
        };
        OtioError {
            code: e.code,
            message,
        }
    }
}

/// A rational time value with a rate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RationalTime {
    pub value: f64,
    pub rate: f64,
}

impl RationalTime {
    /// Create a new `RationalTime` with the given value and rate.
    #[must_use]
    pub fn new(value: f64, rate: f64) -> Self {
        Self { value, rate }
    }

    /// Create a `RationalTime` from seconds at the given rate.
    #[must_use]
    pub fn from_seconds(seconds: f64, rate: f64) -> Self {
        Self {
            value: seconds * rate,
            rate,
        }
    }

    /// Convert to seconds.
    #[must_use]
    pub fn to_seconds(self) -> f64 {
        self.value / self.rate
    }
}

impl From<RationalTime> for ffi::OtioRationalTime {
    fn from(rt: RationalTime) -> Self {
        ffi::OtioRationalTime {
            value: rt.value,
            rate: rt.rate,
        }
    }
}

/// A time range with start time and duration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeRange {
    pub start_time: RationalTime,
    pub duration: RationalTime,
}

impl TimeRange {
    /// Create a new `TimeRange` with the given start time and duration.
    #[must_use]
    pub fn new(start_time: RationalTime, duration: RationalTime) -> Self {
        Self {
            start_time,
            duration,
        }
    }

    /// Get the end time of this range.
    #[must_use]
    pub fn end_time(&self) -> RationalTime {
        RationalTime::new(
            self.start_time.value + self.duration.value,
            self.start_time.rate,
        )
    }
}

impl From<TimeRange> for ffi::OtioTimeRange {
    fn from(tr: TimeRange) -> Self {
        ffi::OtioTimeRange {
            start_time: tr.start_time.into(),
            duration: tr.duration.into(),
        }
    }
}

/// A timeline is the top-level container for editorial content.
pub struct Timeline {
    ptr: *mut ffi::OtioTimeline,
}

impl Timeline {
    /// Create a new timeline with the given name.
    #[must_use]
    pub fn new(name: &str) -> Self {
        let c_name = CString::new(name).unwrap();
        let ptr = unsafe { ffi::otio_timeline_create(c_name.as_ptr()) };
        Self { ptr }
    }

    /// Set the global start time of the timeline.
    ///
    /// # Errors
    ///
    /// Returns an error if the global start time cannot be set.
    pub fn set_global_start_time(&mut self, time: RationalTime) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_timeline_set_global_start_time(self.ptr, time.into(), &mut err) };
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Add a video track to the timeline.
    #[must_use]
    pub fn add_video_track(&mut self, name: &str) -> Track {
        let c_name = CString::new(name).unwrap();
        let ptr = unsafe { ffi::otio_timeline_add_video_track(self.ptr, c_name.as_ptr()) };
        Track { ptr, owned: false } // Timeline owns this track
    }

    /// Add an audio track to the timeline.
    #[must_use]
    pub fn add_audio_track(&mut self, name: &str) -> Track {
        let c_name = CString::new(name).unwrap();
        let ptr = unsafe { ffi::otio_timeline_add_audio_track(self.ptr, c_name.as_ptr()) };
        Track { ptr, owned: false } // Timeline owns this track
    }

    /// Write the timeline to a JSON file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let c_path = CString::new(path.to_string_lossy().as_ref()).unwrap();
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_timeline_write_to_file(self.ptr, c_path.as_ptr(), &mut err) };
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Read a timeline from a JSON file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn read_from_file(path: &Path) -> Result<Self> {
        let c_path = CString::new(path.to_string_lossy().as_ref()).unwrap();
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let ptr = unsafe { ffi::otio_timeline_read_from_file(c_path.as_ptr(), &mut err) };
        if ptr.is_null() {
            Err(err.into())
        } else {
            Ok(Self { ptr })
        }
    }

    /// Get the root stack (tracks container) for this timeline.
    ///
    /// The returned `StackRef` is a non-owning reference to the timeline's stack.
    /// Use `tracks().children()` to iterate over the tracks.
    #[must_use]
    pub fn tracks(&self) -> StackRef<'_> {
        let ptr = unsafe { ffi::otio_timeline_get_tracks(self.ptr) };
        StackRef::new(ptr)
    }
}

traits::impl_has_metadata!(Timeline, otio_timeline_set_metadata_string, otio_timeline_get_metadata_string);

impl Drop for Timeline {
    fn drop(&mut self) {
        unsafe { ffi::otio_timeline_free(self.ptr) }
    }
}

// Safety: Timeline is safe to send between threads
unsafe impl Send for Timeline {}

/// A track contains clips, gaps, and other items.
///
/// Tracks can be created standalone or added to a Timeline. When created
/// standalone, the Track owns its memory. When added to a Timeline or Stack,
/// ownership transfers to the parent.
pub struct Track {
    ptr: *mut ffi::OtioTrack,
    owned: bool,
}

impl Track {
    /// Create a new video track with the given name.
    #[must_use]
    pub fn new_video(name: &str) -> Self {
        let c_name = CString::new(name).unwrap();
        let ptr = unsafe { ffi::otio_track_create_video(c_name.as_ptr()) };
        Self { ptr, owned: true }
    }

    /// Create a new audio track with the given name.
    #[must_use]
    pub fn new_audio(name: &str) -> Self {
        let c_name = CString::new(name).unwrap();
        let ptr = unsafe { ffi::otio_track_create_audio(c_name.as_ptr()) };
        Self { ptr, owned: true }
    }

    /// Append a clip to this track.
    ///
    /// # Errors
    ///
    /// Returns an error if the clip cannot be appended.
    #[allow(clippy::forget_non_drop)] // Clip ownership transfers to C++
    pub fn append_clip(&mut self, clip: Clip) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result = unsafe { ffi::otio_track_append_clip(self.ptr, clip.ptr, &mut err) };
        std::mem::forget(clip); // Track now owns the clip
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Append a gap to this track.
    ///
    /// # Errors
    ///
    /// Returns an error if the gap cannot be appended.
    #[allow(clippy::forget_non_drop)] // Gap ownership transfers to C++
    pub fn append_gap(&mut self, gap: Gap) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result = unsafe { ffi::otio_track_append_gap(self.ptr, gap.ptr, &mut err) };
        std::mem::forget(gap); // Track now owns the gap
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Append a stack to this track.
    ///
    /// This is useful for versioning and alternative cuts within a track.
    ///
    /// # Errors
    ///
    /// Returns an error if the stack cannot be appended.
    #[allow(clippy::forget_non_drop)] // Stack ownership transfers to C++
    pub fn append_stack(&mut self, stack: Stack) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result = unsafe { ffi::otio_track_append_stack(self.ptr, stack.ptr, &mut err) };
        std::mem::forget(stack); // Track now owns the stack
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Get the number of children in this track.
    #[must_use]
    pub fn children_count(&self) -> usize {
        let count = unsafe { ffi::otio_track_children_count(self.ptr) };
        count.max(0) as usize
    }

    /// Iterate over children of this track.
    ///
    /// Returns an iterator of `Composable` items (clips, gaps, stacks).
    pub fn children(&self) -> TrackChildIter<'_> {
        TrackChildIter::new(self.ptr)
    }

    /// Remove a child at the given index.
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds.
    pub fn remove_child(&mut self, index: usize) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_track_remove_child(self.ptr, index as i32, &mut err) };
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Insert a clip at the given index.
    ///
    /// # Errors
    ///
    /// Returns an error if the clip cannot be inserted.
    #[allow(clippy::forget_non_drop)]
    pub fn insert_clip(&mut self, index: usize, clip: Clip) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_track_insert_clip(self.ptr, index as i32, clip.ptr, &mut err) };
        std::mem::forget(clip);
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Insert a gap at the given index.
    ///
    /// # Errors
    ///
    /// Returns an error if the gap cannot be inserted.
    #[allow(clippy::forget_non_drop)]
    pub fn insert_gap(&mut self, index: usize, gap: Gap) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_track_insert_gap(self.ptr, index as i32, gap.ptr, &mut err) };
        std::mem::forget(gap);
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Insert a stack at the given index.
    ///
    /// # Errors
    ///
    /// Returns an error if the stack cannot be inserted.
    #[allow(clippy::forget_non_drop)]
    pub fn insert_stack(&mut self, index: usize, stack: Stack) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_track_insert_stack(self.ptr, index as i32, stack.ptr, &mut err) };
        std::mem::forget(stack);
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Clear all children from this track.
    ///
    /// # Errors
    ///
    /// Returns an error if the children cannot be cleared.
    pub fn clear_children(&mut self) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result = unsafe { ffi::otio_track_clear_children(self.ptr, &mut err) };
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }
}

traits::impl_has_metadata!(Track, otio_track_set_metadata_string, otio_track_get_metadata_string);

impl Drop for Track {
    fn drop(&mut self) {
        if self.owned {
            unsafe { ffi::otio_track_free(self.ptr) }
        }
    }
}

// Safety: Track is safe to send between threads
unsafe impl Send for Track {}

/// A clip represents a segment of media.
pub struct Clip {
    ptr: *mut ffi::OtioClip,
}

impl Clip {
    /// Create a new clip with the given name and source range.
    #[must_use]
    pub fn new(name: &str, source_range: TimeRange) -> Self {
        let c_name = CString::new(name).unwrap();
        let ptr = unsafe { ffi::otio_clip_create(c_name.as_ptr(), source_range.into()) };
        Self { ptr }
    }

    /// Set the media reference for this clip.
    ///
    /// # Errors
    ///
    /// Returns an error if the media reference cannot be set.
    #[allow(clippy::forget_non_drop)] // Reference ownership transfers to C++
    pub fn set_media_reference(&mut self, reference: ExternalReference) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_clip_set_media_reference(self.ptr, reference.ptr, &mut err) };
        std::mem::forget(reference); // Clip now owns the reference
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }
}

traits::impl_has_metadata!(Clip, otio_clip_set_metadata_string, otio_clip_get_metadata_string);

/// A gap represents empty space in a track.
pub struct Gap {
    ptr: *mut ffi::OtioGap,
}

impl Gap {
    /// Create a new gap with the given duration.
    #[must_use]
    pub fn new(duration: RationalTime) -> Self {
        let ptr = unsafe { ffi::otio_gap_create(duration.into()) };
        Self { ptr }
    }
}

traits::impl_has_metadata!(Gap, otio_gap_set_metadata_string, otio_gap_get_metadata_string);

/// An external reference points to a media file.
pub struct ExternalReference {
    ptr: *mut ffi::OtioExternalRef,
}

impl ExternalReference {
    /// Create a new external reference with the given URL.
    #[must_use]
    pub fn new(target_url: &str) -> Self {
        let c_url = CString::new(target_url).unwrap();
        let ptr = unsafe { ffi::otio_external_ref_create(c_url.as_ptr()) };
        Self { ptr }
    }

    /// Set the available range for this media reference.
    ///
    /// # Errors
    ///
    /// Returns an error if the available range cannot be set.
    pub fn set_available_range(&mut self, range: TimeRange) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_external_ref_set_available_range(self.ptr, range.into(), &mut err) };
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }
}

traits::impl_has_metadata!(ExternalReference, otio_external_ref_set_metadata_string, otio_external_ref_get_metadata_string);

/// A stack is a composition that layers its children.
///
/// Stacks are used for:
/// - Timeline's root tracks container
/// - Nested compositions within tracks (for versioning/alternatives)
/// - Clip stacks for layered effects
pub struct Stack {
    ptr: *mut ffi::OtioStack,
}

impl Stack {
    /// Create a new stack with the given name.
    #[must_use]
    pub fn new(name: &str) -> Self {
        let c_name = CString::new(name).unwrap();
        let ptr = unsafe { ffi::otio_stack_create(c_name.as_ptr()) };
        Self { ptr }
    }

    /// Append a track to this stack.
    ///
    /// # Errors
    ///
    /// Returns an error if the track cannot be appended.
    #[allow(clippy::forget_non_drop)] // Track ownership transfers to C++
    pub fn append_track(&mut self, track: Track) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result = unsafe { ffi::otio_stack_append_track(self.ptr, track.ptr, &mut err) };
        std::mem::forget(track); // Stack now owns the track
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Append a clip to this stack.
    ///
    /// # Errors
    ///
    /// Returns an error if the clip cannot be appended.
    #[allow(clippy::forget_non_drop)] // Clip ownership transfers to C++
    pub fn append_clip(&mut self, clip: Clip) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result = unsafe { ffi::otio_stack_append_clip(self.ptr, clip.ptr, &mut err) };
        std::mem::forget(clip); // Stack now owns the clip
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Append a gap to this stack.
    ///
    /// # Errors
    ///
    /// Returns an error if the gap cannot be appended.
    #[allow(clippy::forget_non_drop)] // Gap ownership transfers to C++
    pub fn append_gap(&mut self, gap: Gap) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result = unsafe { ffi::otio_stack_append_gap(self.ptr, gap.ptr, &mut err) };
        std::mem::forget(gap); // Stack now owns the gap
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Append a child stack to this stack.
    ///
    /// # Errors
    ///
    /// Returns an error if the child stack cannot be appended.
    #[allow(clippy::forget_non_drop)] // Stack ownership transfers to C++
    pub fn append_stack(&mut self, child: Stack) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result = unsafe { ffi::otio_stack_append_stack(self.ptr, child.ptr, &mut err) };
        std::mem::forget(child); // Parent stack now owns the child
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Get the number of children in this stack.
    #[must_use]
    pub fn children_count(&self) -> usize {
        let count = unsafe { ffi::otio_stack_children_count(self.ptr) };
        count.max(0) as usize
    }

    /// Iterate over children of this stack.
    ///
    /// Returns an iterator of `Composable` items (clips, gaps, stacks, tracks).
    pub fn children(&self) -> StackChildIter<'_> {
        StackChildIter::new(self.ptr)
    }

    /// Remove a child at the given index.
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds.
    pub fn remove_child(&mut self, index: usize) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_stack_remove_child(self.ptr, index as i32, &mut err) };
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Insert a track at the given index.
    ///
    /// # Errors
    ///
    /// Returns an error if the track cannot be inserted.
    #[allow(clippy::forget_non_drop)]
    pub fn insert_track(&mut self, index: usize, track: Track) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_stack_insert_track(self.ptr, index as i32, track.ptr, &mut err) };
        std::mem::forget(track);
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Insert a clip at the given index.
    ///
    /// # Errors
    ///
    /// Returns an error if the clip cannot be inserted.
    #[allow(clippy::forget_non_drop)]
    pub fn insert_clip(&mut self, index: usize, clip: Clip) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_stack_insert_clip(self.ptr, index as i32, clip.ptr, &mut err) };
        std::mem::forget(clip);
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Insert a gap at the given index.
    ///
    /// # Errors
    ///
    /// Returns an error if the gap cannot be inserted.
    #[allow(clippy::forget_non_drop)]
    pub fn insert_gap(&mut self, index: usize, gap: Gap) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_stack_insert_gap(self.ptr, index as i32, gap.ptr, &mut err) };
        std::mem::forget(gap);
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Insert a child stack at the given index.
    ///
    /// # Errors
    ///
    /// Returns an error if the stack cannot be inserted.
    #[allow(clippy::forget_non_drop)]
    pub fn insert_stack(&mut self, index: usize, child: Stack) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result =
            unsafe { ffi::otio_stack_insert_stack(self.ptr, index as i32, child.ptr, &mut err) };
        std::mem::forget(child);
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Clear all children from this stack.
    ///
    /// # Errors
    ///
    /// Returns an error if the children cannot be cleared.
    pub fn clear_children(&mut self) -> Result<()> {
        let mut err = ffi::OtioError {
            code: 0,
            message: [0; 256],
        };
        let result = unsafe { ffi::otio_stack_clear_children(self.ptr, &mut err) };
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }
}

traits::impl_has_metadata!(Stack, otio_stack_set_metadata_string, otio_stack_get_metadata_string);

impl Drop for Stack {
    fn drop(&mut self) {
        unsafe { ffi::otio_stack_free(self.ptr) }
    }
}

// Safety: Stack is safe to send between threads
unsafe impl Send for Stack {}
