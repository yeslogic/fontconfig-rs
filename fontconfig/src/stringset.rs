use std::ffi::CStr;
use std::ptr::NonNull;

use fontconfig_sys as sys;

use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

/// FcStrSet holds a list of strings that can be appended to and enumerated.
/// Its unique characteristic is that the enumeration works even while strings are appended during enumeration.
#[doc(alias = "FcStrSet")]
pub struct StringSet {
    pub(crate) set: NonNull<sys::FcStrSet>,
}

impl StringSet {
    /// Create a new, empty `StringSet`.
    pub fn new() -> StringSet {
        let set = unsafe { ffi_dispatch!(LIB, FcStrSetCreate,) };
        StringSet {
            set: NonNull::new(set).unwrap(),
        }
    }

    /// Creates an iterator to list the strings in set.
    pub fn iter<'a>(&'a self) -> StringSetIter<'a> {
        StringSetIter::new(self)
    }

    fn as_mut_ptr(&mut self) -> *mut sys::FcStrSet {
        self.set.as_ptr()
    }

    fn as_ptr(&self) -> *const sys::FcStrSet {
        self.set.as_ptr()
    }
}

impl Drop for StringSet {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcStrSetDestroy, self.as_mut_ptr()) };
    }
}

///
pub struct StringSetIter<'a> {
    _set: &'a StringSet,
    iter: NonNull<sys::FcStrList>,
}

impl<'a> StringSetIter<'a> {
    fn new(set: &'a StringSet) -> StringSetIter<'a> {
        let iter = unsafe {
            let iter = ffi_dispatch!(LIB, FcStrListCreate, set.as_ptr() as *mut _);
            // ffi_dispatch!(LIB, FcStrListFirst, iter);
            iter
        };
        StringSetIter {
            _set: set,
            iter: NonNull::new(iter).unwrap(),
        }
    }
    fn as_mut_ptr(&mut self) -> *mut sys::FcStrList {
        self.iter.as_ptr()
    }
}

impl Drop for StringSetIter<'_> {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcStrListDone, self.as_mut_ptr()) };
    }
}

impl<'a> Iterator for StringSetIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let s = ffi_dispatch!(LIB, FcStrListNext, self.as_mut_ptr());
            if s.is_null() {
                None
            } else {
                CStr::from_ptr(s.cast()).to_str().ok()
            }
        }
    }
}
