//!

use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};

use fontconfig_sys as sys;
use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

use crate::FcTrue;

/// Wrapper around `FcCharSet`.
pub struct OwnedCharSet {
    pub(crate) fcset: NonNull<sys::FcCharSet>,
}

/// Wrapper around `FcCharSet`.
#[repr(transparent)]
pub struct CharSet {
    pub(crate) fcset: sys::FcCharSet,
}

impl CharSet {
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

    /// Intersect self with other `CharSet`.
    pub fn intersect(&self, other: &Self) -> OwnedCharSet {
        let fcset =
            unsafe { ffi_dispatch!(LIB, FcCharSetIntersect, self.as_ptr(), other.as_ptr()) };
        OwnedCharSet {
            fcset: NonNull::new(fcset).expect("intersect failed"),
        }
    }

    /// Subtract other `CharSet` from self.
    pub fn subtract(&self, other: &Self) -> OwnedCharSet {
        let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetSubtract, self.as_ptr(), other.as_ptr()) };
        OwnedCharSet {
            fcset: NonNull::new(fcset).expect("subtract failed"),
        }
    }

    /// Union self with other `CharSet`.
    pub fn union(&self, other: &Self) -> OwnedCharSet {
        let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetUnion, self.as_ptr(), other.as_ptr()) };
        OwnedCharSet {
            fcset: NonNull::new(fcset).expect("union failed"),
        }
    }

    fn as_ptr(&self) -> *const sys::FcCharSet {
        &self.fcset
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut sys::FcCharSet {
        &mut self.fcset
    }
}

impl OwnedCharSet {
    /// Create a new, empty `CharSet`.
    pub fn new() -> Self {
        let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetCreate,) };
        OwnedCharSet {
            fcset: NonNull::new(fcset).unwrap(),
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
}

impl PartialEq for CharSet {
    fn eq(&self, other: &Self) -> bool {
        let res = unsafe { ffi_dispatch!(LIB, FcCharSetEqual, self.as_ptr(), other.as_ptr()) };
        res == FcTrue
    }
}

// NOTE: This just add reference, it is not safe.
// impl<'a> Clone for CharSet<'a> {
//     fn clone(&self) -> CharSet<'a> {
//         let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetCopy, self.fcset.as_ptr()) };
//         CharSet {
//             fcset: NonNull::new(fcset).expect("Can't clone CharSet"),
//             _marker: PhantomData,
//         }
//     }
// }

impl Drop for OwnedCharSet {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcCharSetDestroy, self.as_mut_ptr()) };
    }
}

impl Default for OwnedCharSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for OwnedCharSet {
    type Target = CharSet;
    fn deref(&self) -> &CharSet {
        unsafe { &*(self.fcset.as_ptr() as *const CharSet) }
    }
}

impl DerefMut for OwnedCharSet {
    fn deref_mut(&mut self) -> &mut CharSet {
        unsafe { &mut *(self.fcset.as_ptr() as *mut CharSet) }
    }
}

impl AsRef<CharSet> for OwnedCharSet {
    fn as_ref(&self) -> &CharSet {
        self
    }
}

impl AsMut<CharSet> for OwnedCharSet {
    fn as_mut(&mut self) -> &mut CharSet {
        self
    }
}
