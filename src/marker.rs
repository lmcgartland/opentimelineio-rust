//! Marker type for annotating timeline positions.

use crate::{ffi, macros, traits, TimeRange};
use std::ffi::CString;

/// Predefined marker colors matching OTIO's `Marker::Color` constants.
pub mod colors {
    pub const PINK: &str = "PINK";
    pub const RED: &str = "RED";
    pub const ORANGE: &str = "ORANGE";
    pub const YELLOW: &str = "YELLOW";
    pub const GREEN: &str = "GREEN";
    pub const CYAN: &str = "CYAN";
    pub const BLUE: &str = "BLUE";
    pub const PURPLE: &str = "PURPLE";
    pub const MAGENTA: &str = "MAGENTA";
    pub const BLACK: &str = "BLACK";
    pub const WHITE: &str = "WHITE";
}

/// A marker annotation on a timeline.
///
/// Markers are used to annotate specific points or ranges in a timeline
/// with colors, comments, and metadata.
///
/// # Example
///
/// ```no_run
/// use otio_rs::{Marker, RationalTime, TimeRange, marker::colors};
///
/// let range = TimeRange::new(
///     RationalTime::new(100.0, 24.0),
///     RationalTime::new(24.0, 24.0),
/// );
/// let mut marker = Marker::new("Important", range, colors::RED);
/// marker.set_comment("Review this section");
/// ```
pub struct Marker {
    pub(crate) ptr: *mut ffi::OtioMarker,
}

impl Marker {
    /// Create a new marker with the given name, range, and color.
    ///
    /// Use constants from the `colors` module for standard colors.
    #[must_use]
    pub fn new(name: &str, marked_range: TimeRange, color: &str) -> Self {
        let c_name = CString::new(name).unwrap();
        let c_color = CString::new(color).unwrap();
        let ptr = unsafe {
            ffi::otio_marker_create(c_name.as_ptr(), marked_range.into(), c_color.as_ptr())
        };
        Self { ptr }
    }

    /// Create a new marker with the default green color.
    #[must_use]
    pub fn with_default_color(name: &str, marked_range: TimeRange) -> Self {
        Self::new(name, marked_range, colors::GREEN)
    }

    macros::impl_string_getter!(name, otio_marker_get_name, "Get the name of this marker.");
    macros::impl_string_getter!(color, otio_marker_get_color, "Get the color of this marker.");
    macros::impl_string_setter!(set_color, otio_marker_set_color, "Set the color of this marker.");
    macros::impl_time_range_getter!(
        marked_range,
        otio_marker_get_marked_range,
        "Get the marked range."
    );
    macros::impl_time_range_setter!(
        set_marked_range,
        otio_marker_set_marked_range,
        "Set the marked range."
    );
    macros::impl_string_getter!(comment, otio_marker_get_comment, "Get the comment.");
    macros::impl_string_setter!(set_comment, otio_marker_set_comment, "Set the comment.");
}

traits::impl_has_metadata!(
    Marker,
    otio_marker_set_metadata_string,
    otio_marker_get_metadata_string
);

impl Drop for Marker {
    fn drop(&mut self) {
        unsafe { ffi::otio_marker_free(self.ptr) }
    }
}

// Safety: Marker is safe to send between threads
unsafe impl Send for Marker {}
