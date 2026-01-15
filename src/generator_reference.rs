//! `GeneratorReference` type for generated media content.

use crate::{ffi, macros, traits, TimeRange};
use std::ffi::CString;

/// Common generator kinds.
pub mod kinds {
    /// Solid color generator.
    pub const SOLID_COLOR: &str = "SolidColor";
    /// Color bars generator (SMPTE).
    pub const SMPTE_BARS: &str = "SMPTEBars";
    /// Black video generator.
    pub const BLACK: &str = "Black";
}

/// A reference to generated media content.
///
/// `GeneratorReference` is used for procedurally generated content like
/// color bars, solid colors, black video, or other synthetic media.
///
/// # Example
///
/// ```no_run
/// use otio_rs::{GeneratorReference, generator_reference::kinds, TimeRange, RationalTime};
///
/// let mut gen = GeneratorReference::new("Color Bars", kinds::SMPTE_BARS);
/// gen.set_available_range(TimeRange::new(
///     RationalTime::new(0.0, 24.0),
///     RationalTime::new(240.0, 24.0),
/// )).unwrap();
/// ```
pub struct GeneratorReference {
    pub(crate) ptr: *mut ffi::OtioGeneratorRef,
}

impl GeneratorReference {
    /// Create a new generator reference.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for this reference
    /// * `generator_kind` - The type of generator (use constants from `kinds` module)
    #[must_use]
    pub fn new(name: &str, generator_kind: &str) -> Self {
        let c_name = CString::new(name).unwrap();
        let c_kind = CString::new(generator_kind).unwrap();
        let ptr =
            unsafe { ffi::otio_generator_ref_create(c_name.as_ptr(), c_kind.as_ptr()) };
        Self { ptr }
    }

    /// Create a black video generator reference.
    #[must_use]
    pub fn black(name: &str) -> Self {
        Self::new(name, kinds::BLACK)
    }

    /// Create a SMPTE color bars generator reference.
    #[must_use]
    pub fn smpte_bars(name: &str) -> Self {
        Self::new(name, kinds::SMPTE_BARS)
    }

    macros::impl_string_getter!(
        name,
        otio_generator_ref_get_name,
        "Get the name of this generator reference."
    );
    macros::impl_string_getter!(
        generator_kind,
        otio_generator_ref_get_generator_kind,
        "Get the generator kind."
    );
    macros::impl_string_setter!(
        set_generator_kind,
        otio_generator_ref_set_generator_kind,
        "Set the generator kind."
    );
    macros::impl_time_range_setter!(
        set_available_range,
        otio_generator_ref_set_available_range,
        "Set the available range of this generator."
    );

    /// Get the available range of this generator.
    #[must_use]
    #[allow(clippy::float_cmp)] // Sentinel value comparison is intentional
    pub fn available_range(&self) -> Option<TimeRange> {
        let ffi_range = unsafe { ffi::otio_generator_ref_get_available_range(self.ptr) };
        // Check if this is a zero range (meaning no range set)
        if ffi_range.duration.value == 0.0 && ffi_range.duration.rate == 1.0 {
            return None;
        }
        Some(TimeRange::new(
            crate::RationalTime::new(ffi_range.start_time.value, ffi_range.start_time.rate),
            crate::RationalTime::new(ffi_range.duration.value, ffi_range.duration.rate),
        ))
    }
}

traits::impl_has_metadata!(
    GeneratorReference,
    otio_generator_ref_set_metadata_string,
    otio_generator_ref_get_metadata_string
);

impl Drop for GeneratorReference {
    fn drop(&mut self) {
        unsafe { ffi::otio_generator_ref_free(self.ptr) }
    }
}

// Safety: GeneratorReference is safe to send between threads
unsafe impl Send for GeneratorReference {}
