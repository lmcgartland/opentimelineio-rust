//! Effect type for representing effects applied to clips.

use crate::{ffi, macros, traits};
use std::ffi::CString;

/// An effect that can be applied to clips or other items.
///
/// Effects represent operations like color correction, blur, or other
/// processing applied to media. The `effect_name` identifies the type
/// of effect (e.g., `ColorCorrection`, `Blur`).
///
/// # Example
///
/// ```no_run
/// use otio_rs::Effect;
///
/// let mut effect = Effect::new("My Effect", "ColorCorrection");
/// effect.set_effect_name("Blur");
/// ```
pub struct Effect {
    pub(crate) ptr: *mut ffi::OtioEffect,
}

impl Effect {
    /// Create a new effect with the given name and effect type.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for this effect instance
    /// * `effect_name` - Type/category of effect (e.g., `ColorCorrection`)
    #[must_use]
    pub fn new(name: &str, effect_name: &str) -> Self {
        let c_name = CString::new(name).unwrap();
        let c_effect_name = CString::new(effect_name).unwrap();
        let ptr = unsafe { ffi::otio_effect_create(c_name.as_ptr(), c_effect_name.as_ptr()) };
        Self { ptr }
    }

    macros::impl_string_getter!(name, otio_effect_get_name, "Get the name of this effect.");
    macros::impl_string_getter!(
        effect_name,
        otio_effect_get_effect_name,
        "Get the effect type/category name."
    );
    macros::impl_string_setter!(
        set_effect_name,
        otio_effect_set_effect_name,
        "Set the effect type/category name."
    );
}

traits::impl_has_metadata!(
    Effect,
    otio_effect_set_metadata_string,
    otio_effect_get_metadata_string
);

impl Drop for Effect {
    fn drop(&mut self) {
        unsafe { ffi::otio_effect_free(self.ptr) }
    }
}

// Safety: Effect is safe to send between threads
unsafe impl Send for Effect {}
