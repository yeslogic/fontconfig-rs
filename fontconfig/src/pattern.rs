//!
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
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

use crate::{
    CharSet, Error, FcFalse, FcStr, FcTrue, FontConfig, FontFormat, FontSet, LangSet, Matrix,
    ObjectSet, Result, ToResult,
};

/// A safe wrapper around fontconfig's `FcPattern`.
pub struct Pattern {
    /// Raw pointer to `FcPattern`
    pub(crate) pat: NonNull<FcPattern>,
}

impl Pattern {
    /// Create a new empty `Pattern`.
    pub fn new() -> Pattern {
        let pat = unsafe { ffi_dispatch!(LIB, FcPatternCreate,) };
        assert!(!pat.is_null());

        Pattern {
            pat: NonNull::new(pat).expect("out of memory"),
        }
    }

    /// Create a `Pattern` from a raw fontconfig FcPattern pointer.
    ///
    /// The pattern is referenced.
    ///
    /// **Panic:** If the pointer is null.
    ///
    /// **Safety:** The pattern pointer must be valid/non-null.
    // pub unsafe fn from_raw(pat: *mut FcPattern) -> Pattern {
    //     // ffi_dispatch!(LIB, FcPatternReference, pat);
    //
    //     Pattern {
    //         pat: NonNull::new(pat).unwrap(),
    //     }
    // }

    /// Add a key-value pair to this pattern.
    ///
    /// See useful keys in the [fontconfig reference][1].
    ///
    /// [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/x19.html
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
                self.pat.as_ptr(),
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
                self.pat.as_ptr(),
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
            ffi_dispatch!(LIB, FcPatternPrint, self.pat.as_ptr());
        }
    }

    /// Filter the objects of pattern
    ///
    /// Returns a new pattern that only has those objects from p that are in os.
    /// If os is None, a duplicate of p is returned.
    pub fn filter(&self, os: Option<&mut ObjectSet>) -> Option<Self> {
        let os = os.map(|o| o.as_mut_ptr()).unwrap_or(ptr::null_mut());
        let pat = unsafe {
            let pat = ffi_dispatch!(LIB, FcPatternFilter, self.pat.as_ptr(), os);
            if pat.is_null() {
                return None;
            }
            pat
        };
        NonNull::new(pat).map(|pat| Pattern { pat })
    }

    /// Format a pattern into a string according to a format specifier
    ///
    /// See [pattern-format](https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcpatternformat.html)
    pub fn format(&self, fmt: &CStr) -> FcStr {
        unsafe {
            let s = ffi_dispatch!(
                LIB,
                FcPatternFormat,
                self.pat.as_ptr(),
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
            ffi_dispatch!(LIB, FcDefaultSubstitute, self.pat.as_mut());
        }
    }

    /// Get the best available match for this pattern, returned as a new pattern.
    pub fn font_match(&mut self, config: &mut FontConfig) -> Pattern {
        // self.default_substitute();
        // config.substitute(self, MatchKind::Pattern);

        unsafe {
            let mut res = sys::FcResultNoMatch;
            let pat = ffi_dispatch!(
                LIB,
                FcFontMatch,
                config.as_mut_ptr(),
                self.pat.as_mut(),
                &mut res
            );
            res.ok().unwrap();
            Pattern {
                pat: NonNull::new(pat).unwrap(),
            }
        }
    }

    /// Get the list of fonts sorted by closeness to self.
    /// If trim is `true`, elements in the list which don't include Unicode coverage not provided by earlier elements in the list are elided.
    pub fn font_sort(&mut self, config: &mut FontConfig, trim: bool) -> Option<FontSet> {
        // self.default_substitute();
        // config.substitute(self, MatchKind::Pattern);
        let mut pat = self.clone();
        unsafe {
            // What is this result actually used for? Seems redundant with
            // return type.
            let mut res = sys::FcResultNoMatch;

            let mut charsets: *mut _ = ptr::null_mut();

            let fcset = ffi_dispatch!(
                LIB,
                FcFontSort,
                config.as_mut_ptr(),
                pat.as_mut_ptr(),
                if trim { FcTrue } else { FcFalse }, // Trim font list.
                &mut charsets,
                &mut res
            );
            res.opt()?;
            Some(FontSet {
                fcset: NonNull::new(fcset).unwrap(),
            })
        }
    }

    /// Prepare pattern for loading font file.
    ///
    /// Creates a new pattern consisting of elements of font not appearing in pat,
    /// elements of pat not appearing in font and the best matching value from pat for elements appearing in both.
    /// The result is passed to FcConfigSubstituteWithPat with kind FcMatchFont and then returned.
    #[doc(alias = "FcFontRenderPrepare")]
    pub fn render_prepare(&self, font: &Self) -> Self {
        let pat = unsafe {
            ffi_dispatch!(
                LIB,
                FcFontRenderPrepare,
                ptr::null_mut(),
                self.pat.as_ptr(),
                font.pat.as_ptr()
            )
        };
        Pattern {
            pat: NonNull::new(pat).unwrap(),
        }
    }

    /// Get character map
    #[doc(alias = "FcPatternGetCharSet")]
    pub fn charset(&self) -> Option<CharSet> {
        unsafe {
            let mut charsets = ffi_dispatch!(LIB, FcCharSetCreate,);
            ffi_dispatch!(
                LIB,
                FcPatternGetCharSet,
                self.pat.as_ptr(),
                FC_CHARSET.as_ptr(),
                0,
                &mut charsets
            );
            if charsets.is_null() {
                None
            } else {
                let charsets = ffi_dispatch!(LIB, FcCharSetCopy, charsets);
                NonNull::new(charsets).map(|fcset| CharSet {
                    fcset,
                    _marker: PhantomData,
                })
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

    /// Returns a raw pointer to underlying `FcPattern`.
    pub fn as_ptr(&self) -> *const FcPattern {
        self.pat.as_ptr()
    }

    /// Returns an unsafe mutable pointer to the underlying `FcPattern`.
    pub fn as_mut_ptr(&mut self) -> *mut FcPattern {
        self.pat.as_ptr()
    }
}

impl FromStr for Pattern {
    type Err = Error;
    /// Converts `name` from the standard text format described above into a pattern.
    fn from_str(s: &str) -> Result<Self> {
        let c_str = CString::new(s).unwrap();
        unsafe {
            let pat = ffi_dispatch!(LIB, FcNameParse, c_str.as_ptr().cast());
            if let Some(pat) = NonNull::new(pat) {
                Ok(Pattern { pat })
            } else {
                Err(Error::OutOfMemory)
            }
        }
    }
}

impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fcstr = unsafe { ffi_dispatch!(LIB, FcNameUnparse, self.pat.as_ptr()) };
        let fcstr = unsafe { CStr::from_ptr(fcstr as *const c_char) };
        let result = write!(f, "{:?}", fcstr);
        unsafe { ffi_dispatch!(LIB, FcStrFree, fcstr.as_ptr() as *mut u8) };
        result
    }
}

impl Clone for Pattern {
    fn clone(&self) -> Self {
        let cloned = unsafe { ffi_dispatch!(LIB, FcPatternDuplicate, self.pat.as_ptr()) };
        Pattern {
            pat: NonNull::new(cloned).unwrap(),
        }
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(LIB, FcPatternDestroy, self.pat.as_ptr());
        }
    }
}

impl Pattern {
    /// Get the languages set of this pattern.
    pub fn lang_set(&mut self) -> Option<LangSet> {
        // let mut langset = LangSet::new();
        let langset = unsafe {
            let mut langset = ffi_dispatch!(LIB, FcLangSetCreate,);
            ffi_dispatch!(
                LIB,
                FcPatternGetLangSet,
                self.as_mut_ptr(),
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
    pub fn matrix(&mut self) -> Option<Matrix> {
        let mut matrix = Matrix::new();
        unsafe {
            ffi_dispatch!(
                LIB,
                FcPatternGetMatrix,
                self.as_mut_ptr(),
                FC_MATRIX.as_ptr(),
                0,
                &mut matrix.as_mut_ptr()
            )
            .opt()?;
        }
        Some(matrix)
    }
}

impl Default for Pattern {
    fn default() -> Self {
        Pattern::new()
    }
}
