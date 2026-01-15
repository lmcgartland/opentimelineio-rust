//! Transition type for transitions between clips.

use crate::{ffi, macros, traits, RationalTime};
use std::ffi::CString;

/// Predefined transition types matching OTIO's `Transition::Type` constants.
pub mod types {
    /// Standard SMPTE dissolve transition.
    pub const SMPTE_DISSOLVE: &str = "SMPTE_Dissolve";
    /// Custom transition type.
    pub const CUSTOM: &str = "Custom_Transition";
}

/// A transition between two clips in a track.
///
/// Transitions define how one clip blends into the next. The `in_offset`
/// specifies how much of the outgoing clip overlaps, and `out_offset`
/// specifies how much of the incoming clip overlaps.
///
/// # Example
///
/// ```no_run
/// use otio_rs::{Transition, RationalTime, transition::types};
///
/// // Create a 12-frame dissolve at 24fps
/// let in_offset = RationalTime::new(12.0, 24.0);
/// let out_offset = RationalTime::new(12.0, 24.0);
/// let transition = Transition::new("Dissolve", types::SMPTE_DISSOLVE, in_offset, out_offset);
/// ```
pub struct Transition {
    pub(crate) ptr: *mut ffi::OtioTransition,
}

impl Transition {
    /// Create a new transition.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for this transition
    /// * `transition_type` - Type of transition (use constants from `types` module)
    /// * `in_offset` - Duration of overlap into the outgoing clip
    /// * `out_offset` - Duration of overlap into the incoming clip
    #[must_use]
    pub fn new(
        name: &str,
        transition_type: &str,
        in_offset: RationalTime,
        out_offset: RationalTime,
    ) -> Self {
        let c_name = CString::new(name).unwrap();
        let c_type = CString::new(transition_type).unwrap();
        let ptr = unsafe {
            ffi::otio_transition_create(
                c_name.as_ptr(),
                c_type.as_ptr(),
                in_offset.into(),
                out_offset.into(),
            )
        };
        Self { ptr }
    }

    /// Create a standard SMPTE dissolve transition.
    #[must_use]
    pub fn dissolve(name: &str, in_offset: RationalTime, out_offset: RationalTime) -> Self {
        Self::new(name, types::SMPTE_DISSOLVE, in_offset, out_offset)
    }

    macros::impl_string_getter!(
        name,
        otio_transition_get_name,
        "Get the name of this transition."
    );
    macros::impl_string_getter!(
        transition_type,
        otio_transition_get_transition_type,
        "Get the transition type."
    );
    macros::impl_string_setter!(
        set_transition_type,
        otio_transition_set_transition_type,
        "Set the transition type."
    );
    macros::impl_rational_time_getter!(
        in_offset,
        otio_transition_get_in_offset,
        "Get the in offset (overlap into outgoing clip)."
    );
    macros::impl_rational_time_setter!(
        set_in_offset,
        otio_transition_set_in_offset,
        "Set the in offset."
    );
    macros::impl_rational_time_getter!(
        out_offset,
        otio_transition_get_out_offset,
        "Get the out offset (overlap into incoming clip)."
    );
    macros::impl_rational_time_setter!(
        set_out_offset,
        otio_transition_set_out_offset,
        "Set the out offset."
    );
    macros::impl_rational_time_getter!(
        duration,
        otio_transition_get_duration,
        "Get the total duration of the transition."
    );
}

traits::impl_has_metadata!(
    Transition,
    otio_transition_set_metadata_string,
    otio_transition_get_metadata_string
);

impl Drop for Transition {
    fn drop(&mut self) {
        unsafe { ffi::otio_transition_free(self.ptr) }
    }
}

// Safety: Transition is safe to send between threads
unsafe impl Send for Transition {}
