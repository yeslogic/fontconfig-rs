//!
use fontconfig_sys as sys;

use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

/// FcBlanks
#[doc(alias = "FcBlanks")]
pub struct Blanks(*mut sys::FcBlanks);

impl Blanks {
    /// Create an FcBlanks
    pub fn new() -> Blanks {
        let ptr = unsafe { ffi_dispatch!(LIB, FcBlanksCreate,) };
        Blanks(ptr)
    }

    #[allow(dead_code)]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut sys::FcBlanks {
        self.0
    }
}

impl Default for Blanks {
    fn default() -> Self {
        Self::new()
    }
}
