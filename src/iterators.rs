//! Iteration support for OTIO compositions.
//!
//! This module provides types for iterating over children of Track and Stack.

use std::ffi::CStr;
use std::marker::PhantomData;

use crate::ffi;
use crate::{RationalTime, TimeRange};

/// Child type constants (must match C header defines)
const CHILD_TYPE_CLIP: i32 = 0;
const CHILD_TYPE_GAP: i32 = 1;
const CHILD_TYPE_STACK: i32 = 2;
const CHILD_TYPE_TRACK: i32 = 3;

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
        if ptr.is_null() {
            String::new()
        } else {
            let result = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
            unsafe { ffi::otio_free_string(ptr) };
            result
        }
    }

    /// Get the source range of this clip.
    #[must_use]
    pub fn source_range(&self) -> TimeRange {
        let range = unsafe { ffi::otio_clip_get_source_range(self.ptr) };
        TimeRange {
            start_time: RationalTime {
                value: range.start_time.value,
                rate: range.start_time.rate,
            },
            duration: RationalTime {
                value: range.duration.value,
                rate: range.duration.rate,
            },
        }
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
        if ptr.is_null() {
            String::new()
        } else {
            let result = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
            unsafe { ffi::otio_free_string(ptr) };
            result
        }
    }
}

crate::traits::impl_has_metadata!(
    GapRef<'_>,
    otio_gap_set_metadata_string,
    otio_gap_get_metadata_string
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
        if ptr.is_null() {
            String::new()
        } else {
            let result = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
            unsafe { ffi::otio_free_string(ptr) };
            result
        }
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
        if ptr.is_null() {
            String::new()
        } else {
            let result = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
            unsafe { ffi::otio_free_string(ptr) };
            result
        }
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
            _ => self.next(), // Skip unknown types
        }
    }

    #[allow(clippy::cast_sign_loss)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.index).max(0) as usize;
        (0, Some(remaining))
    }
}
