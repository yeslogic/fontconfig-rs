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
    /// Delete a property from a pattern
    pub fn del(&mut self, name: &CStr) -> bool {
        FcTrue == unsafe { ffi_dispatch!(LIB, FcPatternDel, self.as_mut_ptr(), name.as_ptr()) }
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
        self.get(&attributes::FC_FULLNAME, 0)
    }

    /// Get the "file" (path on the filesystem) of this font pattern.
    pub fn filename(&self) -> Option<&str> {
        self.get(&attributes::FC_FILE, 0)
    }

    /// Get the "index" (The index of the font within the file) of this pattern.
    pub fn face_index(&self) -> Option<i32> {
        self.get(&attributes::FC_INDEX, 0)
    }

    /// Get the "slant" (Italic, oblique or roman) of this pattern.
    pub fn slant(&self) -> Option<i32> {
        self.get(&attributes::FC_SLANT, 0)
    }

    /// Get the "weight" (Light, medium, demibold, bold or black) of this pattern.
    pub fn weight(&self) -> Option<i32> {
        self.get(&attributes::FC_WEIGHT, 0)
    }

    /// Get the "width" (Condensed, normal or expanded) of this pattern.
    pub fn width(&self) -> Option<i32> {
        self.get(&attributes::FC_WIDTH, 0)
    }

    /// Get the "fontformat" ("TrueType" "Type 1" "BDF" "PCF" "Type 42" "CID Type 1" "CFF" "PFR" "Windows FNT") of this pattern.
    pub fn fontformat(&self) -> Result<FontFormat> {
        self.get(&attributes::FC_FONTFORMAT, 0)
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
    }

    ///
    pub fn get<'a, 'pat, V>(
        &'pat self,
        name: &'a attributes::Attribute<'pat, V>,
        index: usize,
    ) -> Option<V::Returns>
    where
        V: attributes::AttributeType<'pat>,
    {
        name.value_of(self, index)
    }

    ///
    pub fn add<'a, 'pat, V>(
        &'pat mut self,
        name: &'a attributes::Attribute<'pat, V>,
        value: V,
    ) -> bool
    where
        V: attributes::AttributeType<'pat>,
    {
        name.value_for(self, value)
    }
}

impl Default for OwnedPattern {
    fn default() -> Self {
        let mut pat = OwnedPattern::new();
        pat.default_substitute();
        pat
    }
}

///
pub mod attributes {
    use std::ffi::CStr;
    use std::marker::PhantomData;
    use std::ptr::NonNull;

    use fontconfig_sys as sys;

    #[cfg(feature = "dlopen")]
    use sys::statics::LIB;
    #[cfg(not(feature = "dlopen"))]
    use sys::*;

    use sys::ffi_dispatch;

    use crate::{FcFalse, FcTrue, ToResult};

    use super::Pattern;

    ///
    pub struct Attribute<'pat, V: AttributeType<'pat>> {
        name: &'static CStr,
        val: PhantomData<&'pat V>,
    }

    impl<'pat, V> Attribute<'pat, V>
    where
        V: AttributeType<'pat>,
    {
        pub(super) fn value_of<'a>(
            &'a self,
            pat: &'pat Pattern,
            index: usize,
        ) -> Option<V::Returns> {
            V::value(pat, self, index)
        }

        pub(super) fn value_for<'a>(&'a self, pat: &'pat mut Pattern, value: V) -> bool {
            value.set_to(pat, self)
        }
    }

    mod private {
        use crate::Pattern;

        use super::{Attribute, AttributeType};

        pub trait MaybeRef<'a> {
            type Returns;
        }

        pub trait Sealed<'pat>: Sized + MaybeRef<'pat> {
            fn value<'a>(
                pat: &'pat Pattern,
                attr: &'a Attribute<'pat, Self>,
                index: usize,
            ) -> Option<<Self as MaybeRef<'pat>>::Returns>
            where
                Self: AttributeType<'pat>;
            fn set_to<'a>(self, pat: &'pat mut Pattern, attr: &'a Attribute<'pat, Self>) -> bool
            where
                Self: AttributeType<'pat>;
        }
    }

    ///
    pub trait AttributeType<'pat>: private::Sealed<'pat> {}

    impl<'pat, T> AttributeType<'pat> for T where T: private::Sealed<'pat> {}

    impl<'pat> private::MaybeRef<'pat> for String {
        type Returns = &'pat str;
    }

    impl<'pat> private::Sealed<'pat> for String {
        fn value<'a>(
            pat: &'pat Pattern,
            name: &'a Attribute<'pat, Self>,
            index: usize,
        ) -> Option<Self::Returns> {
            let c_str = unsafe {
                let mut ret: *mut sys::FcChar8 = std::ptr::null_mut();
                ffi_dispatch!(
                    LIB,
                    FcPatternGetString,
                    pat.as_ptr() as *mut _,
                    name.name.as_ptr(),
                    index as i32,
                    &mut ret
                )
                .opt()?;
                if ret.is_null() {
                    return None;
                }
                CStr::from_ptr(ret as *const _)
            };
            c_str.to_str().ok()
        }

        fn set_to<'a>(mut self, pat: &'pat mut Pattern, name: &'a Attribute<'pat, Self>) -> bool {
            self.push('\0');
            let c_str = CStr::from_bytes_with_nul(self.as_bytes()).unwrap();
            FcTrue
                == unsafe {
                    ffi_dispatch!(
                        LIB,
                        FcPatternAddString,
                        pat.as_mut_ptr(),
                        name.name.as_ptr(),
                        c_str.as_ptr() as *mut _
                    )
                }
        }
    }

    impl<'a> private::MaybeRef<'a> for i32 {
        type Returns = i32;
    }

    impl<'a> private::Sealed<'a> for i32 {
        fn value(pat: &Pattern, name: &Attribute<Self>, index: usize) -> Option<Self::Returns> {
            let mut val: i32 = 0;
            unsafe {
                ffi_dispatch!(
                    LIB,
                    FcPatternGetInteger,
                    pat.as_ptr() as *mut _,
                    name.name.as_ptr(),
                    index as i32,
                    &mut val
                )
                .opt()?;
            };
            Some(val)
        }

        fn set_to(self, pat: &mut Pattern, attr: &Attribute<Self>) -> bool {
            FcTrue
                == unsafe {
                    ffi_dispatch!(
                        LIB,
                        FcPatternAddInteger,
                        pat.as_mut_ptr(),
                        attr.name.as_ptr(),
                        self
                    )
                }
        }
    }

    impl<'a> private::MaybeRef<'a> for bool {
        type Returns = bool;
    }

    impl<'a> private::Sealed<'a> for bool {
        fn value(pat: &Pattern, name: &Attribute<Self>, index: usize) -> Option<Self::Returns> {
            let mut val: i32 = 0;
            unsafe {
                ffi_dispatch!(
                    LIB,
                    FcPatternGetBool,
                    pat.as_ptr() as *mut _,
                    name.name.as_ptr(),
                    index as i32,
                    &mut val
                )
                .opt()?;
            };
            Some(val == FcTrue)
        }

        fn set_to(self, pat: &mut Pattern, attr: &Attribute<Self>) -> bool {
            FcTrue
                == unsafe {
                    ffi_dispatch!(
                        LIB,
                        FcPatternAddBool,
                        pat.as_mut_ptr(),
                        attr.name.as_ptr(),
                        if self { FcTrue } else { FcFalse }
                    )
                }
        }
    }

    impl<'a> private::MaybeRef<'a> for f64 {
        type Returns = f64;
    }

    impl<'a> private::Sealed<'a> for f64 {
        fn value(pat: &Pattern, name: &Attribute<Self>, index: usize) -> Option<Self::Returns> {
            let mut val: f64 = 0.;
            unsafe {
                ffi_dispatch!(
                    LIB,
                    FcPatternGetDouble,
                    pat.as_ptr() as *mut _,
                    name.name.as_ptr(),
                    index as i32,
                    &mut val
                )
                .opt()?;
            };
            Some(val)
        }

        fn set_to(self, pat: &mut Pattern, attr: &Attribute<Self>) -> bool {
            FcTrue
                == unsafe {
                    ffi_dispatch!(
                        LIB,
                        FcPatternAddDouble,
                        pat.as_mut_ptr(),
                        attr.name.as_ptr(),
                        self
                    )
                }
        }
    }

    impl<'a> private::MaybeRef<'a> for crate::Matrix {
        type Returns = &'a crate::Matrix;
    }

    impl<'pat> private::Sealed<'pat> for crate::Matrix {
        fn value(
            pat: &'pat Pattern,
            name: &Attribute<Self>,
            index: usize,
        ) -> Option<Self::Returns> {
            let val = unsafe {
                let mut val = std::ptr::null_mut();
                ffi_dispatch!(
                    LIB,
                    FcPatternGetMatrix,
                    pat.as_ptr() as *mut _,
                    name.name.as_ptr(),
                    index as i32,
                    &mut val
                )
                .opt()?;
                if val.is_null() {
                    return None;
                }
                &*(val as *mut crate::Matrix)
            };
            Some(val)
        }

        fn set_to(self, pat: &mut Pattern, attr: &Attribute<Self>) -> bool {
            // Safety: It copy the matrix, so it is safe to use it as a mutable pointer.
            FcTrue
                == unsafe {
                    ffi_dispatch!(
                        LIB,
                        FcPatternAddMatrix,
                        pat.as_mut_ptr(),
                        attr.name.as_ptr(),
                        &self.matrix
                    )
                }
        }
    }

    impl<'a> private::MaybeRef<'a> for crate::OwnedCharSet {
        type Returns = &'a crate::CharSet;
    }

    impl<'a> private::Sealed<'a> for crate::OwnedCharSet {
        fn value(pat: &Pattern, name: &Attribute<Self>, index: usize) -> Option<Self::Returns> {
            unsafe {
                let mut val = std::ptr::null_mut();
                ffi_dispatch!(
                    LIB,
                    FcPatternGetCharSet,
                    pat.as_ptr() as *mut _,
                    name.name.as_ptr(),
                    index as i32,
                    &mut val
                )
                .opt()?;
                if val.is_null() {
                    return None;
                }
                Some(&*(val as *const crate::CharSet))
            }
        }

        fn set_to(self, pat: &mut Pattern, attr: &Attribute<Self>) -> bool {
            // unimplemented!("set &'CharSet to pattern is unsound.");
            FcTrue
                == unsafe {
                    ffi_dispatch!(
                        LIB,
                        FcPatternAddCharSet,
                        pat.as_mut_ptr(),
                        attr.name.as_ptr(),
                        self.fcset.as_ptr()
                    )
                }
        }
    }

    impl<'a> private::MaybeRef<'a> for crate::LangSet {
        type Returns = crate::LangSet;
    }

    impl<'a> private::Sealed<'a> for crate::LangSet {
        fn value(pat: &Pattern, name: &Attribute<Self>, index: usize) -> Option<Self::Returns> {
            let val = unsafe {
                let mut val = std::ptr::null_mut();
                ffi_dispatch!(
                    LIB,
                    FcPatternGetLangSet,
                    pat.as_ptr() as *mut _,
                    name.name.as_ptr(),
                    index as i32,
                    &mut val
                )
                .opt()?;
                ffi_dispatch!(LIB, FcLangSetCopy, val)
            };
            NonNull::new(val).map(|langset| crate::LangSet { langset })
        }

        fn set_to(self, pat: &mut Pattern, attr: &Attribute<Self>) -> bool {
            FcTrue
                == unsafe {
                    ffi_dispatch!(
                        LIB,
                        FcPatternAddLangSet,
                        pat.as_mut_ptr(),
                        attr.name.as_ptr(),
                        self.langset.as_ptr()
                    )
                }
        }
    }

    macro_rules! attribute {
        ($bytes:literal, $name:ident, $vtype:ty, $comment:literal) => {
            /// $comment
            pub const $name: Attribute<$vtype> = Attribute {
                name: unsafe { &*($bytes as *const [u8] as *const CStr) },
                val: PhantomData,
            };
        };
    }

    attribute!(b"family\0", FC_FAMILY, String, "Font family names");
    attribute!(
        b"familylang\0",
        FC_FAMILYLANG,
        String,
        "Language corresponding to each family name"
    );
    attribute!(
        b"style\0",
        FC_STYLE,
        String,
        "Font style. Overrides weight and slant"
    );
    attribute!(
        b"stylelang\0",
        FC_STYLELANG,
        String,
        "Language corresponding to each style name"
    );
    attribute!(
        b"fullname\0",
        FC_FULLNAME,
        String,
        "Font face full name where different from family and family + style"
    );
    attribute!(
        b"fullnamelang\0",
        FC_FULLNAMELANG,
        String,
        "Language corresponding to each fullname"
    );
    attribute!(b"slant\0", FC_SLANT, i32, "Italic, oblique or roman");
    attribute!(
        b"weight\0",
        FC_WEIGHT,
        i32,
        "Light, medium, demibold, bold or black"
    );
    attribute!(b"width\0", FC_WIDTH, i32, "Condensed, normal or expanded");
    attribute!(b"size\0", FC_SIZE, f64, "Point size");
    attribute!(
        b"aspect\0",
        FC_ASPECT,
        f64,
        "Stretches glyphs horizontally before hinting"
    );
    attribute!(b"pixelsize\0", FC_PIXEL_SIZE, f64, "Pixel size");
    attribute!(
        b"spacing\0",
        FC_SPACING,
        i32,
        "Proportional, dual-width, monospace or charcell"
    );
    attribute!(b"foundry\0", FC_FOUNDRY, String, "Font foundry name");
    attribute!(
        b"antialias\0",
        FC_ANTIALIAS,
        bool,
        "Whether glyphs can be antialiased"
    );
    attribute!(
        b"hintstyle\0",
        FC_HINT_STYLE,
        i32,
        "Automatic hinting style"
    );
    attribute!(
        b"hinting\0",
        FC_HINTING,
        bool,
        "Whether the rasterizer should use hinting"
    );
    attribute!(
        b"verticallayout\0",
        FC_VERTICAL_LAYOUT,
        bool,
        "Use vertical layout"
    );
    attribute!(
        b"autohint\0",
        FC_AUTOHINT,
        bool,
        "Use autohinter instead of normal hinter"
    );
    attribute!(
        b"globaladvance\0",
        FC_GLOBAL_ADVANCE,
        bool,
        "Use font global advance data (deprecated)"
    );
    attribute!(
        b"file\0",
        FC_FILE,
        String,
        "The filename holding the font relative to the config's sysroot"
    );
    attribute!(
        b"index\0",
        FC_INDEX,
        i32,
        "The index of the font within the file"
    );
    // attribute!(
    //     b"ftface\0",
    //     FC_FT_FACE,
    //     FT_Face,
    //     "Use the specified FreeType face object"
    // );
    attribute!(
        b"rasterizer\0",
        FC_RASTERIZER,
        String,
        "Which rasterizer is in use (deprecated)"
    );
    attribute!(
        b"outline\0",
        FC_OUTLINE,
        bool,
        "Whether the glyphs are outlines"
    );
    attribute!(
        b"scalable\0",
        FC_SCALABLE,
        bool,
        "Whether glyphs can be scaled"
    );
    attribute!(b"dpi\0", FC_DPI, f64, "Target dots per inch");
    attribute!(
        b"rgba\0",
        FC_RGBA,
        i32,
        "unknown, rgb, bgr, vrgb, vbgr, none - subpixel geometry"
    );
    attribute!(
        b"scale\0",
        FC_SCALE,
        f64,
        "Scale factor for point->pixel conversions (deprecated)"
    );
    attribute!(
        b"minspace\0",
        FC_MINSPACE,
        bool,
        "Eliminate leading from line spacing"
    );
    attribute!(
        b"charset\0",
        FC_CHARSET,
        crate::OwnedCharSet,
        "Unicode chars encoded by the font"
    );
    attribute!(
        b"lang\0",
        FC_LANG,
        crate::LangSet,
        "Set of RFC-3066-style languages this font supports"
    );
    attribute!(
        b"fontversion\0",
        FC_FONTVERSION,
        i32,
        "Version number of the font"
    );
    attribute!(
        b"capability\0",
        FC_CAPABILITY,
        String,
        "List of layout capabilities in the font"
    );
    attribute!(
        b"fontformat\0",
        FC_FONTFORMAT,
        String,
        "String name of the font format"
    );
    attribute!(
        b"embolden\0",
        FC_EMBOLDEN,
        bool,
        "Rasterizer should synthetically embolden the font"
    );
    attribute!(
        b"embeddedbitmap\0",
        FC_EMBEDDED_BITMAP,
        bool,
        "Use the embedded bitmap instead of the outline"
    );
    attribute!(
        b"decorative\0",
        FC_DECORATIVE,
        bool,
        "Whether the style is a decorative variant"
    );
    attribute!(b"lcdfilter\0", FC_LCD_FILTER, i32, "Type of LCD filter");
    attribute!(
        b"namelang\0",
        FC_NAMELANG,
        String,
        "Language name to be used for the default value of familylang, stylelang and fullnamelang"
    );
    attribute!(
        b"fontfeatures\0",
        FC_FONT_FEATURES,
        String,
        "List of extra feature tags in OpenType to be enabled"
    );
    attribute!(
        b"prgname\0",
        FC_PRGNAME,
        String,
        "Name of the running program"
    );
    attribute!(
        b"hash\0",
        FC_HASH,
        String,
        "SHA256 hash value of the font data with \"sha256:\" prefix (deprecated)"
    );
    attribute!(
        b"postscriptname\0",
        FC_POSTSCRIPT_NAME,
        String,
        "Font name in PostScript"
    );
    attribute!(
        b"symbol\0",
        FC_SYMBOL,
        bool,
        "Whether font uses MS symbol-font encoding"
    );
    attribute!(b"color\0", FC_COLOR, bool, "Whether any glyphs have color");
    attribute!(
        b"fontvariations\0",
        FC_FONT_VARIATIONS,
        String,
        "comma-separated string of axes in variable font"
    );
    attribute!(
        b"variable\0",
        FC_VARIABLE,
        bool,
        "Whether font is Variable Font"
    );
    attribute!(
        b"fonthashint\0",
        FC_FONT_HAS_HINT,
        bool,
        "Whether font has hinting"
    );
    attribute!(b"order\0", FC_ORDER, i32, "Order number of the font");
}

#[cfg(test)]
mod tests {
    use crate::pattern::attributes as attrs;

    #[test]
    fn test_into_inner() {
        let mut pat = super::OwnedPattern::new();
        pat.add(&attrs::FC_FAMILY, "nomospace".to_string());
        let pat = pat.into_inner();
        let pat = pat as *mut super::Pattern;
        assert_eq!(
            unsafe { &*pat }.get(&attrs::FC_FAMILY, 0),
            Some("nomospace")
        );
    }

    #[test]
    fn test_get_family() {
        let pat = super::OwnedPattern::new();
        assert!(pat.get(&attrs::FC_FAMILY, 0).is_none());
    }

    #[test]
    fn test_get_family_exists() {
        let mut pat = super::OwnedPattern::default();
        pat.add(&attrs::FC_FAMILY, "nomospace".to_string());
        assert!(pat.get(&attrs::FC_FAMILY, 0).is_some());
    }

    #[test]
    fn test_get_filepath() {
        let mut cfg = crate::FontConfig::default();
        let mut pat = super::OwnedPattern::default();
        pat.add(&attrs::FC_FAMILY, "nomospace".to_string());
        cfg.substitute(&mut pat, crate::MatchKind::Pattern);
        let pat = pat.font_match(&mut cfg);
        let file = pat.get(&attrs::FC_FILE, 0);
        assert!(file.is_some());
        let dpi = pat.get(&attrs::FC_DPI, 0);
        println!("{:?}", dpi);
        if let Some(file) = file {
            assert!(file.starts_with("/usr/share/fonts"));
            assert!(std::path::Path::new(&file).exists(), "{}", file);
        }
    }
}
