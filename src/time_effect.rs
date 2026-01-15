//! Time effect types for speed changes and freeze frames.

use crate::{ffi, macros, traits};
use std::ffi::CString;

/// A linear time warp effect that changes playback speed.
///
/// The `time_scalar` determines the speed:
/// - 1.0 = normal speed
/// - 2.0 = double speed (2x fast forward)
/// - 0.5 = half speed (slow motion)
/// - -1.0 = reverse at normal speed
/// - 0.0 = freeze frame
///
/// # Example
///
/// ```no_run
/// use otio_rs::LinearTimeWarp;
///
/// // Create a 2x speed effect
/// let mut effect = LinearTimeWarp::new("Fast Forward", 2.0);
///
/// // Create a slow motion effect
/// let slow_mo = LinearTimeWarp::slow_motion("Slow Mo", 0.5);
///
/// // Create a reverse effect
/// let reverse = LinearTimeWarp::reverse("Reverse");
/// ```
pub struct LinearTimeWarp {
    pub(crate) ptr: *mut ffi::OtioLinearTimeWarp,
}

impl LinearTimeWarp {
    /// Create a new linear time warp effect.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for this effect
    /// * `time_scalar` - Speed multiplier (1.0 = normal, 2.0 = 2x speed, etc.)
    #[must_use]
    pub fn new(name: &str, time_scalar: f64) -> Self {
        let c_name = CString::new(name).unwrap();
        let ptr = unsafe { ffi::otio_linear_time_warp_create(c_name.as_ptr(), time_scalar) };
        Self { ptr }
    }

    /// Create a slow motion effect.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for this effect
    /// * `speed` - Speed multiplier (0.5 = half speed, 0.25 = quarter speed, etc.)
    #[must_use]
    pub fn slow_motion(name: &str, speed: f64) -> Self {
        Self::new(name, speed)
    }

    /// Create a reverse playback effect at normal speed.
    #[must_use]
    pub fn reverse(name: &str) -> Self {
        Self::new(name, -1.0)
    }

    /// Create a fast forward effect.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for this effect
    /// * `multiplier` - Speed multiplier (2.0 = 2x speed, 4.0 = 4x speed, etc.)
    #[must_use]
    pub fn fast_forward(name: &str, multiplier: f64) -> Self {
        Self::new(name, multiplier)
    }

    macros::impl_string_getter!(
        name,
        otio_linear_time_warp_get_name,
        "Get the name of this effect."
    );
    macros::impl_double_getter!(
        time_scalar,
        otio_linear_time_warp_get_time_scalar,
        "Get the time scalar (speed multiplier)."
    );
    macros::impl_double_setter!(
        set_time_scalar,
        otio_linear_time_warp_set_time_scalar,
        "Set the time scalar (speed multiplier)."
    );
}

traits::impl_has_metadata!(
    LinearTimeWarp,
    otio_linear_time_warp_set_metadata_string,
    otio_linear_time_warp_get_metadata_string
);

impl Drop for LinearTimeWarp {
    fn drop(&mut self) {
        unsafe { ffi::otio_linear_time_warp_free(self.ptr) }
    }
}

// Safety: LinearTimeWarp is safe to send between threads
unsafe impl Send for LinearTimeWarp {}

/// A freeze frame effect that holds a single frame.
///
/// This is a specialized time effect where `time_scalar = 0`, meaning
/// time does not advance and a single frame is displayed for the
/// duration of the clip.
///
/// # Example
///
/// ```no_run
/// use otio_rs::FreezeFrame;
///
/// let freeze = FreezeFrame::new("Hold Frame");
/// ```
pub struct FreezeFrame {
    pub(crate) ptr: *mut ffi::OtioFreezeFrame,
}

impl FreezeFrame {
    /// Create a new freeze frame effect.
    #[must_use]
    pub fn new(name: &str) -> Self {
        let c_name = CString::new(name).unwrap();
        let ptr = unsafe { ffi::otio_freeze_frame_create(c_name.as_ptr()) };
        Self { ptr }
    }

    macros::impl_string_getter!(
        name,
        otio_freeze_frame_get_name,
        "Get the name of this effect."
    );
}

traits::impl_has_metadata!(
    FreezeFrame,
    otio_freeze_frame_set_metadata_string,
    otio_freeze_frame_get_metadata_string
);

impl Drop for FreezeFrame {
    fn drop(&mut self) {
        unsafe { ffi::otio_freeze_frame_free(self.ptr) }
    }
}

// Safety: FreezeFrame is safe to send between threads
unsafe impl Send for FreezeFrame {}
