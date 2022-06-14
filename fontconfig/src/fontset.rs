//!
use std::mem;
use std::ptr::NonNull;

use fontconfig_sys as sys;

use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

use crate::{Blanks, FcTrue, Pattern};

/// Wrapper around `FcFontSet`.
#[repr(transparent)]
pub struct FontSet {
    pub(crate) fcset: NonNull<sys::FcFontSet>,
}

impl FontSet {
    /// Create a new, empty `FontSet`.
    pub fn new() -> FontSet {
        let fcset = unsafe { ffi_dispatch!(LIB, FcFontSetCreate,) };
        FontSet {
            fcset: NonNull::new(fcset).unwrap(),
        }
    }

    /// Add a `Pattern` to this `FontSet`.
    #[doc(alias = "FcFontSetAdd")]
    pub fn push(&mut self, mut pat: Pattern) {
        unsafe {
            assert_eq!(
                ffi_dispatch!(LIB, FcFontSetAdd, self.as_mut_ptr(), pat.as_mut_ptr()),
                FcTrue,
            );
        }
        mem::forget(pat);
    }

    /// Print this `FontSet` to stdout.
    pub fn print(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcFontSetPrint, self.as_mut_ptr()) };
    }

    /// Iterate the fonts (as `Patterns`) in this `FontSet`.
    pub fn iter(&self) -> Iter {
        Iter {
            fcset: self,
            index: 0,
        }
    }

    /// Iterate the fonts (as `Patterns`) in this `FontSet`.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            fcset: self,
            index: 0,
        }
    }

    /// Compute all patterns from font file (and index)
    ///
    /// Constructs patterns found in 'file'.
    /// If id is -1, then all patterns found in 'file' are added to 'set'.
    /// Otherwise, this function works exactly like FcFreeTypeQuery().
    /// The number of faces in 'file' is returned in 'count'.
    /// The number of patterns added to 'set' is returned.
    /// FcBlanks is deprecated, blanks is ignored and accepted only for compatibility with older code.
    pub fn freetype_query_all(
        &mut self,
        _file: &std::path::Path,
        _index: isize,
        _blanks: Option<&mut Blanks>,
        _count: Option<&mut usize>,
    ) -> usize {
        unimplemented!()
        // unsafe {
        //     let blanks = blanks.map(|s| s.as_mut_ptr()).unwrap_or(std::ptr::null());
        //     assert_eq!(
        //         ffi_dispatch!(
        //             LIB,
        //             FcFreeTypeQueryAll,
        //             file.as_ptr(),
        //             index,
        //             blanks,
        //             ptr::null_mut(),
        //             self.as_mut_ptr()
        //         ),
        //         FcTrue,
        //     );
        // }
    }

    fn as_mut_ptr(&mut self) -> *mut sys::FcFontSet {
        self.fcset.as_ptr()
    }

    fn as_ptr(&self) -> *const sys::FcFontSet {
        self.fcset.as_ptr()
    }
}

impl Default for FontSet {
    fn default() -> FontSet {
        FontSet::new()
    }
}

impl Drop for FontSet {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcFontSetDestroy, self.as_mut_ptr()) }
    }
}

#[doc(hidden)]
pub struct Iter<'a> {
    fcset: &'a FontSet,
    index: usize,
}

impl<'a> Iterator for Iter<'a> {
    // FIXME: return reference.
    type Item = Pattern;

    fn next(&mut self) -> Option<Self::Item> {
        let fcset = self.fcset.as_ptr();
        if self.index >= unsafe { (*fcset).nfont } as usize {
            return None;
        }
        let pat = unsafe {
            let font = (*fcset).fonts.add(self.index);
            if font.is_null() {
                return None;
            }
            *font
        };
        if pat.is_null() {
            return None;
        }
        let pat = unsafe {
            ffi_dispatch!(LIB, FcPatternReference, pat);
            NonNull::new_unchecked(pat)
        };
        self.index += 1;
        Some(Pattern { pat })
    }
}

#[doc(hidden)]
pub struct IterMut<'a> {
    fcset: &'a mut FontSet,
    index: usize,
}

impl<'a> Iterator for IterMut<'a> {
    // FIXME: return reference.
    type Item = Pattern;

    fn next(&mut self) -> Option<Self::Item> {
        let fcset = self.fcset.as_ptr();
        if self.index >= unsafe { (*fcset).nfont } as usize {
            return None;
        }
        let pat = unsafe {
            let font = (*fcset).fonts.add(self.index);
            if font.is_null() {
                return None;
            }
            *font
        };
        if pat.is_null() {
            return None;
        }
        let pat = unsafe {
            ffi_dispatch!(LIB, FcPatternReference, pat);
            NonNull::new_unchecked(pat)
        };
        self.index += 1;
        Some(Pattern { pat })
    }
}
