//! `MissingReference` type for representing missing media.

use crate::{ffi, traits};

/// A reference to missing media.
///
/// `MissingReference` is used when the actual media file is not available
/// or cannot be found. This is useful for representing offline clips or
/// placeholders in a timeline.
///
/// # Example
///
/// ```no_run
/// use otio_rs::{MissingReference, Clip, TimeRange, RationalTime};
///
/// let missing = MissingReference::new();
/// let mut clip = Clip::new("Offline Clip", TimeRange::new(
///     RationalTime::new(0.0, 24.0),
///     RationalTime::new(48.0, 24.0),
/// ));
/// clip.set_missing_reference(missing).unwrap();
/// ```
pub struct MissingReference {
    pub(crate) ptr: *mut ffi::OtioMissingRef,
}

impl MissingReference {
    /// Create a new missing reference.
    #[must_use]
    pub fn new() -> Self {
        let ptr = unsafe { ffi::otio_missing_ref_create() };
        Self { ptr }
    }
}

impl Default for MissingReference {
    fn default() -> Self {
        Self::new()
    }
}

traits::impl_has_metadata!(
    MissingReference,
    otio_missing_ref_set_metadata_string,
    otio_missing_ref_get_metadata_string
);

impl Drop for MissingReference {
    fn drop(&mut self) {
        unsafe { ffi::otio_missing_ref_free(self.ptr) }
    }
}

// Safety: MissingReference is safe to send between threads
unsafe impl Send for MissingReference {}
