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
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod macros;
mod traits;
pub use traits::HasMetadata;

mod types;
pub use types::*;

mod iterators;
pub use iterators::{
    ClipRef, ClipSearchIter, Composable, GapRef, ParentRef, StackChildIter, StackRef,
    TrackChildIter, TrackRef, TransitionRef,
};

mod builders;
pub use builders::{ClipBuilder, ExternalReferenceBuilder, TimelineBuilder};

pub mod marker;
pub use marker::Marker;

mod effect;
pub use effect::Effect;

pub mod transition;
pub use transition::Transition;

mod missing_reference;
pub use missing_reference::MissingReference;

pub mod generator_reference;
pub use generator_reference::GeneratorReference;

pub mod image_sequence_reference;
pub use image_sequence_reference::ImageSequenceReference;

mod time_effect;
pub use time_effect::{FreezeFrame, LinearTimeWarp};

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
        let mut err = macros::ffi_error!();
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
        let mut err = macros::ffi_error!();
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
        let mut err = macros::ffi_error!();
        let ptr = unsafe { ffi::otio_timeline_read_from_file(c_path.as_ptr(), &mut err) };
        if ptr.is_null() {
            Err(err.into())
        } else {
            Ok(Self { ptr })
        }
    }

    /// Serialize this timeline to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the timeline cannot be serialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use otio_rs::Timeline;
    ///
    /// let timeline = Timeline::new("My Timeline");
    /// let json = timeline.to_json_string().unwrap();
    /// println!("Timeline JSON: {}", json);
    /// ```
    pub fn to_json_string(&self) -> Result<String> {
        let mut err = macros::ffi_error!();
        let ptr = unsafe { ffi::otio_timeline_to_json_string(self.ptr, &mut err) };
        if ptr.is_null() {
            return Err(err.into());
        }
        let result = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
        unsafe { ffi::otio_free_string(ptr) };
        Ok(result)
    }

    /// Deserialize a timeline from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON cannot be parsed or doesn't contain a timeline.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use otio_rs::Timeline;
    ///
    /// let json = r#"{"OTIO_SCHEMA": "Timeline.1", "name": "Test"}"#;
    /// let timeline = Timeline::from_json_string(json).unwrap();
    /// ```
    pub fn from_json_string(json: &str) -> Result<Self> {
        let c_json = CString::new(json).unwrap();
        let mut err = macros::ffi_error!();
        let ptr = unsafe { ffi::otio_timeline_from_json_string(c_json.as_ptr(), &mut err) };
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

    /// Get the name of this timeline.
    #[must_use]
    pub fn name(&self) -> String {
        let ptr = unsafe { ffi::otio_timeline_get_name(self.ptr) };
        if ptr.is_null() {
            return String::new();
        }
        let result = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
        unsafe { ffi::otio_free_string(ptr) };
        result
    }

    /// Get the global start time of this timeline.
    ///
    /// Returns `None` if no global start time has been set.
    #[must_use]
    #[allow(clippy::float_cmp)] // Sentinel value comparison is intentional
    pub fn global_start_time(&self) -> Option<RationalTime> {
        let rt = unsafe { ffi::otio_timeline_get_global_start_time(self.ptr) };
        // A zero rate indicates no value was set
        if rt.rate == 1.0 && rt.value == 0.0 {
            return None;
        }
        Some(RationalTime::new(rt.value, rt.rate))
    }

    /// Get the duration of this timeline.
    ///
    /// The duration is computed from the timeline's tracks.
    ///
    /// # Errors
    ///
    /// Returns an error if the duration cannot be computed.
    pub fn duration(&self) -> Result<RationalTime> {
        let mut err = macros::ffi_error!();
        let range = unsafe { ffi::otio_timeline_get_duration(self.ptr, &mut err) };
        if err.code != 0 {
            return Err(err.into());
        }
        Ok(RationalTime::new(range.duration.value, range.duration.rate))
    }

    /// Find all clips in this timeline (recursively).
    ///
    /// Returns an iterator over all clips found in the timeline's tracks
    /// and any nested compositions.
    #[must_use]
    pub fn find_clips(&self) -> ClipSearchIter<'_> {
        let ptr = unsafe { ffi::otio_timeline_find_clips(self.ptr) };
        ClipSearchIter::new(ptr)
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

    // Child operations generated by macro
    macros::impl_track_ops!();

    /// Iterate over children of this track.
    ///
    /// Returns an iterator of `Composable` items (clips, gaps, stacks).
    #[must_use]
    pub fn children(&self) -> TrackChildIter<'_> {
        TrackChildIter::new(self.ptr)
    }

    /// Get the kind of this track (video or audio).
    #[must_use]
    pub fn kind(&self) -> TrackKind {
        let kind = unsafe { ffi::otio_track_get_kind(self.ptr) };
        if kind == 1 {
            TrackKind::Audio
        } else {
            TrackKind::Video
        }
    }

    /// Set the kind of this track.
    pub fn set_kind(&mut self, kind: TrackKind) {
        let kind_val = match kind {
            TrackKind::Video => 0,
            TrackKind::Audio => 1,
        };
        unsafe { ffi::otio_track_set_kind(self.ptr, kind_val) };
    }

    /// Add a marker to this track.
    ///
    /// # Errors
    ///
    /// Returns an error if the marker cannot be added.
    #[allow(clippy::forget_non_drop)]
    pub fn add_marker(&mut self, marker: Marker) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe { ffi::otio_track_add_marker(self.ptr, marker.ptr, &mut err) };
        if result != 0 {
            return Err(err.into());
        }
        std::mem::forget(marker);
        Ok(())
    }

    /// Get the number of markers on this track.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn markers_count(&self) -> usize {
        let count = unsafe { ffi::otio_track_markers_count(self.ptr) };
        count.max(0) as usize
    }

    /// Get the range of a child at the given index within this track.
    ///
    /// This returns the time range of the child relative to the track's
    /// start time, taking into account all preceding children.
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds.
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn range_of_child_at_index(&self, index: usize) -> Result<TimeRange> {
        let mut err = macros::ffi_error!();
        let range = unsafe {
            ffi::otio_track_range_of_child_at_index(self.ptr, index as i32, &mut err)
        };
        if err.code != 0 {
            return Err(err.into());
        }
        Ok(TimeRange::new(
            RationalTime::new(range.start_time.value, range.start_time.rate),
            RationalTime::new(range.duration.value, range.duration.rate),
        ))
    }

    /// Get the trimmed range of this track.
    ///
    /// The trimmed range is computed from the children of the track.
    ///
    /// # Errors
    ///
    /// Returns an error if the range cannot be computed.
    pub fn trimmed_range(&self) -> Result<TimeRange> {
        let mut err = macros::ffi_error!();
        let range = unsafe { ffi::otio_track_trimmed_range(self.ptr, &mut err) };
        if err.code != 0 {
            return Err(err.into());
        }
        Ok(TimeRange::new(
            RationalTime::new(range.start_time.value, range.start_time.rate),
            RationalTime::new(range.duration.value, range.duration.rate),
        ))
    }

    /// Get the parent stack of this track.
    ///
    /// Returns `None` if the track is not attached to a stack.
    #[must_use]
    pub fn parent(&self) -> Option<StackRef<'_>> {
        iterators::get_track_parent(self.ptr)
    }

    /// Find all clips in this track.
    ///
    /// Returns an iterator over all clips that are direct children of this track.
    /// For a recursive search through nested compositions, use `find_clips()` on
    /// the containing Stack or Timeline instead.
    #[must_use]
    pub fn find_clips(&self) -> ClipSearchIter<'_> {
        let ptr = unsafe { ffi::otio_track_find_clips(self.ptr) };
        ClipSearchIter::new(ptr)
    }

    // =========================================================================
    // Edit Algorithms
    // =========================================================================

    /// Overwrite content in this track at the specified range with a new clip.
    ///
    /// This is equivalent to a 3-point edit in NLE software. The clip is placed
    /// at the specified range, replacing any existing content.
    ///
    /// # Arguments
    ///
    /// * `clip` - The clip to insert (ownership transfers to the track)
    /// * `range` - The time range to overwrite
    /// * `remove_transitions` - Whether to remove transitions that intersect the range
    ///
    /// # Errors
    ///
    /// Returns an error if the overwrite operation fails.
    #[allow(clippy::forget_non_drop)]
    pub fn overwrite(
        &mut self,
        clip: Clip,
        range: TimeRange,
        remove_transitions: bool,
    ) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe {
            ffi::otio_track_overwrite(
                self.ptr,
                clip.ptr,
                range.into(),
                i32::from(remove_transitions),
                &mut err,
            )
        };
        if result != 0 {
            return Err(err.into());
        }
        std::mem::forget(clip);
        Ok(())
    }

    /// Insert a clip at a specific time, shifting subsequent items.
    ///
    /// This splits any item at the insertion point and pushes all subsequent
    /// items later in the track to make room for the new clip.
    ///
    /// # Arguments
    ///
    /// * `clip` - The clip to insert (ownership transfers to the track)
    /// * `time` - The time at which to insert
    /// * `remove_transitions` - Whether to remove transitions that intersect the time
    ///
    /// # Errors
    ///
    /// Returns an error if the insert operation fails.
    #[allow(clippy::forget_non_drop)]
    pub fn insert_at_time(
        &mut self,
        clip: Clip,
        time: RationalTime,
        remove_transitions: bool,
    ) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe {
            ffi::otio_track_insert_at_time(
                self.ptr,
                clip.ptr,
                time.into(),
                i32::from(remove_transitions),
                &mut err,
            )
        };
        if result != 0 {
            return Err(err.into());
        }
        std::mem::forget(clip);
        Ok(())
    }

    /// Slice (split) the track at a specific time point.
    ///
    /// This creates a cut at the specified time, splitting any item that
    /// spans that point into two items.
    ///
    /// # Arguments
    ///
    /// * `time` - The time at which to slice
    /// * `remove_transitions` - Whether to remove transitions that intersect the time
    ///
    /// # Errors
    ///
    /// Returns an error if the slice operation fails.
    pub fn slice_at_time(&mut self, time: RationalTime, remove_transitions: bool) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe {
            ffi::otio_track_slice_at_time(
                self.ptr,
                time.into(),
                i32::from(remove_transitions),
                &mut err,
            )
        };
        if result != 0 {
            return Err(err.into());
        }
        Ok(())
    }

    /// Remove the item at a specific time.
    ///
    /// # Arguments
    ///
    /// * `time` - The time at which to remove
    /// * `fill_with_gap` - If true, fills the removed space with a gap; otherwise
    ///   subsequent items are concatenated
    ///
    /// # Errors
    ///
    /// Returns an error if the remove operation fails.
    pub fn remove_at_time(&mut self, time: RationalTime, fill_with_gap: bool) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe {
            ffi::otio_track_remove_at_time(
                self.ptr,
                time.into(),
                i32::from(fill_with_gap),
                &mut err,
            )
        };
        if result != 0 {
            return Err(err.into());
        }
        Ok(())
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
        let mut err = macros::ffi_error!();
        let result =
            unsafe { ffi::otio_clip_set_media_reference(self.ptr, reference.ptr, &mut err) };
        if result != 0 {
            return Err(err.into());
        }
        std::mem::forget(reference); // Clip now owns the reference - only forget on success
        Ok(())
    }

    /// Set a missing reference for this clip (for offline/placeholder clips).
    ///
    /// # Errors
    ///
    /// Returns an error if the reference cannot be set.
    #[allow(clippy::forget_non_drop)]
    pub fn set_missing_reference(&mut self, reference: MissingReference) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result =
            unsafe { ffi::otio_clip_set_missing_reference(self.ptr, reference.ptr, &mut err) };
        if result != 0 {
            return Err(err.into());
        }
        std::mem::forget(reference);
        Ok(())
    }

    /// Set a generator reference for this clip (for generated content).
    ///
    /// # Errors
    ///
    /// Returns an error if the reference cannot be set.
    #[allow(clippy::forget_non_drop)]
    pub fn set_generator_reference(&mut self, reference: GeneratorReference) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result =
            unsafe { ffi::otio_clip_set_generator_reference(self.ptr, reference.ptr, &mut err) };
        if result != 0 {
            return Err(err.into());
        }
        std::mem::forget(reference);
        Ok(())
    }

    /// Set an image sequence reference for this clip (for VFX image sequences).
    ///
    /// # Errors
    ///
    /// Returns an error if the reference cannot be set.
    #[allow(clippy::forget_non_drop)]
    pub fn set_image_sequence_reference(
        &mut self,
        reference: ImageSequenceReference,
    ) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe {
            ffi::otio_clip_set_image_sequence_reference(self.ptr, reference.ptr, &mut err)
        };
        if result != 0 {
            return Err(err.into());
        }
        std::mem::forget(reference);
        Ok(())
    }

    /// Add a marker to this clip.
    ///
    /// # Errors
    ///
    /// Returns an error if the marker cannot be added.
    #[allow(clippy::forget_non_drop)]
    pub fn add_marker(&mut self, marker: Marker) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe { ffi::otio_clip_add_marker(self.ptr, marker.ptr, &mut err) };
        if result != 0 {
            return Err(err.into());
        }
        std::mem::forget(marker);
        Ok(())
    }

    /// Get the number of markers on this clip.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn markers_count(&self) -> usize {
        let count = unsafe { ffi::otio_clip_markers_count(self.ptr) };
        count.max(0) as usize
    }

    /// Add an effect to this clip.
    ///
    /// # Errors
    ///
    /// Returns an error if the effect cannot be added.
    #[allow(clippy::forget_non_drop)]
    pub fn add_effect(&mut self, effect: Effect) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe { ffi::otio_clip_add_effect(self.ptr, effect.ptr, &mut err) };
        if result != 0 {
            return Err(err.into());
        }
        std::mem::forget(effect);
        Ok(())
    }

    /// Add a linear time warp effect to this clip.
    ///
    /// # Errors
    ///
    /// Returns an error if the effect cannot be added.
    #[allow(clippy::forget_non_drop)]
    pub fn add_linear_time_warp(&mut self, effect: LinearTimeWarp) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe { ffi::otio_clip_add_linear_time_warp(self.ptr, effect.ptr, &mut err) };
        if result != 0 {
            return Err(err.into());
        }
        std::mem::forget(effect);
        Ok(())
    }

    /// Get the number of effects on this clip.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn effects_count(&self) -> usize {
        let count = unsafe { ffi::otio_clip_effects_count(self.ptr) };
        count.max(0) as usize
    }

    // =========================================================================
    // Edit Algorithms
    // =========================================================================

    /// Slip the clip's media content by a time delta.
    ///
    /// Slipping adjusts which portion of the source media is shown without
    /// changing the clip's position or duration in the track. The media
    /// "slides" under the clip boundaries.
    ///
    /// # Arguments
    ///
    /// * `delta` - The time amount to slip (positive = later in source, negative = earlier)
    ///
    /// # Errors
    ///
    /// Returns an error if the slip operation fails.
    pub fn slip(&mut self, delta: RationalTime) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe { ffi::otio_clip_slip(self.ptr, delta.into(), &mut err) };
        if result != 0 {
            return Err(err.into());
        }
        Ok(())
    }

    /// Slide the clip's position in the track.
    ///
    /// Sliding moves the clip earlier or later in the track, adjusting the
    /// duration of the previous item to compensate. The clip's content and
    /// duration remain unchanged.
    ///
    /// # Arguments
    ///
    /// * `delta` - The time amount to slide (positive = later, negative = earlier)
    ///
    /// # Errors
    ///
    /// Returns an error if the slide operation fails.
    pub fn slide(&mut self, delta: RationalTime) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe { ffi::otio_clip_slide(self.ptr, delta.into(), &mut err) };
        if result != 0 {
            return Err(err.into());
        }
        Ok(())
    }

    /// Trim the clip's in and out points.
    ///
    /// Trimming adjusts the source range boundaries without affecting other
    /// clips in the track. Empty space created is filled with a gap.
    ///
    /// # Arguments
    ///
    /// * `delta_in` - Adjustment to the in point (positive = trim later, negative = extend earlier)
    /// * `delta_out` - Adjustment to the out point (positive = extend later, negative = trim earlier)
    ///
    /// # Errors
    ///
    /// Returns an error if the trim operation fails.
    pub fn trim(&mut self, delta_in: RationalTime, delta_out: RationalTime) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe {
            ffi::otio_clip_trim(self.ptr, delta_in.into(), delta_out.into(), &mut err)
        };
        if result != 0 {
            return Err(err.into());
        }
        Ok(())
    }

    /// Ripple edit the clip's duration.
    ///
    /// Rippling adjusts the clip's duration and propagates the change through
    /// the rest of the track - subsequent clips move to accommodate the change.
    ///
    /// # Arguments
    ///
    /// * `delta_in` - Adjustment to the in point
    /// * `delta_out` - Adjustment to the out point
    ///
    /// # Errors
    ///
    /// Returns an error if the ripple operation fails.
    pub fn ripple(&mut self, delta_in: RationalTime, delta_out: RationalTime) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result = unsafe {
            ffi::otio_clip_ripple(self.ptr, delta_in.into(), delta_out.into(), &mut err)
        };
        if result != 0 {
            return Err(err.into());
        }
        Ok(())
    }

    /// Roll the edit point between this clip and adjacent clips.
    ///
    /// Rolling moves the edit point between clips without changing the
    /// overall duration of the combined clips. One clip gets longer while
    /// the adjacent clip gets shorter.
    ///
    /// # Arguments
    ///
    /// * `delta_in` - Adjustment to the in point (affects previous clip's out)
    /// * `delta_out` - Adjustment to the out point (affects next clip's in)
    ///
    /// # Errors
    ///
    /// Returns an error if the roll operation fails.
    pub fn roll(&mut self, delta_in: RationalTime, delta_out: RationalTime) -> Result<()> {
        let mut err = macros::ffi_error!();
        let result =
            unsafe { ffi::otio_clip_roll(self.ptr, delta_in.into(), delta_out.into(), &mut err) };
        if result != 0 {
            return Err(err.into());
        }
        Ok(())
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
        let mut err = macros::ffi_error!();
        let result =
            unsafe { ffi::otio_external_ref_set_available_range(self.ptr, range.into(), &mut err) };
        if result != 0 {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Get the target URL of this media reference.
    #[must_use]
    pub fn target_url(&self) -> String {
        let ptr = unsafe { ffi::otio_external_ref_get_target_url(self.ptr) };
        if ptr.is_null() {
            return String::new();
        }
        let result = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
        unsafe { ffi::otio_free_string(ptr) };
        result
    }

    /// Get the available range of this media reference.
    ///
    /// Returns `None` if no available range has been set.
    #[must_use]
    #[allow(clippy::float_cmp)] // Sentinel value comparison is intentional
    pub fn available_range(&self) -> Option<TimeRange> {
        let range = unsafe { ffi::otio_external_ref_get_available_range(self.ptr) };
        // Check for zero range (no range set)
        if range.duration.value == 0.0 && range.duration.rate == 1.0 {
            return None;
        }
        Some(TimeRange::new(
            RationalTime::new(range.start_time.value, range.start_time.rate),
            RationalTime::new(range.duration.value, range.duration.rate),
        ))
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

    // Child operations generated by macro
    macros::impl_stack_ops!();

    /// Iterate over children of this stack.
    ///
    /// Returns an iterator of `Composable` items (clips, gaps, stacks, tracks).
    #[must_use]
    pub fn children(&self) -> StackChildIter<'_> {
        StackChildIter::new(self.ptr)
    }

    /// Get the range of a child at the given index within this stack.
    ///
    /// For stacks, all children typically start at the same time (they layer
    /// rather than sequence like tracks).
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds.
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn range_of_child_at_index(&self, index: usize) -> Result<TimeRange> {
        let mut err = macros::ffi_error!();
        let range = unsafe {
            ffi::otio_stack_range_of_child_at_index(self.ptr, index as i32, &mut err)
        };
        if err.code != 0 {
            return Err(err.into());
        }
        Ok(TimeRange::new(
            RationalTime::new(range.start_time.value, range.start_time.rate),
            RationalTime::new(range.duration.value, range.duration.rate),
        ))
    }

    /// Get the trimmed range of this stack.
    ///
    /// The trimmed range is the union of all children's ranges.
    ///
    /// # Errors
    ///
    /// Returns an error if the range cannot be computed.
    pub fn trimmed_range(&self) -> Result<TimeRange> {
        let mut err = macros::ffi_error!();
        let range = unsafe { ffi::otio_stack_trimmed_range(self.ptr, &mut err) };
        if err.code != 0 {
            return Err(err.into());
        }
        Ok(TimeRange::new(
            RationalTime::new(range.start_time.value, range.start_time.rate),
            RationalTime::new(range.duration.value, range.duration.rate),
        ))
    }

    /// Get the parent stack of this stack.
    ///
    /// Returns `None` if this stack is not nested within another stack.
    #[must_use]
    pub fn parent(&self) -> Option<StackRef<'_>> {
        iterators::get_stack_parent(self.ptr)
    }

    /// Find all clips in this stack (recursively).
    ///
    /// Returns an iterator over all clips found in this stack and its nested
    /// compositions (tracks and nested stacks).
    #[must_use]
    pub fn find_clips(&self) -> ClipSearchIter<'_> {
        let ptr = unsafe { ffi::otio_stack_find_clips(self.ptr) };
        ClipSearchIter::new(ptr)
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
