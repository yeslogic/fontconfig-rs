//!

use std::marker::PhantomData;
use std::ptr::{self, NonNull};

use fontconfig_sys as sys;
use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

use crate::FcTrue;

/// Wrapper around `FcCharSet`.
pub struct CharSet<'a> {
    pub(crate) fcset: NonNull<sys::FcCharSet>,
    pub(crate) _marker: PhantomData<&'a sys::FcCharSet>,
}

impl<'a> CharSet<'a> {
    /// Count entries in a charset
    pub fn len(&self) -> usize {
        let size = unsafe { ffi_dispatch!(LIB, FcCharSetCount, self.as_ptr()) };
        size as usize
    }

    /// Check if charset has entries
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if a character is in the `CharSet`.
    pub fn has_char(&self, c: char) -> bool {
        let res = unsafe { ffi_dispatch!(LIB, FcCharSetHasChar, self.as_ptr(), c as u32) };
        res == FcTrue
    }

    /// Check if self is a subset of other `CharSet`.
    pub fn is_subset(&self, other: &Self) -> bool {
        let res = unsafe { ffi_dispatch!(LIB, FcCharSetIsSubset, self.as_ptr(), other.as_ptr()) };
        res == FcTrue
    }

    /// Merge self with other `CharSet`.
    pub fn merge(&mut self, other: &Self) {
        let res = unsafe {
            ffi_dispatch!(
                LIB,
                FcCharSetMerge,
                self.as_mut_ptr(),
                other.as_ptr(),
                ptr::null_mut()
            )
        };
        assert_eq!(res, FcTrue);
    }

    /// Intersect self with other `CharSet`.
    pub fn intersect(&self, other: &Self) -> Self {
        let fcset =
            unsafe { ffi_dispatch!(LIB, FcCharSetIntersect, self.as_ptr(), other.as_ptr()) };
        Self {
            fcset: NonNull::new(fcset).expect("intersect failed"),
            _marker: PhantomData,
        }
    }

    /// Subtract other `CharSet` from self.
    pub fn subtract(&self, other: &Self) -> Self {
        let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetSubtract, self.as_ptr(), other.as_ptr()) };
        Self {
            fcset: NonNull::new(fcset).expect("subtract failed"),
            _marker: PhantomData,
        }
    }

    /// Union self with other `CharSet`.
    pub fn union(&self, other: &Self) -> Self {
        let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetUnion, self.as_ptr(), other.as_ptr()) };
        Self {
            fcset: NonNull::new(fcset).expect("union failed"),
            _marker: PhantomData,
        }
    }

    fn as_ptr(&self) -> *const sys::FcCharSet {
        self.fcset.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut sys::FcCharSet {
        self.fcset.as_ptr()
    }
}

impl CharSet<'static> {
    /// Create a new, empty `CharSet`.
    pub fn new() -> CharSet<'static> {
        let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetCreate,) };
        CharSet {
            fcset: NonNull::new(fcset).unwrap(),
            _marker: PhantomData,
        }
    }
    /// Add a character to the `CharSet`.
    pub fn add_char(&mut self, c: char) {
        let res = unsafe { ffi_dispatch!(LIB, FcCharSetAddChar, self.as_mut_ptr(), c as u32) };
        assert_eq!(res, FcTrue);
    }

    /// Delete a character from the `CharSet
    pub fn del_char(&mut self, c: char) {
        let res = unsafe { ffi_dispatch!(LIB, FcCharSetDelChar, self.as_mut_ptr(), c as u32) };
        assert_eq!(res, FcTrue);
    }
}

impl<'a> PartialEq for CharSet<'a> {
    fn eq(&self, other: &Self) -> bool {
        let res = unsafe {
            ffi_dispatch!(
                LIB,
                FcCharSetEqual,
                self.fcset.as_ptr(),
                other.fcset.as_ptr()
            )
        };
        res == FcTrue
    }
}

impl<'a> Clone for CharSet<'a> {
    fn clone(&self) -> CharSet<'a> {
        let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetCopy, self.fcset.as_ptr()) };
        CharSet {
            fcset: NonNull::new(fcset).expect("Can't clone CharSet"),
            _marker: PhantomData,
        }
    }
}

impl Drop for CharSet<'_> {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcCharSetDestroy, self.as_mut_ptr()) };
    }
}

impl Default for CharSet<'static> {
    fn default() -> Self {
        Self::new()
    }
}
