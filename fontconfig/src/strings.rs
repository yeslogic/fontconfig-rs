//! To help applications deal with these UTF-8 strings in a locale-insensitive manner.

use std::ffi::CStr;
use std::fmt;
use std::ops::Deref;
use std::ptr::NonNull;

use fontconfig_sys as sys;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

use sys::ffi_dispatch;

/// C String
#[repr(transparent)]
pub struct FcStr(NonNull<sys::FcChar8>);

/// Represented with the FcChar8 type with ownership.
impl FcStr {
    #[doc(hidden)]
    pub unsafe fn from_ptr(ptr: *mut sys::FcChar8) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| FcStr(ptr))
    }
}

impl Drop for FcStr {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcStrFree, self.0.as_ptr()) }
    }
}

impl Deref for FcStr {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.0.as_ptr() as *const i8) }
    }
}

impl AsRef<CStr> for FcStr {
    fn as_ref(&self) -> &CStr {
        self.deref()
    }
}

impl fmt::Debug for FcStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}
