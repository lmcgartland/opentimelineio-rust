//! Iteration support for OTIO compositions.
//!
//! This module provides types for iterating over children of Track and Stack,
//! as well as parent navigation and clip search functionality.

use std::marker::PhantomData;

use crate::ffi;
use crate::ffi_string_to_rust;
use crate::macros;
use crate::time_range_from_ffi;
use crate::{OtioError, RationalTime, Result, TimeRange};

/// Child type constants (must match C header defines)
const CHILD_TYPE_CLIP: i32 = 0;
const CHILD_TYPE_GAP: i32 = 1;
const CHILD_TYPE_STACK: i32 = 2;
const CHILD_TYPE_TRACK: i32 = 3;
const CHILD_TYPE_TRANSITION: i32 = 4;

/// Parent type constants (must match C header defines)
const PARENT_TYPE_TRACK: i32 = 1;
const PARENT_TYPE_STACK: i32 = 2;

/// Convert an FFI pointer and type to a Composable enum variant.
///
/// Returns `None` if the pointer is null or the type is unknown.
pub(crate) fn composable_from_ffi<'a>(
    ptr: *mut std::ffi::c_void,
    child_type: i32,
) -> Option<Composable<'a>> {
    if ptr.is_null() {
        return None;
    }
    match child_type {
        CHILD_TYPE_CLIP => Some(Composable::Clip(ClipRef::new(ptr.cast()))),
        CHILD_TYPE_GAP => Some(Composable::Gap(GapRef::new(ptr.cast()))),
        CHILD_TYPE_STACK => Some(Composable::Stack(StackRef::new(ptr.cast()))),
        CHILD_TYPE_TRACK => Some(Composable::Track(TrackRef::new(ptr.cast()))),
        CHILD_TYPE_TRANSITION => Some(Composable::Transition(TransitionRef::new(ptr.cast()))),
        _ => None,
    }
}

/// A composable child item from a Track or Stack.
///
/// This enum represents the different types of items that can be children
/// of a Track or Stack composition.
#[derive(Debug)]
pub enum Composable<'a> {
    /// A clip reference.
    Clip(ClipRef<'a>),
    /// A gap reference.
    Gap(GapRef<'a>),
    /// A nested stack reference.
    Stack(StackRef<'a>),
    /// A nested track reference.
    Track(TrackRef<'a>),
    /// A transition reference.
    Transition(TransitionRef<'a>),
}

/// A non-owning reference to a Clip.
///
/// This type is returned when iterating over children and does not own
/// the underlying memory (which is owned by the parent composition).
#[derive(Debug)]
pub struct ClipRef<'a> {
    ptr: *mut ffi::OtioClip,
    _marker: PhantomData<&'a ()>,
}

impl ClipRef<'_> {
    pub(crate) fn new(ptr: *mut ffi::OtioClip) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Get the name of this clip.
    #[must_use]
    pub fn name(&self) -> String {
        let ptr = unsafe { ffi::otio_clip_get_name(self.ptr) };
        ffi_string_to_rust(ptr)
    }

    /// Get the source range of this clip.
    #[must_use]
    pub fn source_range(&self) -> TimeRange {
        let range = unsafe { ffi::otio_clip_get_source_range(self.ptr) };
        time_range_from_ffi(&range)
    }

    /// Get the available range of this clip's media.
    ///
    /// This is the range of media that is available from the media reference,
    /// which may differ from the `source_range` (the portion actually used).
    ///
    /// # Errors
    ///
    /// Returns an error if the clip has no media reference or the range cannot
    /// be computed.
    pub fn available_range(&self) -> Result<TimeRange> {
        let mut err = macros::ffi_error!();
        let range = unsafe { ffi::otio_clip_available_range(self.ptr, &mut err) };
        if err.code != 0 {
            return Err(OtioError::from(err));
        }
        Ok(time_range_from_ffi(&range))
    }

    /// Get the parent composition of this clip.
    ///
    /// Returns `None` if the clip is not attached to a composition.
    #[must_use]
    pub fn parent(&self) -> Option<ParentRef<'_>> {
        get_clip_parent(self.ptr)
    }

    /// Get the range of this clip within its parent track.
    ///
    /// This returns the time range occupied by this clip in the parent's
    /// coordinate space.
    ///
    /// # Errors
    ///
    /// Returns an error if the clip has no parent or the range cannot be computed.
    pub fn range_in_parent(&self) -> Result<TimeRange> {
        let mut err = macros::ffi_error!();
        let range = unsafe { ffi::otio_clip_range_in_parent(self.ptr, &mut err) };
        if err.code != 0 {
            return Err(OtioError::from(err));
        }
        Ok(time_range_from_ffi(&range))
    }

    /// Transform a time from this clip's coordinate space to a target item's space.
    ///
    /// This is useful for converting times between different items in the timeline
    /// hierarchy. For example, converting a clip-local time to track time.
    ///
    /// # Arguments
    ///
    /// * `time` - The time in this clip's coordinate space
    /// * `to_track` - The target track reference
    ///
    /// # Errors
    ///
    /// Returns an error if the items are not related in the hierarchy.
    pub fn transformed_time_to_track(
        &self,
        time: RationalTime,
        to_track: &TrackRef<'_>,
    ) -> Result<RationalTime> {
        let mut err = macros::ffi_error!();
        let result = unsafe {
            ffi::otio_item_transformed_time(
                self.ptr.cast(),
                CHILD_TYPE_CLIP,
                time.into(),
                to_track.ptr.cast(),
                CHILD_TYPE_TRACK,
                &mut err,
            )
        };
        if err.code != 0 {
            return Err(OtioError::from(err));
        }
        Ok(RationalTime::new(result.value, result.rate))
    }

    /// Transform a time range from this clip's coordinate space to a target track's space.
    ///
    /// # Arguments
    ///
    /// * `range` - The time range in this clip's coordinate space
    /// * `to_track` - The target track reference
    ///
    /// # Errors
    ///
    /// Returns an error if the items are not related in the hierarchy.
    pub fn transformed_time_range_to_track(
        &self,
        range: TimeRange,
        to_track: &TrackRef<'_>,
    ) -> Result<TimeRange> {
        let mut err = macros::ffi_error!();
        let result = unsafe {
            ffi::otio_item_transformed_time_range(
                self.ptr.cast(),
                CHILD_TYPE_CLIP,
                range.into(),
                to_track.ptr.cast(),
                CHILD_TYPE_TRACK,
                &mut err,
            )
        };
        if err.code != 0 {
            return Err(OtioError::from(err));
        }
        Ok(time_range_from_ffi(&result))
    }
}

crate::traits::impl_has_metadata!(
    ClipRef<'_>,
    otio_clip_set_metadata_string,
    otio_clip_get_metadata_string
);

/// A non-owning reference to a Gap.
#[derive(Debug)]
pub struct GapRef<'a> {
    ptr: *mut ffi::OtioGap,
    _marker: PhantomData<&'a ()>,
}

impl GapRef<'_> {
    pub(crate) fn new(ptr: *mut ffi::OtioGap) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Get the name of this gap.
    #[must_use]
    pub fn name(&self) -> String {
        let ptr = unsafe { ffi::otio_gap_get_name(self.ptr) };
        ffi_string_to_rust(ptr)
    }

    /// Get the parent composition of this gap.
    ///
    /// Returns `None` if the gap is not attached to a composition.
    #[must_use]
    pub fn parent(&self) -> Option<ParentRef<'_>> {
        get_gap_parent(self.ptr)
    }

    /// Get the range of this gap within its parent track.
    ///
    /// This returns the time range occupied by this gap in the parent's
    /// coordinate space.
    ///
    /// # Errors
    ///
    /// Returns an error if the gap has no parent or the range cannot be computed.
    pub fn range_in_parent(&self) -> Result<TimeRange> {
        let mut err = macros::ffi_error!();
        let range = unsafe { ffi::otio_gap_range_in_parent(self.ptr, &mut err) };
        if err.code != 0 {
            return Err(OtioError::from(err));
        }
        Ok(time_range_from_ffi(&range))
    }
}

crate::traits::impl_has_metadata!(
    GapRef<'_>,
    otio_gap_set_metadata_string,
    otio_gap_get_metadata_string
);

/// A non-owning reference to a Transition.
#[derive(Debug)]
pub struct TransitionRef<'a> {
    ptr: *mut ffi::OtioTransition,
    _marker: PhantomData<&'a ()>,
}

impl TransitionRef<'_> {
    pub(crate) fn new(ptr: *mut ffi::OtioTransition) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Get the name of this transition.
    #[must_use]
    pub fn name(&self) -> String {
        let ptr = unsafe { ffi::otio_transition_get_name(self.ptr) };
        ffi_string_to_rust(ptr)
    }

    /// Get the transition type.
    #[must_use]
    pub fn transition_type(&self) -> String {
        let ptr = unsafe { ffi::otio_transition_get_transition_type(self.ptr) };
        ffi_string_to_rust(ptr)
    }

    /// Get the in offset (overlap into outgoing clip).
    #[must_use]
    pub fn in_offset(&self) -> RationalTime {
        let rt = unsafe { ffi::otio_transition_get_in_offset(self.ptr) };
        RationalTime::new(rt.value, rt.rate)
    }

    /// Get the out offset (overlap into incoming clip).
    #[must_use]
    pub fn out_offset(&self) -> RationalTime {
        let rt = unsafe { ffi::otio_transition_get_out_offset(self.ptr) };
        RationalTime::new(rt.value, rt.rate)
    }

    /// Get the total duration of the transition.
    #[must_use]
    pub fn duration(&self) -> RationalTime {
        let rt = unsafe { ffi::otio_transition_get_duration(self.ptr) };
        RationalTime::new(rt.value, rt.rate)
    }
}

crate::traits::impl_has_metadata!(
    TransitionRef<'_>,
    otio_transition_set_metadata_string,
    otio_transition_get_metadata_string
);

/// A non-owning reference to a Stack.
#[derive(Debug)]
pub struct StackRef<'a> {
    pub(crate) ptr: *mut ffi::OtioStack,
    pub(crate) _marker: PhantomData<&'a ()>,
}

impl StackRef<'_> {
    pub(crate) fn new(ptr: *mut ffi::OtioStack) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Get the raw pointer to the stack.
    #[must_use]
    pub fn as_ptr(&self) -> *mut ffi::OtioStack {
        self.ptr
    }

    /// Get the name of this stack.
    #[must_use]
    pub fn name(&self) -> String {
        let ptr = unsafe { ffi::otio_stack_get_name(self.ptr) };
        ffi_string_to_rust(ptr)
    }

    /// Get the number of children in this stack.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn children_count(&self) -> usize {
        let count = unsafe { ffi::otio_stack_children_count(self.ptr) };
        count.max(0) as usize
    }

    /// Iterate over children of this stack.
    #[must_use]
    pub fn children(&self) -> StackChildIter<'_> {
        StackChildIter::new(self.ptr)
    }
}

crate::traits::impl_has_metadata!(
    StackRef<'_>,
    otio_stack_set_metadata_string,
    otio_stack_get_metadata_string
);

/// A non-owning reference to a Track.
#[derive(Debug)]
pub struct TrackRef<'a> {
    ptr: *mut ffi::OtioTrack,
    _marker: PhantomData<&'a ()>,
}

impl TrackRef<'_> {
    pub(crate) fn new(ptr: *mut ffi::OtioTrack) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Get the name of this track.
    #[must_use]
    pub fn name(&self) -> String {
        let ptr = unsafe { ffi::otio_track_get_name(self.ptr) };
        ffi_string_to_rust(ptr)
    }

    /// Get the number of children in this track.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn children_count(&self) -> usize {
        let count = unsafe { ffi::otio_track_children_count(self.ptr) };
        count.max(0) as usize
    }

    /// Iterate over children of this track.
    #[must_use]
    pub fn children(&self) -> TrackChildIter<'_> {
        TrackChildIter::new(self.ptr)
    }

    /// Get the parent stack of this track.
    ///
    /// Returns `None` if the track is not attached to a stack.
    #[must_use]
    pub fn parent(&self) -> Option<StackRef<'_>> {
        get_track_parent(self.ptr)
    }

    /// Get the kind of this track (video or audio).
    #[must_use]
    pub fn kind(&self) -> crate::TrackKind {
        let kind = unsafe { ffi::otio_track_get_kind(self.ptr) };
        if kind == 1 {
            crate::TrackKind::Audio
        } else {
            crate::TrackKind::Video
        }
    }
}

crate::traits::impl_has_metadata!(
    TrackRef<'_>,
    otio_track_set_metadata_string,
    otio_track_get_metadata_string
);

/// Iterator over Track children.
pub struct TrackChildIter<'a> {
    ptr: *mut ffi::OtioTrack,
    index: i32,
    count: i32,
    _marker: PhantomData<&'a ()>,
}

impl TrackChildIter<'_> {
    pub(crate) fn new(ptr: *mut ffi::OtioTrack) -> Self {
        let count = unsafe { ffi::otio_track_children_count(ptr) };
        Self {
            ptr,
            index: 0,
            count,
            _marker: PhantomData,
        }
    }
}

impl<'a> Iterator for TrackChildIter<'a> {
    type Item = Composable<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        let child_type = unsafe { ffi::otio_track_child_type(self.ptr, self.index) };
        let child_ptr = unsafe { ffi::otio_track_child_at(self.ptr, self.index) };

        self.index += 1;

        if child_ptr.is_null() {
            return self.next(); // Skip null children
        }

        match child_type {
            CHILD_TYPE_CLIP => Some(Composable::Clip(ClipRef::new(child_ptr.cast()))),
            CHILD_TYPE_GAP => Some(Composable::Gap(GapRef::new(child_ptr.cast()))),
            CHILD_TYPE_STACK => Some(Composable::Stack(StackRef::new(child_ptr.cast()))),
            CHILD_TYPE_TRACK => Some(Composable::Track(TrackRef::new(child_ptr.cast()))),
            CHILD_TYPE_TRANSITION => {
                Some(Composable::Transition(TransitionRef::new(child_ptr.cast())))
            }
            _ => self.next(), // Skip unknown types
        }
    }

    #[allow(clippy::cast_sign_loss)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.index).max(0) as usize;
        (0, Some(remaining))
    }
}

/// Iterator over Stack children.
pub struct StackChildIter<'a> {
    ptr: *mut ffi::OtioStack,
    index: i32,
    count: i32,
    _marker: PhantomData<&'a ()>,
}

impl StackChildIter<'_> {
    pub(crate) fn new(ptr: *mut ffi::OtioStack) -> Self {
        let count = unsafe { ffi::otio_stack_children_count(ptr) };
        Self {
            ptr,
            index: 0,
            count,
            _marker: PhantomData,
        }
    }
}

impl<'a> Iterator for StackChildIter<'a> {
    type Item = Composable<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        let child_type = unsafe { ffi::otio_stack_child_type(self.ptr, self.index) };
        let child_ptr = unsafe { ffi::otio_stack_child_at(self.ptr, self.index) };

        self.index += 1;

        if child_ptr.is_null() {
            return self.next(); // Skip null children
        }

        match child_type {
            CHILD_TYPE_CLIP => Some(Composable::Clip(ClipRef::new(child_ptr.cast()))),
            CHILD_TYPE_GAP => Some(Composable::Gap(GapRef::new(child_ptr.cast()))),
            CHILD_TYPE_STACK => Some(Composable::Stack(StackRef::new(child_ptr.cast()))),
            CHILD_TYPE_TRACK => Some(Composable::Track(TrackRef::new(child_ptr.cast()))),
            CHILD_TYPE_TRANSITION => {
                Some(Composable::Transition(TransitionRef::new(child_ptr.cast())))
            }
            _ => self.next(), // Skip unknown types
        }
    }

    #[allow(clippy::cast_sign_loss)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.index).max(0) as usize;
        (0, Some(remaining))
    }
}

// =============================================================================
// Parent Navigation
// =============================================================================

/// A reference to a parent composition.
///
/// Items in OTIO (clips, gaps, transitions) can have a parent which is either
/// a Track or a Stack. This enum represents a non-owning reference to that parent.
#[derive(Debug)]
pub enum ParentRef<'a> {
    /// The parent is a Track.
    Track(TrackRef<'a>),
    /// The parent is a Stack.
    Stack(StackRef<'a>),
}

/// Helper to get parent from a clip pointer.
pub(crate) fn get_clip_parent(ptr: *mut ffi::OtioClip) -> Option<ParentRef<'static>> {
    let parent_type = unsafe { ffi::otio_clip_get_parent_type(ptr) };
    match parent_type {
        PARENT_TYPE_TRACK => {
            let parent_ptr = unsafe { ffi::otio_clip_get_parent(ptr) };
            if parent_ptr.is_null() {
                None
            } else {
                Some(ParentRef::Track(TrackRef::new(parent_ptr.cast())))
            }
        }
        PARENT_TYPE_STACK => {
            let parent_ptr = unsafe { ffi::otio_clip_get_parent(ptr) };
            if parent_ptr.is_null() {
                None
            } else {
                Some(ParentRef::Stack(StackRef::new(parent_ptr.cast())))
            }
        }
        _ => None,
    }
}

/// Helper to get parent from a gap pointer.
pub(crate) fn get_gap_parent(ptr: *mut ffi::OtioGap) -> Option<ParentRef<'static>> {
    let parent_type = unsafe { ffi::otio_gap_get_parent_type(ptr) };
    match parent_type {
        PARENT_TYPE_TRACK => {
            let parent_ptr = unsafe { ffi::otio_gap_get_parent(ptr) };
            if parent_ptr.is_null() {
                None
            } else {
                Some(ParentRef::Track(TrackRef::new(parent_ptr.cast())))
            }
        }
        PARENT_TYPE_STACK => {
            let parent_ptr = unsafe { ffi::otio_gap_get_parent(ptr) };
            if parent_ptr.is_null() {
                None
            } else {
                Some(ParentRef::Stack(StackRef::new(parent_ptr.cast())))
            }
        }
        _ => None,
    }
}

/// Helper to get parent from a track pointer.
pub(crate) fn get_track_parent(ptr: *mut ffi::OtioTrack) -> Option<StackRef<'static>> {
    let parent_type = unsafe { ffi::otio_track_get_parent_type(ptr) };
    if parent_type == PARENT_TYPE_STACK {
        let parent_ptr = unsafe { ffi::otio_track_get_parent(ptr) };
        if !parent_ptr.is_null() {
            return Some(StackRef::new(parent_ptr.cast()));
        }
    }
    None
}

/// Helper to get parent from a stack pointer.
pub(crate) fn get_stack_parent(ptr: *mut ffi::OtioStack) -> Option<StackRef<'static>> {
    let parent_type = unsafe { ffi::otio_stack_get_parent_type(ptr) };
    if parent_type == PARENT_TYPE_STACK {
        let parent_ptr = unsafe { ffi::otio_stack_get_parent(ptr) };
        if !parent_ptr.is_null() {
            return Some(StackRef::new(parent_ptr.cast()));
        }
    }
    None
}

// =============================================================================
// Clip Search Iterator
// =============================================================================

/// An iterator over clips found in a composition.
///
/// This iterator is created by calling `find_clips()` on a Track, Stack, or Timeline.
/// It iterates over all clips found in the composition (recursively for Stack/Timeline).
pub struct ClipSearchIter<'a> {
    ptr: *mut ffi::OtioClipIterator,
    _marker: PhantomData<&'a ()>,
}

impl ClipSearchIter<'_> {
    /// Create a new clip search iterator from a raw pointer.
    pub(crate) fn new(ptr: *mut ffi::OtioClipIterator) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Get the total number of clips found.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn count(&self) -> usize {
        if self.ptr.is_null() {
            0
        } else {
            unsafe { ffi::otio_clip_iterator_count(self.ptr) }.max(0) as usize
        }
    }

    /// Reset the iterator to the beginning.
    pub fn reset(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::otio_clip_iterator_reset(self.ptr) };
        }
    }
}

impl<'a> Iterator for ClipSearchIter<'a> {
    type Item = ClipRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }
        let clip_ptr = unsafe { ffi::otio_clip_iterator_next(self.ptr) };
        if clip_ptr.is_null() {
            None
        } else {
            Some(ClipRef::new(clip_ptr))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.count();
        (0, Some(count))
    }
}

impl Drop for ClipSearchIter<'_> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::otio_clip_iterator_free(self.ptr) };
        }
    }
}

// ============================================================================
// Track Iterator (for video_tracks / audio_tracks)
// ============================================================================

/// An iterator over tracks returned by [`Timeline::video_tracks`] or
/// [`Timeline::audio_tracks`].
pub struct TrackIter<'a> {
    ptr: *mut ffi::OtioTrackIterator,
    _marker: PhantomData<&'a ()>,
}

impl TrackIter<'_> {
    /// Create a new track iterator from a raw pointer.
    pub(crate) fn new(ptr: *mut ffi::OtioTrackIterator) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Get the total number of tracks.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn count(&self) -> usize {
        if self.ptr.is_null() {
            0
        } else {
            unsafe { ffi::otio_track_iterator_count(self.ptr) }.max(0) as usize
        }
    }

    /// Reset the iterator to the beginning.
    pub fn reset(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::otio_track_iterator_reset(self.ptr) };
        }
    }
}

impl<'a> Iterator for TrackIter<'a> {
    type Item = TrackRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }
        let track_ptr = unsafe { ffi::otio_track_iterator_next(self.ptr) };
        if track_ptr.is_null() {
            None
        } else {
            Some(TrackRef {
                ptr: track_ptr,
                _marker: PhantomData,
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.count();
        (0, Some(count))
    }
}

impl ExactSizeIterator for TrackIter<'_> {
    fn len(&self) -> usize {
        self.count()
    }
}

impl Drop for TrackIter<'_> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::otio_track_iterator_free(self.ptr) };
        }
    }
}
