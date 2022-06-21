//!

use std::fmt;
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

    ///
    pub fn iter(&self) -> Iter {
        let mut map = [0; sys::constants::FC_CHARSET_MAP_SIZE as usize];
        let mut next = 0;
        let codepoint = unsafe {
            ffi_dispatch!(
                LIB,
                FcCharSetFirstPage,
                self.as_ptr(),
                map.as_mut_ptr(),
                &mut next
            )
        };
        Iter {
            cs: self,
            codepoint,
            next,
            map,
            i: 0,
            bit: 0,
        }
    }

    pub(crate) fn as_ptr(&self) -> *const sys::FcCharSet {
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
    pub fn merge(&mut self, other: &CharSet) {
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

impl fmt::Debug for CharSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl fmt::Debug for OwnedCharSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

///
pub struct Iter<'a> {
    cs: &'a CharSet,
    codepoint: sys::FcChar32,
    next: sys::FcChar32,
    map: [sys::FcChar32; sys::constants::FC_CHARSET_MAP_SIZE as usize],
    i: usize,
    bit: usize,
}

impl Iterator for Iter<'_> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        const MAP_SIZE: usize = sys::constants::FC_CHARSET_MAP_SIZE as usize;
        loop {
            if self.codepoint == sys::constants::FC_CHARSET_DONE {
                return None;
            }
            // end of page.
            if self.i >= MAP_SIZE {
                if self.next == sys::constants::FC_CHARSET_DONE {
                    // end of last page.
                    self.codepoint = sys::constants::FC_CHARSET_DONE;
                } else {
                    // next page
                    self.codepoint = unsafe {
                        ffi_dispatch!(
                            LIB,
                            FcCharSetNextPage,
                            self.cs.as_ptr(),
                            self.map.as_mut_ptr(),
                            &mut self.next
                        )
                    };
                    self.i = 0;
                }
                continue;
            }
            let mut bits = self.map[self.i];
            if bits == 0 {
                self.i += 1;
                self.bit = 0;
                continue;
            }

            let mut ok = true;
            // println!(" -> bits {:#b} {}", bits, self.bit);
            let mut n = bits >> self.bit;
            while n & 1 == 0 {
                if n == 0 {
                    self.i += 1;
                    self.bit = 0;
                    ok = false;
                    break;
                }
                self.bit += 1;
                if self.bit > 31 {
                    self.i += 1;
                    self.bit = 0;
                    ok = false;
                    break;
                }
                if self.i >= MAP_SIZE {
                    ok = false;
                    break;
                }
                bits = self.map[self.i];
                // println!("bits {:#b} x {}", bits, self.bit);
                n = bits >> self.bit;
            }
            // 如果上面的while循环正常执行完，没有break，才可以执行这里.
            // if the while loop is not broken, then we can execute this.
            if ok {
                let codepoint =
                    self.codepoint + (32u32 * self.i as u32) + u32::try_from(self.bit).ok()?;
                // println!("{}({})", codepoint, char::from_u32(codepoint).unwrap());
                self.bit += 1;
                if self.bit > 31 {
                    self.i += 1;
                    self.bit = 0;
                }
                return char::try_from(codepoint).ok();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charset_modify() {
        let mut cs = OwnedCharSet::new();
        assert!(!cs.has_char('a'));
        cs.add_char('a');
        assert!(cs.has_char('a'));
        cs.del_char('a');
        assert!(!cs.has_char('a'));
    }

    #[test]
    fn charset_merge() {
        let mut cs1 = OwnedCharSet::new();
        let mut cs2 = OwnedCharSet::new();
        cs1.add_char('a');
        cs2.add_char('b');
        cs1.merge(&cs2);
        assert!(cs1.has_char('a'));
        assert!(cs1.has_char('b'));
    }

    #[test]
    fn charset_union() {
        let mut cs1 = OwnedCharSet::new();
        let mut cs2 = OwnedCharSet::new();
        cs1.add_char('a');
        cs2.add_char('b');
        let cs3 = cs1.union(&cs2);
        assert!(cs3.has_char('a'));
        assert!(cs3.has_char('b'));
        assert!(!cs1.has_char('b'));
        assert!(!cs2.has_char('a'));
    }

    #[test]
    fn charset_intersect() {
        let mut cs1 = OwnedCharSet::new();
        let mut cs2 = OwnedCharSet::new();
        cs1.add_char('a');
        cs1.add_char('c');
        cs2.add_char('b');
        cs2.add_char('c');
        let cs3 = cs1.intersect(&cs2);
        assert!(!cs3.has_char('a'));
        assert!(!cs3.has_char('b'));
        assert!(cs3.has_char('c'));
        assert!(!cs1.has_char('b'));
        assert!(!cs2.has_char('a'));
    }

    #[test]
    fn charset_subtract() {
        let mut cs1 = OwnedCharSet::new();
        let mut cs2 = OwnedCharSet::new();
        cs1.add_char('a');
        cs1.add_char('c');
        cs2.add_char('b');
        cs2.add_char('c');
        let cs3 = cs1.subtract(&cs2);
        assert!(cs3.has_char('a'));
        assert!(!cs3.has_char('b'));
        assert!(!cs3.has_char('c'));
        assert!(!cs1.has_char('b'));
        assert!(!cs2.has_char('a'));
    }

    #[test]
    fn charset_equal() {
        let mut cs1 = OwnedCharSet::new();
        let mut cs2 = OwnedCharSet::new();
        cs1.add_char('a');
        cs1.add_char('c');
        cs2.add_char('b');
        cs2.add_char('c');
        assert_ne!(cs1.as_ref(), cs2.as_ref());
        cs2.add_char('a');
        cs2.del_char('b');
        assert_eq!(cs1.as_ref(), cs2.as_ref());
    }

    #[test]
    fn charset_iter() {
        let mut cs = OwnedCharSet::new();
        cs.add_char('a');
        cs.add_char('b');
        cs.add_char('c');
        cs.add_char('汉');
        cs.add_char('字');
        cs.add_char('😁');
        let mut iter = cs.iter();
        assert_eq!(iter.next(), Some('a'));
        assert_eq!(iter.next(), Some('b'));
        assert_eq!(iter.next(), Some('c'));
        assert_eq!(iter.next(), Some('字'));
        assert_eq!(iter.next(), Some('汉'));
        assert_eq!(iter.next(), Some('😁'));
        assert_eq!(iter.next(), None);
    }
}
