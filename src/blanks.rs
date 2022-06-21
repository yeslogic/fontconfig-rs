//! Wrapper for deprecated [`sys::FcBlanks`]
use fontconfig_sys as sys;

use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

/// FcBlanks is deprecated and should not be used in newly written code.
#[doc(alias = "FcBlanks")]
#[deprecated(note = "This type is deprecated and should not be used in newly written code.")]
pub struct Blanks(*mut sys::FcBlanks);

#[allow(deprecated)]
impl Blanks {
    /// Create an FcBlanks
    #[deprecated(note = "This type is deprecated and should not be used in newly written code.")]
    pub fn new() -> Blanks {
        let ptr = unsafe { ffi_dispatch!(LIB, FcBlanksCreate,) };
        Blanks(ptr)
    }

    #[allow(dead_code)]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut sys::FcBlanks {
        self.0
    }
}

#[allow(deprecated)]
impl Default for Blanks {
    fn default() -> Self {
        Self::new()
    }
}
