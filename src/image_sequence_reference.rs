//! `ImageSequenceReference` type for VFX image sequence media.

use crate::{ffi, ffi_string_to_rust, is_unset_time_range, macros, traits, RationalTime, Result, TimeRange};
use std::ffi::CString;

/// Policy for handling missing frames in an image sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MissingFramePolicy {
    /// Return an error when a frame is missing.
    #[default]
    Error = 0,
    /// Hold the last valid frame.
    Hold = 1,
    /// Show black for missing frames.
    Black = 2,
}

impl From<i32> for MissingFramePolicy {
    fn from(value: i32) -> Self {
        match value {
            1 => MissingFramePolicy::Hold,
            2 => MissingFramePolicy::Black,
            _ => MissingFramePolicy::Error,
        }
    }
}

/// A reference to an image sequence on disk.
///
/// `ImageSequenceReference` is used for VFX workflows where media consists
/// of numbered image files (e.g., EXR, DPX, or TIFF sequences).
///
/// # Example
///
/// ```no_run
/// use otio_rs::{ImageSequenceReference, TimeRange, RationalTime};
/// use otio_rs::image_sequence_reference::MissingFramePolicy;
///
/// // Create a reference to an EXR sequence: shot_0001.exr, shot_0002.exr, ...
/// let mut seq = ImageSequenceReference::new(
///     "/path/to/render/",  // target_url_base
///     "shot_",             // name_prefix
///     ".exr",              // name_suffix
///     1,                   // start_frame
///     1,                   // frame_step
///     24.0,                // rate (fps)
///     4,                   // frame_zero_padding (e.g., 0001)
/// );
///
/// seq.set_available_range(TimeRange::new(
///     RationalTime::new(0.0, 24.0),
///     RationalTime::new(100.0, 24.0), // 100 frames
/// )).unwrap();
///
/// seq.set_missing_frame_policy(MissingFramePolicy::Hold);
/// ```
pub struct ImageSequenceReference {
    pub(crate) ptr: *mut ffi::OtioImageSeqRef,
}

impl ImageSequenceReference {
    /// Create a new image sequence reference.
    ///
    /// # Arguments
    ///
    /// * `target_url_base` - Base path/URL to the image sequence directory
    /// * `name_prefix` - Prefix before the frame number (e.g., "shot_")
    /// * `name_suffix` - Suffix after the frame number (e.g., ".exr")
    /// * `start_frame` - First frame number in the sequence
    /// * `frame_step` - Step between frame numbers (usually 1)
    /// * `rate` - Frame rate in fps
    /// * `frame_zero_padding` - Number of digits for frame number (e.g., 4 for 0001)
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        target_url_base: &str,
        name_prefix: &str,
        name_suffix: &str,
        start_frame: i32,
        frame_step: i32,
        rate: f64,
        frame_zero_padding: i32,
    ) -> Self {
        let c_url = CString::new(target_url_base).unwrap();
        let c_prefix = CString::new(name_prefix).unwrap();
        let c_suffix = CString::new(name_suffix).unwrap();
        let ptr = unsafe {
            ffi::otio_image_seq_ref_create(
                c_url.as_ptr(),
                c_prefix.as_ptr(),
                c_suffix.as_ptr(),
                start_frame,
                frame_step,
                rate,
                frame_zero_padding,
            )
        };
        Self { ptr }
    }

    macros::impl_string_getter!(
        target_url_base,
        otio_image_seq_ref_get_target_url_base,
        "Get the base URL/path for the image sequence."
    );
    macros::impl_string_getter!(
        name_prefix,
        otio_image_seq_ref_get_name_prefix,
        "Get the prefix before the frame number."
    );
    macros::impl_string_getter!(
        name_suffix,
        otio_image_seq_ref_get_name_suffix,
        "Get the suffix after the frame number (usually the file extension)."
    );

    macros::impl_string_setter!(
        set_target_url_base,
        otio_image_seq_ref_set_target_url_base,
        "Set the base URL/path for the image sequence."
    );
    macros::impl_string_setter!(
        set_name_prefix,
        otio_image_seq_ref_set_name_prefix,
        "Set the prefix before the frame number."
    );
    macros::impl_string_setter!(
        set_name_suffix,
        otio_image_seq_ref_set_name_suffix,
        "Set the suffix after the frame number."
    );

    /// Get the start frame number.
    #[must_use]
    pub fn start_frame(&self) -> i32 {
        unsafe { ffi::otio_image_seq_ref_get_start_frame(self.ptr) }
    }

    /// Get the end frame number.
    ///
    /// This is computed from the start frame, frame step, and available range.
    #[must_use]
    pub fn end_frame(&self) -> i32 {
        unsafe { ffi::otio_image_seq_ref_get_end_frame(self.ptr) }
    }

    /// Get the frame step.
    #[must_use]
    pub fn frame_step(&self) -> i32 {
        unsafe { ffi::otio_image_seq_ref_get_frame_step(self.ptr) }
    }

    /// Get the frame rate in fps.
    #[must_use]
    pub fn rate(&self) -> f64 {
        unsafe { ffi::otio_image_seq_ref_get_rate(self.ptr) }
    }

    /// Get the frame zero padding (number of digits).
    #[must_use]
    pub fn frame_zero_padding(&self) -> i32 {
        unsafe { ffi::otio_image_seq_ref_get_frame_zero_padding(self.ptr) }
    }

    /// Get the missing frame policy.
    #[must_use]
    pub fn missing_frame_policy(&self) -> MissingFramePolicy {
        let policy = unsafe { ffi::otio_image_seq_ref_get_missing_frame_policy(self.ptr) };
        MissingFramePolicy::from(policy)
    }

    /// Set the start frame number.
    pub fn set_start_frame(&mut self, frame: i32) {
        unsafe { ffi::otio_image_seq_ref_set_start_frame(self.ptr, frame) }
    }

    /// Set the frame step.
    pub fn set_frame_step(&mut self, step: i32) {
        unsafe { ffi::otio_image_seq_ref_set_frame_step(self.ptr, step) }
    }

    /// Set the frame rate in fps.
    pub fn set_rate(&mut self, rate: f64) {
        unsafe { ffi::otio_image_seq_ref_set_rate(self.ptr, rate) }
    }

    /// Set the frame zero padding (number of digits).
    pub fn set_frame_zero_padding(&mut self, padding: i32) {
        unsafe { ffi::otio_image_seq_ref_set_frame_zero_padding(self.ptr, padding) }
    }

    /// Set the missing frame policy.
    pub fn set_missing_frame_policy(&mut self, policy: MissingFramePolicy) {
        unsafe { ffi::otio_image_seq_ref_set_missing_frame_policy(self.ptr, policy as i32) }
    }

    /// Get the number of images in the sequence.
    ///
    /// This is computed from the available range and frame step.
    #[must_use]
    pub fn number_of_images(&self) -> i32 {
        unsafe { ffi::otio_image_seq_ref_number_of_images(self.ptr) }
    }

    /// Get the frame number for a given time.
    ///
    /// # Errors
    ///
    /// Returns an error if the time is outside the available range.
    pub fn frame_for_time(&self, time: RationalTime) -> Result<i32> {
        let mut err = macros::ffi_error!();
        let frame = unsafe { ffi::otio_image_seq_ref_frame_for_time(self.ptr, time.into(), &mut err) };
        if err.code != 0 {
            return Err(err.into());
        }
        Ok(frame)
    }

    /// Get the target URL for a specific image number.
    ///
    /// # Errors
    ///
    /// Returns an error if the image number is invalid.
    pub fn target_url_for_image_number(&self, image_number: i32) -> Result<String> {
        let mut err = macros::ffi_error!();
        let ptr = unsafe {
            ffi::otio_image_seq_ref_target_url_for_image_number(self.ptr, image_number, &mut err)
        };
        if ptr.is_null() {
            return Err(err.into());
        }
        Ok(ffi_string_to_rust(ptr))
    }

    /// Get the available range of this image sequence.
    #[must_use]
    pub fn available_range(&self) -> Option<TimeRange> {
        let ffi_range = unsafe { ffi::otio_image_seq_ref_get_available_range(self.ptr) };
        if is_unset_time_range(&ffi_range) {
            return None;
        }
        Some(TimeRange::new(
            RationalTime::new(ffi_range.start_time.value, ffi_range.start_time.rate),
            RationalTime::new(ffi_range.duration.value, ffi_range.duration.rate),
        ))
    }

    macros::impl_time_range_setter!(
        set_available_range,
        otio_image_seq_ref_set_available_range,
        "Set the available range of this image sequence."
    );
}

traits::impl_has_metadata!(
    ImageSequenceReference,
    otio_image_seq_ref_set_metadata_string,
    otio_image_seq_ref_get_metadata_string
);

impl Drop for ImageSequenceReference {
    fn drop(&mut self) {
        unsafe { ffi::otio_image_seq_ref_free(self.ptr) }
    }
}

// Safety: ImageSequenceReference is safe to send between threads
unsafe impl Send for ImageSequenceReference {}
