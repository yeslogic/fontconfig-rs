//!
use std::ptr::NonNull;

use fontconfig_sys as sys;

use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

#[allow(deprecated)]
use crate::Blanks;

use crate::{FcTrue, OwnedPattern, Pattern};

/// Wrapper around `FcFontSet`.
#[doc(alias = "FcFontSet")]
#[repr(transparent)]
pub struct FontSet<'pat> {
    pub(crate) fcset: NonNull<sys::FcFontSet>,
    pub(crate) _marker: ::std::marker::PhantomData<&'pat mut Pattern>,
}

impl<'pat> FontSet<'pat> {
    /// Create an empty [`FontSet`].
    #[doc(alias = "FcFontSetCreate")]
    pub fn new() -> FontSet<'pat> {
        let fcset = unsafe { ffi_dispatch!(LIB, FcFontSetCreate,) };
        FontSet {
            fcset: NonNull::new(fcset).unwrap(),
            _marker: ::std::marker::PhantomData,
        }
    }

    /// Add a [`Pattern`] to this [`FontSet`].
    #[doc(alias = "FcFontSetAdd")]
    pub fn push(&mut self, pat: OwnedPattern) {
        unsafe {
            assert_eq!(
                ffi_dispatch!(LIB, FcFontSetAdd, self.as_mut_ptr(), pat.into_inner()),
                FcTrue,
            );
        }
    }

    /// How many fonts are in this [`FontSet`]
    #[doc(alias = "FcFontSet->nfont")]
    pub fn len(&self) -> usize {
        unsafe { (*self.as_ptr()).nfont as _ }
    }

    /// If there are no fonts in this [`FontSet`], return `true`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Print this [`FontSet`] to stdout.
    #[doc(alias = "FcFontSetPrint")]
    pub fn print(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcFontSetPrint, self.as_mut_ptr()) };
    }

    /// Iterate the fonts (as `Patterns`) in this [`FontSet`].
    pub fn iter<'fs>(&'fs self) -> Iter<'fs, 'pat> {
        Iter {
            fcset: self,
            index: 0,
        }
    }

    /// Iterate the fonts (as `Patterns`) in this [`FontSet`].
    pub fn iter_mut<'fs>(&'fs mut self) -> IterMut<'fs, 'pat> {
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
    /// [`Blanks`] is deprecated, blanks is ignored and accepted only for compatibility with older code.
    #[doc(alias = "FcFreeTypeQuery")]
    #[allow(deprecated)]
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

impl<'a> Default for FontSet<'a> {
    fn default() -> FontSet<'a> {
        FontSet::new()
    }
}

impl Drop for FontSet<'_> {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcFontSetDestroy, self.as_mut_ptr()) }
    }
}

/// Iterator over the fonts in a [`FontSet`].
pub struct Iter<'fs, 'pat> {
    fcset: &'fs FontSet<'pat>,
    index: usize,
}

impl<'fs, 'pat> Iterator for Iter<'fs, 'pat> {
    type Item = &'pat Pattern;

    fn next(&mut self) -> Option<Self::Item> {
        let fcset = self.fcset.as_ptr();
        let index = self.index;
        self.index += 1;
        if index >= unsafe { (*fcset).nfont } as usize {
            return None;
        }
        let pat = unsafe {
            let font = (*fcset).fonts.add(index);
            if font.is_null() {
                return None;
            }
            *font
        };
        if pat.is_null() {
            return None;
        }
        Some(unsafe { &*(pat as *const sys::FcPattern as *const Pattern) })
    }
}

/// Iterator over the fonts in a [`FontSet`].
pub struct IterMut<'fs, 'pat> {
    fcset: &'fs mut FontSet<'pat>,
    index: usize,
}

impl<'fs, 'pat> Iterator for IterMut<'fs, 'pat> {
    type Item = &'pat mut Pattern;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        let fcset = self.fcset.as_ptr();
        self.index += 1;
        if index >= unsafe { (*fcset).nfont } as usize {
            return None;
        }
        let pat = unsafe {
            let font = (*fcset).fonts.add(index);
            if font.is_null() {
                return None;
            }
            *font
        };
        if pat.is_null() {
            return None;
        }
        Some(unsafe { &mut *(pat as *mut sys::FcPattern as *mut Pattern) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes;
    use crate::pattern::OwnedPattern;

    #[test]
    fn fontset_new() {
        let fontset = FontSet::new();
        assert_eq!(fontset.len(), 0);
    }

    #[test]
    fn fontset_iter() {
        let mut fontset = FontSet::new();
        let mut pat = OwnedPattern::default();
        pat.add(&attributes::FC_FAMILY, "sans-serif".to_string());
        fontset.push(pat);
        assert_eq!(fontset.len(), 1);
        let mut c = 0;
        for pat in fontset.iter() {
            c += 1;
            // pat.add(&attributes::FC_DPI, 10.);  // this should be failed.
            assert_eq!(pat.get(&attributes::FC_FAMILY, 0), Some("sans-serif"));
        }

        assert_eq!(c, fontset.len());
    }

    #[test]
    fn fontset_iter_mut() {
        let mut fontset = FontSet::new();
        let mut pat = OwnedPattern::new();
        pat.add(&attributes::FC_FAMILY, "sans-serif".to_string());
        fontset.push(pat);
        assert_eq!(fontset.len(), 1);
        let mut c = 0;
        for pat in fontset.iter_mut() {
            c += 1;
            assert_eq!(pat.get(&attributes::FC_DPI, 0), None);
            assert!(pat.add(&attributes::FC_DPI, 20.));
            assert_eq!(pat.get(&attributes::FC_FAMILY, 0), Some("sans-serif"));
            assert_eq!(pat.get(&attributes::FC_DPI, 0), Some(20.));
        }

        assert_eq!(c, fontset.len());
    }
}
