//!
use std::borrow::{Borrow, BorrowMut};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_char;
use std::ptr::{self, NonNull};
use std::str::FromStr;

///!
use fontconfig_sys as sys;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

use sys::constants::*;
use sys::{ffi_dispatch, FcPattern};

use crate::charset::OwnedCharSet;
use crate::{
    CharSet, Error, FcFalse, FcStr, FcTrue, FontConfig, FontFormat, FontSet, LangSet, Matrix,
    ObjectSet, Result, ToResult,
};

/// Representation of a borrowed fontconfig's [`sys::FcPattern`].
///
/// An `Pattern` is an opaque type that holds both patterns to match against the available fonts, as well as the information about each font.
#[doc(alias = "FcPattern")]
#[repr(transparent)]
pub struct Pattern(FcPattern);

/// A type representing an owned fontconfig's [`sys::FcPattern`].
#[doc(alias = "FcPattern")]
#[repr(transparent)]
pub struct OwnedPattern {
    /// Raw pointer to `FcPattern`
    pub(crate) pat: NonNull<FcPattern>,
}

impl OwnedPattern {
    /// Create a new empty [`OwnedPattern`].
    pub fn new() -> OwnedPattern {
        let pat = unsafe { ffi_dispatch!(LIB, FcPatternCreate,) };
        assert!(!pat.is_null());

        OwnedPattern {
            pat: NonNull::new(pat).expect("out of memory"),
        }
    }

    pub(crate) fn into_inner(self) -> *mut FcPattern {
        let ptr = self.pat.as_ptr() as *mut FcPattern;
        std::mem::forget(self);
        ptr
    }
}

impl Pattern {
    /// Add a key-value pair to this pattern.
    ///
    /// See useful keys in the [fontconfig reference](http://www.freedesktop.org/software/fontconfig/fontconfig-devel/x19.html)
    pub fn add_string(&mut self, name: &CStr, val: &CStr) {
        unsafe {
            ffi_dispatch!(
                LIB,
                FcPatternAddString,
                self.as_mut_ptr(),
                name.as_ptr(),
                val.as_ptr() as *const u8
            );
        }
    }

    /// Delete a property from a pattern
    pub fn del(&mut self, name: &CStr) -> bool {
        FcTrue == unsafe { ffi_dispatch!(LIB, FcPatternDel, self.as_mut_ptr(), name.as_ptr()) }
    }

    /// Get string the value for a key from this pattern.
    pub fn string<'a>(&'a self, name: &'a CStr) -> Option<&'a str> {
        unsafe {
            let mut ret: *mut sys::FcChar8 = ptr::null_mut();
            ffi_dispatch!(
                LIB,
                FcPatternGetString,
                self.as_ptr() as *mut FcPattern,
                name.as_ptr(),
                0,
                &mut ret as *mut _
            )
            .opt()?;
            let cstr = CStr::from_ptr(ret as *const c_char);
            Some(cstr.to_str().unwrap())
        }
    }

    /// Get the integer value for a key from this pattern.
    pub fn int(&self, name: &CStr) -> Option<i32> {
        unsafe {
            let mut ret: i32 = 0;
            ffi_dispatch!(
                LIB,
                FcPatternGetInteger,
                self.as_ptr() as *mut FcPattern,
                name.as_ptr(),
                0,
                &mut ret as *mut i32
            )
            .opt()?;
            Some(ret)
        }
    }

    /// Print this pattern to stdout with all its values.
    pub fn print(&self) {
        unsafe {
            ffi_dispatch!(LIB, FcPatternPrint, self.as_ptr());
        }
    }

    /// Filter the objects of pattern
    ///
    /// Returns a new pattern that only has those objects from `self` that are in os.
    /// If os is None, a duplicate of `self` is returned.
    pub fn filter(&self, os: Option<&mut ObjectSet>) -> Option<OwnedPattern> {
        let os = os.map(|o| o.as_mut_ptr()).unwrap_or(ptr::null_mut());
        let pat = unsafe {
            let pat = ffi_dispatch!(LIB, FcPatternFilter, self.as_ptr() as *mut FcPattern, os);
            if pat.is_null() {
                return None;
            }
            pat
        };
        NonNull::new(pat).map(|pat| OwnedPattern { pat })
    }

    /// Format a pattern into a string according to a format specifier
    ///
    /// See: [pattern-format](https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternformat.html)
    pub fn format(&self, fmt: &CStr) -> Option<FcStr> {
        unsafe {
            let s = ffi_dispatch!(
                LIB,
                FcPatternFormat,
                self.as_ptr() as *mut FcPattern,
                fmt.as_ptr() as *const u8
            );
            FcStr::from_ptr(s)
        }
    }

    /// Perform default substitutions in a pattern
    ///
    /// Supplies default values for underspecified font patterns:
    ///
    /// * Patterns without a specified style or weight are set to Medium   
    /// * Patterns without a specified style or slant are set to Roman   
    /// * Patterns without a specified pixel size are given one computed from any specified point size (default 12), dpi (default 75) and scale (default 1).  
    pub fn default_substitute(&mut self) {
        unsafe {
            ffi_dispatch!(LIB, FcDefaultSubstitute, self.as_mut_ptr());
        }
    }

    /// Get the best available match for this pattern, returned as a new pattern.
    ///
    /// Finds the font in sets most closely matching pattern and returns the result of [`Pattern::render_prepare`] for that font and the provided pattern.   
    /// This function should be called only after [`FontConfig::substitute`] and [`Pattern::default_substitute`] have been called for the pattern.    
    /// otherwise the results will not be correct.
    #[doc(alias = "FcFontMatch")]
    pub fn font_match(&mut self, config: &mut FontConfig) -> OwnedPattern {
        // self.default_substitute();
        // config.substitute(self, MatchKind::Pattern);

        unsafe {
            let mut res = sys::FcResultNoMatch;
            let pat = ffi_dispatch!(
                LIB,
                FcFontMatch,
                config.as_mut_ptr(),
                self.as_mut_ptr(),
                &mut res
            );
            res.ok().unwrap();
            OwnedPattern {
                pat: NonNull::new(pat).unwrap(),
            }
        }
    }

    /// List fonts
    ///
    /// Selects fonts matching `self`,
    /// creates patterns from those fonts containing only the objects in os and returns the set of unique such patterns.    
    pub fn font_list(&self, config: &mut FontConfig, os: Option<&mut ObjectSet>) -> FontSet<'_> {
        let os = os.map(|o| o.as_mut_ptr()).unwrap_or(ptr::null_mut());
        let set = unsafe {
            ffi_dispatch!(
                LIB,
                FcFontList,
                config.as_mut_ptr(),
                self.as_ptr() as *mut _,
                os
            )
        };
        // NOTE: Referenced by FontSet, should not drop it.
        FontSet {
            fcset: NonNull::new(set).unwrap(),
            _marker: PhantomData,
        }
    }

    /// Get the list of fonts sorted by closeness to self.
    ///
    /// If trim is `true`, elements in the list which don't include Unicode coverage not provided by earlier elements in the list are elided.    
    /// This function should be called only after [`FontConfig::substitute`] and [`Pattern::default_substitute`] have been called for this pattern;    
    /// otherwise the results will not be correct.
    pub fn font_sort(&mut self, config: &mut FontConfig, trim: bool) -> Result<FontSet<'static>> {
        unsafe {
            // What is this result actually used for? Seems redundant with
            // return type.
            let mut res = sys::FcResultNoMatch;

            let mut charsets: *mut _ = ptr::null_mut();

            let fcset = ffi_dispatch!(
                LIB,
                FcFontSort,
                config.as_mut_ptr(),
                self.as_ptr() as *mut _,
                if trim { FcTrue } else { FcFalse }, // Trim font list.
                &mut charsets,
                &mut res
            );
            res.ok()?;
            if fcset.is_null() {
                return Err(Error::OutOfMemory);
            }
            let fcset = NonNull::new_unchecked(fcset);
            Ok(FontSet {
                fcset,
                _marker: PhantomData,
            })
        }
    }

    /// Get the list of fonts sorted by closeness to self.
    ///
    /// If trim is `true`, elements in the list which don't include Unicode coverage not provided by earlier elements in the list are elided.    
    /// The union of Unicode coverage of all of the fonts is returned in [`CharSet`].   
    /// This function should be called only after [`FontConfig::substitute`] and [`Pattern::default_substitute`] have been called for this pattern;    
    /// otherwise the results will not be correct.
    pub fn font_sort_with_charset(
        &mut self,
        config: &mut FontConfig,
        trim: bool,
    ) -> Option<(FontSet<'_>, OwnedCharSet)> {
        // self.default_substitute();
        // config.substitute(self, MatchKind::Pattern);
        unsafe {
            // What is this result actually used for? Seems redundant with
            // return type.
            let mut res = sys::FcResultNoMatch;

            let mut charsets: *mut _ = ptr::null_mut();

            let fcset = ffi_dispatch!(
                LIB,
                FcFontSort,
                config.as_mut_ptr(),
                self.as_mut_ptr(),
                if trim { FcTrue } else { FcFalse }, // Trim font list.
                &mut charsets,
                &mut res
            );
            res.opt()?;
            Some((
                FontSet {
                    fcset: NonNull::new(fcset).unwrap(),
                    _marker: PhantomData,
                },
                OwnedCharSet {
                    fcset: NonNull::new(charsets).unwrap(),
                },
            ))
        }
    }

    /// Prepare pattern for loading font file.
    ///
    /// Creates a new pattern consisting of elements of font not appearing in pat,
    /// elements of pat not appearing in font and the best matching value from pat for elements appearing in both.    
    /// The result is passed to [`FontConfig::substitute_with_pat`] with kind [`crate::MatchKind::Font`] and then returned.
    #[doc(alias = "FcFontRenderPrepare")]
    pub fn render_prepare(&mut self, config: &mut FontConfig, font: &mut Self) -> OwnedPattern {
        let pat = unsafe {
            ffi_dispatch!(
                LIB,
                FcFontRenderPrepare,
                config.as_mut_ptr(),
                self.as_mut_ptr(),
                font.as_mut_ptr()
            )
        };
        OwnedPattern {
            pat: NonNull::new(pat).unwrap(),
        }
    }

    /// Get character map
    #[doc(alias = "FcPatternGetCharSet")]
    pub fn charset(&self) -> Option<&CharSet> {
        unsafe {
            let mut charsets = ffi_dispatch!(LIB, FcCharSetCreate,);
            ffi_dispatch!(
                LIB,
                FcPatternGetCharSet,
                self.as_ptr() as *mut _,
                FC_CHARSET.as_ptr(),
                0,
                &mut charsets
            );
            if charsets.is_null() {
                None
            } else {
                Some(&*(charsets as *const CharSet))
            }
        }
    }

    /// Get the "fullname" (human-readable name) of this pattern.
    pub fn name(&self) -> Option<&str> {
        self.string(FC_FULLNAME.as_cstr())
    }

    /// Get the "file" (path on the filesystem) of this font pattern.
    pub fn filename(&self) -> Option<&str> {
        self.string(FC_FILE.as_cstr())
    }

    /// Get the "index" (The index of the font within the file) of this pattern.
    pub fn face_index(&self) -> Option<i32> {
        self.int(FC_INDEX.as_cstr())
    }

    /// Get the "slant" (Italic, oblique or roman) of this pattern.
    pub fn slant(&self) -> Option<i32> {
        self.int(FC_SLANT.as_cstr())
    }

    /// Get the "weight" (Light, medium, demibold, bold or black) of this pattern.
    pub fn weight(&self) -> Option<i32> {
        self.int(FC_WEIGHT.as_cstr())
    }

    /// Get the "width" (Condensed, normal or expanded) of this pattern.
    pub fn width(&self) -> Option<i32> {
        self.int(FC_WIDTH.as_cstr())
    }

    /// Get the "fontformat" ("TrueType" "Type 1" "BDF" "PCF" "Type 42" "CID Type 1" "CFF" "PFR" "Windows FNT") of this pattern.
    pub fn fontformat(&self) -> Result<FontFormat> {
        self.string(FC_FONTFORMAT.as_cstr())
            .ok_or_else(|| Error::UnknownFontFormat(String::new()))
            .and_then(|format| format.parse())
    }

    /// Returns a raw pointer to underlying [`sys::FcPattern`].
    pub(crate) fn as_ptr(&self) -> *const FcPattern {
        self as *const _ as *const FcPattern
    }

    /// Returns an unsafe mutable pointer to the underlying [`sys::FcPattern`].
    pub(crate) fn as_mut_ptr(&mut self) -> *mut FcPattern {
        self as *mut _ as *mut FcPattern
    }
}

impl ToOwned for Pattern {
    type Owned = OwnedPattern;

    fn to_owned(&self) -> OwnedPattern {
        OwnedPattern {
            pat: NonNull::new(unsafe { ffi_dispatch!(LIB, FcPatternDuplicate, self.as_ptr()) })
                .unwrap(),
        }
    }
}

impl Borrow<Pattern> for OwnedPattern {
    fn borrow(&self) -> &Pattern {
        unsafe { &*(self.as_ptr() as *const Pattern) }
    }
}

impl BorrowMut<Pattern> for OwnedPattern {
    fn borrow_mut(&mut self) -> &mut Pattern {
        unsafe { &mut *(self.as_mut_ptr() as *mut Pattern) }
    }
}

impl FromStr for OwnedPattern {
    type Err = Error;
    /// Converts `name` from the standard text format described above into a pattern.
    fn from_str(s: &str) -> Result<Self> {
        let c_str = CString::new(s).unwrap();
        unsafe {
            let pat = ffi_dispatch!(LIB, FcNameParse, c_str.as_ptr().cast());
            if let Some(pat) = NonNull::new(pat) {
                Ok(OwnedPattern { pat })
            } else {
                Err(Error::OutOfMemory)
            }
        }
    }
}

impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fcstr = unsafe { ffi_dispatch!(LIB, FcNameUnparse, self.as_ptr() as *mut FcPattern) };
        let fcstr = unsafe { CStr::from_ptr(fcstr as *const c_char) };
        let result = write!(f, "{:?}", fcstr);
        unsafe { ffi_dispatch!(LIB, FcStrFree, fcstr.as_ptr() as *mut u8) };
        result
    }
}

impl Clone for OwnedPattern {
    fn clone(&self) -> Self {
        let cloned = unsafe { ffi_dispatch!(LIB, FcPatternDuplicate, self.pat.as_ptr()) };
        OwnedPattern {
            pat: NonNull::new(cloned).unwrap(),
        }
    }
}

impl Drop for OwnedPattern {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(LIB, FcPatternDestroy, self.pat.as_ptr());
        }
    }
}

impl Deref for OwnedPattern {
    type Target = Pattern;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.pat.as_ptr() as *const _) }
    }
}

impl DerefMut for OwnedPattern {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.pat.as_ptr() as *mut _) }
    }
}

impl AsRef<Pattern> for OwnedPattern {
    fn as_ref(&self) -> &Pattern {
        self
    }
}

impl AsMut<Pattern> for OwnedPattern {
    fn as_mut(&mut self) -> &mut Pattern {
        self
    }
}

impl Pattern {
    /// Get the languages set of this pattern.
    pub fn lang_set(&self) -> Option<LangSet> {
        // let mut langset = LangSet::new();
        let langset = unsafe {
            let mut langset = ffi_dispatch!(LIB, FcLangSetCreate,);
            ffi_dispatch!(
                LIB,
                FcPatternGetLangSet,
                self.as_ptr() as *mut _,
                FC_LANG.as_ptr(),
                0,
                &mut langset
            )
            .opt()?;
            ffi_dispatch!(LIB, FcLangSetCopy, langset)
        };
        NonNull::new(langset).map(|langset| LangSet { langset })
    }

    /// Get the matrix from this pattern.
    pub fn matrix(&mut self) -> Option<&Matrix> {
        let mut matrix = ptr::null_mut();
        unsafe {
            ffi_dispatch!(
                LIB,
                FcPatternGetMatrix,
                self.as_mut_ptr(),
                FC_MATRIX.as_ptr(),
                0,
                &mut matrix
            )
            .opt()?;
            if matrix.is_null() {
                None
            } else {
                Some(&*(matrix as *mut crate::Matrix))
            }
        }
        Some(matrix)
    }
}

impl Default for OwnedPattern {
    fn default() -> Self {
        let mut pat = OwnedPattern::new();
        pat.default_substitute();
        pat
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_into_inner() {
        let mut pat = super::OwnedPattern::new();
        pat.add_string(
            crate::FC_FAMILY.as_cstr(),
            &std::ffi::CString::new("nomospace").unwrap(),
        );
        let pat = pat.into_inner();
        let pat = pat as *mut super::Pattern;
        assert_eq!(
            unsafe { &*pat }.string(crate::FC_FAMILY.as_cstr()),
            Some("nomospace")
        );
    }
}
