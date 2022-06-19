//!

use std::ffi::CStr;
use std::iter::FromIterator;
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

    /// Add a string to the `ObjectSet`.
    pub fn add(&mut self, name: &CStr) {
        let res = unsafe { ffi_dispatch!(LIB, FcObjectSetAdd, self.as_mut_ptr(), name.as_ptr()) };
        assert_eq!(res, FcTrue);
    }

    /// Build object set from args
    pub fn build(args: &[&'_ CStr]) -> ObjectSet {
        let mut set = ObjectSet::new();
        for arg in args {
            set.add(arg);
        }
        set
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

impl<'a> FromIterator<&'a CStr> for ObjectSet {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a CStr>,
    {
        let mut set = Self::new();
        for name in iter {
            set.add(name);
        }
        set
    }
}

impl Drop for ObjectSet {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcObjectSetDestroy, self.as_mut_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objectset_new() {
        let set = ObjectSet::new(); // there will panic if failed.
        assert!(!set.as_ptr().is_null());
    }

    #[test]
    fn objectset_add() {
        let mut set = ObjectSet::new();
        set.add(CStr::from_bytes_with_nul(b"foo\0").unwrap());
    }

    #[test]
    fn objectset_build() {
        let set = ObjectSet::build(&[CStr::from_bytes_with_nul(b"foo\0").unwrap()]);
        assert!(!set.as_ptr().is_null());
    }
}
