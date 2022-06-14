//!

use std::ffi::CStr;
use std::ptr::NonNull;

use fontconfig_sys as sys;
use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

use crate::FcTrue;

/// Wrapper around `FcObjectSet`.
pub struct ObjectSet {
    fcset: NonNull<sys::FcObjectSet>,
}

impl ObjectSet {
    /// Create a new, empty `ObjectSet`.
    pub fn new() -> ObjectSet {
        let fcset = unsafe { ffi_dispatch!(LIB, FcObjectSetCreate,) };

        ObjectSet {
            fcset: NonNull::new(fcset).unwrap(),
        }
    }

    /// Wrap an existing `FcObjectSet`.
    ///
    /// The `FcObjectSet` must not be null. This method assumes ownership of the `FcObjectSet`.
    ///
    /// **Safety:** The object set pointer must be valid/non-null.
    // pub unsafe fn from_raw(_: &Fontconfig, raw_set: *mut sys::FcObjectSet) -> ObjectSet {
    //     // ObjectSet { fcset: raw_set }
    // }

    /// Add a string to the `ObjectSet`.
    pub fn add(&mut self, name: &CStr) {
        let res = unsafe { ffi_dispatch!(LIB, FcObjectSetAdd, self.as_mut_ptr(), name.as_ptr()) };
        assert_eq!(res, FcTrue);
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut sys::FcObjectSet {
        self.fcset.as_ptr()
    }

    #[allow(dead_code)]
    fn as_ptr(&self) -> *const sys::FcObjectSet {
        self.fcset.as_ptr()
    }
}

impl Default for ObjectSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ObjectSet {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcObjectSetDestroy, self.as_mut_ptr()) }
    }
}
