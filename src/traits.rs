//! Traits for OTIO types.

/// Trait for types that support string metadata.
///
/// All OTIO objects can store arbitrary string key-value metadata pairs.
/// This trait provides a unified interface for getting and setting metadata.
///
/// # Example
///
/// ```no_run
/// use otio_rs::{Clip, RationalTime, TimeRange, HasMetadata};
///
/// let range = TimeRange::new(
///     RationalTime::new(0.0, 24.0),
///     RationalTime::new(48.0, 24.0),
/// );
/// let mut clip = Clip::new("My Clip", range);
///
/// clip.set_metadata("external_id", "abc123");
/// assert_eq!(clip.get_metadata("external_id"), Some("abc123".to_string()));
/// ```
pub trait HasMetadata {
    /// Set a string metadata value.
    fn set_metadata(&mut self, key: &str, value: &str);

    /// Get a string metadata value.
    ///
    /// Returns `None` if the key doesn't exist.
    fn get_metadata(&self, key: &str) -> Option<String>;
}

/// Macro to implement `HasMetadata` for a type with a pointer field.
///
/// This macro generates the boilerplate code for FFI calls to get/set metadata.
/// The getter properly frees the C-allocated string after copying.
macro_rules! impl_has_metadata {
    ($type:ty, $set_fn:ident, $get_fn:ident) => {
        impl $crate::traits::HasMetadata for $type {
            fn set_metadata(&mut self, key: &str, value: &str) {
                let c_key = std::ffi::CString::new(key).unwrap();
                let c_value = std::ffi::CString::new(value).unwrap();
                unsafe {
                    $crate::ffi::$set_fn(self.ptr, c_key.as_ptr(), c_value.as_ptr());
                }
            }

            fn get_metadata(&self, key: &str) -> Option<String> {
                let c_key = std::ffi::CString::new(key).unwrap();
                let ptr = unsafe { $crate::ffi::$get_fn(self.ptr, c_key.as_ptr()) };
                if ptr.is_null() {
                    None
                } else {
                    // Copy the string before freeing the C allocation
                    let result = unsafe {
                        std::ffi::CStr::from_ptr(ptr)
                            .to_string_lossy()
                            .into_owned()
                    };
                    // Free the C-allocated string
                    unsafe { $crate::ffi::otio_free_string(ptr) };
                    Some(result)
                }
            }
        }
    };
}

pub(crate) use impl_has_metadata;
